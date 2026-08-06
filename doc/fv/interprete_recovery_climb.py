#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Intérprete MULTI-CICLO de la RECUPERACIÓN — hermano derivado (§192).

Sexta pieza de la estirpe FV-1. El drift medido antes de derivar: 11/12
anclas idénticas — recovery construye first_row por asignación directa
(el ancla propia lo recoge; la máscara {0} no cambia). El sujeto es
circuit_recovery_climb. Desde §193 la lógica compartida vive UNA vez en
censo_nucleo.py; este fichero es la ESPEC del circuito.
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import censo_nucleo as nucleo

ESPEC = {
    "FICHERO": "circuit_recovery_climb.rs",
    "TITULO": "circuit_recovery_climb",
    "ROTULO": "COMPUERTA-RECOVERY",
    "COL_CIERRE": 38,
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
    ("first_row[0] = one;", 1),  # drift §192: asignación directa
    ("first_s[seg * SEGMENT_LENGTH] = one;", 1),
    ("cont_s[seg * SEGMENT_LENGTH + p] = one;", 1),
    ("link[(seg + 1) * SEGMENT_LENGTH - 2] = one;", 1),
],
    "ESPERADO": {"TRACE_WIDTH": 43, "TRACE_LENGTH": 512, "SEGMENT_LENGTH": 64,
                 "NUM_SEGMENTS": 1, "LANE_B": 12, "CYC_ACC": 3, "ROW_LEAF_LINK": 7,
                 "ROW_SALT_LINK": 15, "ROW_LEAF_DONE": 23, "ROW_ACCT_ROOT": 279},
    "MASCARAS": nucleo.mascaras_clasicas,
    "SANIDAD": nucleo.sanidad_clasica,
    "P2SEL": nucleo.P2SEL_CLASICO,
    "MUTANTES": [
        ("C_HORNER", r"\n\s*result\[C_HORNER\][^;]*;",
         "el paso de Horner: el acumulador en el cuerpo de segmento"),
        ("C_SALT_IN_A", r"\n\s*result\[C_SALT_IN_A \+ i\][^;]*;",
         "la atadura del salt testigo al rate, carril A (§117/§93.4: la copia)"),
    ],
}

sys.exit(nucleo.correr(ESPEC))
