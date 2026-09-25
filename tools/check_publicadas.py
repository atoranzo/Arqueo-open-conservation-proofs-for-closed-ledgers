#!/usr/bin/env python3
"""ATADO B - la CONSTANTE contra los DOCUMENTOS.

Su gemelo es el test la_cifra_publicada_sigue_siendo_la_medida, en
crates/zk-ssl/src/metrics.rs, que ata el INSTRUMENTO a la constante.
Aqui se ata la constante a lo PUBLICADO. Cada rojo dice cual de las dos
cosas se quedo atras.

Por que existe: el 31-07-2026 el asiento §83.3 midio la deriva de la
cifra publicada en 2,2 % y la declaro "dentro de la guarda". El 14-08
era 4,8 %. Una TOLERANCIA declarada una vez sobre algo que crece es una
promesa, no un gate.

LAS DOS UNIDADES. La cifra es binaria. Un documento puede escribirla en
MiB (2^20) o dar su equivalente SI en MB (10^6); las dos son correctas y
las dos se derivan de los MISMOS bytes: PUBLICADA_PAGO_MIN_B y
PUBLICADA_PAGO_MAX_B, los extremos de la banda (S538). Lo que NO vale es
el valor binario con etiqueta MB, que es el defecto que §83.3 cazo y que
hace leer un 9,9 % menos.

VENTANA. El contexto se busca en la linea y en las DOS de alrededor: la
primera version miraba solo la linea y por eso no vio tres cifras SI que
viven un renglon mas abajo que su frase. Una frase que cruza un salto de
linea no se cuenta con un patron de una linea.

CITAS HISTORICAS. Una cifra que aparece junto a la marca de una version
anterior NO es lo que este repo publica: es lo que publico otro deposito.
Cambiarla seria falsificar el registro. Se salta, y **se imprime**: una
exclusion silenciosa es un agujero.

SIN DECIMAL (§308). Hasta aqui el patron EXIGIA parte decimal, y por eso
no veia siete citas vivas escritas con el entero pelado: 59 MB y 124 MiB.
Ahora el decimal es OPCIONAL. Y el patron ancla su borde IZQUIERDO: sin
eso "1.126,2 MiB" casaria como "126,2" -el separador de miles, el mismo
caso que ya mordio con un patron de 994 sobre 41.994-, y ese agujero lo
tenia la version anterior sin saberlo.

LA VIA (S362). La cifra se mide sobre `send`/`claim` de la capa, que es
la via que las cifras publicadas DESCRIBEN, y no sobre la del cliente.
Es deliberado y su permiso vive en `mod tests` de metrics.rs: cuando
llegue la migracion -- ENTRADA 32 del BACKLOG -- las mediciones cambian
con ella, y eso se DECLARA, no se absorbe.

MEDIDO el 2026-08-26: la via DOCUMENTADA -- send_materials ->
client::prove_send -> apply_send -- da los MISMOS bytes, 66_739 y
66_692, en cinco repeticiones y sin una sola diferencia. La cifra
publicada NO depende de la via. Desde el S362 lo PINA un segundo
gemelo, los_dos_lados_del_pago_atan_la_banda, que asierta esos dos
bytes ademas de la relacion temporal.

Ceguera declarada:
  - no ve una cifra escrita con palabras;
  - no entra en AUDITORIA.md: los asientos son REGISTRO HISTORICO y una
    cifra vieja dentro de un asiento es correcta;
  - no entra en doc/preprints/: entrada 28, SUSPENDIDA hasta el fin del
    proyecto. Lo que alli quede rancio se DECLARA, no se repara;
  - no vigila TIEMPOS ni RATIOS: dependen de la maquina y publicarlos en
    absoluto es una decision de diseno, no un numero que atar. Declarado
    en el asiento del §304.
  - no ve GB ni GiB, ni ninguna escala que no sea POR MIL: la CIFRA solo
    casa MiB|MB y los dos valores esperados salen de PAGO_B x 1000. Una
    cifra derivada al millon queda fuera del gate por construccion;
  - el CONTEXTO es un vocabulario fijo de frases "por mil", asi que una
    tabla que dice "1M op/dia" no lo dispara y sus cifras no se miran;
  - no entra en el CODIGO: solo recorre los .md de la raiz. log.rs:191
    publicaba la aritmetica de un paso y ningun gate la veia; se reparo a
    mano en §308, y extender el atado a los .rs queda por censar.

ATADO C (S403) - la URL DEL REPOSITORIO contra los documentos. Mismo
principio, otra constante: el repositorio se renombro y la URL vieja solo
vivia por una redireccion ajena; dieciseis lineas de diez ficheros la citaban
y ningun gate lo veia. URL_REPO se declara UNA vez aqui; el gate exige que
el token viejo no aparezca en ningun documento vivo ni en CITATION.cff, que
CITATION.cff declare exactamente URL_REPO en repository-code, y -prueba de
vida- que la URL nueva se cite al menos una vez: un universo vacio no pasa.
Universo PROPIO (documentos_url): recursivo bajo la raiz, porque la URL vive
tambien en doc/; fuera AUDITORIA.md y BACKLOG.md (registro) y doc/preprints/
(entrada 28), que se imprimen como exclusion.
"""
import os
import re
import sys

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FUENTE = os.path.join(RAIZ, "crates", "zk-ssl", "src", "metrics.rs")
VENTANA = 2

