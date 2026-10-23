#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Intérprete del COBRO PENDIENTE del kit (zk-ssl-air/src/cobro_pendiente.rs).

Dos carriles de doce columnas y un solo bit (D-H): el carril A compone
M(receptor, sal) en el ciclo 0, C1 en el 1 y C2 en el 2; el B no hashea
en los ciclos 0 y 1 (su bandera arranca en CYC_META * CICLO), compone la
hoja de meta en el 2, y los dos suben del 3 al 34. Tres segmentos de
rango: importe, importe - inferior, superior - importe.

Lo que el núcleo aprende aquí: DOS banderas de hash (hash_a, hash_b) que
la ronda lee EN LÍNEA (`periodic[P_HASH_B]`), la subida de los dos
carriles por un bucle de TUPLAS y la tabla `const TRANSPORTE`. Doctrina,
cosecha conservadora (§186) y sesgo al rojo (§59.2): en el núcleo.
Referenciada ≠ determinada (doc §1).
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import censo_nucleo as nucleo


def mascaras_cobro(v):
    return nucleo.mascaras_kit(
        v,
        {"hash_a": nucleo.filas_hash(v, 0, v["ROW_RAIZ"]),
         "hash_b": nucleo.filas_hash(v, v["CYC_META"] * v["CICLO"], v["ROW_RAIZ"])},
        {"link_importe": v["ROW_ENLACE_IMPORTE"], "link_x": v["ROW_ENLACE_X"],
         "link_place": v["ROW_HOJA_LISTA"], "first_row": 0},
        segmentos=True)


def sanidad_cobro(v, masc):
    assert len(masc["hash_a"]) == 245 and len(masc["hash_b"]) == 231
    assert len(masc["link_merkle"]) == 31 and len(masc["cont_s"]) == 189


ESPEC = {
    "RAIZ": nucleo.RAIZ_KIT,
    "FICHERO": "cobro_pendiente.rs",
    "TITULO": "cobro pendiente (kit)",
    "ROTULO": "COMPUERTA-COBRO",
    "COL_CIERRE": 21,
    "KIT": True,
    "SIGUIENTE": "siguiente",
    "PERIODICA": "periodic",
    "ALIAS_CONST": dict(nucleo.ALIAS_KIT, NUM_SEGMENTS="SEGMENTOS"),
    "NO_SELECTORES": nucleo.NO_SELECTORES_KIT,
    "RONDAS_ESPERADAS": 2,
    "EXTERNAS": nucleo.EXTERNAS_KIT,
    "ANCLAS": [
        ("for desde in [0, CYC_META * CICLO] {", 1),  # las dos banderas
        ("for r in desde..=ROW_RAIZ {", 1),
        ("for r in 0..=ROW_RAIZ {", 1),  # bucle de ARK
        ("if r % CICLO < RONDAS {", 1),
        ("bandera[r] = uno;", 1),
        ("enlace_arbol[(CYC_ACC + nivel) * CICLO + CICLO - 1] = uno;", 1),
        ("for nivel in 0..PROFUNDIDAD - 1 {", 1),
        ("for fila in [ROW_ENLACE_IMPORTE, ROW_ENLACE_X, ROW_HOJA_LISTA, 0] {", 1),
        ("primera_s[seg * LARGO_SEGMENTO] = uno;", 1),
        ("sigue_s[seg * LARGO_SEGMENTO + p] = uno;", 1),
        ("enlace[(seg + 1) * LARGO_SEGMENTO - 2] = uno;", 1),
        ("let enlace = enlace_arbol + enlace_sitio;", 1),
        ("for (c_cap, c_place, base) in [(C_CAP_A, C_PLACE_A, C_A), (C_CAP_B, C_PLACE_B, C_B)] {", 1),
        ("const TRANSPORTE: [usize; 17] = [", 1),
        ("ronda_rescue(", 2),
    ],
    "ESPERADO": {"ANCHO": 44, "TRAZA": 512, "LARGO_SEGMENTO": 64, "SEGMENTOS": 3,
                 "NUM_RESTRICCIONES": 110, "CYC_META": 2, "CYC_ACC": 3, "CYC_RAIZ": 35,
                 "ROW_ENLACE_IMPORTE": 7, "ROW_ENLACE_X": 15, "ROW_HOJA_LISTA": 23,
                 "ROW_RAIZ": 279, "C_A": 0, "C_B": 30, "P_HASH_A": 0, "P_HASH_B": 1,
                 "P_LINK_MERKLE": 26, "P_LINK_IMPORTE": 27, "P_LINK_X": 28,
                 "P_LINK_PLACE": 29, "P_FIRST_ROW": 30, "P_FIRST_S": 31, "P_CONT_S": 32,
                 "P_SEG_LINK": 33, "NUM_PERIODICAS": 36},
    "MASCARAS": mascaras_cobro,
    "SANIDAD": sanidad_cobro,
    "P2SEL": {"P_HASH_A": "hash_a", "P_HASH_B": "hash_b", "P_LINK_MERKLE": "link_merkle",
              "P_LINK_IMPORTE": "link_importe", "P_LINK_X": "link_x",
              "P_LINK_PLACE": "link_place", "P_FIRST_ROW": "first_row",
              "P_FIRST_S": "first_s", "P_CONT_S": "cont_s"},
    "MUTANTES": [
        ("C_X_IN", r"\n\s*result\[C_X_IN \+ i\][^;]*;",
         "la atadura del sobre X testigo al rate de C2 (D-I)"),
        ("C_M_IN + 1", r"\n\s*result\[C_M_IN \+ 1\][^;]*;",
         "el nacido en la hoja de meta"),
    ],
}

if __name__ == "__main__":
    sys.exit(nucleo.correr(ESPEC))
