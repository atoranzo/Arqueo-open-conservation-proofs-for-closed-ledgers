#!/usr/bin/env python3
"""Comprueba que **ningún documento vivo contradiga los pines del canon**.

## Por qué existe (§237)

`ARQUITECTURA.md` llevaba **veintisiete sellos** diciendo que la capa tenía
172 tests. Tenía 256. `PAPER.md` y `PAPER_EN.md` decían lo mismo, y que el
backend STARK añadía 5 circuitos cuando añade 18.

Ninguna compuerta lo veía, porque **el canon vigila el código y nadie
vigilaba la prosa**. Un número escrito a mano en un `.md` envejece sin
avisar, y quien lo lee no tiene forma de saber que envejeció.

## Qué comprueba, exactamente

Para cada crate con pin en `tools/canon.sh`, busca en los documentos VIVOS
frases del tipo «<crate> … <N> tests» y exige que `N` sea el pin.

**No inventa el pin: lo LEE de `tools/canon.sh`.** Si el canon cambia, esto
cambia con él y sin tocar nada.

## Qué NO comprueba, y por qué

⚠️ **Los REGISTROS quedan fuera, a propósito**:

- `AUDITORIA.md` — los asientos describen **un momento** y no se reescriben.
- `BACKLOG.md` — los números no se reutilizan ni se renumeran.
- `doc/preprints/*` — **depositados con DOI**, conservados verbatim
  (`doc/preprints/ERRATA.md`).
- `doc/ESCALADO.md`, `doc/CONFIANZA_RESIDUAL.md` — **cuerpo VERBATIM** con
  cabecera-mapa encima (§120, decisión A).
- `spec/rfc/*` — un RFC es un documento fechado.

Corregir una cifra ahí **no sería transparencia: sería falsificar el
referente** de las correcciones que ya se publicaron sobre esos textos. Es
la regla que el propio repositorio se dio, y esta herramienta la respeta.

⚠️ Y **no entiende prosa**: caza el patrón «número + tests» cerca del
nombre de un crate. Una afirmación rancia escrita de otra forma **se le
escapa**, y eso hay que saberlo. No es un verificador de documentación: es
una red para la cifra que ya se rancció una vez.

⚠️ **El hueco se midió y se estrechó** (§239): `PRINCIPIOS.md` decía «539
tests» **sin nombrar ningún crate**, y esta herramienta no lo veía. Ahora
también comprueba los TOTALES —una cifra de cuatro dígitos junto a
«tests»— contra la suma de los pines, que es la otra forma en que un
documento cuenta lo mismo.

El hueco no está cerrado: una frase como «unos quinientos tests» sigue
escapándose. **Lo que se puede decir es que las dos formas que ya se
rancciaron están cubiertas.**

## Uso

    python3 tools/check_cifras.py          # exit 0 si nada contradice
"""

import os
import re
import sys

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# ⚠️ REGISTROS: no se tocan. Ver la cabecera.
EXCLUIDOS = {
    "AUDITORIA.md",
    "BACKLOG.md",
    "doc/ESCALADO.md",
    "doc/CONFIANZA_RESIDUAL.md",
}
PREFIJOS_EXCLUIDOS = ("doc/preprints/", "spec/rfc/")

# ⚠️ Lineas que cuentan un MOMENTO PASADO, no el presente. `doc/ZENODO.md`
# narra el estado del sistema cuando se deposito: «The system WAS complete:
# 373 tests». Corregir eso falsificaria el relato, igual que corregir un
# asiento. Se excluye POR LINEA, no por fichero, para que el resto del
# documento siga vigilado (§239).
LINEAS_HISTORICAS = ("The system was complete",)


def pines():
    """Lee de `tools/canon.sh` los pines de tests. NO se escriben aquí."""
    ruta = os.path.join(RAIZ, "tools", "canon.sh")
    fuera = {}
    for linea in open(ruta, encoding="utf-8"):
        m = re.match(r"^([a-z0-9-]+)\s+\w+\s+(\d+)\s+(\d+)\s+(\d+)\s", linea)
        if m:
            fuera[m.group(1)] = int(m.group(2))
    return fuera


