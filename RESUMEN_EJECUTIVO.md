# ZK-SSL — Resumen ejecutivo

## Qué es

Una capa de liquidación con privacidad criptográfica y cumplimiento
demostrable, **sin ninguna ceremonia de confianza**, más el trabajo
comparativo que fundamentó su diseño: **el mismo sistema implementado en
cinco paradigmas de prueba distintos y medido en condiciones idénticas**.

Todo verificado con tests ejecutables. Ninguna cifra de este documento
procede de la literatura: todas se midieron en la misma máquina, en modo
release.

---

## ⚠️ Antes que nada: el operador del nodo es un intermediario de confianza

El proyecto parte del principio de eliminar intermediarios de confianza
centralizados. Se eliminó uno —los participantes de una ceremonia de
setup, que podrían coludir y crear dinero sin dejar rastro— y esa
propiedad es real.

**Pero esta capa es un nodo único.** Quien lo opera ve todos los saldos,
ordena las operaciones, puede censurar y es un punto único de fallo. Es
el intermediario que el principio señala, y sigue ahí.

**Las transiciones de estado están demostradas matemáticamente. El estado
y la completitud del historial, no.** Cerrar esa brecha requiere consenso
distribuido, que es trabajo pendiente y de otra disciplina.

**Lo que esto es**: una demostración de que las propiedades
criptográficas de una liquidación soberana son construibles y medibles.
**Lo que no es**: una capa descentralizada.

---

## 1. El artefacto: `crates/zk-ssl`

```rust
let layer = SovereignLayer::open("./ledger", custodios, gobernanza, limite, tope, max_cuentas)?;

// FASE 1 — el pagador envía. La capa no ve su clave.
let m = layer.send_materials(alice, id_de_bob, 250_000, aleatorio)?;
let envio = client::prove_send(&m, clave_alice, proof_options())?;  // LOCAL
layer.apply_send(&envio, alice, &estado_alice, 250_000)?;

// FASE 2 — el receptor cobra. Tampoco ve la suya.
let m = layer.claim_materials(bob, &envio.notice)?;
let cobro = client::prove_claim(&m, clave_bob, proof_options())?;   // LOCAL
layer.apply_claim(&cobro, bob, &estado_bob, &envio.notice)?;
```

**Ninguna clave llega a la capa**: entrega caminos y raíces —datos
públicos— y recibe pruebas que verifica. La revelación selectiva
(`audit`: "estoy entre X e Y") la produce el titular y la verifica el
supervisor **sin acceso al ledger**.

**1007 tests en la compuerta de sello** —1144 con todos los pines, 1158
declarados—, todos en release, 0 fallos y 24 warnings **pinchados**.
No se recuerdan: los ejecuta `bash tools/canon.sh --sello`.

### Qué garantiza, sin revelar identidades, saldos ni importes

| Vía de ataque | Cerrada por |
|---|---|
| Transferir más de lo debitado | Conservación (partida doble) |
| Abrir cuenta con saldo | Apertura siempre a cero |
| Emitir sin autorización | **Dos custodios** demostrados en circuito |
| Emisión encubierta | Suministro público atado en el circuito |
| Superar el tope de emisión | Tope inmutable del ledger |
| Gastar dos veces | Encadenamiento de raíces (orden total del nodo único) |
| Gastar sin ser el titular | Autoridad de gasto |
| **Gastar estando congelada** | No-pertenencia al árbol de congelados |
| Reenviar una operación válida | Encadenamiento de raíces |
| Operar sobre estado corrupto | Verificación de integridad al arrancar |
| **Reescribir el historial** | Registro encadenado de transiciones |

### Ciclo monetario completo

| Operación | Autoridad | Suministro |
|---|---|---|
| `mint` | Emisor, dentro del tope | Sube |
| `transfer` (dos fases: `send` → `claim`) | Titular | No cambia |
| `burn` | Titular | Baja |
| `audit` | Titular | — |

