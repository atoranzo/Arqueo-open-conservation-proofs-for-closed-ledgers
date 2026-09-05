#!/usr/bin/env python3
# ── EL CENSO DEL NUCLEO (RFC-0005, E1, §407) ─────────────────────────
# Deriva la superficie ALCANZABLE de zk-ssl-verify y los `pub` de zk-ssl-hash
# desde el fuente, y la compara con la tabla de spec/NUCLEO.md. Reglas,
# declaradas ANTES de mirar (molde: la R5 de check_dominios.py):
#
#  R1  todo elemento derivado tiene EXACTAMENTE UNA fila en la tabla, con el
#      mismo fichero; un elemento sin fila es un pub que nadie ha clasificado.
#  R2  toda fila de la tabla existe en el arbol como elemento derivado; una
#      fila sin elemento es prosa que envejecio.
#  R3  la clase de cada fila es una de las cuatro: NUCLEO, REFERENCIA, LIBRO,
#      REGISTRO. Otra palabra es rojo: la clasificacion no admite un quinto
#      cajon sin RFC.
#  R4  los totales que el documento DECLARA (por crate y por clase) son los
#      derivados: un contador declarado que no se cruza con el derivado es
#      un numero tecleado (PRECISION 16).
#
# Como se deriva (el mismo instrumento que el PASTE-407-M2 de la sesion 97):
#  - las zonas de test se recortan por ANIDAMIENTO REAL DE LLAVES desde cada
#    `#[cfg(test)]`, nunca por la primera marca (un censo cortado en la primera
#    marca ve de menos y parece hallazgo);
#  - alcanzable en verify = los `pub` de lib.rs + todos los `pub` de los
#    modulos `pub mod` + los NOMBRES que los `pub use` sacan de los modulos
#    privados. Los reexports de zk_ssl_hash NO se cuentan en verify: ya son
#    filas de hash (un elemento, una fila);
#  - en hash, todos los `pub` de lib.rs fuera de las zonas de test.
# Cita por NOMBRE y FICHERO, nunca por linea: las lineas caducan, los nombres
# no. Sale con 1 si hay algun rojo, y cada rojo dice que fila editar.
import os, re, sys, unicodedata

RAIZ = os.environ.get('CANON_RAIZ') or os.path.normpath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
DOC = os.environ.get('NUCLEO_DOC') or os.path.join('spec', 'NUCLEO.md')  # NUCLEO_DOC: solo para ensayar el gate contra una copia
VERIFY = os.path.join('crates', 'zk-ssl-verify', 'src')
HASH = os.path.join('crates', 'zk-ssl-hash', 'src', 'lib.rs')
CLASES = ('NUCLEO', 'REFERENCIA', 'LIBRO', 'REGISTRO')
ITEM = re.compile(r'^\s*pub(?:\((?:crate|super)\))?\s+(?:unsafe\s+)?'
                  r'(fn|const|static|struct|enum|mod|use|type|trait)\s+([A-Za-z_][A-Za-z0-9_]*)?')
MOD = re.compile(r'^\s*(pub\s+)?mod\s+(\w+)\s*;')


def ascii_(s):
    return unicodedata.normalize('NFD', s).encode('ascii', 'ignore').decode()


def zonas_test(L):
    z = []
    i = 0
    while i < len(L):
        if re.match(r'^\s*#\[cfg\(test\)\]', L[i]):
            j = i + 1
            while j < len(L) and (L[j].strip() == '' or L[j].strip().startswith('#[')):
                j += 1
            depth = 0
            started = False
            k = j
            for k in range(j, len(L)):
                for ch in L[k]:
                    if ch == '{':
                        depth += 1
                        started = True
                    elif ch == '}':
                        depth -= 1
                if started and depth == 0:
                    break
                if not started and L[k].rstrip().endswith(';'):
                    break
            z.append((i, k))
            i = k + 1
        else:
            i += 1
    return z


def pubs(ruta):
    """(items, usos) fuera de las zonas de test. items: (kind, name); usos: texto entero."""
    L = open(ruta, encoding='utf-8').read().split('\n')
    z = zonas_test(L)
    items, usos = [], []
    for i, l in enumerate(L):
        if any(a <= i <= b for a, b in z):
            continue
        m = ITEM.match(l)
        if m:
            k, n = m.group(1), m.group(2)
            if k == 'use':
                j = i
                while not L[j].rstrip().endswith(';'):
                    j += 1
                usos.append(' '.join(x.strip() for x in L[i:j + 1]))
            elif k != 'mod':
                items.append((k, n))
    return L, items, usos


def nombres_de_use(u):
    """los nombres que exporta un `pub use a::b::{x, y};` o `pub use a::b::x;`"""
    cuerpo = u.split('pub use', 1)[1].strip().rstrip(';')
    m = re.search(r'\{([^}]*)\}', cuerpo)
    if m:
        return cuerpo.split('::')[0], [x.strip().split(' as ')[-1] for x in m.group(1).split(',') if x.strip()]
    return cuerpo.split('::')[0], [cuerpo.split('::')[-1]]


