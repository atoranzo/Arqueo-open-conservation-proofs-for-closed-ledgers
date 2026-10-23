#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Intérprete de la PRENDA del kit (zk-ssl-air/src/prenda.rs).

El molde del cobro pendiente sin banda y sin meta, con dos ciclos nuevos
en el carril B: el de la CLAVE en el ciclo 0 (la identidad derivada, que
la fila 7 compara con el receptor) y el de la MARCA en el ciclo 3,
sembrado en la fila 23 con el C2 que entra en la subida (D-AW). La
bandera del B es la única del kit que no es un tramo: dos ciclos sueltos.
Sin segmentos de rango, así que sin candidato C_SEG_LINK.

Doctrina, cosecha conservadora (§186) y sesgo al rojo (§59.2): en el
núcleo. Referenciada ≠ determinada (doc §1).
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import censo_nucleo as nucleo


def mascaras_prenda(v):
    C, R = v["CICLO"], v["RONDAS"]
    hash_b = [ciclo * C + pos for ciclo in (0, v["CYC_MARCA"]) for pos in range(R)]
    return nucleo.mascaras_kit(
        v,
        {"hash_a": nucleo.filas_hash(v, 0, v["ROW_RAIZ"]), "hash_b": hash_b},
        {"link_importe": v["ROW_ENLACE_IMPORTE"], "link_x": v["ROW_ENLACE_X"],
         "link_place": v["ROW_HOJA_LISTA"], "first_row": 0},
        segmentos=False)


def sanidad_prenda(v, masc):
    assert len(masc["hash_a"]) == 245 and len(masc["hash_b"]) == 14
    assert len(masc["link_merkle"]) == 31


ESPEC = {
    "RAIZ": nucleo.RAIZ_KIT,
    "FICHERO": "prenda.rs",
    "TITULO": "prenda (kit)",
    "ROTULO": "COMPUERTA-PRENDA",
    "COL_CIERRE": None,
    "KIT": True,
    "SIGUIENTE": "siguiente",
    "PERIODICA": "periodic",
    "ALIAS_CONST": nucleo.ALIAS_KIT,
    "NO_SELECTORES": nucleo.NO_SELECTORES_KIT,
    "RONDAS_ESPERADAS": 2,
    "EXTERNAS": nucleo.EXTERNAS_KIT,
    "ANCLAS": [
        ("for r in 0..=ROW_RAIZ {", 2),  # bandera A + bucle de ARK
        ("if r % CICLO < RONDAS {", 1),
        ("bandera_a[r] = uno;", 1),
        ("for ciclo in [0, CYC_MARCA] {", 1),  # bandera B: dos ciclos sueltos
        ("for pos in 0..RONDAS {", 1),
        ("bandera_b[ciclo * CICLO + pos] = uno;", 1),
        ("enlace_arbol[(CYC_ACC + nivel) * CICLO + CICLO - 1] = uno;", 1),
        ("for nivel in 0..PROFUNDIDAD - 1 {", 1),
        ("for fila in [ROW_ENLACE_IMPORTE, ROW_ENLACE_X, ROW_HOJA_LISTA, 0] {", 1),
        ("let enlace = enlace_arbol + enlace_sitio;", 1),
        ("const TRANSPORTE: [usize; 17] = [", 1),
        ("ronda_rescue(", 2),
    ],
    "ESPERADO": {"ANCHO": 42, "TRAZA": 512, "NUM_RESTRICCIONES": 106, "CYC_ACC": 3,
                 "CYC_MARCA": 3, "CYC_RAIZ": 35, "ROW_ENLACE_IMPORTE": 7,
                 "ROW_ENLACE_X": 15, "ROW_HOJA_LISTA": 23, "ROW_MARCA": 31,
                 "ROW_RAIZ": 279, "C_A": 0, "C_B": 30, "P_HASH_A": 0, "P_HASH_B": 1,
                 "P_LINK_MERKLE": 26, "P_LINK_IMPORTE": 27, "P_LINK_X": 28,
                 "P_LINK_PLACE": 29, "P_FIRST_ROW": 30, "NUM_PERIODICAS": 31},
    "MASCARAS": mascaras_prenda,
    "SANIDAD": sanidad_prenda,
    "P2SEL": {"P_HASH_A": "hash_a", "P_HASH_B": "hash_b", "P_LINK_MERKLE": "link_merkle",
              "P_LINK_IMPORTE": "link_importe", "P_LINK_X": "link_x",
              "P_LINK_PLACE": "link_place", "P_FIRST_ROW": "first_row"},
    "CANDIDATO": None,
    "MUTANTES": [
        ("C_X_IN", r"\n\s*result\[C_X_IN \+ i\][^;]*;",
         "el sobre X testigo en el rate de C2 (D-I)"),
        ("C_MARCA_CAP", r"\n\s*result\[C_MARCA_CAP\][^;]*;",
         "el dominio de la marca en la capacidad (D-AW)"),
    ],
}

if __name__ == "__main__":
    sys.exit(nucleo.correr(ESPEC))
