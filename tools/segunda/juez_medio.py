#!/usr/bin/env python3
"""El juez del MEDIO, con la segunda implementacion (BACKLOG 85; RFC-0013): ML-DSA-44 de FIPS 204
(`mldsa44.py`), la nota checkpoint con su firma 0x06 y la inclusion de RFC 9162 (`medio.py`), contra
lo que escribieron otras manos. El sobre `ancla-cofirmada` lo juzga `tools/conformidad.sh` con su
MANIFIESTO, como a las otras formas de `verificador.py`; aqui se juzgan sus piezas.

    python3 tools/segunda/juez_medio.py

Lo que comprueba, por bloques:
  1. las 33 notas de `crates/zk-ssl-medio/tests/vectores/notas/` (§632) -firmadas por torchwood sobre
     el `crypto/mldsa` de Go y, las `rust-*`, por el crate `ml-dsa` del arbol-: el veredicto de cada
     una es el de la columna 2 de su MANIFIESTO, la variante de `ErrorDeNota` incluida; la cofirma del
     testigo verifica con su vkey, y una tocada no; y la raiz de cada nota de torchwood es el MTH de
     sus hojas sinteticas;
  2. un FIRMANTE DE PRUEBA de FIPS 204 (KeyGen y Sign, algoritmos 6 y 7, aqui y solo aqui: ni
     `mldsa44.py` ni `medio.py` firman) que, con las semillas de `torchwood/main.go`, rehace las dos
     vkeys y, en determinista, las lineas del publicador y del testigo de las notas de torchwood BYTE
     A BYTE. Con el, dos firmas de FRONTERA: una con |z|_inf = gamma1 - beta, que todo lo demas da por
     buena y la norma tiene que rechazar, y otra con gamma1 - beta - 1, que tiene que verificar; y,
     con una clave A MEDIDA (t1 = 0), una pista donde la parte baja es CERO, que ningun firmante de
     FIPS 204 pone: UseHint la baja, y con r1 - 1 verifica y con r1 + 1 no;
  3. la unicidad de la escritura de una firma: las pistas desordenadas o con un indice repetido, con
     relleno distinto de cero, con un corte que baja, que pasa de omega o que pasa de omega + k (sin
     leer fuera de la firma), un contexto distinto y un byte cambiado en cada tercio no verifican;
  4. la base64 estricta (RFC 4648, seccion 10, y los siete rechazos de `base64.rs`);
  5. la inclusion: las 98 sondas `inclusion/` de transparency-dev (6 positivas, 26 que rechaza el
     tipo, 66 el arbol, las cifras de `vectores_transparency.rs`), sus positivas con un bit cambiado,
     y todo camino de todo arbol de 1 a 33 hojas, y con el indice vecino, un hermano tocado, un hash
     de menos o uno de mas, no. El tamano no: un camino no lo ata, lo ata la nota firmada;
  6. el `{:?}` de Rust con el que salen los nombres en un rechazo (`medio.debug_rust`): once cadenas
     con lo que se escapa y lo que no, contra lo que imprime el Rust que compila el kit;
  7. cinco reglas que existen y que nada aislaba (§859): un corte que baja sobre un polinomio sin
     pistas -la firma de la pista sobre r0 = 0, con la misma h escrita de otra manera-; el hash del
     subarbol vacio y la marca de mas de 2^63 - 1 de `mensaje_cofirmado`, con notas firmadas a mano
     por otra clave a medida sobre el mensaje que la regla prohibe; el `+` en el nombre de una linea
     de firma, y una vkey con el key_id de otra clave. Cada uno con su control, que se acepta.

Salida: una linea `OK` o `ROJO` por comprobacion con nombre, y al final `medio: <ok> de <n>
comprobaciones`. Exit 0 si todas; 1 si no. Una excepcion que un bloque no espera (la de un mutante
de `mldsa44.py` o de `medio.py`) es un ROJO con su nombre, no una traza: la cuenta final sale igual.
"""
import hashlib
import json
import os
import sys
import time

AQUI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, AQUI)
import medio as MD  # noqa: E402
import mldsa44 as M  # noqa: E402

RAIZ = os.path.normpath(os.path.join(AQUI, "..", ".."))
NOTAS = os.path.join(RAIZ, "crates/zk-ssl-medio/tests/vectores/notas")
SONDAS = os.path.join(RAIZ, "crates/zk-ssl-medio/tests/vectores/transparency-dev/inclusion")
TESTIGO = "testigo.invalid/ajeno"
Q = M.Q

ok = n = 0


def caso(nombre, bien, detalle=""):
    global ok, n
    n += 1
    if bien:
        ok += 1
        print(f"OK   {nombre}")
    else:
        print(f"ROJO {nombre}{': ' + detalle if detalle else ''}")


def leer(f):
    # newline="": sin traducir los saltos, o el `\r` de neg-retorno-de-carro llega como `\n`
    with open(os.path.join(NOTAS, f), encoding="utf-8", newline="") as h:
        return h.read()


def semilla(desde):
    return bytes(desde + i for i in range(32))


