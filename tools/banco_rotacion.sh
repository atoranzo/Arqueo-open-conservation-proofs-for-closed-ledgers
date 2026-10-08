#!/usr/bin/env bash
# tools/banco_rotacion.sh -- el banco de la E5 del RFC-0015: la ROTACION de la clave del operador,
# juzgada por el mando con la cadena de actas, en vivo.
#
# UN nodo REAL rota dos veces sobre el mismo diario: A (genesis, comprometiendo a B) -> B, con la
# firma de A (`--clave-anterior-fichero`) -> C, sin la de B (la declara quemada). De cada tramo, una
# cabeza; de C, los caminos de consistencia que unen las cimas y la cadena de `zkssl_keyActs`. El
# mando juzga, con el nodo parado, la extension entre dos claves con `actas`: un eslabon con las dos
# firmas, uno con la de la que entra sola, y dos eslabones de un salto -el titular que estuvo apagado
# y ve la clave de hoy-. Despues, las dos conductas del operador que la cadena delata, sembradas con
# sus claves de verdad: A, ya rotada, firmando otra vez con su contador restaurado -SOLAPAMIENTO por
# la regla 3- y B firmando con un contador fresco por debajo de su acta -fuera de su tramo, la
# regla 4-. Y un negativo por cada regla del lector y de la cadena que una mutacion produce.
# Desde el §686, las dos conductas tambien en su propio sobre, `tipo: "solapamiento"` -la cabeza y la
# cadena, sin la otra cabeza-, con las tres claves dentro de su tramo como negativos. Y desde el §687,
# un TESTIGO de verdad sobre otro diario: ve la rotacion de A a B, sigue, ve a B firmar por debajo
# de su acta con un contador fresco y se detiene; `--auditar --sobres` arma el sobre del solapamiento
# desde su diario, y el mando lo juzga sin el nodo ni el testigo.
# Y desde el §771, la rotacion en los sobres de CONSUMO y de COMPLETITUD, del mismo nodo: con A en
# curso, un envio de ceros que la capa rechaza deja su recibo y una cabeza de A cierra su era; con B
# ya rotada, dos consumos se publican antes de la cabeza emparejada, y la ausencia se pide bajo la
# cabeza de A y la presencia bajo la de B. El consumo entre las dos claves, y la completitud con la
# vigente de B, se juzgan con la cadena, sin ella y con ella cortada.
#
# FUERA del canon: levanta procesos y espera latidos. NO ESCRIBE EN EL ARBOL: todo vive en un
# temporal bajo $HOME, que borra al salir, y lo comprueba al final por `git status --porcelain`.
#
#   bash tools/banco_rotacion.sh [--guardar <dir>]
#
# --guardar  copia los sobres a <dir>, con `entradas.txt` -fichero|codigo|texto, las lineas del
#            manifiesto-: de ahi sale el catalogo, COPIADO. Desde el §771, los de consumo y los de
#            completitud van a <dir>/consumo/ y <dir>/completitud/, con `entradas-consumo.txt` y
#            `entradas-completitud.txt` en <dir>: cada uno a su familia.
set -u
msg(){ echo "BANCO-ROTACION| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
GUARDAR=""
while [ $# -gt 0 ]; do
  case "$1" in
    --guardar) GUARDAR="${2:?--guardar exige un directorio}"; mkdir -p "$GUARDAR"; shift ;;
    *) fallo "uso: bash tools/banco_rotacion.sh [--guardar <dir>]" ;;
  esac
  shift
done
[ -z "$(git status --porcelain)" ] || fallo "el arbol no esta limpio: el banco comprueba al final que no lo toca"
DIR=$(mktemp -d "$HOME/.banco_rotacion.XXXXXX")
trap 'rm -rf "$DIR"' EXIT INT TERM HUP QUIT
msg "compilando nodo y verificador en RELEASE (aqui se firma de verdad)"
cargo build --release -q -p zk-ssl-node -p zk-ssl-verify -p zk-ssl-cli 2>/dev/null \
  || cargo build --release -p zk-ssl-node -p zk-ssl-verify -p zk-ssl-cli || fallo "no compila"

