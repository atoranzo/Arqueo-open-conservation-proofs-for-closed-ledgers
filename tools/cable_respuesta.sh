#!/usr/bin/env bash
# tools/cable_respuesta.sh - el ADAPTADOR del consumidor del cable al contrato del mando (RFC-0005 E3, S409).
# `tools/conformidad.sh <binario> [manifiesto]` exige un ejecutable de UN argumento con exit 0/1/2. El
# consumidor del cable de la referencia es el testigo (`zk-ssl-cli witness --respuesta`), que no es un
# binario propio: este fichero une los dos contratos sin doblar ninguno.
#   bash tools/conformidad.sh tools/cable_respuesta.sh spec/vectors/cable/MANIFIESTO.txt
# Usa el binario de release del target que da tools/target_dir.sh (o ZK_SSL_CLI); una segunda implementacion pone aqui el suyo.
[ $# -eq 1 ] || { echo "uso: tools/cable_respuesta.sh <respuesta.json>" >&2; exit 2; }
[ -n "${ZK_SSL_CLI:-}" ] || T=$(bash "$(dirname "${BASH_SOURCE[0]}")/target_dir.sh") || { echo "ROJO: no se donde deja cargo el binario (tools/target_dir.sh)" >&2; exit 2; }
BIN="${ZK_SSL_CLI:-$T/release/zk-ssl-cli}"  # ⚠️ §844: decia target/release/, sin CARGO_TARGET_DIR; ⚠️ §855: decia ${CARGO_TARGET_DIR:-target}
[ -x "$BIN" ] || { echo "ROJO: no existe el binario $BIN (cargo build --release -p zk-ssl-cli)" >&2; exit 2; }
exec "$BIN" witness --respuesta "$1"
