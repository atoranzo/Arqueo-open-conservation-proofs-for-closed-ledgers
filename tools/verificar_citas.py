#!/usr/bin/env python3
"""Guardián de integridad de citas, v2 (AUDITORIA §120, §125).

v1: todo fichero `.md` citado desde `.rs` o `.md` debe existir en el árbol.
v2 (entrada 64): además, toda cita `FICHERO.md §N[.M]` **desde código
`.rs`** debe apuntar a una sección que exista en ese documento. Ámbito
declarado: solo `.rs` — el código debe citar secciones vivas; los
documentos pueden narrar numeraciones viejas (las cabeceras-mapa lo hacen
a propósito, y no son rot).
⚠️ §792: hasta aquí ese ámbito era el único que miraba un NÚMERO de sección
(la v3 mira títulos), y dejaba fuera las citas del registro, «el tejido»
de `AUDITORIA.md` y `BACKLOG.md` (la 66, AUDITORIA §125.4). La v2 sigue
siendo de `.rs`; la v4 mira el registro y excluye por BLOQUE marcado, no
por extensión.
v3 (§616, la 112): la cita por encabezado, `FICHERO.md` §«Título», más abajo.

v3 bis (§831): un fichero de la lista SOLO_POR_ENCABEZADO, que cita su documento por encabezado,
no vuelve a citarlo por línea. Solo los de la lista: los demás no se miran.
  R1  En cada fichero de SOLO_POR_ENCABEZADO, toda cita por línea al documento que declara,
      «RPC.md:<número>» —con o sin la ruta delante y con o sin comilla invertida antes de los dos
      puntos—, es ROJO. Fuera, como en la v5, lo que vaya en un bloque de numeración vieja (v4 R3).
      Hoy son nueve, todos con `spec/RPC.md`. Uno es `doc/USE_CASES.md`, que lo citaba por línea
      en tres sitios: el §389 las escribió bien, el §409 y el §415 movieron las secciones, y
      ninguna caía en su sitio. Los otros ocho lo citan por encabezado y ninguno por línea:
      entran sin tocar su texto.
  R2  PRUEBA DE VIDA: el autotest exige que el patrón vea las formas reales y no tome por cita
      por línea la cita por encabezado ni la marca que narra la línea vieja («`:785-812`»), y
      siembra un texto con una de cada y una dentro de un bloque, y exige que la función que mira
      el árbol vea solo la suya. Cada fichero declarado tiene que citar su documento por
      encabezado al menos una vez: una declaración que no mira nada es ROJO, y un fichero
      declarado que no existe y una lista vacía, también.
  R3  MÍNIMO (§858): la lista tiene al menos MINIMO_SOLO_POR_ENCABEZADO ficheros, nueve, los que
      tenía en 02c546b. Quitar uno es ROJO: hasta el §858 solo bajaba la cuenta del resumen, y la
      regla dejaba de mirar ese fichero en silencio. El sello que quite uno baja el mínimo en el
      mismo diff, con su razón. Añadir uno no obliga a subirlo.
  Ceguera declarada. Solo los ficheros de la lista: las demás citas por línea a `spec/RPC.md` de
  los documentos vivos —53 en 13 ficheros al escribir esto, sin `AUDITORIA.md` ni
  `doc/ecst/VERIFICACION.md`— no se miran, y muchas ya no caen en su sitio. `spec/PAQUETE.md`,
  el RFC-0021 y `BACKLOG.md` citan `spec/RPC.md` de las dos formas y no están. Solo la forma
  «:<número>»: «la línea 860 de `spec/RPC.md`» no se mira. ⚠️ §858: el mínimo cuenta ficheros,
  no cuáles: cambiar uno de la lista por otro no lo ve R3. Lo para R1 si el nuevo cita su
  documento por línea, y R2 si no lo cita por encabezado; en 02c546b, todos los ficheros que citan
  `spec/RPC.md` por encabezado y no están en la lista lo citan también por línea. ⚠️ §827: aquí ya no: el RFC-0022 y el RFC-0023, del §827, lo citan solo por encabezado y no están en la lista; cambiar por uno de ellos un fichero de la lista no lo para ninguna regla (el V2 del §858).

v4 (§792, la 66): las citas §n del REGISTRO. Reglas, declaradas antes de mirar:
  R1  En `AUDITORIA.md` y `BACKLOG.md`, todo «§n» o «§n.m» que no lleve justo
      delante el nombre de otro documento (`SECURITY.md` §3.9, o
      [`PAPER.md`](…) §8.3) cita el registro: n tiene que ser el número de
      un encabezado «## n.» o «## §n» de `AUDITORIA.md`.
  R2  Fuera del registro, en todo `.md` y `.rs`, la cita que lo nombra,
      «AUDITORIA §n» o «`AUDITORIA.md` §n», también.
  R3  MARCA DE BLOQUE: lo que va entre una línea `<!-- numeracion-vieja -->`
      y una `<!-- /numeracion-vieja -->` narra a propósito una numeración
      que no es la del árbol —una cabecera-mapa— y no se mira, ni por R1-R2
      ni por la v5. La marca va sola en su línea (en un `.rs`, tras `//`;
      tras `//!` no se reconoce). Una apertura sin cierre, un cierre sin
      apertura o una apertura dentro de otra son ROJO: una marca rota no se
      traga un fichero en silencio.
  R4  TOLERANCIAS, solo en `AUDITORIA.md`, porque los asientos no se
      reescriben ni se marcan: los números RESERVADOS que el propio registro
      reparte entre sesiones (§631, §740, §770, §800, §830, §860), y dos citas al 0,
      por el asiento que las lleva (TOLERADAS): el 0 de otro documento sin
      nombrarlo y un número provisional que el asiento narra. Se cuentan y se
      imprimen. En `BACKLOG.md` no hay ninguna. Una TOLERADA que ya no se
      usa es ROJO. Un reparto nuevo entra en RESERVADOS con su fuente.
  R5  PRUEBA DE VIDA: un autotest siembra una cita muerta, una de otro
      documento, una con el nombre del registro, una dentro de un bloque y
      un bloque sin cerrar, y exige el veredicto de cada una; y un mínimo de
      encabezados y de citas miradas en cada fichero del registro: un censo
      vacío es un instrumento ciego.
  Ceguera declarada. Mira el n, no el .m: medido en 8c02941, de 620 «§n.m»
  de `AUDITORIA.md` sin otro documento delante, 102 no abren ningún
  encabezado «n.m» del registro, y muchos son la numeración de otro
  documento con su nombre detrás o sin él («§3.11 de `VISION.md`», en la
  sección 13 del registro, `AUDITORIA.md:950`). Una cita a otro documento
  sin su fichero, `X.md`, justo delante y en su misma línea («PRINCIPIOS
  §7», «§3.11 de `VISION.md`», un salto de línea entre `X.md` y §n) se mira
  contra el registro y, con secciones del 1 al 789, casi cualquier n
  pequeño pasa: caza los números que no existen —un hueco, uno futuro, el
  0—, no la cita a una sección equivocada. En un rango
  «§583–587» solo se mira el extremo con §. Los «§n» sueltos de los demás
  documentos no se miran: en `doc/blueprint-v2.md`, «§6» es su sección 6.

v5 (§792): ninguna cita por LÍNEA al `BACKLOG.md` en un documento vivo.
  R1  En todo `.md` y `.rs`, «BACKLOG.md:<número>», con o sin comilla
      invertida antes de los dos puntos y con o sin el `.md` («BACKLOG:2017»
      y «`BACKLOG`:1767», las dos formas sin él que usa `AUDITORIA.md`), es
      ROJO. El `BACKLOG.md` crece por dentro —las notas se añaden en su
      entrada— y la línea citada se mueve con cada sello: hasta el §792, dos
      documentos vivos lo arreglaban con cadenas «hasta el §705, `:N`; desde
      el §706, …», una más por sello. Se cita la ENTRADA, «la entrada 22 de
      `BACKLOG.md`»: su número no se reutiliza ni se renumera.
  R2  Fuera, declarados (EXCLUIDOS_LINEA): `AUDITORIA.md` —un asiento narra
      las líneas de su fecha y no se reescribe— y `doc/ecst/VERIFICACION.md`
      —registro generado de `doc/ecst/borrador/final.json`, salvo sus líneas
      «Lo que se puede afirmar», que son vivas (v5 bis), con las rutas del
      árbol en 71c5aad, que él mismo dice que envejecen—. ⚠️ §847: decía
      «generado entero», y la de M11 lleva a mano el pin de `zk-ssl`. Y lo que
      vaya en un bloque de numeración vieja (v4 R3).
  R3  PRUEBA DE VIDA en los dos sentidos: el patrón reconoce las formas
      reales y no confunde la cita por entrada; y en los ficheros excluidos
      VE alguna (18 al escribir esto): una exclusión que no excluye nada es
      un patrón ciego.
  Ceguera declarada. Solo la forma «:<número>»: «la línea 720 de
  `BACKLOG.md`» o «`BACKLOG.md` (l. 720)» no se miran (en `8c02941` no hay
  ninguna en el árbol). Solo el `BACKLOG.md`: la cita por línea a otro
  fichero se desfasa igual y no se mira —`doc/blueprint-v2.md` tenía tres
  cadenas así, a `spec/rfc/0018-el-tren-0-5.md` y a
  `doc/integracion-vertical-evaluacion.md`, que el §792 pasó a citas por
  encabezado—. ⚠️ §831: salvo en los ficheros de la v3 bis, que la miran para
  su documento. Y solo `.md` y `.rs`: `doc/ecst/borrador/*.json` tiene
  decenas, generadas.

v5 bis (§847): ninguna cita por LÍNEA a `tools/canon.sh` en los documentos vivos de la lista
SIN_LINEA_AL_CANON. La tabla del canon crece por dentro —cada sello alarga la nota de su fila— y
por encima de ella: en ace3243, seis líneas de cuatro documentos vivos lo citaban por línea, y
ninguna caía en su sitio.
  R1  En cada fichero de la lista, y en su ámbito, toda cita «canon.sh:<número>» —con o sin la
      ruta delante y con o sin comilla invertida antes de los dos puntos— es ROJO. El ámbito es el
      fichero entero o, en `doc/ecst/VERIFICACION.md`, las líneas «- **Lo que se puede afirmar:**»,
      donde va a mano el pin de una fila: el autor decidió que el registro es vivo en esto, y el
      ámbito lo propone la sesión. Se cita la fila («la fila `zk-ssl` de la tabla de
      `tools/canon.sh`»), su columna o el bloque («el bloque «3 · conformidad»»). Fuera, como en
      la v3 bis, lo que vaya en un bloque de numeración vieja (v4 R3), y la marca que narra la
      cita vieja («decía `:109`»), que no lleva el nombre.
  R2  PRUEBA DE VIDA, como la v3 bis: el autotest exige que el patrón vea las formas reales y no
      tome por cita por línea la cita por fila, la marca ni la orden «bash tools/canon.sh
      --sello», y siembra un texto con una de cada, una dentro de un bloque y una fuera del ámbito,
      y exige que la función que mira el árbol vea solo la suya. Cada fichero declarado tiene que
      nombrar `tools/canon.sh` en su ámbito al menos una vez: una declaración que no mira nada es
      ROJO, y un fichero declarado que no existe y una lista vacía, también.
  ⚠️ §854: la lista gana `doc/METODO.md`, `doc/blueprint-v2.md`, el RFC-0005 y
  `tools/check_publicadas.py`, enteros: en diecinueve líneas citaban el canon por línea, veinte
  citas y dieciocho rancias, y pasan a citar la fila, la sección o el bloque.
  Ceguera declarada. Solo los ficheros de la lista y su ámbito. Las «Fuentes:» de
  `doc/ecst/VERIFICACION.md` siguen siendo de 71c5aad, y las citas por línea al canon de
  `doc/ecst/borrador/`, `BACKLOG.md` y los demás ficheros de `tools/` no se miran (⚠️ §854:
  decía también `doc/METODO.md`, `doc/blueprint-v2.md` y el RFC-0005, y «los comentarios de
  `tools/`»). Solo la forma «:<número>». Y quitar un fichero de la lista no es ROJO: se ve en
  la cuenta del resumen (⚠️ §858: decía «como en la v3 bis», que desde el §858 tiene su mínimo).

v6 (§805): el número que falta. Un sello escribe «§» y la letra N como marcador del número
que aún no tiene, y lo numera al sellar; el §802 se publicó con 91 líneas sin numerar, en
dieciocho ficheros —SECURITY.md, el código de la capa, la spec, el BACKLOG—, y ninguna regla
lo vio: la v4 solo mira «§» con cifras.
  R1  Todo marcador sin número en un `.md`, `.rs`, `.py`, `.sh`, `.js` o `.toml` del árbol es
      ROJO, salvo los que hablan del marcador mismo, declarados uno a uno con su razón
      (MARCADOR_LEGITIMO): una convención que se describe, no un número que falta. Uno
      declarado que ya no se ve es ROJO.
      Este fichero no se mira: es el que define el marcador.
  R2  PRUEBA DE VIDA: el autotest siembra un marcador sin número, uno numerado y una palabra que
      empieza por N tras el «§», y exige el veredicto de cada uno.

Uso: python3 tools/verificar_citas.py [raíz] — sale con 1 si hay fantasmas,
secciones muertas, encabezados muertos, citas del registro muertas, citas
por línea al `BACKLOG.md` en un documento vivo, una cita por línea en un fichero de
la v3 bis o al canon en uno de la v5 bis, un marcador sin número (v6),
una marca rota o una prueba de vida que no ve.
"""
import re
import sys
from collections import Counter
from pathlib import Path

