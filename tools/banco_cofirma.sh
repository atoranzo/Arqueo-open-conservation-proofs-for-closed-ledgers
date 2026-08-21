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
#   NEGATIVO-B2 (§334) duplicar una linea NO acusa a nadie: es la misma
#               firma sobre el MISMO mensaje. Antes exigia `indice-repetido`,
#               y por eso cualquiera podia desacreditar a un cofirmante
#               honesto duplicandole una linea.
#   NEGATIVO-B3 (§334) el mismo indice sobre OTRO mensaje SI acusa: se
#               reescribe la clave del operador de una copia -mismo indice
#               embebido, preambulo distinto- y sale `indice-repetido`. Es
#               el caso que de verdad quema una clave de un solo uso.
#   ENVIO       (§316) el testigo ENVIA la cofirma al nodo y el nodo la
#               SIRVE. Cuatro pasos: el nodo virgen da `n:0` —el rojo—;
#               el testigo con `--enviar-cofirmas` obtiene aceptaciones y
#               ningun rechazo —**el camino de ACEPTACION, que es lo que
#               el §315 dejo declarado y sin cubrir**—; pedir la epoca de
#               la ultima cofirma la devuelve —la RETENCION—; y sin
#               parametros se MIDE lo que sale, sin asertarlo.
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
                 # $2 = latido en segundos. Default 1: las dos llamadas
                 #      que ya existian no cambian de comportamiento.
  "$NB" --listen "127.0.0.1:$PORT" --latido "${2:-1}" \
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
            # $2=flags extra, opcionales. \u26a0 Van FUERA de la lista fija
            #    a proposito: los cinco tramos llaman a esta funcion, y
            #    encender aqui una escritura de red cambiaria lo que
            #    NEGATIVO-A mide.
  # shellcheck disable=SC2086
  "$CB" witness --nodo "http://127.0.0.1:$PORT" --cada 1 --veces "$1" --no-color ${2:-} \
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
                                            "sin-camino", "consistencia-sin-respuesta",
                                            "no-extiende")]
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

# ══ NEGATIVO-B2 · una linea duplicada NO acusa ════════════════════════
# ⚠️⚠️ §334 · esto duplicaba la primera linea y EXIGIA `indice-repetido`.
#    Pero la misma linea es la misma firma sobre el MISMO mensaje: no revela
#    nada, y quemaba a un cofirmante honesto. Ahora se comprueba que NO acusa.
head -1 "$DIR/cofirmas.jsonl" > "$DIR/duplicado.jsonl"
head -1 "$DIR/cofirmas.jsonl" >> "$DIR/duplicado.jsonl"
"$CB" witness --verificar-cofirmas "$DIR/duplicado.jsonl" > "$DIR/v3.out" 2>&1
RC=$?
[ "$RC" = "0" ] || { cat "$DIR/v3.out" >&2; fallo "duplicar una linea ACUSA a alguien"; }
grep -q 'sin hallazgos' "$DIR/v3.out" || { cat "$DIR/v3.out" >&2; fallo "salio 0 pero no dijo 'sin hallazgos'"; }
msg "NEGATIVO-B2: exit $RC — duplicar una linea NO acusa a nadie"

# ══ NEGATIVO-B3 · el mismo indice sobre OTRO mensaje ══════════════════
# ⚠️ Reusar un indice XMSS filtra la clave. El guardian lo impide DENTRO;
#    esto comprueba que un tercero lo ve DESDE FUERA.
# ⚠️⚠️ Lo que se reescribe es la CLAVE DEL OPERADOR y no el epochDigest:
#    el mensaje firmado es el PREAMBULO entero, asi que basta con eso para
#    que sean dos mensajes distintos bajo el MISMO indice embebido. Molde:
#    el adulterador del NEGATIVO-B, unas lineas mas arriba.
python3 - "$DIR/cofirmas.jsonl" "$DIR/repetido.jsonl" <<'PYEOF' || fallo "no pude forjar la copia"
import json, sys
ls = [json.loads(l) for l in open(sys.argv[1], encoding="utf-8") if l.strip()]
uno = ls[0]
otro = dict(uno)
otro["clavePublicaOperador"] = "0xdeadbeef"
with open(sys.argv[2], "w", encoding="utf-8") as w:
    w.write(json.dumps(uno) + "\n")
    w.write(json.dumps(otro) + "\n")