EXCLUIDOS = {
    "AUDITORIA.md": "registro historico: una cifra vieja en un asiento es correcta",
    "BACKLOG.md": "registro: las entradas no se reescriben (la 22 cita la cifra de su fecha)",
    "VISION.md": "nota 98: la seccion 3.8 ENTERA deriva de la era de UN PASO "
                 "-59 MB/mil, 59 GB/dia x3, 131 MB/dia, 463x, 137 B por entrada-. "
                 "No es una cifra rancia: es un analisis. Corte propio",
}

CONTEXTO = re.compile(
    r"mil pagos|mil transferencias|mil operaciones|por cada mil|unidades SI|"
    r"thousand transfers|thousand complete payments|per thousand|SI units",
    re.I,
)
HISTORICA = re.compile(
    r"versiones anteriores|versi[oó]n anterior|version anterior|"
    r"earlier version|previous version|retirada desde entonces",
    re.I,
)
CIFRA = re.compile(r"(?<![\d.,])(\d{2,3})(?:[.,](\d))?\s*(MiB|MB)\b")

# ATADO C (S403)
URL_REPO = "https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers"
TOKEN_VIEJO = "ZK-SSL-ZK-Sovereign-Settlement-Layer"
CFF = os.path.join(RAIZ, "CITATION.cff")
EXCLUIDOS_URL = {
    "AUDITORIA.md": "registro historico: un asiento cita la URL de su fecha",
    "BACKLOG.md": "registro: las entradas no se reescriben",
}
PREFIJOS_EXCLUIDOS_URL = ("doc/preprints/",)

# ATADO D (S549) - el ESTADO de un RFC contra lo que los documentos publican de el.
# El vocabulario lo dicta spec/rfc/PROCESO.md (BORRADOR -> PROPUESTO -> ACEPTADO ->
# FINAL, o RETIRADO); aqui va la cara que cada estado tiene PUBLICADA, por idioma.
DIR_RFC = os.path.join(RAIZ, "spec", "rfc")
PLANTILLA = "0000"
ESTADOS = {
    "ACEPTADO": ("ACCEPTED", ("aceptado", "aceptados", "accepted")),
    "PROPUESTO": ("PROPOSED", ("propuesto", "propuestos", "proposed")),
}
# Los sitios que hoy publican la CUENTA entera. No es un tope: si aparece uno nuevo se
# cruza igual. Esta lista existe para que uno que DEJE de publicarla no pase callando.
CARDINALES = ("README.md", "README_EN.md", "RESUMEN_BILINGUE.md", "RESUMEN_EJECUTIVO.md")
FILA_RFC = "spec/README.md"
UMBRAL_CARDINAL = 3   # tres numeros en una linea; con menos es una mencion, no una cuenta
NUM_RFC = re.compile(r"\b(000\d)\b")
PAL_ESTADO = re.compile(r"\b(aceptad[oa]s?|propuest[oa]s?|accepted|proposed)\b", re.I)
RANGO_RFC = re.compile(r"000\d\s*[-\u2013]\s*000\d")



