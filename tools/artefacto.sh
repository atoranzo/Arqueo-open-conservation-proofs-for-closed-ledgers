#!/usr/bin/env bash
# tools/artefacto.sh — el ARTEFACTO de H2 (§401): el verificador distribuible.
#
# Compila `zk-ssl-verify` en release con `--remap-path-prefix` (la huella del binario no depende de la ruta del arbol ni de HOME DENTRO DE UNA MISMA MAQUINA, con CARGO_HOME y RUSTUP_HOME bajo HOME; ⚠️ §824: eso es con las reglas de `b6994c3`; con las de este sello, tampoco depende de CARGO_HOME ni de RUSTUP_HOME fuera de HOME, ni de si `rust-src` esta instalado, y lo que depende de la maquina es el cc, el as y los objetos de arranque del host, que VERSION nombra;
# depende del toolchain, de `Cargo.lock` y de la maquina; ⚠️ §823: decia «no depende de la ruta del arbol ni del usuario: depende del toolchain y de `Cargo.lock`», sin la condicion de la maquina, y la 0.4.2 rehecha desde su commit en otra sale con 13 de sus 35 secciones distintas: doc/KIT.md §«Reproducir el kit, para quien no se fíe de la descarga»),
# monta el juego que un tercero necesita —binario, el arnes `conformidad.sh` (§408),
# `spec/PAQUETE.md`, los DOS manifiestos y sus vectores, las licencias, `NOTICE`, `THIRD-PARTY.txt`,
# `VERSION`, `SHA256SUMS`— y lo
# empaqueta en un tarball reconstruible bajo `target/artefacto/` (ignorado por git), con la misma huella dentro de una misma maquina (⚠️ §823: decia «un tarball REPRODUCIBLE», sin la condicion de la maquina).
#
#     bash tools/artefacto.sh            # construye y deja target/artefacto/<nombre>.tar.gz
#     bash tools/artefacto.sh --check    # la PROPIEDAD, para el canon (§401, 3 ter): dos
#                                        # compilaciones en dos target dan la misma huella y
#                                        # cero rutas de $HOME (⚠️ §823: decia «cero rutas de la maquina», y solo busca las de $HOME); dos tarballs, la misma
#                                        # huella; el binario pasa los DOS manifiestos, y los
#                                        # pasa igual DESDE DENTRO del tarball, sin repo (§425). ⚠️ §824: desde este sello, las dos compilaciones son del kit en DOS RAICES —el arbol y su copia en target/artefacto/raiz2—, no en dos target del mismo arbol, y el binario no lleva ninguna ruta de las raices, de HOME, de CARGO_HOME ni de RUSTUP_HOME, no solo de $HOME.
#     bash tools/artefacto.sh --binarios # §824: el nodo de produccion y el testigo, con los MISMOS flags, en las dos raices: misma huella, o ROJO con las secciones que difieren.
#
# Lo que el binario exige: x86_64 Linux y la glibc que `VERSION` nombra (no es estatico).
# Lo que este fichero NO hace: publicar. La release en GitHub se sube a mano, con las huellas
# que este script imprime y el asiento registra. Exit 0 = VERDE; cualquier fallo, exit 1.
set -euo pipefail
cd "$(dirname "$0")/.."
RAIZ=$(pwd -P)   # §824: la ruta FISICA, la que cargo ve y el remap tiene que casar (era `pwd`, la logica)
OUT="$RAIZ/target/artefacto"
VER=$(sed -n 's/^version = "\(.*\)"/\1/p' crates/zk-ssl-verify/Cargo.toml | head -n 1)
RAIZ2="$OUT/raiz2"   # §824: la SEGUNDA RAIZ (sembrar(), mas abajo): el --check y --binarios compilan ahi otra vez
h16(){ sha256sum "$1" | cut -c1-16; }
rojo(){ echo "ROJO: $*" >&2; exit 1; }
# §824 · FAMILIAS sigue en esta linea, la 31, que citan los documentos; lo que pregunta a rustc (HOST, NOMBRE, el sysroot), detras de limpio(), en el bloque del entorno.
FAMILIAS="paquete consumo conflicto rechazo edad pendiente pago prenda completitud ancla ancla-cofirmada rotacion"   # S423 - UN productor: montar() y manifiesto() la comparten

