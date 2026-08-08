#!/usr/bin/env python3
"""Caza ficheros `.rs` bajo `src/` que **no declara nadie**.

## Por qué existe (§227)

`crates/zk-core/src/marlin_proof_system.rs` estuvo ahí sin estar declarado en
`lib.rs`. Nadie lo notó, y cuando se notó se le atribuyó lo que no era: se
selló dos veces como «código muerto sin cablear» cuando en realidad era una
**investigación descartada** que ni siquiera compilaba —importaba cuatro
dependencias que no están en ningún `Cargo.toml`—.

Las dos cosas son un problema, y la segunda peor que la primera:

1. Un `.rs` sin declarar **no se compila**, así que nada lo verifica: puede
   estar roto durante años sin que ninguna compuerta lo sepa.
2. Y como no se compila, **invita a especular sobre él**. Lo que no se
   compila no se puede afirmar.

## La regla

Un fichero `src/a/b/c.rs` tiene que estar declarado como `mod c;` (o
`pub mod c;`) en su padre: `src/a/b.rs` o `src/a/b/mod.rs`. Y `src/x.rs`, en
`src/lib.rs` o `src/main.rs`.

Solo se mira `src/`. `tests/`, `examples/` y `benches/` son **objetivos
aparte**, no módulos: cargo los compila por su cuenta y no necesitan
declaración.

⚠️ Y `src/bin/` **también es especial**: cada `.rs` de ahí es un BINARIO
independiente, no un módulo. `gen_openrpc.rs` —el que genera
`spec/openrpc.json`— vive ahí. La primera versión de esta herramienta lo
señalaba como huérfano: un falso positivo, que es exactamente lo que hizo
que nadie mirase `check_tests.py` durante seis sellos. Se excluye.

⚠️ **Límite conocido**: no entiende `#[path = "..."]`. Si alguien lo usa,
esta herramienta dará un falso positivo — y entonces la respuesta es
declararlo aquí, no callar la compuerta.

## Uso

    python3 tools/check_modulos.py          # exit 0 si todo esta declarado
"""

import os
import re
import sys

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# Ficheros que son la RAIZ de un arbol de modulos: no los declara nadie.
RAICES = {"lib.rs", "main.rs", "mod.rs"}
# `src/bin/` es un directorio ESPECIAL de cargo: un binario por fichero, no
# modulos. Nadie los declara y es correcto.
CARPETAS_APARTE = {"bin"}


def crates():
    base = os.path.join(RAIZ, "crates")
    for d in sorted(os.listdir(base)):
        src = os.path.join(base, d, "src")
        if os.path.isdir(src):
            yield d, src


def declara(fichero, nombre):
    """¿`fichero` declara `mod nombre;`?"""
    if not os.path.isfile(fichero):
        return False
    txt = open(fichero, encoding="utf-8").read()
    # `mod x;` o `pub mod x;` o `pub(crate) mod x;`, con o sin cfg delante.
    return re.search(rf"^\s*(pub(\([^)]*\))?\s+)?mod\s+{re.escape(nombre)}\s*;",
                     txt, re.M) is not None


def padres_de(ruta, src):
    """Los dos ficheros que PODRIAN declarar `ruta`."""
    carpeta = os.path.dirname(ruta)
    if os.path.abspath(carpeta) == os.path.abspath(src):
        return [os.path.join(src, "lib.rs"), os.path.join(src, "main.rs")]
    # src/a/b/c.rs -> lo declara src/a/b.rs o src/a/b/mod.rs
    return [carpeta + ".rs", os.path.join(carpeta, "mod.rs")]


def main():
    huerfanos = []
    total = 0
    for crate, src in crates():
        for base, dirs, ficheros in os.walk(src):
            # No se desciende a src/bin/: son objetivos, no modulos.
            if os.path.abspath(base) == os.path.abspath(src):
                dirs[:] = [d for d in dirs if d not in CARPETAS_APARTE]
            for f in sorted(ficheros):
                if not f.endswith(".rs") or f in RAICES:
                    continue
                total += 1
                ruta = os.path.join(base, f)
                nombre = f[:-3]
                if not any(declara(p, nombre) for p in padres_de(ruta, src)):
                    huerfanos.append((crate, os.path.relpath(ruta, RAIZ),
                                      [os.path.relpath(p, RAIZ)
                                       for p in padres_de(ruta, src)]))

    for crate, rel, padres in huerfanos:
        print(f"  HUERFANO  {rel}")
        print(f"            nadie lo declara. Deberia estar como `mod "
              f"{os.path.basename(rel)[:-3]};` en: {' o '.join(padres)}")
        print(f"            Si NO es codigo, no debe vivir en src/.")

    if huerfanos:
        print(f"\n{len(huerfanos)} fichero(s) .rs que nadie declara, de {total} "
              f"bajo src/. Lo que no se compila, no se verifica.")
        return 1
    print(f"{total} ficheros .rs bajo src/: todos declarados por su padre.")
    return 0


if __name__ == "__main__":
    # ⚠️ SIGPIPE: si alguien canaliza esto por `head`, el `print` revienta con
    # BrokenPipeError y la compuerta muere por una razon que no es la suya.
    # Una compuerta que falla por como se la lee no vigila nada.
    try:
        import signal
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    except (ImportError, AttributeError, ValueError):
        pass  # en Windows no existe; alli no hay tuberia que romper
    sys.exit(main())
