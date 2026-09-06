# El núcleo congelado — lo que un verificador recompone y no cambia

- **Estado:** normativa vigente desde §407 (RFC-0005, etapa E1)
- **Versión del protocolo:** `zkssl/0.3` — este documento no la mueve: el núcleo no cruza el cable
- **Origen:** `spec/rfc/0005-nucleo-congelado.md` (D-A, D-B, D-C); asiento que lo sella: §407
- **Implementación de referencia:** los crates `zk-ssl-verify` y `zk-ssl-hash`; el binario `zk-ssl-verify`
- **Atado:** `tools/check_nucleo.py`, en el canon: la tabla de este documento y los `pub` del árbol dicen lo mismo en las dos direcciones

Este documento es el **único productor normativo** de QUÉ no cambia en el protocolo y POR QUÉ. Enumera
—por derivación, no de memoria— cada elemento público que un tercero alcanza al verificar sin el
nodo, sin la capa y sin el probador (§243), y le pone una de cuatro clases. La cabecera de
`zk-ssl-verify` dice que «las familias se nombran; los elementos NO se enumeran; la verdad se mide en
los `pub`»: aquí se enumeran **porque se miden** — `tools/check_nucleo.py` deriva el censo del fuente
en cada canon y falla si la tabla y el árbol dejan de coincidir. Una lista que un gate no re-deriva
caduca a la primera ampliación, y la de esa cabecera ya caducó una vez (§247).

**Cita por nombre y fichero, nunca por línea.** Las líneas caducan con cada sello; los nombres,
sólo por RFC.

## 1. Qué es el núcleo

**Lo que se congela es el lado del verificador** (RFC-0005, D-A): el paquete de evidencia
(`spec/PAQUETE.md`) y todo lo que su verificación **recompone o comprueba bajo una firma**. Una
segunda implementación, escrita desde `spec/` sin leer el código de referencia, tiene que producir
**los mismos bytes** en cada una de estas piezas, o no verifica lo mismo.

El cable JSON-RPC (`spec/RPC.md`) queda **fuera**: es el contrato de interoperabilidad, gobernado por
el mismo proceso de RFC, y exige el probador para reproducirse. El libro (los árboles de cuentas,
pendientes y congelados) queda fuera: el paquete lo trata como raíces opacas bajo la cabeza firmada.

## 2. Las cuatro clases

- **NÚCLEO** — se firma, se compone o se comprueba. No cambia salvo por versión nueva del preámbulo
  (sección 3). Una segunda implementación lo reproduce byte a byte.
- **REFERENCIA** — de esta implementación, no del formato: tipos de error, el apaño del OID de
  `xmss 0.1.0-pre.0` (§240), la lectura de la clave. Una segunda implementación con una biblioteca
  correcta **no debe** replicarlos.
- **LIBRO** — composiciones del estado que el paquete no recompone (las trata como raíces opacas);
  las gobierna la capa y su propio RFC.
- **REGISTRO** — la familia de reverificación del registro de transiciones y los sellos de operación:
  la componen el nodo y el cliente por el cable (`spec/RPC.md`), y por eso no pertenecen al sobre.

## 3. La regla de extensión, ejercida

**Primera mitad — lo que se firma crece sólo por versión.** El conjunto de versiones de cabeza que
el núcleo acepta tiene **un solo productor**: `VersionCabeza` en `zk-ssl-verify` (§406), un `enum`
exhaustivo cuyo texto («v2 o v3») se deriva de sus variantes y del que el mando y el testigo
**consumen**, sin repetirlo. Una composición nueva es una variante nueva: el compilador marca cada
`match` que la olvide, y `VERSION_FORMATO` tiene que ser miembro (atado en los tests del crate).
Una `formatVersion` fuera del conjunto se rechaza con texto y **sin truncar**: `0x103` no es un 3.

**Segunda mitad — lo que no se firma no existe para el núcleo.** Una clave del sobre que ninguna
composición ni ningún preámbulo lee no puede alterar lo que se comprueba; el verificador de
referencia las ignora (cero atributos `serde`, catorce claves leídas por `.get()`). Que el consumidor
del cable de la referencia rechace además claves desconocidas es elección declarada de esa
implementación, no regla del formato.

**Familias nuevas** entran con su `tipo` o su dominio, sus vectores y su RFC. Si necesitan entrar
bajo la firma de la cabeza, entran por la primera mitad, como versión nueva del preámbulo.

## 4. El censo

