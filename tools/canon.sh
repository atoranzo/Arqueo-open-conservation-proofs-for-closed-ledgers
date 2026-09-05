#!/usr/bin/env bash
# =========================== EL CANON ===========================
#
# **La fuente unica.** Antes de §224 cada BLOQUE reescribia sus compuertas
# a mano, y por eso `zk-ssl-wire` paso SEIS sellos sin que nadie corriera
# sus tests.
#
#     bash tools/canon.sh --sello      # cada bloque · **151 s**
#     bash tools/canon.sh --largo      # + halo2 y plonk · **~20 min**
#     bash tools/canon.sh --completo   # + zk-core · **53,6 min**
#
# ⚠️ Estos tiempos los midio el CANON, no los bancos (§226, primera
# pasada de `--completo`): 151 + 347 (halo2) + 689 (plonk) + 2.032
# (zk-core) = **3.217 s**. Los bancos F.1 y G.2 daban cifras mayores
# porque midieron en FRIO; el canon mide con el `target` caliente,
# que es como se corre de verdad. La diferencia no es pequeña:
# zk-core 2.032 s aqui frente a los 2.317 que sumaban las dos
# invocaciones separadas de G.2.
#     bash tools/canon.sh --lista      # solo enseña la tabla, no ejecuta
#
# ## Lo que lo hace ROBUSTO, y no es la tabla
#
# 1. **Los miembros del workspace se LEEN de Cargo.toml.** Un crate nuevo
#    sin fila pone la compuerta ROJA, y una fila sin crate tambien. El
#    agujero de §223 pasa de "hay que acordarse" a **imposible**.
# 2. **Pines EXACTOS, no minimos.** Nada mejora ni empeora en silencio.
# 3. **No se para en el primer fallo:** da el inventario completo.
# 4. **Timeout POR CRATE.** Un crate que cuelgue no cuelga la compuerta.
# 5. **Anclas ASCII.** `.` de grep casa un BYTE: un ancla con tilde no
#    falla, se queda MUDA (§223).
# 6. **Here-string, nunca `echo | grep`**: con `pipefail`, `grep -q` sale
#    al encontrar, `echo` recibe SIGPIPE y devuelve 141 aunque la busqueda
#    acierte. Una compuerta intermitente es peor que ninguna.
# 7. **Dice que linea editar** cuando un pin no cuadra.
# 8. **Ningun `.rs` bajo `src/` puede quedar sin declarar** (§227). Lo que
#    no se compila no se verifica — y encima invita a especular.
#
# ## Los TRES niveles, y por que tres (§225)
#
# `zk-core` cuesta **33,9 minutos** medidos por el canon (2.032 s). G.2
# lo midio en dos invocaciones separadas y en frio: 1.472 s los 73 tests
# de biblioteca con 8 hilos mas **845 s la ceremonia**. Y no hay hilo
# que lo arregle: medido, generar la prueba añade un 5 % y rayon un
# 17 % — **el coste es SINTETIZAR el circuito**, y `ark-r1cs-std`
# sintetiza en serie.
#
# Meter 54 minutos en cada sello garantizaria que alguien acabe saltandose
# la compuerta, y una compuerta que se salta no protege. Es exactamente lo
# que le paso a `check_tests.py`.
#
# ⚠️ **El nivel `--completo` NO lo fuerza nadie.** Es disciplina, no
# compuerta, y la disciplina es justo lo que ha fallado seis veces en este
# proyecto. Lo unico que se puede hacer por construccion es **no fiarlo a
# la memoria**: cada `--completo` que pasa deja constancia en
# `.canon/ultimo-completo`, y TODA invocacion dice cuando fue y cuantos
# sellos han pasado desde entonces.
#
# ## Como se actualiza un pin
#
# Se MIDE primero y se edita la tabla despues. Nunca al reves.
#
# ================================================================

# ⚠️ La guarda va ANTES de cualquier bashismo: con `sh` el `set -o pipefail`
# de abajo revienta primero y el aviso no llega a imprimirse nunca.
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
SELLO_FILE=".canon/ultimo-completo"