def pines_sello():
    """Solo los pines del nivel `--sello`: la suma que se cita mas."""
    ruta = os.path.join(RAIZ, "tools", "canon.sh")
    fuera = {}
    for linea in open(ruta, encoding="utf-8"):
        m = re.match(r"^([a-z0-9-]+)\s+sello\s+(\d+)\s", linea)
        if m:
            fuera[m.group(1)] = int(m.group(2))
    return fuera


def vivos():
    for base, dirs, fs in os.walk(RAIZ):
        dirs[:] = [d for d in dirs if d not in {".git", "target", ".canon"}]
        for f in fs:
            if not f.endswith(".md"):
                continue
            rel = os.path.relpath(os.path.join(base, f), RAIZ)
            if rel in EXCLUIDOS or rel.startswith(PREFIJOS_EXCLUIDOS):
                continue
            yield rel


# ── §262 · los DESGLOSES ────────────────────────────────────────
#
# ⚠️ El hueco que esto cierra: `PRINCIPIOS.md` decia «12 del verificador
#    independiente» cuando §256 lo habia dejado en 22, y VIAJO UN SELLO
#    ENTERO porque en esa forma la cifra no lleva «tests» al lado ni el
#    nombre del crate — las dos unicas formas que este fichero cazaba.
#
# ⚠️ Y por que NO se caza el patron suelto: se midio. Buscar «N <palabra>»
#    por todo el documento dio CINCO discrepancias y LAS CINCO FALSAS.
#    Una compuerta con falsos positivos se acaba ignorando, y una
#    compuerta ignorada es peor que su ausencia declarada.
#
# ⚠️ Lo que la hace solida NO es acotar por acotar: es que **el ancla ya
#    esta validada**. El desglose se lee donde el documento ya declaro un
#    total que este mismo fichero verifica desde §239. Fuera de ahi, un
#    numero junto a una palabra no afirma nada. Engancharse a una
#    comprobacion que existe en vez de crear una segunda superficie.
#
# ⚠️ ADYACENCIA, operativa y no interpretada: entre el total y el guion NO
#    PUEDE HABER UN PUNTO. Es decir, misma frase. Medido contra la entrada
#    real: sin esa regla el ancla saltaba un punto y sesenta caracteres
#    hasta OTRO parrafo de `PRINCIPIOS.md`.

FRASE_DESGLOSE = re.compile(
    r"(\d{3,4})\s*(?:tests|pruebas)([^\u2014.]*)\u2014([^\u2014]*)\u2014")
ITEM_DESGLOSE = re.compile(
    # ⚠️ EL CONECTOR ES OBLIGATORIO, y el ensayo lo exigio: sin el, el
    #    tramo que sigue a un total casa TAMBIEN OTROS TOTALES —«873
    #    contando los pines», «887 declaradas»— y da diez falsas alarmas.
    #    Los siete items reales llevan «de», «del» o «de la»; los totales
    #    y el ruido no llevan ninguno.
    r"\*{0,2}(\d{1,4})\*{0,2}\s+(?:de\s+la\s+|de\s+los\s+|del\s+|de\s+)"
    r"\*{0,2}([a-zA-Z\u00e0-\u00ff]+(?:\s+[a-zA-Z\u00e0-\u00ff]+)?)")


def alias_de_crates():
    """Lee de `tools/canon.sh` los alias EN PROSA de cada crate.

    ⚠️ Viven en LA FILA DEL CRATE y no en este fichero **a proposito**. Una
    copia aqui no duplicaria un dato: **afirmaria una correspondencia que
    nadie comprueba**. El dia que el testigo cambiara de crate, `canon.sh`
    cambiaria la fila y esto seguiria leyendo el pin viejo bajo un nombre
    que ya no corresponde — **no fallaria: validaria mal, en silencio**.
    """
    ruta = os.path.join(RAIZ, "tools", "canon.sh")
    out = {}
    for linea in open(ruta, encoding="utf-8"):
        m = re.match(r"^([a-z0-9-]+)\s+\w+\s+\d+\s+\d+\s+\d+\s+\d+\s+alias=([^\u00b7\n]+)",
                     linea)
        if m:
            out[" ".join(m.group(2).split()).lower()] = m.group(1)
    return out


