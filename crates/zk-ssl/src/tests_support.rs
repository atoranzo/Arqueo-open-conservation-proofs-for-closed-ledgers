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
// El `allow` de abajo ya no ampara al fondeo: ampara `open_account`
// (64 bits, opt-in por §97.4, fuera de B por §160) y lo que aun
// ejercita la via antigua a proposito hasta B-2/B-3.
//
// §65.3: el permiso va en los tests, no en la definicion.
#![allow(deprecated)]

use super::*;
use stark_experiment::circuit_mint_climb as climb;
use stark_experiment::circuit_frozen_climb as climb_frozen;
use stark_experiment::circuit_recovery_climb as climb_recovery;
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

/// Autorización de gobernanza válida. Índices 1 y 3: el 0 tiene todos
/// los bits de camino a cero y degeneraría la traza.
pub fn valid_governance_auth() -> GovernanceAuth {
    let keys = governance_keys();
    let (_, paths) = build_governance_set(&keys);
    GovernanceAuth {
        key_a: keys[1],
        index_a: 1,
        path_a: paths[1].clone(),
        key_b: keys[3],
        index_b: 3,
        path_b: paths[3].clone(),
    }
}

pub fn custodian_root() -> Digest {
    stark_experiment::circuit_threshold::build_custodian_set(&custodian_keys()).0
}

/// Autorización válida: custodios 1 y 3, en orden estricto.
pub fn valid_auth() -> ThresholdAuth {
    let keys = custodian_keys();
    let (_, paths) = stark_experiment::circuit_threshold::build_custodian_set(&keys);
    ThresholdAuth {
        key_a: keys[1],
        index_a: 1,
        path_a: paths[1].clone(),
        key_b: keys[3],
        index_b: 3,
        path_b: paths[3].clone(),
    }
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
    let pa = prover.prove(ta).expect("autorizacion A");
    let tb = auth::build_trace(d, ck[b], &cp[b], op);
    let ib = prover.get_pub_inputs(&tb);
    let pb = prover.prove(tb).expect("autorizacion B");
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
        .prove(trace)
        .expect("subida")
}

/// Emite `amount` en `idx` por la VÍA DELEGADA: custodios 1 y 3
/// autorizan sin que sus claves toquen el operador.
pub fn fund_delegated(layer: &mut SovereignLayer, idx: AccountIndex, amount: u64) {
    let op = mint_commitment(layer, idx, amount);
    let subida = mint_climb_proof(layer, idx, amount);
    let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
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
        .prove(trace)
        .expect("subida de congelacion")
}

/// Congela o descongela `idx` por la VÍA DELEGADA: custodios 1 y 3.
pub fn set_frozen_delegated(layer: &mut SovereignLayer, idx: AccountIndex, frozen: bool) {
    let op = freeze_commitment(layer, idx, frozen);
    let subida = freeze_climb_proof(layer, idx, frozen);
    let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
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
    let pa = prover.prove(ta).expect("autorizacion A");
    let tb = auth::build_trace(d, gk[b], &gp[b], op);
    let ib = prover.get_pub_inputs(&tb);
    let pb = prover.prove(tb).expect("autorizacion B");
    (pa, ia, pb, ib)
}

/// Cambia el conjunto de custodios por la VÍA DELEGADA: miembros 1 y 3.
pub fn update_custodians_delegated(layer: &mut SovereignLayer, nueva: Digest) {
    let op = governance_commitment(layer, nueva);
    let (pa, ia, pb, ib) = governance_pair(op, 1, 3);
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
        .prove(trace)
        .expect("subida de recuperacion")
}

/// Recupera `idx` hacia la identidad `nueva` por la VÍA DELEGADA.
pub fn recover_delegated(layer: &mut SovereignLayer, idx: AccountIndex, nueva: Digest) {
    let op = recovery_commitment(layer, idx, nueva);
    let subida = recovery_climb_proof(layer, idx, nueva);
    let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
    layer
        .apply_recovery_delegated(subida, pa, ia, pb, ib, idx, nueva)
        .expect("la recuperacion delegada legitima debe aplicarse");
}

/// [`open_and_fund`], por la vía delegada. Misma firma y mismo abridor
/// estrecho: B migra la custodia, no la apertura (§160, §90).
pub fn open_and_fund_delegated(layer: &mut SovereignLayer, sk: u64, amount: u64) -> AccountIndex {
    let idx = layer.open_account(BaseElement::new(sk));
    if amount > 0 {
        fund_delegated(layer, idx, amount);
    }
    idx
}

/// [`open_and_fund_wide`], por la vía delegada.
pub fn open_and_fund_wide_delegated(
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

#[cfg(test)]
mod la_palanca {
    use super::*;

    /// La delegada fondea IGUAL que la vieja: misma identidad, mismo
    /// saldo, mismo nonce, mismo suministro y misma colocación. Si esto
    /// rompe, las dos vías divergen — hallazgo, no accidente.
    #[test]
    fn open_and_fund_delegated_matches_the_old_road() {
        let mut vieja = new_layer();
        let mut nueva = new_layer();
        let a = vieja.open_account(BaseElement::new(SK_ALICE));
        let r = vieja.mint(&valid_auth(), a, 250_000).expect("emision vieja");
        vieja.apply_mint(&r, a).expect("aplicar");
        let b = open_and_fund_delegated(&mut nueva, SK_ALICE, 250_000);
        assert_eq!(a, b, "misma colocacion pid-mod");
        let (sv, sn) = (state_of(&vieja, a), state_of(&nueva, b));
        assert_eq!(sv.public_id, sn.public_id, "misma identidad");
        assert_eq!(sv.balance, sn.balance, "mismo saldo");
        assert_eq!(sv.nonce, sn.nonce, "mismo nonce");
        assert_eq!(vieja.total_supply(), nueva.total_supply(), "mismo suministro");
    }

    /// La anchura ancha, por la misma puerta.
    #[test]
    fn the_wide_road_also_matches() {
        let mut vieja = new_layer();
        let mut nueva = new_layer();
        let a = vieja.open_account_wide(wide_key(SK_BOB));
        let r = vieja.mint(&valid_auth(), a, 77_000).expect("emision vieja");
        vieja.apply_mint(&r, a).expect("aplicar");
        let b = open_and_fund_wide_delegated(&mut nueva, wide_key(SK_BOB), 77_000);
        assert_eq!(a, b, "misma colocacion");
        let (sv, sn) = (state_of(&vieja, a), state_of(&nueva, b));
        assert_eq!(sv.public_id, sn.public_id, "misma identidad");
        assert_eq!(sv.balance, sn.balance, "mismo saldo");
        assert_eq!(sv.nonce, sn.nonce, "mismo nonce");
        assert_eq!(vieja.total_supply(), nueva.total_supply(), "mismo suministro");
    }
}