# ── LA TABLA ────────────────────────────────────────────────────
# crate | nivel | pasan | ignorados | warnings | timeout_s | nota
#
# Medido en §224 (banco F.1) y corregido en §225 (bancos G.1 y G.2) sobre
# el sello 06106c9, en release. `pasan` e `ignorados` son lo que el arnes
# EJECUTA, no los `#[test]` declarados.
TABLA=$(cat <<'FIN_TABLA'
zk-ssl             sello     307   3   0   600  alias=capa · la capa · §292: 262 -> 263, la cima en el digest · §304: 263 -> 264, el atado de la cifra publicada · §340: 264 -> 265, la altura entra en el digest · §345: 265 -> 269, el compositor v2 (RFC-0003, E1a) · §350: 269 -> 272, el cobro aprende el sobre (E3b-1 del RFC-0003) · §351: 272 -> 276, la apertura en el recibo (E3b-2 del RFC-0003) · §353: 276 -> 279, la via viva aprende el sobre (E3c-1b del RFC-0003) · §355: 279 -> 284, la guarda de ancho llega a los gemelos · §357: 284 -> 285, el testigo funcional de la des-emision (deuda del §351) · §359: 285 -> 286, el export del snapshot falla cerrado · §362: 286 -> 287, el reloj sale del atado a su contrato propio · §379: 287 -> 289, la conservacion del dinero se comprueba al abrir · §387: 289 -> 292, la raiz del pendiente se guarda y se comprueba al abrir · §388: 292 -> 294, la raiz de meta del pendiente se guarda y se comprueba al abrir · §390: 294 -> 295, descongelar sobrevive al reinicio · §391: 295 -> 298, la raiz de congelados en reposo · §392: 298 -> 301, la raiz del registro en reposo · §393: 301 -> 304, los contadores custodiados los deriva el registro · §394: 304 -> 307, la cuota de custodios la deriva el registro
stark-experiment   sello     318  10   0   600  alias=circuitos · los circuitos · §345: 297 -> 299, el gemelo nativo v2 (RFC-0003, E1a) · §346: 299 -> 304, RefundAirV2 en paralelo (E2a del RFC-0003) · §347: 304 -> 310, ClaimAirV2 en paralelo (E2b del RFC-0003) · §348: 310 -> 312, la puerta de E2 saldada (ladron-con-aviso y tercero-como-retorno) · §352: 312 -> 318, el envio aprende el sobre (E3c-1a del RFC-0003)
ceremony           sello      34   0  11   300  alias=ceremonia · DEUDA: 11 warnings, pinchados para que no crezcan
settlement-layer   sello      17   0   0   300  alias=liquidación
iso-bridge         sello       3   0   0   300
zk-ssl-sdk         sello       6   0   0   300
zk-ssl-wire        sello      15   0   0    60  §259: 20 metodos, y el json publicado atado a la tabla · §311: 3 -> 7, el DTO de la cabeza firmada · §312: 7 -> 10, el accesor falible de la cabeza firmada · §313: 10 -> 13, los tres constructores nombrados · §315: 13 -> 15, la cofirma en el cable
zk-ssl-cli         sello      95   0   0   120  alias=testigo · §250: 22 -> 26, la vista dividida EN FRIO · §294: 28 -> 39, la historia como segundo canal · §295: 39 -> 43, el banco del consumidor · §299: 43 -> 51, el cofirmante del testigo · §300: 51 -> 54, el mando de la cofirma · §301: 54 -> 58, la herramienta del tercero · §310: 58 -> 59, la serie de cofirmas es por testigo · §312: 59 -> 61, el testigo consume la cabeza tipada · §316: 61 -> 62, el testigo envia su cofirma al nodo · §319: 62 -> 66, la politica que nombra y la k que falla cerrada · §320: 66 -> 71, la recoleccion de cofirmas · §314: 71 -> 75, la ausencia de respuesta es una clase · §333: 75 -> 79, la clave de la serie pasa al indice embebido · §334: 79 -> 81, la marca solo se dispara sobre lo que prueba · §336: 81 -> 86, la politica del cofirmante sale de fn run y nace con tests · §337: 86 -> 91, el testigo vuelve a arrancar tras cofirmar · §404: 91 -> 92, otra version no se cree · §406: 92 -> 93, el testigo lee la version por VersionCabeza antes de la firma · §409: 93 -> 94, el mando de --respuesta y el texto del veredicto · §411: 94 -> 95, los KAT del nucleo
zk-ssl-node        sello      91   0   0   400  alias=nodo · §261: 51 -> 56, la credencial para los caminos · §292: 79 -> 80, la cima en el digest · §293: 80 -> 82, la extension servida · §296: 82 -> 73, el guardian se muda a su crate · §298: 73 -> 71, la lectura del indice se muda · §302: 71 -> 72, el contrato publicado atado al despacho · §309: 72 -> 73, el conjunto de claves servidas, declarado · §315: 73 -> 76, el transporte de la cofirma · §317: 76 -> 77, la retencion de las cofirmas, atada · §318: 77 -> 80, el disparador de la acumulacion · §328: 80 -> 83, el arranque reconcilia · §331: 83 -> 84, la clave en cero no arranca · §335: 84 -> 91, la clave vuelve a su indice y el contador se ata al diario
zk-ssl-verify      sello      79   0   0   120  alias=verificador independiente · §257: 22 -> 23, del cable a la cabeza sin la capa · §291: 39 -> 46, el MMR de cabezas · §292: 46 -> 48, la cima en el digest · §297: 48 -> 55, el objeto de la cofirma · §322: 55 -> 61, las cofirmas dentro del paquete de evidencia · §324: 61 -> 62, la mudanza de COFIRMA_VERSION · §332: 62 -> 66, el indice declarado se ata al que va dentro de la firma · §335: 66 -> 69, el apano del OID pasa a tener un solo dueno · S395: 69 -> 72, la autonomia del verificador deja de ser prosa · §399: 72 -> 75, el indice declarado de la cabeza se ata al que va dentro de la firma · §406: 75 -> 79, el conjunto de versiones tiene un solo productor
zk-ssl-hash        sello      24   0   0    60  §270: 14 -> 16, el acuse componible por un tercero · §292: 22 -> 24, la cima en el digest
zk-ssl-guardian    sello       26   0   0    60  §296: el guardian del indice, MUDADO del nodo (sus 9 tests salen del pin 82) · §298: 9 -> 11, y llega con sus dos tests de layout · §330: 11 -> 19, el lector comun de la semilla · §331: 19 -> 22, ClaveEnCero y su guarda · §335: 22 -> 25, el escritor del indice y su techo derivado del ancho · §336: 25 -> 26, el invariante del arranque tiene dueno
settlement-prover  sello       0   0   0   300  sin tests
nova-experiment    sello       0   0   0   300  0 es CORRECTO: sus 3 tests exigen --features test-setup
halo2-experiment   largo      27   0   0  1200  caro: 438 s medidos
plonk-experiment   largo      36   0   3  1800  caro: 749 s medidos. DEUDA: 3 warnings
zk-core            completo   74   0  10  3600  2075 s medidos en 6cb8883 (§260). La nota anterior (§225) decia 38,6 min = 1472 lib + 845 ceremonia: envejecio A LA BAJA. Los 10 warnings son de `ceremony`, no suyos
FIN_TABLA
)

