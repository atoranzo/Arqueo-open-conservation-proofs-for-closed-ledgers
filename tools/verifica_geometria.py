#!/usr/bin/env python3
# Verificación mecánica de la geometría de circuit_send.rs (HEAD 1cbfedc).
# Reconstruye el calendario fila a fila desde las constantes EXACTAS del
# código y cruza las tres representaciones de §139. No confía: cuenta.

CYCLE = 8
NUM_ROUNDS = 7
TRACE_LENGTH = 1024
TREE_DEPTH = 32     # merkle.rs:55
FROZEN_DEPTH = 24   # circuit_freeze.rs:61

# R1 — constantes de fila (circuit_send.rs:143-167)
ROW_LEAF_LINK = 7
ROW_LEAF_DONE = 15
ROW_ROOT = 271
ROW_PK_START = 272
ROW_PK_DONE = 279
ROW_FROZEN_ROOT = 471
ROW_PEND_INNER = 479
ROW_PENDING_ENTRY = 487
ROW_PENDING_ROOT = 743

EXPLICIT = {
    ROW_LEAF_LINK: "LEAF_LINK  (siembra nonce)",
    ROW_LEAF_DONE: "LEAF_DONE  (coloca hoja, nivel 0 cuentas)",
    ROW_ROOT: "ROOT       (raíces A/B; siembra dominio+clave)",
    ROW_PK_DONE: "PK_DONE    (titularidad; coloca cero, nivel 0 frozen)",
    ROW_FROZEN_ROOT: "FROZEN_ROOT(raíz frozen; siembra id_receptor+salt)",
    ROW_PEND_INNER: "PEND_INNER (siembra interno+importe)",
    ROW_PENDING_ENTRY: "PEND_ENTRY (A cero / B compromiso, nivel 0 pend)",
}

def brazo(r, rows_shift=0, offsets_shift=0):
    """Replica el `match r` (circuit_send.rs:410-478).
    rows_shift: simula mover las ROW_* (+8 en los intentos).
    offsets_shift: simula mover los offsets -2/-35/-61 (+1 en el intento 2).
    Devuelve (tipo, nivel|None). Lanza IndexError como place_* en Rust."""
    ex = {k + rows_shift: v for k, v in EXPLICIT.items()}
    if r in ex:
        return ("explicito", ex[r])
    nc = (r + 1) // CYCLE
    a, f, p = 2 + offsets_shift, 35 + offsets_shift, 61 + offsets_shift
    if a <= nc < a + 32:            # (2..34) en el original
        lvl = nc - a
        if lvl >= TREE_DEPTH:
            raise IndexError(f"place: nivel {lvl} sobre path de {TREE_DEPTH} (fila {r})")
        return ("cuentas", lvl)
    if f + 1 <= nc < f + 25:        # (36..60) en el original
        lvl = nc - f
        if lvl >= FROZEN_DEPTH:
            raise IndexError(f"place_frozen: nivel {lvl} sobre path de {FROZEN_DEPTH} (fila {r})")
        return ("frozen", lvl)
    if p + 1 <= nc < p + 33:        # (62..94) en el original
        lvl = nc - p
        if lvl >= TREE_DEPTH:
            raise IndexError(f"place_pending: nivel {lvl} sobre path de {TREE_DEPTH} (fila {r})")
        return ("pendientes", lvl)
    return ("VACIO", None)          # ningún brazo: estado queda a cero

print("=" * 72)
print("A. CALENDARIO — un evento por fila de enlace (r = 8c+7), árbol real")
print("=" * 72)
cuentas, frozen, pend, vacios = [], [], [], []
for r in range(0, ROW_PENDING_ROOT):        # bucle EXACTO: for r in 0..ROW_PENDING_ROOT
    if r % CYCLE == NUM_ROUNDS:
        t, x = brazo(r)
        c = r // CYCLE
        if t == "cuentas": cuentas.append((r, x))
        elif t == "frozen": frozen.append((r, x))
        elif t == "pendientes": pend.append((r, x))
        elif t == "VACIO": vacios.append(r)

print(f"cuentas   genérico: niveles {cuentas[0][1]}..{cuentas[-1][1]} en filas {cuentas[0][0]}..{cuentas[-1][0]}  ({len(cuentas)} colocaciones; nivel 0 explícito en {ROW_LEAF_DONE})")
print(f"frozen    genérico: niveles {frozen[0][1]}..{frozen[-1][1]} en filas {frozen[0][0]}..{frozen[-1][0]}  ({len(frozen)} colocaciones; nivel 0 explícito en {ROW_PK_DONE})")
print(f"pendiente genérico: niveles {pend[0][1]}..{pend[-1][1]} en filas {pend[0][0]}..{pend[-1][0]}  ({len(pend)} colocaciones; nivel 0 explícito en {ROW_PENDING_ENTRY})")
print(f"filas de enlace sin brazo (estado a cero): {vacios if vacios else 'NINGUNA'}")

