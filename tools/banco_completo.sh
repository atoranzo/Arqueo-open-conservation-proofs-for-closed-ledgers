#!/usr/bin/env bash
# tools/banco_completo.sh - el banco del PAQUETE COMPLETO (325).
#
# banco_apagado.sh (290) demostro que una posicion se sostiene SIN el nodo:
#   cabeza + acuse, dos capturas espalda contra espalda.
# banco_cofirma.sh (301) demostro que el testigo avala, y solo cuando debe.
# banco_evidencia_v2.sh (323) demostro que las COFIRMAS viajan dentro del
#   paquete: el MINIMO, cabeza + cofirmas, sin acuse y sin fondeo.
#
# Este demuestra LAS TRES A LA VEZ, que es lo que ningun banco hacia:
# {v:2, cabeza, acuse, cofirmas} armado en una sola ventana de epoca y
# verificado por un tercero sin el nodo, sin el testigo y sin el diario.
#
#   ENVIO-A     el rojo primero, y es gratis: nodo virgen, zkssl_cosigs n:0.
#   CALENTAMIENTO  el testigo no cofirma hasta `Nueva`+`Extiende`, y eso son
#               ~6 latidos (banco_cofirma:217-218). Se paga UNA vez y FUERA
#               del bucle, sondeando el par como el 323. Meterlo dentro
#               gastaria los intentos midiendo la arrancada del testigo.
#   ATENCION, FALSADO POR LA CORRIDA DEL r1: el "acto seguido" del diseno
#               vale para cabeza+acuse pero NO para las cofirmas. Cuando el
#               latido cierra la epoca, la cabeza ya existe y su cofirma NO:
#               el testigo tiene que verla, firmar y submitir. Preguntar sin
#               esperar dio n:0 SEIS veces con las otras tres causas a cero.
#               Aqui las cofirmas se SONDEAN por su nombre.
#
#   POSITIVO    seis fondeos como sondeo. Cada uno fija SU epoca -el acuse
#               ata a la cabeza que cerro esa epoca y no a otra- y el trio
#               se verifica en el acto. El primero que sale verde gana.
#   NEGATIVO-1  un v1 que trae cofirmas se RECHAZA.
#   NEGATIVO-2  un v3 desconocido se niega en voz alta.
#   NEGATIVO-3  un nibble de la firma de una cofirma: exit 1 nombrando cual.
#   NEGATIVO-4  un nibble del hashPrueba del acuse: exit 1 por el acuse.
#               Es el negativo que este banco anade sobre el de v2.
#
# ALCANCE, declarado: aqui el paquete lleva las TRES cosas. El binario ya
# lo soporta -su cabecera declara v2 con `acuse` y `cofirmas` OPCIONALES,
# y el tramo del acuse corre sea cual sea la version-, asi que este banco
# no pide codigo nuevo: pide una VENTANA.
#
# ATENCION al asertar: un paquete SIN acuse y un v2 con CERO cofirmas
# tambien imprimen VERDE. Por eso el positivo exige las DOS lineas -la del
# acuse y la del conteo- y no solo el VERDE. Un banco que solo mirara el
# VERDE pasaria en verde sin acuse y sin cofirmas.
#
# DECLARADO Y NO REPARADO: la cabecera del binario declara un campo `s` en
# el acuse que NINGUN codigo lee -el tramo lee hashPrueba, seq, siblings e
# isRight y nada mas-. Se empaqueta igual, TAL CUAL lo hace banco_apagado,
# porque quien reescribe adultera. Si `s` es vestigio o reserva NO esta
# medido. Renglon 247 para otro corte.
#
# FUERA del canon: levanta procesos y espera latidos. Se corre a mano o
# desde un bloque. Exit 0 = BANCO VERDE; cualquier fallo, exit 1.
#
# RELEASE, no debug: el testigo FIRMA, y una firma XMSS mide 144,5 ms en
# release (292); en debug esto tardaria minutos.
# Bajo $HOME, NUNCA en /tmp: el guardian del indice se niega donde fsync no
# persiste (K.1, 234).
# En un fallo se ensenan CLASES y CONTEOS, nunca los ficheros.
# DESVIACION DECLARADA del molde: `set -uo pipefail` y NUNCA se toca `-e`.
# banco_evidencia_v2 alterna `set +e` / `set -e` alrededor de cada llamada
# al verificador (:237, :254, :271, :289) y deja errexit ENCENDIDO desde la
# primera vez, aunque el fichero arranca sin el. Ahi sobrevive porque todo
# lo que sigue va protegido por `||` o `&&`, pero es una trampa cargada.
# Aqui el rc se captura siempre en la linea siguiente y no hace falta.
set -uo pipefail

