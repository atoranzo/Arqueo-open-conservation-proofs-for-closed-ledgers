#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""FV-2 — exportador SMT2 de `circuit_refund` (doc §2; entrada 72).

Emite el sistema (20 restricciones con selectores EVALUADOS POR FILA + 12
aserciones, traza 16×12, primo Goldilocks) en tres consultas:

  q1_determinacion.smt2   dos copias A/B; publics (P, amount) como símbolos
                          COMPARTIDOS y celdas libres de fila 0 IGUALADAS;
                          se pide alguna celda distinta en filas 1..15.
                          UNSAT = determinación funcional (doc §4).
  q1_mut_carry.smt2       lo mismo con C_CARRY+0 borrado en AMBAS copias:
                          la caza. SAT = el mutante abre un segundo testigo;
                          timeout = intratable con acta (válido, doc §2).
  q2_cadena_uf.smt2       la cadena SIN seguir las rondas: cada ciclo de 7
                          rondas se abstrae como permutación NO interpretada
                          R (12 funciones F¹²→F, COMPARTIDAS entre copias y
                          entre ciclos — mismo programa de rondas). Quedan
                          C_CAP, C_CARRY y las 12 aserciones. UNSAT = la
                          fontanería no filtra libertad al margen del hash.
                          **Interpretación DECLARADA aquí** (rito de §4:
                          verificar el teorema correcto y decirlo).

La aritmética de Rescue la canta winterfell: las constantes MDS/INV_MDS/
ARK1/ARK2 llegan en JSON desde `examples/volcado_rescue.rs` (cargo run).
`--sinteticas` genera un juego consistente (matriz invertible real mod p,
ARKs deterministas) para ensayar la TUBERÍA sin winterfell: misma
semántica, mismos conteos; los veredictos SMT solo valen con las reales.

