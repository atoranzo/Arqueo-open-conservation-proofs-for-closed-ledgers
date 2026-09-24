#!/usr/bin/env bash
# tools/banco_rechazo.sh -- el banco del sobre de rechazo de H4 (RFC-0007 E5, cortes 3b y 4c):
# el PRODUCTOR del sobre.
#
# Demuestra el SOBRE DE RECHAZO como cosa producida y no reunida a mano, con TRES causas sobre el
# MISMO libro: nodo real con --dev -> se abren dos cuentas y se fondean -> una se CONGELA por
# `dev_freeze` (la via delegada real, con la subida del arbol y los dos custodios de la suite) ->
# se custodia la cabeza v5 que ya compromete esa congelacion -> el nodo MUERE -> el nodo produce
# con `--prueba-rechazo` el sobre de AccountFrozen, el de AccountNotFound y, desde el corte 4c,
# el de InsufficientBalance con su prueba de banda -> el verificador en VERDE **sin el nodo** con
# los tres. Y QUINCE negativos de los sobres, uno por regla, mas CUATRO del productor.
#
# ⚠️ Es la PRIMERA congelacion de un banco del arbol (5.A-216): de los quince `.sh` de `tools/`
# -este incluido, y eran catorce cuando nacio- el unico que nombraba `dev_freeze` era `canon.sh`,
# y la unica congelacion viva eran los tests del nodo.
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
# --guardar  copia los TRES sobres positivos, los QUINCE cuerpos negativos, la cabeza y lo que
#            el mando y el nodo DIJERON (`salida-*.txt`: los negativos del sobre y las piezas
#            de la tercera causa), a huella. De esas capturas se derivan por MUTACION los
#            vectores que hagan falta (regla 2 del PROCESO: los vectores jamas se reescriben).
#
# BANCO_RECHAZO_MANIFIESTO (entorno) el MANIFIESTO del que se DERIVAN los textos de los
#            positivos; por defecto, el del arbol. Existe para el corte que trae una causa: su
#            fila aun no esta en el arbol cuando se capturan sus vectores, y el banco corre
#            contra el manifiesto CANDIDATO que ese corte va a escribir.
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
FONDEO=1000 # el saldo de las dos cuentas; el importe de la tercera causa se DERIVA de el
MONTO=$(printf '0x%x' "$FONDEO")
DIR=$(mktemp -d "$HOME/.banco_rechazo.XXXXXX")
PID=""
# Los dos contadores se declaran AQUI, antes de su primer uso: el del productor se
# incrementa mas arriba que donde vivia (`set -u` lo mata, y `bash -n` no lo ve).
PROD=0
limpiar(){
  if [ -n "$PID" ]; then kill -9 "$PID" 2>/dev/null || true; fi
  rm -rf "$DIR"
}
trap limpiar EXIT INT TERM HUP QUIT

# LA BASE DE LA PUREZA, tomada ANTES de nada: el invariante no es <<el arbol esta limpio>> sino
# <<yo no lo ensucie>>, y este banco puede correr con el arbol ya cargado por su propio sello.
git status --porcelain | sort > "$DIR/porcelain.base"

# El texto del POSITIVO, DERIVADO de su unico productor.
MANIFIESTO="${BANCO_RECHAZO_MANIFIESTO:-spec/vectors/rechazo/MANIFIESTO.txt}"
[ -f "$MANIFIESTO" ] || fallo "no existe el MANIFIESTO $MANIFIESTO"
msg "MANIFIESTO de los textos: $MANIFIESTO"
FILA=$(grep '^cuenta-congelada\.json|' "$MANIFIESTO") \
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
anota(){ printf '%s\n' "$2" > "$DIR/salida-$1.txt"; }   # anota <rotulo> <lo que se dijo>

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
FR0=$(campo "$CAB" result.frozenRoot)
msg "nodo vivo y firmando: seq $SEQ0 (frozenRoot de partida ${FR0:0:18}...)"

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
  R=$(rpc dev_fund "{\"index\":\"$(printf '0x%x' "$i")\",\"amount\":\"$MONTO\"}")
  case "$R" in *'"error"'*) fallo "dev_fund fallo sobre la cuenta $i: $R" ;; esac
