#!/usr/bin/env python3
"""Segunda implementacion del NUCLEO congelado de Arqueo (spec/NUCLEO.md, seccion 6), en Python y sin
dependencias: solo la biblioteca estandar.

Que es. BACKLOG 85: <<una spec que solo implementa un codigo no es una spec, es documentacion>>.
Este fichero es otro codigo. Reproduce, byte a byte, cada `fn` NUCLEO que `spec/vectors/nucleo/`
fija con un KAT, y nada mas: ni la firma XMSS, ni las pruebas STARK, ni los sobres de PAQUETE.md.
Lo juzga `juez_nucleo.py`, que es el unico llamador.

De donde sale cada pieza, y de donde NO:
  - la permutacion Rescue-Prime `Rp64_256`: de la fuente de `winter-crypto 0.13.1` (ronda XLIX,
    siete rondas, alfa 7, su inversa, la matriz MDS y las constantes ARK1/ARK2 copiadas de las
    tablas `pub const` de `rp64_256/mod.rs`). Es la primitiva, no la casa; NUCLEO.md la nombra asi.
  - el campo de Goldilocks: p = 2^64 - 2^32 + 1, aritmetica sobre enteros de Python.
  - BLAKE3: escrito aqui desde la especificacion publica, y validado en `autotest()` contra 19
    salidas del crate `blake3 1.8.5` del Cargo.lock sobre el patron de los vectores oficiales; la
    permutacion, contra el vector que `winter-crypto` atribuye a la implementacion Sage.
  - las composiciones, los dominios, los preambulos y la serializacion: de NUCLEO.md seccion 6,
    y de ningun fichero `.rs` de la casa. Donde la seccion 6 no basta, `juez_nucleo.py` lo dice.

Lo que NO afirma: que la seccion 6 este completa (ver las NOTAS del juez), ni nada sobre lo que
un KAT no fija (la firma, la clave, los sobres).
"""

# ─── el campo de Goldilocks ───────────────────────────────────────────────────────────────────
P = 2**64 - 2**32 + 1

# ─── Rescue-Prime Rp64_256 (winter-crypto 0.13.1) ────────────────────────────────────────────
STATE_WIDTH = 12
NUM_ROUNDS = 7
ALPHA = 7
INV_ALPHA = 10540996611094048183
DIGEST = slice(4, 8)

