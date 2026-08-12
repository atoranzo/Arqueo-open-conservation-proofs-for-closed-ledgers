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
    /// **Envío en dos fases: el dinero sale y queda en un pendiente.**
    ///
    /// ⚠️ Estas tres faltaban. `two_phase.rs` era **el único módulo que no
    /// registraba nada**, así que los envíos, los cobros y las emisiones a
    /// pendiente **no dejaban rastro en el registro de transiciones** — que
    /// es el mecanismo de auditoría del sistema.
    ///
    /// Se descubrió al migrar `the_log_chains_every_operation`: el recuento
    /// bajó de 5 a 4 en vez de subir a 6. Ver `AUDITORIA.md` §26.
    Send,
    /// Cobro de un pendiente: el receptor lo hace suyo.
    Claim,
    /// Emisión de custodios directamente a un pendiente.
    MintToPending,
    /// **Migración única de B13/B14** (spec del paso 1): reposiciona las
    /// cuentas por identidad, envuelve las hojas con el salt del record
    /// (cero para lo legacy) y remapea frozen a profundidad 32. Cambia
    /// DOS árboles: las raíces de cuentas van en la entrada; las de
    /// frozen, comprometidas en el payload (no es prueba, es compromiso
    /// replicable). Se registra sin prueba, como OpenAccount.
    Migration,
    /// Reembolso de un pendiente caducado al emisor (§178, R-2c) — o su
    /// des-emisión si nació por emisión (centinela).
    Refund,
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
            8 => OpKind::Send,
            9 => OpKind::Claim,
            10 => OpKind::MintToPending,
            11 => OpKind::Migration,
            12 => OpKind::Refund,
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
            OpKind::Send => 8,
            OpKind::Claim => 9,
            OpKind::MintToPending => 10,
            OpKind::Migration => 11,
            OpKind::Refund => 12,
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
    ///
    /// ⚠️ **CORRECCIÓN (§271): esa última frase es FALSA hoy, y justo para
    /// las dos que nombra.** `governance.rs:162` y `freeze.rs:138` asientan
    /// con **prueba vacía**, así que su `proof_digest` es una constante y no
    /// ata ningún detalle. Para esas dos, la entrada dice sólo *«pasó algo
    /// de esta clase en este `seq`»*. Ver la nota 78 del BACKLOG.
    ///
    /// ✅ **CERRADO en §278.** Las cinco vías delegadas —`mint`, `freeze`,
    /// `recovery`, `governance` y `MintToPending`— asientan ya el **sello
    /// de autorización** de su compromiso, así que la frase de arriba
    /// vuelve a ser cierta con una precisión: lo que ata la entrada no es
    /// la prueba, es **el compromiso contra el que se verificó**. Ver
    /// [`sello_de_autorizacion`].
    ///
    /// ⚠️ Lo que **sí** está atado, y conviene no confundirlo: la
    /// AUTORIZACIÓN. `apply_governance_delegated` verifica contra
    /// `commit_operation(OP_GOVERNANCE, raíz_vieja ‖ raíz_nueva ‖ count_old
    /// ‖ count_new)`, así que el cambio está ligado a la transición exacta.
    /// Lo que no ocurre es que **eso llegue al registro**.
    pub root_old: Digest,
    pub root_new: Digest,
    /// **El compromiso autorizante, como campo** (§281; la lista de la
    /// nota 82, §280). `None` = **era 1**: entrada anterior a §281, cuyo
    /// compromiso no existe en ningún sitio — no se guardó nunca.
    /// `Some` = **era 2**: toda entrada nueva. Las delegadas llevan el
    /// compromiso contra el que se verificó (que CONTIENE las raíces del
    /// árbol que de verdad mueven); el resto lleva
    /// [`COMPROMISO_AUSENTE`], declarado. La era elige la fórmula del
    /// encadenado — ver [`chain_digest_v2`] — y la longitud en disco
    /// (137/169) la discrimina, precedente 49-A del store.
    pub compromiso: Option<Digest>,
    /// Resumen de la prueba, no la prueba entera.
    ///
    /// Guardar las pruebas completas serían ~62 KB por operación: mil
    /// transferencias son 59 MB. El resumen basta para atar la entrada a
    /// una prueba concreta, y quien quiera verificarla puede pedirla.
    ///
    /// ⚠️ **CORRECCIÓN (§271): en CUATRO vías delegadas no ata nada.**
    /// `mint.rs:131`, `freeze.rs:138`, `recovery.rs:166` y
    /// `governance.rs:162` llaman a `append` con `&[]`, así que las cuatro
    /// comparten un único valor —el de la prueba vacía— y no hay prueba
    /// concreta que pedir. Medido: `digest_of_proof(&[])` es
    ///
    /// ```text
    /// 74de079ffffa783f99bdd9ffa25e4112f8c395b141c83325e06ad3e10625cfa1
    /// ```
    ///
    /// Gradación, porque no todas fallan igual: `mint` y `recovery` al menos
    /// registran la transición —sus raíces difieren—; `freeze` y
    /// `governance` asientan `raiz, raiz` y no registran nada de lo que
    /// cambiaron. Ver las notas 78 y 79 del BACKLOG.
    ///
    /// ✅ **CERRADO en §278**, y con una corrección al recuento de arriba:
    /// eran **seis** los asientos con prueba vacía, no cuatro —faltaban
    /// `two_phase.rs:1285` (`MintToPending`, con el `append` partido en
    /// seis líneas) y `accounts.rs:243` (`OpenAccount`)—. Las cinco vías
    /// delegadas asientan ahora [`sello_de_autorizacion`] de su
    /// compromiso; `OpenAccount` asienta [`sello_sin_prueba`], que **no es
    /// el mismo valor**: declarar que no hay prueba y olvidarse de
    /// registrarla dejaron de ser indistinguibles.
    pub proof_digest: Digest,
    /// Resumen encadenado hasta esta entrada, inclusive.
    pub chain: Digest,
}

