#!/usr/bin/env python3
"""Las AIR del paquete de evidencia, para el verificador STARK de la segunda implementacion
(`stark.py`, BACKLOG 85, cuarto hito).

De donde salen, y lo que eso significa. TRANSCRITAS de `crates/zk-ssl-air/src/*.rs`, restriccion a
restriccion, con sus mismos indices. Ninguna RFC escribe estas restricciones: el `.rs` es la unica
fuente que existe. Por eso este fichero NO es una segunda opinion sobre las AIR: dice lo mismo que
ellas, y una AIR infra-restringida pasaria aqui igual que alli. Lo que el cuarto hito mide con
ellas es la maquinaria de `stark.py` -Fiat-Shamir, FRI, DEEP, Merkle con sal, Oculta-, que si es
otro codigo.

Cada AIR expone lo que el verificador necesita y nada del probador:
  forma(p)          la forma de traza que su `verificar()` exige antes de construir el AIR
  enunciado()       las comprobaciones de `comprobar_enunciado`, si las tiene
  entradas()        `PublicInputs::to_elements()`
  grados()          `TransitionConstraintDegree` principales, como (base, [ciclos])
  grados_aux()      los auxiliares
  exenciones()      `num_transition_exemptions` del AIR interno (1 si no lo cambia)
  periodicas()      `get_periodic_column_values()`
  transicion(...)   `evaluate_transition`
  transicion_aux()  `evaluate_aux_transition`
  aserciones()      `get_assertions()`, como (columna, fila, valor) -todas simples-
  aserciones_aux()  `get_aux_assertions()`
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import stark as S  # noqa: E402
from stark import Rechazo  # noqa: E402

# zk-ssl-air/src/lib.rs
CICLO = 8
ESTADO = 12
PROFUNDIDAD = 32
RONDAS = 7


class Air:
    nombre = "?"
    traza = None
    ancho = None
    ancho_aux = 0
    aleatorios = 0

    def forma(self, p):
        forma = (p.ancho, p.ancho_aux, p.aleatorios, p.L)
        pide = (self.ancho + 1, self.ancho_aux, self.aleatorios, 2 * self.traza)
        if forma != pide:
            raise Rechazo(f"forma de traza {forma}; el enunciado pide {pide}")

    def enunciado(self):
        pass

    def grados_aux(self):
        return []

    def exenciones(self):
        return 1

    def transicion_aux(self, *a):
        return []

    def aserciones_aux(self, aleatorios):
        return []

    num_aserciones_aux = 0

    def num_aserciones(self):
        return len(self.aserciones()) + self.num_aserciones_aux


# ─────────────────────────────────────────────────────────────── BandaAir (banda.rs)

class Banda(Air):
    """La banda del saldo bajo una raiz, sin titularidad (RFC-0007 E5, corte 4a)."""
    nombre = "banda"
    traza = 512
    ancho = 27
    LARGO_SEGMENTO, SEGMENTOS = 64, 3
    MAX_VALOR = 0x3fffffffffffffff
    COL_BIT, COL_ID, COL_BAL, COL_NONCE = 12, 13, 17, 18
    COL_LOWER, COL_UPPER, COL_SBIT, COL_SACC, COL_LEAF_SALT = 19, 20, 21, 22, 23
    CYC_ACC = 3
    ROW_ENLACE_HOJA, ROW_ENLACE_SALT, ROW_HOJA_LISTA, ROW_RAIZ = 7, 15, 23, 279

    def __init__(self, root, public_id, lower, upper):
        self.root, self.public_id, self.lower, self.upper = root, public_id, lower, upper

    def enunciado(self):
        l, u = self.lower, self.upper
        if l > self.MAX_VALOR or u > self.MAX_VALOR:
            raise Rechazo(f"los limites {l} y {u} pasan del techo {self.MAX_VALOR}")
        if l > u:
            raise Rechazo(f"banda vacia: inferior {l} sobre superior {u}")

    def entradas(self):
        return list(self.root) + list(self.public_id) + [self.lower, self.upper]

    def grados(self):
        T = self.traza
        c = lambda d: (d, [T])  # noqa: E731
        g = lambda d: (d, [])  # noqa: E731
        return ([c(7)] * 12 + [c(1)] * 4 + [c(2)] * 4 + [g(2)] + [c(1)] * 14 + [g(1)] * 8
                + [g(2)] * 2 + [c(1)] * 6 + [c(1)] * 12)

    def periodicas(self):
        T = self.traza
        cols = []
        bandera = [0] * T
        for r in range(self.ROW_RAIZ + 1):
            if r % CICLO < RONDAS:
                bandera[r] = 1
        cols.append(bandera)
        for ark in (S.ARK1, S.ARK2):
            for i in range(ESTADO):
                col = [0] * T
                for r in range(self.ROW_RAIZ + 1):
                    pos = r % CICLO
                    if pos < RONDAS:
                        col[r] = ark[pos][i]
                cols.append(col)
        enlace = [0] * T
        for nivel in range(PROFUNDIDAD - 1):
            enlace[(self.CYC_ACC + nivel) * CICLO + CICLO - 1] = 1
        cols.append(enlace)
        for fila in (self.ROW_ENLACE_HOJA, self.ROW_ENLACE_SALT, self.ROW_HOJA_LISTA, 0):
            sel = [0] * T
            sel[fila] = 1
            cols.append(sel)
        primera, sigue = [0] * T, [0] * T
        L = self.LARGO_SEGMENTO
        for seg in range(self.SEGMENTOS):
            primera[seg * L] = 1
            for p in range(L - 1):
                sigue[seg * L + p] = 1
        cols += [primera, sigue]
        for seg in range(self.SEGMENTOS):
            e = [0] * T
            e[(seg + 1) * L - 2] = 1
            cols.append(e)
        assert len(cols) == 35
        return cols

    def transicion(self, a, s, per):
        r = [None] * 63
        bandera, ark1, ark2 = per[0], per[1:13], per[13:25]
        enl_arbol, enl_hoja, enl_salt, enl_sitio = per[25], per[26], per[27], per[28]
        primera_fila, primera_s, sigue_s = per[29], per[30], per[31]
        r[0:12] = S.ronda_rescue(a, s, ark1, ark2, bandera)
        bit = s[self.COL_BIT]
        enlace = enl_arbol + enl_sitio
        for i in range(4):
            r[12 + i] = enlace * s[i]
            d = a[4 + i]
            r[16 + i] = enlace * ((1 - bit) * (s[4 + i] - d) + bit * (s[8 + i] - d))
        r[20] = a[self.COL_BIT] * (a[self.COL_BIT] - 1)
        for i in range(4):
            r[21 + i] = enl_hoja * s[i]
            r[25 + i] = enl_hoja * (s[4 + i] - a[4 + i])
        r[29] = enl_hoja * (s[8] - a[self.COL_NONCE])
        for i in range(4):
            r[51 + i] = enl_salt * s[i]
            r[55 + i] = enl_salt * (s[4 + i] - a[4 + i])
            r[59 + i] = enl_salt * (s[8 + i] - a[self.COL_LEAF_SALT + i])
        for i in range(4):
            r[30 + i] = primera_fila * (a[4 + i] - a[self.COL_ID + i])
        r[34] = primera_fila * (a[8] - a[self.COL_BAL])
        for k, col in enumerate((self.COL_BAL, self.COL_NONCE, self.COL_LOWER, self.COL_UPPER)):
            r[35 + k] = s[col] - a[col]
        for i in range(4):
            r[39 + i] = s[self.COL_ID + i] - a[self.COL_ID + i]
        sa, ss = a[self.COL_SBIT], s[self.COL_SBIT]
        ca, cs = a[self.COL_SACC], s[self.COL_SACC]
        r[43] = sa * (sa - 1)
        r[44] = ss * (ss - 1)
        r[45] = primera_s * sa
        r[46] = primera_s * ca
        r[47] = sigue_s * (cs - (ca + ca + ss))
        bal = a[self.COL_BAL]
        esperado = [bal, bal - a[self.COL_LOWER], a[self.COL_UPPER] - bal]
        for seg in range(3):
            r[48 + seg] = per[32 + seg] * (cs - esperado[seg])
        return r

    def aserciones(self):
        a = [(i, 0, 0) for i in range(4)] + [(i, 0, 0) for i in range(9, ESTADO)]
        a += [(4 + i, self.ROW_RAIZ, self.root[i]) for i in range(4)]
        a += [(self.COL_ID + i, 0, self.public_id[i]) for i in range(4)]
        a += [(self.COL_LOWER, 0, self.lower), (self.COL_UPPER, 0, self.upper)]
        assert len(a) == 17
        return a


# ─────────────────────────────────────────────────────────────── PrendaAir (prenda.rs)

SPEND_KEY_DOMAIN = 0x53504B59                              # zk-ssl-hash, "SPKY"
DOMINIO_PRENDA = int.from_bytes(b"PREND_V1", "big")        # zk-ssl-hash, u64::from_be_bytes


class Prenda(Air):
    """Un pendiente que solo puede cobrar quien tiene la clave del receptor (RFC-0008 E3)."""
    nombre = "prenda"
    traza = 512
    ancho = 42
    C_A, COL_BIT, COL_RECEPTOR, COL_IMPORTE, COL_KEY, COL_SAL, COL_X, C_B = 0, 12, 13, 17, 18, 22, 26, 30
    CYC_ACC = CYC_MARCA = 3
    ROW_ENLACE_IMPORTE, ROW_ENLACE_X, ROW_HOJA_LISTA, ROW_MARCA, ROW_RAIZ = 7, 15, 23, 31, 279
    TRANSPORTE = ([13, 14, 15, 16, 17] + [18, 19, 20, 21] + [22, 23, 24, 25] + [26, 27, 28, 29])

    def __init__(self, pending_root, receptor, marca):
        self.pending_root, self.receptor, self.marca = pending_root, receptor, marca

    def entradas(self):
        return list(self.pending_root) + list(self.receptor) + list(self.marca)

    def grados(self):
        c = lambda d: (d, [self.traza])  # noqa: E731
        # 24 rondas, 4 capacidades, 4 sitios, el bit, C_IMP_CAP..C_TRANSPORT ciclicas, el transporte
        return [c(7)] * 24 + [c(1)] * 4 + [c(2)] * 4 + [(2, [])] + [c(1)] * (89 - 33) + [(1, [])] * 17

    def periodicas(self):
        T = self.traza
        cols = []
        ba = [0] * T
        for r in range(self.ROW_RAIZ + 1):
            if r % CICLO < RONDAS:
                ba[r] = 1
        cols.append(ba)
        bb = [0] * T
        for ciclo in (0, self.CYC_MARCA):
            for pos in range(RONDAS):
                bb[ciclo * CICLO + pos] = 1
        cols.append(bb)
        for ark in (S.ARK1, S.ARK2):
            for i in range(ESTADO):
                col = [0] * T
                for r in range(self.ROW_RAIZ + 1):
                    pos = r % CICLO
                    if pos < RONDAS:
                        col[r] = ark[pos][i]
                cols.append(col)
        enlace = [0] * T
        for nivel in range(PROFUNDIDAD - 1):
            enlace[(self.CYC_ACC + nivel) * CICLO + CICLO - 1] = 1
        cols.append(enlace)
        for fila in (self.ROW_ENLACE_IMPORTE, self.ROW_ENLACE_X, self.ROW_HOJA_LISTA, 0):
            sel = [0] * T
            sel[fila] = 1
            cols.append(sel)
        assert len(cols) == 31
        return cols

    def transicion(self, a, s, per):
        r = [None] * 106
        A, B = self.C_A, self.C_B
        ark1, ark2 = per[2:14], per[14:26]
        enl_arbol, enl_imp, enl_x, enl_sitio, primera = per[26], per[27], per[28], per[29], per[30]
        r[0:12] = S.ronda_rescue(a[A:A + 12], s[A:A + 12], ark1, ark2, per[0])
        r[12:24] = S.ronda_rescue(a[B:B + 12], s[B:B + 12], ark1, ark2, per[1])
        bit = s[self.COL_BIT]
        enlace = enl_arbol + enl_sitio
        for i in range(4):
            r[24 + i] = enlace * s[A + i]
            d = a[A + 4 + i]
            r[28 + i] = enlace * ((1 - bit) * (s[A + 4 + i] - d) + bit * (s[A + 8 + i] - d))
        r[32] = a[self.COL_BIT] * (a[self.COL_BIT] - 1)
        for i in range(4):
            r[33 + i] = enl_imp * s[A + i]
            r[37 + i] = enl_imp * (s[A + 4 + i] - a[A + 4 + i])
        r[41] = enl_imp * (s[A + 8] - a[self.COL_IMPORTE])
        for i in range(3):
            r[42 + i] = enl_imp * s[A + 9 + i]
        for i in range(4):
            r[45 + i] = enl_x * s[A + i]
            r[49 + i] = enl_x * (s[A + 4 + i] - a[A + 4 + i])
            r[53 + i] = enl_x * (s[A + 8 + i] - a[self.COL_X + i])
        r[57] = enl_sitio * (s[B] - DOMINIO_PRENDA)
        for i in range(1, 4):
            r[57 + i] = enl_sitio * s[B + i]
        for i in range(4):
            r[61 + i] = enl_sitio * (s[B + 4 + i] - a[A + 4 + i])
        for i in range(4, 8):
            r[61 + i] = enl_sitio * s[B + 4 + i]
        for i in range(4):
            r[69 + i] = primera * (a[A + 4 + i] - a[self.COL_RECEPTOR + i])
            r[73 + i] = primera * (a[A + 8 + i] - a[self.COL_SAL + i])
        r[77] = primera * (a[B + 4] - SPEND_KEY_DOMAIN)
        for i in range(1, 4):
            r[77 + i] = primera * a[B + 4 + i]
        for i in range(4):
            r[81 + i] = primera * (a[B + 8 + i] - a[self.COL_KEY + i])
        for i in range(4):
            r[85 + i] = enl_imp * (a[B + 4 + i] - a[self.COL_RECEPTOR + i])
        for k, col in enumerate(self.TRANSPORTE):
            r[89 + k] = s[col] - a[col]
        return r

    def aserciones(self):
        A, B = self.C_A, self.C_B
        a = [(A + i, 0, 0) for i in range(4)] + [(B + i, 0, 0) for i in range(4)]
        a += [(A + 4 + i, self.ROW_RAIZ, self.pending_root[i]) for i in range(4)]
        a += [(self.COL_RECEPTOR + i, 0, self.receptor[i]) for i in range(4)]
        a += [(B + 4 + i, self.ROW_MARCA, self.marca[i]) for i in range(4)]
        assert len(a) == 20
        return a


# ─────────────────────────────────────────────────────── CobroPendienteAir (cobro_pendiente.rs)

DOMINIO_META_PENDIENTE = int.from_bytes(b"PMETA_V1", "big")  # zk-ssl-hash


def _periodicas_dos_carriles(air, desde_b, segmentos):
    """Las columnas periodicas comunes a cobro y pago: dos banderas de hash (la del carril B
    arranca en `desde_b`), ARK1/ARK2, el enlace del arbol, los cuatro selectores y, si hay rango,
    sus `segmentos` de 64 bits."""
    T = air.traza
    cols = []
    for desde in (0, desde_b):
        band = [0] * T
        for r in range(desde, air.ROW_RAIZ + 1):
            if r % CICLO < RONDAS:
                band[r] = 1
        cols.append(band)
    for ark in (S.ARK1, S.ARK2):
        for i in range(ESTADO):
            col = [0] * T
            for r in range(air.ROW_RAIZ + 1):
                pos = r % CICLO
                if pos < RONDAS:
                    col[r] = ark[pos][i]
            cols.append(col)
    enlace = [0] * T
    for nivel in range(PROFUNDIDAD - 1):
        enlace[(air.CYC_ACC + nivel) * CICLO + CICLO - 1] = 1
    cols.append(enlace)
    for fila in (air.ROW_ENLACE_IMPORTE, air.ROW_ENLACE_X, air.ROW_HOJA_LISTA, 0):
        sel = [0] * T
        sel[fila] = 1
        cols.append(sel)
    primera, sigue = [0] * T, [0] * T
    for seg in range(segmentos):
        primera[seg * 64] = 1
        for p in range(63):
            sigue[seg * 64 + p] = 1
    cols += [primera, sigue]
    for seg in range(segmentos):
        e = [0] * T
        e[(seg + 1) * 64 - 2] = 1
        cols.append(e)
    return cols


class CobroPendiente(Air):
    """Un pendiente a nombre del receptor por al menos `inferior`, nacido en `nacido`."""
    nombre = "cobro"
    traza = 512
    ancho = 44
    MAX_VALOR = 0x3fffffffffffffff
    C_A, COL_BIT, COL_RECEPTOR, COL_IMPORTE, COL_INFERIOR, COL_SUPERIOR = 0, 12, 13, 17, 18, 19
    COL_SBIT, COL_SACC, COL_SAL, COL_X, C_B, COL_EMISOR, COL_NACIDO = 20, 21, 22, 26, 30, 42, 43
    CYC_META, CYC_ACC = 2, 3
    ROW_ENLACE_IMPORTE, ROW_ENLACE_X, ROW_HOJA_LISTA, ROW_RAIZ = 7, 15, 23, 279
    TRANSPORTE = [13, 14, 15, 16, 17, 18, 19, 22, 23, 24, 25, 26, 27, 28, 29, 42, 43]

    def __init__(self, seq, pending_root, pmeta_root, receptor, nacido, inferior):
        self.seq = seq
        self.pending_root, self.pmeta_root, self.receptor = pending_root, pmeta_root, receptor
        self.nacido, self.inferior, self.superior = nacido, inferior, self.MAX_VALOR

    def enunciado(self):
        if self.nacido >= self.seq:
            raise Rechazo(f"nacido {self.nacido} no es anterior a la cabeza de seq {self.seq}: "
                          "una meta nacida despues de la cabeza que la firma")
        l, u = self.inferior, self.superior
        if l > self.MAX_VALOR or u > self.MAX_VALOR:
            raise Rechazo(f"las cotas {l} y {u} pasan del techo {self.MAX_VALOR}")
        if l > u:
            raise Rechazo(f"banda vacia: inferior {l} sobre superior {u}")

    def entradas(self):
        return (list(self.pending_root) + list(self.pmeta_root) + list(self.receptor)
                + [self.nacido, self.inferior, self.superior])

    def grados(self):
        c = lambda d: (d, [self.traza])  # noqa: E731
        return ([c(7)] * 24 + ([c(1)] * 4 + [c(2)] * 4) * 2 + [(2, [])] + [c(1)] * (85 - 41)
                + [(1, [])] * (102 - 85) + [(2, [])] * 2 + [c(1)] * (110 - 104))

    def periodicas(self):
        cols = _periodicas_dos_carriles(self, self.CYC_META * CICLO, 3)
        assert len(cols) == 36
        return cols

    def transicion(self, a, s, per):
        r = [None] * 110
        A, B = self.C_A, self.C_B
        ark1, ark2 = per[2:14], per[14:26]
        enl_arbol, enl_imp, enl_x, enl_sitio, primera = per[26], per[27], per[28], per[29], per[30]
        primera_s, sigue_s = per[31], per[32]
        r[0:12] = S.ronda_rescue(a[A:A + 12], s[A:A + 12], ark1, ark2, per[0])
        r[12:24] = S.ronda_rescue(a[B:B + 12], s[B:B + 12], ark1, ark2, per[1])
        bit = s[self.COL_BIT]
        enlace = enl_arbol + enl_sitio
        for c_cap, c_place, base in ((24, 28, A), (32, 36, B)):
            for i in range(4):
                r[c_cap + i] = enlace * s[base + i]
                d = a[base + 4 + i]
                r[c_place + i] = enlace * ((1 - bit) * (s[base + 4 + i] - d)
                                           + bit * (s[base + 8 + i] - d))
        r[40] = a[self.COL_BIT] * (a[self.COL_BIT] - 1)
        for i in range(4):
            r[41 + i] = enl_imp * s[A + i]
            r[45 + i] = enl_imp * (s[A + 4 + i] - a[A + 4 + i])
        r[49] = enl_imp * (s[A + 8] - a[self.COL_IMPORTE])
        for i in range(3):
            r[50 + i] = enl_imp * s[A + 9 + i]
        for i in range(4):
            r[53 + i] = enl_x * s[A + i]
            r[57 + i] = enl_x * (s[A + 4 + i] - a[A + 4 + i])
            r[61 + i] = enl_x * (s[A + 8 + i] - a[self.COL_X + i])
        r[65] = enl_x * (s[B] - DOMINIO_META_PENDIENTE)
        for i in range(1, 4):
            r[65 + i] = enl_x * s[B + i]
        r[69] = enl_x * (s[B + 4] - a[self.COL_EMISOR])
        r[70] = enl_x * (s[B + 5] - a[self.COL_NACIDO])
        for i in range(2, 8):
            r[69 + i] = enl_x * s[B + 4 + i]
        for i in range(4):
            r[77 + i] = primera * (a[A + 4 + i] - a[self.COL_RECEPTOR + i])
            r[81 + i] = primera * (a[A + 8 + i] - a[self.COL_SAL + i])
        for k, col in enumerate(self.TRANSPORTE):
            r[85 + k] = s[col] - a[col]
        sa, ss = a[self.COL_SBIT], s[self.COL_SBIT]
        ca, cs = a[self.COL_SACC], s[self.COL_SACC]
        r[102] = sa * (sa - 1)
        r[103] = ss * (ss - 1)
        r[104] = primera_s * sa
        r[105] = primera_s * ca
        r[106] = sigue_s * (cs - (ca + ca + ss))
        imp = a[self.COL_IMPORTE]
        esperado = [imp, imp - a[self.COL_INFERIOR], a[self.COL_SUPERIOR] - imp]
        for seg in range(3):
            r[107 + seg] = per[33 + seg] * (cs - esperado[seg])
        return r

    def aserciones(self):
        A, B = self.C_A, self.C_B
        a = [(A + i, 0, 0) for i in range(4)]
        a += [(A + 4 + i, self.ROW_RAIZ, self.pending_root[i]) for i in range(4)]
        a += [(B + 4 + i, self.ROW_RAIZ, self.pmeta_root[i]) for i in range(4)]
        a += [(self.COL_RECEPTOR + i, 0, self.receptor[i]) for i in range(4)]
        a += [(self.COL_NACIDO, 0, self.nacido), (self.COL_INFERIOR, 0, self.inferior),
              (self.COL_SUPERIOR, 0, self.superior)]
        assert len(a) == 19
        return a


# ─────────────────────────────────────────────────────────── PagoEnCursoAir (pago_en_curso.rs)

class PagoEnCurso(Air):
    """Un pendiente a nombre del receptor por `importe` EXACTO que quien lo pago no puede
    revertir antes de `t`, nacido en `nacido`."""
    nombre = "pago"
    traza = 512
    ancho = 44
    MAX_VALOR = 0x3fffffffffffffff
    C_A, COL_BIT, COL_RECEPTOR, COL_IMPORTE, COL_T, COL_SBIT, COL_SACC = 0, 12, 13, 17, 18, 19, 20
    COL_SAL, COL_REFUND, COL_DELTA, C_B, COL_EMISOR, COL_NACIDO = 21, 25, 29, 30, 42, 43
    CYC_X, CYC_ACC = 1, 3
    ROW_ENLACE_IMPORTE, ROW_ENLACE_X, ROW_HOJA_LISTA, ROW_RAIZ = 7, 15, 23, 279
    TRANSPORTE = [13, 14, 15, 16, 17, 18, 21, 22, 23, 24, 25, 26, 27, 28, 29, 42, 43]

    def __init__(self, seq, pending_root, pmeta_root, receptor, importe, t, nacido):
        self.seq = seq
        self.pending_root, self.pmeta_root, self.receptor = pending_root, pmeta_root, receptor
        self.importe, self.t, self.nacido = importe, t, nacido

    def enunciado(self):
        if self.nacido >= self.seq:
            raise Rechazo(f"nacido {self.nacido} no es anterior a la cabeza de seq {self.seq}: "
                          "una meta nacida despues de la cabeza que la firma")
        i, t, n = self.importe, self.t, self.nacido
        if i > self.MAX_VALOR:
            raise Rechazo(f"el importe {i} pasa del techo {self.MAX_VALOR}")
        if t > self.MAX_VALOR or n > self.MAX_VALOR:
            raise Rechazo(f"T {t} o nacido {n} pasan del techo {self.MAX_VALOR}")
        if n > t:
            raise Rechazo(f"nacido {n} posterior a T {t}: el plazo iria hacia atras")

    def entradas(self):
        return (list(self.pending_root) + list(self.pmeta_root) + list(self.receptor)
                + [self.importe, self.t, self.nacido])

    def grados(self):
        c = lambda d: (d, [self.traza])  # noqa: E731
        return ([c(7)] * 24 + ([c(1)] * 4 + [c(2)] * 4) * 2 + [(2, [])] + [c(1)] * (97 - 41)
                + [(1, [])] * (114 - 97) + [(2, [])] * 2 + [c(1)] * (120 - 116))

    def periodicas(self):
        cols = _periodicas_dos_carriles(self, self.CYC_X * CICLO, 1)
        assert len(cols) == 34
        return cols

    def transicion(self, a, s, per):
        r = [None] * 120
        A, B = self.C_A, self.C_B
        ark1, ark2 = per[2:14], per[14:26]
        enl_arbol, enl_imp, enl_x, enl_sitio, primera = per[26], per[27], per[28], per[29], per[30]
        primera_s, sigue_s = per[31], per[32]
        r[0:12] = S.ronda_rescue(a[A:A + 12], s[A:A + 12], ark1, ark2, per[0])
        r[12:24] = S.ronda_rescue(a[B:B + 12], s[B:B + 12], ark1, ark2, per[1])
        bit = s[self.COL_BIT]
        enlace = enl_arbol + enl_sitio
        for c_cap, c_place, base in ((24, 28, A), (32, 36, B)):
            for i in range(4):
                r[c_cap + i] = enlace * s[base + i]
                d = a[base + 4 + i]
                r[c_place + i] = enlace * ((1 - bit) * (s[base + 4 + i] - d)
                                           + bit * (s[base + 8 + i] - d))
        r[40] = a[self.COL_BIT] * (a[self.COL_BIT] - 1)
        for i in range(4):
            r[41 + i] = enl_imp * s[B + i]
            r[45 + i] = enl_imp * (s[B + 4 + i] - a[self.COL_REFUND + i])
        r[49] = enl_imp * (s[B + 8] - a[self.COL_DELTA])
        for i in range(3):
            r[50 + i] = enl_imp * s[B + 9 + i]
        for i in range(4):
            r[53 + i] = enl_imp * s[A + i]
            r[57 + i] = enl_imp * (s[A + 4 + i] - a[A + 4 + i])
        r[61] = enl_imp * (s[A + 8] - a[self.COL_IMPORTE])
        for i in range(3):
            r[62 + i] = enl_imp * s[A + 9 + i]
        for i in range(4):
            r[65 + i] = enl_x * s[A + i]
            r[69 + i] = enl_x * (s[A + 4 + i] - a[A + 4 + i])
            r[73 + i] = enl_x * (s[A + 8 + i] - a[B + 4 + i])
        r[77] = enl_x * (s[B] - DOMINIO_META_PENDIENTE)
        for i in range(1, 4):
            r[77 + i] = enl_x * s[B + i]
        r[81] = enl_x * (s[B + 4] - a[self.COL_EMISOR])
        r[82] = enl_x * (s[B + 5] - a[self.COL_NACIDO])
        for i in range(2, 8):
            r[81 + i] = enl_x * s[B + 4 + i]
        for i in range(4):
            r[89 + i] = primera * (a[A + 4 + i] - a[self.COL_RECEPTOR + i])
            r[93 + i] = primera * (a[A + 8 + i] - a[self.COL_SAL + i])
        for k, col in enumerate(self.TRANSPORTE):
            r[97 + k] = s[col] - a[col]
        sa, ss = a[self.COL_SBIT], s[self.COL_SBIT]
        ca, cs = a[self.COL_SACC], s[self.COL_SACC]
        r[114] = sa * (sa - 1)
        r[115] = ss * (ss - 1)
        r[116] = primera_s * sa
        r[117] = primera_s * ca
        r[118] = sigue_s * (cs - (ca + ca + ss))
        esperado = a[self.COL_DELTA] - (a[self.COL_T] - a[self.COL_NACIDO])
        r[119] = per[33] * (cs - esperado)
        return r

    def aserciones(self):
        A, B = self.C_A, self.C_B
        a = [(A + i, 0, 0) for i in range(4)]
        a += [(A + 4 + i, self.ROW_RAIZ, self.pending_root[i]) for i in range(4)]
        a += [(B + 4 + i, self.ROW_RAIZ, self.pmeta_root[i]) for i in range(4)]
        a += [(self.COL_RECEPTOR + i, 0, self.receptor[i]) for i in range(4)]
        a += [(self.COL_IMPORTE, 0, self.importe), (self.COL_T, 0, self.t),
              (self.COL_NACIDO, 0, self.nacido)]
        assert len(a) == 19
        return a


# ───────────────────────────────────────────────────────────────── EdadAir (zk-ssl-air lib.rs)

def _vacios(profundidad):
    v = [[0, 0, 0, 0]]
    for k in range(1, profundidad + 1):
        v.append(S.N.native_merge(v[k - 1], v[k - 1]))
    return v


def raiz_desde_subraiz(sub, m, profundidad=PROFUNDIDAD):
    v = _vacios(profundidad)
    acc = list(sub)
    for k in range(m, profundidad):
        acc = S.N.native_merge(acc, v[k])
    return acc


def m_canonico(n):
    return 3 if n <= 8 else (n - 1).bit_length()


class Edad(Air):
    """A lo sumo `k` posiciones vivas del emisor (o de cualquiera) con edad >= T bajo la cabeza
    (RFC-0007 E4b-2). La unica AIR multisegmento del paquete: un multiconjunto por derivada
    logaritmica en una traza auxiliar de 6 columnas con 2 aleatorios."""
    nombre = "edad"
    ancho, ancho_aux, aleatorios = 83, 6, 2
    num_aserciones_aux = 2
    BITS = 32
    C_A, C_B, C_M, C_P, C_EMISOR, C_NACIDO, C_VIVO = 0, 12, 24, 36, 40, 41, 42
    C_QA, C_QS, C_WINV, C_Z, C_CUENTA, C_CICLO, C_ACT, C_FUERA, C_BITS = (43, 44, 45, 46, 47, 48,
                                                                          49, 50, 51)
    X_H, X_S = 0, 5
    CARRIL_A, CARRIL_B = 1, 2

    def __init__(self, seq, pending_root, pmeta_root, next_pending, t, k, emisor,
                 subraiz_pend, subraiz_meta):
        self.cab_seq, self.pending_root, self.pmeta_root = seq, pending_root, pmeta_root
        self.m = m_canonico(next_pending)
        self.n, self.seq, self.t, self.k = next_pending, seq, t, k
        self.emisor = emisor if emisor is not None else 0
        self.todos = emisor is None
        self.subraiz_pend, self.subraiz_meta = subraiz_pend, subraiz_meta
        self.hojas = 1 << self.m
        self.traza = CICLO << self.m

    def enunciado(self):
        # comprobar_enunciado, y la subida de las subraices (verificar_contra_cabeza)
        if self.m < 3 or self.m > 24:
            raise Rechazo(f"m = {self.m} fuera de 3..=24")
        if self.n > (1 << self.m):
            raise Rechazo(f"n = {self.n} no cabe en 2^{self.m}")
        if self.t >= (1 << self.BITS) or self.seq >= (1 << self.BITS):
            raise Rechazo(f"T = {self.t} o seq = {self.seq} no caben en {self.BITS} bits")
        if self.k > self.n:
            raise Rechazo(f"k = {self.k} mayor que n = {self.n}")
        if raiz_desde_subraiz(self.subraiz_pend, self.m) != list(self.pending_root):
            raise Rechazo("la subraiz de pendientes, subida a 32 niveles, no es el pendingRoot "
                          "de la cabeza")
        if raiz_desde_subraiz(self.subraiz_meta, self.m) != list(self.pmeta_root):
            raise Rechazo("la subraiz de meta, subida a 32 niveles, no es el pmetaRoot de la cabeza")

    def entradas(self):
        return (list(self.subraiz_pend) + list(self.subraiz_meta)
                + [self.m, self.n, self.seq, self.t, self.emisor, int(self.todos), self.k])

    def grados(self):
        c = lambda d: (d, [CICLO])  # noqa: E731
        return ([c(7)] * 36 + [c(1)] * 8 + [c(1)] * 12 + [c(2)] * 4 + [c(2)] * 5 + [c(2)]
                + [c(1) if self.todos else c(3)] + [c(3)] + [(2, [])] * (4 + self.BITS)
                + [c(1)] * 2 + [c(2)] + [(1, [])] + [c(1)] + [c(1)])

    def grados_aux(self):
        c = lambda d: (d, [CICLO])  # noqa: E731
        return [c(2)] * 5 + [c(2), c(2), c(3)] + [c(2)]

    def periodicas(self):
        fila = lambda k: [1 if i == k else 0 for i in range(CICLO)]  # noqa: E731
        cols = [[1] * RONDAS + [0], fila(0), fila(6)]
        for i in range(ESTADO):
            cols.append([S.ARK1[r][i] for r in range(RONDAS)] + [0])
        for i in range(ESTADO):
            cols.append([S.ARK2[r][i] for r in range(RONDAS)] + [0])
        return cols

    def transicion(self, c, s, per):
        r = []
        hf, f0 = per[0], per[1]
        enlace = 1 - hf
        ark1, ark2 = per[3:15], per[15:27]
        for base in (self.C_A, self.C_B, self.C_M):
            r += S.ronda_rescue(c[base:base + 12], s[base:base + 12], ark1, ark2, hf)
        r += [f0 * c[self.C_A + k] for k in range(4)] + [f0 * c[self.C_B + k] for k in range(4)]
        for k in range(ESTADO):
            esperado = {0: S.e2(DOMINIO_META_PENDIENTE), 4: c[self.C_EMISOR],
                        5: c[self.C_NACIDO]}.get(k, S.CERO)
            r.append(f0 * (c[self.C_M + k] - esperado))
        vivo, fuera = c[self.C_VIVO], c[self.C_FUERA]
        r += [f0 * (1 - vivo) * c[self.C_P + k] for k in range(4)]
        r += [f0 * fuera * c[self.C_P + k] for k in range(4)]
        r.append(f0 * fuera * vivo)
        resto, peso = S.CERO, 1
        for b in range(self.BITS):
            resto = resto + c[self.C_BITS + b] * peso
            peso = peso * 2
        r.append(f0 * (1 - c[self.C_QA]) * ((self.t - 1 - self.seq) + c[self.C_NACIDO] - resto))
        qs = c[self.C_QS]
        if self.todos:
            r.append(f0 * (1 - qs))
        else:
            r.append(f0 * (1 - qs) * ((c[self.C_EMISOR] - self.emisor) * c[self.C_WINV] - 1))
        r.append(f0 * (c[self.C_Z] - vivo * c[self.C_QA] * qs))
        for col in (self.C_VIVO, self.C_QA, self.C_QS, self.C_FUERA):
            r.append(c[col] * (c[col] - 1))
        for b in range(self.BITS):
            x = c[self.C_BITS + b]
            r.append(x * (x - 1))
        r.append(hf * (s[self.C_VIVO] - vivo))
        r.append(hf * (s[self.C_FUERA] - fuera))
        r.append(enlace * fuera * (1 - s[self.C_FUERA]))
        r.append(s[self.C_CICLO] - c[self.C_CICLO] - enlace)
        r.append(s[self.C_ACT] - c[self.C_ACT] - enlace * (1 - c[self.C_ACT]))
        r.append(s[self.C_CUENTA] - c[self.C_CUENTA] - f0 * c[self.C_Z])
        assert len(r) == 110
        return r

    def _codificar(self, pot, beta, etiqueta, v, carril):
        return beta - (etiqueta + pot[1] * v[0] + pot[2] * v[1] + pot[3] * v[2] + pot[4] * v[3]
                       + pot[5] * carril)

    @staticmethod
    def _potencias(alfa):
        p = [S.UNO]
        for _ in range(5):
            p.append(p[-1] * alfa)
        return p

    def transicion_aux(self, c, s, x, y, per, aleatorios):
        f0, f6 = per[1], per[2]
        pot = self._potencias(aleatorios[0])
        beta = aleatorios[1]
        dig = lambda fila, col: fila[col:col + 4]  # noqa: E731
        ciclo = c[self.C_CICLO]
        izq = ciclo * 2
        der = izq + 1
        ca, cb = self.CARRIL_A, self.CARRIL_B
        act, vivo = c[self.C_ACT], c[self.C_VIVO]
        A, B = self.C_A, self.C_B
        lecturas = [
            self._codificar(pot, beta, izq, dig(c, A + 4), ca),
            self._codificar(pot, beta, der, dig(c, A + 8), ca),
            self._codificar(pot, beta, izq, dig(c, B + 4), cb),
            self._codificar(pot, beta, der, dig(c, B + 8), cb),
            self._codificar(pot, beta, ciclo + self.hojas, dig(c, self.C_P), ca),
        ]
        m = dig(s, self.C_M + 4)
        hoja_meta = [vivo * m[i] for i in range(4)]
        escrituras = [
            self._codificar(pot, beta, ciclo, dig(s, A + 4), ca),
            self._codificar(pot, beta, ciclo, dig(s, B + 4), cb),
            self._codificar(pot, beta, ciclo + self.hojas, hoja_meta, cb),
        ]
        r = [f0 * (x[self.X_H + k] * lecturas[k] - 1) for k in range(5)]
        r += [f6 * (x[self.X_H + k] * escrituras[k] - 1) for k in range(3)]
        H = self.X_H
        leidas = x[H] + x[H + 1] + x[H + 2] + x[H + 3]
        escritas = act * (x[H] + x[H + 1]) + x[H + 2]
        r.append(y[self.X_S] - x[self.X_S] - f0 * (x[H + 4] - act * leidas) - f6 * escritas)
        return r

    def aserciones(self):
        ultima = self.traza - 1
        n = self.n
        a = [(self.C_CICLO, 0, 0), (self.C_ACT, 0, 0), (self.C_CUENTA, 0, 0),
             (self.C_CUENTA, ultima, self.k)]
        if n >= 1:
            a.append((self.C_FUERA, CICLO * (n - 1), 0))
        if n < self.hojas:
            a.append((self.C_FUERA, CICLO * n, 1))
        raiz = 2 * CICLO - 1
        for k in range(4):
            a.append((self.C_A + 4 + k, raiz, self.subraiz_pend[k]))
            a.append((self.C_B + 4 + k, raiz, self.subraiz_meta[k]))
        return a

    def aserciones_aux(self, aleatorios):
        pot = self._potencias(aleatorios[0])
        a = self._codificar(pot, aleatorios[1], 1, [S.e2(v) for v in self.subraiz_pend],
                            self.CARRIL_A)
        b = self._codificar(pot, aleatorios[1], 1, [S.e2(v) for v in self.subraiz_meta],
                            self.CARRIL_B)
        return [(self.X_S, 0, S.CERO), (self.X_S, self.traza - 1, a.inv() + b.inv())]
