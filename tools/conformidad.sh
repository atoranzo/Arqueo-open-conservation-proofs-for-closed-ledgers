#!/usr/bin/env bash
# tools/conformidad.sh - el ARNES DE CONFORMIDAD del paquete de evidencia (RFC-0005, E4, S408).
#
# Corre el manifiesto de vectores del paquete (spec/PAQUETE.md, seccion 9) contra CUALQUIER
# binario que se le pase y dice, entrada a entrada, si el codigo de salida y el texto son los
# que el manifiesto exige. Es el UNICO productor de ese bucle: `tools/canon.sh` (3 bis) y
# `tools/artefacto.sh` lo consumen sobre el binario de referencia; una segunda implementacion
# lo corre sobre el suyo y publica la salida. Viaja dentro del tarball del artefacto.
#
#     bash tools/conformidad.sh <binario> [manifiesto]
#
# <binario>    la ruta de un ejecutable que cumpla el contrato del mando (PAQUETE.md, seccion 6):
#              un argumento -la ruta de un paquete JSON-, exit 0 verde, 1 fallo con nombre, 2 uso,
#              y desde el §573 el 3 del cuarto estado del sobre de completitud: el manifiesto lo
#              pide como pide cualquier otro codigo. Corre con la entrada estandar vacia (§693).
# [manifiesto] por defecto spec/vectors/paquete/MANIFIESTO.txt relativo al directorio actual, que
#              es donde vive tanto en el arbol como dentro del tarball. Los vectores se buscan en
#              el directorio del manifiesto. Formato: fichero|codigo esperado|texto que la salida
#              (stdout+stderr) tiene que CONTENER; las lineas vacias y las que empiezan por # no
#              cuentan. Una entrada cuyo fichero no existe es un vector que falta: se comprueba
#              igual (el binario tiene que rechazar una ruta que no existe). Un mismo fichero
#              puede tener varias entradas, y cada una cuenta.
#
# Desde el §693 (spec/PAQUETE.md, seccion 6.1, «La causa del rechazo») el arnes exige la causa:
#   - NO SOPORTADO. Una salida que dice que el binario no lee ese sobre no cuenta NUNCA, diga lo
#     que diga el manifiesto: un verificador que no verifica no supera un negativo. El texto que el
#     contrato reserva es `NO SOPORTADO`; «no lee este sobre», «no implementado» y «not
#     implemented» se reconocen por cortesia, y otra forma de decirlo -«sin implementar»- no: sale
#     como un ROJO mas, y no supera el negativo porque no dice su causa.
#   - La causa va en la linea del rechazo. En una entrada con codigo 1, la salida tiene UNA linea
#     con `ROJO:` y el texto del manifiesto esta EN ella: no basta con que aparezca en otra.
#   - SIN CAUSA. Un vector con entradas de codigo 1 necesita al menos una cuyo texto nombre la
#     causa: un texto vacio, o que acaba en `:` -el sujeto solo, `edad:`, `cabeza: `-, no la
#     nombra. Es un ROJO del manifiesto, como el vector sin entrada.
#
# Salida: una linea por entrada, `OK   <fichero>` o `ROJO <fichero>: <motivo>`; una entrada no
# soportada es `ROJO <fichero>: NO SOPORTADO - <la linea que lo dice>`, con su nombre como cualquier
# otro ROJO, que es lo que el canon lee; al final la linea `conformidad: <ok> de <n>
# entradas dicen lo que deben (<ficheros> .json, <con entrada> con entrada[; <k> NO SOPORTADAS]
# [; <k> SIN CAUSA]) - binario <sha16>`, donde los dos ultimos solo salen si no son cero. Exit 0
# solo si TODAS las entradas dicen lo que deben y el manifiesto no esta vacio, todo .json del
# directorio tiene su entrada y todo negativo nombra su causa; 1 si alguna falla; 2 uso; 3 si el
# binario no es ejecutable o el manifiesto no existe. Nunca escribe fuera de un directorio
# temporal propio, que borra al salir.
set -uo pipefail
uso(){ echo "uso: bash tools/conformidad.sh <binario> [manifiesto]" >&2; exit 2; }
[ $# -ge 1 ] && [ $# -le 2 ] || uso
BIN="$1"; MAN="${2:-spec/vectors/paquete/MANIFIESTO.txt}"
[ -f "$BIN" ] && [ -x "$BIN" ] || { echo "ROJO: el binario no existe o no es ejecutable: $BIN" >&2; exit 3; }
[ -f "$MAN" ] || { echo "ROJO: el manifiesto no existe: $MAN" >&2; exit 3; }
DIR=$(dirname "$MAN")
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
h16(){ sha256sum "$1" | cut -c1-16; }
# §693: lo que dice que el binario NO lee el sobre. El texto reservado del contrato es `NO SOPORTADO`
# (PAQUETE.md, seccion 6); los otros tres se reconocen por cortesia, y no son todos los posibles.
NO_SOPORTA='no soportad|no lee este sobre|no implementad|not (yet )?implemented'
: > "$TMP/negativos"; : > "$TMP/con_causa"
n=0; ok=0; rojo=0; ns=0; sc=0
while IFS='|' read -r VF VRC VTX; do
  case "$VF" in ''|'#'*) continue;; esac
  n=$((n + 1))
  # §693: con la entrada estandar vacia. Antes heredaba el manifiesto: un binario que la leyera se
  # comia las entradas que quedaban, y el bucle acababa VERDE con las que habia visto.
  "$BIN" "$DIR/$VF" > "$TMP/salida.txt" 2>&1 < /dev/null; VR=$?
  if [ "$VRC" = "1" ]; then
    echo "$VF" >> "$TMP/negativos"
    T="${VTX%"${VTX##*[![:space:]]}"}"
    case "$T" in ''|*:) ;; *) echo "$VF" >> "$TMP/con_causa";; esac
  fi
  if grep -qiE -- "$NO_SOPORTA" "$TMP/salida.txt"; then
    echo "ROJO $VF: NO SOPORTADO - $(grep -iE -m1 -- "$NO_SOPORTA" "$TMP/salida.txt")"; ns=$((ns + 1)); rojo=$((rojo + 1)); continue
  fi
  if [ "$VR" != "$VRC" ]; then echo "ROJO $VF: exit $VR, el manifiesto espera $VRC"; rojo=$((rojo + 1)); continue; fi
  if [ "$VRC" = "1" ]; then
    NR=$(grep -c 'ROJO:' "$TMP/salida.txt" || true)
    if [ "$NR" != "1" ]; then echo "ROJO $VF: $NR lineas con ROJO:, el contrato pide una"; rojo=$((rojo + 1)); continue; fi
    grep 'ROJO:' "$TMP/salida.txt" > "$TMP/linea.txt"
    if grep -qF -- "$VTX" "$TMP/linea.txt"; then echo "OK   $VF"; ok=$((ok + 1))
    elif grep -qF -- "$VTX" "$TMP/salida.txt"; then echo "ROJO $VF: dice '$VTX', pero no en la linea del ROJO"; rojo=$((rojo + 1))
    else echo "ROJO $VF: no dice '$VTX'"; rojo=$((rojo + 1)); fi
    continue
  fi
  if grep -qF -- "$VTX" "$TMP/salida.txt"; then echo "OK   $VF"; ok=$((ok + 1)); else echo "ROJO $VF: no dice '$VTX'"; rojo=$((rojo + 1)); fi
