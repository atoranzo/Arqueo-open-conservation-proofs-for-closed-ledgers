//! **Reverificacion del registro** (§279, nota 79 del BACKLOG).
//!
//! §278 hizo que cada entrada delegada atara el compromiso que la
//! autorizo. Este modulo responde la pregunta siguiente, que es la de la
//! nota 79: **¿puede un tercero, con el registro en la mano y sin el
//! nodo, comprobar esa atadura?**
//!
//! La respuesta honesta es «en parte», y el instrumento la dice entrada
//! por entrada en vez de colapsarla a verde o rojo (§254).
//!
//! ⚠️ **Lo que se puede recomputar depende de si los parametros del
//! compromiso estan en el registro**, y medido sobre el arbol de §278 no
//! lo estan casi nunca:
//!
//! | Via | ¿recomputable con el log solo? |
//! |---|---|
//! | `OpenAccount` | **Si**: su sello es una constante |
//! | `Recovery` | **Si**: raices en la entrada + contador contable |
//! | `Mint` | No: faltan `amount` y el suministro |
//! | `Freeze` | No: el compromiso ata raices del arbol de CONGELADOS |
//! | `Governance` | No: ata raices del conjunto de CUSTODIOS |
//! | `MintToPending` | No: ata raices de PENDIENTES, mas importe |
//!
//! Cerrar el resto exige que el compromiso viaje en la entrada, que es
//! una rotura de formato y por tanto **otro sello** — anotada en la 79.
//!
//! ✅ **Ese sello fue §281, y §282 lee las dos eras.** La tabla de arriba
//! sigue siendo cierta para las entradas anteriores a la rotura.
//!
//! ## Era 2 (desde §281): el compromiso viaja en la entrada
//!
//! §281 rompio el formato del registro y anadio `compromiso` a cada
//! entrada nueva. Con el, las **seis clases con sello** se recomputan:
//!
//! | Via | era 1 (`None`) | era 2 (`Some`) |
//! |---|---|---|
//! | `OpenAccount` | Si (sello constante) | Si (igual) |
//! | `Recovery` | Si, con el log completo | **Si, y basta un TRAMO** |
//! | `Mint` | No | **Si** |
//! | `Freeze` | No | **Si** |
//! | `Governance` | No | **Si** |
//! | `MintToPending` | No | **Si** |
//!
//! ⚠️ **«Seis de seis» son las seis clases CON SELLO**, no todas las
//! entradas. `Send`, `Claim`, `Burn`, `Refund` y `Migration` asientan el
//! resumen de una prueba REAL, y el registro no guarda la prueba: no seran
//! recomputables desde el registro nunca, y eso es diseno, no deuda.
//!
//! ⚠️ **La exigencia de registro completo pasa a ser POR ENTRADA.** En era
//! 1 el contador de recuperaciones se deriva contando las anteriores, asi
//! que hace falta el genesis. En era 2 el compromiso esta en la entrada y
//! **un tramo de `zkssl_logEntries` basta** — que era el caso normal.
//!
//! ## Un compromiso repetido, desde el §716 (el reenvio de una emision delegada)
//!
//! Un compromiso que lleva contador no se repite nunca en un registro honrado: cada entrada
//! consume un valor del contador. Si se repite, la entrada miente, y es ROJO
//! ([`ReverificacionError::CompromisoRepetido`]). Llevan contador desde antes de la era 2
//! `Freeze`, `Governance` y `Recovery`. `Mint` y `MintToPending` lo llevan desde el arreglo del
//! reenvio, y el registro no dice desde que entrada: lo declara quien llama
//! ([`EmisionConContador`]).
//!
//! ⚠️ Sin contador, un compromiso repetido NO es ROJO: puede ser una reautorizacion legitima de
//! la misma emision desde el mismo estado, que da el mismo compromiso. Marcarla ROJO acusaria a
//! un operador honrado. Sale `Parcial`, con lo que falta dicho: un reenvio y una reautorizacion
//! no se distinguen.
//!
//! ⚠️ Solo ve lo que se le da: con un tramo, un compromiso repetido fuera del tramo no se ve.
//!
//! ⚠️ Solo ve un reenvio que se asienta como tal, con el MISMO compromiso. Los pares de los
//! custodios no se guardan (ECST §5.11): un nodo deshonesto puede asentar la emision reenviada
//! con un compromiso nuevo y su sello, y sale `Verificada`. Un compromiso fabricado no se
//! distingue aqui de uno firmado.
//!
//! ⚠️ [`reverificar`] no declara el contador ([`EmisionConContador::No`]): con el, una `Mint` o
//! una `MintToPending` repetidas salen `Parcial`, no ROJO. El ROJO de una emision pide
//! [`reverificar_con`] con la `seq` desde la que el libro lleva contador.

