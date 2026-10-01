//! # El árbol del medio (RFC-0013 D-B)
//!
//! La hoja `i` es la huella del ancla `i` (32 bytes, RFC-0012 D-A); la hoja
//! del árbol es `SHA-256(0x00 || huella)` y los nodos, `SHA-256(0x01 || i
//! || d)`: el MTH de RFC 6962 que cualquier testigo genérico recompone. No
//! es un segundo productor de la historia: el árbol canónico sigue siendo
//! el de cabezas (§292, Rescue Prime sobre Goldilocks), y este solo afirma
//! que **la secuencia de anclas publicadas creció y no se reescribió**.
//!
//! ## Las dos comprobaciones de un testigo, con sus bordes
//!
//! [`verificar_inclusion`] y [`verificar_consistencia`] son las de RFC 9162
//! sobre el árbol entero (`[0, tamaño)`), y delegan en
//! [`crate::subtree`]. Lo que añaden son **los bordes**, que es donde dos
//! implementaciones del mismo RFC discrepan, y se fijan como los fija
//! `transparency-dev/merkle` (`proof.VerifyInclusion`,
//! `proof.VerifyConsistency`), con cuyas 685 sondas se contrasta:
//!
//! - un índice fuera del árbol no tiene inclusión;
//! - un log no decrece: `viejo > nuevo` es un error, no una prueba vacía;
//! - **desde el árbol vacío no hay consistencia que probar**: `viejo = 0`
//!   es un error aquí, y `tlog-witness` lo trata en el protocolo (`old 0`
//!   sin líneas de prueba), no en el árbol;
//! - con `viejo = nuevo` la prueba es vacía y las dos raíces, iguales.
//!
//! ## ⚠️ Lo que este módulo NO hace
//!
//! - **No persiste.** El árbol se reconstruye de las anclas, que ya
//!   persiste el nodo (RFC-0012); dónde vive el del publicador es E3.
//! - **Es O(n) por consulta**: el único proveedor es
//!   [`crate::subtree::LeafHashes`], la recursión literal. El log con nodos
//!   en caché de mtc-core (`log::IssuanceLog`) no se copió porque es de
//!   entradas MTC; si el publicador lo necesita, se mide en E3.
//! - **No firma ni lee notas**: la nota `checkpoint` y la firma `0x06` son
//!   E2b.

use crate::hash::{hash_leaf, HashValue};
use crate::subtree::{
    consistency_proof, inclusion_proof, mth, verify_consistency_proof, verify_inclusion_proof,
    LeafHashes, Subtree, SubtreeError,
};

/// Bytes de la huella de un ancla (RFC-0012 D-A).
pub const BYTES_HUELLA: usize = 32;

/// La hoja del medio para el ancla de huella `huella`:
/// `MTH({huella}) = SHA-256(0x00 || huella)`.
pub fn hoja(huella: &[u8; BYTES_HUELLA]) -> HashValue {
    hash_leaf(huella)
}

/// Lo que puede fallar en el medio.
///
/// ⚠️ Los tres primeros son **preguntas mal hechas** (un tamaño o un índice
/// que no casan); `Arbol` es **la respuesta del árbol**: un camino mal
/// formado o una raíz distinta, que es la prueba de que algo no estaba o se
/// reescribió.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorDelMedio {
    /// El índice no cae en un árbol de `tamano` hojas, o el tamaño pedido
    /// excede lo que hay.
    FueraDelArbol { posicion: u64, tamano: u64 },
    /// `viejo > nuevo`: un log no decrece.
    Decrece { viejo: u64, nuevo: u64 },
    /// Consistencia desde el árbol vacío: no hay nada que probar.
    DesdeVacio,
    /// Lo que dice el árbol.
    Arbol(SubtreeError),
}

