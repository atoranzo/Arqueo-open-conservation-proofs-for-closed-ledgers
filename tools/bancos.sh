#!/usr/bin/env bash
# =========================== LOS BANCOS ===========================
#
#     bash tools/bancos.sh             # todos los tools/banco_*.sh · **~15 min** (⚠️ §804: menos los de FUERA, punto 5)
#     bash tools/canon.sh --bancos     # lo mismo, desde el canon
#
# **Por que existe (§582).** Los bancos viven FUERA del canon: arrancan el
# nodo, el testigo y el mando reales, y cuestan minutos que no caben en
# cada sello. Por eso el de la reutilizacion estuvo ROJO desde el §337
# hasta el §579 sin que nada lo dijera, y cinco rotulos decian «cabeza v5»
# con el nodo firmando v6. Nadie los corria.
#
# Como el `--completo`, esto NO lo fuerza nadie: es disciplina, no
# compuerta. Lo que se hace por construccion es **no fiarlo a la memoria**:
# cada pasada VERDE deja constancia en `.canon/ultimo-bancos`, y TODA
# invocacion del canon dice cuando fue y si lo que los bancos ejercen ha
# cambiado desde entonces.
#
# ## Lo que lo hace robusto
#
# 1. **La lista se LEE del directorio**: todo `tools/banco_*.sh`. Un banco
#    nuevo entra solo; ninguno se puede olvidar en una lista a mano. ⚠️ §804: la unica lista a mano es la de los que NO corren, `FUERA`, y cada pasada la dice.
# 2. **No se para en el primer fallo**: da el inventario entero.
# 3. **Timeout POR BANCO** (`BANCO_TIMEOUT`, 1800 s): uno que cuelgue no
#    cuelga los demas.
# 4. **El arbol, LIMPIO antes y despues de cada banco.** Un banco mide el
#    arbol del sello; uno que lo ensucia es un fallo, y se nombra.
# 5. **Declarado FUERA**: el `--largo` de la completitud (unos 24 minutos
#    mas, §574) y el del recibo sin resolver (otros tantos, §602). Aqui
#    corren sus modos por defecto. ⚠️ §804: y FUERA entero, por su nombre, `tools/banco_cortes.sh`: mide seis commits fijos de la historia, no el arbol del sello, y compila seis veces.
#
# ================================================================

if [ -z "${BASH_VERSION:-}" ]; then
  echo "bancos.sh necesita bash: usa 'bash tools/bancos.sh', no 'sh'." >&2
  exit 2
fi
set -uo pipefail

RAIZ="${CANON_RAIZ:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
cd "$RAIZ" || exit 2
BANCOS_FILE=".canon/ultimo-bancos"
LOGS="${OUT:-/tmp/bancos}"
TOPE="${BANCO_TIMEOUT:-1800}"

msg() { echo "$*" >&2; }
fallos=()

sucio() { git status --porcelain -uall; }
[ -z "$(sucio)" ] || {
  msg "bancos: el arbol NO esta limpio; un banco mide el arbol de un sello:"
  sucio | head -5 | sed 's/^/    /' >&2
  exit 2
}

# Punto 5 (§804): los que no miden el arbol del sello, por su nombre. El de los cortes repite el
# §699 sobre seis commits fijos: su resultado no lo cambia un sello, y entero tardo 1.251 s en el
# §804, 1.115 de ellos en sus seis compilaciones, con la maquina compartida: otros veinte minutos
# sobre los de esta pasada. Se corre aparte, y cada pasada lo nombra.
FUERA=(tools/banco_cortes.sh)
mapfile -t BANCOS < <(ls tools/banco_*.sh 2>/dev/null | sort | grep -vxF -f <(printf '%s\n' "${FUERA[@]}"))
[ "${#BANCOS[@]}" -gt 0 ] || { msg "bancos: ningun tools/banco_*.sh"; exit 2; }
mkdir -p "$LOGS"
msg "== BANCOS · ${#BANCOS[@]} bancos, uno a uno, sobre $(git rev-parse --short HEAD) =="
for f in "${FUERA[@]}"; do [ -f "$f" ] && msg "   FUERA (punto 5): $f, aparte: bash $f"; done
msg "   banco                     exit     seg"
T_INI=$(date +%s)
for b in "${BANCOS[@]}"; do
  n=$(basename "$b" .sh); n=${n#banco_}
  t0=$(date +%s)
  timeout "$TOPE" bash "$b" > "$LOGS/$n.log" 2>&1; rc=$?
  t=$(( $(date +%s) - t0 ))
  printf '   %-24s %5s %7s\n' "$n" "$rc" "$t" >&2
  if [ "$rc" = "124" ]; then
    fallos+=("$n: pasados $TOPE s sin acabar (timeout)")
  elif [ "$rc" != "0" ]; then
    fallos+=("$n: exit $rc — ver $LOGS/$n.log")
  fi
  s=$(sucio)
  if [ -n "$s" ]; then
    fallos+=("$n: dejo el arbol SUCIO: $(head -3 <<< "$s" | tr '\n' ' ')")
    msg "   ⚠️ $n dejo el arbol sucio; los demas no se corren sobre el"
    break
  fi
done
msg "   ── total: $(( $(date +%s) - T_INI )) s"

msg ""
if [ "${#fallos[@]}" -eq 0 ]; then
  mkdir -p "$(dirname "$BANCOS_FILE")"
  foto="$(git rev-parse --short HEAD) $(date -Iseconds) ${#BANCOS[@]}"
  printf '%s\n' "$foto" > "$BANCOS_FILE"
  msg "== BANCOS: VERDE · ${#BANCOS[@]} de ${#BANCOS[@]} =="
  msg "   anotado en $BANCOS_FILE — para que nadie tenga que acordarse."
  msg "   salida integra en $LOGS/"
  exit 0
fi
msg "== BANCOS: ROJO · ${#fallos[@]} fallo(s) =="
for f in "${fallos[@]}"; do msg "   · $f"; done
msg "   salida integra en $LOGS/"
exit 1
