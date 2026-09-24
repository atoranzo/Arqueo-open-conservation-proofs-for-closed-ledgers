//! Los tres subcomandos. Toda la salida de usuario pasa por el `Tracer`,
//! así `--json` produce JSON Lines puro sin tocar la lógica.

use std::path::PathBuf;

use clap::Args;
use zk_ssl::log::OpKind;
use zk_ssl::SovereignLayer;
use zk_ssl_sdk::{keystore, Wallet};

use crate::fmt::{hex_short, Digest};
use crate::sandbox::{self, Params};
use crate::trace::{TraceEvent, Tracer};

/// Parámetros de la capa, compartidos por los tres subcomandos.
/// Por defecto, los de la suite del proyecto.
#[derive(Args, Clone, Copy)]
pub struct LayerParams {
    /// Límite regulatorio por operación.
    #[arg(long, default_value_t = Params::default().regulatory_limit)]
    limit: u64,
    /// Tope de suministro.
    #[arg(long, default_value_t = Params::default().max_supply)]
    max_supply: u64,
    /// Máximo de cuentas.
    #[arg(long, default_value_t = Params::default().max_accounts)]
    max_accounts: u64,
}

impl From<LayerParams> for Params {
    fn from(p: LayerParams) -> Self {
        Params {
            regulatory_limit: p.limit,
            max_supply: p.max_supply,
            max_accounts: p.max_accounts,
        }
    }
}

// ───────────────────────────── simulate ─────────────────────────────

#[derive(Args)]
pub struct SimulateArgs {
    /// Ruta de un ledger persistido (sled). Sin ella, todo va en memoria.
    #[arg(long)]
    ledger: Option<String>,

    /// Índice de la cuenta pagadora.
    #[arg(long, default_value_t = 0)]
    from: u64,
    /// Índice de la cuenta receptora.
    #[arg(long, default_value_t = 1)]
    to: u64,
    /// Importe del envío.
    #[arg(long, default_value_t = 250_000)]
    amount: u64,

    /// Cuentas mínimas del escenario (se abren y fondean las que falten).
    #[arg(long, default_value_t = 2)]
    accounts: u64,
    /// Fondos iniciales por cuenta nueva (emisión delegada, dos custodios).
    #[arg(long, default_value_t = 1_000_000)]
    fund: u64,

    /// Deja el envío SIN cobrar, para ver el pendiente en tránsito.
    #[arg(long)]
    no_claim: bool,

    /// Envía por la VÍA v2 (RFC-0003): el aviso lleva el sobre `X` opaco, que es lo que la
    /// prueba del cobro (RFC-0008 E1) necesita. La pareja `(f, delta)` es la de la
    /// conformidad: `f` = la identidad del emisor, `delta` = 96.
    #[arg(long)]
    v2: bool,
    /// Escribe el aviso v2 en un fichero PROPIO del cliente (posición, sal, importe, `x`).
    #[arg(long, requires = "v2")]
    aviso: Option<String>,
    /// Escribe la credencial del receptor (`index`, `publicId`, `viewKey`) en un fichero
    /// propio, aparte del aviso: dos ficheros, dos dueños (RFC-0008 D-P).
    #[arg(long, requires = "v2")]
    credencial: Option<String>,

    /// Escribe el RETORNO del pagador (`refundId`, `delta`) en su propio fichero: la
    /// pareja del sobre `X`, que ni el aviso ni la credencial llevan (RFC-0008 D-AI).
    ///
    /// Aquí lo escribe el simulador porque conoce la semilla y la pareja que usa; fuera
    /// del sandbox lo escribe quien envía, o se pierde (reversión 37 del RFC-0008).
    #[arg(long, requires = "v2")]
    retorno: Option<String>,

    /// Escribe la credencial del PAGADOR (`index`, `publicId`, `viewKey`) en su fichero:
    /// la MISMA terna que `--credencial`, pero de quien envía (RFC-0008 D-AI, S508).
    ///
    /// Sin ella nadie puede pedir la foto COMO pagador: `zkssl_pendingPath` exige SU
    /// credencial, no la del receptor (D-AE, S505). Su testigo es `tools/banco_pago.sh`:
    /// si esto escribiera la del receptor, el nodo la aceptaría y la puerta del pagador
    /// serviría la NADA.
    #[arg(long, requires = "v2")]
    credencial_pagador: Option<String>,

