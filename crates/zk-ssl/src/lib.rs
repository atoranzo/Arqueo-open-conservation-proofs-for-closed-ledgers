//! # ZK-Sovereign Settlement Layer
//!
//! Capa de liquidación con privacidad criptográfica y cumplimiento
//! demostrable, **sin ninguna ceremonia de confianza**.
//!
//! ## Por qué STARK y no otro paradigma
//!
//! El proyecto implementó el mismo circuito en cinco paradigmas (Groth16,
//! Halo2/IPA, STARK/FRI, PLONK/KZG y Nova) y midió sus trade-offs. La
//! elección para la capa se deduce del principio de soberanía:
//!
//! **Groth16 y PLONK-KZG exigen una ceremonia de confianza.** Si sus
//! participantes coluden, pueden falsificar pruebas y **crear dinero de
//! la nada**, sin que nadie lo detecte jamás. Para una infraestructura
//! soberana eso es una dependencia externa permanente e inauditable.
//!
//! Sin ceremonia quedan Halo2/IPA y STARK/FRI. De los dos, **STARK gana
//! en todo salvo el tamaño de prueba** y es el único con resistencia
//! cuántica — lo que importa en una infraestructura pensada para durar
//! décadas.
//!
//! El precio, medido. Y hay **dos cifras**, que no son la misma y
//! durante muchos sellos se citaron como si lo fueran:
//!
//! | qué | tamaño | dónde se mide |
//! |---|---|---|
//! | **circuito de comparación** (blowup 16, ext. cuadrática) | 36,7 KB | `FIVE_BACKENDS.md`: es lo que se enfrenta a Groth16, PLONK y Halo2 sobre la MISMA carga |
//! | **circuitos de esta capa** | **53,6-65,3 KB** | §218, banco C0.1: `mint` 54.858 y 55.568 B · `send` 66.164 B · `claim` 66.820 B |
//!
//! Los circuitos de producción son **1,5-1,8× más grandes** que el de
//! comparación. Las dos medidas son correctas; lo que faltaba era
//! decir que son de cosas distintas. Nadie había medido los de
//! producción hasta C0.1.
//!
//! Frente a los 192 bytes de Groth16, es el coste de no depender de
//! nadie: sin ceremonia de confianza y con resistencia cuántica.
//!
//! ## Qué garantiza cada operación
//!
//! Sin revelar identidades, saldos ni importes:
//!
//! | Vía de crear dinero | Cerrada por |
//! |---|---|
//! | Transferir más de lo debitado | Conservación (partida doble) |
//! | Abrir cuenta con saldo | Apertura siempre a cero |
//! | Emitir sin autorización | Dos custodios demostrados en circuito |
//! | Emisión encubierta | Suministro público atado en el circuito |
//! | Gastar dos veces | Encadenamiento de raíces (orden total del nodo único) |
//! | Gastar sin ser el titular | Autoridad de gasto |
//! | Reenviar una operación válida | Encadenamiento de raíces en la capa |
//!
//! ## El modelo de operación
//!
//! ```text
//! layer.open_account(sk)                    → cuenta con saldo CERO
//! layer.apply_mint_delegated(...)           → DOS custodios DISTINTOS
//!
//! // Pago en dos fases, SIN entregar ninguna clave a la capa:
//! let m = layer.send_materials(origen, id_receptor, importe, aleatorio)?;
//! let envio = client::prove_send(&m, sk, opts)?; // en la maquina del titular
//! layer.apply_send(&envio, origen, &estado, importe)?;
//! // ...y el receptor cobra igual: claim_materials → prove_claim → apply_claim
//! ```
//!
//! Generar la prueba y aplicarla están **separados a propósito**: permite
//! que quien produce la prueba y quien la acepta sean partes distintas,
//! que es el caso real entre entidades.
//!
//! ## ⚠️ Lo que esta capa NO es
//!
//! - **No hay red ni consenso.** Es un nodo único. Una
//!   federación real necesita acuerdo sobre el orden de las operaciones —
//!   un problema de sistemas distribuidos, no de criptografía.
//! - **No persiste por sí sola.** Abierta sin almacenamiento vive en
//!   memoria y reiniciar pierde el ledger; con él, `persistence.rs` lo
//!   guarda en `sled`, con cifrado autenticado en reposo.
//! - **No hay delegación de la prueba.** Quien la genera necesita la
//!   clave de gasto; en un banco, la clave estaría en un HSM y el cómputo
//!   en otro servicio.
//! - **No hay política monetaria.** La destrucción de circulante SÍ
//!   existe —`burn.rs`, con su circuito y su medida en `metrics.rs`—;
//!   lo que no hay es una regla que gobierne emisión y destrucción
//!   más allá del tope inmutable del ledger.
//! - **No hay umbral configurable.** Emitir, emitir a pendiente, congelar y
//!   recuperar exigen DOS custodios distintos del conjunto autorizado, y ese
//!   dos es fijo: no hay k-de-n. Cambiar ese conjunto es otro umbral de dos,
//!   sobre el conjunto de GOBERNANZA y con dominio propio. La garantía es
//!   "dos claves comprometidas en vez de una", **no**
//!   "dos voluntades independientes": en un nodo único, quien genera la
//!   prueba necesita las dos claves a la vez.
//! - **Nada de esto ha sido auditado por terceros.**

