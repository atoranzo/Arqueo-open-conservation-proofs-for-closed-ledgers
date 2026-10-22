#!/usr/bin/env bash
# tools/banco_reinicio.sh — el banco del REINICIO sobre el mismo libro (la 18, §783 y §813).
#
# La 18 temia que un nodo real que se reinicia justo despues de cerrarse no pudiera reabrir su
# libro: el cerrojo de `sled` sobre el directorio habia hecho fallar un test 1 de cada 12 pasadas
# (§79), con `WouldBlock`, y los tests se protegieron con `open_encrypted_retry` (la 45). Esto lo
# mide con un nodo de verdad y TRES conductas:
#   REINICIO    el nodo muere -SIGKILL y SIGTERM-, se ESPERA su salida real (`wait`: el, sus
#               hilos y su cerrojo) y se rearranca al instante sobre el MISMO libro, $N veces con
#               cada senal: tiene que volver a contestar todas, y A LA PRIMERA, sin esperar al
#               cerrojo dentro del nodo. Si alguna no, la 18 se reproduce y este banco lo dice ROJO.
#   SOLAPE      SIGTERM y el nuevo SIN esperar al viejo, $N veces: lo que hace un supervisor que no
#               espera. Mientras el viejo sale, el cerrojo es suyo, y el nodo nuevo lo espera,
#               acotado (diez miradas cada 50 ms, §813): abre a la primera si llega tarde, tras esperar
#               si llega antes, y se NIEGA con el nombre del problema si el viejo tarda mas que la
#               espera. Cuantos de cada, se dice, y no se exige ninguna cifra, porque depende del
#               planificador; ROJO si uno falla de otra manera -vivo sin contestar, exit 0, una
#               negativa sin nombre- o si, ya salido el viejo, no reabre a la primera.
#   DOS NODOS   un segundo nodo sobre el libro que otro tiene abierto se NIEGA, tras su espera -de
#               verdad: no antes de 450 ms-, con el nombre del problema (`un nodo por libro`) y sin
#               tocar nada: el primero sigue contestando.
# Que espera el nodo lo dice su linea «estaba tomado; abierto tras esperar al cerrojo» (`open_layer`).
#
# ⚠️ §813: el REINICIO decia «el nodo muere —SIGKILL, y SIGTERM sin esperar a que el viejo termine—
# y se rearranca al instante sobre el MISMO libro, $N veces cada una: tiene que volver a contestar
# todas». Con SIGTERM exigia ganar una carrera que el nuevo podia perder sin que nada estuviera mal:
# el viejo, que aun salia, tenia el cerrojo, y el nuevo se negaba. Medido el 15-10-2026 sobre
# `b6994c3`, en 4 CPU, ese banco: ROJO 1 pasada de 20 sin carga anadida y 1 de 20 con cuatro `yes`;
# con otro trabajo en la maquina, 8 de 8 con cuatro `yes` (carga de 9,2 a 12,5) y 10 de 10 con ocho
# (de 6,0 a 13,4). Aislada esa vuelta, no reabren a la primera 2 de 200 rearranques sin carga, 8 de
# 200 con cuatro `yes` y, con ocho, 33 de 200 y 12 de 120; esperando la salida del viejo, con ocho,
# 0 de 200 y 0 de 120. Ese caso es ahora el SOLAPE, el nodo espera al cerrojo, y el REINICIO espera
# la salida.
#
# ⚠️ Bajo $HOME, NUNCA en /tmp: el guardian del indice se niega donde `fsync` no persiste (§740).
set -uo pipefail
msg(){ echo "BANCO-REINICIO| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
N="${BANCO_REINICIO_N:-15}"
DIR=$(mktemp -d "$HOME/.banco_reinicio.XXXXXX") || { msg "no pude crear el directorio"; exit 2; }
P=""; P2=""; VIEJO=""
limpiar(){ for x in $P $P2 $VIEJO; do kill -9 "$x" 2>/dev/null; done; rm -rf "$DIR"; }
trap limpiar EXIT

cd "$(dirname "$0")/.." || fallo "no encuentro la raiz"
msg "compilando el nodo (release)"
cargo build --locked --release -q -p zk-ssl-node || cargo build --locked --release -p zk-ssl-node || fallo "no compila"
TD=$(bash tools/target_dir.sh) || fallo "no se donde deja cargo los binarios (tools/target_dir.sh)"; NB="$TD/release/zk-ssl-node"
[ -x "$NB" ] || fallo "falta $NB"
python3 -c "print('5b'*96, end='')" > "$DIR/semilla.hex" || fallo "semilla"
chmod 600 "$DIR/semilla.hex"

PORT=8619
ESPERO='estaba tomado; abierto tras esperar al cerrojo'
vivo(){  # $1 = puerto
  curl -s --max-time 2 "http://127.0.0.1:$1" -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"zkssl_supply","params":{}}' 2>/dev/null | grep -q result
}
arranca(){  # $1 = etiqueta del registro de errores. Deja el pid en P.
  "$NB" --listen "127.0.0.1:$PORT" --latido 1 --clave-fichero "$DIR/semilla.hex" --custodia fichero \
    --diario "$DIR/nodo.jsonl" --ledger "$DIR/ledger" --contador-recepcion "$DIR/recepcion.bin" \
    --indice-firma "$DIR/indice.bin" --log warn 2>>"$DIR/err.$1" &
  P=$!
  for _ in $(seq 1 100); do
    kill -0 "$P" 2>/dev/null || return 1
    vivo "$PORT" && return 0
    sleep 0.1
  done
  return 1
}
a_la_primera(){  # $1 = etiqueta: el nodo abrio sin esperar al cerrojo
  ! grep -q "$ESPERO" "$DIR/err.$1"
}

