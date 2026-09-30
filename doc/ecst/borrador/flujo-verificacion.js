export const meta = {
  name: 'ecst-fact-check',
  description: 'Fact-check every claim of the ECST / HBS-STATE drafts against the Arqueo tree, the hbs-state repo and the literature, with an adversarial skeptic per cluster',
  phases: [
    { title: 'Verify', detail: 'one fact-checker per claim cluster' },
    { title: 'Refute', detail: 'one skeptic per cluster re-derives every verdict' },
  ],
}

const ARQ = '/home/user/Arqueo-open-conservation-proofs-for-closed-ledgers'
const HBS = '/home/user/atoranzo/hbs-state'

const COMMON = `
CONTEXT. The author of two repositories pasted a large set of AI-generated drafts (in Spanish) about:
 (a) hbs-state, an "index guard" for stateful hash-based signatures (XMSS/XMSS^MT) and its spec HBS-STATE v0.3;
 (b) Arqueo (formerly ZK-SSL), a closed-ledger engine with STARK proofs, XMSS-signed epoch heads, witnesses and an offline verifier;
 (c) a paper "Evidence-Carrying State Transitions (ECST)" that treats hbs-state and Arqueo as two instantiations of one architectural model;
 (d) an IETF Internet-Draft "draft-toranzo-hbs-state-00/-01", a CFRG announcement e-mail, and advice on the IETF process and Zenodo.
The drafts are known to mix true statements with fabricated numbers, invented code facts and mis-cited references. The goal is a single rigorous technical report where every statement is traceable. The Arqueo repository's own rule is "medir antes que afirmar": no figure without its measurement, distinguish measured / estimated / assumed.

SOURCES ON DISK (read-only; DO NOT modify any file; DO NOT run cargo — a release build is running in the background and would lock the target dir; you may use grep/sed/cat/find/python3 to read):
 - Arqueo repo: ${ARQ}   (key files: README.md, SECURITY.md, ARQUITECTURA.md, AUDITORIA.md [2.4 MB, grep it; entries are "§N"], BACKLOG.md, FIVE_BACKENDS.md, PAPER.md, GENAI.md, spec/NUCLEO.md, spec/PAQUETE.md, spec/RPC.md, spec/rfc/*.md, doc/*.md, crates/*/src/*.rs, tools/*)
 - hbs-state repo (shallow clone of github.com/atoranzo/hbs-state @ a960828): ${HBS}   (README.md, CITATION.cff, .zenodo.json, spec/HBS-STATE-v0.3.md, spec/state-vectors-v0.3.json, spec/verify-state.py, src/lib.rs, src/seed.rs, src/bin/subject.rs, tests/vectors.rs)
Network: rfc-editor.org, zenodo.org, datatracker are BLOCKED for direct fetch. WebSearch works (load it with ToolSearch "select:WebSearch,WebFetch"); WebFetch may work for some other domains (dblp.org, eprint.iacr.org, usenix.org, arxiv.org, doi.org) — try, and record which source actually confirmed each fact.

HOW TO REPORT. For EVERY claim listed below, return one finding with:
 - id: the claim id given below;
 - claim: the claim as the drafts state it (short, in Spanish or as quoted);
 - verdict: CONFIRMADA (the sources say exactly this), PARCIAL (partly right / needs a qualification), FALSA (contradicted by the sources), SIN_FUENTE (no source in either repo or literature; e.g. an invented number), NO_VERIFICABLE (cannot be checked from here, e.g. blocked network);
 - severity: alta if a report repeating it would publish something false or misleading about the code, the spec or a citation; media if imprecise; baja if cosmetic;
 - evidence: concrete pointers you actually opened: "path:line" (relative to the repo root, prefix hbs-state files with "hbs-state/"), "AUDITORIA.md §N", test names, or URLs/search results. Quote the decisive words when short;
 - correct_statement: in Spanish, the statement a rigorous report SHOULD make instead (or the same statement if CONFIRMADA), with its source.
Also return extra_facts: facts the drafts omit but that a rigorous ECST report needs from your area (each with evidence), e.g. the real formula, the real hash, the real limits declared by the repo.
Be exhaustive and precise; open the files, do not guess from names. If a claim bundles several sub-claims, split them into ids like H6a, H6b.`

