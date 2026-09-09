# Arqueo — pruebas abiertas de conservación para libros cerrados

> **English:** [`README_EN.md`](./README_EN.md) is this page in English, section by section.

**Antes se llamaba ZK-SSL.** Cambió el nombre del proyecto, no los identificadores publicados:
el cable sigue siendo `zkssl/0.3`, los crates `zk-ssl-*`, y los depósitos con DOI conservan el
título con el que se depositaron. Lo que dice este documento está verificado contra `main` en el
commit `343d3b6`; lo que cambie después lo registra [`AUDITORIA.md`](./AUDITORIA.md), un asiento
por cambio.

Un operador lleva un libro cerrado. Quienes dependen de él —socios, titulares, beneficiarios,
contrapartes— no pueden verlo, y las dos partes no se fían. Hoy ese conflicto lo resuelve un tercero
que abre el libro: un auditor, un supervisor, un juzgado; una vez al año; por muestreo. Arqueo
sustituye la apertura del libro por una **prueba de que el libro hizo lo que sus reglas dicen**, que
cualquiera comprueba **sin el libro, sin red y sin fiarse del autor**. Encaja donde la unidad de
cuenta **nace y muere dentro del libro**: la emite el operador, se mueve entre cuentas, la retira el
operador. Prueba **conservación, no solvencia**: las pruebas hablan del libro, no del mundo.

---

## Leer antes de nada

- **Qué es.** Un motor de libro en Rust: cuentas; pagos en dos fases cuya prueba STARK se genera en
  la máquina del pagador —sin ceremonia de setup y sin curvas—; una **cabeza de época firmada** con
  una firma basada en hashes (XMSS) que el nodo publica; testigos que la **cofirman**; y un
  **paquete de evidencia** que un verificador independiente comprueba **con el nodo apagado**.
