#!/bin/bash
# Genera el PDF de un preprint a partir de su fuente markdown.
#
#   ./generar.sh ZK-SSL-preprint
#
# Reproduce los PDF publicados en Zenodo. Necesita pandoc y wkhtmltopdf;
# los publicados se hicieron con wkhtmltopdf 0.12.6.
set -e
base="$1"
[ -z "$base" ] && { echo "uso: ./generar.sh <nombre-sin-extension>"; exit 1; }
cd "$(dirname "$0")"
pandoc "$base.md" -f markdown+pipe_tables -t html5 -s --metadata title="" \
  -c estilo.css -o "$base.html"
wkhtmltopdf --quiet --enable-local-file-access \
  --margin-top 18mm --margin-bottom 18mm --margin-left 16mm --margin-right 16mm \
  "$base.html" "$base.pdf"
rm -f "$base.html"
echo "generado: $base.pdf"
