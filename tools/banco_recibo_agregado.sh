#!/usr/bin/env bash
# tools/banco_recibo_agregado.sh -- el banco de la E5 del RFC-0014: el recibo del LOTE y el de la
# PRENDA, resueltos por el sobre de COMPLETITUD, en vivo.
#
# Tres nodos REALES que firman, uno tras otro, y el mando (§612, §613) juzgando SIN ellos lo que el
# titular custodia:
#   - EL LOTE (nodo con el grifo `--dev`, latido de 1 s): el recibo de la via directa da el digest
#     de una prueba de CEROS -el de la casa, §116-; un lote de ceros cae al VALIDAR (StaleState) y
#     otro por su FORMA (la misma cuenta dos veces); y el ejemplo `d2_lote_rpc` del sdk paga un LOTE
#     de dos envios con pruebas STARK reales hechas en el cliente e imprime lo que el agregador
#     reenvia -la composicion y la respuesta, con su recibo (§614)-. Salen: RESUELTA como LOTE
#     aplicado -un acuse por prueba, cada uno por `zkssl_ackPath`-, como LOTE rechazado con prueba,
#     como LOTE rechazado por su FORMA, el cuarto estado (DERIVADO por MUTACION de la causa), y dos
#     RECHAZOS SIN FUNDAMENTO, por cuenta y por posicion (DERIVADOS por MUTACION del `data`: un nodo
#     honrado no los produce).
#   - LA PRENDA (dos nodos, latido de 3 s, cada uno con su siembra del sandbox del cli y una prenda
#     DE VERDAD hecha por la boca del prendador): la prenda MUTADA se evalua y se rechaza -RESUELTA
#     como PRENDA rechazada con prueba contra la cabeza JUZGADA-; la buena entra -RESUELTA como PRENDA
#     aceptada, el PAR, con el camino de `zkssl_consumoPath`-; el RECHAZO SIN FUNDAMENTO, DERIVADO
#     por MUTACION de la respuesta aceptada; y en el segundo nodo la boca libre escribe antes un
#     OCUPANTE en la posicion de la marca y la capa rechaza con ConsumoColision: RESUELTA como PRENDA
#     rechazada por la capa, con su sobre de rechazo y el recibo dentro del `data` (§613).
#   - y un negativo por cada regla PRODUCIBLE de las dos resoluciones, derivado por UNA mutacion de
#     lo capturado. No se producen aqui, y se declara: las claves DISTINTAS (el molde es el del
#     banco de completitud) y una resolucion FUERA de la ventana (n = 1.440 cabezas).
#
# FUERA del canon: levanta procesos y espera latidos. NO ESCRIBE EN EL ARBOL: todo vive en un
# temporal bajo $HOME, que borra al salir, y lo comprueba al final por `git status --porcelain`.
#
#   bash tools/banco_recibo_agregado.sh [--guardar <dir>]
#
# --guardar  copia los sobres (positivos y negativos) a <dir>: de ahi salen sus vectores en
#            `spec/vectors/completitud/`, COPIADOS. Dos pruebas del mismo enunciado no dan los
#            mismos bytes (S538): un vector de prenda no se re-deriva, se copia.
set -u
msg(){ echo "BANCO-AGREG| $*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }
GUARDAR=""
while [ $# -gt 0 ]; do
  case "$1" in
    --guardar) GUARDAR="${2:?--guardar exige un directorio}"; mkdir -p "$GUARDAR"; GUARDAR=$(cd "$GUARDAR" && pwd); shift ;;
    *) fallo "uso: bash tools/banco_recibo_agregado.sh [--guardar <dir>]" ;;
  esac
  shift