# ── lo que cubre el nivel --completo, y nadie mas (§284) ────────
# La regla vieja «tocar un Cargo.* vence la foto» era SINTACTICA donde el
# riesgo es SEMANTICO: sobredisparaba sobre todo el proyecto salvo esto.
# `--completo` añade UNA fila sobre --sello: zk-core (su fila, arriba).
# Cierre real, leido de crates/zk-core/Cargo.toml: settlement-prover
# (path), ceremony (dev-dependency a proposito, grafo aciclico) y los
# ark-* que pincha el [workspace.dependencies] del Cargo.toml RAIZ.
#   - VENCE la foto: un cambio bajo estas rutas, o lineas ark-* en el
#     diff del Cargo.toml raiz.
#   - NO la vence (declarado ANTES de mirar nada): el Cargo.lock solo
#     —un lock movido sin toml cubierto imprime que VALE, con la razon—
#     ni la raiz tocada sin ark-*.
COMPLETO_CUBRE="crates/zk-core crates/settlement-prover crates/ceremony"

# ── utilidades ──────────────────────────────────────────────────
rojo=0
fallos=()
msg() { echo "$*" >&2; }
falla() { rojo=1; fallos+=("$1"); msg "  XX  $1"; }

nivel_num() {
  case "$1" in
    sello) echo 1 ;; largo) echo 2 ;; completo) echo 3 ;; *) echo 9 ;;
  esac
}