RAIZ = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
FICHERO = re.compile(r"[A-Za-z0-9_-]+\.md")
SECCION = re.compile(r"([A-Za-z0-9_-]+\.md)`?\s*§\s*(\d+(?:\.\d+)?)")
IGNORAR = {"target", ".git", "node_modules"}


def ficheros(ext):
    for p in RAIZ.rglob(f"*.{ext}"):
        if not IGNORAR & set(p.parts):
            yield p


docs = {}
for p in ficheros("md"):
    docs.setdefault(p.name, p)

fallos = 0

# ⚠️ PLANTILLAS DE NOMBRE, que no son citas. `spec/rfc/PROCESO.md`
# ensena como bautizar una RFC dando el patron del nombre, y el patron
# de arriba lo toma por una cita a un fichero que no existe. Declarado
# y acotado: si aparece otra plantilla, se anade aqui con su fuente.
PLANTILLAS = {"NNNN" + "-titulo-corto.md"}

citas = {}
for p in list(ficheros("rs")) + list(ficheros("md")):
    for tok in FICHERO.findall(p.read_text(encoding="utf-8", errors="replace")):
        if tok in PLANTILLAS:
            continue
        citas.setdefault(tok, []).append(str(p))
for t, ps in sorted(citas.items()):
    if t not in docs:
        print(f"FALTA  {t}  ({len(ps)} citas; p. ej. {ps[0]})")
        fallos += 1

