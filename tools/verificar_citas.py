#!/usr/bin/env python3
"""Guardián de integridad de citas (AUDITORIA §119.1).

Todo fichero `.md` citado desde código (`.rs`) o documentación (`.md`)
debe existir en el árbol. Nació de encontrar 24 citas a dos documentos
inexistentes — y en su primera ejecución cazó una tercera, del propio
proceso que lo escribió.

Ámbito declarado: solo `.rs` y `.md` son fuentes de citas — un token
dentro de un `.sh` ("$base.md") no es una cita, y ese falso positivo de
la versión de una línea muere aquí de fábrica.

Uso: python3 tools/verificar_citas.py [raíz]   → sale con 1 si hay fantasmas.
"""
import re
import sys
from pathlib import Path

RAIZ = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
PATRON = re.compile(r"[A-Za-z0-9_-]+\.md")
IGNORAR = {"target", ".git", "node_modules"}


def ficheros(ext):
    for p in RAIZ.rglob(f"*.{ext}"):
        if not IGNORAR & set(p.parts):
            yield p


existentes = {p.name for p in ficheros("md")}
citas = {}
for p in list(ficheros("rs")) + list(ficheros("md")):
    texto = p.read_text(encoding="utf-8", errors="replace")
    for tok in PATRON.findall(texto):
        citas.setdefault(tok, []).append(str(p))

fantasmas = {t: ps for t, ps in sorted(citas.items()) if t not in existentes}
for t, ps in fantasmas.items():
    print(f"FALTA  {t}  ({len(ps)} citas; p. ej. {ps[0]})")
print(f"{len(citas)} nombres citados · {len(fantasmas)} fantasmas")
sys.exit(1 if fantasmas else 0)
