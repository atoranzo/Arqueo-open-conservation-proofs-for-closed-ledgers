#!/usr/bin/env bash
# tools/banco_prenda.sh -- el banco de E3 del RFC-0008, la PRENDA (D-AV..D-BI).
#
# Demuestra la PRUEBA PORTABLE DE LA PRENDA de punta a punta, y es la primera vez que la boca del
# prendador habla con un nodo. Un libro con un pendiente v2 VIVO -puesto por el sandbox del cli con
# el nodo PARADO, con prueba STARK real y por la via v2- -> nodo real que lo abre, lo mete en la
# FOTO de su latido y FIRMA una cabeza v5 -> la BOCA del prendador (`zk-ssl-cli prueba-prenda`)
# abre su KEYSTORE, pide la cabeza y la foto, prueba en el cliente y escribe el sobre `prenda`
# (spec/PAQUETE.md 2.10); la segunda vez ademas PUBLICA la marca con `zkssl_pledge` bajo el mismo
# latido -> el nodo MUERE -> el verificador en VERDE **sin el nodo**. Dos positivos; UN rechazo EN
# VIVO, el barato, que no llega ni a pedir; y SIETE negativos sobre el sobre, uno por regla.
#
# ⚠️ ESTE BANCO PRUEBA MEDIA PRENDA, Y LO DICE. El PAR es la marca bajo el `consRoot` firmado MAS
# este sobre (D-AS); el mando lo imprime en su propio veredicto. La otra mitad se pide con
# `zkssl_consumoPath` y tiene su banco aparte (`tools/banco_consumo.sh`, RFC-0006 E3). Aqui se
# comprueba que `zkssl_pledge` ACEPTA -- que es lo que este lado puede afirmar --, no que la marca
# este publicada: eso seria probar el consumo con el banco de la prenda.
#
# ⚠️ EL ORDEN ES EL DEL HERMANO (`tools/banco_pago.sh`, S508), y por la misma razon: la foto de los
# pendientes vive en el latido, en memoria del nodo, y `sled` abre el libro en EXCLUSIVA, asi que
# la siembra va ANTES de arrancar el nodo y nada de lo que sigue vuelve a abrir el libro.
#
# ⚠️ LA SIEMBRA ESCRIBE CINCO FICHEROS, y el quinto es del S544: el KEYSTORE del receptor, que es
# quien prenda. La boca no admite la clave de otra forma (D-BC), y sale de la MISMA
# `key_of(key_seed, to)` que escribe su credencial: el banco lo COMPRUEBA cruzando los dos
# `publicId` antes de arrancar el nodo. Si algun dia dejaran de ser el mismo, este banco lo dice
# antes de gastar una prueba.
#
# ⚠️ LA VENTANA DEL LATIDO: la boca prueba y DESPUES publica, y `zkssl_pledge` rehusa si cayo un
# latido entre medias. El latido va CORTO -3 s, como el hermano- y la carrera se REINTENTA: medido
# en el PASTE-545-M, la prueba tarda 1 s, asi que la ventana sobra. Una corrida que muriera por ahi
# no distinguiria una carrera de un fallo real.
#
# ⚠️ EL ENUNCIADO DE LA PRENDA TIENE DOS CAMPOS, receptor y marca, asi que los negativos del
# hermano que mienten el importe o el nacido NO tienen gemelo aqui: en su lugar van los dos que
# mienten lo que si afirma. Y los dos dan EXACTAMENTE el mismo texto -lo pone winterfell-, por eso
# `niega` exige el prefijo `prenda:` del mando y NUNCA el nombre del error, que es de la libreria.
#
# FUERA del canon: levanta procesos y espera latidos. Compila en RELEASE porque aqui se firma y se
# prueba de verdad.
#
# NO ESCRIBE EN EL ARBOL: todo lo suyo vive en un temporal bajo $HOME, que borra al salir. Lo
# comprueba al final por `git status --porcelain`, por DELTA.
#
#   bash tools/banco_prenda.sh [--guardar <dir>]
#
# --guardar  copia a <dir>, con su huella: los DOS sobres positivos, los SIETE cuerpos negativos y
#            los CINCO ficheros de la siembra. De estas capturas se derivan los vectores del
#            catalogo `prenda-*` (regla 2 del PROCESO). ⚠️ Y HAY QUE CAPTURARLAS: dos pruebas del
#            MISMO enunciado bajo la MISMA cabeza NO dan los mismos bytes desde que el probador
#            oculta (S538) -68.068 B y 66.333 B en el PASTE-545-M-, asi que un vector de prenda no
#            se puede re-derivar corriendo esto otra vez.
#
# ⚠️ Los fragmentos de rechazo de abajo estan LEIDOS del fuente: del mando
# (`crates/zk-ssl-verify/src/main.rs`), de la boca (`crates/zk-ssl-cli/src/prenda.rs`) y del
# keystore del SDK (`crates/zk-ssl-sdk/src/keystore.rs`), salvo el de la prueba que no verifica,
# que lo pone winterfell y no se predice. El instrumento mide; no adivina.
set -euo pipefail
msg(){ printf 'BANCO-PRENDA| %s\n' "$*" >&2; }
fallo(){ msg "ROJO: $*"; exit 1; }

