#!/usr/bin/env bash
# tools/banco_ancla.sh -- el banco de la E4 del RFC-0012: el sobre del ANCLA, en vivo.
#
# Nodo REAL firmando latidos (uno por segundo) -> el mando DERIVA el ancla de una cabeza firmada
# (modo 1: el productor de B10.6) -> el ancla derivada se compara EXACTA contra su cabeza (modo 2)
# -> una cabeza posterior EXTIENDE el ancla de una anterior con el camino de
# zkssl_consistencyProof (modo 3, B10.7) -> y la VISTA DIVIDIDA se siembra reproduciendo el
# ATAQUE de verdad (modo 4): la MISMA semilla con un contador de indice FRESCO y un libro en el
# que un envio de ceros evaluado-y-rechazado movio el recepCount firmado -- dos cabezas con el
# mismo indice embebido y digests distintos, que solo quien tiene la clave puede producir.
# Y un negativo por cada regla PRODUCIBLE del sobre, derivado por UNA mutacion de lo capturado;
# los de claves DISTINTAS, con la cabeza de un SEGUNDO nodo de otra semilla que firma y se para.
#
# FUERA del canon: levanta procesos y espera latidos. NO ESCRIBE EN EL ARBOL: todo vive en un
# temporal bajo $HOME, que borra al salir, y lo comprueba al final por `git status --porcelain`.
#
#   bash tools/banco_ancla.sh [--guardar <dir>]
#
# --guardar  copia los sobres (positivos y negativos) a <dir>: de ahi sale el catalogo, COPIADO.
set -u
msg(){ echo "BANCO-ANCLA| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
GUARDAR=""
while [ $# -gt 0 ]; do
  case "$1" in
    --guardar) GUARDAR="${2:?--guardar exige un directorio}"; mkdir -p "$GUARDAR"; shift ;;
    *) fallo "uso: bash tools/banco_ancla.sh [--guardar <dir>]" ;;
  esac
  shift
done
[ -z "$(git status --porcelain)" ] || fallo "el arbol no esta limpio: el banco comprueba al final que no lo toca"
DIR=$(mktemp -d "$HOME/.banco_ancla.XXXXXX")
trap 'rm -rf "$DIR"' EXIT INT TERM HUP QUIT
msg "compilando nodo y verificador en RELEASE (aqui se firma de verdad)"
cargo build --release -q -p zk-ssl-node -p zk-ssl-verify 2>/dev/null \
  || cargo build --release -p zk-ssl-node -p zk-ssl-verify || fallo "no compila"

python3 - "$DIR" "${GUARDAR:-}" <<'PY'
import json, os, shutil, subprocess, sys, time, urllib.request
DIR, GUARDAR = sys.argv[1], sys.argv[2]
NODO, MANDO = 'target/release/zk-ssl-node', 'target/release/zk-ssl-verify'
def msg(m): print('BANCO-ANCLA| ' + m, file=sys.stderr, flush=True)
def levantar(nombre, semilla, port):
    d = '%s/%s' % (DIR, nombre); os.makedirs(d)
    open(d + '/semilla.hex', 'w').write(semilla * 96); os.chmod(d + '/semilla.hex', 0o600)
    return subprocess.Popen([NODO, '--listen', '127.0.0.1:%d' % port, '--latido', '1',
                             '--clave-fichero', d + '/semilla.hex', '--custodia', 'fichero',
                             '--diario', d + '/diario.jsonl', '--ledger', d + '/ledger',
                             '--contador-recepcion', d + '/recepcion.bin',
                             '--indice-firma', d + '/indice.bin', '--log', 'warn'],
                            stdout=subprocess.DEVNULL, stderr=open(d + '/nodo.err', 'w'))
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
P, P_AJENO = 8798, 8799
Z = '0x' + '00' * 32
# El envio de CEROS del banco de la completitud: la capa lo RECHAZA y el rechazo CONSUME un
# numero de recepcion -- el recepCount firmado se mueve sin aplicar nada. Ese es el dato que
# separa las dos corridas de la vista dividida.
cero_pi = {'rootOld': Z, 'rootNew': Z, 'frozenRoot': Z, 'pendingRootOld': Z, 'pendingRootNew': Z,
           'amount': '0x5', 'regulatoryLimit': '0x3e8', 'supplyOld': '0x0', 'supplyNew': '0x0'}
envio = {'receipt': {'proof': '0x' + '00' * 32, 'publicInputs': cero_pi, 'commitment': Z,
                     'notice': {'position': '0x0', 'salt': Z, 'amount': '0x5'}},
         'sender': '0x0', 'senderState': {'publicId': Z, 'balance': '0x5', 'nonce': '0x0'}, 'amount': '0x5'}