done
[ -z "$(git status --porcelain)" ] || fallo "el arbol no esta limpio: el banco comprueba al final que no lo toca"
DIR=$(mktemp -d "$HOME/.banco_recibo_agregado.XXXXXX")
trap 'rm -rf "$DIR"' EXIT INT TERM HUP QUIT
msg "compilando cli, nodo, verificador y el ejemplo d2_lote_rpc en RELEASE (aqui se firma y se prueba de verdad)"
{ cargo build --release -q -p zk-ssl-cli -p zk-ssl-node -p zk-ssl-verify \
  && cargo build --release -q -p zk-ssl-sdk --example d2_lote_rpc; } 2>/dev/null \
  || { cargo build --release -p zk-ssl-cli -p zk-ssl-node -p zk-ssl-verify \
       && cargo build --release -p zk-ssl-sdk --example d2_lote_rpc; } || fallo "no compila"

python3 - "$DIR" "${GUARDAR:-}" <<'PY'
import json, os, shutil, subprocess, sys, threading, time, urllib.request
DIR, GUARDAR = sys.argv[1], sys.argv[2]
T = 'target/release/'
CLI, NODO, MANDO, D2 = T + 'zk-ssl-cli', T + 'zk-ssl-node', T + 'zk-ssl-verify', T + 'examples/d2_lote_rpc'
def msg(m): print('BANCO-AGREG| ' + m, file=sys.stderr, flush=True)
fallos = hechos = 0
def mando(nombre, sobre, codigo, texto):
    global fallos, hechos
    hechos += 1
    ruta = '%s/%s.json' % (DIR, nombre)
    json.dump(sobre, open(ruta, 'w'), indent=1, sort_keys=True)
    p = subprocess.run([MANDO, ruta], capture_output=True, text=True)
    out = p.stdout + p.stderr
    ok = p.returncode == codigo and texto in out
    fallos += 0 if ok else 1
    msg('%s %-40s exit %d (se esperaba %d: %s)' % ('OK  ' if ok else 'ROJO', nombre, p.returncode, codigo, texto))
    if not ok:
        msg('     ' + out.strip().replace('\n', '\n     '))
    if GUARDAR:
        shutil.copy(ruta, GUARDAR)
def sin(x, k):
    return {a: b for a, b in x.items() if a != k}
def morir(m):
    msg('ROJO: ' + m); sys.exit(1)