def nota_aceptada(f, publicador):
    """`verificar_nota` sobre una nota que se tiene que aceptar. Si no, un ROJO con su causa y None."""
    try:
        return MD.verificar_nota(leer(f), publicador)
    except MD.ErrorDeNota as e:
        caso(f"{f} se acepta", False, f"{e.clase}: {e}")
        return None


# ─── el firmante de PRUEBA (FIPS 204, algoritmos 6 y 7): solo para este juez ─────────────────────

def _rej_bounded_poly(s):
    """Algoritmo 31 con eta = 2 y CoeffFromHalfByte (15): 2 - (b mod 5) para b < 15."""
    h = M._Lector(hashlib.shake_256, s, 272)
    a = []
    while len(a) < 256:
        z = h.leer(1)[0]
        for b in (z & 15, z >> 4):
            if b < 15 and len(a) < 256:
                a.append((2 - b % 5) % Q)
    return a


def _centrado(x):
    x %= Q
    return x - Q if x > (Q - 1) // 2 else x


def _prod(a, b):
    return [x * y % Q for x, y in zip(a, b)]


def _suma(a, b):
    return [(x + y) % Q for x, y in zip(a, b)]


def keygen(xi):
    """Algoritmo 6. Devuelve (pk, sk) con sk = (rho, K, tr, s1, s2, t0) sin codificar."""
    e = M.H(xi + bytes([M.K, M.L]), 128)
    rho, rho_p, kk = e[:32], e[32:96], e[96:]
    a = M.expand_a(rho)
    s1 = [_rej_bounded_poly(rho_p + r.to_bytes(2, "little")) for r in range(M.L)]
    s2 = [_rej_bounded_poly(rho_p + (r + M.L).to_bytes(2, "little")) for r in range(M.K)]
    s1n = [M.ntt(p) for p in s1]
    t1, t0 = [], []
    for r in range(M.K):
        acc = [0] * 256
        for s in range(M.L):
            acc = _suma(acc, _prod(a[r][s], s1n[s]))
        t = _suma(M.ntt_inversa(acc), s2[r])
        uno, cero = [], []
        for x in t:  # Power2Round (35): r0 en (-2^12, 2^12]
            r0 = x % (1 << M.D)
            if r0 > 1 << (M.D - 1):
                r0 -= 1 << M.D
            uno.append((x - r0) >> M.D)
            cero.append(r0)
        t1.append(uno)
        t0.append(cero)
    pk = rho + b"".join(sum(v << (10 * i) for i, v in enumerate(p)).to_bytes(320, "little") for p in t1)
    return pk, (rho, kk, M.H(pk, 64), s1, s2, t0)


def _high(x):
    return M.decompose(x)[0]


def _sig_encode(c_tilde, z, h):
    """Algoritmo 26, con BitPack (gamma1 - z en 18 bits) y HintBitPack (20)."""
    out = c_tilde
    for p in z:
        out += sum((M.GAMMA1 - v) << (18 * i) for i, v in enumerate(p)).to_bytes(M.BYTES_Z, "little")
    y, indice = bytearray(M.OMEGA + M.K), 0
    for i in range(M.K):
        for j in range(256):
            if h[i][j]:
                y[indice] = j
                indice += 1
        y[M.OMEGA + i] = indice
    return out + bytes(y)


def firmar(sk, mensaje, rnd=bytes(32), frontera=None):
    """Algoritmo 2 sobre el 7, contexto vacio; `rnd` cero es la variante determinista. Con `frontera`
    (un entero) NO es FIPS 204: el primer coeficiente de y se fija para que z[0][0] valga exactamente
    `frontera`, y se acepta |z|_inf = frontera; las demas condiciones de rechazo son las del estandar.
    Es la firma que la norma, y solo la norma, tiene que juzgar."""
    rho, kk, tr, s1, s2, t0 = sk
    a = M.expand_a(rho)
    s1n, s2n, t0n = [M.ntt(p) for p in s1], [M.ntt(p) for p in s2], [M.ntt(p) for p in t0]
    mu = M.H(tr + bytes([0, 0]) + mensaje, 64)
    rho2 = M.H(kk + rnd + mu, 64)
    kappa = 0
    while True:
        y = [[M.GAMMA1 - v for v in M._enteros(M.H(rho2 + (kappa + r).to_bytes(2, "little"), M.BYTES_Z), 18)]
             for r in range(M.L)]  # ExpandMask (34)
        kappa += M.L
        if frontera is not None:
            y[0][0] = frontera
        yn = [M.ntt([v % Q for v in p]) for p in y]
        w = []
        for r in range(M.K):
            acc = [0] * 256
            for s in range(M.L):
                acc = _suma(acc, _prod(a[r][s], yn[s]))
            w.append(M.ntt_inversa(acc))
        c_tilde = M.H(mu + M.w1_encode([[_high(x) for x in p] for p in w]), M.BYTES_C)
        c = M.sample_in_ball(c_tilde)
        if frontera is not None:
            # el coeficiente 0 de c * s1[0] en el anillo negaciclico, sin NTT: el filtro barato
            cs = c[0] * s1[0][0] - sum(c[i] * s1[0][256 - i] for i in range(1, 256) if c[i])
            if _centrado(cs) != 0:
                continue
        cn = M.ntt(c)
        z = [[_centrado(y[s][i] + x) for i, x in enumerate(M.ntt_inversa(_prod(cn, s1n[s])))] for s in range(M.L)]
        cs2 = [M.ntt_inversa(_prod(cn, s2n[r])) for r in range(M.K)]
        r0 = [[M.decompose(w[r][i] - cs2[r][i])[1] for i in range(256)] for r in range(M.K)]
        cota = M.GAMMA1 - M.BETA if frontera is None else frontera + 1
        if M.norma_infinita(z) >= cota or max(abs(v) for p in r0 for v in p) >= M.GAMMA2 - M.BETA:
            continue
        if frontera is not None and M.norma_infinita(z) != frontera:
            continue
        ct0 = [M.ntt_inversa(_prod(cn, t0n[r])) for r in range(M.K)]
        h = [[int(_high(w[r][i] - cs2[r][i] + ct0[r][i]) != _high(w[r][i] - cs2[r][i])) for i in range(256)]
             for r in range(M.K)]  # MakeHint(-ct0, w - cs2 + ct0) (39)
        if max(abs(_centrado(v)) for p in ct0 for v in p) >= M.GAMMA2 or sum(map(sum, h)) > M.OMEGA:
            continue
        return _sig_encode(c_tilde, z, h)