# ── §824 · EL ENTORNO DE CARGO: solo lo que la herramienta deja pasar ─────────────────────────
# cargo, cc-rs y los build.rs de las dependencias leen del entorno cosas que cambian el binario sin
# que VERSION lo diga: el compilador de C y sus flags (CC, CFLAGS, CC_<target>, HOST_CC...; blake3
# lee CC y CFLAGS por su cuenta, y CARGO_FEATURE_PURE, que le quita el ensamblador), otro rustc
# (RUSTC) o un wrapper (RUSTC_WRAPPER), el perfil (CARGO_PROFILE_RELEASE_*), el enlazador
# (CARGO_TARGET_<triple>_LINKER), el modo incremental, RUSTC_BOOTSTRAP, y CARGO_ENCODED_RUSTFLAGS, que
# cargo prefiere a RUSTFLAGS y taparia los flags de reglas(). En la revision del §824, con `CC=clang` el
# kit salia otro (7cb169729272c13a) y VERSION seguia nombrando el gcc. Una lista de lo prohibido no se
# cierra -un build.rs puede leer cualquier variable, y bash no sabe quitar las que llevan un guion,
# como `CC_x86_64-unknown-linux-gnu`, que cc-rs si lee-, asi que va al reves: lo que llama a cargo,
# a rustc o al cc lo hace con `env -i` y solo las variables de PASAN, lo que hace falta para
# encontrar el toolchain, el registry y la red. VERSION nombra la lista. Lo que no es el entorno
# -los ficheros de configuracion de cargo-, configs_que_mueven(), mas abajo.
PASAN=(PATH HOME TMPDIR CARGO_HOME RUSTUP_HOME RUSTUP_TOOLCHAIN CARGO_BUILD_JOBS 'CARGO_HTTP_*' 'CARGO_NET_*'
       SSL_CERT_FILE SSL_CERT_DIR http_proxy https_proxy no_proxy all_proxy HTTP_PROXY HTTPS_PROXY NO_PROXY ALL_PROXY)
ENT=()
while IFS= read -r -d '' E; do
  for G in "${PASAN[@]}"; do
    if [[ ${E%%=*} == $G ]]; then ENT+=("$E"); break; fi
  done
done < <(env -0)
limpio(){ env -i "${ENT[@]}" "$@"; }
HOST=$(limpio rustc -vV | awk '/^host:/{print $2}')
NOMBRE="arqueo-verify-${VER}-${HOST}"
CH="${CARGO_HOME:-$HOME/.cargo}"; RH="${RUSTUP_HOME:-$HOME/.rustup}"; SYS=$(limpio rustc --print sysroot); RHASH=$(limpio rustc -vV | awk '/^commit-hash:/{print $2}')   # §824: lo que leen reglas() y rutas_de()

# ── §824 · LAS REGLAS DEL REMAP: UN productor, para el kit, el nodo y el testigo ──────────────
# Hasta el §824 eran cuatro, escritas a mano en una linea y derivadas de $HOME: la raiz, el registry,
# $HOME/.cargo y $HOME, en ese orden. rustc aplica la ULTIMA regla que casa, asi que con
# CARGO_HOME=$HOME/.cargo la de $HOME, la ultima, ganaba siempre y la del registry no ganaba nunca:
# medido en el nodo y en el testigo por el censo de H6, 286 y 306 rutas `/home/x/.cargo/registry/src/...`
# (doc/KIT.md, «Reproducir el kit, para quien no se fíe de la descarga»). Y ninguna llevaba las rutas
# de `rust-src` a /rustc/<commit-hash>: con el componente instalado y RUSTUP_HOME bajo $HOME, la de
# $HOME solo les cambia el prefijo y conservan el nombre del toolchain (once
# `/home/x/.rustup/toolchains/stable-.../lib/rustlib/src/rust/...` en la 0.4.2 publicada); con
# RUSTUP_HOME fuera de $HOME, entran enteras.
# Desde el §824: cinco reglas, derivadas de lo que cargo y rustc usan de verdad (CARGO_HOME, no
# $HOME/.cargo; el sysroot que dice rustc), y ORDENADAS por la longitud del prefijo, de la mas
# corta a la mas larga. De dos reglas que casan con una ruta, una es prefijo de la otra, y gana la
# mas larga, sea cual sea la maquina: el arbol dentro de $HOME, CARGO_HOME fuera de el, o un
# RUSTUP_HOME dentro del arbol.
#   <home>                         $HOME                                -> /home/x
#   <cargo_home>                   CARGO_HOME (si no, $HOME/.cargo)     -> /cargo
#   <cargo_home>/registry/src      el registry                          -> /registry
#   <sysroot>/lib/rustlib/src/rust `rust-src`, si esta instalado        -> /rustc/<commit-hash>
#                                  (la ruta que rustc escribe cuando NO lo esta: con el componente
#                                  o sin el, el mismo binario; medido con la 0.4.2: doc/KIT.md,
#                                  «Reproducir el kit, para quien no se fíe de la descarga»)
#   <raiz>                         la raiz del arbol que se compila     -> /arqueo
tabla_de_reglas(){ # $1 = raiz -> "nombre<TAB>prefijo<TAB>destino" por linea, en el orden de la tabla de arriba
  printf '<home>\t%s\t/home/x\n' "$HOME"
  printf '<cargo_home>\t%s\t/cargo\n' "$CH"
  printf '<cargo_home>/registry/src\t%s\t/registry\n' "$CH/registry/src"
  printf '<sysroot>/lib/rustlib/src/rust\t%s\t/rustc/%s\n' "$SYS/lib/rustlib/src/rust" "$RHASH"
  printf '<raiz>\t%s\t/arqueo\n' "$1"
}
reglas(){ # $1 = raiz -> las mismas, de la mas corta a la mas larga: el orden en que se aplican
  tabla_de_reglas "$1" | awk -F'\t' '{ print length($2) "\t" $0 }' | sort -n -s -k1,1 | cut -f2-
}
flags_de(){ reglas "$1" | awk -F'\t' '{ printf "%s--remap-path-prefix=%s=%s", (NR > 1 ? " " : ""), $2, $3 }'; }
# VERSION las nombra en el orden de la TABLA, no en el aplicado: el aplicado depende de la longitud de
# las rutas de cada maquina, y VERSION -y con el la huella del tarball- no tiene por que (medido en el
# §824: con un sysroot mas largo que la raiz, el orden aplicado cambia y el binario no).
genericos_de(){ tabla_de_reglas "$1" | awk -F'\t' '{ printf "%s--remap-path-prefix=%s=%s", (NR > 1 ? " " : ""), $1, $3 }'; }
# RUSTFLAGS se parte por espacios: una ruta con espacios partiria su regla y el remap no casaria.
for P in "$RAIZ" "$HOME" "$CH" "$SYS"; do
  case "$P" in *[[:space:]]*) rojo "la ruta «$P» lleva espacios: RUSTFLAGS la partiria y el remap no casaria" ;; esac
