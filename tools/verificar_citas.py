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

citas = {}
for p in list(ficheros("rs")) + list(ficheros("md")):
    for tok in FICHERO.findall(p.read_text(encoding="utf-8", errors="replace")):
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

print(f"{len(citas)} nombres citados · {sum(1 for t in citas if t not in docs)} fantasmas · {muertas} secciones muertas")
sys.exit(1 if fallos else 0)
