#!/usr/bin/env python3
"""Detecta ranuras de restriccion escritas dos veces, sobrepasadas o muertas.

ENTRADA 37. El solapamiento de §38 —dos sitios distintos de
`evaluate_transition` escribiendo en la MISMA ranura, de modo que el segundo
pisa al primero y la restriccion pisada no se impone— produjo TRES fallos de
solidez en el circuito de creacion de pagos (§39, §50, §50.7). Ninguna
herramienta del proyecto lo detectaba: el detector de restricciones vacuas no
puede, porque ve reaccionar la ranura sobrescrita (§38).

Esta lo detecta, y lo hace por **indice absoluto**, no por grupo. Es
importante: en los circuitos de dos carriles `result[C_HASH_A + lane * 12 + i]`
escribe a proposito dentro del rango de `C_HASH_B` cuando `lane = 1`. Eso es
correcto y no es colision, porque lo escribe UN solo sitio del codigo.

Para cada circuito:

  1. resuelve las constantes a valores numericos;
  2. expande cada `result[...]` sobre los bucles que lo envuelven, obteniendo
     el conjunto de indices absolutos que escribe;
  3. cruza los conjuntos:

     COLISION   una ranura escrita por dos sitios distintos -> firma de §38
     DESBORDE   una ranura por encima de NUM_CONSTRAINTS
     MUERTA     una ranura por debajo de NUM_CONSTRAINTS que nadie escribe

Lo que esta herramienta NO puede hacer: los indices salen de un analisis
sintactico, no de ejecutar el circuito. Cuando una expresion no se puede
expandir con certeza se marca INDETERMINADA y **cuenta como hueco del
barrido, no como aprobado**. Un barrido que aprueba lo que no entiende es
peor que no tener barrido: fue el error de §42.5, en esta misma auditoria.

Uso:
    python3 tools/check_constraint_layout.py
    python3 tools/check_constraint_layout.py --verbose
"""

import itertools
import os
import re
import sys

RAIZ = os.path.join(os.path.dirname(__file__), "..", "crates", "stark-experiment", "src")

# Constantes de otros modulos que la cadena necesita. Si aparece una nueva sin
# declarar aqui, el barrido lo dice en vez de suponer un valor.
EXTERNAS = {
    "STATE_WIDTH": 12,
    "NUM_ROUNDS": 7,
    "TREE_DEPTH": 32,
    "FROZEN_DEPTH": 24,
    "CUSTODIAN_DEPTH": 4,
    "CYCLE_LENGTH": 8,
}

RE_CONST = re.compile(r"^(?:pub )?const ([A-Z][A-Z_0-9]*): usize = (.+?);", re.MULTILINE)
RE_WRITE = re.compile(r"result\[\s*(C_[A-Z_0-9]*)\s*(?:\+\s*([^\]]+?))?\s*\]")
RE_FOR = re.compile(r"\bfor\s+(\([^)]*\)|[a-z_][a-z_0-9]*)\s+in\s+([^{]+?)\s*\{")
RE_LET_ARRAY = re.compile(r"let\s+([a-z_][a-z_0-9]*)\s*=\s*\[([^\]]*)\]\s*;", re.S)


def resolver_simbolos(texto):
    crudos = {m.group(1): m.group(2).strip() for m in RE_CONST.finditer(texto)}
    valores = dict(EXTERNAS)
    for _ in range(len(crudos) + 2):
        cambio = False
        for nombre, expr in crudos.items():
            if nombre in valores:
                continue
            try:
                v = eval(expr, {"__builtins__": {}}, valores)  # noqa: S307
            except Exception:
                continue
            if isinstance(v, int):
                valores[nombre] = v
                cambio = True
        if not cambio:
            break
    return valores, crudos, [n for n in crudos if n not in valores]


def contar_elementos(cuerpo):
    """Cuenta elementos de un cuerpo de array, respetando anidamiento.

    Se parte por comas de nivel cero y se cuentan los trozos NO vacios. Contar
    comas + 1 daria uno de mas cuando hay coma final, que es el estilo del
    proyecto: ese error hizo que este barrido reportara NUEVE colisiones
    inexistentes la primera vez que se ejecuto, en circuitos que acababan de
    corregirse. Verificar la herramienta antes de creerle es parte del metodo.
    """
    prof, trozos, actual = 0, [], []
    for ch in cuerpo:
        if ch in "([":
            prof += 1
        elif ch in ")]":
            prof -= 1
        if ch == "," and prof == 0:
            trozos.append("".join(actual))
            actual = []
        else:
            actual.append(ch)
    trozos.append("".join(actual))
    return len([t for t in trozos if t.strip()])


def elementos_de_array(texto, nombre):
    """Cuenta elementos de `let nombre = [ ... ];`."""
    for m in RE_LET_ARRAY.finditer(texto):
        if m.group(1) == nombre:
            return contar_elementos(m.group(2))
    return None