# ⚠️ LA RAIZ SE DERIVA, no se supone (copiado del hermano): las dos procedencias, en orden, y la
# elegida trae su PRUEBA DE VIDA.
RAIZ=""
for cand in "$(cd "$(dirname "$0")/.." 2>/dev/null && pwd)" \
            "$(git rev-parse --show-toplevel 2>/dev/null)" ; do
  [ -n "$cand" ] || continue
  if [ -f "$cand/Cargo.toml" ] && [ -d "$cand/crates/zk-ssl-verify" ]; then RAIZ="$cand"; break; fi
done
[ -n "$RAIZ" ] || fallo "no se localizo la raiz del arbol: ni junto a este fichero ni en el repo del directorio actual hay un Cargo.toml con crates/zk-ssl-verify. Correr desde ~/zk-ssl-real"
cd "$RAIZ"
msg "raiz DERIVADA: $RAIZ"
command -v curl    >/dev/null 2>&1 || fallo "curl no esta en el PATH"
command -v python3 >/dev/null 2>&1 || fallo "python3 no esta en el PATH"

GUARDAR=""
case "${1:-}" in
  --guardar) GUARDAR="${2:?--guardar exige un directorio}"; mkdir -p "$GUARDAR" ;;
  "") : ;;
  *) fallo "uso: bash tools/banco_prenda.sh [--guardar <dir>]" ;;
esac

PORT=8617   # libre: censados en tools/*.sh estan 8593, 8594 (PROXY), 8597, 8598, 8599, 8601,
            # 8603, 8605, 8607, 8609, 8611, 8613 y 8615
LATIDO=3    # corto, como el hermano: la prueba tarda 1 s (PASTE-545-M) y la carrera se reintenta
DIR=$(mktemp -d "$HOME/.banco_prenda.XXXXXX")
PID=""
limpiar(){
  if [ -n "$PID" ]; then kill -9 "$PID" 2>/dev/null || true; fi
  rm -rf "$DIR"
}
trap limpiar EXIT INT TERM HUP QUIT

# ⚠️ LA BASE DE LA PUREZA, tomada ANTES de nada: el invariante no es <<el arbol esta limpio>> sino
# <<yo no lo ensucie>> (PRECISION 169). Se mide por DELTA.
git status --porcelain | sort > "$DIR/porcelain.base"

msg "compilando cli, nodo y verificador en RELEASE (aqui se firma y se prueba de verdad)"
cargo build --release -q -p zk-ssl-cli -p zk-ssl-node -p zk-ssl-verify 2>/dev/null \
  || cargo build --release -p zk-ssl-cli -p zk-ssl-node -p zk-ssl-verify || fallo "no compila"
CLI=target/release/zk-ssl-cli
NODO=target/release/zk-ssl-node
VER=target/release/zk-ssl-verify
python3 -c "print('37'*96, end='')" > "$DIR/semilla.hex"
chmod 600 "$DIR/semilla.hex"

