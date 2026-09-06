# El paquete de evidencia portable — especificación del sobre y del mando

- **Estado:** normativa vigente desde §397 (RFC-0004, etapa E1)
- **Versión del protocolo:** `zkssl/0.3` — este documento no la mueve: el paquete no cruza el cable
- **Origen:** `spec/rfc/0004-paquete-de-evidencia.md`; asiento que lo sella: §397
- **Implementación de referencia:** el binario `zk-ssl-verify` (`crates/zk-ssl-verify/src/main.rs`)

Este documento es el **único productor normativo** del formato del paquete de evidencia y del
contrato del mando que lo verifica. Hasta §397 vivía en la cabecera del propio binario (líneas
1..90, sha de región `293990fedc785833`); `spec/RPC.md` delegaba ahí por escrito, y esa cabecera
ya había caducado una vez sin que nada lo cazara. Aquí se muda **lo que estaba escrito**, se
añade lo que sólo estaba en el código —el orden de comprobación completo, el catálogo de
rechazos y el contrato del mando— y la cabecera del binario pasa a remitir a este fichero.

**Especifica el SOBRE, no los campos.** Los valores del paquete son respuestas del cable **tal
cual** el nodo las sirve —quien reescribe, adultera—, así que la semántica de cada campo vive en
`spec/RPC.md` y aquí sólo se remite por línea. Duplicarla crearía dos productores del mismo
contrato, que es el defecto que este documento repara.

## 1. Qué es

Lo que sostiene la posición del titular ante un tercero **cuando el operador desaparece o
miente**: las respuestas del cable que ya custodia, reunidas en **un** fichero JSON y verificadas
**sin el nodo, sin la capa y sin el probador** (§243) — sólo el binario y lo publicado. Es el
procedimiento de apagado que `spec/RPC.md` declara en su sección «Apagado — el fin de vida»
(`RPC.md:810-855`): al cerrar, el operador no publica nada que no esté ya publicado; el titular se
lleva lo que ya custodia; y con este paquete lo sostiene después.

El paquete **REPORTA, no juzga.** Dice si la cabeza es de quien dice, si el acuse sube hasta la
raíz firmada y cuántas cofirmas acreditan esa cabeza y ese operador. **Qué testigos valen y
cuántos hacen falta lo decide el CLIENTE** con su política (§319, los mandos `--testigos` y `--k`
del testigo), no el paquete: quien lo arma puede ser el operador, y dejarle elegir su propia `k`
le devolvería justo lo que la cofirma le quita.

## 2. Las tres formas

El binario acepta tres objetos. Los tres son JSON; los esqueletos van con puntos suspensivos
donde el valor es una respuesta del cable sin reescribir.

### 2.1 El paquete v1 — la posición

```text
{
  "v": 1,
  "cabeza": { …payload de zkssl_signedEpochHead con available:true… },
  "acuse": {                                  // OPCIONAL
    "seq": "0x…",                             // la entrada del titular
    "hashPrueba": "0x…64hex",                 // digest de SU prueba
    "s": "0x…",                               // de zkssl_ackPath
    "camino": { "siblings": […], "isRight": […] }
  }
}
```

- `cabeza` es el `result` de `zkssl_signedEpochHead` (`RPC.md:433-478`), con `available:true`.
- `acuse` es lo que `zkssl_ackPath` devuelve (`RPC.md:564-735`) más el `hashPrueba` de la entrada
  del titular (el `proofDigest` asentado). Si no viaja, la cabeza sola queda demostrada.

### 2.2 El paquete v2 — las cofirmas dentro (§322)

```text
{
  "v": 2,
  "cabeza":   { …igual que en v1… },
  "acuse":    { …igual que en v1, OPCIONAL… },
  "cofirmas": [ …la respuesta de zkssl_cosigs TAL CUAL, OPCIONAL… ]
}
```

- `cofirmas` es el contenido de `zkssl_cosigs` sin reescribir (`RPC.md:737-779`): cada elemento
  acredita que **un** testigo vio **esta** cabeza de **este** operador, y nada más.
