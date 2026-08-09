//! **LA FRAGUA** — los helpers nativos del mundo vivo, mudados del museo
//! (`circuit_settlement`, §36/§158) en la operación M-1 del frente §175.
//! Ocho circuitos y toda la capa `zk-ssl` computan hojas, nulificadores,
//! identidades, salts y claves de vista con estas funciones: el museo se
//! va, la fragua se queda — aquí.

use winterfell::math::{fields::f64::BaseElement, FieldElement};

use crate::merkle::{native_merge, Digest, MerklePath, TREE_DEPTH};
use crate::nullifier::NULLIFIER_DOMAIN;

/// Un elemento como digest: el resto del ancho, a cero. El mismo
/// utilitario privado que cada circuito replica (p. ej. governance).
fn as_digest(x: BaseElement) -> Digest {
    [x, BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
}

/// Dominio de derivación de la identidad desde la clave de gasto.
pub const SPEND_KEY_DOMAIN: u64 = 0x53504B59; // "SPKY"

/// Identidad de cuenta desde la clave de gasto. **Digest completo.**
pub fn derive_public_id(spend_key: BaseElement) -> Digest {
    native_merge(
        as_digest(BaseElement::new(SPEND_KEY_DOMAIN)),
        as_digest(spend_key),
    )
}

/// **Identidad desde una clave de CUATRO elementos** (entrada 15, §82).
///
/// La estrecha toma un solo elemento de Goldilocks: **2^64**, y `pk` es
/// publica, asi que agotar el espacio cuesta 2^63 —2,38 millones de
/// años-nucleo medidos en §82.3, cota floja—.
///
/// ⚠️ **Es una generalizacion, no un reemplazo**: rellenando con ceros
/// devuelve **exactamente lo mismo** que la estrecha, y hay test que lo fija
/// (`the_wide_derivation_generalises_the_narrow_one`). De ahi que migrar
/// **no invalide cuentas**.
///
/// ⚠️ **Pero conservar la identidad no conserva la seguridad.** Una clave
/// rellenada con ceros sigue teniendo 64 bits de entropia. Lo que la version
/// ancha permite es **generar claves de 256 bits**; las viejas hay que
/// rotarlas, y hasta entonces valen lo que valian.
pub fn derive_public_id_wide(spend_key: Digest) -> Digest {
    native_merge(as_digest(BaseElement::new(SPEND_KEY_DOMAIN)), spend_key)
}

// ⚠️ §258: **LA HOJA SE DEFINE EN `zk-ssl-hash`**, y aqui se REEXPORTA.
//    No es una delegacion: es LA MISMA FUNCION, asi que no pueden divergir
//    ni en silencio ni de ninguna otra forma. La ruta
//    `stark_experiment::native::native_leaf` no se mueve — hay ocho
//    circuitos y toda la capa llamandola.
//
//    Razon: `verificar_inclusion` (§256) prueba que UNA hoja estaba en la
//    cabeza firmada; para saber que es LA TUYA hay que recomponerla, y
//    componerla exigia compilar el PROBADOR. Un recibo que su destinatario
//    no puede interpretar no es un recibo.
//
// ⚠️ El `as_digest` privado de este fichero NO se toca: sigue siendo una de
//    las siete copias anotadas (§255), porque no cruza al verificador. Lo
//    que §258 anade es una publica —`zk_ssl_hash::embeber`— contra la que
//    compararlas el dia que se aborden.
pub use zk_ssl_hash::{native_leaf, native_leaf_salted};

/// Nullifier desde la CLAVE, no desde la identidad pública.
pub fn native_nullifier(spend_key: BaseElement, nonce: BaseElement) -> Digest {
    let inner = native_merge(
        as_digest(BaseElement::new(NULLIFIER_DOMAIN)),
        as_digest(spend_key),
    );
    native_merge(inner, as_digest(nonce))
}

/// Nullifier desde una clave de cuatro elementos.
///
/// Misma estructura que el estrecho —dominio, clave, nonce— con la clave
/// ocupando el digest entero en vez de su primer elemento.
pub fn native_nullifier_wide(spend_key: Digest, nonce: BaseElement) -> Digest {
    let inner = native_merge(as_digest(BaseElement::new(NULLIFIER_DOMAIN)), spend_key);
    native_merge(inner, as_digest(nonce))
}

pub fn native_climb(leaf: Digest, path: &MerklePath) -> Digest {
    let mut current = leaf;
    for level in 0..TREE_DEPTH {
        current = if path.is_right[level] {
            native_merge(path.siblings[level], current)
        } else {
            native_merge(current, path.siblings[level])
        };
    }
    current
}

/// Dominio del salt de hoja (entrada 50; cierra §108.4).
///
/// Valor autodescriptivo —"SALTLEAF" en ASCII— y **distinto de todo dominio
/// existente: `t2a_dominio` lo comprueba, no lo promete**. Si colisionara
/// con `NULLIFIER_DOMAIN`, el salt seria el estado interno del nullifier.
pub const LEAF_SALT_DOMAIN: u64 = 0x53414C54_4C454146;

/// Anchura estrecha: rellena y hereda la garantia de §90 —
/// `[sk,0,0,0]` es la MISMA cuenta, luego el MISMO salt.
pub fn derive_leaf_salt(spend_key: BaseElement) -> Digest {
    derive_leaf_salt_wide(as_digest(spend_key))
}

/// **Salt de hoja, derivado de la clave de gasto ANCHA.**
///
/// Decision de la entrada 50: el salt no es un secreto nuevo — se deriva de
/// la clave, en cliente, con la familia de hash del proyecto. Quien tiene la
/// clave lo re-deriva; quien la pierde ya lo habia perdido todo (§93.4: el
/// cliente no custodia estado, y esto no se lo pide).
///
/// ⚠️ Declarado: (1) acopla el salt a la clave — rotar clave implicara
/// nueva hoja; (2) es **convencion del cliente de referencia**, el protocolo
/// no impone el origen; (3) protege de terceros que ven caminos y pruebas —
/// **del operador no, y no lo pretende** (el operador ve los saldos).
pub fn derive_leaf_salt_wide(spend_key: Digest) -> Digest {
    native_merge(as_digest(BaseElement::new(LEAF_SALT_DOMAIN)), spend_key)
}

// ⚠️ §258: `native_leaf_salted` vive en `zk-ssl-hash` y se reexporta arriba,
//    junto a `native_leaf`. El despliegue en los circuitos y sus AIR
//    (B13/B14) sigue pendiente y sin cambios: mover la definicion no
//    despliega nada.

/// Dominio de la CLAVE DE VISTA (entrada 49). Distinto de todo dominio
/// vivo — `t7_vista` lo comprueba: si coincidiera con SPEND_KEY la clave
/// de vista SERIA la identidad y no cegaria nada; si con LEAF_SALT,
/// presentar la vista revelaria el salt de hoja.
pub const VIEW_KEY_DOMAIN: u64 = 0x56494557_4B455900; // "VIEWKEY\0"

/// **Clave de vista**: credencial de LECTURA derivada de la clave de
/// gasto (entrada 49; patron de §117 aplicado a lectura). El titular la
/// presenta; la capa la compara contra el `view_id` guardado al abrir la
/// cuenta. Barata (un merge, verificable NATIVAMENTE — no el STARK de
/// ~600 ms que la 49 declara inaceptable), y NO viaja en cada operacion
/// —a diferencia del salt que §109 descarto por eso—.
///
/// ⚠️ Limitacion declarada: acoplada a la clave, solo rota rotando la
/// clave (como el salt de §117). Una credencial de lectura ROTABLE de
/// verdad exigiria un secreto nuevo custodiado (§93.4 lo prohibe) — se
/// elige el acoplamiento sobre el secreto nuevo, conscientemente.
pub fn derive_view_key(spend_key: BaseElement) -> Digest {
    native_merge(
        as_digest(BaseElement::new(VIEW_KEY_DOMAIN)),
        as_digest(spend_key),
    )
}

/// El `view_id` que la cuenta guarda: hash de la clave de vista. Guardar
/// el hash y no la clave permite verificar por presentacion sin que el
/// operador quede con material que le deje LEER (solo COMPARAR).
/// Variante ANCHA de la clave de vista (49-A paso 2). Hereda §90:
/// `[sk,0,0,0]` y `sk` dan el MISMO view_id porque `derive_public_id`
/// ya lo garantiza y esta se define sobre la misma anchura.
pub fn derive_view_key_wide(spend_key: Digest) -> Digest {
    native_merge(as_digest(BaseElement::new(VIEW_KEY_DOMAIN)), spend_key)
}

/// **view_id a partir de una CLAVE DE VISTA ya derivada** (49-A paso 4).
/// El titular presenta `derive_view_key(sk)` —no su clave de gasto— y la
/// capa computa este merge para comparar contra el `view_id` guardado.
/// Es el segundo merge de `view_id_of`: `view_id_of(sk) ==
/// view_id_from_view_key(derive_view_key(sk))`. Existe para que la puerta
/// autenticada no reimplemente el hash ni reciba la clave de gasto.
pub fn view_id_from_view_key(view_key: Digest) -> Digest {
    native_merge(as_digest(BaseElement::new(VIEW_KEY_DOMAIN)), view_key)
}

pub fn view_id_of(spend_key: BaseElement) -> Digest {
    native_merge(
        as_digest(BaseElement::new(VIEW_KEY_DOMAIN)),
        derive_view_key(spend_key),
    )
}

/// Variante ANCHA del view_id almacenado (49-A paso 2).
pub fn view_id_of_wide(spend_key: Digest) -> Digest {
    native_merge(
        as_digest(BaseElement::new(VIEW_KEY_DOMAIN)),
        derive_view_key_wide(spend_key),
    )
}
