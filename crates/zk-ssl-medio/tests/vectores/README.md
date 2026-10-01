# Vectores del medio: procedencia y licencias

Tres corpus de dos manos ajenas, copiados **byte a byte** (§631). Ninguno lo
escribió Arqueo; lo que Arqueo añade está en los `.rs` de `tests/` y se dice
allí. El cuarto, las notas `checkpoint` contrastadas con torchwood (§632),
tiene su propio `notas/README.md`.

## 1. Los vectores acumulados del borrador del IETF

`../vectores_ietf.rs` no lee ficheros: lleva dentro los cuatro hashes finales
del apéndice «Test Vectors» de `draft-ietf-plants-merkle-tree-certs` (712
hashes de subárbol, 12.807 caminos de inclusión, 42.893 de consistencia y
8.646 coberturas: 65.058 casos sobre todo subárbol de todo árbol hasta 130
hojas). El fichero es `tests/vectors.rs` de mtc-core
(`github.com/atoranzo/mtc-core`, commit
`d3b0ca614e51f177a0c30f1d21abd87b91208f59`), con las adaptaciones marcadas
`ADAPTADO (§631)` en su cabecera.

## 2. Los vectores grandes del mismo borrador

`large_inclusion_proofs.json` y `large_consistency_proofs.json` son los
ficheros que referencia el apéndice «Large Subtree Test Vectors»: pruebas
sobre árboles de `2^48-1`, `2^63-1` y `2^64-1` hojas, que nadie construye y
que el **verificador** tiene que evaluar sin desbordar. Vienen del directorio
`demo/` del repositorio del grupo PLANTS (`ietf-plants-wg/merkle-tree-certs`,
commit `99097c9`, 2026-09-29) a través de mtc-core `d3b0ca6`
(`tests/vectors/`), sin cambiar un byte:

| fichero | SHA-256 (16 primeros) |
|---|---|
| `large_consistency_proofs.json` | `7fcc6d9d0283ecf7` |
| `large_inclusion_proofs.json` | `19ffc377d123613d` |

El repositorio de origen declara que todo su material son contribuciones al
proceso de normalización del IETF (BCP 78, BCP 79 y las *IETF Trust Legal
Provisions*), y que sus componentes de código, vectores incluidos, están
bajo la **Simplified BSD License** del IETF Trust. Se conservan aquí bajo
esa licencia y con esta atribución, aparte de la licencia MIT OR Apache-2.0
del resto del crate:

> Copyright (c) IETF Trust and the persons identified as authors of the
> code. All rights reserved. Redistribution and use in source and binary
> forms, with or without modification, is permitted pursuant to, and
> subject to the license terms contained in, the Simplified BSD License
> set forth in Section 4.c of the IETF Trust's Legal Provisions Relating
> to IETF Documents (https://trustee.ietf.org/license-info).

## 3. Las sondas de transparency-dev

`transparency-dev/` son las cuatro carpetas de `testdata/` de
`github.com/transparency-dev/merkle`, commit
`fbbcd741c3d1c69d8498487baa8edc9e5824847c` (2026-09-21): `inclusion` (98),
`consistency` (98), `subtreeinclusion` (204) y `subtreeconsistency` (285),
685 ficheros JSON que genera su `cmd/proofgen`. El SHA-256 de la lista
ordenada de sus `sha256sum` (`find . -type f | LC_ALL=C sort | xargs
sha256sum | sha256sum`, desde la carpeta) es
`d9edf5f2c30801c8acde4c783ecbad3e30ecb40eebda2cf45428c449d204da6e`, el mismo
aquí que en el origen.

Es la biblioteca con la que el testigo de transparency-dev comprueba la
consistencia de un checkpoint antes de cofirmarlo
(`github.com/transparency-dev/witness`, `witness/witness.go`,
`proof.VerifyConsistency(rfc6962.DefaultHasher, ...)`, commit `b4c9458`).

Licencia: **Apache License 2.0** (Copyright 2025 Google LLC, cabecera de
`cmd/proofgen/main.go`), la misma que una de las dos de Arqueo; su texto
está en `LICENSE-APACHE`, en la raíz del repositorio. Los JSON no llevan
cabecera propia y no se han modificado.

`../vectores_transparency.rs` cuenta aparte lo que rechaza el **tipo** (un
hash que no mide 32 bytes) y lo que rechaza el **árbol**, y fija por nombre
las cuatro positivas que aquí no se pueden escribir. Por qué importa esa
separación —un punto ciego medido— está en el test
`sus_positivas_con_un_bit_cambiado_en_32_bytes`.
