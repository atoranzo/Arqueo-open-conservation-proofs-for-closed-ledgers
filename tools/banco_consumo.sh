#!/usr/bin/env bash
# tools/banco_consumo.sh -- el banco de E3 del RFC-0006 (la prueba portable del consumo).
#
# Demuestra la PRUEBA PORTABLE como servicio: nodo real firmando latidos -> cabeza VIEJA v4
# custodiada -> se PUBLICA un consumo por el cable -> mas latidos -> zkssl_consistencyProof +
# cabeza NUEVA v4 que FIRMA ese tamano -> los DOS caminos por zkssl_consumoPath (la AUSENCIA bajo
# el `seq` de la vieja, la PRESENCIA bajo el de la nueva) -> sobre de consumo -> el verificador en
# VERDE **sin el nodo**. Y SIETE negativos: uno del CABLE (nodo vivo) y SEIS sobre el sobre.
#
# FUERA del canon: levanta procesos y espera latidos. Hermano de `tools/banco_extension.sh`, del
# que copia su forma; se desvia de el en UNA cosa, declarada: **compila en RELEASE**, porque aqui
# se firma de verdad y en depuracion XMSS tarda minutos (regla de la casa).
#
# NO ESCRIBE EN EL ARBOL: todo lo suyo vive en un temporal bajo $HOME, que borra al salir. Lo
# comprueba al final por `git status --porcelain`.
#
#   bash tools/banco_consumo.sh [--guardar <dir>]        (cuando ya vive en el arbol)
#   cd ~/zk-ssl-real && bash <ruta-suelta> [--guardar <dir>]   (mientras vive en Downloads)
#
# --guardar  copia el sobre POSITIVO y los siete cuerpos de los negativos a <dir>, con su huella.
#            De esas capturas se derivan por MUTACION los vectores del catalogo (regla 2 del
#            PROCESO: los vectores jamas se reescriben, se derivan de capturas reales).
set -euo pipefail
msg(){ printf 'BANCO-CONS| %s\n' "$*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }

# ⚠️ LA RAIZ SE DERIVA, no se supone. El hermano hace `cd "$(dirname "$0")/.."` porque vive en
# `tools/`; este fichero puede correrse todavia SUELTO, desde Downloads, y esa linea llevaria a
# un directorio sin `Cargo.toml`. Se prueban las dos procedencias, en orden, y la elegida tiene
# que traer su PRUEBA DE VIDA: `Cargo.toml` Y `crates/zk-ssl-verify`, o no es este arbol.
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
  *) fallo "uso: bash tools/banco_consumo.sh [--guardar <dir>]" ;;
esac

PORT=8598   # el hermano usa el 8597: asi los dos bancos pueden correr a la vez
DIR=$(mktemp -d "$HOME/.banco_consumo.XXXXXX")
PID=""
limpiar(){
  if [ -n "$PID" ]; then kill -9 "$PID" 2>/dev/null || true; fi
  rm -rf "$DIR"
}
trap limpiar EXIT INT TERM HUP QUIT

msg "compilando nodo y verificador en RELEASE (aqui se firma de verdad)"
cargo build --release -q -p zk-ssl-node -p zk-ssl-verify 2>/dev/null \
  || cargo build --release -p zk-ssl-node -p zk-ssl-verify || fallo "no compila"
NODO=target/release/zk-ssl-node
VER=target/release/zk-ssl-verify
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
# OJO: `campo` devuelve el valor YA CODIFICADO en JSON (una cadena viene con sus comillas), asi
# que los `seq` y los `mmrSize` se interpolan a pelo. El CONSUMO lo compone este banco, no viene
# de una respuesta: va ENTRECOMILLADO a mano, o el cuerpo JSON sale invalido.
crudo(){ python3 -c 'import json,sys; print(json.loads(sys.argv[1]))' "$1"; }