python3 - "$DIR" "${GUARDAR:-}" <<'PY'
import copy, glob, json, os, shutil, subprocess, sys, time, urllib.request
DIR, GUARDAR = sys.argv[1], sys.argv[2]
NODO, MANDO, CLI = 'target/release/zk-ssl-node', 'target/release/zk-ssl-verify', 'target/release/zk-ssl-cli'
P = 8831
def msg(m): print('BANCO-ROTACION| ' + m, file=sys.stderr, flush=True)
def rojo(m): msg('ROJO: ' + m); sys.exit(1)
# Cuatro claves de prueba: A, B, C y la D que C compromete. Semillas de banco, '41' a '44' x 96.
CLAVES = {}
for k, s in zip('ABCD', ('41', '42', '43', '44')):
    CLAVES[k] = '%s/%s.hex' % (DIR, k)
    open(CLAVES[k], 'w').write(s * 96); os.chmod(CLAVES[k], 0o600)
def huella(k):
    p = subprocess.run([NODO, '--huella-de-clave-fichero', CLAVES[k]], capture_output=True, text=True)
    if p.returncode != 0:
        rojo('la huella de %s no sale: %s' % (k, p.stderr.strip()))
    return p.stdout.strip().splitlines()[-1]
def levantar(k, d, *banderas):
    os.makedirs(d, exist_ok=True)
    return subprocess.Popen([NODO, '--listen', '127.0.0.1:%d' % P, '--latido', '1',
                             '--clave-fichero', CLAVES[k], '--custodia', 'fichero',
                             '--diario', d + '/diario.jsonl', '--ledger', d + '/ledger',
                             '--contador-recepcion', d + '/recepcion.bin',
                             '--indice-firma', d + '/indice.bin', '--log', 'warn'] + list(banderas),
                            stdout=subprocess.DEVNULL, stderr=open('%s/nodo-%s.err' % (DIR, k), 'a'))
def parar(nodo):
    nodo.terminate()
    try:
        nodo.wait(timeout=20)
    except subprocess.TimeoutExpired:
        nodo.kill(); nodo.wait()
    time.sleep(0.5)
def rpc(m, p):
    c = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': m, 'params': p}).encode()
    r = urllib.request.Request('http://127.0.0.1:%d' % P, c, {'Content-Type': 'application/json'})
    return json.loads(urllib.request.urlopen(r, timeout=10).read())
def embebido(c):
    # XMSS^MT 40/8: el indice de hoja son los cinco primeros bytes de la firma, big-endian.
    return int(c['signature'][2:12], 16)
def cabeza(cuando, que):
    for _ in range(240):
        try:
            r = rpc('zkssl_signedEpochHead', {}).get('result', {})
            if r.get('available') and cuando(r):
                return r
        except Exception:
            pass
        time.sleep(0.25)
    rojo('ninguna cabeza firmada: ' + que)
def camino(old):
    for _ in range(40):
        try:
            r = rpc('zkssl_consistencyProof', {'oldSize': old}).get('result', {})
            if r.get('available'):
                return r
        except Exception:
            pass
        time.sleep(0.25)
    rojo('zkssl_consistencyProof no llego a available:true desde %s' % old)
def emparejado(old, que):
    # La pareja firmada es el acumulador ANTES de la cabeza: el camino de tamano t lo firma la
    # cabeza siguiente. Se espera a la que firma exactamente ese t, como en la extension.
    ack = camino(old)
    return ack, cabeza(lambda r: r['mmrSize'] == ack['mmrSize'], '%s, mmrSize %s' % (que, ack['mmrSize']))
HB, HC, HD = huella('B'), huella('C'), huella('D')
op = DIR + '/op'
# ── §771 · lo que hace falta para el consumo y la completitud, calcado de sus bancos. Las cabezas
#    por su indice declarado: la que cierra la era del recibo es la de `zkssl_recepPath`, no la
#    ultima, y hay que haberla visto (banco_completitud.sh).
cabezas = {}
def recoger():
    try:
        r = rpc('zkssl_signedEpochHead', {}).get('result', {})
        if r.get('available'):
            cabezas[int(r['index'], 16)] = r
    except Exception:
        pass
