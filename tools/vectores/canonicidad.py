#!/usr/bin/env python3
"""§718, la canonicidad u64 -> campo: los negativos de un `u64` que no es la escritura
canonica de su elemento (`x + p` en lugar de `x`), o que no es un indice de cuenta.

Cada vector sale de UN positivo del arbol por UNA mutacion de UN campo, y se escribe con el formato
de los negativos de su familia: `json.dumps` sin sangria ni salto final en edad, pago y pendiente
(los positivos ya van con las claves ordenadas), y con sangria 1 y sin salto final en rotacion,
como su positivo. Determinista: dos corridas, los mismos bytes.

    python3 -I tools/vectores/canonicidad.py <raiz del arbol>

No reescribe un fichero que ya exista con otros bytes: los vectores no se reescriben (regla 2 del
PROCESO). Si existe con los mismos bytes, lo dice y sigue.
"""

import json
import os
import sys

P = 2**64 - 2**32 + 1


def mas_p(q):
    """La escritura de `x + p` de un Q del cable, en su forma minima."""
    x = int(q, 16)
    assert x + P < 2**64, f"{q} no tiene alias: es mayor que 2^32 - 2"
    return hex(x + P)


def edad_emisor(d, valor):
    d["enunciado"]["emisor"] = valor


def pago_importe(d):
    d["enunciado"]["importe"] = mas_p(d["enunciado"]["importe"])


def pago_t(d):
    d["enunciado"]["t"] = mas_p(d["enunciado"]["t"])


def cobro_inferior(d):
    d["enunciado"]["inferior"] = mas_p(d["enunciado"]["inferior"])


def acta_mmr_size(d):
    # La procedencia del acta 1: la segunda de la cadena, la primera que no es la genesis.
    proc = d["actas"][1]["acta"]["procedencia"]
    proc["mmrSize"] = mas_p(proc["mmrSize"])


# (directorio del negativo, positivo, negativo, mutacion, sangria, directorio del positivo). §914
# (el corte zkssl/0.5): los positivos de la 0.4 viven bajo `spec/vectors/0.4/`, y los negativos que
# salieron de ellos y caen antes de la prueba siguen en su familia con sus bytes; el que caia en el
# enunciado, `neg-emisor-fuera-de-cuentas`, se mudo con ellos a `0.4/edad/` -el juez de la 0.5 mira
# la marca antes que el enunciado- y renace de `edad-todos-0.5.json` con la version en el nombre.
VECTORES = [
    ("edad", "edad-todos.json", "neg-emisor-no-canonico.json",
     lambda d: edad_emisor(d, hex(P)), None, "0.4/edad"),
    ("0.4/edad", "edad-todos.json", "neg-emisor-fuera-de-cuentas.json",
     lambda d: edad_emisor(d, hex(1 << 32)), None, "0.4/edad"),
    ("edad", "edad-todos-0.5.json", "neg-emisor-fuera-de-cuentas-0.5.json",
     lambda d: edad_emisor(d, hex(1 << 32)), None, "edad"),
    ("pago", "pago-t-frontera.json", "neg-importe-alias.json", pago_importe, None, "0.4/pago"),
    ("pago", "pago-t-frontera.json", "neg-t-alias.json", pago_t, None, "0.4/pago"),
    ("pendiente", "cobro-inferior-importe.json", "neg-inferior-alias.json", cobro_inferior, None, "0.4/pendiente"),
    ("rotacion", "rotacion-un-eslabon.json", "neg-acta-mmrsize-alias.json", acta_mmr_size, 1, "rotacion"),
]


def main():
    if len(sys.argv) != 2:
        print("uso: python3 -I tools/vectores/canonicidad.py <raiz>", file=sys.stderr)
        sys.exit(2)
    raiz = os.path.join(sys.argv[1], "spec", "vectors")
    rc = 0
    for familia, positivo, negativo, mutar, sangria, de in VECTORES:
        with open(os.path.join(raiz, de, positivo), "rb") as f:
            d = json.loads(f.read())
        mutar(d)
        texto = json.dumps(d, indent=sangria).encode()
        ruta = os.path.join(raiz, familia, negativo)
        if os.path.exists(ruta):
            with open(ruta, "rb") as f:
                igual = f.read() == texto
            print(f"{'IGUAL' if igual else 'DISTINTO, NO SE TOCA'} {familia}/{negativo}")
            rc |= 0 if igual else 1
            continue
        with open(ruta, "wb") as f:
            f.write(texto)
        print(f"NUEVO {familia}/{negativo} ({len(texto)} B)")
    sys.exit(rc)


if __name__ == "__main__":
    main()
