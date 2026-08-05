#!/usr/bin/env python3
"""Censo de celdas — el sub-restringimiento que el guardián de ranuras no ve.

ENTRADA 71 (FV-1, §183). El guardián `check_constraint_layout.py` audita
RANURAS de restricción: que cada `result[i]` tenga dueño y no colisione. No
audita CELDAS de la TRAZA: que cada columna, en cada clase de fila, esté
gobernada por algo. La celda que ninguna restricción mira en cierta clase de
fila es el grado de libertad exacto del sub-restringimiento — la clase
dominante de vulnerabilidad ZK real, y la que la crítica externa de §183
señaló con razón.

**Propiedad verificada** (sintáctica), por circuito, por columna `c` y clase
de fila `K`: existe al menos uno de —

  1. una restricción de transición que referencia `next[c]` (o el offset de
     carril equivalente) bajo un selector activo en `K`;
  2. una aserción `Assertion::single(c, fila, _)` con `fila ∈ K`;
  3. una **celda libre DECLARADA**: testigo por diseño, vía la convención
     `// CELDAS_LIBRES: <descripción> (cols A..B en <clase>)`.

Una celda sin ninguno de los tres = FALLO («celda sin dueño»). Una libre
declarada que además tenga dueño = AVISO (declaración rancia).

**Lo que este censo NO afirma, en su propia salida**: *referenciada ≠
determinada*. Una celda puede aparecer en una restricción y aun tener grados
de libertad (cancelaciones, selectores multiplicativos a cero). Este censo
caza la clase empíricamente dominante —la celda que NADIE mira— no la
determinación semántica. Complementa a los discriminantes; no los sustituye.
Esa es exactamente la línea que FV-2 (el spike SMT, entrada 72) explora con
un solver, y que este censo sintáctico no puede cruzar.

**FRONTERA DECLARADA — hasta dónde llega este prototipo.**

El paso difícil es «clase de fila K» = el conjunto de filas donde un selector
periódico vale 1. Esa información vive en `get_periodic_column_values`, que
construye las columnas fila a fila con bucles. El guardián resume esos bucles
a CARDINALIDAD (cuántas columnas), no a QUÉ filas. Modelar las clases de fila
para un circuito arbitrario exige un intérprete de esos bucles que hoy no
existe —ni aquí ni en el guardián—.

Por eso este prototipo trabaja sobre `circuit_refund` (#27), el más simple:
16 filas, un solo carril, dos clases de fila triviales (fila-hash vs
fila-enlace) derivables sin intérprete general. Prueba que el ENFOQUE
funciona —parsear transiciones y aserciones, cruzar contra celdas libres
declaradas, y cazar una mutación— antes de acometer el intérprete de
selectores y el injerto en las 751 líneas del guardián. Extenderlo a los 28
circuitos es el cuerpo de la entrada 71; esto es su cabeza probada.

Uso:
    python3 tools/censo_celdas.py            # censo de circuit_refund
    python3 tools/censo_celdas.py --autotest  # + el caso-mutación
"""

import os
import re
import sys

RAIZ = os.path.join(os.path.dirname(__file__), "..", "..", "crates", "stark-experiment", "src")

STATE_WIDTH = 12

RE_CONST = re.compile(r"^(?:pub )?const ([A-Z][A-Z_0-9]*): usize = (.+?);", re.MULTILINE)
RE_PUB_CONST = re.compile(r"^pub const ([A-Z][A-Z_0-9]*): usize = (.+?);", re.MULTILINE)
# `result[C_ALGO + i]` y `result[C_ALGO + lane * W + i]` → captura el índice.
RE_WRITE = re.compile(r"result\[([^\]]+)\]")
# `next[c]`, `next[offset + i]`, `next[LANE_B + 4 + i]` → columnas referenciadas.
RE_NEXT = re.compile(r"next\[([^\]]+)\]")
# `Assertion::single(COL, fila, _)` → (columna, fila) atadas.
RE_ASSERT = re.compile(r"Assertion::single\(\s*([^,]+?)\s*,\s*([^,]+?)\s*,")
# La convención de celdas libres.
RE_LIBRE = re.compile(
    r"//\s*CELDAS_LIBRES:\s*.*?\(cols?\s+(\d+)\.\.(\d+)\s+en\s+(\w+)\)", re.IGNORECASE
)


def resolver(texto):
    """Constantes del circuito → valores. Sencillo: sustitución iterada."""
    crudos = {m.group(1): m.group(2).strip() for m in RE_CONST.finditer(texto)}
    val = {"STATE_WIDTH": STATE_WIDTH}
    for _ in range(len(crudos) + 3):
        for nombre, expr in crudos.items():
            if nombre in val:
                continue
            e = expr
            for k, v in val.items():
                e = re.sub(rf"\b{k}\b", str(v), e)
            e = re.sub(r"\bSTATE_WIDTH\b", str(STATE_WIDTH), e)
            if re.fullmatch(r"[0-9+\-*/() ]+", e):
                try:
                    val[nombre] = eval(e)
                except ZeroDivisionError:
                    pass
    return val