mod accounts;
mod migration;
mod audit;
pub mod commitment;
pub mod pending;
pub mod two_phase;
pub mod client;
pub mod crypto;
pub mod iso;
pub mod log;
use crate::log::{OpKind, TransitionLog};
mod burn;
mod mint;
mod freeze;
mod governance;
mod persistence;
mod recovery;
pub mod snapshot;
pub mod sparse_tree;
pub mod store;

// §318: este modulo YA NO va tras cfg(test). Su CONTENIDO entero sigue
// estandolo -las dos regiones #[cfg(test)] de metrics.rs-, asi que en
// release compila VACIO salvo por la cifra publicada, que es lo unico
// que sale de aqui y lo unico que el nodo consume.
mod metrics;
#[cfg(test)]
mod tests;
#[cfg(any(test, feature = "sandbox"))]
pub mod tests_support;

use std::collections::BTreeSet;
use std::collections::HashMap;
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::FieldElement;
use winterfell::{
    crypto::hashers::Blake3_256, crypto::DefaultRandomCoin, crypto::MerkleTree, verify,
    AcceptableOptions, BatchingMethod, FieldExtension, ProofOptions, Prover,
};

use stark_experiment::circuit_audit::{
    build_trace as build_audit_trace, AuditAir, AuditProver, AuditPublicInputs, AuditWitness,
    MAX_VALUE,
};
use stark_experiment::circuit_burn::{
    build_trace as build_burn_trace, BurnAir, BurnProver, BurnPublicInputs,
};
/// Autorización de dos miembros del conjunto de gobernanza.
use stark_experiment::circuit_freeze::{frozen_leaf, FROZEN_DEPTH};
pub use stark_experiment::circuit_governance::{build_governance_set};
/// Autorización de dos custodios, necesaria para emitir.
/// Construcción del conjunto de custodios: devuelve su raíz y los
/// caminos de cada miembro.
pub use stark_experiment::circuit_threshold::build_custodian_set;
// §318: la cifra por pago se hace API porque el nodo la consume.
// `mod metrics` sigue privado; lo que se publica es solo esta const.
pub use crate::metrics::PUBLICADA_PAGO_B;
use stark_experiment::native::{
    derive_public_id, native_leaf, native_leaf_salted,
};
// El circuito de la via en dos fases. Estaba importado solo dentro de
// `two_phase.rs`, asi que `client.rs` no podia generar un envio sin pasar
// por la capa — que es justo lo que `AUDITORIA.md` §33 senala.
use stark_experiment::circuit_claim::{
    build_trace as build_claim_trace, ClaimProver,
};
use stark_experiment::circuit_send::{
    build_trace as build_send_trace, SendProver,
};
use stark_experiment::merkle::{Digest, MerklePath};

