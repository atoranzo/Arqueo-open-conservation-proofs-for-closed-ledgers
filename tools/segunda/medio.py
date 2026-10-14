#!/usr/bin/env python3
"""El MEDIO del ancla en la segunda implementacion (RFC-0013 D-A, D-B y D-C), en Python y sin
dependencias: la base64 estricta, la inclusion de RFC 9162 en el arbol SHA-256 de anclas y la nota
`checkpoint` con su firma ML-DSA-44 de tipo 0x06. Solo verificar.

De donde sale cada pieza, y de donde NO:
  - el arbol: RFC 9162, 2.1.1 (MTH, con los prefijos 0x00 de hoja y 0x01 de nodo de RFC 6962) y
    2.1.3.2 (verificar un camino de inclusion), mas los bordes que declara la cabecera de
    `crates/zk-ssl-medio/src/medio.rs` (una posicion fuera del arbol no tiene inclusion);
  - la nota: las especificaciones C2SP `signed-note`, `tlog-checkpoint` y `tlog-cosignature` (el
    `cosigned_message` con la etiqueta `subtree/v1\\n\\0`, el `key_id` de cuatro bytes y la `vkey`),
    con las reglas que DECLARA la cabecera de `crates/zk-ssl-medio/src/nota.rs` -las de `note.Open`
    y torchwood, y las dos mas estrictas: el `origin` es el nombre del publicador y una sola linea
    suya-, que es lo que spec/PAQUETE.md 2.13 manda leer. Los textos de los rechazos son los del
    `Display` de `ErrorDeNota` y `SubtreeError`, para que la causa sea la del binario (PAQUETE.md,
    seccion 6.1): esos textos SI se leyeron del `.rs`, como las AIR de `airs.py`;
  - la base64: RFC 4648, estandar y con relleno, con la estrictez que declara
    `crates/zk-ssl-medio/src/base64.rs` (relleno obligatorio, bits sobrantes a cero, ningun blanco);
  - ML-DSA-44: `mldsa44.py`, desde FIPS 204.

Lo que NO hace: la consistencia entre dos tamanos del arbol (un testigo la comprueba antes de
cofirmar; el sobre no la pide), ni juzgar a los testigos: las lineas ajenas se devuelven sin
verificar, como en la referencia, y `verificar_cofirma` comprueba una si se le da su clave.
"""
import hashlib
import os
import sys
import unicodedata

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mldsa44  # noqa: E402

TIPO_MLDSA44 = 0x06
ETIQUETA = b"subtree/v1\n\x00"
PREFIJO_ORIGEN = "zkssl/v1/"
MAX_LINEAS_DE_FIRMA = 100
PREFIJO_FIRMA = "— "  # la raya y un espacio
I64_MAX = (1 << 63) - 1


# ─── la base64 estricta ─────────────────────────────────────────────────────────────────────────

_ALFABETO = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
_VALOR = {ord(c): i for i, c in enumerate(_ALFABETO)}


def _utf8(s):
    """Los bytes UTF-8 de una cadena, como los ve Rust. Una suplente sola (`\\ud800`) no llega aqui
    desde `verificador.py`: el JSON de Python la deja pasar y serde_json no, y `juzgar` la rechaza
    antes, con `JSON ilegible`, como el binario. A quien llame con otra cadena, `surrogatepass` le da
    sus tres bytes, que no son UTF-8, en vez de una excepcion."""
    return s.encode("utf-8", "surrogatepass")


class Base64Malo(Exception):
    pass


def base64_codificar(b):
    out = []
    for i in range(0, len(b), 3):
        trozo = b[i:i + 3]
        n = int.from_bytes(trozo + b"\x00" * (3 - len(trozo)), "big")
        out.append(_ALFABETO[(n >> 18) & 63] + _ALFABETO[(n >> 12) & 63])
        out.append(_ALFABETO[(n >> 6) & 63] if len(trozo) > 1 else "=")
        out.append(_ALFABETO[n & 63] if len(trozo) > 2 else "=")
    return "".join(out)


