//! El cliente del medio contra lo que contestó omniwitness (RFC-0013 E3a, §848).
//!
//! `tests/vectores/testigos/omniwitness.json` es lo que grabó `rehacer_omniwitness.sh` de ese
//! directorio: diez pasos del publicador contra dos `omniwitness` LOCALES de
//! `github.com/transparency-dev/witness` en `5397155`, sin tocar, con el log `0x06` del medio
//! configurado a mano (`--additional_logs`) —omniwitness-a con una clave ML-DSA-44, y
//! omniwitness-b con una ML-DSA-44 y detrás una Ed25519 del mismo nombre—, cada intercambio
//! HTTP tal cual. El canon no corre Go: vuelve a pasar el cliente por lo grabado y comprueba
//! aquí, en cada pasada:
//!
//! - que la nota de cada paso **sale byte a byte** firmando aquí en determinista con la misma
//!   semilla y la misma marca;
//! - que cada petición que arma el cliente es, **byte a byte**, la que se le envió a
//!   omniwitness y que éste aceptó o rechazó: la línea `old`, la prueba de consistencia y la
//!   nota;
//! - que con lo que contestó el cliente llega al mismo resultado, paso a paso y testigo a
//!   testigo —el `404`, la primera cofirma, el `409` y su reintento, el `409` con un tamaño
//!   mayor que el de la nota, los dos `422` y el `403`—, sin pedir ni una petición más de las
//!   grabadas;
//! - que cada cofirma ML-DSA-44 de omniwitness verifica aquí con su `vkey`, y la Ed25519 de
//!   omniwitness-b es de su otra clave, y el cliente la ignora;
//! - que la `vkey` ML-DSA-44 de cada testigo, que sacó de su semilla el ML-DSA de Go
//!   (`filippo.io/mldsa`, el de `formats`), es la que saca de la misma semilla el de aquí;
//! - que la nota cofirmada sale igual, y el sobre que el kit dio VERDE en el banco lleva esa
//!   nota.

use std::collections::VecDeque;
use std::path::PathBuf;

use serde_json::Value;
use zk_ssl_medio::base64::base64_decode;
use zk_ssl_medio::hash::HashValue;
use zk_ssl_medio::medio::{hoja, verificar_inclusion, ArbolDelMedio};
use zk_ssl_medio::nota::{verificar_nota, Checkpoint, ClaveDeNota, Cofirmante, Publicador};
use zk_ssl_medio::testigos::{
    pedir_cofirma, ErrorDeTestigo, RespuestaHttp, Resultado, Retencion, Testigo, Transporte,
};

const A: &str = "testigo.invalid/omniwitness-a";
const B: &str = "testigo.invalid/omniwitness-b";

fn grabacion() -> Value {
    let ruta =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectores/testigos/omniwitness.json");
    serde_json::from_str(&std::fs::read_to_string(ruta).unwrap()).unwrap()
}

fn hex32(v: &Value) -> [u8; 32] {
    let h = v.as_str().unwrap().strip_prefix("0x").unwrap();
    assert_eq!(h.len(), 64);
    let c = |b: u8| (b as char).to_digit(16).unwrap() as u8;
    core::array::from_fn(|i| c(h.as_bytes()[2 * i]) << 4 | c(h.as_bytes()[2 * i + 1]))
}

/// La nota del paso, firmada aquí en determinista, y su árbol.
fn nota_del_paso(e: &Value) -> (String, ArbolDelMedio, Publicador) {
    let origen = e["origen"].as_str().unwrap();
    let mut arbol = ArbolDelMedio::nuevo();
    for h in e["hojas"].as_array().unwrap() {
        arbol.anadir(&hex32(h));
    }
    let publicador = Publicador::determinista(origen, hex32(&e["publicador"]["semilla"])).unwrap();
    let c = Checkpoint {
        origen: origen.to_string(),
        tamano: arbol.tamano(),
        raiz: arbol.raiz(),
    };
    let nota = publicador
        .firmar(&c, e["publicador"]["marca"].as_u64().unwrap())
        .unwrap();
    (nota, arbol, publicador)
}

/// Devuelve lo grabado, en su orden, y exige que la petición sea la grabada.
struct Grabado(VecDeque<Value>);

impl Transporte for Grabado {
    fn add_checkpoint(
        &mut self,
        _: &Testigo,
        cuerpo: &[u8],
    ) -> Result<RespuestaHttp, ErrorDeTestigo> {
        let i = self
            .0
            .pop_front()
            .expect("el cliente pide más de lo que se grabó");
        assert_eq!(
            std::str::from_utf8(cuerpo).unwrap(),
            i["peticion"].as_str().unwrap(),
            "la petición no es, byte a byte, la que se envió"
        );
        Ok(RespuestaHttp {
            estado: i["estado"].as_u64().unwrap() as u16,
            tipo: i["tipo"].as_str().map(str::to_string),
            cuerpo: i["respuesta"].as_str().unwrap().as_bytes().to_vec(),
        })
    }
}

