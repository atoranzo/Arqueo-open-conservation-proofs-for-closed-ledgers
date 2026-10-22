#!/usr/bin/env bash
# tools/banco_bifurcacion.sh -- §712, RFC-0012 D-C: el reloj es el indice EMBEBIDO.
#
# La consistencia del MMR prueba que una cima contiene a otra. Con el MISMO mmrSize es la identidad,
# y no ve dos cabezas DISTINTAS en el mismo sitio de la historia: una bifurcacion firmada pasaba por
# extension. Tampoco ve una cabeza posterior que firma por DEBAJO de una anterior. Este banco las
# produce con un nodo REAL y el binario SIN TOCAR -las mentiras con clave no piden otro codigo, piden
# la CLAVE (§598)-, y las juzga con el mando y con el testigo:
#
#   CORRIDA A  la semilla '39' firma; el nodo para, su contador salta K hojas -los indices HUERFANOS
#              de una caida entre reservar y firmar, el caso normal (§335)- y vuelve con su diario.
#              Un testigo VIVO lo mira entero y NO se detiene: huerfanos y reinicio son honestos.
#              El sobre de extension por encima de los huerfanos es VERDE.
#   ESCENA 1   LA BIFURCACION. El diario de A recortado a T lineas y el MISMO contador: la cabeza
#              de mmrSize T vuelve con otra hoja, otro contenido -un consumo publicado- y la misma
#              cima. El testigo vivo SE DETIENE con `no-extiende` en T. Sobres de extension y de
#              consumo: ROJO, «mismo mmrSize y digest distinto». §714: la clase es
#              `bifurcacion`, con su `t` y el digest de la primera cabeza de ese tamaño.
#   ESCENA 2   LA VISTA DIVIDIDA, con el contador restaurado a la hoja de A en T: el mismo mmrSize y
#              el mismo indice embebido. Sobre de consumo: ROJO, por la misma causa.
#   ESCENA 3   EL RELOJ QUE RETROCEDE. El diario y el contador restaurados de antes de los
#              huerfanos: la historia sigue CONSISTENTE con la cabeza de A, y la hoja de la nueva
#              queda por debajo. Sobres de extension y de consumo: ROJO, «el indice embebido no crece».
#              Y bajo un testigo VIVO (§712): custodia una cabeza de A -una copia con el diario de
#              entonces y el contador en su hoja, que repite las de A byte a byte-, y llega el
#              operador restaurado, sin trafico: el MISMO contenido en cada mmrSize, una historia
#              que CONTIENE a la custodiada, hojas por debajo. SE DETIENE con `no-extiende` desde la
#              custodiada en la primera posterior; sin el reloj la anotaba `extiende` y, con
#              --cofirmar, la cofirmaba. Su --auditar lo ve en frio: `indice-retrocede`.
#   ESCENA 4   LA RECUPERACION DEL §690 bajo un testigo vivo: A ('41') rota a B ('42') con
#              `--desde-minimo`, por encima de todo lo que A pudo firmar. El testigo rota y sigue.
#   ESCENA 5   §714 · EL REINICIO, CON Y SIN DIARIO, bajo un testigo VIVO. Con su diario un
#              nodo honesto no repite un mmrSize: la hoja del MMR entra donde se anota el diario, y
#              la cabeza se sirve despues (`zk-ssl-node`, latido.rs, `conservar`). Dos reinicios con
#              trafico entre medias, uno con SIGTERM y otro con SIGKILL: el testigo no se detiene, y
#              lo que el nodo sirvio lleva UN digest por mmrSize. Despues, el mismo libro y el mismo
#              contador con el diario PERDIDO: el nodo vuelve a firmar desde el mmrSize 0 con el
#              libro de ahora, otra cosa donde ya firmo. El testigo SE DETIENE con `bifurcacion` en
#              un tamaño que ya habia visto, sin esperar a que el nodo pase de lo custodiado -el del
#              §712 lo anotaba `por-detras`-, y su --auditar --sobres arma el modo 5 del ancla, que el
#              mando juzga VERDE: BIFURCACION.
#   Y los dos sentidos de la vista dividida del ancla (`spec/vectors/ancla/vista-dividida.json`) como
#   sobre de extension, recombinados sin otra corrida: ROJO, «mismo mmrSize y digest distinto».
#
# FUERA del canon: levanta procesos y espera latidos. NO ESCRIBE EN EL ARBOL: todo vive en un
# temporal bajo $HOME, que borra al salir, y lo comprueba al final por `git status --porcelain`.
# `BANCO_BIN` elige los binarios (por defecto, los de release del target de este arbol, el que da
# `tools/target_dir.sh`): con los de antes del arreglo, los rojos de arriba salen VERDE y el banco lo
# dice uno a uno.
#
#   bash tools/banco_bifurcacion.sh [--guardar <dir>]
#
# --guardar  copia los sobres a <dir>/paquete y <dir>/consumo, con `entradas.txt` -fichero|codigo|
#            texto, las lineas del manifiesto-: de ahi sale el catalogo, COPIADO.
set -u
msg(){ echo "BANCO-BIFURCACION| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
GUARDAR=""
while [ $# -gt 0 ]; do
  case "$1" in
    --guardar) GUARDAR="${2:?--guardar exige un directorio}"; mkdir -p "$GUARDAR"; shift ;;
    *) fallo "uso: bash tools/banco_bifurcacion.sh [--guardar <dir>]" ;;
  esac
  shift
