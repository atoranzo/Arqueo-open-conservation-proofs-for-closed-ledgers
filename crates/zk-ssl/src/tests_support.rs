//! Ayudantes compartidos por los tests de los distintos módulos.
//!
//! Viven aquí y no en `tests.rs` porque el puente ISO también los
//! necesita, y duplicarlos haría que dos suites divergieran en silencio.

use super::*;

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
pub fn open_and_fund(layer: &mut SovereignLayer, sk: u64, amount: u64) -> AccountIndex {
    let idx = layer.open_account(BaseElement::new(sk));
    if amount > 0 {
        let receipt = layer
            .mint(&valid_auth(), idx, amount)
            .expect("la emision autorizada deberia generar prueba");
        layer.apply_mint(&receipt, idx).expect("aplicar emision");
    }
    idx
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
