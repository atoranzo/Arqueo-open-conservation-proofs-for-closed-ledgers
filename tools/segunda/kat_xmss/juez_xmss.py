#!/usr/bin/env python3
"""El juez de los KAT de XMSS^MT: verifica cada firma de `vectores.json` -producidas por el crate
`xmss 0.1.0-pre.0` que la casa clava con `=`- con `xmss.py`, escrito desde RFC 8391 y sin
ningun crate detras. Dos implementaciones, dos origenes, los mismos bytes: es la red que la entrada 77
del BACKLOG echaba en falta bajo la version clavada.

    python3 juez_xmss.py [vectores.json]

Por cada firma: la adjunta (firma || mensaje) se parte por la longitud de RFC 8391, el mensaje
recuperado tiene que ser el del vector y la firma tiene que verificar; la separada tiene que ser
byte a byte la misma firma (misma semilla, mismo indice, mismo mensaje: RFC 8391 es determinista
dado el estado) y verificar; y el indice embebido tiene que ser el declarado. Falsadores: un byte
volteado en la firma, en el mensaje y en la raiz de la clave, cada uno tiene que NO verificar.
Exit 0 si todo cuadra; 1 si no.
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import xmss  # noqa: E402


def main(argv):
    ruta = argv[1] if len(argv) > 1 else os.path.join(os.path.dirname(os.path.abspath(__file__)), "vectores.json")
    doc = json.load(open(ruta, encoding="utf-8"))
    ok = total = 0
    rojos = []
    for c in doc["conjuntos"]:
        pk = bytes.fromhex(c["clave_publica"])
        oid, root, seed = xmss.parsear_clave(pk)
        if int(c["oid"], 16) != oid:
            rojos.append(f"{c['conjunto']}: el OID del vector {c['oid']} no es el de la clave {oid:#010x}")
        largo = xmss.largo_firma(oid)
        for f in c["firmas"]:
            total += 1
            nombre = f"{c['conjunto']} idx {f['indice']} ({len(f['mensaje']) // 2} B)"
            m = bytes.fromhex(f["mensaje"])
            adjunta = bytes.fromhex(f["firmado"])
            separada = bytes.fromhex(f["firma"])
            firma, recuperado = adjunta[:largo], adjunta[largo:]
            motivos = []
            if recuperado != m:
                motivos.append("el mensaje recuperado de la adjunta no es el del vector")
            if firma != separada:
                motivos.append("la firma separada no es byte a byte la de la adjunta")
            if xmss.indice_embebido(oid, firma) != f["indice"]:
                motivos.append(f"indice embebido {xmss.indice_embebido(oid, firma)} != {f['indice']}")
            if not xmss.verificar(pk, m, firma):
                motivos.append("la firma NO verifica con la implementacion de RFC 8391")
            # falsadores
            f2 = bytearray(firma); f2[len(f2) // 3] ^= 0x80
            if xmss.verificar(pk, m, bytes(f2)):
                motivos.append("FALSADOR: una firma con un byte volteado verifica")
            m2 = bytearray(m) + b"\x00" if not m else bytearray(m)
            if m:
                m2[0] ^= 0x01
            if xmss.verificar(pk, bytes(m2), firma):
                motivos.append("FALSADOR: otro mensaje verifica con la misma firma")
            pk2 = bytearray(pk); pk2[4 + 5] ^= 0x01
            if xmss.verificar(bytes(pk2), m, firma):
                motivos.append("FALSADOR: una clave con la raiz tocada verifica")
            if motivos:
                rojos.append(f"{nombre}: " + "; ".join(motivos))
                print(f"ROJO {nombre}: " + "; ".join(motivos))
            else:
                ok += 1
                print(f"OK   {nombre}")
    print(f"kat_xmss: {ok} de {total} firmas verifican con la implementacion de RFC 8391, y los tres falsadores "
          f"{'callan' if not rojos else 'NO callan'} - generador: {doc['generador']}")
    return 0 if ok == total and not rojos and total > 0 else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
