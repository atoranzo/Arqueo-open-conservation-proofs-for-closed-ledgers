# RFC-0010 — El recibo de recepción: lo que el operador no puede negar haber recibido

- **Estado:** PROPUESTO
- **Autores:** Che, con Claude (sesión 184)
- **Fecha:** 2026-09-25
- **Versión del protocolo afectada:** `zkssl/0.4` — **no sube** (ver Compatibilidad). La cabeza
  pasa a **v6** con una pareja aditiva, exactamente como la v4 del RFC-0006 (§414) y la v5 del
  RFC-0007 (§451), que tampoco movieron el cable; los vectores de la era nueva nacen como
  nacieron los de la v5 en el §453.
- **Asiento(s) de AUDITORIA:** §115 (el latido y su cadencia), §116 (`digest_of_proof` colisiona
  con ceros finales), §120 (el nombre «acuse», reservado), §121 (la política del acuse: el techo
  `N`, el reloj de cabezas firmadas, y las dos correcciones — nada de firma por acuse, y el hash
  de la prueba con longitud codificada), §234 (el guardián con `fsync` que se niega en `tmpfs`),
  §242 (los huecos de índice por reinicio, declarados benignos), §248 (el RPC no sirve la raíz:
  el titular la recompone), §253 (el contador de recepción), §270–§275 (el árbol de acuses: la
  hoja, las delegadas, los límites del diario, la vista del nodo y la `n` firmada), §414 y §415
  (la pareja de consumos, el molde de una pareja nueva en la cabeza), §451–§453 (la cabeza v5 y
  los vectores de su era), §555 (el operador del ATADO D, sin el cual este RFC entra ciego); y
  el §556, que lo adopta.
- **Hito:** H5b de la propuesta enviada a NLnet Restack (120 h), en sus palabras: *«The proof of
  completeness. Every committed acknowledgement resolves within a bounded epoch as an applied
  transition or a rejection with proof; the residue declared.»*

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la promesa, escrita | este texto: qué objeto nace y por qué no es el acuse (D-A), la hoja y su dominio (D-B), la pareja en la cabeza v6 (D-C), la era que el recibo declara (D-D), qué cuenta y qué no (D-E), el sobre de completitud y sus tres veredictos (D-F), la grieta de las causas sin prueba portable (D-G) y el residuo (D-H) | no | **propuesta** |
| E2 — la raíz de recepción en la cabeza | las reglas compartidas en `zk-ssl-verify` con el molde de `acuses.rs`, la vista del nodo con el molde de `vista_acuses.rs`, la composición v6 en el núcleo con su vector conocido, y la pareja firmada | no (aditivo, v6) | propuesta |
| E3 — el recibo, en el cable | `{rx, era, n}` como DATO en el resultado y en `error.data` de las vías del titular, y el método de lectura del camino cuando la era cierra | no (métodos aditivos) | propuesta |
| E4 — el sobre portable de completitud | `tipo: "completitud"`, que un tercero verifica con el kit y sin nodo, con sus tres veredictos y sus reglas de rechazo | no | propuesta |
| E5 — catálogo y banco | `spec/vectors/completitud/`, su MANIFIESTO, la familia en `FAMILIAS` y su estrofa del canon, y el banco que siembra una recepción resuelta y una sin resolver | no | propuesta |

### La frontera con H4 y con H5, y qué es de cada uno

El RFC-0007 escribió que la completitud de los acuses **no es suya**, y el RFC-0008 repitió la
frontera. Este documento la mira desde el otro lado y la cierra:

- **Del RFC-0007 es** *qué* rechaza el nodo y *cómo lo prueba*: la causa como dato (E2) y el
  sobre `tipo: "rechazo"` que el mando verifica sin nodo (E3). Este RFC **no toca una causa ni
  un vector de aquella familia**: los consume.
- **Del RFC-0008 es** lo que un tercero puede comprobar de un pendiente. Nada de esto depende
  de este documento ni al revés.
- **De aquí es** *cuándo*: que toda operación que el nodo llegó a evaluar se resuelva dentro de
  una ventana acotada, y que el titular tenga con qué demostrar que no se resolvió.

## Motivación

El nodo ordena, y puede callar. Está declarado en cuatro documentos vivos y no es un descuido:
es el modelo. Lo que no está es la consecuencia operativa, y la medida la enseña sin ambigüedad.

El camino de una operación tiene cuatro etapas —`parse` → `try_into` → `apply` → `append`— y
`seq` sólo nace en la última. **La censura vive en el hueco entre recibir y aplicar, y hoy ese
tramo no deja huella firmada.** Un operador que censura no incrementa `seq`, y nada lo delata.