miembros_del_workspace() {
  sed -n '/^\[workspace\]/,/^\[[^w]/p' Cargo.toml \
    | grep -oE '"crates/[a-z0-9-]+"' | tr -d '"' | sed 's|crates/||'
}

estado_completo() {
  if [ -f "$SELLO_FILE" ]; then
    local c f n toc raiz lock ark
    read -r c f < "$SELLO_FILE"
    n=$(git rev-list --count "$c..HEAD" 2>/dev/null || echo "?")
    if [ "$n" = "0" ]; then
      msg "  ultimo --completo: $c ($f) · AL DIA"
    else
      msg "  ultimo --completo: $c ($f) · **$n sello(s) por detras de HEAD**"
      if [ "$n" = "?" ]; then
        msg "  la foto apunta a $c y ese sello ya no se alcanza: se remide con --completo"
      else
        toc=$(git diff --name-only "$c..HEAD" -- $COMPLETO_CUBRE 2>/dev/null)
        raiz=$(git diff --name-only "$c..HEAD" -- Cargo.toml 2>/dev/null)
        lock=$(git diff --name-only "$c..HEAD" -- Cargo.lock 2>/dev/null)
        ark=0
        [ -n "$raiz" ] && ark=$(git diff "$c..HEAD" -- Cargo.toml 2>/dev/null | grep -cE '^[+-].*ark-')
        if [ -n "$toc" ]; then
          msg "  ⚠️ VENCIDA: lo cubierto cambio desde la foto — toca --completo:"
          sed 's/^/      /' <<< "$toc" >&2
        elif [ "${ark:-0}" -gt 0 ]; then
          msg "  ⚠️ VENCIDA: el Cargo.toml raiz movio lineas ark-* ($ark) — toca --completo"
        elif [ -n "$raiz" ]; then
          msg "  la raiz cambio sin mover ark-*: por detras pero limpio en lo cubierto, la foto VALE"
        elif [ -n "$lock" ]; then
          msg "  el lock se movio sin un Cargo.toml cubierto: por detras pero limpio en lo cubierto, la foto VALE"
        else
          msg "  por detras pero limpio en lo cubierto: la foto VALE"
        fi
      fi
    fi
  else
    msg "  ultimo --completo: **NUNCA**. Los 54 min del nivel completo no los ha corrido nadie."
  fi
}

case "$NIVEL" in
  --sello|--largo|--completo|--lista) : ;;
  *) msg "nivel desconocido: $NIVEL. Usa --sello, --largo, --completo o --lista."; exit 2 ;;
esac

# ── 0 · COHERENCIA: la tabla y el workspace dicen lo mismo ──────
msg "== CANON · coherencia con el workspace =="
MIEMBROS=$(miembros_del_workspace)
EN_TABLA=$(awk 'NF{print $1}' <<< "$TABLA")
n_m=$(wc -l <<< "$MIEMBROS"); n_t=$(wc -l <<< "$EN_TABLA")
msg "  miembros en Cargo.toml: $n_m · filas en la tabla: $n_t"
# ⚠️ Here-string, NO tuberia: ver el punto 6 de la cabecera.
for c in $MIEMBROS; do
  grep -qx "$c" <<< "$EN_TABLA" || falla "el crate '$c' esta en el workspace y NO en el canon"
done
for c in $EN_TABLA; do
  grep -qx "$c" <<< "$MIEMBROS" || falla "la fila '$c' no corresponde a ningun crate del workspace"
done
[ $rojo -eq 0 ] && msg "  OK  todos los crates tienen fila y todas las filas tienen crate"
estado_completo

