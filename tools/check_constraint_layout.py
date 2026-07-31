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

Y **la cadena de columnas PERIODICAS**, que hasta la entrada 39 no miraba
nadie: `result[...]` y `periodic[...]` son dos arrays distintos y esta
herramienta solo cruzaba uno.

     DESBORDE PERIODICA  se lee una columna por encima de las construidas
     MUERTA PERIODICA    se construye una columna que NADIE lee

Importa por lo que NO se ve: al extraer `circuit_mint_climb` quedaron tres
constantes `P_*` muertas y el indice se salio del array (§66.2). Se noto
**porque desbordo**. Si el desplazamiento fuera hacia abajo, la restriccion
leeria la columna periodica equivocada **en silencio**, activandose en las
filas que no son —la clase de §39 y §72, que ninguna otra comprobacion ve—.

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
# ⚠️ Captura el indice ENTERO, no «constante + desplazamiento». La primera
# version solo entendia `result[C_ALGO + i]` y saltaba **en silencio** los
# circuitos que indexan con numeros crudos —`result[24 + i]`, `result[44]`—.
# Eran DIEZ de veinticuatro, y el resumen decia «todos limpios» sobre los
# catorce que si entendia (§59.2).
RE_WRITE = re.compile(r"result\[([^\]]+)\]")
# Lecturas de la cadena periodica.
#
# ⚠️ **`periodic\w*` y no `periodic`, y esto ya fallo una vez.** La primera
# version de esta regex casaba solo `periodic[...]`, y SEIS circuitos
# -`merkle`, `rescue_hash`, `compliance_circuit`, `dual_climb`,
# `circuit_frozen_climb`, `nullifier`- llaman al parametro `periodic_values`.
# El barrido dio 159 columnas muertas que no existian: **el mismo agujero que
# RE_WRITE tenia antes de §59.2**, cometido al cerrarlo.
#
# Incluye ademas los cortes `periodic[P_A..P_A + N]`, que son 35 de las 217
# lecturas del repositorio.
RE_PERIODIC = re.compile(r"\bperiodic\w*\[([^\]]+)\]")
RE_PUSH = re.compile(r"\bcolumns\.push\(")
# `solvency` no usa `push`: devuelve `vec![a, b, c]` directamente. Se excluyen
# las formas `vec![x; n]`, que construyen UNA columna, no una lista.
RE_VEC = re.compile(r"vec!\[([^\[\]]*)\]")
RE_FN_PERIODIC = re.compile(r"fn get_periodic_column_values")
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
    expr = m.group(1).strip()
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
        salida.add(v)
    return salida


def _valores_de_expr(texto, pos, expr, valores):
    """Valores que toma una expresion en `pos`, expandiendo sus bucles.

    Es el nucleo de `indices_escritos`, extraido para que las lecturas de
    `periodic[...]` —que pueden ser cortes `a..b`— usen la MISMA maquinaria.
    Duplicarla habria hecho que las dos divergieran en silencio.
    """
    expr = expr.strip()
    envolventes = bucles_que_envuelven(texto, pos, valores)
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
        salida.add(v)
    return salida


def indices_leidos(texto, m, valores):
    """Indices que lee un `periodic[...]`, incluidos los cortes `a..b`."""
    expr = m.group(1).strip()
    if ".." not in expr:
        return _valores_de_expr(texto, m.start(), expr, valores)

    ini_txt, fin_txt = expr.split("..", 1)
    inclusivo = fin_txt.startswith("=")
    if inclusivo:
        fin_txt = fin_txt[1:]
    a = _valores_de_expr(texto, m.start(), ini_txt, valores)
    b = _valores_de_expr(texto, m.start(), fin_txt, valores)
    if a is None or b is None:
        return None
    salida = set()
    for x in a:
        for y in b:
            salida.update(range(x, y + 1 if inclusivo else y))
    return salida


def tamano_de_bucle(texto, rango, valores):
    """Cuantas veces itera un bucle, o None si no se acota.

    Para CONTAR empujes basta el tamano; los valores no hacen falta. Por eso
    acepta arrays literales como `[true, false]`, que `rango_de_bucle` no
    puede convertir en indices y devolveria None.
    """
    vals = rango_de_bucle(texto, rango, valores)
    if vals is not None:
        return len(vals)
    r = rango.strip()
    if r.startswith("[") and r.endswith("]") and ".iter()" not in r:
        n = contar_elementos(r[1:-1])
        return n if n else None
    return None


def _envolventes_acotadas(texto, pos, valores):
    """Producto de las iteraciones de los bucles abiertos en `pos`."""
    total = 1
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
        n = tamano_de_bucle(texto, m.group(2), valores)
        if n is None:
            return None
        total *= n
    return total