    /// Escribe el KEYSTORE del receptor -`zkssl-keystore/1`, el del SDK- cifrado con la frase de
    /// `--frase-fichero`: el QUINTO fichero de la siembra, y el UNICO con material de GASTO.
    ///
    /// Es lo que la boca del prendador exige (`prueba-prenda`, RFC-0008 D-BC), y sale de la MISMA
    /// `key_of(key_seed, to)` que escribe `--credencial` aqui arriba: un solo productor de la
    /// clave. Su testigo es `tools/banco_prenda.sh`: si esto escribiera otra clave, el sobre no
    /// nombraria al receptor del pendiente y la prenda no probaria nada.
    ///
    /// Claves de JUGUETE y deterministas, como todo el sandbox: esto no es el wallet de nadie.
    #[arg(long, requires = "v2")]
    keystore: Option<String>,
    /// El fichero con la frase que cifra `--keystore` (D-BG): nunca en la linea de ordenes. Va
    /// con `--keystore`, y uno sin el otro es un rojo con nombre.
    #[arg(long, requires = "v2")]
    frase_fichero: Option<String>,

    /// Semilla base de las claves deterministas del sandbox.
    #[arg(long, default_value_t = 0xA11CE)]
    key_seed: u64,
    /// Semilla del aleatorio (salt) del pendiente.
    #[arg(long, default_value_t = 7)]
    salt_seed: u64,

    #[command(flatten)]
    params: LayerParams,
}

