#!/usr/bin/env bash
# tools/banco_dos_libros.sh -- el banco de E4a del RFC-0006 (la deteccion ENTRE LIBROS).
#
# Demuestra el HECHO que E4 existe para detectar: DOS nodos con DOS claves distintas -dos libros-
# aceptan EL MISMO consumo, cada uno bajo su propia raiz firmada, y ninguno de los dos puede
# saberlo. Dentro de un libro el uso unico es un invariante; entre libros no hay quien ordene, y
# por eso lo que queda es DETECCION, nunca prevencion (RFC-0006, D-4).
#
# Y demuestra EN VIVO el limite del mando de HOY: con una cabeza de cada libro, el binario sale 1
# con <<las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante>> -por las DOS vias,
# la de extension y la de consumo, que son dos productores del mismo texto-. Ese texto es uno de
# los CINCO que `spec/PAQUETE.md` seccion 9 declara SIN VECTOR, y este banco lo produce por fin
# AISLADO: las dos cabezas son v4, las dos recomponen su digest y las dos firmas verifican, asi
# que el UNICO defecto del sobre es la clave.
#
# LO QUE ESTE BANCO NO HACE, y va escrito: NO comprueba que el consumo este bajo las dos raices.
# Eso es aritmetica de Merkle y la hace el mando, que todavia no lee la forma del conflicto: es
# E4a-2. Aqui se PRODUCE el material y se demuestra el hecho; alli se verifica sin los nodos.
#
# FUERA del canon: levanta DOS procesos y espera latidos. Hermano de `tools/banco_consumo.sh`, del
# que copia su forma; se desvia de el en lo unico que el frente pide: dos de todo -puerto, estado,
# semilla e indice de firma-. **Compila en RELEASE**, porque aqui se firma de verdad.
#
# ⚠️ LOS DOS INDICES DE FIRMA SON DOS FICHEROS. Compartirlo haria que los dos nodos quemaran el
# mismo indice XMSS, que es la clase de la nota 100 y de los S331-S333. Se derivan del rotulo.
#
# NO ESCRIBE EN EL ARBOL: todo lo suyo vive en un temporal bajo $HOME, que borra al salir. Lo
# comprueba al final por `git status --porcelain`.
#
#   bash tools/banco_dos_libros.sh [--guardar <dir>]
#
# --guardar  copia las DOS cabezas, los DOS caminos de presencia, el sobre del rechazo por claves
#            distintas y el sobre de CONFLICTO ya armado -que ningun binario lee todavia-, cada uno
#            con su huella. De esas capturas se derivan por MUTACION los vectores de E4a-2 (regla 2
#            del PROCESO: los vectores jamas se reescriben, se derivan de capturas reales).
set -euo pipefail
msg(){ printf 'BANCO-2LIB| %s\n' "$*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }

# ⚠️ LA RAIZ SE DERIVA, no se supone: este fichero puede correrse desde `tools/` o todavia SUELTO
# desde Downloads. Se prueban las dos procedencias, en orden, y la elegida trae su PRUEBA DE VIDA.
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
  *) fallo "uso: bash tools/banco_dos_libros.sh [--guardar <dir>]" ;;
esac

# 8597 lo usa banco_extension y 8598 banco_consumo: estos dos van detras, asi que los cuatro
# bancos pueden correr a la vez.
PORT_A=8599
PORT_B=8600
DIR=$(mktemp -d "$HOME/.banco_dos_libros.XXXXXX")
PID_A=""
PID_B=""
limpiar(){
  for P in "$PID_A" "$PID_B"; do
    if [ -n "$P" ]; then kill -9 "$P" 2>/dev/null || true; fi
  done
  rm -rf "$DIR"
}
trap limpiar EXIT INT TERM HUP QUIT

msg "compilando nodo y verificador en RELEASE (aqui se firma de verdad)"
cargo build --release -q -p zk-ssl-node -p zk-ssl-verify 2>/dev/null \
  || cargo build --release -p zk-ssl-node -p zk-ssl-verify || fallo "no compila"
NODO=target/release/zk-ssl-node
VER=target/release/zk-ssl-verify

# DOS semillas DISTINTAS: son lo que hace que sean dos LIBROS y no dos copias. El patron de cada
# una se teclea aqui a proposito -es material de prueba, no un dato derivable- y se comprueba que
# midan lo mismo y que NO sean iguales, que es la unica propiedad que este banco necesita de ellas.
mkdir -p "$DIR/a" "$DIR/b"
python3 -c "print('5b'*96, end='')" > "$DIR/a/semilla.hex"
python3 -c "print('a3'*96, end='')" > "$DIR/b/semilla.hex"
chmod 600 "$DIR/a/semilla.hex" "$DIR/b/semilla.hex"
[ "$(wc -c < "$DIR/a/semilla.hex")" = "$(wc -c < "$DIR/b/semilla.hex")" ] \
  || fallo "las dos semillas no miden lo mismo"
