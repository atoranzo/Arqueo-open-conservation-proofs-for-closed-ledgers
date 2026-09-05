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
#              un argumento -la ruta de un paquete JSON-, exit 0 verde, 1 fallo con nombre, 2 uso.
# [manifiesto] por defecto spec/vectors/paquete/MANIFIESTO.txt relativo al directorio actual, que
#              es donde vive tanto en el arbol como dentro del tarball. Los vectores se buscan en
#              el directorio del manifiesto. Formato: fichero|codigo esperado|texto que la salida
#              (stdout+stderr) tiene que CONTENER; las lineas vacias y las que empiezan por # no
#              cuentan. Una entrada cuyo fichero no existe es un vector que falta: se comprueba
#              igual (el binario tiene que rechazar una ruta que no existe).
#
# Salida: una linea por entrada, `OK   <fichero>` o `ROJO <fichero>: <motivo>`; al final la linea
# `conformidad: <ok> de <n> entradas dicen lo que deben (<ficheros> .json, <con entrada> con
# entrada) - binario <sha16>`. Exit 0 solo si TODAS las entradas dicen lo que deben y el manifiesto
# no esta vacio y todo .json del directorio tiene su entrada; 1 si alguna falla; 2 uso; 3 si el
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
n=0; ok=0; rojo=0
while IFS='|' read -r VF VRC VTX; do
  case "$VF" in ''|'#'*) continue;; esac
  n=$((n + 1))
  "$BIN" "$DIR/$VF" > "$TMP/salida.txt" 2>&1; VR=$?
  if [ "$VR" != "$VRC" ]; then echo "ROJO $VF: exit $VR, el manifiesto espera $VRC"; rojo=$((rojo + 1)); continue; fi
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
echo "conformidad: $ok de $n entradas dicen lo que deben ($NJ .json, $NM con entrada) - binario $(h16 "$BIN")"
[ "$rojo" = "0" ] && [ "$ok" = "$n" ]
