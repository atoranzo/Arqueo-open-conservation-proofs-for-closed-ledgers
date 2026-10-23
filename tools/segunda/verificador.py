#!/usr/bin/env python3
"""La SEGUNDA implementacion del mando (BACKLOG 85, tercer hito): un verificador del paquete de
evidencia escrito desde spec/PAQUETE.md, en Python y sin dependencias, con el contrato de salida del
mando (seccion 6): un argumento, la ruta de un sobre; las lineas numeradas de su forma por la salida
estandar y el VERDE de su forma; `ROJO: {motivo}` por la de error con un texto del catalogo de la
seccion 5; exit 0 verde, 1 el primer fallo con nombre, 2 uso.

Lo que lee hoy: las cinco formas que no exigen una prueba STARK -POSICION (v1 y v2, con acuse y
cofirmas), EXTENSION, CONSUMO, CONFLICTO y ANCLA-, y desde el §649 las `actas` que unen dos claves de
un operador en la extension y el consumo (RFC-0015); las que exigen una prueba quedan fuera. ⚠️ §810: aqui
faltaba el sobre del SOLAPAMIENTO, que lee desde el §686 (RFC-0015 E5c). Y desde el §810 el del ANCLA
COFIRMADA (RFC-0013 E4a, PAQUETE.md 2.13): la cabeza, la nota checkpoint del medio con su firma ML-DSA-44
de tipo 0x06, las cofirmas con clave en el sobre y la inclusion RFC 9162 del ancla. Un `tipo` conocido que
esta implementacion no lee todavia sale ROJO con su nombre, nunca VERDE. Desde el §693 esos tipos estan DECLARADOS, en NO_SOPORTADOS, y su ROJO
lleva el texto reservado `NO SOPORTADO` (PAQUETE.md, seccion 6): tools/conformidad.sh no lo cuenta nunca como
un negativo superado. Y la causa de un rechazo se nombra con el texto del binario de referencia (seccion 6.1).

De donde sale: PAQUETE.md secciones 2.1 a 2.3, 3, 4, 5 y 6; NUCLEO.md seccion 6 a traves de
nucleo.py; RFC 8391 a traves de kat_xmss/xmss.py; y la consistencia del MMR de RFC 6962 (RFC 9162, 2.1.4.2)
con mmr_nodo como nodo. Desde el §810, FIPS 204 a traves de mldsa44.py, y RFC 9162 y las notas de C2SP a
traves de medio.py. Lo juzga tools/conformidad.sh con los manifiestos de spec/vectors/.
"""
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "kat_xmss"))  # xmss.py vive ahi
import medio  # noqa: E402
import nucleo as N  # noqa: E402
import xmss  # noqa: E402

VERSIONES_CABEZA = (2, 3, 4, 5, 6)
VERSIONES_EXTENSION = (3, 4, 5, 6)
COFIRMA_V_MAX = 1
TIPOS_CONOCIDOS = ("extension", "consumo", "conflicto", "rechazo", "edad", "cobro_pendiente", "pago_en_curso",
                   "prenda", "completitud", "ancla", "ancla-cofirmada", "solapamiento")
# §693: lo que esta implementacion NO lee, declarado. Salen `ROJO: NO SOPORTADO: ...`, el texto que el contrato
# reserva (PAQUETE.md, seccion 6), y el arnes no los cuenta nunca: ni como acierto ni como negativo superado.
# §810: `ancla-cofirmada` pasa de NO_SOPORTADOS a TIPOS_LEIDOS (ML-DSA-44 en mldsa44.py, la nota en medio.py).
TIPOS_LEIDOS = ("extension", "consumo", "conflicto", "ancla", "ancla-cofirmada", "solapamiento")
NO_SOPORTADOS = ("rechazo", "edad", "cobro_pendiente", "pago_en_curso", "prenda", "completitud")
assert sorted(TIPOS_LEIDOS + NO_SOPORTADOS) == sorted(TIPOS_CONOCIDOS), "un tipo conocido sin declarar"


class Rojo(Exception):
    """El primer fallo, con nombre (PAQUETE.md, seccion 5)."""


# ─── forma de los valores (seccion 5, <<Forma de los valores>>) ──────────────────────────────

def hex_a_bytes(s):
    if not s.startswith("0x"):
        raise Rojo(f"sin 0x: {s[:18]}")
    h = s[2:]
    if len(h) % 2 == 1:
        raise Rojo(f"hex impar ({len(h)} chars)")
    # §650: la regla canonica, la misma que el mando: solo [0-9a-f]. bytes.fromhex
    # admite mayusculas y se salta los espacios.
    malo = next((i for i, c in enumerate(h) if c not in "0123456789abcdef"), None)
    if malo is not None:
        raise Rojo(f"hex: cifra no admitida en la posicion {malo}")
    try:
        return bytes.fromhex(h)
    except ValueError as e:
        raise Rojo(f"hex: {e}")


def digest_de(obj, campo):
    s = obj.get(campo)
    if not isinstance(s, str):
        raise Rojo(f"falta {campo} o no es cadena")
    try:
        b = hex_a_bytes(s)
    except Rojo as e:
        raise Rojo(f"{campo}: {str(e)!r}")
    if len(b) != 32:
        raise Rojo(f"{campo}: {len(b)} bytes, se esperaban 32")
    try:
        return N.digest_from_bytes(b)
    except ValueError as e:
        raise Rojo(f"{campo}: {e}")  # RFC-0016: el texto de la referencia, que lo da por Display


def u64_de(obj, campo):
    s = obj.get(campo)
    if not isinstance(s, str):
        raise Rojo(f"falta {campo} o no es cadena 0x")
    if not s.startswith("0x"):
        raise Rojo(f"{campo} sin 0x")
    # §693: la escritura minima de zk_ssl_hash::cantidad_canonica (§662), con sus textos, como el mando y como
    # _q_del_acta. Hasta aqui int(s, 16): admitia mayusculas, ceros a la izquierda y `_`, y nombraba la
    # cifra mala con el texto de Rust de antes del §650.
    h = s[2:].encode("utf-8")
    if not h or len(h) > 16 or (len(h) > 1 and h[:1] == b"0"):
        raise Rojo(f"{campo}: cantidad hex no minima")
    malo = next((i for i, c in enumerate(h) if c not in b"0123456789abcdef"), None)
    if malo is not None:
        raise Rojo(f"{campo}: hex: cifra no admitida en la posicion {malo}")
    v = int(h, 16)
    try:
        return N.u64_canonico(v)  # RFC-0016 (S631): un u64 que entra en una composicion es menor que p
    except ValueError as e:
        raise Rojo(f"{campo}: {e}")


def bytes_de(obj, campo):
    s = obj.get(campo)
    if not isinstance(s, str):
        raise Rojo(f"falta {campo}")
    try:
        return hex_a_bytes(s)
    except Rojo as e:
        raise Rojo(f"{campo}: {e}")


# ─── la cabeza (pasos 1 y 2) ──────────────────────────────────────────────────────────────────

def leer_cabeza(c, sujeto=""):
    """Lee los campos en el orden de la seccion 4 y recompone el epochDigest: NUNCA se cree."""
    p = f"{sujeto}: " if sujeto else ""
    try:
        if c.get("available") is not True:
            raise Rojo("la cabeza empaquetada no era available:true" if not sujeto else "la cabeza no era available:true")
        v = u64_de(c, "formatVersion")
        if v not in VERSIONES_CABEZA:
            raise Rojo(f"formatVersion {v}: el paquete v1 empaqueta cabezas v2, v3, v4, v5 o v6 (la pareja acusesRoot/n "
                       f"viaja firmada desde §275; la del MMR, desde §292)")
        seq = u64_de(c, "seq")
        accounts = digest_de(c, "accountsRoot")
        pending = digest_de(c, "pendingRoot")
        frozen = digest_de(c, "frozenRoot")
        chain = digest_de(c, "chainDigest")
        acuses = digest_de(c, "acusesRoot")
        n = u64_de(c, "n")
        d = N.epoch_digest_v2(N.epoch_digest(seq, accounts, pending, frozen, chain), acuses, n)
        mmr_root = mmr_size = cons_root = cons_count = None
        if v >= 3:
            mmr_root = digest_de(c, "mmrRoot")
            mmr_size = u64_de(c, "mmrSize")
            d = N.epoch_digest_v3(d, mmr_root, mmr_size)
        if v >= 4:
            cons_root = digest_de(c, "consRoot")
            cons_count = u64_de(c, "consCount")
            d = N.epoch_digest_v4(d, cons_root, cons_count)
        if v >= 5:
            d = N.epoch_digest_v5(d, digest_de(c, "paramsDigest"), digest_de(c, "pmetaRoot"), u64_de(c, "nextPending"),
                                  u64_de(c, "nextIndex"), u64_de(c, "totalSupply"))
        if v >= 6:
            d = N.epoch_digest_v6(d, digest_de(c, "recepRoot"), u64_de(c, "recepCount"))
        empaquetado = digest_de(c, "epochDigest")
        if d != empaquetado:
            raise Rojo("los siete campos NO recomponen el epochDigest empaquetado: o el paquete esta adulterado o la "
                       "cabeza nunca fue esa" if not sujeto else "los campos NO recomponen su epochDigest — adulterada o inventada")
        pk = bytes_de(c, "publicKey")
        firmado = bytes_de(c, "signature")
        index = u64_de(c, "index")
    except Rojo as e:
        raise Rojo(p + str(e))
    return {"v": v, "seq": seq, "n": n, "acusesRoot": acuses, "mmrRoot": mmr_root, "mmrSize": mmr_size,
            "consRoot": cons_root, "consCount": cons_count, "digest": d, "pk": pk, "firmado": firmado, "index": index}