- **Qué comprueba hoy un tercero, medido.** Conservación (suministro = saldos + en vuelo, también al
  reabrir); uso único de una etiqueta dentro de un libro y **detección** de la misma etiqueta en dos
  libros; historia no reescribible, con prueba de extensión; inclusión con recibo; autoría sin que
  la clave viaje. La tabla de [«Qué garantiza y qué no»](#qué-garantiza-y-qué-no) da la fuente de
  cada fila.
- **Qué no es.** No es una cadena: un nodo, un escritor, sin consenso distribuido ni token. **El
  operador ve todos los saldos** y puede omitir una operación sin dejar rastro. Entre libros
  **detecta, no previene**.
- **No está auditado por terceros.** Ninguna cantidad de tests propios lo sustituye. Una
  dependencia criptográfica (`xmss`, pre-release) va clavada con `=` y declarada. Todo en
  [`SECURITY.md`](./SECURITY.md).
- **Los seis depósitos con DOI preceden a correcciones del árbol.** Lo que se corrigió se marca, no
  se borra: [`doc/preprints/ERRATA.md`](./doc/preprints/ERRATA.md).

---

## Pruébalo en cinco minutos

### Sin Rust y sin repositorio: el kit del verificador

Una descarga, y cuatro comprobaciones en la máquina del que comprueba, sin red. La release vigente
es `arqueo-verify-v0.2.0`, producida sobre el commit `1528943fdfb9399f56fd836f75ffbe655d004d78`;
la huella del tarball se publica con su commit al lado, en la página de la release y en el asiento
de `AUDITORIA.md` que la registra. El guion completo, con la salida esperada de cada paso, es
[`doc/KIT.md`](./doc/KIT.md).

```bash
sha256sum arqueo-verify-*.tar.gz          # tiene que ser la huella publicada junto al commit
tar xzf arqueo-verify-*.tar.gz && cd arqueo-verify-*/
sha256sum -c SHA256SUMS                   # cada fichero de dentro, contra su huella
cat VERSION                               # commit, describe, toolchain, glibc_max
```

```bash
# 1 · un expediente que cuadra                       -> VERDE, salida 0
./zk-ssl-verify spec/vectors/paquete/posicion-v2.json
./zk-ssl-verify spec/vectors/consumo/consumo.json

# 2 · uno manipulado no cuadra, y el programa nombra la regla rota  -> salida 1
./zk-ssl-verify spec/vectors/paquete/rechazo-n-adulterado.json; echo "salida $?"
./zk-ssl-verify spec/vectors/consumo/rechazo-cons-ausencia-ya-estaba.json; echo "salida $?"

# 3 · la misma etiqueta en dos libros, con los dos nodos apagados  -> DETECCION, no prevencion
./zk-ssl-verify spec/vectors/conflicto/conflicto.json

# 4 · un intercambio de libros se rechaza con su nombre            -> salida 1 en los dos
./zk-ssl-verify spec/vectors/conflicto/rechazo-conf-camino-no-sube.json; echo "salida $?"
./zk-ssl-verify spec/vectors/conflicto/rechazo-conf-misma-clave.json; echo "salida $?"
```

Y los catálogos enteros, con el arnés que viaja dentro del tarball: `bash conformidad.sh
./zk-ssl-verify` comprueba entrada a entrada que cada vector dice lo que su manifiesto declara.

### Con Rust estable

`--release` es obligatorio para la capa, no una optimización (los circuitos validan en depuración
con grados que dependen del testigo).

```bash
git clone https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers
cd Arqueo-open-conservation-proofs-for-closed-ledgers

# un pago completo (fondear ×2 -> enviar -> cobrar) con pruebas STARK reales y traza por fases
cargo run --release -p zk-ssl-cli -- simulate --amount 250000

# el contrato de una segunda implementación: re-ejecuta el escenario canónico y compara campo a campo
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.3.json

# el canon: la tabla de tests por crate (--lista sólo la enseña; --sello la corre entera)
bash tools/canon.sh --lista
```

Deben salir `cadena de transiciones íntegra` y `CONFORMIDAD: … todo IDENTICO`. La CLI entera
—`simulate`, `trace-tx`, `inspect-state`, `conformance`— está en [`doc/README-CLI.md`](./doc/README-CLI.md).

---

## Cómo funciona, en un párrafo

Un pago son dos transiciones: el pagador **envía** (una hoja) y el receptor **cobra** (otra), cada
uno con una prueba generada en su máquina; la capa entrega caminos y raíces —datos públicos—,
verifica pruebas y **nunca ve una clave de gasto**. Cada época el nodo firma una **cabeza** que ata
las raíces del estado en reposo, el registro encadenado de transiciones y el árbol de consumos
publicados; testigos independientes la cofirman y **fijan la clave la primera vez que la ven**. Un
**paquete de evidencia** lleva una cabeza firmada, un acuse con su camino y las cofirmas: un
verificador que no conoce al nodo lo recompone y lo acepta o lo rechaza **nombrando la regla**. El
núcleo está en [`spec/NUCLEO.md`](./spec/NUCLEO.md), el paquete en
[`spec/PAQUETE.md`](./spec/PAQUETE.md) y el consumo publicado en
[`spec/rfc/0006-consumo-publicado.md`](./spec/rfc/0006-consumo-publicado.md).

---

## Qué garantiza y qué no

La misma tabla, en inglés y con los casos de uso, está en [`doc/USE_CASES.md`](./doc/USE_CASES.md).

| # | propiedad | lo que un tercero comprueba | estado |
|---|---|---|---|
| 1 | Conservación | suministro = saldos + en vuelo; nada se crea ni se pierde entre épocas | medida, en vuelo y al reabrir (`AUDITORIA.md` §387–§394) |
| 2 | Uso único | una etiqueta se consume una vez en un libro y se publica en su cabeza firmada; la misma etiqueta en dos libros se detecta desde las dos | medida (RFC-0006; `doc/KIT.md`) |
| 3 | Historia no reescribible, con prueba de extensión | la cabeza de hoy extiende la de ayer sin borrar ni reordenar | medida (`spec/RPC.md`, `zkssl_consistencyProof`) |
| 4 | Inclusión con recibo | una entrada está en el libro, demostrable sin el operador | medida (`spec/RPC.md`, `zkssl_inclusionReceipt`, `zkssl_ackPath`) |
| 5 | Autoría sin que la clave viaje | sólo quien tiene la clave mueve su cuenta; el operador no puede | medida (`spec/RPC.md`, el principio de la API) |
| 6 | Corte y completitud | nada queda en vuelo pasado su plazo; cada acuse acaba aplicado o rechazado, con traza | planeada |
| 7 | Rechazo con causa | una negativa lleva la regla que la produjo | planeada |

Las filas 6 y 7 **no existen en el árbol**: se listan para que se sepa qué preguntas quiere
responder el motor y todavía no responde.

**Lo que nada de esto afirma:**

- Privacidad frente al operador: el operador lo ve todo.
- Que las unidades del libro existan fuera del libro.
- Que una operación omitida se detecte: la censura no deja rastro.
- Prevención entre libros: dos libros pueden aceptar la misma etiqueta; un tercero con las dos
  cabezas firmadas lo ve después, nunca antes.
- Quién está detrás de una clave, ni que una persona tenga una sola cuenta.
- Las filas 6 y 7 como existentes.

**Lo que falta, por orden de importancia:** consenso distribuido (sin él, el operador ve los saldos
y puede censurar; la alternativa que este proyecto sí persigue —responsabilidad demostrable, al modo
de Certificate Transparency— tiene el firmante, el guardián del índice, el latido, el verificador
independiente y los testigos, y le falta un ancla anterior al primer encuentro y una custodia de
clave **comprobada**, no sólo declarada) · auditoría externa · el recibo de admisión (`AUDITORIA.md`
§121) · delegar la prueba a terceros (verificar la firma en circuito) · una política de caducidad
para las congelaciones. Todo lo demás está enumerado en [`AUDITORIA.md`](./AUDITORIA.md), sección 4.

---

## Orden de lectura

| Si eres… | Empieza por |
|---|---|
| **Un evaluador o un auditor externo** | [`doc/KIT.md`](./doc/KIT.md) · [`doc/USE_CASES.md`](./doc/USE_CASES.md) · [`SECURITY.md`](./SECURITY.md) · [`AUDITORIA.md`](./AUDITORIA.md) |
| **Vas a implementar el protocolo** | [`spec/README.md`](./spec/README.md) · [`spec/RPC.md`](./spec/RPC.md) + [`spec/openrpc.json`](./spec/openrpc.json) · [`spec/vectors/`](./spec/vectors/) · [`spec/PAQUETE.md`](./spec/PAQUETE.md) · [`spec/rfc/`](./spec/rfc/) |
| **Un criptógrafo** | [`spec/NUCLEO.md`](./spec/NUCLEO.md) · [`ARQUITECTURA.md`](./ARQUITECTURA.md) · [`doc/VERIFICACION_FORMAL.md`](./doc/VERIFICACION_FORMAL.md) + [`doc/fv/mapa_fv_capas.md`](./doc/fv/mapa_fv_capas.md) · [`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md) |
| Quieres el planteamiento | [`PRINCIPIOS.md`](./PRINCIPIOS.md) · [`doc/IDEA_CENTRAL.md`](./doc/IDEA_CENTRAL.md) · [`doc/APORTACION.md`](./doc/APORTACION.md) · [`doc/CONSECUENCIAS.md`](./doc/CONSECUENCIAS.md) |
| Tienes preguntas | [`PREGUNTAS.md`](./PREGUNTAS.md) · [`QUESTIONS.md`](./QUESTIONS.md) |
| Tienes cinco minutos y no eres técnico | [`RESUMEN_EJECUTIVO.md`](./RESUMEN_EJECUTIVO.md) · [`RESUMEN_BILINGUE.md`](./RESUMEN_BILINGUE.md) |
| Instituciones, y los límites de escala | [`doc/INSTITUCIONAL.md`](./doc/INSTITUCIONAL.md) · [`doc/INSTITUTIONAL.md`](./doc/INSTITUTIONAL.md) |
| Llegas desde Zenodo | [`doc/ZENODO.md`](./doc/ZENODO.md) |
| Vas a contribuir o a reportar una vulnerabilidad | [`CONTRIBUTING.md`](./CONTRIBUTING.md) · [`SECURITY.md`](./SECURITY.md) |

`AUDITORIA.md` incluye una sección con **los puntos donde el autor tiene menos confianza**. Si vas
a mirar el código con intención de romperlo, empieza ahí.

---

## Estado

| pieza | dónde se mide |
|---|---|
| **17 crates** en un workspace; el canon (`tools/canon.sh --sello`) corre los tests de todos, en release, y las ocho herramientas de `tools/` que vigilan cifras, citas, dominios y geometría | la tabla de [`tools/canon.sh`](./tools/canon.sh) lleva los tests que pasan por crate; cada sello la actualiza |
| **Protocolo `zkssl/0.3`**: 26 métodos JSON-RPC (24 `zkssl_*`, 2 `dev_*`), OpenRPC generado desde el código, vectores por versión que jamás se reescriben | [`spec/RPC.md`](./spec/RPC.md) · [`spec/openrpc.json`](./spec/openrpc.json) · [`spec/vectors/`](./spec/vectors/) (135 ficheros: cable, núcleo, paquete, consumo, conflicto y los tres `zkssl-0.N.json`) |
| **RFC**: 0002, 0003, 0004 y 0006 aceptados; 0005 (el núcleo congelado) propuesto | [`spec/rfc/`](./spec/rfc/) |
| **Verificador independiente** `zk-ssl-verify` 0.2.0, release `arqueo-verify-v0.2.0`, reproducible desde el commit que su `VERSION` nombra | [`doc/KIT.md`](./doc/KIT.md) · [`tools/artefacto.sh`](./tools/artefacto.sh) |
| **Registro**: un asiento por cambio verificado, con su commit; lo corregido se marca, no se borra | [`AUDITORIA.md`](./AUDITORIA.md) · [`BACKLOG.md`](./BACKLOG.md) |

El diseño se eligió midiendo **el mismo circuito en cinco sistemas de prueba**
([`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md)); STARK/FRI sin ceremonia fue la única decisión tomada
contra los números, porque una ceremonia cuyos participantes coluden puede crear dinero sin dejar
rastro. Los tiempos y tamaños medidos, con su dispersión, están en `AUDITORIA.md` (§130 y §131 la
dispersión entre tandas, §229 el techo del nodo por RPC); aquí no se repiten para que no envejezcan.