class Nodo:
    def __init__(self, nombre, port, latido, dev=False, ledger=None):
        self.d = '%s/%s' % (DIR, nombre); os.makedirs(self.d, exist_ok=True)
        self.port, self.cabezas = port, {}
        open(self.d + '/semilla.hex', 'w').write('37' * 96); os.chmod(self.d + '/semilla.hex', 0o600)
        args = [NODO, '--listen', '127.0.0.1:%d' % port, '--latido', str(latido),
                '--clave-fichero', self.d + '/semilla.hex', '--custodia', 'fichero',
                '--diario', self.d + '/diario.jsonl', '--ledger', ledger or (self.d + '/ledger'),
                '--contador-recepcion', self.d + '/recepcion.bin',
                '--indice-firma', self.d + '/indice.bin', '--log', 'warn'] + (['--dev'] if dev else [])
        self.p = subprocess.Popen(args, stdout=subprocess.DEVNULL, stderr=open(self.d + '/nodo.err', 'w'))
        for _ in range(240):
            if self.recoger().get('available'): break
            time.sleep(0.25)
        else:
            morir('el nodo %s no firmo' % nombre)
        # Las cabezas se RECOGEN todo el rato, tambien mientras el sdk prueba: la del acuse y la
        # juzgada tienen que estar, y un latido que nadie mira se pierde.
        self.vivo = True
        self.hilo = threading.Thread(target=self.mirar, daemon=True); self.hilo.start()
    def mirar(self):
        while self.vivo:
            self.recoger(); time.sleep(0.2)
    def rpc(self, m, p):
        c = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': m, 'params': p}).encode()
        r = urllib.request.Request('http://127.0.0.1:%d' % self.port, c, {'Content-Type': 'application/json'})
        return json.loads(urllib.request.urlopen(r, timeout=30).read())
    def recoger(self):
        try:
            r = self.rpc('zkssl_signedEpochHead', {}).get('result', {})
        except Exception:
            return {}
        if r.get('available'):
            self.cabezas[int(r['index'], 16)] = r
        return r
    def base(self, rec):
        camino = None
        for _ in range(600):
            self.recoger()
            v = self.rpc('zkssl_recepPath', {'rx': rec['rx']})['result']
            if v.get('available'):
                camino = v; break
            time.sleep(0.1)
        if camino is None: morir('zkssl_recepPath no dio el camino del rx %s' % rec['rx'])
        idx = int(camino['index'], 16)
        for _ in range(100):
            if idx in self.cabezas: break
            time.sleep(0.05); self.recoger()
        cs = dict(self.cabezas)
        cierre = cs[idx]
        ant = [i for i in cs if i < idx]
        q = int(cs[max(ant)]['recepCount'], 16) if ant else 0
        return {'v': 1, 'tipo': 'completitud', 'cierre': cierre, 'recepcion': rec,
                'limiteAnterior': '0x%x' % q, 'camino': camino['camino']}
    def cabeza_de_seq(self, seq):
        for _ in range(300):
            r = self.recoger()
            cs = dict(self.cabezas)
            hay = [i for i in cs if int(cs[i]['seq'], 16) == seq]
            if hay: return cs[min(hay)]
            if r.get('available') and int(r['seq'], 16) > seq and not hay:
                morir('ninguna cabeza firmada capturada con el seq %d' % seq)
            time.sleep(0.1)
        morir('ninguna cabeza firmada llego al seq %d' % seq)
    def cabeza_hasta(self, seq):
        for _ in range(300):
            r = self.recoger()
            if r.get('available') and int(r['seq'], 16) >= seq: return r
            time.sleep(0.2)
        morir('ninguna cabeza firmada llego al seq %d' % seq)
    def juzgada(self, rec):
        e = int(rec['era'], 16)
        if e - 1 not in self.cabezas: morir('no se capturo la cabeza de indice %d, la juzgada' % (e - 1))
        return self.cabezas[e - 1]
    def parar(self):
        self.vivo = False; self.hilo.join()
        self.p.kill(); self.p.wait()

Z = '0x' + '00' * 32
OTRO = '0x' + '11' * 32
cero_pi = {'rootOld': Z, 'rootNew': Z, 'frozenRoot': Z, 'pendingRootOld': Z, 'pendingRootNew': Z,
           'amount': '0x5', 'regulatoryLimit': '0x3e8', 'supplyOld': '0x0', 'supplyNew': '0x0'}
def envio(pos):
    return {'receipt': {'proof': '0x' + '00' * 32, 'publicInputs': cero_pi, 'commitment': Z,
                        'notice': {'position': '0x%x' % pos, 'salt': Z, 'amount': '0x5'}},
            'sender': '0x0', 'senderState': {'publicId': Z, 'balance': '0x5', 'nonce': '0x0'},
            'amount': '0x5'}
