#!/usr/bin/env bash
# =========================== EL CANON ===========================
#
# **La fuente unica.** Antes de §224 cada BLOQUE reescribia sus compuertas
# a mano, y por eso `zk-ssl-wire` paso SEIS sellos sin que nadie corriera
# sus tests.
#
#     bash tools/canon.sh --sello      # cada bloque · **470 a 926 s** de reloj (⚠️ §811, 14-10-2026) (⚠️ §864: sin el «1 bis», que les suma de 16 a 26 s en caliente, de 61 a 69 tras tocar `zk-ssl-hash` y unos 100 en frio, medidos en el carril donde se escribio el §864, que era otro arbol; y de 7 a 8 en caliente y de 39 a 41 tras tocar, medidos al portarlo encima del §863; su razon la imprime el)
#     bash tools/canon.sh --largo      # + halo2 y plonk, y el nodo y el testigo desde dos raices (⚠️ §824: decia «+ halo2 y plonk») · sin medir (⚠️ §811, 14-10-2026)
#     bash tools/canon.sh --completo   # + zk-core · **58 min** de tests, §629 (⚠️ §811, 14-10-2026) (⚠️ §822, 15-10-2026: con el arbol denso de zk-core; al pie)
#
# ⚠️ §811, 14-10-2026: las cifras de los niveles, y lo que consta de cada uno,
# al pie. Las de antes las midio el CANON, no los bancos (§226, primera
# pasada de `--completo`): 151 + 347 (halo2) + 689 (plonk) + 2.032
# (zk-core) = **3.217 s**. Los bancos F.1 y G.2 daban cifras mayores
# porque midieron en FRIO; el canon mide con el `target` caliente, que es
# como se corre de verdad. La diferencia no es pequeña: zk-core 2.032 s
# aqui frente a los 2.317 que sumaban las dos invocaciones separadas de G.2.
#     bash tools/canon.sh --lista      # solo enseña la tabla, no ejecuta
#     bash tools/canon.sh --bancos     # los bancos, todos (tools/bancos.sh) · ⚠️ §811, 14-10-2026
#                                      # (⚠️ §804: menos el de los cortes, que se corre aparte:
#                                      # su punto 5)
#     bash tools/canon.sh --conservados  # spec/vectors/<version>/, cada uno con el kit de SU
#                                        # commit (tools/conservados.sh) · **~2 min** (§798)
#     bash tools/canon.sh --conservados --en-seco  # su plan, sin construir nada · ~1 s
# ⚠️ §864: y lo que NO ve ninguno, clase por clase, con su razon y cuando se reabre: «Ceguera declarada», al pie (la 94).
# ## Lo que lo hace ROBUSTO, y no es la tabla
#
# 1. **Los miembros del workspace se LEEN de Cargo.toml.** Un crate nuevo
#    sin fila pone la compuerta ROJA, y una fila sin crate tambien. El
#    agujero de §223 pasa de "hay que acordarse" a **imposible**. Los lee tools/miembros.py (§833).
# 2. **Pines EXACTOS, no minimos.** Nada mejora ni empeora en silencio.
# 3. **No se para en el primer fallo:** da el inventario completo.
# 4. **Timeout POR CRATE.** Un crate que cuelgue no cuelga la compuerta.
# 5. **Anclas ASCII.** `.` de grep casa un BYTE: un ancla con tilde no
#    falla, se queda MUDA (§223).
# 6. **Here-string, nunca `echo | grep`**: con `pipefail`, `grep -q` sale
#    al encontrar, `echo` recibe SIGPIPE y devuelve 141 aunque la busqueda
#    acierte. Una compuerta intermitente es peor que ninguna.
# 7. **Dice que linea editar** cuando un pin no cuadra.
# 8. **Ningun `.rs` bajo `src/` puede quedar sin declarar** (§227). Lo que
#    no se compila no se verifica — y encima invita a especular.
# 9. **El lock no se mueve** (§694). Todo `cargo` que este fichero lanza
#    en los niveles (`--sello`, `--largo`, `--completo`) corre con
#    `--locked`, y `tools/artefacto.sh` ya lo llevaba: un `Cargo.toml` que
#    pida algo que el `Cargo.lock` no tiene pone ROJA la fila, en vez de
#    reescribir el lock en silencio y probar otra cosa. Los bancos de
#    `--bancos` (`tools/banco_*.sh`), desde el §786, con su puerta mas abajo (§807: decia «todavia no»).
#    Que el lock lleve lo que debe lo dicen dos tests del kit, en su fila:
#    `las_fijaciones_son_exactas` (los `=` de winterfell, `xmss` y `ml-dsa`,
#    y el fork por ruta) y `la_clausura_del_kit_no_lleva_el_probador` (la
#    clausura del kit, una lista cerrada).
#
# ## Los TRES niveles, y por que tres (§225)
#
# `zk-core` cuesta **35 minutos** medidos por el canon (2.103 s, §629;
# ⚠️ §811, 14-10-2026: decia 33,9, los 2.032 s del §226). G.2 lo midio en
# dos invocaciones separadas y en frio: 1.472 s los 73 tests de biblioteca
# con 8 hilos mas **845 s la ceremonia**. Y no hay hilo que lo arregle:
# medido, generar la prueba añade un 5 % y rayon un 17 % — **el coste es
# SINTETIZAR el circuito**, y `ark-r1cs-std` sintetiza en serie. (⚠️ §822, 15-10-2026: en la biblioteca no lo era, y los 35 minutos se midieron con el arbol denso; al pie)
#
# Meter 58 minutos en cada sello (⚠️ §811, 14-10-2026: decia 54) garantizaria
# que alguien acabe saltandose la compuerta, y una compuerta que se salta no
# protege. Es exactamente lo que le paso a `check_tests.py`.
#
# ⚠️ **El nivel `--completo` NO lo fuerza nadie.** Es disciplina, no
# compuerta, y la disciplina es justo lo que ha fallado seis veces en este
# proyecto. Lo unico que se puede hacer por construccion es **no fiarlo a
# la memoria**: cada `--completo` que pasa deja constancia en
# `.canon/ultimo-completo`, y TODA invocacion dice cuando fue y cuantos
# sellos han pasado desde entonces.
#
# ⚠️ **Los bancos, tampoco (§582).** Viven fuera de los niveles: arrancan
# el nodo, el testigo y el mando reales. `--bancos` los corre todos
# (`tools/bancos.sh`), cada pasada VERDE deja constancia en
# `.canon/ultimo-bancos`, y TODA invocacion dice cuando fue y si lo que
# ejercen -`crates/`, los `Cargo.*` y los propios bancos- cambio desde
# entonces. El de la reutilizacion estuvo ROJO del §337 al §579 sin que
# nada lo dijera.
#
# ⚠️ **Los conservados, tampoco (§798).** `spec/vectors/0.3/` y `0.4/` guardan
# catalogos que el kit del arbol ya no verifica y que no se reescriben: los
# verifica el `zk-ssl-verify` del commit que nombra su cabecera. Hasta el §798
# los de `0.3/` los midio la mano (§620, §693); los dos de `0.4/completitud`
# los paso el canon del §681 con el kit de entonces, `41f765d`, y el «2 de 2»
# del §682 es el de la release `arqueo-verify-v0.4.2`, no el de ese kit.
# `--conservados` construye cada kit en un worktree de su commit, con
# `--locked` y un target propio, y lo juzga con DOS arneses, el de su commit
# y el del arbol (`tools/conservados.sh`). Cada pasada VERDE deja constancia
# en `.canon/ultimo-conservados`, y TODA invocacion dice cuando fue y si
# cambio lo que ejerce: los catalogos conservados, el arnes, la tabla de
# `tools/conservados.sh` y el rustc. Un cambio en `crates/` NO lo vence: los
# kits son de su commit. ⚠️ **Su ruta real se ha corrido UNA vez** (§798), en
# un clon desechable: VERDE en 125 s, 41 de 41 en `0.3/` y 2 de 2 en
# `0.4/completitud` con los dos arneses; el detalle, en la cabecera de
# `tools/conservados.sh`.
#
# ## Como se actualiza un pin
#
# Se MIDE primero y se edita la tabla despues. Nunca al reves.
#
# ================================================================

# ⚠️ La guarda va ANTES de cualquier bashismo: con `sh` el `set -o pipefail`
# de abajo revienta primero y el aviso no llega a imprimirse nunca.
if [ -z "${BASH_VERSION:-}" ]; then
  echo "canon.sh necesita bash: usa 'bash tools/canon.sh', no 'sh'." >&2
  exit 2
fi

set -uo pipefail

# La raiz sale de la ruta del propio fichero. `CANON_RAIZ` la sobreescribe,
# y existe para PROBAR esta herramienta desde fuera del repo: una copia
# mutilada en /tmp resolveria su raiz a /tmp y no vigilaria nada. Una
# compuerta que no se puede probar en negativo no es una compuerta.
RAIZ="${CANON_RAIZ:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
cd "$RAIZ" || exit 2
NIVEL="${1:---sello}"
SELLO_FILE=".canon/ultimo-completo"
BANCOS_FILE=".canon/ultimo-bancos"
CONSERVADOS_FILE=".canon/ultimo-conservados"

