#!/usr/bin/env bash
# =========================== EL CANON ===========================
#
# **La fuente unica.** Antes de §224 cada BLOQUE reescribia sus compuertas
# a mano, y por eso `zk-ssl-wire` paso SEIS sellos sin que nadie corriera
# sus tests. Ahora hay un fichero, esta en el repo, y los bloques lo
# invocan:
#
#     bash tools/canon.sh --sello     # nivel de sello (cada bloque)
#     bash tools/canon.sh --largo     # todo, incluidos los caros
#     bash tools/canon.sh --lista     # solo enseña la tabla, no ejecuta
#
# ## Lo que hace ROBUSTO a esto, y no es la tabla
#
# 1. **Los miembros del workspace se LEEN de Cargo.toml.** Un crate nuevo
#    sin fila en la tabla pone la compuerta ROJA, y una fila sin crate
#    tambien. El agujero de §223 —un crate entero fuera de las
#    compuertas— pasa de "hay que acordarse" a **imposible**.
# 2. **Pines EXACTOS, no minimos.** Si alguien arregla un warning o añade
#    un test, esto se pone rojo y hay que actualizar el pin A PROPOSITO.
#    Nada mejora ni empeora en silencio.
# 3. **No se para en el primer fallo.** Acumula y da el inventario
#    completo: un banco que se para en el primer susto oculta los otros.
# 4. **Timeout POR CRATE.** `zk-core` no termina —73 de 76 y a los 1800 s
#    sigue— y esta pinchado COMO ANOMALIA, no escondido. Un crate que
#    cuelga no puede colgar la compuerta.
# 5. **Anclas ASCII.** `.` de grep casa un BYTE, no un caracter: un ancla
#    con tilde no falla, se queda MUDA (§223).
# 6. **Dice exactamente que linea editar** cuando un pin no cuadra.
#
# ## Como se actualiza un pin
#
# Se MIDE primero y se edita la tabla despues. Nunca al reves: fijar un
# numero sin medirlo es lo que esta casa lleva ocho sellos evitando.
#
# ================================================================
# ⚠️ La guarda va ANTES de cualquier bashismo: con `sh` el `set -o pipefail`
# de abajo revienta primero y el aviso no llega a imprimirse nunca. Lo cazo
# el ensayo, no el recuerdo.
if [ -z "${BASH_VERSION:-}" ]; then
  echo "canon.sh necesita bash: usa 'bash tools/canon.sh', no 'sh'." >&2
  exit 2
fi

set -uo pipefail

# La raiz sale de la ruta del propio fichero. `CANON_RAIZ` la sobreescribe,
# y existe para PROBAR esta herramienta desde fuera del repo: una copia
# mutilada en /tmp resolveria su raiz a /tmp y no vigilaria nada. Una
# compuerta que no se puede probar en negativo no es una compuerta.
RAIZ="${CANON_RAIZ:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
cd "$RAIZ" || exit 2
NIVEL="${1:---sello}"

# ── LA TABLA ────────────────────────────────────────────────────
# crate | nivel | pasan | ignorados | warnings | timeout_s | nota
#
# Medido en §224 (banco F.1) sobre el sello 7a33e58, en release.
# `pasan` e `ignorados` son lo que el arnes EJECUTA, no los `#[test]`
# declarados: en zk-ssl son 259 declarados, 256 pasan y 3 son
# instrumentos de medida.
TABLA=$(cat <<'FIN_TABLA'
zk-ssl             sello  256   3   0   600  la capa
stark-experiment   sello  297  10   0   600  los circuitos
ceremony           sello   34   0  11   300  DEUDA: 11 warnings, pinchados para que no crezcan
settlement-layer   sello   17   0   0   300
iso-bridge         sello    3   0   0   300
zk-ssl-sdk         sello    6   0   0   300
zk-ssl-wire        sello    2   0   0   300  fuera de toda compuerta hasta §223
zk-ssl-cli         sello    0   0   0   300  sin tests: es un binario
zk-ssl-node        sello    0   0   0   300  sin tests: HUECO DECLARADO, esta en la cola
settlement-prover  sello    0   0   0   300  sin tests
nova-experiment    sello    0   0   0   300  0 es CORRECTO: sus 3 tests exigen --features test-setup
halo2-experiment   largo   27   0   0  1200  caro: 438 s medidos
plonk-experiment   largo   36   0   3  1800  caro: 749 s medidos. DEUDA: 3 warnings
zk-core            largo   73   0  10  1800  ⚠️ NO TERMINA: 73 de 76 y timeout. Pinchado COMO ANOMALIA
FIN_TABLA
)

# `zk-core` no termina: su fila espera exit 124 (timeout), no 0. Si algun
# dia termina, esta compuerta se pone roja y habra que mirar por que —
# que es exactamente lo que se quiere.
ANOMALIA_TIMEOUT="zk-core"

# ── utilidades ──────────────────────────────────────────────────
rojo=0
fallos=()
msg() { echo "$*" >&2; }
falla() { rojo=1; fallos+=("$1"); msg "  XX  $1"; }

miembros_del_workspace() {
  sed -n '/^\[workspace\]/,/^\[[^w]/p' Cargo.toml \
    | grep -oE '"crates/[a-z0-9-]+"' | tr -d '"' | sed 's|crates/||'
}

fila_de() { awk -v c="$1" '$1==c {print; exit}' <<< "$TABLA"; }

