#!/usr/bin/env python3
"""La puerta de inmutabilidad de los vectores: lo publicado no se reescribe (§692).

## Por qué existe

`README.md` promete «vectores por versión que jamás se reescriben», y la regla 2 de
`spec/rfc/PROCESO.md` lo exige: si un cambio rompe el cable, la versión sube y los vectores
viejos se conservan bajo la suya. Hasta el §692 ninguna puerta lo comprobaba. El canon
comprueba que el árbol PASA sus vectores, no que los vectores sean los que se publicaron: un
vector reescrito en su sitio, con su línea de manifiesto al día, pasaba igual. Y ha pasado:
`zkssl-0.2.json` se re-emitió dos veces bajo su misma versión (§278 y §281; lo cuenta
`crates/zk-ssl-cli/src/conformance.rs`). Un tercero que guardó el vector de entonces no tenía
cómo saber que el de hoy es otro.

## La regla

`spec/vectors/HUELLAS.sha256` fija el sha256 de cada `spec/vectors/**/*.json` y de cada línea
de cada `MANIFIESTO.txt`. Contra él, declarado ANTES de mirar:

  R1  un fichero registrado sigue en su ruta con los mismos bytes;
  R2  o se MUEVE, con los mismos bytes, a un directorio de versión: de `spec/vectors/X` a
      `spec/vectors/<versión>/X`, con `<versión>` como `0.4`. Es lo que hizo el §682 con dos
      vectores de completitud, y lo que hace un cambio que rompe el cable (regla 2). Al
      moverse deja en las huellas su ruta vieja como LÁPIDA, que no se borra: un `.json` que
      vuelva a esa ruta es ROJO, con los mismos bytes o con otros, en ese sello o en
      cualquiera de los siguientes. Si no, mover un vector liberaría su nombre, y en el sello
      siguiente otro fichero entraría con él: un tercero que guardó el de entonces
      encontraría otro con el mismo nombre;
  R3  cada línea registrada de un manifiesto sigue en él, en cualquier posición —se pueden
      intercalar líneas nuevas, y el §682 lo hizo—, o se mueve idéntica al manifiesto del
      mismo sitio bajo un directorio de versión, y SOLO CON SU VECTOR: la línea de un vector
      se mueve si ese vector se movió a la misma versión, en el mismo sello o en uno anterior
      (su lápida lo dice); un comentario o una línea en blanco, si se movió alguno de los
      vectores de ese directorio. Una línea cuyo vector no existe —la de `no-existe.json`—
      no se mueve. Si no, mover una línea sola y escribir otra con otro código cambiaría en su
      sitio el veredicto de un vector que no se ha movido;
  R4  lo nuevo, fichero o línea, entra registrado EN EL MISMO SELLO: sin huella es ROJO hasta
      `--registrar`. Si no, un vector podría nacer en un sello y reescribirse en el siguiente
      sin que nadie lo hubiera fijado nunca;
  R5  bajo `spec/vectors/` no vive nada más que `.json`, `MANIFIESTO.txt` y este fichero de
      huellas: lo que la puerta no sabe juzgar no entra callado;
  R6  las huellas de `HEAD` siguen en las del árbol (R1 a R3 aplicadas al propio fichero de
      huellas, y sus lápidas, todas): reescribir a mano la huella de un vector, borrar una
      lápida, o borrar el fichero y registrarlo de cero, es ROJO antes del sello. Sin git —una
      copia para ensayar— se dice y se salta;
  R7  el fichero de huellas está en su forma canónica: ordenado, una cabecera, un formato.

`--registrar` añade las huellas de lo nuevo, lleva la de un vector movido a su ruta nueva y
deja su lápida en la vieja. Nunca sobrescribe ni borra una huella ni una lápida: si hay un ROJO
que no sea R4 o R7, no escribe nada.

## El formato

    <sha256>  <ruta relativa a spec/vectors>             un vector .json
    <sha256>  <ruta del manifiesto>#linea                UNA línea, sin su salto de línea
    <sha256>  <ruta vieja de un vector>#movido           la lápida: la huella que tenía allí

Una entrada por línea de manifiesto, repetida si la línea se repite; ordenado por ruta y por
huella. Las rutas de fichero son las de `sha256sum`: un tercero comprueba los vectores que
tiene sin Python, con `cd spec/vectors && sha256sum -c --ignore-missing HUELLAS.sha256` (las
entradas `#linea` y `#movido` no son ficheros, y `--ignore-missing` las salta).

## Uso

    python3 tools/check_vectores.py                 # exit 0 VERDE, 1 ROJO
    python3 tools/check_vectores.py --registrar     # añade lo nuevo; no sobrescribe nada
    python3 tools/check_vectores.py --desde REV     # R6 contra REV en vez de HEAD
    VECTORES_RAIZ=<copia> python3 tools/check_vectores.py   # ensayar contra una copia

## Lo que NO mira

Los vectores que viven fuera de `spec/vectors/` —el corpus KAT de XMSS en
`tools/segunda/kat_xmss/`, por ejemplo— y si un vector dice lo que debe: eso lo dicen los
manifiestos, con `tools/conformidad.sh`.
"""

