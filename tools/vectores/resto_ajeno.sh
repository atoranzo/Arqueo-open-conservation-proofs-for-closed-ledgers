#!/usr/bin/env bash
# §713: captura el sobre de edad cuya prueba se compromete con OTRO resto FRI, el vector
# `spec/vectors/edad/neg-prueba-resto-ajeno.json`.
#
# El camino de `tools/banco_edad.sh` -dos sandbox con pendientes vivos, un nodo real que firma su
# cabeza, el nodo muerto y la capa que produce con `--prueba-edad`- con un nodo compilado contra
# `winter-fri` 0.13.1 y el parche `resto_ajeno.diff` (a su lado): con `ARQUEO_RESTO_AJENO` en el
# entorno, `set_remainder` se compromete con el hash de OTRO resto -el mismo con su primer
# coeficiente mas uno- y la prueba lleva el de verdad. Sin la variable, el nodo es el de siempre.
# La misma cabeza da dos sobres: el honesto (`edad-todos.json`) y el ajeno
# (`neg-prueba-resto-ajeno.json`). El vector es el ajeno, tal como lo escribe el nodo.
#
# FUERA del canon y del arbol: todo vive en <dir>. Compila en RELEASE una copia del arbol en HEAD
# (`git archive`; otra revision con REV_CAPTURA), con su propio CARGO_TARGET_DIR, para no tocar los
# binarios del arbol.
#
# ⚠️ El nodo verifica su propia prueba antes de escribir el sobre
# (`crates/zk-ssl/src/prueba_edad.rs`, `edad::verificar_contra_cabeza`), con el canal del fork. Con
# el §713 en la copia rechazaria la ajena, y no habria sobre. Por eso, en la COPIA y solo en ella,
# el `read_remainder` de `crates/winter-verifier/src/channel.rs` deja de mirar el compromiso: es el
# canal de antes del §713. Una revision de antes del §713 no lleva la comprobacion y se compila tal
# cual. Si el `read_remainder` del fork cambia, el guion se para con su causa: no captura con un
# canal que no sabe desatar.
#
# Los sobres no salen byte a byte de una corrida a otra: la cabeza lleva `emittedAtUnix` y su firma,
# y la prueba va oculta y con sal. Sale la misma forma, y el mismo juicio del kit.
#
#   bash tools/vectores/resto_ajeno.sh <dir>
#   REV_CAPTURA=<revision> bash tools/vectores/resto_ajeno.sh <dir>
set -euo pipefail
msg(){ printf 'RESTO-AJENO| %s\n' "$*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }

AQUI=$(cd "$(dirname "$0")" && pwd)
RAIZ=$(git -C "$AQUI" rev-parse --show-toplevel)
DIR=${1:?uso: resto_ajeno.sh <dir>}
mkdir -p "$DIR"
DIR=$(cd "$DIR" && pwd)
PORT=8651   # libre: ningun banco de tools/ lo usa
PID=""
limpiar(){ if [ -n "$PID" ]; then kill -9 "$PID" 2>/dev/null || true; fi; }
trap limpiar EXIT INT TERM HUP QUIT

# ---------------------------------------------------------------- EL NODO DEL PARCHE
FRI=$(ls -d "${CARGO_HOME:-$HOME/.cargo}"/registry/src/*/winter-fri-0.13.1 | head -n 1)
[ -d "$FRI" ] || fallo "no esta winter-fri 0.13.1 en el registro de cargo"
rm -rf "$DIR/arbol" "$DIR/winter-fri"
mkdir -p "$DIR/arbol"
REV=${REV_CAPTURA:-HEAD}
git -C "$RAIZ" archive "$REV" | tar -x -C "$DIR/arbol"
msg "copia del arbol en $REV ($(git -C "$RAIZ" rev-parse --short "$REV"))"
# El canal de la copia, sin la comprobacion del §713 (ver la cabecera). Solo en <dir>/arbol.
CANAL="$DIR/arbol/crates/winter-verifier/src/channel.rs"
python3 - "$CANAL" <<'PY' || fallo "no se pudo desatar el resto en la copia"
import sys
ruta = sys.argv[1]
s = open(ruta).read()
ancla = "        if H::hash_elements(&remainder) != self.fri_remainder_commitment {\n"
if "fri_remainder_commitment" not in s:
    print("RESTO-AJENO| la copia es de antes del §713: su canal no ata el resto", file=sys.stderr)
    sys.exit(0)
if s.count(ancla) != 1:
    sys.exit("RESTO-AJENO| el read_remainder del fork cambio: no se sabe desatar el resto")
open(ruta, "w").write(s.replace(ancla, ancla.replace("if H::", "if false && H::")))
print("RESTO-AJENO| la copia lleva el §713: su read_remainder deja de mirar el compromiso",
      file=sys.stderr)
PY
cp -r "$FRI" "$DIR/winter-fri"
patch -s -d "$DIR/winter-fri" -p1 < "$AQUI/resto_ajeno.diff"
# ⚠️ Al portar el §713 (el §786 de `main`): el `winter-fri` del parche entra por ruta, y el
# `Cargo.lock` de la copia pierde las dos lineas de su origen en el registro, `source` y
# `checksum`, y nada mas. Se quitan aqui, y `cargo build --locked` comprueba que es justo lo que
# cargo pide: con otra deriva del lock, no compila.
python3 - "$DIR/arbol/Cargo.toml" "$DIR/arbol/Cargo.lock" "$DIR/winter-fri" <<'PY'
import sys
ruta, lock, fri = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(ruta).read()
ancla = 'winter-verifier = { path = "crates/winter-verifier" }\n'
assert s.count(ancla) == 1, "el [patch.crates-io] del arbol cambio"
open(ruta, "w").write(s.replace(ancla, ancla + f'winter-fri = {{ path = "{fri}" }}\n'))
s = open(lock).read()
ini = s.index('[[package]]\nname = "winter-fri"\nversion = "0.13.1"\n')
fin = s.index('\n\n', ini)
bloque = s[ini:fin]
fuera = [l for l in bloque.split("\n") if l.startswith(("source = ", "checksum = "))]
assert len(fuera) == 2, "el winter-fri del Cargo.lock cambio"
nuevo = "\n".join(l for l in bloque.split("\n") if l not in fuera)
open(lock, "w").write(s[:ini] + nuevo + s[fin:])
PY
msg "compilando cli y nodo con el parche (release, target propio)"
( cd "$DIR/arbol" \
  && CARGO_TARGET_DIR="$DIR/target" \
     cargo build --locked --release -q -p zk-ssl-cli -p zk-ssl-node ) \
  || fallo "no compila"
CLI="$DIR/target/release/zk-ssl-cli"
NODO="$DIR/target/release/zk-ssl-node"

# ---------------------------------------------------------------- EL LIBRO, LA CABEZA
T="$DIR/corrida"
rm -rf "$T"; mkdir -p "$T"
python3 -c "print('37'*96, end='')" > "$T/semilla.hex"
chmod 600 "$T/semilla.hex"
# ⚠️ Al portar el §713: las semillas de claves van de dos en dos, como en `tools/banco_edad.sh`
# desde el §802. Con `0xA11CE + S`, la corrida de S=8 volvia a abrir la identidad 0xA11D6 que ya
# abrio la de S=7, y desde el §802 una identidad abre una sola cuenta. El vector se capturo con la
# regla de antes y no cambia: esto es para recapturar sobre este arbol.
I=0
for S in 7 8; do
  I=$(( I + 1 ))
  "$CLI" --log warn simulate --ledger "$T/ledger" --no-claim \
    --salt-seed "$S" --key-seed "$(( 0xA11CE + 2 * S ))" --accounts "$(( 2 * I ))" \
    >"$T/sim-$S.txt" 2>&1 || fallo "el sandbox no dejo el envio en vuelo (salt-seed $S)"
done
"$NODO" --listen "127.0.0.1:$PORT" --latido 2 \
  --clave-fichero "$T/semilla.hex" --custodia fichero \
  --diario "$T/diario.jsonl" --ledger "$T/ledger" \
  --contador-recepcion "$T/recepcion.bin" --indice-firma "$T/indice-firma.bin" \
  --log warn 2>>"$T/nodo.err" &
PID=$!
CAB=""
for _ in $(seq 1 60); do
  V=$(curl -s --max-time 20 "http://127.0.0.1:$PORT" -H 'Content-Type: application/json' \
      -d '{"jsonrpc":"2.0","id":1,"method":"zkssl_signedEpochHead","params":{}}' \
      2>/dev/null || true)
  case "$V" in *'"available":true'*) CAB="$V"; break;; esac
  sleep 0.5
done
[ -n "$CAB" ] || fallo "no llego una cabeza firmada"
python3 -c 'import json,sys; json.dump(json.loads(sys.argv[1])["result"], open(sys.argv[2], "w"))' \
  "$CAB" "$T/cabeza.json"
kill -9 "$PID"; wait "$PID" 2>/dev/null || true
PID=""

# ---------------------------------------------------------------- LOS DOS SOBRES
"$NODO" --ledger "$T/ledger" --log warn \
  --prueba-edad "$DIR/edad-todos.json" --edad-cabeza "$T/cabeza.json" --edad-t 0 >&2 \
  || fallo "la capa no produjo el sobre honesto"
ARQUEO_RESTO_AJENO=1 "$NODO" --ledger "$T/ledger" --log warn \
  --prueba-edad "$DIR/neg-prueba-resto-ajeno.json" --edad-cabeza "$T/cabeza.json" --edad-t 0 >&2 \
  || fallo "la capa no produjo el sobre ajeno"
for f in edad-todos neg-prueba-resto-ajeno; do
  msg "$f.json  $(sha256sum "$DIR/$f.json" | cut -c1-16)  $(wc -c < "$DIR/$f.json") B"
done
msg "VERDE: dos sobres de la misma cabeza en $DIR"
