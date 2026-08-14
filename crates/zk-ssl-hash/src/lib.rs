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

/// Compone el digest de una cabeza de epoca **v2** (§275): la composicion
/// v1 **en su posicion exacta**, mas la pareja de acuses.
///
///   `v2 = merge( epoch_digest(los cinco), merge(acuses_root, as_digest(n)) )`
///
/// ⚠️ La composicion de cinco NO se duplica: esta funcion **llama** a
/// [`epoch_digest`]. Dos recomponedores que compartieran solo la forma
/// divergirian en silencio — la razon de §255, otra vez.
///
/// ⚠️ **El desambiguador v1/v2 es el BYTE DE VERSION del preambulo, no el
/// dominio** (§236): los merges van a pelo, como en `epoch_digest`, y el
/// dominio no lleva version a proposito.
pub fn epoch_digest_v2(
    seq: u64,
    accounts_root: Digest,
    pending_root: Digest,
    frozen_root: Digest,
    chain_digest: Digest,
    acuses_root: Digest,
    n: u64,
) -> Digest {
    native_merge(
        epoch_digest(seq, accounts_root, pending_root, frozen_root, chain_digest),
        native_merge(acuses_root, as_digest(n)),
    )
}

/// **v3 (§292): la envoltura de v2, otra vez** — el molde de §275:
///
/// ```text
///   v3 = merge( epoch_digest_v2(los siete), merge(cima_mmr, as_digest(t)) )
/// ```
///
/// `cima_mmr` es la cima del MMR de cabezas (§291) sobre los digests de
/// TODAS las cabezas anteriores, y `t` cuantas hojas acumula. Con esto
/// **una cabeza nueva prueba que EXTIENDE a la vieja** (eslabon 2 de la
/// nota 83): la prueba de consistencia sube de cima a cima.
///
/// ⚠️ **Genesis, DECLARADO**: la PRIMERA cabeza compone con
/// `cima_mmr = as_digest(0)` y `t = 0` — un acumulador vacio no tiene
/// cima y este es el valor que la composicion fija para ese caso.
///
/// ⚠️ Sin tag de dominio, por la razon de `epoch_digest`: siempre se
/// consume dentro de un preambulo firmado, y el **byte de version** del
/// preambulo (2 → 3) es lo que separa las composiciones (§236).
pub fn epoch_digest_v3(
    seq: u64,
    accounts_root: Digest,
    pending_root: Digest,
    frozen_root: Digest,
    chain_digest: Digest,
    acuses_root: Digest,
    n: u64,
    cima_mmr: Digest,
    t: u64,
) -> Digest {
    native_merge(
        epoch_digest_v2(seq, accounts_root, pending_root, frozen_root, chain_digest, acuses_root, n),
        native_merge(cima_mmr, as_digest(t)),
    )
}

#[cfg(test)]
mod tests_digest_v3 {
    use super::*;

    #[test]
    fn v3_no_es_v2_ni_con_la_pareja_de_genesis() {
        // La envoltura SIEMPRE separa: hasta el genesis (cima=0, t=0)
        // compone distinto de v2 — si no, una cabeza v3 recien nacida
        // seria confundible con una v2.
        let d = as_digest(7);
        let v2 = epoch_digest_v2(1, d, d, d, d, d, 5);
        let v3 = epoch_digest_v3(1, d, d, d, d, d, 5, as_digest(0), 0);
        assert_ne!(v2, v3);
    }

    #[test]
    fn la_cima_y_t_mueven_el_digest_cada_uno_por_su_lado() {
        let d = as_digest(7);
        let base = epoch_digest_v3(1, d, d, d, d, d, 5, as_digest(0), 0);
        assert_ne!(base, epoch_digest_v3(1, d, d, d, d, d, 5, as_digest(9), 0));
        assert_ne!(base, epoch_digest_v3(1, d, d, d, d, d, 5, as_digest(0), 1));
    }
}

