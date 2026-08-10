//! # Las primitivas de FORMATO, fuera del probador
//!
//! ⚠️ **El nombre del crate dice `hash`, y ya no es solo el hash.** Lo que
//! vive aquí es **lo que el nodo y un verificador independiente tienen que
//! componer IGUAL**: el hash 2-a-1, cómo se embebe un `u64`, cómo se sube
//! un camino Merkle, y **cómo se compone el digest de una cabeza**.
//!
//! > **Una decisión de formato tiene que tener UNA SOLA DEFINICIÓN**, por
//! > la misma razón que el hash: si el nodo y el verificador la componen
//! > distinto, **divergen en silencio**.
//!
//! §254 movió `native_merge` aquí y creyó cerrar el problema. Al escribir
//! el **recibo de inclusión** aparecieron **tres piezas más** que el
//! verificador necesitaba y no tenía: `as_digest` era **privada** en
//! `log.rs`, subir el camino vivía en `stark-experiment` **atado a
//! `TREE_DEPTH`**, y **la composición de `EpochHead::digest()`** estaba
//! solo en el nodo. **El problema no estaba resuelto: estaba destapado a
//! medias.**
//!
//! ## ⚠️ Por qué está separado: no es organización
//!
//! **§243** estableció que **un verificador independiente no compila el
//! servidor**: `zk-ssl-verify` solo depende de `xmss`, y hay compuerta que
//! lo vigila.
//!
//! Este crate **extiende esa regla un nivel más abajo: tampoco compila el
//! probador.**
//!
//! El problema apareció al diseñar el **recibo de inclusión**: cualquier
//! cosa con forma *hoja → camino Merkle → raíz → cabeza firmada* necesita
//! **el mismo hash que el nodo**, y hasta ahora ese hash **solo existía
//! dentro de `stark-experiment`**, que arrastra `winterfell` entero,
//! `sled` y `settlement-prover`.
//!
//! > **La primitiva de verificación independiente no estaba donde tenía que
//! > estar**, y no se sabía porque **hasta ahora nadie fuera del nodo había
//! > necesitado recomputar una raíz**.
//!
//! Las dos salidas obvias eran malas:
//!
//! | salida | por qué no |
//! |---|---|
//! | reimplementar `native_merge` en el verificador | **dos implementaciones del mismo hash**, que divergirían **en silencio** — un recibo válido declarado inválido, o peor, al revés. Es lo que §253 evitó reusando `GuardianIndice` **entero** |
//! | que el verificador dependa de `stark-experiment` | **mata la propiedad de §243**, que tiene compuerta |
//!
//! ## Qué necesita de verdad, y qué no
//!
//! `native_merge` usa **tres cosas**, y **ninguna es del AIR**:
//! `Rp64_256` (de `winter-crypto`), `BaseElement` (de `winter-math`) y
//! `STATE_WIDTH`, que es **una constante del propio hasher** —
//! `Rp64_256::STATE_WIDTH`—, no del circuito.
//!
//! ⚠️ **No usa `apply_sbox`, ni `NUM_ROUNDS`, ni `MerkleTree`, ni
//! `ColMatrix`.** El AIR del circuito de hash **se queda entero donde
//! está**: aquí no se mueve nada de la maquinaria de pruebas.
//!
//! ## ⚠️ Las versiones van fijadas con `=`
//!
//! `winter-math` y `winter-crypto` se toman **sueltos y clavados a
//! `=0.13.1`**, que es lo que el `Cargo.lock` ya tenía resuelto vía
//! `winterfell 0.13`.
//!
//! Si se tomaran por rango, cargo podría resolver **dos versiones del mismo
//! subcrate** en el árbol — y **dos `BaseElement` de versiones distintas no
//! son el mismo tipo**. El buen caso sería que no compilara; el malo, que
//! compilara con conversiones y **divergiera en silencio**.
//!
//! ## Este sello no cambia ni un byte
//!
//! Es un **refactor puro**: `stark-experiment` reexporta `native_merge`
//! desde aquí, y **los 172 usos en 31 ficheros siguen igual**.
//!
//! ⚠️ Y la corrección **no la demuestra un argumento, la demuestran las
//! compuertas que ya existen**: 297 tests de `stark-experiment`, 256 de la
//! capa, los seis censos y **la conformidad `zkssl/0.2`, que pincha el
//! `epoch_digest`**.
//!
//! Como `native_merge` es **la primitiva del árbol y de `chain_digest`**,
//! un corte mal hecho **revienta la conformidad inmediatamente**. No hay
//! forma de que pase inadvertido.

