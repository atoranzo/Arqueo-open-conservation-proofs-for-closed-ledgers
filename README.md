# Arqueo — pruebas abiertas de conservación para libros cerrados

> **English:** [`README_EN.md`](./README_EN.md) is this page in English, section by section.

**Antes se llamaba ZK-SSL.** Cambió el nombre del proyecto, no los identificadores publicados:
el cable era `zkssl/0.3` y desde el asiento 538 es `zkssl/0.4` (por la ocultación, RFC-0009; no
por el nombre; y esa etiqueta no dice qué pruebas acepta un nodo: «Estado»), los crates `zk-ssl-*`, y los depósitos con DOI conservan el
título con el que se depositaron. Lo que dice este documento está verificado contra el commit
`8c02941` (el §789), en el §794; lo que cambie después lo registra [`AUDITORIA.md`](./AUDITORIA.md),
un asiento por cambio. ⚠️ **§794**: hasta aquí decía «verificado contra `main` en el commit
`343d3b6`», el del §444; lo que la relectura del §794 corrigió lleva su marca en su sitio. Esa
relectura no compiló: las salidas que esta página espera de `simulate`, de `conformance` y del kit
se contrastaron con el código, el canon y los manifiestos, no se corrieron.

Un operador lleva un libro cerrado. Quienes dependen de él —socios, titulares, beneficiarios,
contrapartes— no pueden verlo, y las dos partes no se fían. Hoy ese conflicto lo resuelve un tercero
que abre el libro: un auditor, un supervisor, un juzgado; una vez al año; por muestreo. Arqueo
sustituye la apertura del libro por **pruebas de que el libro hizo lo que sus reglas dicen**. Una
parte la comprueba cualquiera **sin el libro, sin red y sin fiarse del autor**: la historia, la
inclusión con recibo, el uso único de una etiqueta, el corte y la completitud y, para las causas
que cubre el sobre de rechazo, la causa de una negativa. Otra la comprueba hoy sólo el nodo: la
**conservación** y la **autoría** de cada pago van dentro de su prueba, y esa prueba la verifica el
nodo antes de aplicarlo, no el verificador del tercero (filas 1 y 5 de [«Qué garantiza y qué
no»](#qué-garantiza-y-qué-no)). Encaja donde la unidad de cuenta **nace y muere dentro del libro**:
la emite el operador, se mueve entre cuentas, la retira el operador. Prueba **conservación, no
solvencia**: las pruebas hablan del libro, no del mundo.

---

## Leer antes de nada

- **Qué es.** Un motor de libro en Rust: cuentas; pagos en dos fases cuya prueba STARK se genera en
  la máquina del pagador —sin ceremonia de setup y sin curvas—; una **cabeza de época firmada** con
  una firma basada en hashes (XMSS) que el nodo publica; testigos que la **cofirman**; y un
  **paquete de evidencia** que un verificador independiente comprueba **con el nodo apagado**.
- **Qué comprueba hoy un tercero, medido.** Uso único de una etiqueta dentro de un libro y
  **detección** de la misma etiqueta en dos libros; historia no reescribible, con prueba de
  extensión; inclusión con recibo. La tabla de [«Qué garantiza y qué no»](#qué-garantiza-y-qué-no)
  da la fuente de cada fila.
- **Qué comprueba el nodo, y un tercero todavía no.** La conservación de cada transición —en el
  envío y el cobro, el saldo cambia exactamente en el importe y el suministro no cambia; en la
  quema, los dos bajan en el importe— y la autoría —sólo quien tiene la clave mueve su cuenta— las
  imponen en circuito las pruebas de envío, cobro y quema; el agregado (suministro = saldos + en
  vuelo) lo comprueba la capa al reabrir el libro. El nodo verifica cada una de esas pruebas antes
  de aplicar la transición, y el registro guarda sólo su resumen: el verificador del tercero no las
  vuelve a verificar, ni las de emisión y reembolso. Del suministro, un tercero ve el que firma la
  cabeza; de la autoría, la de la prenda.
- **Qué no es.** No es una cadena: un nodo, un escritor, sin consenso distribuido ni token. **El
  operador ve todos los saldos** y puede omitir una operación: sin dejar rastro si no emite
  recibo; si lo emitió, el sobre de completitud lo nombra (RFC-0010). Entre libros **detecta, no
  previene**.
- **No está auditado por terceros.** Ninguna cantidad de tests propios lo sustituye. Dos
  dependencias criptográficas sin auditoría independiente —`xmss`, pre-release, y `ml-dsa`, en
  el kit desde el §633— van clavadas con `=` y declaradas, y desde el §694 lo comprueba el canon,
  con las de winterfell y la clausura del kit como lista cerrada. Y una tercera, `argon2` 0.5.3, el
  KDF de reposo de la capa desde el §702, va clavada igual y sin auditar por el proyecto
  (`AUDITORIA.md` §702: «No audita `argon2` 0.5.3»); de una auditoría por terceros no hay registro
  en el árbol. El vector del RFC 9106 y los KAT comprueban la salida, no el código, y su fijación la
  comprueba un test de la capa, `argon2_entra_clavada_y_solo_por_la_capa`. Lo que no está auditado,
  en [`SECURITY.md`](./SECURITY.md); lo que declaran las dos primeras, en el manifiesto que las
  clava (`crates/zk-ssl-node/Cargo.toml`:48-52, `crates/zk-ssl-medio/Cargo.toml`:29-36), y por qué
  va clavada la tercera, en el suyo (`crates/zk-ssl/Cargo.toml`:45-58). ⚠️
  **Corregido en el §794**: decía «Todo en `SECURITY.md`», que no dice de ninguna de las dos que vaya
  sin auditoría ni clavada: nombra XMSS y ML-DSA sólo como esquemas (`SECURITY.md` §2, «Qué NO
  protege», y §3.11; ⚠️ §893: decía `:189` y `:728`, que eran sus líneas en el §794).
  ⚠️ **Corregido en el §819**: no nombraba `argon2`, clavada desde el §702 y sin auditar por el
  proyecto (`AUDITORIA.md` §702), y decía «lo que declaran las dos dependencias, en el manifiesto que
  las clava».
- **Los seis depósitos con DOI preceden a correcciones del árbol.** Lo que se corrigió se marca, no
  se borra: [`doc/preprints/ERRATA.md`](./doc/preprints/ERRATA.md).

---

## Pruébalo en cinco minutos

### Sin Rust y sin repositorio: el kit del verificador

Una descarga, y cinco comprobaciones en la máquina del que comprueba, sin red. La release vigente
es `arqueo-verify-v0.4.3`, producida sobre el commit `5fd5212b9f6e04c4c619655ffe969ab04116477a`;
la huella del tarball se publica con su commit al lado, en la página de la release y en el asiento
de `AUDITORIA.md` que la registra. El guion completo, con la salida esperada de cada paso, es
[`doc/KIT.md`](./doc/KIT.md). Recompone cabezas hasta la v6, la que el nodo firma hoy, y lee doce
familias de sobres, la duodécima la de la rotación (§649); la 0.2.0, que se queda en la v4,
sigue publicada y rechaza lo posterior —falla cerrada, nunca un VERDE falso—. Medido y declarado
en `doc/KIT.md`, sección 0. ⚠️ **Corregido en el §794**: decía «lee las diez familias de sobres»;
desde la 0.4.0 son once (`doc/KIT.md`:27-28, :98; ⚠️ §823: decía :40, que corrieron el §815, con su aviso de la 0.4.3, y el §818, que la da por vigente), y el árbol monta ya doce, con la rotación
(§649; `FAMILIAS` en `tools/artefacto.sh`:31), que ninguna release publicada lleva. ⚠️ **§818**:
la vigente era la `arqueo-verify-v0.4.2`, sobre `a4e888d`, y este párrafo decía «lee once familias
de sobres, la undécima la del ancla cofirmada (§634)»; la 0.4.3 es la primera release publicada que
lleva la rotación, un arreglo de seguridad con su aviso (`doc/KIT.md`, sección 0), y la 0.4.2 sigue
publicada, con los defectos que la 0.4.3 corrige, el del aviso entre ellos (§815).

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

# 5 · lo que el nodo RECIBIO bajo su firma: resuelto en su ventana, o el operador nombrado
./zk-ssl-verify spec/vectors/completitud/resuelta-por-acuse.json; echo "salida $?"   # -> 0
./zk-ssl-verify spec/vectors/completitud/no-resuelta.json; echo "salida $?"          # -> 1
```

Y los catálogos enteros, con el arnés que viaja dentro del tarball: `bash conformidad.sh
./zk-ssl-verify` comprueba entrada a entrada que cada vector del paquete dice lo que su manifiesto
declara, y con `spec/vectors/<familia>/MANIFIESTO.txt` como segundo argumento, el de cada una de
las otras. ⚠️ **Corregido en el §794**: decía que esa línea sola comprueba «cada vector»; sin
segundo argumento, el arnés corre el manifiesto del paquete (`tools/conformidad.sh`:16, :48).

### Con Rust estable

`--release` es obligatorio para la capa, no una optimización (los circuitos validan en depuración
con grados que dependen del testigo).

```bash
git clone https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers
cd Arqueo-open-conservation-proofs-for-closed-ledgers

# un pago completo (fondear ×2 -> enviar -> cobrar) con pruebas STARK reales y traza por fases
cargo run --release -p zk-ssl-cli -- simulate --amount 250000

# el contrato de una segunda implementación: re-ejecuta el escenario canónico y compara campo a campo
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.4.json

# el canon: la tabla de tests por crate (--lista sólo la enseña; --sello la corre entera)
bash tools/canon.sh --lista
```

Deben salir `cadena de transiciones íntegra` y `CONFORMIDAD: … todo IDENTICO`. Los cuatro mandos
de la sandbox —`simulate`, `trace-tx`, `inspect-state`, `conformance`— están en
[`doc/README-CLI.md`](./doc/README-CLI.md); la CLI tiene otros cuatro que esa página no cuenta:
`witness`, el testigo, y `prueba-cobro`, `prueba-pago` y `prueba-prenda`, que escriben los sobres
del pendiente del RFC-0008 con lo que piden a un nodo vivo. ⚠️ **Corregido en el §794**: el
`--check` iba contra `zkssl-0.3.json`, que desde el §538 se rechaza como «de OTRA version» —el
canon lo exige así (su bloque «3 · conformidad», en `tools/canon.sh`; `crates/zk-ssl-cli/src/conformance.rs`:238-242)—, y
esta línea decía «La CLI entera» de esos cuatro (`crates/zk-ssl-cli/src/main.rs`:59-86 tiene ocho). ⚠️ **Corregido en el §847**: citaba ese bloque del canon por línea, `:404-407`, y el fichero creció por encima: hoy esas líneas son el comentario del bloque «2 ter». Se cita el bloque.

---

## Cómo funciona, en un párrafo

Un pago son dos transiciones: el pagador **envía** (una hoja) y el receptor **cobra** (otra), cada
uno con una prueba generada en su máquina; la capa entrega caminos y raíces —datos públicos— y
verifica pruebas. **La clave de gasto no viaja por la API y, desde el §538, tampoco sale literal en
la prueba**: el probador de la casa oculta su testigo (RFC-0009 E3b-2, `zkssl/0.4`); entre el §521
y el §538 la prueba la publicaba (ver [`SECURITY.md`](./SECURITY.md) §3.bis). Cada época el nodo
firma una **cabeza** que ata las raíces del estado en reposo, el registro encadenado de
transiciones y el árbol de consumos publicados; testigos independientes la cofirman y **fijan la
clave la primera vez que la ven**. Un **paquete de evidencia** lleva una cabeza firmada, un acuse
con su camino y las cofirmas: un verificador que no conoce al nodo lo recompone y lo acepta o lo
rechaza **nombrando la regla**. El núcleo está en [`spec/NUCLEO.md`](./spec/NUCLEO.md), el paquete
en [`spec/PAQUETE.md`](./spec/PAQUETE.md) y el consumo publicado en
[`spec/rfc/0006-consumo-publicado.md`](./spec/rfc/0006-consumo-publicado.md).

---

## Qué garantiza y qué no

La misma tabla, en inglés y con los casos de uso, está en [`doc/USE_CASES.md`](./doc/USE_CASES.md).

| # | propiedad | lo que un tercero comprueba | estado |
|---|---|---|---|
| 1 | Conservación | el suministro que firma la cabeza (desde la v5) y, ante un rechazo por tope, que el suministro de la cabeza más el importe pasa el tope comprometido (`spec/PAQUETE.md`, 2.6); **no** que suministro = saldos + en vuelo | medida **en el nodo**: en circuito, en cada envío, cobro y quema, y el agregado en la capa al reabrir (`AUDITORIA.md` §379; las raíces, §387, §388, §391 y §392); **sin el nodo, no**: el kit no verifica pruebas de transición —tampoco las de emisión y reembolso—, y el registro no las guarda |
| 2 | Uso único | una etiqueta se consume una vez en un libro y se publica en su cabeza firmada; la misma etiqueta en dos libros se detecta desde las dos | medida (RFC-0006; `doc/KIT.md`) |
| 3 | Historia no reescribible, con prueba de extensión | la cabeza de hoy extiende la de ayer sin borrar ni reordenar | medida (`spec/RPC.md`, `zkssl_consistencyProof`) |
| 4 | Inclusión con recibo | una entrada está en el libro, demostrable sin el operador | medida (`spec/RPC.md`, `zkssl_inclusionReceipt`, `zkssl_ackPath`) |
| 5 | Autoría sin que la clave viaje | de la prenda, que la produjo quien tiene la clave (`spec/PAQUETE.md`, 2.10); **no** que sólo quien tiene la clave mueva su cuenta | medida **en el nodo**: en circuito (`C_PK_CHECK`), en cada envío, cobro y quema; **sin el nodo, sólo la prenda**. Que la clave no viaje: `spec/RPC.md`, el principio de la API |
| 6 | Corte y completitud | nada queda en vuelo pasado su plazo; toda operación que el nodo recibe acaba aplicada, rechazada con prueba o declarada, o un rojo nombrado dice que no | medida (RFC-0010; `spec/PAQUETE.md`, 2.11; la caja vacía, RFC-0007 E4) |
| 7 | Rechazo con causa | una negativa lleva la regla que la produjo | medida (RFC-0007; qué causas se prueban sin el nodo: `spec/PAQUETE.md`, 2.6) |

La fila 6, desde el RFC-0010 (H5b, §556–§577): toda operación que el nodo evalúa por las vías
directas del titular lleva un recibo bajo su cabeza firmada, y el sobre de completitud dice, sin el
nodo, que se resolvió en su ventana —aplicada, o rechazada con prueba— o nombra al operador que no
la resolvió («NO RESUELTA EN LA VENTANA»), o la cuenta aparte cuando su causa no tiene prueba
portable. Su residuo, declarado: un operador que no emite recibo no deja rastro (D-H). El lote y
la prenda lo llevan desde el §611, y el sobre de completitud los resuelve desde el §613 (RFC-0014).
⚠️ §815: el recibo lo deja toda operación que el nodo JUZGA; una evaluación cortada —una caída, un
pánico, un fallo del almacén— deja su número hueco y ningún recibo (`SECURITY.md` §3.15).

**Lo que nada de esto afirma:**

- Que un tercero compruebe sin el nodo la conservación o la autoría de un pago: las pruebas de
  envío, cobro y quema las verifica el nodo, el registro guarda sólo su resumen y ningún método del
  cable las sirve (filas 1 y 5).
- Privacidad frente al operador: el operador lo ve todo.
- Que las unidades del libro existan fuera del libro.
- Que se detecte una operación que el nodo nunca acusó: un operador que no emite recibo no deja
  rastro (RFC-0010, D-H); del lote y la prenda, que llevan recibo desde el §611, tampoco la que un
  agregador no reenvía ni la prenda con `seq` viejo (RFC-0014, D-H). ⚠️ **Corregido en el §794**:
  decía «y el lote y la prenda no lo llevan (D-E)», que el párrafo de la fila 6 ya contradecía
  (`spec/rfc/0010-el-recibo-de-recepcion.md`:291-297).
- Prevención entre libros: dos libros pueden aceptar la misma etiqueta; un tercero con las dos
  cabezas firmadas lo ve después, nunca antes.
- Quién está detrás de una clave, ni que una persona tenga una sola cuenta.
- Del rechazo con causa, que el nodo rechazara, ni cuándo, ni que la regla sea justa: la prueba
  dice que la regla se aplicó sobre lo que una cabeza firmada compromete.

⚠️ **Corregido en el §581**: hasta entonces la fila 6 decía «en parte» y esta lista decía también
«la censura no deja rastro» y «la fila 6 entera: la completitud de los acuses no existe todavía».

⚠️ **Corregido en el §696**: hasta entonces la fila 1 decía que un tercero comprueba «suministro =
saldos + en vuelo; nada se crea ni se pierde entre épocas», «medida, en vuelo y al reabrir»; la
fila 5, «sólo quien tiene la clave mueve su cuenta; el operador no puede», «medida»; el párrafo de
entrada, «una prueba de que el libro hizo lo que sus reglas dicen, que cualquiera comprueba sin el
libro, sin red y sin fiarse del autor», sin excepción; y «Leer antes de nada» contaba la
conservación y la autoría entre lo que comprueba hoy un tercero. Los §379 y §387–§394 son
comprobaciones del nodo, no de un tercero. El título no cambia en ese asiento: si se reformula, o
se construye el camino que haga comprobable por un tercero la conservación, lo decide el autor
(`BACKLOG.md`, la 115).

**Lo que falta, por orden de importancia:** consenso distribuido (sin él, el operador ve los saldos
y puede censurar; la alternativa que este proyecto sí persigue —responsabilidad demostrable, al modo
de Certificate Transparency— tiene el firmante, el guardián del índice, el latido, el verificador
independiente y los testigos, y le falta desplegar el ancla externa —el formato y su verificador
existen desde el RFC-0012, y del medio, propuesto en el RFC-0013, están construidos el árbol, la
nota firmada y su sobre en el kit (§631–§634); faltan su publicador y el despliegue, y con él el
ancla anterior al primer encuentro— y una custodia de
clave **comprobada**, no sólo declarada) · auditoría externa · el recibo de admisión (`AUDITORIA.md`
§121) · delegar la prueba a terceros (verificar la firma en circuito) · una política de caducidad
para las congelaciones. Todo lo demás está enumerado en `AUDITORIA.md` §«16. Donde el autor tiene
MENOS confianza». ⚠️ **Corregido en el §794**: decía «falta el medio», y la tabla de etapas del
RFC-0013 da por construidas la E2 y la E4 y pendiente la E3, el publicador
(`spec/rfc/0013-el-medio-del-ancla.md`:42-47); y remitía a la «sección 4», que lo era cuando se
escribió esta frase y hoy es «FALLO GRAVE SIN RESOLVER: el pagador ve el saldo del receptor».

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
| Buscas la CA de certificados Merkle (MTC) que nació aquí | [`doc/MTC.md`](./doc/MTC.md) · <https://github.com/atoranzo/mtc-core> |
| Buscas los vectores de respuesta conocida de XMSS^MT que nacieron aquí | [`tools/segunda/kat_xmss/`](./tools/segunda/kat_xmss/) · <https://github.com/atoranzo/xmss-kat> |

`AUDITORIA.md` incluye una sección con **los puntos donde el autor tiene menos confianza**. Si vas
a mirar el código con intención de romperlo, empieza ahí.

---

## Estado

| pieza | dónde se mide |
|---|---|
| **22 crates** en un workspace —19 propios y los tres del fork de winterfell 0.13.1 (§533)—; el canon (`tools/canon.sh`) corre en release los tests de todos —diecinueve en `--sello`; `halo2-experiment` y `plonk-experiment` desde `--largo`, y `zk-core` en `--completo`—, y las once herramientas de su bucle de `tools/` que vigilan cifras, citas, dominios, el censo del núcleo, la geometría, que ninguna curva entre en los binarios de producción y que lo prometido del protocolo no se rompa sin que su lista lo diga. ⚠️ **Corregido en el §794**: decía que `--sello` «corre los tests de todos»; la columna `nivel` de su tabla deja tres fuera (las filas `halo2-experiment`, `plonk-experiment` y `zk-core` de la tabla de `tools/canon.sh`, su función `nivel_num` y el salto por nivel de su bloque «1 · los tests, crate a crate»). ⚠️ **Corregido en el §847**: citaba esas tres cosas por línea, `:128-130`, `:154-158` y `:271`, y la tabla las movió. Se citan las filas, la función y el bloque. ⚠️ **Corregido en el §835**: decía que las herramientas vigilan «que ninguna curva entre en los binarios de producción»; la de curvas no ve el TLS de `ureq`, que trae curvas al cli y al SDK para el transporte (la 122 de `BACKLOG.md`). ⚠️ **Corregido en el §838**: el §835 decía que no lo ve; desde el §838 lo ve, y lo admite solo como la excepción del §106 para el transporte del cli y del SDK, `rustls` sobre `ring` bajo `ureq`, con su coste: la clave de vista viaja bajo un ECDHE clásico | la tabla de [`tools/canon.sh`](./tools/canon.sh) lleva los tests que pasan por crate; cada sello la actualiza |
| **Protocolo `zkssl/0.4`**: 32 métodos JSON-RPC (29 `zkssl_*`, 3 `dev_*`), OpenRPC generado desde el código, vectores por versión con la huella de cada uno fijada en `spec/vectors/HUELLAS.sha256` y comprobada en cada canon (§692), que desde entonces no deja reescribir ninguno. ⚠️ **La versión no identifica qué pruebas acepta un nodo** (§699): las pruebas cambiaron sin subirla en el §641 (los diecisiete circuitos de la capa), el §680 (envío, cobro y quema) y el §684 (umbral), porque la conformidad no compara bytes de prueba; medido, un nodo no acepta el envío de un cliente si los dos dicen `zkssl/0.4` y entre ellos queda el §641 o el §680, sea cual sea el más viejo. Vale hasta el corte de `zkssl/0.5`. Y `zkssl-0.2.json` se re-emitió tres veces bajo su versión antes de la huella (§275, §278 y §281). ⚠️ **Corregido en el §699**: decía «vectores por versión que jamás se reescriben». **La promesa**, escrita en el §801: qué superficie es estable, qué la rompe y qué sigue verificando después de una ruptura, en `spec/RPC.md` §«Versionado y compatibilidad» (RFC-0021 E2); `tools/check_superficie.py` compara en cada canon la lista de esa sección con lo que da cada productor (E3a) | [`spec/RPC.md`](./spec/RPC.md) · [`spec/openrpc.json`](./spec/openrpc.json) · [`spec/vectors/`](./spec/vectors/) (543 ficheros: cable, núcleo, nodo, paquete, consumo, conflicto, rechazo, edad, pendiente, pago, prenda, completitud, ancla, ancla cofirmada, rotación, los catálogos 0.3 bajo `0.3/`, los dos de completitud del kit 0.4 bajo `0.4/`, los cuatro `zkssl-0.N.json` y `HUELLAS.sha256`) |
| **RFC**: 0002, 0003, 0004, 0006, 0007 (las pruebas sobre el estado comprometido), 0008 (las pruebas portables del pendiente), 0009 (lo que revela una prueba), 0010 (el recibo de recepción), 0011 (el nodo mentiroso), 0012 (el ancla de cabezas), 0014 (el recibo del lote y de la prenda) y 0016 (un valor, una escritura) aceptados; 0001 (el KDF de reposo: Argon2id con sal y coste), 0005 (el núcleo congelado), 0013 (el medio del ancla), 0015 (el ciclo de vida del firmante), 0017 (la firma acredita la aritmética), 0018 (el tren `zkssl/0.5`), 0019 (la completitud que no se esquiva), 0020 (el esquema dentro de lo que se firma), 0021 (la política de compatibilidad), 0022 (el aviso en ISO 20022) y 0023 (el reloj de la caducidad) propuestos | [`spec/rfc/`](./spec/rfc/) |
| **Verificador independiente** `zk-ssl-verify` 0.4.3, publicada como `arqueo-verify-v0.4.3` (§818), un arreglo de seguridad que cambia veredictos con su aviso (RFC-0021 R6 f; `doc/KIT.md` §0; `GHSA-6p4m-qg2w-8r6j`), reconstruible desde el commit que su `VERSION` nombra; con la misma huella, sólo demostrado dentro de una misma máquina. ⚠️ **§823**: decía «reproducible desde el commit que su `VERSION` nombra», sin la condición de la máquina: rehecha desde su commit en otra, la 0.4.2 sale con 13 de sus 35 secciones distintas, y la 0.4.3 no consta rehecha con su `rustc` 1.97.1 fuera de la máquina que la produjo (`doc/KIT.md` §«Reproducir el kit, para quien no se fíe de la descarga»). ⚠️ **§794**: el `zk-ssl-verify` de este árbol no es esa release, aunque su `Cargo.toml` siga diciendo 0.4.2: desde `a4e888d`, el commit de la release, lo cambian quince commits, del §643 al §782 (`git log a4e888d..8c02941 -- crates/zk-ssl-verify/src`), y cortar la siguiente es del autor. ⚠️ **§815**: cortada por decisión del autor: la 0.4.3 lleva esos commits y el arreglo del §815, y su `Cargo.toml` dice 0.4.3. ⚠️ **§818**: decía «0.4.3, cortada (§815), un arreglo de seguridad que cambia veredictos con su aviso (…); la publicada es `arqueo-verify-v0.4.2`», la release de la que habla la nota del §794; la 0.4.3 la publicó el autor sobre `5fd5212`, el commit del §817, y la 0.4.2 sigue publicada, ya no vigente | [`doc/KIT.md`](./doc/KIT.md) · [`tools/artefacto.sh`](./tools/artefacto.sh) |
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

### Uso de IA generativa

Este proyecto se desarrolla con asistencia de un modelo de IA generativa
(Claude, de Anthropic): el asistente propone y **el autor ejecuta, mide y
acepta**; fuera de ese paso, una sesión de Claude Code aplica, comprueba y
commitea en su rama, su asiento lo dice y dice qué compuertas pasó, y la
aceptación del autor es la entrada del cambio en `main`. El método, su
alcance y dónde vive el registro por cambio: [`GENAI.md`](./GENAI.md).
⚠️ **Corregido en el §794**: decía sólo «el autor ejecuta, mide y acepta, y
sólo entra en `main` lo que el canon deja pasar»; `GENAI.md` §«Fuera del
paso 4» cuenta los asientos que commitea una sesión desde el §628, y en
ellos las compuertas son las que la sesión declara, no siempre el canon
entero.

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