use sparse_tree::SparseTree;
use store::{digest_from_bytes, digest_to_bytes, StoreError};

type Blake3 = Blake3_256<BaseElement>;

/// Índice de una cuenta dentro del árbol.
pub type AccountIndex = u64;

/// Opciones de prueba del sistema.
///
/// **Configuración de 127 bits conjeturados** (blowup 16, extensión
/// cuadrática). Ver `FIVE_BACKENDS.md` sobre la diferencia entre
/// seguridad conjeturada y demostrable: alcanzar 128 bits DEMOSTRABLES
/// costaría 125,6 KB por prueba en vez de 36,7.
pub fn proof_options() -> ProofOptions {
    ProofOptions::new(
        42,
        16,
        21,
        FieldExtension::Quadratic,
        8,
        31,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

/// `T` por defecto de la caducidad (§178), en latidos de `log.seq`.
/// Línea sistémica: se declara, se publica y se revisa con datos.
pub const DEFAULT_REFUND_TTL: u64 = 64;
/// Centinela de `pending_meta`: el pendiente nació por EMISIÓN, no tiene
/// emisor-cuenta; su caducidad DES-EMITE (§178 §4).
pub const REFUND_SENDER_NONE: u64 = u64::MAX;

#[derive(Debug)]
pub enum LayerError {
    /// El pendiente aún no cumplió la `T` de caducidad (§178).
    RefundTooEarly { born: u64, now: u64, ttl: u64 },
    /// La posición no admite reembolso: sin meta (legado) o centinela de
    /// emisión (su vía es la des-emisión, no el crédito).
    RefundUnavailable,
    /// La posición no casa con los materiales: hoja distinta del
    /// compromiso o importe distinto del registrado.
    PendingMismatch,
    AccountNotFound(AccountIndex),
    InsufficientBalance { available: u64, requested: u64 },
    OverRegulatoryLimit { limit: u64, requested: u64 },
    NullifierAlreadySpent,
    /// **La posicion del nullificador esta ocupada por OTRO nullificador.**
    ///
    /// No es un doble gasto: es una colision. La posicion se deriva del
    /// propio nullificador —`nullificador[0] mod 2^32`— asi que dos pagos
    /// legitimos distintos pueden caer en la misma.
    ///
    /// ⚠️ **El pago queda bloqueado y no hay reintento posible**: el
    /// nullificador es determinista a partir del estado de la cuenta.
    ///
    /// Antes esto se reportaba como `NullifierAlreadySpent`, que **acusaba
    /// al usuario honesto de algo que no habia hecho**. Ver `AUDITORIA.md`
    /// §13.
    NullifierPositionCollision { position: u64 },
    /// **El arbol de pendientes agoto sus posiciones.**
    ///
    /// ⚠️ **Corregido en §211.** Este comentario decia que `next_pending`
    /// «solo sube: nunca reutiliza las posiciones ya reclamadas», y que
    /// por tanto el limite era de transferencias TOTALES. **Es falso**:
    /// `allocate_pending` recorre desde cero y **reutiliza** los huecos que
    /// deja `apply_claim`. El propio `two_phase.rs` ya habia corregido la
    /// misma afirmacion en su comentario; aqui sobrevivio. El limite real
    /// es de pendientes **simultaneos**, no totales.
    ///
    /// ⚠️ Sin esta comprobacion, `path_for` produciria un camino que no
    /// llega a la raiz y la prueba fallaria **sin decir por que**.
    ///
    /// Cerrarlo exige rotar el arbol o reutilizar posiciones liberadas, y
    /// **no esta hecho**. Ver `AUDITORIA.md` §13.
    PendingTreeExhausted { capacity: u64 },
    /// **Dos operaciones de la misma cuenta en un lote** (§215).
    ///
    /// `apply_many` valida las N contra una instantanea de arranque. Si
    /// dos tocaran la misma cuenta, la segunda habria calculado su hoja
    /// nueva sobre un saldo que la primera ya cambio, y su prueba dejaria
    /// de acreditar la transicion que se aplica. Se rechaza el lote
    /// entero: es un error de composicion de quien lo arma, no del pago.
    DuplicateAccountInBatch { index: AccountIndex },
    /// **Dos operaciones sobre la misma posicion de pendiente en un lote**
    /// (§215). Misma razon: quien arma el lote debe reservar (§211).
    DuplicatePendingInBatch { position: u64 },
    NotTheIssuer,
    /// El conjunto de custodios agotó su cupo de intervenciones.
    ///
    /// **No es un fallo: es la rotación funcionando.**
    CustodianSetExhausted { uses: u64, max: u64 },
    ProofFailed(String),
    VerificationFailed(String),
    StaleState,
    WrongRegulatoryLimit { expected: u64, declared: u64 },
    /// Fallo de persistencia, incluida la corrupción del ledger.
    Store(StoreError),
    /// Quien pide la revelación no es el titular de la cuenta.
    NotTheAccountHolder,
    /// El saldo no está en la banda que se pretende demostrar.
    BalanceOutsideBand { lower: u64, upper: u64 },
    /// La emisión superaría el tope del sistema.
    SupplyCapExceeded { cap: u64, would_be: u64 },
    /// La recuperación asigna la misma identidad que ya tenía: no
    /// cambiaría nada y solo incrementaría el contador.
    RecoveryToSameIdentity,
    /// Se alcanzó el tope de cuentas del sistema.
    AccountLimitReached { limit: u64 },
    /// La cuenta está congelada: no puede gastar.
    AccountFrozen(AccountIndex),
    /// La congelación no cambiaría nada.
    AlreadyInThatFreezeState,
}

impl From<StoreError> for LayerError {
    fn from(e: StoreError) -> Self {
        LayerError::Store(e)
    }
}

impl std::fmt::Display for LayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LayerError::*;
        match self {
            AccountNotFound(i) => write!(f, "la cuenta {i} no existe"),
            RefundTooEarly { born, now, ttl } => write!(
                f,
                "el pendiente nacio en seq {born}; a seq {now} no cumple la \
                 T de caducidad ({ttl})"
            ),
            RefundUnavailable => write!(
                f,
                "la posicion no admite reembolso: sin meta (legado) o de emision"
            ),
            PendingMismatch => write!(
                f,
                "la posicion no casa con los materiales (hoja o importe)"
            ),
            InsufficientBalance {
                available,
                requested,
            } => write!(f, "saldo insuficiente: hay {available}, se piden {requested}"),
            OverRegulatoryLimit { limit, requested } => write!(
                f,
                "importe {requested} por encima del limite regulatorio {limit}"
            ),
            PendingTreeExhausted { capacity } => write!(
                f,
                "el arbol de pendientes agoto sus {capacity} posiciones: el \
                 contador nunca reutiliza las liberadas, asi que el limite \
                 es de transferencias totales, no simultaneas"
            ),
            NullifierPositionCollision { position } => write!(
                f,
                "la posicion {position} ya esta ocupada por OTRO nullificador: \
                 es una colision, no un doble gasto. El pago no puede \
                 completarse y no hay reintento posible"
            ),
            NullifierAlreadySpent => {
                write!(f, "el nullifier ya se gasto: doble gasto rechazado")
            }
            CustodianSetExhausted { uses, max } => write!(
                f,
                "el conjunto de custodios agoto su cupo ({uses}/{max}): la \
                 gobernanza debe rotarlo"
            ),
            NotTheIssuer => write!(
                f,
                "crear dinero exige dos custodios distintos del conjunto autorizado"
            ),
            ProofFailed(e) => write!(f, "no se pudo generar la prueba: {e}"),
            VerificationFailed(e) => write!(f, "la prueba no verifica: {e}"),
            StaleState => write!(
                f,
                "la operacion parte de un estado que ya no es el actual"
            ),
            WrongRegulatoryLimit { expected, declared } => write!(
                f,
                "limite regulatorio invalido: el sistema impone {expected}, se declara {declared}"
            ),
            Store(e) => write!(f, "{e}"),
            NotTheAccountHolder => write!(
                f,
                "solo el titular puede revelar el saldo de su cuenta"
            ),
            BalanceOutsideBand { lower, upper } => write!(
                f,
                "el saldo no esta en la banda [{lower}, {upper}] que se pretende demostrar"
            ),
            SupplyCapExceeded { cap, would_be } => write!(
                f,
                "la emision llevaria el suministro a {would_be}, por encima del tope {cap}"
            ),
            RecoveryToSameIdentity => write!(
                f,
                "la recuperacion asignaria la misma identidad que ya tenia"
            ),
            AccountLimitReached { limit } => write!(
                f,
                "se alcanzo el tope de {limit} cuentas del sistema"
            ),
            AccountFrozen(i) => write!(f, "la cuenta {i} esta congelada y no puede gastar"),
            AlreadyInThatFreezeState => {
                write!(f, "la cuenta ya esta en ese estado de congelacion")
            }
            DuplicateAccountInBatch { index } => write!(
                f,
                "el lote lleva dos operaciones de la cuenta {index}: hay que \
                 armarlo con una por cuenta"
            ),
            DuplicatePendingInBatch { position } => write!(
                f,
                "el lote lleva dos operaciones sobre la posicion pendiente \
                 {position}: hay que reservar antes de repartir materiales"
            ),
        }
    }
}
impl std::error::Error for LayerError {}