# ── LA TABLA ────────────────────────────────────────────────────
# crate | nivel | pasan | ignorados | warnings | timeout_s | nota
#
# Medido en §224 (banco F.1) y corregido en §225 (bancos G.1 y G.2) sobre
# el sello 06106c9, en release. `pasan` e `ignorados` son lo que el arnes
# EJECUTA, no los `#[test]` declarados.
TABLA=$(cat <<'FIN_TABLA'
zk-ssl             sello     564   9   0   900  alias=capa · la capa · §292: 262 -> 263, la cima en el digest · §304: 263 -> 264, el atado de la cifra publicada · §340: 264 -> 265, la altura entra en el digest · §345: 265 -> 269, el compositor v2 (RFC-0003, E1a) · §350: 269 -> 272, el cobro aprende el sobre (E3b-1 del RFC-0003) · §351: 272 -> 276, la apertura en el recibo (E3b-2 del RFC-0003) · §353: 276 -> 279, la via viva aprende el sobre (E3c-1b del RFC-0003) · §355: 279 -> 284, la guarda de ancho llega a los gemelos · §357: 284 -> 285, el testigo funcional de la des-emision (deuda del §351) · §359: 285 -> 286, el export del snapshot falla cerrado · §362: 286 -> 287, el reloj sale del atado a su contrato propio · §379: 287 -> 289, la conservacion del dinero se comprueba al abrir · §387: 289 -> 292, la raiz del pendiente se guarda y se comprueba al abrir · §388: 292 -> 294, la raiz de meta del pendiente se guarda y se comprueba al abrir · §390: 294 -> 295, descongelar sobrevive al reinicio · §391: 295 -> 298, la raiz de congelados en reposo · §392: 298 -> 301, la raiz del registro en reposo · §393: 301 -> 304, los contadores custodiados los deriva el registro · §394: 304 -> 307, la cuota de custodios la deriva el registro · §413: 307 -> 317, el consumo publicado (RFC-0006, E1) · §415: 317 -> 318, la pareja de consumos en el digest (RFC-0006, E2b) · §416: 318 -> 323, el camino del consumo (RFC-0006, E3a) · §452: 323 -> 326, la familia de v5 en la cabeza de la capa (RFC-0007, E1b) · §454: 326 -> 329, la causa de un rechazo como dato (RFC-0007, E2) · §458: 329 -> 330, el camino de congelados en la capa (RFC-0007, E3b) · §461: 330 -> 335, el instrumento de la puerta de la prueba de edad (RFC-0007, E4a) · §463: 335 -> 337, la prueba de edad real sobre el libro de la capa (RFC-0007, E4b-1) · §465: 337 -> 340, la prueba de edad se enlaza a la cabeza del libro (RFC-0007, E4b-2) · §466: 340 -> 344, el productor de la prueba de edad (RFC-0007, E4b-3) · §474: 344 -> 345, el camino de cuentas y su atado (RFC-0007, E5, 3b) · §478: 345 -> 350, el productor de la banda del saldo (RFC-0007, E5, corte 4b) · §485: 350 -> 354, el instrumento del cobro pendiente portable (RFC-0008, E1) · §487: 354 -> 357, la no-congelacion la impone el aplicador, no la prueba (5.A-264, A) · §491: 357 -> 365, el productor del cobrador (RFC-0008, E1) · §492: 365 -> 372, la foto de los pendientes que toma el latido (RFC-0008, D-F) · §495: 372 -> 373, el enlace del cobro pendiente en el productor de la capa (RFC-0008, E1) · §502: 373 -> 378, el instrumento del pago en curso portable (RFC-0008, E2) · §504: 378 -> 389, el productor del pago en curso en la capa (RFC-0008, E2) · §518: 389 -> 398, el productor de la prenda en la capa, con la clave del prendador (RFC-0008, E3) · §529: 398 -> 399, el testigo de la guarda de forma: la tupla entera, antes de construir el AIR, en los quince verify de la capa (5.A-381) · §531: 399 -> 421, el testigo de lo que revela una prueba: las 21 filas de D-B y el censo del cable (RFC-0009, E2) · §570: 421 -> 422, la pareja de recepcion entra en el digest y la cabeza compone v6 (RFC-0010, E2d) · §578: 422 -> 423, toda verificacion viva de la capa lee con sal: los quince y la pareja umbral · §641: 423 -> 424, la cota nativa de la banda en verify_audit, la atadura de identidad en circuit_audit y la aritmetica comprobada del envio/cobro/quema (RFC-0017) · §651: 424 -> 425, una prueba oculta con el meta vaciado es Err y no panico, en la guarda de forma y en apply_send (D3-META) · §653: 425 -> 426, una prueba con bytes de cola no verifica (SEG-04) · §654: 426 -> 428, una prueba del titular ya aplicada no vuelve a valer: tras un reembolso y tras un ciclo A->B->A (SEG-03) · §659: 428 -> 429, una prueba maleada por dentro -un nodo de mas en el lote de Merkle- no se reenvia tras un ciclo A->B->A (SEG-03b) · §668: 429 -> 431, el tope de suministro de la capa es 2^62 - 1: new y open lo exigen · §697: 431 -> 433, el nivel de produccion atado: conjeturada, LDR y UDR sobre las pruebas ocultas reales de longitud fija, y ningun verificador acepta por nivel minimo (ZK-1 de blueprint-v2) · §701: 433 -> 436, la capa con claves anchas de custodios y de gobernanza: las cinco autoridades, una clave de un elemento que no autoriza y la rotacion de un libro estrecho (RFC-0018 E2) · §702: 436 -> 446, el KDF de reposo con su version (RFC-0001): el vector de Argon2id del RFC 9106, los KAT de la clave del libro en las dos versiones, la cabecera que solo lee la v2 con su coste, cada libro su sal y la clave sin fijar que no cifra, argon2 clavada y solo en la capa, el libro v1 que se migra al abrirlo y el que con otra frase no se toca, la cabecera y la sal de cada libro, y la instantanea v2 con su cabecera y la v1 que se sigue importando · §704: 446 -> 447, el montaje comparte sus pruebas y nada mas: las cinco ayudas delegadas prueban una vez por traza, y la CPU de la fila baja de 643 a 410 s, antes del doble muestreo del test de tiempos; y el timeout 600 -> 900, MEDIDO con un solo nucleo, lo que la fila tiene con la maquina cargada (de 1,0 a 1,3 nucleos): recompilar la capa 254 s y sus tests 433 s, 687 s que no caben en 600 (medido el 07 y el 08-10) · §774: 447 -> 448, la instantanea cifrada se importa desde memoria, sin escribir el claro al lado · §802: 448 -> 467, una identidad, una cuenta en la vida del libro: abrir rechaza la que ya tiene cuenta o la que una recuperacion retiro, recuperar tambien, y las retiradas se guardan, se exigen al reabrir, viajan en la instantanea v9 y cruzan las dos migraciones (SECURITY 3.13) · §812: 467 -> 481, el recifrado del libro (RFC-0001, E4): la migracion v1 no deja en los ficheros lo sellado con v1, recifrar cambia frase y sal sin dejar nada de la vieja, uno que falla antes del compromiso no toca el libro, una muerte inyectada en cada paso antes y despues del compromiso, con el hermano en 0700 hasta tener los modos del libro, una muerte en cada paso de la migracion no deja en el directorio nada de la v1 ni de una sal anterior, un error pasado el compromiso dice con que frase queda el libro, el libro migrado por el 702 se limpia recifrandolo, la apertura rehusa los estados que el recifrado no deja, recifrar rehusa un enlace, lo que no es de sled y una ruta sin libro, recifrar conserva las identidades retiradas del 802, y la importacion de una instantanea barre el claro que dejaba una importacion muerta (SECURITY 3.14) · §839: 481 -> 482, el censo de las longitudes del cable: la T de cada AIR que verifican la capa, la pareja umbral y el kit esta entre las que mide el test del nivel, y la subida de congelados (T = 256) entra en ese test · §840: 482 -> 484, el censo de los native_leaf con la anchura de su identidad, y la guarda: ninguna estrecha ni el modulo que la usa llega a la produccion de las cuatro raices (la 51 de BACKLOG.md) · §842: 484 -> 485, el censo de las bandas de bytes: cada probador de stark-experiment que oculta tiene banda, y cada prueba del testigo de lo que revela una prueba cae en la de su familia (la 45 de la lista) · §816: 485 -> 496, la carrera de la E4 (RFC-0001) y el cerrojo del libro: una apertura y un recifrado a la vez no crean un libro vacio, una apertura que resuelve y un recifrado a la vez no se llevan el libro -con el retirado o con el recifrando de una muerte-, dos recifrados a la vez uno acaba y el otro no toca nada, una apertura en el tope del cerrojo da Io sin tocar nada -mientras otro recifra y mientras otro migra-, el cerrojo del libro es un fichero que nada renombra ni borra y que no se toma si no es un fichero, un libro nuevo en un directorio que no existe se abre como antes, la reentrada del cerrojo no se queda tras volver, un cerrojo que cambia mientras se toma no se sigue, uno que se borra o se cambia mientras se tiene no deja seguir, una ruta sin nombre no abre el libro sin su cerrojo, y el cerrojo del libro no cruza de hilo (SECURITY 3.14) · §817: 496 -> 497, con clave una instantanea en claro no se importa: la que sustituye a una cifrada, con otros saldos, falla con la frase buena y su error dice que esta en claro, y sin clave se importa (SECURITY 3.16) · §819: 497 -> 498, si la importacion no puede borrar el claro que dejaba una importacion muerta falla sin importar nada, nombra el fichero y no avisa de un borrado que no hizo, con el borrado mismo hecho fallar, y si puede lo borra y lo avisa una vez, nombrandolo y con EN CLARO (SECURITY 3.14) · §820: 498 -> 501, la exportacion de la instantanea, entera o nada (la 120): un error o una muerte inyectados a mitad dejan la anterior byte a byte y legible, el temporal de una muerte con una capa cifrada va cifrado como el destino y la siguiente exportacion rehusa nombrandolo, una que acaba deja la nueva, y la nueva conserva el dueno -con privilegios-, el grupo y los bits de modo de la anterior -sin su grupo, sin los bits de grupo ni de otros- y no sustituye un enlace ni un directorio · §821: 501 -> 503, la capa importada con clave conserva la clave (la 121): la de una cifrada de la version 2 se reexporta cifrada con la misma cabecera, abre con la misma frase y no sin clave ni con otra, con los saldos, el suministro y las cinco raices, y la cadena sigue, tambien tras abrir una cuenta en la capa importada; la de una v1, en la version 2 con sal nueva, otra en cada importacion y la misma en cada exportacion, y con su clave ya fijada sin la frase no se importa ni se toca · §828: 503 -> 515, el recifrado y la migracion v1->v2 rehusan el libro de otro dueno y conservan su grupo (SECURITY 3.17): como root, recifrar un v2 o un v1 de 12345:12346, asentados antes y con su cerrojo, rehusa antes de la pieza de la E4, y abrir con clave el v1 rehusa en su paso 0 o 1, con un error inyectado al acabar el paso 1 que la guarda adelanta -antes de copiar nada-, los tres con el texto que nombra a los dos usuarios y sin dejar hermanos, y el libro queda dueno a dueno como estaba y con lo mismo en su db; sin asentar y sin cerrojo, en el directorio de su dueno y con la umask 077, recifrar un libro de otro, o uno con su db, su conf o su blobs de otro, rehusa antes de tomar el cerrojo y de abrirlo, y no deja nada -ni un snap, ni el cerrojo, ni la sonda-, y su dueno lo abre despues; con el directorio de root y su db, su conf o su blobs de otro, tambien, el recifrado antes del cerrojo y la migracion en el paso 1, nombrando lo que es de otro, y la apertura no borra un .recifrando de root junto a un libro asi ni uno con su db de otro; recifrado o migrado por su dueno, root, un libro de 0:12346 lleva en la copia, al acabar el paso 4, el grupo y el modo de cada homonimo -el setuid de conf, el setgid del directorio- y de db la entrada sin homonimo, y los conserva, tambien el .recifrando que termina la apertura tras una muerte en el paso 5, y uno de otro grupo se termina con el suyo; sin poder poner el grupo, & 0o700; lo que lleva blobs en la copia toma lo de db, y un directorio en blobs hace rehusar; un enlace, un enlace duro, una entrada de otro dueno o la copia abierta a 0755 hacen rehusar sin tocar el destino, y la copia cambiada por otro directorio, entre crearla y compararla o al empezar el paso 4, tambien, sin darle a lo que lleva los modos del libro; y la apertura no termina ni borra un .recifrando de otro dueno; junto a lo que no es un libro no se crea la sonda, y la que no se puede borrar -un directorio en su sitio, o el padre con chattr +a- hace rehusar una vez, nombrandola, sin tomar el cerrojo; y si la sonda no se puede crear -un nombre en el que no cabe, o su nombre ya puesto, por un enlace simbolico a lo que no esta, uno a lo que esta o uno duro, que no sigue ni cambia-, su dueno lo recifra, y root, con el libro de otro, rehusa en el paso 1, de respaldo, con el libro como estaba, y deja su cerrojo · §711: 515 -> 518, toda verificacion ajena de la capa va dentro de la red del panico, la red la recoge como VerificationFailed, y un envio con cero consultas es Err con causa y no panico · §715: 518 -> 522, el reenvio con un usize escrito de mas o con un grupo FRI de mas no vale, la huella del contenido lo para y el verificador tambien, y tres bytes que hacian entrar en panico al lector son rechazo con causa · §716: 522 -> 530, una emision delegada no se reenvia cuando la raiz y el suministro vuelven: tras quemarla, en bucle con la reautorizacion legitima que si vale, a pendiente tras cobrarla y quemarla, a pendiente tras desemitirla, tras el reinicio y tras la instantanea, con su causa AutorizacionConsumida y con una autorizacion nueva que emite despues; una autorizacion vale para un solo valor del contador, y una sin contador ya no emite · §717: 530 -> 531, un libro cuyo nextPending pasa del tope de m no tiene prueba de edad y el productor lo dice antes de leer el libro, con la causa del juez · §718: 531 -> 546, la meta de los pendientes, el saldo, el importe pendiente y el suministro con su alias x + p no abren, la instantanea no importa, la colocacion salta el indice del centinela, y el productor de la edad no escribe un emisor que no es un indice; y lo que la cabeza firma sin raiz que lo cubra: el plazo de reembolso, el tope de custodios y los dos contadores con su alias no abren, el limite y el tope de cuentas no arrancan, la instantanea no importa ni esos ni una cuenta en el indice del centinela, y el ancho del indice del juez es el del arbol · §723: 546 -> 547, una hoja con un saldo de la ventana [2^62, 2^63) no abre del almacen, sola o con el suministro a la par, ni se importa de la instantanea; y los censos de las bandas (§842) y de las longitudes del cable (§839) cuentan la AIR con version de la banda y del cobro por su familia · §896: 547 -> 552, la cota de ocultacion medida sobre las 28 pruebas de los 23 probadores con fila -por columna y por trozo, 45 a enmascarar frente a T >= 64 y m = 64-, el rango pleno de sus matrices de Lagrange y de Vandermonde, el juguete con la geometria de produccion y sus dos mutantes con nombre, m = 43 y T = 32 (A1 del HVZK, RFC-0009 D-I) · §898: 552 -> 557, R3 medida por geometria y los tres falsadores del juguete (A3 del HVZK) · §899: 557 -> 558, el juguete de R3 con dos columnas aleatorias y una sobre E (la 133) · §900: 558 -> 561, la sonda de linealidad de las columnas reales sobre el juguete, M_lin medida en el juguete contra lo clavado de R3 y la tabla del censo de la 134 por familia (la 134), con los ignorados 7 -> 8 por su instrumento · §905: 561 -> 562, el juguete con la base fina: Λ sobreyectiva y la mascara que no baja de la gruesa (la 135) · §907: 562 -> 563, la instancia adversa de desafios -una colision de cosets, una coincidencia coset-tocado y una posicion siguiente de otra- fuerza las tres parejas en el juguete, la medida la admite y la huella de la composicion es estable (la revision del agente) · §908: 563 -> 564, con las dos columnas aleatorias pareadas como una sobre E las mascaras cubren K entero en el juguete y en las cinco geometrias, con la semilla 1 y con las posiciones adversas (el lema L9, la D-AL bis), con los ignorados 8 -> 9 por su instrumento de las geometrias grandes
stark-experiment   sello     490  13   0   600  alias=circuitos · los circuitos · §345: 297 -> 299, el gemelo nativo v2 (RFC-0003, E1a) · §346: 299 -> 304, RefundAirV2 en paralelo (E2a del RFC-0003) · §347: 304 -> 310, ClaimAirV2 en paralelo (E2b del RFC-0003) · §348: 310 -> 312, la puerta de E2 saldada (ladron-con-aviso y tercero-como-retorno) · §352: 312 -> 318, el envio aprende el sobre (E3c-1a del RFC-0003) · §463: 318 -> 328, el probador de la prueba de edad (RFC-0007, E4b-1) · §465: 328 -> 330, la cabeza fija la marca y el seq de la prueba de edad (RFC-0007, E4b-2) · §477: 330 -> 337, el probador de la banda del saldo (RFC-0007, E5, corte 4a) · §478: 337 -> 338, la boca del probador de la banda (RFC-0007, E5, corte 4b) · §490: 338 -> 348, el probador del cobro pendiente (RFC-0008, E1) · §495: 348 -> 350, la cabeza fija las raices y el nacido del cobro (RFC-0008, E1) · §503: 350 -> 364, el probador del pago en curso y sus catorce (RFC-0008, E2) · §511: 364 -> 379, el atado del camino de congelados a la cuenta (5.A-272, arreglo B) · §517: 379 -> 390, el probador de la prenda y sus once testigos, con los ignorados 12 -> 13 por su instrumento (RFC-0008, E3) · §529: 390 -> 391, el testigo del par ante una prueba de forma ajena, con la variante WrongTraceWidth (5.A-381) · §532: 391 -> 394, la foto del probador pristino: tres KAT con entradas fijas (RFC-0009, E3a-0) · §534: 394 -> 401, los siete falsadores de la ocultacion del nucleo, con el modo oculto solo en ellos (RFC-0009, E3a-2) · §537: 401 -> 403, los dos falsadores de D-AH: el probador oculto devuelve Err, no panico, con testigo malo o con m que no cabe (RFC-0009) · §641: 403 -> 404, la banda no se atribuye a un public_id ajeno, y el rango de los AIR baja a 62 bits, cerrando el wraparound (RFC-0017) · §651: 404 -> 405, el par umbral sin marca da WrongTraceWidth y no panico (D3-META) · §652: 405 -> 407, las semillas de la ocultacion son de 32 bytes: el ultimo byte mueve la prueba y la de produccion es fresca (CRIPTO-01) · §680: 407 -> 417, los dos acumuladores suman el bit que sube: ataques F e I en los cinco circuitos (RFC-0018 E1) · §683: 417 -> 420, las derivaciones anchas de custodios, gobernadores y gasto generalizan las estrechas (RFC-0018 E2) · §684: 420 -> 423, el umbral con nulificador lleva la clave entera y nulifica en dos hashes (RFC-0018 E2) · §713: 423 -> 425, el resto FRI atado a su compromiso: una prueba que se compromete con el hash de OTRO resto, fabricada sin tocar el probador -un hasher que cambia esa llamada-, sobre WorkAir y sobre la prueba de edad oculta y con sal, cae con RemainderCommitmentMismatch · §715: 425 -> 427, una fila o un grupo FRI de mas con el compromiso de Merkle de upstream, sin sal, los para solo el canal del fork: TraceQueryDoesNotMatchCommitment y LayerCommitmentMismatch, y sin el canal verifican · §717: 427 -> 428, el probador no prueba un libro sobre el tope de m, con la causa del juez · §719: 428 -> 475, ninguna restriccion vacia en el envio, el cobro y la apertura v2 que produce la capa, ni en la subida del credito ni en la apertura v1 que usa al des-emitir, ni en las 110 principales de la edad; cada segmento de rango de los diecisiete circuitos, el del tope de la emision a pendiente y de su subida, y los de solvency fuerzan a cero sus dos bits altos y admiten el tercero; el tope de la emision a pendiente baja a 62 bits y un suministro negativo con el margen en la ventana alta no se prueba ni verifica; un envio y una quema que envuelven, y un importe o un saldo de 2^62, no se prueban en el envio v2, el cobro v2 ni la quema, con la fila que los para, y con el probador sin ocultar los rechaza el verificador; y range_check y solvency rechazan la ventana alta · §720: 475 -> 478, ninguna de las 9 restricciones auxiliares de la edad esta vacia, en los dos modos del enunciado, el detector del tramo auxiliar nombra una anulada y solo a ella, y un pendiente vivo sin meta que se apaga en la fila 6 de su ciclo no se cuenta: lo tumba la constancia de vivo y nada mas · §721: 478 -> 479, el probador rechaza m = 14 con la causa del juez por los tres caminos: el libro, el m declarado y la forma de la traza · §723: 479 -> 488, la banda y el cobro pendiente v2: cada segmento fuerza a cero sus dos bits altos y la v1 solo el alto; el probador de la casa no prueba un valor de la ventana; con un probador que no comprueba la traza y recorta el cociente, el juez del kit lo rechaza por la coherencia OOD; una prueba de la v1 se juzga por su superior; y el KAT de la banda v2 · §899: 488 -> 489, el falsador de la ocultacion de laboratorio con dos columnas (la 133) · §906: 489 -> 490, el falsador de la marca v2 de laboratorio con el byte de columnas (RFC-0018 D-G)
ceremony           sello      34   0  11   300  alias=ceremonia · DEUDA: 11 warnings, pinchados para que no crezcan
settlement-layer   sello      17   0   0   300  alias=liquidación · §795: los dos setups Groth16 de SettlementLayer::new se pagan una vez por suite y no una por test, con un OnceLock en el modulo de tests y sin tocar produccion (la 75): la CPU de la fila (user+sys) baja de 206 a 93 s y la pared de 102 a 65 s, segunda de dos corridas con el binario ya compilado y la maquina cargada (carga 5,5 a 10,6 en 4 nucleos), asi que el dato es el de CPU · §845: timeout 300, sin cambio, MEDIDO con un solo nucleo (taskset -c 3) y la maquina en reposo (carga de 1,0 a 1,1), la orden de la fila suelta: tras un cambio en zk-core, que la fila recompila con la liquidacion, 89,6 y 87,6 s sobre 77bca6c y 115,2 s sobre 69597f3, de 30 a 36 s de compilar y de 58 a 79 s de tests; en caliente, 59,0 y 60,9 s. 300 cubre los 115 s con algo mas de un tercio de nucleo
iso-bridge         sello       3   0   0   300  §822: el arbol de Merkle de zk-core no materializa el relleno (la 75): sus 3 tests, de 53 y 58 s de pared a 3 y 4, con el binario compilado y la maquina cargada (15-10-2026); el --sello de 0315b14 le dio 52 s. El timeout se queda en 300
zk-ssl-sdk         sello      22   0   0   300  §542: 6 -> 11, la prenda desde el Wallet, con la clave que no sale del crate (RFC-0008 E3, D-BH) · §550: 11 -> 13, el lector del keystore nombra el fichero y comprueba sus permisos (5.A-403, 5.A-404) · §606: 13 -> 15, el titular guarda la constancia -el recibo y el acuse de la respuesta, tal como llegaron- (la 109) · §702: 15 -> 20, el keystore v2 con sal y coste (RFC-0001): lo que se escribe, el v1 de 590caae que se abre y queda migrado, el v1 que con otra frase no se toca, lo que la v2 declara y se comprueba, y los KAT de la clave del keystore · §774: 20 -> 21, guardar barre el `<keystore>.escribiendo-<pid>` de un proceso muerto · §775: 21 -> 22, el `logSeq` de la constancia, en forma canonica
zk-ssl-wire        sello      27   0   0   180  §259: 20 metodos, y el json publicado atado a la tabla · §311: 3 -> 7, el DTO de la cabeza firmada · §312: 7 -> 10, el accesor falible de la cabeza firmada · §313: 10 -> 13, los tres constructores nombrados · §315: 13 -> 15, la cofirma en el cable · §415: 15 -> 17, la cabeza v4 en el cable y la vista por version (RFC-0006, E2b) · §434: 17 -> 19, del cable a la cabeza (RFC-0006, E4b-1b-i) · §452: 19 -> 22, la cabeza v5 en el cable y firmada() por version (RFC-0007, E1b) · §533: timeout 60 -> 180; su clausura lleva el probador por path y, tras un cambio en winter-*, el cable paga ~60 s de compilacion antes de sus 2 s de tests (exit 124 en el primer BLOQUE-533) · §570: 22 -> 23, la cabeza v6 en el cable y la vista por version (RFC-0010, E2d) · §641: 23 -> 24, el saldo del cliente se lee canonico en el cable (RFC-0017) · §650: 24 -> 25, el DATA del cable es el hex canonico, un multibyte es BadHex y no un panico · §662: 25 -> 26, la QUANTITY del cable sin +, sin mayusculas y sin ceros a la izquierda · §699: 26 -> 27, el aviso de compatibilidad del OpenRPC va con su version
zk-ssl-cli         sello      175   0   0   600  alias=testigo · §250: 22 -> 26, la vista dividida EN FRIO · §294: 28 -> 39, la historia como segundo canal · §295: 39 -> 43, el banco del consumidor · §299: 43 -> 51, el cofirmante del testigo · §300: 51 -> 54, el mando de la cofirma · §301: 54 -> 58, la herramienta del tercero · §310: 58 -> 59, la serie de cofirmas es por testigo · §312: 59 -> 61, el testigo consume la cabeza tipada · §316: 61 -> 62, el testigo envia su cofirma al nodo · §319: 62 -> 66, la politica que nombra y la k que falla cerrada · §320: 66 -> 71, la recoleccion de cofirmas · §314: 71 -> 75, la ausencia de respuesta es una clase · §333: 75 -> 79, la clave de la serie pasa al indice embebido · §334: 79 -> 81, la marca solo se dispara sobre lo que prueba · §336: 81 -> 86, la politica del cofirmante sale de fn run y nace con tests · §337: 86 -> 91, el testigo vuelve a arrancar tras cofirmar · §404: 91 -> 92, otra version no se cree · §406: 92 -> 93, el testigo lee la version por VersionCabeza antes de la firma · §409: 93 -> 94, el mando de --respuesta y el texto del veredicto · §411: 94 -> 95, los KAT del nucleo · §414: 95 -> 96, el canal de la historia ve la v4 (RFC-0006, E2a) · §415: 96 -> 97, el diario custodia la pareja de consumos (RFC-0006, E2b) · §451: 97 -> 98, el testigo recompone la v5 (RFC-0007, E1a) · §452: 98 -> 100, el diario custodia la v5 y su lista es la del cable (RFC-0007, E1b) · §497: 100 -> 107, la boca del cobrador, prueba-cobro (RFC-0008 E4-cobro, corte A) · §507: 107 -> 113, la boca del pagador, prueba-pago (RFC-0008 E2, D-AI) · §543: 113 -> 120, la boca del prendador, prueba-prenda, y la puerta de clausura del cli (RFC-0008 E3, D-BC..D-BI; 5.A-359) · §544: 120 -> 124, el keystore del receptor como quinto fichero de la siembra (RFC-0008 E3) · §559: 124 -> 125, la guarda del cobro pasa al predicado de la familia (RFC-0010 E2a) · §594: 125 -> 128, el gate de las cofirmas en todo estado que lea un contador (ECST §8.1): un positivo, dos rojos · §635: 128 -> 131, el juez del umbral del medio con la politica del cliente: cuenta testigos nombrados y distintos, no cuenta los que no verifican ni los repetidos, y falla cerrada bajo k y con k = 0 (RFC-0013 D-D) · §650: 131 -> 132, el testigo no cae ante un hex con un multibyte · §647: 132 -> 135, el cambio de clave explicado con las actas, el que no se explica y el auditor que rejuzga la rotacion (RFC-0015, E4) · §687: 135 -> 136, el testigo arma el sobre del solapamiento desde su diario (RFC-0015, E5c) · §688: 136 -> 137, la vista dividida del diario sale como el sobre del ancla, modo 4 (RFC-0011) · §689: 137 -> 138, dos diarios dan un sobre de la vista dividida por indice (RFC-0011) · §692: 138 -> 139, emitir los KAT del nucleo no reescribe uno publicado · §709: 139 -> 141, dos cofirmantes sobre el mismo contador y el segundo no arranca, y el contador del cofirmante con su ruta absoluta (SEC-1) · §770: 141 -> 143, el auditor da la hoja embebida mas alta por clave, lo que pide --desde-minimo (RFC-0015 D-G, la 92 c) · §772: 143 -> 145, el buffer del SK del cofirmante es Zeroizing: se borra por cualquier salida y no se reubica (la 102) · §775: 145 -> 147, el testigo lee los u64 del nodo en forma canonica: `leer_q` y el indice de `--ausentes` · §780: 147 -> 148, la cofirma con versionFormato que no cabe en un byte no se trunca · §781: 148 -> 151, conformance --emit y el emisor de KAT no pisan un vector registrado en HUELLAS.sha256 · §785: 151 -> 152, la epoca de --recolectar-cofirmas se pide por su nombre, en forma canonica · §802: 152 -> 153, abrir dos veces la misma clave en el sandbox es un error con su causa, no un panico (SECURITY 3.13) · §845: timeout 120 -> 600, MEDIDO con un solo nucleo (taskset -c 3) y la maquina en reposo (carga de 0,1 a 1,9), la orden de la fila suelta: tras un cambio en zk-ssl, que la fila recompila con zk-ssl-wire, zk-ssl-sdk y el cli, 136,4 y 136,6 s sobre 77bca6c y 163,4 s sobre 69597f3, de 2 min a 2 min 21 s de compilar y de 14 a 22 s de tests; tras uno en winter-air, once crates, 221,9 y 226 s sobre 77bca6c y 268,7 s sobre 69597f3; en caliente, de 15,7 a 20,4 s. Con 120, la fila tras el cambio en zk-ssl da exit 124. 600 cubre los 269 s con medio nucleo y los 163 con un tercio; el margen se remide cuando la fila crezca · §712: 153 -> 167, el testigo cuenta por el indice embebido -una cuenta por operador- en la vuelta viva, en --auditar, en --comparar y en --ausentes, y aplica el reloj del kit a cada cabeza que llega: el diario_B, dos testigos, el recorte que detiene, el reseteo visible que no, los huerfanos, la cabeza posterior que firma por debajo de la custodiada y no se cofirma, el reseteo visible que --comparar no llama bifurcacion y el por-detras del auditor con la pareja del diario (RFC-0012 D-C) · §714: 167 -> 173, la bifurcacion con el mismo mmrSize detiene con cualquier cima y por detras, con su clase en el diario (v5): la vuelta viva con otra cima, la historia del operador que la rotacion no borra, la linea de la clase nueva, el auditor por operador con el genesis, --comparar con la cadena de los diarios, y --comparar que solo cree las cabezas que el testigo verifico · §718: 173 -> 175, el lector de cantidades del testigo es el del kit: los ocho enteros de la cabeza con su alias no verifican y la historia no ancla en t = p
zk-ssl-node        sello      244   0   0   400  alias=nodo · §261: 51 -> 56, la credencial para los caminos · §292: 79 -> 80, la cima en el digest · §293: 80 -> 82, la extension servida · §296: 82 -> 73, el guardian se muda a su crate · §298: 73 -> 71, la lectura del indice se muda · §302: 71 -> 72, el contrato publicado atado al despacho · §309: 72 -> 73, el conjunto de claves servidas, declarado · §315: 73 -> 76, el transporte de la cofirma · §317: 76 -> 77, la retencion de las cofirmas, atada · §318: 77 -> 80, el disparador de la acumulacion · §328: 80 -> 83, el arranque reconcilia · §331: 83 -> 84, la clave en cero no arranca · §335: 84 -> 91, la clave vuelve a su indice y el contador se ata al diario · §417: 91 -> 95, el consumo por el cable (RFC-0006, E3a) · §436: 95 -> 98, la puerta de libros ajenos (RFC-0006, E4b) · §440: 98 -> 100, los dos testigos de la frontera (RFC-0006, E4b-2) · §452: 100 -> 101, el nodo sirve la cabeza v5 y lo servido recompone (RFC-0007, E1b) · §454: 101 -> 104, la causa en el cable y en la negativa del consumo (RFC-0007, E2) · §458: 104 -> 108, el camino de congelados y dev_freeze (RFC-0007, E3b) · §493: 108 -> 112, la foto en el latido y zkssl_pendingPath (RFC-0008, D-F) · §505: 112 -> 115, la puerta del pagador en zkssl_pendingPath (RFC-0008 E2, D-AE) · §519: 115 -> 122, el cable de la prenda, zkssl_pledge (RFC-0008, E3) · §530: 122 -> 125, la red del panico: la PARADA con causa y sus tres testigos (5.A-382) · §559: 125 -> 126, la guarda del nodo sale a su fn y pregunta por el predicado (RFC-0010 E2a) · §565: 126 -> 137, el registro de recepcion y su reconciliacion al abrir (RFC-0010, E2c-1) · §568: 137 -> 146, la vista de recibos (RFC-0010, E2c-2) · §569: 146 -> 155, el registro conectado: anota al recibir y se reconcilia al arrancar (RFC-0010, E2c-2) · §570: 155 -> 160, el nodo compone, firma y sirve la cabeza v6 (RFC-0010, E2d) · §571: 160 -> 165, el recibo de recepcion en el cable y su camino (RFC-0010, E3) · §573: 165 -> 166, el arbol de recibos atado a RECEP_DEPTH (RFC-0010, E4) · §576: 166 -> 167, el alcance del recibo, decidido y atado (RFC-0010, D-E) · §594: 167 -> 171, el gate del diario en todo estado que lea un contador (ECST §8.1): dos positivos, dos rojos · §611: 171 -> 173, el lote y la prenda evaluada llevan su recibo, y el testigo del §576 cae al reves (RFC-0014, E3) · §636: 173 -> 175, la firma a demanda que no existe y el diario que si dice de que clave es cada linea, atados · §638: 175 -> 176, la clave nueva que sigue la cuenta de la vieja (el alcance de lo que dijo el §636) · §650: 176 -> 177, un hex con un multibyte es -32602 y el nodo sigue, sin PARADA · §655: 177 -> 179, un fallo del almacen pone el nodo en PARADA, y un rechazo de otra causa no (SEG-06 c) · §666: 179 -> 180, un byte ilegible en el diario se lleva su linea y no el diario · §667: 180 -> 181, con las hojas del MMR envenenadas no hay pareja ni latido · §672: 181 -> 182, el arbol de la epoca con rebuild_from, la misma raiz que hoja a hoja · §673: 182 -> 183, la frontera del MMR da la cima de todas las hojas · §674: 183 -> 184, el indice del diario da lo mismo que leerlo entero · §644: 184 -> 188, el nodo firma sus actas: la decision al arrancar, el arranque entero con claves de verdad, el aviso de agotamiento y la linea del acta que ningun lector de cabezas ve (RFC-0015, E3a) · §645: 188 -> 190, la vieja firma el acta de rotacion y el firmante no reserva ni firma en el techo (RFC-0015, E3b-1) · §646: 190 -> 191, el cable sirve la cadena de actas y el kit la juzga (RFC-0015, E3b-2) · §676: 191 -> 192, el arbol del MMR da el camino de consistencia de todas las hojas · §677: 192 -> 193, el tramo de la epoca da la misma raiz y el mismo camino que el registro entero · §678: 193 -> 194, zkssl_verifyChain como mucho una vez por intervalo · §690: 194 -> 195, la rotacion salta por encima del desde minimo (RFC-0015, D-G) · §709: 195 -> 199, al firmar los dos contadores van con ruta absoluta y sin valor por defecto, decidido en frio y en las banderas, y dos arranques sobre los mismos contadores: el segundo no abre (SEC-1) · §772: 199 -> 201, el buffer del SK es Zeroizing al resincronizar y al firmar con la anterior: se borra por cualquier salida y no se reubica (la 102) · §780: 201 -> 202, zkssl_submitCosig rechaza la versionFormato que no cabe en un byte · §783: 202 -> 203, dos nodos sobre el mismo libro: el segundo se niega con nombre (la 18) · §784: 203 -> 204, el libro se constituye con las raices de --custodian-root y --governance-root (ARQ-02) · §802: 204 -> 206, abrir por el cable una identidad con cuenta es -32000 con su causa, y dev_openSeeded repetido rechaza sin parar el nodo (SECURITY 3.13) · §813: 206 -> 210, el nodo espera al cerrojo del libro, acotado -diez miradas cada 50 ms, las de cerrojo_libre-: tomado 200 ms y suelto abre a la quinta mirada, tomado mas que el tope se niega con su nombre, las dos causas y OTRA clave para otro libro, sin tocar un byte, ningun otro error se espera, ni un Io de otra causa ni un flock sin WouldBlock, y open_layer tarda al menos 450 ms en negarse (la 18) · §815: 210 -> 229, el ORDEN del recibo (privado, F1): juzgar y despues anotar en los cuatro brazos, el corte tras juzgar no deja hoja y deja lo aplicado en el libro, el corte tras anotar deja la hoja con su veredicto ya en el libro, el rechazo cortado igual, la prueba basura de un tercero sin cuenta se lleva su recibo ya juzgada, el lote, la prenda y el cobro cortados, la era fijada al recibir, el panico y el almacen a media evaluacion sin hoja, el registro que falla tras el veredicto da el veredicto sin recibo y PARA (sustituye a «si anotar falla la operacion no se evalua»), la marca de fallo en vuelo bajo el candado del estado (A5) y el latido que no firma con ella ni en PARADA (F2), y publishConsumo y la prenda con un Store van a la PARADA; y, tras la revision, el desenlace desconocido como funcion del TIPO del juicio de cada brazo, la marca que solo se pone y se mira con el candado del estado tomado, y la carrera A5 con hilos reales · §863: 229 -> 230, el nodo responde lo que dice cada vector de spec/vectors/nodo/, por la funcion del transporte y sin --dev, y el primero es el del §802: abrir por el cable una identidad con cuenta es -32000 con su causa IdentityUnavailable (la 90, la R6 f del RFC-0021) · §711: 230 -> 234, un applySend y una prenda con cero consultas son rechazo con causa y el nodo no pasa a PARADA, y la unica prueba ajena que el nodo juzga por su cuenta va dentro de la red del panico, con su censo y el testigo de que el censo distingue · §714: 234 -> 236, con su diario el nodo no repite un mmrSize, y sin el si: la siembra del MMR sale de main a su funcion y se ata; y el hueco declarado: si diario::anotar falla, el nodo repite un mmrSize · §715: 236 -> 237, el reenvio maleado por zkssl_applySend tras un ciclo A->B->A no debita, y las particiones a cero no llevan el nodo a PARADA · §718: 237 -> 239, --edad-emisor es un indice de cuenta, y --limit y --max-accounts no arrancan con su alias · §722: 239 -> 240, --relevo-minimo: con un diario por detras de lo que la vieja firmo, la rotacion no se firma ni gasta, y reconstruido su relevo es el numero de cabezas · §866: 240 -> 244, el latido en su hilo y no en el ejecutor (la 123): la firma se pide desde el hilo «latido», el RPC contesta por TCP con la firma retenida, ningun candado del nodo esta tomado mientras se firma, y con ella retenida el despacho lee y escribe y lo escrito entra en la cabeza siguiente
zk-ssl-verify      sello      225   0   0   120  alias=verificador independiente · §257: 22 -> 23, del cable a la cabeza sin la capa · §291: 39 -> 46, el MMR de cabezas · §292: 46 -> 48, la cima en el digest · §297: 48 -> 55, el objeto de la cofirma · §322: 55 -> 61, las cofirmas dentro del paquete de evidencia · §324: 61 -> 62, la mudanza de COFIRMA_VERSION · §332: 62 -> 66, el indice declarado se ata al que va dentro de la firma · §335: 66 -> 69, el apano del OID pasa a tener un solo dueno · S395: 69 -> 72, la autonomia del verificador deja de ser prosa · §399: 72 -> 75, el indice declarado de la cabeza se ata al que va dentro de la firma · §406: 75 -> 79, el conjunto de versiones tiene un solo productor · §414: 79 -> 82, la cabeza v4 en el nucleo y el mando (RFC-0006, E2a) · §419: 82 -> 88, las reglas del arbol de consumos en el verificador (RFC-0006, E3b-2) · §430: 88 -> 92, el lector del conflicto entre libros (RFC-0006, E4a-2b) · §451: 92 -> 96, la cabeza v5 en el nucleo y el mando (RFC-0007, E1a) · §458: 96 -> 100, las reglas del arbol de congelados (RFC-0007, E3b) · §465: 100 -> 104, el kit verifica la prueba de edad sin el probador (RFC-0007, E4b-2) · §475: 104 -> 108, las reglas del arbol de cuentas y el noveno brazo del mando (RFC-0007, E5, 3b) · §495: 108 -> 112, el brazo del cobro pendiente en el mando (RFC-0008, E1) · §506: 112 -> 116, el brazo del pago en curso en el mando (RFC-0008, E2) · §520: 116 -> 120, el octavo brazo del mando, `tipo: "prenda"` (RFC-0008, E3) · §558: 120 -> 121, la variante V6 entra en el conjunto y los sobres preguntan por el predicado (RFC-0010, E2a) · §562: 121 -> 129, las reglas del arbol de recibos de recepcion · §566: 129 -> 132, el acuse y la inclusion contra la cabeza v6 (RFC-0010, E2a) · §567: 132 -> 133, el rx que no pertenece no tiene posicion: la era (Q, R] (RFC-0010, E2b) · §573: 133 -> 139, el sobre de completitud: sus veredictos, el cuarto estado y el camino del recibo (RFC-0010, E4) · §586: 139 -> 140, el ancho del indice sale del conjunto de xmss (entrada 101) · §592: 140 -> 144, el sobre del ancla: sus tres modos y la vista dividida (RFC-0012, E3) · §612: 144 -> 146, la resolucion del lote: su composicion y su forma juzgada otra vez (RFC-0014, E4a) · §613: 146 -> 147, la resolucion de la prenda se ata por su prueba y lleva una de tres (RFC-0014, E4b) · §633: 147 -> 152, el sobre del ancla cofirmada (RFC-0013, E4a): la forma, una cabeza firmada de verdad con su ancla en el medio, cada mentira por su nombre -el medio ajeno bien firmado incluido-, el desconocido que lo enumera y el kit que verifica el medio sin firmar · §640: 152 -> 157, el mando lee canonico -un u64 y un elemento menores que p- y la particion del MMR es total (RFC-0016) · §650: 157 -> 160, el kit lee el hex canonico: un multibyte es ROJO y no 101 · §651: 160 -> 165, las cinco familias del kit con el meta vaciado son ROJO, no 101 (D3-META) · §653: 165 -> 166, una prueba con bytes de cola es ROJO (SEG-04) · §662: 166 -> 167, cada u64 del sobre en su escritura minima · §664: 167 -> 168, una clave publicada con el OID ya apanado no se lee · §643: 168 -> 172, el acta de clave: la genesis, la rotacion comprometida y un rojo por regla (RFC-0015, E2) · §646: 172 -> 173, el acta en JSON, con un productor (RFC-0015, E3b-2) · §647: 173 -> 174, la rotacion juzgada con la cadena y el solapamiento con nombre (RFC-0015, E4) · §681: 174 -> 175, la ventana se mide con el indice que la firma acredita (RFC-0019 E1) · §682: 175 -> 176, un StaleState no resuelve solo (RFC-0019 E2) · §648: 176 -> 178, la continuidad de dos claves con su cadena de actas, en el nucleo y en los sobres (RFC-0015, E5a) · §686: 178 -> 181, el sobre del solapamiento: los tramos de una clave en el nucleo, el sobre y su tipo (RFC-0015, E5c) · §694: 181 -> 182, las fijaciones exactas en el lock y en cada manifiesto, y la clausura del kit pasa a lista cerrada · §782: 182 -> 185, verificar_inclusion exige el camino entero de su cuenta (RFC-0016 E6) · §800: 185 -> 190, el registro de esquemas y los dos preambulos v7, apagados: el registro sin el cero y con la entrada del acta, ninguna escritura de clave compartida, lo que declara es lo que xmss escribe y una firma sobre el v7 no la acepta nadie, y los dos preambulos byte a byte (RFC-0020, E2) · §815: 190 -> 191, VerificationFailed y ProofFailed se declaran -el cuarto estado, salida 3, en la via directa y en el lote- y su sobre de rechazo sigue sin probarse, sobre los tres vectores nuevos de completitud (kit 0.4.3; RFC-0007 D-I, RFC-0010 D-G; privado, F3): la lista de las causas sin prueba portable pasa de cuatro a seis, y su test literal se renombra (las_seis) · §711: 191 -> 196, la puerta de regresion: cabecera, forma y azar de las pruebas de edad, pago, prenda y cobro pendiente son Err y nunca panico, una capa FRI de menos o de mas es Err, y un panico en la frontera del mando es ROJO · §712: 196 -> 199, el reloj de la extension, del consumo y del ancla del modo 3 -el mismo digest con el mismo mmrSize y la hoja que crece con la misma clave-, sobre los nueve vectores nuevos y puro (RFC-0012 D-C) · §713: 199 -> 201, el resto FRI atado a su compromiso: el vector capturado de edad cuyo ultimo compromiso FRI es el hash de OTRO resto, y un bit del resto de las cuatro pruebas de la puerta del §711, caen por RemainderCommitmentMismatch, antes que por el plegado · §714: 201 -> 203, el modo 5 del ancla delata la bifurcacion: los doce sobres nuevos y los papeles cambiados; y la forma por presencia y la rama como sujeto: los nueve sobres nuevos · §715: 203 -> 208, una prueba tiene una sola escritura: cada usize de las cuatro pruebas de la puerta escrito de mas, una fila, un grupo FRI y un nodo de mas, y las restricciones mas 2^32, son ROJO con su causa · §716: 208 -> 210, el reverificador da ROJO a un compromiso repetido donde lleva contador -una congelacion siempre, una emision desde la entrada que se declara- y Parcial donde no · §717: 210 -> 211, los tres paquetes de atacante -cabeza con firma propia y prueba fabricada- caen con su causa y en el tiempo de uno honesto · §718: 211 -> 215, el acta lee sus cantidades menores que p, y la API de pago, cobro y edad rechaza el alias x + p con las pruebas reales de los vectores · §722: 215 -> 218, el relevo ata la cadena de actas al MMR de las cabezas -cada cabeza en el relevo de su tramo por sus dos bordes, la anterior en un tramo de su clave, los relevos no retroceden-, en el nucleo con la escena y con una cadena alternada A->B->A->B, y sobre los trece vectores de la historia comun · §723: 218 -> 220, una banda de la v1 con el superior bajo la cota no es VERDE sino el cuarto estado, la de la hoja fabricada, salvo la ya publicada, que la lista reconoce; la de la v2 maleada, ROJO; el control y la captura de la v2, VERDE; y en completitud el cuarto estado de la banda va despues de las comprobaciones de cada brazo: la operacion fuera del lote y la otra marca de la prenda siguen siendo ROJO · §862: 220 -> 225, el despacho por esquema (RFC-0020, E3): el conjunto aceptado y el emitido, atados, el implicito para todo byte de version, el esquema fuera del registro y el que no se acepta rechazados con su numero antes de leer la clave, y la firma de la clave que se va juzgada con el esquema del acta previa
zk-ssl-hash        sello      57   0   0    60  §270: 14 -> 16, el acuse componible por un tercero · §292: 22 -> 24, la cima en el digest · §414: 24 -> 26, epoch_digest_v4 (RFC-0006, E2a) · §416: 26 -> 27, la posicion del consumo en el nucleo (RFC-0006, E3a) · §451: 27 -> 31, epoch_digest_v5 y params_digest (RFC-0007, E1a) · §557: 31 -> 34, epoch_digest_v6, recibo_digest y el dominio RECEP_V1 (RFC-0010, E2a-0) · §591: 34 -> 37, ancla_digest, huella_de_clave y el dominio ANCLA_V1 (RFC-0012, E2) · §610: 37 -> 40, la huella del lote, con dominio propio y longitud codificada (RFC-0014, E2) · §640: 40 -> 45, la canonicidad del nucleo: el modulo del campo, la lectura canonica y la biyeccion de la escritura (RFC-0016) · §650: 45 -> 49, hex_canonico y bytes_de_hex sobre bytes, y la puerta que no deja trocear un &str · §662: 49 -> 50, la QUANTITY en su escritura minima: cantidad_canonica · §643: 50 -> 52, acta_digest y el dominio ACTAS_V1 (RFC-0015, E2) · §773: 52 -> 56, digest_pi y la hoja RECEP_V2 (RFC-0019, E3, primera pieza) · §716: 56 -> 57, el compromiso de emision de doce elementos y el de catorce, con el contador, no colisionan, y con [0, 0] detras si
zk-ssl-guardian    sello       40   0   0    60  §296: el guardian del indice, MUDADO del nodo (sus 9 tests salen del pin 82) · §298: 9 -> 11, y llega con sus dos tests de layout · §330: 11 -> 19, el lector comun de la semilla · §331: 19 -> 22, ClaveEnCero y su guarda · §335: 22 -> 25, el escritor del indice y su techo derivado del ancho · §336: 25 -> 26, el invariante del arranque tiene dueno · §594: 26 -> 27, un contador borrado reabre en cero y COINCIDE con la clave en cero: la premisa del ECST §8.1, medida · §650: 27 -> 28, la semilla se lee sobre bytes: un multibyte o un + se rechazan sin panico · §690: 28 -> 29, adelantar el contador sin firmar, y nunca hacia atras (RFC-0015, D-G) · §709: 29 -> 35, un solo proceso por contador (SEC-1): dos guardianes sobre la misma ruta y el segundo no abre, diciendo quien lo tiene; el cerrojo es del fichero y no del nombre; un enlace simbolico no se sigue; y al firmar, la ruta del contador es obligatoria y absoluta · §740: 35 -> 40, en Linux tmpfs se niega por el tipo del sistema de ficheros y no por el reloj, y la medida de fsync va con la mediana: el falsador de la mediana, la mediana que no rechaza un disco, el reloj que no mide, la segunda red cableada y el lector de mountinfo; y en_tmpfs_se_niega_a_operar, que con el df de GNU se saltaba siempre, prueba de verdad en Linux
zk-ssl-medio       sello      62   0   0    60  §631: nace con 25, el arbol del medio del ancla -SHA-256 de RFC 6962, copiado de mtc-core d3b0ca6 con su procedencia-: los 65.058 casos acumulados del IETF, sus vectores grandes hasta 2^64-1 hojas, las 685 sondas de transparency-dev/merkle (43 positivas, 4 de ellas sin tipo; 642 negativas, ninguna aceptada) y 46 negativas de 32 bytes hechas de sus positivas, que tapan el punto ciego medido: sin comparar la raiz vieja se pasaban las 685 (RFC-0013, E2a). 6 s en frio · §632: 25 -> 37, la nota del medio (RFC-0013, E2b): la base64 estricta de mtc-core, la nota checkpoint firmada con ML-DSA-44 tipo 0x06 y 33 notas contrastadas con torchwood v0.10.0 sobre el ML-DSA de Go 1.27 -la linea del publicador sale byte a byte, ninguna negativa que torchwood rechace la acepta el medio, y 4 que torchwood acepta caen aqui por tres reglas declaradas-; 11 s en frio con ml-dsa · §808: 37 -> 58, el lado del publicador (RFC-0013, E3a, SOLO EN LOCAL): el cliente add-checkpoint de tlog-witness v1.1.0, la retencion de las cofirmas 0x06 y la nota cofirmada publicada sin retroceder (10 unitarios); 8 en 127.0.0.1, 6 contra un testigo minimo de test que responde por trozos (la 103 y la bifurcacion entre ellos) y 2 contra un servidor de un solo uso que mide el transporte, el que no acaba de contestar y el que no para de mandar; y 3 que rehacen byte a byte lo que contestaron el motor de litewitness con ML-DSA-44 y litewitness v0.10.0 (torchwood), grabado por tests/vectores/testigos/rehacer.sh, fuera del canon · §848: 58 -> 62, el mismo cliente contra omniwitness (transparency-dev/witness 5397155) con el log 0x06 del medio configurado a mano: 4 que releen lo que contestaron omniwitness-a, con una clave ML-DSA-44, y omniwitness-b, con una ML-DSA-44 y una Ed25519 del mismo nombre, a diez pasos -cada peticion byte a byte, cada resultado, cada cofirma ML-DSA-44 verificada aqui, la Ed25519 ignorada, la vkey de cada semilla igual a la de Go y el sobre del banco-, grabado por tests/vectores/testigos/rehacer_omniwitness.sh, fuera del canon
zk-ssl-air         sello      66   0   0   180  §463: nace con 4, el AIR de la prueba de edad y su juez, sin el probador (RFC-0007, E4b-1) · §465: 4 -> 5, la m de una marca es la minima (RFC-0007, E4b-2) · §477: 5 -> 10, el AIR de la banda del saldo, sin titularidad (RFC-0007, E5, corte 4a) · §490: 10 -> 15, el AIR del cobro pendiente, sin el probador (RFC-0008, E1) · §495: 15 -> 16, el enlace del cobro pendiente a la cabeza v5 (RFC-0008, E1) · §503: 16 -> 22, el AIR del pago en curso y sus seis testigos (RFC-0008, E2) · §516: 22 -> 29, el AIR de la prenda con su juez y su enlace, sin el probador (RFC-0008, E3) · §535: 29 -> 36, la sal como tipo -MerkleConSal y sus siete testigos-, sin encender nada (RFC-0009, E3b-0) · §575: 36 -> 40, la lectura de una prueba no reserva lo que sus bytes no traen: el lote con sal y Proof::from_bytes · §578: 40 -> 41, toda verificacion del kit lee con sal: las cinco · §659: 41 -> 42, un nodo de mas en el lote no verifica: cada vector trae los nodos que la raiz consume (SEG-03b) · §697: 42 -> 43, el piso de la LDR de la edad para cada m que el enunciado admite, y m = 24 declarada sin piso (ZK-1 de blueprint-v2) · §708: 43 -> 44, la conjeturada con el termino DEEP, para cada m, con su replica contrastada con la funcion -con blowup 16 y 2- y con el fuente del fork · §853: timeout 60 -> 180, MEDIDO con un solo nucleo (taskset -c 3) y nada mas compilando (carga de 1,0 a 1,5), la orden de la fila suelta sobre 378e1fa: lo caro es compilar zk-ssl-air, unos 50 s; tras un cambio en zk-ssl-hash, 51,2 y 52,5 s (dos crates), y 52,9 y 54,8 s con winter-air y winter-verifier por recompilar (cuatro); tras uno en winter-air, tres crates, 53,4 s; los tests, 0,01 s; en caliente, 0,2 s. Con medio nucleo la fila tarda 106 s, y con 60 da exit 124. 180 cubre la peor con medio nucleo, 110 s, con la deriva del 29 % que el §845 midio entre dos tandas (141 s), y con un tercio de nucleo (164 s); el margen se remide cuando la fila crezca · §711: 44 -> 46, la red del panico de quien juzga una prueba ajena, y un lote de mas de 31 niveles es error con causa y no panico · §715: 46 -> 48, el lote con sal lee sus cuentas en su escritura minima, y un item de mas con su sal no verifica · §717: 48 -> 51, el tope de m, 13: sobre el tope el enunciado cae antes de leer la prueba, con su causa; en el tope, rechazar una prueba fabricada no cuesta mucho mas que en m = 3; y el tope de la segunda es el del juez. Los pisos del §697 y las cifras del §708 quedan en las m que el enunciado admite, de 3 a 13 · §718: 51 -> 56, el emisor de la edad es un indice de cuenta y con todos es 0, y el importe, el plazo y la cota del cobro se acotan sobre el u64 crudo; el ancho del indice de cuenta de la segunda es el del juez · §721: 56 -> 58, el tope de m es 13 y no otro: m = 13 entra al juez y se juzga la prueba, y m = 14 cae por el tope, con su causa y en milisegundos; y el umbral del coste de rechazar en el tope baja de 25 a 10 · §723: 58 -> 65, la v2 de la banda y del cobro pendiente fuerza el bit 62 y nada mas cambia; la ventana de la v1 acaba en la cota, 2^32 - 2, y la segunda tiene la misma cota y la misma causa; la causa cuando ninguna version verifica es la de la que llego mas lejos; y la v1 ya publicada se reconoce por su huella y su enunciado, en una lista cerrada que la segunda lleva igual · §906: 65 -> 66, el registro de las familias de la marca v2 y su KAT
winter-air         sello      64   0   0    60  FORK de winterfell 0.13.1, APAGADO (RFC-0009 E3a-1, §533): los 49 tests de upstream, tal cual; entra por [patch.crates-io] con su nombre, y la foto de los circuitos (394) lo vigila · §575: Proof::from_bytes lee con un lector ACOTADO (src/proof/acotado.rs, declarado en su README); los 49 de upstream, tal cual · §711: 49 -> 56, una prueba ajena se rechaza con error y su causa, no con un panico: los siete testigos de src/proof/tests_sin_panico.rs (declarado en su README); los 49 de upstream, tal cual · §715: 56 -> 62, un usize de la prueba tiene una sola escritura, la minima, y la misma causa en toda plataforma: una longitud desde 2^32 es fin de bytes, y el numero de restricciones lo acota el contexto antes de pasar a usize: los seis testigos de src/proof/tests_canonica.rs (declarado en su README); los 49 de upstream, tal cual · §717: 62 -> 64, las exenciones de una transicion son un bloque, sin lista: dan lo que daba la lista -el grado, las exenciones y el divisor, en la extension y en el dominio de la traza, de 2^3 a 2^20 filas- y construir el divisor no depende de cuantas son (2^30 + 1); los 49 de upstream, tal cual
winter-prover      sello       7   0   0    60  FORK de winterfell 0.13.1, APAGADO (§533): los 6 tests de upstream; sin el bench de criterion; el lint de trace_table.rs:199 arreglado, por eso 0 warnings · §717: 6 -> 7, el probador evalua las exenciones en su dominio por recurrencia y no punto a punto, con los mismos valores: el dominio entero y un trozo, ce 2, 4 y 8, y la caida al calculo punto a punto donde un denominador es cero; los 6 de upstream, tal cual
winter-verifier    sello       0   0   0    60  FORK de winterfell 0.13.1, APAGADO (§533): upstream no trae tests; 0 es CORRECTO
settlement-prover  sello       0   0   0   300  sin tests
nova-experiment    sello       0   0   0   300  0 es CORRECTO: sus 3 tests exigen --features test-setup
halo2-experiment   largo      27   0   0  1200  caro: 386 s medidos en el §629, maquina del autor (⚠️ §811, 14-10-2026: decia 438 s, la cifra en frio del banco F.1)
plonk-experiment   largo      36   0   0  1800  caro: 730 s medidos en el §629, maquina del autor (⚠️ §811, 14-10-2026: decia 749 s, la cifra en frio del banco F.1) · §795: la DEUDA de 3 warnings, saldada: eran UN aviso -un use sin usar de dusk_bytes::Serializable en persistent_nullifier_registry.rs, cuyo to_bytes ya era el inherente de BlsScalar- y las dos lineas de resumen de cargo, (lib) y (lib test), que el contador cuenta porque empiezan por warning; quitado el use quedan 0 lineas, y 36 de 36 en 1374 s de pared y 1973 s de CPU con la maquina cargada (carga 5 a 14 en 4 nucleos): cabe en el timeout
zk-core            completo   77   0  10  3600  2075 s medidos en 6cb8883 (§260). La nota anterior (§225) decia 38,6 min = 1472 lib + 845 ceremonia: envejecio A LA BAJA. Los 10 warnings son de `ceremony`, no suyos · ⚠️ §811, 14-10-2026: el §629 midio 2.103 s, en la maquina del autor · ⚠️ §822, 15 y 21-10-2026: con el arbol disperso, el 15-10-2026, 486 s la fila en 4 CPU compartidas (carga 8,2), 37 de la biblioteca y 448 de la ceremonia, con 76 tests; el 21-10-2026, con el merkle.rs final y 77 tests, 271 s sobre 4d8017d (carga 3,9), 15 de la biblioteca y 256 de la ceremonia, y 555 s sobre 5fd5212, el §817 (medido antes del rebase, con el mismo codigo; carga 7,8), 48 y 506; con el denso, el 15-10-2026, la biblioteca sola tardo 3.546 s en la misma maquina (al pie) · §822: 74 -> 77, el arbol de Merkle de los tests no materializa el relleno y da la raiz del denso en cuatro casos de un KAT sacado del denso, con los caminos de 0, 3, 4, 100 y 2^20 - 1 con los bits de su indice y subiendo a ella; un camino fuera del arbol entra en panico, y un arbol con mas hojas de las que caben, tambien (la 75)
FIN_TABLA
)

