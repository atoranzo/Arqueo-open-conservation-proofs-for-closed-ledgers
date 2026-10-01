//! Las sondas de `transparency-dev/merkle` (`testdata/`, commit `fbbcd74`):
//! 685 ficheros, 43 positivos y 642 negativos, que su `cmd/proofgen`
//! genera a partir de pruebas correctas y luego estropea de todas las
//! maneras que un verificador descuidado deja pasar (raíces cruzadas,
//! tamaños desplazados, un hash vacío delante o detrás, un bit cambiado,
//! un elemento de 9 o de 12 bytes). Es la biblioteca con la que el testigo
//! de transparency-dev comprueba la consistencia
//! (`witness/witness.go`, `proof.VerifyConsistency`).
//!
//! Cada sonda se corre con la función de aquí que le corresponde, y el
//! veredicto tiene que ser el suyo:
//!
//! | carpeta              | función de aquí                          | suya                       |
//! |----------------------|------------------------------------------|----------------------------|
//! | `inclusion`          | `medio::verificar_inclusion`             | `VerifyInclusion`          |
//! | `consistency`        | `medio::verificar_consistencia`          | `VerifyConsistency`        |
//! | `subtreeinclusion`   | `subtree::verify_inclusion_proof`        | `VerifySubtreeInclusion`   |
//! | `subtreeconsistency` | `subtree::verify_consistency_proof`      | `VerifySubtreeConsistency` |
//!
//! ⚠️ **Dos rechazos distintos, contados aparte.** Aquí un hash es
//! `[u8; 32]`: un elemento de 0, 9 o 12 bytes **no llega al verificador**,
//! lo rechaza el tipo al leer la sonda. Ese rechazo es correcto —es lo que
//! hace un lector estricto— pero no es mérito del verificador, y por eso
//! las cifras de cada carpeta separan los negativos que rechaza el tipo de
//! los que rechaza el árbol. Un lector que **descartara** el elemento vacío
//! en vez de rechazarlo convertiría «preceding garbage» en una prueba
//! válida: por eso se rechaza.
//!
//! ⚠️ **Y cuatro positivas que aquí no se pueden escribir.** Su
//! `cmd/proofgen` pone como raíz el texto `"don't care 1"` o `"don't care 2"`
//! (12 bytes) en los casos en que su verificador no la mira: tamaños iguales
//! con prueba vacía, o un subárbol vacío. Go las acepta porque compara bytes
//! de cualquier longitud; aquí una raíz de 12 bytes no es una raíz, y el
//! tipo las rechaza. Es una divergencia del lado estricto, y se fija por
//! nombre en [`SIN_TIPO`] para que una quinta no se cuele callada.
//!
//! ⚠️ **Contar aparte no es un detalle**: sus negativas de raíz o de hoja
//! equivocada usan textos de 9 bytes, y en un verificador tipado no llegan
//! nunca a la comparación. Lo que eso esconde, y cómo se tapa, en
//! [`sus_positivas_con_un_bit_cambiado_en_32_bytes`].
//!
//! El JSON se lee a mano, como en `vectores_grandes.rs`, para que `serde`
//! no entre por la puerta de atrás.

use std::path::{Path, PathBuf};

use zk_ssl_medio::hash::{hash_leaf, HashValue};
use zk_ssl_medio::medio::{verificar_consistencia, verificar_inclusion};
use zk_ssl_medio::subtree::{mth, verify_consistency_proof, verify_inclusion_proof, Subtree};

/// Un valor del subconjunto de JSON que usan las sondas.
#[derive(Debug, Clone)]
enum Valor {
    Texto(String),
    Numero(u64),
    Logico(bool),
    Nulo,
    Lista(Vec<Valor>),
}

