#!/usr/bin/env bash
# tools/banco_pago.sh -- el banco de E2 del RFC-0008, lado del PAGADOR (D-AI).
#
# Demuestra la PRUEBA PORTABLE DEL PAGO EN CURSO de punta a punta, y es la PRIMERA vez que la
# boca del pagador habla con un nodo: hasta el S507 solo se habia ejercitado contra la capa. Un
# libro con un pendiente v2 VIVO -puesto por el sandbox del cli con el nodo PARADO, con prueba
# STARK real y por la via v2, que es la que lleva el sobre `X`- -> nodo real que lo abre, lo mete
# en la FOTO de su latido y FIRMA una cabeza v5 -> la BOCA del pagador (`zk-ssl-cli prueba-pago`)
# pide la cabeza y la foto CON `receiverId` (D-AE, S505), prueba en el cliente y escribe el sobre
# `pago_en_curso` (spec/PAQUETE.md 2.9) -> el nodo MUERE -> el verificador en VERDE **sin el
# nodo**. Dos positivos; DOS rechazos EN VIVO, uno de ellos la puerta barata que no llega ni a
# pedir; y SIETE negativos sobre el sobre, uno por regla.
#
# ⚠️ EL ORDEN ES EL DEL HERMANO (`tools/banco_pendiente.sh`, S498), y por la misma razon: la foto
# de los pendientes vive en el latido, en memoria del nodo, y `zkssl_pendingPath` la sirve solo
# con el nodo vivo (D-F). Y `sled` abre el libro en EXCLUSIVA, asi que la siembra va ANTES de
# arrancar el nodo, y nada de lo que sigue vuelve a abrir el libro.
#
# ⚠️ LO QUE EL PAGADOR TIENE Y EL COBRADOR NO, y por que la siembra escribe CUATRO ficheros: el
# aviso (posicion, sal, importe y `x`) y la credencial del RECEPTOR los escribia ya el S497 (D-P);
# el RETORNO -la pareja `(refundId, delta)` con la que se compuso `X`, que el receptor recibe
# OPACA- lo escribe el S507 (D-AI); y la credencial del PAGADOR la escribe el S508, porque
# `zkssl_pendingPath` exige SU credencial y no la del receptor. Cuatro ficheros, dos duenos.
#
# ⚠️ LA VENTANA DEL LATIDO: la boca pide la cabeza y DESPUES la foto, y exige que sean del MISMO
# latido; si cae uno entre las dos llamadas, muere con su texto y NO reintenta. Aqui se reintenta,
# como en el hermano: una corrida que muriera por ahi no distinguiria una carrera de un fallo real.
#
# ⚠️ `--t` es ABSOLUTO (D-AI-4), y aqui NO se teclea: el primer positivo usa el `seq` de la cabeza
# que el nodo sirve, y de SU sobre se lee el `nacido` para derivar la FRONTERA `nacido + delta`,
# que es el segundo positivo y, con uno mas, el rechazo en vivo. El `delta` sale del retorno, que
# es su unica fuente.
#
# FUERA del canon: levanta procesos y espera latidos. Compila en RELEASE porque aqui se firma y se
# prueba de verdad.
#
# NO ESCRIBE EN EL ARBOL: todo lo suyo vive en un temporal bajo $HOME, que borra al salir. Lo
# comprueba al final por `git status --porcelain`, por DELTA.
#
#   bash tools/banco_pago.sh [--guardar <dir>]        (cuando ya vive en el arbol)
#   cd ~/zk-ssl-real && bash <ruta-suelta> [--guardar <dir>]   (mientras vive en Downloads)
#
# --guardar  copia a <dir>, con su huella: los DOS sobres positivos, los SIETE cuerpos negativos y
#            los cuatro ficheros de la siembra (sin ellos una captura no se puede volver a
#            producir). NO guarda la cabeza aparte: el sobre la lleva VERBATIM (D-J). De estas
#            capturas se derivan los vectores del catalogo `pago-*` (regla 2 del PROCESO: los
#            vectores jamas se reescriben, se derivan de capturas reales).
#
# ⚠️ Los fragmentos de rechazo de abajo estan LEIDOS del fuente -del mando
# (`crates/zk-ssl-verify/src/main.rs`), del juez del AIR (`zk-ssl-air::pago_en_curso`) y de la
# boca (`crates/zk-ssl-cli/src/pago.rs`)-, salvo el de la prueba que no verifica, que lo pone
# winterfell y no se predice: por eso `niega` exige el fragmento Y dice <<cayo, pero NO por su
# regla>> cuando el rojo es de otro. El instrumento mide; no adivina.
set -euo pipefail
msg(){ printf 'BANCO-PAGO| %s\n' "$*" >&2; }
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
  *) fallo "uso: bash tools/banco_pago.sh [--guardar <dir>]" ;;
