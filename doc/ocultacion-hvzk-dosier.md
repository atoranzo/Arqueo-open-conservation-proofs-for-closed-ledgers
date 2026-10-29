# El dosier para la revisión ajena del argumento de ocultación (§903)

**Estado**: la 136 de `BACKLOG.md`. Lo escribió la sesión en el §903 para un revisor que no sea de
la casa, y no es una promesa: lo que la casa promete de sus pruebas sigue siendo el censo
—«una prueba que verifica demuestra la transición que enuncia y no publica literal ningún valor de
columna constante de su traza», RFC-0009 D-A y `SECURITY.md` §3.bis—, y lo que se podría prometer
está escrito en la D-AM del RFC-0009 y espera esta revisión. El revisor no decide qué se promete:
dice si el argumento se sostiene, dónde no, y qué no miden las medidas. Decide el autor, después,
con un asiento.

## 1. Qué se pide revisar, en una frase

Que, para dieciséis de las veintitrés familias de pruebas con fila en la tabla de D-B del RFC-0009
—las seis de T = 1024, las nueve de T = 512 y la solvencia—, con la construcción de `zkssl/0.4` tal
como está —T filas aleatorias por columna, una columna aleatoria, el cociente cegado con m = 64 y
las hojas saladas—, la vista del verificador honesto se simula sin el testigo: para toda traza real
que la AIR acepta, la vista y la salida del simulador del apartado 5 de `doc/ocultacion-hvzk.md`
son estadísticamente indistinguibles en el modelo de oráculo aleatorio programable, bajo las
hipótesis de su apartado 4; y que para las otras siete —MerkleAir, FrozenClimbAir y las cinco de
T = 64— no está establecido, por lo que el apartado 14 dice de cada una.

⚠️ **§915:** desde el corte `zkssl/0.5` (§914) lo que se pide revisar es la proposición del apartado 18 de `doc/ocultacion-hvzk.md`, para las veintitrés familias con la construcción de producción —T ≥ 128, dos columnas aleatorias pareadas en DEEP como una sobre E, m = 64 y las hojas saladas—, con L9 (apartado 17) en el lugar de R3 medida y sin M_lin ni la cuenta fina; lo de arriba describe `zkssl/0.4`, que sigue siendo lo que era para los vectores conservados bajo `spec/vectors/0.4/`. El orden de lectura del apartado 2 vale, con los apartados 17 y 18 al final; las cifras del apartado 4 se reproducen igual, con `K_ENTERO_PAREADAS` y su fila de T = 128.

## 2. Qué leer, y en qué orden

1. **Lo decidido y lo no decidido**: `spec/rfc/0009-lo-que-revela-una-prueba.md`, la D-A (la
   promesa, revertida en el §538), la D-I (la sal; con sus notas §896 a §902), la D-AL (la máscara
   de DEEP en E, la 133) y la D-AM (lo que se podría prometer y lo que le falta). Media hora.
2. **El argumento**: `doc/ocultacion-hvzk.md`, entero y en su orden: el protocolo tal como está
   implementado (1), la vista del verificador (2), los lemas L0 a L6 (3), las hipótesis con su clase
   (4), el simulador paso a paso (5), la condición R3 como instrumento (6), lo que no cubre (7), el
   enunciado (8), lo medido en A1 y lo corregido (9), dónde el fork se aparta de la nota 2024/1037
   (10), las objeciones de las dos revisiones de la sesión con su cierre (11), lo que pide a A3, A4
   y A5 (12), A3 medido (13) y la 134 medida (14); ⚠️ §905 y §907: la 135 medida con su lema L1'
   (15) y la revisión del agente cerrada, con L7, L8 y las instancias medidas (16). Las marcas
   ⚠️ §n dicen qué decía antes cada pasaje y por qué cambió. Medio día.
