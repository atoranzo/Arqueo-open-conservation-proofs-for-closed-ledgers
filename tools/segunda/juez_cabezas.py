#!/usr/bin/env python3
"""El juez de las CABEZAS FIRMADAS, con la segunda implementacion (BACKLOG 85, segundo hito): recorre
los vectores, y para cada cabeza (`cabeza`, `vieja`, `nueva`, o el resultado del cable) recompone el
`epochDigest` desde sus campos segun `formatVersion` (NUCLEO.md seccion 6), compara el preambulo
recuperado de la firma con `preambulo(version, digest)`, verifica la firma XMSS^MT contra
`publicKey` (RFC 8391, tools/segunda/kat_xmss/xmss.py) y comprueba que el `index` declarado queda por encima
del embebido (PAQUETE.md, 2/3 y seccion 8). Y lo mismo con cada cofirma de testigo, con
`preambulo_cofirma`.

    python3 tools/segunda/juez_cabezas.py [raiz de vectores]

Salida: una linea por fichero con cabezas, `OK` si todas sus cabezas y cofirmas verifican, `NO`
con el motivo si alguna no; un FALSADOR que voltea un byte de la firma y uno del preambulo de la
primera cabeza que verifica; y al final cuantas cabezas y cofirmas verifican y si cada fichero con
una cabeza que NO verifica es un negativo segun su MANIFIESTO. Exit 0 si todo cuadra; 1 si no.
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "kat_xmss"))  # xmss.py vive ahi
import nucleo as N  # noqa: E402
import xmss  # noqa: E402

CAMPOS_V = {  # campos que cada version anade, en el orden de las composiciones de NUCLEO.md seccion 6
    2: ["acusesRoot", "n"],
    3: ["mmrRoot", "mmrSize"],
    4: ["consRoot", "consCount"],
    5: ["paramsDigest", "pmetaRoot", "nextPending", "nextIndex", "totalSupply"],
    6: ["recepRoot", "recepCount"],
}


def hexbytes(s):
    if not isinstance(s, str):
        raise ValueError(f"se esperaba hex y llego {type(s).__name__}")
    s = s[2:] if s.startswith("0x") else s
    return bytes.fromhex(s)


def u64(s):
    return int(s, 16)


def dig(s):
    return N.digest_from_bytes(hexbytes(s))


def recomponer(c):
    v = u64(c["formatVersion"])
    d = N.epoch_digest(u64(c["seq"]), dig(c["accountsRoot"]), dig(c["pendingRoot"]), dig(c["frozenRoot"]),
                       dig(c["chainDigest"]))
    if v >= 2:
        d = N.epoch_digest_v2(d, dig(c["acusesRoot"]), u64(c["n"]))
    if v >= 3:
        d = N.epoch_digest_v3(d, dig(c["mmrRoot"]), u64(c["mmrSize"]))
    if v >= 4:
        d = N.epoch_digest_v4(d, dig(c["consRoot"]), u64(c["consCount"]))
    if v >= 5:
        d = N.epoch_digest_v5(d, dig(c["paramsDigest"]), dig(c["pmetaRoot"]), u64(c["nextPending"]),
                              u64(c["nextIndex"]), u64(c["totalSupply"]))
    if v >= 6:
        d = N.epoch_digest_v6(d, dig(c["recepRoot"]), u64(c["recepCount"]))
    if v > 6 or v < 1:
        raise ValueError(f"formatVersion {v} fuera del conjunto")
    return v, d


def partir(firmado, oid):
    """La firma de RFC 8391 va DELANTE y el mensaje detras (medido: los cinco primeros bytes son el
    indice embebido, y los ultimos 50 o 124 el preambulo)."""
    largo = xmss.largo_firma(oid)
    return firmado[:largo], firmado[largo:]


def juzgar_cabeza(c):
    """Devuelve (verifica: bool, motivo: str, idx_embebido)."""
    try:
        v, d = recomponer(c)
    except (KeyError, ValueError, TypeError) as ex:
        return False, f"no recompone: {type(ex).__name__} {ex}", None
    try:
        if N.digest_to_bytes(d) != hexbytes(c["epochDigest"]):
            return False, "los campos no recomponen el epochDigest", None
    except ValueError as ex:
        return False, f"epochDigest ilegible: {ex}", None
    try:
        pk = hexbytes(c["publicKey"])
        oid, _, _ = xmss.parsear_clave(pk)
        firma, mensaje = partir(hexbytes(c["signature"]), oid)
    except ValueError as ex:
        return False, f"clave o firma ilegible: {ex}", None
    if mensaje != N.preambulo(v, d):
        return False, "el preambulo recuperado no es el esperado", None
    if not xmss.verificar(pk, mensaje, firma):
        return False, "la firma XMSS no verifica", None
    idx = xmss.indice_embebido(oid, firma)
    if "index" not in c:
        return False, f"falta index (el embebido es {idx})", idx
    try:
        declarado = u64(c["index"])
    except ValueError:
        return False, f"index ilegible: {c['index']!r}", idx
    if declarado <= idx:
        return False, f"index declarado {declarado} no queda por encima del embebido {idx}", idx
    return True, f"v{v}, indice embebido {idx}", idx


def juzgar_cofirma(co):
    try:
        v = u64(co["versionFormato"])
        d = dig(co["epochDigest"])
        pk_t = hexbytes(co["clavePublicaTestigo"])
        pk_op = hexbytes(co["clavePublicaOperador"])
        oid, _, _ = xmss.parsear_clave(pk_t)
        firma, mensaje = partir(hexbytes(co["firma"]), oid)
    except (KeyError, ValueError, TypeError) as ex:
        return False, f"cofirma: {type(ex).__name__} {ex}"
    if mensaje != N.preambulo_cofirma(v, d, pk_op):
        return False, "cofirma: el preambulo recuperado no es el esperado"
    if not xmss.verificar(pk_t, mensaje, firma):
        return False, "cofirma: la firma XMSS no verifica"
    return True, "cofirma verifica"


def cabezas_de(obj, ruta=""):
    """Todas las cabezas firmadas y cofirmas de un JSON, donde esten."""
    if isinstance(obj, dict):
        if {"publicKey", "signature", "epochDigest", "formatVersion", "seq"} <= set(obj):
            yield "cabeza", ruta or "cabeza", obj
        elif {"clavePublicaTestigo", "firma", "epochDigest", "versionFormato", "clavePublicaOperador"} <= set(obj):
            yield "cofirma", ruta or "cofirma", obj
        else:
            for k, v in obj.items():
                yield from cabezas_de(v, f"{ruta}.{k}" if ruta else k)
    elif isinstance(obj, list):
        for i, v in enumerate(obj):
            yield from cabezas_de(v, f"{ruta}[{i}]")


def manifiesto_de(directorio):
    ruta = os.path.join(directorio, "MANIFIESTO.txt")
    esperado = {}
    if os.path.exists(ruta):
        for linea in open(ruta, encoding="utf-8"):
            linea = linea.strip()
            if linea and not linea.startswith("#") and "|" in linea:
                f, codigo, *_ = linea.split("|")
                esperado[f] = int(codigo)
    return esperado


def falsador(c):
    """Un byte volteado en la firma y uno en el preambulo: las dos tienen que NO verificar."""
    pk = hexbytes(c["publicKey"])
    oid, _, _ = xmss.parsear_clave(pk)
    firma, mensaje = partir(hexbytes(c["signature"]), oid)
    f2 = bytearray(firma); f2[len(f2) // 2] ^= 0x01
    m2 = bytearray(mensaje); m2[-1] ^= 0x01
    return (not xmss.verificar(pk, mensaje, bytes(f2))) and (not xmss.verificar(pk, bytes(m2), firma))


def main(argv):
    raiz = argv[1] if len(argv) > 1 else os.path.join("spec", "vectors")
    total_c = ok_c = total_co = ok_co = 0
    ficheros_no, ficheros = [], 0
    falsado = None
    for dirpath, _, nombres in sorted(os.walk(raiz)):
        esperado = manifiesto_de(dirpath)
        for nombre in sorted(nombres):
            if not nombre.endswith(".json"):
                continue
            ruta = os.path.join(dirpath, nombre)
            try:
                doc = json.load(open(ruta, encoding="utf-8"))
            except Exception:
                continue
            motivos = []
            n_aqui = 0
            for clase, donde, obj in cabezas_de(doc):
                n_aqui += 1
                if clase == "cabeza":
                    total_c += 1
                    ok, motivo, _ = juzgar_cabeza(obj)
                    ok_c += ok
                    if ok and falsado is None:
                        falsado = falsador(obj)
                else:
                    total_co += 1
                    ok, motivo = juzgar_cofirma(obj)
                    ok_co += ok
                if not ok:
                    motivos.append(f"{donde}: {motivo}")
            if n_aqui == 0:
                continue
            ficheros += 1
            rel = os.path.relpath(ruta, raiz)
            if motivos:
                codigo = esperado.get(nombre)
                etiqueta = "negativo" if codigo not in (None, 0) else ("SIN MANIFIESTO" if codigo is None else "POSITIVO")
                ficheros_no.append((rel, etiqueta))
                print(f"NO   {rel} [{etiqueta}]: " + "; ".join(motivos))
            else:
                print(f"OK   {rel} ({n_aqui})")
    positivos_rotos = [r for r, e in ficheros_no if e == "POSITIVO"]
    print(f"FALSADOR: un byte volteado en la firma y otro en el preambulo " + ("NO verifican: VERDE" if falsado else "VERIFICAN: ROJO"))
    print(f"cabezas: {ok_c} de {total_c} verifican; cofirmas: {ok_co} de {total_co}; ficheros con cabezas: {ficheros}; "
          f"con alguna que NO verifica: {len(ficheros_no)}, de ellos positivos segun su MANIFIESTO: {len(positivos_rotos)}")
    return 0 if (falsado and not positivos_rotos and total_c > 0) else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
