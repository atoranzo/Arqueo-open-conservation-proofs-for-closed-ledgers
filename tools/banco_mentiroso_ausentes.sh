#!/usr/bin/env bash
# tools/banco_mentiroso_ausentes.sh -- la E2 del RFC-0011: la FIRMA QUE EL DIARIO NO RECOGE.
#
# `--ausentes` (§283) nunca habia dado ROJO en un banco: la comprobacion DIRIGIDA -indices que el
# testigo tiene y el diario del nodo no- solo se ejercitaba en tests. Aqui el operador firma con un
# diario y ENSENA otro, con el binario del nodo SIN TOCAR (la decision del §598: la mentira pide
# la clave y el fichero, no otro codigo). Dos escenas:
#
#   ESCENA 1  EL DIARIO CAMBIADO. El nodo firma con un testigo mirando; el operador lo reinicia con
#             el MISMO contador, el mismo libro y OTRO `--diario`, y ensena solo el nuevo. El nodo
#             ARRANCA y no reutiliza ningun indice: nada en el exige que el diario siga, y solo lo
#             avisa en su propio log (el limite declarado en main.rs y en CONFIANZA_RESIDUAL). El
#             testigo vivo anota `por-detras` y sigue: el reseteo VISIBLE del §292, que no es
#             oponible. `--ausentes` del testigo contra el diario ensenado: ROJO, con cada indice
#             del primer periodo que el testigo vio. Control: contra los DOS diarios juntos -la
#             verdad entera-, sin ausentes.
#   ESCENA 2  LA LINEA BORRADA. Un nodo honesto y su testigo; `--ausentes` contra el diario entero
#             sale VERDE aunque el nodo tenga lineas que el testigo no pidio (esa direccion es
#             paisaje, BACKLOG 80). Se borra a mano UNA linea firmada que el testigo vio:
#             `--ausentes` nombra ESE indice y ninguno mas.
#
# Exito (D-D del RFC-0011): que la defensa DISPARE nombrando los indices. Los esperados salen de
# los diarios crudos (indices firmados del testigo menos los del diario ensenado), no del texto de
# la herramienta. Claves de prueba siempre (D-E): semillas deterministas, nunca un keystore.
#
# FUERA del canon: levanta procesos y espera latidos. NO ESCRIBE EN EL ARBOL: todo vive en un
# temporal bajo $HOME, que borra al salir, y lo comprueba al final por `git status --porcelain`.
#
#   bash tools/banco_mentiroso_ausentes.sh
set -u
msg(){ echo "BANCO-MENT-AUSENTES| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
[ $# -eq 0 ] || fallo "uso: bash tools/banco_mentiroso_ausentes.sh"
[ -z "$(git status --porcelain)" ] || fallo "el arbol no esta limpio: el banco comprueba al final que no lo toca"
# Bajo $HOME, no /tmp: el guardian del indice (K.1, §234) se niega sobre un fsync que no
# persiste, y en WSL /tmp es tmpfs (medido en §290).
DIR=$(mktemp -d "$HOME/.banco_mentiroso_ausentes.XXXXXX")
trap 'rm -rf "$DIR"' EXIT INT TERM HUP QUIT
msg "compilando nodo y cli en RELEASE (aqui se firma de verdad)"
cargo build --release -q -p zk-ssl-node -p zk-ssl-cli 2>/dev/null \
  || cargo build --release -p zk-ssl-node -p zk-ssl-cli || fallo "no compila"

python3 - "$DIR" <<'PY'
import json, os, re, subprocess, sys, time, urllib.request
DIR = sys.argv[1]
NODO, CLI = 'target/release/zk-ssl-node', 'target/release/zk-ssl-cli'
def msg(m): print('BANCO-MENT-AUSENTES| ' + m, file=sys.stderr, flush=True)
fallos = 0
def exigir(ok, texto, detalle=''):
    global fallos
    fallos += 0 if ok else 1
    msg('%s %s' % ('OK  ' if ok else 'ROJO', texto))
    if not ok and detalle:
        msg('     ' + detalle.strip().replace('\n', '\n     '))
procesos = []
def nodo(d, semilla, port, diario):
    # El MISMO directorio conserva contador de indice, libro y contador de recepcion entre
    # arranques; el diario es el unico fichero que cambia.
    os.makedirs(d, exist_ok=True)
    if not os.path.exists(d + '/semilla.hex'):
        open(d + '/semilla.hex', 'w').write(semilla * 96); os.chmod(d + '/semilla.hex', 0o600)
    p = subprocess.Popen([NODO, '--listen', '127.0.0.1:%d' % port, '--latido', '1',
                          '--clave-fichero', d + '/semilla.hex', '--custodia', 'fichero',
                          '--diario', d + '/' + diario, '--ledger', d + '/ledger',
                          '--contador-recepcion', d + '/recepcion.bin',
                          '--indice-firma', d + '/indice.bin', '--log', 'warn'],
                         stdout=subprocess.DEVNULL, stderr=open('%s/%s.err' % (d, diario), 'w'))
    procesos.append(p)
    return p
def testigo(nombre, port, veces, cada=1):
    d = '%s/%s' % (DIR, nombre); os.makedirs(d)
    p = subprocess.Popen([CLI, 'witness', '--nodo', 'http://127.0.0.1:%d' % port, '--cada', str(cada),
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
    return None
def lineas(ruta):
    return [l for l in open(ruta) if l.strip()] if os.path.exists(ruta) else []
def firmados(ls):
    # los indices de las lineas con cabeza firmada: lo mismo que `--ausentes` cuenta
    s = set()
    for l in ls:
        v = json.loads(l)
        if v.get('signature') and v.get('index'):
            s.add(int(v['index'], 16))
    return s
def escribir(ruta, ls):
    open(ruta, 'w').write(''.join(ls))
    return ruta
def ausentes(t, n):
    p = subprocess.run([CLI, 'witness', '--ausentes', t, n, '--no-color'], capture_output=True, text=True)
    out = p.stdout + p.stderr
    return p.returncode, out, sorted(int(x) for x in re.findall(r'AUSENTE en el diario del nodo: indice (\d+)', out))
def clases(ls):
    # la cabeza y la historia, los dos canales del testigo: 'nueva/extiende x5 · ...'
    rachas = []
    for l in ls:
        v = json.loads(l)
        c = v.get('clase', '?') + ('/' + v['consistencia']['clase'] if v.get('consistencia') else '')
        if rachas and rachas[-1][0] == c:
            rachas[-1][1] += 1
        else:
            rachas.append([c, 1])
    return ' · '.join(c if n == 1 else '%s x%d' % (c, n) for c, n in rachas)
try:
    # ── ESCENA 1 · el operador reinicia con OTRO diario y ensena solo ese ──
    P1, N1 = 8814, DIR + '/nodo-1'
    msg('ESCENA 1: el nodo firma con el diario 1 y un testigo mirando')
    n = nodo(N1, '41', P1, 'diario-1.jsonl')
    exigir(primera_cabeza(P1) is not None, 'el nodo firma con el diario 1')
    w1, dw1 = testigo('testigo-1', P1, 0)
    time.sleep(10)
    matar(n)
    msg('nodo PARADO; vuelve con el MISMO contador y el MISMO libro, y OTRO --diario')
    n = nodo(N1, '41', P1, 'diario-2.jsonl')
    arranca = primera_cabeza(P1)
    err2 = open(N1 + '/diario-2.jsonl.err').read()
    exigir(arranca is not None,
           'el nodo ARRANCA con el diario vacio: nada en el le exige conservar el anterior', err2[-800:])
    time.sleep(6)
    vivo = w1.poll() is None
    matar(w1); matar(n)
    lt, l1, l2 = lineas(dw1 + '/diario.jsonl'), lineas(N1 + '/diario-1.jsonl'), lineas(N1 + '/diario-2.jsonl')
    msg('     el testigo en vivo: %s (%s)' % (clases(lt), 'seguia vivo' if vivo else 'se habia detenido'))
    for linea in err2.splitlines():
        if 'resincroniza' in linea or 'CONTADOR' in linea or 'contador' in linea:
            msg('     el nodo, al arrancar: ' + re.sub(r'\x1b\[[0-9;]*m', '', linea).strip()[:300])
    T, D1, D2 = firmados(lt), firmados(l1), firmados(l2)
    esperados = sorted(T - D2)
    exigir(esperados and set(esperados) <= D1 and D2 and max(esperados) < min(D2),
           'los diarios crudos: el testigo vio %d indice(s) del periodo 1 que el diario 2 no trae: %s'
           % (len(esperados), esperados), 'testigo %s\ndiario 1 %s\ndiario 2 %s' % (sorted(T), sorted(D1), sorted(D2)))
    rc, out, nombrados = ausentes(dw1 + '/diario.jsonl', N1 + '/diario-2.jsonl')
    exigir(rc != 0 and nombrados == esperados and '%d indice(s) del testigo ausentes' % len(esperados) in out,
           '--ausentes contra el diario ENSENADO: ROJO, nombra los %d' % len(esperados), out[-1500:])
    juntos = escribir(DIR + '/diario-1-y-2.jsonl', l1 + l2)
    rc, out, nombrados = ausentes(dw1 + '/diario.jsonl', juntos)
    exigir(rc == 0 and 'sin ausentes' in out and not nombrados,
           '--ausentes contra los DOS diarios juntos: sin ausentes -la verdad entera no tiene huecos-', out[-800:])

    # ── ESCENA 2 · un nodo honesto, y una linea borrada a mano ──
    P2, N2, VECES = 8815, DIR + '/nodo-2', 8
    # El testigo MUESTREA -una consulta cada 2 s contra un latido de 1 s-: el nodo anota lo que
    # el testigo no pidio, y esa direccion no puede contar como ausencia (BACKLOG 80).
    msg('ESCENA 2: un nodo honesto y su testigo, %d vueltas, una cada 2 s' % VECES)
    n = nodo(N2, '42', P2, 'diario.jsonl')
    exigir(primera_cabeza(P2) is not None, 'el nodo firma')
    w2, dw2 = testigo('testigo-2', P2, VECES, cada=2)
    r2 = esperar(w2, 120)
    matar(n)
    lt, ln = lineas(dw2 + '/diario.jsonl'), lineas(N2 + '/diario.jsonl')
    T, D = firmados(lt), firmados(ln)
    exigir(r2 == 0 and T and T <= D, 'el testigo termina sin hallazgo y todo lo que vio esta en el diario',
           open(dw2 + '/salida.txt').read()[-600:])
    rc, out, nombrados = ausentes(dw2 + '/diario.jsonl', N2 + '/diario.jsonl')
    exigir(rc == 0 and 'sin ausentes' in out,
           '--ausentes contra el diario ENTERO: VERDE, aunque el nodo anote %d indice(s) que el testigo no pidio'
           % len(D - T), out[-800:])
    vistos = sorted(T)
    borrado = vistos[len(vistos) // 2]
    resto = [l for l in ln if not (json.loads(l).get('signature')
                                   and int(json.loads(l).get('index', '0x0'), 16) == borrado)]
    recortado = escribir(DIR + '/diario-recortado.jsonl', resto)
    msg('se borra a mano la linea del indice %d (%d lineas -> %d)' % (borrado, len(ln), len(resto)))
    rc, out, nombrados = ausentes(dw2 + '/diario.jsonl', recortado)
    exigir(rc != 0 and nombrados == [borrado] and '1 indice(s) del testigo ausentes' in out,
           '--ausentes contra el diario RECORTADO: ROJO, nombra el %d y ninguno mas' % borrado, out[-1500:])
finally:
    for p in procesos:
        matar(p)
sys.exit(1 if fallos else 0)
PY
RC=$?
[ "$RC" = 0 ] || fallo "alguna defensa no dijo lo que debia"
[ -z "$(git status --porcelain)" ] || fallo "el banco dejo el arbol sucio: no debe tocarlo"
msg "BANCO-MENT-AUSENTES VERDE"
