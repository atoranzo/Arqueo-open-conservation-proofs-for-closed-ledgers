# El núcleo congelado — lo que un verificador recompone y no cambia

- **Estado:** normativa vigente desde §407 (RFC-0005, etapa E1)
- **Versión del protocolo:** `zkssl/0.4` — este documento no la mueve: el núcleo no cruza el cable
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
Desde el §458 hay una excepción, y es por una prueba: del árbol de congelados entran su
profundidad y la regla de su hoja, porque el rechazo `AccountFrozen` se prueba con un camino bajo
`frozenRoot` (RFC-0007, E3b; familia CONGELADOS).

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
exhaustivo cuyo texto («v2, v3, v4, v5 o v6» desde el §558; «v2, v3, v4 o v5» desde el §451) se deriva de sus variantes y del que el mando y el testigo
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

**Censo derivado:** 98 elementos alcanzables en `zk-ssl-verify` y 60 `pub` en `zk-ssl-hash`
(LIBRO 5, NÚCLEO 125, REFERENCIA 13, REGISTRO 15). Alcanzable en `zk-ssl-verify` es lo que
`lib.rs` exporta: sus propios `pub`, todo lo `pub` de los módulos `pub mod` (`acuses`, `mmr`, `consumos`, `congelados`,
`cuentas`, `recibos`, `actas`) y los
nombres que sus `pub use` sacan de los módulos privados (`inclusion`, `reverificacion`). Las
reexportaciones de `zk-ssl-hash` no se cuentan dos veces: un elemento, una fila. En `zk-ssl-hash`,
todo `pub` de `lib.rs` fuera de las zonas de test. Las zonas de test se recortan por el anidamiento
real de sus llaves, no por la primera marca.

| elemento | fichero | clase | familia | qué es |
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
| `lleva_consumos` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `lleva_recepcion` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `lleva_parametros` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `preambulo` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
| `preambulo_cofirma` | `verify/lib.rs` | NÚCLEO | FIRMA | `fn` |
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
  difieran aquí no coinciden en nada. Desde el §640 (RFC-0016) la familia fija también **qué es una
  escritura válida**: el módulo `p` y la lectura canónica, sin la cual una misma firma acreditaba dos
  enteros y las dos implementaciones del núcleo leían distinto el mismo byte.
- **CABEZA** — las cinco composiciones del digest de la cabeza (v1, v2, v3, v4, v5) y el digest de
  los parámetros que v5 firma. Lo custodiado no caduca (§290): una composición vieja tiene que
  poder recomponerse siempre.
- **FIRMA** — el esquema (`XmssMtSha2_40_8_256`), los dos dominios, el byte de versión y su conjunto,
  los dos preámbulos y el índice embebido en la firma. Son los bytes exactos bajo la firma; cambiar
  uno haría colisionar o dejaría de verificar lo custodiado.
- **ACUSES** — las reglas del árbol de acuses y su composición: el nodo las construye y el
  verificador las comprueba llamando las mismas (§274).
- **MMR** — el MMR de cabezas: la hoja, el nodo, la cima, la inclusión y la consistencia (§291).
- **INCLUSIÓN** — la hoja de cuenta (las dos formas) y los recibos de inclusión.
- **CONGELADOS** — la profundidad del árbol de congelados, el cruce del camino con el índice de la
  cuenta y la regla de la hoja (congelada es no vacía): lo que sostiene `AccountFrozen` sin la capa
  (§458). La profundidad la fija el núcleo porque el merge no separa hoja de nodo.
- **CUENTAS** — el espejo del anterior sobre el árbol de cuentas: la profundidad que fija el
  núcleo, el cruce del camino con el índice y la regla de la hoja (no existir es hoja VACIA): lo
  que sostiene `AccountNotFound` sin la capa (§475). `ACCOUNTS_DEPTH` vale hoy lo mismo que
  `FROZEN_DEPTH` y son dos hechos distintos.
- **ANCLA** — la huella del ancla de cabezas y la huella de la clave del operador (RFC-0012):
  lo que se publica en un medio ajeno al operador y lo que un tercero recompone para compararlo.
  Un solo productor: dos implementaciones que difieran aquí no comparan la misma ancla.