- **El binario lee v1 y v2**: lo custodiado no caduca (§290). Lo que la subida de versión compra
  es que un binario viejo se niegue en voz alta ante un v2 en vez de ignorar las cofirmas e
  imprimir VERDE: *un campo que nadie mira es peor que un campo que falta*. Por eso **un v1 que
  traiga `cofirmas` se RECHAZA** — subir la versión es exactamente lo que las hace parte del
  contrato.
- Dos convenciones de versión conviven, y conviene saberlo: la del PAQUETE (`v`) es un número
  JSON desnudo; la de cada COFIRMA (`v`) viaja como `Q`, cadena hex con `0x`, porque llega del
  cable tal cual (`RPC.md:39-48`). No se unifican: reescribir es adulterar.

### 2.3 El paquete de extensión (§293)

```text
{ "v": 1, "tipo": "extension", "vieja": {…}, "nueva": {…}, "camino": […] }
```

- `vieja` y `nueva` son dos cabezas **v3** firmadas, cada una el `result` de
  `zkssl_signedEpochHead`; `camino` es la prueba de consistencia entre sus cimas
  (`RPC.md:781-808`). Quien custodia la vieja comprueba que la nueva la **extiende**, con el MMR
  de cabezas (§291) como juez, sin el registro y sin el nodo.
- La forma se elige por `tipo`: si vale `"extension"`, el sobre es este; si no, es el de posición.
  `v` se comprueba antes en los dos casos.

## 3. El sobre — lo que el binario lee

El binario lee **31 nombres** distintos del JSON. Los 14 primeros son el sobre propiamente dicho;
los demás son campos de las respuestas del cable que el binario necesita para recomponer y
verificar, y cuyo significado está en `spec/RPC.md`.

| objeto | claves que el binario lee | dónde está su semántica |
|---|---|---|
| sobre | `v`, `tipo`, `cabeza`, `acuse`, `cofirmas`, `vieja`, `nueva`, `camino` | este documento, sección 2 |
| `cabeza` (y `vieja`/`nueva`) | `available`, `formatVersion`, `seq`, `n`, `accountsRoot`, `pendingRoot`, `frozenRoot`, `chainDigest`, `acusesRoot`, `epochDigest`, `publicKey`, `signature`, `index`; y en v3 `mmrRoot`, `mmrSize` | `zkssl_signedEpochHead`, `RPC.md:433-478` |
| `acuse` | `hashPrueba`, `seq`, `camino` → `siblings`, `isRight` | `zkssl_ackPath`, `RPC.md:564-735` |
| cada cofirma | `v`, `epochDigest`, `clavePublicaOperador`, `clavePublicaTestigo`, `firma`, `versionFormato`, `indice` | `zkssl_cosigs`, `RPC.md:737-779` |
| extensión | `camino` (lista de digests) | `RPC.md:781-808` |

Cantidades en convención `Q` (`0x` + hex, u64); digests como `0x` + 64 hex; firmas y claves como
`0x` + hex. Un valor que no tenga esa forma se rechaza **antes** de tocar la criptografía (sección 5).

## 4. Lo que se comprueba, en orden

El orden importa: lo barato y lo estructural va antes, y **el digest nunca se cree, se
recompone**. Cada paso que pasa imprime una línea en la salida estándar.

**Paquete de posición (v1 y v2):**

0. el fichero se lee y es JSON; `v` es 1 o 2; un v1 no trae `cofirmas`; `tipo` no es `extension`;
1. **`1/3`** — `cabeza` existe y es `available:true`; `formatVersion` es 2, 3 o 4; **la versión
   elige recomponedor**: v2 con la pareja de acuses (§275), v3 además con la del MMR (§292), v4
   además con la raíz y la cuenta de consumos (RFC-0006 E2a, §414); los
   campos de la cabeza recomponen su `epochDigest`;
2. **`2/3`** — la firma XMSS verifica contra `publicKey` **y** el preámbulo recuperado es el
   esperado (verificar sin comparar no prueba nada) **y** el `index` declarado queda por
   encima del índice de hoja que va dentro de la firma (§399; la cota es por abajo, sección 8);
