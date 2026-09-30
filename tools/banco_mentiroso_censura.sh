#!/usr/bin/env bash
# tools/banco_mentiroso_censura.sh -- la E3 del RFC-0011: la CENSURA ANTES DEL RECIBO, medida.
#
# El residuo D-H del RFC-0010: la operacion para la que el nodo NUNCA firmo un recibo. «Si no
# contesta, o contesta sin recibo, no hay objeto que oponer.» Hasta aqui estaba declarado; este
# banco lo MIDE. Es la excepcion de D-D del RFC-0011: su exito no es que una defensa dispare, sino
# el SILENCIO de todas, asertado una por una.
#
# El operador pone delante de su nodo HONESTO -el binario SIN TOCAR- un proxy que reenvia todo y
# descarta a quien quiere: la unica mentira que se puede sin la clave es la omision (D-B). El proxy
# vive EMBEBIDO en el banco, nace y muere con el. Tres envios del mismo titular, por el proxy:
#
#   A  el proxy REENVIA: el recibo llega (`error.data.recepcion`, el envio de ceros que la capa
#      rechaza y que consume un numero de recepcion) y la cabeza firmada pasa a recepCount 1.
#   B  el proxy NO CONTESTA: cierra la conexion sin respuesta.
#   C  el proxy CONTESTA SIN RECIBO: un error generico, sin `recepcion`.
#
# El titular lo sabe al instante en B y en C -su peticion no trae recibo-, pero no puede probarlo:
# el nodo no las vio, y su registro firmado lo dice sin mentir -recepCount sigue en 1-. Dos
# testigos miran todo el rato, uno POR EL PROXY y otro DIRECTO al nodo, y todas las defensas
# callan: los dos terminan sin hallazgo, `--auditar` limpio, `--comparar` sin divergencias,
# `--ausentes` sin ausentes. El banco sale ROJO si alguna habla, y tambien si la censura no ocurrio
# de verdad: si el contador firmado se moviera con B o con C, el nodo las habria visto.
#
# FUERA del canon: levanta procesos y espera latidos. NO ESCRIBE EN EL ARBOL: todo vive en un
# temporal bajo $HOME, que borra al salir, y lo comprueba al final por `git status --porcelain`.
#
#   bash tools/banco_mentiroso_censura.sh
set -u
msg(){ echo "BANCO-MENT-CENSURA| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
[ $# -eq 0 ] || fallo "uso: bash tools/banco_mentiroso_censura.sh"
[ -z "$(git status --porcelain)" ] || fallo "el arbol no esta limpio: el banco comprueba al final que no lo toca"
# Bajo $HOME, no /tmp: el guardian del indice (K.1, §234) se niega sobre un fsync que no
# persiste, y en WSL /tmp es tmpfs (medido en §290).
DIR=$(mktemp -d "$HOME/.banco_mentiroso_censura.XXXXXX")
trap 'rm -rf "$DIR"' EXIT INT TERM HUP QUIT
msg "compilando nodo y cli en RELEASE (aqui se firma de verdad)"
cargo build --release -q -p zk-ssl-node -p zk-ssl-cli 2>/dev/null \
  || cargo build --release -p zk-ssl-node -p zk-ssl-cli || fallo "no compila"

python3 - "$DIR" <<'PY'
import http.server, json, os, re, subprocess, sys, threading, time, urllib.request
DIR = sys.argv[1]
NODO, CLI = 'target/release/zk-ssl-node', 'target/release/zk-ssl-cli'
PN, PP, VECES = 8816, 8817, 24
def msg(m): print('BANCO-MENT-CENSURA| ' + m, file=sys.stderr, flush=True)
fallos = 0
def exigir(ok, texto, detalle=''):
    global fallos
    fallos += 0 if ok else 1
    msg('%s %s' % ('OK  ' if ok else 'ROJO', texto))
    if not ok and detalle:
        msg('     ' + detalle.strip().replace('\n', '\n     '))

# ── el PROXY del operador: reenvia todo; zkssl_applySend, segun el modo ──
MODO = {'envio': 'reenvia'}
class Proxy(http.server.BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.1'
    def log_message(self, *a):
        pass
    def responder(self, crudo):
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(crudo)))
        self.end_headers()
        self.wfile.write(crudo)
    def do_POST(self):
        cuerpo = self.rfile.read(int(self.headers.get('Content-Length', 0)))
        try:
            pet = json.loads(cuerpo)
        except Exception:
            pet = {}
        if pet.get('method') == 'zkssl_applySend':
            if MODO['envio'] == 'calla':
                # NO CONTESTA: la conexion se cierra sin una sola cabecera
                self.close_connection = True
                return
            if MODO['envio'] == 'sin-recibo':
                self.responder(json.dumps({'jsonrpc': '2.0', 'id': pet.get('id'), 'error': {
                    'code': -32000, 'message': 'servicio no disponible, reintente'}}).encode())
                return
        try:
            r = urllib.request.Request('http://127.0.0.1:%d' % PN, cuerpo, {'Content-Type': 'application/json'})
            with urllib.request.urlopen(r, timeout=15) as x:
                crudo = x.read()
        except Exception as e:
            # el nodo aun no escucha, o ya no: se dice como lo diria el cable
            crudo = json.dumps({'jsonrpc': '2.0', 'id': pet.get('id'), 'error': {
                'code': -32000, 'message': 'proxy: %s' % e}}).encode()
        self.responder(crudo)
proxy = http.server.ThreadingHTTPServer(('127.0.0.1', PP), Proxy)
threading.Thread(target=proxy.serve_forever, daemon=True).start()

procesos = []
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
def cabeza(port):
    try:
        r = rpc(port, 'zkssl_signedEpochHead', {}).get('result', {})
        return r if r.get('available') else None
    except Exception:
        return None
def esperar_cabeza(port, cond, seg=60):
    fin = time.time() + seg
    while time.time() < fin:
        c = cabeza(port)
        if c and cond(c):
            return c
        time.sleep(0.25)
    return None
def testigo(nombre, port):
    d = '%s/%s' % (DIR, nombre); os.makedirs(d)
    p = subprocess.Popen([CLI, 'witness', '--nodo', 'http://127.0.0.1:%d' % port, '--cada', '1',
                          '--veces', str(VECES), '--diario', d + '/diario.jsonl', '--no-color'],
                         stdout=open(d + '/salida.txt', 'w'), stderr=subprocess.STDOUT)
    procesos.append(p)
    return p, d
def cli(*args):
    p = subprocess.run([CLI, 'witness'] + list(args) + ['--no-color'], capture_output=True, text=True)
    return p.returncode, p.stdout + p.stderr
def recep(c):
    return int(c['recepCount'], 16)
Z = '0x' + '00' * 32
# El envio de CEROS de banco_completitud.sh: la capa lo RECHAZA y el rechazo consume un numero de
# recepcion; su recibo viaja en el error (§571). Es una operacion real con recibo, sin prueba STARK.
cero_pi = {'rootOld': Z, 'rootNew': Z, 'frozenRoot': Z, 'pendingRootOld': Z, 'pendingRootNew': Z,
           'amount': '0x5', 'regulatoryLimit': '0x3e8', 'supplyOld': '0x0', 'supplyNew': '0x0'}
envio = {'receipt': {'proof': '0x' + '00' * 32, 'publicInputs': cero_pi, 'commitment': Z,
                     'notice': {'position': '0x0', 'salt': Z, 'amount': '0x5'}},
         'sender': '0x0', 'senderState': {'publicId': Z, 'balance': '0x5', 'nonce': '0x0'}, 'amount': '0x5'}
def enviar_por_el_proxy():
    # lo que el TITULAR tiene en la mano tras enviar: la respuesta, o la excepcion de no tenerla
    try:
        return rpc(PP, 'zkssl_applySend', envio), None
    except Exception as e:
        return None, e
try:
    d = DIR + '/nodo'; os.makedirs(d)
    open(d + '/semilla.hex', 'w').write('43' * 96); os.chmod(d + '/semilla.hex', 0o600)
    n = subprocess.Popen([NODO, '--listen', '127.0.0.1:%d' % PN, '--latido', '1',
                          '--clave-fichero', d + '/semilla.hex', '--custodia', 'fichero',
                          '--diario', d + '/diario.jsonl', '--ledger', d + '/ledger',
                          '--contador-recepcion', d + '/recepcion.bin',
                          '--indice-firma', d + '/indice.bin', '--log', 'warn'],
                         stdout=subprocess.DEVNULL, stderr=open(d + '/nodo.err', 'w'))
    procesos.append(n)
    exigir(esperar_cabeza(PP, lambda c: True) is not None, 'el nodo firma, y el proxy del operador lo sirve')
    (w1, d1), (w2, d2) = testigo('testigo-proxy', PP), testigo('testigo-directo', PN)
    msg('dos testigos, %d vueltas: uno POR EL PROXY, otro DIRECTO al nodo' % VECES)

    msg('A: el proxy REENVIA')
    r, e = enviar_por_el_proxy()
    rx = ((r or {}).get('error') or {}).get('data', {}).get('recepcion')
    exigir(rx is not None, 'A: el titular tiene su RECIBO en la mano (error.data.recepcion)', repr(r or e)[:400])
    ca = esperar_cabeza(PN, lambda c: recep(c) >= 1)
    exigir(ca is not None and recep(ca) == 1,
           'A: la cabeza firmada lo cuenta: recepCount = %s' % (recep(ca) if ca else None))

    msg('B: el proxy NO CONTESTA')
    MODO['envio'] = 'calla'
    r, e = enviar_por_el_proxy()
    exigir(r is None and e is not None,
           'B: el titular lo sabe al instante -sin respuesta: %s-' % type(e).__name__, repr(r)[:300])

    msg('C: el proxy CONTESTA SIN RECIBO')
    MODO['envio'] = 'sin-recibo'
    r, e = enviar_por_el_proxy()
    err = (r or {}).get('error') or {}
    exigir(r is not None and err and 'recepcion' not in (err.get('data') or {}),
           'C: el titular lo sabe al instante -un error sin recepcion: «%s»-' % err.get('message'),
           repr(r or e)[:300])

    tras = cabeza(PN)
    base = int(tras['index'], 16) if tras else 0
    cb = esperar_cabeza(PN, lambda c: int(c['index'], 16) >= base + 4)
    exigir(cb is not None and recep(cb) == 1,
           'B y C: cuatro cabezas despues, el registro firmado sigue en recepCount %s -el nodo no las vio-'
           % (recep(cb) if cb else None))

    r1, r2 = esperar(w1, 120), esperar(w2, 120)
    matar(n)
    msg('el nodo esta PARADO; las defensas se juzgan SIN el')
    lt1 = [json.loads(l) for l in open(d1 + '/diario.jsonl') if l.strip()]
    lt2 = [json.loads(l) for l in open(d2 + '/diario.jsonl') if l.strip()]
    vistos = sorted({int(v['recepCount'], 16) for v in lt1 + lt2 if v.get('signature') and v.get('recepCount')})
    exigir(vistos and max(vistos) == 1,
           'los dos testigos vieron recepCount %s y nunca mas de 1' % vistos)
    exigir(r1 == 0 and r2 == 0, 'SILENCIO: los dos testigos terminan sin hallazgo (exit %s y %s)' % (r1, r2),
           open(d1 + '/salida.txt').read()[-500:] + '\n' + open(d2 + '/salida.txt').read()[-500:])
    for nombre, dd in (('testigo-proxy', d1), ('testigo-directo', d2)):
        rc, out = cli('--auditar', dd + '/diario.jsonl')
        exigir(rc == 0 and 'sin hallazgos' in out, 'SILENCIO: --auditar %s, sin hallazgos' % nombre, out[-600:])
    rc, out = cli('--comparar', d1 + '/diario.jsonl', d2 + '/diario.jsonl')
    exigir(rc == 0 and 'sin divergencias' in out,
           'SILENCIO: --comparar, el proxy y el directo sin divergencias -el proxy no toca las cabezas-', out[-600:])
    for nombre, dd in (('testigo-proxy', d1), ('testigo-directo', d2)):
        rc, out = cli('--ausentes', dd + '/diario.jsonl', d + '/diario.jsonl')
        exigir(rc == 0 and 'sin ausentes' in out,
               'SILENCIO: --ausentes %s contra el diario del nodo, sin ausentes' % nombre, out[-600:])
finally:
    for p in procesos:
        matar(p)
    proxy.shutdown()
if not fallos:
    msg('NINGUNA defensa habla de B ni de C: el residuo D-H del RFC-0010, MEDIDO. Sin recibo no hay')
    msg('objeto que oponer; el titular sabe que no lo tiene, y no puede probar que envio.')
sys.exit(1 if fallos else 0)
PY
RC=$?
[ "$RC" = 0 ] || fallo "alguna defensa hablo, o la censura no ocurrio"
[ -z "$(git status --porcelain)" ] || fallo "el banco dejo el arbol sucio: no debe tocarlo"
msg "BANCO-MENT-CENSURA VERDE"