done
R=$(rpc dev_freeze "{\"index\":\"$(printf '0x%x' "$IDX")\",\"frozen\":true}")
case "$R" in *'"error"'*) fallo "dev_freeze fallo sobre la cuenta $IDX: $R" ;; esac
msg "cuenta $IDX CONGELADA por la via delegada; la $LIBRE queda libre y fondeada"

# La cabeza tiene que ser POSTERIOR a la congelacion, y eso se DERIVA del libro, no del reloj ni
# del seq: el seq crece con los fondeos antes de que la congelacion este en la cabeza (carrera
# cazada por el ENSAYO-538, S538: seq 4 sin la congelacion). Vale la cabeza cuyo frozenRoot ya
# no es el de partida: solo la congelacion mueve el arbol de congelados.
CAB=""
for _ in $(seq 1 60); do
  V=$(rpc zkssl_signedEpochHead '{}' 2>/dev/null || true)
  case "$V" in *'"available":true'*)
    S=$(qnum "$(campo "$V" result.seq)")
    if [ "$S" -gt "$SEQ0" ] && [ "$(campo "$V" result.frozenRoot)" != "$FR0" ]; then CAB="$V"; break; fi ;;
  esac
  sleep 0.5
done
[ -n "$CAB" ] || fallo "no llego una cabeza firmada con la congelacion dentro (seq de partida $SEQ0, frozenRoot de partida $FR0)"
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
  *"no hay sobre que producir"*) PROD=$((PROD+1)); msg "NEGATIVO productor: exit $RC -- $(echo "$SAL" | tail -n 1)" ;;
  *) fallo "el productor cayo, pero NO por su regla: $SAL" ;;
esac
[ ! -f "$DIR/no-debe-nacer.json" ] || fallo "murio, pero dejo el fichero escrito"

# ---------------------------------------------------------------- LOS CUATRO NEGATIVOS DEL SOBRE
ROTOS=0
niega(){ # niega <fichero> <fragmento esperado> <rotulo>
  local f="$1" frag="$2" rot="$3" s r
  set +e; s=$("$VER" "$f" 2>&1); r=$?; set -e
  anota "neg-$rot" "$s"
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



# ---------------------------------------------------------------- LA SEGUNDA CAUSA: AccountNotFound
# El nodo ya esta MUERTO y el libro es el mismo: la causa nueva no cuesta un nodo mas. El indice
# SIN cuenta se DERIVA de una identidad que este banco NO abre, y su prueba de vida es que el
# productor rehusa si esa cuenta existe (el negativo de mas abajo).
FILA2=$(grep '^cuenta-inexistente\.json|' "$MANIFIESTO") \
  || fallo "el MANIFIESTO del rechazo no tiene fila para cuenta-inexistente.json"
ESPERADO2=$(printf '%s' "$FILA2" | cut -d'|' -f3)
[ -n "$ESPERADO2" ] || fallo "la fila de cuenta-inexistente.json no trae texto esperado"
msg "el VERDE de la segunda causa, DERIVADO del MANIFIESTO: <<$ESPERADO2>>"

SINCUENTA=$(python3 -c 'import sys; print(int(sys.argv[1], 16))' 0x45ff0001)
[ "$SINCUENTA" != "$IDX" ] && [ "$SINCUENTA" != "$LIBRE" ] \
  || fallo "el indice sin cuenta coincide con una de las abiertas: no discriminaria"

SOBRE2="$DIR/rechazo-cuenta-no-existe.json"
set +e
SAL=$("$NODO" --ledger "$DIR/ledger" --log warn \
        --prueba-rechazo "$SOBRE2" --rechazo-cabeza "$DIR/cabeza.json" \
        --rechazo-causa AccountNotFound --rechazo-cuenta "$SINCUENTA" 2>&1)
RC=$?
set -e
[ "$RC" = "0" ] || fallo "el nodo no produjo el sobre de AccountNotFound (exit $RC): $SAL"
echo "$SAL" | sed 's/^/BANCO-RECHAZO|   /' >&2

set +e; SAL=$("$VER" "$SOBRE2" 2>&1); RC=$?; set -e
[ "$RC" = "0" ] || fallo "el sobre de AccountNotFound dio exit $RC (se esperaba 0): $SAL"
case "$SAL" in
  *"$ESPERADO2"*) : ;;
  *) fallo "el mando dio VERDE pero NO con el texto que el MANIFIESTO declara: $SAL" ;;