def base64_decodificar(s):
    """Estricta: longitud multiplo de cuatro, relleno solo al final y como mucho dos, nada fuera del
    alfabeto (ni un blanco) y los bits sobrantes a cero. Una sola escritura por contenido."""
    simbolos = _utf8(s) if isinstance(s, str) else bytes(s)
    if len(simbolos) % 4:
        raise Base64Malo()
    out = bytearray()
    cuartetos = len(simbolos) // 4
    for i in range(cuartetos):
        q = simbolos[4 * i:4 * i + 4]
        relleno = len(q) - len(q.rstrip(b"="))
        if relleno > 2 or (relleno > 0 and i != cuartetos - 1):
            raise Base64Malo()
        n = 0
        for c in q[:4 - relleno]:
            if c not in _VALOR:
                raise Base64Malo()
            n = (n << 6) | _VALOR[c]
        n <<= 6 * relleno
        b = n.to_bytes(3, "big")
        if relleno == 0:
            out += b
        elif relleno == 1:
            if n & 0xff:
                raise Base64Malo()
            out += b[:2]
        else:
            if n & 0xffff:
                raise Base64Malo()
            out += b[:1]
    return bytes(out)


# ─── el arbol de anclas: RFC 9162 ───────────────────────────────────────────────────────────────

def sha256(b):
    return hashlib.sha256(b).digest()


def hoja(huella):
    """La hoja del ancla de huella `huella`: SHA-256(0x00 || huella)."""
    return sha256(b"\x00" + bytes(huella))


def nodo(izq, der):
    return sha256(b"\x01" + izq + der)


def mth(hojas):
    """MTH de RFC 9162 2.1.1 sobre las hojas YA prefijadas (`hoja(...)`). Solo para el juez."""
    n = len(hojas)
    if n == 0:
        return sha256(b"")
    if n == 1:
        return hojas[0]
    k = 1 << ((n - 1).bit_length() - 1)
    return nodo(mth(hojas[:k]), mth(hojas[k:]))


def camino(hojas, m):
    """PATH(m, D[n]) de RFC 9162 2.1.3.1, sobre hojas prefijadas. Solo para el juez."""
    n = len(hojas)
    if n == 1:
        return []
    k = 1 << ((n - 1).bit_length() - 1)
    if m < k:
        return camino(hojas[:k], m) + [mth(hojas[k:])]
    return camino(hojas[k:], m - k) + [mth(hojas[:k])]


class ErrorDelMedio(Exception):
    pass


def verificar_inclusion(hoja_, indice, tamano, prueba, raiz):
    """RFC 9162 2.1.3.2 sobre el arbol entero [0, tamano). Lanza ErrorDelMedio con el texto de la
    referencia (`ErrorDelMedio` y `SubtreeError`, en ese orden de comprobacion)."""
    if indice >= tamano:
        raise ErrorDelMedio(f"{indice} no cae en un árbol de {tamano} hojas")
    fn, sn, r = indice, tamano - 1, hoja_
    for p in prueba:
        if sn == 0:
            raise ErrorDelMedio("leftover hashes in the path")
        if fn & 1 or fn == sn:
            r = nodo(p, r)
            while fn != 0 and fn & 1 == 0:
                fn >>= 1
                sn >>= 1
        else:
            r = nodo(r, p)
        fn >>= 1
        sn >>= 1
    if sn != 0:
        raise ErrorDelMedio("the path ends before the root")
    if r != raiz:
        raise ErrorDelMedio("the path climbs to a different hash")


# ─── la nota checkpoint y su firma 0x06 ─────────────────────────────────────────────────────────

class ErrorDeNota(Exception):
    """`clase` es la variante de la referencia (la columna 2 del MANIFIESTO de las notas); el texto, su
    `Display`."""

    def __init__(self, clase, texto):
        super().__init__(texto)
        self.clase = clase


def _malformada(s):
    return ErrorDeNota("Malformada", f"nota mal formada: {s}")


def _checkpoint(s):
    return ErrorDeNota("Checkpoint", f"checkpoint mal formado: {s}")


def _clave(s):
    return ErrorDeNota("Clave", f"clave: {s}")


FIRMA_INVALIDA = ("FirmaInvalida", "la firma del publicador no verifica")

# White_Space de Unicode, el conjunto de `char::is_whitespace`: `str.isspace` de Python no es el mismo
# (cuenta tambien U+001C a U+001F, que no son White_Space).
_BLANCOS = set(range(0x09, 0x0e)) | {0x20, 0x85, 0xa0, 0x1680} | set(range(0x2000, 0x200b)) | \
    {0x2028, 0x2029, 0x202f, 0x205f, 0x3000}


def _es_control(c):
    """`char::is_control`: la categoria Cc, U+0000 a U+001F y U+007F a U+009F."""
    o = ord(c)
    return o < 0x20 or 0x7f <= o <= 0x9f


