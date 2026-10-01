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

Lo que el formato de los KAT no dice y este juez tiene que saber (NOTA al final): que `embeber.x`
y `balance`/`nonce` de `native_leaf*` son un u64 en OCHO BYTES little-endian, mientras `seq`, `n`,
`t`, los contadores y `x` de `as_digest` son el u64 como numero hex. Son dos codificaciones del
mismo tipo en el mismo catalogo.
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
    "recibo_digest": {"hash_prueba": D, "era": U, "n": U},
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
    if fn == "recibo_digest":
        return dig(N.recibo_digest(e["hash_prueba"], e["era"], e["n"]))
    raise KeyError(fn)


def juzgar(directorio):
    ficheros = sorted(f for f in os.listdir(directorio) if f.endswith(".json"))
    ok, notas, vistos, verde = 0, [], set(), {}
    for f in ficheros:
        kat = json.load(open(os.path.join(directorio, f)))
        fn, salida = kat["fn"], kat["salida"].lower()
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
        except Exception as ex:  # un KAT que no se puede leer es un ROJO con nombre
            print(f"ROJO {fn}: {type(ex).__name__}: {ex}")
            continue
        verde[fn] = got == salida
        if got == salida:
            ok += 1
            print(f"OK   {fn}")
        else:
            print(f"ROJO {fn}: esperado {salida[:18]}.. obtenido {got[:18]}..")
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
    notas.append("formato: embeber.x y balance/nonce de native_leaf* van como u64 en ocho bytes LE; "
                 "seq, n, t, los contadores y as_digest.x como numero hex (dos codificaciones, un catalogo)")
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