PYEOF
"$CB" witness --verificar-cofirmas "$DIR/repetido.jsonl" > "$DIR/v4.out" 2>&1
RC=$?
[ "$RC" != "0" ] || { cat "$DIR/v4.out" >&2; fallo "el mismo indice sobre OTRO mensaje paso por bueno"; }
grep -q 'indice-repetido' "$DIR/v4.out" || { cat "$DIR/v4.out" >&2; fallo "murio, pero no por 'indice-repetido'"; }
msg "NEGATIVO-B3: exit $RC — clase 'indice-repetido', vista desde fuera"

# ══ ENVIO · el testigo SUBMITE y el nodo SIRVE (§316) ═════════════
#
# ⚠⚠ DOS numeros que se tiran el uno del otro, y el r1 solo miro uno.
#    LA VENTANA: con `--latido 1` una cofirma puede quedar a caballo de un
#    latido —firmar XMSS cuesta ~145 ms (§292)— y el nodo la rechaza por
#    «no es de la epoca en curso»; un latido mas largo la ensancha.
#    EL CALENTAMIENTO: el testigo no cofirma hasta `Nueva`+`Extiende`, y
#    eso son ~6 latidos —anclar, y esperar la cabeza que firma el mmrSize—.
#    MEDIDO aqui al lado: POSITIVO da 5 cofirmas en 14 vueltas a latido 1, y
#    NEGATIVO-A da 2 en 8 sobre nodo fresco. Alargar el latido ENSANCHA la
#    ventana y ALARGA el calentamiento en la misma proporcion.
#    ⇒ latido 4 y 40 vueltas: ~10 latidos, calentamiento holgado, y una
#      ventana 27 veces mayor que la firma.
msg "ENVIO: nodo nuevo con latido 4; el testigo envia sus cofirmas en 40 vueltas"
kill "$NODO" 2>/dev/null; wait "$NODO" 2>/dev/null; NODO=""
sleep 1
arranca_nodo 3 4

pregunta_cosigs(){  # $1 = el epochDigest, o NADA para preguntar por la actual
  # ⚠⚠ NI UNA COMILLA DE JSON ESCRITA A MANO. El r3 murio porque el
  #    cuerpo pasaba por cuatro capas de escapado —parche python, cadena
  #    python del bloque, bash, y el JSON— y se perdio una. `json.dumps`
  #    escapa lo que haga falta y curl lo manda desde fichero, sin releerlo.
  python3 - "${1:-}" > "$DIR/peticion.json" <<'PYEOF'
import json, sys
d = sys.argv[1] if len(sys.argv) > 1 else ""
p = {"epochDigest": d} if d else None
print(json.dumps({"jsonrpc": "2.0", "id": 9,
                  "method": "zkssl_cosigs", "params": p}))
PYEOF
  curl -s --max-time 10 "http://127.0.0.1:$PORT"     -H 'Content-Type: application/json'     -d @"$DIR/peticion.json" 2>/dev/null || true
}

# ── (A) EL ROJO, y va PRIMERO ─────────────────────────────
# Un banco sin su rojo es un adorno, y este es gratis: nadie ha enviado
# nada a este nodo todavia.
R0=$(pregunta_cosigs)
python3 - "$R0" <<'PYEOF' >&2 || fallo "el nodo virgen no dijo n:0"
import json, sys
def q(x):
    """\u26a0 `Q` viaja como cadena hex en el cable, pero esto no depende de
    ello: acepta entero y cadena, con `0x` o sin el. Un banco que se rompa
    por la REPRESENTACION estaria midiendo otra cosa."""
    if isinstance(x, bool): raise TypeError("bool no es Q")
    if isinstance(x, int): return x
    s = str(x)
    return int(s, 16) if s.lower().startswith("0x") else int(s)
d = json.loads(sys.argv[1])
if "error" in d:
    sys.exit("el nodo devolvio error a zkssl_cosigs: %r" % (d["error"],))
v = d["result"]
n = q(v["n"])
assert n == 0, "un nodo sin envios debe dar n:0 y dio %d" % n
print("BANCO-COFIRMA| ENVIO-A (el rojo): nodo virgen, zkssl_cosigs da n:0")
PYEOF