def verificar_cabeza(cab, sujeto=""):
    """Paso 2: la firma verifica, el preambulo recuperado ES el esperado, y el index declarado queda por
    encima del embebido (§399). Devuelve el indice embebido."""
    p = f"{sujeto}: cabeza: " if sujeto else "cabeza: "
    try:
        oid, _, _ = xmss.parsear_clave(cab["pk"])
    except ValueError as e:
        raise Rojo(p + str(e))
    largo = xmss.largo_firma(oid)
    firma, mensaje = cab["firmado"][:largo], cab["firmado"][largo:]
    if len(firma) != largo:
        raise Rojo(p + f"la firma tiene {len(firma)} bytes y RFC 8391 pide {largo}")
    # §714: primero la firma y despues el preambulo, en el orden y con el texto del kit
    # (`VerificaError::PreambuloDistinto`). Antes comparaba el preambulo primero y decia «el preambulo
    # recuperado no es el esperado»: una firma valida de OTRA cabeza -`ancla/neg-rama-firma-de-otra-cabeza`-
    # fallaba en las dos implementaciones con textos distintos, y su manifiesto no podia pinar la causa.
    if not xmss.verificar(cab["pk"], mensaje, firma):
        raise Rojo(p + "la firma no verifica")  # §693: la causa con el texto del mando (seccion 6.1)
    esperado = N.preambulo(cab["v"], cab["digest"])
    if mensaje != esperado:
        raise Rojo(p + f"la firma es VALIDA pero de otro mensaje (preambulo esperado {len(esperado)} bytes, "
                   f"recibido {len(mensaje)}). Verificar sin comparar no prueba nada.")
    embebido = xmss.indice_embebido(oid, firma)
    if cab["index"] <= embebido:
        raise Rojo(p + f"el indice declarado ({cab['index']}) no cuadra con el que va dentro de la firma ({embebido})")
    return embebido


# ─── el acuse (paso 3) y las cofirmas (paso 4) ────────────────────────────────────────────────

def leer_camino(cam):
    sib = cam.get("siblings")
    if sib is None:
        raise Rojo("camino sin siblings")
    der = cam.get("isRight")
    if der is None:
        raise Rojo("camino sin isRight")
    if not isinstance(sib, list):
        raise Rojo("siblings no es una lista")
    if not isinstance(der, list):
        raise Rojo("isRight no es una lista")
    siblings = []
    for i, s in enumerate(sib):
        if not isinstance(s, str):
            raise Rojo(f"sibling {i} no es cadena")
        try:
            b = hex_a_bytes(s)
        except Rojo as e:
            raise Rojo(f"sibling {i}: {str(e)!r}")
        if len(b) != 32:
            raise Rojo(f"sibling {i}: {len(b)} bytes")
        try:
            siblings.append(N.digest_from_bytes(b))
        except ValueError as e:
            raise Rojo(f"sibling {i}: {str(e)!r}")
    for r in der:
        if not isinstance(r, bool):
            raise Rojo("isRight no booleano")
    return siblings, der


def acuse(doc, cab):
    a = doc.get("acuse")
    if a is None:
        return "3/3 sin acuse en el paquete: la cabeza sola queda demostrada"
    cam = a.get("camino")
    if cam is None:
        raise Rojo("acuse sin camino")
    siblings, is_right = leer_camino(cam)
    seq = u64_de(a, "seq")
    hash_prueba = digest_de(a, "hashPrueba")
    hoja = N.hoja_de_acuse(hash_prueba, seq, cab["n"])
    if N.path_root(hoja, siblings, is_right) != cab["acusesRoot"]:
        raise Rojo("acuse: RaizDistinta")  # §693: el nombre que el mando le da (seccion 6.1)
    return f"3/3 el acuse sube hasta la raiz firmada: la entrada {seq} queda demostrada"


def cofirmas(doc, cab):
    lista = doc.get("cofirmas")
    if lista is None or lista == []:
        return "cofirmas: el paquete v2 no trae ninguna: la cabeza queda sola"
    if not isinstance(lista, list):
        raise Rojo("cofirmas no es una lista")
    testigos = []  # §663: un testigo, una cofirma
    for i, co in enumerate(lista, start=1):
        if not isinstance(co, dict):
            raise Rojo(f"cofirma {i}: no es un objeto")
        cv = u64_de(co, "v")
        if cv > COFIRMA_V_MAX or cv == 0:
            raise Rojo(f"cofirma {i}: version {cv} desconocida, este binario lee hasta la {COFIRMA_V_MAX}")
        for campo in ("epochDigest", "clavePublicaOperador", "clavePublicaTestigo", "firma", "versionFormato"):
            if not isinstance(co.get(campo), str):
                raise Rojo(f"cofirma {i}: falta {campo}")
        try:
            digest = digest_de(co, "epochDigest")
            clave_op = hex_a_bytes(co["clavePublicaOperador"])
            clave_t = hex_a_bytes(co["clavePublicaTestigo"])
            firmado = hex_a_bytes(co["firma"])
            version = u64_de(co, "versionFormato")
        except Rojo as e:
            raise Rojo(f"cofirma {i}: {e}")
        if digest != cab["digest"]:
            raise Rojo(f"cofirma {i}: acredita OTRA cabeza, no la empaquetada")
        if clave_op != cab["pk"]:
            raise Rojo(f"cofirma {i}: acredita a OTRO operador, no al que firmo la cabeza")
        if clave_t in testigos:
            raise Rojo(f"cofirma {i}: repite el testigo de la cofirma {testigos.index(clave_t) + 1}")
        testigos.append(clave_t)
        # §780: el preambulo lleva la version en UN byte; el kit lo dice con este texto, y aqui
        # bytes([version]) se caia con un ValueError en el mismo vector.
        if version > 0xFF:
            raise Rojo(f"cofirma {i}: versionFormato {version:#x} no cabe en el byte que el preambulo le reserva")
        try:
            oid, _, _ = xmss.parsear_clave(clave_t)
        except ValueError as e:
            raise Rojo(f"cofirma {i}: {e}")
        largo = xmss.largo_firma(oid)
        firma, mensaje = firmado[:largo], firmado[largo:]
        if mensaje != N.preambulo_cofirma(version, digest, clave_op):
            raise Rojo(f"cofirma {i}: el preambulo recuperado no es el esperado")
        if not xmss.verificar(clave_t, mensaje, firma):
            raise Rojo(f"cofirma {i}: la firma no verifica")  # §693: el texto del mando (seccion 6.1)
    return (f"cofirmas: {len(lista)} verifican contra ESTA cabeza y ESTE operador (cuantas hacen falta lo decide TU "
            f"politica, no el paquete)")


# ─── la consistencia del MMR (RFC 9162, 2.1.4.2, con mmr_nodo) ────────────────────────────────

def consistencia(first, second, first_hash, second_hash, proof):
    if first == 0 or first > second:
        return False
    if first == second:
        return not proof and first_hash == second_hash
    proof = list(proof)
    if first & (first - 1) == 0:
        proof = [first_hash] + proof
    if not proof:
        return False
    fn, sn = first - 1, second - 1
    while fn & 1:
        fn >>= 1
        sn >>= 1
    fr = sr = proof[0]
    for c in proof[1:]:
        if sn == 0:
            return False
        if fn & 1 or fn == sn:
            fr = N.mmr_nodo(c, fr)
            sr = N.mmr_nodo(c, sr)
            while not (fn & 1) and fn != 0:
                fn >>= 1
                sn >>= 1
        else:
            sr = N.mmr_nodo(sr, c)
        fn >>= 1
        sn >>= 1
    return fr == first_hash and sr == second_hash and sn == 0


# ─── las formas ──────────────────────────────────────────────────────────────────────────────

def posicion(doc, v):
    salida = []
    c = doc.get("cabeza")
    if c is None:
        raise Rojo("falta cabeza")
    cab = leer_cabeza(c)
    salida.append(f"1/3 los campos de la cabeza (v{cab['v']}) recomponen el epochDigest — el digest no se ha creido")
    embebido = verificar_cabeza(cab)
    salida.append(f"2/3 la firma verifica y el preambulo ES el esperado (indice de firma {embebido})")
    salida.append(acuse(doc, cab))
    if v == 2:
        salida.append(cofirmas(doc, cab))
    salida.append("VERDE: el paquete se sostiene sin el nodo")
    return salida


# ─── las actas de clave (RFC-0015 D-C y D-E; PAQUETE.md 2.3; §648, §649) ─────────────────────────
# El sobre que exige UN firmante acepta `actas`, la cadena de zkssl_keyActs: si une las dos claves,
# la continuidad es de UN operador. Escrito desde la D-C del RFC-0015 y la seccion 2.3 de PAQUETE.md,
# con el digest y el preambulo de NUCLEO.md seccion 6 (nucleo.py) y la firma de RFC 8391 (xmss.py).
# Los textos son los de la referencia, letra por letra, salvo los que alli son el `Debug` de un error
# de la biblioteca XMSS: aqui dicen lo mismo con las palabras de xmss.py.
ESQUEMA_XMSSMT_SHA2_40_8_256 = (1 << 32) | 0x00000005
ACTA_VERSION = 1