/// Datos que el nodo guarda de cada cuenta.
///
/// **El operador del nodo ve estos valores.** La privacidad es frente a
/// terceros que solo ven las pruebas, no frente a quien mantiene el
/// estado. Una federación real repartiría el estado entre nodos.
#[derive(Clone, Debug)]
struct AccountRecord {
    public_id: Digest,
    balance: u64,
    nonce: BaseElement,
    /// Compromiso de la clave de vista (49-A): `view_id_of(spend_key)`,
    /// fijado al abrir. `VIEW_ID_LEGACY` (cero) en cuentas pre-49-A.
    /// Las operaciones que no rotan la clave lo PRESERVAN; recovery lo
    /// toca y es la costura 49-A<->52 (ver TODO en recovery.rs).
    view_id: Digest,
    /// Salt de la hoja (entrada 50 / B13-B14): `derive_leaf_salt(sk)`,
    /// fijado al abrir. `LEAF_SALT_LEGACY` (cero) en cuentas migradas
    /// (salt-cero, no-retro §126.4) y pre-B13/B14. La capa lo lee para
    /// recomputar la hoja salteada (no puede derivarlo: §93.4). Se
    /// PRESERVA al operar; recovery lo toca (misma costura que view_id).
    leaf_salt: Digest,
}

/// Revelación dirigida a un supervisor.
///
/// Viaja sola: el supervisor **no necesita acceso a la capa** para
/// verificarla, solo la raíz del estado que audita.
#[derive(Debug)]
pub struct AuditDisclosure {
    pub proof: Vec<u8>,
    pub public_inputs: AuditPublicInputs,
}