use zk_ssl_hash::{
    commit_operation, digest_of_proof, digest_to_bytes, sello_de_autorizacion, sello_sin_prueba,
    Digest, COMPROMISO_AUSENTE, OP_RECOVERY,
};
use winter_math::fields::f64::BaseElement;

/// Lo que el reverificador necesita de una entrada del registro.
///
/// ⚠️ **Struct propio, como `ReciboInclusion`**: este crate no ve
/// `zk_ssl::log::LogEntry` —no depende de la capa— y quien tenga las
/// entradas (nodo, cable, un tercero con `zkssl_logEntries`) traduce.
#[derive(Clone, Debug)]
pub struct EntradaLog {
    pub seq: u64,
    /// El nombre del tipo tal como lo sirve el cable: `"Recovery"`, etc.
    pub kind: String,
    pub root_old: Digest,
    pub root_new: Digest,
    pub proof_digest: Digest,
    /// **El portador de la era** (§281): `None` = entrada anterior a la
    /// rotura, cuyo compromiso no existe en ningun sitio; `Some` = era 2.
    /// El centinela `COMPROMISO_AUSENTE` marca las clases sin compromiso
    /// propio, y no es lo mismo que no tenerlo.
    pub compromiso: Option<Digest>,
}

/// Lo que se pudo comprobar de UNA entrada. Nunca un booleano: un
/// instrumento que colapsa esto a verde/rojo miente por omision.
#[derive(Clone, Debug, PartialEq)]
pub enum Veredicto {
    /// El sello se recompuso desde el registro y coincide.
    Verificada { seq: u64 },
    /// El sello no es recomputable, pero SI se comprobo algo: que la
    /// entrada no lleva un sello reservado que no le corresponde.
    Parcial { seq: u64, comprobado: &'static str, falta: &'static str },
    /// No hay nada comprobable mas alla de la cadena.
    NoDerivable { seq: u64, falta: &'static str },
}

/// Una entrada que MIENTE no es una clase del censo: lo invalida.
///
/// ⚠️ Por eso esto es un error y no un cuarto veredicto. Es el fail-stop
/// que el propio proyecto declara (`doc/CONFIANZA_RESIDUAL.md` §5.2): la
/// capa se protege negandose a fluir sin prueba, no anotando la anomalia
/// y siguiendo.
#[derive(Clone, Debug, PartialEq)]
pub enum ReverificacionError {
    /// El sello recompuesto no coincide con el asentado.
    SelloDiscrepante { seq: u64 },
    /// La entrada lleva un sello reservado a otra clase — la regresion
    /// que §278 cerro, detectada.
    SelloReservadoAjeno { seq: u64 },
    /// §716 (el reenvio de una emision delegada): la entrada `seq` repite el compromiso de
    /// la entrada `primera`, y su compromiso lleva contador. Una autorizacion se aplico dos veces.
    CompromisoRepetido { seq: u64, primera: u64 },
}

/// §716 (el reenvio de una emision delegada): desde que entrada el compromiso de `Mint` y
/// de `MintToPending` lleva el contador de emisiones. El registro no lo dice; lo declara quien
/// reverifica, que sabe que libro mira.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmisionConContador {
    /// No lo lleva, o no se sabe. Un compromiso de emision repetido no es ROJO.
    No,
    /// Lo lleva desde esta `seq`, inclusive. Un libro nacido con el arreglo: `DesdeSeq(0)`.
    DesdeSeq(u64),
}

/// Las delegadas cuyo compromiso lleva contador desde antes de la era 2.
const CON_CONTADOR: [&str; 3] = ["Freeze", "Recovery", "Governance"];

