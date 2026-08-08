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

   Se aceptan DOS formas, y solo dos:

   a) `#[cfg_attr(debug_assertions, ignore = "...")]`, que salta el test
      solo donde el problema existe y deja que `--release` lo ejecute.

   b) `#[ignore = "instrumento de medida..."]` — un test que **no es una
      comprobación**: mide y reporta, y fallar no significa nada porque no
      afirma nada. Se declara con esa marca literal en el motivo.

   ⚠️ **Por qué existe (b), y no es una comodidad.** Hasta §224 esta
   herramienta daba **trece falsos positivos** —uno por circuito, más tres
   de la capa— contra un árbol sellado y correcto. Una herramienta que
   grita cuando no pasa nada se deja de mirar, y entonces **tampoco avisa
   cuando sí pasa**. Que nadie la invocara en ningún sello no era descuido:
   era la consecuencia.

   La marca no es una lista de excepciones por fichero: es una **regla**.
   Un instrumento nuevo se declara igual; un `#[ignore]` que no se declare
   sigue siendo un fallo.

Uso:

    python3 tools/check_tests.py
"""

import os
import re
import sys

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# Marca literal que declara un `#[ignore]` como instrumento de medida y no
# como comprobacion. ASCII a proposito: un patron con tilde es fragil
# segun la codificacion con que se lea el fichero.
MARCA_INSTRUMENTO = "instrumento de medida"


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
    """`#[ignore]` fuera de un `cfg_attr`.

    Devuelve DOS listas: los que no protegen y los declarados como
    instrumento de medida. Separarlos es lo que hace utilizable la
    herramienta: ver la seccion (b) de la cabecera.
    """
    hallazgos = []
    instrumentos = []
    lineas = open(ruta, encoding="utf-8").readlines()
    for num, linea in enumerate(lineas, 1):
        s = linea.strip()
        if s.startswith("//"):
            continue
        if re.match(r"#\[ignore\b", s):
            if MARCA_INSTRUMENTO in s:
                instrumentos.append(num)
            else:
                hallazgos.append(num)
    return hallazgos, instrumentos


def main():
    problemas = 0
    total = 0
    n_instrumentos = 0
    for ruta in sorted(ficheros_rust()):
        rel = os.path.relpath(ruta, RAIZ)
        total += open(ruta, encoding="utf-8").read().count("#[test]")

        for n in anidados_en_funcion(ruta):
            print(f"  ANIDADO   {rel}:{n} — `#[test]` dentro de una función: "
                  f"compila pero no se ejecuta")
            problemas += 1

        sueltos, instrumentos = ignore_incondicional(ruta)
        n_instrumentos += len(instrumentos)
        for n in sueltos:
            print(f"  IGNORADO  {rel}:{n} — `#[ignore]` sin condición y sin "
                  f"declararse instrumento: no se ejecuta en ningún modo. "
                  f"¿`#[cfg_attr(debug_assertions, "
                  f"ignore = \"…\")]`?")
            problemas += 1

    if problemas:
        print(f"\n{problemas} test(s) que no protegen, de {total} declarados "
              f"({n_instrumentos} instrumentos de medida, declarados).")
        return 1
    print(f"{total} tests declarados: ninguno anidado, ninguno ignorado sin "
          f"condición ni declaracion. "
          f"{n_instrumentos} instrumentos de medida, declarados.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