# ── lo que cubre el nivel --completo, y nadie mas (§284) ────────
# La regla vieja «tocar un Cargo.* vence la foto» era SINTACTICA donde el
# riesgo es SEMANTICO: sobredisparaba sobre todo el proyecto salvo esto.
# `--completo` añade UNA fila sobre --sello: zk-core (su fila, arriba).
# Cierre real, leido de crates/zk-core/Cargo.toml: settlement-prover
# (path), ceremony (dev-dependency a proposito, grafo aciclico) y los
# ark-* que pincha el [workspace.dependencies] del Cargo.toml RAIZ.
#   - VENCE la foto: un cambio bajo estas rutas, o lineas ark-* en el
#     diff del Cargo.toml raiz.
#   - NO la vence (declarado ANTES de mirar nada): el Cargo.lock solo
#     —un lock movido sin toml cubierto imprime que VALE, con la razon—
#     ni la raiz tocada sin ark-*.
COMPLETO_CUBRE="crates/zk-core crates/settlement-prover crates/ceremony"

# ── utilidades ──────────────────────────────────────────────────
rojo=0
fallos=()
msg() { echo "$*" >&2; }
falla() { rojo=1; fallos+=("$1"); msg "  XX  $1"; }

nivel_num() {
  case "$1" in
    sello) echo 1 ;; largo) echo 2 ;; completo) echo 3 ;; *) echo 9 ;;
  esac
}

