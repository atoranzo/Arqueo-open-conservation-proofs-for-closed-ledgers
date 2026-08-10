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

⚠️ Y **no entiende prosa**: caza el patrón «número + tests» cerca del
nombre de un crate. Una afirmación rancia escrita de otra forma **se le
escapa**, y eso hay que saberlo. No es un verificador de documentación: es
una red para la cifra que ya se rancció una vez.

⚠️ **El hueco se midió y se estrechó** (§239): `PRINCIPIOS.md` decía «539
tests» **sin nombrar ningún crate**, y esta herramienta no lo veía. Ahora
también comprueba los TOTALES —una cifra de cuatro dígitos junto a
«tests»— contra la suma de los pines, que es la otra forma en que un
documento cuenta lo mismo.

El hueco no está cerrado: una frase como «unos quinientos tests» sigue
escapándose. **Lo que se puede decir es que las dos formas que ya se
rancciaron están cubiertas.**

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

# ⚠️ Lineas que cuentan un MOMENTO PASADO, no el presente. `doc/ZENODO.md`
# narra el estado del sistema cuando se deposito: «The system WAS complete:
# 373 tests». Corregir eso falsificaria el relato, igual que corregir un
# asiento. Se excluye POR LINEA, no por fichero, para que el resto del
# documento siga vigilado (§239).
LINEAS_HISTORICAS = ("The system was complete",)


def pines():
    """Lee de `tools/canon.sh` los pines de tests. NO se escriben aquí."""
    ruta = os.path.join(RAIZ, "tools", "canon.sh")
    fuera = {}
    for linea in open(ruta, encoding="utf-8"):
        m = re.match(r"^([a-z0-9-]+)\s+\w+\s+(\d+)\s+(\d+)\s+(\d+)\s", linea)
        if m:
            fuera[m.group(1)] = int(m.group(2))
    return fuera


def pines_sello():
    """Solo los pines del nivel `--sello`: la suma que se cita mas."""
    ruta = os.path.join(RAIZ, "tools", "canon.sh")
    fuera = {}
    for linea in open(ruta, encoding="utf-8"):
        m = re.match(r"^([a-z0-9-]+)\s+sello\s+(\d+)\s", linea)
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


# ── §262 · los DESGLOSES ────────────────────────────────────────
#
# ⚠️ El hueco que esto cierra: `PRINCIPIOS.md` decia «12 del verificador
#    independiente» cuando §256 lo habia dejado en 22, y VIAJO UN SELLO
#    ENTERO porque en esa forma la cifra no lleva «tests» al lado ni el
#    nombre del crate — las dos unicas formas que este fichero cazaba.
#
# ⚠️ Y por que NO se caza el patron suelto: se midio. Buscar «N <palabra>»
#    por todo el documento dio CINCO discrepancias y LAS CINCO FALSAS.
#    Una compuerta con falsos positivos se acaba ignorando, y una
#    compuerta ignorada es peor que su ausencia declarada.
#
# ⚠️ Lo que la hace solida NO es acotar por acotar: es que **el ancla ya
#    esta validada**. El desglose se lee donde el documento ya declaro un
#    total que este mismo fichero verifica desde §239. Fuera de ahi, un
#    numero junto a una palabra no afirma nada. Engancharse a una
#    comprobacion que existe en vez de crear una segunda superficie.
#
# ⚠️ ADYACENCIA, operativa y no interpretada: entre el total y el guion NO
#    PUEDE HABER UN PUNTO. Es decir, misma frase. Medido contra la entrada
#    real: sin esa regla el ancla saltaba un punto y sesenta caracteres
#    hasta OTRO parrafo de `PRINCIPIOS.md`.

FRASE_DESGLOSE = re.compile(
    r"(\d{3,4})\s*(?:tests|pruebas)([^\u2014.]*)\u2014([^\u2014]*)\u2014")
ITEM_DESGLOSE = re.compile(
    # ⚠️ EL CONECTOR ES OBLIGATORIO, y el ensayo lo exigio: sin el, el
    #    tramo que sigue a un total casa TAMBIEN OTROS TOTALES —«873
    #    contando los pines», «887 declaradas»— y da diez falsas alarmas.
    #    Los siete items reales llevan «de», «del» o «de la»; los totales
    #    y el ruido no llevan ninguno.
    r"\*{0,2}(\d{1,4})\*{0,2}\s+(?:de\s+la\s+|de\s+los\s+|del\s+|de\s+)"
    r"\*{0,2}([a-zA-Z\u00e0-\u00ff]+(?:\s+[a-zA-Z\u00e0-\u00ff]+)?)")


