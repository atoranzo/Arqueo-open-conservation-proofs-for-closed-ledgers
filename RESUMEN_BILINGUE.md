# Arqueo — Resumen ejecutivo · Executive Summary

*Español primero, English below.* Antes ZK-SSL; cambió el nombre, no los identificadores
publicados. Verificado contra `main` en `3294986`. · *Formerly ZK-SSL; the name changed, the
published identifiers did not. Verified against `main` at `3294986`.*

---

## 🇪🇸 Qué es

Un operador lleva un libro cerrado y quienes dependen de él no pueden verlo. Hoy ese conflicto lo
resuelve un tercero que abre el libro, una vez al año y por muestreo. Arqueo sustituye la apertura
del libro por una **prueba de que el libro hizo lo que sus reglas dicen**, que cualquiera comprueba
**sin el libro, sin red y sin fiarse del autor**. Prueba conservación, no solvencia.

### Qué comprueba un tercero, medido

Que el dinero se conserva (lo emitido = cuentas + en vuelo, también al reabrir). Que una unidad se
usó una sola vez en el libro, y que la misma unidad en dos libros se detecta con las dos cabezas
firmadas. Que la historia no se reescribió. Que una entrada está dentro, con recibo. Que sólo el
titular movió su cuenta. Dos propiedades más —corte y completitud, rechazo con causa— están
planeadas, no hechas.

### Qué NO es

No es una cadena: un nodo, sin consenso ni token. El operador ve todos los saldos y puede omitir
una operación sin dejar rastro. Entre libros detecta, no previene. No está auditado por terceros
ni lo usa nadie con dinero real.

### Cómo se comprueba

Con el kit del verificador: una descarga, cuatro comprobaciones en tu máquina y sin red; el
programa dice `VERDE` o nombra la regla rota, y se reproduce desde el commit que su `VERSION`
declara ([`doc/KIT.md`](./doc/KIT.md)).

### Dónde encaja

Donde la unidad nace y muere dentro del libro y hay un tercero que no puede verlo: depósito y
retorno, garantías de origen, monedas comunitarias, custodia de fondos de clientes, ayudas públicas
con doble financiación, registros de derechos, compensación entre operadores. No: cámaras de
contrapartida ni una moneda de banco central ([`doc/USE_CASES.md`](./doc/USE_CASES.md)).

### Estado

17 crates en Rust con canon en cada cambio; protocolo `zkssl/0.3` con 26 métodos y vectores que no
se reescriben; RFC 0002, 0003, 0004 y 0006 aceptados, 0005 y 0007 propuestos; verificador
`zk-ssl-verify` 0.2.0 (release `arqueo-verify-v0.2.0`, reproducible); registro con un asiento por
cambio. Falta: auditoría externa, custodia de clave comprobada, un ancla anterior al primer
encuentro del testigo, y las dos propiedades planeadas.

### La decisión que define el diseño

El mismo circuito medido en cinco sistemas de prueba ([`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md)).
Se descartó Groth16, el más rápido, porque su ceremonia de confianza permite crear dinero sin dejar
rastro; STARK sin ceremonia fue la única decisión tomada contra los números.

---

## 🇬🇧 What it is

An operator keeps a closed ledger and the people who depend on it cannot see it. Today that
conflict is settled by a third party who opens the ledger, once a year and by sample. Arqueo
replaces the opening of the ledger with a **proof that the ledger did what its rules say**, which
anyone checks **without the ledger, offline and without trusting the author**. It proves
conservation, not solvency.

### What a third party checks, measured

That money is conserved (issued = balances + in flight, also on reopening). That a unit was used
once inside the ledger, and that the same unit in two ledgers is detected from their two signed
heads. That history was not rewritten. That an entry is inside, with a receipt. That only the
holder moved their account. Two more properties — cut-off and completeness, rejection with cause —
are planned, not built.

### What it is NOT

Not a chain: one node, no consensus, no token. The operator sees every balance and can omit an
operation without leaving a trace. Across ledgers it detects, it does not prevent. Not audited by
third parties, and nobody uses it with real money.

### How it is checked

With the verifier kit: one download, four checks on your own machine, offline; the program prints
`VERDE` or names the broken rule, and it is reproduced from the commit its `VERSION` declares
([`doc/KIT_EN.md`](./doc/KIT_EN.md)).

### Where it fits

Where the unit is born and dies inside the ledger and a third party cannot see it: deposit-return
schemes, guarantees of origin, community currencies, safeguarding of client funds, public aid
with double funding, registers of entitlements, netting between operators. Not central
counterparties, nor a central-bank digital currency ([`doc/USE_CASES.md`](./doc/USE_CASES.md)).

### Status

17 crates in Rust with the canon on every change; protocol `zkssl/0.3` with 26 methods and vectors
that are never rewritten; RFCs 0002, 0003, 0004 and 0006 accepted, 0005 and 0007 proposed; verifier
`zk-ssl-verify` 0.2.0 (release `arqueo-verify-v0.2.0`, reproducible); a record with one entry per
change. Missing: an external audit, a verified key custody, an anchor prior to the witness's first
encounter, and the two planned properties.

### The decision that defines the design

The same circuit measured in five proof systems ([`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md)).
Groth16, the fastest, was rejected because its trusted ceremony allows creating money without a
trace; STARK without a ceremony was the only decision taken against the numbers.

---

## Enlaces · Links

| | |
|---|---|
| Empezar · Start | [`README.md`](./README.md) · [`README_EN.md`](./README_EN.md) |
| El kit · The kit | [`doc/KIT.md`](./doc/KIT.md) · [`doc/KIT_EN.md`](./doc/KIT_EN.md) |
| Dónde encaja · Where it fits | [`doc/USE_CASES.md`](./doc/USE_CASES.md) |
| Preguntas · Questions | [`PREGUNTAS.md`](./PREGUNTAS.md) · [`QUESTIONS.md`](./QUESTIONS.md) |
| Lo abierto · What is open | [`SECURITY.md`](./SECURITY.md) |
| El registro · The record | [`AUDITORIA.md`](./AUDITORIA.md) |
| El artículo · The paper | [`PAPER.md`](./PAPER.md) · [`PAPER_EN.md`](./PAPER_EN.md) |