arranca 0 || { sed -n '1,10p' "$DIR/err.0" >&2; fallo "el nodo no arranca ni la primera vez"; }
for SENAL in KILL TERM; do
  for i in $(seq 1 "$N"); do
    VIEJO=$P
    kill -"$SENAL" "$VIEJO"
    # La salida REAL del viejo: `kill` vuelve en cuanto la senal queda puesta, no cuando el proceso
    # ha terminado, y el cerrojo es suyo hasta que sale.
    wait "$VIEJO" 2>/dev/null; VIEJO=""
    arranca "$SENAL$i" || { sed -n '1,5p' "$DIR/err.$SENAL$i" >&2;
      fallo "la 18 SE REPRODUCE: el viejo ya SALIO (SIG$SENAL, vuelta $i) y el nodo no reabre su libro"; }
    a_la_primera "$SENAL$i" || { grep "$ESPERO" "$DIR/err.$SENAL$i" >&2;
      fallo "el viejo ya SALIO (SIG$SENAL, vuelta $i) y el nodo nuevo tuvo que esperar al cerrojo"; }
  done
  msg "REINICIO: $N de $N rearranques a la primera tras SIG$SENAL y la salida del viejo"
done

# SOLAPE: SIGTERM y el nuevo al instante, SIN esperar al viejo.
PRIMERA=0; TRAS=0; NIEGA=0
for i in $(seq 1 "$N"); do
  VIEJO=$P
  kill -TERM "$VIEJO"
  if arranca "S$i"; then
    if a_la_primera "S$i"; then PRIMERA=$((PRIMERA+1)); else TRAS=$((TRAS+1)); fi
    wait "$VIEJO" 2>/dev/null; VIEJO=""
    continue
  fi
  kill -0 "$P" 2>/dev/null && fallo "SOLAPE, vuelta $i: el nodo nuevo sigue vivo y no contesta"
  wait "$P"; RC=$?
  [ "$RC" != "0" ] || fallo "SOLAPE, vuelta $i: el nodo nuevo salio con 0 sin contestar"
  grep -q 'un nodo por libro' "$DIR/err.S$i" \
    || { sed -n '1,5p' "$DIR/err.S$i" >&2; fallo "SOLAPE, vuelta $i: el nuevo se nego, pero sin decir por que"; }
  wait "$VIEJO" 2>/dev/null; VIEJO=""
  arranca "S${i}b" || { sed -n '1,5p' "$DIR/err.S${i}b" >&2;
    fallo "la 18 SE REPRODUCE: el viejo ya SALIO (SOLAPE, vuelta $i) y el nodo no reabre su libro"; }
  a_la_primera "S${i}b" || fallo "el viejo ya SALIO (SOLAPE, vuelta $i) y el nodo nuevo tuvo que esperar al cerrojo"
  NIEGA=$((NIEGA+1))
