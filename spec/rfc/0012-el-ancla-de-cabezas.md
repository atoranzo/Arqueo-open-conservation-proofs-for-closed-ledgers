# RFC-0012 — El ancla de cabezas: la historia publicada donde el operador no puede borrarla

- **Estado:** ACEPTADO (§607) — **la regla 4 del PROCESO, saldada con medida**, que es lo que esta
  línea pedía: «su canon y su merge». (1) La spec: `PAQUETE.md` 2.12, el sobre del ancla (§592), y
  en `NUCLEO.md` el REGISTRO con `DOMINIO_ANCLA`, `ancla_digest` y `huella_de_clave` (§591). (2) El
  OpenRPC no se mueve: ni un método ni un campo. (3) Los vectores, nuevos bajo `zkssl/0.4`:
  `nucleo/ancla_digest.json` y `nucleo/huella_de_clave.json` (§591) y la familia `ancla/`, veintiún
  sobres (§593). (4) Las suites: el canon `--sello` del autor, VERDE en cada sello desde su merge
  con «ancla 21/21» desde el árbol y desde dentro del tarball, y `tools/banco_ancla.sh` VERDE en su
  `--bancos` de 20 sobre `639b5eb`. Las cuatro etapas se construyeron en la sesión 194 (E1 §590,
  E2 §591, E3 §592, E4 §593); las decisiones D-A a D-H eran DELEGADAS y REVERSIBLES, y se aceptan
  con su residuo D-H: el ancla sigue sin medio donde vivir -lo propone el RFC-0013-.
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude (sesión 194, que lo escribe entero; sesión 193, el giro del §607) — ver [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-09-30
- **Versión del protocolo afectada:** `zkssl/0.4` — **no la mueve, y tampoco mueve la cabeza**:
  ni un método nuevo, ni un campo nuevo, ni un byte nuevo bajo la firma. La familia entra por su
  `tipo`, su dominio y sus vectores, que es la vía que `NUCLEO.md` §3 reserva a las familias
  nuevas que no necesitan entrar bajo la firma.
- **Asiento(s) de AUDITORIA:** §174 (el diseño, medida 10, `doc/ANCLAJE_EXTERNO.md`), §116 (el
  digest que colisiona con ceros finales; aquí, la huella de la clave con longitud codificada),
  §121 (el techo `N_max` y su reloj), §248 (la raíz servida por el acusado no prueba nada; dos
  diarios prueban que emitió dos cosas para el mismo índice), §291–§292 (el MMR de cabezas y su
  pareja firmada), §399 (el índice embebido, el único que la firma acredita), §567 (el reloj es
  el índice XMSS, no el `seq`), §580 (la casa no cría código sin llamador); y los de esta serie:
  §590 (E1), §591 (E2), §592 (E3), §593 (E4); y §607 (la aceptación).
- **Backlog:** B10.6 (el formato del ancla + `M`) y B10.7 (el verificador en cliente) del
  backlog de `doc/CONFIANZA_RESIDUAL.md`; la mitad portable de B10.2 (la prueba de fraude); y
  compone con las entradas 83 y 86 del BACKLOG (el ancla acota la ventana del primer encuentro).

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la promesa, escrita | este texto y `doc/ANCLAJE_EXTERNO.md` reconstruido: qué objeto nace (D-A), su huella y la de la clave (D-B), el reloj embebido (D-C), el lote que ya existía (D-D), la vista dividida dentro (D-E), la cadencia `M` sin constante (D-F), qué revela (D-G) y el residuo (D-H) | no | **construida (§590)** |
| E2 — el ancla en el núcleo | `DOMINIO_ANCLA` en el REGISTRO, `huella_de_clave` y `ancla_digest` en `zk-ssl-hash`, sus KAT en `spec/vectors/nucleo/` y sus filas del censo | no (aditivo) | **construida (§591)** |
| E3 — el sobre, en el mando | `tipo: "ancla"`, undécima forma: derivar, ancla exacta, ancla anterior por consistencia MMR, y la vista dividida; cada regla con su nombre | no | **construida (§592)** |
| E4 — catálogo y banco | `spec/vectors/ancla/` con su MANIFIESTO, la familia décima en `FAMILIAS`, su estrofa del canon y `tools/banco_ancla.sh`, que siembra la vista dividida reproduciendo el ataque de verdad | no | **construida (§593)** |

## Motivación

El residuo #1 (orden y completitud, `SECURITY.md` §2.bis) tiene todas sus piezas de ACOTACIÓN
construidas: la cabeza firmada, el techo del acuse, el recibo de recepción y su sobre (RFC-0010).
Todas suponen que alguien ajeno custodie cabezas desde antes del conflicto. La vía de ELIMINACIÓN
que el §174 diseñó —publicar lo que la cabeza ya firma en un medio append-only ajeno— llevaba
desde entonces en un documento cuyo contenido, además, el árbol publicado perdió (S548: un
marcador de posición). Este RFC ejecuta ese diseño: B10.6, el formato del ancla; B10.7, su
verificador en cliente. Lo que queda fuera, y se dice: elegir el medio y desplegar la cadencia.

## Diseño

### D-A — El ancla es un objeto DERIVABLE, no un método del cable

Un método `zkssl_ancla` sería el §248 otra vez: una raíz servida por el acusado no prueba nada.
El ancla se deriva de la cabeza firmada sola, y puede derivarla **cualquiera** —el operador que
la publica, el testigo que lo vigila, el titular que desconfía de los dos—. El productor es el
propio mando (`zk-ssl-verify`), que con la cabeza sola imprime el ancla y su huella: el
componente de B10.6 es el binario que ya viaja en el kit. El cable no se toca.

```text
ancla = { v: 1,
          clave:       huella_de_clave(publicKey),
          indice:      el índice XMSS EMBEBIDO en la firma,
          epochDigest: el digest que la firma cubre,
          mmrRoot:     la raíz del árbol de cabezas,
          mmrSize:     cuántas contiene }

huella_del_ancla = ancla_digest(clave, indice, epochDigest, mmrRoot, mmrSize)
                 = merge(as_digest(ANCLA_V1),
                     merge(clave,
                       merge(as_digest(indice),
                         merge(epochDigest, merge(mmrRoot, as_digest(mmrSize))))))
```

Lo que se publica en el medio es el ancla —seis claves de JSON— o, donde solo quepan 32 bytes,
su huella. Quien tenga la huella exige el ancla a quien la publicó y la recompone.

### D-B — La huella de la clave, con longitud codificada y dominio de bytes propio

`huella_de_clave(clave) = Blake3(b"ZK-SSL-anchor-key-v1" ‖ len(u64 LE) ‖ clave)`, el molde
exacto de `digest_of_proof` con su corrección del §116 —dos claves que difieran en ceros finales
no colisionan— y un dominio de bytes nuevo en el REGISTRO. Va la huella y no la clave entera por
el medio: una clave XMSS-MT publicada pesa lo que pesa, y 32 bytes caben en cualquier sitio. El
que compara tiene siempre la clave entera (viaja en cada cabeza firmada): recomputar la huella
es un merge, no una confianza.

### D-C — El reloj del ancla es el índice EMBEBIDO, no el declarado

La firma acredita un solo índice: el embebido en sus primeros cinco bytes (§399). El declarado
solo está acotado **por abajo** —dos cabezas honestas pueden declarar el mismo—, así que un
ancla sobre el declarado fabricaría vistas divididas falsas. El ancla compromete el embebido, el
mando lo extrae con `indice_de_firma`, y es el mismo reloj que la era del recibo usa desde el
§567: la clave solo firma cabezas, cada firma quema un índice, y un censor no lo congela.

### D-D — El lote es el árbol que la cabeza ya firma

El §174 pidió «la raíz de un árbol de cabezas cada `M` latidos». Ese árbol existe desde el §292:
la pareja `(mmrRoot, mmrSize)` viaja dentro del digest firmado, con inclusión y consistencia
O(log N) en el verificador (§291, RFC 6962). Anclar una cabeza ancla sus `mmrSize` anteriores
—el ancla individual de cada una es su camino Merkle— y ata a las posteriores por consistencia.
**Nada nuevo se construye, se persiste ni se firma**: cero índices XMSS extra, la corrección que
salvó al acuse (§121.2), heredada entera.

### D-E — La vista dividida entra en esta familia

Dos cabezas de la misma clave con el mismo índice embebido y contenidos distintos son el par
condenatorio que `CONFIANZA_RESIDUAL.md` §2.1 prometió: solo quien tiene la clave puede
producirlo, porque producirlo es reusar una firma de un solo uso. El §248 lo dejó medido —dos
diarios «prueban que el operador emitió dos cosas firmadas para el mismo índice»— y faltaba el
objeto portable. Entra aquí y no en una familia propia porque es la otra cara del ancla: el
ancla obliga al operador a elegir UNA historia; este sobre es lo que le espera si eligió dos.
Es la mitad portable de B10.2, y el sobre la imprime como DETECCIÓN con salida 0, el molde del
conflicto de libros (§430): el sobre que la exhibe no falla — delata.

### D-F — La cadencia `M` se declara en prosa; la constante espera a su llamador

`M` es línea sistémica —familia de `N_max` (§121) y `T` (`doc/CADUCIDAD_PENDIENTE.md`)—: se
elige, se publica, se mide. La propuesta, hasta que haya operador real: **`M` = 1.440 latidos**,
un ancla al día, el horizonte de `N_max` y el MMD de CT. **No nace una constante en el código**:
su único llamador sería el componente de despliegue —el que publica en el medio elegido—, que
este RFC deja fuera a propósito, y la casa no cría código sin llamador (§580). Cuando ese
componente exista, `M` entra con él y con su vigía, como `N_MAX_CABEZAS` tiene el suyo.

### D-G — Qué revela el ancla

Nada que la cabeza firmada no publique ya: los cinco campos son la clave pública (por su
huella, derivable por cualquiera que tenga la clave), el índice que la firma lleva en claro, y
tres valores que viajan en cada `zkssl_signedEpochHead`. Publicar el ancla en un medio ajeno
revela **que ese operador existe y a qué ritmo firma** —eso es exactamente la propiedad— y su
volumen aproximado de épocas, que la `mmrSize` ya publicaba a quien preguntara.

### D-H — El residuo, declarado

Los tres del §174, intactos: el medio es confianza **desplazada, no eliminada**; la cola entre
anclas queda acotada por `M` y N2, **no cerrada**; y el ancla prueba **qué historia era
canónica, no que estuviera completa** — la completitud sigue siendo de los recibos y la
no-inclusión (§121.3, RFC-0010). Y dos propios: el ancla del génesis (`mmrSize` 0) no tiene
historia que extender —se compara entera o espera a la siguiente—; y mientras nadie publique en
un medio real, el ancla es un objeto correcto sin ningún sitio donde vivir: **pendiente de
despliegue**, exactamente lo que `SECURITY.md` §2.bis dice.

## Lo que se DESCARTÓ al medir

- **Un método del cable que sirva el ancla.** El §248, y además movería los cardinales
  publicados sin ganar nada: la cabeza firmada ya viaja y el ancla se deriva de ella.
- **Anclar la clave entera en vez de su huella.** No cabe en los medios de 32–80 bytes y no
  añade nada: la huella con longitud codificada ata la clave byte a byte.
- **El índice declarado como reloj del ancla.** Acotado solo por abajo (§399): dos cabezas
  honestas pueden compartirlo, y una vista dividida falsa mataría la familia entera.
- **Un árbol de anclas propio.** Segundo productor del árbol de cabezas que ya existe (§292);
  dos implementaciones del mismo problema pueden discrepar.
- **Una constante `M` en el código, hoy.** Sin publicador no hay llamador (§580, D-F).

## Compatibilidad

El cable **no se mueve**: `zkssl/0.4`, treinta y un métodos, OpenRPC intacto. La cabeza **no se
mueve**: `VERSION_FORMATO` sigue en 6 y nada nuevo entra bajo la firma. La familia entra con su
`tipo` («ancla»), su dominio (`ANCLA_V1` y `ZK-SSL-anchor-key-v1`, ambos en el REGISTRO), sus
composiciones en el núcleo con sus KAT, y sus vectores bajo `spec/vectors/ancla/` — la vía de
`NUCLEO.md` §3 para las familias que no necesitan entrar bajo la firma. Los vectores viejos no
se tocan. El sobre lee cabezas v3 a v6 —las que llevan la pareja del MMR—, y lo custodiado no
caduca (§290).

### Por qué entra por RFC

Añade una familia de vectores, un objeto al paquete de evidencia y tres elementos al núcleo
congelado. Es exactamente lo que el PROCESO reserva para un RFC, y lo que hizo el RFC-0010.

## Seguridad

- **El principio del API se conserva**: la clave de gasto no viaja. El ancla lleva la huella de
  una clave de FIRMA pública, dos números y tres digests; ninguna clave privada de nada se
  acerca a esto.
- **El ancla no acredita el contenido**: que la historia anclada fuera la canónica no dice que
  fuera completa ni justa (D-H). Un verificador que lea VERDE en un sobre de ancla sabe qué
  historia firmó el operador; qué había dentro lo dicen las otras diez formas del paquete.
- **La vista dividida exige las DOS cabezas firmadas.** Quien solo tiene una no prueba nada, y
  el sobre lo dice: el silencio del operador con una de las partes sigue siendo el residuo D-H
  del RFC-0010. Lo que este objeto cambia: emitir las dos ya no es gratis.
- **La fecha del ancla es la palabra del medio.** El sobre verifica la criptografía; el orden
  externo —que el ancla estaba publicada ANTES— lo da el medio elegido, y esa confianza queda
  desplazada y declarada (D-H). Este RFC no elige el medio (B10.6: «no elige medio»).

## Referencias

`doc/ANCLAJE_EXTERNO.md` (el diseño, reconstruido en el §590), `doc/CONFIANZA_RESIDUAL.md` §2.1,
§8 y su backlog B10.*, `SECURITY.md` §2.bis y §6, `spec/NUCLEO.md` (el MMR y el registro de
dominios), `spec/PAQUETE.md` 2.12 (la forma del sobre), las entradas 70, 83 y 86 del BACKLOG,
el RFC-0010 (el molde de una familia que entra sin mover el cable) y los asientos de la cabecera.
