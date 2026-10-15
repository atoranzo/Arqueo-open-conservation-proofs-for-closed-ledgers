# Erratas de los preprints publicados

**Qué es este fichero.** Los preprints de este directorio están
**depositados con DOI**: son artefactos publicados, y el árbol los
conserva **verbatim**. Cuando se detecta un error o una omisión en uno de
ellos, **no se edita el cuerpo**: se registra aquí.

**Por qué.** El proyecto corrige sus documentos vivos y **deja las
secuencias visibles** en los publicados. Reescribir un artefacto
depositado **falsifica el referente** de las correcciones que ya se
publicaron sobre él (`AUDITORIA.md` §119, §122) — el mismo criterio por el
que `doc/ESCALADO.md` y `doc/CONFIANZA_RESIDUAL.md` se commitearon con
cuerpo intacto y **cabecera-mapa** encima, en vez de arreglados.

**Qué NO es.** No es una revisión. Un lector que descargue el PDF
depositado **no ve este fichero**: lo que aquí consta llega a él en la
siguiente revisión, o no llega. Registrar una errata **no la corrige para
quien ya la leyó**, y decir lo contrario sería el tipo de afirmación que
este proyecto castiga.

---

## Entrada 1 — `ZK-SSL-residual-trust.md` §4.1: omite el poder mayor

**Detectada**: 2026-08-02 · **Backlog**: entrada 54 · `AUDITORIA.md`
§128.2, §135.

La tabla «What the operator can still do» enumera ordenar, censurar,
observar el estado y custodiar el registro. **Omite el mayor de todos:
sustituir el verificador.**

> Ordenar, censurar y observar actúan **dentro** de las reglas. Sustituir
> el verificador **redefine las reglas** bajo las que todas las demás se
> juzgan: un operador que lo cambia puede aceptar como válido lo que las
> reglas publicadas rechazarían.

Y hoy **no deja rastro**: el sistema no tiene noción de «reglas vigentes»
—`OpKind` dice qué circuito usar, no qué versión estaba activa—, así que
el cambio no es comprobable a posteriori.

⚠️ **Es la cuarta ceguera del encuadre de §4.1**, junto a las tres que el
propio documento ya reconoce —la fuga de confidencialidad, los límites de
capacidad y el privilegio que no expira—. Un documento cuya contribución
es **nombrar la confianza residual** queda falsado por la que no supo
nombrar; el propio §24 lo dice.

**Cierre diseñado**: `hash_verificador_vigente` en la cabeza de época
atestiguada, que vuelve pública toda actualización
(`doc/CONFIANZA_RESIDUAL.md` §2.2). **Depende de dar antes al sistema esa
noción de reglas vigentes**, que no existe.

**Dónde vive el contenido hoy**: `SECURITY.md` §2, en la lista de lo que
el sistema no protege.

