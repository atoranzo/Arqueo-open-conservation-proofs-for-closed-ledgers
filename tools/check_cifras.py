#!/usr/bin/env python3
"""Comprueba que **ningún documento vivo contradiga los pines del canon**.

## Por qué existe (§237)

`ARQUITECTURA.md` llevaba **veintisiete sellos** diciendo que la capa tenía
172 tests. Tenía 256. `PAPER.md` y `PAPER_EN.md` decían lo mismo, y que el
backend STARK añadía 5 circuitos cuando añade 18.

Ninguna compuerta lo veía, porque **el canon vigila el código y nadie
vigilaba la prosa**. Un número escrito a mano en un `.md` envejece sin
avisar, y quien lo lee no tiene forma de saber que envejeció.

## Qué comprueba, exactamente

Para cada crate con pin en `tools/canon.sh`, busca en los documentos VIVOS
frases del tipo «<crate> … <N> tests» y exige que `N` sea el pin.

**No inventa el pin: lo LEE de `tools/canon.sh`.** Si el canon cambia, esto
cambia con él y sin tocar nada.

## Qué NO comprueba, y por qué

⚠️ **Los REGISTROS quedan fuera, a propósito**:

- `AUDITORIA.md` — los asientos describen **un momento** y no se reescriben.
- `BACKLOG.md` — los números no se reutilizan ni se renumeran.
- `doc/preprints/*` — **depositados con DOI**, conservados verbatim
  (`doc/preprints/ERRATA.md`).
- `doc/ESCALADO.md`, `doc/CONFIANZA_RESIDUAL.md` — **cuerpo VERBATIM** con
  cabecera-mapa encima (§120, decisión A).
- `spec/rfc/*` — un RFC es un documento fechado.

Corregir una cifra ahí **no sería transparencia: sería falsificar el
referente** de las correcciones que ya se publicaron sobre esos textos. Es
la regla que el propio repositorio se dio, y esta herramienta la respeta.

⚠️ Y **no entiende prosa**: solo caza el patrón «número + tests» cerca del
nombre de un crate. Una afirmación rancia escrita de otra forma **se le
escapa**, y eso hay que saberlo. No es un verificador de documentación: es
una red para la cifra que ya se rancció una vez.

## Uso

    python3 tools/check_cifras.py          # exit 0 si nada contradice
"""

import os
import re
import sys

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# ⚠️ REGISTROS: no se tocan. Ver la cabecera.
EXCLUIDOS = {
    "AUDITORIA.md",
    "BACKLOG.md",
    "doc/ESCALADO.md",
    "doc/CONFIANZA_RESIDUAL.md",
}
PREFIJOS_EXCLUIDOS = ("doc/preprints/", "spec/rfc/")


def pines():
    """Lee de `tools/canon.sh` los pines de tests. NO se escriben aquí."""
    ruta = os.path.join(RAIZ, "tools", "canon.sh")
    fuera = {}
    for linea in open(ruta, encoding="utf-8"):
        m = re.match(r"^([a-z0-9-]+)\s+\w+\s+(\d+)\s+(\d+)\s+(\d+)\s", linea)
        if m:
            fuera[m.group(1)] = int(m.group(2))
    return fuera


def vivos():
    for base, dirs, fs in os.walk(RAIZ):
        dirs[:] = [d for d in dirs if d not in {".git", "target", ".canon"}]
        for f in fs:
            if not f.endswith(".md"):
                continue
            rel = os.path.relpath(os.path.join(base, f), RAIZ)
            if rel in EXCLUIDOS or rel.startswith(PREFIJOS_EXCLUIDOS):
                continue
            yield rel


def main():
    p = pines()
    if not p:
        print("ROJO: no se pudo leer ningun pin de tools/canon.sh")
        return 2

    malas = []
    revisadas = 0
    for rel in vivos():
        texto = open(os.path.join(RAIZ, rel), encoding="utf-8").read()
        for n, linea in enumerate(texto.split("\n"), 1):
            for crate, pin in p.items():
                # «<crate> ... <N> tests» en la misma linea, en cualquier orden.
                if crate not in linea:
                    continue
                # ⚠️ Una linea que filtra por un test o un circuito CONCRETO
                # no es una cuenta del crate: `cargo test -p zk-core
                # circuit_audit` dice cuantos tiene ESE circuito. Fue un
                # falso positivo real (§237), y un falso positivo hace que
                # la herramienta se deje de mirar.
                # El filtro va DESPUES de --release y NO empieza por `#`:
                # `... --release circuit_audit` es un filtro;
                # `... --release   # 242 tests` es un comentario.
                if re.search(r"--release\s+[^#\s]", linea) or "circuit_" in linea:
                    continue
                for m in re.finditer(r"(\d[\d.]*)\s*(?:tests|pruebas)", linea):
                    revisadas += 1
                    v = int(m.group(1).replace(".", ""))
                    if v != pin:
                        malas.append((rel, n, crate, v, pin, linea.strip()[:78]))

    for rel, n, crate, v, pin, l in malas:
        print(f"  RANCIA  {rel}:{n}")
        print(f"          dice {v} tests para `{crate}` y el canon pina {pin}")
        print(f"          {l}")

    if malas:
        print(f"\n{len(malas)} cifra(s) que contradicen el canon. "
              f"Un numero a mano en un .md envejece sin avisar.")
        return 1
    print(f"{revisadas} cifra(s) de tests en documentos vivos: ninguna "
          f"contradice el canon ({len(p)} pines leidos de tools/canon.sh).")
    return 0


if __name__ == "__main__":
    try:
        import signal
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    except (ImportError, AttributeError, ValueError):
        pass
    sys.exit(main())
