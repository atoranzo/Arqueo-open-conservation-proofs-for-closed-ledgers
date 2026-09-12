#!/usr/bin/env bash
# tools/banco_rechazo.sh -- el banco del corte 3a de H4 (RFC-0007 E5): el PRODUCTOR del sobre.
#
# Demuestra el SOBRE DE RECHAZO como cosa producida y no reunida a mano: nodo real con --dev ->
# se abre una cuenta y se fondea -> se CONGELA por `dev_freeze` (la via delegada real, con la
# subida del arbol y los dos custodios de la suite) -> se custodia la cabeza v5 que ya compromete
# esa congelacion -> el nodo MUERE -> el nodo produce el sobre con `--prueba-rechazo` -> el
# verificador en VERDE **sin el nodo**. Y CINCO negativos: cuatro del sobre y uno del productor.
#
# ⚠️ Es la PRIMERA congelacion de un banco del arbol (5.A-216): de los catorce `.sh` de `tools/`,
# el unico que nombraba `dev_freeze` era `canon.sh`, y la unica congelacion viva eran los tests
# del nodo.
#
# ⚠️ El nodo se para ANTES de producir: `sled` abre el libro en EXCLUSIVA, asi que el modo
# `--prueba-rechazo` no puede correr con el servidor vivo. Es la razon del hermano, palabra por
# palabra.
#
# FUERA del canon: levanta procesos y espera latidos. Hermano de `tools/banco_edad.sh`, del que
# copia su forma; compila en RELEASE porque aqui se firma de verdad.
#
# NO ESCRIBE EN EL ARBOL: todo lo suyo vive en un temporal bajo $HOME, que borra al salir. Lo
# comprueba al final por DELTA sobre `git status --porcelain`, no en absoluto (PRECISION 169).
#
#   bash tools/banco_rechazo.sh [--guardar <dir>]
#   cd ~/zk-ssl-real && bash <ruta-suelta> [--guardar <dir>]   (mientras vive en Downloads)
#
# --guardar  copia el sobre positivo, los cuatro cuerpos negativos y la cabeza a <dir>, con su
#            huella. De esas capturas se derivan por MUTACION los vectores que hagan falta
#            (regla 2 del PROCESO: los vectores jamas se reescriben).
#
# ⚠️ El texto que el POSITIVO tiene que decir NO se teclea aqui: se DERIVA de la fila de
# `spec/vectors/rechazo/MANIFIESTO.txt`, que es su unico productor. Los fragmentos de los
# negativos estan LEIDOS del fuente del mando (`verificar_rechazo`), no adivinados.
set -euo pipefail
msg(){ printf 'BANCO-RECHAZO| %s\n' "$*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }

# ⚠️ LA RAIZ SE DERIVA, no se supone (copiado del hermano): este fichero puede correrse todavia
# SUELTO desde Downloads. Las dos procedencias, en orden, y la elegida trae su PRUEBA DE VIDA.
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
  *) fallo "uso: bash tools/banco_rechazo.sh [--guardar <dir>]" ;;
esac

PORT=8609   # libre: los censados en tools/*.sh son 8600, 8601, 8603, 8605, 8607 y 8613
DIR=$(mktemp -d "$HOME/.banco_rechazo.XXXXXX")
PID=""
limpiar(){
  if [ -n "$PID" ]; then kill -9 "$PID" 2>/dev/null || true; fi
  rm -rf "$DIR"
}
trap limpiar EXIT INT TERM HUP QUIT

# LA BASE DE LA PUREZA, tomada ANTES de nada: el invariante no es <<el arbol esta limpio>> sino
# <<yo no lo ensucie>>, y este banco puede correr con el arbol ya cargado por su propio sello.
git status --porcelain | sort > "$DIR/porcelain.base"

# El texto del POSITIVO, DERIVADO de su unico productor.
FILA=$(grep '^cuenta-congelada\.json|' spec/vectors/rechazo/MANIFIESTO.txt) \
  || fallo "el MANIFIESTO del rechazo no tiene fila para cuenta-congelada.json"
ESPERADO=$(printf '%s' "$FILA" | cut -d'|' -f3)
[ -n "$ESPERADO" ] || fallo "la fila del MANIFIESTO no trae texto esperado"
msg "el VERDE que se exige, DERIVADO del MANIFIESTO: <<$ESPERADO>>"

msg "compilando nodo y verificador en RELEASE (aqui se firma y se congela de verdad)"
cargo build --release -q -p zk-ssl-node -p zk-ssl-verify 2>/dev/null \
  || cargo build --release -p zk-ssl-node -p zk-ssl-verify || fallo "no compila"
NODO=target/release/zk-ssl-node
VER=target/release/zk-ssl-verify
python3 -c "print('41'*96, end='')" > "$DIR/semilla.hex"
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

