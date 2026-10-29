#!/usr/bin/env python3
"""§914 (el corte zkssl/0.5): los negativos de la MARCA v2, los de la tabla de la D-G del RFC-0018.

Cada uno sale de UN positivo de la 0.5 de su familia por UNA mutacion de su prueba —o es la positiva
de la 0.4 de esa familia, con sus bytes, para los dos casos que la D-G pide asi—, y se escribe con el
formato de los negativos de su familia (`json.dumps`, sin salto final). En la edad van los catorce;
en las otras cuatro familias del kit, `marca-v1`, `meta-vacio` y `familia-ajena`, porque en el kit
cada familia tiene su guarda y el resto lo cubre la edad. Los escribe en `<salida>/<familia>/`;
copiarlos a `spec/vectors/<familia>/` es un paso aparte, a mano, y sus lineas van AL FINAL del
MANIFIESTO (nunca se reescribe un vector ni una linea). Determinista: dos corridas, los mismos bytes.

La cabeza de una prueba: cuatro bytes (ancho, ancho auxiliar, aleatorios, log2 de la longitud), el
largo del meta en dos bytes little-endian y el meta; la marca v2 son 25 bytes, `arqueo:oculta:2`,
m (u32), familia (u16), version de la AIR (u16), perfil (u8) y columnas (u8). Detras, el modulo
(un byte de largo y ocho) y las diez de las opciones, la primera el numero de consultas.

    python3 -I tools/vectores/marca_v2.py <arbol> <salida>
"""
import json
import os
import sys

PREFIJO = b"arqueo:oculta:2"
LARGO_V2 = 25

# familia: (positivo de la 0.5, positivo de la 0.4 conservado, ruta de la prueba en el sobre,
#           familia de la marca, una familia AJENA del kit)
FAMILIAS = {
    "edad": ("edad/edad-todos-0.5.json", "0.4/edad/edad-todos.json", ["prueba"], 14, 16),
    "pago": ("pago/pago-t-frontera-0.5.json", "0.4/pago/pago-t-frontera.json", ["prueba"], 16, 15),
    "pendiente": ("pendiente/cobro-inferior-importe-0.5.json", "0.4/pendiente/cobro-inferior-importe.json",
                  ["prueba"], 15, 16),
    "prenda": ("prenda/prenda-publicada-0.5.json", "0.4/prenda/prenda-publicada.json", ["prueba"], 5, 14),
    "rechazo": ("rechazo/saldo-insuficiente-0.5.json", "0.4/rechazo/saldo-insuficiente.json",
                ["banda", "prueba"], 13, 5),
}
TODOS = ["marca-v1", "meta-vacio", "prefijo-otro", "largo-26", "largo-24", "largo-v1", "familia-ajena",
         "version-air", "perfil", "perfil-reservado", "m-otra", "columnas", "opciones", "familia-y-forma",
         "m-y-opciones"]
LAS_CUATRO = ["marca-v1", "meta-vacio", "familia-ajena"]


def cabeza(b):
    largo = int.from_bytes(b[4:6], "little")
    return b[:4], b[6:6 + largo], b[6 + largo:]


def con_meta(b, meta, resto=None):
    c, _, r = cabeza(b)
    return c + len(meta).to_bytes(2, "little") + meta + (r if resto is None else resto)


def tocar_opciones(resto):
    # el modulo (1 + 8 bytes) y despues la primera de las diez de las opciones, las consultas
    r = bytearray(resto)
    assert r[0] == 8, "el modulo de ocho bytes"
    r[9] += 1
    return bytes(r)


def mutar(caso, b, ajena, viejo):
    _, meta, resto = cabeza(b)
    assert meta[:15] == PREFIJO and len(meta) == LARGO_V2, "el positivo de la 0.5 lleva la marca v2"
    m = bytearray(meta)
    if caso == "marca-v1":
        return viejo
    if caso == "largo-v1":
        _, meta_v1, _ = cabeza(viejo)
        assert len(meta_v1) == 19, "la marca v1 de 19 bytes"
        return con_meta(viejo, meta_v1 + b"\x00")
    if caso == "meta-vacio":
        return con_meta(b, b"")
    if caso == "prefijo-otro":
        m[14] = ord("3")
    elif caso == "largo-26":
        m += b"\x00"
    elif caso == "largo-24":
        m = m[:-1]
    elif caso in ("familia-ajena", "familia-y-forma"):
        m[19:21] = ajena.to_bytes(2, "little")
    elif caso == "version-air":
        m[21:23] = (2).to_bytes(2, "little")
    elif caso == "perfil":
        m[23] = 2
    elif caso == "perfil-reservado":
        m[23] = 255
    elif caso in ("m-otra", "m-y-opciones"):
        m[15:19] = (32).to_bytes(4, "little")
    elif caso == "columnas":  # §914 (1h bis): una columna, la de zkssl/0.4, en una marca v2
        m[24] = 1
    elif caso != "opciones":
        raise ValueError(caso)
    if caso in ("opciones", "m-y-opciones"):
        resto = tocar_opciones(resto)
    nb = con_meta(b, bytes(m), resto)
    if caso == "familia-y-forma":
        nb = bytes([nb[0] + 1]) + nb[1:]
    return nb


def main():
    arbol, salida = sys.argv[1:3]
    for familia, (pos, viejo_rel, ruta, _propia, ajena) in FAMILIAS.items():
        with open(os.path.join(arbol, "spec", "vectors", pos), encoding="utf-8") as f:
            d = json.load(f)
        with open(os.path.join(arbol, "spec", "vectors", viejo_rel), "rb") as f:
            viejo_bytes = f.read()
        viejo = json.loads(viejo_bytes)
        nodo, nodo_viejo = d, viejo
        for k in ruta[:-1]:
            nodo, nodo_viejo = nodo[k], nodo_viejo[k]
        b = bytes.fromhex(nodo[ruta[-1]][2:])
        prueba_vieja = bytes.fromhex(nodo_viejo[ruta[-1]][2:])
        os.makedirs(os.path.join(salida, familia), exist_ok=True)
        for caso in (TODOS if familia == "edad" else LAS_CUATRO):
            fichero = f"neg-v2-{caso}.json"
            if caso == "marca-v1":
                texto = viejo_bytes  # la positiva de la 0.4, con sus bytes
            else:
                nb = mutar(caso, b, ajena, prueba_vieja)
                assert nb != b
                base = json.loads(json.dumps(viejo if caso == "largo-v1" else d))
                n = base
                for k in ruta[:-1]:
                    n = n[k]
                n[ruta[-1]] = "0x" + nb.hex()
                texto = json.dumps(base).encode()
            with open(os.path.join(salida, familia, fichero), "wb") as f:
                f.write(texto)
            print(f"{familia}/{fichero}")


if __name__ == "__main__":
    main()
