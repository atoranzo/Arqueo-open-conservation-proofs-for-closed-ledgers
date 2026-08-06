#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Intérprete del ENVÍO (circuit_send) — octava pieza (§195).

La segunda nacida directamente como ESPEC sobre censo_nucleo.py, y la
primera FUERA de la traza 512: dos carriles reales sobre 1024×56. Lo que
send añade a la estirpe:
  · TRES tramos de árbol — cuentas (raíz A/B en 279), NO-pertenencia a
    congelados (raíz en 543, solo carril A) e inserción en pendientes
    (raíces A/B en 815); el hash corre hasta ROW_PENDING_ROOT y deja
    HOLGURA: 208 filas sin selector — aquí SÍ hay clase plana.
  · tree_link = link_merkle + link_place y pend_any = pend_entry +
    pend_link: alias-SUMA por `let` (v2). Y (frozen_entry + frozen_link)
    SUMA INLINE, sin `let` — la forma que motivó el núcleo v4 (un
    reclamo por sumando, sumandos fuera del conjunto estricto).
  · transport de 10 columnas con sumas de literal (COL_KEY + 1..3):
    la otra mitad de v4 (arrays con `+`, elementos evaluados).
  · sel_root {279} y sel_pk_done {287} del triple one-hot
    `for row in [0, ROW_ROOT, ROW_PK_DONE]` — first_row incluido;
    frozen_entry comparte fila con sel_pk_done: firma compuesta.
  · pend_in {543} · pend_val {551} · pend_entry {559}: la fase del
    compromiso, tres selectores de una fila.
  · 5 segmentos Horner en 0..319: NO llenan la traza (a diferencia de
    toda la estirpe anterior).
Sin POW2. Doctrina, cosecha conservadora (§186) y sesgo al rojo (§59.2):
en el núcleo. Referenciada ≠ determinada (doc §1).
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import censo_nucleo as nucleo


def mascaras_send(v):
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


def sanidad_send(v, masc):
    assert len(masc["hash_flag"]) == 714 and len(masc["link_merkle"]) == 31
    assert len(masc["frozen_link"]) == 31 and len(masc["pend_link"]) == 31
    assert len(masc["cont_s"]) == 315


P2SEL_SEND = {"P_HASH_FLAG": "hash_flag", "P_LINK_MERKLE": "link_merkle",
              "P_LINK_LEAF": "link_leaf", "P_LINK_SALT": "link_salt",
              "P_LINK_PLACE": "link_place", "P_FIRST_ROW": "first_row",
              "P_SEL_ROOT": "sel_root", "P_SEL_PK_DONE": "sel_pk_done",
              "P_FIRST_S": "first_s", "P_CONT_S": "cont_s",
              "P_FROZEN_ENTRY": "frozen_entry", "P_FROZEN_LINK": "frozen_link",
              "P_PEND_IN": "pend_in", "P_PEND_VAL": "pend_val",
              "P_PEND_ENTRY": "pend_entry", "P_PEND_LINK": "pend_link"}

ESPEC = {
    "FICHERO": "circuit_send.rs",
    "TITULO": "circuit_send",
    "ROTULO": "COMPUERTA-SEND",
    "COL_CIERRE": 40,
    "EXTERNAS": {"STATE_WIDTH": 12, "NUM_ROUNDS": 7, "CYCLE_LENGTH": 8,
                 "TREE_DEPTH": 32, "FROZEN_DEPTH": 32},
    "ANCLAS": [
        ("for r in 0..=ROW_PENDING_ROOT", 2),  # hash_flag + bucle de ARK
        ("if r % CYCLE_LENGTH < NUM_ROUNDS {", 1),
        ("hash_flag[r] = one;", 1),
        ("link_merkle[(CYC_ACC + level) * CYCLE_LENGTH + 7] = one;", 1),
        ("link_leaf[ROW_LEAF_LINK] = one;", 1),
        ("link_salt[ROW_SALT_LINK] = one;", 1),
        ("link_place[ROW_LEAF_DONE] = one;", 1),
        ("for row in [0, ROW_ROOT, ROW_PK_DONE] {", 1),  # triple one-hot
        ("first_s[seg * SEGMENT_LENGTH] = one;", 1),
        ("cont_s[seg * SEGMENT_LENGTH + p] = one;", 1),
        ("link[(seg + 1) * SEGMENT_LENGTH - 2] = one;", 1),
        ("frozen_entry[ROW_PK_DONE] = one;", 1),
        ("frozen_link[(CYC_FROZEN + level) * CYCLE_LENGTH + 7] = one;", 1),
        ("pend_link[(CYC_PEND_CLIMB + level) * CYCLE_LENGTH + 7] = one;", 1),
        ("for level in 0..TREE_DEPTH - 1 {", 2),  # merkle + pendientes
        ("for level in 0..FROZEN_DEPTH - 1 {", 1),
        ("let tree_link = link_merkle + link_place;", 1),  # alias-suma (v2)
        ("let pend_any = pend_entry + pend_link;", 1),     # alias-suma (v2)
        ("(frozen_entry + frozen_link)", 1),               # suma inline (v4)
        ("let transport = [", 1),                          # array con + (v4)
        ("AirContext::new(trace_info, degrees, 42, options)", 1),
    ],
    "ESPERADO": {"TRACE_WIDTH": 56, "TRACE_LENGTH": 1024, "SEGMENT_LENGTH": 64,
                 "NUM_SEGMENTS": 5, "LANE_B": 12, "NUM_CONSTRAINTS": 203,
                 "CYC_ACC": 3, "CYC_PK": 35, "CYC_FROZEN": 36,
                 "CYC_PEND_IN": 68, "CYC_PEND_CLIMB": 70, "CYC_FIN": 102,
                 "ROW_LEAF_LINK": 7, "ROW_SALT_LINK": 15, "ROW_LEAF_DONE": 23,
                 "ROW_ROOT": 279, "ROW_PK_START": 280, "ROW_PK_DONE": 287,
                 "ROW_FROZEN_ROOT": 543, "ROW_PEND_INNER": 551,
                 "ROW_PENDING_ENTRY": 559, "ROW_PENDING_ROOT": 815},
    "MASCARAS": mascaras_send,
    "SANIDAD": sanidad_send,
    "P2SEL": P2SEL_SEND,
    "MUTANTES": [
        ("C_HORNER", r"\n\s*result\[C_HORNER\][^;]*;",
         "el paso de Horner: el acumulador en el cuerpo de segmento"),
        ("C_SALT_IN_A", r"\n\s*result\[C_SALT_IN_A \+ i\][^;]*;",
         "la atadura del salt testigo al rate, carril A (§117/§138)"),
    ],
}

sys.exit(nucleo.correr(ESPEC))