# ── (B) LA ACEPTACION ──────────────────────────────
# ⚠⚠ Esto es lo unico que el §315 dejo DECLARADO Y SIN CUBRIR: sus tres
#    tests del nodo miden los RECHAZOS, porque aceptar exige fabricar una
#    cofirma XMSS valida y el cofirmante vive en el testigo. Aqui hay
#    testigo de verdad, asi que aqui se cubre.
ANTES_E=$(wc -l < "$DIR/cofirmas.jsonl" | tr -d ' ')
testigo 40 --enviar-cofirmas > "$DIR/t3.out" 2>&1
RC=$?
[ "$RC" = "0" ] || { tail -20 "$DIR/t3.out" >&2; fallo "el testigo salio con $RC"; }
DESPUES_E=$(wc -l < "$DIR/cofirmas.jsonl" | tr -d ' ')
COF=$((DESPUES_E - ANTES_E))
ENV=$(grep -c 'cofirma enviada' "$DIR/t3.out" || true)
TARDE=$(grep -c 'no es de la epoca en curso' "$DIR/t3.out" || true)
DURO=$(grep -cE 'dio ERROR|no se pudo enviar' "$DIR/t3.out" || true)
msg "ENVIO-B: $COF cofirmada(s) · $ENV enviada(s) · $TARDE tarde · $DURO dura(s)"

# ⚠⚠ EL INSTRUMENTO DISTINGUE LOS DOS FALLOS (§254). El r1 dijo solo
#    «ninguna cofirma llego a enviarse», y eso no separa «el tramo no da
#    tiempo a cofirmar» de «cofirma y el flag no llega al bucle». Son
#    causas distintas y arreglos distintos.
[ "$COF" -ge 1 ] || { tail -20 "$DIR/t3.out" >&2; fallo "el testigo no llego a COFIRMAR en $COF vuelta(s) utiles: es DIMENSIONADO del tramo (mas --veces o menos latido), NO el envio"; }
[ "$ENV" -ge 1 ] || { tail -20 "$DIR/t3.out" >&2; fallo "cofirmo $COF y no envio NINGUNA: el flag no esta llegando al bucle"; }
[ "$DURO" = "0" ] || { grep -E 'dio ERROR|no se pudo enviar' "$DIR/t3.out" | head -5 >&2; fallo "$DURO fallo(s) DUROS de envio: transporte o error RPC"; }

# ⚠ Un rechazo por «no es de la epoca en curso» NO es defecto: es la
#   cofirma que cruzo un latido mientras se firmaba. Se exige que sea la
#   EXCEPCION y no la norma — un gate a cero aqui seria mas estricto que
#   la realidad y daria rojo por algo legitimo.
[ "$TARDE" -lt "$ENV" ] || fallo "mas rechazos por epoca ($TARDE) que aceptaciones ($ENV): eso ya no es cruzar un latido"

# ⚠⚠ EL DATO QUE YA VIAJABA Y NADIE MIRABA: el nodo contesta cuantas
#    tiene GUARDADAS, y el testigo lo imprime. Si la ultima aceptacion dijo
#    `guardadas 1`, el almacen TENIA la cofirma en ese instante — y
#    entonces cualquier fallo posterior es de la PREGUNTA, no del almacen.
GUARD=$(grep 'cofirma enviada' "$DIR/t3.out" | tail -1 | sed 's/.*guardadas //' | tr -d '"' || true)
msg "ENVIO-B: el nodo dijo guardadas='${GUARD:-?}' en la ultima aceptacion"

# ── (C) EL SERVICIO, y con el la RETENCION ──────────────────
# ⚠⚠ Se pide POR `epochDigest`, no sin parametros: el almacen del nodo
#    solo se purga al recibir un envio de OTRA epoca —y un envio rechazado
#    vuelve ANTES del `retain`—, asi que la cofirma de la epoca X sigue
#    ahi. **Es la RETENCION**, y el §317 la ata en el nodo y en
#    `spec/RPC.md`, que hoy publican lo contrario.
#
# ⚠⚠⚠ ESTE TRAMO IMPRIME LO QUE HACE. El r2 murio aqui sin decir una
#    palabra, y tres causas distintas dan el mismo silencio. Cada paso deja
#    su rastro ANTES de juzgar: si vuelve a caer, se sabra por que.

# sonda de vida: separa «el nodo no contesta» de todo lo demas
VIVO=$(pregunta_cosigs)
[ -n "$VIVO" ] || fallo "el nodo no contesta a zkssl_cosigs: curl devolvio VACIO"
msg "ENVIO-C: sonda de vida -> $(printf '%.140s' "$VIVO")"

NL=$(wc -l < "$DIR/cofirmas.jsonl" | tr -d ' ')
msg "ENVIO-C: el fichero de cofirmas lleva $NL linea(s)"