/// El resultado, escrito como lo escribe el ejemplo `testigos_locales`.
fn como_se_grabo(r: &Result<Resultado, ErrorDeTestigo>) -> Vec<(&'static str, Value)> {
    match r {
        Ok(Resultado::Cofirmada {
            cofirma,
            viejo,
            ignoradas,
        }) => vec![
            ("resultado", "cofirmada".into()),
            ("viejo", (*viejo).into()),
            ("ignoradas", (*ignoradas).into()),
            ("marca", cofirma.marca.into()),
        ],
        Ok(Resultado::SinSuCofirma { viejo, ignoradas }) => vec![
            ("resultado", "sin-su-cofirma".into()),
            ("viejo", (*viejo).into()),
            ("ignoradas", (*ignoradas).into()),
        ],
        Ok(Resultado::Retroceso { recordado, nota }) => vec![
            ("resultado", "retroceso".into()),
            ("recordado", (*recordado).into()),
            ("nota", (*nota).into()),
        ],
        Ok(Resultado::Rechazada(r)) => vec![
            ("resultado", "rechazada".into()),
            ("respuesta", format!("{r:?}").into()),
        ],
        Err(e) => vec![
            ("resultado", "error".into()),
            ("error", e.to_string().into()),
        ],
    }
}

/// Cada paso, rehecho aquí sobre lo que contestaron los dos omniwitness.
#[test]
fn el_cliente_rehace_cada_paso_contra_lo_que_contesto_omniwitness() {
    let g = grabacion();
    let pasos = g["pasos"].as_array().unwrap();
    let nombres: Vec<&str> = pasos.iter().map(|p| p["paso"].as_str().unwrap()).collect();
    assert_eq!(
        nombres,
        [
            "desconocido",
            "primera",
            "sin-memoria",
            "crece",
            "atrasado",
            "nota-menor",
            "prueba-invalida",
            "otra-raiz",
            "impostor",
            "igual"
        ]
    );
    let mut intercambios = 0;
    for p in pasos {
        let (e, s) = (&p["entrada"], &p["salida"]);
        let (nota, arbol, publicador) = nota_del_paso(e);
        assert_eq!(nota, s["nota"].as_str().unwrap(), "{}: la nota", p["paso"]);
        let k = publicador.clave_publica();
        let mut retencion = Retencion::nueva(&nota, k).unwrap();
        let testigos = e["testigos"].as_array().unwrap();
        assert_eq!(testigos.len(), 2);
        for (t, visto) in testigos.iter().zip(s["testigos"].as_array().unwrap()) {
            let vkey = t["vkey"].as_str().unwrap();
            assert_eq!(vkey, visto["vkey"].as_str().unwrap());
            let testigo = Testigo::nuevo(vkey, t["prefijo"].as_str().unwrap()).unwrap();
            let grabados: VecDeque<Value> = visto["intercambios"]
                .as_array()
                .unwrap()
                .iter()
                .cloned()
                .collect();
            intercambios += grabados.len();
            let mut gr = Grabado(grabados);
            let r = pedir_cofirma(
                &mut gr,
                &testigo,
                k,
                &nota,
                &arbol,
                t["viejo"].as_u64().unwrap(),
            );
            assert!(
                gr.0.is_empty(),
                "{}: quedan intercambios sin pedir",
                p["paso"]
            );
            for (campo, valor) in como_se_grabo(&r) {
                assert_eq!(
                    visto[campo],
                    valor,
                    "{} {}: {campo}",
                    p["paso"],
                    testigo.clave().nombre()
                );
            }
            if let Ok(Resultado::Cofirmada { cofirma, .. }) = r {
                assert!(retencion.retener(testigo.clave(), &cofirma).unwrap());
            }
        }
        assert_eq!(
            retencion.nota_cofirmada(),
            s["nota_cofirmada"].as_str().unwrap(),
            "{}: la nota cofirmada",
            p["paso"]
        );
    }
    // 10 pasos x 2 testigos, y un reintento por testigo en «sin-memoria» y en «atrasado».
    assert_eq!(intercambios, 24);
}