Media máquina está construida y medida. Hay un contador de recepción monótono, persistido con
`fsync` antes de devolver el número, que se niega a operar sobre un medio que no persiste y que
sobrevive al reinicio sin reusar (§234, §253). Viaja al titular como `receptionSeq`. Con él, dos
titulares que cooperan **detectan** la reordenación: A tiene la recepción 100, B la 101, la de B
está en el registro y la de A no. Y ahí se acaba: **no pueden probarla**, porque nadie tiene
nada firmado por el operador que diga *«recibí la tuya la 100»*. El operador puede negarlo.

Lo que falta está nombrado en el propio código desde el §253 —*«el acuse como hoja bajo una raíz
en la cabeza»*— y fichado como una de las dos extensiones pendientes de la cabeza. Y la política
ya se decidió en el §121, con su techo: **no inclusión en `N` épocas es censura**, con
`N = 1.440` cabezas firmadas, veinticuatro horas al latido del §115, con el precedente del MMD
de Certificate Transparency. Esa constante vive hoy en el árbol, viaja firmada en la cabeza
desde el §275 y entra en cada hoja del acuse, de modo que una `n` mentida no verifica.

La pieza que falta es un objeto, no una idea: **una raíz de recepción en la cabeza firmada**.
Con ella, la ecuación que `doc/CONFIANZA_RESIDUAL.md` ya escribe se vuelve comprobable por un
tercero: recibo firmado más no resolución en `N` cabezas es evidencia portable de censura. La
censura sigue siendo posible; deja de ser gratuita.

## Diseño

### D-A — El objeto nuevo es el RECIBO DE RECEPCIÓN, y no es el acuse

El «acuse» está tomado. Hoy es la hoja de una **transición aplicada**: `vista_acuses` construye
el árbol de la época desde `transition_log().entries()`, así que un acuse **sólo existe para lo
que ya se aplicó**. Por eso la frase del hito —«todo acuse comprometido se resuelve… o como
rechazo con prueba»— es hoy cierta por vacío: la única rama que puede darse es la primera.

Nace un objeto hermano y separado, el **recibo de recepción**: la hoja de una operación que el
nodo **llegó a evaluar**, se aplicara o no. Dos árboles, dos raíces, dos parejas en la cabeza.
No se reutiliza el árbol de acuses: mezclarlos haría que el número de hojas dejara de significar
una sola cosa, y la casa no recicla un nombre cuando cambia lo que compromete.

**Reversión:** un solo árbol con un bit de clase por hoja. Se descarta en «Lo que se DESCARTÓ».

### D-B — La hoja, con dominio propio y el molde del acuse

`recibo_digest(hash_de_la_prueba, era, n) = merge(as_digest(RECEP_V1), merge(hash_de_la_prueba,
merge(as_digest(era), as_digest(n))))`, calcado de `acuse_digest` con un dominio séptimo,
`RECEP_V1`, en el REGISTRO de `zk-ssl-hash`. Misma forma, misma clase de objeto, otro dominio:
un recibo nunca puede pasar por un acuse ni al revés.

`hash_de_la_prueba` va **con la longitud codificada**, no como `digest_of_proof` a secas: el §116
midió que ese digest colisiona con ceros finales, y el §121 ya lo dejó decidido. Se conserva.

`n` va **dentro de la hoja**, como en el acuse (§270): cambiar el techo cambia el árbol entero, y
por eso un `n` mentido en una respuesta produce una hoja que no verifica contra la raíz
recompuesta con el `n` de la cabeza **firmada**.

### D-C — La cabeza v6 firma la pareja `(recepRoot, recepCount)`

`v6 = merge(v5, merge(recep_root, as_digest(recep_count)))`, con el molde exacto de la pareja de
consumos que estrenó la v4 (§414). Génesis: la raíz del árbol vacío y `recep_count = 0`.

`recep_count` es el contador de recepción en el momento de componer la cabeza. El límite
**inferior** de la era no se firma: es el `recep_count` de la cabeza anterior, y el titular que
custodia dos cabezas consecutivas lo tiene. Es el mismo reparto que `limites_para` ya hace con
los `seq` del diario, y la misma convención de borde: la era es `[Q, R)`, con `Q` **inclusivo**,
porque con `Q` exclusivo la recepción número uno no pertenecería a ninguna era.

