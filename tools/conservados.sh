#!/usr/bin/env bash
# ======================== LOS CONSERVADOS ========================
#
#     bash tools/conservados.sh              # cada catalogo conservado, con el kit de SU commit
#                                            # · **~2 min** en frio (§798: corrido una vez, abajo)
#     bash tools/conservados.sh --en-seco    # el plan y lo que no necesita compilar · **~1 s**
#     bash tools/conservados.sh --frio       # lo mismo, con el target de cada kit borrado antes
#     bash tools/canon.sh --conservados      # lo mismo, desde el canon (`--en-seco` y `--frio`
#                                            # van detras: `--conservados --en-seco`)
#
# **Por que existe (§798).** `spec/vectors/<version>/` guarda los catalogos
# que el kit del arbol ya NO verifica y que no se reescriben (regla 2 del
# PROCESO): `0.3/` desde el S538 (D-AB del RFC-0009: nacieron con el
# probador apagado, y el kit 0.4 rechaza su forma, D-AD) y `0.4/completitud`
# desde el §682 (un `StaleState` ya no resuelve solo: el kit de hoy les da el
# cuarto estado, salida 3). Cada MANIFIESTO dice en su cabecera que kit los
# verifica -«el zk-ssl-verify del arbol en <commit>»- y hasta el §798, entre
# medida y medida, eso no lo vigilaba nada. Los de `0.3/` los midio LA MANO
# dos veces: el §620 (38 de 38, binario `775428d29ee1ca1b`) y el §693 (41 de
# 41, compilando `0eda58c` de su tarball porque el clon era superficial).
# Los dos de `0.4/completitud`, con el kit del arbol en `41f765d`, no los
# midio la mano: los paso el canon del §681, cuando aun eran entradas del
# catalogo vivo, con las mismas dos lineas (`41f765d:tools/canon.sh:453`).
# El «2 de 2» del §682 es el de la release `arqueo-verify-v0.4.2`, binario
# `7734376463d120b0`, que este nivel no juzga (abajo, Lo que NO hace).
# Cuando el §693 endurecio el arnes, tres de esos manifiestos pasaron
# a salir `SIN CAUSA`, y lo que afirman sus cabeceras (todas sus entradas
# dicen lo que deben, asiento 620: 38 de 38 en el §620) habria dejado de
# valer con el arnes del arbol; se vio porque ese sello se acordo de
# compilar el kit viejo y mirar, y por eso los tres llevan una linea nueva.
#
# ## Lo que lo hace robusto
#
# 1. **La lista se LEE del arbol**: todo `spec/vectors/<x.y>/<familia>/`. Un
#    directorio conservado sin fila en la TABLA es ROJO, una fila sin
#    directorio tambien, y uno sin MANIFIESTO.txt tambien. El agujero del
#    §223 del canon, cerrado igual: un conservado nuevo no puede quedarse
#    fuera por olvido.
# 2. **El commit lo dicen DOS, y tienen que decir lo mismo.** La TABLA lo da
#    entero (40 hex); la cabecera del manifiesto lo nombra, y la cabecera no
#    se reescribe. Toda mencion «zk-ssl-verify del arbol en <hex>» de sus
#    comentarios tiene que ser un prefijo del de la tabla, y tiene que haber
#    al menos una. Si no cuadran, la fila NO se edita para que cuadre: se
#    MIDE. Los comentarios se leen POR TRAMOS, no linea a linea: cada tramo de
#    lineas `#` seguidas se junta en una, sin el `#`, con los saltos hechos un
#    espacio -o nada, tras una linea que acaba en «-»-; una linea `#` vacia, o
#    de solo blancos, sigue en el tramo y no anade nada. Cada linea pierde
#    ANTES los caracteres invisibles de formato (Cf: el espacio de ancho cero,
#    el guion blando) y DESPUES va a la forma NFKC de Unicode (la tilde
#    combinada se compone, aunque un Cf la separase de su letra; el espacio
#    duro, el fino y los de ancho completo pasan a su ASCII), con todo blanco
#    hecho un espacio y en minusculas. Entre las palabras de la mencion el
#    espacio es OPCIONAL: un Cf puesto EN LUGAR del espacio pega las palabras
#    («del<U+200B>arbol» queda «delarbol») y tambien se mira. Asi una mencion
#    partida en dos lineas («...zk-ssl-verify del» / «# arbol en <otro>...», o
#    «...el zk-ssl-» / «# verify del arbol...», y tambien con una linea `#`
#    vacia en medio) se mira, y un hex en mayusculas, un «del arbol» con un
#    espacio duro o un «árbol» con la tilde combinada tambien. Las cabeceras
#    dicen «arbol» (anclas ASCII, punto 5 del canon); con «árbol» o «Árbol»,
#    o con el mando o el hex entre comillas invertidas, tambien se mira. El
#    hex se lee ENTERO, sin tope de cifras y con lo que lleve pegado: uno de
#    41 cifras, o con una letra detras, no es prefijo de nada y es ROJO. NO ve
#    otra frase («el kit de 0eda58c»), ni un hex de menos de 7 cifras o
#    partido por el salto antes de la septima, ni lo que sigue a un salto
#    DENTRO del hex («...en 0eda58c9» / «# ffff» se compara solo por
#    «0eda58c9»: el salto es un espacio, y «a» o «de» tambien son hex), ni
#    una palabra partida por el salto sin guion o con un guion blando U+00AD
#    (se quita antes de juntar el tramo), ni un salto que sea un CR suelto (el
#    texto se parte solo por LF), ni un tramo cortado por una linea en blanco
#    SIN `#`, ni otra grafia que la NFKC no lleve al ASCII (un
#    guion Unicode en el mando, una «а» cirilica), ni un invisible que no sea
#    de formato ni blanco: entre ellos el CGJ U+034F o un selector de variante,
#    que son marcas (Mn), y un relleno Hangul U+3164, que es letra (Lo).
# 3. **El commit es del ARBOL**: existe en este clon y es antepasado de HEAD,
#    que es lo que la cabecera afirma («del arbol»). Un clon superficial, como
#    el del §693, da ROJO con su remedio, nunca un VERDE de lo que no se pudo
#    construir. Y en el commit tienen que estar el crate y su arnes.
# 4. **Un kit por COMMIT, no por catalogo**: los cuatro de `0.3/` comparten el
#    de `0eda58c` y se construye una vez.
# 5. **Se construye DE SU COMMIT**, en un worktree temporal (`git worktree add
#    --detach`, bajo `$TMPDIR`) y con `--locked` (§694): un lock que no cuadre
#    con su toml es ROJO, nunca un lock reescrito que compile otra cosa. El
#    target es PROPIO, `.canon/conservados/<commit>/target`: no pisa el
#    binario del arbol -lo que el §693 hizo a mano con un target aparte- y
#    queda caliente para la pasada siguiente; `--frio` lo borra antes. El
#    worktree tiene que quedar LIMPIO tras construir: un kit que escribe en el
#    arbol de su commit es un fallo, y se nombra.
# 6. **Dos jueces, y los dos VERDES.** El arnes DE SU COMMIT
#    (`<worktree>/tools/conformidad.sh`), que es el que la cabecera afirma, y
#    el DEL ARBOL, el de hoy: desde el §693 es mas estricto -NO SOPORTADO, la
#    causa en la linea del ROJO, SIN CAUSA- y es el que, al endurecerse, habria
#    dejado de pasar tres de ellos sin que nada lo dijera. Un conservado que el
#    arnes de hoy no pasa tiene que verse el dia que el arnes cambia, no cuando
#    alguien se acuerde.
# 7. **Pin EXACTO de entradas por catalogo**, y su prueba de vida: el pin
#    tiene que ser lo que el manifiesto tiene -contado con el bucle del
#    arnes- y lo que CADA arnes dice que juzgo. Cero catalogos es ROJO: un
#    nivel que no mira nada no es VERDE. Una entrada nueva en un conservado
#    (como las del §693) se MIDE y se edita el pin, nunca al reves.
# 8. **No se para en el primer fallo**: da el inventario entero. Un kit que no
#    construye deja ROJOS sus catalogos, con su nombre, y los demas se juzgan.
# 9. **El arbol, como estaba**: cada worktree se quita en cuanto sus catalogos
#    estan juzgados, y al salir, pase lo que pase (`trap`; un INT sale con
#    130 y un TERM con 143). Desde `canon.sh --conservados` el nivel corre en
#    segundo plano y nace con el INT IGNORADO, y el canon le pasa como TERM
#    el TERM y el INT (canon.sh, `parar_conservados`). Una senal que llega
#    entre el `&` de un hijo y la linea que anota su PID se ANOTA y se atiende
#    en cuanto el PID esta anotado, aqui (`anotar`, `atender`) y en el canon:
#    sin eso, el hijo seguia huerfano. Una senal que llega
#    durante un juicio espera a que acabe el arnes en curso (segundos, con un
#    tope de 600 s; no medido): el juicio corre en primer plano. Despues se
#    comprueba que git ya no lista el worktree. Si
#    `git worktree remove --force` falla es ROJO, y aun asi se limpia: uno
#    BLOQUEADO -asi deja el suyo un `git worktree add` cortado a medias,
#    «initializing»- no lo quita `remove --force` ni lo poda `prune`, y como
#    el directorio es nuestro (de `mktemp`) se desbloquea, se borra y se poda.
#    ⚠️ Ese `git worktree prune` va sin `--expire` y poda EN EL ACTO tambien
#    las entradas de OTROS worktrees cuyo directorio ya no existe y que nadie
#    bloqueo. `git gc` no hace eso: poda con `--expire` y el plazo de
#    `gc.worktreePruneExpire`, tres meses por defecto (medido en el §798 con git
#    2.43.0: una entrada huerfana recien hecha sigue tras `git gc` y se va con
#    `git worktree prune`). Solo corre si `remove --force` fallo.
#    `git status` es el mismo antes y despues, o es ROJO. Los registros van a
#    `$OUT`, o a `$TMPDIR/conservados` (`/tmp` sin `TMPDIR`), y solo se crean
#    si algun kit se va a construir.
# 10. **`--en-seco`** comprueba 1, 2, 3 y 7 contra el manifiesto -lo que no
#    necesita compilar-, imprime el plan -cada kit, su worktree, su target y
#    las ordenes exactas- y sale sin crear, construir ni escribir nada. Exit 0
#    si el plan es VERDE.
#
# ## Lo que NO hace (declarado ANTES de medir)
#
# - **Su ruta real se ha corrido UNA vez** (§798), en un clon desechable del
#   arbol del sello, con rustc 1.94.1: VERDE en 125 s. Los dos kits, cada uno
#   construido de su commit en su worktree con `--locked`, en 41 s y 37 s:
#   `0eda58c` da el binario `775428d29ee1ca1b`, el mismo que midio el §620, y
#   `41f765d`, `a9644c19f9ca7518`; con los dos arneses, 12 de 12, 10 de 10,
#   10 de 10, 9 de 9 y 2 de 2. No hay otra pasada, y ningun nivel que corra
#   solo la repite. Antes de esa, lo probado (§798), en un clon desechable
#   con `CANON_RAIZ`: `--en-seco`, VERDE; los dos arneses -el
#   de `0eda58c` y el del arbol- con el binario del §620 (`775428d29ee1ca1b`)
#   sobre los cuatro de `0.3/`, 12 de 12, 10 de 10, 10 de 10 y 9 de 9 con
#   cada uno; la tuberia de punta a punta con un `cargo` FALSO, que deja ese
#   binario para `0eda58c` y un guion que imprime lo esperado para `41f765d`
#   (VERDE, y el `.canon/ultimo-conservados` que dejo es el de ese falso, en
#   el clon, que se borro); y sus falsadores en ROJO: la tabla, el arbol, la
#   cabecera (partida, partida en el mando tras «zk-ssl-», con el hex en
#   mayusculas, con tilde, entre comillas invertidas, sin mencion), el kit
#   (ausente, no antepasado, que no construye), el clon superficial, el
#   worktree bloqueado (ROJO y limpio) y el TERM (143 y limpio). Y, por
#   `canon.sh --conservados` con un `cargo` falso dormido, el TERM, el INT y
#   el INT a todo el grupo, que es el Ctrl-C: 143, 130 y 130, y limpio. Con
#   una `canon.sh` que lo lanza en primer plano, el TERM da 143 y el nivel
#   sigue VIVO, con su worktree listado. Tras pasar la cabecera a Python y
#   cerrar la ventana del PID (§798, en otro clon desechable que se borro, y con
#   `TMPDIR` propio): ROJOS tambien una mencion con 41 cifras, con una letra
#   pegada al hex, con un espacio duro o uno fino en lugar del espacio, con
#   uno de ancho cero (donde iba no quedo anotado: con esa forma, ROJO dentro
#   de una palabra o junto a un espacio y VERDE en lugar del espacio, abajo),
#   con la tilde combinada (y en mayusculas), con una cifra de ancho
#   completo, en CRLF y en latin-1 -con la primera forma, en `awk`, todos
#   salvo el CRLF salian VERDES junto a la mencion buena-; VERDE, como
#   declara el punto 2, la cifra que sigue a un salto dentro del hex; sin
#   python3, exit 2, y con un python3 que falla, ROJO. Con un `kill -TERM $$`
#   metido en una copia justo tras el `&` de la construccion, o justo antes, o
#   tras anotar el PID: 143, sin `cargo` vivo y limpio (antes de cerrarla, el
#   mismo falsador dejaba el `cargo` falso vivo); en una copia de canon.sh
#   tras su `&`: 143, sin nivel ni `cargo` vivos y sin worktree (antes, el
#   nivel seguia huerfano, con su worktree listado). El TERM, el INT y el INT
#   al grupo, rehechos: 143, 130 y 130, limpio. Y la tuberia, rehecha con un
#   binario oraculo que dice lo que pide cada manifiesto: VERDE, los 43 por
#   los dos arneses. Tras quitar los Cf ANTES de la NFKC, hacer opcional el
#   espacio entre las palabras de la mencion y no dejar que una linea `#`
#   vacia anada nada (§798, en un clon desechable nuevo que se borro, con
#   `TMPDIR` propio, y cada caso con las dos formas, junto a las menciones
#   buenas de `0.3/edad`): ROJOS tambien el de ancho cero, el U+2060 y el
#   U+FEFF EN LUGAR del espacio («del<U+200B>arbol», «en<U+200B>ffffffff»), la
#   mencion entera sin espacios o con un U+200B en cada uno, un guion blando
#   entre la «a» y su tilde combinada, y una linea `#` vacia, dos, una de
#   blancos o una con solo un U+200B en medio de la mencion partida -con la
#   forma anterior, VERDES los once-; siguen ROJOS los de antes, y el de ancho
#   cero dentro de una palabra o junto a un espacio, con las dos; y VERDES,
#   como declara el punto 2, el CGJ, el selector de variante, el relleno
#   Hangul, el guion U+2011 en el mando, la «а» cirilica, otra frase, la linea
#   en blanco sin `#` y la cifra tras un salto dentro del hex.
# - **No juzga las releases publicadas**, aunque la cabecera de
#   `0.4/completitud` nombre tambien la `arqueo-verify-v0.4.2`: bajarlas es un
#   acto con red, y la mide el asiento que la registra.
# - **No fija el rustc**: construye con el de la maquina (o el que fije el
#   commit, si fija uno; hoy ninguno) y lo anota en `.canon/ultimo-conservados`.
#   El binario NO es un artefacto -sin remap, otra huella con otro rustc: el
#   §620 dio `775428d29ee1ca1b` con 1.94.1 y el §693 `a2cce0c5bdb6b4b5` con
#   1.97.0-: es una medida. Que un rustc nuevo deje de compilar un kit viejo
#   es justo lo que este nivel tiene que ver, y el canon avisa cuando cambia.
# - **No corre los tests del kit viejo**: solo su mando, sobre su manifiesto.
# - **No es compuerta.** Como `--completo` y `--bancos`, es disciplina: cada
#   pasada VERDE deja constancia en `.canon/ultimo-conservados`, y TODA
#   invocacion del canon dice cuando fue y si cambio lo que ejerce. NO lo
#   vence un cambio en `crates/`: los kits son de su commit, y lo que se
#   mueve es el arnes, la tabla, los catalogos conservados y el rustc.
#
# ## El tiempo (estimado, NO medido: se mide en la primera pasada y se edita)
#
# Con los cuatro nucleos libres y el target frio: **18 s** el kit de
# `0eda58c` -medido sin querer: la construccion del §620, del `.cargo-lock`
# de su target a su binario, 75 unidades- y ~25 s el de `41f765d`, estimado
# por la clausura que `cargo metadata --locked --offline` resuelve para Linux
# (68 paquetes frente a 54: entran `zk-ssl-medio` y `ml-dsa`); dos checkouts
# de 25 y 49 MB, unos segundos; y el juicio, 43 entradas por dos arneses: 3 s
# con el binario del §620 sobre los cuatro de `0.3/`. **~1 min.** Con la
# maquina cargada como en el §704 (de 1,0 a 1,3 nucleos), **~3 min**. Con el
# target caliente se recompilan igual los crates POR RUTA de cada clausura
# (cinco y seis: el kit, el nucleo, las AIR, el medio en `41f765d` y los dos
# del fork de winterfell), porque el worktree es nuevo en cada pasada; los
# del registro, no. Los dos locks resuelven con `--locked --offline` en la
# maquina del §798: alli la primera pasada no necesita red.
#
# ================================================================

