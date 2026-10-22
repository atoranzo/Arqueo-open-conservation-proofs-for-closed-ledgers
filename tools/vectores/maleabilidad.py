"""§715: los vectores negativos de la maleabilidad de los bytes de la prueba.

Cada uno sale de un vector positivo de `spec/vectors/` por UNA mutacion de su prueba, y se
escribe con el formato de los negativos de su familia (`json.dumps`, sin salto final). Los
escribe en `<salida>`; copiarlos a `spec/vectors/<familia>/` es un paso aparte, a mano, y sus
lineas van AL FINAL del MANIFIESTO (nunca se reescribe un vector ni una linea).

Un lector que recuerda cuantos bytes ocupa cada `usize` de la prueba, y que escribe lo que lee:
cada mutacion cambia UNA cosa y deja el resto byte a byte. Sale de `edad-todos.json` y de
`pago-t-frontera.json` (en el pago, sin la fila de mas, que no cabe).

uso: python3 -I tools/vectores/maleabilidad.py <arbol> <salida>
"""
import copy
import json
import os
import struct
import sys

DIG = 32
GRUPO = 8 * 16  # el plegado de la casa (8) en la extension cuadratica (16 bytes)


def largo_minimo(v):
    for n in range(1, 9):
        if v < (1 << (7 * n)):
            return n
    return 9


class R:
    def __init__(s, b):
        s.b, s.i = b, 0

    def tomar(s, n):
        assert s.i + n <= len(s.b), 'EOF'
        s.i += n
        return s.b[s.i - n:s.i]

    def u8(s):
        return s.tomar(1)[0]

    def u16(s):
        return struct.unpack('<H', s.tomar(2))[0]

    def u32(s):
        return struct.unpack('<I', s.tomar(4))[0]

    def vint(s):
        f = s.b[s.i]
        n = (f & -f).bit_length() if f else 9
        if n == 9:
            s.tomar(1)
            v = struct.unpack('<Q', s.tomar(8))[0]
        else:
            v = int.from_bytes(s.tomar(n), 'little') >> n
        return {'v': v, 'de_mas': n - largo_minimo(v)}


def vint_bytes(x):
    v = x['v']
    n = min(9, largo_minimo(v) + x['de_mas'])
    if n == 9:
        return b'\0' + struct.pack('<Q', v)
    return ((v << n) | (1 << (n - 1))).to_bytes(n, 'little')


def leer_lote(b):
    r = R(b)
    lote = {'depth': r.u8(), 'vectores': r.vint(), 'nodos': []}
    for _ in range(lote['vectores']['v']):
        k = r.vint()
        lote['nodos'].append({'k': k, 'n': [r.tomar(DIG) for _ in range(k['v'])]})
    lote['sales'] = [r.tomar(DIG) for _ in range(r.u32())]
    lote['cola'] = b[r.i:]
    assert not lote['cola'], 'cola en el lote'
    return lote


def escribir_lote(l):
    l['vectores']['v'] = len(l['nodos'])
    o = bytes([l['depth']]) + vint_bytes(l['vectores'])
    for v in l['nodos']:
        v['k']['v'] = len(v['n'])
        o += vint_bytes(v['k']) + b''.join(v['n'])
    return o + struct.pack('<I', len(l['sales'])) + b''.join(l['sales']) + l.get('cola', b'')


def leer(b):
    r = R(b)
    t = {}
    i = 6 + struct.unpack_from('<H', b, 4)[0]
    i += 1 + b[i] + 10
    t['cabeza'] = r.tomar(i)
    t['restricciones'] = r.vint()
    t['unicas'] = r.u8()
    t['compromisos'] = r.tomar(r.u16())
    t['consultas'] = []
    for _ in range(2 + (1 if b[1] else 0)):
        lv = r.vint()
        valores = r.tomar(lv['v'])
        la = r.vint()
        t['consultas'].append({'lv': lv, 'valores': valores, 'la': la, 'lote': leer_lote(r.tomar(la['v']))})
    t['ood_traza'] = r.tomar(r.u16())
    t['ood_cociente'] = r.tomar(r.u16())
    t['capas'] = []
    for _ in range(r.u8()):
        valores = r.tomar(r.u32())
        t['capas'].append({'valores': valores, 'lote': leer_lote(r.tomar(r.u32()))})
    t['resto'] = r.tomar(r.u16())
    t['particiones'] = r.u8()
    t['nonce'] = r.tomar(8)
    t['cola'] = b[r.i:]
    return t


def escribir(t):
    o = t['cabeza'] + vint_bytes(t['restricciones']) + bytes([t['unicas']])
    o += struct.pack('<H', len(t['compromisos'])) + t['compromisos']
    for c in t['consultas']:
        c['lv']['v'] = len(c['valores'])
        lote = escribir_lote(c['lote'])
        c['la']['v'] = len(lote)
        o += vint_bytes(c['lv']) + c['valores'] + vint_bytes(c['la']) + lote
    o += struct.pack('<H', len(t['ood_traza'])) + t['ood_traza']
    o += struct.pack('<H', len(t['ood_cociente'])) + t['ood_cociente']
    o += bytes([len(t['capas'])])
    for c in t['capas']:
        lote = escribir_lote(c['lote'])
        o += struct.pack('<I', len(c['valores'])) + c['valores'] + struct.pack('<I', len(lote)) + lote
    o += struct.pack('<H', len(t['resto'])) + t['resto'] + bytes([t['particiones']]) + t['nonce']
    return o + t['cola']


