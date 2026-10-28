#!/usr/bin/env python3
"""El cable en Python, sin dependencias y sin una linea de Rust: dos piezas en un fichero, cada una
con su juez, y ninguna es nucleo.

    python3 tools/segunda/cable.py <respuesta.json>
    bash tools/conformidad.sh tools/segunda/cable.py spec/vectors/cable/MANIFIESTO.txt
    python3 tools/segunda/juez_cadena.py spec/vectors/zkssl-0.*.json

⚠️ §809: las dos se escribieron en dos sellos con el mismo nombre de fichero -la cadena en el §797,
el consumidor en el §809- y se juntaron aqui: no comparten ningun nombre, la cadena no lee nada del
consumidor ni el consumidor de la cadena, y `main` es solo del consumidor. `juez_cadena.py` importa
el modulo entero (`import cable as C`) y usa la parte I.

I. LA CADENA DEL REGISTRO (§797)

La cadena del REGISTRO de transiciones, como la sirve el cable (spec/RPC.md, <<La composicion de
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

II. EL SEGUNDO CONSUMIDOR DEL CABLE (§809)

El SEGUNDO CONSUMIDOR del cable (§809; RFC-0005, E3 y E5; BACKLOG 85): juzga una respuesta de
`zkssl_signedEpochHead` desde fichero y SIN nodo, como `zk-ssl-cli witness --respuesta` con la memoria
fresca, en Python y sin una linea de Rust.

La respuesta es el CUERPO JSON-RPC tal como cruza el cable. Pasa por el camino que spec/RPC.md da al
testigo («Los rechazos del cable»: `result`, el tipado del cable, el ancla de la clave, la firma, la
recomposicion), y lo que queda es lo que el juez de las cabezas de la segunda consume: la cabeza, que
se parte con `juez_cabezas.partir`, se verifica con kat_xmss/xmss.py (RFC 8391) y se recompone con
`juez_cabezas.recomponer` (NUCLEO.md, seccion 6, a traves de nucleo.py).

Contrato (RPC.md, «Los rechazos del cable»): exit 0 y `VERDE: nueva` solo si la clase es `nueva`; si
no, UNA linea `ROJO: <clase> · <texto>` y exit 1; uso, 2. Con la memoria fresca no hay clave fijada ni
pareja custodiada: la clave se fija al verla (TOFU) y el segundo canal, la consistencia, no se juzga.

De donde sale: los tipos de RPC.md («Codificacion» y «`zkssl_signedEpochHead`»), los textos de
spec/vectors/cable/MANIFIESTO.txt y, para lo que ninguno de los dos escribe -que claves se exigen y
cuales sobran, en que orden se miran y con que texto se rechaza lo que el manifiesto no pina-, la
conducta MEDIDA del testigo de referencia sobre respuestas reales y manipuladas de un nodo, como caja
negra: el diferencial de tools/segunda/README.md, «El segundo consumidor del cable».
"""
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "kat_xmss"))  # xmss.py vive ahi
import juez_cabezas as J  # noqa: E402
import nucleo as N  # noqa: E402
import xmss  # noqa: E402


# ═══ I · la cadena del registro (§797) ════════════════════════════════════════════════════════════

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


# ═══ II · el segundo consumidor del cable (§809) ══════════════════════════════════════════════════

# Las versiones de cabeza que se recomponen (RPC.md, formatos v3 a v6; la v2 sigue verificando).
VERSIONES = (2, 3, 4, 5, 6)
# La unica familia de clave que la casa firma: XMSSMT-SHA2_40/8_256 (kat_xmss/xmss.py).
OID_CASA = 0x00000005