def linea(nombre, clave_sk, ident, checkpoint, marca):
    m = MD.mensaje_cofirmado(nombre, marca, checkpoint.origen, 0, checkpoint.tamano, checkpoint.raiz)
    firma = firmar(clave_sk, m)
    return f"{MD.PREFIJO_FIRMA}{nombre} {MD.base64_codificar(ident + marca.to_bytes(8, 'big') + firma)}\n"


# ─── 1 · las 33 notas del §632 ──────────────────────────────────────────────────────────────────

def bloque_notas():
    publicador = MD.ClaveDeNota.leer_vkey(leer("publicador.vkey").strip())
    testigo = MD.ClaveDeNota.leer_vkey(leer("testigo.vkey").strip())
    caso("las dos vkeys se leen y su key_id es el de su clave", publicador.vkey() == leer("publicador.vkey").strip()
         and testigo.vkey() == leer("testigo.vkey").strip())
    filas = {}
    for f in leer("MANIFIESTO.txt").splitlines():
        if f.strip() and not f.startswith("#"):
            c = f.split("#")[0].split()
            filas[c[0]] = c[1]
    ficheros = sorted(f for f in os.listdir(NOTAS) if f.endswith(".txt") and f != "MANIFIESTO.txt"
                      and not f.startswith("veredictos"))
    caso("cada nota tiene su fila en el MANIFIESTO y cada fila su nota", ficheros == sorted(filas))
    t0, cuenta = time.time(), {"ACEPTA": 0}
    for f in ficheros:
        try:
            MD.verificar_nota(leer(f), publicador)
            aqui = "ACEPTA"
        except MD.ErrorDeNota as e:
            aqui = e.clase
        cuenta[aqui] = cuenta.get(aqui, 0) + 1
        caso(f"nota {f}: {filas.get(f)}", aqui == filas.get(f), f"aqui {aqui}")
    print(f"     ({len(ficheros)} notas en {time.time() - t0:.1f} s: {cuenta['ACEPTA']} aceptadas, "
          f"{len(ficheros) - cuenta['ACEPTA']} rechazadas)")
    for k in (0, 1, 2, 5, 13):
        v = nota_aceptada(f"pos-{k:02}-anclas.txt", publicador)
        if v is None:
            continue
        hojas = [MD.hoja(hashlib.sha256(i.to_bytes(8, "big")).digest()) for i in range(k)]
        caso(f"pos-{k:02}: la raiz de torchwood (x/mod/sumdb/tlog) es el MTH de RFC 9162", v.checkpoint.raiz == MD.mth(hojas))
        try:
            marca = testigo.verificar_cofirma(v.checkpoint, v.ajenas[0])
            caso(f"pos-{k:02}: la cofirma del testigo verifica con su vkey", marca > 1_700_000_000)
        except MD.ErrorDeNota as e:
            caso(f"pos-{k:02}: la cofirma del testigo verifica con su vkey", False, str(e))
        try:
            publicador.verificar_cofirma(v.checkpoint, v.ajenas[0])
            caso(f"pos-{k:02}: la linea del testigo no es del publicador", False)
        except MD.ErrorDeNota as e:
            caso(f"pos-{k:02}: la linea del testigo no es del publicador", e.clase == "Clave")
    v = nota_aceptada("pos-05-anclas.txt", publicador)
    if v is None:
        return publicador, testigo
    tocada = MD.FirmaAjena(v.ajenas[0].nombre, v.ajenas[0].id, bytearray(v.ajenas[0].firma))
    tocada.firma[100] ^= 1
    tocada.firma = bytes(tocada.firma)
    for nombre, cp, ln in (("una cofirma tocada", v.checkpoint, tocada),
                           ("la cofirma sobre otro tamano", MD.Checkpoint(v.checkpoint.origen, v.checkpoint.tamano + 1,
                                                                         v.checkpoint.raiz), v.ajenas[0])):
        try:
            testigo.verificar_cofirma(cp, ln)
            caso(f"{nombre} no verifica", False)
        except MD.ErrorDeNota as e:
            caso(f"{nombre} no verifica", e.clase == "FirmaInvalida", e.clase)
    return publicador, testigo


