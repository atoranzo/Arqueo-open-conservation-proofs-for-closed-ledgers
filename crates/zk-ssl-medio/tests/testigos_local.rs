//! El publicador del medio contra un testigo LOCAL (RFC-0013 E3a, §808).
//!
//! ⚠️ **El testigo de aquí es mínimo, de test y escrito en este fichero**:
//! un servidor HTTP/1.1 en `127.0.0.1` que sigue `tlog-witness` v1.1.0 —el
//! `404` del origin que no conoce, el `403` de la firma que no es de la clave
//! que tiene, el `400` de lo que no se entiende, el `409` con el tamaño que
//! recuerda, el `422` de la consistencia que no cuadra, y el `200` con su
//! cofirma ML-DSA-44 `0x06`—, que responde POR TROZOS, como responde el de Go
//! cuando el cuerpo pasa de 2 KiB. **No es litewitness**, y verifica con el
//! mismo `zk_ssl_medio` que el cliente: lo que lo ata a una implementación de
//! otra mano es `tests/testigos_torchwood.rs`, que relee lo que contestaron
//! el motor de litewitness y litewitness mismo.
//!
//! Aquí se miden los caminos del cliente con un testigo de verdad al otro
//! lado del socket: la primera cofirma, el crecimiento con su prueba, el
//! publicador sin memoria que aprende del `409`, el directorio restaurado
//! (la 103), la bifurcación, la nota de otra clave, las líneas de claves que
//! no conoce y la nota cofirmada de dos testigos, publicada y releída. Los
//! dos últimos tests no hablan con él sino con un servidor de un solo uso
//! que no sigue `tlog-witness` y mide solo el transporte: el que no acaba de
//! contestar y el que no para de mandar.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use zk_ssl_medio::base64::base64_decode;
use zk_ssl_medio::hash::{hash_empty, sha256, HashValue};
use zk_ssl_medio::medio::{verificar_consistencia, ArbolDelMedio};
use zk_ssl_medio::nota::{
    origen_del_medio, verificar_nota, Checkpoint, ClaveDeNota, Cofirmante, Publicador,
};
use zk_ssl_medio::testigos::{
    pedir_cofirma, publicar, ErrorDeTestigo, HttpLocal, Respuesta, Resultado, Retencion, Testigo,
    FICHERO, MAX_CUERPO,
};

/// La huella de la clave del operador de `spec/vectors/ancla/`.
const HUELLA: [u8; 32] = [
    0x8c, 0x40, 0xb5, 0x5b, 0x1d, 0x40, 0xcf, 0x7f, 0x71, 0xf9, 0xea, 0x99, 0xc7, 0x70, 0xf7, 0x01,
    0xdb, 0x95, 0xda, 0x58, 0x76, 0xe6, 0xce, 0x88, 0x9f, 0x86, 0xaf, 0x12, 0xb0, 0xde, 0xf5, 0x3a,
];

/// Cómo contesta el testigo de prueba a un `add-checkpoint` que acepta.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Modo {
    /// Su cofirma, sola.
    Normal,
    /// Delante de la suya, la de otra clave con su mismo nombre.
    ConOtraClave,
    /// Solo la de otra clave con su mismo nombre.
    SoloOtraClave,
    /// La suya con un byte cambiado.
    Tocada,
}

/// Lo que recuerda el testigo de cada origin: las claves que acepta, y el
/// último tamaño y la última raíz que cofirmó.
struct Estado {
    logs: HashMap<String, (Vec<ClaveDeNota>, u64, HashValue)>,
    modo: Modo,
    marca: u64,
}

struct TestigoDePrueba {
    estado: Arc<Mutex<Estado>>,
    testigo: Testigo,
}