# ---------------------------------------------------------------- EL NODO, CON --dev
"$NODO" --listen "127.0.0.1:$PORT" --latido 2 --dev \
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
[ -n "$CAB" ] || { sed 's/^/BANCO-RECHAZO|   /' "$DIR/nodo.err" >&2; fallo "no llego una cabeza firmada"; }
SEQ0=$(qnum "$(campo "$CAB" result.seq)")
msg "nodo vivo y firmando: seq $SEQ0"

# ---------------------------------------------------------------- LA CUENTA CONGELADA Y SU PAREJA
abrir(){ # abrir <semilla-hex> -> imprime el indice en decimal
  local r
  r=$(rpc dev_openSeeded "{\"seed\":\"$1\"}")
  case "$r" in *'"index"'*) : ;; *) fallo "dev_openSeeded no abrio la cuenta $1: $r" ;; esac
  qnum "$(campo "$r" result.index)"
}
IDX=$(abrir 0x458d)
LIBRE=$(abrir 0x458e)
[ "$IDX" != "$LIBRE" ] || fallo "las dos cuentas cayeron en la misma posicion: el negativo del productor no discriminaria"
for i in "$IDX" "$LIBRE"; do
  R=$(rpc dev_fund "{\"index\":\"$(printf '0x%x' "$i")\",\"amount\":\"0x3e8\"}")
  case "$R" in *'"error"'*) fallo "dev_fund fallo sobre la cuenta $i: $R" ;; esac
done
R=$(rpc dev_freeze "{\"index\":\"$(printf '0x%x' "$IDX")\",\"frozen\":true}")
case "$R" in *'"error"'*) fallo "dev_freeze fallo sobre la cuenta $IDX: $R" ;; esac
msg "cuenta $IDX CONGELADA por la via delegada; la $LIBRE queda libre y fondeada"

# La cabeza tiene que ser POSTERIOR a la congelacion, y eso se DERIVA del seq, no del reloj:
# cada transicion aplicada lo mueve.
CAB=""
for _ in $(seq 1 60); do
  V=$(rpc zkssl_signedEpochHead '{}' 2>/dev/null || true)
  case "$V" in *'"available":true'*)
    S=$(qnum "$(campo "$V" result.seq)")
    if [ "$S" -gt "$SEQ0" ]; then CAB="$V"; break; fi ;;
  esac
  sleep 0.5
done
[ -n "$CAB" ] || fallo "no llego una cabeza firmada POSTERIOR a la congelacion (seq de partida $SEQ0)"
FV=$(campo "$CAB" result.formatVersion)
SEQ=$(qnum "$(campo "$CAB" result.seq)")
[ "$(qnum "$FV")" = "5" ] || fallo "la cabeza dice formatVersion $(qnum "$FV") y se esperaba v5"
campo "$CAB" result > "$DIR/cabeza.json"
msg "cabeza v5 custodiada DESPUES de congelar: seq $SEQ (era $SEQ0)"

kill -9 "$PID"; wait "$PID" 2>/dev/null || true
PID=""
msg "el nodo esta MUERTO; el libro queda libre y a partir de aqui NADA de lo que sigue lo toca"

# ---------------------------------------------------------------- EL NODO PRODUCE, SIN SERVIDOR
SOBRE="$DIR/rechazo-cuenta-congelada.json"
set +e
SAL=$("$NODO" --ledger "$DIR/ledger" --log warn \
        --prueba-rechazo "$SOBRE" --rechazo-cabeza "$DIR/cabeza.json" \
        --rechazo-causa AccountFrozen --rechazo-cuenta "$IDX" 2>&1)
RC=$?
set -e
[ "$RC" = "0" ] || fallo "el nodo no produjo el sobre (exit $RC): $SAL"
echo "$SAL" | sed 's/^/BANCO-RECHAZO|   /' >&2

set +e; SAL=$("$VER" "$SOBRE" 2>&1); RC=$?; set -e
[ "$RC" = "0" ] || fallo "el sobre POSITIVO dio exit $RC (se esperaba 0): $SAL"
case "$SAL" in
  *"$ESPERADO"*) : ;;
  *) fallo "el mando dio VERDE pero NO con el texto que el MANIFIESTO declara: $SAL" ;;
esac
echo "$SAL" | sed 's/^/BANCO-RECHAZO|   /' >&2
msg "POSITIVO: exit 0 y el texto del MANIFIESTO, con el nodo MUERTO"