def desgloses(pins, alias):
    """Comprueba las cifras POR CRATE dentro de la frase de un total.

    ⚠️ **INERTE si nadie ha declarado un alias.** Una compuerta que exige
    datos que aun no existen no es una mejora: es una parada, y con la
    causa en el sello anterior.

    ⚠️ La regla es «**todo numero que APARECE resuelve y cuadra**», no
    «todos los pines aparecen»: un desglose parcial es prosa legitima.
    """
    if not alias:
        return [], 0
    fallos, vistas = [], 0
    for rel in vivos():
        texto = open(os.path.join(RAIZ, rel), encoding="utf-8").read()
        plano = " ".join(texto.split())
        for fr in FRASE_DESGLOSE.finditer(plano):
            for it in ITEM_DESGLOSE.finditer(fr.group(3)):
                v = int(it.group(1))
                nombre = " ".join(it.group(2).split()).lower()
                crate = alias.get(nombre) or alias.get(nombre.split()[0])
                vistas += 1
                if crate is None:
                    fallos.append((rel, 0, f"desglose <{nombre}>", v,
                                   "ningun alias declarado en tools/canon.sh",
                                   fr.group(3)[:78]))
                elif pins.get(crate) != v:
                    fallos.append((rel, 0, crate, v, pins.get(crate),
                                   fr.group(3)[:78]))
    return fallos, vistas


# ── §553 · el TOTAL CON LARGOS ────────────────────────────
#
# ⚠️ El hueco que esto cierra (5.A-409): el bucle de TOTALES exige la cifra con
#    «tests» o «pruebas» a un paso. Los tres documentos que publican el total
#    del sello publican TAMBIEN el total con largos, y esa cifra va seguida de
#    su propia cara —«contando los pines», «counting the pins»—, sin el
#    sustantivo. Al mover el pin del SDK (§550) el gate canto las TRES del
#    sello y callo las TRES del largo, que se corrigieron a mano.
#
# ⚠️ NO es una segunda superficie: se cuelga del ANCLA QUE YA ESTA VALIDADA,
#    como los desgloses del §262. Y no re-detecta el ancla: el bucle de
#    TOTALES la APUNTA cuando la valida. UN productor, dos consumidores.
#
# ⚠️ ADYACENCIA, medida, y la regla del hermano NO sirve: entre las dos cifras
#    hay DOS PUNTOS en los dos PAPER —«winterfell 0.13.1»—, así que el «sin
#    punto en medio» de los desgloses daría rojo por algo legítimo (la lección
#    de la 117). Lo que discrimina es la CARA que sigue a la cifra, y el
#    párrafo —de blanco a blanco— como ámbito.
#
# ⚠️ PRESENCIA, con el molde del ATADO D (§549): un gate que sólo cruza lo que
#    ya lleva la marca es ciego justo a lo que falta. Donde se publica el total
#    del sello se publica el del largo, y el que deje de publicarlo NO pasa
#    callando. Si algún día un documento cita legítimamente sólo el del sello,
#    esto se pone rojo y lo paga un corte: es fail-closed a propósito.

CARAS_LARGOS = ("contando los pines", "counting the pins")
LARGOS_EN_PROSA = re.compile(
    r"(?<![\d.])(\d{3,4})(?![\d])\*{0,2}\s*(?:%s)" % "|".join(CARAS_LARGOS))


def parrafo(lineas, n):
    """El párrafo que contiene la línea n (1-based), de blanco a blanco, aplanado."""
    i = j = n - 1
    while i > 0 and lineas[i - 1].strip():
        i -= 1
    while j + 1 < len(lineas) and lineas[j + 1].strip():
        j += 1
    return " ".join("\n".join(lineas[i:j + 1]).split())