### Cifras medidas

| Operación | Generar | Verificar | Prueba |
|---|---|---|---|
| **Arranque** | **0,67 ms** | — | — |
| Emisión (2-de-N custodios) | ~105 ms | ~2 ms | 57.342 B |
| Transferencia | ~620 ms | ~4 ms | 61.966 B |
| Destrucción | ~110 ms | ~2 ms | 54.924 B |
| Auditoría (banda) | ~250 ms | ~1,5 ms | 48.782 B |

**Verificar cuesta el 0,5-0,8% de generar.** El arranque no genera
claves: no hay ceremonia ni secreto que destruir.

**Límites cuantificados**: mil transferencias son **~590 s** de prueba
(un pago son dos: send 353,2 ms + claim 237 ms, protocolo §89.1) y
**126,2 MiB** acumulados.

⚠️ **Aquí decía que el techo era «1,5–1,9 TPS». Era falso** (§229): esa
cifra medía el ciclo entero en una sola máquina y se atribuía al nodo. El
nodo trabajaba el **4 %** del tiempo. Medido aparte, **aplica 248 op/s por
RPC** (§229, banco H.1), y el objetivo de un RTGS —21 op/s— es el **8,5 %**
de ese techo.

Lo que sí serializa es **la raíz**, no el candado (§230): dos emisores que
salen a la vez aplican uno, y el otro tira sus pruebas. El nodo rechaza
barato; el precio lo paga quien pierde.

⚠️ **Un límite que existió, y cómo se fue**: la vía de un paso tenía
colisiones probables a los ~65.000 pagos. Esa vía está **retirada** y su
árbol con ella (`AUDITORIA.md` §32 y §36): hoy nada los genera. El
límite no se resolvió, se evitó — quien la recupere, lo recupera.

⚠️ Una sola ejecución en una máquina. Sirven para comparar órdenes de
magnitud, no como benchmark.

---

## 1.bis — De implementación a protocolo (§197–§199, agosto 2026)

La capa ya no está sola: tiene **contrato público** para que exista una
segunda implementación sin leer el código del nodo. `spec/RPC.md`
(**`zkssl/0.3`** desde §354; la `0.2` rigió desde §209, 24 métodos) · `spec/openrpc.json` **generado** desde el
código (regenerarlo debe reproducirlo byte a byte) · **vectores de
conformidad** versionados (`conformance --check` los re-ejecuta campo a
campo: es compuerta permanente) · proceso RFC · nodo de referencia
(`zk-ssl-node`) · SDK donde la prueba se hace **en local** y el wallet
duerme cifrado (keystore con dominio propio). Dos comandos para tocarlo:

```bash
cargo run --release -p zk-ssl-cli -- simulate --amount 250000
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.3.json
```

---

## 2. El trabajo comparativo: cinco paradigmas

El mismo circuito de cumplimiento, misma máquina, todo en release.

| | Groth16 | Halo2/IPA | STARK/FRI | PLONK/KZG |
|---|---|---|---|---|
| Paradigma | R1CS | Plonkish | AIR | Plonkish |
| Ceremonia | **Por circuito** | Ninguna | **Ninguna** | Universal |
| Setup | 438 ms | 16,3 s | **ninguno** | 26,3 s + 12,8 s |
| Generación | 422 ms | 4,86 s | **39 ms** | 6,85 s |
| Verificación | 5 ms | 91 ms | **1 ms** | 8 ms |
| Tamaño | **192 B** | 4.096 B | 36,7 KB | 1.008 B |
| Post-cuántico | No | No | **Sí** | No |

**Nova/folding**, quinto paradigma, medido aparte por ser de naturaleza
distinta: **~250 ms por transacción, constante** (el paso 9 costó 0,77
veces el paso 1), con 1,84 s de cierre amortizables entre todas.

---

## 3. Los hallazgos