/// **Dominio del acuse**, con version en el propio valor.
///
/// Son los ocho bytes ASCII de `ACUSE_V1` leidos como `u64`. Se escribe asi
/// —y no como numero magico— para que se lea lo que es, y **con sufijo**:
/// de los cinco dominios del arbol, tres llevan version y uno no, y el
/// sexto no entra siendo el segundo sin ella por inercia. Si algun dia hay
/// un acuse v2, el sufijo es lo que permite que convivan.
///
/// ⚠️ **El registro de dominios existe desde §286**: la tabla vive mas
/// abajo, junto a las dos familias, y `tools/check_dominios.py` la compara
/// con el CENSO del arbol en cada sello — un literal suelto, una deriva de
/// valor o una linea de tabla que miente es rojo que dice que editar.
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


/// **Dominios del MMR de cabezas** (§291, eslabon 2 de la nota 83), con
/// version en el propio valor, como `ACUSE_V1`: ocho bytes ASCII leidos
/// como `u64`, sufijo incluido para que un futuro v2 conviva sin pisar.
///
/// Dos y no uno porque la separacion hoja/interior es la defensa
/// clasica de segunda preimagen del arbol de historia (RFC 6962): sin
/// ella, un nodo interno presentado como hoja compone el mismo valor.
pub const DOMINIO_MMR_HOJA: u64 = u64::from_be_bytes(*b"MMRHOJA1");
/// El dominio de los nodos interiores del MMR. Ver [`DOMINIO_MMR_HOJA`].
pub const DOMINIO_MMR_NODO: u64 = u64::from_be_bytes(*b"MMRNODO1");

/// **La hoja del MMR de cabezas**: un digest de cabeza, etiquetado.
///
/// ⚠️ **Esta es LA composicion**, por la misma razon que
/// [`acuse_digest`]: el cliente que verifica una extension tiene que
/// componer exactamente igual que el nodo que la sirve, y la unica
/// forma segura es que sea la misma funcion. La forma del tag es la de
/// siempre: el dominio mezclado por delante.
pub fn mmr_hoja(cabeza: Digest) -> Digest {
    native_merge(as_digest(DOMINIO_MMR_HOJA), cabeza)
}

