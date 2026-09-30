# Transiciones de estado con evidencia incorporada (ECST)

*Una disciplina de composición y dos instanciaciones: HBS-STATE y Arqueo*

**Informe técnico — BORRADOR v0.1, no depositado**

- **Autor:** Ángel José Toranzo Portela
- **Fecha:** 2026-09-30
- **Base:** Arqueo, `main` en 71c5aad (S582); `hbs-state` en a960828 (crate 0.2.0, que implementa la
  especificación HBS-STATE v0.3 revisada el 2026-09-27).
- **Registro de verificación:** `VERIFICACION.md`, en este mismo directorio: cada afirmación de los
  borradores previos con su veredicto, su fuente y su redacción correcta (§1.5).

> **Cómo leer las marcas.** Toda cifra, todo resultado de ejecución y todo estado que un repositorio
> declara de sí mismo llevan una de estas marcas:
>
> - **[MEDIDO]** — lo midió el autor y está registrado. Se da la fuente (`AUDITORIA.md` §N, o el README o
>   la especificación de `hbs-state`) y, cuando consta, la máquina y la dispersión.
> - **[REPRODUCIDO]** — se re-ejecutó el 2026-09-28 o el 2026-09-30 en una microVM Firecracker, no en la
>   máquina del autor (`doc/ecst/borrador/REPRODUCCION.md`). Es una ejecución en un entorno: no
>   sustituye a lo medido.
> - **[DERIVADO]** — calculado a partir del código o de cifras medidas; nadie lo ha medido.
> - **[DECLARADO]** — lo que un repositorio declara de sí mismo (un umbral, una política, un estado).
> - **[PROPUESTA]** — parte del modelo que no está implementada.
> - **[NO MEDIDO]** — nadie lo ha medido; no se da cifra.
> - Y dos más, por precisión: **[LECTURA]** — deducido leyendo el código, sin ejecutarlo; **[CITADO]** —
>   cifra de una fuente externa, no medida en ninguno de los dos repositorios.
>
> Las descripciones del código sin marca son **[LECTURA]**: remiten a una ruta `fichero:línea` o a una
> ficha del registro, las cotejaron con el árbol los agentes verificadores y nadie las ha ejercido salvo
> que se diga. «VERIFICACION.md, L3a» remite a la ficha de la afirmación L3a en el registro. Las rutas
> `fichero:línea` son del árbol en 71c5aad.

---

## Resumen

ECST (*Evidence-Carrying State Transitions*) es una disciplina para componer primitivas conocidas
—pruebas por transición, encadenado por hash, cabezas firmadas y reconciliación del estado físico— de modo
que cada transición de un sistema con estado deje un paquete comprobable por partes: el estado comprometido
anterior y el nuevo, la operación, la evidencia criptográfica, la autorización y un resumen acumulado de la
historia. No es una primitiva nueva, ni computación verificable incremental (IVC), ni consenso, ni una
garantía de durabilidad física. Su tesis es de separación: la integridad de la historia, la validez
criptográfica, la autorización, la conformidad semántica y la unicidad de la historia (consenso) son
propiedades distintas y, en general, ninguna implica a las otras. Se enuncia el atado retroactivo —la
propiedad clásica del sellado temporal por hash— con sus hipótesis exactas, separadas de lo que un tercero
necesita para aprovecharlo; una función R(C,K) que clasifica en cuatro estados el par contador–índice de un
recurso de consumo monótono, con una regla para los índices de uso indeterminado; y un protocolo de
auditoría por niveles, propuesto y sin implementar, cuyo resultado es un vector con un componente por
predicado. Se analizan dos instanciaciones del mismo autor: HBS-STATE v0.3, una especificación con
implementación de referencia para decidir tras un reinicio si el índice de una firma basada en hash con
estado es seguro, que sólo instancia la reconciliación; y Arqueo, un libro cerrado de un solo operador con
una prueba STARK por transición, que instancia la mayoría de los componentes con huecos declarados —no
conserva las pruebas, ningún tercero recompone la cadena, no hay herramienta de auditoría— y usa la misma
R(C,K) para dos recursos. Cada cifra lleva su marca y su fuente, y las cifras sin medición de los borradores
previos se retiran. Se informa además de un hallazgo de lectura, no reproducido: con el fichero del
contador de firma borrado o puesto a cero, el nodo volvería a firmar hojas XMSS ya usadas; está corregido,
con tests y el canon en verde, en el §594 de `AUDITORIA.md` (commit 78d71a4 de la rama de trabajo) (§8.1).

---

## 1. Introducción

### 1.1 El problema

**El índice de una firma con estado.** En XMSS y LMS [RFC8391; RFC8554], construidos sobre árboles de
Merkle [Merkle89], cada firma consume el índice de una clave de un solo uso; firmar dos mensajes con el
mismo índice permite falsificar con un coste factible [BH17; Fluhrer23], y la clave debe darse por perdida
(VERIFICACION.md, B17d). El estado del que depende la seguridad vive fuera de la criptografía: un contador
en un fichero, un proceso que muere entre reservar y firmar, un disco que confirma un `fsync` sin
persistir, una copia de seguridad que se restaura. NIST SP 800-208 exige que la generación de claves y de
firmas se valide sólo dentro de módulos criptográficos hardware [SP800-208]. RFC 10033, informativo y
publicado en septiembre de 2026 por el grupo PQUIP, recomienda en su §4 que la gestión del estado cumpla las
propiedades ACID («should») y hardware dedicado para lograr contadores resistentes al retroceso, y analiza
en su §5 nueve enfoques, sin vectores de prueba [RFC10033].

**El libro cerrado auditado por terceros.** Un único operador lleva un libro, y los titulares, un auditor o
un supervisor quieren comprobar qué ocurrió sin fiarse de él y, si es posible, con él apagado. La pregunta
útil no es si el libro es «verificable», sino qué puede comprobar quién, con qué datos y bajo qué
hipótesis.

La forma común: transiciones sobre un estado, recursos que se consumen una sola vez, evidencia que tiene
que sobrevivir al proceso que la produjo y la necesidad de que «el proceso murió» sea un estado explícito
y clasificable.

### 1.2 Qué es ECST y qué no es

ECST es una **disciplina de composición**. Pide que cada transición produzca un paquete con seis campos,
que cuatro predicados —reglas semánticas, validez criptográfica, autorización y enlace histórico— se
comprueben y se informen por separado, que la reconciliación del estado físico tras una caída sea un
componente con estados nombrados y que una auditoría devuelva un vector y no un booleano.

ECST **no es** una primitiva criptográfica nueva: todas sus piezas están en la literatura (§2). **No es** IVC
ni PCD [Valiant08; ChiesaTromer10]: no demuestra dentro de la prueba Π_i que Π_{i−1} fuera válida, sino que el
resumen acumulado H_i compromete el resumen de cada prueba (VERIFICACION.md, P17). **No es** consenso: no dice
nada de la unicidad de la historia si nadie compara lo que ha visto. **No es** una garantía de durabilidad
física: el nivel al que llega es el de `fsync`, y la resistencia a un corte de corriente no está medida en
ninguno de los dos repositorios.

### 1.3 Contribuciones

Ninguna es una pieza nueva:

1. **Una formulación explícita** del paquete de transición E_i, de cuatro predicados separados y de una
   proposición de atado retroactivo cuyas hipótesis —codificación inyectiva y de versión fija, resistencia
   a colisiones de dos funciones de salida de anchura fija, atadura de los bytes reales de la evidencia
   (R4)— se distinguen de las condiciones de uso por un tercero —cabeza autenticada y custodiada antes de
   la alteración, génesis y longitud conocidas, datos disponibles, versión de la codificación—. Para dos de
   esas hipótesis se dan contraejemplos de la historia de Arqueo: la resistencia a colisiones de
   $\mathrm{Hp}$ como función de los bytes de la prueba, que falló por su relleno interno aunque su
   permutación no se rompiera, y R4.
2. **La reconciliación del estado físico, R(C,K), como componente del modelo**, con una regla para los
   índices de uso indeterminado —darlos por perdidos, como la reserva por intervalos de [McGrew16]
   (RFC 10033, §5.9), sin perder la vivacidad que exige [Ariadne16]—, y la observación, que sí es de este
   trabajo, de que el significado de cada estado depende del modelo de clave.
3. **Un análisis de dos instanciaciones** con rutas de código: qué partes del modelo cubre cada una, qué
   está medido, qué se reprodujo en esta revisión y qué falta. Incluye la segunda instancia de R(C,K)
   dentro de Arqueo, el registro de recepción: un segundo uso de la misma clasificación, del mismo autor,
   que indica que se generaliza como clasificación pero no es evidencia independiente.
4. **Un registro público de verificación** de los borradores generados con IA que precedieron a este
   informe (`VERIFICACION.md`), con las cifras que se retiran (§7.5), y un hallazgo de lectura sobre el
   guardián del índice de Arqueo (§8.1).

### 1.4 Lo que este informe no afirma

- Que ECST sea más barato que IVC: no hay ninguna comparación medida (§7.4).
- Una demostración formal: la Proposición 1 lleva un esbozo de prueba, no una prueba mecanizada.
- Que HBS-STATE sea un estándar: es un borrador derivado de una sola implementación (§4.10).
- Que Arqueo esté en producción, maneje dinero real o haya sido auditado por terceros: no ocurre ninguna de
  las tres cosas (§5.1).
- Que un tercero pueda hoy auditar la historia completa de un libro de Arqueo: no puede (§5.7).
- Durabilidad frente a un corte de corriente, en ninguno de los dos sistemas.

### 1.5 Procedencia

Este informe parte de borradores generados con IA —sobre ECST, HBS-STATE, un Internet-Draft y un correo al
CFRG— que mezclaban ideas razonables con cifras, hechos de código y citas inventados. Sus 305 afirmaciones
se verificaron una a una: 104 CONFIRMADAS, 105 PARCIALES, 68 FALSAS, 23 SIN FUENTE y 5 NO VERIFICABLES
(`VERIFICACION.md`, «El recuento»). Aquí sólo se recoge lo que resistió, en su redacción corregida.

---

## 2. Trabajo relacionado

**Sellado temporal y registros a prueba de manipulación.** Encadenar cada registro con el resumen del
anterior para que alterar el pasado cambie todo lo posterior es la idea de Haber y Stornetta [HS91].
Schneier y Kelsey la aplicaron a registros de auditoría en máquinas no fiables [SK99], y Crosby y Wallach
dieron estructuras de árbol de historia con pruebas eficientes de pertenencia y de extensión [CW09].
Dowling et al. definieron cuatro propiedades de seguridad de los esquemas de registro como CT y
demostraron que CT las cumple [DGHS16]: es el marco formal más cercano a la Proposición 1. El atado
retroactivo de ECST (§3.5) es exactamente la propiedad de [HS91] y no se presenta como aportación.

**Transparencia y responsabilidad.** Certificate Transparency publica un registro de solo añadido con
pruebas de inclusión y de consistencia [RFC6962; RFC9162; Laurie14]: no impide la mala conducta, la hace
difícil de ocultar si alguien compara. Detectar una vista dividida comparando lo que cada cliente ha visto
es la consistencia por bifurcación (*fork consistency*) que introdujeron Mazières y Shasha [MS02], y que
SUNDR aplicó a un repositorio de datos no fiable [SUNDR04]; el testigo de §5.6 es una instancia de ese
patrón con un solo observador. PeerReview da responsabilidad en sistemas distribuidos con registros a
prueba de manipulación y testigos [PeerReview07], y CONIKS lleva la transparencia a las claves de usuario
con monitorización por los propios clientes [CONIKS15]. Arqueo adopta el patrón de CT y los algoritmos del
árbol de historia de RFC 6962 sobre primitivas propias; no implementa el protocolo CT (VERIFICACION.md,
L10a).

**Libros y máquinas de estado verificables.** LedgerDB es una base de datos de libro centralizada con
evidencia de manipulación y auditoría por terceros [LedgerDB20]; zkLedger permite auditar un libro sin
revelar al auditor el contenido de las transacciones [zkLedger18]; en las máquinas de estado replicadas
sin ejecución replicada [Piperine20] y en las máquinas de estado verificables [VSM20], un probador no
fiable acompaña cada cambio de estado de una prueba sucinta de que la transición es correcta; y Al-Bassam
et al. combinan pruebas de fraude y de disponibilidad de datos para que un cliente ligero detecte bloques
inválidos [AlBassam21]. ECST comparte ese esqueleto; lo que añade es la tabla de lo que cada predicado no
detecta y la reconciliación del estado físico.

**Código portador de pruebas, IVC y PCD.** El término «evidence-carrying» remite al código portador de
pruebas de Necula [Necula97], donde el consumidor comprueba una prueba que acompaña al código. La
computación verificable incremental [Valiant08], los datos portadores de pruebas [ChiesaTromer10], su
composición recursiva [BCCT13] y los esquemas de plegado [Nova22; HyperNova24] resuelven un problema más
fuerte que el de ECST: que una sola prueba acredite la validez de toda la cadena. También hay acumulación
sin supuestos de clave pública: basada en hash, en el modelo de oráculo aleatorio y, en su primera
versión, con un número acotado de pasos [BMNW25]. ECST renuncia a todo eso a cambio de no meter un
verificador dentro del circuito; el precio es que auditar n transiciones exige n verificaciones y la
disponibilidad de n pruebas (§3.10). Las pruebas de Arqueo son STARK [BBHR19] (versión completa en
[STARK18]), en la implementación de winterfell.

**Continuidad de estado y protección frente al retroceso.** Memoir [Memoir11], Ariadne [Ariadne16] y ROTE
[ROTE17] estudian cómo un módulo protegido evita que se le restaure un estado anterior, que es la
dificultad que RFC 10033 atribuye a los contadores resistentes al retroceso implementados sólo en
software. Ariadne exige además vivacidad: una pérdida inesperada de corriente no debe dejar nunca el
sistema en un estado del que no pueda reanudar, que es el problema de §3.8 para un recurso de un solo uso.
Nimble ofrece como servicio en la nube, a aplicaciones en entornos de ejecución confiables, la detección de
retrocesos [Nimble23]. ECST no resuelve el retroceso: la reconciliación de §3.7 clasifica lo que ve, y lo
que no ve —una restauración conjunta de contador y clave, por ejemplo— queda fuera y se declara.

**Gestión del estado en firmas basadas en hash.** McGrew et al. analizan los modos de fallo del estado y
proponen reservar intervalos de índices por adelantado [McGrew16]; `hbs-state` persiste antes de cada
firma, que es el caso degenerado de esa estrategia (intervalo 1): paga una escritura sincronizada por
firma y a cambio pierde como mucho un índice por caída. ETSI TR 103 692 analiza los retos y riesgos de la
gestión del estado [ETSI21]. RFC 10033 recoge nueve enfoques y recomienda hardware dedicado [RFC10033], y
observa en su §4 que un verificador con acceso a todas las firmas detecta un índice repetido, que es lo
que puede hacer un testigo (§5.6); SP 800-208 exige módulos hardware para la validación [SP800-208]. La
alternativa sin estado, SLH-DSA [FIPS205], elimina el problema, y RFC 10033 (§1.1) la recomienda para la
mayoría de las aplicaciones. EIP-8310, un borrador de Ethereum, exige una marca de agua alta persistida
antes de entregar la firma y la detección de su regresión tras restaurar una copia [EIP8310]. El coste de
reutilizar un índice lo analizaron Groot Bruinderink y Hülsing [BH17]; Fluhrer corrigió su análisis de
Winternitz, que suponía independientes probabilidades que no lo son, y mantuvo la conclusión de que el
reúso permite falsificar [Fluhrer23].

**Consistencia ante caídas.** Pillai et al. mostraron lo difícil que es escribir protocolos de
actualización correctos ante caídas sobre sistemas de ficheros reales, con una herramienta, ALICE, que
busca vulnerabilidades de caída en los protocolos de actualización de las aplicaciones [Pillai14]; y
Chidambaram et al., cómo separar orden y durabilidad [Chidambaram13]. POSIX admite una implementación
nula de `fsync()` cuando `_POSIX_SYNCHRONIZED_IO` no está definido [POSIX24], aunque la glibc de Linux sí
lo define; el riesgo práctico es otro, el de dispositivos que no conservan lo confirmado ante un corte de
corriente, que Zheng et al. estudiaron en SSD [Zheng13]. Todo ello es coherente con que el autocontrol de
§4.8 mida el coste de `fsync` en lugar de fiarse de su valor de retorno; la razón que da el propio
repositorio es la medida en tmpfs de `AUDITORIA.md` §234.

