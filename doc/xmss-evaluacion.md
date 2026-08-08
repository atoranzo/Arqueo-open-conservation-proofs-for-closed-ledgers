# Evaluación de candidatas XMSS — entrada 53 (rellena)

**Estado**: evaluada. Pruebas ejecutadas el 01-08-2026 en la máquina del
proyecto, `--release` (`~/eval-xmss`: `tests/evaluacion.rs`, `tests/medidas.rs`;
suite del propio crate desde clone de `master`).

**Convención intacta**: ✅ comprobado con la prueba · ❌ falla la prueba ·
⚠️ matiz o pendiente · **«doc»** = solo lo dice la documentación, no cuenta.

---

## 0. Contexto

Sin cambios respecto al original: se firman `EpochHead`, una por época.
A 1 época/s son ~31,5 M de firmas/año — y ese número, más lo medido en §5,
decide altura **y** conjunto de parámetros.

**El campo cambió el 25-06-2026**: el crate `xmss` de crates.io es hoy de
**RustCrypto/signatures** (0.1.0-pre.0) — la misma organización que `sha2` y
`chacha20poly1305`, ya aceptadas por política en el `Cargo.toml` del proyecto.
Pre-release y **sin auditoría independiente**, declarado por el propio crate.

---

## 1. Resultado por criterio — `xmss` 0.1.0-pre.0

| # | prueba ejecutada | resultado |
|---|---|---|
| **1** | `cargo tree` + grep de curvas/retículos | ✅ **0 coincidencias**; el árbol completo son hashes (`sha2`, `sha3`/`keccak`) y soporte, ~24 crates. ⚠️ matiz registrado: `rand` arrastra `chacha20` (cifrado de flujo, para el RNG de keygen) — no es familia de supuestos nueva y ya está en el proyecto. |
| **2** | firmar → serializar → deserializar → índice | El estado **persiste** (round-trip identidad) pero **la API no expone el índice**: solo `sign`/`sign_detached` + bytes. Según la letra («se descarta»): ❌. Verificado el matiz: SK = 136 B = **OID(4, =0x00000001) + índice(4, BE) + 4×32** — formato de referencia exacto; tras una firma solo cambia el byte 7.
⚠️ **Esas cifras son del conjunto de ÁRBOL ÚNICO** (§236, sonda S.3). Para
el conjunto ELEGIDO, `XMSSMT-SHA2_40/8_256`, se midió: **SK = 137 B =
OID(4) + índice(5, BE) + 4×32**, con el índice en los bytes [4, 9) —el
ancho es ⌈h/8⌉ = 5 para h=40, como esta misma fila advertía—. Al firmar
una vez cambia el **byte 8**, el menos significativo. Derivable, frágil (en MT el índice mide ⌈h/8⌉: el offset depende del conjunto). **Se degrada a ⚠️ condicionado**: issue upstream pidiendo `index()`, y guardián con test propio de layout mientras tanto. |
| **3** | firmar en n → restaurar estado previo → firmar | ❌ **firma**: dos firmas válidas al mismo índice. Segunda vía sin disco: `SigningKey` implementa `Clone` — clonar y firmar con ambas reproduce el reúso. **Predicción registrada y cumplida** — ver E1: ninguna candidata puede pasar esta prueba tal como estaba escrita. |
| **3b** | agotar 2^h firmas (h=10) | ✅ `Err(KeyExhausted)` en la firma 1025 **exacta**, variante nombrada, sin wrap. |
| **4** | dos conjuntos, uno simple y uno multiárbol | ✅ h=10 y MT 20/2 instancian, firman y verifican. Alturas = **menú discreto** (E4). |
| **5** | vectores en su suite; historial | ⚠️ Suite de `master`: **10/10 en 4,1 s**, pero lo visible es funcional (round-trips), sin fichero de vectores en el paquete publicado ni directorio `tests/`. A favor, **conformidad de formato probada byte a byte**: firmas de 18.469 y 27.688 B (las cifras del RFC) y SK en formato de referencia con OID correcto. Pendiente el grep de los 10 nombres y fijar el commit del tag (master ya difiere: `sha3`→`shake`). RustCrypto excluye KATs de los paquetes publicados por precedente (ml-dsa), así que su ausencia en cargo no informa. |

### Enmiendas al protocolo (lo que las pruebas enseñaron sobre las pruebas)

