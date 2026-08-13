#!/usr/bin/env python3
# ── EL REGISTRO DE DOMINIOS (§286) ──────────────────────────────────
# Censa el arbol y lo compara con la tabla REGISTRO escrita en
# crates/zk-ssl-hash/src/lib.rs. Reglas, declaradas ANTES de mirar:
#
#  R1  todo literal b"ZK-SSL-..." en CODIGO vive dentro de una const;
#      un uso inline es rojo (la linea de comentario no cuenta: la
#      medicion filtra //, y por eso este gate filtra igual).
#  R2  (solo familia u64) el mismo NOMBRE declarado N veces lleva el
#      mismo VALOR las N — dos declaraciones que hoy coinciden y nada
#      obliga a que sigan coincidiendo son deriva esperando a pasar.
#      A la familia &[u8] no se le aplica: `DOMINIO` de verify y
#      `DOMINIO` del sdk son nombres LOCALES de dominios distintos, y
#      a esa familia la protege R4 por valor.
#  R3  valores unicos DENTRO de cada grupo de hash: produccion
#      (stark-experiment + zk-ssl-hash comparten la permutacion Rescue)
#      y cada paradigma (zk-core, halo2, plonk) su propio espacio.
#      Entre grupos la reutilizacion es legitima: hashes distintos.
#  R4  valores &[u8] unicos globalmente (todos entran en Blake3).
#  R5  la tabla REGISTRO y el censo coinciden EXACTAMENTE, en las dos
#      direcciones; el rojo dice que linea editar.
import os, re, sys

RAIZ = os.environ.get('CANON_RAIZ') or os.path.normpath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
TABLA_EN = os.path.join('crates', 'zk-ssl-hash', 'src', 'lib.rs')
GRUPO = {
    'stark-experiment': 'produccion', 'zk-ssl-hash': 'produccion',
    'zk-core': 'zk-core', 'halo2-experiment': 'halo2',
    'plonk-experiment': 'plonk',
}
NOMBRE_DOM = re.compile(r'(_DOMAIN$|^DOMINIO(_|$)|^OP_)')
DECL_U64 = re.compile(r'^\s*(?:pub\s+)?const\s+(\w+)\s*:\s*u64\s*=\s*(.+?);')
DECL_B = re.compile(r'^\s*(?:pub\s+)?const\s+(\w+)\s*:\s*&\[u8\]\s*=\s*b"([^"]+)";')
LIT_ZK = re.compile(r'b"(ZK-SSL-[^"]*)"')

def valor_u64(expr):
    expr = expr.split('//')[0].strip()
    m = re.match(r'^0x([0-9A-Fa-f_]+)$', expr)
    if m:
        return int(m.group(1).replace('_', ''), 16)
    m = re.match(r'^u64::from_be_bytes\(\*b"(.{8})"\)$', expr)
    if m:
        return int.from_bytes(m.group(1).encode('latin-1'), 'big')
    return None

def es_comentario(linea):
    s = linea.lstrip()
    return s.startswith('//') or s.startswith('//!') or s.startswith('///')

rojo = []
u64_decl = []     # (grupo, nombre, valor, sitio)
bytes_decl = []   # (cadena, nombre, sitio)
inline = []       # sitios R1

for crate in sorted(os.listdir(os.path.join(RAIZ, 'crates'))):
    src = os.path.join(RAIZ, 'crates', crate, 'src')
    if not os.path.isdir(src):
        continue
    for dirpath, _, files in os.walk(src):
        for fn in sorted(f for f in files if f.endswith('.rs')):
            ruta = os.path.join(dirpath, fn)
            rel = os.path.relpath(ruta, RAIZ)
            for num, linea in enumerate(open(ruta, encoding='utf-8'), 1):
                if es_comentario(linea):
                    continue
                m = DECL_B.match(linea)
                if m and m.group(2).startswith('ZK-SSL-'):
                    bytes_decl.append((m.group(2), m.group(1), '%s:%d' % (rel, num)))
                    continue
                m = DECL_U64.match(linea)
                if m and NOMBRE_DOM.search(m.group(1)):
                    v = valor_u64(m.group(2))
                    if v is None:
                        rojo.append('valor ilegible en %s:%d: %s' % (rel, num, m.group(2).strip()))
                        continue
                    u64_decl.append((GRUPO.get(crate, crate), m.group(1), v, '%s:%d' % (rel, num)))
                    continue
                if LIT_ZK.search(linea):
                    inline.append('%s:%d' % (rel, num))

