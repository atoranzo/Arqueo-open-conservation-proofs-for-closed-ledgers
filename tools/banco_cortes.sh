#!/usr/bin/env bash
# tools/banco_cortes.sh — el banco de los CORTES dentro de `zkssl/0.4` (§699, §804): qué envío acepta
# un nodo cuando el cliente que lo prueba es de otro punto de la historia.
#
# El §699 midió que `zkssl/0.4` no identifica qué pruebas acepta un nodo: el AIR del envío cambió
# sin subir la versión en el §641 (RFC-0017 E2, el selector periódico de la capa) y en el §680
# (RFC-0018 E1, los acumuladores de la posición). Lo midió con un banco que no dejó en el árbol, y
# lo describió para repetirlo. Este es ese banco, sobre los mismos seis commits:
#
#   §622 3d3bb3d · §640 7534380 · §641 2901c54, hijo del 7534380
#   §679 3a1b5d7 · §680 f543ccf, hijo del 3a1b5d7 · §699 b9e7b0c
#
# El sexto es el commit del §699: el asiento midió su árbol antes de redactarse, y en `crates/` el
# commit sólo difiere del §698 en la descripción del OpenRPC y en su test, como dice el asiento.
#
#   COMPILAR  cada commit, sacado con `git archive` a un temporal —nunca un worktree—, en release y
#             con --locked: `zk-ssl-node` y el ejemplo `e2e` de `zk-ssl-sdk`, el mismo blob en los
#             seis (lo comprueba). Un solo CARGO_TARGET_DIR para los seis, uno tras otro: lo que no
#             cambia de un commit a otro —las dependencias de crates.io— no se recompila.
#   CORRER    cada par (nodo, cliente) de la tabla del §699, en los dos sentidos y dos veces: un nodo
#             `--dev` nuevo, con las opciones de `tools/banco_mentiroso_sin_resolver.sh`, y el `e2e`
#             contra él: abre dos cuentas con claves aleatorias, fondea una con `dev_fund`, paga
#             250.000 —la prueba de envío se hace en el cliente—, cobra y comprueba los dos saldos.
#             48 corridas, y en cada una el nodo contesta `zkssl/0.4` a `zkssl_protocolVersion`.
#   COMPARAR  casilla a casilla con lo que midió el §699: dentro de un tramo —del §622 al §640, del
#             §641 al §679 y del §680 al §699— pagan y cobran; entre dos tramos el envío se rechaza,
#             y siempre igual: `zkssl_applySend`, -32000, `VerificationFailed`, «envio:
#             InconsistentOodConstraintEvaluations». Una casilla que no dice lo del §699 es ROJO,
#             con la salida de su corrida.
#
# Medido en el §804, con la máquina compartida: los seis compilados en 1.115 s con un solo target de
# 1,1 GB, y las 48 corridas en 134 s, de 1,6 a 3,9 s cada una; VERDE, casilla a casilla lo que dio
# el §699. Con los binarios del §641 cambiados por los del §640 (--binarios), ROJO en las tres
# filas que el §641 toca: 0 de 4, 0 de 4 y 8 de 12.
#
# ⚠️ Los ficheros se extraen con la hora de ahora (`tar -m`), no con la del commit: cargo da por
#    fresco un crate del workspace si sus fuentes son más viejas que lo que ya compiló. Sin el
#    `-m`, medido en el §804: el §641 «compila» en 0 s, detrás del §640 en el mismo target, y su
#    nodo es byte a byte el del §640; con él, otro binario, y el corte se ve.
# ⚠️ Mide la HISTORIA, no el árbol del sello: los seis commits son fijos, y lo que puede cambiar su
#    resultado es la cadena de herramientas o el registro de crates, no un sello. Por eso vive FUERA
#    de `tools/bancos.sh` (su punto 5), y por lo que cuesta: seis compilaciones en release.
# ⚠️ Lo que el §699 no midió tampoco lo mide este: el cobro y la quema a través de los cortes (el
#    `e2e` para en el envío), las demás pruebas de la capa, el umbral del §684, los commits de en
#    medio de cada tramo uno a uno, y lo de antes del §622.
# ⚠️ Necesita la historia hasta el §622: en un clon superficial sale ROJO, diciendo qué commit falta.
#
# NO ESCRIBE EN EL ÁRBOL: fuentes, target, binarios y nodos viven en un temporal bajo $HOME —el
# guardián del índice se niega donde `fsync` no persiste (K.1, §234)—, que borra al salir. Lo
# comprueba al final por DELTA sobre `git status --porcelain`: mide commits, no el árbol, y corre
# también con el árbol sucio.
#
#   bash tools/banco_cortes.sh [--guardar <dir>] [--binarios <dir>]
#
# --guardar   deja en <dir> los doce binarios que compila: `<asiento>-nodo` y `<asiento>-e2e`.
# --binarios  no compila: corre los de <dir>, guardados antes con --guardar. Para repetir la parte
#             viva sin las seis compilaciones; lo que mide entonces es lo que haya en <dir>.
set -uo pipefail
msg(){ printf 'BANCO-CORTES| %s\n' "$*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
GUARDAR=""; BINARIOS=""
while [ $# -gt 0 ]; do
  case "$1" in
    --guardar) GUARDAR="${2:?--guardar exige un directorio}"; shift ;;
    --binarios) BINARIOS="${2:?--binarios exige un directorio}"; shift ;;
    *) fallo "uso: bash tools/banco_cortes.sh [--guardar <dir>] [--binarios <dir>]" ;;
  esac
  shift
