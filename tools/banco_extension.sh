#!/usr/bin/env bash
# tools/banco_extension.sh — el banco del (iii) (§293).
#
# Demuestra la EXTENSION como servicio: nodo real firmando latidos →
# cabeza VIEJA custodiada → mas latidos → zkssl_consistencyProof +
# cabeza NUEVA que FIRMA ese tamano → paquete de extension → el
# verificador en VERDE **sin el nodo**. Y los dos negativos de verdad:
# (a) un rearranque con diario NUEVO responde «va POR DETRAS» en el
#     cable — el reseteo visible que §292 prometio;
# (b) un nibble del camino adulterado cae en el verificador, exit 1.
# FUERA del canon: levanta procesos y espera latidos.
set -euo pipefail
cd "$(dirname "$0")/.."
msg(){ printf 'BANCO-EXT| %s\n' "$*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
command -v curl >/dev/null 2>&1 || fallo "curl no esta en el PATH"
command -v python3 >/dev/null 2>&1 || fallo "python3 no esta en el PATH"

PORT=8597
DIR=$(mktemp -d "$HOME/.banco_extension.XXXXXX")
PID=""
limpiar(){
  if [ -n "$PID" ]; then kill -9 "$PID" 2>/dev/null || true; fi
  rm -rf "$DIR"
}
trap limpiar EXIT

msg "compilando nodo y verificador"
cargo build -q -p zk-ssl-node -p zk-ssl-verify 2>/dev/null \
  || cargo build -p zk-ssl-node -p zk-ssl-verify || fallo "no compila"
NODO=target/debug/zk-ssl-node
VER=target/debug/zk-ssl-verify
python3 -c "print('37'*96, end='')" > "$DIR/semilla.hex"
chmod 600 "$DIR/semilla.hex"

arranca(){ # $1 diario  $2 ledger  $3 recepcion  $4 latido
  "$NODO" --listen "127.0.0.1:$PORT" --latido "$4" \
    --clave-fichero "$DIR/semilla.hex" --custodia fichero \
    --diario "$1" --ledger "$2" \
    --contador-recepcion "$3" \
    --indice-firma "$DIR/indice-firma.bin" \
    --log warn 2>>"$DIR/nodo.err" &
  PID=$!
}
rpc(){
  curl -s --max-time 15 "http://127.0.0.1:$PORT" \
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

arranca "$DIR/diario.jsonl" "$DIR/ledger" "$DIR/recepcion.bin" 2

VIEJA=""
for _ in $(seq 1 60); do
  V=$(rpc zkssl_signedEpochHead '{}' 2>/dev/null || true)
  case "$V" in
    *'"available":true'*)
      MS=$(campo "$V" result.mmrSize) || true
      if [ -n "${MS:-}" ] && [ "$(qnum "$MS")" -ge 2 ]; then VIEJA="$V"; break; fi
      ;;
  esac
  sleep 0.5
done
[ -n "$VIEJA" ] || fallo "no llego una cabeza firmada con mmrSize >= 2"
OLDSIZE=$(campo "$VIEJA" result.mmrSize)
msg "cabeza VIEJA custodiada (mmrSize $OLDSIZE)"

sleep 5  # que la historia crezca de verdad

ACK=""
for _ in $(seq 1 20); do
  ACK=$(rpc zkssl_consistencyProof "{\"oldSize\":$OLDSIZE}" || true)
  case "$ACK" in *'"available":true'*) break;; esac
  ACK=""; sleep 0.5
done
[ -n "$ACK" ] || fallo "consistencyProof no llego a available:true"
MA=$(campo "$ACK" result.mmrSize)
# ⚠️ La pareja FIRMADA es el acumulador ANTES de la cabeza (el push va
# despues del emit): el camino de tamano t lo firma LA SIGUIENTE cabeza
# en emitirse. Se espera hasta que una cabeza firme exactamente ese t.
NUEVA=""
for _ in $(seq 1 20); do
  V=$(rpc zkssl_signedEpochHead '{}' || true)
  case "$V" in
    *'"available":true'*)
      MN=$(campo "$V" result.mmrSize) || true
      if [ "${MN:-}" = "$MA" ]; then NUEVA="$V"; break; fi
      ;;
  esac
  sleep 0.5
done
[ -n "$NUEVA" ] || fallo "ninguna cabeza firmo mmrSize $MA (se esperaba en un latido)"
msg "camino y cabeza NUEVA emparejados: la cabeza que firma t=$MA llego"

python3 - "$VIEJA" "$NUEVA" "$ACK" "$DIR/extension.json" <<'PY'
import json, sys
vieja = json.loads(sys.argv[1])["result"]
nueva = json.loads(sys.argv[2])["result"]
ack = json.loads(sys.argv[3])["result"]
assert vieja.get("available") is True and nueva.get("available") is True
assert ack.get("available") is True
p = {"v": 1, "tipo": "extension", "vieja": vieja, "nueva": nueva, "camino": ack["camino"]}
open(sys.argv[4], "w").write(json.dumps(p))
print("paquete de extension armado: las respuestas TAL CUAL")
PY

kill -9 "$PID"; wait "$PID" 2>/dev/null || true
PID=""
msg "el nodo esta MUERTO; verificando la extension SIN el"
set +e; "$VER" "$DIR/extension.json"; RC=$?; set -e
[ "$RC" = "0" ] || fallo "la extension legitima dio exit $RC (se esperaba 0)"
msg "POSITIVO: exit 0 — la cabeza nueva PRUEBA que extiende a la custodiada"

python3 - "$DIR/extension.json" "$DIR/malo.json" <<'PY'
import json, sys
p = json.load(open(sys.argv[1]))
c = p["camino"][0]
ult = c[-1]
nuevo = '0' if ult != '0' else '1'
assert nuevo != ult
p["camino"][0] = c[:-1] + nuevo
open(sys.argv[2], "w").write(json.dumps(p))
print("adulterado UN nibble del camino[0]:", ult, "->", nuevo)
PY
set +e; SAL=$("$VER" "$DIR/malo.json" 2>&1); RC=$?; set -e
[ "$RC" = "1" ] || fallo "el camino adulterado dio exit $RC (se esperaba 1): $SAL"
msg "NEGATIVO-b: exit 1 — $SAL"

msg "rearrancando con diario NUEVO (latido 60: que t no alcance)"
arranca "$DIR/diario2.jsonl" "$DIR/ledger2" "$DIR/recepcion2.bin" 60
R=""
for _ in $(seq 1 40); do
  R=$(rpc zkssl_supply '{}' 2>/dev/null || true)
  case "$R" in *result*) break;; esac
  sleep 0.5
done
case "$R" in *result*) : ;; *) sed -n '1,20p' "$DIR/nodo.err" >&2; fallo "el nodo 2 no arranco";; esac
D=$(rpc zkssl_consistencyProof "{\"oldSize\":$OLDSIZE}")
case "$D" in
  *'"available":false'*'POR DETRAS'*|*'POR DETRAS'*'"available":false'*)
    msg "NEGATIVO-a: el cable DICE el reseteo — $(campo "$D" result.reason)" ;;
  *) fallo "el rearranque sin diario no se declaro por detras: $D" ;;
esac
kill -9 "$PID"; wait "$PID" 2>/dev/null || true
PID=""

msg "BANCO-EXTENSION VERDE: servido, verificado sin el nodo, y el reseteo habla"
