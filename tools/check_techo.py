#!/usr/bin/env python3
# ── EL ATADO DEL TECHO N (RFC-0010, antes de la E4; 5.A-414) ─────────
# El techo de la promesa —«no inclusion en N cabezas firmadas es
# censura» (§121)— tiene UN productor: `N_MAX_CABEZAS`, en
# crates/zk-ssl-node/src/vista_acuses.rs, firmado en cada cabeza desde
# el §275 y dentro de cada hoja de acuse y de recibo. La PROSA lo cita
# en un perimetro publicado, y hasta aqui nadie comparaba las dos cosas.
# Reglas, declaradas ANTES de mirar:
#
#  R1  el productor se LEE del codigo, no se teclea aqui: cambiar el
#      techo es cambiar una linea, y el juez sigue a la linea.
#  R2  cada sitio del PERIMETRO se localiza por PATRON, nunca por numero
#      de linea: una cita por numero de linea caduca con el corte
#      anterior y no la vigila nadie (5.A-432, §561).
#  R3  cada patron aparece EXACTAMENTE las veces declaradas: si la prosa
#      se mueve, se duplica o se borra, el juez lo dice en vez de callar.
#  R4  cada numero que el patron captura es el del productor, en la
#      forma en que ese sitio lo escribe (decimal, hex del cable, o con
#      punto de millar).
#  R5  los literales de TEST no son el techo y NO se miran: un juez que
#      los mirara seria mas estricto que su invariante (§560). Tampoco
#      el gasto de indices del latido —1.440 al dia a un latido por
#      minuto—, que coincide con N y no es N.
#
# uso: python3 tools/check_techo.py            (exit 0 VERDE, 1 ROJO)
#      TECHO_RAIZ=<copia> python3 tools/check_techo.py
#      —la trampilla para ensayar el gate contra una copia sin tocar el
#      arbol, el molde de `NUCLEO_DOC` en check_nucleo (§561)—.
import os
import re
import sys

RAIZ = os.environ.get('TECHO_RAIZ', '.')
PRODUCTOR = 'crates/zk-ssl-node/src/vista_acuses.rs'

# (fichero, patron con uno o mas grupos que capturan el numero, forma, veces)
PERIMETRO = [
    ('spec/RPC.md', r'\*\*`n` = (\d+)\*\*', 'dec', 1),
    # los dos ejemplos de la respuesta del titular: el acuse (§274) y el recibo (§571)
    ('spec/RPC.md', r'"n": "0x([0-9a-f]+)"', 'hex', 2),
    ('doc/CONFIANZA_RESIDUAL.md', r'N_max = ([\d.]+) cabezas firmadas', 'miles', 1),
    ('spec/rfc/0010-el-recibo-de-recepcion.md', r'`N = ([\d.]+)` cabezas firmadas', 'miles', 1),
    ('spec/rfc/0010-el-recibo-de-recepcion.md',
     r'`N = ([\d.]+)` vuelve a ser lo que dice: ([\d.]+) cabezas firmadas', 'miles', 1),
]


def leer(rel):
    with open(os.path.join(RAIZ, rel), encoding='utf-8') as f:
        return f.read()


def valor(texto, forma):
    if forma == 'dec':
        return int(texto)
    if forma == 'hex':
        return int(texto, 16)
    if forma == 'miles':
        if not re.fullmatch(r'\d{1,3}(\.\d{3})*', texto):
            return None
        return int(texto.replace('.', ''))
    raise ValueError(forma)


rojos = []
m = re.search(r'^pub const N_MAX_CABEZAS: u64 = ([0-9_]+);$', leer(PRODUCTOR), re.M)
if not m:
    print('ROJO  R1: no encuentro `pub const N_MAX_CABEZAS: u64 = ...;` en ' + PRODUCTOR)
    sys.exit(1)
N = int(m.group(1).replace('_', ''))

citas = 0
ficheros = set()
for rel, patron, forma, veces in PERIMETRO:
    texto = leer(rel)
    halladas = list(re.finditer(patron, texto))
    if len(halladas) != veces:
        rojos.append('R3  %s: el patron %r aparece %d vez/veces y se declararon %d'
                     % (rel, patron, len(halladas), veces))
        continue
    for h in halladas:
        linea = texto.count('\n', 0, h.start()) + 1
        for g in h.groups():
            v = valor(g, forma)
            if v != N:
                rojos.append('R4  %s:%d dice %s y el productor es %d (%s)'
                             % (rel, linea, g, N, PRODUCTOR))
            citas += 1
    ficheros.add(rel)

if rojos:
    for r in rojos:
        print('ROJO  ' + r)
    print('%d fallo(s) en el atado del techo: la prosa y el productor ya no dicen lo mismo.'
          % len(rojos))
    sys.exit(1)
print('check_techo: N = %d (%s), y las %d citas del perimetro publicado lo dicen, en %d ficheros.'
      % (N, PRODUCTOR, citas, len(ficheros)))