if [ -z "${BASH_VERSION:-}" ]; then
  echo "conservados.sh necesita bash: usa 'bash tools/conservados.sh', no 'sh'." >&2
  exit 2
fi
set -uo pipefail
command -v python3 > /dev/null 2>&1 \
  || { echo "conservados.sh necesita python3: lee con el las cabeceras (punto 2)." >&2; exit 2; }

# La raiz sale de la ruta del propio fichero; `CANON_RAIZ` la sobreescribe, como en el canon,
# para PROBAR esta herramienta con una copia mutilada (y en negativo).
RAIZ="${CANON_RAIZ:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
cd "$RAIZ" || exit 2
CONS_FILE=".canon/ultimo-conservados"
CONS_DIR="${CONSERVADOS_DIR:-$RAIZ/.canon/conservados}"
TOPE="${CONSERVADOS_TIMEOUT:-1800}"   # por kit, la construccion; el juicio, 600 s por arnes
TMPB="${TMPDIR:-/tmp}"
LOGS="${OUT:-$TMPB/conservados}"      # se crea al construir el primer kit, no antes (punto 9)
LOGS_USADO=0

msg() { echo "$*" >&2; }
rojo=0
fallos=()
falla() { rojo=1; fallos+=("$1"); msg "  XX  $1"; }

