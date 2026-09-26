//! # Las reglas del arbol de RECIBOS DE RECEPCION: era, posicion, hoja y ventana
//!
//! ## Por que esto vive AQUI, y en un solo sitio
//!
//! El **constructor** del arbol sera del nodo (E2c, molde `vista_acuses`);
//! el **verificador** que recompone la raiz desde recibos servidos es de
//! este crate. Si la regla de que recibo cae en que era —o el indice que
//! ocupa— viviera solo en el constructor, aqui se escribiria otra version
//! y habria **dos**. Es el argumento de `acuses` desde §274, literal.
//!
//! ## ⚠️ La era NO se computa como la epoca del acuse
//!
//! `epoca_de_acuse(seq)` toma el `seq` de una ENTRADA del diario. La era
//! toma el `seq` de la ULTIMA CABEZA FIRMADA en el instante de recibir
//! (RFC-0010, D-D), que es un hecho de ese instante y que el verificador
//! **no tiene**: le llega la era ya fijada, DENTRO del recibo. Misma
//! aritmetica, argumento de otra naturaleza.
//!
//! Por eso [`era_de_recibo`] y [`hoja_de_recibo`] estan **separadas** y no
//! fundidas como en el molde. Fundirlas obligaria al verificador a
//! sostener un dato que nadie le manda.
//!
//! ## El borde `[Q, R)`, con Q INCLUSIVO
//!
//! Igual que la epoca del acuse, y por el mismo borde: con `Q` exclusivo
//! **la recepcion numero uno no perteneceria a ninguna era** (D-C).
//!
//! ## Denso — y por eso SIN cruce
//!
//! `consumos` cruza la posicion porque su arbol es disperso y el mismo
//! camino sirve a una posicion ocupada y a una vacia. Aqui el indice es
//! **denso desde cero** (`rx - Q`): la posicion se DERIVA de dos cabezas
//! firmadas, no se recibe. No hay ambiguedad que cruzar, y la ausencia de
//! cruce es una decision leida, no un olvido.
//!
//! ## ⚠️ La ventana es ARITMETICA, y no es prueba de ausencia
//!
//! [`dentro_de_ventana`] dice si `S - e <= N` sobre dos numeros firmados.
//! El veredicto 3 del D-F —«no resuelta en la ventana»— **no es una prueba
//! criptografica de ausencia**: probar que algo no esta en ninguna de `N`
//! epocas exigiria las `N` epocas enteras. Es evidencia OPONIBLE, y el RFC
//! lo dice con todas las letras. Esta funcion no dice mas que eso.

use zk_ssl_hash::recibo_digest;
// Los tipos viajan ya re-exportados por `acuses` (§274): un solo cable, y
// no se abre un segundo.
use crate::acuses::Digest;

/// ¿Cae la recepcion `rx` en la era `[limite_anterior, limite)`?
///
/// `limite_anterior` = `recep_count` de la cabeza anterior (0 si no hay
/// ninguna); `limite` = `recep_count` de la cabeza que cierra la era. El
/// limite inferior **no se firma**: lo tiene el titular que custodia dos
/// cabezas consecutivas (D-C).
pub fn pertenece_a_era(rx: u64, limite_anterior: u64, limite: u64) -> bool {
    limite_anterior <= rx && rx < limite
}

/// La posicion de la hoja dentro del arbol de su era: densa desde 0.
pub fn indice_de_recibo(rx: u64, limite_anterior: u64) -> u64 {
    rx - limite_anterior
}

/// La era que el recibo declara: **la primera cabeza que puede
/// contenerlo** (D-D). Se computa EN LA RECEPCION, sobre el `seq` de la
/// ultima cabeza firmada — no sobre el `seq` de una entrada del diario.
///
/// ⚠️ Atar la era a la cabeza que acabe conteniendo el recibo pondria el
/// valor de la evidencia en manos del acusado: el titular no podria fijar
/// su recibo hasta que el operador decidiera.
pub fn era_de_recibo(seq_ultima_cabeza_firmada: u64) -> u64 {
    seq_ultima_cabeza_firmada + 1
}

/// La hoja: `recibo_digest(hash_prueba, era, n)`, con la era como DATO.
///
/// ⚠️ `hash_prueba` va **con la longitud codificada** (§116, §121): lo hace
/// `digest_of_proof` desde su `v2`. Quien componga la hoja con otro resumen
/// de la prueba se sale del contrato.
///
/// ⚠️ `n` va **dentro** de la hoja y viaja firmado en la cabeza: una `n`
/// mentida en una respuesta produce una hoja que no verifica contra la raiz
/// recompuesta con la `n` FIRMADA.
pub fn hoja_de_recibo(hash_prueba: Digest, era: u64, n: u64) -> Digest {
    recibo_digest(hash_prueba, era, n)
}

