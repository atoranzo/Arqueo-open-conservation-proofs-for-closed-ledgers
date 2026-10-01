#!/usr/bin/env bash
# tools/banco_ancla_cofirmada.sh -- el banco de la E4b del RFC-0013: el sobre del ANCLA
# COFIRMADA, en vivo.
#
# Nodo REAL firmando latidos (uno por segundo) -> tres cabezas firmadas -> el mando DERIVA el
# ancla de cada una (el modo 1 del sobre del ancla: un solo productor) -> el medio del operador
# publica las tres anclas en su arbol SHA-256 y firma su nota checkpoint con ML-DSA-44 tipo 0x06,
# y testigos de prueba la cofirman (`zk-ssl-medio`, ejemplo `medio`, con sal) -> el mando juzga
# cada sobre SIN el nodo. Cinco positivos y un negativo por cada regla PRODUCIBLE del sobre,
# derivado por UNA mutacion de lo capturado; el medio de OTRA clave, con la cabeza de un SEGUNDO
# nodo de otra semilla que firma una vez y se para.
#
# FUERA del canon: levanta procesos y espera latidos. NO ESCRIBE EN EL ARBOL: todo vive en un
# temporal bajo $HOME, que borra al salir, y lo comprueba al final por `git status --porcelain`.
#
#   bash tools/banco_ancla_cofirmada.sh [--guardar <dir>]
#
# --guardar  copia los sobres (positivos y negativos) a <dir>: de ahi sale el catalogo, COPIADO.
set -u
msg(){ echo "BANCO-ANCLA-COFIRMADA| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
GUARDAR=""
while [ $# -gt 0 ]; do
  case "$1" in
    --guardar) GUARDAR="${2:?--guardar exige un directorio}"; mkdir -p "$GUARDAR"; shift ;;
    *) fallo "uso: bash tools/banco_ancla_cofirmada.sh [--guardar <dir>]" ;;
  esac
  shift
done
[ -z "$(git status --porcelain)" ] || fallo "el arbol no esta limpio: el banco comprueba al final que no lo toca"
DIR=$(mktemp -d "$HOME/.banco_ancla_cofirmada.XXXXXX")
trap 'rm -rf "$DIR"' EXIT INT TERM HUP QUIT
msg "compilando nodo, verificador y el medio del banco en RELEASE (aqui se firma de verdad)"
{ cargo build --release -q -p zk-ssl-node -p zk-ssl-verify \
    && cargo build --release -q -p zk-ssl-medio --example medio; } 2>/dev/null \
  || { cargo build --release -p zk-ssl-node -p zk-ssl-verify \
    && cargo build --release -p zk-ssl-medio --example medio; } || fallo "no compila"

python3 - "$DIR" "${GUARDAR:-}" <<'PY'
import json, os, shutil, subprocess, sys, time, urllib.request
DIR, GUARDAR = sys.argv[1], sys.argv[2]
NODO, MANDO = 'target/release/zk-ssl-node', 'target/release/zk-ssl-verify'
MEDIO = 'target/release/examples/medio'
def msg(m): print('BANCO-ANCLA-COFIRMADA| ' + m, file=sys.stderr, flush=True)
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
def cabezas(port, n):
    vistas = {}
    for _ in range(240):
        try:
            c = rpc(port, 'zkssl_signedEpochHead', {}).get('result', {})
            if c.get('available'):
                vistas[int(c['index'], 16)] = c
                if len(vistas) >= n:
                    return [vistas[i] for i in sorted(vistas)][:n]
        except Exception:
            pass
        time.sleep(0.25)
    msg('ROJO: el nodo del puerto %d no firmo %d cabezas' % (port, n)); sys.exit(1)
fallos = 0
def mando(nombre, sobre, codigo, texto):
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
    if GUARDAR:
        shutil.copy(ruta, GUARDAR)
    return out
def sin(d, k):
    return {x: y for x, y in d.items() if x != k}