struct Lector<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Lector<'a> {
    fn blancos(&mut self) {
        while self.i < self.b.len() && self.b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    fn espera(&mut self, c: u8) {
        self.blancos();
        assert_eq!(self.b.get(self.i), Some(&c), "se esperaba {:?}", c as char);
        self.i += 1;
    }

    fn texto(&mut self) -> String {
        self.espera(b'"');
        let ini = self.i;
        while self.b[self.i] != b'"' {
            assert_ne!(self.b[self.i], b'\\', "el lector no admite escapes");
            self.i += 1;
        }
        self.i += 1;
        String::from_utf8(self.b[ini..self.i - 1].to_vec()).unwrap()
    }

    fn literal(&mut self, s: &str) {
        assert!(self.b[self.i..].starts_with(s.as_bytes()), "literal {s}");
        self.i += s.len();
    }

    fn valor(&mut self) -> Valor {
        self.blancos();
        match self.b[self.i] {
            b'"' => Valor::Texto(self.texto()),
            b't' => {
                self.literal("true");
                Valor::Logico(true)
            }
            b'f' => {
                self.literal("false");
                Valor::Logico(false)
            }
            b'n' => {
                self.literal("null");
                Valor::Nulo
            }
            b'[' => {
                self.i += 1;
                let mut v = Vec::new();
                self.blancos();
                if self.b[self.i] == b']' {
                    self.i += 1;
                    return Valor::Lista(v);
                }
                loop {
                    v.push(self.valor());
                    self.blancos();
                    match self.b[self.i] {
                        b',' => self.i += 1,
                        b']' => {
                            self.i += 1;
                            return Valor::Lista(v);
                        }
                        c => panic!("lista: {:?}", c as char),
                    }
                }
            }
            c if c.is_ascii_digit() => {
                let ini = self.i;
                while self.i < self.b.len() && self.b[self.i].is_ascii_digit() {
                    self.i += 1;
                }
                let s = std::str::from_utf8(&self.b[ini..self.i]).unwrap();
                Valor::Numero(s.parse().expect("u64"))
            }
            c => panic!("valor: {:?}", c as char),
        }
    }

    /// Un objeto plano `{"clave": valor, ...}`.
    fn objeto(texto: &'a str) -> Vec<(String, Valor)> {
        let mut l = Lector {
            b: texto.as_bytes(),
            i: 0,
        };
        l.espera(b'{');
        let mut campos = Vec::new();
        loop {
            let k = l.texto();
            l.espera(b':');
            campos.push((k, l.valor()));
            l.blancos();
            match l.b[l.i] {
                b',' => l.i += 1,
                b'}' => break,
                c => panic!("objeto: {:?}", c as char),
            }
        }
        l.i += 1;
        l.blancos();
        assert_eq!(l.i, l.b.len(), "basura tras el objeto");
        campos
    }
}

/// Base64 estándar con relleno, estricta: un carácter ajeno es un error.
fn base64(s: &str) -> Vec<u8> {
    const TABLA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    assert_eq!(s.len() % 4, 0, "base64 sin relleno: {s:?}");
    let cuerpo = s.trim_end_matches('=');
    assert!(s.len() - cuerpo.len() <= 2, "relleno de más: {s:?}");
    let mut out = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0;
    for b in cuerpo.bytes() {
        let v = TABLA.iter().position(|t| *t == b).expect("base64") as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    out
}

/// Una sonda leída.
struct Sonda {
    fichero: PathBuf,
    campos: Vec<(String, Valor)>,
}

impl Sonda {
    fn campo(&self, k: &str) -> &Valor {
        &self
            .campos
            .iter()
            .find(|(c, _)| c == k)
            .unwrap_or_else(|| panic!("{}: falta {k}", self.fichero.display()))
            .1
    }

    fn numero(&self, k: &str) -> u64 {
        match self.campo(k) {
            Valor::Numero(n) => *n,
            v => panic!("{}: {k} = {v:?}", self.fichero.display()),
        }
    }

    fn quiere_error(&self) -> bool {
        match self.campo("wantErr") {
            Valor::Logico(b) => *b,
            v => panic!("wantErr = {v:?}"),
        }
    }

    /// `None`: no mide 32 bytes, y el tipo lo rechaza antes de llegar al
    /// verificador.
    fn hash(&self, k: &str) -> Option<HashValue> {
        match self.campo(k) {
            Valor::Texto(s) => base64(s).try_into().ok(),
            v => panic!("{}: {k} = {v:?}", self.fichero.display()),
        }
    }

    /// `proof`: `null` es la lista vacía, como en Go.
    fn prueba(&self) -> Option<Vec<HashValue>> {
        match self.campo("proof") {
            Valor::Nulo => Some(Vec::new()),
            Valor::Lista(v) => v
                .iter()
                .map(|e| match e {
                    Valor::Texto(s) => base64(s).try_into().ok(),
                    e => panic!("elemento {e:?}"),
                })
                .collect(),
            v => panic!("proof = {v:?}"),
        }
    }
}

fn sondas(carpeta: &str) -> Vec<Sonda> {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/vectores/transparency-dev")
        .join(carpeta);
    let mut ficheros = Vec::new();
    let mut pendientes = vec![raiz];
    while let Some(d) = pendientes.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                pendientes.push(p);
            } else {
                assert_eq!(p.extension().and_then(|x| x.to_str()), Some("json"));
                ficheros.push(p);
            }
        }
    }
    ficheros.sort();
    ficheros
        .into_iter()
        .map(|f| {
            let texto = std::fs::read_to_string(&f).unwrap();
            Sonda {
                campos: Lector::objeto(&texto),
                fichero: f,
            }
        })
        .collect()
}

