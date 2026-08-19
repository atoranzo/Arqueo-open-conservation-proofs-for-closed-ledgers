#!/usr/bin/env bash
# tools/banco_evidencia_v2.sh - el banco del PAQUETE DE EVIDENCIA v2 (322).
#
# banco_apagado.sh (290) demostro que una posicion se sostiene SIN el nodo.
# banco_cofirma.sh (301) demostro que el testigo avala, y solo cuando debe.
# Este demuestra lo que el 322 anadio: que las COFIRMAS viajan DENTRO del
# paquete y las verifica un tercero sin el nodo, sin el testigo y sin el
# diario.
#
#   ENVIO-A     el rojo primero, y es gratis: nodo virgen, zkssl_cosigs n:0.
#   POSITIVO    testigo en SEGUNDO PLANO cofirmando y enviando; se sondea el
#               PAR (cabeza, cofirmas de SU epoca) hasta que casa; el paquete
#               v2 se arma con result.cosigs TAL CUAL -sin reescribir nada,
#               que quien reescribe adultera- y zk-ssl-verify lo da por bueno.
#   NEGATIVO-1  un v1 que trae cofirmas se RECHAZA. La compuerta existe en
#               el binario y NO la cubre ningun test.
#   NEGATIVO-2  un v3 desconocido se niega en voz alta. Idem.
#   NEGATIVO-3  un nibble de la firma de una cofirma: exit 1 nombrando cual.
#               Es el unico que exige testigo vivo, y el que demuestra que
#               la criptografia corre de verdad.
#
# ALCANCE, declarado: este banco es el MINIMO -cabeza + cofirmas-. El paquete
# COMPLETO (con acuse) exige combinar --dev y fondeo con un testigo
# cofirmante, cosa que hoy no hace ningun banco, y sube a TRES las capturas
# dentro de la misma ventana de epoca. Queda como sucesor nombrado.
#
# FUERA del canon: levanta procesos y espera latidos. Se corre a mano o desde
# un bloque. Exit 0 = BANCO VERDE; cualquier fallo, exit 1.
#
# RELEASE, no debug: el testigo FIRMA, y una firma XMSS mide 144,5 ms en
# release (292); en debug esto tardaria minutos. Suma zk-ssl-verify al build,
# que ningun banco compilaba en release hasta hoy.
# Bajo $HOME, NUNCA en /tmp: el guardian del indice se niega donde fsync no
# persiste (K.1, 234).
# En un fallo se ensenan CLASES y CONTEOS, nunca los ficheros.
set -uo pipefail

msg(){ printf 'BANCO-EV2| %s\n' "$*" >&2; }
DIR=$(mktemp -d "$HOME/.banco_evidencia_v2.XXXXXX") || { msg "no pude crear el directorio"; exit 2; }
NODO=""; TESTIGO=""
limpiar(){
  [ -n "$TESTIGO" ] && kill "$TESTIGO" 2>/dev/null
  [ -n "$NODO" ] && kill "$NODO" 2>/dev/null
  wait 2>/dev/null
  rm -rf "$DIR"
}
trap limpiar EXIT
fallo(){ msg "ROJO: $*"; exit 1; }

cd "$(dirname "$0")/.." || fallo "no encuentro la raiz"
command -v curl >/dev/null 2>&1 || fallo "curl no esta en el PATH"
command -v python3 >/dev/null 2>&1 || fallo "python3 no esta en el PATH"

msg "compilando nodo, cli y verificador (release: el testigo FIRMA)"
cargo build --release -q -p zk-ssl-node -p zk-ssl-cli -p zk-ssl-verify \
  || cargo build --release -p zk-ssl-node -p zk-ssl-cli -p zk-ssl-verify \
  || fallo "no compila"
NB=target/release/zk-ssl-node
CB=target/release/zk-ssl-cli
VB=target/release/zk-ssl-verify
[ -x "$NB" ] || fallo "falta $NB"
[ -x "$CB" ] || fallo "falta $CB"
[ -x "$VB" ] || fallo "falta $VB"

# Semillas DETERMINISTAS: un banco que dependa de /dev/urandom no se puede
# volver a correr igual. Y en DOS FORMATOS, porque el proyecto los tiene: el
# NODO lee su clave en HEX, el TESTIGO en binario crudo. Esa divergencia es
# del proyecto, no de este banco.
python3 -c "print('5b'*96, end='')" > "$DIR/semilla-nodo.hex" || fallo "semilla del nodo"
chmod 600 "$DIR/semilla-nodo.hex"
python3 -c "
import sys
sys.stdout.buffer.write(bytes(((i*31+9) % 256) for i in range(96)))
" > "$DIR/semilla-testigo.bin" || fallo "semilla del testigo"
chmod 600 "$DIR/semilla-testigo.bin"