# ── 0 · COHERENCIA: la tabla y el workspace dicen lo mismo ──────
msg "== CANON · coherencia con el workspace =="
MIEMBROS=$(miembros_del_workspace)
EN_TABLA=$(awk 'NF{print $1}' <<< "$TABLA")
n_m=$(wc -l <<< "$MIEMBROS"); n_t=$(wc -l <<< "$EN_TABLA")
msg "  miembros en Cargo.toml: $n_m · filas en la tabla: $n_t"
# ⚠️ Here-string, NO tuberia. `echo X | grep -q` es INTERMITENTE con
# `pipefail`: grep -q sale en cuanto encuentra, echo recibe SIGPIPE y
# devuelve 141, y pipefail lo propaga como fallo de la tuberia AUNQUE la
# busqueda haya acertado. Una compuerta intermitente es peor que ninguna:
# enseña a ignorarla. Lo cazo el ensayo — daba stark-experiment por
# ausente estando presente.
for c in $MIEMBROS; do
  grep -qx "$c" <<< "$EN_TABLA" || falla "el crate '$c' esta en el workspace y NO en el canon"
done
for c in $EN_TABLA; do
  grep -qx "$c" <<< "$MIEMBROS" || falla "la fila '$c' no corresponde a ningun crate del workspace"
done
[ $rojo -eq 0 ] && msg "  OK  todos los crates tienen fila y todas las filas tienen crate"

if [ "$NIVEL" = "--lista" ]; then
  msg ""
  msg "== LA TABLA =="
  sed 's/^/  /' <<< "$TABLA" >&2
  exit $rojo
fi

# ── 1 · los tests, crate a crate ────────────────────────────────
OUT="${OUT:-/tmp/canon}"
rm -rf "$OUT"; mkdir -p "$OUT"
msg ""
msg "== CANON $NIVEL · tests =="
msg "   crate                exit  pasan(pin) ignor(pin) warn(pin)   seg"
T_TOTAL=0
while read -r c niv pasan ign warn tmo resto; do
  [ -n "$c" ] || continue
  if [ "$NIVEL" = "--sello" ] && [ "$niv" != "sello" ]; then continue; fi
  T0=$(date +%s)
  timeout "${tmo}s" cargo test -p "$c" --release > "$OUT/$c.txt" 2>&1
  RC=$?
  T1=$(date +%s); T_TOTAL=$((T_TOTAL + T1 - T0))
  P=$(grep -oE "[0-9]+ passed" "$OUT/$c.txt" | grep -oE "[0-9]+" | paste -sd+ | bc 2>/dev/null); P=${P:-0}
  I=$(grep -oE "[0-9]+ ignored" "$OUT/$c.txt" | grep -oE "[0-9]+" | paste -sd+ | bc 2>/dev/null); I=${I:-0}
  W=$(grep -c "^warning" "$OUT/$c.txt")
  printf "   %-20s %4s %6s(%s) %6s(%s) %5s(%s) %5s\n" "$c" "$RC" "$P" "$pasan" "$I" "$ign" "$W" "$warn" "$((T1-T0))" >&2
  # exit esperado: 0, salvo la anomalia declarada
  esperado=0
  [ "$c" = "$ANOMALIA_TIMEOUT" ] && esperado=124
  [ "$RC" = "$esperado" ] || falla "$c: exit $RC, el canon espera $esperado  ->  tools/canon.sh, fila '$c'"
  [ "$P" = "$pasan" ] || falla "$c: pasan $P, el canon dice $pasan  ->  MEDIR y editar tools/canon.sh, fila '$c'"
  [ "$I" = "$ign" ]   || falla "$c: ignorados $I, el canon dice $ign  ->  tools/canon.sh, fila '$c'"
  [ "$W" = "$warn" ]  || falla "$c: warnings $W, el canon dice $warn  ->  tools/canon.sh, fila '$c'"
done <<< "$TABLA"
msg "   ── total: ${T_TOTAL} s"

# ── 2 · tests que no protegen ───────────────────────────────────
msg ""
msg "== CANON · tests que no protegen =="
python3 tools/check_tests.py > "$OUT/check_tests.txt" 2>&1
if [ $? -eq 0 ]; then
  msg "  OK  $(tail -2 "$OUT/check_tests.txt" | tr '\n' ' ')"
else
  sed 's/^/      /' "$OUT/check_tests.txt" >&2
  falla "check_tests.py encontro tests que no protegen"
fi

# ── 3 · conformidad ─────────────────────────────────────────────
msg ""
msg "== CANON · conformidad =="
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.2.json > "$OUT/c02.txt" 2>&1
if grep -q "todo IDENTICO" "$OUT/c02.txt"; then msg "  OK  0.2 -> todo IDENTICO"; else falla "0.2 NO da 'todo IDENTICO'"; fi
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.1.json > "$OUT/c01.txt" 2>&1
if [ $? -ne 0 ]; then msg "  OK  0.1 RECHAZADO"; else falla "0.1 deberia rechazarse y no lo hace"; fi

# ── veredicto ───────────────────────────────────────────────────
msg ""
if [ $rojo -eq 0 ]; then
  msg "== CANON $NIVEL: VERDE =="
else
  msg "== CANON $NIVEL: ROJO · ${#fallos[@]} fallo(s) =="
  for f in "${fallos[@]}"; do msg "   · $f"; done
fi
msg "   salida integra en $OUT/"
exit $rojo
