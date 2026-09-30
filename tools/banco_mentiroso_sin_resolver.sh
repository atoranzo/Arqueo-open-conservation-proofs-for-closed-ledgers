#!/usr/bin/env bash
# tools/banco_mentiroso_sin_resolver.sh -- la E4 del RFC-0011: el RECIBO QUE NO SE RESUELVE, por un
# nodo de verdad.
#
# El veredicto 3 del sobre de completitud (RFC-0010, D-F) -«NO RESUELTA EN LA VENTANA»- solo se
# habia sembrado contra un nodo HONESTO, callando la resolucion que existia. Este banco hace que el
# operador de verdad no la tenga, con el binario del nodo SIN TOCAR: el §598 dejo por medir si se
# podia sin codigo, y se puede. El operador aplica la operacion y responde con su recibo; ANTES de
# la cabeza siguiente para el nodo y RESTAURA el libro a una copia anterior, conservando el
# contador y el registro de recepcion. El recibo queda firmado por la cabeza que cierra su era; la
# operacion ya no existe, y ninguna cabeza firmara jamas su acuse.
#
#   CONTROL  un titular paga por la via directa contra el mismo nodo honrado: su recibo se resuelve,
#            y el sobre con el acuse de la cabeza que lo firma sale VERDE -el veredicto 1, EN VIVO
#            por primera vez: el RFC-0010 lo acepto sin vector porque pide una prueba STARK real de
#            un envio aplicado, y aqui la pone el ejemplo `e2e` del sdk-.
#   MENTIRA  latido de 30 s -el §121 eligio 60: la ventana entre aplicar y firmar es del diseno-,
#            un segundo titular paga y cobra, y el operador restaura el libro. Tras rearrancar: los
#            dos recibos, bajo la recepRoot de una cabeza firmada; el `seq` firmado se queda donde
#            estaba; `zkssl_ackPath` contesta «vuelve tras el proximo latido» cabeza tras cabeza; el
#            testigo, que miraba desde el principio, no ve nada; y el sobre dice «ventana ABIERTA»:
#            el reloj de la unica defensa que puede verlo esta en marcha.
#   --largo  espera a que la ventana expire -n = 1.440 cabezas, unos 24 minutos a un latido por
#            segundo- y el sobre sale ROJO NOMBRADO, «NO RESUELTA EN LA VENTANA», esta vez con
#            razon: el operador no tiene resolucion que exhibir. El del control sigue VERDE.
#
# El titular guarda lo que recibe con el SDK: desde el §606 `pay` y `claim` devuelven su
# constancia -el recibo y el acuse tal como llegaron- y el ejemplo `e2e` la imprime, una linea JSON
# por operacion. Hasta el §606 el sdk la tiraba y este banco la guardaba con un proxy de registro
# de su lado (la 109). Claves de prueba siempre (D-E): semillas deterministas y el grifo `--dev`.
#
# FUERA del canon: levanta procesos y espera latidos. NO ESCRIBE EN EL ARBOL: todo vive en un
# temporal bajo $HOME, que borra al salir, y lo comprueba al final por `git status --porcelain`.
#
#   bash tools/banco_mentiroso_sin_resolver.sh [--largo] [--guardar <dir>]
#
# --guardar  copia a <dir> el sobre RESUELTO del control como `resuelta-por-acuse.json`: de ahi sale
#            el vector del veredicto 1 de `spec/vectors/completitud/` (§605), COPIADO.
set -u
msg(){ echo "BANCO-MENT-SINRES| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
LARGO=0; GUARDAR=""
while [ $# -gt 0 ]; do
  case "$1" in
    --largo) LARGO=1 ;;
    --guardar) GUARDAR="${2:?--guardar exige un directorio}"; mkdir -p "$GUARDAR"; shift ;;
    *) fallo "uso: bash tools/banco_mentiroso_sin_resolver.sh [--largo] [--guardar <dir>]" ;;
  esac
  shift
done
[ -z "$(git status --porcelain)" ] || fallo "el arbol no esta limpio: el banco comprueba al final que no lo toca"
# Bajo $HOME, no /tmp: el guardian del indice (K.1, §234) se niega sobre un fsync que no
# persiste, y en WSL /tmp es tmpfs (medido en §290).
DIR=$(mktemp -d "$HOME/.banco_mentiroso_sin_resolver.XXXXXX")
trap 'rm -rf "$DIR"' EXIT INT TERM HUP QUIT
msg "compilando nodo, cli, verificador y el ejemplo e2e del sdk en RELEASE (aqui se prueba y se firma de verdad)"
{ cargo build --release -q -p zk-ssl-node -p zk-ssl-cli -p zk-ssl-verify 2>/dev/null \
    || cargo build --release -p zk-ssl-node -p zk-ssl-cli -p zk-ssl-verify; } || fallo "no compila"
{ cargo build --release -q -p zk-ssl-sdk --example e2e 2>/dev/null \
    || cargo build --release -p zk-ssl-sdk --example e2e; } || fallo "no compila el e2e"