use winter_crypto::hashers::Rp64_256;
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;

/// Cuatro elementos de campo: el resumen que circula por todo el proyecto.
pub type Digest = [BaseElement; 4];

/// Anchura del estado de la permutación, **tomada del propio hasher**.
///
/// ⚠️ No es una constante del circuito: es `Rp64_256::STATE_WIDTH`. Copiarla
/// a mano la desligaría de la implementación que de verdad se usa.
pub const STATE_WIDTH: usize = Rp64_256::STATE_WIDTH;

/// Hash 2-a-1 nativo, con la implementación **real** de `winter-crypto`.
///
/// ⚠️ Es **la primitiva del árbol disperso y de `chain_digest`**: si esto
/// cambiara, cambiarían todas las raíces del proyecto y la conformidad
/// `zkssl/0.2` lo diría en el acto.
pub fn native_merge(left: Digest, right: Digest) -> Digest {
    let mut state = [BaseElement::ZERO; STATE_WIDTH];
    state[4..8].copy_from_slice(&left);
    state[8..12].copy_from_slice(&right);
    Rp64_256::apply_permutation(&mut state);
    [state[4], state[5], state[6], state[7]]
}

/// **Embebe un elemento como digest**: el valor en el primer hueco, ceros
/// el resto.
///
/// ⚠️ **Esta es LA decisión de formato** —dónde va el elemento y con qué se
/// rellena—. [`as_digest`] es un **atajo tipado** que la llama: no son dos
/// definiciones con dos firmas, es **una y su conveniencia**.
///
/// ⚠️ Siete copias privadas de esta misma función viven en
/// `stark-experiment` (`native.rs`, `nullifier.rs`, `circuit_governance.rs`,
/// `circuit_threshold.rs`, `compliance_circuit.rs`, `double_entry.rs`,
/// `circuit_mint_pending.rs`). **§258 no las toca**: no cruzan al
/// verificador, igual que en §255. Pero ahora existe **una pública contra
/// la que compararlas** el día que se aborden.
pub fn embeber(x: BaseElement) -> Digest {
    [x, BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
}

/// Embebe un `u64` como digest. **Atajo tipado** sobre [`embeber`].
///
/// ⚠️ Era **privada** en `zk-ssl/src/log.rs`. Es una **decisión de
/// formato** —dónde va el número y con qué se rellena—, así que un
/// verificador independiente tiene que usar **esta misma**, no una copia.
pub fn as_digest(x: u64) -> Digest {
    embeber(BaseElement::new(x))
}

/// **Hoja de cuenta**: `merge(merge(pk, saldo), nonce)`.
///
/// ⚠️ **Sin esto, §256 seguía siendo un recibo ilegible para su
/// destinatario.** `verificar_inclusion` prueba que **una** hoja estaba en
/// la cabeza firmada; para saber que es **la tuya** hay que recomponerla, y
/// eso vivía en `stark-experiment` —el probador—. Un titular con solo
/// `zk-ssl-verify` no podía.
///
/// ⚠️ **No cambia un byte**: `stark-experiment` la reexporta desde aquí, así
/// que las dos son literalmente la misma función. Y la **conformidad 0.2**
/// es el juez: si compusiera distinto, el `epoch_digest` se movería.
pub fn native_leaf(public_id: Digest, balance: BaseElement, nonce: BaseElement) -> Digest {
    let inner = native_merge(public_id, embeber(balance));
    native_merge(inner, embeber(nonce))
}

/// **Hoja salteada** (entrada 50, §117): la vieja **con un merge más** de
/// salt al final.
///
/// ⚠️ **Las dos formas conviven**, y cuál aplica es propiedad **del ledger
/// entero**, no de la cuenta: al reabrir, `meta:migrated` en sled —o la
/// versión del snapshot— decide si las hojas se reconstruyen saladas o no.
/// En caliente, toda cuenta nueva sale **salada**.
///
/// ⚠️ Y **no son intercambiables**: `native_leaf` NO es
/// `native_leaf_salted` con salt cero — hay test. Por eso el recibo de §259
/// tendrá que **declarar cuál se usó**: quien componga la que no es verá
/// `RaizDistinta` **sin saber por qué**.
pub fn native_leaf_salted(
    public_id: Digest,
    balance: BaseElement,
    nonce: BaseElement,
    leaf_salt: Digest,
) -> Digest {
    native_merge(native_leaf(public_id, balance, nonce), leaf_salt)
}

/// Sube un camino Merkle desde la hoja y devuelve la raíz.
///
/// ⚠️ **Itera sobre la LONGITUD DEL CAMINO, no sobre una constante.** La
/// versión de `stark-experiment` usaba `TREE_DEPTH` fijo, mientras
/// `SparseTree::path_for` genera caminos de `self.depth`: con un árbol de
/// otra profundidad, la constante habría leído fuera del camino o dejado
/// niveles sin subir.
///
/// `is_right[i] == true` significa que **el nodo actual va a la derecha** y
/// el hermano a la izquierda.
pub fn path_root(leaf: Digest, siblings: &[Digest], is_right: &[bool]) -> Digest {
    debug_assert_eq!(siblings.len(), is_right.len(), "camino descuadrado");
    let mut current = leaf;
    for (hermano, derecha) in siblings.iter().zip(is_right) {
        current = if *derecha {
            native_merge(*hermano, current)
        } else {
            native_merge(current, *hermano)
        };
    }
    current
}

/// Compone el digest de una cabeza de época.
///
/// ⚠️ **Esta es LA composición**, y `EpochHead::digest()` la llama. Un
/// verificador que quiera comprobar que una raíz de cuentas pertenece a una
/// cabeza firmada **necesita componerla exactamente igual** — y la única
/// forma segura de garantizarlo es que **sea la misma función**.
pub fn epoch_digest(
    seq: u64,
    accounts_root: Digest,
    pending_root: Digest,
    frozen_root: Digest,
    chain_digest: Digest,
) -> Digest {
    let a = native_merge(as_digest(seq), accounts_root);
    let b = native_merge(pending_root, frozen_root);
    native_merge(native_merge(a, b), chain_digest)
}

/// **Dominio del acuse**, con version en el propio valor.
///
/// Son los ocho bytes ASCII de `ACUSE_V1` leidos como `u64`. Se escribe asi
/// —y no como numero magico— para que se lea lo que es, y **con sufijo**:
/// de los cinco dominios del arbol, tres llevan version y uno no, y el
/// sexto no entra siendo el segundo sin ella por inercia. Si algun dia hay
/// un acuse v2, el sufijo es lo que permite que convivan.
///
/// ⚠️ **No hay registro de dominios en el proyecto**: son literales sueltos
/// en cuatro crates. Construirlo es otro sello; esto solo evita empeorarlo.
pub const DOMINIO_ACUSE: u64 = u64::from_be_bytes(*b"ACUSE_V1");

/// **Acuse de recepcion**: ata una prueba a la epoca y al `N` declarado.
///
/// ⚠️ **Esta es LA composicion**, por la misma razon que `epoch_digest`: un
/// tercero que quiera comprobar un acuse **necesita componerlo exactamente
/// igual**, y la unica forma segura de garantizarlo es que **sea la misma
/// funcion**. Hasta §270 vivia como funcion privada dentro de un
/// `#[cfg(test)]` de la capa — es decir, **nadie de fuera podia llamarla**,
/// que es el defecto que §257 y §258 fueron a corregir.
///
/// ⚠️ **Y lleva tag de dominio mientras que `epoch_digest`, ahi arriba, no.
/// La asimetria es deliberada.** `epoch_digest` no lo necesita porque
/// **siempre se consume dentro de un preambulo firmado** —
/// `b"ZK-SSL-epoch-head"`—, que es donde vive su separacion. El acuse
/// **hoy no tiene preambulo**, y el tag hace ese papel: sin el, un acuse y
/// un nodo interno de cualquier arbol son el mismo valor compuesto de la
/// misma forma. **Quien las vea juntas y quiera armonizarlas, que lea esto
/// antes de quitar la etiqueta.**
///
/// La forma del tag no se inventa: es la de `stark-experiment/src/
/// native.rs`, que ya separa `SPEND_KEY_DOMAIN` y `NULLIFIER_DOMAIN`
/// mezclando el dominio por delante.
pub fn acuse_digest(hash_prueba: Digest, epoca: u64, n: u64) -> Digest {
    let par = native_merge(as_digest(epoca), as_digest(n));
    native_merge(as_digest(DOMINIO_ACUSE), native_merge(hash_prueba, par))
}

// ═══════════════════════════════════════════════════════════════════════
//  §257 · EL BYTE Y EL ELEMENTO
//
//  ⚠️ **Sin esto, §256 no era alcanzable.** `verificar_inclusion` toma
//  `Digest` —cuatro elementos de campo—, y lo que viaja por el cable son
//  **32 bytes**. La conversión entre las dos cosas vivía en
//  `zk-ssl/src/store.rs`, **dentro de la capa**: un tercero que quisiera
//  comprobar un recibo tenía que **compilar el código del operador**, que
//  es exactamente lo que §243 existe para impedir.
//
//  Es una **decisión de formato** —little-endian, cuatro elementos en
//  orden—, y por el criterio de §254 le corresponde vivir aquí.
//  `store.rs` **delega**: una sola definición, no dos.
//
//  ⚠️ **Los mensajes de error se conservan letra a letra.** Delegar no
//  debe cambiar lo que ve quien ya dependía de esto, y hay test que lo
//  fija en los dos lados.
// ═══════════════════════════════════════════════════════════════════════

/// Lo que puede ir mal al leer un digest o un elemento del cable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatoError {
    /// Un elemento que no mide 8 bytes.
    LongitudElemento(usize),
    /// Un digest que no mide 32 bytes.
    LongitudDigest(usize),
}