done
FLAGS_GENERICOS=$(genericos_de "$RAIZ")   # lo que VERSION dice, sin las rutas de esta maquina

# §824 · LOS FICHEROS DE CONFIGURACION DE CARGO, que `env -i` no quita: $CARGO_HOME/config.toml, y
# `.cargo/config.toml` en el directorio desde el que se llama a cargo y en cada uno de sus padres
# (con `.toml` o sin el). Sus tablas [profile], [env], [target], [host], [build], [source], [patch] y
# [unstable], o sus claves con punto (`profile.release.opt-level = 2`), cambian lo que se compila o
# con que -el perfil, el compilador de C, el enlazador, el wrapper de rustc, de donde salen las
# fuentes- sin que VERSION lo diga: si alguno las trae, ROJO, con el fichero y la linea. Las demas
# ([net], [http], [registries], [term], [alias]...) cambian como se descarga o como se habla, no que.
configs_que_mueven(){ # $1... = directorios desde los que se llama a cargo -> "<fichero>:<linea>" por cada una; nada si ninguna
  local D F L rc
  local -a FS=("$CH/config" "$CH/config.toml")
  local -A VISTO=()
  for D in "$@"; do
    while :; do FS+=("$D/.cargo/config" "$D/.cargo/config.toml"); [ "$D" = / ] && break; D=$(dirname "$D"); done
  done
  for F in "${FS[@]}"; do
    [ -z "${VISTO[$F]:-}" ] || continue; VISTO[$F]=1   # la raiz y la segunda comparten padres
    [ -f "$F" ] || continue
    rc=0; L=$(grep -nE '^[[:space:]]*(\[+[[:space:]]*)?"?(profile|env|target|host|build|source|patch|unstable)"?[[:space:]]*([].=]|$)' "$F") || rc=$?
    [ "$rc" -le 1 ] || { echo "$F:ilegible"; continue; }
    [ -z "$L" ] || printf '%s\n' "$L" | cut -d: -f1 | sed "s|^|$F:|"
  done
}
CFGS=$(configs_que_mueven "$RAIZ" "$RAIZ2") || rojo "no se pueden mirar los ficheros de configuracion de cargo"
[ -z "$CFGS" ] || rojo "un fichero de configuracion de cargo cambia lo que se compila sin que VERSION lo diga: ${CFGS//$'\n'/ } (quitelo, o llame con un CARGO_HOME y desde una ruta sin el)"