def superficie():
    """[(fichero_corto, kind, nombre)] alcanzable de verify + pub de hash."""
    S = []
    lib = os.path.join(VERIFY, 'lib.rs')
    L, items, usos = pubs(lib)
    mods = {}
    for l in L:
        m = MOD.match(l)
        if m:
            mods[m.group(2)] = 'pub' if m.group(1) else 'priv'
    for k, n in items:
        S.append(('verify/lib.rs', k, n))
    for mod, vis in mods.items():
        rmod = os.path.join(VERIFY, mod + '.rs')
        _, mitems, musos = pubs(rmod)
        if vis == 'pub':
            for k, n in mitems:
                S.append((f'verify/{mod}.rs', k, n))
            # un pub use de zk_ssl_hash dentro de un pub mod es un reexport de hash: no se cuenta
    for u in usos:
        src, ns = nombres_de_use(u)
        if src == 'zk_ssl_hash':
            continue
        if src in mods:
            _, mitems, _ = pubs(os.path.join(VERIFY, src + '.rs'))
            tipos = {n: k for k, n in mitems}
            for n in ns:
                S.append((f'verify/{src}.rs', tipos.get(n, '?'), n))
    _, hitems, _ = pubs(HASH)
    for k, n in hitems:
        S.append(('hash/lib.rs', k, n))
    return S


def tabla(doc):
    """filas de la tabla del censo: (elemento, fichero, clase, familia, linea)"""
    filas = []
    dentro = False
    for num, l in enumerate(open(doc, encoding='utf-8'), 1):
        if l.startswith('| elemento |'):
            dentro = True
            continue
        if dentro and l.startswith('|---'):
            continue
        if dentro and not l.startswith('|'):
            dentro = False
            continue
        if dentro:
            c = [x.strip() for x in l.strip().strip('|').split('|')]
            if len(c) >= 4:
                filas.append((c[0].strip('`'), c[1].strip('`'), ascii_(c[2]).upper(), c[3], num))
    return filas


def declarados(doc):
    """la linea 'Censo derivado: N elementos ... verify ... M ... hash' -> (N, M)"""
    t = open(doc, encoding='utf-8').read()
    m = re.search(r'\*\*Censo derivado:\*\* (\d+) elementos alcanzables en `zk-ssl-verify` y (\d+) `pub` en `zk-ssl-hash`', t)
    if not m:
        return None
    return int(m.group(1)), int(m.group(2))


def main():
    os.chdir(RAIZ)
    rojo = []
    if not os.path.exists(DOC):
        print(f'ROJO: falta {DOC}')
        return 1
    S = superficie()
    nv = sum(1 for f, _, _ in S if f.startswith('verify/'))
    nh = sum(1 for f, _, _ in S if f.startswith('hash/'))
    if nv == 0 or nh == 0:
        print(f'ROJO: el censo derivado esta vacio (verify {nv}, hash {nh}): el instrumento no ve')
        return 1
    F = tabla(DOC)
    if not F:
        print('ROJO: la tabla del censo no se encuentra en el documento (cabecera `| elemento |`)')
        return 1
    derivados = {}
    for f, k, n in S:
        derivados.setdefault((n, f), []).append(k)
    filas = {}
    for n, f, c, fam, num in F:
        filas.setdefault((n, f), []).append((c, fam, num))
    # R1
    for (n, f), ks in sorted(derivados.items()):
        if (n, f) not in filas:
            rojo.append(f'R1: `{n}` ({f}, {"/".join(ks)}) es pub en el arbol y NO tiene fila: anadirla a la tabla de {DOC}')
        elif len(filas[(n, f)]) != 1:
            rojo.append(f'R1: `{n}` ({f}) tiene {len(filas[(n, f)])} filas (lineas {[x[2] for x in filas[(n, f)]]}): una sola')
    # R2
    for (n, f), fs in sorted(filas.items()):
        if (n, f) not in derivados:
            rojo.append(f'R2: la fila `{n}` ({f}, linea {fs[0][2]}) no es un pub del arbol: quitarla o corregir el fichero')
    # R3
    for (n, f), fs in filas.items():
        for c, fam, num in fs:
            if c not in CLASES:
                rojo.append(f'R3: la fila `{n}` (linea {num}) lleva la clase {c!r}; las clases son {"/".join(CLASES)}')
    # R4
    d = declarados(DOC)
    if d is None:
        rojo.append('R4: el documento no declara su censo (linea `**Censo derivado:** N elementos ...`)')
    elif d != (nv, nh):
        rojo.append(f'R4: el documento declara {d[0]}/{d[1]} y el arbol deriva {nv}/{nh}: corregir la linea del censo')
    por_clase = {}
    for (n, f), fs in filas.items():
        por_clase[fs[0][0]] = por_clase.get(fs[0][0], 0) + 1
    if rojo:
        for r in rojo:
            print('  ROJO ' + r)
        print(f'check_nucleo: {len(rojo)} rojo(s); derivados verify {nv} + hash {nh}, filas {len(F)}')
        return 1
    print(f'check_nucleo: la tabla de {DOC} y el censo dicen lo mismo en las dos direcciones: '
          f'{nv} elementos de verify + {nh} de hash = {len(F)} filas '
          f'({", ".join(f"{k} {v}" for k, v in sorted(por_clase.items()))})')
    return 0


if __name__ == '__main__':
    sys.exit(main())