miembros_del_workspace() {
  # ⚠️ §833: los lee tools/miembros.py, el unico lector de `members`; lo que hacia el sed de aqui, en su cabecera
  python3 tools/miembros.py --con-crates | sed 's|^crates/||'
}

estado_completo() {
  if [ -f "$SELLO_FILE" ]; then
    local c f n toc raiz lock ark
    read -r c f < "$SELLO_FILE"
    n=$(git rev-list --count "$c..HEAD" 2>/dev/null || echo "?")
    if [ "$n" = "0" ]; then
      msg "  ultimo --completo: $c ($f) · AL DIA"
    else
      msg "  ultimo --completo: $c ($f) · **$n sello(s) por detras de HEAD**"
      if [ "$n" = "?" ]; then
        msg "  la foto apunta a $c y ese sello ya no se alcanza: se remide con --completo"
      else
        toc=$(git diff --name-only "$c..HEAD" -- $COMPLETO_CUBRE 2>/dev/null)
        raiz=$(git diff --name-only "$c..HEAD" -- Cargo.toml 2>/dev/null)
        lock=$(git diff --name-only "$c..HEAD" -- Cargo.lock 2>/dev/null)
        ark=0
        [ -n "$raiz" ] && ark=$(git diff "$c..HEAD" -- Cargo.toml 2>/dev/null | grep -cE '^[+-].*ark-')
        if [ -n "$toc" ]; then
          msg "  ⚠️ VENCIDA: lo cubierto cambio desde la foto — toca --completo:"
          sed 's/^/      /' <<< "$toc" >&2
        elif [ "${ark:-0}" -gt 0 ]; then
          msg "  ⚠️ VENCIDA: el Cargo.toml raiz movio lineas ark-* ($ark) — toca --completo"
        elif [ -n "$raiz" ]; then
          msg "  la raiz cambio sin mover ark-*: por detras pero limpio en lo cubierto, la foto VALE"
        elif [ -n "$lock" ]; then
          msg "  el lock se movio sin un Cargo.toml cubierto: por detras pero limpio en lo cubierto, la foto VALE"
        else
          msg "  por detras pero limpio en lo cubierto: la foto VALE"
        fi
      fi
    fi
  else
    msg "  ultimo --completo: **NUNCA**. Ningun --completo ha pasado VERDE aqui; el del §629 fueron 58 min de tests."
  fi
}

# Los bancos (§582): cuando pasaron VERDES por ultima vez, y si cambio lo que ejercen.
estado_bancos() {
  if [ -f "$BANCOS_FILE" ]; then
    local c f k n toc
    read -r c f k < "$BANCOS_FILE"
    n=$(git rev-list --count "$c..HEAD" 2>/dev/null || echo "?")
    if [ "$n" = "0" ]; then
      msg "  ultimo --bancos: $c ($f, $k bancos) · AL DIA"
    elif [ "$n" = "?" ]; then
      msg "  ultimo --bancos: $c ($f) · ese sello ya no se alcanza: se remide con --bancos"
    else
      toc=$(git diff --name-only "$c..HEAD" -- crates Cargo.toml Cargo.lock 'tools/banco_*' \
            2>/dev/null | wc -l | tr -d ' ')
      if [ "$toc" != "0" ]; then
        msg "  ultimo --bancos: $c ($f) · **$n sello(s) por detras**, y $toc fichero(s) de lo que ejercen cambiados — toca --bancos"
      else
        msg "  ultimo --bancos: $c ($f) · $n sello(s) por detras, limpio en lo que ejercen: la foto VALE"
      fi
    fi
  else
    msg "  ultimo --bancos: **NUNCA** desde que se anotan (§582). Ningun --bancos ha pasado VERDE aqui; el del §709 tardo 1.891 s."
  fi
}

# Los conservados (§798): cuando pasaron VERDES por ultima vez, y si cambio lo que ejercen.
# ⚠️ Lo que ejercen NO es `crates/`: cada kit es el de su commit. Es lo que los JUZGA -el arnes
# del arbol y la tabla de tools/conservados.sh-, lo juzgado -spec/vectors/<version>/- y el rustc
# con que se construyen los kits viejos, que es lo que mas se mueve sin que nadie toque nada.
estado_conservados() {
  if [ -f "$CONSERVADOS_FILE" ]; then
    local c f k r n toc rv
    read -r c f k r < "$CONSERVADOS_FILE"
    rv=$(rustc -V 2>/dev/null | awk '{print $2}')
    n=$(git rev-list --count "$c..HEAD" 2>/dev/null || echo "?")
    if [ "$n" = "?" ]; then
      msg "  ultimo --conservados: $c ($f) · ese sello ya no se alcanza: se remide con --conservados"
    else
      toc=$(git diff --name-only "$c..HEAD" -- spec/vectors tools/conformidad.sh tools/conservados.sh 2>/dev/null \
            | grep -cE '^(spec/vectors/[0-9]+\.[0-9]+/|tools/)')
      if [ "$toc" != "0" ]; then
        msg "  ultimo --conservados: $c ($f) · **$n sello(s) por detras**, y $toc fichero(s) de lo que ejercen cambiados — toca --conservados"
      elif [ "$n" = "0" ]; then
        msg "  ultimo --conservados: $c ($f, $k catalogos) · AL DIA"
      else
        msg "  ultimo --conservados: $c ($f) · $n sello(s) por detras, limpio en lo que ejercen: la foto VALE"
      fi
    fi
    [ "${rv:-?}" = "${r:-?}" ] \
      || msg "  ⚠️ el rustc de la maquina ya no es el de esa pasada (${r:-?} -> ${rv:-NINGUNO}): los kits viejos se construyen con el — toca --conservados"
  else
    msg "  ultimo --conservados: **NUNCA** en este clon. El §798 lo corrio una vez, VERDE, en otro;"
    msg "    antes, spec/vectors/<version>/ lo juzgaron la mano (0.3/) y el canon del §681."
  fi
}

