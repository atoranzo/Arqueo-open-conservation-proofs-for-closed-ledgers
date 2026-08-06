#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Intérprete de la EMISIÓN (circuit_mint, pagos) — séptima pieza (§194).

La primera nacida directamente como ESPEC sobre censo_nucleo.py: sin
derivación por cortes, con sus familias PROPIAS declaradas como delta
sobre las clásicas. Lo que mint añade a la estirpe *_climb:
  · DOS ascensos — cuentas (raíz en 279) y custodios (raíz en 319); el
    hash corre hasta ROW_CUST_ROOT.
  · cust_link: los enlaces del árbol de custodios {287, 295, 303, 311}.
  · sel_acct_root {279} y sel_cust_root {319}: selectores de fila que
    nacen del triple one-hot `for row in [0, ROW_ACCT_ROOT,
    ROW_CUST_ROOT]` — first_row incluido.
  · P_POW2: columna de VALORES (1<<nivel en las filas de cust_link),
    como las ARK — no es máscara y no entra en P2SEL.
  · `any_link = acct_link + cust_link`: alias-SUMA de selectores —
    forma que el núcleo v2 censa expandiendo un reclamo por sumando.
  · 8 segmentos Horner que llenan las 512 filas: aquí NO hay clase
    plana.
Doctrina, cosecha conservadora (§186) y sesgo al rojo (§59.2): en el
núcleo. Referenciada ≠ determinada (doc §1).
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import censo_nucleo as nucleo


def mascaras_mint(v):
    m = nucleo.mascaras_clasicas(v)
    pasos = range(v["TRACE_LENGTH"] - 1)
    # El hash de mint corre por AMBOS ascensos: hasta la raíz de custodios.
    m["hash_flag"] = frozenset(
        r for r in pasos
        if r <= v["ROW_CUST_ROOT"] and r % v["CYCLE_LENGTH"] < v["NUM_ROUNDS"])
    m["cust_link"] = frozenset(
        (v["CYC_ACCT_ROOT"] + lv) * v["CYCLE_LENGTH"] + 7
        for lv in range(v["CUSTODIAN_DEPTH"]))
    m["sel_acct_root"] = frozenset({v["ROW_ACCT_ROOT"]})
    m["sel_cust_root"] = frozenset({v["ROW_CUST_ROOT"]})
    return m


def sanidad_mint(v, masc):
    assert len(masc["hash_flag"]) == 280 and len(masc["acct_link"]) == 32
    assert len(masc["cust_link"]) == 4


P2SEL_MINT = dict(nucleo.P2SEL_CLASICO,
                  P_CUST_LINK="cust_link",
                  P_SEL_ACCT_ROOT="sel_acct_root",
                  P_SEL_CUST_ROOT="sel_cust_root")

ESPEC = {
    "FICHERO": "circuit_mint.rs",
    "TITULO": "circuit_mint",
    "ROTULO": "COMPUERTA-MINT",
    "COL_CIERRE": 44,
    "EXTERNAS": {"STATE_WIDTH": 12, "NUM_ROUNDS": 7, "CYCLE_LENGTH": 8,
                 "TREE_DEPTH": 32, "CUSTODIAN_DEPTH": 4},
    "ANCLAS": [
        ("for r in 0..=ROW_CUST_ROOT", 2),  # hash_flag + bucle de ARK
        ("if r % CYCLE_LENGTH < NUM_ROUNDS {", 1),
        ("hash_flag[r] = one;", 1),
        ("acct_link[ROW_LEAF_DONE] = one;", 1),
        ("for level in 0..TREE_DEPTH - 1 {", 1),
        ("acct_link[(CYC_ACC + level) * CYCLE_LENGTH + 7] = one;", 1),
        ("link_leaf[ROW_LEAF_LINK] = one;", 1),
        ("link_salt[ROW_SALT_LINK] = one;", 1),
        ("for level in 0..CUSTODIAN_DEPTH {", 2),  # periódicas + bits
        ("cust_link[row] = one;", 1),
        ("pow2[row] = BaseElement::new(1u64 << level);", 1),
        ("for row in [0, ROW_ACCT_ROOT, ROW_CUST_ROOT] {", 1),  # triple one-hot
        ("first_s[seg * SEGMENT_LENGTH] = one;", 1),
        ("cont_s[seg * SEGMENT_LENGTH + p] = one;", 1),
        ("link[(seg + 1) * SEGMENT_LENGTH - 2] = one;", 1),
        ("let any_link = acct_link + cust_link;", 1),  # alias-suma (v2)
    ],
    "ESPERADO": {"TRACE_WIDTH": 49, "TRACE_LENGTH": 512, "SEGMENT_LENGTH": 64,
                 "NUM_SEGMENTS": 8, "LANE_B": 12, "CYC_ACC": 3,
                 "ROW_LEAF_LINK": 7, "ROW_SALT_LINK": 15, "ROW_LEAF_DONE": 23,
                 "ROW_ACCT_ROOT": 279, "ROW_CUST_START": 280,
                 "ROW_CUST_ROOT": 319, "CYC_ACCT_ROOT": 35, "CYC_CUST": 36},
    "MASCARAS": mascaras_mint,
    "SANIDAD": sanidad_mint,
    "P2SEL": P2SEL_MINT,
    "MUTANTES": [
        ("C_HORNER", r"\n\s*result\[C_HORNER\][^;]*;",
         "el paso de Horner: el acumulador en el cuerpo de segmento"),
        ("C_SALT_IN_A", r"\n\s*result\[C_SALT_IN_A \+ i\][^;]*;",
         "la atadura del salt testigo al rate, carril A (§117/§138)"),
    ],
}

sys.exit(nucleo.correr(ESPEC))