def rango_de_bucle(texto, rango, valores):
    """Conjunto de valores de la variable de un bucle, o None si no se acota."""
    rango = rango.strip()

    m = re.match(r"^(.+?)\.\.(=?)(.+)$", rango)
    if m and ".iter()" not in rango:
        try:
            ini = eval(m.group(1), {"__builtins__": {}}, valores)  # noqa: S307
            fin = eval(m.group(3), {"__builtins__": {}}, valores)  # noqa: S307
        except Exception:
            return None
        if isinstance(ini, int) and isinstance(fin, int):
            return set(range(ini, fin + 1 if m.group(2) else fin))
        return None

    # for (k, x) in ARRAY.iter().enumerate()
    m = re.match(r"^([a-z_][a-z_0-9]*)\s*\.iter\(\)\s*\.enumerate\(\)$", rango, re.S)
    if m:
        n = elementos_de_array(texto, m.group(1))
        return set(range(n)) if n is not None else None

    # for (k, x) in [A, B, C].iter().enumerate()  — array en linea, que el
    # proyecto parte en varias lineas.
    m = re.match(r"^\[(.*)\]\s*\.iter\(\)\s*\.enumerate\(\)$", rango, re.S)
    if m:
        return set(range(contar_elementos(m.group(1))))

    if rango.startswith("[") and rango.endswith("]"):
        primeros = re.findall(r"\(\s*([0-9]+)(?:usize)?\s*,", rango)
        if primeros:
            return {int(p) for p in primeros}
        return None

    return None


def bucles_que_envuelven(texto, pos, valores):
    """Bucles abiertos y no cerrados en `pos`, con el rango de su variable."""
    fuera = []
    for m in RE_FOR.finditer(texto[:pos]):
        try:
            ini = texto.index("{", m.end() - 1)
        except ValueError:
            continue
        prof, cerrado = 0, False
        for i in range(ini, pos):
            if texto[i] == "{":
                prof += 1
            elif texto[i] == "}":
                prof -= 1
                if prof == 0:
                    cerrado = True
                    break
        if cerrado:
            continue
        var, rango = m.group(1), m.group(2)
        vals = rango_de_bucle(texto, rango, valores)
        if var.startswith("("):
            nombres = [v.strip() for v in var[1:-1].split(",")]
            fuera.append((nombres[0], vals))
            for otro in nombres[1:]:
                fuera.append((otro, None))
        else:
            fuera.append((var, vals))
    return fuera


def indices_escritos(texto, m, valores):
    """Indices absolutos que escribe una sentencia, o None si no se expande."""
    base_nombre, expr = m.group(1), m.group(2)
    if base_nombre not in valores:
        return None
    base = valores[base_nombre]
    if expr is None:
        return {base}

    envolventes = bucles_que_envuelven(texto, m.start(), valores)
    libres = sorted(set(re.findall(r"\b([a-z_][a-z_0-9]*)\b", expr)))
    libres = [v for v in libres if v not in valores]

    dominios = []
    for v in libres:
        vals = None
        for nombre, conjunto in envolventes:
            if nombre == v:
                vals = conjunto
                break
        if vals is None:
            return None
        dominios.append(sorted(vals))

    salida = set()
    for combo in (itertools.product(*dominios) if dominios else [()]):
        entorno = dict(valores)
        entorno.update(dict(zip(libres, combo)))
        try:
            v = eval(expr, {"__builtins__": {}}, entorno)  # noqa: S307
        except Exception:
            return None
        if not isinstance(v, int):
            return None
        salida.add(base + v)
    return salida


def analizar(ruta):
    texto = open(ruta, encoding="utf-8").read()
    valores, crudos, sin_resolver = resolver_simbolos(texto)
    grupos = {n: valores[n] for n in crudos if n.startswith("C_") and n in valores}
    if not grupos:
        return None
    total = valores.get("NUM_CONSTRAINTS")

    escrituras, indeterminadas = [], []
    for m in RE_WRITE.finditer(texto):
        idx = indices_escritos(texto, m, valores)
        linea = texto[: m.start()].count("\n") + 1
        if idx is None:
            indeterminadas.append((linea, m.group(0)))
        else:
            escrituras.append((linea, idx))

    duenos, colisiones = {}, {}
    for linea, idx in escrituras:
        for i in idx:
            if i in duenos and duenos[i] != linea:
                colisiones.setdefault(i, set()).update({duenos[i], linea})
            else:
                duenos.setdefault(i, linea)

    cubiertas = set(duenos)
    desbordes = sorted(i for i in cubiertas if total is not None and i >= total)
    muertas = sorted(set(range(total)) - cubiertas) if total is not None else []

    return {
        "total": total,
        "colisiones": colisiones,
        "desbordes": desbordes,
        "muertas": muertas,
        "indeterminadas": indeterminadas,
        "sin_resolver": sin_resolver,
        "grupos": sorted(grupos.items(), key=lambda kv: kv[1]),
    }


