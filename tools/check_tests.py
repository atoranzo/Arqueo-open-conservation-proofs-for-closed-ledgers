#!/usr/bin/env python3
"""Busca tests que NO PROTEGEN.

Un test puede estar escrito, compilar, y no comprobar nada. Esta herramienta
busca las dos formas en que ha ocurrido en este proyecto:

1. **Anidado dentro de una función.** Compila, no se registra en el arnés, y
   no aparece en la lista de ejecución: no falla y no avisa. Apareció una vez
   (`balances_plus_pending`, AUDITORIA §17) y solo lo delató contar `#[test]`
   declarados frente a ejecutados.

2. **`#[ignore]` sin condición.** No se ejecuta en ningún modo, y su
   documentación suele explicar cómo lanzarlo a mano. Un test que depende de
   que alguien lo recuerde no protege nada. Apareció una vez
   (`zero_value_only_works_in_release_mode`, AUDITORIA §20).

   Se acepta `#[cfg_attr(debug_assertions, ignore = "...")]`, que salta el
   test solo donde el problema existe y deja que `--release` lo ejecute.

Uso:

    python3 tools/check_tests.py
"""

import os
import re
import sys

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def ficheros_rust():
    for base, _, nombres in os.walk(os.path.join(RAIZ, "crates")):
        if "target" in base:
            continue
        for n in nombres:
            if n.endswith(".rs"):
                yield os.path.join(base, n)


def anidados_en_funcion(ruta):
    """`#[test]` con una `fn` abierta por encima."""
    hallazgos = []
    pila = []
    for num, linea in enumerate(open(ruta, encoding="utf-8"), 1):
        s = linea.strip()
        if s.startswith("//"):
            continue
        if s == "#[test]" and "fn" in pila:
            hallazgos.append(num)
        abre = linea.count("{")
        if abre:
            if re.search(r"\bmod\s+\w+", linea):
                tipo = "mod"
            elif re.search(r"\bfn\s+\w+", linea):
                tipo = "fn"
            else:
                tipo = "otro"
            pila.extend([tipo] * abre)
        for _ in range(linea.count("}")):
            if pila:
                pila.pop()
    return hallazgos


def ignore_incondicional(ruta):
    """`#[ignore]` fuera de un `cfg_attr`."""
    hallazgos = []
    lineas = open(ruta, encoding="utf-8").readlines()
    for num, linea in enumerate(lineas, 1):
        s = linea.strip()
        if s.startswith("//") or s.startswith("///"):
            continue
        if re.match(r"#\[ignore\b", s):
            hallazgos.append(num)
    return hallazgos


def main():
    problemas = 0
    total = 0
    for ruta in sorted(ficheros_rust()):
        rel = os.path.relpath(ruta, RAIZ)
        total += open(ruta, encoding="utf-8").read().count("#[test]")

        for n in anidados_en_funcion(ruta):
            print(f"  ANIDADO   {rel}:{n} — `#[test]` dentro de una función: "
                  f"compila pero no se ejecuta")
            problemas += 1

        for n in ignore_incondicional(ruta):
            print(f"  IGNORADO  {rel}:{n} — `#[ignore]` sin condición: no se "
                  f"ejecuta en ningún modo. ¿`#[cfg_attr(debug_assertions, "
                  f"ignore = \"…\")]`?")
            problemas += 1

    if problemas:
        print(f"\n{problemas} test(s) que no protegen, de {total} declarados.")
        return 1
    print(f"{total} tests declarados: ninguno anidado, ninguno ignorado sin "
          f"condición.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