# ─── 2 · el firmante de prueba contra Go, y las dos firmas de frontera ────────────────────────────

def bloque_firmante(publicador):
    t0 = time.time()
    pk_p, sk_p = keygen(semilla(0x00))
    pk_t, sk_t = keygen(semilla(0x20))
    caso("KeyGen de FIPS 204 rehace la vkey del publicador de torchwood",
         MD.ClaveDeNota(publicador.nombre, pk_p).vkey() == leer("publicador.vkey").strip())
    caso("KeyGen de FIPS 204 rehace la vkey del testigo de torchwood",
         MD.ClaveDeNota(TESTIGO, pk_t).vkey() == leer("testigo.vkey").strip())
    for f in ("pos-01-anclas.txt", "pos-05-anclas.txt", "pos-13-anclas.txt"):
        nota = leer(f)
        v = nota_aceptada(f, publicador)
        if v is None:
            continue
        mia = linea(publicador.nombre, sk_p, publicador.id, v.checkpoint, v.marca)
        caso(f"{f}: la linea del publicador, firmada aqui en determinista, sale byte a byte", mia in nota)
        marca_t = int.from_bytes(v.ajenas[0].firma[:8], "big")
        mia = linea(TESTIGO, sk_t, v.ajenas[0].id, v.checkpoint, marca_t)
        caso(f"{f}: la del testigo, tambien", nota.endswith(mia))
    mensaje = b"la frontera de la norma (FIPS 204, algoritmo 8, paso 13)"
    for frontera, debe in ((M.GAMMA1 - M.BETA, False), (M.GAMMA1 - M.BETA - 1, True)):
        sigma = firmar(sk_p, mensaje, frontera=frontera)
        _, z, h = M.sig_decode(sigma)
        caso(f"frontera |z|_inf = {frontera}: la firma es la que dice (pistas legibles, z[0][0] = {frontera})",
             h is not None and M.norma_infinita(z) == frontera and z[0][0] == frontera)
        caso(f"frontera |z|_inf = gamma1 - beta{' - 1' if debe else ''}: {'VERIFICA' if debe else 'NO verifica'}",
             M.verificar(pk_p, mensaje, sigma) is debe)
    print(f"     (firmante de prueba: {time.time() - t0:.1f} s)")


def bloque_pista_en_cero():
    """UseHint (algoritmo 40) sube r1 si r0 > 0 y lo baja si no: con r0 = 0, lo BAJA. Una firma de
    FIPS 204 no lleva nunca una pista donde la parte baja es cero -r0 = 0 es el centro de su intervalo,
    y MakeHint solo la pone si c*t0, de norma menor que gamma2, saca de el a w~-, asi que ninguna
    distingue `r0 > 0` de `r0 >= 0`. La verificacion acepta cualquier clave: con t1 = 0, w~ = A*z no
    depende de c, y se firma sin buscar un punto fijo del hash. Se elige z = (k, 0, ...) con A*z igual
    a m * 2 gamma2 EXACTO en un coeficiente, y la firma lleva ahi su unica pista."""
    rho = M.H(b"juez_medio: una clave a medida, t1 = 0", 32)
    pk = rho + bytes(M.K * M.BYTES_T1)
    a = M.expand_a(rho)
    columna = [M.ntt_inversa(a[r][0]) for r in range(M.K)]  # A[r][0], en coeficientes
    # El primer (i, m) con m de 2 a 42 -sin la vuelta de r1 por 0 y 43- y un k que quepa en la norma.
    i, m, k = next((i, m, _centrado(m * 2 * M.GAMMA2 * pow(columna[0][i], -1, Q))) for i in range(256)
                   if columna[0][i] for m in range(2, M.M_W1 - 1)
                   if 0 < abs(_centrado(m * 2 * M.GAMMA2 * pow(columna[0][i], -1, Q))) < M.GAMMA1 - M.BETA)
    z = [[0] * 256 for _ in range(M.L)]
    z[0][0] = k
    w = [[k * x % Q for x in p] for p in columna]  # A*z: k veces la primera columna
    w1 = [[M.decompose(x)[0] for x in p] for p in w]
    h = [[0] * 256 for _ in range(M.K)]
    h[0][i] = 1
    mensaje = b"una pista sobre r0 = 0 (FIPS 204, algoritmo 40)"
    mu = M.H(M.H(pk, 64) + bytes([0, 0]) + mensaje, 64)

    def con_w1(valor):
        w1[0][i] = valor % M.M_W1
        return _sig_encode(M.H(mu + M.w1_encode(w1), M.BYTES_C), z, h)
    baja, sube = con_w1(m - 1), con_w1(m + 1)
    _, zz, hh = M.sig_decode(baja)
    caso(f"pista sobre r0 = 0: con t1 = 0 y z[0][0] = {k}, w~[0][{i}] = {m} * 2 gamma2 exacto, y la firma lleva "
         f"ahi su unica pista", M.decompose(w[0][i]) == (m, 0) and zz == z and hh is not None
         and sum(map(sum, hh)) == 1 and hh[0][i] == 1)
    caso("pista sobre r0 = 0: UseHint baja r1, y con w1 = r1 - 1 VERIFICA", M.verificar(pk, mensaje, baja) is True)
    caso("pista sobre r0 = 0: con w1 = r1 + 1 NO verifica", M.verificar(pk, mensaje, sube) is False)
    return pk, mensaje, baja


