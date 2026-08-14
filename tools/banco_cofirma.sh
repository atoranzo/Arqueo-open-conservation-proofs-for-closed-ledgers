#!/usr/bin/env bash
# tools/banco_cofirma.sh — el banco de LA COFIRMA (§301).
#
# `banco_consistencia.sh` (§295) demostro que el testigo JUZGA la historia.
# Este demuestra que **la avala, y solo cuando debe**:
#
#   POSITIVO    nodo real + testigo real cofirmando: salen cofirmas, y
#               `--verificar-cofirmas` las da por buenas **sin el nodo, sin
#               el diario y sin el testigo**. Es el criterio del §300
#               —«la linea basta por si sola»— hecho EJECUTABLE (§301).
#   CONTRATO    las DOS LISTAS cuadran en vivo: cada `cofirmada` del diario
#               tiene su linea en el fichero, con el MISMO indice, y no hay
#               cofirmas huerfanas. En unitario ya estaba atado; aqui se
#               ejercita con ficheros de verdad.
#   NEGATIVO-A  **la regla, en vivo**: se resetea el nodo debajo del testigo
#               y las vueltas que NO son `nueva`+`extiende` **no cofirman**.
#               Un aval sobre algo que hizo saltar al testigo no es un aval,
#               y sin este tramo esa regla seria solo una frase.
#   NEGATIVO-B  una cofirma ADULTERADA (un nibble de la firma) muere en
#               `--verificar-cofirmas` con clase `no-verifica`. Sin su rojo,
#               un banco es un adorno.
#
# ⚠️ Bajo $HOME, NUNCA en /tmp: el guardian del indice se niega a operar
#    donde `fsync` no persiste (K.1, §234), y /tmp suele ser tmpfs.
# ⚠️ En un fallo se enseñan CLASES y CONTEOS, nunca los ficheros: una linea
#    de diario pesa ~37 KB y una cofirma ~18 KB mas.
set -uo pipefail

msg(){ printf 'BANCO-COFIRMA| %s\n' "$*" >&2; }
DIR=$(mktemp -d "$HOME/.banco_cofirma.XXXXXX") || { msg "no pude crear el directorio"; exit 2; }
NODO=""; TESTIGO=""
limpiar(){
  [ -n "$NODO" ] && kill "$NODO" 2>/dev/null
  [ -n "$TESTIGO" ] && kill "$TESTIGO" 2>/dev/null
  wait 2>/dev/null
  rm -rf "$DIR"
}
trap limpiar EXIT
fallo(){ msg "ROJO: $*"; exit 1; }

cd "$(dirname "$0")/.." || fallo "no encuentro la raiz"
# ⚠️ RELEASE, no debug: aqui el TESTIGO firma, y una firma XMSS mide 144,5 ms
#    en release (§292). En debug el banco tardaria minutos.
msg "compilando nodo y cli (release: el testigo FIRMA)"
cargo build --release -q -p zk-ssl-node -p zk-ssl-cli \
  || cargo build --release -p zk-ssl-node -p zk-ssl-cli || fallo "no compila"
NB=target/release/zk-ssl-node
CB=target/release/zk-ssl-cli
[ -x "$NB" ] || fallo "falta $NB"
[ -x "$CB" ] || fallo "falta $CB"

# ⚠️ Semillas DETERMINISTAS: un banco que dependa de /dev/urandom no se
#    puede volver a correr igual.
# ⚠️⚠️ Y en DOS FORMATOS, porque el proyecto los tiene: el NODO lee su clave
#    en HEX (`--clave-fichero` + `--custodia fichero`) y el TESTIGO en
#    BINARIO crudo (`--cofirmar`, que hace `fs::read`). **Esa divergencia no
#    es de este banco: es del proyecto**, y queda declarada en el asiento.
python3 -c "print('5b'*96, end='')" > "$DIR/semilla-nodo.hex" || fallo "semilla del nodo"
chmod 600 "$DIR/semilla-nodo.hex"
python3 -c "
import sys
sys.stdout.buffer.write(bytes(((i*31+9) % 256) for i in range(96)))
" > "$DIR/semilla-testigo.bin" || fallo "semilla del testigo"
chmod 600 "$DIR/semilla-testigo.bin"