rpc(){
  curl -s --max-time 20 "http://127.0.0.1:$PORT" \
    -H 'Content-Type: application/json' \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$1\",\"params\":$2}"
}
qfich(){ python3 -c 'import json,sys; v=json.load(open(sys.argv[1]))[sys.argv[2]]; print(int(v,16) if sys.argv[3]=="n" else v)' "$1" "$2" "$3"; }

# ---------------------------------------------------------------- LA SIEMBRA, CON EL NODO PARADO
# UNA sola corrida del sandbox: los dos positivos son dos pruebas del MISMO pendiente. Nada se
# teclea: el importe sale del aviso, el indice y el publicId de la credencial, y la clave del
# keystore que el propio sandbox sella (S544).
printf 'la frase del banco de la prenda\n' > "$DIR/frase.txt"
chmod 600 "$DIR/frase.txt"
msg "sandbox: un envio v2 EN VUELO, con los CINCO ficheros de la siembra"
"$CLI" --log warn simulate --ledger "$DIR/ledger" --no-claim --v2 \
  --aviso "$DIR/aviso.json" --credencial "$DIR/credencial.json" \
  --retorno "$DIR/retorno.json" --credencial-pagador "$DIR/credencial-pagador.json" \
  --keystore "$DIR/keystore.json" --frase-fichero "$DIR/frase.txt" \
  >"$DIR/sim.txt" 2>&1 || { sed 's/^/BANCO-PRENDA|   /' "$DIR/sim.txt" >&2; fallo "el sandbox no dejo el envio v2 en vuelo"; }
for f in aviso credencial retorno credencial-pagador keystore; do
  [ -s "$DIR/$f.json" ] || fallo "el sandbox no escribio $f.json"
done
X=$(qfich "$DIR/aviso.json" x s 2>/dev/null || true)
[ -n "$X" ] || fallo "el aviso no lleva x: es v1, y la prenda es del compromiso v2"
[ "$(stat -c%a "$DIR/keystore.json")" = "600" ] \
  || fallo "el keystore de la siembra no esta en modo 600: es el unico fichero con material de gasto"
IMPORTE=$(qfich "$DIR/aviso.json" amount n)
IXREC=$(qfich "$DIR/credencial.json" index s)
IDREC=$(qfich "$DIR/credencial.json" publicId s)
IDKS=$(qfich "$DIR/keystore.json" public_id s)
[ "$IDREC" = "$IDKS" ] \
  || fallo "el keystore NO lleva la clave del receptor ($IDKS contra $IDREC): un solo productor, y se rompio"
msg "siembra: receptor index $IXREC, importe $IMPORTE; el keystore es el de su credencial"

# ---------------------------------------------------------------------------- EL NODO VIVO
"$NODO" --listen "127.0.0.1:$PORT" --latido "$LATIDO" \
  --clave-fichero "$DIR/semilla.hex" --custodia fichero \
  --diario "$DIR/diario.jsonl" --ledger "$DIR/ledger" \
  --contador-recepcion "$DIR/recepcion.bin" \
  --indice-firma "$DIR/indice-firma.bin" \
  --log warn 2>>"$DIR/nodo.err" &
PID=$!
CAB=""
for _ in $(seq 1 120); do
  V=$(rpc zkssl_signedEpochHead '{}' 2>/dev/null || true)
  case "$V" in *'"available":true'*) CAB="$V"; break;; esac
  sleep 0.5
done
[ -n "$CAB" ] || { sed 's/^/BANCO-PRENDA|   /' "$DIR/nodo.err" >&2; fallo "el nodo no firmo una cabeza"; }
printf '%s' "$CAB" > "$DIR/cabeza.json"
FV=$(python3 -c 'import json,sys; print(int(json.load(open(sys.argv[1]))["result"]["formatVersion"],16))' "$DIR/cabeza.json")
SEQ=$(python3 -c 'import json,sys; print(int(json.load(open(sys.argv[1]))["result"]["seq"],16))' "$DIR/cabeza.json")
[ "$FV" = "5" ] || fallo "el nodo firmo una cabeza v$FV y la prenda exige v5 (D-BF)"
msg "nodo VIVO: cabeza v5 firmada, seq $SEQ"

