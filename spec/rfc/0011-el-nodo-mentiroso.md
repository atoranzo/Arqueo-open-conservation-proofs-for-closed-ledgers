# RFC-0011 — El nodo mentiroso: las defensas, ejercitadas contra quien miente de verdad

- **Estado:** PROPUESTO (§588), con sus decisiones TOMADAS en el §598 -delegadas por el autor y
  REVERSIBLES, ver «Decisiones»-. Sin código todavía: las etapas se abren en el orden del §598.
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude (sesión 193, §588 y §598) — ver [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-09-30
- **Versión del protocolo afectada:** `zkssl/0.4` — **no sube**: nada de esto toca el cable.
- **Asiento(s) de AUDITORIA:** §588 (la propuesta); §598 (las decisiones).

## Motivación

La entrada 93 del `BACKLOG`: el proyecto tiene defensas contra un operador que miente, y
ninguna ha visto nunca a un operador que mienta. Se ejercitan contra datos FABRICADOS en un test o
contra un nodo HONESTO; «alcanzable, no ejercitada» es lo más que pueden decir. Medido en el §588:

| la mentira | la defensa que debería verla | cómo se ejercita hoy |
|---|---|---|
| dos cabezas distintas firmadas con el mismo índice (vista dividida) | el testigo, `--comparar` y `--auditar` («el mismo índice con dos digests», `witness.rs`), y dos cofirmas de un testigo con el mismo índice | diarios fabricados en tests; ningún nodo real la produce |
| una cabeza firmada que su diario no recoge | el testigo, `--ausentes` (§283) | nunca ROJO en un banco |
| un recibo emitido que no se resuelve | el sobre de completitud, «NO RESUELTA EN LA VENTANA» (RFC-0010 E4) | el banco arma el sobre sin resolución contra un nodo honesto |
| una operación censurada ANTES de emitir recibo | ninguna, por diseño: el residuo D-H del RFC-0010 | declarada, no medida |

La última fila no es un hueco que este RFC cierre: es un residuo que ejercitar para MEDIR su
silencio. Un banco que censura sin recibo y comprueba que todas las defensas siguen VERDES convierte
«no detectable» de afirmación en medida.

⚠️ Ningún auto-informe del nodo protege de nada: «yo soy honesto» es el operador hablando de sí
mismo. Lo que se construye aquí es instrumentación, no un modelo de confianza.

## Diseño

**D-A. La mentira vive en un crate APARTE, del que el nodo no depende.** Es la más fuerte de las
tres garantías que la entrada 93 ordena: un crate aparte; una `feature` no-default, que vive en
el mismo crate y que `--all-features` compila; y una bandera en tiempo de ejecución, la más débil.
Las dos últimas se DESCARTAN: un binario de producción que lleva la mentira dentro, apagada, es un
binario que puede mentir. Precedente medido: `zk-core` depende de `ceremony` sólo como
dev-dependency, a propósito, para el grafo acíclico. Nombre propuesto: `zk-ssl-mentiroso`
(REVERSIBLE).

**D-B. Una costura, no una rama.** Las mentiras que interesan -la vista dividida, la firma que el
diario no recoge- exigen la CLAVE: un proxy delante del nodo sólo puede mentir por omisión. Así
que el nodo expone un punto de extensión -quién firma y quién anota-, el binario de producción
construye sólo la implementación honesta, y la mentirosa vive en `zk-ssl-mentiroso` (el patrón
«envoltorio en vez de cambio de firma», §281). ⚠️ **El coste, medido:** `zk-ssl-node` es hoy SÓLO
binario, sin `[lib]`; para que otro crate reutilice su código, el nodo gana primero una
biblioteca, y su `main.rs` pasa a ser el envoltorio fino del binario honesto. Es el corte más caro
del RFC y va solo, en su etapa.

**D-C. Lo que lo hace COMPROBABLE y no prometido: una compuerta del canon.** El árbol de
dependencias NORMALES de `zk-ssl-node` no contiene `zk-ssl-mentiroso`; sólo puede aparecer como
dev-dependency o no aparecer. La compuerta se ensaya con su falsador -el crate añadido como
dependencia normal pone el canon ROJO- antes de que exista una sola mentira.

**D-D. Cada mentira se ejercita en un banco que espera ROJO de la defensa.** El éxito de un banco
del mentiroso es que la defensa DISPARE, nombrando lo que vio; un banco del mentiroso que sale VERDE
sin que la defensa hable es un fallo. La única excepción es la del residuo D-H, cuyo banco asierta
el SILENCIO de todas las defensas y lo dice.

**D-E. Claves de prueba, siempre.** El mentiroso firma con semillas deterministas de la suite,
como `dev_openSeeded` y los custodios de PRUEBA; nunca lee `--clave-fichero` ni un keystore.

## Etapas

- **E1 — la compuerta y el crate vacío.** Nace `zk-ssl-mentiroso` sin código, con su fila en el
  canon a pin cero; nace la compuerta D-C, con su falsador ensayado.
- **E2 — el proxy: mentir por omisión.** Sin tocar el nodo. Censurar antes del recibo (el residuo
  D-H, cuyo banco asierta el silencio) y tragarse una respuesta.
- **E3 — la biblioteca del nodo.** El corte de D-B, sin cambiar comportamiento: la suite del nodo y
  los bancos, iguales antes y después.
- **E4 — la vista dividida.** Dos cabezas con el mismo índice: `--comparar` y `--auditar` ROJOS en
  un banco, nombrando el índice.
- **E5 — la firma sin anotar.** `--ausentes` ROJO en un banco por primera vez.

⚠️ **Esta lista la SUSTITUYE la del §598**, en «Decisiones»: sin crate ni biblioteca, las etapas
E1 y E3 de arriba desaparecen, y el orden cambia. Se conserva como lo que se propuso.

## Compatibilidad

Ninguna sobre el cable ni sobre los vectores. El workspace gana un miembro, con su fila en el
canon, y en E3 el nodo gana una biblioteca: el binario `zk-ssl-node` y su comportamiento no cambian,
y lo prueban su suite y los bancos.

## Seguridad

El principio del API -la clave de gasto no viaja- no se toca. El riesgo propio de este RFC es que
la mentira llegue a producción, y D-A y D-C son la respuesta: no hay `feature` ni bandera que la
encienda en el binario del nodo, y el canon comprueba el árbol de dependencias en cada sello. Lo que
NO cubre: un operador que compile su propio binario puede mentir igual; esto no lo impide ni lo
pretende, sólo hace que las defensas se prueben contra lo que él haría.

## Decisiones

Las tres preguntas que el §588 dejó al autor, tal como se formularon:

- **D-A**, el nombre del crate: `zk-ssl-mentiroso` (REVERSIBLE).
- **D-B**, darle biblioteca al nodo en E3, o limitar el RFC a las mentiras por omisión (E1 y E2) y
  dejar la vista dividida y la firma sin anotar en «alcanzable, no ejercitada».
- **El orden** E1 a E5: la compuerta antes que ninguna mentira, y la omisión antes que la clave.

**TOMADAS en el §598**, por delegación del autor («HACER: tu decisión») y con los principios de la
casa -medir antes de construir, lo más barato que responde, no tocar el binario de producción sin
necesidad-. Todas REVERSIBLES.

**Lo que se midió antes de decidir.** El banco del ancla (`tools/banco_ancla.sh`, §593) ya produce
una vista dividida DE VERDAD con el binario del nodo SIN TOCAR: su corrida B levanta la misma
semilla con directorios frescos, el contador de índice vuelve a empezar, y el nodo firma dos cabezas con el
mismo índice y digests distintos. Las mentiras «que exigen la clave» (D-B) exigen la CLAVE, no un
código distinto: cualquier operador con la semilla las produce con el binario honesto. Eso es,
además, justo lo que las defensas tienen que ver.

- **D-B: NO se le da biblioteca al nodo.** No hace falta para ninguna de las mentiras con clave
  conocidas, y partir `main.rs` sería el corte más caro del RFC sin nada que lo pida. Se reabre si
  aparece una mentira que sólo un código modificado produce; la candidata es la última etapa.
- **D-A: tampoco hay crate, por ahora.** Las mentiras viven en `tools/banco_mentiroso_*.sh` -bash y
  python, como los demás bancos-, fuera del workspace de Cargo: nada puede enlazarlas, que es más
  fuerte que el crate aparte que D-A proponía. El nombre `zk-ssl-mentiroso` queda reservado por si
  una etapa necesita un crate. Consecuencia: la compuerta D-C no tiene árbol que vigilar y no nace;
  nacería con el crate. Los bancos los vigila `--bancos` (§582), como a todos.
- **El orden, rehecho**, de lo que la casa ya sabe producir a lo que está por medir:
  - **E1 — la vista dividida contra el TESTIGO.** El método del banco del ancla: la misma semilla
    con un contador fresco. Éxito: `--comparar` y `--auditar` ROJOS, nombrando el índice.
  - **E2 — la firma que el diario no recoge.** El operador firma con un `--diario` y enseña otro.
    Éxito: `--ausentes` ROJO en un banco por primera vez. Que se pueda sin código se mide en ella.
  - **E3 — la censura antes del recibo.** Un proxy que reenvía y descarta, escrito dentro del banco.
    Éxito: el SILENCIO de todas las defensas, asertado: el residuo D-H del RFC-0010, medido.
  - **E4 — el recibo que no se resuelve, por un nodo de verdad.** Cómo producirlo sin código -parar
    el nodo entre el recibo y la resolución- está por medir. Si sólo se puede con código, D-B se
    reabre aquí.

## Referencias

`BACKLOG.md`, entrada 93 (el diseño de partida) y entrada 94 (lo que el canon no ve); §281 (el
envoltorio); §283 (`--ausentes`); §285 («quien firma, anota»); RFC-0010, D-H y E4;
`crates/zk-ssl-cli/src/witness.rs` (`--comparar`, `--auditar`, `--ausentes`).
