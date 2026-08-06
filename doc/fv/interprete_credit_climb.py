#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Intérprete MULTI-CICLO del CRÉDITO — hermano derivado (resto de la 71, §191).

Quinta pieza de la estirpe FV-1; derivado ANCLADO del intérprete de mint
(las 12 anclas del constructor de periódicas son idénticas en ambos — se
midió antes de derivar). El sujeto es circuit_credit_climb: periódicas de longitud-de-traza (512 filas)
en CUATRO familias que el intérprete de ciclo corto no sabía leer (doc §9):
  1. hash_flag multi-ciclo   r % CYCLE_LENGTH < NUM_ROUNDS  Y  r <= ROW_ACCT_ROOT
  2. one-hots de árbol       acct_link una-por-nivel; link_leaf/link_salt puntuales
  3. selector de fila-0      first_row
  4. segmentos Horner        first_s / cont_s / seg_link por segmento

Doctrina (doc/VERIFICACION_FORMAL.md §1): dueño de la celda (col, clase) =
restricción que referencia next[col] bajo selector activo en la clase, o
aserción con fila en la clase (§185: la aserción tiene clase), o declaración
CELDAS_LIBRES en el propio circuito. Libre sin declarar = SIN DUEÑO (fallo).
Declarada que además tiene dueño = aviso rancio.

El riesgo declarado del mapa se asume de entrada: la clase deja de ser un
patrón de ciclo y pasa a ser la FIRMA de selectores activos en cada paso,
computada sobre 0..TRACE_LENGTH-2 (pasos de transición).

Cosecha CONSERVADORA (§186, la trampa del alias, resuelta y extendida):
  · alias de columna      let x = next[EXPR];
  · alias de selector     let any_link = acct_link;
  · arrays de constantes  let transport = [COL_A, ...];  y el deref *col
  · bucles reales anidados (pila de ámbitos por llaves), nunca i=0..12 a ciegas
  · forma-hash: si un result con selector no toca next en su sentencia y vive
    en el bucle de carril, se cosecha el BLOQUE del carril (los next[offset+j]
    de la construcción de b).
Lo no cosechado (p. ej. sib_a, alias compuesto) jamás SUMA cobertura: el
sesgo es hacia el rojo, nunca hacia el falso verde (§59.2).