cache = {}
muertas = 0
for p in ficheros("rs"):
    for doc, num in SECCION.findall(p.read_text(encoding="utf-8", errors="replace")):
        if doc not in docs:
            continue  # ya contado como FALTA
        if doc not in cache:
            cache[doc] = docs[doc].read_text(encoding="utf-8", errors="replace")
        texto = cache[doc]
        pat = re.compile(r"(?m)^(?:#{1,6}\s*|\*\*)?%s[\s\.\)]" % re.escape(num))
        if not (pat.search(texto) or ("§" + num) in texto):
            print(f"SECCION MUERTA  {doc} §{num}  (citada en {p})")
            muertas += 1
            fallos += 1

# v3 (§616, la 112): la cita por ENCABEZADO, `FICHERO.md` §«Titulo», desde `.md` y desde `.rs`.
# El titulo tiene que ABRIR un encabezado vivo de ese documento, leido sin sus `#`, sus `**` y sus
# ⚠️, y fuera de los bloques de codigo. Es la forma con que la guia de `spec/README.md` cita
# `spec/RPC.md` desde el §616: los numeros de linea se desfasaban sin que nada lo viera -en
# 221170f, de diecisiete, solo uno caia en su sitio-, y un titulo que se renombra o desaparece
# ahora se NOMBRA aqui. El fichero se resuelve junto al que cita, despues en la raiz, y por
# ultimo por su nombre.
ENCABEZADO = re.compile(r"`([A-Za-z0-9_./-]+\.md)`\s*§\s*«([^»]+)»")