impl core::fmt::Display for FormatoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // ⚠️ Estas dos frases son **las de `store.rs` antes de delegar**.
        // Cambiarlas rompería los mensajes que la capa ya emitía.
        match self {
            FormatoError::LongitudElemento(n) => write!(f, "elemento de {n} bytes"),
            FormatoError::LongitudDigest(n) => {
                write!(f, "digest de {n} bytes, se esperaban 32")
            }
        }
    }
}

impl std::error::Error for FormatoError {}

/// Un elemento de Goldilocks cabe en 8 bytes, little-endian.
pub fn element_to_bytes(e: BaseElement) -> [u8; 8] {
    e.as_int().to_le_bytes()
}

pub fn element_from_bytes(b: &[u8]) -> Result<BaseElement, FormatoError> {
    let arr: [u8; 8] = b
        .try_into()
        .map_err(|_| FormatoError::LongitudElemento(b.len()))?;
    Ok(BaseElement::new(u64::from_le_bytes(arr)))
}

/// Cuatro elementos en orden, 8 bytes cada uno.
///
/// ⚠️ **El orden es parte del formato**, no un detalle: si se invirtiera,
/// el nodo y el verificador compondrían raíces distintas de los mismos
/// bytes y **ninguna compuerta lo vería**. Hay test que lo fija.
pub fn digest_to_bytes(d: &Digest) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, e) in d.iter().enumerate() {
        out[i * 8..(i + 1) * 8].copy_from_slice(&element_to_bytes(*e));
    }
    out
}

