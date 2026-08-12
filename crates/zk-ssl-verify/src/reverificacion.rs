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

use zk_ssl_hash::{
    commit_operation, digest_of_proof, sello_de_autorizacion, sello_sin_prueba, Digest,
    OP_RECOVERY,
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
}

/// Las vias delegadas: las que deben llevar sello de AUTORIZACION.
const DELEGADAS: [&str; 5] = ["Mint", "Freeze", "Recovery", "Governance", "MintToPending"];

/// **Reverifica un registro COMPLETO desde el genesis.**
///
/// ⚠️ El registro tiene que empezar en `seq = 0`: el contador de
/// recuperaciones se deriva contando las entradas anteriores, y
/// `zkssl_logEntries` sirve TRAMOS (`fromSeq`, limite 1000), asi que el
/// registro parcial es el caso normal, no la excepcion. Con un tramo, la
/// respuesta no es «no coincide»: es que no se puede saber.
pub fn reverificar(entradas: &[EntradaLog]) -> Result<Vec<Veredicto>, ReverificacionError> {
    let completo = entradas.first().map(|e| e.seq == 0).unwrap_or(false);
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

        if e.kind == "OpenAccount" {
            if e.proof_digest != declarado {
                return Err(ReverificacionError::SelloDiscrepante { seq: e.seq });
            }
            fuera.push(Veredicto::Verificada { seq: e.seq });
            continue;
        }

        if e.kind == "Recovery" {
            if !completo {
                fuera.push(Veredicto::NoDerivable {
                    seq: e.seq,
                    falta: "el registro no empieza en seq 0: el contador no es derivable",
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
                falta: "los parametros del compromiso no estan en el registro",
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
}