- **ACTA** — el acta con que una clave del operador entra, y compromete a su sucesora
  (RFC-0015, E2): su huella, el preámbulo que firma, el esquema que presenta y las reglas con que
  un tercero juzga una rotación contra el acta previa. Una implementación que componga otra
  huella no reconoce la sucesora comprometida, y una rotación legítima le parece un robo.

## 6. Los bytes: lo que un KAT fija

Este documento **nombra**; los bytes los fijan los vectores de `spec/vectors/nucleo/` (§411,
RFC-0005 E5): un fichero por `fn` NÚCLEO, `{fn, entradas, salida}` en hex `0x…`, emitidos por la
referencia y reproducidos en cada canon (`zk-ssl-cli`, `nucleo_kat`). ⚠️ Los `u64` de las entradas
van como **número** hex (`seq`, `n`, `t`, los contadores, `as_digest.x`), salvo `embeber.x` y
`balance`/`nonce` de `native_leaf*`, que van como **ocho bytes little-endian**: dos codificaciones
en un catálogo, y un juez tiene que saberlo por fuera (medido por la segunda implementación,
`tools/segunda/juez_nucleo.py`). Una segunda implementación
que dé estos bytes en cada uno da los mismos bytes en todo lo que se firma. Son una foto de la
referencia, y se declara: fijan la propiedad «dos implementaciones dan estos bytes».

- **La permutación**: Rescue-Prime `Rp64_256`, tal como la implementa `winter-crypto =0.13.1`,
  sobre el campo de Goldilocks (`winter-math =0.13.1`, `f64::BaseElement`). Un `Digest` son cuatro
  elementos. `native_merge(l, r)`: estado de doce elementos a cero, `l` en `[4..8]`, `r` en
  `[8..12]`, una permutación, salida `[4..8]`; la capacidad `[0..4]` queda a cero. `embeber(x) =
  [x, 0, 0, 0]`; `as_digest(u)` embebe el `u64`.
- **Serialización**: cada elemento en 8 bytes *little-endian* (`as_int`), los cuatro en orden;
  `digest_from_bytes` exige 32 bytes. En el cable van como `DATA`/`Digest` (`RPC.md`).
- **Canonicidad** (RFC-0016, §640). El campo es `F_p` con `p = MODULO = 2^64 - 2^32 + 1`. Un `u64`
  **escribe** un elemento sólo si es menor que `p`; los `2^32 - 1` valores de `[p, 2^64)` no escriben
  ninguno. Dos reglas, con un solo productor, `u64_canonico`:

  1. **Un elemento se lee canónico.** `element_from_bytes`, y con él `digest_from_bytes`, rechaza
     los ocho bytes de un valor `>= p` con `0x… no es canonico: no es menor que p = 2^64 - 2^32 + 1`,
     en vez de reducirlo. Así escribir y leer son inversas exactas: `element_to_bytes` es una
     biyección de `F_p` sobre `[0, p)`, y un digest tiene **una** escritura de 32 bytes.
  2. **Un `u64` que entra en una composición se lee canónico.** `as_digest(x) = [x mod p, 0, 0, 0]`
     es inyectiva en `[0, p)` y en ningún sitio más: `as_digest(x + p) = as_digest(x)` para
     `x < 2^32 - 1`. La función no cambia y sus KAT no se mueven; quien lee de fuera un `u64` que
     va a componer (`seq`, `n`, `t`, los contadores, las marcas, el índice) lo pasa por
     `u64_canonico` antes, y lo rechaza con el mismo texto si no cabe.
  3. **Un productor reduce.** Lo que sale de Blake3 —`digest_of_proof`, `huella_de_clave`,
     `hash_del_lote`— son 32 bytes que se leen como cuatro `u64` *little-endian* y se REDUCEN cada
     uno módulo `p` (`BaseElement::new`): la salida es un digest canónico y una función de los bytes,
     aunque un limbo no quepa (`2^-32` por limbo). Reducir es de quien produce; rechazar, de quien lee
     una escritura que llega de fuera. La segunda implementación devolvía los bytes de Blake3 tal cual
     y rechazaba el limbo que no cabía: divergía aquí también, y desde el §640 reduce.

  La consecuencia, que es lo que el núcleo promete: **sobre lo canónico, cada composición de esta
  sección que produce un digest es inyectiva en sus entradas salvo colisión del hash**, con una
  excepción que no es del hash sino de la forma: en `path_root`, un nivel cuyo hermano es igual al
  nodo que sube da lo mismo con las dos orientaciones (`merge(x, x)`). Por eso la posición no se lee
  nunca de la raíz: donde la posición afirma algo, el verificador cruza el camino contra el índice
  (CONSUMO, CONGELADOS, CUENTAS, RECIBOS). Fuera de lo canónico
  no había tal función: una cabeza firmada con `n` verificaba igual con `n + p`, y quien razonaba
  sobre el entero -la ventana de la promesa, el tamaño del MMR- razonaba sobre uno que la firma no
  fija (medido en el §640 con vectores reales, que ahora son negativos con nombre).
