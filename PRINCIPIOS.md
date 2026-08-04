# ZK-Sovereign Settlement Layer (ZK-SSL)

**Documento de principios y arquitectura — versión contrastada con la
implementación**

---

## Nota sobre esta versión

Este documento parte del planteamiento original del proyecto y lo
**contrasta con lo que se ha construido y medido**. Los principios se
conservan íntegros; las afirmaciones técnicas se han sustituido por
resultados verificables.

Cada componente lleva su estado real:

| | |
|---|---|
| ✅ | Implementado y verificado con tests |
| ⚠️ | Parcial — se indica qué falta |
| ❌ | No implementado |

Todas las cifras proceden de mediciones propias en una misma máquina, en
modo release. Ninguna se cita de la literatura.

---

## 0. Lo primero: qué NO es esto todavía

El principio fundador del proyecto es **eliminar la dependencia de
intermediarios de confianza centralizados**. Se ha eliminado uno, y es
importante: los participantes de una ceremonia de setup, que en Groth16 o
PLONK-KZG podrían coludir y falsificar pruebas —creando dinero sin que
nadie lo detectara jamás—.

**Pero la implementación actual es un nodo único.** Quien lo opera ve
todos los saldos, ordena las operaciones, puede censurar, y es un punto
único de fallo.

**Ese operador es exactamente el intermediario de confianza centralizado
que el principio señala.** Sigue ahí.

### Qué está demostrado y qué se confía

| Afirmación | ¿Demostrada matemáticamente? |
|---|---|
| Esta transferencia conserva el dinero | ✅ |
| El saldo de esta cuenta es X | ✅ |
| No se ha creado dinero fuera del suministro | ✅ |
| Nadie gasta sin ser el titular | ✅ |
| Nadie gasta dos veces | ✅ |
| **Este es el estado actual del sistema** | ❌ Confianza en el operador |
| **Estas son todas las operaciones que hubo** | ❌ Nada impide omitir |

**Las transiciones están demostradas. El estado y la completitud del
historial, no.**

**Lo que esto es**: una demostración de que las propiedades
criptográficas de una liquidación soberana son construibles y medibles.
**Lo que no es**: una capa descentralizada.

Este apartado va primero, y no en una lista de limitaciones, porque
enterrarlo sería vender una propiedad mientras se esconde su contraria.

---

## 1. Introducción

En un contexto de fragmentación geopolítica, saturación de vigilancia
digital y erosión de la confianza en instituciones centralizadas, hace
falta infraestructura que reconcilie **privacidad profunda con
cumplimiento normativo demostrable**.

Ese sigue siendo el planteamiento. Lo que este proyecto aporta es
**evidencia de que esa reconciliación es técnicamente construible**, con
las propiedades verificadas una a una y el coste de cada una medido.

---

## 2. Declaración del problema

Los sistemas actuales presentan deficiencias estructurales:

- **Centralizados**: vulnerables a censura, puntos únicos de fallo y
  abuso de poder.
- **Blockchain clásica**: transparencia excesiva, escalabilidad limitada,
  difícil adopción institucional.
- **Financieros tradicionales**: liquidación lenta, costes de
  intermediación, dependencia de confianza institucional.

**Añadido tras la implementación**: hay un cuarto problema que no estaba
en el planteamiento original y que resultó ser el más determinante.

> **La mayoría de los sistemas ZK exigen una ceremonia de confianza.**
> Groth16 y PLONK-KZG necesitan participantes que generen parámetros y
> destruyan un secreto. Si coluden, pueden crear dinero sin que nadie lo
> detecte jamás. Es una dependencia externa, permanente e inauditable —y
> es incompatible con "soberanía".

Esa observación, que solo se hace evidente al implementar, es la que
determinó toda la arquitectura.

---

## 3. Objetivos del protocolo

| Objetivo | Estado |
|---|---|
| Soberanía sobre datos y capital | ⚠️ Criptográfica sí; frente al operador, no |
| Liquidación con finalidad criptográfica | ✅ |
| Privacidad selectiva compatible con regulación | ✅ |
| Capa base neutral, minimalista y resistente | ⚠️ Ver apartado 0 |
| Transición ordenada desde infraestructuras tradicionales | ❌ |

---

## 4. Arquitectura técnica

### 4.1 Componentes

**ZK Core Engine** — ⚠️
Se implementó el mismo circuito en **cinco paradigmas** (Groth16,
Halo2/IPA, STARK/FRI, PLONK/KZG, Nova/folding) y se midieron sus
trade-offs. La agregación recursiva **no existe**: solo se probó plegado
(Nova) como prueba de concepto, y se descartó para la capa por
reintroducir ceremonia y perder resistencia cuántica.

**Sovereign Settlement Layer** — ✅
`crates/zk-ssl`. Mantiene el estado, encadena raíces, aplica operaciones.
172 tests.

**Autenticación criptográfica estructural** — ⚠️
Hay claves de gasto e identidades derivadas criptográficamente
(`pk = H(DOMAIN, sk)`), con separación de dominio. **No hay DIDs ni
SSI.**

