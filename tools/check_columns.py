#!/usr/bin/env python3
"""
Comprueba que cada columna declarada en un circuito se RELLENE de verdad.

## Por qué existe

Al construir `circuit_mint_pending` se declararon **siete columnas** con sus
restricciones escritas, y la traza nunca les asignaba valor. Todas valían
cero, así que sus restricciones se cumplían trivialmente —`0 − 0 = 0`— y
**ningún test lo detectó**:

- Los tests negativos pasaban igual, porque fallaban por otras
  restricciones.
- El testigo honesto no revela nada: en su caso las columnas *deberían*
  tener valor, y él las rellenaría... si el código las rellenara.

Es el mismo modo de fallo que las **restricciones idénticamente cero**
documentadas en `AUDITORIA.md`. Nada automático lo detectaba.

## ⚠️ Esta comprobación se equivocó dos veces antes de funcionar

Primera versión: solo contaba lecturas de la forma `current[COL]`, así que
**no detectaba** columnas usadas dentro de literales de array.

Segunda versión: no reconocía el patrón `state[COL] = …` de `trace.fill`, y
daba **cinco falsos positivos** en `solvency.rs`.

Ambos errores se encontraron **validando la comprobación contra un caso
conocido**: quitar a propósito el relleno de una columna y confirmar que
salta, y comprobar que un circuito correcto no dispara.

**Una verificación rota es peor que ninguna**: dirige la atención al sitio
equivocado.

## Uso

    python3 tools/check_columns.py crates/stark-experiment/src

Sale con código 1 si encuentra alguna columna sin rellenar.

## ⚠️ Lo que NO comprueba

- Que la columna se rellene con el valor **correcto**.
- Que se rellene en **todas** las filas donde hace falta.
- Columnas que se rellenan con un patrón distinto a los tres reconocidos.

Un patrón nuevo de relleno haría que esta herramienta diera falsos
positivos, no falsos negativos: es el sentido seguro del error.
"""

import os
import re
import sys


def analizar(ruta):
    """Devuelve [(columna, usos)] de las declaradas y nunca rellenadas."""
    with open(ruta) as f:
        c = f.read()
    columnas = re.findall(r"^const (COL_\w+): usize", c, re.M)
    malos = []
    for col in columnas:
        escrituras = 0
        # rows[..][COL] = …
        escrituras += len(re.findall(rf"\brows?\[[^\]]*\]\[{col}[^\]]*\]\s*=", c))
        # row[COL] = …   y   row[COL..COL + n]
        escrituras += len(re.findall(rf"\brow\[{col}[^\]]*\]\s*=", c))
        escrituras += len(re.findall(rf"\brow\[{col}\.\.", c))
        # state[COL] = …  (patrón de trace.fill)
        escrituras += len(re.findall(rf"\bstate\[{col}[^\]]*\]\s*=", c))
        escrituras += len(re.findall(rf"\bstate\[{col}\.\.", c))

        usos = len(re.findall(rf"\b{col}\b", c)) - 1 - escrituras
        if escrituras == 0 and usos > 0:
            malos.append((col, usos))
    return malos


def main():
    directorio = sys.argv[1] if len(sys.argv) > 1 else "."
    ficheros = sorted(
        f
        for f in os.listdir(directorio)
        if f.endswith(".rs") and f not in ("lib.rs", "merkle.rs")
    )
    total = 0
    for f in ficheros:
        malos = analizar(os.path.join(directorio, f))
        if malos:
            total += len(malos)
            print(f"  !! {f}")
            for col, usos in malos:
                print(f"       {col}: 0 escrituras, {usos} usos")

    if total:
        print(f"\n{total} columnas declaradas que la traza NUNCA rellena.")
        print("Sus restricciones se cumplen trivialmente y ningún test lo detecta.")
        return 1

    print(f"{len(ficheros)} circuitos: todas las columnas declaradas se rellenan.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
