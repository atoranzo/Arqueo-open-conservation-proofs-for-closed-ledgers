#!/usr/bin/env python3
"""El juez de la segunda implementacion del NUCLEO (BACKLOG 85, primer hito): corre cada KAT de
`spec/vectors/nucleo/` contra `tools/segunda/nucleo.py` y dice, vector a vector, si reproduce la
salida byte a byte.

    python3 tools/segunda/juez_nucleo.py [directorio de los KAT]

Salida: una linea por vector, `OK   <fn>` o `ROJO <fn>: esperado <..> obtenido <..>`; `NOTA` para
las lecturas que NUCLEO.md seccion 6 deja abiertas y el KAT cierra; al final
`nucleo: <ok> de <n> vectores reproducidos byte a byte - implementacion <sha16>`.
Exit 0 solo si todos; 1 si alguno falla; 2 uso; 3 si el autotest de las primitivas falla, porque
entonces ningun veredicto vale.

Lo que el formato de los KAT no dice y este juez tiene que saber (NOTA al final): que `embeber.x`,
`balance`/`nonce` de `native_leaf*` y cada una de las `entradas_publicas` de `digest_pi` (S773) son
un u64 en OCHO BYTES little-endian, mientras `seq`, `n`, `t`, los contadores, `x` de `as_digest` y
`familia`/`version_air` de `digest_pi` son el u64 como numero hex. Son dos codificaciones del
mismo tipo en el mismo catalogo.

§800 · Y que uno no es de una `fn`: `registro_de_esquemas` (RFC-0020, E2) trae como `salida` la LISTA
de las entradas del registro de esquemas, no un hex. Este juez la compara entera con el registro de
`nucleo.py`, escrito desde NUCLEO.md seccion 6, y antes le pasa las reglas del registro a los dos,
con un falsador: una entrada gemela que comparte la escritura de la clave tiene que caer.

§862 · Y con el mismo KAT, los dos conjuntos del despacho por esquema (RFC-0020, E3) y el implicito
de `nucleo.py`: el implicito se emite, lo emitido se acepta y lo aceptado esta en el registro, con
un falsador: un conjunto aceptado vacio tiene que caer. No los fija el KAT, que es del registro.

§914 · Y otro que no es de una `fn`: `registro_de_familias` (RFC-0018 D-G, el corte zkssl/0.5) trae
como `salida` la LISTA de las 23 familias de la marca v2 de NUCLEO.md seccion 4 bis, cada una con su
numero, su AIR, la version de su AIR, su perfil y los 25 bytes de su marca, y como `entradas` la m y
las columnas de la casa. Este juez escribe cada marca con `nucleo.py` desde la tabla de la spec y la
compara byte a byte, y antes le pasa las reglas del registro a los dos, con un falsador: una AIR con
dos numeros tiene que caer.
"""
import hashlib
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import nucleo as N  # noqa: E402

