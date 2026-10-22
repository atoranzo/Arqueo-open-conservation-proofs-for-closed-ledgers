#!/usr/bin/env bash
# crates/zk-ssl-medio/tests/vectores/testigos/rehacer_omniwitness.sh -- el publicador del medio
# contra omniwitness, el testigo de github.com/transparency-dev/witness en 5397155, en LOCAL
# (RFC-0013 E3a, §848):
#
#   - omniwitness-a: omniwitness sin tocar, con una clave ML-DSA-44 (--private_key_path);
#   - omniwitness-b: omniwitness sin tocar, con dos claves del mismo nombre, una ML-DSA-44 y
#     detras una Ed25519, como en una rotacion de claves de su README.
#
# El log del medio (tipo 0x06) se le configura a mano, con --additional_logs. El cliente es
# `zk_ssl_medio::testigos` (el ejemplo `testigos_locales`), y los pasos son los de rehacer.sh que
# aplican, dichos en terminos de tlog-witness: el origin que no conoce (404), la primera cofirma
# (old 0), el publicador sin memoria (409 con 3 y el reintento), el crecimiento, el atrasado (409
# con 7 y la prueba de 7 a 9), una nota de 5 cuando el testigo tiene 9 (409 con un tamano mayor que
# el de la nota: el cliente PARA), una prueba de consistencia que no verifica (422), la misma talla
# con otra raiz (422), la nota de otra clave (403) y, al final, la de 9 con old 9, que cofirma. Y
# de punta a punta: la nota cofirmada por omniwitness-a y -b entra en el sobre `ancla-cofirmada`
# de `spec/vectors/` y el kit la da VERDE; el juez del cli la ACREDITA con k = 2 y no con k = 3; y
# formats, el verificador de omniwitness, lee las cofirmas retenidas y la linea Ed25519.
#
# ⚠️ FUERA del canon y FUERA de tools/bancos.sh: pide Go (GO=..., si no esta en el PATH) y red
# para bajar witness y sus modulos por el proxy de Go, y la cadena de Go 1.26 que su go.mod pide.
# NO TOCA testigos reales: todo escucha en 127.0.0.1, sin metricas, sin distribuidor y sin la red
# publica de testigos. Vive en un temporal bajo $HOME que borra al salir, procesos incluidos, y
# comprueba al final que no ha tocado el arbol.
#
#   bash crates/zk-ssl-medio/tests/vectores/testigos/rehacer_omniwitness.sh [--guardar]
#
# --guardar  reescribe omniwitness.json en este directorio: lo que relee
#            tests/testigos_omniwitness.rs.
set -u
msg(){ echo "OMNIWITNESS-LOCAL| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
GUARDAR=""
case "${1:-}" in
  --guardar) GUARDAR=1 ;;
  "") ;;
  *) fallo "uso: bash crates/zk-ssl-medio/tests/vectores/testigos/rehacer_omniwitness.sh [--guardar]" ;;
esac
cd "$(git rev-parse --show-toplevel)" || fallo "fuera de un arbol de git"
AQUI=crates/zk-ssl-medio/tests/vectores/testigos
GRAB=omniwitness.json
# Lo que hay en el arbol, con lo que no esta commiteado: se compara al final.
huella(){ { git status --porcelain=v1 -uall | grep -v " $AQUI/$GRAB\$"; \
            git diff -- . ":!$AQUI/$GRAB"; \
            git ls-files -o --exclude-standard -z | grep -zv "^$AQUI/$GRAB\$" | xargs -0 -r sha256sum; } | sha256sum; }
ANTES=$(huella)
GO="${GO:-$(command -v go || echo /usr/local/go/bin/go)}"
[ -x "$GO" ] || fallo "sin Go: GO=/ruta/al/go"
# Los binarios de Rust, donde los deja cargo (§844): lo dice tools/target_dir.sh (§855).
T=$(bash tools/target_dir.sh) || fallo "no se donde deja cargo los binarios (tools/target_dir.sh)"  # ⚠️ §855: decia ${CARGO_TARGET_DIR:-target}
DIR=$(mktemp -d "$HOME/.omniwitness_local.XXXXXX")
trap 'chmod -R u+w "$DIR" 2>/dev/null; rm -rf "$DIR"' EXIT INT TERM HUP QUIT
# Los modulos y la cadena de Go, en el temporal salvo que quien llama exporte los suyos.
export GOMODCACHE="${GOMODCACHE:-$DIR/go/mod}" GOCACHE="${GOCACHE:-$DIR/go/cache}" GOPATH="${GOPATH:-$DIR/go/path}"
export GOFLAGS="-mod=mod -modcacherw" GOTOOLCHAIN=auto

