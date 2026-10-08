//! Ayudantes compartidos por los tests de los distintos módulos.
//!
//! Viven aquí y no en `tests.rs` porque el puente ISO también los
//! necesita, y duplicarlos haría que dos suites divergieran en silencio.

// ✅ **`open_and_fund` fondea por la VIA DELEGADA desde B-0b-ii (§163).**
//
// Era el punto por el que la mitad de la suite dependia de la via
// marcada **sin nombrarla**; hoy sus 185 usos ejercitan la via real.
// El precio se midio ANTES de girar la llave (§162): x3,54-3,70 por
// fondeo, pagado en maquina de CI y no en usuario.
//
// El `allow` de abajo ampara únicamente `open_account` (64 bits,
// opt-in por §97.4, fuera de B por §160). La vía antigua de custodios
// ya no existe: B-3 la retiró con el libro en la mano (§171).
//
// §65.3: el permiso va en los tests, no en la definicion.
// Ampara `open_account` (64 bits), viva y opt-in por §97.4 (fuera de B
// por §160). La vía-recibo de custodios que antes también amparaba se
// retiró en B-3 (§171).
#![allow(deprecated)]

use super::*;
use stark_experiment::circuit_mint_climb as climb;
use stark_experiment::circuit_frozen_climb as climb_frozen;
use stark_experiment::circuit_recovery_climb as climb_recovery;
use stark_experiment::circuit_mint_pending_climb as climb_pending;
use stark_experiment::circuit_threshold::CUSTODIAN_DOMAIN;
use stark_experiment::circuit_threshold_single_nullifier as auth;

pub const SK_ALICE: u64 = 0xA11CE;
pub const SK_BOB: u64 = 0xB0B;
/// Cinco custodios. Emitir exige dos distintos.
pub fn custodian_keys() -> Vec<BaseElement> {
    vec![
        BaseElement::new(0xC0570D1A),
        BaseElement::new(0xC0570D1B),
        BaseElement::new(0xC0570D1C),
        BaseElement::new(0xC0570D1D),
        BaseElement::new(0xC0570D1E),
    ]
}

/// Claves del conjunto de GOBERNANZA. Distintas de las de custodio: la
/// separación de dominio es lo que hace real la jerarquía.
pub fn governance_keys() -> Vec<BaseElement> {
    vec![
        BaseElement::new(0x60_5E_00),
        BaseElement::new(0x60_5E_01),
        BaseElement::new(0x60_5E_02),
        BaseElement::new(0x60_5E_03),
    ]
}

pub fn governance_root() -> Digest {
    build_governance_set(&governance_keys()).0
}

pub fn custodian_root() -> Digest {
    stark_experiment::circuit_threshold::build_custodian_set(&custodian_keys()).0
}

pub const LIMIT: u64 = 500_000;
pub const MAX_SUPPLY: u64 = 100_000_000;
pub const MAX_ACCOUNTS: u64 = 1_000;

/// Capa con un cupo de custodios pequeño, para probar la rotación sin
/// hacer cien emisiones.
/// El estado que un titular conoce de su propia cuenta.
///
/// En los tests se obtiene de la capa por comodidad. **En un despliegue lo
/// lleva el cliente**: la capa por compromisos no lo tendría.
/// **Transferencia completa por la vía en dos fases: enviar y cobrar.**
///
/// Existe para los tests donde la transferencia es **montaje**, no lo que
/// se comprueba. Sin él, cada uno repetiría catorce líneas de ciclo y el
/// ruido taparía lo que el test dice comprobar.
///
/// ⚠️ **No usar donde la transferencia SEA el objeto del test.** Ahí hay que
/// ver las dos fases por separado: que el receptor no tiene el dinero hasta
/// cobrarlo es una propiedad, no un detalle.
pub fn two_phase_transfer(
    layer: &mut SovereignLayer,
    from: AccountIndex,
    from_key: u64,
    to: AccountIndex,
    to_key: u64,
    amount: u64,
    salt: Digest,
) -> Result<(), crate::LayerError> {
    let estado_from = state_of(layer, from);
    let receptor = layer
        .public_id_of(to)
        .ok_or(crate::LayerError::AccountNotFound(to))?;
    let recibo = layer.send(
        BaseElement::new(from_key),
        from,
        &estado_from,
        receptor,
        salt,
        amount,
    )?;
    layer.apply_send(&recibo, from, &estado_from, amount)?;

    let estado_to = state_of(layer, to);
    let cobro = layer.claim(BaseElement::new(to_key), to, &estado_to, &recibo.notice)?;
    layer.apply_claim(&cobro, to, &estado_to, &recibo.notice)?;
    Ok(())
}