U, ULE, D, B, I = "u64", "u64le", "digest", "bytes", "int"
CABEZA = {"seq": U, "accounts_root": D, "pending_root": D, "frozen_root": D, "chain_digest": D}
ESQUEMA = {
    "acta_digest": {"huella_clave": D, "esquema": U, "desde": U, "siguiente": D, "procedencia": "procedencia"},
    "acuse_digest": {"hash_prueba": D, "epoca": U, "n": U},
    "ancla_digest": {"huella_clave": D, "indice": U, "epoch_digest": D, "mmr_root": D, "mmr_size": U},
    "as_digest": {"x": U},
    "cima": {"hojas": "digests"},
    "digest_from_bytes": {"bytes": B},
    "digest_pi": {"familia": U, "version_air": U, "entradas_publicas": "u64les"},
    "digest_to_bytes": {"d": "sym", "x": U},
    "embeber": {"x": ULE},
    "epoch_digest": CABEZA,
    "epoch_digest_v2": {**CABEZA, "acuses_root": D, "n": U},
    "epoch_digest_v3": {**CABEZA, "acuses_root": D, "n": U, "cima_mmr": D, "t": U},
    "epoch_digest_v4": {**CABEZA, "acuses_root": D, "n": U, "cima_mmr": D, "t": U, "cons_root": D, "cons_count": U},
    "epoch_digest_v5": {**CABEZA, "acuses_root": D, "n": U, "cima_mmr": D, "t": U, "cons_root": D, "cons_count": U,
                        "params_digest": D, "pmeta_root": D, "next_pending": U, "next_index": U, "total_supply": U},
    "epoch_digest_v6": {**CABEZA, "acuses_root": D, "n": U, "cima_mmr": D, "t": U, "cons_root": D, "cons_count": U,
                        "params_digest": D, "pmeta_root": D, "next_pending": U, "next_index": U, "total_supply": U,
                        "recep_root": D, "recep_count": U},
    "hash_del_lote": {"operaciones": "ops"},
    "hoja_de_acuse": {"hash_prueba": D, "seq": U, "n": U},
    "huella_de_clave": {"clave": B},
    "mmr_hoja": {"cabeza": D},
    "mmr_nodo": {"izquierda": D, "derecha": D},
    "native_leaf": {"public_id": D, "balance": ULE, "nonce": ULE},
    "native_leaf_salted": {"public_id": D, "balance": ULE, "nonce": ULE, "leaf_salt": D},
    "native_merge": {"left": D, "right": D},
    "params_digest": {"regulatory_limit": U, "max_supply": U, "max_accounts": U, "custodian_set_root": D,
                      "governance_set_root": D, "refund_ttl": U, "max_custodian_uses": U},
    "path_root": {"leaf": D, "siblings": "digests", "is_right": "bools"},
    "preambulo": {"version": I, "epoch_digest": D},
    "preambulo_acta": {"version": I, "acta_digest": D},
    "preambulo_cofirma": {"version": I, "epoch_digest": D, "clave_del_operador": B},
    "preambulo_cofirma_v7": {"esquema_testigo": U, "esquema_operador": U, "epoch_digest": D, "clave_del_operador": B},
    "preambulo_v7": {"esquema": U, "epoch_digest": D},
    "recibo_digest": {"hash_prueba": D, "era": U, "n": U},
    "recibo_digest_v2": {"hash_prueba": D, "digest_pi": D, "era": U, "n": U},
    "registro_de_esquemas": {},
    "registro_de_familias": {"m": U, "columnas": U},
}


def hexbytes(s):
    s = s[2:] if s.startswith("0x") else s
    return bytes.fromhex(s)


def leer(valor, tipo):
    if tipo == U:
        return int(valor, 16)
    if tipo == ULE:
        b = hexbytes(valor)
        assert len(b) == 8, "u64 little-endian de ocho bytes"
        return int.from_bytes(b, "little")
    if tipo == D:
        return N.digest_from_bytes(hexbytes(valor))
    if tipo == B:
        return hexbytes(valor)
    if tipo == I:
        return int(valor)
    if tipo == "digests":
        return [N.digest_from_bytes(hexbytes(v)) for v in valor]
    if tipo == "u64les":
        return [leer(v, ULE) for v in valor]
    if tipo == "bools":
        return [bool(v) for v in valor]
    if tipo == "ops":
        return [(hexbytes(o["hash_prueba"]), int(o["cuenta"], 16), int(o["posicion"], 16)) for o in valor]
    if tipo == "sym":
        return valor
    if tipo == "procedencia":
        if valor is None:
            return None
        return (N.digest_from_bytes(hexbytes(valor["anterior"])), N.digest_from_bytes(hexbytes(valor["epoch_digest"])),
                N.digest_from_bytes(hexbytes(valor["mmr_root"])), int(valor["mmr_size"], 16))
    raise ValueError(tipo)


def hx(b):
    return "0x" + bytes(b).hex()


def dig(d):
    return hx(N.digest_to_bytes(d))


def registro_en_json(registro):
    """El registro con la escritura del KAT: el numero como QUANTITY, el prefijo en hex."""
    return [{"esquema": hex(x["esquema"]), "nombre": x["nombre"], "clave_bytes": x["clave_bytes"],
             "clave_prefijo": hx(x["clave_prefijo"]), "firma_bytes": x["firma_bytes"],
             "preambulo_adjunto": x["preambulo_adjunto"], "con_estado": x["con_estado"]}
            for x in registro]