impl TestigoDePrueba {
    fn levantar(nombre: &str, semilla: u8) -> TestigoDePrueba {
        let firmante = Cofirmante::desde_semilla(nombre, [semilla; 32]).unwrap();
        let otra = Cofirmante::desde_semilla(nombre, [semilla ^ 0xff; 32]).unwrap();
        let estado = Arc::new(Mutex::new(Estado {
            logs: HashMap::new(),
            modo: Modo::Normal,
            marca: 1_790_000_000,
        }));
        let escucha = TcpListener::bind("127.0.0.1:0").unwrap();
        let prefijo = format!("http://{}/testigo", escucha.local_addr().unwrap());
        let testigo = Testigo::nuevo(&firmante.clave_publica().vkey(), &prefijo).unwrap();
        let e = Arc::clone(&estado);
        thread::spawn(move || {
            for s in escucha.incoming() {
                let Ok(s) = s else { return };
                atender(s, &e, &firmante, &otra);
            }
        });
        TestigoDePrueba { estado, testigo }
    }

    fn registrar(&self, publicador: &ClaveDeNota) {
        self.estado.lock().unwrap().logs.insert(
            publicador.nombre().to_string(),
            (vec![publicador.clone()], 0, hash_empty()),
        );
    }

    fn recuerda(&self, origen: &str) -> u64 {
        self.estado.lock().unwrap().logs[origen].1
    }

    fn modo(&self, modo: Modo) {
        self.estado.lock().unwrap().modo = modo;
    }
}

/// Lee una petición HTTP entera: la cabeza, y el cuerpo por su
/// `Content-Length`. Entera, para que al cerrar no quede nada sin leer en el
/// socket: si quedara, el cierre sería un RST y no un FIN.
fn leer_peticion(s: &mut TcpStream) -> (String, Vec<u8>) {
    let mut leido = Vec::new();
    let mut trozo = [0u8; 4096];
    let (cabeza, largo) = loop {
        let n = s.read(&mut trozo).unwrap();
        assert!(n > 0, "la conexión se cerró a medias");
        leido.extend_from_slice(&trozo[..n]);
        if let Some(i) = leido.windows(4).position(|w| w == b"\r\n\r\n") {
            let cabeza = String::from_utf8(leido[..i].to_vec()).unwrap();
            let largo: usize = cabeza
                .lines()
                .find_map(|l| l.strip_prefix("Content-Length: "))
                .unwrap()
                .parse()
                .unwrap();
            leido.drain(..i + 4);
            break (cabeza, largo);
        }
    };
    while leido.len() < largo {
        let n = s.read(&mut trozo).unwrap();
        assert!(n > 0, "la conexión se cerró a medias");
        leido.extend_from_slice(&trozo[..n]);
    }
    (cabeza, leido)
}

/// Lee la petición HTTP, la contesta y cierra.
fn atender(mut s: TcpStream, e: &Mutex<Estado>, firmante: &Cofirmante, otra: &Cofirmante) {
    let (cabeza, leido) = leer_peticion(&mut s);
    assert!(
        cabeza.starts_with("POST /testigo/add-checkpoint HTTP/1.1\r\n"),
        "{cabeza}"
    );
    let (estado, tipo, cuerpo) = add_checkpoint(&leido, e, firmante, otra);
    let mut r = format!("HTTP/1.1 {estado} X\r\nConnection: close\r\n");
    if let Some(t) = tipo {
        r.push_str(&format!("Content-Type: {t}\r\n"));
    }
    // Por trozos, de 1000 bytes: así responde el servidor HTTP de Go.
    r.push_str("Transfer-Encoding: chunked\r\n\r\n");
    let mut b = r.into_bytes();
    for t in cuerpo.as_bytes().chunks(1000) {
        b.extend_from_slice(format!("{:x}\r\n", t.len()).as_bytes());
        b.extend_from_slice(t);
        b.extend_from_slice(b"\r\n");
    }
    b.extend_from_slice(b"0\r\n\r\n");
    s.write_all(&b).unwrap();
}