# ------------------------------------------------------------------------------- LA BOCA
boca(){ # boca <keystore> <frase> <salida> <rotulo> [--publicar]
  local ks="$1" fr="$2" out="$3" rot="$4" pub="${5:-}" i
  for i in 1 2 3 4 5; do
    set +e
    BOCA_S=$("$CLI" --log warn prueba-prenda --nodo "http://127.0.0.1:$PORT" \
      --aviso "$DIR/aviso.json" --keystore "$ks" --frase-fichero "$fr" \
      --index "$IXREC" --salida "$out" $pub 2>&1)
    BOCA_R=$?
    set -e
    case "$BOCA_S" in
      *"cayo un latido entre las dos llamadas"*|*"la ultima cabeza firmada va por el"*)
        msg "carrera del latido en $rot (intento $i de 5): se vuelve a pedir"
        rm -f "$out"; sleep 2; continue ;;
    esac
    return 0
  done
  fallo "la boca cayo CINCO veces por la ventana del latido en $rot: el latido es demasiado corto"
}

# POSITIVO 1: el sobre, sin publicar. La prenda AUTORIZA; publicar la marca es otro acto.
boca "$DIR/keystore.json" "$DIR/frase.txt" "$DIR/prenda.json" "sin-publicar"
[ "$BOCA_R" = "0" ] || fallo "la boca no escribio el sobre: $BOCA_S"
[ -s "$DIR/prenda.json" ] || fallo "la boca dijo que si y no dejo el sobre"
msg "POSITIVO 1: el sobre esta escrito, y el nodo no sabe nada todavia"

# POSITIVO 2: el mismo enunciado, y ademas `zkssl_pledge` bajo el MISMO latido. Lo que se exige es
# que el nodo lo ACEPTE; que la marca quede bajo el consRoot es del banco del consumo (D-AS).
boca "$DIR/keystore.json" "$DIR/frase.txt" "$DIR/prenda-publicada.json" "publicada" --publicar
[ "$BOCA_R" = "0" ] || fallo "la boca no publico: $BOCA_S"
case "$BOCA_S" in
  *'"accepted":true'*) msg "POSITIVO 2: zkssl_pledge ACEPTO la marca bajo el mismo latido" ;;
  *) fallo "zkssl_pledge no la acepto, y la boca no se paro: $BOCA_S" ;;
esac

# RECHAZO EN VIVO, Y ES EL BARATO: una frase que no abre el keystore. La boca muere en el SDK,
# antes de pedirle nada al nodo, y no deja fichero. El texto es de `keystore::load`.
printf 'la frase que no es\n' > "$DIR/frase-mala.txt"
set +e
boca "$DIR/keystore.json" "$DIR/frase-mala.txt" "$DIR/no-debe-existir.json" "frase-mala"
set -e
[ "$BOCA_R" != "0" ] || fallo "la boca ESCRIBIO un sobre con una frase que no abre el keystore"
case "$BOCA_S" in
  *"contrasena incorrecta o fichero manipulado"*)
    msg "RECHAZO EN VIVO: la frase mala se caza en el keystore, SIN tocar el nodo" ;;
  *) fallo "la boca cayo con la frase mala, pero NO por su regla: se esperaba <<contrasena incorrecta o fichero manipulado>> y dijo: $BOCA_S" ;;
esac
[ ! -e "$DIR/no-debe-existir.json" ] || fallo "la boca dejo un fichero al rechazar: un sobre que no prueba no se escribe"

kill -9 "$PID"; wait "$PID" 2>/dev/null || true
PID=""
msg "el nodo esta MUERTO; a partir de aqui NADA necesita red ni libro"

# ---------------------------------------------------------------- EL MANDO, SIN EL NODO
for f in prenda prenda-publicada; do
  set +e; SAL=$("$VER" "$DIR/$f.json" 2>&1); RC=$?; set -e
  [ "$RC" = "0" ] || fallo "el sobre POSITIVO $f dio exit $RC (se esperaba 0): $SAL"
  echo "$SAL" | sed 's/^/BANCO-PRENDA|   /' >&2