esac

PORT=8615   # libre: censados en tools/*.sh estan 8593, 8594 (PROXY), 8597, 8598, 8599, 8600,
            # 8601, 8603, 8605, 8607, 8609, 8611 y 8613
LATIDO=3    # corto: el primer latido cae tras LATIDO segundos, y la carrera se reintenta
DIR=$(mktemp -d "$HOME/.banco_pago.XXXXXX")
PID=""
limpiar(){
  if [ -n "$PID" ]; then kill -9 "$PID" 2>/dev/null || true; fi
  rm -rf "$DIR"
}
trap limpiar EXIT INT TERM HUP QUIT

# ⚠️ LA BASE DE LA PUREZA, tomada ANTES de nada. El invariante de un banco no es <<el arbol esta
# limpio>> sino <<yo no lo ensucie>> (PRECISION 169): este banco corre tambien DENTRO del bloque
# que sella su propio corte, y un gate absoluto cobraria deuda AJENA. Se mide por DELTA.
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
qfich(){ python3 -c 'import json,sys; v=json.load(open(sys.argv[1]))[sys.argv[2]]; print(int(v,16) if sys.argv[3]=="n" else v)' "$1" "$2" "$3"; }

# ---------------------------------------------------------------- LA SIEMBRA, CON EL NODO PARADO
# UNA sola corrida del sandbox: los dos positivos son dos `--t` sobre EL MISMO pendiente, y una
# corrida evita la trampa de las posiciones logicas del arbol disperso. Nada de lo que sigue se
# teclea: el importe sale del aviso, el delta del retorno, y las dos credenciales de sus ficheros.
msg "sandbox: un envio v2 EN VUELO, con los CUATRO ficheros de la siembra"
"$CLI" --log warn simulate --ledger "$DIR/ledger" --no-claim --v2 \
  --aviso "$DIR/aviso.json" --credencial "$DIR/credencial.json" \
  --retorno "$DIR/retorno.json" --credencial-pagador "$DIR/credencial-pagador.json" \
  >"$DIR/sim.txt" 2>&1 || { sed 's/^/BANCO-PAGO|   /' "$DIR/sim.txt" >&2; fallo "el sandbox no dejo el envio v2 en vuelo"; }
for f in aviso credencial retorno credencial-pagador; do
  [ -s "$DIR/$f.json" ] || fallo "el sandbox no escribio $f.json"
done
X=$(qfich "$DIR/aviso.json" x s 2>/dev/null || true)
[ -n "$X" ] || fallo "el aviso no lleva x: es v1, y E2 es del compromiso v2"
IMPORTE=$(qfich "$DIR/aviso.json" amount n)
RECEPTOR=$(qfich "$DIR/credencial.json" publicId s)
DELTA=$(qfich "$DIR/retorno.json" delta n)
REFUNDID=$(qfich "$DIR/retorno.json" refundId s)
INDEX=$(qfich "$DIR/credencial-pagador.json" index s)
VIEWKEY=$(qfich "$DIR/credencial-pagador.json" viewKey s)
IDPAG=$(qfich "$DIR/credencial-pagador.json" publicId s)
[ "$INDEX" != "$(qfich "$DIR/credencial.json" index s)" ] \
  || fallo "las dos credenciales son del MISMO indice: --credencial-pagador no es del pagador"
