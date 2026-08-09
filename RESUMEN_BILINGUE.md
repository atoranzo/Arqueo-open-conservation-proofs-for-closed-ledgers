# ZK-SSL — Resumen ejecutivo · Executive Summary

*Español primero, English below.*

---

## 🇪🇸 Qué es

Una **capa de liquidación financiera** donde las transferencias son
privadas, el cumplimiento normativo es **demostrable criptográficamente**,
y **no hace falta confiar en ninguna ceremonia de setup**.

Y el trabajo comparativo que fundamentó su diseño: **el mismo circuito
implementado en cinco sistemas de prueba de conocimiento cero**, medido en
condiciones idénticas.

### Qué NO es

Un **nodo único**. Quien lo opera **ve todos los saldos** y **puede
censurar operaciones**. Eso exige consenso distribuido, que no está
implementado.

**No lo ha auditado nadie.**

### Lo que garantiza, sin revelar identidades, saldos ni importes

Nadie puede crear dinero, gastar de una cuenta ajena, gastar dos veces,
gastar estando congelado, reenviar una operación válida, ni operar sobre
un estado corrupto. Cada garantía tiene su test que intenta romperla.

Y para supervisión: **revelación selectiva** con tres modos —saldo exacto,
mínimo de reservas, o banda ("estoy entre X e Y")—. El titular produce la
prueba; el supervisor la verifica **sin acceso al ledger**. No hay ninguna
clave maestra que robar.

### Los números

| | |
|---|---|
| Arranque de la capa | **0,67 ms** (sin ceremonia, sin claves que generar) |
| Verificar una transferencia | ~4 ms |
| Generarla | ~620 ms |
| **Verificar / generar** | **0,5 %** |
| Prueba de liquidación | 62 KB |
| **Mil transferencias** | **~590 s** y **~124 MiB** acumulados |
| ⚠️ **Techo real bajo concurrencia** | **1,5-1,9 TPS** ← el que muerde antes (anclaje de raíz, §123) |

> ⚠️ **Esa razón es la de la AUDITORÍA, no la de la transferencia.**
>
> `verify_audit` **solo verifica**: 1,6 ms frente a 274 de generación, un
> **0,58 %**. Es la cifra correcta para el argumento que sostiene —un
> supervisor comprueba sin tocar el estado— pero **estaba atribuida a la
> transferencia**.
>
> Aplicar una transferencia cuesta **28,5 %** de generarla, porque `apply`
> **verifica, muta el árbol y escribe a disco**. No es comparable.
>
> Se detectó ejecutando `cargo test -p zk-ssl --release metrics --
> --nocapture` y comparando con lo publicado. Ver `AUDITORIA.md` §22.

### La decisión que define el diseño

**Se descartó Groth16 siendo más rápido y con pruebas 320 veces menores.**

El motivo: exige una ceremonia de confianza cuyos participantes, si
coluden, pueden **crear dinero sin dejar rastro**. Las pruebas falsas
verifican. No hay detección posterior.

Es la única decisión del proyecto tomada **contra** los números de
rendimiento.

### Lo diferenciado

No la capa —Zcash y Aztec llevan años de ventaja— sino **haber implementado
lo mismo cinco veces y medirlo**. De ahí salieron ocho hallazgos que no
están en la literatura comparativa, porque solo aparecen al portar una
**aplicación completa**, no un SHA-256 de referencia.

El principal: **la aritmetización AIR carece de restricciones de copia**,
lo que abre un agujero de solidez silencioso al actualizar árboles de
Merkle. Invisible para testigos honestos.

### Estado

**731 tests en la compuerta de sello** —868 con todos los pines, 882
declarados—, 0 fallos y 24 warnings **pinchados**, ejecutados por
`tools/canon.sh`. Reproducibles con Rust estable, sin instaladores
externos. Los errores propios están documentados, no borrados.

Desde agosto de 2026 hay además **contrato público de protocolo**:
especificación, OpenRPC generado desde el código y vectores de
conformidad que una segunda implementación debe reproducir.

---

## 🇬🇧 What it is

A **financial settlement layer** where transfers are private, regulatory
compliance is **cryptographically provable**, and **no trusted setup
ceremony is required**.

Plus the comparative work that informed its design: **the same circuit
implemented across five zero-knowledge proof systems**, measured under
identical conditions.

### What it is NOT

A **single node**. Whoever operates it **sees every balance** and **can
censor operations**. Fixing that requires distributed consensus, which is
not implemented.

**It has not been audited by anyone.**

### What it guarantees, without revealing identities, balances or amounts

No one can create money, spend from someone else's account, double-spend,
spend while frozen, replay a valid operation, or operate on corrupted
state. Each guarantee has a test that tries to break it.

And for supervision: **selective disclosure** in three modes — exact
balance, minimum reserves, or band ("I am between X and Y"). The holder
produces the proof; the supervisor verifies it **without ledger access**.
There is no master key to steal.

### The numbers

| | |
|---|---|
| Layer startup | **0.67 ms** (no ceremony, no keys to generate) |
| Verifying a transfer | ~4 ms |
| Proving it | ~620 ms |
| **Verify / prove** | **0.5 %** |
| Settlement proof | 62 KB |
| **One thousand transfers** | **~590 s**, **~124 MiB** accumulated |
| ⚠️ **Real ceiling under concurrency** | **1.5-1.9 TPS** ← the limit that bites first (root anchoring) |

### The decision that defines the design

**Groth16 was rejected despite being faster and producing proofs 320×
smaller.**

The reason: it requires a trusted ceremony whose participants, if they
collude, can **create money leaving no trace**. Forged proofs verify. There
is no subsequent detection.

It is the only decision in the project taken **against** the performance
figures.

### What is distinctive

Not the layer — Zcash and Aztec have years of head start — but **having
implemented the same thing five times and measured it**. That produced
eight findings absent from the comparative literature, because they only
surface when porting a **complete application**, not a reference SHA-256.

The principal one: **AIR arithmetization lacks copy constraints**, which
opens a silent soundness gap when updating Merkle trees. Invisible to
honest witnesses.

### Status

**731 tests in the sealing gate** —868 across all pinned levels, 882
declared—, 0 failures and 24 **pinned** warnings, run by
`tools/canon.sh`. Reproducible with stable Rust, no external
toolchains. Our own errors are documented, not erased.

Since August 2026 there is also a **public protocol contract**: a
normative spec, an OpenRPC document generated from the code, and
conformance vectors that a second implementation must reproduce.

---

## Enlaces · Links

| | |
|---|---|
| Repositorio · Repository | `https://github.com/atoranzo/ZK-SSL-ZK-Sovereign-Settlement-Layer-` |
| Revisión de seguridad · Security review | [`AUDITORIA.md`](./AUDITORIA.md) |
| Comparativa · Comparison | [`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md) |
| Artículo · Paper | [`PAPER_EN.md`](./PAPER_EN.md) |
