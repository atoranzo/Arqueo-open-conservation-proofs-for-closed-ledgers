#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Intérprete de la EDAD del kit (zk-ssl-air/src/lib.rs, `EdadAir`).

La única AIR del kit con tramo auxiliar, y la única CÍCLICA: la traza
mide 8 * 2^m y cada ciclo de 8 filas es un nodo del árbol (tres carriles
de Rescue, A, B y la hoja de meta M). Sus clases de fila no dependen de m:
la fila 0 del ciclo (f0, con hash), las 1..5 (hash), la 6 (f6, con hash)
y la 7 (el enlace, `1 - hf`). El censo corre con la traza mínima, m = 3.

Lo que la doctrina de stark-experiment no ve y aquí se añade, declarado:
  · un AIR cíclico ata la fila 0 de cada ciclo por su fila ACTUAL
    (`f0 * c[C_A + k]`, la meta, la edad, el emisor). Con ESPEC["ACTUAL"],
    una referencia a la fila actual en la transición r+1 es dueña de la
    celda de la fila r+1, si su selector está activo en la clase de r+1;
  · el tramo auxiliar (el multiconjunto) lee en f0 los hijos y la hoja del
    pendiente y escribe en f6 la salida del ciclo. Esos dueños los declara
    RECLAMOS_EXTRA y los anclan las líneas de `evaluate_aux_transition`.
Las celdas del propio tramo auxiliar (X_H, X_S) no entran: el censo mira
la traza principal. Las de la principal que solo lee el auxiliar, sí.

Doctrina, cosecha conservadora (§186) y sesgo al rojo (§59.2): en el
núcleo. Referenciada ≠ determinada (doc §1): una referencia a la fila
actual solo es dueña BAJO SELECTOR, y en todas las sucesoras de la clase.
La booleanidad `c[x] * (c[x] - 1)` no lleva selector: referencia qa, qs y
los bits en todas las filas, pero no los determina, y fuera de la fila 0
del ciclo se declaran libres (lib.rs). Contarla como dueña tapaba la
constancia de vivo dentro del ciclo: el tercer mutante lo mide.
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import censo_nucleo as nucleo

# La traza mínima: m = 3, 8 hojas, 8 * 8 filas (D-AG).
TRAZA_CENSO = 64


def mascaras_edad(v):
    C, R = v["CICLO"], v["RONDAS"]
    pasos = range(v["TRACE_LENGTH"] - 1)
    return {
        "hash": frozenset(r for r in pasos if r % C < R),
        "f0": frozenset(r for r in pasos if r % C == 0),
        "f6": frozenset(r for r in pasos if r % C == 6),
        "enlace": frozenset(r for r in pasos if r % C == C - 1),
    }


def sanidad_edad(v, masc):
    assert len(masc["hash"]) == 56 and len(masc["f0"]) == 8
    assert len(masc["f6"]) == 8 and len(masc["enlace"]) == 7


def cuatro(nombre):
    return lambda v: range(v[nombre], v[nombre] + 4)


def de(desde, nombre):
    return lambda v: range(v[nombre] + desde, v[nombre] + desde + 4)