esac
echo "$SAL" | sed 's/^/BANCO-RECHAZO|   /' >&2
msg "POSITIVO 2: exit 0 y el texto del MANIFIESTO, con el nodo MUERTO"

# El negativo del PRODUCTOR de esta causa: pedirle AccountNotFound sobre la cuenta CONGELADA.
# La causa que sale es AccountFrozen -las guardas van en ORDEN- y tiene que rehusar NOMBRANDOLA
# (PRECISION 410). Un productor que se fiara del orden pasaria por aqui sin decir nada.
set +e
SAL=$("$NODO" --ledger "$DIR/ledger" --log warn \
        --prueba-rechazo "$DIR/tampoco-debe-nacer.json" --rechazo-cabeza "$DIR/cabeza.json" \
        --rechazo-causa AccountNotFound --rechazo-cuenta "$IDX" 2>&1)
RC=$?
set -e
[ "$RC" != "0" ] || fallo "el productor escribio un sobre de AccountNotFound para una cuenta que existe"
case "$SAL" in
  *"no la pedida"*) PROD=$((PROD+1)); msg "NEGATIVO productor 2: exit $RC -- $(echo "$SAL" | tail -n 1)" ;;
  *) fallo "el productor cayo, pero NO por su regla: $SAL" ;;
esac
[ ! -f "$DIR/tampoco-debe-nacer.json" ] || fallo "murio, pero dejo el fichero escrito"

# TRES negativos del sobre, uno por REGLA. No hay un cuarto a proposito: mover un hermano del
# camino cae por la MISMA guarda que mover la hoja, y un falsador que no discrimina no prueba
# nada (asiento 476, D-2).
python3 - "$DIR" <<'PY'
import json, sys
d = sys.argv[1]
base = json.load(open(d + "/rechazo-cuenta-no-existe.json"))
def esc(n, p): open("%s/%s.json" % (d, n), "w").write(json.dumps(p))
def cop(): return json.loads(json.dumps(base))
p = cop(); p["data"]["campos"]["index"] = hex(int(base["data"]["campos"]["index"], 16) + 1)
esc("neg2-index-movido", p)
p = cop(); p["data"]["seq"] = hex(int(base["data"]["seq"], 16) + 1); esc("neg2-seq-movido", p)
p = cop(); p["cuenta"]["leaf"] = base["cabeza"]["accountsRoot"]; esc("neg2-hoja-otra", p)
crudo = json.dumps(base)
import os
iguales = [f for f in sorted(os.listdir(d))
           if f.startswith("neg2-") and open(d + "/" + f).read() == crudo]
if iguales:
    print("ROJO: estos sabotajes NO cambian un byte del positivo: " + " ".join(iguales))
    raise SystemExit(3)
print("tres cuerpos negativos derivados por MUTACION, y los tres DIFIEREN del positivo")
PY

# Los tres fragmentos estan LEIDOS de `verificar_rechazo` en crates/zk-ssl-verify/src/main.rs.
niega "$DIR/neg2-index-movido.json"  "no es el del camino"          "2-index-movido"
niega "$DIR/neg2-seq-movido.json"    "no es la del rechazo"         "2-seq-movido"
niega "$DIR/neg2-hoja-otra.json"     "NO sube al accountsRoot"      "2-hoja-otra"