/// Las vias delegadas: las que deben llevar sello de AUTORIZACION.
const DELEGADAS: [&str; 5] = ["Mint", "Freeze", "Recovery", "Governance", "MintToPending"];

/// **Reverifica un registro, entrada por entrada y era por era.**
///
/// ⚠️ Un registro de **era 1** tiene que empezar en `seq = 0` para que sus
/// `Recovery` sean recomputables: el contador se deriva contando las
/// anteriores. Un registro de **era 2** no lo necesita —el compromiso
/// viaja en la entrada—, asi que un TRAMO (`fromSeq`, limite 1000) vale.
/// La condicion es **por entrada**, no del registro entero.
///
/// §716 (el reenvio de una emision delegada): es [`reverificar_con`] sin declarar el
/// contador de las emisiones, [`EmisionConContador::No`].
pub fn reverificar(entradas: &[EntradaLog]) -> Result<Vec<Veredicto>, ReverificacionError> {
    reverificar_con(entradas, EmisionConContador::No)
}

/// §716 (el reenvio de una emision delegada): [`reverificar`], con el contador de las
/// emisiones declarado. Un compromiso repetido es ROJO cuando la entrada que lo repite lleva
/// contador.
pub fn reverificar_con(
    entradas: &[EntradaLog],
    emision: EmisionConContador,
) -> Result<Vec<Veredicto>, ReverificacionError> {
    let completo = entradas.first().map(|e| e.seq == 0).unwrap_or(false);
    // §716 (el reenvio de una emision delegada): cada compromiso de una delegada visto, con
    // la primera entrada que lo llevo
    let mut vistos: std::collections::HashMap<[u8; 32], u64> = std::collections::HashMap::new();
    let lleva_contador = |e: &EntradaLog| {
        CON_CONTADOR.contains(&e.kind.as_str())
            || (matches!(e.kind.as_str(), "Mint" | "MintToPending")
                && matches!(emision, EmisionConContador::DesdeSeq(d) if e.seq >= d))
    };
    let vacio = digest_of_proof(&[]);
    let declarado = digest_of_proof(&sello_sin_prueba());

    let mut fuera = Vec::with_capacity(entradas.len());
    let mut recuperaciones: u64 = 0;

    for e in entradas {
        let delegada = DELEGADAS.contains(&e.kind.as_str());

        // El negativo que detecta la regresion de §278: un sello que no
        // corresponde a la clase de la entrada.
        if delegada && (e.proof_digest == vacio || e.proof_digest == declarado) {
            return Err(ReverificacionError::SelloReservadoAjeno { seq: e.seq });
        }
        if e.kind != "OpenAccount" && !delegada && e.proof_digest == declarado {
            return Err(ReverificacionError::SelloReservadoAjeno { seq: e.seq });
        }
        // Y el de era 2: una delegada que asienta el CENTINELA en vez de su
        // compromiso. El centinela esta reservado a las no-delegadas, asi
        // que esto es la capa escribiendo mal, no una entrada opaca.
        if delegada && e.compromiso == Some(COMPROMISO_AUSENTE) {
            return Err(ReverificacionError::SelloReservadoAjeno { seq: e.seq });
        }

        // `OpenAccount` lo decide su `proof_digest` —el sello de ausencia
        // declarada, que es una CONSTANTE—, no su compromiso: en era 2
        // lleva el centinela y sigue siendo verificable.
        if e.kind == "OpenAccount" {
            if e.proof_digest != declarado {
                return Err(ReverificacionError::SelloDiscrepante { seq: e.seq });
            }
            fuera.push(Veredicto::Verificada { seq: e.seq });
            continue;
        }

        // ── Era 2: el compromiso esta en la entrada. Cualquier delegada
        //    se recompone sin mirar el resto del registro.
        if let Some(c) = e.compromiso {
            if c != COMPROMISO_AUSENTE {
                if e.proof_digest != digest_of_proof(&sello_de_autorizacion(&c)) {
                    return Err(ReverificacionError::SelloDiscrepante { seq: e.seq });
                }
                if e.kind == "Recovery" {
                    recuperaciones += 1;
                }
                // §716 (el reenvio de una emision delegada): el compromiso repetido.
                if let Some(&primera) = vistos.get(&digest_to_bytes(&c)) {
                    if lleva_contador(e) {
                        return Err(ReverificacionError::CompromisoRepetido { seq: e.seq, primera });
                    }
                    fuera.push(Veredicto::Parcial {
                        seq: e.seq,
                        comprobado: "el sello es el de su compromiso",
                        falta: "repite un compromiso sin contador: reenvio o reautorizacion, no se sabe",
                    });
                    continue;
                }
                vistos.insert(digest_to_bytes(&c), e.seq);
                fuera.push(Veredicto::Verificada { seq: e.seq });
                continue;
            }
            // Centinela en una no-delegada: su prueba es real y no se
            // guarda. Lo comprobable es que no usurpa un sello ajeno.
            fuera.push(Veredicto::Parcial {
                seq: e.seq,
                comprobado: "no lleva un sello reservado a otra clase",
                falta: "la prueba no se guarda: su resumen no es recomputable",
            });
            continue;
        }

        // ── Era 1: sin compromiso en la entrada.
        if e.kind == "Recovery" {
            if !completo {
                fuera.push(Veredicto::NoDerivable {
                    seq: e.seq,
                    falta: "era 1 y el registro no empieza en seq 0: el contador no es derivable",
                });
                recuperaciones += 1;
                continue;
            }
            let mut params: Vec<BaseElement> = e.root_old.to_vec();
            params.extend_from_slice(&e.root_new);
            params.push(BaseElement::new(recuperaciones));
            params.push(BaseElement::new(recuperaciones + 1));
            let op = commit_operation(OP_RECOVERY, &params);
            if e.proof_digest != digest_of_proof(&sello_de_autorizacion(&op)) {
                return Err(ReverificacionError::SelloDiscrepante { seq: e.seq });
            }
            recuperaciones += 1;
            fuera.push(Veredicto::Verificada { seq: e.seq });
            continue;
        }

        if delegada {
            fuera.push(Veredicto::Parcial {
                seq: e.seq,
                comprobado: "no lleva un sello reservado a otra clase",
                falta: "era 1: los parametros del compromiso no estan en el registro",
            });
        } else {
            fuera.push(Veredicto::Parcial {
                seq: e.seq,
                comprobado: "no lleva el sello de ausencia declarada",
                falta: "la prueba no se guarda: su resumen no es recomputable",
            });
        }
    }
    Ok(fuera)
}