def constante():
    """Lee la fuente unica. Por ESTRUCTURA, no por posicion."""
    try:
        with open(FUENTE, encoding="utf-8") as fh:
            src = fh.read()
    except OSError as exc:
        print("ROJO: no se puede leer la fuente unica: %s" % exc)
        sys.exit(1)
    # S538 (RFC-0009 D-AC): la cifra es una BANDA con dos extremos publicos; una cita
    # vale si es uno de los dos, en MiB (2^20) o en su equivalente SI (10^6).
    ext = {}
    for nombre in ("PUBLICADA_PAGO_MIN_B", "PUBLICADA_PAGO_MAX_B"):
        m = re.search(r"const %s:\s*usize\s*=\s*([0-9_]+)" % nombre, src)
        if not m:
            print("ROJO: no aparece %s en %s" % (nombre, FUENTE))
            print("      la fuente unica son esas dos constantes: sin ellas no hay atado")
            sys.exit(1)
        ext[nombre] = int(m.group(1).replace("_", ""))
    b = (ext["PUBLICADA_PAGO_MIN_B"], ext["PUBLICADA_PAGO_MAX_B"])
    f = re.search(r'const PUBLICADA_FECHA:\s*&str\s*=\s*"([^"]+)"', src)
    mib = tuple("%.1f" % (x * 1000 / 1048576.0) for x in b)
    msi = tuple("%.1f" % (x * 1000 / 1000000.0) for x in b)
    return b, mib, msi, (f.group(1) if f else "sin fecha")


def documentos():
    for nombre in sorted(os.listdir(RAIZ)):
        if nombre.endswith(".md") and nombre not in EXCLUIDOS:
            yield nombre



def documentos_url():
    """Universo del ATADO C: todo .md bajo la raiz, recursivo, menos los registros
    y doc/preprints/; mas CITATION.cff, que no es .md y es lo primero que lee
    quien llega de fuera."""
    for base, dirs, fs in os.walk(RAIZ):
        dirs[:] = [d for d in dirs if d not in {".git", "target", ".canon"}]
        for f in sorted(fs):
            if not f.endswith(".md"):
                continue
            rel = os.path.relpath(os.path.join(base, f), RAIZ)
            if rel in EXCLUIDOS_URL or rel.startswith(PREFIJOS_EXCLUIDOS_URL):
                continue
            yield rel
    yield "CITATION.cff"


def atado_c():
    """Devuelve (fallos, citas de la URL nueva, documentos recorridos)."""
    fallos = []
    nuevas = 0
    recorridos = 0
    for rel in documentos_url():
        ruta = os.path.join(RAIZ, rel)
        try:
            with open(ruta, encoding="utf-8") as fh:
                lineas = fh.readlines()
        except OSError as exc:
            fallos.append((rel, 0, "ILEGIBLE", str(exc)))
            continue
        recorridos += 1
        for i, linea in enumerate(lineas, 1):
            if TOKEN_VIEJO in linea:
                fallos.append((rel, i, "URL VIEJA", linea.strip()[:70]))
            if URL_REPO in linea:
                nuevas += 1
    try:
        with open(CFF, encoding="utf-8") as fh:
            cff = fh.read()
    except OSError as exc:
        fallos.append(("CITATION.cff", 0, "ILEGIBLE", str(exc)))
        return fallos, nuevas, recorridos
    m = re.search(r'^repository-code:\s*"([^"]+)"', cff, re.M)
    if not m:
        fallos.append(("CITATION.cff", 0, "SIN repository-code", ""))
    elif m.group(1) != URL_REPO:
        fallos.append(("CITATION.cff", 0, "repository-code", m.group(1)))
    return fallos, nuevas, recorridos


def rfcs():
    """(numero, ruta, estado) de cada RFC del directorio, menos la plantilla.
    El PRODUCTOR del estado es la linea `- **Estado:**` del propio RFC: una sola."""
    salida = []
    for f in sorted(os.listdir(DIR_RFC)):
        m = re.match(r"^(\d{4})-.*\.md$", f)
        if not m or m.group(1) == PLANTILLA:
            continue
        with open(os.path.join(DIR_RFC, f), encoding="utf-8") as fh:
            txt = fh.read()
        e = re.findall(r"^- \*\*Estado:\*\* ([A-Z]+)", txt, re.M)
        salida.append((m.group(1), "spec/rfc/" + f, e))
    return salida


def repartir(linea):
    """{numero: ESTADO} de una linea que ENUMERA: los numeros que van antes de una
    palabra de estado son suyos, desde la palabra anterior. Devuelve tambien los
    numeros repetidos, porque un diccionario se queda con el ultimo y no lo diria."""
    grupos = {}
    repes = [n for n in set(NUM_RFC.findall(linea))
             if NUM_RFC.findall(linea).count(n) > 1]
    ini = 0
    for m in PAL_ESTADO.finditer(linea):
        pal = m.group(0).lower()
        estado = None
        for e, (_, palabras) in ESTADOS.items():
            if pal in palabras:
                estado = e
        if estado is not None:
            for num in NUM_RFC.findall(linea[ini:m.start()]):
                grupos[num] = estado
        ini = m.end()
    return grupos, sorted(repes)