- **Los dominios**: `u64` leídos *big-endian* de ocho bytes ASCII (`ACUSE_V1`, `MMRHOJA1`,
  `MMRNODO1`, `PARAM_V1`), embebidos con `as_digest` y mezclados por delante. Los de la firma
  son cadenas de bytes: `b"ZK-SSL-epoch-head"` (17), `b"ZK-SSL-witness-cosign"` (21) y, desde el
  §643, `b"ZK-SSL-key-act"` (14), el del acta.
- **Los preámbulos** (mudados aquí desde `zk-ssl-verify/src/lib.rs`, que remite a esta sección):

  ```text
  b"ZK-SSL-epoch-head" ‖ version ‖ epoch_digest                            (17 + 1 + 32 = 50)
  b"ZK-SSL-witness-cosign" ‖ version ‖ epoch_digest ‖ len(u16 BE) ‖ clave_op     (21 + 1 + 32 + 2 + N)
  b"ZK-SSL-key-act" ‖ version ‖ acta_digest                                (14 + 1 + 32 = 47)
  ```

- **Las composiciones**, en el orden exacto de los merges: `native_leaf = merge(merge(pk,
  embeber(saldo)), embeber(nonce))` y la salteada `merge(hoja, salt)`; `path_root` sube desde la
  hoja con el hermano a la izquierda si `is_right`; `epoch_digest = merge(merge(merge(as_digest(seq),
  accounts), merge(pending, frozen)), chain)`; `v2 = merge(v1, merge(acuses_root, as_digest(n)))`;
  `v3 = merge(v2, merge(cima, as_digest(t)))`, génesis `as_digest(0)` y `t = 0`;
  `v4 = merge(v3, merge(cons_root, as_digest(cons_count)))` (RFC-0006 E2a, §414), génesis la raíz
  del árbol de consumos vacío y `cons_count = 0`;
  `v5 = merge(v4, merge(merge(params_digest, pmeta_root), merge(as_digest(next_pending),
  merge(as_digest(next_index), as_digest(total_supply)))))` (RFC-0007 E1a, §451), génesis los
  parámetros de apertura, la raíz del árbol de meta vacío, `next_pending = 0` y las cuentas y el
  suministro de esa cabeza;
  `v6 = merge(v5, merge(recep_root, as_digest(recep_count)))` (RFC-0010 E2, §557), génesis la raíz
  del árbol de recibos vacío y `recep_count = 0`;
  `recibo_digest = merge(as_digest(RECEP_V1), merge(hash_prueba, merge(as_digest(era),
  as_digest(n))))`, el molde de `acuse_digest` con el séptimo dominio;
  `ancla_digest = merge(as_digest(ANCLA_V1), merge(huella_clave, merge(as_digest(indice),
  merge(epoch_digest, merge(mmr_root, as_digest(mmr_size))))))` (RFC-0012, §591), con el octavo
  dominio y el índice EMBEBIDO en la firma;
  `huella_de_clave = Blake3(b"ZK-SSL-anchor-key-v1" ‖ len(u64 LE) ‖ clave)`, el molde de
  `digest_of_proof` (§116) con dominio de bytes propio;
  `acta_digest = merge(as_digest(ACTAS_V1), merge(as_digest(etiqueta), resto))` (RFC-0015,
  §643), con `cuerpo = merge(huella_clave, merge(as_digest(esquema), merge(as_digest(desde),
  siguiente)))`; en el acta génesis `etiqueta = 0` y `resto = cuerpo`; en una rotación
  `etiqueta = 1` y `resto = merge(anterior, merge(merge(epoch_digest, merge(mmr_root,
  as_digest(mmr_size))), cuerpo))`, con `anterior` y `siguiente` huellas de clave, `desde` el
  índice EMBEBIDO de la primera firma de la clave que entra, y `esquema` el de la clave de hoy,
  `ESQUEMA_XMSSMT_SHA2_40_8_256 = 0x1_0000_0005` (la familia de RFC 8391 en los 32 bits altos
  y su OID en los bajos);
  `hash_del_lote = Blake3(b"ZK-SSL-batch-v1" ‖ len(u64 LE) ‖ (hash_prueba_i ‖ cuenta_i ‖
  posicion_i)*)` (RFC-0014, §610), el mismo molde, con 48 bytes por operación en el orden del lote
  -el digest de su prueba, y su cuenta y su posición como `u64 LE`-; `len` son los **bytes** de la
  composición, `48·k`, como en el molde del §116, y `k` queda implícito en ella; es lo que va como
  `hash_prueba` en el `recibo_digest` de un lote. ⚠️ **Corregido por la segunda implementación**
  (`tools/segunda/`, entrada 85): hasta entonces aquí decía «`k` en la longitud», y con `len = k`
  el KAT no reproduce (medido); el error se registra en vez de borrarse;
  `params_digest = merge(as_digest(PARAM_V1),
  merge(merge(as_digest(regulatory_limit), as_digest(max_supply)),
  merge(merge(as_digest(max_accounts), custodian_set_root), merge(governance_set_root,
  merge(as_digest(refund_ttl), as_digest(max_custodian_uses))))))`, los siete en ese orden;
  `acuse_digest = merge(as_digest(ACUSE_V1), merge(hash_prueba, merge(as_digest(epoca),
  as_digest(n))))`; `mmr_hoja = merge(as_digest(MMRHOJA1), cabeza)`; `mmr_nodo =
  merge(as_digest(MMRNODO1), merge(izq, der))`; la cima es el árbol de Merkle con el corte en
  la mayor potencia de dos menor que `n`, **con cada cabeza pasada por `mmr_hoja` dentro de la
  cima** (la lectura de RFC 6962: `cima([c]) = mmr_hoja(c)`); ⚠️ precisado por la segunda
  implementación (`tools/segunda/`, entrada 85): hasta entonces no se decía, y el KAT lo fija.
  **La hoja de acuse**, que `PAQUETE.md` 2.1 nombra `hoja_de_acuse(hashPrueba, seq, n)` y hasta
  aquí no estaba escrita: es `acuse_digest(hashPrueba, seq + 1, n)`, porque su `seq` es el
  `logSeq` de la operación y la época del acuse es `logSeq + 1`, la primera cabeza que puede
  contenerla (`RPC.md`, «El acuse en la respuesta», §274); con `epoca = seq` el KAT no reproduce
  (medido).
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

