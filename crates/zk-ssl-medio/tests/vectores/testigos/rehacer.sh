#!/usr/bin/env bash
# crates/zk-ssl-medio/tests/vectores/testigos/rehacer.sh -- el publicador del medio contra
# testigos LOCALES (RFC-0013 E3a, §808), de la mano del autor de las especificaciones:
#
#   - torchwood-a y torchwood-b: el add-checkpoint de litewitness (filippo.io/torchwood v0.10.0,
#     internal/witness) con una clave ML-DSA-44 cada uno (`testigo-local mldsa`);
#   - litewitness: litewitness v0.10.0 SIN TOCAR, con su Ed25519 en un agente SSH en memoria
#     (`testigo-local agente`), registrado con witnessctl como los otros dos.
#
# El cliente es `zk_ssl_medio::testigos` (el ejemplo `testigos_locales`). Nueve pasos: el origin
# que no conocen, la primera cofirma, el publicador sin memoria, el crecimiento, el atrasado, el
# directorio restaurado (la 103), la bifurcacion, la misma talla con otra raiz y la nota de otra
# clave. Y de punta a punta: la nota cofirmada por torchwood-a y torchwood-b entra en el sobre
# `ancla-cofirmada` de `spec/vectors/` y el kit la da VERDE; el juez del cli la ACREDITA con k = 2
# y no con k = 3; y la linea Ed25519 de litewitness, que el cliente ignora, verifica en Go.
#
# ⚠️ FUERA del canon y FUERA de tools/bancos.sh: pide Go (GO=..., si no esta en el PATH) y red
# para bajar torchwood y sus modulos por el proxy de Go, y la cadena de Go 1.27 que su go.mod
# pide. NO TOCA testigos reales: todo escucha en 127.0.0.1. Vive en un temporal bajo $HOME que
# borra al salir, procesos incluidos, y comprueba al final que no ha tocado el arbol.
#
#   bash crates/zk-ssl-medio/tests/vectores/testigos/rehacer.sh [--guardar]
#
# --guardar  reescribe torchwood.json en este directorio: lo que relee tests/testigos_torchwood.rs.
set -u
msg(){ echo "TESTIGOS-LOCALES| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
GUARDAR=""
case "${1:-}" in
  --guardar) GUARDAR=1 ;;
  "") ;;
  *) fallo "uso: bash crates/zk-ssl-medio/tests/vectores/testigos/rehacer.sh [--guardar]" ;;
esac
cd "$(git rev-parse --show-toplevel)" || fallo "fuera de un arbol de git"
AQUI=crates/zk-ssl-medio/tests/vectores/testigos
# Lo que hay en el arbol, con lo que no esta commiteado: se compara al final.
huella(){ { git status --porcelain=v1 -uall | grep -v " $AQUI/torchwood.json\$"; \
            git diff -- . ":!$AQUI/torchwood.json"; \
            git ls-files -o --exclude-standard -z | grep -zv "^$AQUI/torchwood.json\$" | xargs -0 -r sha256sum; } | sha256sum; }
ANTES=$(huella)
GO="${GO:-$(command -v go || echo /usr/local/go/bin/go)}"
[ -x "$GO" ] || fallo "sin Go: GO=/ruta/al/go"
# Los binarios de Rust, donde los deja cargo: lo dice tools/target_dir.sh (§855).
T=$(bash tools/target_dir.sh) || fallo "no se donde deja cargo los binarios (tools/target_dir.sh)"
DIR=$(mktemp -d "$HOME/.testigos_locales.XXXXXX")
trap 'chmod -R u+w "$DIR" 2>/dev/null; rm -rf "$DIR"' EXIT INT TERM HUP QUIT
# Los modulos y la cadena de Go, en el temporal salvo que quien llama exporte los suyos.
export GOMODCACHE="${GOMODCACHE:-$DIR/go/mod}" GOCACHE="${GOCACHE:-$DIR/go/cache}" GOPATH="$DIR/go/path"
export GOFLAGS="-mod=mod -modcacherw" GOTOOLCHAIN=auto