/// **Semilla determinista para el aleatorio de un pendiente.**
///
/// Vive aquí y no en `tests.rs` porque `metrics.rs` también lo necesita: el
/// arné­s mide la vía en dos fases, y un envío exige un aleatorio.
pub fn salt_de(seed: u64) -> Digest {
    [
        BaseElement::new(seed),
        BaseElement::new(seed + 1),
        BaseElement::new(seed + 2),
        BaseElement::new(seed + 3),
    ]
}

pub fn state_of(layer: &SovereignLayer, index: AccountIndex) -> crate::commitment::ClientState {
    crate::commitment::ClientState {
        public_id: layer.public_id_of(index).expect("cuenta"),
        balance: layer.balance_of(index).expect("cuenta"),
        nonce: layer.nonce_of(index).expect("cuenta"),
    }
}

pub fn new_layer_with_quota(quota: u64) -> SovereignLayer {
    let mut l = new_layer();
    l.set_max_custodian_uses(quota);
    l
}

pub fn new_layer() -> SovereignLayer {
    SovereignLayer::new(custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
}

/// Abre una cuenta y le emite fondos: el único camino legítimo para
/// que una cuenta tenga saldo.
/// Abre una cuenta con clave **estrecha** y la fondea.
///
/// ⚠️ Sigue tomando un `u64` a proposito: son **185 usos** que no ganan nada
/// con claves anchas, y §90 garantiza que rellenar da la misma identidad.
/// Para ejercitar los 256 bits esta [`open_and_fund_wide`].
pub fn open_and_fund(layer: &mut SovereignLayer, sk: u64, amount: u64) -> AccountIndex {
    let idx = layer.open_account(BaseElement::new(sk));
    if amount > 0 {
        fund_delegated(layer, idx, amount);
    }
    idx
}

/// **Abre una cuenta con clave ANCHA de verdad**, y la fondea.
///
/// ⚠️ Los cuatro elementos **no nulos**: es lo unico que ejercita los 256
/// bits que los cinco circuitos verifican desde §92.19. Con
/// [`open_and_fund`] —que rellena con ceros— el camino funciona pero **no
/// prueba nada nuevo** (§90.3).
pub fn open_and_fund_wide(
    layer: &mut SovereignLayer,
    sk: Digest,
    amount: u64,
) -> AccountIndex {
    let idx = layer.open_account_wide(sk);
    if amount > 0 {
        fund_delegated(layer, idx, amount);
    }
    idx
}

/// Clave ancha de prueba, derivada de una semilla corta.
///
/// Los tres elementos extra **no son cero**, que es el punto: una clave
/// `[sk, 0, 0, 0]` tiene 64 bits de entropia y no ejercita nada.
pub fn wide_key(sk: u64) -> Digest {
    [
        BaseElement::new(sk),
        BaseElement::new(sk ^ 0xA11CE),
        BaseElement::new(sk ^ 0x0DDBA11),
        BaseElement::new(sk ^ 0x5EA51DE),
    ]
}

/// Abre un ledger reintentando ante errores transitorios de E/S.
///
/// **Por qué hace falta**: `sled` mantiene un bloqueo del directorio que
/// puede tardar en liberarse tras cerrar. Un segundo `open` inmediato
/// —como el de los tests que comprueban parámetros inmutables— lo
/// encuentra a veces todavía tomado y devuelve un error de E/S en vez del
/// que se espera.
///
/// Esto **no es un artefacto de los tests**: un nodo que se reinicie
/// inmediatamente tras cerrarse puede sufrir lo mismo. Está documentado
/// como limitación operativa.
///
/// El reintento **solo** absorbe errores de E/S. Cualquier otro error
/// —incluido `ParameterMismatch`, que es lo que estos tests comprueban—
/// se devuelve de inmediato.
/// **Hermano de `open_retry` para el ledger cifrado.**
///
/// ⚠️ **Por que hace falta, medido y no supuesto.** El 31-07-2026
/// `an_encrypted_ledger_needs_the_right_passphrase` fallo **1 de 12
/// pasadas a 16 hilos** en release, con
/// *«could not acquire lock on .../db: WouldBlock»* al **reabrir
/// inmediatamente tras cerrar**. Es la entrada 18 —el bloqueo de directorio
/// de `sled`— con manifestacion medida por primera vez.
///
/// ⚠️ **Y `open_retry` ya existia**, con este mismo remedio, usado en 39
/// llamadas. Las 9 que abren cifrado no lo tenian: la proteccion existia y
/// no estaba aplicada a todo el codigo, que es el patron de §59.2.
///
/// Absorbe **solo** errores de E/S. Cualquier otro —incluida una contraseña
/// equivocada, que es lo que estos tests comprueban— se devuelve de
/// inmediato y no puede quedar enmascarado.
#[allow(clippy::too_many_arguments)]
pub fn open_encrypted_retry(
    path: &str,
    custodians: Digest,
    governance: Digest,
    limit: u64,
    max_supply: u64,
    max_accounts: u64,
    key: Option<crate::crypto::LedgerKey>,
) -> Result<SovereignLayer, LayerError> {
    for intento in 0..10 {
        match SovereignLayer::open_encrypted(
            path,
            custodians,
            governance,
            limit,
            max_supply,
            max_accounts,
            key.clone(),
        ) {
            Err(LayerError::Store(StoreError::Io(_))) if intento < 9 => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            otro => return otro,
        }
    }
    unreachable!()
}

/// `sled::open`, con el mismo rito de reintentos que [`open_retry`]:
/// tras soltar una capa, sled puede tardar en liberar el cerrojo del
/// directorio (WouldBlock), y la manipulacion directa del db en los
/// tests de corrupcion llegaba en crudo — la especie que la compuerta
/// destapo en B-2b (§165).
pub fn sled_open_retry(path: &str) -> sled::Db {
    for _ in 0..10 {
        match sled::open(path) {
            Ok(db) => return db,
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(50)),
        }
    }
    sled::open(path).expect("abrir db tras diez reintentos")
}