class Fallo(Exception):
    """Un acta que no se lee o no vale, con el texto de la referencia; quien lo recoge pone delante
    el sitio (`actas[k]: `, `el acta k de la cadena no vale: `)."""


def _texto_de(obj, campo):
    s = obj.get(campo) if isinstance(obj, dict) else None
    if not isinstance(s, str):
        raise Fallo(f"falta {campo}, o no es una cadena")
    return s


def _hex_canonico(s):
    """El hex del cable (zk_ssl_hash::hex_canonico): `0x`, minuscula, por pares; mide en bytes."""
    if not s.startswith("0x"):
        raise ValueError("hex sin 0x")
    h = s[2:].encode("utf-8")
    if len(h) % 2:
        raise ValueError(f"hex de longitud impar ({len(h)})")
    malo = next((i for i, c in enumerate(h) if c not in b"0123456789abcdef"), None)
    if malo is not None:
        raise ValueError(f"hex: cifra no admitida en la posicion {malo}")
    return bytes.fromhex(h.decode())


def _data_de(obj, campo):
    try:
        return _hex_canonico(_texto_de(obj, campo))
    except ValueError as e:
        raise Fallo(f"{campo}: {e}")


def _digest_del_acta(obj, campo):
    b = _data_de(obj, campo)
    if len(b) != 32:
        raise Fallo(f"{campo}: {len(b)} bytes, se esperaban 32")
    try:
        return N.digest_from_bytes(b)
    except ValueError as e:
        raise Fallo(f"{campo}: {e}")


def _q_del_acta(obj, campo):
    """Un Q en su escritura minima (zk_ssl_hash::cantidad_canonica, §662) y menor que p (RFC-0016,
    u64_canonico), como `q_de` de `actas.rs`. §718, la canonicidad: sin la cota, un acta
    firmada con mmrSize N verificaba igual con N + p, aqui y en la referencia (medido)."""
    s = _texto_de(obj, campo)
    if not s.startswith("0x"):
        raise Fallo(f"{campo}: hex sin 0x")
    h = s[2:].encode("utf-8")
    if not h or len(h) > 16 or (len(h) > 1 and h[:1] == b"0"):
        raise Fallo(f"{campo}: cantidad hex no minima")
    malo = next((i for i, c in enumerate(h) if c not in b"0123456789abcdef"), None)
    if malo is not None:
        raise Fallo(f"{campo}: hex: cifra no admitida en la posicion {malo}")
    try:
        return N.u64_canonico(int(h, 16))
    except ValueError as e:
        raise Fallo(f"{campo}: {e}")


def acta_de_json(v):
    """Lee el JSON de un acta (spec/RPC.md, zkssl_keyActs): leer NO es juzgar."""
    a = v.get("acta") if isinstance(v, dict) else None
    if not isinstance(a, dict):
        raise Fallo("falta acta, o no es un objeto")
    if "procedencia" not in a:
        raise Fallo("falta acta.procedencia: null en la genesis")
    p = a["procedencia"]
    procedencia = None if p is None else (_digest_del_acta(p, "anterior"), _digest_del_acta(p, "epochDigest"),
                                          _digest_del_acta(p, "mmrRoot"), _q_del_acta(p, "mmrSize"))
    if "firmaAnterior" not in v:
        raise Fallo("falta firmaAnterior: null si la clave que se va no firma")
    firma_anterior = None if v["firmaAnterior"] is None else _data_de(v, "firmaAnterior")
    return {"clave": _data_de(a, "clave"), "esquema": _q_del_acta(a, "esquema"), "desde": _q_del_acta(a, "desde"),
            "siguiente": _digest_del_acta(a, "siguiente"), "procedencia": procedencia,
            "firma": _data_de(v, "firma"), "firmaAnterior": firma_anterior}


def _huella(clave):
    return N.digest_from_bytes(N.huella_de_clave(clave))


def _firma_sobre(clave, mensaje, firmado):
    """La firma verifica, el mensaje que lleva dentro ES el esperado, y devuelve su indice embebido."""
    try:
        oid, _, _ = xmss.parsear_clave(clave)
    except ValueError as e:
        raise Fallo(f"clave publica ilegible: {e}")
    largo = xmss.largo_firma(oid)
    firma, recuperado = firmado[:largo], firmado[largo:]
    if len(firma) != largo:
        raise Fallo(f"firma ilegible: la firma tiene {len(firma)} bytes y RFC 8391 pide {largo}")
    if not xmss.verificar(clave, recuperado, firma):
        raise Fallo("la firma no verifica: XMSS^MT de RFC 8391")
    if recuperado != mensaje:
        raise Fallo(f"la firma es VALIDA pero de otro mensaje (preambulo esperado {len(mensaje)} bytes, recibido "
                    f"{len(recuperado)}). Verificar sin comparar no prueba nada.")
    return xmss.indice_embebido(oid, firma)


def verificar_acta(a, previa):
    """Las reglas 1 a 3 de la D-C, ANTES que las firmas: ninguna firma rescata una clave no comprometida."""
    if a["esquema"] != ESQUEMA_XMSSMT_SHA2_40_8_256:
        raise Fallo(f"el acta presenta una clave de esquema {a['esquema']:#x}, que este verificador no conoce")
    p = a["procedencia"]
    if p is None and previa is None:
        if a["firmaAnterior"] is not None:
            raise Fallo("un acta genesis no tiene clave anterior que la firme")
    elif p is None:
        raise Fallo("un acta genesis no continua a otra: no lleva procedencia")
    elif previa is None:
        raise Fallo("una rotacion se juzga contra el acta de la que viene, y no esta")
    else:
        if p[0] != _huella(previa["clave"]):
            raise Fallo("la anterior del acta no es la clave del acta previa")
        if _huella(a["clave"]) != previa["siguiente"]:
            raise Fallo("la clave que entra NO es la que el acta previa comprometio: la pre-rotacion la rechaza")
        if a["desde"] <= previa["desde"]:
            raise Fallo(f"el desde {a['desde']} no supera el {previa['desde']} de la clave previa: la cuenta es del "
                        f"operador")
    digest = N.acta_digest(_huella(a["clave"]), a["esquema"], a["desde"], a["siguiente"], p)
    pre = N.preambulo_acta(ACTA_VERSION, digest)
    try:
        embebido = _firma_sobre(a["clave"], pre, a["firma"])
    except Fallo as e:
        raise Fallo(f"la firma de la clave que entra: {e}")
    if embebido != a["desde"]:
        raise Fallo(f"la clave que entra firma su acta en la hoja {embebido} y el acta dice desde {a['desde']}")
    if a["firmaAnterior"] is not None and previa is not None:
        try:
            e = _firma_sobre(previa["clave"], pre, a["firmaAnterior"])
        except Fallo as x:
            raise Fallo(f"la firma de la clave que se va: {x}")
        if e <= previa["desde"] or e >= a["desde"]:
            raise Fallo(f"la clave que se va firma en la hoja {e}, fuera de ({previa['desde']}, {a['desde']})")


def _eslabones_del_paso(actas, de, a):
    """La ultima entrada de `a` y, antes, la ultima de `de`: un productor para la rotacion y el relevo."""
    j = next((k for k in range(len(actas) - 1, -1, -1) if actas[k]["clave"] == a), None)
    if j is None:
        raise Fallo("la clave que llega no esta en la cadena: nadie la comprometio")
    i = next((k for k in range(j - 1, -1, -1) if actas[k]["clave"] == de), None)
    if i is None:
        raise Fallo("la clave que se tenia no esta en la cadena antes de la que llega")
    return i, j


def juzgar_rotacion(actas, de, a, ultimo):
    """La cadena entera vale, `a` esta en ella y `de` antes, y lo visto de `de` queda por debajo del
    `desde` de su sucesora. Devuelve (desde, hasta, eslabones) del tramo de `a`."""
    for i, x in enumerate(actas):
        try:
            verificar_acta(x, actas[i - 1] if i else None)
        except Fallo as e:
            raise Fallo(f"el acta {i} de la cadena no vale: {e}")
    i, j = _eslabones_del_paso(actas, de, a)
    if ultimo is not None and ultimo >= actas[i + 1]["desde"]:
        raise Fallo(f"SOLAPAMIENTO: la clave que se va firmo en la hoja {ultimo}, y su sucesora empieza en la "
                    f"{actas[i + 1]['desde']}")
    return actas[j]["desde"], (actas[j + 1]["desde"] if j + 1 < len(actas) else None), j - i


def juzgar_continuidad(actas, una, otra):
    """El indice EMBEBIDO ordena las dos cabezas (la cuenta es una, D-A); la anterior, por la regla 3, y
    la posterior en su tramo, por la 4. Cada lado es (clave, embebido)."""
    anterior, posterior = (otra, una) if otra[1] < una[1] else (una, otra)
    desde, hasta, eslabones = juzgar_rotacion(actas, anterior[0], posterior[0], anterior[1])
    e = posterior[1]
    if not (e > desde and (hasta is None or e < hasta)):
        raise Fallo(f"SOLAPAMIENTO: la clave que llega firmo en la hoja {e}, fuera de su tramo: por encima de la "
                    f"{desde}" + (f" y por debajo de la {hasta}" if hasta is not None else ""))
    return desde, hasta, eslabones


def relevo_del_acta(a):
    """§722: el relevo de un acta, (mmrSize, mmrRoot) de su procedencia; la genesis no releva."""
    p = a["procedencia"]
    return None if p is None else (p[3], p[2])


