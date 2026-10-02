# RFC-0019 — La completitud que no se esquiva: la ventana con el índice acreditado, y el `StaleState` que no resuelve solo

- **Estado:** PROPUESTO (§675), con sus cuatro decisiones tomadas (§679, §681) y E1 construida (§681).
  Enmienda el RFC-0010 (ACEPTADO); el paso a ACEPTADO exige decidirlas y construir sus etapas (regla 4
  del PROCESO).
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude, en una sesión de Claude Code en la nube (§675): redactó el borrador
  sobre lo que midió el re-triaje del segundo enjambre del plano v2.0, fuera del paso 4 de
  `GENAI.md` — ver [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-10-02
- **Versión del protocolo afectada:** `zkssl/0.4`. E1 no cambia el cable; cambia un veredicto del kit.
  E2 cambia el veredicto de un vector publicado. E3, la hoja del recibo, sí cambia el cable.
- **Asiento(s) de AUDITORIA:** §675 (este borrador), §679 (las decisiones), §681 (la D-A y E1).

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

**E1, construida (§681).** `indice_embebido` e `indice_acreditado` en el mando del kit; la ventana
empieza en el embebido del cierre y `S` es el acreditado de la vigente o de la cabeza de la
resolución (acuse, rechazo, lote y prenda). Los 74 vectores publicados del sobre de completitud dicen
lo mismo; uno nuevo, `neg-vigente-index-inflado.json`, es la sonda H10: la vigente con el `index`
inflado a 1444 daba NO RESUELTA y da ventana ABIERTA.

**E2 — `StaleState` no resuelve solo.** Una resolución por rechazo cuya causa sea `StaleState` sale
como el cuarto estado del RFC-0010 D-G, «DECLARADA, NO PROBADA» (salida 3), y no como VERDE: la causa
se sostiene sobre el estado comprometido, pero su atadura a ESTA operación es la palabra del nodo.

**E3 — La hoja del recibo lleva el digest de las entradas públicas.** El recibo de recepción gana
`digest_pi`, el digest de las entradas públicas de la prueba recibida, bajo la firma. Un rechazo
`StaleState` se ata entonces por la firma y no por el `data`, y E2 puede volver a dar VERDE cuando la
atadura sea verificable. Cambia la hoja del recibo, así que va al tren `zkssl/0.5` (RFC-0018).

## Compatibilidad

E1 no cambia el cable. Cambia el veredicto del kit solo cuando los índices declarados se apartan de
los acreditados. ⚠️ La frase de este borrador decía «que un nodo honesto no firma»: no es cierto, un
nodo honesto con índices huérfanos los firma (§332), y el declarado de cualquier cabeza lo puede
cambiar quien arma el sobre, porque no va firmado. Medido (§681): ninguno de los 74 vectores
publicados cambia de veredicto.

E2 cambia el veredicto de vectores publicados: `completitud/resuelta-por-rechazo.json` y
`completitud/lote-rechazado-con-prueba.json` se resuelven por `StaleState` y sus manifiestos exigen
VERDE. Los vectores no se reescriben: los manifiestos pasan a ser por versión del kit, y los de `0.4`
se conservan con su veredicto de entonces.

E3 rompe el cable: `zkssl/0.5`.

## Decisiones (del autor, §679)

El autor las decidió en la sesión del §676 al §678, «como recomienda» la sesión; la D-A, cuando la
sesión le dijo que lo recomendado no se sostenía (§681).

- **D-A — el índice acreditado, y la D-D del RFC-0010 enmendada (§681).** La sesión recomendó primero
  exigir `declarado == embebido + 1` y rechazar la cabeza que no lo cumpla. No se sostenía: el §332 y
  el §399 lo descartaron por escrito (`crates/zk-ssl-verify/src/lib.rs`, `verificar_cabeza` y
  `verificar_cofirma`): `GuardianIndice::reservar` persiste el índice ANTES de firmar, así que un
  proceso muerto entre la reserva y la firma deja un índice huérfano y exigir el +1 daría ROJO sobre
  cabezas legítimas. El autor eligió la otra rama: medir con el acreditado y enmendar la D-D. Al
  construirla, la sesión vio que medir con el acreditado solo la cabeza que mide dejaba otro agujero:
  la `era` de la hoja la cuenta el nodo con su contador (`anotar`: el declarado más uno), y en esa
  unidad un índice quemado sin firmar —gratis para el operador— retrasaría todas las ventanas. Lo
  construido mide los dos extremos en la unidad de la firma: el inicio es el embebido del cierre y
  `S` el acreditado de la cabeza que mide. La enmienda está en la D-D del RFC-0010.
- **D-B — el cuarto estado.** E2 sale como «DECLARADA, NO PROBADA» (salida 3), no como VERDE con una
  advertencia, que se acabaría leyendo como VERDE.
- **D-C — vectores nuevos, con nombre nuevo.** Los dos vectores publicados se quedan con su manifiesto
  y su veredicto de `0.4`; los de E2 nacen con otro nombre. Ningún vector se reescribe.
- **D-D — E3 dentro del tren del RFC-0018** (su D-D).

## Seguridad

La clave de gasto no viaja y este RFC no la toca. E1 quita un ROJO fabricable contra el operador; E2
quita un VERDE fabricable por el operador; E3 cierra el residuo que E2 nombra.

## Referencias

RFC-0010 (D-D y su corrección del §567, D-F, D-G, E4); `AUDITORIA.md` §332, §567, §571, §675;
`crates/zk-ssl-verify/src/main.rs` (`verificar_completitud`, `resolver_por_rechazo`,
`exige_mismo_recibo`).
