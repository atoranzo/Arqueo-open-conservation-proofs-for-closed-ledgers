<!--
  ANCLAJE_EXTERNO — la cabeza publicada donde el operador no puede borrarla.
  Documento de diseño (medida 10, §174). Hermano de CONFIANZA_RESIDUAL.md y
  de CADUCIDAD_PENDIENTE.md. La vía de ELIMINACIÓN del residuo #1 (orden y
  completitud) que el acuse (§121) solo acota. No toca circuito, no toca el
  cable, no toca la cabeza: todo sobre las cabezas que ya existen.
-->

# ANCLAJE_EXTERNO — un reloj que ni el operador ni el emisor controlan

⚠️ **Este texto es una RECONSTRUCCIÓN (§590, sesión 194).** El documento
original nació en el §174 y el árbol publicado en el S548 lo llevaba como
marcador de posición de una línea: el contenido se perdió al publicar. Lo
que sigue se reconstruye desde el acta del §174 —que resume qué decía— sin
contradecir ninguna de sus afirmaciones, y se declara como reconstrucción,
no como el texto original. La nota §9 lleva el detalle.

## 0. El problema, con su acta

El operador ordena y puede omitir, y el registro que lo ata vive en sus
manos: es el **residuo #1 (orden y completitud)** de `SECURITY.md` §2.bis,
declarado desde el primer día. Las piezas construidas lo **acotan**: la
cabeza firmada con su latido (§115), el techo `N_max` del acuse (§121), el
recibo de recepción y su sobre de completitud (RFC-0010). Pero todas
comparten un supuesto: que alguien AJENO custodie cabezas. El testigo las
custodia **desde que las ve** (TOFU, §245), y `CONFIANZA_RESIDUAL.md` §8
dejó dos puntos abiertos: la **vista dividida** —el operador firma cabezas
distintas a testigos distintos, cada historia internamente consistente, y
«no hay nadie situado para comparar»— y el **RPO del siniestro** —lo
posterior a la última cabeza atestiguada solo se recupera si el operador
coopera—. La medida 10 pidió la interfaz que los ataca: **publicar lo que
la cabeza ya firma en un medio append-only independiente del operador**
(§174). Este documento es esa interfaz.

## 1. El ataque que mata al diseño ingenuo — y las dos herencias

**Firmar el ancla revienta la aritmética.** Un ancla con firma propia por
época quemaría índices XMSS al ritmo del latido: es la misma trampa que el
§121.2 mató para el acuse («firma-por-acuse, jamás»). **El ancla hereda la
firma de la cabeza: cero índices nuevos.**

**Confiar en el gossip repite el error de CT.** Certificate Transparency
enseñó las dos caras: el patrón funciona, y su pieza de comparación entre
testigos estuvo años infradesplegada (`CONFIANZA_RESIDUAL.md` §8). Un
diseño que exija testigos que se hablen hereda esa fragilidad. **El ancla
la elimina: contra un medio append-only público, la vista dividida es
AUTODELATORA sin gossip** —dos anclas de la misma clave para el mismo
índice son la delación, y las publica el propio operador o no hay ancla—.
B10.7 convierte la comparación de B10.2 en **lectura**.

**Reconstruir un árbol de cabezas sería un segundo productor.** La cabeza
ya firma «la raíz de un árbol de cabezas»: la pareja del MMR
(`mmrRoot`, `mmrSize`), dentro del digest desde la v3 (§291–§292), con
inclusión y consistencia O(log N) en el verificador. **El lote del ancla
es ese árbol, tal cual**: anclar una cabeza ancla las `mmrSize` anteriores
—el ancla individual de cada una es un camino Merkle— y ata a las
posteriores por consistencia. Nada nuevo se construye ni se firma.

## 2. El mecanismo

**El ancla** de una cabeza firmada es un objeto de seis campos, derivable
por CUALQUIERA —operador, testigo o titular— de la cabeza sola:

```text
ancla = { v: 1,
          clave:       huella_de_clave(publicKey)   ← quién firma (32 B)
          indice:      índice XMSS EMBEBIDO         ← el reloj (§399, §567)
          epochDigest: lo que la firma cubre        ← la historia entera
          mmrRoot:     la raíz del árbol de cabezas ← el lote (§292)
          mmrSize:     cuántas contiene }
huella_del_ancla = ancla_digest(los cinco)          ← 32 B, lo que se publica
```

- **El índice es el EMBEBIDO en la firma**, no el declarado: es «el único
  que la firma acredita» (§399), y es el reloj que un censor no congela
  (§567): la clave solo firma cabezas, y cada firma quema un índice.
- **La huella cabe en cualquier medio**: 32 bytes. Quien tenga el ancla
  entera la recompone; quien tenga solo la huella exige el ancla a quien
  la publicó, y la comprueba.
- **La cadencia es `M`**: un ancla cada `M` latidos. `M` es **línea
  sistémica declarada** —familia de `N_max` (§121) y `T`
  (`CADUCIDAD_PENDIENTE.md`)—: se elige, se publica, se mide. Propuesta
  hasta que haya operador real: **`M` = 1.440 latidos** —un ancla al día,
  el horizonte de `N_max` y el MMD de CT—.
- **El medio NO se elige aquí** (B10.6): un log de transparencia que
  acepte digests, un boletín, una cadena pública barata, un periódico.
  Lo que se exige del medio es una propiedad, no una marca: **append-only
  ajeno al operador, con orden legible por terceros**. La entrada 86 del
  BACKLOG lleva lo que hay que medir antes de elegir: qué logs aceptan
  digests arbitrarios, con qué cadencia, a qué coste.