case "$NIVEL" in
  --sello|--largo|--completo|--lista|--bancos|--conservados) : ;;
  *) msg "nivel desconocido: $NIVEL. Usa --sello, --largo, --completo, --bancos, --conservados o --lista."; exit 2 ;;
esac

# ── 0 · COHERENCIA: la tabla y el workspace dicen lo mismo ──────
msg "== CANON · coherencia con el workspace =="
MIEMBROS=$(miembros_del_workspace) || falla "tools/miembros.py en ROJO: \`members\` ilegible, o crates/ y \`members\` no dicen lo mismo"
EN_TABLA=$(awk 'NF{print $1}' <<< "$TABLA")
n_m=$(wc -l <<< "$MIEMBROS"); n_t=$(wc -l <<< "$EN_TABLA")
msg "  miembros en Cargo.toml: $n_m · filas en la tabla: $n_t"
# ⚠️ Here-string, NO tuberia: ver el punto 6 de la cabecera.
for c in $MIEMBROS; do
  grep -qx "$c" <<< "$EN_TABLA" || falla "el crate '$c' esta en el workspace y NO en el canon"
done
for c in $EN_TABLA; do
  grep -qx "$c" <<< "$MIEMBROS" || falla "la fila '$c' no corresponde a ningun crate del workspace"
done
[ $rojo -eq 0 ] && msg "  OK  todos los crates tienen fila y todas las filas tienen crate; crates/ y members, lo mismo"
estado_completo
estado_bancos
estado_conservados

if [ "$NIVEL" = "--bancos" ]; then
  msg ""
  bash tools/bancos.sh; rb=$?
  [ $rojo -eq 0 ] || rb=1
  exit $rb
fi

# ⚠️ §798: lo que va detras del nivel (`--en-seco`, `--frio`) es de tools/conservados.sh, que
# rechaza con exit 2 lo que no conoce. Ningun otro nivel lee argumentos detras del primero.
# ⚠️ El nivel corre en SEGUNDO PLANO y se espera con `wait`, por lo mismo que su construccion
# (tools/conservados.sh, junto a `parar`). En primer plano y sin trap, un TERM al canon lo mata
# solo a el -sale con 143- y el nivel sigue, huerfano y con su worktree listado; con trap, bash no
# lo correria hasta que el nivel acabase. El `wait` vuelve en cuanto llega la senal, y el trap se
# la pasa al nivel como TERM -el INT tambien: un hijo en segundo plano de un bash sin control de
# trabajos nace con el INT IGNORADO, y lo que se ignora al nacer no se puede atrapar-, espera a
# que limpie y sale con 143 (TERM) o 130 (INT). Medido en el §798 con un cargo falso dormido, las
# dos cosas, y el INT ignorado en el `SigIgn` de /proc/<pid>/status del nivel. `--bancos` sigue
# en primer plano, con el mismo hueco (no medido).
# ⚠️ Entre el `&` y el `HIJO_C=$!` hay una ventana: una senal ahi encontraria HIJO_C vacio y el
# nivel quedaria huerfano. En ella la senal solo se ANOTA (PEND_C) y se atiende en cuanto HIJO_C
# tiene el PID, como hace tools/conservados.sh con su construccion (`anotar`, `atender`). No con
# `${HIJO_C:-$!}`: con `set -u`, `$!` antes del primer hijo es una variable sin valor, y una senal
# que llegase antes del `&` haria salir al canon con 127, no con 143 (medido en el §798, bash 5.2).
if [ "$NIVEL" = "--conservados" ]; then
  msg ""
  HIJO_C=""; PEND_C=""
  parar_conservados() {
    [ -z "$HIJO_C" ] || { kill -TERM "$HIJO_C" 2>/dev/null; wait "$HIJO_C" 2>/dev/null; }
    exit "$1"
  }
  trap 'PEND_C=143' TERM
  trap 'PEND_C=130' INT
  bash tools/conservados.sh "${@:2}" &
  HIJO_C=$!
  trap 'parar_conservados 143' TERM   # 128 + 15
  trap 'parar_conservados 130' INT    # 128 + 2
  [ -z "$PEND_C" ] || parar_conservados "$PEND_C"
  wait "$HIJO_C"; rcv=$?
  trap - TERM INT
  [ $rojo -eq 0 ] || rcv=1
  exit $rcv
fi

if [ "$NIVEL" = "--lista" ]; then
  msg ""
  msg "== LA TABLA =="
  sed 's/^/  /' <<< "$TABLA" >&2
  exit $rojo
fi

# ── 1 · los tests, crate a crate ────────────────────────────────
PEDIDO=$(nivel_num "${NIVEL#--}")
OUT="${OUT:-/tmp/canon}"
rm -rf "$OUT"; mkdir -p "$OUT"
# ── §855 · antes de los tests: donde deja cargo los binarios, y si check_curvas podra leer el arbol ──
# El target lo da tools/target_dir.sh, de `cargo metadata`: sigue CARGO_TARGET_DIR,
# CARGO_BUILD_TARGET_DIR y build.target-dir, como cargo. Lo usan el 3 bis y las familias. Y en un clon
# recien hecho a la cache de cargo le faltan fuentes que lee check_curvas (§843), o hasta el indice
# si esta vacia: se avisa aqui, con el remedio, sin bajar nada. El ROJO, si lo hay, lo da el 2 ter.
DIR_TARGET=$(bash tools/target_dir.sh 2> "$OUT/target_dir.txt") \
  || falla "no se donde deja cargo los binarios: tools/target_dir.sh en rojo ($(tail -n 1 "$OUT/target_dir.txt"))"
python3 tools/check_curvas.py --cache > "$OUT/check_curvas_cache.txt" 2>&1 \
  || { msg ""; sed 's/^/  ⚠️  /' "$OUT/check_curvas_cache.txt" >&2; }
msg ""
msg "== CANON $NIVEL · tests =="
msg "   crate                exit  pasan(pin) ignor(pin) warn(pin)   seg"
T_TOTAL=0
while read -r c niv pasan ign warn tmo resto; do
  [ -n "$c" ] || continue
  [ "$(nivel_num "$niv")" -le "$PEDIDO" ] || continue
  T0=$(date +%s)
  timeout "${tmo}s" cargo test -p "$c" --release --locked > "$OUT/$c.txt" 2>&1
  RC=$?
  T1=$(date +%s); T_TOTAL=$((T_TOTAL + T1 - T0)); echo "$c $((T1 - T0))" >> "$OUT/tiempos.txt"  # ⚠️ §864: la razon del «1 bis» (la 94)
  P=$(grep -oE "[0-9]+ passed" "$OUT/$c.txt" | grep -oE "[0-9]+" | paste -sd+ | bc 2>/dev/null); P=${P:-0}
  I=$(grep -oE "[0-9]+ ignored" "$OUT/$c.txt" | grep -oE "[0-9]+" | paste -sd+ | bc 2>/dev/null); I=${I:-0}
  W=$(grep -c "^warning" "$OUT/$c.txt")
  printf "   %-20s %4s %6s(%s) %6s(%s) %5s(%s) %5s\n" "$c" "$RC" "$P" "$pasan" "$I" "$ign" "$W" "$warn" "$((T1-T0))" >&2
  [ "$RC" = "0" ]     || falla "$c: exit $RC  ->  tools/canon.sh, fila '$c'"
  [ "$P" = "$pasan" ] || falla "$c: pasan $P, el canon dice $pasan  ->  MEDIR y editar tools/canon.sh, fila '$c'"
  [ "$I" = "$ign" ]   || falla "$c: ignorados $I, el canon dice $ign  ->  tools/canon.sh, fila '$c'"
  [ "$W" = "$warn" ]  || falla "$c: warnings $W, el canon dice $warn  ->  tools/canon.sh, fila '$c'"
done <<< "$TABLA"
msg "   ── total: ${T_TOTAL} s"

# ── 2 · tests que no protegen ───────────────────────────────────
msg ""
msg "== CANON · tests que no protegen =="
python3 tools/check_tests.py > "$OUT/check_tests.txt" 2>&1
if [ $? -eq 0 ]; then
  msg "  OK  $(tail -2 "$OUT/check_tests.txt" | tr '\n' ' ')"
else
  sed 's/^/      /' "$OUT/check_tests.txt" >&2
  falla "check_tests.py encontro tests que no protegen"
fi

# ── 2 bis · ficheros .rs que no declara nadie ──
# Un `.rs` sin declarar NO se compila, asi que nada lo verifica — y encima
# invita a especular sobre el. `marlin_proof_system.rs` estuvo asi, y se
# sello DOS VECES una descripcion falsa de lo que era (§227).
msg ""
msg "== CANON · ficheros .rs que no declara nadie =="
python3 tools/check_modulos.py > "$OUT/check_modulos.txt" 2>&1
if [ $? -eq 0 ]; then
  msg "  OK  $(tail -1 "$OUT/check_modulos.txt")"
else
  sed 's/^/      /' "$OUT/check_modulos.txt" >&2
  falla "hay ficheros .rs que nadie declara"
fi

# ── 2 ter · las herramientas del bucle ──────────────────────────
# Hasta 269 el canon invocaba DOS de las siete de `tools/`. Las otras cinco
# las corria cada BLOQUE a mano, y una herramienta que nadie recuerda meter
# en un bloque es una herramienta que nadie corre: DOS llevaban rojas sin
# que nadie lo viera. Es exactamente lo que le paso a `check_tests.py`, y
# por lo que esta aqui arriba.
#
# Entran a PIN CERO. No hay pin de fallos: los rojos que tenian eran defecto
# de las propias herramientas —exclusiones que les faltaban y un patron que
# tomaba una plantilla por cita—, no deuda de documentacion, y 269 los
# arreglo ANTES de cablearlas.
#
# Y estar en el bucle no basta: hay que mirar algo (§695). `check_columns`
# corria aqui sin argumentos, leia `.` -desde la raiz, CERO `.rs`- y decia
# «0 circuitos», verde. Su universo y su prueba de vida viven ahora en la
# herramienta (`UNIVERSO`): los circuitos de `stark-experiment` y las AIR del
# kit, con su probador, y ROJO si cualquiera de los dos baja de su minimo de
# ficheros o de columnas. La llamada sigue sin argumentos, como las demas:
# asi la que se hace a mano, desde donde sea, mira lo mismo que esta.
msg ""
msg "== CANON · las herramientas de tools/ =="
for H in check_cifras check_figures check_columns check_constraint_layout verificar_citas check_dominios check_publicadas check_nucleo check_techo check_curvas check_superficie; do
  python3 "tools/$H.py" > "$OUT/$H.txt" 2>&1
  if [ $? -eq 0 ]; then
    msg "  OK  $H"
  else
    sed 's/^/      /' "$OUT/$H.txt" >&2
    falla "$H.py en rojo"
  fi
done

# ── 2 quater · los vectores no se reescriben (§692) ─────────────
# README.md promete vectores por version que jamas se reescriben (regla 2 de spec/rfc/PROCESO.md),
# y hasta el §692 ninguna compuerta lo miraba: el canon comprobaba que el arbol PASA sus vectores,
# no que fueran los publicados. tools/check_vectores.py fija el sha256 de cada vector y de cada
# linea de manifiesto en spec/vectors/HUELLAS.sha256: se puede AÑADIR, y MOVER con los mismos bytes
# a un directorio de version (spec/vectors/<version>/, como el §682); modificar o borrar es ROJO,
# con el fichero o la linea. Lo nuevo entra registrado en el mismo sello, con `--registrar`, que
# nunca sobrescribe una huella. Pin cero, como las de 2 ter; fuera de su bucle para que cada ROJO
# entre con su nombre. Menos de un segundo.
#
# Y contra la base publicada. La regla 6 compara las huellas del arbol con las de HEAD: un commit
# que reescriba a la vez un vector y su huella sin pasar por el canon ya no lo ve el canon
# siguiente, y no hay CI que lo vea. Mientras no se empuje, el canon lo ve: `--desde` la base,
# el commit de origin/main del que desciende HEAD. Sin git, sin origin/main o con HEAD en ella, no
# hay nada mas que mirar. De esa segunda pasada solo cuentan sus ROJO R6: lo demas es lo de arriba.
msg ""
msg "== CANON · los vectores no se reescriben =="
if python3 tools/check_vectores.py > "$OUT/check_vectores.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/check_vectores.txt")"
else
  while IFS= read -r L; do falla "vectores $L"; done < <(grep '^ROJO ' "$OUT/check_vectores.txt" | sed 's/^ROJO //')
  grep -q '^ROJO ' "$OUT/check_vectores.txt" || falla "check_vectores.py en rojo sin nombrar el fichero ($(tail -n 1 "$OUT/check_vectores.txt"))"
  msg "      que hacer: $OUT/check_vectores.txt; --registrar solo para lo NUEVO"
fi
BASE_V=$(git merge-base HEAD origin/main 2>/dev/null || true)
if [ -n "$BASE_V" ] && [ "$BASE_V" != "$(git rev-parse HEAD 2>/dev/null)" ]; then
  python3 tools/check_vectores.py --desde "$BASE_V" > "$OUT/check_vectores_base.txt" 2>&1
  if grep -q '^ROJO R6' "$OUT/check_vectores_base.txt"; then
    while IFS= read -r L; do falla "vectores, desde ${BASE_V:0:7} (origin/main): $L"; done < <(grep '^ROJO R6' "$OUT/check_vectores_base.txt" | sed 's/^ROJO //')
  elif grep -q 'primer registro' "$OUT/check_vectores_base.txt"; then
    msg "  OK  ${BASE_V:0:7}, la base en origin/main, aun no tiene huellas"
  else
    msg "  OK  y las huellas de ${BASE_V:0:7}, la base en origin/main, siguen en las del arbol"
  fi
fi

# ── 2 quinquies · el catalogo de rechazos (§777) ─────────────────
# spec/PAQUETE.md seccion 5 y la cabecera del kit (crates/zk-ssl-verify/src/main.rs) prometen que,
# al sellar, el censo de los rechazos se re-deriva POR LLAMADA y que cada texto que el kit puede
# emitir esta en el catalogo. Desde el §397, que lo hizo a mano (50 llamadas, 49 textos), nadie lo
# re-derivo, y el §776 encontro lo que eso deja: dos bloques enteros sin catalogar (el pago en curso
# y la completitud), los conjuntos de versiones sin la v6 y textos que el codigo ya decia de otra
# manera; contra el arbol del §775, 78 ROJO. tools/check_catalogo.py hace ese censo: cada llamada
# de rechazo del kit tiene entrada en la seccion 5 (ida), cada entrada tiene un productor en el kit o
# en la biblioteca de su envoltorio (vuelta), y los huecos que valen un conjunto de VersionCabeza se
# leen de crates/zk-ssl-verify/src/lib.rs, no de la prosa. Lo que no enumera lo IMPRIME como
# exclusion. Pin cero, fuera del bucle de 2 ter para que cada ROJO entre con su texto, como el de
# los vectores. Menos de un segundo.
msg ""
msg "== CANON · el catalogo de rechazos =="
if python3 tools/check_catalogo.py > "$OUT/check_catalogo.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/check_catalogo.txt")"
else
  while IFS= read -r L; do falla "catalogo $L"; done < <(grep '^ROJO ' "$OUT/check_catalogo.txt" | sed 's/^ROJO //')
  grep -q '^ROJO ' "$OUT/check_catalogo.txt" || falla "check_catalogo.py en rojo sin nombrar el texto ($(tail -n 1 "$OUT/check_catalogo.txt"))"
  msg "      que hacer: $OUT/check_catalogo.txt; el codigo manda: se corrige la seccion 5, no el kit"
fi

# ── 3 · conformidad ─────────────────────────────────────────────
msg ""
msg "== CANON · conformidad =="
# §716: el compromiso de las dos emisiones lleva el contador de emisiones, y el de las dos Mint
# del escenario cambia. La 0.4 ya no se reproduce: se conserva y se rechaza por version, como las
# anteriores; el escenario se fija en la 0.5.
cargo run --release --locked -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.5.json > "$OUT/c05.txt" 2>&1
if grep -q "todo IDENTICO" "$OUT/c05.txt"; then msg "  OK  0.5 -> todo IDENTICO (en lo que el circuito fija, D-AI)"; else falla "0.5 NO da 'todo IDENTICO'"; fi
cargo run --release --locked -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.4.json > "$OUT/c04.txt" 2>&1
if grep -q "OTRA version" "$OUT/c04.txt"; then msg "  OK  0.4 RECHAZADO (otra version)"; else falla "0.4 deberia rechazarse por version"; fi
cargo run --release --locked -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.3.json > "$OUT/c03.txt" 2>&1
if grep -q "OTRA version" "$OUT/c03.txt"; then msg "  OK  0.3 RECHAZADO (otra version)"; else falla "0.3 deberia rechazarse por version"; fi
cargo run --release --locked -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.2.json > "$OUT/c02.txt" 2>&1
if grep -q "OTRA version" "$OUT/c02.txt"; then msg "  OK  0.2 RECHAZADO (otra version)"; else falla "0.2 deberia rechazarse por version"; fi
cargo run --release --locked -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.1.json > "$OUT/c01.txt" 2>&1
if grep -q "OTRA version" "$OUT/c01.txt"; then msg "  OK  0.1 RECHAZADO (otra version)"; else falla "0.1 deberia rechazarse por version"; fi