3. **`3/3`** — si hay `acuse`: la hoja `hoja_de_acuse(hashPrueba, seq, n)` sube por `camino`
   hasta `acusesRoot`, y los campos vuelven a componer el digest firmado (v2), el digest y la
   cima (v3), o además la raíz y la cuenta de consumos (v4). Si no hay acuse, la cabeza sola
   queda demostrada;
4. **cofirmas** (sólo v2) — cada una, antes de tocar la criptografía, nombra **esta** cabeza y
   **este** operador; después su firma verifica. Se imprime cuántas verifican; cuántas hacen
   falta no es asunto del paquete.

**Paquete de extensión:** `1/3` las dos cabezas (v3 o v4) recomponen su digest y sus firmas verifican ·
`2/3` misma `publicKey` en las dos: la continuidad es de **un** firmante · `3/3` la cima nueva
extiende a la vieja por `camino`.

Cabezas **v2, v3 y v4** (`formatVersion`): una cabeza v2 custodiada **sigue verificando** — el
apagado de §290 no caduca. Una cabeza v1 se verifica con la biblioteca, no con este mando.

## 5. Catálogo de rechazos

Cualquier rechazo para el binario con **el primer fallo, con nombre**, en la salida de error, y
sale con código 1 (sección 6). No hay pánico por paquete mal formado: cada lectura de campo falla
cerrada. El catálogo es la lista de mensajes que el binario puede emitir, con sus huecos entre
llaves; un lector que vea uno de estos textos sabe qué regla ha caído. **La verdad se mide en el
fuente**: al sellar, el censo de llamadas se re-deriva por llamada (no por línea: seis textos van
partidos en el fuente) y cada texto tiene que estar aquí.

**Lectura del fichero y versión del sobre**

- `no se puede leer {ruta}: {e}`
- `JSON ilegible: {e}`
- `el paquete no declara su version en `v``
- `el paquete declara v:{v_paquete} — este binario lee v1 y v2`
- `un paquete v1 con `cofirmas`: subir la version es lo que las hace parte del contrato — declaralo v2, o quitalas`

**Forma de los valores** (`hex_a_bytes`, `digest_de`, `u64_de`; `{campo}` es la clave que se leía)

- `sin 0x: {s:.18}` · `hex impar ({} chars)` · `hex: {e}`
- `falta {campo} o no es cadena` · `{campo}: {} bytes, se esperaban 32` · `{campo}: {e:?}`
- `falta {campo} o no es cadena 0x` · `{campo} sin 0x` · `{campo}: {e}`

**La cabeza** (paso 1 y 2)

- `falta cabeza`
- `la cabeza empaquetada no era available:true`
- `formatVersion {version}: el paquete v1 empaqueta cabezas v2, v3 o v4 (la pareja acusesRoot/n viaja firmada desde §275; la del MMR, desde §292)` — el texto dice «v1» aunque el sobre sea v2: regla vigente, prosa que la implementación de referencia debe corregir sin cambiar la regla.
- `los siete campos NO recomponen el epochDigest empaquetado: o el paquete esta adulterado o la cabeza nunca fue esa`
- `falta publicKey` · `falta signature`
- `cabeza: {e}` — la firma no verifica, el preámbulo no es el esperado, o el `index` declarado no queda por encima del que va dentro de la firma (§399); `{e}` es el error de la biblioteca.

**El acuse** (paso 3)

- `acuse sin camino` · `camino sin siblings` · `camino sin isRight`
- `sibling {i} no es cadena` · `sibling {i}: {} bytes` · `sibling {i}: {e:?}`
- `isRight no booleano`
- `acuse: {e:?}` — la hoja no sube hasta la raíz firmada, en v2 o en v3 (dos sitios, un texto).

**Las cofirmas** (paso 4, sólo v2)

- `cofirmas no es una lista`
- `cofirma {n}: falta {campo}`
- `cofirma {n}: version {cv} desconocida, este binario lee hasta la {}` — el tope es `COFIRMA_V_MAX` de la biblioteca.
- `cofirma {n}: acredita OTRA cabeza, no la empaquetada`
- `cofirma {n}: acredita a OTRO operador, no al que firmo la cabeza`
- `cofirma {n}: {e}` — la firma del testigo no verifica.
- Una lista vacía o ausente no es un error: el paquete v2 imprime que la cabeza queda sola.