MDS = [
    [7, 23, 8, 26, 13, 10, 9, 7, 6, 22, 21, 8],
    [8, 7, 23, 8, 26, 13, 10, 9, 7, 6, 22, 21],
    [21, 8, 7, 23, 8, 26, 13, 10, 9, 7, 6, 22],
    [22, 21, 8, 7, 23, 8, 26, 13, 10, 9, 7, 6],
    [6, 22, 21, 8, 7, 23, 8, 26, 13, 10, 9, 7],
    [7, 6, 22, 21, 8, 7, 23, 8, 26, 13, 10, 9],
    [9, 7, 6, 22, 21, 8, 7, 23, 8, 26, 13, 10],
    [10, 9, 7, 6, 22, 21, 8, 7, 23, 8, 26, 13],
    [13, 10, 9, 7, 6, 22, 21, 8, 7, 23, 8, 26],
    [26, 13, 10, 9, 7, 6, 22, 21, 8, 7, 23, 8],
    [8, 26, 13, 10, 9, 7, 6, 22, 21, 8, 7, 23],
    [23, 8, 26, 13, 10, 9, 7, 6, 22, 21, 8, 7],
]
ARK1 = [
    [13917550007135091859, 16002276252647722320, 4729924423368391595, 10059693067827680263, 9804807372516189948, 15666751576116384237, 10150587679474953119, 13627942357577414247, 2323786301545403792, 615170742765998613, 8870655212817778103, 10534167191270683080],
    [14572151513649018290, 9445470642301863087, 6565801926598404534, 12667566692985038975, 7193782419267459720, 11874811971940314298, 17906868010477466257, 1237247437760523561, 6829882458376718831, 2140011966759485221, 1624379354686052121, 50954653459374206],
    [16288075653722020941, 13294924199301620952, 13370596140726871456, 611533288599636281, 12865221627554828747, 12269498015480242943, 8230863118714645896, 13466591048726906480, 10176988631229240256, 14951460136371189405, 5882405912332577353, 18125144098115032453],
    [6076976409066920174, 7466617867456719866, 5509452692963105675, 14692460717212261752, 12980373618703329746, 1361187191725412610, 6093955025012408881, 5110883082899748359, 8578179704817414083, 9311749071195681469, 16965242536774914613, 5747454353875601040],
    [13684212076160345083, 19445754899749561, 16618768069125744845, 278225951958825090, 4997246680116830377, 782614868534172852, 16423767594935000044, 9990984633405879434, 16757120847103156641, 2103861168279461168, 16018697163142305052, 6479823382130993799],
    [13957683526597936825, 9702819874074407511, 18357323897135139931, 3029452444431245019, 1809322684009991117, 12459356450895788575, 11985094908667810946, 12868806590346066108, 7872185587893926881, 10694372443883124306, 8644995046789277522, 1422920069067375692],
    [17619517835351328008, 6173683530634627901, 15061027706054897896, 4503753322633415655, 11538516425871008333, 12777459872202073891, 17842814708228807409, 13441695826912633916, 5950710620243434509, 17040450522225825296, 8787650312632423701, 7431110942091427450],
]
ARK2 = [
    [7989257206380839449, 8639509123020237648, 6488561830509603695, 5519169995467998761, 2972173318556248829, 14899875358187389787, 14160104549881494022, 5969738169680657501, 5116050734813646528, 12120002089437618419, 17404470791907152876, 2718166276419445724],
    [2485377440770793394, 14358936485713564605, 3327012975585973824, 6001912612374303716, 17419159457659073951, 11810720562576658327, 14802512641816370470, 751963320628219432, 9410455736958787393, 16405548341306967018, 6867376949398252373, 13982182448213113532],
    [10436926105997283389, 13237521312283579132, 668335841375552722, 2385521647573044240, 3874694023045931809, 12952434030222726182, 1972984540857058687, 14000313505684510403, 976377933822676506, 8407002393718726702, 338785660775650958, 4208211193539481671],
    [2284392243703840734, 4500504737691218932, 3976085877224857941, 2603294837319327956, 5760259105023371034, 2911579958858769248, 18415938932239013434, 7063156700464743997, 16626114991069403630, 163485390956217960, 11596043559919659130, 2976841507452846995],
    [15090073748392700862, 3496786927732034743, 8646735362535504000, 2460088694130347125, 3944675034557577794, 14781700518249159275, 2857749437648203959, 8505429584078195973, 18008150643764164736, 720176627102578275, 7038653538629322181, 8849746187975356582],
    [17427790390280348710, 1159544160012040055, 17946663256456930598, 6338793524502945410, 17715539080731926288, 4208940652334891422, 12386490721239135719, 10010817080957769535, 5566101162185411405, 12520146553271266365, 4972547404153988943, 5597076522138709717],
    [18338863478027005376, 115128380230345639, 4427489889653730058, 10890727269603281956, 7094492770210294530, 7345573238864544283, 6834103517673002336, 14002814950696095900, 15939230865809555943, 12717309295554119359, 4130723396860574906, 7706153020203677238],
]


def _mds(state):
    return [sum(MDS[i][j] * state[j] for j in range(STATE_WIDTH)) % P for i in range(STATE_WIDTH)]


def permutacion(state):
    """Rescue-XLIX: siete rondas de (sbox, MDS, ARK1, sbox inversa, MDS, ARK2)."""
    assert len(state) == STATE_WIDTH
    s = [x % P for x in state]
    for r in range(NUM_ROUNDS):
        s = [pow(x, ALPHA, P) for x in s]
        s = _mds(s)
        s = [(x + k) % P for x, k in zip(s, ARK1[r])]
        s = [pow(x, INV_ALPHA, P) for x in s]
        s = _mds(s)
        s = [(x + k) % P for x, k in zip(s, ARK2[r])]
    return s


# ─── BLAKE3, desde la especificacion publica ──────────────────────────────────────────────────
_IV = [0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19]
_PERM = [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8]
_CHUNK_START, _CHUNK_END, _PARENT, _ROOT = 1, 2, 4, 8
_M32 = 0xFFFFFFFF
_BLOQUE, _TROZO = 64, 1024


def _rotr(x, n):
    return ((x >> n) | (x << (32 - n))) & _M32