# El envio de CEROS: la forma del cable acepta sus bytes y la capa lo rechaza, y su recibo viaja
# en el error (banco_completitud.sh, el `envio_de_ceros` del nodo).
Z = '0x' + '00' * 32
cero_pi = {'rootOld': Z, 'rootNew': Z, 'frozenRoot': Z, 'pendingRootOld': Z, 'pendingRootNew': Z,
           'amount': '0x5', 'regulatoryLimit': '0x3e8', 'supplyOld': '0x0', 'supplyNew': '0x0'}
envio = {'receipt': {'proof': '0x' + '00' * 32, 'publicInputs': cero_pi, 'commitment': Z,
                     'notice': {'position': '0x0', 'salt': Z, 'amount': '0x5'}},
         'sender': '0x0', 'senderState': {'publicId': Z, 'balance': '0x5', 'nonce': '0x0'}, 'amount': '0x5'}
def digest_canonico(bytes_):
    # UN byte no nulo en el offset 3 de cada grupo de ocho: canonico en el campo leido en LE o en
    # BE (banco_consumo.sh).
    b = bytearray(32)
    for i, v in zip((3, 11, 19, 27), bytes_):
        b[i] = v
    return '0x' + b.hex()
CONSUMO = digest_canonico((0x52, 0x6F, 0x74, 0x41))
SEGUNDO = digest_canonico((0x2C, 0x9A, 0x37, 0xE1))
def camino_de_consumo(seq, que):
    r = rpc('zkssl_consumoPath', {'consumo': CONSUMO, 'seq': seq}).get('result', {})
    if not r.get('available'):
        rojo('zkssl_consumoPath (%s) no dio camino: %s' % (que, json.dumps(r)[:300]))
    return r

# ── A: la genesis, comprometiendo a B. Unas cabezas mas tras la custodiada, para que el acta de
#    B quede lejos de ella.
nodo = levantar('A', op, '--siguiente', HB)
try:
    h_a = cabeza(lambda r: int(r['mmrSize'], 16) >= 2, 'A, mmrSize >= 2')
    cabezas[int(h_a['index'], 16)] = h_a
    # §771 · el envio de ceros, con A en curso: su recibo, y la cabeza de A que cierra su era.
    r = rpc('zkssl_applySend', envio)
    if 'error' not in r or 'recepcion' not in r['error'].get('data', {}):
        rojo('el envio de ceros no dejo recibo en el error: %s' % json.dumps(r)[:300])
    rec = r['error']['data']['recepcion']
    camino_rx = None
    for _ in range(300):
        recoger()
        v = rpc('zkssl_recepPath', {'rx': rec['rx']}).get('result', {})
        if v.get('available'):
            camino_rx = v; break
        time.sleep(0.1)
    if camino_rx is None:
        rojo('zkssl_recepPath no dio el camino en 30 s')
    idx = int(camino_rx['index'], 16)
    for _ in range(50):
        if idx in cabezas:
            break
        time.sleep(0.05); recoger()
    if idx not in cabezas:
        rojo('la cabeza de indice %d, la que cierra la era del recibo, no se vio' % idx)
    cierre = cabezas[idx]
    ant = [i for i in cabezas if i < idx]
    q = int(cabezas[max(ant)]['recepCount'], 16) if ant else 0
    time.sleep(3)
finally:
    parar(nodo)
msg('A: cabeza custodiada de mmrSize %s, indice embebido %d' % (h_a['mmrSize'], embebido(h_a)))
msg('A: el envio de ceros se RECHAZA y deja su recibo (rx %s); la cabeza de A de indice embebido %d '
    'cierra su era (Q %d)' % (rec['rx'], embebido(cierre), q))
