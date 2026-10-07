#!/usr/bin/env python3
"""
Comprueba que cada columna declarada en un circuito se RELLENE de verdad.

## Por qué existe

Al construir `circuit_mint_pending` se declararon **siete columnas** con sus
restricciones escritas, y la traza nunca les asignaba valor. Todas valían
cero, así que sus restricciones se cumplían trivialmente —`0 − 0 = 0`— y
**ningún test lo detectó**:

- Los tests negativos pasaban igual, porque fallaban por otras
  restricciones.
- El testigo honesto no revela nada: en su caso las columnas *deberían*
  tener valor, y él las rellenaría... si el código las rellenara.

Es el mismo modo de fallo que las **restricciones idénticamente cero**
documentadas en `AUDITORIA.md`. Nada automático lo detectaba.

## ⚠️ Esta comprobación se equivocó dos veces antes de funcionar

Primera versión: solo contaba lecturas de la forma `current[COL]`, así que
**no detectaba** columnas usadas dentro de literales de array.

Segunda versión: no reconocía el patrón `state[COL] = …` de `trace.fill`, y
daba **cinco falsos positivos** en `solvency.rs`.

Ambos errores se encontraron **validando la comprobación contra un caso
conocido**: quitar a propósito el relleno de una columna y confirmar que
salta, y comprobar que un circuito correcto no dispara.

**Una verificación rota es peor que ninguna**: dirige la atención al sitio
equivocado.

## ⚠️ Y después corrió en el canon sin mirar nada (§695)

Sin argumentos leía `.`, el directorio desde el que se la llamaba. El canon
la llama sin argumentos desde la raíz del repositorio, donde no hay ningún
`.rs`: decía «0 circuitos: todas las columnas declaradas se rellenan» y salía
con 0. El §511 y el §556 lo ficharon (5.A-334).

Y con su directorio tampoco miraba todo. Solo leía `const COL_…` al principio
de la línea, sin `pub`: en `crates/zk-ssl-air/src`, donde las AIR del kit
declaran `pub const COL_…`, encontraba cinco ficheros y CERO columnas, y en
`crates/stark-experiment/src` se le escapaban once de 270, todas `pub`. Las
AIR del kit no rellenan su traza: la rellena su probador, en
`crates/stark-experiment/src/circuit_<nombre>.rs`, con `fila[…]` y
`filas[…][…]`; y las once de `stark-experiment` se rellenan con
`t.set(COL, …)`, menos una. Tres patrones que tampoco reconocía.

Desde el §695 el universo vive aquí (`UNIVERSO`), con su **prueba de vida**:
sin argumentos mira los dos directorios, se la llame desde donde se la llame,
y sale con 1 si cualquiera de los dos baja de su mínimo de ficheros o de
columnas. Una puerta que cuenta lo que mira no puede volver a pasar vacía sin
que lo diga.

Y la primera pasada de ese arreglo se equivocó otra vez. Contaba
`.set(COL, …)` en cualquier sitio, y 16 de las 26 llamadas de los dos
directorios están en los módulos de test: son negativos que manipulan la
traza, no la rellenan. Quitar el relleno de `COL_R_ID` en
`circuit_mint_pending.rs`, que un negativo escribe con `trace.set`, daba
verde, y la versión de antes daba ROJO. Desde entonces, las escrituras se
cuentan fuera de los módulos `#[cfg(test)]`.

## Uso

    python3 tools/check_columns.py
    python3 tools/check_columns.py crates/stark-experiment/src

Sin argumentos, el `UNIVERSO`, con su prueba de vida; es lo que corre el
canon. Con argumentos, esos directorios, sin mínimos, pero un directorio que
no existe, sin ficheros o sin columnas también es ROJO: un universo vacío no
pasa. Sale con código 1 si encuentra alguna columna sin rellenar o si la
prueba de vida falla.

## Lo que reconoce

- Declaraciones: `const COL_…: usize`, con o sin `pub` o `pub(…)`, al
  principio de la línea o sangradas.
- Escrituras: `row[COL] = …`, `rows[…][COL] = …`, `fila[COL] = …`,
  `filas[…][COL] = …`, `state[COL] = …`, el tramo `row[COL..]`,
  `fila[COL..]` o `state[COL..]`, y `t.set(COL, …)` (`TraceTable::set`).
- Las escrituras cuentan sólo fuera de los módulos de test: un
  `#[cfg(test)]` y un `mod <nombre> {` en la columna 0, hasta la primera `}`
  sola en la columna 0. Lo que escribe un test cuenta como uso.
- Una columna de `crates/zk-ssl-air/src/<nombre>.rs` se busca rellenada en su
  fichero y en `crates/stark-experiment/src/circuit_<nombre>.rs`, el de su
  probador. El emparejamiento es por nombre de fichero: se declara aquí
  (`KIT`, `PROBADOR_DEL_KIT`) y no se deduce.

## ⚠️ Lo que NO comprueba

- Que la columna se rellene con el valor **correcto**.
- Que se rellene en **todas** las filas donde hace falta.
- Columnas que se rellenan con un patrón distinto de los de arriba.
- Que el tramo `row[COL..]` sea una escritura: lo cuenta como tal aunque se
  lea, como las versiones anteriores.
- Que el receptor de `.set(COL, …)` sea una `TraceTable`: lo cuenta como
  escritura sea cual sea el receptor.
- Un módulo de test con otra forma (sangrado, o con otro `cfg`) o un fichero
  entero de test, como `kat_probador.rs`, que `lib.rs` declara bajo
  `#[cfg(test)]`: no los recorta, y lo que escriben cuenta como relleno.
  Recortar de más, en cambio, sólo quita escrituras: da ROJO, no VERDE.
- `lib.rs` y `merkle.rs`, que salta por su nombre. En `zk-ssl-air` eso deja
  fuera la AIR de la edad: vive en su `lib.rs`, sus columnas se llaman `C_…`
  y su probador rellena por columnas, `col[C_…][fila]`. De las cinco AIR del
  kit, esta herramienta mira cuatro; `sal.rs` es el quinto fichero del
  directorio y no declara columnas.
- Que el fichero emparejado sea de verdad el probador de esa AIR: lo da por
  hecho por su nombre.

Un patrón nuevo de relleno haría que esta herramienta diera falsos
positivos, no falsos negativos: es el sentido seguro del error. Un patrón
nuevo de DECLARACIÓN, en cambio, la deja sin ver la columna, y por eso la
prueba de vida cuenta columnas, no solo ficheros.
"""

