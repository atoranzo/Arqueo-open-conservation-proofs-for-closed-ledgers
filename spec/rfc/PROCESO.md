# Proceso RFC del protocolo ZK-SSL

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
   gasto no viaja jamás**. Un RFC que lo erosione nace RETIRADO.
4. ACEPTADO exige: la spec actualizada + OpenRPC regenerado + vectores
   re-emitidos (o nuevos bajo la versión nueva) + suites verdes.
5. El asiento de AUDITORIA.md que selle el cambio referencia el RFC por
   número; el RFC referencia el asiento. Doble hilo, como todo aquí.