**La extensión** (`{cual}` es `vieja` o `nueva`)

- `falta vieja` · `falta nueva`
- `{cual}: la cabeza no era available:true`
- `{cual}: formatVersion {version} — la extension exige cabezas v3 o v4: una v2 no lleva la pareja del MMR que extender`
- `{cual}: los campos NO recomponen su epochDigest — adulterada o inventada`
- `{cual}: falta publicKey` · `{cual}: falta signature`
- `{cual}: cabeza: {e}`
- `las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante`
- `falta camino (lista de digests)`
- `camino[{i}] no es cadena` · `camino[{i}]: {} bytes` · `camino[{i}]: {e:?}`
- `la nueva (t={t_n}) NO extiende a la vieja (t={t_v}): historia bifurcada, recortada, o camino que no es el suyo`

## 6. El contrato del mando

- **Invocación:** `zk-ssl-verify <paquete.json>` — **un** argumento, la ruta del fichero. Es la
  única lectura de disco del binario; no hay red, ni reloj, ni telemetría (§395 lo gatea).
- **Salida estándar:** las líneas `1/3` · `2/3` · `3/3` (o `3/3 sin acuse en el paquete: la cabeza
  sola queda demostrada`), la de cofirmas cuando el sobre es v2, y al final
  `VERDE: el paquete se sostiene sin el nodo` o `VERDE: la extension se sostiene sin el nodo`.
- **Salida de error:** `ROJO: {motivo}` con un texto del catálogo de la sección 5, y para.
- **Tres códigos de salida:** `0` verde · `1` el primer fallo con nombre · `2` uso (ningún
  argumento, o más de uno; imprime el uso en la salida de error).

## 7. Quién arma el paquete

Hoy **ningún mando del árbol emite el paquete**: el testigo y el cli sirven y custodian las
respuestas del cable, y quien las reúne en el sobre es el titular — en el árbol, los bancos
`tools/banco_apagado.sh`, `tools/banco_completo.sh`, `tools/banco_evidencia_v2.sh` y
`tools/banco_extension.sh`, que capturan las respuestas de un nodo real y las envuelven sin
reescribir un campo. Ese es el contrato: **reunir, no recomponer**. Un mando que arme el paquete
es un frente propio y no cambia este documento: cambiaría quién escribe el sobre, no el sobre.

## 8. Lo que este documento NO afirma

- Que exista un verificador independiente **no hace las firmas oponibles**: sigue faltando la
  custodia declarada de la clave del operador (`SECURITY.md`). Esto hace posible verificar; no
  hace válido lo verificado.
- Que las cofirmas verifiquen no dice que basten: la política es del cliente (sección 1).
- Nada sobre el contenido de la posición: el paquete demuestra **que** la entrada está acusada
  bajo esa cabeza firmada, no **qué** dice.
- El `index` de la cabeza sólo está acotado **por abajo**. La firma acredita el índice de hoja
  que lleva dentro y el binario exige que el declarado sea mayor (§399); un sobre que declare
  más de lo que firmó no se rechaza, y lo que el mando imprime como «indice de firma» es el
  embebido, no el declarado.

## 9. Vectores y puerta

Los vectores del paquete viven en `spec/vectors/paquete/` (etapa E2 del RFC-0004, sellada en §398; E3 en §399):
un positivo por forma —v1, v2, v2 sin acuse, v2 con cero cofirmas, extensión— y **un negativo por
cada regla de la sección 5 que se puede producir a partir de un paquete real**, derivados por
mutación de dos capturas de los bancos. Los dos negativos del índice (`rechazo-index-atrasado`,
`rechazo-index-cero`) entraron con su regla en §399.
`MANIFIESTO.txt` dice, por cada fichero, el código de
salida y el texto que el binario tiene que emitir. **El arnés `tools/conformidad.sh <binario>`**
(RFC-0005, E4, §408) corre el manifiesto entero contra cualquier binario que cumpla el contrato
de la sección 6 —el de la referencia o el de una segunda implementación— y dice, entrada a
entrada, si el código de salida y el texto son los del manifiesto; es el único productor de ese
bucle. `tools/canon.sh` lo corre en cada canon sobre el binario de referencia y se pone en rojo
si un solo vector no dice lo que el manifiesto dice, o si aparece un vector sin entrada. Un
nibble adulterado en cualquiera pone el canon en rojo.

