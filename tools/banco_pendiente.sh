#!/usr/bin/env bash
# tools/banco_pendiente.sh -- el banco de E4 del RFC-0008, lado del COBRO (corte B; D-R..D-W).
#
# Demuestra la PRUEBA PORTABLE DEL COBRO PENDIENTE como servicio, de punta a punta: un libro con
# un pendiente v2 VIVO -puesto por el sandbox del cli con el nodo PARADO, con prueba STARK real y
# por la via v2, que es la que lleva el sobre `X`- -> nodo real que lo abre, lo mete en la FOTO
# de su latido y FIRMA una cabeza v5 -> la BOCA del cobrador (`zk-ssl-cli prueba-cobro`) pide la
# cabeza y la foto con el nodo VIVO, prueba en el cliente y escribe el sobre `cobro_pendiente`
# (spec/PAQUETE.md 2.8) -> el nodo MUERE -> el verificador en VERDE **sin el nodo**. Dos positivos,
# que son las dos formas de D-N; un rechazo de la boca EN VIVO; y SIETE negativos sobre el sobre,
# uno por regla (D-Q).
#
# ⚠️ EL ORDEN ES EL INVERSO DEL HERMANO (`tools/banco_edad.sh`): la edad produce con el nodo
# muerto, porque la capa abre el libro y prueba sola. El cobro NO puede: la foto de los pendientes
# vive en el latido, en memoria del nodo, y `zkssl_pendingPath` la sirve solo con el nodo vivo
# (D-F). Y `sled` abre el libro en EXCLUSIVA, asi que la siembra va ANTES de arrancar el nodo, y
# nada de lo que sigue vuelve a abrir el libro.
#
# ⚠️ Por que el cli siembra y no el cable: ningun metodo publica un pendiente sin una prueba de
# CLIENTE. `simulate --ledger --no-claim --v2` deja el envio EN VUELO sobre un libro persistido y
# escribe lo que el cobrador se lleva en DOS ficheros de dos duenos: el aviso (del pagador) y la
# credencial (suya) (D-P). Sus parametros por defecto son los del nodo -los dos abren con las
# raices de `tests_support` y los mismos tres valores-; si no lo fueran, `SovereignLayer::open`
# daria `ParameterMismatch` y este banco moriria ahi (5.A, dos productores sin atado).
#
# ⚠️ LA VENTANA DEL LATIDO: la boca pide la cabeza y DESPUES la foto, y exige que sean del MISMO
# latido; si cae uno entre las dos llamadas, muere con su texto y NO reintenta. Aqui se reintenta
# (D-T): una corrida que muriera por ahi no distinguiria una carrera de un fallo real.
#
# FUERA del canon: levanta procesos y espera latidos. Compila en RELEASE porque aqui se firma y se
# prueba de verdad.
#
# NO ESCRIBE EN EL ARBOL: todo lo suyo vive en un temporal bajo $HOME, que borra al salir. Lo
# comprueba al final por `git status --porcelain`, por DELTA.
#
#   bash tools/banco_pendiente.sh [--guardar <dir>]        (cuando ya vive en el arbol)
#   cd ~/zk-ssl-real && bash <ruta-suelta> [--guardar <dir>]   (mientras vive en Downloads)
#
# --guardar  copia a <dir>, con su huella: los DOS sobres positivos, los SIETE cuerpos negativos,
#            el aviso y la credencial (sin ellos una captura no se puede volver a producir). NO
#            guarda la cabeza aparte: el sobre la lleva VERBATIM (D-J), y una segunda copia serian
#            dos fuentes del mismo dato. De estas capturas se derivan los vectores del catalogo
#            (regla 2 del PROCESO: los vectores jamas se reescriben, se derivan de capturas reales).
#
# ⚠️ Los fragmentos de rechazo de abajo estan LEIDOS del fuente del mando, del juez del AIR y del
# productor de la capa, salvo el de la prueba que no verifica, que lo pone winterfell y no se
# predice: por eso `niega` exige el fragmento Y dice <<cayo, pero NO por su regla>> cuando el rojo
# es de otro. El instrumento mide; no adivina.
set -euo pipefail
msg(){ printf 'BANCO-PENDIENTE| %s\n' "$*" >&2; }
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
  *) fallo "uso: bash tools/banco_pendiente.sh [--guardar <dir>]" ;;
esac