impl core::fmt::Display for ErrorDelMedio {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorDelMedio::FueraDelArbol { posicion, tamano } => {
                write!(f, "{posicion} no cae en un árbol de {tamano} hojas")
            }
            ErrorDelMedio::Decrece { viejo, nuevo } => {
                write!(f, "el log decrece: de {viejo} a {nuevo} hojas")
            }
            ErrorDelMedio::DesdeVacio => {
                write!(f, "no hay consistencia que probar desde el árbol vacío")
            }
            ErrorDelMedio::Arbol(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ErrorDelMedio {}

impl From<SubtreeError> for ErrorDelMedio {
    fn from(e: SubtreeError) -> Self {
        ErrorDelMedio::Arbol(e)
    }
}

/// El árbol de anclas del medio, en memoria: solo crece.
#[derive(Clone, Debug, Default)]
pub struct ArbolDelMedio {
    hojas: LeafHashes,
}

impl ArbolDelMedio {
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// Añade el ancla de huella `huella` y devuelve su posición.
    pub fn anadir(&mut self, huella: &[u8; BYTES_HUELLA]) -> u64 {
        self.hojas.0.push(hoja(huella));
        self.hojas.0.len() as u64 - 1
    }

    /// Cuántas anclas tiene.
    pub fn tamano(&self) -> u64 {
        self.hojas.0.len() as u64
    }

    /// La raíz del árbol entero; la del vacío es `SHA-256("")`.
    pub fn raiz(&self) -> HashValue {
        mth(&self.hojas.0)
    }

    /// La raíz del árbol tal como era con `tamano` hojas.
    pub fn raiz_en(&self, tamano: u64) -> Result<HashValue, ErrorDelMedio> {
        self.comprobar_tamano(tamano)?;
        Ok(mth(&self.hojas.0[..tamano as usize]))
    }

    /// La prueba de inclusión del ancla `indice` en el árbol de `tamano`
    /// hojas (RFC 9162 `PATH`).
    pub fn prueba_de_inclusion(
        &self,
        indice: u64,
        tamano: u64,
    ) -> Result<Vec<HashValue>, ErrorDelMedio> {
        self.comprobar_tamano(tamano)?;
        if indice >= tamano {
            return Err(ErrorDelMedio::FueraDelArbol {
                posicion: indice,
                tamano,
            });
        }
        Ok(inclusion_proof(&self.hojas, entero(tamano), indice)?)
    }

    /// La prueba de consistencia de `viejo` a `nuevo` hojas (RFC 9162
    /// `PROOF`), con los mismos bordes que [`verificar_consistencia`].
    pub fn prueba_de_consistencia(
        &self,
        viejo: u64,
        nuevo: u64,
    ) -> Result<Vec<HashValue>, ErrorDelMedio> {
        self.comprobar_tamano(nuevo)?;
        bordes_de_consistencia(viejo, nuevo)?;
        Ok(consistency_proof(&self.hojas, nuevo, entero(viejo))?)
    }

    fn comprobar_tamano(&self, tamano: u64) -> Result<(), ErrorDelMedio> {
        if tamano > self.tamano() {
            return Err(ErrorDelMedio::FueraDelArbol {
                posicion: tamano,
                tamano: self.tamano(),
            });
        }
        Ok(())
    }
}

/// `[0, tamano)`: un subárbol válido para todo `tamano` (el borrador exige
/// que `start` sea múltiplo de `BIT_CEIL(end - start)`, y 0 lo es de todo).
fn entero(tamano: u64) -> Subtree {
    Subtree {
        start: 0,
        end: tamano,
    }
}

fn bordes_de_consistencia(viejo: u64, nuevo: u64) -> Result<(), ErrorDelMedio> {
    if viejo > nuevo {
        return Err(ErrorDelMedio::Decrece { viejo, nuevo });
    }
    if viejo == 0 {
        return Err(ErrorDelMedio::DesdeVacio);
    }
    Ok(())
}

/// **Verifica la inclusión** de la hoja `hoja` (ya con su prefijo `0x00`)
/// en la posición `indice` del árbol de `tamano` hojas y raíz `raiz`.
pub fn verificar_inclusion(
    hoja: &HashValue,
    indice: u64,
    tamano: u64,
    prueba: &[HashValue],
    raiz: &HashValue,
) -> Result<(), ErrorDelMedio> {
    if indice >= tamano {
        return Err(ErrorDelMedio::FueraDelArbol {
            posicion: indice,
            tamano,
        });
    }
    Ok(verify_inclusion_proof(
        hoja,
        entero(tamano),
        indice,
        prueba,
        raiz,
    )?)
}

/// **Verifica la consistencia** entre el árbol de `viejo` hojas y raíz
/// `raiz_vieja` y el de `nuevo` hojas y raíz `raiz_nueva`: que el nuevo
/// CONTIENE al viejo como prefijo. Es lo que un testigo comprueba antes de
/// cofirmar (`tlog-witness`, `add-checkpoint`).
///
/// Con `viejo = nuevo`, el verificador del borrador ya exige la prueba
/// vacía y las dos raíces iguales: las sondas lo comprueban.
pub fn verificar_consistencia(
    viejo: u64,
    nuevo: u64,
    prueba: &[HashValue],
    raiz_vieja: &HashValue,
    raiz_nueva: &HashValue,
) -> Result<(), ErrorDelMedio> {
    bordes_de_consistencia(viejo, nuevo)?;
    Ok(verify_consistency_proof(
        nuevo,
        entero(viejo),
        prueba,
        raiz_vieja,
        raiz_nueva,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::{hash_empty, sha256};

    /// Huellas distintas y deterministas, sin RNG.
    fn huella(i: u64) -> [u8; BYTES_HUELLA] {
        sha256(&i.to_be_bytes())
    }

    fn arbol(n: u64) -> ArbolDelMedio {
        let mut a = ArbolDelMedio::nuevo();
        for i in 0..n {
            assert_eq!(a.anadir(&huella(i)), i);
        }
        a
    }

    #[test]
    fn el_vacio_es_sha256_de_nada_y_la_hoja_lleva_su_prefijo() {
        assert_eq!(ArbolDelMedio::nuevo().raiz(), hash_empty());
        // La hoja NO es la huella ni su SHA-256 desnudo: lleva el 0x00.
        let h = huella(7);
        assert_ne!(hoja(&h), h);
        assert_ne!(hoja(&h), sha256(&h));
        // Un árbol de una hoja tiene por raíz esa hoja.
        assert_eq!(arbol(1).raiz(), hoja(&huella(0)));
    }

    /// Todo índice de todo tamaño hasta 40, y toda pareja `viejo < nuevo`:
    /// lo generado verifica, y la misma prueba contra otra raíz o con un
    /// hash de más o de menos, no.
    #[test]
    fn lo_generado_verifica_y_lo_tocado_no() {
        let a = arbol(40);
        for n in 1..=40 {
            let raiz = a.raiz_en(n).unwrap();
            for i in 0..n {
                let p = a.prueba_de_inclusion(i, n).unwrap();
                let h = hoja(&huella(i));
                verificar_inclusion(&h, i, n, &p, &raiz).unwrap();
                assert!(verificar_inclusion(&hoja(&huella(i + 1)), i, n, &p, &raiz).is_err());
                let mut larga = p.clone();
                larga.push(raiz);
                assert!(verificar_inclusion(&h, i, n, &larga, &raiz).is_err());
                if !p.is_empty() {
                    assert!(verificar_inclusion(&h, i, n, &p[1..], &raiz).is_err());
                }
            }
            for m in 1..n {
                let vieja = a.raiz_en(m).unwrap();
                let p = a.prueba_de_consistencia(m, n).unwrap();
                assert!(!p.is_empty(), "{m} -> {n}");
                verificar_consistencia(m, n, &p, &vieja, &raiz).unwrap();
                // Las raíces cruzadas, o un tamaño viejo que no es el suyo.
                assert!(verificar_consistencia(m, n, &p, &raiz, &vieja).is_err());
                if m + 1 < n {
                    assert!(verificar_consistencia(m + 1, n, &p, &vieja, &raiz).is_err());
                }
                assert!(verificar_consistencia(m, n, &p[1..], &vieja, &raiz).is_err());
            }
        }
    }

    /// Un árbol reescrito —la misma longitud, una huella cambiada— no es
    /// consistente con la raíz que el testigo recuerda (D-D); uno que
    /// retrocede es `Decrece` (`los_bordes`).
    #[test]
    fn una_huella_reescrita_rompe_la_consistencia() {
        let a = arbol(12);
        let vieja = a.raiz_en(5).unwrap();
        let mut b = ArbolDelMedio::nuevo();
        for i in 0..12 {
            b.anadir(&huella(if i == 3 { 99 } else { i }));
        }
        let p = b.prueba_de_consistencia(5, 12).unwrap();
        assert_eq!(
            verificar_consistencia(5, 12, &p, &vieja, &b.raiz()),
            Err(ErrorDelMedio::Arbol(SubtreeError::HashMismatch))
        );
        // El árbol reescrito es consistente consigo mismo: lo que lo delata
        // no es la prueba, es la raíz vieja que el testigo guardó.
        verificar_consistencia(5, 12, &p, &b.raiz_en(5).unwrap(), &b.raiz()).unwrap();
    }

    #[test]
    fn los_bordes() {
        let a = arbol(8);
        let r = a.raiz();
        assert_eq!(
            a.prueba_de_consistencia(0, 8),
            Err(ErrorDelMedio::DesdeVacio)
        );
        assert_eq!(
            verificar_consistencia(0, 8, &[], &hash_empty(), &r),
            Err(ErrorDelMedio::DesdeVacio)
        );
        assert_eq!(
            a.prueba_de_consistencia(5, 4),
            Err(ErrorDelMedio::Decrece { viejo: 5, nuevo: 4 })
        );
        assert_eq!(
            a.prueba_de_consistencia(4, 9),
            Err(ErrorDelMedio::FueraDelArbol {
                posicion: 9,
                tamano: 8
            })
        );
        // viejo = nuevo: prueba vacía y raíces iguales, y nada más.
        assert_eq!(a.prueba_de_consistencia(8, 8), Ok(vec![]));
        verificar_consistencia(8, 8, &[], &r, &r).unwrap();
        assert!(verificar_consistencia(8, 8, &[r], &r, &r).is_err());
        assert!(verificar_consistencia(8, 8, &[], &hash_empty(), &r).is_err());
        assert_eq!(
            verificar_inclusion(&hoja(&huella(8)), 8, 8, &[], &r),
            Err(ErrorDelMedio::FueraDelArbol {
                posicion: 8,
                tamano: 8
            })
        );
    }
}
