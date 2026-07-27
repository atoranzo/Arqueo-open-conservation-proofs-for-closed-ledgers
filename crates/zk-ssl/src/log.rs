//! **Registro verificable de transiciones.** El operador no puede
//! reescribir el historial en secreto.
//!
//! ## ⚠️ Esto NO es descentralización
//!
//! El operador de un nodo único tiene tres poderes distintos:
//!
//! | Poder | ¿Lo cierra este registro? |
//! |---|---|
//! | Ve todos los saldos | **No** |
//! | Ordena las operaciones y puede censurar | **No** |
//! | **Bifurcar o reescribir el historial** | **Sí** |
//!
//! Cerrar los dos primeros exige consenso distribuido, que es un problema
//! de otra disciplina y no está abordado.
//!
//! Lo que sí se cierra aquí es el tercero, y tiene nombre propio: **no
//! repudio del historial**.
//!
//! ## Cómo
//!
//! Cada operación aplicada deja una entrada encadenada:
//!
//! ```text
//! entrada = (numero, tipo, raiz_antigua, raiz_nueva, prueba, resumen_anterior)
//! resumen = H(numero, tipo, raiz_antigua, raiz_nueva, H(prueba), resumen_anterior)
//! ```
//!
//! El **resumen encadenado** es lo que hace el registro inmutable en la
//! práctica: alterar una entrada antigua cambia todos los resúmenes
//! posteriores. No se puede reescribir el pasado sin reescribir todo lo
//! que vino después.
//!
//! ## Qué permite comprobar a cualquiera
//!
//! 1. **Que la cadena es coherente**: cada raíz antigua es la raíz nueva
//!    de la entrada anterior. Sin huecos ni saltos.
//! 2. **Que cada transición está demostrada**: la prueba está ahí y
//!    verifica.
//! 3. **Que dos copias del registro coinciden**: si el operador mostró
//!    historiales distintos a partes distintas, los resúmenes divergen en
//!    la entrada donde se bifurcó.
//!
//! Es lo que hace *Certificate Transparency* con las autoridades de
//! certificación: no impide que se porten mal, **hace que no puedan
//! hacerlo en secreto**.
//!
//! ## ⚠️ Lo que sigue sin resolver
//!
//! - **Nadie está obligado a mirar.** El registro permite detectar una
//!   bifurcación; que alguien la detecte depende de que haya observadores
//!   comparando copias.
//! - **El operador podría no publicar el registro.** Que exista no obliga
//!   a entregarlo. Obligar a publicar es, otra vez, consenso.
//! - **No impide la censura.** Una operación que nunca se procesa no deja
//!   entrada, y su ausencia es indistinguible de que nunca se pidió.

use super::*;
use stark_experiment::merkle::native_merge;

/// Tipo de operación registrada.
///
/// Se guarda para que un verificador sepa **qué circuito** debe usar al
/// comprobar la prueba de cada entrada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpKind {
    OpenAccount,
    Mint,
    Transfer,
    Burn,
    Recovery,
    Governance,
    Freeze,
}

impl OpKind {
    /// Etiqueta de un byte, para serializar.
    pub fn tag_byte(&self) -> u8 {
        self.tag() as u8
    }

    /// Inversa de `tag_byte`. `None` si el byte no corresponde a ninguna
    /// operación conocida.
    pub fn from_tag_byte(b: u8) -> Option<Self> {
        Some(match b {
            1 => OpKind::OpenAccount,
            2 => OpKind::Mint,
            3 => OpKind::Transfer,
            4 => OpKind::Burn,
            5 => OpKind::Recovery,
            6 => OpKind::Governance,
            7 => OpKind::Freeze,
            _ => return None,
        })
    }

    fn tag(&self) -> u64 {
        match self {
            OpKind::OpenAccount => 1,
            OpKind::Mint => 2,
            OpKind::Transfer => 3,
            OpKind::Burn => 4,
            OpKind::Recovery => 5,
            OpKind::Governance => 6,
            OpKind::Freeze => 7,
        }
    }
}