# ------------------------------------------------------------ LA TERCERA CAUSA: InsufficientBalance
# RFC-0007 E5, corte 4c. El libro y la cabeza son los MISMOS: la cuenta libre tiene el saldo que
# este banco le dio (FONDEO) y ningun movimiento mas, asi que el importe que la rechaza se DERIVA
# de el -el borde exacto, FONDEO + 1- y la banda que se prueba es [0, FONDEO]. Es la primera
# corrida de `prueba_de_banda` sobre el libro de un nodo real: su puerta compara el seq y el
# accountsRoot de la cabeza con los del libro, y aqui se falsa en vivo.
FILA3=$(grep '^saldo-insuficiente\.json|' "$MANIFIESTO") \
  || fallo "el MANIFIESTO del rechazo no tiene fila para saldo-insuficiente.json"
ESPERADO3=$(printf '%s' "$FILA3" | cut -d'|' -f3)
[ -n "$ESPERADO3" ] || fallo "la fila de saldo-insuficiente.json no trae texto esperado"
msg "el VERDE de la tercera causa, DERIVADO del MANIFIESTO: <<$ESPERADO3>>"

IMPORTE3=$((FONDEO + 1))
# El techo del campo se LEE del AIR, su unico productor; no se teclea.
TECHO=$(sed -n 's/^pub const MAX_VALOR: u64 = \(0x[0-9a-f]*\);$/\1/p' \
          crates/zk-ssl-air/src/banda.rs)
[ -n "$TECHO" ] || fallo "no se leyo MAX_VALOR de crates/zk-ssl-air/src/banda.rs"
SOBRA=$(python3 -c 'import sys; print(int(sys.argv[1], 16) + 2)' "$TECHO")
msg "importe DERIVADO $IMPORTE3 (fondeo $FONDEO + 1); techo $TECHO; por encima del techo: $SOBRA"

SOBRE3="$DIR/rechazo-saldo-insuficiente.json"
set +e
SAL=$("$NODO" --ledger "$DIR/ledger" --log warn \
        --prueba-rechazo "$SOBRE3" --rechazo-cabeza "$DIR/cabeza.json" \
        --rechazo-causa InsufficientBalance --rechazo-cuenta "$LIBRE" \
        --rechazo-importe "$IMPORTE3" 2>&1)
RC=$?
set -e
anota productor-3 "$SAL"
[ "$RC" = "0" ] || fallo "el nodo no produjo el sobre de InsufficientBalance (exit $RC): $SAL"
echo "$SAL" | sed 's/^/BANCO-RECHAZO|   /' >&2

set +e; SAL=$("$VER" "$SOBRE3" 2>&1); RC=$?; set -e
anota positivo-3 "$SAL"
[ "$RC" = "0" ] || fallo "el sobre de InsufficientBalance dio exit $RC (se esperaba 0): $SAL"
case "$SAL" in
  *"$ESPERADO3"*) : ;;
  *) fallo "el mando dio VERDE pero NO con el texto que el MANIFIESTO declara: $SAL" ;;
esac
echo "$SAL" | sed 's/^/BANCO-RECHAZO|   /' >&2
msg "POSITIVO 3: exit 0 y el texto del MANIFIESTO, con el nodo MUERTO"

# La D-0 del corte 4b, EN VIVO: el `data` del sobre NO trae el saldo (ni la prueba, S538), y el
# importe es el pedido en los dos sitios donde viaja.
python3 - "$SOBRE3" "$IMPORTE3" <<'PY'
import json, sys
p = json.load(open(sys.argv[1]))
c = p["data"]["campos"]
if "available" in c:
    print("ROJO: el sobre de InsufficientBalance publica available (D-0 del corte 4b)")
    raise SystemExit(3)
pedido = int(sys.argv[2])
for sitio, v in (("data.campos.requested", c.get("requested")),
                 ("banda.requested", p.get("banda", {}).get("requested"))):
    if v is None or int(v, 16) != pedido:
        print("ROJO: %s es %r y el importe pedido era %d" % (sitio, v, pedido))
        raise SystemExit(3)
print("el sobre no lleva el saldo, y pide %d en data y en banda" % pedido)
PY