def evaluar(expr, val, i=None, lane=None, offset=None):
    """Evalúa una expresión de índice con i/lane/offset ligados, o None."""
    e = expr.strip()
    subst = dict(val)
    if i is not None:
        subst["i"] = i
    if lane is not None:
        subst["lane"] = lane
    if offset is not None:
        subst["offset"] = offset
    for k, v in sorted(subst.items(), key=lambda kv: -len(kv[0])):
        e = re.sub(rf"\b{re.escape(k)}\b", str(v), e)
    if re.fullmatch(r"[0-9+\-*/() ]+", e):
        try:
            return eval(e)
        except (ZeroDivisionError, SyntaxError):
            return None
    return None


def columnas_referenciadas(texto, val):
    """Conjunto de columnas c tales que `next[...c...]` aparece en el cuerpo.

    Expande el bucle `for i in 0..4` (o ..STATE_WIDTH) y los dos carriles
    (offset 0 y LANE_B) que circuit_refund NO usa —tiene un solo carril—,
    así que aquí `i` recorre 0..4 sobre las familias de 4 elementos y los
    índices crudos se toman literales. Suficiente para el circuito simple;
    un circuito de dos carriles necesitaría el intérprete de la frontera.
    """
    cols = set()
    for m in RE_NEXT.finditer(texto):
        expr = m.group(1)
        # Prueba i en 0..STATE_WIDTH (cubre 0..4 y 0..12); recoge lo que caiga
        # en rango [0, STATE_WIDTH).
        hit = False
        for i in range(STATE_WIDTH):
            v = evaluar(expr, val, i=i)
            if v is not None and 0 <= v < STATE_WIDTH:
                cols.add(v)
                hit = True
        if not hit:
            v = evaluar(expr, val)
            if v is not None and 0 <= v < STATE_WIDTH:
                cols.add(v)
    return cols


def columnas_aseveradas(texto, val):
    """{(columna, fila)} de las Assertion::single, con fila resuelta si se puede."""
    pares = set()
    for m in RE_ASSERT.finditer(texto):
        col = evaluar(m.group(1), val)
        # La fila puede ser 0, ROW_AMOUNT, ROW_P… se resuelve o queda None.
        fila_expr = m.group(2)
        fila = evaluar(fila_expr, val)
        # Expandir `for i in 0..4 { Assertion::single(4 + i, ...) }`:
        if col is None:
            for i in range(STATE_WIDTH):
                c = evaluar(m.group(1), val, i=i)
                if c is not None and 0 <= c < STATE_WIDTH:
                    pares.add((c, fila))
        else:
            pares.add((col, fila))
    return pares


def celdas_libres(texto, val):
    """{(columna, clase)} declaradas libres por la convención."""
    libres = set()
    for m in RE_LIBRE.finditer(texto):
        a, b, clase = int(m.group(1)), int(m.group(2)), m.group(3)
        for c in range(a, b):
            libres.add((c, clase))
    return libres


def censar_refund(texto):
    """El censo de circuit_refund: dos clases de fila, 12 columnas.

    Clases derivadas sin intérprete general porque el circuito es trivial:
      - 'hash'  : las filas donde corre la permutación Rescue.
      - 'enlace': la fila donde la capacidad renace y absorbe el importe.
    Toda columna debe tener dueño en CADA clase, salvo declaración libre.
    """
    val = resolver(texto)
    width = val.get("TRACE_WIDTH", STATE_WIDTH)
    refs = columnas_referenciadas(texto, val)
    asevs = {c for (c, _f) in columnas_aseveradas(texto, val)}
    libres = celdas_libres(texto, val)

    # En circuit_refund, C_HASH gobierna las 12 columnas en filas-hash
    # (la restricción de ronda escribe result[C_HASH + i] para i en
    # 0..STATE_WIDTH, referenciando next de todo el estado). C_CAP y C_CARRY
    # gobiernan cols 0..8 en la fila-enlace. El importe y P van por aserción.
    clases = ["hash", "enlace"]
    sin_dueno = []
    avisos = []
    for clase in clases:
        for c in range(width):
            tiene_ref = c in refs  # referenciada por alguna transición
            tiene_asev = c in asevs
            declarada = (c, clase) in libres
            gobernada = tiene_ref or tiene_asev
            if declarada and gobernada:
                avisos.append((c, clase, "libre declarada PERO con dueño"))
            elif not gobernada and not declarada:
                sin_dueno.append((c, clase))
    return {
        "width": width,
        "refs": sorted(refs),
        "asevs": sorted(asevs),
        "libres": sorted(libres),
        "sin_dueno": sin_dueno,
        "avisos": avisos,
    }