def _cuerpo_de_funcion(texto, nombre):
    """Cuerpo de una funcion y su desplazamiento, por conteo de llaves."""
    # ⚠️ Busqueda de texto PLANO, sin expresion regular.
    #
    # Un `\b` aqui tuvo que atravesar dos capas de comillas y se
    # colapso en un caracter de RETROCESO (0x08). El patron dejaba de casar,
    # el contador daba cero en los 27 circuitos y el barrido reportaba 823
    # desbordes inexistentes. Sin escapes no hay nada que colapsar.
    pos = texto.find("fn " + nombre)
    if pos < 0:
        return None
    try:
        ini = texto.index("{", pos + len(nombre) + 3)
    except ValueError:
        return None
    prof = 0
    for i in range(ini, len(texto)):
        if texto[i] == "{":
            prof += 1
        elif texto[i] == "}":
            prof -= 1
            if prof == 0:
                return texto[ini:i + 1], ini
    return None


def columnas_construidas(texto, valores):
    """Cuantas columnas periodicas construye el circuito, o None.

    ⚠️ **None significa NO COMPROBADO, nunca «cero».** Devolver 0 cuando no
    se entiende la construccion hace que TODA lectura parezca un desborde:
    `solvency` —que devuelve `vec![a, b, c]` en vez de usar `push`— aparecio
    asi con seis desbordes inexistentes.

    Un barrido que **condena** lo que no entiende es tan malo como uno que lo
    aprueba (§42.5, §59.2).
    """
    cuerpo_pos = _cuerpo_de_funcion(texto, "get_periodic_column_values")
    if cuerpo_pos is None:
        return 0  # el circuito no tiene cadena periodica
    cuerpo, desplazamiento = cuerpo_pos

    # Forma 1: `columns.push(...)`, expandiendo sus bucles.
    total, hubo = 0, False
    for m in RE_PUSH.finditer(cuerpo):
        hubo = True
        n = _envolventes_acotadas(texto, desplazamiento + m.start(), valores)
        if n is None:
            return None
        total += n
    if hubo:
        return total

    # Forma 2: `vec![a, b, c]` devuelto directamente.
    listas = [g for g in RE_VEC.findall(cuerpo) if ";" not in g]
    if listas:
        n = contar_elementos(listas[-1])
        if n:
            return n

    return None


def sin_comentarios(texto):
    """Sustituye el contenido de los comentarios por espacios, conservando las
    posiciones y los saltos de linea.

    Hace falta porque la documentacion del proyecto **ilustra el problema con
    codigo**: `mutation.rs` explica las restricciones vacuas escribiendo
    `result[C_X]` en su propia prosa, y el barrido lo tomaba por una escritura
    real que no sabia leer. Un aviso falso gasta la atencion que hace falta
    para los verdaderos.
    """
    salida = []
    i, n = 0, len(texto)
    while i < n:
        if texto.startswith("//", i):
            j = texto.find("\n", i)
            j = n if j == -1 else j
            salida.append(" " * (j - i))
            i = j
        elif texto.startswith("/*", i):
            j = texto.find("*/", i + 2)
            j = n if j == -1 else j + 2
            salida.append("".join(ch if ch == "\n" else " " for ch in texto[i:j]))
            i = j
        else:
            salida.append(texto[i])
            i += 1
    return "".join(salida)


