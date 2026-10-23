#!/usr/bin/env python3
"""§717 y §721: los cuatro negativos de `spec/vectors/edad/` del coste del juez de la edad.

Cada uno es un paquete que cualquiera arma sin la clave del operador -cabeza v5 con firma propia,
de una clave XMSS de test de semilla fija- y sin probar -una prueba fabricada que solo tiene la
forma-. Los escribe el binario de `tools/vectores/coste_edad/`; aqui solo se les da el formato de
los negativos de la familia (`json.dumps` con las claves ordenadas, sin salto final). Los escribe
en `<salida>`; copiarlos a `spec/vectors/edad/` es un paso aparte, a mano, y sus lineas van AL
FINAL del MANIFIESTO. Jamas se reescriben: el guion se puede volver a correr y tiene que dar los
mismos bytes (`cmp`).

    cargo build --release --locked --manifest-path tools/vectores/coste_edad/Cargo.toml
    python3 -I tools/vectores/coste_edad.py <binario coste-edad> <salida>

El binario queda en `tools/vectores/coste_edad/target/release/coste-edad`, o bajo
`$CARGO_TARGET_DIR` si esta puesto.
"""
import json
import os
import subprocess
import sys
import tempfile

# Las m van fijas y no salen del tope: las tres primeras valen con cualquier tope de 13 a 16; la
# cuarta (§721) no, y con la tercera fija el tope. Sobre el tope, m = 17 y m = 23 (la mayor que el
# lector de la prueba admite); por debajo o en el tope, m = 13. Cada uno de los tres primeros
# ficheros se nombra por lo que es con cualquier tope de ese rango.
VECTORES = [
    # (fichero, m de la prueba fabricada, nextPending)
    ("neg-m-sobre-tope.json", 17, (1 << 16) + 1),
    ("neg-m-mayor-legible.json", 23, 1 << 23),
    ("neg-m-13-fabricada.json", 13, 1 << 13),
    # §721, con el tope en 13: la marca mas pequena sobre el tope. Este SI depende del tope: cae
    # por el tope con 13 y llega a la prueba con 14, asi que junto con el de m = 13 fija el tope.
    ("neg-m-14-sobre-tope.json", 14, (1 << 13) + 1),
]


def main():
    binario, salida = sys.argv[1], sys.argv[2]
    with tempfile.TemporaryDirectory() as tmp:
        for fichero, m, n in VECTORES:
            crudo = os.path.join(tmp, fichero)
            subprocess.run([binario, str(m), str(n), crudo], check=True)
            with open(crudo, encoding="utf-8") as f:
                d = json.load(f)
            with open(os.path.join(salida, fichero), "w", encoding="utf-8") as f:
                f.write(json.dumps(d, sort_keys=True))
            print(fichero, m, n)


if __name__ == "__main__":
    main()