/// Cada `200` de omniwitness-a es una línea ML-DSA-44 de su clave; cada uno de omniwitness-b,
/// la de su clave ML-DSA-44 y detrás la Ed25519 de su otra clave —`key_id`, marca y 64
/// bytes—. Las ML-DSA-44 verifican aquí con su `vkey` sobre el checkpoint de la nota de su
/// paso; la Ed25519 no es de ninguna clave que el cliente conozca.
#[test]
fn cada_cofirma_ml_dsa_44_de_omniwitness_verifica_y_la_ed25519_es_de_la_otra_clave() {
    let g = grabacion();
    let id = |vkey: &Value| -> String {
        vkey.as_str()
            .unwrap()
            .split('+')
            .nth(1)
            .unwrap()
            .to_string()
    };
    let claves_b = g["testigos"][B]["claves"].as_array().unwrap();
    assert_eq!(claves_b[1]["alg"], "ed25519");
    let id_ed = id(&claves_b[1]["vkey"]);
    // [ML-DSA-44 de a, ML-DSA-44 de b, Ed25519 de b]
    let mut vistas = [0usize; 3];
    for p in g["pasos"].as_array().unwrap() {
        let (nota, _, publicador) = nota_del_paso(&p["entrada"]);
        for t in p["salida"]["testigos"].as_array().unwrap() {
            let clave = ClaveDeNota::leer_vkey(t["vkey"].as_str().unwrap()).unwrap();
            for i in t["intercambios"].as_array().unwrap() {
                if i["estado"] != 200 {
                    continue;
                }
                let r = i["respuesta"].as_str().unwrap();
                let v = verificar_nota(&format!("{nota}{r}"), publicador.clave_publica()).unwrap();
                let lineas: Vec<(&str, Vec<u8>)> = r
                    .lines()
                    .map(|l| {
                        let (nombre, b64) = l
                            .strip_prefix("\u{2014} ")
                            .unwrap()
                            .split_once(' ')
                            .unwrap();
                        (nombre, base64_decode(b64).unwrap())
                    })
                    .collect();
                assert_eq!(lineas.len(), v.ajenas.len());
                let marca = clave
                    .verificar_cofirma(&v.checkpoint, &v.ajenas[0])
                    .unwrap();
                assert_eq!(lineas[0].0, clave.nombre());
                assert_eq!(lineas[0].1.len(), 4 + 8 + 2420);
                assert_eq!(lineas[0].1[4..12], marca.to_be_bytes());
                if clave.nombre() == A {
                    assert_eq!(lineas.len(), 1);
                    vistas[0] += 1;
                } else {
                    assert_eq!(clave.nombre(), B);
                    assert_eq!(lineas.len(), 2);
                    vistas[1] += 1;
                    let ed = &lineas[1];
                    let id_linea: String = ed.1[..4].iter().map(|b| format!("{b:02x}")).collect();
                    assert_eq!(
                        (ed.0, ed.1.len(), id_linea.as_str()),
                        (B, 4 + 8 + 64, id_ed.as_str())
                    );
                    assert!(clave
                        .verificar_cofirma(&v.checkpoint, &v.ajenas[1])
                        .is_err());
                    vistas[2] += 1;
                }
            }
        }
    }
    // Cinco cofirmas por testigo: «primera», «sin-memoria», «crece», «atrasado» e «igual».
    assert_eq!(vistas, [5, 5, 5]);
}

/// La `vkey` ML-DSA-44 de cada testigo la sacó de su semilla `omniwitness-local`, con el
/// ML-DSA de Go que usa `formats`; la de aquí, de la misma semilla, da la misma: la misma
/// clave pública y el mismo `key_id`.
#[test]
fn la_vkey_ml_dsa_44_de_cada_omniwitness_sale_de_su_semilla() {
    let g = grabacion();
    let mut vistas = 0;
    for (nombre, t) in g["testigos"].as_object().unwrap() {
        let mldsa: Vec<&Value> = t["claves"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["alg"] == "mldsa")
            .collect();
        assert_eq!(mldsa.len(), 1, "{nombre}");
        assert_eq!(t["vkey"], mldsa[0]["vkey"]);
        let aqui = Cofirmante::determinista(nombre, hex32(&mldsa[0]["semilla"]))
            .unwrap()
            .clave_publica()
            .vkey();
        assert_eq!(aqui, t["vkey"].as_str().unwrap(), "{nombre}");
        vistas += 1;
    }
    assert_eq!(vistas, 2);
}

/// El sobre que el kit dio VERDE en el banco lleva la nota cofirmada de «primera»: la de su
/// publicador y las cofirmas de omniwitness-a y -b, que verifican aquí con sus `vkey`, y el
/// ancla de su cabeza en la posición 1.
#[test]
fn el_sobre_del_banco_lleva_la_nota_cofirmada_por_omniwitness() {
    let g = grabacion();
    let sobre = &g["sobre"];
    let primera = &g["pasos"][1];
    assert_eq!(primera["paso"], "primera");
    assert_eq!(sobre["nota"], primera["salida"]["nota_cofirmada"]);
    let publicador = ClaveDeNota::leer_vkey(sobre["publicador"].as_str().unwrap()).unwrap();
    let v = verificar_nota(sobre["nota"].as_str().unwrap(), &publicador).unwrap();
    let testigos: Vec<ClaveDeNota> = sobre["testigos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| ClaveDeNota::leer_vkey(t.as_str().unwrap()).unwrap())
        .collect();
    assert_eq!(
        testigos.iter().map(|t| t.nombre()).collect::<Vec<_>>(),
        [A, B]
    );
    // Una línea por testigo: la Ed25519 de omniwitness-b no se retuvo.
    assert_eq!(v.ajenas.len(), 2);
    for (t, linea) in testigos.iter().zip(&v.ajenas) {
        t.verificar_cofirma(&v.checkpoint, linea).unwrap();
    }
    let huella = hex32(&primera["entrada"]["hojas"][1]);
    let camino: Vec<HashValue> = sobre["inclusion"]
        .as_array()
        .unwrap()
        .iter()
        .map(hex32)
        .collect();
    assert_eq!(sobre["posicion"], "0x1");
    verificar_inclusion(
        &hoja(&huella),
        1,
        v.checkpoint.tamano,
        &camino,
        &v.checkpoint.raiz,
    )
    .unwrap();
}