pub fn simulate(a: SimulateArgs, tr: &mut dyn Tracer) -> anyhow::Result<()> {
    let mut layer = sandbox::open_layer(a.ledger.as_deref(), a.params.into())?;

    tr.emit(&TraceEvent::Note {
        text: format!(
            "⚠️ sandbox con claves DETERMINISTAS (semilla {:#x}); pruebas STARK reales, tarda unos segundos",
            a.key_seed
        ),
    });

    // Montaje: abre y fondea las cuentas que falten.
    // El arbol de cuentas es DISPERSO: open_account_wide devuelve la
    // POSICION DERIVADA de la hoja, no un contador. Guardamos el mapeo
    // logico->real de las cuentas abiertas en ESTA corrida; --from/--to
    // son posiciones logicas (0, 1, ...) sobre ese mapeo, o un indice
    // real ya existente (caso --ledger).
    let needed = a.accounts.max(a.from.max(a.to) + 1);
    let mut abiertas: Vec<u64> = Vec::new();
    while (layer.account_count() as u64) < needed {
        let i = abiertas.len() as u64;
        let idx =
            sandbox::open_funded(&mut layer, sandbox::key_of(a.key_seed, i), a.fund, tr)?;
        abiertas.push(idx);
    }
    let from_real = match abiertas.get(a.from as usize) {
        Some(&ix) => ix,
        None if layer.public_id_of(a.from).is_some() => a.from,
        None => anyhow::bail!(
            "la cuenta logica #{} no se abrio en esta corrida y tampoco es un \
             indice real del arbol (disperso: posiciones derivadas)",
            a.from
        ),
    };
    let to_real = match abiertas.get(a.to as usize) {
        Some(&ix) => ix,
        None if layer.public_id_of(a.to).is_some() => a.to,
        None => anyhow::bail!(
            "la cuenta logica #{} no se abrio en esta corrida y tampoco es un \
             indice real del arbol (disperso: posiciones derivadas)",
            a.to
        ),
    };

    // FASE 1 — enviar, por la vía v1 o por la v2 (RFC-0008 D-M: el sandbox aprende la v2).
    let envio = if a.v2 {
        let f = layer
            .public_id_of(from_real)
            .ok_or_else(|| anyhow::anyhow!("la cuenta emisora #{from_real} no existe"))?;
        // La pareja del sobre se liga UNA vez: el envío y el fichero del retorno salen
        // de la MISMA variable, no de dos literales iguales (RFC-0008 D-AI).
        let delta = 96;
        let recibo = sandbox::run_send_v2(
            &mut layer,
            from_real,
            sandbox::key_of(a.key_seed, a.from),
            to_real,
            a.amount,
            a.salt_seed,
            f,
            delta,
            tr,
        )?;
        if let Some(ruta) = &a.retorno {
            crate::cobro::escribir(ruta, &crate::pago::retorno_de(f, delta))?;
            tr.emit(&TraceEvent::Note {
                text: format!("retorno del pagador #{from_real} escrito en {ruta}"),
            });
        }
        recibo
    } else {
        sandbox::run_send(
            &mut layer,
            from_real,
            sandbox::key_of(a.key_seed, a.from),
            to_real,
            a.amount,
            a.salt_seed,
            tr,
        )?
    };
    // Lo que el cobrador se lleva a su máquina: el aviso (del pagador) y su credencial (suya).
    if let Some(ruta) = &a.aviso {
        let av = crate::cobro::aviso_de(&envio.notice).map_err(|e| anyhow::anyhow!("{e}"))?;
        crate::cobro::escribir(ruta, &av)?;
        tr.emit(&TraceEvent::Note { text: format!("aviso v2 escrito en {ruta}") });
    }
    if let Some(ruta) = &a.credencial {
        let c = crate::cobro::credencial_de(sandbox::key_of(a.key_seed, a.to), to_real);
        crate::cobro::escribir(ruta, &c)?;
        tr.emit(&TraceEvent::Note {
            text: format!("credencial del receptor #{to_real} escrita en {ruta}"),
        });
    }
    // La del PAGADOR es la MISMA derivación con SU clave y SU índice (S508): la capa la
    // acepta para ese índice y rechaza la ajena, medido en el PASTE-E2g-M y con testigo
    // en `pago.rs`. Va aparte porque es de otro dueño, como el aviso y la credencial.
    if let Some(ruta) = &a.credencial_pagador {
        let c = crate::cobro::credencial_de(sandbox::key_of(a.key_seed, a.from), from_real);
        crate::cobro::escribir(ruta, &c)?;
        tr.emit(&TraceEvent::Note {
            text: format!("credencial del pagador #{from_real} escrita en {ruta}"),
        });
    }

    // EL QUINTO FICHERO (S544): el keystore del RECEPTOR, que es quien prenda. La clave es la
    // MISMA `key_of(key_seed, to)` de su credencial de arriba, y la boca no acepta otra cosa: la
    // de gasto entra por el keystore del SDK y el cli nunca la ve (D-BC).
    if let Some(ruta) = keystore_del_sandbox(
        a.keystore.as_ref(),
        a.frase_fichero.as_ref(),
        sandbox::key_of(a.key_seed, a.to),
    )? {
        tr.emit(&TraceEvent::Note {
            text: format!("keystore del receptor #{to_real} escrito en {ruta} (modo 600)"),
        });
    }

    // FASE 2 — cobrar (salvo que se pida ver el pendiente en tránsito).
    if a.no_claim {
        tr.emit(&TraceEvent::Note {
            text: "envío SIN cobrar: el importe queda inmovilizado hasta claim o refund (§29/§30)"
                .into(),
        });
    } else {
        sandbox::run_claim(
            &mut layer,
            to_real,
            sandbox::key_of(a.key_seed, a.to),
            &envio.notice,
            tr,
        )?;
    }

    verify_chain(&layer, tr);
    for idx in [from_real, to_real] {
        if let Some(balance) = layer.balance_of(idx) {
            tr.emit(&TraceEvent::Note {
                text: format!("saldo final #{idx}: {balance} (vista del operador)"),
            });
        }
    }
    sandbox::emit_summary(&layer, tr);
    Ok(())
}

