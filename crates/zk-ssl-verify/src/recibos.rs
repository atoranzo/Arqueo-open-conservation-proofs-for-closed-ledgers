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
//! toma el **indice XMSS** de la ULTIMA CABEZA FIRMADA en el instante de
//! recibir (RFC-0010, D-D), que es un hecho de ese instante y que el
//! verificador **no tiene**: le llega la era ya fijada, DENTRO del recibo.
//!
//! ⚠️ CORRECCION (§567), citada y no borrada. Decia: <<La era toma el `seq`
//! de la ULTIMA CABEZA FIRMADA>>. El `seq` de una cabeza es `log.len()`
//! -el reloj del §340-, y el latido emite cabeza aunque no haya
//! transiciones: dos cabezas seguidas llevan el MISMO `seq`. Contar la
//! ventana en `seq` contaba ENTRADAS APLICADAS, no cabezas, y un censor que
//! dejara de aplicar CONGELABA el reloj: la ventana no expiraba nunca. El
//! indice XMSS lo quema UNA firma de cabeza -la clave del nodo solo firma
//! cabezas- y cuenta lo que la D-F promete, <<N cabezas firmadas>>. Decision
//! del autor, sesion 193.
//!
//! Por eso [`era_de_recibo`] y [`hoja_de_recibo`] estan **separadas** y no
//! fundidas como en el molde. Fundirlas obligaria al verificador a
//! sostener un dato que nadie le manda.
//!
//! ## El borde `(Q, R]`, con Q EXCLUSIVO y R INCLUSIVO
//!
//! `R` es el `recep_count` de la cabeza que cierra la era: las recepciones
//! que el nodo habia evaluado al componerla, que es tambien el ULTIMO `rx`
//! reservado, porque el contador empieza en 1 (`reservar` devuelve
//! `actual + 1`). `Q` es el de la cabeza anterior, 0 en el genesis. La era
//! son los `rx` con `Q < rx <= R`, y el genesis `recep_count = 0` sigue
//! siendo el arbol vacio.
//!
//! ⚠️ CORRECCION (§567), citada y no borrada. Este borde era `[Q, R)` <<igual
//! que la epoca del acuse>>, con el argumento de que con `Q` exclusivo <<la
//! recepcion numero uno no perteneceria a ninguna era>>. Razonaba con una
//! recepcion 0 que el contador NO da: con `[Q, R)` y `R = actual()`, la
//! ULTIMA recepcion de cada era caia fuera de su arbol. La convencion del
//! recibo es la del acuse aplicada a `rx - 1`, y el testigo del cruce lo
//! dice asi. Decision del autor, sesion 193.
//!
//! ## Denso — y por eso SIN cruce
//!
//! `consumos` cruza la posicion porque su arbol es disperso y el mismo
//! camino sirve a una posicion ocupada y a una vacia. Aqui el indice es
//! **denso desde cero** (`rx - Q - 1`): la posicion se DERIVA de dos cabezas
//! firmadas, no se recibe. No hay ambiguedad que cruzar, y la ausencia de
//! cruce es una decision leida, no un olvido.
//!
//! ## ⚠️ La ventana es ARITMETICA, y no es prueba de ausencia
//!
//! [`dentro_de_ventana`] dice si `S - e <= N` sobre dos numeros firmados:
//! `S` el indice XMSS de una cabeza firmada y `e` la era del recibo.
//! El veredicto 3 del D-F —«no resuelta en la ventana»— **no es una prueba
//! criptografica de ausencia**: probar que algo no esta en ninguna de `N`
//! epocas exigiria las `N` epocas enteras. Es evidencia OPONIBLE, y el RFC
//! lo dice con todas las letras. Esta funcion no dice mas que eso.

use zk_ssl_hash::recibo_digest;
// Los tipos viajan ya re-exportados por `acuses` (§274): un solo cable, y
// no se abre un segundo.
use crate::acuses::Digest;

/// ¿Cae la recepcion `rx` en la era `(limite_anterior, limite]`?
///
/// `limite_anterior` = `recep_count` de la cabeza anterior (0 si no hay
/// ninguna); `limite` = `recep_count` de la cabeza que cierra la era, que es
/// el ULTIMO `rx` que el nodo reservo antes de componerla. El limite
/// inferior **no se firma**: lo tiene el titular que custodia dos cabezas
/// consecutivas (D-C).
pub fn pertenece_a_era(rx: u64, limite_anterior: u64, limite: u64) -> bool {
    limite_anterior < rx && rx <= limite
}

