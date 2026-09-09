# Arqueo — Resumen ejecutivo

Para quien tiene cinco minutos y no es técnico. En dos lenguas y más corto:
[`RESUMEN_BILINGUE.md`](./RESUMEN_BILINGUE.md). El proyecto se llamaba antes ZK-SSL; cambió el
nombre, no los identificadores publicados (`zkssl/0.3`, `zk-ssl-*`). Lo que dice esta página está
verificado contra `main` en el commit `3294986`.

## Qué es, en tres frases

Un operador lleva un libro cerrado —cuentas, pagos, emisiones, retiradas— y quienes dependen de él
no pueden verlo: socios, titulares, beneficiarios, contrapartes. Hoy ese conflicto lo resuelve un
tercero que abre el libro: un auditor, un supervisor, un juzgado; una vez al año; por muestreo.
Arqueo sustituye la apertura del libro por una **prueba de que el libro hizo lo que sus reglas
dicen**, que cualquiera comprueba **sin el libro, sin red y sin fiarse del autor**.

## Qué puede comprobar hoy un tercero, medido

- **Que el dinero se conserva**: lo emitido es igual a lo que hay en las cuentas más lo que está en
  vuelo; nada se crea ni se pierde entre una época y la siguiente, tampoco al reabrir el libro.
- **Que una unidad se usó una sola vez** dentro del libro, y que si dos libros distintos aceptaron
  la misma unidad, se **detecta** con las dos cabezas firmadas, sin que ningún nodo participe.
- **Que la historia no se reescribió**: la cabeza de hoy extiende la de ayer sin borrar ni reordenar.
- **Que una entrada está dentro**, con un recibo que no depende del operador.
- **Que sólo el titular movió su cuenta**: el operador no puede, y la clave nunca viaja.

Dos cosas más que el motor quiere responder y todavía no responde: el corte y la completitud de
un periodo, y el rechazo con causa. Están listadas como planeadas, no como hechas.

## Qué NO es

No es una cadena: un nodo, un escritor, sin consenso distribuido ni token. **El operador ve todos
los saldos** y puede omitir una operación sin dejar rastro. Entre libros detecta, no previene. No
está auditado por terceros. Nadie lo usa con dinero real. Y no prueba solvencia: prueba que el
libro es coherente consigo mismo, no que sus unidades existan fuera de él.

## Cómo se comprueba, sin saber programar

Con **el kit del verificador**: una descarga, un programa de una sola línea y cuatro
comprobaciones en la máquina del que comprueba, sin red. Un expediente que cuadra. Uno manipulado
que no cuadra, y el programa dice qué regla se rompió. La misma unidad publicada en dos libros,
detectada con los dos nodos apagados. Y un intercambio de libros, rechazado con su nombre. Las
huellas del programa se publican junto al commit del que sale, y el programa se reproduce desde
ese commit: no hay que fiarse de la descarga. Guion paso a paso: [`doc/KIT.md`](./doc/KIT.md).

## Dónde encaja

Donde la unidad de cuenta **nace y muere dentro del libro** y hay un tercero que no puede verlo:
sistemas de depósito y retorno, garantías de origen y derechos de emisión, monedas comunitarias,
custodia de fondos de clientes, ayudas públicas donde el fraude es la doble financiación,
registros de derechos y cupos, compensación entre operadores o entre administraciones. No sirve a
una cámara de contrapartida central, cuyo problema es el riesgo de contraparte, ni es un componente
de una moneda digital de banco central. Los casos, con qué propiedad resuelve cada uno y cuáles
están revisados: [`doc/USE_CASES.md`](./doc/USE_CASES.md).

## Qué existe, medido

| pieza | estado |
|---|---|
| El motor | 17 crates en Rust; cada cambio pasa por el canon (los tests de todos los crates y ocho herramientas que vigilan cifras, citas, dominios y geometría) |
| El protocolo | `zkssl/0.3`: 26 métodos JSON-RPC, vectores de conformidad por versión que jamás se reescriben; RFC 0002, 0003, 0004 y 0006 aceptados, 0005 propuesto |
| El verificador | `zk-ssl-verify` 0.2.0, release `arqueo-verify-v0.2.0`, reproducible desde el commit que su `VERSION` nombra |
| El registro | [`AUDITORIA.md`](./AUDITORIA.md): un asiento por cambio verificado, con su commit; lo que se corrige se marca, no se borra |

## La decisión de fondo

El diseño se eligió midiendo el mismo circuito en cinco sistemas de prueba de conocimiento cero
([`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md)). Se descartó el más rápido y el que produce las pruebas
más pequeñas, Groth16, porque exige una ceremonia de confianza cuyos participantes, si coluden,
pueden **crear dinero sin dejar rastro**. STARK sin ceremonia —sólo hashes, sin curvas, resistente a
un adversario cuántico en su solidez— fue la única decisión del proyecto tomada contra los números.
De ese trabajo salieron ocho hallazgos que no estaban en la literatura comparativa y seis depósitos
con DOI; lo que se corrigió después está marcado en su fe de erratas.

## Qué falta para que un tercero real se apoye en esto

Una auditoría externa, que no depende de más código. Una custodia de la clave de firma
**comprobada**, no sólo declarada. Un ancla anterior al primer encuentro entre el testigo y el
nodo. Las dos propiedades planeadas. El consenso distribuido es otra disciplina y no es el camino
de este proyecto: el camino es la responsabilidad demostrable, al modo de *Certificate
Transparency*, y sus piezas están construidas.

## Para seguir

| | |
|---|---|
| Empezar | [`README.md`](./README.md) |
| Comprobarlo sin fiarte de nadie | [`doc/KIT.md`](./doc/KIT.md) |
| Dónde encaja y dónde no | [`doc/USE_CASES.md`](./doc/USE_CASES.md) |
| Veintidós preguntas | [`PREGUNTAS.md`](./PREGUNTAS.md) |
| Lo que sigue abierto | [`SECURITY.md`](./SECURITY.md) |
| Romperlo | [`AUDITORIA.md`](./AUDITORIA.md) |
| El artículo | [`PAPER.md`](./PAPER.md) |
