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

Y **las AIR del kit** (`crates/zk-ssl-air/src`), que juzgan las pruebas de
los hitos y que la segunda implementacion transcribe del `.rs`. Hasta que
entraron, el barrido no miraba su crate, y sus formas le eran ajenas: 45
puntos sin expandir. Ahora las entiende —el contador secuencial de la edad
(`let mut i = 0; result[i] = ...; i += 1;`), los cortes
`&mut result[a..b]` que recibe `ronda_rescue`, los bucles de tuplas de los
dos carriles, las tablas `const TRANSPORTE`, la lista periodica con otro
nombre, las constantes que cada AIR importa de la raiz del crate y el total
de ranuras del tramo auxiliar— y cada forma tiene su mutante en el
autotest. Los censos FV-1 de las cinco AIR corren con las suyas.

En las dos raices del arbol, un hueco tambien es GRAVE: las dos estan en 0
huecos (medido), y una pisada en una forma que el barrido no expande sale
como hueco, no como COLISION. Y el autotest corre siempre antes del
barrido: si falla, el veredicto sobre el arbol no vale y el exit es 1.

Uso:
    python3 tools/check_constraint_layout.py
    python3 tools/check_constraint_layout.py --verbose
    python3 tools/check_constraint_layout.py --autotest   (solo el autotest, con su salida)