**Execution Environment** — ⚠️
La capa consume **pacs.008** y devuelve **pacs.002** con la prueba
adjunta, traduciendo sus errores a códigos de motivo ISO reales
(`AM04`, `AC01`, `AG01`...). No es un parser XML ni cubre el estándar
completo, y no hay máquina virtual.

**Privacy & Compliance Module** — ✅
Revelación selectiva con tres modos: saldo exacto, mínimo, y **banda**
("estoy entre X e Y"). El titular produce la prueba; el supervisor la
verifica **sin acceso al ledger**.

**Governance Layer** — ✅
Jerarquía de dos niveles: los **custodios** emiten y recuperan cuentas;
el **conjunto de gobernanza** puede cambiar a los custodios. Ambos exigen
2-de-N, ambos con contador público de intervenciones.

El límite regulatorio y el tope de emisión siguen siendo inmutables.

⚠️ **El conjunto de gobernanza es inmutable**: si se compromete, la única
salida es crear un ledger nuevo. La circularidad no se resuelve, se
traslada a claves que se usan casi nunca y pueden guardarse sin
conexión.

### 4.2 Flujo operativo real

```rust
let mut layer = SovereignLayer::open("./ledger", issuer_key, limite, tope)?;

let alice  = layer.open_account(sk_alice);              // saldo CERO
let recibo = layer.mint(issuer_key, alice, 1_000_000)?; // EXIGE clave del emisor
layer.apply_mint(&recibo, alice)?;

// Un pago son DOS fases: el dinero sale, y el receptor lo cobra.
let envio = layer.send(sk_alice, alice, &estado, id_bob, aleatorio, 250_000)?;
layer.apply_send(&envio, alice, &estado, 250_000)?;
let cobro = layer.claim(sk_bob, bob, &estado_bob, &envio.notice)?;
layer.apply_claim(&cobro, bob, &estado_bob, &envio.notice)?;

let d = layer.audit(sk_alice, alice, 900_000, 1_100_000)?;
verify_audit(&d)?;                                       // el supervisor, sin la capa
```

Generar la prueba y aplicarla están **separados a propósito**: permite
que quien produce la prueba y quien la acepta sean partes distintas.

---

## 5. Fundamentos tecnológicos

### La decisión que define la arquitectura

Se descartó **Groth16 pese a ser el más rápido y tener pruebas 320 veces
más pequeñas** (192 bytes frente a ~65 KB — §130).

El motivo no fue técnico sino de coherencia: exige ceremonia de
confianza. Sin ceremonia quedan Halo2/IPA y STARK/FRI; de los dos, STARK
gana en todo salvo tamaño y es el único post-cuántico.

**Es la única decisión del proyecto tomada contra los números de
rendimiento.**

### Comparativa medida

| | Groth16 | Halo2/IPA | **STARK/FRI** | PLONK/KZG |
|---|---|---|---|---|
| Ceremonia | Por circuito | Ninguna | **Ninguna** | Universal |
| Setup | 438 ms | 16,3 s | **ninguno** | 26,3 s + 12,8 s |
| Generación | 422 ms | 4,86 s | **39 ms** | 6,85 s |
| Verificación | 5 ms | 91 ms | **1 ms** | 8 ms |
| Tamaño | **192 B** | 4.096 B | 36,7 KB | 1.008 B |
| Post-cuántico | No | No | **Sí** | No |

**Nova/folding**, medido aparte por ser de naturaleza distinta:
**~250 ms por transacción, constante**; cierre de 1,84 s amortizable.
Descartado para la capa: usa curvas y exige ceremonia.

---

## 6. Comparación con proyectos existentes

**Corrección respecto al planteamiento original**, que afirmaba
alineación con Sovereign SDK y zkSync sin contrastarla.

| Proyecto | Relación real |
|---|---|
| **Zcash** | En producción desde 2016, diseño más completo que este |
| **Aztec, Aleo, Miden** | Equipos de decenas de personas, años de trabajo |
| **zk-Bench** (UCL) | **Cubre el eje comparativo con rigor académico** |
| **ZK-ACE** | Nullifiers sobre dos backends |
| **Drex, mBridge, Project Hamilton** | Consorcios institucionales con respaldo |

**Lo diferenciado aquí es más estrecho de lo que sugería el documento
original**: no la comparativa de rendimiento —que zk-Bench ya cubre— sino
los **hallazgos de diseño que emergen al portar una aplicación completa**
entre paradigmas. Eso no se descubre midiendo SHA-256.

---

### 6.bis — El «último intermediario» y el dinero cuántico

Hay un argumento reciente que este protocolo toma en serio: *Bitcoin
eliminó a los bancos pero no a la confianza — la trasladó al consenso,
que es el último intermediario*. El dinero cuántico (teorema de
no-clonación) promete eliminarlo por física; su memoria aún no existe a
escala.

ZK-SSL es el intento clásico de la misma dirección, con herramientas de
hoy, y conviene decir con precisión qué elimina y qué no:

**Eliminado**: la ceremonia de setup (STARK/FRI — no hay secreto
retenible por terceros); las claves de custodio en el nodo para las
operaciones migradas a la vía delegada; y, desde la entrada 50, la
recuperación de saldos por diccionario sobre los caminos que el
protocolo entrega (hoja envuelta, §117).