**Censo derivado:** 50 elementos alcanzables en `zk-ssl-verify` y 37 `pub` en `zk-ssl-hash`
(LIBRO 2, NÚCLEO 63, REFERENCIA 7, REGISTRO 15). Alcanzable en `zk-ssl-verify` es lo que
`lib.rs` exporta: sus propios `pub`, todo lo `pub` de los módulos `pub mod` (`acuses`, `mmr`) y los
nombres que sus `pub use` sacan de los módulos privados (`inclusion`, `reverificacion`). Las
reexportaciones de `zk-ssl-hash` no se cuentan dos veces: un elemento, una fila. En `zk-ssl-hash`,
todo `pub` de `lib.rs` fuera de las zonas de test. Las zonas de test se recortan por el anidamiento
real de sus llaves, no por la primera marca.

| elemento | fichero | clase | familia | qué es |
|---|---|---|---|---|
| `Digest` | `hash/lib.rs` | NÚCLEO | HASH | `type` |
| `FormatoError` | `hash/lib.rs` | REFERENCIA | HASH | `enum` |
| `CONS_DEPTH` | `hash/lib.rs` | NÚCLEO | HASH | `const` |
| `STATE_WIDTH` | `hash/lib.rs` | NÚCLEO | HASH | `const` |
| `as_digest` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `digest_from_bytes` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `digest_to_bytes` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `element_from_bytes` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `element_to_bytes` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `embeber` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `native_merge` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `path_root` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `posicion_de_consumo` | `hash/lib.rs` | NÚCLEO | HASH | `fn` |
| `epoch_digest` | `hash/lib.rs` | NÚCLEO | CABEZA | `fn` |
| `epoch_digest_v2` | `hash/lib.rs` | NÚCLEO | CABEZA | `fn` |
| `epoch_digest_v3` | `hash/lib.rs` | NÚCLEO | CABEZA | `fn` |
| `epoch_digest_v4` | `hash/lib.rs` | NÚCLEO | CABEZA | `fn` |
| `ANCHO_INDICE` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `COFIRMA_VERSION` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `COFIRMA_V_MAX` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `CabezaFirmada` | `verify/lib.rs` | NÚCLEO | FIRMA | `struct` |
| `Conjunto` | `verify/lib.rs` | NÚCLEO | FIRMA | `type` |
| `DOMINIO` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `DOMINIO_COFIRMA` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `FIRMA_RFC_BYTES` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `OFFSET_MT_UPSTREAM` | `verify/lib.rs` | REFERENCIA | FIRMA | `const` |
| `TODAS` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `VERSION_FORMATO` | `verify/lib.rs` | NÚCLEO | FIRMA | `const` |
| `VerificaError` | `verify/lib.rs` | REFERENCIA | FIRMA | `enum` |
| `VersionCabeza` | `verify/lib.rs` | NÚCLEO | FIRMA | `enum` |
| `VersionCabezaDesconocida` | `verify/lib.rs` | REFERENCIA | FIRMA | `struct` |
| `aplicar_apano_del_oid` | `verify/lib.rs` | REFERENCIA | FIRMA | `fn` |
| `as_u8` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `clave_desde_bytes` | `verify/lib.rs` | REFERENCIA | FIRMA | `fn` |
| `indice_de_firma` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `lleva_mmr` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `preambulo` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `preambulo_cofirma` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `texto` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `texto_con_mmr` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `verificar_cabeza` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `verificar_cofirma` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `DOMINIO_ACUSE` | `hash/lib.rs` | NÚCLEO | ACUSES | `const` |
| `acuse_digest` | `hash/lib.rs` | NÚCLEO | ACUSES | `fn` |
| `epoca_de_acuse` | `verify/acuses.rs` | NÚCLEO | ACUSES | `fn` |
| `hoja_de_acuse` | `verify/acuses.rs` | NÚCLEO | ACUSES | `fn` |
| `indice_de_hoja` | `verify/acuses.rs` | NÚCLEO | ACUSES | `fn` |
| `pertenece` | `verify/acuses.rs` | NÚCLEO | ACUSES | `fn` |
| `ReciboAcuse` | `verify/inclusion.rs` | NÚCLEO | ACUSES | `struct` |
| `verificar_acuse` | `verify/inclusion.rs` | NÚCLEO | ACUSES | `fn` |
| `verificar_acuse_v3` | `verify/inclusion.rs` | NÚCLEO | ACUSES | `fn` |
| `verificar_acuse_v4` | `verify/inclusion.rs` | NÚCLEO | ACUSES | `fn` |
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
| `native_leaf` | `hash/lib.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `native_leaf_salted` | `hash/lib.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `InclusionError` | `verify/inclusion.rs` | REFERENCIA | INCLUSIÓN | `enum` |
| `ReciboInclusion` | `verify/inclusion.rs` | NÚCLEO | INCLUSIÓN | `struct` |
| `verificar_inclusion` | `verify/inclusion.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `verificar_inclusion_v2` | `verify/inclusion.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `verificar_inclusion_v3` | `verify/inclusion.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `verificar_inclusion_v4` | `verify/inclusion.rs` | NÚCLEO | INCLUSIÓN | `fn` |
| `DOMINIO_META_PENDIENTE` | `hash/lib.rs` | LIBRO | LIBRO | `const` |
| `meta_pendiente_hoja` | `hash/lib.rs` | LIBRO | LIBRO | `fn` |
| `COMPROMISO_AUSENTE` | `hash/lib.rs` | REGISTRO | REGISTRO | `const` |
| `OP_FREEZE` | `hash/lib.rs` | REGISTRO | REGISTRO | `const` |
| `OP_GOVERNANCE` | `hash/lib.rs` | REGISTRO | REGISTRO | `const` |
| `OP_MINT` | `hash/lib.rs` | REGISTRO | REGISTRO | `const` |
| `OP_MINT_PENDING` | `hash/lib.rs` | REGISTRO | REGISTRO | `const` |
| `OP_RECOVERY` | `hash/lib.rs` | REGISTRO | REGISTRO | `const` |
| `commit_operation` | `hash/lib.rs` | REGISTRO | REGISTRO | `fn` |
| `digest_of_proof` | `hash/lib.rs` | REGISTRO | REGISTRO | `fn` |
| `sello_de_autorizacion` | `hash/lib.rs` | REGISTRO | REGISTRO | `fn` |
| `sello_sin_prueba` | `hash/lib.rs` | REGISTRO | REGISTRO | `fn` |
| `EntradaLog` | `verify/reverificacion.rs` | REGISTRO | REGISTRO | `struct` |
| `ReverificacionError` | `verify/reverificacion.rs` | REGISTRO | REGISTRO | `enum` |
| `Veredicto` | `verify/reverificacion.rs` | REGISTRO | REGISTRO | `enum` |
| `censo` | `verify/reverificacion.rs` | REGISTRO | REGISTRO | `fn` |
| `reverificar` | `verify/reverificacion.rs` | REGISTRO | REGISTRO | `fn` |

## 5. Por qué no puede cambiar, por familia

- **HASH** — la permutación y el merge 2-a-1, cómo se embebe un `u64`, cómo se sube un camino y cómo
  un digest se escribe en bytes: es la frontera entre el JSON y los bytes. Dos implementaciones que
  difieran aquí no coinciden en nada.
- **CABEZA** — las cuatro composiciones del digest de la cabeza (v1, v2, v3, v4). Lo custodiado no caduca
  (§290): una composición vieja tiene que poder recomponerse siempre.
- **FIRMA** — el esquema (`XmssMtSha2_40_8_256`), los dos dominios, el byte de versión y su conjunto,
  los dos preámbulos y el índice embebido en la firma. Son los bytes exactos bajo la firma; cambiar
  uno haría colisionar o dejaría de verificar lo custodiado.
- **ACUSES** — las reglas del árbol de acuses y su composición: el nodo las construye y el
  verificador las comprueba llamando las mismas (§274).
- **MMR** — el MMR de cabezas: la hoja, el nodo, la cima, la inclusión y la consistencia (§291).
- **INCLUSIÓN** — la hoja de cuenta (las dos formas) y los recibos de inclusión.

## 6. Los bytes: lo que un KAT fija

Este documento **nombra**; los bytes los fijan los vectores de `spec/vectors/nucleo/` (§411,
RFC-0005 E5): un fichero por `fn` NÚCLEO, `{fn, entradas, salida}` en hex `0x…`, emitidos por la
referencia y reproducidos en cada canon (`zk-ssl-cli`, `nucleo_kat`). Una segunda implementación
que dé estos bytes en cada uno da los mismos bytes en todo lo que se firma. Son una foto de la
referencia, y se declara: fijan la propiedad «dos implementaciones dan estos bytes».

- **La permutación**: Rescue-Prime `Rp64_256`, tal como la implementa `winter-crypto =0.13.1`,
  sobre el campo de Goldilocks (`winter-math =0.13.1`, `f64::BaseElement`). Un `Digest` son cuatro
  elementos. `native_merge(l, r)`: estado de doce elementos a cero, `l` en `[4..8]`, `r` en
  `[8..12]`, una permutación, salida `[4..8]`; la capacidad `[0..4]` queda a cero. `embeber(x) =
  [x, 0, 0, 0]`; `as_digest(u)` embebe el `u64`.
- **Serialización**: cada elemento en 8 bytes *little-endian* (`as_int`), los cuatro en orden;
  `digest_from_bytes` exige 32 bytes. En el cable van como `DATA`/`Digest` (`RPC.md`).
- **Los dominios**: `u64` leídos *big-endian* de ocho bytes ASCII (`ACUSE_V1`, `MMRHOJA1`,
  `MMRNODO1`), embebidos con `as_digest` y mezclados por delante. Los de la firma son cadenas de
  bytes: `b"ZK-SSL-epoch-head"` (17) y `b"ZK-SSL-witness-cosign"` (21).
- **Los preámbulos** (mudados aquí desde `zk-ssl-verify/src/lib.rs`, que remite a esta sección):

  ```text
  b"ZK-SSL-epoch-head" ‖ version ‖ epoch_digest                            (17 + 1 + 32 = 50)
  b"ZK-SSL-witness-cosign" ‖ version ‖ epoch_digest ‖ len(u16 BE) ‖ clave_op     (21 + 1 + 32 + 2 + N)
  ```

- **Las composiciones**, en el orden exacto de los merges: `native_leaf = merge(merge(pk,
  embeber(saldo)), embeber(nonce))` y la salteada `merge(hoja, salt)`; `path_root` sube desde la
  hoja con el hermano a la izquierda si `is_right`; `epoch_digest = merge(merge(merge(as_digest(seq),
  accounts), merge(pending, frozen)), chain)`; `v2 = merge(v1, merge(acuses_root, as_digest(n)))`;
  `v3 = merge(v2, merge(cima, as_digest(t)))`, génesis `as_digest(0)` y `t = 0`;
  `v4 = merge(v3, merge(cons_root, as_digest(cons_count)))` (RFC-0006 E2a, §414), génesis la raíz
  del árbol de consumos vacío y `cons_count = 0`;
  `acuse_digest = merge(as_digest(ACUSE_V1), merge(hash_prueba, merge(as_digest(epoca),
  as_digest(n))))`; `mmr_hoja = merge(as_digest(MMRHOJA1), cabeza)`; `mmr_nodo =
  merge(as_digest(MMRNODO1), merge(izq, der))`; la cima es el árbol de Merkle con el corte en
  la mayor potencia de dos menor que `n`.
- **Lo que un KAT no puede dar**: la firma. `XmssMtSha2_40_8_256` es RFC 8391 y la clave
  publicada lleva su OID correcto; el apaño del OID es de lectura de `xmss 0.1.0-pre.0`
  (REFERENCIA) y una biblioteca correcta no lo necesita.

## 7. Lo que este documento NO afirma

- No congela el cable ni el libro: `spec/RPC.md` y la capa tienen sus propias reglas y sus propios RFC.
- No afirma que una segunda implementación exista (E5 del RFC-0005): afirma qué tendría que reproducir.
- Que exista un verificador independiente no hace las firmas oponibles: sigue faltando la custodia
  declarada de la clave del operador (`SECURITY.md`).
- Nada del núcleo identifica nada fuera del libro (RFC-0005, D-C): ni una factura, ni una etiqueta
  compartida entre operadores. La unicidad entre libros es familia nueva con RFC propio.

## 8. Historia

- S416 - `CONS_DEPTH` y `posicion_de_consumo`: la posicion del consumo se recompone sin la
  capa (RFC-0006, E3a). Dos filas nuevas.
- §414 — `epoch_digest_v4`, la variante `V4` y `lleva_mmr`: el núcleo y el mando aceptan la cabeza
  v4 (RFC-0006, E2a); el KAT de `epoch_digest_v4`. Cinco filas nuevas.
- §411 — la sección 6 y los KAT de `spec/vectors/nucleo/`: los bytes, fijados (RFC-0005, E5).
- §407 — nace este documento (RFC-0005, E1) con su atado `tools/check_nucleo.py` en el canon.
- §406 — `VersionCabeza`, el único productor del conjunto de versiones (E2).
- §404 — el recompositor del testigo deja de creer versiones desconocidas.
- §405 — el RFC-0005 entra como PROPUESTO.
- Cambiar este documento es cambiar el contrato: entra por RFC (`spec/rfc/PROCESO.md`).