pub fn open_retry(
    path: &str,
    custodians: Digest,
    governance: Digest,
    limit: u64,
    max_supply: u64,
    max_accounts: u64,
) -> Result<SovereignLayer, LayerError> {
    for intento in 0..10 {
        match SovereignLayer::open(path, custodians, governance, limit, max_supply, max_accounts)
        {
            Err(LayerError::Store(StoreError::Io(_))) if intento < 9 => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            otro => return otro,
        }
    }
    unreachable!()
}

// ============================================================================
// La palanca (B-0, §160): la vía delegada como ayudante común.
//
// El molde vivía DUPLICADO en el bloque de tests de cada módulo (mint,
// freeze, governance, recovery, two_phase); aquí se asienta UNA vez, con
// los topes leídos DE LA CAPA — no de constantes locales — para servir a
// cualquier suite. §51 preside: el orden estricto se EJERCITA, no se
// esquiva. B-0a: las variantes delegadas CONVIVEN con las viejas; el flip
// de entrañas es B-0b, con su medición al lado.
// ============================================================================

/// El par de umbral sobre un conjunto de claves ARBITRARIO (dominio de
/// custodios): la pieza para probar conjuntos entrantes tras una
/// rotación, o salientes contra la raíz nueva.
pub fn custodian_pair_with(
    keys: &[BaseElement],
    op: Digest,
    a: usize,
    b: usize,
) -> (
    winterfell::Proof,
    auth::NullifierThresholdPublicInputs,
    winterfell::Proof,
    auth::NullifierThresholdPublicInputs,
) {
    assert!(a < b, "§51: index_a < index_b, estricto");
    let (_, cp) = stark_experiment::circuit_threshold::build_custodian_set(keys);
    let d = BaseElement::new(CUSTODIAN_DOMAIN);
    let prover = auth::NullifierThresholdProver::new(proof_options());
    let ta = auth::build_trace(d, keys[a], &cp[a], op);
    let ia = prover.get_pub_inputs(&ta);
    let pa = prover.prove(ta).expect("autorizacion A");
    let tb = auth::build_trace(d, keys[b], &cp[b], op);
    let ib = prover.get_pub_inputs(&tb);
    let pb = prover.prove(tb).expect("autorizacion B");
    (pa, ia, pb, ib)
}

/// El par de autorizaciones de umbral para `op`: custodios `a` y `b`,
/// distintos y en orden estricto.
pub fn delegated_pair(
    op: Digest,
    a: usize,
    b: usize,
) -> (
    winterfell::Proof,
    auth::NullifierThresholdPublicInputs,
    winterfell::Proof,
    auth::NullifierThresholdPublicInputs,
) {
    assert!(a < b, "§51: index_a < index_b, estricto");
    let ck = custodian_keys();
    let (_, cp) = stark_experiment::circuit_threshold::build_custodian_set(&ck);
    let d = BaseElement::new(CUSTODIAN_DOMAIN);
    let prover = auth::NullifierThresholdProver::new(proof_options());
    let ta = auth::build_trace(d, ck[a], &cp[a], op);
    let ia = prover.get_pub_inputs(&ta);
    let pa = prover.probar(ta).expect("autorizacion A");
    let tb = auth::build_trace(d, ck[b], &cp[b], op);
    let ib = prover.get_pub_inputs(&tb);
    let pb = prover.probar(tb).expect("autorizacion B");
    (pa, ia, pb, ib)
}

