#!/usr/bin/env python3
"""El juez de la CADENA del registro (§797): recompone, desde sus entradas, cada `chain` de los vectores
del cable con `tools/segunda/cable.py`, escrito desde spec/RPC.md (<<La composicion de `chain`, byte a
byte>>), y dice entrada a entrada si cuadra.

    python3 tools/segunda/juez_cadena.py [fichero.json ...]    (por defecto spec/vectors/zkssl-0.4.json)

⚠️ Esto juzga el CABLE, no el nucleo: `chain` es de la clase REGISTRO (NUCLEO.md, <<2. Las cuatro
clases>>; RFC-0005 D-A), y este juez no cuenta en el censo del nucleo. Tampoco
sustituye a `conformance --check`, que vuelve a correr el probador: aqui no se prueba nada, se
recompone lo que el fichero ya trae. ⚠️ §851: aqui decia «este juez no esta en el canon ni cuenta en
el censo del nucleo»; desde el §851 el canon lo corre (tools/canon.sh, bloque «3 duodecies») sobre
spec/vectors/zkssl-0.*.json, a pin cero.

Lee dos formas, cada una con su grafia y ninguna mezclada:
  - el VECTOR de conformidad: un objeto con `spec` y `entradas` (`Entrada` y `Vectores` de
    crates/zk-ssl-cli/src/conformance.rs), con las claves en snake_case: `root_old`, `root_new`,
    `proof_digest`;
  - el CABLE: la lista `LogEntry[]` que devuelve `zkssl_logEntries` (`LogEntryDto`,
    crates/zk-ssl-wire/src/lib.rs, `rename_all = "camelCase"`): `rootOld`, `rootNew`, `proofDigest`.
Los valores se escriben igual en las dos: `0x` y el hex en minuscula de `digest_to_bytes` (`hexd` en
conformance.rs, `B32` en el cable), y la QUANTITY con `0x{:x}` (`q`, `Q`).

Reglas, tomadas de spec/RPC.md. R1 a R7 se escribieron ANTES de mirar lo que este juez dice de los
vectores (un prototipo desechable recompuso las 24 cadenas de los cuatro antes de escribirlo). R0 y
lo que llevan marcado R1, R6 y R7 vienen de la revision del mismo sello (§797); ninguno cambia el
veredicto sobre los cuatro vectores, que dan lo mismo antes y despues:
  R0  (§797) la forma: un JSON que no se lee, una raiz que no es ni el objeto del vector ni la lista
      del cable, una entrada que no es un objeto, una clave repetida, una de la otra grafia o una
      que no es de las siete, es ROJO con el nombre del fichero. `LogEntryDto` las rechaza igual
      (`deny_unknown_fields`, y la derivacion de serde, la repetida); `Entrada` del vector
      (crates/zk-ssl-cli/src/conformance.rs:37-51, sin `deny_unknown_fields`) rechaza la repetida,
      pero ignora las ajenas y las de la otra grafia (si no le falta la suya). Aqui se rechazan
      todas: una clave que la cadena no ata no pasa en silencio.
  R1  los `seq` son consecutivos desde 0, en hex canonico. (§797) Canonico es la escritura de
      `cantidad_canonica` (§662): `0x`, de 1 a 16 cifras [0-9a-f], sin ceros a la izquierda. La
      primera redaccion leia con `int(s, 16)`, que admite `0x1_0` y cifras Unicode: `0x١` era
      el 1.
  R2  `root_old` de una entrada es `root_new` de la anterior. La raiz del genesis viaja, como
      `root_old` de la entrada seq 0x0, pero el fichero no trae otra raiz de genesis, independiente,
      con la que contrastarla (`verify` la recibe de fuera, crates/zk-ssl/src/log.rs:465): R2
      empieza en la segunda entrada (declarado).
  R3  `chain` = chain_v1 si la entrada no trae `compromiso` y chain_v2 si lo trae, con
      `anterior` = el `chain` de la entrada anterior, y el digest cero para la primera.
  R4  `kind` es uno de los trece; un `kind` sin etiqueta no se recompone.
  R5  en la era 2, las clases sin compromiso propio llevan COMPROMISO_AUSENTE, y una delegada NO
      lo lleva.
  R6  `proof_digest`, donde la clase lo fija sin la prueba, es el que fija; donde no, queda
      DECLARADO. (§797) Solo en la ERA 2: la columna `proofDigest` de la tabla de RPC.md vale desde
      el §278, y una entrada de era 1 no dice de que lado cae. En la era 1 el valor depende de
      cuando se asento -hasta el §209 el hash algebraico (`zkssl/0.1`); del §209 al §278, la
      prueba vacia en `OpenAccount` y en las cinco delegadas; del §278 al §281, los sellos, sin el
      compromiso en la entrada-: R6 la declara, y dice con que valor conocido coincide, si
      coincide con alguno. La primera redaccion solo eximia `zkssl/0.1`, y una `OpenAccount` de
      era 1 del §209 al §278 daba un ROJO falso.
  R7  cada digest se lee canonico (RFC-0016): un elemento >= p es ROJO, no se reduce. (§797) Y su
      texto, el de `hex_canonico`: `0x` y [0-9a-f] por pares. La primera redaccion leia con
      `bytes.fromhex`, que admite mayusculas y espacios. (§797) Y `"compromiso": null` es ROJO por
      R7, aunque la entrada recomponga como era 1: aqui la era 1 es la clave AUSENTE. Es mas
      estricto que el DTO, que lee el `null` como la ausencia (`Option<B32>` con `default`,
      crates/zk-ssl-wire/src/lib.rs:559-560); el nodo no lo escribe nunca (`skip_serializing_if`).

Lo que este juez NO ve, y lo dice en su salida:
  - ⚠️ un registro TRUNCADO ni uno REESCRITO de forma coherente: quitando la ultima entrada de
    zkssl-0.4.json sale 5 de 5. Recomponer dice que el fichero es coherente consigo mismo; que su
    ultima `chain` sea la que el nodo firmo lo diria compararla con el `chainDigest` de una cabeza
    firmada, y este juez no lo hace (NOTA por fichero).
  - ⚠️ lo que ningun fichero ejercita: con los cuatro vectores del cable, solo cuatro de las
    trece etiquetas (OpenAccount, Mint, Send, Claim) y, de R6, solo el sello de ausencia
    (OpenAccount) y el de autorizacion con Mint. Una errata en la etiqueta o en la fila de R6 de
    una clase sin entrada no la ve: con la etiqueta de `Consumo` cambiada a 14 en una copia de
    cable.py, los cuatro siguen dando 24 de 24. La lista sale, calculada, en las lineas
    `SIN EJERCITAR`.
  - que las eras sean monotonas: una entrada de era 1 detras de una de era 2 no la escribe ya
    `append` (todo asiento es era 2, crates/zk-ssl/src/log.rs:366-408: `append` delega con el
    centinela, :373, y el unico `entries.push` asienta `chain_digest_v2`, :397, y
    `compromiso: Some`, :403), pero ni `verify_chain` (log.rs:420-447) ni este juez la rechazan;
    sale como NOTA, no como ROJO.
  - un tramo de `zkssl_logEntries` que no empiece en seq 0 (`fromSeq`): sin el `chain` de la
    entrada anterior no hay con que encadenar la primera, y R1 lo da ROJO.

Salida: `OK`, `DECLARADO` o `ROJO <fichero> seq <seq> (<kind>): <motivo>` por entrada; los
FALSADORES, que alteran un campo cada uno -o el orden del ultimo merge- y exigen que la cadena deje
de recomponerse; las NOTA por fichero; las SIN EJERCITAR; y al final `cadena: <ok> de <n> cadenas
recompuestas ...` si todas, o `ROJO cadena: ...` con el nombre de cada una que no. Exit 0 solo si
todas las cadenas recomponen y ningun R ni falsador sale ROJO; 1 si no; 2 uso; 3 si el autotest de
las primitivas falla, porque entonces ningun veredicto vale. (§797) El falsador del anterior pone a
cero el de la ultima entrada; si la ultima es seq 0x0, cuyo anterior ya es el cero, pone [1, 0, 0,
0]. La primera redaccion lo ponia a cero siempre, y a todo fichero de una sola entrada le daba un
ROJO FALSADOR falso.
"""
import hashlib
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cable as C  # noqa: E402
import nucleo as N  # noqa: E402

