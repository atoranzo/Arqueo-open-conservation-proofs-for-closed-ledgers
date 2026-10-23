#!/usr/bin/env bash
# tools/banco_historia_comun.sh -- el banco de la HISTORIA COMUN: dos operadores de acuerdo, ¿pueden
# fabricarse una historia comun que refute un conflicto genuino? (spec/PAQUETE.md 2.5 y 2.14).
#
# DOS nodos REALES, sin una linea de codigo tocada. Los dos operadores lo preparan de antemano:
#   - A arranca con su genesis comprometiendo a la clave de B (`--siguiente`): la pre-rotacion.
#   - A acepta un consumo C y firma la cabeza que lo lleva.
#   - B COPIA el diario de A -cuyas lineas son digests de cabeza, publicos- y su contador, y arranca
#     con su clave sobre esa copia, con un libro VACIO y `--desde-minimo` alto: el nodo hace una rotacion de
#     verdad, firma su acta con la procedencia que lee de la copia y siembra su MMR con las hojas de
#     A. Su contador salta por encima de todo lo que A firmara en la vida del banco.
#   - A SIGUE: vuelve a arrancar sobre su propio directorio y sigue latiendo con su clave.
#   - B acepta el MISMO consumo C, que su libro vacio no conoce.
# Dos libros distintos -dos estados, dos arboles de consumos- aceptaron C. El sobre de conflicto lo
# exhibe; la refutacion es el sobre de extension entre las dos cabezas, con la cadena de B.
#
# Lo que mide, cada sobre armado de lo que sirvio el cable y juzgado por el mando con los nodos
# parados:
#   1. el CONFLICTO, VERDE: el mismo consumo bajo las dos raices, de dos claves.
#   2. la REFUTACION, VERDE: la extension de la cabeza de A a la de B con las actas -la cadena vale,
#      el MMR de B contiene al de A y el reloj cruza las claves por sus tramos-. Es el agujero: el
#      sobre no distingue a dos operadores de acuerdo de uno que roto. Sigue VERDE despues del
#      arreglo, y se declara: la historia de cabezas no ata el ESTADO.
#   3. el SOLAPAMIENTO por el RELEVO, VERDE desde el arreglo: una cabeza de A firmada DESPUES de la
#      copia lleva un mmrSize que el acta de B ya daba a B. Antes del arreglo el sobre lo daba por
#      bueno -la hoja de A esta en su tramo, muy por debajo del desde de B- y la refutacion quedaba
#      sin respuesta. Con la regla, cualquier cliente de A que guarde una cabeza posterior la delata,
#      si A siguio firmando con su clave. Si los dos libros se ALTERNAN -A, B, A, B, cada clave en sus
#      turnos- no queda cabeza que delatar: es residuo declarado (spec/PAQUETE.md 2.5).
#   3 bis. la BIFURCACION del modo 5 del ancla, VERDE antes y despues: una cabeza de A y una de B en
#      el MISMO mmrSize, con la cadena. Es lo unico que ya delataba, y pide las DOS cabezas.
#   4. la EXTENSION de esa cabeza de A a una posterior de B: ROJO por el relevo, antes de la
#      consistencia.
#   5. los dos negativos del solapamiento: la cabeza de A del conflicto y la de B, dentro de su tramo
#      y de su relevo.
#
# FUERA del canon: levanta procesos y espera latidos. NO ESCRIBE EN EL ARBOL: todo vive en un
# temporal bajo $HOME, que borra al salir; lo comprueba al final por DELTA de `git status`.
#
#   bash tools/banco_historia_comun.sh [--guardar <dir>]
#
# --guardar  copia los sobres a <dir>, con `entradas.txt` -fichero|codigo|texto, las lineas del
#            manifiesto-: de ahi sale el catalogo, COPIADO.
set -u
msg(){ echo "BANCO-HISTORIA| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
GUARDAR=""
while [ $# -gt 0 ]; do
  case "$1" in
    --guardar) GUARDAR="${2:?--guardar exige un directorio}"; mkdir -p "$GUARDAR"; shift ;;
    *) fallo "uso: bash tools/banco_historia_comun.sh [--guardar <dir>]" ;;
  esac
  shift
done
RAIZ="$(cd "$(dirname "$0")/.." && pwd)"
cd "$RAIZ" || fallo "no se llega a la raiz del arbol"
DIR=$(mktemp -d "$HOME/.banco_historia.XXXXXX")
trap 'rm -rf "$DIR"' EXIT INT TERM HUP QUIT
git status --porcelain | sort > "$DIR/porcelain_pre.txt"
msg "compilando nodo y verificador en RELEASE (aqui se firma de verdad)"
cargo build --locked --release -q -p zk-ssl-node -p zk-ssl-verify 2>/dev/null \
  || cargo build --locked --release -p zk-ssl-node -p zk-ssl-verify || fallo "no compila"