/// El compromiso de emisión delegada: estado ANTES y DESPUÉS de abonar
/// `amount` en `idx`, sellado con `OP_MINT`. Calcado del molde de mint.
pub fn mint_commitment(layer: &SovereignLayer, idx: AccountIndex, amount: u64) -> Digest {
    let rec = layer.records.get(&idx).expect("cuenta").clone();
    let mut t = layer.accounts.clone();
    t.set_leaf(
        idx,
        native_leaf_salted(
            rec.public_id,
            BaseElement::new(rec.balance + amount),
            rec.nonce,
            rec.leaf_salt,
        ),
    );
    let mut v: Vec<BaseElement> = layer.accounts.root().to_vec();
    v.extend_from_slice(&t.root());
    v.push(BaseElement::new(amount));
    v.push(BaseElement::new(layer.total_supply()));
    v.push(BaseElement::new(layer.total_supply() + amount));
    v.push(BaseElement::new(layer.max_supply()));
    auth::commit_operation(auth::OP_MINT, &v)
}

/// La subida de mint para `idx`/`amount`, con los topes de la capa.
pub fn mint_climb_proof(
    layer: &SovereignLayer,
    idx: AccountIndex,
    amount: u64,
) -> winterfell::Proof {
    let rec = layer.records.get(&idx).expect("cuenta").clone();
    let path = layer.accounts.path_for(idx);
    let trace = climb::build_trace(
        rec.public_id,
        rec.balance,
        rec.nonce,
        rec.leaf_salt,
        &path,
        amount,
        layer.total_supply(),
        amount,
        layer.max_supply(),
    );
    climb::MintClimbProver::new(proof_options())
        .probar(trace)
        .expect("subida")
}

/// Emite `amount` en `idx` por la VÍA DELEGADA: custodios 1 y 3
/// autorizan con sus pruebas, que hasta el §538 publicaban sus claves (§523).
pub fn fund_delegated(layer: &mut SovereignLayer, idx: AccountIndex, amount: u64) {
    let op = mint_commitment(layer, idx, amount);
    let subida = de_montaje(|| mint_climb_proof(layer, idx, amount));
    let (pa, ia, pb, ib) = de_montaje(|| delegated_pair(op, 1, 3));
    layer
        .apply_mint_delegated(subida, pa, ia, pb, ib, idx, amount)
        .expect("la emision delegada legitima debe aplicarse");
}

/// El compromiso de congelación delegada: raíz ANTES y DESPUÉS del
/// árbol de congelados, con el contador atado `count → count+1`.
/// Calcado del molde de freeze::tests_delegada.
pub fn freeze_commitment(layer: &SovereignLayer, idx: AccountIndex, frozen: bool) -> Digest {
    let root_old = layer.frozen_root();
    let mut t = layer.frozen.clone();
    t.set_leaf(idx, frozen_leaf(frozen));
    let mut v: Vec<BaseElement> = root_old.to_vec();
    v.extend_from_slice(&t.root());
    v.push(BaseElement::new(layer.freeze_count()));
    v.push(BaseElement::new(layer.freeze_count() + 1));
    auth::commit_operation(auth::OP_FREEZE, &v)
}

/// La subida de congelación para `idx` hacia el estado `frozen`.
pub fn freeze_climb_proof(
    layer: &SovereignLayer,
    idx: AccountIndex,
    frozen: bool,
) -> winterfell::Proof {
    let path = layer.frozen.path_for(idx);
    let trace = climb_frozen::build_trace(frozen_leaf(!frozen), frozen_leaf(frozen), &path);
    climb_frozen::FrozenClimbProver::new(proof_options())
        .probar(trace)
        .expect("subida de congelacion")
}

/// Congela o descongela `idx` por la VÍA DELEGADA: custodios 1 y 3.
pub fn set_frozen_delegated(layer: &mut SovereignLayer, idx: AccountIndex, frozen: bool) {
    let op = freeze_commitment(layer, idx, frozen);
    let subida = de_montaje(|| freeze_climb_proof(layer, idx, frozen));
    let (pa, ia, pb, ib) = de_montaje(|| delegated_pair(op, 1, 3));
    layer
        .apply_freeze_delegated(subida, pa, ia, pb, ib, idx, frozen)
        .expect("la congelacion delegada legitima debe aplicarse");
}

