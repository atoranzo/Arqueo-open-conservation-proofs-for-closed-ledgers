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
  MUTANTES    [(nombre, patron_regex, descripcion[, reemplazo]), ...] — los
              internos (2 en stark-experiment). Sin reemplazo, el mutante
              BORRA lo que casa; con él, lo cambia. La COMPUERTA dice
              «mutantes cazados/total»
  COL_CIERRE  la columna que Horner co-posee en las clases de cierre
              (el mensaje del candidato la nombra)

Las claves OPCIONALES del kit (crates/zk-ssl-air, que no escribe como
stark-experiment). Sin ellas, el núcleo hace exactamente lo de siempre:
la paridad de los seis censos del guardián se midió byte a byte.
  RAIZ        el directorio del .rs (por defecto, stark-experiment/src)
  SIGUIENTE   el nombre de la fila siguiente (`siguiente`, `s`)
  PERIODICA   el nombre de las periódicas (`periodic_values` en la edad)
  ALIAS_CONST {nombre_del_núcleo: nombre_del_circuito}: TRACE_WIDTH <- ANCHO,
              TRACE_LENGTH <- TRAZA, STATE_WIDTH <- ESTADO, ...
  KIT         True: las formas del kit. Selectores EN LÍNEA
              (`periodic[P_HASH_A]` dentro de la sentencia), bucles sobre
              listas y tuplas (`for (c_cap, c_place, base) in [(..), (..)]`),
              tablas `const TRANSPORTE: [usize; N]` y las llamadas a
              `ronda_rescue`, cuyo segundo argumento es la fila siguiente
              del carril y el quinto su selector. ⚠️ Un selector en línea
              que no está en P2SEL ni en NO_SELECTORES es error: tomarlo
              por «sin selector» reclamaría la celda en TODAS las clases,
              y eso aprueba en silencio (§59.2).
  NO_SELECTORES  las P_* que no son máscara (las constantes de ronda)
  CANDIDATO   el patrón del mutante C_SEG_LINK, o None si el circuito no
              tiene segmentos de rango (la prenda, la edad)
  ACTUAL      el nombre de la fila actual; con él, una referencia a la fila
              actual en la transición r+1, BAJO SELECTOR, también es dueña de
              la celda de la fila r+1 si el selector está activo en todas las
              sucesoras de la clase (la edad: un AIR CÍCLICO, que ata la fila
              0 de cada ciclo por su `current`). Sin selector no: la
              booleanidad referencia, no determina
  RECLAMOS_EXTRA  [(selector, "actual"|"siguiente", columnas, porqué)]: los
              dueños que el intérprete no lee del texto (el tramo auxiliar de
              la edad), declarados y anclados por ANCLAS

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
# Las formas del kit (solo con ESPEC["KIT"]).
RE_FOR_LISTA = re.compile(
    r"for\s+(\(\s*\w+(?:\s*,\s*\w+)*\s*\)|\w+)\s+in\s+\[([^\]]*)\]\s*\{")
RE_CONST_ARRAY = re.compile(
    r"const\s+([A-Z][A-Z_0-9]*)\s*:\s*\[[^\]]*\]\s*=\s*\[\s*([A-Z_0-9\s,+]+)\]\s*;")
RE_LET_PAR = re.compile(
    r"let\s*\(\s*(\w+)\s*,\s*(\w+)\s*\)\s*=\s*\(\s*&?([^,]+?)\s*,\s*&?([^;]+?)\s*\)\s*;")
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