# ── §786 · los bancos compilan con --locked ──
# Una cifra que publica un banco sale de un binario; sin --locked, cargo puede resolver otro
# Cargo.lock que el del sello y el banco mediria otro arbol. Hasta el §786 ninguna de las 53
# llamadas lo llevaba, ni las dos del banco del reinicio (§783). Esta puerta exige que ninguna lo olvide.
msg ""
msg "== CANON · los bancos compilan con --locked =="
# Por LLAMADA, no por linea: una linea con dos cargo, uno con --locked y otro sin el, no pasa.
CARGO_BANCOS=$(python3 - <<'PYEOF'
import glob, re
n, sin = 0, []
for f in sorted(glob.glob('tools/banco_*.sh')):
    for k, l in enumerate(open(f, encoding='utf-8'), 1):
        if l.lstrip().startswith('#'):
            continue
        for m in re.finditer(r'cargo (?:build|run|test|check)\b[^|;&]*', l):
            n += 1
            if '--locked' not in m.group(0):
                sin.append('%s:%d' % (f, k))
print(n, ' '.join(sin))
PYEOF
)
N_CARGO=${CARGO_BANCOS%% *}; SIN_LOCK=${CARGO_BANCOS#* }
if [ -z "$SIN_LOCK" ] && [ "${N_CARGO:-0}" -gt 0 ]; then
  msg "  OK  $N_CARGO llamadas a cargo en tools/banco_*.sh, todas con --locked"
else
  falla "llamadas a cargo SIN --locked en los bancos (${N_CARGO:-0} en total): $(echo "$SIN_LOCK" | cut -c1-200)"
fi

# ── §784 · ARQ-02 · el nodo SIN `dev` —el de produccion— se compila en cada sello ──
# Hasta el §784 ese build no lo compilaba nadie (`default = ["dev"]`), y su unica rama sin `dev`
# decia «pendiente de flags». Es `cargo check`: compila y tipa, sin generar el binario, para no
# pisar el `target/release/zk-ssl-node` con `dev` que usan los bancos. El aviso que queda es el de
# `podar` (§580), a la vista a proposito: el pin es 1, y uno mas es ROJO. ⚠️ §864: y uno menos tambien, y cuenta todo aviso, no los de una lista de clases: es el contador del §834 (la 94).
msg ""
msg "== CANON · el nodo sin dev (ARQ-02) =="
if cargo check --release --locked -p zk-ssl-node --no-default-features > "$OUT/sin_dev.txt" 2>&1; then
  AV=$(grep -E '^warning' "$OUT/sin_dev.txt" | grep -cvE 'generated [0-9]+ warnings?')  # ⚠️ §864: era una cota (-le 1) sobre once clases de aviso
  if [ "${AV:-0}" = 1 ]; then
    msg "  OK  zk-ssl-node --no-default-features compila: ${AV:-0} aviso(s), el de podar (pin 1, exacto)"
  else
    falla "el nodo sin dev da ${AV} avisos (pin 1, el de podar): $(grep -E '^warning: ' "$OUT/sin_dev.txt" | head -3 | tr '\n' ' ')"
  fi
else
  falla "el nodo sin dev NO compila: $(grep -E '^error' "$OUT/sin_dev.txt" | head -3 | tr '\n' ' ')"
fi

# ── §834 · los binarios y los ejemplos compilan en release y con --locked (la tarea 31) ──
# Hasta el §834 el canon no compilaba los diez `etapa_*` de `crates/zk-ssl/examples` (la 81 del
# BACKLOG ya lo decia): piden `sandbox` en su `required-features`, y `cargo test -p zk-ssl` los
# salta sin decirlo. Un argumento de menos en uno, o una feature que no existe en su
# `required-features`, pasaban el canon en VERDE. Tampoco compilaba el canon en release el binario
# `gen_openrpc` de `zk-ssl-wire`: su `cargo test` lo compila como test. Esta puerta lee de
# `cargo metadata` cada `bin` y cada `example` de los miembros cuyo nivel cabe en el pedido, y los
# compila con `cargo build --release --locked`: una llamada por paquete, con las features que piden
# sus objetivos. Por paquete y no con `--workspace`, porque unificar features entre paquetes rehace
# un binario con lo que pide otro, y el del kit es el que juzga el 3 bis. Lo que el canon ya
# compilaba sale fresco: el mando (3) y los ejemplos sin features. El kit lo compila aqui y el 3 bis
# lo encuentra hecho. El nodo con `dev` en release sale como el de los bancos que lo compilan solo
# o con el kit; los que lo compilan con el mando lo rehacen, porque cargo unifica sus features.
# Prueba de vida: cada objetivo listado sale con su ejecutable en un `compiler-artifact` de su
# llamada, y al menos MIN_CON_FEATURES piden features. La cuenta mezcla paquetes: hoy 12, los diez
# `etapa_*` y los dos de `zk-ssl-medio`. Sin holgura: uno de menos es ROJO; uno de mas deja holgura 1.
# Los avisos de rustc y de cargo, con pin exacto por paquete: cero, salvo el nodo, el de `podar`
# (§580), como en el §784. Fuera, declarado: el generador de `kat_xmss`, sin `Cargo.lock` versionado.
msg ""
msg "== CANON · los binarios y los ejemplos, en release y con --locked =="
MIN_CON_FEATURES=12  # ⚠️ §837: era 10, y el comentario decia que el minimo dejaba dos de holgura
T_OBJ0=$(date +%s)
# Una linea por paquete con objetivos: nombre, features (o -), cuantos piden features, y los objetivos.
PLAN_OBJ=$(cargo metadata --format-version 1 --no-deps --locked 2> "$OUT/objetivos_metadata.txt" | python3 -c '
import json, sys
for p in json.load(sys.stdin)["packages"]:
    obj = [t for t in p["targets"] if {"bin", "example"} & set(t["kind"])]
    if not obj:
        continue
    fe = sorted({f for t in obj for f in (t.get("required-features") or [])})
    print(p["name"], ",".join(fe) or "-", sum(1 for t in obj if t.get("required-features")),
          " ".join(("bin:" if "bin" in t["kind"] else "example:") + t["name"] for t in obj))
')
[ -n "$PLAN_OBJ" ] || falla "no leo los binarios ni los ejemplos de cargo metadata: $(head -c 200 "$OUT/objetivos_metadata.txt")"
N_OBJ=0; N_FE=0; N_PAQ=0; F_OBJ0=${#fallos[@]}
while read -r P FE NF OBJS; do
  [ -n "$P" ] || continue
  NIV=$(awk -v c="$P" '$1 == c {print $2}' <<< "$TABLA")
  if [ "$(nivel_num "${NIV:-?}")" -gt "$PEDIDO" ]; then
    msg "  --  $P: nivel ${NIV:-sin fila}, fuera de $NIVEL"
    continue
  fi
  ARGS=()
  case " $OBJS" in *" bin:"*) ARGS+=(--bins) ;; esac
  case " $OBJS" in *" example:"*) ARGS+=(--examples) ;; esac
  [ "$FE" = "-" ] || ARGS+=(--features "$FE")
  J="$OUT/objetivos_${P//-/_}.json"; E="$OUT/objetivos_${P//-/_}.txt"
  if ! cargo build --release --locked -p "$P" "${ARGS[@]}" --message-format=json-render-diagnostics > "$J" 2> "$E"; then
    falla "$P: sus binarios o ejemplos NO compilan en release con --locked (${ARGS[*]}): $(grep -E '^error' "$E" | head -3 | tr '\n' ' ')"
    continue
  fi
  # Los avisos de rustc y de cargo; la linea de resumen de cargo («generated N warning(s)») no cuenta.
  W=$(grep -E '^warning' "$E" | grep -cvE 'generated [0-9]+ warnings?')
  case "$P" in zk-ssl-node) PIN_W=1 ;; *) PIN_W=0 ;; esac
  [ "$W" = "$PIN_W" ] || falla "$P: sus binarios o ejemplos dan $W aviso(s), pin $PIN_W: $(grep -E '^warning' "$E" | head -3 | tr '\n' ' ')"
  FALTAN=$(python3 -c '
import json, sys
hechos = set()
for l in open(sys.argv[1], encoding="utf-8"):
    try:
        m = json.loads(l)
    except ValueError:
        continue
    if m.get("reason") == "compiler-artifact" and m.get("executable"):
        for k in m["target"]["kind"]:
            hechos.add(k + ":" + m["target"]["name"])
print(" ".join(o for o in sys.argv[2:] if o not in hechos))
' "$J" $OBJS)
  [ -z "$FALTAN" ] || falla "$P: cargo no dio el ejecutable de: $FALTAN"
  N_PAQ=$((N_PAQ + 1)); N_OBJ=$((N_OBJ + $(wc -w <<< "$OBJS"))); N_FE=$((N_FE + NF))
done <<< "$PLAN_OBJ"
[ "$N_FE" -ge "$MIN_CON_FEATURES" ] \
  || falla "solo $N_FE objetivos con features compilados, y la prueba de vida pide $MIN_CON_FEATURES, los de hoy, sin holgura (cuenta todos los paquetes: los diez etapa_* y los dos de zk-ssl-medio)"
[ "${#fallos[@]}" != "$F_OBJ0" ] \
  || msg "  OK  $N_OBJ objetivos en $N_PAQ paquetes, $N_FE con features ($(( $(date +%s) - T_OBJ0 )) s)"

# ── 3 bis · conformidad del PAQUETE (spec/PAQUETE.md, RFC-0004 E2, §398; el arnes: RFC-0005 E4, §408) ──
msg ""
msg "== CANON · conformidad del paquete =="
cargo build --release -q --locked -p zk-ssl-verify > "$OUT/paquete_build.txt" 2>&1 || falla "zk-ssl-verify no compila en release"
# UN solo productor del bucle del manifiesto: tools/conformidad.sh, el mismo que un tercero corre
# sobre cualquier binario y el que artefacto.sh corre sobre el binario con remap. Cada ROJO del
# arnes entra aqui con su nombre; la prueba de vida (manifiesto vacio, vector sin entrada) es suya.
if bash tools/conformidad.sh "$DIR_TARGET/release/zk-ssl-verify" > "$OUT/paquete.txt" 2>&1; then  # ⚠️ §844: decia target/release/, sin CARGO_TARGET_DIR; ⚠️ §855: decia ${CARGO_TARGET_DIR:-target}
  msg "  OK  paquete: $(tail -n 1 "$OUT/paquete.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "paquete $L"; done < <(grep '^ROJO' "$OUT/paquete.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/paquete.txt" || falla "paquete: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/paquete.txt"))"
fi

# ── 3 bis cable · los rechazos del CABLE (RFC-0005 E3, §409): el MISMO arnes, otro manifiesto ──
msg ""
msg "== CANON · los rechazos del cable =="
cargo build --release -q --locked -p zk-ssl-cli > "$OUT/cable_build.txt" 2>&1 || falla "zk-ssl-cli no compila en release"
# tools/cable_respuesta.sh es el adaptador al contrato del mando: un argumento, exit 0/1/2; una
# segunda implementacion del consumidor pone su ejecutable en su lugar. Cada ROJO entra por falla.
if bash tools/conformidad.sh tools/cable_respuesta.sh spec/vectors/cable/MANIFIESTO.txt > "$OUT/cable.txt" 2>&1; then
  msg "  OK  cable: $(tail -n 1 "$OUT/cable.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "cable $L"; done < <(grep '^ROJO' "$OUT/cable.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/cable.txt" || falla "cable: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/cable.txt"))"
fi

# ── 3 bis familias · los sobres del kit, UN bucle sobre FAMILIAS (§787) ──
# Hasta el §787 aqui iban once bloques identicos salvo el nombre, uno por familia, copiados a mano
# cada vez que entraba una. FAMILIAS tiene UN productor, tools/artefacto.sh (S423), y aqui se LEE,
# no se copia. Prueba de vida en los dos sentidos: cada familia tiene su MANIFIESTO con entradas, y
# cada spec/vectors/<x>/MANIFIESTO.txt es de una familia o del cable (el 3 bis cable): un
# manifiesto que ningun bucle corre es ROJO. El binario es el MISMO que el 3 bis ya construyo en
# release; el bucle es tools/conformidad.sh, con cada ROJO entrando por falla con su nombre. ⚠️ §863: o del nodo (spec/vectors/nodo/), que juzga un test del nodo: el «3 bis nodo».
FAMILIAS=$(sed -n 's/^FAMILIAS="\([^"]*\)".*/\1/p' tools/artefacto.sh)
[ -n "$FAMILIAS" ] || falla "no leo FAMILIAS de tools/artefacto.sh"
titulo_de(){
  case "$1" in
    consumo|conflicto) echo "los rechazos del sobre de $1" ;;
    rechazo) echo "las causas del sobre de rechazo" ;;
    pendiente) echo "las formas del sobre de cobro pendiente" ;;
    pago) echo "las formas del sobre de pago en curso" ;;
    ancla) echo "las formas del sobre del ancla" ;;
    ancla-cofirmada) echo "las formas del sobre del ancla cofirmada" ;;
    rotacion) echo "la rotacion de la clave, con su cadena de actas" ;;
    *) echo "las formas del sobre de $1" ;;
  esac
}
for F in $FAMILIAS; do
  [ "$F" = paquete ] && continue   # el 3 bis, que compila el binario
  M="spec/vectors/$F/MANIFIESTO.txt"
  msg ""
  msg "== CANON · $(titulo_de "$F") =="
  if [ ! -s "$M" ] || ! grep -qv '^#' "$M"; then
    falla "$F: la familia esta en FAMILIAS y $M no existe o no tiene entradas"
    continue
  fi
  O="$OUT/${F//-/_}.txt"
  if bash tools/conformidad.sh "$DIR_TARGET/release/zk-ssl-verify" "$M" > "$O" 2>&1; then  # ⚠️ §844: decia target/release/; ⚠️ §855: decia ${CARGO_TARGET_DIR:-target}
    msg "  OK  $F: $(tail -n 1 "$O" | sed 's/^conformidad: //')"
  else
    while IFS= read -r L; do falla "$F $L"; done < <(grep '^ROJO' "$O" | sed 's/^ROJO //')
    grep -q '^ROJO' "$O" || falla "$F: el arnes falla sin nombrar la entrada ($(tail -n 1 "$O"))"
  fi
done
for M in spec/vectors/*/MANIFIESTO.txt; do
  D=$(basename "$(dirname "$M")")
  case " $FAMILIAS cable nodo " in
    *" $D "*) ;;
    *) falla "$M: ningun bucle del canon lo corre (falta $D en FAMILIAS, tools/artefacto.sh)" ;;
  esac
done

# ── 3 duodecies · la SEGUNDA implementacion (BACKLOG 85; tools/segunda/) ──
# Otro codigo, en Python y sin dependencias, reproduce el nucleo congelado desde NUCLEO.md (34 KAT
# desde el §800; ⚠️ §800: aqui decia 26, y eran 31: el §643 y el §773 anadieron cinco sin subirla)
# y verifica las cabezas firmadas de TODOS los vectores con XMSS^MT escrito desde RFC 8391
# (tools/segunda/kat_xmss/xmss.py, dentro del directorio extraible). Esta
# aqui por lo mismo que las herramientas del bucle (2 ter): un KAT nuevo o una version nueva de
# cabeza que la segunda implementacion no reproduzca tiene que verse en el sello, no cuando alguien
# se acuerde de correrla. Pin cero: los tres jueces y el segundo verificador fallan con nombre. ~45 s,
# casi todos de XMSS en Python.
msg ""
msg "== CANON · la segunda implementacion (tools/segunda) =="
T_SEG0=$(date +%s)
if python3 tools/segunda/juez_nucleo.py > "$OUT/segunda_nucleo.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/segunda_nucleo.txt")"
else
  grep '^ROJO' "$OUT/segunda_nucleo.txt" | sed 's/^/      /' >&2
  falla "la segunda implementacion NO reproduce los KAT del nucleo ($(tail -n 1 "$OUT/segunda_nucleo.txt"))"
fi
if python3 tools/segunda/juez_cabezas.py > "$OUT/segunda_cabezas.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/segunda_cabezas.txt") ($(( $(date +%s) - T_SEG0 )) s)"
else
  grep '^NO' "$OUT/segunda_cabezas.txt" | grep -v '\[negativo\]' | sed 's/^/      /' >&2
  falla "la segunda implementacion NO verifica las cabezas firmadas ($(tail -n 1 "$OUT/segunda_cabezas.txt"))"
fi
# El corpus KAT de XMSS^MT (tools/segunda/kat_xmss, entrada 77): firmas del crate clavado, juzgadas
# por la implementacion de RFC 8391. Si `xmss` cambia de version y de bytes, aqui se ve. ~1 s.
# ⚠️ §786 · Y el corpus mismo, por su huella: el juez comprueba que las firmas verifican, no que
#    sean ESTAS; un corpus regenerado con otros bytes que tambien verificaran pasaba en silencio.
#    Como los vectores de spec/vectors/ (HUELLAS.sha256), un corpus publicado no se reescribe: uno
#    nuevo entra con otro nombre y su huella, en el sello que lo diga.
HUELLA_KAT_XMSS=259b8df2bb6633ffe5c40da91a1c4d2dc7697b375546b0fa3bd63e2a2a3c653a
HK=$(sha256sum tools/segunda/kat_xmss/vectores.json 2>/dev/null | cut -c1-64)
if [ "$HK" = "$HUELLA_KAT_XMSS" ]; then
  msg "  OK  kat_xmss: el corpus es el fijado (sha256 ${HK:0:16}…)"
else
  falla "el corpus KAT de xmss NO es el fijado: tools/segunda/kat_xmss/vectores.json da ${HK:-<ilegible>} y el canon fija $HUELLA_KAT_XMSS"
fi
if python3 tools/segunda/kat_xmss/juez_xmss.py > "$OUT/segunda_kat_xmss.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/segunda_kat_xmss.txt" | sed 's/ - generador:.*//')"
else
  grep '^ROJO' "$OUT/segunda_kat_xmss.txt" | sed 's/^/      /' >&2
  falla "el corpus KAT de xmss NO verifica con la implementacion de RFC 8391 ($(tail -n 1 "$OUT/segunda_kat_xmss.txt"))"
fi
# El MEDIO en la segunda implementacion (tools/segunda/mldsa44.py y medio.py, §810): ML-DSA-44 escrito
# desde FIPS 204, la nota checkpoint con su firma 0x06 y la inclusion de RFC 9162, contra lo que
# escribieron otras manos: las 33 notas del §632 (torchwood sobre el crypto/mldsa de Go, y el crate
# ml-dsa), las vkeys y las lineas de Go rehechas byte a byte por un firmante de prueba, las firmas de
# frontera de la norma y de UseHint, la unicidad de la escritura de una firma, las 98 sondas de
# inclusion de transparency-dev y el {:?} de Rust de los nombres; y desde el §859, cinco reglas que
# nada aislaba, con su caso y su control (92 comprobaciones). Pin cero, como los otros. ~2 s.
if python3 tools/segunda/juez_medio.py > "$OUT/segunda_medio.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/segunda_medio.txt")"
else
  grep '^ROJO' "$OUT/segunda_medio.txt" | sed 's/^/      /' >&2
  falla "la segunda implementacion NO juzga el medio como las otras manos ($(tail -n 1 "$OUT/segunda_medio.txt"))"
fi
# El SEGUNDO VERIFICADOR (tools/segunda/verificador.py): las cinco formas sin STARK del paquete -y
# desde el §649 la extension entre dos claves con su cadena de actas, la familia rotacion-, con
# el contrato del mando, juzgadas por el MISMO arnes y los MISMOS manifiestos que el binario de
# referencia. Si un vector nuevo entra en estas familias, los dos codigos tienen que decir lo mismo.
# ⚠️ §810: y desde el §810 la sexta, `ancla-cofirmada` (RFC-0013 E4a), que hasta aqui salia NO SOPORTADO
# en sus 27 entradas y ahora pasa con el mando entero: la cabeza, la nota, las cofirmas y la inclusion.
for F in paquete consumo conflicto ancla rotacion ancla-cofirmada; do
  if bash tools/conformidad.sh tools/segunda/verificador.py "spec/vectors/$F/MANIFIESTO.txt" > "$OUT/segunda_$F.txt" 2>&1; then
    msg "  OK  segunda $F: $(tail -n 1 "$OUT/segunda_$F.txt" | sed 's/^conformidad: //; s/ - binario.*//')"
  else
    while IFS= read -r L; do falla "segunda $F $L"; done < <(grep '^ROJO' "$OUT/segunda_$F.txt" | sed 's/^ROJO //')
    grep -q '^ROJO' "$OUT/segunda_$F.txt" || falla "segunda $F: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/segunda_$F.txt"))"
  fi
done
# El VERIFICADOR STARK (tools/segunda/stark.py y airs.py, cuarto hito de la 85, §626): las pruebas
# ocultas y con sal de las familias que el kit verifica hoy -rechazo, edad, pago, pendiente,
# prenda y completitud-, cada una con el enunciado que el mando le compone, contra lo que fija su
# MANIFIESTO, y tres falsadores por positivo. Las AIR estan transcritas del .rs: lo que este juez
# mide es la maquinaria (Fiat-Shamir, FRI, DEEP, Merkle con sal, Oculta), no las AIR. ~15 s.
if python3 tools/segunda/juez_stark.py > "$OUT/segunda_stark.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/segunda_stark.txt")"
else
  grep '^FALLA' "$OUT/segunda_stark.txt" | sed 's/^/      /' >&2
  falla "la segunda implementacion NO juzga las pruebas STARK como el manifiesto ($(tail -n 1 "$OUT/segunda_stark.txt"))"
