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

*Este fichero crece por adición. Las entradas no se editan ni se borran:
si una errata se corrige en una revisión posterior, se anota debajo con su
fecha y su versión.*