# ── las mutaciones: cada una cambia UNA cosa ──

def usize_no_minimo(t):  # el numero de restricciones con un byte de mas
    t['restricciones']['de_mas'] = 1


def lote_usize_no_minimo(t):  # la cuenta de nodos del primer vector del lote de la traza
    t['consultas'][0]['lote']['nodos'][0]['k']['de_mas'] = 1


def fri_usize_no_minimo(t):  # la cuenta de nodos del primer vector del lote de la capa FRI 0
    t['capas'][0]['lote']['nodos'][0]['k']['de_mas'] = 1


def restricciones_alias(t):  # n + 2^32, en su escritura minima: el mismo `as u32`
    t['restricciones'] = {'v': t['restricciones']['v'] + 2**32, 'de_mas': 0}


def capa_fri_de_mas(t):  # una capa minima: un grupo y un lote vacio del dominio siguiente
    ult = t['capas'][-1]
    t['capas'].append({'valores': ult['valores'][:GRUPO],
                       'lote': {'depth': ult['lote']['depth'] - 3,
                                'vectores': {'v': 0, 'de_mas': 0}, 'nodos': [], 'sales': []}})


def grupo_fri_de_mas(t):  # una copia del primer grupo de la capa 0, con su sal
    c = t['capas'][0]
    c['valores'] += c['valores'][:GRUPO]
    c['lote']['sales'].append(c['lote']['sales'][0])


def fila_de_mas(t):  # una consulta unica mas, y una copia de la primera fila de cada tabla con su sal
    assert t['unicas'] < 42, 'no cabe una fila de mas: las unicas ya son las 42 de las opciones'
    for c in t['consultas']:
        fila = len(c['valores']) // t['unicas']
        c['valores'] += c['valores'][:fila]
        c['lote']['sales'].append(c['lote']['sales'][0])
    t['unicas'] += 1


def nodo_de_mas(t):  # §659: una copia del primer nodo al final del primer vector del lote de la traza
    v = t['consultas'][0]['lote']['nodos'][0]
    v['n'].append(v['n'][0])


def cola(t):  # §653: un byte de cola
    t['cola'] = b'\0'


def cola_apertura_traza(t):  # un byte de cola DENTRO de la apertura de la traza (su longitud, +1)
    t['consultas'][0]['lote']['cola'] = b'\0'


def cola_apertura_cociente(t):  # el mismo, dentro de la apertura del cociente
    t['consultas'][-1]['lote']['cola'] = b'\0'


def cola_lote_fri(t):  # el mismo, dentro del lote de la capa FRI 0
    t['capas'][0]['lote']['cola'] = b'\0'


def particiones_cero(t):  # las particiones de las opciones (su noveno byte) a cero
    c = bytearray(t['cabeza'])
    c[-2] = 0
    t['cabeza'] = bytes(c)


def marco_ood_3(t):  # el marco OOD de la traza con 3 filas
    t['ood_traza'] = b'\x03' + t['ood_traza'][1:]


def fri_particiones_64(t):  # las particiones FRI: 2^64
    t['particiones'] = 64


MALEABLES = ['usize-no-minimo', 'lote-usize-no-minimo', 'fri-usize-no-minimo', 'restricciones-alias',
             'capa-fri-de-mas', 'grupo-fri-de-mas', 'fila-de-mas', 'nodo-de-mas', 'cola']
PANICOS = ['particiones-cero', 'marco-ood-3', 'fri-particiones-64']
# la cola DENTRO de cada apertura y del lote FRI (la segunda implementacion las aceptaba y el kit
# no). Van detras: los vectores de las listas de arriba no cambian.
COLAS = ['cola-apertura-traza', 'cola-apertura-cociente', 'cola-lote-fri']
PLAN = [('edad', 'edad-todos.json', MALEABLES + PANICOS + COLAS),
        ('pago', 'pago-t-frontera.json', [m for m in MALEABLES if m != 'fila-de-mas'] + COLAS)]


def main():
    arbol, salida = sys.argv[1:3]
    for familia, base, mutaciones in PLAN:
        d = json.load(open(os.path.join(arbol, 'spec', 'vectors', familia, base)))
        b = bytes.fromhex(d['prueba'][2:])
        t0 = leer(b)
        assert escribir(copy.deepcopy(t0)) == b, f'{base}: el lector escribe lo que lee'
        os.makedirs(os.path.join(salida, familia), exist_ok=True)
        for m in mutaciones:
            t = copy.deepcopy(t0)
            globals()[m.replace('-', '_')](t)
            nb = escribir(t)
            assert nb != b
            ruta = os.path.join(salida, familia, f'neg-prueba-{m}.json')
            with open(ruta, 'w') as f:
                f.write(json.dumps(dict(d, prueba='0x' + nb.hex())))
            print(f'{familia}/neg-prueba-{m}.json\t{len(b)} -> {len(nb)} bytes')


if __name__ == '__main__':
    main()