def analizar(ruta):
    texto = sin_comentarios(open(ruta, encoding="utf-8").read())
    valores, crudos, sin_resolver = resolver_simbolos(texto)
    grupos = {n: valores[n] for n in crudos if n.startswith("C_") and n in valores}
    total = valores.get("NUM_CONSTRAINTS")
    if not RE_WRITE.search(texto):
        return None

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
    # Sin `NUM_CONSTRAINTS` no se puede saber cuantas ranuras deberia haber,
    # asi que no se puede hablar de ranuras muertas. Las COLISIONES —que son
    # la firma de §38— si se detectan igual.
    muertas = sorted(set(range(total)) - cubiertas) if total is not None else []

    # ===== LA CADENA PERIODICA (entrada 39) =====
    #
    # Mismo cruce, otro array. Se construyen N columnas y se leen indices; si
    # alguno se sale, la restriccion lee **otra columna** —y si se sale hacia
    # abajo, lo hace en silencio—.
    p_construidas = columnas_construidas(texto, valores)
    p_leidas, p_indeterminadas = set(), []
    for m in RE_PERIODIC.finditer(texto):
        idx = indices_leidos(texto, m, valores)
        linea = texto[: m.start()].count("\n") + 1
        if idx is None:
            p_indeterminadas.append((linea, m.group(0)))
        else:
            p_leidas.update(idx)

    if p_construidas is None:
        p_desbordes, p_muertas = [], []
    else:
        p_desbordes = sorted(i for i in p_leidas if i >= p_construidas)
        p_muertas = sorted(set(range(p_construidas)) - p_leidas)

    return {
        "total": total,
        "colisiones": colisiones,
        "desbordes": desbordes,
        "muertas": muertas,
        "indeterminadas": indeterminadas,
        "sin_resolver": sin_resolver,
        "grupos": sorted(grupos.items(), key=lambda kv: kv[1]),
        "p_construidas": p_construidas,
        "p_desbordes": p_desbordes,
        "p_muertas": p_muertas,
        "p_indeterminadas": p_indeterminadas,
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


CASO_66 = """
const P_UNO: usize = 0;
const P_DOS: usize = 1;
const P_TRES: usize = 2;
const C_ALGO: usize = 0;
const NUM_CONSTRAINTS: usize = 1;

fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
    let mut columns = Vec::new();
    columns.push(uno);
    columns.push(dos);
    columns
}

fn evaluate_transition(&self) {
    let a = periodic[P_UNO];
    let c = periodic[P_TRES];
    result[C_ALGO] = a * c;
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

    # ===== Y que caza la cadena periodica rota de §66.2 (entrada 39) =====
    #
    # `CASO_66` reproduce lo que quedo en `circuit_mint_climb`: una periodica
    # que se construye y nadie lee, y una lectura por encima de las
    # construidas. Un detector que nunca ha detectado nada no esta probado.
    with tempfile.NamedTemporaryFile("w", suffix=".rs", delete=False) as f:
        f.write(CASO_66)
        ruta = f.name
    try:
        r = analizar(ruta)
    finally:
        os.unlink(ruta)

    if r["p_construidas"] != 2:
        print(f"AUTOTEST FALLA: esperaba 2 columnas, conto {r['p_construidas']}")
        return 1
    if r["p_muertas"] != [1]:
        print(f"AUTOTEST FALLA: esperaba la columna 1 muerta, hallo {r['p_muertas']}")
        return 1
    if r["p_desbordes"] != [2]:
        print(f"AUTOTEST FALLA: esperaba desborde en 2, hallo {r['p_desbordes']}")
        return 1
    print("autotest: el detector caza la periodica muerta y el desborde de §66.2")
    return 0


def main():
    if "--autotest" in sys.argv:
        return autotest()
    verbose = "--verbose" in sys.argv
    graves = huecos = barridos = 0

    no_barridos = []
    for fichero in sorted(os.listdir(RAIZ)):
        if not fichero.endswith(".rs"):
            continue
        ruta = os.path.join(RAIZ, fichero)
        r = analizar(ruta)
        if r is None:
            # ⚠️ Un circuito que escribe restricciones pero no usa constantes
            # `C_` no se puede analizar con este barrido. Callarlo daria una
            # falsa seguridad: el resumen diria «todos limpios» sobre un
            # subconjunto. `dual_climb` indexa con numeros crudos
            # (`result[24 + i]`), y por eso quedaba fuera sin avisar.
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

        # ===== CADENA PERIODICA (entrada 39) =====
        for i in r["p_desbordes"]:
            lineas.append(
                f"    [DESBORDE PERIODICA] se lee periodic[{i}] y solo se "
                f"construyen {r['p_construidas']} columnas"
            )
            graves += 1

        if r["p_muertas"]:
            lineas.append(
                f"    [MUERTA PERIODICA] {len(r['p_muertas'])} columna(s) que "
                f"se construyen y NADIE lee: {r['p_muertas']}"
            )
            huecos += 1

        for linea, txt in r["p_indeterminadas"]:
            lineas.append(
                f"    [?] linea {linea}: lectura periodica no expandible  {txt}"
            )
            huecos += 1

        if r["p_construidas"] is None:
            lineas.append(
                "    [?] no se pudo contar cuantas columnas periodicas se "
                "construyen: la cadena queda SIN COMPROBAR"
            )
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
            f"{barridos} circuitos: ninguna ranura colisiona, desborda ni queda "
            "muerta, y ninguna columna periodica se lee fuera de rango ni se "
            "construye sin leerse."
        )
    if no_barridos:
        print()
        print(
            f"⚠️  {len(no_barridos)} circuito(s) NO analizados —indexan las "
            "restricciones con numeros crudos en vez de constantes `C_`, y este "
            "barrido no sabe leerlos:"
        )
        for f in no_barridos:
            print(f"      {f}")
        print(
            "    No estan aprobados: estan sin comprobar. Con indices a mano el "
            "defecto de §38 es MAS facil, no menos, porque no hay una cadena de "
            "constantes que delate el desajuste."
        )
    return 1 if graves else 0


if __name__ == "__main__":
    sys.exit(main())