msg "bajando filippo.io/torchwood v0.10.0 (comprobado contra sum.golang.org)"
ORIGEN_TW=$(cd "$DIR" && "$GO" mod download -json filippo.io/torchwood@v0.10.0 \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["Dir"])') || fallo "no baja torchwood"
cp -r "$ORIGEN_TW" "$DIR/torchwood" && chmod -R u+w "$DIR/torchwood"
mkdir -p "$DIR/torchwood/cmd/testigo-local" "$DIR/bin"
cp "$AQUI/testigo-local/main.go" "$DIR/torchwood/cmd/testigo-local/"
msg "compilando litewitness, witnessctl y testigo-local con la cadena de Go que pide torchwood"
(cd "$DIR/torchwood" && "$GO" build -o "$DIR/bin/" ./cmd/litewitness ./cmd/witnessctl ./cmd/testigo-local) \
  || fallo "Go no compila"
VERSION_GO=$(cd "$DIR/torchwood" && "$GO" version)
msg "$VERSION_GO"
msg "compilando el kit, el cli y el cliente del medio en RELEASE"
{ cargo build --locked --release -q -p zk-ssl-verify -p zk-ssl-cli \
    && cargo build --locked --release -q -p zk-ssl-medio --example testigos_locales; } \
  || fallo "no compila"

python3 - "$DIR" "$AQUI" "${GUARDAR:-}" "$VERSION_GO" "$T" <<'PY'
import atexit, hashlib, json, os, re, socket, subprocess, sys, time
DIR, AQUI, GUARDAR, VERSION_GO, T = sys.argv[1:6]
BIN = DIR + '/bin/'
# ⚠️ §855: las tres decian 'target/release/', sin mirar donde deja cargo los binarios
MANDO, JUEZ = T + '/release/zk-ssl-verify', T + '/release/zk-ssl-cli'
CLIENTE = T + '/release/examples/testigos_locales'
CAT = 'spec/vectors/ancla-cofirmada/'
def msg(m): print('TESTIGOS-LOCALES| ' + m, file=sys.stderr, flush=True)
fallos = 0
def comprobar(ok, que):
    global fallos
    fallos += 0 if ok else 1
    msg('%s %s' % ('OK  ' if ok else 'ROJO', que))
    return ok
def semilla(desde):
    return ''.join('%02x' % ((desde + i) & 0xff) for i in range(32))
procesos = []
atexit.register(lambda: [p.kill() for p in procesos])
def lanzar(args, err):
    p = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=open(DIR + '/' + err, 'w'), text=True)
    procesos.append(p)
    return p
def derivar(cabeza):
    # El productor del ancla es el kit (RFC-0012, D-A): se le da la cabeza y se lee lo que imprime.
    ruta = DIR + '/derivar.json'
    json.dump({'v': 1, 'tipo': 'ancla', 'cabeza': cabeza}, open(ruta, 'w'))
    p = subprocess.run([MANDO, ruta], capture_output=True, text=True)
    assert p.returncode == 0, p.stdout + p.stderr
    clave = huella = None
    for linea in p.stdout.splitlines():
        if linea.strip().startswith('{'):
            clave = json.loads(linea)['clave']
        if 'la huella del ancla:' in linea:
            huella = linea.split('la huella del ancla:')[1].split()[0]
    assert clave and huella, p.stdout
    return clave, huella
def sintetica(i):
    return '0x' + hashlib.sha256(i.to_bytes(8, 'big')).hexdigest()

# Las tres anclas del catalogo, derivadas de sus tres cabezas firmadas, y detras anclas sinteticas.
cat = {n: json.load(open(CAT + n + '.json'))
       for n in ('cofirmada-medio-de-una', 'cofirmada-un-testigo', 'cofirmada-ultima', 'cofirmada-dos-testigos')}
(clave, a1), (_, a2), (_, a3) = (derivar(cat[n]['cabeza']) for n in
                                 ('cofirmada-medio-de-una', 'cofirmada-un-testigo', 'cofirmada-ultima'))
ORIGEN = 'zkssl/v1/' + clave[2:]
def hojas(n, reescrita=None):
    h = [a1, a2, a3] + [sintetica(i) for i in range(3, n)]
    if reescrita is not None:
        h[reescrita] = sintetica(reescrita + 1000)
    return h[:n]

# Los testigos.
testigos = {}
for nombre, desde, db in (('testigo.invalid/torchwood-a', 0x20, 'a.db'), ('testigo.invalid/torchwood-b', 0x40, 'b.db')):
    p = lanzar([BIN + 'testigo-local', 'mldsa', '-name', nombre, '-seed', semilla(desde),
                '-db', DIR + '/' + db, '-listen', '127.0.0.1:0'], db + '.err')
    vkey, dire = p.stdout.readline().strip(), p.stdout.readline().strip()
    testigos[nombre] = {'vkey': vkey, 'prefijo': 'http://' + dire, 'db': DIR + '/' + db}