import collections
import hashlib
import os
import re
import subprocess
import sys

RAIZ = os.environ.get("VECTORES_RAIZ") or os.path.dirname(
    os.path.dirname(os.path.abspath(__file__)))
DIR = "spec/vectors"
HUELLAS = "HUELLAS.sha256"
MANIFIESTO = "MANIFIESTO.txt"
SUFIJO_LINEA = "#linea"
SUFIJO_LAPIDA = "#movido"
VERSION = re.compile(r"^\d+(\.\d+)+$")
REGISTRO = re.compile(r"^([0-9a-f]{64})  (\S.*)$")

CABECERA = """\
# HUELLAS de spec/vectors/: el sha256 de cada vector .json y de cada linea de cada MANIFIESTO.txt.
# Lo escribe `python3 tools/check_vectores.py --registrar` y lo comprueba la misma herramienta en
# cada canon (§692). Solo crece: un vector o una linea registrados no se reescriben ni se borran;
# si su veredicto cambia, se MUEVEN identicos a spec/vectors/<version>/ y lo nuevo entra con otro
# nombre (regla 2 de spec/rfc/PROCESO.md). Formato: "<sha256>  <ruta relativa a spec/vectors>",
# ordenado; en "<manifiesto>#linea" la huella es la de UNA linea sin su salto, una entrada por
# linea; "<ruta>#movido" es la lapida de un vector movido a su version: esa ruta no se reutiliza.
# Los vectores, sin Python: cd spec/vectors && sha256sum -c --ignore-missing HUELLAS.sha256
"""

QUE_HACER = """\
Un vector publicado no se reescribe ni se borra (regla 2 de spec/rfc/PROCESO.md; README.md:
«vectores por versión que jamás se reescriben»). Lo que toca:
  - si fue sin querer, se restaura: git checkout -- <ruta>;
  - si su veredicto cambia, el vector se MUEVE entero, con los mismos bytes, a
    spec/vectors/<version>/<misma ruta>, su linea de manifiesto se mueve identica al manifiesto
    de alli, con el, y lo nuevo entra con OTRO nombre, como hizo el §682: la ruta vieja queda
    como lapida y no se reutiliza;
  - lo NUEVO se registra: python3 tools/check_vectores.py --registrar (no sobrescribe nada)."""


def sha(b):
    return hashlib.sha256(b).hexdigest()


def es_de_version(rel):
    return bool(VERSION.match(rel.split("/", 1)[0]))


def lineas_de(datos):
    """Las líneas de un manifiesto, sin su salto. La última sin salto cuenta igual."""
    if not datos:
        return []
    partes = datos.split(b"\n")
    if datos.endswith(b"\n"):
        partes = partes[:-1]
    return partes