python3 - "$DIR" "$LARGO" "${GUARDAR:-}" <<'PY'
import json, os, re, shutil, subprocess, sys, threading, time, urllib.request
DIR, LARGO, GUARDAR = sys.argv[1], sys.argv[2] == '1', sys.argv[3]
NODO, CLI, MANDO = 'target/release/zk-ssl-node', 'target/release/zk-ssl-cli', 'target/release/zk-ssl-verify'
E2E = 'target/release/examples/e2e'
PN = 8818
def msg(m): print('BANCO-MENT-SINRES| ' + m, file=sys.stderr, flush=True)
fallos = 0
def exigir(ok, texto, detalle=''):
    global fallos
    fallos += 0 if ok else 1
    msg('%s %s' % ('OK  ' if ok else 'ROJO', texto))
    if not ok and detalle:
        msg('     ' + str(detalle).strip().replace('\n', '\n     '))
    return ok

def rpc(m, p):
    c = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': m, 'params': p}).encode()
    r = urllib.request.Request('http://127.0.0.1:%d' % PN, c, {'Content-Type': 'application/json'})
    return json.loads(urllib.request.urlopen(r, timeout=10).read())
def rpc_vivo(m, p, seg=60):
    # tras un arranque el nodo tarda en escuchar: se reintenta mientras la conexion se rechace
    fin = time.time() + seg
    while True:
        try:
            return rpc(m, p)
        except Exception:
            if time.time() > fin:
                raise
            time.sleep(0.2)
# ── las cabezas firmadas, todas: el titular custodia las que le hagan falta ──
cabezas, vivo = {}, [True]
def recoger():
    while vivo[0]:
        try:
            c = rpc('zkssl_signedEpochHead', {}).get('result', {})
            if c.get('available'):
                cabezas.setdefault(int(c['index'], 16), c)
        except Exception:
            pass
        time.sleep(0.15)
threading.Thread(target=recoger, daemon=True).start()
def ultima():
    return cabezas[max(cabezas)] if cabezas else None
def esperar_indice(minimo, seg=120):
    fin = time.time() + seg
    while time.time() < fin:
        if cabezas and max(cabezas) >= minimo:
            return ultima()
        time.sleep(0.1)
    return None

procesos = []
def matar(p):
    if p.poll() is None:
        p.kill()
    p.wait()
N = DIR + '/nodo'; os.makedirs(N)
open(N + '/semilla.hex', 'w').write('44' * 96); os.chmod(N + '/semilla.hex', 0o600)
def nodo(latido, err):
    p = subprocess.Popen([NODO, '--listen', '127.0.0.1:%d' % PN, '--latido', str(latido), '--dev',
                          '--clave-fichero', N + '/semilla.hex', '--custodia', 'fichero',
                          '--diario', N + '/diario.jsonl', '--ledger', N + '/ledger',
                          '--contador-recepcion', N + '/recepcion.bin',
                          '--indice-firma', N + '/indice.bin', '--log', 'warn'],
                         stdout=subprocess.DEVNULL, stderr=open(N + '/' + err, 'w'))
    procesos.append(p)
    return p
def titular(nombre):
    # el ejemplo e2e del sdk: abre dos cuentas, fondea con el grifo dev, PAGA y COBRA con pruebas
    # STARK reales hechas en el cliente -la clave de gasto no viaja-, e imprime la constancia que el
    # sdk le devuelve de cada operacion (§606): lo que el titular custodia, sin nadie en medio
    t = time.time()
    p = subprocess.run([E2E], env=dict(os.environ, ZKSSL_URL='http://127.0.0.1:%d' % PN),
                       capture_output=True, text=True, timeout=600)
    c = {}
    for linea in p.stdout.splitlines():
        m = re.match(r'E2E: constancia-(pago|cobro) (\{.*\})$', linea)
        if m:
            c[m.group(1)] = json.loads(m.group(2))
    ok = exigir(p.returncode == 0 and sorted(c) == ['cobro', 'pago']
                and all(c[k].get('recepcion') and c[k].get('acuse') for k in c),
                '%s paga y cobra con pruebas reales en %.1f s, y el sdk le deja sus dos constancias'
                % (nombre, time.time() - t), p.stdout[-600:] + p.stderr[-600:])
    if not ok:
        raise SystemExit(1)
    return c['pago'], c['cobro']