# ── B: la rotacion, con la firma de A.
nodo = levantar('B', op, '--siguiente', HC, '--clave-anterior-fichero', CLAVES['A'])
try:
    time.sleep(2)
    # §771 · el consumo se PUBLICA con B en curso, ANTES de la cabeza emparejada: la ausencia queda
    #    bajo la custodiada de A y la presencia bajo la de B. El segundo, a proposito: con uno solo
    #    los dos caminos son el mismo (banco_consumo.sh). La aritmetica de los seq se EXIGE. Antes,
    #    la primera cabeza de B: el nodo ya escucha, y el consumo entra con B en curso.
    cabeza(lambda r: r['publicKey'] != h_a['publicKey'], 'B en curso, antes del consumo')
    pub = rpc('zkssl_publishConsumo', {'consumo': CONSUMO}).get('result', {})
    if not pub.get('accepted'):
        rojo('publishConsumo no acepto el consumo: %s' % json.dumps(pub)[:300])
    log_seq = int(pub['logSeq'], 16)
    if not int(h_a['seq'], 16) < log_seq:
        rojo('el seq de la cabeza de A (%d) no queda por debajo del logSeq del consumo (%d)'
             % (int(h_a['seq'], 16), log_seq))
    if not rpc('zkssl_publishConsumo', {'consumo': SEGUNDO}).get('result', {}).get('accepted'):
        rojo('el segundo consumo no se acepto')
    ack_ab, h_b = emparejado(h_a['mmrSize'], 'B')
    if not int(h_b['seq'], 16) >= log_seq:
        rojo('el seq de la cabeza de B (%d) no alcanza el logSeq del consumo (%d)'
             % (int(h_b['seq'], 16), log_seq))
    ausencia = camino_de_consumo(h_a['seq'], 'ausencia bajo A')
    presencia = camino_de_consumo(h_b['seq'], 'presencia bajo B')
    time.sleep(2)
finally:
    parar(nodo)
msg('B: cabeza de mmrSize %s, indice embebido %d' % (h_b['mmrSize'], embebido(h_b)))
msg('B: consumo publicado en el logSeq %d, entre el seq %d de A y el %d de B'
    % (log_seq, int(h_a['seq'], 16), int(h_b['seq'], 16)))
# ── C: la rotacion SIN la firma de B: el acta la declara quemada.
nodo = levantar('C', op, '--siguiente', HD)
try:
    time.sleep(2)
    ack_bc, h_c1 = emparejado(h_b['mmrSize'], 'C desde B')
    ack_ac, h_c2 = emparejado(h_a['mmrSize'], 'C desde A')
    actas = rpc('zkssl_keyActs', {})['result']['actas']
finally:
    parar(nodo)
desdes = [int(a['acta']['desde'], 16) for a in actas]
if len(actas) != 3 or [a['firmaAnterior'] is not None for a in actas] != [False, True, False]:
    rojo('la cadena no es genesis + rotacion firmada por las dos + rotacion sin la vieja: %s'
         % [(a['acta']['desde'], a['firmaAnterior'] is not None) for a in actas])
d_b, d_c = desdes[1], desdes[2]
msg('la cadena de zkssl_keyActs: tres actas, desde %s' % desdes)
# ── A otra vez, YA rotada: su contador restaurado en un directorio fresco. Firma por encima del
#    desde de su sucesora: el SOLAPAMIENTO de la regla 3, que solo quien tiene la clave produce.
zombi = DIR + '/zombi'
os.makedirs(zombi)
shutil.copy(op + '/indice.bin', zombi + '/indice.bin')
nodo = levantar('A', zombi)
try:
    h_z = cabeza(lambda r: True, 'A restaurada')
finally:
    parar(nodo)
# ── C, la clave de hoy, firma por encima de la cabeza de A: la posterior de ese par.
nodo = levantar('C', op)
try:
    h_c3 = cabeza(lambda r: embebido(r) > embebido(h_z), 'C por encima de la A restaurada')
finally:
    parar(nodo)
# ── B con un contador FRESCO: firma por debajo del desde de su acta, fuera de su tramo (regla 4).
nodo = levantar('B', DIR + '/fresco')
try:
    h_f = cabeza(lambda r: embebido(r) > embebido(h_a), 'B fresca por encima de la custodiada de A')
finally:
    parar(nodo)