# §824 · LA CACHE DE CARGO NO SABE DEL HOST NI DE TODO EL ENTORNO. cargo recompila si cambian las
# fuentes, el Cargo.lock, los flags o el rustc, pero no si cambia el cc o el as del host, ni si cambia
# una variable que un build.rs leyo sin declararla: cc-rs lee `CC_x86_64-unknown-linux-gnu` y no la
# declara, y medido en la revision del §824, un target compilado con ella (con clang) daba despues, ya
# con `env -i`, el kit de clang (7cb169729272c13a) con VERSION nombrando el gcc. Por eso cada target
# lleva el sello de con que se compilo -esta herramienta; el rustc y el cargo; el cc, el as, la glibc
# y los objetos de arranque que VERSION nombra; y el cc, el cc1 y el as que de verdad corren, por su
# huella-, y si no es el de ahora se vacia antes de compilar: la primera pasada tras cambiar la
# herramienta o el host es en frio.
binario_de(){ # $1 = programa o ruta -> "<ruta real>:<h16>", o "<programa>:ausente"
  local P
  if P=$(limpio sh -c 'command -v -- "$1"' _ "$1" 2>/dev/null) && P=$(readlink -f -- "$P") && [ -f "$P" ]; then
    echo "$P:$(h16 "$P")"
  else
    echo "$1:ausente"
  fi
}
sello_del_host(){ # -> 16 hex; cambia si cambia cualquier cosa de la que dependa lo compilado y cargo no mire
  { h16 "$RAIZ/tools/artefacto.sh"; limpio rustc -vV; limpio cargo --version
    cc_del_host; as_del_host; glibc_del_host; objetos_de_arranque
    binario_de cc
    binario_de "$(limpio cc -print-prog-name=cc1 2>/dev/null || echo cc1)"
    binario_de "$(limpio cc -print-prog-name=as 2>/dev/null || echo as)"
  } | sha256sum | cut -c1-16
}
preparar_target(){ # $1 = target dir bajo $OUT: si no lleva el sello de ahora, se vacia
  local T="$1"
  case "$T" in "$OUT"/?*) : ;; *) rojo "preparar_target: «$T» no esta bajo $OUT" ;; esac
  if [ "$(cat "$T/.sello-del-host" 2>/dev/null || true)" != "$SELLO_HOST" ]; then
    rm -rf -- "${T:?}"; mkdir -p "$T"; echo "$SELLO_HOST" > "$T/.sello-del-host"
  fi
}

construir(){ # $1 = raiz, $2 = target dir, $3... = lo que se compila; los flags, los de flags_de $1
  local R="$1" T="$2"; shift 2
  preparar_target "$T"
  ( cd "$R" && limpio RUSTFLAGS="$(flags_de "$R")" CARGO_TARGET_DIR="$T" cargo build --release -q --locked "$@" )
}
construir_kit(){ # $1 = raiz, $2 = target dir -> imprime la ruta del binario
  construir "$1" "$2" -p zk-ssl-verify
  de_su_raiz "$1" "$2" zk-ssl-verify
  echo "$2/release/zk-ssl-verify"
}
# §824 · PRUEBA DE VIDA de la segunda raiz: el dep-info que cargo deja junto al binario nombra, con su
# ruta absoluta, las fuentes con que se compilo. Si no son las de esa raiz, las dos compilaciones
# serian una, y su igualdad no diria nada.
de_su_raiz(){ # $1 = raiz, $2 = target dir, $3 = binario
  grep -qF -- " $1/crates/$3/src/" "$2/release/$3.d" 2>/dev/null \
    || rojo "prueba de vida: $2/release/$3.d no nombra las fuentes de $1/crates/$3"
}

# §824 · LA SEGUNDA RAIZ. Una copia de lo que git sigue o no ignora (`git ls-files --cached --others
# --exclude-standard`): lo que el arbol compilaria AHORA, tambien lo que aun no se ha commiteado,
# que es lo que el canon juzga. Vive en $RAIZ2, bajo target/, y no se borra entre pasadas: solo se
# reescribe lo que cambio de CONTENIDO, con el mtime de ahora, y lo igual conserva el suyo, para
# que cargo recompile alli lo que debe y nada mas; un fichero que deja la lista se quita, y uno que
# sigue en el indice pero ya no esta en el arbol (borrado sin `git rm`), tambien: si no, la copia lo
# conservaria y la segunda raiz compilaria lo que la primera ya no tiene (un build.rs, un bin/), un
# ROJO falso (visto en la revision del §824). No lleva `.git`: ningun build.rs del arbol lo lee (no
# hay ninguno), y lo que montar() pregunta a git se pregunta en la primera raiz.
sembrar(){ # $1 = destino, una ruta absoluta bajo $OUT
  local D="$1" F
  case "$D" in "$OUT"/?*) : ;; *) rojo "sembrar: «$D» no esta bajo $OUT" ;; esac
  mkdir -p "$D"
  git ls-files -z --cached --others --exclude-standard | sort -z > "$D.lista.nueva"
  if [ -f "$D.lista" ]; then
    comm -z -23 "$D.lista" "$D.lista.nueva" | while IFS= read -r -d '' F; do
      [ -n "$F" ] && rm -f -- "${D:?}/${F:?}"
    done
  fi
  while IFS= read -r -d '' F; do
    [ -f "$F" ] || { rm -f -- "${D:?}/${F:?}"; continue; }   # borrado del arbol y aun en el indice: fuera de la copia
    cmp -s "$F" "$D/$F" 2>/dev/null && continue
    mkdir -p "$D/$(dirname "$F")"; cp "$F" "$D/$F"
  done < "$D.lista.nueva"
  mv "$D.lista.nueva" "$D.lista"
}