def _borde_del_relevo(actas, k, talla, cima):
    """§722: el borde del relevo de la entrada `k` que cruza una cabeza con ese mmrSize y esa cima, o
    None si cae dentro. Abajo primero (el relevo de su acta, y en el la cima), despues el de la siguiente. Un
    productor para la extension y el solapamiento: con una cadena, un veredicto."""
    desde = relevo_del_acta(actas[k])
    hasta = relevo_del_acta(actas[k + 1]) if k + 1 < len(actas) else None
    if desde is not None:
        if talla < desde[0]:
            return ("antes", desde[0])
        if talla == desde[0] and cima != desde[1]:
            return ("otra-cima", desde[0])
    if hasta is not None and talla >= hasta[0]:
        return ("tras", hasta[0])
    return None


def juzgar_historia(actas, una, otra):
    """§722: la continuidad y el RELEVO (PAQUETE.md 2.3), en su orden. Cada lado es (clave, embebido,
    mmrSize, mmrRoot). 1, la anterior en un tramo de su clave hasta el paso, el que su indice dice; 2, los relevos
    que se cruzan no retroceden; 3, la anterior en el relevo de su tramo, por los dos bordes; 4, la posterior en el
    del suyo. Los textos son los de la referencia."""
    r = juzgar_continuidad(actas, una[:2], otra[:2])
    anterior, posterior = (otra, una) if otra[1] < una[1] else (una, otra)
    i, j = _eslabones_del_paso(actas, anterior[0], posterior[0])
    tramos = [(m, a["desde"], actas[m + 1]["desde"] if m + 1 < len(actas) else None)
              for m, a in enumerate(actas[:i + 1]) if a["clave"] == anterior[0]]
    k = next((t[0] for t in tramos if _contiene(t, anterior[1])), None)
    if k is None:
        raise Fallo(f"SOLAPAMIENTO: la clave que se va firmo en la hoja {anterior[1]}, fuera de todos sus tramos: la "
                    f"cuenta del operador la daba a otra clave suya o a un acta")
    for m in range(k + 2, j + 1):
        previo, actual = relevo_del_acta(actas[m - 1]), relevo_del_acta(actas[m])
        if previo is not None and actual is not None and actual[0] < previo[0]:
            raise Fallo(f"RELEVO: el acta {m} releva en el mmrSize {actual[0]}, por debajo del {previo[0]} de la previa: "
                        f"la historia de cabezas no retrocede")
    borde = _borde_del_relevo(actas, k, anterior[2], anterior[3])
    if borde is not None:
        tipo, s = borde
        if tipo == "antes":
            raise Fallo(f"RELEVO: la cabeza de la clave que se va lleva el mmrSize {anterior[2]}, y el acta {k} la hace "
                        f"entrar en el {s}: la clave que se va firma desde ahi")
        if tipo == "otra-cima":
            raise Fallo(f"RELEVO: la cabeza de la clave que se va firma el mmrSize {s} de su relevo con otra cima que la "
                        f"que el acta {k} le hace heredar")
        raise Fallo(f"RELEVO: la cabeza de la clave que se va lleva el mmrSize {anterior[2]}, y el acta {k + 1} la "
                    f"releva en el {s}: la clave que se va firma por debajo")
    borde = _borde_del_relevo(actas, j, posterior[2], posterior[3])
    if borde is not None:
        tipo, s = borde
        if tipo == "antes":
            raise Fallo(f"RELEVO: la cabeza de la clave que llega lleva el mmrSize {posterior[2]}, y el acta {j} la hace "
                        f"entrar en el {s}: la clave que llega firma desde ahi")
        if tipo == "otra-cima":
            raise Fallo(f"RELEVO: la cabeza de la clave que llega firma el mmrSize {s} de su relevo con otra cima que la "
                        f"que el acta {j} le hace heredar")
        raise Fallo(f"RELEVO: la cabeza de la clave que llega lleva el mmrSize {posterior[2]}, y el acta {j + 1} la "
                    f"releva en el {s}: la clave que llega firma por debajo")
    return r


def leer_actas(doc):
    """Las `actas` del sobre, LEIDAS (leer no es juzgar), con su posicion en el texto."""
    lista = doc["actas"]
    if not isinstance(lista, list):
        raise Rojo("actas no es lista (la cadena de zkssl_keyActs)")
    actas = []
    for k, x in enumerate(lista):
        try:
            actas.append(acta_de_json(x))
        except Fallo as e:
            raise Rojo(f"actas[{k}]: {e}")
    return actas


def _contiene(tramo, e):
    """La regla 4 de la D-C, con la genesis sin juzgar lo firmado antes de optar (§686)."""
    eslabon, desde, hasta = tramo
    abajo = e != desde if eslabon == 0 else e > desde
    return abajo and (hasta is None or e < hasta)


def _texto_de_tramo(tramo):
    eslabon, desde, hasta = tramo
    abajo = (f"acta 0, la genesis: cualquier hoja salvo la {desde} de su acta" if eslabon == 0
             else f"acta {eslabon}: por encima de la hoja {desde}")
    return f"({abajo}, por debajo de la {hasta})" if hasta is not None else f"({abajo})"


def solapamiento(doc):
    """PAQUETE.md 2.14 (§686, RFC-0015 E5c): una cabeza fuera de TODOS los tramos de su clave en la
    cadena es una hoja que la cuenta del operador ya daba a otra clave suya o a un acta. DETECCION."""
    salida = []
    if doc.get("cabeza") is None:
        raise Rojo("falta cabeza (la firmada que se juzga contra su tramo)")
    if "actas" not in doc:
        raise Rojo("falta actas (la cadena de zkssl_keyActs: sin ella no hay tramo que cruzar)")
    actas = leer_actas(doc)
    msg_v = lambda s, fv: (f"{s}: formatVersion {fv}: el sobre del solapamiento lee cabezas v3, v4, v5 o v6: las que "
                           f"firma un nodo con actas")
    cab = leer_cabeza_de(doc, "cabeza", (3, 4, 5, 6), msg_v)
    e = cab["embebido"]
    salida.append(f"1/3 la cabeza (v{cab['v']}) recompone su digest y su firma verifica (indice embebido {e})")
    for i, x in enumerate(actas):
        try:
            verificar_acta(x, actas[i - 1] if i else None)
        except Fallo as f:
            raise Rojo(f"el acta {i} de la cadena no vale: {f}")
    tramos = [(i, a["desde"], actas[i + 1]["desde"] if i + 1 < len(actas) else None)
              for i, a in enumerate(actas) if a["clave"] == cab["pk"]]
    if not tramos:
        raise Rojo("la clave de la cabeza no esta en la cadena: nadie la comprometio")
    salida.append(f"2/3 la cadena vale entera, {len(actas)} acta(s), y la clave de la cabeza entra en {len(tramos)} de "
                  f"ellas")
    for t in tramos:
        if _contiene(t, e):
            # §722: dentro de su tramo, el RELEVO (la otra cuenta, la de la historia de cabezas).
            desde = relevo_del_acta(actas[t[0]])
            hasta = relevo_del_acta(actas[t[0] + 1]) if t[0] + 1 < len(actas) else None
            talla, cima = cab["mmrSize"], cab["mmrRoot"]
            borde = _borde_del_relevo(actas, t[0], talla, cima)
            if borde is None:
                raise Rojo(f"la cabeza firma en la hoja {e}, dentro de su tramo {_texto_de_tramo(t)}: no hay solapamiento")
            otra = ", con otra cima que la que hereda" if borde[0] == "otra-cima" else ""
            texto = (f"acta {t[0]}, la genesis" if desde is None
                     else f"acta {t[0]}: desde el mmrSize {desde[0]}, con la cima que hereda")
            texto = f"({texto}, por debajo del mmrSize {hasta[0]})" if hasta is not None else f"({texto})"
            salida.append(f"3/3 la hoja {e} cae en su tramo {_texto_de_tramo(t)}, y el mmrSize {talla}{otra} fuera de su "
                          f"relevo {texto}")
            salida.append(f"VERDE: SOLAPAMIENTO - la clave firmo una cabeza con el mmrSize {talla}, fuera del relevo de su "
                          f"tramo: la cadena da ese sitio de la historia de cabezas a otra clave del operador. Es DETECCION "
                          f"del operador: solo quien tiene la clave pudo firmarla")
            return salida
    salida.append(f"3/3 la hoja {e} cae fuera de los tramos de su clave: {', '.join(_texto_de_tramo(t) for t in tramos)}")
    salida.append(f"VERDE: SOLAPAMIENTO - la clave firmo una cabeza en la hoja {e}, que la cuenta del operador (RFC-0015 "
                  f"D-A) daba a otra clave suya o a un acta. Es DETECCION del operador: solo quien tiene la clave pudo "
                  f"firmarla")
    return salida


def misma_continuidad(doc, una, otra):
    """PAQUETE.md 2.3: con la misma clave, None y `actas` no se lee; sin `actas`, el rechazo de siempre;
    con ellas, el juez. Cada lado es (publicKey en bytes, indice embebido)."""
    if una[0] == otra[0]:
        return None
    if "actas" not in doc:
        raise Rojo("las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante")
    actas = leer_actas(doc)
    try:
        return juzgar_continuidad(actas, una, otra)
    except Fallo as e:
        raise Rojo(f"las cabezas llevan claves DISTINTAS y las actas no las unen: {e}")


