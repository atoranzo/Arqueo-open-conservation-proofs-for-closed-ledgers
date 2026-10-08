#!/usr/bin/env bash
# tools/banco_un_proceso.sh — el banco de UN SOLO PROCESO POR CONTADOR (§709, SEC-1).
#
# Los tests del guardian, del nodo y del testigo prueban el cerrojo dentro de un
# proceso: es del descriptor, y dos aperturas en el mismo proceso se excluyen igual
# que en dos. Este banco lo prueba con los BINARIOS lanzados dos veces, que es lo que
# hace un operador:
#
#   SIN-BANDERAS  dos nodos sin clave y sin banderas, desde el MISMO directorio: el
#                 segundo NO arranca, y dice que proceso tiene `recepcion.bin`.
#   KILL-9        muerto el primero con `kill -9`, un tercero arranca en el mismo
#                 sitio: el cerrojo se va con el proceso, y no queda nada que limpiar.
#   FIRMANTES     dos nodos con la MISMA semilla y el mismo `--indice-firma`, cada uno
#                 con su diario y su recepcion: el segundo NO arranca, y el primero
#                 sigue firmando.
#   RUTAS         al firmar, sin `--indice-firma`, con una ruta relativa o sin
#                 `--contador-recepcion`, el nodo NO arranca, y no deja ningun contador.
#   COFIRMANTES   dos testigos que cofirman, con semillas distintas, sobre el mismo
#                 `--indice-cofirma`: el segundo NO arranca. Con una ruta relativa,
#                 tampoco.
#
# ⚠️ Sobre `54fe931`, el arbol de antes del §709, cuatro tramos salen ROJOS: el segundo
#    proceso arranca —o el primero de RUTAS, que firma sin que nadie nombre su contador—
#    y lo para el `timeout` de este banco. KILL-9 pasa tambien alli, y tiene que pasar:
#    no prueba el arreglo sino que no estorbe, porque un cerrojo que sobreviviera al
#    proceso dejaria un nodo caido sin poder volver.
# ⚠️ Exige que el error dé el PID del primero. Los tests sólo exigen que, si lo da, sea
#    el suyo: depende de lo que el sistema enseñe en /proc. Donde no lo da, el mensaje
#    dice «otro proceso (este sistema no dice cual)», y este banco sale ROJO por eso y
#    no por el cerrojo: el log del tramo lo muestra.
# ⚠️ Lo que NO mide, porque el cerrojo no lo cubre: la misma semilla con OTRO contador,
#    otra maquina, un sistema de ficheros de red. Lo declaran `SECURITY.md` §2 y
#    `doc/CONFIANZA_RESIDUAL.md`.
# ⚠️ Bajo $HOME, NUNCA en /tmp: el guardian se niega donde `fsync` no persiste (K.1, §234).
set -uo pipefail
msg(){ printf 'BANCO-UN-PROCESO| %s\n' "$*" >&2; }
DIR=$(mktemp -d "$HOME/.banco_un_proceso.XXXXXX") || { msg "no pude crear el directorio"; exit 2; }
PIDS=()
limpiar(){
  for p in "${PIDS[@]}"; do kill -9 "$p" 2>/dev/null; done
  wait 2>/dev/null
  rm -rf "$DIR"
}
trap limpiar EXIT
fallo(){ msg "ROJO: $*"; exit 1; }

cd "$(dirname "$0")/.." || fallo "no encuentro la raiz"
RAIZ=$(pwd)
msg "compilando nodo y cli (release: aqui se firma)"
cargo build --release -q -p zk-ssl-node -p zk-ssl-cli 2>/dev/null \
  || cargo build --release -p zk-ssl-node -p zk-ssl-cli || fallo "no compila"
NB="$RAIZ/target/release/zk-ssl-node"; CB="$RAIZ/target/release/zk-ssl-cli"
[ -x "$NB" ] && [ -x "$CB" ] || fallo "faltan los binarios"

# ⚠️ Semillas DETERMINISTAS, en los dos formatos del proyecto: el nodo la lee en HEX y
#    el testigo en BINARIO crudo (§301).
mkdir -p "$DIR/sin" "$DIR/f" "$DIR/r" "$DIR/c"
python3 -c "print('5b'*96, end='')" > "$DIR/f/semilla.hex"; chmod 600 "$DIR/f/semilla.hex"
python3 -c "
import sys
sys.stdout.buffer.write(bytes(((i*31+9) % 256) for i in range(96)))" > "$DIR/c/t1.bin"
python3 -c "
import sys
sys.stdout.buffer.write(bytes(((i*17+4) % 256) for i in range(96)))" > "$DIR/c/t2.bin"
chmod 600 "$DIR/c/t1.bin" "$DIR/c/t2.bin"