msg('A restaurada en la hoja %d (sucesora desde %d); B fresca en la %d (su tramo: %d a %d)'
    % (embebido(h_z), d_b, embebido(h_f), d_b, d_c))
if not embebido(h_z) >= d_b or not embebido(h_a) < embebido(h_f) <= d_b:
    rojo('las conductas sembradas no son las que el banco quiere medir')

fallos = 0
entradas = {}
def mando(nombre, sobre, codigo, texto, familia='rotacion'):
    global fallos
    ruta = '%s/%s.json' % (DIR, nombre)
    json.dump(sobre, open(ruta, 'w'), indent=1, sort_keys=True)
    p = subprocess.run([MANDO, ruta], capture_output=True, text=True)
    out = p.stdout + p.stderr
    ok = p.returncode == codigo and texto in out
    fallos += 0 if ok else 1
    msg('%s %-36s exit %d (se esperaba %d: %s)' % ('OK  ' if ok else 'ROJO', nombre, p.returncode, codigo, texto))
    if not ok:
        msg('     ' + out.strip().replace('\n', '\n     '))
    entradas.setdefault(familia, []).append('%s.json|%d|%s' % (nombre, codigo, texto))
    if GUARDAR:
        # §771: la rotacion, plana en <dir> como desde el §649; las otras familias, en la suya.
        destino = GUARDAR if familia == 'rotacion' else os.path.join(GUARDAR, familia)
        os.makedirs(destino, exist_ok=True)
        shutil.copy(ruta, destino)
def con(sobre, f):
    s = copy.deepcopy(sobre); f(s); return s
def nibble(hexs, en):
    c = hexs[en]
    return hexs[:en] + ('0' if c != '0' else '1') + hexs[en + 1:]
def extension(vieja, nueva, ack):
    return {'v': 1, 'tipo': 'extension', 'vieja': vieja, 'nueva': nueva, 'camino': ack['camino'],
            'actas': actas}
base = extension(h_a, h_b, ack_ab)
operador = 'el mismo OPERADOR en los dos extremos'
# ── los positivos
mando('rotacion-un-eslabon', base, 0,
      '(1 eslabon(es), la posterior desde la hoja %d): %s' % (d_b, operador))
mando('rotacion-sin-firma-de-la-vieja', extension(h_b, h_c1, ack_bc), 0,
      '(1 eslabon(es), la posterior desde la hoja %d): %s' % (d_c, operador))
mando('rotacion-dos-eslabones', extension(h_a, h_c2, ack_ac), 0,
      '(2 eslabon(es), la posterior desde la hoja %d): %s' % (d_c, operador))
# ── las conductas del operador, con sus claves
mando('neg-solapamiento-la-vieja-firma-despues', extension(h_z, h_c3, ack_ac), 1,
      'SOLAPAMIENTO: la clave que se va firmo en la hoja %d, y su sucesora empieza en la %d'
      % (embebido(h_z), d_b))
mando('neg-solapamiento-la-nueva-firma-antes', extension(h_a, h_f, ack_ab), 1,
      'SOLAPAMIENTO: la clave que llega firmo en la hoja %d, fuera de su tramo: por encima de la %d '
      'y por debajo de la %d' % (embebido(h_f), d_b, d_c))
# ── el sobre: sin actas, y lo que el lector no lee
mando('neg-sin-actas', con(base, lambda s: s.pop('actas')), 1,
      'las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante')
mando('neg-actas-no-es-lista', con(base, lambda s: s.update(actas=actas[0])), 1,
      'actas no es lista (la cadena de zkssl_keyActs)')
mando('neg-acta-sin-procedencia', con(base, lambda s: s['actas'][1]['acta'].pop('procedencia')), 1,
      'actas[1]: falta acta.procedencia: null en la genesis')
mando('neg-acta-sin-firma-anterior', con(base, lambda s: s['actas'][1].pop('firmaAnterior')), 1,
      'actas[1]: falta firmaAnterior: null si la clave que se va no firma')
