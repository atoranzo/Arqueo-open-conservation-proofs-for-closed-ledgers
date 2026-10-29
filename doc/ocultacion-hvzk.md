# La ocultación del fork, con su simulador: el argumento de A2 (§897)

**Estado**: documento de trabajo de la 130 de `BACKLOG.md` (A2 del HVZK), escrito en el §897 por la
sesión, y **revisable**: cada objeción con su cierre va en el apartado 11 y en el asiento que la
lleve. ⚠️ §898 (A3): R3 está medida, y no se cumple en ninguna geometría de producción; lo que eso
cambia está marcado con §898 donde toca y reunido en el apartado 13. **No es una promesa.** Lo que la casa promete sigue siendo el censo —«una prueba que verifica
demuestra la transición que enuncia y no publica literal ningún valor de columna constante de su
traza», RFC-0009 D-A y `SECURITY.md` §3.bis— hasta que A4, con A3 sellada, decida qué se promete y
con qué palabras. Nada de lo que sigue cambia `SECURITY.md`, el RFC-0009 más allá de la nota de su
D-I, ni la regla 3 del PROCESO; ningún texto vivo dice HVZK de nada que exista. ⚠️ §901 (A4):
decidido en la D-AM del RFC-0009: la promesa sigue siendo el censo; lo que se podría prometer para
dieciséis familias con la construcción de hoy —la proposición del apartado 8, bajo las hipótesis
del apartado 4 con la H12 del §900— queda escrito allí con sus palabras y es del autor; las siete
que no, con nombre y con lo que les falta. ⚠️ §902: el autor acepta ese camino y decide A5: la sal
se queda (RFC-0009 D-I); lo escrito espera la revisión ajena (la 136 de `BACKLOG.md`). ⚠️ §903: el
dosier para esa revisión —qué revisar, qué leer, cómo reproducir cada cifra y dónde atacar— es
`doc/ocultacion-hvzk-dosier.md`. ⚠️ §915: con el corte `zkssl/0.5` sellado (§914), la proposición para las veintitrés familias con la construcción de producción —las dos columnas pareadas, T ≥ 128, m = 64, las hojas saladas— está escrita en el apartado 18, con sus hipótesis y su cuenta, y lo que se podría prometer con ella, con sus palabras, en la D-AM del RFC-0009 (su borrador del §915). **Sigue sin ser una promesa**: es del autor, y el argumento espera la revisión ajena de la 136, que desde aquí es de los apartados 1 a 18 con la construcción del corte.

**Qué es.** El argumento de ocultación del probador oculto del fork de winterfell 0.13.1
(`crates/winter-prover`, `crates/winter-air`, `crates/winter-verifier`, con la sal de
`crates/zk-ssl-air/src/sal.rs`) tal como está implementado, con su simulador y sus lemas, cada
lema atado al test del §896 y del §897 que mide su hipótesis (`crates/zk-ssl/src/instrumento_cota.rs`),
la lista de hipótesis con su clase —medida, demostrada, de modelo o supuesta—, lo que no cubre y
dónde el fork se aparta de la nota 2024/1037.

**Qué da, en una frase.** ⚠️ §898: hasta aquí este párrafo decía que para diecisiete de las
veintitrés familias la vista del verificador honesto se simulaba sin el testigo «si se cumple una
condición suficiente que nadie ha medido, R3», y que seis quedaban fuera, las cinco con T = 64 por
L1 y la solvencia por cuenta. A3 midió R3 (apartado 13) y **no se cumple en ninguna geometría de
producción**: la columna aleatoria vive en F y, en cada valor de E que FRI abre, sólo mueve una de
sus dos coordenadas; los aleatorizadores del cociente aportan 80 dimensiones en total, no 40 por
trozo; y lo que FRI abre sin determinar son 168, 632, 760 y 1016 dimensiones frente a 126, 378 a
382, 510 y 766 de máscara. Hoy el argumento independiente de la AIR no cubre ninguna familia: lo
que sostenga la ocultación más allá del censo, si algo la sostiene, depende de la AIR —de una
máscara lineal propia, donde sus columnas entran sólo linealmente en las restricciones (M_lin, la
134 de `BACKLOG.md`, sin medir), o del testigo (L6)—, y nadie lo ha argumentado. La salida
independiente de la AIR es la 133: que la máscara de DEEP viva en E, en un corte futuro. El
enunciado del apartado 8 no se puede hacer hoy para ninguna familia; la promesa sigue siendo el
censo. ⚠️ **§900 (la 134): M_lin, medida (apartado 14).** En dieciséis de las veintitrés familias
—las seis de T = 1024, las nueve de T = 512 y la solvencia— R3 se cumple con la construcción de hoy
una vez contadas las filas aleatorias de las columnas que entran en las restricciones linealmente y
con coeficiente público, y en todas ellas esas filas con los aleatorizadores bastan solas;
MerkleAir y FrozenClimbAir (T = 256) no tienen ninguna columna así y siguen con la R3 del §898; las
cinco de T = 64 no tienen M_lin que medir (la 131). Para las dieciséis, la proposición del
apartado 8 vale con la construcción de hoy, bajo las demás hipótesis; la promesa sigue siendo el
censo hasta A4.
⚠️ **§914 (el corte `zkssl/0.5`, RFC-0018 D-H):** la construcción de producción es desde el corte la de las dos columnas aleatorias pareadas como una sobre E (D-AL bis, L9) en las 23 familias, con T = 128 en las cinco que tenían 64: R3 medida en verde en las cinco geometrías (apartado 13, `K_ENTERO_PAREADAS`), L1 cubriendo las 23 (`INDEPENDIENTE_DE_LA_AIR`), la cota de A1 en verde en las 28 pruebas. Lo que este documento dice de «hoy» desde aquí hasta el apartado 17 describe `zkssl/0.4`; la proposición para las 23, con sus hipótesis y su cuenta, se escribe en el asiento siguiente, y la promesa sigue siendo el censo hasta que el autor la acepte.
⚠️ **§915: con la construcción del corte, en una frase.** Para las veintitrés familias, la vista del verificador honesto se simula sin el testigo con el argumento independiente de la AIR: L1 por geometría (2q + 6 = 90 ≤ 128 ≤ T, en las 23), L2 (q + 2 = 44 ≤ m = 64), L9 (las dos columnas pareadas y los aleatorizadores cubren K entero, para cualesquiera posiciones: R3 es teorema, no hipótesis), L4 con la sal y L5 con las semillas, en el modelo de L0; ni M_lin, ni la cuenta fina, ni L6, ni L8 hacen falta, y quedan como instrumentos de guarda. La ventaja: 2^{−60} (H3) + 2^{−128} (S5) + Q·2^{−256} (L0) + |E|^{−(N−1)} (L9), sin ningún |F|. Lo que queda es lo que ningún test mide —H3, H7, H8 y H9— y la revisión ajena; la promesa sigue siendo el censo hasta que el autor la acepte (apartado 18).

**La nota, no leída.** La nota 2024/1037 de ePrint —Ulrich Haböck y Al Kindi, «A note on adding
zero-knowledge to STARKs», recibida el 26 de junio de 2024 y revisada por última vez el 22 de
febrero de 2025, según lo que devuelve una búsqueda de su página— no se pudo leer desde el
contenedor de esta sesión: `eprint.iacr.org` no resuelve, y para leerla desde una sesión en la nube
hay que permitir ese dominio en la red de la sesión. El apartado 10 compara con lo que la casa sabía
de ella por el RFC-0009 y con su resumen; la comparación apartado por apartado queda pendiente.

**Convención.** «Demostrado» es lo que este documento demuestra; «medido», lo que un test del árbol
mide en cada canon; «modelo», una hipótesis estándar que ningún test puede medir (el oráculo
aleatorio); «supuesto», lo que ni se demuestra ni se mide, con su probabilidad de fallo cuando se
acota. Los ⚠️ marcan lo que el §896 decía y este documento corrige.

## 1. El protocolo, tal como está implementado

**Notación.** F es el campo de producción, el de 64 bits de winterfell (`f64`, p = 2^64 − 2^32 + 1);
E su extensión cuadrática, donde viven z, los coeficientes del cociente y el polinomio DEEP
(`FieldExtension::Quadratic` en `proof_options()`, `crates/zk-ssl/src/lib.rs`); conj es la
conjugación de E sobre F. La traza real tiene T filas, potencia de dos con T ≥ 64 (RFC-0009 D-AG),
W columnas principales y W_aux auxiliares; la oculta, L = 2T filas. H = ⟨g⟩ ⊂ F es el dominio de la
traza oculta, con g de orden L; las filas reales son h_0 … h_{T−1} y las aleatorias h_T … h_{L−1}.
Λ = 7·⟨ω⟩ es el dominio LDE, con ω de orden 16L (blowup 16) y 7 el generador de F*
(`domain_offset`, `crates/winter-air/src/options.rs`); como 7 tiene orden p − 1, que no es potencia
de dos, 7 ∉ ⟨ω⟩ y Λ ∩ H = ∅: ninguna posición abierta cae en una fila, y 0 ∉ Λ. Las opciones de
producción (`proof_options()`): 42 consultas, blowup 16, molienda de 21 bits, extensión cuadrática,
plegado 8 y resto de grado ≤ 31 en FRI, lote lineal. q es el número de posiciones únicas que abre
una prueba, de 40 a 42 según la corrida, leído de ella (`num_unique_queries`).

- **P1. Filas y columna aleatorias** (`generate_proof_oculto`, `crates/winter-prover/src/lib.rs`).
  Una moneda `DefaultRandomCoin<Blake3>` se siembra con Blake3(semilla_filas), semilla de 32 bytes
  de `OsRng` por prueba (`ocultacion_encendida`, `crates/stark-experiment/src/lib.rs`;
  `sal::semilla`). De esa moneda salen, columna a columna, T elementos de F detrás de las T filas
  reales de cada columna principal; después L elementos de F para una columna principal más —la
  columna aleatoria, sin ninguna restricción—; y después, si hay tramo auxiliar, T elementos de E
  detrás de las filas reales de cada columna auxiliar. El meta de la traza lleva la marca con m
  (D-S); la traza real se comprueba contra el AIR interno antes de ocultarla (D-AH).
- **P2. El envoltorio** `Oculta<A>` (`crates/winter-air/src/air/oculta.rs`): presenta al núcleo un
  AIR de L filas con W + 1 columnas principales y las exenciones de transición del interno más T
  —las restricciones de transición no se imponen entre una fila aleatoria y la siguiente, ni entre
  la última real y la primera aleatoria— y delega en el interno transiciones, aserciones (que
  nombran filas reales, porque el interno ve T), columnas periódicas y tramo auxiliar. Sube el
  factor del dominio de restricciones si hace falta; eso sólo lo usa el probador.
- **P3. Los polinomios de columna.** Para cada columna, el polinomio P ∈ F[x] (E[x] en las
  auxiliares) de grado < L que interpola sus L valores sobre H. Su evaluación sobre Λ se
  compromete con `MerkleConSal` (P5), una hoja por punto de Λ con la fila entera del tramo.
- **P4. El cociente y sus trozos** (`segmentar`, `crates/winter-prover/src/constraints/composition_poly.rs`;
  `num_constraint_composition_columns`, `crates/winter-air/src/air/context.rs`). Con los
  coeficientes α de las restricciones, sacados de la moneda tras el compromiso de la traza, el
  polinomio de composición C(x) = Σ_c α_c·C_c(x)/Z_c(x) —transiciones sobre el marco (P(x), P(g·x))
  de todas las columnas, y aserciones— tiene grado < N·s, con s = L − m y N el número de trozos.
  Se parte C(x) = Σ_{k<N} x^{k·s}·h_k(x), con deg h_k < s, y cada trozo se enmascara:
  h'_k = h_k + x^s·r_k − r_{k−1}, con r_{−1} = r_{N−1} = 0 y r_0 … r_{N−2} ∈ E[x] de grado < m,
  uniformes de una moneda sembrada con Blake3(semilla_cociente), la otra semilla de 32 bytes. La
  suma telescopa: Σ_k y^{k·s}·h'_k(y) = C(y) para todo y. Cada h'_k tiene grado < L y se
  compromete, evaluado sobre Λ, con `MerkleConSal`. m = 64 en las 28 pruebas (`M_OCULTACION`).
- **P5. El compromiso con sal** (`MerkleConSal`, `crates/zk-ssl-air/src/sal.rs`): cada hoja es
  `merge(hash_elements(fila), sal)` —Blake3 del resumen de la fila y de la sal— con sal = Blake3
  de 32 bytes de `OsRng`, fresca por hoja; la apertura lleva el camino y la sal. ⚠️ §907 (la O5
  de la revisión del agente): decía «Blake3(item ‖ sal)», y el item es el resumen de la fila, no
  la fila; lo que un adversario tendría que adivinar sin la sal es la fila entera, no un resumen
  que conozca; L4 no cambia. Lo usan la traza, los trozos y las capas de FRI (winter-fri construye sus capas
  con `V::new`); el resto de FRI no va en un árbol: se compromete como Blake3 de sus coeficientes y
  viaja entero.
- **P6. El punto fuera del dominio.** z ∈ E sale de la moneda tras el compromiso del cociente. El
  probador manda el marco OOD de la traza —P_i(z) y P_i(z·g) de cada columna, principal (la
  aleatoria incluida) y auxiliar— y el del cociente, h'_k(z) y h'_k(z·g) de cada trozo
  (`QuotientOodFrame`). El verificador comprueba Σ_k z^{k·s}·h'_k(z) = C(z), con C(z) calculado
  del marco de la traza (`crates/winter-verifier/src/lib.rs`, paso 3); en z·g no comprueba nada:
  esos valores entran en DEEP.
- **P7. DEEP** (`crates/winter-prover/src/composer/mod.rs`, `crates/winter-verifier/src/composer.rs`).
  Con un coeficiente cc por columna y por trozo, sacado de la moneda tras el marco OOD (lote
  lineal): D(x) = Σ_i cc_i·Q[P_i](x) + Σ_k cc'_k·Q[h'_k](x), con
  Q[P](x) = (P(x) − P(z))/(x − z) + (P(x) − P(z·g))/(x − z·g). Es un polinomio de grado ≤ L − 2
  sobre E, y el probador aserta que el grado es exactamente L − 2.
- **P8. FRI** (winter-fri 0.13.1, sin bifurcar). Sobre D evaluado en Λ: en cada capa los valores se
  agrupan de ocho en ocho —los cosets de plegado, {x : x^8 = c}— en una hoja, se comprometen con
  `MerkleConSal`, sale α_i de la moneda y se pliega. Para las trazas de la casa son una capa
  (T = 64) o dos (T ≥ 256), y el resto, de 8 a 32 coeficientes según la longitud, viaja entero
  (§896, tabla).
- **P9. Molienda y posiciones.** El probador busca un nonce con 21 ceros a la izquierda en el hash
  de la moneda; de la moneda con el nonce salen 42 posiciones de Λ, con repeticiones, y el probador
  abre las únicas: q ≤ 42. Las posiciones no viajan; las deriva el verificador.
- **P10. Aberturas.** En cada posición x_j: la fila entera de la traza principal (las W + 1
  columnas) y de la auxiliar, con su sal y su camino; el valor de cada trozo h'_k(x_j), con su sal;
  y en FRI, en cada capa, la hoja entera del coset de plegado de la posición plegada —ocho
  valores, uno de ellos el de la posición— con su sal.
- **P11. El verificador** (`perform_verification`): reconstruye la moneda, comprueba la molienda,
  la consistencia OOD en z y las aperturas contra las raíces, calcula D(x_j) de las aperturas y de
  los marcos con la fórmula de P7 y se lo da a FRI, que comprueba que el valor calculado es el que
  la hoja de la capa 0 lleva en su posición, pliega cada coset con α_i y comprueba la capa
  siguiente, y al final el resto.
- **P12. Lo público por diseño.** Las entradas públicas, el contexto (longitudes, anchos, opciones,
  la marca con m) y lo que el sobre, el recibo o la operación lleven en claro (RFC-0009 D-B,
  columna «público por diseño»). No es objeto de este argumento.

## 2. La vista del verificador honesto

Lo que recibe, en orden: (V1) las raíces de la traza principal y, si la hay, de la auxiliar; (V2)
el marco OOD de la traza, 2(W + 1 + W_aux) valores de E; (V3) la raíz del cociente; (V4) el marco
OOD del cociente, 2N valores de E; (V5) las raíces de las capas de FRI, el compromiso del resto
—Blake3 de sus coeficientes, el último de `commitments`, que `read_remainder` del fork ata al
resto— y el resto entero; (V6) el nonce; (V7) en cada posición única x_j, la fila principal (W + 1
valores de F), la auxiliar (W_aux de E), los N trozos (E), y sus sales y caminos; (V8) en cada capa
de FRI y cada posición plegada, los ocho valores del coset, con sal y camino. Más lo que él mismo
deriva: los desafíos (los elementos del tramo auxiliar, α, z, cc, α_i, el nonce y las posiciones) y
lo que calcula de lo anterior: C(z), D(x_j) y —aunque no lo comprueba— C(x_j) =
Σ_k x_j^{k·s}·h'_k(x_j), C(z·g) = Σ_k (z·g)^{k·s}·h'_k(z·g) y D(x₀), con x₀ = (z + z·g)/2 (L3).
⚠️ §907 (la O5 de la revisión del agente, apartado 11): la `Proof` lleva además
`num_unique_queries`, un `u8` que es función de V6 —las posiciones únicas que `draw_integers`
deriva del nonce—, y el compromiso del resto, que V5 no nombraba y es función del resto; con los
dos dichos, la vista y la `Proof` coinciden campo a campo (`context`, `num_unique_queries`,
`commitments`, `trace_queries`, `constraint_queries`, `ood_frame`, `fri_proof`, `pow_nonce`).

Los conjuntos de puntos, por columna:

- **Abiertos**, S_ab = {x_1 … x_q} ∪ {z, z·g}: q + 2 puntos. Las posiciones viven en F y los dos de
  fuera en E, así que lo abierto de una columna principal mide **q + 4 dimensiones sobre F** (46 con
  q = 42); lo de una auxiliar, q + 2 sobre E; lo de un trozo, **q + 2 sobre E** (44).
