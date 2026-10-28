# La revisión del agente del argumento de ocultación (§907)

> **Qué es este fichero.** El informe, íntegro y sin retocar, de la primera revisión del argumento
> de `doc/ocultacion-hvzk.md` hecha desde fuera de la sesión: un agente —otra instancia de Claude
> Code, sin la crónica ni las conclusiones de la sesión— al que se le dio el dosier
> (`doc/ocultacion-hvzk-dosier.md`) y una copia del árbol del §903 (`git archive`), en lectura
> pura: no ejecutó nada ni tocó ningún fichero. Lo pidió la sesión en el §903 y lo recoge el §907
> de `AUDITORIA.md`. **No es la revisión ajena de la 136 de `BACKLOG.md`**, que es de una persona
> que elige el autor: es la tercera revisión de la casa, más lejos de la sesión que las dos del
> §897 y el §898. Lo que dice se registra aquí tal cual, con sus aciertos y sus errores; su cierre
> está en el apartado 11 de `doc/ocultacion-hvzk.md` (la revisión del §907) y lo que pidió escribir
> y medir, en su apartado 16. Dos cosas que el lector debe saber al leerlo: la revisión es
> anterior al §905 —habla de la 135 por cuenta, sin su medida, y la medida la desmintió en parte— y
> su O4 da por buena una cota «sobre |E|» que el §907 corrige a «sobre |F|».

# Revisión ajena del argumento de ocultación (HVZK) de `zkssl/0.4` — la 136

Revisor: un agente externo al proyecto, en lectura pura sobre la copia
`scratchpad/revision_copia` (rutas relativas a ella). No se ejecutó nada; no se tocó ningún
fichero. Las fuentes de winterfell no bifurcadas (`winter-fri` y `winter-crypto` 0.13.1) se
leyeron del registro de cargo del contenedor, porque el argumento depende de lo que FRI abre y de
cómo la moneda deriva el nonce y las posiciones, y el fork no las trae.

## 1. Qué leí, en qué orden, y cuánto

En el orden del apartado 2 del dosier:

1. `doc/ocultacion-hvzk-dosier.md`, entero (150 líneas).
2. `spec/rfc/0009-lo-que-revela-una-prueba.md`: cabecera y D-A (líneas 1–100), D-I con sus notas
   §896–§902 (296–378), D-AL y D-AM (863–975). Unas 300 líneas; no leí D-B a D-AK ni «Seguridad».
3. `doc/ocultacion-hvzk.md`, entero (947 líneas), apartados 1 a 14, con sus marcas ⚠️.
4. El fork, con el documento al lado:
   - `crates/winter-prover/src/lib.rs` entero (`generate_proof` y `generate_proof_oculto`),
     `constraints/composition_poly.rs` entero (`segmentar`), `domain.rs`, `composer/mod.rs`
     (el DEEP del probador), `constraints/evaluator/default.rs`, `boundary.rs`,
     `periodic_table.rs`, `evaluation_table.rs` (`combine`), `channel.rs` (posiciones y
     molienda);
   - `crates/winter-air/src/air/oculta.rs` y `marca.rs` enteros; `divisor.rs`;
     `transition/mod.rs` (`combine_evaluations`); `context.rs`
     (`num_constraint_composition_columns`); `air/mod.rs` (coeficientes, periódicas, fronteras);
     `proof/mod.rs` (la estructura `Proof`) y `proof/ood_frame.rs` entero;
   - `crates/winter-verifier/src/lib.rs`, `channel.rs`, `composer.rs` y `evaluator.rs` enteros;
   - `crates/zk-ssl-air/src/sal.rs`, la implementación (líneas 1–330; no sus tests);
   - `crates/stark-experiment/src/lib.rs` (`ocultacion_encendida`), `crates/zk-ssl/src/lib.rs`
     (`proof_options`), y los `Cargo.toml` (el rasgo `concurrent`).
5. Upstream sin bifurcar: `winter-fri-0.13.1` (`prover/mod.rs`, `prover/channel.rs`,
   `verifier/mod.rs`, `folding/mod.rs`, `utils.rs`, `options.rs`) y `winter-crypto-0.13.1`
   (`random/default.rs`).
6. Los instrumentos, enteros: `crates/zk-ssl/src/instrumento_cota.rs` (936),
   `instrumento_r3.rs` (558), `instrumento_mlin.rs` (1262) e
   `instrumento_falsadores_hvzk.rs` (594).
7. Las AIR: `crates/stark-experiment/src/solvency.rs` (la AIR entera, por ser la familia más
   ajustada), y sobre las dieciséis familias de la D-AM un rastreo de `new`,
   `evaluate_transition` y `get_assertions` (ramas sobre entradas públicas, aserciones
   periódicas, tramo auxiliar).