done
[ -z "$(git status --porcelain)" ] || fallo "el arbol no esta limpio: el banco comprueba al final que no lo toca"
# Bajo $HOME, no /tmp: el guardian del indice se niega sobre un fsync que no persiste (§290).
DIR=$(mktemp -d "$HOME/.banco_bifurcacion.XXXXXX")
trap 'rm -rf "$DIR"' EXIT INT TERM HUP QUIT
# Con --locked, como los demas bancos (§786), y los binarios donde cargo los deja (§855).
if [ -n "${BANCO_BIN:-}" ]; then
  BIN="$BANCO_BIN"
else
  msg "compilando nodo, cli y mando en RELEASE (aqui se firma de verdad)"
  cargo build --locked --release -q -p zk-ssl-node -p zk-ssl-cli -p zk-ssl-verify 2>/dev/null \
    || cargo build --locked --release -p zk-ssl-node -p zk-ssl-cli -p zk-ssl-verify || fallo "no compila"
  TD=$(bash tools/target_dir.sh) || fallo "no se donde deja cargo los binarios (tools/target_dir.sh)"
  BIN="$TD/release"
fi

python3 - "$DIR" "${GUARDAR:-}" "$BIN" <<'PY'
import glob, json, os, shutil, struct, subprocess, sys, time, urllib.request
DIR, GUARDAR, BIN = sys.argv[1], sys.argv[2], sys.argv[3]
NODO, CLI, MANDO = BIN + '/zk-ssl-node', BIN + '/zk-ssl-cli', BIN + '/zk-ssl-verify'
P, P2 = 8851, 8852
def msg(m): print('BANCO-BIFURCACION| ' + m, file=sys.stderr, flush=True)
def rojo(m): msg('ROJO: ' + m); sys.exit(1)
fallos = 0
def exigir(ok, texto, detalle=''):
    global fallos
    fallos += 0 if ok else 1
    msg('%s %s' % ('OK  ' if ok else 'ROJO', texto))
    if not ok and detalle:
        msg('     ' + detalle.strip().replace('\n', '\n     '))
CLAVES = {}
for k, s in (('E', '39'), ('A', '41'), ('B', '42'), ('C', '43')):
    CLAVES[k] = '%s/%s.hex' % (DIR, k)
    open(CLAVES[k], 'w').write(s * 96); os.chmod(CLAVES[k], 0o600)
procesos = []
def levantar(k, d, port, latido, *banderas):
    os.makedirs(d, exist_ok=True)
    p = subprocess.Popen([NODO, '--listen', '127.0.0.1:%d' % port, '--latido', str(latido),
                          '--clave-fichero', CLAVES[k], '--custodia', 'fichero',
                          '--diario', d + '/diario.jsonl', '--ledger', d + '/ledger',
                          '--contador-recepcion', d + '/recepcion.bin',
                          '--indice-firma', d + '/indice.bin', '--log', 'warn'] + list(banderas),
                         stdout=subprocess.DEVNULL, stderr=open(d + '/nodo.err', 'a'))
    procesos.append(p)
    return p
def parar(p):
    if p.poll() is None:
        p.terminate()
        try:
            p.wait(timeout=20)
        except subprocess.TimeoutExpired:
            p.kill(); p.wait()
    time.sleep(0.5)
def rpc(port, m, params):
    c = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': m, 'params': params}).encode()
    r = urllib.request.Request('http://127.0.0.1:%d' % port, c, {'Content-Type': 'application/json'})
    return json.loads(urllib.request.urlopen(r, timeout=10).read())
def responde(port):
    for _ in range(240):
        try:
            rpc(port, 'zkssl_params', {})
            return
        except Exception:
            time.sleep(0.1)
    rojo('el nodo del puerto %d no responde' % port)