/// Una entrada del registro.
#[derive(Clone, Debug)]
pub struct LogEntry {
    /// Número de secuencia, desde 0.
    pub seq: u64,
    pub kind: OpKind,
    /// **Raíz de estado de CUENTAS**, siempre — no la del árbol que la
    /// operación modifica.
    ///
    /// Encadenar raíces de árboles distintos no funciona: la raíz de
    /// custodios de una entrada no tiene por qué ser la de cuentas de la
    /// siguiente. Las operaciones que no tocan el árbol de cuentas
    /// —gobernanza, congelación— tienen `root_old == root_new`, y el
    /// detalle de lo que sí cambiaron queda atado por `proof_digest`.
    pub root_old: Digest,
    pub root_new: Digest,
    /// Resumen de la prueba, no la prueba entera.
    ///
    /// Guardar las pruebas completas serían ~62 KB por operación: mil
    /// transferencias son 59 MB. El resumen basta para atar la entrada a
    /// una prueba concreta, y quien quiera verificarla puede pedirla.
    pub proof_digest: Digest,
    /// Resumen encadenado hasta esta entrada, inclusive.
    pub chain: Digest,
}

fn as_digest(x: u64) -> Digest {
    [
        BaseElement::new(x),
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
    ]
}

/// Resumen de una prueba serializada.
pub fn digest_of_proof(proof: &[u8]) -> Digest {
    // Se procesa por bloques de 8 bytes hacia un acumulador. No pretende
    // ser una funcion hash de proposito general: solo tiene que atar la
    // entrada a una prueba concreta.
    let mut acc: Digest = [BaseElement::ZERO; 4];
    for bloque in proof.chunks(8) {
        let mut b = [0u8; 8];
        b[..bloque.len()].copy_from_slice(bloque);
        // Reducir al campo: Goldilocks no admite todos los u64.
        let v = u64::from_le_bytes(b) % 0xFFFF_FFFF_0000_0001;
        acc = native_merge(acc, as_digest(v));
    }
    acc
}

/// Calcula el resumen encadenado de una entrada.
pub fn chain_digest(
    seq: u64,
    kind: OpKind,
    root_old: Digest,
    root_new: Digest,
    proof_digest: Digest,
    previous: Digest,
) -> Digest {
    let cabecera = native_merge(as_digest(seq), as_digest(kind.tag()));
    let raices = native_merge(root_old, root_new);
    let cuerpo = native_merge(cabecera, raices);
    native_merge(native_merge(cuerpo, proof_digest), previous)
}

/// Registro encadenado de transiciones.
#[derive(Clone, Debug, Default)]
pub struct TransitionLog {
    entries: Vec<LogEntry>,
}