- **Tocados**, S_toc = S_ab ∪ {g·x_1 … g·x_q} ∪ {g²·z}: 2q + 3 puntos, porque C(x_j) lee el marco
  (P(x_j), P(g·x_j)) y C(z·g) lee (P(z·g), P(g²·z)). Mide **2q + 6 dimensiones sobre F** por
  columna principal (90 con q = 42) y 2q + 3 sobre E por auxiliar. Un g·x_j que coincide con otra
  posición abierta cuenta una vez.
- **x₀** = (z + z·g)/2 ∈ E, donde el factor lineal con el que cada objeto entra en DEEP se anula.

## 3. Los lemas

Cada lema dice qué demuestra, con qué hipótesis, y qué test mide esa hipótesis sobre las 28 pruebas
reales de los 23 probadores con fila (`la_cota_de_ocultacion_de_los_23_probadores_con_fila`, en
cada canon) o sobre el juguete, `WorkAir` con 128 filas reales, m = 64 y las consultas y el blowup
de producción.

### L0 — La transcripción y el oráculo

Fiat–Shamir con la moneda `DefaultRandomCoin<Blake3>`: cada desafío es Blake3 de lo comprometido
hasta entonces. El argumento se hace sobre el protocolo interactivo con verificador honesto —el
simulador elige los desafíos, uniformes, antes de construir nada— y pasa al no interactivo en el
modelo de oráculo aleatorio programable: el simulador fija las salidas de Blake3 en los prefijos de
su transcripción para que la moneda dé sus desafíos, y falla sólo si el adversario ya había
consultado uno de esos prefijos, que llevan raíces frescas (L4) y tienen entropía de sobra. La
molienda no depende del testigo: la búsqueda del nonce —el primero cuyo hash con la moneda lleva
21 ceros— corre sobre salidas del oráculo que el simulador NO fija, para que el nonce salga con su
distribución; lo que fija son las salidas de después, las que `draw_integers` deriva del nonce
para las posiciones. ⚠️ §907 (la O6 de la revisión del agente): «el primero» es la búsqueda
secuencial, `(1..u64::MAX).find`, la que compila la casa: ningún `Cargo.toml` del espacio de
trabajo enciende el rasgo `concurrent` de `winter-prover`; con él la búsqueda es `find_any` y el
nonce es el primero que encuentra un hilo, una distribución que tampoco depende del testigo y que
un simulador con la misma búsqueda paralela reproduce. L0 queda atada al rasgo apagado; quien lo
encienda corre el simulador con la misma búsqueda. **Clase: modelo.** Ningún test la mide; es la hipótesis estándar de
toda prueba de Fiat–Shamir, y la casa ya la usa para la solidez.

### L1 — Las filas aleatorias cubren un conjunto de puntos si su cuenta sobre F cabe en T

**Enunciado.** Sea P = P_real + Σ_{r=T}^{L−1} ρ_r·L_r el polinomio de una columna principal, con ρ
uniforme en F^T y L_r la base de Lagrange sobre H. Sea S ⊂ E \ H finito, S_F = S ∩ F, S_E = S \ F,
y S̄ = S_F ∪ S_E ∪ conj(S_E). Si los puntos de S̄ son distintos dos a dos y |S_F| + 2|S_E| ≤ T,
entonces (P(y))_{y∈S} es uniforme en F^{S_F} × E^{S_E}, para cualquier P_real. Para una columna
auxiliar, con ρ uniforme en E^T: uniforme en E^S si |S| ≤ T y los puntos de S son distintos. Para la
columna aleatoria, con sus L valores uniformes en F: lo mismo con L en lugar de T.

**Demostración.** Un polinomio de grado < L que vale 0 en las T filas reales es Z_real·R, con
Z_real = Π_{r<T}(x − h_r) y R ∈ F[x] de grado < T, y la correspondencia ρ ↦ Σ ρ_r·L_r ↦ R es
biyectiva: Z_real no se anula en las filas aleatorias, y evaluar R en T puntos distintos, escalado
por constantes no nulas, es biyectivo. Así P(y) = P_real(y) + Z_real(y)·R(y) con Z_real(y) ≠ 0 fuera
de H, y la uniformidad de (P(y))_S equivale a que la evaluación ev_S: F[x]_{<T} → F^{S_F} × E^{S_E}
sea sobreyectiva. Para R ∈ F[x], R(conj y) = conj R(y): el valor en un punto de E lleva el de su
conjugado. Dado un objetivo (v_y)_{S_F}, (w_y)_{S_E}, se extiende a S̄ con conj(w_y) en conj(y); la
Vandermonde de E[x]_{<T} sobre S̄ es sobreyectiva si los puntos son distintos y |S̄| ≤ T, y da una
solución R' ∈ E[x]; conj(R') resuelve el mismo sistema, y R = (R' + conj R')/2 ∈ F[x]_{<T} también
(2 es invertible en F). Recíprocamente, si |S_F| + 2|S_E| > T la dimensión no alcanza, y si y y
conj(y) están los dos en S_E, el objetivo con w_{conj y} ≠ conj(w_y) no se alcanza. En las auxiliares
es la Vandermonde de E[x]_{<T} sobre S sin más. ∎

**Lo que mide el instrumento.** `rango_f(matriz_de_lagrange(S, T))`, el rango sobre F de la matriz
(L_r(y))_{y∈S, T≤r<L}, es el rango de ev_S —la matriz de Lagrange es la de ev_S compuesta con la
biyección de arriba—, y el test lo exige igual a |S_F| + 2|S_E|:

- sobre S_ab, `rango_pleno`: 46 de 46 en las 28 pruebas (45 de 45 en la que abrió 41 posiciones
  únicas; q + 4 ≤ T, con T ≥ 64);
- sobre S_toc, `rango_de_los_tocados`: 90 de 90 en las 19 pruebas de las 18 familias con T ≥ 256, y
  T de 2q + 6 —64 de 90, o 64 de 88 en la que abrió 41— en las 9 pruebas de las 5 familias con
  T = 64 (NullifierThresholdAir, ThresholdAir, EdadAir con ocho hojas, RefundAir, RefundAirV2), que
  la tabla `INDEPENDIENTE_DE_LA_AIR` declara y el test contrasta en los dos sentidos;
- el mutante T = 32 (`el_mutante_t_32_deja_las_columnas_sin_cota`): 46 frente a 32, rango 32;
- la prueba de vida (`el_rango_es_pleno_con_puntos_distintos_y_cae_con_uno_repetido`): con un punto
  repetido el rango baja en uno; 87 puntos tocados distintos dan 90 dimensiones con T = 128 y 64
  con T = 64.

**Hipótesis.** (H1, medida) la cuenta, q + 4 ≤ T sobre S_ab y 2q + 6 ≤ T sobre S_toc, por familia,
con el q leído de cada prueba; el rango pleno que el instrumento calcula es sobre posiciones que
elige el test y un z fijo, una prueba de vida de la lema, no una medida sobre las posiciones
reales. (H2, demostrada) Λ ∩ H = ∅ y las posiciones son
distintas (el probador abre únicas). (H3, supuesta, con probabilidad de fallo acotada) z ∉ F ∪ H y
ningún conjugado de z, z·g, g²·z coincide con otro de los tres: cada coincidencia es una ecuación no
trivial en z, de la forma z^{p−1} = g^e, con a lo sumo p − 1 soluciones en E, así que la
probabilidad total es menor que 2^{−60} por prueba; el instrumento usa un z fijo que las cumple, y
las posiciones y el z reales no viajan (§896, «lo que NO hace»). Una coincidencia entre g·x_j y otra
posición abierta reduce S_toc y no estorba: el simulador muestrea por punto distinto.

### L2 — Los trozos abren, en cada punto, un vector uniforme de su hiperplano