def normal(t):
    return re.sub(r"\s+", " ", t.replace("⚠️", "").replace("**", "")).strip()


def titulos(ruta):
    fuera, dentro = [], False
    for linea in ruta.read_text(encoding="utf-8", errors="replace").splitlines():
        if linea.lstrip().startswith("```"):
            dentro = not dentro
            continue
        if not dentro and linea.startswith("#"):
            fuera.append(normal(linea.lstrip("#")))
    return fuera


encabezados = 0
rotos = 0
cache_t = {}
for p in list(ficheros("rs")) + list(ficheros("md")):
    for doc, titulo in ENCABEZADO.findall(p.read_text(encoding="utf-8", errors="replace")):
        ruta = next((c for c in (p.parent / doc, RAIZ / doc) if c.is_file()),
                    docs.get(Path(doc).name))
        if ruta is None:
            continue  # ya contado como FALTA
        encabezados += 1
        if ruta not in cache_t:
            cache_t[ruta] = titulos(ruta)
        if not any(h.startswith(normal(titulo)) for h in cache_t[ruta]):
            print(f"ENCABEZADO MUERTO  {doc} §«{titulo}»  (citado en {p})")
            rotos += 1
            fallos += 1

print(f"{len(citas)} nombres citados · {sum(1 for t in citas if t not in docs)} fantasmas · {muertas} secciones muertas · {encabezados} encabezados citados, {rotos} muertos")

# v4 (§792, la 66) y v5 (§792): las reglas, en la cabecera. Aqui, lo que las hace.
ABRE = re.compile(r"^\s*(?://+|#)?\s*<!-- numeracion-vieja -->\s*$")
CIERRA = re.compile(r"^\s*(?://+|#)?\s*<!-- /numeracion-vieja -->\s*$")
ENCAB_REG = re.compile(r"^##\s+(?:§\s*)?(\d+)")
CITA_REG = re.compile(r"§\s*(\d+)")
# el nombre de un documento JUSTO delante: `X.md` §n, X.md §n, [`X.md`](ruta) §n
DELANTE = re.compile(r"([A-Za-z0-9_-]+)\.md`?(?:\]\([^)]*\))?\s*$")
CITA_AUD = re.compile(r"AUDITORIA(?:\.md)?`?(?:\]\([^)]*\))?\s*§\s*(\d+)")
LINEA_BACKLOG = re.compile(r"BACKLOG(?:\.md)?`?:\d")
REGISTRO = ("AUDITORIA.md", "BACKLOG.md")
MINIMO_SECCIONES = 700  # medido en 8c02941: 730 numeros distintos
MINIMO_CITAS = {"AUDITORIA.md": 5000, "BACKLOG.md": 800}  # medido en 8c02941: 5.844 y 962

# ⚠️ RESERVADOS: los numeros que el registro reparte entre sesiones y que este arbol no tiene
# todavia. Solo valen en `AUDITORIA.md`, donde los asientos narran el reparto.
#   630        «el §630 es el de la otra sesion» (§631); «su proximo numero libre es el §630»
#              (§629)
#   711-739    «§711 a §739 la de la migracion» (§740, §770)
#   741-769    «§741 a §769 la del medio» (§770)
#   790-799    «§770 a §799 esta» (§770): lo que queda del rango de la sesion que lo escribe
#   800-829    «§800 a §829», el rango que el autor asigno a la misma sesion para lo que faltaba
#              (§800): lo que queda de el
#   830-859    «§830 a §859», el rango que el autor asigno para lo que queda de la lista de tareas
#              de la sesion que lo escribe (§830): lo que queda de el
#   860-889    «§860 a §889», el segundo rango que el autor asigno a la sesion del §800 (§860):
#              lo que queda de el
RESERVADOS = {630} | set(range(711, 740)) | set(range(741, 770)) | set(range(790, 800)) | \
    set(range(800, 830)) | set(range(830, 860)) | set(range(860, 890))
# ⚠️ TOLERADAS: (seccion que la lleva, numero citado). Citas que no nombran una seccion del
# registro, en asientos, que no se reescriben.
TOLERADAS = {
    (235, 0): "«el §0 de la evaluación»: la sección 0 de otro documento",
    (594, 0): "«§0-ECST»: el número provisional que, narra el asiento, pasó a ser §594",
}
# ⚠️ v5 R2: fuera de la regla de la cita por linea al BACKLOG, con su razon.
# ⚠️ §847: la de VERIFICACION decia «registro generado» sin mas; sus lineas «Lo que se puede
# afirmar» son vivas (v5 bis).
EXCLUIDOS_LINEA = {
    "AUDITORIA.md": "registro: un asiento narra las líneas de su fecha y no se reescribe",
    "doc/ecst/VERIFICACION.md":
        "registro generado de doc/ecst/borrador/final.json salvo sus líneas «Lo que se puede "
        "afirmar», vivas (v5 bis); rutas de 71c5aad",
}