/// Verifica una revelación. **Función libre a propósito**: un supervisor
/// la usa sin tener el ledger ni ningún estado del sistema.
///
/// Devuelve `Ok(())` si la prueba es válida para la raíz declarada. Queda
/// a cargo de quien verifica comprobar que esa raíz es la del estado que
/// pretende auditar — igual que en las liquidaciones, el circuito prueba
/// la afirmación y quien la recibe ancla el contexto.
pub fn verify_audit(disclosure: &AuditDisclosure) -> Result<(), LayerError> {
    let proof = winterfell::Proof::from_bytes(&disclosure.proof)
        .map_err(|e| LayerError::VerificationFailed(format!("prueba mal formada: {e:?}")))?;
    let min_opts = AcceptableOptions::OptionSet(vec![proof_options()]);
    verify::<AuditAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
        proof,
        disclosure.public_inputs.clone(),
        &min_opts,
    )
    .map_err(|e| LayerError::VerificationFailed(format!("{e:?}")))
}

/// Recibo de una emisión.
/// Recibo de una destrucción de circulante.
///
/// **La identidad de la cuenta no aparece**: desde fuera se sabe que
/// alguien con autoridad destruyó ese importe y que el suministro bajó en
/// consecuencia, pero no de quién era.
#[derive(Debug)]
pub struct BurnReceipt {
    pub proof: Vec<u8>,
    pub public_inputs: BurnPublicInputs,
}