# Las familias de los GEMELOS de 1024 (send/claim, §195-§196): tres tramos
# de arbol, triple one-hot, fase del pendiente y segmentos que NO llenan
# la traza. Una sola copia; cada gemela trae su ESPEC (NS y sanidad).
P2SEL_GEMELAS = {"P_HASH_FLAG": "hash_flag", "P_LINK_MERKLE": "link_merkle",
                 "P_LINK_LEAF": "link_leaf", "P_LINK_SALT": "link_salt",
                 "P_LINK_PLACE": "link_place", "P_FIRST_ROW": "first_row",
                 "P_SEL_ROOT": "sel_root", "P_SEL_PK_DONE": "sel_pk_done",
                 "P_FIRST_S": "first_s", "P_CONT_S": "cont_s",
                 "P_FROZEN_ENTRY": "frozen_entry",
                 "P_FROZEN_LINK": "frozen_link",
                 "P_PEND_IN": "pend_in", "P_PEND_VAL": "pend_val",
                 "P_PEND_ENTRY": "pend_entry", "P_PEND_LINK": "pend_link"}


def mascaras_gemelas(v):
    pasos = range(v["TRACE_LENGTH"] - 1)
    C = v["CYCLE_LENGTH"]
    m = {}
    m["hash_flag"] = frozenset(
        r for r in pasos
        if r <= v["ROW_PENDING_ROOT"] and r % C < v["NUM_ROUNDS"])
    m["link_merkle"] = frozenset(
        (v["CYC_ACC"] + lv) * C + 7 for lv in range(v["TREE_DEPTH"] - 1))
    m["link_leaf"] = frozenset({v["ROW_LEAF_LINK"]})
    m["link_salt"] = frozenset({v["ROW_SALT_LINK"]})
    m["link_place"] = frozenset({v["ROW_LEAF_DONE"]})
    m["first_row"] = frozenset({0})
    m["sel_root"] = frozenset({v["ROW_ROOT"]})
    m["sel_pk_done"] = frozenset({v["ROW_PK_DONE"]})
    m["first_s"] = frozenset(s * v["SEGMENT_LENGTH"] for s in range(v["NUM_SEGMENTS"]))
    m["cont_s"] = frozenset(
        s * v["SEGMENT_LENGTH"] + p
        for s in range(v["NUM_SEGMENTS"]) for p in range(v["SEGMENT_LENGTH"] - 1))
    for s in range(v["NUM_SEGMENTS"]):
        m["seg_link%d" % s] = frozenset({(s + 1) * v["SEGMENT_LENGTH"] - 2})
    m["frozen_entry"] = frozenset({v["ROW_PK_DONE"]})
    m["frozen_link"] = frozenset(
        (v["CYC_FROZEN"] + lv) * C + 7 for lv in range(v["FROZEN_DEPTH"] - 1))
    m["pend_in"] = frozenset({v["ROW_FROZEN_ROOT"]})
    m["pend_val"] = frozenset({v["ROW_PEND_INNER"]})
    m["pend_entry"] = frozenset({v["ROW_PENDING_ENTRY"]})
    m["pend_link"] = frozenset(
        (v["CYC_PEND_CLIMB"] + lv) * C + 7 for lv in range(v["TREE_DEPTH"] - 1))
    return m


def regex_de(E):
    """Las expresiones del sujeto. Sin SIGUIENTE ni PERIODICA, las de
    siempre, los MISMOS objetos: la paridad de stark-experiment no cambia."""
    sig, per = E.get("SIGUIENTE"), E.get("PERIODICA")
    if sig is None and per is None:
        return {"NEXT": RE_NEXT, "ALIAS_NEXT": RE_ALIAS_NEXT,
                "SEL_BIND": RE_SEL_BIND, "SEGSEL": RE_SEGSEL, "PER": None,
                "ACTUAL": None, "SIG": "next", "ACT": None}
    sig, per = sig or "next", per or "periodic"
    act = E.get("ACTUAL")
    return {
        "NEXT": re.compile(r"\b%s\[([^\]]+)\]" % sig),
        "ALIAS_NEXT": re.compile(r"let\s+(\w+)\s*=\s*%s\[([^\]]+)\]\s*;" % sig),
        "SEL_BIND": re.compile(r"let\s+(\w+)\s*=\s*%s\[(P_\w+)\]\s*;" % per),
        "SEGSEL": re.compile(r"%s\[\s*P_SEG_LINK\s*\+\s*(\w+)\s*\]" % per),
        "PER": re.compile(r"\b%s\[\s*(P_\w+)\s*\]" % per),
        "ACTUAL": re.compile(r"\b%s\[([^\]]+)\]" % act) if act else None,
        "SIG": sig,
        "ACT": act,
    }


