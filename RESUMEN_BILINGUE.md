# Arqueo — Resumen ejecutivo · Executive Summary

*Español primero, English below.* Antes ZK-SSL; cambió el nombre, no los identificadores
publicados. Verificado contra `main` en `3294986`. · *Formerly ZK-SSL; the name changed, the
published identifiers did not. Verified against `main` at `3294986`.*

---

## 🇪🇸 Qué es

Un operador lleva un libro cerrado y quienes dependen de él no pueden verlo. Hoy ese conflicto lo
resuelve un tercero que abre el libro, una vez al año y por muestreo. Arqueo sustituye la apertura
del libro por **pruebas de que el libro hizo lo que sus reglas dicen**, y una parte la comprueba
cualquiera **sin el libro, sin red y sin fiarse del autor**. Prueba conservación, no solvencia.

### Qué comprueba un tercero, medido

Que una unidad se usó una sola vez en el libro, y que la misma unidad en dos libros se detecta con
las dos cabezas firmadas. Que la historia no se reescribió. Que una entrada está dentro, con
recibo. Que una negativa lleva su causa y, para las causas que el sobre de rechazo cubre, que se
sostiene sin el nodo. Que nada en vuelo es más viejo que una edad dada.
Y la completitud, medida desde el RFC-0010: lo que el nodo evalúa por las vías del titular deja
un recibo bajo su cabeza firmada y acaba aplicado, rechazado con causa, o señalado con un rojo
nombrado; sus residuos, declarados. ⚠️ Corregido en el §589: aquí decía «Falta la completitud».

Que cada pago conserva el dinero y que sólo el titular movió su cuenta van dentro de la prueba de
cada pago, y esa prueba la comprueba el nodo, no un tercero: el registro guarda su resumen y el kit
no la vuelve a verificar. El agregado (lo emitido = cuentas + en vuelo) lo comprueba la capa del
nodo al reabrir el libro. ⚠️ Corregido en el §696: las dos estaban en la lista de arriba, y «Qué
es» decía «una prueba de que el libro hizo lo que sus reglas dicen, que cualquiera comprueba», sin
excepción.

### Qué NO es

No es una cadena: un nodo, sin consenso ni token. El operador ve todos los saldos y puede omitir
una operación: sin dejar rastro si no emite recibo; si lo emitió, el sobre de completitud lo nombra
(RFC-0010). Entre libros detecta, no previene. No está auditado por terceros
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

22 crates en Rust (19 propios y el fork de winterfell 0.13.1 en tres) con canon en cada cambio;
protocolo `zkssl/0.4` con 32 métodos y vectores que no se reescriben;
RFC 0002, 0003, 0004, 0006, 0007, 0008, 0009, 0010, 0011, 0012 y 0014 aceptados, 0005, 0013, 0015, 0016, 0017, 0018 y 0019 propuestos; verificador
`zk-ssl-verify` 0.4.2 (publicada, `arqueo-verify-v0.4.2`, reproducible); registro con un asiento por
cambio. Falta: auditoría externa, custodia de clave comprobada, y un ancla anterior al primer
encuentro del testigo (⚠️ §589: esta lista pedía también «la completitud de los acuses»).

### La decisión que define el diseño

El mismo circuito medido en cinco sistemas de prueba ([`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md)).
Se descartó Groth16, el más rápido, porque su ceremonia de confianza permite crear dinero sin dejar
rastro; STARK sin ceremonia fue la única decisión tomada contra los números.

---

## 🇬🇧 What it is

An operator keeps a closed ledger and the people who depend on it cannot see it. Today that
conflict is settled by a third party who opens the ledger, once a year and by sample. Arqueo
replaces the opening of the ledger with **proofs that the ledger did what its rules say**, and
part of it anyone checks **without the ledger, offline and without trusting the author**. It
proves conservation, not solvency.

### What a third party checks, measured

That a unit was used once inside the ledger, and that the same unit in two ledgers is detected
from their two signed heads. That history was not rewritten. That an entry is inside, with a
receipt. That a refusal carries its cause and, for the causes the rejection envelope covers, that
it holds without the node. That nothing in flight is older than a given age. And completeness,
measured since RFC-0010: what the node evaluates through the holder's direct paths leaves a
receipt under its signed head and ends applied, rejected with a cause, or flagged by a named red;
its residues, declared. ⚠️ Corrected in §589: this read "Missing: completeness".

That each payment conserves money and that only the holder moved their account travel inside the
proof of each payment, and that proof is checked by the node, not by a third party: the log keeps
its digest and the kit does not verify it again. The aggregate (issued = balances + in flight) is
checked by the node's layer on reopening the ledger. ⚠️ Corrected in §696: both were in the list
above, and «What it is» said «a proof that the ledger did what its rules say, which anyone
checks», with no exception.

### What it is NOT

Not a chain: one node, no consensus, no token. The operator sees every balance and can omit an
operation: without a trace if it issues no receipt; if it issued one, the completeness envelope
names it (RFC-0010). Across ledgers it detects, it does not prevent. Not audited by
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

22 crates in Rust (19 of our own and the winterfell 0.13.1 fork in three) with the canon on every
change; protocol `zkssl/0.4` with 32 methods and vectors that are never rewritten;
RFCs 0002, 0003, 0004, 0006, 0007, 0008, 0009, 0010, 0011, 0012 and 0014 accepted, 0005, 0013, 0015, 0016, 0017, 0018 and 0019 proposed;
verifier `zk-ssl-verify` 0.4.2 (published, `arqueo-verify-v0.4.2`, reproducible); a record with one
entry per change. Missing: an external audit, a verified key custody, and an anchor prior to the
witness's first encounter (⚠️ §589: this list also asked for "the completeness of
acknowledgements").

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