El índice de la hoja dentro del árbol de su era es **denso desde cero**: `rx - Q`. Denso a
propósito, como en el acuse: la era es pequeña y cualquiera reconstruye posiciones desde dos
cabezas firmadas sin datos extra.

### D-D — La era que el recibo declara es la primera cabeza que puede contenerlo

`era = seq_de_la_última_cabeza_firmada + 1`, computada **en la recepción**, no al cerrar. Es el
espejo del `epoca_de_acuse(seq) = seq + 1` y se elige por la misma razón que allí: atar la era a
la cabeza real que acabe conteniéndolo **pondría el valor de la evidencia en manos del acusado**
—el titular no podría fijar su recibo hasta que el operador decidiera—.

Y da la magnitud que la promesa acota: si el recibo declara `era = e` y el titular exhibe una
cabeza firmada con `seq = S`, entonces `S - e` es el retraso **en cabezas**, legible del recibo y
de la cabeza solos. La promesa es `S - e <= N`.

Una era adelantada no le sirve al operador: el titular contrasta `e` contra la cabeza que ya
custodia en el momento de recibir, y una `e` por delante de lo que el nodo ha firmado es visible
al instante. Y como la era va **dentro** de la hoja, mentirla después rompe el camino.

### D-E — Cuenta lo que el nodo llegó a EVALUAR; ni el ruido ni lo aceptado

La regla ya está medida y escrita en `recepcion.rs`, y este RFC la eleva a norma sin cambiarla:

| falla en | ¿consume recibo? | por qué |
|---|---|---|
| el parseo o el cable | **no** | es ruido, no una operación; si contara, cualquiera podría abrir huecos en el registro ajeno mandando basura |
| la capa (prueba inválida, raíz movida) | **sí** | el nodo verificó una operación de verdad y decidió no aplicarla, **y ahí es donde se escondería un censor**: rechazar alegando prueba inválida |

El contador es el que ya existe (`ContadorRecepcion`), con su `fsync` antes de devolver y su
negativa a arrancar sobre un medio que no persiste. **No nace un segundo contador**: dos
implementaciones del mismo problema pueden discrepar.

### D-F — El sobre de completitud, y sus tres veredictos

`tipo: "completitud"`, familia novena, verificado por el mando **sin nodo**. Lleva el recibo con
su camino bajo la `recepRoot` de una cabeza firmada, la cabeza de cierre con su `seq`, y —si
existe— la resolución. El verificador dice una de tres cosas:

1. **Resuelta como transición aplicada**: la resolución es el camino de la hoja del acuse bajo
   la `acusesRoot` de una cabeza con `seq <= e + N`, con el mismo `hash_de_la_prueba`. VERDE.
2. **Resuelta como rechazo con prueba**: la resolución es un sobre `tipo: "rechazo"` del
   RFC-0007, verificado por sus propias reglas, sobre una cabeza dentro de la ventana. VERDE.
3. **No resuelta en la ventana**: el recibo verifica, la ventana ha expirado —`S - e > N` con `S`
   de una cabeza firmada— y no se presenta ninguna de las dos. **ROJO NOMBRADO**, y ése es el
   producto: un objeto portable que dice, con la firma del propio operador dentro, que se
   comprometió a resolver y no lo hizo.

⚠️ El tercer veredicto es una afirmación sobre lo que el operador **no exhibe**, y eso se dice
aquí con todas las letras: no es una prueba criptográfica de ausencia, porque probar que algo no
está en ninguna de `N` épocas exigiría las `N` épocas enteras. Lo que es —y basta para el hito—
es evidencia **oponible**: la promesa está firmada por el acusado, la ventana es aritmética
sobre dos cabezas firmadas, y la carga de exhibir la resolución es de quien la tiene.

### D-G — La grieta: las causas de rechazo sin prueba portable

La segunda rama de la disyunción **no es total, y no se va a vender como si lo fuera**. El
RFC-0007 declaró causas que no producen prueba portable. Un recibo que se resuelva por una de
ellas no puede exhibir el veredicto 2, y el sobre lo dice con su nombre en vez de callarlo: el
verificador responde «resolución declarada, no probada», que es un cuarto estado y se cuenta
aparte. Cerrar esa grieta es dar prueba portable a esas causas, y eso es del RFC-0007, no de
aquí.

### D-H — El residuo, declarado

Lo que este RFC **no cierra**: la operación para la que el nodo **nunca firmó un recibo**. Si no
contesta, o contesta sin recibo, no hay objeto que oponer. El titular lo sabe al instante —su
petición no trae recibo— y puede reintentar y publicar la ausencia, pero **no puede probarla**.