Cuatro textos del catálogo **no tienen vector**, y se declaran: «las cabezas llevan claves
DISTINTAS» exige dos cabezas firmadas por dos operadores distintos; y `{campo}: {e:?}`,
`sibling {i}: {e:?}` y `camino[{i}]: {e:?}` exigen 32 bytes que `digest_from_bytes` rechace, y no
se conoce un valor que lo haga. Siguen siendo reglas: lo que no tienen es testigo en el árbol.
La demostración en vivo con nodo sigue siendo `tools/banco_apagado.sh`.

## 10. Historia

- §289: nace el paquete (formato v1) y su binario; §290: el apagado declarado; §293: el paquete de
  extensión; §322: el v2 con las cofirmas dentro.
- §399 — el `index` declarado se ata al que va dentro de la firma (E3); el mando imprime el embebido; el testigo lee el `index` servido.
- §400 — el RFC-0004 pasa a ACEPTADO: la regla 4 del PROCESO, saldada con medida.
- §401 — el artefacto: `tools/artefacto.sh`, el tarball reproducible y el 3 ter del canon (sección 11).
- §408 — el arnés de conformidad (RFC-0005, E4): `tools/conformidad.sh`, el único productor del bucle
  del manifiesto, consumido por el canon y por `artefacto.sh`, y dentro del tarball (sección 9).
- Hasta §397 este contrato vivía en la cabecera de `crates/zk-ssl-verify/src/main.rs` (1..90,
  `293990fedc785833`), que ya confesó una vez (§247) haber declarado su superficie como completa
  sin serlo. §397 lo muda aquí y deja la cabecera remitiendo, sin enumerar.
- Cambiar este documento es cambiar el contrato: entra por RFC (`spec/rfc/PROCESO.md`).

## 11. El artefacto

Lo que un tercero descarga es `arqueo-verify-<versión>-<host>.tar.gz` (§401), y dentro:
`zk-ssl-verify` (el binario), `conformidad.sh` (el arnés de la sección 9, §408), `spec/PAQUETE.md`
(este documento), `spec/vectors/paquete/` (el manifiesto y sus vectores), `LICENSE-APACHE`,
`LICENSE-MIT`, `NOTICE`, `THIRD-PARTY.txt` (las
licencias de todo lo enlazado), `VERSION` (el commit, el toolchain y los flags con que se compiló)
y `SHA256SUMS` (la huella de cada fichero de dentro). Se comprueba con `sha256sum -c SHA256SUMS`,
y el binario contra su propio manifiesto con `bash conformidad.sh ./zk-ssl-verify`: cada entrada
dice el código de salida y el texto.

La huella del binario **no depende de la máquina ni del usuario** —se compila con
`--remap-path-prefix`—, pero sí del toolchain y de `Cargo.lock`: con el `rustc` que `VERSION`
nombra, `bash tools/artefacto.sh` sobre el commit que `VERSION` nombra vuelve a producir el mismo
binario y el mismo tarball, y `tools/canon.sh` comprueba esa propiedad en cada sello (dos
compilaciones en dos rutas, misma huella; dos tarballs, misma huella; el manifiesto entero). Lo
que el binario exige: x86_64 Linux y una glibc igual o mayor que la que `VERSION` declara
(`glibc_max`); no es estático, y se dice.

Lo que el artefacto NO es: no es una publicación en crates.io (el crate no lleva la spec ni los
vectores) ni prueba nada sobre la clave del operador (sección 8). La release es un fichero con
huella, y la huella vive en el asiento que lo selló.
