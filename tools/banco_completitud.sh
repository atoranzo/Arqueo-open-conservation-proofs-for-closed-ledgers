#!/usr/bin/env bash
# tools/banco_completitud.sh -- el banco de la E5 del RFC-0010: el sobre de COMPLETITUD, en vivo.
#
# Nodo REAL firmando latidos (uno por segundo) -> un envio con prueba de CEROS que la capa RECHAZA
# (StaleState): su recibo de recepcion viaja en el error (§571) -> el latido cierra su era en una
# cabeza v6 firmada (§570) -> zkssl_recepPath da el camino (§571) -> el titular arma sus sobres con
# lo que custodia, y el mando (§573) los juzga SIN el nodo:
#   - RESUELTA COMO RECHAZO CON PRUEBA: el sobre de rechazo se arma con el error.data del cable -con
#     su recepcion, que lo ata al recibo (D3)-, la cabeza de su seq y los publicInputs del envio;
#   - DECLARADA, NO PROBADA: el mismo data, con una causa de las que el RFC-0007 dejo sin prueba
#     portable (derivada por MUTACION: el nodo no produce hoy esas causas por esta via), salida 3;
#   - con --largo, NO RESUELTA EN LA VENTANA: se espera a la cabeza de indice era + n + 1 -con
#     n = 1.440 son unos 24 minutos a un latido por segundo- y el sobre sin resolucion sale ROJO
#     NOMBRADO, que es el producto del hito;
#   - y un negativo por cada regla PRODUCIBLE del sobre, derivado por UNA mutacion de lo capturado:
#     treinta siempre, y con --largo dos mas, los de una resolucion FUERA de la ventana. Las tres
#     de las claves DISTINTAS se producen con la cabeza de un SEGUNDO nodo, de otra semilla, que se
#     levanta un momento y se para.
# El veredicto 1 (resuelta como transicion APLICADA) NO se siembra aqui: pide una prueba STARK real
# de un envio aplicado, y su verificacion es la del paquete de posicion, que tiene sus vectores. Lo
# siembra desde el §605 `tools/banco_mentiroso_sin_resolver.sh --guardar`, y su vector es el cuarto
# positivo de `spec/vectors/completitud/`.
#
# FUERA del canon: levanta procesos y espera latidos. NO ESCRIBE EN EL ARBOL: todo vive en un
# temporal bajo $HOME, que borra al salir, y lo comprueba al final por `git status --porcelain`.
#
#   bash tools/banco_completitud.sh [--largo] [--guardar <dir>]
#
# --largo    espera a que la ventana EXPIRE y siembra el veredicto 3 (unos 24 minutos).
# --guardar  copia los sobres (positivos y negativos) a <dir>: de ahi sale el catalogo, COPIADO.
set -u
msg(){ echo "BANCO-COMPL| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
LARGO=0; GUARDAR=""
while [ $# -gt 0 ]; do
  case "$1" in
    --largo) LARGO=1 ;;
    --guardar) GUARDAR="${2:?--guardar exige un directorio}"; mkdir -p "$GUARDAR"; shift ;;
    *) fallo "uso: bash tools/banco_completitud.sh [--largo] [--guardar <dir>]" ;;
  esac
  shift
done
[ -z "$(git status --porcelain)" ] || fallo "el arbol no esta limpio: el banco comprueba al final que no lo toca"
DIR=$(mktemp -d "$HOME/.banco_completitud.XXXXXX")
trap 'rm -rf "$DIR"' EXIT INT TERM HUP QUIT
msg "compilando nodo y verificador en RELEASE (aqui se firma de verdad)"
cargo build --release -q -p zk-ssl-node -p zk-ssl-verify 2>/dev/null \
  || cargo build --release -p zk-ssl-node -p zk-ssl-verify || fallo "no compila"

python3 - "$DIR" "$LARGO" "${GUARDAR:-}" <<'PY'
import json, os, shutil, subprocess, sys, time, urllib.request
DIR, LARGO, GUARDAR = sys.argv[1], sys.argv[2] == '1', sys.argv[3]
NODO, MANDO = 'target/release/zk-ssl-node', 'target/release/zk-ssl-verify'
def msg(m): print('BANCO-COMPL| ' + m, file=sys.stderr, flush=True)
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
P, P_AJENO = 8796, 8797
cabezas = {}
def recoger():
    r = rpc(P, 'zkssl_signedEpochHead', {}).get('result', {})
    if r.get('available'):
        cabezas[int(r['index'], 16)] = r
    return r
Z = '0x' + '00' * 32
OTRO = '0x' + '11' * 32
# El envio de CEROS: la forma del cable acepta sus bytes y la capa lo rechaza. Es el mismo que
# el testigo del nodo `envio_de_ceros` arma en Rust.
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
def sin(d, k):
    return {x: y for x, y in d.items() if x != k}
