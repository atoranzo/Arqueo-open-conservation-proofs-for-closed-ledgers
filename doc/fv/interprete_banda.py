#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Intérprete de la BANDA del kit (zk-ssl-air/src/banda.rs).

La primera AIR del kit con censo. Es la que juzga la banda del saldo en
el hito de la edad, y la que la segunda implementación transcribe del .rs:
una celda sin dueño pasaría en las dos. Un carril de doce columnas, la
hoja de cuenta en los ciclos 0..2 (identidad, saldo, nonce y sal), la
subida del 3 al 34 y tres segmentos de rango sobre el saldo y sus dos
holguras (saldo, saldo - inferior, superior - saldo).

Las formas del kit que el núcleo aprende para ella (ESPEC["KIT"]): la
ronda por `ronda_rescue`, la fila siguiente llamada `siguiente` y la
tabla de transporte en un `let`. Doctrina, cosecha conservadora (§186) y
sesgo al rojo (§59.2): en el núcleo. Referenciada ≠ determinada (doc §1).
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import censo_nucleo as nucleo


def mascaras_banda(v):
    return nucleo.mascaras_kit(
        v,
        {"hash": nucleo.filas_hash(v, 0, v["ROW_RAIZ"])},
        {"link_leaf": v["ROW_ENLACE_HOJA"], "link_salt": v["ROW_ENLACE_SALT"],
         "link_place": v["ROW_HOJA_LISTA"], "first_row": 0},
        segmentos=True)


def sanidad_banda(v, masc):
    assert len(masc["hash"]) == 245 and len(masc["link_merkle"]) == 31
    assert len(masc["cont_s"]) == 189


ESPEC = {
    "RAIZ": nucleo.RAIZ_KIT,
    "FICHERO": "banda.rs",
    "TITULO": "banda (kit)",
    "ROTULO": "COMPUERTA-BANDA",
    "COL_CIERRE": 22,
    "KIT": True,
    "SIGUIENTE": "siguiente",
    "PERIODICA": "periodic",
    "ALIAS_CONST": dict(nucleo.ALIAS_KIT, NUM_SEGMENTS="SEGMENTOS"),
    "NO_SELECTORES": nucleo.NO_SELECTORES_KIT,
    "RONDAS_ESPERADAS": 1,
    "EXTERNAS": nucleo.EXTERNAS_KIT,
    "ANCLAS": [
        ("for r in 0..=ROW_RAIZ {", 2),  # bandera + bucle de ARK
        ("if r % CICLO < RONDAS {", 1),
        ("bandera[r] = uno;", 1),
        ("enlace_arbol[(CYC_ACC + nivel) * CICLO + CICLO - 1] = uno;", 1),
        ("for nivel in 0..PROFUNDIDAD - 1 {", 1),
        ("for fila in [ROW_ENLACE_HOJA, ROW_ENLACE_SALT, ROW_HOJA_LISTA, 0] {", 1),
        ("primera_s[seg * LARGO_SEGMENTO] = uno;", 1),
        ("sigue_s[seg * LARGO_SEGMENTO + p] = uno;", 1),
        ("enlace[(seg + 1) * LARGO_SEGMENTO - 2] = uno;", 1),
        ("let enlace = enlace_arbol + enlace_sitio;", 1),  # alias-suma
        ("let transporte = [", 1),
        ("ronda_rescue(", 1),
    ],
    "ESPERADO": {"ANCHO": 27, "TRAZA": 512, "LARGO_SEGMENTO": 64, "SEGMENTOS": 3,
                 "NUM_RESTRICCIONES": 63, "CYC_ACC": 3, "CYC_RAIZ": 35,
                 "ROW_ENLACE_HOJA": 7, "ROW_ENLACE_SALT": 15, "ROW_HOJA_LISTA": 23,
                 "ROW_RAIZ": 279, "P_HASH_FLAG": 0, "P_LINK_MERKLE": 25,
                 "P_LINK_LEAF": 26, "P_LINK_SALT": 27, "P_LINK_PLACE": 28,
                 "P_FIRST_ROW": 29, "P_FIRST_S": 30, "P_CONT_S": 31, "P_SEG_LINK": 32,
                 "NUM_PERIODICAS": 35},
    "MASCARAS": mascaras_banda,
    "SANIDAD": sanidad_banda,
    "P2SEL": {"P_HASH_FLAG": "hash", "P_LINK_MERKLE": "link_merkle",
              "P_LINK_LEAF": "link_leaf", "P_LINK_SALT": "link_salt",
              "P_LINK_PLACE": "link_place", "P_FIRST_ROW": "first_row",
              "P_FIRST_S": "first_s", "P_CONT_S": "cont_s"},
    "MUTANTES": [
        ("C_HORNER", r"\n\s*result\[C_HORNER\][^;]*;",
         "el paso de Horner: el acumulador en el cuerpo de segmento"),
        ("C_SALT_IN", r"\n\s*result\[C_SALT_IN \+ i\][^;]*;",
         "la atadura de la sal testigo al rate de la hoja"),
    ],
}

if __name__ == "__main__":
    sys.exit(nucleo.correr(ESPEC))
