#!/usr/bin/env bash
# tools/banco_edad.sh -- el banco de E4b-3 del RFC-0007 (la prueba de edad, de punta a punta).
#
# Demuestra la PRUEBA DE EDAD como servicio: un libro con pendientes VIVOS -puestos por el
# sandbox del cli, con pruebas STARK reales- -> nodo real que lo abre y FIRMA una cabeza v5 ->
# se custodia esa cabeza -> el nodo MUERE -> la CAPA produce la prueba sobre ese libro, contra
# esa cabeza -> el verificador en VERDE **sin el nodo**. Y NUEVE negativos sobre el sobre.
#
# ⚠️ Por que el cli pone los pendientes y no el cable: ningun metodo publica un pendiente sin
# una prueba de CLIENTE, y no hay cliente que hable con el nodo (medido: cero usos de
# `zkssl_sendMaterials` y `zkssl_applySend` en `tools/`). `simulate --ledger --no-claim` deja el
# envio EN VUELO sobre un libro persistido, y sus parametros por defecto son los del nodo -si no
# lo fueran, `SovereignLayer::open` daria `ParameterMismatch` y este banco moriria ahi-.
#
# ⚠️ El nodo se para ANTES de producir: `sled` abre el libro en EXCLUSIVA, asi que el modo
# `--prueba-edad` no puede correr con el servidor vivo. Y el libro no puede salir de la maquina:
# `export_snapshot` REHUSA con pagos en vuelo (§359), que es justo lo que la prueba necesita.
#
# FUERA del canon: levanta procesos y espera latidos. Hermano de `tools/banco_consumo.sh`, del
# que copia su forma; compila en RELEASE porque aqui se firma y se prueba de verdad.
#
# NO ESCRIBE EN EL ARBOL: todo lo suyo vive en un temporal bajo $HOME, que borra al salir. Lo
# comprueba al final por `git status --porcelain`.
#
#   bash tools/banco_edad.sh [--guardar <dir>]        (cuando ya vive en el arbol)
#   cd ~/zk-ssl-real && bash <ruta-suelta> [--guardar <dir>]   (mientras vive en Downloads)
#
# --guardar  copia los DOS sobres positivos y los nueve cuerpos negativos a <dir>, con su huella.
#            De esas capturas se derivan por MUTACION los vectores del catalogo (regla 2 del
#            PROCESO: los vectores jamas se reescriben, se derivan de capturas reales).
#
# ⚠️ Los fragmentos de rechazo de abajo estan LEIDOS del fuente del mando y del juez, salvo el de
# la cota, que lo pone winterfell y no se predice: por eso `niega` exige el fragmento Y dice
# <<cayo, pero NO por su regla>> cuando el rojo es de otro. El instrumento mide; no adivina.
set -euo pipefail
msg(){ printf 'BANCO-EDAD| %s\n' "$*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }

# ⚠️ LA RAIZ SE DERIVA, no se supone (copiado del hermano): este fichero puede correrse todavia
# SUELTO desde Downloads, y `dirname $0/..` llevaria a un directorio sin `Cargo.toml`. Las dos
# procedencias, en orden, y la elegida trae su PRUEBA DE VIDA.
RAIZ=""
for cand in "$(cd "$(dirname "$0")/.." 2>/dev/null && pwd)" \
            "$(git rev-parse --show-toplevel 2>/dev/null)" ; do
  [ -n "$cand" ] || continue
  if [ -f "$cand/Cargo.toml" ] && [ -d "$cand/crates/zk-ssl-verify" ]; then RAIZ="$cand"; break; fi
done
[ -n "$RAIZ" ] || fallo "no se localizo la raiz del arbol: ni junto a este fichero ni en el repo del directorio actual hay un Cargo.toml con crates/zk-ssl-verify. Correr desde ~/zk-ssl-real"
cd "$RAIZ"
msg "raiz DERIVADA: $RAIZ"
command -v curl    >/dev/null 2>&1 || fallo "curl no esta en el PATH"
command -v python3 >/dev/null 2>&1 || fallo "python3 no esta en el PATH"

GUARDAR=""
case "${1:-}" in
  --guardar) GUARDAR="${2:?--guardar exige un directorio}"; mkdir -p "$GUARDAR" ;;
  "") : ;;
  *) fallo "uso: bash tools/banco_edad.sh [--guardar <dir>]" ;;
esac

PORT=8607   # libre: censados en tools/*.sh estan 8593, 8594, 8597, 8598, 8599, 8600, 8601,
            # 8603, 8605 y 8613 (los dos del banco de dos libros van por PORT_A y PORT_B)