USO = "python3 tools/segunda/juez_cadena.py [fichero.json ...]    (por defecto spec/vectors/zkssl-0.4.json)"
RAIZ = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
POR_DEFECTO = os.path.join(RAIZ, "spec", "vectors", "zkssl-0.4.json")

# R0: las dos grafias. El orden de la tupla es el de `leer`.
GRAFIA_VECTOR = ("seq", "kind", "root_old", "root_new", "proof_digest", "chain")
GRAFIA_CABLE = ("seq", "kind", "rootOld", "rootNew", "proofDigest", "chain")
OPCIONAL = "compromiso"

HEX_DIGEST = re.compile(r"0x[0-9a-f]{64}")  # R7: hex_canonico de 32 bytes
HEX_Q = re.compile(r"0x(0|[1-9a-f][0-9a-f]{0,15})")  # R1: cantidad_canonica


class Rojo(Exception):
    """Un fallo de R0, con su motivo."""


def sin_repetidas(pares):
    """R0: una clave repetida es ROJO (json.load se quedaria con la ultima, sin decirlo)."""
    vistas = {}
    for k, v in pares:
        if k in vistas:
            raise Rojo(f"R0: clave repetida: {k!r}")
        vistas[k] = v
    return vistas


def dig(s):
    if not isinstance(s, str) or not HEX_DIGEST.fullmatch(s):
        raise ValueError(f"R7: digest no canonico (0x y 64 cifras [0-9a-f]): {str(s)[:20]!r}")
    return N.digest_from_bytes(bytes.fromhex(s[2:]))  # R7: < p o ValueError con el texto del nucleo