done
msg "SOLAPE: $N rearranques sin esperar al viejo: $PRIMERA abren a la primera, $TRAS tras esperar al cerrojo dentro del nodo, $NIEGA se niegan con su nombre (el viejo tardo mas que la espera) y abren en cuanto sale; ninguno falla de otra manera"

# DOS NODOS: el segundo, con sus propios contadores, sobre el MISMO libro.
T0=$(date +%s%N)
"$NB" --listen "127.0.0.1:$((PORT+1))" --latido 1 --clave-fichero "$DIR/semilla.hex" --custodia fichero \
  --diario "$DIR/otro.jsonl" --ledger "$DIR/ledger" --contador-recepcion "$DIR/recepcion2.bin" \
  --indice-firma "$DIR/indice2.bin" --log warn > "$DIR/otro.out" 2>&1 &
P2=$!
for _ in $(seq 1 100); do kill -0 "$P2" 2>/dev/null || break; sleep 0.1; done
if kill -0 "$P2" 2>/dev/null; then fallo "un SEGUNDO nodo sigue vivo sobre el libro de otro"; fi
wait "$P2"; RC=$?; P2=""
MS=$(( ($(date +%s%N) - T0) / 1000000 ))
[ "$RC" != "0" ] || fallo "el segundo nodo salio con 0 sobre un libro tomado"
grep -q 'un nodo por libro' "$DIR/otro.out" \
  || { sed -n '1,5p' "$DIR/otro.out" >&2; fallo "el segundo nodo se nego, pero sin decir por que"; }
grep -q 'mirado 10 veces' "$DIR/otro.out" \
  || { sed -n '1,5p' "$DIR/otro.out" >&2; fallo "el segundo nodo se nego sin su espera acotada"; }
# §813: y la espera es de verdad: nueve pausas de 50 ms, que `sleep` da como minimo. Medido desde antes
# de lanzarlo hasta su salida, nunca baja de 450 ms con el nodo bueno; uno que mira sin dormir se
# negaba en la revision en 123 a 135 ms.
[ "$MS" -ge 450 ] || { sed -n '1,5p' "$DIR/otro.out" >&2;
  fallo "el segundo nodo se nego en $MS ms, sin dormir su espera (nueve pausas de 50 ms, 450 ms)"; }
# ⚠️ §813: era una sola sonda de 2 s, y son hasta cinco seguidas: la misma sonda que `arranca`, que
# hace hasta cien con 0,1 s entre una y otra. Medido el 15-10-2026 con ocho `yes` en 4 CPU, una sonda de 2 s a
# un nodo vivo queda sin respuesta 10 de 100 veces SIN segundo nodo y 5 de 100 con el (la 123 del
# BACKLOG). Lo que se comprueba es que sigue vivo y contesta.
contesta(){ for _ in 1 2 3 4 5; do kill -0 "$P" 2>/dev/null || return 1; vivo "$PORT" && return 0; done; return 1; }
contesta || fallo "el primer nodo dejo de contestar cuando el segundo intento abrir su libro"
msg "DOS NODOS: el segundo se niega con su nombre tras su espera ($MS ms, exit $RC) y el primero sigue contestando"
msg "BANCO-REINICIO VERDE: tras la salida del viejo, un nodo reabre su libro a la primera; mientras sale, lo espera, acotado, o se niega con su nombre; dos sobre el mismo, no"