**Enunciado.** Sea S un conjunto de puntos distintos de E \ {0} con |S| ≤ m. Con r_0 … r_{N−2}
uniformes e independientes en E[x]_{<m}, los vectores (h'_k(y))_{k<N}, y ∈ S, son independientes
entre sí, y cada uno es uniforme en el hiperplano afín {v ∈ E^N : Σ_k y^{k·s}·v_k = C(y)},
cualesquiera que sean los h_k.

**Demostración.** Para y fijo, la aplicación (r_0(y), …, r_{N−2}(y)) ↦ (y^s·r_0(y),
y^s·r_1(y) − r_0(y), …, y^s·r_{N−2}(y) − r_{N−3}(y), −r_{N−2}(y)) es lineal e inyectiva —triangular
desde la última coordenada— y su imagen está en el hiperplano Σ_k y^{k·s}·u_k = 0, porque la suma
telescopa; el hiperplano tiene dimensión N − 1, la del dominio: es el hiperplano entero. Y
(r_k(y))_{y∈S} es uniforme en E^S para cada k, independiente de los otros k, porque la Vandermonde
de E[x]_{<m} sobre S tiene rango |S| cuando |S| ≤ m y los puntos son distintos. Sumar h_k(y), fijo,
traslada el hiperplano a C(y). ∎

**Lo que mide el instrumento.** `rango(matriz_de_vandermonde(S_ab, m))` sobre E: 44 de 44 en las
28 pruebas (q + 2 ≤ m = 64); el mutante m = 43 del juguete
(`el_mutante_m_43_deja_los_trozos_sin_cota_y_sigue_verificando`): 44 frente a 43, rango 43, y la
prueba sigue verificando (m decide la ocultación, no la solidez, RFC-0009 D-H); y el mutante del
árbol, `M_OCULTACION` 64 → 43, corrido en el §897 con la cuenta corregida: en rojo las 23 pruebas
que abrieron 42 posiciones únicas —«trozos: 44 puntos abiertos, 44 dimensiones sobre E, frente a
m = 43» y rango 43— y en verde las cuatro que abrieron 41 y la que abrió 40, como dice la
desigualdad. En una primera corrida el mismo mutante cayó antes en otra puerta, la banda de bytes
de D-AC: con m = 43 los trozos son menos y una prueba de RefundAirV2 con q = 41 adelgazó por debajo
de su banda (38.595 B frente a 39.019), y `asierta` paró el test antes de juzgar la cota. ⚠️ El
§896 contaba q + 2 + 1 ≤ m, puso en rojo las 28 de su corrida y decía que m = 43 caía con q ≥ 41;
con la cuenta buena, q + 2 ≤ m, m = 43 cae sólo con q = 42, m = 42 con q ≥ 41, y m ≤ 41 con toda
prueba de producción (q de 40 a 42). El instrumento lo dice así desde el §897.

**Hipótesis.** (H4, medida) q + 2 ≤ m y el rango pleno. (H5, demostrada) s = L − m ≥ 1 y N ≥ 2
(`segmentar` lo aserta), y ≠ 0 (0 ∉ Λ; z = 0 tiene probabilidad p^{−2}). Lo que el lema NO dice:
nada de los h_k ni de C(y). Lo que el verificador aprende de los trozos en y es exactamente C(y), y
L1 y L6 dicen qué es C(y).

### L3 — DEEP y FRI: lo determinado, lo que se abre de más y la condición R3

**Lo determinado.** D(x_j), en las q posiciones, lo calcula el verificador de V2, V4 y V7: no es
información nueva. Y en x₀ = (z + z·g)/2: para cualquier P,
Q[P](x) = [P(x)·(2x − z − z·g) − P(z)·(x − z·g) − P(z·g)·(x − z)] / ((x − z)(x − z·g)), y en x₀ el
primer término se anula, así que Q[P](x₀) = 2·(P(z·g) − P(z))/(z·g − z): la pendiente entre los dos
valores del marco. Luego D(x₀) = 2·[Σ_i cc_i·(P_i(z·g) − P_i(z)) + Σ_k cc'_k·(h'_k(z·g) − h'_k(z))]
/ (z·g − z) es público. ⚠️ El §896 contaba esa dirección como «una abertura más que un simulador
tiene que enmascarar» (`DIRECCION_RESIDUAL`, la nota de la D-I): no lo es. La máscara de la columna
aleatoria se anula en x₀, cierto; pero el valor de D en x₀ no depende del testigo más que a través
del marco OOD, que ya es de la vista: ni fuga ni abertura. El instrumento ya no la cuenta.

**Lo que se abre de más.** FRI abre de D mucho más que D(x_j): en la capa 0, los otros siete
valores del coset de plegado de cada posición; en cada capa siguiente, los ocho valores del coset
de la posición plegada, que son funcionales lineales de D —la capa i es el pliegue de la i − 1 con
α_i (`apply_drp`), un polinomio de grado < L/8^i—; y el resto entero. Llámese O a la aplicación
lineal que lleva D ∈ E[x]_{≤L−2} a todo lo que FRI abre de él, los D(x_j) incluidos. Su rango no
es el número de valores abiertos: con c₀ cosets distintos en la capa 0 y c₁ en la capa 1 (c₀ y
c₁ ≤ q), la capa 0 aporta 8c₀ dimensiones sobre E; la capa 1, a lo sumo L/8 —la dimensión del
espacio de su polinomio—, de las que c₀ ya estaban (el pliegue de cada coset abierto); y el resto
no aporta nada cuando la capa anterior ya lo determina (8c₁ ≥ L/8, siempre en la casa). Con q = 42
y los cosets distintos: T = 64, 127 sobre E y 254 sobre F, todo D —FRI entrega D entero—; T = 256,
358 y 716; T = 512, 422 y 844; T = 1024, 550 y 1100, de las 1022, 2046 y 4094 dimensiones que mide
D. Y lo que queda sin determinar, rango_F(O|_K) con K el núcleo de las q + 1 condiciones de abajo,
es el rango de O menos 2q y menos lo que D(x₀) añada (0 o 2): unas 168, 630, 758 y 1014. ⚠️ La
primera versión de este documento, en el §897 y antes de su revisión, contaba filas en vez de
rangos —704, 1276, 1292 y 1324— y decía que con T = 256 FRI entregaba D entero; no es así:
716 < 1022.

**Las máscaras, y cuáles cuentan.** De los tres aleatorizadores, sólo dos entran en D de forma
lineal e independiente del testigo: la columna aleatoria, que no está en ninguna restricción y
entra sólo por cc_R·Q[R], y los aleatorizadores r_k del cociente, que entran por
Σ_k cc'_k·Q[x^s·r_k − r_{k−1}]. Las filas aleatorias de las columnas reales NO: entran en D por
Q[P_i], lineal, pero también por los trozos h_k, que son los de
C(x) = Σ_c α_c·C_c(P(x), P(g·x))/Z_c(x), y C depende de los polinomios de columna enteros —las
exenciones quitan factores del divisor, no términos del numerador— con el grado de cada
restricción: en los puntos abiertos no se nota, porque C(y) sólo lee el marco tocado y L1 y L2 lo
absorben, pero FRI abre D en los x_j·μ con μ ≠ 1 y en las capas, donde entran los h_k(x_j·μ), que
dependen de las filas aleatorias de forma polinómica y, en su parte lineal, del testigo. Lo mismo
las auxiliares. Así que, condicionada la vista a lo abierto, a lo tocado y a los valores de los
r_k en S_ab, D = D_resto + M_R(ρ_R) + M_r(r_res): D_resto depende del testigo, de las filas
aleatorias de las columnas reales y de lo condicionado, de cualquier forma; ρ_R es lo libre de la
columna aleatoria, L − q − 4 dimensiones sobre F (sus L valores menos los q + 4 abiertos); r_res,
lo libre de los aleatorizadores, 2(m − q − 2) por cada uno de los N − 1 (sus m coeficientes de E
menos sus valores en los q + 2 puntos abiertos). ⚠️ §898: esa cuenta de los aleatorizadores es la
de sus grados de libertad, no la de lo que llega a D: los N − 1 entran en D por dos huecos,
Σ_k cc'_k·(x^s·r_k − r_{k−1}) = x^s·A − B con A y B en E[x]_{<m}, así que su imagen en D mide a lo
sumo 2(m − q − 2) sobre E tras lo abierto, 40, es decir 80 sobre F, con cualquier N ≥ 3 (40 con
N = 2); medido (apartado 13). Y la columna aleatoria, en cada punto x de Λ, aporta
cc_R·Q[R](x) = R(x)·c(x) con R(x) ∈ F y c(x) ∈ E fijo: una sola dimensión sobre F de las dos que
tiene cada valor que FRI abre, por mucho que L − q − 4 sea grande. M_R y M_r son lineales sobre F e
inyectivas (el núcleo de P ↦ Q[P] son las constantes, que lo residual no contiene), independientes
de D_resto, y
valen 0 en x_j y en x₀, porque lo residual no mueve los valores en S_ab y Q[·](x₀) sólo depende
de ellos: im(M_R), im(M_r) ⊆ K = {D : D(x_j) = 0 ∀j, D(x₀) = 0}, y D_resto está en el espacio afín
de D.

**La condición R3.** La vista real de FRI es O(D_resto) + O(M_R ρ_R + M_r r_res); la del simulador
(apartado 5, S5) es uniforme en O(D_0) + O(K), con D_0 cualquier polinomio con los valores
determinados. Si **rango_F(O∘[M_R | M_r]) = rango_F(O|_K)**, entonces O(M_R ρ_R + M_r r_res) es
uniforme en O(K) e independiente de D_resto, y la vista real es uniforme en O(D_resto) + O(K) =
O(D_0) + O(K): la misma. R3 es **suficiente**, no necesaria: si falla, lo que quede sin cubrir lo
mueven las filas aleatorias de las columnas reales de una forma que depende de la AIR y del
testigo, como en L6, y ningún instrumento lineal lo mide. ⚠️ La primera versión de este documento
la enunciaba como «si y sólo si», con las filas aleatorias de las columnas reales dentro de M; la
revisión escéptica del §897 lo corrigió (apartado 11, objeciones 1 y 2). Es una condición de rango
sobre matrices que se construyen de la geometría (q, T, m, N, capas, resto) y de los desafíos
(posiciones, z, cc, α_i), como las de L1 y L2, y **nadie la ha medido**: ni el §896 ni el §897.
Las cuentas de dimensión, necesarias y no suficientes, con q = 42: las máscaras suman a lo sumo
(L − q − 4) + 40·(N − 1); con T = 1024 (L = 2048, N = 8), 2002 + 280 frente a unas 1014, y la
columna aleatoria sola llega por cuenta; con T = 512 (N = 8), 978 + 280 frente a unas 758, y la
columna sola también; con T = 256, 466 + 320 = 786 frente a unas 630 en MerkleAir y FrozenClimbAir
(N = 9), a medir, y **466 + 40 = 506 frente a unas 630 en SolvencyAir (N = 2): R3 falla por cuenta,
y con q = 40 y cosets repetidos sigue fallando**; con T = 64, 82 + 560 frente a unas 168, pero
esas cinco ya quedaron fuera por L1. ⚠️ §898: esas cuentas eran cotas por arriba y la estructura
las hunde: medidas, las máscaras llegan a 126, 378, 382, 510 y 766 frente a 168, 632, 632, 760 y
1016 (apartado 13), y R3 falla en las cinco geometrías, no sólo en la solvencia. ⚠️ Lo que la nota de la D-I del §896 daba por hecho —«la
columna aleatoria … enmascara DEEP en todas las direcciones menos una»— no es cierto en general:
la columna aleatoria entra en DEEP como cc_R·Q[R] con R ∈ F[x]_{<L}, y Q[R] recorre una imagen de
dimensión L − q − 4 sobre F dentro de un espacio de 2(L − 1); con T = 64 y T = 256 no puede
enmascarar sola lo que FRI abre, y en todas la uniformidad depende del rango conjunto.

**Hipótesis.** (H6, **no medida**) R3, suficiente. Es lo primero que A3 tiene que medir, y el
apartado 6 dice cómo; las matrices son pequeñas y cabe en el sello. Si R3 falla en una familia, el
argumento de este documento NO vale para ella: en la solvencia falla por cuenta, y la salida es la
de la 132, una máscara propia de DEEP o más filas (apartado 10). ⚠️ §898: **medida, y falsa en las
cinco geometrías de producción** (apartado 13): el argumento de este documento no vale hoy para
ninguna familia, y la salida es la 133.

### L4 — Las hojas saladas esconden lo que no se abre y dejan equivocar lo que se abre

**Enunciado** (modelo de oráculo aleatorio programable, Blake3). Con sal fresca de 256 bits por
hoja: (a) el digest Blake3(item ‖ sal) de una hoja no abierta es indistinguible de uniforme para
quien no conoce la sal, salvo que consulte el oráculo en ese (item ‖ sal), probabilidad 2^{−256}
por consulta; (b) para una hoja abierta, un simulador que comprometió un digest uniforme puede
después elegir el item y una sal fresca y programar Blake3(item ‖ sal) := ese digest, consistente
salvo esa misma probabilidad. Luego las raíces y los caminos de la traza, los trozos y las capas de
FRI no dicen nada de las hojas no abiertas. El simulador del apartado 5 usa (a) y no (b): elige los
desafíos de antemano y compromete lo que va a abrir; (b) es lo que usaría un simulador que no
pudiera, y es lo que A5 pesa al decidir la sal. El resto de FRI no lleva sal: viaja entero, y su
compromiso Blake3(resto) no esconde nada ni tiene que esconderlo.

**Lo que mide el árbol.** `crates/zk-ssl-air/src/sal.rs`: `la_sal_pesa_treinta_y_dos_bytes_por_hoja_abierta`,
`la_sal_mueve_la_raiz_y_dos_compromisos_de_lo_mismo_difieren`, `una_sal_ajena_o_un_item_ajeno_rechazan`
y `una_sal_de_menos_falla_cerrada`; que el `VC` de los 23 probadores y del kit es `MerkleConSal` lo
exige el censo de D-Z. **Hipótesis.** (H7, modelo) Blake3 como oráculo aleatorio programable.
(H8, supuesta, `SECURITY.md` §3.11) `OsRng` da bytes frescos: una sal repetida entre dos hojas del
mismo árbol rompe (a) sólo para esas dos; ningún test lo comprueba ni puede.

### L5 — Las semillas y la moneda: el azar de las máscaras

Las filas y la columna aleatorias y los aleatorizadores no son uniformes: son salidas de
`DefaultRandomCoin<Blake3>` sembrada con Blake3(semilla), 32 bytes de `OsRng` por prueba y por
uso (filas y cociente, dos semillas). En el modelo de L4, las salidas de la moneda con una semilla
que el adversario no ha consultado son uniformes e independientes; la ventaja de un distinguidor es
la de adivinar 256 bits, por consulta. Los dos usos tienen semillas distintas, así que las filas y
los aleatorizadores son independientes entre sí y de la sal. **Lo que mide el árbol.**
`crates/stark-experiment/src/falsadores_oculta.rs`: `la_semilla_de_produccion_son_32_bytes_frescos`
(§652) y `el_ultimo_byte_de_cada_semilla_mueve_la_prueba`, que cambia el byte 31 de cada semilla y
exige que la prueba cambie. **Hipótesis.** (H7, modelo) como en L4. (H9, supuesta) ninguna semilla
se repite entre dos pruebas: con la misma semilla de filas, dos pruebas de dos testigos abren
P(x_j) − P'(x_j) = P_real(x_j) − P'_real(x_j) en cada posición común, y la diferencia de los testigos
sale en claro; es el falsador de A3. (H10, demostrada) la moneda de las máscaras no comparte estado
con la pública: se siembra aparte, en el probador, y la pública no la ve.

### L6 — Lo que depende de la AIR, enunciado y no demostrado

Para una familia con 2q + 6 > T —las cinco de T = 64— L1 no cubre S_toc: los valores tocados de una
columna principal viven en una variedad afín de dimensión T dentro de 2q + 6 sobre F, y C(x_j) y
C(z·g) dependen del testigo a través de P(g·x_j) y P(g²·z) de una forma que la AIR decide. Un
argumento para esas familias tendría que demostrar, AIR a AIR, que la distribución de (C(x_j))_j y
C(z·g) dada lo abierto no depende del testigo —por ejemplo, porque la aplicación de lo residual de
(P(g·x_j))_j en (C(x_j))_j es sobreyectiva sobre E^q para toda traza real—, y eso depende del
álgebra de sus transiciones. No se intenta aquí, y la fuga no es un escrúpulo: en el juguete, cuya
transición es x_{i+1} = x_i³ + 42, C(x_j) da P(g·x_j) despejando, y con T = 64 la vista depende del
testigo —87 evaluaciones de una columna de grado < 128 que 64 filas aleatorias no cubren—; no es
una recuperación por álgebra lineal (quedan 23 ecuaciones lineales sobre 64 valores reales, y la
ligadura que sobra es cúbica), es una distribución que cambia con el testigo, y el falsador de A3
es un distinguidor (apartado 12). Las salidas: L6 por AIR, o T ≥ 2q + 6 = 90, es decir T = 128 para las cinco —el doble de
filas y de coste para ellas, en un corte futuro, nunca en `zkssl/0.4`— (la 131 de `BACKLOG.md`);
la edad con 16 hojas o más (T = 8·2^m con m ≥ 4) ya cumple la cuenta, medida sólo en la cota: el
instrumento la mide con ocho hojas.
⚠️ §905: la otra salida, la cuenta fina para las columnas lineales (L1', apartado 15), está
medida: no rescata hoy a ninguna de las cinco; al umbral conjunto lo cubriría con la 133.

## 4. Las hipótesis, con su clase

| hipótesis | clase | lema | qué la mide o la sostiene |
|---|---|---|---|
| H1: q + 4 ≤ T sobre S_ab y 2q + 6 ≤ T sobre S_toc | medida, en cada canon; el rango, prueba de vida sobre puntos del test | L1 | `rango_pleno`, `rango_de_los_tocados`, `INDEPENDIENTE_DE_LA_AIR`; el mutante T = 32; la prueba de vida |
| H2: Λ ∩ H = ∅, posiciones únicas | demostrada | L1 | el orden de 7; el `dedup` del verificador |
| H3: z fuera de F y de H, sin coincidencias de conjugados | supuesta, fallo < 2^{−60} por prueba | L1 | no se mide: las posiciones y z no viajan |
| H4: q + 2 ≤ m, Vandermonde de rango pleno sobre E | medida, en cada canon | L2 | `rango_pleno`; el mutante m = 43; el mutante del árbol |
| H5: s ≥ 1, N ≥ 2, y ≠ 0 | demostrada | L2 | los asertos de `segmentar` |
| H6: R3, suficiente: las dos máscaras independientes del testigo cubren lo que FRI abre | **medida en el §898: con M = {columna, aleatorizadores}, falsa en las cinco geometrías**; ⚠️ §900: con M_lin (H12), verdadera en dieciséis familias y falsa en MerkleAir y FrozenClimbAir (apartado 14) | L3 | `instrumento_r3`, en cada canon, con sus cifras clavadas (apartado 13); `instrumento_mlin`, el censo en cada canon y la medida por familia (apartado 14) ⚠️ §915: con `zkssl/0.5`, demostrada (L9, apartado 17) y medida además en las cinco geometrías de producción, la de T = 128 incluida (`K_ENTERO_PAREADAS`, apartado 18) |
| H7: Blake3 como oráculo aleatorio programable | modelo | L0, L4, L5 | — |
| H8: la sal es fresca | supuesta | L4 | `SECURITY.md` §3.11; los tests de `sal.rs` miden tamaño y efecto, no frescura |
| H9: ninguna semilla se repite | supuesta; lo que cuesta, medido en el juguete (§898) | L5 | `instrumento_falsadores_hvzk`: dos pruebas con la misma semilla dan sus dos constantes |
| H10: la moneda de las máscaras no comparte estado con la pública | demostrada | L5 | `generate_proof_oculto`, `segmentar` |
| H11: T ≥ 2q + 6 | medida: 18 familias sí, 5 no; y lo que cuesta, en el juguete (§898) | L1, L6 | `INDEPENDIENTE_DE_LA_AIR`; con T = 64 lo tocado dice el testigo, con T = 128 no ⚠️ §915: medida, 23 familias sí: T ≥ 128 ≥ 90 en todas (`INDEPENDIENTE_DE_LA_AIR`) |
| H12 (§900): las filas aleatorias de una columna que entra en C linealmente y con coeficiente público son una máscara más, M_lin: uniformes en un núcleo que no depende del testigo una vez condicionadas a S_toc, con imagen lineal y pública en D | medida: qué columnas lo son, en cada canon, AIR a AIR; y R3 con M_lin, por familia; ⚠️ §907: demostrada como lema, con el núcleo conjunto que descuenta lo abierto de los trozos (L7) | L3, L7 | `instrumento_mlin`: `CENSO_134` y `MLIN_MEDIDO` (apartado 14) ⚠️ §915: instrumento de guarda; la proposición del apartado 18 no la necesita |
| H13 (§905): Λ, lo que C ve en S_ab de las columnas lineales condicionadas a lo visto, es sobreyectiva: C(S_ab) uniforme e independiente del testigo sin pasar por S_toc | medida, por familia, en las que no caben con la gruesa: sólo en el umbral conjunto | L1' | `instrumento_mlin`: `sobreyectiva`, `FINA_MEDIDO` (apartado 15) ⚠️ §915: instrumento de guarda; la proposición del apartado 18 no la necesita |
| H14 (§907): R3 con M_lin vale en los desafíos reales, no sólo en la instancia medida | demostrada en la capa 0 con T ≥ 512 fuera de las coincidencias coset–tocado, para cualesquiera posiciones, con fallo ≤ 7c₀/\|F\| (L8); medida en lo demás, en tres instancias por familia —la semilla 2, la 3 en la solvencia, y una adversa con una colisión de cosets, una coincidencia y una posición siguiente de otra—, con la huella de cada composición clavada | L7, L8 | `instrumento_mlin`: `adversa`, `MLIN_INSTANCIAS`, `HUELLA_C` (apartado 16) ⚠️ §915: instrumento de guarda; la proposición del apartado 18 no la necesita |

## 5. El simulador

S recibe la AIR, las entradas públicas, el contexto (T, W, W_aux, m, N, opciones) y nada del
testigo.

- **S1. Los desafíos.** Muestrea, uniformes: los elementos aleatorios del tramo auxiliar si lo hay,
  los α de las restricciones, z ∈ E (vuelve a muestrear si viola H3), los cc de DEEP, los α_i de
  FRI, y 42 posiciones de Λ con repetición, de las que toma las únicas; el nonce, con la misma
  búsqueda que el probador. En el protocolo no interactivo, programa el oráculo para que la moneda
  los dé (L0).
- **S2. La traza en los puntos tocados.** Para cada columna principal real y cada auxiliar,
  muestrea uniformes sus valores en S_toc (F o E); para la columna aleatoria, en S_ab. Un punto que
  aparece dos veces se muestrea una vez. Válido por L1 si 2q + 6 ≤ T (H11).
- **S3. El cociente en los puntos abiertos.** Calcula C(y) para y ∈ S_ab con las fórmulas públicas
  del verificador (`evaluate_constraints`) sobre los marcos de S2 —(y, g·y) para y = x_j, (z, z·g)
  y (z·g, g²·z)— y muestrea (h'_k(y))_k uniforme en el hiperplano Σ_k y^{k·s}·v_k = C(y). Válido
  por L2 (H4).
- **S4. Lo determinado de DEEP.** d_j = D(x_j) con la fórmula de P7 sobre S2, S3 y los cc;
  d₀ = D(x₀) con la fórmula de L3.
- **S5. DEEP y FRI.** Muestrea D̃ uniforme en {D ∈ E[x]_{≤L−2} : D(x_j) = d_j, D(x₀) = d₀},
  condicionado a grado exactamente L − 2 como el probador aserta (diferencia 2^{−128}), y corre FRI
  honesto sobre D̃ en Λ con los α_i de S1: capas, resto, cosets. Coincide con la vista real si R3
  (H6), que es suficiente. ⚠️ §907 (la O7 de la revisión del agente): el aserto es
  `assert_eq!(trace_length - 2, deep_composition_poly.degree())` en `generate_proof_oculto`
  (`crates/winter-prover/src/lib.rs`), heredado del `generate_proof` de winterfell: con
  probabilidad ≈ 2^{−128} un testigo VÁLIDO hace pánico al probador, contra la letra de la D-AH del
  RFC-0009 —ni siquiera es un testigo malo—; no toca la ocultación, y es la 137 de `BACKLOG.md`.
  ⚠️ §911: cerrada: es `ProverError::GradoDeDeep`, un `Err` con nombre, en los dos caminos; y el
  suceso existe sin esperar a 2^{−128}: una traza degenerada en claro, de columnas constantes, lo da
  siempre, y oculta nunca, porque las filas aleatorias no son constantes.
- **S6. Los compromisos.** Para la traza, el cociente y cada capa de FRI: un árbol cuyas hojas
  abiertas son las de S2, S3 y S5 con sal fresca, y cuyas hojas no abiertas son digests uniformes
  (L4). El resto de FRI, entero, es el de S5.
- **S7. La transcripción.** V1 … V8 con lo anterior. El verificador la acepta: la consistencia OOD
  por S3, D(x_j) por S4, FRI por S5, las aperturas por S6, la molienda por S1.

**Por qué la distribución es la real** (bajo H1 … H11): los desafíos son uniformes en los dos
lados; los valores tocados, uniformes e independientes por columna (L1), y el verificador real ve
menos que el simulador —los tocados que no se abren—, así que igualar la vista extendida iguala la
vista; los trozos, uniformes en su hiperplano dado C(y) (L2), con C(y) la misma función de los
tocados en los dos lados; lo que FRI abre, uniforme en el mismo espacio afín si R3 (L3); las hojas
no abiertas, indistinguibles (L4); y las máscaras reales son salidas de la moneda, indistinguibles
de uniformes (L5). Todo salvo L1 y L2 descansa en el modelo de L0 y L4: el enunciado es de
ocultación **estadística en el modelo de oráculo aleatorio programable**, con ventaja acotada por
el 2^{−60} de H3 más las consultas al oráculo entre 2^{−256}, y **computacional fuera de él**, con
Blake3 en el lugar del oráculo. ⚠️ §907 (las O1 y O4 de la revisión del agente): la cuenta entera
está en el apartado 16: el 2^{−128} de S5 se suma, la caída de rango sobre los desafíos se acota
con |F| en el denominador y no con |E|, y lo que depende de las posiciones tiene lema (L8) en la
capa 0 con T ≥ 512 y medida, no cota, en lo demás.

## 6. La condición R3, como instrumento

**O**, filas: para cada posición única x_j, los ocho puntos de su coset de plegado en Λ,
{x_j·μ : μ^8 = 1} —D evaluado ahí: 8c₀ filas sobre E, de las que las de las posiciones mismas son
lo determinado—; para cada capa i ≥ 1, los ocho valores del coset de la posición plegada, cada uno
un funcional lineal de D: el pliegue de winter-fri con α_i aplicado a los cosets de la capa
anterior (`apply_drp`); y el resto, sus coeficientes, funcionales lineales de D (la interpolación
de la última capa). El rango de O es el de L3, no su número de filas. **M**, columnas, sólo las dos
fuentes que entran en D de forma lineal e independiente del testigo: la columna aleatoria,
cc_R·Q[L_r] para cada una de sus L filas, restringido al núcleo de la evaluación en S_ab
(L − q − 4 columnas sobre F tras el cambio de base); y cada aleatorizador r_k,
(cc'_k·Q[x^{s+d}] − cc'_{k+1}·Q[x^d])_{d<m}, restringido al núcleo de la evaluación en S_ab
(2(m − q − 2) columnas sobre F). Las filas aleatorias de las columnas reales y auxiliares no van en
M (L3). **La comparación**: rango_F(O∘M) frente a rango_F(O|_K), con una base de K construida de la
de E[x]_{≤L−2} con las q + 1 condiciones lineales. Iguales: R3 se cumple para esos desafíos. Un
atajo suficiente: si rango_F(O∘M_R) = rango_F(O|_K) con M_R la columna aleatoria sola, R3 vale por
ella sola; con T ≥ 512 la cuenta de dimensión lo permite (978 frente a unas 758; 2002 frente a unas
1014), con T = 256 no (466 frente a unas 630). ⚠️ §898: la cuenta lo permitía y la estructura no:
medido, rango_F(O∘M_R) es 294 + 2·(L/8 − 42) —338, 466 y 722 con T = 256, 512 y 1024—, una
dimensión por cada uno de los 7q puntos de la capa 0 y dos por cada funcional nuevo de la capa 1,
porque R(x) ∈ F en cada punto de Λ.

Como en L1 y L2, el rango para unos desafíos que elige el test es la instancia medida. Que valga
para los reales: sobre z, cc y α_i, una caída de rango es una condición polinómica (un menor que se
anula) y su probabilidad se acota por el grado sobre |E|; sobre las posiciones no es polinómica, y
el instrumento tiene que probar varias elecciones y decirlo. ⚠️ §907: «sobre |E|» era un error,
y la revisión del agente lo dio por bueno (la O4 del apartado 11): la aplicación cuyo rango se
mide es F-lineal —u y w viven en F[x], porque la columna aleatoria y las columnas reales viven en
F— y su matriz sobre F lleva las dos coordenadas de cada entrada de E, así que un menor es un
polinomio en las coordenadas sobre F de los desafíos y Schwartz–Zippel acota su anulación por
grado/|F|, con |F| = 2^64. Para unas posiciones fijas en las que la instancia medida da el rango
pleno, el menor entero —tamaño ≤ 1016, grado acotado por una decena por entrada— da del orden de
2^{−51}; L8 (apartado 16) lo hace por puntos y lo baja a 7c₀/|F| ≈ 2^{−55,8} en la capa 0 con
T ≥ 512, para cualesquiera posiciones. El denominador es de la construcción, no del argumento:
con la máscara de DEEP en E (la 133) todo sería E-lineal y la cota iría sobre |E|. Y sobre las
posiciones: medido en más instancias, una adversa, apartado 16. El coste es pequeño: M tiene a lo sumo
L − q − 4 + 40·(N − 1) columnas —2282 con T = 1024 y N = 8— y O a lo sumo 2·(8c₀ + 8c₁ + resto)
filas sobre F, 1408; construir O∘M y eliminar cabe en el sello. Lo que el instrumento de A3 tiene
que imprimir, por familia: rango_F(O), rango_F(O|_K), rango_F(O∘M) y rango_F(O∘M_R), y el veredicto.
⚠️ §898: hecho, `crates/zk-ssl/src/instrumento_r3.rs`, con dos semillas de desafíos que dan los
mismos rangos, y en el sello por geometría de producción (apartado 13). ⚠️ §907: la segunda
semilla de T ≥ 256 era una corrida de exploración que no estaba en el árbol (la O1); desde el
§907 está, con M_lin, en `MLIN_INSTANCIAS` (apartado 16).

## 7. Lo que el argumento no cubre

- Las cinco familias con T = 64 (L6, H11): NullifierThresholdAir y ThresholdAir, que son del cable,
  RefundAir, RefundAirV2 y EdadAir con ocho hojas. ⚠️ §914: ya no hay familias con T = 64: las cinco pasan a 128 en el corte, y L1 las cubre (H11 se cumple en las 23).
- La solvencia (SolvencyAir, T = 256 con dos trozos): R3 le falla por cuenta de dimensión (L3), y
  no tiene argumento.
- R3 (H6), en las otras diecisiete, hasta que A3 la mida. ⚠️ §898: medida, falla en todas: hoy el
  argumento no cubre ninguna familia.
- La dependencia de los trozos del cociente en las filas aleatorias de las columnas reales fuera de
  los puntos abiertos (L3): no es lineal ni independiente del testigo, y este documento no la usa
  como máscara; donde R3 se cumple no estorba, y donde falla es lo que queda.
- La solidez: este documento no la toca; el probador oculto sigue probando lo que decía (RFC-0009
  D-H, y la foto de D-R).
- Lo público por diseño (P12) y lo que el nodo ve por ser nodo (RFC-0009, «Seguridad»).
- La semilla repetida (H9), la sal repetida (H8), un `OsRng` predecible: fuera del modelo,
  `SECURITY.md` §3.11.
- Blake3 fuera del modelo de oráculo aleatorio, y un adversario cuántico: «nivel cuántico: no
  medido», sin cambio.
- Los canales laterales del probador (tiempo, memoria) y lo que la capa, el SDK o el kit revelen
  fuera de los bytes de la prueba.
- Las pruebas de la 0.3 (`spec/vectors/0.3/`), que no ocultan, y las dos del catálogo de rechazos
  (RFC-0009 D-P).
- Las posiciones y el z reales de cada prueba (H3 se supone, no se mide), y los desafíos reales de
  R3 (apartado 6). ⚠️ §907: para R3, L8 cubre la capa 0 con T ≥ 512 fuera de las coincidencias
  coset–tocado para cualesquiera posiciones; la capa 1, esas coincidencias y T = 256 quedan en la
  medida (apartado 16).
- La auditoría ajena del fork y de la construcción (H7 de `SECURITY.md`): este documento la pide,
  no la sustituye.

## 8. El enunciado

**Proposición (condicional).** Bajo H1 … H11 —en particular R3 (H6), suficiente y no medida, y
T ≥ 2q + 6 (H11)— existe un simulador S, el del apartado 5, tal que, para toda traza real que el
AIR interno acepta, la vista del verificador honesto del protocolo oculto y la salida de S son
estadísticamente indistinguibles en el modelo de oráculo aleatorio programable, con ventaja menor
que 2^{−60} más Q·2^{−256} para Q consultas al oráculo. Puede enunciarse, a falta de medir R3, para
diecisiete familias. **Para las cinco familias con T = 64 no se enuncia**: les falta L6 o T = 128.
**Para la solvencia tampoco**: R3 le falla por cuenta (L3, la 132). ⚠️ §898: **con R3 medida y
falsa en las cinco geometrías, hoy no se enuncia para ninguna familia**; vale como enunciado de lo
que la construcción tendría si la máscara de DEEP viviera en E (la 133), no de lo que tiene.
⚠️ §899: con la construcción de laboratorio de la 133 —dos columnas aleatorias—, R3 se cumple en
las cinco geometrías (apartado 13), y la proposición vale para ella, bajo las demás hipótesis,
para las dieciocho familias con T ≥ 2q + 6, la solvencia incluida; no para las cinco con T = 64
(L1). Es lo que A4 podrá enunciar cuando el corte la encienda, y no antes: hoy ninguna prueba de
producción la lleva. ⚠️ §900: con R3 medida CON M_lin (H12, apartado 14), la proposición vale
para la construcción de hoy, bajo las demás hipótesis, para **dieciséis familias**: las seis de
T = 1024, las nueve de T = 512 y la solvencia; **no** para MerkleAir ni FrozenClimbAir (T = 256, sin
ninguna columna lineal: su R3 es la del §898) ni para las cinco con T = 64 (L1). Es lo que A4 puede
enunciar, con estas palabras y con las siete fuera por su nombre. **Nada de esto es una promesa
de la casa**: A4 decidirá, con A3 sellada, qué se promete y para qué familias, y hasta entonces la
promesa es el censo. ⚠️ §907: la revisión del agente (apartado 11, la tercera;
`doc/ocultacion-hvzk-revision-agente.md`) no encontró un paso falso y dejó el hueco de su O1: lo
medido era una instancia de desafíos por familia, y las posiciones reales no son genéricas. Con el
§907 la proposición se enuncia con la cuenta del apartado 16: ventaja ≤ 2^{−60} (H3) + 2^{−128}
(S5) + Q·2^{−256} (L0) + ε_R3, con ε_R3 ≤ 7c₀/|F| ≈ 2^{−55,8} para lo que L8 demuestra —la capa 0
con T ≥ 512, fuera de las coincidencias coset–tocado— y, para el resto —la capa 1, las
coincidencias y la solvencia entera—, lo que dicen las instancias medidas, que no es una cota.
Sigue sin ser una promesa.
⚠️ §915: la proposición para las veintitrés con la construcción de `zkssl/0.5` —todas, la solvencia y las cinco de T = 128 incluidas, sin excepción por su nombre— es la del apartado 18, con la cuenta de L9; lo de arriba describe `zkssl/0.4`. Sigue sin ser una promesa.

## 9. Lo medido en A1 y lo que A2 corrige

| el §896 decía | lo que es | dónde |
|---|---|---|
| 45 a enmascarar por columna y por trozo: 44 vistas más una «dirección residual» | por columna, q + 4 = 46 dimensiones sobre F; por trozo, q + 2 = 44 sobre E; la residual es pública, no una abertura | L3; el instrumento, `Aberturas`; la nota de la D-I |
| la columna aleatoria enmascara DEEP en todas las direcciones menos una | enmascara una imagen de dimensión L − q − 4 sobre F de un espacio de 2(L − 1); la uniformidad de lo que FRI abre es R3, sin medir | L3 |
| la cota más ajustada es T = 64, 45 frente a 64, diecinueve de holgura | la cuenta por columna es la de lo tocado, 2q + 6 = 90, y T = 64 no la cumple: cinco familias sin argumento independiente de la AIR | L1, L6, la 131 |
| el rango de la Lagrange, sobre E | sobre F, porque las filas aleatorias principales viven en F: 46 ≤ T, no 44 | el instrumento, `rango_f` |
| m = 43 cae con q ≥ 41 | con q + 2 ≤ m, cae sólo con q = 42, medido en el §897 (23 rojas con q = 42, cinco verdes con q ≤ 41); m ≤ 41 cae con toda prueba de producción | L2 |

Lo medido en el §897, con el instrumento corregido (`cargo test -p zk-ssl --release --locked
instrumento_cota`): 28 pruebas de 23 AIR, 23 probadores que encienden; en todas, m = 64 y 44 puntos
abiertos por objeto con q = 42 (45 y 43 en la que abrió 41), 46 dimensiones sobre F por columna
frente a T de 64 a 1024, 44 sobre E por trozo frente a m = 64, rango pleno en los dos; tocados 90
frente a T: cubre, con rango 90 de 90, en las 19 pruebas con T ≥ 256; NO cubre, con rango 64 de
90, en las 9 con T = 64. Anchos, con la columna aleatoria: de 6 (SolvencyAir) a 84 + 6 auxiliares
(EdadAir); trozos, 2 (SolvencyAir), 8 (T ≥ 512), 9 (MerkleAir, FrozenClimbAir) y 15 (T = 64). El
juguete, T = 128: 46 frente a 128, 44 frente a 64, 90 tocados cubiertos con rango 90 de 90, 4
trozos, una capa de FRI y resto de 32, y verifica; sus rangos los calcula el instrumento en el
campo de producción, con su extensión, sobre la geometría del juguete, que vive en f128 sin
extensión —allí z ∈ F y lo abierto por columna serían q + 2—: es una medida de la geometría, no del
juguete.

## 10. Dónde el fork se aparta de la nota 2024/1037

Lo que la casa sabía de la nota por el RFC-0009 (D-G, D-I): el cociente se parte con paso 2T − m y
sus trozos se aleatorizan con polinomios de grado < m que se cancelan en la suma (su apartado 4.2,
el único que el RFC cita), con m ≥ 44 por «las 42 consultas, z y z·g», que es la cuenta de L2 sobre
E; las filas aleatorias y el polinomio aleatorio antes de FRI vienen del issue 9 de winterfell, y la
sal de las hojas también. Lo que devuelve una búsqueda de su página: que trata la ocultación de los
argumentos univariados con FRI como compromiso sobre campos pequeños; que se detiene en dos
técnicas, «la aleatorización con polinomios sobre el campo base» y «partir el cociente en polinomios
de grado menor»; que revisita la construcción de 2019 que cita y subraya su paso de enmascarado; y
que la descomposición del cociente es una fuente conocida de errores, en la literatura y en el
código. Con eso, lo que se puede decir, y lo que no:

- **El cociente** (L2): la construcción del fork es la que el RFC describe y la cuenta q + 2 ≤ m es
  la que la nota pide según el RFC. Pendiente: contrastar con el texto si la nota cuenta también
  los trozos en z·g (`QuotientOodFrame`, que es de winterfell 0.13 y no del issue).
- **La máscara de DEEP** (L3): el fork enmascara con una columna aleatoria de la traza, R ∈ F[x]_{<L},
  que entra en DEEP por el cociente, cc_R·Q[R]: L − q − 4 grados de libertad sobre F. Un polinomio
  aleatorio del grado de D sumado a D —lo que el issue 9 llama «el polinomio aleatorio antes de
  FRI» y lo que el resumen de la nota llama el paso de enmascarado de la construcción de 2019—
  tendría 2(L − 1) y haría R3 automática. Si la nota construye la máscara como una columna del
  campo base, la cuenta de R3 es suya también; si la suma a D, el fork se aparta de ella aquí, y
  R3 es el precio, que en la solvencia no se paga: con dos trozos, R3 le falla por cuenta (L3).
  **Pendiente de leerla.** ⚠️ §898: el precio no se paga en ninguna familia: la máscara en F frente
  a un DEEP en E deja una coordenada de cada dos sin cubrir en todo lo que FRI abre. Que la nota
  trate, según su resumen, «la aleatorización con polinomios sobre el campo base» hace más
  urgente leerla: es exactamente esto.
- **Lo tocado** (L1): la cuenta de la D-I, 44 por columna frente a T, no contaba g·x_j ni g²·z; si
  la nota los cuenta, la D-AG se decidió con una cuenta menor que la suya. Pendiente.
- **La sal** (L4): el issue la pide para la traza; el fork la pone en la traza, los trozos y FRI.
  Qué pide la nota, pendiente.
- **La extensión**: el fork saca z y los aleatorizadores de E y las filas aleatorias principales
  de F; la cuenta sobre F de L1 (dos dimensiones por punto de E) es consecuencia de eso, y si la
  nota la hace, pendiente.

## 11. Objeciones y cierres

La revisión escéptica del §897 —un agente de la sesión con el documento, el fork, winter-fri y el
instrumento delante, sin las conclusiones— puso ocho objeciones, dos altas. Cada una, y cómo quedó:

1. **(alta) Las filas aleatorias de las columnas reales no son una máscara lineal de D.** Entran en
   los trozos del cociente a través de C, con el grado de cada restricción, y su parte lineal
   depende del testigo; en los puntos abiertos L1 y L2 lo absorben, en los x_j·μ y en las capas no.
   La primera versión las contaba en M y enunciaba R3 como «si y sólo si». **Cerrada**: M son la
   columna aleatoria y los aleatorizadores, R3 es suficiente y no necesaria, y si falla lo que queda
   depende de la AIR y del testigo (L3, apartado 6, apartado 7).
2. **(alta) El rango de O no es su número de filas.** Las capas pliegan: la capa 1 aporta a lo sumo
   L/8 dimensiones sobre E y el resto ninguna. Con T = 256, 716 sobre F y no 1276, y FRI no entrega
   D entero. Con las cuentas buenas, la solvencia —T = 256 con dos trozos— tiene 506 dimensiones de
   máscara frente a unas 630 sin determinar: R3 le falla por cuenta. **Cerrada**: rangos en lugar de
   filas en L3 y en el apartado 6, los «frente a» rehechos, la solvencia fuera del enunciado del
   apartado 8 y en la 132.
3. **(media) La cabecera y el enunciado sobredimensionaban.** «Una condición» que era «si y sólo
   si» y «dieciocho familias». **Cerrada**: diecisiete familias, R3 suficiente, la solvencia aparte,
   y la no linealidad de los trozos en el apartado 7.
4. **(media) La «recuperación por álgebra lineal» del juguete con T = 64 no estaba justificada.**
   Despejar P(g·x_j) de C(x_j) vale, pero 87 evaluaciones de una columna de grado < 128 son 87
   ecuaciones sobre 128 incógnitas, y la ligadura que sobra es cúbica. **Cerrada**: la fuga con
   T = 64 es distribucional y el falsador de A3 es un distinguidor; la recuperación lineal se queda
   para T = 32 con una columna constante (L6, apartado 12).
5. **(baja) La molienda.** La búsqueda del nonce corre sobre salidas del oráculo que el simulador
   no fija; lo programable son las de después, de las que salen las posiciones. **Cerrada** en L0.
6. **(baja) Los rangos del juguete** los calcula el instrumento en el campo de producción, con su
   extensión, y el juguete vive en f128 sin ella. **Cerrada**: dicho en el apartado 9.
7. **(baja) «Medida» en H1** podía leerse como medida sobre las posiciones reales. **Cerrada**: lo
   medido es la cuenta; el rango es una prueba de vida sobre puntos del test (L1, tabla).
8. **(baja) Una cifra y una cita.** «46 de 46 en las 28 con q = 42» no contaba la prueba que abrió
   41, y el apartado 11 remitía a un asiento que aún no llevaba las objeciones. **Cerrada**: la
   cifra, en L1; las objeciones, aquí y en el asiento.

**La revisión del §898**, sobre el instrumento de R3, el apartado 13 y los falsadores, con el
código de winter-fri y del fork delante; dio por correctos O (el mapa exacto del probador de FRI),
las dos máscaras, K, la comparación de rangos, la afirmación estructural de una dimensión por punto
y los tres falsadores, y puso seis objeciones:

1. **(alta) M deja fuera una máscara lineal que existe en la solvencia.** La objeción 1 del §897
   vale para columnas que entran en alguna restricción con grado ≥ 2 o con coeficientes del
   testigo; en `SolvencyAir` cuatro columnas entran linealmente en todas, con coeficientes
   constantes o periódicos, y sus filas aleatorias son una máscara lineal independiente del testigo
   que el instrumento no cuenta y que puede cubrir lo que falta. **Cerrada**: el veredicto se
   enuncia como «R3 con M = {columna, aleatorizadores} no se cumple», la cabecera del instrumento y
   este apartado lo dicen, y medir M_lin por AIR es la 134. ⚠️ §900: medida (apartado 14): la
   máscara existe en dieciséis familias y cubre todo lo que falta; en la solvencia, 632 de 632.
2. **(media) El apartado 13 afirmaba más de lo medido**: «H6 es falsa, el simulador no reproduce …
   en ninguna familia», y «faltan exactamente 250, la segunda coordenada», que es una inferencia.
   **Cerrada**: reescrito como arriba, y la inferencia dicha como inferencia.
3. **(media) Lo sellado era una sola instancia de desafíos**, semilla 1 y q = 42, mientras el texto
   decía «dos semillas» y «q = 40 y 41». **Cerrada**: en el sello, la geometría de T = 64 corre
   además con q = 40 y 41 y con la semilla 2; lo demás se dice como corrida de exploración, fuera
   del árbol. Y «las posiciones y z no viajan» era inexacto: no se rehacen en el instrumento de la
   cota, que no tiene el AIR; en los falsadores sí.
4. **(baja) Sin autocomprobación de inclusión** en `mide`. **Cerrada**: tres asertos, cada máscara
   dentro de las dos juntas y las dos dentro de K.
5. **(baja) La cabecera de los falsadores decía 92 ecuaciones** y son 91. **Cerrada**.
6. **(baja) Lo que valida la moneda rehecha** no es el aserto de las posiciones únicas sino la
   consistencia del primer ataque, y `siguiente_de` divide por una pendiente que sería 0 con una
   constante secreta 0. **Cerrada**: dicho en la cabecera y en el apartado 13, y un aserto en la
   pendiente.

**La revisión del §907**, de un agente externo a la sesión —otra instancia, sin la crónica ni las
conclusiones, con el dosier, una copia del árbol del §903 (`git archive`) y las fuentes de
winter-fri y winter-crypto 0.13.1 del registro de cargo delante, en lectura pura, sin ejecutar
nada; el informe entero, con lo que leyó y en qué orden, lo que dio por correcto y con qué, y lo
que el dosier no le dejó responder, es `doc/ocultacion-hvzk-revision-agente.md`—. No es la
revisión ajena de la 136, que es de una persona que elige el autor; es la tercera de la casa, más
lejos de la sesión que las del §897 y el §898. No encontró un paso falso ni una medida que mida
otra cosa de lo que dice, y puso siete objeciones, ninguna alta:

1. **(media) R3 con M_lin estaba medida en UNA instancia de desafíos por familia, y las posiciones
   reales no son genéricas en el sentido que el argumento necesita.** Las 42 posiciones caen en
   |Λ|/8 cosets de plegado: dos comparten coset en la capa 0 en el 57 % de las pruebas de la
   solvencia (34 % con T = 512, 19 % con T = 1024); con frecuencia Θ(1) por prueba un punto del
   coset de una posición coincide con un tocado de otra, x_j·μ = g·x_k, y ahí el camino directo de
   toda columna lineal vale cero; la instancia clavada tenía 42 cosets distintos, y la cota de la
   proposición no tenía término para las posiciones. **Cerrada a medias** (apartado 16): la
   solvencia se mide en cada canon con las semillas 2 y 3 enteras y con una instancia adversa que
   fuerza una colisión, una coincidencia y una posición siguiente de otra; las otras quince, con
   la semilla 2 y la adversa, en el test largo; L8 demuestra la cobertura de la capa 0 por puntos
   para cualesquiera posiciones con T ≥ 512, fuera de las coincidencias; la capa 1, las
   coincidencias y T = 256 siguen medidas y no acotadas, y la proposición lo dice (apartado 8).
2. **(media) H12 se aplicaba como el instrumento la calcula, pero el lema que lo justifica no
   estaba escrito**: las filas aleatorias de una columna lineal mueven las aperturas de los
   trozos, y «una máscara más de L3» no decía por qué condicionar a esas aperturas no rompe la
   independencia; un lector que compruebe M_lin contra la definición de L3 la rechaza por la
   objeción 1 del §897. **Cerrada**: L7, el núcleo conjunto, apartado 16.
3. **(media) Lo que el canon medía de la 134 era el censo y la solvencia; el rango de las otras
   quince familias era una foto**: un cambio en las restricciones de una AIR que conserve las
   cuentas del censo no movía ninguna guarda. **Cerrada**: la huella de la composición de cada
   familia, clavada (`HUELLA_C`) y comparada en cada canon; un cambio obliga a remedir la 134
   (apartado 16).
4. **(baja) La cuenta de la ventaja estaba incompleta**: el 2^{−128} de S5 no se sumaba, el grado
   del menor no se decía y las posiciones no tenían término. **Cerrada** (apartado 16), y con una
   corrección que la revisión no vio: la cota sobre los desafíos va sobre |F|, no sobre |E|
   (apartado 6).
5. **(baja) Dos imprecisiones de la vista**: la hoja salada es `merge(hash_elements(fila), sal)`,
   no Blake3(fila ‖ sal); y el compromiso del resto y `num_unique_queries` no estaban en V1–V8.
   **Cerrada**, en P5 y en el apartado 2.
6. **(baja) L0 describía la molienda del rasgo `concurrent` apagado, sin decirlo.** **Cerrada**,
   en L0.
7. **(baja) El grado del DEEP se aserta con pánico, en producción**: con probabilidad ≈ 2^{−128}
   un testigo válido hace pánico al probador, contra la D-AH. **Cerrada a medias**: dicho en S5, y
   la 137 de `BACKLOG.md` para el `Err`, en un sello aparte, porque toca el camino de producción
   del fork.

Y dos cosas que vio al leer, fuera del dosier: que una `Assertion::periodic` de una AIR interna se
exigiría también en las filas aleatorias —hoy ninguna de las 23 la usa; el probador oculto fallaría
en el aserto de `segmentar`, sin fugar nada—, que es la 138; y que la corrida de exploración del
§898 con la semilla 2 para T ≥ 256 no estaba en el árbol: desde el §907 la segunda semilla está,
con M_lin, en `MLIN_INSTANCIAS`. Sobre la 135, por cuenta y sin medir, dijo que como mucho
rescataría a una de las cinco, y señaló la autorización delegada; el §905 midió que a ninguna, y
que la única con Λ sobreyectiva es el umbral conjunto, que la cuenta del agente descartaba: la
cuenta no basta, el rango de Λ decide.

Lo que la revisión del §907 dio por correcto, con el código delante: P1, P2, P4, P5/L4, P6/P11, P7,
P8/P10/V8 y P9/L0 frente a las fuentes del fork y de winter-fri; la vista entera frente a la
`Proof`; L3, con un argumento cerrado de que D(x₀) es lo único público de D fuera de las posiciones
—el residuo de un solo trozo recorre todo K—; el rango de O; `instrumento_r3` frente al apartado
6, con las fórmulas de las dos máscaras deducidas aparte y coincidentes; de `instrumento_mlin`,
que la composición es línea a línea la del verificador y no una réplica del probador, la sonda, la
construcción de M_lin, el núcleo conjunto y la inclusión; que en las dieciséis AIR la forma de C no
depende de las entradas públicas; los falsadores y la moneda rehecha; L5; la cuenta de lo tocado; y
el orden del simulador frente a Fiat–Shamir, prefijo a prefijo.

Lo que la revisión del §897 dio por correcto, con el código delante: el orden de los compromisos y
de los desafíos, las dos monedas y sus semillas (P1, P4, P6, P7); `segmentar` y el número de trozos; la
demostración de L1 entera, con las exenciones en las últimas filas; L2; Q[P](x₀) y que D(x₀) es
público; que K es el espacio correcto y que las dos máscaras lícitas caen en él; lo que FRI abre y
cómo; la sal y los tests que se citan; el simulador, paso a paso, y que nada del `Proof` queda sin
simular salvo lo de la objeción 1; y que el documento no afirma HVZK en ningún sitio.

## 12. Lo que pide a A3, A4 y A5

- **A3** (⚠️ §898: hecha, apartado 13; lo que sigue es lo que pedía), en este orden: (1) el
  instrumento de R3 del apartado 6, por familia, en el sello o en
  `--largo` con su pin; (2) el falsador de la recuperación por álgebra lineal, en el juguete: con
  T = 32 y una columna constante, las q + 4 ecuaciones de L1 sobre T + 1 incógnitas devuelven la
  constante; y para la 131, con T = 64 en el juguete, un distinguidor y no una recuperación: de
  C(x_j) se despeja P(g·x_j), como dice L6, pero las 2q + 3 evaluaciones de una columna de grado
  < 2T son 87 ecuaciones lineales sobre 128 incógnitas, y la ligadura que sobra, x_{i+1} = x_i³ + 42,
  es cúbica; lo que se mide es que la vista cambia con el testigo, no que el testigo salga; (3) la
  semilla repetida en dos pruebas, en sello (H9); (4) el distinguidor estadístico sólo si cabe en
  `--largo` con su pin, y si no, declarado fuera.
- **A4**: la promesa nueva sólo con R3 medida en verde y con las cinco familias de la 131 dichas
  aparte; las palabras, las del apartado 8 y no más; H7 conservada hasta una revisión ajena; «nivel
  cuántico: no medido», intacto. ⚠️ §898: R3 está medida y en rojo en todas: A4 no puede enunciar
  HVZK para ninguna familia con el argumento de este documento. Lo que A4 puede hacer es decir eso
  con las palabras de este documento —el censo sigue, y lo que falta y por qué— y dejar la promesa
  de HVZK para lo que la 134 mida AIR a AIR o para el corte que lleve la 133. ⚠️ §899: la 133 está
  medida y construida en laboratorio (apartado 13): A4 puede decir qué tendrá la construcción del
  corte, con R3 en verde, y qué tiene la de hoy, sin él. ⚠️ §900: la 134 está medida (apartado
  14): con la construcción de hoy, R3 se cumple con M_lin en dieciséis familias y no en siete,
  con nombre; A4 puede enunciar para las dieciséis lo que este documento demuestra, con sus
  hipótesis, y decir de las siete lo que les falta: la 133 a MerkleAir y FrozenClimbAir, y la
  131 —o la 135— a las cinco de T = 64.
  ⚠️ §915: el corte está sellado (§914): A4 puede enunciar para las veintitrés lo que el apartado 18 demuestra con la construcción de `zkssl/0.5`, con sus hipótesis y sin ninguna familia fuera por su nombre; las palabras están en la D-AM del RFC-0009 (el borrador del §915), y la promesa sigue siendo el censo hasta que el autor la acepte.
- **A5**: la sal es lo que hace L4 incondicional. Sin ella, (a) necesita que cada hoja no abierta
  tenga entropía frente al oráculo —la fila entera de la traza en un punto de Λ la tiene por L1, el
  vector de los trozos por L2, y los ocho valores de un coset de FRI sólo por R3— y (b) desaparece.
  La recomendación de este documento al autor es conservarla hasta que R3 esté medida, y decidir
  después con la cuenta de las hojas delante. ⚠️ §902: decidida por el autor: se conserva.

## 13. A3, medido (§898)

**El instrumento de R3** (`crates/zk-ssl/src/instrumento_r3.rs`) construye, para una geometría
(q, T, m, N, capas, resto) y unos desafíos de una moneda Blake3 sembrada con un número, lo que el
apartado 6 pide: O con el pliegue de winter-fri (`apply_drp`, `fold_positions`, el mismo offset),
K, M_R y M_r con sus fórmulas de L3, y los rangos sobre F por eliminación, sobre combinaciones
aleatorias con más filas que O (Schwartz–Zippel sobre 2^64), y con los asertos de inclusión que
la construcción promete (cada máscara dentro de las dos juntas, y las dos dentro de K). Corre en el
sello, dentro de `la_cota_de_ocultacion_de_los_23_probadores_con_fila`, una vez por geometría de
producción con q = 42 y la semilla 1, contra sus cifras clavadas (`R3_MEDIDO`), y en la geometría de
T = 64, además, con q = 40 y 41 y con la semilla 2, con el mismo veredicto. Las demás semillas y
consultas de las otras geometrías —los mismos rangos con la semilla 2— y el rango de O entera, que
informa y no decide, se midieron en la corrida de exploración del §898 y no están en el árbol.

| geometría (T, trozos, capas, resto) | familias | rango de O (E) | sin determinar, O sobre K (F) | columna | aleatorizadores | juntas | R3 |
|---|---|---|---|---|---|---|---|
| 64, 15, 1, 16 | las cinco de T = 64 | 127 (D entero) | 168 | 82 | 80 | 126 | no |
| 256, 2, 2, 8 | SolvencyAir | 358 | 632 | 338 | 40 | 378 | no |
| 256, 9, 2, 8 | MerkleAir, FrozenClimbAir | 358 | 632 | 338 | 80 | 382 | no |
| 512, 8, 2, 16 | las nueve de T = 512 | 422 | 760 | 466 | 80 | 510 | no |
| 1024, 8, 2, 32 | las seis de T = 1024 | 550 | 1016 | 722 | 80 | 766 | no |
| 128, 4, 1, 32 (el juguete) | — | 255 | 424 | 210 | 80 | 254 | no |

Todo sobre F salvo la columna del rango de O. Con q = 41 (T = 64): 170 frente a 129. Y la prueba
de vida: con cuatro consultas en la geometría del juguete, 112 frente a 112, R3 se cumple; el
instrumento sabe decir que sí.

**Por qué falla, leído de las cifras.** El rango de O es el de L3 corregido por la revisión
(objeción 2): 8c₀ + L/8 − c₀ sobre E. Lo que la columna aleatoria cubre es 294 + 2·(L/8 − 42):
**una** dimensión por cada uno de los 7q = 294 puntos de la capa 0 que no son posiciones, y dos
por cada funcional nuevo de la capa 1. Es estructural: la columna aleatoria es un polinomio de
F[x], y en un punto x de Λ ⊂ F su contribución a D es R(x)·c(x), un múltiplo en F del número fijo
c(x) = cc_R·ℓ(x)/((x − z)(x − z·g)) ∈ E; el valor D(x) que FRI abre tiene dos coordenadas sobre F, y
la columna sólo mueve una. Los aleatorizadores del cociente cubren a lo sumo 80: entran todos por
los dos huecos x^s·A − B. Con T ≥ 256 faltan 250 = 294 − 44 dimensiones en las tres geometrías;
que sean la segunda coordenada de los 294 puntos de la capa 0 menos lo que los aleatorizadores
alcanzan es una inferencia consistente con las cifras (632 = 2·294 + 44, columna = 294 + 44), no
una medida: el instrumento mide rangos, no el subespacio que falta. Con T = 64, FRI entrega D
entero y faltan 42. ⚠️ Donde este documento decía, en L3 y en
el apartado 6, que «con T ≥ 512 la columna aleatoria sola llega por cuenta», contaba dimensiones
y no estructura: la cuenta era una cota por arriba.

**Lo que significa**, con la precisión que puso la revisión del §898 (apartado 11): lo medido es
que **R3 con M = {columna aleatoria, aleatorizadores} no se cumple** en ninguna geometría de
producción, y por tanto el argumento de este documento no establece la simulabilidad de lo que FRI
abre para ninguna familia. No establece lo contrario: la condición es suficiente, y lo que queda sin
cubrir lo mueven las filas aleatorias de las columnas reales. En una AIR cuyas restricciones toquen
una columna con grado ≥ 2 o con coeficientes que dependan del testigo, esa contribución no es
lineal (objeción 1 del §897); pero en una AIR que toque alguna columna sólo linealmente y con
coeficientes públicos —la solvencia, donde cuatro de sus columnas entran así en todas sus
restricciones—, las filas aleatorias de esa columna son una máscara lineal más, independiente del
testigo, M_lin, que este instrumento no cuenta y que puede cubrir lo que falta: medirla, AIR a AIR,
es la 134. Lo que se puede acotar por cuenta: la parte directa de DEEP de todas las columnas sobre
F, módulo la dirección que la columna aleatoria ya cubre, cabe en un solo polinomio de
Z_real·F[x]_{<T} y da a lo sumo T − 46 dimensiones —210 con T = 256, menos que las 250 que faltan—,
así que con T = 256 sólo la vía de los trozos puede cerrar el hueco, y con T ≥ 512 ni eso está
excluido por cuenta. La solvencia (la 132) y las cinco de T = 64 (la 131, que fallan además por L1)
siguen como estaban. La salida independiente de la AIR que las cifras señalan es una máscara de
DEEP que viva en E: la columna aleatoria en el tramo auxiliar, que winterfell lleva sobre E, o un
polinomio aleatorio de E[x] del grado de D sumado a D, el «polinomio aleatorio antes de FRI» del
issue 9; es la 133 de `BACKLOG.md`, de un corte futuro y nunca de `zkssl/0.4`, y es del autor; la 134
dice antes qué familias la necesitan.

**Los falsadores del juguete** (`crates/zk-ssl/src/instrumento_falsadores_hvzk.rs`): `ConstanteAir`,
una columna constante secreta con una transición de grado 3 y una columna pública con dos
aserciones, en el campo de producción con su extensión y con las consultas, el blowup y el
plegado de `proof_options()`, m = 64 —la mitad cuando T = 32, que no admite más—, sin molienda ni
sal. El adversario es el test: rehace la moneda pública desde los bytes de
la prueba exactamente como el verificador —el contexto y las entradas públicas, las raíces, los
coeficientes de las restricciones, z, el marco OOD, los coeficientes de DEEP, un α por compromiso
de FRI, el nonce y las posiciones— y lee lo abierto. Lo medido, en el sello:

1. **La cota de la traza tiene dientes** (L1). Con T = 32: 45 ecuaciones sobre F (41 posiciones
   únicas, z y z·g) sobre 33 incógnitas —la constante y los 32 coeficientes de R, con la columna
   escrita como c·I_real + Z_real·R, I_real el indicador de las filas reales—, y la constante sale
   por eliminación. Con T = 64: 46 sobre 65, y no sale.
2. **Lo tocado dice el testigo** (la 131). Con T = 64 y dos candidatos, los 85 a 90 valores tocados
   —las posiciones, sus filas siguientes despejadas de C(x_j), z, z·g y g²·z despejado de C(z·g)—
   son consistentes con `Z_real·F[x]_{<64}` para el candidato bueno y no para el malo: la vista dice
   el testigo, en las dos pruebas. Con T = 128, los dos candidatos son consistentes: no dice nada.
   No es una recuperación: es que la distribución cambia con el testigo, como dijo la revisión.
3. **La semilla repetida** (H9). Dos pruebas de dos constantes con la misma semilla de filas
   comparten R: 91 ecuaciones sobre 66 incógnitas devuelven las dos constantes con T = 64, donde
   cada prueba sola (46 sobre 65) no devuelve ninguna. Con semillas distintas, 91 sobre 130: ninguna.

**El distinguidor estadístico** queda declarado fuera: el de arriba es algebraico y decide con
certeza, y para lo que FRI abre la cuestión no es estadística sino de rango, y está medida. Lo
que valida la moneda rehecha de los falsadores no es el aserto de las posiciones únicas, que es
débil, sino el primer ataque: 45 ecuaciones sobre 33 incógnitas sólo son consistentes si z y las
posiciones son las de la prueba.

**Lo que A3 no hace.** No ataca ninguna prueba de la casa ni ningún AIR del cable: el juguete es el
juguete, y lo que dice de las familias reales es lo que dicen sus geometrías. No toca la promesa.
No decide la 131, la 132 ni la 133. No lee la nota.

**§899: la 133, medida y construida en laboratorio.** El instrumento de R3 mide también otras
máscaras (`Mascaras`): k columnas aleatorias sobre F, cada una con su coeficiente de DEEP en E, y
una columna aleatoria sobre E, cuya imagen residual es exactamente K por construcción
(V·(x − z)(x − z·g)·E[x]_{<L−q−2} ↦ cc·V·ℓ·v). Lo medido, sobre F, con la semilla 1:

| geometría (T, trozos, capas, resto) | sin determinar | dos columnas sobre F: columnas / juntas | tres: columnas / juntas | una sobre E: columna |
|---|---|---|---|---|
| 64, 15, 1, 16 | 168 | 164 / **168** | 164 / 168 | 168 |
| 256, 2, 2, 8 | 632 | 632 / **632** | 632 / 632 | 632 |
| 256, 9, 2, 8 | 632 | 632 / **632** | 632 / 632 | 632 |
| 512, 8, 2, 16 | 760 | 760 / **760** | 760 / 760 | 760 |
| 1024, 8, 2, 32 | 1016 | 1016 / **1016** | 1016 / 1016 | 1016 |
| 128, 4, 1, 32 (el juguete) | 424 | 420 / **424** | 420 / 424 | 424 |

Dos columnas sobre F bastan en todas: con T ≥ 256 cubren solas lo que FRI abre sin determinar, y
con T = 64 y en el juguete les faltan cuatro dimensiones que los aleatorizadores ponen. Tres no
añaden nada. La columna sobre E lo cubre sola, como dice su construcción. Se elige la de dos
columnas sobre F (RFC-0009 D-AL): no toca el tramo auxiliar, que winterfell sólo admite con una
restricción y una aserción propias, y es un parámetro, `Ocultacion::columnas`. Lo que hay en el
árbol: con `columnas: 1`, la construcción de `zkssl/0.4` byte a byte; con 2, la de laboratorio,
con la marca `arqueo:oculta:lab` que `verify` rechaza y sólo `verify_laboratorio` acepta; un
falsador en `falsadores_oculta` lo mide, y la cota de las dos columnas está clavada por geometría
en cada canon (`R3_MEDIDO_LABORATORIO`), junto a la de una. Lo que no cambia: L1 y la 131 (las
cinco de T = 64 siguen fuera), L6 y la 134, y la promesa. Lo que la 133 deja al corte 0.5: encender
las dos columnas en los 23 probadores con las columnas en la marca v2 (RFC-0018 E5).

## 14. La 134, medida (§900)

**Lo que mide.** El §898 midió R3 con las dos máscaras que no dependen de la AIR y el §897 dejó una
objeción abierta (apartado 11, la 1 del §898): las filas aleatorias de una columna real cuyas
restricciones la toquen sólo linealmente y con coeficientes públicos —constantes, periódicos, o
los elementos aleatorios del tramo auxiliar, que son del verificador— son una máscara lineal más,
M_lin, independiente del testigo. `crates/zk-ssl/src/instrumento_mlin.rs` la mide, sobre cada AIR
oculta sin saber su tipo: `Sonda`, que `Oculta<A>` implementa para toda `A`, da las anchuras, los
trozos del contexto, las columnas periódicas y la composición C en un punto con un marco,
calculada como la calcula el probador —la combinación de las transiciones, principales y
auxiliares, entre su divisor, más cada grupo de aserciones entre el suyo—. Dos pasos:

1. **La sonda.** Para cada columna real, en tres puntos x de E al azar, con dos marcos al azar, f y
   f', y un desplazamiento δ, para la fila actual y para la siguiente: C es afín en la entrada si
   C(f + 2δe) − C(f) = 2·(C(f + δe) − C(f)), y la pendiente es pública si en f' es la misma que en
   f. Tres veredictos por columna: lineal con coeficiente público; no lineal, grado ≥ 2 en ella;
   lineal con pendiente del testigo, la multiplica otra columna. Schwartz-Zippel sobre E: que una
   no lineal pase por lineal en los tres puntos es un suceso del orden de 2^{−120}. En un juguete
   de cuatro columnas —grado 3; un transporte; lineal con la pendiente de la primera; lineal con
   una pendiente periódica— la sonda da exactamente eso, con dos semillas.
2. **M_lin, y R3 con ella.** De cada columna lineal queda libre, tras condicionarla a los 2q + 6
   tocados de L1, P_ρ = Z_real·V_toc·w —Z_real nula en las T filas reales, V_toc en los tocados y
   sus conjugados, w ∈ F[x] de grado < T − 2q − 6—, y entra en D por dos caminos: el directo,
   cc_i·Q[P_ρ], y el del cociente, C(P_ρ) − C(0), lineal en P_ρ —la pendiente de cada columna en
   cada punto del dominio de restricciones, precalculada—, troceado como `segmentar` con paso
   s = L − m y cada trozo con su coeficiente de DEEP. Lo que el verificador ve de los trozos —cada
   uno en las q posiciones, en z y en z·g— lo descuenta el núcleo CONJUNTO con los aleatorizadores:
   rango(O∘M) = rango([abiertos | O∘D]) − rango(abiertos), sobre combinaciones aleatorias de todo el
   espacio de parámetros, más que su rango puede ser; con la misma O, la misma K y los mismos
   desafíos que el instrumento de R3. Dos comprobaciones dentro de la medida: que C(P_ρ) cabe en
   los N trozos de paso s —que es como se nota que el numerador se anula donde el divisor; si no,
   el interpolado no sería un polinomio—, y que las máscaras no salen de K, por rangos: el de las
   máscaras con K, menos el de los abiertos, es el de K.

**Cómo se valida.** Sin columnas lineales la medida reproduce exactamente lo clavado del §898 —en
el juguete, 424 sin determinar, 254 de máscara y 80 de aleatorizadores solos; en la solvencia, en
cada canon, 632 y 378—: la misma O, la misma K y el mismo subespacio, con otro muestreo y el núcleo
conjunto en vez del residuo por aleatorizador. Con las dos lineales del juguete, 38 libres cada una
(T − 2q − 6 con T = 128), la máscara crece en exactamente sus 76 dimensiones, dentro de K: 156 =
80 + 76 con los aleatorizadores, 330 = 254 + 76 las tres.

**El censo**, medido en cada canon sobre las 23 familias (`CENSO_134`), con la traza y las
opciones de una prueba real de cada una —con m = 64—, la sonda con dos semillas y las entradas
públicas vacías, que ninguna AIR mira al construirse salvo la edad, que va con unas de ocho hojas:

| AIR | T | reales | lineales, coeficiente público | lineales, pendiente del testigo | no lineales | libres por lineal (T − 2q − 6) | veredicto |
|---|---|---|---|---|---|---|---|
| SendAir | 1024 | 58 | 30 | 0 | 28 | 934 | cumple |
| SendV2Air | 1024 | 62 | 34 | 0 | 28 | 934 | cumple |
| ClaimAir | 1024 | 57 | 29 | 0 | 28 | 934 | cumple |
| ClaimAirV2 | 1024 | 61 | 33 | 0 | 28 | 934 | cumple |
| BurnAir | 1024 | 48 | 21 | 0 | 27 | 934 | cumple |
| DoubleEntryAir | 1024 | 41 | 15 | 0 | 26 | 934 | cumple |
| PrendaAir | 512 | 42 | 17 | 0 | 25 | 422 | cumple |
| MintPendingClimbAir | 512 | 39 | 13 | 0 | 26 | 422 | cumple |
| AuditAir | 512 | 31 | 17 | 0 | 14 | 422 | cumple |
| BandaAir | 512 | 27 | 13 | 0 | 14 | 422 | cumple |
| CobroPendienteAir | 512 | 44 | 18 | 0 | 26 | 422 | cumple |
| PagoEnCursoAir | 512 | 44 | 18 | 0 | 26 | 422 | cumple |
| CreditClimbAir | 512 | 39 | 13 | 0 | 26 | 422 | cumple |
| MintClimbAir | 512 | 42 | 16 | 0 | 26 | 422 | cumple |
| RecoveryClimbAir | 512 | 43 | 17 | 0 | 26 | 422 | cumple |
| SolvencyAir | 256 | 5 | 4 | 0 | 1 | 166 | cumple |
| MerkleAir | 256 | 13 | 0 | 0 | 13 | 166 | sin lineales: necesita la 133 |
| FrozenClimbAir | 256 | 25 | 0 | 0 | 25 | 166 | sin lineales: necesita la 133 |
| NullifierThresholdAir | 64 | 21 | 8 | 0 | 13 | 0 | no cabe: la 131 antes |
| ThresholdAir | 64 | 34 | 7 | 0 | 27 | 0 | no cabe: la 131 antes |
| EdadAir | 64 | 83 + 6 aux | 4 | 7 | 72 | 0 | no cabe: la 131 antes |
| RefundAir | 64 | 12 | 0 | 0 | 12 | 0 | no cabe: la 131 antes |
| RefundAirV2 | 64 | 16 | 4 | 0 | 12 | 0 | no cabe: la 131 antes |

En ninguna AIR sin tramo auxiliar hay una columna lineal con pendiente del testigo: las columnas o
entran con grado ≥ 2 —las de los hashes, los bits de los rangos— o entran linealmente con
coeficientes constantes o periódicos —los transportes de raíces, importes y contadores—. En la
edad, siete columnas principales entran linealmente con la pendiente de otra, por las lecturas del
tramo auxiliar.

**Lo medido** (`MLIN_MEDIDO`; la solvencia en cada canon, las demás en el test largo
`mide_m_lin_en_las_23_familias`, una vez, con la semilla 1; ⚠️ §907: y con la semilla 2 y una
instancia adversa, apartado 16, con la huella de la composición de cada familia clavada, que
obliga a remedir si una AIR cambia), sobre F:

| AIR | T, trozos | sin determinar (O sobre K) | §898: columna y aleatorizadores | lineales y aleatorizadores | las tres | R3 con M_lin |
|---|---|---|---|---|---|---|
| SendAir, SendV2Air, ClaimAir, ClaimAirV2, BurnAir, DoubleEntryAir | 1024, 8 | 1016 | 766 | **1016** | 1016 | se cumple |
| PrendaAir, MintPendingClimbAir, AuditAir, BandaAir, CobroPendienteAir, PagoEnCursoAir, CreditClimbAir, MintClimbAir, RecoveryClimbAir | 512, 8 | 760 | 510 | **760** | 760 | se cumple |
| SolvencyAir | 256, 2 | 632 | 378 | **632** | 632 | se cumple |
| MerkleAir, FrozenClimbAir | 256, 9 | 632 | 382 | — (sin lineales) | 382 | no se cumple (§898) |

Los tiempos, en el contenedor de la sesión: de 15 s (la solvencia) a 314 s (SendV2Air) por
familia, con las eliminaciones sobre F como coste dominante.

**Lo que dice.** Dieciséis de las veintitrés familias cumplen R3 con la construcción de hoy una vez
contada M_lin: todas las de T ≥ 512 y la solvencia. En todas, las lineales con los aleatorizadores
llegan a K sin la columna aleatoria, y la razón es la que el §898 señaló al revés: cada columna
lineal entra en D con su propio coeficiente de DEEP en E por el camino directo y con los de los
trozos por el del cociente —direcciones distintas de E en cada punto que FRI abre—, mientras que la
columna aleatoria sola, con un único coeficiente, movía una coordenada de dos. MerkleAir y
FrozenClimbAir no tienen ninguna columna lineal —la sbox de grado 7 de Rescue toca todas, como
preveía la 134—: su R3 es la del §898, 382 de 632, y **necesitan la 133**. Las cinco de T = 64 no
tienen M_lin que medir, T − 2q − 6 < 0, y siguen en la 131; tienen 8, 7, 4, 0 y 4 columnas lineales,
que la cuenta fina de la 135 podría aprovechar sin más filas, con un lema que nadie ha escrito.
**Qué familias necesitan la 133 para R3**: siete, MerkleAir, FrozenClimbAir y las cinco de T = 64
(éstas, la 131 antes); las otras dieciséis, no. De las siete, cinco son del cable —la autorización
delegada, la edad, los dos reembolsos y la subida de la congelación delegada—; el umbral conjunto y
el camino de Merkle no lo son.

**Lo que vale.** L3 no cambia: K, O, lo público y la condición son los mismos; crece M con una
máscara más que cumple lo que L3 pide de una máscara —las filas aleatorias de una columna lineal,
condicionadas a S_toc, son uniformes en el núcleo de la evaluación en S_toc, que no depende del
testigo, y su imagen en D es lineal y pública— (⚠️ §907: y el condicionamiento a las aperturas de
los trozos, que M_lin mueve, no rompe la independencia: L7, apartado 16), y el paso S5 del
simulador del apartado 5 —D̃ uniforme en la clase de K— queda justificado por R3 con ella, como
dice L3. Con L1 (las dieciocho
de T ≥ 256) y las demás hipótesis, la proposición del apartado 8 vale para las dieciséis con la
construcción de hoy. R3 con
M_lin es suficiente, no necesaria: el instrumento condiciona cada columna lineal a sus 2q + 6
tocados, más de lo que el verificador ve de ella (la 135), así que un «no se cumple» no es un
contraejemplo; para MerkleAir y FrozenClimbAir no hace falta la cuenta fina: no tienen columna que
contar.
⚠️ §905: la cuenta fina está medida (apartado 15): L1' vale como lema y hoy no cambia el censo.

**Lo que no hace.** No cuenta las filas aleatorias del tramo auxiliar (sólo la edad lo tiene, con
T = 64). No mide los desafíos de ninguna prueba real: la geometría y las opciones son las de una
prueba de cada familia, y los desafíos los elige el test, como en el §898 (⚠️ §907: tres instancias
por familia, una adversa, y L8 para la capa 0 con T ≥ 512, apartado 16). No toca un byte de
producción: el instrumento es sólo de tests. No es una promesa: lo que A4 enuncie, lo enuncia A4.

## 15. La 135, medida (§905): la cuenta fina no rescata ninguna de las cinco

### L1' — La cuenta fina para las columnas lineales: lo visto, no lo tocado (§905)

**Lo que L1 cuenta de más.** L1 pide que las T filas aleatorias de una columna cubran los 2q + 3
puntos que la prueba toca de ella —las q posiciones, sus q siguientes, z, z·g y g²·z, 2q + 6
dimensiones sobre F—, porque C(x_j) lee P(g·x_j) y C(z·g) lee P(g²·z). Pero el verificador no ve
P(g·x_j) ni P(g²·z): ve P en S_ab —q + 4 dimensiones sobre F por columna— y ve C en S_ab, una suma
de todas las columnas con q + 2 valores de E en total, 2q + 2 dimensiones sobre F que entran en el
marco OOD y en las aperturas de los trozos (y C(z), que lee P(z) y P(z·g), es función de lo visto).
Para una columna que entra en C con grado ≥ 2 eso no ayuda: su parte en C(x_j) depende del testigo
de una forma que la AIR decide (L6). Para las columnas lineales con coeficiente público, sí.

**Enunciado.** Sea Lin el conjunto de columnas principales que entran en C linealmente con
coeficiente público (H12). Para cada i ∈ Lin, lo que queda libre de sus filas aleatorias una vez
condicionadas a lo VISTO de ella es P_ρ,i = Z_real·V_ab·w_i, con V_ab nulo en S_ab y sus
conjugados (grado q + 4) y w_i ∈ F[x] de grado < T − q − 4; y C en S_ab se escribe como
C(y) = A(y) + Σ_{i∈Lin} [σ_i^cur(y)·P_ρ,i(y) + σ_i^next(y)·P_ρ,i(g·y)] con σ públicas y A función
del testigo, de las columnas no lineales y de lo visto. Como P_ρ,i(y) = 0 en S_ab, lo que queda es
Λ(w) := (Σ_i σ_i^next(x_j)·P_ρ,i(g·x_j))_j, Σ_i σ_i^next(z·g)·P_ρ,i(g²·z)), una aplicación
F-lineal de ⊕_i F^{T−q−4} en E^{q+1} (2q + 2 dimensiones sobre F). **Si Λ es sobreyectiva**, la
distribución de (P(S_ab) de todas las columnas, C(S_ab)) dada la traza real es la uniforme sobre su
soporte, independiente del testigo: la de P(S_ab) por L1 con q + 4 ≤ T, y la de C(S_ab) porque
A + Λ(w) con w uniforme y Λ sobre es uniforme sea cual sea A. Y lo residual de w tras condicionar a
C(S_ab) —el núcleo de Λ— es lo que queda para enmascarar D: es lo que mide el instrumento con la
base fina, con el núcleo conjunto de los aleatorizadores, que ya descuenta las aperturas de los
trozos.

**Demostración.** Las filas aleatorias de cada columna son uniformes en F^T e independientes del
testigo y entre columnas (L5). Para i ∈ Lin, condicionar P_i a sus q + 4 valores en S_ab es
condicionar a una aplicación lineal de rango q + 4 (la matriz de Lagrange sobre S_ab, de rango
pleno por H1), así que lo residual es uniforme en el núcleo, Z_real·V_ab·F[x]_{<T−q−4}, y no
depende del testigo. C es afín en las columnas de Lin con pendientes públicas (H12), así que
C(y) − A(y) = Σ_i [σ_i^cur(y)·P_ρ,i(y) + σ_i^next(y)·P_ρ,i(g·y)], y el primer sumando es nulo en
S_ab. Si Λ es sobreyectiva, Λ(w) es uniforme en E^{q+1} y, al ser w independiente de A, C(S_ab) es
uniforme e independiente del testigo; la distribución conjunta con P(S_ab) de las demás columnas
también, porque esas filas son independientes de w. □

**Lo que pide, y se mide.** (1) q + 4 ≤ T con la Lagrange de rango pleno sobre S_ab: ya medido
para todas las familias (`rango_pleno`). (2) Λ sobreyectiva: su rango sobre F es 2q + 2; lo mide
`instrumento_mlin::sobreyectiva`, por familia. (3) R3 con la base fina: las máscaras —la columna
aleatoria, los aleatorizadores y el núcleo de Λ— cubren O|_K, con el núcleo conjunto de siempre.
Con las tres, el simulador del apartado 5 cambia sólo en S2: muestrea P(S_ab) y C(S_ab) en vez de
P(S_toc); lo demás, igual. Lo que no pide: nada de las columnas no lineales más que q + 4 ≤ T.

**Lo que cuenta por cuenta, con T = 64.** Cada columna lineal deja T − q − 4 = 18 dimensiones, y Λ
necesita 2q + 2 = 86: hacen falta al menos cinco columnas lineales. Los dos umbrales tienen 8 y 7;
la edad, 4 (más siete con pendiente del testigo, que no sirven); el reembolso v2, 4; el v1, 0. Por
cuenta, L1' sólo puede valer para los dos umbrales, y la medida dice si vale.

**Lo medido** (`instrumento_mlin`: `Base::Vistos`, `sobreyectiva`, `FINA_MEDIDO`; en cada canon,
con la semilla 1, sobre las familias que no caben con la cuenta gruesa y tienen columnas lineales,
y sobre el juguete como control):

| AIR | T | lineales | rango de Λ (blanco 86) | R3 con la base fina: lineales y aleatorizadores / las tres (K) | veredicto |
|---|---|---|---|---|---|
| NullifierThresholdAir | 64 | 8 | 36 | no se mide | no: Λ no es sobreyectiva |
| ThresholdAir | 64 | 7 | **86** | 120 / **158** (168) | no: L1' vale y a R3 le faltan 10 |
| EdadAir | 64 | 4 (+ 7 del testigo) | 18 | no se mide | no |
| RefundAirV2 | 64 | 4 | 72 | no se mide | no: 4 · 18 = 72 < 86, por cuenta |
| RefundAir | 64 | 0 | — | — | no: sin lineales |
| el juguete | 128 | 2 | 86 | 158 / 332 (424) | control: la fina no baja de la gruesa (156 / 330) |

Que Λ no llegue con ocho columnas lineales (36 = 2 · 18) dice que casi todas entran en C sólo por
la fila actual, o con pendientes dependientes en la fila siguiente; es una inferencia consistente
con la cifra, no una medida. Y la base fina añade poco a R3: en el juguete, de las 2 · 82 libres
se van 86 en hacer uniforme C(S_ab) —el núcleo conjunto fija cada trozo en S_ab, y con ello C— y
quedan 78, dos más que las 76 de la gruesa.

**Lo que dice.** L1' es un lema que vale, y hoy no rescata a ninguna de las cinco: en cuatro, Λ no
es sobreyectiva, y en el umbral conjunto lo es pero R3 con la base fina se queda a diez
dimensiones de K. Lo que sí deja escrito para el corte: con la 133 —dos columnas aleatorias, con
las que R3 se cumple por geometría también con T = 64 (apartado 13)— y L1' medido, el umbral
conjunto queda cubierto sin más filas; los otros cuatro —la autorización delegada, la edad y los
dos reembolsos— necesitan la 131, T = 128, y con ella la cuenta gruesa (90 ≤ 128) y la 133 los
cubren por geometría. Después del corte, pues: dieciséis de hoy, más MerkleAir y FrozenClimbAir
por la 133, más el umbral conjunto por la 133 y L1', diecinueve; y cuatro que esperan a la 131.

**Lo que no hace.** No cambia la cuenta gruesa de L1 ni lo clavado del §900: la base fina es otra
medida, con sus propias constantes. No rescata nada hoy. No toca la promesa.

## 16. La revisión del agente, cerrada (§907)

**De dónde sale.** La 136 pide una revisión ajena; la que hay desde el §907 es de un agente externo
a la sesión, no de una persona: otra instancia, sin la crónica ni las conclusiones, con el dosier,
una copia del árbol del §903 y las fuentes de winterfell delante, en lectura pura
(`doc/ocultacion-hvzk-revision-agente.md`, el informe entero). No es la revisión de la 136 —al
revisor lo elige el autor— y cuenta como la tercera revisión de la casa. Sus siete objeciones
están en el apartado 11 con su cierre; este apartado escribe los dos lemas que pidió, rehace la
cuenta de la ventaja y mide lo que pidió medir.

### L7 — El núcleo conjunto: por qué M_lin es una máscara aunque mueva lo abierto de los trozos (§907)

**Lo que faltaba en L3.** L3 pide de una máscara dos cosas: imagen lineal y pública en D, y
uniformidad independiente del resto de D una vez condicionada la vista. Las filas aleatorias de
una columna lineal cumplen la primera, y la segunda frente a lo abierto de la traza —se anulan en
S_toc— y frente a C(x_j) y C(z·g) —que no las ven—; pero NO frente a las aperturas de los trozos:
P_ρ mueve cada h'_k(y) en y ∈ S_ab, y sólo la suma Σ_k y^{k·s}·h'_k(y) = C(y) queda fija. El
instrumento lo resuelve con el núcleo conjunto (apartado 14); el lema que lo justifica es éste.

**Lema L7.** Sean W = Π_i F[x]_{<T−2q−6} lo libre de las columnas lineales tras L1
(P_ρ,i = Z_real·V_toc·w_i), R = E[x]_{<m}^{N−1} los aleatorizadores, y o = (h'_k(y))_{k<N, y∈S_ab}
las aperturas de los trozos. Entonces o = n + A(w) + B(r), con A y B lineales y públicas —el camino
del cociente de w, troceado como `segmentar`, y x^s·r_k − r_{k−1}, evaluados en S_ab— y n los
trozos de C sin la parte de (w, r): función del testigo y de las filas aleatorias de las columnas
no lineales. Condicionada a todo lo demás de la vista y a n, la pareja (w, r) es uniforme en un
trasladado t + J del núcleo conjunto J = ker[A | B]: t depende del testigo, J no. Y D = D' + M(j),
con j uniforme en J e independiente de D', donde M es la imagen lineal de (w, r) en D —cc_i·Q[P_ρ,i]
por el camino directo, Σ_k cc'_k·Q[·] por los trozos— y D' lo determina el resto de la vista.

**Demostración.** (a) El prior de (w, r) es uniforme en W × R: cada w_i por L1 —el residuo de una
columna uniforme condicionada a sus valores en S_toc es uniforme en el núcleo de la evaluación en
S_toc— y cada r_k por construcción (L5), independientes entre sí y del testigo. (b) La condición
«las aperturas valen o» es afín en (w, r): [A | B](w, r) = o − n. Una distribución uniforme en un
espacio vectorial, condicionada a una ecuación afín, es uniforme en la fibra —un trasladado del
núcleo— si la fibra no es vacía. (c) La fibra no es vacía para ningún w: en cada y ∈ S_ab, el
vector (y^s·r_k(y) − r_{k−1}(y))_{k<N} recorre, cuando (r_k(y))_{k<N−1} recorre E^{N−1}, el
hiperplano {v : Σ_k y^{k·s}·v_k = 0} (la triangularidad de L2), y (r_k(y))_{y∈S_ab} recorre E^{q+2}
cuando r_k recorre E[x]_{<m}, porque q + 2 ≤ m (H4, Vandermonde). Y o − n − A(w) está en el producto
de esos hiperplanos para todo w: Σ_k y^{k·s}·o_{k,y} = C(y) y Σ_k y^{k·s}·n_{k,y} = C(y), porque el
camino del cociente de w no mueve C en S_ab —C(P_ρ)(y) − C(0)(y) = σ(y)·P_ρ(y) + σ'(y)·P_ρ(g·y) = 0,
los dos puntos son tocados— y la suma telescópica no ve r. (d) Luego (w, r), condicionada a la
vista sin D y a n, es uniforme en t + J, con t una solución cualquiera y J el núcleo, que sólo
depende de A y B, públicas. D es afín en (w, r) con parte lineal M, así que D = D' + M(t) + M(j) con
j uniforme en J; M(t) va con D'. ∎

**Lo que da.** rango_F(O∘M|_J) = rango[abiertos | O∘M] − rango[abiertos], que es lo que
`instrumento_mlin` calcula sobre combinaciones aleatorias del espacio de parámetros (rango y
nulidad: el rango de la pila es el de la primera más la dimensión de la imagen de su núcleo); y R3
con M_lin es O(M(J)) + O(M_R(U)) = O(K), con U lo libre de la columna aleatoria, que no entra en
ningún trozo y es independiente de (w, r). Con eso S5 queda justificado: la clase de D módulo K no
depende del testigo (L3) y, dentro de K, la máscara es uniforme sobre O(K). H12 pasa de medida a
demostrada como lema, con la medida de su rango.

### L8 — La cobertura de la capa 0 por puntos, para cualesquiera posiciones (§907)

**Lo que se puede demostrar sin medir.** Sea P₀ el conjunto de los puntos de Λ que FRI abre en la
capa 0 fuera de las posiciones: los siete compañeros de coset de cada posición, 7c₀ puntos
distintos con c₀ cosets distintos; y P₀' ⊆ P₀ los que no son tocados, x ≠ g·x_k para toda posición
x_k (los otros son las coincidencias de la objeción 1). Una colisión de cosets sólo quita puntos
—c₀ < q hace P₀ más pequeño—, y el lema es monótono en eso.

**Lema L8.** (i) Para cualesquiera posiciones distintas, si 7c₀ ≤ L − q − 4, la columna aleatoria
sola cubre en cada x ∈ P₀ la recta F·a_x ⊂ E, con
a_x = cc_R·(2x − z − z·g)·V_pos(x)·(x − z̄)(x − z̄·g) ≠ 0, y las 7c₀ rectas a la vez: lo libre de la
columna es Φ·u con u ∈ F[x]_{<L−q−4}, y u ↦ (u(x))_{x∈P₀} es sobreyectiva sobre F^{P₀}
(Vandermonde, con nodos distintos de F y no más nodos que coeficientes). Es exacto, sin
probabilidad: 294 ≤ 466, 978 y 2002 con T = 256, 512 y 1024. (ii) Sea i una columna lineal. Su
camino directo cubre en cada x ∈ P₀' la recta F·b_x, con
b_x = cc_i·(2x − z − z·g)·Z_real(x)·V_toc(x)/((x − z)(x − z·g)) ≠ 0, y b_x/a_x = (cc_i/cc_R)·f_x
con f_x = Z_real(x)·(x − g²z)(x − g²z̄)·Π(x − g·x_k) ∈ F, no nulo (el producto, sobre los g·x_k que no
son posiciones): si cc_i/cc_R ∉ F, (a_x, b_x) es una F-base de E en cada x ∈ P₀'. Si además
|P₀'| ≤ T − 2q − 6, la aplicación (u, w_i) ↦ (D(x))_{x∈P₀'} —con el camino directo Y el del
cociente de w_i, Γ_x(w_i), que no es puntual— es sobreyectiva sobre E^{P₀'} salvo con probabilidad
≤ |P₀'|/|F| sobre cc_i. (iii) Con n columnas lineales, «alguna con cc_i/cc_R ∉ F» falla con
probabilidad |F|^{−n}: con n ≥ 4, menos de 2^{−256}.

**Demostración de (ii).** La matriz sobre F de (u, w) ↦ (D(x))_{x∈P₀'} ∈ E^{P₀'} ≅ F^{2|P₀'|}
tiene las columnas de u fijas —a_x·u(x), sin cc_i— y las de w afines en las dos coordenadas
(c₀, c₁) de cc_i: cc_i·ψ(x)·w(x) + Γ_x(w), con Γ el camino del cociente, que lleva los cc'_k y los
coeficientes de las restricciones y no cc_i. Fíjese un cc_i* con cc_i*/cc_R ∉ F. Sin Γ, la
aplicación es sobreyectiva en cc_i*: en cada x, (a_x, b_x) es base, y u y w recorren F^{P₀'} cada
uno (Vandermonde: |P₀'| ≤ T − 2q − 6 ≤ L − q − 4); luego algún menor de tamaño 2|P₀'| de la matriz
sin Γ, con |P₀'| columnas de u y |P₀'| de w, es no nulo en cc_i*. Tómense esas columnas en la
matriz con Γ: su menor Q es un polinomio en (c₀, c₁) de grado ≤ |P₀'| —cada columna de w es afín
en ellas— cuya parte homogénea de grado |P₀'| es el menor sin Γ, no nulo; así que Q no es el
polinomio nulo, y Schwartz–Zippel sobre F² da Pr[Q(cc_i) = 0] ≤ |P₀'|/|F|. Donde Q ≠ 0, la
aplicación es sobreyectiva. ∎

**Lo que dice y lo que no.** Con T = 512 y 1024, |P₀'| ≤ 294 ≤ 422 y 934: la capa 0 está cubierta
coordenada a coordenada para cualesquiera posiciones, fuera de los puntos de coincidencia, con
fallo ≤ 294/2^64 ≈ 2^{−55,8} sobre el coeficiente de DEEP de una sola columna lineal —una cota de
la técnica, no del suceso: el único suceso malo que la demostración nombra, cc_i ∈ F·cc_R, tiene
probabilidad 1/|F| por columna y |F|^{−n} con las n que hay, y aun en él el camino del cociente
puede cubrir—. Con T = 256, 166 < 294: la solvencia no entra con una columna, y con sus cuatro
juntas (664 ≥ 294) la sobreyectividad es una condición de rango conjunto, no de Vandermonde, que
la medida decide instancia a instancia. El lema no cubre la segunda coordenada en los puntos de
coincidencia —b_x = 0, y sólo el camino del cociente y los aleatorizadores pueden ponerla— ni los
L/8 − c₀ funcionales nuevos de la capa 1 (22 sobre E con T = 256 y c₀ = 42): eso queda en la
medida. El denominador |F| en vez de |E| es de la construcción: u y w viven en F[x] porque la
columna aleatoria y las columnas reales viven en F, la aplicación es F-lineal y sus menores son
polinomios en las coordenadas sobre F de los desafíos; con la máscara de DEEP en E (la 133) la
cota iría sobre |E|.

**La cuenta de la ventaja, rehecha (O4).** Lo que el simulador del apartado 5 deja: 2^{−60} de H3
(z sin coincidencias de conjugados, por prueba); 2^{−128} de S5 (el grado exacto L − 2, que el
probador aserta); Q·2^{−256} de L0 (Q consultas al oráculo sobre prefijos con raíces frescas); y
ε_R3, que R3 con M_lin falle en los desafíos reales. De ε_R3: en la capa 0 con T ≥ 512, fuera de
las coincidencias, ≤ 7c₀/|F| ≈ 2^{−55,8} por L8; para unas posiciones fijas en las que una
instancia medida da el rango pleno, la caída de rango sobre (z, cc, cc', α_i) es la anulación de un
menor no nulo —tamaño ≤ 1016, grado acotado por una decena por entrada en las coordenadas sobre F
de los desafíos—, ≤ 10·1016/|F| ≈ 2^{−50,7}, del orden de 2^{−51} (⚠️ el apartado 6 decía «sobre
|E|»; corregido); y para lo demás —la capa 1, las coincidencias y la solvencia entera— no hay
cota: hay las instancias medidas de abajo. La proposición del apartado 8 se enuncia con esta
cuenta, y sigue sin ser una promesa.

**Lo medido (O1).** La solvencia, la familia de margen más fino —cuatro lineales con 166 libres,
un solo aleatorizador—, en cada canon, con cuatro instancias de desafíos: la semilla 1 (la de
`MLIN_MEDIDO`), las semillas 2 y 3 enteras, y la adversa —las posiciones de la semilla 1 con tres
parejas forzadas: p₁ = p₀ + |Λ|/8 (una colisión de cosets en la capa 0), p₃ = p₂ + 3|Λ|/8 − blowup
(una coincidencia coset–tocado: g·x₃ = x₂·μ³, el siguiente de x₃ en el coset de x₂) y p₅ = p₄ − blowup (g·x₅ = x₄, una posición siguiente
de otra), con la z, los coeficientes y los α de la semilla 1—. Lo que cada instancia trae y lo que
mide, sobre F (`MLIN_INSTANCIAS`):

| instancia | colisiones · coincidencias · siguientes | libres por lineal | sin determinar (O sobre K) | lineales y aleatorizadores | las tres | lineales solas | R3 con M_lin |
|---|---|---|---|---|---|---|---|
| semilla 1 | 0 · 3 · 0 | 166 | 632 | 632 | 632 | 587 | se cumple |
| semilla 2 | 0 · 1 · 0 | 166 | 632 | 632 | 632 | — | se cumple |
| semilla 3 | 2 · 1 · 0 | 166 | 604 | 604 | 604 | — | se cumple |
| adversa 1 | 1 · 4 · 1 | 167 | 618 | 618 | 618 | — | se cumple |

Cada colisión quita 14 a lo que FRI abre sin determinar —siete puntos sobre E—, y la máscara baja
exactamente lo mismo: R3 se cumple en las cuatro, con las lineales y los aleatorizadores justos en
todas, y con coincidencias en las cuatro —tres ya en la semilla 1, que la revisión no podía ver, y
cuatro en la adversa: puntos de la capa 0 donde el camino directo de las cuatro lineales vale cero
y la segunda coordenada la pone el camino del cociente con el aleatorizador—. Las posiciones sólo
dependen de |Λ| y q, así que lo que trae cada semilla es lo mismo para todas las familias de una
T: con T = 512 y 1024, las semillas 1 y 2 no traen ninguna pareja, la 3 trae una colisión y la
adversa una de cada (el test del juguete lo imprime). Las lineales solas, sin aleatorizador, llegan a 587 de 632 en la semilla 1: el
aleatorizador pone 45, lo que la revisión pedía saber (su apartado 4.1). Lo que esto dice es lo que
dice: cuatro instancias, no una cota.

**Las otras quince**, en el test largo, con la semilla 1 (la de `MLIN_MEDIDO`, ahora con las
lineales solas), la semilla 2 entera y la adversa; las posiciones de cada semilla son las mismas
para todas las familias de una T, y los rangos salieron iguales dentro de cada T:

| familias | instancia | colisiones · coincidencias · siguientes | libres por lineal | sin determinar (O sobre K) | lineales y aleatorizadores | las tres | lineales solas | R3 con M_lin |
|---|---|---|---|---|---|---|---|---|
| las seis de T = 1024 (SendAir, SendV2Air, ClaimAir, ClaimAirV2, BurnAir, DoubleEntryAir) | semilla 1 | 0 · 0 · 0 | 934 | 1016 | 1016 | 1016 | **1016** | se cumple |
| | semilla 2 | 0 · 0 · 0 | 934 | 1016 | 1016 | 1016 | — | se cumple |
| | adversa 1 | 1 · 1 · 1 | 935 | 1002 | 1002 | 1002 | — | se cumple |
| las nueve de T = 512 (PrendaAir, MintPendingClimbAir, AuditAir, BandaAir, CobroPendienteAir, PagoEnCursoAir, CreditClimbAir, MintClimbAir, RecoveryClimbAir) | semilla 1 | 0 · 0 · 0 | 422 | 760 | 760 | 760 | **760** | se cumple |
| | semilla 2 | 0 · 0 · 0 | 422 | 760 | 760 | 760 | — | se cumple |
| | adversa 1 | 1 · 1 · 1 | 423 | 746 | 746 | 746 | — | se cumple |

R3 con M_lin se cumple en las 48 instancias de las dieciséis familias, la adversa incluida —la
colisión quita 14 a K y lo mismo a la máscara, en todas—. Y lo que la columna de las lineales
solas dice, que ninguna medida anterior decía: con T ≥ 512, las filas aleatorias de las columnas
lineales cubren POR SÍ SOLAS todo lo que FRI abre sin determinar, sin la columna aleatoria y sin
los aleatorizadores del cociente; sólo la solvencia, con cuatro lineales y 166 libres, necesita al
aleatorizador, que pone 45. Los tiempos: de 279 a 425 s por familia la semilla 1 con T = 1024
(cuatro eliminaciones sobre F), de 118 a 133 s con T = 512 y 18 s la solvencia; la semilla 2 y la
adversa, dos eliminaciones cada una, de 189 a 296 s y de 77 a 90 s.

**La huella de cada composición (O3).** `huella`: Blake3 de las cuentas de la AIR —L, columnas
reales y auxiliares, elementos auxiliares, factor del dominio, trozos, transiciones y aserciones—
y de C en cuatro puntos de E con cuatro marcos al azar de una moneda fija, con unos coeficientes y
unos elementos auxiliares fijos. Cambia con cualquier cambio en las restricciones —otra
pendiente, otra aserción, otro orden de columnas, otro grado, otra periódica— y no con un
comentario ni con el testigo. `HUELLA_C` clava las 23 y el test de las 23 familias la compara en
cada canon: un cambio es rojo y obliga a remedir la 134 en el test largo y a volver a clavar
`MLIN_MEDIDO`, `MLIN_INSTANCIAS` y `HUELLA_C`. Es lo que el censo no guardaba: una AIR que cambie
de restricciones conservando cuántas columnas entran linealmente movía el rango sin mover
ninguna guarda.

**Lo que queda.** Una cota para lo que sigue medido y no acotado: la capa 1 (L/8 − c₀
funcionales nuevos sobre E), la segunda coordenada en las coincidencias coset–tocado, y la
solvencia entera, donde 166 < 294 y las cuatro columnas juntas son una condición de rango
conjunto; L8 dice dónde está el filo y por qué el denominador es |F|. El pánico del aserto de grado
(la 137) y la guarda de las aserciones periódicas (la 138), del fork (⚠️ §911: cerradas las dos). Y la revisión de la 136, que
es de una persona que elige el autor: esta no la sustituye, y le ahorra lo que ya está visto.

## 17. Lo que el corte haría del argumento: R3 como teorema con las columnas pareadas (§908)

**La observación.** Todo lo que el apartado 16 deja medido y no acotado —la capa 1, las
coincidencias coset–tocado, la solvencia, y el |F| en el denominador de L8— viene de una sola
cosa: la máscara de DEEP vive en F. Con las dos columnas aleatorias de la D-AL (§899) y dos
coeficientes de DEEP independientes, cc₁ y cc₂, lo que las dos dejan libre en D es
2(x − x₀)·V_pos·(x − z̄)(x − z̄·g)·{cc₁·u₀ + cc₂·u₁} con u_i ∈ F[x]_{<L−q−4}, que es un espacio
sobre E —E[x]_{<L−q−4} entero— si y sólo si cc₂/cc₁ ∉ F. Con cc₂ := cc₁·ν, ν ∈ E \ F fijo, lo es
siempre: las dos columnas son una columna sobre E, R₀ + ν·R₁, con un solo coeficiente, y el
argumento entero pasa a ser E-lineal. Es la D-AL bis del RFC-0009, propuesta; no cambia un byte de
la prueba, y es una línea en el compositor de DEEP del probador y otra en el del verificador.

### L9 — Con las columnas pareadas, las máscaras cubren K entero, para cualesquiera posiciones (§908)

**Enunciado.** Sea M_par = 2(x − x₀)·V_pos·(x − z̄)(x − z̄·g)·E[x]_{<L−q−4} lo que las dos columnas
pareadas dejan libre en D, y M_r = Σ_k (cc'_k·x^s − cc'_{k+1})·V_pos·(2x − z − z·g)·E[x]_{<m−q−2}
lo que dejan los aleatorizadores del cociente (L2, L7). Entonces M_par + M_r = K para cualesquiera
posiciones, α_i y AIR, salvo que cc'_{k+1} = cc'_k·z̄^s para todo k < N − 1, un suceso de
probabilidad ≤ |E|^{−(N−1)} sobre los coeficientes de DEEP de los trozos; bajo H3 (z ∉ F).

**Demostración.** K = (x − x₀)·V_pos·E[x]_{<L−q−2}. Sea ev: K → E², k ↦ (k(z̄), k(z̄·g)). Como
(x − x₀)·V_pos no se anula en z̄ ni en z̄·g (z̄ ∉ F ⊇ Λ, y z̄ = x₀ pediría z̄/z = (1 + g)/2 ∈ F de
norma 1, es decir g = 1 o g = −3), ev es sobreyectiva —E[x]_{<L−q−2} interpola dos valores en dos
puntos— y ker(ev) = {k ∈ K : (x − z̄)(x − z̄·g) | k} = M_par. Luego K = M_par + M_r si y sólo si
ev(M_r) = E². Tómese un k y v ∈ E[x]_{<m−q−2}: ev de su término es (A·v(z̄), B·v(z̄·g)) con
A = (cc'_k·z̄^s − cc'_{k+1})·V_pos(z̄)·(2z̄ − z − z·g) y B lo mismo en z̄·g. V_pos(z̄) ≠ 0 (z̄ ∉ F);
2z̄ − z − z·g = 0 es el caso g ∈ {1, −3} de arriba, y 2z̄·g − z − z·g = 0 pediría (1 + g)/(2g) de
norma 1, g ∈ {1, −1/3}: ninguno es un generador de orden L ≥ 128. Queda cc'_{k+1} = cc'_k·z̄^s,
probabilidad 1/|E| por k, independientes. Si A ≠ 0 y B ≠ 0, como m − q − 2 = 20 ≥ 2,
(v(z̄), v(z̄·g)) recorre E² cuando v recorre E[x]_{<m−q−2}, y ev(M_r) = E². ∎

**Lo que da.** R3 (H6) deja de ser una hipótesis medida: las máscaras independientes del testigo
cubren K ENTERO —no sólo lo que FRI abre de él— para cualesquiera posiciones, α_i y AIR; ni M_lin
(H12), ni la cuenta fina (H13), ni L6, ni L8 (H14) hacen falta para el enunciado, y siguen como
instrumentos de guarda. El simulador del apartado 5 no cambia: S5 muestrea D̃ uniforme en la clase
de K, y ahora la distribución real es exactamente ésa. La cuenta de la ventaja queda en 2^{−60}
(H3) + 2^{−128} (S5) + Q·2^{−256} (L0) + |E|^{−(N−1)} (L9), sin ε_R3 y sin ningún |F|. Lo que la
proposición sigue pidiendo: L1 por geometría (T ≥ 2q + 6, la 131 para las cinco de T = 64), L2
(q + 2 ≤ m), H3, y el modelo de L0, L4 y L5.

**Medido** (`instrumento_r3::cubren_k`, `K_ENTERO_PAREADAS`), en el espacio de coeficientes y sin
pasar por O, sobre F, con la semilla 1 y con las posiciones adversas del §907:

| geometría | instancias | 2·dim_E K | una columna (hoy): columna · aleatorizadores · juntas | dos sin parear (D-AL): columnas · aleatorizadores · juntas | dos pareadas (D-AL bis): columnas · aleatorizadores · juntas | ev(M_r), de 4 |
|---|---|---|---|---|---|---|
| juguete, T = 128, 4 trozos | semilla 1 y adversa | 424 | 210 · 80 · 254 | 420 · 80 · 424 | 420 · 80 · **424** | 4 |
| T = 64, 15 trozos | ídem | 168 | 82 · 80 · 126 | 164 · 80 · 168 | 164 · 80 · **168** | 4 |
| T = 256, 2 trozos | ídem | 936 | 466 · 40 · 506 | 932 · 40 · 936 | 932 · 40 · **936** | 4 |
| T = 256, 9 trozos | ídem | 936 | 466 · 80 · 510 | 932 · 80 · 936 | 932 · 80 · **936** | 4 |
| T = 512, 8 trozos | ídem | 1960 | 978 · 80 · 1022 | 1956 · 80 · 1960 | 1956 · 80 · **1960** | 4 |
| T = 1024, 8 trozos | ídem | 4008 | 2002 · 80 · 2046 | 4004 · 80 · 4008 | 4004 · 80 · **4008** | 4 |

En las doce medidas —seis geometrías, la semilla 1 y la adversa del §907— las columnas pareadas
solas dan 2(L − q − 4), cuatro dimensiones sobre F menos que K (las de z̄ y z̄·g), y con los
aleatorizadores K entero: lo que L9 dice, cifra a cifra; las dos sin parear dan lo mismo en estas
semillas, donde cc₂/cc₁ ∉ F; una columna, la mitad más los aleatorizadores. El criterio del lema
—ev(M_r), el rango sobre F de los aleatorizadores evaluados en (z̄, z̄·g)— es 4 de 4 en las doce.
Lo que cuesta medirlo entero: 1.755 s las seis geometrías con las tres construcciones, casi todo
en T = 1024, con rangos de 4008 × 4096 sobre F; por eso el sello mide entero el juguete, T = 64 y
las dos de T = 256 (55 s) y el criterio en las seis, y el test largo de `instrumento_r3`
(`k_entero_en_las_geometrias_grandes`, ignorado y declarado) mide entero T = 512 y 1024, con sus
cifras clavadas en `K_ENTERO_PAREADAS`.


**Lo que no hace.** No enciende nada: la D-AL bis es una propuesta y el corte es del autor (RFC-0018
D-H). ⚠️ §909: la D-AL bis está aceptada y construida en el fork —`Oculta<A>` parea los
coeficientes de DEEP de las columnas aleatorias cuando son dos o más, en el probador y en el
verificador—, y el falsador `laboratorio_columnas_pareadas` mide que está en vigor: la prueba de
laboratorio verifica pareando y no con `verify_laboratorio_sin_parear`; la de producción, con una
columna, no cambia. Lo que queda encendido es lo de siempre; el corte la lleva a los 23. No cambia el enunciado de hoy (apartado 8 con el 16): la construcción de `zkssl/0.4` sigue
siendo la que es, con una columna. No toca H3, cuyo 2^{−60} es ahora el término mayor, ni el 2^{−128}
de S5 (la 137).

## 18. La proposición para las 23 con la construcción de `zkssl/0.5` (§915)

**Lo que cambió con el corte** (§914: RFC-0018 D-H; RFC-0009 D-AL bis). Desde `zkssl/0.5` toda
prueba de producción de las veintitrés familias lleva dos columnas aleatorias pareadas en DEEP como
una sobre E (L9), T filas aleatorias por columna con T ≥ 128 en todas —las cinco que tenían 64
pasan a 128—, el cociente cegado con m = 64 y las hojas saladas. La marca v2 que la identifica
(25 bytes: m, la familia, la versión de la AIR, el perfil y las columnas) es constante por familia
y no depende del testigo: entra en la vista como lo que ya era público por diseño. Nada más del
argumento cambia: el simulador del apartado 5 es el mismo, con S5 muestreando D̃ uniforme en la
clase de K, que con L9 es exactamente la distribución real.

**Las hipótesis, con el corte.** De la tabla del apartado 4:

| hipótesis | con `zkssl/0.5` | qué la mide o la sostiene |
|---|---|---|
| H1 (q + 4 ≤ T sobre S_ab, 2q + 6 ≤ T sobre S_toc) y H11 (T ≥ 2q + 6) | medidas, en las 23: 2q + 6 = 90 ≤ 128 ≤ T | `INDEPENDIENTE_DE_LA_AIR`, «cubre» en las 23; `rango_de_los_tocados` y `rango_pleno`, en cada canon |
| H2 (Λ ∩ H = ∅, posiciones únicas) | demostrada | L1 |
| H3 (z fuera de F y de H, sin coincidencias de conjugados) | supuesta, fallo < 2^{−60} por prueba | no se mide |
| H4 (q + 2 ≤ m) | medida: 44 ≤ 64 | `rango_pleno`; el mutante m = 43 |
| H5, H10 | demostradas | L2, L5 |
| H6 (R3) | **demostrada**: L9, para cualesquiera posiciones, α_i y AIR; y medida en las cinco geometrías de producción, K entero | `K_ENTERO_PAREADAS`: en cada canon, las geometrías de T ≤ 256 enteras y el criterio ev(M_r) en todas; el test largo, T = 512 y 1024 |
| H7 (Blake3 como oráculo aleatorio programable) | modelo | — |
| H8 (la sal es fresca), H9 (ninguna semilla se repite) | supuestas | `SECURITY.md` §3.11; el falsador de la semilla repetida |
| H12, H13, H14 (M_lin, la cuenta fina, los desafíos reales de R3 con M_lin) | **no hacen falta**: L9 cubre K entero sin ellas; siguen como instrumentos de guarda | `instrumento_mlin`, en cada canon y en su test largo |

**Proposición (condicional, para las veintitrés).** Bajo H1, H2, H3, H4, H5, H7, H8, H9, H10 y
H11 —H6 es L9—, para cada una de las veintitrés familias con fila en la tabla de D-B del RFC-0009
con la construcción de `zkssl/0.5`, existe un simulador S, el del apartado 5, tal que, para toda
traza real que la AIR interna acepta, la vista del verificador honesto del protocolo oculto y la
salida de S son estadísticamente indistinguibles en el modelo de oráculo aleatorio programable, con
ventaja menor que 2^{−60} (H3) + 2^{−128} (S5) + Q·2^{−256} (L0) + |E|^{−(N−1)} (L9) para Q
consultas al oráculo, donde N es el número de trozos del cociente de la geometría: 2 en la
solvencia (2^{−128}) y 8, 9 o 10 en las demás. Ninguna familia queda fuera por su nombre, y ninguna
necesita una hipótesis propia de su AIR (L6 no se usa). Lo que la proposición no dice: nada de la
solidez, nada frente a un adversario cuántico, nada de los canales laterales y nada de lo público
por diseño (apartado 7, lo que sigue en pie).

**Lo medido sobre la construcción del corte** (§914 y §915):

| qué | dónde | resultado |
|---|---|---|
| la cota de A1 (H1, H4, H11) sobre las pruebas de producción | `instrumento_cota`, en cada canon | 28 pruebas de los 23 probadores: q de 40 a 42, 90 tocados frente a T ≥ 128, rango pleno; «cubre» en las 23 |
| R3 con las columnas pareadas: K entero | `instrumento_r3`, `K_ENTERO_PAREADAS`, con la semilla 1 y las posiciones adversas del §907 | las cinco geometrías de producción: T = 128 con 10 trozos, 424 = 420 + 80 (con una columna, 254 de 424); T = 256 con 2 y con 9 trozos, 936; T = 512, 1960; T = 1024, 4008; ev(M_r) 4 de 4 en todas |
| M_lin en las cuatro de T = 128 con columnas lineales (guarda) | `instrumento_mlin`, el test largo (§914) | el umbral conjunto cumple R3 con su máscara lineal sola (424 de 424); la autorización delegada, la edad y el reembolso v2 no (406 de 424); el reembolso v1 no tiene lineales: a esas cuatro R3 se la da L9 |
| el censo de la 134 y la huella de cada composición | `CENSO_134`, `HUELLA_C` | las cinco de T = 128 remedidas (§914); las dieciocho restantes no cambian de traza ni de AIR |
| las bandas de bytes | `instrumento_revela`, `BANDAS` | las cinco de T = 128 entran en sus bandas; ningún byte de banda movido |
| K entero en las geometrías grandes, el test largo (§915) | `instrumento_r3`, `k_entero_en_las_geometrias_grandes`, en release sobre el árbol del corte, 1.695 s | T = 512: 1956 + 80 = 1960 de 1960; T = 1024: 4004 + 80 = 4008 de 4008, con la semilla 1 y con las posiciones adversas; con una columna, 1022 y 2046: las cifras de `K_ENTERO_PAREADAS` |
| M_lin en las 23 familias, el test largo (§915) | `instrumento_mlin`, `mide_m_lin_en_las_23_familias`, en release sobre el árbol del corte, 8.499 s | las 23, cada una con sus tres instancias (semilla 1, semilla 2 y la adversa del §907), reproducen el censo de la 134 (`CENSO_134`), `MLIN_MEDIDO`, `MLIN_INSTANCIAS` y la huella de cada composición (`HUELLA_C`) tal como están clavados: ninguna discrepancia; es guarda, no hipótesis |

**Lo que no cubre**, sin cambio del apartado 7 salvo lo resuelto: la solidez; H7 —ni el fork ni la
construcción están auditados— y la revisión ajena, que este documento pide y no sustituye; Blake3
fuera del modelo y el adversario cuántico; los canales laterales del probador; las pruebas de la
0.3 y las de la 0.4 conservadas bajo su versión, que son lo que eran; H3, H8 y H9, supuestas; y lo
que el nodo ve por ser nodo y lo público por diseño.

**Qué le falta para ser promesa.** (1) La aceptación del autor, de quien es toda promesa
(`doc/METODO.md` §7): lo que se podría prometer con estas palabras está en la D-AM del RFC-0009,
su borrador del §915, y con él cambiarían D-A, `SECURITY.md` §3.bis y la cabecera de este
documento; (2) la revisión ajena de la 136, de una persona, que desde aquí es de los apartados 1 a
18 con la construcción del corte, como dice el dosier; (3) nada más que medir: lo que la
proposición pide se mide en cada canon o está demostrado. Hasta entonces, la promesa es el censo.