msg "sembrado: importe $IMPORTE, delta $DELTA (leidos de sus ficheros); pagador con index $(qnum "\"$INDEX\"")"

# ---------------------------------------------------------------- EL NODO VIVO
"$NODO" --listen "127.0.0.1:$PORT" --latido "$LATIDO" \
  --clave-fichero "$DIR/semilla.hex" --custodia fichero \
  --diario "$DIR/diario.jsonl" --ledger "$DIR/ledger" \
  --contador-recepcion "$DIR/recepcion.bin" \
  --indice-firma "$DIR/indice-firma.bin" \
  --log warn 2>>"$DIR/nodo.err" &
PID=$!

CAB=""
for _ in $(seq 1 120); do
  V=$(rpc zkssl_signedEpochHead '{}' 2>/dev/null || true)
  case "$V" in *'"available":true'*) CAB="$V"; break;; esac
  sleep 0.5
done
[ -n "$CAB" ] || { sed 's/^/BANCO-PAGO|   /' "$DIR/nodo.err" >&2; fallo "no llego una cabeza firmada"; }
FV=$(campo "$CAB" result.formatVersion)
SEQ=$(qnum "$(campo "$CAB" result.seq)")
NP=$(campo "$CAB" result.nextPending)
[ "$(qnum "$FV")" = "5" ] \
  || fallo "la cabeza dice formatVersion $(qnum "$FV"): el pago en curso exige v5, la unica que firma pmetaRoot"
[ "$(qnum "$NP")" -ge 1 ] \
  || fallo "la cabeza firma nextPending $(qnum "$NP"): el sandbox no dejo la posicion viva"
msg "cabeza v5 firmada: seq $SEQ - nextPending $(qnum "$NP"); el nodo sigue VIVO para la boca"

# ---------------------------------------------------------------- LA BOCA, CON EL NODO VIVO
# `boca` corre la boca del pagador y REINTENTA solo ante la carrera del latido: el texto es el de
# la boca, leido de `cobro.rs`, que `pago.rs` reusa tal cual. Deja la salida en BOCA_S y el rc en
# BOCA_R, y quien la llama decide que esperaba; asi los dos rechazos en vivo pasan por la misma
# ventana.
BOCA_S=""; BOCA_R=""
boca(){ # boca <t> <retorno> <salida> <rotulo>
  local tt="$1" ret="$2" out="$3" rot="$4" i t0 t1
  for i in 1 2 3 4 5; do
    t0=$(date +%s)
    set +e
    BOCA_S=$("$CLI" --log warn prueba-pago --nodo "http://127.0.0.1:$PORT" \
      --aviso "$DIR/aviso.json" --retorno "$ret" --index "$INDEX" --receptor "$RECEPTOR" \
      --view-key "$VIEWKEY" --t "$tt" --salida "$out" 2>&1)
    BOCA_R=$?
    set -e
    t1=$(date +%s)
    case "$BOCA_S" in
      *"cayo un latido entre las dos llamadas"*)
        msg "carrera del latido en $rot (intento $i de 5): se vuelve a pedir"; sleep 1; continue ;;
    esac
    msg "boca $rot: exit $BOCA_R en $((t1 - t0)) s"
    echo "$BOCA_S" | sed 's/^/BANCO-PAGO|   /' >&2
    return 0
  done
  fallo "la boca cayo CINCO veces seguidas por la ventana del latido en $rot: eso ya no es una carrera"
}

# PRIMER POSITIVO: `--t` = el `seq` de la cabeza que el nodo acaba de servir. Es la afirmacion
# minima honesta -<<atado al menos hasta ahora>>- y no exige conocer el `nacido` todavia.
boca "$SEQ" "$DIR/retorno.json" "$DIR/pago-t-seq.json" "t-seq"
[ "$BOCA_R" = "0" ] && [ -s "$DIR/pago-t-seq.json" ] \
  || fallo "la boca no escribio el sobre con t = seq (exit $BOCA_R)"