done
[ -n "$GUARDAR" ] && [ -n "$BINARIOS" ] && fallo "--guardar y --binarios no van juntos: con --binarios no se compila nada"

cd "$(dirname "$0")/.." || fallo "no encuentro la raiz"
ANTES=$(git status --porcelain)
DIR=$(mktemp -d "$HOME/.banco_cortes.XXXXXX") || { msg "no pude crear el directorio"; exit 2; }
trap 'rm -rf "$DIR"' EXIT INT TERM HUP QUIT

# Los seis puntos del §699, en orden de historia: asiento y commit entero.
ASIENTOS=(622 640 641 679 680 699)
COMMITS=(3d3bb3dfcc9933dae3bddc6b9e348fab0383de20 7534380a0b381b4ba1bcff312bc6c0f91cc45915
         2901c544a38e7bad6e0def3567704db774946146 3a1b5d7a321ba1e756890b09c1d3f36d59c5b645
         f543ccfac8f7dd1de013748bd996f3ce55c74d83 b9e7b0cae88865dbb5c822f69c7e3761af59185c)
E2E_RS=crates/zk-ssl-sdk/examples/e2e.rs

if [ -n "$BINARIOS" ]; then
  BIN=$(cd "$BINARIOS" 2>/dev/null && pwd) || fallo "--binarios: no existe $BINARIOS"
  for a in "${ASIENTOS[@]}"; do
    [ -x "$BIN/$a-nodo" ] && [ -x "$BIN/$a-e2e" ] || fallo "--binarios: faltan $a-nodo o $a-e2e en $BIN"
  done
  msg "no se compila: corren los binarios de $BIN (--binarios)"