fi
# ⚠️ §851 · El CABLE en la segunda implementacion (tools/segunda/cable.py), que no es nucleo, sus dos
# piezas. Hasta el §851 ninguna corria aqui: se corrian a mano (el §797 y el §809 lo dejaron dicho).
# I · la CADENA del registro: juez_cadena.py recompone cada `chain` de los vectores del cable
#     (spec/vectors/zkssl-0.*.json; el glob, para que un catalogo nuevo entre solo) desde sus
#     entradas, con sus falsadores. Prueba de vida: el glob da al menos un fichero, y el juez dice
#     haberlos leido todos. Sus lineas SIN EJERCITAR (las etiquetas y las filas de R6 que ninguna
#     entrada contrasta) no son ROJO: el juez las calcula y las dice; quedan en su salida.
# II · el segundo CONSUMIDOR: cable.py con el arnes y el manifiesto del cable, el mismo que el
#     «3 bis cable» corre con el adaptador del testigo: los dos consumidores, la misma lista.
# Pin cero, como los otros: cada ROJO entra por falla con su nombre. ~1 s y ~2 s.
CATALOGOS=(spec/vectors/zkssl-0.*.json)
if [ ! -e "${CATALOGOS[0]}" ]; then
  falla "segunda cadena: el glob spec/vectors/zkssl-0.*.json no da ningun vector del cable"
elif python3 tools/segunda/juez_cadena.py "${CATALOGOS[@]}" > "$OUT/segunda_cadena.txt" 2>&1; then
  if tail -n 1 "$OUT/segunda_cadena.txt" | grep -q " - ${#CATALOGOS[@]} fichero(s) - "; then
    msg "  OK  $(tail -n 1 "$OUT/segunda_cadena.txt" | sed 's/ - implementacion.*//')"
  else
    falla "segunda cadena: el juez sale con 0 y no dice haber leido los ${#CATALOGOS[@]} vectores ($(tail -n 1 "$OUT/segunda_cadena.txt"))"
  fi
else
  grep '^ROJO' "$OUT/segunda_cadena.txt" | sed 's/^/      /' >&2
  falla "la segunda implementacion NO recompone la cadena del registro ($(tail -n 1 "$OUT/segunda_cadena.txt"))"
fi
if bash tools/conformidad.sh tools/segunda/cable.py spec/vectors/cable/MANIFIESTO.txt > "$OUT/segunda_cable.txt" 2>&1; then
  msg "  OK  segunda cable: $(tail -n 1 "$OUT/segunda_cable.txt" | sed 's/^conformidad: //; s/ - binario.*//')"
else
  while IFS= read -r L; do falla "segunda cable $L"; done < <(grep '^ROJO' "$OUT/segunda_cable.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/segunda_cable.txt" || falla "segunda cable: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/segunda_cable.txt"))"
fi

# ── 3 bis nodo · lo que RESPONDE el nodo (spec/vectors/nodo/, ⚠️ §863, la 90) ──
# Los vectores del NODO: lo que un nodo conforme responde a una secuencia de peticiones JSON-RPC, sobre
# un libro recien constituido. Nacen con el del §802, que la R6 f del RFC-0021 pedia: abrir por el cable
# una identidad que ya tiene cuenta es -32000 con la causa IdentityUnavailable. Su manifiesto no tiene el
# contrato del mando -la segunda columna es un `code` del cable-, asi que tools/conformidad.sh no lo
# corre: lo juzga un test del nodo, que lee el manifiesto y cada vector, manda cada peticion por la
# funcion del transporte y compara. La fila zk-ssl-node del bloque 1 ya lo corrio; aqui se exige, por su
# nombre, que haya pasado: un manifiesto del nodo con su test borrado, renombrado o ignorado es ROJO, y
# no solo un pin que se mueve. Es un 3 bis por lo que hace -la conformidad de una familia de vectores,
# como el 3 bis, el 3 bis cable y el 3 bis familias-, y va aqui, detras del 3 duodecies, para no mover
# las lineas de los bloques de delante. El OK cuenta las entradas como el test: sin los comentarios ni
# las lineas en blanco.
msg ""
msg "== CANON · lo que responde el nodo =="
NODO_T="tests_vectores_del_nodo::el_nodo_responde_lo_que_dice_cada_vector_de_spec_vectors_nodo"
if grep -qx "test $NODO_T \.\.\. ok" "$OUT/zk-ssl-node.txt" 2>/dev/null; then
  msg "  OK  nodo: $(grep -cvE '^#|^[[:space:]]*$' spec/vectors/nodo/MANIFIESTO.txt) vector(es) de spec/vectors/nodo/, juzgados por $NODO_T"
else
  falla "nodo: el test $NODO_T no paso en la fila zk-ssl-node ($OUT/zk-ssl-node.txt): nadie juzga spec/vectors/nodo/"
fi