# ===== El KIT (crates/zk-ssl-air) =====
#
# Las cuatro AIR de un solo tramo del kit (banda, cobro pendiente, pago en
# curso, prenda) comparten molde: traza de 512, subida de PROFUNDIDAD - 1
# enlaces desde CYC_ACC, raíz en ROW_RAIZ, selectores de una fila hechos con
# `for fila in [..., 0]` y, salvo la prenda, segmentos de rango. Una sola
# copia de sus máscaras; cada intérprete dice qué filas lleva.
RAIZ_KIT = os.path.join(os.path.dirname(__file__), "..", "..", "crates", "zk-ssl-air", "src")
EXTERNAS_KIT = {"CICLO": 8, "ESTADO": 12, "RONDAS": 7, "PROFUNDIDAD": 32}
ALIAS_KIT = {"TRACE_WIDTH": "ANCHO", "TRACE_LENGTH": "TRAZA", "STATE_WIDTH": "ESTADO",
             "CYCLE_LENGTH": "CICLO", "NUM_ROUNDS": "RONDAS"}
NO_SELECTORES_KIT = ("P_ARK1", "P_ARK2")


def mascaras_kit(v, banderas, unas, segmentos):
    """Máscaras del molde del kit.

    `banderas`: {nombre: filas de hash} (cada carril trae las suyas);
    `unas`: {nombre: fila} de los selectores de una fila; `segmentos`: si el
    AIR lleva segmentos de rango (first_s, cont_s, seg_link%d)."""
    m = {}
    for nombre, filas in banderas.items():
        m[nombre] = frozenset(filas)
    m["link_merkle"] = frozenset(
        (v["CYC_ACC"] + lv) * v["CICLO"] + v["CICLO"] - 1 for lv in range(v["PROFUNDIDAD"] - 1))
    for nombre, fila in unas.items():
        m[nombre] = frozenset({fila})
    if segmentos:
        L = v["LARGO_SEGMENTO"]
        m["first_s"] = frozenset(s * L for s in range(v["SEGMENTOS"]))
        m["cont_s"] = frozenset(s * L + p for s in range(v["SEGMENTOS"]) for p in range(L - 1))
        for s in range(v["SEGMENTOS"]):
            m["seg_link%d" % s] = frozenset({(s + 1) * L - 2})
    return m


def filas_hash(v, desde, hasta):
    """Las filas de ronda (`r % CICLO < RONDAS`) entre `desde` y `hasta`, ambas incluidas."""
    return [r for r in range(desde, hasta + 1) if r % v["CICLO"] < v["RONDAS"]]


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


def ambitos(cuerpo, val, arrays, kit=False):
    abre = {}
    if kit:
        for m in RE_FOR_LISTA.finditer(cuerpo):
            pat = m.group(1)
            nombres = ([x.strip() for x in pat[1:-1].split(",")]
                       if pat.startswith("(") else [pat])
            filas = []
            for e in _partir(m.group(2)):
                if e.startswith("(") and e.endswith(")"):
                    filas.append(_partir(e[1:-1]))
                else:
                    filas.append([e])
            abre[m.end() - 1] = ("lista", nombres, filas)
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
        if sc[0] == "lista":
            _, nombres, filas = sc
            for d in out:
                for fila in filas:
                    nu.append(dict(d, **{n: ev(e, val) for n, e in zip(nombres, fila)}))
        elif sc[0] == "carril":
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


