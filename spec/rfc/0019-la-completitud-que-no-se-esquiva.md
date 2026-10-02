# RFC-0019 — La completitud que no se esquiva: la ventana con el índice acreditado, y el `StaleState` que no resuelve solo

- **Estado:** PROPUESTO (§675), con sus cuatro decisiones abiertas y nada construido, como entró el
  RFC-0015. Enmienda el RFC-0010 (ACEPTADO); el paso a ACEPTADO exige decidirlas y construir sus
  etapas (regla 4 del PROCESO).
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude, en una sesión de Claude Code en la nube (§675): redactó el borrador
  sobre lo que midió el re-triaje del segundo enjambre del plano v2.0, fuera del paso 4 de
  `GENAI.md` — ver [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-10-02
- **Versión del protocolo afectada:** `zkssl/0.4`. E1 no cambia el cable; cambia un veredicto del kit.
  E2 cambia el veredicto de un vector publicado. E3, la hoja del recibo, sí cambia el cable.
- **Asiento(s) de AUDITORIA:** §675 (este borrador).

## Motivación

El sobre de completitud (RFC-0010) dice, sin el nodo, si una operación que el nodo recibió bajo su
firma se resolvió dentro de su ventana. El re-triaje del segundo enjambre midió dos maneras de que el
kit diga algo falso, en direcciones opuestas.

1. **La ventana se mide con el índice DECLARADO.** El kit decide «NO RESUELTA EN LA VENTANA» comparando
   la era del recibo con el `index` declarado de la cabeza vigente. La firma XMSS acredita su índice
   embebido, y la única regla entre los dos es `embebido < declarado` (§332): el declarado no tiene
   cota superior. Una cabeza firmada con el `index` inflado convierte una era todavía abierta en un
   ROJO portable contra el operador (medido: índice 1444 en una cabeza firmada con el 1).
2. **Un `StaleState` resuelve sin estar atado a la operación.** Una resolución por rechazo se ata al
   recibo por el `hashPrueba` del `data` del rechazo, que no va firmado (el kit lo llama «la palabra
   del nodo»), y la causa `StaleState` se sostiene con un `recibo` que tampoco lo va. El operador
   acusado puede responder con un `StaleState` inventado y el kit dice VERDE. El RFC-0010 lo declara
   en parte (E4, la palabra del nodo); el veredicto sigue siendo VERDE.

## Diseño

**E1 — La ventana con el índice acreditado.** En la ventana del sobre de completitud, en sus
resoluciones y en la prenda, el kit usa `embebido + 1`, el índice que la firma acredita, en lugar del
declarado. Un solo lector de cabezas devuelve `(declarado, embebido, acreditado)`. No cambia el cable
ni ningún byte firmado.

**E2 — `StaleState` no resuelve solo.** Una resolución por rechazo cuya causa sea `StaleState` sale
como el cuarto estado del RFC-0010 D-G, «DECLARADA, NO PROBADA» (salida 3), y no como VERDE: la causa
se sostiene sobre el estado comprometido, pero su atadura a ESTA operación es la palabra del nodo.

**E3 — La hoja del recibo lleva el digest de las entradas públicas.** El recibo de recepción gana
`digest_pi`, el digest de las entradas públicas de la prueba recibida, bajo la firma. Un rechazo
`StaleState` se ata entonces por la firma y no por el `data`, y E2 puede volver a dar VERDE cuando la
atadura sea verificable. Cambia la hoja del recibo, así que va al tren `zkssl/0.5` (RFC-0018).

## Compatibilidad

E1 no cambia el cable. Cambia el veredicto del kit solo ante cabezas con el `index` declarado por
encima del acreditado, que un nodo honesto no firma.

E2 cambia el veredicto de vectores publicados: `completitud/resuelta-por-rechazo.json` y
`completitud/lote-rechazado-con-prueba.json` se resuelven por `StaleState` y sus manifiestos exigen
VERDE. Los vectores no se reescriben: los manifiestos pasan a ser por versión del kit, y los de `0.4`
se conservan con su veredicto de entonces.

E3 rompe el cable: `zkssl/0.5`.

## Decisiones abiertas (del autor)

- **D-A.** E1 choca con la frase de la D-D del RFC-0010 (§567) de que los índices huérfanos «cuentan en
  contra de quien los quemó»: con el acreditado, un índice quemado sin firma deja de acortar la
  ventana. ¿Se enmienda la D-D así, o se exige `declarado == embebido + 1` y se rechaza la cabeza que
  no lo cumpla?
- **D-B.** ¿E2 como cuarto estado (recomendado), o VERDE con una advertencia explícita en la salida?
- **D-C.** ¿Manifiestos por versión del kit para los dos vectores de E2, o vectores nuevos bajo un
  nombre nuevo?
- **D-D.** ¿E3 dentro del RFC-0018 o en su propio tren?

## Seguridad

La clave de gasto no viaja y este RFC no la toca. E1 quita un ROJO fabricable contra el operador; E2
quita un VERDE fabricable por el operador; E3 cierra el residuo que E2 nombra.

## Referencias

RFC-0010 (D-D y su corrección del §567, D-F, D-G, E4); `AUDITORIA.md` §332, §567, §571, §675;
`crates/zk-ssl-verify/src/main.rs` (`verificar_completitud`, `resolver_por_rechazo`,
`exige_mismo_recibo`).