---

## Publicación

Los preprints del proyecto, con su DOI vigente en Zenodo. Las
versiones anteriores siguen accesibles y se citan aquí: **una cifra
publicada que se corrige no se borra, se marca**.

**Comparative Implementation of a Zero-Knowledge Settlement Layer across Five
Proof Systems: Design Findings and Measurements**
DOI: [10.5281/zenodo.21736125](https://doi.org/10.5281/zenodo.21736125)

*Versiones anteriores: [10.5281/zenodo.21683239](https://doi.org/10.5281/zenodo.21683239)
y [10.5281/zenodo.21677737](https://doi.org/10.5281/zenodo.21677737). La primera
publica 59,1 MB por mil operaciones y 17,5 % de aplicar sobre generar: las dos
cifras miden la vía de un paso, **retirada desde entonces** (§31, §32).*

**Provable Compliance without Full Ledger Disclosure — A Zero-Knowledge
Settlement Architecture for Supervisory Audit**
DOI: [10.5281/zenodo.21736082](https://doi.org/10.5281/zenodo.21736082)

*Versión anterior: [10.5281/zenodo.21678396](https://doi.org/10.5281/zenodo.21678396).*

**From Institutional Trust to Verifiable Properties — A Minimal ZK Settlement
Layer and Its Residual Trust Surface**
DOI: [10.5281/zenodo.21905595](https://doi.org/10.5281/zenodo.21905595)

*Versión anterior: [10.5281/zenodo.21679208](https://doi.org/10.5281/zenodo.21679208).*

**Reputation, Residual Power, and Verification Investment in Digital Settlement**
DOI: [10.5281/zenodo.22078086](https://doi.org/10.5281/zenodo.22078086)

**Residual Surfaces in Retail CBDC Incidents and the Verification–Residual Partition: A Coding Study with Contrast to an Explicit Residual-Trust Settlement Design**
DOI: [10.5281/zenodo.22077991](https://doi.org/10.5281/zenodo.22077991)

**Residual Trust After Verification: A Microeconomic Account of What Proofs Cannot Eliminate**
DOI: [10.5281/zenodo.22076721](https://doi.org/10.5281/zenodo.22076721)

### Qué corrige la tercera revisión

> ⚠️ **Esta tabla es HISTORICA.** Dice que corrigio *esa* revision respecto
> de la anterior, y sus cifras son las de entonces. **No se actualizan**:
> meterle los numeros de hoy reescribiria lo que se publico. Para el estado
> actual, `AUDITORIA.md`.

| Corrección | Antes | Ahora | Ver |
|---|---|---|---|
| Cobertura de la prueba por mutación | 12 circuitos limpios | **11** cubiertos (10 de producción); el informe del duodécimo salió de una traza inválida | §12, §20 |
| Árbol de nullificadores | «se conserva por compatibilidad, es peso muerto» | **retirado** con migración verificada; instantánea v4 que sigue leyendo v3 | §32, §36 |
| Confidencialidad frente al receptor | condicionada a que la vía en dos fases fuera la única | la condición **se cumplió**: es la única vía | §32 |
| Tests de los dos crates | 369 | **375** | — |
| Tests que fallan sin `--release` | 56 | **65** de 174, medido | §20 |

⚠️ **Las referencias cruzadas entre los tres preprints apuntan a versiones
anteriores de sus compañeros**, no a las terceras revisiones. Los enlaces
resuelven y el contenido citado sigue siendo el correcto, pero un lector que
los siga leerá una versión con cifras ya corregidas. Se arreglará en la
próxima revisión de los tres.

## Autoría y licencia

**Angel Toranzo Portela**, 2026. Cómo citar: [`CITATION.cff`](./CITATION.cff).

Licenciado bajo **MIT** o **Apache-2.0**, a elección de quien lo use.

Las dos licencias exigen **conservar el aviso de copyright y el texto de la
licencia** en cualquier copia o trabajo derivado. Apache-2.0 exige además
respetar el fichero [`NOTICE`](./NOTICE).

### Código de terceros

`crates/ceremony/` **no es código original de este proyecto**: procede de
`penumbra-sdk-proof-setup` (Penumbra Labs), bajo la misma licencia dual.
Ver [`crates/ceremony/ATTRIBUTION.md`](./crates/ceremony/ATTRIBUTION.md).

### ⚠️ Sin afiliación institucional

Este proyecto es un trabajo **independiente**. **No está afiliado,
respaldado ni encargado por el Banco Central Europeo, el Eurosistema, ni
ninguna otra institución** pública o privada.

Las referencias al euro digital son al diseño público publicado por esas
instituciones y se citan como contexto de un problema técnico. **Ninguna
afirmación de este repositorio debe leerse como posición de nadie más que
su autor.**