def atado_d():
    """Devuelve (fallos, RFC con estado, sitios con cuenta, cuentas halladas)."""
    fallos = []
    lista = rfcs()
    conocidos = set(num for num, _, _ in lista)
    esperado = {}
    for num, rel, estados in lista:
        if len(estados) != 1:
            fallos.append((rel, 0, "ESTADO", "declara %d lineas de estado, se esperaba 1"
                           % len(estados)))
        elif estados[0] not in ESTADOS:
            fallos.append((rel, 0, "ESTADO", "%s no tiene cara publicada; el vocabulario "
                           "vive en spec/rfc/PROCESO.md" % estados[0]))
        else:
            esperado[num] = estados[0]

    # D.1 - la FILA de spec/README.md: una por RFC, y con su token de estado
    with open(os.path.join(RAIZ, FILA_RFC), encoding="utf-8") as fh:
        filas = fh.read().split("\n")
    for num, rel, _ in lista:
        if num not in esperado:
            continue
        cabeza = "| `" + rel[len("spec/"):] + "`"
        hit = [(i + 1, l) for i, l in enumerate(filas) if l.startswith(cabeza)]
        if len(hit) != 1:
            fallos.append((FILA_RFC, 0, "FILA", "el RFC-%s tiene %d filas, se esperaba 1"
                           % (num, len(hit))))
            continue
        n, linea = hit[0]
        token = "RFC-%s, %s" % (num, ESTADOS[esperado[num]][0])
        if token not in linea:
            dice = re.search(r"RFC-%s[,:]?\s*[A-Z]*" % num, linea)
            fallos.append((FILA_RFC, n, "TOKEN", "no dice <<%s>>; dice <<%s>>"
                           % (token, dice.group(0).strip() if dice else "nada")))

    # D.2 - los CARDINALES: toda linea del universo con tres numeros y una palabra de
    # estado es una cuenta, y una cuenta las lleva TODAS, cada una en su grupo.
    sitios = {}
    for rel in documentos_url():
        if rel == FILA_RFC or not rel.endswith(".md"):
            continue
        try:
            with open(os.path.join(RAIZ, rel), encoding="utf-8") as fh:
                lineas = fh.read().split("\n")
        except OSError as exc:
            fallos.append((rel, 0, "ILEGIBLE", str(exc)))
            continue
        for i, linea in enumerate(lineas, 1):
            if not PAL_ESTADO.search(linea):
                continue
            if len(set(NUM_RFC.findall(linea))) < UMBRAL_CARDINAL:
                continue
            sitios.setdefault(rel, []).append(i)
            if RANGO_RFC.search(linea):
                fallos.append((rel, i, "RANGO", "una cuenta se ENUMERA: un rango lo lee "
                               "mal quien lo lee y quien lo mide"))
                continue
            grupos, repes = repartir(linea)
            for num in repes:
                fallos.append((rel, i, "DUPLICADO", "el RFC-%s sale dos veces" % num))
            for num in sorted(esperado):
                if num not in grupos:
                    fallos.append((rel, i, "AUSENTE", "el RFC-%s no esta en la cuenta" % num))
                elif grupos[num] != esperado[num]:
                    fallos.append((rel, i, "DISCREPA", "el RFC-%s sale como %s y su RFC dice "
                                   "%s" % (num, grupos[num], esperado[num])))
            for num in sorted(set(grupos) - set(esperado)):
                if num in conocidos:
                    continue   # su RFC ya tiene su ROJO arriba: una causa, un rojo
                fallos.append((rel, i, "SOBRA", "cita el RFC-%s, que no esta en spec/rfc/"
                               % num))
    for nombre in CARDINALES:
        if nombre not in sitios:
            fallos.append((nombre, 0, "SIN CUENTA", "sitio conocido que ha dejado de "
                           "publicar la cuenta de los RFC"))
    return fallos, len(esperado), len(sitios), sum(len(v) for v in sitios.values())