mando('neg-acta-desde-no-minimo', con(base, lambda s: s['actas'][1]['acta'].update(desde='0x0' + s['actas'][1]['acta']['desde'][2:])), 1,
      'actas[1]: desde: cantidad hex no minima')
mando('neg-acta-siguiente-corto', con(base, lambda s: s['actas'][0]['acta'].update(siguiente=s['actas'][0]['acta']['siguiente'][:-2])), 1,
      'actas[0]: siguiente: 31 bytes, se esperaban 32')
# ── la cadena: un eslabon por regla de la D-C que una mutacion produce
pre = 'las cabezas llevan claves DISTINTAS y las actas no las unen: '
mando('neg-cadena-sin-genesis', con(base, lambda s: s.update(actas=s['actas'][1:])), 1,
      pre + 'el acta 0 de la cadena no vale: una rotacion se juzga contra el acta de la que viene, y no esta')
mando('neg-cadena-corta', con(base, lambda s: s.update(actas=s['actas'][:1])), 1,
      pre + 'la clave que llega no esta en la cadena: nadie la comprometio')
mando('neg-genesis-repetida', con(base, lambda s: s.update(actas=s['actas'][:1] + s['actas'])), 1,
      pre + 'el acta 1 de la cadena no vale: un acta genesis no continua a otra: no lleva procedencia')
mando('neg-genesis-con-firma-anterior', con(base, lambda s: s['actas'][0].update(firmaAnterior=s['actas'][1]['firmaAnterior'])), 1,
      pre + 'el acta 0 de la cadena no vale: un acta genesis no tiene clave anterior que la firme')
mando('neg-esquema-ajeno', con(base, lambda s: s['actas'][0]['acta'].update(esquema='0x100000006')), 1,
      pre + 'el acta 0 de la cadena no vale: el acta presenta una clave de esquema 0x100000006, que este '
      'verificador no conoce')
mando('neg-anterior-ajena', con(base, lambda s: s['actas'][1]['acta']['procedencia'].update(anterior=s['actas'][0]['acta']['siguiente'])), 1,
      pre + 'el acta 1 de la cadena no vale: la anterior del acta no es la clave del acta previa')
mando('neg-clave-no-comprometida', con(base, lambda s: s['actas'][1]['acta'].update(clave=s['actas'][2]['acta']['clave'])), 1,
      pre + 'el acta 1 de la cadena no vale: la clave que entra NO es la que el acta previa comprometio')
mando('neg-desde-no-crece', con(base, lambda s: s['actas'][2]['acta'].update(desde=s['actas'][1]['acta']['desde'])), 1,
      pre + 'el acta 2 de la cadena no vale: el desde %d no supera el %d de la clave previa' % (d_b, d_b))
mando('neg-desde-tocado', con(base, lambda s: s['actas'][1]['acta'].update(desde=hex(d_b + 1))), 1,
      pre + 'el acta 1 de la cadena no vale: la firma de la clave que entra: la firma es VALIDA pero de otro mensaje')
mitad = lambda h: len(h) // 2
mando('neg-firma-tocada', con(base, lambda s: s['actas'][1].update(firma=nibble(s['actas'][1]['firma'], mitad(s['actas'][1]['firma'])))), 1,
      pre + 'el acta 1 de la cadena no vale: la firma de la clave que entra: la firma no verifica')
mando('neg-firma-de-la-vieja-tocada', con(base, lambda s: s['actas'][1].update(firmaAnterior=nibble(s['actas'][1]['firmaAnterior'], mitad(s['actas'][1]['firmaAnterior'])))), 1,
      pre + 'el acta 1 de la cadena no vale: la firma de la clave que se va: la firma no verifica')
# ── las actas no ordenan: con los papeles cambiados la continuidad se sostiene y cae la extension
mando('neg-papeles-cambiados', con(base, lambda s: s.update(vieja=h_b, nueva=h_a)), 1,
      'NO extiende a la vieja')
# ── §686 · el sobre del SOLAPAMIENTO: una cabeza y la cadena. DETECCION con salida 0.
def solap(cab, act=None):
    return {'v': 1, 'tipo': 'solapamiento', 'cabeza': cab, 'actas': actas if act is None else act}