DIR=$(mktemp -d "$HOME/.banco_edad.XXXXXX")
PID=""
limpiar(){
  if [ -n "$PID" ]; then kill -9 "$PID" 2>/dev/null || true; fi
  rm -rf "$DIR"
}
trap limpiar EXIT INT TERM HUP QUIT

# ⚠️ LA BASE DE LA PUREZA, tomada ANTES de nada. El invariante de un banco no es <<el arbol esta
# limpio>> sino <<yo no lo ensucie>> (PRECISION 169), y este banco corre tambien DENTRO del
# bloque que sella su propio corte: alli el arbol ya lleva los ficheros del sello, y un gate
# absoluto cobraria deuda AJENA. Se mide por DELTA.
git status --porcelain | sort > "$DIR/porcelain.base"

msg "compilando cli, nodo y verificador en RELEASE (aqui se firma y se prueba de verdad)"
cargo build --release -q -p zk-ssl-cli -p zk-ssl-node -p zk-ssl-verify 2>/dev/null \
  || cargo build --release -p zk-ssl-cli -p zk-ssl-node -p zk-ssl-verify || fallo "no compila"
CLI=target/release/zk-ssl-cli
NODO=target/release/zk-ssl-node
VER=target/release/zk-ssl-verify
python3 -c "print('37'*96, end='')" > "$DIR/semilla.hex"
chmod 600 "$DIR/semilla.hex"

rpc(){
  curl -s --max-time 20 "http://127.0.0.1:$PORT" \
    -H 'Content-Type: application/json' \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$1\",\"params\":$2}"
}
campo(){
  python3 - "$1" "$2" <<'PY'
import json, sys
d = json.loads(sys.argv[1])
for k in sys.argv[2].split('.'):
    d = d[int(k)] if k.isdigit() else d[k]
print(json.dumps(d))
PY
}
qnum(){ python3 -c 'import json,sys; print(int(json.loads(sys.argv[1]), 16))' "$1"; }

# ---------------------------------------------------------------- EL LIBRO CON PENDIENTES VIVOS
# Dos corridas del sandbox con semillas de sal DISTINTAS: dos posiciones vivas, nacidas a dos
# alturas del registro distintas, que es lo que hace que la edad discrimine.
# ⚠️ `--accounts` SUBE en cada corrida, y esto esta LEIDO del fuente (`commands.rs`, el montaje
# de `simulate`), no supuesto: el bucle abre cuentas solo mientras
# `account_count() < max(--accounts, max(from,to) + 1)`, y `--from/--to` son posiciones LOGICAS
# sobre las abiertas EN ESA CORRIDA. Sobre un libro que ya tiene dos cuentas, un `--accounts 2`
# no abre ninguna, la lista logica queda vacia y la #0 no resuelve -el arbol es disperso y las
# posiciones se derivan de la identidad (F3), asi que tampoco son 0 y 1-. Con el censo por
# delante, cada corrida abre SU pareja. La semilla de claves va aparte para que las identidades
# no colisionen.
I=0
for S in 7 8; do
  I=$(( I + 1 ))
  K=$(( 0xA11CE + S ))
  A=$(( 2 * I ))
  msg "sandbox: un envio EN VUELO sobre el libro (salt-seed $S, key-seed $K, accounts $A)"
  "$CLI" --log warn simulate --ledger "$DIR/ledger" --no-claim \
    --salt-seed "$S" --key-seed "$K" --accounts "$A" \
    >"$DIR/sim-$S.txt" 2>&1 || { sed 's/^/BANCO-EDAD|   /' "$DIR/sim-$S.txt" >&2; fallo "el sandbox no dejo el envio en vuelo (salt-seed $S, key-seed $K, accounts $A)"; }
done

# ---------------------------------------------------------------- LA CABEZA v5 FIRMADA
"$NODO" --listen "127.0.0.1:$PORT" --latido 2 \
  --clave-fichero "$DIR/semilla.hex" --custodia fichero \
  --diario "$DIR/diario.jsonl" --ledger "$DIR/ledger" \
  --contador-recepcion "$DIR/recepcion.bin" \
  --indice-firma "$DIR/indice-firma.bin" \
  --log warn 2>>"$DIR/nodo.err" &
PID=$!

CAB=""
for _ in $(seq 1 60); do
  V=$(rpc zkssl_signedEpochHead '{}' 2>/dev/null || true)
  case "$V" in *'"available":true'*) CAB="$V"; break;; esac
  sleep 0.5