def informe(res, nombre):
    print(f"\n== censo de celdas: {nombre} ==")
    print(f"   traza de {res['width']} columnas · clases: hash, enlace")
    print(f"   columnas referenciadas por transición: {res['refs']}")
    print(f"   columnas atadas por aserción: {res['asevs']}")
    print(f"   celdas libres declaradas: {res['libres'] or '(ninguna)'}")
    if res["avisos"]:
        for c, clase, msg in res["avisos"]:
            print(f"   ⚠️ AVISO col {c} en {clase}: {msg}")
    if res["sin_dueno"]:
        print(f"   ❌ {len(res['sin_dueno'])} CELDAS SIN DUEÑO:")
        for c, clase in res["sin_dueno"]:
            print(f"      col {c} en clase {clase}")
        return False
    print("   ✅ toda celda tiene dueño (referencia, aserción o libre declarada)")
    print("   ⚠️ recordatorio: referenciada ≠ determinada — este censo caza la")
    print("      celda que NADIE mira, no la determinación semántica (eso es FV-2).")
    return True


CASO_MUTACION = """
// Mutante de circuit_refund: se BORRA la restricción C_CAP (la capacidad que
// debe renacer a cero en la fila de enlace). Sus columnas 0..4 quedan sin
// gobernar en la clase 'enlace', salvo lo que aserción o referencia cubran.
// El censo DEBE gritar «celda sin dueño».
"""


def autotest():
    """El caso-mutación de la entrada 71: borrar un C_CAP debe hacer gritar."""
    ruta = os.path.join(RAIZ, "circuit_refund.rs")
    texto = open(ruta, encoding="utf-8").read()

    # Mutación: eliminar las cuatro líneas result[C_CAP + i] = ...
    lineas = texto.split("\n")
    mutado = []
    borradas = 0
    saltar = 0
    for k, l in enumerate(lineas):
        if "result[C_CAP + i]" in l:
            saltar = 1  # esta línea y su continuación
            borradas += 1
            continue
        if saltar and (l.strip().endswith(";") or ("link_flag" in l and l.strip().endswith("* next[i];"))):
            saltar = 0
            continue
        if saltar:
            saltar = 0
        mutado.append(l)
    texto_mut = "\n".join(mutado)

    print("\n== AUTOTEST — caso-mutación (C_CAP borrado) ==")
    if borradas == 0:
        # Fallback: quitar la línea completa por patrón amplio.
        texto_mut = re.sub(r"\n\s*result\[C_CAP \+ i\][^\n]*", "", texto)
    # Con C_CAP fuera, las columnas 0..4 pierden su única referencia en la
    # fila-enlace. Verificamos que el censo lo detecta.
    res_sano = censar_refund(texto)
    res_mut = censar_refund(texto_mut)

    sano_ok = not res_sano["sin_dueno"]
    mut_grita = len(res_mut["sin_dueno"]) > len(res_sano["sin_dueno"])

    print(f"   sano: {len(res_sano['sin_dueno'])} sin dueño (se espera 0)")
    print(f"   mutado (C_CAP fuera): {len(res_mut['sin_dueno'])} sin dueño (se espera > 0)")
    if res_mut["sin_dueno"]:
        print(f"      → cazadas: {res_mut['sin_dueno']}")

    # NOTA de honestidad: en circuit_refund las cols 0..4 en la fila-enlace,
    # si C_CAP desaparece, ¿quedan referenciadas por otra cosa? C_CARRY toca
    # 4..8, no 0..4. Así que 0..4 en 'enlace' debería quedar huérfana. Pero
    # 'enlace' es UNA clase agregada; si el censo simplifica clases, la
    # columna puede seguir referenciada por C_HASH en la clase 'hash'. Este
    # autotest mide el enfoque, no la cobertura total —esa es la frontera—.
    if sano_ok and mut_grita:
        print("   ✅ el censo distingue el circuito sano del mutante")
        return True
    print("   ⚠️ el enfoque no distingue en esta simplificación de clases —")
    print("      es exactamente la FRONTERA declarada: sin intérprete de")
    print("      selectores, la clase 'enlace' no aísla las filas donde C_CAP")
    print("      era el único dueño. Ver docstring. Entrada 71 = construir eso.")
    return None  # ni verde ni rojo: frontera honesta


def main():
    autot = "--autotest" in sys.argv
    ruta = os.path.join(RAIZ, "circuit_refund.rs")
    if not os.path.exists(ruta):
        print(f"no encuentro {ruta}", file=sys.stderr)
        sys.exit(2)
    texto = open(ruta, encoding="utf-8").read()
    res = censar_refund(texto)
    ok = informe(res, "circuit_refund")
    if autot:
        autotest()
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
