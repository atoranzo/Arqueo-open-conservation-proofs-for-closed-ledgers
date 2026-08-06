#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Intérprete de la RECLAMACIÓN (circuit_claim) — novena pieza (§196).

La gemela invertida de send sobre la misma geometría de 1024: dos
carriles, 55 columnas (sin COL_LIMIT), 4 segmentos Horner (0..255 — los
hitos 279/287 y los enlaces frozen quedan FUERA de cont_s: clases
puras que send no tenía). Las dos inversiones viven en el código y el
censo las lee igual: el saldo SUBE (C_BALANCE con `+`) y el pendiente
SALE (carril A arrastra el compromiso, B entra a cero). El cierre de
§39 es estructural y se ancla: el compromiso interno ata la identidad
a `COL_ACC_ID` — la cuenta que COBRA —, no a `COL_R_ID`.
Las dos formas del núcleo v4 (arrays con suma de literal; suma INLINE
`(frozen_entry + frozen_link)`) también están aquí: v4 ya las habla.
Sin POW2. Doctrina, cosecha conservadora (§186) y sesgo al rojo
(§59.2): en el núcleo. Referenciada ≠ determinada (doc §1).
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import censo_nucleo as nucleo


def sanidad_claim(v, masc):
    assert len(masc["hash_flag"]) == 714 and len(masc["link_merkle"]) == 31
    assert len(masc["frozen_link"]) == 31 and len(masc["pend_link"]) == 31
    assert len(masc["cont_s"]) == 252


ESPEC = {
    "FICHERO": "circuit_claim.rs",
    "TITULO": "circuit_claim",
    "ROTULO": "COMPUERTA-CLAIM",
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
        ("AirContext::new(trace_info, degrees, 41, options)", 1),
        # El cierre de §39, anclado: el compromiso ata la identidad de la
        # CUENTA QUE COBRA, no COL_R_ID.
        ("result[C_PEND_IN + 4 + i] = pend_in * (next[4 + i] - "
         "current[COL_ACC_ID + i]);", 1),
    ],
    "ESPERADO": {"TRACE_WIDTH": 55, "TRACE_LENGTH": 1024, "SEGMENT_LENGTH": 64,
                 "NUM_SEGMENTS": 4, "LANE_B": 12, "NUM_CONSTRAINTS": 201,
                 "CYC_ACC": 3, "CYC_PK": 35, "CYC_FROZEN": 36,
                 "CYC_PEND_IN": 68, "CYC_PEND_CLIMB": 70, "CYC_FIN": 102,
                 "ROW_LEAF_LINK": 7, "ROW_SALT_LINK": 15, "ROW_LEAF_DONE": 23,
                 "ROW_ROOT": 279, "ROW_PK_START": 280, "ROW_PK_DONE": 287,
                 "ROW_FROZEN_ROOT": 543, "ROW_PEND_INNER": 551,
                 "ROW_PENDING_ENTRY": 559, "ROW_PENDING_ROOT": 815},
    "MASCARAS": nucleo.mascaras_gemelas,
    "SANIDAD": sanidad_claim,
    "P2SEL": nucleo.P2SEL_GEMELAS,
    "MUTANTES": [
        ("C_HORNER", r"\n\s*result\[C_HORNER\][^;]*;",
         "el paso de Horner: el acumulador en el cuerpo de segmento"),
        ("C_SALT_IN_A", r"\n\s*result\[C_SALT_IN_A \+ i\][^;]*;",
         "la atadura del salt testigo al rate, carril A (§117/§138)"),
    ],
}

sys.exit(nucleo.correr(ESPEC))
