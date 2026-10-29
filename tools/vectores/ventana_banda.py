#!/usr/bin/env python3
"""§723, la ventana del saldo privado: los sobres que arma un operador con su propia clave.

- `rechazo` (por defecto): los tres de `spec/vectors/rechazo/`, con una raiz de cuentas de UNA hoja.
- `completitud`: los cinco de `spec/vectors/completitud/` cuya resolucion es un rechazo
  `InsufficientBalance` con la banda honesta de la v1; los dos de prenda copian el sobre de prenda
  de `spec/vectors/completitud/prenda-rechazada-por-la-capa.json`.

Los escribe el binario de `tools/vectores/ventana_banda/`; aqui solo se les da el formato de los
negativos armados de la casa (`json.dumps` con las claves ordenadas, sin salto final). Los escribe
en `<salida>`; copiarlos a `spec/vectors/` es un paso aparte, a mano, y sus lineas van AL FINAL de
su MANIFIESTO. ⚠️ No es determinista: cada corrida da otras pruebas. Los vectores salen de UNA
corrida y jamas se reescriben.

    cargo build --release --locked --manifest-path tools/vectores/ventana_banda/Cargo.toml
    python3 -I tools/vectores/ventana_banda.py <binario ventana-banda> <salida> [rechazo|completitud]

El binario queda en `tools/vectores/ventana_banda/target/release/ventana-banda`, o bajo
`$CARGO_TARGET_DIR` si esta puesto.
"""
import json
import os
import subprocess
import sys
import tempfile

AQUI = os.path.dirname(os.path.abspath(__file__))
PRENDA = os.path.normpath(os.path.join(AQUI, "..", "..", "spec", "vectors", "completitud",
                                       "prenda-rechazada-por-la-capa.json"))

# §914 (el corte zkssl/0.5): los ocho renacen con la version del cable en el nombre; sus pruebas
# llevan la marca v2 de la banda y las dos columnas (el probador de `stark-experiment`, por su
# `ocultacion_encendida`). Los de la 0.4 viven bajo `spec/vectors/0.4/`.
VECTORES = {
    # familia: [(fichero, caso del binario)]
    "rechazo": [
        ("rechazo-rech-banda-ventana-v1-0.5.json", "ventana-v1"),
        ("rechazo-rech-banda-ventana-v2-0.5.json", "ventana-v2"),
        ("rechazo-banda-ventana-control-0.5.json", "control"),
    ],
    "completitud": [
        ("rechazo-banda-v1-declarada-0.5.json", "completitud-v1"),
        ("lote-banda-v1-declarada-0.5.json", "lote-v1"),
        ("neg-lote-banda-v1-operacion-fuera-0.5.json", "lote-v1-operacion-fuera"),
        ("prenda-banda-v1-declarada-0.5.json", "prenda-v1"),
        ("neg-prenda-banda-v1-marca-otra-0.5.json", "prenda-v1-marca-otra"),
    ],
}


def main():
    binario, salida = sys.argv[1], sys.argv[2]
    familia = sys.argv[3] if len(sys.argv) > 3 else "rechazo"
    with tempfile.TemporaryDirectory() as tmp:
        for fichero, caso in VECTORES[familia]:
            crudo = os.path.join(tmp, fichero)
            extra = [PRENDA] if caso.startswith("prenda") else []
            subprocess.run([binario, caso, crudo] + extra, check=True)
            with open(crudo, encoding="utf-8") as f:
                d = json.load(f)
            with open(os.path.join(salida, fichero), "w", encoding="utf-8") as f:
                f.write(json.dumps(d, sort_keys=True))
            print(fichero, caso)


if __name__ == "__main__":
    main()