done
# Lo que los dos sobres afirman se LEE de ellos. Y se comprueba lo que NINGUN otro banco puede:
# que dos pruebas del MISMO enunciado bajo la MISMA cabeza NO son los mismos bytes.
python3 - "$DIR" "$IDREC" "$SEQ" "$IMPORTE" <<'PY'
import json, sys
d, receptor, seq, importe = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
a = json.load(open(d + "/prenda.json"))
b = json.load(open(d + "/prenda-publicada.json"))
for s, n in ((a, "prenda"), (b, "prenda-publicada")):
    assert s["tipo"] == "prenda" and s["v"] == 1, n
    assert sorted(s["enunciado"]) == ["marca", "receptor"], "%s: el enunciado no es receptor+marca" % n
    assert s["enunciado"]["receptor"] == receptor, "%s: el receptor no es el de la credencial" % n
    assert int(s["cabeza"]["seq"], 16) == seq, "%s: no es la cabeza que el nodo firmo" % n
    assert int(s["cabeza"]["formatVersion"], 16) == 5, "%s: la cabeza no es v5" % n
    assert hex(importe) not in json.dumps(s), "%s: el sobre lleva el importe, que es testigo" % n
assert a["enunciado"] == b["enunciado"], "los dos sobres no afirman lo mismo"
assert a["cabeza"] == b["cabeza"], "los dos sobres no son de la misma cabeza"
assert a["prueba"] != b["prueba"], \
    "las dos pruebas son los MISMOS bytes: la ocultacion del S538 estaria apagada"
print("los dos sobres afirman lo mismo bajo la misma cabeza v5 de seq %d, y sus pruebas DIFIEREN "
      "(%d B y %d B): un vector de prenda se CAPTURA, no se re-deriva"
      % (seq, (len(a["prueba"]) - 2) // 2, (len(b["prueba"]) - 2) // 2))
PY
msg "POSITIVOS: exit 0 los dos, sin el nodo; enunciado de DOS campos y ningun literal del testigo"

# ---------------------------------------------------------------- LOS SIETE NEGATIVOS
ROTOS=0
niega(){ # niega <fichero> <fragmento esperado> <rotulo> [fragmento PROHIBIDO]
  local f="$1" frag="$2" rot="$3" no="${4:-}" s r
  set +e; s=$("$VER" "$f" 2>&1); r=$?; set -e
  [ "$r" = "1" ] || fallo "NEGATIVO $rot dio exit $r (se esperaba 1): $s"
  case "$s" in
    *"$frag"*) : ;;
    *) fallo "NEGATIVO $rot cayo, pero NO por su regla: se esperaba <<$frag>> y dijo: $s" ;;
  esac
  if [ -n "$no" ]; then
    case "$s" in *"$no"*) fallo "NEGATIVO $rot cayo por OTRA regla (<<$no>>), no por la suya: $s" ;; esac
  fi
  msg "NEGATIVO $rot: exit 1 -- $(echo "$s" | tail -n 1)"; ROTOS=$((ROTOS+1))
}

python3 - "$DIR" <<'PY'
import json, os, sys
d = sys.argv[1]
base = json.load(open(d + "/prenda-publicada.json"))
def esc(n, p): open("%s/%s.json" % (d, n), "w").write(json.dumps(p))
def cop(): return json.loads(json.dumps(base))
def miente(h): return h[:-1] + ("0" if h[-1] != "0" else "1")

# Las tres AUSENCIAS: el mando las nombra una a una antes de tocar la firma.
for clave, nombre in (("enunciado", "neg-sin-enunciado"), ("prueba", "neg-sin-prueba"),
                      ("cabeza", "neg-sin-cabeza")):
    p = cop(); del p[clave]; esc(nombre, p)
