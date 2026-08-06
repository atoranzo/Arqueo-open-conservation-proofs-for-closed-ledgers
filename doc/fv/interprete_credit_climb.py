#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Intérprete MULTI-CICLO del CRÉDITO — hermano derivado (resto de la 71, §191).

Quinta pieza de la estirpe FV-1; nació derivado ANCLADO del intérprete de
mint (las 12 anclas del constructor de periódicas son idénticas en ambos —
se midió antes de derivar). El sujeto es circuit_credit_climb. Desde §193
la lógica compartida vive UNA vez en censo_nucleo.py; este fichero es la
ESPEC del circuito (doctrina §185/§186/§59.2 en el núcleo).
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import censo_nucleo as nucleo

ESPEC = {
    "FICHERO": "circuit_credit_climb.rs",
    "TITULO": "circuit_credit_climb",
    "ROTULO": "COMPUERTA-CREDIT",
    "COL_CIERRE": 34,
    "EXTERNAS": {"STATE_WIDTH": 12, "NUM_ROUNDS": 7, "CYCLE_LENGTH": 8, "TREE_DEPTH": 32},
    "ANCLAS": [
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
],
    "ESPERADO": {"TRACE_WIDTH": 39, "TRACE_LENGTH": 512, "SEGMENT_LENGTH": 64,
                 "NUM_SEGMENTS": 3, "LANE_B": 12, "CYC_ACC": 3, "ROW_LEAF_LINK": 7,
                 "ROW_SALT_LINK": 15, "ROW_LEAF_DONE": 23, "ROW_ACCT_ROOT": 279},
    "MASCARAS": nucleo.mascaras_clasicas,
    "SANIDAD": nucleo.sanidad_clasica,
    "P2SEL": nucleo.P2SEL_CLASICO,
    "MUTANTES": [
        ("C_HORNER", r"\n\s*result\[C_HORNER\][^;]*;",
         "el paso de Horner: el acumulador en el cuerpo de segmento"),
        ("C_SALT_IN_A", r"\n\s*result\[C_SALT_IN_A \+ i\][^;]*;",
         "la atadura del salt testigo al rate, carril A (§117: el cerrojo #2)"),
    ],
}

sys.exit(nucleo.correr(ESPEC))