**Posición.** Lo que aquí podría ser nuevo no son las piezas, sino dos cosas: la **separación explícita**
de los predicados en un solo modelo, con lo que cada uno no detecta (§3.6) —la literatura de clientes
ligeros ya separa la validez de un bloque de la disponibilidad de sus datos [AlBassam21]—, y la
**reconciliación como componente clasificado** del modelo (§3.7-§3.8). El **contrato semántico
ejecutable** de HBS-STATE —cuatro estados, un juez, vectores y un protocolo de sujeto— (§4) es trabajo
previo del autor, que aquí se analiza junto con las **dos instanciaciones**, sus rutas de código y sus
huecos declarados.

---

## 3. El modelo ECST

### 3.1 Objetos

Un sistema con estado avanza por transiciones i = 1, 2, … Para cada una, ECST exige un paquete

```text
E_i = ( C_{i-1}, C_i, O_i, Π_i, A_i, H_i )
```

- $C_{i-1}$, $C_i$: el **compromiso** del estado anterior y del nuevo, $C_i = \mathsf{Com}(S_i)$ con
  $\mathsf{Com}$ vinculante. Los borradores escribían $S_{i-1}$ y $S_i$; lo que viaja y se encadena es un
  compromiso, no el estado.
- $O_i$: la **operación**, con su clase y sus parámetros públicos.
- $\Pi_i$: la **evidencia criptográfica**: una prueba, un sello o, explícitamente, nada; la ausencia se
  codifica con una constante de dominio propio, distinta de $\mathrm{Hp}$ de la cadena vacía (R3).
- $A_i$: la **autorización**. Puede ir dentro de $\Pi_i$ —la autoría de un pago en Arqueo—, en un objeto
  aparte —el par de pruebas de dos custodios— o no existir.
- $H_i$: el **resumen acumulado de la historia**, con

```text
H_i = Hc( dom_c ‖ H_{i-1} ‖ Enc_v(i, O_i, C_{i-1}, C_i, a_i) ‖ Hp(Π_i) ),     H_0 = g
```

donde $a_i$ es la parte de $A_i$ que se encadena (puede ser vacía), $\mathrm{Hp}$ es el resumen de la
evidencia y $g$ una génesis constante y pública; $\mathsf{dom}_c$, $H_{i-1}$ y $\mathrm{Hp}(\Pi_i)$ tienen
anchura fija. Esta fórmula es el **modelo abstracto**; no es la de Arqueo, que se da en §5.4
(VERIFICACION.md, L1b). Ni $E_i$ ni $H_i$ comprometen $vk_i$ ni las reglas vigentes: si no se encadena un
resumen de ellas —por ejemplo, dentro de $O_i$—, las reglas de cada época entran en el nivel B de §3.10
como dato no autenticado, que es lo que permite cambiar el verificador sin rastro (§6.1). Arqueo no lo
encadena; su `paramsDigest`, en la cabeza desde la v5, compromete parámetros del sistema, no el AIR.

### 3.2 Requisitos de codificación

- **R1 · Inyectividad y canonicidad.** Tuplas distintas dan codificaciones distintas, y cada tupla tiene
  una sola codificación aceptada: campos de anchura fija, o con longitud y dominio.
- **R2 · Versión.** La codificación lleva su versión, o una marca que la discrimina, y un verificador
  rechaza la versión que no conoce.
- **R3 · Separación de dominio.** $\mathsf{dom}_c$ no coincide con ningún otro uso de $\mathrm{Hc}$;
  $\mathrm{Hp}$ lleva su propio dominio y la longitud de la entrada.
- **R4 · Bytes reales.** $\mathrm{Hp}$ se calcula sobre los bytes de $\Pi_i$ tal como se verificaron,
  no sobre una constante ni sobre una reserialización.

Arqueo ha violado R4, y ha tenido un $\mathrm{Hp}$ no resistente a colisiones sin que se rompiera ninguna
permutación: el relleno interno de `digest_of_proof` añadía ceros sin codificar la longitud (`AUDITORIA.md`
§116, la cláusula de longitud de R3) y reducía bloques de 8 bytes módulo $p$ (§124), de modo que dos
pruebas distintas daban el mismo resumen. No fue un fallo de $\mathrm{Enc}_v$, sino de $\mathrm{Hp}$ como
función de los bytes de la prueba; por eso la Proposición 1 pide la resistencia a colisiones de esa
función, con su codificación interna, y no sólo la de su permutación. R4 se violó en seis vías que
encadenaron el resumen constante de la prueba vacía hasta el §278 (VERIFICACION.md, P9b); el consumo aún
encadena `digest_of_proof(&[])`, con el resumen del consumo en el compromiso (§5.4). R3 sólo se cumple
estructuralmente (§5.4).

### 3.3 Cuatro predicados

- $\mathsf{StateOK}(S_{i-1}, O_i, S_i)$: la transición respeta las reglas semánticas del sistema. Se
  evalúa sobre aperturas de $(C_{i-1}, C_i)$; sin ellas —un tercero en Arqueo— sólo es comprobable lo que
  $\Pi_i$ prueba.
- $\mathsf{Verify}(vk_i, x_i, \Pi_i) = 1$: la evidencia es criptográficamente válida para las entradas
  públicas $x_i$, derivadas de $(C_{i-1}, C_i, O_i)$, con la clave de verificación $vk_i$ vigente en $i$.
- $\mathsf{Authorize}(A_i, O_i)$: quien debía autorizar $O_i$ lo hizo.
- $\mathsf{Link}(H_{i-1}, H_i, E_i)$: $H_i$ es la recomposición de la fórmula de §3.1 sobre los campos de
  $E_i$.

Sobre un tramo se definen $\mathsf{HistoryOK}_n$ —$\mathsf{Link}$ en cada paso y $H_n$ igual a una cabeza
autenticada $\hat H_n$— y $\mathsf{CryptoValid}_n$ —$\mathsf{Verify}$ en cada paso—.

### 3.4 La tesis de separación

> **Compromiso ≠ Validez ≠ Conformidad semántica ≠ Consenso.**

El lema es de no implicación **en general**. En un sistema concreto una propiedad puede implicar otra por
construcción —en el envío de Arqueo, $\mathsf{Verify}$ implica $\mathsf{Authorize}$ porque la autoría va
dentro de $\Pi_i$—, pero eso es una afirmación sobre ese sistema que hay que demostrar.

- **HistoryOK ⇏ CryptoValid.** La cadena enlaza resúmenes; no dice que las pruebas resumidas verificaran
  (VERIFICACION.md, P10a). Caso real: hasta el 31-07-2026 la capa de Arqueo aceptaba envíos y cobros sin
  verificar su prueba, y nada en la cadena lo habría delatado (`AUDITORIA.md` §73).
- **CryptoValid ⇏ StateOK.** La solidez es relativa a la relación que el circuito restringe, y que esa
  relación capture todas las reglas es otra afirmación. Caso real: en el §487 se midió que una cuenta
  congelada producía pruebas que verificaban en cinco circuitos; lo cerró primero la capa, que rechaza
  con `AccountFrozen`, y el §511 añadió al AIR de esos circuitos el atado de la posición (`AUDITORIA.md`
  §487, §511).
- **Ninguno implica Consenso.** Un operador puede mostrar dos historias coherentes a dos observadores; sólo
  lo detecta quien compara cabezas firmadas del mismo índice. Tampoco implican **completitud**: una
  operación que nunca entró en la cadena no altera ningún $H_i$.

### 3.5 Proposición 1: atado retroactivo

**Proposición 1.** Sean $\mathrm{Hc}$ y $\mathrm{Hp}$ resistentes a colisiones y de salida de anchura
fija —$\mathrm{Hp}$ como función de los bytes de la prueba, con su codificación interna—;
$\mathrm{Enc}_v$ inyectiva (R1) para una versión $v$ fija y conocida (R2); y $\mathrm{Hp}$ calculado sobre
los bytes de $\Pi_i$ tal como se verificaron (R4). Sean dos historias $(E_1,\dots,E_n)$ y
$(E'_1,\dots,E'_n)$ con la misma génesis $H_0$ y la misma longitud $n$, que cumplen $\mathsf{Link}$ en cada
paso y que difieren en algún campo encadenado —$O_k$, $C_{k-1}$, $C_k$, $a_k$ o $\Pi_k$— de algún $E_k$ con
$k \le n$. Entonces, si $H_n = H'_n$, a partir de las dos historias se exhibe, con a lo sumo $2n$
evaluaciones de $\mathrm{Hc}$ y $2n$ de $\mathrm{Hp}$, una colisión explícita de una de las dos. Como
$\mathrm{Hc}$ y $\mathrm{Hp}$ no llevan clave, se lee como una reducción constructiva [Rogaway06]: quien
produzca tales pares produce una colisión, con al menos el mismo éxito y un coste adicional lineal en $n$.
En el modelo abstracto R4 se cumple por definición; se nombra porque es la condición para que una
implementación instancie la fórmula.

