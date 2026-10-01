//! # El árbol del medio del ancla (RFC-0013 E2a)
//!
//! El RFC-0013 publica cada ancla del RFC-0012 en un log de checkpoints
//! C2SP que testigos ajenos cofirman (D-A, D-D). Este crate es la parte de
//! ese medio que no necesita red ni claves: **el árbol SHA-256 de RFC 6962
//! cuyas hojas son las huellas de las anclas** (D-B), con sus pruebas de
//! inclusión y de consistencia. La nota `checkpoint` y su firma ML-DSA-44
//! de tipo `0x06` son E2b; el publicador, E3; el sobre del kit, E4.
//!
//! - [`hash`] y [`subtree`]: el árbol y las pruebas del borrador
//!   `draft-ietf-plants-merkle-tree-certs` (RFC 9162 extendido a subárboles
//!   `[start, end)`), copiados de mtc-core.
//! - [`medio`]: lo propio de aquí, el árbol de anclas y las dos
//!   comprobaciones que un testigo corre, con los bordes de RFC 9162.
//!
//! ## ⚠️ Copiado, no dependido (D-G, decidido en §631)
//!
//! `hash` y `subtree` son `src/hash.rs` y `src/subtree.rs` de mtc-core
//! (`github.com/atoranzo/mtc-core`, commit
//! `d3b0ca614e51f177a0c30f1d21abd87b91208f59`), del mismo autor y con la
//! misma licencia, **copiados byte a byte salvo las líneas marcadas
//! `ADAPTADO (§631)`**, y en inglés porque así se escribieron. El D-G dejaba
//! dos caminos; se copia por tres razones medidas:
//!
//! - mtc-core depende, sin `optional`, de `hbs-state` por git y, por defecto,
//!   de `ml-dsa`, y trae el formato X.509/MTC entero; el medio usa dos
//!   ficheros sin más dependencia que `sha2`, que ya estaba en el
//!   `Cargo.lock` del workspace.
//! - El kit (E4) tiene que poder reconstruirse desde este repositorio y
//!   crates.io; una dependencia por git a otro repositorio es una pieza
//!   más que un tercero tiene que ir a buscar.
//! - La premisa del D-G —«como se depende de `hbs-state`»— **era falsa**:
//!   ningún `Cargo.toml` ni el `Cargo.lock` de Arqueo nombran `hbs-state`
//!   (medido en §631). No había un precedente de dependencia por git que
//!   seguir.
//!
//! El precio, declarado: el mismo código vive en dos repositorios y puede
//! divergir. Lo que ata esta copia a su origen no es la confianza: son los
//! mismos vectores del IETF, que pasan aquí en el canon como pasan allí.
//!
//! ## Tres corpus, de tres manos
//!
//! - Los vectores acumulados del borrador del IETF: 65.058 casos sobre todo
//!   subárbol de todo árbol hasta 130 hojas (`tests/vectores_ietf.rs`).
//! - Sus vectores grandes, sobre árboles de hasta `2^64-1` hojas
//!   (`tests/vectores_grandes.rs`).
//! - Las 685 sondas de `transparency-dev/merkle`, 43 positivas y 642
//!   negativas: la biblioteca con la que el testigo de transparency-dev
//!   comprueba la consistencia (`tests/vectores_transparency.rs`).
//!
//! Procedencia y licencias de los tres, en `tests/vectores/README.md`.

pub mod hash;
pub mod medio;
pub mod subtree;