EN_SECO=0; FRIO=0
for a in "$@"; do
  case "$a" in
    --en-seco) EN_SECO=1 ;;
    --frio) FRIO=1 ;;
    *) msg "uso: bash tools/conservados.sh [--en-seco] [--frio]"; exit 2 ;;
  esac
done

# ── LA TABLA ────────────────────────────────────────────────────
# catalogo (bajo spec/vectors/) | commit del kit que lo juzga (40 hex) | entradas | nota
#
# El commit es el que la cabecera de su MANIFIESTO nombra (punto 2); las entradas, las que el
# arnes cuenta. Se MIDE primero y se edita la tabla despues. Nunca al reves.
TABLA=$(cat <<'FIN_TABLA'
0.3/edad          0eda58c9ae234b3cdbfabad754cca655a684dc14  12  S538 lo conserva (D-AB); el §620 lo midio 11 de 11 a mano; el §693, +1 linea con la causa
0.3/pago          0eda58c9ae234b3cdbfabad754cca655a684dc14  10  S538 (D-AB); §620, 9 de 9; §693, +1 linea con la causa
0.3/pendiente     0eda58c9ae234b3cdbfabad754cca655a684dc14  10  S538 (D-AB); §620, 9 de 9; §693, +1 linea con la causa
0.3/rechazo       0eda58c9ae234b3cdbfabad754cca655a684dc14   9  S538 (D-AB): los nueve con prueba que el banco regenera; §620, 9 de 9
0.4/completitud   41f765db550ad0dba6e9338cef107a919cdfc5c2   2  §682 (RFC-0019 E2, D-C): el VERDE de un StaleState, de antes del cuarto estado; el canon del §681 los paso
0.4/completitud   cbe95d7032bcb024e65abe806a855fcf34eda5ef  10  §914 (el corte zkssl/0.5, R6 b): la prenda aceptada y sus cuatro derivados (§614) y los cinco de la banda v1 (§723), pruebas de zkssl/0.4 que el kit 0.5 rechaza por la marca; su tramo, bajo su mencion (punto 10)
0.4/edad          cbe95d7032bcb024e65abe806a855fcf34eda5ef  32  §914 (el corte zkssl/0.5, R6 b): las 31 pruebas de zkssl/0.4 de la edad, con la doble linea de neg-cota-movida; las juzga el kit 0.4.4 del §913, el ultimo sello del cable 0.4
0.4/pago          cbe95d7032bcb024e65abe806a855fcf34eda5ef  21  §914 (R6 b): las 20 pruebas de zkssl/0.4 del pago, con la doble linea de neg-importe-mentido
0.4/pendiente     cbe95d7032bcb024e65abe806a855fcf34eda5ef   7  §914 (R6 b): las seis del cobro pendiente, la v1 y la v2 del §723, con la doble linea de neg-inferior-por-encima
0.4/prenda        cbe95d7032bcb024e65abe806a855fcf34eda5ef   6  §914 (R6 b): las cuatro de la prenda, con las dobles lineas del §693
0.4/rechazo       cbe95d7032bcb024e65abe806a855fcf34eda5ef   7  §914 (R6 b): las siete bandas del brazo decimo, la v1 publicada y las del §723 incluidas
FIN_TABLA
)