def _partir(cuerpo):
    """Trozos de nivel cero de una lista, sin los vacíos."""
    prof, trozos, actual = 0, [], []
    for ch in cuerpo:
        if ch in "([":
            prof += 1
        elif ch in ")]":
            prof -= 1
        if ch == "," and prof == 0:
            trozos.append("".join(actual))
            actual = []
        else:
            actual.append(ch)
    trozos.append("".join(actual))
    return [t.strip() for t in trozos if t.strip()]


def tramo_carril(tramos, pos):
    for ini, fin, info in tramos:
        if info[0] == "carril" and ini < pos < fin:
            return (ini, fin)
    return None


def censo_transicion(cuerpo, val, arrays, sel_vars, alias_next, suma_alias=None,
                     rx=None, kit=None):
    """`kit` (opcional): {"P2SEL": ..., "NO_SELECTORES": ...} para los
    selectores en línea del kit. Sin él, lo de siempre."""
    suma_alias = suma_alias or {}
    rx = rx or regex_de({})
    tramos = ambitos(cuerpo, val, arrays, kit=kit is not None)
    reclamos, n_result = set(), 0
    for m in RE_RESULT.finditer(cuerpo):
        n_result += 1
        fin = cuerpo.find(";", m.end()) if kit is None else fin_de_sentencia(cuerpo, m.end())
        entera = cuerpo[m.start():fin] if fin > 0 else cuerpo[m.start():m.start() + 300]
        # En el kit, cada SUMANDO del lado derecho con sus selectores: la edad
        # escribe `s[C_CICLO] - c[C_CICLO] - enlace`, donde `enlace` no
        # selecciona la restricción sino que se resta, y la celda siguiente
        # queda atada en TODAS las filas. Con la sentencia entera, el censo la
        # tomaba por seleccionada y pintaba huérfanas que no lo son. Un
        # sumando sin selector no hereda el de otro; uno con selectores
        # dentro de paréntesis o llaves los conserva (sesgo al rojo, §59.2).
        partes = [entera] if kit is None else sumandos(entera)
        for sent in partes:
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
            if kit is not None:
                sels |= selectores_en_linea(sent, rx, kit)
            seg_m = rx["SEGSEL"].search(sent)
            exprs = [x.group(1) for x in rx["NEXT"].finditer(sent)]
            for nom, ex in alias_next.items():
                if re.search(r"\b%s\b" % nom, sent):
                    exprs.append(ex)
            carril = tramo_carril(tramos, m.start())
            ancho = False
            if not exprs and sels and carril:
                exprs = [x.group(1) for x in rx["NEXT"].finditer(cuerpo[carril[0]:carril[1]])]
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


def sumandos(sent):
    """Los sumandos de nivel cero del lado derecho de `result[..] = ...`,
    cada uno precedido de la cabecera `result[..] =` (solo el kit)."""
    m = re.match(r"\s*result\[[^\]]+\]\s*=", sent)
    if m is None:
        return [sent]
    cab, rhs = sent[:m.end()], sent[m.end():]
    prof, trozos, actual = 0, [], []
    for k, ch in enumerate(rhs):
        if ch in "([{":
            prof += 1
        elif ch in ")]}":
            prof -= 1
        if ch in "+-" and prof == 0 and "".join(actual).strip():
            trozos.append("".join(actual))
            actual = []
            continue
        actual.append(ch)
    trozos.append("".join(actual))
    return [cab + " " + t for t in trozos if t.strip()]


def fin_de_sentencia(cuerpo, ini):
    """El `;` de nivel cero desde `ini` (solo el kit). La edad escribe
    `result[i] = if todos { .. } else { let nombrado = ..; .. };`: cortar en el
    primer `;` dejaba fuera la rama que lee `C_WINV`."""
    prof = 0
    for k in range(ini, len(cuerpo)):
        ch = cuerpo[k]
        if ch in "([{":
            prof += 1
        elif ch in ")]}":
            prof -= 1
        elif ch == ";" and prof == 0:
            return k
    return -1


