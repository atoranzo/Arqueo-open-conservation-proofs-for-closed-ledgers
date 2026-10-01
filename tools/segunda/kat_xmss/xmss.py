#!/usr/bin/env python3
"""Verificacion XMSS^MT de RFC 8391, en Python y sin dependencias, para el conjunto que la casa firma:
XMSSMT-SHA2_40/8_256 (OID 0x00000005; n = 32, w = 16, h = 40, d = 8). Solo verificar: ni claves ni
firmas se producen aqui.

De donde sale: del texto de RFC 8391 (secciones 2 a 4: toByte, ADRS, F/H/H_msg/PRF con SHA-256,
WOTS+, L-tree, RAND_HASH, XMSS_rootFromSig y XMSSMT_verify) y de ningun crate. Lee la clave publica
como el RFC manda, OID || root || SEED, con el OID en el registro MULTIARBOL; por eso no necesita el
apano de lectura que `xmss 0.1.0-pre.0` obliga a su consumidor a aplicar (Arqueo, AUDITORIA S240).

El formato de una firma de XMSSMT-SHA2_40/8_256: 18.469 bytes, idx (5) || r (32) || 8 x (WOTS+
67x32 || auth 5x32). Lo fijan los vectores de este directorio, producidos por el crate de RustCrypto.
Nacio en Arqueo (https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers),
donde verifica las cabezas firmadas de su catalogo de vectores.
"""
import hashlib

N, W, LG_W = 32, 16, 4
LEN_1, LEN_2 = 64, 3
LEN = LEN_1 + LEN_2
PARAMETROS = {  # OID -> (h, d)
    0x00000005: (40, 8),  # XMSSMT-SHA2_40/8_256, el de la casa
    0x00000001: (20, 2), 0x00000002: (20, 4), 0x00000003: (40, 2), 0x00000004: (40, 4),
    0x00000006: (60, 3), 0x00000007: (60, 6), 0x00000008: (60, 12),
}


def to_byte(x, y):
    return x.to_bytes(y, "big")


def _sha(prefijo, key, m):
    return hashlib.sha256(to_byte(prefijo, N) + key + m).digest()


def F(key, m):
    return _sha(0, key, m)


def H(key, m):
    return _sha(1, key, m)


def H_msg(key, m):
    return _sha(2, key, m)


def PRF(key, m):
    return _sha(3, key, m)


class Adrs:
    """Los ocho words big-endian de la direccion (RFC 8391, 2.5)."""
    OTS, LTREE, HASHTREE = 0, 1, 2

    def __init__(self):
        self.w = [0] * 8

    def layer(self, v): self.w[0] = v
    def tree(self, v): self.w[1], self.w[2] = (v >> 32) & 0xFFFFFFFF, v & 0xFFFFFFFF
    def tipo(self, v): self.w[3] = v; self.w[4] = self.w[5] = self.w[6] = self.w[7] = 0
    def ots(self, v): self.w[4] = v
    def chain(self, v): self.w[5] = v
    def hash(self, v): self.w[6] = v
    def ltree(self, v): self.w[4] = v
    def altura(self, v): self.w[5] = v
    def indice(self, v): self.w[6] = v
    def indice_actual(self): return self.w[6]
    def key_and_mask(self, v): self.w[7] = v
    def bytes(self): return b"".join(x.to_bytes(4, "big") for x in self.w)


def base_w(x, out_len):
    digitos = []
    for b in x:
        digitos.append(b >> 4)
        digitos.append(b & 0xF)
        if len(digitos) >= out_len:
            break
    return digitos[:out_len]


def chain(x, i, s, seed, adrs):
    for j in range(i, i + s):
        adrs.hash(j)
        adrs.key_and_mask(0)
        key = PRF(seed, adrs.bytes())
        adrs.key_and_mask(1)
        bm = PRF(seed, adrs.bytes())
        x = F(key, bytes(a ^ b for a, b in zip(x, bm)))
    return x