**Reducido pero presente**: el operador. Ve el estado —la privacidad es
frente a terceros—, ordena, y podría omitir historial. Ese residuo
tiene nombre y ataque diseñados: cabezas atestiguadas y **acuse**
(§121, `doc/CONFIANZA_RESIDUAL.md`), que convierten «confía en el
operador» en «el operador no puede mentir sin dejar evidencia
fail-stop».

**Fuera del dominio clásico**: la garantía física. Un STARK es solidez
computacional con margen amplio y post-cuántico (evaluación XMSS en
`doc/xmss-evaluacion.md`), no un teorema de la naturaleza. Cuando la
memoria cuántica madure, el dinero cuántico podrá retirar al último
intermediario del todo; hasta entonces, este diseño busca dejarlo
**mínimo, medido y con evidencia** — que es lo máximo que la
criptografía clásica honesta puede prometer.

## 7. Aportaciones reales

Siete hallazgos, ninguno presente en los materiales que comparan
paradigmas:

**1. AIR carece de restricciones de copia.** Al portar la actualización
de estado a STARK apareció un agujero que no existe en los otros
paradigmas: nada obliga a que las dos subidas del árbol usen los mismos
hermanos. Es **silencioso** — los testigos honestos nunca lo revelarían.
Obligó a diseñar un patrón en lockstep.

**2. Goldilocks es demasiado estrecho para identidades.** Un elemento son
64 bits: colisión en 2³². En BLS12-381 (255 bits) el problema no existe.

**3. Techo de 63 bits de solidez** en STARK sobre Goldilocks sin
extensión de campo. La configuración por defecto **no es comparable** con
los ~128 bits de los otros.

**4. Brecha entre seguridad conjeturada y demostrable**: 127 bits
conviven con 29-63. Cerrarla cuesta 125,6 KB en vez de 36,7.

**5. PLONK-KZG resultó el generador más lento** de los cuatro, 16-22
veces más lento que Groth16. *Matiz: parte puede deberse a la
implementación, y estos datos no permiten separarlo.*

**6. Solo `nova-snark` impide en código el setup de una sola parte.** Las
otras cuatro librerías lo permiten y confían en la documentación.

**7. El ecosistema PLONK-KZG en Rust son stacks verticales cerrados.**
Seis vías investigadas, cinco rotas.

---

## 8. Hoja de ruta

**Corrección de la original**, que preveía testnet en 2026-2027 y mainnet
en 2027-2028. Eso no es alcanzable con los recursos actuales y decirlo
sería faltar al principio de transparencia.

### Lo hecho

- Cinco paradigmas implementados y medidos.
- Capa de liquidación con ciclo monetario completo, persistencia,
  auditoría y verificación de integridad.
- 172 tests en la capa; cada propiedad de seguridad con test
  discriminante.

### Lo alcanzable

| Fase | Contenido | Corrige un principio |
|---|---|---|
| **1** | Refactor + delegación de prueba + emisión con umbral | **Sí** |
| **2** | Puente ISO 20022 + métricas de la capa | No |
| **3** | Publicación | — |

### Lo que requeriría otros recursos

**Descentralización del estado** (20-40 rondas, sistemas distribuidos).
Es la única pieza que corregiría los principios de neutralidad y
soberanía, y está fuera del alcance actual. **Sin ella, esos principios
no se cumplen.**

**Auditoría externa.** Ninguna cantidad de tests propios la sustituye.

---

## 9. Método

Lo que distingue este trabajo no es el código, sino cómo se verificó.

**Cada propiedad tiene un test discriminante**: un testigo *internamente
coherente* que solo viola la restricción concreta. Un testigo corrupto a
lo bruto rompe varias a la vez, y el test pasa aunque la que interesa no
haga nada.

**Tres veces un test negativo resultó no discriminar** y hubo que
rehacerlo. En los tres casos el código era correcto; el test no probaba
lo que decía.

**Errores propios documentados, no borrados:**
- La comparativa mezcló cifras de debug con release, haciendo parecer a
  STARK 130 veces más rápido que Groth16 cuando la cifra real es ~11.
- Hacer el nullifier privado rompía la capacidad de la capa de mantener
  su árbol.
- `apply` no insertaba el nullifier: habría hecho vacua la garantía más
  cara del sistema.

---

## 10. Conclusión

El planteamiento original describía una infraestructura para "un nuevo
orden económico fundamentado en la verdad matemática".

**Lo construido es más modesto y más verificable**: una demostración de
que las propiedades criptográficas de una liquidación soberana —
privacidad, cumplimiento demostrable, conservación del dinero, auditoría
selectiva, ausencia de ceremonia— son construibles, medibles y
compatibles entre sí.

Falta lo que convertiría eso en infraestructura: descentralización,
auditoría externa, y adopción. **Ninguna de las tres es un problema
criptográfico.**

Si el criterio es transparencia, coherencia e imagen fiel de la realidad,
esta es la descripción exacta de lo que hay.