# ─── 3 · una firma tiene una sola escritura ─────────────────────────────────────────────────────

def bloque_unicidad(publicador):
    v = nota_aceptada("pos-05-anclas.txt", publicador)
    if v is None:
        return
    m = MD.mensaje_cofirmado(publicador.nombre, v.marca, v.checkpoint.origen, 0, v.checkpoint.tamano, v.checkpoint.raiz)
    nota = leer("pos-05-anclas.txt")
    b = MD.base64_decodificar(nota.split("\n\n", 1)[1].split("\n")[0].split(" ", 2)[2])
    sigma = b[12:]
    caso("la firma de pos-05 verifica sin tocar", M.verificar(publicador.bytes, m, sigma))
    caso("un contexto que no es el vacio no la verifica", not M.verificar(publicador.bytes, m, sigma, ctx=b"x"))
    caso("un contexto de 256 bytes se rechaza", not M.verificar(publicador.bytes, m, sigma, ctx=bytes(256)))
    pistas = M.BYTES_C + M.L * M.BYTES_Z
    y = bytearray(sigma[pistas:])
    cortes = list(y[M.OMEGA:])
    i = next(i for i in range(M.K) if cortes[i] - (cortes[i - 1] if i else 0) >= 2)
    a = cortes[i - 1] if i else 0

    def con(cambio):
        s = bytearray(sigma)
        cambio(s)
        return bytes(s)

    def desorden(s):
        s[pistas + a], s[pistas + a + 1] = s[pistas + a + 1], s[pistas + a]
    caso("las pistas desordenadas (el mismo conjunto) no verifican", not M.verificar(publicador.bytes, m, con(desorden)))

    def repetida(s):
        # El mismo conjunto, con el primer indice del polinomio i escrito dos veces: su corte y los
        # siguientes suben uno y el relleno pierde un cero. Quien pida que suban sin pedir que suban
        # ESTRICTAMENTE lee las mismas pistas, y la firma verificaria con otra escritura.
        ind = list(y[:cortes[-1]])
        ind.insert(a, ind[a])
        s[pistas:pistas + M.OMEGA] = bytes(ind + [0] * (M.OMEGA - len(ind)))
        s[pistas + M.OMEGA:] = bytes(c + 1 if j >= i else c for j, c in enumerate(cortes))
    caso("una pista repetida (el mismo conjunto, con un indice escrito dos veces) no verifica", cortes[-1] < M.OMEGA
         and not M.verificar(publicador.bytes, m, con(repetida)))

    def relleno(s):
        s[pistas + cortes[-1]] = 1
    caso("un relleno de las pistas distinto de cero no verifica", cortes[-1] < M.OMEGA
         and not M.verificar(publicador.bytes, m, con(relleno)))

    def corte_alto(s):
        s[pistas + M.OMEGA + M.K - 1] = M.OMEGA + 1
    caso("un corte que pasa de omega no verifica", not M.verificar(publicador.bytes, m, con(corte_alto)))

    def cortes_fuera(s):
        # Las pistas 0, 1, ..., 79, que suben, y los cortes 200, 201, 202 y 255: quien no mira la cota
        # lee los cortes como indices y despues fuera de la firma. Es un rechazo, no una excepcion.
        s[pistas:pistas + M.OMEGA] = bytes(range(M.OMEGA))
        s[pistas + M.OMEGA:] = bytes([200, 201, 202, 255])
    try:
        bien, detalle = not M.verificar(publicador.bytes, m, con(cortes_fuera)), ""
    except Exception as e:  # noqa: BLE001 - lo que se juzga es justamente que no lance
        bien, detalle = False, f"{type(e).__name__}: {e}"
    caso("unos cortes por encima de omega + k (200, 201, 202, 255) no verifican, y sin leer fuera de la firma",
         bien, detalle)

    def corte_baja(s):
        s[pistas + M.OMEGA + M.K - 1] = cortes[M.K - 2] - 1
    caso("un corte que baja no verifica", cortes[M.K - 2] > 0 and not M.verificar(publicador.bytes, m, con(corte_baja)))
    for nombre, pos in (("c~", 5), ("z", M.BYTES_C + 700), ("las pistas", pistas + 1)):
        def voltea(s, pos=pos):
            s[pos] ^= 0x10
        caso(f"un bit cambiado en {nombre} no verifica", not M.verificar(publicador.bytes, m, con(voltea)))
    caso("una firma de 2419 bytes no verifica", not M.verificar(publicador.bytes, m, sigma[:-1]))


# ─── 4 · la base64 estricta ─────────────────────────────────────────────────────────────────────