def derivar(cabeza):
    # El PRODUCTOR del ancla es el mando (RFC-0012, D-A): se le da la cabeza sola y se lee lo
    # que imprime -la huella de la clave y la del ancla-, sin recomputar nada aqui.
    ruta = '%s/derivar.json' % DIR
    json.dump({'v': 1, 'tipo': 'ancla', 'cabeza': cabeza}, open(ruta, 'w'))
    p = subprocess.run([MANDO, ruta], capture_output=True, text=True)
    assert p.returncode == 0, p.stdout + p.stderr
    clave = huella = None
    for linea in p.stdout.splitlines():
        if linea.strip().startswith('{'):
            clave = json.loads(linea)['clave']
        if 'la huella del ancla:' in linea:
            huella = linea.split('la huella del ancla:')[1].split()[0]
    assert clave and huella, 'el mando no imprimio el ancla derivada:\n' + p.stdout
    return clave, huella
def medio(origen, hojas, posicion, testigos=(), semilla='01'):
    e = {'origen': origen, 'hojas': hojas, 'posicion': posicion,
         'publicador': {'semilla': '0x' + semilla * 32, 'marca': int(time.time())},
         'testigos': [{'nombre': n, 'semilla': '0x' + s * 32, 'marca': int(time.time()) + 60}
                      for n, s in testigos]}
    p = subprocess.run([MEDIO], input=json.dumps(e), capture_output=True, text=True)
    assert p.returncode == 0, p.stderr
    return json.loads(p.stdout)
def nibble(hexs):
    ult = hexs[-1]
    return hexs[:-1] + ('0' if ult != '0' else '1')
def tocar_linea(nota, nombre):
    # Cambia UN caracter de la base64 de la linea de `nombre`, lejos de su key_id.
    lineas = nota.split('\n')
    for k, l in enumerate(lineas):
        if l.startswith('— ' + nombre + ' '):
            i = len('— ' + nombre + ' ') + 100
            lineas[k] = l[:i] + ('A' if l[i] != 'A' else 'B') + l[i + 1:]
            return '\n'.join(lineas)
    raise AssertionError('la nota no lleva la linea de ' + nombre)
def linea_de(nota, nombre):
    for l in nota.split('\n'):
        if l.startswith('— ' + nombre + ' '):
            return l + '\n'
    raise AssertionError(nombre)
P, P_AJENO = 8796, 8797
UNO, DOS = 'testigo.invalid/uno', 'testigo.invalid/dos'
# La cabeza de OTRA clave: un segundo nodo, de otra semilla, que firma una vez y se para.
ajeno = levantar('ajeno', '38', P_AJENO)
try:
    (cabeza_ajena,) = cabezas(P_AJENO, 1)
finally:
    ajeno.kill(); ajeno.wait()
clave_ajena, _ = derivar(cabeza_ajena)
msg('la cabeza de OTRA clave: indice %s, del segundo nodo, ya parado' % cabeza_ajena['index'])
nodo = levantar('nodo', '37', P)
try:
    c1, c2, c3 = cabezas(P, 3)
finally:
    nodo.kill(); nodo.wait()
msg('tres cabezas firmadas, indices %s, %s y %s; el nodo esta MUERTO y el mando juzga SIN el'
    % (c1['index'], c2['index'], c3['index']))
(clave, a1), (_, a2), (_, a3) = derivar(c1), derivar(c2), derivar(c3)
origen = 'zkssl/v1/' + clave[2:]
otro_origen = 'zkssl/v1/' + clave_ajena[2:]
msg('el medio %s publica las tres anclas, en su orden' % origen)
m = medio(origen, [a1, a2, a3], 1, [(UNO, '02')])
base = {'v': 1, 'tipo': 'ancla-cofirmada', 'cabeza': c2, 'nota': m['nota'],
        'publicador': m['publicador'], 'testigos': m['testigos'], 'posicion': '0x1',
        'inclusion': m['inclusion']}
# los cinco positivos
mando('cofirmada-un-testigo', base, 0, 'la cofirman 1 testigo(s)')
m2 = medio(origen, [a1, a2, a3], 1, [(UNO, '02'), (DOS, '03')])
mando('cofirmada-dos-testigos', dict(base, nota=m2['nota'], publicador=m2['publicador'],
                                     testigos=m2['testigos']), 0, 'la cofirman 2 testigo(s)')
mando('cofirmada-sin-claves-de-testigo', sin(base, 'testigos'), 0,
      'lineas sin clave en el sobre, sin juzgar: 1')
m1 = medio(origen, [a1], 0, [(UNO, '02')])
mando('cofirmada-medio-de-una', dict(base, cabeza=c1, nota=m1['nota'], publicador=m1['publicador'],
                                     testigos=m1['testigos'], posicion='0x0', inclusion=[]),
      0, 'en la posicion 0')