pub fn digest_from_bytes(b: &[u8]) -> Result<Digest, FormatoError> {
    if b.len() != 32 {
        return Err(FormatoError::LongitudDigest(b.len()));
    }
    let mut d = [BaseElement::ZERO; 4];
    for (i, hueco) in d.iter_mut().enumerate() {
        *hueco = element_from_bytes(&b[i * 8..(i + 1) * 8])?;
    }
    Ok(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: u64) -> Digest {
        [
            BaseElement::new(n),
            BaseElement::new(n + 1),
            BaseElement::new(n + 2),
            BaseElement::new(n + 3),
        ]
    }

    // ── §257 · el byte y el elemento ──────────────────────────────

    #[test]
    fn el_digest_da_la_vuelta_por_los_bytes() {
        let x = d(7);
        assert_eq!(digest_from_bytes(&digest_to_bytes(&x)).unwrap(), x);
    }

    #[test]
    fn el_orden_es_little_endian_y_por_elementos() {
        // ⚠️ ESTO FIJA EL FORMATO. Si alguien invirtiera el orden de los
        // cuatro elementos o el de los bytes, el nodo y el verificador
        // compondrian raices distintas de los MISMOS bytes y nada lo
        // delataria. Aqui si.
        let b = digest_to_bytes(&as_digest(1));
        assert_eq!(b[0], 1, "el valor va en el primer byte del primer elemento");
        assert!(b[1..32].iter().all(|x| *x == 0), "el resto son ceros");
    }

    #[test]
    fn una_longitud_que_no_es_32_se_rechaza_y_dice_cuanto() {
        // ⚠️ Un instrumento que falla dice QUE fallo (§254): no basta con
        // rechazar, tiene que decir cuantos bytes llegaron.
        let e = digest_from_bytes(&[0u8; 31]).unwrap_err();
        assert_eq!(e, FormatoError::LongitudDigest(31));
        assert_eq!(format!("{e}"), "digest de 31 bytes, se esperaban 32");
        let e2 = element_from_bytes(&[0u8; 7]).unwrap_err();
        assert_eq!(format!("{e2}"), "elemento de 7 bytes");
    }

    #[test]
    fn as_digest_pone_el_valor_delante_y_ceros_detras() {
        let v = as_digest(7);
        assert_eq!(v[0], BaseElement::new(7));
        assert!(v[1..].iter().all(|x| *x == BaseElement::ZERO));
    }

    #[test]
    fn un_camino_de_un_nivel_sube_en_los_dos_sentidos() {
        // ⚠️ Si el orden no importara, un camino no distinguiria izquierda
        // de derecha y CUALQUIER hoja probaria CUALQUIER posicion.
        let (hoja, hermano) = (d(1), d(9));
        assert_eq!(path_root(hoja, &[hermano], &[false]), native_merge(hoja, hermano));
        assert_eq!(path_root(hoja, &[hermano], &[true]), native_merge(hermano, hoja));
    }

    #[test]
    fn path_root_itera_sobre_el_camino_no_sobre_una_constante() {
        // ⚠️ La version de stark-experiment usaba TREE_DEPTH FIJO. Con un
        // camino mas corto habria leido fuera; con uno mas largo, habria
        // dejado niveles sin subir.
        let hoja = d(1);
        assert_eq!(path_root(hoja, &[], &[]), hoja, "camino vacio: la hoja ES la raiz");
        let dos = path_root(hoja, &[d(9), d(11)], &[false, true]);
        assert_eq!(dos, native_merge(d(11), native_merge(hoja, d(9))));
        let tres = path_root(hoja, &[d(9), d(11), d(13)], &[false, true, false]);
        assert_ne!(dos, tres, "cada nivel cuenta");
    }

    #[test]
    fn el_digest_de_epoca_depende_de_sus_cinco_partes() {
        // ⚠️ Si alguna no entrara, el operador podria cambiarla sin que la
        // cabeza firmada lo delatara.
        let base = epoch_digest(1, d(10), d(20), d(30), d(40));
        assert_ne!(base, epoch_digest(2, d(10), d(20), d(30), d(40)), "seq");
        assert_ne!(base, epoch_digest(1, d(11), d(20), d(30), d(40)), "accounts_root");
        assert_ne!(base, epoch_digest(1, d(10), d(21), d(30), d(40)), "pending_root");
        assert_ne!(base, epoch_digest(1, d(10), d(20), d(31), d(40)), "frozen_root");
        assert_ne!(base, epoch_digest(1, d(10), d(20), d(30), d(41)), "chain_digest");
    }

    #[test]
    fn el_hash_es_determinista() {
        assert_eq!(native_merge(d(1), d(5)), native_merge(d(1), d(5)));
    }

    #[test]
    fn el_orden_de_los_hijos_importa() {
        // ⚠️ Si no importara, un camino Merkle no distinguiria izquierda de
        // derecha y CUALQUIER hoja probaria CUALQUIER posicion.
        assert_ne!(native_merge(d(1), d(5)), native_merge(d(5), d(1)));
    }

    #[test]
    fn entradas_distintas_dan_salidas_distintas() {
        let a = native_merge(d(0), d(0));
        assert_ne!(a, native_merge(d(0), d(1)));
        assert_ne!(a, native_merge(d(1), d(0)));
    }

    #[test]
    fn la_anchura_del_estado_es_la_del_hasher() {
        // ⚠️ Copiar 12 a mano desligaria la constante de la implementacion.
        assert_eq!(STATE_WIDTH, Rp64_256::STATE_WIDTH);
        assert!(STATE_WIDTH >= 12, "native_merge escribe hasta el indice 11");
    }

    // ── §258 · la hoja, componible sin el probador ────────────────

    #[test]
    fn embeber_y_as_digest_componen_lo_mismo() {
        // ⚠️ `as_digest` es un ATAJO, no una segunda definicion. Si algun
        // dia dejara de delegar, esto lo dice.
        for v in [0u64, 1, 7, u64::MAX >> 1] {
            assert_eq!(as_digest(v), embeber(BaseElement::new(v)));
        }
        let e = embeber(BaseElement::new(9));
        assert_eq!(e[0], BaseElement::new(9), "el valor va delante");
        assert!(e[1..].iter().all(|x| *x == BaseElement::ZERO), "y ceros detras");
    }

    #[test]
    fn la_hoja_salada_es_la_vieja_con_un_merge_mas() {
        // ⚠️ FIJA LA ESTRUCTURA. El circuito paga un bloque Rescue de mas
        // por hoja (entrada 15, §82); si la composicion cambiara sin que
        // nadie lo notase, el coste medido dejaria de corresponder.
        let id = d(10);
        let (saldo, nonce, sal) = (BaseElement::new(1_000), BaseElement::new(3), d(20));
        assert_eq!(
            native_leaf_salted(id, saldo, nonce, sal),
            native_merge(native_leaf(id, saldo, nonce), sal)
        );
    }

    #[test]
    fn una_hoja_sin_sal_no_es_la_salada_con_sal_cero() {
        // ⚠️⚠️ LAS DOS FORMAS SON DISTINTAS DE VERDAD, y de aqui sale una
        // consecuencia para §259: el recibo TIENE QUE DECLARAR cual se
        // uso. Quien componga la que no es vera `RaizDistinta` sin saber
        // por que — y un fallo ilegible es el que hace que se culpe al
        // sitio equivocado.
        //
        // Si esto fuera igualdad, el campo `leafFormat` de §259 seria
        // decoracion. No lo es.
        let id = d(10);
        let (saldo, nonce) = (BaseElement::new(1_000), BaseElement::new(3));
        let cero: Digest = [BaseElement::ZERO; 4];
        assert_ne!(
            native_leaf(id, saldo, nonce),
            native_leaf_salted(id, saldo, nonce, cero)
        );
    }
}

