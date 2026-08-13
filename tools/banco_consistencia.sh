#!/usr/bin/env bash
# tools/banco_consistencia.sh — el banco del CONSUMIDOR (§295).
#
# `banco_extension.sh` (§293) demuestra la extension del lado del
# VERIFICADOR: paquete armado a mano, mando en VERDE sin el nodo. Este
# demuestra la del lado del TESTIGO (§294): el cliente VIVO que pide el
# camino, ESPERA a la cabeza que lo firma, juzga, anota — y se detiene o
# no segun lo que vea.
#
#   POSITIVO    nodo real + testigo real: el diario trae una linea
#               `extiende` con deT != aT y camino NO vacio — la extension
#               NO TRIVIAL, juzgada por el testigo. **Ningun unitario del
#               cli puede fabricar ese caso**: no hay Digest -> hex.
#   BONUS       `--auditar` sobre ese mismo diario: el criterio de §248
#               —lo suficiente para reverificar SIN el nodo— ejecutable.
#   NEGATIVO-A  un PROXY TONTO entre testigo y nodo adultera UN nibble
#               del camino: el testigo **SE DETIENE**, exit != 0, y su
#               ultima linea dice `no-extiende`. Es la version EN VIVO
#               del nibble de §290/§293; sin el, `NoExtiende` no tiene
#               negativo de verdad y este banco seria un adorno.
# ⚠️ En un fallo se enseñan las CLASES del diario, nunca el diario: cada
#    linea lleva la firma entera (~37 KB en hex, §248) y volcarlo ahoga
#    el log del bloque — medido en la primera corrida del §295.
#
#   NEGATIVO-B  el nodo se resetea DEBAJO del testigo (diario y ledger
#               nuevos, MISMO indice de firma): el testigo anota
#               `por-detras`, **descarta lo pendiente** y NO se detiene.
#               Es la decision D1 del §294 y la fe de erratas del §295,
#               las dos demostradas de una vez.
#
# ⚠️ El proxy es TONTO A PROPOSITO: reenvia todo tal cual y solo toca
#    `camino[0]` de `zkssl_consistencyProof`. Vive EMBEBIDO —nace en
#    $DIR y muere con el trap—: el censo de tools/ no gana una pieza por
#    algo cuya unica razon de existir es este banco.
#
# FUERA del canon: levanta procesos y espera latidos. Exit 0 = VERDE.
set -euo pipefail
cd "$(dirname "$0")/.."
msg(){ printf 'BANCO-CONS| %s\n' "$*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
command -v curl >/dev/null 2>&1 || fallo "curl no esta en el PATH"
command -v python3 >/dev/null 2>&1 || fallo "python3 no esta en el PATH"

PORT=8593
PROXY=8594
# Bajo $HOME, no /tmp: el guardian del indice (K.1, §234) se niega sobre
# un fsync que no persiste, y en WSL /tmp es tmpfs (medido en §290).
DIR=$(mktemp -d "$HOME/.banco_consistencia.XXXXXX")
NPID=""
PPID_=""
limpiar(){
  [ -n "$NPID" ] && kill -9 "$NPID" 2>/dev/null || true
  [ -n "$PPID_" ] && kill -9 "$PPID_" 2>/dev/null || true
  rm -rf "$DIR"
}
trap limpiar EXIT

msg "compilando nodo y cli"
cargo build -q -p zk-ssl-node -p zk-ssl-cli 2>/dev/null \
  || cargo build -p zk-ssl-node -p zk-ssl-cli || fallo "no compila"
NODO=target/debug/zk-ssl-node
CLI=target/debug/zk-ssl-cli
[ -x "$NODO" ] || fallo "falta $NODO"
[ -x "$CLI" ] || fallo "falta $CLI"

python3 -c "print('5b'*96, end='')" > "$DIR/semilla.hex"
chmod 600 "$DIR/semilla.hex"

arranca(){ # $1 diario  $2 ledger  $3 recepcion  $4 latido
  "$NODO" --listen "127.0.0.1:$PORT" --latido "$4" \
    --clave-fichero "$DIR/semilla.hex" --custodia fichero \
    --diario "$1" --ledger "$2" \
    --contador-recepcion "$3" \
    --indice-firma "$DIR/indice-firma.bin" \
    --log warn 2>>"$DIR/nodo.err" &
  NPID=$!
}
espera_vivo(){
  local R=""
  for _ in $(seq 1 40); do
    R=$(curl -s --max-time 10 "http://127.0.0.1:$PORT" \
        -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","id":1,"method":"zkssl_supply","params":{}}' 2>/dev/null || true)
    case "$R" in *result*) return 0;; esac
    sleep 0.5
  done
  sed -n '1,20p' "$DIR/nodo.err" >&2
  fallo "el nodo no arranco en 20 s"
}

# ── el proxy TONTO, embebido ──────────────────────────────────────────
cat > "$DIR/proxy.py" <<'PROXYEOF'
"""Proxy TONTO del banco del §295. Reenvia TODO tal cual y solo toca
`camino[0]` de `zkssl_consistencyProof`: un nibble, como el de §290/§293
pero EN VIVO, sobre el cable que el testigo esta consumiendo.
Si el camino viene vacio (caso identidad) inyecta uno falso: un camino
donde no debe haberlo tambien es una historia que no extiende."""
import http.server, json, sys, urllib.request

DEST = sys.argv[1]
PUERTO = int(sys.argv[2])
FALSO = "0x" + "ab" * 32


class H(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_POST(self):
        n = int(self.headers.get("Content-Length", 0))
        cuerpo = self.rfile.read(n)
        try:
            pet = urllib.request.Request(
                DEST, data=cuerpo, headers={"Content-Type": "application/json"}
            )
            with urllib.request.urlopen(pet, timeout=15) as r:
                crudo = r.read()
        except Exception as e:
            crudo = json.dumps({"jsonrpc": "2.0", "id": 1,
                                "error": {"code": -32000,
                                          "message": "proxy: %s" % e}}).encode()
        try:
            if json.loads(cuerpo).get("method") == "zkssl_consistencyProof":
                d = json.loads(crudo)
                res = d.get("result") or {}
                cam = res.get("camino")
                if cam:
                    s = cam[0]
                    ult = s[-1]
                    cam[0] = s[:-1] + ("0" if ult != "0" else "1")
                elif res.get("available") is True:
                    res["camino"] = [FALSO]
                else:
                    raise ValueError("nada que adulterar")
                crudo = json.dumps(d).encode()
        except Exception:
            pass
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(crudo)))
        self.end_headers()
        self.wfile.write(crudo)

    def log_message(self, *a):
        pass