assert [l for _, l in cuentas] == list(range(1, TREE_DEPTH)),  "cuentas: niveles 1..31 exactos"
assert [l for _, l in frozen] == list(range(1, FROZEN_DEPTH)), "frozen: niveles 1..23 exactos"
assert [l for _, l in pend] == list(range(1, TREE_DEPTH)),     "pendiente: niveles 1..31 exactos"
assert not vacios, "ninguna fila de enlace queda sin brazo"
print("✔ cada nivel de cada árbol se coloca EXACTAMENTE una vez; sin filas huérfanas")

print()
print("=" * 72)
print("B. GUARDIANES DE FRONTERA — quién impide cada valor de rango ilegal")
print("=" * 72)
def quien_guarda(nc):
    r = nc * CYCLE - 1
    if r >= ROW_PENDING_ROOT:
        return f"nc={nc} (fila {r}): FUERA por el LÍMITE DEL BUCLE (0..ROW_PENDING_ROOT)"
    if r in EXPLICIT:
        return f"nc={nc} (fila {r}): SOMBREADO por brazo explícito {EXPLICIT[r].split()[0]}"
    return f"nc={nc} (fila {r}): dentro del rango genérico"
for nc in [2, 33, 34, 35, 36, 58, 59, 60, 61, 62, 92, 93, 94]:
    print("  " + quien_guarda(nc))

print()
print("=" * 72)
print("C. CRUCE R1×R2 — bucles de bits y periódicas contra el calendario")
print("=" * 72)
# Bits: el bit del nivel l vive en el ciclo que HASHEA el nivel l
# (build_trace:485-516). La restricción C_PLACE lee next[COL_BIT] en la
# transición r→r+1 de la colocación: r+1 debe caer dentro de ese ciclo.
ok = True
for (arranque, prof, nombre, fila_n0) in [(2, TREE_DEPTH, "COL_BIT ", ROW_LEAF_DONE),
                                          (35, FROZEN_DEPTH, "COL_FBIT", ROW_PK_DONE),
                                          (61, TREE_DEPTH, "COL_PBIT", ROW_PENDING_ENTRY)]:
    filas_bit = {l: range((arranque + l) * CYCLE, (arranque + l) * CYCLE + CYCLE) for l in range(prof)}
    # nivel 0: colocación explícita
    ok &= (fila_n0 + 1) in filas_bit[0]
    # niveles 1..prof-1: colocación genérica en fila (arranque+l)*8-1
    for l in range(1, prof):
        ok &= ((arranque + l) * CYCLE - 1 + 1) in filas_bit[l]
    print(f"  {nombre}: bit del nivel l en filas ({arranque}+l)*8..+7 — cubre la fila `next` de su colocación ✔")
assert ok
# Periódicas de enlace (get_periodic_column_values:691-760)
lm = [(2 + l) * CYCLE + 7 for l in range(TREE_DEPTH - 1)]
fl = [(35 + l) * CYCLE + 7 for l in range(FROZEN_DEPTH - 1)]
pl = [(61 + l) * CYCLE + 7 for l in range(TREE_DEPTH - 1)]
assert lm == [ROW_LEAF_DONE + CYCLE] + [r for r, _ in cuentas][:-1] + [] or True
assert lm == [r for r, _ in cuentas] + [] if False else True
# comparación directa: enlaces genéricos = filas de colocación de niveles 1..prof-1
assert lm == [r for r, _ in cuentas], "link_merkle == colocaciones genéricas de cuentas"
assert fl == [r for r, _ in frozen], "frozen_link == colocaciones genéricas frozen"
assert pl == [r for r, _ in pend],   "pend_link == colocaciones genéricas pendiente"
print(f"  link_merkle ({len(lm)} unos) == filas de colocación cuentas 1..31 ✔")
print(f"  frozen_link ({len(fl)} unos) == filas de colocación frozen  1..23 ✔")
print(f"  pend_link   ({len(pl)} unos) == filas de colocación pend.   1..31 ✔")
print("  (el último ciclo de cada subida NO lleva enlace: su salida es la raíz, atada por aserción)")

print()
print("=" * 72)
print("D. RECONSTRUCCIÓN DE LOS DOS INTENTOS FALLIDOS (§139)")
print("=" * 72)
for nombre, rs, os_ in [("INTENTO 1: ROW_* +8, offsets viejos, rangos viejos", CYCLE, 0),
                        ("INTENTO 2: ROW_* +8 y offsets +1, rangos viejos", CYCLE, 1)]:
    try:
        for r in range(0, ROW_PENDING_ROOT + rs):
            if r % CYCLE == NUM_ROUNDS:
                brazo(r, rows_shift=rs, offsets_shift=0)  # rangos y offsets del match: SIN tocar
        print(f"  {nombre}: no rompió (??)")
    except IndexError as e:
        print(f"  {nombre}:\n      💥 {e}")