if cmp -s "$DIR/a/semilla.hex" "$DIR/b/semilla.hex"; then
  fallo "las dos semillas son IGUALES: entonces no hay dos libros, hay uno repetido"
fi
msg "dos semillas, distintas y del mismo ancho"

arranca(){ # $1 rotulo (a|b)  $2 puerto
  local L="$1" P="$2"
  "$NODO" --listen "127.0.0.1:$P" --latido 2 \
    --clave-fichero "$DIR/$L/semilla.hex" --custodia fichero \
    --diario "$DIR/$L/diario.jsonl" --ledger "$DIR/$L/ledger" \
    --contador-recepcion "$DIR/$L/recepcion.bin" \
    --indice-firma "$DIR/$L/indice-firma.bin" \
    --log warn 2>>"$DIR/$L/nodo.err" &
  if [ "$L" = "a" ]; then PID_A=$!; else PID_B=$!; fi
}
rpc(){ # $1 puerto  $2 metodo  $3 params
  curl -s --max-time 20 "http://127.0.0.1:$1" \
    -H 'Content-Type: application/json' \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$2\",\"params\":$3}"
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
crudo(){ python3 -c 'import json,sys; print(json.loads(sys.argv[1]))' "$1"; }

# El consumo tiene que ser CANONICO en el campo (PRECISION 109: el campo REDUCE). Un byte no nulo
# en el offset 3 de cada grupo de ocho lo deja canonico leido en LE y en BE, asi que no hace falta
# suponer el orden de bytes del cable. Es el MISMO para los dos libros: ese es el punto entero.
CONSUMO=$(python3 - <<'PY'
b = bytearray(32)
for i, v in ((3, 0xA7), (11, 0x5C), (19, 0x13), (27, 0x91)):
    b[i] = v
print("0x" + b.hex())
PY
)
msg "el consumo de la corrida, el MISMO para los dos libros: $CONSUMO"

# ---------------------------------------------------------------- LOS DOS LIBROS, EN PIE
arranca a "$PORT_A"
arranca b "$PORT_B"

cabeza_v4(){ # $1 puerto  $2 rotulo  -- espera una cabeza FIRMADA v4 con mmrSize >= 2
  local P="$1" L="$2" V MS
  for _ in $(seq 1 60); do
    V=$(rpc "$P" zkssl_signedEpochHead '{}' 2>/dev/null || true)
    case "$V" in
      *'"available":true'*)
        MS=$(campo "$V" result.mmrSize) || true
        if [ -n "${MS:-}" ] && [ "$(qnum "$MS")" -ge 2 ]; then echo "$V"; return 0; fi
        ;;
    esac
    sleep 0.5
  done
  fallo "el libro $L no dio una cabeza firmada con mmrSize >= 2"
}
V_A=$(cabeza_v4 "$PORT_A" A)
V_B=$(cabeza_v4 "$PORT_B" B)
for PAR in "A:$V_A" "B:$V_B"; do
  L="${PAR%%:*}"; V="${PAR#*:}"
  [ "$(qnum "$(campo "$V" result.formatVersion)")" = "4" ] \
    || fallo "la cabeza del libro $L no es v4: sin consRoot no hay nada que comparar"
done

# ⚠️ LA PROPIEDAD QUE HACE QUE SEAN DOS LIBROS, y se comprueba ANTES de nada: las claves publicas
# tienen que ser DISTINTAS. Si fueran iguales, todo lo que sigue seria la historia de un solo
# firmante y el banco estaria demostrando otra cosa.
CLAVE_A=$(crudo "$(campo "$V_A" result.publicKey)")
CLAVE_B=$(crudo "$(campo "$V_B" result.publicKey)")
[ "$CLAVE_A" != "$CLAVE_B" ] \
  || fallo "los dos nodos publican la MISMA clave: no son dos libros"
msg "dos libros en pie, con claves DISTINTAS (${CLAVE_A:0:18}... / ${CLAVE_B:0:18}...)"