# Las entradas de un manifiesto, contadas con el MISMO bucle que el arnes (tools/conformidad.sh):
# una linea cuenta si su primer campo no es vacio y no empieza por `#`.
entradas_de() {
  local n=0 VF R
  while IFS='|' read -r VF R; do
    case "$VF" in ''|'#'*) continue ;; esac
    n=$((n + 1))
  done < "$1"
  echo "$n"
}

h16() { sha256sum "$1" | cut -c1-16; }

# Los commits que nombran los comentarios de un manifiesto (punto 2), leidos POR TRAMOS: cada tramo
# de lineas `#` seguidas se junta en una, sin el `#`, con los saltos hechos un espacio -o nada tras
# una linea que acaba en «-», que asi no parte el mando («zk-ssl-» / «# verify del arbol...»)-; una
# linea `#` vacia, o de solo blancos, sigue en el tramo y no anade nada (con ella en medio, el
# tramo quedaba «del  arbol», con dos espacios, y no casaba). Linea a linea, una mencion partida en
# dos («zk-ssl-verify del» / «# arbol en <otro>») no se veria. Cada linea pasa antes por `norma`:
# PRIMERO sin los caracteres de formato (Cf: el espacio de ancho cero, el guion blando) y DESPUES
# NFKC (la «a» con la tilde combinada es la «á», tambien con un Cf en medio, que quitado tras la
# NFKC la dejaba descompuesta; el espacio duro, el fino y los de ancho completo, el espacio y las
# cifras ASCII), con todo blanco Unicode hecho un espacio y en minusculas Unicode (la «Á» de
# «ÁRBOL» tambien). Entre las palabras de la mencion el espacio es OPCIONAL (` ?`): un Cf EN LUGAR
# del espacio las pega («del<U+200B>arbol» queda «delarbol»), y con el espacio obligatorio no
# casaba. El hex se toma ENTERO -sus cifras sin tope y lo que lleve pegado, `\w*`-, para que uno
# de 41 cifras, o con una letra detras, no pase por prefijo de los 40 de la tabla.
# ⚠️ En Python y no con `awk | tr | grep` bajo `LC_ALL=C`, que fue su primera forma en el §798:
# byte a byte, «del» + espacio duro + «arbol» y la «á» descompuesta no casaban, y con
# `[0-9a-f]{7,40}` un hex de 41 cifras se cortaba en la 40; junto a otra mencion buena, esas tres
# salian VERDES, y las demas de la lista de la cabecera salvo el CRLF (medido en el §798, los diez
# falsadores con las dos formas). `[0-9a-f]` es ASCII (un rango de `re` es de puntos de codigo) y
# la «á» va escrita `\u00e1`: el guion entero es ASCII y no depende del locale de la maquina. Sin
# python3 el nivel no arranca (exit 2), y si falla es ROJO: nunca una cabecera leida a medias.
nombrados_de() {
  python3 -I - "$1" <<'PY'
import re, sys, unicodedata
def norma(s):
    s = ''.join(c for c in s if unicodedata.category(c) != 'Cf')     # ANTES de la NFKC
    s = unicodedata.normalize('NFKC', s)
    return re.sub(r'\s+', ' ', s).strip().lower()
crudo = open(sys.argv[1], 'rb').read()
try:
    texto = crudo.decode('utf-8')
except UnicodeDecodeError:
    texto = crudo.decode('latin-1')
tramos, b = [], None
for linea in texto.split('\n'):          # solo '\n', como el `read` del arnes
    if not linea.startswith('#'):
        if b is not None:
            tramos.append(b)
        b = None
        continue
    t = norma(linea.lstrip('#'))
    if b is None:
        b = t
    elif t:                              # una linea `#` vacia, o de blancos, no anade nada
        b = b + t if b == '' or b.endswith('-') else b + ' ' + t
if b is not None:
    tramos.append(b)
pat = re.compile(r'zk-ssl-verify`? ?del ?(?:arbol|\u00e1rbol) ?en ?`?([0-9a-f]{7,}\w*)')
hexes = sorted({h for b in tramos for h in pat.findall(b)})
sys.stdout.buffer.write(''.join(h + '\n' for h in hexes).encode('utf-8'))
PY
}