# LA FRONTERA SE DERIVA: el `nacido` sale del sobre que la boca acaba de escribir, y el `delta`
# del retorno. `nacido + delta` es la ultima epoca que el pago sostiene.
NACIDO=$(python3 -c 'import json,sys; print(int(json.load(open(sys.argv[1]))["enunciado"]["nacido"],16))' "$DIR/pago-t-seq.json")
FRONTERA=$((NACIDO + DELTA))
msg "frontera DERIVADA: nacido $NACIDO (del sobre) + delta $DELTA (del retorno) = $FRONTERA"
[ "$NACIDO" -lt "$SEQ" ] || fallo "el sobre dice nacido $NACIDO y la cabeza seq $SEQ: no puede ser"

boca "$FRONTERA" "$DIR/retorno.json" "$DIR/pago-t-frontera.json" "t-frontera"
[ "$BOCA_R" = "0" ] && [ -s "$DIR/pago-t-frontera.json" ] \
  || fallo "la boca no escribio el sobre con t = nacido + delta (exit $BOCA_R)"

# RECHAZO EN VIVO 1: una epoca mas alla de la frontera. El productor de la capa se para por su
# nombre (leido de `prueba_pago.rs`) y la boca no escribe nada.
boca $((FRONTERA + 1)) "$DIR/retorno.json" "$DIR/no-debe-existir.json" "t-pasada-la-frontera"
[ "$BOCA_R" != "0" ] \
  || fallo "la boca ESCRIBIO un sobre con t mas alla de la frontera: el pago no se sostiene y aun asi probo"
case "$BOCA_S" in
  *"NO se sostiene hasta T"*) msg "RECHAZO EN VIVO 1: la boca se paro por su regla con t $((FRONTERA + 1))" ;;
  *) fallo "la boca cayo con t = frontera + 1, pero NO por su regla: se esperaba <<NO se sostiene hasta T>>" ;;
esac
[ ! -e "$DIR/no-debe-existir.json" ] || fallo "la boca dejo un fichero al rechazar: un sobre que no prueba no se escribe"

# RECHAZO EN VIVO 2, Y ES EL BARATO (D-AI-3): un retorno que no es el de ESE aviso. La boca lo
# caza recomponiendo el `x` ANTES de pedirle nada al nodo, y por eso este rechazo no gasta ni una
# llamada ni una prueba. El delta mentido va +1, que no colisiona en Goldilocks (D-AG).
python3 - "$DIR" "$REFUNDID" "$DELTA" <<'PY'
import json, sys
d, rid, delta = sys.argv[1], sys.argv[2], int(sys.argv[3])
json.dump({"refundId": rid, "delta": hex(delta + 1)}, open(d + "/retorno-mentido.json", "w"))
PY
boca "$SEQ" "$DIR/retorno-mentido.json" "$DIR/tampoco-debe-existir.json" "retorno-ajeno"
[ "$BOCA_R" != "0" ] || fallo "la boca ESCRIBIO un sobre con un retorno que no abre el aviso"
case "$BOCA_S" in
  *"no recompone su x"*) msg "RECHAZO EN VIVO 2: la puerta barata cazo el retorno ajeno SIN tocar el nodo" ;;
  *) fallo "la boca cayo con el retorno mentido, pero NO por su regla: se esperaba <<no recompone su x>>" ;;
esac
[ ! -e "$DIR/tampoco-debe-existir.json" ] || fallo "la boca dejo un fichero al rechazar el retorno"

kill -9 "$PID"; wait "$PID" 2>/dev/null || true
PID=""
msg "el nodo esta MUERTO; a partir de aqui NADA necesita red ni libro"

# ---------------------------------------------------------------- EL MANDO, SIN EL NODO
for f in pago-t-seq pago-t-frontera; do
  set +e; SAL=$("$VER" "$DIR/$f.json" 2>&1); RC=$?; set -e
  [ "$RC" = "0" ] || fallo "el sobre POSITIVO $f dio exit $RC (se esperaba 0): $SAL"
  echo "$SAL" | sed 's/^/BANCO-PAGO|   /' >&2