def vivas(texto, donde, rojo, cuenta):
    """Las lineas fuera de los bloques de numeracion vieja, con su numero (v4 R3)."""
    abierto = 0
    for i, linea in enumerate(texto.splitlines(), 1):
        if ABRE.match(linea):
            if abierto:
                rojo.append(f"MARCA ROTA  {donde}:{i}: abre un bloque dentro del de la línea "
                            f"{abierto}")
            else:
                abierto = i
                cuenta["bloques"] += 1
            continue
        if CIERRA.match(linea):
            if not abierto:
                rojo.append(f"MARCA ROTA  {donde}:{i}: cierra un bloque que no se abrió")
            abierto = 0
            continue
        if not abierto:
            yield i, linea
    if abierto:
        rojo.append(f"MARCA ROTA  {donde}:{abierto}: el bloque no se cierra")


def del_registro(texto, donde, secciones, rojo, cuenta):
    """v4 R1: las citas §n de un fichero del registro que no nombran un encabezado.
    Da (linea, seccion que la lleva, n)."""
    actual = None
    for i, linea in vivas(texto, donde, rojo, cuenta):
        m = ENCAB_REG.match(linea)
        if m:
            actual = int(m.group(1))
        for c in CITA_REG.finditer(linea):
            otro = DELANTE.search(linea[:c.start()])
            if otro and otro.group(1) != "AUDITORIA":
                cuenta["de otro documento"] += 1
                continue
            cuenta["citas"] += 1
            n = int(c.group(1))
            if n not in secciones:
                yield i, actual, n


def autotest():
    """v4 R5 y v5 R3: el instrumento ve lo que tiene que ver."""
    mal = []
    sembrado = "\n".join([
        "## 1. uno",
        "viva §2, viva §3.4",
        "muerta §9999",
        "de otro documento: `SECURITY.md` §9998 y [`PAPER.md`](PAPER.md) §9998",
        "con el nombre del registro: `AUDITORIA.md` §9997",
        "<!-- numeracion-vieja -->",
        "narrada a proposito: la antigua §9996",
        "<!-- /numeracion-vieja -->",
    ])
    rojo, cuenta = [], Counter()
    vistas = [n for _, _, n in del_registro(sembrado, "autotest", {1, 2, 3}, rojo, cuenta)]
    if vistas != [9999, 9997] or rojo or cuenta["bloques"] != 1 or cuenta["de otro documento"] != 2:
        mal.append(f"v4: el autotest da {vistas}, {rojo}, {dict(cuenta)}; "
                   "esperaba [9999, 9997], sin rojo")
    abre, cierra = "<!-- numeracion-vieja -->", "<!-- /numeracion-vieja -->"
    for roto in (abre + "\n§1", cierra, abre + "\n" + abre + "\n" + cierra):
        rojo = []
        list(vivas(roto, "autotest", rojo, Counter()))
        if not rojo:
            mal.append(f"v4: una marca rota no da ROJO: {roto!r}")
    for si in ("`BACKLOG.md:720`", "BACKLOG.md:1401-1408", "`BACKLOG.md`:2019",
               "BACKLOG:2017", "`BACKLOG`:1767"):
        if not LINEA_BACKLOG.search(si):
            mal.append(f"v5: el patron no ve {si}")
    for no in ("la entrada 85 de `BACKLOG.md`", "`BACKLOG.md` §«Estado»", "BACKLOG.md: la 66",
               "BACKLOG 92"):
        if LINEA_BACKLOG.search(no):
            mal.append(f"v5: el patron toma por cita por linea {no}")
    return mal


vida = autotest()
for v in vida:
    print(f"PRUEBA DE VIDA  {v}")
    fallos += 1

reg_rojo, reg_cuenta = [], Counter()
auditoria = (RAIZ / "AUDITORIA.md").read_text(encoding="utf-8", errors="replace")
secciones = {int(m.group(1)) for m in map(ENCAB_REG.match, auditoria.splitlines()) if m}
if len(secciones) < MINIMO_SECCIONES:
    reg_rojo.append(f"PRUEBA DE VIDA  {len(secciones)} encabezados en AUDITORIA.md, "
                    f"menos de {MINIMO_SECCIONES}")
reg_muertas, toleradas_usadas, reservadas = 0, Counter(), 0
for nombre in REGISTRO:
    texto = (auditoria if nombre == "AUDITORIA.md"
             else (RAIZ / nombre).read_text(encoding="utf-8", errors="replace"))
    antes = reg_cuenta["citas"]
    for i, sec, n in del_registro(texto, nombre, secciones, reg_rojo, reg_cuenta):
        if nombre == "AUDITORIA.md" and (sec, n) in TOLERADAS:
            toleradas_usadas[(sec, n)] += 1
        elif nombre == "AUDITORIA.md" and n in RESERVADOS:
            reservadas += 1
        else:
            reg_rojo.append(f"CITA DEL REGISTRO MUERTA  {nombre}:{i}: §{n} no es ningún "
                            "encabezado de AUDITORIA.md")
            reg_muertas += 1
    if reg_cuenta["citas"] - antes < MINIMO_CITAS[nombre]:
        reg_rojo.append(f"PRUEBA DE VIDA  {nombre}: {reg_cuenta['citas'] - antes} citas miradas, "
                        f"menos de {MINIMO_CITAS[nombre]}")