/// El compromiso del cambio de custodios: raíz saliente → entrante,
/// contador atado. Dominio de GOBERNANZA. Calcado del molde.
pub fn governance_commitment(layer: &SovereignLayer, nueva: Digest) -> Digest {
    let mut p: Vec<BaseElement> = layer.custodian_set_root().to_vec();
    p.extend_from_slice(&nueva);
    p.push(BaseElement::new(layer.governance_change_count()));
    p.push(BaseElement::new(layer.governance_change_count() + 1));
    auth::commit_operation(auth::OP_GOVERNANCE, &p)
}

/// El par de autorizaciones de GOBERNANZA para `op`: miembros `a` y
/// `b`, distintos y en orden estricto — §51 también preside aquí.
pub fn governance_pair(
    op: Digest,
    a: usize,
    b: usize,
) -> (
    winterfell::Proof,
    auth::NullifierThresholdPublicInputs,
    winterfell::Proof,
    auth::NullifierThresholdPublicInputs,
) {
    assert!(a < b, "§51: index_a < index_b, estricto");
    let gk = governance_keys();
    let (_, gp) = stark_experiment::circuit_governance::build_governance_set(&gk);
    let d = BaseElement::new(stark_experiment::circuit_governance::GOVERNANCE_DOMAIN);
    let prover = auth::NullifierThresholdProver::new(proof_options());
    let ta = auth::build_trace(d, gk[a], &gp[a], op);
    let ia = prover.get_pub_inputs(&ta);
    let pa = prover.probar(ta).expect("autorizacion A");
    let tb = auth::build_trace(d, gk[b], &gp[b], op);
    let ib = prover.get_pub_inputs(&tb);
    let pb = prover.probar(tb).expect("autorizacion B");
    (pa, ia, pb, ib)
}

/// Cambia el conjunto de custodios por la VÍA DELEGADA: miembros 1 y 3.
pub fn update_custodians_delegated(layer: &mut SovereignLayer, nueva: Digest) {
    let op = governance_commitment(layer, nueva);
    let (pa, ia, pb, ib) = de_montaje(|| governance_pair(op, 1, 3));
    layer
        .apply_governance_delegated(pa, ia, pb, ib, nueva)
        .expect("el cambio delegado legitimo debe aplicarse")
}

/// El compromiso de recuperación delegada: raíz ANTES → raíz con LA
/// COPIA (identidad nueva, saldo y salt preservados, nonce avanzado),
/// contador atado. Calcado del molde de recovery.
pub fn recovery_commitment(layer: &SovereignLayer, idx: AccountIndex, nueva: Digest) -> Digest {
    let rec = layer.records.get(&idx).expect("cuenta").clone();
    let mut t = layer.accounts.clone();
    t.set_leaf(idx, native_leaf_salted(nueva, BaseElement::new(rec.balance),
                                rec.nonce + BaseElement::ONE,
                                rec.leaf_salt));
    let mut v: Vec<BaseElement> = layer.accounts.root().to_vec();
    v.extend_from_slice(&t.root());
    v.push(BaseElement::new(layer.recovery_count()));
    v.push(BaseElement::new(layer.recovery_count() + 1));
    auth::commit_operation(auth::OP_RECOVERY, &v)
}

/// La subida de recuperación para `idx` hacia `nueva`.
pub fn recovery_climb_proof(
    layer: &SovereignLayer,
    idx: AccountIndex,
    nueva: Digest,
) -> winterfell::Proof {
    let rec = layer.records.get(&idx).expect("cuenta").clone();
    let path = layer.accounts.path_for(idx);
    let trace = climb_recovery::build_trace(
        rec.public_id, nueva, rec.balance, rec.balance, rec.nonce,
        rec.leaf_salt, &path,
        layer.recovery_count(), 1,
    );
    climb_recovery::RecoveryClimbProver::new(proof_options())
        .probar(trace)
        .expect("subida de recuperacion")
}

/// Recupera `idx` hacia la identidad `nueva` por la VÍA DELEGADA.
pub fn recover_delegated(layer: &mut SovereignLayer, idx: AccountIndex, nueva: Digest) {
    let op = recovery_commitment(layer, idx, nueva);
    let subida = de_montaje(|| recovery_climb_proof(layer, idx, nueva));
    let (pa, ia, pb, ib) = de_montaje(|| delegated_pair(op, 1, 3));
    layer
        .apply_recovery_delegated(subida, pa, ia, pb, ib, idx, nueva)
        .expect("la recuperacion delegada legitima debe aplicarse");
}