# El consumo es un digest de 32 bytes que tiene que ser CANONICO en el campo (PRECISION 109: el
# campo REDUCE, y `digest_from_wire` rechaza lo que no lo sea). Se compone con UN byte no nulo en
# el offset 3 de cada grupo de ocho: leido en LE o en BE, cada limbo sale muy por debajo de p, asi
# que es canonico en las dos lecturas y no hace falta suponer cual usa el cable.
CONSUMO=$(python3 - <<'PY'
b = bytearray(32)
for i, v in ((3, 0xA7), (11, 0x5C), (19, 0x13), (27, 0x91)):
    b[i] = v
print("0x" + b.hex())
PY
)
msg "consumo de la corrida: $CONSUMO"

# ---------------------------------------------------------------- LA CABEZA VIEJA
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
SEQ_V=$(campo "$VIEJA" result.seq)
FV_V=$(campo "$VIEJA" result.formatVersion)
[ "$(qnum "$FV_V")" = "4" ] || fallo "la cabeza vieja dice formatVersion $(qnum "$FV_V"): el sobre de consumo EXIGE v4 (una v2 o v3 no lleva consRoot)"
msg "cabeza VIEJA custodiada: seq $(qnum "$SEQ_V") - mmrSize $(qnum "$OLDSIZE") - v4"

# ---------------------------------------------------------------- SE PUBLICA EL CONSUMO
PUB=$(rpc zkssl_publishConsumo "{\"consumo\":\"$CONSUMO\"}")
case "$PUB" in
  *'"accepted":true'*) : ;;
  *) fallo "publishConsumo no acepto el consumo: $PUB" ;;
esac
LOGSEQ=$(campo "$PUB" result.logSeq)
msg "consumo PUBLICADO: logSeq $(qnum "$LOGSEQ")"

# NEGATIVO 1 (del CABLE, con el nodo VIVO): el mismo consumo otra vez. Un repetido NO es error
# del que llama: se responde `accepted:false` con su razon, y el texto es el de la CAPA.
REP=$(rpc zkssl_publishConsumo "{\"consumo\":\"$CONSUMO\"}")
case "$REP" in
  *'"accepted":false'*) : ;;
  *) fallo "NEGATIVO repetido-en-el-cable: el mismo consumo se acepto DOS veces: $REP" ;;
esac
case "$REP" in
  *"ya esta publicado"*) msg "NEGATIVO repetido-en-el-cable: accepted:false -- $(crudo "$(campo "$REP" result.reason)")" ;;
  *) fallo "NEGATIVO repetido-en-el-cable cayo, pero NO por su regla: se esperaba <<ya esta publicado>> y dijo: $REP" ;;
esac

# ⚠️ EL DISCRIMINANTE ARITMETICO, y es la puerta de una DEDUCCION declarada.
# El `seq` de la cabeza firmada NO sale de `transition_log().len()` en el despacho: sale del DTO,
# o sea de la foto del ultimo latido, y quien lo compone vive en la capa (`epoch_head`). Este
# banco NO lo supone: lo EXIGE por aritmetica. `logSeq` es la longitud del registro DESPUES de
# insertar, luego la vieja tiene que quedar por debajo y la nueva por encima o igual. Si el `seq`
# de la cabeza no fuera un indice de ese mismo registro, esto se pone rojo AQUI, con su nombre, en
# vez de dejar que el mando falle luego por una razon que no es la suya.
[ "$(qnum "$SEQ_V")" -lt "$(qnum "$LOGSEQ")" ] \
  || fallo "el seq de la cabeza vieja ($(qnum "$SEQ_V")) NO queda por debajo del logSeq del consumo ($(qnum "$LOGSEQ")): el seq de la cabeza firmada no es un indice de este registro, y el banco no puede pedir los caminos bajo las cabezas correctas"