def bloque_base64():
    rfc = [(b"", ""), (b"f", "Zg=="), (b"fo", "Zm8="), (b"foo", "Zm9v"), (b"foob", "Zm9vYg=="),
           (b"fooba", "Zm9vYmE="), (b"foobar", "Zm9vYmFy")]
    caso("base64: los siete vectores de RFC 4648, seccion 10, en los dos sentidos",
         all(MD.base64_codificar(b) == s and MD.base64_decodificar(s) == b for b, s in rfc))
    malos = []
    for s in ("Zg", "Zh==", "Zm9v!A==", "Zg==Zg==", "Z===", "Zm9v\nYmFy\n", "Zm9v YmFy"):
        try:
            MD.base64_decodificar(s)
            malos.append(s)
        except MD.Base64Malo:
            pass
    caso("base64: los siete rechazos de base64.rs (sin relleno, bits sobrantes, alfabeto, relleno dentro, "
         "relleno de mas y dos blancos)", not malos, repr(malos))


# ─── 5 · la inclusion de RFC 9162 ───────────────────────────────────────────────────────────────

def bloque_inclusion():
    def h(s):
        try:
            b = MD.base64_decodificar(s)
        except MD.Base64Malo:
            return None
        return b if len(b) == 32 else None

    cuenta = {"aceptadas": 0, "tipo": 0, "arbol": 0}
    malas, volteadas = [], 0
    ficheros = sorted(os.path.join(d, f) for d, _, fs in os.walk(SONDAS) for f in fs)
    for f in ficheros:
        with open(f, encoding="utf-8") as fh:
            s = json.load(fh)
        hoja_, raiz = h(s["leafHash"]), h(s["root"])
        prueba = [h(x) for x in (s["proof"] or [])]
        if hoja_ is None or raiz is None or None in prueba:
            veredicto = None
        else:
            try:
                MD.verificar_inclusion(hoja_, s["leafIdx"], s["treeSize"], prueba, raiz)
                veredicto = True
            except MD.ErrorDelMedio:
                veredicto = False
        if not s["wantErr"] and veredicto:
            cuenta["aceptadas"] += 1
            for hh, rr in ((bytes([hoja_[0] ^ 1]) + hoja_[1:], raiz), (hoja_, bytes([raiz[0] ^ 1]) + raiz[1:])):
                try:
                    MD.verificar_inclusion(hh, s["leafIdx"], s["treeSize"], prueba, rr)
                    malas.append(os.path.relpath(f, SONDAS) + " con un bit cambiado")
                except MD.ErrorDelMedio:
                    volteadas += 1
        elif s["wantErr"] and veredicto is None:
            cuenta["tipo"] += 1
        elif s["wantErr"] and veredicto is False:
            cuenta["arbol"] += 1
        else:
            malas.append(os.path.relpath(f, SONDAS))
    caso(f"inclusion: las {len(ficheros)} sondas de transparency-dev dan 6 positivas, 26 que rechaza el tipo y 66 el "
         f"arbol", (len(ficheros), cuenta["aceptadas"], cuenta["tipo"], cuenta["arbol"]) == (98, 6, 26, 66) and not malas,
         f"{cuenta} {malas}")
    caso("inclusion: sus 6 positivas, con un bit cambiado en la hoja o en la raiz, no verifican (12)", volteadas == 12)
    fallos, rechazos = 0, 0
    for t in range(1, 34):
        hojas = [MD.hoja(hashlib.sha256(b"ancla %d" % i).digest()) for i in range(t)]
        raiz = MD.mth(hojas)
        for i in range(t):
            p = MD.camino(hojas, i)
            try:
                MD.verificar_inclusion(hojas[i], i, t, p, raiz)
            except MD.ErrorDelMedio:
                fallos += 1
            # El vecino de indice, un hermano tocado, un hash de mas y uno de menos. El TAMANO no: un
            # camino no lo ata (el de la hoja 0 de 3 hojas es el de 4), lo ata la nota firmada.
            malos = []
            if i ^ 1 < t:
                malos.append((i ^ 1, p))
            if p:
                malos += [(i, [bytes([p[0][0] ^ 1]) + p[0][1:]] + p[1:]), (i, p[:-1])]
            malos.append((i, p + [raiz]))
            for otro_i, otro_p in malos:
                try:
                    MD.verificar_inclusion(hojas[i], otro_i, t, otro_p, raiz)
                    fallos += 1
                except MD.ErrorDelMedio:
                    rechazos += 1
    caso(f"inclusion: todo camino de todo arbol de 1 a 33 hojas verifica (561), y con el indice vecino, un hermano "
         f"tocado, un hash de menos o uno de mas no ({rechazos})", fallos == 0, f"{fallos} fallos")
    try:
        MD.verificar_inclusion(bytes(32), 3, 3, [], bytes(32))
        caso("inclusion: una posicion fuera del arbol se rechaza con su nombre", False)
    except MD.ErrorDelMedio as e:
        caso("inclusion: una posicion fuera del arbol se rechaza con su nombre", str(e) == "3 no cae en un árbol de 3 hojas")


# ─── 6 · el `{:?}` de Rust de los nombres ───────────────────────────────────────────────────────