def q(s):
    if not isinstance(s, str) or not HEX_Q.fullmatch(s):
        raise ValueError(f"R1: QUANTITY no canonica: {str(s)[:20]!r}")
    return N.u64_canonico(int(s[2:], 16))


def hx(d):
    return "0x" + N.digest_to_bytes(d).hex()


def leer(e, grafia):
    """Una entrada, leida: (seq, kind, root_old, root_new, proof_digest, chain, compromiso)."""
    if not isinstance(e, dict):
        raise Rojo(f"R0: la entrada no es un objeto, es {type(e).__name__}")
    otra = GRAFIA_CABLE if grafia is GRAFIA_VECTOR else GRAFIA_VECTOR
    ajenas = sorted(k for k in e if k not in grafia and k != OPCIONAL)
    if ajenas:
        de_la_otra = [k for k in ajenas if k in otra]
        raise Rojo(f"R0: claves {'de la otra grafia' if de_la_otra else 'que la cadena no ata'}: {ajenas}")
    faltan = [k for k in grafia if k not in e]
    if faltan:
        raise Rojo(f"R0: faltan {faltan}")
    kind = e[grafia[1]]
    if not isinstance(kind, str):
        raise Rojo(f"R0: kind no es una cadena: {kind!r}")
    compromiso = dig(e[OPCIONAL]) if OPCIONAL in e else None
    return (q(e[grafia[0]]), kind, dig(e[grafia[2]]), dig(e[grafia[3]]), dig(e[grafia[4]]),
            dig(e[grafia[5]]), compromiso)