def selectores_en_linea(sent, rx, kit):
    """Los selectores que la sentencia lee EN LÍNEA (`periodic[P_HASH_A]`).

    ⚠️ Uno que no está en P2SEL ni en NO_SELECTORES es error, no «sin
    selector»: sin selector, la restricción reclamaría la celda en TODAS
    las clases y el censo aprobaría lo que no entiende (§59.2)."""
    sels = set()
    for p in rx["PER"].findall(sent):
        if p in kit["P2SEL"]:
            sels.add(kit["P2SEL"][p])
        elif p not in kit.get("NO_SELECTORES", ()):
            raise AssertionError("selector en línea sin máscara: %s en <<%s>>"
                                 % (p, " ".join(sent.split())[:90]))
    return sels


def reclamos_ronda(cuerpo, val, sel_vars, arrays, rx, kit, nombre="ronda_rescue"):
    """Los dueños que pone cada llamada a `ronda_rescue`: las ESTADO celdas
    de la fila siguiente que recibe como segundo argumento, bajo el selector
    del quinto (`bandera * (sbox(B) - A)` con `B` de `siguiente[0..ESTADO]`,
    `zk-ssl-air/src/lib.rs`).

    El segundo argumento puede ser la fila entera (`siguiente`), un corte
    (`&siguiente[C_B..C_B + ESTADO]`) o un nombre atado por
    `let (a, b) = (&c[..], &s[base..base + ESTADO]);` dentro de un bucle
    (la edad). Lo que no se entiende es error: una ronda sin dueños deja las
    celdas del carril SIN DUEÑO, y una con dueños de más, aprobadas."""
    tramos = ambitos(cuerpo, val, arrays, kit=True)
    pares = {}
    for m in RE_LET_PAR.finditer(cuerpo):
        pares[m.group(1)] = m.group(3)
        pares[m.group(2)] = m.group(4)
    sig = rx["SIG"]
    reclamos, n = set(), 0
    for m in re.finditer(r"\b%s\(" % nombre, cuerpo):
        if cuerpo[max(0, m.start() - 3):m.start()] == "fn ":
            continue
        prof, j = 0, m.end() - 1
        for k in range(m.end() - 1, len(cuerpo)):
            if cuerpo[k] == "(":
                prof += 1
            elif cuerpo[k] == ")":
                prof -= 1
                if prof == 0:
                    j = k
                    break
        args = _partir(cuerpo[m.end():j])
        assert len(args) == 6, "%s con %d argumentos" % (nombre, len(args))
        n += 1
        fila, sel = args[1].lstrip("&").strip(), args[4].strip()
        fila = pares.get(fila, fila).lstrip("&").strip()
        if sel in sel_vars:
            canon = sel_vars[sel]
        else:
            ps = rx["PER"].findall(sel)
            assert len(ps) == 1 and ps[0] in kit["P2SEL"], \
                "selector de ronda no reconocido: %s" % sel
            canon = kit["P2SEL"][ps[0]]
        for d in combos(tramos, m.start(), val):
            if fila == sig:
                cols = range(0, val["STATE_WIDTH"])
            else:
                mc = re.fullmatch(r"%s\[(.+?)\.\.(.+)\]" % re.escape(sig), fila)
                assert mc, "fila de ronda no reconocida: %s" % fila
                a, b = ev(mc.group(1), val, d), ev(mc.group(2), val, d)
                assert a is not None and b is not None and b - a == val["STATE_WIDTH"], \
                    "corte de ronda no evaluable: %s" % fila
                cols = range(a, b)
            for c in cols:
                reclamos.add((frozenset({canon}), c))
    return reclamos, n


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
    rx = regex_de(E)
    val = resolver(texto, E["EXTERNAS"])
    for nucleo_nombre, propio in E.get("ALIAS_CONST", {}).items():
        val[nucleo_nombre] = val[propio]
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
    kit = None
    if E.get("KIT"):
        kit = {"P2SEL": E["P2SEL"], "NO_SELECTORES": E.get("NO_SELECTORES", ())}
        for m in RE_CONST_ARRAY.finditer(texto):
            arrays.setdefault(m.group(1), [t.strip() for t in m.group(2).split(",") if t.strip()])
    sel_vars = {}
    for m in rx["SEL_BIND"].finditer(tex_tr):
        if m.group(2) in E["P2SEL"]:
            sel_vars[m.group(1)] = E["P2SEL"][m.group(2)]
    for m in RE_ALIAS_SEL.finditer(tex_tr):
        if m.group(2) in sel_vars:
            sel_vars[m.group(1)] = sel_vars[m.group(2)]
    alias_next = {m.group(1): m.group(2) for m in rx["ALIAS_NEXT"].finditer(tex_tr)}
    suma_alias = {}
    for m in RE_ALIAS_SUMA.finditer(tex_tr):
        if m.group(2) in sel_vars and m.group(3) in sel_vars:
            suma_alias[m.group(1)] = [sel_vars[m.group(2)], sel_vars[m.group(3)]]
    for nombre, canon in E.get("SEL_EXTRA", {}).items():
        sel_vars[nombre] = canon
    reclamos, n_res = censo_transicion(tex_tr, val, arrays, sel_vars, alias_next, suma_alias,
                                       rx=rx, kit=kit)
    if kit is not None:
        r_ronda, n_ronda = reclamos_ronda(tex_tr, val, sel_vars, arrays, rx, kit)
        assert n_ronda == E.get("RONDAS_ESPERADAS", n_ronda), \
            "llamadas a ronda_rescue: %d" % n_ronda
        reclamos |= r_ronda
    # Dueños por la fila ACTUAL (AIR cíclicos) y los declarados (tramo auxiliar).
    actuales = set()
    if rx["ACTUAL"] is not None:
        actuales |= reclamos_actual(tex_tr, val, arrays, sel_vars, rx, kit)
    for sel, fila, cols, _porque in E.get("RECLAMOS_EXTRA", ()):
        # La misma regla que lo cosechado: en la fila actual, solo bajo selector.
        assert sel or fila != "actual", "reclamo de la fila actual sin selector: %s" % _porque
        firma = frozenset([sel] if sel else [])
        for c in (cols(val) if callable(cols) else cols):
            (actuales if fila == "actual" else reclamos).add((firma, c))
    asev = censo_aserciones(tex_as, val, fila2clase)
    libres = celdas_libres(texto, clases, fila2clase)
    cubierto = set(asev)
    for nom, (firma, _) in clases.items():
        for sels, c in reclamos:
            if sels <= firma:
                cubierto.add((c, nom))
    if actuales:
        # La celda de la fila r+1 que la clase de r deja: la mira la transición
        # r+1 por su fila ACTUAL si su selector está activo en la clase de r+1.
        # ⚠️ En TODAS las sucesoras de la clase, no en una: la clase `hash` de
        # la edad (filas 1..5) tiene por sucesoras `hash` y `f6+hash`, y un
        # selector activo solo en f6 dejaba aprobada la celda de las filas
        # 2..5, que no mira. El `and` evita que una clase sin sucesoras
        # apruebe por vacío (sesgo al rojo, §59.2).
        sucesoras = {nom: set() for nom in clases}
        for nom, (_, filas) in clases.items():
            for r in filas:
                sig_nom = fila2clase.get(r + 1)
                if sig_nom is not None:
                    sucesoras[nom].add(sig_nom)
        for nom in clases:
            for sels, c in actuales:
                if sucesoras[nom] and all(sels <= clases[s2][0] for s2 in sucesoras[nom]):
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


