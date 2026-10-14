//! El cliente del medio contra lo que contestaron torchwood y litewitness (RFC-0013 E3a, §808).
//!
//! `tests/vectores/testigos/torchwood.json` es lo que grabó `rehacer.sh` de ese directorio:
//! nueve pasos del publicador contra tres testigos LOCALES de la mano del autor de las
//! especificaciones C2SP —el `add-checkpoint` de litewitness (`filippo.io/torchwood` v0.10.0,
//! `internal/witness`) con una clave ML-DSA-44, dos veces, y litewitness v0.10.0 sin tocar, que
//! cofirma con Ed25519—, cada intercambio HTTP tal cual. El canon no corre Go: vuelve a pasar el
//! cliente por lo grabado y comprueba aquí, en cada pasada:
//!
//! - que la nota de cada paso **sale byte a byte** firmando aquí en determinista con la misma
//!   semilla y la misma marca;
//! - que cada petición que arma el cliente es, **byte a byte**, la que se le envió al testigo de
//!   Go y que éste aceptó o rechazó: la línea `old`, la prueba de consistencia que comprobó
//!   `tlog.CheckTree` y la nota;
//! - que con lo que contestaron el cliente llega al mismo resultado, paso a paso y testigo a
//!   testigo —el `404`, la primera cofirma, el `409` y su reintento, el directorio restaurado,
//!   las dos bifurcaciones (`422`) y la nota de otra clave (`403`)—, sin pedir ni una petición
//!   más de las grabadas;
//! - que las líneas de litewitness son Ed25519 de su clave, y el cliente las ignora;
//! - que la nota cofirmada sale igual, cada cofirma de torchwood verifica con su `vkey`, y el
//!   sobre que el kit dio VERDE en el banco lleva esa nota.

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

fn grabacion() -> Value {
    let ruta =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectores/testigos/torchwood.json");
    serde_json::from_str(&std::fs::read_to_string(ruta).unwrap()).unwrap()
}

fn hex32(v: &Value) -> [u8; 32] {
    let h = v.as_str().unwrap().strip_prefix("0x").unwrap();
    assert_eq!(h.len(), 64);
    let c = |b: u8| (b as char).to_digit(16).unwrap() as u8;
    core::array::from_fn(|i| c(h.as_bytes()[2 * i]) << 4 | c(h.as_bytes()[2 * i + 1]))
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

/// Cada paso, rehecho aquí sobre lo que contestaron los tres testigos.
#[test]
fn el_cliente_rehace_cada_paso_contra_lo_que_contestaron() {
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
            "retroceso",
            "bifurcado",
            "otra-raiz",
            "impostor"
        ]
    );
    let mut intercambios = 0;
    for p in pasos {
        let (e, s) = (&p["entrada"], &p["salida"]);
        let origen = e["origen"].as_str().unwrap();
        let mut arbol = ArbolDelMedio::nuevo();
        for h in e["hojas"].as_array().unwrap() {
            arbol.anadir(&hex32(h));
        }
        let publicador =
            Publicador::determinista(origen, hex32(&e["publicador"]["semilla"])).unwrap();
        let c = Checkpoint {
            origen: origen.to_string(),
            tamano: arbol.tamano(),
            raiz: arbol.raiz(),
        };
        let nota = publicador
            .firmar(&c, e["publicador"]["marca"].as_u64().unwrap())
            .unwrap();
        assert_eq!(nota, s["nota"].as_str().unwrap(), "{}: la nota", p["paso"]);
        let k = publicador.clave_publica();
        let mut retencion = Retencion::nueva(&nota, k).unwrap();
        let testigos = e["testigos"].as_array().unwrap();
        assert_eq!(testigos.len(), 3);
        for (t, visto) in testigos.iter().zip(s["testigos"].as_array().unwrap()) {
            let vkey = match t["vkey"].as_str() {
                Some(v) => v.to_string(),
                None => {
                    Cofirmante::determinista(t["nombre"].as_str().unwrap(), hex32(&t["semilla"]))
                        .unwrap()
                        .clave_publica()
                        .vkey()
                }
            };
            assert_eq!(vkey, visto["vkey"].as_str().unwrap());
            let testigo = Testigo::nuevo(&vkey, t["prefijo"].as_str().unwrap()).unwrap();
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
    // 9 pasos x 3 testigos, y un reintento por testigo en «sin-memoria» y en «atrasado».
    assert_eq!(intercambios, 33);
}

/// Lo que contestó litewitness es Ed25519 de su clave —`key_id`, marca y 64 bytes de firma—,
/// y por eso el cliente no lo retiene; lo de torchwood-a y -b, ML-DSA-44 de las suyas.
#[test]
fn litewitness_cofirma_con_ed25519_y_el_motor_con_ml_dsa_44() {
    let g = grabacion();
    let lw = &g["testigos"]["testigo.invalid/litewitness"];
    let id_lw = lw["vkey_ed25519"]
        .as_str()
        .unwrap()
        .split('+')
        .nth(1)
        .unwrap()
        .to_string();
    let mut vistas = [0usize; 2];
    for p in g["pasos"].as_array().unwrap() {
        for t in p["salida"]["testigos"].as_array().unwrap() {
            for i in t["intercambios"].as_array().unwrap() {
                if i["estado"] != 200 {
                    continue;
                }
                let r = i["respuesta"].as_str().unwrap();
                let (nombre, b64) = r
                    .strip_prefix("\u{2014} ")
                    .unwrap()
                    .trim_end()
                    .split_once(' ')
                    .unwrap();
                let bytes = base64_decode(b64).unwrap();
                let id: String = bytes[..4].iter().map(|b| format!("{b:02x}")).collect();
                if nombre == "testigo.invalid/litewitness" {
                    assert_eq!((bytes.len(), id.as_str()), (4 + 8 + 64, id_lw.as_str()));
                    vistas[0] += 1;
                } else {
                    let clave = ClaveDeNota::leer_vkey(t["vkey"].as_str().unwrap()).unwrap();
                    assert_eq!(bytes.len(), 4 + 8 + 2420);
                    assert_eq!(bytes[..4], clave.id());
                    vistas[1] += 1;
                }
            }
        }
    }
    // Cuatro cofirmas por testigo: «primera», «sin-memoria», «crece» y «atrasado».
    assert_eq!(vistas, [4, 8]);
}

/// El sobre que el kit dio VERDE en el banco lleva la nota cofirmada de «primera»: la de su
/// publicador y las cofirmas de torchwood-a y -b, que verifican aquí con sus `vkey`, y el ancla
/// de su cabeza en la posición 1.
#[test]
fn el_sobre_del_banco_lleva_la_nota_cofirmada_por_torchwood() {
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
        ["testigo.invalid/torchwood-a", "testigo.invalid/torchwood-b"]
    );
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