- §648 — `juzgar_continuidad` en el módulo `actas`: dos cabezas de claves distintas son de UN
  operador si la cadena lleva de la de índice embebido menor a la otra, con las reglas 3 y 4 de la
  D-C; el juez de los sobres del kit que leen `actas` (RFC-0015, E5a). Una fila nueva, familia
  ACTA. `RotacionError` gana una variante, `FueraDeTramo` —el solapamiento de la regla 4, con su
  tramo—, y no gana fila: la fila es del tipo.
- §647 — `verificar_cadena`, `Rotacion` con `en_su_tramo`, `RotacionError` y `juzgar_rotacion`
  en el módulo `actas`: la cadena entera y el paso de una clave a otra, juzgados por un
  productor que usan el nodo al arrancar y el testigo en vivo y al auditar (RFC-0015, E4), con
  las reglas 3 y 4 de la D-C —el `desde` de la sucesora por encima de lo que la vieja firmó, y
  cada cabeza en el tramo de su clave— y el solapamiento con nombre. Cinco filas nuevas,
  familia ACTA; el error, REFERENCIA, como `ActaError`.
- §646 — `acta_a_json` y `acta_de_json` en el módulo `actas`: el acta tiene un JSON y un
  productor, que escribe la línea del diario y lo que sirve `zkssl_keyActs` (RFC-0015, E3b-2), y
  que leerán los sobres de la E5. Lo que se compone no se mueve: el `acta_digest` se calcula sobre
  los campos, no sobre el JSON. Un elemento de un digest que vale `p` o más no se lee: la regla del
  §640, que el lector hereda de `digest_from_bytes`. Dos filas nuevas, familia ACTA.