def cargar(ruta):
    """R0: (spec, entradas, grafia, vec) o Rojo con el motivo."""
    try:
        with open(ruta, "rb") as f:
            vec = json.loads(f.read().decode("utf-8"), object_pairs_hook=sin_repetidas)
    except Rojo:
        raise
    except (OSError, UnicodeDecodeError, ValueError) as ex:  # JSONDecodeError es un ValueError
        raise Rojo(f"R0: no es un JSON legible: {type(ex).__name__}: {ex}")
    if isinstance(vec, list):
        return "?", vec, GRAFIA_CABLE, {}
    if isinstance(vec, dict) and isinstance(vec.get("entradas"), list):
        spec = vec.get("spec") if isinstance(vec.get("spec"), str) else "?"
        return spec, vec["entradas"], GRAFIA_VECTOR, vec
    raise Rojo("R0: la raiz no es ni el objeto del vector (con `entradas`, una lista) ni la lista "
               f"LogEntry[] del cable: es {type(vec).__name__}")


def juzgar_fichero(ruta, total):
    nombre = os.path.basename(ruta)
    try:
        spec, entradas, grafia, vec = cargar(ruta)
    except Rojo as ex:
        print(f"ROJO {nombre}: {ex}")
        total["rojos"].append(f"{nombre} (forma)")
        return [], False
    if not entradas:
        print(f"ROJO {nombre}: el fichero no trae entradas")
        total["rojos"].append(f"{nombre} (sin entradas)")
        return [], False
    forma = "vector (snake_case)" if grafia is GRAFIA_VECTOR else "cable, LogEntry[] (camelCase)"
    leidas, anterior, root_prev, verde, era_prev = [], C.CERO, None, True, None
    for i, e in enumerate(entradas):
        seq_txt = e.get("seq", "?") if isinstance(e, dict) else "?"
        kind_txt = e.get("kind", "?") if isinstance(e, dict) else "?"
        etq = f"{nombre} seq {str(seq_txt)[:20]} ({str(kind_txt)[:20]})"
        total["n"] += 1
        try:
            seq, kind, ro, rn, pd, ch, comp = leer(e, grafia)
        except (Rojo, ValueError) as ex:  # una entrada que no se puede leer es un ROJO con nombre
            print(f"ROJO {etq}: {ex}")
            total["rojos"].append(etq)
            anterior, root_prev, verde = None, None, False  # la siguiente no tiene con que encadenar
            continue
        motivos = []
        if seq != i:
            motivos.append(f"R1: seq {seq:#x} en la posicion {i}")
        if root_prev is not None and ro != root_prev:
            motivos.append("R2: root_old no es el root_new de la anterior")
        if kind not in C.ETIQUETA:
            motivos.append(f"R4: kind {kind!r} sin etiqueta")
        recompuesta = False
        if kind in C.ETIQUETA and anterior is not None:
            got = C.chain(seq, kind, ro, rn, pd, anterior, comp)
            recompuesta = got == ch
            if not recompuesta:
                motivos.append(f"R3: chain esperado {hx(ch)} obtenido {hx(got)}")
        elif anterior is None:
            motivos.append("R2 y R3: la anterior no se pudo leer, no hay con que encadenar")
        if comp is not None:
            esperado = C.compromiso_esperado(kind)
            if esperado is not None and comp != esperado:
                motivos.append(f"R5: {kind} no tiene compromiso propio y no lleva el centinela")
            if esperado is None and comp == C.COMPROMISO_AUSENTE:
                motivos.append(f"R5: {kind} tiene compromiso propio y lleva el centinela")
        era = 1 if comp is None else 2
        if era == 1 and era_prev == 2:
            print(f"NOTA {etq}: era 1 detras de una de era 2: `append` ya no la escribe, pero ni "
                  f"verify_chain ni este juez la rechazan")
        era_prev = era
        pd_esp, declarado = C.proof_digest_esperado(kind, comp)
        if era == 1:
            if spec == "zkssl/0.1":
                declarado = "zkssl/0.1: hash algebraico de antes del S209"
            coincide = C.coincidencia_era1(pd)
            if coincide:
                declarado += f"; coincide con {coincide}"
        elif pd_esp is not None:
            if pd_esp != pd:
                motivos.append(f"R6: proof_digest esperado {hx(pd_esp)} y el fichero trae {hx(pd)}")
            elif kind in C.ETIQUETA:
                total["r6"].add(kind)
        total["era"][era] += 1
        if recompuesta:
            total["ok"] += 1
            total["etq"].add(kind)
        if motivos:
            print(f"ROJO {etq}: " + "; ".join(motivos))
            total["rojos"].append(etq)
            verde = False
        else:
            if declarado is None:
                total["pd_ok"] += 1
                print(f"OK   {etq}: era {era}, chain y proof_digest recompuestos")
            else:
                total["pd_decl"] += 1
                print(f"OK   {etq}: era {era}, chain recompuesta")
                print(f"DECLARADO {etq}: proof_digest no se recompone: {declarado}")
        leidas.append((seq, kind, ro, rn, pd, ch, comp))
        anterior, root_prev = ch, rn
    print(f"NOTA {nombre}: leido como {forma}")
    if "epoch_digest" in vec:
        print(f"NOTA {nombre}: epoch_digest no se recompone desde el fichero: la cabeza pide pendingRoot y "
              f"frozenRoot, y el fichero no los trae (conformance.rs, `recolectar`, la compone con la capa entera)")
    print(f"NOTA {nombre}: la raiz del genesis viaja como root_old de seq 0x0, pero no hay otra, "
          f"independiente, con que contrastarla: R2 empieza en seq 0x1")
    if leidas:
        print(f"NOTA {nombre}: la ultima chain (seq {leidas[-1][0]:#x}) no se contrasta con ninguna cabeza "
              f"firmada: un registro truncado, o reescrito coherente, sale igual de verde")
    return leidas, verde