impl TransitionLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reconstruye desde entradas leídas del disco.
    ///
    /// **No las valida**: quien lo use debe llamar a `verify` después. Se
    /// separa a propósito, para que cargar y comprobar sean dos actos
    /// distintos y visibles.
    pub fn from_entries(entries: Vec<LogEntry>) -> Self {
        Self { entries }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    /// Resumen de la cabeza. Es **el compromiso de todo el historial**:
    /// dos nodos con la misma cabeza tienen la misma historia.
    pub fn head(&self) -> Digest {
        self.entries
            .last()
            .map(|e| e.chain)
            .unwrap_or([BaseElement::ZERO; 4])
    }

    /// Añade una entrada.
    pub fn append(
        &mut self,
        kind: OpKind,
        root_old: Digest,
        root_new: Digest,
        proof: &[u8],
    ) -> Digest {
        let seq = self.entries.len() as u64;
        let proof_digest = digest_of_proof(proof);
        let previous = self.head();
        let chain = chain_digest(seq, kind, root_old, root_new, proof_digest, previous);
        self.entries.push(LogEntry {
            seq,
            kind,
            root_old,
            root_new,
            proof_digest,
            chain,
        });
        chain
    }

    /// **Comprueba la coherencia interna del registro**, sin necesitar la
    /// raíz de génesis.
    ///
    /// Existe para verificar un registro **restaurado de una copia**,
    /// donde no se conoce el estado inicial. Detecta manipulación del
    /// contenido de cualquier entrada, porque el encadenamiento la
    /// propaga.
    ///
    /// No detecta que falte el principio: para eso hace falta `verify`
    /// con el génesis.
    pub fn verify_chain(&self) -> Result<(), LogError> {
        let mut previous = [BaseElement::ZERO; 4];
        for (i, e) in self.entries.iter().enumerate() {
            if e.seq != i as u64 {
                return Err(LogError::OutOfSequence {
                    position: i as u64,
                    found: e.seq,
                });
            }
            if i > 0 && e.root_old != self.entries[i - 1].root_new {
                return Err(LogError::BrokenChain { at: e.seq });
            }
            let esperado = chain_digest(
                e.seq,
                e.kind,
                e.root_old,
                e.root_new,
                e.proof_digest,
                previous,
            );
            if esperado != e.chain {
                return Err(LogError::TamperedEntry { at: e.seq });
            }
            previous = e.chain;
        }
        Ok(())
    }

    /// **Verifica el registro entero desde el génesis.**
    ///
    /// Comprueba tres cosas, y la segunda es la que importa:
    ///
    /// 1. Los números de secuencia son consecutivos.
    /// 2. **Cada raíz antigua es la raíz nueva de la entrada anterior**:
    ///    sin huecos ni saltos. Es lo que impide insertar o borrar
    ///    operaciones del medio.
    /// 3. Cada resumen encadenado es el que corresponde.
    ///
    /// No verifica las pruebas —no las guarda— pero sí que cada entrada
    /// esté atada a una prueba concreta.
    pub fn verify(&self, genesis_root: Digest) -> Result<(), LogError> {
        let mut previous = [BaseElement::ZERO; 4];
        let mut expected_root = genesis_root;

        for (i, e) in self.entries.iter().enumerate() {
            if e.seq != i as u64 {
                return Err(LogError::OutOfSequence {
                    position: i as u64,
                    found: e.seq,
                });
            }
            if e.root_old != expected_root {
                return Err(LogError::BrokenChain { at: e.seq });
            }
            let esperado = chain_digest(
                e.seq,
                e.kind,
                e.root_old,
                e.root_new,
                e.proof_digest,
                previous,
            );
            if esperado != e.chain {
                return Err(LogError::TamperedEntry { at: e.seq });
            }
            previous = e.chain;
            expected_root = e.root_new;
        }
        Ok(())
    }

    /// **Compara dos registros y localiza dónde divergen.**
    ///
    /// Es la operación que detecta una bifurcación: si el operador mostró
    /// historiales distintos a dos partes, esto dice en qué entrada
    /// empezó la mentira.
    ///
    /// Devuelve `None` si uno es prefijo del otro —lo normal cuando una
    /// copia está más atrasada—.
    pub fn first_divergence(&self, other: &TransitionLog) -> Option<u64> {
        for (a, b) in self.entries.iter().zip(other.entries.iter()) {
            if a.chain != b.chain {
                return Some(a.seq);
            }
        }
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LogError {
    OutOfSequence { position: u64, found: u64 },
    /// Una raíz antigua no coincide con la nueva de la entrada anterior:
    /// falta una operación, o se insertó una que no estaba.
    BrokenChain { at: u64 },
    TamperedEntry { at: u64 },
}

impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogError::OutOfSequence { position, found } => write!(
                f,
                "la entrada en la posicion {position} dice ser la {found}"
            ),
            LogError::BrokenChain { at } => write!(
                f,
                "la entrada {at} no encadena con el estado anterior: falta una \
                 operacion o se inserto una que no estaba"
            ),
            LogError::TamperedEntry { at } => {
                write!(f, "el resumen de la entrada {at} no corresponde a su contenido")
            }
        }
    }
}
impl std::error::Error for LogError {}

impl SovereignLayer {
    /// Registro encadenado de transiciones.
    pub fn transition_log(&self) -> &TransitionLog {
        &self.log
    }

