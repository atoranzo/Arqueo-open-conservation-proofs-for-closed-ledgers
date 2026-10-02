#!/usr/bin/env bash
# tools/banco_mentiroso_vista.sh -- la E1 del RFC-0011: la VISTA DIVIDIDA contra el TESTIGO.
#
# Hasta aqui `--comparar` y `--auditar` solo habian visto vistas divididas FABRICADAS en tests
# (entrada 93 del BACKLOG). Este banco se la sirve un nodo REAL, con el binario SIN TOCAR: la
# decision del §598 es que las mentiras con clave no piden otro codigo, piden la CLAVE. El metodo
# es el de banco_ancla.sh -la MISMA semilla con un contador de indice FRESCO, y un libro en el que
# un envio de ceros evaluado-y-rechazado movio el recepCount firmado-: dos cabezas con el mismo
# indice y digests distintos, que solo quien tiene la clave puede producir. Dos escenas:
#
#   ESCENA 1  DOS TESTIGOS, DOS VISTAS. Dos nodos de la misma semilla a la vez, un testigo en cada
#             uno. Cada testigo, SOLO, no ve nada (exit 0) y su diario pasa `--auditar` LIMPIO:
#             el silencio se ASIERTA, porque es lo que el testigo aislado puede ver y lo que no.
#             `--comparar` de los dos diarios: ROJO, nombrando CADA indice dividido, misma clave. Desde
#             el §689, `--comparar --sobres` arma un sobre del ancla, modo 4, por indice, y el mando
#             los juzga sin los nodos ni los testigos.
#   ESCENA 2  EL CONTADOR RESTAURADO, bajo un testigo VIVO. El nodo cae y vuelve con la misma
#             semilla y el contador fresco en el MISMO puerto: el testigo SE DETIENE con
#             `vista-dividida` y el indice, y `--auditar` de su diario lo ve SIN el nodo. Desde el
#             §688, `--auditar --sobres` arma de su diario el sobre del ancla, modo 4, y el mando lo
#             juzga sin el nodo ni el testigo: VERDE, VISTA DIVIDIDA.
#
# Exito (D-D del RFC-0011): que la defensa DISPARE nombrando el indice. Los indices esperados
# salen de los diarios crudos (el mismo indice, dos digests), no del texto de la herramienta.
# Claves de prueba siempre (D-E): semillas deterministas de la suite, nunca un keystore.
#
# FUERA del canon: levanta procesos y espera latidos. NO ESCRIBE EN EL ARBOL: todo vive en un
# temporal bajo $HOME, que borra al salir, y lo comprueba al final por `git status --porcelain`.
#
#   bash tools/banco_mentiroso_vista.sh
set -u
msg(){ echo "BANCO-MENT-VISTA| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
[ $# -eq 0 ] || fallo "uso: bash tools/banco_mentiroso_vista.sh"
[ -z "$(git status --porcelain)" ] || fallo "el arbol no esta limpio: el banco comprueba al final que no lo toca"
# Bajo $HOME, no /tmp: el guardian del indice (K.1, §234) se niega sobre un fsync que no
# persiste, y en WSL /tmp es tmpfs (medido en §290).
DIR=$(mktemp -d "$HOME/.banco_mentiroso_vista.XXXXXX")
trap 'rm -rf "$DIR"' EXIT INT TERM HUP QUIT
msg "compilando nodo, cli y mando en RELEASE (aqui se firma de verdad)"
cargo build --release -q -p zk-ssl-node -p zk-ssl-cli -p zk-ssl-verify 2>/dev/null \
  || cargo build --release -p zk-ssl-node -p zk-ssl-cli -p zk-ssl-verify || fallo "no compila"

python3 - "$DIR" <<'PY'
import glob, json, os, re, subprocess, sys, time, urllib.request
DIR = sys.argv[1]
NODO, CLI, MANDO = 'target/release/zk-ssl-node', 'target/release/zk-ssl-cli', 'target/release/zk-ssl-verify'
def msg(m): print('BANCO-MENT-VISTA| ' + m, file=sys.stderr, flush=True)
fallos = 0
def exigir(ok, texto, detalle=''):
    global fallos
    fallos += 0 if ok else 1
    msg('%s %s' % ('OK  ' if ok else 'ROJO', texto))
    if not ok and detalle:
        msg('     ' + detalle.strip().replace('\n', '\n     '))
procesos = []
def levantar(nombre, semilla, port):
    d = '%s/%s' % (DIR, nombre); os.makedirs(d)
    open(d + '/semilla.hex', 'w').write(semilla * 96); os.chmod(d + '/semilla.hex', 0o600)
    p = subprocess.Popen([NODO, '--listen', '127.0.0.1:%d' % port, '--latido', '1',
                          '--clave-fichero', d + '/semilla.hex', '--custodia', 'fichero',
                          '--diario', d + '/diario.jsonl', '--ledger', d + '/ledger',
                          '--contador-recepcion', d + '/recepcion.bin',
                          '--indice-firma', d + '/indice.bin', '--log', 'warn'],
                         stdout=subprocess.DEVNULL, stderr=open(d + '/nodo.err', 'w'))
    procesos.append(p)
    return p
def testigo(nombre, port, veces):
    d = '%s/%s' % (DIR, nombre); os.makedirs(d)
    p = subprocess.Popen([CLI, 'witness', '--nodo', 'http://127.0.0.1:%d' % port, '--cada', '1',
                          '--veces', str(veces), '--diario', d + '/diario.jsonl', '--no-color'],
                         stdout=open(d + '/salida.txt', 'w'), stderr=subprocess.STDOUT)
    procesos.append(p)
    return p, d
def matar(p):
    if p.poll() is None:
        p.kill()
    p.wait()
def esperar(p, seg):
    try:
        return p.wait(timeout=seg)
    except subprocess.TimeoutExpired:
        matar(p)
        return None
def rpc(port, m, p):
    c = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': m, 'params': p}).encode()
    r = urllib.request.Request('http://127.0.0.1:%d' % port, c, {'Content-Type': 'application/json'})
    return json.loads(urllib.request.urlopen(r, timeout=10).read())
def primera_cabeza(port):
    for _ in range(120):
        try:
            r = rpc(port, 'zkssl_signedEpochHead', {}).get('result', {})
            if r.get('available'):
                return r
        except Exception:
            pass
        time.sleep(0.25)
    msg('ROJO: el nodo del puerto %d no firmo ninguna cabeza' % port); sys.exit(1)
Z = '0x' + '00' * 32
# El envio de CEROS de banco_ancla.sh: la capa lo RECHAZA y el rechazo CONSUME un numero de
# recepcion -- el recepCount firmado se mueve sin aplicar nada. Es lo que separa las dos vistas.
cero_pi = {'rootOld': Z, 'rootNew': Z, 'frozenRoot': Z, 'pendingRootOld': Z, 'pendingRootNew': Z,
           'amount': '0x5', 'regulatoryLimit': '0x3e8', 'supplyOld': '0x0', 'supplyNew': '0x0'}
envio = {'receipt': {'proof': '0x' + '00' * 32, 'publicInputs': cero_pi, 'commitment': Z,
                     'notice': {'position': '0x0', 'salt': Z, 'amount': '0x5'}},
         'sender': '0x0', 'senderState': {'publicId': Z, 'balance': '0x5', 'nonce': '0x0'}, 'amount': '0x5'}
def mover_recepcion(port, rotulo):
    r = rpc(port, 'zkssl_applySend', envio)
    assert 'error' in r and 'recepcion' in r['error'].get('data', {}), json.dumps(r)[:300]
    msg('%s: el envio de ceros RECHAZADO consume el recibo 1: recepCount firmado = 1' % rotulo)
def lineas(d):
    return [json.loads(l) for l in open(d + '/diario.jsonl') if l.strip()]
def digests(ls):
    # indice -> digests en orden de aparicion, solo lineas con cabeza firmada
    m = {}
    for v in ls:
        if v.get('signature') and v.get('index'):
            ds = m.setdefault(int(v['index'], 16), [])
            if v['epochDigest'] not in ds:
                ds.append(v['epochDigest'])
    return m
def cli(*args):
    p = subprocess.run([CLI, 'witness'] + list(args) + ['--no-color'], capture_output=True, text=True)
    return p.returncode, p.stdout + p.stderr
def clases(ls):
    # las rachas comprimidas: 'nueva x9 · sin-respuesta x2 · repetida x2 · vista-dividida'
    rachas = []
    for v in ls:
        c = v.get('clase', '?')
        if rachas and rachas[-1][0] == c:
            rachas[-1][1] += 1
        else:
            rachas.append([c, 1])
    return ' · '.join(c if n == 1 else '%s x%d' % (c, n) for c, n in rachas)
try:
    # ── ESCENA 1 · dos nodos de la misma semilla, a la vez, y un testigo en cada uno ──
    PA, PB, VECES = 8811, 8812, 20
    msg('ESCENA 1: dos nodos de la semilla 37 A LA VEZ, cada uno con su contador fresco')
    na, nb = levantar('nodo-a', '37', PA), levantar('nodo-b', '37', PB)
    primera_cabeza(PA); primera_cabeza(PB)
    mover_recepcion(PA, 'nodo A')
    (w1, d1), (w2, d2) = testigo('testigo-1', PA, VECES), testigo('testigo-2', PB, VECES)
    r1, r2 = esperar(w1, 120), esperar(w2, 120)
    matar(na); matar(nb)
    msg('los nodos estan MUERTOS; los diarios se juzgan SIN ellos')
    l1, l2 = lineas(d1), lineas(d2)
    exigir(r1 == 0 and r2 == 0,
           'cada testigo, SOLO, no ve nada: exit %s y %s en %d vueltas' % (r1, r2, VECES),
           open(d1 + '/salida.txt').read()[-600:] + '\n' + open(d2 + '/salida.txt').read()[-600:])
    for n, d in (('testigo-1', d1), ('testigo-2', d2)):
        rc, out = cli('--auditar', d + '/diario.jsonl')
        exigir(rc == 0 and 'sin hallazgos' in out,
               '--auditar %s: LIMPIO -- un diario, solo, no puede contener la otra vista' % n, out)
    g1, g2 = digests(l1), digests(l2)
    divididos = sorted(i for i in set(g1) & set(g2) if g1[i][0] != g2[i][0])
    exigir(len(divididos) > 0, 'los diarios crudos traen %d indice(s) con dos digests: %s'
           % (len(divididos), divididos),
           'testigo-1: %s\ntestigo-2: %s' % (sorted(g1), sorted(g2)))
    rc, out = cli('--comparar', d1 + '/diario.jsonl', d2 + '/diario.jsonl')
    nombrados = sorted(int(x) for x in re.findall(r'VISTA DIVIDIDA en el indice (\d+):', out))
    exigir(rc != 0 and nombrados == divididos and divididos
           and out.count('(misma clave: true)') == len(divididos)
           and '%d divergencia(s)' % len(divididos) in out,
           '--comparar: ROJO, nombra los %d indices divididos, todos de la MISMA clave'
           % len(divididos), out[-1500:])
    # §689: cada indice dividido, como el sobre del ancla, modo 4, armado de los dos diarios.
    rc, out = cli('--comparar', d1 + '/diario.jsonl', d2 + '/diario.jsonl', '--sobres', DIR + '/sobres-1')
    sobres = sorted(glob.glob(DIR + '/sobres-1/vista-dividida-indice-*.json'))
    exigir(len(sobres) == len(divididos),
           '--comparar --sobres arma %d sobre(s), uno por indice dividido' % len(divididos), out[-1000:])
    veredictos = [subprocess.run([MANDO, x], capture_output=True, text=True) for x in sobres]
    exigir(sobres and all(q.returncode == 0 and 'VERDE: VISTA DIVIDIDA' in q.stdout for q in veredictos),
           'el mando los juzga sin los nodos ni los testigos: VERDE, VISTA DIVIDIDA, los %d' % len(sobres),
           '\n'.join((q.stdout + q.stderr)[-300:] for q in veredictos))

    # ── ESCENA 2 · el contador restaurado, bajo un testigo VIVO ──
    PC = 8813
    msg('ESCENA 2: un testigo VIVO; el nodo cae y vuelve con la semilla 37 y el contador FRESCO')
    nc = levantar('nodo-c', '37', PC)
    primera_cabeza(PC)
    w3, d3 = testigo('testigo-3', PC, 0)
    mover_recepcion(PC, 'nodo C')
    time.sleep(10)  # que el testigo vea indices con recepCount 1
    exigir(w3.poll() is None, 'el testigo sigue vivo antes del reinicio: nada que ver todavia',
           open(d3 + '/salida.txt').read()[-600:])
    matar(nc)
    msg('nodo C MUERTO; nodo D: la MISMA semilla, directorios frescos, el MISMO puerto')
    nd = levantar('nodo-d', '37', PC)
    r3 = esperar(w3, 90)
    matar(nd)
    sal3, l3 = open(d3 + '/salida.txt').read(), lineas(d3)
    m = re.search(r'EL TESTIGO SE DETIENE: VistaDividida \{ indice: (\d+),', sal3)
    detenido = int(m.group(1)) if m else None
    exigir(r3 not in (0, None) and detenido is not None,
           'el testigo SE DETIENE con vista-dividida en el indice %s (exit %s)' % (detenido, r3),
           sal3[-800:] + '\nclases: ' + clases(l3))
    g3 = digests(l3)
    ultima = l3[-1] if l3 else {}
    exigir(ultima.get('clase') == 'vista-dividida' and detenido is not None
           and int(ultima.get('index', '0x0'), 16) == detenido and len(g3.get(detenido, [])) == 2,
           'su ULTIMA linea es la evidencia: vista-dividida, indice %s, dos digests en el diario crudo'
           % detenido, 'clases: ' + clases(l3))
    msg('     el diario del testigo: ' + clases(l3))
    rc, out = cli('--auditar', d3 + '/diario.jsonl')
    vistos = sorted({int(x) for x in re.findall(r'VistaDividida \{ linea: \d+, indice: (\d+),', out)})
    exigir(rc != 0 and vistos == [detenido],
           '--auditar, SIN el nodo: ROJO, vista-dividida en el indice %s' % detenido, out[-1500:])
    otros = sorted(set(re.findall(r'⚠️ ([a-z-]+) ·', out)) - {'vista-dividida'})
    msg('     y ademas: %s (el contador que vuelve atras)' % (', '.join(otros) or 'nada'))
    # §688: la deteccion del testigo, portable: el sobre del ancla, modo 4, armado de su diario.
    rc, out = cli('--auditar', d3 + '/diario.jsonl', '--sobres', d3 + '/sobres')
    sobres = sorted(glob.glob(d3 + '/sobres/vista-dividida-*.json'))
    exigir(len(sobres) == 1, '--auditar --sobres arma el sobre de la vista dividida desde el diario',
           out[-1000:])
    if sobres:
        q = subprocess.run([MANDO, sobres[0]], capture_output=True, text=True)
        exigir(q.returncode == 0 and 'VERDE: VISTA DIVIDIDA - la clave firmo DOS cabezas' in q.stdout,
               'el mando lo juzga sin el nodo ni el testigo: VERDE, VISTA DIVIDIDA',
               q.stdout + q.stderr)
finally:
    for p in procesos:
        matar(p)
sys.exit(1 if fallos else 0)
PY
RC=$?
[ "$RC" = 0 ] || fallo "alguna defensa no dijo lo que debia"
[ -z "$(git status --porcelain)" ] || fallo "el banco dejo el arbol sucio: no debe tocarlo"
msg "BANCO-MENT-VISTA VERDE"