# La firma que falta: la cabeza deja de ser verificable, y el mando lo dice CON su prefijo.
p = cop(); del p["cabeza"]["signature"]; esc("neg-sin-firma", p)
# La ERA: la prenda exige v5, y con SU razon -no la de sus hermanos- (D-BF).
p = cop(); p["cabeza"]["formatVersion"] = "0x4"; esc("neg-cabeza-v4", p)
# LOS DOS CAMPOS QUE EL ENUNCIADO SI AFIRMA. Son entradas publicas del AIR, asi que la prueba deja
# de verificar el enunciado que el sobre dice. El texto lo pone winterfell y es el MISMO para los
# dos: `niega` solo exige el prefijo del mando.
p = cop(); p["enunciado"]["receptor"] = miente(p["enunciado"]["receptor"]); esc("neg-receptor-mentido", p)
p = cop(); p["enunciado"]["marca"] = miente(p["enunciado"]["marca"]); esc("neg-marca-mentida", p)

# LA PUERTA: un sabotaje que no cambia un byte no prueba nada. Se comprueba AQUI, antes de gastar
# una corrida del verificador, y se dice CUAL.
crudo = json.dumps(base)
iguales = [f for f in sorted(os.listdir(d))
           if f.startswith("neg-") and open(d + "/" + f).read() == crudo]
if iguales:
    print("ROJO: estos sabotajes NO cambian un byte del positivo: " + " ".join(iguales))
    raise SystemExit(3)
print("siete cuerpos negativos derivados por MUTACION del positivo publicado, y los siete DIFIEREN")
PY

niega "$DIR/neg-sin-enunciado.json"     "falta enunciado"                 "sin-enunciado"
niega "$DIR/neg-sin-prueba.json"        "falta prueba o no es cadena 0x"  "sin-prueba"
niega "$DIR/neg-sin-cabeza.json"        "falta cabeza"                    "sin-cabeza"
niega "$DIR/neg-sin-firma.json"         "cabeza: falta signature"         "sin-firma"
niega "$DIR/neg-cabeza-v4.json"         "exige una cabeza v5"             "cabeza-v4"
niega "$DIR/neg-receptor-mentido.json"  "prenda:"                         "receptor-mentido" "exige una cabeza"
niega "$DIR/neg-marca-mentida.json"     "prenda:"                         "marca-mentida"    "exige una cabeza"

# ---------------------------------------------------------------- GUARDAR Y PUREZA
if [ -n "$GUARDAR" ]; then
  for f in "$DIR"/prenda*.json "$DIR"/neg-*.json "$DIR/aviso.json" "$DIR/credencial.json" \
           "$DIR/retorno.json" "$DIR/credencial-pagador.json" "$DIR/keystore.json"; do
    cp "$f" "$GUARDAR/"
    printf 'BANCO-PRENDA|   %-30s %s  %s B\n' "$(basename "$f")" \
      "$(sha256sum "$f" | cut -c1-16)" "$(wc -c < "$f")" >&2
  done
  msg "capturas guardadas en $GUARDAR (de aqui salen los vectores del catalogo prenda-*, por MUTACION)"
  msg "OJO: el keystore lleva una clave de JUGUETE del sandbox, cifrada, pero es material de gasto"
fi

git status --porcelain | sort > "$DIR/porcelain.post"
SUCIO=$(comm -13 "$DIR/porcelain.base" "$DIR/porcelain.post" | wc -l)
if [ "$SUCIO" -ne 0 ]; then
  comm -13 "$DIR/porcelain.base" "$DIR/porcelain.post" | sed 's/^/BANCO-PRENDA|   /' >&2
  fallo "el banco ensucio $SUCIO entradas del arbol: no debe tocarlo"
fi
[ "$ROTOS" = "7" ] || fallo "se esperaban 7 negativos y cayeron $ROTOS"

msg "BANCO-PRENDA VERDE: el prendador probo con su keystore y el nodo vivo, zkssl_pledge acepto la marca, el kit verifico SIN el nodo, la boca se paro en vivo con la frase mala sin llegar a pedir, y las siete reglas del sobre cayeron EN VIVO. Es MEDIA prenda: el PAR con la marca bajo el consRoot es del banco del consumo (D-AS)"