def misma_historia(doc, una, otra):
    """§722: misma_continuidad con el relevo, para la extension y el consumo. Cada lado es la cabeza
    leida (dict con pk, embebido, mmrSize y mmrRoot)."""
    if una["pk"] == otra["pk"]:
        return None
    if "actas" not in doc:
        raise Rojo("las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante")
    actas = leer_actas(doc)
    lado = lambda c: (c["pk"], c["embebido"], c["mmrSize"], c["mmrRoot"])
    try:
        return juzgar_historia(actas, lado(una), lado(otra))
    except Fallo as e:
        raise Rojo(f"las cabezas llevan claves DISTINTAS y las actas no las unen: {e}")


def linea_de_continuidad(paso, r, resto=""):
    if r is None:
        return None
    return (f"{paso} claves distintas que las actas unen ({r[2]} eslabon(es), la posterior desde la hoja {r[0]})"
            f"{resto}")



def extension(doc):
    salida = []
    cabs = {}
    for cual in ("vieja", "nueva"):
        c = doc.get(cual)
        if c is None:
            raise Rojo(f"falta {cual}")
        try:
            fv = u64_de(c, "formatVersion") if c.get("available") is True else None
        except Rojo as e:
            raise Rojo(f"{cual}: {e}")
        if fv is not None and fv not in VERSIONES_EXTENSION:
            raise Rojo(f"{cual}: formatVersion {fv} — la extension exige cabezas v3, v4 o v5: una v2 no lleva la pareja del "
                       f"MMR que extender")
        cab = leer_cabeza(c, cual)
        cab["embebido"] = verificar_cabeza(cab, cual)
        cabs[cual] = cab
    vieja, nueva = cabs["vieja"], cabs["nueva"]
    salida.append("1/3 las DOS cabezas recomponen su digest y sus firmas verifican")  # §649, como la referencia
    r = misma_historia(doc, vieja, nueva)
    salida.append(linea_de_continuidad("2/3", r, ": el mismo OPERADOR en los dos extremos")
                  or "2/3 misma publicKey: el mismo firmante en los dos extremos")
    cam = doc.get("camino")
    if not isinstance(cam, list):
        raise Rojo("falta camino (lista de digests)")
    camino = []
    for i, s in enumerate(cam):
        if not isinstance(s, str):
            raise Rojo(f"camino[{i}] no es cadena")
        try:
            b = hex_a_bytes(s)
        except Rojo as e:
            raise Rojo(f"camino[{i}]: {str(e)!r}")
        if len(b) != 32:
            raise Rojo(f"camino[{i}]: {len(b)} bytes")
        try:
            camino.append(N.digest_from_bytes(b))
        except ValueError as e:
            raise Rojo(f"camino[{i}]: {str(e)!r}")
    if not consistencia(vieja["mmrSize"], nueva["mmrSize"], vieja["mmrRoot"], nueva["mmrRoot"], camino):
        raise Rojo(f"la nueva (t={nueva['mmrSize']}) NO extiende a la vieja (t={vieja['mmrSize']}): historia bifurcada, "
                   f"recortada, o camino que no es el suyo")
    salida.append("3/3 la cima nueva EXTIENDE a la vieja: consistencia O(log N), sin el registro")
    salida.append(el_reloj_avanza(vieja, nueva))  # §712: despues de la consistencia, como la referencia
    salida.append("VERDE: la extension se sostiene sin el nodo")
    return salida


def el_reloj_avanza(vieja, nueva, nombres=("la vieja", "la nueva")):
    """§712, RFC-0012 D-C: el reloj es el indice EMBEBIDO. Despues de la consistencia, que con el
    mismo mmrSize es la identidad: con el mismo mmrSize, el mismo digest; con mmrSize mayor y la misma clave,
    el embebido de la nueva estrictamente mayor. Con claves distintas el indice no se compara aqui: la cuenta
    es una por operador y el cruce entre sus claves lo juzgan los tramos de la cadena. Los textos son los de
    la referencia (`el_reloj_avanza`). Un lado es un dict con mmrSize, digest, pk y embebido.

    `nombres` da el sujeto del texto: el ancla del modo 3 usa el mismo juez como «el ancla» y «la cabeza».
    Sin el, con el mismo mmrSize, el modo 3 salia VERDE (medido)."""
    nv, nn = nombres
    tv, tn = vieja["mmrSize"], nueva["mmrSize"]
    if tv == tn:
        if vieja["digest"] != nueva["digest"]:
            raise Rojo(f"extension: mismo mmrSize y digest distinto - {nv} y {nn} firman la misma historia de "
                       f"{tv} cabeza(s) y no son la misma cabeza: una bifurcacion firmada, no una extension")
        return f"   {nv} y {nn} son la MISMA cabeza: mismo mmrSize y mismo digest"
    if vieja["pk"] != nueva["pk"]:
        return "   claves distintas: el orden de sus hojas lo juzgan los tramos de la cadena, no el reloj de una clave"
    ev, en = vieja["embebido"], nueva["embebido"]
    if en <= ev:
        raise Rojo(f"extension: el indice embebido no crece - {nn} (t={tn}) firma en la hoja {en} y {nv} "
                   f"(t={tv}) en la {ev}: con la misma clave, una cabeza posterior firma por encima (RFC-0012 D-C)")
    return f"   el reloj de la clave avanza con la historia: de la hoja {ev} a la {en} (RFC-0012 D-C)"


# ─── el arbol de consumos (RFC-0006 §413; NUCLEO.md, familia CONSUMO) ─────────────────────────
CONS_DEPTH = 63
HOJA_VACIA = [0, 0, 0, 0]


def posicion_de_consumo(consumo):
    """Los 63 bits bajos de los primeros ocho bytes del consumo (el primer elemento del digest)."""
    return consumo[0] & ((1 << CONS_DEPTH) - 1)


def is_right_de_posicion(pos):
    return [bool((pos >> i) & 1) for i in range(CONS_DEPTH)]


def leer_camino_consumo(doc, cual, clave=None):
    """`{cual}` es `presencia`, `ausencia` o `libro[i]`; `clave` es donde vive en el JSON si no es `cual`."""
    cam = doc.get(clave or cual)
    if cam is None:
        raise Rojo(f"falta {cual} (camino del consumo)")
    sib = cam.get("siblings") if isinstance(cam, dict) else None
    if sib is None:
        raise Rojo(f"{cual}: falta siblings")
    der = cam.get("isRight")
    if der is None:
        raise Rojo(f"{cual}: falta isRight")
    if not isinstance(sib, list) or not isinstance(der, list):
        raise Rojo(f"{cual}: siblings e isRight tienen que ser listas")
    siblings = []
    for i, x in enumerate(sib):
        if not isinstance(x, str):
            raise Rojo(f"{cual}: siblings[{i}] no es cadena")
        try:
            b = hex_a_bytes(x)
        except Rojo as e:
            raise Rojo(f"{cual}: siblings[{i}]: {str(e)!r}")
        if len(b) != 32:
            raise Rojo(f"{cual}: siblings[{i}]: {len(b)} bytes")
        try:
            siblings.append(N.digest_from_bytes(b))
        except ValueError as e:
            raise Rojo(f"{cual}: siblings[{i}]: {str(e)!r}")
    for i, r in enumerate(der):
        if not isinstance(r, bool):
            raise Rojo(f"{cual}: isRight[{i}] no es booleano")
    return siblings, der


def cruza_posicion(cual, siblings, is_right, pos):
    if len(siblings) != CONS_DEPTH or len(is_right) != CONS_DEPTH:
        raise Rojo(f"{cual}: el camino no tiene los {CONS_DEPTH} niveles del arbol de consumos")
    if is_right != is_right_de_posicion(pos):
        raise Rojo(f"{cual}: el isRight recibido NO es el de la posicion {pos} que el consumo DERIVA - un camino de otra "
                   f"posicion no prueba nada de este consumo")


def leer_cabeza_de(doc, cual, versiones, mensaje_version, sujeto=None):
    """Una cabeza bajo la clave `cual`, con el conjunto de versiones de su forma y su mensaje."""
    c = doc.get(cual)
    if c is None:
        raise Rojo(f"falta {cual}")
    sujeto = cual if sujeto is None else sujeto
    # §714: una cabeza que no es un objeto -una cadena, un numero- reventaba
    # en `c.get` con un traceback, y la salida 1 no nombraba la causa. Ahora es un ROJO con nombre.
    if not isinstance(c, dict):
        raise Rojo(f"{sujeto}: no es una cabeza (objeto)")
    try:
        fv = u64_de(c, "formatVersion") if c.get("available") is True else None
    except Rojo as e:
        raise Rojo(f"{sujeto}: {e}")
    if fv is not None and fv not in versiones:
        raise Rojo(mensaje_version(sujeto, fv))
    cab = leer_cabeza(c, sujeto)
    cab["embebido"] = verificar_cabeza(cab, sujeto)
    return cab


