#!/usr/bin/env bash
# tools/target_dir.sh -- el directorio donde cargo deja los binarios de este arbol, con su ruta
# absoluta, en una linea (§855).
#
#   bash tools/target_dir.sh        # p. ej. /home/yo/arqueo/target
#
# Lo lee de `cargo metadata`, su `target_directory`. Asi sigue lo mismo que sigue cargo:
# CARGO_TARGET_DIR, CARGO_BUILD_TARGET_DIR y el `build.target-dir` de un `.cargo/config.toml`, y
# un valor relativo, desde la raiz. La forma del §844, `${CARGO_TARGET_DIR:-target}`, y la de
# Python, `os.environ.get('CARGO_TARGET_DIR') or 'target'`, solo seguian la primera, y relativa al
# directorio de quien llama. Con `--no-deps` y `--offline` no resuelve dependencias ni pide red:
# vale en un clon recien hecho, con la cache de cargo vacia, en menos de 0,1 s.
#
# Es el unico lector: lo llaman el canon (el paquete y las familias de sobres), el adaptador del
# cable (tools/cable_respuesta.sh), los bancos (tools/banco_*.sh, salvo el de los cortes, que fija
# el suyo) y los dos rehacer*.sh de crates/zk-ssl-medio/tests/vectores/testigos/.
#
# Si cargo no lo da, no imprime nada en stdout, deja el error de cargo en stderr y sale con 2.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 2
J=$(cargo metadata --format-version 1 --no-deps --locked --offline) \
  || { echo "tools/target_dir.sh: cargo metadata fallo: no se donde deja cargo los binarios" >&2; exit 2; }
python3 -c '
import json, os, sys
try:
    t = json.loads(sys.stdin.read())["target_directory"]
except (ValueError, KeyError, TypeError):
    t = None
if not isinstance(t, str) or not os.path.isabs(t):
    sys.exit("tools/target_dir.sh: cargo metadata no da un target_directory absoluto: %r" % (t,))
print(t)
' <<< "$J" || exit 2
