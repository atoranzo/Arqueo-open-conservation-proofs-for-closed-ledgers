#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""NÚCLEO del censo FV-1 multi-ciclo — la lógica compartida de la estirpe (§193).

Nace de la entrada 73 con sus datos delante: tres hermanos derivados por
cortes (mint, credit, recovery) y un cuarto sujeto (circuit_mint, PAGOS)
cuyo drift ya no es de forma sino de LÓGICA — familias de máscara nuevas
(cust_link, sel_acct_root, sel_cust_root), otro cierre de tubería, otro
first_row. El rito veta derivar sobre lógica compartida; el núcleo la
comparte de verdad: una sola copia, y cada circuito trae su ESPEC.

ESPEC (dict) — lo que cada intérprete fino declara:
  FICHERO     nombre del .rs bajo crates/stark-experiment/src/
  EXTERNAS    constantes que el .rs importa (STATE_WIDTH, ...)
  ANCLAS      [(texto_verbatim, conteo_exacto), ...] — si el circuito
              cambia de forma, esto grita antes de modelar máscaras rancias
  ESPERADO    constantes resueltas que DEBEN salir (verbatim del traspaso)
  MASCARAS    callable(val) -> dict nombre->frozenset de filas
  SANIDAD     callable(val, masc) -> asserts de conteo sobre las máscaras
  P2SEL       P_* -> nombre canónico de selector (solo los que son máscara)
  TITULO      nombre del circuito para la cabecera
  ROTULO      prefijo de la línea COMPUERTA (p.ej. "COMPUERTA-CREDIT")
  MUTANTES    [(nombre, patron_regex, descripcion), ...] — los 2 internos
  COL_CIERRE  la columna que Horner co-posee en las clases de cierre
              (el mensaje del candidato la nombra)

