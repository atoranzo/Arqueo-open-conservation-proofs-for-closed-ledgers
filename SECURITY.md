# Política de seguridad — ZK-SSL

## Léelo antes que el resto

Este texto describe **intenciones de diseño y limitaciones medidas**, no
garantías auditadas:

- **Nada de este proyecto ha sido auditado por terceros.** Ninguna cantidad de
  pruebas propias sustituye una auditoría externa. No la hay.
- Las propiedades de seguridad listadas son **objetivos del diseño**, validados
  —cuando lo están— por los tests del propio autor.
- Los problemas de §3 son **hallazgos internos**. La mayoría están medidos y
  con referencia a `AUDITORIA.md`; **re-verifícalos contra el código actual**
  antes de confiar en ellos o darlos por resueltos.
- La fuente autorizada del comportamiento real es el código. **Si este
  documento y el código discrepan, el código gana y este documento está mal.**

⚠️ Ese último punto no es retórica. Este proyecto ha registrado **cuatro veces**
una garantía enunciada en prosa cuya condición nadie verificó
(`AUDITORIA.md` §95.2, §98.4). Si algo aquí suena más seguro de lo que puedes
verificar tú leyendo el repositorio, trátalo como afirmación pendiente.

---

## Estado del proyecto

- **Prototipo de investigación**, no un producto. No maneja dinero real y no
  debe manejarlo en su estado actual.
- **Nodo único operado por una sola parte.** Pese a la palabra «Sovereign»,
  **no es descentralizado**: el operador ve el estado, ordena las operaciones y
  puede censurar. Es una característica declarada, no un descuido.
- **Su tesis no es la evasión**: privacidad frente a terceros con supervisión
  demostrable —revelación selectiva del titular, límites de emisión en
  circuito—.
- **Su aportación medible** es la comparación empírica de cinco sistemas de
  prueba sobre el mismo circuito.

---

## 1. Qué pretende garantizar el diseño

**Propiedades objetivo.** Su corrección depende de que las restricciones de
circuito sean completas, y eso **no está formalmente especificado ni auditado**
(§3.1).

- **No creación de dinero**: una transición válida conserva el valor total.
- **Autoridad de gasto**: solo quien controla la clave puede gastar.
- **No reescritura silenciosa del historial**: el registro encadenado impide
  reescrituras **detectables por quien haya observado una cabeza anterior**.
  ⚠️ Garantía condicional, y **hoy nadie fuera del operador observa cabezas**
  (§2).
- **Privacidad del contenido frente a terceros**: ⚠️ **medido que no se
  cumple**, ver §3.2.

Ninguna debe citarse fuera de este repositorio como «garantizada». La
formulación honesta es: *«el diseño pretende X; no está auditado»*.

⚠️ **Y dos de estas propiedades ya fallaron en implementación** mientras el
diseño las garantizaba: hasta el 31-07-2026 la capa no verificaba las pruebas
de la vía de pago (`AUDITORIA.md` §73) y el compromiso del pendiente no estaba
atado al importe (§74). Ambas corregidas y medidas.

## 2. Qué NO protege

- **El operador ve el estado.** Es el mayor límite de privacidad y está
  asumido.
- **El operador puede censurar y ordenar.**
- **La custodia del registro está en manos del operador.** Sin observadores
  externos de sus cabezas, podría presentar historias distintas a partes
  distintas. Cerrarlo exige publicar cabezas a testigos: **es una propuesta,
  no una función existente**.
- **No hay recuperación si el nodo desaparece.**
- **Metadatos**: qué posiciones cambian y cuándo siguen siendo observables.
- **Solidez de circuitos y del sistema de prueba**: no verificada formalmente.
- ⚠️ **Rotar la clave exige dos custodios.** Un titular puede gastar sin
  permiso de nadie y **no puede mejorar su propia seguridad sin permiso de
  dos** (§98.4).

## 3. Problemas de seguridad identificados

> Hallazgos internos, **no auditados**. Cuatro de los cinco están **medidos**
> con fecha y referencia; se listan porque ocultar debilidades conocidas sería
> lo contrario de la imagen fiel.

### 3.1 Ausencia de especificación formal del AIR — **prioridad más alta**

Un circuito sin restricciones completas puede admitir un testigo fraudulento
que pase la verificación. **Un fallo de solidez es dinero falso invisible.**

Mientras no exista una especificación formal de cada AIR —qué se restringe, de
qué grado, y qué explícitamente **no** se restringe— con tests de solidez
negativos, la propiedad de «no creación de dinero» descansa sobre una
suposición no comprobada.

⚠️ **Y no es hipotético en este proyecto**: §72 registra una restricción
escrita **sobre el carril equivocado** —bien formada, de grado correcto, y
atando lo que no era—. Ninguna herramienta la detectó.

**Estado: abierto.** Backlog 48 (B12.1).

### 3.2 El compromiso de hoja **no es ocultante** — ⚠️ MEDIDO

`native_leaf(identity, balance, nonce)` = `H(H(id, saldo), nonce)`, **sin
salt**. Y `path_for` entrega al cliente el hermano de nivel 0, que **es la hoja
de la cuenta vecina**.