# Puerto propio: 8599 es de banco_apagado y 8601 de banco_cofirma.
PORT=8603
LATIDO=4
VUELTAS=40

arranca_nodo(){
  "$NB" --listen "127.0.0.1:$PORT" --latido "$LATIDO" \
    --clave-fichero "$DIR/semilla-nodo.hex" --custodia fichero \
    --diario "$DIR/nodo.jsonl" --ledger "$DIR/ledger" \
    --contador-recepcion "$DIR/recepcion.bin" \
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

# El testigo, EN SEGUNDO PLANO. Es la desviacion del molde y su razon esta
# en la cabecera: el par cabeza/cofirmas tiene que ser de la misma epoca.
testigo_fondo(){
  "$CB" witness --nodo "http://127.0.0.1:$PORT" --cada 1 --veces "$VUELTAS" --no-color \
        --enviar-cofirmas \
        --diario "$DIR/diario.jsonl" \
        --cofirmar "$DIR/semilla-testigo.bin" \
        --indice-cofirma "$DIR/contador.bin" \
        --cofirmas "$DIR/cofirmas.jsonl" > "$DIR/t.out" 2>&1 &
  TESTIGO=$!
}

# NI UNA COMILLA DE JSON ESCRITA A MANO donde viaja un dato: json.dumps lo
# escapa y curl lo manda desde fichero, sin releerlo.
pregunta_cosigs(){  # $1 = el epochDigest, o NADA para la actual
  python3 - "${1:-}" > "$DIR/peticion.json" <<'PYEOF'
import json, sys
d = sys.argv[1] if len(sys.argv) > 1 else ""
p = {"epochDigest": d} if d else None
print(json.dumps({"jsonrpc": "2.0", "id": 9, "method": "zkssl_cosigs", "params": p}))
PYEOF
  curl -s --max-time 10 "http://127.0.0.1:$PORT" \
    -H 'Content-Type: application/json' -d @"$DIR/peticion.json" 2>/dev/null || true
}

# Aqui no viaja ningun dato: el cuerpo es constante.
cabeza_firmada(){
  curl -s --max-time 15 "http://127.0.0.1:$PORT" \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":2,"method":"zkssl_signedEpochHead","params":{}}' 2>/dev/null || true
}

# == ENVIO-A - EL ROJO, y va PRIMERO =================================
# Gratis: nadie ha enviado nada a este nodo todavia.
msg "ENVIO-A: nodo virgen con latido $LATIDO"
arranca_nodo
R0=$(pregunta_cosigs)
python3 - "$R0" <<'PYEOF' >&2 || fallo "el nodo virgen no dijo n:0"
import json, sys
def q(x):
    if isinstance(x, bool): raise TypeError("bool no es Q")
    if isinstance(x, int): return x
    s = str(x)
    return int(s, 16) if s.lower().startswith("0x") else int(s)
crudo = sys.argv[1]
if not crudo.strip():
    sys.exit("el nodo no contesto a zkssl_cosigs: respuesta VACIA")
d = json.loads(crudo)
if "error" in d:
    sys.exit("el nodo devolvio error a zkssl_cosigs: %r" % (d["error"],))
n = q(d["result"]["n"])
assert n == 0, "un nodo sin envios debe dar n:0 y dio %d" % n
print("BANCO-EV2| ENVIO-A (el rojo): nodo virgen, zkssl_cosigs da n:0")
PYEOF

# == POSITIVO - el par (cabeza, cofirmas de SU epoca) ================
msg "POSITIVO: testigo en segundo plano, $VUELTAS vueltas, enviando cofirmas"
testigo_fondo
CAB=""; COS=""; DIG=""
for i in $(seq 1 60); do
  C=$(cabeza_firmada)
  case "$C" in *'"available":true'*) : ;; *) sleep 1; continue;; esac
  D=$(python3 - "$C" <<'PYEOF' 2>&1 || true
import json, sys
d = json.loads(sys.argv[1])["result"]
v = d.get("epochDigest")
print(v if isinstance(v, str) else "SIN-CAMPO:%s" % ",".join(sorted(d.keys()))[:200])
PYEOF
)
  case "$D" in
    SIN-CAMPO:*) fallo "el payload de signedEpochHead no lleva epochDigest. Sus claves: ${D#SIN-CAMPO:}" ;;
    "")          sleep 1; continue ;;
  esac
  R1=$(pregunta_cosigs "$D")
  N=$(python3 - "${R1:-}" <<'PYEOF' 2>&1 || true
import json, sys
def q(x):
    if isinstance(x, bool): raise TypeError("bool no es Q")
    if isinstance(x, int): return x
    s = str(x)
    return int(s, 16) if s.lower().startswith("0x") else int(s)
crudo = sys.argv[1] if len(sys.argv) > 1 else ""
if not crudo.strip():
    print("VACIA"); raise SystemExit(0)
try:
    d = json.loads(crudo)
except Exception as e:
    print("NO-JSON:%s" % e); raise SystemExit(0)
if "error" in d:
    print("ERROR-RPC:%s" % json.dumps(d["error"])[:120]); raise SystemExit(0)
v = d.get("result")
if v is None:
    print("SIN-RESULT"); raise SystemExit(0)
print("N:%d" % q(v["n"]))
PYEOF
)
  case "$N" in
    ERROR-RPC:*) fallo "el nodo RECHAZO la pregunta ($N): no es el almacen, es la peticion" ;;
    N:0)         [ $((i % 5)) = 0 ] && msg "POSITIVO: vuelta $i, la epoca en curso aun no tiene cofirmas" ;;
    N:*)         CAB="$C"; COS="$R1"; DIG="$D"
                 msg "POSITIVO: en la vuelta $i casan cabeza y cofirmas de la misma epoca ($N)"
                 break ;;
    *)           msg "POSITIVO: vuelta $i, veredicto $N" ;;
  esac
  sleep 1