def falsadores(leidas, nombre):
    """Cada uno altera UN campo de una entrada que recompone, o el orden de su ultimo merge, y
    exige que deje de recomponer.
    Un falsador que sigue recomponiendo es un juez ciego: ROJO."""
    if not leidas:
        return []
    rojos = []

    def una(i):
        seq, kind, ro, rn, pd, ch, comp = leidas[i]
        anterior = leidas[i - 1][5] if i > 0 else C.CERO
        return seq, kind, ro, rn, pd, ch, comp, anterior

    k = len(leidas) - 1  # la ultima: la que arrastra a todas
    seq, kind, ro, rn, pd, ch, comp, ant = una(k)
    otro_kind = next(x for x in C.ETIQUETA if x != kind)
    pd_volteado = [pd[0] ^ 1] + list(pd[1:])
    # (§797) Si la ultima es seq 0x0 su anterior YA es el cero, y ponerlo a cero no altera nada: la
    # primera redaccion daba ahi un ROJO FALSADOR falso a todo fichero de una sola entrada.
    otro_ant, texto_ant = ((C.CERO, "el anterior a cero") if list(ant) != C.CERO else
                           ([1, 0, 0, 0], "el anterior [1, 0, 0, 0] en vez del cero de seq 0x0"))
    casos = [
        ("un bit del proof_digest volteado", lambda: C.chain(seq, kind, ro, rn, pd_volteado, ant, comp)),
        (f"kind {kind} cambiado por {otro_kind}", lambda: C.chain(seq, otro_kind, ro, rn, pd, ant, comp)),
        ("seq + 1", lambda: C.chain(seq + 1, kind, ro, rn, pd, ant, comp)),
        (texto_ant, lambda: C.chain(seq, kind, ro, rn, pd, otro_ant, comp)),
        ("root_old y root_new cruzadas", lambda: C.chain(seq, kind, rn, ro, pd, ant, comp)),
    ]
    if comp is not None:
        comp_volteado = [comp[0] ^ 1] + list(comp[1:])
        casos += [
            ("la era cambiada: la entrada de era 2 con chain_v1", lambda: C.chain_v1(seq, kind, ro, rn, pd, ant)),
            ("un bit del compromiso volteado", lambda: C.chain(seq, kind, ro, rn, pd, ant, comp_volteado)),
            ("el merge final al reves: merge(compromiso, chain_v1)",
             lambda: N.native_merge(comp, C.chain_v1(seq, kind, ro, rn, pd, ant))),
        ]
    else:
        casos += [
            ("la era cambiada: la entrada de era 1 con chain_v2 y el centinela",
             lambda: C.chain_v2(seq, kind, ro, rn, pd, ant, C.COMPROMISO_AUSENTE)),
            ("el merge final al reves: merge(anterior, ...)",
             lambda: N.native_merge(ant, N.native_merge(N.native_merge(
                 N.native_merge(N.as_digest(seq), N.as_digest(C.etiqueta(kind))), N.native_merge(ro, rn)), pd))),
        ]
    if ro == rn:
        casos = [c for c in casos if not c[0].startswith("root_old")]
    for texto, f in casos:
        if f() == ch:
            print(f"ROJO FALSADOR {nombre} seq {seq:#x}: {texto} y la cadena SIGUE recomponiendo: el juez no lo ve")
            rojos.append(f"falsador {texto}")
        else:
            print(f"FALSADOR {nombre} seq {seq:#x}: {texto}: la cadena deja de recomponerse")
    return rojos