# ── 0 bis · cada miembro, en una clase de lo que el canon NO ve (⚠️ §864, la 94) ──
# Lo que el canon no ve, clase por clase, con su razon y cuando se reabre, lo declara «Ceguera
# declarada», al pie. Una lista en prosa de que crates ve cada pasada envejeceria sin que nada se
# pusiera rojo, que es la (d) de la misma 94; por eso la clasificacion es CODIGO: cada miembro del
# workspace, el MIEMBROS que el bloque 0 lee con tools/miembros.py, va en exactamente una de estas
# cuatro clases, y cada nombre de una clase es un miembro. Un crate nuevo sin clase, o uno en dos,
# es ROJO, como un crate sin fila en la tabla (punto 1 de la cabecera). Es un 0 bis por lo que hace,
# la coherencia con el workspace, y va aqui, detras del 3 bis nodo, por lo mismo que el: para no
# mover las lineas de los bloques de delante.
#   ASERCIONES_PASADA  lo que un tercero compila sin el probador -el kit, su clausura y el guardian
#                      del testigo-: la pasada del 1 bis. Va de las hojas al kit, para que en frio
#                      cada crate compile lo suyo.
#   CIEGA_PROBADOR     el probador y su clausura: los que llevan `winter-prover`, y
#                      `settlement-prover`, que es dependencia de `stark-experiment`: ciega,
#                      PERMANENTE.
#   CIEGA_FORKS        los forks del juez: el 1 bis los ejerce como dependencias, no sus tests.
#   CIEGA_OTROS        los experimentos de arkworks y los otros, fuera de la clausura del kit y
#                      del nodo: ciega, PERMANENTE.
ASERCIONES_PASADA="zk-ssl-hash zk-ssl-guardian zk-ssl-medio zk-ssl-air zk-ssl-verify"
CIEGA_PROBADOR="zk-ssl zk-ssl-node zk-ssl-cli zk-ssl-sdk zk-ssl-wire stark-experiment settlement-prover winter-prover"
CIEGA_FORKS="winter-air winter-verifier"
CIEGA_OTROS="zk-core ceremony settlement-layer iso-bridge halo2-experiment plonk-experiment nova-experiment"
msg ""
msg "== CANON · cada miembro, en una clase de lo que el canon no ve (la 94) =="
F_CL0=${#fallos[@]}
CLASES=" $ASERCIONES_PASADA $CIEGA_PROBADOR $CIEGA_FORKS $CIEGA_OTROS "
[ -n "$MIEMBROS" ] || falla "ceguera declarada: no hay miembros que clasificar (tools/miembros.py no dio ninguno)"
for c in $MIEMBROS; do
  n=0; for x in $CLASES; do [ "$x" = "$c" ] && n=$((n + 1)); done
  [ "$n" = 1 ] || falla "ceguera declarada: el miembro '$c' esta en $n clases, y va en una  ->  tools/canon.sh, bloque 0 bis"
done
for x in $CLASES; do
  grep -qx "$x" <<< "$MIEMBROS" || falla "ceguera declarada: '$x' tiene clase y no es miembro del workspace  ->  tools/canon.sh, bloque 0 bis"
done
[ "${#fallos[@]}" != "$F_CL0" ] \
  || msg "  OK  los $(wc -w <<< "$MIEMBROS") miembros, cada uno en una clase: $(wc -w <<< "$ASERCIONES_PASADA") en la pasada de las aserciones, $(wc -w <<< "$CIEGA_PROBADOR") con el probador, $(wc -w <<< "$CIEGA_FORKS") forks del juez y $(wc -w <<< "$CIEGA_OTROS") fuera de la clausura"

# ── 1 bis · lo que un tercero compila sin el probador, con las aserciones de depuracion (⚠️ §864, la 94) ──
# El bloque 1 corre cada crate en release, y en release no vive ningun `debug_assert!` ni nada de lo
# que va bajo `cfg(debug_assertions)`: el canon era ciego a esa clase entera. Quien integra el kit
# como dependencia lo compila con su perfil de desarrollo, con las aserciones encendidas, y
# CONTRIBUTING.md §4 manda correr en depuracion. Esta pasada corre otra vez los tests de
# ASERCIONES_PASADA -el kit, su clausura y el guardian del testigo, lo que un tercero compila sin el
# probador-, con el perfil `aserciones` del Cargo.toml raiz: hereda de release -opt 3 y los
# `overflow-checks` del §669- y enciende `debug-assertions`, en su propio `<target>/aserciones/`,
# sin pisar el `target/release` de los bancos. Las features son las de la fila: en los cinco, las
# por defecto son todas. Los pines son los de la fila de cada crate en la TABLA -pasan, ignorados y
# avisos, contados como en el bloque 1-, y el timeout, el doble del suyo.
# Prueba de vida: cada «Running» sale de `<target>/aserciones/deps/`, cada crate corre al menos uno,
# corren todos los de la lista, y la lista no esta vacia. Y la del PERFIL: el JSON de cargo de la
# misma orden con `--no-run`, que sale fresco, da cada artefacto con `"debug_assertions":true`, y
# cada «Running» es uno de sus ejecutables. Sin ella, un perfil con `debug-assertions = false` -o
# sin esa linea, que hereda el false de release- corria en el mismo directorio, con los mismos
# pines, y decia «encendidas»; con ella es ROJO, y tambien si las apaga un override por paquete o
# una variable CARGO_PROFILE_ASERCIONES_* (los tres, medidos en el §864).
# Y la razon: la pared de la pasada contra la suma de las filas de release de los mismos crates en
# ESTA corrida (`$OUT/tiempos.txt`, del bloque 1). Se imprime en cada corrida, para que se vea si
# deriva, y no es compuerta: bajo carga seria intermitente. El nivel lo fijo la razon medida en el
# §864, con el tope del doble fijado antes de medir (su asiento). Se reabre si una corrida posterior
# da mas en uno de los dos escenarios que lo decidieron -en caliente, o tras tocar `zk-ssl-hash`,
# que recompila la clausura en los dos perfiles-: entonces baja a --completo, con su foto y
# COMPLETO_CUBRE ampliado. La que compila uno solo de los dos perfiles -en frio, o con release al
# dia y la pasada por compilar, el 3,00 del §864- se informa y no decide.
# Es un 1 bis por lo que hace, los tests crate a crate, y va aqui por lo mismo que el 0 bis.
NIVEL_ASERCIONES=sello
PERFIL_AS=aserciones
msg ""
msg "== CANON · lo que un tercero compila sin el probador, con las aserciones de depuracion (la 94) =="
if [ "$(nivel_num "$NIVEL_ASERCIONES")" -gt "$PEDIDO" ]; then
  msg "  --  nivel $NIVEL_ASERCIONES, fuera de $NIVEL"
else
  F_AS0=${#fallos[@]}; T_AS=0; T_REL=0; N_AS=0
  [ -n "$ASERCIONES_PASADA" ] || falla "aserciones: ASERCIONES_PASADA esta vacia y la pasada no mira nada"
  msg "   crate                exit  pasan(pin) ignor(pin) warn(pin)   seg  release  aserc(json)"
  for c in $ASERCIONES_PASADA; do
    FILA=$(awk -v c="$c" '$1 == c' <<< "$TABLA")
    read -r _ _ pasan ign warn tmo _ <<< "$FILA"
    if [ -z "$tmo" ]; then falla "aserciones: '$c' no tiene fila en la tabla, y sus pines salen de ella"; continue; fi
    A="$OUT/aserciones_${c//-/_}.txt"; AJ="$OUT/aserciones_${c//-/_}.json"
    T0=$(date +%s)
    timeout "$((2 * tmo))s" cargo test -p "$c" --profile "$PERFIL_AS" --locked > "$A" 2>&1
    RC=$?
    : > "$AJ"
    [ "$RC" = "124" ] \
      || timeout "${tmo}s" cargo test -p "$c" --profile "$PERFIL_AS" --locked --no-run --message-format=json > "$AJ" 2> "$AJ.txt"
    T1=$(date +%s); T_AS=$((T_AS + T1 - T0))
    P=$(grep -oE "[0-9]+ passed" "$A" | grep -oE "[0-9]+" | paste -sd+ | bc 2>/dev/null); P=${P:-0}
    I=$(grep -oE "[0-9]+ ignored" "$A" | grep -oE "[0-9]+" | paste -sd+ | bc 2>/dev/null); I=${I:-0}
    W=$(grep -c "^warning" "$A")
    TR=$(awk -v c="$c" '$1 == c {print $2}' "$OUT/tiempos.txt" 2>/dev/null); T_REL=$((T_REL + ${TR:-0}))
    read -r NR NF <<< "$(awk '/^ +Running /{n++; if ($0 !~ /\((.*\/)?aserciones\/deps\/[^\/]+\)$/) f++} END{print n+0, f+0}' "$A")"
    # La del perfil: artefactos del JSON, cuantos van sin debug_assertions, y «Running» que no son
    # ejecutables del JSON: lo que corrio no es lo que el JSON describe. Las rutas se comparan
    # resueltas: cargo las da relativas a la raiz.
    read -r NJ NSA NX <<< "$(python3 -c '
import json, os, re, sys
ejec, nj, nsa = set(), 0, 0
for l in open(sys.argv[1], encoding="utf-8", errors="replace"):
    try:
        m = json.loads(l)
    except ValueError:
        continue
    if m.get("reason") != "compiler-artifact":
        continue
    nj += 1
    if (m.get("profile") or {}).get("debug_assertions") is not True:
        nsa += 1
    if m.get("executable"):
        ejec.add(os.path.realpath(m["executable"]))
nx = 0
for l in open(sys.argv[2], encoding="utf-8", errors="replace"):
    r = re.match(r" +Running .*\((.+)\)$", l.rstrip("\n"))
    if r and os.path.realpath(r.group(1)) not in ejec:
        nx += 1
print(nj, nsa, nx)
' "$AJ" "$A" 2>/dev/null)"
    NJ=${NJ:-0}; NSA=${NSA:-?}; NX=${NX:-?}
    printf "   %-20s %4s %6s(%s) %6s(%s) %5s(%s) %5s %8s  %s/%s\n" "$c" "$RC" "$P" "$pasan" "$I" "$ign" "$W" "$warn" "$((T1-T0))" "${TR:-?}" "$((NJ - ${NSA//\?/0}))" "$NJ" >&2
    [ "$RC" != "124" ]  || falla "aserciones $c: exit 124, el timeout de $((2 * tmo)) s, el doble del de su fila  ->  $A"
    [ "$RC" = "0" ] || [ "$RC" = "124" ] || falla "aserciones $c: exit $RC  ->  $A"
    [ "$P" = "$pasan" ] || falla "aserciones $c: pasan $P, su fila dice $pasan: algo cae o no corre con las aserciones  ->  $A"
    [ "$I" = "$ign" ]   || falla "aserciones $c: ignorados $I, su fila dice $ign: un test gateado en depuracion gana su pin propio (la 94)"
    [ "$W" = "$warn" ]  || falla "aserciones $c: warnings $W, su fila dice $warn  ->  $A"
    VIDA=1
    if [ "$NR" -gt 0 ] && [ "$NF" = 0 ]; then :; else
      VIDA=0; falla "aserciones $c: prueba de vida: $NR «Running», $NF fuera de <target>/aserciones/deps/: no corrio con el perfil aserciones"
    fi
    if [ "$RC" = "124" ]; then
      VIDA=0  # sin JSON: la pasada no acabo, y su ROJO ya esta arriba
    elif [ "$NJ" -gt 0 ] && [ "$NSA" = 0 ]; then :; else
      VIDA=0; falla "aserciones $c: prueba de vida del perfil: $NSA de los $NJ artefactos del JSON de cargo van sin debug_assertions: el perfil $PERFIL_AS no las enciende (Cargo.toml raiz, [profile.aserciones])  ->  $AJ"
    fi
    [ "$RC" = "124" ] || [ "$NX" = 0 ] \
      || { VIDA=0; falla "aserciones $c: prueba de vida del perfil: $NX «Running» que no son ejecutables del JSON de cargo: lo que corrio no es lo que el JSON describe  ->  $AJ"; }
    [ "$VIDA" = 0 ] || N_AS=$((N_AS + 1))
  done
  [ "$N_AS" = "$(wc -w <<< "$ASERCIONES_PASADA")" ] \
    || falla "aserciones: prueba de vida: corrieron con el perfil $N_AS de los $(wc -w <<< "$ASERCIONES_PASADA") crates de ASERCIONES_PASADA"
  RAZON=$(awk -v a="$T_AS" -v r="$T_REL" 'BEGIN { if (r > 0) printf "%.2f", a / r; else print "sin denominador" }')
  msg "   ── total: ${T_AS} s; sus filas de release en esta corrida, ${T_REL} s: razon ${RAZON} (el tope, 2, en caliente o tras tocar; en frio o con un solo perfil por compilar, no decide; no es compuerta)"
  [ "${#fallos[@]}" != "$F_AS0" ] \
    || msg "  OK  $N_AS crates con las aserciones de depuracion encendidas -cada artefacto, en el JSON de cargo-, con los pines de su fila"
fi

# ── 3 ter · el ARTEFACTO (tools/artefacto.sh --check, §401): la PROPIEDAD, no un pin ──
# ⚠️ §824: desde aqui el --check compila el kit en DOS RAICES -el arbol y su copia en
# target/artefacto/raiz2-, no dos target del mismo arbol; comprueba que cada binario salio de las
# fuentes de su raiz (el dep-info de cargo), y busca las rutas de las raices, de HOME, de CARGO_HOME,
# de RUSTUP_HOME, de <sysroot>/lib/rustlib y `/home/x/`; si no puede buscarlas, ROJO. cargo corre con
# `env -i` y cada target lleva el sello del host con que se compilo (tools/artefacto.sh). Su coste, en
# cuatro nucleos y con la carga media del minuto entre parentesis: con la primera version del §824
# (15-10-2026), 164 s en frio, 41 s sin cambios y 48 s con un cambio en crates/zk-ssl-verify (2,6 a
# 8,8); con la segunda (20 y 21-10-2026), 66, 23 y 29 s (2,8 a 3,8); con la que se sella (21-10-2026),
# de 62 a 65 s en frio -y lo es la primera pasada tras cambiar la herramienta o el host- y 24 s
# sin cambios (2,3 a 3,9) (el de antes: 156, 51; en los catorce --sello del §811, de 35 a 69); todo
# eso, sobre las fuentes de b6994c3. Sobre las del kit de 5fd5212, el §817 (21-10-2026, en el arbol
# de antes del rebase, con las mismas), 177 s en frio y 53 s sin cambios (4,9 a 5,7); sobre el de
# este sello, medido sobre el §852 (21-10-2026; sobre el §859 lo que compila el kit es lo mismo, y no
# se volvio a medir), 182 y 243 s con la espera del cerrojo de cargo, y sin ella unos 171 y 50,
# estimados (5,7 a 8,3). Cabe en el sello, y se queda en el.
msg ""
msg "== CANON · el artefacto =="
T_ART0=$(date +%s)
if bash tools/artefacto.sh --check > "$OUT/artefacto.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/artefacto.txt" | sed 's/^ *OK *//') ($(( $(date +%s) - T_ART0 )) s)"
else
  tail -n 5 "$OUT/artefacto.txt" >&2; falla "el artefacto no es reproducible o no pasa el manifiesto"
fi

# ── 3 ter binarios · el NODO y el TESTIGO desde dos raices (tools/artefacto.sh --binarios, §824) ──
# El nodo de produccion (sin `dev`) y el testigo, con las MISMAS reglas que el kit (un productor:
# reglas(), en tools/artefacto.sh), el mismo entorno y el mismo sello del host, en las mismas dos
# raices: la misma huella byte a byte o ROJO, con las secciones que difieren. Va desde --largo, no en
# el sello, con el criterio de los niveles: una compuerta que encarece cada sello acaba saltandose
# (doc/METODO.md §«6. El canon y los bancos», el §225). Medido en cuatro nucleos, con la carga media
# del minuto entre parentesis: con la primera version del §824 (15-10-2026), 474 s en frio, 5 s sin
# cambios y 188 s con un cambio en crates/zk-ssl (2,2 a 4,4); con la segunda (20 y 21-10-2026), 341, 4
# y 164 s (3,9 a 7,3); con la que se sella (21-10-2026), 366 s en frio y 4 sin cambios (2,4 a 3,7),
# sobre las fuentes de b6994c3 (sobre el arbol del §817 y sobre el de este sello, sin medir): lo que
# pagaria cualquier sello que toque la capa, sobre los 470 a 926 de un --sello (cada compilacion en
# frio de los dos juntos, en el censo de H6, de 172 a 516 s). Deja en disco
# target/artefacto/tA y target/artefacto/raiz2, 624 y 687 MB (unos 1,3 GB), que no se borran
# solos (doc/KIT.md, «El nodo y el testigo, desde dos raíces», dice como). Lo que NO prueba: que el
# operador corra ese binario (la misma seccion; la 54 sigue abierta).
if [ "$PEDIDO" -ge 2 ]; then
  msg ""
  msg "== CANON · el nodo y el testigo, desde dos raices =="
  T_BIN0=$(date +%s)
  if bash tools/artefacto.sh --binarios > "$OUT/binarios.txt" 2>&1; then
    msg "  OK  $(tail -n 1 "$OUT/binarios.txt" | sed 's/^ *OK *//') ($(( $(date +%s) - T_BIN0 )) s)"
  else
    tail -n 12 "$OUT/binarios.txt" >&2; falla "el nodo o el testigo no salen iguales desde dos raices"
  fi
fi

# ── veredicto ───────────────────────────────────────────────────
msg ""
if [ $rojo -eq 0 ]; then
  msg "== CANON $NIVEL: VERDE =="
  if [ "$NIVEL" = "--completo" ]; then
    mkdir -p "$(dirname "$SELLO_FILE")"
    printf '%s %s\n' "$(git rev-parse --short HEAD)" "$(date -Iseconds)" > "$SELLO_FILE"
    msg "   anotado en $SELLO_FILE — para que nadie tenga que acordarse."
  fi
else
  msg "== CANON $NIVEL: ROJO · ${#fallos[@]} fallo(s) =="
  for f in "${fallos[@]}"; do msg "   · $f"; done
fi
msg "   salida integra en $OUT/"
exit $rojo

# ── AL PIE (⚠️ §811, 14-10-2026): los tiempos de la cabecera, remedidos ────────
# Va DESPUES del `exit` y no se ejecuta. Esta aqui y no arriba para que
# ninguna linea de arriba se mueva: las citas por linea a este fichero, en los
# documentos vivos, siguen señalando lo que señalaban. Cada cifra corregida
# arriba lleva su «⚠️ §811» y remite a este pie.
#
# ⚠️ §811, 14-10-2026: dos unidades, y arriba se dicen. El RELOJ es lo que tarda
# la pasada entera. Los TESTS son la linea «total» de la tabla: la suma de lo
# que tarda el `cargo test` de cada fila, compilacion incluida (el bloque
# «1 · los tests, crate a crate»). Los 151 s y los 53,6 min de antes eran de
# tests (§226). Arriba, `--sello` da el reloj, y `--completo` los tests: el
# reloj de un `--completo` no consta.
#
# ⚠️ §811, 14-10-2026: `--sello` decia **151 s**, los tests de sus filas en el
# §226. Catorce `--sello` VERDES, del §776 al §789, el 14-10-2026 entre las
# 08:01 y las 12:02 (UTC), en el contenedor de una sesion: de 470 a 926 s de
# reloj (mediana 579) y de 318 a 765 s de tests (mediana 415,5), de los que la
# fila de la capa sola se lleva de 111 a 352. La carga no se anoto con
# ninguna, y no consta que la maquina estuviera tranquila. El del propio
# `b6491ef` (§794), en la tanda siguiente: 514 s de reloj y 344 de tests,
# mientras corria a la vez el `--largo` de la completitud (`tools/bancos.sh`,
# al pie). Lo que no son tests suma ya de 152 a 227 s de reloj; entre ello,
# los dos tramos cronometrados de la segunda implementacion, de 44 a 52, y el
# artefacto, de 35 a 69. En frio, una pasada sobre `7d13f26` tardo 16 min
# 45 s de reloj, con carga de 1,6 a 2,8 (`doc/blueprint-v2.md`, 2.1). En una
# maquina tranquila, sin medir.
#
# ⚠️ §811, 14-10-2026: `--largo` decia **~20 min**: la suma 151 + 347 + 689 s
# del §226, no una medida, y ningun asiento da el tiempo de una pasada de
# `--largo`. Con el `--sello` de hoy esa suma ya no vale, y otra sin medir no
# se escribe.
#
# ⚠️ §811, 14-10-2026: `--completo` decia **53,6 min**, los 3.217 s de tests del
# §226. La pasada de `--completo` mas reciente que consta es la del §629
# (01-10-2026, en la maquina del autor, sobre `2c0671e`): halo2 386 s, plonk
# 730 s y zk-core 2.103 s, y «3.481 s en total» de sus crates: 58 min de
# tests. Su reloj no consta, y en este sello no se ha medido.
#
# ⚠️ §811, 14-10-2026: de esa misma pasada del §629 salen las demas cifras
# corregidas arriba:
# - «Los TRES niveles» decia que `zk-core` cuesta **33,9 minutos**, los
#   2.032 s del §226 (entre medias el §260 midio 2.075): ahora 35, los 2.103.
# - «Meter 54 minutos en cada sello»: los 54 eran los 3.217 s (53,6 min) del
#   §226, que escribio la frase; el §260 midio despues 3.268. Ahora 58, los
#   3.481 s de tests del §629.
# - En la TABLA, halo2 decia «438 s» y plonk «749 s»: las cifras en frio del
#   banco F.1 (§224), que el canon bajo a 347 y 689 en el §226. Ahora 386 y
#   730. La nota de zk-core suma los 2.103 s.
# - `estado_completo` decia «Los 54 min del nivel completo no los ha corrido
#   nadie»; ahora, que ningun `--completo` ha pasado VERDE aqui -la foto solo
#   se escribe con el VERDE- y los 58 min de tests del §629.
#
# ⚠️ §811, 14-10-2026: `--bancos` decia **~15 min**, y `estado_bancos`, «Los
# ~15 min de --bancos»; los escribio el §582, que midio 779 s con 15 bancos.
# Lo que consta despues esta en el pie de `tools/bancos.sh`: la pasada entera
# mas reciente que consta es la del §709, con 24 bancos, 1.891 s con las
# compilaciones, y `estado_bancos` la da ahora. Con los 25 que corre hoy una
# pasada (el 26, el de los cortes, va FUERA desde el §804), sin medir. ⚠️ §712: decia 25 y 26; con `banco_bifurcacion.sh` una pasada corre 26 (el 27, el de los cortes, FUERA), y de `bifurcacion` tampoco consta ninguna pasada entera: sin medir. ⚠️ §722: decia 26 y 27; con `banco_historia_comun.sh` una pasada corre 27 (el 28, el de los cortes, FUERA), y de `historia_comun` consta una corrida suelta (su asiento), no una pasada entera: sin medir.
#
# ── ⚠️ §822, 15 y 21-10-2026: el arbol de Merkle de zk-core, disperso (la 75) ───
# `SimpleMerkleTree` (`crates/zk-core/src/merkle.rs`) rellenaba hasta 2^20
# hojas y hasheaba el arbol entero en cada `build`, 1.048.575 hashes Poseidon
# en serie para usar ocho; ahora guarda solo los nodos que cubren hojas dadas,
# y la raiz y los caminos son los del denso: la raiz la ata un KAT sacado del
# denso; los bits de cada camino, los de su indice, que son los del denso; y
# los hermanos, que el camino suba a esa raiz. Lo usan los tests de `zk-core`,
# su ceremonia, las medidas de su `performance.rs` y el test de `iso-bridge`.
# Medido el 15-10-2026 en 4 CPU compartidas con otras sesiones, con el binario
# ya compilado (`--no-run` antes), reloj y carga tomados al empezar a correr
# cada orden, no mientras esperaba a otras compilaciones, y
# `nice -n 19 cargo test --release --locked`:
# - la biblioteca de `zk-core` (`-p zk-core --lib`): con el denso, 3.546 s de
#   pared y 4.959 de CPU, 73 tests, carga 8,9 al empezar, una sola corrida;
#   con el disperso, 51 y 38 s de pared, 75 tests, carga 10,0 y 8,0.
# - la fila de `zk-core` entera (`-p zk-core`, como la corre esta tabla), con
#   el disperso: 76 de 76 en 486 s, 37 de la biblioteca y 448 de la ceremonia,
#   carga 8,2. Con el denso, entera, no se midio. El 21-10-2026, con el
#   `merkle.rs` final (77 tests) y menos carga: 77 de 77 en 271 s, 15 de la
#   biblioteca y 256 de la ceremonia, carga 3,9 al empezar. Las dos, sobre
#   `4d8017d`. El mismo dia, sobre `5fd5212` (el §817) con este cambio encima
#   y con mas carga: 77 de 77 en 555 s, 48 de la biblioteca y 506 de la
#   ceremonia, carga 7,8; la biblioteca sola, 76 de 76 en 47 s, carga 6,3.
# - `iso-bridge`: de 53 y 58 s de pared a 3 y 4; en el `--sello` de `0315b14`
#   (el §813 sobre `b6994c3`, la misma `zk-core`) su fila tardo 52 s. El
#   21-10-2026, 1,3 s sobre `4d8017d` (carga 4,0) y 5,2 s sobre `5fd5212`
#   (carga 7,7). Lo de `5fd5212`, medido antes del rebase: el mismo codigo.
# Eso corrige, sin moverlas de linea, dos cosas de arriba:
# - «Los TRES niveles» dice que el coste de `zk-core` es SINTETIZAR el
#   circuito. El §225 lo dedujo por descarte -generar la prueba añadia un 5 %
#   y rayon un 17 %- y no midio el arbol. En la biblioteca no lo era: era
#   construir el arbol denso, y con el disperso sus tests -75 el 15-10-2026,
#   76 el 21-10-2026- caben en menos de un minuto en esta maquina, sintesis y
#   pruebas incluidas. Lo que pesa ahora en la fila es la ceremonia: 448 de
#   486 s el 15-10-2026; 256 de 271 y 506 de 555 el 21-10-2026.
# - Los 35 minutos de `zk-core` y los 58 de `--completo` son del §629, medidos
#   con el arbol denso, y se quedan como su cifra. Con el denso, la biblioteca sola
#   -3.546 s- casi llenaba aqui los 3.600 s del timeout de la fila, que cubre
#   biblioteca y ceremonia; en la maquina del autor, la fila entera tardo
#   2.103 s (§629). El timeout no se toca, y tampoco el de `iso-bridge`
#   (300 s), ni el nivel de ninguna fila: eso no lo decide este sello.

# ── CEGUERA DECLARADA (⚠️ §864, la 94): lo que el canon NO ve, clase por clase ────────
# Va despues del `exit`, al final del fichero, detras de los pies del §811 y del §822, como ellos,
# para no mover las lineas que otros textos citan; la cabecera la nombra. «toda herramienta,
# ayudante o barrido declara su cobertura, y la declaración se comprueba» (CONTRIBUTING.md §1). La
# parte que se comprueba es la clasificacion de los miembros, que es codigo -el bloque 0 bis: cada
# miembro en una clase, y ROJO si falta o sobra uno-; lo demas es esta declaracion. Decidido por
# delegacion del autor en el §864, REVERSIBLE (§642): cada clase dice donde se vigila y cuando se
# reabre.
#
# (a) Las aserciones de depuracion: `debug_assert!` y lo que vive bajo `cfg(debug_assertions)`, que
#     en release no existen. Por clases de miembros:
#     - ASERCIONES_PASADA, lo que un tercero compila sin el probador -el kit, `zk-ssl-verify`; su
#       clausura, `zk-ssl-hash`, `zk-ssl-air` y `zk-ssl-medio`; y el guardian del testigo,
#       `zk-ssl-guardian`-: la VE el 1 bis, en --sello, y comprueba en el JSON de cargo que el
#       perfil las enciende. Se reabre si una corrida posterior da una razon mayor que 2 en uno de
#       los dos escenarios que fijaron el nivel -en caliente, o tras tocar `zk-ssl-hash`-: baja a
#       --completo, con su foto y COMPLETO_CUBRE ampliado; la que compila un solo perfil -en frio, o
#       con release al dia y la pasada por compilar- se informa y no decide. Y se reabre si un test
#       de los cinco tiene que gatearse en depuracion: gana su pin propio.
#     - CIEGA_PROBADOR: PERMANENTE, nunca en --sello. Costaria como poco otra vez sus filas de
#       release, y la de la capa es la mas cara del sello (§225); y las 70 marcas (⚠️ §713: decia 59, y 18 en `stark-experiment`; el falsador del resto FRI sobre la prueba de edad, `falsadores_resto.rs`, solo en release como sus hermanos de `circuit_edad.rs`, suma una) (⚠️ §719: decia 60, 38 en `zk-ssl` y 19 en `stark-experiment`; los cinco falsadores del verificador del envio v2, el cobro v2, la quema, la emision a pendiente y su subida, solo en release, suman cinco, y el del alias del emisor del §718 en `zk-ssl`, `la_prueba_de_un_emisor_no_verifica_con_su_alias`, que su sello no conto, una) (⚠️ §720: decia 66 y 24 en `stark-experiment`; el falsador de la constancia de vivo de la edad, `un_vivo_que_se_apaga_en_la_fila_6_no_verifica` en `circuit_edad.rs`, solo en release como sus hermanos, suma una) (⚠️ §723: decia 67 y 25 en `stark-experiment`; los dos falsadores del juez de la ventana, `el_juez_rechaza_un_saldo_de_la_ventana` y `el_juez_rechaza_un_importe_de_la_ventana`, y `kat_banda_v2`, solo en release, suman tres)
#       `cfg_attr(debug_assertions, ignore)` -39 en `zk-ssl`, 28 en `stark-experiment` y 3 en
#       `zk-ssl-sdk`- son de la migracion de Winterfell: hoy la pasada mediria las marcas, no el
#       codigo. Se reabre cuando la migracion las decida, o ante un defecto medido en release que un
#       `debug_assert!` propio del probador habria cazado: se mide con el perfil `aserciones` y, si
#       se monta, va en --completo.
#     - CIEGA_FORKS: el 1 bis los ejerce como dependencias, con las aserciones encendidas; sus tests
#       de upstream no corren ahi (⚠️ §711: ni los siete de Arqueo, `winter-air/src/proof/tests_sin_panico.rs`) (⚠️ §715: decia siete; los seis de `winter-air/src/proof/tests_canonica.rs`, tampoco: trece). Se reabre cuando aterrice la migracion.
#     - CIEGA_OTROS: PERMANENTE. No son la clausura del kit ni la del nodo, y hoy ninguno entra en
#       ellas; se reabre si uno entra.
#     Y los tests con `#[ignore]` sin condicion no los corre ningun nivel: la tabla cuenta sus
#     ignorados, no los ejecuta, y check_tests exige que cada uno se declare instrumento de medida.
#     Uno, el instrumento del vecino, no corre en ningun perfil; lo que se hace con el lo decide la
#     125, no esto.
# (b) Los avisos del build normal y (c) los de los ejemplos: CERRADAS. Los binarios y los ejemplos,
#     la puerta del §834; las bibliotecas, `cargo test`, que ya las compila en su perfil normal -la
#     fila de `ceremony` cuenta sus avisos «(lib)»-; el nodo sin `dev`, el bloque del §784, con su
#     pin exacto desde el §864. Residuo declarado: el generador de `kat_xmss` (§834), y los avisos
#     que solo saldrian sin las features que encienden las dependencias de desarrollo -con el
#     resolver 2, `cargo test` las unifica-, sin instancia conocida. Se reabre si la puerta del §834
#     sale de --sello -y entonces esto dice en que nivel se ven- o si aparece una instancia del
#     residuo, que gana su pasada.
#     Y fuera del build normal, el codigo que solo compila con una feature que no enciende ninguna
#     fila ni ninguna puerta, en ningun nivel: de el no ve el canon ni los avisos, ni los errores,
#     ni los tests. Con instancia, medidas con `cargo tree --locked -e features` en el §864:
#     `parallel` de `ceremony` (4 `cfg(feature = "parallel")`; `zk-core` la usa sin features);
#     `test-setup` de `nova-experiment` (el modulo de sus 3 tests, `folding_chain.rs`, que su fila
#     declara); y en los forks, `concurrent` de `winter-prover` (18 `cfg(feature = "concurrent")`) y
#     su `async` (8 `#[maybe_async]` y 9 `maybe_await!`, que solo con ella son `async`), y el
#     `no_std` de `winter-air` (4 `cfg(not(feature = "std"))`; `std` va por defecto). Declarada, no
#     vista: las dos primeras son de CIEGA_OTROS, y las de los forks, de la migracion de Winterfell.
#     Se reabre si una fila o una puerta enciende una de esas features, si uno de esos crates entra
#     en la clausura del kit o del nodo, o cuando la migracion decida los forks. La lista es la
#     medida de hoy, no una comprobacion: una feature nueva que no encienda nadie no la pone ROJA
#     nada.
# (d) La prosa que envejece: PERMANENTE. Una afirmacion semantica no admite un test discriminante
#     (PRINCIPIOS §9). Lo que tiene productor ya tiene herramienta -check_cifras, check_figures,
#     verificar_citas, check_catalogo (§777), check_superficie (§801) y check_publicadas-; lo demas
#     se corrige con marca cuando se mide. Se reabre cada vez que una subclase gana productor: gana
#     su herramienta, como la (e).
# (e) Los dominios que viajan como cadena: CERRADA en el §789 (R6 de check_dominios).
# (f) Los bancos: VIGILADOS desde el §582, no obligados (`--bancos` y `estado_bancos`).
