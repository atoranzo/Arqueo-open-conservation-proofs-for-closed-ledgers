#!/usr/bin/env bash
# tools/banco_reinicio.sh — el banco del REINICIO sobre el mismo libro (la 18, §783).
#
# La 18 temia que un nodo real que se reinicia justo despues de cerrarse no pudiera reabrir su
# libro: el cerrojo de `sled` sobre el directorio habia hecho fallar un test 1 de cada 12 pasadas
# (§79), con `WouldBlock`, y los tests se protegieron con `open_encrypted_retry` (la 45). Esto lo
# mide con un nodo de verdad y DOS conductas:
#   REINICIO    el nodo muere —SIGKILL, y SIGTERM sin esperar a que el viejo termine— y se
#               rearranca al instante sobre el MISMO libro, $N veces cada una: tiene que volver
#               a contestar todas. Si alguna no, la 18 se reproduce y este banco lo dice ROJO.
#   DOS NODOS   un segundo nodo sobre el libro que otro tiene abierto se NIEGA, con el nombre del
#               problema (`un nodo por libro`) y sin tocar nada: el primero sigue contestando.
#
# ⚠️ Bajo $HOME, NUNCA en /tmp: el guardian del indice se niega donde `fsync` no persiste (§740).
set -uo pipefail
msg(){ echo "BANCO-REINICIO| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
N="${BANCO_REINICIO_N:-15}"
DIR=$(mktemp -d "$HOME/.banco_reinicio.XXXXXX") || { msg "no pude crear el directorio"; exit 2; }
P=""; P2=""
limpiar(){ for x in $P $P2; do kill -9 "$x" 2>/dev/null; done; rm -rf "$DIR"; }
trap limpiar EXIT

cd "$(dirname "$0")/.." || fallo "no encuentro la raiz"
msg "compilando el nodo (release)"
cargo build --release -q -p zk-ssl-node || cargo build --release -p zk-ssl-node || fallo "no compila"
NB=target/release/zk-ssl-node
[ -x "$NB" ] || fallo "falta $NB"
python3 -c "print('5b'*96, end='')" > "$DIR/semilla.hex" || fallo "semilla"
chmod 600 "$DIR/semilla.hex"

PORT=8619
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

arranca 0 || { sed -n '1,10p' "$DIR/err.0" >&2; fallo "el nodo no arranca ni la primera vez"; }
for SENAL in KILL TERM; do
  for i in $(seq 1 "$N"); do
    kill -"$SENAL" "$P"
    # Con SIGKILL se espera al viejo (el cerrojo es suyo hasta que muere); con SIGTERM, NO: es el
    # rearranque inmediato que la 18 temia.
    [ "$SENAL" = KILL ] && wait "$P" 2>/dev/null
    arranca "$SENAL$i" || { sed -n '1,5p' "$DIR/err.$SENAL$i" >&2;
      fallo "la 18 SE REPRODUCE: tras SIGTERM/SIGKILL ($SENAL, vuelta $i) el nodo no reabre su libro"; }
  done
  msg "REINICIO: $N de $N rearranques a la primera tras SIG$SENAL"
done

# DOS NODOS: el segundo, con sus propios contadores, sobre el MISMO libro.
"$NB" --listen "127.0.0.1:$((PORT+1))" --latido 1 --clave-fichero "$DIR/semilla.hex" --custodia fichero \
  --diario "$DIR/otro.jsonl" --ledger "$DIR/ledger" --contador-recepcion "$DIR/recepcion2.bin" \
  --indice-firma "$DIR/indice2.bin" --log warn > "$DIR/otro.out" 2>&1 &
P2=$!
for _ in $(seq 1 100); do kill -0 "$P2" 2>/dev/null || break; sleep 0.1; done
if kill -0 "$P2" 2>/dev/null; then fallo "un SEGUNDO nodo sigue vivo sobre el libro de otro"; fi
wait "$P2"; RC=$?; P2=""
[ "$RC" != "0" ] || fallo "el segundo nodo salio con 0 sobre un libro tomado"
grep -q 'un nodo por libro' "$DIR/otro.out" \
  || { sed -n '1,5p' "$DIR/otro.out" >&2; fallo "el segundo nodo se nego, pero sin decir por que"; }
vivo "$PORT" || fallo "el primer nodo dejo de contestar cuando el segundo intento abrir su libro"
msg "DOS NODOS: el segundo se niega con su nombre (exit $RC) y el primero sigue contestando"
msg "BANCO-REINICIO VERDE: un nodo reabre su libro a la primera; dos sobre el mismo, no"