# ---------------------------------------------------------------- EL MISMO CONSUMO, EN LOS DOS
publica(){ # $1 puerto  $2 rotulo
  local R
  R=$(rpc "$1" zkssl_publishConsumo "{\"consumo\":\"$CONSUMO\"}")
  case "$R" in
    *'"accepted":true'*) echo "$R" ;;
    *) fallo "el libro $2 NO acepto el consumo: $R" ;;
  esac
}
PUB_A=$(publica "$PORT_A" A)
PUB_B=$(publica "$PORT_B" B)
SEQ_LOG_A=$(campo "$PUB_A" result.logSeq)
SEQ_LOG_B=$(campo "$PUB_B" result.logSeq)
msg "EL HECHO: el MISMO consumo aceptado por los DOS libros (logSeq $(qnum "$SEQ_LOG_A") y $(qnum "$SEQ_LOG_B"))"
msg "  y ninguno de los dos puede saberlo: no se hablan, y nada los ordena"

# El repetido, dentro de cada libro, sigue cayendo: el invariante INTRA-libro no se ha roto.
for PAR in "A:$PORT_A" "B:$PORT_B"; do
  L="${PAR%%:*}"; P="${PAR#*:}"
  R=$(rpc "$P" zkssl_publishConsumo "{\"consumo\":\"$CONSUMO\"}")
  case "$R" in
    *'"accepted":false'*) : ;;
    *) fallo "el libro $L acepto DOS VECES el mismo consumo: el invariante intra-libro esta roto" ;;
  esac
  case "$R" in
    *"ya esta publicado"*) msg "  dentro del libro $L el repetido SIGUE cayendo: $(crudo "$(campo "$R" result.reason)")" ;;
    *) fallo "el repetido del libro $L cayo, pero NO por su regla: $R" ;;
  esac
done

# ---------------------------------------------------------------- LAS CABEZAS QUE LO ACREDITAN
# La pareja firmada es el acumulador ANTES de la cabeza, asi que se espera a una cabeza cuyo `seq`
# alcance el `logSeq` del consumo: solo esa puede llevar el consumo bajo su consRoot.
sleep 5
nueva_tras(){ # $1 puerto  $2 rotulo  $3 logSeq
  local P="$1" L="$2" META="$3" V SQ
  for _ in $(seq 1 40); do
    V=$(rpc "$P" zkssl_signedEpochHead '{}' || true)
    case "$V" in
      *'"available":true'*)
        SQ=$(campo "$V" result.seq) || true
        if [ -n "${SQ:-}" ] && [ "$(qnum "$SQ")" -ge "$(qnum "$META")" ]; then echo "$V"; return 0; fi
        ;;
    esac
    sleep 0.5
  done
  fallo "el libro $L no emitio una cabeza con seq >= $(qnum "$META")"
}
N_A=$(nueva_tras "$PORT_A" A "$SEQ_LOG_A")
N_B=$(nueva_tras "$PORT_B" B "$SEQ_LOG_B")
for PAR in "A:$N_A" "B:$N_B"; do
  L="${PAR%%:*}"; V="${PAR#*:}"
  [ "$(qnum "$(campo "$V" result.formatVersion)")" = "4" ] || fallo "la cabeza nueva de $L no es v4"
  [ "$(qnum "$(campo "$V" result.consCount)")" -ge 1 ] \
    || fallo "la cabeza nueva de $L dice consCount 0: no acredita ningun consumo"
done
CR_A=$(crudo "$(campo "$N_A" result.consRoot)")
CR_B=$(crudo "$(campo "$N_B" result.consRoot)")
msg "las DOS cabezas acreditan su conjunto: consRoot ${CR_A:0:18}... y ${CR_B:0:18}..."
[ "$CR_A" != "$CR_B" ] || msg "  OJO: los dos consRoot COINCIDEN (mismo consumo, mismo arbol vacio detras)"

# ---------------------------------------------------------------- LOS DOS CAMINOS DE PRESENCIA
camino(){ # $1 puerto  $2 seq  $3 rotulo
  local R
  R=$(rpc "$1" zkssl_consumoPath "{\"consumo\":\"$CONSUMO\",\"seq\":$2}")
  case "$R" in
    *'"available":true'*) echo "$R" ;;
    *) fallo "consumoPath del libro $3 no dio camino: $R" ;;
  esac
}
P_A=$(camino "$PORT_A" "$(campo "$N_A" result.seq)" A)
P_B=$(camino "$PORT_B" "$(campo "$N_B" result.seq)" B)
msg "los dos caminos de PRESENCIA servidos, uno por libro"

kill -9 "$PID_A" 2>/dev/null || true
kill -9 "$PID_B" 2>/dev/null || true
wait "$PID_A" 2>/dev/null || true
wait "$PID_B" 2>/dev/null || true
PID_A=""; PID_B=""
msg "los DOS nodos estan MUERTOS; a partir de aqui nada de lo que sigue los toca"