8. `BACKLOG.md`: las entradas 131 y 135.

No leí: la crónica de `AUDITORIA.md` §896–§902 (el apartado 11 del documento resume las dos
revisiones de la sesión y me bastó), la nota 2024/1037 (no está en el árbol), el código completo
de las otras quince AIR, `SECURITY.md`, los tests de `sal.rs`.

## 2. Objeciones

Ninguna objeción de severidad alta: no encontré un paso del argumento que sea falso ni una medida
que mida otra cosa de lo que dice. Lo que sigue son huecos entre lo medido y lo enunciado, y un
lema que el instrumento aplica y el documento no escribe.

### O1 (media). R3 con M_lin está medida en UNA instancia de desafíos por familia, y las posiciones reales no son «genéricas» en el sentido que el documento necesita

Pasajes. `doc/ocultacion-hvzk.md`, apartado 6 (línea 489): «sobre z, cc y α_i, una caída de
rango es una condición polinómica (un menor que se anula) y su probabilidad se acota por el grado
sobre |E|; sobre las posiciones no es polinómica, y el instrumento tiene que probar varias
elecciones y decirlo». Apartado 14, «Lo que no hace» (línea 945): «No mide los desafíos de
ninguna prueba real: la geometría y las opciones son las de una prueba de cada familia, y los
desafíos los elige el test, como en el §898». Apartado 8 / D-AM: «estadísticamente
indistinguibles en el modelo de oráculo aleatorio programable, con ventaja menor que 2^{−60} más
Q·2^{−256}».

Lo que el código hace. La medida de la 134 corre con la semilla 1 y nada más: en el sello,
`instrumento_cota.rs:846` (`instrumento_mlin::mide(&*sonda, &g, &censo.lineales(), 1, …)`,
sólo la solvencia) y en el test largo `instrumento_mlin.rs:1175`
(`mide(&*s, &g, &censo.lineales(), 1, LARGO)`, `#[ignore]`, línea 1136). Las corridas con la
semilla 2 y con q = 40 y 41 (`instrumento_cota.rs:768`) son sólo de la geometría T = 64 y sólo
con las dos máscaras del §898, no con M_lin. El apartado 6 (y el 13) dice de R3 del §898 «con dos semillas
de desafíos que dan los mismos rangos» (línea 494), pero la segunda semilla de T ≥ 256 «no está
en el árbol».

Lo que el documento ya declara, lo declara; lo que añado:

1. Las posiciones «genéricas» no son el caso típico. Las 42 posiciones caen en |Λ|/8 cosets de
   plegado: 1024 con T = 256, 2048 con T = 512, 4096 con T = 1024. La probabilidad de que dos de
   las 42 compartan coset en la capa 0 es ≈ 1 − exp(−861/1024) ≈ 57 % para la solvencia, ≈ 34 %
   con T = 512 y ≈ 19 % con T = 1024; en la capa 1 (128, 256 y 512 cosets) las coincidencias son
   casi seguras. La instancia clavada tiene 42 cosets distintos en la capa 0 (el «rango de O» de
   la tabla del apartado 13, 7·42 + L/8, lo dice). Es decir: la mayoría de las pruebas reales de
   la solvencia tienen una O distinta de la medida.
2. Hay un suceso dependiente de las posiciones, de probabilidad Θ(1) por prueba, en el que la
   estructura de M_lin cambia: un punto del coset de plegado de una posición que coincide con un
   punto TOCADO de otra, x_j·μ = g·x_k (μ^8 = 1). Como g = ω^16 y μ ∈ ⟨ω^{2L}⟩, ocurre cuando
   p_j − p_k ≡ 16 − 2L·t (mod 16L); el número esperado de pares es 8·42·41/(16L): ≈ 1,7 con
   T = 256, ≈ 0,8 con T = 512, ≈ 0,4 con T = 1024. En ese punto el camino DIRECTO de toda columna
   lineal vale cero (P_ρ = Z_real·V_toc·w se anula en los tocados), y la segunda coordenada de
   D(x) sólo puede cubrirla el camino del cociente o el aleatorizador. En la solvencia el margen
   es el más fino de las dieciséis (4 lineales × 166 libres y un solo aleatorizador de 40
   dimensiones, frente a 632; «lineales y aleatorizadores» da 632 justos), así que es ahí donde
   una instancia distinta puede dar otro rango.
3. La cota de ventaja de la proposición no tiene término para esto. Sobre (z, cc, α_i) el
   argumento de Schwartz–Zippel vale (aunque el grado del menor no se dice; es del orden de
   miles, lo que da ~2^{−115}, de sobra); sobre las posiciones no hay cota y la proposición la
   enuncia como si la hubiera.