# ── el árbol ────────────────────────────────────────────────────
def leer_arbol(raiz):
    """(ficheros {rel: sha}, lineas {manifiesto: Counter(sha)}, textos {(m, sha): (n, texto)}, ajenos)."""
    base = os.path.join(raiz, DIR)
    ficheros, lineas, textos, ajenos = {}, {}, {}, []
    for d, dirs, nombres in os.walk(base):
        dirs.sort()
        for nombre in sorted(nombres):
            ruta = os.path.join(d, nombre)
            rel = os.path.relpath(ruta, base).replace(os.sep, "/")
            if rel == HUELLAS:
                continue
            with open(ruta, "rb") as fh:
                datos = fh.read()
            if nombre == MANIFIESTO:
                cuenta = collections.Counter()
                for n, linea in enumerate(lineas_de(datos), 1):
                    h = sha(linea)
                    cuenta[h] += 1
                    textos.setdefault((rel, h), (n, linea.decode("utf-8", "replace")))
                lineas[rel] = cuenta
            elif nombre.endswith(".json"):
                ficheros[rel] = sha(datos)
            else:
                ajenos.append(rel)
    return ficheros, lineas, textos, ajenos


# ── el fichero de huellas ───────────────────────────────────────
def parsear(texto):
    """(ficheros, lineas, lapidas, errores). Un error es (n, motivo)."""
    ficheros, lineas, lapidas, errores, donde = {}, {}, {}, [], {}
    for n, linea in enumerate(texto.split("\n"), 1):
        if not linea or linea.startswith("#"):
            continue
        m = REGISTRO.match(linea)
        if not m:
            errores.append((n, "linea ilegible: %r" % linea[:90]))
            continue
        h, rel = m.group(1), m.group(2)
        if rel.endswith(SUFIJO_LINEA):
            man = rel[:-len(SUFIJO_LINEA)]
            if os.path.basename(man) != MANIFIESTO:
                errores.append((n, "%s no es la linea de un %s" % (rel, MANIFIESTO)))
                continue
            lineas.setdefault(man, collections.Counter())[h] += 1
        elif rel.endswith(SUFIJO_LAPIDA):
            vieja = rel[:-len(SUFIJO_LAPIDA)]
            if not vieja.endswith(".json"):
                errores.append((n, "%s no es la lapida de un .json" % rel))
                continue
            if vieja in lapidas and lapidas[vieja] != h:
                errores.append((n, "dos lapidas distintas para %s/%s" % (DIR, vieja)))
                continue
            lapidas[vieja] = h
            donde.setdefault(vieja, n)
        elif rel.endswith(".json"):
            if rel in ficheros and ficheros[rel] != h:
                errores.append((n, "dos huellas distintas para %s/%s" % (DIR, rel)))
                continue
            ficheros[rel] = h
            donde.setdefault(rel, n)
        else:
            errores.append((n, "%s no es ni un .json, ni la linea de un manifiesto, ni una lapida"
                            % rel))
    for rel in sorted(set(ficheros) & set(lapidas)):
        errores.append((donde[rel], "%s/%s tiene huella y lapida: la ruta de un vector movido no "
                        "se reutiliza" % (DIR, rel)))
    return ficheros, lineas, lapidas, errores


def serializar(ficheros, lineas, lapidas):
    regs = [(rel, h) for rel, h in ficheros.items()]
    regs.extend((rel + SUFIJO_LAPIDA, h) for rel, h in lapidas.items())
    for man, cuenta in lineas.items():
        for h, k in cuenta.items():
            regs.extend([(man + SUFIJO_LINEA, h)] * k)
    regs.sort()
    return CABECERA + "".join("%s  %s\n" % (h, rel) for rel, h in regs)