# §824 · Las rutas de ESTA maquina que un binario no debe llevar: las dos raices, $HOME, CARGO_HOME,
# RUSTUP_HOME y, del sysroot, `<sysroot>/lib/rustlib`. Hasta el §824 se buscaba solo $HOME. Y `/home/x/`,
# el destino de la regla MAS GENERAL: todo lo que hay bajo $HOME y entra en un binario lo cubre una
# regla mas especifica (la raiz, CARGO_HOME, el registry, rust-src), asi que `/home/x/` solo aparece
# si el orden de las reglas deja ganar a la general. Con el orden de `b6994c3`, el kit llevaba 41
# `/home/x/.cargo/registry/...`.
# Se buscan ENTERAS, sin exigirles nada detras: un $HOME desnudo (un env!("HOME")) es igual en las
# dos raices y solo esta busqueda lo ve; con la barra pasaba. Tampoco una frontera de componente:
# strings pega las cadenas contiguas (`/usr/lib/debug/.build-id//usr/lib/debug/.debug_abbrev.dwo...`,
# en los tres binarios). La barra, solo en `/home/x/`. Del sysroot, `<sysroot>/lib/rustlib` y no
# `<sysroot>/`: con un rustc de distribucion el sysroot es /usr, y esa cadena de la biblioteca
# estandar daria un ROJO falso.
# La salida de strings va a un FICHERO, no a una variable: con una `S` exportada por quien llama, bash
# conservaba el export en la local, los megas de strings entraban en el entorno de grep, grep moria
# con E2BIG y el `|| true` se lo tragaba: VERDE con rutas dentro (medido en la revision del §824). Un
# strings que falla o no saca nada, o un grep que sale con 2 o mas, es ROJO; 0 y 1 son respuestas.
rutas_de(){ # $1 = binario -> "<ruta>:<cuantas>" por cada una que aparece; nada si ninguna; ROJO si no puede mirar
  local P n rc STR
  STR=$(mktemp "$OUT/strings.XXXXXX") || rojo "rutas_de: no se puede crear un temporal en $OUT"
  strings -n 8 "$1" > "$STR" || rojo "rutas_de: strings falla sobre $1"
  [ -s "$STR" ] || rojo "rutas_de: strings no saca nada de $1"
  for P in "$RAIZ" "$RAIZ2" "$HOME" "$CH" "$RH" "$SYS/lib/rustlib" /home/x/; do
    rc=0; n=$(grep -cF -- "$P" "$STR") || rc=$?
    [ "$rc" -le 1 ] || rojo "rutas_de: grep sale con $rc buscando «$P» en $1"
    [ "$n" = "0" ] || printf '%s:%s ' "$P" "$n"
  done
  rm -f -- "$STR"
}

# §824 · Si dos binarios difieren, DONDE: seccion a seccion (ELF64), con su tamano y su huella.
secciones(){ # $1, $2 = dos ELF -> las secciones distintas, y cuantas de cuantas
  python3 - "$1" "$2" <<'PYEOF'
import hashlib, struct, sys
def secciones(ruta):
    d = open(ruta, "rb").read()
    if d[:4] != b"\x7fELF" or d[4] != 2 or d[5] != 1:
        sys.exit(f"{ruta}: no es un ELF64 little-endian")
    shoff, = struct.unpack_from("<Q", d, 0x28)
    shentsize, shnum, shstrndx = struct.unpack_from("<HHH", d, 0x3A)
    hs = [struct.unpack_from("<IIQQQQ", d, shoff + i * shentsize) for i in range(shnum)]
    base = hs[shstrndx][4]
    out = []
    for nombre, tipo, _, _, off, tam in hs:
        n = d[base + nombre: d.index(b"\0", base + nombre)].decode() or "(nula)"
        cuerpo = b"" if tipo == 8 else d[off: off + tam]   # SHT_NOBITS no ocupa fichero
        out.append((n, tam, hashlib.sha256(cuerpo).hexdigest()[:16]))
    return len(d), out
za, a = secciones(sys.argv[1]); zb, b = secciones(sys.argv[2])
print(f"  tamanos {za} / {zb} B; secciones {len(a)} / {len(b)}")
dist = 0
for x, y in zip(a, b):
    if x != y:
        dist += 1
        print(f"  DISTINTA {x[0]}: {x[1]} B {x[2]} / {y[0]}: {y[1]} B {y[2]}")
print(f"  secciones distintas: {dist} de {max(len(a), len(b))}")
PYEOF
}

