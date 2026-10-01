# RFC-0015 — El ciclo de vida del firmante: rotar, agotar y perder el índice, con la clave siguiente comprometida

- **Estado:** PROPUESTO (§639) — el texto y sus decisiones, escritos sobre lo medido en el árbol
  (§636, §638) y en el RFC 10033 de la IETF; ninguna etapa construida. Las cinco decisiones son
  del autor: el texto recomienda una en cada una y no toma ninguna.
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
  (lo que hoy pasa al cambiar de clave, medido) y §639 (esta propuesta).
- **Backlog:** la **84** (agotamiento, rotación y pérdida del índice), con la **92** (custodia y
  supervivencia del índice) y la **19** en su línea de familia, que el §288 pidió cortar juntas;
  la **87** (agilidad criptográfica: el acta lleva el esquema de la clave que presenta); y la
  **103** (la puerta del diario se apaga quitando un fichero), que esta propuesta no cierra.

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — el ciclo de vida, escrito | este texto | no | **propuesto (§639)** |
| E2 — el núcleo | `DOMINIO_ACTA`, `acta_digest` y su KAT, y `verificar_acta` en el kit, con la segunda implementación | no (aditivo: una familia nueva) | pendiente |
| E3 — el nodo | el acta génesis, la rotación como orden del operador, el aviso de agotamiento y `zkssl_keyActs` | no (aditivo: un método) | pendiente |
| E4 — el testigo | ante un cambio de clave pide el acta, la juzga contra la clave que fijó y sigue o se detiene; `--auditar` la juzga en el diario | no | pendiente |
| E5 — el kit, el catálogo y el banco | el campo `actas` en los sobres que comparan cabezas, sus vectores y `tools/banco_rotacion.sh` contra un nodo real | no (aditivo: un campo opcional) | pendiente |
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

### D-F — El agotamiento: un aviso con umbral, y la sucesora ya comprometida

El RFC 10033 §3.4, tal cual: un umbral `M` configurable de índices restantes. Por debajo, el nodo
lo dice en cada latido, y no arranca sin un reconocimiento explícito del operador. A la cadencia
del latido el techo está a dos millones de años y el aviso no saltará; se pone porque es barato,
porque la cadencia es configurable, y porque con D-B agotar la clave **es** rotar a la sucesora,
que ya está comprometida. Cuando falla `firmar` en el techo, el latido deja de quemar el contador.

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