/// Intervenciones que admite un conjunto de custodios antes de exigir
/// rotación.
///
/// ⚠️ **Es un valor por defecto, no una recomendación.** Elegirlo es una
/// decisión de política: más bajo reduce la ventana de una clave
/// comprometida; más alto reduce la fricción operativa.
pub const DEFAULT_MAX_CUSTODIAN_USES: u64 = 100;

/// ⚠️ **No implementa `Debug` deliberadamente.** Contiene los saldos de
/// todas las cuentas, y un `{:?}` en un registro de diagnóstico los
/// expondría. Para inspeccionar el estado están los accesores
/// (`balance_of`, `state_root`, `total_supply`...), que devuelven lo
/// concreto que se pide.
impl SovereignLayer {
    /// **Cabeza de época publicable.** `CONFIANZA_RESIDUAL.md` B10.1.
    ///
    /// Reúne lo que un testigo externo necesita para comprobar que su vista
    /// del sistema coincide con la de otro: la altura, las tres raíces y el
    /// compromiso de todo el historial.
    ///
    /// ⚠️ **Todo esto ya existía**; lo que faltaba era **exponerlo junto**.
    /// La garantía del README —«no puede reescribir el historial en
    /// secreto»— es condicional a que alguien haya observado una cabeza
    /// anterior (`AUDITORIA.md` §76). ⚠️ **Eso dejó de ser cierto en el
    /// §268**: el nodo firma la cabeza con XMSS (`firma_cabeza.rs`), la sirve
    /// por `zkssl_signedEpochHead` y el testigo de la CLI la consume y la
    /// verifica. Sigue siendo **detectable, no impedido**: vale para quien ya
    /// observó una cabeza anterior y guarde con qué comparar, y el nodo solo
    /// firma si tiene clave (`sin_clave_hay_cabeza_pero_no_firma`).
    ///
    /// Los dos campos de §275 **los aporta el llamante**: la raíz de
    /// acuses y `n` son del nodo (diario + `vista_acuses`), y la capa no
    /// va a adivinarlos. Una pareja neutra (`as_digest(0)`, `0`) sirve
    /// donde el árbol de acuses no pinta nada, y se ve que es neutra.
    pub fn epoch_head(
        &self,
        acuses_root: zk_ssl_hash::Digest,
        n: u64,
        mmr_cima: zk_ssl_hash::Digest,
        mmr_t: u64,
    ) -> crate::log::EpochHead {
        crate::log::EpochHead {
            acuses_root,
            n,
            mmr_cima,
            mmr_t,
            seq: self.log.len() as u64,
            accounts_root: self.accounts.root(),
            pending_root: self.pending.root(),
            frozen_root: self.frozen.root(),
            chain_digest: self.log.head(),
        }
    }
}