La doctrina, la cosecha conservadora (§186) y el sesgo hacia el rojo
(§59.2) viven aquí, una vez. La paridad se probó byte a byte contra los
tres stdouts patrón-oro en el ensayo de §193.
"""
import os, re, sys

RAIZ = os.path.join(os.path.dirname(__file__), "..", "..", "crates", "stark-experiment", "src")

RE_CONST = re.compile(r"^(?:pub )?const ([A-Z][A-Z_0-9]*): usize = (.+?);", re.MULTILINE)
RE_RESULT = re.compile(r"result\[([^\]]+)\]\s*=")
RE_NEXT = re.compile(r"next\[([^\]]+)\]")
RE_SEGSEL = re.compile(r"periodic\[\s*P_SEG_LINK\s*\+\s*(\w+)\s*\]")
RE_SEL_BIND = re.compile(r"let\s+(\w+)\s*=\s*periodic\[(P_\w+)\]\s*;")
RE_ALIAS_NEXT = re.compile(r"let\s+(\w+)\s*=\s*next\[([^\]]+)\]\s*;")
RE_ALIAS_SEL = re.compile(r"let\s+(\w+)\s*=\s*(\w+)\s*;")
RE_ALIAS_SUMA = re.compile(r"let\s+(\w+)\s*=\s*(\w+)\s*\+\s*(\w+)\s*;")
RE_SUMA_INLINE = re.compile(r"\(\s*(\w+)\s*\+\s*(\w+)\s*\)")
RE_ARRAY = re.compile(r"let\s+(\w+)\s*=\s*\[\s*([A-Z_0-9\s,+]+)\]\s*;", re.DOTALL)
RE_ASSERT = re.compile(r"Assertion::single\(\s*([^,]+?)\s*,\s*([^,]+?)\s*,")
RE_FOR_LANE = re.compile(r"for\s*\(\s*lane\s*,\s*offset\s*\)\s*in\s*\[[^\]]*\]\s*\{")
RE_FOR_RANGE = re.compile(r"for\s+(\w+)\s+in\s+([A-Za-z_0-9]+)\s*\.\.(=?)\s*([A-Za-z_0-9]+)\s*\{")
RE_FOR_ARR = re.compile(r"for\s*\(\s*(\w+)\s*,\s*(\w+)\s*\)\s*in\s+(\w+)\s*\.iter\(\)\.enumerate\(\)\s*\{")
RE_LIBRE_LINEA = re.compile(r"//\s*CELDAS_LIBRES:(.*)")
RE_LOCUS = re.compile(
    r"\(\s*(clase|fila)\s+([^,)]+?)\s*,\s*cols?\s+([0-9]+)\s*(?:\.\.\s*([0-9]+))?\s*\)")

# Las cuatro familias clásicas del multi-ciclo, compartidas por la estirpe
# original (mint/credit/recovery_climb). Los PAGOS traen las suyas.
P2SEL_CLASICO = {"P_HASH_FLAG": "hash_flag", "P_ACCT_LINK": "acct_link",
                 "P_LINK_LEAF": "link_leaf", "P_LINK_SALT": "link_salt",
                 "P_FIRST_ROW": "first_row", "P_FIRST_S": "first_s",
                 "P_CONT_S": "cont_s"}


def mascaras_clasicas(v):
    """Las máscaras de la estirpe *_climb: hash multi-ciclo hasta la raíz
    de cuentas, one-hots de árbol y enlaces, fila 0, segmentos Horner."""
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


def sanidad_clasica(v, masc):
    assert len(masc["hash_flag"]) == 245 and len(masc["acct_link"]) == 32


def resolver(texto, externas):
    crudos = {m.group(1): m.group(2).strip() for m in RE_CONST.finditer(texto)}
    val = dict(externas)
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
                    nu.append(dict(d, **{kv: k, cv: ev(nombre, val)}))
        out = nu
    return out


def tramo_carril(tramos, pos):
    for ini, fin, info in tramos:
        if info[0] == "carril" and ini < pos < fin:
            return (ini, fin)
    return None


def censo_transicion(cuerpo, val, arrays, sel_vars, alias_next, suma_alias=None):
    suma_alias = suma_alias or {}
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
        alternos = [frozenset()]
        for nom, canones in suma_alias.items():
            if re.search(r"\b%s\b" % nom, sent):
                alternos = [al | {c} for al in alternos for c in canones]
        for msum in RE_SUMA_INLINE.finditer(sent):
            a, b = sel_vars.get(msum.group(1)), sel_vars.get(msum.group(2))
            if a and b:
                alternos = [al | {c} for al in alternos for c in (a, b)]
                sels.discard(a); sels.discard(b)
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
          for alt in alternos:
            s2 = set(sels) | set(alt)
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
            sels = [s.strip() for s in quien[4:].split(" ni ")]
            noms = [n for n, (f, _) in clases.items()
                    if all(s not in f for s in sels)]
        else:
            noms = [quien] if quien in clases else []
        for n in noms:
            for c in cols:
                libres.add((c, n))
    return libres


def censar(texto, E):
    for a, n in E["ANCLAS"]:
        c = texto.count(a)
        assert c == n, "ancla <<%s>>: %d != %d" % (a, c, n)
    val = resolver(texto, E["EXTERNAS"])
    # verbatim del traspaso, medido de nuevo aquí — no recordado
    for k, e in E["ESPERADO"].items():
        assert val.get(k) == e, "constante %s: resuelta %s, esperada %s" % (k, val.get(k), e)
    masc = E["MASCARAS"](val)
    E["SANIDAD"](val, masc)
    clases, fila2clase = clasificar(masc, val)
    tex_tr = cuerpo_fn(texto, "fn evaluate_transition")
    tex_as = cuerpo_fn(texto, "fn get_assertions")
    arrays = {m.group(1): [t.strip() for t in m.group(2).split(",") if t.strip()]
              for m in RE_ARRAY.finditer(tex_tr)}
    sel_vars = {}
    for m in RE_SEL_BIND.finditer(tex_tr):
        if m.group(2) in E["P2SEL"]:
            sel_vars[m.group(1)] = E["P2SEL"][m.group(2)]
    for m in RE_ALIAS_SEL.finditer(tex_tr):
        if m.group(2) in sel_vars:
            sel_vars[m.group(1)] = sel_vars[m.group(2)]
    alias_next = {m.group(1): m.group(2) for m in RE_ALIAS_NEXT.finditer(tex_tr)}
    suma_alias = {}
    for m in RE_ALIAS_SUMA.finditer(tex_tr):
        if m.group(2) in sel_vars and m.group(3) in sel_vars:
            suma_alias[m.group(1)] = [sel_vars[m.group(2)], sel_vars[m.group(3)]]
    reclamos, n_res = censo_transicion(tex_tr, val, arrays, sel_vars, alias_next, suma_alias)
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


def correr(E):
    texto = open(os.path.join(RAIZ, E["FICHERO"]), encoding="utf-8").read()
    val, clases, sin, rancias, libres, n_res = censar(texto, E)
    n_cl, W = len(clases), val["TRACE_WIDTH"]
    print("== intérprete multi-ciclo: %s · %d columnas · %d pasos ==" %
          (E["TITULO"], W, val["TRACE_LENGTH"] - 1))
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
        return censar(t, E)[2]

    def mutar(patron):
        nuevo, k = re.subn(patron, "", texto)
        return (nuevo if k == 1 else None), k

    for nombre, patron, descripcion in E["MUTANTES"]:
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
            print("   ✅ al censo se le escapa (Horner co-posee la col %d en las clases de cierre)"
                  % E["COL_CIERRE"])
            print("      — lo ven las DOS redes del guardián (ranuras MUERTAS y periódicas")
            print("        sin leerse), GRAVES desde §189: el exit del guardián cae solo")
        else:
            print("   ❌ ni censo ni lectura de periódicas: SIN RED")
            resultado = 1

    print("\nFV-1 caza la celda que NADIE mira; referenciada ≠ determinada (doc §1).")
    print("%s: %d clases · %d celdas-clase · %d libres declaradas · %d sin dueño · "
          "mutantes %d/2 · C_SEG_LINK→%s"
          % (E["ROTULO"], n_cl, n_cl * W, len(libres), len(sin), cazados,
             "censo" if a_censo else "guardián"))
    return resultado