for clave, razon in TOLERADAS.items():
    if clave not in toleradas_usadas:
        reg_rojo.append(f"TOLERADA SIN USO  §{clave[1]} en el §{clave[0]} ({razon}): "
                        "se quita de TOLERADAS")

fuera_aud, fuera_muertas = 0, 0
excluidas_vistas, en_vivos = Counter(), 0
registro = {(RAIZ / n).resolve() for n in REGISTRO}
for p in list(ficheros("rs")) + list(ficheros("md")):
    rel = p.relative_to(RAIZ).as_posix()
    texto = p.read_text(encoding="utf-8", errors="replace")
    del_reg = p.resolve() in registro
    # las marcas del registro ya las juzgo el bucle de arriba: aqui no se cuentan dos veces
    lineas = list(vivas(texto, rel, [], Counter()) if del_reg
                  else vivas(texto, rel, reg_rojo, reg_cuenta))
    if not del_reg:
        for i, linea in lineas:
            for m in CITA_AUD.finditer(linea):
                fuera_aud += 1
                if int(m.group(1)) not in secciones:
                    reg_rojo.append(f"CITA DEL REGISTRO MUERTA  {rel}:{i}: AUDITORIA §{m.group(1)} "
                                    "no es ningún encabezado")
                    fuera_muertas += 1
    for i, linea in lineas:
        if LINEA_BACKLOG.search(linea):
            if rel in EXCLUIDOS_LINEA:
                excluidas_vistas[rel] += 1
            else:
                reg_rojo.append(f"CITA POR LINEA AL BACKLOG  {rel}:{i}: "
                                "se cita la entrada, no la línea")
                en_vivos += 1
if not excluidas_vistas:
    reg_rojo.append("PRUEBA DE VIDA  v5: ninguna cita por línea en los ficheros excluidos: "
                    "el patrón está ciego")

# ⚠️ v3 bis (§831): en un fichero de esta lista, que cita su documento por ENCABEZADO, ninguna
# cita por LINEA a ese documento. Los demás ficheros no se miran. (fichero, documento que cita):
# la razon va en la cabecera.
SOLO_POR_ENCABEZADO = {
    "doc/USE_CASES.md": "spec/RPC.md",
    "spec/README.md": "spec/RPC.md",
    "README.md": "spec/RPC.md",
    "README_EN.md": "spec/RPC.md",
    "doc/KIT.md": "spec/RPC.md",
    "doc/KIT_EN.md": "spec/RPC.md",
    "spec/NUCLEO.md": "spec/RPC.md",
    "spec/rfc/0018-el-tren-0-5.md": "spec/RPC.md",
    "tools/segunda/README.md": "spec/RPC.md",
}
# ⚠️ v3 bis R3 (§858): medido en 02c546b, los nueve de arriba. Quitar uno baja este minimo en el
# mismo diff, con su razon.
MINIMO_SOLO_POR_ENCABEZADO = 9


def linea_a(doc):
    """La cita por linea a `doc`: `spec/RPC.md:785-812`, spec/RPC.md`:860, RPC.md:88."""
    return re.compile(re.escape(Path(doc).name) + r"`?:\d")


def por_linea(texto, rel, doc):
    """v3 bis R1: las lineas (1-based) de `texto` que citan `doc` por linea."""
    pat = linea_a(doc)
    return [i for i, linea in vivas(texto, rel, [], Counter()) if pat.search(linea)]


def autotest_v3bis():
    mal = []
    pat = linea_a("spec/RPC.md")
    sembrado = "\n".join(["`spec/RPC.md` §«Notas operativas»", "`spec/RPC.md:785-812`",
                           "<!-- numeracion-vieja -->", "`spec/RPC.md:860`",
                           "<!-- /numeracion-vieja -->", "decía `:860-871`"])
    if por_linea(sembrado, "autotest", "spec/RPC.md") != [2]:
        mal.append(f"v3 bis: el sembrado da {por_linea(sembrado, 'autotest', 'spec/RPC.md')}, "
                   "se esperaba [2]")
    for si in ("`spec/RPC.md:785-812`", "`spec/RPC.md`:860", "RPC.md:88", "(spec/RPC.md:568-739,"):
        if not pat.search(si):
            mal.append(f"v3 bis: el patron no ve {si}")
    for no in ("`spec/RPC.md` §«Notas operativas»", "until here it cited lines, `:785-812`",
               "`spec/RPC.md`, its first item: 3", "`openrpc.json`:12"):
        if pat.search(no):
            mal.append(f"v3 bis: el patron toma por cita por linea {no}")
    return mal


for v in autotest_v3bis():
    reg_rojo.append(f"PRUEBA DE VIDA  {v}")
if not SOLO_POR_ENCABEZADO:
    reg_rojo.append("PRUEBA DE VIDA  v3 bis: SOLO_POR_ENCABEZADO está vacía: la regla no mira nada")
if len(SOLO_POR_ENCABEZADO) < MINIMO_SOLO_POR_ENCABEZADO:
    reg_rojo.append(f"PRUEBA DE VIDA  v3 bis: SOLO_POR_ENCABEZADO tiene {len(SOLO_POR_ENCABEZADO)} "
                    f"ficheros, menos de {MINIMO_SOLO_POR_ENCABEZADO}: quien quita uno baja "
                    "MINIMO_SOLO_POR_ENCABEZADO con su razón")