def embebido(c):
    # XMSS^MT 40/8: el indice de hoja son los cinco primeros bytes de la firma, big-endian.
    return int(c['signature'][2:12], 16)
def q(x):
    return int(x, 16)
def cabeza(port, cuando, que, vistas=None):
    for _ in range(400):
        try:
            r = rpc(port, 'zkssl_signedEpochHead', {}).get('result', {})
            if r.get('available'):
                if vistas is not None:
                    vistas.setdefault(q(r['mmrSize']), r)
                if cuando(r):
                    return r
        except Exception:
            pass
        time.sleep(0.25)
    rojo('ninguna cabeza firmada: ' + que)
def emparejado(port, old, que, vistas=None):
    # La pareja firmada es el acumulador ANTES de la cabeza: el camino de tamano t lo firma la
    # cabeza siguiente. Se espera a la que firma exactamente ese t (banco_extension.sh).
    for _ in range(40):
        try:
            ack = rpc(port, 'zkssl_consistencyProof', {'oldSize': '0x%x' % old}).get('result', {})
            if ack.get('available'):
                return ack, cabeza(port, lambda r: r['mmrSize'] == ack['mmrSize'],
                                   '%s, mmrSize %s' % (que, ack['mmrSize']), vistas)
        except Exception:
            pass
        time.sleep(0.25)
    rojo('zkssl_consistencyProof no llego a available:true desde %d' % old)
def camino_consumo(port, consumo, seq, que):
    r = rpc(port, 'zkssl_consumoPath', {'consumo': consumo, 'seq': seq}).get('result', {})
    if not r.get('available'):
        rojo('consumoPath(%s) no dio camino: %s' % (que, json.dumps(r)[:300]))
    return r['camino']
def lineas(ruta):
    return [l for l in open(ruta).read().splitlines() if l.strip()]
def contador(d):
    return struct.unpack('<Q', open(d + '/indice.bin', 'rb').read(8))[0]
def poner_contador(d, v):
    with open(d + '/indice.bin', 'wb') as f:
        f.write(struct.pack('<Q', v)); f.flush(); os.fsync(f.fileno())
def recortar(origen, destino, t, cont=None):
    # La copia del operador: el diario hasta la linea t (el MMR sembrado con t hojas) y, si se
    # pide, el contador puesto a mano. El libro y el registro de recepcion, tal cual.
    shutil.copytree(origen, destino)
    open(destino + '/diario.jsonl', 'w').write('\n'.join(lineas(origen + '/diario.jsonl')[:t]) + '\n')
    if os.path.exists(destino + '/nodo.err'):
        os.remove(destino + '/nodo.err')
    if cont is not None:
        poner_contador(destino, cont)
# El consumo, canonico en el campo por construccion (el molde de banco_consumo.sh): un byte no
# nulo en el offset 3 de cada grupo de ocho, muy por debajo de p en las dos lecturas.
b = bytearray(32)
for i, v in ((3, 0x3B), (11, 0x6E), (19, 0x2D), (27, 0x5A)):
    b[i] = v
CONSUMO = '0x' + b.hex()
def publicar(port, consumo=CONSUMO):
    r = rpc(port, 'zkssl_publishConsumo', {'consumo': consumo}).get('result', {})
    if not r.get('accepted'):
        rojo('publishConsumo no acepto el consumo: %s' % json.dumps(r)[:300])
def consumo_n(k):
    # §714: otro consumo canonico por cada k, con el mismo molde que CONSUMO.
    c = bytearray(b)
    c[3] = 0x10 + k
    return '0x' + c.hex()
def mirar(port, hasta, todas, que):
    # §714: TODO lo que el nodo sirve, por mmrSize -no solo la primera cabeza de cada tamaño-,
    # hasta que sirve una de mmrSize >= hasta.
    for _ in range(400):
        try:
            r = rpc(port, 'zkssl_signedEpochHead', {}).get('result', {})
            if r.get('available'):
                todas.setdefault(q(r['mmrSize']), set()).add(r['epochDigest'])
                if q(r['mmrSize']) >= hasta:
                    return r
        except Exception:
            pass
        time.sleep(0.25)
    rojo('ninguna cabeza firmada: ' + que)
def matar_ya(p):
    # SIGKILL: el reinicio por caida, sin que el nodo cierre nada.
    if p.poll() is None:
        p.kill(); p.wait()
    time.sleep(0.5)