PORT=8611   # libre: censados en tools/*.sh estan 8593, 8594 (PROXY), 8597, 8598, 8599, 8600,
            # 8601, 8603, 8605, 8607, 8609 y 8613
LATIDO=3    # corto: el primer latido cae tras LATIDO segundos, y la carrera se reintenta (D-T)
DIR=$(mktemp -d "$HOME/.banco_pendiente.XXXXXX")
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
# UNA sola corrida del sandbox (D-S): los dos positivos son dos `--inferior` sobre EL MISMO
# pendiente, y una corrida evita la trampa de las posiciones logicas del arbol disperso (el
# hermano la paga subiendo `--accounts` en cada corrida). El importe NO se teclea aqui: se lee
# del aviso que el sandbox escribio, que es la unica fuente.
msg "sandbox: un envio v2 EN VUELO sobre el libro, con el aviso y la credencial en dos ficheros"
"$CLI" --log warn simulate --ledger "$DIR/ledger" --no-claim --v2 \
  --aviso "$DIR/aviso.json" --credencial "$DIR/credencial.json" \
  >"$DIR/sim.txt" 2>&1 || { sed 's/^/BANCO-PENDIENTE|   /' "$DIR/sim.txt" >&2; fallo "el sandbox no dejo el envio v2 en vuelo"; }
[ -s "$DIR/aviso.json" ]      || fallo "el sandbox no escribio el aviso v2"
[ -s "$DIR/credencial.json" ] || fallo "el sandbox no escribio la credencial del receptor"
X=$(qfich "$DIR/aviso.json" x s 2>/dev/null || true)
[ -n "$X" ] || fallo "el aviso no lleva x: es v1, y E1 es del compromiso v2"
IMPORTE=$(qfich "$DIR/aviso.json" amount n)
INDEX=$(qfich "$DIR/credencial.json" index s)
RECEPTOR=$(qfich "$DIR/credencial.json" publicId s)
VIEWKEY=$(qfich "$DIR/credencial.json" viewKey s)
msg "sembrado: importe $IMPORTE (leido del aviso), receptor con index $(qnum "\"$INDEX\"")"

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
[ -n "$CAB" ] || { sed 's/^/BANCO-PENDIENTE|   /' "$DIR/nodo.err" >&2; fallo "no llego una cabeza firmada"; }
FV=$(campo "$CAB" result.formatVersion)
SEQ=$(campo "$CAB" result.seq)
NP=$(campo "$CAB" result.nextPending)
[ "$(qnum "$FV")" = "5" ] || [ "$(qnum "$FV")" = "6" ] \
  || fallo "la cabeza dice formatVersion $(qnum "$FV"): el cobro pendiente exige v5 o v6, las que firman pmetaRoot"
[ "$(qnum "$NP")" -ge 1 ] \
  || fallo "la cabeza firma nextPending $(qnum "$NP"): el sandbox no dejo la posicion viva"
msg "cabeza v5 firmada: seq $(qnum "$SEQ") - nextPending $(qnum "$NP"); el nodo sigue VIVO para la boca"

# ---------------------------------------------------------------- LA BOCA, CON EL NODO VIVO
# `boca` corre la boca del cobrador y REINTENTA solo ante la carrera del latido (D-T): el texto
# es el de la boca, leido de `cobro.rs`. Deja la salida en BOCA_S y el rc en BOCA_R, y quien la
# llama decide que esperaba; asi el rechazo en vivo de abajo pasa por la misma ventana.
BOCA_S=""; BOCA_R=""
boca(){ # boca <inferior> <salida> <rotulo>
  local inf="$1" out="$2" rot="$3" i t0 t1
  for i in 1 2 3 4 5; do
    t0=$(date +%s)
    set +e
    BOCA_S=$("$CLI" --log warn prueba-cobro --nodo "http://127.0.0.1:$PORT" \
      --aviso "$DIR/aviso.json" --index "$INDEX" --receptor "$RECEPTOR" --view-key "$VIEWKEY" \
      --inferior "$inf" --salida "$out" 2>&1)
    BOCA_R=$?
    set -e
    t1=$(date +%s)
    case "$BOCA_S" in
      *"cayo un latido entre las dos llamadas"*)
        msg "carrera del latido en $rot (intento $i de 5): se vuelve a pedir"; sleep 1; continue ;;
    esac
    msg "boca $rot: exit $BOCA_R en $((t1 - t0)) s"
    echo "$BOCA_S" | sed 's/^/BANCO-PENDIENTE|   /' >&2
    return 0
  done
  fallo "la boca cayo CINCO veces seguidas por la ventana del latido en $rot: eso ya no es una carrera"
}