# El tipado del cable: las treinta claves que una respuesta de signedEpochHead puede llevar, y de que
# tipo es cada una (RPC.md, «Codificacion»): Q es QUANTITY, D un Digest, B DATA, s una cadena y b un
# booleano. Una clave que no esta aqui rompe la lectura en voz alta, como en la referencia («aditivo
# en el cable, rotura en el parser»: RPC.md, formato v3).
TIPOS = {
    "available": "b", "beatSeconds": "Q", "custody": "s", "custodyChecked": "b", "reason": "s", "seq": "Q",
    "epochDigest": "D", "emittedAtUnix": "Q", "domain": "s", "formatVersion": "Q", "mmrRoot": "D",
    "mmrSize": "Q", "consRoot": "D", "consCount": "Q", "paramsDigest": "D", "pmetaRoot": "D",
    "nextPending": "Q", "nextIndex": "Q", "totalSupply": "Q", "recepRoot": "D", "recepCount": "Q",
    "index": "Q", "accountsRoot": "D", "pendingRoot": "D", "frozenRoot": "D", "chainDigest": "D",
    "acusesRoot": "D", "n": "Q", "signature": "B", "publicKey": "B",
}
# Las que viajan en las TRES respuestas (RPC.md: `custody` y `custodyChecked` «siempre», y `beatSeconds`),
# en ese orden; el resto puede faltar, o venir a `null`, y lo que falta se dice despues, con su nombre.
OBLIGADAS = ("beatSeconds", "custody", "custodyChecked")
# Lo que una cabeza FIRMADA tiene que traer: primero lo que su version anade desde la v4 (si la
# version es una de las que se recomponen), despues lo comun, en el orden en que se mira.
POR_VERSION = (
    (4, ("consRoot", "consCount")),
    (5, ("paramsDigest", "pmetaRoot", "nextPending", "nextIndex", "totalSupply")),
    (6, ("recepRoot", "recepCount")),
)
COMUNES = ("seq", "epochDigest", "emittedAtUnix", "domain", "formatVersion", "mmrRoot", "mmrSize", "index",
           "accountsRoot", "pendingRoot", "frozenRoot", "chainDigest", "acusesRoot", "n", "signature", "publicKey")
# Lo que compone la v1 (NUCLEO.md, seccion 6: `epoch_digest`), y el orden en que la recomposicion lee
# los campos que compone: los contadores de la v1 y la v2 delante.
CAMPOS_V1 = ("seq", "accountsRoot", "pendingRoot", "frozenRoot", "chainDigest")
LECTURA = ("seq", "n", "accountsRoot", "pendingRoot", "frozenRoot", "chainDigest", "acusesRoot", "mmrRoot",
           "mmrSize", "consRoot", "consCount", "paramsDigest", "pmetaRoot", "nextPending", "nextIndex",
           "totalSupply", "recepRoot", "recepCount")
assert sorted(k for _, ks in POR_VERSION for k in ks) == sorted(k for v, ks in J.CAMPOS_V.items() if v >= 4 for k in ks)
assert sorted(LECTURA) == sorted(list(CAMPOS_V1) + [k for ks in J.CAMPOS_V.values() for k in ks]), "un campo sin leer"
assert set(COMUNES) | {k for _, ks in POR_VERSION for k in ks} | set(OBLIGADAS) | {"available", "reason"} == set(TIPOS)


class Rojo(Exception):
    """El veredicto que no es `nueva`, con su clase y su texto."""

    def __init__(self, clase, texto):
        super().__init__(f"{clase} · {texto}")


def no_verifica(texto):
    return Rojo("no-verifica", texto)


def forma(texto):
    return no_verifica(f"forma del cable: {texto}")


# ─── el tipado del cable (RPC.md, «Codificacion»): solo la escritura, no el valor ──────────────

def _hex(s):
    return all(c in "0123456789abcdef" for c in s)