# ---------------------------------------------------------------- UN SEGUNDO CONSUMO, A PROPOSITO
# Con UN SOLO consumo en el arbol, el camino de la AUSENCIA bajo la vieja y el de la PRESENCIA
# bajo la nueva son IDENTICOS: los dos son la cadena de subarboles vacios. Lo que separa las dos
# afirmaciones no es el camino, es la HOJA (D-11). Un segundo consumo cambia el hermano de P en el
# nivel donde sus posiciones divergen, y con eso el sobre ejercita la forma REAL.
SEGUNDO=$(python3 - <<'PY'
b = bytearray(32)
for i, v in ((3, 0x4E), (11, 0xB2), (19, 0x08), (27, 0xD5)):
    b[i] = v
print("0x" + b.hex())
PY
)
PUB2=$(rpc zkssl_publishConsumo "{\"consumo\":\"$SEGUNDO\"}")
case "$PUB2" in
  *'"accepted":true'*) msg "segundo consumo publicado (el arbol deja de ser trivial): logSeq $(qnum "$(campo "$PUB2" result.logSeq)")" ;;
  *) fallo "el segundo consumo no se acepto: $PUB2" ;;
esac

# ---------------------------------------------------------------- EL CAMINO Y LA CABEZA NUEVA
sleep 5  # que la historia crezca de verdad

ACK=""
for _ in $(seq 1 20); do
  ACK=$(rpc zkssl_consistencyProof "{\"oldSize\":$OLDSIZE}" || true)
  case "$ACK" in *'"available":true'*) break;; esac
  ACK=""; sleep 0.5
done
[ -n "$ACK" ] || fallo "consistencyProof no llego a available:true"
MA=$(campo "$ACK" result.mmrSize)
# ⚠️ La pareja FIRMADA es el acumulador ANTES de la cabeza (el push va despues del emit): el
# camino de tamano t lo firma LA SIGUIENTE cabeza en emitirse. Se espera hasta que una cabeza
# firme exactamente ese t. Copiado del hermano, donde esta medido.
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
[ -n "$NUEVA" ] || fallo "ninguna cabeza firmo mmrSize $(qnum "$MA") (se esperaba en un latido)"
SEQ_N=$(campo "$NUEVA" result.seq)
FV_N=$(campo "$NUEVA" result.formatVersion)
[ "$(qnum "$FV_N")" = "4" ] || fallo "la cabeza nueva dice formatVersion $(qnum "$FV_N"): el sobre EXIGE v4"
[ "$(qnum "$SEQ_N")" -ge "$(qnum "$LOGSEQ")" ] \
  || fallo "el seq de la cabeza nueva ($(qnum "$SEQ_N")) NO alcanza el logSeq del consumo ($(qnum "$LOGSEQ")): esa cabeza no puede acreditar el consumo"
msg "cabeza NUEVA emparejada: seq $(qnum "$SEQ_N") - mmrSize $(qnum "$MA") - v4"

# ---------------------------------------------------------------- LOS DOS CAMINOS
# La MISMA llamada sirve las dos direcciones: quien verifica elige la HOJA (D-11). La AUSENCIA se
# pide bajo el `seq` de la VIEJA y la PRESENCIA bajo el de la NUEVA; `cons_path` reconstruye el
# arbol tal como lo firmo la cabeza de ese seq, asi que NO hace falta orden temporal.
pide_camino(){ # $1 seq  $2 rotulo
  local R
  R=$(rpc zkssl_consumoPath "{\"consumo\":\"$CONSUMO\",\"seq\":$1}")
  case "$R" in
    *'"available":true'*) echo "$R" ;;
    *) fallo "consumoPath($2) no dio camino: $R" ;;
  esac
}
AUS=$(pide_camino "$SEQ_V" ausencia)
PRE=$(pide_camino "$SEQ_N" presencia)
NIV=$(python3 -c 'import json,sys; print(len(json.loads(sys.argv[1])["result"]["camino"]["siblings"]))' "$AUS")
DIF=$(python3 - "$AUS" "$PRE" <<'PY'
import json, sys
a = json.loads(sys.argv[1])["result"]["camino"]["siblings"]
p = json.loads(sys.argv[2])["result"]["camino"]["siblings"]
print(sum(1 for x, y in zip(a, p) if x != y))
PY
)
msg "los dos caminos servidos: $NIV niveles cada uno; DIFIEREN en $DIF de ellos"
# Se DECLARA, no se exige: si difirieran en CERO el sobre seguiria siendo correcto -lo que separa
# presencia de ausencia es la HOJA-, pero un sabotaje que INTERCAMBIE los dos caminos no
# discriminaria. Por eso los dos negativos de <<no sube>> adulteran un hermano, no intercambian.