def registro_de_json(lista):
    return [{**x, "esquema": int(x["esquema"], 16), "clave_prefijo": hexbytes(x["clave_prefijo"])} for x in lista]


def familias_en_json(registro, m, columnas):
    """El registro de familias con la escritura del KAT: los numeros como QUANTITY, la marca en hex."""
    return [{"familia": hex(f), "air": air, "version": hex(N.VERSION_INICIAL), "perfil": hex(N.PERFIL_PRODUCCION),
             "marca": hx(N.marca_v2(m, f, N.VERSION_INICIAL, N.PERFIL_PRODUCCION, columnas))}
            for f, air in registro]


def familias_de_json(lista):
    return [(int(x["familia"], 16), x["air"]) for x in lista]


def cabeza_v1(e):
    return N.epoch_digest(e["seq"], e["accounts_root"], e["pending_root"], e["frozen_root"], e["chain_digest"])


def calcular(fn, e, notas):
    if fn == "acta_digest":
        return dig(N.acta_digest(e["huella_clave"], e["esquema"], e["desde"], e["siguiente"], e["procedencia"]))
    if fn == "acuse_digest":
        return dig(N.acuse_digest(e["hash_prueba"], e["epoca"], e["n"]))
    if fn == "ancla_digest":
        return dig(N.ancla_digest(e["huella_clave"], e["indice"], e["epoch_digest"], e["mmr_root"], e["mmr_size"]))
    if fn == "as_digest":
        return dig(N.as_digest(e["x"]))
    if fn == "cima":
        return dig(N.cima(e["hojas"], con_hoja=True))
    if fn == "digest_from_bytes":
        return dig(N.digest_from_bytes(e["bytes"]))
    if fn == "digest_pi":
        return hx(N.digest_pi(e["familia"], e["version_air"], e["entradas_publicas"]))
    if fn == "digest_to_bytes":
        assert e["d"] == "as_digest(x)", "el KAT nombra d simbolicamente como as_digest(x)"
        return hx(N.digest_to_bytes(N.as_digest(e["x"])))
    if fn == "embeber":
        return dig(N.embeber(e["x"]))
    if fn == "epoch_digest":
        return dig(cabeza_v1(e))
    if fn.startswith("epoch_digest_v"):
        v = N.epoch_digest_v2(cabeza_v1(e), e["acuses_root"], e["n"])
        if fn >= "epoch_digest_v3":
            v = N.epoch_digest_v3(v, e["cima_mmr"], e["t"])
        if fn >= "epoch_digest_v4":
            v = N.epoch_digest_v4(v, e["cons_root"], e["cons_count"])
        if fn >= "epoch_digest_v5":
            v = N.epoch_digest_v5(v, e["params_digest"], e["pmeta_root"], e["next_pending"], e["next_index"], e["total_supply"])
        if fn >= "epoch_digest_v6":
            v = N.epoch_digest_v6(v, e["recep_root"], e["recep_count"])
        return dig(v)
    if fn == "hash_del_lote":
        return hx(N.hash_del_lote(e["operaciones"]))
    if fn == "hoja_de_acuse":
        return dig(N.hoja_de_acuse(e["hash_prueba"], e["seq"], e["n"]))
    if fn == "huella_de_clave":
        return hx(N.huella_de_clave(e["clave"]))
    if fn == "mmr_hoja":
        return dig(N.mmr_hoja(e["cabeza"]))
    if fn == "mmr_nodo":
        return dig(N.mmr_nodo(e["izquierda"], e["derecha"]))
    if fn == "native_leaf":
        return dig(N.native_leaf(e["public_id"], e["balance"], e["nonce"]))
    if fn == "native_leaf_salted":
        return dig(N.native_leaf_salted(e["public_id"], e["balance"], e["nonce"], e["leaf_salt"]))
    if fn == "native_merge":
        return dig(N.native_merge(e["left"], e["right"]))
    if fn == "params_digest":
        return dig(N.params_digest(e["regulatory_limit"], e["max_supply"], e["max_accounts"], e["custodian_set_root"],
                                   e["governance_set_root"], e["refund_ttl"], e["max_custodian_uses"]))
    if fn == "path_root":
        return dig(N.path_root(e["leaf"], e["siblings"], e["is_right"]))
    if fn == "preambulo":
        return hx(N.preambulo(e["version"], e["epoch_digest"]))
    if fn == "preambulo_acta":
        return hx(N.preambulo_acta(e["version"], e["acta_digest"]))
    if fn == "preambulo_cofirma":
        return hx(N.preambulo_cofirma(e["version"], e["epoch_digest"], e["clave_del_operador"]))
    if fn == "preambulo_cofirma_v7":
        return hx(N.preambulo_cofirma_v7(e["esquema_testigo"], e["esquema_operador"], e["epoch_digest"],
                                         e["clave_del_operador"]))
    if fn == "preambulo_v7":
        return hx(N.preambulo_v7(e["esquema"], e["epoch_digest"]))
    if fn == "registro_de_esquemas":
        propios = N.registro_valido(N.REGISTRO_DE_ESQUEMAS)
        if propios:
            raise ValueError(f"el registro de nucleo.py no cumple sus reglas: {propios}")
        gemela = {**N.REGISTRO_DE_ESQUEMAS[0], "esquema": N.REGISTRO_DE_ESQUEMAS[0]["esquema"] + (6 << 32)}
        if not N.registro_valido(N.REGISTRO_DE_ESQUEMAS + [gemela]):
            raise ValueError("falsador: una entrada gemela que comparte la escritura de la clave NO cae")
        # §862 · los dos conjuntos del despacho (RFC-0020, E3) y el implicito, contra el registro, con
        #    su falsador: un conjunto aceptado vacio tiene que caer.
        conjuntos = N.conjuntos_validos(N.REGISTRO_DE_ESQUEMAS)
        if conjuntos:
            raise ValueError(f"los conjuntos del despacho de nucleo.py no cumplen su regla: {conjuntos}")
        if not N.conjuntos_validos(N.REGISTRO_DE_ESQUEMAS, aceptados=()):
            raise ValueError("falsador: un conjunto aceptado vacio NO cae")
        return registro_en_json(N.REGISTRO_DE_ESQUEMAS)
    if fn == "registro_de_familias":
        propios = N.registro_de_familias_valido(N.REGISTRO_DE_FAMILIAS)
        if propios:
            raise ValueError(f"el registro de familias de nucleo.py no cumple sus reglas: {propios}")
        gemela = N.REGISTRO_DE_FAMILIAS + [(len(N.REGISTRO_DE_FAMILIAS) + 1, N.REGISTRO_DE_FAMILIAS[0][1])]
        if not N.registro_de_familias_valido(gemela):
            raise ValueError("falsador: una AIR con dos numeros NO cae")
        return familias_en_json(N.REGISTRO_DE_FAMILIAS, e["m"], e["columnas"])
    if fn == "recibo_digest":
        return dig(N.recibo_digest(e["hash_prueba"], e["era"], e["n"]))
    if fn == "recibo_digest_v2":
        return dig(N.recibo_digest_v2(e["hash_prueba"], e["digest_pi"], e["era"], e["n"]))
    raise KeyError(fn)