v3bis_lineas, v3bis_encab = 0, {}
for rel, doc in SOLO_POR_ENCABEZADO.items():
    p = RAIZ / rel
    if not p.is_file():
        reg_rojo.append(f"SOLO POR ENCABEZADO SIN FICHERO  {rel}: se quita de SOLO_POR_ENCABEZADO")
        continue
    texto = p.read_text(encoding="utf-8", errors="replace")
    for i in por_linea(texto, rel, doc):
        reg_rojo.append(f"CITA POR LINEA  {rel}:{i}: a {doc}, que este fichero cita por "
                        "encabezado: se cita el encabezado")
        v3bis_lineas += 1
    v3bis_encab[rel] = sum(1 for d, _ in ENCABEZADO.findall(texto)
                           if Path(d).name == Path(doc).name)
    if not v3bis_encab[rel]:
        reg_rojo.append(f"PRUEBA DE VIDA  v3 bis: {rel} no cita {doc} por encabezado: "
                        "la declaración no mira nada")

# ⚠️ v5 bis (§847): en un documento vivo de esta lista, ninguna cita por LINEA a `tools/canon.sh`.
# (fichero, prefijo de las lineas que mira; None, el fichero entero): la razon va en la cabecera.
CANON = "tools/canon.sh"
SIN_LINEA_AL_CANON = {
    "README.md": None,
    "README_EN.md": None,
    "RESUMEN_EJECUTIVO.md": None,
    "doc/ecst/VERIFICACION.md": "- **Lo que se puede afirmar:**",
    # ⚠️ §854: cuatro más, enteros; en el .py, los comentarios y los mensajes que imprime.
    "doc/METODO.md": None,
    "doc/blueprint-v2.md": None,
    "spec/rfc/0005-nucleo-congelado.md": None,
    "tools/check_publicadas.py": None,
}


def al_canon(texto, rel, prefijo):
    """v5 bis R1: (lineas que nombran el canon, lineas que lo citan por linea), en el ambito."""
    pat, nombran, citan = linea_a(CANON), [], []
    for i, linea in vivas(texto, rel, [], Counter()):
        if prefijo is not None and not linea.startswith(prefijo):
            continue
        if Path(CANON).name in linea:
            nombran.append(i)
        if pat.search(linea):
            citan.append(i)
    return nombran, citan


def autotest_v5bis():
    mal = []
    pat = linea_a(CANON)
    sembrado = "\n".join(["- **Lo que se puede afirmar:** la fila `zk-ssl` de `tools/canon.sh`",
                           "- **Lo que se puede afirmar:** fija 100 (tools/canon.sh:109)",
                           "- **Fuentes:** `tools/canon.sh:89: pin actual`",
                           "<!-- numeracion-vieja -->",
                           "- **Lo que se puede afirmar:** `tools/canon.sh`:128-130",
                           "<!-- /numeracion-vieja -->",
                           "- **Lo que se puede afirmar:** ⚠️ §847: decía `:109`"])
    prefijo = SIN_LINEA_AL_CANON.get("doc/ecst/VERIFICACION.md")
    if al_canon(sembrado, "autotest", prefijo) != ([1, 2], [2]):
        mal.append(f"v5 bis: el sembrado da {al_canon(sembrado, 'autotest', prefijo)}, "
                   "se esperaba ([1, 2], [2])")
    if al_canon(sembrado, "autotest", None)[1] != [2, 3]:
        mal.append("v5 bis: sin prefijo, el sembrado no da las lineas [2, 3]")
    for si in ("(tools/canon.sh:109)", "`tools/canon.sh`:404-407", "`tools/canon.sh:128-130`",
               "canon.sh:97"):
        if not pat.search(si):
            mal.append(f"v5 bis: el patron no ve {si}")
    for no in ("la fila `zk-ssl` de la tabla de `tools/canon.sh`", "decía `:128-130`, `:154-158`",
               "bash tools/canon.sh --sello", "`tools/canon.sh`: la tabla",
               # ⚠️ §854: la cita por fila y la marca de un comentario de tools/, sin tilde
               "(la fila winter-air de tools/canon.sh; ⚠️ §854: decia :123)-"):
        if pat.search(no):
            mal.append(f"v5 bis: el patron toma por cita por linea {no}")
    return mal


for v in autotest_v5bis():
    reg_rojo.append(f"PRUEBA DE VIDA  {v}")
if not SIN_LINEA_AL_CANON:
    reg_rojo.append("PRUEBA DE VIDA  v5 bis: SIN_LINEA_AL_CANON está vacía: la regla no mira nada")
v5bis_lineas, v5bis_nombran = 0, {}
for rel, prefijo in SIN_LINEA_AL_CANON.items():
    p = RAIZ / rel
    if not p.is_file():
        reg_rojo.append(f"SIN LINEA AL CANON SIN FICHERO  {rel}: se quita de SIN_LINEA_AL_CANON")
        continue
    nombran, citan = al_canon(p.read_text(encoding="utf-8", errors="replace"), rel, prefijo)
    for i in citan:
        reg_rojo.append(f"CITA POR LINEA AL CANON  {rel}:{i}: se cita la fila, su columna o el "
                        "bloque de tools/canon.sh, no la línea")
        v5bis_lineas += 1
    v5bis_nombran[rel] = len(nombran)
    if not nombran:
        reg_rojo.append(f"PRUEBA DE VIDA  v5 bis: {rel} no nombra {CANON} en su ámbito: "
                        "la declaración no mira nada")