vivos = []
try:
    # ============================================================== EL LOTE
    L = Nodo('lote', 8621, 1, dev=True); vivos.append(L)
    h0 = L.rpc('zkssl_applySend', envio(0))['error']['data']['recepcion']['hashPrueba']
    da = L.rpc('zkssl_applyMany', {'ops': [dict(envio(0), kind='send')]})['error']['data']
    db = L.rpc('zkssl_applyMany', {'ops': [dict(envio(0), kind='send'), dict(envio(1), kind='send')]})['error']['data']
    msg('LOTE: uno cae al VALIDAR (%s, operacion %s) y otro por su FORMA (%s, operacion %s), cada uno con su recibo'
        % (da['causa'], da['operacion'], db['causa'], db['operacion']))
    # Sus eras se cierran ANTES de que el sdk mueva el registro, y el sobre de rechazo de una
    # StaleState se juzga sobre la cabeza del MISMO seq que el rechazo (`exige_misma`).
    base_a, base_b = L.base(da['recepcion']), L.base(db['recepcion'])
    rech_a = {'v': 1, 'tipo': 'rechazo', 'data': da, 'cabeza': L.cabeza_de_seq(int(da['seq'], 16)), 'recibo': cero_pi}
    p = subprocess.run([D2, 'http://127.0.0.1:%d' % L.port, '2', '1'], capture_output=True, text=True)
    lin = [l for l in p.stdout.splitlines() if l.startswith('D2: lote-envio ')]
    if p.returncode != 0 or not lin: morir('d2_lote_rpc no pago el lote: ' + (p.stdout + p.stderr)[-800:])
    ap = json.loads(lin[0][len('D2: lote-envio '):])
    comp_ap, res_ap = ap['composicion'], ap['respuesta']
    msg('LOTE: el sdk paga un lote de %d envios con pruebas reales; recibo rx %s'
        % (len(comp_ap), res_ap['recepcion']['rx']))
    acuses = []
    for op, a in zip(comp_ap, res_ap['applied']):
        k = None
        for _ in range(300):
            L.recoger()
            v = L.rpc('zkssl_ackPath', {'seq': a['logSeq']})['result']
            if v.get('available'): k = v; break
            time.sleep(0.1)
        if k is None: morir('zkssl_ackPath no dio el acuse de la entrada %s' % a['logSeq'])
        acuses.append({'cabeza': L.cabeza_de_seq(int(k['s'], 16)),
                       'acuse': {'hashPrueba': op['hashPrueba'], 'seq': a['logSeq'], 'camino': k['camino']}})
    comp_a = [{'hashPrueba': h0, 'cuenta': '0x0', 'posicion': '0x0'}]
    comp_b = [{'hashPrueba': h0, 'cuenta': '0x0', 'posicion': '0x0'},
              {'hashPrueba': h0, 'cuenta': '0x0', 'posicion': '0x1'}]
    base_ap = L.base(res_ap['recepcion'])
    def lote(b, x): return dict(b, resolucion=dict(x, tipo='lote'))
    mando('lote-aplicado', lote(base_ap, {'composicion': comp_ap, 'acuses': acuses}), 0, 'RESUELTA como LOTE aplicado')
    # §682 (RFC-0019 E2): su sobre es un StaleState; el de antes vive en ../0.4/completitud
    mando('lote-rechazado-stale-declarada', lote(base_a, {'composicion': comp_a, 'sobre': rech_a}),
          3, 'DECLARADA, NO PROBADA: resolucion.sobre: el operador resuelve por StaleState')
    mando('lote-por-su-forma', lote(base_b, {'composicion': comp_b, 'data': db}),
          0, 'RESUELTA como LOTE rechazado por su FORMA')
    mando('lote-declarada', lote(base_b, {'composicion': comp_b, 'data': dict(db, causa='NotTheIssuer')}),
          3, 'DECLARADA, NO PROBADA')
    mando('lote-sin-fundamento-cuenta', lote(base_b, {'composicion': comp_b, 'data': dict(db, campos={'index': '0x9'})}),
          1, 'RECHAZO SIN FUNDAMENTO')
    mando('lote-sin-fundamento-posicion', lote(base_b, {'composicion': comp_b, 'data': dict(db, causa='DuplicatePendingInBatch', campos={'position': '0x1'})}),
          1, 'RECHAZO SIN FUNDAMENTO')
    mando('neg-lote-sin-composicion', lote(base_b, {'data': db}), 1, 'falta composicion')
    mando('neg-lote-composicion-vacia', lote(base_b, {'composicion': [], 'data': db}), 1, 'composicion VACIA')
    mando('neg-lote-composicion-sin-cuenta', lote(base_b, {'composicion': [comp_b[0], sin(comp_b[1], 'cuenta')], 'data': db}),
          1, 'resolucion.composicion[1]: falta cuenta')
    mando('neg-lote-composicion-ajena', lote(base_b, {'composicion': comp_b[::-1], 'data': db}),
          1, 'la composicion NO es la del recibo')
    mando('neg-lote-una-de-tres', lote(base_a, {'composicion': comp_a, 'sobre': rech_a, 'data': da}), 1, 'lleva UNA de tres')
    mando('neg-lote-acuses-de-menos', lote(base_ap, {'composicion': comp_ap, 'acuses': acuses[:-1]}),
          1, 'acuse(s) para un lote de %d' % len(comp_ap))
    mando('neg-lote-acuse-de-otra-prueba', lote(base_ap, {'composicion': comp_ap, 'acuses': acuses[::-1]}),
          1, 'resolucion.acuses[0]: el acuse es de OTRA prueba')
    mando('neg-lote-acuse-sin-cabeza', lote(base_ap, {'composicion': comp_ap, 'acuses': [sin(acuses[0], 'cabeza')] + acuses[1:]}),
          1, 'resolucion.acuses[0]: falta cabeza')
    mando('neg-lote-operacion-fuera', lote(base_a, {'composicion': comp_a, 'sobre': dict(rech_a, data=dict(da, operacion='0x1'))}),
          1, 'la operacion 1 no esta en un lote de 1')
    mando('neg-lote-sin-operacion', lote(base_b, {'composicion': comp_b, 'data': sin(db, 'operacion')}), 1, 'nombra su operacion')
    mando('neg-lote-data-de-otra-operacion', lote(base_b, {'composicion': comp_b, 'data': dict(db, recepcion=dict(db['recepcion'], hashPrueba=OTRO))}),
          1, 'resolucion.data: es de OTRA operacion')
    mando('neg-lote-sin-campos', lote(base_b, {'composicion': comp_b, 'data': sin(db, 'campos')}), 1, 'resolucion.data: falta campos')
    mando('neg-lote-causa-con-prueba', lote(base_b, {'composicion': comp_b, 'data': dict(db, causa='StaleState')}),
          1, 'se exhibe su sobre de rechazo')
    L.parar(); vivos.remove(L)

    # ============================================================== LA PRENDA
    def sembrar(nombre):
        d = '%s/%s' % (DIR, nombre); os.makedirs(d)
        open(d + '/frase.txt', 'w').write('la frase del banco del recibo agregado\n'); os.chmod(d + '/frase.txt', 0o600)
        p = subprocess.run([CLI, '--log', 'warn', 'simulate', '--ledger', d + '/ledger', '--no-claim', '--v2',
                            '--aviso', d + '/aviso.json', '--credencial', d + '/credencial.json',
                            '--retorno', d + '/retorno.json', '--credencial-pagador', d + '/credencial-pagador.json',
                            '--keystore', d + '/keystore.json', '--frase-fichero', d + '/frase.txt'],
                           capture_output=True, text=True)
        if p.returncode != 0: morir('la siembra de %s fallo: %s' % (nombre, (p.stdout + p.stderr)[-800:]))
        return d
    def boca(n, d, salida):
        ix = json.load(open(d + '/credencial.json'))['index']
        for _ in range(6):
            p = subprocess.run([CLI, '--log', 'warn', 'prueba-prenda', '--nodo', 'http://127.0.0.1:%d' % n.port,
                                '--aviso', d + '/aviso.json', '--keystore', d + '/keystore.json',
                                '--frase-fichero', d + '/frase.txt', '--index', ix, '--salida', salida],
                               capture_output=True, text=True)
            out = p.stdout + p.stderr
            if 'cayo un latido entre las dos llamadas' in out or 'la ultima cabeza firmada va por el' in out:
                time.sleep(2); continue
            if p.returncode != 0: morir('la boca del prendador fallo: ' + out[-600:])
            return json.load(open(salida))
        morir('la boca perdio seis carreras con el latido')
    def prendar(n, s):
        n.recoger()
        r = n.rpc('zkssl_pledge', {'prueba': s['prueba'], 'receptor': s['enunciado']['receptor'],
                                   'marca': s['enunciado']['marca'], 'seq': s['cabeza']['seq']})['result']
        n.recoger()
        return r
    def prenda(b, x): return dict(b, resolucion=dict(x, tipo='prenda'))

    da_ = sembrar('prenda-a')
    A = Nodo('prenda-a', 8623, 3, ledger=da_ + '/ledger'); vivos.append(A)
    sobre = boca(A, da_, da_ + '/prenda.json')
    malo = dict(sobre); pr = bytearray.fromhex(sobre['prueba'][2:]); pr[len(pr) // 2] ^= 1
    malo['prueba'] = '0x' + pr.hex()
    r_malo = prendar(A, malo)
    if r_malo.get('accepted') is not False or 'recepcion' not in r_malo:
        morir('la prenda mutada no dio negativa con recibo: %s' % json.dumps(r_malo)[:300])
    r_bueno = prendar(A, sobre)
    if r_bueno.get('accepted') is not True or 'recepcion' not in r_bueno:
        morir('la prenda buena no entro con recibo: %s' % json.dumps(r_bueno)[:300])
    msg('PRENDA: la MUTADA se evalua y se rechaza (rx %s); la BUENA entra (rx %s)'
        % (r_malo['recepcion']['rx'], r_bueno['recepcion']['rx']))
    jz_malo, jz_bueno = A.juzgada(r_malo['recepcion']), A.juzgada(r_bueno['recepcion'])
    cab = A.cabeza_hasta(int(r_bueno['logSeq'], 16))
    cp = A.rpc('zkssl_consumoPath', {'consumo': sobre['enunciado']['marca'], 'seq': cab['seq']})['result']
    if not cp.get('available'): morir('sin camino del consumo: %s' % cp)
    b_malo, b_bueno = A.base(r_malo['recepcion']), A.base(r_bueno['recepcion'])
    consumo = {'cabeza': cab, 'camino': cp['camino']}
    negada = {'accepted': False, 'reason': r_malo['reason'], 'recepcion': r_bueno['recepcion']}
    mando('prenda-aceptada', prenda(b_bueno, {'sobre': sobre, 'consumo': consumo}), 0, 'RESUELTA como PRENDA aceptada')
    mando('prenda-rechazada-con-prueba', prenda(b_malo, {'sobre': malo, 'respuesta': r_malo, 'juzgada': jz_malo}),
          0, 'RESUELTA como PRENDA rechazada con prueba')
    mando('prenda-sin-fundamento', prenda(b_bueno, {'sobre': sobre, 'respuesta': negada, 'juzgada': jz_bueno}),
          1, 'RECHAZO SIN FUNDAMENTO')
    cs = dict(A.cabezas)
    otra = max(i for i in cs if i != int(jz_bueno['index'], 16))
    mando('neg-prenda-juzgada-otra', prenda(b_bueno, {'sobre': sobre, 'respuesta': negada, 'juzgada': cs[otra]}),
          1, 'no es el de la cabeza que el nodo juzgo')
    mando('neg-prenda-sin-juzgada', prenda(b_bueno, {'sobre': sobre, 'respuesta': negada}), 1, 'falta juzgada')
    mando('neg-prenda-respuesta-aceptada', prenda(b_bueno, {'sobre': sobre, 'respuesta': r_bueno, 'juzgada': jz_bueno}),
          1, 'no es una negativa')
    mando('neg-prenda-de-otra-prueba', prenda(b_bueno, {'sobre': malo, 'consumo': consumo}), 1, 'la prenda es de OTRA prueba')
    mando('neg-prenda-negativa-de-otro-recibo', prenda(b_malo, {'sobre': malo, 'respuesta': negada, 'juzgada': jz_malo}),
          1, 'resolucion.respuesta: es de OTRA operacion')
    mando('neg-prenda-sin-sobre', prenda(b_bueno, {'consumo': consumo}), 1, 'resolucion: falta sobre (el de la prenda')
    mando('neg-prenda-sobre-no-prenda', prenda(b_bueno, {'sobre': dict(sobre, tipo='rechazo'), 'consumo': consumo}),
          1, 'el sobre no es de tipo prenda')
    mando('neg-prenda-una-de-tres', prenda(b_bueno, {'sobre': sobre}), 1, 'lleva UNA de tres')
    mando('neg-prenda-consumo-sin-camino', prenda(b_bueno, {'sobre': sobre, 'consumo': sin(consumo, 'camino')}),
          1, 'falta resolucion.consumo.camino')
    cam = dict(cp['camino'], isRight=[not x for x in cp['camino']['isRight']])
    mando('neg-prenda-consumo-otra-posicion', prenda(b_bueno, {'sobre': sobre, 'consumo': dict(consumo, camino=cam)}),
          1, 'el isRight recibido NO es el de la posicion')
    mando('neg-prenda-consumo-no-publicado', prenda(b_bueno, {'sobre': sobre, 'consumo': dict(consumo, cabeza=jz_bueno)}),
          1, 'la marca NO esta bajo el consRoot de su cabeza')
    A.parar(); vivos.remove(A)

    db_ = sembrar('prenda-b')
    B = Nodo('prenda-b', 8625, 3, ledger=db_ + '/ledger'); vivos.append(B)
    s1 = boca(B, db_, db_ + '/prenda1.json')
    marca = s1['enunciado']['marca']
    ocupante = marca[:18] + '07' + '00' * 7 + '08' + '00' * 7 + '09' + '00' * 7
    if ocupante == marca: morir('el ocupante salio igual a la marca')
    pub = B.rpc('zkssl_publishConsumo', {'consumo': ocupante})['result']
    if pub.get('accepted') is not True: morir('el ocupante no entro: %s' % pub)
    h_oc = B.cabeza_hasta(int(pub['logSeq'], 16))
    s2 = boca(B, db_, db_ + '/prenda2.json')
    r_col = prendar(B, s2)
    if r_col.get('accepted') is not False or r_col.get('data', {}).get('causa') != 'ConsumoColision':
        morir('la prenda no cayo por colision: %s' % json.dumps(r_col)[:400])
    if r_col['data'].get('recepcion') != r_col.get('recepcion'): morir('el data de la colision no lleva su recibo')
    msg('PRENDA: con un OCUPANTE en la posicion de su marca, la capa la rechaza con ConsumoColision (rx %s)'
        % r_col['recepcion']['rx'])
    pres = B.rpc('zkssl_consumoPath', {'consumo': ocupante, 'seq': h_oc['seq']})['result']
    b_col = B.base(r_col['recepcion'])
    rech = {'v': 1, 'tipo': 'rechazo', 'data': r_col['data'], 'cabeza': h_oc, 'presencia': pres['camino']}
    mando('prenda-rechazada-por-la-capa', prenda(b_col, {'sobre': s2, 'rechazo': rech}),
          0, 'RESUELTA como PRENDA rechazada por la capa')
    mando('neg-prenda-colision-como-negativa', prenda(b_col, {'sobre': s2, 'respuesta': r_col, 'juzgada': B.juzgada(r_col['recepcion'])}),
          1, 'la negativa lleva causa')
    mando('neg-prenda-rechazo-de-otro-recibo', prenda(b_col, {'sobre': s2, 'rechazo': dict(rech, data=dict(r_col['data'], recepcion=r_bueno['recepcion']))}),
          1, 'resolucion.rechazo: es de OTRA operacion')
    mando('neg-prenda-tipo-desconocido', dict(b_col, resolucion={'tipo': 'prendas'}), 1, 'desconocido: se lee acuse, rechazo o declarada')
finally:
    for n in vivos:
        n.parar()
msg(('VERDE: %d sobres, todos dicen lo que deben' % hechos) if fallos == 0 else ('ROJO: %d de %d sobres no dicen lo que deben' % (fallos, hechos)))
sys.exit(1 if fallos else 0)
PY
rc=$?
[ -z "$(git status --porcelain)" ] || fallo "el banco ENSUCIO el arbol"
[ "$rc" = 0 ] || fallo "el mando no dijo lo que debia en todos los sobres"
msg "VERDE: el arbol, intacto"