# ---------------------------------------------------------------- EL NEGATIVO DEL PRODUCTOR
# Pedir AccountFrozen sobre una cuenta que NO lo esta tiene que morir DICIENDOLO: un productor
# que se fia del orden de las guardas estaria tecleando una posicion (5.A-215).
set +e
SAL=$("$NODO" --ledger "$DIR/ledger" --log warn \
        --prueba-rechazo "$DIR/no-debe-nacer.json" --rechazo-cabeza "$DIR/cabeza.json" \
        --rechazo-causa AccountFrozen --rechazo-cuenta "$LIBRE" 2>&1)
RC=$?
set -e
[ "$RC" != "0" ] || fallo "el productor escribio un sobre para una cuenta que NO esta congelada"
case "$SAL" in
  *"no hay sobre que producir"*) msg "NEGATIVO productor: exit $RC -- $(echo "$SAL" | tail -n 1)" ;;
  *) fallo "el productor cayo, pero NO por su regla: $SAL" ;;
esac
[ ! -f "$DIR/no-debe-nacer.json" ] || fallo "murio, pero dejo el fichero escrito"

# ---------------------------------------------------------------- LOS CUATRO NEGATIVOS DEL SOBRE
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
base = json.load(open(d + "/rechazo-cuenta-congelada.json"))
def esc(n, p): open("%s/%s.json" % (d, n), "w").write(json.dumps(p))
def cop(): return json.loads(json.dumps(base))

# UNA mutacion cada uno, y cada sustituto es un digest CANONICO que ya viaja en el sobre: asi el
# rojo es el de la regla y no el de la FORMA, que es otra (el precedente del hermano).
p = cop(); p["data"]["campos"]["index"] = hex(int(base["data"]["campos"]["index"], 16) + 1)
esc("neg-index-movido", p)
p = cop(); p["data"]["seq"] = hex(int(base["data"]["seq"], 16) + 1); esc("neg-seq-movido", p)
p = cop(); p["congelados"]["leaf"] = base["cabeza"]["frozenRoot"]; esc("neg-hoja-otra", p)
p = cop()
s0 = base["congelados"]["camino"]["siblings"]
if len(s0) < 2:
    print("ROJO: el camino trae %d hermanos: no hay con que mutar sin inventar un digest" % len(s0))
    raise SystemExit(3)
p["congelados"]["camino"]["siblings"][0] = s0[1]
esc("neg-hermano-otro", p)

# LA PUERTA: un sabotaje que no cambia un byte no prueba nada. Se comprueba AQUI, y se dice CUAL.
crudo = json.dumps(base)
iguales = [f for f in sorted(os.listdir(d))
           if f.startswith("neg-") and open(d + "/" + f).read() == crudo]
if iguales:
    print("ROJO: estos sabotajes NO cambian un byte del positivo: " + " ".join(iguales))
    raise SystemExit(3)
print("cuatro cuerpos negativos derivados por MUTACION, y los cuatro DIFIEREN del positivo")
PY

# Los cuatro fragmentos estan LEIDOS de `verificar_rechazo` en crates/zk-ssl-verify/src/main.rs.
niega "$DIR/neg-index-movido.json"  "no es el del camino"        "index-movido"
niega "$DIR/neg-seq-movido.json"    "no es la del rechazo"       "seq-movido"
niega "$DIR/neg-hoja-otra.json"     "NO sube al frozenRoot"      "hoja-otra"
niega "$DIR/neg-hermano-otro.json"  "NO sube al frozenRoot"      "hermano-otro"

# ---------------------------------------------------------------- GUARDAR Y PUREZA
if [ -n "$GUARDAR" ]; then
  for f in "$SOBRE" "$DIR"/neg-*.json "$DIR/cabeza.json"; do
    cp "$f" "$GUARDAR/"
    printf 'BANCO-RECHAZO|   %-32s %s  %s B\n' "$(basename "$f")" \
      "$(sha256sum "$f" | cut -c1-16)" "$(wc -c < "$f")" >&2
  done
  msg "capturas guardadas en $GUARDAR (de aqui salen los vectores, por MUTACION)"
fi

git status --porcelain | sort > "$DIR/porcelain.post"
SUCIO=$(comm -13 "$DIR/porcelain.base" "$DIR/porcelain.post" | wc -l)
if [ "$SUCIO" -ne 0 ]; then
  comm -13 "$DIR/porcelain.base" "$DIR/porcelain.post" | sed 's/^/BANCO-RECHAZO|   /' >&2
  fallo "el banco ensucio $SUCIO entradas del arbol: no debe tocarlo"
fi
[ "$ROTOS" = "4" ] || fallo "se esperaban 4 negativos del sobre y cayeron $ROTOS"

msg "BANCO-RECHAZO VERDE: el nodo PRODUJO el sobre sobre un libro real, el kit lo verifico SIN el nodo con el texto del MANIFIESTO, y cinco reglas cayeron EN VIVO"