3. **El código que el argumento describe**, con el documento al lado: en el fork,
   `crates/winter-prover/src/lib.rs` (`generate_proof_oculto`: las filas, la columna, las dos
   monedas y sus semillas), `crates/winter-prover/src/constraints/composition_poly.rs`
   (`segmentar`: los trozos y los aleatorizadores), `crates/winter-air/src/air/oculta.rs` (el
   envoltorio: 2T filas, las exenciones), `crates/winter-air/src/marca.rs` (la marca),
   `crates/winter-verifier/src/lib.rs` (`verificar_oculta`: lo que el verificador reconstruye) y
   `crates/zk-ssl-air/src/sal.rs` (`MerkleConSal`, las hojas saladas); y los instrumentos de
   `crates/zk-ssl/src/`: `instrumento_cota.rs` (A1: la cota y L1 por familia),
   `instrumento_r3.rs` (A3: O, K, las máscaras y R3 por geometría),
   `instrumento_falsadores_hvzk.rs` (los tres falsadores del juguete, con la moneda rehecha de una
   prueba real) e `instrumento_mlin.rs` (la 134: la sonda, M_lin y R3 por familia). Un día.
4. **La crónica**: los asientos §896 a §902 de `AUDITORIA.md`: qué midió cada uno, qué corrigió del
   anterior y las objeciones que cada revisión de la sesión puso. Una hora. ⚠️ §907: y el §905 (la
   135) y el §907 (la revisión del agente), más `doc/ocultacion-hvzk-revision-agente.md`, el informe
   íntegro de esa revisión: lo que leyó, sus siete objeciones y lo que dio por correcto con qué.
   Una hora más.

## 3. Lo que se afirma, con su clase y dónde se mide

| afirmación | clase | dónde se mide o se sostiene |
|---|---|---|
| L0: la transcripción no interactiva se simula programando el oráculo | modelo (H7) | — |
| L1: las T filas aleatorias cubren los 2q + 6 puntos tocados por columna, sobre F, en 18 familias | medida en cada canon | `instrumento_cota`: `INDEPENDIENTE_DE_LA_AIR`, `rango_de_los_tocados`, `rango_pleno`; el mutante T = 32 |
| L2: los m coeficientes de cada aleatorizador cubren los q + 2 puntos abiertos por trozo, sobre E | medida en cada canon | `instrumento_cota`: `rango_pleno` (Vandermonde); el mutante m = 43 |
| L3: lo que FRI abre de DEEP es simulable si las máscaras independientes del testigo cubren O|_K (R3) | demostrado como condición suficiente; R3 medida | `instrumento_r3`: `R3_MEDIDO` (falsa con la columna y los aleatorizadores), `R3_MEDIDO_LABORATORIO` (verdadera con dos columnas); `instrumento_mlin`: `MLIN_MEDIDO` (verdadera con M_lin en dieciséis familias) |
| H12: las filas aleatorias de una columna que entra linealmente con coeficiente público son una máscara más | medida en cada canon (el censo) y por familia; ⚠️ §907: en cada canon sólo el censo de las 23 y el rango de la solvencia; el rango de las otras quince se midió en el test largo y lo guarda la huella de cada composición, `HUELLA_C` (la O3 de la revisión del agente); y demostrada como lema, L7 | `instrumento_mlin`: `CENSO_134`, la sonda; el juguete reproduce lo clavado sin lineales y suma exactamente lo libre con ellas |
| H14 (§907): R3 con M_lin vale en los desafíos reales | demostrada en la capa 0 con T ≥ 512 fuera de las coincidencias coset–tocado (L8, fallo ≤ 7c₀/\|F\|); medida en lo demás en tres instancias por familia, una adversa | `instrumento_mlin`: `adversa`, `MLIN_INSTANCIAS`; la solvencia en cada canon, las quince en el test largo |
| L4: las hojas saladas esconden lo no abierto | demostrado con la sal fresca (H8, supuesta) | `crates/zk-ssl-air/src/sal.rs`, tests de tamaño y efecto; la frescura no se mide |
| L5: las máscaras salen de monedas propias, sembradas aparte de la pública | demostrado por el código; H9 (ninguna semilla repetida), supuesta | `generate_proof_oculto`, `segmentar`; el falsador de la semilla repetida |
| L6: lo que depende de la AIR con T = 64 | enunciado, no demostrado | el distinguidor del juguete con T = 64 |
| H3: z fuera de F y de H, sin coincidencias de conjugados | supuesta, con fallo acotado | no se mide |