/// La posicion de la hoja dentro del arbol de su era: densa desde 0. El
/// primer `rx` de la era es `limite_anterior + 1` y ocupa la posicion 0.
///
/// ⚠️ Llamarla con un `rx` que no pertenece es un error del llamante: con
/// `rx <= limite_anterior` no hay posicion, y se DICE con un `panic` en vez
/// de devolver un numero enorme por desbordamiento.
pub fn indice_de_recibo(rx: u64, limite_anterior: u64) -> u64 {
    assert!(
        rx > limite_anterior,
        "indice_de_recibo: rx {rx} no es posterior al limite anterior {limite_anterior}"
    );
    rx - limite_anterior - 1
}

/// La era que el recibo declara: **la primera cabeza que puede
/// contenerlo** (D-D). Se computa EN LA RECEPCION, sobre el **indice XMSS**
/// de la ultima cabeza firmada — no sobre su `seq`, que es `log.len()` y no
/// avanza sin transiciones (ver la cabecera).
///
/// ⚠️ Atar la era a la cabeza que acabe conteniendo el recibo pondria el
/// valor de la evidencia en manos del acusado: el titular no podria fijar
/// su recibo hasta que el operador decidiera.
pub fn era_de_recibo(indice_ultima_cabeza_firmada: u64) -> u64 {
    indice_ultima_cabeza_firmada + 1
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

/// ¿Sigue viva la promesa? `S - e <= N`, con `S` el **indice XMSS** de una
/// cabeza firmada y `e` la era del recibo (D-D): `S - e` cuenta CABEZAS
/// FIRMADAS, mas los indices huerfanos -quemados sin firma-, que solo
/// acortan la ventana y cuentan en contra del operador que los quemo.
///
/// ⚠️ Un `false` NO prueba que el recibo no se resolvio: prueba que la
/// ventana expiro. Ver la cabecera del modulo.
pub fn dentro_de_ventana(era: u64, indice_cierre: u64, n: u64) -> bool {
    // `indice_cierre < era` es una cabeza ANTERIOR a la era: la ventana no
    // ha empezado a correr, luego no ha expirado. Se DICE, no se satura en
    // silencio.
    if indice_cierre < era {
        return true;
    }
    indice_cierre - era <= n
}

/// **La profundidad del arbol de recibos** (RFC-0010 E4, §573): la del `SparseTree::new()` con
/// que el nodo lo compone, `TREE_DEPTH` = 32. Un camino de recibo mide EXACTAMENTE esto en sus dos
/// lados: la profundidad no la elige quien sirve el camino. El nodo ata esta constante a su arbol
/// con un testigo (`vista_recibos`), como `ACCOUNTS_DEPTH` y `FROZEN_DEPTH` a los suyos.
pub const RECEP_DEPTH: usize = 32;

/// Sube la hoja de un recibo hasta su raiz, **o `None`** si el camino no mide [`RECEP_DEPTH`] en
/// sus dos lados, si la posicion no cabe en el arbol, o si sus lados no son los de `indice` -la
/// posicion densa `rx - Q - 1` de [`indice_de_recibo`]-. Molde de `congelados::raiz_de_hoja` con
/// el cruce de `congelados::cruza_indice`: un camino de otra posicion no prueba nada de este recibo.
pub fn raiz_de_camino_de_recibo(
    indice: u64,
    hoja: Digest,
    siblings: &[Digest],
    is_right: &[bool],
) -> Option<Digest> {
    if siblings.len() != RECEP_DEPTH
        || is_right.len() != RECEP_DEPTH
        || indice >= (1u64 << RECEP_DEPTH)
    {
        return None;
    }
    if (0..RECEP_DEPTH).any(|i| is_right[i] != ((indice >> i) & 1 == 1)) {
        return None;
    }
    Some(zk_ssl_hash::path_root(hoja, siblings, is_right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_camino_de_un_recibo_mide_recep_depth_y_cruza_su_posicion() {
        // §573: la profundidad y los lados los fija la regla, no quien sirve el camino.
        let hoja = zk_ssl_hash::as_digest(77);
        let hermanos: Vec<Digest> = (0..RECEP_DEPTH as u64).map(zk_ssl_hash::as_digest).collect();
        let indice = 5u64;
        let lados: Vec<bool> = (0..RECEP_DEPTH).map(|i| (indice >> i) & 1 == 1).collect();
        assert_eq!(
            raiz_de_camino_de_recibo(indice, hoja, &hermanos, &lados),
            Some(zk_ssl_hash::path_root(hoja, &hermanos, &lados)),
            "el camino bueno sube"
        );
        assert_eq!(
            raiz_de_camino_de_recibo(indice, hoja, &hermanos[1..], &lados[1..]),
            None,
            "un camino truncado no sube"
        );
        let mut otro = lados.clone();
        otro[0] = !otro[0];
        assert_eq!(
            raiz_de_camino_de_recibo(indice, hoja, &hermanos, &otro),
            None,
            "los lados de otra posicion no cruzan"
        );
        assert_eq!(
            raiz_de_camino_de_recibo(1u64 << RECEP_DEPTH, hoja, &hermanos, &lados),
            None,
            "una posicion que no cabe no cruza con nada"
        );
    }
    use crate::acuses::{as_digest, hoja_de_acuse};

    #[test]
    fn la_era_excluye_q_e_incluye_r() {
        // La cabeza que cierra lleva recep_count = R -el ULTIMO rx reservado
        // al componerla- y contiene Q < rx <= R (§567).
        assert!(!pertenece_a_era(5, 5, 9), "el limite anterior es de la era ANTERIOR");
        assert!(pertenece_a_era(6, 5, 9), "el primero tras Q pertenece");
        assert!(pertenece_a_era(9, 5, 9), "el cierre mismo -el ultimo rx- pertenece");
        assert!(!pertenece_a_era(10, 5, 9), "lo posterior a R es de la era siguiente");
    }

    #[test]
    fn la_primera_era_cubre_la_recepcion_numero_uno() {
        // El contador empieza en 1: la recepcion numero uno es rx = 1, y
        // con el genesis Q = 0 pertenece a la primera era y ocupa la
        // posicion 0. El borde viejo razonaba con un rx = 0 que no existe.
        assert!(pertenece_a_era(1, 0, 3), "la recepcion 1 pertenece a la primera era");
        assert_eq!(indice_de_recibo(1, 0), 0, "y ocupa la posicion 0");
        assert!(!pertenece_a_era(0, 0, 3), "no hay recepcion 0 que meter");
    }

    #[test]
    fn el_indice_es_denso_y_reversible() {
        // Denso desde 0, y rx se recupera de (Q, indice): cualquiera
        // reconstruye posiciones desde dos cabezas firmadas, sin datos extra.
        for rx in 6..=9 {
            let i = indice_de_recibo(rx, 5);
            assert_eq!(i, rx - 6);
            assert_eq!(5 + 1 + i, rx, "el indice no es reversible");
        }
    }

    #[test]
    #[should_panic(expected = "no es posterior al limite anterior")]
    fn un_rx_que_no_pertenece_no_tiene_indice() {
        // Sin la guarda, 5 - 5 - 1 desbordaria: en release un numero enorme
        // que ningun arbol tiene; en debug un panic sin nombre.
        let _ = indice_de_recibo(5, 5);
    }

    #[test]
    fn la_era_es_la_primera_cabeza_que_puede_contenerlo() {
        // Indice + 1, y sobre el INDICE XMSS de la ULTIMA CABEZA FIRMADA, no
        // sobre un seq (§567).
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
    fn la_convencion_del_borde_es_la_del_acuse_sobre_rx_menos_uno() {
        // HASTA EL §567 este testigo exigia que los dos bordes COINCIDIERAN,
        // y existia para NOMBRAR el dia en que divergieran. Ese dia es este:
        // el acuse cuenta `seq` desde 0 y el recibo `rx` desde 1, y el borde
        // del recibo es EXACTAMENTE el del acuse aplicado a `rx - 1`. Se
        // sigue cruzando -no se comparte la funcion: dos objetos, dos
        // convenciones-, ahora con la relacion que es cierta.
        for x in 1..13u64 {
            for q in 0..6u64 {
                for r in q..12u64 {
                    assert_eq!(
                        pertenece_a_era(x, q, r),
                        crate::acuses::pertenece(x - 1, q, r),
                        "el borde de la era no es el de la epoca sobre rx - 1 en ({x}, {q}, {r})"
                    );
                    if pertenece_a_era(x, q, r) {
                        assert_eq!(
                            indice_de_recibo(x, q),
                            crate::acuses::indice_de_hoja(x - 1, q),
                            "la posicion no es la del acuse sobre rx - 1 en ({x}, {q})"
                        );
                    }
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
