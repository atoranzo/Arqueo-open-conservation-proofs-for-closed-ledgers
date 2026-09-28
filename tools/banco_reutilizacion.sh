#!/usr/bin/env bash
# tools/banco_reutilizacion.sh — el banco de LA NEGATIVA A REUTILIZAR (§331).
#
# `banco_cofirma.sh` (§301) asierta el CONTRATO de las cofirmas. Este asierta
# otra cosa y por eso es otro banco: **que el firmante NO VUELVA A FIRMAR un
# indice ya gastado** cuando su clave vuelve a cero con el contador vivo.
# Mezclar los dos invariantes haria que un rojo no dijera cual de los dos fallo.
#
#   POSITIVO   un proceso limpio cofirma: sin esto, los negativos son vacios.
#   REINICIO   un SEGUNDO proceso con la MISMA semilla y el MISMO contador
#              RESINCRONIZA la clave hasta el contador y sigue, y ninguna
#              cofirma nueva repite un indice de las viejas (§337).
#   NEGATIVO   el contador RESTAURADO hacia atras, con las cofirmas en su
#              sitio: **falla cerrada**, nombrando el retroceso, y no escribe
#              ni una cofirma (§337: el borde es `<=`, derivado).
#
# ⚠️ CORRECCION (§579): hasta el §337 el NEGATIVO de este banco era «un segundo
# proceso con el mismo contador NO arranca» -la tesis del §331-. El §337 la
# cambio A PROPOSITO, porque negarse dejaba inservible a todo testigo tras su
# primera cofirma, y cambio su test; este banco no se toco y estaba ROJO desde
# entonces, sin que nadie lo corriera. Medido en el §579.
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

# Los indices de las cofirmas que imprimio una corrida: `[n] cofirmada · indice 0x..`.
indices(){ grep -o 'cofirmada · indice 0x[0-9a-f]*' "$1" | sed 's/.*0x/0x/' | python3 -c "
import sys; print(' '.join(str(int(l, 16)) for l in sys.stdin.read().split()))"; }

# ══ REINICIO ══ el mismo contador y la misma semilla, proceso NUEVO: resincroniza (§337)
msg "REINICIO: segundo proceso con el MISMO contador y la MISMA semilla"
testigo 3 "$DIR/t2.out" || { tail -20 "$DIR/t2.out" >&2; fallo "el segundo proceso no arranco: el §337 lo resincroniza"; }
grep -q 'clave resincronizada en el indice' "$DIR/t2.out" \
  || { tail -20 "$DIR/t2.out" >&2; fallo "arranco sin decir que resincronizo la clave"; }
N2=$(wc -l < "$DIR/cofirmas.jsonl" | tr -d ' ')
[ "$N2" -gt "$N" ] || { tail -20 "$DIR/t2.out" >&2; fallo "resincronizo y no cofirmo: el reinicio no se ejercito"; }
I1=$(indices "$DIR/t1.out"); I2=$(indices "$DIR/t2.out")
python3 -c "
a, b = [int(x) for x in '$I1'.split()], [int(x) for x in '$I2'.split()]
assert a and b, 'sin indices que comparar'
assert min(b) > max(a), 'REUTILIZA: la cofirma nueva %d no pasa de la vieja %d' % (min(b), max(a))
" || fallo "una cofirma del segundo proceso repite o baja de un indice ya gastado ($I1 | $I2)"
msg "REINICIO: resincronizo y cofirmo $((N2-N)) mas; indices viejos [$I1], nuevos [$I2]: ninguno repite"

# ══ NEGATIVO ══ el contador RESTAURADO hacia atras, con las cofirmas en su sitio: falla cerrada
python3 -c "
p = '$DIR/contador.bin'; b = open(p, 'rb').read()
open(p, 'wb').write((1).to_bytes(len(b), 'little'))"
msg "NEGATIVO: el contador restaurado a 1, con las cofirmas del indice $(echo $I2 | tr ' ' '\n' | tail -1) en su sitio"
testigo 3 "$DIR/t3.out"; RC=$?
[ "$RC" != "0" ] || { tail -20 "$DIR/t3.out" >&2; fallo "ARRANCO con el contador hacia atras: firmaria una hoja ya gastada"; }
grep -q 'EL CONTADOR HA RETROCEDIDO' "$DIR/t3.out" \
  || { tail -20 "$DIR/t3.out" >&2; fallo "murio, pero NO por el retroceso del contador: $RC"; }
N3=$(wc -l < "$DIR/cofirmas.jsonl" | tr -d ' ')
[ "$N3" = "$N2" ] || fallo "el tercer proceso escribio $((N3-N2)) cofirma(s) pese a negarse"
msg "NEGATIVO: exit $RC, 'EL CONTADOR HA RETROCEDIDO', y cero cofirmas nuevas ($N2 antes, $N3 despues)"
msg "BANCO-REUSO VERDE: tras reiniciar, el firmante resincroniza y NO repite un indice; con el contador hacia atras, NO arranca"