glibc_de(){ strings -n 6 "$1" | grep -o 'GLIBC_[0-9.]*' | sort -uV | tail -n 1; }   # §824: en montar(), fuera del echo

# §824 · Los objetos que el cc enlaza en todo ejecutable PIE, y su huella: los de la glibc del host
# (Scrt1.o, crti.o, crtn.o, de libc6-dev en Debian y Ubuntu) y los de GCC (crtbeginS.o, crtendS.o).
# Medido con nm en el kit, el nodo y el testigo: `_start` y `__abi_tag` de Scrt1.o, `_init` y
# `_fini` de crti.o, y lo de crtbeginS.o y crtendS.o; de libc_nonshared.a, libgcc.a y libgcc_eh.a,
# ningun simbolo (salvo `__floattidf` en el nodo, que compiler_builtins tambien define).
cc_del_host(){ { limpio cc --version 2>/dev/null || echo ausente; } | sed -n 1p; }
as_del_host(){ { limpio "$(limpio cc -print-prog-name=as 2>/dev/null || echo as)" --version 2>/dev/null || echo ausente; } | sed -n 1p; }
glibc_del_host(){ { limpio ldd --version 2>/dev/null || echo ausente; } | sed -n 1p; }
objetos_de_arranque(){ # -> "<objeto>:<h16>" por cada uno, o "<objeto>:ausente"
  local O P S=""
  for O in Scrt1.o crti.o crtn.o crtbeginS.o crtendS.o; do
    P=$(limpio cc -print-file-name="$O" 2>/dev/null) || P=""
    case "$P" in
      /*) [ -f "$P" ] || P="" ;;
      *) P="" ;;   # cc devuelve el nombre a secas si no lo encuentra
    esac
    if [ -n "$P" ]; then S="$S $O:$(h16 "$P")" || return 1; else S="$S $O:ausente"; fi
  done
  echo "${S# }"
}

montar(){ # $1 = binario, $2 = directorio destino
  # §824 · la glibc, ANTES del bloque de VERSION: dentro de un `echo "...$(glibc_de ...)"` un strings
  # que falla dejaba `glibc_max=` vacio y nadie lo decia.
  local G A; G=$(glibc_de "$1") || rojo "montar: no se lee la glibc de $1 (strings o grep fallan, o no lleva GLIBC_)"
  A=$(objetos_de_arranque) || rojo "montar: no se leen los objetos de arranque que enlaza el cc"
  rm -rf "$2"; mkdir -p "$2/spec"
  cp "$1" "$2/zk-ssl-verify"; chmod 755 "$2/zk-ssl-verify"
  cp spec/PAQUETE.md "$2/spec/"
  for F in $FAMILIAS; do
    mkdir -p "$2/spec/vectors/$F"
    cp "spec/vectors/$F/MANIFIESTO.txt" spec/vectors/$F/*.json "$2/spec/vectors/$F/"
  done
  # §408 · el arnes viaja dentro: `bash conformidad.sh ./zk-ssl-verify` desde la raiz del tarball
  cp tools/conformidad.sh "$2/conformidad.sh"; chmod 755 "$2/conformidad.sh"
  cp LICENSE-APACHE LICENSE-MIT NOTICE "$2/"
  # §533 · el fork de winterfell (RFC-0009 E3a-1) es path-dep del arbol: THIRD-PARTY.txt lo
  # nombra como FORK, no como el crate de crates.io del mismo nombre y version.
  limpio cargo tree -p zk-ssl-verify -e normal --prefix none --format '{p} {l}' --locked | sed 's| (/[^)]*/crates/winter-[a-z]*)| (FORK en el arbol, RFC-0009 E3a-1, §533)|; s/ (\/[^)]*)//; s/ (\*)//' | sort -u > "$2/THIRD-PARTY.txt"
  {
    echo "artefacto=$NOMBRE"
    echo "commit=$(git rev-parse HEAD)"
    echo "describe=$(git describe --tags --long --always)"
    limpio rustc -vV | sed 's/^/rustc./'
    echo "cargo=$(limpio cargo --version)"
    echo "rustflags=$FLAGS_GENERICOS"
    echo "rustflags_orden=se aplican de la regla de prefijo mas corto a la de prefijo mas largo: gana la mas especifica"
    # §824 · el codigo nativo que compila el host (el ensamblador de blake3; el C de ring, en el
    # testigo) depende de su cc y de su as, y la huella con el: la 0.4.2 publicada y la rehecha
    # desde su commit en otra maquina difieren en 240 bytes de ese ensamblador y en la .comment.
    # El cc es el que cc-rs usa por defecto con host = target, `cc` (las variables que lo cambiarian
    # no pasan: PASAN, arriba), y el mismo con que rustc enlaza; el as, el que ese cc llama
    # (`cc -print-prog-name=as`). La glibc del host, por `ldd`, y los objetos que el cc enlaza
    # siempre, por su huella (objetos_de_arranque, arriba).
    echo "cc=$(cc_del_host)"
    echo "as=$(as_del_host)"
    echo "glibc_host=$(glibc_del_host)"
    echo "objetos_de_arranque=$A"
    echo "entorno_de_cargo=env -i, y solo ${PASAN[*]}"
    echo "arch=$HOST"
    echo "glibc_max=$G"
    echo "dinamico=si"
  } > "$2/VERSION"
  ( cd "$2" && find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS )
}