# Los DOS positivos, que son las dos formas de D-N: la existencia (`inferior = 0`) y la banda
# ajustada (`inferior = importe`).
boca 0 "$DIR/cobro-inferior-0.json" "inferior-0"
[ "$BOCA_R" = "0" ] && [ -s "$DIR/cobro-inferior-0.json" ] \
  || fallo "la boca no escribio el sobre con inferior 0 (exit $BOCA_R)"
boca "$IMPORTE" "$DIR/cobro-inferior-importe.json" "inferior-importe"
[ "$BOCA_R" = "0" ] && [ -s "$DIR/cobro-inferior-importe.json" ] \
  || fallo "la boca no escribio el sobre con inferior = importe (exit $BOCA_R)"

# El rechazo EN VIVO: con `inferior = importe + 1` la banda no se sostiene y el productor de la
# capa se para por su nombre (leido de `prueba_cobro.rs`); la boca no escribe nada.
boca $((IMPORTE + 1)) "$DIR/no-debe-existir.json" "inferior-por-encima"
[ "$BOCA_R" != "0" ] \
  || fallo "la boca ESCRIBIO un sobre con inferior por encima del importe: la banda no se sostiene y aun asi probo"
case "$BOCA_S" in
  *"la banda NO se sostiene"*) msg "RECHAZO EN VIVO: la boca se paro por su regla con inferior $((IMPORTE + 1))" ;;
  *) fallo "la boca cayo con inferior importe+1, pero NO por su regla: se esperaba <<la banda NO se sostiene>>" ;;
esac
[ ! -e "$DIR/no-debe-existir.json" ] || fallo "la boca dejo un fichero al rechazar: un sobre que no prueba no se escribe"

kill -9 "$PID"; wait "$PID" 2>/dev/null || true
PID=""
msg "el nodo esta MUERTO; a partir de aqui NADA necesita red ni libro"

# ---------------------------------------------------------------- EL MANDO, SIN EL NODO
for f in cobro-inferior-0 cobro-inferior-importe; do
  set +e; SAL=$("$VER" "$DIR/$f.json" 2>&1); RC=$?; set -e
  [ "$RC" = "0" ] || fallo "el sobre POSITIVO $f dio exit $RC (se esperaba 0): $SAL"
  echo "$SAL" | sed 's/^/BANCO-PENDIENTE|   /' >&2
done
# Lo que los dos sobres afirman se LEE de ellos, no se supone: el mismo receptor, el mismo nacido,
# y las dos cotas que la boca recibio.
python3 - "$DIR" "$IMPORTE" "$RECEPTOR" <<'PY'
import json, sys
d, importe, receptor = sys.argv[1], int(sys.argv[2]), sys.argv[3]
a = json.load(open(d + "/cobro-inferior-0.json"))
b = json.load(open(d + "/cobro-inferior-importe.json"))
for s, n in ((a, "inferior-0"), (b, "inferior-importe")):
    assert s["tipo"] == "cobro_pendiente" and s["v"] == 1, n
    assert s["enunciado"]["receptor"] == receptor, "%s: el receptor del sobre no es el de la credencial" % n
    assert int(s["enunciado"]["nacido"], 16) < int(s["cabeza"]["seq"], 16), "%s: nacido >= seq" % n