def tipar(campo, valor):
    tipo = TIPOS[campo]
    if tipo == "b":
        if not isinstance(valor, bool):
            raise forma(f"`{campo}` no es un booleano")
        return
    if not isinstance(valor, str):
        raise forma(f"`{campo}` no es una cadena")
    if tipo == "s":
        return
    if tipo == "Q":
        # QUANTITY: 0x y la escritura minima de un u64, en minusculas (RFC-0016 y el §662).
        if not valor.startswith("0x"):
            raise forma(f"QUANTITY sin 0x en `{campo}`")
        h = valor[2:]
        if not h or len(h) > 16 or not _hex(h) or (len(h) > 1 and h[0] == "0"):
            raise forma(f"QUANTITY inválida en `{campo}`")
        return
    h = valor[2:]
    if not valor.startswith("0x") or len(h) % 2 or not _hex(h):
        raise forma(f"hex mal formado (se espera \"0x…\") en `{campo}`")
    if tipo == "D" and len(h) != 64:
        raise forma(f"longitud {len(h) // 2}, se esperaban 32 bytes en `{campo}`")


# ─── los valores, al leerlos: canonicos (RFC-0016, NUCLEO.md seccion 6, «Canonicidad») ─────────

def q(c, campo):
    u = int(c[campo], 16)
    try:
        return N.u64_canonico(u)
    except ValueError as e:
        raise no_verifica(f"QUANTITY \"{c[campo]}\": {e} (`{campo}`)")


def d(c, campo):
    try:
        return N.digest_from_bytes(bytes.fromhex(c[campo][2:]))
    except ValueError:
        raise no_verifica(f"{campo}: no es un digest del campo")


def _sin_constantes(nombre):
    raise ValueError(f"{nombre} no es JSON")


# ─── los limites del lector: lo que el de la referencia no lee y el de Python si ─────────────
# RFC 8259 deja al lector poner limites (seccion 9: la profundidad del anidamiento y el rango de los
# numeros) y no dice que hacer con un sustituto UTF-16 suelto (seccion 8.2). El lector del testigo
# (serde_json 1.0.151, el del Cargo.lock, sin `float_roundtrip`) pone los tres sobre el CUERPO
# ENTERO, no solo sobre `result`, y lo que los pasa es `respuesta ilegible`; el `json` de Python no
# pone ninguno. Se imitan, medidos contra el testigo (tools/segunda/README.md, «El segundo
# consumidor del cable», punto 6): no son reglas del formato, y ningun vector los pina.
PROFUNDIDAD = 127  # niveles de `[` y `{` anidados que se leen; el 128 ya no
U64 = (1 << 64) - 1
I32 = (1 << 31) - 1
POW10 = tuple(float(f"1e{i}") for i in range(309))  # la tabla 1e0..1e308 del lector
_NUMERO = re.compile(r"-?(\d+)(?:\.(\d+))?(?:[eE]([+-]?)(\d+))?")
_TOKEN = re.compile(r'"[^"\\]*(?:\\.[^"\\]*)*"|[\[\]{}]', re.S)
_ESCAPE = re.compile(r"\\(?:u([0-9a-fA-F]{4})|.)", re.S)
SUELTO = "sustituto UTF-16 suelto en una cadena: no es texto Unicode"


def _fuera_de_rango(literal):
    """Si el lector de la referencia da `number out of range`: compone el f64 como el, con la mantisa
    en un u64 -las cifras que ya no caben se dejan caer y, si son de la parte entera, suben el
    exponente-, el exponente en un i32 y una sola potencia de 10 de la tabla; y el numero no se lee
    si sale infinito. No es el redondeo correcto: 1.7976931348623158e308 sale infinito aqui."""
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


def _numero(literal):
    if _fuera_de_rango(literal):
        raise ValueError(f"numero fuera del rango de un f64: {literal[:40]}")
    return float(literal) if any(c in literal for c in ".eE") or len(literal) > 20 else int(literal)