msg(){ printf 'BANCO-COMPLETO| %s\n' "$*" >&2; }
DIR=$(mktemp -d "$HOME/.banco_completo.XXXXXX") || { msg "no pude crear el directorio"; exit 2; }
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

# Semillas DETERMINISTAS y en DOS FORMATOS, porque el proyecto los tiene:
# el NODO lee su clave en HEX, el TESTIGO en binario crudo. Esa divergencia
# es del proyecto, no de este banco (nota de los dos formatos de semilla).
python3 -c "print('7c'*96, end='')" > "$DIR/semilla-nodo.hex" || fallo "semilla del nodo"
chmod 600 "$DIR/semilla-nodo.hex"
python3 -c "
import sys
sys.stdout.buffer.write(bytes(((i*17+5) % 256) for i in range(96)))
" > "$DIR/semilla-testigo.bin" || fallo "semilla del testigo"
chmod 600 "$DIR/semilla-testigo.bin"

# Puerto propio: 8599 apagado, 8601 cofirma, 8603 evidencia v2.
PORT=8605
LATIDO=4
# 90 vueltas a --cada 1 = ~90 s de testigo vivo. La cuenta, declarada:
#   ~6 latidos de calentamiento (banco_cofirma:217-218) = ~24 s
# + 6 intentos a UN latido cada uno                     = ~24 s
# = ~48 s tipicos. El peor caso, con sobrecarga NO medida de ~2 s por
# intento, daria ~60 s. Los dos caben en 90 con holgura.
VUELTAS=90
FONDEOS=6
# La cofirma de una cabeza NO existe cuando la cabeza aparece: el testigo tiene
# que VERLA, firmar XMSS y submitirla, y eso llega ~1 s despues. Preguntar
# "acto seguido" es preguntar ANTES de que exista. Medido en la corrida del
# r1: seis intentos, seis n:0, y las otras tres causas a CERO.
# 10 sondeos de 0,5 s = 5 s = mas de un latido, que es lo que dura la
# retencion antes de que la submision de la epoca siguiente purgue esta.
ESPERA_COFIRMAS=10