# ── la comparación: lo registrado contra lo que hay ─────────────
def comparar(reg, act, texto):
    """Aplica R1-R4 de lo registrado (reg) a lo actual (act).

    `reg` y `act` son (ficheros, lineas, lapidas); las lápidas de `act` son None cuando `act` es
    el árbol, que no las tiene, y un dict cuando es otro registro (R6). `texto(man, sha)` da el
    texto de una línea de `act`, o None. Devuelve un dict con: modificados [(rel, reg, act)],
    borrados [rel], movidos [(rel, destino)], nuevos [rel], reusados [rel], lapidas [rel],
    perdidas [(man, sha, k)], movidas [(man, destino, sha, k)], nuevas [(man, sha, k)] y solas
    {(man, sha): destino}: las líneas que están en su versión sin que su vector se moviera.
    """
    reg_f, reg_l, reg_lap = reg
    act_f, act_l, act_lap = act
    versiones = sorted({rel.split("/", 1)[0] for rel in list(act_f) + list(act_l)
                        if es_de_version(rel)})
    r = {k: [] for k in ("modificados", "borrados", "movidos", "nuevos", "reusados", "lapidas",
                         "perdidas", "movidas", "nuevas")}
    r["solas"] = {}
    destinos = set()
    for rel in sorted(reg_f):
        h = reg_f[rel]
        if rel in act_f:
            if act_f[rel] != h:
                r["modificados"].append((rel, h, act_f[rel]))
            continue
        destino = None
        if not es_de_version(rel):
            destino = next((v + "/" + rel for v in versiones
                            if act_f.get(v + "/" + rel) == h), None)
        if destino:
            r["movidos"].append((rel, destino))
            destinos.add(destino)
        else:
            r["borrados"].append(rel)
    # R2: la ruta vieja de un vector movido no se reutiliza, ni con sus bytes ni con otros.
    r["reusados"] = sorted(rel for rel in act_f if rel in reg_lap)
    r["nuevos"] = sorted(rel for rel in act_f
                         if rel not in reg_f and rel not in destinos and rel not in reg_lap)
    if act_lap is not None:
        r["lapidas"] = sorted(rel for rel, h in reg_lap.items() if act_lap.get(rel) != h)

    # R3: una línea se mueve a su versión solo con su vector.
    movido_a = dict(r["movidos"])

    def vector_movido(vieja, v):
        nueva = v + "/" + vieja
        if movido_a.get(vieja) == nueva:
            return True
        h = reg_lap.get(vieja)
        return h is not None and act_f.get(nueva) == h

    def algun_vector_movido(d, v):
        viejas = [x for x, _ in r["movidos"]] + list(reg_lap)
        return any(os.path.dirname(x) == d and vector_movido(x, v) for x in viejas)

    def puede_moverse(man, dest, h):
        txt = texto(dest, h)
        if txt is None:
            return False
        v, d = dest.split("/", 1)[0], os.path.dirname(man)
        limpio = txt.strip()
        if not limpio or limpio.startswith("#"):
            return algun_vector_movido(d, v)
        vf = limpio.split("|", 1)[0].strip()
        return vector_movido(os.path.normpath(os.path.join(d, vf)).replace(os.sep, "/"), v)

    sobra = {m: act_l.get(m, collections.Counter()) - reg_l.get(m, collections.Counter())
             for m in set(act_l) | set(reg_l)}
    for man in sorted(reg_l):
        falta = reg_l[man] - act_l.get(man, collections.Counter())
        for h in sorted(falta):
            k = falta[h]
            if not es_de_version(man):
                for v in versiones:
                    dest = v + "/" + man
                    hay = sobra.get(dest, collections.Counter())[h]
                    if not hay:
                        continue
                    if not puede_moverse(man, dest, h):
                        r["solas"][(man, h)] = dest
                        continue
                    toma = min(k, hay)
                    sobra[dest][h] -= toma
                    k -= toma
                    r["movidas"].append((man, dest, h, toma))
                    if not k:
                        break
            if k:
                r["perdidas"].append((man, h, k))
    for man in sorted(sobra):
        for h in sorted(sobra[man]):
            if sobra[man][h] > 0:
                r["nuevas"].append((man, h, sobra[man][h]))
    return r