# Other_Grapheme_Extend de PropList.txt, Unicode 16.0 (la tabla de `regex-syntax` 0.8.11, en el
# registro de cargo, generada de ese UCD). Con Mn y Me es Grapheme_Extend. El kit toma la suya del
# `rustc` que lo compila, 1.94.1, que es la 17.0: escalar a escalar, ninguno sale distinto por esta
# tabla (medido en README.md, «Lo que NO cubre»).
_OTRAS_EXTENSIONES = (
    (0x9be, 0x9be), (0x9d7, 0x9d7), (0xb3e, 0xb3e), (0xb57, 0xb57), (0xbbe, 0xbbe), (0xbd7, 0xbd7),
    (0xcc0, 0xcc0), (0xcc2, 0xcc2), (0xcc7, 0xcc8), (0xcca, 0xccb), (0xcd5, 0xcd6), (0xd3e, 0xd3e),
    (0xd57, 0xd57), (0xdcf, 0xdcf), (0xddf, 0xddf), (0x1715, 0x1715), (0x1734, 0x1734), (0x1b35, 0x1b35),
    (0x1b3b, 0x1b3b), (0x1b3d, 0x1b3d), (0x1b43, 0x1b44), (0x1baa, 0x1baa), (0x1bf2, 0x1bf3),
    (0x200c, 0x200c), (0x302e, 0x302f), (0xa953, 0xa953), (0xa9c0, 0xa9c0), (0xff9e, 0xff9f),
    (0x111c0, 0x111c0), (0x11235, 0x11235), (0x1133e, 0x1133e), (0x1134d, 0x1134d), (0x11357, 0x11357),
    (0x113b8, 0x113b8), (0x113c2, 0x113c2), (0x113c5, 0x113c5), (0x113c7, 0x113c9), (0x113cf, 0x113cf),
    (0x114b0, 0x114b0), (0x114bd, 0x114bd), (0x115af, 0x115af), (0x116b6, 0x116b6), (0x11930, 0x11930),
    (0x1193d, 0x1193d), (0x11f41, 0x11f41), (0x16ff0, 0x16ff1), (0x1d165, 0x1d166), (0x1d16d, 0x1d172),
    (0xe0020, 0xe007f),
)
# Las categorias que `core::unicode::printable` da por no imprimibles (su generador, `printable.py`),
# el espacio aparte.
_NO_IMPRIMIBLES = frozenset(("Zs", "Zl", "Zp", "Cc", "Cf", "Cs", "Co", "Cn"))


def _escapa_en_debug(c):
    """`char::escape_debug_ext` con `escape_grapheme_extended`, lo que `<str as Debug>::fmt` pide para
    TODO caracter (no solo el primero, como `str::escape_debug`): Grapheme_Extend, o no imprimible."""
    o = ord(c)
    if unicodedata.category(c) in ("Mn", "Me") or any(a <= o <= b for a, b in _OTRAS_EXTENSIONES):
        return True
    return c != " " and unicodedata.category(c) in _NO_IMPRIMIBLES


def debug_rust(s):
    """El `{:?}` de una cadena en Rust: comillas dobles; `\\0`, `\\t`, `\\r`, `\\n`, `\\\\` y `\\"`; y
    `\\u{..}` en hex para lo que extiende un grafema y lo no imprimible (las categorias de
    `_NO_IMPRIMIBLES`). La comilla simple no se escapa. Las categorias son las de `unicodedata`, la
    version de Unicode del Python que corre (14.0 en 3.11) y no la de Rust: lo asignado despues es Cn
    aqui y sale escapado, y en Rust puede no estarlo. Medido contra Rust en README.md, «El medio» y
    «Lo que NO cubre»."""
    out = ['"']
    for c in s:
        if c in '\\"':
            out.append("\\" + c)
        elif c == "\n":
            out.append("\\n")
        elif c == "\r":
            out.append("\\r")
        elif c == "\t":
            out.append("\\t")
        elif c == "\0":
            out.append("\\0")
        elif _escapa_en_debug(c):
            out.append("\\u{%x}" % ord(c))
        else:
            out.append(c)
    out.append('"')
    return "".join(out)


def nombre_valido(nombre):
    """Un nombre de clave de `signed-note`: no vacio, sin blancos y sin `+`."""
    return bool(nombre) and not any(ord(c) in _BLANCOS or c == "+" for c in nombre)


def origen_del_medio(huella_de_clave):
    """El `origin` del medio de un operador (D-A): `zkssl/v1/` y la huella de su clave XMSS en hex."""
    return PREFIJO_ORIGEN + bytes(huella_de_clave).hex()