/// El compromiso de emisión-a-pendiente delegada: raíz del árbol de
/// pendientes ANTES/DESPUÉS de colocar el compromiso del receptor,
/// suministro atado y tope leído de la capa. Calcado del molde.
pub fn mint_pending_commitment(
    layer: &SovereignLayer,
    receiver_id: Digest,
    salt: Digest,
    amount: u64,
) -> Digest {
    let position = layer.allocate_pending().expect("posicion libre");
    let c = crate::pending::pending_commitment(receiver_id, salt, amount);
    let mut t = layer.pending.clone();
    t.set_leaf(position, c);
    let mut v: Vec<BaseElement> = layer.pending.root().to_vec();
    v.extend_from_slice(&t.root());
    v.push(BaseElement::new(amount));
    v.push(BaseElement::new(layer.total_supply()));
    v.push(BaseElement::new(layer.total_supply() + amount));
    v.push(BaseElement::new(layer.max_supply()));
    auth::commit_operation(auth::OP_MINT_PENDING, &v)
}

/// La subida de emisión-a-pendiente, con el tope de la capa.
pub fn mint_pending_climb_proof(
    layer: &SovereignLayer,
    receiver_id: Digest,
    salt: Digest,
    amount: u64,
) -> winterfell::Proof {
    let position = layer.allocate_pending().expect("posicion libre");
    let path = layer.pending.path_for(position);
    let trace = climb_pending::build_trace(
        layer.total_supply(),
        amount,
        layer.max_supply(),
        amount,
        receiver_id,
        salt,
        &path,
    );
    climb_pending::MintPendingClimbProver::new(proof_options())
        .probar(trace)
        .expect("subida a pendiente")
}

/// Emite `amount` a un PENDIENTE del receptor, por la VÍA DELEGADA.
pub fn mint_to_pending_delegated(
    layer: &mut SovereignLayer,
    receiver_id: Digest,
    salt: Digest,
    amount: u64,
) {
    let op = mint_pending_commitment(layer, receiver_id, salt, amount);
    let subida = de_montaje(|| mint_pending_climb_proof(layer, receiver_id, salt, amount));
    let (pa, ia, pb, ib) = de_montaje(|| delegated_pair(op, 1, 3));
    let _ = layer
        .apply_mint_pending_delegated(subida, pa, ia, pb, ib, receiver_id, salt, amount)
        .expect("la emision delegada a pendiente debe aplicarse");
}

// ============================================================================
// Las claves anchas de custodios y de gobernanza (RFC-0018 E2, §701).
//
// La capa solo recibe las dos raíces: con qué claves se constituyó un conjunto
// no lo sabe, y no tiene por qué. Estos ayudantes constituyen los dos conjuntos
// con claves de cuatro elementos NO NULOS y firman con ellas, para ejercitar los
// 256 bits que el umbral del cable ata desde el §684. Los de arriba —claves de
// un elemento, rellenadas con ceros— dan la misma identidad que antes (§683) y
// no ejercitan nada nuevo.
// ============================================================================

/// Las claves anchas de los cinco custodios: las de [`custodian_keys`], abiertas
/// a cuatro elementos con [`wide_key`].
pub fn custodian_keys_wide() -> Vec<Digest> {
    custodian_keys().iter().map(|k| wide_key(k.as_int())).collect()
}

/// Las claves anchas de los cuatro miembros de la gobernanza, por el mismo molde.
pub fn governance_keys_wide() -> Vec<Digest> {
    governance_keys().iter().map(|k| wide_key(k.as_int())).collect()
}

/// La raíz del conjunto de custodios constituido con [`custodian_keys_wide`].
pub fn custodian_root_wide() -> Digest {
    stark_experiment::circuit_threshold::build_custodian_set_wide(&custodian_keys_wide()).0
}

/// La raíz del conjunto de gobernanza constituido con [`governance_keys_wide`].
pub fn governance_root_wide() -> Digest {
    stark_experiment::circuit_governance::build_governance_set_wide(&governance_keys_wide()).0
}

/// Una capa constituida con los dos conjuntos anchos y los topes de la suite.
pub fn new_layer_wide() -> SovereignLayer {
    SovereignLayer::new(
        custodian_root_wide(),
        governance_root_wide(),
        LIMIT,
        MAX_SUPPLY,
        MAX_ACCOUNTS,
    )
}