def total_largos(anclas, suma_todos):
    """El TOTAL CON LARGOS: su VALOR en todo documento vivo, y su PRESENCIA
    donde el total del sello ya está validado.

    `anclas` son los (documento, línea) que el bucle de TOTALES apuntó al
    validar el total del sello: no se vuelven a detectar aquí.
    """
    fallos, ausentes, vistas = [], [], 0
    for rel in vivos():
        plano = " ".join(open(os.path.join(RAIZ, rel), encoding="utf-8").read().split())
        for m in LARGOS_EN_PROSA.finditer(plano):
            vistas += 1
            v = int(m.group(1))
            if v != suma_todos:
                fallos.append((rel, 0, "TOTAL con largos", v, suma_todos,
                               plano[max(0, m.start() - 30):m.start() + 48]))
    for rel, n in anclas:
        lineas = open(os.path.join(RAIZ, rel), encoding="utf-8").read().split("\n")
        if not LARGOS_EN_PROSA.search(parrafo(lineas, n)):
            ausentes.append((rel, n))
    return fallos, ausentes, vistas


def cronica():
    """La CRONICA de cada fila del canon: si una fila cuenta su historia, su pin tiene que ser
    el segundo numero de su ULTIMA entrada.

    Nace en el S552. El S550 movio el pin del SDK de 11 a 13 y dejo su cronica en el S542:
    nueve filas de diez cuadraban y la decima contaba una historia falsa. Ninguna compuerta lo
    veia, y es la misma familia que el ATADO B -- el canon vigila el codigo, y su propia tabla
    no la miraba nadie.

    NO se exige que la cadena de entradas sea CONTINUA. Hoy cuatro filas saltan (mudanzas de
    crate, entradas que no se escribieron), y un juez mas estricto que su invariante da rojo
    por algo legitimo: esa es la leccion de la 117.
    """
    ruta = os.path.join(RAIZ, "tools", "canon.sh")
    fallos, con_cronica = [], 0
    for n, linea in enumerate(open(ruta, encoding="utf-8"), 1):
        m = re.match(r"^([a-z0-9-]+)\s+\w+\s+(\d+)\s+\d+\s+\d+\s", linea)
        if not m:
            continue
        ents = re.findall(r"\u00a7(\d+[A-Z-]*): (\d+) -> (\d+)", linea)
        if not ents:
            continue
        con_cronica += 1
        if int(ents[-1][2]) != int(m.group(2)):
            fallos.append((n, m.group(1), int(m.group(2)), ents[-1][0], int(ents[-1][2])))
    return fallos, con_cronica