def _u64_de_texto(s):
    """`str::parse::<u64>` de Rust: un `+` opcional y cifras ASCII, sin desbordar. None si no."""
    t = s[1:] if s.startswith("+") else s
    if not t or any(c not in "0123456789" for c in t):
        return None
    v = int(t)
    return v if v < 1 << 64 else None


class Checkpoint:
    def __init__(self, origen, tamano, raiz):
        self.origen, self.tamano, self.raiz = origen, tamano, raiz

    def __eq__(self, o):
        return (self.origen, self.tamano, self.raiz) == (o.origen, o.tamano, o.raiz)

    def comprobar(self):
        if not self.origen or len(_utf8(self.origen)) > 255:
            raise _checkpoint("el origin mide de 1 a 255 bytes")
        if "\n" in self.origen or any(_es_control(c) for c in self.origen):
            raise _checkpoint("el origin no es una línea")
        if self.tamano > I64_MAX:
            raise _checkpoint("el tamaño pasa de 2^63-1")

    @staticmethod
    def leer(texto):
        """Solo la escritura canonica de un checkpoint de tres lineas, sin lineas de extension."""
        if not texto.endswith("\n"):
            raise _checkpoint("el texto no acaba en salto de línea")
        lineas = texto[:-1].split("\n")
        if len(lineas) < 3:
            raise _checkpoint("menos de tres líneas")
        if len(lineas) > 3:
            raise _checkpoint("líneas de extensión: el tipo 0x06 no las cubre")
        tamano = _u64_de_texto(lineas[1])
        if tamano is None:
            raise _checkpoint("el tamaño no es un número")
        if lineas[1] != str(tamano):
            raise _checkpoint("el tamaño no está en su forma canónica")
        try:
            raiz = base64_decodificar(lineas[2])
        except Base64Malo:
            raise _checkpoint("la raíz no es base64 estricto")
        if len(raiz) != 32:
            raise _checkpoint("la raíz no mide 32 bytes")
        c = Checkpoint(lineas[0], tamano, raiz)
        c.comprobar()
        return c


def es_subarbol(inicio, fin):
    """La definicion de subarbol de `tlog-cosignature`: `inicio <= fin` e `inicio` multiplo de
    BIT_CEIL(fin - inicio), con la guarda del desbordamiento de su ejemplo en C++."""
    if inicio > fin:
        return False
    tam = fin - inicio
    if tam > 1 << 63:
        return inicio == 0
    techo = 1 if tam <= 1 else 1 << (tam - 1).bit_length()
    return inicio & (techo - 1) == 0


def mensaje_cofirmado(nombre, marca, origen, inicio, fin, h):
    """El `cosigned_message` de `tlog-cosignature` para ML-DSA-44. None si no se puede formar: la
    referencia lo da como error y quien verifica lo convierte en FirmaInvalida."""
    n, o = _utf8(nombre), _utf8(origen)
    if not 1 <= len(n) <= 255 or not 1 <= len(o) <= 255 or marca > I64_MAX:
        return None
    if marca != 0 and inicio != 0:
        return None
    if not es_subarbol(inicio, fin):
        return None
    if inicio == fin and h != sha256(b""):
        return None
    return (ETIQUETA + bytes([len(n)]) + n + marca.to_bytes(8, "big") + bytes([len(o)]) + o
            + inicio.to_bytes(8, "big") + fin.to_bytes(8, "big") + h)


def id_de_clave(nombre, clave_publica):
    """`SHA-256(nombre || "\\n" || 0x06 || clave)[:4]`."""
    return sha256(_utf8(nombre) + b"\n" + bytes([TIPO_MLDSA44]) + bytes(clave_publica))[:4]