fallos = 0
def mando(nombre, sobre, codigo, texto):
    global fallos
    ruta = '%s/%s.json' % (DIR, nombre)
    json.dump(sobre, open(ruta, 'w'), indent=1, sort_keys=True)
    p = subprocess.run([MANDO, ruta], capture_output=True, text=True)
    out = p.stdout + p.stderr
    ok = p.returncode == codigo and texto in out
    fallos += 0 if ok else 1
    msg('%s %-34s exit %d (se esperaba %d: %s)' % ('OK  ' if ok else 'ROJO', nombre, p.returncode, codigo, texto))
    if not ok:
        msg('     ' + out.strip().replace('\n', '\n     '))
    if GUARDAR:
        shutil.copy(ruta, GUARDAR)
    return out
def sin(d, k):
    return {x: y for x, y in d.items() if x != k}
def derivar(cabeza):
    # El PRODUCTOR es el mando (RFC-0012, D-A): se le da la cabeza sola y se
    # lee el ancla que imprime, sin recomputar nada aqui -- un solo productor.
    ruta = '%s/derivar.json' % DIR
    json.dump({'v': 1, 'tipo': 'ancla', 'cabeza': cabeza}, open(ruta, 'w'))
    p = subprocess.run([MANDO, ruta], capture_output=True, text=True)
    assert p.returncode == 0, p.stdout + p.stderr
    for linea in p.stdout.splitlines():
        if linea.strip().startswith('{'):
            return json.loads(linea)
    raise AssertionError('el mando no imprimio el ancla derivada:\n' + p.stdout)
def nibble(hexs):
    ult = hexs[-1]
    return hexs[:-1] + ('0' if ult != '0' else '1')
# La cabeza de OTRA clave: un segundo nodo, de otra semilla, que firma una vez y se para.
ajeno = levantar('ajeno', '38', P_AJENO)
try:
    cabeza_ajena = primera_cabeza(P_AJENO)
finally:
    ajeno.kill(); ajeno.wait()
ancla_ajena = derivar(cabeza_ajena)
msg('la cabeza de OTRA clave: indice %s, del segundo nodo, ya parado' % cabeza_ajena['index'])
# CORRIDA A: la semilla '37'. El envio de ceros entra PRONTO, para que sus cabezas
# lleven recepCount 1 desde el principio y la corrida B (sin envio) diverja ya en
# los primeros indices.
cabezas_a = {}
nodo = levantar('nodo-a', '37', P)
try:
    primera_cabeza(P)
    r = rpc(P, 'zkssl_applySend', envio)
    assert 'error' in r and 'recepcion' in r['error'].get('data', {}), json.dumps(r)[:300]
    msg('corrida A: el envio de ceros RECHAZADO consume el recibo 1: recepCount firmado = 1')
    vieja = None
    for _ in range(240):
        c = rpc(P, 'zkssl_signedEpochHead', {}).get('result', {})
        if c.get('available'):
            cabezas_a[int(c['index'], 16)] = c
            if vieja is None and int(c['mmrSize'], 16) >= 2 and int(c['recepCount'], 16) >= 1:
                vieja = c
                break
        time.sleep(0.25)
    assert vieja is not None, 'no llego una cabeza con mmrSize >= 2'
    oldsize = int(vieja['mmrSize'], 16)
    msg('cabeza VIEJA custodiada (mmrSize %d)' % oldsize)
    time.sleep(4)  # que la historia crezca de verdad
    ack = None
    for _ in range(40):
        v = rpc(P, 'zkssl_consistencyProof', {'oldSize': '0x%x' % oldsize}).get('result', {})
        if v.get('available'):
            ack = v; break
        time.sleep(0.5)
    assert ack is not None, 'consistencyProof no llego a available:true'
    objetivo = int(ack['mmrSize'], 16)
    # La pareja FIRMADA es el acumulador ANTES de la cabeza: el camino de tamano t
    # lo firma LA SIGUIENTE cabeza en emitirse (el molde de banco_extension.sh).
    nueva = None
    for _ in range(40):
        c = rpc(P, 'zkssl_signedEpochHead', {}).get('result', {})
        if c.get('available'):
            cabezas_a[int(c['index'], 16)] = c
            if int(c['mmrSize'], 16) == objetivo:
                nueva = c; break
        time.sleep(0.25)
    assert nueva is not None, 'ninguna cabeza firmo mmrSize %d' % objetivo
    msg('camino y cabeza NUEVA emparejados: la que firma t=%d llego' % objetivo)
    # unos indices mas, para tener donde emparejar la vista dividida
    for _ in range(16):
        c = rpc(P, 'zkssl_signedEpochHead', {}).get('result', {})
        if c.get('available'):
            cabezas_a[int(c['index'], 16)] = c
        time.sleep(0.25)
finally:
    nodo.kill(); nodo.wait()
# CORRIDA B: la MISMA semilla '37' con directorios FRESCOS -- el contador de indice
# vuelve a empezar y el libro no vio el envio: el ataque del contador restaurado.
cabezas_b = {}
nodo = levantar('nodo-b', '37', P)
try:
    primera_cabeza(P)
    tope = max(cabezas_a)
    for _ in range(240):
        c = rpc(P, 'zkssl_signedEpochHead', {}).get('result', {})
        if c.get('available'):
            i = int(c['index'], 16)
            cabezas_b[i] = c
            if i >= tope:
                break
        time.sleep(0.25)