/// **El nodo interior del MMR de cabezas**. Ver [`mmr_hoja`]: mismo
/// contrato, dominio propio.
pub fn mmr_nodo(izquierda: Digest, derecha: Digest) -> Digest {
    native_merge(as_digest(DOMINIO_MMR_NODO), native_merge(izquierda, derecha))
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

#[cfg(test)]
mod tests_cabeza_v2 {
    use super::*;

    fn cinco() -> (u64, Digest, Digest, Digest, Digest) {
        (7, as_digest(11), as_digest(22), as_digest(33), as_digest(44))
    }

    #[test]
    fn v2_es_merge_de_v1_y_la_pareja_literal() {
        // La VARIANTE de §275, comprobada letra a letra: el recomponedor
        // v2 reusa la composicion v1 como subarbol izquierdo.
        let (s, a, p, f, c) = cinco();
        let (r, n) = (as_digest(55), 1_440);
        assert_eq!(
            epoch_digest_v2(s, a, p, f, c, r, n),
            native_merge(epoch_digest(s, a, p, f, c), native_merge(r, as_digest(n))),
        );
    }

    #[test]
    fn v2_no_es_v1_con_los_mismos_cinco() {
        // Si coincidieran, una cabeza v2 verificaria como v1 y el byte de
        // version del preambulo no separaria nada.
        let (s, a, p, f, c) = cinco();
        assert_ne!(
            epoch_digest_v2(s, a, p, f, c, as_digest(0), 0),
            epoch_digest(s, a, p, f, c),
        );
    }

    #[test]
    fn mover_la_raiz_de_acuses_mueve_v2() {
        let (s, a, p, f, c) = cinco();
        assert_ne!(
            epoch_digest_v2(s, a, p, f, c, as_digest(1), 9),
            epoch_digest_v2(s, a, p, f, c, as_digest(2), 9),
        );
    }

    #[test]
    fn mover_n_mueve_v2() {
        // n viaja firmado: prometer otro n produce OTRA cabeza.
        let (s, a, p, f, c) = cinco();
        assert_ne!(
            epoch_digest_v2(s, a, p, f, c, as_digest(1), 1_440),
            epoch_digest_v2(s, a, p, f, c, as_digest(1), 720),
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// §279 · LAS REGLAS DEL SELLO, DONDE LAS VE UN TERCERO
//
// ⚠️ Todo lo de aqui abajo **vivia en otro sitio** y se muda por la misma
// razon que `native_merge` (§254), `as_digest` (§255) y `native_leaf`
// (§258): el CONSTRUCTOR y el VERIFICADOR tienen que llamar LAS MISMAS
// reglas, y `zk-ssl-verify` no puede depender ni de la capa ni del
// probador. Los llamantes antiguos siguen funcionando por REEXPORTACION;
// no hay dos implementaciones de nada.
//
// ⚠️ **Dos familias de dominio conviven aqui, y es a proposito.** Los
// dominios algebraicos son `u64` (`DOMINIO_ACUSE`) porque entran en la
// permutacion Rescue como elemento de campo; los de abajo son `&[u8]`
// porque entran en Blake3 como bytes. **No se armonizan**: una conversion
// entre las dos cambiaria digests ya publicados.
// ═══════════════════════════════════════════════════════════════════

// ── EL REGISTRO DE DOMINIOS (§286) ─────────────────────────────────
// La deuda que DOMINIO_ACUSE dejo escrita arriba prometia este sello.
// La tabla es lo que `tools/check_dominios.py` compara contra el CENSO
// del arbol en cada --sello: una linea de menos, un literal suelto o una
// deriva de valor es ROJO que dice que editar. Los grupos separan
// ESPACIOS DE HASH: `produccion` (stark-experiment + zk-ssl-hash, la
// misma permutacion Rescue) exige valores unicos entre si; cada
// paradigma es su propio espacio, y reutilizar el mnemonico entre grupos
// es legitimo —"NULL" en cinco crates es el mismo proposito sobre
// hashes que no colisionan entre si—. La familia bytes entra entera en
// Blake3: valor unico GLOBAL y un solo sitio de declaracion por cadena.
// REGISTRO: u64 produccion SPEND_KEY_DOMAIN 0x53504B59
// REGISTRO: u64 produccion NULLIFIER_DOMAIN 0x4E554C4C
// REGISTRO: u64 produccion CUSTODIAN_DOMAIN 0x43555354
// REGISTRO: u64 produccion GOVERNANCE_DOMAIN 0x474F5645
// REGISTRO: u64 produccion LEAF_SALT_DOMAIN 0x53414C544C454146
// REGISTRO: u64 produccion VIEW_KEY_DOMAIN 0x564945574B455900
// REGISTRO: u64 produccion DOMINIO_ACUSE 0x41435553455F5631
// REGISTRO: u64 produccion OP_MINT 0x4D494E54
// REGISTRO: u64 produccion OP_MINT_PENDING 0x4D504E44
// REGISTRO: u64 produccion OP_FREEZE 0x46525A45
// REGISTRO: u64 produccion OP_RECOVERY 0x5245434F
// REGISTRO: u64 produccion OP_GOVERNANCE 0x474F5652
// REGISTRO: u64 produccion DOMINIO_MMR_HOJA 0x4D4D52484F4A4131
// REGISTRO: u64 produccion DOMINIO_MMR_NODO 0x4D4D524E4F444F31
// REGISTRO: u64 zk-core NULLIFIER_DOMAIN 0x4E554C4C
// REGISTRO: u64 zk-core SPEND_KEY_DOMAIN 0x53504B59
// REGISTRO: u64 zk-core ISSUER_DOMAIN 0x49535355
// REGISTRO: u64 halo2 NULLIFIER_DOMAIN 0x4E554C4C
// REGISTRO: u64 plonk LEAF_DOMAIN 0x4C454146
// REGISTRO: u64 plonk NULLIFIER_DOMAIN 0x4E554C4C
// REGISTRO: bytes ZK-SSL-ledger-key-v1
// REGISTRO: bytes ZK-SSL-epoch-head
// REGISTRO: bytes ZK-SSL-keystore-v1
// REGISTRO: bytes ZK-SSL-proof-digest-v2
// REGISTRO: bytes ZK-SSL-authorization-seal-v1
// REGISTRO: bytes ZK-SSL-no-proof-by-design-v1
// REGISTRO: bytes ZK-SSL-witness-cosign

/// Dominios de operacion. **Uno por tipo**, para que una autorizacion de
/// congelacion no pueda reutilizarse como autorizacion de emision.
pub const OP_MINT: u64 = 0x4D494E54; // "MINT"
pub const OP_MINT_PENDING: u64 = 0x4D504E44; // "MPND"
pub const OP_FREEZE: u64 = 0x46525A45; // "FRZE"
pub const OP_RECOVERY: u64 = 0x5245434F; // "RECO"
pub const OP_GOVERNANCE: u64 = 0x474F5652; // "GOVR"

/// Resume los parametros de una operacion en un `Digest` que los custodios
/// firman.
///
/// Esponja sobre la permutacion Rescue: capacidad `state[0..4]` con el dominio
/// en `state[0]`, ritmo `state[4..12]` de ocho elementos, modo sobrescritura.
///
/// ⚠️ **Supone longitud FIJA por dominio.** No lleva relleno, asi que dos
/// mensajes del mismo dominio con longitudes distintas podrian colisionar
/// (`[a]` y `[a, 0]` dan lo mismo). Cada operacion tiene un numero fijo de
/// parametros, asi que la suposicion se cumple hoy —y los dominios impiden
/// colisiones ENTRE operaciones—. **Si alguna operacion pasa a tener
/// parametros de longitud variable, esto necesita una regla de relleno antes
/// de usarse.** Queda escrito porque es la clase de suposicion que se olvida.
pub fn commit_operation(domain: u64, elements: &[BaseElement]) -> Digest {
    let zero = BaseElement::ZERO;
    let mut state = [zero; STATE_WIDTH];
    state[0] = BaseElement::new(domain);
    for chunk in elements.chunks(8) {
        for i in 0..8 {
            state[4 + i] = if i < chunk.len() { chunk[i] } else { zero };
        }
        Rp64_256::apply_permutation(&mut state);
    }
    [state[4], state[5], state[6], state[7]]
}

/// **Dominio del resumen de prueba.** Separa este uso de cualquier otro
/// hash del proyecto: dos entradas de dominios distintos no pueden
/// colisionar aunque compartan bytes.
const DOMINIO_PRUEBA: &[u8] = b"ZK-SSL-proof-digest-v2";

/// **Dominio del sello de autorizacion** (§278). Lo que una via delegada
/// puede atar no es una prueba sino el **compromiso que la autorizo**.
const DOMINIO_AUTORIZACION: &[u8] = b"ZK-SSL-authorization-seal-v1";

/// **Dominio de la ausencia declarada** (§278). Separa «no demostrable por
/// diseno» de «autorizada y no registrada».
const DOMINIO_SIN_PRUEBA: &[u8] = b"ZK-SSL-no-proof-by-design-v1";

/// Resumen de una prueba serializada, con dominio y **longitud codificada**
/// por delante (§116: dos pruebas que difieran en ceros finales no
/// colisionan).
pub fn digest_of_proof(proof: &[u8]) -> Digest {
    use winter_crypto::hashers::Blake3_256;
    use winter_crypto::{Digest as _, Hasher as _};

    let mut entrada = Vec::with_capacity(DOMINIO_PRUEBA.len() + 8 + proof.len());
    entrada.extend_from_slice(DOMINIO_PRUEBA);
    entrada.extend_from_slice(&(proof.len() as u64).to_le_bytes());
    entrada.extend_from_slice(proof);

    let bytes = Blake3_256::<BaseElement>::hash(&entrada).as_bytes();
    let mut salida: Digest = [BaseElement::ZERO; 4];
    for (i, hueco) in salida.iter_mut().enumerate() {
        let mut w = [0u8; 8];
        w.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
        *hueco = BaseElement::new(u64::from_le_bytes(w));
    }
    salida
}

/// **Sello de autorizacion** (§278): lo que una via delegada asienta en
/// lugar de una prueba — el compromiso contra el que se verifico.
pub fn sello_de_autorizacion(operation: &Digest) -> Vec<u8> {
    let mut v = Vec::with_capacity(DOMINIO_AUTORIZACION.len() + 32);
    v.extend_from_slice(DOMINIO_AUTORIZACION);
    v.extend_from_slice(&digest_to_bytes(operation));
    v
}

/// **Sello de ausencia declarada** (§278): lo que asienta una transicion
/// que no genera prueba **por diseno**.
pub fn sello_sin_prueba() -> Vec<u8> {
    DOMINIO_SIN_PRUEBA.to_vec()
}

#[cfg(test)]
mod tests_sello_movido {
    use super::*;

    /// **La composicion del sello es pura**: los dominios separan, la
    /// ausencia declarada es una CONSTANTE —cualquiera la recomputa sin
    /// parametros— y la autorizacion es determinista y ata SU compromiso.
    #[test]
    fn el_sello_separa_por_dominio_y_ata_su_compromiso() {
        let a = digest_of_proof(&sello_de_autorizacion(&as_digest(7)));
        let b = digest_of_proof(&sello_de_autorizacion(&as_digest(8)));
        let sin = digest_of_proof(&sello_sin_prueba());

        assert_eq!(sello_sin_prueba(), sello_sin_prueba(), "constante");
        assert_eq!(a, digest_of_proof(&sello_de_autorizacion(&as_digest(7))), "determinista");
        assert_ne!(a, b, "ata SU compromiso, no la clase");
        assert_ne!(a, sin, "declarar y omitir no coinciden");
        assert_ne!(sin, digest_of_proof(&[]), "la ausencia declarada no es el vacio");
    }

    /// **El aviso de longitud fija, ejercitado.** `commit_operation` no
    /// lleva relleno: la garantia que SI se sostiene es que dos longitudes
    /// distintas del mismo prefijo, dentro del mismo dominio, no colisionan
    /// mientras la longitud no cruce el borde del ritmo de ocho.
    #[test]
    fn commit_operation_distingue_longitudes_del_mismo_prefijo() {
        let uno = commit_operation(OP_MINT, &[BaseElement::new(5)]);
        let dos = commit_operation(OP_MINT, &[BaseElement::new(5), BaseElement::new(9)]);
        assert_ne!(uno, dos, "mismo prefijo, longitud distinta");

        let otro_dominio = commit_operation(OP_FREEZE, &[BaseElement::new(5)]);
        assert_ne!(uno, otro_dominio, "el dominio separa operaciones");
    }
}
/// **Centinela de compromiso ausente** (§281, mudado aqui en §282).
///
/// Lo asientan las clases que NO son delegadas —las que llevan resumen de
/// prueba real y `OpenAccount`—, para que el campo `compromiso` de una
/// entrada de era 2 nunca quede sin significado. **No es un hash**: es una
/// constante declarada, del mismo genero que `VIEW_ID_LEGACY`.
///
/// ⚠️ Vive aqui y no en la capa porque **el reverificador independiente
/// tiene que distinguir centinela de compromiso real**, y `zk-ssl-verify`
/// no puede depender de `zk-ssl`. Duplicar la constante seria duplicar una
/// regla: misma razon que `native_merge` (§254), `as_digest` (§255),
/// `native_leaf` (§258) y la composicion del sello (§279).
pub const COMPROMISO_AUSENTE: Digest = [BaseElement::new(0xC0_A9_2281); 4];
