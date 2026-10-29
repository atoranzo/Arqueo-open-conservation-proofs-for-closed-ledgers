# The frozen core — what a verifier recomposes and does not change

> **Español:** [`spec/NUCLEO.md`](./NUCLEO.md) is the original of this page, in Spanish; it prevails
> wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and was
> produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`): it
> is a generated artefact, not text written by the author. Line citations of the form `file:N` refer
> to the Spanish files.

- **Status:** normative text in force since §407 (RFC-0005, stage E1)
- **Protocol version:** `zkssl/0.5` (§914; ⚠️ §914: it said `zkssl/0.4`) — this document does not
  move it: the core does not cross the wire
- **Origin:** `spec/rfc/0005-nucleo-congelado.md` (D-A, D-B, D-C); entry that seals it: §407
- **Reference implementation:** the crates `zk-ssl-verify` and `zk-ssl-hash`; the binary
  `zk-ssl-verify`
- **Tie:** `tools/check_nucleo.py`, in the canon: the table of this document and the `pub`s of the
  tree say the same thing in both directions

This document is the **sole normative producer** of WHAT does not change in the protocol and WHY.
It enumerates —by derivation, not from memory— every public element a third party reaches when
verifying without the node, without the layer and without the prover (§243), and gives it one of
four classes. The header of `zk-ssl-verify` says that «the families are named; the elements are
NOT enumerated; the truth is measured in the `pub`s»: here they are enumerated **because they are
measured** — `tools/check_nucleo.py` derives the census from the source in every canon and fails
if the table and the tree stop agreeing. A list that a gate does not re-derive goes stale at the
first extension, and that header's already went stale once (§247).

**Cite by name and file, never by line.** Lines go stale with every seal; names, only by RFC.

## 1. What the core is

**What is frozen is the verifier's side** (RFC-0005, D-A): the evidence package
(`spec/PAQUETE.md`) and everything its verification **recomposes or checks under a signature**. A
second implementation, written from `spec/` without reading the reference code, has to produce
**the same bytes** in each of these pieces, or it does not verify the same thing.

The JSON-RPC wire (`spec/RPC.md`) stays **out**: it is the interoperability contract, governed by
the same RFC process, and it needs the prover to be reproduced. The ledger (the trees of accounts,
pending items and frozen accounts) stays out: the package treats them as opaque roots under the
signed head. Since §458 there is one exception, and it is because of a proof: of the frozen tree,
its depth and the rule of its leaf come in, because the `AccountFrozen` rejection is proven with a
path under `frozenRoot` (RFC-0007, E3b; family CONGELADOS).

## 2. The four classes

- **NÚCLEO** [core] — it is signed, composed or checked. It does not change except by a new
  version of the preamble (section 3). A second implementation reproduces it byte for byte.
- **REFERENCIA** [reference] — of this implementation, not of the format: error types, the OID
  workaround of `xmss 0.1.0-pre.0` (§240), the reading of the key. A second implementation with a
  correct library **must not** replicate them.
- **LIBRO** [ledger] — compositions of the state that the package does not recompose (it treats
  them as opaque roots); the layer and its own RFC govern them.
- **REGISTRO** [log] — the re-verification family of the transition log and the operation seals:
  the node and the client compose them over the wire (`spec/RPC.md`), and that is why they do not
  belong to the envelope.

## 3. The extension rule, exercised

**First half — what is signed grows only by version.** The set of head versions the core accepts
has **a single producer**: `VersionCabeza` in `zk-ssl-verify` (§406), an exhaustive `enum` whose
text («v2, v3, v4, v5 o v6» since §558; «v2, v3, v4 o v5» since §451) is derived from its variants
and which the command and the witness
**consume**, without repeating it. A new composition is a new variant: the compiler flags every
`match` that forgets it, and `VERSION_FORMATO` has to be a member (tied in the crate's tests). A
`formatVersion` outside the set is rejected with text and **without truncating**: `0x103` is not a
3. ⚠️ §862: the set of schemes that are accepted follows the same mould —`Esquema::ACEPTADOS`, with
`Esquema::EMITIDOS` tied to it as `VERSION_FORMATO` is to the version set—, and a scheme outside the
registry or the set is rejected with its number, without truncating (RFC-0020, E3).

**Second half — what is not signed does not exist for the core.** A key of the envelope that no
composition and no preamble reads cannot alter what is checked; the reference verifier ignores
them (zero `serde` attributes, fourteen keys read by `.get()`). That the reference's wire consumer
also rejects unknown keys is a declared choice of that implementation, not a rule of the format.

**New families** enter with their `tipo` or their domain, their vectors and their RFC. If they need
to enter under the head's signature, they enter through the first half, as a new version of the
preamble. ⚠️ §914: and the proofs, through their v2 mark: the family and the version of its AIR are
in the registry of §4 bis (RFC-0018 D-G), and a proof of another wire version or of a family that is
not the one asked for is rejected by name before it is read. What of this is promised —that the set
only grows, and that a new version the node emits breaks the wire and raises `zkssl/0.N`— is said by
`RPC.md` §«Versionado y compatibilidad» [versioning and compatibility] (§801), and
`tools/check_superficie.py` compares `VersionCabeza::TODAS` and `VERSION_FORMATO` with its list in
every canon.

## 4. The census

**Derived census:** 120 reachable elements in `zk-ssl-verify` and 64 `pub`s in `zk-ssl-hash`
(LIBRO 5, NÚCLEO 146, REFERENCIA 14, REGISTRO 18). Reachable in `zk-ssl-verify` is what
`lib.rs` exports: its own `pub`s, everything `pub` in the `pub mod` modules (`acuses`, `mmr`,
`consumos`, `congelados`,
`cuentas`, `recibos`, `actas`) and the
names its `pub use` brings out of the private modules (`inclusion`, `reverificacion`). The
re-exports of `zk-ssl-hash` are not counted twice: one element, one row. In `zk-ssl-hash`, every
`pub` of `lib.rs` outside the test zones. The test zones are cut by the real nesting of their
braces, not by the first marker.

| element | file | class | family | what it is |
|---|---|---|---|---|
| `Digest` | `hash/lib.rs` | NÚCLEO | HASH | `type` |
| `FormatoError` | `hash/lib.rs` | REFERENCIA | HASH | `enum` |
| `HexError` | `hash/lib.rs` | REFERENCIA | HASH | `enum` |
| `ACCOUNTS_DEPTH` | `hash/lib.rs` | NÚCLEO | HASH | `const` |
| `CONS_DEPTH` | `hash/lib.rs` | NÚCLEO | HASH | `const` |
| `FROZEN_DEPTH` | `hash/lib.rs` | NÚCLEO | HASH | `const` |
| `STATE_WIDTH` | `hash/lib.rs` | NÚCLEO | HASH | `const` |
| `as_digest` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `digest_from_bytes` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `digest_to_bytes` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `element_from_bytes` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `element_to_bytes` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `bytes_de_hex` | `hash/lib.rs` | REFERENCIA | HASH | `fn` |
| `hex_canonico` | `hash/lib.rs` | REFERENCIA | HASH | `fn` |
| `cantidad_canonica` | `hash/lib.rs` | REFERENCIA | HASH | `fn` |
| `embeber` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `native_merge` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `path_root` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `posicion_de_consumo` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `MODULO` | `hash/lib.rs` | NÚCLEO | HASH | `const` |
| `u64_canonico` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `epoch_digest` | `hash/lib.rs` | NÚCLEO | CABEZA | `fn` |
| `epoch_digest_v2` | `hash/lib.rs` | NÚCLEO | CABEZA | `fn` |
| `epoch_digest_v3` | `hash/lib.rs` | NÚCLEO | CABEZA | `fn` |
| `epoch_digest_v4` | `hash/lib.rs` | NÚCLEO | CABEZA | `fn` |
| `epoch_digest_v5` | `hash/lib.rs` | NÚCLEO | CABEZA | `fn` |
| `epoch_digest_v6` | `hash/lib.rs` | NÚCLEO | CABEZA | `fn` |
| `params_digest` | `hash/lib.rs` | NÚCLEO | CABEZA | `fn` |
| `DOMINIO_PARAMS` | `hash/lib.rs` | NÚCLEO | CABEZA | `const` |
| `ACEPTADOS` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `ANCHO_INDICE` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `COFIRMA_VERSION` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `COFIRMA_V_MAX` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `CabezaFirmada` | `verify/lib.rs` | NÚCLEO | FIRMA | `struct` |
| `Conjunto` | `verify/lib.rs` | NÚCLEO | FIRMA | `type` |
| `DOMINIO` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `DOMINIO_COFIRMA` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `EMITIDOS` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `EntradaEsquema` | `verify/lib.rs` | NÚCLEO | FIRMA | `struct` |
| `Esquema` | `verify/lib.rs` | NÚCLEO | FIRMA | `enum` |
| `EsquemaNoRegistrado` | `verify/lib.rs` | REFERENCIA | FIRMA | `struct` |
| `FIRMA_RFC_BYTES` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `IMPLICITO` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `OFFSET_MT_UPSTREAM` | `verify/lib.rs` | REFERENCIA | FIRMA | `const` |
| `TODAS` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `TODOS` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `VERSION_FORMATO` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `VerificaError` | `verify/lib.rs` | REFERENCIA | FIRMA | `enum` |
| `VersionCabeza` | `verify/lib.rs` | NÚCLEO | FIRMA | `enum` |
| `VersionCabezaDesconocida` | `verify/lib.rs` | REFERENCIA | FIRMA | `struct` |
| `aplicar_apano_del_oid` | `verify/lib.rs` | REFERENCIA | FIRMA | `fn` |
| `as_u64` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `as_u8` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `clave_desde_bytes` | `verify/lib.rs` | REFERENCIA | FIRMA | `fn` |
| `entrada` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `indice_de_firma` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `lleva_mmr` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `lleva_consumos` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `lleva_recepcion` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `lleva_parametros` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `preambulo` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `preambulo_cofirma` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `preambulo_cofirma_v7` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `preambulo_v7` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `texto` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `texto_con_mmr` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `texto_con_consumos` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `texto_con_recepcion` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `texto_con_parametros` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `verificar_cabeza` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `verificar_cofirma` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `DOMINIO_ACUSE` | `hash/lib.rs` | NÚCLEO | ACUSES | `const` |
| `acuse_digest` | `hash/lib.rs` | NÚCLEO | ACUSES | `fn` |
| `DOMINIO_RECEP` | `hash/lib.rs` | NÚCLEO | ACUSES | `const` |
| `recibo_digest` | `hash/lib.rs` | NÚCLEO | ACUSES | `fn` |
| `DOMINIO_RECEP_V2` | `hash/lib.rs` | NÚCLEO | ACUSES | `const` |
| `recibo_digest_v2` | `hash/lib.rs` | NÚCLEO | ACUSES | `fn` |
| `epoca_de_acuse` | `verify/acuses.rs` | NÚCLEO | ACUSES | `fn` |
| `hoja_de_acuse` | `verify/acuses.rs` | NÚCLEO | ACUSES | `fn` |
| `indice_de_hoja` | `verify/acuses.rs` | NÚCLEO | ACUSES | `fn` |
| `pertenece` | `verify/acuses.rs` | NÚCLEO | ACUSES | `fn` |
| `ReciboAcuse` | `verify/inclusion.rs` | NÚCLEO | ACUSES | `struct` |
| `verificar_acuse` | `verify/inclusion.rs` | NÚCLEO | ACUSES | `fn` |
| `verificar_acuse_v3` | `verify/inclusion.rs` | NÚCLEO | ACUSES | `fn` |
| `verificar_acuse_v4` | `verify/inclusion.rs` | NÚCLEO | ACUSES | `fn` |
| `verificar_acuse_v5` | `verify/inclusion.rs` | NÚCLEO | ACUSES | `fn` |
| `verificar_acuse_v6` | `verify/inclusion.rs` | NÚCLEO | ACUSES | `fn` |
| `DOMINIO_MMR_HOJA` | `hash/lib.rs` | NÚCLEO | MMR | `const` |
| `DOMINIO_MMR_NODO` | `hash/lib.rs` | NÚCLEO | MMR | `const` |
| `mmr_hoja` | `hash/lib.rs` | NÚCLEO | MMR | `fn` |
| `mmr_nodo` | `hash/lib.rs` | NÚCLEO | MMR | `fn` |
| `cima` | `verify/mmr.rs` | NÚCLEO | MMR | `fn` |
| `hoja_desde_bytes` | `verify/mmr.rs` | NÚCLEO | MMR | `fn` |
| `prueba_de_consistencia` | `verify/mmr.rs` | NÚCLEO | MMR | `fn` |
| `prueba_de_inclusion` | `verify/mmr.rs` | NÚCLEO | MMR | `fn` |
| `verificar_consistencia` | `verify/mmr.rs` | NÚCLEO | MMR | `fn` |
| `verificar_inclusion` | `verify/mmr.rs` | NÚCLEO | MMR | `fn` |
| `cruza_posicion` | `verify/consumos.rs` | NÚCLEO | CONSUMO | `fn` |
| `hoja_vacia` | `verify/consumos.rs` | NÚCLEO | CONSUMO | `fn` |
| `is_right_de_posicion` | `verify/consumos.rs` | NÚCLEO | CONSUMO | `fn` |
| `raiz_de_ausencia` | `verify/consumos.rs` | NÚCLEO | CONSUMO | `fn` |
| `raiz_de_presencia` | `verify/consumos.rs` | NÚCLEO | CONSUMO | `fn` |
| `cruza_indice` | `verify/congelados.rs` | NÚCLEO | CONGELADOS | `fn` |
| `esta_congelada` | `verify/congelados.rs` | NÚCLEO | CONGELADOS | `fn` |
| `is_right_de_indice` | `verify/congelados.rs` | NÚCLEO | CONGELADOS | `fn` |
| `raiz_de_hoja` | `verify/congelados.rs` | NÚCLEO | CONGELADOS | `fn` |
| `cruza_indice` | `verify/cuentas.rs` | NÚCLEO | CUENTAS | `fn` |
| `is_right_de_indice` | `verify/cuentas.rs` | NÚCLEO | CUENTAS | `fn` |
| `no_existe` | `verify/cuentas.rs` | NÚCLEO | CUENTAS | `fn` |
| `raiz_de_hoja` | `verify/cuentas.rs` | NÚCLEO | CUENTAS | `fn` |
| `RECEP_DEPTH` | `verify/recibos.rs` | NÚCLEO | RECIBOS | `const` |
| `dentro_de_ventana` | `verify/recibos.rs` | NÚCLEO | RECIBOS | `fn` |
| `era_de_recibo` | `verify/recibos.rs` | NÚCLEO | RECIBOS | `fn` |
| `hoja_de_recibo` | `verify/recibos.rs` | NÚCLEO | RECIBOS | `fn` |
| `indice_de_recibo` | `verify/recibos.rs` | NÚCLEO | RECIBOS | `fn` |
| `pertenece_a_era` | `verify/recibos.rs` | NÚCLEO | RECIBOS | `fn` |
| `raiz_de_camino_de_recibo` | `verify/recibos.rs` | NÚCLEO | RECIBOS | `fn` |
| `hash_del_lote` | `hash/lib.rs` | NÚCLEO | RECIBOS | `fn` |
| `digest_pi` | `hash/lib.rs` | NÚCLEO | RECIBOS | `fn` |
| `DOMINIO_ANCLA` | `hash/lib.rs` | NÚCLEO | ANCLA | `const` |
| `ancla_digest` | `hash/lib.rs` | NÚCLEO | ANCLA | `fn` |
| `huella_de_clave` | `hash/lib.rs` | NÚCLEO | ANCLA | `fn` |
| `DOMINIO_ACTA` | `hash/lib.rs` | NÚCLEO | ACTA | `const` |
| `acta_digest` | `hash/lib.rs` | NÚCLEO | ACTA | `fn` |
| `ACTA_VERSION` | `verify/actas.rs` | NÚCLEO | ACTA | `const` |
| `DOMINIO_ACTA_FIRMA` | `verify/actas.rs` | NÚCLEO | ACTA | `const` |
| `ESQUEMA_XMSSMT_SHA2_40_8_256` | `verify/actas.rs` | NÚCLEO | ACTA | `const` |
| `preambulo_acta` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `Procedencia` | `verify/actas.rs` | NÚCLEO | ACTA | `struct` |
| `Acta` | `verify/actas.rs` | NÚCLEO | ACTA | `struct` |
| `digest` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `ActaFirmada` | `verify/actas.rs` | NÚCLEO | ACTA | `struct` |
| `ActaError` | `verify/actas.rs` | REFERENCIA | ACTA | `enum` |
| `verificar_acta` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `acta_a_json` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `acta_de_json` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `verificar_cadena` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `Rotacion` | `verify/actas.rs` | NÚCLEO | ACTA | `struct` |
| `en_su_tramo` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `RotacionError` | `verify/actas.rs` | REFERENCIA | ACTA | `enum` |
| `juzgar_rotacion` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `juzgar_continuidad` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `Tramo` | `verify/actas.rs` | NÚCLEO | ACTA | `struct` |
| `contiene` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `tramos_de` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `relevo_del_acta` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `CabezaDeLaCadena` | `verify/actas.rs` | NÚCLEO | ACTA | `struct` |
| `juzgar_historia` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `Relevo` | `verify/actas.rs` | NÚCLEO | ACTA | `struct` |
| `relevo_del_tramo` | `verify/actas.rs` | NÚCLEO | ACTA | `fn` |
| `native_leaf` | `hash/lib.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `native_leaf_salted` | `hash/lib.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `InclusionError` | `verify/inclusion.rs` | REFERENCIA | INCLUSIÓN | `enum` |
| `ReciboInclusion` | `verify/inclusion.rs` | NÚCLEO | INCLUSIÓN | `struct` |
| `verificar_inclusion` | `verify/inclusion.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `verificar_inclusion_v2` | `verify/inclusion.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `verificar_inclusion_v3` | `verify/inclusion.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `verificar_inclusion_v4` | `verify/inclusion.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `verificar_inclusion_v5` | `verify/inclusion.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `verificar_inclusion_v6` | `verify/inclusion.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `DOMINIO_META_PENDIENTE` | `hash/lib.rs` | LIBRO | LIBRO | `const` |
| `meta_pendiente_hoja` | `hash/lib.rs` | LIBRO | LIBRO | `fn` |
| `DOMINIO_PRENDA` | `hash/lib.rs` | LIBRO | LIBRO | `const` |
| `marca_prenda` | `hash/lib.rs` | LIBRO | LIBRO | `fn` |
| `SPEND_KEY_DOMAIN` | `hash/lib.rs` | LIBRO | LIBRO | `const` |
| `COMPROMISO_AUSENTE` | `hash/lib.rs` | REGISTRO | REGISTRO | `const` |
| `OP_FREEZE` | `hash/lib.rs` | REGISTRO | REGISTRO | `const` |
| `OP_GOVERNANCE` | `hash/lib.rs` | REGISTRO | REGISTRO | `const` |
| `OP_MINT` | `hash/lib.rs` | REGISTRO | REGISTRO | `const` |
| `OP_MINT_PENDING` | `hash/lib.rs` | REGISTRO | REGISTRO | `const` |
| `OP_RECOVERY` | `hash/lib.rs` | REGISTRO | REGISTRO | `const` |
| `commit_operation` | `hash/lib.rs` | REGISTRO | REGISTRO | `fn` |
| `digest_of_proof` | `hash/lib.rs` | REGISTRO | REGISTRO | `fn` |
| `huella_de_contenido` | `hash/lib.rs` | REGISTRO | REGISTRO | `fn` |
| `sello_de_autorizacion` | `hash/lib.rs` | REGISTRO | REGISTRO | `fn` |
| `sello_sin_prueba` | `hash/lib.rs` | REGISTRO | REGISTRO | `fn` |
| `EmisionConContador` | `verify/reverificacion.rs` | REGISTRO | REGISTRO | `enum` |
| `EntradaLog` | `verify/reverificacion.rs` | REGISTRO | REGISTRO | `struct` |
| `ReverificacionError` | `verify/reverificacion.rs` | REGISTRO | REGISTRO | `enum` |
| `Veredicto` | `verify/reverificacion.rs` | REGISTRO | REGISTRO | `enum` |
| `censo` | `verify/reverificacion.rs` | REGISTRO | REGISTRO | `fn` |
| `reverificar` | `verify/reverificacion.rs` | REGISTRO | REGISTRO | `fn` |
| `reverificar_con` | `verify/reverificacion.rs` | REGISTRO | REGISTRO | `fn` |

### 4 bis. The registry of the families of the v2 mark (laboratory, §906; in force, §914)

⚠️ **§906:** the v2 mark of the 0.5 cut (RFC-0018 D-G) carries the proof's family in a `u16`, the
version of its AIR in another and the options profile in a byte, and since §899 the random columns
in another (RFC-0009 D-AL): `arqueo:oculta:2`, m, family, version, profile and columns, 25 fixed
bytes, all *little-endian*. This registry is the one of D-G, and it comes into force in the seal of
the cut: until then it lives in the tree in laboratory, with a single producer in the code,
`zk_ssl_air::familias` (`crates/zk-ssl-air/src/familias.rs`), and one KAT per entry in its tests
—the 25 bytes of each mark with m = 64, version 1, profile 1 and two columns—, which the cut will
take to `spec/vectors/nucleo/`. No production prover writes the v2 mark and no judge accepts it
(`Marca::leer` rejects it as unknown); it is written by the prover that asks for it
(`Ocultacion::marca_v2`) and read by `verify_v2`, of laboratory, with the causes of D-G in their
order. Zero is not assigned in any field; profile 255 is reserved for another engine.
⚠️ **§914: in force since the `zkssl/0.5` cut.** The 23 provers write the v2 mark
(`ocultacion_encendida(air)`, with the family of this registry), and it is read before anything else
by the single judge (`zk_ssl_air::juez`), the kit's five judges and `tools/segunda`, with the causes
of D-G and one more: the columns the mark declares are those of the construction, two (1h bis). The
version of each AIR is still 1 and the profile 1 (the production options). The KATs of the 23 marks
stay in the tests of `familias.rs`, and also go to `spec/vectors/nucleo/registro_de_familias.json`,
which the second implementation reproduces (`tools/segunda/juez_nucleo.py`).

| family | AIR |
|---|---|
| 1 | `SendAir` |
| 2 | `SendV2Air` |
| 3 | `ClaimAir` |
| 4 | `ClaimAirV2` |
| 5 | `PrendaAir` |
| 6 | `MintPendingClimbAir` |
| 7 | `NullifierThresholdAir` |
| 8 | `ThresholdAir` |
| 9 | `AuditAir` |
| 10 | `BurnAir` |
| 11 | `SolvencyAir` |
| 12 | `DoubleEntryAir` |
| 13 | `BandaAir` |
| 14 | `EdadAir` |
| 15 | `CobroPendienteAir` |
| 16 | `PagoEnCursoAir` |
| 17 | `MerkleAir` |
| 18 | `RefundAir` |
| 19 | `RefundAirV2` |
| 20 | `CreditClimbAir` |
| 21 | `MintClimbAir` |
| 22 | `FrozenClimbAir` |
| 23 | `RecoveryClimbAir` |

The initial version of every family is 1 and the production profile is 1 (a single `OptionSet` per
wire version). They are the 23 AIRs with a hidden prover, in the order of the table of D-B of
RFC-0009.

## 5. Why it cannot change, by family

- **HASH** — the permutation and the 2-to-1 merge, how a `u64` is embedded, how a path is climbed
  and how a digest is written in bytes: it is the boundary between the JSON and the bytes. Two
  implementations that differ here agree on nothing. Since §640 (RFC-0016) the family also fixes
  **what a valid encoding is**: the modulus `p` and the canonical reading, without which one same
  signature attested two integers and the two implementations of the core read the same byte
  differently.
- **CABEZA** [head] — the five compositions of the head digest (v1, v2, v3, v4, v5) and the digest
  of the parameters v5 signs. What is in custody does not expire (§290): an old composition must
  always be recomposable.
- **FIRMA** [signature] — the scheme (`XmssMtSha2_40_8_256`), the two domains, the version byte and
  its set, the two preambles and the index embedded in the signature. They are the exact bytes
  under the signature; changing one would make what is in custody collide or stop verifying. Since
  §800, also the scheme registry, with what each entry declares, and the two v7 preambles
  (RFC-0020, E2), which nobody signs yet: a scheme number that changed owner would make a
  signature in custody name another scheme. ⚠️ §862: and since E3, the two sets of the dispatch, the
  accepted one and the emitted one, and the implicit scheme of v1 to v6: a scheme that left the
  accepted set would leave what it signed unverified.
- **ACUSES** [acknowledgements] — the rules of the acknowledgement tree and its composition: the
  node builds them and the verifier checks them by calling the same ones (§274).
- **MMR** — the MMR of heads: the leaf, the node, the peak, inclusion and consistency (§291).
- **INCLUSIÓN** [inclusion] — the account leaf (both forms) and the inclusion receipts.
- **CONGELADOS** [frozen] — the depth of the frozen tree, the crossing of the path against the
  account's index and the rule of its leaf (frozen is non-empty): what sustains `AccountFrozen`
  without the layer (§458). The core fixes the depth because the merge does not tell leaf from node.
- **CUENTAS** [accounts] — the mirror of the previous one over the account tree: the depth the
  core fixes, the crossing of the path against the index and the rule of the leaf (not existing is
  an EMPTY leaf): what sustains `AccountNotFound` without the layer (§475). `ACCOUNTS_DEPTH` is
  worth the same as `FROZEN_DEPTH` today, and they are two distinct facts.
- **ANCLA** [anchor] — the fingerprint of the head anchor and the fingerprint of the operator's
  key (RFC-0012): what is published in a medium outside the operator's control and what a third
  party recomposes to compare it. A single producer: two implementations that differ here do not
  compare the same anchor.
- **ACTA** [act] — the act with which an operator key enters, and commits to its successor
  (RFC-0015, E2): its fingerprint, the preamble it signs, the scheme it presents and the rules
  with which a third party judges a rotation against the previous act. An implementation that
  composes another fingerprint does not recognise the committed successor, and a legitimate
  rotation looks like a theft to it.

## 6. The bytes: what a KAT fixes

This document **names**; the bytes are fixed by the vectors of `spec/vectors/nucleo/` (§411,
RFC-0005 E5): one file per NÚCLEO `fn`, `{fn, entradas, salida}` in hex `0x…`, emitted by the
reference and reproduced in every canon (`zk-ssl-cli`, `nucleo_kat`). ⚠️ The `u64`s of the inputs
go as a hex **number** (`seq`, `n`, `t`, the counters, `as_digest.x`), except `embeber.x` and the
`balance`/`nonce` of `native_leaf*`, which go as **eight little-endian bytes**: two encodings in
one catalogue, and a judge has to know it from outside (measured by the second implementation,
`tools/segunda/juez_nucleo.py`). A second implementation
that gives these bytes in each one gives the same bytes in everything that is signed. They are a
snapshot of the reference, and it is declared: they fix the property «two implementations give
these bytes». Since §800 there is one that is not of a `fn`: `registro_de_esquemas.json` fixes
the whole scheme registry (below), and its `salida` is the LIST of its entries, with the number as
QUANTITY, the key prefix in hex, the lengths as numbers and the two yes-or-no as JSON booleans.
Since §914 there is another: `registro_de_familias.json` fixes the whole registry of the families of
the v2 mark of section 4 bis (RFC-0018 D-G), and its `salida` is the LIST of its 23 entries, each
with the family, the version of its AIR and the profile as QUANTITY, the name of the AIR and the 25
bytes of its mark in hex, with the house's m and columns as `entradas`; `tools/check_nucleo.py` (R5)
ties the table to its producer, `zk_ssl_air::familias`.

- **The permutation**: Rescue-Prime `Rp64_256`, as `winter-crypto =0.13.1` implements it, over
  the Goldilocks field (`winter-math =0.13.1`, `f64::BaseElement`). A `Digest` is four elements.
  `native_merge(l, r)`: a state of twelve elements at zero, `l` in `[4..8]`, `r` in `[8..12]`, one
  permutation, output `[4..8]`; the capacity `[0..4]` stays at zero. `embeber(x) =
  [x, 0, 0, 0]`; `as_digest(u)` embeds the `u64`.
- **Serialisation**: each element in 8 *little-endian* bytes (`as_int`), the four in order;
  `digest_from_bytes` requires 32 bytes. On the wire they go as `DATA`/`Digest` (`RPC.md`).
- **Canonicity** (RFC-0016, §640). The field is `F_p` with `p = MODULO = 2^64 - 2^32 + 1`. A `u64`
  **encodes** an element only if it is less than `p`; the `2^32 - 1` values of `[p, 2^64)` encode
  none. Two rules, with a single producer, `u64_canonico`:

  1. **An element is read canonical.** `element_from_bytes`, and with it `digest_from_bytes`,
     rejects the eight bytes of a value `>= p` with
     `0x… no es canonico: no es menor que p = 2^64 - 2^32 + 1`,
     instead of reducing it. So writing and reading are exact inverses: `element_to_bytes` is a
     bijection from `F_p` onto `[0, p)`, and a digest has **one** 32-byte encoding.
  2. **A `u64` that enters a composition is read canonical.** `as_digest(x) = [x mod p, 0, 0, 0]`
     is injective on `[0, p)` and nowhere else: `as_digest(x + p) = as_digest(x)` for
     `x < 2^32 - 1`. The function does not change and its KATs do not move; whoever reads from
     outside a `u64` that is going to be composed (`seq`, `n`, `t`, the counters, the marks, the
     index) passes it through `u64_canonico` first, and rejects it with the same text if it does
     not fit.
  3. **A producer reduces.** What comes out of Blake3 —`digest_of_proof`, `huella_de_clave`,
     `hash_del_lote`— is 32 bytes that are read as four *little-endian* `u64`s and each one is
     REDUCED modulo `p` (`BaseElement::new`): the output is a canonical digest and a function of
     the bytes, even if a limb does not fit (`2^-32` per limb). Reducing is for whoever produces;
     rejecting, for whoever reads an encoding that arrives from outside. The second implementation
     returned the Blake3 bytes as they were and rejected the limb that did not fit: it diverged
     here too, and since §640 it reduces.

  The consequence, which is what the core promises: **over the canonical, every composition of
  this section that produces a digest is injective in its inputs barring a hash collision**, with
  one exception that is not of the hash but of the shape: in `path_root`, a level whose sibling
  equals the node climbing up gives the same with both orientations (`merge(x, x)`). That is why
  the position is never read from the root: where the position asserts something, the verifier
  crosses the path against the index (CONSUMO, CONGELADOS, CUENTAS, RECIBOS). Outside the canonical
  there was no such function: a head signed with `n` verified just the same with `n + p`, and
  whoever reasoned about the integer -the window of the promise, the size of the MMR- reasoned
  about one the signature does not fix (measured in §640 with real vectors, which are now
  negatives with a name).
- **The domains**: `u64`s read *big-endian* from eight ASCII bytes (`ACUSE_V1`, `MMRHOJA1`,
  `MMRNODO1`, `PARAM_V1`), embedded with `as_digest` and merged in front. Those of the signature
  are byte strings: `b"ZK-SSL-epoch-head"` (17), `b"ZK-SSL-witness-cosign"` (21) and, since
  §643, `b"ZK-SSL-key-act"` (14), the act's.
- **The preambles** (moved here from `zk-ssl-verify/src/lib.rs`, which refers to this section):

  ```text
  b"ZK-SSL-epoch-head" ‖ version ‖ epoch_digest                            (17 + 1 + 32 = 50)
  b"ZK-SSL-witness-cosign" ‖ version ‖ epoch_digest ‖ len(u16 BE) ‖ clave_op     (21 + 1 + 32 + 2 + N)
  b"ZK-SSL-key-act" ‖ version ‖ acta_digest                                (14 + 1 + 32 = 47)
  b"ZK-SSL-epoch-head" ‖ 7 ‖ esquema(u64 BE) ‖ epoch_digest                (17 + 1 + 8 + 32 = 58)
  b"ZK-SSL-witness-cosign" ‖ 7 ‖ esquema_testigo(u64 BE) ‖ esquema_operador(u64 BE)
      ‖ epoch_digest ‖ len(u16 BE) ‖ clave_op                              (21 + 1 + 8 + 8 + 32 + 2 + N)
  ```

  The last two, since §800 (RFC-0020, E2), are **the v7 preambles** of the head and of the
  co-signature: the version byte, 7, says the preamble carries the signature's scheme —a number
  from the registry below, in eight big-endian bytes—, and in the co-signature the witness's goes
  first and the operator's after, with the operator's key last, as up to v6. The domain does not
  change, and neither does `epoch_digest`: the scheme is the signature's, not the ledger's. They
  are **switched off**: nobody signs or accepts them until the `zkssl/0.5` cut (E4 of RFC-0020),
  and no verifier accepts a signature over the v7 today. Their KATs are `preambulo_v7.json` and
  `preambulo_cofirma_v7.json`; the co-signature's carries two distinct schemes, to fix their
  order, and the operator's is not in the registry: the preamble composes bytes, and whether a
  number is in the registry is checked by whoever verifies (E3). ⚠️ §862: since E3 it checks it,
  before reading the key; the two v7 preambles are still switched off, and a signature over them is
  still not accepted.

- **The scheme registry** (RFC-0020, D-A; §800). The number with which a signature's scheme is
  named is the `esquema` of the act, a `u64` with the family in the high 32 bits and the family's
  parameter in the low ones; the act composes it with `as_digest`, and the v7 preambles write it
  in eight big-endian bytes: one encoding and not two. It has a single producer, `Esquema` in
  `zk-ssl-verify` —the mould of `VersionCabeza`—, from which `ESQUEMA_XMSSMT_SHA2_40_8_256` comes,
  and its KAT, `registro_de_esquemas.json`, which fixes the whole list with what each entry
  declares: the encoding of its public key, that of its signature, whether it has state and the
  text with which it is named in rejections. It is born with one entry:

  | scheme | family and parameter | name | public key | signature | state |
  |---|---|---|---|---|---|
  | `0x1_0000_0005` | family 1, the **XMSS^MT** registry of RFC 8391 —not the XMSS one, where 5 is
  XMSS-SHA2_16_512 (§240)—, and its OID, 5 | `XMSSMT-SHA2_40/8_256` | 68 B:
  `OID (u32 BE) ‖ root (32) ‖ SEED (32)`, which opens with `00 00 00 05` | 18.469 B of RFC 8391
  —`idx_sig` (5 B) ‖ `r` (32 B) ‖ eight layers of a WOTS+ signature (67 · 32 B) and its path (5 · 32
  B)—, with the preamble attached behind | with state: the index goes in the signature, and a
  guardian keeps it |

  Its rules: **zero is not assigned**, as in the v2 mark of D-G of RFC-0018; **two schemes of the
  registry share no key encoding** —the same length, with a prefix that is a prefix of the
  other— (decision 4 of RFC-0020, branch (a)): the `siguiente` of an act is the fingerprint of
  the bytes of a key, and it only says which scheme the successor is of if no key byte is valid
  for two; and every number is **less than `p`**, because the act composes it with `as_digest`
  (DERIVED from rule 2 of canonicity). No other entry is assigned here: which scheme would be B is
  measured by entry 108 of `BACKLOG.md`, and the registry gives its number when it enters. It is
  **switched off**: nobody reads it yet —the verifier fixes `Conjunto` in its code and the act
  compares against its constant—; choosing the verifier by the scheme is E3 of RFC-0020. ⚠️ §862:
  that held until E3, which reads it: the verifier chooses by the scheme, with an accepted set and
  an emitted one (`Esquema::ACEPTADOS` and `Esquema::EMITIDOS`); that of a head or a co-signature v1
  to v6 is the implicit one (`Esquema::IMPLICITO`); and the act looks at the accepted set and
  verifies the signature of the key that leaves with the scheme of the previous act. Nobody signs
  with the scheme inside yet: that is E4.

- **The compositions**, in the exact order of the merges: `native_leaf = merge(merge(pk,
  embeber(saldo)), embeber(nonce))` and the salted one `merge(hoja, salt)`; `path_root` climbs from
  the leaf, with the sibling left if `is_right`; `epoch_digest = merge(merge(merge(as_digest(seq),
  accounts), merge(pending, frozen)), chain)`; `v2 = merge(v1, merge(acuses_root, as_digest(n)))`;
  `v3 = merge(v2, merge(cima, as_digest(t)))`, genesis `as_digest(0)` and `t = 0`;
  `v4 = merge(v3, merge(cons_root, as_digest(cons_count)))` (RFC-0006 E2a, §414), genesis the root
  of the empty consumption tree and `cons_count = 0`;
  `v5 = merge(v4, merge(merge(params_digest, pmeta_root), merge(as_digest(next_pending),
  merge(as_digest(next_index), as_digest(total_supply)))))` (RFC-0007 E1a, §451), genesis the
  opening parameters, the root of the empty meta tree, `next_pending = 0` and the accounts and the
  supply of that head;
  `v6 = merge(v5, merge(recep_root, as_digest(recep_count)))` (RFC-0010 E2, §557), genesis the root
  of the empty receipt tree and `recep_count = 0`;
  `recibo_digest = merge(as_digest(RECEP_V1), merge(hash_prueba, merge(as_digest(era),
  as_digest(n))))`, the mould of `acuse_digest` with the seventh domain;
  `recibo_digest_v2 = merge(as_digest(RECEP_V2), merge(hash_prueba, merge(digest_pi,
  merge(as_digest(era), as_digest(n)))))` (RFC-0019 E3, §773), the v1 leaf with `digest_pi` behind
  `hash_prueba` and the tenth domain, `RECEP_V2`; nothing consumes it until the `zkssl/0.5` cut;
  `ancla_digest = merge(as_digest(ANCLA_V1), merge(huella_clave, merge(as_digest(indice),
  merge(epoch_digest, merge(mmr_root, as_digest(mmr_size))))))` (RFC-0012, §591), with the eighth
  domain and the index EMBEDDED in the signature;
  `huella_de_clave = Blake3(b"ZK-SSL-anchor-key-v1" ‖ len(u64 LE) ‖ clave)`, the mould of
  `digest_of_proof` (§116) with its own byte domain;
  `acta_digest = merge(as_digest(ACTAS_V1), merge(as_digest(etiqueta), resto))` (RFC-0015,
  §643), with `cuerpo = merge(huella_clave, merge(as_digest(esquema), merge(as_digest(desde),
  siguiente)))`; in the genesis act `etiqueta = 0` and `resto = cuerpo`; in a rotation
  `etiqueta = 1` and `resto = merge(anterior, merge(merge(epoch_digest, merge(mmr_root,
  as_digest(mmr_size))), cuerpo))`, with `anterior` and `siguiente` key fingerprints, `desde` the
  EMBEDDED index of the first signature of the key that enters, and `esquema` that of today's key,
  `ESQUEMA_XMSSMT_SHA2_40_8_256 = 0x1_0000_0005` (the family of RFC 8391 in the high 32 bits
  and its OID in the low ones);
  `hash_del_lote = Blake3(b"ZK-SSL-batch-v1" ‖ len(u64 LE) ‖ (hash_prueba_i ‖ cuenta_i ‖
  posicion_i)*)` (RFC-0014, §610), the same mould, with 48 bytes per operation in the order of the
  batch -the digest of its proof, and its account and its position as `u64 LE`-; `len` is the
  **bytes** of the composition, `48·k`, as in the mould of §116, and `k` is left implicit in it; it
  is what goes as `hash_prueba` in the `recibo_digest` of a batch. ⚠️ **Corrected by the second
  implementation** (`tools/segunda/`, entry 85): until then it said here «`k` in the length», and
  with `len = k` the KAT does not reproduce (measured); the error is recorded instead of erased;
  `digest_pi = Blake3(b"ZK-SSL-public-inputs-v1" ‖ len(u64 LE) ‖ familia(u16 LE) ‖
  version_air(u16 LE) ‖ e_0 ‖ … ‖ e_{k-1})` (RFC-0019 E3, §773), the same mould, with the family
  and the version of the AIR of the received proof in the bytes the v2 mark of RFC-0018 gives them,
  and the elements of its public inputs in the order of `to_elements()`, eight LE bytes each; `len`
  is the bytes, `4 + 8·k`, and `k` is left implicit in it; it ties the family and the version, not
  the profile;
  `params_digest = merge(as_digest(PARAM_V1),
  merge(merge(as_digest(regulatory_limit), as_digest(max_supply)),
  merge(merge(as_digest(max_accounts), custodian_set_root), merge(governance_set_root,
  merge(as_digest(refund_ttl), as_digest(max_custodian_uses))))))`, the seven in that order;
  `acuse_digest = merge(as_digest(ACUSE_V1), merge(hash_prueba, merge(as_digest(epoca),
  as_digest(n))))`; `mmr_hoja = merge(as_digest(MMRHOJA1), cabeza)`; `mmr_nodo =
  merge(as_digest(MMRNODO1), merge(izq, der))`; the peak is the Merkle tree with the cut at the
  largest power of two less than `n`, **with every head passed through `mmr_hoja` inside the
  peak** (the reading of RFC 6962: `cima([c]) = mmr_hoja(c)`); ⚠️ made precise by the second
  implementation (`tools/segunda/`, entry 85): until then it was not said, and the KAT fixes it.
  **The acknowledgement leaf**, which `PAQUETE.md` 2.1 names `hoja_de_acuse(hashPrueba, seq, n)` and
  until here was not written down: it is `acuse_digest(hashPrueba, seq + 1, n)`, because its `seq`
  is the `logSeq` of the operation and the epoch of the acknowledgement is `logSeq + 1`, the first
  head that can contain it (`RPC.md`, «El acuse en la respuesta» [the acknowledgement in the
  response], §274); with `epoca = seq` the KAT does not reproduce (measured).
- **What a KAT cannot give**: the signature. `XmssMtSha2_40_8_256` is RFC 8391 and the published
  key carries its correct OID; the OID workaround is of the reading by `xmss 0.1.0-pre.0`
  (REFERENCIA) and a correct library does not need it.

## 7. What this document does NOT claim

- It does not freeze the wire or the ledger: `spec/RPC.md` and the layer have their own rules and
  their own RFCs.
- It does not claim that a second implementation exists (E5 of RFC-0005): it states what one would
  have to reproduce.
- That an independent verifier exists does not make the signatures opposable: the declared
  custody of the operator's key is still missing (`SECURITY.md`).
- Nothing in the core identifies anything outside the ledger (RFC-0005, D-C): not an invoice, not
  a label shared between operators. Uniqueness across ledgers is a new family with its own RFC.

## 8. History

- §914 — the registry of the families of the v2 mark (section 4 bis) **in force**, with the
  `zkssl/0.5` cut (RFC-0018 D-G and D-H): the 23 provers write the mark and the judges read it
  before anything else; its KAT, `registro_de_familias.json`, which the second implementation
  reproduces, and R5 of `tools/check_nucleo.py`, which ties the table to `zk_ssl_air::familias`. No
  new row in the census: the registry lives in `zk-ssl-air`, outside the two crates of the census.
  Not a byte of the core moves.
- §862 — dispatch by scheme in `zk-ssl-verify` (RFC-0020, E3): `ACEPTADOS`, `EMITIDOS` and
  `IMPLICITO` in `Esquema`, and two variants of `VerificaError`, `EsquemaNoRegistrado` and
  `EsquemaNoAceptado`. The head, the co-signature and the act choose the verifier by the scheme,
  and one that is not accepted is rejected with its number before reading the key. Three new rows,
  family FIRMA; the variants gain no row, which is the type's, and the dispatch is private. Not a
  byte, not a KAT, not a verdict of the command changes: with a single registered scheme, today's
  is the implicit one and it is accepted.
- §722 — `relevo_del_acta`, `CabezaDeLaCadena`,
  `juzgar_historia`, `Relevo`, with `contiene`, and `relevo_del_tramo` in the `actas` module: the
  `procedencia` of an act fixes in the MMR of heads the place of the key change, and two heads of
  one history fall next to it; the judge of the extension and of the consumption with `actas`, and
  the handover of the stretch in the overlap envelope (`spec/PAQUETE.md` 2.3 and 2.14). Five new
  rows, family ACTA. `RotacionError` gains eight variants —seven of the handover, for the two edges
  of the two heads and the one that goes backwards, and the previous one outside all its stretches—
  and gains no row: the row is the type's. After its review, the judge is the one of the eight
  places that demand continuity, and each head falls in the handover of its stretch by its two
  edges; the edge is computed by a private producer of the module, the same one
  `Relevo::contiene` uses, and it gains no row.
- §800 — the scheme registry (`Esquema`, with `TODOS`, `as_u64` and `entrada`, `EntradaEsquema` and
  `EsquemaNoRegistrado`) and the two v7 preambles, `preambulo_v7` and `preambulo_cofirma_v7`, in
  `zk-ssl-verify` (RFC-0020, E2): the number of the scheme that signs, the act's `u64`, in the
  head's preamble and in the co-signature's, with three KATs the second implementation
  reproduces. Eight new rows, family FIRMA; the error, REFERENCIA. `ESQUEMA_XMSSMT_SHA2_40_8_256`
  now comes from the registry with the same value. Switched off: neither the node, nor the witness,
  nor the kit sign or accept them until E3 and E4; the head, the version set and the 31 earlier
  KATs do not move.
- §773 — `DOMINIO_RECEP_V2`, `recibo_digest_v2` and `digest_pi` in `zk-ssl-hash` (RFC-0019 E3, first
  piece): the leaf of the reception receipt with the digest of the public inputs, which also ties
  the family and the version of the AIR (RFC-0018 D-G), with two KATs the second implementation
  reproduces. Three new rows, in ACUSES and RECIBOS. Nothing consumes it yet: the node, the wire
  and the kit stay with v1 until the `zkssl/0.5` cut.
- §686 — `Tramo`, with `contiene`, and `tramos_de` in the `actas` module: the stretches of a key in
  the chain, one for each time it enters, with the genesis not judging what was signed before
  opting in; the judge of the overlap envelope (RFC-0015, E5c). Three new rows, family ACTA.
- §648 — `juzgar_continuidad` in the `actas` module: two heads of distinct keys are ONE operator's
  if the chain leads from the one with the smaller embedded index to the other, with rules 3 and
  4 of D-C; the judge of the kit envelopes that read `actas` (RFC-0015, E5a). One new row, family
  ACTA. `RotacionError` gains one variant, `FueraDeTramo` —the overlap of rule 4, with its
  stretch—, and gains no row: the row is the type's.
- §647 — `verificar_cadena`, `Rotacion` with `en_su_tramo`, `RotacionError` and `juzgar_rotacion`
  in the `actas` module: the whole chain and the step from one key to another, judged by one
  producer that the node uses on start-up and the witness live and when auditing (RFC-0015, E4),
  with rules 3 and 4 of D-C —the `desde` of the successor above what the old one signed, and
  each head in the stretch of its key— and the overlap with a name. Five new rows,
  family ACTA; the error, REFERENCIA, like `ActaError`.
- §646 — `acta_a_json` and `acta_de_json` in the `actas` module: the act has a JSON and one
  producer, which writes the journal line and what `zkssl_keyActs` serves (RFC-0015, E3b-2), and
  which the envelopes of E5 will read. What is composed does not move: the `acta_digest` is
  computed over the fields, not over the JSON. An element of a digest worth `p` or more is not
  read: the rule of §640, which the reader inherits from `digest_from_bytes`. Two new rows, family
  ACTA.
- §643 — `DOMINIO_ACTA` and `acta_digest` in `zk-ssl-hash`, and the `actas` module of
  `zk-ssl-verify`: the key act of RFC-0015 (E2), with the byte domain `ZK-SSL-key-act` of its
  signature, and three KATs -the genesis, the rotation and the preamble-, which the second
  implementation reproduces. Twelve new rows, family ACTA. The head and the version set do not
  move: the act does not enter under the signature of any head.
- §662 — `cantidad_canonica`: the QUANTITY (a `u64` in hexadecimal) in its minimal encoding,
  without `+`, without upper case and without leading zeros, for the kit and the wire. One new
  row, REFERENCIA: it composes no signed byte; `HexError` gains `NoMinima`.
- §650 — `HexError`, `bytes_de_hex` and `hex_canonico`: a single hexadecimal reader, over bytes,
  for the wire, the kit, the SDK and the witness; the canonical one requires `0x` and `[0-9a-f]`,
  the form of the values of `PAQUETE.md`. Three new rows, REFERENCIA: they compose no signed byte.
- §640 — canonicity (RFC-0016): `MODULO` and `u64_canonico`, two new rows in the HASH family;
  `element_from_bytes` stops reducing and rejects what is not less than `p`; section 6 gains its
  paragraph «Canonicidad» [canonicity]. Not a byte of what the core produces moves: the 26 KATs,
  the same; what changes is which bytes and which integers are ACCEPTED on reading.

- §623 — the second implementation of the core
  (`tools/segunda/`, entry 85) reproduces the 26 KATs from this section and verifies the signed
  heads of the vectors with an XMSS^MT written from RFC 8391; it leaves three clarifications of
  prose in section 6 (the acknowledgement leaf with its `logSeq + 1`, the length of
  `hash_del_lote`, the peak with `mmr_hoja` inside) and one on the format of the KATs. Not a byte
  of the format moves.

- §610 — `hash_del_lote`: the core composes the fingerprint of a `zkssl_applyMany` batch
  (RFC-0014, E2), with `DOMINIO_LOTE` in the bytes family; its KAT. One new row, family RECIBOS.
  The receipt leaf and the head do not move: one receipt for what the node evaluates as a unit.
- §591 — `DOMINIO_ANCLA`, `ancla_digest` and `huella_de_clave`: the core composes the fingerprint
  of the head anchor and that of the operator's key (RFC-0012, E2); the KATs of the two functions.
  Three new rows, family ANCLA. The head and the version set do not move: the anchor is derived
  from what is already signed.
- §573 — `RECEP_DEPTH` and `raiz_de_camino_de_recibo` in the `recibos` module: the depth of the
  receipt tree and the rule of its path —the length and the crossing against the position
  `rx - Q - 1`—, which the completeness envelope uses and the node ties to its tree (RFC-0010,
  E4). Two new rows.
- §570 — `VERSION_FORMATO` goes from 5 to 6: the node composes, signs and serves the v6 head, with
  the reception pair its log gives (RFC-0010, E2d). No new row: the constant already had one and
  its value is not census.
- §567 — the `recibos` module changes what it says and not what it is called: the era is `(Q, R]`
  —the reception counter starts at 1— and the position `rx - Q - 1`; the era and the window are
  counted in the XMSS index of the signature and not in the head's `seq` (RFC-0010, E2b;
  corrections cited in its D-C and its D-D). No new row.
- §566 — `verificar_acuse_v6` and `verificar_inclusion_v6`: the acknowledgement and the inclusion
  against a v6 head are recomposed with v6. Until here the command ACCEPTED v6 in its step 1 and
  sent its acknowledgement to the v5 recomposer (RFC-0010, E2a). Two new rows.
- §562 — the verifier's `recibos` module (`pertenece_a_era`, `indice_de_recibo`,
  `era_de_recibo`, `hoja_de_recibo`, `dentro_de_ventana`): the rules of the reception receipt
  tree, shared by the node's builder and the verifier (RFC-0010, E2b). Five new rows. Entry that
  was missing: §562 added the rows and not their history, and §567 wrote it.
- §558 — the `V6` variant, `lleva_recepcion` and `texto_con_recepcion`, `lleva_parametros` and
  `texto_con_parametros`: the set a verifier ACCEPTS grows to v6, and the envelopes that demanded
  the state family stop asking <<is it V5?>> and ask for the predicate instead (RFC-0010, E2a).
  Four new rows; the variant is not a row, the `enum` already had it.
- §557 — `epoch_digest_v6`, `recibo_digest` and `DOMINIO_RECEP`: the core composes the v6 head and
  the leaf of the reception receipt (RFC-0010, E2); the KATs of both. Three new rows. The `V6`
  variant does NOT enter here: the set a verifier accepts moves in its own seal.
- §475 — the verifier's `cuentas` module (`is_right_de_indice`, `cruza_indice`, `raiz_de_hoja`,
  `no_existe`): the mirror of `congelados` over the account tree, with the EMPTY leaf as the rule,
  which is what sustains `AccountNotFound` without the layer (RFC-0007, E5, cut 3b). Four new
  rows; `ACCOUNTS_DEPTH` already entered with §474.
- §458 — `FROZEN_DEPTH` and the verifier's `congelados` module (`is_right_de_indice`,
  `cruza_indice`, `raiz_de_hoja`, `esta_congelada`): the depth of the frozen tree, fixed by the
  core, and the rule of its leaf (RFC-0007, E3b). Five new rows.
- §452 — `VERSION_FORMATO` goes from 4 to 5: the node signs and serves the v5 head (RFC-0007, E1b).
  No new row: the constant already had one and its value is not census.
- §451 — `epoch_digest_v5`, `params_digest` and `DOMINIO_PARAMS`, the `V5` variant,
  `lleva_consumos` and `texto_con_consumos`, `verificar_acuse_v5` and `verificar_inclusion_v5`:
  the core and the command accept the v5 head (RFC-0007, E1a); the KATs of `epoch_digest_v5` and
  `params_digest`. Seven new rows.
- §419 — the rules of the consumption tree in the verifier (RFC-0006, E3b-2): the empty leaf, the
  path convention and the crossing that ties the proof to the consumption's position. Five new
  rows.
- S416 - `CONS_DEPTH` and `posicion_de_consumo`: the consumption's position is recomposed without
  the layer (RFC-0006, E3a). Two new rows.
- §414 — `epoch_digest_v4`, the `V4` variant and `lleva_mmr`: the core and the command accept the
  v4 head (RFC-0006, E2a); the KAT of `epoch_digest_v4`. Five new rows.
- §411 — section 6 and the KATs of `spec/vectors/nucleo/`: the bytes, fixed (RFC-0005, E5).
- §407 — this document is born (RFC-0005, E1) with its tie `tools/check_nucleo.py` in the canon.
- §406 — `VersionCabeza`, the sole producer of the version set (E2).
- §404 — the witness's recomposer stops believing unknown versions.
- §405 — RFC-0005 enters as PROPOSED.
- Changing this document is changing the contract: it enters by RFC (`spec/rfc/PROCESO.md`).