FV-1 no afirma determinación: referenciada ≠ determinada (doc §1). Este
intérprete caza la celda que NADIE mira, no los grados de libertad.
"""
import os, re, sys

RAIZ = os.path.join(os.path.dirname(__file__), "..", "..", "crates", "stark-experiment", "src")
FICHERO = "circuit_credit_climb.rs"
EXTERNAS = {"STATE_WIDTH": 12, "NUM_ROUNDS": 7, "CYCLE_LENGTH": 8, "TREE_DEPTH": 32}

# Anclas verbatim del constructor de periódicas, con su conteo exacto: si el
# circuito cambia de forma, esto grita antes de modelar máscaras rancias.
ANCLAS = [
    ("for r in 0..=ROW_ACCT_ROOT", 2),  # hash_flag + bucle de ARK
    ("if r % CYCLE_LENGTH < NUM_ROUNDS {", 1),
    ("hash_flag[r] = one;", 1),
    ("acct_link[ROW_LEAF_DONE] = one;", 1),
    ("for level in 0..TREE_DEPTH - 1 {", 1),
    ("acct_link[(CYC_ACC + level) * CYCLE_LENGTH + 7] = one;", 1),
    ("link_leaf[ROW_LEAF_LINK] = one;", 1),
    ("link_salt[ROW_SALT_LINK] = one;", 1),
    ("for row in [0] {", 1),
    ("first_s[seg * SEGMENT_LENGTH] = one;", 1),
    ("cont_s[seg * SEGMENT_LENGTH + p] = one;", 1),
    ("link[(seg + 1) * SEGMENT_LENGTH - 2] = one;", 1),
]

RE_CONST = re.compile(r"^(?:pub )?const ([A-Z][A-Z_0-9]*): usize = (.+?);", re.MULTILINE)
RE_RESULT = re.compile(r"result\[([^\]]+)\]\s*=")
RE_NEXT = re.compile(r"next\[([^\]]+)\]")
RE_SEGSEL = re.compile(r"periodic\[\s*P_SEG_LINK\s*\+\s*(\w+)\s*\]")
RE_SEL_BIND = re.compile(r"let\s+(\w+)\s*=\s*periodic\[(P_\w+)\]\s*;")
RE_ALIAS_NEXT = re.compile(r"let\s+(\w+)\s*=\s*next\[([^\]]+)\]\s*;")
RE_ALIAS_SEL = re.compile(r"let\s+(\w+)\s*=\s*(\w+)\s*;")
RE_ARRAY = re.compile(r"let\s+(\w+)\s*=\s*\[\s*([A-Z_0-9\s,]+)\]\s*;", re.DOTALL)
RE_ASSERT = re.compile(r"Assertion::single\(\s*([^,]+?)\s*,\s*([^,]+?)\s*,")
RE_FOR_LANE = re.compile(r"for\s*\(\s*lane\s*,\s*offset\s*\)\s*in\s*\[[^\]]*\]\s*\{")
RE_FOR_RANGE = re.compile(r"for\s+(\w+)\s+in\s+([A-Za-z_0-9]+)\s*\.\.(=?)\s*([A-Za-z_0-9]+)\s*\{")
RE_FOR_ARR = re.compile(r"for\s*\(\s*(\w+)\s*,\s*(\w+)\s*\)\s*in\s+(\w+)\s*\.iter\(\)\.enumerate\(\)\s*\{")
RE_LIBRE_LINEA = re.compile(r"//\s*CELDAS_LIBRES:(.*)")
RE_LOCUS = re.compile(
    r"\(\s*(clase|fila)\s+([^,)]+?)\s*,\s*cols?\s+([0-9]+)\s*(?:\.\.\s*([0-9]+))?\s*\)")

P2SEL = {"P_HASH_FLAG": "hash_flag", "P_ACCT_LINK": "acct_link",
         "P_LINK_LEAF": "link_leaf", "P_LINK_SALT": "link_salt",
         "P_FIRST_ROW": "first_row", "P_FIRST_S": "first_s", "P_CONT_S": "cont_s"}


def resolver(texto):
    crudos = {m.group(1): m.group(2).strip() for m in RE_CONST.finditer(texto)}
    val = dict(EXTERNAS)
    for _ in range(len(crudos) + 3):
        for n, e in crudos.items():
            if n in val:
                continue
            ee = e
            for k, v in sorted(val.items(), key=lambda kv: -len(kv[0])):
                ee = re.sub(rf"\b{k}\b", str(v), ee)
            if re.fullmatch(r"[0-9+\-*/() ]+", ee):
                try:
                    val[n] = eval(ee)
                except ZeroDivisionError:
                    pass
    return val


def ev(expr, val, extra=None):
    e = expr.strip()
    if e.startswith("*"):  # deref de array: next[*col]
        e = e[1:].strip()
    s = dict(val)
    if extra:
        for k, v in extra.items():
            if v is not None:
                s[k] = v
    for k, v in sorted(s.items(), key=lambda kv: -len(kv[0])):
        e = re.sub(rf"\b{re.escape(k)}\b", str(v), e)
    if re.fullmatch(r"[0-9+\-*/() ]+", e):
        try:
            return eval(e)
        except (ZeroDivisionError, SyntaxError):
            return None
    return None


def mascaras(v):
    pasos = range(v["TRACE_LENGTH"] - 1)  # transiciones: 0..T-2
    m = {}
    m["hash_flag"] = frozenset(
        r for r in pasos
        if r <= v["ROW_ACCT_ROOT"] and r % v["CYCLE_LENGTH"] < v["NUM_ROUNDS"])
    m["acct_link"] = frozenset(
        {v["ROW_LEAF_DONE"]}
        | {(v["CYC_ACC"] + lv) * v["CYCLE_LENGTH"] + 7 for lv in range(v["TREE_DEPTH"] - 1)})
    m["link_leaf"] = frozenset({v["ROW_LEAF_LINK"]})
    m["link_salt"] = frozenset({v["ROW_SALT_LINK"]})
    m["first_row"] = frozenset({0})
    m["first_s"] = frozenset(s * v["SEGMENT_LENGTH"] for s in range(v["NUM_SEGMENTS"]))
    m["cont_s"] = frozenset(
        s * v["SEGMENT_LENGTH"] + p
        for s in range(v["NUM_SEGMENTS"]) for p in range(v["SEGMENT_LENGTH"] - 1))
    for s in range(v["NUM_SEGMENTS"]):
        m["seg_link%d" % s] = frozenset({(s + 1) * v["SEGMENT_LENGTH"] - 2})
    return m


def clasificar(masc, v):
    firmas = {}
    for r in range(v["TRACE_LENGTH"] - 1):
        sig = frozenset(n for n, filas in masc.items() if r in filas)
        firmas.setdefault(sig, []).append(r)
    clases, fila2clase = {}, {}
    for sig, filas in firmas.items():
        nombre = "+".join(sorted(sig)) if sig else "plana"
        clases[nombre] = (sig, sorted(filas))
        for r in filas:
            fila2clase[r] = nombre
    return clases, fila2clase


def cuerpo_fn(texto, firma_fn):
    i = texto.find(firma_fn)
    assert i >= 0, "no se halla " + firma_fn
    j = texto.find("{", i)
    prof = 0
    for k in range(j, len(texto)):
        if texto[k] == "{":
            prof += 1
        elif texto[k] == "}":
            prof -= 1
            if prof == 0:
                return texto[j:k + 1]
    raise AssertionError("llaves sin cerrar en " + firma_fn)


def ambitos(cuerpo, val, arrays):
    abre = {}
    for m in RE_FOR_LANE.finditer(cuerpo):
        abre[m.end() - 1] = ("carril",)
    for m in RE_FOR_RANGE.finditer(cuerpo):
        lo, hi = ev(m.group(2), val), ev(m.group(4), val)
        if lo is None or hi is None:
            continue
        abre[m.end() - 1] = ("rango", m.group(1), lo, hi + (1 if m.group(3) else 0))
    for m in RE_FOR_ARR.finditer(cuerpo):
        arr = arrays.get(m.group(3))
        if arr:
            abre[m.end() - 1] = ("arr", m.group(1), m.group(2), arr)
    pila, tramos = [], []
    for i, ch in enumerate(cuerpo):
        if ch == "{":
            pila.append((i, abre.get(i)))
        elif ch == "}" and pila:
            ini, info = pila.pop()
            if info:
                tramos.append((ini, i, info))
    return tramos


def combos(tramos, pos, val):
    activos = [info for ini, fin, info in tramos if ini < pos < fin]
    out = [{}]
    for sc in activos:
        nu = []
        if sc[0] == "carril":
            for d in out:
                nu.append(dict(d, lane=0, offset=0))
                nu.append(dict(d, lane=1, offset=val["LANE_B"]))
        elif sc[0] == "rango":
            _, var, lo, hi = sc
            for d in out:
                for x in range(lo, hi):
                    nu.append(dict(d, **{var: x}))
        else:
            _, kv, cv, arr = sc
            for d in out:
                for k, nombre in enumerate(arr):
                    nu.append(dict(d, **{kv: k, cv: val.get(nombre)}))
        out = nu
    return out


def tramo_carril(tramos, pos):
    for ini, fin, info in tramos:
        if info[0] == "carril" and ini < pos < fin:
            return (ini, fin)
    return None


def censo_transicion(cuerpo, val, arrays, sel_vars, alias_next):
    tramos = ambitos(cuerpo, val, arrays)
    reclamos, n_result = set(), 0
    for m in RE_RESULT.finditer(cuerpo):
        n_result += 1
        fin = cuerpo.find(";", m.end())
        sent = cuerpo[m.start():fin] if fin > 0 else cuerpo[m.start():m.start() + 300]
        sels = set()
        for nom, canon in sel_vars.items():
            if re.search(r"\b%s\b" % nom, sent):
                sels.add(canon)
        seg_m = RE_SEGSEL.search(sent)
        exprs = [x.group(1) for x in RE_NEXT.finditer(sent)]
        for nom, ex in alias_next.items():
            if re.search(r"\b%s\b" % nom, sent):
                exprs.append(ex)
        carril = tramo_carril(tramos, m.start())
        ancho = False
        if not exprs and sels and carril:
            exprs = [x.group(1) for x in RE_NEXT.finditer(cuerpo[carril[0]:carril[1]])]
            ancho = True
        for d in combos(tramos, m.start(), val):
            s2 = set(sels)
            if seg_m:
                sv = d.get(seg_m.group(1), ev(seg_m.group(1), val))
                if sv is None:
                    continue
                s2.add("seg_link%d" % sv)
            dd_lista = [dict(d, j=jj) for jj in range(val["STATE_WIDTH"])] if ancho else [d]
            for dd in dd_lista:
                for ex in exprs:
                    c = ev(ex, val, dd)
                    if c is not None and 0 <= c < val["TRACE_WIDTH"]:
                        reclamos.add((frozenset(s2), c))
    return reclamos, n_result


def censo_aserciones(cuerpo, val, fila2clase):
    tramos = ambitos(cuerpo, val, {})
    fuera = set()
    for m in RE_ASSERT.finditer(cuerpo):
        for d in combos(tramos, m.start(), val):
            fila = ev(m.group(2), val, d)
            col = ev(m.group(1), val, d)
            if fila is None or col is None:
                continue
            cl = fila2clase.get(fila)
            if cl is not None and 0 <= col < val["TRACE_WIDTH"]:
                fuera.add((col, cl))
    return fuera


def celdas_libres(texto, clases, fila2clase):
    libres = set()
    for lin in RE_LIBRE_LINEA.finditer(texto):
      for m in RE_LOCUS.finditer(lin.group(1)):
        tipo, quien = m.group(1), m.group(2).strip()
        a = int(m.group(3))
        cols = range(a, int(m.group(4))) if m.group(4) else [a]
        if tipo == "fila":
            cl = fila2clase.get(int(quien)) if quien.isdigit() else None
            noms = [cl] if cl else []
        elif quien == "*":
            noms = list(clases)
        elif quien.startswith("sin "):
            sel = quien[4:].strip()
            noms = [n for n, (f, _) in clases.items() if sel not in f]
        else:
            noms = [quien] if quien in clases else []
        for n in noms:
            for c in cols:
                libres.add((c, n))
    return libres


def censar(texto):
    for a, n in ANCLAS:
        c = texto.count(a)
        assert c == n, "ancla <<%s>>: %d != %d" % (a, c, n)
    val = resolver(texto)
    # verbatim del traspaso, medido de nuevo aquí — no recordado
    esperado = {"TRACE_WIDTH": 39, "TRACE_LENGTH": 512, "SEGMENT_LENGTH": 64,
                "NUM_SEGMENTS": 3, "LANE_B": 12, "CYC_ACC": 3, "ROW_LEAF_LINK": 7,
                "ROW_SALT_LINK": 15, "ROW_LEAF_DONE": 23, "ROW_ACCT_ROOT": 279}
    for k, e in esperado.items():
        assert val.get(k) == e, "constante %s: resuelta %s, esperada %s" % (k, val.get(k), e)
    masc = mascaras(val)
    assert len(masc["hash_flag"]) == 245 and len(masc["acct_link"]) == 32
    clases, fila2clase = clasificar(masc, val)
    tex_tr = cuerpo_fn(texto, "fn evaluate_transition")
    tex_as = cuerpo_fn(texto, "fn get_assertions")
    arrays = {m.group(1): [t.strip() for t in m.group(2).split(",") if t.strip()]
              for m in RE_ARRAY.finditer(tex_tr)}
    sel_vars = {}
    for m in RE_SEL_BIND.finditer(tex_tr):
        if m.group(2) in P2SEL:
            sel_vars[m.group(1)] = P2SEL[m.group(2)]
    for m in RE_ALIAS_SEL.finditer(tex_tr):
        if m.group(2) in sel_vars:
            sel_vars[m.group(1)] = sel_vars[m.group(2)]
    alias_next = {m.group(1): m.group(2) for m in RE_ALIAS_NEXT.finditer(tex_tr)}
    reclamos, n_res = censo_transicion(tex_tr, val, arrays, sel_vars, alias_next)
    asev = censo_aserciones(tex_as, val, fila2clase)
    libres = celdas_libres(texto, clases, fila2clase)
    cubierto = set(asev)
    for nom, (firma, _) in clases.items():
        for sels, c in reclamos:
            if sels <= firma:
                cubierto.add((c, nom))
    sin, rancias = [], []
    for nom in clases:
        for c in range(val["TRACE_WIDTH"]):
            cu, li = (c, nom) in cubierto, (c, nom) in libres
            if cu and li:
                rancias.append((c, nom))
            elif not cu and not li:
                sin.append((c, nom))
    return val, clases, sorted(sin), sorted(rancias), libres, n_res


def main():
    texto = open(os.path.join(RAIZ, FICHERO), encoding="utf-8").read()
    val, clases, sin, rancias, libres, n_res = censar(texto)
    n_cl, W = len(clases), val["TRACE_WIDTH"]
    print("== intérprete multi-ciclo: circuit_credit_climb · %d columnas · %d pasos ==" %
          (W, val["TRACE_LENGTH"] - 1))
    for nom, (_, filas) in sorted(clases.items(), key=lambda kv: kv[1][1][0]):
        print("   clase %-28s · %3d filas · %d..%d" % (nom, len(filas), filas[0], filas[-1]))
    print("   %d clases · %d celdas-clase · %d libres declaradas · %d rancias · %d sentencias result"
          % (n_cl, n_cl * W, len(libres), len(rancias), n_res))
    print("   celdas SIN DUEÑO (sano): %d %s" % (len(sin), "✅" if not sin else ""))
    if sin:
        for c, nom in sin:
            print("      col %2d · %s" % (c, nom))
    if rancias:
        print("   ⚠ declaraciones rancias: %s" % rancias)
    resultado = 0 if (not sin and not rancias) else 1
    cazados = 0

    def censo_de(t):
        return censar(t)[2]

    def mutar(patron):
        nuevo, k = re.subn(patron, "", texto)
        return (nuevo if k == 1 else None), k

    for nombre, patron, descripcion in [
        ("C_HORNER", r"\n\s*result\[C_HORNER\][^;]*;",
         "el paso de Horner: el acumulador en el cuerpo de segmento"),
        ("C_SALT_IN_A", r"\n\s*result\[C_SALT_IN_A \+ i\][^;]*;",
         "la atadura del salt testigo al rate, carril A (§117: el cerrojo #2)"),
    ]:
        mut, k = mutar(patron)
        print("\n== MUTANTE (%s borrado: %s) ==" % (nombre, descripcion))
        if mut is None:
            print("   ❌ el patrón casó %d veces (≠1)" % k)
            resultado = 1
            continue
        sin_m = censo_de(mut)
        nuevas = sorted(set(sin_m) - set(sin))
        print("   celdas sin dueño: %d" % len(sin_m))
        if nuevas:
            print("   ✅ CAZA EL MUTANTE — nuevas huérfanas: %s" % nuevas)
            cazados += 1
        else:
            print("   ❌ no lo caza")
            resultado = 1

    # El candidato del mapa §9: borrar una C_SEG_LINK. Se MIDE qué red lo para.
    mut, k = mutar(r"\n\s*result\[C_SEG_LINK \+ seg\][^;]*;")
    print("\n== MUTANTE (C_SEG_LINK borrado: el candidato del mapa §9) ==")
    a_censo = False
    if mut is None:
        print("   ❌ el patrón casó %d veces (≠1)" % k)
        resultado = 1
    else:
        nuevas = sorted(set(censo_de(mut)) - set(sin))
        lee_sano = texto.count("periodic[P_SEG_LINK")
        lee_mut = mut.count("periodic[P_SEG_LINK")
        print("   delta del censo: %d nuevas %s" % (len(nuevas), nuevas if nuevas else ""))
        print("   lecturas de P_SEG_LINK: sano %d · mutado %d · construidas NUM_SEGMENTS=%d"
              % (lee_sano, lee_mut, val["NUM_SEGMENTS"]))
        if nuevas:
            print("   ✅ el censo lo caza")
            a_censo = True
        elif lee_sano >= 1 and lee_mut == 0:
            print("   ✅ al censo se le escapa (Horner co-posee la col 34 en las clases de cierre)")
            print("      — lo ven las DOS redes del guardián (ranuras MUERTAS y periódicas")
            print("        sin leerse), GRAVES desde §189: el exit del guardián cae solo")
        else:
            print("   ❌ ni censo ni lectura de periódicas: SIN RED")
            resultado = 1

    print("\nFV-1 caza la celda que NADIE mira; referenciada ≠ determinada (doc §1).")
    print("COMPUERTA-CREDIT: %d clases · %d celdas-clase · %d libres declaradas · %d sin dueño · "
          "mutantes %d/2 · C_SEG_LINK→%s"
          % (n_cl, n_cl * W, len(libres), len(sin), cazados,
             "censo" if a_censo else "guardián"))
    return resultado


sys.exit(main())
