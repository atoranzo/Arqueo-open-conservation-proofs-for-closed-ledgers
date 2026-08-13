#!/usr/bin/env bash
# tools/banco_apagado.sh — el banco de la nota 91 (§290).
#
# Demuestra el APAGADO: levanta un nodo REAL (clave + diario + ledger),
# fondea una posicion, captura el paquete de evidencia por el cable
# (zkssl_ackPath y zkssl_signedEpochHead espalda contra espalda, dentro
# del mismo latido), MATA el proceso con kill -9, y verifica que la
# posicion se sostiene con el paquete solo: zk-ssl-verify en VERDE.
# Despues adultera UN nibble del hashPrueba y exige el ROJO con exit 1.
#
# FUERA del canon: levanta procesos y espera latidos. Se corre a mano
# o desde un bloque. Exit 0 = BANCO VERDE; cualquier fallo, exit 1.
set -euo pipefail
cd "$(dirname "$0")/.."
msg(){ printf 'BANCO| %s\n' "$*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
command -v curl >/dev/null 2>&1 || fallo "curl no esta en el PATH"
command -v python3 >/dev/null 2>&1 || fallo "python3 no esta en el PATH"

PORT=8599
# La casa del banco vive bajo $HOME, no en /tmp: el guardian del indice
# (K.1, §234) se niega a arrancar sobre un fsync que no persiste, y en
# WSL /tmp es tmpfs — medido en §290: razon 1.0x frente al minimo 10x.
DIR=$(mktemp -d "$HOME/.banco_apagado.XXXXXX")
PID=""
limpiar(){
  if [ -n "$PID" ]; then kill -9 "$PID" 2>/dev/null || true; fi
  rm -rf "$DIR"
}
trap limpiar EXIT

msg "compilando nodo y verificador (silencioso; si falla, en voz alta)"
cargo build -q -p zk-ssl-node -p zk-ssl-verify 2>/dev/null \
  || cargo build -p zk-ssl-node -p zk-ssl-verify || fallo "no compila"
NODO=target/debug/zk-ssl-node
VER=target/debug/zk-ssl-verify
[ -x "$NODO" ] || fallo "falta $NODO"
[ -x "$VER" ] || fallo "falta $VER"

python3 -c "print('42'*96, end='')" > "$DIR/semilla.hex"
chmod 600 "$DIR/semilla.hex"

"$NODO" --listen "127.0.0.1:$PORT" --dev --latido 3 \
  --clave-fichero "$DIR/semilla.hex" --custodia fichero \
  --diario "$DIR/diario.jsonl" --ledger "$DIR/ledger" \
  --contador-recepcion "$DIR/recepcion.bin" \
  --indice-firma "$DIR/indice-firma.bin" \
  --log warn 2>"$DIR/nodo.err" &
PID=$!

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

R=""
for _ in $(seq 1 40); do
  R=$(rpc zkssl_supply '{}' 2>/dev/null || true)
  case "$R" in *result*) break;; esac
  sleep 0.5
done
case "$R" in
  *result*) msg "nodo vivo (pid $PID) en 127.0.0.1:$PORT" ;;
  *) sed -n '1,20p' "$DIR/nodo.err" >&2; fallo "el nodo no arranco" ;;
esac

R=$(rpc dev_openSeeded '{"seed":"0x1"}') || fallo "el cable callo en dev_openSeeded"
case "$R" in *'"error"'*) fallo "dev_openSeeded: $R";; esac
IDX=$(campo "$R" result.index) || fallo "sin index: $R"

R=$(rpc dev_fund "{\"index\":$IDX,\"amount\":\"0x64\"}") || fallo "el cable callo en dev_fund"
case "$R" in *'"error"'*) fallo "dev_fund: $R";; esac
SEQ=$(campo "$R" result.logSeq) || fallo "sin logSeq: $R"
msg "posicion fondeada: cuenta $IDX, entrada $SEQ del registro"

R=$(rpc zkssl_logEntry "{\"seq\":$SEQ}") || fallo "el cable callo en logEntry"
case "$R" in *proofDigest*) : ;; *) fallo "logEntry sin proofDigest: $R";; esac
PD=$(campo "$R" result.proofDigest) || fallo "no pude leer proofDigest"

ACK=""
for _ in $(seq 1 40); do
  # un intento puede caer ENCIMA del latido (la firma XMSS en un binario
  # debug bloquea el cable segundos): el silencio transitorio se tolera,
  # solo el agotamiento del sondeo es rojo — y con nombre.
  ACK=$(rpc zkssl_ackPath "{\"seq\":$SEQ}" || true)
  case "$ACK" in *'"available":true'*) break;; esac
  sleep 0.5
done
case "$ACK" in
  *'"available":true'*) : ;;
  *) fallo "el sondeo de ackPath se agoto sin epoca cerrada; ultima respuesta: $ACK" ;;
esac
CAB=$(rpc zkssl_signedEpochHead '{}') || fallo "el cable callo en signedEpochHead"
case "$CAB" in
  *'"available":true'*) msg "capturas espalda contra espalda: ackPath + signedEpochHead" ;;
  *) fallo "signedEpochHead sin firma: $CAB" ;;
esac

python3 - "$CAB" "$ACK" "$SEQ" "$PD" "$DIR/paquete.json" <<'PY'
import json, sys
cab = json.loads(sys.argv[1])["result"]
ack = json.loads(sys.argv[2])["result"]
assert cab.get("available") is True
assert ack.get("available") is True
paquete = {
    "v": 1,
    "cabeza": cab,
    "acuse": {
        "seq": json.loads(sys.argv[3]),
        "hashPrueba": json.loads(sys.argv[4]),
        "s": ack["s"],
        "camino": ack["camino"],
    },
}
open(sys.argv[5], "w").write(json.dumps(paquete))
print("paquete v1 armado: las respuestas TAL CUAL, reunidas")
PY

msg "matando el nodo: kill -9 $PID (no es una cortesia)"
kill -9 "$PID"
wait "$PID" 2>/dev/null || true
if kill -0 "$PID" 2>/dev/null; then fallo "el proceso sigue vivo"; fi
R=$(rpc zkssl_supply '{}' 2>/dev/null || true)
case "$R" in *result*) fallo "el RPC responde despues del kill";; esac
PID_MUERTO=$PID
PID=""
msg "el nodo esta MUERTO y el cable no responde"

set +e; "$VER" "$DIR/paquete.json"; RC=$?; set -e
[ "$RC" = "0" ] || fallo "el paquete legitimo dio exit $RC (se esperaba 0)"
msg "POSITIVO: exit 0 — la posicion se sostiene SIN el nodo"

python3 - "$DIR/paquete.json" "$DIR/malo.json" <<'PY'
import json, sys
p = json.load(open(sys.argv[1]))
h = p["acuse"]["hashPrueba"]
ult = h[-1]
nuevo = '0' if ult != '0' else '1'
assert nuevo != ult
p["acuse"]["hashPrueba"] = h[:-1] + nuevo
open(sys.argv[2], "w").write(json.dumps(p))
print("adulterado UN nibble de hashPrueba:", ult, "->", nuevo)
PY
set +e; SALIDA=$("$VER" "$DIR/malo.json" 2>&1); RC=$?; set -e
[ "$RC" = "1" ] || fallo "el adulterado dio exit $RC (se esperaba 1): $SALIDA"
msg "NEGATIVO: exit 1 — $SALIDA"

msg "BANCO-APAGADO VERDE: nodo $PID_MUERTO matado, posicion demostrada sin el"