def main():
    p = pines()
    if not p:
        print("ROJO: no se pudo leer ningun pin de tools/canon.sh")
        return 2

    malas = []
    revisadas = 0
    for rel in vivos():
        texto = open(os.path.join(RAIZ, rel), encoding="utf-8").read()
        for n, linea in enumerate(texto.split("\n"), 1):
            for crate, pin in p.items():
                # «<crate> ... <N> tests» en la misma linea, en cualquier orden.
                if crate not in linea:
                    continue
                # ⚠️ Una linea que filtra por un test o un circuito CONCRETO
                # no es una cuenta del crate: `cargo test -p zk-core
                # circuit_audit` dice cuantos tiene ESE circuito. Fue un
                # falso positivo real (§237), y un falso positivo hace que
                # la herramienta se deje de mirar.
                # El filtro va DESPUES de --release y NO empieza por `#`:
                # `... --release circuit_audit` es un filtro;
                # `... --release   # 242 tests` es un comentario.
                if re.search(r"--release\s+[^#\s]", linea) or "circuit_" in linea:
                    continue
                for m in re.finditer(r"(\d[\d.]*)\s*(?:tests|pruebas)", linea):
                    revisadas += 1
                    v = int(m.group(1).replace(".", ""))
                    if v != pin:
                        malas.append((rel, n, crate, v, pin, linea.strip()[:78]))

    # ⚠️ TOTALES: una cifra de 3-4 digitos junto a «tests» y SIN nombre de
    # crate suele ser la suma. Se compara contra las sumas posibles —la del
    # sello, la de todos los pines— con margen cero: si no es ninguna, se
    # señala para que alguien mire, porque asi se colo el 539.
    suma_sello = sum(pines_sello().values())
    suma_todos = sum(p.values())
    posibles = {suma_sello, suma_todos}
    anclas_sello = []
    for rel in vivos():
        texto = open(os.path.join(RAIZ, rel), encoding="utf-8").read()
        for n, linea in enumerate(texto.split("\n"), 1):
            if any(c in linea for c in p):
                continue          # ya lo mira el bucle de arriba
            if re.search(r"--release\s+[^#\s]", linea) or "circuit_" in linea:
                continue
            if any(h in linea for h in LINEAS_HISTORICAS):
                continue
            # ⚠️ **UNA palabra entre la cifra y el sustantivo** (5.A-350, §519-B). El
            # patron exigia `tests` PEGADO al numero, y el gemelo ingles dice
            # <<1300 executable tests>>: llevaba VEINTE sellos rancio y esta compuerta
            # salia verde. No es de idioma, es de forma. Y no se afloja mas: el
            # sustantivo sigue siendo obligatorio, porque un patron suelto ya se midio
            # y dio cinco falsas (lineas 122-125). Con este, el censo del arbol vivo
            # pasa de 14 hits a 15, y el que entra es exactamente `PAPER_EN.md:33`.
            for m in re.finditer(
                    r"\*?\*?(\d{3,4})\*?\*?\s*(?:[a-zA-Z\u00e0-\u00ff]+\s+)?(?:tests|pruebas)", linea):
                v = int(m.group(1))
                revisadas += 1
                if v == suma_sello:
                    anclas_sello.append((rel, n))   # §553: el ancla se APUNTA aqui
                if v not in posibles:
                    malas.append((rel, n, "TOTAL", v,
                                  f"{suma_sello} (sello) o {suma_todos} (todos)",
                                  linea.strip()[:78]))

    fallos_cr, con_cronica = cronica()
    print("  CRONICA: %d fila(s) del canon cuentan su historia, y su pin es el de su ultima "
          "entrada" % con_cronica)

    fallos_lg, ausentes_lg, vistas_lg = total_largos(anclas_sello, suma_todos)
    malas.extend(fallos_lg)
    revisadas += vistas_lg
    print("  LARGOS: %d cita(s) del total con largos, y %d de las %d ancla(s) del total de "
          "sello lo llevan en su parrafo"
          % (vistas_lg, len(anclas_sello) - len(ausentes_lg), len(anclas_sello)))

    fallos_desglose, vistas_desglose = desgloses(p, alias_de_crates())
    malas.extend(fallos_desglose)
    revisadas += vistas_desglose

    for rel, n, crate, v, pin, l in malas:
        print(f"  RANCIA  {rel}:{n}")
        print(f"          dice {v} tests para `{crate}` y el canon pina {pin}")
        print(f"          {l}")

    if fallos_cr:
        print("")
        print("ROJO: %d fila(s) del canon con el pin y su cronica discordes" % len(fallos_cr))
        for n, crate, pin, sello, hasta in fallos_cr:
            print("  tools/canon.sh:%-4d %s: pina %d y su ultima entrada dice "
                  "§%s: -> %d" % (n, crate, pin, sello, hasta))
        print("")
        print("  Un pin que se mueve sin escribir su entrada deja la fila contando una")
        print("  historia falsa. El pin lo mueve el corte; la entrada, el mismo corte.")
        return 1

    if ausentes_lg:
        print("")
        print("ROJO: %d documento(s) publican el total de sello y no el total con largos"
              % len(ausentes_lg))
        for rel, n in ausentes_lg:
            print("  %s:%-4d su parrafo no lleva ninguna cifra con %s"
                  % (rel, n, " ni ".join("<<%s>>" % c for c in CARAS_LARGOS)))
        print("")
        print("  Un gate que solo cruza lo que ya lleva la marca es ciego a lo que falta:")
        print("  el que DEJE de publicar la cuenta no puede pasar callando (ATADO D, S549).")
        return 1

    if malas:
        print(f"\n{len(malas)} cifra(s) que contradicen el canon. "
              f"Un numero a mano en un .md envejece sin avisar.")
        return 1
    print(f"{revisadas} cifra(s) de tests en documentos vivos: ninguna "
          f"contradice el canon ({len(p)} pines leidos de tools/canon.sh). "
          f"De ellas, {vistas_desglose} de desglose.")
    return 0


if __name__ == "__main__":
    try:
        import signal
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    except (ImportError, AttributeError, ValueError):
        pass
    sys.exit(main())