/// Una autorización de umbral con una clave de cuatro elementos (§684), sobre el
/// camino que se le dé: la prueba y sus entradas públicas. La clave y el camino
/// van sueltos a propósito, para que un falsador pueda firmar con una clave que no
/// es la de la hoja.
pub fn autorizacion_ancha(
    dominio: u64,
    key: Digest,
    path: &stark_experiment::circuit_threshold::CustodianPath,
    op: Digest,
) -> (winterfell::Proof, auth::NullifierThresholdPublicInputs) {
    let prover = auth::NullifierThresholdProver::new(proof_options());
    let traza = auth::build_trace_wide(BaseElement::new(dominio), key, path, op);
    let entradas = prover.get_pub_inputs(&traza);
    (prover.prove(traza).expect("autorizacion con clave ancha"), entradas)
}

/// El par de custodios `a` y `b` de un conjunto de claves ANCHAS: el gemelo de
/// [`custodian_pair_with`].
pub fn custodian_pair_wide_with(
    keys: &[Digest],
    op: Digest,
    a: usize,
    b: usize,
) -> (
    winterfell::Proof,
    auth::NullifierThresholdPublicInputs,
    winterfell::Proof,
    auth::NullifierThresholdPublicInputs,
) {
    assert!(a < b, "§51: index_a < index_b, estricto");
    let (_, cp) = stark_experiment::circuit_threshold::build_custodian_set_wide(keys);
    let (pa, ia) = autorizacion_ancha(CUSTODIAN_DOMAIN, keys[a], &cp[a], op);
    let (pb, ib) = autorizacion_ancha(CUSTODIAN_DOMAIN, keys[b], &cp[b], op);
    (pa, ia, pb, ib)
}

/// El par de miembros `a` y `b` de una gobernanza de claves ANCHAS: el gemelo de
/// [`governance_pair`].
pub fn governance_pair_wide_with(
    keys: &[Digest],
    op: Digest,
    a: usize,
    b: usize,
) -> (
    winterfell::Proof,
    auth::NullifierThresholdPublicInputs,
    winterfell::Proof,
    auth::NullifierThresholdPublicInputs,
) {
    use stark_experiment::circuit_governance::{build_governance_set_wide, GOVERNANCE_DOMAIN};
    assert!(a < b, "§51: index_a < index_b, estricto");
    let (_, gp) = build_governance_set_wide(keys);
    let (pa, ia) = autorizacion_ancha(GOVERNANCE_DOMAIN, keys[a], &gp[a], op);
    let (pb, ib) = autorizacion_ancha(GOVERNANCE_DOMAIN, keys[b], &gp[b], op);
    (pa, ia, pb, ib)
}

// ============================================================================
// §704: las pruebas del montaje, generadas una vez y compartidas.
//
// Con la máquina cargada, la fila de la capa del canon no cabía en los 600 s de su timeout (§704,
// «Medido»), y una parte de su CPU era montaje repetido: en los fuentes de `2ac6db4`, 78 de las
// 194 llamadas a `new_layer()` llevan en la línea siguiente el fondeo
// `open_and_fund(&mut <capa>, SK_ALICE, 1_000_000)` de una capa recién creada, que es el MISMO
// enunciado en todas (la orden que lo cuenta, en el §704), y cada uno volvía a probar su subida y
// sus dos autorizaciones de custodio.
//
// Los cinco montajes delegados —`fund_delegated`, `set_frozen_delegated`,
// `update_custodians_delegated`, `recover_delegated` y `mint_to_pending_delegated`— piden sus
// pruebas dentro de [`de_montaje`], y ahí [`ProbarDeMontaje::probar`] las guarda por proceso
// con la huella de lo que se prueba: el tipo del probador, sus opciones y la traza entera,
// celda a celda, con su meta. La misma huella es el mismo enunciado con el mismo testigo, así que
// la prueba guardada es una prueba válida de lo que se pide, y la capa la verifica entera en
// cada test, como antes. Lo que se comparte es la generación, no la comprobación.
//
// Fuera de `de_montaje` nada cambia: los ayudantes públicos (`mint_climb_proof`,
// `delegated_pair`, ...) prueban de nuevo en cada llamada, que es lo que necesitan los tests que
// los llaman a mano —los que miden, los que repiten una prueba a propósito y los que fabrican una
// mala—. Y sólo en los tests de esta capa (`cfg(test)`): con la feature `sandbox`, que es como
// usan este módulo el nodo, el cli y los bancos, `probar` es `prove` y `de_montaje` no hace nada.
// Lo vigila `el_montaje_comparte_sus_pruebas_y_nada_mas`, en `tests.rs`.
// ============================================================================

