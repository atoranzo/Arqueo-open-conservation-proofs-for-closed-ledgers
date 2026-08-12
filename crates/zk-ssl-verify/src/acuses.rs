//! # Las reglas del árbol de acuses: pertenencia, posición, época y hoja
//!
//! ## Por qué esto vive AQUÍ, y en un solo sitio
//!
//! El **constructor** del árbol es del nodo (§274, `vista_acuses`); el
//! **verificador** que recompute hojas desde entradas servidas será de este
//! crate (§275). Si la regla de qué entrada cae en qué época —o el índice
//! que ocupa— viviera sólo en el constructor, el verificador escribiría su
//! versión y habría **dos**. El borde de abajo es la prueba de que habrían
//! divergido.
//!
//! ## La convención: `P <= seq < S`, y el borde que la fuerza
//!
//! La cabeza con `seq = S` se compone **después** del último `append`
//! (`epoch_head()` toma `log.len()`), así que contiene exactamente las
//! entradas con `seq < S`. La época entre la cabeza `P` y la cabeza `S`
//! es `[P, S)`.
//!
//! ⚠️ **Con `P` exclusivo, la entrada 0 no pertenecería a ninguna época.**
//! Ese borde apareció al montar §274 — en dos definiciones, habría
//! divergido exactamente ahí.
//!
//! ## La época de la hoja es `seq + 1`, no `S`
//!
//! `seq + 1` es la **primera cabeza que puede contener** la entrada. La
//! alternativa —atar la cabeza real `S`— registra la *publicación*, que ya
//! es derivable del registro y del `chain_digest`, y **pone el valor de la
//! evidencia en manos del acusado**: el titular no podría fijar su hoja
//! hasta que el operador decidiera. Con `seq + 1` la hoja se computa **en
//! el apply** y queda fija para siempre — y una hoja que declara `seq + 1`
//! viviendo bajo la cabeza `S` hace legible `S − (seq + 1)` desde la hoja
//! y la cabeza solas: **la magnitud que la promesa acota**.
//!
//! ## El índice es denso, y la divergencia con §157 es deliberada
//!
//! `accounts.rs` coloca por `public_id mod capacidad` para que los índices
//! **no** sean enumerables. Aquí enumerable es justo lo que se quiere: la
//! época es pequeña, el árbol denso, y cualquiera reconstruye la posición
//! desde el registro y los límites del diario (§272) sin datos extra.

use zk_ssl_hash::acuse_digest;
// El nodo no depende de `zk-ssl-hash`: los tipos que las reglas usan
// viajan re-exportados desde aquí — un solo cable, también para tipos.
pub use zk_ssl_hash::{as_digest, Digest};

/// ¿Cae `seq` en la época `[limite_anterior, limite)`?
///
/// `limite_anterior` = `seq` de la cabeza anterior (0 si no hay ninguna);
/// `limite` = `seq` de la cabeza que cierra esta época.
pub fn pertenece(seq: u64, limite_anterior: u64, limite: u64) -> bool {
    limite_anterior <= seq && seq < limite
}

/// La posición de la hoja dentro del árbol de su época: densa desde 0.
pub fn indice_de_hoja(seq: u64, limite_anterior: u64) -> u64 {
    seq - limite_anterior
}

/// La época que el acuse declara: **la primera cabeza que puede
/// contenerlo**. Es el desfase de uno, en un solo sitio: `applied()`
/// devuelve `seq = n`, y una entrada con `seq = n` sólo está en cabezas
/// con `seq >= n + 1`.
pub fn epoca_de_acuse(seq: u64) -> u64 {
    seq + 1
}

/// La hoja: `acuse_digest(hash_prueba, epoca_de_acuse(seq), n)`.
///
/// ⚠️ `n` va **dentro** de la hoja (§270): cuando `n` viaje firmado en la
/// cabeza (§275), un operador que prometa otro `n` en la respuesta produce
/// una hoja que **no verifica** contra el árbol recomputado con el `n` de
/// la cabeza. La respuesta no puede mentir sobre `n` sin que el camino
/// falle.
pub fn hoja_de_acuse(hash_prueba: Digest, seq: u64, n: u64) -> Digest {
    acuse_digest(hash_prueba, epoca_de_acuse(seq), n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zk_ssl_hash::as_digest;

    #[test]
    fn pertenece_incluye_p_y_excluye_s() {
        // La cabeza S contiene seq < S: P entra, S queda para la siguiente.
        assert!(pertenece(5, 5, 9), "el limite anterior pertenece");
        assert!(pertenece(8, 5, 9), "la ultima antes de la cabeza pertenece");
        assert!(!pertenece(9, 5, 9), "la cabeza misma NO pertenece");
        assert!(!pertenece(4, 5, 9), "lo anterior a P es de otra epoca");
    }

    #[test]
    fn la_primera_epoca_cubre_desde_la_entrada_cero() {
        // El borde que fuerza la convencion: con P exclusivo, la entrada 0
        // no perteneceria a NINGUNA epoca.
        assert!(pertenece(0, 0, 3), "la entrada 0 pertenece a la primera epoca");
        assert_eq!(indice_de_hoja(0, 0), 0, "y ocupa la posicion 0");
    }

    #[test]
    fn el_indice_es_denso_y_reversible() {
        // Denso desde 0, y seq se recupera de (limite_anterior, indice):
        // cualquiera reconstruye posiciones sin datos extra.
        for seq in 5..9 {
            let i = indice_de_hoja(seq, 5);
            assert_eq!(i, seq - 5);
            assert_eq!(5 + i, seq, "el indice no es reversible");
        }
    }

    #[test]
    fn la_epoca_es_la_primera_cabeza_que_puede_contenerla() {
        // seq+1, no S: el valor lo fija el titular en el apply, no el
        // operador al cerrar. S - (seq+1) queda legible desde fuera.
        assert_eq!(epoca_de_acuse(0), 1);
        assert_eq!(epoca_de_acuse(41), 42);
    }

    #[test]
    fn la_hoja_liga_prueba_epoca_y_n() {
        let hp = as_digest(0xA11CE);
        let h = hoja_de_acuse(hp, 100, 1_440);
        assert_ne!(h, hoja_de_acuse(hp, 101, 1_440), "otra epoca, misma hoja");
        assert_ne!(h, hoja_de_acuse(hp, 100, 720), "otro n, misma hoja");
        assert_ne!(h, hoja_de_acuse(as_digest(0xBEEF), 100, 1_440), "otra prueba, misma hoja");
        assert_eq!(h, hoja_de_acuse(hp, 100, 1_440), "no determinista");
    }

    #[test]
    fn toda_entrada_tiene_hoja_delegadas_incluidas() {
        // La hoja de una delegada sale de su proof_digest compartido: es
        // computable hoy, inofensiva, y el dia que la nota 78 les de
        // pruebas reales sus acuses cobran sentido SIN cambio de formato.
        let compartido = as_digest(0x74DE);
        let a = hoja_de_acuse(compartido, 7, 1_440);
        let b = hoja_de_acuse(compartido, 8, 1_440);
        assert_ne!(a, b, "misma prueba compartida, epocas distintas: hojas distintas");
    }
}
