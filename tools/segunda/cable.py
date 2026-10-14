#!/usr/bin/env python3
"""La cadena del REGISTRO de transiciones, como la sirve el cable (spec/RPC.md, <<La composicion de
`chain`, byte a byte>>), en Python y sin dependencias. Lo juzga `juez_cadena.py`.

Que es, y que NO es. ⚠️ §797: esto NO es nucleo. `chain` es de la clase REGISTRO de NUCLEO.md
(<<2. Las cuatro clases>>): la componen el nodo y el cliente por el cable, y el nucleo congelado es el
lado del verificador, no el cable (RFC-0005, D-A, spec/rfc/0005-nucleo-congelado.md). Por eso vive
aqui y no en `nucleo.py`, que reproduce lo que `spec/vectors/nucleo/` fija con un KAT y nada mas: de
`nucleo.py` se toman las PRIMITIVAS (la permutacion, el merge, el embebido, la serializacion, BLAKE3
y la reduccion del productor), no se le anade ninguna composicion. Sirve a un segundo implementador
del CLIENTE que quiera recomponer `zkssl_logEntries` sin fiarse del nodo.

De donde sale cada pieza: de spec/RPC.md, la seccion de arriba, y de ningun `.rs`. La seccion cita
el codigo que la produce (`crates/zk-ssl/src/log.rs`, `chain_digest` y `chain_digest_v2`;
`crates/zk-ssl-hash/src/lib.rs`, el molde y los sellos); este fichero no lo lee.

Lo que NO afirma: que `conformance --check` pase (exige el probador, y desde el S538 declara
`proof_digest` y `chain` no reproducibles: RFC-0009 D-AI); ni nada sobre el `proof_digest` de las
clases con prueba real, que solo los bytes de la prueba fijan.

⚠️ Lo que ningun vector contrasta (§797). Los cuatro vectores del cable (spec/vectors/zkssl-0.1.json a
zkssl-0.4.json) solo traen OpenAccount, Mint, Send y Claim: de `ETIQUETA` solo el 1, el 2, el 8 y
el 9 estan medidos contra el nodo, y los otros nueve son transcripcion de spec/RPC.md sin contraste.
De `proof_digest_esperado`, solo el sello de ausencia (OpenAccount) y el de autorizacion con Mint;
las otras cuatro delegadas y `Consumo`, igual. `juez_cadena.py` lo calcula y lo dice en sus lineas
`SIN EJERCITAR`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import nucleo as N  # noqa: E402

merge, as_digest = N.native_merge, N.as_digest

# ─── la etiqueta de `kind` (OpKind::tag; el texto del cable es el nombre de la variante) ─────────
# ⚠️ Medidas contra un vector: 1, 2, 8 y 9. Las otras nueve, sin entrada que las ejercite (§797).
ETIQUETA = {
    "OpenAccount": 1, "Mint": 2, "Transfer": 3, "Burn": 4, "Recovery": 5, "Governance": 6, "Freeze": 7,
    "Send": 8, "Claim": 9, "MintToPending": 10, "Migration": 11, "Refund": 12, "Consumo": 13,
}

# El anterior de la entrada seq 0: el digest cero (la cabeza de un registro vacio).
CERO = [0, 0, 0, 0]

# El centinela de la era 2 para las clases sin compromiso autorizante: un valor DECLARADO, no un hash.
COMPROMISO_AUSENTE = [0xC0A92281] * 4


def etiqueta(kind):
    if kind not in ETIQUETA:
        raise ValueError(f"kind desconocido: {kind!r} (no tiene etiqueta)")
    return ETIQUETA[kind]


def chain_v1(seq, kind, root_old, root_new, proof_digest, anterior):
    """Era 1 (entrada sin `compromiso`): cabecera, raices, cuerpo, y el digest de prueba y el anterior."""
    cabecera = merge(as_digest(seq), as_digest(etiqueta(kind)))
    raices = merge(root_old, root_new)
    cuerpo = merge(cabecera, raices)
    return merge(merge(cuerpo, proof_digest), anterior)


def chain_v2(seq, kind, root_old, root_new, proof_digest, anterior, compromiso):
    """Era 2 (entrada con `compromiso`, desde el S281): la de era 1 entera, como subarbol, y el compromiso."""
    return merge(chain_v1(seq, kind, root_old, root_new, proof_digest, anterior), compromiso)


def chain(seq, kind, root_old, root_new, proof_digest, anterior, compromiso=None):
    """La era la decide la PRESENCIA del compromiso, entrada a entrada; no hay byte de version."""
    if compromiso is None:
        return chain_v1(seq, kind, root_old, root_new, proof_digest, anterior)
    return chain_v2(seq, kind, root_old, root_new, proof_digest, anterior, compromiso)


# ─── lo que asienta cada clase como `proofDigest` (la tabla de la seccion) ─────────────────────
DOMINIO_PRUEBA = b"ZK-SSL-proof-digest-v2"
DOMINIO_AUTORIZACION = b"ZK-SSL-authorization-seal-v1"
DOMINIO_SIN_PRUEBA = b"ZK-SSL-no-proof-by-design-v1"

DELEGADAS = ("Mint", "Freeze", "Recovery", "Governance", "MintToPending")
CON_PRUEBA = ("Send", "Claim", "Burn", "Refund")


def digest_of_proof(b):
    """El molde del S116: dominio, longitud en u64 LE y los bytes; un productor REDUCE modulo p."""
    b = bytes(b)
    return N.digest_from_bytes(N.limbos_reducidos(N.blake3(DOMINIO_PRUEBA + len(b).to_bytes(8, "little") + b)))


def sello_sin_prueba():
    return DOMINIO_SIN_PRUEBA


def sello_de_autorizacion(compromiso):
    return DOMINIO_AUTORIZACION + N.digest_to_bytes(compromiso)


ERA_1 = ("era 1: el sello depende de cuando se asento (hasta el S209 el hash algebraico; del S209 al"
         " S278 la prueba vacia en OpenAccount y las delegadas; del S278 al S281 los sellos, sin el"
         " compromiso en la entrada) y la entrada no lo dice")


def proof_digest_esperado(kind, compromiso):
    """(digest, None) si la clase lo fija sin la prueba; (None, motivo) si queda DECLARADO.

    Solo para la ERA 2 (desde el S281): la columna `proofDigest` de la tabla vale desde el S278, y una
    entrada de era 1 no dice de que lado cae (§797, de la revision: la primera redaccion esperaba de
    una OpenAccount de era 1 el sello de ausencia, que no vale del S209 al S278)."""
    if compromiso is None:
        return None, ERA_1
    if kind == "OpenAccount":
        return digest_of_proof(sello_sin_prueba()), None
    if kind in DELEGADAS:
        return digest_of_proof(sello_de_autorizacion(compromiso)), None
    if kind == "Consumo":
        return digest_of_proof(b""), None
    if kind in CON_PRUEBA:
        return None, "bytes de la prueba: el registro no los guarda y desde el S538 no se reproducen (D-AI)"
    if kind == "Migration":
        return None, "el payload de la migracion no viaja en la entrada"
    return None, "sin llamador en la capa"


def coincidencia_era1(proof_digest):
    """Con que valor conocido, sin la prueba, coincide el `proof_digest` de una entrada de era 1."""
    if proof_digest == digest_of_proof(b""):
        return "la prueba vacia, digest_of_proof([]) (OpenAccount y delegadas del S209 al S278)"
    if proof_digest == digest_of_proof(sello_sin_prueba()):
        return "el sello de ausencia declarada (OpenAccount del S278 al S281)"
    return None


def compromiso_esperado(kind):
    """El centinela, para las clases que no tienen compromiso propio; None si la entrada lleva el suyo."""
    if kind in DELEGADAS or kind == "Consumo":
        return None
    return COMPROMISO_AUSENTE