# La cabeza de OTRA clave: un segundo nodo, de otra semilla, que firma una vez y se para.
ajeno = levantar('ajeno', '38', P_AJENO)
try:
    cabeza_ajena = primera_cabeza(P_AJENO)
finally:
    ajeno.kill(); ajeno.wait()
msg('la cabeza de OTRA clave: indice %s, del segundo nodo, ya parado' % cabeza_ajena['index'])
nodo = levantar('nodo', '37', P)
try:
    primera_cabeza(P)
    r = rpc(P, 'zkssl_applySend', envio)
    if 'error' not in r or 'recepcion' not in r['error'].get('data', {}):
        msg('ROJO: el envio de ceros no dejo recibo en el error: %s' % json.dumps(r)[:300]); sys.exit(1)
    data = r['error']['data']
    rec = data['recepcion']
    era, n = int(rec['era'], 16), int(rec['n'], 16)
    msg('el envio se RECHAZA (%s) y su recibo viaja en el error: rx %s, era %d' % (data['causa'], rec['rx'], era))
    camino = None
    for _ in range(300):
        recoger()
        v = rpc(P, 'zkssl_recepPath', {'rx': rec['rx']})['result']
        if v.get('available'):
            camino = v; break
        time.sleep(0.1)
    if camino is None:
        msg('ROJO: zkssl_recepPath no dio el camino en 30 s'); sys.exit(1)
    idx = int(camino['index'], 16)
    for _ in range(50):
        if idx in cabezas: break
        time.sleep(0.05); recoger()
    cierre = cabezas[idx]
    ant = [i for i in cabezas if i < idx]
    q = int(cabezas[max(ant)]['recepCount'], 16) if ant else 0
    msg('la cabeza firmada de indice %d cierra su era (Q %d, R %s)' % (idx, q, cierre['recepCount']))
    base = {'v': 1, 'tipo': 'completitud', 'cierre': cierre, 'recepcion': rec,
            'limiteAnterior': '0x%x' % q, 'camino': camino['camino']}
    sobre_rechazo = {'v': 1, 'tipo': 'rechazo', 'data': data, 'cabeza': cierre, 'recibo': cero_pi}
    declarada = dict(data, causa='NotTheIssuer')
    acuse = {'seq': cierre['seq'], 'hashPrueba': rec['hashPrueba'], 'camino': camino['camino']}
    def res(x):
        return dict(base, resolucion=x)
    # los dos veredictos que no esperan
    mando('resuelta-por-rechazo', res({'tipo': 'rechazo', 'sobre': sobre_rechazo}),
          0, 'VERDE: el recibo se resolvio dentro de la ventana, y se sostiene sin el nodo')
    mando('declarada', res({'tipo': 'declarada', 'data': declarada}), 3, 'DECLARADA, NO PROBADA')
    # el cierre
    mando('neg-sin-cierre', sin(base, 'cierre'), 1, 'falta cierre')
    mando('neg-cierre-formatVersion-7', dict(base, cierre=dict(cierre, formatVersion='0x7')),
          1, 'el sobre de completitud lee cabezas')
    mando('neg-cierre-v5', dict(base, cierre=dict(cierre, formatVersion='0x5')), 1, 'exige una cabeza v6')
    mando('neg-cierre-recepRoot-mentida', dict(base, cierre=dict(cierre, recepRoot=OTRO)),
          1, 'cierre: los campos NO recomponen su epochDigest')
    # el recibo y su era
    mando('neg-sin-recepcion', sin(base, 'recepcion'), 1, 'falta recepcion')
    mando('neg-n-mentida', dict(base, recepcion=dict(rec, n='0x%x' % (n + 1))), 1, 'no es la que el cierre firma')
    mando('neg-q-mentido', dict(base, limiteAnterior=rec['rx']), 1, 'no pertenece a la era')
    # el camino
    mando('neg-sin-camino', sin(base, 'camino'), 1, 'falta camino')
    cam = camino['camino']
    mando('neg-camino-truncado', dict(base, camino={'siblings': cam['siblings'][:-1], 'isRight': cam['isRight'][:-1]}),
          1, 'no mide 32 niveles')
    mando('neg-era-mentida', dict(base, recepcion=dict(rec, era='0x%x' % (era + 1))), 1, 'NO sube a la recepRoot')
    # sin resolucion: la ventana
    mando('neg-sin-resolucion-ni-vigente', base, 1, 'sin resolucion y sin cabeza vigente')
    mando('neg-ventana-abierta', dict(base, vigente=cierre), 1, 'ventana ABIERTA')
    mando('neg-vigente-de-otra-clave', dict(base, vigente=cabeza_ajena), 1, 'claves DISTINTAS')
    # la resolucion por acuse
    mando('neg-acuse-sin-cabeza', res({'tipo': 'acuse', 'acuse': acuse}), 1, 'resolucion: falta cabeza')
    mando('neg-acuse-sin-acuse', res({'tipo': 'acuse', 'cabeza': cierre}), 1, 'resolucion: falta acuse')
    mando('neg-acuse-de-otra-clave', res({'tipo': 'acuse', 'cabeza': cabeza_ajena, 'acuse': acuse}),
          1, 'claves DISTINTAS')
    mando('neg-acuse-de-otra-prueba', res({'tipo': 'acuse', 'cabeza': cierre, 'acuse': dict(acuse, hashPrueba=OTRO)}),
          1, 'el acuse es de OTRA prueba')
    # la resolucion por rechazo
    mando('neg-rechazo-sin-sobre', res({'tipo': 'rechazo'}), 1, 'resolucion: falta sobre')
    mando('neg-rechazo-de-otro-tipo', res({'tipo': 'rechazo', 'sobre': dict(sobre_rechazo, tipo='prenda')}),
          1, 'el sobre no es de tipo rechazo')
    mando('neg-rechazo-sin-data', res({'tipo': 'rechazo', 'sobre': sin(sobre_rechazo, 'data')}),
          1, 'resolucion.sobre: falta data')
    mando('neg-rechazo-data-sin-recepcion', res({'tipo': 'rechazo', 'sobre': dict(sobre_rechazo, data=sin(data, 'recepcion'))}),
          1, 'resolucion.sobre: su data no lleva recepcion')
    otra = dict(data, recepcion=dict(rec, hashPrueba=OTRO))
    mando('neg-rechazo-de-otra-operacion', res({'tipo': 'rechazo', 'sobre': dict(sobre_rechazo, data=otra)}),
          1, 'resolucion.sobre: es de OTRA operacion')
    mando('neg-rechazo-sin-cabeza', res({'tipo': 'rechazo', 'sobre': sin(sobre_rechazo, 'cabeza')}),
          1, 'resolucion.sobre: falta cabeza')
    mando('neg-rechazo-de-otra-clave', res({'tipo': 'rechazo', 'sobre': dict(sobre_rechazo, cabeza=cabeza_ajena)}),
          1, 'claves DISTINTAS')
    # la resolucion declarada
    mando('neg-declarada-sin-data', res({'tipo': 'declarada'}), 1, 'resolucion: falta data')
    mando('neg-declarada-sin-causa', res({'tipo': 'declarada', 'data': sin(declarada, 'causa')}),
          1, 'resolucion.data: falta causa')
    mando('neg-declarada-con-prueba', res({'tipo': 'declarada', 'data': data}),
          1, 'no es de las que el RFC-0007 dejo sin prueba portable')
    mando('neg-declarada-data-sin-recepcion', res({'tipo': 'declarada', 'data': sin(declarada, 'recepcion')}),
          1, 'resolucion.data: su data no lleva recepcion')
    mando('neg-declarada-de-otra-operacion', res({'tipo': 'declarada', 'data': dict(declarada, recepcion=otra['recepcion'])}),
          1, 'resolucion.data: es de OTRA operacion')
    mando('neg-resolucion-desconocida', res({'tipo': 'otra'}), 1, 'desconocido: se lee acuse, rechazo o declarada')
    if LARGO:
        objetivo = era + n + 1
        msg('--largo: esperando la cabeza de indice %d (la ventana expira); a un latido por segundo' % objetivo)
        while True:
            c = recoger()
            if int(c['index'], 16) >= objetivo:
                break
            time.sleep(2)
        mando('no-resuelta', dict(base, vigente=c), 1, 'NO RESUELTA EN LA VENTANA')
        mando('neg-acuse-fuera-de-ventana', res({'tipo': 'acuse', 'cabeza': c, 'acuse': acuse}),
              1, 'resolucion: llega FUERA de la ventana')
        mando('neg-rechazo-fuera-de-ventana', res({'tipo': 'rechazo', 'sobre': dict(sobre_rechazo, cabeza=c)}),
              1, 'resolucion: llega FUERA de la ventana')
finally:
    nodo.kill(); nodo.wait()
sys.exit(1 if fallos else 0)
PY
RC=$?
[ "$RC" = 0 ] || fallo "algun sobre no dijo lo que debia"
[ -z "$(git status --porcelain)" ] || fallo "el banco dejo el arbol sucio: no debe tocarlo"
[ -n "$GUARDAR" ] && msg "sobres guardados en $GUARDAR (de ahi sale el catalogo, COPIADO)"
msg "BANCO-COMPLETITUD VERDE"