# ---------------------------------------------------------------- EL LIMITE DEL MANDO DE HOY
# El texto <<las cabezas llevan claves DISTINTAS>> es uno de los CINCO que la seccion 9 declara sin
# vector. Aqui se produce AISLADO -las dos cabezas son v4, recomponen su digest y sus firmas
# verifican- y por las DOS vias, que son dos productores del mismo texto (medido en la 108).
python3 - "$V_A" "$N_B" "$DIR" <<'PY'
import json, sys
a = json.loads(sys.argv[1])["result"]
b = json.loads(sys.argv[2])["result"]
d = sys.argv[3]
# Las respuestas van TAL CUAL: reescribir seria adulterar.
json.dump({"v": 1, "tipo": "extension", "vieja": a, "nueva": b, "camino": []},
          open(d + "/rechazo-claves-distintas-ext.json", "w"))
json.dump({"v": 1, "tipo": "consumo", "vieja": a, "nueva": b, "camino": []},
          open(d + "/rechazo-claves-distintas-cons.json", "w"))
print("dos sobres armados con una cabeza de CADA libro")
PY

niega(){ # $1 fichero  $2 fragmento  $3 rotulo
  local s r
  set +e; s=$("$VER" "$1" 2>&1); r=$?; set -e
  [ "$r" = "1" ] || fallo "$3 dio exit $r (se esperaba 1): $s"
  case "$s" in
    *"$2"*) msg "ROJO EN VIVO $3: exit 1 -- $(echo "$s" | tail -n 1)" ;;
    *) fallo "$3 cayo, pero NO por su regla: se esperaba <<$2>> y dijo: $s" ;;
  esac
}
niega "$DIR/rechazo-claves-distintas-ext.json"  "claves DISTINTAS" "por la via de EXTENSION"
niega "$DIR/rechazo-claves-distintas-cons.json" "claves DISTINTAS" "por la via de CONSUMO"
msg "el mismo texto por las DOS vias: dos productores, uno solo declarado en el catalogo"

# El sobre de CONFLICTO, armado y GUARDADO, que ningun binario lee todavia: es E4a-2.
python3 - "$N_A" "$N_B" "$P_A" "$P_B" "$CONSUMO" "$DIR" <<'PY'
import json, sys
na = json.loads(sys.argv[1])["result"]
nb = json.loads(sys.argv[2])["result"]
pa = json.loads(sys.argv[3])["result"]
pb = json.loads(sys.argv[4])["result"]
consumo, d = sys.argv[5], sys.argv[6]
p = {"v": 1, "tipo": "conflicto", "consumo": consumo,
     "libros": [{"cabeza": na, "presencia": pa["camino"]},
                {"cabeza": nb, "presencia": pb["camino"]}]}
json.dump(p, open(d + "/conflicto.json", "w"))
print("sobre de CONFLICTO armado (captura para E4a-2; hoy ningun binario lo lee)")
PY

# Y se DEMUESTRA que hoy no lo lee, con su nombre: fail-closed, no <<sigue por compatibilidad>>.
niega "$DIR/conflicto.json" "tipo desconocido" "el mando de HOY ante el sobre de conflicto"

# ---------------------------------------------------------------- GUARDAR Y PUREZA
if [ -n "$GUARDAR" ]; then
  python3 - "$V_A" "$N_A" "$V_B" "$N_B" "$P_A" "$P_B" "$DIR" <<'PY'
import json, sys
d = sys.argv[7]
for nom, cru in (("cabeza-A-vieja", 1), ("cabeza-A-nueva", 2), ("cabeza-B-vieja", 3),
                 ("cabeza-B-nueva", 4), ("presencia-A", 5), ("presencia-B", 6)):
    json.dump(json.loads(sys.argv[cru])["result"], open("%s/%s.json" % (d, nom), "w"))
print("capturas por libro escritas")
PY
  for f in "$DIR"/*.json; do
    cp "$f" "$GUARDAR/"
    printf 'BANCO-2LIB|   %-38s %s  %s B\n' "$(basename "$f")" \
      "$(sha256sum "$f" | cut -c1-16)" "$(wc -c < "$f")" >&2
  done
  msg "capturas guardadas en $GUARDAR (de aqui salen los vectores de E4a-2, por MUTACION)"
fi

PORC=$(git status --porcelain | wc -l)
[ "$PORC" -eq 0 ] || fallo "el banco dejo el arbol sucio ($PORC): no debe tocarlo"

msg "BANCO-DOS-LIBROS VERDE: el MISMO consumo vive en DOS libros con DOS claves, cada uno bajo su"
msg "  raiz firmada; el invariante INTRA-libro sigue en pie; y el mando de hoy no puede juntarlos."
msg "  Eso es la deteccion que E4a-2 hara portable. Prevencion, ninguna: nadie ordena entre libros."