PORT=8601
arranca_nodo(){  # $1 = sufijo del ledger/diario (1 = primero, 2 = tras el reseteo)
  "$NB" --listen "127.0.0.1:$PORT" --latido 1 \
    --clave-fichero "$DIR/semilla-nodo.hex" --custodia fichero \
    --diario "$DIR/nodo$1.jsonl" --ledger "$DIR/ledger$1" \
    --contador-recepcion "$DIR/recepcion$1.bin" \
    --indice-firma "$DIR/indice-firma.bin" \
    --log warn 2>>"$DIR/nodo.err" &
  NODO=$!
  for _ in $(seq 1 40); do
    R=$(curl -s --max-time 10 "http://127.0.0.1:$PORT" \
        -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","id":1,"method":"zkssl_supply","params":{}}' 2>/dev/null || true)
    case "$R" in *result*) return 0;; esac
    sleep 0.5
  done
  sed -n '1,20p' "$DIR/nodo.err" >&2
  fallo "el nodo no arranco en 20 s"
}

testigo(){  # $1=veces
  "$CB" witness --nodo "http://127.0.0.1:$PORT" --cada 1 --veces "$1" --no-color \
        --diario "$DIR/diario.jsonl" \
        --cofirmar "$DIR/semilla-testigo.bin" \
        --indice-cofirma "$DIR/contador.bin" \
        --cofirmas "$DIR/cofirmas.jsonl"
}

# ══ POSITIVO ══════════════════════════════════════════════════════════
msg "POSITIVO: nodo con latido 1, testigo cofirmando 14 vueltas"
arranca_nodo 1
testigo 14 > "$DIR/t1.out" 2>&1
RC=$?
[ "$RC" = "0" ] || { tail -20 "$DIR/t1.out" >&2; fallo "el testigo salio con $RC"; }
[ -s "$DIR/cofirmas.jsonl" ] || { tail -20 "$DIR/t1.out" >&2; fallo "no se emitio ni una cofirma"; }
N=$(wc -l < "$DIR/cofirmas.jsonl" | tr -d ' ')
msg "POSITIVO: $N cofirma(s) emitidas"

# ⚠️ EL CRITERIO, EJECUTABLE: un tercero con SOLO el fichero.
"$CB" witness --verificar-cofirmas "$DIR/cofirmas.jsonl" > "$DIR/v1.out" 2>&1 \
  || { cat "$DIR/v1.out" >&2; fallo "las cofirmas emitidas NO verifican"; }
msg "POSITIVO: $(tail -1 "$DIR/v1.out")"

# ══ CONTRATO · las dos listas, en vivo ════════════════════════════════
python3 - "$DIR/diario.jsonl" "$DIR/cofirmas.jsonl" <<'PYEOF' >&2 || fallo "las dos listas NO cuadran"
import json, sys
dia, cof = sys.argv[1], sys.argv[2]
marcas = []
for l in open(dia, encoding="utf-8"):
    v = json.loads(l)
    if "cofirmada" in v:
        marcas.append(v["cofirmada"]["indice"])
idx = [json.loads(l)["indice"] for l in open(cof, encoding="utf-8")]
assert marcas == idx, "marcas del diario %r != indices del fichero %r" % (marcas, idx)
assert len(set(idx)) == len(idx), "un indice repetido en el fichero: %r" % idx
print("BANCO-COFIRMA| CONTRATO: %d marca(s) en el diario == %d linea(s) de cofirma,"
      " mismos indices y ninguno repetido" % (len(marcas), len(idx)))
PYEOF

# ══ NEGATIVO-A · la regla, en vivo ════════════════════════════════════
msg "NEGATIVO-A: el nodo se resetea debajo del testigo; las vueltas que no"
msg "            son nueva+extiende NO deben cofirmar"
ANTES=$(wc -l < "$DIR/cofirmas.jsonl" | tr -d ' ')
kill "$NODO" 2>/dev/null; wait "$NODO" 2>/dev/null; NODO=""
sleep 1
arranca_nodo 2                      # ledger y diario NUEVOS: el nodo va por detras
testigo 8 > "$DIR/t2.out" 2>&1 || true
DESPUES=$(wc -l < "$DIR/cofirmas.jsonl" | tr -d ' ')

python3 - "$DIR/diario.jsonl" "$DIR/cofirmas.jsonl" "$ANTES" <<'PYEOF' >&2 || fallo "la regla no se cumplio"
import json, sys
dia, cof, antes = sys.argv[1], sys.argv[2], int(sys.argv[3])
filas = [json.loads(l) for l in open(dia, encoding="utf-8")]