4. Lo que sí es independiente de las posiciones, y el documento podría decir: la cobertura de
   UNA dimensión sobre F por punto de la capa 0 por la columna aleatoria es un hecho de
   Vandermonde, no una medida. Su residuo es V_pos·(x−z)(x−z·g)(x−z̄)(x−z̄·g)·u con u ∈ F[x] de
   grado < L − q − 4, y en los 7c₀ puntos de los cosets (distintos entre sí, fuera de las
   posiciones y de H) el factor fijo no se anula, así que u ↦ (u(x))_x es sobreyectiva sobre
   F^{7c₀} siempre que 7c₀ ≤ L − q − 4 (294 ≤ 466 con T = 256). Lo mismo para el camino directo
   de cada columna lineal, con T − 2q − 6 en lugar de L − q − 4, en los puntos que no son
   tocados. Lo dependiente de las posiciones queda reducido a la segunda coordenada en los
   puntos de coincidencia del punto 2 y a los funcionales nuevos de la capa 1.

Qué la cierra, barato: en el sello, la solvencia con dos o tres semillas más (15 s cada una) y
con una instancia de posiciones que fuerce una coincidencia x_j·μ = g·x_k y una colisión de
cosets; en el test largo, las dieciséis con dos semillas. O un lema que haga la cobertura
independiente de las posiciones, por la vía del punto 4. Hasta entonces, la proposición de la
D-AM vale para la instancia medida y para lo que la cuenta de dimensiones sugiere, no para «toda
prueba» con una cota numérica.

### O2 (media). H12 se aplica como el instrumento la calcula, pero el lema que lo justifica no está escrito

Pasaje. Apartado 14, «Lo que vale»: «crece M con una máscara más que cumple lo que L3 pide de
una máscara —las filas aleatorias de una columna lineal, condicionadas a S_toc, son uniformes en
el núcleo de la evaluación en S_toc, que no depende del testigo, y su imagen en D es lineal y
pública—». Y el paso 2 (línea 853): «Lo que el verificador ve de los trozos —cada uno en las q
posiciones, en z y en z·g— lo descuenta el núcleo CONJUNTO con los aleatorizadores: rango(O∘M)
= rango([abiertos | O∘D]) − rango(abiertos)».

Por qué es un hueco. L3 pide de una máscara dos cosas: imagen lineal en D y uniformidad
INDEPENDIENTE de D_resto una vez condicionada la vista. Las filas aleatorias de una columna lineal
cumplen la segunda frente a lo abierto de la traza y frente a C(x_j) y C(z·g) (todo se anula en
S_toc), pero NO frente a las aperturas de los trozos: P_ρ mueve cada h'_k(y) en y ∈ S_ab (sólo
la suma Σ_k y^{ks}·h'_k(y) = C(y) queda fija). El instrumento lo resuelve bien —restringe el
espacio de parámetros (w_i, r_k) al núcleo conjunto de las aperturas de los trozos, con la
fórmula del rango—, pero la frase citada presenta M_lin como «una máscara más» de L3 sin decir
por qué el condicionamiento a las aperturas de los trozos no rompe la independencia. El lema que
falta es éste, y es corto:

> Sean A_L y A_r las aplicaciones lineales de (ρ_lin residual, r) en las aperturas de los
> trozos (h'_k(y))_{k, y∈S_ab}, y A_h(N) la contribución, no lineal, del testigo y de las filas
> aleatorias de las columnas no lineales. Condicionada a las aperturas o y a N, la pareja
> (ρ_lin, r) es uniforme en un trasladado del núcleo conjunto J = ker[A_L | A_r]: el prior es
> uniforme en un producto de espacios vectoriales y la condición es afín. El trasladado depende
> del testigo (a través de N y de o), la parte libre j ∈ J no. La fibra no es vacía para ningún
> ρ_lin porque A_r es sobreyectiva sobre el hiperplano de cada punto (la triangularidad de L2).
> Luego D = D' + M(j) + M_R(u) con D' en D₀ + K determinado por el resto de la vista y j, u
> uniformes e independientes de todo lo demás; y O(D) es uniforme en O(D₀) + O(K) si y sólo si
> O(M(J) ⊕ M_R) = O(K), que es exactamente rango([abiertos | O∘D]) − rango(abiertos) = rango(O|_K).

Sin este párrafo, un lector que compruebe M_lin contra la definición de máscara de L3 la rechaza
por la objeción 1 del §897 (mueve lo abierto) y no entiende por qué el instrumento resta rangos.
Conviene escribirlo como lema (L7, o H12 con demostración) y citar L2 desde él.

### O3 (media). Lo que el canon mide de la 134 es el censo y la solvencia; el rango de las otras quince familias es una foto

Pasaje. Dosier, apartado 3, fila H12: «medida en cada canon (el censo) y por familia». Apartado
14: «`MLIN_MEDIDO`; la solvencia en cada canon, las demás en el test largo
`mide_m_lin_en_las_23_familias`, una vez, con la semilla 1».

Lo que añado: en el sello, para quince de las dieciséis familias, lo único que se contrasta con
`MLIN_MEDIDO` es la COHERENCIA del veredicto declarado con la fila clavada
(`instrumento_cota.rs`, el bloque «§900»: `esperado = … if m.5 == m.2`), no el rango. Un cambio
en las restricciones de SendAir que conserve las cuentas del censo (30 lineales, 0, 28) —otra
pendiente, otra aserción, otro orden de columnas— no mueve ninguna guarda del canon aunque cambie
el rango de M_lin; sólo lo vería quien vuelva a correr el test largo a mano. El censo es una
guarda de ESTRUCTURA, no de rango. Para una promesa de seguridad conviene que al menos las
familias con el margen más fino (las de T = 512 tienen 422 libres por lineal y ≥ 13 lineales; la
solvencia está en el sello) o una por geometría se midan en cada canon, o que el canon exija la
fecha/sha de la última corrida del test largo frente al sha de los ficheros de las AIR.

### O4 (baja). La cuenta de la ventaja de la proposición está incompleta

Pasaje. Apartado 5, final, y apartado 8: «ventaja acotada por el 2^{−60} de H3 más las consultas
al oráculo entre 2^{−256}». El 2^{−128} del condicionamiento al grado exacto L − 2 (S5, línea
445) no se suma; la caída de rango en (z, cc, α_i) se dice acotable «por el grado sobre |E|» sin
dar el grado; y las posiciones (O1) no tienen término. Ninguno de los dos primeros cambia nada
(son 2^{−115} o menores); el tercero sí. La cuenta de H3 la rehíce y es correcta: cada
coincidencia z̄ = z·g^e es z^{p−1} = g^e, p − 1 soluciones en E, ≈ 2^{−64} por ecuación.

### O5 (baja). Dos imprecisiones de la vista del verificador

- P5: «cada hoja es Blake3(item ‖ sal) con sal = Blake3(32 bytes de `OsRng`)». Lo que
  `sal.rs` hace (líneas 194 y 254) es hoja = `merge(hash_elements(fila), sal)` con sal =
  `H::hash(32 bytes)`: el item es el resumen de la fila, no la fila, y la sal es un resumen. No
  cambia L4, pero «item» debería decir «resumen de la fila» para que (a) se lea bien: lo que el
  adversario tendría que adivinar sin la sal es la fila entera, no un resumen que conozca.
- Apartado 2: (V5) «las raíces de las capas de FRI y el resto entero» no nombra el compromiso del
  resto (`hash_elements(resto)`, el último de `commitments`, que `read_remainder` del fork ata);
  y `num_unique_queries` (un `u8` de la `Proof`) no está en V1–V8. Los dos son funciones de lo
  listado; conviene decirlo para que el modelo de la vista y la `Proof` coincidan campo a campo.

### O6 (baja). L0 describe la molienda del rasgo `concurrent` apagado, y no lo dice

Pasaje. L0 (línea 177): «la búsqueda del nonce —el primero cuyo hash con la moneda lleva 21
ceros— corre sobre salidas del oráculo que el simulador NO fija». El código es
`(1..u64::MAX).find(…)` sin `concurrent` (`winter-prover/src/channel.rs:179`) y `find_any`
con él (línea 185). Comprobé que ningún `Cargo.toml` del espacio de trabajo activa `concurrent`
(sólo lo define `winter-prover`), así que hoy L0 es exacta. Pero con `find_any` el nonce real no
es el primero —depende del reparto entre hilos— y un simulador «con la misma búsqueda» no
reproduciría su distribución; no depende del testigo, no fuga nada, y es una desviación formal.
El documento debería atar L0 al rasgo apagado, o decir que el simulador corre la misma búsqueda
paralela.

### O7 (baja). El grado del DEEP se aserta con pánico, en producción

Pasaje. S5: «condicionado a grado exactamente L − 2 como el probador aserta». El aserto es
`assert_eq!(trace_length - 2, deep_composition_poly.degree())` en `generate_proof_oculto`
(`winter-prover/src/lib.rs:729`) y otro igual dentro de `add_trace_polys`
(`composer/mod.rs:165`): con probabilidad ≈ 2^{−128} un testigo VÁLIDO hace pánico, contra la
letra de D-AH («un testigo malo es un `Err`, nunca un pánico» —aquí ni siquiera es malo—). No
toca la ocultación (el simulador condiciona igual y la diferencia es la que el documento dice),
pero el documento lo presenta como una condición del protocolo y es un pánico heredado de
winterfell.

## 3. Lo que di por correcto, y con qué

Cada línea: la frase del documento frente al código que la sostiene.

- **P1** («T elementos de F detrás de las T filas reales de cada columna principal; después L
  elementos de F para … la columna aleatoria; y después, si hay tramo auxiliar, T elementos de E»)
  frente a `generate_proof_oculto`: el bucle `for c in 0..principal.num_cols()` con `t` sacadas,
  `(0..2 * t).map(…)` para las columnas aleatorias, y `for _ in 0..t { col.push(sacar::<E>) }`
  en el tramo auxiliar; la moneda `DefaultRandomCoin::new(&[])` resembrada con
  `Blake3::hash(&semilla)` y nunca con nada de la pública (H10).
- **P4** y **L2** («h'_k = h_k + x^s·r_k − r_{k−1}, con r_{−1} = r_{N−1} = 0 y r ∈ E[x] de grado
  < m … s = L − m») frente a `segmentar`: `s = trace_len - m`; `t[k] -= r_ant[k]` para k < m;
  `t[s + k] += r[k]`; `r` se saca con `moneda.draw::<E>()`; el último trozo no recibe r; el
  aserto de que los coeficientes más allá de s·N son cero; `m < trace_len && num_cols >= 2` (H5).
- **P2** frente a `Oculta::new`: `n = base.num_transition_exemptions() + t`, el AIR interno con
  `t = l / 2` y `main_trace_width − columnas`, el `ce` que sube hasta que `cabe_con`; las
  transiciones, aserciones, periódicas y tramo auxiliar delegados sin tocar. Con n = n_base + T
  las filas exentas son T − n_base … 2T − 1: la última real → primera aleatoria y todas las
  aleatorias, como dice el documento. Λ ∩ H = ∅: el offset es `GENERATOR` = 7, de orden p − 1,
  que no divide a 16L.
- **P5/L4** frente a `sal.rs`: `with_options` saca una sal por hoja (`sal::<H>()`, `OsRng`),
  `con_sal` hace `merge(item, sal)`, `open_many` devuelve las sales de las hojas abiertas y
  `verify_many` las vuelve a mezclar; los tres compromisos pasan por `V::new` (traza y cociente
  en `matrix/col_matrix.rs:285` y `row_matrix.rs:227`; FRI en `build_layer_commitment` de
  `winter-fri`), así que las tres clases de hoja van saladas.
- **P6/P11** frente a `perform_verification`: la consistencia OOD sólo en z, con
  `paso = L − m` (`winter-verifier/src/lib.rs:363`) y `current_row()`; en z·g no se comprueba
  nada; el DEEP lo rehace `DeepComposer::compose_columns` con la fórmula de P7 (un mismo cc por
  columna para los dos cocientes, `self.cc.trace[i]` y `self.cc.constraints[i]`) y FRI lo
  comprueba contra la hoja de la capa 0 (`evaluations != query_values` en `verify_generic`).
- **P7** frente a `composer/mod.rs` (probador): `acc_trace_poly` resta el valor OOD, se divide por
  (x − z) y (x − z·g) y se suman; un coeficiente por polinomio.
- **P8/P10/V8** frente a `winter-fri` 0.13.1: `transpose_slice` agrupa de ocho en ocho (la fila
  i son los valores i, i + n/8, …: el coset {x·μ}), `apply_drp` pliega con α, `set_remainder`
  interpola y manda `len / blowup` coeficientes enteros; el verificador lee la hoja entera del
  coset (`read_layer_queries`), interpola la fila y evalúa en α; el resto queda atado a su
  compromiso en el canal del fork (`read_remainder`). Los números de capas y restos por T (1/16,
  2/8, 2/16, 2/32) salen de `num_fri_layers` con `max_remainder_size = 32·16`.
- **P9/L0** frente a `DefaultRandomCoin`: `check_leading_zeros` mira `merge_with_int(seed,
  nonce)`; `draw_integers` resiembra con el nonce y saca las posiciones de
  `merge_with_int(seed', counter)` para counter = 1..42, con repetición; el probador
  (`get_query_positions`) y el verificador hacen `sort_unstable(); dedup()`. Son salidas del
  oráculo distintas de las de la molienda: lo que L0 necesita para fijar unas sin fijar las
  otras. La afirmación «no depende del testigo» se sostiene.
- **Apartado 2** frente a `Proof { context, num_unique_queries, commitments, trace_queries,
  constraint_queries, ood_frame, fri_proof, pow_nonce }`: nada de la prueba queda fuera de
  V1–V8 salvo lo de O5 (funciones de lo listado). El `OodFrame` lleva las dos filas de la traza
  (la aleatoria incluida) y las dos de los trozos, y `merge_ood_evaluations` mete las cuatro en
  la moneda.
- **L3, Q[P](x₀)**: rehecho a mano; 2·(P(z·g) − P(z))/(z·g − z). Que D(x₀) es lo ÚNICO público de
  D fuera de D(x_j) lo di por correcto con un argumento que el documento no escribe y vale la
  pena: el residuo de un solo trozo h'_k (grado < L, nulo en S_ab) es (x−z)(x−z·g)·V_pos·v con
  deg v ≤ L − q − 3, y su imagen Q[·] es ℓ·V_pos·v, que recorre TODO K = ℓ·V_pos·E[x]_{≤L−q−3};
  luego ningún funcional de D fuera del generado por {ev_{x_j}, ev_{x₀}} es función de lo
  abierto. K es el espacio correcto como soporte; la distribución sobre él es R3.
- **L3, rango de O**: 8c₀ + (L/8 − c₀) sobre E cuando 8c₁ ≥ L/8, coherente con la tabla del
  apartado 13 (550, 422, 358; 127 = D entero con T = 64) y con «sin determinar» = 2·rango − 2q
  (− 2 sólo con T = 64, donde x₀ cae en el espacio de filas). La «una dimensión por punto» de la
  columna aleatoria es cierta y, como digo en O1.4, demostrable.
- **`instrumento_r3`** frente al apartado 6: `Mirilla::observa` reproduce exactamente lo que
  `FriProver` abre (misma transposición, `fold_positions`, `apply_drp`, mismo offset, el resto
  como coeficientes); K = V_pos·(x − x₀)·u; M_R = cc_R·ℓ·V_pos·(x−z̄)(x−z̄g)·u con u ∈ F[x]
  —deduje la fórmula del residuo de R y coincide—; M_r = (cc'_k·x^s − cc'_{k+1})·V_pos·ℓ·x^i
  —deducida igual—; los rangos sobre F por `sobre_f` con el par (fila, u·fila) para los
  espacios sobre E; los asertos de inclusión. Las cifras clavadas son coherentes con la
  estructura (632 = 2·294 + 44; columna = 294 + 44).
- **`instrumento_mlin`**, lo central de la 134:
  - `Sonda::composicion` NO es una réplica del evaluador del probador: es, línea a línea, el
    `evaluate_constraints` del verificador (`get_transition_constraints` →
    `combine_evaluations` con el divisor en x; `get_boundary_constraints` →
    `evaluate_at`; las periódicas como `p(x^{L/c})`, igual que `periodic_table.rs` con
    `num_cycles = L / len`). Como C es un polinomio y la fórmula racional coincide con él en todo
    x donde los divisores no se anulan —y el dominio de restricciones 7·⟨ω_ce⟩ no toca H ni las
    exenciones—, la interpolación de `muestra` sobre ese dominio devuelve la parte lineal de la C
    del probador exactamente. Que esa fórmula es la del probador lo garantiza que las 28 pruebas
    reales pasan la comprobación OOD en z. La validación «contra el marco OOD de una AIR con
    periódicas» que el dosier echa en falta (ataque 4) no añadiría nada que no dé ya el
    verificador en cada prueba.
  - La sonda: segunda diferencia nula a lo largo de e (afín) y pendiente igual en dos marcos
    aleatorios (pública). Caza los productos con otra columna y el bilineal P_i(x)·P_i(g·x) (la
    pendiente lleva la otra entrada del marco, que cambia entre f y f'), y un falso «lineal» cuesta
    que un polinomio no nulo se anule en tres puntos aleatorios de E. «Lineal» implica afín en
    (P_i(x), P_i(g·x)) conjuntamente, que es lo que `pendientes` usa después.
  - M_lin: base Z_real·V_toc (V_toc con los conjugados, asertado en F[x]), libres = T − 2q − 6,
    la pendiente en cada punto del dominio de restricciones para la fila actual y la siguiente
    (`kg = (k + ce) % ce_tam` es el índice de g·x_k), el troceado con paso s como `segmentar`, los
    aleatorizadores sumados y restados como allí, las aperturas de los trozos en S_ab como bloque
    «abiertos», D = Q[directa + Σ cc'_k·trozo_k] + Φ·u. La fórmula rango([A | B]) − rango(A) =
    dim B(ker A) es correcta, y la comprobación de inclusión (rango con K menos rango(abiertos)
    = rango(K)) prueba B(ker A) ⊆ O(K), como deduje. Con M_lin vacío reproduce `R3_MEDIDO`,
    que valida O, K y las dos máscaras pero NO el camino del cociente; a ése lo valida el juguete
    (76 = 2·38 exactas dentro de K).
  - Las entradas públicas vacías (ataque 5 del dosier): en las dieciséis AIR las entradas
    públicas sólo son VALORES de `Assertion::single` (p. ej. `banda.rs:150–159`,
    `cobro_pendiente.rs:164–175`, `pago_en_curso.rs:171–182`) o se guardan; ningún `if`/`match`
    sobre ellas en `new`, `evaluate_transition` ni `get_assertions`; el número de aserciones es
    una constante de `new` (la solvencia: `AirContext::new(trace_info, degrees, 1, options)`).
    Ninguna de las dieciséis usa `Assertion::periodic`/`sequence` ni tramo auxiliar (sólo
    `circuit_edad.rs` y `zk-ssl-air/src/lib.rs`, la edad). La forma de C no depende de las
    entradas públicas en las dieciséis.
  - El censo de la solvencia (4 lineales, 0, 1 no lineal) frente a su AIR: `COL_BIT` entra en
    bit·(bit − 1) (grado 2); `COL_ACC` y los tres transportes entran linealmente con pendientes
    constantes o periódicas (`first`, `cont`, `link_*`, matrices fijas). Coherente.
- **`instrumento_cota`**: la Lagrange `Z_H(y)·h_r/(2T·(y − h_r))` es la base correcta sobre H de
  tamaño 2T; `rango_f` parte cada fila de E en sus dos coordenadas; S_toc son los abiertos, los
  g·x_j y g²·z sin repetidos. Añado que el «rango pleno» de H1 no es sólo medido: la propia
  demostración de L1 lo da para cualesquiera puntos distintos con |S̄| ≤ T (Vandermonde con nodos
  distintos y el truco (R' + conj R')/2); lo supuesto es sólo la distinción (H3). El documento
  lo dice como prueba de vida; podría decirlo como teorema.
- **Los falsadores**: `vista` rehace la moneda exactamente en el orden de `perform_verification`
  (contexto y entradas, raíz de la traza, α, raíz del cociente, z, el resumen del marco OOD, cc,
  un α por compromiso FRI, el nonce, las posiciones con `dedup`), y los tres ataques son los que
  el documento describe; «45 ecuaciones sobre 33 incógnitas sólo consistentes con los desafíos
  reales» es una validación fuerte de la moneda rehecha, como dice el apartado 13.
- **L5**: `ocultacion_encendida` llama dos veces a `sal::semilla()` (32 bytes de `OsRng` cada
  una); las dos monedas se siembran aparte de la pública y la pública no las ve.
- **Apartado 2, la cuenta de lo tocado**: las transiciones de winterfell 0.13 leen sólo
  (fila, fila siguiente) (`EvaluationFrame`), las fronteras sólo la fila actual, las periódicas
  no son columnas de la traza: S_toc = S_ab ∪ g·{x_j} ∪ {g²·z}, 2q + 3 puntos.
- **El orden del simulador frente a Fiat–Shamir** (S1–S7 con L0): comprobé que la cadena de
  programación es consistente: las posiciones se eligen antes de construir ningún árbol; los
  prefijos que se programan (α tras la raíz de la traza, z tras la del cociente, cc tras el
  resumen OOD, α_i tras cada raíz FRI, las posiciones tras el nonce) llevan raíces con resúmenes
  frescos; el compromiso del resto y la molienda son salidas honestas del oráculo.

## 4. Lo que el dosier no me deja responder, y qué haría falta

1. **Los desafíos reales (O1).** Sin ejecutar, no sé qué rango da R3 con M_lin en la solvencia
   con otra semilla, con una colisión de cosets o con una coincidencia x_j·μ = g·x_k. Hace falta
   correr `instrumento_mlin::mide` con varias semillas y con posiciones forzadas (basta un
   `Desafios` construido a mano), e imprimir también `rango_lineales` SIN aleatorizadores, que
   el instrumento no reporta: diría cuánto del margen de la solvencia descansa en los 40 del
   aleatorizador.
2. **La corrida de exploración del §898** con la semilla 2 para T ≥ 256 no está en el árbol; el
   apartado 13 la cita. Un fichero con su salida, o su inclusión en el test largo, la haría
   revisable.
3. **Un lema de cobertura independiente de las posiciones.** La vía de O1.4 (Vandermonde en los
   puntos de los cosets) cubre la capa 0 coordenada a coordenada si, además de la columna
   aleatoria, al menos una columna lineal con T − 2q − 6 ≥ 7q + (coincidencias) libres aporta la
   dirección cc_i·c(x), F-independiente de cc_R·c(x) cuando cc_i/cc_R ∉ F. Con T ≥ 512 (422 y
   934 libres frente a 294) es inmediato; con T = 256 (166 < 294) hacen falta las cuatro
   columnas juntas y el argumento es de rango conjunto, no de Vandermonde. Lo que no cubre esa
   vía son los L/8 − c₀ funcionales de la capa 1 (22 sobre E con T = 256). Escribirlo convertiría
   «medida en una instancia» en «demostrada salvo la capa 1», que es lo que una promesa necesita.
4. **La 135, con números.** El dosier (ataque 1) pregunta si la cuenta fina rescataría las cinco
   de T = 64. Por cuenta, y sólo por cuenta: cada columna lineal condicionada a S_ab conserva
   T − q − 4 = 18 libres; C(x_j) y C(z·g) son 2q + 2 = 86 dimensiones sobre F EN TOTAL (una suma
   para todas las columnas, no por columna); y FRI entrega D entero, 168 sin determinar, con 126
   cubiertas por las dos máscaras del §898. NullifierThresholdAir (8 lineales: 144 − 86 = 58;
   126 + 58 = 184 ≥ 168) es la única que pasa la cuenta; ThresholdAir (7: 126 − 86 = 40;
   166 < 168) no, y RefundAir (0), RefundAirV2 (4: 72 < 86) y EdadAir (4 + tramo auxiliar) no
   llegan ni a C. Luego la 135 puede, como mucho, rescatar una de las cinco; las otras cuatro
   necesitan la 131 o la 133 por cuenta, no por estructura. Haría falta el lema L1' que la 135
   pide y la medida con V_ab.
5. **La sal (ataque 7).** Con R3 en verde, la pregunta «qué fallaría sin ella, hoja a hoja» tiene
   respuesta en el modelo: una hoja de la traza no abierta en x ∉ S_toc lleva R(x) uniforme
   (82 a 2002 libres frente a los 46 abiertos) y, con T ≥ 256, cada P_i(x) uniforme (2q + 7 ≤ T),
   así que su preimagen tiene ≥ 64·(W + 1) bits de entropía; una hoja de los trozos, ≥ 128·(N − 1)
   (q + 3 ≤ m); una hoja de FRI de un coset no abierto, ≥ 8·64 por la sola coordenada que mueve
   la columna aleatoria en cada punto (7c₀ + 8 ≤ L − q − 4). Nada se rompería con T ≥ 256; con
   T = 64 las hojas de FRI tendrían entropía cero, pero D ya es público entero. Lo que la sal
   compra es que el SIMULADOR no tenga que argumentar entropía hoja a hoja (L4(a)); sin ella el
   simulador cambia (L4(b) o un lema por clase de hoja). La recomendación de conservarla es
   correcta y barata de sostener; retirarla pediría reescribir S6 y L4.
6. **La nota 2024/1037.** No está en la copia ni la leí fuera; el apartado 10 sigue pendiente
   tal como lo dice.
7. **Dos cosas del fork que no son de este dosier pero que vi al leer**, para la auditoría de
   H7: (a) `ConstraintDivisor::from_assertion(assertion, L)` toma `num_steps` sobre L, así que
   una `Assertion::periodic` de un AIR interno se exigiría también en las filas aleatorias
   (hoy ninguna de las 23 la usa; si una lo hiciera, el probador oculto fallaría en el aserto de
   `segmentar`, no fugaría nada); (b) las exenciones de `Oculta` «se escriben sin la aserción»
   de `AirContext` y la segunda cota se decide en `cabe_con`, que sólo el probador mira.

## 5. Veredicto

La proposición de la D-AM para las dieciséis familias se sostiene en su estructura: la vista del
verificador honesto es la que el apartado 2 describe (lo comprobé contra la `Proof` y contra
`perform_verification`), K es el soporte correcto de lo que FRI abre sin determinar (y di un
argumento cerrado de que D(x₀) es lo único público de D fuera de las posiciones), L1 y L2 son
teoremas con hipótesis honestas, R3 es suficiente por el razonamiento de L3, y H12 es correcta
—con el lema del núcleo conjunto que el instrumento aplica y el documento no escribe (O2)—; la
sonda y los dos instrumentos de rango hacen lo que el documento dice que hacen, con la
composición del verificador y no una réplica, y las dieciséis AIR no cambian de forma con sus
entradas públicas. Lo que le falta para ser una promesa con la cota numérica que enuncia es lo
que O1 describe: R3 con M_lin está medida en UNA instancia de desafíos por familia, las
posiciones reales no son genéricas en el sentido del argumento (colisiones de cosets en más de la
mitad de las pruebas de la solvencia, coincidencias coset–tocado con frecuencia Θ(1)), la cota
2^{−60} + Q·2^{−256} no tiene término para ellas, y la familia donde eso importa —la solvencia,
con el margen más fino— es también la única que el canon remide. Con dos o tres semillas más y una
instancia de posiciones adversa en el sello, o con el lema de cobertura por puntos de O1.4, la
proposición pasaría de «vale para la instancia medida y para lo que la cuenta sugiere» a lo que
la D-AM quiere decir; hasta entonces, el autor puede aceptarla sabiendo que ese es el hueco, y que
no es de estructura sino de cobertura de la medida.