def reclamos_actual(cuerpo, val, arrays, sel_vars, rx, kit):
    """Las referencias a la fila ACTUAL de cada sentencia `result[..] =`,
    con su selector: (firma, columna). Mismo cosechado que la siguiente.

    ⚠️ Solo los sumandos CON selector. Un sumando sin selector que lee la
    fila actual es, en un AIR cíclico, la booleanidad `c[x] * (c[x] - 1)`:
    referencia la celda, no la determina. Contarlo la hacía dueña de VIVO,
    QA, QS, FUERA y los bits de la edad en todas las clases, y eso tapaba la
    constancia por ciclo de VIVO (medido: cambiar `hf` por `enlace` en
    `hf * (s[C_VIVO] - vivo)` dejaba el censo en 0 sin dueño). La fila
    siguiente sin selector sigue siendo dueña, como en el §1: es la doctrina
    de siempre y no se toca."""
    tramos = ambitos(cuerpo, val, arrays, kit=kit is not None)
    # Los alias de la fila actual (`let vivo = c[C_VIVO];`, `let x = c[C_BITS + b];`).
    alias = {m.group(1): m.group(2) for m in re.finditer(
        r"let\s+(\w+)\s*=\s*%s\[([^\]]+)\]\s*;" % re.escape(rx["ACT"]),
        cuerpo)}
    out = set()
    for m in RE_RESULT.finditer(cuerpo):
        fin = fin_de_sentencia(cuerpo, m.end())
        entera = cuerpo[m.start():fin] if fin > 0 else cuerpo[m.start():m.start() + 300]
        for sent in sumandos(entera):
            sels = set()
            for nom, canon in sel_vars.items():
                if re.search(r"\b%s\b" % nom, sent):
                    sels.add(canon)
            if kit is not None:
                sels |= selectores_en_linea(sent, rx, kit)
            if not sels:
                continue
            exprs = [x.group(1) for x in rx["ACTUAL"].finditer(sent)]
            for nom, ex in alias.items():
                if re.search(r"\b%s\b" % nom, sent):
                    exprs.append(ex)
            for d in combos(tramos, m.start(), val):
                for ex in exprs:
                    c = ev(ex, val, d)
                    if c is not None and 0 <= c < val["TRACE_WIDTH"]:
                        out.add((frozenset(sels), c))
    return out