"""

import itertools
import os
import re
import subprocess
import sys

RAIZ = os.path.join(os.path.dirname(__file__), "..", "crates", "stark-experiment", "src")
RAIZ_KIT = os.path.join(os.path.dirname(__file__), "..", "crates", "zk-ssl-air", "src")

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
RE_CONST_ARRAY = re.compile(
    r"(?:pub\s+)?const\s+([A-Z][A-Z_0-9]*)\s*:\s*\[[^\]]*\]\s*=\s*\[([^\]]*)\]\s*;", re.S
)
# Los contadores secuenciales: `let mut i = 0;` y luego `result[i] = ...; i += 1;`.
RE_LET_MUT = re.compile(r"\blet\s+mut\s+([a-z_][a-z_0-9]*)\s*(?::\s*usize\s*)?=\s*([^;]+);")

# Los nombres del total de ranuras. Los circuitos de `stark-experiment` usan
# `NUM_CONSTRAINTS`; el kit, `NUM_RESTRICCIONES` en sus cuatro AIR de un solo
# tramo y `PRINCIPALES` / `AUXILIARES` en la edad. Sin el total no hay
# ranuras MUERTAS ni DESBORDES que mirar.
TOTALES = ("NUM_CONSTRAINTS", "NUM_RESTRICCIONES", "PRINCIPALES")
TOTALES_AUX = ("NUM_AUX_CONSTRAINTS", "AUXILIARES")


def resolver_simbolos(texto, del_crate=None):
    """Constantes del fichero resueltas a enteros.

    `del_crate` son las constantes que el fichero importa de la raiz de su
    crate (`use crate::{CICLO, ESTADO, ...}` en el kit). ⚠️ Una constante
    del propio fichero GANA a la del crate: el cobro pendiente declara su
    `C_A` y su `C_B` (columnas de sus carriles) y la raiz del kit declara
    otras con el mismo nombre (columnas de la edad).
    """
    crudos = {m.group(1): m.group(2).strip() for m in RE_CONST.finditer(texto)}
    valores = dict(EXTERNAS)
    for nombre, v in (del_crate or {}).items():
        if nombre not in crudos:
            valores[nombre] = v
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
    """Elementos de `let nombre = [ ... ];` o de `const NOMBRE: [T; N] = [ ... ];`.

    El kit declara su tabla de transporte como constante del modulo
    (`const TRANSPORTE: [usize; 17] = [...]`), no como `let` dentro de la
    funcion: sin la segunda forma, `result[C_TRANSPORT + k]` quedaba sin
    expandir en el cobro pendiente, el pago en curso y la prenda.
    """
    for m in RE_LET_ARRAY.finditer(texto):
        if m.group(1) == nombre:
            return partir_elementos(m.group(2))
    for m in RE_CONST_ARRAY.finditer(texto):
        if m.group(1) == nombre:
            return partir_elementos(m.group(2))
    return None


def partir_elementos(cuerpo):
    """Trozos de nivel cero de un cuerpo de array, sin los vacios."""
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
    return [t.strip() for t in trozos if t.strip()]


def valor_de(expr, valores):
    """Valor entero de una expresion, o None. Acepta el sufijo `usize`."""
    e = re.sub(r"\b([0-9]+)usize\b", r"\1", expr.strip())
    try:
        v = eval(e, {"__builtins__": {}}, valores)  # noqa: S307
    except Exception:
        return None
    return v if isinstance(v, int) and not isinstance(v, bool) else None


def iteraciones_de_bucle(texto, patron, rango, valores):
    """Lista de entornos `{variable: valor}`, uno por iteracion, o None.

    Un valor None dentro de un entorno quiere decir «esta variable existe
    pero no se sabe cuanto vale» (un `true`, una expresion de campo). Usarla
    en un indice deja ese indice INDETERMINADO, nunca aprobado.

    ⚠️ **Las variables de un patron de tupla van JUNTAS.** El kit sube sus
    dos carriles con `for (c_cap, c_place, base) in [(C_CAP_A, C_PLACE_A,
    C_A), (C_CAP_B, C_PLACE_B, C_B)]`. Expandir cada variable por su lado y
    cruzarlas daria cuatro combinaciones, dos de ellas falsas: `c_cap` del
    carril A con `base` del B. El barrido pintaria escrituras que no existen.
    """
    rango = rango.strip()
    if patron.startswith("("):
        nombres = [v.strip() for v in patron[1:-1].split(",")]
    else:
        nombres = [patron]

    def entornos(tuplas):
        salida = []
        for tupla in tuplas:
            d = {n: None for n in nombres}
            for n, v in zip(nombres, tupla):
                d[n] = v
            salida.append(d)
        return salida

    # for i in a..b, for i in a..=b
    m = re.match(r"^(.+?)\.\.(=?)(.+)$", rango, re.S)
    if m and ".iter()" not in rango and not rango.startswith("["):
        ini, fin = valor_de(m.group(1), valores), valor_de(m.group(3), valores)
        if ini is None or fin is None:
            return None
        return entornos((v,) for v in range(ini, fin + 1 if m.group(2) else fin))

    # for (k, x) in ARRAY.iter().enumerate()
    m = re.match(r"^([A-Za-z_][A-Za-z_0-9]*)\s*\.iter\(\)\s*\.enumerate\(\)$", rango, re.S)
    if m:
        elems = elementos_de_array(texto, m.group(1))
        if elems is None:
            return None
        return entornos((k, valor_de(e, valores)) for k, e in enumerate(elems))

    # for (k, x) in [A, B, C].iter().enumerate()  — array en linea, que el
    # proyecto parte en varias lineas.
    m = re.match(r"^\[(.*)\]\s*\.iter\(\)\s*\.enumerate\(\)$", rango, re.S)
    if m:
        elems = partir_elementos(m.group(1))
        return entornos((k, valor_de(e, valores)) for k, e in enumerate(elems))

    # for x in [A, B, C], for (a, b) in [(A, B), (C, D)]
    if rango.startswith("[") and rango.endswith("]"):
        filas = []
        for e in partir_elementos(rango[1:-1]):
            if e.startswith("(") and e.endswith(")"):
                filas.append(tuple(valor_de(x, valores) for x in partir_elementos(e[1:-1])))
            else:
                filas.append((valor_de(e, valores),))
        return entornos(filas)

    return None


def bucles_que_envuelven(texto, pos, valores):
    """Bucles abiertos y no cerrados en `pos`: (variables, iteraciones), de fuera adentro."""
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
        patron, rango = m.group(1), m.group(2)
        if patron.startswith("("):
            nombres = [v.strip() for v in patron[1:-1].split(",")]
        else:
            nombres = [patron]
        fuera.append((nombres, iteraciones_de_bucle(texto, patron, rango, valores)))
    return fuera


def _valores_de_expr(texto, pos, expr, valores, extra=None):
    """Valores que toma una expresion en `pos`, expandiendo sus bucles.

    Es el nucleo de `indices_escritos` y de las lecturas de `periodic[...]`
    —que pueden ser cortes `a..b`—, para que usen la MISMA maquinaria.
    Duplicarla habria hecho que las dos divergieran en silencio.

    `extra` son variables ya fijadas (los contadores del interprete
    secuencial): no se buscan en los bucles.
    """
    expr = expr.strip()
    base = dict(valores)
    if extra:
        base.update(extra)
    envolventes = bucles_que_envuelven(texto, pos, valores) if extra is None else []
    libres = sorted(set(re.findall(r"\b([a-z_][a-z_0-9]*)\b", expr)))
    libres = [v for v in libres if v not in base]

    # Cada variable libre sale del bucle MAS EXTERNO que la nombra (el orden
    # de siempre); los bucles necesarios se cruzan enteros, con sus tuplas.
    necesarios = []
    for v in libres:
        for k, (nombres, _) in enumerate(envolventes):
            if v in nombres:
                if k not in necesarios:
                    necesarios.append(k)
                break
        else:
            return None
    listas = []
    for k in necesarios:
        its = envolventes[k][1]
        if its is None:
            return None
        listas.append(its)

    salida = []
    for combo in (itertools.product(*listas) if listas else [()]):
        entorno = dict(base)
        for d in combo:
            entorno.update(d)
        if any(entorno.get(v) is None for v in libres):
            return None
        v = valor_de(expr, entorno)
        if v is None:
            return None
        salida.append(v)
    return salida


def _valores_de_indice(texto, pos, expr, valores, extra=None):
    """Indices de un `x[...]`, incluidos los cortes `a..b`, CON repeticiones."""
    expr = expr.strip()
    if ".." not in expr:
        return _valores_de_expr(texto, pos, expr, valores, extra)
    ini_txt, fin_txt = expr.split("..", 1)
    inclusivo = fin_txt.startswith("=")
    if inclusivo:
        fin_txt = fin_txt[1:]
    a = _valores_de_expr(texto, pos, ini_txt, valores, extra)
    b = _valores_de_expr(texto, pos, fin_txt, valores, extra)
    if a is None or b is None or len(a) != len(b):
        return None
    # ⚠️ Los extremos van en PAREJA, de la misma iteracion: `result[i..i + N]`
    # dentro de un bucle no es el producto de todos los inicios por todos los
    # finales.
    salida = []
    for x, y in zip(a, b):
        salida.extend(range(x, y + 1 if inclusivo else y))
    return salida


def indices_leidos(texto, m, valores):
    """Indices que lee un `periodic[...]`, incluidos los cortes `a..b`."""
    v = _valores_de_indice(texto, m.start(), m.group(1), valores)
    return None if v is None else set(v)


def tamano_de_bucle(texto, rango, valores):
    """Cuantas veces itera un bucle, o None si no se acota.

    Para CONTAR empujes basta el tamano; los valores no hacen falta. Por eso
    acepta arrays literales como `[true, false]`, cuyos elementos no son
    indices.
    """
    its = iteraciones_de_bucle(texto, "_", rango, valores)
    return len(its) if its is not None else None


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


DELEGADA = "delegada"


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

    # Un envoltorio que DELEGA la cadena en otro AIR (`self.0.get_periodic_
    # column_values()`, los dobles de prueba de `circuit_edad`) no construye
    # columnas propias: su cadena se comprueba donde vive. Si el fichero
    # ademas LEE periodicas, `analizar` lo marca como hueco.
    if re.fullmatch(r"\{\s*self\.\w+\.get_periodic_column_values\(\)\s*\}", cuerpo):
        return DELEGADA

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

    # Forma 1b: la lista que la funcion DEVUELVE, se llame como se llame. El
    # kit la llama `columnas` y la edad `cols`; con `columns` fijo, las cinco
    # AIR del kit quedaban con la cadena periodica sin comprobar. Se cuentan
    # solo los `push` a la variable devuelta: la edad empuja tambien a sus
    # columnas sueltas (`hash.push(...)`, `v.push(...)`), que no son la lista.
    m_dev = re.search(r"\b([a-z_][a-z_0-9]*)\s*\}\s*$", cuerpo)
    if m_dev:
        total, hubo = 0, False
        for m in re.finditer(r"\b%s\.push\(" % re.escape(m_dev.group(1)), cuerpo):
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


_AUSENTE = object()


def _fin_de_sentencia(texto, i):
    """Fin (exclusivo) de la sentencia que empieza en `i`, o None.

    Una sentencia acaba en el `;` de nivel cero. Un bloque que no lleva `;`
    (`if ... { } else { }`) acaba en su llave de cierre si lo que sigue no es
    un `else`.
    """
    prof = 0
    j, n = i, len(texto)
    while j < n:
        ch = texto[j]
        if ch in "([{":
            prof += 1
        elif ch in ")]}":
            prof -= 1
            if prof < 0:
                return None
            if ch == "}" and prof == 0:
                resto = texto[j + 1:].lstrip()
                if not resto.startswith("else") and not resto.startswith(";") \
                        and not resto.startswith(".") and not resto.startswith(")"):
                    # `let x = match k { ... };` sigue con `;`: no acaba aqui.
                    cabeza = texto[i:j].lstrip()
                    if re.match(r"(if|while|loop|match|unsafe)\b|\{", cabeza):
                        return j + 1
        elif ch == ";" and prof == 0:
            return j + 1
        j += 1
    return None


def interpretar_contadores(cuerpo, valores, contadores, texto=None):
    """Ejecuta el cuerpo de una funcion con sus contadores secuenciales.

    La edad no escribe `result[C_X + k]`: lleva un contador, `let mut i = 0;`,
    y escribe `result[i] = ...; i += 1;`. El barrido sintactico no puede
    expandir eso sin seguir el orden de las sentencias: eran 18 de los 45
    puntos sin comprobar del kit. Este interprete lo sigue.

    Entiende: `let mut c = <entero>;`, `c += <expr>;`, `c = <expr>;`, los
    `for` con las formas de `iteraciones_de_bucle` (anidados, desenrollados
    en orden) y las escrituras `result[...]` en cualquier sentencia, cortes
    incluidos.

    Devuelve `{posicion_en_cuerpo: [indices, con repeticiones]}` con TODAS
    las ejecuciones de cada sitio, o None si no puede seguir el cuerpo: un
    contador que cambia dentro de un `if`, de un cierre o de un bucle que no
    se acota. ⚠️ None es «no comprobado»: el sitio vuelve a ser un hueco del
    barrido, nunca un aprobado.

    ⚠️ Con un contador, un mismo sitio PUEDE escribir dos veces la misma
    ranura (un `i += 1` olvidado dentro de un bucle). Por eso devuelve las
    repeticiones: es la firma de §38 en su forma secuencial.
    """
    salida = {}

    def ejecutar(ini, fin, entorno):
        p = ini
        while p < fin:
            m_ws = re.compile(r"\s*").match(cuerpo, p)
            p = m_ws.end()
            if p >= fin:
                return True
            m_for = RE_FOR.match(cuerpo, p)
            if m_for:
                abre = cuerpo.index("{", m_for.end() - 1)
                prof, cierra = 0, None
                for k in range(abre, fin):
                    if cuerpo[k] == "{":
                        prof += 1
                    elif cuerpo[k] == "}":
                        prof -= 1
                        if prof == 0:
                            cierra = k
                            break
                if cierra is None:
                    return False
                bloque = cuerpo[abre + 1:cierra]
                toca = RE_WRITE.search(bloque) or any(
                    re.search(r"\b%s\s*(\+=|=[^=])" % c, bloque) for c in contadores)
                entorno_valores = dict(valores)
                entorno_valores.update({k: v for k, v in entorno.items() if v is not None})
                # Las tablas (`const TRANSPORTE: [usize; N]`) viven fuera de la
                # funcion: se buscan en el fichero entero.
                its = iteraciones_de_bucle(texto if texto is not None else cuerpo,
                                           m_for.group(1), m_for.group(2), entorno_valores)
                if its is None:
                    if toca:
                        return False
                else:
                    # ⚠️ El MISMO entorno: lo que un contador avanza dentro del
                    # bucle sigue avanzado en la vuelta siguiente y al salir.
                    # Solo las variables del bucle vuelven a lo que eran.
                    antes = {k: entorno.get(k, _AUSENTE) for it in its[:1] for k in it}
                    for it in its:
                        entorno.update(it)
                        if not ejecutar(abre + 1, cierra, entorno):
                            return False
                    for k, v in antes.items():
                        if v is _AUSENTE:
                            entorno.pop(k, None)
                        else:
                            entorno[k] = v
                p = cierra + 1
                continue

            f = _fin_de_sentencia(cuerpo[:fin], p)
            if f is None:
                return False
            sent = cuerpo[p:f]
            # Un contador que cambia dentro de llaves (un `if`, un cierre) no
            # se sigue: el orden ya no es el del texto.
            interior = sent[sent.find("{"):] if "{" in sent else ""
            for c in contadores:
                if re.search(r"\b%s\s*(\+=|-=|=[^=])" % c, interior):
                    return False
            m_let = RE_LET_MUT.match(sent)
            if m_let and m_let.group(1) in contadores:
                v = valor_de(m_let.group(2), dict(valores, **{
                    k: x for k, x in entorno.items() if x is not None}))
                if v is None:
                    return False
                entorno[m_let.group(1)] = v
            else:
                m_asig = re.match(r"([a-z_][a-z_0-9]*)\s*(\+=|-=|=)\s*([^;]+);", sent)
                if m_asig and m_asig.group(1) in contadores:
                    v = valor_de(m_asig.group(3), dict(valores, **{
                        k: x for k, x in entorno.items() if x is not None}))
                    if v is None:
                        return False
                    viejo = entorno.get(m_asig.group(1))
                    if m_asig.group(2) != "=" and viejo is None:
                        return False
                    entorno[m_asig.group(1)] = (
                        v if m_asig.group(2) == "="
                        else viejo + v if m_asig.group(2) == "+="
                        else viejo - v)
            for m_w in RE_WRITE.finditer(sent):
                fijas = {k: x for k, x in entorno.items() if x is not None}
                idx = _valores_de_indice(cuerpo, p + m_w.start(), m_w.group(1), valores, fijas)
                pos = p + m_w.start()
                if idx is None:
                    salida[pos] = None
                elif salida.get(pos, []) is not None:
                    salida.setdefault(pos, []).extend(idx)
            p = f
        return True

    # `_cuerpo_de_funcion` devuelve el cuerpo CON sus llaves.
    ini, fin = (1, len(cuerpo) - 1) if cuerpo.startswith("{") else (0, len(cuerpo))
    if not ejecutar(ini, fin, {}):
        return None
    return salida


def contadores_de(cuerpo, valores):
    """Los `let mut c = <entero>;` del cuerpo que indexan un `result[...]`."""
    usados = set()
    for m in RE_WRITE.finditer(cuerpo):
        usados.update(re.findall(r"\b([a-z_][a-z_0-9]*)\b", m.group(1)))
    return sorted(
        m.group(1) for m in RE_LET_MUT.finditer(cuerpo)
        if m.group(1) in usados and valor_de(m.group(2), valores) is not None
    )


# Un uso de `result` que no es un indice: `result.iter_mut()`, `result.fill(..)`,
# `result.copy_from_slice(..)`. RE_WRITE no lo ve, ni siquiera como hueco.
RE_RESULT_SUELTO = re.compile(r"\bresult\b(?!\s*\[)")
RE_DELEGACION = re.compile(r"\.evaluate_(?:aux_)?transition\s*\(")


def usos_sin_indice(texto):
    """Los usos de `result` sin indice dentro de las funciones de evaluacion.

    ⚠️ Cada uno es un HUECO: una escritura por `result.iter_mut().skip(k)`
    puede anular una ranura que otro sitio escribe, y el barrido, que solo
    lee `result[..]`, no la veia ni como hueco (medido: una pisada asi sobre
    la cuenta de la edad salia con exit 0). Se exceptua la delegacion
    ENTERA a otra AIR (`self.0.evaluate_transition(.., result)`), la forma
    de los dobles de prueba: las ranuras de la AIR delegada las barre su
    propio fichero.
    """
    salida = []
    for nombre_fn in ("evaluate_transition", "evaluate_aux_transition"):
        busca = 0
        while True:
            k = texto.find("fn " + nombre_fn + "<", busca)
            k2 = texto.find("fn " + nombre_fn + "(", busca)
            k = min(x for x in (k, k2, len(texto)) if x >= 0)
            if k >= len(texto):
                break
            cp = _cuerpo_de_funcion(texto[k:], nombre_fn)
            if cp is None:
                break
            cuerpo, desp = cp
            for u in RE_RESULT_SUELTO.finditer(cuerpo):
                antes = cuerpo[:u.start()]
                inicio = max(antes.rfind(";"), antes.rfind("{"), antes.rfind("}"))
                llamadas = list(RE_DELEGACION.finditer(antes, inicio + 1))
                # `result` entero, como ULTIMO argumento de la llamada, no dentro
                # de otra expresion: `x.evaluate_transition(f, p, g(result))` no vale.
                if llamadas:
                    args = antes[llamadas[-1].end():]
                    if (args.count("(") == args.count(")")
                            and re.match(r"\s*\)", cuerpo[u.end():])):
                        continue
                linea = texto[: k + desp + u.start()].count("\n") + 1
                fin = cuerpo.find(";", u.start())
                trozo = " ".join(cuerpo[u.start():fin if fin > 0 else u.end() + 40].split())
                salida.append((linea, "uso de `result` sin indice: " + trozo[:60]))
            busca = k + desp + len(cuerpo)
    return salida


def analizar(ruta, del_crate=None):
    texto = sin_comentarios(open(ruta, encoding="utf-8").read())
    valores, crudos, sin_resolver = resolver_simbolos(texto, del_crate)
    grupos = {n: valores[n] for n in crudos if n.startswith("C_") and n in valores}
    # El total sale de una constante DEL FICHERO, nunca de la raiz del crate:
    # la banda no tiene las 110 ranuras de la edad.
    total = next((valores[n] for n in TOTALES if n in crudos and n in valores), None)
    total_aux = next((valores[n] for n in TOTALES_AUX if n in crudos and n in valores), None)
    if not RE_WRITE.search(texto):
        return None
    # Un grupo es una constante `C_` con la que se indexa `result`. En la edad
    # las `C_` son COLUMNAS (C_A, C_P, ...) y las ranuras van por contador:
    # rotular «ranura 36 (C_P+0)» señalaria una columna como si fuera grupo.
    en_indices = " ".join(m.group(1) for m in RE_WRITE.finditer(texto))
    if not re.search(r"\bC_[A-Z_0-9]*\b", en_indices):
        grupos = {}

    # §534: las escrituras dentro de `evaluate_aux_transition` van al `result` del tramo
    # auxiliar, OTRO espacio de ranuras (winterfell le pasa su propio `result`): no colisionan
    # con las del principal, y entre si se miran igual. Hasta el §534 ningun circuito del
    # barrido tenia tramo auxiliar (los de la casa viven en zk-ssl-air, fuera de RAIZ); el
    # juguete del censo de `falsadores_oculta.rs` lo tiene, y el barrido le pintaba dos
    # colisiones que no eran: la ranura 0 y la 1 del principal contra las del auxiliar.
    aux = _cuerpo_de_funcion(texto, "evaluate_aux_transition")
    aux_ini, aux_fin = (aux[1], aux[1] + len(aux[0])) if aux else (-1, -1)

    # Las funciones con contador secuencial (la edad) se ejecutan en orden;
    # sus sitios salen del interprete, con repeticiones.
    interpretadas = {}
    for nombre_fn in ("evaluate_transition", "evaluate_aux_transition"):
        cp = _cuerpo_de_funcion(texto, nombre_fn)
        if cp is None:
            continue
        cuerpo, desplazamiento = cp
        contadores = contadores_de(cuerpo, valores)
        if not contadores:
            continue
        r = interpretar_contadores(cuerpo, valores, contadores, texto)
        if r is None:
            continue  # sus sitios quedan como huecos, no como aprobados
        for pos, idx in r.items():
            interpretadas[desplazamiento + pos] = idx

    escrituras, indeterminadas = [], []
    for m in RE_WRITE.finditer(texto):
        if m.start() in interpretadas:
            idx = interpretadas[m.start()]
        else:
            # Con repeticiones: un sitio en un bucle cuyo indice no depende de
            # la variable del bucle, o una tupla repetida, escribe la misma
            # ranura dos veces. Como conjunto, eso no se veia.
            idx = _valores_de_indice(texto, m.start(), m.group(1), valores)
        linea = texto[: m.start()].count("\n") + 1
        if idx is None:
            indeterminadas.append((linea, m.group(0)))
        else:
            escrituras.append((linea, idx, aux_ini <= m.start() < aux_fin))
    indeterminadas.extend(usos_sin_indice(texto))

    # ⚠️ Una ranura ya escrita es COLISION aunque la vuelva a escribir el
    # mismo sitio: con un contador, un `i += 1` olvidado en un bucle escribe
    # la misma ranura desde la misma linea. Los sitios sin contador no
    # repiten indice, asi que para ellos nada cambia.
    duenos, colisiones = {}, {}
    duenos_aux, colisiones_aux = {}, {}
    for linea, idx, en_aux in escrituras:
        d, c = (duenos_aux, colisiones_aux) if en_aux else (duenos, colisiones)
        for i in idx:
            if i in d:
                if i not in c:
                    c[i] = [d[i]]
                c[i].append(linea)
            else:
                d[i] = linea

    cubiertas = set(duenos)
    desbordes = sorted(i for i in cubiertas if total is not None and i >= total)
    # Sin `NUM_CONSTRAINTS` no se puede saber cuantas ranuras deberia haber,
    # asi que no se puede hablar de ranuras muertas. Las COLISIONES —que son
    # la firma de §38— si se detectan igual.
    muertas = sorted(set(range(total)) - cubiertas) if total is not None else []
    # Y lo mismo en el tramo auxiliar, contra su propio total.
    cubiertas_aux = set(duenos_aux)
    desbordes_aux = sorted(i for i in cubiertas_aux if total_aux is not None and i >= total_aux)
    muertas_aux = sorted(set(range(total_aux)) - cubiertas_aux) if total_aux is not None else []

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

    if p_construidas == DELEGADA:
        # Sin cadena propia no hay nada que cruzar; leerla aqui SI seria un hueco.
        p_desbordes, p_muertas = [], []
        p_construidas = 0 if not (p_leidas or p_indeterminadas) else None
    if p_construidas is None:
        p_desbordes, p_muertas = [], []
    else:
        p_desbordes = sorted(i for i in p_leidas if i >= p_construidas)
        p_muertas = sorted(set(range(p_construidas)) - p_leidas)

    return {
        "total": total,
        "total_aux": total_aux,
        "desbordes_aux": desbordes_aux,
        "muertas_aux": muertas_aux,
        "colisiones": colisiones,
        "colisiones_aux": colisiones_aux,
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

# ===== EL CENSO DE CELDAS-CLASE (FV-1; §188-§189) =====
#
# La Capa 1-2 (§188-§189) midio que censo y redes clasicas son
# COMPLEMENTARIOS, en los dos sentidos: borrar C_SEG_LINK deja el censo en
# delta 0 (Horner co-posee la col 37 en las clases de cierre) y son las
# redes de ESTE barrido quienes lo ven (MUERTA + MUERTA PERIODICA, graves
# desde §189); cambiar el SELECTOR de una restriccion (cont_s→first_s en
# Horner) deja todas las ranuras escritas y todas las periodicas leidas
# —redes ciegas— y es el censo quien grita: 5 celdas-clase sin dueño
# (§189, medido). Complementarias por MEDICION, no por diseño: por eso el
# censo entra aqui como compuerta y no como evidencia (doc §9, Capa 2).
#
# El interprete vive en doc/fv/ —donde §188 lo cita— y se ejecuta tal cual:
# caza sus DOS mutantes en cada corrida («mutantes 2/2» en su COMPUERTA),
# asi que el caso-mutacion viene sembrado de fabrica y aqui SE EXIGE. Un
# interprete puede llevar mas (la edad del kit lleva tres: «mutantes 3/3»);
# se exigen todos, y nunca menos de dos.
CENSO_RUTA = os.path.join(
    os.path.dirname(__file__), "..", "doc", "fv", "interprete_multiciclo.py"
)
RE_COMPUERTA = re.compile(
    r"^COMPUERTA: (\d+) clases \u00b7 (\d+) celdas-clase \u00b7 (\d+) libres "
    r"declaradas \u00b7 (\d+) sin due\u00f1o \u00b7 mutantes (\d+)/(\d+) \u00b7 "
    r"C_SEG_LINK\u2192guardi\u00e1n$"
)
CENSO_CREDIT_RUTA = os.path.join(
    os.path.dirname(__file__), "..", "doc", "fv", "interprete_credit_climb.py"
)
RE_COMPUERTA_CREDIT = re.compile(
    r"^COMPUERTA-CREDIT: (\d+) clases \u00b7 (\d+) celdas-clase \u00b7 (\d+) libres "
    r"declaradas \u00b7 (\d+) sin due\u00f1o \u00b7 mutantes (\d+)/(\d+) \u00b7 "
    r"C_SEG_LINK\u2192guardi\u00e1n$"
)
CENSO_MINT_PAGOS_RUTA = os.path.join(
    os.path.dirname(__file__), "..", "doc", "fv", "interprete_mint.py"
)
RE_COMPUERTA_MINT = re.compile(
    r"^COMPUERTA-MINT: (\d+) clases \u00b7 (\d+) celdas-clase \u00b7 (\d+) libres "
    r"declaradas \u00b7 (\d+) sin due\u00f1o \u00b7 mutantes (\d+)/(\d+) \u00b7 "
    r"C_SEG_LINK\u2192guardi\u00e1n$"
)
CENSO_RECOVERY_RUTA = os.path.join(
    os.path.dirname(__file__), "..", "doc", "fv", "interprete_recovery_climb.py"
)
RE_COMPUERTA_RECOVERY = re.compile(
    r"^COMPUERTA-RECOVERY: (\d+) clases \u00b7 (\d+) celdas-clase \u00b7 (\d+) libres "
    r"declaradas \u00b7 (\d+) sin due\u00f1o \u00b7 mutantes (\d+)/(\d+) \u00b7 "
    r"C_SEG_LINK\u2192guardi\u00e1n$"
)
CENSO_SEND_RUTA = os.path.join(
    os.path.dirname(__file__), "..", "doc", "fv", "interprete_send.py"
)
RE_COMPUERTA_SEND = re.compile(
    r"^COMPUERTA-SEND: (\d+) clases \u00b7 (\d+) celdas-clase \u00b7 (\d+) libres "
    r"declaradas \u00b7 (\d+) sin due\u00f1o \u00b7 mutantes (\d+)/(\d+) \u00b7 "
    r"C_SEG_LINK\u2192guardi\u00e1n$"
)
CENSO_CLAIM_RUTA = os.path.join(
    os.path.dirname(__file__), "..", "doc", "fv", "interprete_claim.py"
)
RE_COMPUERTA_CLAIM = re.compile(
    r"^COMPUERTA-CLAIM: (\d+) clases \u00b7 (\d+) celdas-clase \u00b7 (\d+) libres "
    r"declaradas \u00b7 (\d+) sin due\u00f1o \u00b7 mutantes (\d+)/(\d+) \u00b7 "
    r"C_SEG_LINK\u2192guardi\u00e1n$"
)


def _censo_fv1(sujeto, ruta, prefijo, regex):
    """Nucleo compartido de los censos FV-1 (§189/§191): corre el interprete
    del `sujeto` y devuelve (linea_limpia, faltas). Toda falta es GRAVE,
    incluida la ausencia del interprete: una compuerta que se salta cuando
    su herramienta no esta es el falso verde de §59.2."""
    if not os.path.exists(ruta):
        return None, [
            f"[CENSO] falta {os.path.basename(ruta)}: la compuerta "
            "no puede correr, y eso es GRAVE, no un hueco (§59.2)"
        ]
    p = subprocess.run(
        [sys.executable, ruta], capture_output=True, text=True
    )
    linea = next(
        (l for l in reversed(p.stdout.splitlines())
         if l.startswith(prefijo)),
        None,
    )
    faltas = []
    if p.returncode != 0:
        faltas.append(f"[CENSO] el interprete salio con codigo {p.returncode}")
    if linea is None:
        faltas.append("[CENSO] sin linea COMPUERTA en la salida del interprete")
    else:
        m = regex.match(linea)
        if m is None:
            faltas.append(f"[CENSO] COMPUERTA con forma inesperada: {linea}")
        else:
            clases, celdas, libres, huerf, mut, total = m.groups()
            if huerf != "0":
                faltas.append(
                    f"[CENSO] {huerf} celda(s)-clase SIN DUE\u00d1O en {sujeto}"
                )
            # Todos los mutantes cazados, y al menos los dos de siempre: un
            # interprete que pierde un mutante no puede salir verde por contar
            # menos.
            if mut != total or int(total) < 2:
                faltas.append(f"[CENSO] el interprete cazo {mut}/{total} mutantes")
            if not faltas:
                return (
                    f"Censo {sujeto}: {celdas} celdas-clase en {clases} "
                    f"clases \u00b7 {libres} libres declaradas \u00b7 0 sin "
                    f"due\u00f1o \u00b7 mutantes {mut}/{total}."
                ), []
    return None, faltas


def censo_mint_climb():
    return _censo_fv1("mint_climb", CENSO_RUTA, "COMPUERTA:", RE_COMPUERTA)


def censo_credit_climb():
    return _censo_fv1("credit_climb", CENSO_CREDIT_RUTA,
                      "COMPUERTA-CREDIT:", RE_COMPUERTA_CREDIT)


def censo_recovery_climb():
    return _censo_fv1("recovery_climb", CENSO_RECOVERY_RUTA,
                      "COMPUERTA-RECOVERY:", RE_COMPUERTA_RECOVERY)


def censo_mint_pagos():
    return _censo_fv1("mint (pagos)", CENSO_MINT_PAGOS_RUTA,
                      "COMPUERTA-MINT:", RE_COMPUERTA_MINT)


def censo_send():
    return _censo_fv1("send", CENSO_SEND_RUTA,
                      "COMPUERTA-SEND:", RE_COMPUERTA_SEND)


def censo_claim():
    return _censo_fv1("claim", CENSO_CLAIM_RUTA,
                      "COMPUERTA-CLAIM:", RE_COMPUERTA_CLAIM)


CENSOS_STARK = (
    ("circuit_mint_climb.rs", censo_mint_climb),
    ("circuit_credit_climb.rs", censo_credit_climb),
    ("circuit_recovery_climb.rs", censo_recovery_climb),
    ("circuit_mint.rs", censo_mint_pagos),
    ("circuit_send.rs", censo_send),
    ("circuit_claim.rs", censo_claim),
)


# ===== LOS CENSOS DEL KIT (zk-ssl-air) =====
#
# Las cinco AIR del kit, con el mismo nucleo (`doc/fv/censo_nucleo.py`) y
# sus formas propias (ESPEC["KIT"]). La banda, el cobro pendiente y el pago
# en curso tienen segmentos de rango, y su candidato C_SEG_LINK lo para este
# guardian; la prenda y la edad no tienen segmentos. La edad es ciclica y
# tiene tramo auxiliar: su censo cuenta tambien la fila ACTUAL, solo bajo
# selector, y los dueños que declara el multiconjunto
# (doc/fv/interprete_edad.py). Su tercer mutante cambia el selector de la
# constancia de vivo (`hf` por `enlace`): ni el barrido ni la vacuidad lo
# ven, porque la ranura sigue escrita y la restriccion sigue reaccionando.
def _re_compuerta(rotulo, cola):
    return re.compile(
        r"^COMPUERTA-%s: (\d+) clases \u00b7 (\d+) celdas-clase \u00b7 (\d+) libres "
        r"declaradas \u00b7 (\d+) sin due\u00f1o \u00b7 mutantes (\d+)/(\d+) \u00b7 "
        r"C_SEG_LINK\u2192%s$" % (rotulo, cola)
    )


def _censo_kit(sujeto, interprete, rotulo, cola):
    ruta = os.path.join(os.path.dirname(__file__), "..", "doc", "fv", interprete)
    return lambda: _censo_fv1(sujeto, ruta, "COMPUERTA-%s:" % rotulo,
                              _re_compuerta(rotulo, cola))


CENSOS_KIT = (
    ("banda.rs", _censo_kit("banda", "interprete_banda.py", "BANDA", "guardi\u00e1n")),
    ("cobro_pendiente.rs", _censo_kit("cobro pendiente", "interprete_cobro_pendiente.py",
                                      "COBRO", "guardi\u00e1n")),
    ("pago_en_curso.rs", _censo_kit("pago en curso", "interprete_pago_en_curso.py",
                                    "PAGO", "guardi\u00e1n")),
    ("prenda.rs", _censo_kit("prenda", "interprete_prenda.py", "PRENDA", "sin segmento")),
    ("lib.rs (edad)", _censo_kit("edad", "interprete_edad.py", "EDAD", "sin segmento")),
)


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


CASO_188 = """
const C_A: usize = 0;
const C_B: usize = 1;
const NUM_CONSTRAINTS: usize = 2;
const P_UNO: usize = 0;
const P_LINK: usize = 1;

fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
    let mut columns = Vec::new();
    columns.push(uno);
    columns.push(link);
    columns
}

fn evaluate_transition(&self) {
    let a = periodic[P_UNO];
    result[C_A] = a;
}
"""


CASO_534 = """
const C_A: usize = 0;
const C_B: usize = 1;
const NUM_CONSTRAINTS: usize = 2;

fn evaluate_transition(&self) {
    result[C_A] = a;
    result[C_B] = b;
}

fn evaluate_aux_transition(&self) {
    result[0] = c;
    result[1] = d;
    result[1] = e;
}
"""


# Las formas del kit (zk-ssl-air), en pequeño: un contador secuencial con un
# corte (la edad), una tupla de carriles y una tabla de transporte constante
# (el cobro pendiente, el pago en curso y la prenda), y la lista periodica con
# otro nombre. Sano: 2 + 4 + 2 + 2 = 10 ranuras, cada una una vez.
CASO_KIT = """
const ESTADO: usize = 2;
const C_CAP_A: usize = 4;
const C_CAP_B: usize = 6;
const C_TRANSPORT: usize = 8;
const NUM_RESTRICCIONES: usize = 10;
const AUXILIARES: usize = 1;
const TRANSPORTE: [usize; 2] = [
    COL_UNO,
    COL_DOS,
];
const P_UNO: usize = 0;
const P_DOS: usize = 1;

fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
    let mut cols = Vec::new();
    let mut v = Vec::new();
    v.push(uno);
    cols.push(v);
    for k in [true, false] {
        cols.push(otra);
    }
    cols
}

fn evaluate_transition(&self) {
    let mut i = 0;
    ronda(&mut result[i..i + ESTADO]);
    i += ESTADO;
    for k in 0..2 {
        result[i] = periodic[P_UNO];
        i += 1;
    }
    for (c_cap, base) in [(C_CAP_A, 0), (C_CAP_B, 12)] {
        for j in 0..2 {
            result[c_cap + j] = periodic[P_DOS + j] * siguiente[base + j];
        }
    }
    for (k, col) in TRANSPORTE.iter().enumerate() {
        result[C_TRANSPORT + k] = siguiente[*col] - actual[*col];
    }
    let _ = i;
}

fn evaluate_aux_transition(&self) {
    result[0] = suma;
}
"""


def _barrer_caso(texto, huecos_graves=False):
    """Barre un circuito sembrado, sin censo; devuelve (rc, salida)."""
    import io
    import tempfile
    from contextlib import redirect_stdout

    d = tempfile.mkdtemp()
    ruta = os.path.join(d, "caso.rs")
    with open(ruta, "w") as f:
        f.write(texto)
    buf = io.StringIO()
    try:
        with redirect_stdout(buf):
            rc = barrer(d, verbose=False, con_censo=False, huecos_graves=huecos_graves)
    finally:
        os.unlink(ruta)
        os.rmdir(d)
    return rc, buf.getvalue()


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

    # ===== Y que la firma del candidato ciego de §188 TUMBA (§189) =====
    #
    # `CASO_188` reproduce lo que deja borrar `C_SEG_LINK` en mint_climb:
    # una ranura declarada que nadie escribe y una periodica construida
    # que nadie lee. Con la politica vieja ambas eran huecos y el exit
    # era 0: el candidato salia limpio. Este caso pide el barrido ENTERO
    # (no `analizar`) para que una regresion de la politica no pueda
    # pasar en silencio.
    import io
    from contextlib import redirect_stdout

    d = tempfile.mkdtemp()
    ruta = os.path.join(d, "caso_188.rs")
    with open(ruta, "w") as f:
        f.write(CASO_188)
    try:
        with redirect_stdout(io.StringIO()):
            rc = barrer(d, verbose=False, con_censo=False)
    finally:
        os.unlink(ruta)
        os.rmdir(d)
    if rc != 1:
        print(f"AUTOTEST FALLA: la firma del candidato ciego salio "
              f"con {rc}, no con 1")
        return 1
    print("autotest: la firma del candidato ciego de §188 es GRAVE "
          "y tumba el exit")

    # §534: el tramo auxiliar es otro espacio de ranuras. Las ranuras 0 y 1 se escriben en el
    # principal y en el auxiliar y NO colisionan; la 1 del auxiliar, escrita dos veces, SI.
    with tempfile.NamedTemporaryFile("w", suffix=".rs", delete=False) as f:
        f.write(CASO_534)
        ruta = f.name
    try:
        r = analizar(ruta)
    finally:
        os.unlink(ruta)
    if r["colisiones"] or sorted(r["colisiones_aux"]) != [1]:
        print(f"AUTOTEST FALLA: el tramo auxiliar tenia que ser otro espacio; "
              f"principal {sorted(r['colisiones'])}, auxiliar {sorted(r['colisiones_aux'])}")
        return 1
    print("autotest: el tramo auxiliar es otro espacio de ranuras (§534), "
          "y sus colisiones se ven igual")

    # ===== Las formas del kit: limpias, sin huecos, y cada fallo visto =====
    #
    # Sin estas, el barrido del kit podria salir verde por no entender: los
    # 45 puntos que no expandia eran exactamente estas formas.
    rc, salida = _barrer_caso(CASO_KIT)
    # Sin `--verbose`, el barrido solo nombra el fichero si tiene algo que decir.
    if rc != 0 or "caso.rs" in salida or "NO ha comprobado" in salida:
        print(f"AUTOTEST FALLA: el caso sano del kit no sale limpio (rc {rc}):\n{salida}")
        return 1
    mutantes_kit = (
        # (que se rompe, de, a, lo que tiene que gritar)
        ("el i += 1 olvidado en el bucle del contador",
         "        result[i] = periodic[P_UNO];\n        i += 1;\n",
         "        result[i] = periodic[P_UNO];\n", "(el mismo sitio, mas de una vez)"),
        ("el contador que no avanza tras el corte",
         "    i += ESTADO;\n", "", "[COLISION]"),
        ("la tupla del carril B con la capacidad del A",
         "(C_CAP_B, 12)", "(C_CAP_A, 12)", "[COLISION]"),
        ("una fila de la tabla de transporte de menos",
         "    COL_UNO,\n", "", "[MUERTA]"),
        ("la ranura auxiliar sin escribir",
         "    result[0] = suma;\n", "    let _ = suma;\n", "[MUERTA AUX]"),
        ("una periodica de mas en la lista devuelta",
         "    cols\n}", "    cols.push(tercera);\n    cols\n}", "[MUERTA PERIODICA]"),
    )
    for que, de, a, grito in mutantes_kit:
        if CASO_KIT.count(de) != 1:
            print(f"AUTOTEST FALLA: el mutante «{que}» no casa una sola vez")
            return 1
        rc, salida = _barrer_caso(CASO_KIT.replace(de, a))
        if rc != 1 or grito not in salida:
            print(f"AUTOTEST FALLA: el barrido no ve «{que}» (rc {rc}):\n{salida}")
            return 1
    print(f"autotest: las formas del kit salen limpias y sin huecos, y el barrido "
          f"ve sus {len(mutantes_kit)} mutantes (contador, corte, tupla, tabla, "
          f"tramo auxiliar, lista periodica)")

    # ===== Un hueco tambien tumba el exit donde la raiz esta en 0 huecos =====
    #
    # Una pisada en una forma que el barrido no expande no da COLISION: sale
    # como hueco. Sin `huecos_graves`, el exit era 0 (medido sobre una copia
    # de la banda). Tres formas: un bucle que el barrido no sabe recorrer, un
    # alias de un corte de `result` y un iterador sobre `result`, que antes
    # ni siquiera era hueco. Las tres pisan una ranura que otro sitio escribe.
    sin_aux = CASO_534.split("fn evaluate_aux_transition")[0]
    pisadas = (
        # (la forma, el caso sano, de, a)
        ("un bucle al reves", sin_aux, "    result[C_B] = b;\n",
         "    result[C_B] = b;\n    for j in (0..1).rev() {\n"
         "        result[C_A + j] = cero;\n    }\n"),
        ("un alias del corte", CASO_KIT, "    let _ = i;\n}",
         "    let r = &mut result[C_CAP_A..];\n    r[0] = cero;\n    let _ = i;\n}"),
        ("un iterador sobre result", sin_aux, "    result[C_B] = b;\n",
         "    result[C_B] = b;\n    result.iter_mut().skip(1).for_each(|x| *x = cero);\n"),
    )
    for que, sano, de, a in pisadas:
        # El caso sano sale limpio CON huecos_graves: el hueco es la pisada.
        rc_sano, salida = _barrer_caso(sano, huecos_graves=True)
        if sano.count(de) != 1 or rc_sano != 0 or "NO ha comprobado" in salida:
            print(f"AUTOTEST FALLA: la base de «{que}» no sale limpia (rc {rc_sano}):\n{salida}")
            return 1
        caso = sano.replace(de, a)
        rc_blando, salida = _barrer_caso(caso)
        rc_duro, _ = _barrer_caso(caso, huecos_graves=True)
        if "NO ha comprobado" not in salida or "[COLISION]" in salida \
                or (rc_blando, rc_duro) != (0, 1):
            print(f"AUTOTEST FALLA: la pisada por «{que}» tenia que ser un hueco, "
                  f"con exit 0 sin huecos_graves y 1 con el; da {rc_blando} y "
                  f"{rc_duro}:\n{salida}")
            return 1
    print(f"autotest: una pisada que el barrido no expande es un hueco, y con "
          f"huecos_graves tumba el exit ({len(pisadas)} formas)")

    # ===== Y sobre la edad REAL: un `i += 1` de menos es colision =====
    ruta_edad = os.path.join(RAIZ_KIT, "lib.rs")
    real = open(ruta_edad, encoding="utf-8").read()
    de = "            i += 1;\n        }\n        i += 4;\n        let dominio"
    if real.count(de) != 1:
        print("AUTOTEST FALLA: el ancla del mutante de la edad no casa una sola vez "
              "(la edad cambio de forma: rehacer el mutante)")
        return 1
    import shutil
    import tempfile

    d = tempfile.mkdtemp()
    try:
        with open(os.path.join(d, "lib.rs"), "w", encoding="utf-8") as f:
            f.write(real.replace(de, "        }\n        i += 4;\n        let dominio"))
        r = analizar(os.path.join(d, "lib.rs"), constantes_del_crate(RAIZ_KIT))
    finally:
        shutil.rmtree(d)
    if not r["colisiones"] or r["indeterminadas"]:
        print(f"AUTOTEST FALLA: la edad sin su `i += 1` no da colision "
              f"({len(r['colisiones'])} colisiones, {len(r['indeterminadas'])} huecos)")
        return 1
    print(f"autotest: la edad real sin un `i += 1` da {len(r['colisiones'])} "
          f"ranuras en colision")
    return 0


def constantes_del_crate(raiz):
    """Las constantes de la raiz del crate (`lib.rs`), resueltas.

    El kit declara `CICLO`, `ESTADO`, `PROFUNDIDAD` y `RONDAS` en su raiz y
    cada AIR las importa con `use crate::{...}`. Sin ellas, la banda no sabia
    cuanto mide `result[C_HASH..C_HASH + ESTADO]`.
    """
    ruta = os.path.join(raiz, "lib.rs")
    if not os.path.exists(ruta):
        return {}
    texto = sin_comentarios(open(ruta, encoding="utf-8").read())
    valores, _, _ = resolver_simbolos(texto)
    return valores


def barrer(raiz, verbose, con_censo, del_crate=None, censos=None, rotulo="circuitos",
           huecos_graves=False):
    """Recorre `raiz`; devuelve 1 si hay GRAVES, 0 si no.

    `con_censo=False` existe para el autotest: permite barrer un
    directorio sembrado sin arrastrar el censo del arbol real. `censos` es
    la lista de censos FV-1 de esa raiz; por defecto, los de
    `stark-experiment`.

    `huecos_graves=True`: un hueco del barrido tambien tumba el exit. ⚠️ Sin
    eso, una pisada escrita en una forma que el barrido no sabe expandir
    (`for i in (0..1).rev()`, un alias de un corte de `result`) no da
    COLISION ni MUERTA: sale como hueco, y el exit era 0. Las dos raices del
    arbol estan hoy en 0 huecos (medido), y `main` lo fija: un hueco nuevo es
    un fallo hasta que el barrido aprenda su forma.
    """
    graves = huecos = barridos = 0

    no_barridos = []
    for fichero in sorted(os.listdir(raiz)):
        if not fichero.endswith(".rs"):
            continue
        ruta = os.path.join(raiz, fichero)
        r = analizar(ruta, del_crate)
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
                f"escrita en las lineas {sorted(set(sitios))}"
                + (" (el mismo sitio, mas de una vez)" if len(set(sitios)) == 1 else "")
            )
            graves += 1

        for i, sitios in sorted(r["colisiones_aux"].items()):
            lineas.append(
                f"    [COLISION AUX] ranura {i} del tramo auxiliar: "
                f"escrita en las lineas {sorted(set(sitios))}"
                + (" (el mismo sitio, mas de una vez)" if len(set(sitios)) == 1 else "")
            )
            graves += 1

        for i in r["desbordes_aux"]:
            lineas.append(
                f"    [DESBORDE AUX] ranura {i} del tramo auxiliar >= {r['total_aux']}"
            )
            graves += 1

        if r["muertas_aux"]:
            lineas.append(
                f"    [MUERTA AUX] {len(r['muertas_aux'])} ranura(s) del tramo "
                f"auxiliar sin escribir: {r['muertas_aux']}"
            )
            graves += 1

        for i in r["desbordes"]:
            lineas.append(
                f"    [DESBORDE] ranura {i} >= NUM_CONSTRAINTS={r['total']}"
            )
            graves += 1

        if r["muertas"]:
            # §189 — POLITICA: hallazgo POSITIVO medido = GRAVE;
            # incapacidad del barrido = hueco. Era hueco, y el
            # candidato ciego de §188 (borrar C_SEG_LINK) salia con
            # exit 0 apoyandose exactamente en eso.
            gm = sorted({grupo_de(i, r["grupos"]).split("+")[0] for i in r["muertas"]})
            lineas.append(
                f"    [MUERTA] {len(r['muertas'])} ranura(s) sin escribir "
                f"(grupos: {', '.join(gm)})"
            )
            graves += 1

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
            # §189 — la otra red que ve al candidato ciego: misma politica.
            lineas.append(
                f"    [MUERTA PERIODICA] {len(r['p_muertas'])} columna(s) que "
                f"se construyen y NADIE lee: {r['p_muertas']}"
            )
            graves += 1

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
            if r["total"] is None:
                # Sin total propio no hay MUERTAS ni DESBORDES que mirar: un doble de
                # prueba, o un circuito que no declara su `NUM_CONSTRAINTS`.
                print(f"\n  {fichero}: sin total propio: solo se miran colisiones, "
                      "y no hay ninguna")
            else:
                extra = (f" + {r['total_aux']} auxiliares"
                         if r["total_aux"] is not None else "")
                print(f"\n  {fichero}: {r['total']} ranuras{extra}, "
                      "cada una escrita una vez")

    linea_censo = None
    if con_censo:
        # ===== LOS CENSOS (FV-1, §189/§191): ven la celda que estas redes no ven =====
        partes = []
        for fichero_censo, fn_censo in (censos if censos is not None else CENSOS_STARK):
            linea_c, faltas_censo = fn_censo()
            if faltas_censo:
                print(f"\n  {fichero_censo} (censo FV-1)")
                for falta in faltas_censo:
                    print(f"    {falta}")
                graves += len(faltas_censo)
            else:
                partes.append(linea_c)
        if partes and not graves:
            linea_censo = " ".join(partes)

    print()
    if graves:
        print(
            f"{graves} problema(s) GRAVE(S) en {barridos} {rotulo}. Una COLISION "
            "es la firma de §38 y puede ser un fallo de solidez como §39, §50 o "
            "§50.7: se mira con un test discriminante, no por lectura."
        )
    if huecos:
        print(
            f"{huecos} punto(s) que este barrido NO ha comprobado. No son "
            "aprobados: son huecos del propio barrido, y hay que mirarlos a mano."
            + (" Aqui son GRAVES: esta raiz esta fijada en 0 huecos." if huecos_graves else "")
        )
    if not graves and not huecos:
        censo_txt = f" {linea_censo}" if linea_censo else ""
        print(
            f"{barridos} {rotulo}: ninguna ranura colisiona, desborda ni queda "
            "muerta, y ninguna columna periodica se lee fuera de rango ni se "
            "construye sin leerse." + censo_txt
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
    return 1 if graves or (huecos_graves and huecos) else 0


def main():
    if "--autotest" in sys.argv:
        return autotest()
    # El autotest corre siempre antes del barrido: cuesta una decima de
    # segundo, y sin el una regresion del propio barrido (una forma que deja
    # de entender, un mutante que deja de ver) pasaria en silencio. Solo se
    # imprime si falla.
    import io
    from contextlib import redirect_stdout

    buf = io.StringIO()
    with redirect_stdout(buf):
        rc_auto = autotest()
    if rc_auto != 0:
        print(buf.getvalue(), end="")
        print("El autotest del barrido FALLA: su veredicto sobre el arbol no vale.")
        return 1
    verbose = "--verbose" in sys.argv
    rc = barrer(RAIZ, verbose, con_censo=True, huecos_graves=True)
    # Las AIR del kit: las que juzgan las pruebas de los hitos (banda y edad;
    # cobro pendiente, pago en curso y prenda), y las que la segunda
    # implementacion transcribe. Hasta aqui tenian menos guardias que los
    # circuitos de `stark-experiment`: el barrido no entraba en su crate.
    print()
    print("== el kit (zk-ssl-air) ==")
    rc_kit = barrer(RAIZ_KIT, verbose, con_censo=True,
                    del_crate=constantes_del_crate(RAIZ_KIT),
                    censos=CENSOS_KIT, rotulo="AIR del kit", huecos_graves=True)
    return 1 if (rc or rc_kit) else 0


if __name__ == "__main__":
    sys.exit(main())
