#!/usr/bin/env python3
"""Guardián de integridad de citas, v2 (AUDITORIA §120, §125).

v1: todo fichero `.md` citado desde `.rs` o `.md` debe existir en el árbol.
v2 (entrada 64): además, toda cita `FICHERO.md §N[.M]` **desde código
`.rs`** debe apuntar a una sección que exista en ese documento. Ámbito
declarado: solo `.rs` — el código debe citar secciones vivas; los
documentos pueden narrar numeraciones viejas (las cabeceras-mapa lo hacen
a propósito, y no son rot).

Uso: python3 tools/verificar_citas.py [raíz] — sale con 1 si hay fantasmas
o secciones muertas.
"""
import re
import sys
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
sys.exit(1 if fallos else 0)