# §914 (punto 10): las ENTRADAS de un manifiesto que juzga el kit $2, por TRAMOS: cada entrada es
# del ultimo kit que nombra un comentario por encima de ella («zk-ssl-verify del arbol en <hex>»,
# leido como en `nombrados_de`); si el manifiesto nombra un solo kit, todas son suyas. Una entrada
# sin kit por encima, en un manifiesto que nombra dos, es ROJO (exit 3): ningun kit la juzgaria.
# Imprime las lineas de las entradas, tal cual.
tramo_de() {
  python3 -I - "$1" "$2" <<'PY'
import re, sys, unicodedata
def norma(s):
    s = ''.join(c for c in s if unicodedata.category(c) != 'Cf')
    s = unicodedata.normalize('NFKC', s)
    return re.sub(r'\s+', ' ', s).strip().lower()
pat = re.compile(r'zk-ssl-verify`? ?del ?(?:arbol|\u00e1rbol) ?en ?`?([0-9a-f]{7,}\w*)')
crudo = open(sys.argv[1], 'rb').read()
try:
    texto = crudo.decode('utf-8')
except UnicodeDecodeError:
    texto = crudo.decode('latin-1')
sha = sys.argv[2]
lineas = texto.split('\n')
if lineas and lineas[-1] == '':
    lineas.pop()
entradas, b, actual, nombrados = [], None, None, set()
def cierra():
    global b, actual
    if b is not None:
        hs = pat.findall(b)
        if hs:
            actual = hs[-1]; nombrados.update(hs)
    b = None
for linea in lineas:
    if linea.startswith('#'):
        t = norma(linea.lstrip('#'))
        if b is None:
            b = t
        elif t:
            b = b + t if b == '' or b.endswith('-') else b + ' ' + t
        continue
    cierra()
    if linea.split('|', 1)[0] == '':
        continue
    entradas.append((actual, linea))
cierra()
if len(nombrados) <= 1:
    for _, l in entradas:
        print(l)
    sys.exit(0)
for h, l in entradas:
    if h is None:
        sys.stderr.write('entrada sin kit por encima: %s\n' % l.split('|', 1)[0]); sys.exit(3)
    if sha.startswith(h):
        print(l)
PY
}

# ── 1 · la lista, leida del arbol, contra la tabla ──────────────
msg "== CONSERVADOS · los catalogos de spec/vectors/<version>/ contra la tabla =="
EN_ARBOL=""
for d in spec/vectors/[0-9]*.[0-9]*/; do
  [ -d "$d" ] || continue
  for f in "$d"*; do
    [ -e "$f" ] || { falla "${d#spec/vectors/}: un directorio de version vacio"; continue; }
    if [ -d "$f" ]; then
      EN_ARBOL+="${f#spec/vectors/}"$'\n'
    else
      falla "${f#spec/vectors/}: fuera de la forma <version>/<familia>/, ningun kit lo juzga"
    fi
  done
done
EN_ARBOL=$(LC_ALL=C sort -u <<< "${EN_ARBOL%$'\n'}")
EN_TABLA=$(awk 'NF && $1 !~ /^#/ {print $1}' <<< "$TABLA" | LC_ALL=C sort -u)
n_a=$(grep -c . <<< "$EN_ARBOL"); n_t=$(grep -c . <<< "$EN_TABLA")
msg "  conservados en el arbol: $n_a · filas en la tabla: $n_t"
# ⚠️ Here-string, NO tuberia: el punto 6 de la cabecera del canon.
for c in $EN_ARBOL; do
  grep -qxF "$c" <<< "$EN_TABLA" || falla "spec/vectors/$c esta conservado y NO tiene fila en tools/conservados.sh"
done
for c in $EN_TABLA; do
  grep -qxF "$c" <<< "$EN_ARBOL" || falla "la fila '$c' no corresponde a ningun spec/vectors/<version>/<familia>/"
done
[ "$n_t" -gt 0 ] || falla "la tabla no tiene ninguna fila: un nivel que no mira nada no es VERDE"
# §914 (punto 10): un catalogo puede tener una fila por KIT -un tramo cada una-; lo que no se repite
# es el par catalogo-kit.
[ "$(awk 'NF && $1 !~ /^#/ {print $1 " " $2}' <<< "$TABLA" | LC_ALL=C sort | uniq -d | wc -l)" = "0" ] \
  || falla "la tabla repite un catalogo con el mismo kit"