/// Las positivas cuya raíz es `"don't care N"` (12 bytes): Go no la mira y
/// las acepta; aquí no se pueden escribir.
const SIN_TIPO: [&str; 4] = [
    "consistency/additional/sizes-are-equal-one-and-proof-is-empty.json",
    "subtreeconsistency/additional/sizes-are-equal-one-and-proof-is-empty.json",
    "subtreeconsistency/additional/subtree-is-empty-sizes-are-equal-one-subtree-root-valid-tree-root-random-proof-is-empty.json",
    "subtreeconsistency/additional/subtree-is-empty-sizes-are-equal-zero-subtree-root-valid-tree-root-random-proof-is-empty.json",
];

/// Lo que dio una carpeta.
#[derive(Debug, PartialEq, Eq)]
struct Cuenta {
    /// Positivas que el verificador acepta.
    aceptadas: usize,
    /// Positivas que el tipo no deja escribir (todas en [`SIN_TIPO`]).
    sin_tipo: usize,
    /// Negativas que rechaza el tipo: algún hash no mide 32 bytes.
    rechaza_el_tipo: usize,
    /// Negativas que llegan al verificador y las rechaza él.
    rechaza_el_arbol: usize,
}

/// Corre cada sonda: `None` si el tipo la rechaza, `Some(acepta)` si llega.
fn correr(carpeta: &str, f: impl Fn(&Sonda) -> Option<bool>) -> Cuenta {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vectores/transparency-dev");
    let mut c = Cuenta {
        aceptadas: 0,
        sin_tipo: 0,
        rechaza_el_tipo: 0,
        rechaza_el_arbol: 0,
    };
    for s in sondas(carpeta) {
        let veredicto = f(&s);
        let nombre = s.fichero.strip_prefix(&base).unwrap().to_str().unwrap();
        match (s.quiere_error(), veredicto) {
            (false, Some(true)) => c.aceptadas += 1,
            (false, None) if SIN_TIPO.contains(&nombre) => c.sin_tipo += 1,
            (false, v) => panic!("{nombre}: positiva rechazada ({v:?})"),
            (true, None) => c.rechaza_el_tipo += 1,
            (true, Some(false)) => c.rechaza_el_arbol += 1,
            (true, Some(true)) => panic!("{nombre}: negativa ACEPTADA"),
        }
    }
    c
}

/// `Cuenta` en una línea, para que las cuatro cifras se lean juntas.
fn cuenta(aceptadas: usize, sin_tipo: usize, tipo: usize, arbol: usize) -> Cuenta {
    Cuenta {
        aceptadas,
        sin_tipo,
        rechaza_el_tipo: tipo,
        rechaza_el_arbol: arbol,
    }
}

#[test]
fn sondas_de_inclusion() {
    let c = correr("inclusion", |s| {
        let (hoja, raiz, prueba) = (s.hash("leafHash")?, s.hash("root")?, s.prueba()?);
        Some(
            verificar_inclusion(
                &hoja,
                s.numero("leafIdx"),
                s.numero("treeSize"),
                &prueba,
                &raiz,
            )
            .is_ok(),
        )
    });
    assert_eq!(c, cuenta(6, 0, 26, 66));
}

#[test]
fn sondas_de_consistencia() {
    let c = correr("consistency", |s| {
        let (r1, r2, prueba) = (s.hash("root1")?, s.hash("root2")?, s.prueba()?);
        Some(
            verificar_consistencia(s.numero("size1"), s.numero("size2"), &prueba, &r1, &r2).is_ok(),
        )
    });
    assert_eq!(c, cuenta(5, 1, 26, 66));
}

#[test]
fn sondas_de_inclusion_en_subarbol() {
    let c = correr("subtreeinclusion", |s| {
        let (hoja, raiz, prueba) = (s.hash("leafHash")?, s.hash("root")?, s.prueba()?);
        let st = match Subtree::new(s.numero("start"), s.numero("end")) {
            Ok(st) => st,
            Err(_) => return Some(false),
        };
        Some(verify_inclusion_proof(&hoja, st, s.numero("leafIdx"), &prueba, &raiz).is_ok())
    });
    assert_eq!(c, cuenta(12, 0, 52, 140));
}

#[test]
fn sondas_de_consistencia_de_subarbol() {
    let c = correr("subtreeconsistency", |s| {
        let (r1, r2, prueba) = (s.hash("root1")?, s.hash("root2")?, s.prueba()?);
        let st = match Subtree::new(s.numero("start"), s.numero("end")) {
            Ok(st) => st,
            Err(_) => return Some(false),
        };
        Some(verify_consistency_proof(s.numero("size"), st, &prueba, &r1, &r2).is_ok())
    });
    assert_eq!(c, cuenta(16, 3, 58, 208));
}