dentro = 'la cabeza firma en la hoja %d, dentro de su tramo %s: no hay solapamiento'
mando('solapamiento-la-vieja-firma-despues', solap(h_z), 0,
      'VERDE: SOLAPAMIENTO - la clave firmo una cabeza en la hoja %d' % embebido(h_z))
mando('solapamiento-la-nueva-firma-antes', solap(h_f), 0,
      'VERDE: SOLAPAMIENTO - la clave firmo una cabeza en la hoja %d' % embebido(h_f))
mando('neg-solap-la-vieja-en-su-tramo', solap(h_a), 1, dentro % (embebido(h_a),
      '(acta 0, la genesis: cualquier hoja salvo la %d de su acta, por debajo de la %d)' % (desdes[0], d_b)))
mando('neg-solap-la-nueva-en-su-tramo', solap(h_b), 1, dentro % (embebido(h_b),
      '(acta 1: por encima de la hoja %d, por debajo de la %d)' % (d_b, d_c)))
mando('neg-solap-la-ultima-en-su-tramo', solap(h_c1), 1, dentro % (embebido(h_c1),
      '(acta 2: por encima de la hoja %d)' % d_c))
mando('neg-solap-clave-fuera-de-la-cadena', solap(h_c1, actas[:2]), 1,
      'la clave de la cabeza no esta en la cadena: nadie la comprometio')
mando('neg-solap-cadena-sin-genesis', solap(h_z, actas[1:]), 1,
      'el acta 0 de la cadena no vale: una rotacion se juzga contra el acta de la que viene, y no esta')
mando('neg-solap-sin-actas', con(solap(h_a), lambda s: s.pop('actas')), 1,
      'falta actas (la cadena de zkssl_keyActs: sin ella no hay tramo que cruzar)')
mando('neg-solap-actas-no-es-lista', solap(h_a, actas[0]), 1, 'actas no es lista (la cadena de zkssl_keyActs)')
mando('neg-solap-sin-cabeza', con(solap(h_a), lambda s: s.pop('cabeza')), 1,
      'falta cabeza (la firmada que se juzga contra su tramo)')
mando('neg-solap-cabeza-v2', solap(dict(h_a, formatVersion='0x2')), 1,
      'cabeza: formatVersion 2: el sobre del solapamiento lee cabezas v3, v4, v5 o v6: las que firma un nodo con actas')
# ── §771 · el CONSUMO entre las dos claves: la vieja de A, la nueva de B, la cadena que las une.
consumo = {'v': 1, 'tipo': 'consumo', 'vieja': h_a, 'nueva': h_b, 'camino': ack_ab['camino'],
           'consumo': CONSUMO, 'ausencia': ausencia['camino'], 'presencia': presencia['camino'],
           'actas': actas}
mando('consumo-rotado', consumo, 0,
      '2/5 claves distintas que las actas unen (1 eslabon(es), la posterior desde la hoja %d), y las '
      'dos cabezas llevan consRoot' % d_b, 'consumo')
mando('rechazo-cons-rotado-sin-actas', con(consumo, lambda s: s.pop('actas')), 1,
      'las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante', 'consumo')
mando('rechazo-cons-rotado-cadena-corta', con(consumo, lambda s: s.update(actas=actas[:1])), 1,
      pre + 'la clave que llega no esta en la cadena: nadie la comprometio', 'consumo')
# ── §771 · la COMPLETITUD: el cierre de A, la vigente de B. La ventana se mide en la cuenta UNICA
#    de indices del operador (RFC-0015 D-A): con B dentro de ella el sobre es prematuro.
completitud = {'v': 1, 'tipo': 'completitud', 'cierre': cierre, 'recepcion': rec,
               'limiteAnterior': '0x%x' % q, 'camino': camino_rx['camino'], 'vigente': h_b,
               'actas': actas}
mando('rotada-ventana-abierta', completitud, 1,
      'ventana ABIERTA: la cabeza vigente tiene indice acreditado %d y la ventana empieza en %d'
      % (embebido(h_b) + 1, embebido(cierre)), 'completitud')