else
  BIN="$DIR/bin"; mkdir -p "$BIN"
  # ── la historia: los seis commits, los dos padres que hacen de cada corte un par, y un solo e2e ──
  for i in "${!ASIENTOS[@]}"; do
    git cat-file -e "${COMMITS[$i]}^{commit}" 2>/dev/null \
      || fallo "falta el commit del §${ASIENTOS[$i]} (${COMMITS[$i]:0:7}): la historia de este clon no llega"
  done
  [ "$(git rev-parse "${COMMITS[2]}^")" = "${COMMITS[1]}" ] || fallo "el §641 no es hijo del §640"
  [ "$(git rev-parse "${COMMITS[4]}^")" = "${COMMITS[3]}" ] || fallo "el §680 no es hijo del §679"
  BLOBS=$(for c in "${COMMITS[@]}"; do git rev-parse "$c:$E2E_RS"; done | sort -u | wc -l)
  [ "$BLOBS" = 1 ] || fallo "el ejemplo e2e no es el mismo blob en los seis commits ($BLOBS distintos)"
  msg "los seis commits estan; el §641 y el §680, hijos de su par; $E2E_RS, un solo blob"

  # ── compilar: un target para los seis, uno tras otro ──
  export CARGO_TARGET_DIR="$DIR/target"
  T_INI=$(date +%s)
  for i in "${!ASIENTOS[@]}"; do
    a=${ASIENTOS[$i]}; c=${COMMITS[$i]}; F="$DIR/fuente-$a"; L="$DIR/compila-$a.log"
    mkdir -p "$F"
    git archive "$c" | tar -x -m -C "$F" || fallo "no pude sacar el §$a (${c:0:7})"
    t0=$(date +%s)
    msg "compilando el §$a (${c:0:7}): zk-ssl-node y el e2e, release, --locked"
    ( cd "$F" && cargo build --locked --release -p zk-ssl-node > "$L" 2>&1 \
        && cargo build --locked --release -p zk-ssl-sdk --example e2e >> "$L" 2>&1 ) \
      || { tail -n 25 "$L" | sed 's/^/     /' >&2; fallo "el §$a (${c:0:7}) no compila"; }
    cp "$CARGO_TARGET_DIR/release/zk-ssl-node" "$BIN/$a-nodo" && cp "$CARGO_TARGET_DIR/release/examples/e2e" "$BIN/$a-e2e" \
      || fallo "el §$a compilo, pero no encuentro sus binarios en el target"
    rm -rf "$F"
    msg "     el §$a, compilado en $(( $(date +%s) - t0 )) s"
  done
  msg "los seis, compilados en $(( $(date +%s) - T_INI )) s, con un solo target de $(du -sh "$CARGO_TARGET_DIR" | cut -f1)"
  rm -rf "$CARGO_TARGET_DIR"
  if [ -n "$GUARDAR" ]; then
    mkdir -p "$GUARDAR" && cp "$BIN"/* "$GUARDAR"/ || fallo "--guardar: no pude copiar a $GUARDAR"
    msg "     guardados en $GUARDAR (--guardar): se repite la parte viva con --binarios"
  fi
fi

python3 - "$DIR" "$BIN" <<'PY'
import json, os, subprocess, sys, time, urllib.request
DIR, BIN = sys.argv[1], sys.argv[2]
PN = 8851
VECES = 2
def msg(m): print('BANCO-CORTES| ' + m, file=sys.stderr, flush=True)
ASIENTOS = [622, 640, 641, 679, 680, 699]
TRAMO = {622: 0, 640: 0, 641: 1, 679: 1, 680: 2, 699: 2}
# La tabla del §699, fila a fila: el rotulo, los pares, y si el envio se acepta.
TABLA = [
    ('cada nodo con su cliente (los seis)', [(a, a) for a in ASIENTOS], True),
    ('§622 y §640', [(622, 640)], True),
    ('§640 y §641', [(640, 641)], False),
    ('§641 y §679', [(641, 679)], True),
    ('§679 y §680', [(679, 680)], False),
    ('§680 y el del §699', [(680, 699)], True),
    ('§679 y el del §699', [(679, 699)], False),
    ('§622 con §641, con §679 y con el del §699', [(622, 641), (622, 679), (622, 699)], False),
]
# la tabla es la regla de los tramos: se acepta si y solo si los dos caen en el mismo
for _, pares, acepta in TABLA:
    assert all((TRAMO[a] == TRAMO[b]) == acepta for a, b in pares), 'la tabla contradice los tramos'
RECHAZO = ('zkssl_applySend', '-32000', 'VerificationFailed', 'envio: InconsistentOodConstraintEvaluations')

def rpc(m, p):
    c = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': m, 'params': p}).encode()
    r = urllib.request.Request('http://127.0.0.1:%d' % PN, c, {'Content-Type': 'application/json'})
    return json.loads(urllib.request.urlopen(r, timeout=10).read())
def libre():
    try:
        rpc('zkssl_protocolVersion', [])
        return False
    except Exception:
        return True
if not libre():
    msg('ROJO: el puerto %d ya contesta: otro nodo vive ahi' % PN)
    sys.exit(1)

def corrida(k, nodo, cliente):
    """Un nodo --dev nuevo del asiento `nodo` y el e2e del asiento `cliente` contra el."""
    N = '%s/corrida-%02d' % (DIR, k); os.makedirs(N)
    open(N + '/semilla.hex', 'w').write('44' * 96); os.chmod(N + '/semilla.hex', 0o600)
    p = subprocess.Popen(['%s/%d-nodo' % (BIN, nodo), '--listen', '127.0.0.1:%d' % PN, '--latido', '2', '--dev',
                          '--clave-fichero', N + '/semilla.hex', '--custodia', 'fichero',
                          '--diario', N + '/diario.jsonl', '--ledger', N + '/ledger',
                          '--contador-recepcion', N + '/recepcion.bin',
                          '--indice-firma', N + '/indice.bin', '--log', 'warn'],
                         stdout=subprocess.DEVNULL, stderr=open(N + '/nodo.err', 'w'))
    try:
        version, fin = None, time.time() + 60
        while version is None and time.time() < fin and p.poll() is None:
            try:
                version = rpc('zkssl_protocolVersion', []).get('result')
            except Exception:
                time.sleep(0.1)
        if version is None:
            return None, 'el nodo no contesta: ' + open(N + '/nodo.err').read()[-400:], 0.0
        t = time.time()
        e = subprocess.run(['%s/%d-e2e' % (BIN, cliente)], env=dict(os.environ, ZKSSL_URL='http://127.0.0.1:%d' % PN),
                           capture_output=True, text=True, timeout=600)
        seg = time.time() - t
        out = e.stdout + e.stderr
        if e.returncode == 0 and 'E2E OK' in e.stdout:
            res = 'paga'
        elif e.returncode != 0 and all(x in out for x in RECHAZO) and 'FASE 1 aplicada' not in e.stdout:
            res = 'rechazo'
        else:
            res = 'otro'
        return version, (res, out), seg
    finally:
        if p.poll() is None:
            p.kill()
        p.wait()
        subprocess.run(['rm', '-rf', N])

fallos, versiones, k, filas = 0, [], 0, []
T = time.time()
for rotulo, pares, acepta in TABLA:
    esperado = 'paga' if acepta else 'rechazo'
    combos = []
    for a, b in pares:
        combos += [(a, b)] if a == b else [(a, b), (b, a)]
    bien = total = 0
    for nodo, cliente in combos:
        for vez in range(1, VECES + 1):
            k += 1; total += 1
            version, r, seg = corrida(k, nodo, cliente)
            versiones.append(version)
            if version is None:
                fallos += 1
                msg('ROJO nodo §%d · cliente §%d (%d/%d): %s' % (nodo, cliente, vez, VECES, r))
                continue
            res, out = r
            ok = res == esperado and version == 'zkssl/0.4'
            bien += ok
            dice = {'paga': 'pagan y cobran', 'rechazo': 'el envio se rechaza (InconsistentOodConstraintEvaluations)',
                    'otro': 'NI paga NI se rechaza como en el §699'}[res]
            msg('%s nodo §%d · cliente §%d (%d/%d): %s, %.1f s; el nodo dice %s'
                % ('OK  ' if ok else 'ROJO', nodo, cliente, vez, VECES, dice, seg, version))
            if not ok:
                fallos += 1
                msg('     ' + out.strip()[-700:].replace('\n', '\n     '))
    filas.append((rotulo, esperado, bien, total))

msg('')
msg('las %d corridas, en %.0f s' % (k, time.time() - T))
msg('| nodo y cliente | el §699, en los dos sentidos y dos veces cada uno | hoy |')
msg('|---|---|---|')
for rotulo, esperado, bien, total in filas:
    dice = 'pagan y cobran' if esperado == 'paga' else 'el envio se rechaza'
    msg('| %s | %s: %d de %d | %s |' % (rotulo, dice, total, total,
                                       'lo mismo' if bien == total else 'ROJO: %d de %d' % (bien, total)))
v04 = sum(v == 'zkssl/0.4' for v in versiones)
msg('los seis nodos contestan zkssl/0.4 a zkssl_protocolVersion: %d de %d veces' % (v04, len(versiones)))
if k != 48 or v04 != k:
    fallos += 1
    msg('ROJO: el §699 dio 48 corridas y las 48 con zkssl/0.4; aqui %d y %d' % (k, v04))
sys.exit(1 if fallos else 0)
PY
RC=$?
[ "$RC" = 0 ] || fallo "alguna casilla no dice lo que midio el §699"
[ "$(git status --porcelain)" = "$ANTES" ] || fallo "el banco toco el arbol: no debe"
msg "BANCO-CORTES VERDE: las 48 corridas dicen lo que midio el §699; los cortes son el §641 y el §680"