# 5397155 es 539715523ab9d0ee7d036926f9d1b00266e1c567, la punta de main el 2026-10-15.
WITNESS=github.com/transparency-dev/witness@v0.0.0-20261014123032-539715523ab9
msg "bajando $WITNESS (comprobado contra sum.golang.org)"
ORIGEN_W=$(cd "$DIR" && "$GO" mod download -json "$WITNESS" \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["Dir"])') || fallo "no baja witness"
cp -r "$ORIGEN_W" "$DIR/witness" && chmod -R u+w "$DIR/witness"
mkdir -p "$DIR/witness/cmd/omniwitness-local" "$DIR/bin"
cp "$AQUI/omniwitness-local/main.go" "$DIR/witness/cmd/omniwitness-local/"
msg "compilando omniwitness y omniwitness-local con la cadena de Go que pide witness"
(cd "$DIR/witness" && "$GO" build -o "$DIR/bin/" ./cmd/omniwitness ./cmd/omniwitness-local) \
  || fallo "Go no compila"
VERSION_GO=$(cd "$DIR/witness" && "$GO" version)
FORMATS=$(cd "$DIR/witness" && "$GO" list -m github.com/transparency-dev/formats) || fallo "sin formats"
msg "$VERSION_GO; $FORMATS"
msg "compilando el kit, el cli y el cliente del medio en RELEASE"
{ cargo build --locked --release -q -p zk-ssl-verify -p zk-ssl-cli \
    && cargo build --locked --release -q -p zk-ssl-medio --example testigos_locales; } \
  || fallo "no compila"

python3 - "$DIR" "$AQUI" "${GUARDAR:-}" "$VERSION_GO" "$FORMATS" "$T" "$GRAB" <<'PY'
import atexit, json, hashlib, socket, subprocess, sys, time
DIR, AQUI, GUARDAR, VERSION_GO, FORMATS, T, GRAB = sys.argv[1:8]
BIN = DIR + '/bin/'
MANDO, JUEZ = T + '/release/zk-ssl-verify', T + '/release/zk-ssl-cli'
CLIENTE = T + '/release/examples/testigos_locales'
CAT = 'spec/vectors/ancla-cofirmada/'
def msg(m): print('OMNIWITNESS-LOCAL| ' + m, file=sys.stderr, flush=True)
fallos = 0
def comprobar(ok, que):
    global fallos
    fallos += 0 if ok else 1
    msg('%s %s' % ('OK  ' if ok else 'ROJO', que))
    return ok
def semilla(desde):
    return ''.join('%02x' % ((desde + i) & 0xff) for i in range(32))
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

# Las claves de los testigos: ML-DSA-44 de una semilla, y en -b detras una Ed25519 del mismo nombre.
testigos = {}
for nombre, claves in (('testigo.invalid/omniwitness-a', [('mldsa', 0x80)]),
                       ('testigo.invalid/omniwitness-b', [('mldsa', 0xa0), ('ed25519', 0xc0)])):
    t = {'claves': [], 'ficheros': []}
    for i, (alg, desde) in enumerate(claves):
        f = '%s/%s-%d.key' % (DIR, nombre.split('-')[-1], i)
        r = subprocess.run([BIN + 'omniwitness-local', 'clave', '-alg', alg, '-name', nombre,
                            '-seed', semilla(desde), '-out', f], capture_output=True, text=True, check=True)
        t['claves'].append({'alg': alg, 'semilla': '0x' + semilla(desde), 'vkey': r.stdout.strip()})
        t['ficheros'].append(f)
    t['vkey'] = t['claves'][0]['vkey']
    t['db'] = '%s/%s.db' % (DIR, nombre.split('-')[-1])
    s = socket.socket(); s.bind(('127.0.0.1', 0)); t['puerto'] = s.getsockname()[1]; s.close()
    t['prefijo'] = 'http://127.0.0.1:%d' % t['puerto']
    testigos[nombre] = t