arranca_nodo(){
  # --dev: dev_openSeeded y dev_fund. Es lo que ningun banco combinaba con
  #        un testigo cofirmante, y la razon de que este banco exista.
  # --diario: OBLIGATORIO. Sin el, zkssl_ackPath contesta que "los limites
  #        de epoca no se conservan" y no hay acuse que empaquetar.
  "$NB" --listen "127.0.0.1:$PORT" --dev --latido "$LATIDO" \
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

testigo_fondo(){
  "$CB" witness --nodo "http://127.0.0.1:$PORT" --cada 1 --veces "$VUELTAS" --no-color \
        --enviar-cofirmas \
        --diario "$DIR/diario.jsonl" \
        --cofirmar "$DIR/semilla-testigo.bin" \
        --indice-cofirma "$DIR/contador.bin" \
        --cofirmas "$DIR/cofirmas.jsonl" > "$DIR/t.out" 2>&1 &
  TESTIGO=$!
}

# Los cuerpos con DATO dentro van por json.dumps y por fichero: ni una
# comilla de JSON escrita a mano donde viaja un dato.
rpc_cuerpo(){  # $1 = la peticion JSON entera, ya serializada por python
  printf '%s' "$1" > "$DIR/peticion.json"
  curl -s --max-time 15 "http://127.0.0.1:$PORT" \
    -H 'Content-Type: application/json' -d @"$DIR/peticion.json" 2>/dev/null || true
}
peticion(){  # $1 = metodo, $2.. = pares clave valor (valor TAL CUAL del cable)
  python3 - "$@" <<'PYEOF'
import json, sys
m = sys.argv[1]
p = {}
resto = sys.argv[2:]
for i in range(0, len(resto), 2):
    p[resto[i]] = json.loads(resto[i + 1])
print(json.dumps({"jsonrpc": "2.0", "id": 7, "method": m, "params": p or None}))
PYEOF
}
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
cabeza_firmada(){
  curl -s --max-time 15 "http://127.0.0.1:$PORT" \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":2,"method":"zkssl_signedEpochHead","params":{}}' 2>/dev/null || true
}
digest_de_cabeza(){  # $1 = respuesta cruda de signedEpochHead. Imprime la cadena CRUDA.
  python3 - "$1" <<'PYEOF' 2>&1
import json, sys
d = json.loads(sys.argv[1])["result"]
v = d.get("epochDigest")
print(v if isinstance(v, str) else "SIN-CAMPO:%s" % ",".join(sorted(d.keys()))[:200])
PYEOF
}
campo(){  # $1 = respuesta cruda, $2 = ruta con puntos. Imprime JSON.
  python3 - "$1" "$2" <<'PYEOF'
import json, sys
d = json.loads(sys.argv[1])
for k in sys.argv[2].split('.'):
    d = d[int(k)] if k.isdigit() else d[k]
print(json.dumps(d))
PYEOF
}
cuantas(){  # $1 = respuesta cruda de cosigs. Imprime N:<n> o una CLASE.
  python3 - "${1:-}" <<'PYEOF'
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
}

# == ENVIO-A - EL ROJO, y va PRIMERO =================================
msg "ENVIO-A: nodo virgen con latido $LATIDO en el puerto $PORT"
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
print("BANCO-COMPLETO| ENVIO-A (el rojo): nodo virgen, zkssl_cosigs da n:0")
PYEOF

# == CALENTAMIENTO - fuera del bucle =================================
# El testigo no cofirma hasta `Nueva`+`Extiende`. Sondear el PAR es la
# senal barata de que ya cofirma: si hay una cabeza con cofirmas de SU
# epoca, el calentamiento termino. Meter esto dentro del bucle de fondeos
# gastaria los seis intentos midiendo la arrancada del testigo.
msg "CALENTAMIENTO: testigo al fondo, $VUELTAS vueltas, enviando cofirmas"
testigo_fondo
CALIENTE=0
for i in $(seq 1 60); do
  if ! kill -0 "$TESTIGO" 2>/dev/null; then break; fi
  C=$(cabeza_firmada)
  case "$C" in *'"available":true'*) : ;; *) sleep 1; continue;; esac
  D=$(digest_de_cabeza "$C")
  case "$D" in "" | SIN-CAMPO:*) sleep 1; continue;; esac
  N=$(cuantas "$(pregunta_cosigs "$D")")
  case "$N" in
    ERROR-RPC:*) fallo "el nodo RECHAZO la pregunta ($N): no es el almacen, es la peticion" ;;
    N:0)         [ $((i % 5)) = 0 ] && msg "CALENTAMIENTO: vuelta $i, la epoca en curso aun no tiene cofirmas" ;;
    N:*)         CALIENTE=1; msg "CALENTAMIENTO listo en la vuelta $i: el testigo ya cofirma ($N)"; break ;;
    *)           msg "CALENTAMIENTO: vuelta $i, veredicto $N" ;;
  esac
  sleep 1
