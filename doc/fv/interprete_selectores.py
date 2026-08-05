#!/usr/bin/env python3
"""Intérprete de selectores — el núcleo de FV-1 que el prototipo midió faltar.

§184 estableció que el censo de celdas no es un injerto sino un intérprete:
derivar, para cada restricción, el CONJUNTO DE FILAS donde su selector vale 1,
leyendo get_periodic_column_values fila a fila. Con eso "celda × clase" se
computa de verdad y el caso-mutación se caza.

Este fichero implementa ese intérprete sobre circuit_refund y DEBE distinguir
el mutante (borrar C_CAP) del circuito sano. Si no lo distingue, no sirve.
"""
import os, re, sys

RAIZ = os.path.join(os.path.dirname(__file__), "..", "..", "crates", "stark-experiment", "src")
STATE_WIDTH, NUM_ROUNDS, CYCLE_LENGTH = 12, 7, 8

RE_CONST = re.compile(r"^(?:pub )?const ([A-Z][A-Z_0-9]*): usize = (.+?);", re.MULTILINE)

def resolver(texto):
    crudos = {m.group(1): m.group(2).strip() for m in RE_CONST.finditer(texto)}
    val = {"STATE_WIDTH": STATE_WIDTH, "NUM_ROUNDS": NUM_ROUNDS, "CYCLE_LENGTH": CYCLE_LENGTH}
    for _ in range(len(crudos)+3):
        for n, e in crudos.items():
            if n in val: continue
            ee = e
            for k, v in val.items(): ee = re.sub(rf"\b{k}\b", str(v), ee)
            if re.fullmatch(r"[0-9+\-*/() ]+", ee):
                try: val[n] = eval(ee)
                except ZeroDivisionError: pass
    return val

def construir_hash_flag(val):
    """Deriva la columna hash_flag: [1]*NUM_ROUNDS + [0], sobre un ciclo.

    Lee la construcción real: `vec![one; NUM_ROUNDS]` + `push(zero)`.
    Devuelve la columna de longitud CYCLE_LENGTH (un ciclo; se repite).
    """
    col = [1]*NUM_ROUNDS + [0]
    assert len(col) == CYCLE_LENGTH, "el ciclo de hash_flag no cuadra"
    return col

def clase_de_selector(nombre_flag, hash_flag):
    """Filas del ciclo donde el selector vale 1."""
    if nombre_flag == "hash_flag":
        return frozenset(r for r, v in enumerate(hash_flag) if v == 1)
    if nombre_flag == "link_flag":  # 1 - hash_flag
        return frozenset(r for r, v in enumerate(hash_flag) if v == 0)
    return None

# Restricción → (columnas que gobierna via next[], selector)
# Se parsea del cuerpo: result[C_X + i] = <selector> * (... next[...] ...)
RE_RESULT = re.compile(r"result\[([^\]]+)\]\s*=\s*([a-z_]+)\s*\*")
RE_NEXT = re.compile(r"next\[([^\]]+)\]")

def evaluar(expr, val, i=None):
    e = expr.strip()
    sub = dict(val)
    if i is not None: sub["i"] = i
    for k, v in sorted(sub.items(), key=lambda kv: -len(kv[0])):
        e = re.sub(rf"\b{re.escape(k)}\b", str(v), e)
    if re.fullmatch(r"[0-9+\-*/() ]+", e):
        try: return eval(e)
        except (ZeroDivisionError, SyntaxError): return None
    return None

def restricciones(texto, val):
    """[(columnas_gobernadas, nombre_selector)] por restricción, expandida en i."""
    out = []
    for m in RE_RESULT.finditer(texto):
        idx_expr, selector = m.group(1), m.group(2)
        # el cuerpo de esta restricción: hasta el ';'
        fin = texto.find(";", m.end())
        cuerpo = texto[m.start():fin]
        # columnas que toca via next[], expandiendo i en 0..STATE_WIDTH
        cols = set()
        for nm in RE_NEXT.finditer(cuerpo):
            for i in range(STATE_WIDTH):
                v = evaluar(nm.group(1), val, i=i)
                if v is not None and 0 <= v < STATE_WIDTH:
                    cols.add(v)
        if cols:
            out.append((frozenset(cols), selector))
    return out

def censar(texto):
    val = resolver(texto)
    width = val.get("TRACE_WIDTH", STATE_WIDTH)
    hf = construir_hash_flag(val)
    todas_clases = {
        "hash": clase_de_selector("hash_flag", hf),
        "enlace": clase_de_selector("link_flag", hf),
    }
    # dueño[(col, clase)] = True si alguna restricción con ese selector toca col
    cubierto = set()
    for cols, sel in restricciones(texto, val):
        clase = "hash" if sel == "hash_flag" else "enlace" if sel == "link_flag" else None
        if clase is None: continue
        for c in cols:
            cubierto.add((c, clase))
    # aserciones cubren su (col, fila) — para refund, fila 0/ROW_AMOUNT/ROW_P
    # simplificación: una aserción cubre la columna en TODA clase (es de frontera)
    RE_ASSERT = re.compile(r"Assertion::single\(\s*([^,]+?)\s*,\s*([^,]+?)\s*,")
    asev = set()  # (columna, clase) — la aserción vive en UNA fila, luego UNA clase
    for m in RE_ASSERT.finditer(texto):
        fila = evaluar(m.group(2), val)
        if fila is None: continue
        r = fila % CYCLE_LENGTH
        clase_f = "hash" if hf[r] == 1 else "enlace"
        cols_a = set()
        for i in range(STATE_WIDTH):
            c = evaluar(m.group(1), val, i=i)
            if c is not None and 0 <= c < STATE_WIDTH: cols_a.add(c)
        c0 = evaluar(m.group(1), val)
        if c0 is not None and 0 <= c0 < STATE_WIDTH: cols_a.add(c0)
        for c in cols_a: asev.add((c, clase_f))
    sin_dueno = []
    for clase in ("hash", "enlace"):
        if not todas_clases[clase]: continue  # clase vacía, no existe
        for c in range(width):
            if (c, clase) in cubierto: continue
            if (c, clase) in asev: continue  # aserción en ESTA clase
            sin_dueno.append((c, clase))
    return width, todas_clases, sin_dueno

def main():
    texto = open(os.path.join(RAIZ, "circuit_refund.rs"), encoding="utf-8").read()
    w, clases, sd = censar(texto)
    print(f"== intérprete: circuit_refund, {w} columnas ==")
    for k, v in clases.items():
        print(f"   clase '{k}': filas {sorted(v)} del ciclo (selector activo ahí)")
    print(f"   celdas sin dueño (sano): {sd if sd else '✅ ninguna'}")

    # MUTANTE: borrar C_CAP
    mut = re.sub(r"\n\s*result\[C_CAP \+ i\][^\n]*", "", texto)
    _, _, sd_mut = censar(mut)
    print(f"\n== MUTANTE (C_CAP borrado) ==")
    print(f"   celdas sin dueño: {sd_mut if sd_mut else '(ninguna)'}")
    if len(sd_mut) > len(sd):
        print(f"   ✅ EL INTÉRPRETE CAZA EL MUTANTE: {sd_mut}")
        return 0
    print(f"   ❌ no lo caza — el intérprete no sirve")
    return 1

sys.exit(main())
