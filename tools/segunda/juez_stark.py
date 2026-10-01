#!/usr/bin/env python3
"""El juez del verificador STARK de la segunda implementacion (BACKLOG 85, cuarto hito).

Toma cada vector de `spec/vectors/` que lleva una prueba STARK en las familias que el kit verifica
hoy -rechazo (la banda), edad, pago, pendiente (el cobro), prenda y completitud (la prenda de una
resolucion)-, saca el PAR que el mando juzga -la prueba y el enunciado que le compone, con los
mismos campos y la misma cabeza que `zk-ssl-verify` lee-, lo verifica con `stark.py` y `airs.py`, y
lo compara con lo que fija el MANIFIESTO de la familia:

  1. un positivo (codigo 0): el par verifica;
  2. un negativo cuya causa es del juez de la AIR -su texto lleva el prefijo del juez (`banda:`,
     `edad:`, `pago:`, `cobro:`, `prenda:`) o es uno de sus mensajes-: el par NO verifica, y con esa
     causa;
  3. un negativo que cae ANTES del juez (la cabeza, la firma, un campo): no se compara; se cuenta;
  4. completitud, por sus tres veredictos de la prenda: «aceptada» (el sobre contra SU cabeza
     verifica), «rechazada con prueba» (contra la cabeza que el nodo juzgo NO verifica) y «RECHAZO
     SIN FUNDAMENTO» (contra esa cabeza SI verifica: el nodo dijo que no sin razon);
  5. y tres falsadores por positivo: un byte volteado en el primer, el segundo y el ultimo tercio
     de la prueba, y los tres tienen que dejar de verificar.

Los vectores del cable 0.3 (`spec/vectors/0.3/`) quedan FUERA: sin ocultar y sin sal, con las AIR
de `0eda58c`, y el kit de hoy no los verifica (rechaza su forma, D-AD).

Salida: una linea por par y una de resumen; exit 0 si todo dice lo que debe, 1 si no.
"""

import json
import os
import sys
import time

AQUI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, AQUI)
import airs as A  # noqa: E402
import stark as S  # noqa: E402
import verificador as V  # noqa: E402

RAIZ = os.path.normpath(os.path.join(AQUI, "..", ".."))
VECTORES = os.path.join(RAIZ, "spec", "vectors")

PREFIJO = {"rechazo": "banda:", "edad": "edad:", "pago": "pago:", "pendiente": "cobro:",
           "prenda": "prenda:"}

# Los mensajes del juez de cada AIR que un manifiesto puede citar sin su prefijo: los de
# `comprobar_enunciado` y `verificar_contra_cabeza` (airs.py los reproduce con el mismo texto).
MENSAJES_DEL_JUEZ = (
    "no es anterior a la cabeza de seq",
    "no es el pendingRoot de la cabeza",
    "no es el pmetaRoot de la cabeza",
    "pasan del techo",
    "pasa del techo",
    "banda vacia",
    "fuera de 3..=24",
    "mayor que n",
    "forma de traza",
)


class NoLlega(Exception):
    """El mando cae antes del juez: no hay par que juzgar."""


def _campo(obj, k):
    if not isinstance(obj, dict) or k not in obj:
        raise NoLlega(f"falta {k}")
    return obj[k]


def _leer(f, *a):
    try:
        return f(*a)
    except V.Rojo as e:
        raise NoLlega(str(e))


def _cabeza_v5(c):
    if _leer(V.u64_de, c, "formatVersion") < 5:
        raise NoLlega("la cabeza no es v5")


def par_rechazo(d):
    datos = _campo(d, "data")
    if datos.get("causa") != "InsufficientBalance" or "banda" not in d:
        raise NoLlega("sin banda")
    campos = _campo(datos, "campos")
    if "available" in campos:
        raise NoLlega("publica el saldo")
    g, c = d["banda"], _campo(d, "cabeza")
    pedido = _leer(V.u64_de, campos, "requested")
    if pedido != _leer(V.u64_de, g, "requested") or pedido == 0:
        raise NoLlega("el importe")
    prueba = _leer(V.hex_a_bytes, _campo(g, "prueba"))
    air = A.Banda(_leer(V.digest_de, c, "accountsRoot"), _leer(V.digest_de, g, "publicId"), 0,
                  pedido - 1)
    return [("banda", prueba, air)]