done
[ "$CALIENTE" = "1" ] || { tail -20 "$DIR/t.out" >&2; fallo "el testigo no llego a cofirmar: es el CALENTAMIENTO, no la ventana"; }

# == POSITIVO - los seis fondeos ====================================
# El acuse ata a UNA cabeza: la que cerro la epoca de su entrada. El fondeo
# FIJA esa epoca y el banco no puede elegirla, asi que la carrera se
# convierte en sondeo: se fondea, se intenta el trio, y si no casa se
# fondea otra vez. Es lo que el 323 hizo con el par, aplicado al trio.
msg "POSITIVO: abriendo la cuenta y fondeando hasta $FONDEOS veces"
R=$(rpc_cuerpo "$(peticion dev_openSeeded seed '"0x1"')")
case "$R" in *'"error"'*) fallo "dev_openSeeded: $R";; esac
IDX=$(campo "$R" result.index) || fallo "sin index: $R"

# Las cuatro causas, contadas por separado: un instrumento que falla dice
# QUE fallo, no cuantos (254). Cada una tiene un arreglo distinto.
C_ACUSE=0      # el sondeo de ackPath se agoto: la epoca de la entrada no cerro
C_CABEZA=0     # el verificador murio en el acuse: la cabeza ya avanzo
C_COFIRMAS=0   # cosigs dio n:0 para esa epoca
C_TESTIGO=0    # el testigo termino sus vueltas: se sube VUELTAS
LOGRADO=0
K=0
for intento in $(seq 1 "$FONDEOS"); do
  if ! kill -0 "$TESTIGO" 2>/dev/null; then
    C_TESTIGO=1
    msg "POSITIVO: intento $intento abortado - el testigo ya termino sus $VUELTAS vueltas"
    break
  fi
  R=$(rpc_cuerpo "$(peticion dev_fund index "$IDX" amount '"0x64"')")
  case "$R" in *'"error"'*) fallo "dev_fund: $R";; esac
  SEQ=$(campo "$R" result.logSeq) || fallo "sin logSeq: $R"

  # El proofDigest de una entrada no cambia: esta llamada esta FUERA de la
  # ventana y va sin prisa.
  R=$(rpc_cuerpo "$(peticion zkssl_logEntry seq "$SEQ")")
  case "$R" in *proofDigest*) : ;; *) fallo "logEntry sin proofDigest: $R";; esac
  PD=$(campo "$R" result.proofDigest) || fallo "no pude leer proofDigest"

  # available:true dice que la epoca de ESA entrada cerro. Es el disparo de
  # la ventana, y dura un latido.
  ACK=""
  for _ in $(seq 1 24); do
    ACK=$(rpc_cuerpo "$(peticion zkssl_ackPath seq "$SEQ")")
    case "$ACK" in *'"available":true'*) break;; esac
    sleep 0.5
  done
  case "$ACK" in
    *'"available":true'*) : ;;
    *) C_ACUSE=$((C_ACUSE + 1)); msg "POSITIVO: intento $intento - la epoca de la entrada no cerro en 12 s"; continue ;;
  esac

  # ACTO SEGUIDO. El espalda contra espalda de banco_apagado no es una
  # optimizacion: la cabeza siguiente lleva otro acusesRoot.
  CAB=$(cabeza_firmada)
  case "$CAB" in *'"available":true'*) : ;; *) C_CABEZA=$((C_CABEZA + 1)); msg "POSITIVO: intento $intento - signedEpochHead sin firma"; continue;; esac
  DIG=$(digest_de_cabeza "$CAB")
  case "$DIG" in "" | SIN-CAMPO:*) C_CABEZA=$((C_CABEZA + 1)); msg "POSITIVO: intento $intento - la cabeza no trae epochDigest"; continue;; esac

  # NO "acto seguido": SONDEO. La cabeza y el acuse ya existen cuando el
  # latido cierra la epoca; la cofirma de esa cabeza todavia no. Se pregunta
  # POR SU NOMBRE, asi que un sondeo largo no puede traer la de otra epoca:
  # o llega la que se pide, o sigue dando 0.
  COS=""; N="N:0"; SONDEOS=0
  for s_i in $(seq 1 "$ESPERA_COFIRMAS"); do
    COS=$(pregunta_cosigs "$DIG")
    N=$(cuantas "$COS")
    SONDEOS=$s_i
    case "$N" in
      ERROR-RPC:*) fallo "el nodo RECHAZO la pregunta ($N): no es el almacen, es la peticion" ;;
      N:0)         sleep 0.5 ;;
      N:*)         break ;;
      *)           sleep 0.5 ;;
    esac
  done
  case "$N" in
    N:0 | ERROR-RPC:* | VACIA | NO-JSON:* | SIN-RESULT)
      # El instrumento distingue los dos fallos (254): preguntar SIN nombre
      # dice si el nodo tiene cofirmas de OTRA epoca -y entonces es la
      # CARRERA- o si no tiene ninguna -y entonces es el testigo-.
      NSIN=$(cuantas "$(pregunta_cosigs)")
      C_COFIRMAS=$((C_COFIRMAS + 1))
      msg "POSITIVO: intento $intento - esa epoca no dio cofirmas en $SONDEOS sondeos; el almacen SIN nombre dice $NSIN"
      continue ;;
  esac
  msg "POSITIVO: intento $intento - las cofirmas de esa epoca llegaron en el sondeo $SONDEOS"

  # EL PAQUETE COMPLETO. La cabeza va TAL CUAL y las cofirmas tambien; el
  # acuse es RECOMPOSICION declarada, con cuatro campos de DOS respuestas,
  # exactamente como banco_apagado:115-125. `s` viaja aunque el binario no
  # lo lea: quien reescribe adultera.
  K=$(python3 - "$CAB" "$ACK" "$SEQ" "$PD" "$COS" "$DIR/paquete.json" <<'PYEOF' 2>&1
import json, sys
cab = json.loads(sys.argv[1])["result"]
ack = json.loads(sys.argv[2])["result"]
sob = json.loads(sys.argv[5])["result"]
assert cab.get("available") is True, "la cabeza no era available:true"
assert ack.get("available") is True, "el acuse no era available:true"
cos = sob.get("cosigs")
assert isinstance(cos, list) and cos, "el sobre no trae una lista cosigs con contenido"
paquete = {
    "v": 2,
    "cabeza": cab,
    "acuse": {
        "seq": json.loads(sys.argv[3]),
        "hashPrueba": json.loads(sys.argv[4]),
        "s": ack["s"],
        "camino": ack["camino"],
    },
    "cofirmas": cos,
}
open(sys.argv[6], "w", encoding="utf-8").write(json.dumps(paquete))
print(len(cos))
PYEOF
)
  RCK=$?
  [ "$RCK" = "0" ] || fallo "no pude armar el paquete: $K"

  SAL=$("$VB" "$DIR/paquete.json" 2>&1); RC=$?
  if [ "$RC" != "0" ]; then
    case "$SAL" in
      *"ROJO: acuse:"*) C_CABEZA=$((C_CABEZA + 1)); msg "POSITIVO: intento $intento - el acuse no verifica contra esta cabeza: ya avanzo"; continue ;;
      *) printf '%s\n' "$SAL" >&2; fallo "el paquete completo dio exit $RC por algo que no es la ventana" ;;
    esac
  fi
  LOGRADO=$intento
  msg "POSITIVO: intento $intento CASA - cabeza, acuse y $K cofirma(s) en la misma ventana"
  break