/// El QUINTO fichero de la siembra (S544): el keystore del RECEPTOR, que es quien prenda.
///
/// **Un solo productor de la clave.** La `clave` que recibe es la misma
/// `sandbox::key_of(key_seed, to)` con la que se escribio su credencial; aqui no se vuelve a
/// derivar nada. El sello lo pone `zk_ssl_sdk::keystore` -la ley de reposo del proyecto, con
/// dominio propio- y la frase entra por FICHERO, con el MISMO lector que la boca
/// (`crate::prenda::leer_frase`, D-BG).
///
/// Los dos argumentos van juntos o no van: uno sin el otro no escribe nada y se para por su
/// nombre. Devuelve la ruta escrita, o `None` si no se pidio.
///
/// OJO: es el unico fichero de la siembra con material de GASTO. Se escribe con modo 600, y la
/// regla del proyecto es que eso se comprueba al LEER (`zk_ssl_guardian::semilla`): el lector del
/// SDK todavia no lo hace, y eso va a la cola, no aqui.
pub fn keystore_del_sandbox(
    ruta: Option<&String>,
    frase_ruta: Option<&String>,
    clave: Digest,
) -> anyhow::Result<Option<String>> {
    let (ruta, frase_ruta) = match (ruta, frase_ruta) {
        (None, None) => return Ok(None),
        (Some(k), Some(f)) => (k, f),
        _ => anyhow::bail!(
            "--keystore y --frase-fichero van juntos: uno sin el otro no escribe el keystore"
        ),
    };
    let frase = crate::prenda::leer_frase(&PathBuf::from(frase_ruta))?;
    let w = Wallet::from_elements(clave.map(|e| e.as_int()));
    keystore::save(&PathBuf::from(ruta), &w, &frase)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(ruta, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(Some(ruta.clone()))
}

// ───────────────────────────── trace-tx ─────────────────────────────

#[derive(Args)]
pub struct TraceTxArgs {
    /// Ruta de un ledger persistido. Sin ella se genera un escenario de
    /// demostración en memoria y se traza SU registro.
    #[arg(long)]
    ledger: Option<String>,

    /// Número de secuencia de la operación a detallar.
    #[arg(long)]
    seq: Option<u64>,

    /// Sin --seq: cuántas entradas recientes mostrar.
    #[arg(long, default_value_t = 10)]
    last: usize,

    #[command(flatten)]
    params: LayerParams,
}

pub fn trace_tx(a: TraceTxArgs, tr: &mut dyn Tracer) -> anyhow::Result<()> {
    let layer = layer_or_demo(a.ledger.as_deref(), a.params.into(), tr)?;
    let entries = layer.transition_log().entries();

    match a.seq {
        Some(seq) => {
            let e = entries
                .iter()
                .find(|e| e.seq == seq)
                .ok_or_else(|| anyhow::anyhow!("no hay entrada con seq {seq}"))?;
            emit_entry(e, tr);
        }
        None => {
            let skip = entries.len().saturating_sub(a.last);
            for e in &entries[skip..] {
                emit_entry(e, tr);
            }
        }
    }

    verify_chain(&layer, tr);
    Ok(())
}

fn emit_entry(e: &zk_ssl::log::LogEntry, tr: &mut dyn Tracer) {
    tr.emit(&TraceEvent::LogEntry {
        seq: e.seq,
        op: format!("{:?}", e.kind),
        circuit_hint: circuit_hint(e.kind).into(),
        root_old: hex_short(&e.root_old),
        root_new: hex_short(&e.root_new),
        proof_digest: hex_short(&e.proof_digest),
        chain: hex_short(&e.chain),
    });
}

/// Qué verifica cada entrada. Orientativo: la fuente de verdad es
/// `OpKind` (zk-ssl/src/log.rs) y los circuitos de `stark-experiment`.
fn circuit_hint(k: OpKind) -> &'static str {
    match k {
        OpKind::OpenAccount => "sin prueba: apertura a CERO",
        OpKind::Mint => "mint_climb + 2× threshold_single_nullifier",
        OpKind::Transfer => "transferencia (vía histórica)",
        OpKind::Burn => "circuit_burn",
        OpKind::Recovery => "recovery_climb",
        OpKind::Governance => "circuit_governance",
        OpKind::Freeze => "frozen_climb",
        OpKind::Send => "circuit_send (fase 1)",
        OpKind::Claim => "circuit_claim (fase 2)",
        OpKind::MintToPending => "mint_pending_climb",
        OpKind::Migration => "sin prueba: compromiso replicable",
        OpKind::Refund => "reembolso de pendiente caducado",
        OpKind::Consumo => "sin prueba: consumo publicado (RFC-0006 E1)",
    }
}