done
# Lo que los dos sobres afirman se LEE de ellos, no se supone: el mismo receptor, el mismo importe
# EXACTO, el mismo nacido, y las dos epocas que la boca recibio.
python3 - "$DIR" "$IMPORTE" "$RECEPTOR" "$SEQ" "$FRONTERA" "$IDPAG" <<'PY'
import json, sys
d, importe, receptor = sys.argv[1], int(sys.argv[2]), sys.argv[3]
seq, frontera, idpag = int(sys.argv[4]), int(sys.argv[5]), sys.argv[6]
a = json.load(open(d + "/pago-t-seq.json"))
b = json.load(open(d + "/pago-t-frontera.json"))
for s, n in ((a, "t-seq"), (b, "t-frontera")):
    assert s["tipo"] == "pago_en_curso" and s["v"] == 1, n
    assert s["enunciado"]["receptor"] == receptor, "%s: el receptor no es el de la credencial" % n
    assert int(s["enunciado"]["importe"], 16) == importe, "%s: el importe no es el del aviso" % n
    assert int(s["enunciado"]["nacido"], 16) < int(s["cabeza"]["seq"], 16), "%s: nacido >= seq" % n
    assert "sal" not in json.dumps(s["enunciado"]), "%s: el enunciado no publica la sal" % n
    assert idpag not in json.dumps(s), "%s: el sobre lleva la identidad del PAGADOR" % n
assert int(a["enunciado"]["t"], 16) == seq, "t-seq no dice el seq"
assert int(b["enunciado"]["t"], 16) == frontera, "t-frontera no dice nacido + delta"
assert a["enunciado"]["nacido"] == b["enunciado"]["nacido"], "los dos sobres no son del mismo pendiente"
print("los dos sobres afirman lo pedido: receptor de la credencial, importe %d EXACTO, nacido %d < seq %d, t %d y %d; prueba %d B y %d B"
      % (importe, int(a["enunciado"]["nacido"], 16), int(a["cabeza"]["seq"], 16), seq, frontera,
         (len(a["prueba"]) - 2) // 2, (len(b["prueba"]) - 2) // 2))
PY
msg "POSITIVOS: exit 0 los dos, sin el nodo; y ninguno publica ni la sal, ni la pareja, ni al pagador"

# ---------------------------------------------------------------- LOS SIETE NEGATIVOS
ROTOS=0
niega(){ # niega <fichero> <fragmento esperado> <rotulo> [fragmento PROHIBIDO]
  local f="$1" frag="$2" rot="$3" no="${4:-}" s r
  set +e; s=$("$VER" "$f" 2>&1); r=$?; set -e
  [ "$r" = "1" ] || fallo "NEGATIVO $rot dio exit $r (se esperaba 1): $s"
  case "$s" in
    *"$frag"*) : ;;
    *) fallo "NEGATIVO $rot cayo, pero NO por su regla: se esperaba <<$frag>> y dijo: $s" ;;
  esac
  if [ -n "$no" ]; then
    case "$s" in *"$no"*) fallo "NEGATIVO $rot cayo por OTRA regla (<<$no>>), no por la suya: $s" ;; esac
  fi
  msg "NEGATIVO $rot: exit 1 -- $(echo "$s" | tail -n 1)"; ROTOS=$((ROTOS+1))
}

python3 - "$DIR" "$IMPORTE" <<'PY'
import json, os, sys
d, importe = sys.argv[1], int(sys.argv[2])
base = json.load(open(d + "/pago-t-frontera.json"))
def esc(n, p): open("%s/%s.json" % (d, n), "w").write(json.dumps(p))
def cop(): return json.loads(json.dumps(base))

# Las tres AUSENCIAS: el mando las nombra una a una antes de tocar la firma.
for clave, nombre in (("enunciado", "neg-sin-enunciado"), ("prueba", "neg-sin-prueba"),
                      ("cabeza", "neg-sin-cabeza")):
    p = cop(); del p[clave]; esc(nombre, p)