# ⚠️ v6 (§805): el marcador sin número. El patrón se compone para que este fichero no lo lleve.
SIN_NUMERO = re.compile("\u00a7" + r"N\b")
EXTENSIONES_V6 = ("md", "rs", "py", "sh", "js", "toml")
# (fichero, un trozo de la línea que lo lleva): razón. Hablan del marcador; no les falta un número.
MARCADOR_LEGITIMO = {
    ("AUDITORIA.md", "cita de la forma «*fichero* `.md` \u00a7N» desde `.rs`"):
        "un asiento describe la forma de una cita, la v2 de esta herramienta",
    ("AUDITORIA.md", "por su encabezado `## \u00a7N`"):
        "un asiento describe la forma del encabezado de un asiento",
    ("AUDITORIA.md", "las cadenas «desde el \u00a7N» de"):
        "un asiento describe la forma de una cadena de citas",
    ("doc/blueprint-v2.md", "(la sesión, o el \u00a7N del registro)"):
        "la convención del documento: de dónde sale una medida",
    ("doc/ecst/ECST.md", "(`AUDITORIA.md` \u00a7N, o el README"):
        "la convención del informe, fechado: de dónde sale una medida",
    ("doc/ecst/ECST_EN.md", "(`AUDITORIA.md` \u00a7N, or"):
        "la convención del informe, en inglés",
    ("doc/ecst/borrador/flujo-verificacion.js", "entries are \"\u00a7N\""):
        "el guion que generó el informe describe los asientos",
    ("doc/ecst/borrador/flujo-verificacion.js", "\"AUDITORIA.md \u00a7N\", test names"):
        "el guion que generó el informe describe una cita",
}


def sin_numero(texto):
    """Las líneas (1-based) con un marcador sin número."""
    return [(i, linea) for i, linea in enumerate(texto.split("\n"), 1) if SIN_NUMERO.search(linea)]


def autotest_v6():
    mal = []
    for sembrado, esperado in (("\u00a7" + "N: hasta aquí decía", 1),
                               ("el \u00a7802 abre la 119", 0),
                               ("\u00a7" + "Nota aparte", 0), ("desde el \u00a7" + "N.", 1)):
        if len(sin_numero(sembrado)) != esperado:
            mal.append(f"v6: «{sembrado}» da {len(sin_numero(sembrado))}, se esperaba {esperado}")
    return mal


for v in autotest_v6():
    reg_rojo.append(f"PRUEBA DE VIDA  {v}")
legitimos_vistos, sin_numero_vivos = Counter(), 0
for ext in EXTENSIONES_V6:
    for p in ficheros(ext):
        rel = p.relative_to(RAIZ).as_posix()
        if rel == "tools/verificar_citas.py":
            continue
        for i, linea in sin_numero(p.read_text(encoding="utf-8", errors="replace")):
            claves = [k for k in MARCADOR_LEGITIMO if k[0] == rel and k[1] in linea]
            if claves:
                legitimos_vistos[claves[0]] += 1
            else:
                reg_rojo.append(f"MARCADOR SIN NUMERO  {rel}:{i}: "
                                "el sello que lo escribió no lo numeró")
                sin_numero_vivos += 1
for clave, razon in MARCADOR_LEGITIMO.items():
    if clave not in legitimos_vistos:
        reg_rojo.append(f"MARCADOR DECLARADO SIN USO  {clave[0]} ({razon}): "
                        "se quita de MARCADOR_LEGITIMO")

for r in reg_rojo:
    print(r)
fallos += len(reg_rojo)
print(f"registro (v4): {len(secciones)} encabezados · {reg_cuenta['citas']} citas § miradas en "
      f"AUDITORIA.md y BACKLOG.md, {reg_muertas} muertas, {sum(toleradas_usadas.values())} "
      f"toleradas, {reservadas} a números reservados, {reg_cuenta['de otro documento']} de otro "
      f"documento · {fuera_aud} «AUDITORIA §n» fuera del registro, {fuera_muertas} muertas · "
      f"{reg_cuenta['bloques']} bloques de numeración vieja")
print(f"línea al BACKLOG (v5): {en_vivos} en documentos vivos · "
      + ", ".join(f"{k} {v}" for k, v in sorted(excluidas_vistas.items())) + " (excluidos)")
print(f"línea a un documento citado por encabezado (v3 bis): {v3bis_lineas} · "
      f"{len(v3bis_encab)} ficheros (mínimo {MINIMO_SOLO_POR_ENCABEZADO}), "
      f"{sum(v3bis_encab.values())} citas por encabezado a su documento ("
      + ", ".join(f"{r} {n}" for r, n in sorted(v3bis_encab.items())) + ")")
print(f"línea al canon en un documento vivo (v5 bis): {v5bis_lineas} · {len(v5bis_nombran)} "
      "ficheros, que lo nombran en su ámbito (" + ", ".join(f"{r} {n}" for r, n in
                                                 sorted(v5bis_nombran.items())) + ")")
print(f"marcador sin número (v6): {sin_numero_vivos} en el árbol · "
      f"{sum(legitimos_vistos.values())} que hablan del marcador, declarados")
sys.exit(1 if fallos else 0)