// ─────────────────────────── inspect-state ──────────────────────────

#[derive(Args)]
pub struct InspectStateArgs {
    /// Ruta de un ledger persistido. Sin ella se inspecciona un escenario
    /// de demostración en memoria.
    #[arg(long)]
    ledger: Option<String>,

    /// Lista además las cuentas (índice, id público, saldo, nonce).
    #[arg(long)]
    accounts: bool,

    #[command(flatten)]
    params: LayerParams,
}

pub fn inspect_state(a: InspectStateArgs, tr: &mut dyn Tracer) -> anyhow::Result<()> {
    let layer = layer_or_demo(a.ledger.as_deref(), a.params.into(), tr)?;

    sandbox::emit_summary(&layer, tr);

    if a.accounts {
        tr.emit(&TraceEvent::Note {
            text: "⚠️ saldos y nonces: vista del OPERADOR por diseño (§129); el titular usa account_view_authenticated".into(),
        });
        for idx in 0..layer.account_count() as u64 {
            if let (Some(id), Some(balance), Some(nonce)) = (
                layer.public_id_of(idx),
                layer.balance_of(idx),
                layer.nonce_of(idx),
            ) {
                tr.emit(&TraceEvent::AccountRow {
                    index: idx,
                    public_id: hex_short(&id),
                    balance,
                    nonce: format!("{nonce}"),
                });
            }
        }
    }

    verify_chain(&layer, tr);
    Ok(())
}

// ───────────────────────────── comunes ──────────────────────────────

/// Abre el ledger indicado o, sin él, monta el escenario de demostración:
/// dos cuentas fondeadas y una transferencia completa en dos fases.
fn layer_or_demo(
    ledger: Option<&str>,
    p: Params,
    tr: &mut dyn Tracer,
) -> anyhow::Result<SovereignLayer> {
    if let Some(path) = ledger {
        return sandbox::open_layer(Some(path), p);
    }
    tr.emit(&TraceEvent::Note {
        text: "sin --ledger: generando escenario de demostración en memoria…".into(),
    });
    let mut layer = sandbox::open_layer(None, p)?;
    let seed = 0xA11CE;
    let a0 = sandbox::open_funded(&mut layer, sandbox::key_of(seed, 0), 1_000_000, tr)?;
    let a1 = sandbox::open_funded(&mut layer, sandbox::key_of(seed, 1), 1_000_000, tr)?;
    let envio = sandbox::run_send(
        &mut layer,
        a0,
        sandbox::key_of(seed, 0),
        a1,
        250_000,
        7,
        tr,
    )?;
    sandbox::run_claim(&mut layer, a1, sandbox::key_of(seed, 1), &envio.notice, tr)?;
    Ok(layer)
}

