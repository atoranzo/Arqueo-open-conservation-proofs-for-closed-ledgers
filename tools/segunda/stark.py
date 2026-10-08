#!/usr/bin/env python3
"""Verificador STARK de la segunda implementacion (BACKLOG 85, cuarto hito), en Python y sin
dependencias: solo la biblioteca estandar y `nucleo.py`.

Que es. Otro codigo para lo que el kit verifica con `winter-verifier`: las pruebas STARK de las
formas del paquete de evidencia que llevan prueba. Esas pruebas van OCULTAS (`Oculta<AIR>`, la
marca `arqueo:oculta:1` en el meta de la traza) y con SAL (`MerkleConSal`: cada hoja es
`merge(item, sal)` y la sal viaja en cada apertura). Este fichero es la maquinaria, comun a todas;
las AIR viven en `airs.py` y el juez es `juez_stark.py`.

De donde sale cada pieza, y de donde NO:
  - la maquinaria -el formato de `Proof`, la moneda publica sobre Blake3, el orden de Fiat-Shamir,
    la autenticacion Merkle por lotes, el chequeo fuera del dominio, la composicion DEEP, FRI, los
    divisores y las aserciones de frontera-: leida de las fuentes de winterfell 0.13.1 (las crates
    publicadas `winter-fri`, `winter-crypto`, `winter-math` y `winter-utils`, y la copia de
    `winter-air` y `winter-verifier` en `crates/`). Es la primitiva, como Rescue en `nucleo.py`.
  - la envoltura `Oculta` (L = 2T, una columna mas, exenciones + T, el cociente que avanza de
    L - m en L - m) y la sal: de `crates/winter-air/src/air/oculta.rs`, `marca.rs`, `context.rs`
    y `crates/zk-ssl-air/src/sal.rs`.
  - Blake3 y el campo: de `nucleo.py`.

Lo que NO afirma: nada sobre las AIR (ver `airs.py`), ni sobre lo que el mando comprueba fuera del
STARK (la cabeza, la firma, los campos del sobre): eso es `verificador.py` y el binario.
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import nucleo as N  # noqa: E402

P = N.P
blake3 = N.blake3

# ─── el campo de Goldilocks y su extension cuadratica (winter-math 0.13.1, f64) ────────────────
GENERADOR = 7                        # f64::GENERATOR, el desplazamiento del dominio LDE
DOS_ADICIDAD = 32
RAIZ_DOS_ADICA = 7277203076849721926  # f64::TWO_ADIC_ROOT_OF_UNITY


def raiz_unidad(log_n):
    """La raiz primitiva de orden 2^log_n (`get_root_of_unity`)."""
    return pow(RAIZ_DOS_ADICA, 1 << (DOS_ADICIDAD - log_n), P)


def inv(x):
    return pow(x % P, P - 2, P)


class E2:
    """Un elemento de QuadExtension<f64>: a + b·φ con φ² = φ - 2 (`ExtensibleField<2>::mul`)."""
    __slots__ = ("a", "b")

    def __init__(self, a, b=0):
        self.a = a % P
        self.b = b % P

    def __add__(self, o):
        if isinstance(o, int):
            return E2(self.a + o, self.b)
        return E2(self.a + o.a, self.b + o.b)

    __radd__ = __add__

    def __sub__(self, o):
        if isinstance(o, int):
            return E2(self.a - o, self.b)
        return E2(self.a - o.a, self.b - o.b)

    def __rsub__(self, o):
        return E2(o - self.a, -self.b)

    def __neg__(self):
        return E2(-self.a, -self.b)

    def __mul__(self, o):
        if isinstance(o, int):
            return E2(self.a * o, self.b * o)
        a0b0 = self.a * o.a
        return E2(a0b0 - 2 * self.b * o.b, (self.a + self.b) * (o.a + o.b) - a0b0)

    __rmul__ = __mul__

    def inv(self):
        # la norma x·frob(x), con frob(a + bφ) = (a + b) - bφ, cae en el campo base
        n = (self.a * self.a + self.a * self.b + 2 * self.b * self.b) % P
        if n == 0:
            raise ZeroDivisionError("inverso de cero en la extension")
        ni = inv(n)
        return E2((self.a + self.b) * ni, -self.b * ni)

    def __truediv__(self, o):
        if isinstance(o, int):
            return self * inv(o)
        return self * o.inv()

    def __pow__(self, e):
        r, b = E2(1), self
        while e:
            if e & 1:
                r = r * b
            b = b * b
            e >>= 1
        return r

    def __eq__(self, o):
        if isinstance(o, int):
            return self.b == 0 and self.a == o % P
        return self.a == o.a and self.b == o.b

    def __hash__(self):
        return hash((self.a, self.b))

    def __repr__(self):
        return f"E2({self.a}, {self.b})"

    def bytes(self):
        return self.a.to_bytes(8, "little") + self.b.to_bytes(8, "little")


CERO, UNO = E2(0), E2(1)


def e2(x):
    return x if isinstance(x, E2) else E2(x)


# ─── el veredicto ──────────────────────────────────────────────────────────────────────────────

class Rechazo(Exception):
    """ROJO, con el nombre del error de `winter-verifier` (su `Debug`) que corresponde."""


def deser(msg):
    return Rechazo(f'ProofDeserializationError("{msg}")')


# ─── la serializacion de winter-utils 0.13.1 ──────────────────────────────────────────────────

class Lector:
    """`SliceReader` con la cota del §575: ninguna cuenta se cree mas alla de los bytes que hay."""

    def __init__(self, b):
        self.b = b
        self.p = 0

    def resto(self):
        return len(self.b) - self.p

    def tomar(self, n):
        if n < 0 or n > self.resto():
            raise deser("unexpected EOF")
        v = self.b[self.p:self.p + n]
        self.p += n
        return v

    def u8(self):
        return self.tomar(1)[0]

    def u16(self):
        return int.from_bytes(self.tomar(2), "little")

    def u32(self):
        return int.from_bytes(self.tomar(4), "little")

    def u64(self):
        return int.from_bytes(self.tomar(8), "little")

    def usize(self):
        """vint64 (`read_usize`): la longitud es trailing_zeros(primer byte) + 1."""
        if self.resto() < 1:
            raise deser("unexpected EOF")
        b0 = self.b[self.p]
        largo = 9 if b0 == 0 else ((b0 & -b0).bit_length())
        if largo == 9:
            self.tomar(1)
            return self.u64()
        return int.from_bytes(self.tomar(largo), "little") >> largo


def base_de(b):
    v = int.from_bytes(b, "little")
    if v >= P:
        raise deser("invalid field element")
    return v


def bases(b):
    return [base_de(b[i:i + 8]) for i in range(0, len(b), 8)]


def e2s(b):
    return [E2(base_de(b[i:i + 8]), base_de(b[i + 8:i + 16])) for i in range(0, len(b), 16)]


def bytes_base(xs):
    return b"".join(x.to_bytes(8, "little") for x in xs)


def bytes_e2(xs):
    return b"".join(x.bytes() for x in xs)


# ─── la prueba (winter-air proof/mod.rs, en el orden de `write_into`) ─────────────────────────

OPCIONES_KIT = bytes([42, 16, 21, 2, 8, 31, 0, 0, 1, 1])  # zk-ssl-air lib.rs `opciones()`
MODULO = (P).to_bytes(8, "little")
PREFIJO_MARCA = b"arqueo:oculta:1"            # winter-air marca.rs `PREFIJO`
LARGO_MARCA = len(PREFIJO_MARCA) + 4          # `LARGO`: el prefijo y los cuatro bytes de m
M_OCULTACION = 64                             # zk-ssl-air lib.rs `M_OCULTACION` (§651)


class Prueba:
    pass


def leer_queries(r):
    q = Prueba()
    q.valores = r.tomar(r.usize())
    q.apertura = r.tomar(r.usize())
    return q


def leer_prueba(b):
    r = Lector(b)
    p = Prueba()
    # Context: TraceInfo, el modulo, las opciones y el numero de restricciones
    p.ancho, p.ancho_aux, p.aleatorios, log_l = r.u8(), r.u8(), r.u8(), r.u8()
    if p.ancho == 0 or log_l > 62:
        raise deser("trace info")
    p.meta = r.tomar(r.u16())
    p.L = 1 << log_l
    p.modulo = r.tomar(r.u8())
    p.opciones = r.tomar(10)
    p.num_restricciones = r.usize()
    p.consultas_unicas = r.u8()
    p.compromisos = r.tomar(r.u16())
    p.segmentos = 2 if p.ancho_aux > 0 else 1
    p.traza = [leer_queries(r) for _ in range(p.segmentos)]
    p.cociente = leer_queries(r)
    p.ood_traza = r.tomar(r.u16())
    p.ood_cociente = r.tomar(r.u16())
    capas = r.u8()
    p.fri = [(r.tomar(r.u32()), r.tomar(r.u32())) for _ in range(capas)]
    p.resto = r.tomar(r.u16())
    p.fri_particiones = 1 << r.u8()
    p.nonce = r.u64()
    return p


def lectura_de_marca(meta):
    """`Marca::leer` del fork, un solo lector: devuelve lo que lee escrito como lo escribe su
    `Debug` -el texto que el kit pone en su rechazo- y la m, si la hay. Vacio: `Ok(None)`; sin el
    prefijo: `Err(Desconocida)`; con el prefijo y otra longitud que 19: `Err(Largo)`; si no,
    `Ok(Some(Marca { m: .. }))`."""
    if not meta:
        return "Ok(None)", None
    if not meta.startswith(PREFIJO_MARCA):
        return "Err(Desconocida)", None
    if len(meta) != LARGO_MARCA:
        return "Err(Largo)", None
    m = int.from_bytes(meta[len(PREFIJO_MARCA):], "little")
    return f"Ok(Some(Marca {{ m: {m} }}))", m


def comprobar_marca(meta):
    """`zk_ssl_air::comprobar_marca` (§651, §706): la marca es parte de la FORMA, y solo la de la
    casa, con m = M_OCULTACION, es una prueba que se juzga. Un meta vacio no es una prueba sin
    ocultar: es un rechazo. El texto es el del kit, letra por letra, para que las dos
    implementaciones fallen por la misma causa (spec/PAQUETE.md, seccion 6.1)."""
    lectura, m = lectura_de_marca(meta)
    if m != M_OCULTACION:
        raise Rechazo(f"marca de la ocultacion {lectura}; el enunciado pide m = {M_OCULTACION}")


def marca_de(meta):
    """El despacho de `winter-verifier` por `Marca::leer`: vacio -> sin marca; la marca -> su m;
    otra cosa -> error. Detras de `comprobar_marca` solo le llega la de la casa."""
    lectura, m = lectura_de_marca(meta)
    if lectura.startswith("Err"):
        raise deser("meta de traza desconocido")
    return m


# ─── la semilla de Fiat-Shamir (Context::to_elements ++ pub_inputs.to_elements) ───────────────

def con_relleno(trozo):
    return int.from_bytes(trozo + bytes(8 - len(trozo)), "little")


def elementos_del_contexto(p):
    e = []
    buf = p.ancho
    buf = (buf << 8) | (1 if p.ancho_aux > 0 else 0)
    if p.ancho_aux > 0:
        buf = (buf << 8) | p.ancho_aux
        buf = (buf << 8) | p.aleatorios
    e.append(buf)
    e.append(p.L & 0xFFFFFFFF)
    for i in range(0, len(p.meta), 7):
        e.append(con_relleno(p.meta[i:i + 7]))
    mitad = len(p.modulo) // 2
    e.append(con_relleno(p.modulo[:mitad]))
    e.append(con_relleno(p.modulo[mitad:]))
    e.append(p.num_restricciones & 0xFFFFFFFF)
    o = p.opciones
    buf = (((o[3] << 8 | o[4]) << 8 | o[5]) << 8) | o[1]
    e += [buf, o[2], o[0]]
    return e


# ─── la moneda publica: DefaultRandomCoin<Blake3_256<f64>> ─────────────────────────────────────

def hash_base(xs):
    return blake3(bytes_base(xs))


def hash_e2(xs):
    return blake3(bytes_e2(xs))


def merge(a, b):
    return blake3(a + b)


class Moneda:
    def __init__(self, semilla):
        self.semilla = hash_base(semilla)
        self.contador = 0

    def resembrar(self, d):
        self.semilla = merge(self.semilla, d)
        self.contador = 0

    def _siguiente(self):
        self.contador += 1
        return blake3(self.semilla + self.contador.to_bytes(8, "little"))

    def sacar(self):
        for _ in range(1000):
            v = self._siguiente()
            a, b = int.from_bytes(v[:8], "little"), int.from_bytes(v[8:16], "little")
            if a < P and b < P:
                return E2(a, b)
        raise Rechazo("RandomCoinError")

    def ceros(self, nonce):
        """`check_leading_zeros`: cuenta los ceros FINALES de la primera palabra LE."""
        v = int.from_bytes(blake3(self.semilla + nonce.to_bytes(8, "little"))[:8], "little")
        return 64 if v == 0 else (v & -v).bit_length() - 1

    def enteros(self, num, dominio, nonce):
        self.semilla = blake3(self.semilla + nonce.to_bytes(8, "little"))
        self.contador = 0
        mascara = dominio - 1
        vs = []
        for _ in range(1000):
            vs.append(int.from_bytes(self._siguiente()[:8], "little") & mascara)
            if len(vs) == num:
                break
        if len(vs) < num:
            raise Rechazo("RandomCoinError")
        return vs


# ─── el compromiso vectorial: BatchMerkleProof (winter-crypto) bajo MerkleConSal (zk-ssl-air) ──

class Lote:
    pass


def leer_lote(r):
    """`VariasConSal::read_from`: el lote Octopus con sus cuentas acotadas y la lista de sales."""
    lote = Lote()
    lote.profundidad = r.u8()
    vectores = r.usize()
    if vectores > r.resto():
        raise deser("unexpected EOF")
    lote.nodos = []
    for _ in range(vectores):
        k = r.usize()
        if k > r.resto():
            raise deser("unexpected EOF")
        lote.nodos.append([r.tomar(32) for _ in range(k)])
    n = r.u32()
    if n > (1 << lote.profundidad) or n > r.resto():
        raise deser("sales")
    lote.sales = [r.tomar(32) for _ in range(n)]
    return lote


class Invalida(Exception):
    pass


def raiz_del_lote(lote, indices, hojas):
    """`BatchMerkleProof::get_root`, tal cual: con su `nodes[i]` indexado por la posicion en la
    lista del nivel, que es como el probador lo construye."""
    if not indices:
        raise Invalida()
    prof = lote.profundidad
    num_hojas = 1 << prof
    mapa = {}
    for i, ix in enumerate(indices):
        if ix >= num_hojas:
            raise Invalida()
        mapa[ix] = i
    if len(mapa) != len(indices):
        raise Invalida()
    norm = sorted({ix - (ix & 1) for ix in indices})
    if len(norm) != len(lote.nodos):
        raise Invalida()
    v = {}
    sig = []
    punteros = []
    for i, ix in enumerate(norm):
        n = lote.nodos[i]
        if ix in mapa:
            if len(hojas) <= mapa[ix]:
                raise Invalida()
            izq = hojas[mapa[ix]]
            if ix + 1 in mapa:
                if len(hojas) <= mapa[ix + 1]:
                    raise Invalida()
                der = hojas[mapa[ix + 1]]
                punteros.append(0)
            else:
                if not n:
                    raise Invalida()
                der = n[0]
                punteros.append(1)
        else:
            if not n:
                raise Invalida()
            izq = n[0]
            if ix + 1 not in mapa or len(hojas) <= mapa[ix + 1]:
                raise Invalida()
            der = hojas[mapa[ix + 1]]
            punteros.append(1)
        padre = (num_hojas + ix) >> 1
        v[padre] = merge(izq, der)
        sig.append(padre)
    for _ in range(1, prof):
        actuales, sig = sig, []
        i = 0
        while i < len(actuales):
            nodo = actuales[i]
            hermano_ix = nodo ^ 1
            if i + 1 < len(actuales) and actuales[i + 1] == hermano_ix:
                if hermano_ix not in v:
                    raise Invalida()
                hermano = v[hermano_ix]
                i += 1
            else:
                pt = punteros[i]
                if len(lote.nodos[i]) <= pt:
                    raise Invalida()
                hermano = lote.nodos[i][pt]
                punteros[i] += 1
            if nodo not in v:
                raise Invalida()
            par = (hermano, v[nodo]) if nodo & 1 else (v[nodo], hermano)
            padre = nodo >> 1
            v[padre] = merge(*par)
            sig.append(padre)
            i += 1
    if 1 not in v:
        raise Invalida()
    return v[1]


def verificar_con_sal(compromiso, indices, items, lote):
    """`MerkleConSal::verify_many`: hojas = merge(item, sal), y el arbol de debajo."""
    if len(items) != len(lote.sales):
        raise Invalida()
    hojas = [merge(it, s) for it, s in zip(items, lote.sales)]
    if raiz_del_lote(lote, indices, hojas) != compromiso:
        raise Invalida()


# ─── utilidades de polinomios ─────────────────────────────────────────────────────────────────

def eval_periodica(valores, y):
    """El polinomio que interpola `valores` sobre las raices n-esimas de la unidad, en `y`
    (barycentrico: p(y) = (y^n - 1)/n · Σ v_i ω^i / (y - ω^i))."""
    n = len(valores)
    if n == 1:
        return e2(valores[0])
    w = raiz_unidad(n.bit_length() - 1)
    yn1 = (y ** n) - 1
    acc = CERO
    wi = 1
    # una inversion por lote (Montgomery) sobre los y - ω^i que pesan
    dens, nums = [], []
    for v in valores:
        if v:
            dens.append(y - wi)
            nums.append(v * wi % P)
        wi = wi * w % P
    invs = inversion_por_lote(dens)
    for num, iv in zip(nums, invs):
        acc = acc + iv * num
    return acc * yn1 * inv(n)


def inversion_por_lote(xs):
    if not xs:
        return []
    pref = [UNO]
    for x in xs:
        pref.append(pref[-1] * x)
    t = pref[-1].inv()
    out = [None] * len(xs)
    for i in range(len(xs) - 1, -1, -1):
        out[i] = t * pref[i]
        t = t * xs[i]
    return out


def lagrange_en(xs, ys, alfa):
    """El valor en `alfa` del polinomio que pasa por (xs, ys): el que `interpolate_batch` devuelve
    y `polynom::eval` evalua. Unico; no importa el algoritmo."""
    total = CERO
    for k in range(len(xs)):
        num, den = UNO, 1
        for m in range(len(xs)):
            if m != k:
                num = num * (alfa - xs[m])
                den = den * (xs[k] - xs[m]) % P
        total = total + ys[k] * num * inv(den)
    return total


# ─── el contexto de la AIR, con o sin Oculta (winter-air air/context.rs, air/oculta.rs) ───────

def grado_de_evaluacion(grado, L):
    base, ciclos = grado
    return base * (L - 1) + sum((L // c) * (c - 1) for c in ciclos)


class Contexto:
    """Lo que `AirContext` le da al verificador. Con marca: el envoltorio `Oculta`."""

    def __init__(self, air, p, m):
        self.air = air
        self.L = p.L
        self.oculta = m is not None
        self.m = m or 0
        T = p.L // 2 if self.oculta else p.L
        self.T = T
        self.grados = air.grados()
        self.grados_aux = air.grados_aux()
        exenciones = air.exenciones()
        self.exenciones = exenciones + T if self.oculta else exenciones
        self.ancho = p.ancho
        self.ancho_aux = p.ancho_aux
        self.gL = raiz_unidad(self.L.bit_length() - 1)
        self.lde = self.L * p.opciones[1]
        self.g_lde = raiz_unidad(self.lde.bit_length() - 1)
        alto = max(grado_de_evaluacion(g, self.L) for g in self.grados + self.grados_aux)
        div = self.L - self.exenciones
        if self.m == 0:
            k = -(-(alto - div) // self.L)
        else:
            k = -(-(alto - div + 1) // (self.L - self.m))
        self.columnas_cociente = max(k, 1)
        self.paso = self.L - self.m

    def num_transiciones(self):
        return len(self.grados) + len(self.grados_aux)


# ─── la verificacion (winter-verifier lib.rs `verify` y `perform_verification`) ──────────────

def capas_fri(lde, opciones):
    resto_max = (opciones[5] + 1) * opciones[1]
    n = 0
    while lde > resto_max:
        lde //= opciones[3 + 1]
        n += 1
    return n


def verificar(prueba, air, opciones_aceptadas=OPCIONES_KIT):
    """Verifica `prueba` (bytes) contra `air` (con sus entradas publicas). Devuelve None o lanza
    `Rechazo` con el nombre del error que daria `winter-verifier`."""
    air.enunciado()
    p = leer_prueba(prueba)
    # el orden del juez de la AIR en el kit (p. ej. banda::verificar), antes de construirla: la
    # forma de traza, la marca (§651) y solo entonces `verify`, que valida antes que nada las
    # opciones. Hasta el §706 aqui iban la forma, las opciones y la marca, sin exigir su m.
    air.forma(p)
    comprobar_marca(p.meta)
    if p.opciones != opciones_aceptadas:
        raise Rechazo("UnacceptableProofOptions")
    semilla = elementos_del_contexto(p) + air.entradas()
    m = marca_de(p.meta)
    # detras de comprobar_marca, m es M_OCULTACION; la guarda queda como la del despacho del fork
    if m is not None and (p.L < 16 or p.ancho < 2 or m >= p.L):
        raise deser("traza oculta mal formada")
    ctx = Contexto(air, p, m)
    # ── VerifierChannel::new ──
    if p.modulo != MODULO:
        raise Rechazo("InconsistentBaseField")
    nfri = capas_fri(ctx.lde, p.opciones)
    esperado = (p.segmentos + 1 + nfri + 1) * 32
    if len(p.compromisos) != esperado:
        raise deser("compromisos")
    c = [p.compromisos[i:i + 32] for i in range(0, len(p.compromisos), 32)]
    comp_traza = c[:p.segmentos]
    comp_cociente = c[p.segmentos]
    comp_fri = c[p.segmentos + 1:]
    nq = p.consultas_unicas
    ncc = ctx.columnas_cociente

    def tabla(q, ancho, bytes_por, leer):
        if len(q.valores) != nq * bytes_por * ancho:
            raise deser("queries")
        r = Lector(q.apertura)
        lote = leer_lote(r)
        if (1 << lote.profundidad) != ctx.lde:
            raise deser("queries domain")
        filas = [leer(q.valores[i * bytes_por * ancho:(i + 1) * bytes_por * ancho])
                 for i in range(nq)]
        return filas, lote

    filas_main, lote_main = tabla(p.traza[0], p.ancho, 8, bases)
    filas_aux = lote_aux = None
    if p.segmentos == 2:
        filas_aux, lote_aux = tabla(p.traza[1], p.ancho_aux, 16, e2s)
    filas_cc, lote_cc = tabla(p.cociente, ncc, 16, e2s)
    if p.fri_particiones != 1:
        raise deser("particiones FRI")
    if len(p.resto) % 16 or len(p.resto) == 0:
        raise deser("resto")
    resto = e2s(p.resto)
    if len(resto) & (len(resto) - 1):
        raise deser("resto")
    plegado = p.opciones[4]
    if len(p.fri) != nfri:
        raise deser("capas FRI")
    capas = []
    dom = ctx.lde
    for valores, caminos in p.fri:
        if len(valores) % (16 * plegado):
            raise deser("capa FRI")
        lote = leer_lote(Lector(caminos))
        if (1 << lote.profundidad) != dom // plegado:
            raise deser("capa FRI domain")
        vs = e2s(valores)
        capas.append(([vs[i:i + plegado] for i in range(0, len(vs), plegado)], lote))
        dom //= plegado
    ancho_t = p.ancho + p.ancho_aux
    tb, qb = p.ood_traza, p.ood_cociente
    if not tb or tb[0] != 2 or len(tb) != 1 + 2 * ancho_t * 16:
        raise deser("ood traza")
    if not qb or qb[0] != 2 or len(qb) != 1 + 2 * ncc * 16:
        raise deser("ood cociente")
    ot = e2s(tb[1:])
    oq = e2s(qb[1:])
    ood_act, ood_sig = ot[:ancho_t], ot[ancho_t:]
    oq_act, oq_sig = oq[:ncc], oq[ncc:]

    # ── perform_verification ──
    moneda = Moneda(semilla)
    moneda.resembrar(comp_traza[0])
    aleatorios = None
    if p.segmentos == 2:
        aleatorios = [moneda.sacar() for _ in range(p.aleatorios)]
        moneda.resembrar(comp_traza[1])
    coef = [moneda.sacar() for _ in range(ctx.num_transiciones() + air.num_aserciones())]
    moneda.resembrar(comp_cociente)
    z = moneda.sacar()

    h1 = evaluar_restricciones(ctx, air, coef, ood_act, ood_sig, aleatorios, z, p.ancho)
    h2 = CERO
    for i, v in enumerate(oq_act):
        h2 = h2 + (z ** ((i * ctx.paso) & 0xFFFFFFFF)) * v
    if h1 != h2:
        raise Rechazo("InconsistentOodConstraintEvaluations")

    moneda.resembrar(hash_e2(ood_act + oq_act + ood_sig + oq_sig))
    deep = [moneda.sacar() for _ in range(ancho_t + ncc)]

    # FriVerifier::new
    alfas = []
    max_grado_mas_1 = ctx.L
    for profundidad, compromiso in enumerate(comp_fri):
        moneda.resembrar(compromiso)
        alfas.append(moneda.sacar())
        if profundidad != len(comp_fri) - 1 and max_grado_mas_1 % plegado:
            raise Rechazo("FriVerificationFailed(DegreeTruncation)")
        max_grado_mas_1 //= plegado

    if moneda.ceros(p.nonce) < p.opciones[2]:
        raise Rechazo("QuerySeedProofOfWorkVerificationFailed")
    posiciones = sorted(set(moneda.enteros(p.opciones[0], ctx.lde, p.nonce)))

    # las consultas de la traza y del cociente, contra sus compromisos
    try:
        verificar_con_sal(comp_traza[0], posiciones, [hash_base(f) for f in filas_main], lote_main)
        if p.segmentos == 2:
            verificar_con_sal(comp_traza[1], posiciones, [hash_e2(f) for f in filas_aux], lote_aux)
    except Invalida:
        raise Rechazo("TraceQueryDoesNotMatchCommitment")
    try:
        verificar_con_sal(comp_cociente, posiciones, [hash_e2(f) for f in filas_cc], lote_cc)
    except Invalida:
        raise Rechazo("ConstraintQueryDoesNotMatchCommitment")

    evaluaciones = componer_deep(ctx, posiciones, z, deep, filas_main, filas_aux, filas_cc,
                                 ood_act, ood_sig, oq_act, oq_sig, p.ancho, p.ancho_aux)
    verificar_fri(ctx, posiciones, evaluaciones, capas, comp_fri, alfas, resto, plegado)


def evaluar_restricciones(ctx, air, coef, act, sig, aleatorios, z, ancho):
    """`evaluate_constraints` en z: la combinacion de las transiciones entre su divisor, mas las
    aserciones de frontera, cada una entre (z - g^paso)."""
    L = ctx.L
    periodicas = []
    for col in air.periodicas():
        periodicas.append(eval_periodica(col, z ** (L // len(col))))
    main_act, main_sig = act[:ancho], sig[:ancho]
    t_main = air.transicion(main_act, main_sig, periodicas)
    if len(t_main) != len(ctx.grados):
        raise RuntimeError(f"{air.nombre}: {len(t_main)} transiciones, se declaran {len(ctx.grados)}")
    n_t = ctx.num_transiciones()
    c_trans, c_front = coef[:n_t], coef[n_t:]
    acc = CERO
    for v, cf in zip(t_main, c_trans):
        acc = acc + cf * v
    if ctx.grados_aux:
        t_aux = air.transicion_aux(main_act, main_sig, act[ancho:], sig[ancho:], periodicas,
                                   aleatorios)
        if len(t_aux) != len(ctx.grados_aux):
            raise RuntimeError(f"{air.nombre}: {len(t_aux)} auxiliares")
        for v, cf in zip(t_aux, c_trans[len(ctx.grados):]):
            acc = acc + cf * v
    # divisor de las transiciones: (z^L - 1) / Π (z - g^k) sobre las ultimas `exenciones` filas
    num = (z ** L) - 1
    den = UNO
    gk = pow(ctx.gL, L - ctx.exenciones, P)
    for _ in range(ctx.exenciones):
        den = den * (z - gk)
        gk = gk * ctx.gL % P
    resultado = acc * den / num
    # aserciones: ordenadas como `prepare_assertions` (paso, fila, columna) y emparejadas con su
    # coeficiente en ese orden; la principal antes que la auxiliar
    principales = sorted(air.aserciones(), key=lambda a: (0, a[1], a[0]))
    auxiliares = sorted(air.aserciones_aux(aleatorios), key=lambda a: (0, a[1], a[0]))
    if len(principales) + len(auxiliares) != len(c_front):
        raise RuntimeError(f"{air.nombre}: cuenta de aserciones")
    for (col, fila, valor), cf in zip(principales, c_front):
        resultado = resultado + cf * (act[col] - valor) / (z - pow(ctx.gL, fila, P))
    for (col, fila, valor), cf in zip(auxiliares, c_front[len(principales):]):
        resultado = resultado + cf * (act[ancho + col] - valor) / (z - pow(ctx.gL, fila, P))
    return resultado


def componer_deep(ctx, posiciones, z, cc, filas_main, filas_aux, filas_cc, ood_act, ood_sig,
                  oq_act, oq_sig, ancho, ancho_aux):
    """`DeepComposer::compose_columns`."""
    zg = z * ctx.gL
    out = []
    for j, pos in enumerate(posiciones):
        x = pow(ctx.g_lde, pos, P) * GENERADOR % P
        d1, d2 = e2(x) - z, e2(x) - zg
        t1, t2 = CERO, CERO
        for i, v in enumerate(filas_main[j]):
            t1 = t1 + (e2(v) - ood_act[i]) * cc[i]
            t2 = t2 + (e2(v) - ood_sig[i]) * cc[i]
        num = t1 * d2 + t2 * d1
        if filas_aux is not None:
            t1, t2 = CERO, CERO
            for i, v in enumerate(filas_aux[j]):
                t1 = t1 + (v - ood_act[ancho + i]) * cc[ancho + i]
                t2 = t2 + (v - ood_sig[ancho + i]) * cc[ancho + i]
            num = num + t1 * d2 + t2 * d1
        t1, t2 = CERO, CERO
        base = ancho + ancho_aux
        for i, v in enumerate(filas_cc[j]):
            t1 = t1 + (v - oq_act[i]) * cc[base + i]
            t2 = t2 + (v - oq_sig[i]) * cc[base + i]
        num = num + t1 * d2 + t2 * d1
        out.append(num / (d1 * d2))
    return out


def verificar_fri(ctx, posiciones, evaluaciones, capas, comp_fri, alfas, resto, N):
    """`FriVerifier::verify_generic::<N>` y el resto."""
    dominio = ctx.lde
    g = ctx.g_lde
    raices = [pow(g, dominio // N * i, P) for i in range(N)]
    max_mas_1 = ctx.L
    posiciones = list(posiciones)
    evaluaciones = list(evaluaciones)
    for prof, (hojas, lote) in enumerate(capas):
        fila = dominio // N
        plegadas = []
        for pos in posiciones:
            q = pos % fila
            if q not in plegadas:
                plegadas.append(q)
        try:
            verificar_con_sal(comp_fri[prof], plegadas, [hash_e2(h) for h in hojas], lote)
        except Invalida:
            raise Rechazo("FriVerificationFailed(LayerCommitmentMismatch)")
        if len(hojas) != len(plegadas):
            raise Rechazo("FriVerificationFailed(LayerCommitmentMismatch)")
        consultadas = [hojas[plegadas.index(pos % fila)][pos // fila] for pos in posiciones]
        if consultadas != evaluaciones:
            raise Rechazo(f"FriVerificationFailed(InvalidLayerFolding({prof}))")
        alfa = alfas[prof]
        nuevas = []
        for q, h in zip(plegadas, hojas):
            xe = pow(g, q, P) * GENERADOR % P
            nuevas.append(lagrange_en([xe * r % P for r in raices], h, alfa))
        if max_mas_1 % N:
            raise Rechazo("FriVerificationFailed(DegreeTruncation)")
        g = pow(g, N, P)
        max_mas_1 //= N
        dominio //= N
        posiciones = plegadas
        evaluaciones = nuevas
    if len(resto) > max_mas_1:
        raise Rechazo(f"FriVerificationFailed(RemainderDegreeMismatch({max_mas_1 - 1}))")
    for pos, ev in zip(posiciones, evaluaciones):
        x = GENERADOR * pow(g, pos, P) % P
        acc = CERO
        for coef in resto:
            acc = acc * x + coef
        if acc != ev:
            raise Rechazo("FriVerificationFailed(InvalidRemainderFolding)")


# ─── Rescue en el circuito: la ronda por encuentro en el medio (zk-ssl-air lib.rs) ────────────

def _invertir(M):
    n = len(M)
    A = [list(r) + [1 if i == j else 0 for j in range(n)] for i, r in enumerate(M)]
    for c in range(n):
        piv = next(r for r in range(c, n) if A[r][c] % P)
        A[c], A[piv] = A[piv], A[c]
        iv = inv(A[c][c])
        A[c] = [x * iv % P for x in A[c]]
        for r in range(n):
            if r != c and A[r][c]:
                f = A[r][c]
                A[r] = [(x - f * y) % P for x, y in zip(A[r], A[c])]
    return [r[n:] for r in A]


MDS = N.MDS
INV_MDS = _invertir(MDS)
ARK1, ARK2 = N.ARK1, N.ARK2


def sbox(x):
    x2 = x * x
    x4 = x2 * x2
    return x4 * x2 * x


def ronda_rescue(act, sig, ark1, ark2, bandera):
    """`ronda_rescue`: bandera · (sbox(B) - (A + ark1)), con A = MDS·sbox(act) y
    B = INV_MDS·(sig - ark2)."""
    s = [sbox(act[j]) for j in range(12)]
    d = [sig[j] - ark2[j] for j in range(12)]
    out = []
    for i in range(12):
        a, b = CERO, CERO
        for j in range(12):
            a = a + s[j] * MDS[i][j]
            b = b + d[j] * INV_MDS[i][j]
        out.append(bandera * (sbox(b) - (a + ark1[i])))
    return out


def autotest():
    # la extension: φ² = φ - 2, y el inverso por la norma
    phi = E2(0, 1)
    assert phi * phi == E2(-2, 1)
    x = E2(123456789, 987654321)
    assert x * x.inv() == UNO
    # INV_MDS, contra la tabla de winter-crypto si esta a mano
    for i in range(12):
        for j in range(12):
            assert sum(MDS[i][k] * INV_MDS[k][j] for k in range(12)) % P == (1 if i == j else 0)
    ruta = os.path.expanduser("~/.cargo/registry/src")
    if os.path.isdir(ruta):
        import glob
        import re
        for f in glob.glob(ruta + "/*/winter-crypto-0.13.1/src/hash/rescue/rp64_256/mod.rs"):
            src = open(f).read()
            bloque = src[src.index("\nconst INV_MDS: [[BaseElement"):]
            nums = [int(n) for n in re.findall(r"BaseElement::new\((\d+)\)", bloque)[:144]]
            assert nums == [v for fila in INV_MDS for v in fila], "INV_MDS no es la de winter-crypto"
    # la raiz de la unidad tiene el orden que dice
    w = raiz_unidad(10)
    assert pow(w, 1024, P) == 1 and pow(w, 512, P) != 1
    # la marca (§706): la de la casa pasa; cualquier otra cae con el texto del kit, que escribe lo
    # leido con el `Debug` de `Result<Option<Marca>, MarcaError>`
    casa = PREFIJO_MARCA + M_OCULTACION.to_bytes(4, "little")
    assert comprobar_marca(casa) is None and marca_de(casa) == M_OCULTACION
    for meta, lectura in ((b"", "Ok(None)"),
                          (PREFIJO_MARCA + (32).to_bytes(4, "little"), "Ok(Some(Marca { m: 32 }))"),
                          (b"arqueo:oculta:2" + casa[15:], "Err(Desconocida)"),
                          (casa + b"\x00", "Err(Largo)")):
        try:
            comprobar_marca(meta)
            raise AssertionError(f"la marca {meta!r} pasa")
        except Rechazo as e:
            assert str(e) == f"marca de la ocultacion {lectura}; el enunciado pide m = 64", str(e)
    return True


if __name__ == "__main__":
    autotest()
    print("stark.py: autotest OK (extension, INV_MDS calculada = tabla de winter-crypto, raices, marca)")