def juzgar(directorio):
    ficheros = sorted(f for f in os.listdir(directorio) if f.endswith(".json"))
    ok, notas, vistos, verde = 0, [], set(), {}
    for f in ficheros:
        kat = json.load(open(os.path.join(directorio, f)))
        fn, salida = kat["fn"], kat["salida"]
        salida = salida.lower() if isinstance(salida, str) else salida
        vistos.add(fn)
        if fn not in ESQUEMA:
            print(f"ROJO {fn}: el juez no conoce esta fn (fichero {f})")
            continue
        try:
            e = {k: leer(v, ESQUEMA[fn][k]) for k, v in kat["entradas"].items()}
            faltan = set(ESQUEMA[fn]) - set(e)
            if faltan:
                raise KeyError(f"entradas que faltan: {sorted(faltan)}")
            got = calcular(fn, e, notas)
            if fn == "registro_de_esquemas":
                ajenos = N.registro_valido(registro_de_json(salida))
                if ajenos:
                    raise ValueError(f"el registro del KAT no cumple sus reglas: {ajenos}")
            if fn == "registro_de_familias":
                ajenos = N.registro_de_familias_valido(familias_de_json(salida))
                if ajenos:
                    raise ValueError(f"el registro de familias del KAT no cumple sus reglas: {ajenos}")
        except Exception as ex:  # un KAT que no se puede leer es un ROJO con nombre
            print(f"ROJO {fn}: {type(ex).__name__}: {ex}")
            continue
        verde[fn] = got == salida
        if got == salida:
            ok += 1
            print(f"OK   {fn}")
        else:
            print(f"ROJO {fn}: esperado {str(salida)[:18]}.. obtenido {str(got)[:18]}..")
            if fn == "cima":
                alt = dig(N.cima(e["hojas"], con_hoja=False))
                notas.append("cima: con las hojas CRUDAS da " + ("el KAT" if alt == salida else "otra cosa"))
    for fn in sorted(set(ESQUEMA) - vistos):
        print(f"AVISO {fn}: el juez la conoce y no hay KAT")
    return ok, len(ficheros), notas, verde