def aplicar(reg, r, act):
    """Lo registrado tras `--registrar`: lo de antes, con lo movido en su ruta nueva y su lápida
    en la vieja, y lo nuevo."""
    reg_f, reg_l, reg_lap = reg
    act_f = act[0]
    f, lap = dict(reg_f), dict(reg_lap)
    for rel, destino in r["movidos"]:
        lap[rel] = f.pop(rel)
        f[destino] = lap[rel]
    for rel in r["nuevos"]:
        f[rel] = act_f[rel]
    l = {m: collections.Counter(c) for m, c in reg_l.items()}
    for man, dest, h, k in r["movidas"]:
        l[man][h] -= k
        l.setdefault(dest, collections.Counter())[h] += k
    for man, h, k in r["nuevas"]:
        l.setdefault(man, collections.Counter())[h] += k
    return f, {m: +c for m, c in l.items() if +c}, lap


# ── git, para R6 ────────────────────────────────────────────────
def git(*args):
    try:
        p = subprocess.run(("git", "-C", RAIZ) + args, capture_output=True, timeout=60)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return p.stdout if p.returncode == 0 else None


def en_git():
    top = git("rev-parse", "--show-toplevel")
    return top is not None and os.path.realpath(top.decode().strip()) == os.path.realpath(RAIZ)


def texto_de_linea(rev, man, h):
    """El texto de una línea perdida, sacado de git: la huella sola no dice qué línea era."""
    datos = git("show", "%s:%s/%s" % (rev, DIR, man)) if rev else None
    for linea in lineas_de(datos or b""):
        if sha(linea) == h:
            return linea.decode("utf-8", "replace")
    return None


# ── el informe ──────────────────────────────────────────────────
def informe(r, rev, rojos, prefijo, que):
    """Añade a `rojos` un ROJO por cada cosa que R1-R3 no admiten; imprime lo movido."""
    for rel, h, a in r["modificados"]:
        rojos.append("%s %s/%s: sus bytes ya no son los registrados en %s (sha256 %s..., hoy %s...)"
                     % (prefijo + "MODIFICADO", DIR, rel, que, h[:16], a[:16]))
    for rel in r["borrados"]:
        rojos.append("%s %s/%s: registrado en %s y ya no esta, ni movido con sus bytes a un "
                     "directorio de version" % (prefijo + "BORRADO", DIR, rel, que))
    for rel in r["reusados"]:
        rojos.append("%s %s/%s: es la ruta de un vector que se movio a su version, y su lapida "
                     "esta en %s: esa ruta no se reutiliza; lo nuevo entra con OTRO nombre"
                     % (prefijo + "REUSADO", DIR, rel, que))
    for rel in r["lapidas"]:
        rojos.append("%s %s/%s: la lapida registrada en %s ya no esta, o cambio: una lapida no se "
                     "borra" % (prefijo + "LAPIDA", DIR, rel, que))
    for man, h, k in r["perdidas"]:
        txt = texto_de_linea(rev, man, h)
        cual = ("«%s»" % txt) if txt is not None else "la de sha256 %s..." % h[:16]
        veces = "" if k == 1 else " (%d veces)" % k
        sola = r["solas"].get((man, h))
        donde = ""
        if sola:
            donde = ("; esta en %s/%s, pero su vector no se movio alli: una linea se mueve solo "
                     "con su vector, y un comentario con alguno de los de su directorio"
                     % (DIR, sola))
        rojos.append("%s %s/%s: falta la linea registrada en %s%s: %s%s"
                     % (prefijo + "LINEA", DIR, man, que, veces, cual, donde))
    for rel, destino in r["movidos"]:
        print("  movido, con sus bytes: %s/%s -> %s/%s" % (DIR, rel, DIR, destino))
    for man, dest, h, k in r["movidas"]:
        print("  linea movida, identica: %s/%s -> %s/%s (sha256 %s...)"
              % (DIR, man, DIR, dest, h[:16]))