http.server.HTTPServer(("127.0.0.1", PUERTO), H).serve_forever()
PROXYEOF

# ── el analizador del diario, embebido ────────────────────────────────
cat > "$DIR/mira.py" <<'MIRAEOF'
"""Lee un diario del testigo y responde una pregunta por invocacion.
Sale 0 si la respuesta es SI, 1 con su razon si es NO."""
import json, sys

ruta, pregunta = sys.argv[1], sys.argv[2]
ls = []
for l in open(ruta, encoding="utf-8"):
    if l.strip():
        try:
            ls.append(json.loads(l))
        except Exception:
            print("linea ilegible en el diario"); sys.exit(1)
if not ls:
    print("el diario esta VACIO"); sys.exit(1)
cons = [l.get("consistencia") for l in ls if l.get("consistencia")]
clases = [c["clase"] for c in cons]

if pregunta == "extension-no-trivial":
    for c in cons:
        if c["clase"] == "extiende" and c.get("deT") != c.get("aT") and c.get("camino"):
            print("EXTIENDE no trivial: deT=%s aT=%s |camino|=%d"
                  % (c["deT"], c["aT"], len(c["camino"])))
            sys.exit(0)
    print("ninguna linea `extiende` con deT != aT y camino NO vacio; clases: %s" % clases)
    sys.exit(1)

if pregunta == "acaba-en-no-extiende":
    if clases and clases[-1] == "no-extiende":
        print("la ultima palabra del testigo: %s" % clases[-1]); sys.exit(0)
    print("la ultima clase fue %r, no `no-extiende`; clases: %s"
          % (clases[-1] if clases else None, clases))
    sys.exit(1)

if pregunta == "hay-por-detras":
    if "por-detras" in clases:
        print("`por-detras` anotado, y el testigo siguio; clases: %s" % clases)
        sys.exit(0)
    print("ninguna linea `por-detras`; clases: %s" % clases)
    sys.exit(1)