def main(argv):
    if len(argv) > 2:
        print(__doc__.strip().splitlines()[3].strip(), file=sys.stderr)
        return 2
    directorio = argv[1] if len(argv) == 2 else os.path.join("spec", "vectors", "nucleo")
    if not os.path.isdir(directorio):
        print(f"no existe el directorio {directorio}", file=sys.stderr)
        return 2
    fallos = N.autotest()
    if fallos:
        print("autotest de las primitivas: ROJO, ningun veredicto vale")
        [print("  ", x) for x in fallos]
        return 3
    print("autotest de las primitivas (BLAKE3 contra blake3 1.8.5; Rescue contra el vector Sage de winter-crypto): VERDE")
    ok, n, notas, verde = juzgar(directorio)
    notas.append("formato: embeber.x, balance/nonce de native_leaf* y las entradas_publicas de digest_pi van como u64 en ocho bytes LE; "
                 "seq, n, t, los contadores y as_digest.x como numero hex (dos codificaciones, un catalogo)")
    notas.append("formato: registro_de_esquemas no es de una fn; su salida es la lista de las entradas del registro, "
                 "con el esquema como numero hex, el prefijo de la clave en hex, los largos como numero y dos booleanos")
    notas.append("formato: registro_de_familias tampoco es de una fn; su salida es la lista de las 23 familias de la "
                 "marca v2 (NUCLEO.md seccion 4 bis), con la familia, la version y el perfil como numero hex, la AIR por "
                 "su nombre y los 25 bytes de la marca en hex; la m y las columnas de la casa van como entradas")
    notas.append("hoja_de_acuse: RPC.md (zkssl_ackPath) la da como acuse_digest(hashPrueba, epoca, n) y RPC.md (El acuse "
                 "en la respuesta) dice epoca = logSeq + 1; NUCLEO.md seccion 6 lo escribe desde el S622; "
                 "con seq + 1 el KAT " + ("la sostiene" if verde.get("hoja_de_acuse") else "NO la sostiene")
                 + "; con seq, no (probado)")
    notas.append("hash_del_lote: la longitud codificada es la de los bytes de la composicion (48*k), no k; "
                 "<<k va en la longitud>> se lee como <<k queda implicito en la longitud>>")
    notas.append("cima: NUCLEO.md seccion 6 no dice si cada hoja pasa por mmr_hoja dentro de la cima; "
                 "aqui si (lectura RFC 6962)" + (", y el KAT lo sostiene" if verde.get("cima") else ""))
    for x in notas:
        print("NOTA", x)
    sha = hashlib.sha256(open(N.__file__, "rb").read()).hexdigest()[:16]
    print(f"nucleo: {ok} de {n} vectores reproducidos byte a byte - implementacion {sha}")
    return 0 if ok == n and n > 0 else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