import os
import re
import sys

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# El universo sin argumentos y su prueba de vida (§695): directorio relativo a
# la raíz, mínimo de ficheros y mínimo de columnas. Los mínimos son los medidos
# en el §695. Un circuito nuevo no obliga a tocarlos; uno que se va, sí: es un
# suelo, no un pin, y bajarlo se hace aquí, a mano y con su asiento.
UNIVERSO = (
    ("crates/stark-experiment/src", 42, 270),
    ("crates/zk-ssl-air/src", 5, 37),
)

# Las AIR del kit viven en `zk-ssl-air` sin su probador; la traza la rellena
# `stark-experiment`, en `circuit_<nombre>.rs`.
KIT = "crates/zk-ssl-air/src"
PROBADOR_DEL_KIT = "crates/stark-experiment/src"

EXCLUIDOS = ("lib.rs", "merkle.rs")
IGNORAR = {"target", ".git"}

DECLARACION = re.compile(
    r"^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?const (COL_\w+)[ \t]*:[ \t]*usize", re.M
)

# Un módulo de test en línea: `#[cfg(test)]` y `mod <nombre> {` en la columna 0,
# con otros atributos en medio o sin ellos, hasta la primera `}` sola en la
# columna 0. Lo que escribe un test no rellena la traza: la manipula para un
# negativo, y contarlo como relleno es un falso negativo.
MODULO_DE_TEST = re.compile(r"^#\[cfg\(test\)\]\n(?:#\[[^\n]*\]\n)*mod \w+ \{$", re.M)
CIERRE = re.compile(r"^\}$", re.M)


def leer(ruta):
    with open(ruta, encoding="utf-8") as f:
        return f.read()


def sin_tests(c):
    """El texto de `c` sin sus módulos `#[cfg(test)]` en línea.

    Un módulo sin `}` en la columna 0 se lleva el resto del fichero: recortar de
    más sólo quita escrituras, y eso da ROJO, no VERDE."""
    trozos, i = [], 0
    for m in MODULO_DE_TEST.finditer(c):
        if m.start() < i:
            continue
        trozos.append(c[i : m.start()])
        fin = CIERRE.search(c, m.end())
        if not fin:
            return "".join(trozos)
        i = fin.end()
    trozos.append(c[i:])
    return "".join(trozos)