T=$(bash tools/target_dir.sh) || fallo "no se donde deja cargo los binarios (tools/target_dir.sh)"

python3 - "$DIR" "${GUARDAR:-}" "$T" <<'PY'
import copy, json, os, shutil, subprocess, sys, time, urllib.request
DIR, GUARDAR, T = sys.argv[1], sys.argv[2], sys.argv[3]
NODO, MANDO = T + '/release/zk-ssl-node', T + '/release/zk-ssl-verify'
PA, PB = 8861, 8862
DESDE_MINIMO = 1000000
def msg(m): print('BANCO-HISTORIA| ' + m, file=sys.stderr, flush=True)
def rojo(m):
    msg('ROJO: ' + m)
    for k in 'AB':
        ruta = '%s/nodo-%s.err' % (DIR, k)
        if os.path.exists(ruta):
            msg('  el nodo %s dijo: %s' % (k, open(ruta).read().strip()[-800:]))
    sys.exit(1)
# Tres claves de banco: A, B y la C que B compromete. Semillas '61' a '63' x 96, de nadie mas.
CLAVES = {}
for k, s in zip('ABC', ('61', '62', '63')):
    CLAVES[k] = '%s/%s.hex' % (DIR, k)
    open(CLAVES[k], 'w').write(s * 96); os.chmod(CLAVES[k], 0o600)
def huella(k):
    p = subprocess.run([NODO, '--huella-de-clave-fichero', CLAVES[k]], capture_output=True, text=True)
    if p.returncode != 0:
        rojo('la huella de %s no sale: %s' % (k, p.stderr.strip()))
    return p.stdout.strip().splitlines()[-1]
