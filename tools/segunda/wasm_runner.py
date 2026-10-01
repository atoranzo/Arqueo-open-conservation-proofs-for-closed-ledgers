#!/usr/bin/env python3
"""Corre el verificador de Arqueo compilado a `wasm32-wasip1` dentro de wasmtime, con el contrato del
mando (PAQUETE.md, seccion 6): un argumento, la ruta de un sobre; exit 0, 1, 2 o 3; stdout y stderr
tal cual. Es el envoltorio que `tools/conformidad.sh` necesita para juzgar el `.wasm` como juzga
al binario nativo. Spike de `doc/integracion-vertical-evaluacion.md`, 5.3.

    python3 tools/segunda/wasm_runner.py --precompilar <modulo.wasm> <salida.cwasm>
    python3 tools/segunda/wasm_runner.py <modulo.cwasm> <sobre.json>

Necesita el paquete `wasmtime` de PyPI (`python3 -m pip install wasmtime`): es el motor wasmtime
con sus bindings, no una reimplementacion. Preabre `.` y `/` para que las rutas relativas y
absolutas del manifiesto lleguen igual que al binario nativo (una u otra: ver `correr`).
"""
import sys

from wasmtime import Engine, ExitTrap, Linker, Module, Store, WasiConfig


def precompilar(wasm, cwasm):
    engine = Engine()
    Module.from_file(engine, wasm)  # valida
    with open(cwasm, "wb") as f:
        f.write(Module.from_file(engine, wasm).serialize())


def correr(cwasm, args):
    engine = Engine()
    store = Store(engine)
    wasi = WasiConfig()
    wasi.argv = ["zk-ssl-verify"] + list(args)
    wasi.inherit_stdout()
    wasi.inherit_stderr()
    # MEDIDO: con `.` y `/` preabiertos a la vez, wasi-libc no resuelve las rutas relativas (ENOENT,
    # os error 44); con `.` solo, si. Se preabre lo que las rutas de la llamada necesitan.
    if any(a.startswith("/") for a in args):
        wasi.preopen_dir("/", "/")
    else:
        wasi.preopen_dir(".", ".")
    store.set_wasi(wasi)
    linker = Linker(engine)
    linker.define_wasi()
    module = Module.deserialize_file(engine, cwasm)
    instancia = linker.instantiate(store, module)
    try:
        instancia.exports(store)["_start"](store)
    except ExitTrap as salida:
        return salida.code
    return 0


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "--precompilar":
        precompilar(sys.argv[2], sys.argv[3])
        raise SystemExit(0)
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(correr(sys.argv[1], sys.argv[2:]))