/// ¿Sigue viva la promesa? `S - e <= N`, con `S` el `seq` de una cabeza
/// firmada y `e` la era del recibo (D-D).
///
/// ⚠️ Un `false` NO prueba que el recibo no se resolvio: prueba que la
/// ventana expiro. Ver la cabecera del modulo.
pub fn dentro_de_ventana(era: u64, seq_cierre: u64, n: u64) -> bool {
    // `seq_cierre < era` es una cabeza ANTERIOR a la era: la ventana no ha
    // empezado a correr, luego no ha expirado. Se DICE, no se satura en
    // silencio.
    if seq_cierre < era {
        return true;
    }
    seq_cierre - era <= n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acuses::{as_digest, hoja_de_acuse};

    #[test]
    fn la_era_incluye_q_y_excluye_r() {
        // La cabeza que cierra lleva recep_count = R, y contiene rx < R.
        assert!(pertenece_a_era(5, 5, 9), "el limite anterior pertenece");
        assert!(pertenece_a_era(8, 5, 9), "la ultima antes del cierre pertenece");
        assert!(!pertenece_a_era(9, 5, 9), "el cierre mismo NO pertenece");
        assert!(!pertenece_a_era(4, 5, 9), "lo anterior a Q es de otra era");
    }

    #[test]
    fn la_primera_era_cubre_la_recepcion_numero_uno() {
        // El borde que FUERZA la convencion (D-C): con Q exclusivo, la
        // recepcion numero uno no perteneceria a NINGUNA era.
        assert!(pertenece_a_era(0, 0, 3), "la recepcion 0 pertenece a la primera era");
        assert_eq!(indice_de_recibo(0, 0), 0, "y ocupa la posicion 0");
    }

    #[test]
    fn el_indice_es_denso_y_reversible() {
        // Denso desde 0, y rx se recupera de (Q, indice): cualquiera
        // reconstruye posiciones desde dos cabezas firmadas, sin datos extra.
        for rx in 5..9 {
            let i = indice_de_recibo(rx, 5);
            assert_eq!(i, rx - 5);
            assert_eq!(5 + i, rx, "el indice no es reversible");
        }
    }

    #[test]
    fn la_era_es_la_primera_cabeza_que_puede_contenerlo() {
        // S+1, y sobre la ULTIMA CABEZA FIRMADA, no sobre un seq del diario.
        assert_eq!(era_de_recibo(0), 1);
        assert_eq!(era_de_recibo(41), 42);
    }

    #[test]
    fn la_hoja_liga_prueba_era_y_n() {
        let hp = as_digest(0xA11CE);
        let h = hoja_de_recibo(hp, 100, 1_440);
        assert_ne!(h, hoja_de_recibo(hp, 101, 1_440), "otra era, misma hoja");
        assert_ne!(h, hoja_de_recibo(hp, 100, 720), "otro n, misma hoja");
        assert_ne!(h, hoja_de_recibo(as_digest(0xBEEF), 100, 1_440), "otra prueba, misma hoja");
        assert_eq!(h, hoja_de_recibo(hp, 100, 1_440), "no determinista");
    }

    #[test]
    fn un_recibo_no_pasa_por_acuse() {
        // D-A y D-B en un assert: dos objetos, dos dominios. Se comparan con
        // los MISMOS numeros -el acuse cuya epoca cae exactamente en la era-,
        // que es el unico caso en que una colision seria posible.
        let hp = as_digest(0x5EC0);
        let era = 100;
        assert_ne!(
            hoja_de_recibo(hp, era, 1_440),
            hoja_de_acuse(hp, era - 1, 1_440),
            "un recibo pasa por acuse: los dominios no separan"
        );
    }

    #[test]
    fn la_convencion_del_borde_coincide_hoy_con_la_del_acuse() {
        // Los dos arboles usan HOY el mismo borde [Q, R). No se comparte la
        // funcion a proposito (dos objetos, dos convenciones): se CRUZA, para
        // que el dia que una cambie este testigo lo NOMBRE en vez de que la
        // divergencia viaje muda.
        for x in 0..12u64 {
            for q in 0..6u64 {
                for r in q..12u64 {
                    assert_eq!(
                        pertenece_a_era(x, q, r),
                        crate::acuses::pertenece(x, q, r),
                        "el borde de la era y el de la epoca han divergido en ({x}, {q}, {r})"
                    );
                }
            }
        }
    }

    #[test]
    fn la_ventana_cuenta_cabezas_y_no_satura_en_silencio() {
        let n = 1_440;
        assert!(dentro_de_ventana(100, 100 + n, n), "S - e == N esta DENTRO");
        assert!(!dentro_de_ventana(100, 100 + n + 1, n), "S - e == N+1 esta FUERA");
        assert!(dentro_de_ventana(100, 100, n), "la propia era esta dentro");
        // Una cabeza ANTERIOR a la era: la ventana no ha empezado a correr.
        // Sin este caso, un `seq_cierre - era` desbordaria y en release daria
        // un numero enorme que caeria FUERA de la ventana, al reves de lo cierto.
        assert!(dentro_de_ventana(100, 99, n), "una cabeza anterior a la era no ha expirado");
        assert!(dentro_de_ventana(100, 0, n), "ni la cabeza cero");
    }
}