def _anidamiento_y_sustitutos(texto):
    """Sobre un texto que ya se leyo como JSON: los `[` y `{` fuera de las cadenas, y los escapes
    \\uXXXX de cada cadena -clave o valor, tambien los de una clave repetida que la lectura pisa-."""
    nivel = 0
    for t in _TOKEN.finditer(texto):
        s = t.group()
        if s in ("[", "{"):
            nivel += 1
            if nivel > PROFUNDIDAD:
                raise ValueError(f"mas de {PROFUNDIDAD} niveles de anidamiento")
        elif s in ("]", "}"):
            nivel -= 1
        elif "\\u" in s:
            bajo_en = None  # tras un sustituto alto, donde tiene que empezar el bajo que lo empareja
            for m in _ESCAPE.finditer(s):
                cp = int(m.group(1), 16) if m.group(1) else -1
                bajo = 0xDC00 <= cp <= 0xDFFF
                if bajo_en is not None:
                    if m.start() != bajo_en or not bajo:
                        raise ValueError(SUELTO)
                    bajo_en = None
                elif 0xD800 <= cp <= 0xDBFF:
                    bajo_en = m.end()
                elif bajo:
                    raise ValueError(SUELTO)
            if bajo_en is not None:
                raise ValueError(SUELTO)


# ─── el camino ────────────────────────────────────────────────────────────────────────────────