def alias_de_crates():
    """Lee de `tools/canon.sh` los alias EN PROSA de cada crate.

    ⚠️ Viven en LA FILA DEL CRATE y no en este fichero **a proposito**. Una
    copia aqui no duplicaria un dato: **afirmaria una correspondencia que
    nadie comprueba**. El dia que el testigo cambiara de crate, `canon.sh`
    cambiaria la fila y esto seguiria leyendo el pin viejo bajo un nombre
    que ya no corresponde — **no fallaria: validaria mal, en silencio**.
    """
    ruta = os.path.join(RAIZ, "tools", "canon.sh")
    out = {}
    for linea in open(ruta, encoding="utf-8"):
        m = re.match(r"^([a-z0-9-]+)\s+\w+\s+\d+\s+\d+\s+\d+\s+\d+\s+alias=([^\u00b7\n]+)",
                     linea)
        if m:
            out[" ".join(m.group(2).split()).lower()] = m.group(1)
    return out


def desgloses(pins, alias):
    """Comprueba las cifras POR CRATE dentro de la frase de un total.

    ⚠️ **INERTE si nadie ha declarado un alias.** Una compuerta que exige
    datos que aun no existen no es una mejora: es una parada, y con la
    causa en el sello anterior.

    ⚠️ La regla es «**todo numero que APARECE resuelve y cuadra**», no
    «todos los pines aparecen»: un desglose parcial es prosa legitima.
    """
    if not alias:
        return [], 0
    fallos, vistas = [], 0
    for rel in vivos():
        texto = open(os.path.join(RAIZ, rel), encoding="utf-8").read()
        plano = " ".join(texto.split())
        for fr in FRASE_DESGLOSE.finditer(plano):
            for it in ITEM_DESGLOSE.finditer(fr.group(3)):
                v = int(it.group(1))
                nombre = " ".join(it.group(2).split()).lower()
                crate = alias.get(nombre) or alias.get(nombre.split()[0])
                vistas += 1
                if crate is None:
                    fallos.append((rel, 0, f"desglose <{nombre}>", v,
                                   "ningun alias declarado en tools/canon.sh",
                                   fr.group(3)[:78]))
                elif pins.get(crate) != v:
                    fallos.append((rel, 0, crate, v, pins.get(crate),
                                   fr.group(3)[:78]))
    return fallos, vistas


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

    # ⚠️ TOTALES: una cifra de 3-4 digitos junto a «tests» y SIN nombre de
    # crate suele ser la suma. Se compara contra las sumas posibles —la del
    # sello, la de todos los pines— con margen cero: si no es ninguna, se
    # señala para que alguien mire, porque asi se colo el 539.
    suma_sello = sum(pines_sello().values())
    suma_todos = sum(p.values())
    posibles = {suma_sello, suma_todos}
    for rel in vivos():
        texto = open(os.path.join(RAIZ, rel), encoding="utf-8").read()
        for n, linea in enumerate(texto.split("\n"), 1):
            if any(c in linea for c in p):
                continue          # ya lo mira el bucle de arriba
            if re.search(r"--release\s+[^#\s]", linea) or "circuit_" in linea:
                continue
            if any(h in linea for h in LINEAS_HISTORICAS):
                continue
            for m in re.finditer(r"\*?\*?(\d{3,4})\*?\*?\s*(?:tests|pruebas)", linea):
                v = int(m.group(1))
                revisadas += 1
                if v not in posibles:
                    malas.append((rel, n, "TOTAL", v,
                                  f"{suma_sello} (sello) o {suma_todos} (todos)",
                                  linea.strip()[:78]))

    fallos_desglose, vistas_desglose = desgloses(p, alias_de_crates())
    malas.extend(fallos_desglose)
    revisadas += vistas_desglose

    for rel, n, crate, v, pin, l in malas:
        print(f"  RANCIA  {rel}:{n}")
        print(f"          dice {v} tests para `{crate}` y el canon pina {pin}")
        print(f"          {l}")

    if malas:
        print(f"\n{len(malas)} cifra(s) que contradicen el canon. "
              f"Un numero a mano en un .md envejece sin avisar.")
        return 1
    print(f"{revisadas} cifra(s) de tests en documentos vivos: ninguna "
          f"contradice el canon ({len(p)} pines leidos de tools/canon.sh). "
          f"De ellas, {vistas_desglose} de desglose.")
    return 0


if __name__ == "__main__":
    try:
        import signal
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    except (ImportError, AttributeError, ValueError):
        pass
    sys.exit(main())