const CLUSTERS = [
  {
    key: 'hbs-state',
    prompt: `CLUSTER H — hbs-state and the HBS-STATE v0.3 spec. Check each against ${HBS} (spec/HBS-STATE-v0.3.md, src/lib.rs, src/seed.rs, src/bin/subject.rs, spec/verify-state.py, spec/state-vectors-v0.3.json, tests/vectors.rs, README.md, CITATION.cff, .zenodo.json). Claims:
H1 "hbs-state es la implementacion de referencia de HBS-STATE v0.3 en Rust; guardian del indice para XMSS y XMSS^MT; no genera ni verifica firmas".
H2 API: IndexGuard::open(path); reserve() persiste con fsync ANTES de devolver el indice; reconcile(key) devuelve Reconciliation con cuatro variantes; reconcile_values existe. Check exact signatures and what reserve does step by step (write order, fsync of file and/or directory, permissions).
H3 The function R(C,K): InSync (key==counter, index=counter); CounterAhead (counter>key, key!=0, orphans=counter-key); KeyAtZero (counter>key, key==0, indeterminate=counter); KeyAhead (key>counter, unrecorded=key-counter, fatal). Precedence.
H4 "KeyAtZero ocurre en claves derivadas de semilla que vuelven a 0 al reiniciar" — and whether persisted-SK subjects can also reach it (spec section 0/1: hbs-lms wipes the key on exhaustion).
H5 "CounterAhead: normal tras un crash; se reservo pero el proceso murio antes de firmar" — versus spec: in seed-derived model CounterAhead is intra-process only.
H6 Policy claims in the drafts: (a) "Solo KeyAhead es fatal; el proceso debe detenerse"; (b) ECST draft: "KeyAtZero: el sistema exige intervencion o aislamiento"; (c) I-D: "Under a Persisted SK model, KeyAtZero MUST be classified as a protocol violation"; (d) I-D: "Under a Seed-Derived SK model, the runtime MUST advance key directly to counter before resuming"; (e) I-D: "CounterAhead: an implementation MAY advance key to match counter"; (f) later ECST sections: "Colapso controlado (C<K o corrupcion): destruye las claves operativas en RAM y marca el arbol HBS como agotado"; (g) "Recuperacion fast-forward (C>K): actualiza K<-C"; (h) ECST 11.7: "tras un corte K (memoria volatil) desaparece; la evaluacion se reformula como R(DurableEvidence) -> {InSync, CounterAhead, Indeterminate}". What does the spec actually say about policy (section 9 "Policy ... belongs to the owner", the judge in section 3)?
H7 fsync self-check: (a) "IndexGuard::open mide el tiempo de fsync al arrancar y se niega si es indistinguible de no persistir (tmpfs)"; (b) method "diferencia de latencia entre escrituras con y sin sync_all()"; (c) numbers "ext4 ~0.9 ms (382 veces mas lento), tmpfs ~0.002 ms" and in another draft "ext4 1.82 ms, tmpfs 0.03 ms"; (d) CFRG reply: "the measurement takes a statistical distribution over N iterations"; (e) I-D: "inspect /proc/mounts", "MUST NOT permit tmpfs, ramfs, procfs paths"; (f) "explicit operator override for ultra-fast NVMe". Find the exact thresholds, number of samples, error type FakePersistence, and whether thresholds are declared/derived.
H8 "Cero dependencias; el lector de vectores JSON esta escrito a mano (sin serde)".
H9 Conformance levels as the drafts define them: ECST: "N0 declaracion del modelo; N1 clasificacion; N2 calculo de campos derivados (indices huerfanos); N3 rechazo estricto y seguro de estados fatales (KeyAhead)"; I-D: "N3 (Judgment): enforces policy judgments (identifying fatal states and operator-defined actions)". Verifier in Python, subject binary hbs-state-subject, subject protocol.
H10 Test vectors in the I-D: {"version":"0.3","test_cases":[{"id":"TC-01","counter":10,"key":10,"expected_state":"InSync","fatal":false}, {"id":"TC-02","counter":15,"key":10,...,"orphans":5}, {"id":"TC-03","counter":5,"key":12,"KeyAhead","unrecorded":7}, {"id":"TC-04","counter":20,"key":0,"KeyAtZero","indeterminate":20}]} and draft-01 adding "algorithm":"LMS_SHA256_M32_H10". Compare with the real vector file format, ids (A1..A12, B1..B7), and whether any LMS vector exists. Are the invented values at least consistent with R?
H11 Parameter set XMSSMT-SHA2_40/8_256, OID 0x00000005, SK layout, and whether the drafts' implication that this layout is RFC 8391's is right (spec section 6).
H12 Provenance: "extraido directamente del sistema ARQUEO, donde lleva operando en entornos reales de firma" / README "in production for a year".
H13 DOI: drafts cite hbs-state as "Zenodo DOI 10.5281/zenodo.22993572", "v0.2.0 / v0.3 spec", "September 2026"; licences "CC0 / Dual MIT/Apache-2.0". Compare with CITATION.cff and .zenodo.json (and search the web for both DOIs).
H14 What the spec says about RFC 10033 (informational, September 2026, section 4 ACID, section 5 nine strategies, the verbatim quote) and NIST SP 800-208 (hardware module requirement, section 8.1 quote).
H15 "reusing an index lets an attacker forge with as low as 2^34 hash evaluations" — spec says what exactly, with which source (QRL curve, second reuse)?
H16 "HBS-STATE applies equally to LMS/HSS; the tuple (counter,key) and the four states are completely algorithm-agnostic" (CFRG reply) vs spec section 9 and the hbs-lms measurement.
H17 I-D requirements: "IndexGuard MUST acquire an exclusive non-blocking lock (flock/fcntl)", "multi-thread: atomic fetch-and-add or mutex", "hardware anchoring TPM NVRAM", "rollback of counter alone triggers KeyAhead on persisted-key systems". Which of these exist in hbs-state code or spec (spec section 9 says how the counter is persisted — fsync, ordering, locks, copies — is NOT covered)? Is the last one logically right (a rolled-back counter with a persisted key ahead gives key>counter)?
H18 Measured durability: "25 of 25 process deaths with not one signature ahead"; the retracted "13 of 25" claim (spec section 0). What exactly was killed and when.
H19 The spec's own status: DRAFT, derived from one implementation, needs a second independent subject; hbs-lms measured as second implementer (not subject); pq-xmss not a second implementer; B8 and family C declared without vectors.
H20 The drafts say the self-check "garantiza persistencia frente a la muerte abrupta del proceso, pero no frente a corte fisico de alimentacion". Confirm wording in README/spec.`,
  },
  {
    key: 'guardian',
    prompt: `CLUSTER G — Arqueo's own index guard and how HBS-STATE relates to Arqueo. Look at ${ARQ}/crates/zk-ssl-guardian (src/lib.rs, src/semilla.rs, Cargo.toml, tests), crates/zk-ssl-node/src/{firma_cabeza.rs,latido.rs,recepcion.rs,registro_recepcion.rs,main.rs}, doc/xmss-evaluacion.md, SECURITY.md, spec/rfc/0010-el-recibo-de-recepcion.md, and grep AUDITORIA.md for "guardian", "HBS-STATE", "KeyAtZero", "tmpfs", "fsync", "FakePersistence", "25 de 25", "muertes". Also compare with ${HBS}/src/lib.rs. Claims:
G1 "hbs-state fue extraido directamente de ARQUEO": is zk-ssl-guardian the origin? Same four states? Same names (Spanish names?)? Same fsync self-check? Is Arqueo's guard seed-derived (SK regenerated from seed, index 0 after restart)? What does Arqueo do at start-up in each state (especially KeyAtZero, which per the spec is ALWAYS the state after a real restart in the seed-derived model) — does it advance the key to the counter, refuse, or what?
G2 "en produccion desde hace un ano / lleva operando en entornos reales de firma". Compare with GENAI.md ("desde la primera sesion del proyecto, el 29 de julio de 2026") and any statement in the repo about production deployments. Is there any evidence of production use in Arqueo's docs?
G3 The fsync self-check in Arqueo: thresholds, samples, ratio, error type, where measured (AUDITORIA §?), the ext4/tmpfs numbers (0.907 ms / 0.002 ms / 382x vs 1.82 ms / 0.03 ms).
G4 Epoch head signed with XMSS: which parameter set, which crate and version (pinned with '='?), who reserves the index (guard) before signing, heartbeat (latido) meaning.
G5 "Testigos (witnesses) cofirman la cabeza y fijan la clave la primera vez que la ven" — how it works; is co-signature REQUIRED for a package to verify, or optional? ("Cada epoca ... requiere la co-firma de testigos" in the drafts).
G6 RFC-0010 / registro_recepcion.rs uses "CounterAhead" and cites HBS-STATE: what monotonic counter is being reconciled there, and how does it map onto R(C,K)? This matters because ECST claims R(C,K) generalises to other monotonic resources.
G7 Process-death test ("25 de 25") in Arqueo: where, how the child is killed, what is asserted.
G8 Does the guard use file locks (flock/fcntl) or thread synchronisation? Does it check permissions on read (0600)? Directory fsync?
G9 The drafts' ECST section: "HBS-STATE mitiga los falsos fsync en entornos virtuales o volatiles, pero la seguridad frente a perdidas de energia requiere NVRAM/HSM" — check against SECURITY.md wording about power loss / custody of the key ("custodia de clave comprobada, no solo declarada" is listed as missing in README).
G10 "el nodo firma la cabeza con XMSS: firma post-cuantica"; any caveat the repo declares about the xmss dependency (pre-release, RustCrypto issue, OID defect).`,
  },
  {
    key: 'history',
    prompt: `CLUSTER L — Arqueo's transition log, epoch head, history, evidence package, offline verification and anchoring. Look at ${ARQ}/crates/zk-ssl/src/log.rs (including the test t1_cabeza_ata_la_historia), crates/zk-ssl-hash/src/lib.rs (epoch_digest, native_merge, as_digest, path_root, domain constants), crates/zk-ssl-node/src/firma_cabeza.rs, crates/zk-ssl-verify/src/*.rs (mmr.rs, inclusion.rs, reverificacion.rs, consumos.rs, acuses.rs, recibos.rs, lib.rs, main.rs), spec/NUCLEO.md, spec/PAQUETE.md, spec/RPC.md (zkssl_consistencyProof, zkssl_inclusionReceipt, zkssl_ackPath), spec/rfc/0003*, 0004*, 0006*, 0007*, doc/ANCLAJE_EXTERNO.md, doc/CONFIANZA_RESIDUAL.md, doc/KIT.md, README.md. Claims:
L1 Draft formula for the chained log: "H_i = H(seq_i, op_i, root_{i-1}, root_i, H(pi_i), H_{i-1})" and the ECST formula "H_i = H(H_{i-1} || S_{i-1} || S_i || O_i || H(Pi_i))". What is the REAL formula (fields, order, encoding, hash function — Rescue? Blake3? SHA-256? — and domain separation)? Does the chained entry include a digest of the STARK proof at all? Does it include authorization material A_i?
L2 "La cabeza de epoca ata las raices del estado en reposo, el registro encadenado de transiciones y el arbol de consumos publicados" — real composition of the head (which version, v5/v6?), what is signed.
L3 Test t1_cabeza_ata_la_historia: what it actually does and asserts (parameters, number of entries, what is altered). Draft claims: "se ejecuto sobre N=100 epocas, se altero un unico byte de la prueba Pi_k en la epoca k=10, H_100' != H_100, detectado sin re-verificar ZK". Confirm or refute each number.
L4 "Historia no reescribible, con prueba de extension (consistency proof)": structure (MMR?), what the verifier checks, which RPC method.
L5 "Verificacion offline con el nodo apagado": what exactly the evidence package contains (signed head, ack with path, co-signatures, ...), what zk-ssl-verify checks, and whether it re-verifies STARK proofs (Verify(Pi_i)) or only commitments/signatures. Are STARK proofs retained/published anywhere so that a third party can re-verify them later? (The ECST draft assumes "la prueba Pi_i se conserva como evidencia adjunta recuperable" and "el auditor recalcula la cadena de H_0 a H_N y ejecuta Verify(Pi_i) para cada transicion".)
L6 Draft: "Crecimiento del log de estado ~256 bytes por transicion (almacenando solo H(Pi_i))", "reduccion del 99.6%", "auditoria historica offline ~125,000 bloques/s", "sobrecarga de encadenamiento < 0.002 ms". Is any of this measured anywhere? Compute the real per-entry size of the chained log from the code if possible.
L7 Anchoring: the ECST drafts state the non-rewritability needs an externally authenticated head. What does the repo say: TOFU of witnesses, "le falta un ancla anterior al primer encuentro", doc/ANCLAJE_EXTERNO.md status?
L8 Draft: "una vez que una transicion es publicada y atada a una cabeza de epoca, la historia se vuelve inalterable y auditable de forma independiente con el nodo apagado" — qualify precisely.
L9 Draft: "un tercero puede tomar el ejecutable del verificador independiente, importar los vectores de conformidad y la cabeza publicada, y verificar toda la secuencia de evidencia historica sin consultar el nodo" — is it the whole history or per-package claims? Is there a release (arqueo-verify-v0.2.0) and a kit?
L10 Draft: "Arqueo extiende CT (RFC 9162)" / README "responsabilidad demostrable, al modo de Certificate Transparency" — how the repo positions itself vs CT.
L11 Which parts are "NUCLEO" frozen (RFC-0005 proposed) and the protocol version zkssl/0.4.
L12 Draft Theorem 1 as applied to Arqueo: altering Pi_k changes H_n. If the chain does NOT include a proof digest, what DOES the chain bind (roots? operation descriptors?) and therefore what would altering a proof change?`,
  },
  {
    key: 'custody',
    prompt: `CLUSTER C — two-phase payments, key custody, what proofs reveal, conservation, single use / nullifiers, recovery, governance/threshold, freeze. Look at ${ARQ}/crates/zk-ssl/src/{two_phase.rs,client.rs,recovery.rs,consumo.rs,governance.rs,mint.rs,freeze.rs,burn.rs,accounts.rs,lib.rs,audit.rs,instrumento_pago.rs}, crates/stark-experiment/src/{circuit_recovery.rs,circuit_recovery_climb.rs,circuit_threshold.rs,circuit_threshold_single.rs,circuit_governance.rs,circuit_send_v2.rs,circuit_claim_v2.rs,circuit_freeze.rs}, crates/zk-ssl-sdk/src/*, spec/RPC.md (the API principle, dev_* methods), spec/rfc/0006-consumo-publicado.md, spec/rfc/0009-lo-que-revela-una-prueba.md, SECURITY.md (esp. section 3.bis), README.md, doc/USE_CASES.md, and grep AUDITORIA.md for §32, §36, §521, §538. Claims:
C1 "send_materials() proporciona unicamente materiales publicos y caminos de autenticacion; prove_send() se ejecuta exclusivamente en la maquina del cliente con la clave secreta; apply_send() recibe y verifica la prueba sin haber tenido acceso jamas a la clave privada".
C2 "Custodia cero garantizada por tipos: apply_send() solo acepta la prueba y compromisos publicos, haciendo tecnicamente imposible que la clave secreta sea transmitida a la capa del ledger". Is there a type-level barrier? Is there ANY other path in the node/layer where the node receives or holds a spend key (e.g. layer-side send/claim that prove on the node, sandbox/tests_support, dev_* RPC methods, the CLI simulate)? tools/check_publicadas.py mentions "la via de la capa send/claim" vs "la via DOCUMENTADA send_materials -> client::prove_send -> apply_send" — explain.
C3 History of key exposure: README says "la clave de gasto no viaja por la API y, desde el §538, tampoco sale literal en la prueba (RFC-0009 E3b-2, zkssl/0.4); entre el §521 y el §538 la prueba la publicaba". Confirm with SECURITY.md 3.bis and the RFC; the ECST drafts never mention it.
C4 Conservation: drafts say "Circuito STARK que demuestra que la suma de saldos de salida es identica a la de entrada sin revelar los saldos individuales" and "el auditor obtiene certeza matematica sin acceso a las bases de datos". What does the repo actually prove and to whom (README: "Prueba conservacion, no solvencia"; "El operador ve todos los saldos"; "suministro = saldos + en vuelo")? Is conservation a property proven per transition in-circuit, checked by the verifier over published data, or both? What does RFC-0009 say a proof reveals to a third party?
C5 Nullifiers: drafts say "Deteccion de Reutilizacion de Etiquetas (Nullifiers): garantiza que ningun activo pueda ser gastado dos veces en el arbol de consumos". README: uso unico within a ledger; across ledgers "detecta, no previene"; the nullifier tree was RETIRED (§32, §36). What replaced it (consumption tree, RFC-0006)? Precise statement.
C6 recovery.rs / circuit_recovery*: drafts claim "circuito STARK que exige conservacion estricta de saldo (S_B = S_A), vaciado atomico de la clave antigua, incremento de un contador publico de recuperacion (Count_B = Count_A + 1) y doble autorizacion administrativa; entradas publicas: raices de la identidad antigua A y nueva B". Check each sub-claim against the circuit and the layer code: which constraints exist, what is public, who authorises (threshold? governance signatures? operator?), and whether recovery is in production path.
C7 Draft: "Al forzar que el contador de recuperacion sea publico, el historial de gobernanza queda sujeto a la misma rigidez matematica que las transacciones, eliminando los canales laterales administrativos" and "los circuitos ZK aplicados a primitivas administrativas cierran la brecha de confianza de los sistemas de gobernanza descentralizada". What admin powers does the operator keep (mint, burn, freeze, recovery, governance)? Are there "congelaciones" without expiry policy (README: "una politica de caducidad para las congelaciones" missing)?
C8 Draft ECST threat-model Claim 2: "la clave secreta del usuario permanece en la maquina del cliente, haciendo tecnicamente imposible que el operador emita transiciones no autorizadas sin una firma/prueba valida". Scope it: which transitions can the operator emit by itself (mint/freeze/recovery/governance)?
C9 Draft: "Arqueo asume un dictador de bloque"; "respeto las reglas del contrato inteligente"; "infraestructuras financieras centralizadas, registros de propiedad". Does Arqueo have smart contracts? What use cases does doc/USE_CASES.md list?
C10 ECST-R proposals: "extendemos Arqueo para que el estado este enmascarado mediante Pedersen Commitments o arboles de Merkle privados" — compare with CONTRIBUTING.md ("No encaja ... supuestos no post-cuanticos contra la tesis del nucleo") and the hash used for commitments in Arqueo (salted Merkle? RFC-0009 "MerkleConSal"). Is anything of ECST-R (threshold spend keys, TEE, key migration state machine Healthy/Suspected/Quarantined/Revoked/Migrated, anomaly detection) implemented?
C11 "Prueba STARK generada en la maquina del pagador, sin ceremonia y sin curvas" — confirm.
C12 Draft ECST "4.1 ... El operador del nodo puede censurar una transaccion recibida u omitirla antes de su inclusion sin dejar rastro" — confirm; and what RFC-0010 (recibo de recepcion, PROPUESTO) is meant to change about that.`,
  },
  {
    key: 'measures',
    prompt: `CLUSTER M — measurements, formal verification status, backends and positioning inside the Arqueo repo. Look at ${ARQ}/AUDITORIA.md (grep for the section headers "§130", "§131", "§229", "§304", "§83", "§226" and for words like "EPYC", "Ryzen", "Intel", "CPU", "maquina", "nucleos", "ms", "KiB", "prueba de pago", "verificacion"), crates/zk-ssl/src/metrics.rs (constants PUBLICADA_PAGO_MIN_B/MAX_B and the tests la_cifra_publicada_sigue_siendo_la_medida, los_dos_lados_del_pago_atan_la_banda), tools/check_publicadas.py, FIVE_BACKENDS.md, PAPER.md / PAPER_EN.md, doc/historia/PERFORMANCE.md, doc/VERIFICACION_FORMAL.md, doc/fv/mapa_fv_capas.md, doc/fv/*.py, crates/nova-experiment/src/*, SECURITY.md, tools/canon.sh header, CONTRIBUTING.md. Claims (all from the ECST drafts' "Evaluacion" sections):
M1 "Entorno de pruebas estandarizado: AMD EPYC 7763, 64 cores, 128 GB RAM, NVMe SSD". Is any such machine mentioned anywhere? What machine(s) did the repo's own measurements use?
M2 "Tamano de la prueba ZK (Pi_i) ~62.4 KB". Real sizes: proof sizes per payment/transition (the published band 66,739 / 66,692 bytes? per what?), STARK 36.7 KB in PAPER, etc. State what each number measures.
M3 "Tiempo de proving 412 ms"; "Tiempo de verificacion 8.1 ms". Real measured proving/verification times, with dispersion and the section where they are recorded (§130, §131).
M4 "Sobrecarga de encadenamiento hash < 0.002 ms por transicion"; "Velocidad de auditoria historica offline ~125,000 bloques/seg"; "Crecimiento ~256 bytes por transicion"; "reduccion del 99.6%". Any measurement? Also the node ceiling via RPC (§229) — what is it?
M5 fsync: "ext4 1.82 ms vs tmpfs 0.03 ms" and "0.907 ms vs 0.002 ms (382x)". Any record in AUDITORIA?
M6 Drafts: "Queda como trabajo futuro la verificacion formal TLA+ de R(C,K)", "Falta de verificacion formal mecanizada". What formal verification work exists in the repo (doc/VERIFICACION_FORMAL.md, doc/fv — SMT export, interpreters)? What is and is not verified? Is there a formal AIR specification (SECURITY.md 3.1 says there is none)?
M7 IVC/PCD: the drafts say "ECST evita la verificacion recursiva intra-circuito, reduciendo el coste" and "IVC impone una sobrecarga computacional masiva". The repo has crates/nova-experiment and FIVE_BACKENDS.md: what did it measure or find about Nova/folding (e.g. whether Nova was usable, why it was not chosen)? Is there any measured comparison ECST-vs-IVC? (If none, a report must say the comparison is qualitative.)
M8 "Post-quantum": what does the repo claim about post-quantum security of its STARK parameters (FRI, hash — Rescue/Blake3 — security bits, conjectured vs proven) and of XMSS? Any caveat?
M9 "Evaluacion: las evaluaciones no deben reportar medias"; the repo's actual practice (median, dispersion across batches "tandas", §131).
M10 "No auditado por terceros"; "dependencia xmss pre-release clavada con '='".
M11 Tests in debug vs release (CONTRIBUTING: 188 pass/93 fail in debug vs 307/0 in release on 26-08-2026) — relevant to reproducibility claims.
M12 The six Zenodo deposits of Arqueo and their DOIs (README "Publicacion"), for the report's bibliography; and doc/preprints/ERRATA.md's rule "una cifra publicada que se corrige no se borra, se marca".
M13 AUDITORIA's section listing "los puntos donde el autor tiene menos confianza": summarise the items (they belong in an honest limitations section).
M14 The drafts assert "t1_cabeza_ata_la_historia ... valida el Teorema 1" and "el banco de vectores de HBS-STATE verifica que implementaciones dispares pueden ser sometidas al mismo analisis" (Arqueo's reference to other implementations). Check what Arqueo's docs say about conformance vectors and a second implementation of the Arqueo protocol (spec/vectors, conformance command).`,
  },
  {
    key: 'biblio',
    prompt: `CLUSTER B — bibliography and literature (use WebSearch; try WebFetch on dblp.org / eprint.iacr.org / usenix.org / arxiv.org / doi.org / ietf.org mirrors when not blocked). For each reference give the CORRECT full citation (authors with full names, title, venue, year, pages, DOI) and say what exactly was wrong in the drafts. Claims:
B1 RFC 8391 "Huelsing, A., Butin, D., Gazdag, S., Rijneveld, J., and A. Mohassel" / BibTeX "Azam Mohassel", May 2018, DOI 10.17487/RFC8391. (Real 5th author: Aziz Mohaisen?) Also the stream/status (IRTF CFRG, Informational).
B2 RFC 8554 "McGrew, D., Curcio, M., and S. Fluhrer", April 2019; BibTeX has a garbage character "McGrew秩序".
B3 RFC 9162 cited as "Laurie, B., Lin, C., Kasper, E., & Messeri, E." and BibTeX "Ben Laurie and Adam Langley and Eran Kasper and Emilia Messeri", 2021. Real authors? (Also RFC 6962 authors, for contrast.)
B4 RFC 10033: exact title, authors, date, category, stream, DOI, originating draft (draft-ietf-pquip-hbs-state?), and whether its section 4 requires ACID properties and section 5 lists nine state-management strategies; the verbatim sentence "in particular, this enables implementing rollback resistant counters, which can be difficult to achieve in a software-only fashion".
B5 NIST SP 800-208: authors "Cooper, Apon, Dang, Davidson, Dworkin, Miller", October 2020, DOI 10.6028/NIST.SP.800-208; the drafts' URL target. Also whether a revision enabling key export is in progress (the spec mentions it).
B6 STARK: Ben-Sasson, Bentov, Horesh, Riabzev, IACR ePrint 2018/046 — correct? (Also the CRYPTO 2019 version.)
B7 Nova: drafts cite "Kothapalli, Setty & Tzialla (2022). Nova: Recursive Zero-Knowledge Proofs without Trusted Setup. CCS 2022, pp. 2161-2174, DOI 10.1145/3548606.3560610" and BibTeX first name "Tsvitcha". Find the real venue (CRYPTO 2022?), title, full names, pages/DOI; and identify what DOI 10.1145/3548606.3560610 really points to.
B8 IVC: drafts cite "Valiant, P. (2008). Incrementally verifiable computation or proof of execution. FOCS 2008, pp. 137-146, DOI 10.1109/FOCS.2008.82". Find the real venue (TCC 2008, LNCS 4948?) and title; identify what the FOCS DOI actually is.
B9 PCD: drafts cite "Chiesa & Tromer (2013). Proof-carrying data and incrementally verifiable computation. ITCS 2013, pp. 310-321, DOI 10.1145/2422436.2422472". Real Chiesa-Tromer PCD paper (ICS 2010 "Proof-Carrying Data and Hearsay Arguments from Signature Cards"?) and what the ITCS DOI is.
B10 Pillai et al. OSDI 14 "All file systems are not created equal: On the fidelity of crash-consistency applications", pp. 433-448, with "Arpaci-Dusseau, G. P." Real title/authors/pages.
B11 Chidambaram et al. "Optimistic crash consistency", SOSP 2013, pp. 361-377, DOI 10.1145/2517349.2517373. Real pages/DOI.
B12 POSIX fsync: "IEEE Std 1003.1-2017" — correct citation.
B13 Merkle, "A certified digital signature", CRYPTO '89, LNCS 435, pp. 218-238, DOI 10.1007/0-387-34805-0_21.
B14 "EIP-8310": does an Ethereum Improvement Proposal 8310 exist, and is it about stateful signature state / high-water marks as the drafts claim ("EIP-8310 y otras especificaciones recomiendan marcas de agua altas")?
B15 RFC 3552 (Rescorla, Korver, July 2003, BCP 72), RFC 8126 (Cotton, Leiba, Narten, June 2017, BCP 26), RFC 2119, RFC 8174, FIPS 202 (August 2015, DOI 10.6028/NIST.FIPS.202).
B16 Related work the drafts OMIT but a reviewer would expect, with correct citations: McGrew, Kampanakis, Fluhrer, Gazdag, Butin, Buchmann "State Management for Hash-Based Signatures" (SSR 2016, ePrint 2016/357); Bruinderink & Huelsing "Oops, I did it again - Security of One-Time Signatures under Two-Message Attacks" (SAC 2017) — the real source for forgery cost after WOTS/XMSS index reuse; Haber & Stornetta "How to time-stamp a digital document" (J. Cryptology 1991); Crosby & Wallach "Efficient Data Structures for Tamper-Evident Logging" (USENIX Security 2009); Schneier & Kelsey "Secure audit logs to support computer forensics" (ACM TISSEC 1999); Haeberlen, Kouznetsov, Druschel "PeerReview: Practical Accountability for Distributed Systems" (SOSP 2007); Laurie "Certificate Transparency" (CACM 2014) or RFC 6962; Melara et al. "CONIKS" (USENIX Security 2015); Bitansky, Canetti, Chiesa, Tromer "Recursive composition and bootstrapping for SNARKs and proof-carrying data" (STOC 2013); Kothapalli & Setty etc. Verify each exists with the correct details.
B17 The "2^34 hash evaluations after index reuse" figure: find its real source (QRL?), and what the literature (Bruinderink-Huelsing) says about forgery after two-message attacks on WOTS+/XMSS.
B18 Zenodo: search the web for "10.5281/zenodo.22993572" and "10.5281/zenodo.22980547" (hbs-state) and the Arqueo DOIs 10.5281/zenodo.21736125, 21736082, 21905595, 22078086, 22077991, 22076721 — report what can be confirmed.
Return each as a finding (ids B1..B18, split as needed), and put the full corrected citations (plain text AND a BibTeX entry string) in extra_facts, one per reference, with the confirming URL(s) as evidence.`,
  },
  {
    key: 'ietf-logic',
    prompt: `CLUSTER P — (1) the IETF/IRTF process advice and the Internet-Draft as written, (2) the logical/formal claims of the ECST drafts. Use WebSearch (ietf.org, irtf.org, rfc-editor.org search snippets; direct fetch of rfc-editor/datatracker is blocked — try WebFetch on www.ietf.org or authors.ietf.org pages) and read ${HBS} and ${ARQ} as needed.
PART 1 — process & I-D:
P1 "CFRG (Crypto Forum Research Group), el brazo de investigacion criptografica del IETF": CFRG is an IRTF research group, not an IETF WG. Confirm.
P2 "Via Grupo de Trabajo: el CFRG adopta el borrador, que pasa a llamarse draft-ietf-cfrg-hbs-state" — real naming of CFRG documents (draft-irtf-cfrg-*).
P3 "El IESG aprueba formalmente el documento" for a CFRG document — IRTF stream documents are approved by the IRSG (with IESG conflict review). Confirm the IRTF stream process (RFC 5743, RFC 7209?).
P4 "Via envio individual: un Area Director de Seguridad puede patrocinarlo" — AD-sponsored is IETF stream; independent submissions go to the ISE (RFC 4846). Which paths exist for an individual informational document?
P5 Which IETF group is the natural home for HBS-STATE? RFC 10033 came from which WG (PQUIP?) — search. LAMPS scope. So the drafts' recommendation "CFRG and LAMPS" vs PQUIP.
P6 "Un Internet-Draft tiene una validez de 6 meses" (true?) ; "todo el proceso es gratuito"; "asistir en persona 700-1000"; "la participacion remota es gratuita (o coste simbolico)" — current IETF meeting fees and remote participation fee/waiver. "Las decisiones se toman en las listas de correo" (RFC 2418/ IETF consensus in mailing lists) — confirm.
P7 Tooling: "gem install kramdown-rfc2629" and command "kramdown-rfc2629 file.md > file.xml; xml2rfc --text --html". Current gem name (kramdown-rfc) and command (kramdown-rfc / kdrfc)? ipr trust200902 OK? For IRTF stream submissions, is "workgroup: Crypto Forum Research Group" appropriate for an individual draft (-00 before adoption)? Are there structural problems in the pasted I-D: the JSON code fence in the Test Vectors appendix is opened with three backticks and never closed; the ASCII decision diagram was collapsed into a single line; the references section was written as plain "[*RFC2119]" lines instead of the YAML reference blocks; the author email is a placeholder "autor@ejemplo.com"; "RFC10033" listed as informative while text relies on it; "normative: NIST-SP800-208" OK; the I-D references "Section 3.1 equation" etc.
P8 The CFRG announcement mail links "https://github.com/atoranzo/hbs-state" and "v0.2.0" and "CC0 / Dual MIT/Apache-2.0" and claims the datatracker URL exists — note that the draft has not been submitted (no evidence) and so the URL cannot be confirmed.
PART 2 — logic (read the repos to ground each point):
P9 Draft Theorem 1 ("Atado retroactivo bajo resistencia a colisiones": changing Pi_k to Pi_k' makes H_n' != H_n except with negligible probability) — is the statement and proof sketch sound? What hypotheses are missing (canonical encoding, the digest actually being in the chain, authenticated head, fixed-length/unambiguous concatenation)? Relation to Haber-Stornetta hash-chain timestamping.
P10 "HistoryOK does not imply CryptoValid" — sound. "CryptoValid + HistoryOK gives a basis to reconstruct the sequence" — under what data-availability hypothesis?
P11 Statement "IVC ... el coste de verificacion O(log n) en el auditor permanece constante" and "tiempo de validacion sublineal respecto al tamano de la transicion" — internally inconsistent? What is true for STARK verification (polylog in trace length) and for an auditor verifying N separate proofs (linear in N) vs one IVC proof (constant/independent of N)? This is the real trade-off ECST must state.
P12 ECST-R claims: "un salto de instruccion inducido por hardware no sirve de nada porque el verificador externo computara Verify(Pi_i)=0" (fault injection) — true only if proofs are published and actually re-verified by a third party; and the node's own acceptance could be glitched. "ECST traslada la carga de la prueba desde el hardware a la verificacion matematica" — qualify. "ningun ataque microarquitectonico puede falsificar ... sin conocer las claves" — the payer's key is on the client machine; a side channel on the client breaks authorship.
P13 "El umbral eleva exponencialmente la complejidad del ataque" (later retracted in the drafts) — correct statement.
P14 "Pedersen commitments" in a post-quantum design — Pedersen binding relies on discrete log, not PQ.
P15 "S in U => NoSign" and "KeyAtZero => NoSign" vs HBS-STATE: in the seed-derived model a real restart ALWAYS yields KeyAtZero; if KeyAtZero implied NoSign the Arqueo node could never sign after a restart. What must the correct rule be (fail closed = treat 0..counter-1 as consumed and continue from counter)? Check what Arqueo's guard actually does.
P16 The drafts' comparison table: "Custodia Cero en API: Garantizada por tipos"; "Recursion ZK intra-circuito: IVC Si (Requerida)"; "Verificacion offline: Ledger convencional requiere sincronizacion"; "Semantica de estado fisico: explicita" — which cells are defensible?
P17 "ECST no es IVC: ECST no compone la validez de Pi_{i-1} dentro de Pi_i; compromete su representacion mediante el digest" — does Arqueo actually commit proof digests at all (see cluster L)? If not, what is the honest mapping of ECST's H(Pi_i) onto Arqueo?
P18 Anything else logically wrong or self-contradictory across the draft versions (e.g. section 3 of one version says C<K is "colapso", section 11.7 says K is RAM and disappears; "8.5/10" self-evaluations; "cambio de paradigma", "revolucionario", "establece un nuevo estandar").`,
  },
]