⚠️ §915: con la construcción del corte, L1 es medida en las 23 (no en 18), L3 queda como condición demostrada por L9 para cualesquiera posiciones (`K_ENTERO_PAREADAS`, las cinco geometrías de producción), H12 y H14 son instrumentos de guarda que la proposición no necesita y L6 no se usa; las clases de L0, L2, L4, L5 y H3 no cambian.

## 4. Cómo reproducir cada cifra

Todo en release y con `--locked`; en depuración winterfell exige que los grados declarados sean
los reales y el test de las familias no corre. Los tiempos son los del contenedor de la sesión, con
cuatro CPU.

```bash
cargo test -p zk-ssl --release --locked instrumento_cota -- --nocapture
#   ~150 s: las 28 pruebas reales de los 23 probadores; la cota por columna y por trozo; L1 por
#   familia contra INDEPENDIENTE_DE_LA_AIR; R3 por geometría contra R3_MEDIDO y
#   R3_MEDIDO_LABORATORIO; el censo de la 134 contra CENSO_134; la solvencia contra MLIN_MEDIDO.
cargo test -p zk-ssl --release --locked instrumento_r3 -- --nocapture
#   4 s: el juguete (T = 128): 424 sin determinar, 254 con una columna, 424 con dos.
cargo test -p zk-ssl --release --locked instrumento_mlin -- --nocapture
#   20 s: la sonda en el juguete de las tres entradas; sin lineales 424/254/80, con las dos
#   lineales 156 y 330; la tabla del censo, coherente con lo clavado.
cargo test -p zk-ssl --release --locked instrumento_falsadores_hvzk -- --nocapture
#   los tres falsadores del juguete, con la moneda rehecha de los bytes de cada prueba:
#   T = 32 recupera la constante y T = 64 no; T = 64 distingue el testigo y T = 128 no; la semilla
#   repetida devuelve las dos constantes y la fresca no.
cargo test -p zk-ssl --release --locked mide_m_lin_en_las_23_familias -- --ignored --nocapture
#   ~45 min: la sonda en las 23 con dos semillas y M_lin en las dieciséis, contra MLIN_MEDIDO;
#   MLIN_SOLO=AuditAir,BandaAir mide sólo esas.
cargo test -p stark-experiment --release --locked falsadores_oculta -- --nocapture
#   los diez falsadores de la ocultación del núcleo; el décimo, el laboratorio de dos columnas.
bash tools/canon.sh --sello
#   ~10 min: todo lo anterior salvo el test largo, con sus pines, y el resto de la casa.
```

Cada cifra clavada está en una constante con su nombre, y cambiarla es rojo: `R3_MEDIDO`,
`R3_MEDIDO_LABORATORIO` (`instrumento_r3.rs`), `CENSO_134`, `MLIN_MEDIDO` (`instrumento_mlin.rs`),
`INDEPENDIENTE_DE_LA_AIR` (`instrumento_cota.rs`). Los desafíos de los instrumentos los elige el
test de una moneda sembrada con un número (la semilla 1 en lo clavado; la 2 y otras consultas en
las corridas del sello para T = 64); los falsadores del juguete rehacen la moneda real.

## 5. Dónde atacar

Lo que la sesión no pudo comprobar por sí misma, o donde el filo es más fino:

1. **La cuenta de L1.** El argumento condiciona cada columna a sus 2q + 6 puntos tocados, aunque el
   verificador sólo ve de ella q + 4 y, de los demás, sumas de todas las columnas (C(x_j) y
   C(z·g)). Es suficiente. ¿Es necesario? Si no lo es, la cuenta fina (la 135 de `BACKLOG.md`)
   podría cubrir las cinco familias de T = 64 sin más filas, y hace falta un lema nuevo.
2. **K y lo público de D.** L3 toma como público D en las q posiciones y en x₀ = (z + z·g)/2 y nada
   más. ¿Hay algo más de D que el verificador determine de lo que ve?
3. **El núcleo conjunto de la 134.** M_lin entra en los trozos del cociente, y el instrumento
   descuenta lo que el verificador ve de cada trozo —las q posiciones, z y z·g— con el núcleo
   conjunto con los aleatorizadores. ¿Es eso exactamente lo que `verificar_oculta` ve de los
   trozos, ni más ni menos?