empaquetar(){ # $1 = directorio montado, $2 = tarball de salida
  ( cd "$(dirname "$1")" && tar --sort=name --mtime='@0' --owner=0 --group=0 --numeric-owner -cf - "$(basename "$1")" | gzip -n -9 > "$2" )
}

manifiesto(){ # $1 = binario, $2 = arnes -> imprime "<familia> ok/n"; exit 1 si alguna falla.
  # §408 · UN solo productor del bucle: tools/conformidad.sh (RFC-0005 E4). Aqui solo se consume.
  # §423 · una corrida por familia, MISMO binario y otro manifiesto (molde del 3 bis del canon).
  # §425 · el arnes se PASA porque desde DENTRO del tarball vive en ./conformidad.sh, no en tools/.
  #        El manifiesto sigue siendo relativo al CWD, que es lo mismo en el arbol y dentro.
  # OJO: la acumuladora NO puede llamarse OUT, que es global en este fichero.
  local SAL RC=0 R F RES="" ARN="${2:-tools/conformidad.sh}"
  for F in $FAMILIAS; do
    R=0; SAL=$(bash "$ARN" "$1" "spec/vectors/$F/MANIFIESTO.txt" 2>&1) || R=$?
    [ "$R" = "0" ] || RC=$R
    RES="$RES$F $(echo "$SAL" | tail -n 1 |
      sed -n 's/^conformidad: \([0-9]*\) de \([0-9]*\) .*/\1\/\2/p') "
  done
  echo "$RES" | sed 's/ *$//'
  [ "$RC" = "0" ]
}

mkdir -p "$OUT"
SELLO_HOST=$(sello_del_host) || rojo "no se puede calcular el sello del host (rustc, cargo, cc, as o los objetos de arranque)"
if [ "${1:-}" = "--check" ]; then
  # §824 · DOS RAICES, no dos target del mismo arbol: la segunda compila desde otra ruta y con su
  # propio target. El tB de antes, el segundo target del mismo arbol, ya no se usa y se quita.
  rm -rf -- "${OUT:?}/tB"
  sembrar "$RAIZ2"
  A=$(construir_kit "$RAIZ" "$OUT/tA"); B=$(construir_kit "$RAIZ2" "$RAIZ2/target")
  HA=$(h16 "$A"); HB=$(h16 "$B")
  if [ "$HA" != "$HB" ]; then
    secciones "$A" "$B" >&2
    rojo "el kit compilado en dos raices da huellas distintas: $HA vs $HB"
  fi
  for X in "$A" "$B"; do
    P=$(rutas_de "$X") || rojo "no se pudo buscar rutas de la maquina en $X"
    [ -z "$P" ] || rojo "el binario $X lleva rutas de la maquina: $P"
  done
  M=$(manifiesto "$A") || rojo "el binario con remap no pasa los manifiestos ($M)"
  montar "$A" "$OUT/$NOMBRE"
  empaquetar "$OUT/$NOMBRE" "$OUT/check-1.tar.gz"; empaquetar "$OUT/$NOMBRE" "$OUT/check-2.tar.gz"
  [ "$(h16 "$OUT/check-1.tar.gz")" = "$(h16 "$OUT/check-2.tar.gz")" ] || rojo "el tarball no es reproducible"
  # §425 · el punto 110: los DOS catalogos, DESDE DENTRO del tarball y sin repo.
  DES="$OUT/desde-dentro"; rm -rf "$DES"; mkdir -p "$DES"
  tar -xzf "$OUT/check-1.tar.gz" -C "$DES" || rojo "no se puede desempaquetar el tarball"
  MD=$( cd "$DES/$NOMBRE" && manifiesto ./zk-ssl-verify ./conformidad.sh ) \
    || rojo "desde DENTRO del tarball los catalogos no pasan ($MD)"
  [ "$MD" = "$M" ] || rojo "desde dentro dice '$MD' y desde el arbol '$M'"
  echo "  OK  artefacto: binario $HA igual desde dos raices y sin rutas de la maquina (raices, HOME, CARGO_HOME, RUSTUP_HOME, <sysroot>/lib/rustlib, /home/x/), manifiestos $M, desde dentro del tarball $MD, tarball $(h16 "$OUT/check-1.tar.gz") reproducible"
  exit 0