def consumo(doc):
    salida = []
    msg_v = lambda s, fv: ("el sobre de consumo exige cabezas v4 o v5: una v2 o v3 no lleva consRoot contra el que comprobar"
                           if fv in (2, 3) else f"{s}: formatVersion {fv} — la extension exige cabezas v3, v4 o v5: una v2 no "
                           f"lleva la pareja del MMR que extender")
    vieja = leer_cabeza_de(doc, "vieja", (4, 5, 6), msg_v)
    nueva = leer_cabeza_de(doc, "nueva", (4, 5, 6), msg_v)
    salida.append("1/5 las DOS cabezas recomponen su digest y sus firmas verifican")
    r = misma_historia(doc, vieja, nueva)
    salida.append(linea_de_continuidad("2/5", r, ", y las dos cabezas llevan consRoot (v4, v5 o v6) a los dos lados")
                  or "2/5 misma publicKey y las dos cabezas llevan consRoot (v4, v5 o v6) a los dos lados")
    camino = leer_lista_de_digests(doc, "camino")
    if not consistencia(vieja["mmrSize"], nueva["mmrSize"], vieja["mmrRoot"], nueva["mmrRoot"], camino):
        raise Rojo(f"la nueva (t={nueva['mmrSize']}) NO extiende a la vieja (t={vieja['mmrSize']}): historia bifurcada, "
                   f"recortada, o camino que no es el suyo")
    salida.append("3/5 la cima nueva EXTIENDE a la vieja: el «antes» es de esta historia")
    salida.append(el_reloj_avanza(vieja, nueva))  # §712: el mismo reloj que la extension
    cons = digest_de(doc, "consumo")
    pos = posicion_de_consumo(cons)
    caminos = {cual: leer_camino_consumo(doc, cual) for cual in ("presencia", "ausencia")}
    for cual, (sib, der) in caminos.items():
        cruza_posicion(cual, sib, der, pos)
    salida.append(f"4/5 los dos caminos son los de la posicion {pos}, DERIVADA del consumo")
    if N.path_root(cons, *caminos["presencia"]) != nueva["consRoot"]:
        raise Rojo("presencia: el camino NO sube al consRoot de la nueva")
    if N.path_root(HOJA_VACIA, *caminos["ausencia"]) != vieja["consRoot"]:
        raise Rojo("ausencia: la hoja vacia NO sube al consRoot de la vieja - el consumo YA estaba")
    salida.append("5/5 el consumo ESTA bajo la nueva y NO estaba bajo la vieja")
    salida.append("VERDE: el consumo se publico entre las dos cabezas, sin el nodo")
    return salida


def conflicto(doc):
    salida = []
    cons = digest_de(doc, "consumo")
    libros = doc.get("libros")
    if libros is None:
        raise Rojo("falta libros")
    if not isinstance(libros, list):
        raise Rojo("libros no es una lista")
    if len(libros) != 2:
        raise Rojo(f"el sobre de conflicto exige DOS libros: se recibieron {len(libros)}")
    msg_v = lambda s, fv: ("el sobre de conflicto exige cabezas v4 o v5: una v2 o v3 no lleva consRoot contra el que comprobar"
                           if fv in (2, 3) else f"{s}: formatVersion {fv} — el conflicto exige cabezas v4, v5 o v6")
    cabs = []
    for i, libro in enumerate(libros):
        cual = f"libro[{i}]"
        if not isinstance(libro, dict) or libro.get("cabeza") is None:
            raise Rojo(f"{cual}: falta cabeza")
        cabs.append(leer_cabeza_de(libro, "cabeza", (4, 5, 6), msg_v, sujeto=cual))
    salida.append("1/4 las DOS cabezas recomponen su digest y sus firmas verifican")
    if cabs[0]["pk"] == cabs[1]["pk"]:
        raise Rojo("las cabezas llevan la MISMA clave: un conflicto es entre DOS firmantes")
    salida.append("2/4 las dos cabezas son de operadores DISTINTOS, y las dos llevan consRoot (v4, v5 o v6)")
    pos = posicion_de_consumo(cons)
    caminos = []
    for i, libro in enumerate(libros):
        cual = f"libro[{i}]"
        sib, der = leer_camino_consumo(libro, cual, clave="presencia")
        cruza_posicion(cual, sib, der, pos)
        caminos.append((sib, der))
    salida.append(f"3/4 los dos caminos son los de la posicion {pos}, DERIVADA del consumo")
    for i, (sib, der) in enumerate(caminos):
        if N.path_root(cons, sib, der) != cabs[i]["consRoot"]:
            raise Rojo(f"libro[{i}]: el camino NO sube al consRoot de su cabeza")
    salida.append("4/4 el MISMO consumo esta bajo el consRoot de los DOS libros")
    salida.append("VERDE: dos libros aceptaron el mismo consumo. Es DETECCION, no prevencion:")
    salida.append("       nadie ordena entre libros, y que la unidad sea la misma es gobernanza")
    return salida


def leer_lista_de_digests(doc, campo):
    cam = doc.get(campo)
    if not isinstance(cam, list):
        raise Rojo(f"falta {campo} (lista de digests)")
    out = []
    for i, x in enumerate(cam):
        if not isinstance(x, str):
            raise Rojo(f"{campo}[{i}] no es cadena")
        try:
            b = hex_a_bytes(x)
        except Rojo as e:
            raise Rojo(f"{campo}[{i}]: {str(e)!r}")
        if len(b) != 32:
            raise Rojo(f"{campo}[{i}]: {len(b)} bytes")
        try:
            out.append(N.digest_from_bytes(b))
        except ValueError as e:
            raise Rojo(f"{campo}[{i}]: {str(e)!r}")
    return out


def huella_como_digest(huella):
    """La huella de la clave, como Digest para el merge de ancla_digest (NUCLEO.md seccion 6). Desde el S631
    (RFC-0016) `huella_de_clave` REDUCE cada limbo, como la referencia: su salida es canonica y esta lectura
    no puede fallar. Hasta entonces aqui habia un ROJO con nombre para el limbo fuera del campo, que la
    referencia reducia y aceptaba: las dos implementaciones divergian con probabilidad 2^-32 por limbo."""
    return N.digest_from_bytes(huella)


def bifurcacion(doc, cab, salida):
    """§714, PAQUETE.md 2.12, modo 5: la BIFURCACION. Dos cabezas firmadas del MISMO operador -la
    misma clave, o claves que las `actas` unen, con el juez de la extension- con el MISMO mmrSize y digests
    distintos: DETECCION con salida 0, como la vista dividida. Un nodo con su diario no repite un mmrSize:
    las dos solo salen de una bifurcacion o de un reinicio sin el diario. Los textos son los de la referencia
    (`bifurcacion_del_ancla`), en el mismo orden: la rama entera, el operador, el mmrSize y el digest."""
    msg_v = lambda s, fv: f"{s}: formatVersion {fv}: el ancla lee cabezas v3, v4, v5 o v6: la pareja del MMR viaja firmada desde ellas"
    # §714: la rama, sujeto de sus rojos y en el orden de la referencia -que sea un
    # objeto, su formatVersion, la version, y despues la cabeza entera-. `leer_cabeza_de` solo lee la
    # version si la cabeza es available:true, y aqui el orden era otro: con una v2 no disponible la
    # referencia decia «rama: formatVersion 2» y esta «la cabeza no era available:true» (medido).
    r = doc.get("rama")
    if not isinstance(r, dict):
        raise Rojo("rama: no es una cabeza (objeto)")
    try:
        fv = u64_de(r, "formatVersion")
    except Rojo as e:
        raise Rojo(f"rama: {e}")
    if fv not in (3, 4, 5, 6):
        raise Rojo(msg_v("rama", fv))
    rama = leer_cabeza_de(doc, "rama", (3, 4, 5, 6), msg_v)
    r = misma_continuidad(doc, (cab["pk"], cab["embebido"]), (rama["pk"], rama["embebido"]))
    operador = ("misma publicKey: el mismo firmante" if r is None else
                f"claves distintas que las actas unen ({r[2]} eslabon(es), la posterior desde la hoja {r[0]}): el mismo "
                f"OPERADOR")
    salida.append(f"2/3 la rama recompone su digest y su firma verifica (indice embebido {rama['embebido']}); {operador}")
    t, t_r = cab["mmrSize"], rama["mmrSize"]
    if t_r != t:
        raise Rojo(f"bifurcacion: mmrSize distinto ({t}, {t_r}) - dos cabezas en sitios distintos de la historia no la "
                   f"bifurcan: el orden entre ellas lo juzga la extension")
    if rama["digest"] == cab["digest"]:
        raise Rojo(f"bifurcacion: las dos cabezas son LA MISMA (mmrSize {t} y el mismo digest): no hay bifurcacion")
    donde = ("la MISMA cima: las dos ramas divergen en esta cabeza" if rama["mmrRoot"] == cab["mmrRoot"] else
             "OTRA cima: las dos ramas divergen antes de esta cabeza")
    tambien = (f" (y la misma hoja {cab['embebido']}: tambien es la vista dividida del modo 4)"
               if rama["embebido"] == cab["embebido"] else "")
    salida.append(f"3/3 el mismo mmrSize {t} y digests DISTINTOS, con {donde}{tambien}")
    salida.append(f"VERDE: BIFURCACION - el operador firmo DOS cabezas distintas con el mmrSize {t}: bifurcacion o "
                  f"reinicio sin diario. Es DETECCION del operador: un nodo con su diario no repite un mmrSize, y solo "
                  f"quien tiene la clave pudo firmarlas")
    return salida