responde(){  # $1 puerto
  local r
  r=$(curl -s --max-time 5 "http://127.0.0.1:$1" -H 'Content-Type: application/json' \
      -d '{"jsonrpc":"2.0","id":1,"method":"zkssl_supply","params":{}}' 2>/dev/null || true)
  case "$r" in *result*) return 0;; esac
  return 1
}
espera_que_responda(){  # $1 puerto  $2 pid  $3 log
  for _ in $(seq 1 120); do
    responde "$1" && return 0
    kill -0 "$2" 2>/dev/null || { sed -n '1,20p' "$3" >&2; fallo "el nodo del puerto $1 murio al arrancar"; }
    sleep 0.5
  done
  sed -n '1,20p' "$3" >&2; fallo "el nodo del puerto $1 no respondio en 60 s"
}
# El valor de un contador: ocho bytes little-endian.
contador(){ python3 -c "import sys; print(int.from_bytes(open(sys.argv[1],'rb').read(),'little'))" "$1"; }
# Un proceso que NO tiene que arrancar: sale solo, distinto de 0, y dice lo que tiene que
# decir. Si sigue vivo a los 60 s, ARRANCO. `no_arranca_en` lo lanza desde un directorio.
no_arranca(){  # $1 rotulo  $2 log  $3 texto  resto: la orden
  local rot="$1" log="$2" texto="$3" rc
  shift 3
  timeout 60 "$@" > "$log" 2>&1; rc=$?
  juzga "$rot" "$log" "$texto" "$rc"
}
no_arranca_en(){  # $1 rotulo  $2 log  $3 texto  $4 directorio  resto: la orden
  local rot="$1" log="$2" texto="$3" dir="$4" rc
  shift 4
  timeout 60 bash -c 'cd "$0" && exec "$@"' "$dir" "$@" > "$log" 2>&1; rc=$?
  juzga "$rot" "$log" "$texto" "$rc"
}
juzga(){  # $1 rotulo  $2 log  $3 texto  $4 rc
  local rot="$1" log="$2" texto="$3" rc="$4"
  [ "$rc" != "124" ] || { tail -5 "$log" >&2; fallo "$rot: ARRANCO, y seguia vivo a los 60 s"; }
  [ "$rc" != "0" ] || { tail -5 "$log" >&2; fallo "$rot: salio con 0"; }
  grep -qF -- "$texto" "$log" || { tail -5 "$log" >&2; fallo "$rot: no dice «$texto»"; }
  msg "$rot: exit $rc, y dice «$texto»"
}

# ══ SIN-BANDERAS ══ dos nodos sin clave desde el mismo directorio
(cd "$DIR/sin" && exec "$NB" --listen 127.0.0.1:8641 --latido 0 --log warn) 2>"$DIR/sin1.err" &
P1=$!; PIDS+=("$P1")
espera_que_responda 8641 "$P1" "$DIR/sin1.err"
msg "SIN-BANDERAS: el primero responde (pid $P1) y tiene $DIR/sin/recepcion.bin"
no_arranca_en "SIN-BANDERAS" "$DIR/sin2.err" "recepcion.bin ya lo tiene abierto el proceso $P1" \
  "$DIR/sin" "$NB" --listen 127.0.0.1:8642 --latido 0 --log warn
responde 8641 || fallo "SIN-BANDERAS: el primero dejo de responder"

# ══ KILL-9 ══ el cerrojo se va con el proceso
kill -9 "$P1"; wait "$P1" 2>/dev/null
(cd "$DIR/sin" && exec "$NB" --listen 127.0.0.1:8641 --latido 0 --log warn) 2>"$DIR/sin3.err" &
P3=$!; PIDS+=("$P3")
espera_que_responda 8641 "$P3" "$DIR/sin3.err"
msg "KILL-9: muerto el primero con kill -9, un tercero arranca sobre el mismo contador"
kill -9 "$P3"; wait "$P3" 2>/dev/null

# ══ FIRMANTES ══ la misma semilla, el mismo indice
firmante(){  # $1 puerto  $2 sufijo: la orden, en el array FIRMANTE
  FIRMANTE=("$NB" --listen "127.0.0.1:$1" --latido 1
    --clave-fichero "$DIR/f/semilla.hex" --custodia fichero
    --diario "$DIR/f/diario$2.jsonl" --indice-firma "$DIR/f/indice-firma.bin"
    --contador-recepcion "$DIR/f/recepcion$2.bin" --log warn)
}
firmante 8643 1
"${FIRMANTE[@]}" 2>"$DIR/f/a.err" &
PA=$!; PIDS+=("$PA")
espera_que_responda 8643 "$PA" "$DIR/f/a.err"
for _ in $(seq 1 60); do
  [ -s "$DIR/f/indice-firma.bin" ] && [ "$(contador "$DIR/f/indice-firma.bin")" -ge 1 ] && break
  sleep 0.5