    /// Resumen de la cabeza: **el compromiso de todo el historial**.
    ///
    /// Publicarlo periódicamente basta para que cualquiera detecte una
    /// reescritura posterior.
    pub fn log_head(&self) -> Digest {
        self.log.head()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(x: u64) -> Digest {
        [
            BaseElement::new(x),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ]
    }

    fn log_de_tres() -> TransitionLog {
        let mut l = TransitionLog::new();
        l.append(OpKind::Mint, d(0), d(1), b"prueba-1");
        l.append(OpKind::Transfer, d(1), d(2), b"prueba-2");
        l.append(OpKind::Burn, d(2), d(3), b"prueba-3");
        l
    }

    /// Un registro bien formado se verifica desde el génesis.
    #[test]
    fn a_well_formed_log_verifies() {
        assert!(log_de_tres().verify(d(0)).is_ok());
    }

    /// **BORRAR UNA OPERACIÓN DEL MEDIO SE DETECTA.**
    ///
    /// Es el ataque que este registro existe para impedir: el operador
    /// haciendo desaparecer una transferencia del historial.
    #[test]
    fn removing_an_operation_is_detected() {
        let mut l = log_de_tres();
        l.entries.remove(1);
        let r = l.verify(d(0));
        assert!(
            matches!(r, Err(LogError::OutOfSequence { .. }) | Err(LogError::BrokenChain { .. })),
            "CRITICO: borrar una operacion del historial debe detectarse. \
             Resultado: {r:?}"
        );
    }

    /// **ALTERAR UNA ENTRADA SE DETECTA.**
    #[test]
    fn tampering_with_an_entry_is_detected() {
        let mut l = log_de_tres();
        l.entries[1].root_new = d(99);
        assert!(l.verify(d(0)).is_err());
    }

    /// **Y alterar también el resumen no basta.**
    ///
    /// Un operador astuto recalcularía el resumen de la entrada que
    /// modifica. Pero el encadenamiento hace que los **posteriores** dejen
    /// de cuadrar: reescribir el pasado exige reescribir todo lo que vino
    /// después.
    #[test]
    fn recomputing_one_digest_is_not_enough() {
        let mut l = log_de_tres();
        l.entries[1].root_new = d(99);
        // El atacante recalcula el resumen de ESA entrada.
        l.entries[1].chain = chain_digest(
            1,
            l.entries[1].kind,
            l.entries[1].root_old,
            d(99),
            l.entries[1].proof_digest,
            l.entries[0].chain,
        );
        let r = l.verify(d(0));
        assert!(
            r.is_err(),
            "CRITICO: recalcular un resumen no debe bastar; el encadenamiento \
             tiene que propagar la inconsistencia"
        );
    }

    /// **UNA BIFURCACIÓN SE LOCALIZA.**
    ///
    /// Si el operador mostró historiales distintos a dos partes, esto dice
    /// en qué entrada empezó.
    #[test]
    fn a_fork_is_located() {
        let a = log_de_tres();
        let mut b = TransitionLog::new();
        b.append(OpKind::Mint, d(0), d(1), b"prueba-1");
        b.append(OpKind::Transfer, d(1), d(7), b"otra-cosa"); // diverge aqui
        b.append(OpKind::Burn, d(7), d(8), b"prueba-3");

        assert_eq!(
            a.first_divergence(&b),
            Some(1),
            "la bifurcacion empieza en la entrada 1"
        );
    }

    /// Una copia atrasada NO es una bifurcación.
    ///
    /// Sin esta distinción, cualquier nodo que aún no haya recibido las
    /// últimas operaciones parecería estar mintiendo.
    #[test]
    fn a_lagging_copy_is_not_a_fork() {
        let completo = log_de_tres();
        let mut atrasado = TransitionLog::new();
        atrasado.append(OpKind::Mint, d(0), d(1), b"prueba-1");
        assert_eq!(completo.first_divergence(&atrasado), None);
    }

    /// **La cabeza compromete todo el historial.**
    ///
    /// Dos nodos con la misma cabeza tienen la misma historia; publicarla
    /// periódicamente basta para que cualquiera detecte una reescritura.
    #[test]
    fn the_head_commits_to_the_whole_history() {
        let a = log_de_tres();
        let mut b = log_de_tres();
        assert_eq!(a.head(), b.head());

        b.append(OpKind::Freeze, d(3), d(4), b"otra");
        assert_ne!(a.head(), b.head());
    }

    /// El resumen de la prueba distingue pruebas distintas.
    #[test]
    fn different_proofs_give_different_digests() {
        assert_ne!(digest_of_proof(b"una prueba"), digest_of_proof(b"otra prueba"));
        assert_eq!(digest_of_proof(b"igual"), digest_of_proof(b"igual"));
    }
}