done < "$MAN"
NJ=$(ls "$DIR"/*.json 2>/dev/null | wc -l | tr -d ' ')
NM=$(grep -c '\.json|' "$MAN" || true)
# prueba de vida: hay entradas, y cada .json del directorio tiene la suya (uno sin entrada no gatea nada)
[ "$n" -gt 0 ] || { echo "ROJO manifiesto vacio: $MAN"; rojo=$((rojo + 1)); }
for j in "$DIR"/*.json; do
  [ -e "$j" ] || continue
  grep -qF -- "$(basename "$j")|" "$MAN" || { echo "ROJO vector sin entrada en el manifiesto: $(basename "$j")"; rojo=$((rojo + 1)); }
done
# §693: todo negativo nombra su causa en al menos una de sus entradas (spec/PAQUETE.md, seccion 6.1)
LC_ALL=C sort -u "$TMP/negativos" > "$TMP/n.txt"; LC_ALL=C sort -u "$TMP/con_causa" > "$TMP/c.txt"
while IFS= read -r VF; do
  echo "ROJO $VF: SIN CAUSA - sus entradas de codigo 1 nombran el sujeto, no la causa (PAQUETE.md, seccion 6.1)"
  sc=$((sc + 1)); rojo=$((rojo + 1))
done < <(LC_ALL=C comm -23 "$TMP/n.txt" "$TMP/c.txt")
EXTRA=""
[ "$ns" = "0" ] || EXTRA="$EXTRA; $ns NO SOPORTADAS"
[ "$sc" = "0" ] || EXTRA="$EXTRA; $sc SIN CAUSA"
echo "conformidad: $ok de $n entradas dicen lo que deben ($NJ .json, $NM con entrada$EXTRA) - binario $(h16 "$BIN")"
[ "$rojo" = "0" ] && [ "$ok" = "$n" ]