4. **La réplica de la composición.** `Sonda::composicion` calcula C como el evaluador del
   probador; se validó contra pruebas reales en el juguete sin columnas periódicas (§898, `vista`)
   y por estructura en la 134 (C(P_ρ) cabe en los N trozos). Una validación contra el marco OOD de
   una prueba real de una AIR con columnas periódicas sería más fuerte, y no está hecha.
5. **La sonda.** «Coeficiente público» es constante, periódico o elemento aleatorio del tramo
   auxiliar. Las AIR se construyen con entradas públicas vacías, salvo la edad; ¿hay alguna
   restricción cuya forma dependa de las entradas públicas de un modo que la sonda no vea? (La edad
   cambia el grado de una restricción con `todos`; se sondeó sólo con `todos`, el enunciado de sus
   dos pruebas del testigo.)
6. **Los desafíos del test frente a los de la prueba.** Los rangos se miden con posiciones, z y
   coeficientes elegidos por el test; el argumento es que el rango es genérico (Schwartz-Zippel
   sobre E). ¿Hay desafíos reales con rango menor, y con qué probabilidad? ⚠️ §907: la revisión
   del agente atacó aquí (su O1): las posiciones reales no son genéricas —colisiones de cosets en
   más de la mitad de las pruebas de la solvencia, coincidencias coset–tocado con frecuencia
   Θ(1)—; y «sobre E» era un error: la aplicación es F-lineal y la cota va sobre |F|. Lo que hay
   ahora: L8 para la capa 0 con T ≥ 512, tres instancias medidas por familia —una adversa— y la
   cuenta de la ventaja rehecha (apartado 16 del documento). Lo que sigue abierto: una cota para
   la capa 1, las coincidencias y T = 256, que hoy sólo están medidas.
7. **La sal (L4).** Con R3 en verde, ¿hace falta la sal? Qué fallaría sin ella, hoja a hoja: la
   fila de la traza en un punto de Λ (L1), el vector de los trozos (L2) y los ocho valores de un
   coset de FRI (sólo R3). El autor decidió conservarla (A5, §902).
8. **La nota 2024/1037.** El apartado 10 compara con lo que la casa sabía de ella por el RFC-0009
   y su resumen; la nota no se pudo leer desde la sesión. Una comparación apartado por apartado
   queda pendiente.
9. **El fork.** Este dosier es del argumento; la otra mitad de H7 —ni el fork ni la construcción
   están auditados— es la lectura del código del fork frente a winterfell 0.13.1, que la casa no ha
   encargado.

Fuera del dosier, y dicho: las filas aleatorias del tramo auxiliar (sólo la edad lo tiene, con
T = 64); las cinco familias de T = 64, que necesitan la 131 o la 135; MerkleAir y FrozenClimbAir,
que necesitan la 133; el nivel frente a un adversario cuántico, que no está medido
(`SECURITY.md` §3.11).

## 6. Cómo se registra lo que diga

Cada objeción va al apartado 11 de `doc/ocultacion-hvzk.md` con su cierre —o sin él, si no lo
tiene—, como las de las dos revisiones de la sesión; un asiento de `AUDITORIA.md` dice quién
revisó, qué objetó y qué cambió; y después decide el autor sobre la D-AM, con su asiento. La casa
corrige en su sitio y marca lo que cambia: un revisor que encuentre un error no verá el error
borrado, sino corregido con una nota que dice qué decía. ⚠️ §907: así se registró la primera
revisión desde fuera de la sesión, la de un agente (apartado 8 de este dosier): su informe entero
en `doc/ocultacion-hvzk-revision-agente.md`, sus siete objeciones con su cierre en el apartado 11
del documento, lo que pidió escribir y medir en el apartado 16, y el asiento §907.

## 7. Lo que este dosier no es

No es una promesa ni cambia ninguna. No es una auditoría del fork. No mide nada: dice dónde está
cada medida y cómo volver a hacerla. Y no sustituye al revisor: la casa no lo elige aquí.

## 8. La revisión del agente (§907), y lo que cambió con ella