done
[ -n "$CAB" ] || { tail -20 "$DIR/t.out" >&2; fallo "en 60 sondeos no hubo una cabeza con cofirmas de SU epoca: es la VENTANA, no el paquete"; }

msg "POSITIVO: esperando a que el testigo termine sus $VUELTAS vueltas"
wait "$TESTIGO"; RC=$?
TESTIGO=""
[ "$RC" = "0" ] || { tail -20 "$DIR/t.out" >&2; fallo "el testigo salio con $RC"; }
ENV=$(grep -c 'cofirma enviada' "$DIR/t.out" || true)
DURO=$(grep -cE 'dio ERROR|no se pudo enviar' "$DIR/t.out" || true)
msg "POSITIVO: $ENV cofirma(s) enviada(s), $DURO fallo(s) duro(s) de envio"
[ "$DURO" = "0" ] || { grep -E 'dio ERROR|no se pudo enviar' "$DIR/t.out" | head -5 >&2; fallo "$DURO fallo(s) DUROS de envio"; }

# EL PAQUETE, armado TAL CUAL: la cabeza es result de signedEpochHead y las
# cofirmas son result.cosigs -no el sobre entero: la clave del paquete es
# cofirmas y el sobre lleva epochDigest y n ademas de la lista-.
K=$(python3 - "$CAB" "$COS" "$DIR/paquete.json" <<'PYEOF' 2>&1
import json, sys
cab = json.loads(sys.argv[1])["result"]
sob = json.loads(sys.argv[2])["result"]
assert cab.get("available") is True, "la cabeza no era available:true"
cos = sob.get("cosigs")
assert isinstance(cos, list) and cos, "el sobre no trae una lista cosigs con contenido"
paquete = {"v": 2, "cabeza": cab, "cofirmas": cos}
open(sys.argv[3], "w", encoding="utf-8").write(json.dumps(paquete))
print(len(cos))
PYEOF
)
RCK=$?
[ "$RCK" = "0" ] || fallo "no pude armar el paquete: $K"
msg "POSITIVO: paquete v2 armado con $K cofirma(s), sin reescribir un solo campo"

set +e; SAL=$("$VB" "$DIR/paquete.json" 2>&1); RC=$?; set -e
[ "$RC" = "0" ] || { printf '%s\n' "$SAL" >&2; fallo "el paquete v2 legitimo dio exit $RC"; }
printf '%s\n' "$SAL" | grep -q "cofirmas: $K verifican" \
  || { printf '%s\n' "$SAL" >&2; fallo "el verificador no dijo que verifican las $K cofirmas"; }
printf '%s\n' "$SAL" | grep -q 'VERDE' \
  || { printf '%s\n' "$SAL" >&2; fallo "no salio el VERDE"; }