*Esbozo.* Se recomponen las dos cadenas. Sea $u_i$ la entrada completa de $\mathrm{Hc}$ en el paso $i$. Se
recorre desde $i = n$ hacia atrás, sabiendo que $H_i = H'_i$. Si $u_i \neq u'_i$, el par es una colisión
de $\mathrm{Hc}$. Si $u_i = u'_i$, entonces, como $\mathsf{dom}_c$, $H_{i-1}$ y $\mathrm{Hp}(\Pi_i)$ tienen
anchura fija, $u_i$ se descompone de una sola manera: $H_{i-1} = H'_{i-1}$, los campos codificados
coinciden (por R1) y $\mathrm{Hp}(\Pi_i) = \mathrm{Hp}(\Pi'_i)$; si además $\Pi_i \neq \Pi'_i$, es una
colisión de $\mathrm{Hp}$. Como las historias difieren en un campo encadenado de $E_k$, el recorrido no
puede llegar por debajo de $k$ sin encontrar una colisión. ∎

**Lo que la proposición no dice.** (i) Nada de los campos que **no** se encadenan: si una clase de
operación encadena un sello en lugar de su prueba, alterar esa prueba no cambia $H_n$ (§5.4). (ii) La
restricción a la misma génesis y la misma longitud es de comodidad: como $i$ entra en $\mathrm{Enc}_v$ y
$H_0$ en $u_1$, el mismo recorrido da una colisión si $H_n = H'_m$ con $n \neq m$ o con $g \neq g'$. Lo que
no da es que el tercero sepa a qué génesis y a qué $n$ corresponde la cabeza: eso es D2. (iii) Nada de la
validez de lo encadenado: es $\mathsf{HistoryOK}$, no $\mathsf{CryptoValid}$. (iv) Es la propiedad de
Haber y Stornetta [HS91], base de [CW09] y de CT [RFC6962]; ECST la usa, no la aporta (VERIFICACION.md,
P9c).

**Condiciones para que un tercero la aproveche.** Son del despliegue, no del teorema, y se separan a
propósito (VERIFICACION.md, P9b):

- **D1 · Cabeza autenticada y anterior a la alteración.** $\hat H_n$ tiene que llegarle firmado, con la
  clave anclada antes del primer encuentro, y custodiado por él o por un testigo **antes** de la
  alteración que se quiere detectar: frente al firmante, que puede volver a firmar, una cabeza firmada
  después de reescribir la historia autentica la historia reescrita. En Arqueo el ancla es TOFU y no hay
  clave de operador real (SECURITY.md, «NO HAY CLAVE QUE ANCLAR»; §5.6).
- **D2 · Génesis y longitud conocidas**: sin ellas, una cabeza correcta de un prefijo pasa por la de la
  historia entera.
- **D3 · Disponibilidad.** Las entradas y los bytes de cada $\Pi_i$, para recomponer $\mathrm{Hp}$. Arqueo
  no conserva las pruebas (§5.4).
- **D4 · La versión de la codificación**, y en Arqueo la era de cada entrada, para saber qué campos entran
  y cómo.

**En Arqueo** (§5.4), $\mathrm{Hc}$ es `native_merge` (Rescue-Prime Rp64_256 sobre Goldilocks) y
$\mathrm{Hp}$, Blake3-256 con dominio y longitud, reducido al campo; esa reducción no es inyectiva —los
valores mayores o iguales que $p$ colisionan con valores pequeños—, una pérdida marginal **[DERIVADO]**:
cada uno de los cuatro limbs cae en $[p, 2^{64})$ con probabilidad $\approx 2^{-32}$, y una colisión
debida a la reducción exige además que los otros tres limbs coincidan. La resistencia a colisiones de
ambas funciones es un supuesto. La proposición se traslada a la composición anidada de §5.4 merge a merge,
porque cada entrada de `native_merge` tiene anchura fija; pero, sin etiqueta de dominio, dos historias que
difieran en la era de una entrada sólo dan una colisión si además es difícil encontrar una salida de
`native_merge` con forma prefijada —`[x,0,0,0]` o la constante `COMPROMISO_AUSENTE`—, una propiedad de
tipo preimagen que la resistencia a colisiones no implica; por eso D4 incluye la era. El módulo
`t1_chain_retroactivo` (`crates/zk-ssl/src/log.rs:1189-1295`) ejerce la proposición en **un** caso —12
entradas, alteración en la 5, pruebas sintéticas de 8 bytes, sin STARK—: la recomposición desde los
campos crudos reproduce la cabeza, la divergencia se localiza en k = 5 —con las dos historias a mano— y
contamina todo j ≥ 5, y `verify_chain` detecta la sustitución
**[REPRODUCIDO: `3 passed`, 2026-09-28 y 2026-09-30]**. Comprueba el encadenado; no mide la resistencia a
colisiones (`AUDITORIA.md` §116; VERIFICACION.md, M14a).

### 3.6 Qué detecta cada capa y qué no

| capa | detecta | no detecta |
|---|---|---|
| **Link / HistoryOK** | con una cabeza autenticada **custodiada antes** de la alteración y $n$ conocido, la alteración, inserción, borrado o reordenación de un campo encadenado (Proposición 1 y su nota (ii)) | que una transición sea inválida; lo que nunca entró en la cadena; una vista dividida si nadie compara; los campos no encadenados |
| **Verify / CryptoValid** | una transición que no cumple la relación del circuito, si el sistema de prueba es sólido | que la relación capture las reglas; que la prueba esté en la historia; que las reglas vigentes sean las publicadas, si el operador puede cambiar el verificador sin rastro |
| **Authorize** | una operación sin la autorización que su clase exige | que la autorización corresponda a una voluntad independiente: dos claves comprometidas autorizan igual |
| **StateOK** | la regla semántica que se comprueba fuera del circuito | una regla que nadie codificó |
| **Reconciliación R(C,K)** | tras reiniciar, la violación del invariante visible en el par (KeyAhead); clasifica el resto en tres estados no fatales, con su intervalo perdido o indeterminado | la restauración conjunta de contador y clave; un contador borrado o puesto a cero cuando la clave nace en cero (§8.1); la durabilidad ante un corte de corriente |
| **Cabeza firmada y testigos** | una vista dividida entre quienes comparan; un cambio de clave tras el primer encuentro | el primer encuentro; testigos que coluden o que no existen |
| **Completitud** (RFC-0010 en Arqueo) | que una operación **con recibo** no se resolvió en su ventana | la operación a la que nunca se dio recibo |

### 3.7 Reconciliación semántica del estado físico

Sea un recurso de **consumo monótono** —un índice de firma de un solo uso, un número de recepción— con dos
datos durables: $C$, un contador que se reserva y persiste **antes** de usar el recurso, y $K$, lo que el
propio recurso dice haber alcanzado —el índice dentro de la clave, el mayor número anotado en un
registro—. El invariante de una ejecución correcta es

> **Todo uso tiene un índice de hoja, contado desde 0, menor que el contador persistido; equivalentemente,
> $K \le C$.**

Con el índice declarado de §5.5, que es la hoja más uno, la misma regla se escribe «ningún índice mayor que
el contador», que es la redacción de HBS-STATE y de `AUDITORIA.md` §234 (§4.2); EIP-8310 fija la convención
con «signing is permitted only strictly above» la marca de agua [EIP8310]. La reconciliación es una
función pura $R : \mathbb{N} \times \mathbb{N} \to$ {InSync, KeyAtZero, CounterAhead, KeyAhead}, con un
campo derivado por estado (§4.4). Su papel en ECST es convertir «el proceso murió» en un estado explícito
y clasificable, con un **juez** que marca un único estado como evidencia de violación del invariante
(KeyAhead) y una **política** que pertenece al dueño del recurso.

Dos advertencias. **El significado de cada estado depende del modelo de clave** (§4.3): con clave derivada
de semilla, todo reinicio real da KeyAtZero; con clave persistida, CounterAhead es lo normal tras una
caída. Y **R sólo ve el par**: un retroceso que lo deja coherente —restaurar juntos contador y registro, o
borrar el contador o ponerlo a cero cuando la clave nace en cero— es invisible para R y exige un dato fuera
del par.

### 3.8 La regla de los índices indeterminados

Cuando $R$ devuelve un estado con un intervalo cuyo uso no se puede decidir con los datos durables
—$[0, C)$ en KeyAtZero, donde con contador 1 y clave 0 morir antes de firmar y morir después dejan el mismo
estado en disco (HBS-STATE v0.3, §2); $[K, C)$ en CounterAhead con clave persistida, donde una firma pudo
salir antes de persistirse la clave—, la regla es:

1. no reutilizar nunca un índice menor que $C$;
2. continuar desde $C$, el primer índice que nunca se reservó, y dar los indeterminados por perdidos: «un
   índice perdido es mejor que uno indeterminado» (la nota 92, entrada de `BACKLOG.md`, tal como la cita
   `crates/zk-ssl-guardian/src/lib.rs:146-147`);
3. detenerse sólo si un registro durable **independiente** contradice $C$, o si $R$ devuelve el estado
   fatal.

Dar por perdido lo reservado y no confirmado es la semántica de la reserva por intervalos de McGrew et al.
[McGrew16], que RFC 10033 (§5.9) resume así: los índices reservados se pierden si el dispositivo pierde la
corriente o se reinicia. Y no quedar parado para siempre es la exigencia de vivacidad de [Ariadne16]. No es
la regla «KeyAtZero ⇒ no firmar», que haría que un firmante derivado de semilla no volviera a firmar nunca
tras su primer reinicio (VERIFICACION.md, P15a, G1f), ni es texto normativo de HBS-STATE, que deja la
política al dueño y exige quemar o registrar los huérfanos (§4.5). ECST adopta como regla general la
política que Arqueo ya aplica a su índice de firma en ClaveEnCero desde el §335 y §337 (§5.13); su
aplicación a CounterAhead y, por lo que muestra §8.1, la consulta del registro independiente del punto 3
**en todos los estados** y no sólo en KeyAtZero son **[PROPUESTA]**.

### 3.9 Uso único: por contador y por conjunto

La propiedad abstracta: cada elemento de un conjunto de recursos se consume **como mucho una vez** y, si el
consumo tiene un efecto externo, se registra antes de producirlo: es la respuesta habitual al problema de
*output commit* de la recuperación por retroceso, que ninguna salida al exterior tenga que revocarse
[Elnozahy02]. Aparece con dos mecanismos. Por contador monótono —la hoja de un árbol XMSS, reservada por
el guardián antes de firmar, y el número de recepción `rx` del RFC-0010—, al que se aplica R(C,K). Y por
conjunto —la etiqueta de consumo del RFC-0006, única dentro de un libro, y la hoja de un pendiente, que el
circuito del cobro deja vacía—, que no tiene par que reconciliar. Con un solo escritor, la propiedad se
**impone**; entre escritores independientes, en el mejor caso se **detecta** después (§5.9).

### 3.10 Protocolo de auditoría por niveles

**[PROPUESTA]** como herramienta; algunas piezas existen en Arqueo (§5.14).

- **Entrada:** una cabeza autenticada $\hat H_n$ (firma y ancla), custodiada antes de la alteración que se
  quiere detectar (D1); las entradas de un tramo; y las reglas —la $vk_i$ y la versión de la codificación—
  vigentes en cada paso.
- **Nivel A · historia:** recomponer $H_i$ desde la génesis o desde una cabeza custodiada. Válido si
  $H_n = \hat H_n$; Inválido si no; Indeterminado si faltan entradas o la especificación de la
  codificación. Con una sola cabeza, Inválido vale para el tramo entero: el índice divergente sólo se
  acota si hay valores $\hat H_j$ autenticados intermedios —cabezas firmadas de épocas anteriores—, y sólo
  al tramo entre la última que coincide y la primera que no.
- **Nivel B · evidencia:** para cada $i$ con nivel A Válido, comprobar que $\mathrm{Hp}(\Pi_i)$ coincide con
  el resumen encadenado en la entrada $i$ y después $\mathsf{Verify}$ sobre $\Pi_i$ con las $x_i$ derivadas
  de esa entrada. Indeterminado si $\Pi_i$ no está disponible o no se conocen las reglas de su época.
- **Nivel C · autorización y semántica:** $\mathsf{Authorize}$ y $\mathsf{StateOK}$ donde se puedan comprobar
  con datos públicos; Indeterminado donde no.

El resultado es un **vector** $(r_{\mathsf{Link}}, r_{\mathsf{Verify}}, r_{\mathsf{Auth}}, r_{\mathsf{State}})$
por transición, con un componente por predicado y cada uno en {Válido, Inválido, Indeterminado, No aplica};
No aplica es para las clases que declaran $\Pi_i$ vacía. Sobre un tramo, cada componente se agrega con
Inválido por encima de Indeterminado y éste por encima de Válido, y se informa la cobertura. Nunca se
colapsa a un booleano, y un Indeterminado no se lee como Válido. Sin recursión, el coste es lineal en n;
con IVC, el verificador no depende de n, a cambio de verificar dentro de cada paso (VERIFICACION.md,
P11b). Cuál conviene para una carga concreta **no está medido** (§7.4).

---

## 4. Instanciación 1: HBS-STATE

### 4.1 Qué es y de dónde sale

HBS-STATE v0.3 especifica **una** propiedad: cómo se decide, tras un reinicio, si el estado de una firma
basada en hash con estado sigue siendo seguro de usar, con cuatro estados, un juez y vectores numéricos.
Presupone [SP800-208] y [RFC8391] y no es una implementación de XMSS (HBS-STATE v0.3, spec/ de
`hbs-state`). Su implementación de referencia, el crate `hbs-state` 0.2.0, sin dependencias y bajo MIT OR
Apache-2.0 (los vectores, CC0-1.0), es una traducción casi literal de `zk-ssl-guardian`, el guardián del
índice de Arqueo, con la reconciliación extraída como función pura, un binario sujeto y `tests/vectors.rs`
(VERIFICACION.md, H1a, H13c, G1a).

### 4.2 El invariante

> **No signature may exist with an index greater than the persisted counter.**

`IndexGuard::reserve` persiste `current + 1` —ocho bytes *little-endian* en sitio y `sync_all`— antes de
devolverlo (`hbs-state/src/lib.rs:482-489`, `:506-519`). No hay `fsync` del directorio, ni temporal con
renombrado, ni cerrojo, y `open` crea el contador a 0 si no existe, sin avisar: un contador borrado es
indistinguible de uno nuevo (VERIFICACION.md, H2a, H2b). Frente a la muerte del proceso **[MEDIDO:
`AUDITORIA.md` §234, banco K.1, una sola máquina, sobre el guardián original de Arqueo cuando aún no
tenía consumidor —antes del firmante de cabezas (§236) y de que existiera `hbs-state`— y sin `xmss` como
dependencia]**, un hijo que persiste y firma, matado con `kill -9` en un instante aleatorio, no dejó
ninguna firma por delante del contador en 25 de 25 muertes; qué «firmaba» el hijo no consta. Se aplica a
`IndexGuard` por ser traducción casi literal de ese guardián (VERIFICACION.md, H18a, G7a, G1a). El código
del banco no está en los árboles; con 0 fallos en 25, la cota superior al 95 % es ≈ 11 %
**[DERIVADO: 1 − 0,05^(1/25)]**, una cota de la tasa por muerte aleatoria, no de la corrección del orden,
porque no se conoce la fracción del ciclo en que un orden invertido fallaría. Y mide que la escritura del
contador precede a la firma tal como la ve la caché de páginas —un `kill -9` no distingue una escritura
con `sync_all` de una sin él—, no la persistencia ni un corte de corriente. Lo que acotaría el orden es
inyectar la caída en cada llamada al sistema, como hace ALICE [Pillai14].

### 4.3 Los dos modelos de clave

Antes de clasificar hay que responder a «la pregunta cero»: tras un reinicio, ¿de dónde sale el índice que
lleva la clave privada? (HBS-STATE v0.3, §1)

| | **clave persistida** | **clave derivada de semilla** |
|---|---|---|
| qué hay en disco | el material de clave con su índice | sólo la semilla |
| al reiniciar | el índice sobrevive | `from_seed` la devuelve en el índice 0 |
| estados tras un reinicio | InSync, CounterAhead, KeyAhead, y KeyAtZero si el sujeto borra la clave al agotarse | **siempre KeyAtZero** (o InSync si el contador es 0) |
| CounterAhead | el caso normal tras una caída | sólo dentro de un proceso vivo |

El contraejemplo que corrigió esa tabla es `hbs-lms` 0.2.0-alpha.1 (Fraunhofer AISEC, commit 7063cc8): con
clave persistida, borra la clave al agotar el árbol y alcanza KeyAtZero, que no distingue una clave
borrada, terminal, de un reinicio; el discriminante es el conjunto de parámetros, que el índice no lleva
(HBS-STATE v0.3, §0-§1; VERIFICACION.md, H4). El «13 de 25 muertes dejan el contador adelantado»
**[MEDIDO: §234]** es de muertes dentro de un proceso, y la inferencia «lo normal tras una caída es
CounterAhead» está retractada en la propia especificación como regla general y para la clave derivada de
semilla; con clave persistida sigue valiendo, como dice la tabla (VERIFICACION.md, H18b, G7b).

### 4.4 Los cuatro estados

Sea `counter` lo reservado y persistido y `key` el índice que lleva la clave.

| estado | condición | campo derivado | fatal |
|---|---|---|---|
| InSync | `key == counter` | `index = counter` | no |
| KeyAtZero | `counter > key` y `key == 0` | `indeterminate = counter` | no |
| CounterAhead | `counter > key` y `key != 0` | `orphans = counter − key` | no |
| KeyAhead | `key > counter` | `unrecorded = key − counter` | **sí** |

Las cuatro condiciones forman una partición de $\mathbb{N}^2$, así que no hace falta precedencia entre
ellas; el código la realiza con un `match` sobre `counter.cmp(&key)` —los tres valores de `Ordering`— con
una sola guarda, `Greater if key == 0`, como brazo previo al `Greater` general, y la especificación llama
«precedencia» a esa guarda: (0,0) es InSync, (0,1) es KeyAhead, `indeterminate` es un solo término y no
una resta, y no hay variante de agotamiento (`hbs-state/src/lib.rs:412-431`; HBS-STATE v0.3, §2 y §8;
VERIFICACION.md, H3).

### 4.5 El juez y la política

El juez es `fatal(state) = (state is KeyAhead)`; `is_fatal` es un predicado puro y detenerse es deber del
llamante. El código de producción de Arqueo no llama a su equivalente, `no_admite_matiz`: cada política
hace su propio `match`; en el testigo, un test exige que su política no arranque donde `no_admite_matiz`
marca fatal, y la política de firma del nodo no tiene ese atado: sus tests enumeran los casos a mano
(`crates/zk-ssl-cli/src/witness.rs:3050-3075`; VERIFICACION.md, H6a). **La política pertenece al dueño**:
la especificación sólo exige que el contador no retroceda y que los huérfanos se quemen o se registren (§2,
§9); no exige intervención ante KeyAtZero ni lo clasifica como violación en ningún modelo (VERIFICACION.md,
H6b, H6c). Su «KeyAtZero cannot be resolved, only failed closed» significa «no se reutilizan los índices
indeterminados», no «no se opera»: N3 exige rehusar operar **sólo** en KeyAhead (VERIFICACION.md, G1f).

### 4.6 Conformidad: N0 a N3 y el protocolo de sujeto

Los niveles son acumulativos (HBS-STATE v0.3, §4):

- **N0 · declara** su modelo, `persisted` o `seed_derived`. Desde la v0.3 el verificador lo exige: quien no
  lo declara queda sin nivel aunque clasifique bien.
- **N1 · clasifica** los cuatro estados. Fundir CounterAhead y KeyAhead en un «error de estado» no cumple
  N1.
- **N2 · deriva** `index`, `orphans`, `indeterminate` y `unrecorded`.
- **N3 · juzga**: marca como fatal exactamente KeyAhead.

Un **sujeto** es un ejecutable que responde a `--model`, `<counter> <key>` y `--sk <hex>`; el verificador
lee sólo la última línea de la salida estándar, que debe ser JSON, y todo fallo —código de salida distinto
de cero, salida estándar vacía o última línea que no es JSON— cuenta como fila fallada, con 60 s por
invocación como máximo. Comprueba el booleano `fatal` sin ejercer un rechazo real, y el sujeto de
referencia declara `seed_derived` sin tocar disco ni claves: su N3 certifica la función pura, no la
durabilidad de `IndexGuard` (VERIFICACION.md, H9a, H9c).
**[REPRODUCIDO, 2026-09-28]**: familia A N1, N2 y N3 12/12, familia B 6/6, N0 `seed_derived`: nivel N3.

### 4.7 Familias de vectores

- **Familia A · reconciliación**: A1 a A12 (`spec/state-vectors-v0.3.json`, esquema `hbs-state/0.3`),
  incluidos los extremos `2^40−1` (A11, A12).
- **Familia B · lectura del índice** en el formato de clave de `XMSSMT-SHA2_40/8_256` (OID `0x00000005`,
  h = 40, d = 8): 137 bytes = OID(4) ‖ índice(5, *big-endian*) ‖ 4×32; en árbol único el índice ocupa 4 bytes
  y la clave 136. B1 a B6 leen; B7 escribe `2^40` y debe fallar con `IndexOutOfField` **sin tocar un byte**.
  El verificador en Python ejerce 18 filas (12 de A y 6 de lectura); B7 sólo lo ata `tests/vectors.rs`
  (VERIFICACION.md, H10a, H9c).
- **Declarados sin vectores**: B8 (con h no múltiplo de 8, escribir `2^h` debe fallar aunque quepa en el
  campo; la implementación de origen no lo cubre) y la familia C, siete errores sobre el estado en disco,
  entre ellos `FakePersistence` (VERIFICACION.md, H19d).

Ese formato de clave **no está normalizado**: RFC 8391 (§4.1.7, §4.2.2), RFC 8554 (§4.2, §5.2) y SP 800-208
renuncian a definirlo. Es el de la implementación de referencia en C y de su traducción `xmss`
0.1.0-pre.0; la revisión del 2026-09-27 retiró la atribución a RFC 8391, y hacer la familia B condicional a
un formato declarado queda para la siguiente (HBS-STATE v0.3, §6; VERIFICACION.md, H11b, H11c).

### 4.8 El autocontrol de `fsync`, tal como está implementado

`IndexGuard::open` escribe una sonda, `.hbs-state-selfcheck`, en el directorio padre del contador:
20 escrituras de 8 bytes sin `sync_all` y después 20 con `sync_all`, cronometradas cada tanda con un solo
`Instant` y divididas entre 20. Calcula la razón de medias con/sin y devuelve
`FakePersistence{with_fsync_us, without_fsync_us, ratio}` **si y sólo si** la razón es menor que 10 **y** el
coste medio con `fsync` es menor que 20 µs (`hbs-state/src/lib.rs:523-556`). No es una distribución
estadística ni una diferencia de latencias (VERIFICACION.md, H7b, H7d). `zk-ssl-guardian` aplica el mismo
algoritmo (VERIFICACION.md, G1d).

Los umbrales son **[DECLARADO]**: 20 muestras, 10× y 20 µs, fijados a partir de una sola máquina. La
medida que los motiva es **[MEDIDO: `AUDITORIA.md` §234, WSL2 sobre i5-1135G7, sin n ni dispersión
publicados]**:

| sistema de ficheros | coste de `fsync` | frente a escribir sin persistir |
|---|---|---|
| ext4 (`$HOME` en WSL2, un disco virtual) | 0,907 ms | 382× |
| tmpfs (`/tmp`) | 0,002 ms | 1× |

El 382× es la razón de `fsync` frente a no persistir **en el mismo ext4**, no el cociente ext4/tmpfs
(VERIFICACION.md, M5b). Una réplica literal del autocontrol (`doc/ecst/borrador/sonda-fsync.rs.txt`), 30
veces por ruta en dos tandas, dio **[REPRODUCIDO, 2026-09-28, microVM Firecracker, 4 vCPU, raíz ext4 sobre
virtio]**:

| ruta | con `fsync`, µs, p50 [mín–máx] | sin `fsync`, µs, p50 | razón p50 [mín–máx] | rechazos |
|---|---|---|---|---|
| ext4, tanda 1 | 128,9 [74,7–292,2] | 0,876 | 161,4 [92,6–292,9] | 0/30 |
| ext4, tanda 2 | 173,1 [103,6–795,2] | 0,879 | 189,4 [116,9–986,5] | 0/30 |
| tmpfs, tanda 1 | 0,537 [0,531–1,582] | 0,421 | 1,3 [0,7–3,8] | 30/30 |
| tmpfs, tanda 2 | 0,536 [0,533–0,732] | 0,421 | 1,3 [0,7–1,8] | 30/30 |

Lo que esto enseña es un límite. Dentro de una microVM, el invitado no puede saber si el anfitrión lleva el
vaciado hasta el medio físico: el autocontrol sólo demuestra que `fsync` **cuesta algo**, que es lo que
declara medir. Detecta el caso «no hay disco» (tmpfs); no detecta un disco o una capa virtual que confirme
escrituras aún en caché volátil, ni un `fsync` falso que cueste 20 µs o más, o que cueste al menos 10
veces una escritura sin persistir (VERIFICACION.md, G9a). Los aproximadamente 100 µs de un NVMe que citan
los comentarios son una suposición, no una medida (VERIFICACION.md, H7f).

### 4.9 Lo que no cubre

Por su §9: cómo se persiste el contador —`fsync`, orden, cerrojos, copias—; la política; varios firmantes
sobre la misma clave pública; LMS y HSS, «la tabla debería valer; no se ha probado»; la familia C y B8. Un
defecto ajeno impide además ejercer de extremo a extremo la rama de clave persistida con la librería Rust
disponible: `xmss` resuelve el OID probando primero árbol único, y 21 de los 56 conjuntos XMSS^MT cargan
mal; `pq-xmss` ya no tiene el defecto (HBS-STATE v0.3, §9; VERIFICACION.md, G10c).

### 4.10 Estado

**[DECLARADO]** HBS-STATE v0.3 es un **BORRADOR**, revisado el 2026-09-27, derivado de **una sola**
implementación; mientras un segundo sujeto independiente no pase los vectores, «describe un programa»
(§10). La etiqueta MEASURED de sus vectores significa «tomado del código y los tests del guardián de
Arqueo»: es autoconformidad (VERIFICACION.md, H19a). `hbs-lms` se midió como segundo **implementador**, con
un arnés aparte y no publicado, sin pasar por el protocolo de sujeto y sin nivel atribuido; `pq-xmss`
comparte autor y código con `xmss` y no es un segundo implementador (VERIFICACION.md, H19b, H19c).
**[REPRODUCIDO, 2026-09-28]**: `cargo test --release` en `hbs-state` sale verde (27 + 8 `passed`,
0 `failed`).

### 4.11 El coste de reutilizar un índice

HBS-STATE v0.3 §2 da, con la etiqueta MEASURED, «a la segunda repetición de un índice, falsificar cuesta
del orden de 2^34 hashes». La cifra procede de un artículo del blog de QRL, «Statefulness and security»,
consultado el 2026-08-12 (`AUDITORIA.md` §288), que no tiene entrada en `referencias.bib` porque sus
metadatos no se pudieron verificar desde el entorno de este informe: ~2^34 hashes con **dos** firmas sobre
el mismo índice, ~2^23 con tres, ~2^18 con cuatro. Es por tanto **[CITADO]**, no medida; 2^34 es el punto
**más caro** de esa curva y no un mínimo, así que la fórmula de los borradores, «as low as 2^34», invierte
el sentido; y «segunda repetición» debe leerse como dos firmas, es decir, un solo reúso (VERIFICACION.md,
H15, B17a, B17b, B17c). Las fuentes primarias que cita RFC 10033 son [BH17] y [Fluhrer23], y la segunda
corrige a la primera: su análisis de Winternitz suponía independientes probabilidades que no lo son. La
cifra de QRL no se ha cotejado con ninguna de las dos, así que no debe citarse como cifra; basta la
conclusión común a las dos: un solo reúso permite falsificar con coste factible.

### 4.12 Correspondencia con ECST

HBS-STATE instancia **sólo** la reconciliación R(C,K) con su juez y el consumo monótono de índices OTS:
no tiene $\Pi_i$, ni cadena, ni cabeza firmada, ni autorización. Lo que aporta, y ninguna fuente de §2
trae todavía —EIP-8310 anuncia casos de prueba de regresión de la marca de agua y de caída, pero aún no
los publica—, es un **contrato semántico ejecutable** contra el que otra implementación se puede medir.

---

## 5. Instanciación 2: Arqueo

### 5.1 Modelo de confianza

**[DECLARADO]** Arqueo es un prototipo de investigación, no un producto; no hay producción ni operador
real, y no maneja dinero real (SECURITY.md, «Estado del proyecto»; `doc/INSTITUCIONAL.md:91-92`;
RFC-0010, «Seguridad»). Un nodo, un escritor, sin consenso distribuido ni token. El operador ve todos los
saldos, ordena las operaciones y puede censurar. Nada del proyecto ha sido auditado por
terceros (README.md; SECURITY.md). El guardián nació en la primera quincena de agosto de 2026
(`AUDITORIA.md` §234), unas siete semanas antes de publicarse `hbs-state`; las frases «in production for a
year» (README.md:170) y «a production system where it has been signing for a year» (README.md:33-34;
`src/lib.rs:73-74`) de `hbs-state` son falsas (VERIFICACION.md, G2a, H12b). El único nodo que
arranca hoy es el compilado con la feature `dev`, activa por defecto, cuyas raíces de custodios y de
gobernanza salen de claves de prueba escritas en el fuente (`crates/zk-ssl-node/src/main.rs:1336-1348`;
`crates/zk-ssl/src/tests_support.rs`).

### 5.2 Pago en dos fases y custodia de la clave

Un pago son dos transiciones: el pagador **envía** y el receptor **cobra**, cada uno con una prueba STARK
generada en su máquina a través del SDK (`Account::pay`, que llama a `client::prove_send`). La capa entrega
caminos y raíces y verifica (README.md, «Cómo funciona»). Tres precisiones:

- **Por diseño, no por tipos.** La clave de gasto no viaja por la API JSON-RPC: ningún método de producción
  la recibe, y en el SDK `Wallet::spend_key` es privado y `Wallet` no implementa `Serialize`. Pero la capa
  conserva métodos públicos que reciben la clave —`send`, `claim`, `burn`, `audit`, entre otros—, que
  usan los tests y las métricas y el nodo no llama; no hay una barrera de tipos (VERIFICACION.md, C2a, C1b).
- **Lo que la prueba publicaba.** Hasta el §538 la prueba que el nodo recibía llevaba **literal** la clave
  de gasto, y la del custodio en cada autorización delegada, porque winterfell 0.13 no oculta su testigo; el
  §521 es cuando se midió, no cuando empezó (README.md lo resume como «entre el §521 y el §538»). Desde el
  §538 (RFC-0009 E3b-2, `zkssl/0.4`) un fork en el árbol de winterfell 0.13.1 lo oculta, y la garantía es lo
  que mide la suite de E2 —cero literales—, no una demostración; ni el fork ni la construcción están
  auditados (README.md; SECURITY.md §3.bis; VERIFICACION.md, C3, C1c).
- **El pago no es firme hasta el cobro.** El importe queda inmovilizado hasta que se cobra o hasta el
  reembolso tras la caducidad T: en los pendientes v1, por defecto 64 entradas del registro (`log.seq`, no
  latidos de 60 s; `DEFAULT_REFUND_TTL`, `crates/zk-ssl/src/lib.rs:226`); en los v2, el plazo `delta`
  comprometido en el propio pendiente (`crates/zk-ssl/src/two_phase.rs:656-676`).

### 5.3 Qué revela una prueba

Frente a un tercero que sólo ve una prueba, desde el §538 no sale literal ningún saldo; antes, el envío y
el cobro publicaban el saldo antes y después. **El importe es entrada pública** de las pruebas de envío y
cobro; los metadatos muestran el emisor o el receptor y la posición del pendiente. Frente al operador no
hay privacidad ninguna (VERIFICACION.md, C4b, P10b). La solidez se apoya en ~127 bits **conjeturados**, no
demostrados (`crates/zk-ssl/src/lib.rs:207-221`; VERIFICACION.md, M8a), y la seguridad «post-cuántica»
significa sin supuestos de curva, no una seguridad cuántica cuantificada.

### 5.4 El registro encadenado

La composición real (`crates/zk-ssl/src/log.rs:250-294`), con $M$ = `native_merge` y
`emb(x) = [x,0,0,0]`:

```text
c1_i = M( M( M( M(emb(seq_i), emb(tag_i)), M(Racc_{i-1}, Racc_i) ), pd_i ), c_{i-1} )    era 1
c_i  = M( c1_i, compromiso_i )                                                          era 2
```

`chain_digest` da `c1_i` y `chain_digest_v2` la envoltura de la era 2, que rige toda entrada nueva desde
el §281. `tag` es el u64 de `OpKind` (1 a 13), las raíces `Racc` son las del árbol de **cuentas** y
`c_{-1} = [0;4]`. En disco, una entrada ocupa 137 bytes en la era 1 y 169 en la era 2, y la longitud
discrimina la era (`crates/zk-ssl/src/store.rs:411-455`).

**Sí incluye un resumen de prueba.** `pd_i` es `digest_of_proof`: Blake3-256 de
`b"ZK-SSL-proof-digest-v2"` ‖ longitud (u64 LE) ‖ bytes de la prueba, partido en cuatro u64 y reducido al
campo (`crates/zk-ssl-hash/src/lib.rs:1290-1320`). Pero depende de la clase (VERIFICACION.md, L1e, L1f):

- **Send, Claim, Burn y Refund** encadenan el resumen de su prueba STARK.
- **Mint, MintToPending, Recovery, Freeze y Governance** encadenan el resumen de `sello_de_autorizacion`
  sobre el compromiso autorizante, no los bytes de ninguna prueba.
- **OpenAccount** encadena el sello constante de ausencia declarada; **Migration**, el resumen de un bloque
  de 64 bytes con las raíces de congelados vieja y nueva; **Consumo**, `digest_of_proof(&[])` con el resumen
  del consumo como compromiso.

Por eso la Proposición 1 cubre las pruebas de envío, cobro, quema y reembolso, y no las de custodios. Las dos
ejecuciones de `simulate` lo ilustran **[REPRODUCIDO]**: con claves deterministas, las entradas Mint (log#1,
log#3) tienen el mismo resumen encadenado en ambas aunque su prueba cambió, y la Send (log#4), con las mismas
raíces, tiene resúmenes distintos (`54dd7634…` y `e1db21f3…`), porque con la ocultación los bytes de la prueba
dependen de la sal (`doc/ecst/borrador/simulate-2026-09-28.log`, `simulate-2026-09-30.log`). Es una
observación sobre dos árboles con la misma fórmula del registro, no un experimento diseñado.

Otros límites (VERIFICACION.md, L1d, L5c, L5d, L12):

- `chain_digest` y `chain_digest_v2` **no llevan etiqueta de dominio**: la era se separa por el merge
  adicional y por la longitud en disco (R3 se cumple sólo estructuralmente).
- **Las pruebas no se conservan.** El nodo guarda sólo el resumen, ningún método RPC sirve pruebas, y
  re-verificarlas depende de que el productor guarde la suya, sin política de retención (D3 falla).
- **Nadie recompone la cadena por un tercero.** La recomponen el nodo sobre sí mismo
  (`zkssl_verifyChain`), la CLI sobre su capa local y la capa al reabrir. `spec/RPC.md:97` describe
  `LogEntry` sin el campo `compromiso`, así que con la especificación en la mano no se puede recomponer
  `chain_digest_v2` (el DTO del cable sí lo lleva).

### 5.5 La cabeza de época v6

Desde el §570 el nodo compone, firma y sirve la cabeza **v6** (`VERSION_FORMATO = 6`,
`crates/zk-ssl-verify/src/lib.rs:166`), que compromete 18 campos: `seq`, las raíces de cuentas, pendientes
y congelados, `chainDigest`, la raíz de acuses y su techo `n` (1.440 cabezas), la cima y el tamaño del árbol
de historia de cabezas, la raíz y la cuenta de consumos, el resumen de parámetros, la raíz de meta de
pendientes, las dos marcas de agua (`nextPending`, `nextIndex`), el suministro, y la pareja
`recepRoot`/`recepCount` del registro de recepción (`crates/zk-ssl/src/log.rs:586-649`;
`crates/zk-ssl-hash/src/lib.rs:469-510`). Antes rigieron la v5 (§452), la v4 (§415), la v3 (§292) y la v2
(§275). Sólo se firman 50 bytes, `b"ZK-SSL-epoch-head"` (17) ‖ versión (1) ‖ `epoch_digest` (32), con
XMSS^MT-SHA2_40/8_256; los campos quedan cubiertos porque recomponen `epoch_digest`. La cabeza ata el
último resumen de la cadena, no las entradas (VERIFICACION.md, L2a, L2b, L2c).

El firmante (`crates/zk-ssl-node/src/firma_cabeza.rs:179-203`) **reserva el índice con `fsync` antes de
firmar** en el guardián y verifica su propia firma antes de devolverla; el índice declarado es el valor del
contador y la hoja gastada, embebida en la firma, es contador − 1 (VERIFICACION.md, G4c). El latido firma
cada 60 s por defecto y sólo si el operador arrancó con clave y diario; sin clave, la cabeza se calcula y
se sirve sin firma (VERIFICACION.md, G4d). La dependencia `xmss` 0.1.0-pre.0 de RustCrypto es una
*pre-release* sin auditoría independiente, clavada con `=` en el nodo, el verificador y la CLI
(VERIFICACION.md, G4b, M10b).

### 5.6 Testigos

El testigo de referencia (`crates/zk-ssl-cli/src/witness.rs`) fija por TOFU la clave que ve primero, se
detiene si cambia o si ve dos resúmenes con el mismo índice y, con `--cofirmar`, cofirma con su propia
clave XMSS y su propio guardián. No hay testigos independientes: sólo esa implementación, que corre el
autor (VERIFICACION.md, G5a, L7b). Las cofirmas son opcionales y cuántas bastan lo decide el cliente
(G5b); el nodo las transporta y comprueba su firma, pero no acredita a ningún testigo (G5c). El operador
sirve sólo la última cabeza firmada; el histórico lo llevan los diarios de los testigos (L4d).

### 5.7 Paquetes de evidencia y el verificador independiente

`zk-ssl-verify` es un binario que no depende de la capa, del nodo ni del cable, y acepta once formas de
paquete: posición v1 y v2, extensión, consumo, conflicto, rechazo, edad, cobro pendiente, pago en curso,
prenda y, desde el §573, completitud (`crates/zk-ssl-verify/src/main.rs:158-180`). Sale con 0 (VERDE), 1
(ROJO con la regla nombrada), 2 (uso) o 3 (el cuarto estado del sobre de completitud).

**Qué comprueba.** En el paquete de posición: recompone el `epochDigest`, verifica la firma XMSS contra la
clave que trae el propio paquete, sube el acuse bajo la raíz de acuses y cuenta las cofirmas. Verifica
pruebas STARK sólo en los sobres que las llevan —edad, cobro pendiente, pago en curso, prenda, ciertos
rechazos—, que son pruebas **nuevas** sobre el estado comprometido bajo una cabeza v5 o v6, no las $\Pi_i$
del registro (VERIFICACION.md, L5b).

**Qué no comprueba.** No re-verifica las pruebas de transición, que no se conservan; no recorre ni
recompone el registro; no verifica ningún agregado de saldos: el invariante suministro = saldos +
pendientes lo comprueba la capa del operador al reabrir, sobre saldos en claro (VERIFICACION.md, L9a,
C4c, C4d). La release publicada `arqueo-verify-v0.2.0` es anterior a las cabezas v5 y v6 y las rechaza; el
verificador del árbol sí las comprueba, pero no se ha publicado como release (VERIFICACION.md, L9b).
**[REPRODUCIDO, 2026-09-30]**: nueve catálogos, 257/257 entradas dicen lo que deben (§7.2).

### 5.8 Consistencia e inclusión

Lo que el código llama «MMR de cabezas» es un árbol de historia al estilo de RFC 6962 (MTH, PATH, SUBPROOF)
sobre primitivas propias, con dominios `MMRHOJA1`/`MMRNODO1`; sus hojas son cabezas anteriores, no
entradas (`crates/zk-ssl-verify/src/mmr.rs`; VERIFICACION.md, L4a). `zkssl_consistencyProof` da el camino
de extensión, y el paquete de extensión se verifica sin el nodo: dos cabezas v3 a v6 que recomponen y
verifican, la misma clave pública y la cima nueva que extiende a la vieja (VERIFICACION.md, L4b, L4c). La
estructura da **evidencia de manipulación, no inmutabilidad**: sólo lo ve quien custodia una cabeza
anterior de la misma clave; la prueba de consistencia no comprueba que el `chainDigest` nuevo continúe al
viejo, y el paquete de extensión no comprueba que la propia cabeza custodiada sea hoja del árbol nuevo
**[LECTURA]** (VERIFICACION.md, L4d).
La inclusión de una operación aplicada que el kit demuestra es la de su **acuse** bajo la raíz de acuses,
que sólo emiten `applySend` y `applyClaim`; además demuestra la de un consumo bajo `consRoot` y, desde el
§573, la de un recibo bajo `recepRoot` (VERIFICACION.md, L5a).

### 5.9 Uso único dentro de un libro, detección entre libros

El árbol de consumos del RFC-0006 da unicidad de una etiqueta pública acordada **dentro** de un libro: la
capa rechaza el consumo repetido, y la raíz y la cuenta de consumos van firmadas desde la cabeza v4. Entre
libros, un tercero con las dos cabezas firmadas detecta después la misma etiqueta, sin nodos: **detección,
no prevención**. El consumo no está atado a ningún pago, ni en circuito ni por autenticación del
publicador (VERIFICACION.md, C5a, C5b). El doble gasto de saldos lo impide otra cosa: el encadenamiento de
raíces bajo el orden total de un solo nodo, «evitado, no resuelto» (SECURITY.md §3.4).

### 5.10 Recuperación

La recuperación sustituye la identidad de **la misma hoja**, con el mismo saldo y el nonce + 1: el circuito
que usa la capa, `circuit_recovery_climb`, construye la hoja vieja y la nueva con la misma columna de saldo,
así que no mueve dinero (VERIFICACION.md, C6a, C6b). Incrementa en uno un contador **global** de
recuperaciones del libro, que entra en el compromiso que firman dos custodios y hace de nonce contra la
repetición; **no está en la cabeza firmada** ni en el RPC, y la prueba que lo asierta no se guarda
(VERIFICACION.md, C6c). La doble autorización está en la capa, no en el circuito: dos pruebas de dos
custodios distintos, con el umbral fijo en 2; nada prueba que el nuevo titular sea legítimo. La
recuperación no está expuesta en el JSON-RPC y sólo la ejercitan los tests (VERIFICACION.md, C6d, C6f).

### 5.11 Poderes del operador, y qué puede re-verificar un tercero

Con dos custodios se puede emitir (con el tope en circuito), emitir a pendiente, congelar o descongelar
cualquier cuenta sin justificación en circuito ni caducidad, y reasignar cualquier cuenta. Con dos miembros
de gobernanza se cambia el conjunto de custodios. El operador, solo, puede ver todos los saldos, ordenar,
abrir cuentas a cero, publicar consumos, fijar el cupo y la caducidad —visible en `paramsDigest` desde la
cabeza v5—, declarar una custodia de su clave que no se comprueba y cambiar el verificador sin rastro
(VERIFICACION.md, C7c, C7d; SECURITY.md). De las operaciones delegadas el registro guarda sólo el sello de
la autorización: un tercero puede recomputar el sello, pero no re-verificar la conservación en circuito ni
que dos custodios distintos autorizaran (VERIFICACION.md, C8). `reverificar()`, que comprueba los sellos,
existe en `crates/zk-ssl-verify/src/reverificacion.rs` y no tiene más llamantes que sus propios tests
(VERIFICACION.md, L5d).

### 5.12 Censura tras el RFC-0010

El RFC-0010 está ACEPTADO desde el §577. Toda operación que el nodo **evalúa** por `applySend` o
`applyClaim`, también si la capa la rechaza, reserva un número de recepción con `fsync`, anota
(`rx`, era, resumen de la prueba) antes de evaluarla y, al cerrarse su era, queda como hoja bajo la
`recepRoot` de una cabeza v6 firmada; la era es el índice XMSS de la última cabeza firmada más uno. Con el
recibo y su camino, que el nodo sirve por `zkssl_recepPath` al cerrarse la era, el titular forma un
**sobre de completitud** que se verifica sin el nodo, con cuatro salidas: resuelta como aplicada;
resuelta como rechazo con prueba; «declarada, no probada», para las causas sin prueba portable; o «NO
RESUELTA EN LA VENTANA» de N = 1.440 cabezas firmadas, un ROJO nombrado. Es **evidencia
oponible firmada por el propio operador, no una prueba criptográfica de ausencia** (RFC-0010 D-F a D-H;
spec/PAQUETE.md, 2.11; VERIFICACION.md, C12a, C12b).

Sigue sin rastro oponible (RFC-0010 D-E y D-H; README.md, «Qué garantiza y qué no»): la operación a la que
el nodo nunca dio recibo —no contesta, contesta sin recibo o falla al anotar—; todo lo que entra por
`zkssl_applyMany` o `zkssl_pledge`, fuera por decisión del §576; y los métodos que no reservan recibo, como
`zkssl_openAccount` o `zkssl_publishConsumo`. Tampoco lo deja un recibo cuyo camino el nodo no sirve y
que el titular no pidió a tiempo. El registro de recibos crece sin poda: `podar` existe y no tiene llamador
por decisión del §580, porque podar al vencer la ventana dejaría sin sobre a ese titular (RFC-0010,
«Seguridad»). **[REPRODUCIDO, 2026-09-30]**: el catálogo de completitud da 35/35.

### 5.13 El índice de firma y el registro de recepción: dos instancias de R(C,K)

Arqueo usa la **misma** `zk_ssl_guardian::Reconciliacion` (Coincide, ContadorAdelantado, ClaveEnCero,
ClaveAdelantada, equivalentes a los cuatro estados de §4.4) para dos recursos, con dos políticas
(`crates/zk-ssl-node/src/main.rs:592-691`; VERIFICACION.md, G1g, G6b).

| estado | **índice de firma** (clave derivada de semilla) | **registro de recepción** (modelo de clave persistida) |
|---|---|---|
| C y K | contador del guardián; índice leído de la clave | contador `rx`; mayor `rx` anotado en el registro (0 si vacío) |
| Coincide | arranca | arranca |
| ContadorAdelantado | arranca avisando (huérfanos) | arranca avisando: huecos declarados |
| ClaveEnCero | resincroniza la clave al contador y abandona 0..C−1, **salvo** que el diario muestre un índice mayor, en cuyo caso no arranca | arranca avisando, sin resincronizar: registro vacío o perdido |
| ClaveAdelantada | no arranca: clave comprometida | no arranca: el contador se restauró sin el registro |

Tres observaciones. (1) La clasificación es la misma y la política no: ante ClaveEnCero una instancia
resincroniza y la otra no mueve nada, porque «no hay clave que mover». Es un segundo uso de la misma
clasificación, con el mismo tipo y del mismo autor: indica que R(C,K) **se generaliza como clasificación**
sobre dos contadores monótonos, uno de los cuales debe ir detrás del otro, pero no es evidencia
independiente, ni la sostiene como garantía (VERIFICACION.md, G6d). (2) El significado depende del modelo: en el
registro, de clave persistida, ContadorAdelantado es lo normal tras una caída y ClaveEnCero nace de un
registro perdido, no de un reinicio. (3) El par no basta: restaurar viejo **sólo** el registro da
ContadorAdelantado, igual que un hueco por caída. Para una restauración conjunta, Arqueo añadió un dato
fuera del par: el latido no compone una cabeza si el contador de recepción está por debajo del `recepCount`
de la anterior, siempre que esa cabeza o el diario sobrevivan (`crates/zk-ssl-node/src/latido.rs:270-287`).
Para el índice de firma, ese dato es el diario del nodo, con los límites que declara
`doc/CONFIANZA_RESIDUAL.md` —restauración del directorio entero, diario borrado— y el que añade §8.1.

`tools/banco_reutilizacion.sh` mide desde el §579 que el testigo resincroniza al reiniciar, por la línea de
log «clave resincronizada»; su aserción de que ninguna cofirma repite índice compara el ordinal declarado,
no el índice WOTS embebido **[LECTURA]** (VERIFICACION.md, G1f). El fallo no es hipotético: antes de la
variante ClaveEnCero, el testigo corrido dos veces con la misma semilla y el mismo contador reutilizó los
índices WOTS 0 a 4, y un detector que miraba el ordinal no lo vio (`AUDITORIA.md` §331).

### 5.14 Componente ECST ↔ Arqueo

Las rutas de cada fila están en el Apéndice A.

| componente ECST | en Arqueo | estado |
|---|---|---|
| $C_{i-1}$, $C_i$ | raíz del árbol de cuentas en la entrada; las demás raíces, en la cabeza | existe |
| $O_i$ | `seq` y `tag` de `OpKind` | existe |
| $\Pi_i$ | STARK por transición, generado por el titular | existe; **no se conserva** |
| $\mathrm{Hp}(\Pi_i)$ | `digest_of_proof` | sólo Send, Claim, Burn, Refund |
| $A_i$ / $a_i$ | autoría dentro del STARK; par de custodios → `compromiso` y sello | se encadena el sello, no las pruebas |
| $H_i$ | `chain_digest_v2` | existe; sin etiqueta de dominio |
| Link | `TransitionLog::verify_chain` y `verify`; `zkssl_verifyChain` | sólo el nodo y la CLI |
| Verify | verificación nativa en `apply_*`; sobres del verificador | no re-verifica $\Pi_i$ históricas |
| cabeza autenticada | cabeza v6 firmada con XMSS^MT | existe; ancla TOFU |
| testigos | testigo de referencia con TOFU y cofirma | existe; ninguno independiente |
| R(C,K) | `GuardianIndice` + `Reconciliacion`, dos políticas | existe; dos instancias |
| uso único | índice XMSS y `rx` (por contador); árbol de consumos y hoja de pendiente (por conjunto) | existe |
| completitud | sobre de completitud | sólo `applySend`/`applyClaim` |
| vector de resultado | códigos 0 / 1 / 3 del verificador | parcial: por paquete, no por historia; el 3 es «declarada, no probada», no falta de datos |
| auditoría por niveles A/B/C | — | **no existe** como herramienta |
| codificación canónica versionada | registro de dominios; núcleo congelado con KAT | parcial |

---

## 6. Modelo de amenazas, garantías y no-garantías

### 6.1 Adversarios

- **Operador o red.** Ordena, retrasa o censura operaciones; puede mostrar vistas distintas a
  observadores distintos; ve todos los saldos; puede cambiar el verificador sin dejar rastro, porque el AIR
  es código y no datos (SECURITY.md; `doc/CONFIANZA_RESIDUAL.md`).
- **Ejecución y almacenamiento.** Mata el proceso en cualquier instante; restaura copias de seguridad;
  borra ficheros; coloca el contador en tmpfs; un disco confirma escrituras que no persistió; se corta la
  corriente.
- **Criptográfico.** Busca colisiones de $\mathrm{Hc}$ o $\mathrm{Hp}$; ataca la solidez del STARK
  (~127 bits conjeturados); aprovecha una hoja XMSS reutilizada; induce fallos durante la firma.
- **Lado cliente.** Roba la clave de gasto en la máquina del pagador, donde viven el probador y la clave;
  aprovecha canales laterales; fuerza una contraseña débil del almacén de claves del SDK, cuya derivación es
  SHA-256 y no una KDF de contraseñas (`crates/zk-ssl-sdk/src/keystore.rs:5-13`).

### 6.2 Garantías, con sus hipótesis y lo que las respalda

| # | garantía | hipótesis | respaldo |
|---|---|---|---|
| G1 | alterar un campo encadenado de $E_k$ cambia $H_n$ | Proposición 1: R1 y R2 (versión y era conocidas), R4, resistencia a colisiones de Rescue-Prime y de Blake3 reducido como función de los bytes; entre eras, la propiedad de tipo preimagen de §3.5 | `t1_chain_retroactivo` **[REPRODUCIDO]**; `AUDITORIA.md` §115 |
| G2 | una cabeza firmada procede de quien tiene la clave | seguridad de XMSS^MT; ninguna hoja reutilizada; custodia de la clave (declarada, sólo la modalidad fichero se comprueba); ancla de la clave (TOFU) | tests de `firma_cabeza.rs`; catálogo `paquete` **[REPRODUCIDO]** |
| G3 | quien custodia una cabeza anterior detecta una historia de cabezas reescrita | la cabeza anterior custodiada; misma clave. No cubre que la propia cabeza custodiada sea hoja del árbol nuevo, que el paquete de extensión no comprueba **[LECTURA]** (VERIFICACION.md, L4d), ni la continuidad del `chainDigest` (§5.8) | `zkssl_consistencyProof`; paquete de extensión en el catálogo `paquete` |
| G4 | una etiqueta se consume una vez en un libro; entre libros se detecta después | orden total de un nodo; dos cabezas firmadas | RFC-0006; catálogos `consumo` y `conflicto` **[REPRODUCIDO]** |
| G5 | una operación con recibo que no se resolvió en N cabezas deja un ROJO oponible | que el nodo emitiera recibo y sirva después su camino (`zkssl_recepPath`), o que el titular lo pidiera a tiempo; firma de la cabeza que cierra la era; y, mientras §8.2 no se resuelva, un ROJO por una prueba rechazada como inválida no distingue al operador que censura del titular que envió una prueba inválida | RFC-0010; catálogo `completitud` **[REPRODUCIDO]**; `tools/banco_completitud.sh` |
| G6 | ninguna firma de cabeza tiene índice mayor que el contador persistido | orden persistir-antes-de-firmar; `fsync` honrado; sin restauración, borrado ni puesta a cero del contador | banco K.1, 25/25, sobre el guardián aislado, antes de que existiera el firmante de cabezas **[MEDIDO: `AUDITORIA.md` §234]**; el orden reservar-firmar-verificar de `FirmanteCabeza::firmar`, por lectura y tests (VERIFICACION.md, G4c); tests del guardián **[REPRODUCIDO: `26 passed`]** |
| G7 | tras reiniciar, la clave de firma no reutiliza los índices indeterminados | la política de §3.8; el diario presente y no restaurado; **y, hasta el §594, el contador ni borrado ni puesto a cero (§8.1)** | para el nodo, `politica_de_reconciliacion` y sus tests unitarios **[LECTURA]**, sin banco que reinicie el nodo; para el testigo cofirmante, `politica_del_cofirmante` y `tools/banco_reutilizacion.sh` (§579), que mide la resincronización, compara el ordinal declarado y no la hoja WOTS, y cuyo negativo restaura el contador a 1, no a 0 (§5.13) |
| G8 | la clave de gasto no viaja por la API ni sale literal en la prueba | despliegue con el SDK; ocultación del fork de winterfell | ningún método RPC la recibe; suite de E2 del RFC-0009 (§538): medido, no demostrado |

### 6.3 No-garantías

- Que un tercero pueda validar la historia completa: las pruebas no se conservan y nadie recompone la cadena
  por él (§5.4, §5.7).
- Que un tercero verifique la conservación: le llegan el suministro y las raíces firmados; el agregado lo
  comprueba el operador (§5.7).
- Privacidad frente al operador, que lo ve todo.
- Completitud de lo que nunca recibió recibo, o entró por lote o prenda (§5.12).
- Prevención entre libros: sólo detección (§5.9).
- Identidad detrás de una clave, o independencia de testigos y custodios: dos claves comprometidas
  autorizan igual que dos voluntades.
- Durabilidad ante un corte de corriente; resistencia a un rollback del directorio entero del nodo, y
  —antes del §594, o sin diario— al borrado o la puesta a cero del contador (§8.1); resistencia a fallos inducidos en la firma XMSS^MT, que
  no se ha evaluado.

---

## 7. Evaluación

### 7.1 Medido

Todo lo de esta subsección es **[MEDIDO]** por el autor en un portátil bajo WSL2 salvo donde se indica,
con las fuentes confirmadas por el escéptico del bloque M. El modelo, un Intel Core i5-1135G7, consta en
§229 y §234; las series anteriores (§89, §130–§131, §204–§217) no documentan el hardware, y §181 menciona
un «Ryzen del piloto» (VERIFICACION.md, M1). Las cifras de tiempo dependen de la máquina, y el propio
repositorio no las ata con compuertas (`AUDITORIA.md` §304).

| qué | cifra | fuente y condiciones |
|---|---|---|
| `fsync` del guardián | ext4 0,907 ms (382× frente a no persistir); tmpfs 0,002 ms (1×) | §234, banco K.1, sin n ni dispersión publicados |
| muerte del proceso | 25 de 25 sin firma por delante; 13 de 25 con el contador adelantado, dentro del proceso | §234, sobre el guardián aún sin consumidor, antes del firmante de cabezas; el código del banco no está en el árbol |
| tamaño de una prueba de envío | 77.444–80.232 B | §538, 15 muestras por eje, con ocultación |
| tamaño de una prueba de cobro | 76.192–79.736 B | §538, ídem |
| un pago (dos pruebas), medido | 155.337–159.329 B | §538 |
| un pago, banda publicada **[DECLARADO]** | 145.953–167.967 B | §538: lo medido con un margen declarado del 5 %; `crates/zk-ssl/src/metrics.rs:82-83` |
| verificar una prueba de envío o cobro | 2,32–2,41 ms (media 2,35 ms, cinco ejecuciones); 2,43–2,49 ms | §89.1 y §204; **anteriores al §538** |
| resumen de prueba | Blake3 0,011 ms frente a 30,99 ms con Rescue, sobre la misma prueba | §204, §209 |
| generar una prueba | envío 322–353 ms, cobro 218–243 ms; σ intra-tanda ≈ 0,5 %, deriva entre tandas ≈ 9 % | §130–§131, dos tandas de cinco; **anteriores al §538** |
| coste implícito por merge Rescue (≈ una permutación), de `set_leaf` y del arranque | 7,44–8,91 µs | §217 (y 7,53 en §204): tiempos de operaciones de árbol divididos entre sus merges, con la sobrecarga incluida |
| techo del nodo por RPC | 248 op/s (recta 0,225 + 4,035·n ms) | §229: lotes `zkssl_applyMany` de 1, 4, 8 y 15 envíos, tres repeticiones; nodo en memoria, un cliente en serie, anterior al §538 y sin recibo de recepción |
| firma XMSS^MT de la cabeza | firmar 144,5–160,5 ms; verificar 2,4–2,7 ms; 18.469 B | `doc/xmss-evaluacion.md` y §236, dos fechas |
| envoltura en zkVM | 47,5 M ciclos; recibo sucinto de 223.234 B | §305–§307, pod con RTX 5090 cuyo anfitrión no se identifica; una prueba anterior al §538 |

Los tiempos de generación posteriores a la ocultación (envío 697,9–741,6 ms y cobro 696,9–715,6 ms, como
mínimos) constan sólo en un comentario de `metrics.rs` y en la D-AJ del RFC-0009, no en `AUDITORIA.md`; no
hay re-medida de la verificación posterior al §538 (VERIFICACION.md, M3a, M3b). Las cifras de 66.739 y
66.692 B que cita un comentario de `metrics.rs` son del §512, anteriores a la ocultación, y no son la cifra
vigente (VERIFICACION.md, M2b).

### 7.2 Reproducido en esta revisión

Entorno: microVM Firecracker, 4 vCPU «Intel(R) Xeon(R) Processor @ 2.10GHz», 15 GiB de RAM, raíz ext4 sobre
virtio y `/dev/shm` en tmpfs; rustc/cargo 1.94.1. El 2026-09-28 sobre Arqueo d531c80 y `hbs-state`
a960828 (kernel 6.18.44-fc-v37); el 2026-09-30 sobre Arqueo 71c5aad más el commit del borrador (kernel
6.18.44-fc-v50). Una ejecución por día: no es una medición con dispersión (`REPRODUCCION.md`).

- **Arqueo, 2026-09-30:** el filtro `t1_` de la capa da `3 passed, 0 failed`; `zk-ssl-guardian` da
  `26 passed, 0 failed`; `tools/conformidad.sh` con el binario del árbol (fea39a053efd7089) da paquete
  70/70, consumo 14/14, conflicto 16/16, rechazo 84/84, edad 11/11, pendiente 9/9, pago 9/9, prenda 9/9 y
  completitud 35/35: **257/257** entradas.
- **`simulate --amount 250000`, una ejecución cada día**, sandbox con claves deterministas (semilla
  0xa11ce), pruebas STARK reales. «KB» de la CLI significa bytes/1024.

| fecha, árbol | emisiones | envío | cobro | aplicar |
|---|---|---|---|---|
| 2026-09-28, d531c80 | 64,3 KiB [417 ms], 65,3 KiB [345 ms] | 77,6 KiB [970 ms] | 76,7 KiB [1.198 ms] | 4–6 ms |
| 2026-09-30, 71c5aad | 66,8 KiB [328 ms], 66,5 KiB [364 ms] | 78,9 KiB [1.333 ms] | 77,8 KiB [879 ms] | 5–6 ms |

Envío más cobro suman unos 158.000 y 160.460 bytes, **dentro** de la banda publicada; el envío del
2026-09-30, unos 80.800 bytes, queda por encima del máximo **medido** en el §538 y dentro de la banda. Con la
ocultación, el tamaño cambia de una ejecución a otra según la sal, y el lado caro se invierte entre corridas:
los tiempos absolutos no se transfieren entre máquinas ni entre ejecuciones. Las pruebas de emisión no
tienen banda publicada.

- **`hbs-state`, 2026-09-28:** `cargo test --release` verde; `verify-state.py` alcanza N3 (§4.6); la tabla
  del autocontrol de `fsync` de §4.8.

### 7.3 Derivado

- **Crecimiento del registro:** 169 B de valor por entrada de la era 2, más 12 B de clave en `sled` (`"log:"`
  y el `seq` en ocho bytes): **181 B** sin cifrar; con el libro cifrado (XChaCha20-Poly1305, 24 B de nonce y
  16 de etiqueta), **221 B**. Sin la sobrecarga interna de `sled`, que no se ha medido
  (`crates/zk-ssl/src/persistence.rs:839-841`; `crates/zk-ssl/src/crypto.rs`). A eso se suman 40 B por cada
  `applySend` o `applyClaim` evaluado en el registro de recepción
  (`crates/zk-ssl-node/src/registro_recepcion.rs:110`).
- **Coste del encadenado:** cinco merges Rescue por entrada en la era 1 y seis en la era 2, al coste implícito
  por merge de 7,44–8,91 µs (§217), dan unos 37–54 µs por entrada; frente a un `apply_send` de 3,11–3,33 ms
  dominado por la verificación STARK, es del orden del 1–2 %. Todo anterior al §538 (VERIFICACION.md, M4a).
- **Guardar la entrada en lugar de la prueba:** 169 B frente a 76.192–80.232 B por prueba es un ahorro del
  99,78–99,79 %. No «reduce» nada desplegado, porque el nodo nunca almacenó pruebas (VERIFICACION.md, M4d).
- **Ritmo de recomposición de la cadena:** unas 19.000–27.000 entradas por segundo y núcleo, estimadas desde
  el coste por merge; no medido (VERIFICACION.md, M4b).
- **Cota del banco K.1:** ≈ 11 % al 95 % para 0 fallos en 25 (§4.2).

### 7.4 No medido

Todo lo de esta subsección es **[NO MEDIDO]**:

- El coste de ECST frente a IVC sobre la misma carga, máquina y circuito. Lo único medido con plegado es una
  prueba de concepto con Nova cuyo paso es un solo hash, con curvas y un compromiso que exige ceremonia; no
  es comparable (FIVE_BACKENDS.md; VERIFICACION.md, M7b, M7c). Tampoco representa a la acumulación sin
  curvas, basada en hash [BMNW25], que es la alternativa que habría que medir contra ECST. La posible
  ventaja de ECST frente a IVC se presenta aquí sólo como argumento cualitativo, sin medir.
- El rendimiento de una auditoría histórica completa por un tercero: no existe la herramienta (§5.7).
- La durabilidad ante un corte de corriente, en los dos sistemas.
- El coste de los dos `fsync` adicionales por operación que añade el recibo de recepción —contador y
  registro, más el del directorio al abrir cada era— y el crecimiento del registro en disco a escala.
- La concurrencia del nodo con varios clientes.
- LMS y HSS contra HBS-STATE.
- El escenario de §8.1, con el contador borrado o puesto a cero: no se ha reproducido.
- Si `xmss` 0.1.0-pre.0 cachea las firmas de las capas intermedias de XMSS^MT, que es la contramedida que
  recoge VERIFICACION.md (P12a) frente a los fallos de tipo injerto, descritos primero contra el marco
  SPHINCS [Grafting18].

### 7.5 Cifras de los borradores que este informe retira

| cifra de los borradores | ficha | por qué se retira | lo que hay |
|---|---|---|---|
| «entorno estandarizado» AMD EPYC 7763, 64 núcleos, 128 GB, NVMe | M1 | ninguna medición del repositorio lo declara; los 64 núcleos son un supuesto de dimensionado | un portátil bajo WSL2 (i5-1135G7 donde consta; §7.1) |
| prueba $\Pi_i$ de ~62,4 KB | M2a | sin fuente; «~62 KB» es prosa anterior a la ocultación | envío 77.444–80.232 B; cobro 76.192–79.736 B (§538) |
| generar en 412 ms | M3a | sin fuente | §130–§131 y los mínimos posteriores al §538 (§7.1) |
| verificar en 8,1 ms | M3b | falsa; los 8 ms de las tablas son la verificación PLONK/KZG del circuito de comparación | 2,32–2,41 ms (§89.1) y 2,43–2,49 ms (§204), anteriores al §538 |
| encadenar en menos de 0,002 ms | M4a, L6d | falsa y sin fuente; 0,002 ms es el `fsync` en tmpfs | estimación de 37–54 µs (§7.3) |
| auditoría a ~125.000 bloques/s | M4b, L6c | sin fuente; Arqueo no tiene bloques | estimación de 19.000–27.000 entradas/s y núcleo, no medida |
| ~256 B por transición | M4c, L6a | falsa | 137 o 169 B por entrada (código) |
| reducción del 99,6 % | M4d, L6b | cociente de dos cifras sin fuente | 99,78–99,79 %, derivado |
| `fsync` ext4 1,82 ms / tmpfs 0,03 ms | M5a, H7c2, G3d | sin fuente | 0,907 ms / 0,002 ms (§234) |
| el test T1 con N = 100 épocas, alteración en k = 10 de un byte de $\Pi_k$ | L3a, L3b, L3c | falsa | N = 12 entradas, K = 5, pruebas sintéticas, sin STARK |

Se retiran también, sin ser cifras de medida, las autoevaluaciones numéricas, el umbral que «eleva
exponencialmente» la complejidad del ataque y la afirmación de que el guardián está «en producción desde
hace un año» (VERIFICACION.md, P18b, P13, G2a).

---

## 8. Limitaciones y hallazgos abiertos

### 8.1 HALLAZGO DE LECTURA, NO REPRODUCIDO: un contador de firma borrado o puesto a cero cae en `Coincide`

> **Estado (2026-09-30): CORREGIDO en el §594 (commit 78d71a4 de la rama de trabajo)**, después de la base del informe. Los
> dos gates —el del diario en el nodo (`<`) y el de las cofirmas en el testigo (`<=`)— se consultan ya en
> `Coincide` y `ContadorAdelantado`, no sólo en `ClaveEnCero`, con sus operadores y textos de siempre.
> **[MEDIDO en esta sesión, microVM]**: un test en `zk-ssl-guardian` mide la premisa —un contador borrado
> reabre en 0 y reconcilia `Coincide { indice: 0 }` con la clave en 0—; el nodo gana cuatro tests y el
> testigo tres, los positivos primero; con el gate nuevo desactivado fallan exactamente los cuatro rojos
> nuevos, y por la rama equivocada; y `tools/canon.sh --sello` sale VERDE con los pines movidos. **Lo que
> sigue sin hacerse**: reproducirlo con un nodo real o en un banco que borre el contador entre dos arranques;
> el mismo cambio en `hbs-state`, cuyo `open` tiene la misma premisa; y el asiento, que es del autor. Lo ya
> declarado sigue igual —sin diario, o con el directorio entero restaurado, no se ve—, y hay una consecuencia
> nueva, declarada en el código: una clave nueva con el diario de la vieja ya no arranca. El texto de abajo
> describe el árbol en 71c5aad y se conserva tal cual; sus números de línea son los de entonces.

**[LECTURA]** En `crates/zk-ssl-node/src/main.rs`, `politica_de_reconciliacion` (líneas 592-652) consulta el
diario —el segundo registro que detecta un contador restaurado hacia atrás— **sólo** en la rama
`ClaveEnCero`:

```rust
// extracto abreviado de crates/zk-ssl-node/src/main.rs, 592-652
zk_ssl_guardian::Reconciliacion::Coincide { indice } => {
    DecisionDeArranque::Arranca(format!("guardian y clave a la par en el indice {indice}"))
}
// ...
zk_ssl_guardian::Reconciliacion::ClaveEnCero { contador, indeterminados } => {
    match tope_diario {
        Some(d) if *contador < d => DecisionDeArranque::NoArranca(/* ... */),
```

Si se borra **sólo** el fichero del contador de firma, `GuardianIndice::abrir`
(`crates/zk-ssl-guardian/src/lib.rs:419-444`) lo recrea a 0 sin avisar; lo mismo ocurre si el fichero se
restaura con el valor 0, por ejemplo desde una copia tomada tras el primer arranque —`abrir` persiste el 0 al
crearlo— y antes de la primera firma. La clave, derivada de la semilla, también está en 0; la reconciliación
da `Coincide { indice: 0 }`, la política arranca sin mirar el diario y el firmante volvería a firmar con
índice declarado 1, es decir, gastando otra vez la hoja 0 y las siguientes: **reutilización de hojas XMSS**,
aunque el diario tenga anotado un índice mayor. El testigo cofirmante tiene la misma forma:
`politica_del_cofirmante` (`crates/zk-ssl-cli/src/witness.rs:2696-2771`) consulta las cofirmas sólo en
`ClaveEnCero`.

No figura entre los límites que declara `doc/CONFIANZA_RESIDUAL.md` —la restauración del directorio entero, el
diario borrado, el diario sin `fsync`, las líneas ilegibles— ni en la entrada 103 de `BACKLOG.md`, y ningún
test ni banco lo cubre: el test de `Coincide` pasa `None` como tope del diario, y el negativo de
`tools/banco_reutilizacion.sh`, que ejerce el testigo y no el nodo, restaura el contador a 1, no a 0. Como el
índice de firma es además el reloj de las eras de los recibos (`crates/zk-ssl-node/src/main.rs:1061-1063`), el
borrado o la puesta a cero haría retroceder también ese reloj **[LECTURA]**. **Se encontró leyendo, no se ha
reproducido**, y se comunica al autor. Arreglo sugerido: aplicar la comprobación del diario —en el testigo, la
de las cofirmas— **sea cual sea** el estado de la reconciliación, no sólo en `ClaveEnCero`, y añadir un banco
que borre el contador, o lo ponga a cero, entre dos arranques del nodo. Es la lección de §3.7: R sólo ve el
par, y el dato de fuera del par tiene que consultarse siempre (VERIFICACION.md, P15b).

### 8.2 Posible hueco en el sobre de completitud

**[LECTURA]** El verificador sólo admite como resolución «declarada, no probada» cuatro causas:
`CustodianSetExhausted`, `PendingTreeExhausted`, `NotTheIssuer` y `NotTheAccountHolder`
(`crates/zk-ssl-verify/src/main.rs:1622-1628`). No admite `ProofFailed` ni `VerificationFailed`, que el
RFC-0007 declaró sin prueba portable y para las que no hay sobre de rechazo. Si esta lectura es correcta, un
envío o cobro rechazado por prueba inválida no puede resolverse en VERDE ni en el cuarto estado, y su sobre
acaba en ROJO, en tensión con la D-G del RFC-0010; y un titular podría provocar un ROJO contra un operador
honrado enviando a propósito una prueba inválida, que sí consume recibo. Es una inferencia del escéptico del
bloque C, recogida en el material de trabajo (`doc/ecst/borrador/refutacion-custody.json`) y **sin vector
que la ejerza**.

### 8.3 Documentos vivos que no recogen el RFC-0010

El §581 puso al día la fila 6 del README y sus listas de lo que no se afirma, pero dejó sin salvedad, en los
resúmenes de cabecera, que el operador «puede omitir una operación sin dejar rastro»: README.md:34,
README_EN.md:35, QUESTIONS.md:30, PREGUNTAS.md:30, RESUMEN_EJECUTIVO.md:35 y RESUMEN_BILINGUE.md:28 y 81
(comprobado en 71c5aad). Contradicen la redacción del propio README, «Qué garantiza y qué no»: la censura de
lo que el nodo nunca acusa no deja rastro; la de lo que acusa, sí. Otras derivas verificadas: el docstring
de `politica_de_reconciliacion` (`crates/zk-ssl-node/src/main.rs:584-591`) aún dice que ContadorAdelantado
es «el caso NORMAL tras una caída», que con clave derivada de semilla no ocurre tras un reinicio (§4.3), y
que ClaveEnCero «SÍ para», al revés que el código, que resincroniza; y `spec/RPC.md:97`
describe `LogEntry` sin `compromiso` (§5.4).

### 8.4 Un comentario de rustdoc pegado a la variante equivocada

En `crates/zk-ssl-guardian/src/lib.rs:337-351`, el comentario de `ClaveAdelantada` («LO QUE NUNCA DEBE
PASAR … La clave debe considerarse comprometida») está escrito encima del de `ClaveEnCero`, así que rustdoc
atribuye los dos a `ClaveEnCero` y deja `ClaveAdelantada` sin documentar. Quien lea la documentación
generada puede concluir justo la regla falsa «KeyAtZero ⇒ no firmar». `hbs-state` tiene el orden correcto
(VERIFICACION.md, P15a).

### 8.5 Estado de verificación y otras limitaciones

- **Verificación formal.** No hay demostración mecanizada de la solidez del AIR, de FRI, de Rescue ni de la
  capa. FV-1, un censo sintáctico de celdas, es compuerta del canon en seis circuitos y no cubre las
  variantes v2; FV-2 fue un sondeo SMT sobre `circuit_refund` con Rescue abstraído; FV-3 (Lean, Coq o K) es
  horizonte declarado (`doc/VERIFICACION_FORMAL.md`; VERIFICACION.md, M6b). **No existe especificación
  formal del AIR**, que SECURITY.md §3.1 declara la carencia de mayor prioridad (VERIFICACION.md, M6c).
- **Ya dichas arriba:** sin auditoría externa, ni de Arqueo ni de su fork de winterfell 0.13.1 (§5.1-§5.2;
  VERIFICACION.md, M10a); `xmss` 0.1.0-pre.0, una *pre-release* clavada con `=` y sin KAT en el proyecto
  (§5.5; G10b); LMS/HSS sin probar contra HBS-STATE y ningún segundo sujeto independiente (§4.9-§4.10).
- **Sin ancla externa:** `doc/ANCLAJE_EXTERNO.md` es un marcador vacío; el diseño sólo está descrito en
  `AUDITORIA.md` §174 y en las filas B10.6/B10.7 de `doc/CONFIANZA_RESIDUAL.md` (VERIFICACION.md, L7c).
- **Los DOI de Zenodo** no se han podido comprobar (§Referencias), y **la reproducción** es de un entorno
  virtual, una ejecución por día, no hecha por el autor en su máquina (§7.2).

---

## 9. Trabajo futuro

- **Codificación canónica ECST** **[PROPUESTA]**, con dominio en el encadenado, versión explícita y el campo
  `compromiso` en la especificación del cable. Arqueo ya tiene una base: el registro de dominios, la tabla
  `REGISTRO` de `crates/zk-ssl-hash/src/lib.rs` que vigila `tools/check_dominios.py` desde el §286, y el
  núcleo congelado de spec/NUCLEO.md con sus KAT (RFC-0005, PROPUESTO).
- **Varias anclas** para la clave de la cabeza, más allá de TOFU, y testigos que no sean el autor.
- **Disponibilidad de la evidencia:** decidir qué pruebas se retienen, quién y cuánto tiempo, para que el
  nivel B del protocolo deje de ser Indeterminado.
- **Verificación mecanizada de R(C,K)** y del protocolo persistir-antes-de-firmar, en TLA+ o con un resolutor
  SMT, incluidos el borrado y la puesta a cero del contador de §8.1. Ninguno de los dos repositorios lo tiene
  (VERIFICACION.md, M6a).
- **Un segundo sujeto independiente** para HBS-STATE, y **vectores LMS**.
- **Agenda ECST-R** **[PROPUESTA, sin implementar en ningún repositorio]** (VERIFICACION.md, C10c), con las
  correcciones de la verificación: el conocimiento cero de una prueba no implica confidencialidad de la
  memoria de quien la genera, y un canal lateral en la máquina del pagador rompe la autoría con pruebas
  válidas (P12c); un umbral k-de-n eleva de 1 a k las claves que hay que comprometer, y sólo con compromisos
  independientes la probabilidad cae como p^k, sin nada «exponencial» (P13); un entorno de ejecución confiable
  traslada la confianza al fabricante y a la atestación, y su propio estado sufre el problema de [Memoir11;
  ROTE17]; un fallo inducido que haga aceptar una transición inválida sólo se detecta después, y sólo si
  $\Pi_i$ se publica y un tercero la re-verifica, y en la firma XMSS^MT un fallo en una capa intermedia puede
  dar una firma que verifica y exponer una segunda firma de un solo uso (P12a; los fallos de tipo injerto se
  describieron contra el marco SPHINCS en [Grafting18]); una máquina de estados del compromiso de claves
  (sana, sospechosa, en cuarentena, revocada, migrada) no existe, y la rotación de claves sigue abierta. **Los
  compromisos de Pedersen quedan excluidos**: no son post-cuánticos y contradicen CONTRIBUTING.md (P14, C10a).
- **Normalización.** Si HBS-STATE se lleva al IETF, su destino natural es el grupo de trabajo PQUIP, que
  produjo RFC 10033; no CFRG ni LAMPS. No consta que el Internet-Draft se haya enviado. Lo que se dice del
  proceso —la carta de PQUIP abarca la orientación operacional sin mecanismos criptográficos nuevos; en el
  flujo del IRTF, un borrador adoptado por el CFRG se llamaría `draft-irtf-cfrg-*`, lo aprobaría el IRSG y el
  IESG sólo haría la revisión de conflictos— descansa en la guía del CFRG, copiada del wiki del IETF, y en
  fragmentos de resultados de búsqueda sobre la carta de PQUIP, RFC 5742 y RFC 5743, no en la lectura de esos
  textos (VERIFICACION.md, P2, P3, P5c, P5d). Las demás correcciones al borrador del I-D están en el bloque P
  de `VERIFICACION.md` (P2 a P8d).

---

## 10. Conclusión

Ninguna pieza de ECST es nueva, y este informe no pretende otra cosa. Lo que ofrece es una forma de no
confundirlas: el encadenado por hash compromete la historia pero no la valida; una prueba válida no
demuestra que las reglas estén completas; nada de eso establece una historia única; y la muerte de un
proceso tiene que acabar en un estado con nombre. Las dos instanciaciones muestran las dos caras. HBS-STATE
aporta un contrato ejecutable para la reconciliación y nada más. Arqueo instancia la mayoría de los
componentes y declara lo que le falta: pruebas que no se conservan, un ancla que es TOFU, ningún tercero
que recomponga la cadena y ninguna herramienta de auditoría. Y su registro de recepción muestra que la
misma R(C,K) sirve para un segundo recurso con otra política. Al contrastar el modelo con el código
apareció además, por lectura y sin reproducir todavía, un caso que la política de Arqueo no parece ver
(§8.1); que R no pueda verlo se sigue de su definición.

> **Compromiso ≠ validez ≠ conformidad semántica ≠ consenso: cada uno se comprueba por separado, y lo que no
> se ha medido se dice.**

---

## Declaración de uso de IA generativa

Este informe se redactó con la asistencia de un asistente de IA generativa (Claude, de Anthropic, en una
sesión de Claude Code), a partir de borradores también generados con IA que el autor aportó. La
verificación la hicieron agentes del mismo asistente —un verificador y un escéptico por bloque—; dos
revisiones adversariales del primer borrador, también de agentes, se aplicaron después
(`doc/ecst/borrador/revision-fidelidad.json` y `revision-rigor.json`); y la reproducción de §7.2 se
ejecutó en un contenedor efímero, no en la máquina del autor. Conforme a GENAI.md, el método del proyecto
es que se mide primero, el autor decide y sólo entra lo que el autor ejecuta y acepta. **Este borrador no
ha pasado todavía por esa aceptación.** El autor es Ángel José Toranzo Portela, y es el único; el
asistente no figura como autor ni como coautor.

Qué se cotejó con fuentes y qué no. Toda cifra y toda afirmación sobre el código o el estado de los dos
repositorios remite a un fichero de los árboles o a una ficha de `VERIFICACION.md`. De la literatura se
verificaron los metadatos de cada entrada de `referencias.bib`, con la fuente anotada en el propio
fichero; el texto se cotejó en RFC 10033, SP 800-208, EIP-8310 y la especificación HBS-STATE v0.3, de los
que hubo copia. Las caracterizaciones de las demás obras de §2 descansan en sus títulos y en resúmenes o
fragmentos de resultados de búsqueda, no en la lectura del texto completo. Lo que se dice del proceso del
IETF y del IRTF (§9) descansa en la guía del CFRG y en fragmentos de búsqueda. El esbozo de la
Proposición 1 y los argumentos de §3 son razonamiento del propio texto, no citas, y nadie los ha revisado
fuera de este proceso.

## Disponibilidad

- **Este informe:** `doc/ecst/ECST.md`; su primer borrador, sin revisar, está en el commit a7c72b9, y las
  dos revisiones adversariales que se le aplicaron, en 6a868ed.
- **Registro de verificación:** `doc/ecst/VERIFICACION.md`; bibliografía en BibTeX, con los metadatos
  verificados y la fuente de cada entrada: `doc/ecst/referencias.bib`.
- **Material intermedio:** `doc/ecst/borrador/` —los veredictos por bloque (`verificacion-*.json`,
  `refutacion-*.json`, `final.json`), `REPRODUCCION.md`, los dos registros de `simulate`, la réplica de la
  sonda de `fsync` (`sonda-fsync.rs.txt`) y las dos revisiones adversariales (`revision-*.json`)—. Es
  material de trabajo: este informe lo cita sólo para localizar la reproducción, la procedencia de una
  inferencia de §8.2 y las revisiones que se le aplicaron.
- **Árboles:** Arqueo, `main` en 71c5aad (S582), con el material de verificación en los commits 71710b2 a
  70f6370, el primer borrador de este informe en a7c72b9 y sus revisiones en 6a868ed, que sólo añaden
  ficheros bajo `doc/ecst/`
  (https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers);
  `hbs-state` en a960828, etiqueta v0.2.0 (https://github.com/atoranzo/hbs-state).

---

## Referencias

Sólo entradas de `doc/ecst/referencias.bib`, cuyos metadatos se verificaron el 2026-09-30 con las fuentes
que anota cada una, más los DOI de Zenodo de los propios proyectos, que se marcan. Cada etiqueta del texto
remite a la clave BibTeX que se indica al final de su entrada. El artículo del blog de QRL de §4.11 no
tiene entrada: se cita a través de `AUDITORIA.md` §288, porque sus metadatos no se pudieron verificar.

- **[AlBassam21]** M. Al-Bassam, A. Sonnino, V. Buterin, I. Khoffi. «Fraud and Data Availability Proofs:
  Detecting Invalid Blocks in Light Clients». FC 2021, Part II, LNCS 12675, Springer, 2021, pp. 279–298. DOI
  10.1007/978-3-662-64331-0_15. BibTeX: `FC:ASBK21`.
- **[Ariadne16]** R. Strackx, F. Piessens. «Ariadne: A Minimal Approach to State Continuity». 25th USENIX
  Security Symposium, 2016, pp. 875–892. BibTeX: `USENIX:StrPie16`.
- **[BBHR19]** E. Ben-Sasson, I. Bentov, Y. Horesh, M. Riabzev. «Scalable Zero Knowledge with No Trusted
  Setup». CRYPTO 2019, Part III, LNCS 11694, Springer, 2019, pp. 701–732. DOI 10.1007/978-3-030-26954-8_23.
  BibTeX: `C:BBHR19`.
- **[BCCT13]** N. Bitansky, R. Canetti, A. Chiesa, E. Tromer. «Recursive Composition and Bootstrapping for
  SNARKs and Proof-Carrying Data». STOC 2013, ACM, pp. 111–120. DOI 10.1145/2488608.2488623. BibTeX:
  `STOC:BCCT13`.
- **[BH17]** L. Groot Bruinderink, A. Hülsing. «“Oops, I Did It Again” – Security of One-Time Signatures Under
  Two-Message Attacks». SAC 2017, LNCS 10719, Springer, 2018, pp. 299–322. DOI 10.1007/978-3-319-72565-9_15.
  ePrint 2016/1042. BibTeX: `SAC:BruHul17`.
- **[BMNW25]** B. Bünz, P. Mishra, W. Nguyen, W. Wang. «Accumulation Without Homomorphism». ITCS 2025, LIPIcs
  325, 2025, pp. 23:1–23:25. DOI 10.4230/LIPIcs.ITCS.2025.23. ePrint 2024/474. BibTeX: `ITCS:BMNW25`.
- **[Chidambaram13]** V. Chidambaram, T. S. Pillai, A. C. Arpaci-Dusseau, R. H. Arpaci-Dusseau. «Optimistic
  Crash Consistency». SOSP ’13, ACM, 2013, pp. 228–243. DOI 10.1145/2517349.2522726. BibTeX:
  `ChidambaramPAA13`.
- **[ChiesaTromer10]** A. Chiesa, E. Tromer. «Proof-Carrying Data and Hearsay Arguments from Signature Cards».
  Innovations in Computer Science (ICS 2010), Tsinghua University Press, 2010, pp. 310–331. BibTeX:
  `ITCS:ChiTro10`.
- **[CONIKS15]** M. S. Melara, A. Blankstein, J. Bonneau, E. W. Felten, M. J. Freedman. «CONIKS: Bringing Key
  Transparency to End Users». 24th USENIX Security Symposium, 2015, pp. 383–398. BibTeX: `USENIX:MBBFF15`.
- **[CW09]** S. A. Crosby, D. S. Wallach. «Efficient Data Structures for Tamper-Evident Logging». 18th USENIX
  Security Symposium, 2009, pp. 317–334. BibTeX: `USENIX:CroWal09`.
- **[DGHS16]** B. Dowling, F. Günther, U. Herath, D. Stebila. «Secure Logging Schemes and Certificate
  Transparency». ESORICS 2016, Part II, LNCS 9879, Springer, 2016, pp. 140–158. DOI
  10.1007/978-3-319-45741-3_8. ePrint 2016/452. BibTeX: `ESORICS:DGHS16`.
- **[EIP8310]** A. Shukla, B. Wagner, G. Singh, G. Ballet, J. Drake, K. Moroz Liebl, P. Ramanujam, S. Naiyer,
  T. Coratger, U. Leepaisalsuwanna. «EIP-8310: Post-Quantum Keystore for Stateful Keys». Ethereum Improvement
  Proposals, borrador, creado el 2026-06-19. BibTeX: `eip8310`.
- **[Elnozahy02]** E. N. Elnozahy, L. Alvisi, Y.-M. Wang, D. B. Johnson. «A Survey of Rollback-Recovery
  Protocols in Message-Passing Systems». ACM Computing Surveys 34(3), 2002, pp. 375–408. DOI
  10.1145/568522.568525. BibTeX: `ElnozahyAWJ02`.
- **[ETSI21]** ETSI. «CYBER; State management for stateful authentication mechanisms». ETSI TR 103 692 V1.1.1,
  noviembre de 2021. BibTeX: `etsi-tr-103-692`.
- **[FIPS205]** NIST. «Stateless Hash-Based Digital Signature Standard». FIPS 205, agosto de 2024. DOI
  10.6028/NIST.FIPS.205. BibTeX: `fips205`.
- **[Fluhrer23]** S. Fluhrer. «Oops, I did it again revisited: another look at reusing one-time signatures».
  Cryptology ePrint Archive, Paper 2023/1905, 2023. BibTeX: `eprint-2023-1905`.
- **[Grafting18]** L. Castelnovi, A. Martinelli, T. Prest. «Grafting Trees: A Fault Attack Against the SPHINCS
  Framework». PQCrypto 2018, LNCS 10786, Springer, 2018, pp. 165–184. DOI 10.1007/978-3-319-79063-3_8. BibTeX:
  `PQCRYPTO:CasMarPre18`.
- **[HS91]** S. Haber, W. S. Stornetta. «How to Time-Stamp a Digital Document». Journal of Cryptology 3(2),
  1991, pp. 99–111. DOI 10.1007/BF00196791. BibTeX: `JC:HabSto91`.
- **[HyperNova24]** A. Kothapalli, S. T. V. Setty. «HyperNova: Recursive Arguments for Customizable Constraint
  Systems». CRYPTO 2024, Part X, LNCS 14929, Springer, 2024, pp. 345–379. DOI 10.1007/978-3-031-68403-6_11.
  BibTeX: `C:KotSet24`.
- **[Laurie14]** B. Laurie. «Certificate Transparency». Communications of the ACM 57(10), 2014, pp. 40–46. DOI
  10.1145/2659897. BibTeX: `Laurie14`.
- **[LedgerDB20]** X. Yang, Y. Zhang, S. Wang, B. Yu, F. Li, Y. Li, W. Yan. «LedgerDB: A Centralized Ledger
  Database for Universal Audit and Verification». PVLDB 13(12), 2020, pp. 3138–3151. DOI
  10.14778/3415478.3415540. BibTeX: `YangZWYLLY20`.
- **[McGrew16]** D. McGrew, P. Kampanakis, S. Fluhrer, S.-L. Gazdag, D. Butin, J. Buchmann. «State Management
  for Hash-Based Signatures». SSR 2016, LNCS 10074, Springer, 2016, pp. 244–260. DOI
  10.1007/978-3-319-49100-4_11. BibTeX: `McGrewKFGBB16`.
- **[Memoir11]** B. Parno, J. R. Lorch, J. R. Douceur, J. W. Mickens, J. M. McCune. «Memoir: Practical State
  Continuity for Protected Modules». IEEE S&P 2011, pp. 379–394. DOI 10.1109/SP.2011.38. BibTeX: `SP:PLDMM11`.
- **[Merkle89]** R. C. Merkle. «A Certified Digital Signature». CRYPTO ’89, LNCS 435, Springer, 1990, pp.
  218–238. DOI 10.1007/0-387-34805-0_21. BibTeX: `C:Merkle89a`.
- **[MS02]** D. Mazières, D. Shasha. «Building Secure File Systems out of Byzantine Storage». PODC 2002, ACM,
  pp. 108–117. DOI 10.1145/571825.571840. BibTeX: `MazieresS02`.
- **[Necula97]** G. C. Necula. «Proof-Carrying Code». POPL ’97, ACM, 1997, pp. 106–119. DOI
  10.1145/263699.263712. BibTeX: `Necula97`.
- **[Nimble23]** S. Angel, A. Basu, W. Cui, T. Jaeger, S. Lau, S. Setty, S. Singanamalla. «Nimble: Rollback
  Protection for Confidential Cloud Services». OSDI 23, USENIX, 2023, pp. 193–208. ePrint 2023/761. BibTeX:
  `AngelBCJLSS23`.
- **[Nova22]** A. Kothapalli, S. Setty, I. Tzialla. «Nova: Recursive Zero-Knowledge Arguments from Folding
  Schemes». CRYPTO 2022, Part IV, LNCS 13510, Springer, 2022, pp. 359–388. DOI 10.1007/978-3-031-15985-5_13.
  BibTeX: `C:KotSetTzi22`.
- **[PeerReview07]** A. Haeberlen, P. Kouznetsov, P. Druschel. «PeerReview: Practical Accountability for
  Distributed Systems». SOSP ’07, ACM, 2007, pp. 175–188. DOI 10.1145/1294261.1294279. BibTeX:
  `HaeberlenKD07`.
- **[Pillai14]** T. S. Pillai, V. Chidambaram, R. Alagappan, S. Al-Kiswany, A. C. Arpaci-Dusseau, R. H.
  Arpaci-Dusseau. «All File Systems Are Not Created Equal: On the Complexity of Crafting Crash-Consistent
  Applications». OSDI ’14, USENIX, 2014, pp. 433–448. BibTeX: `PillaiCAAAA14`.
- **[Piperine20]** J. Lee, K. Nikitin, S. T. V. Setty. «Replicated state machines without replicated
  execution». IEEE S&P 2020. DOI 10.1109/SP40000.2020.00068. ePrint 2020/195. BibTeX: `SP:LeeNikSet20`.
- **[POSIX24]** IEEE y The Open Group. IEEE Std 1003.1-2024 (POSIX.1-2024, Issue 8), System Interfaces,
  `fsync()`. Junio de 2024. BibTeX: `posix2024`.
- **[RFC6962]** B. Laurie, A. Langley, E. Kasper. «Certificate Transparency». RFC 6962, junio de 2013. DOI
  10.17487/RFC6962. Experimental; obsoleto por RFC 9162. BibTeX: `rfc6962`.
- **[RFC8391]** A. Huelsing, D. Butin, S. Gazdag, J. Rijneveld, A. Mohaisen. «XMSS: eXtended Merkle Signature
  Scheme». RFC 8391, mayo de 2018. DOI 10.17487/RFC8391. IRTF (CFRG), Informational. BibTeX: `rfc8391`.
- **[RFC8554]** D. McGrew, M. Curcio, S. Fluhrer. «Leighton-Micali Hash-Based Signatures». RFC 8554, abril de
  2019. DOI 10.17487/RFC8554. IRTF (CFRG), Informational. BibTeX: `rfc8554`.
- **[RFC9162]** B. Laurie, E. Messeri, R. Stradling. «Certificate Transparency Version 2.0». RFC 9162,
  diciembre de 2021. DOI 10.17487/RFC9162. Experimental. BibTeX: `rfc9162`.
- **[RFC10033]** T. Wiggers, K. Bashiri, S. Kölbl, J. Goodman, S. Kousidis. «Hash-Based Signatures: State and
  Backup Management». RFC 10033, septiembre de 2026. DOI 10.17487/RFC10033. IETF (PQUIP), Informational.
  BibTeX: `rfc10033`.
- **[Rogaway06]** P. Rogaway. «Formalizing Human Ignorance». VIETCRYPT 2006, LNCS 4341, Springer, 2006. DOI
  10.1007/11958239_14. ePrint 2006/281, «… Collision-Resistant Hashing without the Keys». BibTeX:
  `VIETCRYPT:Rogaway06`.
- **[ROTE17]** S. Matetic, M. Ahmed, K. Kostiainen, A. Dhar, D. Sommer, A. Gervais, A. Juels, S. Capkun.
  «ROTE: Rollback Protection for Trusted Execution». 26th USENIX Security Symposium, 2017, pp. 1289–1306.
  BibTeX: `USENIX:MAKDSG17`.
- **[SK99]** B. Schneier, J. Kelsey. «Secure Audit Logs to Support Computer Forensics». ACM Transactions on
  Information and System Security 2(2), 1999, pp. 159–176. DOI 10.1145/317087.317089. BibTeX: `SchneierK99`.
- **[SP800-208]** D. A. Cooper, D. C. Apon, Q. H. Dang, M. S. Davidson, M. J. Dworkin, C. A. Miller.
  «Recommendation for Stateful Hash-Based Signature Schemes». NIST SP 800-208, octubre de 2020. DOI
  10.6028/NIST.SP.800-208. BibTeX: `nist-sp800-208`.
- **[STARK18]** E. Ben-Sasson, I. Bentov, Y. Horesh, M. Riabzev. «Scalable, transparent, and post-quantum
  secure computational integrity». Cryptology ePrint Archive, Paper 2018/046, 2018. BibTeX: `eprint-2018-046`.
- **[SUNDR04]** J. Li, M. N. Krohn, D. Mazières, D. Shasha. «Secure Untrusted Data Repository (SUNDR)». OSDI
  2004, USENIX, pp. 121–136. BibTeX: `LiKMS04`.
- **[Valiant08]** P. Valiant. «Incrementally Verifiable Computation or Proofs of Knowledge Imply Time/Space
  Efficiency». TCC 2008, LNCS 4948, Springer, 2008, pp. 1–18. DOI 10.1007/978-3-540-78524-8_1. BibTeX:
  `TCC:Valiant08`.
- **[VSM20]** S. Setty, S. Angel, J. Lee. «Verifiable state machines: Proofs that untrusted services operate
  correctly». Cryptology ePrint Archive, Report 2020/758, 2020. BibTeX: `EPRINT:SetAngLee20`.
- **[Zheng13]** M. Zheng, J. Tucek, F. Qin, M. Lillibridge. «Understanding the Robustness of SSDs under Power
  Fault». FAST 13, USENIX, 2013, pp. 271–284. BibTeX: `ZhengTQL13`.
- **[zkLedger18]** N. Narula, W. Vasquez, M. Virza. «zkLedger: Privacy-Preserving Auditing for Distributed
  Ledgers». NSDI 18, USENIX, 2018, pp. 65–80. ePrint 2018/241. BibTeX: `NarulaVV18`.

**Depósitos de los propios proyectos** (DOI declarados por el autor; **no comprobados desde el entorno en
que se escribió este informe**, porque Zenodo no era accesible):

- **Arqueo:** cita preferida, 10.5281/zenodo.21736125 (CITATION.cff); los seis depósitos se listan en
  README.md, «Publicación», y preceden a correcciones del árbol.
- **`hbs-state`:** 10.5281/zenodo.22980547, que `CITATION.cff` declara y el repositorio llama DOI de concepto.
  El 10.5281/zenodo.22993572 que citaban los borradores podría ser el DOI de la versión 0.2.0; no se ha
  verificado y no se cita (VERIFICACION.md, H13a, B18a, B18b).

---

## Apéndice A — Correspondencia ECST ↔ código

| elemento del modelo | Arqueo | `hbs-state` |
|---|---|---|
| $C_{i-1}$, $C_i$, $O_i$ | raíces y `seq`/`tag` de la entrada, `crates/zk-ssl/src/log.rs` | — |
| $\Pi_i$ | STARK por transición, `crates/zk-ssl/src/client.rs`; circuitos en `crates/stark-experiment/` | — |
| $A_i$ / $a_i$ | `crates/zk-ssl/src/mint.rs`, `recovery.rs`, `freeze.rs`, `governance.rs` | — |
| encadenado $\mathrm{Hc}$ | `native_merge` (Rescue-Prime Rp64_256, Goldilocks), `crates/zk-ssl-hash/src/lib.rs` | — |
| resumen de evidencia $\mathrm{Hp}$ | `digest_of_proof`, `crates/zk-ssl-hash/src/lib.rs:1290-1320` | — |
| $H_i$ | `chain_digest`, `chain_digest_v2`, `crates/zk-ssl/src/log.rs:250-294` | — |
| serialización de la entrada | `log_entry_to_bytes`, 137/169 B, `crates/zk-ssl/src/store.rs:411-455` | — |
| Link | `TransitionLog::verify_chain` y `verify`, `crates/zk-ssl/src/log.rs:393`, `:438`; `zkssl_verifyChain` | — |
| Verify | verificación nativa en `apply_*`, `crates/zk-ssl/src/two_phase.rs`; sobres de `crates/zk-ssl-verify/` | — |
| test del atado retroactivo | `t1_chain_retroactivo`, `crates/zk-ssl/src/log.rs:1189-1295` | — |
| cabeza autenticada | `EpochHead`, `crates/zk-ssl/src/log.rs:586-649`; `epoch_digest_v6`, `crates/zk-ssl-hash/src/lib.rs:469`; `FirmanteCabeza::firmar`, `crates/zk-ssl-node/src/firma_cabeza.rs:179-203` | — |
| testigos | testigo de referencia, `crates/zk-ssl-cli/src/witness.rs` | — |
| árbol de historia y extensión | `crates/zk-ssl-verify/src/mmr.rs` | — |
| verificador sin el nodo | `crates/zk-ssl-verify/src/main.rs` (códigos 0 / 1 / 2 / 3) | `spec/verify-state.py` (conformidad, no evidencia) |
| guardián: reservar antes de usar | `GuardianIndice::reservar`, `crates/zk-ssl-guardian/src/lib.rs` | `IndexGuard::reserve`, `src/lib.rs:482-489` |
| abrir el contador | `GuardianIndice::abrir`, `crates/zk-ssl-guardian/src/lib.rs:419-444` | `IndexGuard::open`, `src/lib.rs:448-473` |
| autocontrol de `fsync` | `comprobar_persistencia`, `crates/zk-ssl-guardian/src/lib.rs:510-543` | `check_persistence`, `src/lib.rs:523-556` |
| R(C,K) | `Reconciliacion`, `crates/zk-ssl-guardian/src/lib.rs:322-353` | `reconcile_values`, `src/lib.rs:412-431` |
| juez | `no_admite_matiz`, `crates/zk-ssl-guardian/src/lib.rs:369-376` | `is_fatal`, `src/lib.rs:370-377` |
| política, índice de firma | `politica_de_reconciliacion`, `crates/zk-ssl-node/src/main.rs:592-652`; testigo: `politica_del_cofirmante`, `crates/zk-ssl-cli/src/witness.rs:2696` | del dueño (spec §9) |
| política, registro de recepción | `politica_del_registro`, `crates/zk-ssl-node/src/main.rs:662-691`; `recepcion.rs`, `registro_recepcion.rs` | — |
| segundo registro fuera del par | diario del nodo, `crates/zk-ssl-node/src/diario.rs`; cofirmas del testigo | — |
| uso único por conjunto | `crates/zk-ssl/src/consumo.rs` (RFC-0006); hoja de pendiente | — |
| uso único por contador | índice XMSS (guardián); `rx`, `crates/zk-ssl-node/src/registro_recepcion.rs` | índice OTS (`IndexGuard`) |
| completitud | sobre de completitud, `crates/zk-ssl-verify/src/main.rs`; `spec/rfc/0010-el-recibo-de-recepcion.md` | — |
| codificación canónica | tabla `REGISTRO` de `crates/zk-ssl-hash/src/lib.rs`; `tools/check_dominios.py`; spec/NUCLEO.md | — |
| vectores | catálogos de `spec/vectors/` y `tools/conformidad.sh` | `spec/state-vectors-v0.3.json` |

## Apéndice B — Reproducción

Las órdenes, tal como las registra `doc/ecst/borrador/REPRODUCCION.md` (en la raíz de cada árbol):

```sh
# Arqueo, en 71c5aad
cargo test --release -p zk-ssl --lib t1_
cargo test --release -p zk-ssl-guardian
cargo build --release -p zk-ssl-verify
for c in paquete consumo conflicto rechazo edad pendiente pago prenda completitud; do
  bash tools/conformidad.sh target/release/zk-ssl-verify spec/vectors/$c/MANIFIESTO.txt
done
cargo run --release -p zk-ssl-cli -- simulate --amount 250000

# hbs-state, en a960828
cargo test --release
cargo build --release
python3 spec/verify-state.py --vectors spec/state-vectors-v0.3.json \
    --subject ./target/release/hbs-state-subject
```

`REPRODUCCION.md` escribe el manifiesto como `[MANIFIESTO]`; aquí se sustituye por el de cada catálogo,
`spec/vectors/<catálogo>/MANIFIESTO.txt`, que es como lo invoca `tools/canon.sh`. La tabla del
autocontrol de `fsync` se obtuvo con la réplica `doc/ecst/borrador/sonda-fsync.rs.txt`, compilada aparte y
ejecutada 30 veces por ruta y tanda sobre la raíz ext4 y sobre `/dev/shm`.