kill -9 "$PID"; wait "$PID" 2>/dev/null || true
PID=""
msg "el nodo esta MUERTO; a partir de aqui NADA de lo que sigue lo toca"

# ---------------------------------------------------------------- EL SOBRE
python3 - "$VIEJA" "$NUEVA" "$ACK" "$AUS" "$PRE" "$CONSUMO" "$DIR" <<'PY'
import json, sys
vieja = json.loads(sys.argv[1])["result"]
nueva = json.loads(sys.argv[2])["result"]
ack   = json.loads(sys.argv[3])["result"]
aus   = json.loads(sys.argv[4])["result"]
pre   = json.loads(sys.argv[5])["result"]
consumo, d = sys.argv[6], sys.argv[7]
for c, n in ((vieja, "vieja"), (nueva, "nueva")):
    assert c.get("available") is True, n
assert ack.get("available") is True
# Las respuestas van TAL CUAL: `result.camino` de consumoPath ES el objeto {siblings, isRight}
# que el sobre pide bajo `presencia` y `ausencia`. Reescribir seria adulterar.
p = {"v": 1, "tipo": "consumo", "vieja": vieja, "nueva": nueva,
     "camino": ack["camino"], "consumo": consumo,
     "ausencia": aus["camino"], "presencia": pre["camino"]}
open(d + "/consumo.json", "w").write(json.dumps(p))
print("sobre de consumo armado: las respuestas TAL CUAL")
PY

set +e; SAL=$("$VER" "$DIR/consumo.json" 2>&1); RC=$?; set -e
[ "$RC" = "0" ] || fallo "el sobre legitimo dio exit $RC (se esperaba 0): $SAL"
echo "$SAL" | sed 's/^/BANCO-CONS|   /' >&2
msg "POSITIVO: exit 0 -- el consumo se publico ENTRE las dos cabezas, sin el nodo"

# ---------------------------------------------------------------- LOS SIETE NEGATIVOS
# El repetido es del CABLE (ya se cobro arriba con el nodo vivo); los seis siguientes son del
# SOBRE y cada uno falsa UNA regla del catalogo de `spec/PAQUETE.md` seccion 5.
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
import json, sys
d = sys.argv[1]
base = json.load(open(d + "/consumo.json"))
def esc(n, p): open("%s/%s.json" % (d, n), "w").write(json.dumps(p))
def cop(): return json.loads(json.dumps(base))