def ancla(doc):
    salida = []
    an = doc.get("ancla")
    # ⚠️ §714: la FORMA y el modo los decide la PRESENCIA de la clave, como en la
    #    referencia (`p.get(k).is_some()`, verdadero tambien con null). Con `doc.get(k) is not None` un
    #    `"ancla": null` o un `"camino": null` pasaban por ausentes: con una rama, una contraria o sin
    #    ancla el kit daba ROJO y esta VERDE (medido; la contraria y el camino ya divergian antes del §714).
    tiene = lambda k: k in doc
    if tiene("contraria") and tiene("ancla"):
        raise Rojo("un sobre con contraria no lleva ancla: la vista dividida se demuestra con las dos cabezas solas")
    if tiene("camino") and not tiene("ancla"):
        raise Rojo("camino sin ancla: no hay nada que extender")
    # §714: la rama va sola con la cabeza, como la contraria; despues de las dos de arriba.
    if tiene("rama") and (tiene("ancla") or tiene("contraria")):
        raise Rojo("un sobre con rama no lleva ancla ni contraria: la bifurcacion se demuestra con las dos cabezas solas")
    if doc.get("cabeza") is None:
        raise Rojo("falta cabeza (la firmada que el ancla compromete)")
    msg_v = lambda s, fv: f"{s}: formatVersion {fv}: el ancla lee cabezas v3, v4, v5 o v6: la pareja del MMR viaja firmada desde ellas"
    cab = leer_cabeza_de(doc, "cabeza", (3, 4, 5, 6), msg_v)
    emb = cab["embebido"]
    salida.append(f"1/3 la cabeza (v{cab['v']}) recompone su digest y su firma verifica (indice embebido {emb})")
    if tiene("rama"):
        return bifurcacion(doc, cab, salida)
    huella = N.huella_de_clave(cab["pk"])
    hd = lambda indice, dig, root, size: N.ancla_digest(huella_como_digest(huella), indice, dig, root, size)
    if tiene("contraria"):
        otra = leer_cabeza_de(doc, "contraria", (3, 4, 5, 6), msg_v)
        if otra["pk"] != cab["pk"]:
            raise Rojo("las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante")
        if otra["embebido"] != emb:
            raise Rojo(f"los indices embebidos son DISTINTOS ({emb}, {otra['embebido']}): dos firmas con su indice propio no "
                       f"dividen la vista")
        if otra["digest"] == cab["digest"] and otra["v"] == cab["v"]:
            raise Rojo("las dos cabezas son LA MISMA: no hay vista que dividir")
        salida.append(f"2/3 la contraria recompone y su firma verifica: misma clave, mismo indice embebido {emb}")
        salida.append("3/3 los contenidos DIFIEREN: dos preambulos bajo un indice de un solo uso")
        salida.append(f"VERDE: VISTA DIVIDIDA - la clave firmo DOS cabezas con el indice embebido {emb}. Es DETECCION del "
                      f"operador: dos historias, y solo quien tiene la clave pudo producirlas")
        return salida
    if not tiene("ancla"):
        derivada = hd(emb, cab["digest"], cab["mmrRoot"], cab["mmrSize"])
        salida.append("2/3 el ancla, derivada de la cabeza sola: lo que se publica en el medio")
        salida.append('   { "v": 1, "clave": "0x%s", "indice": "0x%x", "epochDigest": "0x%s", "mmrRoot": "0x%s", "mmrSize": "0x%x" }'
                      % (huella.hex(), emb, N.digest_to_bytes(cab["digest"]).hex(), N.digest_to_bytes(cab["mmrRoot"]).hex(),
                         cab["mmrSize"]))
        salida.append(f"3/3 la huella del ancla: 0x{N.digest_to_bytes(derivada).hex()} - comparala con el medio: este mando no "
                      f"tiene red")
        salida.append("VERDE: el ancla se deriva de la cabeza firmada, y se sostiene sin el nodo")
        return salida
    if not isinstance(an, dict):
        raise Rojo("el ancla no es un objeto")
    if isinstance(an.get("v"), bool) or an.get("v") != 1:
        raise Rojo("el ancla no declara v 1: este binario lee ancla v1")
    clave = bytes_de(an, "clave")
    indice = u64_de(an, "indice")
    dig = digest_de(an, "epochDigest")
    root = digest_de(an, "mmrRoot")
    size = u64_de(an, "mmrSize")
    if clave != huella:
        raise Rojo("el ancla es de OTRA clave: su clave no es la huella de la publicKey de la cabeza")
    if not tiene("camino"):
        for campo, a, b in (("indice", indice, emb), ("epochDigest", dig, cab["digest"]), ("mmrRoot", root, cab["mmrRoot"]),
                            ("mmrSize", size, cab["mmrSize"])):
            if a != b:
                raise Rojo(f"el ancla no ES esta cabeza: su {campo} no casa")
        salida.append("2/3 los cinco campos del ancla son los de la cabeza firmada")
        salida.append(f"3/3 la huella del ancla: 0x{N.digest_to_bytes(hd(indice, dig, root, size)).hex()}")
        salida.append("VERDE: el ancla ES esta cabeza firmada, y se sostiene sin el nodo")
        return salida
    if size == 0:
        raise Rojo("el ancla del genesis (mmrSize 0) no tiene historia que extender: se compara entera, sin camino")
    if indice >= emb:
        raise Rojo(f"el ancla declara un indice ({indice}) que no es ANTERIOR al embebido de la cabeza ({emb})")
    camino = leer_lista_de_digests(doc, "camino")
    if not consistencia(size, cab["mmrSize"], root, cab["mmrRoot"], camino):
        raise Rojo(f"la cabeza (t={cab['mmrSize']}) NO extiende el ancla (t={size}): historia bifurcada, recortada, o camino "
                   f"que no es el suyo")
    salida.append("2/3 la cima de la cabeza EXTIENDE el lote anclado: consistencia O(log N), sin el registro")
    # §712: el reloj de la extension, como la referencia. Con el mismo
    # mmrSize la consistencia es la identidad: el ancla de una rama contra la cabeza de la otra salia VERDE.
    lado_ancla = {"mmrSize": size, "digest": dig, "pk": cab["pk"], "embebido": indice}
    salida.append(el_reloj_avanza(lado_ancla, cab, ("el ancla", "la cabeza")))
    salida.append(f"3/3 la huella del ancla: 0x{N.digest_to_bytes(hd(indice, dig, root, size)).hex()}")
    salida.append("VERDE: la cabeza extiende el ancla: la historia anclada es un prefijo, y se sostiene sin el nodo")
    return salida


def _sha256_del_medio(x, campo):
    """Un hash SHA-256 del medio: `0x` y 64 hex, los bytes TAL CUAL. No es un digest de Goldilocks: aqui
    no hay campo, y un limbo fuera de p no es motivo de nada."""
    try:
        b = hex_a_bytes(x if isinstance(x, str) else "")
    except Rojo as e:
        raise Rojo(f"{campo}: {e}")
    if len(b) != 32:
        raise Rojo(f"{campo}: {len(b)} bytes, se esperaban 32")
    return b


