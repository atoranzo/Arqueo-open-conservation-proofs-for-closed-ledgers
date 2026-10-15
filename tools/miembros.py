#!/usr/bin/env python3
"""Los miembros del workspace: el UNICO lector de `members` en `[workspace]` (§833).

## Por que existe

Hasta el §833 habia dos lectores, y ninguno leia solo el array:

- `miembros_del_workspace` de `tools/canon.sh` cogia cada `"crates/<x>"` desde
  `[workspace]` hasta la primera tabla que no empezara por `[w`. Asi entraban un
  `exclude`, un `path` de `[workspace.dependencies]` y un miembro comentado.
- `miembros_del_workspace` del test `las_fijaciones_son_exactas`
  (`crates/zk-ssl-verify/src/lib.rs`) cogia las lineas que empiezan por
  `"crates/` dentro de `[workspace]`. Con `members` en una sola linea veia cero.

Cada uno se equivocaba en un sitio distinto. Ahora el canon y `check_cifras`
leen de aqui, y el test del kit ya no lee `members`: recorre `crates/*/`. Lo
ata `--con-crates`, que el canon pasa: cada miembro es un `crates/<x>` con
`Cargo.toml`, y cada `crates/<x>` con `Cargo.toml` es miembro. Si no, exit 1.

## Que lee, exactamente

La clave `members` de la tabla `[workspace]` del `Cargo.toml` raiz, y nada mas:
su array de cadenas, en una linea o en varias, con comentarios `#` y coma final.
Cada cadena es la ruta del miembro tal como la escribe el manifiesto
(`crates/zk-core`).

Falla CERRADA (exit 2, con la causa) ante lo que no sabe leer: sin `[workspace]`,
sin `members`, dos `members`, un array sin cerrar, algo que no es una cadena, una
cadena con `\\`, un comodin (`*`, `?`, `[`) o un miembro repetido. No entiende
TOML entero, y no lo finge: lo que se salga de esa forma no pasa callando.

## Lo que NO lee

Los miembros que cargo anade por su cuenta. Una dependencia por `path` dentro del
directorio del workspace es miembro aunque `members` no la nombre: con
`"crates/ceremony"` comentado, `cargo metadata --no-deps` sigue dando 22, porque
`zk-core` la pide por ruta. Este lector da 21, y el canon se pone ROJO por la fila
de `ceremony`. Es a proposito: un miembro del canon se declara.

## Uso

    python3 tools/miembros.py                # una ruta por linea, en el orden de Cargo.toml
    python3 tools/miembros.py --con-crates   # lo mismo, y exit 1 si crates/ dice otra cosa
"""

import os
import re
import sys

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

CABECERA = re.compile(r"^\[\s*([^\[\]]*?)\s*\]$")
CLAVE_MEMBERS = re.compile(r"^members\s*=")
COMODINES = set("*?[")


class Ilegible(ValueError):
    """`members` no tiene la forma que este lector sabe leer."""


def _sin_comentario(linea):
    """La linea sin su comentario `#`. Fuera de una cadena; dentro, no se corta."""
    dentro = None
    for i, c in enumerate(linea):
        if dentro:
            if c == dentro:
                dentro = None
        elif c in "\"'":
            dentro = c
        elif c == "#":
            return linea[:i]
    return linea


