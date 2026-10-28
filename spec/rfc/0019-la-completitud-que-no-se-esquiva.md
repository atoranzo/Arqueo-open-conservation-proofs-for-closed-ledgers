# RFC-0019 — La completitud que no se esquiva: la ventana con el índice acreditado, y el `StaleState` que no resuelve solo

- **Estado:** PROPUESTO (§675), con sus cuatro decisiones tomadas (§679, §681), E1 y E2 construidas
  (§681, §682) y la primera pieza de E3, el núcleo, construida (§773).
  Enmienda el RFC-0010 (ACEPTADO); el paso a ACEPTADO exige construir sus etapas (regla 4
  del PROCESO). ⚠️ §826: decía «exige decidirlas y construir sus etapas»: las cuatro decisiones están tomadas (§679, §681). Sus etapas, con la superficie que rompe cada una, en «Estado de las etapas», antes de «Seguridad» (§826).
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude, en una sesión de Claude Code en la nube (§675): redactó el borrador
  sobre lo que midió el re-triaje del segundo enjambre del plano v2.0, fuera del paso 4 de
  `GENAI.md` — ver [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-10-02
- **Versión del protocolo afectada:** `zkssl/0.4`. E1 no cambia el cable; cambia un veredicto del kit.
  E2 cambia el veredicto de dos vectores publicados (⚠️ §826: decía «de un vector publicado»). E3, la hoja del recibo, sí cambia el cable (su
  núcleo entra en el §773 sin cambiarlo).
- **Asiento(s) de AUDITORIA:** §675 (este borrador), §679 (las decisiones), §681 (la D-A y E1), §682
  (E2), §707 (lo que ata `digest_pi`, por la D-G del RFC-0018), §773 (E3, primera pieza: el núcleo), §815 (la versión fuerte de `ProofFailed` y `VerificationFailed`, que la E3 lleva también), §826 (la tabla de etapas y la cabecera, corregida). ⚠️ §826: acababa en el §773, y callaba el §815, que tocó este fichero con su párrafo de la E3.

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

**E2, construida (§682).** `stale_no_resuelve`, preguntado en el brazo del rechazo directo y en el
del lote, después de sus propias comprobaciones: un sobre mal formado sigue siendo ROJO por lo suyo.
Cambian exactamente los dos vectores que este RFC nombraba; se mueven enteros a
`spec/vectors/0.4/completitud/` con su manifiesto de entonces, que el kit `0.4.2` publicado cumple, y
los mismos bytes viven en `completitud/` como `rechazo-stale-declarada.json` y
`lote-rechazado-stale-declarada.json`, con el cuarto estado.

**E3 — La hoja del recibo lleva el digest de las entradas públicas.** El recibo de recepción gana
`digest_pi`, el digest de las entradas públicas de la prueba recibida, bajo la firma. Un rechazo
`StaleState` se ata entonces por la firma y no por el `data`, y E2 puede volver a dar VERDE cuando la
atadura sea verificable. Cambia la hoja del recibo, así que va al tren `zkssl/0.5` (RFC-0018).
(§707: `digest_pi` ata también la familia y la versión de la AIR de la prueba recibida, para que un
recibo diga de qué prueba es; lo trae la D-G del RFC-0018, en estado de propuesta, y la forma exacta
la fija esta etapa.)

**E3, primera pieza (§773): el núcleo.** `digest_pi(familia, version_air, entradas)` es
Blake3(`ZK-SSL-public-inputs-v1` ‖ longitud ‖ familia u16 LE ‖ versión de la AIR u16 LE ‖ los
elementos de las entradas públicas, en el orden de `to_elements()` y ocho bytes LE cada uno), reducido
limbo a limbo, el molde del lote; y `recibo_digest_v2(hash_prueba, digest_pi, era, n)` es la hoja v1
con `digest_pi` detrás de `hash_prueba`, bajo su propio dominio, `RECEP_V2`. Las dos tienen KAT, y la
segunda implementación las reproduce (`spec/NUCLEO.md`, sección 6). Nada las consume todavía: el nodo
anotando la v2, el cable, el kit en 2.11 y la segunda en su verificador van con el corte, y los
números de familia y versión los da el registro de la D-G del RFC-0018. Cambiar esta forma, una vez
publicada, es otra versión de la hoja, no reescribirla.

⚠️ **§815: E3 lleva también la versión fuerte de `ProofFailed` y `VerificationFailed`**, por decisión
del autor. Desde el kit 0.4.3 las dos se declaran —el cuarto estado, salida 3—, como pide la D-I
del RFC-0007, y siguen sin probarse (`SECURITY.md` §3.15). Con `hash_prueba` y `digest_pi` bajo la
firma, quien tenga la prueba la presenta, el kit la re-verifica con esas entradas y el rechazo se
prueba o se refuta: la objeción de la D-I —el mando verificaría unos bytes que el nodo eligió—
deja de valer, porque los ata la hoja que el nodo firmó. Hasta el corte, se declaran.

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

## Estado de las etapas

⚠️ §826: el RFC no tenía esta tabla, y la R5 del RFC-0021 la pide un sello antes del corte, diciendo
qué superficie de su D-A rompe cada etapa: S1, el cable; S2, las pruebas que viajan; S3, lo firmado;
S4, los sobres del kit; S5, los vectores; S6, el kit como binario. Va aquí, y no tras la cabecera,
para no mover las líneas de este RFC que otros documentos citan.

| etapa | qué entrega | ¿rompe el cable?, y qué superficie | estado |
|---|---|---|---|
| E1 — la ventana con el índice acreditado | `indice_embebido` e `indice_acreditado` en el mando del kit, y un vector nuevo, `completitud/neg-vigente-index-inflado.json` | no rompe el cable; en S4 cambia el veredicto de un sobre de completitud cuyos índices declarados se apartan de los acreditados —la sonda H10 daba NO RESUELTA y da ventana ABIERTA—, y el de ninguno de los 74 vectores publicados de la familia (§681); en S6 no rompe, porque cambia el texto y no la causa: de los vectores que viajan en el tarball de la 0.4.2, cuatro dan con la 0.4.3 el mismo 1 con otro texto, porque la ventana se nombra con el índice que la firma acredita y su inicio (medido en el §815, `doc/KIT.md` §0), y la línea de su manifiesto, que pina la causa (`spec/PAQUETE.md` §6.1), es la misma en `a4e888d` que en el árbol | construida — §681 |
| E2 — `StaleState` no resuelve solo | `stale_no_resuelve`, en el brazo del rechazo directo y en el del lote: el cuarto estado del RFC-0010, salida 3 | no rompe el cable; en S4, dos sobres publicados pasan de VERDE a «DECLARADA, NO PROBADA»; en S5 no rompe: sus dos vectores se mudaron con sus bytes a `spec/vectors/0.4/completitud/` (R6 b), y los mismos bytes viven en `completitud/` con otro nombre | construida — §682 |
| E3, primera pieza — el núcleo | `digest_pi` y `recibo_digest_v2`, bajo el dominio `RECEP_V2`, con sus KAT y su réplica en `tools/segunda`; nada los consume | no: dos funciones y sus KAT, que nadie emite todavía (R2 del RFC-0021) | construida — §773 |
| E3, el resto — la hoja v2, en uso | el nodo anota la hoja v2, el cable la sirve, el kit la lee en la forma 2.11 y la segunda en su verificador, con los números de familia y de versión del registro de la D-G del RFC-0018; y, por decisión del autor en el §815, la versión fuerte de `ProofFailed` y `VerificationFailed`: el kit re-verifica la prueba que se le presente con `hash_prueba` y `digest_pi` bajo la firma. Para eso tiene que verificar pruebas de la capa, como las de envío y cobro, a las que el nodo da recibo (`SECURITY.md` §3.15), y hoy no verifica pruebas de envío, cobro, quema ni umbral (`spec/rfc/0018-el-tren-0-5.md:110`): la «Compatibilidad» del RFC-0017 da por hecho que el kit no juzga las de la capa, y eso cambia con esta pieza, en el mismo corte | **sí**: S3, la hoja del recibo, que compone bajo la `recepRoot` que la cabeza firma; S1, el recibo del cable; y S4, la forma 2.11 del sobre, y el rechazo por `ProofFailed` o `VerificationFailed`, que desde la 0.4.3, la vigente (§818), se declara —la `declarada` sale 3, y su sobre de rechazo, ROJO (`doc/KIT.md` §0)— y pasa a probarse o refutarse ⚠️ §726: la 0.4.3 fue la vigente hasta el §726; la vigente es la 0.4.4, que también declara las dos causas: sus tres vectores de `completitud/` dan 3 con el binario publicado | sin construir; en el sello del corte `zkssl/0.5` (D-D) |

La E1 y la E2 cambian veredictos del kit (S4) y se construyeron antes de que la política del RFC-0021
rigiera. La `arqueo-verify-v0.4.2` no las lleva: se construyó desde `a4e888d`, el §670, anterior a
las dos, y la registró el §671. Las lleva la 0.4.3, que el §815 cortó como arreglo de seguridad
dentro de `0.4.x` (R6 f), con la lista de lo que cambia, la E1 y la E2 entre ello, en la sección 0
de `doc/KIT.md`. El autor la publicó el 21-10-2026 con su aviso, `GHSA-6p4m-qg2w-8r6j`, producida
sobre `5fd5212`, y desde el §818, que la registra con su huella, es la vigente (`doc/KIT.md` §0). ⚠️ §726: lo fue hasta el §726, que registra la 0.4.4, la vigente desde entonces; la 0.4.4 lleva también la E1 y la E2.
Sus commits, `41f765d` (§681) y `b0b6770` (§682), salen en
`git log a4e888d..5fd5212 -- crates/zk-ssl-verify/src`, de la 0.4.2 al commit de la 0.4.3.

## Seguridad

La clave de gasto no viaja y este RFC no la toca. E1 quita un ROJO fabricable contra el operador; E2
quita un VERDE fabricable por el operador; E3 cierra el residuo que E2 nombra.

## Referencias

RFC-0010 (D-D y su corrección del §567, D-F, D-G, E4); `AUDITORIA.md` §332, §567, §571, §675;
`crates/zk-ssl-verify/src/main.rs` (`verificar_completitud`, `resolver_por_rechazo`,
`exige_mismo_recibo`).