def correr(E):
    texto = open(os.path.join(E.get("RAIZ", RAIZ), E["FICHERO"]), encoding="utf-8").read()
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

    def mutar(patron, por=""):
        nuevo, k = re.subn(patron, por, texto)
        return (nuevo if k == 1 else None), k

    # Cada mutante borra lo que casa su patrón o, con un cuarto elemento, lo
    # cambia por él (`hf` por `enlace`: un selector equivocado, que el barrido
    # de ranuras no ve porque la ranura sigue escrita).
    for mutante in E["MUTANTES"]:
        nombre, patron, descripcion = mutante[:3]
        por = mutante[3] if len(mutante) > 3 else ""
        mut, k = mutar(patron, por)
        print("\n== MUTANTE (%s %s: %s) ==" % (nombre, "cambiado" if por else "borrado",
                                              descripcion))
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
    candidato = E.get("CANDIDATO", r"\n\s*result\[C_SEG_LINK \+ seg\][^;]*;")
    a_censo = False
    mut, k = mutar(candidato) if candidato is not None else (None, None)
    if candidato is None:
        print("\n== MUTANTE C_SEG_LINK: el circuito no tiene segmentos de rango ==")
    else:
        print("\n== MUTANTE (C_SEG_LINK borrado: el candidato del mapa §9) ==")
    if candidato is None:
        pass
    elif mut is None:
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
          "mutantes %d/%d · C_SEG_LINK→%s"
          % (E["ROTULO"], n_cl, n_cl * W, len(libres), len(sin), cazados, len(E["MUTANTES"]),
             "sin segmento" if candidato is None
             else "censo" if a_censo else "guardián"))
    return resultado