# R1
for s in inline:
    rojo.append('R1: literal ZK-SSL- inline (fuera de una const) en %s' % s)

# R2 (u64): mismo nombre => mismo valor, en todo el arbol
por_nombre = {}
for g, n, v, s in u64_decl:
    por_nombre.setdefault(n, []).append((v, s))
for n, vs in sorted(por_nombre.items()):
    if len({v for v, _ in vs}) > 1:
        rojo.append('R2: %s declarado con VALORES DISTINTOS: %s' %
                    (n, ' · '.join('%s=%#x' % (s, v) for v, s in vs)))

# R3: unicidad de valor dentro de cada grupo
por_grupo = {}
for g, n, v, s in u64_decl:
    por_grupo.setdefault(g, {}).setdefault(v, set()).add(n)
for g, vals in sorted(por_grupo.items()):
    for v, nombres in sorted(vals.items()):
        if len(nombres) > 1:
            rojo.append('R3: colision en el grupo %s: %#x lo usan %s' %
                        (g, v, ' y '.join(sorted(nombres))))

# R4: cadenas unicas globalmente
por_cadena = {}
for c, n, s in bytes_decl:
    por_cadena.setdefault(c, []).append(s)
# (la unicidad aqui es de VALOR entre dominios distintos: dos declaraciones
#  de la MISMA cadena serian dos casas para un dominio -> tambien rojo)
for c, ss in sorted(por_cadena.items()):
    if len(ss) > 1:
        rojo.append('R4: la cadena %r esta declarada en %d sitios: %s' % (c, len(ss), ' · '.join(ss)))

# R5: la tabla REGISTRO de lib.rs == censo
tabla_u64, tabla_b = set(), set()
ruta_tabla = os.path.join(RAIZ, TABLA_EN)
for num, linea in enumerate(open(ruta_tabla, encoding='utf-8'), 1):
    m = re.match(r'^// REGISTRO: u64 (\S+) (\w+) 0x([0-9A-Fa-f_]+)\s*$', linea)
    if m:
        tabla_u64.add((m.group(1), m.group(2), int(m.group(3).replace('_', ''), 16)))
        continue
    m = re.match(r'^// REGISTRO: bytes (ZK-SSL-\S+)\s*$', linea)
    if m:
        tabla_b.add(m.group(1))
censo_u64 = {(g, n, v) for g, n, v, _ in u64_decl}
censo_b = set(por_cadena)
for falta in sorted(censo_u64 - tabla_u64):
    rojo.append('R5: en el arbol y NO en la tabla: u64 %s %s %#x -> anadir linea REGISTRO en %s' % (falta + (TABLA_EN,)))
for sobra in sorted(tabla_u64 - censo_u64):
    rojo.append('R5: en la tabla y NO en el arbol: u64 %s %s %#x -> quitar o corregir en %s' % (sobra + (TABLA_EN,)))
for falta in sorted(censo_b - tabla_b):
    rojo.append('R5: en el arbol y NO en la tabla: bytes %s -> anadir linea REGISTRO en %s' % (falta, TABLA_EN))
for sobra in sorted(tabla_b - censo_b):
    rojo.append('R5: en la tabla y NO en el arbol: bytes %s -> quitar o corregir en %s' % (sobra, TABLA_EN))

print('dominios u64: %d declaraciones, %d (grupo,nombre,valor) distintos' % (len(u64_decl), len(censo_u64)))
print('dominios bytes: %d cadenas ZK-SSL- declaradas' % len(censo_b))
if rojo:
    for r in rojo:
        print('  XX  %s' % r)
    print('check_dominios: ROJO · %d fallo(s)' % len(rojo))
    sys.exit(1)
print('check_dominios: el censo y el registro dicen lo mismo; sin literales sueltos ni derivas')
sys.exit(0)
