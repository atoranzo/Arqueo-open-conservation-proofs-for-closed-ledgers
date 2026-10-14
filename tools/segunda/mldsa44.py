#!/usr/bin/env python3
"""Verificacion ML-DSA-44 de FIPS 204, en Python y sin dependencias: solo la biblioteca estandar
(`hashlib` trae SHAKE128 y SHAKE256). Solo verificar: ni claves ni firmas se producen aqui.

Es la firma de la nota del medio (RFC-0013 D-C, el tipo 0x06 de `tlog-cosignature`) y la de las
cofirmas de sus testigos: lo que el sobre `ancla-cofirmada` (spec/PAQUETE.md 2.13) pide verificar.

De donde sale: del texto de FIPS 204 (agosto de 2024). Del crate `ml-dsa` 0.1.1 (su fuente, en el
registro de cargo) se leyo, para contrastar y no para copiar, como arma mu con el contexto y donde
rechaza la norma y las pistas mal escritas (al decodificar la firma, `Signature::decode`): lo mismo
que el estandar, en otro orden, y el veredicto es el mismo. Algoritmo 3 (ML-DSA.Verify:
el contexto, a lo sumo 255 bytes, y el prefijo de dominio 0 de la firma PURA, sin pre-hash) sobre el
algoritmo 8 (ML-DSA.Verify_internal); pkDecode (23), sigDecode (27), HintBitUnpack (21), w1Encode
(28), SampleInBall (29), RejNTTPoly (30) con CoeffFromThreeBytes (14), ExpandA (32), Decompose (36),
UseHint (40), NTT (41) y NTT^-1 (42), con zeta = 1753 y el orden de bits invertido de 8 bits. Los
parametros de ML-DSA-44, de la tabla 1: q = 8380417, d = 13, tau = 39, lambda = 128, gamma1 = 2^17,
gamma2 = (q - 1)/88, (k, l) = (4, 4), eta = 2, beta = tau*eta = 78, omega = 80. Clave de 1.312
bytes y firma de 2.420.

Lo que la casa usa, leido del arbol y no supuesto: `crates/zk-ssl-medio/src/nota.rs` verifica con
`verify_with_context(mensaje, &[], firma)` de `ml-dsa` 0.1.1, el algoritmo 3 con el CONTEXTO VACIO,
y firma igual (`sign_randomized` y `sign_deterministic` con `&[]`): mu = H(tr || 0x00 || 0x00 ||
mensaje, 64), con tr = H(pk, 64). Sin HashML-DSA.

Lo que NO afirma: nada de tiempo constante -aqui no hay secretos- ni de rendimiento: de 7 a 18 ms de
media por verificacion, segun la carga, en la maquina del sello que lo escribio. Lo juzga `juez_medio.py`.
"""
import hashlib

Q = 8380417
D = 13
TAU = 39
LAMBDA = 128
GAMMA1 = 1 << 17
GAMMA2 = (Q - 1) // 88
K, L = 4, 4
ETA = 2
BETA = TAU * ETA
OMEGA = 80

BYTES_C = LAMBDA // 4                                   # c~, 32
BYTES_T1 = 32 * 10                                      # un polinomio de t1: 256 x 10 bits
BYTES_Z = 32 * 18                                       # un polinomio de z: 256 x 18 bits (bitlen(gamma1 - 1) + 1)
BYTES_CLAVE_PUBLICA = 32 + K * BYTES_T1                 # 1312
BYTES_FIRMA = BYTES_C + L * BYTES_Z + OMEGA + K         # 2420
assert (BYTES_CLAVE_PUBLICA, BYTES_FIRMA) == (1312, 2420)

M_W1 = (Q - 1) // (2 * GAMMA2)                          # 44: los valores de w1 van de 0 a 43, 6 bits


def _brv8(x):
    return int(f"{x:08b}"[::-1], 2)


# zetas[i] = zeta^brv8(i) mod q, la tabla del apendice B. La entrada 0 no se usa.
ZETAS = [pow(1753, _brv8(i), Q) for i in range(256)]
assert (ZETAS[1], ZETAS[2], ZETAS[128], ZETAS[255]) == (4808194, 3765607, 1753, 7648983), "el apendice B de FIPS 204"
assert pow(1753, 256, Q) == Q - 1, "zeta es raiz 512-esima primitiva de la unidad"
F_INV_256 = 8347681                                     # 256^-1 mod q, la constante del algoritmo 42
assert 256 * F_INV_256 % Q == 1