# Cada cadena y lo que imprime `println!("{:?}", cadena)` con el rustc 1.94.1 que compila el kit (su
# Unicode, 17.0), medido en el sello que escribio este bloque. Nada de aqui cambia de categoria entre
# el Unicode 14.0 de Python 3.11 y el 17.0 de Rust (U+0378 y U+10FFFF siguen sin asignar en los dos):
# lo asignado despues lo escapa `debug_rust` y Rust no (README.md).
DEBUG_DE_RUST = [
    ('zkssl/v1/\u200bx', '"zkssl/v1/\\u{200b}x"'),
    ('a\'b"c\\d', '"a\'b\\"c\\\\d"'),
    ('\x00\t\r\n\x01\x1f\x7f\x9f', '"\\0\\t\\r\\n\\u{1}\\u{1f}\\u{7f}\\u{9f}"'),
    ('\xa0\u2028\u2029\u3000', '"\\u{a0}\\u{2028}\\u{2029}\\u{3000}"'),
    ('\xad\ufeff\U000e0041', '"\\u{ad}\\u{feff}\\u{e0041}"'),
    ('\ue000\U000f0000', '"\\u{e000}\\u{f0000}"'),
    ('\u0378\U0010ffff', '"\\u{378}\\u{10ffff}"'),
    ('e\u0301\u20dd', '"e\\u{301}\\u{20dd}"'),
    ('\u09be\u0903\uff9e', '"\\u{9be}\u0903\\u{ff9e}"'),
    ('\xf1\u20ac\U0001f600\U0001f3fb ', '"\xf1\u20ac\U0001f600\U0001f3fb "'),
    ('\u0cc0', '"\\u{cc0}"'),
]


def bloque_debug():
    malas = [(s, MD.debug_rust(s), r) for s, r in DEBUG_DE_RUST if MD.debug_rust(s) != r]
    caso(f"debug_rust: las {len(DEBUG_DE_RUST)} cadenas salen como el `{{:?}}` de Rust (U+200B, el acento que se "
         f"combina, Zs, Zl, Zp, Cf, Co, Cn, Other_Grapheme_Extend, la comilla simple que no)", not malas, repr(malas))


# ─── 7 · cinco reglas que ninguna nota ni ninguna firma de FIPS 204 aislaba (§859) ──────────────

def _mensaje_sin_reglas(nombre, marca, origen, inicio, fin, h):
    """El `cosigned_message` de `tlog-cosignature`, byte a byte y SIN ninguna de sus reglas: lo que
    `medio.mensaje_cofirmado` se niega a formar. Solo para firmar a mano lo que una regla prohibe."""
    n, o = nombre.encode("utf-8"), origen.encode("utf-8")
    return (MD.ETIQUETA + bytes([len(n)]) + n + marca.to_bytes(8, "big") + bytes([len(o)]) + o
            + inicio.to_bytes(8, "big") + fin.to_bytes(8, "big") + h)


def _firma_a_medida(pk, mensaje):
    """Una firma de una clave a medida (t1 = 0) sobre cualquier mensaje, sin buscar nada: con z = 0 y
    sin pistas, w~ = A*z - c*t1*2^d = 0, w1 = 0 y c~ = H(mu || w1Encode(0)). El algoritmo 8 la acepta."""
    mu = M.H(M.H(pk, 64) + bytes([0, 0]) + mensaje, 64)
    cero = [[0] * 256 for _ in range(M.K)]
    return _sig_encode(M.H(mu + M.w1_encode(cero), M.BYTES_C), [[0] * 256 for _ in range(M.L)], cero)


def _nota_a_medida(clave, tamano, raiz, marca):
    """Una nota de una sola linea, la del publicador `clave`, firmada a mano sobre el mensaje sin reglas."""
    firma = _firma_a_medida(clave.bytes, _mensaje_sin_reglas(clave.nombre, marca, clave.nombre, 0, tamano, raiz))
    return (f"{clave.nombre}\n{tamano}\n{MD.base64_codificar(raiz)}\n\n{MD.PREFIJO_FIRMA}{clave.nombre} "
            f"{MD.base64_codificar(clave.id + marca.to_bytes(8, 'big') + firma)}\n")


def _veredicto(nota, clave):
    try:
        MD.verificar_nota(nota, clave)
        return "ACEPTA"
    except MD.ErrorDeNota as e:
        return f"{e.clase}: {e}"