done
[ -n "$CAB" ] || { sed 's/^/BANCO-EDAD|   /' "$DIR/nodo.err" >&2; fallo "no llego una cabeza firmada"; }
FV=$(campo "$CAB" result.formatVersion)
SEQ=$(campo "$CAB" result.seq)
NP=$(campo "$CAB" result.nextPending)
[ "$(qnum "$FV")" = "5" ] \
  || fallo "la cabeza dice formatVersion $(qnum "$FV"): la prueba de edad exige v5, la unica que firma pmetaRoot y nextPending"
[ "$(qnum "$NP")" -ge 2 ] \
  || fallo "la cabeza firma nextPending $(qnum "$NP"): el sandbox no dejo las dos posiciones vivas, y sin ellas el positivo no discrimina"
campo "$CAB" result > "$DIR/cabeza.json"
msg "cabeza v5 custodiada: seq $(qnum "$SEQ") - nextPending $(qnum "$NP")"

kill -9 "$PID"; wait "$PID" 2>/dev/null || true
PID=""
msg "el nodo esta MUERTO; el libro queda libre y a partir de aqui NADA de lo que sigue lo toca"

# ---------------------------------------------------------------- LA CAPA PRODUCE
# Dos enunciados, y los dos son formas que el RFC nombra: TODOS (`T = 0`, la cuenta entera) y la
# CAJA VACIA (`T` por encima de la altura: nada tan viejo sigue en vuelo, luego `k = 0`).
produce(){ # produce <fichero> <T> <rotulo>
  local f="$1" t="$2" rot="$3" s r
  set +e
  s=$("$NODO" --ledger "$DIR/ledger" --log warn \
        --prueba-edad "$f" --edad-cabeza "$DIR/cabeza.json" --edad-t "$t" 2>&1)
  r=$?
  set -e
  [ "$r" = "0" ] || fallo "la capa no produjo el sobre $rot (exit $r): $s"
  echo "$s" | sed 's/^/BANCO-EDAD|   /' >&2
}
produce "$DIR/edad-todos.json"      0       todos
produce "$DIR/edad-caja-vacia.json" 1000000 caja-vacia

for f in edad-todos edad-caja-vacia; do
  set +e; SAL=$("$VER" "$DIR/$f.json" 2>&1); RC=$?; set -e
  [ "$RC" = "0" ] || fallo "el sobre POSITIVO $f dio exit $RC (se esperaba 0): $SAL"
  echo "$SAL" | sed 's/^/BANCO-EDAD|   /' >&2
done
K=$(python3 -c 'import json,sys; print(int(json.load(open(sys.argv[1]))["enunciado"]["k"],16))' "$DIR/edad-todos.json")
KV=$(python3 -c 'import json,sys; print(int(json.load(open(sys.argv[1]))["enunciado"]["k"],16))' "$DIR/edad-caja-vacia.json")
[ "$KV" = "0" ] || fallo "la CAJA VACIA no salio vacia: k = $KV con T por encima de la altura"
[ "$K" -ge 2 ] || fallo "el positivo de TODOS cuenta $K posiciones vivas y el libro tiene $(qnum "$NP"): la cota no esta contando"
msg "POSITIVOS: exit 0 los dos -- k = $K con T = 0, y k = 0 con la caja vacia, sin el nodo"

# ---------------------------------------------------------------- LOS NUEVE NEGATIVOS
ROTOS=0
niega(){ # niega <fichero> <fragmento esperado> <rotulo>
  local f="$1" frag="$2" rot="$3" s r
  set +e; s=$("$VER" "$f" 2>&1); r=$?; set -e
  [ "$r" = "1" ] || fallo "NEGATIVO $rot dio exit $r (se esperaba 1): $s"
  case "$s" in
    *"$frag"*) msg "NEGATIVO $rot: exit 1 -- $(echo "$s" | tail -n 1)"; ROTOS=$((ROTOS+1)) ;;
    *) fallo "NEGATIVO $rot cayo, pero NO por su regla: se esperaba <<$frag>> y dijo: $s" ;;
  esac
}

python3 - "$DIR" <<'PY'
import json, os, sys
d = sys.argv[1]
base = json.load(open(d + "/edad-todos.json"))
def esc(n, p): open("%s/%s.json" % (d, n), "w").write(json.dumps(p))
def cop(): return json.loads(json.dumps(base))