/// Cuenta cuantas de cada clase. **Se cuenta la SALIDA del instrumento**,
/// no una tabla por tipo de operacion (§266: un contador puede cuadrar y
/// mentir si mide lo que uno espera en vez de lo que ocurrio).
pub fn censo(v: &[Veredicto]) -> (usize, usize, usize) {
    let mut c = (0, 0, 0);
    for x in v {
        match x {
            Veredicto::Verificada { .. } => c.0 += 1,
            Veredicto::Parcial { .. } => c.1 += 1,
            Veredicto::NoDerivable { .. } => c.2 += 1,
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    // ⚠️ Solo los tests construyen digests sinteticos: importarlo arriba
    // dejaria un `unused_imports` en el target lib, y los warnings de este
    // crate estan pinchados a 0.
    use zk_ssl_hash::as_digest;

    fn entrada(seq: u64, kind: &str, pd: Digest) -> EntradaLog {
        EntradaLog {
            seq,
            kind: kind.into(),
            root_old: as_digest(seq),
            root_new: as_digest(seq + 1),
            proof_digest: pd,
            // Era 1 por defecto: los tests de era 2 usan `entrada_v2`.
            compromiso: None,
        }
    }

    /// Sello de recuperacion tal como lo compone la capa, con LAS MISMAS
    /// funciones: si divergieran, este test pasaria sin probar nada.
    fn sello_recovery(root_old: Digest, root_new: Digest, count: u64) -> Digest {
        let mut p: Vec<BaseElement> = root_old.to_vec();
        p.extend_from_slice(&root_new);
        p.push(BaseElement::new(count));
        p.push(BaseElement::new(count + 1));
        digest_of_proof(&sello_de_autorizacion(&commit_operation(OP_RECOVERY, &p)))
    }

    /// **Verificada**: el escenario canonico reconstruido con las mismas
    /// reglas — dos aperturas, dos emisiones, envio y cobro— mas una
    /// recuperacion recompuesta de punta a punta.
    #[test]
    fn el_censo_del_escenario_canonico_y_una_recuperacion() {
        let declarado = digest_of_proof(&sello_sin_prueba());
        let mut log = vec![
            entrada(0, "OpenAccount", declarado),
            entrada(1, "Mint", digest_of_proof(b"sello-de-mint")),
            entrada(2, "OpenAccount", declarado),
            entrada(3, "Mint", digest_of_proof(b"otro-sello-de-mint")),
            entrada(4, "Send", digest_of_proof(b"prueba-de-envio")),
            entrada(5, "Claim", digest_of_proof(b"prueba-de-cobro")),
        ];
        let v = reverificar(&log).expect("el escenario canonico no miente");
        assert_eq!(censo(&v), (2, 4, 0), "dos verificadas, cuatro parciales");

        // Y una recuperacion, que SI se recompone desde el registro.
        let r = entrada(6, "Recovery", as_digest(0));
        let sello = sello_recovery(r.root_old, r.root_new, 0);
        log.push(EntradaLog { proof_digest: sello, ..r });
        let v = reverificar(&log).expect("la recuperacion honesta verifica");
        assert_eq!(censo(&v), (3, 4, 0), "la recuperacion sube las verificadas");
    }

    /// **Parcial, por su negativo**: una delegada que lleva un sello
    /// reservado a otra clase es la regresion que §278 cerro, y el
    /// instrumento se niega a censarla.
    #[test]
    fn una_delegada_con_sello_ajeno_invalida_el_censo() {
        let vacio = digest_of_proof(&[]);
        let log = vec![
            entrada(0, "OpenAccount", digest_of_proof(&sello_sin_prueba())),
            entrada(1, "Mint", vacio),
        ];
        assert_eq!(
            reverificar(&log),
            Err(ReverificacionError::SelloReservadoAjeno { seq: 1 }),
            "una delegada con el digest vacio es la regresion de §278"
        );
    }

    /// **NoDerivable**: con un TRAMO del registro el contador no existe,
    /// y el instrumento lo dice en vez de concluir (§254).
    #[test]
    fn un_tramo_del_registro_no_deriva_el_contador() {
        let r = entrada(100, "Recovery", as_digest(0));
        let sello = sello_recovery(r.root_old, r.root_new, 0);
        let log = vec![EntradaLog { proof_digest: sello, ..r }];
        let v = reverificar(&log).expect("un tramo no es una mentira");
        assert_eq!(censo(&v), (0, 0, 1), "sin genesis no hay contador");
        assert!(matches!(v[0], Veredicto::NoDerivable { seq: 100, .. }));
    }

    /// Una entrada de **era 2**: el compromiso viaja dentro.
    fn entrada_v2(seq: u64, kind: &str, pd: Digest, c: Digest) -> EntradaLog {
        EntradaLog { compromiso: Some(c), ..entrada(seq, kind, pd) }
    }

    /// Sello de una delegada cualquiera a partir de su compromiso — la
    /// composicion es la MISMA que usa la capa (`zk-ssl-hash`).
    fn sello(c: Digest) -> Digest {
        digest_of_proof(&sello_de_autorizacion(&c))
    }

    /// **El censo de era 2: cuatro de seis.** Predicho antes de escribir
    /// el test y contado sobre la SALIDA del instrumento (§266). Las dos
    /// que faltan son `Send` y `Claim`: su prueba es real y el registro
    /// no la guarda — no es deuda de era, es diseno.
    #[test]
    fn el_censo_del_escenario_canonico_en_era_2() {
        let declarado = digest_of_proof(&sello_sin_prueba());
        let c1 = as_digest(0xA1);
        let c2 = as_digest(0xA2);
        let log = vec![
            entrada_v2(0, "OpenAccount", declarado, COMPROMISO_AUSENTE),
            entrada_v2(1, "Mint", sello(c1), c1),
            entrada_v2(2, "OpenAccount", declarado, COMPROMISO_AUSENTE),
            entrada_v2(3, "Mint", sello(c2), c2),
            entrada_v2(4, "Send", digest_of_proof(b"prueba-de-envio"), COMPROMISO_AUSENTE),
            entrada_v2(5, "Claim", digest_of_proof(b"prueba-de-cobro"), COMPROMISO_AUSENTE),
        ];
        let v = reverificar(&log).expect("el escenario canonico no miente");
        assert_eq!(censo(&v), (4, 2, 0), "era 2: cuatro verificadas, dos parciales");
    }

    /// **Una delegada con el CENTINELA es la capa escribiendo mal.** El
    /// centinela esta reservado a las no-delegadas; el instrumento se
    /// niega a censar el registro en vez de anotarlo y seguir.
    #[test]
    fn una_delegada_con_el_centinela_invalida_el_censo() {
        let c = as_digest(0xB0);
        let log = vec![entrada_v2(7, "Governance", sello(c), COMPROMISO_AUSENTE)];
        assert_eq!(
            reverificar(&log),
            Err(ReverificacionError::SelloReservadoAjeno { seq: 7 }),
            "el centinela no es de una delegada"
        );
    }

    /// §716 (el reenvio de una emision delegada): un compromiso repetido. ROJO donde lleva
    /// contador -una congelacion siempre; una emision, desde la entrada que se declara-, y sin
    /// contador, `Parcial`.
    #[test]
    fn un_compromiso_con_contador_repetido_es_rojo() {
        let declarado = digest_of_proof(&sello_sin_prueba());
        let c = as_digest(0xF1);
        let log = vec![
            entrada_v2(0, "OpenAccount", declarado, COMPROMISO_AUSENTE),
            entrada_v2(1, "Freeze", sello(c), c),
            entrada_v2(2, "Freeze", sello(c), c),
        ];
        assert_eq!(
            reverificar(&log),
            Err(ReverificacionError::CompromisoRepetido { seq: 2, primera: 1 }),
            "una congelacion lleva contador: repetir su compromiso es aplicarla dos veces"
        );
    }

    #[test]
    fn un_compromiso_de_emision_repetido_es_rojo_solo_con_contador() {
        let declarado = digest_of_proof(&sello_sin_prueba());
        let c = as_digest(0xE1);
        let log = vec![
            entrada_v2(0, "OpenAccount", declarado, COMPROMISO_AUSENTE),
            entrada_v2(1, "Mint", sello(c), c),
            entrada_v2(2, "Burn", digest_of_proof(b"prueba-de-quema"), COMPROMISO_AUSENTE),
            entrada_v2(3, "Mint", sello(c), c),
        ];

        // Sin contador: puede ser una reautorizacion legitima. No es ROJO, y no es Verificada.
        let v = reverificar(&log).expect("sin contador no acusa");
        assert_eq!(censo(&v), (2, 2, 0), "la apertura y la primera; la quema y la repetida, parciales");
        assert!(matches!(v[3], Veredicto::Parcial { seq: 3, .. }), "{:?}", v[3]);

        // Con contador desde el genesis: la autorizacion se aplico dos veces.
        assert_eq!(
            reverificar_con(&log, EmisionConContador::DesdeSeq(0)),
            Err(ReverificacionError::CompromisoRepetido { seq: 3, primera: 1 })
        );
        // Y con el contador declarado desde la repetida: lo decide la entrada que repite.
        assert_eq!(
            reverificar_con(&log, EmisionConContador::DesdeSeq(3)),
            Err(ReverificacionError::CompromisoRepetido { seq: 3, primera: 1 })
        );
        // Declarado despues: nada cambia respecto a `reverificar`.
        assert_eq!(reverificar_con(&log, EmisionConContador::DesdeSeq(4)), Ok(v));
    }

    /// **Era 2 en un TRAMO**: sin genesis y aun asi verificable, porque el
    /// compromiso no necesita contar nada anterior. Es lo que la era 1 no
    /// podia hacer, y el tramo es el caso normal de `zkssl_logEntries`.
    #[test]
    fn una_recuperacion_de_era_2_verifica_en_un_tramo() {
        let c = as_digest(0xC7);
        let log = vec![entrada_v2(500, "Recovery", sello(c), c)];
        let v = reverificar(&log).expect("un tramo de era 2 no es una mentira");
        assert_eq!(censo(&v), (1, 0, 0), "el compromiso no necesita el genesis");
        assert_eq!(v[0], Veredicto::Verificada { seq: 500 });
    }
}