fn verify_chain(layer: &SovereignLayer, tr: &mut dyn Tracer) {
    let log = layer.transition_log();
    match log.verify_chain() {
        Ok(()) => tr.emit(&TraceEvent::ChainVerified {
            ok: true,
            entries: log.len(),
            error: None,
        }),
        Err(e) => tr.emit(&TraceEvent::ChainVerified {
            ok: false,
            entries: log.len(),
            error: Some(format!("{e:?}")),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zk_ssl_guardian::semilla::comprobar_permisos;

    /// El nombre lleva el pid Y el nombre de quien llama: `cargo test` corre los cuatro testigos
    /// EN PARALELO, y con rutas compartidas se pisan entre ellos -uno borra la frase que otro
    /// acaba de escribir-. Es la carrera que cazo el ENSAYO-544 r1, y se cierra POR INSTANCIA,
    /// como el S457 cerro la suya. El pid separa dos corridas; `quien`, dos testigos de la misma.
    fn temporales(quien: &str) -> (PathBuf, PathBuf, PathBuf) {
        let (d, id) = (std::env::temp_dir(), std::process::id());
        (
            d.join(format!("m544-{quien}-ks-{id}.json")),
            d.join(format!("m544-{quien}-frase-{id}.txt")),
            d.join(format!("m544-{quien}-otra-{id}.txt")),
        )
    }

    fn escribe(ks: &PathBuf, frase: &PathBuf) -> anyhow::Result<Option<String>> {
        keystore_del_sandbox(
            Some(&ks.display().to_string()),
            Some(&frase.display().to_string()),
            sandbox::key_of(0xA11CE, 1),
        )
    }

    /// **UN SOLO PRODUCTOR.** El keystore que escribe el sandbox lleva la clave del RECEPTOR, y se
    /// prueba contra el otro productor que ya existia: la credencial. Si el keystore saliera de
    /// otra derivacion, estos dos `publicId` no coincidirian.
    #[test]
    fn el_keystore_del_sandbox_es_el_del_receptor_de_su_credencial() {
        let (ks, frase, _) = temporales("productor");
        std::fs::write(&frase, "la frase del sandbox\n").unwrap();
        let ruta = escribe(&ks, &frase).expect("escribe el keystore").expect("devuelve su ruta");
        assert_eq!(ruta, ks.display().to_string());
        let w = crate::prenda::wallet_de(&ks, &frase).expect("abre con su frase");
        let cred = crate::cobro::credencial_de(sandbox::key_of(0xA11CE, 1), 7);
        assert_eq!(
            serde_json::to_string(&zk_ssl_wire::digest_to_wire(&w.public_id())).unwrap(),
            serde_json::to_string(&cred.public_id).unwrap(),
            "el keystore no lleva la clave de la credencial: hay DOS productores"
        );
        let _ = (std::fs::remove_file(&ks), std::fs::remove_file(&frase));
    }

    #[test]
    fn otra_frase_no_abre_el_keystore_del_sandbox() {
        let (ks, frase, otra) = temporales("frase");
        std::fs::write(&frase, "la frase del sandbox\n").unwrap();
        std::fs::write(&otra, "otra frase\n").unwrap();
        escribe(&ks, &frase).unwrap();
        assert!(crate::prenda::wallet_de(&ks, &otra).is_err(), "abrio con otra frase");
        let _ = (
            std::fs::remove_file(&ks),
            std::fs::remove_file(&frase),
            std::fs::remove_file(&otra),
        );
    }

    /// El unico fichero de la siembra con material de gasto no puede quedar legible por el grupo
    /// ni por otros. El juez es el del proyecto, no uno propio.
    #[test]
    fn el_keystore_del_sandbox_se_escribe_cerrado() {
        let (ks, frase, _) = temporales("cerrado");
        std::fs::write(&frase, "la frase del sandbox\n").unwrap();
        escribe(&ks, &frase).unwrap();
        assert!(comprobar_permisos(&ks).is_ok(), "el keystore quedo abierto a grupo u otros");
        let _ = (std::fs::remove_file(&ks), std::fs::remove_file(&frase));
    }

    /// Uno sin el otro no escribe NADA y se para por su nombre: ni un keystore sin frase, ni una
    /// frase sin keystore.
    #[test]
    fn el_keystore_y_su_frase_van_juntos_o_no_van() {
        let (ks, frase, _) = temporales("juntos");
        std::fs::write(&frase, "la frase del sandbox\n").unwrap();
        let clave = sandbox::key_of(0xA11CE, 1);
        assert!(keystore_del_sandbox(None, None, clave).unwrap().is_none());
        let solo_ks = keystore_del_sandbox(Some(&ks.display().to_string()), None, clave);
        let solo_frase = keystore_del_sandbox(None, Some(&frase.display().to_string()), clave);
        for (r, q) in [(solo_ks, "keystore sin frase"), (solo_frase, "frase sin keystore")] {
            let e = r.expect_err(q).to_string();
            assert!(e.contains("van juntos"), "{q}: se paro por otra regla: {e}");
        }
        assert!(!ks.exists(), "se escribio un keystore sin frase");
        let _ = std::fs::remove_file(&frase);
    }
}