class ClaveDeNota:
    def __init__(self, nombre, clave_publica):
        if not nombre_valido(nombre):
            raise _clave("nombre vacío, con blancos o con +")
        if len(clave_publica) != mldsa44.BYTES_CLAVE_PUBLICA:
            raise _clave("la clave ML-DSA-44 mide 1312 bytes")
        self.nombre, self.bytes, self.id = nombre, bytes(clave_publica), id_de_clave(nombre, clave_publica)

    @staticmethod
    def leer_vkey(vkey):
        """`nombre+hex(key_id)+base64(0x06 || clave)`, y el `key_id` tiene que ser el de las otras dos."""
        partes = vkey.split("+", 2)
        if len(partes) != 3:
            raise _clave("una vkey tiene tres partes")
        nombre, id_hex, b64 = partes
        if len(_utf8(id_hex)) != 8 or any(c not in "0123456789abcdefABCDEF" for c in id_hex):
            raise _clave("el key_id son 8 cifras hexadecimales")
        ident = int(id_hex, 16).to_bytes(4, "big")
        try:
            material = base64_decodificar(b64)
        except Base64Malo:
            raise _clave("la clave no es base64")
        if not material:
            raise _clave("la vkey no lleva clave")
        if material[0] != TIPO_MLDSA44:
            raise _clave("la vkey no es de tipo 0x06")
        c = ClaveDeNota(nombre, material[1:])
        if c.id != ident:
            raise _clave("el key_id no es el de esa clave")
        return c

    def vkey(self):
        return f"{self.nombre}+{self.id.hex()}+{base64_codificar(bytes([TIPO_MLDSA44]) + self.bytes)}"

    def verifica_linea(self, checkpoint, resto):
        """`timestamp || firma` sobre el checkpoint, con el nombre de esta clave: la marca si verifica."""
        if len(resto) != 8 + mldsa44.BYTES_FIRMA:
            raise ErrorDeNota(*FIRMA_INVALIDA)
        marca = int.from_bytes(resto[:8], "big")
        m = mensaje_cofirmado(self.nombre, marca, checkpoint.origen, 0, checkpoint.tamano, checkpoint.raiz)
        if m is None or not mldsa44.verificar(self.bytes, m, resto[8:]):
            raise ErrorDeNota(*FIRMA_INVALIDA)
        return marca

    def verificar_cofirma(self, checkpoint, linea):
        if linea.nombre != self.nombre or linea.id != self.id:
            raise _clave("la línea no es de esta clave")
        return self.verifica_linea(checkpoint, linea.firma)


class FirmaAjena:
    """Una linea de firma de una clave que no es la del publicador. NO se ha verificado."""

    def __init__(self, nombre, ident, firma):
        self.nombre, self.id, self.firma = nombre, ident, firma


class NotaVerificada:
    def __init__(self, checkpoint, marca, ajenas):
        self.checkpoint, self.marca, self.ajenas = checkpoint, marca, ajenas


def verificar_nota(nota, publicador):
    """Verifica una nota del medio contra la clave del publicador, en el orden de la referencia: la
    forma de la nota y de sus lineas de firma, el checkpoint, el `origin`, la linea del publicador
    (una, ni cero ni dos) y su firma."""
    if any(c < " " and c != "\n" for c in nota):
        raise _malformada("caracteres de control")
    corte = nota.rfind("\n\n")
    if corte < 0:
        raise _malformada("sin línea en blanco")
    texto, firmas = nota[:corte + 1], nota[corte + 2:]
    if not firmas.endswith("\n"):
        raise _malformada("las firmas no acaban en salto de línea")
    del_publicador, ajenas = [], []
    for n, linea in enumerate(firmas[:-1].split("\n")):
        if n >= MAX_LINEAS_DE_FIRMA:
            raise _malformada("más de 100 líneas de firma")
        if not linea.startswith(PREFIJO_FIRMA):
            raise _malformada("una línea de firma no empieza por «— »")
        resto = linea[len(PREFIJO_FIRMA):]
        nombre, _, b64 = resto.partition(" ")
        if not nombre_valido(nombre):
            raise _malformada("nombre de clave inválido")
        try:
            b = base64_decodificar(b64)
        except Base64Malo:
            raise _malformada("una firma no es base64 estricto")
        if len(b) < 5:
            raise _malformada("una firma de menos de 5 bytes")
        if nombre == publicador.nombre and b[:4] == publicador.id:
            del_publicador.append(b)
        else:
            ajenas.append(FirmaAjena(nombre, b[:4], b[4:]))
    checkpoint = Checkpoint.leer(texto)
    if checkpoint.origen != publicador.nombre:
        raise ErrorDeNota("OrigenAjeno", f"el origin es {debug_rust(checkpoint.origen)} y el publicador "
                                         f"{debug_rust(publicador.nombre)}")
    if not del_publicador:
        raise ErrorDeNota("SinFirmaDelPublicador", "ninguna firma del publicador")
    if len(del_publicador) > 1:
        raise ErrorDeNota("FirmaRepetida", "más de una firma del publicador")
    marca = publicador.verifica_linea(checkpoint, del_publicador[0][4:])
    return NotaVerificada(checkpoint, marca, ajenas)