def main():
    pago_b, mib, msi, fecha = constante()
    pares_mib = {tuple(x.split(".")) for x in mib}
    pares_msi = {tuple(x.split(".")) for x in msi}

    fallos = []
    saltadas = []
    vistos = 0

    for nombre in documentos():
        ruta = os.path.join(RAIZ, nombre)
        with open(ruta, encoding="utf-8") as fh:
            lineas = fh.readlines()
        for i, linea in enumerate(lineas):
            ini = max(0, i - VENTANA)
            fin = min(len(lineas), i + VENTANA + 1)
            bloque = "".join(lineas[ini:fin])
            if not CONTEXTO.search(bloque):
                continue
            for m in CIFRA.finditer(linea):
                par = (m.group(1), m.group(2))
                unidad = m.group(3)
                citada = par[0] if par[1] is None else "%s,%s" % par
                if HISTORICA.search(bloque):
                    saltadas.append(
                        (nombre, i + 1,
                         "%s %s - cita de una version anterior"
                         % (citada, unidad))
                    )
                    continue
                vistos += 1
                if par in pares_mib and unidad == "MiB":
                    continue
                if par in pares_msi and unidad == "MB":
                    continue
                if par in pares_mib and unidad == "MB":
                    fallos.append(
                        (nombre, i + 1, "UNIDAD",
                         "el valor binario lleva etiqueta MB; o MiB, o el "
                         "equivalente SI %s-%s MB" % msi)
                    )
                else:
                    fallos.append(
                        (nombre, i + 1, "VALOR",
                         "dice %s %s; se esperan %s o %s MiB, o %s o %s MB"
                         % ((citada, unidad) + mib + msi))
                    )

    print("check_publicadas: %d..%d B por pago -> %s-%s MiB / %s-%s MB por mil (banda "
          "medida el %s)" % (pago_b + mib + msi + (fecha,)))
    print("  %d citas examinadas en documentos del repo" % vistos)
    for nombre, razon in sorted(EXCLUIDOS.items()):
        print("  excluido %s - %s" % (nombre, razon))
    print("  excluido doc/preprints/ - entrada 28, SUSPENDIDA hasta el fin del proyecto")
    for nombre, n, detalle in saltadas:
        print("  SALTADA  %-18s :%-5d %s" % (nombre, n, detalle))

    fallos_c, nuevas, recorridos = atado_c()
    print("  ATADO C: %d documentos recorridos; la URL del repositorio citada %d vez/veces"
          % (recorridos, nuevas))
    for nombre, razon in sorted(EXCLUIDOS_URL.items()):
        print("  excluido (URL) %s - %s" % (nombre, razon))
    print("  excluido (URL) doc/preprints/ - entrada 28; lo que alli quede se DECLARA")

    fallos_d, n_rfc, n_sitios, n_cuentas = atado_d()
    print("  ATADO D: %d RFC con estado propio; %d cuenta(s) en %d sitio(s), y la fila de "
          "cada uno en %s" % (n_rfc, n_cuentas, n_sitios, FILA_RFC))



    if vistos == 0:
        print("ROJO: CERO citas encontradas.")
        print("      Un censo vacio no es un hallazgo: es un instrumento que")
        print("      dejo de ver. Revisa el patron antes de creerte el cero.")
        return 1

    if fallos:
        print("")
        print("ROJO: %d citas no cuadran con la fuente unica" % len(fallos))
        for nombre, n, clase, detalle in fallos:
            print("  %-18s :%-5d %-7s %s" % (nombre, n, clase, detalle))
        print("")
        print("  La constante vive en crates/zk-ssl/src/metrics.rs.")
        print("  Si el rojo es del SISTEMA y no del documento, quien habla es")
        print("  el otro eslabon: la_cifra_publicada_sigue_siendo_la_medida.")
        return 1

    if fallos_c:
        print("")
        print("ROJO: %d sitio(s) no cuadran con la URL del repositorio" % len(fallos_c))
        for nombre, n, clase, detalle in fallos_c:
            print("  %-40s :%-5d %-19s %s" % (nombre, n, clase, detalle))
        print("")
        print("  La URL vive en URL_REPO, aqui arriba, y en CITATION.cff.")
        return 1

    if fallos_d:
        print("")
        print("ROJO: %d sitio(s) no dicen del RFC lo que el RFC dice de si mismo"
              % len(fallos_d))
        for nombre, n, clase, detalle in fallos_d:
            print("  %-40s :%-5d %-10s %s" % (nombre, n, clase, detalle))
        print("")
        print("  El estado lo declara la linea `- **Estado:**` de cada spec/rfc/NNNN-*.md,")
        print("  y el vocabulario, spec/rfc/PROCESO.md. Si el rojo es del RFC y no del")
        print("  documento, quien se quedo atras es el RFC.")
        return 1

    if nuevas == 0:
        print("ROJO: la URL del repositorio no se cita ni una vez.")
        print("      Un censo vacio no es un hallazgo: revisa URL_REPO y el universo.")
        return 1


    print("OK: todas las citas cuadran en valor y en unidad, y la URL del repositorio es una")
    return 0


if __name__ == "__main__":
    sys.exit(main())