def escrituras(col, c):
    """Cuántas veces `c` escribe la columna `col`, con los patrones reconocidos."""
    n = 0
    # rows[..][COL] = …   y   filas[..][COL] = …
    n += len(re.findall(rf"\b(?:rows?|filas?)\[[^\]]*\]\[{col}[^\]]*\]\s*=(?!=)", c))
    # row[COL] = …, fila[COL] = …  y  state[COL] = …  (patrón de trace.fill)
    n += len(re.findall(rf"\b(?:row|fila|state)\[{col}[^\]]*\]\s*=(?!=)", c))
    # el tramo: row[COL..COL + n], fila[COL..], state[COL..]
    n += len(re.findall(rf"\b(?:row|fila|state)\[{col}\.\.", c))
    # t.set(COL, fila, valor)  (TraceTable::set)
    n += len(re.findall(rf"\.set\(\s*{col}\b", c))
    return n


def analizar(ruta, relleno=None):
    """Devuelve (columnas declaradas, [(columna, usos)] de las nunca rellenadas).

    `relleno` es el fichero del probador, si la traza se rellena fuera. Las
    escrituras se cuentan fuera de los módulos de test; los usos, en todo el
    fichero, y lo que escribe un test cuenta como uso."""
    c = leer(ruta)
    produccion = sin_tests(c)
    otro = sin_tests(leer(relleno)) if relleno and os.path.isfile(relleno) else ""
    columnas = DECLARACION.findall(c)
    malos = []
    for col in columnas:
        propias = escrituras(col, produccion)
        usos = len(re.findall(rf"\b{col}\b", c)) - 1 - propias
        if propias == 0 and usos > 0 and escrituras(col, otro) == 0:
            malos.append((col, usos))
    return columnas, malos


def ficheros(directorio):
    """Los `.rs` del directorio, subdirectorios incluidos, menos los EXCLUIDOS."""
    salida = []
    for base, dirs, nombres in os.walk(directorio):
        dirs[:] = sorted(d for d in dirs if d not in IGNORAR)
        for f in nombres:
            if f.endswith(".rs") and f not in EXCLUIDOS:
                salida.append(os.path.join(base, f))
    return sorted(salida)


def main(argv):
    if argv:
        universo = [(d, 1, 1) for d in argv]
    else:
        universo = [(os.path.join(RAIZ, d), mf, mc) for d, mf, mc in UNIVERSO]
    kit = os.path.realpath(os.path.join(RAIZ, KIT))

    total = 0
    vida = []
    n_ficheros = n_columnas = 0
    for directorio, min_f, min_c in universo:
        nombre = os.path.relpath(directorio, RAIZ) if not argv else directorio
        if not os.path.isdir(directorio):
            vida.append(f"{nombre}: no es un directorio")
            continue
        fs = ficheros(directorio)
        columnas = con_columnas = 0
        for ruta in fs:
            relleno = None
            if os.path.realpath(os.path.dirname(ruta)) == kit:
                relleno = os.path.join(
                    RAIZ, PROBADOR_DEL_KIT, "circuit_" + os.path.basename(ruta)
                )
            cols, malos = analizar(ruta, relleno)
            columnas += len(cols)
            con_columnas += bool(cols)
            if malos:
                total += len(malos)
                print(f"  !! {os.path.relpath(ruta, directorio)} ({nombre})")
                for col, usos in malos:
                    print(f"       {col}: 0 escrituras, {usos} usos")
        print(
            f"  {nombre}: {len(fs)} ficheros, {con_columnas} con columnas, "
            f"{columnas} columnas"
        )
        n_ficheros += len(fs)
        n_columnas += columnas
        if len(fs) < min_f or columnas < min_c:
            vida.append(
                f"{nombre}: {len(fs)} ficheros y {columnas} columnas, "
                f"por debajo de su minimo ({min_f} y {min_c})"
            )

    rojo = False
    if total:
        print(f"\n{total} columnas declaradas que la traza NUNCA rellena.")
        print("Sus restricciones se cumplen trivialmente y ningún test lo detecta.")
        rojo = True
    if vida:
        print("\nPRUEBA DE VIDA: la puerta miraria menos de lo que dice.")
        for v in vida:
            print(f"  !! {v}")
        rojo = True
    if rojo:
        return 1

    dirs = "1 directorio" if len(universo) == 1 else f"{len(universo)} directorios"
    print(
        f"{n_ficheros} ficheros en {dirs}, {n_columnas} columnas: "
        "todas las columnas declaradas se rellenan."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