Autotest interno (corre SIEMPRE, antes de emitir): construye la traza
honesta con la MISMA semántica que `build_trace` (fila a fila:
next = MDS·inv_sbox(MDS·sbox(cur)+ARK1)+ARK2; enlace en pos 7) y evalúa
las 20 restricciones en las 15 transiciones — TODAS deben dar 0 — y las
12 aserciones sobre ella. Un exportador que no reproduce el circuito no
exporta: inventa.
"""
import argparse
import json
import os
import re
import sys

P = 2**64 - 2**32 + 1  # Goldilocks
STATE = 12
ROUNDS = 7
CYCLE = 8
MERGES = 2
LENGTH = MERGES * CYCLE          # 16
ROW_AMOUNT = CYCLE               # 8
ROW_P = LENGTH - 1               # 15
INV_ALPHA = pow(7, -1, P - 1)

AQUI = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(AQUI, "..", ".."))
CIRCUITO = os.path.join(REPO, "crates", "stark-experiment", "src",
                        "circuit_refund.rs")

# ── Anclas verbatim: si el circuito cambia de forma, esto MUERE ──
ANCLAS = [
    ("ronda", "result[C_HASH + i] = hash_flag * (apply_sbox(b[i]) - a[i]);"),
    ("cap", "result[C_CAP + i] = link_flag * next[i];"),
    ("carry", "result[C_CARRY + i] = link_flag * (next[4 + i] - current[4 + i]);"),
    ("a_def", "a[i] = acc + ark1[i];"),
    ("b_def", "acc += E::from(Rp64_256::INV_MDS[i][j]) * (next[j] - ark2[j]);"),
    ("as_cap0", "assertions.push(Assertion::single(i, 0, zero));"),
    ("as_amount", "assertions.push(Assertion::single(8, ROW_AMOUNT, self.amount));"),
    ("as_ceros", "assertions.push(Assertion::single(8 + i, ROW_AMOUNT, zero));"),
    ("as_p", "assertions.push(Assertion::single(4 + i, ROW_P, self.commitment[i]));"),
]
RE_CONST = re.compile(
    r"^(?:pub )?const ([A-Z][A-Z_0-9]*): usize = (.+?);", re.MULTILINE)


def verificar_anclas():
    t = open(CIRCUITO, encoding="utf-8").read()
    for quien, a in ANCLAS:
        n = t.count(a)
        assert n == 1, f"ancla '{quien}' aparece {n} veces, no 1: el circuito cambió"
    crudos = {m.group(1): m.group(2).strip() for m in RE_CONST.finditer(t)}
    v = {"STATE_WIDTH": STATE, "NUM_ROUNDS": ROUNDS}
    for _ in range(len(crudos) + 2):
        for k, e in crudos.items():
            if k in v:
                continue
            try:
                r = eval(e, {"__builtins__": {}}, v)  # noqa: S307
            except Exception:
                continue
            if isinstance(r, int):
                v[k] = r
    esperado = {"CYCLE_LENGTH": CYCLE, "NUM_MERGES": MERGES,
                "TRACE_LENGTH": LENGTH, "TRACE_WIDTH": STATE,
                "ROW_AMOUNT": ROW_AMOUNT, "ROW_P": ROW_P,
                "C_HASH": 0, "C_CAP": STATE, "C_CARRY": STATE + 4,
                "TRANSITION_WIDTH": 20}
    for k, quiero in esperado.items():
        assert v.get(k) == quiero, f"constante {k}: leí {v.get(k)}, espero {quiero}"
    return v


# ── Constantes de Rescue: reales (JSON del example) o sintéticas ──
def ctes_reales(ruta):
    d = json.load(open(ruta, encoding="utf-8"))
    assert d["p"] == P, "el primo del volcado no es Goldilocks"
    for k, filas, cols in [("MDS", STATE, STATE), ("INV_MDS", STATE, STATE),
                           ("ARK1", ROUNDS, STATE), ("ARK2", ROUNDS, STATE)]:
        m = d[k]
        assert len(m) == filas and all(len(f) == cols for f in m), f"forma de {k}"
    mds, inv = d["MDS"], d["INV_MDS"]
    for i in range(STATE):          # inversa de verdad, no de palabra
        for j in range(STATE):
            s = sum(mds[i][k] * inv[k][j] for k in range(STATE)) % P
            assert s == (1 if i == j else 0), f"MDS·INV_MDS≠I en ({i},{j})"
    return d["MDS"], d["INV_MDS"], d["ARK1"], d["ARK2"], "reales"


def _inversa_mod_p(m):
    n = len(m)
    aug = [fila[:] + [1 if i == j else 0 for j in range(n)]
           for i, fila in enumerate(m)]
    for c in range(n):
        piv = next((r for r in range(c, n) if aug[r][c] % P), None)
        if piv is None:
            return None
        aug[c], aug[piv] = aug[piv], aug[c]
        inv = pow(aug[c][c], -1, P)
        aug[c] = [(x * inv) % P for x in aug[c]]
        for r in range(n):
            if r != c and aug[r][c]:
                f = aug[r][c]
                aug[r] = [(a - f * b) % P for a, b in zip(aug[r], aug[c])]
    return [fila[n:] for fila in aug]


def ctes_sinteticas():
    # Determinista (sin azar): m[i][j] = (i*31 + j*17 + 3)^3 + i + 1 mod p,
    # retocando la diagonal hasta que la matriz sea invertible.
    m = [[(pow(i * 31 + j * 17 + 3, 3, P) + i + 1) % P
          for j in range(STATE)] for i in range(STATE)]
    for extra in range(64):
        inv = _inversa_mod_p([f[:] for f in m])
        if inv is not None:
            break
        for i in range(STATE):
            m[i][i] = (m[i][i] + 1 + extra) % P
    assert inv is not None, "no salió una MDS sintética invertible"
    ark1 = [[(pow(r * 131 + i * 7 + 11, 5, P)) % P for i in range(STATE)]
            for r in range(ROUNDS)]
    ark2 = [[(pow(r * 197 + i * 13 + 29, 5, P)) % P for i in range(STATE)]
            for r in range(ROUNDS)]
    return m, inv, ark1, ark2, "sintéticas"


# ── Semántica (idéntica a build_trace + evaluate_transition) ──
def sbox(x):
    return pow(x, 7, P)


def inv_sbox(x):
    return pow(x, INV_ALPHA, P)


def mat_vec(m, v):
    return [sum(m[i][j] * v[j] for j in range(STATE)) % P for i in range(STATE)]


def traza_honesta(mds, inv_mds, ark1, ark2, rid, salt, amount):
    filas = [[0] * STATE for _ in range(LENGTH)]
    s = [0] * STATE
    s[4:8], s[8:12] = list(rid), list(salt)
    filas[0] = s[:]
    for r in range(LENGTH - 1):
        pos = r % CYCLE
        if pos < ROUNDS:
            a = [(x + y) % P for x, y in
                 zip(mat_vec(mds, [sbox(x) for x in s]), ark1[pos])]
            b = [inv_sbox(x) for x in a]
            s = [(x + y) % P for x, y in zip(mat_vec(mds, b), ark2[pos])]
        else:
            dig = s[4:8]
            s = [0] * STATE
            s[4:8] = dig
            s[8] = amount % P
        filas[r + 1] = s[:]
    return filas


def evaluar_sistema(mds, inv_mds, ark1, ark2, filas):
    """Las 20 restricciones sobre las 15 transiciones; lista de no-ceros."""
    malas = []
    for r in range(LENGTH - 1):
        pos = r % CYCLE
        cur, nxt = filas[r], filas[r + 1]
        if pos < ROUNDS:                       # 12 sépticas activas
            a = [(x + y) % P for x, y in
                 zip(mat_vec(mds, [sbox(x) for x in cur]), ark1[pos])]
            b = mat_vec(inv_mds, [(nxt[j] - ark2[pos][j]) % P
                                  for j in range(STATE)])
            for i in range(STATE):
                if (sbox(b[i]) - a[i]) % P:
                    malas.append(("C_HASH", r, i))
        else:                                  # 8 lineales activas
            for i in range(4):
                if nxt[i] % P:
                    malas.append(("C_CAP", r, i))
            for i in range(4):
                if (nxt[4 + i] - cur[4 + i]) % P:
                    malas.append(("C_CARRY", r, i))
    return malas


# ── Emisión SMT2 ──
def ff(k):
    return f"(as ff{k % P} F)"


def neg(t):
    return f"(ff.mul {ff(P - 1)} {t})"


def suma(ts):
    ts = [t for t in ts if t is not None]
    return ts[0] if len(ts) == 1 else "(ff.add " + " ".join(ts) + ")"


def pot7(t):
    return "(ff.mul " + " ".join([t] * 7) + ")"


def var(copia, r, c):
    return f"s{copia}_{r}_{c}"


class Emisor:
    def __init__(self):
        self.lineas = []
        self.n_asserts = 0

    def w(self, s):
        self.lineas.append(s)

    def asrt(self, t):
        self.w(f"(assert {t})")
        self.n_asserts += 1

    def texto(self):
        return "\n".join(self.lineas) + "\n"


def cuerpo_comun(e, copias, mds, inv_mds, ark1, ark2, sin_carry0, con_rondas):
    for cp in copias:
        for r in range(LENGTH):
            for c in range(STATE):
                e.w(f"(declare-const {var(cp, r, c)} F)")
    for s in ("P0", "P1", "P2", "P3", "AMT"):
        e.w(f"(declare-const {s} F)")
    e.w("; --- aserciones (12 por copia) ---")
    for cp in copias:
        for i in range(4):
            e.asrt(f"(= {var(cp, 0, i)} {ff(0)})")
        e.asrt(f"(= {var(cp, ROW_AMOUNT, 8)} AMT)")
        for i in range(1, 4):
            e.asrt(f"(= {var(cp, ROW_AMOUNT, 8 + i)} {ff(0)})")
        for i in range(4):
            e.asrt(f"(= {var(cp, ROW_P, 4 + i)} P{i})")
    e.w("; --- fila 0: el testigo (cols 4..11) IGUALADO entre copias ---")
    if len(copias) == 2:
        for c in range(4, STATE):
            e.asrt(f"(= {var('A', 0, c)} {var('B', 0, c)})")
    e.w("; --- transiciones con selectores evaluados por fila ---")
    for cp in copias:
        for r in range(LENGTH - 1):
            pos = r % CYCLE
            if pos < ROUNDS:
                if not con_rondas:
                    continue
                for i in range(STATE):
                    a_i = suma([f"(ff.mul {ff(mds[i][j])} {pot7(var(cp, r, j))})"
                                for j in range(STATE)] + [ff(ark1[pos][i])])
                    b_i = suma([f"(ff.mul {ff(inv_mds[i][j])} "
                                f"(ff.add {var(cp, r + 1, j)} {neg(ff(ark2[pos][j]))}))"
                                for j in range(STATE)])
                    e.asrt(f"(= {pot7(b_i)} {a_i})")
            else:
                for i in range(4):
                    e.asrt(f"(= {var(cp, r + 1, i)} {ff(0)})")
                for i in range(4):
                    if sin_carry0 and i == 0:
                        e.w(f"; MUTANTE: C_CARRY+0 borrado (copia {cp}, fila {r})")
                        continue
                    e.asrt(f"(= {var(cp, r + 1, 4 + i)} {var(cp, r, 4 + i)})")


def diferencia(e, filas_dif=None):
    """filas_dif=None barre 1..15 (q1, todas atadas por rondas+enlace).
    q2 pasa {7,8,15}: en la abstraccion-R las filas intermedias quedan
    SUELTAS a proposito, y barrerlas daria SAT trivial sin significado —
    la cadena abstracta vive en (0,)7,8,15 (fila 0 ya va igualada)."""
    if filas_dif is None:
        filas_dif = range(1, LENGTH)
    e.w(f"; --- alguna celda distinta en filas {sorted(filas_dif)} ---")
    atomos = [f"(not (= {var('A', r, c)} {var('B', r, c)}))"
              for r in filas_dif for c in range(STATE)]
    e.asrt("(or\n  " + "\n  ".join(atomos) + ")")


def emitir_q1(mds, inv_mds, ark1, ark2, mutante):
    e = Emisor()
    e.w("(set-logic QF_FF)")
    e.w(f"(define-sort F () (_ FiniteField {P}))")
    cuerpo_comun(e, ["A", "B"], mds, inv_mds, ark1, ark2,
                 sin_carry0=mutante, con_rondas=True)
    diferencia(e)
    e.w("(check-sat)")
    return e


def emitir_q2():
    e = Emisor()
    e.w("(set-logic QF_UFFF)")
    e.w(f"(define-sort F () (_ FiniteField {P}))")
    e.w("; R = la permutación de 7 rondas, NO interpretada, compartida")
    dom = " ".join(["F"] * STATE)
    for c in range(STATE):
        e.w(f"(declare-fun R{c} ({dom}) F)")
    cuerpo_comun(e, ["A", "B"], None, None, None, None,
                 sin_carry0=False, con_rondas=False)
    e.w("; --- cada ciclo: salida = R(entrada), mismas R en las dos copias ---")
    for cp in ("A", "B"):
        for (r_in, r_out) in [(0, ROUNDS), (CYCLE, CYCLE + ROUNDS)]:
            args = " ".join(var(cp, r_in, j) for j in range(STATE))
            for c in range(STATE):
                e.asrt(f"(= {var(cp, r_out, c)} (R{c} {args}))")
    diferencia(e, filas_dif=[ROUNDS, CYCLE, ROW_P])
    e.w("(check-sat)")
    return e


def balance(texto):
    prof = 0
    for ch in texto:
        if ch == "(":
            prof += 1
        elif ch == ")":
            prof -= 1
        assert prof >= 0, "paréntesis desbalanceados"
    assert prof == 0, "paréntesis sin cerrar"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ctes", help="JSON de examples/volcado_rescue.rs")
    ap.add_argument("--sinteticas", action="store_true")
    ap.add_argument("--salida", default="/tmp/fv2")
    args = ap.parse_args()

    verificar_anclas()
    if args.sinteticas:
        mds, inv_mds, ark1, ark2, origen = ctes_sinteticas()
    else:
        assert args.ctes, "falta --ctes (o usa --sinteticas para ensayar)"
        mds, inv_mds, ark1, ark2, origen = ctes_reales(args.ctes)

    # Autotest: la traza honesta debe evaluar 0 en las 15×20, y las
    # aserciones deben leerse en ella donde el circuito las mira.
    rid = [1000, 1001, 1002, 1003]
    salt = [2000, 2001, 2002, 2003]
    amount = 250_000
    tz = traza_honesta(mds, inv_mds, ark1, ark2, rid, salt, amount)
    malas = evaluar_sistema(mds, inv_mds, ark1, ark2, tz)
    assert not malas, f"la traza honesta NO satisface: {malas[:3]}"
    assert tz[ROW_AMOUNT][8] == amount and tz[ROW_AMOUNT][9:12] == [0, 0, 0]
    assert tz[0][0:4] == [0, 0, 0, 0]
    n_eval = (LENGTH - 1 - MERGES + 1) * STATE + (MERGES - 1) * 8  # 14·12+8

    os.makedirs(args.salida, exist_ok=True)
    salidas = []
    for nombre, e in [("q1_determinacion", emitir_q1(mds, inv_mds, ark1, ark2, False)),
                      ("q1_mut_carry", emitir_q1(mds, inv_mds, ark1, ark2, True)),
                      ("q2_cadena_uf", emitir_q2())]:
        t = e.texto()
        balance(t)
        ruta = os.path.join(args.salida, nombre + ".smt2")
        open(ruta, "w", encoding="utf-8").write(t)
        salidas.append((nombre, e.n_asserts, len(t.splitlines())))
    a1, a1m, a2 = (s[1] for s in salidas)
    assert a1m == a1 - 2, "el mutante debe quitar EXACTAMENTE 2 asserts (uno por copia)"
    print(f"   ficheros en {args.salida}: " +
          " · ".join(f"{n} ({a} asserts, {l} líneas)" for n, a, l in salidas))
    print(f"COMPUERTA-FV2: 20 restricciones · 12 aserciones · traza 16×12 · "
          f"sano evalúa 0 en {n_eval} activas · q1 {a1} asserts · "
          f"q1m {a1m} · q2 {a2} · ctes {origen}")


if __name__ == "__main__":
    sys.exit(main())