assert int(a["enunciado"]["inferior"], 16) == 0, "inferior-0 no dice 0"
assert int(b["enunciado"]["inferior"], 16) == importe, "inferior-importe no dice el importe"
print("los dos sobres afirman lo pedido: receptor de la credencial, nacido %d < seq %d, cotas 0 y %d; prueba %d B y %d B"
      % (int(a["enunciado"]["nacido"], 16), int(a["cabeza"]["seq"], 16), importe,
         (len(a["prueba"]) - 2) // 2, (len(b["prueba"]) - 2) // 2))
PY
msg "POSITIVOS: exit 0 los dos, sin el nodo"

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
base = json.load(open(d + "/cobro-inferior-importe.json"))
def esc(n, p): open("%s/%s.json" % (d, n), "w").write(json.dumps(p))
def cop(): return json.loads(json.dumps(base))

# Las tres AUSENCIAS: el mando las nombra una a una antes de tocar la firma, con los textos de la
# edad letra por letra (spec/PAQUETE.md, el catalogo del cobro).
for clave, nombre in (("enunciado", "neg-sin-enunciado"), ("prueba", "neg-sin-prueba"),
                      ("cabeza", "neg-sin-cabeza")):
    p = cop(); del p[clave]; esc(nombre, p)
# La firma que falta: la cabeza deja de ser verificable, y el mando lo dice por su nombre.
p = cop(); del p["cabeza"]["signature"]; esc("neg-sin-firma", p)
# La ERA: una v4 no firma pmetaRoot, y la version se juzga ANTES que la firma.
p = cop(); p["cabeza"]["formatVersion"] = "0x4"; esc("neg-cabeza-v4", p)
# El NACIDO pinado entero al seq de la cabeza: una meta nacida despues de la cabeza que la firma
# es una cabeza que miente, y el juez del enlace se para con su nombre ANTES de la prueba.
p = cop(); p["enunciado"]["nacido"] = base["cabeza"]["seq"]; esc("neg-nacido-igual-seq", p)
# La regla del JUEZ, una sola (D-Q): la cota por encima del importe. `inferior` es entrada
# publica del AIR (una asercion de frontera), asi que la prueba deja de verificar el enunciado
# que el sobre afirma. El texto lo pone winterfell y NO se predice: `niega` solo exige el prefijo.
p = cop(); p["enunciado"]["inferior"] = hex(importe + 1); esc("neg-inferior-por-encima", p)

# LA PUERTA: un sabotaje que no cambia un byte no prueba nada. Se comprueba AQUI, antes de gastar
# una corrida del verificador, y se dice CUAL.
crudo = json.dumps(base)
iguales = [f for f in sorted(os.listdir(d))
           if f.startswith("neg-") and open(d + "/" + f).read() == crudo]
if iguales:
    print("ROJO: estos sabotajes NO cambian un byte del positivo: " + " ".join(iguales))
    raise SystemExit(3)
print("siete cuerpos negativos derivados por MUTACION del positivo de la banda, y los siete DIFIEREN")
PY

niega "$DIR/neg-sin-enunciado.json"       "falta enunciado"                       "sin-enunciado"
niega "$DIR/neg-sin-prueba.json"          "falta prueba o no es cadena 0x"        "sin-prueba"
niega "$DIR/neg-sin-cabeza.json"          "falta cabeza"                          "sin-cabeza"
niega "$DIR/neg-sin-firma.json"           "falta signature"                       "sin-firma"
niega "$DIR/neg-cabeza-v4.json"           "exige una cabeza v5"                   "cabeza-v4"
niega "$DIR/neg-nacido-igual-seq.json"    "no es anterior a la cabeza de seq"     "nacido-igual-seq"
niega "$DIR/neg-inferior-por-encima.json" "cobro:"                                "inferior-por-encima" "no es anterior"

# ---------------------------------------------------------------- GUARDAR Y PUREZA
if [ -n "$GUARDAR" ]; then
  for f in "$DIR"/cobro-*.json "$DIR"/neg-*.json "$DIR/aviso.json" "$DIR/credencial.json"; do
    cp "$f" "$GUARDAR/"
    printf 'BANCO-PENDIENTE|   %-30s %s  %s B\n' "$(basename "$f")" \
      "$(sha256sum "$f" | cut -c1-16)" "$(wc -c < "$f")" >&2
  done
  msg "capturas guardadas en $GUARDAR (de aqui salen los vectores, por MUTACION)"
fi

git status --porcelain | sort > "$DIR/porcelain.post"
SUCIO=$(comm -13 "$DIR/porcelain.base" "$DIR/porcelain.post" | wc -l)
if [ "$SUCIO" -ne 0 ]; then
  comm -13 "$DIR/porcelain.base" "$DIR/porcelain.post" | sed 's/^/BANCO-PENDIENTE|   /' >&2
  fallo "el banco ensucio $SUCIO entradas del arbol: no debe tocarlo"
fi
[ "$ROTOS" = "7" ] || fallo "se esperaban 7 negativos y cayeron $ROTOS"

msg "BANCO-PENDIENTE VERDE: el cobrador probo con el nodo vivo, el kit lo verifico SIN el nodo, la boca se paro en vivo por su regla y las siete reglas del sobre cayeron EN VIVO"