/// ⚠️ **El punto ciego, medido en §631.** Sus negativas «wrong root1»,
/// «wrong root2» y «wrong leaf» cambian el hash por el texto `WrongRoot` o
/// `WrongLeaf` (9 bytes): en un verificador tipado las rechaza el tipo y
/// nunca llegan a la comparación. Medido con una mutación: un
/// `verify_consistency_proof` que **no compara la raíz vieja** reconstruida
/// pasa las 685 sondas. Aquí se derivan de cada positiva las mismas
/// negativas **con 32 bytes** (un bit cambiado), que sí llegan al árbol.
/// No son sondas de transparency-dev: son de Arqueo, hechas de las suyas.
#[test]
fn sus_positivas_con_un_bit_cambiado_en_32_bytes() {
    fn bit(mut h: HashValue) -> HashValue {
        h[0] ^= 1;
        h
    }
    let mut n = 0;
    for s in sondas("consistency") {
        let (Some(r1), Some(r2), Some(p)) = (s.hash("root1"), s.hash("root2"), s.prueba()) else {
            continue;
        };
        if s.quiere_error() {
            continue;
        }
        let (m, t) = (s.numero("size1"), s.numero("size2"));
        assert!(verificar_consistencia(m, t, &p, &bit(r1), &r2).is_err());
        assert!(verificar_consistencia(m, t, &p, &r1, &bit(r2)).is_err());
        n += 2;
    }
    for s in sondas("subtreeconsistency") {
        let (Some(r1), Some(r2), Some(p)) = (s.hash("root1"), s.hash("root2"), s.prueba()) else {
            continue;
        };
        let (a, b) = (s.numero("start"), s.numero("end"));
        if s.quiere_error() || a == b {
            continue; // con el subárbol vacío la raíz del árbol no se mira
        }
        let st = Subtree::new(a, b).unwrap();
        let t = s.numero("size");
        assert!(verify_consistency_proof(t, st, &p, &bit(r1), &r2).is_err());
        assert!(verify_consistency_proof(t, st, &p, &r1, &bit(r2)).is_err());
        n += 2;
    }
    for s in sondas("inclusion") {
        let (Some(h), Some(r), Some(p)) = (s.hash("leafHash"), s.hash("root"), s.prueba()) else {
            continue;
        };
        if s.quiere_error() {
            continue;
        }
        let (i, t) = (s.numero("leafIdx"), s.numero("treeSize"));
        assert!(verificar_inclusion(&bit(h), i, t, &p, &r).is_err());
        assert!(verificar_inclusion(&h, i, t, &p, &bit(r)).is_err());
        n += 2;
    }
    // Dos por positiva escribible: 5 de consistencia, 12 de subárbol (sin
    // las 4 de subárbol vacío) y 6 de inclusión.
    assert_eq!(n, 46);
}

/// Las raíces de RFC 6962 de `testonly/constants.go` del mismo repositorio
/// (`LeafInputs`, `RootHashes`): ocho hojas de longitudes 0 a 16 y las
/// nueve raíces, del árbol vacío al de ocho.
#[test]
fn raices_de_rfc6962() {
    const HOJAS: [&[u8]; 8] = [
        b"",
        b"\x00",
        b"\x10",
        b"\x20\x21",
        b"\x30\x31",
        b"\x40\x41\x42\x43",
        b"\x50\x51\x52\x53\x54\x55\x56\x57",
        b"\x60\x61\x62\x63\x64\x65\x66\x67\x68\x69\x6a\x6b\x6c\x6d\x6e\x6f",
    ];
    const RAICES: [&str; 9] = [
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d",
        "fac54203e7cc696cf0dfcb42c92a1d9dbaf70ad9e621f4bd8d98662f00e3c125",
        "aeb6bcfe274b70a14fb067a5e5578264db0fa9b51af5e0ba159158f329e06e77",
        "d37ee418976dd95753c1c73862b9398fa2a2cf9b4ff0fdfe8b30cd95209614b7",
        "4e3bbb1f7b478dcfe71fb631631519a3bca12c9aefca1612bfce4c13a86264d4",
        "76e67dadbcdf1e10e1b74ddc608abd2f98dfb16fbce75277b5232a127f2087ef",
        "ddb89be403809e325750d3d263cd78929c2942b7942a34b77e122c9594a74c8c",
        "5dc9da79a70659a9ad559cb701ded9a2ab9d823aad2f4960cfe370eff4604328",
    ];
    let hojas: Vec<HashValue> = HOJAS.iter().map(|d| hash_leaf(d)).collect();
    for (n, esperada) in RAICES.iter().enumerate() {
        let hex: String = mth(&hojas[..n])
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(&hex, esperada, "raíz de {n} hojas");
    }
}