def wots_pk_from_sig(m, sig, seed, adrs):
    msg = base_w(m, LEN_1)
    csum = sum(W - 1 - v for v in msg)
    csum <<= 8 - ((LEN_2 * LG_W) % 8)
    msg += base_w(to_byte(csum, (LEN_2 * LG_W + 7) // 8), LEN_2)
    pk = []
    for i in range(LEN):
        adrs.chain(i)
        pk.append(chain(sig[i * N:(i + 1) * N], msg[i], W - 1 - msg[i], seed, adrs))
    return pk


def rand_hash(izq, der, seed, adrs):
    adrs.key_and_mask(0)
    key = PRF(seed, adrs.bytes())
    adrs.key_and_mask(1)
    bm0 = PRF(seed, adrs.bytes())
    adrs.key_and_mask(2)
    bm1 = PRF(seed, adrs.bytes())
    return H(key, bytes(a ^ b for a, b in zip(izq, bm0)) + bytes(a ^ b for a, b in zip(der, bm1)))


def ltree(pk, seed, adrs):
    pk = list(pk)
    largo = LEN
    adrs.altura(0)
    while largo > 1:
        for i in range(largo // 2):
            adrs.indice(i)
            pk[i] = rand_hash(pk[2 * i], pk[2 * i + 1], seed, adrs)
        if largo % 2 == 1:
            pk[largo // 2] = pk[largo - 1]
        largo = (largo + 1) // 2
        adrs.altura(adrs.w[5] + 1)
    return pk[0]


def xmss_root_from_sig(idx_sig, sig_ots, auth, m, seed, adrs, h_prima):
    adrs.tipo(Adrs.OTS)
    adrs.ots(idx_sig)
    pk = wots_pk_from_sig(m, sig_ots, seed, adrs)
    adrs.tipo(Adrs.LTREE)
    adrs.ltree(idx_sig)
    nodo = ltree(pk, seed, adrs)
    adrs.tipo(Adrs.HASHTREE)
    adrs.indice(idx_sig)
    for k in range(h_prima):
        adrs.altura(k)
        if (idx_sig >> k) & 1 == 0:
            adrs.indice(adrs.indice_actual() // 2)
            nodo = rand_hash(nodo, auth[k * N:(k + 1) * N], seed, adrs)
        else:
            adrs.indice((adrs.indice_actual() - 1) // 2)
            nodo = rand_hash(auth[k * N:(k + 1) * N], nodo, seed, adrs)
    return nodo


def parsear_clave(pk):
    """OID (4) || root (n) || SEED (n)."""
    if len(pk) != 4 + 2 * N:
        raise ValueError(f"clave de {len(pk)} bytes; se esperan {4 + 2 * N}")
    oid = int.from_bytes(pk[:4], "big")
    if oid not in PARAMETROS:
        raise ValueError(f"OID {oid:#010x} no es un XMSS^MT de RFC 8391")
    return oid, pk[4:4 + N], pk[4 + N:]


def largo_firma(oid):
    h, d = PARAMETROS[oid]
    return (h + 7) // 8 + N + d * (LEN * N + (h // d) * N)


def indice_embebido(oid, firma):
    return int.from_bytes(firma[:(PARAMETROS[oid][0] + 7) // 8], "big")


def verificar(pk, mensaje, firma):
    """XMSSMT_verify (RFC 8391, 4.2.5). Devuelve True si la firma de `mensaje` cuadra con `pk`."""
    oid, root, seed = parsear_clave(pk)
    h, d = PARAMETROS[oid]
    h_prima = h // d
    if len(firma) != largo_firma(oid):
        return False
    idx_bytes = (h + 7) // 8
    idx_sig = int.from_bytes(firma[:idx_bytes], "big")
    r = firma[idx_bytes:idx_bytes + N]
    reducidas = firma[idx_bytes + N:]
    paso = LEN * N + h_prima * N
    m_prima = H_msg(r + root + to_byte(idx_sig, N), mensaje)
    idx_leaf = idx_sig & ((1 << h_prima) - 1)
    idx_tree = idx_sig >> h_prima
    nodo = m_prima
    for j in range(d):
        if j > 0:
            idx_leaf = idx_tree & ((1 << h_prima) - 1)
            idx_tree >>= h_prima
        adrs = Adrs()
        adrs.layer(j)
        adrs.tree(idx_tree)
        capa = reducidas[j * paso:(j + 1) * paso]
        nodo = xmss_root_from_sig(idx_leaf, capa[:LEN * N], capa[LEN * N:], nodo, seed, adrs, h_prima)
    return nodo == root