// ⚠️ §255: `as_digest` vive en `zk-ssl-hash`. Es una DECISION DE FORMATO
// —donde va el numero y con que se rellena—, y un verificador
// independiente tiene que usar LA MISMA, no una copia. Era privada aqui.
use zk_ssl_hash::as_digest;

/// ⚠️ §279: **la composicion del sello vive en `zk-ssl-hash`**, y aqui se
/// REEXPORTA. Misma razon que `as_digest` (§255): un verificador
/// independiente tiene que usar LA MISMA regla, no una copia, y
/// `zk-ssl-verify` no puede depender de esta capa. Los llamantes de
/// `zk_ssl::log::digest_of_proof` siguen funcionando sin cambio.
pub use zk_ssl_hash::{digest_of_proof, sello_de_autorizacion, sello_sin_prueba};

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

/// **Centinela de la era 2 para entradas sin compromiso autorizante**:
/// las vías con prueba real (Send, Claim, Burn, Refund, Migration) y
/// `OpenAccount`. Es un valor DECLARADO, no un hash — el precedente son
/// `VIEW_ID_LEGACY`/`LEAF_SALT_LEGACY` de 49-A. Distinto de cualquier
/// `commit_operation` con probabilidad abrumadora, y el test lo separa
/// de los sellos.
pub const COMPROMISO_AUSENTE: Digest = [BaseElement::new(0xC0_A9_2281); 4];