SIRVE=""; RSIRVE=""
for k in 1 2 3 ; do
  DIG=$(python3 - "$DIR/cofirmas.jsonl" "$k" <<'PYEOF' 2>&1 || true
import json, sys
try:
    ls = [json.loads(l) for l in open(sys.argv[1], encoding="utf-8") if l.strip()]
except Exception as e:
    print("ILEGIBLE:%s" % e); raise SystemExit(0)
i = len(ls) - int(sys.argv[2])
print(ls[i].get("epochDigest", "SIN-CAMPO") if i >= 0 else "")
PYEOF
)
  msg "ENVIO-C: k=$k digest='$(printf '%.20s' "$DIG")' longitud=${#DIG}"
  case "$DIG" in
    "")            msg "ENVIO-C: k=$k no hay linea -$k en el fichero"; break ;;
    ILEGIBLE:*)    fallo "el fichero de cofirmas no se deja leer: $DIG" ;;
    SIN-CAMPO)     fallo "la linea -$k no lleva epochDigest" ;;
  esac
  R1=$(pregunta_cosigs "$DIG")
  msg "ENVIO-C: k=$k respuesta -> $(printf '%.160s' "${R1:-<VACIA>}")"
  VER=$(python3 - "${R1:-}" <<'PYEOF' 2>&1 || true
import json, sys
crudo = sys.argv[1] if len(sys.argv) > 1 else ""
if not crudo.strip():
    print("VACIA"); raise SystemExit(0)
try:
    d = json.loads(crudo)
except Exception as e:
    print("NO-JSON:%s" % e); raise SystemExit(0)
# ⚠⚠ `error` NO es `n:0`. Confundirlos es el defecto que este mismo
#    corte le reprocha al testigo, y que yo cometi en el r2.
if "error" in d:
    print("ERROR-RPC:%s" % json.dumps(d["error"])[:120]); raise SystemExit(0)
v = d.get("result")
if v is None:
    print("SIN-RESULT"); raise SystemExit(0)
x = v.get("n", 0)
n = int(str(x), 16) if str(x).lower().startswith("0x") else int(x)
print("N:%d" % n)
PYEOF
)
  msg "ENVIO-C: k=$k veredicto=$VER"
  case "$VER" in
    ERROR-RPC:*) fallo "el nodo RECHAZO la pregunta ($VER): no es el almacen, es la peticion" ;;
    N:1)         SIRVE="$DIG"; RSIRVE="$R1"
                 msg "ENVIO-C: la cofirma -$k desde el final es la que el nodo retiene"
                 break ;;
  esac
done
[ -n "$SIRVE" ] || fallo "ninguna de las tres ultimas cofirmas esta en el nodo: la RETENCION no se cumple"

TG=$(python3 - "$DIR/cofirmas.jsonl" <<'PYEOF' || true
import json, sys
ls = [json.loads(l) for l in open(sys.argv[1], encoding="utf-8") if l.strip()]
print(ls[-1]["clavePublicaTestigo"])
PYEOF
)
[ -n "$TG" ] || fallo "no pude leer la clave del testigo"

python3 - "$RSIRVE" "$TG" <<'PYEOF' >&2 || fallo "el nodo sirve OTRA cofirma"
import json, sys
def hx(s):
    s = str(s).lower()
    return s[2:] if s.startswith("0x") else s
v = json.loads(sys.argv[1])["result"]
servida = hx(v["cosigs"][0]["clavePublicaTestigo"])
assert servida == hx(sys.argv[2]), "el nodo sirve OTRA clave de testigo"
print("BANCO-COFIRMA| ENVIO-C: n:1 y la clave del testigo cuadra"
      " — la RETENCION, en vivo")
PYEOF

# ── (D) LA VENTANA: se MIDE, no se asierta ──────────────────
# ⚠⚠ CORRECCION MEDIDA EN EL r3: yo daba por hecho que aqui saldria 0
#    —«la epoca ya avanzo»— y es FALSO con este latido: la sonda del r3
#    fue sin parametros y el nodo devolvio la cofirma. Con latido 4 la
#    pregunta cae DENTRO de la misma epoca que la ultima cofirma, que es
#    justo el flujo natural de un cliente. Se ENSEÑA lo que salga y no se
#    exige: el numero depende del latido y del momento, no de una regla.
R2=$(pregunta_cosigs)
python3 - "$R2" <<'PYEOF' >&2 || true
import json, sys
def q(x):
    if isinstance(x, int): return x
    s = str(x)
    return int(s, 16) if s.lower().startswith("0x") else int(s)
d = json.loads(sys.argv[1])
v = d.get("result") or {}
print("BANCO-COFIRMA| ENVIO-D (medido, no asertado): sin parametros, la"
      " epoca EN CURSO da n:%s" % q(v.get("n", 0)))
PYEOF

msg "BANCO-COFIRMA VERDE: el testigo avala lo que juzga, solo cuando debe,"
msg "                     y un tercero lo comprueba con solo el fichero"