def camino_de_recibo(rx, seg=60):
    fin = time.time() + seg
    while time.time() < fin:
        v = rpc_vivo('zkssl_recepPath', {'rx': rx}).get('result', {})
        if v.get('available'):
            return v
        time.sleep(0.2)
    return None
def sobre_base(rec):
    v = camino_de_recibo(rec['rx'])
    if v is None:
        return None, None
    idx = int(v['index'], 16)
    esperar_indice(idx)
    ant = [i for i in cabezas if i < idx]
    q = int(cabezas[max(ant)]['recepCount'], 16) if ant else 0
    return {'v': 1, 'tipo': 'completitud', 'cierre': cabezas[idx], 'recepcion': rec,
            'limiteAnterior': '0x%x' % q, 'camino': v['camino']}, idx
def mando(nombre, sobre):
    ruta = '%s/%s.json' % (DIR, nombre)
    json.dump(sobre, open(ruta, 'w'), indent=1, sort_keys=True)
    p = subprocess.run([MANDO, ruta], capture_output=True, text=True)
    return p.returncode, p.stdout + p.stderr
def acuse_firmado(aplicado, seg=60):
    # ⚠️ el camino y la hoja van por el `logSeq` de la ENTRADA; la `epoca` del acuse es logSeq + 1,
    #    la primera cabeza que puede contenerla (`epoca_de_acuse`)
    fin = time.time() + seg
    while time.time() < fin:
        v = rpc_vivo('zkssl_ackPath', {'seq': aplicado['logSeq']}).get('result', {})
        if v.get('available'):
            return v
        time.sleep(0.2)
    return None