def juzgar(ruta):
    # 1 · el cuerpo, que tiene que leerse como JSON entero: sin BOM, sin cola, sin NaN, y dentro de
    #     los limites del lector de la referencia (arriba).
    try:
        with open(ruta, "rb") as f:
            crudo = f.read()
    except OSError as e:
        raise Rojo("respuesta ilegible", f"no se lee el fichero: {e.strerror}")
    try:
        texto = crudo.decode("utf-8")
        cuerpo = json.loads(texto, parse_constant=_sin_constantes, parse_int=_numero, parse_float=_numero)
        _anidamiento_y_sustitutos(texto)
    except RecursionError:
        raise Rojo("respuesta ilegible", f"mas de {PROFUNDIDAD} niveles de anidamiento")
    except (UnicodeDecodeError, ValueError) as e:
        raise Rojo("respuesta ilegible", str(e))
    # 2 · `result`; un `error` en su lugar es una respuesta sin cabeza, se diga lo que se diga en el.
    if not isinstance(cuerpo, dict) or "result" not in cuerpo:
        raise Rojo("sin-respuesta", "respuesta sin `result`")
    c = cuerpo["result"]
    # 3 · las dos respuestas sin firma de RPC.md (aun sin latido, o el nodo sin --clave): no son un
    #     fallo del cable, son la pieza que falta, y se dicen con el motivo que el nodo da. Solo un
    #     `available` que es JSON `true` lleva cabeza firmada.
    if not isinstance(c, dict) or c.get("available") is not True:
        motivo = c.get("reason") if isinstance(c, dict) else None
        raise Rojo("sin-firma", motivo if isinstance(motivo, str) else "sin motivo declarado")
    # 4 · el tipado: clave a clave, por orden de nombre; lo que sobra rompe, lo que falta de lo
    #     obligado se dice al final, y una clave opcional a `null` es una clave que no esta.
    for campo in sorted(c):
        if campo not in TIPOS:
            raise forma(f"clave desconocida `{campo}`: la respuesta firmada no la lleva")
        if c[campo] is None and campo not in OBLIGADAS:
            continue
        tipar(campo, c[campo])
    for campo in OBLIGADAS:
        if campo not in c:
            raise forma(f"falta `{campo}`")
    cab = {k: v for k, v in c.items() if v is not None}
    # 5 · lo que una cabeza firmada tiene que traer: lo de su version, y lo comun.
    v_escrita = int(cab["formatVersion"], 16) if "formatVersion" in cab else None
    for desde, campos in POR_VERSION:
        if v_escrita in VERSIONES and v_escrita >= desde:
            for campo in campos:
                if campo not in cab:
                    raise no_verifica(f"cabeza firmada sin {campo}: el productor la sirvio incompleta")
    for campo in COMUNES:
        if campo not in cab:
            raise no_verifica(f"cabeza firmada sin {campo}: el productor la sirvio incompleta")
    # 6 · la version, que elige recompositor: un conjunto EXPLICITO, y lo demas se rechaza (RFC-0005 E2).
    v = q(cab, "formatVersion")
    if v not in VERSIONES:
        raise no_verifica(f"formatVersion {v}: el testigo recompone cabezas v2, v3, v4, v5 o v6")
    declarado = q(cab, "index")
    # 6 bis · ⚠️ §862 · el esquema, antes que la clave (RFC-0020, E3): el de una cabeza v2 a v6 es el
    #     implicito, y este consumidor lo acepta; uno que no aceptara se rechazaria con su numero.
    try:
        entrada = N.entrada_aceptada(N.ESQUEMA_IMPLICITO)
    except ValueError as e:
        raise no_verifica(str(e))
    # 7 · el ancla de la clave: con la memoria fresca se fija la que llega (TOFU), si es una clave
    #     de la familia que la casa firma.
    pk = bytes.fromhex(cab["publicKey"][2:])
    if len(pk) < 4:
        raise no_verifica(f"clave publica ilegible: {len(pk)} bytes: no caben ni los 4 del OID")
    oid = int.from_bytes(pk[:4], "big")
    if oid != OID_CASA:
        raise no_verifica(f"clave publica ilegible: OID {oid:#010x}, y la casa firma XMSSMT-SHA2_40/8_256 "
                          f"(OID {OID_CASA:#010x})")
    try:
        xmss.parsear_clave(pk)
    except ValueError as e:
        raise no_verifica(f"clave publica ilegible: {e}")
    # 8 · la firma: la de RFC 8391 va delante y el mensaje firmado detras (juez_cabezas.partir), con la
    #     entrada de su esquema (⚠️ §862: hasta aqui, con el OID de la clave).
    firma, mensaje = J.partir(bytes.fromhex(cab["signature"][2:]), entrada)
    if len(firma) != xmss.largo_firma(oid) or not xmss.verificar(pk, mensaje, firma):
        raise no_verifica("la firma no verifica")
    # 9 · y lo que firma tiene que ser el preambulo de ESTA cabeza: dominio, version y digest servido.
    esperado = N.DOMINIO_EPOCH_HEAD + bytes([v]) + bytes.fromhex(cab["epochDigest"][2:])
    if mensaje != esperado:
        raise no_verifica(f"la firma es VALIDA pero de otro mensaje (preambulo esperado {len(esperado)} bytes, "
                          f"recibido {len(mensaje)}). Verificar sin comparar no prueba nada.")
    # 10 · el index declarado no entra en lo firmado: tiene que quedar por encima del embebido (§399).
    embebido = xmss.indice_embebido(oid, firma)
    if declarado <= embebido:
        raise no_verifica(f"el indice declarado ({declarado}) no cuadra con el que va dentro de la firma "
                          f"({embebido}): el declarado no entra en el preambulo, asi que la firma no lo acredita")
    # 11 · la recomposicion: NUNCA se cree el digest. Cada valor se lee canonico y en su orden, y el juez
    #      de las cabezas compone (juez_cabezas.recomponer, NUCLEO.md seccion 6).
    compuestos = set(CAMPOS_V1) | {k for ver, ks in J.CAMPOS_V.items() if ver <= v for k in ks}
    for campo in LECTURA:
        if campo in compuestos:
            (q if TIPOS[campo] == "Q" else d)(cab, campo)
    _, recompuesto = J.recomponer(cab)
    if N.digest_to_bytes(recompuesto) != bytes.fromhex(cab["epochDigest"][2:]):
        raise no_verifica(f"los campos NO recomponen el epochDigest firmado (v{v}): o el servido esta adulterado "
                          f"o la cabeza nunca fue esa")
    return [f"cabeza v{v}, seq {int(cab['seq'], 16)}, indice embebido {embebido}; clave fijada al verla (TOFU)",
            f"VERDE: nueva · indice {cab['index']} · {cab['epochDigest']}"]


def main(argv):
    if len(argv) != 2:
        print("uso: python3 tools/segunda/cable.py <respuesta.json>\n"
              "     (el segundo consumidor de zkssl_signedEpochHead: spec/RPC.md, «Los rechazos del cable»)",
              file=sys.stderr)
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