done

if [ "$LOGRADO" = "0" ]; then
  msg "las cuatro causas, contadas por separado:"
  msg "  la epoca de la entrada no cerro ....... $C_ACUSE"
  msg "  la cabeza ya habia avanzado ........... $C_CABEZA"
  msg "  esa epoca no tenia cofirmas ........... $C_COFIRMAS"
  msg "  el testigo agoto sus $VUELTAS vueltas .... $C_TESTIGO"
  tail -20 "$DIR/t.out" >&2
  if [ "$C_TESTIGO" != "0" ]; then
    fallo "sin trio en $FONDEOS intentos: se acabo el TESTIGO, no la ventana. Se sube VUELTAS, no se toca el protocolo"
  fi
  fallo "sin trio en $FONDEOS intentos: es la VENTANA. La causa con mas cuenta dice por donde"
fi

# LAS DOS LINEAS QUE NO PUEDEN FALTAR. Un paquete SIN acuse imprime
# "3/3 sin acuse" y un v2 con CERO cofirmas imprime "no trae ninguna", y
# los dos acaban en VERDE: asertar solo el VERDE dejaria pasar un paquete
# que no es el completo.
printf '%s\n' "$SAL" | grep -q '3/3 el acuse sube hasta la raiz firmada' \
  || { printf '%s\n' "$SAL" >&2; fallo "el verificador no dijo que el acuse sube a la raiz firmada"; }