try:
    # ── CONTROL · un nodo honrado resuelve: el veredicto 1, en vivo ──
    n = nodo(2, 'nodo-1.err')
    exigir(esperar_indice(1, 60) is not None, 'el nodo firma (latido de 2 s, grifo --dev)')
    w = subprocess.Popen([CLI, 'witness', '--nodo', 'http://127.0.0.1:%d' % PN, '--cada', '1', '--veces', '0',
                          '--diario', DIR + '/testigo.jsonl', '--no-color'],
                         stdout=open(DIR + '/testigo.txt', 'w'), stderr=subprocess.STDOUT)
    procesos.append(w)
    msg('un testigo mira el nodo desde ahora hasta el final')
    pago1, _ = titular('CONTROL: el titular 1')
    rec1, ac1 = pago1['recepcion'], pago1['acuse']
    base1, cierre1 = sobre_base(rec1)
    ack = acuse_firmado(pago1)
    exigir(base1 is not None and ack is not None,
           'CONTROL: el recibo (rx %d) y el acuse (entrada %d) llegan a cabezas firmadas'
           % (int(rec1['rx'], 16), int(pago1['logSeq'], 16)))
    s = int(ack['s'], 16)
    esperar_indice(max(cabezas) + 1)
    c_ack = cabezas[min(i for i in cabezas if int(cabezas[i]['seq'], 16) == s)]
    resuelto = dict(base1, resolucion={'tipo': 'acuse', 'cabeza': c_ack, 'acuse': {
        'hashPrueba': ac1['hashPrueba'], 'seq': pago1['logSeq'], 'camino': ack['camino']}})
    rc, out = mando('control-resuelto', resuelto)
    if exigir(rc == 0 and 'RESUELTA como transicion aplicada' in out,
              'CONTROL: el sobre con su acuse, VERDE -el veredicto 1, en vivo por primera vez-', out[-1200:]) \
            and GUARDAR:
        shutil.copy(DIR + '/control-resuelto.json', GUARDAR + '/resuelta-por-acuse.json')
        msg('     guardado en %s/resuelta-por-acuse.json (de ahi sale el vector, COPIADO)' % GUARDAR)

    # ── la COPIA del operador, con el nodo parado ──
    matar(n)
    shutil.copytree(N + '/ledger', DIR + '/ledger.copia')
    msg('el operador para el nodo y COPIA el libro')
    n = nodo(30, 'nodo-2.err')
    antes = max(cabezas)
    h = esperar_indice(antes + 1, 90)
    exigir(h is not None, 'el nodo vuelve con latido de 30 s y firma la cabeza %s' % (h or {}).get('index'))

    # ── la MENTIRA · aplicar, responder con recibo, y deshacer antes de firmar ──
    k = max(cabezas)
    pago2, cobro2 = titular('MENTIRA: el titular 2')
    rec2, ac2 = pago2['recepcion'], pago2['acuse']
    ult = rpc('zkssl_signedEpochHead', {}).get('result', {})
    exigir(int(ult.get('index', '0x0'), 16) == k,
           'ninguna cabeza entre la respuesta y la parada: la ultima firmada sigue siendo la %d' % k,
           'se colo la cabeza %s: la mentira no llego a tiempo' % ult.get('index'))
    matar(n)
    shutil.rmtree(N + '/ledger')
    shutil.copytree(DIR + '/ledger.copia', N + '/ledger')
    msg('el operador para el nodo y RESTAURA el libro a su copia; contador y registro de recepcion, intactos')
    n = nodo(1, 'nodo-3.err')
    base2, cierre2 = sobre_base(rec2)
    exigir(base2 is not None,
           'MENTIRA: el recibo (rx %d, era %d) queda bajo la recepRoot de la cabeza firmada %s'
           % (int(rec2['rx'], 16), int(rec2['era'], 16), cierre2))
    base2b, _ = sobre_base(cobro2['recepcion'])
    exigir(base2b is not None, 'MENTIRA: y el del cobro (rx %d), tambien' % int(cobro2['recepcion']['rx'], 16))
    seq_firmado = int(cabezas[k]['seq'], 16)
    fin = esperar_indice(max(cabezas) + 4, 60)
    seqs = sorted({int(cabezas[i]['seq'], 16) for i in cabezas if i > k})
    exigir(seqs == [seq_firmado] and int(ac2['epoca'], 16) > seq_firmado,
           'MENTIRA: el seq firmado se queda en %d cabeza tras cabeza; la entrada %d pedia una cabeza con seq >= %d'
           % (seq_firmado, int(pago2['logSeq'], 16), int(ac2['epoca'], 16)), 'seqs tras el rearranque: %s' % seqs)
    v = rpc('zkssl_ackPath', {'seq': pago2['logSeq']}).get('result', {})
    exigir(not v.get('available') and 'ABIERTA' in (v.get('reason') or ''),
           'MENTIRA: zkssl_ackPath de esa entrada: «%s»' % v.get('reason'), json.dumps(v)[:300])
    rc, out = mando('mentira-ventana-abierta', dict(base2, vigente=ultima()))
    exigir(rc == 1 and 'ventana ABIERTA' in out,
           'MENTIRA: el sobre, hoy: «ventana ABIERTA» -el reloj de la unica defensa que lo vera, en marcha-',
           out[-800:])
    if LARGO:
        objetivo = int(rec2['era'], 16) + int(rec2['n'], 16) + 1
        msg('--largo: esperando la cabeza de indice %d (la ventana expira); a un latido por segundo' % objetivo)
        vig = esperar_indice(objetivo, 3600)
        rc, out = mando('mentira-no-resuelta', dict(base2, vigente=vig))
        exigir(rc == 1 and 'NO RESUELTA EN LA VENTANA' in out,
               'MENTIRA: el sobre, expirada la ventana: ROJO NOMBRADO, «NO RESUELTA EN LA VENTANA»', out[-800:])
        v = rpc('zkssl_ackPath', {'seq': pago2['logSeq']}).get('result', {})
        exigir(not v.get('available'), 'MENTIRA: y el operador sigue sin acuse que exhibir', json.dumps(v)[:300])
        rc, out = mando('control-resuelto-despues', resuelto)
        exigir(rc == 0, 'CONTROL: el sobre resuelto sigue VERDE', out[-600:])
    vivo_testigo = w.poll() is None
    matar(w); matar(n)
    vivo[0] = False
    msg('el nodo esta PARADO; las defensas del testigo se juzgan SIN el')
    exigir(vivo_testigo, 'SILENCIO: el testigo no se detuvo en toda la corrida',
           open(DIR + '/testigo.txt').read()[-600:])
    for args, texto, nombre in ((['--auditar', DIR + '/testigo.jsonl'], 'sin hallazgos', '--auditar'),
                                (['--ausentes', DIR + '/testigo.jsonl', N + '/diario.jsonl'], 'sin ausentes', '--ausentes')):
        p = subprocess.run([CLI, 'witness'] + args + ['--no-color'], capture_output=True, text=True)
        exigir(p.returncode == 0 and texto in p.stdout + p.stderr,
               'SILENCIO: %s del testigo, %s' % (nombre, texto), (p.stdout + p.stderr)[-600:])
finally:
    vivo[0] = False
    for p in procesos:
        matar(p)
sys.exit(1 if fallos else 0)
PY
RC=$?
[ "$RC" = 0 ] || fallo "algo no dijo lo que debia"
[ -z "$(git status --porcelain)" ] || fallo "el banco dejo el arbol sucio: no debe tocarlo"
msg "BANCO-MENT-SINRES VERDE"