def cons(f):
    """⚠️ `consistencia` es un OBJETO con su `clase` dentro, no una cadena.
    Asi lo lee el analizador de `banco_consistencia.sh` (§295), y asi lo
    escribe `linea_de_diario_con`. Compararlo como cadena deja la lista de
    vueltas debidas VACIA y hace que TODAS las cofirmas parezcan indebidas."""
    c = f.get("consistencia")
    return c.get("clase") if isinstance(c, dict) else None

# ⚠️ La invariante EXACTA: cofirma <=> la vuelta fue `nueva` Y `extiende`.
debidas = [f for f in filas if f.get("clase") == "nueva" and cons(f) == "extiende"]
marcadas = [f for f in filas if "cofirmada" in f]
sobra = [f for f in marcadas if f not in debidas]
falta = [f for f in debidas if f not in marcadas]
assert not sobra, ("%d cofirma(s) sobre vueltas que NO eran nueva+extiende: %r"
                   % (len(sobra), [(f.get("clase"), cons(f)) for f in sobra]))
assert not falta, ("%d vuelta(s) nueva+extiende SIN cofirmar: %r"
                   % (len(falta), [(f.get("clase"), cons(f)) for f in falta]))
anomalas = [f for f in filas if cons(f) in ("por-detras", "consistencia-pendiente",
                                            "sin-camino", "no-extiende")]
assert not [f for f in anomalas if "cofirmada" in f], "se cofirmo una vuelta con ANOMALIA"
n = len([l for l in open(cof, encoding="utf-8")])
print("BANCO-COFIRMA| NEGATIVO-A: %d vuelta(s) con anomalia, NINGUNA cofirmada."
      " Cofirmas %d -> %d" % (len(anomalas), antes, n))
PYEOF

# ══ NEGATIVO-B · una cofirma adulterada ═══════════════════════════════
msg "NEGATIVO-B: se adultera UN nibble de una firma en el fichero"
python3 - "$DIR/cofirmas.jsonl" "$DIR/adulterado.jsonl" <<'PYEOF' || fallo "no pude adulterar"
import json, sys
ls = [json.loads(l) for l in open(sys.argv[1], encoding="utf-8")]
assert ls, "no hay cofirmas que adulterar"
f = list(ls[-1]["firma"])
i = 20
f[i] = "b" if f[i] == "a" else "a"
ls[-1]["firma"] = "".join(f)
with open(sys.argv[2], "w", encoding="utf-8") as w:
    for x in ls:
        w.write(json.dumps(x) + "\n")
PYEOF
"$CB" witness --verificar-cofirmas "$DIR/adulterado.jsonl" > "$DIR/v2.out" 2>&1
RC=$?
[ "$RC" != "0" ] || { cat "$DIR/v2.out" >&2; fallo "una cofirma ADULTERADA paso por buena"; }
grep -q 'no-verifica' "$DIR/v2.out" || { cat "$DIR/v2.out" >&2; fallo "murio, pero no por 'no-verifica'"; }
msg "NEGATIVO-B: exit $RC — clase 'no-verifica', como debe"

# ══ NEGATIVO-B2 · un indice repetido ══════════════════════════════════
# ⚠️ Reusar un indice XMSS filtra la clave. El guardian lo impide DENTRO;
#    esto comprueba que un tercero lo ve DESDE FUERA.
head -1 "$DIR/cofirmas.jsonl" > "$DIR/repetido.jsonl"
head -1 "$DIR/cofirmas.jsonl" >> "$DIR/repetido.jsonl"
"$CB" witness --verificar-cofirmas "$DIR/repetido.jsonl" > "$DIR/v3.out" 2>&1
RC=$?
[ "$RC" != "0" ] || { cat "$DIR/v3.out" >&2; fallo "un indice REPETIDO paso por bueno"; }
grep -q 'indice-repetido' "$DIR/v3.out" || { cat "$DIR/v3.out" >&2; fallo "murio, pero no por 'indice-repetido'"; }
msg "NEGATIVO-B2: exit $RC — clase 'indice-repetido', vista desde fuera"

msg "BANCO-COFIRMA VERDE: el testigo avala lo que juzga, solo cuando debe,"
msg "                     y un tercero lo comprueba con solo el fichero"