def ancla_cofirmada(doc):
    """PAQUETE.md 2.13 (RFC-0013 E4a). El ancla NO viaja: se deriva de la cabeza, y por eso es la cabeza
    firmada que dice ser. El orden: la FORMA; la cabeza, entera (el `origin` de la nota sale de su
    clave); la nota y la linea del publicador; las cofirmas con vkey en el sobre; la inclusion.
    REPORTA, NO JUZGA: que testigos valen y cuantos hacen falta lo decide quien verifica (D-D)."""
    salida = []
    for ajeno in ("ancla", "camino", "contraria"):
        if ajeno in doc:
            raise Rojo(f"el sobre del ancla cofirmada no lleva {ajeno}: el ancla se deriva de la cabeza, y la "
                       f"extension y la vista dividida son del sobre del ancla")
    if "cabeza" not in doc:
        raise Rojo("falta cabeza (la firmada cuya ancla se publico)")
    c = doc["cabeza"] if isinstance(doc["cabeza"], dict) else {}
    nota = doc.get("nota")
    if not isinstance(nota, str):
        raise Rojo("falta nota o no es cadena (la nota checkpoint del medio, entera)")
    vkey_publicador = doc.get("publicador")
    if not isinstance(vkey_publicador, str):
        raise Rojo("falta publicador o no es cadena (la vkey del publicador del medio)")
    posicion = u64_de(doc, "posicion")
    inclusion = doc.get("inclusion")
    if not isinstance(inclusion, list):
        raise Rojo("falta inclusion o no es lista (el camino del ancla a la raiz)")
    inclusion = [_sha256_del_medio(x, "inclusion") for x in inclusion]
    testigos = []
    if "testigos" in doc:
        if not isinstance(doc["testigos"], list):
            raise Rojo("testigos no es lista (las vkeys de los testigos)")
        for x in doc["testigos"]:
            try:
                testigos.append(medio.ClaveDeNota.leer_vkey(x if isinstance(x, str) else ""))
            except medio.ErrorDeNota as e:
                raise Rojo(f"testigos: {e}")
    for i, a in enumerate(testigos):
        if any(b.nombre == a.nombre and b.id == a.id for b in testigos[:i]):
            raise Rojo(f"testigos: {a.nombre} esta dos veces")

    # 1/4 la cabeza, ENTERA, como en el sobre del ancla; la version, antes que nada de ella.
    version = u64_de(c, "formatVersion")
    if version not in (3, 4, 5, 6):
        raise Rojo(f"cabeza: formatVersion {version}: el ancla lee cabezas v3, v4, v5 o v6: la pareja del MMR "
                   f"viaja firmada desde ellas")
    cab = leer_cabeza(c, "cabeza")
    embebido = verificar_cabeza(cab, "cabeza")
    huella_clave = N.huella_de_clave(cab["pk"])
    huella_ancla = N.digest_to_bytes(N.ancla_digest(huella_como_digest(huella_clave), embebido, cab["digest"],
                                                    cab["mmrRoot"], cab["mmrSize"]))
    salida.append(f"1/4 la cabeza (v{version}) recompone su digest y su firma verifica (indice embebido {embebido}); "
                  f"su ancla: 0x{huella_ancla.hex()}")

    # 2/4 la nota: del medio de ESTA clave (D-A), y firmada por su publicador.
    origen = medio.origen_del_medio(huella_clave)
    try:
        publicador = medio.ClaveDeNota.leer_vkey(vkey_publicador)
    except medio.ErrorDeNota as e:
        raise Rojo(f"publicador: {e}")
    if publicador.nombre != origen:
        raise Rojo(f"el publicador es {medio.debug_rust(publicador.nombre)} y el medio de esta cabeza es "
                   f"{medio.debug_rust(origen)}: el origin lleva la huella de la clave XMSS (RFC-0013 D-A)")
    try:
        v = medio.verificar_nota(nota, publicador)
    except medio.ErrorDeNota as e:
        raise Rojo(f"la nota: {e}")
    salida.append(f"2/4 la nota del medio verifica con su publicador (ML-DSA-44, tipo 0x06): {v.checkpoint.tamano} "
                  f"anclas, marca {v.marca}")

    # 3/4 las cofirmas: las que tienen clave en el sobre se verifican; las demas se cuentan.
    cofirman, sin_clave = [], 0
    for linea in v.ajenas:
        k = next((t for t in testigos if t.nombre == linea.nombre and t.id == linea.id), None)
        if k is None:
            sin_clave += 1
            continue
        if any(o is k for o, _ in cofirman):
            raise Rojo(f"la nota lleva dos cofirmas de {k.nombre}")
        try:
            cofirman.append((k, k.verificar_cofirma(v.checkpoint, linea)))
        except medio.ErrorDeNota as e:
            raise Rojo(f"la cofirma de {k.nombre} no verifica: {e}")
    salida.append(f"3/4 cofirmas que verifican con las claves del sobre: {len(cofirman)}; lineas sin clave en el "
                  f"sobre, sin juzgar: {sin_clave}; testigos del sobre sin cofirma: {len(testigos) - len(cofirman)}")
    for k, marca in cofirman:
        salida.append(f"   testigo {k.nombre} clave sha256:{medio.sha256(k.bytes).hex()} marca {marca}")

    # 4/4 la inclusion: la hoja SHA-256(0x00 || huella del ancla) sube a la raiz de la nota, donde dice.
    tamano = v.checkpoint.tamano
    try:
        medio.verificar_inclusion(medio.hoja(huella_ancla), posicion, tamano, inclusion, v.checkpoint.raiz)
    except medio.ErrorDelMedio as e:
        raise Rojo(f"el ancla de esta cabeza no esta en la posicion {posicion} de las {tamano} de la nota: {e}")
    salida.append(f"4/4 el ancla esta en la posicion {posicion} de las {tamano} anclas de la nota")
    salida.append(f"VERDE: la cabeza estaba publicada en el medio {origen}, en la posicion {posicion}, y la cofirman "
                  f"{len(cofirman)} testigo(s) con clave en el sobre. Que testigos valen y cuantos hacen falta lo "
                  f"decide quien verifica (RFC-0013 D-D): este mando reporta, no juzga")
    return salida


# §852 · Los numeros que serde_json no lee y `json.loads` si. El kit lee el sobre con serde_json
# (1.0.151, el del Cargo.lock, sin `float_roundtrip`): `NaN`, `Infinity` y `-Infinity` no son JSON, y
# un numero que no cabe en un f64 es `number out of range`. `json.loads` acepta las tres constantes y
# lee `1e400` como infinito y un entero de 401 cifras como un int de Python; en un campo que este
# verificador no lee, el sobre salia VERDE. Ahora los tres son `JSON ilegible`. La regla del rango es
# la de `_fuera_de_rango` en cable.py, copiada y no importada: cable.py importa juez_cabezas.py, y el
# verificador no carga un juez. Como alli, no es el redondeo correcto: compone el f64 como serde_json,
# y `1.7976931348623158e308`, que `float()` lee finito, aqui no se lee (medido contra el kit, §852).
U64 = (1 << 64) - 1
I32 = (1 << 31) - 1
POW10 = tuple(float(f"1e{i}") for i in range(309))  # la tabla 1e0..1e308 del lector
_NUMERO = re.compile(r"-?(\d+)(?:\.(\d+))?(?:[eE]([+-]?)(\d+))?")


def _fuera_de_rango(literal):
    """Si serde_json da `number out of range`: la mantisa en un u64 -las cifras que ya no caben se
    dejan caer y, si son de la parte entera, suben el exponente-, el exponente en un i32 y una sola
    potencia de 10 de la tabla; y el numero no se lee si sale infinito."""
    entera, fraccion, signo, exp = _NUMERO.fullmatch(literal).groups()
    mant, e, lleno = 0, 0, False
    for c in entera:
        if lleno or mant * 10 + int(c) > U64:
            lleno, e = True, e + 1
        else:
            mant = mant * 10 + int(c)
    for c in fraccion or "":
        if mant * 10 + int(c) > U64:
            break
        mant, e = mant * 10 + int(c), e - 1
    if exp is not None:
        cifras = exp.lstrip("0") or "0"
        if len(cifras) > 10 or int(cifras) > I32:  # el exponente no cabe en un i32
            return mant != 0 and signo != "-"
        e = max(-I32 - 1, min(I32, e - int(cifras) if signo == "-" else e + int(cifras)))
    f = float(mant)
    while True:
        if -308 <= e <= 308:
            return e >= 0 and f * POW10[e] == float("inf")
        if f == 0.0:
            return False
        if e >= 0:
            return True
        f, e = f / 1e308, e + 308


def _sin_constantes(nombre):
    raise ValueError(f"{nombre} no es JSON")


def _real(literal):
    if _fuera_de_rango(literal):
        raise ValueError(f"numero fuera del rango de un f64: {literal[:40]}")
    return float(literal)


def _entero(literal):
    if _fuera_de_rango(literal):
        raise ValueError(f"numero fuera del rango de un f64: {literal[:40]}")
    return int(literal)


def juzgar(ruta):
    try:
        with open(ruta, "rb") as f:
            crudo = f.read()
    except OSError as e:
        raise Rojo(f"no se puede leer {ruta}: {e.strerror} (os error {e.errno})")
    try:
        doc = json.loads(crudo.decode("utf-8"), parse_constant=_sin_constantes, parse_float=_real, parse_int=_entero)
        # §810: una suplente sola (`\ud800`), que json.loads deja pasar y serde_json no: JSON ilegible, como el binario
        json.dumps(doc, ensure_ascii=False).encode("utf-8")
    except (ValueError, UnicodeDecodeError) as e:
        raise Rojo(f"JSON ilegible: {e}")
    if not isinstance(doc, dict):
        raise Rojo("JSON ilegible: el sobre no es un objeto")
    v = doc.get("v")
    if isinstance(v, bool) or not isinstance(v, int):
        raise Rojo("el paquete no declara su version en `v`")
    if v not in (1, 2):
        raise Rojo(f"el paquete declara v:{v} — este binario lee v1 y v2")
    tipo = doc.get("tipo")
    if tipo is not None:
        if tipo == "extension":
            return extension(doc)
        if tipo == "consumo":
            return consumo(doc)
        if tipo == "conflicto":
            return conflicto(doc)
        if tipo == "ancla":
            return ancla(doc)
        if tipo == "ancla-cofirmada":
            return ancla_cofirmada(doc)
        if tipo == "solapamiento":
            return solapamiento(doc)
        if tipo in NO_SOPORTADOS:
            raise Rojo(f"NO SOPORTADO: tipo {tipo}: la segunda implementacion no lee este sobre (NO_SOPORTADOS, "
                       f"tools/segunda/README.md)")
        # §634: el texto del binario de referencia, letra por letra (PAQUETE.md, seccion 5); hasta
        # aqui acababa en `prenda`, rancio como el del catalogo desde el §573.
        raise Rojo(f"tipo desconocido: {tipo} - se lee un paquete de posicion (sin `tipo`), `tipo: \"extension\"`, "
                   f"`tipo: \"consumo\"`, `tipo: \"conflicto\"`, `tipo: \"rechazo\"`, `tipo: \"edad\"`, "
                   f"`tipo: \"cobro_pendiente\"`, `tipo: \"pago_en_curso\"`, `tipo: \"prenda\"`, "
                   f"`tipo: \"completitud\"`, `tipo: \"ancla\"`, `tipo: \"ancla-cofirmada\"` o "
                   f"`tipo: \"solapamiento\"`")
    if v == 1 and "cofirmas" in doc:
        raise Rojo("un paquete v1 con `cofirmas`: subir la version es lo que las hace parte del contrato — declaralo v2, "
                   "o quitalas")
    return posicion(doc, v)


def main(argv):
    if len(argv) != 2:
        print("uso: verificador.py <paquete.json>\n     (la segunda implementacion del mando: spec/PAQUETE.md)", file=sys.stderr)
        return 2
    try:
        for linea in juzgar(argv[1]):
            print(linea)
        return 0
    except Rojo as e:
        print(f"ROJO: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