**Corregida en la cuarta revisión** (2026-08-12, versión depositada:
[10.5281/zenodo.21905595](https://doi.org/10.5281/zenodo.21905595)): la
tabla de §4.1 incorpora la fila «Replace the verifier itself» — el poder,
su falta de rastro hoy, y el cierre diseñado
(`hash_verificador_vigente` en la cabeza atestiguada) con su condición.

---

## Entrada 2 — Cuatro notas de corrección se insertaron EN el cuerpo

**Detectada**: 2026-08-02 · `AUDITORIA.md` §100, §135.

La cuarta revisión añadió **cuatro notas de corrección dentro del cuerpo**
de `ZK-SSL-preprint.md` y `ZK-SSL-policy-note.md` —sobre la clave de
gasto, el cobro, la corrección incompleta de §8.2 y la unidad MiB—.

**Su contenido es correcto y sigue vigente.** Lo que no lo es: **usaron
el vehículo que este fichero declara equivocado**, y lo hicieron antes de
que este fichero existiera.

⚠️ **Se registra en vez de deshacerse**, por tres razones:

1. Esas notas **ya están depositadas** en la cuarta revisión: retirarlas
   ahora dejaría el PDF publicado sin ellas y el árbol tampoco.
2. Retirarlas sería **reescribir el artefacto otra vez** — el mismo error,
   en la dirección contraria.
3. **Un fichero de erratas que declara una regla sin admitir que el
   directorio la incumple en cuatro sitios nacería falso.**

**A partir de aquí**: toda corrección a un preprint publicado entra por
este fichero. Las cuatro notas quedan como **la excepción documentada**,
no como el patrón.

---

## Entrada 3 — `ZK-SSL-residual-trust.md` §4.1: la condición del cierre

**Detectada**: 2026-08-19 · **Backlog**: entradas 54 y 83 · `AUDITORIA.md`
§246, §321.

La **cuarta revisión** (2026-08-12,
[10.5281/zenodo.21905595](https://doi.org/10.5281/zenodo.21905595)) incorporó
a la tabla de §4.1 la fila «Replace the verifier itself» **con su
condición**, tal como la registró la entrada 1 de este fichero: el cierre
—`hash_verificador_vigente` en la cabeza atestiguada— **dependería de dar
antes al sistema una noción de «reglas vigentes»**.

**Esa condición es incorrecta, y el árbol la había corregido en el §246**
—antes de la revisión que la publicó—. Es **casi circular**:
`hash_verificador_vigente` **es** el mecanismo para tener esa noción, así
que la condición se pide a sí misma.

> **La razón real: el AIR es código, no datos.** Lo que el campo debe delatar
> es qué es una transición válida, y eso lo define el AIR. Lo único
> hasheable en ejecución son las `ProofOptions`, y **un operador puede cambiar
> el AIR dejándolas idénticas**. Un `verifier_hash` así no sería un campo
> vacío: sería un campo **ciego** — y un campo ciego pasa desapercibido
> **mintiendo justo sobre lo que existe para detectar**.

⚠️ **Y las dos salidas están cerradas por razones ajenas al protocolo**:

- **Hashear el fuente al compilar** no prueba que el binario se construyera de
  ese fuente. Sin **compilación reproducible**, el operador reporta el hash
  grabado y corre otra cosa: miente en el caso que importa.
- **El AIR como datos** —entrada 55— sí sería hasheable, y está parada
  por un motivo que no es esfuerzo: *una especificación escrita por quien
  escribió el circuito hereda sus puntos ciegos*, y debe escribirse **con la
  auditoría, no antes**.

**Consecuencia para el lector del preprint**: el cierre diseñado sigue siendo
el correcto, pero **no es una tarea pendiente de implementación**: es una
decisión de arquitectura con dos precondiciones que viven fuera del
protocolo. La condición publicada subestima el coste.

**La entrada 1 no se edita** — este fichero crece por adición: queda como fue
escrita, y esta entrada la corrige encima. El texto vivo del árbol queda
corregido en `BACKLOG.md` (entrada 83; la 54 ya lo decía), `SECURITY.md` §2,
la cabecera-mapa de `doc/CONFIANZA_RESIDUAL.md` y
`crates/zk-ssl/src/log.rs`, dentro del propio `EpochHead`.

---

## Entrada 4 — Los tres preprints: la URL del repositorio que citan ya no es la del repositorio

**Detectada**: 2026-09-04 · **Corregida en el árbol**: 2026-09-05 (`AUDITORIA.md`
§403).

Los tres preprints depositados dan como repositorio
`https://github.com/atoranzo/ZK-SSL-ZK-Sovereign-Settlement-Layer-` (en la
cabecera de cada uno y en su lista de artefactos o referencias). El
repositorio se renombró a
`https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers`,
y la URL vieja sólo resuelve mientras GitHub la redirija: **caduca por una
decisión ajena** el día que alguien cree un repositorio con ese nombre.

**Qué se hizo en el árbol, y qué no.** Las seis líneas se mudaron en los
tres ficheros de este directorio, como el 27-08-2026 se mudaron sus DOI
(`README.md` de este directorio, «sólo se tocaron identificadores»): la URL es
un identificador de la misma clase, y dos clases de trato para una misma
clase de objeto sería lo que este proyecto castiga. **No se tocó ni una
cifra, ni una afirmación, ni un párrafo.** Desde ese commit los tres ficheros
difieren de los PDF depositados en las líneas de DOI y en las de la URL, y
nada más; nace además una puerta (`tools/check_publicadas.py`, ATADO C) que
pone el canon en rojo si la URL vieja vuelve a un documento vivo.

**Para el lector del PDF depositado**: el enlace funciona hoy por
redirección, y esta entrada no se lo arregla. La URL vigente es la de arriba;
el siguiente depósito la llevará dentro.

---

## Entrada 5 — Los tres preprints: «la mitad cara es del receptor» no la sostiene lo medido

**Detectada**: 2026-08-14 · **Backlog**: entrada 96 (cerrada en el §860, fundida
en la 28) · `AUDITORIA.md` §304, §512, §538 ·
`spec/rfc/0009-lo-que-revela-una-prueba.md` D-AJ.

Los tres depositados dicen que generar la prueba de cobro cuesta «roughly 500
ms against 283 ms for a send» (`ZK-SSL-policy-note.md:235`,
`ZK-SSL-residual-trust.md:182`; `ZK-SSL-preprint.md:329`, con «to generate»
detrás de «500 ms»). Dos de ellos dan la causa: «because the claim circuit
walks both the pending tree and the accounts tree». Y sacan de ahí una
conclusión: el coste de un pago cae sobre quien recibe. En la nota de
política es un consejo («a fact worth knowing before pricing anything»), y en
la de confianza residual un argumento normativo («the party with the least
choice about participating carries the larger cost»). Los 500 frente a 283
equivalen a una razón cobro/envío de 1,77.

**Lo medido después de depositarlos, en orden**:

- **2026-08-14** (§304): al revés. Cuatro corridas: envío 260,7-286,8 ms,
  cobro 159,4-189,6 ms, y las bandas no se solapan. El cobro costaba menos que
  el envío. Se midió por la vía de la capa, que junta el trabajo del cliente
  con el de la capa, y por eso el §362 llevó la medida a la vía del cliente.
  Por esa vía el sentido siguió siendo el mismo hasta el §511: medido apareado
  en `3db491f`, envío 240,8 ms y cobro 160,2.
- **2026-09-19** (§512): el arreglo B (5.A-272) vuelve a invertir el orden.
  Medido apareado: la razón cobro/envío pasa de 0,67 a 1,75, cerca del 1,77
  que equivale a lo publicado. El sentido vuelve a ser el publicado, pero no
  por la causa publicada: el §512 lo atribuye al acumulador del arreglo B, y
  el porqué de esa diferencia no está explicado.
- **2026-09-24** (§538, D-AJ): con el testigo oculto, cada lado cuesta 700-750
  ms y los dos miden lo mismo dentro del ruido. De tres corridas, dos dieron
  receptor y una pagador (razón envío/cobro de 0,975 y de 1,064). El canon
  dejó de asertar el sentido.
- **2026-09-28 y 2026-09-30** (`doc/ecst/ECST.md` §7.2): otra máquina, una
  microVM Firecracker de 4 vCPU, con `simulate --amount 250000` de la CLI y
  una corrida por día —la vía de la capa, la que el §362 descartó como objeto
  de esta relación—: envío 970 ms y cobro 1.198 ms el primer día; envío 1.333
  ms y cobro 879 ms el segundo.

**Lo que no está medido**: la relación sobre el árbol de hoy. Desde el §538
cambiaron los circuitos de la vía del pago (§641, §680) y el probador (§652),
y el canon imprime los tiempos sin asertarlos ni guardarlos. El sello que
escribe esta entrada no los midió: **NO MEDIDO**.

**Consecuencia para el lector del preprint**: lo publicado —el cobro, un 77 %
más caro que el envío— quedó contradicho en el §538, donde los dos lados
midieron lo mismo dentro del ruido, y hoy no está medido qué lado del pago es
el caro. Y la causa publicada ya era errónea al depositarse: en `6384907`
(2026-07-30, el commit que trajo al árbol el texto publicado) y en `69fa2d4`
(2026-08-01), `circuit_send.rs` y `circuit_claim.rs` tienen la misma geometría
—`TRACE_LENGTH` 1024, la raíz de cuentas en la fila 271 y la de pendientes en
la 743—, y el envío lleva un segmento más (5 frente a 4) y una columna más (49
frente a 48 en el primero, 52 frente a 51 en el segundo). En `3db491f`, el
padre del arreglo B, el envío y los dos circuitos de cobro suben por los
mismos ciclos de los dos árboles (`CYC_ACC..CYC_PK` y
`CYC_PEND_CLIMB..CYC_FIN`), y por la vía del cliente el cobro costaba 160,2 ms
y el envío 240,8 (§512). Lo leído en el código es MEDIDO; que «recorrer los
dos árboles» no distinga el coste de los dos lados es DEDUCIDO. Los tiempos
dependen de la máquina, y este proyecto no publica tiempos absolutos como
propiedad (§304). Quien vaya a fijar precios no debe basarse en esa frase.

**Y los bytes del mismo párrafo** —«Each proof is about 62 KB and that has not
changed» y los «120.4 MiB» por mil pagos, «MB» en la nota de política— también
se movieron: desde el §538 una prueba de envío pesa 73.571-84.244 B y una de
cobro 72.382-83.723 B, y mil pagos, 139,2-160,2 MiB (la banda D-AC del
RFC-0009, en `crates/zk-ssl/src/metrics.rs`). No son el objeto de esta
entrada; la siguiente revisión los recoge con ella.

**Corrección**: pendiente de la siguiente revisión (entrada 28 del
`BACKLOG.md`). La frase o se retira o se vuelve a medir sobre el árbol de esa
revisión, y no se copia de aquí. La condición para volver a afirmar un sentido
es la de D-AJ: que se mida estable. No llega al lector del PDF depositado
(«Qué NO es», arriba). El §512 la daba por pendiente y la llamaba «Entrada 3»,
que ya era otra; es ésta.

---

*Este fichero crece por adición. Las entradas no se editan ni se borran:
si una errata se corrige en una revisión posterior, se anota debajo con su
fecha y su versión.*