**Lo que un tercero comprueba, sin el nodo** (B10.7, el verificador en
cliente): que un ancla ES la cabeza firmada que dice ser; que una cabeza
posterior EXTIENDE un ancla (consistencia contra su `mmrRoot`); que una
cabeza anterior ESTÁ en el lote de un ancla (inclusión, el camino
Merkle); y, con dos cabezas de la misma clave y el mismo índice embebido
con digests distintos, la **vista dividida con nombre**: el par
condenatorio que `CONFIANZA_RESIDUAL.md` §2.1 prometió, portable.

## 3. La ventana entre anclas, declarada

Entre ancla y ancla el operador puede contar historias distintas a partes
distintas; el ancla siguiente lo obliga a elegir UNA y a delatarse ante
quien custodie la otra. La exposición es la **cola entre anclas**: acotada
por `M` —y en importe por N2 (`CONFIANZA_RESIDUAL.md` §5.4)—, **no
cerrada**. `M` corto encarece el medio; `M` largo alarga la ventana. Es el
mismo compromiso que `N_max` y `T`, y se decide como ellos: con datos y
publicado.

## 4. El primer encuentro, acotado

El ancla lleva la **huella de la clave** dentro. La primera ancla de una
clave en un medio append-only es un hecho FECHADO POR EL MEDIO y anterior
a cualquier encuentro posterior: el testigo que llega tarde ya no fija lo
primero que ve (TOFU, §245), sino lo que el medio dice que la clave era
ANTES de conocerlo. Es la composición que la entrada 86 nombró: **el ancla
externa acota la ventana del primer encuentro que el TOFU deja abierta**.
Lo que no hace: la fecha es la palabra del medio —confianza desplazada,
§6—, y si la clave YA mentía antes de su primera ancla, el ancla fija la
mentira, igual que el TOFU (§246: lo que falta no es el ancla, es la
custodia comprobada de la clave).

## 5. Lo que este diseño NO toca — con testigos

- **Ningún circuito** (B10.6/B10.7, «ninguna toca circuito», §174): el
  ancla es composición de digests, no restricciones.
- **El cable**: `zkssl/0.4` quieto. El ancla se deriva de lo que
  `zkssl_signedEpochHead` ya sirve; ningún método nuevo hace falta.
- **La cabeza y su firma**: nada nuevo entra en el digest ni en el
  preámbulo; `VERSION_FORMATO` no se mueve. Cero índices XMSS extra.
- **El latido** (§115): la cadencia del ancla es un múltiplo del latido,
  no un reloj segundo.
- **El acuse y el recibo** (§121, RFC-0010): el ancla no los sustituye —
  los vuelve oponibles ante quien no custodió cabezas—.

## 6. Lo que NO resuelve

- **El medio externo es confianza DESPLAZADA, no eliminada**: su
  append-only, su disponibilidad y sus fechas son suyos. Si el ancla se
  vuelve dependencia de liveness del sistema, **va declarada como tal**
  (entrada 86); en este diseño no lo es: sin medio, el sistema opera y el
  ancla simplemente no existe, que es visible.
- **La cola entre anclas sigue acotada por `M` y N2, no cerrada** (§3).
- **El ancla prueba QUÉ raíz era canónica, no que su contenido fuera
  completo**: la completitud sigue siendo de los recibos y la
  no-inclusión (§121.3, RFC-0010). Un operador puede anclar
  puntualmente una historia que censura; lo que ya no puede es contarle
  otra a quien compruebe el ancla.

## 7. Coste y primer paso medible

Un formato (el ancla y su huella, con dominio propio), un verificador en
cliente sobre piezas existentes (la firma, el MMR), y la política `M`
declarada. **Primer paso: B10.6** —el formato del ancla y `M`—, después
**B10.7** —el verificador—. Elegir y medir el medio queda para cuando
haya operador real, y `SECURITY.md` §2.bis lo dice: **pendiente de
despliegue**. El ancla encarece al último intermediario; no lo elimina
(`SECURITY.md` §6).

⚠️ **Ejecutado (§591–§593, sesión 194)**: B10.6 y B10.7 son el
**RFC-0012** (`spec/rfc/0012-el-ancla-de-cabezas.md`): el dominio
`ANCLA_V1` y las composiciones en el núcleo, el sobre `tipo: "ancla"` del
mando con la vista dividida dentro, su banco y su catálogo. El despliegue
—elegir medio, publicar con cadencia `M`, medir el coste— sigue
pendiente, y la entrada 86 sigue abierta por eso.

## 9. NOTA de reconstrucción (§590)

El original de este documento se escribió en el §174 (2026-08-05) y no
sobrevivió a la publicación del árbol: el S548 (2026-09-25), el primer
commit del repositorio público, lo llevaba como marcador de posición de
una línea, y ningún asiento lo advirtió. Esta reconstrucción (sesión 194)
se escribió DESDE el acta del §174 y las citas vivas del árbol —
`SECURITY.md` §2.bis y §6, `CONFIANZA_RESIDUAL.md` B10.6/B10.7,
`CADUCIDAD_PENDIENTE.md` (`M` como línea sistémica), las entradas 70, 83
y 86 del BACKLOG, `doc/DIAGNOSTICO_ESCALADO.md` §4— y no afirma nada que
esas fuentes no carguen. Si el original aparece, manda el original y esta
nota registra la divergencia.
