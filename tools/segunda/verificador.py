#!/usr/bin/env python3
"""La SEGUNDA implementacion del mando (BACKLOG 85, tercer hito): un verificador del paquete de
evidencia escrito desde spec/PAQUETE.md, en Python y sin dependencias, con el contrato de salida del
mando (seccion 6): un argumento, la ruta de un sobre; las lineas numeradas de su forma por la salida
estandar y el VERDE de su forma; `ROJO: {motivo}` por la de error con un texto del catalogo de la
seccion 5; exit 0 verde, 1 el primer fallo con nombre, 2 uso.

Lo que lee hoy: las cinco formas que no exigen una prueba STARK -POSICION (v1 y v2, con acuse y
cofirmas), EXTENSION, CONSUMO, CONFLICTO y ANCLA-, y desde el §649 las `actas` que unen dos claves de
un operador en la extension y el consumo (RFC-0015); las que exigen una prueba quedan fuera. Un `tipo` conocido que esta implementacion no lee todavia
sale ROJO con su nombre, nunca VERDE.

De donde sale: PAQUETE.md secciones 2.1 a 2.3, 3, 4, 5 y 6; NUCLEO.md seccion 6 a traves de
nucleo.py; RFC 8391 a traves de kat_xmss/xmss.py; y la consistencia del MMR de RFC 6962 (RFC 9162, 2.1.4.2)
con mmr_nodo como nodo. Lo juzga tools/conformidad.sh con los manifiestos de spec/vectors/.
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "kat_xmss"))  # xmss.py vive ahi
import nucleo as N  # noqa: E402
import xmss  # noqa: E402

VERSIONES_CABEZA = (2, 3, 4, 5, 6)
VERSIONES_EXTENSION = (3, 4, 5, 6)
COFIRMA_V_MAX = 1
TIPOS_CONOCIDOS = ("extension", "consumo", "conflicto", "rechazo", "edad", "cobro_pendiente", "pago_en_curso",
                   "prenda", "completitud", "ancla", "ancla-cofirmada", "solapamiento")


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
    try:
        v = int(s[2:], 16)
    except ValueError:
        raise Rojo(f"{campo}: invalid digit found in string")
    if s[2:] == "" or v >= 2**64:
        raise Rojo(f"{campo}: number too large to fit in target type")
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
    if mensaje != N.preambulo(cab["v"], cab["digest"]):
        raise Rojo(p + "el preambulo recuperado no es el esperado")
    if not xmss.verificar(cab["pk"], mensaje, firma):
        raise Rojo(p + "la firma XMSS no verifica")
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
        raise Rojo("acuse: 'la hoja no sube hasta la raiz firmada'")
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
        try:
            oid, _, _ = xmss.parsear_clave(clave_t)
        except ValueError as e:
            raise Rojo(f"cofirma {i}: {e}")
        largo = xmss.largo_firma(oid)
        firma, mensaje = firmado[:largo], firmado[largo:]
        if mensaje != N.preambulo_cofirma(version, digest, clave_op):
            raise Rojo(f"cofirma {i}: el preambulo recuperado no es el esperado")
        if not xmss.verificar(clave_t, mensaje, firma):
            raise Rojo(f"cofirma {i}: la firma del testigo no verifica")
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
    """Un Q en su escritura minima (zk_ssl_hash::cantidad_canonica, §662)."""
    s = _texto_de(obj, campo)
    if not s.startswith("0x"):
        raise Fallo(f"{campo}: hex sin 0x")
    h = s[2:].encode("utf-8")
    if not h or len(h) > 16 or (len(h) > 1 and h[:1] == b"0"):
        raise Fallo(f"{campo}: cantidad hex no minima")
    malo = next((i for i, c in enumerate(h) if c not in b"0123456789abcdef"), None)
    if malo is not None:
        raise Fallo(f"{campo}: hex: cifra no admitida en la posicion {malo}")
    return int(h, 16)


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


def juzgar_rotacion(actas, de, a, ultimo):
    """La cadena entera vale, `a` esta en ella y `de` antes, y lo visto de `de` queda por debajo del
    `desde` de su sucesora. Devuelve (desde, hasta, eslabones) del tramo de `a`."""
    for i, x in enumerate(actas):
        try:
            verificar_acta(x, actas[i - 1] if i else None)
        except Fallo as e:
            raise Fallo(f"el acta {i} de la cadena no vale: {e}")
    j = next((k for k in range(len(actas) - 1, -1, -1) if actas[k]["clave"] == a), None)
    if j is None:
        raise Fallo("la clave que llega no esta en la cadena: nadie la comprometio")
    i = next((k for k in range(j - 1, -1, -1) if actas[k]["clave"] == de), None)
    if i is None:
        raise Fallo("la clave que se tenia no esta en la cadena antes de la que llega")
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
            raise Rojo(f"la cabeza firma en la hoja {e}, dentro de su tramo {_texto_de_tramo(t)}: no hay solapamiento")
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
    r = misma_continuidad(doc, (vieja["pk"], vieja["embebido"]), (nueva["pk"], nueva["embebido"]))
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
    salida.append("VERDE: la extension se sostiene sin el nodo")
    return salida


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
    r = misma_continuidad(doc, (vieja["pk"], vieja["embebido"]), (nueva["pk"], nueva["embebido"]))
    salida.append(linea_de_continuidad("2/5", r, ", y las dos cabezas llevan consRoot (v4, v5 o v6) a los dos lados")
                  or "2/5 misma publicKey y las dos cabezas llevan consRoot (v4, v5 o v6) a los dos lados")
    camino = leer_lista_de_digests(doc, "camino")
    if not consistencia(vieja["mmrSize"], nueva["mmrSize"], vieja["mmrRoot"], nueva["mmrRoot"], camino):
        raise Rojo(f"la nueva (t={nueva['mmrSize']}) NO extiende a la vieja (t={vieja['mmrSize']}): historia bifurcada, "
                   f"recortada, o camino que no es el suyo")
    salida.append("3/5 la cima nueva EXTIENDE a la vieja: el «antes» es de esta historia")
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


def ancla(doc):
    salida = []
    an, cam, contraria = doc.get("ancla"), doc.get("camino"), doc.get("contraria")
    if contraria is not None and an is not None:
        raise Rojo("un sobre con contraria no lleva ancla: la vista dividida se demuestra con las dos cabezas solas")
    if cam is not None and an is None:
        raise Rojo("camino sin ancla: no hay nada que extender")
    if doc.get("cabeza") is None:
        raise Rojo("falta cabeza (la firmada que el ancla compromete)")
    msg_v = lambda s, fv: f"{s}: formatVersion {fv}: el ancla lee cabezas v3, v4, v5 o v6: la pareja del MMR viaja firmada desde ellas"
    cab = leer_cabeza_de(doc, "cabeza", (3, 4, 5, 6), msg_v)
    emb = cab["embebido"]
    salida.append(f"1/3 la cabeza (v{cab['v']}) recompone su digest y su firma verifica (indice embebido {emb})")
    huella = N.huella_de_clave(cab["pk"])
    hd = lambda indice, dig, root, size: N.ancla_digest(huella_como_digest(huella), indice, dig, root, size)
    if contraria is not None:
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
    if an is None:
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
    if cam is None:
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
    salida.append(f"3/3 la huella del ancla: 0x{N.digest_to_bytes(hd(indice, dig, root, size)).hex()}")
    salida.append("VERDE: la cabeza extiende el ancla: la historia anclada es un prefijo, y se sostiene sin el nodo")
    return salida


def juzgar(ruta):
    try:
        with open(ruta, "rb") as f:
            crudo = f.read()
    except OSError as e:
        raise Rojo(f"no se puede leer {ruta}: {e.strerror} (os error {e.errno})")
    try:
        doc = json.loads(crudo.decode("utf-8"))
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
        if tipo == "solapamiento":
            return solapamiento(doc)
        if tipo in TIPOS_CONOCIDOS:
            raise Rojo(f"tipo {tipo}: la segunda implementacion no lee este sobre todavia")
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
