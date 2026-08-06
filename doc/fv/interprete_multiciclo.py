#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Intérprete MULTI-CICLO — clase = conjunto explícito de filas (Capa 1, doc §9).

Cuarta pieza de la estirpe FV-1 (prototipo → un carril → dos carriles → este).
El sujeto es circuit_mint_climb: periódicas de longitud-de-traza (512 filas)
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

Desde §193 la lógica vive UNA vez en censo_nucleo.py; este fichero es la
ESPEC del circuito. La cosecha conservadora (§186) y el sesgo al rojo
(§59.2) están en el núcleo. FV-1 no afirma determinación: referenciada ≠
determinada (doc §1) — caza la celda que NADIE mira.
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import censo_nucleo as nucleo

ESPEC = {
    "FICHERO": "circuit_mint_climb.rs",
    "TITULO": "circuit_mint_climb",
    "ROTULO": "COMPUERTA",
    "COL_CIERRE": 37,
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
    "ESPERADO": {"TRACE_WIDTH": 42, "TRACE_LENGTH": 512, "SEGMENT_LENGTH": 64,
                 "NUM_SEGMENTS": 5, "LANE_B": 12, "CYC_ACC": 3, "ROW_LEAF_LINK": 7,
                 "ROW_SALT_LINK": 15, "ROW_LEAF_DONE": 23, "ROW_ACCT_ROOT": 279},
    "MASCARAS": nucleo.mascaras_clasicas,
    "SANIDAD": nucleo.sanidad_clasica,
    "P2SEL": nucleo.P2SEL_CLASICO,
    "MUTANTES": [
        ("C_HORNER", r"\n\s*result\[C_HORNER\][^;]*;",
         "el paso de Horner: el acumulador en el cuerpo de segmento"),
        ("C_SALT_IN_A", r"\n\s*result\[C_SALT_IN_A \+ i\][^;]*;",
         "la atadura del salt testigo al rate, carril A (§117/§138)"),
    ],
}

sys.exit(nucleo.correr(ESPEC))