Antes de que el autor elija al revisor, la sesión pidió una revisión a un agente externo a ella
—otra instancia, sin la crónica ni las conclusiones, con este dosier y una copia del árbol del §903
delante, en lectura pura—. No cuenta como la revisión ajena de la 136, y sirve para que el revisor
humano no gaste su tiempo en lo que ya está visto: el informe íntegro es
`doc/ocultacion-hvzk-revision-agente.md`. No encontró un paso falso ni una medida que mida otra
cosa; puso siete objeciones, ninguna alta: una instancia de desafíos por familia y posiciones no
genéricas (O1), el lema del núcleo conjunto sin escribir (O2), el rango de quince familias sin
guarda en el canon (O3), la cuenta de la ventaja incompleta (O4), dos imprecisiones de la vista
(O5), la molienda del rasgo `concurrent` (O6) y el pánico del aserto de grado del DEEP (O7). El §907
las cerró —L7 y L8 escritos, tres instancias medidas por familia con una adversa, la huella de cada
composición clavada, la cuenta rehecha, los textos corregidos— salvo lo que es del fork o sigue
sin cota, y lo dice: la capa 1, las coincidencias coset–tocado y T = 256 están medidas y no
acotadas; el pánico es la 137 de `BACKLOG.md`; y una corrección que la revisión no vio: la cota
sobre los desafíos va sobre |F| = 2^64, no sobre |E|, porque la columna aleatoria vive en F. Lo que
el revisor humano encontrará, por tanto, es un argumento con una tercera revisión encima y con sus
huecos nombrados; el apartado 5 sigue valiendo, con el 6 reescrito.

## 9. La carta al revisor, para el autor

La elección del revisor es del autor (la 136). Un borrador de carta, en inglés, para cuando la haga;
los nombres los pone él. La elección natural son los autores de la nota ePrint 2024/1037, de la que
es la construcción del fork; segunda, quien mantiene winterfell, para la otra mitad de H7; tercera,
cualquier criptógrafo con STARKs y FRI que acepte una revisión pública con sus objeciones
registradas.

> Subject: Request for an external review of a zero-knowledge argument for a winterfell 0.13.1
> fork built on ePrint 2024/1037
>
> I maintain Arqueo, an open-source settlement layer whose proofs are STARKs produced by a fork of
> winterfell 0.13.1. Since its zkssl/0.4 cut the fork hides the witness with the construction of
> the note "A note on adding zero-knowledge to STARKs" (ePrint 2024/1037): T random rows behind each
> trace column, one random column, a blinded quotient with randomizers of degree < m, and salted
> Merkle leaves. The repository, its reasoning and every measurement are public.
>
> I am writing to ask whether you would review the hiding argument written for that
> implementation. It is a simulator-based argument (honest-verifier, statistical in the
> programmable random-oracle model), with every hypothesis classified as measured, proven,
> modelled or assumed, and each measured hypothesis tied to a test that re-measures it on every
> build. Two things found while measuring may interest you as readers of the construction:
>
> 1. With the random column living in the base field F while the DEEP polynomial lives in the
>    quadratic extension E, the column masks only one of the two E-coordinates of each value FRI
>    opens; together with the quotient randomizers it leaves what FRI opens of DEEP uncovered in
>    every production geometry (a rank condition we call R3). Two random columns, each with its
>    own DEEP coefficient, cover it in all geometries; so does one random column over E.
> 2. For AIRs whose columns enter the constraints linearly with public coefficients, the random
>    rows of those columns are an additional witness-independent linear mask that, in 16 of our
>    23 proof families, covers the gap with the current construction. Two families (hash-only
>    AIRs) have no such column, and five families with T = 64 rows are not covered by the base
>    lemma at all. A layer-0 coverage lemma holds for any query positions when T ≥ 512; what
>    depends on the positions beyond it is measured on several instances, not bounded.
>
> The review package is a single document, `doc/ocultacion-hvzk-dosier.md`: what to read and in
> which order (about two days), each claim with its class and where it is measured, the commands
> that reproduce every figure, the places where we think the argument is thinnest, and a first
> review by an automated reader whose objections are recorded with their resolution. We would
> record your objections verbatim, with their resolution or without one, in the document and in
> the project's audit log, under your name if you consent, and the project's author decides
> afterwards what, if anything, is promised to users. Nothing is promised today: the only claim
> the project makes about hiding is a measured census of literal values.
>
> Please let me know whether this is something you would be willing to do, under what conditions,
> and in what time frame. I would also welcome a pointer to someone you consider better placed.