def levantar(k, d, puerto, *banderas):
    os.makedirs(d, exist_ok=True)
    return subprocess.Popen([NODO, '--listen', '127.0.0.1:%d' % puerto, '--latido', '1',
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
def rpc(puerto, m, p):
    c = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': m, 'params': p}).encode()
    r = urllib.request.Request('http://127.0.0.1:%d' % puerto, c, {'Content-Type': 'application/json'})
    return json.loads(urllib.request.urlopen(r, timeout=10).read())
def embebido(c):
    # XMSS^MT 40/8: el indice de hoja son los cinco primeros bytes de la firma, big-endian.
    return int(c['signature'][2:12], 16)
def t(c):
    return int(c['mmrSize'], 16)
def cabeza(puerto, cuando, que):
    for _ in range(240):
        try:
            r = rpc(puerto, 'zkssl_signedEpochHead', {}).get('result', {})
            if r.get('available') and cuando(r):
                return r
        except Exception:
            pass
        time.sleep(0.25)
    rojo('ninguna cabeza firmada: ' + que)
def camino(puerto, old):
    for _ in range(40):
        try:
            r = rpc(puerto, 'zkssl_consistencyProof', {'oldSize': old}).get('result', {})
            if r.get('available'):
                return r
        except Exception:
            pass
        time.sleep(0.25)
    rojo('zkssl_consistencyProof no llego a available:true desde %s' % old)
def emparejado(puerto, old, cuando, que):
    # La pareja firmada es el acumulador ANTES de la cabeza: el camino de tamano t lo firma la
    # cabeza siguiente. Se espera a la que firma exactamente ese t, como en la extension.
    for _ in range(20):
        ack = camino(puerto, old)
        try:
            return ack, cabeza(puerto, lambda r: r['mmrSize'] == ack['mmrSize'] and cuando(r),
                               '%s, mmrSize %s' % (que, ack['mmrSize']))
        except SystemExit:
            pass
    rojo('ninguna cabeza firma el camino servido: ' + que)
def digest_canonico(bytes_):
    # Un byte no nulo en el offset 3 de cada grupo de ocho: canonico en el campo leido en LE y en BE.
    b = bytearray(32)
    for i, v in zip((3, 11, 19, 27), bytes_):
        b[i] = v
    return '0x' + b.hex()
CONSUMO = digest_canonico((0x6C, 0x39, 0xA2, 0x17))
PROPIO_A = digest_canonico((0x2B, 0x84, 0x5E, 0xC1))
PROPIO_B = digest_canonico((0x93, 0x0D, 0x77, 0x4A))
def publica(puerto, consumo, que):
    r = rpc(puerto, 'zkssl_publishConsumo', {'consumo': consumo}).get('result', {})
    if not r.get('accepted'):
        rojo('%s NO acepto el consumo: %s' % (que, r))
    return int(r['logSeq'], 16)
def presencia(puerto, cab, que):
    r = rpc(puerto, 'zkssl_consumoPath', {'consumo': CONSUMO, 'seq': cab['seq']}).get('result', {})
    if not r.get('available'):
        rojo('consumoPath de %s no dio camino: %s' % (que, r))
    return r['camino']
def lineas_con_digest(ruta):
    n = 0
    for l in open(ruta):
        try:
            if isinstance(json.loads(l).get('epochDigest'), str):
                n += 1
        except ValueError:
            pass
    return n
HB, HC = huella('B'), huella('C')
da, db = DIR + '/a', DIR + '/b'

# ── A: su genesis compromete a B. Su consumo propio y el COMPARTIDO; la cabeza que lo lleva.
nodo_a = levantar('A', da, PA, '--siguiente', HB)
try:
    cabeza(PA, lambda r: t(r) >= 2, 'A, mmrSize >= 2')
    publica(PA, PROPIO_A, 'A (el propio)')
    s_a = publica(PA, CONSUMO, 'A (el compartido)')
    h_a = cabeza(PA, lambda r: int(r['seq'], 16) >= s_a and t(r) >= 2, 'A con el consumo')
    pres_a = presencia(PA, h_a, 'A')
    cabeza(PA, lambda r: t(r) >= t(h_a) + 2, 'A, dos latidos despues')
finally:
    parar(nodo_a)
msg('A: la cabeza del conflicto, mmrSize %d, hoja %d' % (t(h_a), embebido(h_a)))
# ── la COPIA: B toma el diario de A tal cual -digests de cabeza y la genesis-, sin el libro.
os.makedirs(db)
shutil.copy(da + '/diario.jsonl', db + '/diario.jsonl')
# Y el contador de A: la cuenta es una por operador (RFC-0015 D-A), y la puerta del contador del nodo
# no arranca por debajo de las hojas que el diario ya anota. Es lo que hace una rotacion de verdad.
shutil.copy(da + '/indice.bin', db + '/indice.bin')
copiadas = lineas_con_digest(db + '/diario.jsonl')
msg('B copia el diario de A: %d cabezas; su libro, vacio' % copiadas)
# ── A SIGUE, sobre lo suyo; B rota sobre la copia, por encima de toda hoja que A firme aqui.
nodo_a = levantar('A', da, PA)
nodo_b = levantar('B', db, PB, '--siguiente', HC, '--desde-minimo', str(DESDE_MINIMO))
try:
    cabeza(PB, lambda r: r['publicKey'] != h_a['publicKey'], 'B en pie')
    publica(PB, PROPIO_B, 'B (el propio)')
    s_b = publica(PB, CONSUMO, 'B (el compartido): su libro vacio no lo conoce')
    ack, h_b = emparejado(PB, h_a['mmrSize'], lambda r: int(r['seq'], 16) >= s_b, 'B con el consumo')
    pres_b = presencia(PB, h_b, 'B')
    actas = rpc(PB, 'zkssl_keyActs', {})['result']['actas']
    if len(actas) != 2 or actas[1]['acta']['procedencia'] is None:
        rojo('la cadena de B no es la genesis de A y su rotacion: %s' % actas)
    relevo = int(actas[1]['acta']['procedencia']['mmrSize'], 16)
    if relevo != copiadas:
        rojo('el relevo del acta (%d) no es el tamano de la copia (%d)' % (relevo, copiadas))
    # ── A, despues del relevo: su cabeza con un mmrSize que el acta ya da a B.
    h_a2 = cabeza(PA, lambda r: t(r) > relevo, 'A por encima del relevo')
    # ── y una de B posterior a esa, con el camino desde su tamano.
    ack2, h_b2 = emparejado(PB, h_a2['mmrSize'], lambda r: True, 'B por encima de la A tardia')
    # ── las dos ramas en el MISMO mmrSize, por delante de las dos: cada nodo pasa por el.
    mismo = max(t(cabeza(PA, lambda r: True, 'A')), t(cabeza(PB, lambda r: True, 'B'))) + 2
    h_a3 = cabeza(PA, lambda r: t(r) == mismo, 'A en el mmrSize %d' % mismo)
    h_b3 = cabeza(PB, lambda r: t(r) == mismo, 'B en el mmrSize %d' % mismo)
finally:
    parar(nodo_a)
    parar(nodo_b)
d_b = int(actas[1]['acta']['desde'], 16)
msg('B: la cabeza del conflicto, mmrSize %d, hoja %d; su acta, desde la hoja %d, relevo en el mmrSize %d'
    % (t(h_b), embebido(h_b), d_b, relevo))
msg('A, tras el relevo: mmrSize %d, hoja %d (por debajo del desde de B)' % (t(h_a2), embebido(h_a2)))
if not (embebido(h_a) < d_b < embebido(h_b) and embebido(h_a2) < d_b and t(h_a) < relevo <= t(h_b)):
    rojo('la escena no es la que el banco quiere medir')
if h_a['consRoot'] == h_b['consRoot']:
    rojo('los dos consRoot coinciden: no son dos libros')

fallos = 0
entradas = []
def mando(nombre, sobre, codigo, texto):
    global fallos
    ruta = '%s/%s.json' % (DIR, nombre)
    json.dump(sobre, open(ruta, 'w'), indent=1, sort_keys=True)
    p = subprocess.run([MANDO, ruta], capture_output=True, text=True)
    out = p.stdout + p.stderr
    ok = p.returncode == codigo and texto in out
    fallos += 0 if ok else 1
    msg('%s %-40s exit %d (se esperaba %d: %s)' % ('OK  ' if ok else 'ROJO', nombre, p.returncode, codigo, texto))
    if not ok:
        msg('     ' + out.strip().replace('\n', '\n     '))
    entradas.append('%s.json|%d|%s' % (nombre, codigo, texto))
    if GUARDAR:
        shutil.copy(ruta, GUARDAR)
def extension(vieja, nueva, a):
    return {'v': 1, 'tipo': 'extension', 'vieja': vieja, 'nueva': nueva, 'camino': a['camino'],
            'actas': actas}
def solap(cab):
    return {'v': 1, 'tipo': 'solapamiento', 'cabeza': cab, 'actas': actas}
# 1. el conflicto genuino: dos libros, dos claves, el mismo consumo.
mando('historia-comun-conflicto',
      {'v': 1, 'tipo': 'conflicto', 'consumo': CONSUMO,
       'libros': [{'cabeza': h_a, 'presencia': pres_a}, {'cabeza': h_b, 'presencia': pres_b}]},
      0, 'VERDE: dos libros aceptaron el mismo consumo')
# 2. la refutacion fabricada: VERDE, antes y despues del arreglo. Es el residuo que se declara.
mando('historia-comun-refutacion', extension(h_a, h_b, ack), 0,
      '(1 eslabon(es), la posterior desde la hoja %d): el mismo OPERADOR en los dos extremos' % d_b)
# 3. A siguio: su cabeza tras el relevo, con la cadena de la refutacion, la delata.
mando('historia-comun-solapamiento-tras-el-relevo', solap(h_a2), 0,
      'VERDE: SOLAPAMIENTO - la clave firmo una cabeza con el mmrSize %d' % t(h_a2))
# 3 bis. lo que ya delataba antes del arreglo: las dos ramas en el MISMO mmrSize, con la cadena, son
#    la bifurcacion del modo 5 del ancla. Pero hace falta tener las DOS cabezas, una de cada nodo.
mando('historia-comun-bifurcacion', {'v': 1, 'tipo': 'ancla', 'cabeza': h_a3, 'rama': h_b3, 'actas': actas},
      0, 'VERDE: BIFURCACION - el operador firmo DOS cabezas distintas con el mmrSize %d' % mismo)
# 4. la extension de esa cabeza de A a una posterior de B: el relevo, antes de la consistencia.
mando('neg-historia-comun-la-vieja-tras-el-relevo', extension(h_a2, h_b2, ack2), 1,
      'RELEVO: la cabeza de la clave que se va lleva el mmrSize %d, y el acta 1 la releva en el %d'
      % (t(h_a2), relevo))
# 5. los dos negativos del solapamiento: dentro de su tramo y de su relevo.
mando('neg-historia-comun-solap-a-en-su-relevo', solap(h_a), 1,
      'la cabeza firma en la hoja %d, dentro de su tramo (acta 0, la genesis: cualquier hoja salvo la %d '
      'de su acta, por debajo de la %d): no hay solapamiento'
      % (embebido(h_a), int(actas[0]['acta']['desde'], 16), d_b))
mando('neg-historia-comun-solap-b-en-su-relevo', solap(h_b), 1,
      'la cabeza firma en la hoja %d, dentro de su tramo (acta 1: por encima de la hoja %d): no hay solapamiento'
      % (embebido(h_b), d_b))
if GUARDAR:
    open(GUARDAR + '/entradas.txt', 'w').write('\n'.join(entradas) + '\n')
    json.dump({'h_a': h_a, 'h_b': h_b, 'h_a2': h_a2, 'actas': actas, 'camino': ack['camino'],
               'relevo': relevo}, open(GUARDAR + '/escena.json', 'w'), indent=1, sort_keys=True)
sys.exit(1 if fallos else 0)
PY
RC=$?
git status --porcelain | sort > "$DIR/porcelain_post.txt"
cmp -s "$DIR/porcelain_pre.txt" "$DIR/porcelain_post.txt" || fallo "el banco toco el arbol: no debe"
[ "$RC" = 0 ] || fallo "algun sobre no dijo lo que debia"
[ -n "$GUARDAR" ] && msg "sobres guardados en $GUARDAR (de ahi sale el catalogo, COPIADO)"
msg "BANCO-HISTORIA VERDE"
