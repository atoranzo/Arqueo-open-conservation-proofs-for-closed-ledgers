# ZK-SSL — capa de liquidación con privacidad y cumplimiento demostrable

Un sistema de liquidación donde las transferencias son privadas, el
cumplimiento normativo es **demostrable criptográficamente**, y **no hace
falta confiar en ninguna ceremonia de setup**.

Más el trabajo comparativo que fundamentó su diseño: **el mismo circuito
implementado en cinco paradigmas de prueba**, medido en condiciones
idénticas.

```rust
let layer = SovereignLayer::open("./ledger", custodios, gobernanza, limite, tope, max_cuentas)?;

let view = layer.account_view(alice)?;
let nullifier = client::compute_nullifier(clave, view.nonce);   // en el cliente
let m = layer.transfer_materials(alice, bob, 250_000, nullifier)?;
let s = client::prove_transfer(&m, clave)?;                     // la clave NO sale
layer.apply(&s, alice, bob, 250_000)?;
```

---

## ⚠️ Léelo antes que nada: el operador es un intermediario de confianza

Esto es un **nodo único**. Quien lo opera:

- **Ve todos los saldos.** La privacidad es frente a terceros que solo ven
  pruebas, no frente a quien mantiene el estado.
- **Ordena las operaciones y puede censurar.**

Ambas cosas exigen consenso distribuido, que **no está implementado** y es
un problema de otra disciplina.

Lo que sí se cerró: **no puede reescribir el historial en secreto**
(registro encadenado de transiciones), ni crear dinero, ni gastar de una
cuenta ajena, ni operar sobre un estado corrupto.

**Qué es esto**: una demostración de que las propiedades criptográficas de
una liquidación soberana son construibles y medibles.
**Qué no es**: una capa descentralizada.

---

## Números medidos

Todos en release, misma máquina. Una sola ejecución: sirven para comparar
órdenes de magnitud, **no como benchmark**.

| Operación | Generar | Verificar | Prueba |
|---|---|---|---|
| **Arranque** | **0,67 ms** | — | — |
| Emisión (2-de-N custodios) | ~105 ms | ~2 ms | 57.342 B |
| Transferencia | ~620 ms | ~4 ms | 61.966 B |
| Destrucción | ~110 ms | ~2 ms | 54.924 B |
| Auditoría (banda) | ~250 ms | ~1,5 ms | 48.782 B |

**Verificar cuesta el 0,5-0,8% de generar.** El arranque no genera claves:
no hay ceremonia ni secreto que destruir.

**Límite cuantificado**: mil transferencias son ~620 s de prueba y
**59,1 MB** acumulados.

---

## Los cinco paradigmas, comparados

| | Groth16 | Halo2/IPA | **STARK/FRI** | PLONK/KZG |
|---|---|---|---|---|
| Ceremonia | Por circuito | Ninguna | **Ninguna** | Universal |
| Setup | 438 ms | 16,3 s | **ninguno** | 26,3 s + 12,8 s |
| Generación | 422 ms | 4,86 s | **39 ms** | 6,85 s |
| Verificación | 5 ms | 91 ms | **1 ms** | 8 ms |
| Tamaño | **192 B** | 4.096 B | 36,7 KB | 1.008 B |
| Post-cuántico | No | No | **Sí** | No |

**Se eligió STARK descartando Groth16**, que es más rápido y produce
pruebas 320 veces más pequeñas. El motivo: Groth16 exige una ceremonia
cuyos participantes, si coluden, pueden **crear dinero sin dejar rastro**.
Es la única decisión del proyecto tomada contra los números.

**Nova/folding** se midió aparte (~250 ms por transacción, constante) y se
descartó para la capa: usa curvas y exige ceremonia.

---

## Orden de lectura