procesos = {}
atexit.register(lambda: [p.kill() for p in procesos.values()])
def lanzar(nombre, logs=None):
    t = testigos[nombre]
    args = [BIN + 'omniwitness', '--listen', '127.0.0.1:%d' % t['puerto'], '--metrics_listen=',
            '--db_file', t['db']]
    for f in t['ficheros']:
        args += ['--private_key_path', f]
    if logs:
        args += ['--additional_logs', logs]
    err = open('%s/%s.%d.err' % (DIR, nombre.split('-')[-1], len(procesos)), 'w')
    p = subprocess.Popen(args, stdout=err, stderr=err)
    for _ in range(200):
        try:
            socket.create_connection(('127.0.0.1', t['puerto']), timeout=1).close()
            break
        except OSError:
            assert p.poll() is None, open(err.name).read()
            time.sleep(0.05)
    else:
        raise SystemExit('omniwitness no escucha: ' + open(err.name).read())
    procesos[nombre] = p
def parar(nombre):
    p = procesos.pop(nombre)
    p.terminate()
    p.wait(timeout=10)

for nombre in testigos:
    lanzar(nombre)
msg('omniwitness-a y omniwitness-b escuchan en 127.0.0.1, sin el log del medio')

PUB = {'semilla': '0x' + '01' * 32, 'marca': 1790000000}
IMPOSTOR = {'semilla': '0x' + '04' * 32, 'marca': 1790000000}
pasos = []
def paso(nombre, h, viejo, esperado, publicador=PUB):
    e = {'origen': ORIGEN, 'hojas': h, 'publicador': publicador,
         'testigos': [{'vkey': t['vkey'], 'prefijo': t['prefijo'], 'viejo': viejo} for t in testigos.values()]}
    p = subprocess.run([CLIENTE], input=json.dumps(e), capture_output=True, text=True)
    assert p.returncode == 0, p.stderr
    s = json.loads(p.stdout)
    for t, visto, (resultado, campos) in zip(testigos, s['testigos'], esperado):
        ok = visto['resultado'] == resultado and all(visto.get(k) == v for k, v in campos.items())
        dicho = {k: visto[k] for k in ('viejo', 'ignoradas', 'recordado', 'respuesta', 'error') if k in visto}
        comprobar(ok, '%-15s %-14s %-15s %s, %d intercambio(s)'
                  % (nombre, t.split('/')[1], visto['resultado'], dicho, len(visto['intercambios'])))
    pasos.append({'paso': nombre, 'entrada': e, 'salida': s})
    return s

paso('desconocido', hojas(3), 0, [('rechazada', {'respuesta': 'OrigenDesconocido'})] * 2)
publicador = pasos[0]['salida']['publicador']
# El log del medio, configurado a mano: la entrada que omniwitness lee con --additional_logs.
logs = DIR + '/logs.yaml'
open(logs, 'w').write('Logs:\n  - Origin: %s\n    PublicKey: %s\n' % (ORIGEN, publicador))
for nombre in testigos:
    parar(nombre)
    lanzar(nombre, logs)
msg('el medio %s, configurado en los dos con --additional_logs (misma base de datos)' % ORIGEN)
cof = lambda viejo, ign: ('cofirmada', {'viejo': viejo, 'ignoradas': ign})
ambos = lambda viejo: [cof(viejo, 0), cof(viejo, 1)]
primera = paso('primera', hojas(3), 0, ambos(0))
paso('sin-memoria', hojas(3), 0, ambos(3))
paso('crece', hojas(7), 3, ambos(3))
paso('atrasado', hojas(9), 0, ambos(7))
paso('nota-menor', hojas(5), 0, [('retroceso', {'recordado': 9, 'nota': 5})] * 2)
paso('prueba-invalida', hojas(11, 4), 9, [('rechazada', {'respuesta': 'Inconsistente'})] * 2)
paso('otra-raiz', hojas(9, 4), 9, [('rechazada', {'respuesta': 'Inconsistente'})] * 2)
paso('impostor', hojas(11), 9, [('rechazada', {'respuesta': 'Prohibida'})] * 2, IMPOSTOR)
paso('igual', hojas(9), 9, ambos(9))