/// `add-checkpoint`, regla a regla de `tlog-witness` v1.1.0.
fn add_checkpoint(
    cuerpo: &[u8],
    e: &Mutex<Estado>,
    firmante: &Cofirmante,
    otra: &Cofirmante,
) -> (u16, Option<&'static str>, String) {
    let mala = (400, None, "bad request\n".to_string());
    let Ok(cuerpo) = std::str::from_utf8(cuerpo) else {
        return mala;
    };
    let Some((cabeza, nota)) = cuerpo.split_once("\n\n") else {
        return mala;
    };
    let mut lineas = cabeza.split('\n');
    let Some(viejo) = lineas.next().and_then(|l| l.strip_prefix("old ")) else {
        return mala;
    };
    let Ok(viejo) = viejo.parse::<u64>() else {
        return mala;
    };
    let prueba: Vec<HashValue> = match lineas
        .map(|l| base64_decode(l).ok().and_then(|b| b.try_into().ok()))
        .collect::<Option<Vec<HashValue>>>()
    {
        Some(p) if p.len() <= 63 => p,
        _ => return mala,
    };
    let origen = nota.split('\n').next().unwrap_or("");
    let mut e = e.lock().unwrap();
    let modo = e.modo;
    e.marca += 1;
    let marca = e.marca;
    let Some((claves, sabido, raiz_sabida)) = e.logs.get_mut(origen) else {
        return (404, None, "unknown log\n".into());
    };
    let Some(c) = claves.iter().find_map(|k| match verificar_nota(nota, k) {
        Ok(v) => Some(v.checkpoint),
        Err(_) => None,
    }) else {
        return (403, None, "invalid signature\n".into());
    };
    if viejo > c.tamano {
        return mala;
    }
    if c.tamano == 0 && c.raiz != hash_empty() {
        return (422, None, "bad empty tree\n".into());
    }
    if viejo != *sabido {
        return (409, Some("text/x.tlog.size"), format!("{sabido}\n"));
    }
    let consistente = if viejo == 0 {
        prueba.is_empty()
    } else if viejo == c.tamano {
        prueba.is_empty() && c.raiz == *raiz_sabida
    } else {
        verificar_consistencia(viejo, c.tamano, &prueba, raiz_sabida, &c.raiz).is_ok()
    };
    if !consistente {
        return (422, None, "bad consistency proof\n".into());
    }
    (*sabido, *raiz_sabida) = (c.tamano, c.raiz);
    let suya = firmante.cofirmar(&c, marca).unwrap();
    let ajena = otra.cofirmar(&c, marca).unwrap();
    let cuerpo = match modo {
        Modo::Normal => suya,
        Modo::ConOtraClave => format!("{ajena}{suya}"),
        Modo::SoloOtraClave => ajena,
        Modo::Tocada => {
            let i = suya.len() - 200;
            let x = if &suya[i..=i] == "A" { "B" } else { "A" };
            format!("{}{x}{}", &suya[..i], &suya[i + 1..])
        }
    };
    (200, None, cuerpo)
}

/// Las huellas de las anclas, deterministas: la `i` es `SHA-256(u64be(i))`,
/// y una reescrita es la de `i + 1000`.
fn huella(i: u64) -> [u8; 32] {
    sha256(&i.to_be_bytes())
}

fn arbol(n: u64, reescrita: Option<u64>) -> ArbolDelMedio {
    let mut a = ArbolDelMedio::nuevo();
    for i in 0..n {
        a.anadir(&huella(if Some(i) == reescrita { i + 1000 } else { i }));
    }
    a
}

fn publicador(semilla: u8) -> Publicador {
    Publicador::desde_semilla(&origen_del_medio(&HUELLA), [semilla; 32]).unwrap()
}

fn nota(p: &Publicador, a: &ArbolDelMedio) -> String {
    let c = Checkpoint {
        origen: p.clave_publica().nombre().to_string(),
        tamano: a.tamano(),
        raiz: a.raiz(),
    };
    p.firmar(&c, 1_790_000_000).unwrap()
}