def main(argv):
    registrar = "--registrar" in argv
    rev = "HEAD"
    if "--desde" in argv:
        i = argv.index("--desde")
        if i + 1 >= len(argv):
            print("uso: check_vectores.py [--registrar] [--desde REV]")
            return 2
        rev = argv[i + 1]
    desconocidos = [a for a in argv if a.startswith("--") and a not in ("--registrar", "--desde")]
    if desconocidos:
        print("uso: check_vectores.py [--registrar] [--desde REV]; no conozco %s" % desconocidos)
        return 2

    ruta_h = os.path.join(RAIZ, DIR, HUELLAS)
    act_f, act_l, textos, ajenos = leer_arbol(RAIZ)
    n_lineas = sum(sum(c.values()) for c in act_l.values())

    con_git = en_git()
    if con_git and git("rev-parse", "--verify", "--quiet", rev + "^{commit}") is None:
        print("ROJO: --desde %s no es un commit de este repositorio" % rev)
        return 2
    viejo = git("show", "%s:%s/%s" % (rev, DIR, HUELLAS)) if con_git else None

    rojos, forma = [], []
    if os.path.exists(ruta_h):
        with open(ruta_h, encoding="utf-8") as fh:
            texto = fh.read()
        reg_f, reg_l, reg_lap, errores = parsear(texto)
        for n, motivo in errores:
            rojos.append("ROJO HUELLAS %s/%s:%d: %s" % (DIR, HUELLAS, n, motivo))
        if not errores and texto != serializar(reg_f, reg_l, reg_lap):
            forma.append("ROJO FORMA %s/%s no esta en su forma canonica (orden, cabecera o "
                         "formato): --registrar la reescribe con las mismas huellas" % (DIR, HUELLAS))
    elif viejo is not None:
        falta = ("ROJO HUELLAS %s/%s no esta en el arbol y %s si la tiene: se restaura "
                 "(git checkout -- %s/%s); nunca se rehace de cero" % (DIR, HUELLAS, rev, DIR, HUELLAS))
        if not registrar:
            # Sin fichero, R1-R3 no tienen contra que mirar: un ROJO, no uno por vector.
            print(falta)
            print("\nROJO: sin %s no se comprueba nada (%d vectores y %d lineas en el arbol)."
                  % (HUELLAS, len(act_f), n_lineas))
            return 1
        rojos.append(falta)
        reg_f, reg_l, reg_lap = {}, {}, {}
    elif not registrar:
        # Sin fichero no hay nada que comparar: un ROJO, no uno por vector.
        print("ROJO HUELLAS no hay %s/%s: el primer registro lo hace --registrar" % (DIR, HUELLAS))
        print("\nROJO: %d vectores y %d lineas de manifiesto sin huella." % (len(act_f), n_lineas))
        return 1
    else:
        reg_f, reg_l, reg_lap = {}, {}, {}

    for rel in ajenos:
        rojos.append("ROJO AJENO %s/%s: bajo %s/ solo viven .json y %s, y la puerta no sabe "
                     "juzgar este (R5)" % (DIR, rel, DIR, MANIFIESTO))

    reg, act = (reg_f, reg_l, reg_lap), (act_f, act_l, None)

    def texto(man, h):
        return textos.get((man, h), (None, None))[1]

    r = comparar(reg, act, texto)
    informe(r, rev if con_git else None, rojos, "ROJO ", "las huellas")

    # R6: las huellas de REV siguen en las del arbol (o en las que --registrar va a escribir).
    if con_git and viejo is not None:
        v_f, v_l, v_lap, v_err = parsear(viejo.decode("utf-8", "replace"))
        if v_err:
            rojos.append("ROJO HUELLAS las de %s no se leen (%d linea(s)): R6 no se puede aplicar"
                         % (rev, len(v_err)))
        else:
            nuevo_reg = aplicar(reg, r, act) if registrar else reg
            r6 = comparar((v_f, v_l, v_lap), nuevo_reg, texto)
            antes = len(rojos)
            informe({**r6, "movidos": [], "movidas": []}, rev, rojos, "ROJO R6-",
                    "%s (%s)" % (HUELLAS, rev))
            if len(rojos) > antes:
                rojos.append("ROJO R6 una huella de %s se reescribio o se borro en %s/%s: una huella "
                             "no se sobrescribe; se restaura desde %s" % (rev, DIR, HUELLAS, rev))
    elif con_git:
        print("  R6: %s no tiene %s/%s todavia: este es el primer registro" % (rev, DIR, HUELLAS))
    else:
        print("  R6: %s no es la raiz de un repositorio git: no se comparan las huellas con %s"
              % (RAIZ, rev))

    # R4, agrupado: un directorio, una linea; un manifiesto, una linea.
    por_dir = collections.OrderedDict()
    for rel in r["nuevos"]:
        por_dir.setdefault(os.path.dirname(rel), []).append(os.path.basename(rel))
    nuevos = ["SIN HUELLA %s/%s: %d vector(es) nuevo(s): %s"
              % (DIR, d or ".", len(fs), ", ".join(fs[:6]) + (" y %d mas" % (len(fs) - 6)
                                                               if len(fs) > 6 else ""))
              for d, fs in por_dir.items()]
    por_man = collections.OrderedDict()
    for man, h, k in r["nuevas"]:
        por_man.setdefault(man, []).append((textos[(man, h)][0], k, textos[(man, h)][1]))
    for man, ls in por_man.items():
        n, _, txt = min(ls)
        nuevos.append("SIN HUELLA %s/%s: %d linea(s) nueva(s); la primera, la %d: «%s»"
                      % (DIR, man, sum(k for _, k, _ in ls), n, txt[:100]))

    if registrar:
        if rojos:
            for x in rojos:
                print(x)
            print()
            print(QUE_HACER)
            print("\nROJO: no se registra nada. --registrar solo anade, y hay %d ROJO(s) que no son "
                  "lo nuevo: una huella no se sobrescribe." % len(rojos))
            return 1
        n_f, n_l, n_lap = aplicar(reg, r, act)
        nuevo = serializar(n_f, n_l, n_lap)
        with open(ruta_h, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(nuevo)
        for x in nuevos:
            print("  registrado " + x[len("SIN HUELLA "):])
        n_nuevas = sum(k for _, _, k in r["nuevas"])
        print("registradas: %d vector(es) y %d linea(s) nuevas; %d vector(es) y %d linea(s) "
              "movidos a su version; %s/%s lleva %d vectores y %d lineas%s"
              % (len(r["nuevos"]), n_nuevas, len(r["movidos"]),
                 sum(k for *_, k in r["movidas"]), DIR, HUELLAS, len(n_f),
                 sum(sum(c.values()) for c in n_l.values()),
                 " y %d lapida(s)" % len(n_lap) if n_lap else ""))
        return 0

    rojos += ["ROJO " + x for x in nuevos] + forma
    if rojos:
        for x in rojos:
            print(x)
        print()
        print(QUE_HACER)
        print("\nROJO: %d cosa(s) de spec/vectors/ no cuadran con sus huellas (%d vectores y %d "
              "lineas de manifiesto en el arbol)." % (len(rojos), len(act_f), n_lineas))
        return 1
    print("vectores: %d ficheros .json y %d lineas de %d manifiestos, cada uno con su huella; "
          "%d vector(es) y %d linea(s) movidos a su version, con sus bytes%s"
          % (len(act_f), n_lineas, len(act_l), len(r["movidos"]),
             sum(k for *_, k in r["movidas"]),
             "; %d lapida(s) de rutas que no se reutilizan" % len(reg_lap) if reg_lap else ""))
    return 0


if __name__ == "__main__":
    # SIGPIPE: canalizado por `head`, un print no puede tumbar la puerta (check_modulos).
    try:
        import signal
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    except (ImportError, AttributeError, ValueError):
        pass
    sys.exit(main(sys.argv[1:]))