finally:
    nodo.kill(); nodo.wait()
par = None
for i in sorted(set(cabezas_a) & set(cabezas_b)):
    if cabezas_a[i]['epochDigest'] != cabezas_b[i]['epochDigest']:
        par = (cabezas_a[i], cabezas_b[i])
        break
assert par is not None, 'ninguna pareja de cabezas con el mismo indice y digests distintos'
distinta = None
for i in sorted(cabezas_a):
    if cabezas_a[i]['epochDigest'] != par[0]['epochDigest'] and i != int(par[0]['index'], 16):
        distinta = cabezas_a[i]
        break
assert distinta is not None
msg('la vista dividida, sembrada: dos cabezas de indice %s con digests distintos' % par[0]['index'])
msg('los nodos estan MUERTOS; el mando juzga SIN ellos')
ancla_n = derivar(nueva)
ancla_v = derivar(vieja)
base = {'v': 1, 'tipo': 'ancla', 'cabeza': nueva}
exacta = dict(base, ancla=ancla_n)
extendida = dict(base, ancla=ancla_v, camino=ack['camino'])
vista = {'v': 1, 'tipo': 'ancla', 'cabeza': par[0], 'contraria': par[1]}
# los cuatro positivos, uno por modo
mando('ancla-derivada', base, 0, 'VERDE: el ancla se deriva de la cabeza firmada')
mando('ancla-exacta', exacta, 0, 'VERDE: el ancla ES esta cabeza firmada')
mando('ancla-extendida', extendida, 0, 'VERDE: la cabeza extiende el ancla')
mando('vista-dividida', vista, 0, 'VISTA DIVIDIDA')
# la forma
mando('neg-sin-cabeza', sin(base, 'cabeza'), 1, 'falta cabeza')
mando('neg-cabeza-v2', dict(base, cabeza=dict(nueva, formatVersion='0x2')), 1, 'el ancla lee cabezas')
mando('neg-contraria-con-ancla', dict(exacta, contraria=par[1]), 1, 'un sobre con contraria no lleva ancla')
mando('neg-camino-sin-ancla', dict(base, camino=ack['camino']), 1, 'camino sin ancla')
# el ancla contra su cabeza
mando('neg-ancla-v-2', dict(base, ancla=dict(ancla_n, v=2)), 1, 'lee ancla v1')
mando('neg-ancla-de-otra-clave', dict(base, ancla=dict(ancla_n, clave=ancla_ajena['clave'])),
      1, 'el ancla es de OTRA clave')
mando('neg-ancla-indice-mentido', dict(base, ancla=dict(ancla_n, indice=ancla_v['indice'])),
      1, 'su indice no casa')
mando('neg-ancla-digest-mentido', dict(base, ancla=dict(ancla_n, epochDigest=ancla_v['epochDigest'])),
      1, 'su epochDigest no casa')
mando('neg-ancla-raiz-mentida', dict(base, ancla=dict(ancla_n, mmrRoot=nibble(ancla_n['mmrRoot']))),
      1, 'su mmrRoot no casa')
mando('neg-ancla-tamano-mentido', dict(base, ancla=dict(ancla_n, mmrSize=ancla_v['mmrSize'])),
      1, 'su mmrSize no casa')
# el ancla anterior y su camino
mando('neg-ancla-genesis-con-camino', dict(extendida, ancla=dict(ancla_v, mmrSize='0x0')),
      1, 'el ancla del genesis')
mando('neg-ancla-indice-posterior', dict(extendida, ancla=dict(ancla_v, indice='0xffffff')),
      1, 'no es ANTERIOR')
malo = list(ack['camino']); malo[0] = nibble(malo[0])
mando('neg-no-extiende', dict(extendida, camino=malo), 1, 'NO extiende el ancla')
# la vista dividida
mando('neg-contraria-de-otra-clave', dict(vista, contraria=cabeza_ajena), 1, 'claves DISTINTAS')
mando('neg-contraria-indice-distinto', dict(vista, contraria=distinta),
      1, 'los indices embebidos son DISTINTOS')
mando('neg-contraria-identica', dict(vista, contraria=par[0]), 1, 'no hay vista que dividir')
mando('neg-contraria-v2', dict(vista, contraria=dict(par[1], formatVersion='0x2')),
      1, 'contraria: formatVersion')
sys.exit(1 if fallos else 0)
PY
RC=$?
[ "$RC" = 0 ] || fallo "algun sobre no dijo lo que debia"
[ -z "$(git status --porcelain)" ] || fallo "el banco dejo el arbol sucio: no debe tocarlo"
[ -n "$GUARDAR" ] && msg "sobres guardados en $GUARDAR (de ahi sale el catalogo, COPIADO)"
msg "BANCO-ANCLA VERDE"