- §643 — `DOMINIO_ACTA` y `acta_digest` en `zk-ssl-hash`, y el módulo `actas` de
  `zk-ssl-verify`: el acta de clave del RFC-0015 (E2), con el dominio de bytes `ZK-SSL-key-act`
  de su firma, y tres KAT -la génesis, la rotación y el preámbulo-, que la segunda implementación
  reproduce. Doce filas nuevas, familia ACTA. La cabeza y el conjunto de versiones no se mueven:
  el acta no entra bajo la firma de ninguna cabeza.
- §662 — `cantidad_canonica`: la QUANTITY (un `u64` en hexadecimal) en su escritura minima, sin
  `+`, sin mayusculas y sin ceros a la izquierda, para el kit y el cable. Una fila nueva,
  REFERENCIA: no compone ningun byte firmado; `HexError` gana `NoMinima`.
- §650 — `HexError`, `bytes_de_hex` y `hex_canonico`: un solo lector de hexadecimal, sobre
  bytes, para el cable, el kit, el SDK y el testigo; el canónico exige `0x` y `[0-9a-f]`, la forma
  de los valores de `PAQUETE.md`. Tres filas nuevas, REFERENCIA: no componen ningún byte firmado.
- §640 — la canonicidad (RFC-0016): `MODULO` y `u64_canonico`, dos filas nuevas en la familia
  HASH; `element_from_bytes` deja de reducir y rechaza lo que no es menor que `p`; la sección 6
  gana su párrafo «Canonicidad». Ningún byte de lo que el núcleo produce se mueve: los 26 KAT,
  iguales; lo que cambia es qué bytes y qué enteros se ACEPTAN al leer.

- §623 — la segunda implementación del núcleo
  (`tools/segunda/`, entrada 85) reproduce los 26 KAT desde esta sección y verifica las cabezas
  firmadas de los vectores con XMSS^MT escrito desde RFC 8391; deja tres precisiones de prosa en la
  sección 6 (la hoja de acuse con su `logSeq + 1`, la longitud de `hash_del_lote`, la cima con
  `mmr_hoja` dentro) y una sobre el formato de los KAT. Ningún byte del formato se mueve.

- §610 — `hash_del_lote`: el núcleo compone la huella de un lote de `zkssl_applyMany` (RFC-0014,
  E2), con `DOMINIO_LOTE` en la familia bytes; su KAT. Una fila nueva, familia RECIBOS. La hoja del
  recibo y la cabeza no se mueven: un recibo por lo que el nodo evalúa como unidad.
- §591 — `DOMINIO_ANCLA`, `ancla_digest` y `huella_de_clave`: el núcleo compone la huella del
  ancla de cabezas y la de la clave del operador (RFC-0012, E2); los KAT de las dos funciones.
  Tres filas nuevas, familia ANCLA. La cabeza y el conjunto de versiones no se mueven: el ancla
  se deriva de lo ya firmado.
- §573 — `RECEP_DEPTH` y `raiz_de_camino_de_recibo` en el módulo `recibos`: la profundidad del árbol
  de recibos y la regla de su camino —la medida y el cruce con la posición `rx - Q - 1`—, que el
  sobre de completitud usa y el nodo ata a su árbol (RFC-0010, E4). Dos filas nuevas.