ESPEC = {
    "RAIZ": nucleo.RAIZ_KIT,
    "FICHERO": "lib.rs",
    "TITULO": "edad (kit)",
    "ROTULO": "COMPUERTA-EDAD",
    "COL_CIERRE": None,
    "KIT": True,
    "SIGUIENTE": "s",
    "ACTUAL": "c",
    "PERIODICA": "periodic_values",
    "ALIAS_CONST": {"TRACE_WIDTH": "ANCHO", "STATE_WIDTH": "ESTADO",
                    "CYCLE_LENGTH": "CICLO", "NUM_ROUNDS": "RONDAS"},
    "NO_SELECTORES": ("P_ARK1", "P_ARK2"),
    "RONDAS_ESPERADAS": 1,
    "EXTERNAS": {"TRACE_LENGTH": TRAZA_CENSO},
    "SEL_EXTRA": {"enlace": "enlace"},
    "ANCLAS": [
        # las periódicas: hash en 0..6, f0 y f6 de una fila del ciclo
        ("let mut hash = vec![BaseElement::ONE; RONDAS];", 1),
        ("hash.push(BaseElement::ZERO);", 1),
        ("cols.push(fila(0));", 1),
        ("cols.push(fila(6));", 1),
        ("let enlace = E::ONE - hf;", 1),
        ("for base in [C_A, C_B, C_M] {", 1),
        ("let (a, b) = (&c[base..base + ESTADO], &s[base..base + ESTADO]);", 1),
        ("ronda_rescue(", 1),  # la llamada; la definicion lleva `<E: ...>`
        # el tramo auxiliar: lo que declara RECLAMOS_EXTRA, línea a línea
        ("codificar(&pot, beta, izq, dig(c, C_A + 4), ca),", 1),
        ("codificar(&pot, beta, der, dig(c, C_A + 8), ca),", 1),
        ("codificar(&pot, beta, izq, dig(c, C_B + 4), cb),", 1),
        ("codificar(&pot, beta, der, dig(c, C_B + 8), cb),", 1),
        ("codificar(&pot, beta, hojas + ciclo, dig(c, C_P), ca),", 1),
        ("let m = dig(s, C_M + 4);", 1),
        ("codificar(&pot, beta, ciclo, dig(s, C_A + 4), ca),", 1),
        ("codificar(&pot, beta, ciclo, dig(s, C_B + 4), cb),", 1),
        ("let ciclo = E::from(c[C_CICLO]);", 1),
        ("let act = E::from(c[C_ACT]);", 1),
        ("let vivo = E::from(c[C_VIVO]);", 1),
        ("result[k] = f0 * (x[X_H + k] * lecturas[k] - E::ONE);", 1),
        ("result[5 + k] = f6 * (x[X_H + k] * escrituras[k] - E::ONE);", 1),
        # los bits del resto: se acumulan en `resto` y lo lee la edad bajo f0
        ("resto += c[C_BITS + b] * peso;", 1),
        ("result[i] = f0 * (E::ONE - c[C_QA]) * (t - E::ONE - seq + c[C_NACIDO] - resto);", 1),
    ],
    "ESPERADO": {"ANCHO": 83, "PRINCIPALES": 110, "AUXILIARES": 9, "CICLO": 8,
                 "ESTADO": 12, "RONDAS": 7, "BITS": 32, "C_A": 0, "C_B": 12,
                 "C_M": 24, "C_P": 36, "C_EMISOR": 40, "C_NACIDO": 41, "C_VIVO": 42,
                 "C_QA": 43, "C_QS": 44, "C_WINV": 45, "C_Z": 46, "C_CUENTA": 47,
                 "C_CICLO": 48, "C_ACT": 49, "C_FUERA": 50, "C_BITS": 51,
                 "P_HASH": 0, "P_F0": 1, "P_F6": 2, "P_ARK1": 3, "P_ARK2": 15},
    "MASCARAS": mascaras_edad,
    "SANIDAD": sanidad_edad,
    "P2SEL": {"P_HASH": "hash", "P_F0": "f0", "P_F6": "f6"},
    # (selector, fila, columnas, porqué): los dueños del tramo auxiliar.
    "RECLAMOS_EXTRA": [
        ("f0", "actual", de(4, "C_A"), "lectura del hijo izquierdo, carril A"),
        ("f0", "actual", de(8, "C_A"), "lectura del hijo derecho, carril A"),
        ("f0", "actual", de(4, "C_B"), "lectura del hijo izquierdo, carril B"),
        ("f0", "actual", de(8, "C_B"), "lectura del hijo derecho, carril B"),
        ("f0", "actual", cuatro("C_P"), "lectura de la hoja del pendiente"),
        ("f0", "actual", lambda v: [v["C_CICLO"], v["C_ACT"]], "etiquetas y suma"),
        ("f6", "siguiente", de(4, "C_A"), "escritura de la salida, carril A"),
        ("f6", "siguiente", de(4, "C_B"), "escritura de la salida, carril B"),
        ("f6", "siguiente", de(4, "C_M"), "escritura de la hoja de meta"),
        # ⚠️ Esta no es dueña de ninguna celda-clase: la fila 5 comparte clase
        # (`hash`) con las 1..4, y una referencia de la fila actual solo cuenta
        # si su selector rige en TODAS las sucesoras. Se declara porque el
        # tramo auxiliar las lee; el vivo de la fila 6 lo ata la constancia
        # `hf * (s[C_VIVO] - vivo)`, y el tercer mutante lo mide.
        ("f6", "actual", lambda v: [v["C_CICLO"], v["C_ACT"], v["C_VIVO"]],
         "etiqueta, suma y hoja de meta viva"),
        ("f0", "actual", lambda v: range(v["C_BITS"], v["C_BITS"] + v["BITS"]),
         "los bits del resto, que la edad suma en `resto` (no es un alias de la fila)"),
    ],
    "CANDIDATO": None,
    "MUTANTES": [
        ("la meta en la fila 0", r"\n\s*result\[i\] = f0 \* \(c\[C_M \+ k\] - esperado\);",
         "el estado inicial de la hoja de meta: dominio, emisor y nacido"),
        ("la cuenta", r"\n\s*result\[i \+ 5\] = s\[C_CUENTA\][^;]*;",
         "la cuenta de viejos: lo que la prueba afirma"),
        # Un selector equivocado, no un borrado: la ranura sigue escrita y el
        # barrido no lo ve. Con la booleanidad como dueña, el censo tampoco
        # lo veía; el test discriminante es
        # `un_vivo_que_se_apaga_en_la_fila_6_no_verifica` (circuit_edad.rs).
        ("la constancia de vivo", r"result\[i\] = hf \* \(s\[C_VIVO\] - vivo\);",
         "vivo constante solo en el enlace: el vivo de la fila 6 ya no es el de la 0",
         "result[i] = enlace * (s[C_VIVO] - vivo);"),
    ],
}

if __name__ == "__main__":
    sys.exit(nucleo.correr(ESPEC))