- **E1 · El criterio 3 era insatisfacible junto al 2.** Un estado serializado a
  bytes del llamante (lo que exige el test 2) y restaurado es indistinguible de
  uno legítimo: devolver error exigiría información **externa** al blob — que es
  exactamente el contador del §3. SP 800-208 lo concede al exigir módulos
  hardware para HBS con estado. Reformulación: **3a** firma por `&mut` que
  avanza antes de devolver, sin API firmar-con-índice (aquí ✅) y sin `Clone`
  silencioso (aquí ✘, registrado); **3b** fallo duro al agotar (aquí ✅);
  la protección contra retroceso es **siempre** del proyecto.
- **E2 · El grep del criterio 1 presume que las curvas llegan como
  dependencias.** Un monolito lo derrota (ver `purecrypto`): para monolitos la
  prueba es features + qué se compila, no el árbol.
- **E3 · «Vectores de RFC 8391» no existen con ese nombre**: el RFC no publica
  vectores. Los canónicos son los de `xmss-reference` (repo del RFC) y los
  ACVP de NIST para SP 800-208. El criterio queda corregido.
- **E4 · Las alturas son un menú, no un continuo**: 10/16/20 (XMSS) y
  20/40/60 (MT). Las filas 25 y 30 de la tabla original no existen como
  conjuntos estandarizados.
- **E5 · «Solo hash» estricto tiene una excepción registrada**: `chacha20`
  vía `rand`, para generación de claves. Sin supuesto nuevo.

---

## 2. Tabla de evaluación

| candidata | 1 · solo hash | 2 · índice | 3 · reúso | 3b · agotar | 4 · altura | 5 · revisada | veredicto |
|---|---|---|---|---|---|---|---|
| `xmss` (RustCrypto, 0.1.0-pre.0) | ✅ | ⚠️ cond. | ❌ (E1) | ✅ | ✅ | ⚠️ | **viable, la única usable de CUATRO** (§235) — con guardián obligatorio declarado + issue upstream |
| `purecrypto` (KarpelesLab) | ⚠️ proxy no aplica (monolito, E2) | — | — | — | — | ❌ «doc»: *«do not use it for anything real yet»*, del propio autor | descartada por 5 sin gastar los tests |
| XMSS de QRL | — | — | — | — | — | «doc» | descartada del marco: C++ (qrllib) vía FFI — fuera de la superficie pure-Rust y del test 1 |
| `oxicrypt-xmss` (oxiforge, **0.22.0**) | ✅ 0 curvas, 11 crates | ✅ **`leaf_index()`** | — | ✅ `is_exhausted()` | ❌ **altura 10** | ✅ `tests/nist_kat.rs` | **descartada por ALTURA** (§235): implementa **un solo conjunto**, XMSS-SHA2_10_256 → **1.024 firmas en total**. Al latido de 1/min se agota en **17 h**. Corta por un factor **30.762×** frente a las ~31,5 M firmas/año. **No es peor crate: es para otro uso** —su descripción dice «CNSA 2.0 firmware signing», donde 1.024 sobran— |

⚠️ **La tabla tenía tres filas y había una cuarta** (§235). El veredicto
«única» era correcto en la conclusión y **falso en la premisa**: se afirmó
sobre un conjunto de candidatas incompleto. `oxicrypt-xmss` es una
candidata real —no un monolito, no C++ vía FFI, sin avisos de su autor— y
**gana a la elegida en tres criterios**; la descarta la altura, no la
calidad.

⚠️ **Y refuerza el pendiente 3**: `leaf_index()` existe y está publicado.
El issue de RustCrypto podía leerse como una petición de comodidad; con
una implementación en producción que lo expone, pasa a ser **una omisión
de `xmss`**. Lo mismo con su contador de estado interno: el guardián
resuelve un problema que otros también resolvieron.

---

## 3. El guardián ya no es contingencia: es el mecanismo

El §3 original lo enunciaba como posible salida; E1 y las pruebas lo vuelven
**la única**: contador propio, `fsync`, **firmar-después-de-persistir**, con las
dos cautelas originales (`fsync` puede mentir; el orden invierte el flujo
natural) intactas. Se añaden dos requisitos medidos:

- **Test de layout**: el guardián lee el índice del SK en el offset del formato
  de referencia; debe llevar un test que firme con clave conocida y compruebe
  que el byte esperado se mueve — para que un cambio de serialización entre
  versiones **falle en CI, no en producción**.
- **Reconciliación tras reinicio**: si contador propio > índice del SK (caída
  entre `fsync` y firma), el índice huérfano se quema firmando un descarte o se
  registra la divergencia; nunca se retrocede el contador.

Y como exige el §6 original, **se declara**: la seguridad del esquema pasa a
depender de este código propio, no auditado.

---

## 4. Índice y `seq`