if [ "$NIVEL" = "--lista" ]; then
  msg ""
  msg "== LA TABLA =="
  sed 's/^/  /' <<< "$TABLA" >&2
  exit $rojo
fi

# ── 1 · los tests, crate a crate ────────────────────────────────
PEDIDO=$(nivel_num "${NIVEL#--}")
OUT="${OUT:-/tmp/canon}"
rm -rf "$OUT"; mkdir -p "$OUT"
msg ""
msg "== CANON $NIVEL · tests =="
msg "   crate                exit  pasan(pin) ignor(pin) warn(pin)   seg"
T_TOTAL=0
while read -r c niv pasan ign warn tmo resto; do
  [ -n "$c" ] || continue
  [ "$(nivel_num "$niv")" -le "$PEDIDO" ] || continue
  T0=$(date +%s)
  timeout "${tmo}s" cargo test -p "$c" --release > "$OUT/$c.txt" 2>&1
  RC=$?
  T1=$(date +%s); T_TOTAL=$((T_TOTAL + T1 - T0))
  P=$(grep -oE "[0-9]+ passed" "$OUT/$c.txt" | grep -oE "[0-9]+" | paste -sd+ | bc 2>/dev/null); P=${P:-0}
  I=$(grep -oE "[0-9]+ ignored" "$OUT/$c.txt" | grep -oE "[0-9]+" | paste -sd+ | bc 2>/dev/null); I=${I:-0}
  W=$(grep -c "^warning" "$OUT/$c.txt")
  printf "   %-20s %4s %6s(%s) %6s(%s) %5s(%s) %5s\n" "$c" "$RC" "$P" "$pasan" "$I" "$ign" "$W" "$warn" "$((T1-T0))" >&2
  [ "$RC" = "0" ]     || falla "$c: exit $RC  ->  tools/canon.sh, fila '$c'"
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

# ── 2 bis · ficheros .rs que no declara nadie ──
# Un `.rs` sin declarar NO se compila, asi que nada lo verifica — y encima
# invita a especular sobre el. `marlin_proof_system.rs` estuvo asi, y se
# sello DOS VECES una descripcion falsa de lo que era (§227).
msg ""
msg "== CANON · ficheros .rs que no declara nadie =="
python3 tools/check_modulos.py > "$OUT/check_modulos.txt" 2>&1
if [ $? -eq 0 ]; then
  msg "  OK  $(tail -1 "$OUT/check_modulos.txt")"
else
  sed 's/^/      /' "$OUT/check_modulos.txt" >&2
  falla "hay ficheros .rs que nadie declara"
fi

# ── 2 ter · las herramientas del bucle ──────────────────────────
# Hasta 269 el canon invocaba DOS de las siete de `tools/`. Las otras cinco
# las corria cada BLOQUE a mano, y una herramienta que nadie recuerda meter
# en un bloque es una herramienta que nadie corre: DOS llevaban rojas sin
# que nadie lo viera. Es exactamente lo que le paso a `check_tests.py`, y
# por lo que esta aqui arriba.
#
# Entran a PIN CERO. No hay pin de fallos: los rojos que tenian eran defecto
# de las propias herramientas —exclusiones que les faltaban y un patron que
# tomaba una plantilla por cita—, no deuda de documentacion, y 269 los
# arreglo ANTES de cablearlas.
msg ""
msg "== CANON · las herramientas de tools/ =="
for H in check_cifras check_figures check_columns check_constraint_layout verificar_citas check_dominios check_publicadas check_nucleo; do
  python3 "tools/$H.py" > "$OUT/$H.txt" 2>&1
  if [ $? -eq 0 ]; then
    msg "  OK  $H"
  else
    sed 's/^/      /' "$OUT/$H.txt" >&2
    falla "$H.py en rojo"
  fi
done

