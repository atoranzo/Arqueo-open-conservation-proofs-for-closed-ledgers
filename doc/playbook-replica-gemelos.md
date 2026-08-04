# B13/B14 · Paso 3 — PLAYBOOK de la réplica (los nueve gemelos)

Destilado del piloto send (§140-§143, commits `0451462`→`ac6cc30`).
Una sesión por circuito, rito completo. **Orden recomendado**: claim
(tres tramos, hermano de two_phase: máxima transferencia del patrón) →
burn (cuentas+frozen) → mint, audit, mint_climb, recovery,
recovery_climb (un tramo) → freeze, frozen_climb (solo profundidad,
sin salt: gemelo mínimo).

## Optimización de método (aprendida en send, para los nueve)

**Gemelo-primero; SB0 DENTRO del gemelo.** Send hizo SB0 sobre el
legacy porque precedió a la decisión C (§142). Los nueve no: nace la
copia, y la unificación de representaciones se hace ya en ella. El
legacy conserva sus literales — irrelevante: se borra en el flip.

## R0 — El censo previo (media hora que compra el resto)

1. Leer la región de hoja ENTERA del circuito (spec §5: las variantes
   existen — mint no tiene carril de resta, audit no muta estado,
   recovery COPIA el salt, costura 52).
2. Grep de las CUATRO formas (§139/§140): `const ROW_`, los offsets
   `(N + level)`, los rangos del `match` (§141 ya censó cuáles tiene
   cada uno), y los COMENTARIOS con números.
3. Llamadores internos de `build_trace` (en send: un nexo `scenario()`
   + N sitios en dos moldes — buscar el equivalente).
4. Acoplamientos frozen: ¿usa `frozen_climb`/`frozen_leaf` compartidos?
   ⚠️ `frozen_climb` CLAVA 24 por dentro (§143.1) — el gemelo con
   frozen-32 necesita su `frozen_climb_32` local (cláusula de retirada).
5. Colisiones de nombre: send ya tenía `COL_SALT` (el aleatorio del
   pendiente) — el testigo nuevo es SIEMPRE `COL_LEAF_SALT`.
6. ¿Doctests accidentales? (fences ``` en //! deben ser `text`).

## R1 — Nacimiento (= SB1.a de send)

Copia compilante + cabecera-andamio con **cláusula de retirada** +
`pub mod <circuito>_salted;` en `lib.rs` (orden alfabético). El
guardián de ranuras lo absorbe solo (`listdir`). Verificar: suite
stark +N tests del gemelo, capa intacta.

## R2 — SB0 interno (= SB0.1-0.4, en el gemelo)

Bloque `CYC_*` derivado de `(CYCLE_LENGTH, TREE_DEPTH[,
FROZEN_DEPTH])`; `ROW_*` derivadas con clavos transitorios; literales
de bucles a nombres; rangos del `match` a la convención única
`(CYC_arranque..CYC_fin)` + `debug_assert!` en los `place_*`; clavos
fuera y **guarda de presupuesto** `ROW_FINAL < TRACE_LENGTH`;
comentarios reanclados a nombres. Cada sub-paso: suite verde.

## R3 — El mundo envuelto (= SB1.b)

`CYC_SALT` en la cadena (todo se corre solo); `COL_LEAF_SALT` (+4,
ancho +4); brazo del tercer merge (digest arrastrado, CUATRO limbos
del salt al rate); parámetro `leaf_salt: Digest` + difusión; el nexo
de tests al nativo salado — `derive_leaf_salt_wide(key)` +
`native_leaf_salted` (hogar: `circuit_settlement.rs`), NUNCA un
literal de juguete. ⚠️ recovery: el salt se COPIA (§93.4/costura 52) —
su escenario refleja copia, no derivación fresca del estado nuevo.

## R4 — Las seis (= SB1.c)

`C_SALT_{CAP,DIG,IN}_{A,B}` (24 ranuras) ENTRE el último bloque y la
constante que la casa dejó al final; `P_LINK_SALT` junto a
`P_LINK_LEAF` (mismo orden en `get_periodic_column_values`); grados:
24 × `with_cycles(1, full)` en la posición de índice; evaluate con el
molde de los enlaces de hoja. Verificación fuerte gratis: el guardián
de ranuras (estático) + `no_constraint_is_vacuous` (dinámico) cubren
las 24.

## R5 — Mutaciones + espejo (= SB1.d, innegociable, spec §4)

(a) limbo del salt testigo alterado (veneno = **honesto + 1**) →
`C_SALT_IN` rechaza; (b) hoja sin envolver a la transición de
colocación (AMBAS mitades del estado siguiente, bit-agnóstico) →
`C_PLACE` rechaza; + nativo↔circuito limbo a limbo (sin envolver en
`ROW_SALT_LINK`, envuelta en `ROW_LEAF_DONE`, ambos carriles). El
arnés: catch_unwind + prove + verify, `!verifica`.

## R6 — Frozen-32 local (= SB1.e; solo claim, burn, freeze, frozen_climb)

Quitar el import compartido; `const FROZEN_DEPTH: usize = 32;` local
con la nota de flip-en-una-línea; `frozen_climb_32` en tests si el
circuito subía con el compartido. En freeze/frozen_climb (sin salt):
R3-R5 no aplican; su gemelo es R1+R2+R6 + sus propias mutaciones de
profundidad (un camino de 24 declarado como de 32 debe rechazar).

## R7 — Sello por circuito

Fila de la tabla §3 de la spec (hoy / +salt / +frozen-32 / holgura /
veredicto — **no se asume que 1024 alcanza**); asiento en AUDITORIA
con la cadena de md5 y hashes; BACKLOG 50 al día; guardianes 34·28+.

## Reglas vivas que ya mordieron

- Los números de comentario se revisan EN el paso que mueve la
  geometría (§143.2 — mordió al asistente en SB1.b).
- Suite canónica SIEMPRE `--release` (§140, nota de instrumento).
- Guardas atómicas multi-fichero: TODO verificado antes de escribir
  NADA; anclas con conteo esperado explícito cuando son globales.
- Bloques con centinela `=== fin ===`; heredocs cierran solos.

## Adelanto del flip (D4, para cuando los diez existan)

UNA release: la capa cambia imports a los gemelos; se borran legacies,
helpers `_32` y consts locales; `FROZEN_DEPTH` compartida a 32 (o
`frozen_climb` pasa a iterar el camino — decisión de ese paso, §143.1);
fixture legacy prueba la migración; snapshot v7.