print("pregunta desconocida: %s" % pregunta)
sys.exit(2)
MIRAEOF

testigo(){ # $1 puerto  $2 diario  $3 cada  $4 veces
  "$CLI" witness --nodo "http://127.0.0.1:$1" \
    --cada "$3" --veces "$4" --diario "$2" --no-color
}

# ══ POSITIVO ══════════════════════════════════════════════════════════
msg "POSITIVO: nodo con latido 1, testigo cada 1 s x 14 vueltas (la CADENCIA DEL LATIDO: muestrear mas despacio pierde la cabeza que firma el camino)"
arranca "$DIR/diario.jsonl" "$DIR/ledger" "$DIR/recepcion.bin" 1
espera_vivo
set +e; testigo "$PORT" "$DIR/d1.jsonl" 1 14 > "$DIR/t1.out" 2>&1; RC=$?; set -e
[ "$RC" = "0" ] || { cat "$DIR/t1.out" >&2; fallo "el testigo honesto salio con $RC (se esperaba 0)"; }
SAL=$(python3 "$DIR/mira.py" "$DIR/d1.jsonl" extension-no-trivial) \
  || fallo "$SAL"
msg "POSITIVO: $SAL"

# ══ BONUS: el diario v2 se reverifica SIN el nodo ═════════════════════
set +e; SAL=$("$CLI" witness --auditar "$DIR/d1.jsonl" --no-color 2>&1); RC=$?; set -e
[ "$RC" = "0" ] || fallo "--auditar dio exit $RC sobre un diario legitimo: $SAL"
msg "BONUS: $(printf '%s' "$SAL" | tr '\n' ' ')"

# ══ NEGATIVO-A: el proxy adultera el camino EN VIVO ══════════════════
msg "NEGATIVO-A: proxy TONTO en $PROXY -> nodo en $PORT (un nibble del camino)"
python3 "$DIR/proxy.py" "http://127.0.0.1:$PORT" "$PROXY" 2>>"$DIR/proxy.err" &
PPID_=$!
for _ in $(seq 1 20); do
  curl -s --max-time 5 "http://127.0.0.1:$PROXY" -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"zkssl_supply","params":{}}' >/dev/null 2>&1 && break
  sleep 0.5
done
set +e; testigo "$PROXY" "$DIR/d2.jsonl" 1 16 > "$DIR/t2.out" 2>&1; RC=$?; set -e
[ "$RC" != "0" ] || fallo "el testigo NO se detuvo ante un camino adulterado (exit 0)"
SAL=$(python3 "$DIR/mira.py" "$DIR/d2.jsonl" acaba-en-no-extiende) \
  || fallo "$SAL"
msg "NEGATIVO-A: exit $RC — $SAL"
kill -9 "$PPID_" 2>/dev/null || true; wait "$PPID_" 2>/dev/null || true; PPID_=""

# ══ NEGATIVO-B: el nodo se resetea DEBAJO del testigo ════════════════
msg "NEGATIVO-B: el testigo corre y el nodo rearranca sin diario a media corrida"
testigo "$PORT" "$DIR/d3.jsonl" 1 20 > "$DIR/t3.out" 2>&1 &
TPID=$!
sleep 8   # que ancle, pida camino y llegue a tener PENDIENTE viva
kill -9 "$NPID" 2>/dev/null || true; wait "$NPID" 2>/dev/null || true; NPID=""
msg "  nodo muerto; rearrancando con diario y ledger NUEVOS (mismo indice de firma)"
arranca "$DIR/diario2.jsonl" "$DIR/ledger2" "$DIR/recepcion2.bin" 1
espera_vivo
set +e; wait "$TPID"; RC=$?; set -e
[ "$RC" = "0" ] || { cat "$DIR/t3.out" >&2; fallo "el testigo SE DETUVO ante un reseteo legitimo (exit $RC): un reinicio no puede quemarlo"; }
SAL=$(python3 "$DIR/mira.py" "$DIR/d3.jsonl" hay-por-detras) \
  || fallo "$SAL"
msg "NEGATIVO-B: exit 0 — $SAL"

msg "BANCO-CONSISTENCIA VERDE: el testigo juzga la historia, se detiene cuando debe y aguanta un reseteo"