msg "POSITIVO: exit 0 - $(printf '%s\n' "$SAL" | grep 'cofirmas:')"

# == NEGATIVO-1 - un v1 que trae cofirmas ============================
# La compuerta existe en el binario y NO la cubre ningun test.
msg "NEGATIVO-1: el mismo paquete, declarado v1, con las cofirmas dentro"
python3 - "$DIR/paquete.json" "$DIR/n1.json" <<'PYEOF' || fallo "no pude fabricar el v1"
import json, sys
p = json.load(open(sys.argv[1], encoding="utf-8"))
p["v"] = 1
open(sys.argv[2], "w", encoding="utf-8").write(json.dumps(p))
PYEOF
set +e; SAL=$("$VB" "$DIR/n1.json" 2>&1); RC=$?; set -e
[ "$RC" != "0" ] || { printf '%s\n' "$SAL" >&2; fallo "un v1 CON cofirmas paso por bueno"; }
printf '%s\n' "$SAL" | grep -q 'VERDE' && { printf '%s\n' "$SAL" >&2; fallo "murio pero imprimio VERDE"; }
# El literal de la compuerta, MEDIDO en la corrida de hoy: un rc que no
# nombra POR QUE murio deja pasar un rojo por la razon equivocada.
printf '%s\n' "$SAL" | grep -q 'subir la version es lo que las hace parte del contrato' \
  || { printf '%s\n' "$SAL" >&2; fallo "murio, pero no por la compuerta del v1 con cofirmas"; }
msg "NEGATIVO-1: exit $RC - $(printf '%s\n' "$SAL" | grep 'ROJO' | head -1 || true)"

# == NEGATIVO-2 - una version de paquete desconocida =================
msg "NEGATIVO-2: el mismo paquete, declarado v3"
python3 - "$DIR/paquete.json" "$DIR/n2.json" <<'PYEOF' || fallo "no pude fabricar el v3"
import json, sys
p = json.load(open(sys.argv[1], encoding="utf-8"))
p["v"] = 3
open(sys.argv[2], "w", encoding="utf-8").write(json.dumps(p))
PYEOF
set +e; SAL=$("$VB" "$DIR/n2.json" 2>&1); RC=$?; set -e
[ "$RC" != "0" ] || { printf '%s\n' "$SAL" >&2; fallo "un paquete v3 paso por bueno"; }
printf '%s\n' "$SAL" | grep -q 'lee v1 y v2' \
  || { printf '%s\n' "$SAL" >&2; fallo "murio, pero no diciendo que lee v1 y v2"; }
msg "NEGATIVO-2: exit $RC - $(printf '%s\n' "$SAL" | grep 'ROJO' | head -1 || true)"

# == NEGATIVO-3 - un nibble de la firma ==============================
# El unico que exige testigo vivo: aqui la criptografia corre de verdad.
msg "NEGATIVO-3: se adultera UN nibble de la firma de la primera cofirma"
python3 - "$DIR/paquete.json" "$DIR/n3.json" <<'PYEOF' || fallo "no pude adulterar"
import json, sys
p = json.load(open(sys.argv[1], encoding="utf-8"))
f = list(p["cofirmas"][0]["firma"])
i = 20
f[i] = "b" if f[i] == "a" else "a"
p["cofirmas"][0]["firma"] = "".join(f)
open(sys.argv[2], "w", encoding="utf-8").write(json.dumps(p))
PYEOF
set +e; SAL=$("$VB" "$DIR/n3.json" 2>&1); RC=$?; set -e
[ "$RC" != "0" ] || { printf '%s\n' "$SAL" >&2; fallo "una cofirma ADULTERADA paso por buena"; }
printf '%s\n' "$SAL" | grep -q 'cofirma 1' \
  || { printf '%s\n' "$SAL" >&2; fallo "murio, pero sin nombrar QUE cofirma"; }
# El mensaje ensena la linea que FALLA, no la primera. El verificador
# imprime su progreso (1/3, 2/3) ANTES de morir, asi que un head -1 aqui
# reportaba una linea de EXITO para un rojo. Se vio en la primera corrida.
msg "NEGATIVO-3: exit $RC - $(printf '%s\n' "$SAL" | grep 'cofirma 1' | head -1 || true)"

msg "BANCO-EV2 VERDE: las cofirmas viajan dentro del paquete y un tercero las"
msg "                 verifica sin el nodo, sin el testigo y sin el diario"
# FIN-BANCO-EV2