def H(datos, n):
    return hashlib.shake_256(datos).digest(n)


def ntt(w):
    """Algoritmo 41."""
    w = list(w)
    m, ln = 0, 128
    while ln >= 1:
        for start in range(0, 256, 2 * ln):
            m += 1
            z = ZETAS[m]
            for j in range(start, start + ln):
                t = z * w[j + ln] % Q
                w[j + ln] = (w[j] - t) % Q
                w[j] = (w[j] + t) % Q
        ln //= 2
    return w


def ntt_inversa(w):
    """Algoritmo 42."""
    w = list(w)
    m, ln = 256, 1
    while ln < 256:
        for start in range(0, 256, 2 * ln):
            m -= 1
            z = Q - ZETAS[m]
            for j in range(start, start + ln):
                t = w[j]
                w[j] = (t + w[j + ln]) % Q
                w[j + ln] = (t - w[j + ln]) * z % Q
        ln *= 2
    return [x * F_INV_256 % Q for x in w]


def _enteros(b, bits, cuantos=256):
    """BytesToBits (13) y BitsToInteger (10), dentro de SimpleBitUnpack (18) y BitUnpack (19): el bit j
    del byte i es el bit 8i + j, y cada entero se lee con el bit de menos peso primero. Es leer `b`
    como un entero little-endian."""
    x, mascara = int.from_bytes(b, "little"), (1 << bits) - 1
    return [(x >> (bits * i)) & mascara for i in range(cuantos)]


def pk_decode(pk):
    """Algoritmo 23: rho y t1 (SimpleBitUnpack de 10 bits, bitlen(q - 1) - d)."""
    rho = pk[:32]
    t1 = [_enteros(pk[32 + i * BYTES_T1:32 + (i + 1) * BYTES_T1], 10) for i in range(K)]
    return rho, t1


def hint_bit_unpack(y):
    """Algoritmo 21. None es el bottom del estandar: una codificacion de las pistas que no es la unica
    -cortes que bajan o pasan de omega, indices que no suben estrictamente, relleno que no es cero-.
    Por eso una firma no tiene dos escrituras."""
    h = [[0] * 256 for _ in range(K)]
    indice = 0
    for i in range(K):
        corte = y[OMEGA + i]
        if corte < indice or corte > OMEGA:
            return None
        primero = indice
        while indice < corte:
            if indice > primero and y[indice - 1] >= y[indice]:
                return None
            h[i][y[indice]] = 1
            indice += 1
    if any(y[i] != 0 for i in range(indice, OMEGA)):
        return None
    return h


def sig_decode(sigma):
    """Algoritmo 27: c~, z (BitUnpack(., gamma1 - 1, gamma1): gamma1 menos un entero de 18 bits) y h."""
    c_tilde = sigma[:BYTES_C]
    z = []
    for i in range(L):
        trozo = sigma[BYTES_C + i * BYTES_Z:BYTES_C + (i + 1) * BYTES_Z]
        z.append([GAMMA1 - v for v in _enteros(trozo, 18)])
    h = hint_bit_unpack(sigma[BYTES_C + L * BYTES_Z:])
    return c_tilde, z, h


class _Lector:
    """Un XOF leido en orden, como Squeeze: pide mas al hash cuando se acaba lo leido."""

    def __init__(self, xof, datos, n):
        self.xof, self.datos, self.n = xof, datos, n
        self.buf, self.pos = xof(datos).digest(n), 0

    def leer(self, k):
        while self.pos + k > len(self.buf):
            self.n *= 2
            self.buf = self.xof(self.datos).digest(self.n)
        b = self.buf[self.pos:self.pos + k]
        self.pos += k
        return b


def rej_ntt_poly(semilla):
    """Algoritmo 30, con CoeffFromThreeBytes (14): G = SHAKE128, tres bytes por candidato, el bit alto
    del tercero fuera, y fuera lo que no es menor que q."""
    g = _Lector(hashlib.shake_128, semilla, 840)
    a = []
    while len(a) < 256:
        b0, b1, b2 = g.leer(3)
        z = ((b2 & 0x7f) << 16) | (b1 << 8) | b0
        if z < Q:
            a.append(z)
    return a