Ninguno aparece en los materiales que comparan paradigmas. Todos surgieron
al construir.

**1. AIR carece de restricciones de copia.** Al portar la actualización de
estado a STARK apareció un agujero que no existe en los otros
paradigmas: nada obliga a que las dos subidas del árbol (hoja antigua y
nueva) usen los mismos hermanos. Un probador podría usar caminos
distintos y fabricar una raíz que no corresponde a la misma posición.
**Silencioso**: los testigos honestos nunca lo revelarían. Obligó a
diseñar un patrón en lockstep. *Portar de Plonkish a AIR no es mecánico
ni cuando la lógica es idéntica.*

**2. El campo Goldilocks es demasiado estrecho para identidades.** Un
elemento son 64 bits: encontrar otra clave con la misma identidad costaría
2³² operaciones. En BLS12-381 (255 bits) el problema no existe. Corregido
usando digests completos de 256 bits.

**3. Sin extensión de campo, un STARK sobre Goldilocks tiene un techo de
63 bits de solidez.** La configuración "rápida y compacta" que uno
elegiría por defecto **no es comparable** con los ~128 bits de los otros
paradigmas.

**4. La brecha entre seguridad conjeturada y demostrable es enorme.** 127
bits conjeturados conviven con 29-63 demostrables. Cerrarla cuesta 125,6
KB en vez de 36,7.

**5. PLONK-KZG resultó el generador más lento de los cuatro** — 16 a 22
veces más lento que Groth16. Contraintuitivo para lo que suele
presentarse como el estándar de la industria. *Matiz honesto: parte de la
diferencia puede deberse a la implementación (`dusk-plonk` frente a
arkworks) y estos datos no permiten separar ambos efectos.*

**6. Solo dos de seis librerías se defienden del uso inseguro.**
`nova-snark` desactiva `HyperKZG::setup` en compilaciones de producción y
exige ficheros de una ceremonia real. `risc0-zkvm` permite recibos falsos
pero ofrece `disable-dev-mode`, descrita como *"para evitar que un
`RISC0_DEV_MODE` mal puesto rompa la seguridad en sistemas de
producción"* — nombrando el escenario concreto por el que ocurren estos
fallos. Las otras cuatro lo permiten en silencio.

**7. El ecosistema PLONK-KZG en Rust está construido como stacks
verticales cerrados.** Seis vías investigadas, cinco rotas: `plonk-core`
sin publicar, el Poseidon de PSE sin especificación para su curva,
`halo2-lib` con dependencias git sin fijar, y `dusk-plonk 0.21`
arrastrando un `msgpacker` que exige Rust nightly.

**8. Un zkVM no es comparable en igualdad de condiciones.** Se evaluó
RISC Zero como sexto paradigma. Usa STARK sobre Goldilocks —el mismo
sistema y campo que el backend elegido— y su recibo `Succinct` permite
operar **sin ceremonia**: el envoltorio Groth16, que sí la exige, solo
hace falta para verificar en cadena.

Pero necesita una **toolchain externa** para compilar el programa
invitado a RISC-V, y eso incumple el criterio que descartó a
`dusk-plonk 0.21`, `halo2-lib` y `plonk-core`: instalarse solo con
`cargo add`. No es un defecto suyo —es el precio de compilar programas
arbitrarios— pero medirlo junto a los otros cinco falsearía la
comparación, así que **se documenta en vez de implementarse**.

| | Backend STARK propio | RISC Zero |
|---|---|---|
| Dependencias | **3** | **349** |
| Seguridad declarada | 127 bits conjeturados | 98 bits conjeturados |

Tres dependencias frente a trescientas cuarenta y nueve es la medida
concreta de lo que cuesta la generalidad, y la razón por la que un zkVM
contradice el principio de minimalismo.

---

## 4. Método

Lo que distingue este trabajo no es el código, sino cómo se verificó.