printf '%s\n' "$SAL" | grep -q "cofirmas: $K verifican" \
  || { printf '%s\n' "$SAL" >&2; fallo "el verificador no dijo que verifican las $K cofirmas"; }
printf '%s\n' "$SAL" | grep -q 'VERDE' \
  || { printf '%s\n' "$SAL" >&2; fallo "no salio el VERDE"; }
msg "POSITIVO: exit 0 - las TRES piezas verificadas por un tercero sin el nodo"

msg "POSITIVO: esperando a que el testigo termine sus vueltas"
wait "$TESTIGO" 2>/dev/null; RCT=$?
TESTIGO=""
[ "$RCT" = "0" ] || { tail -20 "$DIR/t.out" >&2; fallo "el testigo salio con $RCT"; }
ENV=$(grep -c 'cofirma enviada' "$DIR/t.out" || true)
DURO=$(grep -cE 'dio ERROR|no se pudo enviar' "$DIR/t.out" || true)
msg "POSITIVO: $ENV cofirma(s) enviada(s), $DURO fallo(s) duro(s) de envio"
[ "$DURO" = "0" ] || { grep -E 'dio ERROR|no se pudo enviar' "$DIR/t.out" | head -5 >&2; fallo "$DURO fallo(s) DUROS de envio"; }

# == NEGATIVO-1 - un v1 que trae cofirmas ============================
msg "NEGATIVO-1: el mismo paquete, declarado v1, con las cofirmas dentro"
python3 - "$DIR/paquete.json" "$DIR/n1.json" <<'PYEOF' || fallo "no pude fabricar el v1"
import json, sys
p = json.load(open(sys.argv[1], encoding="utf-8"))
p["v"] = 1
open(sys.argv[2], "w", encoding="utf-8").write(json.dumps(p))
PYEOF
SAL=$("$VB" "$DIR/n1.json" 2>&1); RC=$?
[ "$RC" != "0" ] || { printf '%s\n' "$SAL" >&2; fallo "un v1 CON cofirmas paso por bueno"; }
printf '%s\n' "$SAL" | grep -q 'VERDE' && { printf '%s\n' "$SAL" >&2; fallo "murio pero imprimio VERDE"; }
printf '%s\n' "$SAL" | grep -q 'subir la version es lo que las hace' \
  || { printf '%s\n' "$SAL" >&2; fallo "murio, pero no por la compuerta del v1 con cofirmas"; }
msg "NEGATIVO-1: exit $RC - la compuerta del v1 con cofirmas"

