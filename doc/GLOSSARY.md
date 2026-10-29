# Glossary — the house's words, in Spanish and in English

> This page was written by the project's assistant, a Claude Code session, in audit entry §920
> (see [`GENAI.md`](../GENAI.md)): a generated artefact, not text written by the author. Each term
> points to where the project defines it; that page prevails.

The repository is written mostly in Spanish, and some of its words are the project's own. The
English twins ([`TRANSLATIONS.md`](../TRANSLATIONS.md)) keep a few of them in Spanish, with a gloss,
because they are search keys.

## The method and the record

| Spanish | English | what it is | defined in |
|---|---|---|---|
| asiento | entry | one numbered section of `AUDITORIA.md`, written for one verified change; cited by its number after the section sign; never rewritten, corrected with a ⚠️ mark | [`doc/METODO_EN.md`](METODO_EN.md), 2 |
| sello | seal | a commit whose entry records a full canon run in VERDE on its exact tree | [`doc/METODO_EN.md`](METODO_EN.md), 2 |
| canon | canon | `tools/canon.sh`, the single gate: tests with pinned counts, tools, catalogues, reproducible kit | [`doc/METODO_EN.md`](METODO_EN.md), 6 |
| compuerta | gate | one check of the canon that must pass | [`doc/METODO_EN.md`](METODO_EN.md), 6 |
| banco | bench | a `tools/banco_*.sh` script that runs a scenario against a live node | [`doc/METODO_EN.md`](METODO_EN.md), 6 |
| VERDE, ROJO | GREEN, RED | the verdicts the canon and the kit print; kept in Spanish | [`doc/KIT_EN.md`](KIT_EN.md) |
| la 136 | entry 136 | an entry of `BACKLOG.md`, by its number | [`BACKLOG.md`](../BACKLOG.md) |
| integrar no es aceptar | integrating is not accepting | the mark of what an assistant drafted and got in with the author's acceptance pending | [`GENAI.md`](../GENAI.md), Scope |
| fuera del paso 4 | outside step 4 | the changes a Claude Code session applied and committed itself, since §628 | [`GENAI.md`](../GENAI.md), Outside step 4 |
| gemelo | twin | the generated English translation of a Spanish page | [`TRANSLATIONS.md`](../TRANSLATIONS.md) |

## The specification

| Spanish | English | what it is | defined in |
|---|---|---|---|
| RFC, D-A, E1 | RFC, decision, stage | an RFC records decisions, named by letters (D-A, D-AM), and builds them in stages (E1, E2); its status is proposed or accepted | [`doc/METODO_EN.md`](METODO_EN.md), 5; [`PROCESO`](../spec/rfc/en/PROCESO.md) |
| cable, `zkssl/0.N` | wire | the JSON-RPC protocol between client and node, versioned | [`spec/RPC_EN.md`](../spec/RPC_EN.md) |
| núcleo | core | the frozen encodings, domains and families every implementation must reproduce | [`spec/NUCLEO_EN.md`](../spec/NUCLEO_EN.md) |
| familia | family | one AIR, a kind of proof, with its row in the core's registry | [`spec/NUCLEO_EN.md`](../spec/NUCLEO_EN.md), 4 bis |
| paquete, expediente | evidence package | what a third party receives to check without the node | [`spec/PAQUETE_EN.md`](../spec/PAQUETE_EN.md) |
| sobre | envelope | one typed JSON document of the package, judged by the kit | [`spec/PAQUETE_EN.md`](../spec/PAQUETE_EN.md) |
| kit | verifier kit | the `arqueo-verify` tarball: the verifier, the vectors and the harness, offline | [`doc/KIT_EN.md`](KIT_EN.md) |
| vector | vector | a conformance file with the verdict it must give, fixed by its fingerprint | [`spec/README.md`](../spec/README.md) |

## The ledger

| Spanish | English | what it is | defined in |
|---|---|---|---|
| la capa | the layer | the ledger engine, crate `zk-ssl` | [`ARQUITECTURA_EN.md`](../ARQUITECTURA_EN.md) |
| envío, cobro | send, claim | the two transitions of a payment, each with its proof | [`README_EN.md`](../README_EN.md#how-it-works-in-one-paragraph) |
| pendiente | pending item | an amount in flight between a send and its claim | [RFC-0008](../spec/rfc/en/0008-pruebas-portables-del-pendiente.md) |
| prenda | pledge | the payer's portable proof of a pending item | [RFC-0008](../spec/rfc/en/0008-pruebas-portables-del-pendiente.md) |
| rechazo con prueba | rejection with proof | a refusal that carries the rule that produced it | [RFC-0007](../spec/rfc/en/0007-pruebas-sobre-el-estado-comprometido.md) |
| acuse, recibo | acknowledgement, receipt | what the node signs for every operation it receives | [RFC-0010](../spec/rfc/en/0010-el-recibo-de-recepcion.md) |
| completitud | completeness | every acknowledged operation ends applied, rejected with proof, or with the operator named | [RFC-0010](../spec/rfc/en/0010-el-recibo-de-recepcion.md) |
| cabeza (de época) | (epoch) head | what the node signs every epoch: state, transition log, published consumptions | [`spec/PAQUETE_EN.md`](../spec/PAQUETE_EN.md) |
| testigo | witness | two meanings: an independent party that co-signs heads; or the private input of a proof | [`README_EN.md`](../README_EN.md#how-it-works-in-one-paragraph) |
| cofirma | co-signature | a witness's signature on a head | [RFC-0013](../spec/rfc/en/0013-el-medio-del-ancla.md) |
| ancla | anchor | the head published where the operator cannot erase it | [RFC-0012](../spec/rfc/en/0012-el-ancla-de-cabezas.md) |
| medio | medium | the public log that carries the anchors, as checkpoints co-signed by witnesses | [RFC-0013](../spec/rfc/en/0013-el-medio-del-ancla.md) |
| latido | heartbeat | the node's signed cadence, the clock of expiry | [RFC-0023](../spec/rfc/en/0023-el-reloj-de-la-caducidad.md) |
| custodio | custodian | a key holder; issuance needs two distinct ones | [`SECURITY_EN.md`](../SECURITY_EN.md) |
| guardián | guardian | the guard of the signer's index, crate `zk-ssl-guardian` | [`SECURITY_EN.md`](../SECURITY_EN.md) |

## Hiding

| Spanish | English | what it is | defined in |
|---|---|---|---|
| ocultación | hiding | what a proof reveals of its witness | [RFC-0009](../spec/rfc/en/0009-lo-que-revela-una-prueba.md) |
| censo | census | the measurement, on every canon run, that no literal of the witness appears in a proof | [RFC-0009](../spec/rfc/en/0009-lo-que-revela-una-prueba.md) |
| HVZK | HVZK | honest-verifier zero knowledge: the verifier's view is simulated without the witness | [`doc/ocultacion-hvzk_EN.md`](ocultacion-hvzk_EN.md) |
| H1 … H14 | hypotheses | the hypotheses of the hiding argument, each with its class: measured, proved or assumed | [`doc/ocultacion-hvzk_EN.md`](ocultacion-hvzk_EN.md), 4 |
| L0 … L9 | lemmas | the lemmas of that argument | [`doc/ocultacion-hvzk_EN.md`](ocultacion-hvzk_EN.md), 3 and 15 to 18 |
| AIR | AIR | algebraic intermediate representation: the constraints a STARK proves | [`spec/NUCLEO_EN.md`](../spec/NUCLEO_EN.md) |
