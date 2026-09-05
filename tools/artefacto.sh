#!/usr/bin/env bash
# tools/artefacto.sh — el ARTEFACTO de H2 (§401): el verificador distribuible.
#
# Compila `zk-ssl-verify` en release con `--remap-path-prefix` (la huella del binario no
# depende de la ruta del arbol ni del usuario: depende del toolchain y de `Cargo.lock`),
# monta el juego que un tercero necesita —binario, el arnes `conformidad.sh` (§408),
# `spec/PAQUETE.md`, el manifiesto y sus vectores, las licencias, `NOTICE`, `THIRD-PARTY.txt`,
# `VERSION`, `SHA256SUMS`— y lo
# empaqueta en un tarball REPRODUCIBLE bajo `target/artefacto/` (ignorado por git).
#
#     bash tools/artefacto.sh            # construye y deja target/artefacto/<nombre>.tar.gz
#     bash tools/artefacto.sh --check    # la PROPIEDAD, para el canon (§401, 3 ter): dos
#                                        # compilaciones en dos target dan la misma huella y
#                                        # cero rutas de la maquina; dos tarballs, la misma
#                                        # huella; el binario pasa el manifiesto entero.
#
# Lo que el binario exige: x86_64 Linux y la glibc que `VERSION` nombra (no es estatico).
# Lo que este fichero NO hace: publicar. La release en GitHub se sube a mano, con las huellas
# que este script imprime y el asiento registra. Exit 0 = VERDE; cualquier fallo, exit 1.
set -euo pipefail
cd "$(dirname "$0")/.."
RAIZ=$(pwd)
OUT="$RAIZ/target/artefacto"
VER=$(sed -n 's/^version = "\(.*\)"/\1/p' crates/zk-ssl-verify/Cargo.toml | head -n 1)
HOST=$(rustc -vV | awk '/^host:/{print $2}')
NOMBRE="arqueo-verify-${VER}-${HOST}"
REG="$HOME/.cargo/registry/src"
FLAGS="--remap-path-prefix=$RAIZ=/arqueo --remap-path-prefix=$REG=/registry --remap-path-prefix=$HOME/.cargo=/cargo --remap-path-prefix=$HOME=/home/x"
FLAGS_GENERICOS="--remap-path-prefix=<raiz>=/arqueo --remap-path-prefix=<home>/.cargo/registry/src=/registry --remap-path-prefix=<home>/.cargo=/cargo --remap-path-prefix=<home>=/home/x"
h16(){ sha256sum "$1" | cut -c1-16; }
rojo(){ echo "ROJO: $*" >&2; exit 1; }

construir(){ # $1 = target dir -> imprime la ruta del binario
  RUSTFLAGS="$FLAGS" CARGO_TARGET_DIR="$1" cargo build --release -q -p zk-ssl-verify --locked
  echo "$1/release/zk-ssl-verify"
}

glibc_de(){ strings -n 6 "$1" | grep -o 'GLIBC_[0-9.]*' | sort -uV | tail -n 1; }

montar(){ # $1 = binario, $2 = directorio destino
  rm -rf "$2"; mkdir -p "$2/spec/vectors/paquete"
  cp "$1" "$2/zk-ssl-verify"; chmod 755 "$2/zk-ssl-verify"
  cp spec/PAQUETE.md "$2/spec/"
  cp spec/vectors/paquete/MANIFIESTO.txt spec/vectors/paquete/*.json "$2/spec/vectors/paquete/"
  # §408 · el arnes viaja dentro: `bash conformidad.sh ./zk-ssl-verify` desde la raiz del tarball
  cp tools/conformidad.sh "$2/conformidad.sh"; chmod 755 "$2/conformidad.sh"
  cp LICENSE-APACHE LICENSE-MIT NOTICE "$2/"
  cargo tree -p zk-ssl-verify -e normal --prefix none --format '{p} {l}' --locked | sed 's/ (\/[^)]*)//; s/ (\*)//' | sort -u > "$2/THIRD-PARTY.txt"
  {
    echo "artefacto=$NOMBRE"
    echo "commit=$(git rev-parse HEAD)"
    echo "describe=$(git describe --tags --long --always)"
    rustc -vV | sed 's/^/rustc./'
    echo "cargo=$(cargo --version)"
    echo "rustflags=$FLAGS_GENERICOS"
    echo "arch=$HOST"
    echo "glibc_max=$(glibc_de "$1")"
    echo "dinamico=si"
  } > "$2/VERSION"
  ( cd "$2" && find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS )
}

empaquetar(){ # $1 = directorio montado, $2 = tarball de salida
  ( cd "$(dirname "$1")" && tar --sort=name --mtime='@0' --owner=0 --group=0 --numeric-owner -cf - "$(basename "$1")" | gzip -n -9 > "$2" )
}

manifiesto(){ # $1 = binario -> imprime ok/n; exit 1 si alguno falla.
  # §408 · UN solo productor del bucle: tools/conformidad.sh (RFC-0005 E4). Aqui solo se consume.
  local SAL RC=0
  SAL=$(bash tools/conformidad.sh "$1" 2>&1) || RC=$?
  echo "$SAL" | tail -n 1 | sed -n 's/^conformidad: \([0-9]*\) de \([0-9]*\) .*/\1\/\2/p'
  [ "$RC" = "0" ]
}

mkdir -p "$OUT"
if [ "${1:-}" = "--check" ]; then
  A=$(construir "$OUT/tA"); B=$(construir "$OUT/tB")
  HA=$(h16 "$A"); HB=$(h16 "$B")
  [ "$HA" = "$HB" ] || rojo "dos compilaciones con remap dan huellas distintas: $HA vs $HB"
  [ "$(strings -n 8 "$A" | grep -c "$HOME" || true)" = "0" ] || rojo "el binario lleva rutas de $HOME"
  M=$(manifiesto "$A") || rojo "el binario con remap no pasa el manifiesto ($M)"
  montar "$A" "$OUT/$NOMBRE"
  empaquetar "$OUT/$NOMBRE" "$OUT/check-1.tar.gz"; empaquetar "$OUT/$NOMBRE" "$OUT/check-2.tar.gz"
  [ "$(h16 "$OUT/check-1.tar.gz")" = "$(h16 "$OUT/check-2.tar.gz")" ] || rojo "el tarball no es reproducible"
  echo "  OK  artefacto: binario $HA reproducible entre rutas y sin rutas de la maquina, manifiesto $M, tarball $(h16 "$OUT/check-1.tar.gz") reproducible"
  exit 0
fi

BIN=$(construir "$OUT/build")
M=$(manifiesto "$BIN") || rojo "el binario no pasa el manifiesto ($M)"
montar "$BIN" "$OUT/$NOMBRE"
empaquetar "$OUT/$NOMBRE" "$OUT/$NOMBRE.tar.gz"
echo "artefacto : $OUT/$NOMBRE.tar.gz"
echo "tarball   : $(sha256sum "$OUT/$NOMBRE.tar.gz" | cut -d' ' -f1)  $(wc -c < "$OUT/$NOMBRE.tar.gz" | tr -d ' ') B  $(find "$OUT/$NOMBRE" -type f | wc -l | tr -d ' ') ficheros"
echo "binario   : $(sha256sum "$BIN" | cut -d' ' -f1)  $(wc -c < "$BIN" | tr -d ' ') B"
echo "manifiesto: $M"
sed 's/^/VERSION   : /' "$OUT/$NOMBRE/VERSION"