# ── 2 · cada fila: el commit, su cabecera y su pin ──────────────
declare -A SHA_DE=() PIN_DE=() DIR_DE=() FILAS_DE=() NOMBRA_DE=()
CATS=()     # los catalogos con fila y manifiesto, en el orden de la tabla: «catalogo@kit» (punto 10)
KITS=()     # los commits, cada uno una vez, en el orden de la tabla
declare -A KIT_OK=() CATS_DE=()
SUPERFICIAL=$(git rev-parse --is-shallow-repository 2>/dev/null || echo "?")
while read -r dir sha pin resto; do
  [ -n "$dir" ] || continue
  case "$dir" in '#'*) continue ;; esac
  man="spec/vectors/$dir/MANIFIESTO.txt"
  if ! [[ "$sha" =~ ^[0-9a-f]{40}$ ]]; then falla "$dir: el commit de la tabla no son 40 hex: '$sha'"; continue; fi
  if ! [[ "$pin" =~ ^[0-9]+$ ]]; then falla "$dir: el pin de entradas no es un numero: '$pin'"; continue; fi
  [ -d "spec/vectors/$dir" ] || continue      # ya es ROJO arriba, con su nombre
  if [ ! -f "$man" ]; then falla "$dir: conservado sin MANIFIESTO.txt"; continue; fi
  key="$dir@${sha:0:12}"
  FILAS_DE[$dir]="${FILAS_DE[$dir]:-}$sha "
  # el commit lo dicen dos (punto 2): la fila, y una mencion de su cabecera que sea su prefijo; con
  # varios kits (punto 10), cada mencion tiene que ser de alguna fila, y se mira tras el bucle
  if [ -z "${NOMBRA_DE[$dir]+x}" ]; then
    if ! NOMBRA_DE[$dir]=$(nombrados_de "$man"); then
      falla "$dir: no se pudo leer la cabecera de su manifiesto (python3 fallo)"; NOMBRA_DE[$dir]=""
    elif [ -z "${NOMBRA_DE[$dir]}" ]; then
      falla "$dir: la cabecera de su manifiesto no nombra el kit ('zk-ssl-verify del arbol en <commit>')"
    fi
  fi
  nombrado=0
  for nm in ${NOMBRA_DE[$dir]}; do [[ "$sha" == "$nm"* ]] && nombrado=1; done
  [ -z "${NOMBRA_DE[$dir]}" ] || [ $nombrado = 1 ] \
    || falla "$dir: la tabla pina el kit ${sha:0:12} y la cabecera no lo nombra (nombra: $(tr '\n' ' ' <<< "${NOMBRA_DE[$dir]}"))  ->  MEDIR, la cabecera no se reescribe"
  # el pin, contra el manifiesto (punto 7): las entradas del tramo de este kit (punto 10)
  if ! TRAMO=$(tramo_de "$man" "$sha"); then
    falla "$dir: el tramo del kit ${sha:0:12} no se pudo leer (una entrada sin kit por encima, o python3 fallo)"; TRAMO=""
  fi
  ne=$(grep -c . <<< "$TRAMO")
  [ "$ne" = "$pin" ] || falla "$dir: el tramo del kit ${sha:0:12} tiene $ne entradas y la tabla pina $pin  ->  MEDIR y editar tools/conservados.sh, fila '$dir'"
  # el commit es del arbol (punto 3), con su crate y su arnes
  if [ -z "${KIT_OK[$sha]+x}" ]; then
    KITS+=("$sha"); KIT_OK[$sha]=1; CATS_DE[$sha]=""
    if ! git cat-file -e "$sha^{commit}" 2>/dev/null; then
      if [ "$SUPERFICIAL" = "true" ]; then
        falla "kit ${sha:0:12}: el commit no esta en este clon, que es SUPERFICIAL  ->  git fetch --unshallow"
      else
        falla "kit ${sha:0:12}: el commit no esta en este clon  ->  git fetch origin $sha"
      fi
      KIT_OK[$sha]=0
    else
      git merge-base --is-ancestor "$sha" HEAD 2>/dev/null \
        || { falla "kit ${sha:0:12}: no es antepasado de HEAD, y la cabecera dice 'del arbol'"; KIT_OK[$sha]=0; }
      git cat-file -e "$sha:crates/zk-ssl-verify/Cargo.toml" 2>/dev/null \
        || { falla "kit ${sha:0:12}: en su commit no hay crates/zk-ssl-verify"; KIT_OK[$sha]=0; }
      git cat-file -e "$sha:tools/conformidad.sh" 2>/dev/null \
        || { falla "kit ${sha:0:12}: en su commit no hay tools/conformidad.sh, el arnes que su cabecera afirma"; KIT_OK[$sha]=0; }
    fi
  fi
  CATS+=("$key"); SHA_DE[$key]=$sha; PIN_DE[$key]=$pin; DIR_DE[$key]=$dir
  CATS_DE[$sha]+="$key "
done <<< "$TABLA"
# punto 10: toda mencion de una cabecera es de alguna fila de su catalogo
for dir in "${!NOMBRA_DE[@]}"; do
  for nm in ${NOMBRA_DE[$dir]}; do
    hay=0
    for sha in ${FILAS_DE[$dir]}; do [[ "$sha" == "$nm"* ]] && hay=1; done
    [ $hay = 1 ] || falla "$dir: la cabecera nombra $nm y ninguna fila de la tabla lo lleva  ->  MEDIR, la cabecera no se reescribe"
  done
done
[ "${#CATS[@]}" -gt 0 ] || falla "ningun catalogo que juzgar"
msg "  ${#CATS[@]} catalogo(s), ${#KITS[@]} kit(s); rustc de la maquina: $(rustc -V 2>/dev/null || echo 'NINGUNO')"
[ $rojo -eq 0 ] && msg "  OK  cada conservado tiene fila, cada fila su directorio, su cabecera la nombra y su pin cuadra"

# ── 3 · el plan, o el plan y su ejecucion ───────────────────────
if [ $EN_SECO = 1 ]; then
  msg ""
  msg "== CONSERVADOS · EN SECO: lo que haria, sin construir ni escribir nada =="
  for sha in "${KITS[@]}"; do
    c=${sha:0:12}; tgt="$CONS_DIR/$c/target"
    if [ "${KIT_OK[$sha]}" != "1" ]; then msg "   kit $c: NO se construiria (ROJO arriba)"; continue; fi
    msg "   kit $c · $(git log -1 --format='%ad' --date=short "$sha") · $(git log -1 --format='%s' "$sha" | cut -c1-70)"
    msg "      T=$tgt"
    if [ -d "$tgt" ]; then
      msg "      \$T existe: CALIENTE ($(du -sh "$tgt" 2>/dev/null | cut -f1))$([ $FRIO = 1 ] && echo '; --frio lo borraria antes')"
    else
      msg "      \$T no existe: FRIO"
    fi
    a_c=$(git show "$sha:tools/conformidad.sh" | sha256sum | cut -c1-16); a_a=$(h16 tools/conformidad.sh)
    msg "      arnes de su commit $a_c · arnes del arbol $a_a$([ "$a_c" = "$a_a" ] && echo ' (el MISMO)')"
    msg "      \$ git worktree add --detach $TMPB/conservado-$c.XXXXXX/arbol $sha"
    msg "      \$ (cd <arbol> && CARGO_TARGET_DIR=\$T timeout ${TOPE}s cargo build --release --locked -p zk-ssl-verify)"
    for key in ${CATS_DE[$sha]}; do
      dir=${DIR_DE[$key]}; tr_=""
      [ "$(wc -w <<< "${FILAS_DE[$dir]}")" -gt 1 ] && tr_=" (su tramo, en un directorio temporal)"
      msg "      \$ bash <arbol>/tools/conformidad.sh \$T/release/zk-ssl-verify spec/vectors/$dir/MANIFIESTO.txt   # su commit, ${PIN_DE[$key]} entradas$tr_"
      msg "      \$ bash tools/conformidad.sh \$T/release/zk-ssl-verify spec/vectors/$dir/MANIFIESTO.txt          # el arbol"
    done
    msg "      \$ git worktree remove --force <arbol>"
  done
  msg ""
  if [ $rojo -eq 0 ]; then
    msg "== CONSERVADOS --en-seco: el plan es VERDE · ${#CATS[@]} catalogos, ${#KITS[@]} kits; nada construido =="
  else
    msg "== CONSERVADOS --en-seco: ROJO · ${#fallos[@]} fallo(s) =="
    for f in "${fallos[@]}"; do msg "   · $f"; done
  fi
  exit $rojo