pub struct SovereignLayer {
    accounts: SparseTree,
    /// **Transferencias pendientes de reclamar.**
    ///
    /// El pagador deposita aquí un compromiso atado a la identidad del
    /// receptor; el receptor lo reclama demostrando que es suyo.
    ///
    /// Existe para que **el pagador no necesite el saldo del receptor**:
    /// la liquidación de un solo paso actualizaba las dos hojas, y quien
    /// probaba tenía que conocer las dos.
    pending: SparseTree,
    /// Siguiente posición libre del árbol de pendientes.
    next_pending: u64,
    /// **Posiciones de pendiente ENTREGADAS y aun no aplicadas** (§211,
    /// pieza 1 de la etapa 2 del RFC-0002).
    ///
    /// `allocate_pending` mira el estado ACTUAL del arbol. Con lotes eso
    /// no basta: dos clientes que pidan materiales contra la misma raiz de
    /// arranque recibirian **la misma posicion**, y el segundo `apply`
    /// pisaria la nota del primero (§210). Reservar cierra esa ventana.
    ///
    /// ⚠️ **NO se persiste, a proposito.** Una reserva significa
    /// «entregada, no aplicada»; si el proceso muere, nada se aplico y
    /// todas las reservas deben morir con el. Persistirlas solo serviria
    /// para perder posiciones para siempre.
    reserved_pending: BTreeSet<u64>,
    /// **Importe de cada pendiente sin cobrar, por posición.**
    ///
    /// Existe para poder comprobar la invariante global cuando hay dinero
    /// en tránsito: `suma de saldos + suma de pendientes == suministro`.
    ///
    /// ⚠️ **La capa ya conocía estos importes** —los recibe como parámetro
    /// de `send` y los usa para el límite regulatorio—. Esto no añade
    /// visibilidad: la hace utilizable. Ver `total_pending()`.
    pending_amounts: HashMap<u64, u64>,
    /// Metadatos del pendiente (R-2a, §178/§179): quién lo creó y cuándo,
    /// para la caducidad. `sender_index == REFUND_SENDER_NONE` marca los
    /// de EMISIÓN (des-emisión al caducar, no reembolso). Misma clase de
    /// persistencia que `pending_amounts`: sled (`pmeta:`), no snapshot.
    pending_meta: HashMap<u64, (u64, u64)>,
    /// `T` de la caducidad (§178): latidos de `log.seq` que deben pasar
    /// antes de que un pendiente sea reembolsable/des-emitible. Línea
    /// sistémica declarada, familia `N_max`/`M`.
    refund_ttl: u64,
    records: HashMap<AccountIndex, AccountRecord>,
    next_index: AccountIndex,
    /// **Raíz del conjunto de custodios autorizados.** Crear dinero,
    /// congelar y recuperar cuentas exige dos custodios distintos del
    /// conjunto.
    ///
    /// **Mutable por gobernanza**: si un custodio se compromete, el
    /// conjunto puede cambiarse sin crear un ledger nuevo.
    ///
    /// ⚠️ La garantía es "dos claves comprometidas en vez de una", **no**
    /// "dos voluntades independientes": en un nodo único, quien genera la
    /// prueba necesita las dos claves a la vez.
    custodian_set_root: Digest,
    /// **Raíz del conjunto de gobernanza. INMUTABLE.**
    ///
    /// Es la única autoridad que puede cambiar el conjunto de custodios,
    /// y cambiarla a ella exige crear un ledger nuevo.
    ///
    /// La circularidad no desaparece —quien la controle controla todo—
    /// pero se traslada a claves que se usan casi nunca y pueden
    /// guardarse sin conexión.
    governance_set_root: Digest,
    /// Contador público de cambios de gobernanza.
    governance_change_count: u64,
    total_supply: u64,
    /// **Contador público de recuperaciones.** Cada intervención de los
    /// custodios lo incrementa.
    ///
    /// Sin él, los custodios podrían reasignar cuentas en silencio: desde
    /// fuera, una recuperación es indistinguible de cualquier otra
    /// transición de estado. No impide el abuso —nada en un circuito
    /// puede— pero lo hace **contable**.
    /// **Árbol de congelados.** Profundidad 24 —no 32— porque su subida
    /// tiene que caber en las filas libres del circuito de liquidación.
    ///
    /// Una cuenta congelada no puede gastar, y **eso lo impone el
    /// circuito**: la prueba de liquidación acredita que el emisor no
    /// está en este árbol.
    frozen: SparseTree,
    /// Contador público de congelaciones y descongelaciones.
    freeze_count: u64,
    /// **Registro encadenado de transiciones.**
    ///
    /// Hace que el operador no pueda reescribir el historial en secreto.
    /// No impide que vea los saldos ni que censure: eso exige consenso.
    log: TransitionLog,
    /// Entradas del registro **ya escritas en disco**. `commit` se
    /// salta esas: el registro es de solo anadir, asi que
    /// reescribirlas es trabajo tirado — y con N operaciones son
    /// N(N+1)/2 escrituras. Medido en B.4: `crear` de 1e3 a 1e4
    /// subio 56x cuando las cuentas subieron 10x.
    ///
    /// Solo avanza tras un `flush` con exito, y solo se adelanta en
    /// `load()`, donde lo leido ES el disco. Si se quedara corta se
    /// reescribe de mas: lento, correcto. Nunca al reves.
    log_persisted: usize,
    recovery_count: u64,
    /// **Intervenciones del conjunto de custodios vigente.**
    ///
    /// Emitir, emitir a pendiente, congelar y recuperar lo incrementan. Al
    /// alcanzar `max_custodian_uses`, los custodios
    /// **dejan de poder actuar** hasta que la gobernanza rote el conjunto.
    ///
    /// Es la rotación de privilegios expresada por **uso**, no por tiempo:
    /// esta capa no tiene noción de tiempo. Sin rotación, una clave
    /// comprometida sirve para siempre.
    custodian_uses: u64,
    /// Cuántas intervenciones admite un conjunto antes de exigir rotación.
    max_custodian_uses: u64,
    regulatory_limit: u64,
    /// **Tope de emisión.** Parámetro inmutable del ledger: ni siquiera
    /// el conjunto completo de custodios puede superarlo sin crear un
    /// ledger nuevo, y eso dejaría rastro evidente.
    max_supply: u64,
    /// **Tope de cuentas.** Parámetro inmutable.
    ///
    /// `open_account` no exige autorización de ningún tipo, así que sin
    /// un tope cualquiera podría crear cuentas hasta agotar la memoria
    /// del nodo. No crea dinero —nacen a cero— pero es denegación de
    /// servicio trivial.
    ///
    /// ⚠️ **Acota el daño, no impide el abuso.** Un atacante puede agotar
    /// el cupo y dejar sin sitio a usuarios legítimos. Cerrarlo de verdad
    /// exigiría autorización de custodio para abrir, y eso requiere un
    /// circuito nuevo: abrir hoy no genera ninguna prueba.
    max_accounts: u64,
    options: ProofOptions,
    /// Almacenamiento persistente. `None` para una capa en memoria.
    db: Option<sled::Db>,
    /// Clave de cifrado en reposo. `None` = sin cifrar.
    ///
    /// **No se guarda junto a los datos**: la aporta el operador al
    /// arrancar. Guardarla al lado no protegería nada.
    key: Option<crate::crypto::LedgerKey>,
}