def sin_ejercitar(total):
    """La ceguera, calculada: lo que ninguna entrada que recompuso ejercito."""
    etq = [f"{k} ({v})" for k, v in C.ETIQUETA.items() if k not in total["etq"]]
    if etq:
        print(f"SIN EJERCITAR: {len(etq)} de las {len(C.ETIQUETA)} etiquetas: {', '.join(etq)}. Ninguna entrada "
              f"que recompone las trae, y una errata en su numero no la ve este juez")
    fijas = ["OpenAccount", *C.DELEGADAS, "Consumo"]  # las clases cuyo proof_digest fija R6 en la era 2
    r6 = [k for k in fijas if k not in total["r6"]]
    if r6:
        print(f"SIN EJERCITAR: R6 en {len(r6)} de las {len(fijas)} clases que lo fijan sin la prueba: "
              f"{', '.join(r6)}. Su fila de la tabla de RPC.md no la contrasta ninguna entrada de era 2")


def main(argv):
    if any(a.startswith("-") for a in argv[1:]):
        print(USO, file=sys.stderr)
        return 2
    rutas = argv[1:] or [POR_DEFECTO]
    for r in rutas:
        if not os.path.isfile(r):
            print(f"no existe el fichero {r}", file=sys.stderr)
            return 2
    fallos = N.autotest()
    if fallos:
        print("autotest de las primitivas: ROJO, ningun veredicto vale")
        [print("  ", x) for x in fallos]
        return 3
    print("autotest de las primitivas (BLAKE3 contra blake3 1.8.5; Rescue contra el vector Sage de winter-crypto): VERDE")
    total = {"n": 0, "ok": 0, "pd_ok": 0, "pd_decl": 0, "era": {1: 0, 2: 0}, "rojos": [],
             "etq": set(), "r6": set()}
    for r in rutas:
        leidas, verde = juzgar_fichero(r, total)
        if verde:  # el falsador prueba que el juez no es ciego sobre un VERDE; sobre un ROJO sobra
            total["rojos"] += falsadores(leidas, os.path.basename(r))
        else:
            print(f"FALSADOR {os.path.basename(r)}: no se corre, el fichero ya sale ROJO")
    sin_ejercitar(total)
    sha = hashlib.sha256(open(C.__file__, "rb").read()).hexdigest()[:16]
    cola = (f"(era 1: {total['era'][1]}, era 2: {total['era'][2]}) - proof_digest: {total['pd_ok']} recompuestos, "
            f"{total['pd_decl']} declarados - {len(rutas)} fichero(s) - implementacion {sha}")
    if total["rojos"] or total["ok"] != total["n"]:
        print(f"ROJO cadena: {total['ok']} de {total['n']} cadenas recompuestas {cola}; "
              f"no cuadran: {', '.join(total['rojos'])}")
        return 1
    print(f"cadena: {total['ok']} de {total['n']} cadenas recompuestas {cola}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