# Los DOS negativos del PRODUCTOR de esta causa.
# El BORDE: pedir EXACTAMENTE el saldo es legitimo, y no tiene que nacer sobre.
set +e
SAL=$("$NODO" --ledger "$DIR/ledger" --log warn \
        --prueba-rechazo "$DIR/borde-no-debe-nacer.json" --rechazo-cabeza "$DIR/cabeza.json" \
        --rechazo-causa InsufficientBalance --rechazo-cuenta "$LIBRE" \
        --rechazo-importe "$FONDEO" 2>&1)
RC=$?
set -e
anota productor-3-borde "$SAL"
[ "$RC" != "0" ] || fallo "el productor escribio un sobre pidiendo EXACTAMENTE el saldo: $FONDEO"
case "$SAL" in
  *"no hay sobre que producir"*)
    PROD=$((PROD+1)); msg "NEGATIVO productor 3-borde: exit $RC -- $(echo "$SAL" | tail -n 1)" ;;
  *) fallo "el productor cayo en el borde, pero NO por su regla: $SAL" ;;
esac
[ ! -f "$DIR/borde-no-debe-nacer.json" ] || fallo "murio en el borde, pero dejo el fichero escrito"

# El TECHO (la D-1 del corte 4b): por encima de MAX_VALOR la causa SALE y su prueba de banda NO
# existe; el productor tiene que rehusar NOMBRANDOLO, no escribir un sobre muerto.
set +e
SAL=$("$NODO" --ledger "$DIR/ledger" --log warn \
        --prueba-rechazo "$DIR/techo-no-debe-nacer.json" --rechazo-cabeza "$DIR/cabeza.json" \
        --rechazo-causa InsufficientBalance --rechazo-cuenta "$LIBRE" \
        --rechazo-importe "$SOBRA" 2>&1)
RC=$?
set -e
anota productor-3-techo "$SAL"
[ "$RC" != "0" ] || fallo "el productor escribio un sobre con un importe sobre el techo: $SOBRA"
case "$SAL" in
  *"pasa el techo del campo"*)
    PROD=$((PROD+1)); msg "NEGATIVO productor 3-techo: exit $RC -- $(echo "$SAL" | tail -n 1)" ;;
  *) fallo "el productor cayo por encima del techo, pero NO por su regla: $SAL" ;;
esac
[ ! -f "$DIR/techo-no-debe-nacer.json" ] || fallo "murio en el techo, pero dejo el fichero escrito"

# OCHO negativos del sobre, uno por REGLA del decimo brazo del mando. Seis son UNA mutacion; dos
# son ESCENAS de dos campos -el pedido cero y el techo-, porque una sola mutacion no llega a su
# regla (tocar un `requested` solo cae antes, por el importe distinto), y se declaran.
# Sin vector, y a proposito: una prueba corrupta cae por la MISMA regla que la cuenta otra, y una
# cabeza con otro accountsRoot y el mismo seq no se fabrica sin romper la firma.
python3 - "$DIR" "$TECHO" <<'PY'
import json, os, sys
d = sys.argv[1]
base = json.load(open(d + "/rechazo-saldo-insuficiente.json"))
def esc(n, p): open("%s/%s.json" % (d, n), "w").write(json.dumps(p))
def cop(): return json.loads(json.dumps(base))
pedido = base["data"]["campos"]["requested"]
# UNA mutacion cada uno; cada sustituto es un valor que YA viaja en el sobre.
p = cop(); p["data"]["campos"]["available"] = pedido; esc("neg3-con-saldo", p)
p = cop(); p["data"]["campos"]["requested"] = hex(int(pedido, 16) + 1); esc("neg3-importe-otro", p)
p = cop(); p["data"]["seq"] = hex(int(base["data"]["seq"], 16) + 1); esc("neg3-seq-movido", p)
p = cop(); p["banda"]["publicId"] = base["cabeza"]["accountsRoot"]; esc("neg3-cuenta-otra", p)
p = cop(); del p["banda"]["prueba"]; esc("neg3-sin-prueba", p)
p = cop(); p["cabeza"]["chainDigest"] = base["cabeza"]["accountsRoot"]
esc("neg3-cabeza-adulterada", p)
# Las dos ESCENAS: los dos `requested` a la vez.
p = cop(); p["data"]["campos"]["requested"] = "0x0"; p["banda"]["requested"] = "0x0"
esc("neg3-pedido-cero", p)
t = hex(int(sys.argv[2], 16) + 2)
p = cop(); p["data"]["campos"]["requested"] = t; p["banda"]["requested"] = t
esc("neg3-techo", p)
crudo = json.dumps(base)
iguales = [f for f in sorted(os.listdir(d))
           if f.startswith("neg3-") and open(d + "/" + f).read() == crudo]