def expand_a(rho):
    """Algoritmo 32: A[r][s] = RejNTTPoly(rho || s || r), ya en el dominio NTT."""
    return [[rej_ntt_poly(rho + bytes([s, r])) for s in range(L)] for r in range(K)]


def sample_in_ball(rho):
    """Algoritmo 29, con H = SHAKE256 sobre c~ entero: ocho bytes de signos y, para cada i de 256 - tau
    a 255, un j <= i por rechazo."""
    h = _Lector(hashlib.shake_256, rho, 136)
    signos = int.from_bytes(h.leer(8), "little")
    c = [0] * 256
    for i in range(256 - TAU, 256):
        j = h.leer(1)[0]
        while j > i:
            j = h.leer(1)[0]
        c[i] = c[j]
        c[j] = Q - 1 if (signos >> (i + TAU - 256)) & 1 else 1
    return c


def decompose(r):
    """Algoritmo 36: r = r1 * 2 gamma2 + r0, con r0 en (-gamma2, gamma2] y el caso del borde q - 1."""
    rp = r % Q
    r0 = rp % (2 * GAMMA2)
    if r0 > GAMMA2:
        r0 -= 2 * GAMMA2
    if rp - r0 == Q - 1:
        return 0, r0 - 1
    return (rp - r0) // (2 * GAMMA2), r0


def use_hint(h, r):
    """Algoritmo 40."""
    r1, r0 = decompose(r)
    if h == 1:
        return (r1 + 1) % M_W1 if r0 > 0 else (r1 - 1) % M_W1
    return r1


def w1_encode(w1):
    """Algoritmo 28: SimpleBitPack de cada polinomio con b = 43, seis bits por coeficiente."""
    out = b""
    for p in w1:
        x = 0
        for i, v in enumerate(p):
            x |= v << (6 * i)
        out += x.to_bytes(192, "little")
    return out


def norma_infinita(z):
    """|z|_inf con los coeficientes centrados: z ya viene de sig_decode como entero con signo."""
    return max(abs(v) for p in z for v in p)


def verificar_interno(pk, m_prima, sigma):
    """Algoritmo 8, ML-DSA.Verify_internal. Devuelve True o False."""
    if len(pk) != BYTES_CLAVE_PUBLICA or len(sigma) != BYTES_FIRMA:
        return False
    rho, t1 = pk_decode(pk)
    c_tilde, z, h = sig_decode(sigma)
    if h is None:
        return False
    if norma_infinita(z) >= GAMMA1 - BETA:
        return False
    a = expand_a(rho)
    tr = H(pk, 64)
    mu = H(tr + m_prima, 64)
    c = sample_in_ball(c_tilde)
    z_ntt = [ntt([v % Q for v in p]) for p in z]
    c_ntt = ntt(c)
    w1 = []
    for r in range(K):
        t1_ntt = ntt([v << D for v in t1[r]])  # t1 * 2^d: menor que q, (2^10 - 1) * 2^13 = q - 1
        acc = [0] * 256
        for s in range(L):
            fila = a[r][s]
            zs = z_ntt[s]
            for i in range(256):
                acc[i] += fila[i] * zs[i]
        w_aprox = ntt_inversa([(acc[i] - c_ntt[i] * t1_ntt[i]) % Q for i in range(256)])
        w1.append([use_hint(h[r][i], w_aprox[i]) for i in range(256)])
    return H(mu + w1_encode(w1), BYTES_C) == c_tilde


def verificar(pk, mensaje, sigma, ctx=b""):
    """Algoritmo 3, ML-DSA.Verify: M' = 0x00 || |ctx| || ctx || M, la firma pura. La casa lo llama con
    el contexto vacio (`nota.rs`, `verify_with_context(mensaje, &[], firma)`)."""
    if len(ctx) > 255:
        return False
    return verificar_interno(bytes(pk), bytes([0, len(ctx)]) + bytes(ctx) + bytes(mensaje), bytes(sigma))