def _array(texto, n):
    """Lee el array que empieza en `texto` (justo tras el `=`). Devuelve sus cadenas."""
    i, fuera, espera_valor = 0, [], True
    texto = texto.lstrip()
    if not texto.startswith("["):
        raise Ilegible(f"Cargo.toml:{n}: `members` no es un array")
    i = 1
    while i < len(texto):
        c = texto[i]
        if c in " \t\r\n":
            i += 1
        elif c == "#":
            fin = texto.find("\n", i)
            i = len(texto) if fin < 0 else fin
        elif c == "]":
            resto = _sin_comentario(texto[i + 1:].split("\n", 1)[0]).strip()
            if resto:
                raise Ilegible(f"Cargo.toml: algo tras el `]` de `members`: {resto!r}")
            return fuera
        elif c == ",":
            if espera_valor:
                raise Ilegible("Cargo.toml: una coma sin valor en `members`")
            espera_valor = True
            i += 1
        elif c in "\"'":
            if not espera_valor:
                raise Ilegible("Cargo.toml: dos valores sin coma en `members`")
            fin = texto.find(c, i + 1)
            if fin < 0 or "\n" in texto[i + 1:fin]:
                raise Ilegible("Cargo.toml: una cadena sin cerrar en `members`")
            valor = texto[i + 1:fin]
            if "\\" in valor:
                raise Ilegible(f"Cargo.toml: `members` con un escape: {valor!r}")
            if COMODINES & set(valor):
                raise Ilegible(f"Cargo.toml: `members` con un comodin: {valor!r}")
            fuera.append(valor)
            espera_valor = False
            i = fin + 1
        else:
            raise Ilegible(f"Cargo.toml: en `members` hay algo que no es una cadena: {c!r}")
    raise Ilegible(f"Cargo.toml:{n}: el array de `members` no se cierra")


def miembros(texto):
    """Las rutas de `members` en `[workspace]`, en su orden. `Ilegible` si no puede."""
    lineas = texto.split("\n")
    tabla, hallados, vista_ws = None, [], False
    for n, linea in enumerate(lineas, 1):
        s = _sin_comentario(linea).strip()
        if s.startswith("["):
            m = CABECERA.match(s)
            tabla = m.group(1) if m and not s.startswith("[[") else s
            vista_ws = vista_ws or tabla == "workspace"
            continue
        if tabla == "workspace" and CLAVE_MEMBERS.match(s):
            resto = "\n".join([linea.split("=", 1)[1]] + lineas[n:])
            hallados.append(_array(resto, n))
    if not vista_ws:
        raise Ilegible("Cargo.toml: no hay tabla [workspace]")
    if not hallados:
        raise Ilegible("Cargo.toml: [workspace] no tiene `members`")
    if len(hallados) > 1:
        raise Ilegible("Cargo.toml: [workspace] tiene %d claves `members`" % len(hallados))
    fuera = hallados[0]
    repetidos = sorted({m for m in fuera if fuera.count(m) > 1})
    if repetidos:
        raise Ilegible(f"Cargo.toml: miembros repetidos en `members`: {repetidos}")
    return fuera


def de_la_raiz(raiz=RAIZ):
    with open(os.path.join(raiz, "Cargo.toml"), encoding="utf-8") as f:
        return miembros(f.read())


def discordes_con_crates(fuera, raiz=RAIZ):
    """Lo que `crates/` y `members` no dicen igual. Vacio es VERDE."""
    malos = []
    for m in fuera:
        if not (re.fullmatch(r"crates/[^/]+", m)
                and os.path.isfile(os.path.join(raiz, m, "Cargo.toml"))):
            malos.append(f"el miembro {m!r} no es un crates/<x> con Cargo.toml")
    base = os.path.join(raiz, "crates")
    for d in sorted(os.listdir(base)) if os.path.isdir(base) else []:
        if os.path.isfile(os.path.join(base, d, "Cargo.toml")) and f"crates/{d}" not in fuera:
            malos.append(f"crates/{d} tiene Cargo.toml y no esta en `members`")
    return malos


def main(args):
    if args not in ([], ["--con-crates"]):
        print("uso: python3 tools/miembros.py [--con-crates]", file=sys.stderr)
        return 2
    try:
        fuera = de_la_raiz()
    except (Ilegible, OSError) as e:
        print(f"ROJO: {e}", file=sys.stderr)
        return 2
    for m in fuera:
        print(m)
    malos = discordes_con_crates(fuera) if args else []
    for x in malos:
        print(f"ROJO: {x}", file=sys.stderr)
    return 1 if malos else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