const FINDINGS = {
  type: 'object',
  properties: {
    cluster: { type: 'string' },
    findings: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          id: { type: 'string' },
          claim: { type: 'string' },
          verdict: { type: 'string', enum: ['CONFIRMADA', 'PARCIAL', 'FALSA', 'SIN_FUENTE', 'NO_VERIFICABLE'] },
          severity: { type: 'string', enum: ['alta', 'media', 'baja'] },
          evidence: { type: 'array', items: { type: 'string' } },
          correct_statement: { type: 'string' },
        },
        required: ['id', 'claim', 'verdict', 'severity', 'evidence', 'correct_statement'],
      },
    },
    extra_facts: {
      type: 'array',
      items: {
        type: 'object',
        properties: { fact: { type: 'string' }, evidence: { type: 'array', items: { type: 'string' } } },
        required: ['fact', 'evidence'],
      },
    },
  },
  required: ['cluster', 'findings', 'extra_facts'],
}

const ADJ = {
  type: 'object',
  properties: {
    cluster: { type: 'string' },
    adjudications: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          id: { type: 'string' },
          agree: { type: 'boolean' },
          final_verdict: { type: 'string', enum: ['CONFIRMADA', 'PARCIAL', 'FALSA', 'SIN_FUENTE', 'NO_VERIFICABLE'] },
          reason: { type: 'string' },
          evidence: { type: 'array', items: { type: 'string' } },
          correct_statement: { type: 'string' },
        },
        required: ['id', 'agree', 'final_verdict', 'reason', 'evidence', 'correct_statement'],
      },
    },
    extra_facts_rejected: { type: 'array', items: { type: 'string' } },
  },
  required: ['cluster', 'adjudications', 'extra_facts_rejected'],
}

const results = await pipeline(
  CLUSTERS,
  c => agent(`${COMMON}\n\n${c.prompt}`, { label: `verify:${c.key}`, phase: 'Verify', schema: FINDINGS }),
  (found, c) => found && agent(`${COMMON}

You are the ADVERSARIAL SKEPTIC for cluster "${c.key}". A first fact-checker produced the findings below. Your job is to try to REFUTE each verdict independently: open the cited evidence yourself, look for evidence the checker missed (including evidence that the drafts were RIGHT), and check that each correct_statement is itself fully supported and not overclaiming. Do not trust the checker's quotes; re-open the files / re-run the searches. For EVERY finding id return an adjudication (agree=true only if you reproduced the evidence; otherwise give the final verdict you can support, with your evidence and a corrected correct_statement). Also list any extra_facts you found to be wrong or unsupported (by their text) in extra_facts_rejected.

The original claim list for this cluster was:
${c.prompt}

FINDINGS TO ADJUDICATE (JSON):
${JSON.stringify(found, null, 1)}`, { label: `refute:${c.key}`, phase: 'Refute', schema: ADJ }).then(adj => ({ cluster: c.key, found, adj })),
)

return results.filter(Boolean)
