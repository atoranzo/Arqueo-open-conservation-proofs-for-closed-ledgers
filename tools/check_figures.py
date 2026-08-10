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

# ⚠️ UNA lista de excluidos, no dos. Los REGISTROS que quedan fuera son los
# mismos que los de `check_cifras.py` y se IMPORTAN de alli en vez de
# copiarse: dos copias del mismo criterio y una miente (§217). Si alguien
# anade un preprint a esa lista, esta herramienta se entera sola.
#
# ⚠️ Y la exclusion TIENE CONSECUENCIA, dicha aqui y no deducida: las cifras
# de tests dentro de `AUDITORIA.md` y `BACKLOG.md` dejan de vigilarse **para
# siempre**. Es lo correcto —un asiento describe un momento y no se
# reescribe— pero se paga con ceguera, asi que la premisa se COMPRUEBA:
# ver `premisa_de_la_exclusion()`.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from check_cifras import EXCLUIDOS, PREFIJOS_EXCLUIDOS  # noqa: E402


def premisa_de_la_exclusion():
    """Excluir `BACKLOG.md` solo es honesto mientras sus cifras de tests
    vivan en notas CERRADAS —que narran un sello hecho—. Una cifra dentro
    de una entrada ABIERTA es justo la clase que esta herramienta existe
    para cazar, y la exclusion la dejaria muda. Medido al escribir esto:
    CERO. Si deja de serlo, se dice."""
    try:
        texto = open("BACKLOG.md", encoding="utf-8").read()
    except OSError:
        return
    notas, cur = [], None
    for linea in texto.splitlines():
        m = re.match(r"^- \[( |x)\] \*\*(\d+(?:-[A-Z])?)\.", linea)
        if m:
            if cur:
                notas.append(cur)
            cur = {"abierta": m.group(1) == " ", "num": m.group(2), "txt": [linea]}
        elif cur is not None:
            if linea.startswith("## ") or linea.startswith("---"):
                notas.append(cur)
                cur = None
            else:
                cur["txt"].append(linea)
    if cur:
        notas.append(cur)
    rotas = []
    for n in notas:
        if not n["abierta"]:
            continue
        cuerpo = "\n".join(n["txt"])
        for patron, _ in PATRONES:
            if re.search(patron, cuerpo):
                rotas.append(n["num"])
                break
    if rotas:
        print("  !! BACKLOG.md tiene cifras de tests en entradas ABIERTAS: "
              + ", ".join(sorted(set(rotas))))
        print("     La exclusion las deja sin vigilar. O se quita la cifra de")
        print("     la entrada, o la exclusion deja de ser del fichero entero.")
        sys.exit(1)
    print("  OK  premisa de la exclusion: cero cifras de tests en entradas abiertas")


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


# ⚠️ **Los patrones son ESTRECHOS a proposito.**
#
# Se probaron mas anchos —incluyendo `cargo test -p CRATE … N tests` y filas
# de tabla— y encontraron **2 cifras rancias reales y 3 falsos positivos**.
#
# Los falsos vienen de que los documentos hablan de cuentas en varios
# sentidos: tests que EXISTEN, tests que FALLAN en modo depuracion, y tests
# de una lista comparativa. Un patron por proximidad no los distingue.
#
# **Tres falsos de cinco entrena a ignorar la herramienta**, que es peor que
# una cobertura corta declarada. Las cifras que no se atribuyen se listan al
# final para que quien lea sepa exactamente que queda sin comprobar.
PATRONES = [
    (r"(\d+) tests? en `?([a-z_-]+)`?", "num-primero"),
    (r"`([a-z_]+)`[^.\n]{0,25}?(\d+) tests", "mod-primero"),
    (r"(\d+) tests?\b[^.\n]{0,20}?`([a-z_]+)`", "num-primero"),
]


def main():
    premisa_de_la_exclusion()
    real = cuentas_reales()
    docs = [f for f in os.listdir(".") if f.endswith(".md")]
    if os.path.isdir("doc"):
        docs += ["doc/" + f for f in os.listdir("doc") if f.endswith(".md")]
    docs = [d for d in docs
            if d not in EXCLUIDOS and not d.startswith(PREFIJOS_EXCLUIDOS)]

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