def grupo_de(indice, grupos):
    anterior = None
    for nombre, valor in grupos:
        if valor > indice:
            break
        anterior = (nombre, valor)
    if anterior is None:
        return "?"
    return f"{anterior[0]}+{indice - anterior[1]}"


CASO_50 = """
const C_A: usize = 0;
const C_TRANSPORT: usize = C_A + 1;
const C_ID_CONST: usize = C_TRANSPORT + 7;
const C_SBIT_BOOL: usize = C_ID_CONST + 4;
const C_FIRST_S: usize = C_SBIT_BOOL + 2;
const NUM_CONSTRAINTS: usize = C_FIRST_S + 2;
fn evaluate() {
    result[C_A] = uno;
    let transport = [
        COL_UNO,
        COL_DOS,
        COL_TRES,
        COL_CUATRO,
        COL_CINCO,
        COL_SEIS,
        COL_SIETE,
    ];
    for (k, col) in transport.iter().enumerate() {
        result[C_TRANSPORT + k] = algo;
    }
    for i in 0..4 {
        result[C_TRANSPORT + 7 + i] = identidad;
        result[C_TRANSPORT + 11 + i] = aleatorio;
    }
    for i in 0..4 {
        result[C_ID_CONST + i] = algo;
    }
    result[C_SBIT_BOOL] = algo;
    result[C_SBIT_BOOL + 1] = algo;
    result[C_FIRST_S] = algo;
    result[C_FIRST_S + 1] = algo;
}
"""


def autotest():
    """Comprueba que el detector caza el fallo real de §50.

    Un detector que nunca ha detectado nada no esta probado. `CASO_50`
    reproduce la disposicion que tenia `circuit_send` la manana del
    30-07-2026: `C_TRANSPORT` declara 7 ranuras y escribe 15, asi que las 8
    ultimas pisan a `C_ID_CONST`. Esas 8 restricciones muertas fueron el fallo
    de solidez de §50.
    """
    import tempfile

    with tempfile.NamedTemporaryFile("w", suffix=".rs", delete=False) as f:
        f.write(CASO_50)
        ruta = f.name
    try:
        r = analizar(ruta)
    finally:
        os.unlink(ruta)

    n = len(r["colisiones"])
    if n != 8:
        print(f"AUTOTEST FALLA: esperaba 8 colisiones en el caso de §50, hallo {n}")
        return 1
    print("autotest: el detector caza las 8 ranuras pisadas del fallo de §50")
    return 0


def main():
    if "--autotest" in sys.argv:
        return autotest()
    verbose = "--verbose" in sys.argv
    graves = huecos = barridos = 0

    for fichero in sorted(os.listdir(RAIZ)):
        if not fichero.endswith(".rs"):
            continue
        r = analizar(os.path.join(RAIZ, fichero))
        if r is None:
            continue
        barridos += 1
        lineas = []

        for i, sitios in sorted(r["colisiones"].items()):
            lineas.append(
                f"    [COLISION] ranura {i} ({grupo_de(i, r['grupos'])}): "
                f"escrita en las lineas {sorted(sitios)}"
            )
            graves += 1

        for i in r["desbordes"]:
            lineas.append(
                f"    [DESBORDE] ranura {i} >= NUM_CONSTRAINTS={r['total']}"
            )
            graves += 1

        if r["muertas"]:
            gm = sorted({grupo_de(i, r["grupos"]).split("+")[0] for i in r["muertas"]})
            lineas.append(
                f"    [MUERTA] {len(r['muertas'])} ranura(s) sin escribir "
                f"(grupos: {', '.join(gm)})"
            )
            huecos += 1

        for linea, txt in r["indeterminadas"]:
            lineas.append(f"    [?] linea {linea}: no se pudo expandir  {txt}")
            huecos += 1

        for n in r["sin_resolver"]:
            lineas.append(f"    [?] constante no resuelta: {n}")
            huecos += 1

        if lineas:
            print(f"\n  {fichero}")
            for ln in lineas:
                print(ln)
        elif verbose:
            print(f"\n  {fichero}: {r['total']} ranuras, cada una escrita una vez")

    print()
    if graves:
        print(
            f"{graves} problema(s) GRAVE(S) en {barridos} circuitos. Una COLISION "
            "es la firma de §38 y puede ser un fallo de solidez como §39, §50 o "
            "§50.7: se mira con un test discriminante, no por lectura."
        )
    if huecos:
        print(
            f"{huecos} punto(s) que este barrido NO ha comprobado. No son "
            "aprobados: son huecos del propio barrido, y hay que mirarlos a mano."
        )
    if not graves and not huecos:
        print(
            f"{barridos} circuitos: ninguna ranura colisiona, desborda ni queda muerta."
        )
    return 1 if graves else 0


if __name__ == "__main__":
    sys.exit(main())