# ── 3 · conformidad ─────────────────────────────────────────────
msg ""
msg "== CANON · conformidad =="
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.3.json > "$OUT/c03.txt" 2>&1
if grep -q "todo IDENTICO" "$OUT/c03.txt"; then msg "  OK  0.3 -> todo IDENTICO"; else falla "0.3 NO da 'todo IDENTICO'"; fi
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.2.json > "$OUT/c02.txt" 2>&1
if grep -q "OTRA version" "$OUT/c02.txt"; then msg "  OK  0.2 RECHAZADO (otra version)"; else falla "0.2 deberia rechazarse por version"; fi
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.1.json > "$OUT/c01.txt" 2>&1
if grep -q "OTRA version" "$OUT/c01.txt"; then msg "  OK  0.1 RECHAZADO (otra version)"; else falla "0.1 deberia rechazarse por version"; fi

# ── 3 bis · conformidad del PAQUETE (spec/PAQUETE.md, RFC-0004 E2, §398; el arnes: RFC-0005 E4, §408) ──
msg ""
msg "== CANON · conformidad del paquete =="
cargo build --release -q -p zk-ssl-verify > "$OUT/paquete_build.txt" 2>&1 || falla "zk-ssl-verify no compila en release"
# UN solo productor del bucle del manifiesto: tools/conformidad.sh, el mismo que un tercero corre
# sobre cualquier binario y el que artefacto.sh corre sobre el binario con remap. Cada ROJO del
# arnes entra aqui con su nombre; la prueba de vida (manifiesto vacio, vector sin entrada) es suya.
if bash tools/conformidad.sh target/release/zk-ssl-verify > "$OUT/paquete.txt" 2>&1; then
  msg "  OK  paquete: $(tail -n 1 "$OUT/paquete.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "paquete $L"; done < <(grep '^ROJO' "$OUT/paquete.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/paquete.txt" || falla "paquete: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/paquete.txt"))"
fi

# ── 3 bis cable · los rechazos del CABLE (RFC-0005 E3, §409): el MISMO arnes, otro manifiesto ──
msg ""
msg "== CANON · los rechazos del cable =="
cargo build --release -q -p zk-ssl-cli > "$OUT/cable_build.txt" 2>&1 || falla "zk-ssl-cli no compila en release"
# tools/cable_respuesta.sh es el adaptador al contrato del mando: un argumento, exit 0/1/2; una
# segunda implementacion del consumidor pone su ejecutable en su lugar. Cada ROJO entra por falla.
if bash tools/conformidad.sh tools/cable_respuesta.sh spec/vectors/cable/MANIFIESTO.txt > "$OUT/cable.txt" 2>&1; then
  msg "  OK  cable: $(tail -n 1 "$OUT/cable.txt" | sed 's/^conformidad: //')"
else
  while IFS= read -r L; do falla "cable $L"; done < <(grep '^ROJO' "$OUT/cable.txt" | sed 's/^ROJO //')
  grep -q '^ROJO' "$OUT/cable.txt" || falla "cable: el arnes falla sin nombrar la entrada ($(tail -n 1 "$OUT/cable.txt"))"
fi

# ── 3 ter · el ARTEFACTO (tools/artefacto.sh --check, §401): la PROPIEDAD, no un pin ──
msg ""
msg "== CANON · el artefacto =="
T_ART0=$(date +%s)
if bash tools/artefacto.sh --check > "$OUT/artefacto.txt" 2>&1; then
  msg "  OK  $(tail -n 1 "$OUT/artefacto.txt" | sed 's/^ *OK *//') ($(( $(date +%s) - T_ART0 )) s)"
else
  tail -n 5 "$OUT/artefacto.txt" >&2; falla "el artefacto no es reproducible o no pasa el manifiesto"
fi

# ── veredicto ───────────────────────────────────────────────────
msg ""
if [ $rojo -eq 0 ]; then
  msg "== CANON $NIVEL: VERDE =="
  if [ "$NIVEL" = "--completo" ]; then
    mkdir -p "$(dirname "$SELLO_FILE")"
    printf '%s %s\n' "$(git rev-parse --short HEAD)" "$(date -Iseconds)" > "$SELLO_FILE"
    msg "   anotado en $SELLO_FILE — para que nadie tenga que acordarse."
  fi
else
  msg "== CANON $NIVEL: ROJO · ${#fallos[@]} fallo(s) =="
  for f in "${fallos[@]}"; do msg "   · $f"; done
fi
msg "   salida integra en $OUT/"
exit $rojo