fn pedir(t: &TestigoDePrueba, p: &Publicador, a: &ArbolDelMedio, viejo: u64) -> Resultado {
    pedir_cofirma(
        &mut HttpLocal::default(),
        &t.testigo,
        p.clave_publica(),
        &nota(p, a),
        a,
        viejo,
    )
    .unwrap()
}

/// De punta a punta, con dos testigos: el origin que no conocen, la primera
/// cofirma, el crecimiento con su prueba, y la nota cofirmada de los dos,
/// que el verificador de notas lee, cada cofirma con su testigo, y que se
/// publica y se relee.
#[test]
fn dos_testigos_locales_cofirman_y_la_nota_cofirmada_se_publica() {
    let p = publicador(1);
    let k = p.clave_publica();
    let uno = TestigoDePrueba::levantar("testigo.invalid/uno", 2);
    let dos = TestigoDePrueba::levantar("testigo.invalid/dos", 3);
    let a3 = arbol(3, None);
    assert_eq!(
        pedir(&uno, &p, &a3, 0),
        Resultado::Rechazada(Respuesta::OrigenDesconocido)
    );
    uno.registrar(k);
    dos.registrar(k);
    for t in [&uno, &dos] {
        assert!(matches!(
            pedir(t, &p, &a3, 0),
            Resultado::Cofirmada {
                viejo: 0,
                ignoradas: 0,
                ..
            }
        ));
    }
    let a7 = arbol(7, None);
    let n7 = nota(&p, &a7);
    let mut r = Retencion::nueva(&n7, k).unwrap();
    for t in [&uno, &dos] {
        let res = pedir_cofirma(&mut HttpLocal::default(), &t.testigo, k, &n7, &a7, 3).unwrap();
        let Resultado::Cofirmada {
            cofirma,
            viejo: 3,
            ignoradas: 0,
        } = res
        else {
            panic!("{res:?}")
        };
        assert!(r.retener(t.testigo.clave(), &cofirma).unwrap());
        assert_eq!(t.recuerda(k.nombre()), 7);
    }
    let nc = r.nota_cofirmada();
    let v = verificar_nota(&nc, k).unwrap();
    assert_eq!((v.checkpoint.tamano, v.ajenas.len()), (7, 2));
    for (t, linea) in [&uno, &dos].iter().zip(&v.ajenas) {
        t.testigo
            .clave()
            .verificar_cofirma(&v.checkpoint, linea)
            .unwrap();
    }
    /// El directorio temporal se borra también si el test cae.
    struct Temporal(std::path::PathBuf);
    impl Drop for Temporal {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let t = Temporal(
        std::env::temp_dir().join(format!("zk-ssl-medio-testigos-{}", std::process::id())),
    );
    let _ = std::fs::remove_dir_all(&t.0);
    std::fs::create_dir_all(&t.0).unwrap();
    publicar(&t.0, &r, k).unwrap();
    let publicado = std::fs::read_to_string(t.0.join(FICHERO)).unwrap();
    assert_eq!(publicado, nc);
    let claves = [uno.testigo.clave().clone(), dos.testigo.clave().clone()];
    assert_eq!(Retencion::leer(&publicado, k, &claves).unwrap(), r);
}

/// El publicador que no recuerda el tamaño de un testigo empieza por `old
/// 0`, el testigo le dice el suyo, y el reintento desde él cofirma.
#[test]
fn el_publicador_sin_memoria_aprende_del_409() {
    let p = publicador(1);
    let t = TestigoDePrueba::levantar("testigo.invalid/uno", 2);
    t.registrar(p.clave_publica());
    pedir(&t, &p, &arbol(4, None), 0);
    assert!(matches!(
        pedir(&t, &p, &arbol(9, None), 0),
        Resultado::Cofirmada { viejo: 4, .. }
    ));
    // Y con el mismo tamaño: se reintenta desde él, con la prueba vacía.
    assert!(matches!(
        pedir(&t, &p, &arbol(9, None), 0),
        Resultado::Cofirmada { viejo: 9, .. }
    ));
    assert_eq!(t.recuerda(p.clave_publica().nombre()), 9);
}

/// ⚠️ **La 103.** El directorio del publicador se restaura a una copia con
/// menos anclas: el testigo recuerda más, lo dice, y el cliente PARA. El
/// testigo no se mueve.
#[test]
fn el_testigo_ve_el_directorio_restaurado() {
    let p = publicador(1);
    let t = TestigoDePrueba::levantar("testigo.invalid/uno", 2);
    t.registrar(p.clave_publica());
    pedir(&t, &p, &arbol(9, None), 0);
    for viejo in [0, 5] {
        assert_eq!(
            pedir(&t, &p, &arbol(5, None), viejo),
            Resultado::Retroceso {
                recordado: 9,
                nota: 5
            }
        );
    }
    assert_eq!(t.recuerda(p.clave_publica().nombre()), 9);
}

/// Un ancla reescrita: la prueba es consistente consigo misma, y no con la
/// raíz que el testigo guardó. Con el mismo tamaño, la otra raíz tampoco.
#[test]
fn el_testigo_ve_la_bifurcacion() {
    let p = publicador(1);
    let t = TestigoDePrueba::levantar("testigo.invalid/uno", 2);
    t.registrar(p.clave_publica());
    pedir(&t, &p, &arbol(9, None), 0);
    assert_eq!(
        pedir(&t, &p, &arbol(11, Some(4)), 9),
        Resultado::Rechazada(Respuesta::Inconsistente)
    );
    assert_eq!(
        pedir(&t, &p, &arbol(9, Some(4)), 9),
        Resultado::Rechazada(Respuesta::Inconsistente)
    );
    assert_eq!(t.recuerda(p.clave_publica().nombre()), 9);
}

/// La nota de otra clave con el mismo nombre no la cofirma nadie: `403`.
#[test]
fn una_nota_de_otra_clave_es_403() {
    let p = publicador(1);
    let t = TestigoDePrueba::levantar("testigo.invalid/uno", 2);
    t.registrar(p.clave_publica());
    assert_eq!(
        pedir(&t, &publicador(4), &arbol(3, None), 0),
        Resultado::Rechazada(Respuesta::Prohibida)
    );
    assert_eq!(t.recuerda(p.clave_publica().nombre()), 0);
}

/// Las líneas de una clave que el cliente no conoce se ignoran, también la
/// de otra clave con el nombre del testigo; una suya tocada es un error.
#[test]
fn el_cliente_ignora_las_claves_que_no_conoce() {
    let p = publicador(1);
    let t = TestigoDePrueba::levantar("testigo.invalid/uno", 2);
    t.registrar(p.clave_publica());
    t.modo(Modo::ConOtraClave);
    assert!(matches!(
        pedir(&t, &p, &arbol(3, None), 0),
        Resultado::Cofirmada {
            viejo: 0,
            ignoradas: 1,
            ..
        }
    ));
    t.modo(Modo::SoloOtraClave);
    assert_eq!(
        pedir(&t, &p, &arbol(5, None), 3),
        Resultado::SinSuCofirma {
            viejo: 3,
            ignoradas: 1
        }
    );
    t.modo(Modo::Tocada);
    let a = arbol(7, None);
    let r = pedir_cofirma(
        &mut HttpLocal::default(),
        &t.testigo,
        p.clave_publica(),
        &nota(&p, &a),
        &a,
        5,
    );
    assert_eq!(
        r,
        Err(ErrorDeTestigo::CofirmaInvalida {
            testigo: "testigo.invalid/uno".into()
        })
    );
}

/// Un testigo que contesta un byte cada poco renovaría sin fin el plazo de
/// cada lectura: el transporte tiene además un plazo TOTAL, y el publicador
/// no se queda colgado.
#[test]
fn un_testigo_que_no_acaba_no_cuelga_al_publicador() {
    let p = publicador(1);
    let escucha = TcpListener::bind("127.0.0.1:0").unwrap();
    let prefijo = format!("http://{}", escucha.local_addr().unwrap());
    thread::spawn(move || {
        let (mut s, _) = escucha.accept().unwrap();
        leer_peticion(&mut s);
        // Unos cuatro segundos de cabecera, byte a byte, y cierra.
        for c in b"HTTP/1.1 200 OK\r\nX: ".iter().chain([b'a'; 250].iter()) {
            if s.write_all(&[*c]).is_err() {
                return;
            }
            thread::sleep(Duration::from_millis(15));
        }
    });
    let t = Cofirmante::desde_semilla("testigo.invalid/lento", [5; 32]).unwrap();
    let testigo = Testigo::nuevo(&t.clave_publica().vkey(), &prefijo).unwrap();
    let a = arbol(3, None);
    let mut http = HttpLocal {
        plazo: Duration::from_millis(300),
    };
    let antes = Instant::now();
    let r = pedir_cofirma(&mut http, &testigo, p.clave_publica(), &nota(&p, &a), &a, 0);
    let tardo = antes.elapsed();
    // Lo normal es que salte el plazo total, a los 3 × 300 ms: «el testigo
    // no acaba de contestar». Con la máquina cargada, el hilo del servidor
    // puede quedarse sin turno más de 300 ms y salta antes el plazo de una
    // lectura, que es otro error del transporte y tampoco cuelga: se
    // aceptan los dos. Sin el plazo total, la cabecera llega entera a los
    // cuatro segundos, cortada, y el error es de la respuesta.
    assert!(matches!(r, Err(ErrorDeTestigo::Transporte(_))), "{r:?}");
    assert!(tardo < Duration::from_secs(3), "{tardo:?}");
}

/// Un testigo que no para de mandar no llena la memoria del publicador: el
/// transporte corta en un tope de bytes —por encima del cuerpo más largo
/// que lee, `MAX_CUERPO`, aun por trozos— y lo dice.
#[test]
fn un_testigo_que_no_para_de_mandar_no_llena_al_publicador() {
    let p = publicador(1);
    let escucha = TcpListener::bind("127.0.0.1:0").unwrap();
    let prefijo = format!("http://{}", escucha.local_addr().unwrap());
    thread::spawn(move || {
        let (mut s, _) = escucha.accept().unwrap();
        leer_peticion(&mut s);
        // Un `200` que anuncia y manda, deprisa, tres veces el cuerpo más
        // largo que el publicador lee, y cierra. Si el publicador corta
        // antes, la escritura falla y el hilo acaba.
        let largo = 3 * MAX_CUERPO;
        let cabeza = format!("HTTP/1.1 200 OK\r\nContent-Length: {largo}\r\n\r\n");
        if s.write_all(cabeza.as_bytes()).is_err() {
            return;
        }
        let trozo = vec![b'a'; 64 * 1024];
        for _ in 0..largo / trozo.len() {
            if s.write_all(&trozo).is_err() {
                return;
            }
        }
    });
    let t = Cofirmante::desde_semilla("testigo.invalid/sin-fin", [6; 32]).unwrap();
    let testigo = Testigo::nuevo(&t.clave_publica().vkey(), &prefijo).unwrap();
    let a = arbol(3, None);
    let r = pedir_cofirma(
        &mut HttpLocal::default(),
        &testigo,
        p.clave_publica(),
        &nota(&p, &a),
        &a,
        0,
    );
    // Sin el tope, el publicador lo leería entero hasta el cierre, y el
    // error sería el del `Content-Length` que pasa del máximo.
    assert_eq!(
        r,
        Err(ErrorDeTestigo::Respuesta(
            "la respuesta pasa del máximo".into()
        ))
    );
}