Sin cambios: **contador independiente** (§111.1). Reforzado por E1 — ni la
propia librería distingue un reinicio honesto de un reúso malicioso.

---

## 5. Altura: de tabla teórica a números medidos

**Hallazgo**: en esta implementación **firma ≈ keygen** (645 vs 634 ms en h=10,
constante en 1024 firmas; MT 20/2 firma en 1,27 s ≈ dos capas). Con 136 B de SK
no hay estado de recorrido: **reconstruye el árbol en cada firma**,
O(d·2^(h/d)), no O(h). Es propiedad de la *implementación* (BDS la eliminaría),
no del esquema — tercera víctima del §5, que la tabla original no tenía:
**latencia**.

Modelo validado (~0,62 ms/hoja en esta máquina; error < 1 % en dos
predicciones):

| conjunto | firma (medida/derivada) | verif | tamaño firma | horizonte a 1/s | veredicto a épocas de 1 s |
|---|---|---|---|---|---|
| XMSS 20 | ≈ 650 s *(der.)* | — | 2.820 B | 12 días | ✘✘ |
| MT 40/2 | ≈ 22 min *(der.)* | — | 5.605 B | ~35.000 años | ✘ |
| MT 40/4 | ≈ 2,5 s *(der.)* | — | 9.893 B | ~35.000 años | ✘ |
| **MT 40/8** | **160,5 ms** *(medida)* | 2,7 ms | **18.469 B** | ~35.000 años | ✔ |
| MT 60/6 | ≈ 3,8 s *(der.)* | — | 14.824 B | 2⁶⁰ | ✘ |
| MT 60/12 | **241,2 ms** *(medida)* | 3,8 ms | 27.688 B | 2⁶⁰ | ✔ |

Consecuencias de primer orden a 31,5 M firmas/año:

- **Almacenamiento**: 40/8 ≈ **0,58 TB/año**; 60/12 ≈ 0,87 TB/año — solo en
  firmas de cabezas. Domina el almacenamiento del sistema entero (compárese con
  120 MB/1000 transferencias). Obliga a **política de retención** que no
  existe: la disputa del testigo necesita *dos* cabezas contradictorias, no el
  historial — pero decidirlo es política (`PRINCIPIOS.md`).
- **CPU recurrente**: 16 % de un núcleo permanente (24 % con 60/12).
  Desaparecería con BDS upstream — punto del issue.
- **Keygen** dejó de ser coste (18,9 ms en 40/8: solo la capa superior, 2⁵).

**Propuesta**: `XMSSMT-SHA2_40/8_256`. Domina en firma, tamaño y CPU; 60/12
solo compra horizonte, y el de 40/8 (~35.000 años) solo muerde si la cadencia
bajara de ~10 ms/época. Sigue siendo decisión de `PRINCIPIOS.md`; esta tabla la
deja sin incógnitas.

⚠️ Los 0,62 ms/hoja son de **esta máquina**: si el nodo objetivo es ARM (B9),
repetir `tests/medidas.rs` allí antes de fijar nada.

---

## 6. Registro y pendientes

Asientos redactados aparte (`AUDITORIA.md §112-114`), incluido el que el §6
original exigía: *ninguna cumple el criterio 3, y ninguna puede*.

Pendientes, por orden:

1. ✅ **HECHO** (§235, sonda S.1). **17 tests**, y **sí hay KAT en el
   paquete publicado**: `test_kat_xmss_sha2_10_256_verify`. La nota de §1
   decía «sin fichero de vectores», y es cierto —no hay fichero— pero el
   KAT está dentro del fuente. Mejor de lo registrado.
   ⚠️ **`Clone` sigue derivado en `SigningKey`**: el footgun de E1 está
   intacto, verificado sobre el fuente descargado.
   ⚠️ Y **no hay `index()` ni `remaining()`**: el pendiente 3 sigue vivo.
2. ✅ **HECHO** (§235, sonda S.1): se fijó con `cargo add xmss@=0.1.0-pre.0`
   y se leyó el fuente del registro, no `master`. **El conjunto elegido
   existe**: `XmssMtSha2_40_8_256`, entre las 56 variantes multiárbol.
   Criterio 1 reverificado: **0 curvas, 0 retículos**; `chacha20` vía
   `rand` como única excepción, la ya declarada en E5.
3. **Issue en RustCrypto/signatures** (`issue-rustcrypto.md`): `index()`,
   `remaining()`, plan BDS, y el aviso sobre `Clone`.
4. Guardián: implementación + test de layout (§3).
5. Medidas en ARM si aplica (junto a B9).