# Las cuatro AUSENCIAS: el mando las nombra una a una antes de tocar la firma.
for clave, nombre in (("enunciado", "neg-sin-enunciado"), ("subraices", "neg-sin-subraices"),
                      ("prueba", "neg-sin-prueba"), ("cabeza", "neg-sin-cabeza")):
    p = cop(); del p[clave]; esc(nombre, p)
# La firma que falta: la cabeza deja de ser verificable, y el mando lo dice por su nombre.
p = cop(); del p["cabeza"]["signature"]; esc("neg-sin-firma", p)
# La ERA: una v4 no firma pmetaRoot ni nextPending, y la version se juzga ANTES que la firma.
p = cop(); p["cabeza"]["formatVersion"] = "0x4"; esc("neg-cabeza-v4", p)
# Las dos SUBRAICES, una a una. El sustituto es la OTRA subraiz -no un nibble volteado-, que ya
# viaja en el sobre y por tanto es un digest CANONICO seguro: asi el rojo es el de la subida a la
# raiz y no el de la FORMA, que es otra regla (el precedente del hermano, con el consumo).
p = cop(); p["subraices"]["pendientes"] = base["subraices"]["meta"]; esc("neg-subraiz-pend", p)
p = cop(); p["subraices"]["meta"] = base["subraices"]["pendientes"]; esc("neg-subraiz-meta", p)
# La COTA movida: el enunciado deja de ser el que la prueba prueba. El texto lo pone winterfell y
# NO se predice: `niega` solo exige el prefijo que el mando antepone.
k = int(base["enunciado"]["k"], 16)
p = cop(); p["enunciado"]["k"] = hex(k - 1 if k > 0 else k + 1); esc("neg-cota-movida", p)

# LA PUERTA: un sabotaje que no cambia un byte no prueba nada. Se comprueba AQUI, antes de gastar
# una corrida del verificador, y se dice CUAL.
crudo = json.dumps(base)
iguales = [f for f in sorted(os.listdir(d))
           if f.startswith("neg-") and open(d + "/" + f).read() == crudo]
if iguales:
    print("ROJO: estos sabotajes NO cambian un byte del positivo: " + " ".join(iguales))
    raise SystemExit(3)
print("nueve cuerpos negativos derivados por MUTACION, y los nueve DIFIEREN del positivo")
PY

niega "$DIR/neg-sin-enunciado.json" "falta enunciado"                        "sin-enunciado"
niega "$DIR/neg-sin-subraices.json" "falta subraices"                        "sin-subraices"
niega "$DIR/neg-sin-prueba.json"    "falta prueba"                           "sin-prueba"
niega "$DIR/neg-sin-cabeza.json"    "falta cabeza"                           "sin-cabeza"
niega "$DIR/neg-sin-firma.json"     "falta signature"                        "sin-firma"
niega "$DIR/neg-cabeza-v4.json"     "exige una cabeza v5"                    "cabeza-v4"
niega "$DIR/neg-subraiz-pend.json"  "no es el pendingRoot de la cabeza"      "subraiz-pend"
niega "$DIR/neg-subraiz-meta.json"  "no es el pmetaRoot de la cabeza"        "subraiz-meta"
niega "$DIR/neg-cota-movida.json"   "edad:"                                  "cota-movida"

# ---------------------------------------------------------------- GUARDAR Y PUREZA
if [ -n "$GUARDAR" ]; then
  for f in "$DIR"/edad-*.json "$DIR"/neg-*.json "$DIR/cabeza.json"; do
    cp "$f" "$GUARDAR/"
    printf 'BANCO-EDAD|   %-30s %s  %s B\n' "$(basename "$f")" \
      "$(sha256sum "$f" | cut -c1-16)" "$(wc -c < "$f")" >&2
  done
  msg "capturas guardadas en $GUARDAR (de aqui salen los vectores, por MUTACION)"
fi

git status --porcelain | sort > "$DIR/porcelain.post"
SUCIO=$(comm -13 "$DIR/porcelain.base" "$DIR/porcelain.post" | wc -l)
if [ "$SUCIO" -ne 0 ]; then
  comm -13 "$DIR/porcelain.base" "$DIR/porcelain.post" | sed 's/^/BANCO-EDAD|   /' >&2
  fallo "el banco ensucio $SUCIO entradas del arbol: no debe tocarlo"
fi
[ "$ROTOS" = "9" ] || fallo "se esperaban 9 negativos y cayeron $ROTOS"

msg "BANCO-EDAD VERDE: la capa probo sobre un nodo real, el kit lo verifico SIN el nodo, y las nueve reglas del sobre cayeron EN VIVO"