fi

ANTES=$(git status --porcelain -uall 2>/dev/null)
TMPS=()   # los directorios temporales de los worktrees, para el trap

# ⚠️ Here-string: con `pipefail`, `git ... | grep -q` que acierta da 141 y el ROJO no saltaria.
listado() { grep -qxF "worktree $1" <<< "$(git worktree list --porcelain 2>/dev/null)"; }

# Quita el worktree $1 (punto 9); su salida, a $2. Sale 1 si `remove --force` fallo, y aun asi
# limpia: uno BLOQUEADO -como deja el suyo un `git worktree add` cortado a medias- no lo quita
# `remove --force` ni lo poda `prune`, y el directorio es nuestro (mktemp): se desbloquea, se
# borra y se poda, en ese orden (`prune` no poda un worktree cuyo directorio sigue ahi). ⚠️ El
# `prune` va sin `--expire`: poda en el acto tambien las entradas huerfanas ajenas (punto 9).
quitar_wt() {
  local wt=$1 log=${2:-/dev/null}
  [ -e "$wt" ] || listado "$wt" || return 0
  git worktree remove --force "$wt" >> "$log" 2>&1 && return 0
  git worktree unlock "$wt" >> "$log" 2>&1
  rm -rf "$wt"
  git worktree prune >> "$log" 2>&1
  return 1
}
# ⚠️ Solo en el nivel, no en sus hijos: un subshell que recibe un TERM antes de reponer sus
# senales -el de la construccion, si `parar` llega justo tras el `&`- corre el trap de EXIT que
# heredo, y quitaba el worktree a la vez que el nivel (medido en el §798 con bash 5.2, con un `echo`
# en `limpiar`: dos PID). `$$` es el del nivel tambien en sus subshells; `$BASHPID`, no.
limpiar() {
  [ "$BASHPID" = "$$" ] || return 0
  local p
  for p in "${TMPS[@]}"; do
    quitar_wt "$p/arbol"
    rm -rf "$p"
  done
  TMPS=()
}
# ⚠️ La construccion corre en SEGUNDO PLANO y se espera con `wait`, y no por capricho: `timeout`
# pone a cargo en su propio grupo de procesos, asi que ni el Ctrl-C de la terminal ni un TERM al
# grupo del nivel le llegan, y bash no corre el trap de una senal hasta que acaba el hijo que
# espera en primer plano. Medido en el §798 con un cargo falso dormido: el TERM llego, el worktree
# siguio en `git worktree list` hasta que el hijo acabo solo. El `wait` de bash, en cambio, vuelve
# en cuanto llega una senal con trap; `parar` se la pasa a `timeout`, que se la pasa a cargo, y el
# trap de EXIT quita el worktree.
# ⚠️ Entre el `&` y el `HIJO=$!` hay una ventana: una senal ahi encontraria HIJO vacio, `parar` no
# mataria a nadie y la construccion seguiria, huerfana. En esa ventana la senal solo se ANOTA
# (`anotar`), y se atiende en cuanto HIJO tiene el PID (`atender`). No con `${HIJO:-$!}`: aqui `$!`
# puede ser el PID de un hijo ya recogido -la construccion del kit anterior, o un `< <(...)` de
# `juzgar`, que en este bash 5.2 tambien lo fija (medido)-, y matarlo seria matar a otro. El hijo
# que muere asi, antes de su `exec` y de sus redirecciones, puede dejar en la salida del nivel un
# aviso de su bash, «wait_for: No record of process <n>»: es ruido (medido en el §798, tres de tres,
# sin huerfano y limpio; con la senal medio segundo despues, ni aviso).
HIJO=""; PEND=""
parar() { if [ -n "$HIJO" ]; then kill -TERM "$HIJO" 2>/dev/null; wait "$HIJO" 2>/dev/null; HIJO=""; fi; }
atender() { trap 'parar; exit 130' INT; trap 'parar; exit 143' TERM; }   # 128 + 2 y 128 + 15
anotar() { trap 'PEND=130' INT; trap 'PEND=143' TERM; }
trap limpiar EXIT
atender

# $1 catalogo@kit, $2 binario, $3 arnes, $4 etiqueta -> deja en RES «OK k/n» o «ROJO k/n»; cada ROJO
# del arnes entra por falla con su nombre, como en el 3 bis del canon. §914 (punto 10): si el
# catalogo tiene varios kits, el arnes corre sobre el TRAMO de este -sus entradas y sus vectores,
# copiados a un directorio temporal bajo el del kit-, porque el arnes exige que todo .json del
# directorio tenga entrada y juzga el manifiesto entero; con un solo kit, sobre el manifiesto tal cual.
juzgar() {
  local key=$1 bin=$2 arn=$3 et=$4 dir=${1%@*} sha12=${1#*@} man f rc ok n td
  man="spec/vectors/$dir/MANIFIESTO.txt"
  f="$LOGS/${dir//\//_}.$sha12.$et.txt"
  if [ "$(wc -w <<< "${FILAS_DE[$dir]}")" -gt 1 ]; then
    td="$p/tramo-$sha12-${dir//\//_}"
    rm -rf "$td"; mkdir -p "$td"
    { echo "# tramo del kit $sha12 de spec/vectors/$dir/MANIFIESTO.txt (tools/conservados.sh, punto 10)";
      tramo_de "$man" "${SHA_DE[$key]}"; } > "$td/MANIFIESTO.txt" || { RES="ROJO ?/?"; falla "$dir: el tramo de $sha12 no se pudo escribir"; return 1; }
    while IFS='|' read -r VF R; do
      case "$VF" in ''|'#'*) continue ;; esac
      [ -f "spec/vectors/$dir/$VF" ] && cp "spec/vectors/$dir/$VF" "$td/$VF"
    done < "$td/MANIFIESTO.txt"
    man="$td/MANIFIESTO.txt"
  fi
  timeout 600 bash "$arn" "$bin" "$man" > "$f" 2>&1; rc=$?
  read -r ok n < <(sed -n 's/^conformidad: \([0-9]*\) de \([0-9]*\) entradas.*/\1 \2/p' "$f" | tail -n 1)
  ok=${ok:-?}; n=${n:-?}
  if [ "$rc" = "0" ] && [ "$ok" = "$n" ] && [ "$n" = "${PIN_DE[$key]}" ]; then RES="OK $ok/$n"; return 0; fi
  RES="ROJO $ok/$n"
  while IFS= read -r L; do falla "$dir ($sha12), arnes del $et: $L"; done < <(grep '^ROJO' "$f" | sed 's/^ROJO //')
  if [ "$n" != "${PIN_DE[$key]}" ]; then
    falla "$dir ($sha12), arnes del $et: juzgo $n entradas y la tabla pina ${PIN_DE[$key]} (exit $rc)  ->  $f"
  elif ! grep -q '^ROJO' "$f"; then
    falla "$dir ($sha12), arnes del $et: exit $rc sin nombrar la entrada ($(tail -n 1 "$f"))"
  fi
  return 1
}