#[cfg(test)]
mod acuse {
    //! §270 · Los contratos del acuse.
    use super::*;

    #[test]
    fn el_tag_separa_el_acuse_del_merge_pelado() {
        // Sin tag, el acuse seria indistinguible de cualquier nodo interno
        // compuesto con los mismos operandos. Esto es lo UNICO que el
        // dominio existe para garantizar, y por eso se comprueba.
        let hp = as_digest(0xA11CE);
        let pelado = native_merge(hp, native_merge(as_digest(100), as_digest(1_440)));
        assert_ne!(
            acuse_digest(hp, 100, 1_440),
            pelado,
            "el acuse coincide con el merge pelado: el tag no esta haciendo nada"
        );
    }

    #[test]
    fn el_valor_del_acuse_esta_pinchado() {
        // ⚠️ Este literal NO dice que el traslado desde la capa no movio el
        // valor: LO MOVIO A PROPOSITO, porque §270 anadio el tag de
        // dominio. Lo que fija es que **a partir de aqui no cambia**.
        //
        // Se pudo poner hoy porque el acuse NO viaja: no esta en ningun
        // vector de conformidad, ni en el log, ni en un snapshot — solo
        // dentro de tests. Cuando empiece a viajar, tocarlo sera un cambio
        // de formato con version, y este literal sera lo que lo delate.
        let hp = as_digest(0xA11CE);
        assert_eq!(
            digest_to_bytes(&acuse_digest(hp, 100, 1_440)),
            [0xfb, 0x59, 0x25, 0x85, 0x8d, 0xac, 0xfd, 0x0d, 0xbb, 0x8f, 0xae, 0x00, 0xb9, 0xb3, 0x5a, 0xf3, 0x49, 0x21, 0x94, 0x5b, 0x38, 0x2f, 0xea, 0x3c, 0x72, 0x4b, 0x3b, 0x42, 0x9a, 0xa7, 0x32, 0xc1],
            "el valor del acuse se ha movido: si es a proposito, sube la version del dominio"
        );
    }
}
