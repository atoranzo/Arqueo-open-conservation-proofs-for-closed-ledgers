# RFC-0015 — El ciclo de vida del firmante: rotar, agotar y perder el índice, con la clave siguiente comprometida

- **Estado:** PROPUESTO (§639) — el texto y sus decisiones, escritos sobre lo medido en el árbol
  (§636, §638) y en el RFC 10033 de la IETF. Las cinco decisiones, TOMADAS en el §642 por
  delegación del autor, con el criterio del §609, y REVERSIBLES: ver «Decisiones». Construida la
  E2, el núcleo del acta (§643), la E3a, el nodo que firma sus actas (§644), la E3b-1, la firma
  de la clave que se va y el techo que no reserva (§645), la E3b-2, `zkssl_keyActs` (§646), y la
  E4, el testigo que rota con las actas (§647), la E5a, los sobres del kit que leen `actas`
  (§648), y la E5b, su catálogo con su banco y la segunda implementación (§649); la E6, sin
  construir.
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude (sesión 193, §639, que lo escribe entero sobre la medida de los
  §636 y §638) — ver [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-10-01
- **Versión del protocolo afectada:** `zkssl/0.4` — **no sube, y la cabeza no se mueve**: nace una
  familia con su dominio y sus vectores (el acta), un método de lectura y un campo opcional en los
  sobres que comparan cabezas. Lo que cambia de semántica es el testigo: hoy se detiene ante
  cualquier cambio de clave, y con la E4 seguiría ante uno que traiga su acta.
- **Asiento(s) de AUDITORIA:** §110.2 (reutilizar el índice filtra la clave), §112 (el
  `KeyExhausted` de upstream, medido), §115 (la cadencia), §245 (el TOFU del testigo), §246 (no
  hay clave que anclar), §288 (la familia 19-84-92), §298 y §328–§337 (el guardián y la
  reconciliación al arrancar), §399 (el índice embebido), §567 (el reloj del recibo es el
  índice), §591 (la huella de la clave), §594 (la puerta del diario en todo estado), §636 y §638
  (lo que hoy pasa al cambiar de clave, medido), §639 (esta propuesta), §642 (las decisiones) y
  §643 (la E2),
  §644 (la E3a), §645 (la E3b-1), §646 (la E3b-2), §647 (la E4), §648 (la E5a) y §649 (la E5b).
- **Backlog:** la **84** (agotamiento, rotación y pérdida del índice), con la **92** (custodia y
  supervivencia del índice) y la **19** en su línea de familia, que el §288 pidió cortar juntas;
  la **87** (agilidad criptográfica: el acta lleva el esquema de la clave que presenta); y la
  **103** (la puerta del diario se apaga quitando un fichero), que esta propuesta no cierra.

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — el ciclo de vida, escrito | este texto | no | **propuesto (§639), decidido (§642)** |
| E2 — el núcleo | `DOMINIO_ACTA`, `acta_digest` y su KAT, y `verificar_acta` en el kit, con la segunda implementación | no (aditivo: una familia nueva) | **construida (§643)**: `DOMINIO_ACTA` (`ACTAS_V1`) y `acta_digest` en `zk-ssl-hash`; el módulo `actas` del kit con el preámbulo (`ZK-SSL-key-act`), el esquema de hoy y `verificar_acta`, con las reglas antes que las firmas; tres KAT —génesis, rotación y preámbulo— que la segunda implementación reproduce, 29 de 29; doce filas de la familia ACTA en `NUCLEO.md` |
| E3a — el nodo firma sus actas | el acta génesis, la rotación a la sucesora comprometida y el aviso de agotamiento, al arrancar | no | **construida (§644)**: opt-in con `--siguiente` (sin ella y sin actas, el nodo firma como hasta hoy); `--huella-de-clave-fichero` imprime la huella de la clave fría; al arrancar, el nodo juzga la cadena de actas del diario, firma la génesis o la rotación por el camino de las cabezas y la anota con `fsync`, y no arranca con una clave que nadie comprometió; el umbral de un año de latidos exige `--reconozco-agotamiento`. La línea del acta no la ve ningún lector de cabezas y sí la puerta del contador |
| E3b-1 — la vieja firma y el techo | la firma de la clave que se va (decisión 5), y el latido que deja de quemar el contador en el techo | no | **construida (§645)**: `--clave-anterior-fichero` da la semilla de la clave que se va, solo en una rotación; la vieja firma el mismo preámbulo en la hoja que da el contador y la nueva empieza en la siguiente, y una semilla que no es la del acta en vigor no gasta nada. En el techo el firmante devuelve `Agotada` **sin reservar**: el contador dice el techo, porque el SK no lo representa (medido) |
| E3b-2 — el cable | `zkssl_keyActs`, que sirve la cadena de actas desde la génesis | no (aditivo: un método) | **construida (§646)**: sin parámetros, `{actas}` desde la génesis, armada una vez al arrancar tras juzgarla; el acta en JSON la escriben y la leen `acta_a_json` y `acta_de_json` del kit, que son también la línea del diario —byte a byte la de antes—; un elemento de un digest que vale `p` o más no se lee. La superficie pasa de 31 a 32 métodos y `zkssl/0.4` no sube |
| E4 — el testigo | ante un cambio de clave pide el acta, la juzga contra la clave que fijó y sigue o se detiene; `--auditar` la juzga en el diario | no | **construida (§647)**: ante un cambio de clave el testigo pide `zkssl_keyActs` y la juzga con `juzgar_rotacion` del kit; si la cadena lleva de la clave fijada a la recibida, anota `rotada` con la cadena dentro, fija la nueva con su tramo y sigue; si no, se detiene como antes, con el motivo. Nace `solapamiento`, que detiene (reglas 3 y 4). `--auditar` rejuzga la rotación desde la línea, sin el nodo. El diario del testigo pasa a v4 |
| E5a — el kit | el campo `actas` en los sobres que comparan cabezas | no (aditivo: un campo opcional) | **construida (§648)**: `juzgar_continuidad` en el kit, un juez para los ocho sitios que exigen la continuidad de un firmante —la extensión, el consumo y las seis cabezas que la completitud compara con su cierre—; el índice EMBEBIDO ordena las dos cabezas, la cadena lleva de la anterior a la posterior (reglas 1 a 3) y la posterior cae en su tramo (regla 4, que nace con nombre: `FueraDeTramo`). Sin `actas`, el texto de `claves_distintas` byte a byte. El conflicto y la vista dividida no la leen, y `spec/PAQUETE.md` dice por qué |
| E5b — el catálogo y el banco | los vectores con `actas`, `tools/banco_rotacion.sh` contra un nodo real y la segunda implementación que los lee | no | **construida (§649)**: `spec/vectors/rotacion/`, la duodécima familia del artefacto, copiada de un nodo real que rota dos veces —A, B con la firma de A, C sin la de B—: tres positivos, las dos conductas del operador que la cadena delata sembradas con sus claves —la vieja que firma después, la nueva que firma antes— y dieciocho negativos por una mutación; la segunda implementación lee `actas` y pasa los 23 con el mismo arnés, y su salida es la del binario línea a línea en los positivos |
| E6 — el medio | el acta como hoja del medio de la clave que se va y de la que llega, con la E3 del RFC-0013 | no | pendiente, tras la E3 del RFC-0013 |

## Motivación

La 84 lo dice sin rodeos: «un sistema de liquidación que no tiene respuesta a "hoy toca cambiar
de clave" no está terminado». Lo que el árbol hace hoy, MEDIDO leyendo el código y con los tests
de los §636 y §638:

| caso | qué hace hoy el árbol | dónde |
|---|---|---|
| **el presupuesto** | XMSS^MT-SHA2_40/8_256: 2^40 = 1.099.511.627.776 índices. El latido gasta uno por cabeza, también sin transiciones, y nada más firma con esa clave: a 60 s son unos 2,09 millones de años; a 1 s, unos 34.800 | `zk_ssl_verify::Conjunto`, `latido::latir` |
| **el agotamiento** | no hay umbral ni aviso. Leído en el código y no ejercido por ningún test: `firmar` reserva el índice antes de firmar, así que en el techo el latido falla, lo anota y sigue quemando el contador; al reiniciar, `IndiceFueraDeCampo` y el nodo no arranca | `FirmanteCabeza::firmar`, `poner_indice_en_sk` |
| **la rotación, en el nodo** | no hay transición firmada. El nodo no distingue una rotación de un reinicio: una clave nueva con el contador de la vieja da `ClaveEnCero`, se resincroniza y sigue la cuenta; con un contador nuevo y el diario viejo no arranca | `politica_de_reconciliacion`; tests del §636 y del §638 |
| **la rotación, fuera** | el testigo fija la primera clave que ve y se detiene ante cualquier otra (`CambioDeClave`), y la clave fijada vive en memoria: un testigo reiniciado fija otra vez. Los sobres del kit que comparan cabezas exigen una sola clave. El ancla lleva la huella de la clave, y el medio la lleva en su `origin`: una clave nueva es otro medio | `Memoria::anclar`, `claves_distintas`, RFC-0012 D-B, RFC-0013 D-A |
| **el índice perdido** | el SK no se persiste: la semilla da la clave en cero y el contador es todo el estado. Contador borrado y diario con firmas: no arranca (§594). Contador y diario perdidos a la vez: el nodo vuelve a firmar desde la hoja 0, y con dos firmas de la misma hoja la clave se extrae con unos 2^34 hashes (la curva de QRL que cita la 84) | `GuardianIndice::abrir`, `diario::maximo_indice`; la 103 |
| **la emergencia** | no hay procedimiento escrito para «creo que he reutilizado un índice». El testigo la detecta (`indice-repetido`, vista dividida) pero no hay qué hacer después | la 92 (c) |
| **el esquema** | ninguna firma dice con qué esquema se hizo: el OID va dentro de la clave publicada, que nadie firma | la 87 |

Dos hechos de fuera fijan el marco. El RFC 10033 (PQUIP, «Hash-Based Signatures: State and
Backup Management», septiembre de 2026) propone un aviso de agotamiento configurable que exija
reconocimiento explícito (§3.4); reconoce que un verificador con todas las firmas detecta un
índice repetido, que es lo que hace el testigo (§4); y dice de la rotación que «no es infalible,
porque exige que haya al menos una clave disponible que dé fe de la nueva» (§5.5). Y el RFC 8649
—el que el NIST SP 800-208 sugiere para actualizar la clave almacenada— resuelve justo eso en la
PKI: el certificado raíz en curso publica el **resumen de la clave siguiente**, un compromiso que
deja reconocer sin ambigüedad a la sucesora cuando llegue.

Y un hecho de dentro decide cuándo: **no hay clave que anclar** (§246). El nodo firma con claves
de prueba, y la primera clave de verdad nacerá con el despliegue. La génesis es el único momento
en que comprometer a la sucesora no cuesta nada.

## Diseño

### D-A — La cuenta de índices es del operador, no de la clave

La clave nueva sigue la cuenta de la vieja: empieza en un índice `desde` posterior a todo lo que
la vieja firmó, y no en cero. Es lo que el nodo ya hace con el contador de la vieja (§638), y
convierte en regla lo que hoy es un accidente:

- la **era del recibo** (`índice + 1`, §567) y su ventana `S − e ≤ N` siguen siendo monótonas de
  una clave a la otra;
- el **reloj del ancla**, el índice embebido (RFC-0012 D-C), también;
- la **puerta del diario** (§594) no estorba: el máximo de la vieja queda por debajo;
- el **testigo**, que clasifica por índice, no ve un índice repetido entre dos claves.

El precio son las hojas de la clave nueva por debajo de `desde`, que se pierden: con 2^40, nada.
Un salto hacia delante cuenta en contra del operador, como ya cuenta `recibos::dentro_de_ventana`
los índices huérfanos: acorta o vence las ventanas abiertas, y eso lo paga quien salta, no el
titular.

### D-B — Pre-rotación: cada clave compromete la huella de la siguiente

Cada clave entra con un **acta** (D-C) que lleva la huella de la clave que la sucederá. Una
rotación vale si la clave que llega es exactamente la comprometida y firma su propia acta. La
clave que se va firma también el acta cuando su estado es fiable. Es el RFC 8649 llevado a una
firma con estado, y resuelve los tres casos en los que un acta firmada solo por la vieja falla:

1. **La vieja comprometida.** Quien la robó puede firmar un acta, pero no con la clave siguiente,
   que no tiene: no puede rotar a una suya.
2. **El índice de la vieja indeterminado** (la 92). Firmar el acta con la vieja arriesga
   reutilizar una hoja, que es lo que se quiere evitar. Con pre-rotación la vieja no necesita
   firmar nada más.
3. **La vieja perdida o agotada.** No puede firmar, y la sucesora comprometida rota igual.

La clave siguiente vive **fría**: el nodo nunca recibe su semilla, solo su huella. Lo que el
nodo pierda o filtre no la alcanza.

### D-C — El acta

```text
acta = { v: 1,
         anterior:  huella_de_clave(la clave que se va)          ← ausente en el acta génesis
         clave:     la clave pública que entra, entera
         esquema:   el conjunto de la clave que entra, p. ej. "xmssmt-sha2_40/8_256"
         desde:     el primer índice que firma la clave que entra
         siguiente: huella_de_clave(la clave que sucederá a esta)
         ultima:    { epochDigest, mmrRoot, mmrSize } de la última cabeza de la anterior
                    ← ausente en el acta génesis }

firma de la que entra:  preámbulo(DOMINIO_ACTA, v, acta_digest), con su hoja `desde`
firma de la que se va:  el mismo preámbulo, con su hoja siguiente, si su estado es fiable
```

`huella_de_clave` es la del RFC-0012 D-B, con su longitud codificada: no nace otra. Como la
huella cubre la clave entera, OID incluido, el conjunto XMSS queda comprometido por ella; el
campo `esquema` existe para cuando la clave que entra sea de otra familia, que es la 87. El
dominio separa el acta de la cabeza y de la cofirma: una firma de cabeza no se lee como acta.
La primera hoja de la clave que entra firma su acta, y las cabezas empiezan en la siguiente.

**Las reglas del verificador**, las mismas en el kit, en el testigo y en la segunda
implementación:
1. El acta génesis no tiene `anterior` ni `ultima`, y la firma la clave que presenta.
2. Un acta de rotación vale si su `anterior` es la huella de la clave del acta previa, si la
   huella de su `clave` es el `siguiente` del acta previa, si su firma verifica con su `clave` y,
   cuando trae la de la anterior, si esa también verifica.
3. Su `desde` supera todo índice embebido de la anterior que el verificador tenga delante.
4. Una cabeza de la clave `K` vale si verifica con `K` y su índice embebido está por encima del
   `desde` del acta de `K` —esa hoja es del acta— y por debajo del de la siguiente, si la hay.
   Una cabeza de la vieja por encima del `desde` de la nueva es **solapamiento**: evidencia
   oponible con nombre, como la vista dividida.

**Fijado en la E2 (§643).** La composición exacta de `acta_digest` está en `NUCLEO.md` §6 y sus
bytes en tres KAT: una etiqueta detrás del dominio separa la génesis (0) de la rotación (1), y la
procedencia entra delante del cuerpo. El esquema de hoy es `0x1_0000_0005`: la familia de RFC 8391
en los 32 bits altos y su OID en los bajos; otro se rechaza con su número. El `desde` es el índice
EMBEBIDO, y la primera firma de la clave que entra lo lleva dentro. Y `verificar_acta` juzga las
reglas ANTES que las firmas: una clave no comprometida se rechaza aunque la firmen la nueva y la
vieja, porque ninguna firma la rescata.

**Fijado en la E3a (§644).** El acta entra por el arranque y es **opt-in**: sin `--siguiente` y sin
actas en el diario, el nodo no firma ninguna, y queda el residuo de la D-I. La `procedencia` de una
rotación es la última cabeza que firmó la clave que se va y **el acumulador de cabezas que hereda
la que entra**: la cima y el tamaño del MMR del diario, que es la pareja que firmará su primera
cabeza. Sin cabeza de la anterior, su digest es el cero declarado del génesis del MMR. Y en la E3a
la clave que se va **no firma**: toda acta de rotación que el nodo firma la declara quemada, hasta
que la E3b la deje firmar cuando su estado es fiable.

**Fijado en la E3b-1 (§645).** La clave que se va firma **solo si el operador la da**
(`--clave-anterior-fichero`): darla es afirmar que su estado es fiable, y el nodo no lo puede
saber por su cuenta. Firma el mismo preámbulo que la nueva, en la hoja que el contador del
operador da —la cuenta es una (decisión 2)—, y la nueva empieza en la siguiente: dos hojas
reservadas por una rotación con las dos firmas, una por la que declara quemada a la vieja. La
semilla se comprueba contra la clave del acta en vigor ANTES de reservar nada. Y el techo lo dice
el **contador**, no el SK: tras firmar con la última hoja, el índice del SK sigue leyendo
2^40 − 1, porque su campo de cinco bytes no representa 2^40, y una segunda firma no da
`KeyExhausted` sino una firma que no verifica; el firmante devuelve `Agotada` sin tocar el
contador, y el latido en el techo falla sin quemar un índice por latido.

**Fijado en la E3b-2 (§646).** El acta tiene **un** JSON y un productor, en el kit: lo que la
línea del diario lleva, sin su `v`, su `tipo` y su `index`, es lo que el cable sirve y lo que los
sobres de la E5 llevarán. `procedencia` y `firmaAnterior` van siempre, con `null` cuando no hay.
El nodo arma la cadena al arrancar y no relee el diario por petición: las actas solo nacen al
arrancar, y el diario crece una firma por latido.

**Fijado en la E4 (§647).** El juicio de una rotación tiene un productor, `juzgar_rotacion` en el
kit, que usan el testigo en vivo y `--auditar`, y la cadena entera, `verificar_cadena`, que usa
además el nodo al arrancar. Puede cruzar varios eslabones: un testigo apagado ve la clave de hoy y
no las de en medio. La regla 3 se aplica a lo que el testigo VIO: el mayor índice embebido de la
clave fijada, entre las cabezas que verificaron, queda por debajo del `desde` de su sucesora. La
regla 4, a cada cabeza de una clave que llegó por rotación —con TOFU no hay tramo, porque nadie lo
dijo—. Fuera de su tramo, `solapamiento`, que detiene. La rotación se anota en su propia línea,
con la cadena que el nodo sirvió, y la misma cabeza se juzga después con la clave nueva; las hojas
que gastaron las actas salen como un `hueco`, que no detiene. El significado de `cambio-de-clave`
se estrecha al cambio que la cadena no explica, y por eso el diario del testigo sube a v4.

### D-D — Dónde vive el acta

En tres sitios, y en todos tal cual:
1. en el **diario** del nodo, como una línea con su tipo, que quien firma anota (§285);
2. en el **cable**, con `zkssl_keyActs`, que sirve la cadena entera desde la génesis: la cadena
   de un operador cabe en un fichero que el titular custodia con sus cabezas;
3. en el **medio** del RFC-0013, como hoja, con su E3: un acta publicada donde el operador no
   puede borrarla, y dos actas distintas para la misma clave son una vista dividida de actas que
   los testigos del medio ven.

### D-E — El testigo y el kit

**El testigo** (E4). Ante un `CambioDeClave` pide `zkssl_keyActs`, juzga la cadena desde la clave
que fijó con las reglas de D-C, y fija la nueva si vale: un veredicto nuevo, «rotada», que anota
y sigue. Si no hay acta, o no vale, se detiene como hoy. `--auditar` hace lo mismo con el diario.
El §245 queda igual en lo que importa: el operador no puede cambiar de clave sin que un tercero lo
vea; lo que cambia es que ahora puede **explicarlo con un objeto**, no con una excusa.

**El kit** (E5). Los sobres que comparan cabezas aceptan un campo opcional `actas` con la cadena
que une sus claves. Sin él, el texto de hoy, byte a byte: «las cabezas llevan claves DISTINTAS: la
continuidad es de UN firmante», y ningún vector existente se mueve. Con él, la continuidad es de
UN operador, con las reglas de D-C.

**Fijado en la E5a (§648).** En un sobre no hay un testigo que haya visto pasar la rotación: hay
dos cabezas firmadas y una cadena. El juez es `juzgar_continuidad`, en el kit, y lo que ordena las
dos cabezas es su índice EMBEBIDO, no el nombre del campo que las trae: la cuenta es una (D-A), así
que la de índice menor es la anterior, sea `vieja`, `cierre` o `juzgada`. La regla 3 se aplica a la
anterior —lo que el verificador vio firmar a la clave que se va es esa cabeza— y la 4 a la
posterior; su rojo nace con nombre, `FueraDeTramo`, que también es SOLAPAMIENTO. Con la misma clave
`actas` no se lee: no hay cambio que explicar. Lo que la E5a NO cubre, dicho en `spec/PAQUETE.md`:
el **conflicto** exige claves distintas y lo arma el delator, que no trae la cadena, así que una
rotación se le puede presentar como dos libros —quien lo juzga pide `zkssl_keyActs` a los dos—; la
**vista dividida** es de una clave, y el solapamiento entre dos merece su propio sobre, con una sola
cabeza de la vieja y la cadena; y el **ancla** de la clave que se fue, extendida por una cabeza de la
que llega, es la costura del medio, la E6.

**Fijado en la E5b (§649).** El catálogo sale de UN nodo que rota dos veces sobre el mismo diario, y
las conductas que la cadena delata se siembran con las claves de verdad, como la vista dividida del
ancla: la clave vieja, ya rotada, firmando otra vez con su contador restaurado —su hoja queda por
encima del `desde` de su sucesora, la regla 3—, y la nueva firmando con un contador fresco por
debajo de la hoja de su acta —la 4—. Solo quien tiene las claves produce cualquiera de los dos
pares. Dos reglas de la D-C no tienen vector, porque una mutación no las produce sin volver a
firmar —la hoja de la firma de la que entra distinta de su `desde`, la de la que se va fuera de su
tramo—: las juzgan los tests del núcleo. La segunda implementación, escrita desde esta D-C y la
sección 2.3 de `spec/PAQUETE.md`, dice lo mismo que la referencia en cada entrada.

### D-F — El agotamiento: un aviso con umbral, y la sucesora ya comprometida

El RFC 10033 §3.4, tal cual: un umbral `M` configurable de índices restantes. Por debajo, el nodo
lo dice en cada latido, y no arranca sin un reconocimiento explícito del operador. A la cadencia
del latido el techo está a dos millones de años y el aviso no saltará; se pone porque es barato,
porque la cadencia es configurable, y porque con D-B agotar la clave **es** rotar a la sucesora,
que ya está comprometida. Cuando falla `firmar` en el techo, el latido deja de quemar el contador.

⚠️ **Precisado en el §645.** Con la D-A, «agotar la clave **es** rotar» no vale EN el techo: la
sucesora empieza en un `desde` por encima de todo lo que la vieja firmó, con el mismo campo de
cinco bytes para su índice, así que a 2^40 la que se agota es la cuenta del OPERADOR, y rotar no
devuelve hojas. El techo es del operador, no de la clave, y el aviso de la E3a, que lee el
contador del operador, ya lo trata así. A la cadencia del latido está a dos millones de años y no
cambia nada de lo construido; si llegara a importar, lo que se revisa es la decisión 2, que es
reversible, no el firmante.

### D-G — El índice perdido y la emergencia, escritos

Un índice indeterminado es un índice que pudo firmarse (la 92: «un índice perdido es mejor que
uno indeterminado»). La regla, escrita como procedimiento del operador y no como código:
1. **Si el estado de la clave no es fiable, la clave está quemada.** No firma nada más: ni una
   cabeza, ni el acta.
2. **Se rota a la sucesora comprometida**, cuya acta firma sola (D-B, caso 2).
3. **El `desde` de la sucesora salta por encima de todo índice conocido**: el diario, los diarios
   de los testigos y el medio. El salto lo paga el operador (D-A).
4. **«Creo que he reutilizado un índice»** es el mismo caso. El testigo ya lo detecta desde fuera
   (`indice-repetido`, la vista dividida), y el medio lo verá con su E3; el acta es la respuesta
   del operador, y sus cabezas anteriores siguen siendo evidencia contra él.

Lo que el nodo puede hacer por construcción (E3): ofrecer la rotación con el `desde` calculado
desde lo que él ve, y negarse a arrancar con la clave quemada si se le dice que lo está.

### D-H — La custodia, declarada como hoy

La clave siguiente es fría por diseño (D-B). Dónde vive la en curso sigue siendo una afirmación
del operador (`--custodia`, §244), comprobada solo para `fichero`. El NIST SP 800-208 exige
módulos hardware y el RFC 10033 §4 los recomienda; esta propuesta no los exige, y lo dice.

### D-I — El residuo, declarado

- **La génesis es TOFU.** Quien la ve primero la fija; si el operador ya mentía, la fija igual
  (§245). El ancla y el medio estrechan la ventana; no la cierran.
- **Robar a la vez la clave en curso y la siguiente** rompe la pre-rotación. Es el mismo modelo
  que el RFC 8649: la siguiente se custodia aparte para que eso sea más difícil, no imposible.
- **Un operador sin acta génesis** se queda donde hoy: el testigo se detiene ante el cambio, y
  rotar no se distingue de un robo. Una génesis tardía vale desde que se publica, no antes.
- **El testigo fija en memoria**, y un testigo reiniciado vuelve a fijar. La E4 no lo cambia; su
  diario sí lo ve (`--auditar`).
- **La 103 sigue abierta**: perder a la vez el contador y el diario sigue apagando la puerta. D-G
  dice qué hacer cuando se sabe; cuando no se sabe, no hay procedimiento que valga.
- **El cofirmante del testigo** tiene el mismo ciclo de vida y el mismo guardián (§298). Esta
  propuesta es para la clave del operador; la del testigo seguiría las mismas reglas en una etapa
  propia.

## Decisiones

Las cinco que este texto deja al autor, con la que recomienda primero:

1. **Quién firma el acta (D-B).** Pre-rotación —la sucesora comprometida firma, y la vieja también
   cuando su estado es fiable— frente a un acta firmada solo por la vieja. La segunda es más
   sencilla y falla justo en los tres casos en que hace falta.
2. **La cuenta (D-A).** La clave nueva sigue la cuenta, frente a empezar cada una en cero. Empezar
   en cero obliga a reescribir la era del recibo, el reloj del ancla y la puerta del diario como
   pares `(clave, índice)`, y un estándar con dos relojes por operador es más difícil de copiar.
3. **El medio ante una rotación (D-D).** Un medio por clave —el `origin` del RFC-0013 sigue siendo
   la huella de la clave, y el acta es la última hoja del medio que se va y la primera del que
   llega— frente a un medio por operador, con el `origin` en la huella del acta génesis. La
   primera no toca nada construido del RFC-0013 y deja a los testigos del medio con la regla de
   C2SP, un `origin` por clave; la segunda ahorra reconfigurar testigos en cada rotación.
4. **El umbral de aviso (D-F).** Por defecto, un año de latidos a la cadencia configurada, con
   reconocimiento explícito para arrancar por debajo, frente a solo un aviso.
5. **La firma de la vieja (D-B).** Obligatoria cuando su estado es fiable y prohibida cuando no lo
   es, frente a opcional siempre. La primera da al verificador una firma más donde se puede, y
   una regla que el nodo comprueba.

**TOMADAS en el §642**, por delegación del autor y con el criterio que dictó para el §609 (el
RFC-0014, «Decisiones»): los principios del proyecto y su portada, aplicables al mayor número de
casos de uso sin modificaciones significativas, porque el objetivo es fijar el estándar. Las cinco,
las recomendadas, y todas REVERSIBLES.

- **1, pre-rotación (D-B).** Es la única de las dos que responde en los tres casos en que una
  rotación hace falta —la clave robada, con el índice indeterminado o perdida—, y la que tiene
  precedente con nombre: el RFC 8649, que el NIST SP 800-208 sugiere. Para el verificador son dos
  reglas más (la huella comprometida y la firma de la sucesora), y para el operador, una clave fría
  que el nodo no ve: vale para cualquier operador, con cualquier custodia.
- **2, la cuenta sigue (D-A).** Un reloj por operador, no uno por clave: la era del recibo, el
  reloj del ancla, la puerta del diario y la clasificación del testigo no cambian una línea, y es
  lo que el nodo ya hace con el contador de la vieja (§638). Empezar en cero habría puesto pares
  `(clave, índice)` en cuatro reglas, y otra implementación tendría que copiar las cuatro.
- **3, un medio por clave (D-D).** No toca nada construido del RFC-0013 —ni `origen_del_medio`, ni
  sus 27 sobres, ni las 33 notas contrastadas con torchwood— y deja a los testigos del medio con la
  regla de C2SP, un `origin` por clave, que es lo que un testigo genérico sabe hacer. El acta es la
  costura: la última hoja del medio que se va y la primera del que llega. El precio, reconfigurar
  los testigos del medio en cada rotación, es de una operación rara.
- **4, el umbral por defecto es un año de latidos, con reconocimiento explícito (D-F).** Es el RFC
  10033 §3.4 tal cual —un aviso que no se salta sin querer—, medido en la unidad que el operador
  configura: un año a la cadencia de su latido, sea la que sea. Solo un aviso habría sido un
  mensaje más en un registro.
- **5, la firma de la vieja, obligatoria si su estado es fiable y prohibida si no (D-B).** Una
  regla que el nodo comprueba en vez de una opción. El verificador acepta el acta con la firma de
  la vieja o sin ella, y si la trae, la exige válida; un acta sin ella es la declaración del
  operador de que la vieja está quemada (D-G), y queda escrita en el acta que él firma.

## Lo que se DESCARTÓ al medir

- **Las cadenas de firma de longitud variable** —la última hoja de un árbol firma la raíz del
  siguiente—: el RFC 10033 §5.6 las descarta por la longitud variable de la firma y porque un
  mismo árbol firmaría mensajes y árboles con autorizaciones muy distintas. El acta es otro objeto,
  con su dominio, y la cabeza no cambia de longitud.
- **Varias claves válidas a la vez** (el SP 800-208, en el RFC 10033 §5.1): rompe «la continuidad
  es de UN firmante» y deja invisible una vista dividida repartida entre claves. La pre-rotación
  es secuencial: una clave en curso, una comprometida.
- **La clave o el esquema dentro de la cabeza**: una v7 de la cabeza y todos sus vectores, para
  decir en cada latido lo que el acta dice una vez por clave.
- **Persistir el SK** en vez de la semilla y el contador: el RFC 10033 §4 pide que el estado y la
  clave no puedan divergir ni clonarse, y el diseño de hoy (§328–§337) ya decide eso con el
  guardián. No se toca.

## Compatibilidad

`zkssl/0.4` no sube. La cabeza v6 no se mueve ni un byte. Nace una familia, el acta, con su
dominio en el REGISTRO de `NUCLEO.md`, su KAT y sus vectores; un método de lectura,
`zkssl_keyActs`; y un campo opcional, `actas`, en los sobres que comparan cabezas. Ningún vector
existente se mueve, y el texto de `claves_distintas` tampoco. Un nodo sin acta génesis sigue
funcionando como hoy. Lo que cambia de semántica es el testigo con la E4: se detenía ante todo
cambio de clave, y seguiría ante uno con acta válida.

## Seguridad

- **La clave de gasto no viaja jamás**: esta propuesta no la toca. Todas las claves de este texto
  son del operador.
- **El acta no es una cabeza**: dominio propio, como la cofirma del §297; una firma no se puede
  reutilizar de una familia en otra.
- **Una rotación no borra nada**: las cabezas de la clave vieja siguen verificando con ella, y
  siguen siendo evidencia contra el operador. Rotar no escapa de una vista dividida: la
  documenta.
- **El salto del `desde`** puede vencer ventanas de recibos abiertas. Cuenta contra el operador
  (D-A), que es quien decide saltar.
- **La disponibilidad**: rotar necesita la clave siguiente. Si se pierden la en curso y la
  siguiente, el operador empieza otra identidad, y sus titulares conservan lo que custodian
  (`spec/RPC.md`, «Apagado»).

## Referencias

- RFC 10033, Wiggers, Bashiri, Kölbl, Goodman y Kousidis, «Hash-Based Signatures: State and
  Backup Management», IETF PQUIP, septiembre de 2026: §3.4 (umbral de aviso), §4 (requisitos del
  estado; el verificador que detecta un índice repetido), §5.1 (varias claves), §5.5 (rotación),
  §5.6 (cadenas de longitud variable).
- RFC 8649, Housley, «Hash Of Root Key Certificate Extension», agosto de 2019: el resumen de la
  clave siguiente en el certificado en curso.
- NIST SP 800-208, Cooper et al., «Recommendation for Stateful Hash-Based Signature Schemes»,
  octubre de 2020.
- RFC 8391 (XMSS), y FIPS 205 (SLH-DSA, sin estado), para la 87.
- En este árbol: la 84, la 92, la 19, la 87 y la 103 del `BACKLOG.md`; los RFC-0010 (el recibo y
  su era), 0012 (el ancla y la huella de la clave) y 0013 (el medio); `doc/ecst/ECST.md`, que
  deja escrito que «la rotación de claves sigue abierta».