msg ""
msg "== CONSERVADOS · cada kit, construido de su commit, juzga sus catalogos =="
T_INI=$(date +%s)
FILAS=()
FMT='%-18s %-8s %-14s %-18s %s'
for sha in "${KITS[@]}"; do
  c=${sha:0:12}; tgt="$CONS_DIR/$c/target"; log="$LOGS/kit-$c.log"
  if [ "${KIT_OK[$sha]}" != "1" ]; then
    for key in ${CATS_DE[$sha]}; do FILAS+=("$(printf "$FMT" "${DIR_DE[$key]}" "${sha:0:7}" '-' 'SIN KIT' 'SIN KIT')")
      falla "${DIR_DE[$key]}: sin juzgar, su kit ${sha:0:12} no se construye (arriba)"; done
    continue
  fi
  [ $FRIO = 1 ] && rm -rf "$tgt"
  mkdir -p "$tgt" "$LOGS"; LOGS_USADO=1
  p=$(mktemp -d "$TMPB/conservado-$c.XXXXXX") && p=$(cd "$p" && pwd -P) \
    || { falla "kit $c: no se pudo crear el directorio temporal en $TMPB"; continue; }
  TMPS+=("$p"); wt="$p/arbol"
  t0=$(date +%s)
  CONSTRUIDO=0
  if ! git worktree add --detach "$wt" "$sha" > "$log" 2>&1; then
    falla "kit $c: git worktree add fallo  ->  $log"
  elif [ "$(git -C "$wt" rev-parse HEAD 2>/dev/null)" != "$sha" ]; then
    falla "kit $c: el worktree no esta en su commit"
  else
    RV=$(cd "$wt" && rustc -V 2>/dev/null)
    echo "== rustc del kit: $RV" >> "$log"
    anotar
    ( cd "$wt" && CARGO_TARGET_DIR="$tgt" exec timeout "${TOPE}s" cargo build --release --locked -p zk-ssl-verify ) >> "$log" 2>&1 &
    HIJO=$!
    atender
    [ -z "$PEND" ] || { parar; exit "$PEND"; }
    wait "$HIJO"; rc=$?
    HIJO=""
    if [ "$rc" = "124" ]; then
      falla "kit $c: pasados $TOPE s sin acabar de construir (timeout)  ->  $log"
    elif [ "$rc" != "0" ]; then
      falla "kit $c: no construye de su commit con --locked (exit $rc)  ->  $log"
    elif [ ! -x "$tgt/release/zk-ssl-verify" ]; then
      falla "kit $c: construye y no deja $tgt/release/zk-ssl-verify"
    else
      CONSTRUIDO=1
    fi
    SUCIO=$(git -C "$wt" status --porcelain -uall 2>/dev/null)
    [ -z "$SUCIO" ] || falla "kit $c: construir ensucio el arbol de su commit: $(head -3 <<< "$SUCIO" | tr '\n' ' ')"
  fi
  T_K=$(( $(date +%s) - t0 ))
  if [ $CONSTRUIDO = 1 ]; then
    B="$tgt/release/zk-ssl-verify"
    msg "   kit $c: binario $(h16 "$B") · ${RV:-rustc ?} · $T_K s"
    for key in ${CATS_DE[$sha]}; do
      dir=${DIR_DE[$key]}
      juzgar "$key" "$B" "$wt/tools/conformidad.sh" commit; R1=$RES
      juzgar "$key" "$B" "tools/conformidad.sh" arbol; R2=$RES
      FILAS+=("$(printf "$FMT" "$dir" "${sha:0:7}" "$(tramo_de "spec/vectors/$dir/MANIFIESTO.txt" "$sha" | grep -c .)(${PIN_DE[$key]})" "$R1" "$R2")")
    done
  else
    for key in ${CATS_DE[$sha]}; do FILAS+=("$(printf "$FMT" "${DIR_DE[$key]}" "${sha:0:7}" '-' 'SIN KIT' 'SIN KIT')")
      falla "${DIR_DE[$key]}: sin juzgar, su kit $c no se construyo"; done
  fi
  # el worktree se va en cuanto sus catalogos estan juzgados (punto 9), y se comprueba
  quitar_wt "$wt" "$log" || falla "kit $c: git worktree remove --force fallo (¿bloqueado?);\
 se desbloqueo, se borro y se podo  ->  $log"
  listado "$wt" && falla "kit $c: git sigue listando su worktree $wt\
  ->  git worktree unlock $wt; git worktree prune"
  rm -rf "$p"
done
msg ""
msg "   $(printf "$FMT" catalogo kit 'entradas(pin)' 'arnes del commit' 'arnes del arbol')"
for L in "${FILAS[@]}"; do msg "   $L"; done
msg "   ── total: $(( $(date +%s) - T_INI )) s"

DESPUES=$(git status --porcelain -uall 2>/dev/null)
[ "$ANTES" = "$DESPUES" ] || falla "el nivel dejo el arbol distinto de como lo encontro (git status)"

msg ""
if [ $rojo -eq 0 ]; then
  mkdir -p "$(dirname "$CONS_FILE")"
  printf '%s %s %s %s\n' "$(git rev-parse --short HEAD)" "$(date -Iseconds)" "${#CATS[@]}" \
    "$(rustc -V 2>/dev/null | awk '{print $2}')" > "$CONS_FILE"
  msg "== CONSERVADOS: VERDE · ${#CATS[@]} catalogos con ${#KITS[@]} kits, cada uno el de su commit, y los dos arneses =="
  msg "   anotado en $CONS_FILE — para que nadie tenga que acordarse."
else
  msg "== CONSERVADOS: ROJO · ${#fallos[@]} fallo(s) =="
  for f in "${fallos[@]}"; do msg "   · $f"; done
fi
[ $LOGS_USADO = 1 ] && msg "   salida integra en $LOGS/"
exit $rojo