ag = lanzar([BIN + 'testigo-local', 'agente', '-seed', semilla(0x60), '-sock', DIR + '/agente.sock'], 'agente.err')
huella_ssh = ag.stdout.readline().strip()
s = socket.socket(); s.bind(('127.0.0.1', 0)); puerto = s.getsockname()[1]; s.close()
LW = 'testigo.invalid/litewitness'
lw = lanzar([BIN + 'litewitness', '-name', LW, '-db', DIR + '/l.db', '-ssh-agent', DIR + '/agente.sock',
             '-key', huella_ssh, '-listen', '127.0.0.1:%d' % puerto], 'litewitness.err')
vkey_lw = None
for _ in range(100):
    m = re.search(r'msg="verifier key" vkey="?([^"\s]+)', open(DIR + '/litewitness.err').read())
    if m:
        vkey_lw = m.group(1)
        try:
            socket.create_connection(('127.0.0.1', puerto), timeout=1).close()
            break
        except OSError:
            pass
    time.sleep(0.1)
assert vkey_lw, open(DIR + '/litewitness.err').read()
testigos[LW] = {'nombre': LW, 'semilla': '0x' + '7a' * 32, 'prefijo': 'http://127.0.0.1:%d' % puerto,
                'db': DIR + '/l.db', 'vkey_ed25519': vkey_lw}
msg('torchwood-a, torchwood-b y litewitness escuchan en 127.0.0.1 (litewitness: %s)' % vkey_lw.split('+')[0])

PUB = {'semilla': '0x' + '01' * 32, 'marca': 1790000000}
IMPOSTOR = {'semilla': '0x' + '04' * 32, 'marca': 1790000000}
pasos = []
def paso(nombre, h, viejo, esperado, publicador=PUB):
    e = {'origen': ORIGEN, 'hojas': h, 'publicador': publicador, 'testigos': []}
    for t in testigos.values():
        e['testigos'].append(dict({k: v for k, v in t.items() if k in ('vkey', 'nombre', 'semilla', 'prefijo')},
                                  viejo=viejo))
    p = subprocess.run([CLIENTE], input=json.dumps(e), capture_output=True, text=True)
    assert p.returncode == 0, p.stderr
    s = json.loads(p.stdout)
    for t, visto, (resultado, campos) in zip(testigos, s['testigos'], esperado):
        ok = visto['resultado'] == resultado and all(visto.get(k) == v for k, v in campos.items())
        dicho = {k: visto[k] for k in ('viejo', 'ignoradas', 'recordado', 'respuesta', 'error') if k in visto}
        comprobar(ok, '%-12s %-14s %-15s %s, %d intercambio(s)'
                  % (nombre, t.split('/')[1], visto['resultado'], dicho, len(visto['intercambios'])))
    pasos.append({'paso': nombre, 'entrada': e, 'salida': s})
    return s

paso('desconocido', hojas(3), 0, [('rechazada', {'respuesta': 'OrigenDesconocido'})] * 3)
publicador = pasos[0]['salida']['publicador']
for t in testigos.values():
    for orden in (['add-log', '-db', t['db'], '-origin', ORIGEN],
                  ['add-key', '-db', t['db'], '-origin', ORIGEN, '-key', publicador]):
        subprocess.run([BIN + 'witnessctl'] + orden, check=True, capture_output=True)
msg('el medio %s, registrado en los tres con witnessctl' % ORIGEN)
cof = lambda viejo, ign=0: ('cofirmada', {'viejo': viejo, 'ignoradas': ign})
sin = lambda viejo: ('sin-su-cofirma', {'viejo': viejo, 'ignoradas': 1})
primera = paso('primera', hojas(3), 0, [cof(0), cof(0), sin(0)])
paso('sin-memoria', hojas(3), 0, [cof(3), cof(3), sin(3)])
paso('crece', hojas(7), 3, [cof(3), cof(3), sin(3)])
paso('atrasado', hojas(9), 0, [cof(7), cof(7), sin(7)])
paso('retroceso', hojas(5), 0, [('retroceso', {'recordado': 9, 'nota': 5})] * 3)
paso('bifurcado', hojas(11, 4), 9, [('rechazada', {'respuesta': 'Inconsistente'})] * 3)
paso('otra-raiz', hojas(9, 4), 9, [('rechazada', {'respuesta': 'Inconsistente'})] * 3)
paso('impostor', hojas(11), 9, [('rechazada', {'respuesta': 'Prohibida'})] * 3, IMPOSTOR)
for nombre, t in testigos.items():
    r = subprocess.run([BIN + 'witnessctl', 'list-logs', '-db', t['db']], capture_output=True, text=True)
    logs = [json.loads(l) for l in r.stdout.splitlines() if l.strip()]
    comprobar([(l['origin'], l['size']) for l in logs] == [(ORIGEN, 9)],
              '%s recuerda, tras los nueve pasos, %s' % (nombre.split('/')[1],
                                                         [(l['size'], l['root_hash']) for l in logs]))