/// La fórmula de la **era 2** (§281): la de la era 1 entera, en su
/// posición, más el compromiso. Misma forma que `epoch_digest_v2`
/// (§275): la variante reusa a la vieja como subárbol, y el
/// desambiguador NO es un byte aquí — es la propia era de la entrada
/// (`compromiso: Option`), que en disco discrimina la longitud.
pub fn chain_digest_v2(
    seq: u64,
    kind: OpKind,
    root_old: Digest,
    root_new: Digest,
    proof_digest: Digest,
    previous: Digest,
    compromiso: Digest,
) -> Digest {
    native_merge(
        chain_digest(seq, kind, root_old, root_new, proof_digest, previous),
        compromiso,
    )
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

    /// Añade una entrada **de la era 2 sin compromiso autorizante**
    /// ([`COMPROMISO_AUSENTE`]). Es la vía de las clases con prueba real
    /// y de `OpenAccount`; las delegadas usan
    /// [`Self::append_con_compromiso`]. Firma intacta a propósito: los
    /// llamantes que no tienen compromiso no cambian.
    pub fn append(
        &mut self,
        kind: OpKind,
        root_old: Digest,
        root_new: Digest,
        proof: &[u8],
    ) -> Digest {
        self.append_con_compromiso(kind, root_old, root_new, proof, COMPROMISO_AUSENTE)
    }

    /// Añade una entrada de la **era 2** con su compromiso autorizante.
    ///
    /// ⚠️ El compromiso **entra en la cadena** ([`chain_digest_v2`]):
    /// manipularlo a posteriori es `TamperedEntry`, igual que cualquier
    /// otro campo. Prospectivo por necesidad (§280): las entradas
    /// anteriores quedan en la era 1 — su compromiso no existe.
    pub fn append_con_compromiso(
        &mut self,
        kind: OpKind,
        root_old: Digest,
        root_new: Digest,
        proof: &[u8],
        compromiso: Digest,
    ) -> Digest {
        let seq = self.entries.len() as u64;
        let proof_digest = digest_of_proof(proof);
        let previous = self.head();
        let chain =
            chain_digest_v2(seq, kind, root_old, root_new, proof_digest, previous, compromiso);
        self.entries.push(LogEntry {
            seq,
            kind,
            root_old,
            root_new,
            compromiso: Some(compromiso),
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
            // §281: la ERA de la entrada elige la fórmula. Un registro
            // mixto verifica entrada a entrada; los lotes de RFC-0002
            // etapa 2, cuando existan, se contemplarán AQUÍ — el salto
            // del encadenado por-entrada dentro de un lote es semántica
            // de esta comprobación, no de la serialización (nota 82).
            let esperado = match e.compromiso {
                None => chain_digest(
                    e.seq, e.kind, e.root_old, e.root_new, e.proof_digest, previous,
                ),
                Some(c) => chain_digest_v2(
                    e.seq, e.kind, e.root_old, e.root_new, e.proof_digest, previous, c,
                ),
            };
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
            // §281: la ERA de la entrada elige la fórmula. Un registro
            // mixto verifica entrada a entrada; los lotes de RFC-0002
            // etapa 2, cuando existan, se contemplarán AQUÍ — el salto
            // del encadenado por-entrada dentro de un lote es semántica
            // de esta comprobación, no de la serialización (nota 82).
            let esperado = match e.compromiso {
                None => chain_digest(
                    e.seq, e.kind, e.root_old, e.root_new, e.proof_digest, previous,
                ),
                Some(c) => chain_digest_v2(
                    e.seq, e.kind, e.root_old, e.root_new, e.proof_digest, previous, c,
                ),
            };
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

/// **Cabeza de época: lo que un testigo externo necesita para comparar.**
///
/// `CONFIANZA_RESIDUAL.md` B10.1. El README afirma que el operador «no puede
/// reescribir el historial en secreto», y `AUDITORIA.md` §76 establece que
/// esa garantía **solo vale para quien ya observó una cabeza anterior** —y
/// hoy nadie fuera del operador observa cabezas—.
///
/// Publicar esto permite que dos partes comparen su vista del sistema: si el
/// operador les mostró historiales distintos, sus cabezas difieren para el
/// mismo `seq`.
///
/// # ⚠️ Lo que esto NO es
///
/// **No es oponible.** No lleva firma, y el proyecto **no tiene ninguna
/// primitiva de firma** (§103.1). Dos cabezas contradictorias **detectan**
/// la inconsistencia; no prueban ante un tercero **quién** las emitió. Para
/// eso hace falta B15 —XMSS—, y elegir el esquema es una decisión de tesis:
/// `ed25519` no es post-cuántico (§103.2).
///
/// **No hay testigos.** Esto es una función; que alguien la recoja y compare
/// es operación, no código. `CONFIANZA_RESIDUAL.md` §10.1 lo dice sin
/// adornos: *la independencia de los testigos es un supuesto social, no
/// criptográfico*.
///
/// **Y no cierra la vista dividida**: la hace detectable **si** hay quien
/// compare. Certificate Transparency tuvo el patrón funcionando y su pieza
/// de comparación infradesplegada durante años.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochHead {
    /// Altura del registro. Monótona por construcción.
    pub seq: u64,
    /// Raíz del árbol de cuentas.
    pub accounts_root: Digest,
    /// Raíz del árbol de pendientes.
    pub pending_root: Digest,
    /// Raíz del árbol de congelados: pone **la política de congelación** bajo
    /// observación externa, no solo el dinero.
    pub frozen_root: Digest,
    /// Compromiso de todo el historial. Dos nodos con el mismo valor tienen
    /// la misma historia.
    pub chain_digest: Digest,
    /// Raíz del árbol de acuses de la época en curso (§275). Junto con
    /// `n`, viaja **firmada**: la promesa de retención deja de ser una
    /// nota del nodo y pasa a ser un valor que el titular puede exigir.
    /// El árbol y sus reglas: `vista_acuses` (nodo) y `acuses` (verify).
    pub acuses_root: Digest,
    /// Techo de cabezas retenidas que el nodo declara (§275, §121). El
    /// verificador lo lee de aquí; `N_MAX_CABEZAS` es asunto del nodo.
    pub n: u64,
    // ⚠️ **FALTA `verifier_hash`, y no por olvido.**
    //
    // `CONFIANZA_RESIDUAL.md` §2.2 lo propone con el mejor argumento de esa
    // propuesta: «quien puede actualizar el verificador es la **raíz de
    // confianza real** del sistema y nadie lo ve». Un operador que cambia el
    // verificador cambia **qué es una transición válida** — más poderoso que
    // cualquier operación del sistema.
    //
    // **No se puede rellenar hoy, y la razón NO es la que decía aquí**
    // (§246). Decía «el proyecto no tiene noción de reglas vigentes», que
    // suena a funcionalidad pendiente y **es casi circular**: `verifier_hash`
    // ES el mecanismo para tener esa noción.
    //
    // ⚠️ **La razón real: el AIR es CÓDIGO, no datos.** Lo que este campo
    // debe delatar es **qué es una transición válida**, y eso lo define el
    // AIR. Lo único hasheable en ejecución son las `ProofOptions` — y **un
    // operador puede cambiar el AIR dejando las `ProofOptions` idénticas**.
    //
    // Un `verifier_hash` así **no sería un campo vacío: sería un campo
    // CIEGO**, y eso es peor — un campo vacío se nota, uno ciego pasa
    // desapercibido **mintiendo justo sobre lo que existe para detectar**.
    //
    // ⚠️ Y las dos salidas están cerradas **por razones ajenas a este
    // fichero**:
    //
    // - **Hashear el fuente al compilar** no prueba que el binario se
    //   construyera de ese fuente. Sin compilación reproducible, el operador
    //   reporta el hash grabado y corre otra cosa: miente en el caso que
    //   importa.
    // - **El AIR como datos** —entrada 55— sí sería hasheable, y está parada
    //   por un motivo que no es esfuerzo: *una especificación escrita por
    //   quien escribió el circuito hereda sus puntos ciegos*, y debe
    //   escribirse **con la auditoría, no antes**.
    //
    // El criterio de §104.3 sigue valiendo, y ahora con el matiz que le
    // faltaba: un campo vacío sería peor que su ausencia; **uno ciego, peor
    // que uno vacío**.
    //
    // Backlog 54.
}

impl EpochHead {
    /// Resumen de la cabeza en un solo digest, para comparar de un vistazo.
    ///
    /// ⚠️ **Comparar digests dice que difieren, no dónde.** Para eso está
    /// [`TransitionLog::first_divergence`], que ya existe y localiza la
    /// entrada exacta en que dos historiales se separan.
    /// ⚠️ §255: **la composición vive en `zk-ssl-hash`**, no aquí.
    ///
    /// Un verificador que quiera comprobar que una raíz de cuentas
    /// pertenece a **una cabeza firmada** necesita componerla **exactamente
    /// igual**, y la única forma segura de garantizarlo es que **sea la
    /// misma función**. Dos composiciones divergirían **en silencio**.
    pub fn digest(&self) -> Digest {
        zk_ssl_hash::epoch_digest_v2(
            self.seq,
            self.accounts_root,
            self.pending_root,
            self.frozen_root,
            self.chain_digest,
            self.acuses_root,
            self.n,
        )
    }
}

#[cfg(test)]
mod tests_cabeza {
    use crate::tests_support::*;
    use crate::*;

    /// **Dos vistas divergentes producen cabezas distintas.**
    ///
    /// Es lo único que `EpochHead` demuestra, y es real: si el operador
    /// mostró historiales distintos a dos partes, sus cabezas difieren y
    /// **cualquiera de las dos puede notarlo** comparando con la otra.
    ///
    /// Hoy eso es imposible porque nadie fuera del operador ve cabezas
    /// (`AUDITORIA.md` §76).
    #[test]
    fn two_divergent_views_produce_different_heads() {
        let mut a = new_layer();
        let mut b = new_layer();

        // Misma historia: misma cabeza.
        let alice_a = open_and_fund(&mut a, SK_ALICE, 1_000_000);
        let _alice_b = open_and_fund(&mut b, SK_ALICE, 1_000_000);
        assert_eq!(
            a.epoch_head(zk_ssl_hash::as_digest(0), 0).digest(),
            b.epoch_head(zk_ssl_hash::as_digest(0), 0).digest(),
            "dos nodos con la misma historia deben tener la misma cabeza"
        );

        // La vista A recibe una operación que la B no ve.
        open_and_fund(&mut a, SK_BOB, 0);

        let ha = a.epoch_head(zk_ssl_hash::as_digest(0), 0);
        let hb = b.epoch_head(zk_ssl_hash::as_digest(0), 0);
        assert_ne!(
            ha.digest(),
            hb.digest(),
            "VISTA DIVIDIDA: dos historias distintas deben dar cabezas \
             distintas, o publicar la cabeza no detectaria nada"
        );
        assert_ne!(ha.seq, hb.seq, "y la altura las separa");
        let _ = alice_a;
    }

    /// ⚠️ **Y una cabeza NO dice quién la emitió.**
    ///
    /// Este test **fabrica una cabeza a mano** con valores inventados y
    /// comprueba que es indistinguible en tipo de una legítima. No lleva
    /// firma, así que nada la ata al operador.
    ///
    /// Se escribe explícitamente para que la ausencia quede en el código y
    /// no solo en un comentario: dos cabezas contradictorias **detectan**
    /// la inconsistencia; **no prueban ante un tercero quién mintió**.
    ///
    /// Cerrar esto exige una primitiva de firma que el proyecto **no tiene**
    /// (§103.1), y elegirla es una decisión de tesis: `ed25519` no es
    /// post-cuántico (§103.2). Backlog 53.
    #[test]
    fn a_head_does_not_say_who_issued_it() {
        let layer = new_layer();
        let legitima = layer.epoch_head(zk_ssl_hash::as_digest(0), 0);

        // Cualquiera puede construir esto. No hace falta ser el operador.
        let inventada = crate::log::EpochHead {
            acuses_root: zk_ssl_hash::as_digest(0),
            n: 0,
            seq: legitima.seq,
            accounts_root: [BaseElement::new(0xFA15A); 4],
            pending_root: legitima.pending_root,
            frozen_root: legitima.frozen_root,
            chain_digest: [BaseElement::new(0x3E17A); 4],
        };

        assert_ne!(
            legitima.digest(),
            inventada.digest(),
            "difieren en contenido, si"
        );

        // ⚠️ Pero nada en el tipo distingue una de otra: no hay firma que
        // verificar, ni emisor al que atribuirla.
        assert_eq!(
            std::mem::size_of_val(&legitima),
            std::mem::size_of_val(&inventada),
            "SIN FIRMA: una cabeza inventada es del mismo tipo que una \
             legitima. La vista dividida es DETECTABLE, no OPONIBLE."
        );
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
        l.entries[1].chain = chain_digest_v2(
            1,
            l.entries[1].kind,
            l.entries[1].root_old,
            d(99),
            l.entries[1].proof_digest,
            l.entries[0].chain,
            COMPROMISO_AUSENTE,
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


#[cfg(test)]
mod t1_chain_retroactivo {
    //! T1 - el argumento retroactivo del `chain_digest`, medido.
    //!
    //! Sostiene la decision 1-firma/min (entrada 53): una cabeza firmada en
    //! `n` ata tambien las `n-1` epocas anteriores. Estaba razonado, no
    //! medido. Si `t1_cabeza_ata_la_historia` falla, la ventana era de
    //! impunidad parcial y la decision se rehace a 1/s.
    use super::*;

    const N: u64 = 12;
    const K: u64 = 5; // la epoca de la mentira

    fn tag_valido() -> u8 {
        (0u8..=255)
            .find(|b| OpKind::from_tag_byte(*b).is_some())
            .expect("ningun tag de OpKind valido")
    }

    /// Historia de `n` entradas; con `mentira`, la K lleva otra prueba.
    fn historia(n: u64, mentira: bool) -> TransitionLog {
        let tag = tag_valido();
        let mut log = TransitionLog::new();
        for i in 0..n {
            let base: &[u8] = if mentira && i == K { b"mentira" } else { b"honesta" };
            let mut p = base.to_vec();
            p.push(i as u8); // cada entrada, distinguible
            log.append(
                OpKind::from_tag_byte(tag).unwrap(),
                as_digest(i),
                as_digest(i + 1),
                &p,
            );
        }
        log
    }

    #[test]
    fn t1_divergencia_localizada() {
        let a = historia(N, false);
        let b = historia(N, true);
        assert_eq!(a.first_divergence(&b), Some(K));
        for j in 0..N as usize {
            let iguales = a.entries()[j].chain == b.entries()[j].chain;
            assert_eq!(iguales, (j as u64) < K, "epoca {j}");
        }
    }

    #[test]
    fn t1_cabeza_ata_la_historia() {
        let a = historia(N, false);
        let cabeza_firmada = a.head();

        // El testigo recomputa desde los campos CRUDOS con la funcion
        // publica, sin fiarse de los `chain` almacenados.
        let recompute = |entradas: &[LogEntry], mentir_en: Option<u64>| -> Digest {
            let mut prev = [BaseElement::ZERO; 4];
            for e in entradas {
                let pd = match mentir_en {
                    Some(k) if e.seq == k => digest_of_proof(b"mentira-injertada"),
                    _ => e.proof_digest,
                };
                prev = chain_digest_v2(
                    e.seq,
                    OpKind::from_tag_byte(e.kind.tag_byte()).unwrap(),
                    e.root_old,
                    e.root_new,
                    pd,
                    prev,
            COMPROMISO_AUSENTE,
        );
            }
            prev
        };

        // Control: si esto falla, el mal planteado es el test.
        assert_eq!(
            recompute(a.entries(), None),
            cabeza_firmada,
            "la recomputacion no reproduce append: T1 mal planteado"
        );
        // La propiedad: alterar la epoca K falsifica la cabeza de N.
        assert_ne!(
            recompute(a.entries(), Some(K)),
            cabeza_firmada,
            "IMPUNIDAD: la cabeza de n NO ata la epoca k"
        );
        // Y la otra historia completa tampoco casa.
        assert_ne!(historia(N, true).head(), cabeza_firmada);
    }

    #[test]
    fn t1_verify_chain_caza_sustitucion() {
        let a = historia(N, false);
        // restaurado de copia, intacto: pasa
        assert!(TransitionLog::from_entries(a.entries().to_vec())
            .verify_chain()
            .is_ok());
        // restaurado con la entrada K sustituida: debe fallar
        let mut manipuladas = a.entries().to_vec();
        manipuladas[K as usize].proof_digest = digest_of_proof(b"mentira-injertada");
        assert!(
            TransitionLog::from_entries(manipuladas).verify_chain().is_err(),
            "verify_chain NO detecta la sustitucion en k"
        );
    }
}


#[cfg(test)]
mod t6_digest_inyectivo {
    //! T6 - entrada 58: las dos familias de colision, muertas y medidas.
    use super::*;
    use std::time::Instant;

    #[test]
    fn t6_ceros_finales() {
        for base in [&b""[..], b"x", b"quince_bytes_yy", b"dieciseis_bytes_", b"honesta"] {
            let mut con_cero = base.to_vec();
            con_cero.push(0);
            assert_ne!(digest_of_proof(base), digest_of_proof(&con_cero), "len {}", base.len());
            let mut ocho = base.to_vec();
            ocho.extend_from_slice(&[0u8; 8]);
            assert_ne!(digest_of_proof(base), digest_of_proof(&ocho));
        }
    }

    #[test]
    fn t6_aliasing_del_campo() {
        // Bajo el esquema viejo: un bloque que vale p se reducia a 0 y
        // colisionaba con ceros CON LA MISMA LONGITUD — la familia que el
        // arreglo de §116 (solo longitud) no habria matado.
        let p_le = 0xFFFF_FFFF_0000_0001u64.to_le_bytes();
        assert_ne!(digest_of_proof(&p_le), digest_of_proof(&[0u8; 8]));
    }

    #[test]
    fn t6_a_nivel_de_chain() {
        // La amenaza real de §116: dos pruebas casi-iguales, dos chains.
        let tag = (0u8..=255).find(|b| OpKind::from_tag_byte(*b).is_some()).unwrap();
        let mut a = TransitionLog::new();
        let mut b = TransitionLog::new();
        a.append(OpKind::from_tag_byte(tag).unwrap(), as_digest(1), as_digest(2), b"honesta");
        b.append(OpKind::from_tag_byte(tag).unwrap(), as_digest(1), as_digest(2), b"honesta\0");
        assert_ne!(a.head(), b.head(), "ceros finales aun invisibles al chain");
    }

    #[test]
    fn t6_coste_sobre_prueba_realista() {
        let prueba = vec![0xA5u8; 62_000];
        let t = Instant::now();
        let n = 20u32;
        for _ in 0..n { std::hint::black_box(digest_of_proof(&prueba)); }
        eprintln!("digest de 62 KB: {:?} de media (n={n}) — {} merges",
                  t.elapsed() / n, 62_000 / 16 + 1);
    }
}
/// **Los sellos de §278**: que atan, que no se pisan, y que las seis vías
/// que antes escribían el mismo byte hoy escriben lo que les toca.
#[cfg(test)]
mod tests_sello {
    use super::*;
    use crate::tests_support::*;
    use stark_experiment::native::derive_public_id;

    /// El valor que compartían las seis, y del que hay que alejarse.
    fn vacio() -> Digest {
        digest_of_proof(&[])
    }

    fn ultima(l: &crate::SovereignLayer) -> LogEntry {
        l.log.entries().last().expect("hay al menos una entrada").clone()
    }

    /// **Los tres dominios no se pisan, y el sello ata SU compromiso.**
    ///
    /// Sin la última aserción el sello sería decorativo: distinguiría
    /// «delegada» de «vacía» sin distinguir una delegada de otra.
    #[test]
    fn los_tres_dominios_no_se_pisan() {
        let op = as_digest(7);
        let otra = as_digest(8);
        let auth = digest_of_proof(&sello_de_autorizacion(&op));
        let auth_otra = digest_of_proof(&sello_de_autorizacion(&otra));
        let sin = digest_of_proof(&sello_sin_prueba());

        assert_ne!(auth, vacio(), "una delegada ya no puede parecer vacia");
        assert_ne!(sin, vacio(), "la ausencia declarada tiene valor propio");
        assert_ne!(auth, sin, "declarar y omitir no pueden coincidir");
        assert_ne!(auth, auth_otra, "el sello ata SU compromiso, no la clase");
    }

    /// **Las seis vías, una por una, contra el registro real.**
    ///
    /// El orden importa: la gobernanza va la última porque cambia el
    /// conjunto de custodios con el que se autorizan las demas, y la
    /// congelacion despues de la recuperacion para no bloquear la cuenta
    /// que esta aun por recuperar.
    #[test]
    fn las_seis_vias_atan_lo_que_les_toca() {
        let mut l = new_layer();

        // 1. OpenAccount — ausencia DECLARADA.
        // Apertura ANCHA a proposito: `open_account` esta `#[deprecated]`
        // desde 0.1.0 (clave de 64 bits) y este modulo no silencia el aviso.
        let idx = l.open_account_wide(wide_key(0x5E_11_0));
        assert_eq!(
            ultima(&l).proof_digest,
            digest_of_proof(&sello_sin_prueba()),
            "abrir cuenta declara que no hay prueba"
        );

        // 2. Mint delegada.
        let op = mint_commitment(&l, idx, 1_000);
        fund_delegated(&mut l, idx, 1_000);
        assert_eq!(
            ultima(&l).proof_digest,
            digest_of_proof(&sello_de_autorizacion(&op)),
            "la emision delegada ata su compromiso"
        );

        // 3. MintToPending delegada.
        let receptor = derive_public_id(BaseElement::new(0xB0B));
        let salt = salt_de(0x5A17);
        let op = mint_pending_commitment(&l, receptor, salt, 500);
        mint_to_pending_delegated(&mut l, receptor, salt, 500);
        assert_eq!(
            ultima(&l).proof_digest,
            digest_of_proof(&sello_de_autorizacion(&op)),
            "la emision a pendiente ata su compromiso"
        );

        // 4. Recovery delegada.
        let nueva = derive_public_id(BaseElement::new(0xBEEF_CAFE));
        let op = recovery_commitment(&l, idx, nueva);
        recover_delegated(&mut l, idx, nueva);
        assert_eq!(
            ultima(&l).proof_digest,
            digest_of_proof(&sello_de_autorizacion(&op)),
            "la recuperacion delegada ata su compromiso"
        );

        // 5. Freeze delegada — la que asienta `raiz, raiz` y antes no
        //    registraba NADA de lo que cambiaba.
        let op = freeze_commitment(&l, idx, true);
        set_frozen_delegated(&mut l, idx, true);
        assert_eq!(
            ultima(&l).proof_digest,
            digest_of_proof(&sello_de_autorizacion(&op)),
            "la congelacion delegada ata su compromiso"
        );

        // 6. Governance delegada — la ultima, por lo que dice el doc.
        let nuevas: Vec<BaseElement> = (0..5).map(|i| BaseElement::new(0xD0_0D_00 + i)).collect();
        let nueva_raiz = stark_experiment::circuit_threshold::build_custodian_set(&nuevas).0;
        let op = governance_commitment(&l, nueva_raiz);
        update_custodians_delegated(&mut l, nueva_raiz);
        assert_eq!(
            ultima(&l).proof_digest,
            digest_of_proof(&sello_de_autorizacion(&op)),
            "el cambio de custodios ata su compromiso"
        );

        // Y el negativo agregado: NINGUNA entrada del registro comparte ya
        // el valor de la prueba vacia.
        let vacias = l.log.entries().iter().filter(|e| e.proof_digest == vacio()).count();
        assert_eq!(vacias, 0, "ninguna entrada puede seguir asentando el vacio");
    }
}

#[cfg(test)]
mod tests_dos_eras {
    use super::*;
    use zk_ssl_hash::as_digest;

    fn d(x: u64) -> Digest {
        as_digest(x)
    }

    #[test]
    fn el_centinela_no_es_un_sello_y_las_dos_eras_encadenan_distinto() {
        // El centinela es un valor declarado: distinto de cualquier
        // compromiso real y del digest neutro.
        // un compromiso "real" cualquiera: para estas propiedades basta
        // un Digest distinto — la firma de commit_operation no viajo en
        // el terreno y a ciegas no se llama.
        let op = as_digest(0xFEED_2281);
        assert_ne!(COMPROMISO_AUSENTE, op);
        assert_ne!(COMPROMISO_AUSENTE, as_digest(0));
        // Y el compromiso PARTICIPA en la cadena: dos entradas iguales
        // salvo el compromiso divergen.
        let a = chain_digest_v2(0, OpKind::Mint, d(0), d(1), d(9), d(7), COMPROMISO_AUSENTE);
        let b = chain_digest_v2(0, OpKind::Mint, d(0), d(1), d(9), d(7), op);
        assert_ne!(a, b, "el compromiso no entra en la cadena");
        // Y la era 2 no colisiona con la era 1 de la misma entrada.
        assert_ne!(a, chain_digest(0, OpKind::Mint, d(0), d(1), d(9), d(7)));
    }

    #[test]
    fn un_registro_mixto_verifica_entrada_a_entrada() {
        // Era 1 simulada como llega del DISCO: entradas v1 recompuestas
        // con la formula vieja y compromiso None (from_entries no valida;
        // verify si).
        let mut v1 = Vec::new();
        let mut prev = [BaseElement::ZERO; 4];
        let mut root = d(100);
        for seq in 0..3u64 {
            let root_new = d(101 + seq);
            let pd = digest_of_proof(b"prueba-vieja");
            let chain = chain_digest(seq, OpKind::Mint, root, root_new, pd, prev);
            v1.push(LogEntry {
                seq,
                kind: OpKind::Mint,
                root_old: root,
                root_new,
                compromiso: None,
                proof_digest: pd,
                chain,
            });
            prev = chain;
            root = root_new;
        }
        let mut l = TransitionLog::from_entries(v1);
        // ...y la era 2 continua encima, por las DOS vias.
        let op = as_digest(0xFEED);
        l.append_con_compromiso(OpKind::Mint, root, d(200), b"sello", op);
        l.append(OpKind::Burn, d(200), d(201), b"prueba-real");
        assert_eq!(l.entries()[2].compromiso, None, "la era 1 se queda era 1");
        assert_eq!(l.entries()[3].compromiso, Some(op));
        assert_eq!(l.entries()[4].compromiso, Some(COMPROMISO_AUSENTE));
        l.verify(d(100)).expect("el registro mixto debe verificar");
        l.verify_chain().expect("y sin genesis tambien");
        // El negativo que da sentido al positivo (§266): manipular el
        // compromiso de una entrada era-2 es TamperedEntry.
        let mut m = l.clone();
        m.entries[3].compromiso = Some(COMPROMISO_AUSENTE);
        assert_eq!(m.verify_chain(), Err(LogError::TamperedEntry { at: 3 }));
    }

    #[test]
    fn la_ida_y_vuelta_del_disco_conserva_la_era() {
        // 137 <-> None y 169 <-> Some, por el store real.
        let mut l = TransitionLog::new();
        l.append(OpKind::Mint, d(0), d(1), b"p");
        let e2 = &l.entries()[0];
        let b2 = crate::store::log_entry_to_bytes(e2);
        assert_eq!(b2.len(), 169, "la era 2 pesa 169");
        let r2 = crate::store::log_entry_from_bytes(&b2).expect("169 legible");
        assert_eq!(r2.compromiso, e2.compromiso);
        assert_eq!(r2.chain, e2.chain);
        // Una era 1 fabricada a mano pesa 137 y vuelve como None.
        let pd = digest_of_proof(b"p");
        let v1 = LogEntry {
            seq: 0,
            kind: OpKind::Mint,
            root_old: d(0),
            root_new: d(1),
            compromiso: None,
            proof_digest: pd,
            chain: chain_digest(0, OpKind::Mint, d(0), d(1), pd, [BaseElement::ZERO; 4]),
        };
        let b1 = crate::store::log_entry_to_bytes(&v1);
        assert_eq!(b1.len(), 137, "la era 1 sigue pesando 137");
        let r1 = crate::store::log_entry_from_bytes(&b1).expect("137 legible");
        assert_eq!(r1.compromiso, None);
        assert_eq!(r1.chain, v1.chain);
        // Y una longitud intermedia se RECHAZA (un truncado no se
        // interpreta mal) — la regla del store, ejercitada aqui.
        assert!(crate::store::log_entry_from_bytes(&b2[..150]).is_err());
    }
}