def _g(s, a, b, c, d, mx, my):
    s[a] = (s[a] + s[b] + mx) & _M32
    s[d] = _rotr(s[d] ^ s[a], 16)
    s[c] = (s[c] + s[d]) & _M32
    s[b] = _rotr(s[b] ^ s[c], 12)
    s[a] = (s[a] + s[b] + my) & _M32
    s[d] = _rotr(s[d] ^ s[a], 8)
    s[c] = (s[c] + s[d]) & _M32
    s[b] = _rotr(s[b] ^ s[c], 7)


def _compress(cv, m, counter, block_len, flags):
    s = list(cv) + _IV[:4] + [counter & _M32, (counter >> 32) & _M32, block_len, flags]
    m = list(m)
    for i in range(7):
        _g(s, 0, 4, 8, 12, m[0], m[1]); _g(s, 1, 5, 9, 13, m[2], m[3])
        _g(s, 2, 6, 10, 14, m[4], m[5]); _g(s, 3, 7, 11, 15, m[6], m[7])
        _g(s, 0, 5, 10, 15, m[8], m[9]); _g(s, 1, 6, 11, 12, m[10], m[11])
        _g(s, 2, 7, 8, 13, m[12], m[13]); _g(s, 3, 4, 9, 14, m[14], m[15])
        if i < 6:
            m = [m[_PERM[j]] for j in range(16)]
    return [s[i] ^ s[i + 8] for i in range(8)] + [s[i + 8] ^ cv[i] for i in range(8)]


def _palabras(bloque):
    b = bloque + bytes(_BLOQUE - len(bloque))
    return [int.from_bytes(b[4 * i:4 * i + 4], "little") for i in range(16)]


def _salida_trozo(trozo, contador):
    """(cv_entrada, palabras, contador, largo, banderas) del ULTIMO bloque del trozo, sin ROOT."""
    bloques = [trozo[i:i + _BLOQUE] for i in range(0, len(trozo), _BLOQUE)] or [b""]
    cv = list(_IV)
    for i, blq in enumerate(bloques):
        flags = (_CHUNK_START if i == 0 else 0) | (_CHUNK_END if i == len(bloques) - 1 else 0)
        if i == len(bloques) - 1:
            return cv, _palabras(blq), contador, len(blq), flags
        cv = _compress(cv, _palabras(blq), contador, len(blq), flags)[:8]