**Medido el 31-07-2026: el saldo del vecino se recupera en 10,84 s.**

El coste es una **curva** sobre el rango de saldo que el atacante asuma:

| rango asumido | coste, un núcleo |
|---|---|
| 0–10.000 € | **2,4 min** |
| 0–1 M € | 4,1 h |
| 64 bits uniformes | 8,3 × 10⁷ años-núcleo — **que nunca lo son en dinero** |

Alcance: **una cuenta** por camino. Y los índices son secuenciales, así que
**el vecino se elige, no toca al azar**.

⚠️ **Sin solución conocida.** Derivar el salt de la clave está descartado:
`open_account`, `mint`, `freeze` y `recover` escriben la hoja de un titular
**sin conocer su secreto** (§99.3).

**Estado: abierto.** Backlog 50. `AUDITORIA.md` §93, §99.

### 3.3 El contrato de lectura de cuentas no exige autorización — ⚠️ MEDIDO

`account_view(index)`, `balance_of`, `nonce_of` y `public_id_of` **toman un
índice y no piden credencial**. Los índices son secuenciales (`next_index +=
1`), así que enumerarlos no requiere adivinar nada.

⚠️ **Es un hallazgo de contrato, no de explotación**: no hay capa de red en
este repositorio. Lo cierto hoy: **exponer estas funciones sin control de
acceso convertiría una fuga hacia-el-operador —asumida— en una fuga
hacia-terceros —no asumida—**.

**No confiar en que «hoy no hay red» como sustituto de un control de acceso.**

**Estado: abierto.** Backlog 49. `AUDITORIA.md` §93.1.

### 3.4 Espacio de los identificadores anti-doble-gasto — **no aplica**

La vía de producción es la de **dos fases**, y **no usa nulificadores**: un
envío cambia el saldo del pagador, luego su hoja, luego la raíz, de modo que
un reenvío presenta una raíz obsoleta y se rechaza.

⚠️ **El problema existió y se retiró con su camino.** La vía de un paso
derivaba la posición del marcador del propio marcador, con colisión por
cumpleaños alrededor de **sesenta y cinco mil pagos** frente a los cuatro mil
millones que el árbol anunciaba.

⚠️ **Y fue evitado, no resuelto**: lo que sustituye al marcador es el
encadenamiento de raíces, que **exige un orden total** — el que un nodo único
da y un sistema distribuido no. Quien distribuya esto recupera el límite
intacto.

**Estado: cerrado en el nodo único, abierto para cualquier distribución.**

### 3.5 Definiciones duplicadas de la hoja — ⚠️ MEDIDO

Existen **tres** `native_leaf` en el árbol: en `circuit_settlement`,
`compliance_circuit` y `double_entry`.

**Misma estructura, distinta anchura de dominio**: la de producción toma la
identidad como `Digest` —256 bits—; las otras dos como `BaseElement` —64—.

⚠️ **Compartir nombre lo empeora**: invita a suponer que son la misma función.
Quien verifique una y dé por buenas las otras dos se equivoca, y nada en el
código se lo advierte.

**Estado: abierto.** Backlog 51. `AUDITORIA.md` §94.

## 4. Alcance

**En alcance**: solidez de circuitos y de la función de transición;
autorización de gasto; corrección del verificador; propiedades de privacidad
declaradas; integridad del registro encadenado.

**Fuera de alcance hoy**, por diseño o inmadurez: consenso distribuido;
resistencia a un operador malicioso sin observadores externos; recuperación
ante pérdida del nodo; análisis de metadatos; seguridad económica de un token
—el proyecto **no requiere ninguno**—.

## 5. Cómo reportar una vulnerabilidad

**Fallos de solidez** —crear saldo, gastar sin autorización, o hacer que el
verificador acepte una transición inválida— repórtalos **en privado** antes de
divulgarlos.

> ⚠️ **PENDIENTE: canal de contacto privado.** Hasta que se active, este
> documento **no ofrece una vía privada**, y eso es una carencia con
> consecuencia: quien encuentre un fallo de solidez no tiene dónde reportarlo
> salvo un *issue* público. Se declara en vez de omitirse.
>
> Vía prevista: *GitHub Security Advisories* del repositorio.

**Problemas no sensibles** —documentación, límites ya listados, mejoras—:
*issue* normal.

**Qué incluir**: descripción del fallo, la lógica que lo produce y, si es
posible, un caso mínimo que lo demuestre —por ejemplo, una traza inválida que
el verificador acepta—.

**Qué esperar**: proyecto de investigación sin equipo dedicado. No hay
compromisos de respuesta ni recompensas. Se agradece la divulgación
responsable y se da crédito a quien lo desee.

**No** se harán afirmaciones categóricas sobre la seguridad del sistema en
respuesta a un reporte: se corregirá, o se documentará el límite.

---

*Refleja el estado conocido en el momento de escribirlo, sin auditoría
externa. Cuando este documento y el código dejen de coincidir, corrige el
documento.*