# == NEGATIVO-2 - una version de paquete desconocida =================
msg "NEGATIVO-2: el mismo paquete, declarado v3"
python3 - "$DIR/paquete.json" "$DIR/n2.json" <<'PYEOF' || fallo "no pude fabricar el v3"
import json, sys
p = json.load(open(sys.argv[1], encoding="utf-8"))
p["v"] = 3
open(sys.argv[2], "w", encoding="utf-8").write(json.dumps(p))
PYEOF
SAL=$("$VB" "$DIR/n2.json" 2>&1); RC=$?
[ "$RC" != "0" ] || { printf '%s\n' "$SAL" >&2; fallo "un paquete v3 paso por bueno"; }
printf '%s\n' "$SAL" | grep -q 'lee v1 y v2' \
  || { printf '%s\n' "$SAL" >&2; fallo "murio, pero no diciendo que lee v1 y v2"; }
msg "NEGATIVO-2: exit $RC - la version desconocida se niega en voz alta"

# == NEGATIVO-3 - un nibble de la firma de una cofirma ===============
msg "NEGATIVO-3: se adultera UN nibble de la firma de la primera cofirma"
python3 - "$DIR/paquete.json" "$DIR/n3.json" <<'PYEOF' || fallo "no pude adulterar la firma"
import json, sys
p = json.load(open(sys.argv[1], encoding="utf-8"))
f = list(p["cofirmas"][0]["firma"])
i = 20
f[i] = "b" if f[i] == "a" else "a"
p["cofirmas"][0]["firma"] = "".join(f)
open(sys.argv[2], "w", encoding="utf-8").write(json.dumps(p))
PYEOF
SAL=$("$VB" "$DIR/n3.json" 2>&1); RC=$?
[ "$RC" != "0" ] || { printf '%s\n' "$SAL" >&2; fallo "una cofirma ADULTERADA paso por buena"; }
printf '%s\n' "$SAL" | grep -q 'cofirma 1' \
  || { printf '%s\n' "$SAL" >&2; fallo "murio, pero sin nombrar QUE cofirma"; }
msg "NEGATIVO-3: exit $RC - $(printf '%s\n' "$SAL" | grep 'cofirma 1' | head -1 || true)"

# == NEGATIVO-4 - un nibble del hashPrueba del acuse =================
# El negativo que este banco ANADE. Va sobre hashPrueba y no sobre `s`
# porque el binario NO lee `s`: un nibble ahi no daria rojo, y un negativo
# que no puede fallar es un adorno con cara de prueba.
msg "NEGATIVO-4: se adultera UN nibble del hashPrueba del acuse"
python3 - "$DIR/paquete.json" "$DIR/n4.json" <<'PYEOF' || fallo "no pude adulterar el hashPrueba"
import json, sys
p = json.load(open(sys.argv[1], encoding="utf-8"))
h = p["acuse"]["hashPrueba"]
ult = h[-1]
nuevo = '0' if ult != '0' else '1'
assert nuevo != ult
p["acuse"]["hashPrueba"] = h[:-1] + nuevo
open(sys.argv[2], "w", encoding="utf-8").write(json.dumps(p))
print("adulterado UN nibble de hashPrueba:", ult, "->", nuevo)
PYEOF
SAL=$("$VB" "$DIR/n4.json" 2>&1); RC=$?
[ "$RC" != "0" ] || { printf '%s\n' "$SAL" >&2; fallo "un acuse ADULTERADO paso por bueno"; }
printf '%s\n' "$SAL" | grep -q 'ROJO: acuse:' \
  || { printf '%s\n' "$SAL" >&2; fallo "murio, pero no por el ACUSE"; }
msg "NEGATIVO-4: exit $RC - el acuse adulterado muere por el acuse"

msg "BANCO-COMPLETO VERDE: cabeza + acuse + $K cofirma(s) en UNA ventana,"
msg "                      logrado en el intento $LOGRADO de $FONDEOS,"
msg "                      y un tercero lo verifica sin el nodo ni el testigo"
# FIN-BANCO-COMPLETO