if iguales:
    print("ROJO: estos sabotajes NO cambian un byte del positivo: " + " ".join(iguales))
    raise SystemExit(3)
n = len([f for f in os.listdir(d) if f.startswith("neg3-")])
if n != 8:
    print("ROJO: se esperaban 8 cuerpos neg3 y hay %d" % n)
    raise SystemExit(3)
print("ocho cuerpos negativos: seis por UNA mutacion y dos escenas, y los ocho DIFIEREN")
PY

# El rojo de la cuenta otra lo escribe el verificador STARK, no la casa: su texto no se conoce de
# antemano y NO se teclea. Lo que se exige es que salga por la verificacion de la banda y por
# NINGUNA de las reglas propias que llevan el mismo prefijo (leidas de `verificar_rechazo` y de
# `zk_ssl_air::banda`). Lo que dijo queda en su `salida-*.txt`.
niega_stark(){ # niega_stark <fichero> <rotulo>
  local f="$1" rot="$2" s r
  set +e; s=$("$VER" "$f" 2>&1); r=$?; set -e
  anota "neg-$rot" "$s"
  [ "$r" = "1" ] || fallo "NEGATIVO $rot dio exit $r (se esperaba 1): $s"
  case "$s" in
    *"banda: falta"*|*"pasan del techo"*|*"banda vacia"*|*"no se deserializa"*|*"forma de traza"*)
      fallo "NEGATIVO $rot cayo por una regla de la CASA, no por la verificacion: $s" ;;
    *"banda: "*)
      msg "NEGATIVO $rot: exit 1 -- $(echo "$s" | tail -n 1)"; ROTOS=$((ROTOS+1)) ;;
    *) fallo "NEGATIVO $rot cayo, pero NO por la verificacion de la banda: $s" ;;
  esac
}

# Los fragmentos estan LEIDOS de `verificar_rechazo` y de `zk_ssl_air::banda`.
niega "$DIR/neg3-con-saldo.json"         "esta causa no publica el saldo"         "3-con-saldo"
niega "$DIR/neg3-importe-otro.json"      "no es el que la prueba acota"           "3-importe-otro"
niega "$DIR/neg3-seq-movido.json"        "no es la del rechazo"                   "3-seq-movido"
niega_stark "$DIR/neg3-cuenta-otra.json"                                          "3-cuenta-otra"
niega "$DIR/neg3-sin-prueba.json"        "banda: falta prueba o no es cadena 0x"  "3-sin-prueba"
niega "$DIR/neg3-cabeza-adulterada.json" "NO recomponen su epochDigest"           "3-cabeza-adult"
niega "$DIR/neg3-pedido-cero.json"       "pedir 0 no puede pasar de ningun saldo" "3-pedido-cero"
niega "$DIR/neg3-techo.json"             "pasan del techo"                        "3-techo"

# ---------------------------------------------------------------- GUARDAR Y PUREZA
if [ -n "$GUARDAR" ]; then
  for f in "$SOBRE" "$SOBRE2" "$SOBRE3" "$DIR"/neg*.json "$DIR/cabeza.json" \
           "$DIR"/salida-*.txt; do
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
[ "$ROTOS" = "15" ] || fallo "se esperaban 15 negativos del sobre y cayeron $ROTOS"
[ "$PROD" = "4" ] || fallo "se esperaban 4 negativos del productor y cayeron $PROD"

msg "BANCO-RECHAZO VERDE: el nodo PRODUJO el sobre sobre un libro real, el kit lo verifico SIN el nodo con el texto del MANIFIESTO, y $((ROTOS + PROD)) reglas cayeron EN VIVO"