fi

if [ "${1:-}" = "--binarios" ]; then
  # §824 · el nodo DE PRODUCCION (`--no-default-features`: sin `dev`, la emision con los custodios de
  # prueba; crates/zk-ssl-node/Cargo.toml) y el testigo, cada uno en SU invocacion de cargo: juntos,
  # cargo unifica las features y el `zk-ssl` del nodo entraria con la `sandbox` que pide el testigo
  # (crates/zk-ssl-cli/Cargo.toml). Los flags, los de flags_de, el productor del kit; las raices y
  # los target, los del --check. Si difieren, las secciones que difieren, y ROJO.
  sembrar "$RAIZ2"
  for LADO in A B; do
    if [ "$LADO" = "A" ]; then R="$RAIZ"; T="$OUT/tA"; else R="$RAIZ2"; T="$RAIZ2/target"; fi
    construir "$R" "$T" -p zk-ssl-node --no-default-features; de_su_raiz "$R" "$T" zk-ssl-node
    construir "$R" "$T" -p zk-ssl-cli; de_su_raiz "$R" "$T" zk-ssl-cli
    mkdir -p "$OUT/binarios-$LADO"
    cp "$T/release/zk-ssl-node" "$T/release/zk-ssl-cli" "$OUT/binarios-$LADO/"
  done
  RES=""
  for X in zk-ssl-node zk-ssl-cli; do
    A="$OUT/binarios-A/$X"; B="$OUT/binarios-B/$X"
    if ! cmp -s "$A" "$B"; then
      secciones "$A" "$B" >&2
      rojo "$X compilado en dos raices da huellas distintas: $(h16 "$A") vs $(h16 "$B")"
    fi
    for Y in "$A" "$B"; do
      P=$(rutas_de "$Y") || rojo "no se pudo buscar rutas de la maquina en $Y"
      [ -z "$P" ] || rojo "$X ($Y) lleva rutas de la maquina: $P"
    done
    RES="$RES $X $(sha256sum "$A" | cut -d' ' -f1) $(wc -c < "$A" | tr -d ' ') B;"
  done
  echo "  OK  binarios iguales desde dos raices y sin rutas de la maquina:${RES%;} (el nodo sin dev)"
  exit 0
fi

[ -z "${1:-}" ] || rojo "uso: bash tools/artefacto.sh [--check | --binarios]; «$1» no es un modo"
BIN=$(construir_kit "$RAIZ" "$OUT/build")
M=$(manifiesto "$BIN") || rojo "el binario no pasa los manifiestos ($M)"
montar "$BIN" "$OUT/$NOMBRE"
empaquetar "$OUT/$NOMBRE" "$OUT/$NOMBRE.tar.gz"
echo "artefacto : $OUT/$NOMBRE.tar.gz"
echo "tarball   : $(sha256sum "$OUT/$NOMBRE.tar.gz" | cut -d' ' -f1)  $(wc -c < "$OUT/$NOMBRE.tar.gz" | tr -d ' ') B  $(find "$OUT/$NOMBRE" -type f | wc -l | tr -d ' ') ficheros"
echo "binario   : $(sha256sum "$BIN" | cut -d' ' -f1)  $(wc -c < "$BIN" | tr -d ' ') B"
echo "manifiestos: $M"
sed 's/^/VERSION   : /' "$OUT/$NOMBRE/VERSION"