m3 = medio(origen, [a1, a2, a3], 2, [(UNO, '02')])
mando('cofirmada-ultima', dict(base, cabeza=c3, nota=m3['nota'], publicador=m3['publicador'],
                               testigos=m3['testigos'], posicion='0x2', inclusion=m3['inclusion']),
      0, 'en la posicion 2')
# la forma
mando('neg-con-ancla', dict(base, ancla={}), 1, 'no lleva ancla')
mando('neg-con-contraria', dict(base, contraria=c1), 1, 'no lleva contraria')
mando('neg-sin-cabeza', sin(base, 'cabeza'), 1, 'falta cabeza (la firmada cuya ancla se publico)')
mando('neg-sin-nota', sin(base, 'nota'), 1, 'falta nota')
mando('neg-sin-publicador', sin(base, 'publicador'), 1, 'falta publicador')
mando('neg-sin-posicion', sin(base, 'posicion'), 1, 'falta posicion')
mando('neg-sin-inclusion', sin(base, 'inclusion'), 1, 'falta inclusion')
corta = list(base['inclusion']); corta[0] = corta[0][:-2]
mando('neg-inclusion-corta', dict(base, inclusion=corta), 1, 'inclusion: 31 bytes')
mando('neg-testigo-vkey-mala', dict(base, testigos=['no es una vkey']), 1, 'testigos: clave')
mando('neg-testigo-dos-veces', dict(base, testigos=base['testigos'] * 2), 1, 'esta dos veces')
# la cabeza
mando('neg-cabeza-v2', dict(base, cabeza=dict(c2, formatVersion='0x2')), 1, 'el ancla lee cabezas')
mando('neg-cabeza-tocada', dict(base, cabeza=dict(c2, accountsRoot=nibble(c2['accountsRoot']))),
      1, 'NO recomponen su epochDigest')
# la nota
ajena = medio(otro_origen, [a1, a2, a3], 1, [(UNO, '02')], semilla='05')
mando('neg-medio-de-otra-clave', dict(base, nota=ajena['nota'], publicador=ajena['publicador']),
      1, 'el publicador es')
impostor = medio(origen, [a1, a2, a3], 1, semilla='04')
mando('neg-publicador-otra-clave', dict(base, publicador=impostor['publicador']),
      1, 'ninguna firma del publicador')
mando('neg-nota-firma-tocada', dict(base, nota=tocar_linea(base['nota'], origen)),
      1, 'la firma del publicador no verifica')
texto, resto = base['nota'].split('\n\n', 1)
mando('neg-nota-con-extension', dict(base, nota=texto + '\nextra\n\n' + resto),
      1, 'checkpoint mal formado')
# las cofirmas
mando('neg-cofirma-tocada', dict(base, nota=tocar_linea(base['nota'], UNO)),
      1, 'la cofirma de testigo.invalid/uno no verifica')
mando('neg-cofirma-repetida', dict(base, nota=base['nota'] + linea_de(base['nota'], UNO)),
      1, 'la nota lleva dos cofirmas de testigo.invalid/uno')
# la inclusion
mando('neg-posicion-otra', dict(base, posicion='0x2'), 1, 'no esta en la posicion 2')
tocada = list(base['inclusion']); tocada[0] = nibble(tocada[0])
mando('neg-inclusion-tocada', dict(base, inclusion=tocada), 1, 'no esta en la posicion 1')
sin_ella = medio(origen, [a1, a3, a3], 1, [(UNO, '02')])
mando('neg-medio-sin-el-ancla', dict(base, nota=sin_ella['nota'], publicador=sin_ella['publicador'],
                                     testigos=sin_ella['testigos'], inclusion=sin_ella['inclusion']),
      1, 'no esta en la posicion 1')
mando('neg-ancla-de-otra-cabeza', dict(base, cabeza=c3), 1, 'no esta en la posicion 1')
sys.exit(1 if fallos else 0)
PY
RC=$?
[ "$RC" = 0 ] || fallo "algun sobre no dijo lo que debia"
[ -z "$(git status --porcelain)" ] || fallo "el banco dejo el arbol sucio: no debe tocarlo"
[ -n "$GUARDAR" ] && msg "sobres guardados en $GUARDAR (de ahi sale el catalogo, COPIADO)"
msg "BANCO-ANCLA-COFIRMADA VERDE"