def bloque_cinco(pista, publicador):
    """Las cinco reglas que vivian sin vigilar desde el §810 (README.md, «Los mutantes del propio
    verificador»): existen en `mldsa44.py` y `medio.py`, y en el kit, pero quitarlas no cambiaba ninguna
    salida del juez. Cada caso aisla una: el control de cada uno es la misma entrada sin lo que la
    regla prohibe, y se tiene que aceptar."""
    # HintBitUnpack, el corte que baja: sobre un polinomio sin pistas, la misma h con otra escritura.
    if pista is None:
        caso("un corte que baja sobre un polinomio sin pistas no verifica", False, "sin la firma de la pista")
    else:
        pk, mensaje, baja = pista
        cortes = M.BYTES_C + M.L * M.BYTES_Z + M.OMEGA
        otra = bytearray(baja)
        otra[cortes + 1] = 0
        antes, despues = M.verificar(pk, mensaje, baja), M.verificar(pk, mensaje, bytes(otra))
        caso("un corte que baja sobre un polinomio sin pistas no verifica: la firma de la pista sobre r0 = 0 con "
             "los cortes [1, 0, 1, 1] en vez de [1, 1, 1, 1], la misma h escrita de otra manera",
             list(baja[cortes:]) == [1, 1, 1, 1] and antes is True and despues is False,
             f"cortes {list(baja[cortes:])}; con [1, 1, 1, 1] {antes}; con [1, 0, 1, 1] {despues}")
    # mensaje_cofirmado: el subarbol vacio y la marca de 2^63. Firmados a mano con una clave a medida.
    clave = MD.ClaveDeNota("juez.invalid/a-medida",
                           M.H(b"juez_medio: otra clave a medida, t1 = 0", 32) + bytes(M.K * M.BYTES_T1))
    vacia = _veredicto(_nota_a_medida(clave, 0, MD.sha256(b""), 0), clave)
    otra = _veredicto(_nota_a_medida(clave, 0, MD.sha256(b"no es el arbol vacio"), 0), clave)
    caso("tamano 0 y una raiz que no es SHA-256 de nada, firmado a mano: FirmaInvalida (con la raiz del arbol "
         "vacio, ACEPTA)", vacia == "ACEPTA" and otra.startswith("FirmaInvalida:"), f"vacia {vacia}; otra {otra}")
    raiz = MD.sha256(b"un arbol de una hoja")
    tope = _veredicto(_nota_a_medida(clave, 1, raiz, MD.I64_MAX), clave)
    pasa = _veredicto(_nota_a_medida(clave, 1, raiz, MD.I64_MAX + 1), clave)
    caso("una marca de 2^63, firmada a mano: FirmaInvalida (con 2^63 - 1, ACEPTA)",
         tope == "ACEPTA" and pasa.startswith("FirmaInvalida:"), f"2^63 - 1 {tope}; 2^63 {pasa}")
    # nombre_valido, el `+`: una linea ajena con uno en el nombre. Y leer_vkey, el key_id de otra clave.
    if publicador is None:
        caso("una linea de firma con un + en el nombre es Malformada", False, "sin la clave del publicador")
    else:
        nota, b = leer("pos-05-anclas.txt"), MD.base64_codificar(bytes(5))
        con_guion = _veredicto(nota + f"{MD.PREFIJO_FIRMA}otro-nombre {b}\n", publicador)
        con_mas = _veredicto(nota + f"{MD.PREFIJO_FIRMA}otro+nombre {b}\n", publicador)
        caso("pos-05 con una linea ajena de nombre otro+nombre es Malformada (con otro-nombre, ACEPTA)",
             con_guion == "ACEPTA" and con_mas == "Malformada: nota mal formada: nombre de clave inválido",
             f"otro-nombre {con_guion}; otro+nombre {con_mas}")
    nombre, id_hex, b64 = leer("publicador.vkey").strip().split("+", 2)
    otro_id = (int(id_hex, 16) ^ 1).to_bytes(4, "big").hex()

    def _leer(texto):
        try:
            return "ACEPTA", MD.ClaveDeNota.leer_vkey(texto)
        except MD.ErrorDeNota as e:
            return f"{e.clase}: {e}", None

    tal_cual, k = _leer(f"{nombre}+{id_hex}+{b64}")
    tal_cual_bien = tal_cual == "ACEPTA" and k.id.hex() == id_hex
    r, _ = _leer(f"{nombre}+{otro_id}+{b64}")
    caso(f"la vkey del publicador con el key_id {otro_id} (el ultimo bit cambiado) es Clave (tal cual, con "
         f"{id_hex}, ACEPTA)",
         tal_cual_bien and r == "Clave: clave: el key_id no es el de esa clave", f"{id_hex} {tal_cual}; {otro_id} {r}")


def a_salvo(nombre, bloque, *args):
    """Corre un bloque. Lo que lance sin esperarlo es un ROJO con su nombre, y el juez sigue."""
    try:
        return bloque(*args)
    except Exception as e:  # noqa: BLE001
        caso(f"el bloque «{nombre}» llega a su final", False, f"{type(e).__name__}: {e}")
        return None


def main():
    claves = a_salvo("1 · las notas", bloque_notas)
    if claves is not None:
        a_salvo("2 · el firmante", bloque_firmante, claves[0])
    pista = a_salvo("2 · la pista sobre r0 = 0", bloque_pista_en_cero)
    if claves is not None:
        a_salvo("3 · la unicidad", bloque_unicidad, claves[0])
    a_salvo("4 · la base64", bloque_base64)
    a_salvo("5 · la inclusion", bloque_inclusion)
    a_salvo("6 · el {:?} de Rust", bloque_debug)
    a_salvo("7 · las cinco reglas", bloque_cinco, pista, claves[0] if claves is not None else None)
    print(f"medio: {ok} de {n} comprobaciones (ML-DSA-44 de FIPS 204, la nota checkpoint y la inclusion de RFC 9162)")
    return 0 if ok == n else 1


if __name__ == "__main__":
    raise SystemExit(main())