**Cada propiedad de seguridad tiene un test discriminante**: un testigo
**internamente coherente** que solo viola la restricción concreta. Un
testigo corrupto a lo bruto rompe varias restricciones a la vez y el test
pasa aunque la que interesa no haga nada.

**Tres veces durante el proyecto un test negativo resultó no
discriminar** y hubo que rehacerlo. En los tres casos el código era
correcto, pero el test no probaba lo que decía probar.

**Un error metodológico propio, detectado y corregido públicamente**: la
comparativa mezclaba cifras de debug con cifras de release, haciendo
parecer a STARK 130 veces más rápido que Groth16 cuando la cifra real es
~11. Está documentado como error corregido, no borrado.

**Dos errores de diseño propios, corregidos:** hacer el nullifier privado
(rompía la capacidad de la capa de mantener su árbol) y olvidar
insertarlo en `apply` (habría hecho vacua la garantía más cara del
sistema, 15.522 restricciones). Los dos los destapó un test.

---

## 5. La decisión de fondo

**Se descartó Groth16 pese a ser el más rápido y tener pruebas 320 veces
más pequeñas** (192 bytes frente a 62 KB).

El motivo no fue técnico sino de coherencia: Groth16 y PLONK-KZG exigen
una ceremonia de confianza, y si sus participantes coluden pueden
falsificar pruebas y crear dinero sin que nadie lo detecte jamás. Para
una infraestructura soberana eso es una dependencia externa permanente e
inauditable.

Es la única decisión del proyecto tomada **contra** los números de
rendimiento.

---

## 6. Lo que NO es

- **No hay red ni consenso.** Nodo único.
- **No hay delegación de la prueba.** Quien la genera necesita la clave;
  en un banco, la clave estaría en un HSM y el cómputo en otro servicio.
- **No hay atomicidad entre operaciones.** Si el proceso muere a mitad,
  el arranque detecta la inconsistencia y se detiene — correcto, pero
  requiere intervención manual.
- **No hay copias ni replicación.** El cifrado en reposo **sí** existe
  —ledger y keystore del wallet (`zk-ssl::crypto`, XChaCha20-Poly1305)—
  con su alcance declarado: protege el disco robado, no al operador.
- **No hay umbral configurable.** Emitir, emitir a pendiente, congelar y
  recuperar exigen dos custodios distintos de un conjunto con raíz pública,
  y ese dos es fijo: no hay k-de-n. Cambiar ese conjunto es otro umbral de
  dos, sobre el conjunto de gobernanza. Y en nodo único la garantía es "dos
  claves comprometidas en vez de una", no "dos voluntades independientes".
- **Las mediciones son una sola ejecución en una máquina.** Sirven para
  comparar órdenes de magnitud, no como benchmark riguroso.
- **Nada de esto ha sido auditado por terceros.** Ninguna cantidad de
  tests propios sustituye a que otro lo mire con intención de romperlo.

---

## 7. Contexto

Trabajo relacionado que sí existe: **zk-Bench** (UCL) cubre la evaluación
comparativa de Groth16, PLONK, halo2 y starky con rigor académico;
**ZK-ACE** trabaja con nullifiers sobre dos backends.

Lo menos común aquí es la **combinación**: cinco paradigmas —no dos ni
tres— aplicados a una **aplicación completa** en vez de a circuitos de
referencia como SHA-256. Y eso importa porque los hallazgos de diseño
solo emergen así: que AIR carezca de restricciones de copia no se
descubre implementando SHA-256, sino portando una actualización de
estado.

---

## Reproducir

```bash
cargo test -p zk-ssl --release              # la capa, 307 tests (3 ign.)
cargo test -p stark-experiment --release    # los circuitos, 318 tests
cargo test -p zk-core --release performance -- --nocapture
cargo test -p halo2-experiment --release real_proof -- --nocapture
cargo test -p plonk-experiment --release performance -- --nocapture
cargo test -p nova-experiment --release --features test-setup -- --nocapture
```