mando('neg-rotada-vigente-sin-actas', con(completitud, lambda s: s.pop('actas')), 1,
      'las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante', 'completitud')
mando('neg-rotada-vigente-cadena-corta', con(completitud, lambda s: s.update(actas=actas[:1])), 1,
      pre + 'la clave que llega no esta en la cadena: nadie la comprometio', 'completitud')
# ── §687 · el TESTIGO, de verdad, sobre otro diario: la rotacion de A a B la explica la cadena y
#    sigue; B con un contador fresco firma por debajo de su acta y se detiene. Del diario sale el
#    sobre, y el mando lo juzga sin el nodo ni el testigo.
op2, diario_t = DIR + '/op2', DIR + '/testigo.jsonl'
testigo = subprocess.Popen([CLI, 'witness', '--nodo', 'http://127.0.0.1:%d' % P, '--cada', '1',
                            '--veces', '90', '--no-color', '--diario', diario_t],
                           stdout=open(DIR + '/testigo.out', 'w'), stderr=subprocess.STDOUT)
try:
    nodo = levantar('A', op2, '--siguiente', HB)
    try:
        cabeza(lambda r: int(r['mmrSize'], 16) >= 2, 'A, con el testigo mirando')
        time.sleep(3)
    finally:
        parar(nodo)
    nodo = levantar('B', op2, '--siguiente', HC, '--clave-anterior-fichero', CLAVES['A'])
    try:
        cabeza(lambda r: r['publicKey'] != h_a['publicKey'], 'B rotada, con el testigo mirando')
        time.sleep(4)
    finally:
        parar(nodo)
    nodo = levantar('B', DIR + '/fresco-testigo')
    try:
        testigo.wait(timeout=60)
    except subprocess.TimeoutExpired:
        rojo('el testigo no se detuvo ante la B que firma por debajo de su acta')
    finally:
        parar(nodo)
finally:
    if testigo.poll() is None:
        testigo.kill(); testigo.wait()
clases = [json.loads(l).get('clase') for l in open(diario_t) if l.strip()]
if 'rotada' not in clases or clases[-1] != 'solapamiento':
    rojo('el diario del testigo no es rotada y despues solapamiento: %s' % clases)
msg('el testigo: %d lineas, rotada y despues solapamiento; se detuvo' % len(clases))
p = subprocess.run([CLI, 'witness', '--auditar', diario_t, '--sobres', DIR + '/sobres-testigo', '--no-color'],
                   capture_output=True, text=True)
sobres = sorted(glob.glob(DIR + '/sobres-testigo/*.json'))
if len(sobres) != 1:
    rojo('--auditar --sobres no armo un sobre: %s' % (p.stdout + p.stderr)[-600:])
q = subprocess.run([MANDO, sobres[0]], capture_output=True, text=True)
ok = q.returncode == 0 and 'VERDE: SOLAPAMIENTO' in q.stdout
fallos += 0 if ok else 1
msg('%s el sobre que arma el testigo desde su diario: exit %d (se esperaba 0: VERDE: SOLAPAMIENTO)'
    % ('OK  ' if ok else 'ROJO', q.returncode))
if not ok:
    msg('     ' + (q.stdout + q.stderr).strip().replace('\n', '\n     '))
if GUARDAR:
    open(GUARDAR + '/entradas.txt', 'w').write('\n'.join(entradas['rotacion']) + '\n')
    for familia in ('consumo', 'completitud'):
        open('%s/entradas-%s.txt' % (GUARDAR, familia), 'w').write('\n'.join(entradas[familia]) + '\n')
sys.exit(1 if fallos else 0)
PY
RC=$?
[ "$RC" = 0 ] || fallo "algun sobre no dijo lo que debia"
[ -z "$(git status --porcelain)" ] || fallo "el banco dejo el arbol sucio: no debe tocarlo"
[ -n "$GUARDAR" ] && msg "sobres guardados en $GUARDAR (de ahi sale el catalogo, COPIADO)"
msg "BANCO-ROTACION VERDE"