# De punta a punta: la nota cofirmada por los dos omniwitness, en el sobre del catalogo.
nc = primera['nota_cofirmada']
comprobar(nc.split('\n')[2] == cat['cofirmada-dos-testigos']['nota'].split('\n')[2],
          'la raiz de las tres anclas derivadas es la de la nota del catalogo')
comprobar(publicador == cat['cofirmada-dos-testigos']['publicador'], 'el publicador es el del catalogo (semilla 01)')
vk = [t['vkey'] for t in testigos.values()]
sobre = dict(cat['cofirmada-dos-testigos'], nota=nc, publicador=publicador, testigos=vk)
ruta = DIR + '/sobre.json'
json.dump(sobre, open(ruta, 'w'), indent=1, sort_keys=True)
p = subprocess.run([MANDO, ruta], capture_output=True, text=True)
comprobar(p.returncode == 0 and 'la cofirman 2 testigo(s)' in p.stdout,
          'el kit da VERDE el sobre con la nota cofirmada por omniwitness-a y omniwitness-b')
open(DIR + '/politica.vkeys', 'w').write('\n'.join(vk) + '\n')
for k, codigo in (('2', 0), ('3', 1)):
    p = subprocess.run([JUEZ, 'witness', '--ancla-cofirmada', ruta, '--testigos-medio', DIR + '/politica.vkeys',
                        '--k-medio', k], capture_output=True, text=True)
    comprobar(p.returncode == codigo and (codigo == 1 or 'ACREDITADO el checkpoint' in p.stdout),
              'el juez del cli con k = %s sale con %d' % (k, p.returncode))
for v in vk:
    p = subprocess.run([BIN + 'omniwitness-local', 'comprobar', '-vkey', v], input=nc, capture_output=True, text=True)
    comprobar(p.returncode == 0, 'formats lee la cofirma retenida en la nota cofirmada: ' + (p.stdout + p.stderr).strip())
b = testigos['testigo.invalid/omniwitness-b']
b_200 = primera['testigos'][1]['intercambios'][0]
p = subprocess.run([BIN + 'omniwitness-local', 'comprobar', '-vkey', b['claves'][1]['vkey']],
                   input=primera['nota'] + b_200['respuesta'], capture_output=True, text=True)
comprobar(p.returncode == 0 and p.stdout.startswith('VERIFICA testigo.invalid/omniwitness-b'),
          'la linea Ed25519 de omniwitness-b que el cliente ignora verifica en formats: ' + (p.stdout + p.stderr).strip())

if GUARDAR:
    for t in testigos.values():
        for k in ('db', 'ficheros', 'puerto'):
            t.pop(k, None)
    salida = {'v': 1,
              'que': 'RFC-0013 E3a: el cliente add-checkpoint de zk_ssl_medio::testigos contra omniwitness '
                     '(transparency-dev/witness) con ML-DSA-44 (omniwitness-a) y con ML-DSA-44 y Ed25519 '
                     '(omniwitness-b), con el log 0x06 del medio configurado a mano',
              'witness': 'github.com/transparency-dev/witness 539715523ab9d0ee7d036926f9d1b00266e1c567',
              'formats': FORMATS, 'go': VERSION_GO,
              'tlog_witness': 'C2SP tlog-witness v1.1.0 (b2dd989)',
              'origen': ORIGEN, 'testigos': testigos, 'pasos': pasos, 'sobre': sobre}
    open(AQUI + '/' + GRAB, 'w').write(json.dumps(salida, indent=1, sort_keys=True, ensure_ascii=False) + '\n')
    msg('escrito %s/%s' % (AQUI, GRAB))
sys.exit(1 if fallos else 0)
PY
RC=$?
[ "$RC" = 0 ] || fallo "algun paso no dijo lo que debia"
[ "$(huella)" = "$ANTES" ] || fallo "el arbol cambio mas alla de $GRAB: no debe tocarlo"
msg "OMNIWITNESS-LOCAL VERDE"