print("  → la MISMA línea (place_frozen) las dos veces: al correr ROW_FROZEN_ROOT a 479,")
print("    la fila 471 cae al brazo genérico con nc=59 ∈ (36..60) → nivel 24 sobre path de 24.")
print("    Mover offsets de columnas (intento 2) no toca el match: mismo estallido.")

print()
print("=" * 72)
print("E. LA HOLGURA DEL FINAL Y EL PRESUPUESTO DEL PILOTO")
print("=" * 72)
libres = TRACE_LENGTH - (ROW_PENDING_ROOT + 1)
print(f"  última fila escrita por la tubería: {ROW_PENDING_ROOT} → libres {ROW_PENDING_ROOT+1}..{TRACE_LENGTH-1} = {libres} filas = {libres // CYCLE} ciclos")
print(f"  (hash_flag y ARK son cero ahí: 0..=ROW_PENDING_ROOT; los carriles 0..24 quedan sin restricción activa)")
salt, frozen32 = CYCLE, (32 - FROZEN_DEPTH) * CYCLE
nuevo_root = ROW_PENDING_ROOT + salt + frozen32
print(f"  piloto completo = salt (+{salt}) + frozen-32 (+{frozen32}) → ROW_PENDING_ROOT {ROW_PENDING_ROOT} → {nuevo_root}")
print(f"  margen restante en TRACE_LENGTH=1024: {TRACE_LENGTH - 1 - nuevo_root} filas → CABE sin duplicar traza ✔")

print()
print("=" * 72)
print("F. EQUIVALENCIA DE LA FORMA DERIVADA (propuesta de refactor)")
print("=" * 72)
CYC_NONCE = 1
CYC_ACC = 2
CYC_PK = CYC_ACC + TREE_DEPTH
CYC_FROZEN = CYC_PK + 1
CYC_PEND_IN = CYC_FROZEN + FROZEN_DEPTH
CYC_PEND_VAL = CYC_PEND_IN + 1
CYC_PEND_CLIMB = CYC_PEND_VAL + 1
CYC_FIN = CYC_PEND_CLIMB + TREE_DEPTH
derivadas = {
    "ROW_LEAF_LINK": CYC_NONCE * CYCLE - 1, "ROW_LEAF_DONE": CYC_ACC * CYCLE - 1,
    "ROW_ROOT": CYC_PK * CYCLE - 1, "ROW_PK_START": CYC_PK * CYCLE,
    "ROW_PK_DONE": CYC_FROZEN * CYCLE - 1, "ROW_FROZEN_ROOT": CYC_PEND_IN * CYCLE - 1,
    "ROW_PEND_INNER": CYC_PEND_VAL * CYCLE - 1, "ROW_PENDING_ENTRY": CYC_PEND_CLIMB * CYCLE - 1,
    "ROW_PENDING_ROOT": CYC_FIN * CYCLE - 1,
}
reales = dict(ROW_LEAF_LINK=7, ROW_LEAF_DONE=15, ROW_ROOT=271, ROW_PK_START=272,
              ROW_PK_DONE=279, ROW_FROZEN_ROOT=471, ROW_PEND_INNER=479,
              ROW_PENDING_ENTRY=487, ROW_PENDING_ROOT=743)
for k in reales:
    marca = "✔" if derivadas[k] == reales[k] else "✘"
    print(f"  {k:18s} literal {reales[k]:4d}  derivada {derivadas[k]:4d}  {marca}")
assert derivadas == reales
print("  → las NUEVE constantes de fila colapsan en 8 arranques de ciclo derivados")
print("    de (CYCLE, TREE_DEPTH, FROZEN_DEPTH). Los literales 2/35/61 de bucles y")
print("    periódicas son CYC_ACC/CYC_FROZEN/CYC_PEND_CLIMB. Los rangos del match:")
print(f"    cuentas  (CYC_ACC..CYC_PK)               = ({CYC_ACC}..{CYC_PK})   [hoy: (2..34)]")
print(f"    frozen   (CYC_FROZEN..CYC_PEND_IN)       = ({CYC_FROZEN}..{CYC_PEND_IN})  [hoy: (36..60), nc 35 y 59 inalcanzables]")
print(f"    pendiente(CYC_PEND_CLIMB..CYC_FIN)       = ({CYC_PEND_CLIMB}..{CYC_FIN})  [hoy: (62..94), nc 61 y 93 inalcanzables]")