Eso es el residuo del hito, y es irreducible en este modelo: un operador que no responde es
indistinguible de una red caída. Lo que el recibo cambia es el terreno: quien contesta queda
atado, y el silencio total es una conducta visible y sostenida en el tiempo, no un descarte
silencioso entre miles de operaciones atendidas.

## Lo que se DESCARTÓ al medir

- **Un solo árbol con un bit de clase.** Ahorra una raíz y una pareja, y a cambio el número de
  hojas de una era deja de significar una sola cosa. La claridad manda sobre el byte.
- **Firmar un recibo por operación.** Decidido en contra en el §121 por aritmética, y la medida
  lo confirma: cada firma quema un índice XMSS, y el latido ya quema 1.440 al día. Un recibo por
  operación haría de la firma el cuello de botella del sistema. El recibo hereda la firma de la
  cabeza por estar bajo su raíz: cero índices nuevos.
- **Que el RPC sirva la raíz de recepción.** Es el §248 otra vez: el titular la **recompone** y
  la compara con la cabeza que custodia. Una raíz servida por el acusado no prueba nada.
- **Atar la era a la cabeza que acabe conteniendo el recibo.** Registra la publicación, que ya
  es derivable, y deja al titular esperando a que el operador decida. Es el razonamiento del
  §274, aplicado al objeto nuevo.
- **Contar lo que llega al puerto.** Abriría huecos en el registro ajeno mandando basura.

## Compatibilidad

El cable **no sube**: se queda en `zkssl/0.4`. La cabeza pasa a **v6** con una pareja aditiva, y
hay precedente medido dos veces —la v4 del RFC-0006 (§414) y la v5 del RFC-0007 (§451), las dos
con el cable quieto—. Lo firmado crece sólo por versión, y lo que no está firmado no existe para
el núcleo: la v6 entra en `NUCLEO.md` con su composición exacta y su vector conocido en
`spec/vectors/nucleo/`, y los vectores de la era nueva nacen en `spec/vectors/cable/` como
nacieron los de la v5 en el §453. Los vectores viejos **no se reescriben**.

El método de lectura del camino es **aditivo**, como lo fue `zkssl_applyMany` en el §222: añadir
un método no sube la versión porque los vectores de conformidad no se mueven. Sí mueve los
cardinales publicados de métodos, y eso lo paga la etapa que lo traiga, no ésta.

**El principio del API se conserva**: la clave de gasto no viaja. El recibo lleva el hash de una
prueba, dos números y nada más; ni el nodo ni el sobre la ven en ningún momento.

### Por qué entra por RFC

Toca la cabeza firmada, que es lo que un verificador recompone, y añade una familia de vectores
y un objeto al paquete de evidencia. Es exactamente lo que el PROCESO reserva para un RFC.

## Seguridad

- **Qué revela la raíz de recepción.** El número de hojas de una era dice **cuántas operaciones
  evaluó el nodo**, incluidas las que rechazó. Hoy la `acusesRoot` ya publica cuántas aplicó;
  esto añade el volumen de lo rechazado. Es una fuga de volumen, no de contenido, y se declara.
- **Qué revela el recibo.** El `hash_de_la_prueba` y un número de orden. El orden es justo lo
  que hace detectable la reordenación: es la propiedad, no un efecto colateral.
- **Lo que sigue sin estar cerrado.** El operador puede no emitir recibo (D-H). Puede ordenar.
  Y una cabeza no firmada no obliga a nadie: toda esta promesa cuelga de que el nodo firme sus
  cabezas, que es la misma raíz de confianza que el resto del sistema declara.
- **El reinicio.** El §242 declaró huecos de índice benignos al reiniciar; el contador de
  recepción, en cambio, **no se reinicia** y su fichero tiene su propio guardián. Que fuera
  benigno en un sitio y catastrófico en el otro está medido y escrito desde el §253: dos
  límites que por separado se declaran y juntos romperían la propiedad.

## Referencias

`spec/PAQUETE.md` (las formas del sobre), `spec/NUCLEO.md` (las composiciones de la cabeza y el
registro de dominios), `spec/RPC.md` (el contador de recepción y lo que no es), el RFC-0004 (el
paquete de evidencia), el RFC-0006 (el molde de una pareja nueva en la cabeza), el RFC-0007 (el
rechazo con prueba), `doc/CONFIANZA_RESIDUAL.md` §2.1 (el recibo de recepción firmado como
evidencia portable de censura) y los asientos de la cabecera.