# La firma que falta: la cabeza deja de ser verificable, y el mando lo dice por su nombre.
p = cop(); del p["cabeza"]["signature"]; esc("neg-sin-firma", p)
# La ERA: una v4 no firma pmetaRoot, y la version se juzga ANTES que la firma.
p = cop(); p["cabeza"]["formatVersion"] = "0x4"; esc("neg-cabeza-v4", p)
# El NACIDO pinado al seq de la cabeza: una meta nacida despues de la cabeza que la firma es una
# cabeza que miente, y el juez del enlace se para con su nombre ANTES de la prueba.
p = cop(); p["enunciado"]["nacido"] = base["cabeza"]["seq"]; esc("neg-nacido-igual-seq", p)
# EL IMPORTE, que es lo que este sobre afirma y el del cobro no: es entrada publica del AIR, asi
# que la prueba deja de verificar el enunciado que el sobre dice. El texto lo pone winterfell y
# NO se predice: `niega` solo exige el prefijo del mando.
p = cop(); p["enunciado"]["importe"] = hex(importe + 1); esc("neg-importe-mentido", p)

# LA PUERTA: un sabotaje que no cambia un byte no prueba nada. Se comprueba AQUI, antes de gastar
# una corrida del verificador, y se dice CUAL.
crudo = json.dumps(base)
iguales = [f for f in sorted(os.listdir(d))
           if f.startswith("neg-") and open(d + "/" + f).read() == crudo]
if iguales:
    print("ROJO: estos sabotajes NO cambian un byte del positivo: " + " ".join(iguales))
    raise SystemExit(3)
print("siete cuerpos negativos derivados por MUTACION del positivo de la frontera, y los siete DIFIEREN")
PY

niega "$DIR/neg-sin-enunciado.json"    "falta enunciado"                    "sin-enunciado"
niega "$DIR/neg-sin-prueba.json"       "falta prueba o no es cadena 0x"     "sin-prueba"
niega "$DIR/neg-sin-cabeza.json"       "falta cabeza"                       "sin-cabeza"
niega "$DIR/neg-sin-firma.json"        "falta signature"                    "sin-firma"
niega "$DIR/neg-cabeza-v4.json"        "exige una cabeza v5"                "cabeza-v4"
niega "$DIR/neg-nacido-igual-seq.json" "no es anterior a la cabeza de seq"  "nacido-igual-seq"
niega "$DIR/neg-importe-mentido.json"  "pago:"                              "importe-mentido" "no es anterior"

# ---------------------------------------------------------------- GUARDAR Y PUREZA
if [ -n "$GUARDAR" ]; then
  for f in "$DIR"/pago-*.json "$DIR"/neg-*.json "$DIR/aviso.json" "$DIR/credencial.json" \
           "$DIR/retorno.json" "$DIR/credencial-pagador.json"; do
    cp "$f" "$GUARDAR/"
    printf 'BANCO-PAGO|   %-30s %s  %s B\n' "$(basename "$f")" \
      "$(sha256sum "$f" | cut -c1-16)" "$(wc -c < "$f")" >&2
  done
  msg "capturas guardadas en $GUARDAR (de aqui salen los vectores del catalogo pago-*, por MUTACION)"
fi

git status --porcelain | sort > "$DIR/porcelain.post"
SUCIO=$(comm -13 "$DIR/porcelain.base" "$DIR/porcelain.post" | wc -l)
if [ "$SUCIO" -ne 0 ]; then
  comm -13 "$DIR/porcelain.base" "$DIR/porcelain.post" | sed 's/^/BANCO-PAGO|   /' >&2
  fallo "el banco ensucio $SUCIO entradas del arbol: no debe tocarlo"
fi
[ "$ROTOS" = "7" ] || fallo "se esperaban 7 negativos y cayeron $ROTOS"

msg "BANCO-PAGO VERDE: el pagador probo con el nodo vivo y su propia credencial, el kit lo verifico SIN el nodo, la boca se paro en vivo DOS veces -una sin llegar a pedir- y las siete reglas del sobre cayeron EN VIVO"
