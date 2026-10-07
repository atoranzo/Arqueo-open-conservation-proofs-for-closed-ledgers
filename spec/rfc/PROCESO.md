# Proceso RFC del protocolo de Arqueo (`zkssl/0.N`)

Un cambio al PROTOCOLO (lo que cruza el cable: `spec/RPC.md`,
`spec/openrpc.json`, los vectores de `spec/vectors/`) no entra por
commit directo: entra por RFC. Este proceso es deliberadamente pequeño —
lo que importa es que quede escrito, numerado y decidido.

## Estados

BORRADOR → PROPUESTO → ACEPTADO → FINAL · o RETIRADO en cualquier punto.

## Reglas

1. Un fichero por RFC: `spec/rfc/NNNN-titulo-corto.md`, numeración
   correlativa desde 0001 (la 0000 es la plantilla).
2. Todo RFC declara COMPATIBILIDAD: si rompe el cable, la versión sube
   (`zkssl/0.1` → `zkssl/0.2`) y los vectores viejos se conservan bajo
   su versión — jamás se reescriben.
3. Todo RFC declara su efecto sobre el principio del API: **la clave de
   gasto no viaja jamás**. Un RFC que lo erosione nace RETIRADO. Desde el
   §538 (RFC-0009 E3b-2) el probador oculta el testigo en lo que la suite de
   E2 mide: cero literales en las 23 pruebas con fila; entre el §521 y el
   §538 no se cumplía, y los asientos lo dicen.
4. ACEPTADO exige: la spec actualizada + OpenRPC regenerado + vectores
   re-emitidos (o nuevos bajo la versión nueva) + suites verdes.
5. El asiento de AUDITORIA.md que selle el cambio referencia el RFC por
   número; el RFC referencia el asiento. Doble hilo, como todo aquí.

## Fallos de seguridad

La apertura y los estados suponen que el RFC se discute en el árbol
antes que el cambio. Con un fallo de seguridad —lo que `SECURITY.md` §5
pide reportar en privado— eso publicaría el fallo antes que su arreglo.
Dos cláusulas lo resuelven. Ninguna afloja las cinco reglas.

- **El arreglo propio se redacta en privado y se publica de una vez.**
  Si el arreglo toca el protocolo, su RFC —o la enmienda al RFC que el
  fallo toca— no pasa por el árbol como BORRADOR ni como PROPUESTO: se
  redacta fuera, con el código, y entra en el mismo sello que el arreglo,
  con su etapa ya construida. Toma el número libre al entrar: un hueco en
  la numeración contaría lo que se calla. Las reglas valen igual: si
  rompe el cable, la versión sube y los vectores viejos se conservan bajo
  la suya (regla 2); un RFC nuevo entra ACEPTADO sólo si trae lo que pide
  la regla 4; y el asiento que lo sella dice que se redactó en privado.
  Si el fallo alcanza al kit publicado, con él salen la release que lo
  arregla y el aviso de seguridad (`SECURITY.md` §5).
- **El borrador del reporte a un tercero no entra en el árbol público.**
  Un fallo que es de otro proyecto se reporta a ese proyecto por su canal
  privado, en divulgación coordinada. El borrador, la correspondencia y
  cualquier reproductor se guardan fuera del árbol público. Lo que el
  árbol puede decir mientras el plazo corre es que hay un reporte en
  curso, sin cómo se dispara el fallo. Rige desde el §691: lo que entró
  antes no lo reescribe esta cláusula; sigue con su asiento, y lo que se
  decida de ello irá en uno nuevo.
