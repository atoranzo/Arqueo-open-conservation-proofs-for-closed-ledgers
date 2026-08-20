#!/usr/bin/env bash
# tools/banco_reutilizacion.sh — el banco de LA NEGATIVA A REUTILIZAR (§331).
#
# `banco_cofirma.sh` (§301) asierta el CONTRATO de las cofirmas. Este asierta
# otra cosa y por eso es otro banco: **que el firmante se NIEGUE a arrancar
# cuando su clave vuelve a cero con el contador vivo**. Mezclar los dos
# invariantes haria que un rojo no dijera cual de los dos fallo.
#
#   POSITIVO   un proceso limpio cofirma: sin esto, el negativo es vacio.
#   NEGATIVO   un SEGUNDO proceso con la MISMA semilla y el MISMO contador
#              **no arranca**, y muere por `ClaveEnCero`. Antes del §331
#              arrancaba y volvia a firmar con indices ya quemados.
#
# ⚠️ Nada se rodea: el estado no se fabrica, lo produce un reinicio normal.
# ⚠️ Bajo $HOME, NUNCA en /tmp: el guardian se niega donde `fsync` no persiste.
set -uo pipefail
msg(){ printf 'BANCO-REUSO| %s\n' "$*" >&2; }
DIR=$(mktemp -d "$HOME/.banco_reuso.XXXXXX") || { msg "no pude crear el directorio"; exit 2; }
NODO=""
limpiar(){ [ -n "$NODO" ] && kill "$NODO" 2>/dev/null; wait 2>/dev/null; rm -rf "$DIR"; }
trap limpiar EXIT
fallo(){ msg "ROJO: $*"; exit 1; }
cd "$(dirname "$0")/.." || fallo "no encuentro la raiz"
cargo build --release -p zk-ssl-node -p zk-ssl-cli >/dev/null 2>&1 || fallo "no compila"
NB=target/release/zk-ssl-node; CB=target/release/zk-ssl-cli
[ -x "$NB" ] && [ -x "$CB" ] || fallo "faltan los binarios"
python3 -c "print('5b'*96, end='')" > "$DIR/semilla-nodo.hex"; chmod 600 "$DIR/semilla-nodo.hex"
python3 -c "
import sys
sys.stdout.buffer.write(bytes(((i*31+9) % 256) for i in range(96)))
" > "$DIR/semilla-testigo.bin"; chmod 600 "$DIR/semilla-testigo.bin"
PORT=8613
"$NB" --listen "127.0.0.1:$PORT" --latido 1 \
  --clave-fichero "$DIR/semilla-nodo.hex" --custodia fichero \
  --diario "$DIR/nodo.jsonl" --ledger "$DIR/ledger" \
  --contador-recepcion "$DIR/recepcion.bin" --indice-firma "$DIR/indice-firma.bin" \
  --log warn 2>>"$DIR/nodo.err" &
NODO=$!
for _ in $(seq 1 40); do
  R=$(curl -s --max-time 10 "http://127.0.0.1:$PORT" -H 'Content-Type: application/json' \
      -d '{"jsonrpc":"2.0","id":1,"method":"zkssl_supply","params":{}}' 2>/dev/null || true)
  case "$R" in *result*) break;; esac
  sleep 0.5
done
testigo(){ "$CB" witness --nodo "http://127.0.0.1:$PORT" --cada 1 --veces "$1" --no-color \
    --diario "$DIR/diario.jsonl" --cofirmar "$DIR/semilla-testigo.bin" \
    --indice-cofirma "$DIR/contador.bin" --cofirmas "$DIR/cofirmas.jsonl" > "$2" 2>&1; }

# ══ POSITIVO ══ sin esto, el negativo no prueba nada
msg "POSITIVO: proceso limpio, 14 vueltas"
testigo 14 "$DIR/t1.out" || { tail -20 "$DIR/t1.out" >&2; fallo "el primer testigo salio con error"; }
[ -s "$DIR/cofirmas.jsonl" ] || { tail -20 "$DIR/t1.out" >&2; fallo "no se emitio ni una cofirma: sube --veces"; }
N=$(wc -l < "$DIR/cofirmas.jsonl" | tr -d ' ')
C=$(python3 -c "print(int.from_bytes(open('$DIR/contador.bin','rb').read(),'little'))")
msg "POSITIVO: $N cofirma(s), contador en $C"
[ "$C" -ge 1 ] || fallo "el contador no avanzo: el estado del negativo no existe"

# ══ NEGATIVO ══ el mismo contador y la misma semilla, proceso NUEVO
msg "NEGATIVO: segundo proceso con el MISMO contador y la MISMA semilla"
testigo 3 "$DIR/t2.out"; RC=$?
[ "$RC" != "0" ] || { tail -20 "$DIR/t2.out" >&2; fallo "ARRANCO: la clave vuelve a cero y firmaria indices ya quemados"; }
grep -q 'clave en cero' "$DIR/t2.out" \
  || { tail -20 "$DIR/t2.out" >&2; fallo "murio, pero NO por 'clave en cero': $RC"; }
grep -q 'INDETERMINADOS' "$DIR/t2.out" \
  || { tail -20 "$DIR/t2.out" >&2; fallo "el motivo no nombra los indices indeterminados"; }
msg "NEGATIVO: exit $RC — 'clave en cero', y el motivo lo nombra"

# ⚠️ Y no ha escrito ni una cofirma mas: negarse es negarse.
N2=$(wc -l < "$DIR/cofirmas.jsonl" | tr -d ' ')
[ "$N2" = "$N" ] || fallo "el segundo proceso escribio $((N2-N)) cofirma(s) pese a negarse"
msg "NEGATIVO: cero cofirmas nuevas ($N antes, $N2 despues)"
msg "BANCO-REUSO VERDE: un firmante cuya clave vuelve a cero NO arranca"