done
C1=$(contador "$DIR/f/indice-firma.bin")
[ "$C1" -ge 1 ] || fallo "FIRMANTES: el primero no firmo en 30 s: el estado del tramo no existe"
msg "FIRMANTES: el primero firma (pid $PA), indice en $C1"
firmante 8644 2
no_arranca "FIRMANTES" "$DIR/f/b.err" "indice-firma.bin ya lo tiene abierto el proceso $PA" \
  "${FIRMANTE[@]}"
sleep 3
C2=$(contador "$DIR/f/indice-firma.bin")
[ "$C2" -gt "$C1" ] || fallo "FIRMANTES: el primero dejo de firmar ($C1 -> $C2)"
msg "FIRMANTES: y el primero sigue firmando: indice $C1 -> $C2"

# ══ RUTAS ══ al firmar, los dos contadores se nombran enteros
CLAVE=("$NB" --listen 127.0.0.1:8645 --latido 1 --log warn
  --clave-fichero "$DIR/f/semilla.hex" --custodia fichero --diario "$DIR/r-diario.jsonl")
no_arranca_en "RUTAS sin --indice-firma" "$DIR/r1.err" "--indice-firma es obligatoria" "$DIR/r" \
  "${CLAVE[@]}" --contador-recepcion "$DIR/r-recepcion.bin"
no_arranca_en "RUTAS relativa" "$DIR/r2.err" "\`indice.bin\` es relativa" "$DIR/r" \
  "${CLAVE[@]}" --indice-firma indice.bin --contador-recepcion "$DIR/r-recepcion.bin"
no_arranca_en "RUTAS sin --contador-recepcion" "$DIR/r3.err" "--contador-recepcion es obligatoria" \
  "$DIR/r" "${CLAVE[@]}" --indice-firma "$DIR/r-indice.bin"
[ -z "$(ls -A "$DIR/r")" ] || fallo "RUTAS: quedo algo en el directorio de trabajo: $(ls -A "$DIR/r")"
for f in r-indice.bin r-recepcion.bin r-diario.jsonl; do
  [ ! -e "$DIR/$f" ] || fallo "RUTAS: se creo $f antes de negarse"
done
msg "RUTAS: ninguno arranca, y ninguno deja un contador ni un diario"

# ══ COFIRMANTES ══ dos testigos sobre el mismo contador
"$CB" witness --nodo "http://127.0.0.1:8643" --cada 1 --veces 0 --no-color \
  --diario "$DIR/c/diario1.jsonl" --cofirmar "$DIR/c/t1.bin" \
  --indice-cofirma "$DIR/c/contador.bin" --cofirmas "$DIR/c/cofirmas1.jsonl" > "$DIR/c/w1.out" 2>&1 &
W1=$!; PIDS+=("$W1")
for _ in $(seq 1 60); do
  [ -s "$DIR/c/cofirmas1.jsonl" ] && break
  kill -0 "$W1" 2>/dev/null || { tail -10 "$DIR/c/w1.out" >&2; fallo "COFIRMANTES: el primer testigo murio"; }
  sleep 0.5
done
[ -s "$DIR/c/cofirmas1.jsonl" ] || { tail -10 "$DIR/c/w1.out" >&2; fallo "COFIRMANTES: el primero no cofirmo en 30 s"; }
msg "COFIRMANTES: el primero cofirma (pid $W1)"
no_arranca "COFIRMANTES" "$DIR/c/w2.out" "contador.bin ya lo tiene abierto el proceso $W1" \
  "$CB" witness --nodo "http://127.0.0.1:8643" --cada 1 --veces 0 --no-color \
    --diario "$DIR/c/diario2.jsonl" --cofirmar "$DIR/c/t2.bin" \
    --indice-cofirma "$DIR/c/contador.bin" --cofirmas "$DIR/c/cofirmas2.jsonl"
[ ! -s "$DIR/c/cofirmas2.jsonl" ] || fallo "COFIRMANTES: el segundo escribio cofirmas"
no_arranca_en "COFIRMANTES relativa" "$DIR/c/w3.out" "\`contador.bin\` es relativa" "$DIR/c" \
  "$CB" witness --nodo "http://127.0.0.1:8643" --cada 1 --veces 0 --no-color \
    --cofirmar "$DIR/c/t2.bin" --indice-cofirma contador.bin --cofirmas "$DIR/c/cofirmas3.jsonl"
kill -0 "$W1" 2>/dev/null || fallo "COFIRMANTES: el primero dejo de correr"

msg "BANCO-UN-PROCESO VERDE: un segundo proceso sobre el mismo contador NO arranca —nodo sin clave, firmante, cofirmante—, el cerrojo se va con kill -9, y al firmar los contadores van con su ruta absoluta"