def par_edad(d):
    e, sub, c = _campo(d, "enunciado"), _campo(d, "subraices"), _campo(d, "cabeza")
    _cabeza_v5(c)
    emisor = _leer(V.u64_de, e, "emisor") if "emisor" in e else None
    air = A.Edad(_leer(V.u64_de, c, "seq"), _leer(V.digest_de, c, "pendingRoot"),
                 _leer(V.digest_de, c, "pmetaRoot"), _leer(V.u64_de, c, "nextPending"),
                 _leer(V.u64_de, e, "t"), _leer(V.u64_de, e, "k"), emisor,
                 _leer(V.digest_de, sub, "pendientes"), _leer(V.digest_de, sub, "meta"))
    return [("edad", _leer(V.hex_a_bytes, _campo(d, "prueba")), air)]


def par_pago(d):
    e, c = _campo(d, "enunciado"), _campo(d, "cabeza")
    _cabeza_v5(c)
    air = A.PagoEnCurso(_leer(V.u64_de, c, "seq"), _leer(V.digest_de, c, "pendingRoot"),
                        _leer(V.digest_de, c, "pmetaRoot"), _leer(V.digest_de, e, "receptor"),
                        _leer(V.u64_de, e, "importe"), _leer(V.u64_de, e, "t"),
                        _leer(V.u64_de, e, "nacido"))
    return [("pago", _leer(V.hex_a_bytes, _campo(d, "prueba")), air)]


def par_pendiente(d):
    e, c = _campo(d, "enunciado"), _campo(d, "cabeza")
    _cabeza_v5(c)
    air = A.CobroPendiente(_leer(V.u64_de, c, "seq"), _leer(V.digest_de, c, "pendingRoot"),
                           _leer(V.digest_de, c, "pmetaRoot"), _leer(V.digest_de, e, "receptor"),
                           _leer(V.u64_de, e, "nacido"), _leer(V.u64_de, e, "inferior"))
    return [("cobro", _leer(V.hex_a_bytes, _campo(d, "prueba")), air)]


def _prenda_contra(sobre, cabeza):
    e = _campo(sobre, "enunciado")
    air = A.Prenda(_leer(V.digest_de, cabeza, "pendingRoot"), _leer(V.digest_de, e, "receptor"),
                   _leer(V.digest_de, e, "marca"))
    return _leer(V.hex_a_bytes, _campo(sobre, "prueba")), air


def par_prenda(d):
    c = _campo(d, "cabeza")
    _cabeza_v5(c)
    prueba, air = _prenda_contra(d, c)
    return [("prenda", prueba, air)]


def par_completitud(d):
    x = _campo(d, "resolucion")
    sobre = _campo(x, "sobre")
    if sobre.get("tipo") != "prenda":
        raise NoLlega("el sobre no es de prenda")
    if "consumo" in x and "respuesta" not in x and "rechazo" not in x:
        prueba, air = _prenda_contra(sobre, _campo(sobre, "cabeza"))
        return [("prenda contra su cabeza", prueba, air)]
    if "respuesta" in x and "consumo" not in x and "rechazo" not in x:
        prueba, air = _prenda_contra(sobre, _campo(x, "juzgada"))
        return [("prenda contra la juzgada", prueba, air)]
    raise NoLlega("sin STARK de prenda")


PARES = {"rechazo": par_rechazo, "edad": par_edad, "pago": par_pago, "pendiente": par_pendiente,
         "prenda": par_prenda, "completitud": par_completitud}