| Si eres… | Empieza por |
|---|---|
| Alguien con 5 minutos | [`RESUMEN_EJECUTIVO.md`](./RESUMEN_EJECUTIVO.md) |
| **Un revisor de seguridad** | [`AUDITORIA.md`](./AUDITORIA.md) |
| Interesado en la comparativa | [`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md) |
| Interesado en el diseño | [`ARQUITECTURA.md`](./ARQUITECTURA.md) |
| Interesado en el planteamiento | [`PRINCIPIOS.md`](./PRINCIPIOS.md) |
| **Interesado en la visión y sus consecuencias** | [`VISION.md`](./VISION.md) |

`AUDITORIA.md` incluye una sección con **los puntos donde el autor tiene
menos confianza**. Si vas a mirar el código con intención de romperlo,
empieza ahí.

---

## Reproducir

Requiere Rust estable. Sin instaladores externos ni toolchains aparte.

```bash
cargo test -p zk-ssl --release              # la capa: 105 tests
cargo test -p stark-experiment --release    # los ocho circuitos
cargo test -p zk-ssl --release metrics -- --nocapture
```

La comparativa completa:

```bash
cargo test -p zk-core --release performance -- --nocapture
cargo test -p halo2-experiment --release real_proof -- --nocapture
cargo test -p plonk-experiment --release performance -- --nocapture
cargo test -p nova-experiment --release --features test-setup -- --nocapture
```

**Los tests de circuito conviene ejecutarlos también en debug**: winterfell
solo valida las restricciones al generar en ese modo, y da el índice y la
fila exactos del fallo.

---

## Qué garantiza el sistema

Sin revelar identidades, saldos ni importes:

| Vía de ataque | Cerrada por |
|---|---|
| Transferir más de lo debitado | Conservación (partida doble) |
| Abrir cuenta con saldo | Apertura siempre a cero |
| Emitir sin autorización | **Dos custodios** demostrados en circuito |
| Emisión encubierta | Suministro público atado en el circuito |
| Superar el tope de emisión | Tope inmutable del ledger |
| Gastar dos veces | No-pertenencia demostrable |
| Gastar sin ser el titular | Autoridad de gasto |
| **Gastar estando congelada** | No-pertenencia al árbol de congelados |
| Reenviar una operación válida | Encadenamiento de raíces |
| Operar sobre estado corrupto | Verificación de integridad al arrancar |
| **Reescribir el historial** | Registro encadenado de transiciones |

Y para cumplimiento: **revelación selectiva** con tres modos —saldo
exacto, mínimo de reservas, y banda ("estoy entre X e Y")—. El titular
produce la prueba; el supervisor la verifica **sin acceso al ledger**.

---

## Ocho hallazgos

Ninguno aparece en los materiales que comparan paradigmas. Todos surgieron
al construir. Detallados en [`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md):

1. **AIR carece de restricciones de copia**, y eso abre un agujero
   silencioso al portar actualizaciones de estado.
2. El campo **Goldilocks es demasiado estrecho para identidades**: 64 bits
   son colisión en 2³².
3. Sin extensión de campo, un **STARK sobre Goldilocks tiene techo de 63
   bits** de solidez.
4. La brecha entre seguridad **conjeturada y demostrable**: 127 bits
   conviven con 29-63.
5. **PLONK-KZG resultó el generador más lento** de los cuatro basados en
   curvas.
6. Solo **dos de seis librerías se defienden del uso inseguro** en código.
7. El **ecosistema PLONK-KZG en Rust son stacks verticales cerrados**:
   seis vías investigadas, cinco rotas.
8. **Un zkVM no es comparable en igualdad de condiciones**, y la cifra que
   lo mide son 3 dependencias frente a 349.

---

## Estado y límites

**No auditado por terceros.** Ninguna cantidad de tests propios lo
sustituye.

Lo que falta, por orden de importancia:

- **Consenso distribuido.** Sin él, el operador ve los saldos y puede
  censurar.
- **Auditoría externa.**
- Delegación de la prueba a terceros (verificar firma en circuito).
- Política de caducidad para congelaciones; justificación registrada.

Todo lo demás que falta está enumerado en
[`AUDITORIA.md`](./AUDITORIA.md), sección 4.

---

## Licencia

MIT o Apache-2.0, a elección.