#[cfg(test)]
thread_local! {
    static EN_MONTAJE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Corre `f` con las pruebas de montaje compartidas (ver la cabecera de esta sección). Al salir,
/// aunque `f` entre en pánico, el hilo vuelve a como estaba.
pub fn de_montaje<T>(f: impl FnOnce() -> T) -> T {
    #[cfg(test)]
    {
        struct Vuelve(bool);
        impl Drop for Vuelve {
            fn drop(&mut self) {
                EN_MONTAJE.with(|m| m.set(self.0));
            }
        }
        let _vuelve = Vuelve(EN_MONTAJE.with(|m| m.replace(true)));
        f()
    }
    #[cfg(not(test))]
    {
        f()
    }
}

/// La huella de lo que se prueba: el tipo del probador, sus opciones y la traza entera, con sus
/// dimensiones y su meta, bajo BLAKE3.
#[cfg(test)]
fn huella_de_montaje<P>(probador: &P, traza: &winterfell::TraceTable<BaseElement>) -> [u8; 32]
where
    P: Prover<BaseField = BaseElement, Trace = winterfell::TraceTable<BaseElement>>,
{
    use winterfell::crypto::{Digest as _, Hasher};
    use winterfell::Trace;
    let mut b: Vec<u8> = Vec::new();
    for parte in [std::any::type_name::<P>().to_string(), format!("{:?}", probador.options())] {
        b.extend_from_slice(&(parte.len() as u64).to_le_bytes());
        b.extend_from_slice(parte.as_bytes());
    }
    let meta = traza.info().meta();
    b.extend_from_slice(&(meta.len() as u64).to_le_bytes());
    b.extend_from_slice(meta);
    b.extend_from_slice(&(traza.width() as u64).to_le_bytes());
    b.extend_from_slice(&(traza.length() as u64).to_le_bytes());
    for c in 0..traza.width() {
        for e in traza.get_column(c) {
            b.extend_from_slice(&e.as_int().to_le_bytes());
        }
    }
    Blake3::hash(&b).as_bytes()
}

/// `prove`, y dentro de [`de_montaje`] una sola vez por proceso para cada huella.
///
/// La huella lee del probador su tipo y sus opciones, y nada más: sólo vale para un probador cuyo
/// único estado son sus `ProofOptions`. Por eso se implementa para los cinco del montaje, que sólo
/// llevan `options`, y no para cualquier `Prover`; y la constante de debajo deja de compilar si a
/// alguno le crece otro campo. Un probador con más estado, que moviera sus entradas públicas o su
/// prueba, recibiría dentro del montaje la prueba guardada de otro enunciado.
pub trait ProbarDeMontaje:
    Prover<BaseField = BaseElement, Trace = winterfell::TraceTable<BaseElement>> + Sized
{
    /// La prueba de `traza`. Fuera del montaje, o fuera de los tests de la capa, es `prove`.
    fn probar(
        &self,
        traza: winterfell::TraceTable<BaseElement>,
    ) -> Result<winterfell::Proof, winterfell::ProverError>
    where
        <Self::Air as winterfell::Air>::PublicInputs: Send,
    {
        #[cfg(test)]
        {
            use std::collections::HashMap;
            use std::sync::{Mutex, OnceLock};
            static GUARDADAS: OnceLock<Mutex<HashMap<[u8; 32], winterfell::Proof>>> =
                OnceLock::new();
            if EN_MONTAJE.with(|m| m.get()) {
                let huella = huella_de_montaje(self, &traza);
                let guardadas = GUARDADAS.get_or_init(Default::default);
                if let Some(p) = guardadas.lock().expect("montaje").get(&huella) {
                    return Ok(p.clone());
                }
                let p = self.prove(traza)?;
                guardadas.lock().expect("montaje").insert(huella, p.clone());
                return Ok(p);
            }
        }
        self.prove(traza)
    }
}

impl ProbarDeMontaje for auth::NullifierThresholdProver {}
impl ProbarDeMontaje for climb::MintClimbProver {}
impl ProbarDeMontaje for climb_frozen::FrozenClimbProver {}
impl ProbarDeMontaje for climb_recovery::RecoveryClimbProver {}
impl ProbarDeMontaje for climb_pending::MintPendingClimbProver {}

// Los cinco no llevan más estado que sus opciones: si a uno le crece un campo, esto no compila.
const _: () = {
    use std::mem::size_of;
    let o = size_of::<ProofOptions>();
    assert!(
        size_of::<auth::NullifierThresholdProver>() == o
            && size_of::<climb::MintClimbProver>() == o
            && size_of::<climb_frozen::FrozenClimbProver>() == o
            && size_of::<climb_recovery::RecoveryClimbProver>() == o
            && size_of::<climb_pending::MintPendingClimbProver>() == o,
        "un probador del montaje lleva mas estado que sus ProofOptions"
    );
};