# 1 - UN hermano de la AUSENCIA sustituido: la hoja vacia deja de subir al consRoot viejo.
#     El sustituto es el digest del CONSUMO, que el nodo ya acepto, luego es CANONICO seguro: un
#     nibble volteado podria sacar el limbo del campo y caer por FORMA, que es OTRA regla.
#     Intercambiar los dos caminos NO vale: con un arbol pequeno son identicos y el sabotaje no
#     discrimina -- medido EN VIVO, dio exit 0.
p = cop(); p["ausencia"]["siblings"][0] = base["consumo"]; esc("neg-ausencia-adulterada", p)
# 2 - lo mismo por el otro lado: la PRESENCIA deja de subir al consRoot de la nueva
p = cop(); p["presencia"]["siblings"][0] = base["consumo"]; esc("neg-presencia-adulterada", p)
# 3 - UN bit del isRight de la ausencia: es OTRA posicion, y el cruce de D-17 lo caza
p = cop(); p["ausencia"]["isRight"][0] = not p["ausencia"]["isRight"][0]
esc("neg-isright-de-otra-posicion", p)
# 4 - un nibble del camino del MMR: la nueva deja de extender a la vieja
p = cop(); c = p["camino"][0]; ult = c[-1]; nuevo = '0' if ult != '0' else '1'
p["camino"][0] = c[:-1] + nuevo; esc("neg-camino-mmr-adulterado", p)
# 5 - el camino de la presencia recortado a tres niveles: no tiene los del arbol
# ⚠️ SOLO los hermanos. `verificar_consumo` comprueba el CRUCE de posicion de los DOS caminos
#    ANTES de subir ninguna raiz, y `cruza_posicion` compara el isRight ENTERO contra el que la
#    posicion deriva: si tambien se recorta el isRight, el sobre cae por el CRUCE y la regla de
#    los 63 niveles se queda sin testigo. Medido EN VIVO: cayo por la regla de otro.
#    El orden de un rechazo se LEE del fuente, no se predice (PRECISION 100).
p = cop(); p["presencia"]["siblings"] = p["presencia"]["siblings"][:3]
assert len(p["presencia"]["isRight"]) == len(base["presencia"]["isRight"]), \
    "el isRight tiene que quedar ENTERO, o el cruce se lleva el rojo"
esc("neg-camino-recortado", p)
# 6 - sin `presencia`: el sobre de consumo la exige con nombre
p = cop(); del p["presencia"]; esc("neg-sin-presencia", p)
# LA PUERTA QUE FALTABA: un sabotaje que no cambia un byte no prueba nada. Se comprueba AQUI,
# antes de gastar una corrida del verificador, y se dice CUAL.
import os
crudo_base = json.dumps(base)
iguales = [f for f in sorted(os.listdir(d))
           if f.startswith("neg-") and open(d + "/" + f).read() == crudo_base]
if iguales:
    print("ROJO: estos sabotajes NO cambian un byte del positivo: " + " ".join(iguales))
    raise SystemExit(3)
print("seis cuerpos negativos derivados por MUTACION, y los seis DIFIEREN del positivo")
PY

niega "$DIR/neg-ausencia-adulterada.json"      "el consumo YA estaba"              "ausencia-adulterada"
niega "$DIR/neg-presencia-adulterada.json"     "NO sube al consRoot de la nueva"   "presencia-adulterada"
niega "$DIR/neg-isright-de-otra-posicion.json" "NO es el de la posicion"           "isright-de-otra-posicion"
niega "$DIR/neg-camino-mmr-adulterado.json"    "NO extiende a la vieja"            "camino-mmr-adulterado"
niega "$DIR/neg-camino-recortado.json"         "niveles del arbol de consumos"     "camino-recortado"
niega "$DIR/neg-sin-presencia.json"            "falta presencia"                   "sin-presencia"

# el septimo, el repetido del CABLE, se cobro ARRIBA con el nodo vivo -- y de verdad.

# ---------------------------------------------------------------- GUARDAR Y PUREZA
if [ -n "$GUARDAR" ]; then
  for f in "$DIR"/*.json; do
    cp "$f" "$GUARDAR/"
    printf 'BANCO-CONS|   %-34s %s  %s B\n' "$(basename "$f")" \
      "$(sha256sum "$f" | cut -c1-16)" "$(wc -c < "$f")" >&2
  done
  msg "capturas guardadas en $GUARDAR (de aqui salen los vectores, por MUTACION)"
fi

PORC=$(git status --porcelain | wc -l)
[ "$PORC" -eq 0 ] || fallo "el banco dejo el arbol sucio ($PORC): no debe tocarlo"
[ "$ROTOS" = "6" ] || fallo "se esperaban 6 negativos del sobre y cayeron $ROTOS"

msg "BANCO-CONSUMO VERDE: publicado, servido, verificado SIN el nodo, y las seis reglas del sobre falsadas EN VIVO"