# De punta a punta: la nota cofirmada por los dos torchwood, en el sobre del catalogo.
nc = primera['nota_cofirmada']
comprobar(nc.split('\n')[2] == cat['cofirmada-dos-testigos']['nota'].split('\n')[2],
          'la raiz de las tres anclas derivadas es la de la nota del catalogo')
comprobar(publicador == cat['cofirmada-dos-testigos']['publicador'], 'el publicador es el del catalogo (semilla 01)')
vk = [testigos['testigo.invalid/torchwood-a']['vkey'], testigos['testigo.invalid/torchwood-b']['vkey']]
sobre = dict(cat['cofirmada-dos-testigos'], nota=nc, publicador=publicador, testigos=vk)
ruta = DIR + '/sobre.json'
json.dump(sobre, open(ruta, 'w'), indent=1, sort_keys=True)
p = subprocess.run([MANDO, ruta], capture_output=True, text=True)
comprobar(p.returncode == 0 and 'la cofirman 2 testigo(s)' in p.stdout,
          'el kit da VERDE el sobre con la nota cofirmada por torchwood-a y torchwood-b')
open(DIR + '/politica.vkeys', 'w').write('\n'.join(vk) + '\n')
for k, codigo in (('2', 0), ('3', 1)):
    p = subprocess.run([JUEZ, 'witness', '--ancla-cofirmada', ruta, '--testigos-medio', DIR + '/politica.vkeys',
                        '--k-medio', k], capture_output=True, text=True)
    comprobar(p.returncode == codigo and (codigo == 1 or 'ACREDITADO el checkpoint' in p.stdout),
              'el juez del cli con k = %s sale con %d' % (k, p.returncode))
lw_200 = primera['testigos'][2]['intercambios'][0]
p = subprocess.run([BIN + 'testigo-local', 'comprobar', '-vkey', vkey_lw], input=primera['nota'] + lw_200['respuesta'],
                   capture_output=True, text=True)
comprobar(p.returncode == 0 and p.stdout.startswith('VERIFICA ' + LW),
          'la linea Ed25519 de litewitness que el cliente ignora verifica en torchwood: ' + (p.stdout + p.stderr).strip())
p = subprocess.run([BIN + 'testigo-local', 'comprobar', '-vkey', vk[0]], input=nc, capture_output=True, text=True)
comprobar(p.returncode == 0, 'torchwood lee la cofirma retenida de torchwood-a en la nota cofirmada: ' + p.stdout.strip())

if GUARDAR:
    for t in testigos.values():
        t.pop('db', None)
    salida = {'v': 1,
              'que': 'RFC-0013 E3a: el cliente add-checkpoint de zk_ssl_medio::testigos contra el motor de '
                     'litewitness con ML-DSA-44 (torchwood-a, torchwood-b) y contra litewitness con Ed25519',
              'torchwood': 'filippo.io/torchwood v0.10.0', 'go': VERSION_GO,
              'tlog_witness': 'C2SP tlog-witness v1.1.0 (b2dd989)',
              'origen': ORIGEN, 'testigos': testigos, 'pasos': pasos, 'sobre': sobre}
    open(AQUI + '/torchwood.json', 'w').write(json.dumps(salida, indent=1, sort_keys=True, ensure_ascii=False) + '\n')
    msg('escrito %s/torchwood.json' % AQUI)
sys.exit(1 if fallos else 0)
PY
RC=$?
[ "$RC" = 0 ] || fallo "algun paso no dijo lo que debia"
[ "$(huella)" = "$ANTES" ] || fallo "el arbol cambio mas alla de torchwood.json: no debe tocarlo"
msg "TESTIGOS-LOCALES VERDE"