entradas = {'paquete': [], 'consumo': []}
def mando(familia, nombre, sobre, codigo, texto):
    global fallos
    ruta = '%s/%s' % (DIR, nombre)
    # Cada familia con su escritura: la del paquete, compacta y con salto final; la del consumo,
    # la de json.dumps sin salto, como sus capturas.
    if familia == 'paquete':
        open(ruta, 'w').write(json.dumps(sobre, separators=(',', ':')) + '\n')
    else:
        open(ruta, 'w').write(json.dumps(sobre))
    p = subprocess.run([MANDO, ruta], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    out = p.stdout + p.stderr
    ok = p.returncode == codigo and texto in out
    fallos += 0 if ok else 1
    ultima = (out.strip().splitlines() or [''])[-1]
    msg('%s %-44s exit %d (se esperaba %d: %s)' % ('OK  ' if ok else 'ROJO', nombre, p.returncode, codigo, texto))
    if not ok:
        msg('     ' + ultima[:300])
    entradas[familia].append('%s|%d|%s' % (nombre, codigo, texto))
    if GUARDAR:
        os.makedirs('%s/%s' % (GUARDAR, familia), exist_ok=True)
        shutil.copy(ruta, '%s/%s/%s' % (GUARDAR, familia, nombre))
    return out
def testigo(nombre, port, veces):
    d = '%s/%s' % (DIR, nombre); os.makedirs(d)
    p = subprocess.Popen([CLI, 'witness', '--nodo', 'http://127.0.0.1:%d' % port, '--cada', '1',
                          '--veces', str(veces), '--diario', d + '/diario.jsonl', '--no-color'],
                         stdout=open(d + '/salida.txt', 'w'), stderr=subprocess.STDOUT)
    procesos.append(p)
    return p, d
def esperar(p, seg):
    try:
        return p.wait(timeout=seg)
    except subprocess.TimeoutExpired:
        return None
def diario_de(d):
    return [json.loads(l) for l in lineas(d + '/diario.jsonl')]
def clases(ls):
    rachas = []
    for v in ls:
        c = v.get('clase', '?') + ('/' + v['consistencia']['clase'] if 'consistencia' in v else '')
        if rachas and rachas[-1][0] == c:
            rachas[-1][1] += 1
        else:
            rachas.append([c, 1])
    return ' · '.join(c if n == 1 else '%s x%d' % (c, n) for c, n in rachas)

try:
    # ── CORRIDA A, bajo un testigo VIVO: huerfanos y reinicio, honestos ──
    K = 5
    A = DIR + '/a'
    vistas = {}
    w, dw = testigo('testigo', P, 0)
    nodo = levantar('E', A, P, 1)
    try:
        cabeza(P, lambda r: q(r['mmrSize']) >= 3, 'A, mmrSize >= 3', vistas)
        time.sleep(1)
    finally:
        parar(nodo)
    antes = contador(A)
    poner_contador(A, antes + K)
    t_b = len(lineas(A + '/diario.jsonl'))
    msg('corrida A: parada con el contador en %d y %d cabezas en el diario; el contador salta a %d: '
        '%d hojas HUERFANAS, reservadas y nunca firmadas' % (antes, t_b, antes + K, K))
    nodo = levantar('E', A, P, 1)
    try:
        cabeza(P, lambda r: q(r['mmrSize']) >= t_b + 1, 'A tras los huerfanos', vistas)
        ack_h, h_h = emparejado(P, t_b - 1, 'A, por encima de los huerfanos', vistas)
        cabeza(P, lambda r: q(r['mmrSize']) >= t_b + 5, 'A, unas cabezas mas', vistas)
        time.sleep(2)
    finally:
        parar(nodo)
    la = [json.loads(l) for l in lineas(A + '/diario.jsonl')]
    a_de = dict(vistas)
    if any(t not in a_de for t in range(t_b + 5)):
        rojo('faltan cabezas de A en lo consultado: %s' % sorted(a_de))
    # La primera cabeza tras el salto firma en la hoja del contador. Entre la ultima de antes y
    # ella quedan las K del salto y las que la parada ya hubiera dejado reservadas sin firmar.
    exigir(embebido(a_de[t_b]) == antes + K and embebido(a_de[t_b - 1]) < antes,
           'la primera cabeza tras los huerfanos firma en la hoja %d: %d hojas por encima de la '
           'ultima de antes (%d), %d del salto y %d de la parada'
           % (embebido(a_de[t_b]), embebido(a_de[t_b]) - embebido(a_de[t_b - 1]) - 1,
              embebido(a_de[t_b - 1]), K, antes - embebido(a_de[t_b - 1]) - 1))
    exigir(w.poll() is None, 'el testigo vivo NO se detiene ante los huerfanos ni ante el reinicio',
           open(dw + '/salida.txt').read()[-800:])
    mando('paquete', 'extension-sobre-huerfanos.json',
          {'v': 1, 'tipo': 'extension', 'vieja': a_de[t_b - 1], 'nueva': h_h, 'camino': ack_h['camino']},
          0, 'VERDE: la extension se sostiene sin el nodo')

    # ── ESCENA 1 · la bifurcacion: el diario recortado y el MISMO contador, bajo el testigo ──
    vistos_w = sorted(q(v['mmrSize']) for v in diario_de(dw)
                      if v.get('clase') in ('nueva', 'repetida', 'hueco') and v.get('mmrSize'))
    candidatas = [t for t in vistos_w if t > t_b and t in a_de]
    if not candidatas:
        rojo('el testigo no vio ninguna cabeza de A por encima de los huerfanos: %s' % vistos_w)
    T = candidatas[0]
    B = DIR + '/b'
    recortar(A, B, T)
    msg('ESCENA 1: el diario recortado a %d lineas y el contador de A (%d), en el puerto del testigo'
        % (T, contador(B)))
    nodo = levantar('E', B, P, 3)
    try:
        responde(P)
        publicar(P)
        b_t = cabeza(P, lambda r: q(r['mmrSize']) == T, 'B, mmrSize %d' % T)
        rw = esperar(w, 30)
        aus = camino_consumo(P, CONSUMO, a_de[T]['seq'], 'ausencia bajo la de A')
        pre = camino_consumo(P, CONSUMO, b_t['seq'], 'presencia bajo la de B')
    finally:
        parar(nodo)
    exigir(b_t['epochDigest'] != a_de[T]['epochDigest'] and b_t['mmrRoot'] == a_de[T]['mmrRoot']
           and embebido(b_t) > max(embebido(c) for c in a_de.values()),
           'dos cabezas de mmrSize %d con la misma cima y digests distintos, hojas %d y %d'
           % (T, embebido(a_de[T]), embebido(b_t)))
    lw = diario_de(dw)
    ultima = lw[-1] if lw else {}
    cons = ultima.get('consistencia', {})
    # §714: la clase es `bifurcacion`, con su tamaño y el digest de la PRIMERA cabeza que el
    # testigo vio con el (la de A). El §712 la anotaba `no-extiende` con deT y aT en T.
    exigir(rw not in (None, 0) and cons.get('clase') == 'bifurcacion'
           and cons.get('t') == '0x%x' % T and cons.get('digestPrevio') == a_de[T]['epochDigest']
           and 'bifurcacion o reinicio sin diario' in open(dw + '/salida.txt').read(),
           'el testigo vivo SE DETIENE con bifurcacion en el mmrSize %d (exit %s)' % (T, rw),
           open(dw + '/salida.txt').read()[-800:] + '\nclases: ' + clases(lw))
    if w.poll() is None:
        w.kill(); w.wait()
    p = subprocess.run([CLI, 'witness', '--auditar', dw + '/diario.jsonl', '--no-color'],
                       capture_output=True, text=True)
    exigir(p.returncode != 0 and 'bifurcacion o reinicio sin diario' in p.stdout,
           '--auditar del diario del testigo ve la bifurcacion SIN el nodo', (p.stdout + p.stderr)[-800:])
    ext = {'v': 1, 'tipo': 'extension', 'vieja': a_de[T], 'nueva': b_t, 'camino': []}
    mando('paquete', 'rechazo-ext-bifurcada-mismo-mmrSize.json', ext, 1,
          'extension: mismo mmrSize y digest distinto')
    mando('consumo', 'rechazo-cons-bifurcada-mismo-mmrSize.json',
          dict(ext, tipo='consumo', consumo=CONSUMO, ausencia=aus, presencia=pre), 1,
          'extension: mismo mmrSize y digest distinto')

    # ── ESCENA 2 · la vista dividida: el contador restaurado a la hoja de A en T ──
    C = DIR + '/c'
    recortar(A, C, T, q(la[T - 1]['index']))
    msg('ESCENA 2: el diario recortado a %d lineas y el contador en %d, el de la linea %d'
        % (T, contador(C), T))
    nodo = levantar('E', C, P2, 3)
    try:
        responde(P2)
        publicar(P2)
        c_t = cabeza(P2, lambda r: q(r['mmrSize']) == T, 'C, mmrSize %d' % T)
        aus = camino_consumo(P2, CONSUMO, a_de[T]['seq'], 'ausencia bajo la de A')
        pre = camino_consumo(P2, CONSUMO, c_t['seq'], 'presencia bajo la de C')
    finally:
        parar(nodo)
    exigir(embebido(c_t) == embebido(a_de[T]) and c_t['epochDigest'] != a_de[T]['epochDigest'],
           'dos cabezas de mmrSize %d con la MISMA hoja %d y digests distintos' % (T, embebido(c_t)))
    mando('consumo', 'rechazo-cons-vista-dividida.json',
          {'v': 1, 'tipo': 'consumo', 'vieja': a_de[T], 'nueva': c_t, 'camino': [],
           'consumo': CONSUMO, 'ausencia': aus, 'presencia': pre}, 1,
          'extension: mismo mmrSize y digest distinto')

    # ── ESCENA 3 · el reloj que retrocede: diario y contador de antes de los huerfanos ──
    D = DIR + '/d'
    recortar(A, D, t_b, q(la[t_b - 1]['index']))
    msg('ESCENA 3: el diario recortado a %d lineas y el contador en %d: de antes de los huerfanos'
        % (t_b, contador(D)))
    nodo = levantar('E', D, P2, 3)
    try:
        responde(P2)
        publicar(P2)
        cabeza(P2, lambda r: q(r['mmrSize']) == t_b, 'D, mmrSize %d' % t_b)
        ack_d, d_t = emparejado(P2, t_b, 'D, por encima de la de A')
        aus = camino_consumo(P2, CONSUMO, a_de[t_b]['seq'], 'ausencia bajo la de A')
        pre = camino_consumo(P2, CONSUMO, d_t['seq'], 'presencia bajo la de D')
    finally:
        parar(nodo)
    exigir(q(d_t['mmrSize']) > t_b and embebido(d_t) <= embebido(a_de[t_b]),
           'la nueva (mmrSize %s) firma en la hoja %d, por debajo de la %d de la vieja (mmrSize %d)'
           % (q(d_t['mmrSize']), embebido(d_t), embebido(a_de[t_b]), t_b))
    ext = {'v': 1, 'tipo': 'extension', 'vieja': a_de[t_b], 'nueva': d_t, 'camino': ack_d['camino']}
    mando('paquete', 'rechazo-ext-reloj-retrocede.json', ext, 1, 'extension: el indice embebido no crece')
    mando('consumo', 'rechazo-cons-reloj-retrocede.json',
          dict(ext, tipo='consumo', consumo=CONSUMO, ausencia=aus, presencia=pre), 1,
          'extension: el indice embebido no crece')

    # ── ESCENA 3, bajo un testigo VIVO (§712): el reloj sobre una historia
    #    CONSISTENTE. X repite las cabezas de A desde el mmrSize t_b -su diario de entonces y el
    #    contador en la hoja de A-; D3 es el operador restaurado de antes de los huerfanos, sin
    #    trafico: el mismo contenido en cada mmrSize, firmado por debajo. ──
    X, D3 = DIR + '/x', DIR + '/d3'
    recortar(A, X, t_b, embebido(a_de[t_b]))
    recortar(A, D3, t_b, q(la[t_b - 1]['index']))
    w3, d3w = testigo('testigo-reloj', P2, 0)
    def custodiada(ls):
        # La pareja del testigo: la cabeza de su ultima linea `anclando` o `extiende`.
        hs = [v for v in ls if v.get('signature')
              and v.get('consistencia', {}).get('clase') in ('anclando', 'extiende')]
        return hs[-1] if hs else None
    nodo = levantar('E', X, P2, 1)
    try:
        for _ in range(240):
            if custodiada(diario_de(d3w)) is not None:
                break
            time.sleep(0.25)
        else:
            rojo('el testigo no anclo en la copia de A')
        time.sleep(2)
    finally:
        parar(nodo)
    vc = custodiada(diario_de(d3w))
    msg('ESCENA 3, testigo vivo: custodia el mmrSize %d en la hoja %d (la de A: %s); llega D3, con el '
        'contador en %d' % (q(vc['mmrSize']), embebido(vc),
                            embebido(a_de[q(vc['mmrSize'])]) == embebido(vc), contador(D3)))
    nodo = levantar('E', D3, P2, 1)
    try:
        r3 = esperar(w3, 60)
    finally:
        parar(nodo)
    if w3.poll() is None:
        w3.kill(); w3.wait()
    l3 = diario_de(d3w)
    ultima = l3[-1] if l3 else {}
    cons = ultima.get('consistencia', {})
    exigir(r3 not in (None, 0) and cons.get('clase') == 'no-extiende' and ultima.get('signature')
           and cons.get('deT') == vc['mmrSize'] and q(cons.get('aT', '0x0')) > q(vc['mmrSize'])
           and ultima.get('publicKey') == vc['publicKey'] and embebido(ultima) <= embebido(vc),
           'el testigo vivo SE DETIENE con no-extiende del mmrSize %s al %s: la hoja %s no esta por '
           'encima de la %d custodiada (exit %s)' % (cons.get('deT'), cons.get('aT'),
                                                   embebido(ultima) if ultima.get('signature') else '?',
                                                   embebido(vc), r3),
           open(d3w + '/salida.txt').read()[-800:] + '\nclases: ' + clases(l3))
    exigir(not any(v.get('consistencia', {}).get('clase') == 'extiende' and v.get('signature')
                   and embebido(v) < embebido(vc) for v in l3),
           'ninguna cabeza de D3 por debajo de la custodiada se anoto `extiende` (la que se cofirma)',
           'clases: ' + clases(l3))
    p = subprocess.run([CLI, 'witness', '--auditar', d3w + '/diario.jsonl', '--no-color'],
                       capture_output=True, text=True)
    exigir(p.returncode != 0 and 'indice-retrocede' in p.stdout,
           '--auditar del diario del testigo ve el reloj que retrocede SIN el nodo', (p.stdout + p.stderr)[-800:])

    # ── los dos sentidos de la vista dividida del ancla, como sobre de extension ──
    vista = json.load(open('spec/vectors/ancla/vista-dividida.json'))
    for nombre, (v, n) in (('ab', (vista['cabeza'], vista['contraria'])),
                           ('ba', (vista['contraria'], vista['cabeza']))):
        mando('paquete', 'rechazo-ext-vista-dividida-%s.json' % nombre,
              {'v': 1, 'tipo': 'extension', 'vieja': v, 'nueva': n, 'camino': []}, 1,
              'extension: mismo mmrSize y digest distinto')

    # ── ESCENA 4 · la recuperacion del §690 bajo un testigo vivo: rota y sigue ──
    def huella(k):
        p = subprocess.run([NODO, '--huella-de-clave-fichero', CLAVES[k]], capture_output=True, text=True)
        if p.returncode != 0:
            rojo('la huella de %s no sale: %s' % (k, p.stderr.strip()))
        return p.stdout.strip().splitlines()[-1]
    HB, HC = huella('B'), huella('C')
    R = DIR + '/r'
    w4, d4 = testigo('testigo-690', P2, 45)
    nodo = levantar('A', R, P2, 1, '--siguiente', HB)
    try:
        h_690 = cabeza(P2, lambda r: q(r['mmrSize']) >= 4, 'A del §690, mmrSize >= 4')
        time.sleep(1)
    finally:
        parar(nodo)
    minimo = contador(R) + 20
    msg('ESCENA 4: A parada con el contador en %d; B entra con --desde-minimo %d' % (contador(R), minimo))
    nodo = levantar('B', R, P2, 1, '--siguiente', HC, '--clave-anterior-fichero', CLAVES['A'],
                    '--desde-minimo', str(minimo))
    try:
        cabeza(P2, lambda r: r['publicKey'] != h_690['publicKey'] and embebido(r) > minimo,
               'B por encima del minimo')
        r4 = esperar(w4, 90)
    finally:
        parar(nodo)
    l4 = diario_de(d4)
    exigir(r4 == 0 and any(v.get('clase') == 'rotada' for v in l4),
           'el testigo vivo ve la rotacion del §690 y NO se detiene (exit %s)' % r4,
           open(d4 + '/salida.txt').read()[-800:] + '\nclases: ' + clases(l4))
    p = subprocess.run([CLI, 'witness', '--auditar', d4 + '/diario.jsonl', '--no-color'],
                       capture_output=True, text=True)
    exigir(p.returncode == 0 and 'sin hallazgos' in p.stdout,
           '--auditar del diario de la recuperacion: sin hallazgos', (p.stdout + p.stderr)[-800:])

    # ── ESCENA 5 · §714: el reinicio, CON y SIN diario, bajo un testigo VIVO ──
    F = DIR + '/f'
    w5, d5w = testigo('testigo-reinicio', P, 0)
    todas = {}
    nodo = levantar('E', F, P, 1)
    try:
        mirar(P, 3, todas, 'F, mmrSize >= 3')
        publicar(P, consumo_n(1))
        mirar(P, 6, todas, 'F, tras el primer consumo')
    finally:
        parar(nodo)
    nodo = levantar('E', F, P, 1)
    try:
        responde(P)
        publicar(P, consumo_n(2))
        mirar(P, 9, todas, 'F tras el reinicio con SIGTERM')
        time.sleep(1)
    finally:
        matar_ya(nodo)
    nodo = levantar('E', F, P, 1)
    try:
        responde(P)
        publicar(P, consumo_n(3))
        mirar(P, 12, todas, 'F tras el reinicio con SIGKILL')
        time.sleep(2)
    finally:
        parar(nodo)
    l5 = diario_de(d5w)
    por_t = {}
    for v in l5:
        if v.get('signature') and v.get('mmrSize'):
            por_t.setdefault(q(v['mmrSize']), set()).add(v['epochDigest'])
    repetidos = sorted(t for t, ds in todas.items() if len(ds) > 1)
    # La cabeza de mmrSize t es la linea t del diario del nodo: la hoja t del MMR (§292).
    dn = [json.loads(l) for l in lineas(F + '/diario.jsonl')]
    casan = all(t < len(dn) and ds == {dn[t]['epochDigest']} for t, ds in todas.items())
    exigir(w5.poll() is None and not repetidos and casan and all(len(ds) == 1 for ds in por_t.values()),
           'CON su diario y con trafico, dos reinicios (SIGTERM y SIGKILL): el testigo no se detiene; '
           'el nodo sirvio %d mmrSize, del 0 al %d, cada uno con UN digest, el de la linea t de su '
           'diario (%d lineas); el testigo vio %d' % (len(todas), max(todas), len(dn), len(por_t)),
           'repetidos: %s, casan: %s\n%s\nclases: %s'
           % (repetidos, casan, open(d5w + '/salida.txt').read()[-800:], clases(l5)))
    t_max = max(por_t)
    os.replace(F + '/diario.jsonl', F + '/diario-perdido.jsonl')
    msg('ESCENA 5: el diario se pierde; el libro, el registro y el contador, no. El testigo vio hasta '
        'el mmrSize %d' % t_max)
    nodo = levantar('E', F, P, 1)
    try:
        r5 = esperar(w5, 40)
    finally:
        parar(nodo)
    if w5.poll() is None:
        w5.kill(); w5.wait()
    l5 = diario_de(d5w)
    ultima = l5[-1] if l5 else {}
    cons = ultima.get('consistencia', {})
    t5 = q(cons.get('t', '0x0')) if cons.get('t') else None
    sal5 = open(d5w + '/salida.txt').read()
    exigir(r5 not in (None, 0) and cons.get('clase') == 'bifurcacion' and t5 is not None
           and t5 <= t_max and cons.get('digestPrevio') in por_t.get(t5, ())
           and ultima.get('epochDigest') != cons.get('digestPrevio')
           and 'bifurcacion o reinicio sin diario' in sal5,
           'SIN su diario: el testigo vivo SE DETIENE con bifurcacion en el mmrSize %s, que ya habia '
           'visto -el mayor que vio, %d-, sin esperar a que el nodo pase de lo custodiado (exit %s)'
           % (t5, t_max, r5), sal5[-800:] + '\nclases: ' + clases(l5))
    p = subprocess.run([CLI, 'witness', '--auditar', d5w + '/diario.jsonl', '--sobres', d5w + '/sobres',
                        '--no-color'], capture_output=True, text=True)
    sobres5 = sorted(glob.glob(d5w + '/sobres/bifurcacion-*.json'))
    exigir(p.returncode != 0 and 'bifurcacion o reinicio sin diario' in p.stdout and len(sobres5) >= 1,
           '--auditar --sobres del diario del testigo la ve SIN el nodo y arma %d sobre(s) del modo 5'
           % len(sobres5), (p.stdout + p.stderr)[-800:])
    veredictos = [subprocess.run([MANDO, x], capture_output=True, text=True) for x in sobres5]
    exigir(sobres5 and all(v.returncode == 0 and 'VERDE: BIFURCACION' in v.stdout for v in veredictos),
           'el mando los juzga sin el nodo ni el testigo: VERDE, BIFURCACION, los %d' % len(sobres5),
           '\n'.join((v.stdout + v.stderr)[-300:] for v in veredictos))
finally:
    for p in procesos:
        if p.poll() is None:
            p.kill(); p.wait()
if GUARDAR:
    for f, ls in entradas.items():
        open('%s/%s/entradas.txt' % (GUARDAR, f), 'w').write('\n'.join(ls) + '\n')
msg('%d fallo(s)' % fallos)
sys.exit(1 if fallos else 0)
PY
RC=$?
[ "$RC" = 0 ] || fallo "algun sobre o algun testigo no dijo lo que debia"
[ -z "$(git status --porcelain)" ] || fallo "el banco dejo el arbol sucio: no debe tocarlo"
[ -n "$GUARDAR" ] && msg "sobres guardados en $GUARDAR (de ahi sale el catalogo, COPIADO)"
msg "BANCO-BIFURCACION VERDE"
