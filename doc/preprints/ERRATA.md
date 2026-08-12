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

*Este fichero crece por adición. Las entradas no se editan ni se borran:
si una errata se corrige en una revisión posterior, se anota debajo con su
fecha y su versión.*