def _salida_nodo(datos, contador):
    if len(datos) <= _TROZO:
        return _salida_trozo(datos, contador)
    trozos_llenos = (len(datos) - 1) // _TROZO
    izq = (1 << (trozos_llenos.bit_length() - 1)) * _TROZO
    cv_izq = _cv(_salida_nodo(datos[:izq], contador))
    cv_der = _cv(_salida_nodo(datos[izq:], contador + izq // _TROZO))
    return list(_IV), cv_izq + cv_der, 0, _BLOQUE, _PARENT


def _cv(salida):
    cv, m, contador, largo, flags = salida
    return _compress(cv, m, contador, largo, flags)[:8]


def blake3(datos):
    cv, m, _contador, largo, flags = _salida_nodo(bytes(datos), 0)
    palabras = _compress(cv, m, 0, largo, flags | _ROOT)
    return b"".join(w.to_bytes(4, "little") for w in palabras[:8])


# ─── Digest: cuatro elementos; serializacion de NUCLEO.md seccion 6 ──────────────────────────

def digest_to_bytes(d):
    assert len(d) == 4
    return b"".join(x.to_bytes(8, "little") for x in d)


# La canonicidad (RFC-0016, NUCLEO.md seccion 6, <<Canonicidad>>): un u64 escribe un elemento solo si
# es menor que p. Un productor en esta implementacion, como `u64_canonico` en la de referencia, y el
# MISMO texto, que fijan los manifiestos de los vectores del S631.
NO_CANONICO = "{:#x} no es canonico: no es menor que p = 2^64 - 2^32 + 1"


def u64_canonico(u):
    if not 0 <= u < P:
        raise ValueError(NO_CANONICO.format(u))
    return u


def digest_from_bytes(b):
    if len(b) != 32:
        raise ValueError("digest_from_bytes exige 32 bytes")
    return [u64_canonico(int.from_bytes(b[8 * i:8 * i + 8], "little")) for i in range(4)]


def embeber(x):
    return [x % P, 0, 0, 0]


def as_digest(u):
    if not 0 <= u < 2**64:
        raise ValueError("as_digest espera un u64")
    return embeber(u)


def native_merge(l, r):
    """Estado a cero, l en [4..8], r en [8..12], una permutacion, salida [4..8]; capacidad a cero."""
    state = [0] * 4 + list(l) + list(r)
    return permutacion(state)[DIGEST]


merge = native_merge


# ─── los dominios: u64 big-endian de ocho bytes ASCII ─────────────────────────────────────────

def dominio(ascii8):
    assert len(ascii8) == 8
    return int.from_bytes(ascii8, "big")


ACUSE_V1 = dominio(b"ACUSE_V1")
RECEP_V1 = dominio(b"RECEP_V1")
MMRHOJA1 = dominio(b"MMRHOJA1")
MMRNODO1 = dominio(b"MMRNODO1")
PARAM_V1 = dominio(b"PARAM_V1")
ANCLA_V1 = dominio(b"ANCLA_V1")
ACTAS_V1 = dominio(b"ACTAS_V1")
DOMINIO_ACTA_FIRMA = b"ZK-SSL-key-act"
DOMINIO_EPOCH_HEAD = b"ZK-SSL-epoch-head"
DOMINIO_COFIRMA = b"ZK-SSL-witness-cosign"
DOMINIO_ANCLA_CLAVE = b"ZK-SSL-anchor-key-v1"
DOMINIO_LOTE = b"ZK-SSL-batch-v1"


# ─── las composiciones, en el orden exacto de los merges (NUCLEO.md seccion 6) ───────────────

def native_leaf(pk, saldo, nonce):
    return merge(merge(pk, embeber(saldo)), embeber(nonce))


def native_leaf_salted(pk, saldo, nonce, salt):
    return merge(native_leaf(pk, saldo, nonce), salt)


def path_root(leaf, siblings, is_right):
    nodo = list(leaf)
    for hermano, derecha in zip(siblings, is_right):
        nodo = merge(hermano, nodo) if derecha else merge(nodo, hermano)
    return nodo


def epoch_digest(seq, accounts, pending, frozen, chain):
    return merge(merge(merge(as_digest(seq), accounts), merge(pending, frozen)), chain)


def epoch_digest_v2(v1, acuses_root, n):
    return merge(v1, merge(acuses_root, as_digest(n)))


def epoch_digest_v3(v2, cima_mmr, t):
    return merge(v2, merge(cima_mmr, as_digest(t)))


def epoch_digest_v4(v3, cons_root, cons_count):
    return merge(v3, merge(cons_root, as_digest(cons_count)))


def epoch_digest_v5(v4, params, pmeta_root, next_pending, next_index, total_supply):
    return merge(v4, merge(merge(params, pmeta_root),
                           merge(as_digest(next_pending), merge(as_digest(next_index), as_digest(total_supply)))))


def epoch_digest_v6(v5, recep_root, recep_count):
    return merge(v5, merge(recep_root, as_digest(recep_count)))


def acuse_digest(hash_prueba, epoca, n):
    return merge(as_digest(ACUSE_V1), merge(hash_prueba, merge(as_digest(epoca), as_digest(n))))


def recibo_digest(hash_prueba, era, n):
    return merge(as_digest(RECEP_V1), merge(hash_prueba, merge(as_digest(era), as_digest(n))))


def ancla_digest(huella_clave, indice, epoch_dig, mmr_root, mmr_size):
    return merge(as_digest(ANCLA_V1),
                 merge(huella_clave, merge(as_digest(indice), merge(epoch_dig, merge(mmr_root, as_digest(mmr_size))))))


def acta_digest(huella_clave, esquema, desde, siguiente, procedencia):
    """RFC-0015 D-C y NUCLEO.md seccion 6: el dominio, la etiqueta (0 la genesis, 1 la rotacion) y
    el cuerpo `merge(clave, merge(esquema, merge(desde, siguiente)))`; en la rotacion, delante del
    cuerpo, la anterior y la historia `merge(epoch_digest, merge(mmr_root, mmr_size))`.
    `procedencia`: None, o (anterior, epoch_digest, mmr_root, mmr_size)."""
    cuerpo = merge(huella_clave, merge(as_digest(esquema), merge(as_digest(desde), siguiente)))
    if procedencia is None:
        etiqueta, resto = 0, cuerpo
    else:
        anterior, epoch_dig, mmr_root, mmr_size = procedencia
        historia = merge(epoch_dig, merge(mmr_root, as_digest(mmr_size)))
        etiqueta, resto = 1, merge(anterior, merge(historia, cuerpo))
    return merge(as_digest(ACTAS_V1), merge(as_digest(etiqueta), resto))


def params_digest(regulatory_limit, max_supply, max_accounts, custodian_set_root, governance_set_root,
                  refund_ttl, max_custodian_uses):
    return merge(as_digest(PARAM_V1),
                 merge(merge(as_digest(regulatory_limit), as_digest(max_supply)),
                       merge(merge(as_digest(max_accounts), custodian_set_root),
                             merge(governance_set_root, merge(as_digest(refund_ttl), as_digest(max_custodian_uses))))))


def mmr_hoja(cabeza):
    return merge(as_digest(MMRHOJA1), cabeza)


def mmr_nodo(izq, der):
    return merge(as_digest(MMRNODO1), merge(izq, der))


def cima(hojas, con_hoja=True):
    """El arbol de Merkle con el corte en la mayor potencia de dos menor que n (RFC 6962).
    `con_hoja`: si cada hoja pasa por `mmr_hoja` dentro de la cima (lectura RFC 6962) o entra cruda."""
    n = len(hojas)
    if n == 0:
        raise ValueError("cima de cero hojas")
    if n == 1:
        return mmr_hoja(hojas[0]) if con_hoja else list(hojas[0])
    k = 1 << (n - 1).bit_length() - 1
    return mmr_nodo(cima(hojas[:k], con_hoja), cima(hojas[k:], con_hoja))


def hoja_de_acuse(hash_prueba, seq, n):
    """NUCLEO.md seccion 6 no la escribia; PAQUETE.md, <<4. Lo que se comprueba, en orden>>, paso 3/3, la
    nombra hoja_de_acuse(hashPrueba, seq, n), y RPC.md, <<`zkssl_ackPath`>>, da la hoja como
    acuse_digest(hashPrueba, epoca, n).

    MEDIDO contra el KAT: la epoca de la hoja es seq + 1. El <<+1>> SI esta en la spec, pero en otra
    pagina: RPC.md, <<El acuse en la respuesta (S274)>>: <<epoca = logSeq + 1: la primera cabeza que puede
    contener la operacion>>. Lo
    que ninguna de las tres paginas dice es que el `seq` de hoja_de_acuse es ese logSeq. Quien lea
    solo la seccion 6 de NUCLEO.md, que es lo que su seccion 1 promete que basta, no la reproduce:
    con epoca = seq el KAT no cuadra (probado) y con seq + 1 si."""
    return acuse_digest(hash_prueba, seq + 1, n)


def limbos_reducidos(b32):
    """RFC-0016 (S631): un PRODUCTOR reduce, un LECTOR rechaza. Los 32 bytes de Blake3 son cuatro u64
    little-endian, y la referencia hace cada elemento con `BaseElement::new`, que REDUCE modulo P
    (`resumen_con_dominio`, el molde del S116): la salida es un digest canonico y una FUNCION de los
    bytes. Hasta el S631 esta implementacion devolvia los bytes de Blake3 tal cual y el lector los
    rechazaba si un limbo no cabia (2^-32 por limbo): divergia de la referencia en vez de reducir."""
    return digest_to_bytes([int.from_bytes(b32[8 * i:8 * i + 8], "little") % P for i in range(4)])


def huella_de_clave(clave):
    return limbos_reducidos(blake3(DOMINIO_ANCLA_CLAVE + len(clave).to_bytes(8, "little") + bytes(clave)))


def hash_del_lote(operaciones):
    """operaciones: lista de (hash_prueba_bytes32, cuenta_u64, posicion_u64), en el orden del lote.

    MEDIDO contra el KAT: la longitud codificada es la de los BYTES de la composicion, 48*k, como en
    el molde del S116 (`len(u64 LE)` de lo que sigue); NO es k. La frase <<k va en la longitud>> de
    NUCLEO.md seccion 6 y de RFC-0014 E2 hay que leerla como <<k queda implicito en la longitud>>."""
    cuerpo = b"".join(bytes(h) + c.to_bytes(8, "little") + p.to_bytes(8, "little") for h, c, p in operaciones)
    return limbos_reducidos(blake3(DOMINIO_LOTE + len(cuerpo).to_bytes(8, "little") + cuerpo))


def preambulo(version, epoch_dig):
    return DOMINIO_EPOCH_HEAD + bytes([version]) + digest_to_bytes(epoch_dig)


def preambulo_acta(version, acta_dig):
    return DOMINIO_ACTA_FIRMA + bytes([version]) + digest_to_bytes(acta_dig)


def preambulo_cofirma(version, epoch_dig, clave_op):
    return DOMINIO_COFIRMA + bytes([version]) + digest_to_bytes(epoch_dig) + len(clave_op).to_bytes(2, "big") + bytes(clave_op)


# ─── autotest de las primitivas, contra referencias ajenas a la casa ──────────────────────────
# BLAKE3: salida del crate `blake3 1.8.5` (el del Cargo.lock) sobre el patron byte i = i % 251.
_BLAKE3_REFERENCIA = {
    0: "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
    1: "2d3adedff11b61f14c886e35afa036736dcd87a74d27b5c1510225d0f592e213",
    3: "e1be4d7a8ab5560aa4199eea339849ba8e293d55ca0a81006726d184519e647f",
    63: "e9bc37a594daad83be9470df7f7b3798297c3d834ce80ba85d6e207627b7db7b",
    64: "4eed7141ea4a5cd4b788606bd23f46e212af9cacebacdc7d1f4c6dc7f2511b98",
    65: "de1e5fa0be70df6d2be8fffd0e99ceaa8eb6e8c93a63f2d8d1c30ecb6b263dee",
    127: "d81293fda863f008c09e92fc382a81f5a0b4a1251cba1634016a0f86a6bd640d",
    128: "f17e570564b26578c33bb7f44643f539624b05df1a76c81f30acd548c44b45ef",
    1023: "10108970eeda3eb932baac1428c7a2163b0e924c9a9e25b35bba72b28f70bd11",
    1024: "42214739f095a406f3fc83deb889744ac00df831c10daa55189b5d121c855af7",
    1025: "d00278ae47eb27b34faecf67b4fe263f82d5412916c1ffd97c8cb7fb814b8444",
    2048: "e776b6028c7cd22a4d0ba182a8bf62205d2ef576467e838ed6f2529b85fba24a",
    2049: "5f4d72f40d7a5f82b15ca2b2e44b1de3c2ef86c426c95c1af0b6879522563030",
    3072: "b98cb0ff3623be03326b373de6b9095218513e64f1ee2edd2525c7ad1e5cffd2",
    4096: "015094013f57a5277b59d8475c0501042c0b642e531b0a1c8f58d2163229e969",
    4097: "9b4052b38f1c5fc8b1f9ff7ac7b27cd242487b3d890d15c96a1c25b8aa0fb995",
    8192: "aae792484c8efe4f19e2ca7d371d8c467ffb10748d8a5a1ae579948f718a2a63",
    31744: "62b6960e1a44bcc1eb1a611a8d6235b6b4b78f32e7abc4fb4c6cdcce94895c47",
    102400: "bc3e3d41a1146b069abffad3c0d44860cf664390afce4d9661f7902e7943e085",
}


def autotest():
    """Devuelve la lista de fallos (vacia si todo cuadra)."""
    fallos = []
    for n, esperado in _BLAKE3_REFERENCIA.items():
        got = blake3(bytes(i % 251 for i in range(n))).hex()
        if got != esperado:
            fallos.append(f"blake3({n} bytes): esperado {esperado[:16]}.., obtenido {got[:16]}..")
    # Rescue: el vector de `winter-crypto 0.13.1` (rp64_256/tests.rs, apply_permutation), que su
    # propio comentario atribuye a la implementacion de referencia en Sage.
    esperado = [11084501481526603421, 6291559951628160880, 13626645864671311919, 18397438323058963117,
                7443014167353970324, 17930833023906771425, 4275355080008025761, 7676681476902901785,
                3460534574143792217, 11912731278641497187, 8104899243369883110, 674509706691634438]
    if permutacion(list(range(12))) != esperado:
        fallos.append("la permutacion Rescue no reproduce el vector Sage de winter-crypto")
    if (INV_ALPHA * ALPHA) % (P - 1) != 1:
        fallos.append("INV_ALPHA no es la inversa de ALPHA modulo p-1")
    x = 123456789
    if pow(pow(x, ALPHA, P), INV_ALPHA, P) != x:
        fallos.append("la sbox inversa no deshace la sbox")
    return fallos


if __name__ == "__main__":
    f = autotest()
    print("autotest:", "VERDE" if not f else "ROJO"); [print("  ", x) for x in f]
    raise SystemExit(1 if f else 0)

