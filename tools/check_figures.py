#!/usr/bin/env python3
"""
Comprueba que las cifras de tests que afirma la documentación sean ciertas.

## Por qué existe

`doc/ZENODO.md` documenta que una revisión encontró **cinco cifras rancias**
en los documentos. Esta sesión encontró **cuatro más**, tres de ellas
causadas por añadir tests sin propagar el número:

| Documento | Decía | Era |
|---|---|---|
| Ocho ficheros | 156 tests en `zk-ssl` | 163 |
| Tres ficheros | 319 ejecutables | 359 |
| `ARQUITECTURA.md` | 12 y 16 en `settlement-layer` | 17 |
| `VISION.md` | 13 en `circuit_mint_pending` | 16 |
| `ARQUITECTURA.md` | 9 en `circuit_threshold` | 11 |

El principio del proyecto es que **ninguna cifra sea estimada**. Una que
nadie puede reproducir ejecutando el comando de al lado lo incumple.

## Uso

    python3 tools/check_figures.py

Sale con código 1 si alguna cifra no coincide con el código.

## ⚠️ Lo que NO comprueba

- Cifras de tiempo, tamaño o rendimiento: esas exigen ejecutar.
- Cifras escritas con palabras ("dieciséis tests").
- La prueba **ignorada**: el recuento cuenta `#[test]` declarados, y una de
  `stark-experiment` está marcada `#[ignore]`. Los documentos deben decir
  *"N ejecutables más una ignorada"*, no *"N+1 ejecutables"*.
"""

import os
import re
import sys


def cuentas_reales():
    """Tests declarados por módulo y por crate."""
    real = {}
    for crate in os.listdir("crates"):
        d = os.path.join("crates", crate, "src")
        if not os.path.isdir(d):
            continue
        total = 0
        for root, _, ficheros in os.walk(d):
            for f in ficheros:
                if not f.endswith(".rs"):
                    continue
                with open(os.path.join(root, f)) as fh:
                    n = len(re.findall(r"#\[test\]", fh.read()))
                total += n
                if n:
                    real[f[:-3]] = real.get(f[:-3], 0) + n
        real[crate] = total
    return real


PATRONES = [
    (r"(\d+) tests? en `?([a-z_-]+)`?", "num-primero"),
    (r"`([a-z_]+)`[^.\n]{0,25}?(\d+) tests", "mod-primero"),
    (r"(\d+) tests?\b[^.\n]{0,20}?`([a-z_]+)`", "num-primero"),
]


def main():
    real = cuentas_reales()
    docs = [f for f in os.listdir(".") if f.endswith(".md")]
    if os.path.isdir("doc"):
        docs += ["doc/" + f for f in os.listdir("doc") if f.endswith(".md")]

    malas = []
    comprobadas = 0
    huerfanas = []
    for doc in sorted(docs):
        with open(doc) as fh:
            c = fh.read()
        atribuidas = set()
        for patron, orden in PATRONES:
            for m in re.finditer(patron, c):
                a, b = m.group(1), m.group(2)
                mod, n = (b, int(a)) if orden == "num-primero" else (a, int(b))
                if mod not in real:
                    continue
                comprobadas += 1
                atribuidas.add(m.span())
                if n != real[mod]:
                    malas.append((doc, mod, n, real[mod]))

        # ⚠️ **Cifras que hablan de tests y NO se han podido atribuir.**
        #
        # Los patrones exigen el nombre del crate cerca del número. Una cifra
        # separada por un comentario largo —`cargo test -p zk-ssl  # 172
        # tests`— no se atribuye, y antes la herramienta la ignoraba **en
        # silencio**: decía «todo correcto» sin haberla mirado.
        for m in re.finditer(r"(\d+) tests?\b", c):
            if any(a <= m.start() < b for a, b in atribuidas):
                continue
            if int(m.group(1)) < 10:      # «2 tests», «1 test»: prosa, no cifras
                continue
            linea = c[:m.start()].count("\n") + 1
            huerfanas.append((doc, linea, m.group(0)))

    for doc, mod, dice, hay in sorted(set(malas)):
        print(f"  !! {doc}: {mod} dice {dice}, hay {hay}")

    if malas:
        print(f"\n{len(set(malas))} cifras que la documentación afirma y el "
              f"código desmiente.")
        return 1

    print(f"{len(docs)} documentos, {comprobadas} cifras atribuidas a un crate: "
          f"todas coinciden con el código.")

    if huerfanas:
        print(f"\n⚠️  {len(huerfanas)} cifras hablan de tests y NO se han podido "
              f"atribuir a ningún crate.")
        print("   No se comprueban. Acerca el nombre del crate al número, o "
              "acepta que quedan sin verificar:")
        for doc, linea, txt in huerfanas[:12]:
            print(f"     {doc}:{linea}  «{txt}»")
        if len(huerfanas) > 12:
            print(f"     … y {len(huerfanas) - 12} más")
    return 0


if __name__ == "__main__":
    sys.exit(main())