impl SovereignLayer {
    /// Arranca la capa.
    ///
    /// **No hay setup de claves.** Es la propiedad que distingue este
    /// paradigma: no existe ninguna ceremonia que celebrar ni ningún
    /// secreto que destruir. Los parámetros son públicos y verificables.
    pub fn new(
        custodian_set_root: Digest,
        governance_set_root: Digest,
        regulatory_limit: u64,
        max_supply: u64,
        max_accounts: u64,
    ) -> Self {
        Self {
            accounts: SparseTree::new(),
            pending: SparseTree::new(),
            next_pending: 0,
            reserved_pending: BTreeSet::new(),
            pending_amounts: HashMap::new(),
            pending_meta: HashMap::new(),
            refund_ttl: DEFAULT_REFUND_TTL,
            records: HashMap::new(),
            next_index: 0,
            custodian_set_root,
            governance_set_root,
            governance_change_count: 0,
            custodian_uses: 0,
            max_custodian_uses: DEFAULT_MAX_CUSTODIAN_USES,
            total_supply: 0,
            frozen: SparseTree::with_depth(FROZEN_DEPTH),
            freeze_count: 0,
            log: TransitionLog::new(),
            log_persisted: 0,
            recovery_count: 0,
            regulatory_limit,
            max_supply,
            max_accounts,
            options: proof_options(),
            db: None,
            key: None,
        }
    }

}
