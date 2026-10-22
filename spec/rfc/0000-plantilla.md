# RFC-0000 — <título corto>

- **Estado:** BORRADOR
- **Autor:** <quién>
- **Asistencia GenAI:** <«ninguna», o «Claude, en una sesión de Claude Code en la nube», con los
  asientos y lo que hizo; nunca el identificador de un modelo> — ver [`GENAI.md`](../../GENAI.md)
  ⚠️ §856: decía «<modelo y versión, o «ninguna»>». Seguido al pie de la letra, ponía el
  identificador de un modelo, que `GENAI.md` no da y que el ATADO G de `tools/check_publicadas.py`
  rechaza en toda la cabecera. Esta marca no pasa al RFC nuevo.
- **Fecha:** <aaaa-mm-dd>
- **Versión del protocolo afectada:** zkssl/0.1
- **Asiento(s) de AUDITORIA:** <## NNN. al sellar>

## Motivación

Qué problema real resuelve. Con medida si la hay.

## Diseño

El cambio exacto: métodos, tipos de cable, errores. Diffs de
`spec/RPC.md` si aplica.

## Compatibilidad

¿Rompe el cable? Si sí: nueva versión, y qué pasa con los vectores.

## Seguridad

Efecto sobre el principio del API (la clave de gasto no viaja) y sobre
las deudas declaradas (aviso fuera de banda §21, nodo único, --dev).

## Referencias

Asientos, notas del BACKLOG, código.