- §570 — `VERSION_FORMATO` pasa de 5 a 6: el nodo compone, firma y sirve la cabeza v6, con la
  pareja de recepción que su registro da (RFC-0010, E2d). Ninguna fila nueva: la constante ya la
  tenía y su valor no es censo.
- §567 — el módulo `recibos` cambia lo que dice y no cómo se llama: la era es `(Q, R]` —el
  contador de recepción empieza en 1— y la posición `rx - Q - 1`; la era y la ventana se cuentan
  en el índice XMSS de la firma y no en el `seq` de la cabeza (RFC-0010, E2b; correcciones
  citadas en su D-C y su D-D). Ninguna fila nueva.
- §566 — `verificar_acuse_v6` y `verificar_inclusion_v6`: el acuse y la inclusión contra una
  cabeza v6 se recomponen con la v6. Hasta aquí el mando ACEPTABA la v6 en su paso 1 y mandaba su
  acuse al recomponedor v5 (RFC-0010, E2a). Dos filas nuevas.
- §562 — el módulo `recibos` del verificador (`pertenece_a_era`, `indice_de_recibo`,
  `era_de_recibo`, `hoja_de_recibo`, `dentro_de_ventana`): las reglas del árbol de recibos de
  recepción, compartidas por el constructor del nodo y el verificador (RFC-0010, E2b). Cinco
  filas nuevas. Entrada que faltaba: el §562 añadió las filas y no su historia, y la escribió
  el §567.
- §558 — la variante `V6`, `lleva_recepcion` y `texto_con_recepcion`, `lleva_parametros` y
  `texto_con_parametros`: el conjunto que un verificador ACEPTA crece a v6, y los sobres que
  exigian la familia del estado dejan de preguntar <<¿es V5?>> para preguntar por el predicado
  (RFC-0010, E2a). Cuatro filas nuevas; la variante no es fila, el `enum` ya la tenía.
- §557 — `epoch_digest_v6`, `recibo_digest` y `DOMINIO_RECEP`: el núcleo compone la cabeza v6 y la
  hoja del recibo de recepción (RFC-0010, E2); los KAT de las dos. Tres filas nuevas. La variante
  `V6` NO entra aquí: el conjunto que un verificador acepta se mueve en su propio sello.
- §475 — el módulo `cuentas` del verificador (`is_right_de_indice`, `cruza_indice`,
  `raiz_de_hoja`, `no_existe`): el espejo de `congelados` sobre el árbol de cuentas, con la hoja
  VACIA como regla, que es lo que sostiene `AccountNotFound` sin la capa (RFC-0007, E5, corte 3b).
  Cuatro filas nuevas; `ACCOUNTS_DEPTH` ya entró con el §474.
- §458 — `FROZEN_DEPTH` y el módulo `congelados` del verificador (`is_right_de_indice`,
  `cruza_indice`, `raiz_de_hoja`, `esta_congelada`): la profundidad del árbol de congelados, fijada
  por el núcleo, y la regla de su hoja (RFC-0007, E3b). Cinco filas nuevas.
- §452 — `VERSION_FORMATO` pasa de 4 a 5: el nodo firma y sirve la cabeza v5 (RFC-0007, E1b).
  Ninguna fila nueva: la constante ya la tenía y su valor no es censo.
- §451 — `epoch_digest_v5`, `params_digest` y `DOMINIO_PARAMS`, la variante `V5`, `lleva_consumos`
  y `texto_con_consumos`, `verificar_acuse_v5` y `verificar_inclusion_v5`: el núcleo y el mando
  aceptan la cabeza v5 (RFC-0007, E1a); los KAT de `epoch_digest_v5` y `params_digest`. Siete
  filas nuevas.
- §419 — las reglas del árbol de consumos en el verificador (RFC-0006, E3b-2): la hoja
  vacía, la convención del camino y el cruce que ata la prueba a la posición del consumo.
  Cinco filas nuevas.
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