def esperado(familia, codigo, texto):
    """'VERDE', ('ROJO', causa) o None (no se compara)."""
    if familia == "completitud":
        if codigo == 0 and "PRENDA aceptada" in texto:
            return "VERDE"
        if codigo == 0 and "rechazada con prueba" in texto:
            return ("ROJO", "")
        if codigo != 0 and "RECHAZO SIN FUNDAMENTO" in texto:
            return "VERDE"
        return None
    if codigo == 0:
        return "VERDE"
    pre = PREFIJO[familia]
    if texto.startswith(pre):
        return ("ROJO", texto[len(pre):].strip())
    if any(m in texto for m in MENSAJES_DEL_JUEZ):
        return ("ROJO", texto)
    return None


def juzgar_par(prueba, air):
    try:
        S.verificar(prueba, air)
        return "VERDE", ""
    except S.Rechazo as e:
        return "ROJO", str(e)


def falsadores(prueba, air):
    n = len(prueba)
    vivos = []
    for frac in (1, 2, 3):
        i = min(n - 1, n * frac // 3 - 1 if frac == 3 else n * frac // 3)
        b = bytearray(prueba)
        b[i] ^= 0x01
        v, _ = juzgar_par(bytes(b), air)
        if v == "VERDE":
            vivos.append(i)
    return vivos


def main():
    t0 = time.time()
    total = bien = 0
    no_llegan = 0
    fallos = []
    pruebas_juzgadas = 0
    falsadores_vivos = 0
    falsadores_total = 0
    for familia in ("rechazo", "edad", "pago", "pendiente", "prenda", "completitud"):
        carpeta = os.path.join(VECTORES, familia)
        for linea in open(os.path.join(carpeta, "MANIFIESTO.txt"), encoding="utf-8"):
            linea = linea.rstrip("\n")
            if not linea or linea.startswith("#"):
                continue
            fichero, codigo, texto = (linea.split("|", 2) + ["", ""])[:3]
            ruta = os.path.join(carpeta, fichero)
            if not os.path.exists(ruta):
                continue
            d = json.load(open(ruta, encoding="utf-8"))
            if "0x" not in json.dumps(d) or '"prueba"' not in json.dumps(d):
                continue
            quiere = esperado(familia, int(codigo), texto)
            try:
                pares = PARES[familia](d)
            except NoLlega as e:
                if quiere is not None:
                    total += 1
                    fallos.append(f"{familia}/{fichero}: el manifiesto pide {quiere} y el par no se "
                                  f"compone ({e})")
                    print(f"FALLA {familia}/{fichero}: no se compone el par ({e})")
                else:
                    no_llegan += 1
                continue
            for etiqueta, prueba, air in pares:
                v, causa = juzgar_par(prueba, air)
                pruebas_juzgadas += 1
                if quiere is None:
                    no_llegan += 1
                    print(f"  -- {familia}/{fichero} [{etiqueta}]: el mando cae antes del juez "
                          f"({texto[:50]}); el par da {v}")
                    continue
                total += 1
                if quiere == "VERDE":
                    ok = v == "VERDE"
                else:
                    ok = v == "ROJO" and quiere[1] in causa
                if ok and quiere == "VERDE":
                    vivos = falsadores(prueba, air)
                    falsadores_total += 3
                    falsadores_vivos += len(vivos)
                    if vivos:
                        ok = False
                        causa = f"falsadores que siguen verificando en los bytes {vivos}"
                if ok:
                    bien += 1
                    print(f"OK    {familia}/{fichero} [{etiqueta}]: {v}"
                          + (f" {causa}" if causa else ""))
                else:
                    fallos.append(f"{familia}/{fichero} [{etiqueta}]: pide {quiere}, da {v} {causa}")
                    print(f"FALLA {familia}/{fichero} [{etiqueta}]: pide {quiere}, da {v} {causa}")
    dt = time.time() - t0
    print(f"stark: {bien} de {total} pares dicen lo que deben ({pruebas_juzgadas} pruebas "
          f"juzgadas, {no_llegan} vectores caen antes del juez); falsadores que callan: "
          f"{falsadores_total - falsadores_vivos} de {falsadores_total} ({dt:.0f} s)")
    if fallos or bien != total:
        for f in fallos:
            print("  " + f, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    S.autotest()
    sys.exit(main())
