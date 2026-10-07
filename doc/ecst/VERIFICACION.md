# Registro de verificación de los borradores ECST, HBS-STATE y el Internet-Draft

> ⚠️ **Registro generado, no prosa.** Sale entero de `doc/ecst/borrador/final.json`, que fusiona el veredicto de un
> verificador y el de un escéptico que intentó refutarlo, por afirmación. Lo produjo un asistente de IA generativa
> (Claude, de Anthropic) en dos sesiones —2026-09-28 y 2026-09-30—; nada de aquí ha pasado todavía por la aceptación
> del autor que describe `GENAI.md`. Árboles: Arqueo `main` 71c5aad (S582); `hbs-state` a960828.

## Qué se verificó y cómo

El autor pegó en una sesión un conjunto de borradores generados con IA: una explicación de `hbs-state`, un
Internet-Draft `draft-toranzo-hbs-state-00/-01` con sus consideraciones de seguridad y referencias, un correo de
presentación al CFRG con respuestas a objeciones, varias versiones de un artículo sobre *Evidence-Carrying State*
*Transitions* (ECST), sus extensiones —especificación formal, protocolo de auditoría, modelo de amenazas, resiliencia
frente a canales laterales— y una bibliografía en BibTeX. Sus afirmaciones se partieron en siete bloques, con el
catálogo en `doc/ecst/borrador/flujo-verificacion.js`. Cada bloque tuvo un verificador que abrió las fuentes y un
escéptico que re-derivó cada veredicto; los bloques G y H, verificados sobre d531c80, se re-comprobaron contra 71c5aad.

Veredictos: **CONFIRMADA** (las fuentes dicen eso), **PARCIAL** (necesita un matiz), **FALSA** (las fuentes lo
contradicen), **SIN_FUENTE** (no hay fuente: típicamente una cifra inventada), **NO_VERIFICABLE** (no se puede
comprobar desde aquí; casi siempre, Zenodo o el datatracker bloqueados). La gravedad (`alta`, `media`, `baja`) dice
cuánto daño haría publicar la afirmación tal cual.

⚠️ **Lo que este registro NO es**: una auditoría de Arqueo ni de `hbs-state`. Sólo contrasta lo que los borradores
afirman. Las rutas `path:línea` son del árbol en 71c5aad; con el árbol moviéndose, envejecen.

## El recuento

| bloque | tema | afirmaciones | CONFIRMADA | PARCIAL | FALSA | SIN_FUENTE | NO_VERIFICABLE | el escéptico revocó |
|---|---|---|---|---|---|---|---|---|
| H | hbs-state y la especificación HBS-STATE v0.3 | 58 | 24 | 17 | 8 | 8 | 1 | 1 |
| G | el guardián del índice de Arqueo y su relación con HBS-STATE | 36 | 15 | 12 | 7 | 2 | 0 | 0 |
| L | registro encadenado, cabeza de época, historia, paquete y anclaje (Arqueo) | 39 | 10 | 17 | 9 | 3 | 0 | 1 |
| C | pago en dos fases, custodia, qué revela una prueba, conservación, uso único, recuperación y poderes del operador (Arqueo) | 33 | 10 | 15 | 6 | 2 | 0 | 4 |
| M | las cifras de las «evaluaciones» de los borradores, verificación formal y backends | 30 | 6 | 13 | 5 | 6 | 0 | 1 |
| B | la bibliografía | 58 | 30 | 8 | 17 | 0 | 3 | 2 |
| P | el proceso IETF/IRTF, el Internet-Draft tal como está escrito, y la lógica formal de ECST | 51 | 9 | 23 | 16 | 2 | 1 | 2 |
| **total** | | **305** | **104** | **105** | **68** | **23** | **5** | **11** |

## Las 72 afirmaciones de gravedad alta que son falsas o no tienen fuente

Son las que, repetidas en un depósito, publicarían algo falso sobre el código, la especificación o una cita.

| id | veredicto | lo que decía el borrador |
|---|---|---|
| [H6b](#h6b) | FALSA | ECST: KeyAtZero: el sistema exige intervención o aislamiento |
| [H6c](#h6c) | FALSA | I-D: Under a Persisted SK model, KeyAtZero MUST be classified as a protocol violation |
| [H6f](#h6f) | SIN_FUENTE | ECST: Colapso controlado (C<K o corrupción): destruye las claves operativas en RAM y marca el árbol HBS como agotado |
| [H6h](#h6h) | FALSA | ECST 11.7: tras un corte K (memoria volátil) desaparece; la evaluación se reformula como R(DurableEvidence) -> {InSync, CounterAhead, Indeterminate} |
| [H7c2](#h7c2) | SIN_FUENTE | otro borrador: ext4 1.82 ms, tmpfs 0.03 ms |
| [H7d](#h7d) | FALSA | respuesta CFRG: the measurement takes a statistical distribution over N iterations |
| [H7e](#h7e) | SIN_FUENTE | I-D: inspect /proc/mounts; MUST NOT permit tmpfs, ramfs, procfs paths |
| [H7f](#h7f) | SIN_FUENTE | explicit operator override for ultra-fast NVMe |
| [H9b](#h9b) | FALSA | I-D: N3 (Judgment): enforces policy judgments (identifying fatal states and operator-defined actions) |
| [H10a](#h10a) | FALSA | Vectores del I-D: {"version":"0.3","test_cases":[{"id":"TC-01",...,"expected_state":...}]} |
| [H10b](#h10b) | SIN_FUENTE | draft-01 añade "algorithm":"LMS_SHA256_M32_H10" |
| [H11c](#h11c) | FALSA | implicación de que ese layout es el de RFC 8391 |
| [H12b](#h12b) | FALSA | donde lleva operando en entornos reales de firma / README: "in production for a year" |
| [H17a](#h17a) | SIN_FUENTE | I-D: IndexGuard MUST acquire an exclusive non-blocking lock (flock/fcntl) |
| [H17b](#h17b) | SIN_FUENTE | I-D: multi-thread: atomic fetch-and-add or mutex |
| [G1f](#g1f) | FALSA | En KeyAtZero (estado de todo reinicio real) la implementación de origen 'falla cerrada' / se niega a arrancar (HBS-STATE §2: 'KeyAtZero cannot be resolved, only failed closed') |
| [G2a](#g2a) | FALSA | El guardián 'está en producción desde hace un año' / 'lleva operando en entornos reales de firma' |
| [G3d](#g3d) | SIN_FUENTE | Cifras de fsync '1,82 ms / 0,03 ms' |
| [G5b](#g5b) | FALSA | Cada época requiere la co-firma de testigos (para que el paquete verifique) |
| [G7b](#g7b) | FALSA | '13 de 25 muertes dejan el contador adelantado': es el caso normal tras un reinicio |
| [L1b](#l1b) | FALSA | Fórmula ECST: H_i = H(H_{i-1} \|\| S_{i-1} \|\| S_i \|\| O_i \|\| H(Pi_i)) |
| [L3a](#l3a) | FALSA | El test t1_cabeza_ata_la_historia se ejecutó sobre N=100 épocas |
| [L3b](#l3b) | FALSA | Se alteró la época k=10 |
| [L3c](#l3c) | FALSA | Se alteró un único byte de la prueba Pi_k |
| [L5b](#l5b) | FALSA | La verificación offline re-verifica las pruebas STARK de las transiciones (Verify(Pi_i)) |
| [L5c](#l5c) | FALSA | La prueba Pi_i se conserva como evidencia adjunta recuperable |
| [L5d](#l5d) | FALSA | El auditor recalcula la cadena de H_0 a H_N y ejecuta Verify(Pi_i) para cada transición |
| [L6a](#l6a) | FALSA | Crecimiento del log ~256 bytes por transición (almacenando solo H(Pi_i)) |
| [L6b](#l6b) | SIN_FUENTE | Reducción del 99,6 % |
| [L6c](#l6c) | SIN_FUENTE | Auditoría histórica offline ~125.000 bloques/s |
| [L6d](#l6d) | SIN_FUENTE | Sobrecarga de encadenamiento < 0,002 ms |
| [L9a](#l9a) | FALSA | Un tercero importa los vectores y la cabeza publicada y verifica toda la secuencia de evidencia histórica sin consultar el nodo |
| [C2a](#c2a) | FALSA | Custodia cero garantizada por tipos: apply_send() solo acepta la prueba y compromisos públicos, haciendo técnicamente imposible que la clave secreta sea transmitida a la capa del ledger |
| [C4c](#c4c) | FALSA | el auditor obtiene certeza matemática sin acceso a las bases de datos |
| [C5a](#c5a) | FALSA | Detección de Reutilización de Etiquetas (Nullifiers): garantiza que ningún activo pueda ser gastado dos veces en el árbol de consumos |
| [C7a](#c7a) | FALSA | Al forzar que el contador de recuperación sea público, el historial de gobernanza queda sujeto a la misma rigidez matemática que las transacciones, eliminando los canales laterales administrativos |
| [C7b](#c7b) | SIN_FUENTE | los circuitos ZK aplicados a primitivas administrativas cierran la brecha de confianza de los sistemas de gobernanza descentralizada |
| [C9b](#c9b) | FALSA | respetó las reglas del contrato inteligente |
| [C10a](#c10a) | FALSA | ECST-R: extendemos Arqueo para que el estado esté enmascarado mediante Pedersen Commitments |
| [C10c](#c10c) | SIN_FUENTE | ECST-R: claves de gasto por umbral, TEE, máquina de estados de migración de claves Healthy/Suspected/Quarantined/Revoked/Migrated, detección de anomalías |
| [M1](#m1) | FALSA | Entorno de pruebas estandarizado: AMD EPYC 7763, 64 cores, 128 GB RAM, NVMe SSD |
| [M2a](#m2a) | FALSA | Tamaño de la prueba ZK (Pi_i) ~62.4 KB |
| [M3a](#m3a) | SIN_FUENTE | Tiempo de proving 412 ms |
| [M3b](#m3b) | FALSA | Tiempo de verificación 8,1 ms |
| [M4a](#m4a) | FALSA | Sobrecarga de encadenamiento hash < 0,002 ms por transición |
| [M4b](#m4b) | SIN_FUENTE | Velocidad de auditoría histórica offline ~125.000 bloques/seg |
| [M5a](#m5a) | SIN_FUENTE | fsync: ext4 1,82 ms vs tmpfs 0,03 ms |
| [M7c](#m7c) | SIN_FUENTE | Comparación ECST vs IVC (implícita en los borradores) |
| [B1a](#b1a) | FALSA | RFC 8391: "Huelsing, A., Butin, D., Gazdag, S., Rijneveld, J., and A. Mohassel" |
| [B1b](#b1b) | FALSA | BibTeX de RFC 8391 con autor "Azam Mohassel" |
| [B3a](#b3a) | FALSA | RFC 9162 citado como "Laurie, B., Lin, C., Kasper, E., & Messeri, E." |
| [B3b](#b3b) | FALSA | BibTeX de RFC 9162: "Ben Laurie and Adam Langley and Eran Kasper and Emilia Messeri" |
| [B7a](#b7a) | FALSA | Nova: "Kothapalli, Setty & Tzialla (2022). Nova: Recursive Zero-Knowledge Proofs without Trusted Setup. CCS 2022" |
| [B7b](#b7b) | FALSA | DOI de Nova 10.1145/3548606.3560610 |
| [B8a](#b8a) | FALSA | IVC: "Valiant, P. (2008). Incrementally verifiable computation or proof of execution. FOCS 2008, pp. 137-146" |
| [B8b](#b8b) | FALSA | DOI de Valiant 10.1109/FOCS.2008.82 |
| [B9a](#b9a) | FALSA | PCD: "Chiesa & Tromer (2013). Proof-carrying data and incrementally verifiable computation. ITCS 2013, pp. 310-321" |
| [B9b](#b9b) | FALSA | DOI de Chiesa-Tromer 10.1145/2422436.2422472 |
| [B10a](#b10a) | FALSA | Pillai et al. OSDI 14: "All file systems are not created equal: On the fidelity of crash-consistency applications" |
| [B11b](#b11b) | FALSA | DOI 10.1145/2517349.2517373 |
| [B17b](#b17b) | FALSA | Etiqueta "MEASURED" para el coste de reutilización en HBS-STATE v0.3 |
| [P2](#p2) | FALSA | «Vía Grupo de Trabajo: el CFRG adopta el borrador, que pasa a llamarse draft-ietf-cfrg-hbs-state» |
| [P3](#p3) | FALSA | «El IESG aprueba formalmente el documento» (para un documento del CFRG) |
| [P7e](#p7e) | FALSA | Implícito: el apéndice de vectores (bloque JSON abierto con tres acentos graves y nunca cerrado) se procesa como código |
| [P7g](#p7g) | FALSA | Referencias escritas como líneas «[*RFC2119]» en vez de bloques YAML |
| [P11a](#p11a) | FALSA | «IVC … el coste de verificación O(log n) en el auditor permanece constante» |
| [P12a](#p12a) | FALSA | ECST-R: «un salto de instrucción inducido por hardware no sirve de nada porque el verificador externo computará Verify(Π_i)=0» |
| [P12c](#p12c) | FALSA | «Ningún ataque microarquitectónico puede falsificar … sin conocer las claves» |
| [P14](#p14) | FALSA | Uso de «compromisos de Pedersen» en un diseño post-cuántico |
| [P15a](#p15a) | FALSA | «KeyAtZero ⇒ NoSign» |
| [P16b](#p16b) | FALSA | Tabla: «Recursión ZK intra-circuito: IVC Sí (Requerida)» |
| [P18c](#p18c) | FALSA | «ECST/HBS-STATE establece un nuevo estándar» |

## Bloque H — hbs-state y la especificación HBS-STATE v0.3

### H10a

**FALSA** · gravedad alta

- **Borrador:** Vectores del I-D: {"version":"0.3","test_cases":[{"id":"TC-01",...,"expected_state":...}]}
- **Lo que se puede afirmar:** El fichero real es spec/state-vectors-v0.3.json (schema "hbs-state/0.3", CC0-1.0), con 12 vectores de reconciliación (A1-A12), 6 de lectura de índice (B1-B6), 1 de escritura fuera de campo (B7) y dos candidatos sin vectores (B8 y la familia C). Los casos TC-01..TC-04 son inventados.
- **Fuentes:** `hbs-state/spec/state-vectors-v0.3.json:2 "schema": "hbs-state/0.3" (no 'version')`; `hbs-state/spec/state-vectors-v0.3.json:110-265 colección 'reconciliation' A1..A12 con campos id, counter, key, state, derived{...}, fatal, provenance, note`; `hbs-state/spec/state-vectors-v0.3.json:266-321 'index_read' B1..B6; :322-337 'out_of_field_write' B7; :338-361 'candidates_without_vectors' B8 y C`; `grep de TC-0 y expected_state: 0 coincidencias`; `hbs-state/spec/state-vectors-v0.3.json:2 "schema": "hbs-state/0.3"`; `hbs-state/spec/state-vectors-v0.3.json:110-265 'reconciliation' A1..A12 (id, counter, key, state, derived, fatal, provenance, note); :266-321 B1..B6; :322-337 B7; :338-361 B8 y C; `; `grep 'TC-0\|expected_state' en hbs-state: 0 coincidencias`

### H11c

**FALSA** · gravedad alta

- **Borrador:** implicación de que ese layout es el de RFC 8391
- **Lo que se puede afirmar:** Ningún estándar define el formato serializado del SK: RFC 8391 (§4.1.7 y §4.2.2), RFC 8554 (§4.2 para LM-OTS y §5.2 para LMS) y SP 800-208 renuncian a definirlo; RFC 8391 solo enumera su contenido (en XMSS^MT, un índice de ceil(h/8) bytes) y pone el OID en la clave pública. OID(4)+idx+4×32 es el layout de la implementación de referencia en C y de su port xmss 0.1.0-pre.0. La spec retiró la atribución a RFC 8391 en su revisión del 2026-09-27, así que un sujeto con otro layout que falle la familia B no incumple RFC 8391 (spec §6; aún no implementado en el verificador).
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):46-48 §0: "No document standardises that layout ... the attribution is withdrawn"`; `hbs-state/spec/HBS-STATE-v0.3 (md):287-306 §6`; `RFC 8391 §4.1.7 (réplica): "Note that we do not define any specific format or handling for the XMSS private key SK by introducing this algorithm."`; `RFC 8391 §4.2.2: "This document does not define any specific format for the XMSS^MT private key SK_MT as it is not required for interoperability."`; `RFC 8391: 'PK = OID \|\| root \|\| SEED' (el OID está en la clave PÚBLICA)`; `RFC 8554 §5.2: "The format of the LMS private key is an internal matter to the implementation, and this document does not attempt to define it."`; `hbs-state/spec/HBS-STATE-v0.3 (md):46-48 §0 'the attribution is withdrawn'; :287-306`; `réplica de RFC 8391 §4.1.7: 'Note that we do not define any specific format or handling for the XMSS private key SK by introducing this algorithm.'`

### H12b

**FALSA** · gravedad alta

- **Borrador:** donde lleva operando en entornos reales de firma / README: "in production for a year"
- **Lo que se puede afirmar:** Según los propios registros de ARQUEO, el guardián nació entre el 2026-08-03 y el 2026-08-12 (§234), tuvo su primer consumidor en §236 y adquirió su forma de cuatro estados el 2026-08-20 (§331). ARQUEO se declara prototipo de investigación sin dinero real y sin operador real. Fueron unas siete semanas en un prototipo, no 'un año en producción'.
- **Fuentes:** `hbs-state/README.md:33-34 y :169-170; hbs-state/src/lib.rs:73-74 afirman 'a year'`; `AUDITORIA.md §234 (entre §159 con fecha 2026-08-03 y §275 con fecha 2026-08-12): nace el guardián, "Esta pieza no tiene consumidor todavía"`; `AUDITORIA.md §236: "el guardián de §234 deja de estar sin consumidor" (entra xmss)`; `AUDITORIA.md §331 (2026-08-20): nace ClaveEnCero; §335 (2026-08-20): arreglo de reinicio`; `SECURITY.md:27 "Prototipo de investigación, no un producto. No maneja dinero real"`; `hbs-state/src/lib.rs:71 encabezado "and this piece has no consumer yet" junto a la frase del año`; `historial de hbs-state (clon completo): primer commit 402c889 del 2026-09-26`; `hbs-state/README.md:33-34 y :169-170; hbs-state/src/lib.rs:73-74`

### H6b

**FALSA** · gravedad alta

- **Borrador:** ECST: KeyAtZero: el sistema exige intervención o aislamiento
- **Lo que se puede afirmar:** La especificación no exige intervención: KeyAtZero no es fatal y la política pertenece al dueño (README:73-74; src/lib.rs:358-362). Lo único que se deriva es no reutilizar nunca los índices indeterminados 0..counter-1; la spec no define 'fail closed' y esa lectura es una interpretación, no un texto normativo. En ARQUEO eso se hace automáticamente. La clave se resincroniza a counter abandonando 0..counter-1, y sólo se niega a arrancar si un segundo registro (el diario del nodo o las cofirmas del testigo) muestra que el contador retrocedió. En el registro de recepción, K=0 arranca avisando. El arranque sin intervención lo mide tools/banco_reutilizacion.sh desde el §579.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):144 KeyAtZero fatal = no`; `hbs-state/spec/state-vectors-v0.3.json:81 KeyAtZero "fatal": false`; `hbs-state/spec/HBS-STATE-v0.3 (md):383-384 §9 "Policy: what to do with the orphans belongs to the owner; here the only requirement is not going back"`; `hbs-state/spec/HBS-STATE-v0.3 (md):163 "Why KeyAtZero cannot be resolved, only failed closed"`; `hbs-state/src/lib.rs:143-147 set_index_in_sk: "Putting the key at counter uses a leaf that was NEVER reserved ... a lost index is better than an indeterminate one"`; `crates/zk-ssl-node/src/main.rs:576-611 (ARQUEO) ClaveEnCero => ArrancaResincronizando{hasta: contador} salvo que el diario muestre contador < máximo anotado`; `crates/zk-ssl-cli/src/witness.rs:2731-2756 testigo: igual, atado al fichero de cofirmas (<=)`; `AUDITORIA.md §335 (2026-08-20) "LA CLAVE VUELVE A SU INDICE"`

### H6c

**FALSA** · gravedad alta

- **Borrador:** I-D: Under a Persisted SK model, KeyAtZero MUST be classified as a protocol violation
- **Lo que se puede afirmar:** En HBS-STATE v0.3 la clasificación no depende del modelo, y KeyAtZero no es fatal en ninguno de los dos. Para un sujeto persistido, KeyAtZero puede ser una clave borrada por agotamiento (terminal, caso hbs-lms) y no una violación; distinguir ambos casos exige conocer el conjunto de parámetros, que (counter,key) no lleva.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):130-132 "The model does not change the table in section 2 -- the classification is the same"`; `hbs-state/spec/HBS-STATE-v0.3 (md):234-238 "family A is scored the same for both models, because the rule is a pure function of (counter, key)"`; `hbs-state/spec/HBS-STATE-v0.3 (md):115-122 un sujeto persistido alcanza KeyAtZero legítimamente al borrar la clave agotada (hbs-lms)`; `hbs-state/spec/state-vectors-v0.3.json:36-39 persisted.reachable_after_restart incluye KeyAtZero`; `hbs-state/spec/HBS-STATE-v0.3 (md):130-132 'The model does not change the table in section 2 -- the classification is the same'`; `hbs-state/spec/HBS-STATE-v0.3 (md):234-238 'family A is scored the same for both models'`; `hbs-state/spec/HBS-STATE-v0.3 (md):115-122`; `hbs-state/spec/state-vectors-v0.3.json:33-38`

### H6h

**FALSA** · gravedad alta

- **Borrador:** ECST 11.7: tras un corte K (memoria volátil) desaparece; la evaluación se reformula como R(DurableEvidence) -> {InSync, CounterAhead, Indeterminate}
- **Lo que se puede afirmar:** Tras un reinicio la evaluación sigue siendo R(counter,key) con cuatro estados. Con SK derivado de semilla, key se re-deriva en 0 y el resultado es KeyAtZero (campo indeterminate=counter), o InSync si counter=0; CounterAhead no es alcanzable tras el reinicio. Con SK persistido, key sobrevive y KeyAhead sigue siendo detectable. 'indeterminate' es el nombre de un campo derivado, no de un estado.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):103-108 con SK derivado de semilla K no desaparece: from_seed lo devuelve en 0 => KeyAtZero; con SK persistido K sobrevive`; `hbs-state/spec/HBS-STATE-v0.3 (md):136-146 cuatro estados, incluido KeyAhead; no existe estado 'Indeterminate' ni función 'DurableEvidence'`; `hbs-state/src/lib.rs:323-354 cuatro variantes`; `hbs-state/spec/HBS-STATE-v0.3 (md):103-108`; `hbs-state/spec/state-vectors-v0.3.json:42-47 seed_derived.reachable_after_restart = [KeyAtZero]`; `hbs-state/spec/HBS-STATE-v0.3 (md):141-146 cuatro estados; no existen 'Indeterminate' como estado ni 'DurableEvidence'`; `hbs-state/src/lib.rs:323-354`

### H7d

**FALSA** · gravedad alta

- **Borrador:** respuesta CFRG: the measurement takes a statistical distribution over N iterations
- **Lo que se puede afirmar:** La autocomprobación calcula una sola media por modo (20 escrituras) y una razón; no construye ninguna distribución estadística.
- **Fuentes:** `hbs-state/src/lib.rs:532-541 un Instant sobre 20 iteraciones, devuelve elapsed/20 (media aritmética)`; `hbs-state/src/lib.rs:543-547 una muestra de cada modo, sin varianza, percentiles ni repeticiones`; `hbs-state/src/lib.rs:532-541 un Instant sobre el bucle de 20, devuelve elapsed/20`; `hbs-state/src/lib.rs:543-548 una media por modo, sin varianza, percentiles ni repeticiones`

### H9b

**FALSA** · gravedad alta

- **Borrador:** I-D: N3 (Judgment): enforces policy judgments (identifying fatal states and operator-defined actions)
- **Lo que se puede afirmar:** N3 no evalúa políticas ni acciones definidas por el operador: solo que el juez marque como fatal exactamente KeyAhead (y el verificador lo comprueba con el booleano 'fatal').
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):202 N3 "Refuses to operate on KeyAhead, and only there"`; `hbs-state/spec/HBS-STATE-v0.3 (md):383-384 la política pertenece al dueño`; `hbs-state/src/lib.rs:358-362 "the invariant belongs to the guard; the policy, to each owner"`; `hbs-state/spec/HBS-STATE-v0.3 (md):202 N3 'Refuses to operate on KeyAhead, and only there'`; `hbs-state/spec/HBS-STATE-v0.3 (md):383-384`; `hbs-state/src/lib.rs:358 'the invariant belongs to the guard; the policy, to each owner'`; `hbs-state/spec/verify-state.py:234-235 N3 = derived_ok y fatal == vector`

### H10b

**SIN_FUENTE** · gravedad alta

- **Borrador:** draft-01 añade "algorithm":"LMS_SHA256_M32_H10"
- **Lo que se puede afirmar:** No existe ningún vector LMS ni campo 'algorithm' en HBS-STATE v0.3. El único conjunto de parámetros de referencia es XMSSMT-SHA2_40/8_256; la spec declara LMS/HSS como no probado (§9). LMS_SHA256_M32_H10 es un conjunto real de RFC 8554, pero no forma parte del banco.
- **Fuentes:** `grep 'LMS_SHA256' y '"algorithm"' en hbs-state: 0 coincidencias`; `hbs-state/spec/state-vectors-v0.3.json:10-24 único reference_parameter_set XMSSMT-SHA2_40/8_256`; `hbs-state/spec/HBS-STATE-v0.3 (md):386 "LMS/HSS: the table ought to hold; it has not been tested"`; `grep 'LMS_SHA256\|"algorithm"' en hbs-state: 0 coincidencias`; `hbs-state/spec/HBS-STATE-v0.3 (md):386 'LMS/HSS: the table ought to hold; it has not been tested'`; `réplica de RFC 8554 (clon github.com/tex2e/rfc-translater, html/rfc8554.html): 'LMS_SHA256_M32_H10 \| Section 5 \| 0x00000006'`

### H17a

**SIN_FUENTE** · gravedad alta

- **Borrador:** I-D: IndexGuard MUST acquire an exclusive non-blocking lock (flock/fcntl)
- **Lo que se puede afirmar:** hbs-state no toma ningún cerrojo: dos procesos (o dos IndexGuard) pueden abrir el mismo contador. La spec excluye expresamente los cerrojos y los firmantes múltiples.
- **Fuentes:** `grep flock/fcntl/lock en hbs-state/src: 0 coincidencias`; `hbs-state/spec/HBS-STATE-v0.3 (md):382 §9 "How the counter is persisted: fsync, ordering, locks, copies" NO cubierto; :385 "Multiple signers" no cubierto`; `grep -i 'flock\|fcntl\|lock' en hbs-state/src: 0 coincidencias`; `hbs-state/spec/HBS-STATE-v0.3 (md):382 ''fsync', ordering, locks, copies' fuera de alcance; :385 'Multiple signers'`

### H17b

**SIN_FUENTE** · gravedad alta

- **Borrador:** I-D: multi-thread: atomic fetch-and-add or mutex
- **Lo que se puede afirmar:** No hay primitivas de concurrencia: la exclusión dentro de un proceso depende solo de &mut self sobre una única instancia. La concurrencia queda fuera del alcance de la spec (§9).
- **Fuentes:** `grep Mutex/atomic en hbs-state/src: 0 coincidencias`; `hbs-state/src/lib.rs:482 reserve(&mut self): el préstamo exclusivo de Rust serializa el uso de UNA instancia, nada más`; `grep -i 'mutex\|atomic' en hbs-state/src: 0 coincidencias`; `hbs-state/src/lib.rs:482 reserve(&mut self)`

### H17c

**SIN_FUENTE** · gravedad media

- **Borrador:** I-D: hardware anchoring TPM NVRAM
- **Lo que se puede afirmar:** hbs-state es solo software (contador en fichero + fsync). El anclaje en hardware lo recomienda RFC 10033 (texto fuente) y SP 800-208 lo exige para la validación; TPM NVRAM es una adición del I-D sin implementación.
- **Fuentes:** `grep TPM/NVRAM en hbs-state: 0 coincidencias`; `draft-ietf-pquip-hbs-state (md):525-528 recomienda hardware criptográfico dedicado (sin nombrar TPM NVRAM)`; `grep -i 'tpm\|nvram' en hbs-state: 0 coincidencias`; `draft-ietf-pquip-hbs-state (md):525-528 recomienda hardware criptográfico dedicado, sin nombrar TPM ni NVRAM`

### H6f

**SIN_FUENTE** · gravedad alta

- **Borrador:** ECST: Colapso controlado (C<K o corrupción): destruye las claves operativas en RAM y marca el árbol HBS como agotado
- **Lo que se puede afirmar:** Ni hbs-state ni ARQUEO destruyen claves en RAM ni marcan el árbol como agotado. Ante KeyAhead, la política de ARQUEO es no arrancar y declarar la clave comprometida; ante un registro de recepción por delante de su contador, no arrancar. Ante un fichero de contador de tamaño distinto de 8 bytes, open() devuelve GuardError::Corrupt.
- **Fuentes:** `grep sin resultados de exhaust/wipe/destroy/zeroize/agotad en hbs-state/src y spec`; `hbs-state/src/lib.rs:370-377 is_fatal solo devuelve true`; `hbs-state/src/lib.rs:460-462 corrupción => Err(GuardError::Corrupt{bytes}) al abrir; nada más`; `crates/zk-ssl-node/src/main.rs:612-619 ARQUEO: ClaveAdelantada => NoArranca("LA CLAVE DEBE CONSIDERARSE COMPROMETIDA")`; `crates/zk-ssl-node/src/firma_cabeza.rs:219-237 zeroize solo del búfer temporal al resincronizar, no un colapso`; `grep -i 'exhaust\|wipe\|destroy\|zeroize\|agotad\|colapso' en hbs-state/src y spec: 0 coincidencias`; `hbs-state/src/lib.rs:370-377 is_fatal solo devuelve bool; :460-462 Corrupt{bytes}`; `crates/zk-ssl-node/src/main.rs:612-619 NoArranca 'LA CLAVE DEBE CONSIDERARSE COMPROMETIDA'`

### H7c2

**SIN_FUENTE** · gravedad alta

- **Borrador:** otro borrador: ext4 1.82 ms, tmpfs 0.03 ms
- **Lo que se puede afirmar:** Las únicas cifras registradas son 0,907 ms (ext4) y 0,002 ms (tmpfs), de una sola máquina; 1,82 ms / 0,03 ms no tienen fuente.
- **Fuentes:** `grep de '1.82 ms', '1,82', '0.03 ms' en ambos repos: 0 coincidencias`; `hbs-state/README.md:82-85 dice 0.907 ms y 0.002 ms`; `grep '1[.,]82 ms\|0[.,]03 ms' en hbs-state y ARQUEO (md, rs, sh, py): 0 coincidencias`; `hbs-state/README.md:82-85 0.907 ms y 0.002 ms`; `git grep -nE '1[.,]82 ?(ms\|µs)\|0[.,]03 ?(ms\|µs)' HEAD (Arqueo) => 0`; `hbs-state/README.md:82-85`

### H7e

**SIN_FUENTE** · gravedad alta

- **Borrador:** I-D: inspect /proc/mounts; MUST NOT permit tmpfs, ramfs, procfs paths
- **Lo que se puede afirmar:** hbs-state no inspecciona /proc/mounts ni mantiene una lista de sistemas de ficheros prohibidos: detecta la persistencia falsa solo por tiempos. La especificación no regula la persistencia (§9). Solo tmpfs está medido; ramfs y procfs no se han probado.
- **Fuentes:** `grep de mounts/ramfs/procfs en hbs-state/src, spec y README: 0 coincidencias`; `hbs-state/src/lib.rs:762-769 solo el TEST usa 'df -T --output=fstype' para localizar un tmpfs donde probar`; `hbs-state/src/lib.rs:56 "It is the only signal available from inside the process" (la detección es por tiempo)`; `hbs-state/spec/HBS-STATE-v0.3 (md):382 §9 "How the counter is persisted: fsync, ordering, locks, copies" fuera de alcance`; `grep -i 'mounts\|ramfs\|procfs\|statfs' en hbs-state/src, spec, README, tests: 0 coincidencias`; `hbs-state/src/lib.rs:762-769 solo el test usa 'df -T --output=fstype'`; `hbs-state/src/lib.rs:54-56 'It is the only signal available from inside the process'`; `hbs-state/spec/HBS-STATE-v0.3 (md):382 §9 persistencia fuera de alcance`

### H7f

**SIN_FUENTE** · gravedad alta

- **Borrador:** explicit operator override for ultra-fast NVMe
- **Lo que se puede afirmar:** No existe override. El caso NVMe se resuelve en el diseño: solo se rechaza si la razón es < 10× Y fsync < 20 µs, así que cualquier fsync ≥ 20 µs pasa. Los ~100 µs de un NVMe son una suposición declarada, no una medida. Umbrales (20 muestras, 10×, 20 µs) declarados y no derivados, de una sola máquina.
- **Fuentes:** `grep 'override' en hbs-state: 0 coincidencias`; `hbs-state/src/lib.rs:105-108 FLOOR_MICROS = 20.0: "A fast NVMe does fsync in ~100 us legitimately, so this stays well below it"`; `hbs-state/src/lib.rs:99-103 MIN_RATIO = 10.0 "DECLARED threshold, not derived"`; `grep -i override en hbs-state: 0 coincidencias`; `hbs-state/src/lib.rs:99-103 MIN_RATIO = 10.0 'DECLARED threshold, not derived'`; `hbs-state/src/lib.rs:105-108 FLOOR_MICROS = 20.0 'A fast NVMe does fsync in ~100 us legitimately'`

### H13c

**PARCIAL** · gravedad baja

- **Borrador:** licencias: CC0 / Dual MIT/Apache-2.0
- **Lo que se puede afirmar:** El código está bajo MIT OR Apache-2.0, a elección del receptor; solo los vectores de spec/ son CC0-1.0. El campo 'mit' de Zenodo es una etiqueta: los términos válidos son los de LICENSE-MIT, LICENSE-APACHE y spec/LICENSE-CC0.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):481-482 "The vectors under spec/, CC0-1.0. Everything else, MIT OR Apache-2.0."`; `hbs-state/Cargo.toml:47 license = "MIT OR Apache-2.0"`; `hbs-state/CITATION.cff:4 y :12-14`; `hbs-state/.zenodo.json:4 "license": "mit" (etiqueta; :17 dice que los ficheros LICENSE son los que valen)`; `hbs-state/spec/HBS-STATE-v0.3 (md):481-482 'The vectors under spec/, CC0-1.0. Everything else, MIT OR Apache-2.0.'`; `hbs-state/Cargo.toml:5 license = "MIT OR Apache-2.0"`; `hbs-state/CITATION.cff:4 (abstract) y :12-14`; `hbs-state/.zenodo.json:5 "license": "mit"; :19 'the single field above is a label, not the terms'`

### H14a

**PARCIAL** · gravedad media

- **Borrador:** RFC 10033 (informational, September 2026)
- **Lo que se puede afirmar:** RFC 10033 'Hash-Based Signatures: State and Backup Management' existe (título confirmado por búsqueda) y procede de draft-ietf-pquip-hbs-state, de categoría informativa. El mes de publicación (septiembre de 2026) no se ha podido verificar.
- **Fuentes:** `WebSearch: rfc-editor.org/info/rfc10033 titulado "RFC 10033: Hash-Based Signatures: State and Backup Management" (Wiggers, Bashiri, Kölbl, Goodman, Kousidis)`; `github.com/hbs-guidance/draft-hbs-state (clonado, HEAD 4ee80a2 2026-02-27): draft-ietf-pquip-hbs-state (md) 'category: info'`; `rfc-editor.org y datatracker bloqueados: fecha no verificada`; `WebSearch: 'RFC 10033: Hash-Based Signatures: State and Backup Management \| RFC Editor' (rfc-editor.org/info/rfc10033)`; `clon github.com/hbs-guidance/draft-hbs-state @4ee80a2 (2026-02-27) draft-ietf-pquip-hbs-state (md):2-4 title y 'category: info'; autores Wiggers (PQShield), Bashiri (BSI), Kölbl (G`

### H14b

**PARCIAL** · gravedad media

- **Borrador:** su sección 4 exige las cuatro propiedades ACID sobre el estado
- **Lo que se puede afirmar:** En el texto fuente del WG, la sección 4 'Requirements for Secure State Management' dice que los sistemas 'should satisfy all ACID properties': es una recomendación de un documento informativo, no una exigencia. La propiedad de durabilidad incluye cortes de corriente y la de aislamiento, procesos separados, dos cosas que hbs-state ni mide ni cubre. La numeración del RFC final no se ha verificado.
- **Fuentes:** `draft-ietf-pquip-hbs-state (md):474-497 (fuente del WG) '# Requirements for Secure State Management': "State management systems should satisfy all _ACID_ properties" (Atomicity, Co`; `draft-ietf-pquip-hbs-state (md):474 '# Requirements for Secure State Management' (cuarta sección de nivel 1); :483 'State management systems should satisfy all _ACID_ properties'`; `draft-ietf-pquip-hbs-state (md):492-497 Isolation 'separate processes or devices'; Durability 'must survive crashes, power loss, or device failure'`; `hbs-state/README.md:96 'Nothing against a power cut.'`; `hbs-state/spec/HBS-STATE-v0.3 (md):382 cerrojos fuera de alcance`

### H14c

**PARCIAL** · gravedad baja

- **Borrador:** cita literal: "in particular, this enables implementing rollback resistant counters, which can be difficult to achieve in a software-only fashion"
- **Lo que se puede afirmar:** El texto fuente del WG dice "In particular, this enables implementing rollback resistant counters which can be difficult to achieve in a software-only fashion" (sin coma), justo después de recomendar hardware criptográfico dedicado. La forma exacta en el RFC 10033 publicado no se ha podido comprobar.
- **Fuentes:** `draft-ietf-pquip-hbs-state (md):525-528: "Using dedicated cryptographic hardware is recommended to enforce these requirements, ensure correct behavior, and handle the complexity of`; `WebSearch de la frase con coma: el único resultado es el README de hbs-state`; `draft-ietf-pquip-hbs-state (md):525-528 'Using dedicated cryptographic hardware is recommended ... In particular, this enables implementing rollback resistant counters which can be`; `hbs-state/README.md:23-25 y spec:16-18 citan con coma`

### H15

**PARCIAL** · gravedad alta · **el escéptico revocó el primer veredicto** (FALSA)

- **Borrador:** reusing an index lets an attacker forge with as low as 2^34 hash evaluations
- **Lo que se puede afirmar:** Según el artículo de QRL 'Statefulness and security' (consultado el 2026-08-12, no medido en el proyecto), extraer por fuerza bruta la clave privada OTS de un índice cuesta ~2^34 hashes con dos firmas de ese índice, es decir, un reúso; ~2^23 con tres y ~2^18 con cuatro. Así que 2^34 es el punto MÁS caro de la curva, no un mínimo: 'as low as' invierte el sentido. La etiqueta 'MEASURED' de HBS-STATE es incorrecta: es una cifra consultada. Su 'second reuse' debe leerse con la convención de origen de Arqueo, donde 'segunda repetición' son dos firmas. Las fuentes académicas que cita el RFC son Bruinderink–Hülsing 2016 (eprint 2016/1042) y Fluhrer 2023.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):174-175 "(MEASURED, QRL curve): at the **second reuse** of an index, forging a signature costs on the order of **2^34 hashes**"`; `BACKLOG.md:628-631 "con 2 firmas, ~2^34 hashes; con 3, ~2^23; con 4, ~2^18" (coste de extraer la clave privada OTS)`; `AUDITORIA.md línea 21490 "(QRL, consultado 2026-08-12): 2 firmas ~2^34 hashes, 3 ~2^23, 4 ~2^18"; líneas 21428-21436 "ESTATUTO ... TODO es RAZONADO, no medido"`; `crates/zk-ssl-guardian/Cargo.toml (ARQUEO): "curva QRL: a la cuarta repeticion, ~2^18 hashes"`; `WebSearch: artículo QRL 'Statefulness and security' (P. Waterland) — derivar la clave OTS con 2 firmas del mismo índice ~2^34, 3 ~2^23, 4 ~2^18 (medium.com y theqrl.org bloqueados `; `draft-ietf-pquip-hbs-state (md):212-215 cita BH16 (eprint 2016/1042) y Fluhrer23 como fuentes de falsificación tras reúso`; `hbs-state/spec/HBS-STATE-v0.3 (md):174-175 '(MEASURED, QRL curve): at the **second reuse** of an index, forging a signature costs on the order of **2^34 hashes**'`; `hbs-state/spec/state-vectors-v0.3.json:99`

### H16

**PARCIAL** · gravedad alta

- **Borrador:** HBS-STATE applies equally to LMS/HSS; the tuple (counter,key) and the four states are completely algorithm-agnostic
- **Lo que se puede afirmar:** La spec declara LMS/HSS dentro de su alcance (§ inicial), pero en §9 los marca como no probados. La regla de la familia A es aritmética sobre dos enteros y se puede aplicar a cualquier esquema con un índice monótono único (hbs-lms guarda un solo contador u64). Aun así, el único contacto con LMS (hbs-lms, medido fuera del protocolo de sujeto) reveló una vía nueva a KeyAtZero que (counter,key) no puede desambiguar sin el conjunto de parámetros. La familia B no aplica a LMS.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):386 "LMS/HSS: the table ought to hold; it has not been tested"`; `hbs-state/spec/HBS-STATE-v0.3 (md):115-122 lo único medido con LMS (hbs-lms) falsó una afirmación y deja KeyAtZero ambiguo sin el conjunto de parámetros`; `hbs-state/spec/HBS-STATE-v0.3 (md):436-438 hbs-lms "was not run through the subject protocol ... no level is claimed"`; `hbs-state/spec/HBS-STATE-v0.3 (md):263-306 la familia B es específica del layout de xmss 0.1.0-pre.0`; `hbs-lms @7063cc8 src/hss/reference_impl_private_key.rs:286-288 CompressedUsedLeafsIndexes{count: u64} (un solo contador comprimido para HSS)`; `hbs-state/spec/HBS-STATE-v0.3 (md):3-6 '(XMSS, XMSS^MT, LMS, HSS)'`; `hbs-state/spec/HBS-STATE-v0.3 (md):386 'LMS/HSS: the table ought to hold; it has not been tested'`; `hbs-state/spec/HBS-STATE-v0.3 (md):115-122, 436-438`

### H17d

**PARCIAL** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** I-D: rollback of counter alone triggers KeyAhead on persisted-key systems
- **Lo que se puede afirmar:** Es lógicamente correcto en un caso: si el contador retrocede a C' y el K persistido (no restaurado) está en K > C', R da KeyAhead (unrecorded = K − C'). ARQUEO lo ejerce desde el §569 en su registro de recepción, un recurso de K persistido: contador restaurado con el registro vivo → no arranca. No lo detecta si el retroceso deja C' ≥ K, que con SK persistido tampoco es peligroso, ni si se restauran a la vez clave y contador. En el modelo derivado de semilla, R no ve el retroceso: da KeyAtZero con un counter menor, o InSync(0) si se borra el fichero. La spec no lo afirma, y su lista de 'sólo dos causas' de KeyAhead omite el retroceso. ARQUEO cubre esos huecos fuera de hbs-state con un dato más allá del par: el diario o las cofirmas para el índice de firma, y el recepCount de la cabeza anterior para el contador de recepción.
- **Fuentes:** `hbs-state/src/lib.rs:425-429 Less => KeyAhead{unrecorded: key - counter}`; `hbs-state/spec/HBS-STATE-v0.3 (md):160-161 KeyAhead: "There are only two causes: the order was inverted, or fsync lied" (no menciona el retroceso)`; `hbs-state/spec/HBS-STATE-v0.3 (md):170-172 "a restored state is indistinguishable from a legitimate one"`; `hbs-state/src/lib.rs:455-466 si el fichero del contador falta, open() arranca en 0 sin error`; `crates/zk-ssl-node/src/main.rs:590-596 y crates/zk-ssl-cli/src/witness.rs:2731-2742 (ARQUEO) detección de retroceso con diario/cofirmas, fuera de hbs-state`; `hbs-state/spec/HBS-STATE-v0.3 (md):160-161 'There are only two causes: the order was inverted, or 'fsync' lied'`; `hbs-state/spec/HBS-STATE-v0.3 (md):170-172 'a restored state is indistinguishable from a legitimate one'`; `hbs-state/src/lib.rs:455-466 fichero ausente => 0 sin error`

### H1b

**PARCIAL** · gravedad media

- **Borrador:** guardián del índice para XMSS y XMSS^MT
- **Lo que se puede afirmar:** El contador y la reconciliación son aritmética genérica sobre dos u64; pero index_from_sk/set_index_in_sk están fijados a un único conjunto, XMSSMT-SHA2_40/8_256 con el layout OID(4)\|\|idx(5 B, big-endian)\|\|4×32 = 137 B (el de xmss 0.1.0-pre.0 y de la implementación de referencia en C que porta). Un SK de XMSS de árbol único (136 B, índice de 4 B) se rechaza con UnexpectedLayout (src/lib.rs:128-132, 734-745).
- **Fuentes:** `hbs-state/README.md:3 "The index guard for stateful hash-based signatures (XMSS, XMSS^MT)"`; `hbs-state/src/lib.rs:92-94 index_width() fijo a 5`; `hbs-state/src/lib.rs:128-132 index_from_sk exige sk.len()==137 o devuelve UnexpectedLayout`; `hbs-state/src/lib.rs:738-744 test: un SK de 136 B (árbol único) se rechaza`; `hbs-state/README.md:148-152 "index_width() is fixed at 5"`; `hbs-state/README.md:3 "(XMSS, XMSS^MT)"; :144-152 index_width fijo a 5`; `hbs-state/src/lib.rs:92-94 index_width() -> 5; :128-132 exige sk.len()==137 o UnexpectedLayout`; `hbs-state/src/lib.rs:734-745 test an_sk_of_another_size_is_rejected_not_read (136 B rechazado)`

### H20

**PARCIAL** · gravedad media

- **Borrador:** el self-check garantiza persistencia frente a la muerte abrupta del proceso, pero no frente a corte físico de alimentación
- **Lo que se puede afirmar:** El README no habla de 'garantía' frente a la muerte del proceso: dice que se MIDIÓ la durabilidad frente a la muerte del proceso (25/25) y que no garantiza nada frente a un corte de corriente, que no se ha medido. Esa propiedad viene del orden persistir-antes-de-firmar, no de la autocomprobación, que solo verifica que fsync cueste algo.
- **Fuentes:** `hbs-state/README.md:94-100 "What it does NOT guarantee: **Nothing against a power cut.** ... What was measured is durability against **process death** -- 25 of 25 with not one sign`; `hbs-state/src/lib.rs:63-69 (añade "and that has not been done")`; `hbs-state/src/lib.rs:521-556 el self-check solo compara costes de fsync`; `hbs-state/README.md:94-100 '**Nothing against a power cut.** ... What was measured is durability against **process death** -- 25 of 25 ...'`; `hbs-state/src/lib.rs:63-69 '... and that has not been done'`; `hbs-state/src/lib.rs:521-556`

### H4

**PARCIAL** · gravedad media

- **Borrador:** KeyAtZero ocurre en claves derivadas de semilla que vuelven a 0 al reiniciar
- **Lo que se puede afirmar:** KeyAtZero es el estado counter>0 y key==0. En el modelo de SK derivado de semilla es siempre el estado tras un reinicio con counter>0 (from_seed devuelve índice 0). Desde la revisión del 2026-09-27, la spec reconoce que un sujeto con SK persistido también lo alcanza cuando borra la clave al agotarse (hbs-lms 0.2.0-alpha.1, commit 7063cc8). KeyAtZero no distingue una clave borrada (terminal) de un reinicio: el discriminador es el conjunto de parámetros, que el campo índice no lleva (spec §0 y §1).
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):103-108 tabla: SK SEED-DERIVED "after a restart: always KeyAtZero"; SK PERSISTED "... and KeyAtZero when the subject wipes the key on exhaustion"`; `hbs-state/spec/HBS-STATE-v0.3 (md):115-123 "a **persisted** subject also reaches KeyAtZero. hbs-lms 0.2.0-alpha.1 ... wipes the private key when the tree is exhausted ... A wiped k`; `hbs-state/spec/state-vectors-v0.3.json:32-40 persisted.reachable_after_restart incluye KeyAtZero; note_v0_3_1`; `hbs-lms @7063cc8 src/hss/reference_impl_private_key.rs:93-97 wipe() pone seed, parámetro y contador a cero; :184-186 increment(...).unwrap_or_else(\|_\| self.wipe()) (verificado cl`; `hbs-state/spec/HBS-STATE-v0.3 (md):107 fila 'after a restart' corregida; :115-123`; `clon github.com/Fraunhofer-AISEC/hbs-lms-rust @7063cc8 src/hss/reference_impl_private_key.rs:93-97 fn wipe; :184-186 '.increment(&tree_heights).unwrap_or_else(\|_\| self.wipe())'`; `clon completo de hbs-state: git diff v0.1.0 v0.2.0 muestra que la fila decía antes solo 'InSync, CounterAhead or KeyAhead'`

### H5

**PARCIAL** · gravedad alta

- **Borrador:** CounterAhead: normal tras un crash; se reservó pero el proceso murió antes de firmar
- **Lo que se puede afirmar:** CounterAhead significa que se reservaron índices que nunca se firmaron (huérfanos), por ejemplo porque el proceso murió entre reserve() y la firma. Sólo es 'el caso normal tras un crash' en el modelo de SK persistido: la spec lo afirma, pero no lo ha medido para ese modelo. En Arqueo se ve en el registro de recepción, donde K se persiste. En el modelo derivado de semilla, el de hbs-state y el del índice de firma de ARQUEO, CounterAhead sólo ocurre DENTRO de un proceso vivo, y tras un reinicio el estado es siempre KeyAtZero (spec §1). La cifra 13/25 se midió dentro del proceso. El comentario de módulo de hbs-state/src/lib.rs:33-36 y los de ARQUEO main.rs:586-588 y 605 conservan la inferencia retractada.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):108 fila CounterAhead: SK PERSISTED "the normal case after a crash" / SK SEED-DERIVED "INTRA-PROCESS only"`; `hbs-state/spec/HBS-STATE-v0.3 (md):69-86 §0: "The second sentence is FALSE ... 13 of 25 ... measures ... deaths inside a live process, not restarts"`; `hbs-state/spec/state-vectors-v0.3.json:71 "intra_process_only_if": "sk_model == seed_derived"`; `hbs-state/src/bin/subject.rs:63-68 el sujeto declara sk_model seed_derived`; `hbs-state/src/lib.rs:33-36 y :611-612 todavía dicen "13 of 25 ... It is not the exception: it is the normal path after a crash" sin la salvedad`; `AUDITORIA.md §331 (línea 25552) "K.1: la cifra es correcta y la inferencia no ... Tras un REINICIO la clave nace en cero, asi que ContadorAdelantado no ocurre el 52 % de las veces `; `hbs-state/spec/HBS-STATE-v0.3 (md):108 CounterAhead: PERSISTED 'the normal case after a crash' / SEED-DERIVED 'INTRA-PROCESS only'`; `hbs-state/spec/HBS-STATE-v0.3 (md):69-86 §0: 'The second sentence is FALSE ... measures ... deaths inside a live process, not restarts'`

### H6a

**PARCIAL** · gravedad baja

- **Borrador:** Solo KeyAhead es fatal; el proceso debe detenerse
- **Lo que se puede afirmar:** El juez de HBS-STATE marca fatal exactamente KeyAhead, y sólo KeyAhead (N3: rehusar operar en KeyAhead y sólo ahí). La librería no detiene nada: is_fatal es un predicado puro que devuelve un booleano, y detenerse es deber del llamante. En ARQUEO, el nodo y el testigo devuelven NoArranca (crates/zk-ssl-node/src/main.rs:644-650, crates/zk-ssl-cli/src/witness.rs:2758-2771), y desde el §569 también el registro de recepción ante su ClaveAdelantada (main.rs:682-690).
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):179-185 'fatal(state) = (state is KeyAhead)'`; `hbs-state/spec/HBS-STATE-v0.3 (md):202 "N3 -- judges. Refuses to operate on KeyAhead, and only there."`; `hbs-state/src/lib.rs:356-377 is_fatal; :364 "production does not call it, and that is on purpose"`; `hbs-state/README.md:73-74 "only KeyAhead admits no nuance. The other three are each owner's policy."`; `hbs-state/spec/HBS-STATE-v0.3 (md):202 'N3 -- judges. Refuses to operate on 'KeyAhead', and only there.'`; `hbs-state/README.md:64 'Reconciliation::KeyAhead { .. } => {} // DO NOT START'; :73-74`; `hbs-state/src/lib.rs:364 'production does not call it, and that is on purpose'`; `crates/zk-ssl-node/src/main.rs:612-619 y crates/zk-ssl-cli/src/witness.rs:2757-2770 (ARQUEO) NoArranca`

### H6d

**PARCIAL** · gravedad alta

- **Borrador:** I-D: Under a Seed-Derived SK model, the runtime MUST advance key directly to counter before resuming
- **Lo que se puede afirmar:** Avanzar la clave hasta counter es la POLÍTICA del dueño ARQUEO para su índice de firma (nodo §335, testigo §337, medido en §579), condicionada a que un segundo registro no muestre retroceso del contador. No es un requisito normativo de HBS-STATE v0.3, que deja la política al dueño (§9); hbs-state sólo aporta el mecanismo set_index_in_sk. En otro recurso de ARQUEO, el registro de recepción, KeyAtZero no se resuelve avanzando nada.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):380-384 §9 la política no está cubierta (no hay MUST)`; `hbs-state/src/lib.rs:140-168 set_index_in_sk existe como mecanismo y su doc lo llama CONSERVADOR`; `crates/zk-ssl-node/src/main.rs:590-609 política del nodo ARQUEO: NoArranca si 'contador < d' (diario); si no, ArrancaResincronizando{hasta: contador}`; `crates/zk-ssl-cli/src/witness.rs:2731-2756 política del testigo: NoArranca si 'contador <= t' (cofirmas)`; `AUDITORIA.md §335`; `hbs-state/spec/HBS-STATE-v0.3 (md):380-388 §9 (sin requisitos de política salvo 'not going back')`; `hbs-state/src/lib.rs:140-168 set_index_in_sk, documentado como CONSERVATIVE`; `crates/zk-ssl-node/src/main.rs:590-609 NoArranca si '*contador < d' (diario); si no, ArrancaResincronizando{hasta: *contador}`

### H6e

**PARCIAL** · gravedad media

- **Borrador:** I-D: CounterAhead: an implementation MAY advance key to match counter
- **Lo que se puede afirmar:** La especificación sólo dice que los huérfanos se queman o se registran y que el contador nunca retrocede; qué hacer con ellos es del dueño. Adelantar la clave es compatible con eso, porque es una forma de quemarlos, pero no está escrito. ARQUEO, ante CounterAhead, arranca avisando sin mover la clave, y en el registro de recepción declara los huecos.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):158-159 "The orphans are burnt or recorded; the counter never goes back"`; `hbs-state/spec/HBS-STATE-v0.3 (md):383-384 §9 "what to do with the orphans belongs to the owner"`; `crates/zk-ssl-node/src/main.rs:570-575 ARQUEO: ContadorAdelantado => ArrancaAvisando (no resincroniza la clave)`; `hbs-state/spec/HBS-STATE-v0.3 (md):158-159 'The orphans are burnt or recorded; the counter never goes back'`; `hbs-state/spec/HBS-STATE-v0.3 (md):383-384 §9`; `crates/zk-ssl-node/src/main.rs:570-575 ContadorAdelantado => ArrancaAvisando`; `crates/zk-ssl-cli/src/witness.rs:2703-2712 igual en el testigo`; `hbs-state/spec/HBS-STATE-v0.3 (md):158-159, 383-384`

### H6g

**PARCIAL** · gravedad media

- **Borrador:** Recuperación fast-forward (C>K): actualiza K<-C
- **Lo que se puede afirmar:** En ARQUEO, el fast-forward K<-C sólo se aplica al índice de firma en KeyAtZero (C>K con K==0), y sólo si el diario o las cofirmas no contradicen el contador. En CounterAhead (K!=0) no se mueve la clave, y en el registro de recepción no se mueve nunca. No es regla de la especificación.
- **Fuentes:** `crates/zk-ssl-node/src/main.rs:576-611 (solo KeyAtZero, K==0, y condicionado al diario)`; `crates/zk-ssl-node/src/main.rs:570-575 CounterAhead (K!=0): solo aviso`; `hbs-state/src/lib.rs:154-168 set_index_in_sk falla cerrado si index >= 2^40 (IndexOutOfField)`; `crates/zk-ssl-node/src/main.rs:576-611 KeyAtZero: resincroniza salvo retroceso según el diario`; `hbs-state/src/lib.rs:154-168 set_index_in_sk falla cerrado si index >= 2^(8·5)`; `crates/zk-ssl-node/src/main.rs:602-607 (CounterAhead: solo aviso) y 608-643 (KeyAtZero: resincroniza salvo retroceso)`; `crates/zk-ssl-node/src/main.rs:659-681 (registro: nunca resincroniza)`; `hbs-state/src/lib.rs:154-168`

### H7b

**PARCIAL** · gravedad media

- **Borrador:** método: diferencia de latencia entre escrituras con y sin sync_all()
- **Lo que se puede afirmar:** No es una diferencia sino una RAZÓN de medias (con/sin sync_all) más un suelo absoluto: 20 escrituras de 8 bytes por modo (primero sin sync y luego con sync_all), cronometradas con un único Instant sobre el bucle y divididas entre 20.
- **Fuentes:** `hbs-state/src/lib.rs:96-97 SELFCHECK_SAMPLES = 20`; `hbs-state/src/lib.rs:525-547: 20 escrituras de 8 bytes en offset 0 sin sync, luego 20 con sync_all; media en µs = tiempo total / 20; ratio = with / without`; `hbs-state/src/lib.rs:97 SELFCHECK_SAMPLES = 20`; `hbs-state/src/lib.rs:525-547 abre la sonda con truncate, un Instant sobre 20 iteraciones seek+write_all(8 B)[+sync_all], elapsed/20; without primero, with después; ratio = with/wit`

### H9a

**PARCIAL** · gravedad media

- **Borrador:** ECST: N0 declaración del modelo; N1 clasificación; N2 cálculo de campos derivados (índices huérfanos); N3 rechazo estricto y seguro de estados fatales (KeyAhead)
- **Lo que se puede afirmar:** N0 declara el modelo (persisted \| seed_derived; obligatorio desde v0.3: un sujeto que no lo declara queda sin nivel); N1 distingue los cuatro estados; N2 acierta todos los campos derivados (index, orphans, indeterminate, unrecorded); N3 marca como fatal exactamente KeyAhead y ningún otro estado. Los niveles son acumulativos. El verificador solo comprueba el booleano 'fatal'; no ejercita un rechazo real.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):196-202 N0 declares; N1 classifies; N2 "Gets indeterminate, orphans, unrecorded right"; N3 "Refuses to operate on KeyAhead, and only there"`; `hbs-state/spec/verify-state.py:231-238 N2 exige todos los campos de v['derived'] (incluido index); N3 exige que 'fatal' coincida con el vector`; `hbs-state/spec/verify-state.py:323-326 sin N0 el nivel queda en NONE`; `hbs-state/spec/HBS-STATE-v0.3 (md):196-202`; `hbs-state/spec/verify-state.py:231-238 N2 exige todos los campos de v['derived'] (incluido index); N3 exige 'fatal' igual al vector`; `hbs-state/spec/verify-state.py:323-326 n0_ok False => NONE; :168 n0_ok None = BLIND`

### H13a

**NO_VERIFICABLE** · gravedad alta

- **Borrador:** Zenodo DOI 10.5281/zenodo.22993572
- **Lo que se puede afirmar:** El DOI que declara el repositorio es 10.5281/zenodo.22980547 (DOI de concepto, según el commit 476eaaf). 22993572 no aparece en ninguna revisión y no se ha podido resolver; podría ser un DOI de versión, pero no debe citarse sin verificarlo.
- **Fuentes:** `hbs-state/CITATION.cff:16 doi: 10.5281/zenodo.22980547`; `git log del clon completo de hbs-state: commit 476eaaf "Add CITATION.cff with the concept DOI"; 'git grep' en todas las revisiones: solo aparece zenodo.22980547, nunca 22993572`; `WebSearch "zenodo.22993572" y "10.5281/zenodo.22980547": sin resultados; zenodo.org y doi.org bloqueados`; `clon completo de hbs-state: 'git grep zenodo $(git rev-list --all)' solo encuentra 22980547 (476eaaf CITATION.cff:14 y a960828 CITATION.cff:16); 'git log -S22993572': vacío`; `476eaaf 'Add CITATION.cff with the concept DOI'`; `WebFetch zenodo.org/api/records/22993572 y doi.org: EGRESS_BLOCKED`

### H10c

**CONFIRMADA** · gravedad baja

- **Borrador:** ¿son al menos coherentes con R los valores inventados?
- **Lo que se puede afirmar:** Los cuatro pares inventados son aritméticamente coherentes con R, pero no son vectores de la especificación y no deben citarse como tales.
- **Fuentes:** `python3 con verify-state.reference(): (10,10) InSync index=10; (15,10) CounterAhead orphans=5; (5,12) KeyAhead unrecorded=7 fatal=True; (20,0) KeyAtZero indeterminate=20`; `python3 -B con spec/verify-state.py reference(): (10,10) InSync index=10; (15,10) CounterAhead orphans=5; (5,12) KeyAhead unrecorded=7 fatal=True; (20,0) KeyAtZero indeterminate=20`

### H11a

**CONFIRMADA** · gravedad baja

- **Borrador:** XMSSMT-SHA2_40/8_256, OID 0x00000005
- **Lo que se puede afirmar:** XMSSMT-SHA2_40/8_256 (h=40, d=8, n=32, w=16) tiene OID 0x00000005 en el registro XMSS^MT de RFC 8391 (Table 8, §5.4). La atribución a la 'tabla 11' de SP 800-208 no se ha podido verificar.
- **Fuentes:** `RFC 8391, tabla IANA: "0x00000005 \| XMSSMT-SHA2_40/8_256 \| Section 5.4" (leído en una réplica del texto clonada de github.com/tex2e/rfc-translater, html/rfc8391.html)`; `~/.cargo/registry xmss-0.1.0-pre.0/src/params.rs:960 'XmssMtSha2_40_8_256 = XMSSMT_OID_OFFSET \| 0x05'`; `hbs-state/spec/HBS-STATE-v0.3 (md):265 "id 0x00000005, SP 800-208 table 11" (número de tabla NO verificado: nvlpubs.nist.gov bloqueado)`; `réplica de RFC 8391 (clon github.com/tex2e/rfc-translater, html/rfc8391.html): '0x00000005 \| XMSSMT-SHA2_40/8_256 \| Section 5.4', tabla titulada 'Table 8'`; `~/.cargo/registry/.../xmss-0.1.0-pre.0/src/params.rs:960 'XmssMtSha2_40_8_256 = XMSSMT_OID_OFFSET \| 0x05'`; `hbs-state/spec/HBS-STATE-v0.3 (md):265 y README.md:143 'SP 800-208, table 11' (no verificado)`; `WebSearch 'SP 800-208 Table 11 XMSS^MT ... 0x00000005': sin confirmación`

### H11b

**CONFIRMADA** · gravedad baja

- **Borrador:** SK layout = OID(4) + index(5, big-endian) + 4x32 = 137 B
- **Lo que se puede afirmar:** El layout es OID(4, big-endian) \|\| idx (ceil(h/8) B big-endian; 4 B en árbol único) \|\| SK_SEED \|\| SK_PRF \|\| root \|\| PUB_SEED (4×n). Para 40/8: 4+5+128 = 137 B; en árbol único, 136 B. Es el layout de la implementación de referencia en C (xmss-reference @171ccbd), que xmss 0.1.0-pre.0 porta.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):266-268`; `xmss-0.1.0-pre.0/src/params.rs:59-62 'index_bytes = if d == 1 { 4 } else { h.div_ceil(8) }; 4 + index_bytes + 4 * n'`; `xmss-0.1.0-pre.0/src/xmss_core.rs:92 "Format sk: [(ceil(h/8) bit) index \|\| SK_SEED \|\| SK_PRF \|\| root \|\| PUB_SEED]"`; `hbs-state/tests/vectors.rs:277-287 SK de 137 se lee, de 136 se rechaza`; `xmss-0.1.0-pre.0/src/params.rs:59-61 'let index_bytes = if d == 1 { 4 } else { h.div_ceil(8) }; 4 + index_bytes + 4 * n'`; `xmss-0.1.0-pre.0/src/xmss_core.rs:92 'Format sk: [(ceil(h/8) bit) index \|\| SK_SEED \|\| SK_PRF \|\| root \|\| PUB_SEED]'`; `clon github.com/XMSS/xmss-reference @171ccbd xmss_core.c:131,165 mismo comentario; params.c:738-743 index_bytes; xmss.c:18-25 OID de 4 B antepuesto al SK; README.md:17 'written by `; `hbs-state/tests/vectors.rs:277-287`

### H12a

**CONFIRMADA** · gravedad baja

- **Borrador:** extraído directamente del sistema ARQUEO
- **Lo que se puede afirmar:** hbs-state es una traducción al inglés del crate zk-ssl-guardian de ARQUEO, con la reconciliación extraída como función pura reconcile_values.
- **Fuentes:** `crates/zk-ssl-guardian/src/lib.rs (ARQUEO): GuardianIndice, reservar, reconciliar, Reconciliacion{Coincide, ContadorAdelantado, ClaveEnCero, ClaveAdelantada}, no_admite_matiz, MUES`; `hbs-state/spec/state-vectors-v0.3.json:376 las notas de procedencia apuntan al guardián de ARQUEO`; `crates/zk-ssl-guardian/src/lib.rs:96, 102, 107 MUESTRAS_AUTOCOMPROBACION=20, RAZON_MINIMA=10.0, SUELO_MICROS=20.0`; `crates/zk-ssl-guardian/src/lib.rs:325-352 Reconciliacion{Coincide, ContadorAdelantado, ClaveEnCero, ClaveAdelantada}; :369 no_admite_matiz; :419-527 abrir/reservar/reconciliar/pers`; `hbs-state/spec/state-vectors-v0.3.json:376 'the source and note fields point into the ARQUEO guardian'`; `hbs-state/README.md:169-171`

### H13b

**CONFIRMADA** · gravedad baja

- **Borrador:** v0.2.0 / v0.3 spec; September 2026
- **Lo que se puede afirmar:** Crate 0.2.0 (publicado el 2026-09-27) que implementa la especificación HBS-STATE v0.3 revisada el 2026-09-27. La release v0.1.0 (2026-09-26) contiene la v0.3 antes de las dos correcciones.
- **Fuentes:** `hbs-state/CITATION.cff:10-11 version 0.2.0, date-released 2026-09-27`; `hbs-state/.zenodo.json:17`; `git ls-remote: tags v0.1.0 (4368860, 2026-09-26) y v0.2.0 (a960828, 2026-09-27)`; `hbs-state/spec/HBS-STATE-v0.3 (md):37-38 v0.1.0 archivado el 2026-09-26 con la v0.3 sin corregir`; `hbs-state/.zenodo.json:4 "version": "0.2.0"; :19`; `clon completo: v0.1.0^{} = 4368860 (2026-09-26 18:48); v0.2.0^{} = a960828 (2026-09-27 11:08)`; `hbs-state/spec/HBS-STATE-v0.3 (md):37-40`

### H14d

**CONFIRMADA** · gravedad baja

- **Borrador:** su sección 5 da nueve estrategias de gestión del estado
- **Lo que se puede afirmar:** En el texto fuente del WG, la sección 5 'Potential State Management Approaches' tiene nueve subsecciones (numeración del RFC final no verificada); ninguna trae vectores ejecutables, algo esperable en un documento informativo. Una de ellas, la reserva por intervalos (McGrew et al. 2016), es la técnica que hbs-state implementa con intervalo 1.
- **Fuentes:** `draft-ietf-pquip-hbs-state (md):550-841 '# Potential State Management Approaches' con 9 subsecciones: Multiple Public Keys; Distributed Multi-trees; Sectorization; Key/State Transf`; `draft-ietf-pquip-hbs-state (md):550 '# Potential State Management Approaches'; :560, 585, 627, 673, 704, 717, 742, 769, 841 (nueve '##')`; `draft-ietf-pquip-hbs-state (md):843-852 'State Reservation Strategy described in Section 5 of {{MCGREW}}'; :95-106 MCGREW = eprint 2016/357`

### H14e

**CONFIRMADA** · gravedad baja

- **Borrador:** SP 800-208 exige que la generación de claves y firmas se valide solo dentro de módulos hardware
- **Lo que se puede afirmar:** SP 800-208 dispone que las implementaciones de generación de claves y de firma solo se validen dentro de módulos criptográficos hardware (FIPS 140-2/140-3 nivel 3 o superior de seguridad física). Confirmado por fragmento de búsqueda, no por lectura directa del PDF.
- **Fuentes:** `WebSearch (fragmento del PDF de NIST, nvlpubs bloqueado para lectura directa): "Implementations of the key generation and signature algorithms in this document shall only be valida`; `draft-ietf-pquip-hbs-state (md):591-593 "SP.800-208, which does not permit export of private key material from cryptographic modules"`; `WebSearch: 'Implementations of the key generation and signature algorithms in this document shall only be validated for use within hardware cryptographic modules' ... 'FIPS 140-2 o`; `WebFetch nvlpubs.nist.gov: EGRESS_BLOCKED`

### H14f

**CONFIRMADA** · gravedad baja

- **Borrador:** SP 800-208 §8.1: "The cryptographic module shall not allow for the export of private keying material."
- **Lo que se puede afirmar:** La frase 'The cryptographic module shall not allow for the export of private keying material' aparece en SP 800-208 (fuente secundaria: fragmento de búsqueda; que esté en la §8.1 no se ha comprobado). NIST ha anunciado una revisión que permitiría exportar claves.
- **Fuentes:** `WebSearch: fragmento que reproduce la frase y añade que NIST trabaja en una revisión de SP 800-208 que permitiría la exportación de claves`; `hbs-state/spec/HBS-STATE-v0.3 (md):297-299`; `número de sección §8.1 solo según el fragmento; PDF bloqueado`; `WebSearch: 'The cryptographic module shall not allow for the export of private keying material' en SP 800-208; NIST 'is currently working on a revision of SP 800-208 that would ena`; `WebSearch '"SP 800-208" "Section 8.1"': sin confirmación`; `hbs-state/spec/HBS-STATE-v0.3 (md):297-299, 317-319`

### H18a

**CONFIRMADA** · gravedad media

- **Borrador:** 25 of 25 process deaths with not one signature ahead
- **Lo que se puede afirmar:** Registro de ARQUEO (banco K.1, §234): 25 muertes de un proceso hijo que persiste y luego firma, matado por el padre en un instante aleatorio, sin ninguna firma por delante del contador. Es una muestra de 25 sobre el guardián original de ARQUEO, dentro de un proceso y en una máquina, sin script reproducible en ninguno de los dos árboles.
- **Fuentes:** `AUDITORIA.md §234 (línea 16861-16862): "K.1 lo probó con un hijo que persiste-y-luego-firma y un padre que lo mata en un instante aleatorio: 25 de 25 sin una sola firma por delante`; `hbs-state/README.md:96-100; hbs-state/src/lib.rs:65-69`; `no hay script de K.1 en ninguno de los dos árboles (grep en tools/, doc/, spec/)`; `AUDITORIA.md §234 (16859-16862) encabezado 'se sostiene bajo 'kill -9''; 'un hijo que persiste-y-luego-firma y un padre que lo mata en un instante aleatorio: **25 de 25 sin una sol`; `grep de K.1 en tools/, doc/, spec/ y crates: solo referencias, ningún script del banco`; `AUDITORIA.md §234 (16859-16862)`; `git grep 'K\.1' HEAD: solo referencias`

### H18b

**CONFIRMADA** · gravedad media

- **Borrador:** el '13 de 25' retractado (spec §0)
- **Lo que se puede afirmar:** 13 de 25 muertes (52 %) DENTRO de un proceso dejaron el contador por delante (huérfanos). La inferencia 'lo normal tras un crash es CounterAhead' se retractó en spec §0, porque tras un reinicio con SK derivado de semilla el estado es KeyAtZero. El comentario de módulo de src/lib.rs aún conserva la inferencia retractada.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):69-86 cita la frase de la v0.1 y declara FALSA la inferencia ("the normal thing after a crash is for the counter to be ahead"); la cifra 13/25 se`; `AUDITORIA.md §234 "13 de 25 dejaron el contador adelantado ... El 52 %"; §331 "la cifra es correcta y la inferencia no"`; `hbs-state/src/lib.rs:33-36, :326 y :612 aún repiten '13 of 25 ... normal path after a crash'`; `hbs-state/spec/HBS-STATE-v0.3 (md):69-86`; `AUDITORIA.md §234 (16864-16866) '13 de 25 ... El 52 %'; §331 (25552-25558) 'la cifra es correcta y la inferencia no'`; `hbs-state/src/lib.rs:33-36, :326, :612`; `AUDITORIA.md §234 (16864-16866); §331 (25552-25558)`

### H19a

**CONFIRMADA** · gravedad baja

- **Borrador:** estado: DRAFT, derivada de una sola implementación, necesita un segundo sujeto independiente
- **Lo que se puede afirmar:** HBS-STATE v0.3 es un DRAFT (revisado el 2026-09-27) derivado de una única implementación; hasta que un segundo sujeto independiente pase los vectores, 'describe un programa'. En los vectores, 'MEASURED' significa extraído de la fuente y los tests del guardián de ARQUEO: es autoconformidad.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):420-429 "v0.3 -- DRAFT, revised 2026-09-27. Derived from one single implementation ... a second INDEPENDENT subject measured against these vector`; `hbs-state/spec/HBS-STATE-v0.3 (md):420-429`; `hbs-state/spec/HBS-STATE-v0.3 (md):23-24 'MEASURED = taken byte by byte from a production implementation (the ARQUEO index guard)'`

### H19b

**CONFIRMADA** · gravedad baja

- **Borrador:** hbs-lms medido como segundo implementador (no sujeto)
- **Lo que se puede afirmar:** hbs-lms 0.2.0-alpha.1 (Fraunhofer AISEC, commit 7063cc8) se midió el 2026-09-27 con un arnés aparte, no con el protocolo de sujeto, y no se le atribuye nivel. Sus dos hechos de código se verifican leyendo su fuente: solo entrega la firma después de que el persistidor del llamante haya aceptado el estado actualizado, y borra la clave al agotarse. El arnés no está publicado en ninguno de los dos árboles.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):431-450`; `hbs-lms @7063cc8 (commit 2024-09-30 'Update clap to v4', version 0.2.0-alpha.1, Apache-2.0, authors Fraunhofer AISEC) src/hss/mod.rs:224-236: firma, luego increment() y private_key`; `el arnés de medición de hbs-lms no está en ninguno de los dos repos`; `hbs-lms @7063cc8 (2024-09-30 'Update clap to v4'; Cargo.toml:3 0.2.0-alpha.1, :5 Fraunhofer AISEC, :11 Apache-2.0) src/hss/mod.rs:224-236`; `grep 'hbs-lms\|hbs_lms' en ambos repos: solo aparece en spec/HBS-STATE-v0.3 (md) (el arnés no está publicado)`

### H19c

**CONFIRMADA** · gravedad baja

- **Borrador:** pq-xmss no es un segundo implementador
- **Lo que se puede afirmar:** pq-xmss 0.2.0 comparte autor y código con xmss (hash_address.rs y utils.rs idénticos byte a byte): es un segundo crate, no un segundo implementador. Los dos siguen la implementación de referencia en C. oxicrypt-xmss quedó descartado (versiones yanked; 0.24.0 solo verifica), según spec §10.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):468-473`; `verificado: pq-xmss (github.com/mikelodder7/pq-xmss, version 0.2.0) y xmss-0.1.0-pre.0 tienen el mismo autor (Michael Lodder) y src/hash_address.rs (65 líneas) y src/utils.rs (19 l`; `cmp pq-xmss/src/hash_address.rs (65 líneas) y utils.rs (19) contra xmss-0.1.0-pre.0/src/: idénticos; ambos Cargo.toml: authors = Michael Lodder; pq-xmss version 0.2.0`; `los 24 nombres de función de xmss-0.1.0-pre.0 (xmss_core.rs, hash_address.rs, wots.rs) existen en xmss-reference @171ccbd *.c`

### H19d

**CONFIRMADA** · gravedad baja

- **Borrador:** B8 y familia C declarados sin vectores
- **Lo que se puede afirmar:** B8 (con h no múltiplo de 8, escribir 2^h debe fallar aunque quepa en el campo; la implementación de origen no lo cubre) y la familia C (siete errores sobre el estado en disco, entre ellos FakePersistence) están declarados sin vectores y no puntúan.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):337-340 (B8 CANDIDATE, unmeasured), :344-360 (familia C), :387-388`; `hbs-state/spec/state-vectors-v0.3.json:338-361 candidates_without_vectors [B8, C]`; `salida de verify-state.py: "there are 2 CANDIDATE families with no vectors yet: B8, C"`; `hbs-state/spec/HBS-STATE-v0.3 (md):337-340, 344-360, 387-388`; `hbs-state/spec/state-vectors-v0.3.json:338-361`; `python3 -B spec/verify-state.py: 'there are 2 CANDIDATE families with no vectors yet: B8, C'`

### H1a

**CONFIRMADA** · gravedad baja

- **Borrador:** hbs-state es la implementación de referencia de HBS-STATE v0.3 en Rust
- **Lo que se puede afirmar:** hbs-state (crate 0.2.0, Rust) es la implementación de referencia de la especificación HBS-STATE v0.3; la versión del crate y la de la especificación son independientes (.zenodo.json:19).
- **Fuentes:** `hbs-state/README.md:8 "It is the **reference implementation of [HBS-STATE](spec/HBS-STATE-v0.3 (md))**"`; `hbs-state/.zenodo.json:17 "This release is crate version 0.2.0 and it implements specification HBS-STATE v0.3"`; `hbs-state/Cargo.toml: version = "0.2.0", edition 2021`; `hbs-state/Cargo.toml:3 version = "0.2.0"; :6 description "... Reference implementation of HBS-STATE."`; `hbs-state/.zenodo.json:19 "This release is crate version 0.2.0 and it implements specification HBS-STATE v0.3 ... The two version numbers are independent"`

### H1c

**CONFIRMADA** · gravedad baja

- **Borrador:** no genera ni verifica firmas
- **Lo que se puede afirmar:** No firma ni verifica; sí manipula material de clave: lee y escribe el índice dentro de los bytes del SK (index_from_sk/set_index_in_sk) y lee semillas de 96 B comprobando permisos (src/seed.rs).
- **Fuentes:** `hbs-state/Cargo.toml:59-63 [dependencies] y [dev-dependencies] vacíos (no depende de xmss)`; `hbs-state/src/lib.rs:10-12 "Signing is the OPERATOR's duty"`; `hbs-state/src/lib.rs:126 "it drags no 'xmss' in: it is offset arithmetic over '&[u8]'"`; `AUDITORIA.md §234 "El guardián no firma: cuenta."`; `hbs-state/Cargo.toml:17 [dependencies] y :21 [dev-dependencies], ambos vacíos`; `hbs-state/src/lib.rs:10-11 "Signing is the OPERATOR's duty"`; `hbs-state/src/lib.rs:126-127 "it drags no 'xmss' in: it is offset arithmetic over '&[u8]'"`; `AUDITORIA.md §234 (línea ~16907) "El guardián no firma: cuenta."`

### H2a

**CONFIRMADA** · gravedad baja

- **Borrador:** API: IndexGuard::open(path)
- **Lo que se puede afirmar:** `IndexGuard::open(path: impl AsRef<Path>) -> Result<IndexGuard, GuardError>`: crea el directorio padre, ejecuta la autocomprobación de fsync en ese directorio, lee el contador (8 bytes little-endian; otro tamaño => GuardError::Corrupt) y reescribe+fsync el valor de arranque, incluso si es 0. Si el fichero no existe arranca en 0 sin avisar, de modo que un contador borrado es indistinguible de uno nuevo (src/lib.rs:448-473).
- **Fuentes:** `hbs-state/src/lib.rs:448 'pub fn open(path: impl AsRef<Path>) -> Result<Self, GuardError>'`; `hbs-state/src/lib.rs:449-472: create_dir_all(padre) -> check_persistence(padre) -> si existe, lee y exige 8 bytes (si no, Corrupt{bytes}) -> u64::from_le_bytes -> persist(current) `; `hbs-state/src/lib.rs:450-453 create_dir_all(padre) y check_persistence(&folder) ANTES de leer`; `hbs-state/src/lib.rs:455-466 si existe: exige 8 bytes (si no Corrupt{bytes}), u64::from_le_bytes; si no existe: 0`; `hbs-state/src/lib.rs:468-472 persist(current) incluso si es 0`; `hbs-state/src/lib.rs:412-415 (0,0) => InSync`

### H2b

**CONFIRMADA** · gravedad baja

- **Borrador:** reserve() persiste con fsync ANTES de devolver el índice
- **Lo que se puede afirmar:** reserve(&mut self) calcula current+1 (desbordamiento de u64 => GuardError::Io), lo persiste (abre el fichero, seek 0, escribe 8 bytes little-endian en sitio, sync_all = fsync(2) del fichero) y solo entonces actualiza el estado en memoria y devuelve el índice. No hay fsync del directorio padre, ni temporal+rename, ni cerrojo, ni fijación de permisos del fichero del contador; la comprobación de permisos (sin bits de grupo/otros) solo se aplica a ficheros de semilla (src/seed.rs:32-53). Es la estrategia de reserva de McGrew et al. 2016 con intervalo 1.
- **Fuentes:** `hbs-state/src/lib.rs:482-489 'pub fn reserve(&mut self) -> Result<u64, GuardError>': checked_add(1) -> self.persist(next_one)? -> self.current = next_one -> Ok(next_one)`; `hbs-state/src/lib.rs:506-519 persist: OpenOptions create+write+truncate(false) -> seek(0) -> write_all(value.to_le_bytes()) -> sync_all() ("'sync_all' is 'fsync(2)': data AND metad`; `hbs-state/src/lib.rs:482-489 reserve: checked_add(1) -> persist(next_one)? -> self.current = next_one -> Ok(next_one)`; `hbs-state/src/lib.rs:506-519 persist: OpenOptions create+write+truncate(false), seek(0), write_all(to_le_bytes), sync_all ("'sync_all' is 'fsync(2)'")`; `hbs-state/src/seed.rs:41 'if mode & 0o077 != 0' => PermissionsTooOpen`; `clon github.com/hbs-guidance/draft-hbs-state @4ee80a2 draft-ietf-pquip-hbs-state (md):843-852 reserva escrita a memoria no volátil 'prior to their usage'; claves reservadas se pier`

### H2c

**CONFIRMADA** · gravedad baja

- **Borrador:** reconcile(key) devuelve Reconciliation con cuatro variantes; reconcile_values existe
- **Lo que se puede afirmar:** `reconcile(&self, key_index: u64) -> Reconciliation` delega en la función pura `reconcile_values(counter: u64, key: u64)`; el índice de la clave lo aporta el llamante (p. ej. con index_from_sk). Variantes: InSync{index}, CounterAhead{counter,key,orphans}, KeyAtZero{counter,indeterminate}, KeyAhead{counter,key,unrecorded}; el juez es `is_fatal(&Reconciliation) -> bool`.
- **Fuentes:** `hbs-state/src/lib.rs:502-504 'pub fn reconcile(&self, key_index: u64) -> Reconciliation { reconcile_values(self.current, key_index) }'`; `hbs-state/src/lib.rs:412 'pub fn reconcile_values(counter: u64, key: u64) -> Reconciliation'`; `hbs-state/src/lib.rs:323-354 enum Reconciliation { InSync{index}, CounterAhead{counter,key,orphans}, KeyAtZero{counter,indeterminate}, KeyAhead{counter,key,unrecorded} }`; `hbs-state/src/lib.rs:370-377 'pub fn is_fatal(r: &Reconciliation) -> bool'`; `test the_method_and_the_free_function_agree (src/lib.rs:801-815)`; `hbs-state/src/lib.rs:323-354 enum de cuatro variantes`; `hbs-state/src/lib.rs:370-377 is_fatal`; `hbs-state/src/lib.rs:801-815 test the_method_and_the_free_function_agree`

### H3

**CONFIRMADA** · gravedad baja

- **Borrador:** R(C,K): InSync (key==counter, index=counter); CounterAhead (counter>key, key!=0, orphans=counter-key); KeyAtZero (counter>key, key==0, indeterminate=counter); KeyAhead (key>counter, unrecorded=key-counter, fatal)
- **Lo que se puede afirmar:** R es una función pura de (counter,key) en u64: Equal => InSync(index=counter); Greater con key==0 => KeyAtZero(indeterminate=counter, un solo término); Greater => CounterAhead(orphans=counter-key); Less => KeyAhead(unrecorded=key-counter). Precedencia: Equal antes que la guarda, así que (0,0) es InSync, y la guarda `Greater if key == 0` va antes del Greater general. (0,1) es KeyAhead. No hay variante de agotamiento (A11/A12 en 2^40-1). Solo KeyAhead es fatal (is_fatal, src/lib.rs:370-377).
- **Fuentes:** `hbs-state/src/lib.rs:412-431 match counter.cmp(&key) { Equal => InSync{index: counter}, Greater if key == 0 => KeyAtZero{indeterminate: counter}, Greater => CounterAhead{orphans: c`; `hbs-state/spec/HBS-STATE-v0.3 (md):141-151 tabla y "one guard: 'Greater if key == 0'. That guard is the precedence."`; `hbs-state/spec/HBS-STATE-v0.3 (md):364-372 D1-D5 CONFIRMED (D2 (0,0) InSync; D4 (0,1) KeyAhead; D5 sin variante de agotamiento)`; `hbs-state/spec/verify-state.py:53-71 regla de referencia equivalente`; `python3 spec/verify-state.py (modo referencia): family A N1/N2/N3 12/12`; `hbs-state/spec/HBS-STATE-v0.3 (md):141-151 y :364-372 (D1-D5)`; `hbs-state/spec/verify-state.py:53-71`; `python3 -B spec/verify-state.py: 'family A (12 vectors): N1 12/12 N2 12/12 N3 12/12', rc=0`

### H7a

**CONFIRMADA** · gravedad baja

- **Borrador:** IndexGuard::open mide el tiempo de fsync al arrancar y se niega si es indistinguible de no persistir (tmpfs)
- **Lo que se puede afirmar:** open() mide fsync en el directorio padre del contador con un fichero sonda `.hbs-state-selfcheck` y devuelve GuardError::FakePersistence{with_fsync_us, without_fsync_us, ratio} si, a la vez, la razón con/sin fsync es < 10 y el coste medio con fsync es < 20 µs.
- **Fuentes:** `hbs-state/src/lib.rs:453 Self::check_persistence(&folder)? antes de leer el contador`; `hbs-state/src/lib.rs:521-556 check_persistence`; `hbs-state/src/lib.rs:548-554 'if ratio < MIN_RATIO && with < FLOOR_MICROS { return Err(GuardError::FakePersistence{with_fsync_us, without_fsync_us, ratio}) }'`; `hbs-state/README.md:78-79`; `tests on_tmpfs_it_refuses_to_operate y on_a_real_disk_it_does_operate (src/lib.rs:747-822)`; `hbs-state/src/lib.rs:450-453 folder = path.parent(); check_persistence(&folder)`; `hbs-state/src/lib.rs:523-556 sonda '.hbs-state-selfcheck'; 'if ratio < MIN_RATIO && with < FLOOR_MICROS' => FakePersistence`; `hbs-state/src/lib.rs:747-796 on_tmpfs_it_refuses_to_operate; :817-822 on_a_real_disk_it_does_operate`

### H7c1

**CONFIRMADA** · gravedad baja

- **Borrador:** ext4 ~0.9 ms (382 veces más lento), tmpfs ~0.002 ms
- **Lo que se puede afirmar:** Medido una vez (banco K.1, AUDITORIA §234) en una sola máquina (WSL2 sobre i5-1135G7; el 'ext4' es $HOME bajo WSL2, un disco virtual): fsync en ext4 0,907 ms, 382× frente a no persistir; en tmpfs 0,002 ms, 1×. El script de K.1 no está en ninguno de los dos árboles.
- **Fuentes:** `hbs-state/README.md:82-85 ext4 0.907 ms 382x; tmpfs 0.002 ms 1x`; `hbs-state/src/lib.rs:40-45 y :58-61 "the thresholds come from ONE machine -- WSL2 on an i5-1135G7 -- and are declared, not derived"`; `AUDITORIA.md §234 banco K.1 ($HOME ext4 0,907 ms 382×; /tmp tmpfs 0,002 ms 1×)`; `hbs-state/README.md:82-85`; `hbs-state/src/lib.rs:40-45 y :58-61 'WSL2 on an i5-1135G7 -- and are declared, not derived'`; `AUDITORIA.md §234 (16835-16838) '$HOME (ext4) 0,907 ms 382×; /tmp (tmpfs) 0,002 ms 1×'`; `grep de K.1 en tools/, doc/, spec/: ningún script implementa el banco`; `AUDITORIA.md §234 (16835-16838)`

### H8

**CONFIRMADA** · gravedad baja

- **Borrador:** Cero dependencias; el lector de vectores JSON está escrito a mano (sin serde)
- **Lo que se puede afirmar:** Confirmado: cero dependencias de Cargo en [dependencies] y [dev-dependencies]; el lector JSON de tests/vectors.rs es ad hoc y comprueba el número de filas (12/6/1). Matiz: el test de tmpfs invoca el binario externo `df`, y el verificador es Python con solo la biblioteca estándar.
- **Fuentes:** `hbs-state/Cargo.toml:56-63 [dependencies] y [dev-dependencies] vacíos`; `hbs-state/Cargo.lock: un único paquete (hbs-state 0.2.0)`; `hbs-state/tests/vectors.rs:6-11 "the JSON reader is written BY HAND, without serde"; funciones collection/objects/number/text (32-97)`; `hbs-state/src/bin/subject.rs:27-30 el JSON de salida también se escribe a mano`; `hbs-state/Cargo.toml:14-21 comentario ZERO DEPENDENCIES, [dependencies] y [dev-dependencies] vacíos`; `hbs-state/Cargo.lock: único [[package]] name = "hbs-state" 0.2.0`; `hbs-state/tests/vectors.rs:6-11 'the JSON reader is written BY HAND, without 'serde''; :19-28 EXPECTED_ROWS 12/6/1; :32-97 collection/objects/number/text`; `hbs-state/src/bin/subject.rs:27-30 salida JSON a mano`

### H9c

**CONFIRMADA** · gravedad baja

- **Borrador:** Verificador en Python, binario sujeto hbs-state-subject, protocolo de sujeto
- **Lo que se puede afirmar:** Confirmado. Añadir: el sujeto de referencia declara seed_derived y no toca disco ni claves (subject.rs:17-19, 63-68), así que su N3 certifica la función pura y no la durabilidad del IndexGuard. El verificador no ejercita B7 (solo 'reconciliation' e 'index_read', 18 filas).
- **Fuentes:** `hbs-state/spec/verify-state.py (367 líneas; imports argparse, glob, json, os, subprocess, sys)`; `hbs-state/Cargo.toml:65-67 [[bin]] name = "hbs-state-subject"`; `hbs-state/spec/HBS-STATE-v0.3 (md):208-238 protocolo: '--model', '<counter> <key>', '--sk <hex>'; se lee la ÚLTIMA línea de stdout como JSON; salida no cero/vacía/no JSON = fila fa`; `hbs-state/spec/verify-state.py:42 rc 0/1/2`; `hbs-state/spec/verify-state.py (367 líneas; imports argparse, glob, json, os, subprocess, sys; :42 rc 0/1/2)`; `hbs-state/Cargo.toml:23-25 [[bin]] name = "hbs-state-subject"`; `hbs-state/spec/HBS-STATE-v0.3 (md):208-238 (60 s en :232)`; `hbs-state/src/bin/subject.rs:17-19, 63-68`

## Bloque G — el guardián del índice de Arqueo y su relación con HBS-STATE

### G1f

**FALSA** · gravedad alta · cambió con el árbol (S566–S582)

- **Borrador:** En KeyAtZero (estado de todo reinicio real) la implementación de origen 'falla cerrada' / se niega a arrancar (HBS-STATE §2: 'KeyAtZero cannot be resolved, only failed closed')
- **Lo que se puede afirmar:** Arqueo no se niega a arrancar en ClaveEnCero: el nodo y el testigo cofirmante resincronizan la clave al valor del contador y abandonan 0..contador−1 como perdidos, y sólo fallan cerrados si un segundo registro en disco (el diario del nodo, con '<'; el fichero de cofirmas del testigo, con '<=') prueba un índice mayor que el contador (main.rs:608-643 y 991-1001; witness.rs:2732-2757). Desde el §579, tools/banco_reutilizacion.sh lo MIDE para el testigo: tras reiniciar, resincroniza (lo asierta por la línea de log), y con el contador restaurado hacia atrás no arranca. Su chequeo de 'no repite índice' mira el ordinal declarado, no el índice WOTS embebido. En el otro uso de la misma Reconciliacion, el registro de recepción (§569), ClaveEnCero arranca avisando sin resincronizar (main.rs:674-680). 'Fallar cerrado' en HBS-STATE debe entenderse como 'no reutilizar los índices indeterminados', no como 'no operar' (AUDITORIA §331, §335, §337, §579).
- **Fuentes:** `crates/zk-ssl-node/src/main.rs:569-601 rama ClaveEnCero: si el diario tiene un índice d con contador < d → NoArranca; en otro caso ArrancaResincronizando{hasta: contador} con aviso`; `crates/zk-ssl-node/src/main.rs:833-845 el arranque llama f.resincronizar_a(hasta) (firma_cabeza.rs:225-247: pone el índice del SK en 'contador', aplica el apaño del OID y relee)`; `crates/zk-ssl-cli/src/witness.rs:2695-2760 politica_del_cofirmante: igual, pero contra el máximo índice EMBEBIDO del fichero de cofirmas y con '<=' (NoArranca si contador <= tope)`; `main.rs:647-666 test la_clave_en_cero_resincroniza_si_el_diario_no_lo_contradice; main.rs:688-736 mod gate_del_diario (incluye sin_diario_el_gate_pasa_y_ese_es_su_limite)`; `AUDITORIA.md §331 (2026-08-20): ClaveEnCero nació fallando cerrada; §335 (2026-08-20): 'cualquier reinicio posterior a la primera cabeza firmada dejaba el nodo muerto' → se resincr`; `Comentarios rancios que aún dicen lo contrario: main.rs:556-557 "Y su hermano ClaveEnCero SI para"; witness.rs:2335-2337 "sale ClaveEnCero, que NO arranca"`; `hbs-state/spec/HBS-STATE-v0.3 (md):163 (título) y :383-384 'Policy ... belongs to the owner'; is_fatal(KeyAtZero)=false (hbs-state/src/lib.rs:374)`; `crates/zk-ssl-node/src/main.rs:576-611 rama ClaveEnCero: 'Some(d) if *contador < d => NoArranca' / '_ => ArrancaResincronizando{hasta: *contador, aviso: ... SE ABANDONAN ... PERDID`

### G2a

**FALSA** · gravedad alta

- **Borrador:** El guardián 'está en producción desde hace un año' / 'lleva operando en entornos reales de firma'
- **Lo que se puede afirmar:** No hay evidencia de uso en producción: Arqueo empezó el 29-07-2026, el guardián nació en la entrada 234 (primera quincena de agosto de 2026, unas siete semanas antes de publicar hbs-state), y el propio repositorio se declara prototipo de investigación sin producción, sin clave de operador real y 'sin operador real' todavía al aceptar el RFC-0010 (SECURITY.md:27-28, 145-148, 584; QUESTIONS.md:31; RFC-0010:314-318). Las únicas ejecuciones documentadas son bancos y pruebas en la máquina del autor. La frase 'in production for a year' de hbs-state (README.md:34, :170; src/lib.rs:73-74; spec:23) es falsa y debe retirarse, no repetirse.
- **Fuentes:** `GENAI.md: "Desde cuándo: desde la primera sesión del proyecto, el 29 de julio de 2026"`; `AUDITORIA.md '## 234.' (l.16819) nace el guardián; está entre el triaje del 2026-08-03 (AUDITORIA.md:12110) y BACKLOG.md:3076-3080, que lo da por hecho 'medido sobre af9e786, 10-08`; `SECURITY.md:27-28 "Prototipo de investigación, no un producto. No maneja dinero real y no debe manejarlo"`; `SECURITY.md:145-148 "NO HAY CLAVE QUE ANCLAR ... El operador no tiene ninguna; --clave existe «para ejercitar el mecanismo, no como forma de operar»"`; `doc/INSTITUCIONAL.md:91 "no lo resuelve en producción porque no hay producción"; doc/INSTITUTIONAL.md:190 "no production environment"`; `La afirmación procede del propio repo hbs-state: hbs-state/README.md:33-34 y :169-170, hbs-state/src/lib.rs:73-74 ('signing for a year'), hbs-state/spec/HBS-STATE-v0.3 (md):23 ('a `; `hbs-state v0.1.0 archivada 2026-09-26 (spec:35-36); v0.2.0 2026-09-27 (CITATION.cff)`; `GENAI.md:16 'Desde cuándo: desde la primera sesión del proyecto, el 29 de julio de 2026'`

### G5b

**FALSA** · gravedad alta

- **Borrador:** Cada época requiere la co-firma de testigos (para que el paquete verifique)
- **Lo que se puede afirmar:** Las cofirmas son opcionales: una cabeza firmada verifica sin ninguna, el paquete v1 no las lleva y el v2 las admite (incluso cero); el verificador informa cuántas verifican y el umbral k lo fija el cliente, no el protocolo. El sobre de completitud del RFC-0010 tampoco las exige (spec/PAQUETE.md:29-33, 60-68, 445, 461-463, 744, 2.11).
- **Fuentes:** `spec/PAQUETE.md:29-33 'Qué testigos valen y cuántos hacen falta lo decide el CLIENTE con su política (--testigos y --k), no el paquete'`; `spec/PAQUETE.md:60-67 paquete v2: 'cofirmas: [...] OPCIONAL'; v1 no lleva cofirmas (PAQUETE.md:399)`; `spec/PAQUETE.md:415-417 paso 4: 'Se imprime cuántas verifican; cuántas hacen falta no es asunto del paquete'; :679`; `spec/PAQUETE.md:695 vector positivo 'v2 con cero cofirmas'`; `spec/RPC.md:893-905 el nodo solo transporta: comprueba época en curso y firma, no acredita al testigo; una cofirma viva por clave de testigo y solo la época en curso`; `spec/PAQUETE.md:29-33 'Qué testigos valen y cuántos hacen falta lo decide el CLIENTE ... no el paquete'`; `spec/PAQUETE.md:60-68 v2 'cofirmas: [...] OPCIONAL'; :399 'un v1 no trae cofirmas'`; `spec/PAQUETE.md:415-417 'Se imprime cuántas verifican; cuántas hacen falta no es asunto del paquete'; :679`

### G5c

**FALSA** · gravedad media

- **Borrador:** El nodo acredita/verifica a los testigos al recibir cofirmas
- **Lo que se puede afirmar:** El nodo transporta cofirmas (zkssl_submitCosig/zkssl_cosigs): comprueba que son de la época en curso y verifica criptográficamente su firma con la clave del testigo que viaja en el propio objeto, pero no acredita la identidad de ningún testigo; qué testigos cuentan lo decide el cliente (main.rs:1883-1935; spec/RPC.md:961-975). El mensaje de witness.rs:2304 ('el nodo no verifica, solo filtra') es inexacto.
- **Fuentes:** `spec/RPC.md:874-898 zkssl_submitCosig/zkssl_cosigs: el nodo es TRANSPORTE; 'Comprueba que la cofirma es de la epoca EN CURSO y que la firma cierra ... No acredita al testigo, y no `; `crates/zk-ssl-cli/src/witness.rs:2303-2305 'lo anadido NO esta verificado: el nodo no verifica, solo filtra' (discrepa en matiz con RPC.md, que dice que comprueba la firma)`; `crates/zk-ssl-node/src/main.rs:1691-1742 zkssl_submitCosig: 1) época en curso; 2) 'VERIFICAR, con el mismo verificador que usara el tercero' → zk_ssl_verify::verificar_cofirma; 'la`; `spec/RPC.md:893-900 'Comprueba que la cofirma es de la epoca EN CURSO y que la firma cierra ... No acredita al testigo, y no puede'`; `spec/RPC.md:902-907 una cofirma viva por clave de testigo, solo la época en curso, --max-cofirmas`; `crates/zk-ssl-cli/src/witness.rs:2303 (mensaje inexacto)`; `crates/zk-ssl-node/src/main.rs:1883-1935 (antes 1691-1742)`; `spec/RPC.md:944-976 (961-968 y 970-975)`

### G7b

**FALSA** · gravedad alta

- **Borrador:** '13 de 25 muertes dejan el contador adelantado': es el caso normal tras un reinicio
- **Lo que se puede afirmar:** El 13/25 mide muertes dentro de un proceso vivo (la ventana entre reservar y firmar), no reinicios. Tras un reinicio real con clave derivada de semilla, el estado del índice de firma es siempre ClaveEnCero/KeyAtZero (AUDITORIA §331; HBS-STATE v0.3 §0-§1). Varios comentarios de Arqueo aún dicen lo contrario: main.rs:586-588 y 605, firma_cabeza.rs:251-252, witness.rs:2334-2335 y 2825-2826. ContadorAdelantado sólo es 'normal tras una caída' donde K se persiste, como en el registro de recepción del §569.
- **Fuentes:** `AUDITORIA.md l.16864-16868 (subproducto de K.1: 13/25 = 52 %)`; `AUDITORIA.md §331 l.25552-25560 'la cifra es correcta y la inferencia no ... Tras un REINICIO la clave nace en cero, asi que ContadorAdelantado no ocurre el 52 % sino el 100 %, y n`; `crates/zk-ssl-guardian/src/lib.rs:332-335 'K.1 midió esto DENTRO de un proceso ... no tras un REINICIO'`; `hbs-state/spec/HBS-STATE-v0.3 (md):72-89 (corrección: 'The second sentence is FALSE')`; `AUDITORIA.md l.16864-16868 (13 de 25, 52 %)`; `AUDITORIA.md §331 l.25552-25560 'la cifra es correcta y la inferencia no ... Tras un REINICIO la clave nace en cero, asi que ContadorAdelantado no ocurre el 52 % de las veces sino `; `crates/zk-ssl-guardian/src/lib.rs:332-335`; `hbs-state/spec/HBS-STATE-v0.3 (md):69-86 ('The second sentence is FALSE')`

### G8a

**FALSA** · gravedad media

- **Borrador:** El guardián usa bloqueos de fichero (flock/fcntl) o sincronización propia entre hilos/procesos
- **Lo que se puede afirmar:** No hay bloqueo de fichero: nada impide que dos procesos abran el mismo contador, ni, desde el §569, el mismo registro de recepción. Dentro del proceso, la exclusión viene de la propiedad (&mut self), de un Mutex del llamante (el contador y el registro de recepción) o de una atómica (el índice de la última firma). HBS-STATE deja expresamente fuera los locks y los firmantes múltiples (crates/zk-ssl-guardian/src/lib.rs:410-460; main.rs:430-449; hbs-state/spec/HBS-STATE-v0.3 (md):382-385).
- **Fuentes:** `grep 'flock\|fcntl\|lockfile\|try_lock' en crates/zk-ssl-guardian: 0`; `crates/zk-ssl-guardian/src/lib.rs:411-414, 453: GuardianIndice sin mutabilidad interior; reservar(&mut self) → exclusión solo por préstamo de Rust dentro del proceso`; `crates/zk-ssl-node/src/latido.rs:322-328 el FirmanteCabeza lo posee una sola tarea tokio; main.rs:897-900 el ContadorRecepcion va en std::sync::Mutex`; `hbs-state/spec/HBS-STATE-v0.3 (md):382-385 excluye 'fsync, ordering, locks, copies' y 'Multiple signers over the same public key'`; `grep -E 'flock\|fcntl\|lockfile\|try_lock\|lock_exclusive\|fs2\|fd-lock' en crates/zk-ssl-guardian, zk-ssl-node/src, zk-ssl-cli/src y hbs-state/src: 0`; `crates/zk-ssl-guardian/src/lib.rs:410-414, 453 reservar(&mut self)`; `crates/zk-ssl-node/src/latido.rs:322-329 (el firmante lo posee la tarea); main.rs:897-900 (ContadorRecepcion en std::sync::Mutex)`; `hbs-state/spec/HBS-STATE-v0.3 (md):382-385 excluye 'fsync, ordering, locks, copies' y 'Multiple signers'`

### G8c

**FALSA** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** El guardián hace fsync del directorio
- **Lo que se puede afirmar:** El guardián persiste con sync_all, que hace fsync de los datos y los metadatos del fichero, escribiendo en el sitio 8 bytes little-endian; no sincroniza el directorio. En el nodo, sólo el registro de recepción del RFC-0010 hace fsync del directorio, al crear un fichero de era nuevo en anotar, y ese camino está vivo desde el §569. El de podar existe pero no tiene llamador (§580) (crates/zk-ssl-guardian/src/lib.rs:495-507; registro_recepcion.rs:163-165, 321).
- **Fuentes:** `crates/zk-ssl-guardian/src/lib.rs:495-507 persistir: solo f.sync_all() del fichero; ningún fsync del directorio padre, tampoco al crearlo en abrir (lib.rs:439-443)`; `En cambio registro_recepcion.rs:161-165 y 255-257 sí sincronizan el directorio al crear un fichero de era y al podar ('sin esto el nombre del fichero nuevo puede no sobrevivir a un`; `crates/zk-ssl-guardian/src/lib.rs:495-507 (seek 0, write_all 8 B LE, sync_all); 419-443`; `crates/zk-ssl-node/src/registro_recepcion.rs:161-165 ('if nuevo { File::open(&self.dir)...sync_all() }') y 255-257`; `crates/zk-ssl-guardian/src/lib.rs:419-443, 495-507 (sin cambios)`; `crates/zk-ssl-node/src/registro_recepcion.rs:147-166 ('if nuevo { File::open(&self.dir)...sync_all() }' en 163-165)`; `crates/zk-ssl-node/src/main.rs:2669-2671, 2719-2721, 2956-2966 (anotar, llamado en cada applySend/applyClaim)`; `crates/zk-ssl-node/src/registro_recepcion.rs:284-300 y 321 (podar, 'SIN LLAMADOR, por decisión del autor')`

### G3d

**SIN_FUENTE** · gravedad alta

- **Borrador:** Cifras de fsync '1,82 ms / 0,03 ms'
- **Lo que se puede afirmar:** No existen en ninguna fuente; son inventadas. Deben sustituirse por las medidas de K.1: 0,907 ms (ext4, 382×) y 0,002 ms (tmpfs, 1×), AUDITORIA entrada 234.
- **Fuentes:** `grep de '1[.,]82' y '0[.,]03 ms' en todos los .md/.rs de Arqueo y en todo hbs-state: 0 coincidencias`; `Las únicas cifras del repo son 0,907 ms / 0,002 ms / 382× / 1× (AUDITORIA.md l.16836-16839)`; `grep -E '1[.,]82 ?ms\|0[.,]03 ?ms' en Arqueo y hbs-state: 0 coincidencias`; `AUDITORIA.md l.16836-16839 (únicas cifras: 0,907 ms / 0,002 ms / 382× / 1×)`; `git grep -nE '1[.,]82 ?(ms\|µs)\|0[.,]03 ?(ms\|µs)' HEAD -- '*.md' '*.rs' '*.sh' '*.py' '*.json' ':!doc/ecst' => 0`

### G9b

**SIN_FUENTE** · gravedad media

- **Borrador:** La seguridad frente a pérdidas de energía requiere NVRAM/HSM (según SECURITY.md)
- **Lo que se puede afirmar:** Ningún documento de los repositorios afirma que haga falta NVRAM/HSM. Lo que dicen es que el diseño no garantiza nada frente a un corte de corriente y que eso no se ha medido (guardian lib.rs:65-71; AUDITORIA entrada 234). La recomendación de hardware procede de la literatura externa y debe atribuirse a esas fuentes, no a SECURITY.md: SP 800-208, y RFC 10033 según su cita en HBS-STATE, texto no verificado aquí.
- **Fuentes:** `grep 'corriente\|corte\|fsync\|guardi\|NVRAM' en SECURITY.md: 0 coincidencias; SECURITY.md no habla del guardián ni de cortes de corriente`; `El aviso vive en crates/zk-ssl-guardian/src/lib.rs:65-71 y AUDITORIA.md l.16907-16911: 'Nada frente a un corte de corriente ... Medirlo exige cortar la corriente de verdad, y no se`; `SECURITY.md:158-160 menciona HSM solo a propósito del ancla ('Aunque la clave privada viviera en un HSM ... el testigo seguiría sin poder afirmar de quién es la firma'); main.rs:13`; `Fuera del repo: hbs-state/spec/HBS-STATE-v0.3 (md):12-19 cita SP 800-208 (módulos hardware) y RFC 10033 §4 ('rollback resistant counters ... software-only'); WebSearch encontró la `; `grep -i 'corriente\|corte de\|fsync\|guardi\|NVRAM\|HSM\|energ\|apag' SECURITY.md: solo l.158 (HSM, a propósito del ancla)`; `grep -rn 'NVRAM' en .md/.rs de Arqueo: 0`; `crates/zk-ssl-guardian/src/lib.rs:65-71 y AUDITORIA.md l.16907-16911 'Nada frente a un corte de corriente ... no se ha hecho'`; `hbs-state/README.md:16-26 y spec:12-19 (citan SP 800-208 y RFC 10033 §4, hardware dedicado)`

### G10a

**PARCIAL** · gravedad media

- **Borrador:** El nodo firma la cabeza con XMSS: firma post-cuántica
- **Lo que se puede afirmar:** Cuando el operador le entrega una semilla, el nodo firma la cabeza de época con XMSS^MT (SHA2-40/8-256), un esquema basado en hash y con estado, elegido por no añadir supuestos distintos de los del STARK; desde el §570 la cabeza firmada es v6. Por defecto no firma, no hay clave de operador real y la implementación es una pre-release sin auditar (AUDITORIA §106; latido.rs:17-35; SECURITY.md:145-148).
- **Fuentes:** `AUDITORIA.md §106 (l.8876-8930, 01-08-2026): XMSS elegido porque 'no añade una familia de supuestos nueva' (resistencia de la función hash)`; `crates/zk-ssl-node/src/latido.rs:17-35: sin --clave no firma; SECURITY.md:145-148 no existe clave de operador real`; `crates/zk-ssl-verify/src/lib.rs:53-58 'Esto hace posible verificar; no hace válido lo verificado'`; `AUDITORIA.md §106 l.8876-8925 ('Tomada el 01-08-2026'; 'no añade una familia de supuestos nueva'; XMSS = 'resistencia de la funcion hash — los mismos que ya sostienen los STARK')`; `crates/zk-ssl-node/src/latido.rs:17-35; SECURITY.md:145-148`; `crates/zk-ssl-verify/src/lib.rs:53-58`; `crates/zk-ssl-node/src/firma_cabeza.rs:40 'xmss es 0.1.0-pre.0, sin auditoría independiente'`; `AUDITORIA.md §106 l.8876-8925`

### G10c

**PARCIAL** · gravedad alta

- **Borrador:** El defecto del OID está 'reportado en RustCrypto/signatures#1442' (implícitamente, por el autor) / el issue de Arqueo fue enviado
- **Lo que se puede afirmar:** El defecto de OID existe en xmss 0.1.0-pre.0 y afecta a 21 de los 56 conjuntos XMSS^MT: todos los de OID bruto 1..21, incluidos los ocho SHA2-256. El 'cinco de ocho' del borrador de Arqueo no casa con el fuente. El issue público #1442 lo abrió la cuenta joshhh7 el 2026-09-10, y nada en los repos acredita que sea del autor. El issue propio de Arqueo (index()/remaining()/BDS/Clone) sigue siendo un borrador sin constancia de envío. Un informe no debe atribuir #1442 al autor.
- **Fuentes:** `crates/zk-ssl-guardian/src/lib.rs:472-473 'El issue upstream que pide index() está redactado en doc/issue-rustcrypto.md y sin enviar'; BACKLOG.md:3072-3075 'No consta que se haya p`; `doc/issue-rustcrypto.md:1 'Issue draft', pide index()/remaining(), BDS, aviso de Clone y el OID ('Five of the eight SHA2-256 XMSS^MT sets collide')`; `hbs-state/README.md:156-160 'Reported in RustCrypto/signatures#1442'; hbs-state/spec/HBS-STATE-v0.3 (md):405-406 'Published as RustCrypto/signatures issue 1442', '21 of the 56'`; `WebFetch https://github.com/RustCrypto/signatures/issues/1442 (2026-09-28): 'XMSSMT OID parsing bug', abierto el 2026-09-10 por joshhh7, abierto, sin comentarios; no menciona index`; `xmss-0.1.0-pre.0/src/params.rs:1206-1226 acepta OID de árbol único 0x01..0x15 (21) y :1228-1284 OID MT 0x01..0x38 (56) → 21/56 MT mal parseados, incluidos los 8 SHA2-256 (lectura d`; `WebFetch https://github.com/RustCrypto/signatures/issues/1442 (2026-09-28): título 'XMSSMT OID parsing bug', autor joshhh7, abierto el 10-09-2026, abierto, 0 comentarios, sin '21 o`; `/root/.cargo/registry/src/index.crates.io-*/xmss-0.1.0-pre.0/src/params.rs:1200-1285 (try_from: árbol único 0x01..0x15; MT 0x01..0x38)`; `doc/issue-rustcrypto.md:1 'Issue draft', :59 'Five of the eight SHA2-256 XMSS^MT sets collide'`

### G1e

**PARCIAL** · gravedad media

- **Borrador:** El guardián de Arqueo es 'derivado de semilla': el SK se regenera de la semilla y vuelve al índice 0 tras reiniciar
- **Lo que se puede afirmar:** El guardián en sí es solo un contador de 8 bytes y no conoce xmss; el lector de la semilla (semilla.rs, 96 B, comprobación de permisos) sí vive en su crate. Lo derivado de semilla son sus consumidores firmantes (FirmanteCabeza del nodo y Cofirmante del testigo), que reconstruyen el par con KeyPair::from_seed en cada arranque y no persisten el SK: tras un reinicio real la clave está en el índice 0 y la reconciliación da ClaveEnCero (firma_cabeza.rs:164-176; witness.rs:2798-2808; AUDITORIA §335).
- **Fuentes:** `crates/zk-ssl-node/src/firma_cabeza.rs:164-176 desde_semilla: GuardianIndice::abrir + KeyPair::<Conjunto>::from_seed(semilla)`; `crates/zk-ssl-guardian/src/lib.rs:340-344 "El SK no se persiste: al rearrancar, from_seed la devuelve en CERO"`; `crates/zk-ssl-guardian/src/semilla.rs:19-25 SEMILLA_LEN = 96 = SK_SEED‖SK_PRF‖PUB_SEED`; `AUDITORIA.md §335 (l.25981-25984) "El SK no se persiste: al reiniciar, la clave se rederiva de la semilla y su indice vuelve a cero mientras el contador del guardian sobrevive"`; `crates/zk-ssl-guardian/Cargo.toml: [dependencies] vacío (el guardián no conoce xmss)`; `crates/zk-ssl-cli/src/witness.rs:2797-2807 Cofirmante::desde_semilla (mismo molde)`; `crates/zk-ssl-guardian/src/lib.rs:340-344 'El SK no se persiste: al rearrancar, from_seed la devuelve en CERO'`; `AUDITORIA.md §335 (l.25981-25984) 'El SK no se persiste: al reiniciar, la clave se rederiva de la semilla y su indice vuelve a cero mientras el contador del guardian sobrevive'`

### G2b

**PARCIAL** · gravedad media

- **Borrador:** (implícito) La pieza tiene consumidores/uso real según su propia documentación
- **Lo que se puede afirmar:** Dentro de Arqueo el guardián sí tiene consumidores de código (firmante del nodo, contador de recepción —que desde el §569 se reconcilia además con el registro de recepción al arrancar—, cofirmante del testigo y regla de permisos del SDK), pero ninguno opera en producción. La cabecera de zk-ssl-guardian/src/lib.rs:73-79 ('no tiene consumidor todavía') está rancia y la de hbs-state/src/lib.rs:71-75 se contradice a sí misma.
- **Fuentes:** `crates/zk-ssl-guardian/src/lib.rs:73-79 "Y esta pieza NO tiene consumidor todavía ... el 3 —la cabeza firmada, emitida— no existe" (rancio: consumidores en firma_cabeza.rs:50-52, r`; `hbs-state/src/lib.rs:71-75: bajo el título 'WARNING: and this piece has no consumer yet' pone 'extracted from a production system where it has been signing for a year' (contradicci`; `crates/zk-ssl-guardian/src/lib.rs:73-79 'Y esta pieza NO tiene consumidor todavía'`; `crates/zk-ssl-node/src/firma_cabeza.rs:50-52; crates/zk-ssl-node/src/recepcion.rs:76,85-95; crates/zk-ssl-cli/src/witness.rs:2784-2806; crates/zk-ssl-sdk/src/keystore.rs:27`; `hbs-state/src/lib.rs:71-75`; `crates/zk-ssl-guardian/src/lib.rs:73-79 (sin cambios)`; `crates/zk-ssl-node/src/firma_cabeza.rs:50-52; crates/zk-ssl-node/src/recepcion.rs:76-95 (sin cambios)`; `crates/zk-ssl-cli/src/witness.rs:2785-2808 (antes 2784-2806)`

### G3a

**PARCIAL** · gravedad media

- **Borrador:** Autocomprobación de fsync: umbral 10× frente a no persistir; se niega a operar donde fsync no persiste
- **Lo que se puede afirmar:** El guardián solo se niega a arrancar si fsync cuesta a la vez menos de 10× lo que una escritura sin fsync y menos de 20 µs de media (20 muestras). Detecta, por tanto, el caso 'no hay disco' (tmpfs), no cualquier fsync falso: uno que mienta pero cueste ≥20 µs pasa. Los umbrales son declarados y se midieron en una sola máquina, WSL2 sobre un i5-1135G7 (crates/zk-ssl-guardian/src/lib.rs:60-63, 95-107, 510-543).
- **Fuentes:** `crates/zk-ssl-guardian/src/lib.rs:95-107 MUESTRAS_AUTOCOMPROBACION=20, RAZON_MINIMA=10.0, SUELO_MICROS=20.0`; `lib.rs:510-543: mide media por escritura de 20× (seek(0)+write_all de 8 B) sin sync_all y luego con sync_all en el directorio del contador; rechaza si 'razon < RAZON_MINIMA && con `; `lib.rs:60-63 'Los umbrales salen de UNA máquina —WSL2 sobre un i5-1135G7— y están declarados, no derivados'`; `lib.rs:731-779 test en_tmpfs_se_niega_a_operar: si no encuentra tmpfs, imprime AVISO y pasa ('se salta EN VOZ ALTA')`; `crates/zk-ssl-guardian/src/lib.rs:95-107`; `crates/zk-ssl-guardian/src/lib.rs:510-543, en especial 534-535 'if razon < RAZON_MINIMA && con < SUELO_MICROS'`; `crates/zk-ssl-guardian/src/lib.rs:60-63 'Los umbrales salen de UNA máquina —WSL2 sobre un i5-1135G7— y están declarados, no derivados'`; `crates/zk-ssl-guardian/src/lib.rs:731-779 en_tmpfs_se_niega_a_operar (si no hay tmpfs imprime AVISO y pasa)`

### G4d

**PARCIAL** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** El 'latido' firma la cabeza de cada época
- **Lo que se puede afirmar:** El latido es una tarea periódica (60 s por defecto, §121) que compone la EpochHead —v6 desde el §570, con la pareja (recepRoot, recepCount) del registro de recepción calculada bajo el candado del estado— y sólo la firma si el operador arrancó con --clave/--clave-fichero (y --diario); la firma se hace fuera del candado y, tras firmar, el latido publica el índice XMSS de la firma, que fija la era de los recibos de recepción (§567, §569). Sin clave calcula y sirve la cabeza sin firma, el índice se queda en 0 y la ventana de completitud del RFC-0010 no corre. La firma 'a demanda' mencionada en §121 no está implementada (latido.rs:8, 17-35, 165-212, 389-419).
- **Fuentes:** `crates/zk-ssl-node/src/latido.rs:17-35 'El nodo no firma por defecto. Sin --clave, el latido sigue corriendo y anotando la cabeza; lo que falta es la firma'`; `latido.rs:64 LATIDO_POR_DEFECTO_S = 60; main.rs:156-161 '--latido' (0 lo apaga)`; `main.rs:791-801 firmar exige --diario ('quien firma, anota')`; `latido.rs:136-205: compone la cabeza bajo el Mutex del estado y firma FUERA del candado (144,5 ms); latido.rs:99-131 M.1: p99 igual con y sin firma`; `latido.rs:322-328 el firmante lo posee solo la tarea del latido; 'a demanda' (latido.rs:8) no tiene camino de código`; `crates/zk-ssl-node/src/latido.rs:17-35 'El nodo no firma por defecto'`; `crates/zk-ssl-node/src/latido.rs:65 LATIDO_POR_DEFECTO_S = 60; main.rs:156-161 ('0 apaga el latido')`; `crates/zk-ssl-node/src/main.rs:798-803 'quien firma, anota: --clave/--clave-fichero exige --diario'`

### G5a

**PARCIAL** · gravedad media

- **Borrador:** Testigos (independientes) cofirman la cabeza y fijan la clave la primera vez que la ven
- **Lo que se puede afirmar:** El testigo de referencia fija por TOFU la clave pública que ve por primera vez y se detiene si cambia o si ve dos digests con el mismo índice; opcionalmente (--cofirmar) cofirma con su propia clave XMSS y su propio guardián bajo el dominio 'ZK-SSL-witness-cosign', y desde el §570 lo hace sobre cabezas v6. TOFU no protege el primer encuentro, el ancla vive en la memoria del proceso testigo, y hoy no hay testigos independientes: sólo la implementación de referencia que corre el propio autor (witness.rs:1-53, 631-694, 2319-2367; SECURITY.md:59, 122-143). La frase del README:112-114 ('testigos independientes') no debe repetirse sin esa salvedad.
- **Fuentes:** `README.md:112-114 (texto literal de la afirmación)`; `crates/zk-ssl-cli/src/witness.rs:1-35 TOFU ('Un testigo que anota la clave que vio la primera vez y se detiene si cambia'); witness.rs:676-693 Memoria::anclar; se detiene ante vist`; `witness.rs:630-652 el ancla vive en Memoria (RAM); witness.rs:2310 Memoria::nueva() al arrancar run (no encontré recarga del ancla desde el diario; el auditor offline sí detecta Ca`; `witness.rs:2318-2360 cofirmar es un modo opcional (--cofirmar) con clave XMSS y guardián propios`; `SECURITY.md:59 'hoy nadie fuera del operador observa cabezas'; SECURITY.md:134-143 'TOFU fija la mentira' y 'un testigo que opera el propio operador no prueba nada'`; `README.md:112-114`; `crates/zk-ssl-cli/src/witness.rs:1-53 (TOFU; 'Un testigo que opera el propio operador NO PRUEBA NADA'; vista dividida y cambio de clave detienen)`; `crates/zk-ssl-cli/src/witness.rs:630-693 Memoria{clave_fijada} y anclar; 2310 'let mut m = Memoria::nueva()'`

### G6a

**PARCIAL** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** RFC-0010 / registro_recepcion.rs usa 'CounterAhead' y cita HBS-STATE
- **Lo que se puede afirmar:** El texto del RFC-0010 —ACEPTADO desde el §577— no cita HBS-STATE ni usa CounterAhead. Menciona la reconciliación del registro con el contador sólo en prosa (l.195-198). Las decisiones D-I, D-J y D-K que el código invoca para el registro siguen sin estar escritas en el RFC. Lo citan los comentarios de registro_recepcion.rs:54-78 y recepcion.rs:216-219, que usan la variante castellana ContadorAdelantado de zk-ssl-guardian y citan literalmente el vector A4 'CounterAhead' del spec. La etiqueta 'M5 de HBS-STATE' está corregida desde el §565-B.
- **Fuentes:** `crates/zk-ssl-node/src/registro_recepcion.rs:54-78 usa los nombres castellanos del guardián (ClaveAdelantada, ContadorAdelantado); 'CounterAhead' solo aparece citando el vector A4 `; `registro_recepcion.rs:61-67 y recepcion.rs:216-219: corrección §565-B, la etiqueta 'M5 de HBS-STATE' estaba colgada de la rama equivocada`; `spec/rfc/0010-el-recibo-de-recepcion.md: grep de 'HBS-STATE\|CounterAhead\|ContadorAdelantado' = 0; solo cita §234 (l.14) y §253 (l.243)`; `spec/rfc/0010-el-recibo-de-recepcion.md:3 'Estado: PROPUESTO'; define D-A..D-H, mientras registro_recepcion.rs cita D-I, D-J, D-K que el RFC no define`; `spec/rfc/0010-el-recibo-de-recepcion.md:3 'Estado: PROPUESTO'; grep 'HBS-STATE\|CounterAhead\|ContadorAdelantado' = 0; decisiones D-A..D-H únicamente`; `AUDITORIA.md:40424 'D-I..D-M ya tomadas y sin escribir en el RFC'`; `crates/zk-ssl-node/src/registro_recepcion.rs:10,21,37 (citan D-I, D-K, D-J) y 54-78 (ClaveAdelantada, ContadorAdelantado; 'CounterAhead' solo al citar el vector A4)`; `crates/zk-ssl-node/src/registro_recepcion.rs:61-67 y recepcion.rs:216-219 (corrección §565-B de la etiqueta M5)`

### G6d

**PARCIAL** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** (ECST) R(C,K) se generaliza a otros recursos monótonos
- **Lo que se puede afirmar:** El repositorio muestra un segundo uso real, en producción de código desde el §569, de la misma función R(C,K): contador de recepción frente al mayor rx anotado, con cuatro estados de significado propio y un único fatal (el registro por delante del contador). Eso sostiene la generalización como función de CLASIFICACIÓN sobre dos contadores monótonos, uno de los cuales debe ir detrás del otro. No la sostiene como garantía. La política es de cada dominio y difiere de la del firmante en ClaveEnCero. El significado de cada estado depende del modelo: el registro es de clave persistida, y ahí KeyAtZero es 'registro perdido o nuevo'. El par no distingue restaurar el registro de un hueco legítimo, y el propio Arqueo tuvo que añadir un dato fuera del par para una parte de ese hueco, el recepCount de la cabeza anterior. Las fuentes son recepcion.rs:112-128; main.rs:654-693, 1039-1061; registro_recepcion.rs:69-84; latido.rs:272-296; AUDITORIA §565, §567 y §569.
- **Fuentes:** `recepcion.rs:115-125: Arqueo reutiliza la misma Reconciliacion para un recurso que no es un índice HBS`; `AUDITORIA.md §565 (l.41033-41040): 'Reusar una abstraccion da la TRADUCCION y no da la COBERTURA'`; `registro_recepcion.rs:69-78: restaurar el REGISTRO viejo cae en ContadorAdelantado, indistinguible de un hueco legítimo; 'La D-J cubre media restauración'; la salida 'pide un DATO `; `hbs-state/spec/HBS-STATE-v0.3 (md):97-133: el significado de KeyAtZero depende del modelo (SK persistido vs. derivado de semilla)`; `crates/zk-ssl-node/src/recepcion.rs:112-128`; `AUDITORIA.md §565 l.41042-41043 'Reusar una abstraccion da la TRADUCCION y no da la COBERTURA'`; `crates/zk-ssl-node/src/registro_recepcion.rs:69-78 ('indistinguible de un hueco legítimo por caída ... pide un DATO MÁS ALLÁ DEL PAR ... La D-J cubre media restauración')`; `hbs-state/spec/HBS-STATE-v0.3 (md):93-132 (el significado de KeyAtZero depende del modelo del SK)`

### G7a

**PARCIAL** · gravedad media

- **Borrador:** Prueba de muerte del proceso '25 de 25 sin una sola firma por delante'
- **Lo que se puede afirmar:** La cifra 25/25 está registrada en AUDITORIA (entrada 234, banco K.1): un hijo que persiste y luego firma, un padre que lo mata con kill -9 en un instante aleatorio, y el invariante 'ninguna firma con índice mayor que el contador persistido'. El código del banco no está en el repositorio y en ese momento xmss aún no era dependencia, así que no es reproducible desde el árbol. Mide la muerte del proceso, no un corte de corriente ni un reinicio. Con n=25 y 0 fallos, la cota superior exacta al 95 % de la tasa de fallo es ≈11 % (cálculo propio: 1−0,05^(1/25)).
- **Fuentes:** `AUDITORIA.md '## 234.' l.16856-16862 '### El invariante, y que se sostiene bajo kill -9 ... K.1 lo probó con un hijo que persiste-y-luego-firma y un padre que lo mata en un instant`; `crates/zk-ssl-guardian/src/lib.rs:67-71 y hbs-state/README.md:96-100 repiten la cifra`; `grep 'kill\|SIGKILL\|libc::' en crates/zk-ssl-guardian, zk-ssl-node/src, zk-ssl-cli/src: 0; ningún script K.1 en tools/: el código del banco no está en el árbol`; `AUDITORIA.md l.16914-16916: en la entrada 234 'xmss sigue sin ser dependencia. El guardián no firma: cuenta' (qué 'firmaba' el hijo no consta)`; `AUDITORIA.md l.16856-16862 '### El invariante, y que se sostiene bajo kill -9 ... 25 de 25 sin una sola firma por delante'`; `crates/zk-ssl-guardian/src/lib.rs:67-71; hbs-state/README.md:96-100`; `grep 'K\.1' fuera de AUDITORIA: solo comentarios; grep 'kill -9' en tools/: banco_apagado, banco_pago, etc., ninguno de K.1`; `AUDITORIA.md l.16914-16916 'xmss sigue sin ser dependencia. El guardián no firma: cuenta.'`

### G8b

**PARCIAL** · gravedad media

- **Borrador:** El guardián comprueba permisos 0600 al leer
- **Lo que se puede afirmar:** La comprobación al leer se aplica al fichero de la SEMILLA: sin bits de grupo ni de otros, es decir 0600 o más estricto, y sólo en Unix. No se aplica al contador de índices, que se crea con el umask por defecto y no se comprueba, ni a los ficheros del registro de recepción (semilla.rs:27-52; lib.rs:426-437, 495-507; main.rs:1394-1398; registro_recepcion.rs:147-166).
- **Fuentes:** `crates/zk-ssl-guardian/src/semilla.rs:27-52 comprobar_permisos: error PermisosAbiertos si mode & 0o077 != 0; solo #[cfg(unix)], no-op en otros SO`; `semilla.rs:85-105 leer_hex/leer_cruda la llaman antes de leer la SEMILLA; main.rs:1214-1218 el nodo la aplica a --clave-fichero`; `crates/zk-ssl-guardian/src/lib.rs:426-437 y 495-507: el fichero del CONTADOR se abre y crea sin modo explícito ni comprobación de permisos`; `crates/zk-ssl-guardian/src/semilla.rs:27-52 comprobar_permisos (mode & 0o077 != 0 → PermisosAbiertos; #[cfg(unix)])`; `crates/zk-ssl-guardian/src/semilla.rs:85-105 leer_hex/leer_cruda`; `crates/zk-ssl-node/src/main.rs:1214-1218; crates/zk-ssl-cli/src/witness.rs:2325`; `crates/zk-ssl-guardian/src/lib.rs:426-437 y 495-507 (el contador se abre y crea sin modo explícito ni comprobación)`; `crates/zk-ssl-guardian/src/semilla.rs:27-52, 85-105 (sin cambios)`

### G9a

**PARCIAL** · gravedad media

- **Borrador:** HBS-STATE mitiga los falsos fsync en entornos virtuales o volátiles
- **Lo que se puede afirmar:** La autocomprobación es de la implementación de referencia, no de la especificación: esta deja fuera la persistencia y trata la autocomprobación como candidata sin vectores. Detecta almacenamiento donde fsync cuesta lo mismo que no hacerlo, como tmpfs. No detecta discos o capas virtuales que confirman escrituras todavía en caché volátil, ni cualquier fsync falso que cueste ≥20 µs. Solo se midió en WSL2 (crates/zk-ssl-guardian/src/lib.rs:49-71, 534-541; HBS-STATE:344-360, 382).
- **Fuentes:** `crates/zk-ssl-guardian/src/lib.rs:534-541: rechazo solo si razón<10 Y con_fsync<20 µs`; `lib.rs:49-58 'En tmpfs, fsync devuelve éxito sin persistir nada ... la única señal disponible desde dentro del proceso'`; `lib.rs:65-71 '«fsync puede mentir» habla de discos que confirman escrituras que siguen en caché volátil ... eso no es lo mismo'`; `hbs-state/spec/HBS-STATE-v0.3 (md):350-356: la autocomprobación es fila candidata de la 'Family C' (sin vectores); spec:382 no cubre cómo se persiste el contador`; `hbs-state/spec/HBS-STATE-v0.3 (md):446-450: la durabilidad de hbs-lms se delega al persistidor, 'That gap is the one the fsync self-check of this bench exists to close'`; `crates/zk-ssl-guardian/src/lib.rs:49-58, 65-71, 534-541`; `hbs-state/spec/HBS-STATE-v0.3 (md):344-360 (Family C, CANDIDATE, sin vectores); :382 (fuera de alcance: cómo se persiste el contador)`; `hbs-state/spec/HBS-STATE-v0.3 (md):446-450`

### G10b

**CONFIRMADA** · gravedad media

- **Borrador:** Salvedades declaradas sobre la dependencia xmss
- **Lo que se puede afirmar:** Salvedades que el repo declara sobre xmss 0.1.0-pre.0. Es una pre-release sin auditar, fijada con '='. No expone el índice, que se lee del layout del SK de 137 B. SigningKey es Clone, lo que hace posible una reutilización silenciosa. Un defecto de parseo del OID obliga a un apaño en la lectura y en la resincronización, vigilado por un test centinela. No hay KAT en el proyecto. from_seed deja una copia de la clave sin borrar. Fuentes: Cargo.toml del nodo; zk-ssl-verify/src/lib.rs:541-557, 922-938; AUDITORIA §335; BACKLOG 77.
- **Fuentes:** `crates/zk-ssl-node/Cargo.toml:41-45: pre-release, sin auditoría independiente, master diverge del tag (sha3→shake)`; `crates/zk-ssl-verify/src/lib.rs:511-538 OFFSET_MT_UPSTREAM = 0x0001_0000 (apaño del OID) y centinela el_apano_del_oid_sigue_haciendo_falta (l.918-934)`; `AUDITORIA.md §335 (l.26016-26022): SigningKey::try_from rechaza los bytes de from_seed con InvalidOid(5); resincronizar exige el apaño (firma_cabeza.rs:228-233)`; `~/.cargo/registry/.../xmss-0.1.0-pre.0/src/xmss.rs:21 'XmssOid::try_from(raw_oid).or_else(...)'; xmss.rs:32 #[derive(Clone)] en SigningKey; sin index()/remaining() (grep 0)`; `BACKLOG.md:3065-3071 KAT ausente en el proyecto; doc/CONFIANZA_RESIDUAL.md:58-60 from_seed deja una copia sin borrar (backlog 102)`; `crates/zk-ssl-node/Cargo.toml:41-45`; `crates/zk-ssl-verify/src/lib.rs:538 OFFSET_MT_UPSTREAM = 0x0001_0000; :554 aplicar_apano_del_oid; :919-935 centinela el_apano_del_oid_sigue_haciendo_falta`; `AUDITORIA.md §335 l.26016-26022 (SigningKey::try_from rechaza con InvalidOid(5))`

### G1a

**CONFIRMADA** · gravedad baja

- **Borrador:** hbs-state fue extraído directamente de ARQUEO (el origen es zk-ssl-guardian)
- **Lo que se puede afirmar:** hbs-state es una traducción al inglés, casi literal, del crate zk-ssl-guardian de Arqueo (lib.rs y semilla.rs→seed.rs). Añade tres cosas propias: la función pura reconcile_values con su test de atadura, el binario sujeto del banco y tests/vectors.rs. Fuentes: hbs-state/README.md:169-171 y la comparación de ambos src/lib.rs. La etiqueta 'production implementation' que HBS-STATE:23 da al origen no se sostiene (G2a).
- **Fuentes:** `hbs-state/README.md:169-171 "Extracted from ARQUEO ... This crate is that piece, relabelled to the domain and with no dependencies"`; `hbs-state/spec/HBS-STATE-v0.3 (md):23-24 "(the ARQUEO index guard, XMSSMT-SHA2_40/8_256)"`; `crates/zk-ssl-guardian/src/lib.rs frente a hbs-state/src/lib.rs: traducción línea a línea (mismas constantes lib.rs:87-107 vs hbs-state/src/lib.rs:88-108; mismo abrir/reservar/pers`; `crates/zk-ssl-guardian/src/semilla.rs (247 líneas) vs hbs-state/src/seed.rs (248): mismo lector de semilla`; `Añadidos solo en hbs-state: reconcile_values (hbs-state/src/lib.rs:405-431) + test the_method_and_the_free_function_agree (801-815), src/bin/subject.rs, tests/vectors.rs`; `El repo Arqueo local es un clon superficial (git rev-parse --is-shallow-repository = true, 50 commits): el commit de extracción no es visible`; `hbs-state/README.md:169-171 'Extracted from [ARQUEO] ... This crate is that piece, relabelled to the domain and with no dependencies'`; `crates/zk-ssl-guardian/src/lib.rs:128,154,369,419,453,474,495,510 frente a hbs-state/src/lib.rs:128,154,370,448,482,502,506,523 (misma estructura, traducida)`

### G1b

**CONFIRMADA** · gravedad baja

- **Borrador:** Mismos cuatro estados en Arqueo y en hbs-state
- **Lo que se puede afirmar:** Los cuatro estados, sus campos derivados y la precedencia son idénticos en ambos crates: Coincide/InSync, ContadorAdelantado/CounterAhead, ClaveEnCero/KeyAtZero (que gana sobre ContadorAdelantado cuando la clave vale 0) y ClaveAdelantada/KeyAhead. El juez no_admite_matiz (Arqueo) equivale a is_fatal (hbs-state) y solo marca ClaveAdelantada/KeyAhead (crates/zk-ssl-guardian/src/lib.rs:324-376).
- **Fuentes:** `crates/zk-ssl-guardian/src/lib.rs:325-353 enum Reconciliacion {Coincide{indice}, ContadorAdelantado{contador,clave,huerfanos}, ClaveEnCero{contador,indeterminados}, ClaveAdelantada`; `crates/zk-ssl-guardian/src/lib.rs:474-493 reconciliar: match cmp con guarda 'Greater if indice_de_la_clave == 0' antes del Greater general`; `hbs-state/src/lib.rs:323-354 y 412-431 (InSync, CounterAhead, KeyAtZero, KeyAhead; misma precedencia)`; `crates/zk-ssl-guardian/src/lib.rs:369-376 no_admite_matiz (solo ClaveAdelantada = true) ≡ hbs-state/src/lib.rs:370-377 is_fatal`; `crates/zk-ssl-guardian/src/lib.rs:324-353 enum Reconciliacion {Coincide{indice}, ContadorAdelantado{contador,clave,huerfanos}, ClaveEnCero{contador,indeterminados}, ClaveAdelantada`; `crates/zk-ssl-guardian/src/lib.rs:474-493 'Greater if indice_de_la_clave == 0 => ClaveEnCero' antes de 'Greater => ContadorAdelantado'`; `hbs-state/src/lib.rs:322-354 y 412-431 (misma precedencia)`; `crates/zk-ssl-guardian/src/lib.rs:369-376 frente a hbs-state/src/lib.rs:370-377`

### G1c

**CONFIRMADA** · gravedad baja

- **Borrador:** Los nombres son los mismos (en Arqueo, en castellano)
- **Lo que se puede afirmar:** En Arqueo el tipo es GuardianIndice (abrir/reservar/actual/reconciliar), el enum es Reconciliacion {Coincide, ContadorAdelantado, ClaveEnCero, ClaveAdelantada}, el juez es no_admite_matiz y el error de persistencia es PersistenciaFalsa. hbs-state los renombra IndexGuard (open/reserve/current/reconcile), Reconciliation {InSync, CounterAhead, KeyAtZero, KeyAhead}, is_fatal y FakePersistence.
- **Fuentes:** `crates/zk-ssl-guardian/src/lib.rs:411-414 GuardianIndice{ruta,actual}; métodos abrir/reservar/actual/reconciliar (419-493)`; `crates/zk-ssl-guardian/src/lib.rs:222-251 GuardianError::{Io, IndiceFueraDeCampo, Corrupto, PersistenciaFalsa{con_fsync_us,sin_fsync_us,razon}, PermisosAbiertos, SemillaLongitud, S`; `hbs-state/src/lib.rs:223-252 GuardError::{..., FakePersistence{with_fsync_us,without_fsync_us,ratio}, PermissionsTooOpen, ...}`; `crates/zk-ssl-guardian/src/lib.rs:410-414 struct GuardianIndice{ruta,actual}; abrir:419, reservar:453, actual:463, reconciliar:474`; `crates/zk-ssl-guardian/src/lib.rs:221-251 GuardianError::{Io, IndiceFueraDeCampo, Corrupto, PersistenciaFalsa{con_fsync_us,sin_fsync_us,razon}, PermisosAbiertos, SemillaLongitud, S`; `hbs-state/src/lib.rs:222-252 GuardError::{..., FakePersistence{with_fsync_us,without_fsync_us,ratio}, PermissionsTooOpen, ...}; IndexGuard{path,current} :439-443`

### G1d

**CONFIRMADA** · gravedad baja

- **Borrador:** Misma autocomprobación de fsync en ambos
- **Lo que se puede afirmar:** El algoritmo de autocomprobación es el mismo en ambos crates: 20 escrituras de 8 B sin sync_all y 20 con sync_all en el directorio del contador, con rechazo si la razón es menor que 10× y a la vez fsync cuesta menos de 20 µs de media. Solo cambia el nombre del fichero sonda (crates/zk-ssl-guardian/src/lib.rs:95-107, 510-543).
- **Fuentes:** `crates/zk-ssl-guardian/src/lib.rs:510-543 comprobar_persistencia (fichero sonda '.guardian-autocomprobacion')`; `hbs-state/src/lib.rs:523-556 check_persistence (fichero sonda '.hbs-state-selfcheck')`; `Constantes idénticas: MUESTRAS_AUTOCOMPROBACION=20 / SELFCHECK_SAMPLES=20; RAZON_MINIMA=10.0 / MIN_RATIO=10.0; SUELO_MICROS=20.0 / FLOOR_MICROS=20.0`; `crates/zk-ssl-guardian/src/lib.rs:95-107 MUESTRAS_AUTOCOMPROBACION=20, RAZON_MINIMA=10.0, SUELO_MICROS=20.0`; `hbs-state/src/lib.rs:96-108 SELFCHECK_SAMPLES=20, MIN_RATIO=10.0, FLOOR_MICROS=20.0`; `crates/zk-ssl-guardian/src/lib.rs:510-543 ('.guardian-autocomprobacion') frente a hbs-state/src/lib.rs:523-556 ('.hbs-state-selfcheck')`

### G1g

**CONFIRMADA** · gravedad baja · cambió con el árbol (S566–S582)

- **Borrador:** Política de arranque en los demás estados
- **Lo que se puede afirmar:** En Arqueo, para el índice de firma (nodo y testigo): Coincide arranca; ContadorAdelantado arranca con aviso; ClaveAdelantada no arranca (clave comprometida); ClaveEnCero resincroniza salvo contradicción del diario o de las cofirmas (main.rs:592-652; witness.rs:2696-2772). Para el registro de recepción del RFC-0010 (§569): Coincide arranca; ContadorAdelantado (huecos) y ClaveEnCero (registro vacío) arrancan avisando; ClaveAdelantada no arranca; nunca resincroniza (main.rs:654-693, 1039-1061). no_admite_matiz no la llama producción: decide el match de cada política (crates/zk-ssl-guardian/src/lib.rs:363-369).
- **Fuentes:** `crates/zk-ssl-node/src/main.rs:564-568 Coincide → Arranca`; `main.rs:569-574 ContadorAdelantado → ArrancaAvisando ('Es el caso NORMAL tras una caida')`; `main.rs:602-609 ClaveAdelantada → NoArranca ('LA CLAVE DEBE CONSIDERARSE COMPROMETIDA')`; `crates/zk-ssl-cli/src/witness.rs:2699-2713 y 2761-2774 (misma política en el testigo)`; `crates/zk-ssl-guardian/src/lib.rs:363-368 'La producción no lo llama' (no_admite_matiz solo lo consumen tests)`; `crates/zk-ssl-node/src/main.rs:565-569 Coincide → Arranca; 570-575 ContadorAdelantado → ArrancaAvisando ('Es el caso NORMAL tras una caida'); 612-618 ClaveAdelantada → NoArranca ('`; `crates/zk-ssl-cli/src/witness.rs:2700-2712 y 2757-2768 (misma política)`; `crates/zk-ssl-guardian/src/lib.rs:363-368 'La producción no lo llama, y es a propósito'`

### G3b

**CONFIRMADA** · gravedad baja

- **Borrador:** Tipo de error de la autocomprobación
- **Lo que se puede afirmar:** El error es GuardianError::PersistenciaFalsa{con_fsync_us, sin_fsync_us, razon} en Arqueo y FakePersistence{with_fsync_us, without_fsync_us, ratio} en hbs-state. Lo devuelve GuardianIndice::abrir antes de leer el contador (crates/zk-ssl-guardian/src/lib.rs:232-234, 419-426).
- **Fuentes:** `crates/zk-ssl-guardian/src/lib.rs:232-234 PersistenciaFalsa { con_fsync_us: f64, sin_fsync_us: f64, razon: f64 }`; `lib.rs:267-274 mensaje '... el nodo NO arranca así'`; `hbs-state/src/lib.rs:237 FakePersistence { with_fsync_us, without_fsync_us, ratio }`; `crates/zk-ssl-guardian/src/lib.rs:267-274 '... el nodo NO arranca así'`; `crates/zk-ssl-guardian/src/lib.rs:419-426`

### G3c

**CONFIRMADA** · gravedad baja

- **Borrador:** ext4: fsync 0,907 ms, 382× frente a no persistir; tmpfs: 0,002 ms, 1×
- **Lo que se puede afirmar:** Medido una vez (banco K.1, AUDITORIA entrada 234) en una sola máquina (WSL2 sobre i5-1135G7): fsync en $HOME/ext4 = 0,907 ms (382× una escritura sin fsync) y en /tmp/tmpfs = 0,002 ms (1×). Es una medida de una máquina, sin n ni dispersión publicados.
- **Fuentes:** `AUDITORIA.md '## 234.' l.16831-16841, banco K.1: '$HOME (ext4) 0,907 ms 382× · /tmp (tmpfs) 0,002 ms 1×'`; `AUDITORIA.md l.16912-16915: 'Los umbrales salen de UNA máquina —WSL2 sobre un i5-1135G7—'`; `crates/zk-ssl-guardian/src/lib.rs:42-47; hbs-state/src/lib.rs:40-45; hbs-state/README.md:82-85`; `AUDITORIA.md l.16843-16844: coste = 0,57 % de una firma MT 40/8 (0,907/160,5 ms)`; `AUDITORIA.md l.16831-16841 (banco K.1, entrada 234)`; `AUDITORIA.md l.16912-16915 'Los umbrales salen de UNA máquina —WSL2 sobre un i5-1135G7—'`; `python3: 0.907/160.5 = 0.00565`; `AUDITORIA.md l.16912-16915`

### G3e

**CONFIRMADA** · gravedad baja

- **Borrador:** Dónde está medido y si se ha ejercitado fuera del banco
- **Lo que se puede afirmar:** La medida de referencia está en AUDITORIA entrada 234 (K.1); además hay dos rechazos reales registrados con el nodo vivo sobre /tmp en WSL (razón 0,8× y 1,0×), AUDITORIA l.18102-18109 y l.21575-21578.
- **Fuentes:** `AUDITORIA.md l.18102-18115 (§247/L.1): primer intento con el contador en /tmp → '0.6 µs con fsync frente a 0.8 µs sin él, razón 0.8×, mínimo 10×' y el nodo no arrancó`; `AUDITORIA.md l.21575-21580: un banco puso su casa en /tmp (WSL) y el guardián se negó con 'razon 1.0x frente al minimo 10x'`; `AUDITORIA.md l.18102-18109 (§247/L.1): '0.6 µs con fsync frente a 0.8 µs sin él, razón 0.8×, mínimo 10×'`; `AUDITORIA.md l.21575-21578: 'en WSL /tmp es tmpfs y el fsync dio razon 1.0x frente al minimo 10x'`; `tools/banco_apagado.sh:20-23`; `AUDITORIA.md l.18102-18109 (§247/L.1)`; `AUDITORIA.md l.21575-21578`

### G4a

**CONFIRMADA** · gravedad baja

- **Borrador:** La cabeza de época se firma con XMSS^MT (conjunto de parámetros)
- **Lo que se puede afirmar:** Conjunto XMSSMT-SHA2_40/8_256 (xmss::XmssMtSha2_40_8_256, OID RFC 8391 0x00000005), 2^40 firmas; SK de 137 B con el índice en los bytes [4,9) big-endian (crates/zk-ssl-verify/src/lib.rs:140; crates/zk-ssl-guardian/src/lib.rs:109-138). Desde el §570 lo firmado es la cabeza v6 (VERSION_FORMATO = 6, zk-ssl-verify/src/lib.rs:166).
- **Fuentes:** `crates/zk-ssl-verify/src/lib.rs:69 y :139 'pub type Conjunto = XmssMtSha2_40_8_256' ('2⁴⁰ firmas, ~35.000 años a una por segundo')`; `crates/zk-ssl-verify/src/lib.rs:909-916 test: OID publicado 0x00000005, clave pública 68 B`; `doc/xmss-evaluacion.md §5: elección MT 40/8 (firma 160,5 ms medida, verif 2,7 ms, 18.469 B)`; `crates/zk-ssl-guardian/src/lib.rs:109-138: SK = 137 B = OID(4)+índice(5, BE)+4×32`; `crates/zk-ssl-verify/src/lib.rs:69 'use xmss::{..., XmssMtSha2_40_8_256}' y :138-139 'pub type Conjunto = XmssMtSha2_40_8_256' ('2⁴⁰ firmas, ~35.000 años a una por segundo')`; `crates/zk-ssl-verify/src/lib.rs:909-916 test: OID publicado [0,0,0,5], clave pública 68 B`; `doc/xmss-evaluacion.md:133 'MT 40/8 \| 160,5 ms (medida) \| 2,7 ms \| 18.469 B'`; `crates/zk-ssl-guardian/src/lib.rs:109-138 SK = 137 B = OID(4)+índice(5, BE)+4×32`

### G4b

**CONFIRMADA** · gravedad baja

- **Borrador:** Crate y versión de XMSS, fijada con '='
- **Lo que se puede afirmar:** Se usa el crate xmss 0.1.0-pre.0 de RustCrypto, pre-release sin auditoría independiente, fijado con '=0.1.0-pre.0' en zk-ssl-node, zk-ssl-verify y zk-ssl-cli (Cargo.toml de cada uno; Cargo.lock).
- **Fuentes:** `crates/zk-ssl-node/Cargo.toml:41-45 'xmss = "=0.1.0-pre.0"' (comentario: '0.1.0-pre.0, SIN auditoría independiente ... master ya diverge del tag (sha3 → shake)')`; `crates/zk-ssl-verify/Cargo.toml:45 y crates/zk-ssl-cli/Cargo.toml:78, mismo pin`; `Cargo.lock:3296-3299 xmss 0.1.0-pre.0 (crates.io)`; `README.md:36-37 'Una dependencia criptográfica (xmss, pre-release) va clavada con = y declarada'`; `crates/zk-ssl-node/Cargo.toml:41-45 'xmss = "=0.1.0-pre.0"' ('0.1.0-pre.0, SIN auditoría independiente ... master ya diverge del tag (sha3 → shake)')`; `crates/zk-ssl-verify/Cargo.toml:45; crates/zk-ssl-cli/Cargo.toml:78 (mismo pin)`; `Cargo.lock: [[package]] name = "xmss" version = "0.1.0-pre.0" source = registry crates.io`; `README.md:36-37`

### G4c

**CONFIRMADA** · gravedad baja

- **Borrador:** El guardián reserva el índice antes de firmar
- **Lo que se puede afirmar:** FirmanteCabeza::firmar persiste primero contador+1 con fsync, después firma y por último verifica su propia firma; el índice declarado en la cabeza es el valor del contador y la hoja realmente gastada (embebida en la firma) es contador−1 (firma_cabeza.rs:179-202; tests 312-324 y 327-350; AUDITORIA §331). Desde el §569 ese índice declarado es además el reloj de la era de los recibos de recepción.
- **Fuentes:** `crates/zk-ssl-node/src/firma_cabeza.rs:179-202 firmar: 1) guardian.reservar() (persiste actual+1 con sync_all) 2) sign 3) verificar_cabeza con el mismo verificador del tercero (2,4`; `crates/zk-ssl-guardian/src/lib.rs:453-460 reservar; 495-507 persistir (seek 0, write_all LE, sync_all)`; `firma_cabeza.rs:77-112 test: SK puesto en 5 → CabezaFirmada.indice = 6, indice_de_firma(firma) = 5`; `firma_cabeza.rs:327-350 test el_guardian_persiste_antes_y_los_dos_indices_avanzan_juntos (lee el contador en disco)`; `crates/zk-ssl-node/src/firma_cabeza.rs:179-202 firmar: 1) guardian.reservar() 2) sign 3) verificar_cabeza ('Cuesta 2,4 ms sobre 144,5 —el 1,7 %')`; `crates/zk-ssl-node/src/firma_cabeza.rs:312-324 (c.indice == 1 tras la primera firma) y 327-350 (contador en disco == esperado)`; `AUDITORIA.md §331 l.25533-25537 'el embebido fuera el declarado menos uno ... Salio 5 de 5'`; `crates/zk-ssl-node/src/firma_cabeza.rs:179-202, 312-324, 327-350 (sin cambios)`

### G6b

**CONFIRMADA** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** Qué contador monótono se reconcilia en RFC-0010 y cómo encaja en R(C,K)
- **Lo que se puede afirmar:** En el RFC-0010 (E2c, conectado en el §569), el par de la reconciliación es C = contador de recepción rx (ContadorRecepcion, que persiste con el mismo GuardianIndice: fsync y autocomprobación) y K = mayor rx anotado en el registro por eras (0 si está vacío). Se reconcilia con la misma zk_ssl_guardian::Reconciliacion en cada arranque, antes de servir (main.rs:1039-1061), con política propia (main.rs:654-693). Coincide = al día → arranca. C>K>0 = huecos declarados → arranca avisando; es indistinguible de restaurar un registro viejo. K=0<C = registro vacío o perdido sobre un contador vivo → arranca avisando, sin resincronizar, y esas recepciones quedan sin hoja. K>C = contador restaurado de un respaldo con el registro vivo → no arranca, que es el único fatal. Nunca resincroniza.
- **Fuentes:** `crates/zk-ssl-node/src/recepcion.rs:3-29 y 85-110: ContadorRecepcion = GuardianIndice entero (fsync + autocomprobación); cuenta operaciones que el nodo llega a EVALUAR (no el ruido`; `recepcion.rs:112-128 reconciliar(mayor_anotado) → la misma Reconciliacion ('el MISMO problema con otros dos números'); ClaveAdelantada = 'el registro por delante del contador'`; `registro_recepcion.rs:198-225 mayor_anotado = máximo rx en los ficheros era-<era>.bin (0 si no hay ninguno)`; `registro_recepcion.rs:47-59: CONTADOR restaurado viejo → registro por delante → ClaveAdelantada (fatal); REGISTRO restaurado viejo → ContadorAdelantado (benigno e indistinguible de`; `crates/zk-ssl-node/src/recepcion.rs:3-29 y 85-110 (ContadorRecepcion = GuardianIndice; cuenta lo que el nodo EVALÚA)`; `crates/zk-ssl-node/src/main.rs:146-147 --contador-recepcion por defecto recepcion.bin`; `crates/zk-ssl-node/src/recepcion.rs:112-128 ('el MISMO problema con otros dos números'; ClaveAdelantada = 'el registro por delante del contador')`; `crates/zk-ssl-node/src/registro_recepcion.rs:198-225 mayor_anotado = máximo rx en era-*.bin, o 0`

### G6c

**CONFIRMADA** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** (implícito) La reconciliación de recepción ya opera en el arranque del nodo
- **Lo que se puede afirmar:** Desde el §569 (commit b6d29fb) la reconciliación de recepción opera en cada arranque del nodo. El contador de recepción y el registro por eras se abren antes de servir, se reconcilian con la misma Reconciliacion del guardián y deciden con politica_del_registro, que no arranca sólo si el registro va por delante del contador (main.rs:654-693, 1039-1061). applySend y applyClaim anotan cada recepción, con fsync, antes de evaluarla (main.rs:2669-2728). En d531c80 esto no era así, y cualquier texto fechado antes del §569 que lo afirme era falso en su fecha.
- **Fuentes:** `grep de 'RegistroRecepcion\|registro_recepcion::\|ANCHO_ENTRADA\|mayor_anotado' fuera de registro_recepcion.rs: 0 usos; main.rs:45 solo declara 'mod registro_recepcion;'`; `ContadorRecepcion::reconciliar solo lo llama el test reconciliar_dice_lo_mismo_que_el_guardian (recepcion.rs:199-223; el comentario admite que sin él sería dead_code)`; `ContadorRecepcion::recibir sí está cableado: main.rs:2635-2645 y App.recepcion (main.rs:897-900)`; `tools/canon.sh:97 describe §565 como 'el registro de recepcion y su reconciliacion al abrir', pero RegistroRecepcion::abrir solo crea el directorio (registro_recepcion.rs:131-135)`; `grep 'RegistroRecepcion\|registro_recepcion::\|mayor_anotado\|ANCHO_ENTRADA' fuera de registro_recepcion.rs: solo main.rs:45 'mod registro_recepcion;' y la firma de recepcion.rs:12`; `crates/zk-ssl-node/src/recepcion.rs:199-223 (único llamador de reconciliar; 'Sin un testigo que lo llame, reconciliar sale dead_code')`; `crates/zk-ssl-node/src/main.rs:897-900 y 2636 (ContadorRecepcion::recibir sí está cableado)`; `tools/canon.sh:97 '§565: 126 -> 137, el registro de recepcion y su reconciliacion al abrir'; registro_recepcion.rs:131-135`

### G9c

**CONFIRMADA** · gravedad baja

- **Borrador:** Falta una 'custodia de clave comprobada, no sólo declarada' (README)
- **Lo que se puede afirmar:** La custodia de la clave del operador es una declaración (--custodia); el nodo sólo comprueba el caso 'fichero', es decir, una semilla en un fichero con permisos 0600 o más estrictos. hsm, kms y otro son afirmaciones del operador (SECURITY.md:171-175; main.rs:134-139, 925-936; README.md:159-162).
- **Fuentes:** `README.md:153-156 '...le falta un ancla anterior al primer encuentro y una custodia de clave comprobada, no sólo declarada'`; `SECURITY.md:171-175 'Custodia de la clave privada: declarada, no comprobada (§244) ... Solo fichero se comprueba'`; `crates/zk-ssl-node/src/main.rs:136-138 --custodia {sin-declarar (defecto), fichero, hsm, kms, otro}; main.rs:775-825 solo 'fichero' con --clave-fichero se marca comprobada`; `crates/zk-ssl-node/src/firma_cabeza.rs:37-39 'No hay custodia de clave'`; `README.md:151-155 '...le falta un ancla anterior al primer encuentro y una custodia de clave comprobada, no sólo declarada'`; `crates/zk-ssl-node/src/main.rs:136-138 --custodia {sin-declarar (defecto), fichero, hsm, kms, otro}; main.rs:775-824`; `README.md:159-162 '...le falta un ancla anterior al primer encuentro y una custodia de clave comprobada, no sólo declarada'`; `SECURITY.md:171-175`

## Bloque L — registro encadenado, cabeza de época, historia, paquete y anclaje (Arqueo)

### L1b

**FALSA** · gravedad alta

- **Borrador:** Fórmula ECST: H_i = H(H_{i-1} \|\| S_{i-1} \|\| S_i \|\| O_i \|\| H(Pi_i))
- **Lo que se puede afirmar:** La fórmula H(H_{i-1} \|\| S_{i-1} \|\| S_i \|\| O_i \|\| H(Pi_i)) no es la de Arqueo. Solo vale como modelo abstracto de ECST y declarado como tal. La instanciación en Arqueo es c_i = M(M(M(M(M(emb(seq_i), emb(tag_i)), M(Racc_{i-1}, Racc_i)), pd_i), c_{i-1}), compromiso_i), con merges Rescue 2 a 1 y sin concatenación. S es la raíz de cuentas, O la etiqueta u64 de OpKind, pd el digest de L1e, y compromiso el de L1f (crates/zk-ssl/src/log.rs:250-294).
- **Fuentes:** `crates/zk-ssl/src/log.rs:250-294 (orden real: seq, tag, raíces, proof_digest, anterior, compromiso; merges anidados, no concatenación)`; `crates/zk-ssl-hash/src/lib.rs:100-106 native_merge: estado de 12 elementos, l en [4..8], r en [8..12], una permutación, salida [4..8]`; `spec/NUCLEO.md:236-240 (definición normativa de native_merge y embeber)`; `crates/zk-ssl/src/log.rs:250-294`; `crates/zk-ssl-hash/src/lib.rs:100-106`; `crates/zk-ssl-hash/src/lib.rs:121-132`; `spec/NUCLEO.md:238-242`

### L3a

**FALSA** · gravedad alta

- **Borrador:** El test t1_cabeza_ata_la_historia se ejecutó sobre N=100 épocas
- **Lo que se puede afirmar:** El test t1_cabeza_ata_la_historia usa N = 12 entradas del registro de transiciones (crates/zk-ssl/src/log.rs:1198), no 100 épocas. Está registrado en verde en AUDITORIA.md §115.1, y la re-ejecución del 2026-09-30 sobre 71c5aad dio 3 passed (doc/ecst/borrador/REPRODUCCION.md:53).
- **Fuentes:** `crates/zk-ssl/src/log.rs:1140 'const N: u64 = 12;'`; `AUDITORIA.md §115.1 (tabla de t1_*: '3/3 en verde')`; `crates/zk-ssl/src/log.rs:1189-1199`; `AUDITORIA.md §115.1 (líneas 9467-9481)`; `doc/ecst/borrador/REPRODUCCION.md:13-14, 53`; `grep 'const N: u64 = 100\|K: u64 = 10' crates: sin resultados`

### L3b

**FALSA** · gravedad alta

- **Borrador:** Se alteró la época k=10
- **Lo que se puede afirmar:** La entrada alterada es K = 5 (crates/zk-ssl/src/log.rs:1199). En el test, 'época' es el índice seq de una entrada del registro, no el intervalo entre dos cabezas firmadas.
- **Fuentes:** `crates/zk-ssl/src/log.rs:1141 'const K: u64 = 5; // la epoca de la mentira'`; `crates/zk-ssl/src/log.rs:1199`

### L3c

**FALSA** · gravedad alta

- **Borrador:** Se alteró un único byte de la prueba Pi_k
- **Lo que se puede afirmar:** No se altera 'un byte de la prueba Pi_k'. El test no contiene pruebas STARK: las 'pruebas' son cadenas sintéticas de 8 B (b"honesta" + [i]). La propiedad se ejerce de dos maneras. (1) Sustituyendo el proof_digest de la entrada 5 por digest_of_proof(b"mentira-injertada") al recomputar (log.rs:1243-1256). (2) Construyendo otra historia en la que la entrada 5 lleva b"mentira" + [5]: misma longitud y 5 de 8 bytes distintos (log.rs:1207-1222).
- **Fuentes:** `crates/zk-ssl/src/log.rs:1150-1165 historia(): las 'pruebas' son los bytes b"honesta" + [i] (o b"mentira" + [i] en K); tipo = primer tag válido (1, OpenAccount); raíces as_digest(i`; `crates/zk-ssl/src/log.rs:1183-1200 recompute: en seq == K sustituye el proof_digest por digest_of_proof(b"mentira-injertada")`; `crates/zk-ssl/src/log.rs:1210 assert_ne!(historia(N, true).head(), cabeza_firmada)`; `crates/zk-ssl/src/log.rs:1207-1222`; `crates/zk-ssl/src/log.rs:1214`; `crates/zk-ssl/src/log.rs:1243-1256`; `crates/zk-ssl/src/log.rs:1280-1293`; `python3: len(b'honesta'+bytes([5])) == len(b'mentira'+bytes([5])) == 8; 5 bytes distintos`

### L5b

**FALSA** · gravedad alta · cambió con el árbol (S566–S582)

- **Borrador:** La verificación offline re-verifica las pruebas STARK de las transiciones (Verify(Pi_i))
- **Lo que se puede afirmar:** zk-ssl-verify no re-verifica las pruebas de transición. En el paquete de posición recompone el epochDigest, verifica la firma XMSS contra la clave que trae el propio paquete, sube el acuse (que liga el proofDigest, no la prueba) y cuenta las cofirmas. Solo verifica STARKs en los sobres edad, cobro_pendiente, pago_en_curso, prenda y rechazo por InsufficientBalance, y en el de completitud cuando su resolución es uno de esos rechazos. Son pruebas nuevas sobre el estado comprometido bajo una cabeza v5 o v6 (RFC-0007/0008), no las Pi_i del registro.
- **Fuentes:** `crates/zk-ssl-verify/src/main.rs:198-365 (posición: 1/3 recompone epochDigest según la versión; 2/3 firma XMSS contra la publicKey del paquete y el preámbulo; 3/3 hoja acuse_digest`; `spec/PAQUETE.md:674-676 'el paquete demuestra QUE la entrada está acusada bajo esa cabeza firmada, no QUÉ dice'`; `crates/zk-ssl-verify/Cargo.toml (zk-ssl-air + winter-verifier solo para pruebas sobre el estado comprometido; §465)`; `spec/PAQUETE.md:215-241, 242-358 y 2.6 fila InsufficientBalance (únicos sobres con STARK)`; `crates/zk-ssl-verify/src/main.rs:138-370`; `crates/zk-ssl-verify/src/main.rs:1278, 1339, 1800, 1832`; `crates/zk-ssl-verify/Cargo.toml`; `crates/zk-ssl-air/Cargo.toml`

### L5c

**FALSA** · gravedad alta

- **Borrador:** La prueba Pi_i se conserva como evidencia adjunta recuperable
- **Lo que se puede afirmar:** Arqueo no conserva las pruebas de transición. El nodo guarda solo proof_digest y ningún método RPC sirve pruebas. Re-verificarlas más tarde depende de que el productor (pagador o cobrador) guarde la suya, y el repositorio reconoce que eso no tiene política de retención (doc/DIAGNOSTICO_ESCALADO.md:412-419; RFC-0007 D-I). Las frases 'quien quiera verificarla puede pedirla' (log.rs:198) y 'la prueba está ahí y verifica' (log.rs:38) están rancias y no deben citarse.
- **Fuentes:** `crates/zk-ssl/src/log.rs:194-199 'Resumen de la prueba, no la prueba entera'`; `spec/RPC.md:239-244 'El registro no guarda ninguno de los tres'`; `spec/rfc/0007-pruebas-sobre-el-estado-comprometido.md:488-492 D-I: 'Medido, no hay material que re-verificar. El registro guarda el proof_digest y NO la prueba'`; `doc/DIAGNOSTICO_ESCALADO.md:412-419 'El nodo no los guarda ... si nadie conserva las pruebas, la re-verificación por un tercero depende de que el productor guarde la suya ... hoy n`; `spec/RPC.md:76-93 (ningún método sirve pruebas)`; `crates/zk-ssl/src/log.rs:36-39`; `crates/zk-ssl/src/log.rs:191-198`; `spec/RPC.md:76-95`

### L5d

**FALSA** · gravedad alta

- **Borrador:** El auditor recalcula la cadena de H_0 a H_N y ejecuta Verify(Pi_i) para cada transición
- **Lo que se puede afirmar:** No existe tal procedimiento de auditor. La fórmula de la cadena vive en la capa (crates/zk-ssl/src/log.rs) y no en el núcleo ni en zk-ssl-verify. Recomputan la cadena el propio nodo (zkssl_verifyChain, una autoevaluación), el CLI sobre su capa local y la capa al reabrir. Ninguna herramienta de terceros la recomputa contra una cabeza custodiada. reverificar() comprueba los sellos de las seis clases con sello, pero no tiene llamantes. Verify(Pi_i) es imposible porque las pruebas no se conservan.
- **Fuentes:** `grep: chain_digest solo existe en crates/zk-ssl/src/log.rs:250,281 (no en zk-ssl-hash ni en zk-ssl-verify)`; `crates/zk-ssl-verify/src/reverificacion.rs:44-47 'el registro no guarda la prueba: no seran recomputables desde el registro nunca'`; `grep 'reverificar(': sin llamantes fuera de su módulo (ni el kit, ni el cli, ni el nodo)`; `spec/NUCLEO.md:43-45 (REGISTRO: 'no pertenecen al sobre')`; `crates/zk-ssl/src/log.rs:250, 281, 393-431`; `crates/zk-ssl-node/src/main.rs:2558, 2576`; `crates/zk-ssl-cli/src/commands.rs:464-466`; `crates/zk-ssl/src/persistence.rs:586-590`

### L6a

**FALSA** · gravedad alta

- **Borrador:** Crecimiento del log ~256 bytes por transición (almacenando solo H(Pi_i))
- **Lo que se puede afirmar:** Cada entrada serializada ocupa 169 B en la era 2 (8 de seq + 1 de tag + 5 digests de 32) y 137 B en la era 1 (store.rs:411-445, fijado por test). En sled se guarda con una clave de 12 B. Si el libro se abre cifrado, el valor sube a 209 B. Son cálculos desde el código; nadie ha medido el crecimiento en disco. La cifra de ~256 B no tiene fuente.
- **Fuentes:** `crates/zk-ssl/src/store.rs:418-430 log_entry_to_bytes: seq u64 (8) + tag (1) + root_old, root_new, proof_digest, chain (4x32) [+ compromiso 32]`; `crates/zk-ssl/src/log.rs:1470-1484 test la_ida_y_vuelta_del_disco_conserva_la_era: 169 (era 2) y 137 (era 1)`; `doc/DIAGNOSTICO_ESCALADO.md:409-412 '≈137 bytes ... Estimación desde la estructura, no medida en disco'`; `crates/zk-ssl/src/store.rs:411-445`; `crates/zk-ssl/src/log.rs:1527-1556`; `crates/zk-ssl/src/persistence.rs:20, 49, 839-841`; `crates/zk-ssl/src/crypto.rs:103-113`; `crates/zk-ssl-node/src/main.rs:1350`

### L9a

**FALSA** · gravedad alta

- **Borrador:** Un tercero importa los vectores y la cabeza publicada y verifica toda la secuencia de evidencia histórica sin consultar el nodo
- **Lo que se puede afirmar:** El kit verifica afirmaciones empaquetadas de una en una: posición, extensión entre dos cabezas, consumo, conflicto entre dos libros, causa de un rechazo, pruebas de estado v5/v6 y, en el árbol, completitud de un recibo. No recorre ni recompone el registro de transiciones. Lo más parecido a una historia es el '--auditar' del testigo, que re-verifica sin el nodo el diario de cabezas firmadas que ese testigo recogió.
- **Fuentes:** `crates/zk-ssl-verify/src/main.rs:157-178 (un fichero, una forma; contrato './zk-ssl-verify <fichero.json>')`; `doc/KIT.md:36-40 (VERDE/ROJO por fichero)`; `grep 'reverificar(' sin llamantes; chain_digest solo en crates/zk-ssl/src/log.rs`; `crates/zk-ssl-cli/src/witness.rs:1068-1079 (auditar_lineas: re-verifica un DIARIO de cabezas, no el registro)`; `crates/zk-ssl-verify/src/main.rs:128-187`; `doc/KIT.md:1-45`; `spec/vectors/paquete/posicion-v2.json (cabeza.formatVersion = 0x3)`; `crates/zk-ssl-cli/src/witness.rs:1060-1090 (auditar_lineas)`

### L6b

**SIN_FUENTE** · gravedad alta

- **Borrador:** Reducción del 99,6 %
- **Lo que se puede afirmar:** El repositorio no contiene esa cifra. Si se quiere una razón, es aritmética y no medida: 169 B por entrada frente a 72-84 KB por prueba (metrics.rs) da ≈99,8 %. Y no 'reduce' nada desplegado, porque el nodo nunca almacenó pruebas.
- **Fuentes:** `grep '99[.,]6' en el repo: solo AUDITORIA.md:22949 ('99,65 GiB', no relacionado)`; `crates/zk-ssl/src/metrics.rs:82-83, 111-112 (pago 145.953..167.967 B; envío 73.571..84.244 B, cobro 72.382..83.723 B con margen del 5 %; medidos 77.444..80.232 y 76.192..79.736)`; `cálculo propio: 1-256/62.000 = 99,59 %; 1-169/77.444 = 99,78 %`; `grep '99[.,]6' (solo AUDITORIA '99,65 GiB', no relacionado)`; `crates/zk-ssl/src/metrics.rs:82-83, 111-117`; `cálculo propio con python3`

### L6c

**SIN_FUENTE** · gravedad alta

- **Borrador:** Auditoría histórica offline ~125.000 bloques/s
- **Lo que se puede afirmar:** No hay medida de ritmo de auditoría ni de recomputación de la cadena. Las cifras medidas más próximas son de otra cosa: apply ≈3,6 ms/op con 1e5 cuentas (§219) y digest_of_proof ≈0,01 ms (§209).
- **Fuentes:** `grep '125[.,]000\|125000\|bloques/s\|blocks/s': ninguna medida relacionada`; `AUDITORIA.md §219 (apply 3,58-3,67 ms, 273-280 op/s a 1e3-1e5 cuentas; no es verificación de cadena)`; `grep '125[.,]000\|bloques/s\|blocks/s': sin resultados`; `AUDITORIA.md §219 (15024 y ss.)`

### L6d

**SIN_FUENTE** · gravedad alta

- **Borrador:** Sobrecarga de encadenamiento < 0,002 ms
- **Lo que se puede afirmar:** El coste del encadenado no se ha medido por separado. Se conoce su tamaño: 5 merges Rescue por entrada en la era 1 y 6 en la era 2, más un Blake3 del orden de 0,01 ms por prueba de ~65 KB (§204, §209). El '0,002 ms' del repositorio es el fsync sobre tmpfs del guardián y no tiene relación.
- **Fuentes:** `crates/zk-ssl-guardian/src/lib.rs:44-47 y AUDITORIA.md §234: '0,002 ms' es el coste de fsync en tmpfs`; `AUDITORIA.md §204/§209: 'chain_digest (5 merges) SE QUEDA en Rescue'; Blake3 sobre 65.840 B = 0,011 ms`; `crates/zk-ssl-guardian/src/lib.rs:44-47`; `AUDITORIA.md §234 (16839)`; `AUDITORIA.md §204 (13882-13884), §209`

### L10a

**PARCIAL** · gravedad alta · **el escéptico revocó el primer veredicto** (FALSA)

- **Borrador:** Arqueo extiende CT (RFC 9162)
- **Lo que se puede afirmar:** Arqueo no extiende el protocolo CT ni cita el RFC 9162. Adopta el patrón de CT (no impedir la mala conducta, sino hacerla imposible de ocultar) y los algoritmos del árbol de historia del RFC 6962, sobre primitivas propias: Rescue/Goldilocks con MMRHOJA1/MMRNODO1. Las hojas de ese árbol son cabezas, no entradas; las entradas se encadenan por hash. No hay SCT ni gossip, y la clave no se publica fuera del registro (doc/CONFIANZA_RESIDUAL.md:154; crates/zk-ssl-verify/src/mmr.rs:20-26). El RFC 9162 (CT 2.0) deja obsoleto al 6962, pero Arqueo no se refiere a él.
- **Fuentes:** `grep 'RFC 9162\|9162' en .md/.rs: 0 apariciones; 'RFC 6962': 9`; `doc/CONFIANZA_RESIDUAL.md:154 'El patrón es el de Certificate Transparency (RFC 6962)'`; `crates/zk-ssl-verify/src/mmr.rs:20-26 (algoritmos RFC 6962 'sobre las primitivas de la casa')`; `WebSearch: RFC 9162 (CT v2.0, dic. 2021, Experimental) deja obsoleto el RFC 6962 (rfc-editor.org/rfc/rfc9162; datatracker.ietf.org/doc/rfc9162)`; `WebSearch: RFC 6962 §2.1 MTH con SHA-256, 0x00 hoja / 0x01 interior`; `grep '9162' en .md/.rs: 0`; `doc/CONFIANZA_RESIDUAL.md:154`; `crates/zk-ssl-verify/src/mmr.rs:20-26`

### L12

**PARCIAL** · gravedad alta

- **Borrador:** Teorema 1 aplicado a Arqueo: alterar Pi_k cambia H_n
- **Lo que se puede afirmar:** La cadena sí incluye un digest de prueba, pero solo en Send, Claim, Burn y Refund. En esas clases, alterar un byte de Pi_k cambia digest_of_proof, luego c_k…c_n, luego chainDigest y epoch_digest, y la firma deja de verificar. Hay cuatro límites. (1) Solo lo detecta quien tiene los bytes de Pi_k, porque el nodo no los guarda. (2) Enlazar no es validar: la cadena no demuestra que Pi_k verificara. (3) En las vías delegadas se ata el sello del compromiso autorizante, y en OpenAccount y Consumo constantes, así que alterar sus pruebas de custodios no cambia nada. (4) Cada entrada ata como campos propios seq, tipo, raíces de cuentas vieja y nueva, proof_digest, el anterior y el compromiso. Las raíces de congelados, pendientes y custodios, importes y suministro entran solo a través del compromiso en las delegadas y en Migration. En Send y Claim, importes y partes solo quedan atados indirectamente por el digest de la prueba. En los demás casos, las raíces de pendientes y congelados solo las ata la cabeza firmada.
- **Fuentes:** `crates/zk-ssl/src/log.rs:250-294, 370-372 (la cadena ata seq, tag, raíces de cuentas, proof_digest, el anterior y el compromiso)`; `crates/zk-ssl/src/log.rs:151-160 (solo raíces de CUENTAS por entrada)`; `crates/zk-ssl-hash/src/lib.rs:1303-1320 (Blake3 -> 4 elementos reducidos al campo)`; `crates/zk-ssl/src/log.rs:1179-1211 (T1: sustituye el digest, no los bytes)`; `crates/zk-ssl/src/mint.rs:141 (las delegadas no atan bytes de prueba)`; `spec/rfc/0007-pruebas-sobre-el-estado-comprometido.md:488-492 (no hay prueba que re-verificar)`; `crates/zk-ssl/src/log.rs:250-294, 358-380`; `crates/zk-ssl/src/freeze.rs:122-126`

### L1a

**PARCIAL** · gravedad media

- **Borrador:** Fórmula del registro encadenado: H_i = H(seq_i, op_i, root_{i-1}, root_i, H(pi_i), H_{i-1})
- **Lo que se puede afirmar:** La fórmula del borrador reproduce el esquema abstracto de la era 1 que el propio repositorio publica (log.rs:22-26; ARQUITECTURA.md:390; PAPER.md:879). La composición real (crates/zk-ssl/src/log.rs:250-294) es c1_i = M(M(M(M(emb(seq_i), emb(tag_i)), M(Racc_{i-1}, Racc_i)), pd_i), c_{i-1}). En la era 2, que rige toda entrada nueva desde §281, se toma c_i = M(c1_i, compromiso_i). M es native_merge (Rescue-Prime Rp64_256 de winter-crypto =0.13.1, sobre Goldilocks), emb(x) = [x,0,0,0], tag_i es el u64 de OpKind (de 1 a 13), las raíces son las del árbol de CUENTAS y c_{-1} = [0;4].
- **Fuentes:** `crates/zk-ssl/src/log.rs:24-26 (esquema en el doc del módulo: 'resumen = H(numero, tipo, raiz_antigua, raiz_nueva, H(prueba), resumen_anterior)')`; `ARQUITECTURA.md:390 y PAPER.md:879 (el mismo esquema, era 1)`; `crates/zk-ssl/src/log.rs:250-261 chain_digest: cabecera=native_merge(as_digest(seq), as_digest(kind.tag())); raices=native_merge(root_old, root_new); cuerpo=native_merge(cabecera, `; `crates/zk-ssl/src/log.rs:281-294 chain_digest_v2 = native_merge(chain_digest(...), compromiso)`; `crates/zk-ssl/src/log.rs:358-379 append_con_compromiso: toda entrada nueva es era 2 (compromiso: Some)`; `crates/zk-ssl/src/log.rs:151-160 root_old/root_new son SIEMPRE la raíz del árbol de CUENTAS`; `crates/zk-ssl/src/log.rs:22-26`; `crates/zk-ssl/src/log.rs:36-39`

### L1c

**PARCIAL** · gravedad media

- **Borrador:** Función hash del registro/cabeza tratada como un H genérico (sin decir si es Rescue, Blake3 o SHA-256)
- **Lo que se puede afirmar:** Hay cuatro papeles de hash. (1) La cadena del registro, el digest de la cabeza, todos los árboles del libro y el árbol de historia de cabezas usan native_merge: Rescue-Prime Rp64_256 de winter-crypto =0.13.1 sobre Goldilocks, con estado de 12 y capacidad a cero. (2) El resumen de prueba es Blake3-256(dominio \|\| len \|\| bytes), partido en 4 u64 y reducido al campo (§209). (3) Las pruebas STARK usan internamente Blake3 como hasher de sus compromisos Merkle y de Fiat-Shamir. (4) La firma de la cabeza es XMSS^MT-SHA2_40/8_256 (RFC 8391), es decir, SHA-256 dentro de la firma.
- **Fuentes:** `crates/zk-ssl-hash/src/lib.rs:100-106 (Rp64_256::apply_permutation, winter-crypto =0.13.1, BaseElement f64 = Goldilocks)`; `crates/zk-ssl-hash/src/lib.rs:1303-1320 digest_of_proof usa winter_crypto::hashers::Blake3_256`; `crates/zk-ssl-verify/src/lib.rs:69,139 Conjunto = XmssMtSha2_40_8_256`; `AUDITORIA.md §209: 'Ahora es Blake3(dominio \|\| longitud \|\| prueba)... chain_digest SIGUE en Rescue'`; `spec/NUCLEO.md:236-240`; `crates/zk-ssl-hash/src/lib.rs:100-106`; `crates/zk-ssl-hash/src/lib.rs:1303-1320`; `crates/zk-ssl-hash/Cargo.toml (winter-crypto =0.13.1, winter-math =0.13.1)`

### L1d

**PARCIAL** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** (Implícito en los borradores) el encadenado lleva separación de dominio
- **Lo que se puede afirmar:** chain_digest y chain_digest_v2 no llevan etiqueta de dominio. La era se separa por el merge adicional y, en disco, por la longitud: 137 B en la era 1 y 169 B en la era 2 (store.rs:411-445). Tampoco llevan etiqueta las composiciones de cabeza v1 a v6: las separa el byte de versión del preámbulo firmado, b"ZK-SSL-epoch-head" \|\| versión \|\| epoch_digest, que hoy va con la versión 6. Llevan dominio el resumen de prueba (b"ZK-SSL-proof-digest-v2" más la longitud), los sellos de §278, las hojas de acuse (ACUSE_V1) y de recibo de recepción (RECEP_V1, desde §557), el árbol de historia (MMRHOJA1/MMRNODO1), PARAM_V1 y PMETA_V1.
- **Fuentes:** `crates/zk-ssl/src/log.rs:250-261 (chain_digest sin etiqueta de dominio)`; `crates/zk-ssl-hash/src/lib.rs:1290-1298 dominios de bytes: b"ZK-SSL-proof-digest-v2", b"ZK-SSL-authorization-seal-v1", b"ZK-SSL-no-proof-by-design-v1"`; `crates/zk-ssl-hash/src/lib.rs:1303-1310 digest_of_proof = Blake3(dominio \|\| len_u64_LE \|\| prueba)`; `crates/zk-ssl/src/store.rs:418-445 (137 B = era 1, 169 B = era 2; otra longitud se rechaza)`; `crates/zk-ssl-hash/src/lib.rs:252-266 epoch_digest sin etiqueta: 'siempre se consume dentro de un preambulo firmado'`; `crates/zk-ssl-verify/src/lib.rs:143 DOMINIO = b"ZK-SSL-epoch-head"`; `crates/zk-ssl/src/log.rs:250-294`; `crates/zk-ssl/src/store.rs:411-445`

### L1e

**PARCIAL** · gravedad alta

- **Borrador:** La entrada encadenada incluye un digest de la prueba STARK H(Pi_i)
- **Lo que se puede afirmar:** Depende de la clase. Send, Claim, Burn y Refund encadenan proof_digest = Blake3(b"ZK-SSL-proof-digest-v2" \|\| len \|\| bytes de la prueba STARK). Mint, MintToPending, Recovery, Freeze y Governance encadenan el digest de sello_de_autorizacion(compromiso), no los bytes de ninguna prueba. OpenAccount encadena el sello constante de ausencia declarada, Migration el digest de un payload de 64 B (frozen_old \|\| frozen_new) y Consumo digest_of_proof(&[]). Lo recoge la garantía 4 de spec/RPC.md:240-247.
- **Fuentes:** `crates/zk-ssl/src/log.rs:370-372 proof_digest = digest_of_proof(proof)`; `crates/zk-ssl/src/two_phase.rs:1112 (Send con &receipt.proof), :1378 (Claim), :520/:786 (Refund); crates/zk-ssl/src/burn.rs:217`; `crates/zk-ssl/src/mint.rs:141, freeze.rs:160, recovery.rs:176, governance.rs:166, two_phase.rs:1677: sello_de_autorizacion(&operation) en lugar de la prueba`; `crates/zk-ssl/src/accounts.rs:258-263 OpenAccount: sello_sin_prueba()`; `crates/zk-ssl/src/consumo.rs:146-147 Consumo: append_con_compromiso(..., &[], consumo)`; `crates/zk-ssl/src/migration.rs:109-116 Migration: payload de 64 B (frozen_old\|\|frozen_new), 'NO es una prueba'`; `spec/RPC.md:239-244 garantía 4`; `crates/zk-ssl/src/two_phase.rs:520, 786, 1112, 1378, 1677-1683`

### L1f

**PARCIAL** · gravedad media

- **Borrador:** La entrada encadenada incluye material de autorización A_i
- **Lo que se puede afirmar:** Desde §281 cada entrada nueva encadena un campo compromiso. En las cinco vías delegadas es commit_operation(OP_*, parámetros): el compromiso contra el que se verificó el par de pruebas de umbral de los custodios, que contiene las raíces del árbol que esa vía mueve. En las demás es el centinela COMPROMISO_AUSENTE y en Consumo es el digest del consumo. No se encadenan las pruebas de umbral ni las firmas. En Send y Claim la autoría está dentro del STARK y solo queda atada por el digest de la prueba. Las entradas de la era 1 no tienen compromiso.
- **Fuentes:** `crates/zk-ssl/src/log.rs:182-190 campo compromiso: None = era 1, Some = era 2`; `crates/zk-ssl/src/mint.rs:105-119 operation = commit_operation(OP_MINT, raíces\|\|amount\|\|supply_old\|\|supply_new\|\|max_supply); verify_threshold_pair(proof_a, ..., proof_b, ..`; `crates/zk-ssl-hash/src/lib.rs:1383 COMPROMISO_AUSENTE = [0xC0A92281;4]`; `crates/zk-ssl-verify/src/reverificacion.rs:27-52 (era 2: seis clases con sello recomputables; Send/Claim/Burn/Refund/Migration 'no seran recomputables desde el registro nunca')`; `crates/zk-ssl/src/log.rs:182-190`; `crates/zk-ssl/src/freeze.rs:122-126`; `crates/zk-ssl/src/mint.rs:105-141`; `crates/zk-ssl-hash/src/lib.rs:1274, 1383`

### L2a

**PARCIAL** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** La cabeza de época ata las raíces del estado en reposo, el registro encadenado de transiciones y el árbol de consumos publicados
- **Lo que se puede afirmar:** La frase del README (README.md:110-113) es cierta e incompleta. La cabeza que el nodo firma hoy es la v6 (§570). Compromete 18 campos: seq, accountsRoot, pendingRoot, frozenRoot, chainDigest, acusesRoot, n (1440), mmrRoot, mmrSize, consRoot, consCount, paramsDigest, pmetaRoot, nextPending, nextIndex, totalSupply, recepRoot y recepCount (log.rs:586-649; zk-ssl-hash/src/lib.rs:469-510). Del registro encadenado ata el último resumen, chainDigest, que compromete toda la historia por transitividad, pero no ata las entradas en sí. Excluye verifier_hash a propósito.
- **Fuentes:** `README.md:110-112 (la frase, literal)`; `crates/zk-ssl/src/log.rs:586-661 EpochHead: 16 campos`; `crates/zk-ssl-hash/src/lib.rs:403-452 epoch_digest_v5`; `spec/RPC.md:78 zkssl_epochHead y :521-527 zkssl_signedEpochHead ('los dieciséis campos')`; `README.md:110-113`; `crates/zk-ssl/src/log.rs:586-689 (18 campos; recep_root:645, recep_count:649; FALTA verifier_hash:650)`; `crates/zk-ssl/src/log.rs:703-727 (digest() -> epoch_digest_v6)`; `crates/zk-ssl-hash/src/lib.rs:469-510`

### L2b

**PARCIAL** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** Versión de la cabeza firmada (los borradores no la fijan; v5/v6)
- **Lo que se puede afirmar:** El nodo firma cabezas v6 desde §570 (RFC-0010 E2d; VERSION_FORMATO = 6). La v6 es la v5 envuelta con merge(recepRoot, as_digest(recepCount)). El RFC-0010 está ACEPTADO desde §577. Antes rigieron la v5 (§452), la v4 (§415), la v3 (§292) y la v2 (§275). Una cabeza custodiada de v2 a v5 sigue verificando con su recomponedor. Un informe debe fijar la versión y el commit, porque la versión cambió dentro del periodo en que se escribieron los borradores.
- **Fuentes:** `crates/zk-ssl-verify/src/lib.rs:163 VERSION_FORMATO: u8 = 5 ('4 -> 5 en el §452')`; `crates/zk-ssl/src/log.rs:693-716 EpochHead::digest() llama a epoch_digest_v5`; `crates/zk-ssl-verify/src/lib.rs:185 V6 = 6 (aceptada por el núcleo)`; `spec/rfc/0010-el-recibo-de-recepcion.md:28-30 'E2a y E2b selladas; faltan E2c y E2d'`; `AUDITORIA.md §452`; `crates/zk-ssl-verify/src/lib.rs:158-166`; `crates/zk-ssl-verify/src/lib.rs:175-190 (VersionCabeza V2..V6)`; `crates/zk-ssl-node/src/latido.rs:186-205`

### L2c

**PARCIAL** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** (Implícito) Todo lo que sirve la cabeza firmada está bajo la firma
- **Lo que se puede afirmar:** Solo se firman 50 bytes: b"ZK-SSL-epoch-head" (17) \|\| formatVersion (1 B, hoy 6) \|\| epoch_digest (32). La firma es XMSS^MT-SHA2_40/8_256: 18.469 B en formato RFC 8391 y 18.519 B con el mensaje adjunto. Los 18 campos quedan cubiertos porque recomponen epoch_digest. Viajan sin firma custody, custodyChecked, emittedAtUnix, beatSeconds y el receptionSeq de las respuestas. Este último queda atado a posteriori cuando la era del recibo cierra bajo una cabeza v6 (hoja recibo_digest bajo recepRoot, camino por zkssl_recepPath). El firmante reserva el índice con fsync antes de firmar y verifica su propia firma antes de devolverla.
- **Fuentes:** `crates/zk-ssl-node/src/firma_cabeza.rs:21-23 'preámbulo = b"ZK-SSL-epoch-head" ‖ versión_de_formato ‖ epoch_digest'`; `spec/NUCLEO.md:246-249 preámbulo de 17 + 1 + 32 = 50 B`; `crates/zk-ssl-verify/src/lib.rs:279 FIRMA_RFC_BYTES = 18_469; firma_cabeza.rs:324-325 c.firma.len() == 18_519`; `crates/zk-ssl-node/src/firma_cabeza.rs:178-201 (reserva el índice con fsync ANTES de firmar y verifica su propia salida)`; `spec/RPC.md:616-646 custody/custodyChecked/emittedAtUnix/beatSeconds`; `crates/zk-ssl-node/src/main.rs:2647-2652 receptionSeq 'NO está firmado'`; `crates/zk-ssl-node/src/firma_cabeza.rs:20-23, 178-201, 319-323`; `crates/zk-ssl-verify/src/lib.rs:144, 166, 282`

### L3d

**PARCIAL** · gravedad media

- **Borrador:** Resultado H_100' != H_100 (la cabeza cambia)
- **Lo que se puede afirmar:** El test afirma que recomputar desde los campos crudos reproduce head(), que la cadena final recomputada con la entrada 5 alterada difiere de head() y que la historia alternativa da otro head(), todo con N = 12. Su hermano t1_divergencia_localizada afirma que la divergencia se localiza exactamente en k = 5 y contamina todo j >= 5. 'H_N' es el último chain del registro, no una cabeza firmada. Que la firma cubra la alteración se deduce de que chainDigest entra en epoch_digest; el test no lo ejerce.
- **Fuentes:** `crates/zk-ssl/src/log.rs:1179-1211: cabeza_firmada = a.head() (TransitionLog::head, el último chain); assert_eq!(recompute(None), cabeza_firmada); assert_ne!(recompute(Some(K)), ca`; `crates/zk-ssl/src/log.rs:1201-1222`; `crates/zk-ssl/src/log.rs:1225-1234`; `crates/zk-ssl/src/log.rs:1236-1277`; `AUDITORIA.md §115.1 (líneas 9475-9481)`

### L3e

**PARCIAL** · gravedad media

- **Borrador:** Detectado sin re-verificar ZK
- **Lo que se puede afirmar:** Es trivialmente cierto: el test solo usa digest_of_proof (Blake3) y chain_digest_v2 (Rescue), sin ninguna prueba ni verificador ZK. Mide que alterar el digest de la entrada k cambia el chain de n. No dice nada de la validez de las pruebas, y según AUDITORIA §116 tampoco mide la resistencia a colisión del resumen. Su finalidad declarada es sostener la cadencia de una firma por minuto (§115).
- **Fuentes:** `crates/zk-ssl/src/log.rs:1150-1211 (solo digest_of_proof y chain_digest_v2; ningún verificador STARK)`; `crates/zk-ssl/src/log.rs:1134-1137 'Sostiene la decision 1-firma/min (entrada 53)'`; `crates/zk-ssl/src/log.rs:1189-1195`; `crates/zk-ssl/src/log.rs:1236-1277`; `AUDITORIA.md §115.1 y §116 (líneas 9525-9531)`

### L4a

**PARCIAL** · gravedad media

- **Borrador:** Historia no reescribible sobre un MMR
- **Lo que se puede afirmar:** Lo que el código llama 'MMR de cabezas' es un árbol de historia al estilo del RFC 6962 (MTH, PATH y SUBPROOF, con corte en la mayor potencia de dos menor que n), construido sobre primitivas propias: Rescue con etiquetas MMRHOJA1 y MMRNODO1, no SHA-256 con 0x00/0x01. Sus hojas son los digests de las cabezas anteriores, no entradas del registro. La pareja (mmrRoot, mmrSize), firmada desde la v3 (§292), es el acumulador previo a esa cabeza.
- **Fuentes:** `crates/zk-ssl-verify/src/mmr.rs:20-26 'Los algoritmos son los del arbol de historia de RFC 6962 ... MTH, PATH y SUBPROOF'`; `AUDITORIA.md §291 'Por que RFC 6962 y no un MMR de picos'`; `crates/zk-ssl-hash/src/lib.rs:737-739, 843-851 mmr_hoja = M(as_digest(MMRHOJA1), cabeza); mmr_nodo = M(as_digest(MMRNODO1), M(izq, der))`; `crates/zk-ssl-hash/src/lib.rs:296-333 epoch_digest_v3 (cima, t) firmados`; `spec/RPC.md:922-926 'la pareja firmada es el acumulador ANTES de cada cabeza'`; `crates/zk-ssl-verify/src/mmr.rs:12-26`; `crates/zk-ssl-hash/src/lib.rs:737-739, 843-851`; `AUDITORIA.md §291 (21611)`

### L4d

**PARCIAL** · gravedad alta · cambió con el árbol (S566–S582)

- **Borrador:** Historia no reescribible
- **Lo que se puede afirmar:** La estructura da evidencia de manipulación, no inmutabilidad: la secuencia de cabezas firmadas no se reescribe sin que lo vea quien custodia una cabeza anterior de la misma clave. Límites. (a) Sin una cabeza previa custodiada no hay nada que comparar (§76, TOFU). (b) La prueba de consistencia no comprueba que el chainDigest nuevo continúe al viejo, y ninguna herramienta recompone la cadena contra una cabeza custodiada. (c) El paquete de extensión no comprueba que la propia cabeza vieja sea hoja del acumulador nuevo (lectura, no ejercitada). (d) Un reinicio que pierde el diario queda visible como 'por-detras' y no detiene al testigo. (e) Omisión: desde el RFC-0010, una operación recibida por applySend o applyClaim deja recibo firmado y su falta de resolución se demuestra de forma oponible con el sobre de completitud. En cambio, la operación a la que el nodo nunca dio recibo y la que entra por applyMany o pledge no dejan rastro. (f) El operador no sirve el histórico de cabezas.
- **Fuentes:** `crates/zk-ssl/src/log.rs:561-563 (§76: la garantía 'solo vale para quien ya observó una cabeza anterior')`; `crates/zk-ssl-verify/src/main.rs:578-601 (compara cimas; no comprueba que el digest de la cabeza vieja sea hoja del acumulador nuevo ni relaciona los chainDigest)`; `tools/banco_consistencia.sh:25-29 NEGATIVO-B: nodo reseteado -> 'por-detras', el testigo NO se detiene`; `spec/RPC.md:660-700 (§248: el operador NO sirve el histórico; 'un testigo que no existía no tiene diario')`; `crates/zk-ssl-cli/src/witness.rs:108-114 ('No detecta omisión')`; `SECURITY.md:115-119`; `crates/zk-ssl/src/log.rs:558-563`; `crates/zk-ssl-verify/src/main.rs:608-640`

### L7c

**PARCIAL** · gravedad alta · cambió con el árbol (S566–S582)

- **Borrador:** Anclaje externo: interfaz diseñada en doc/ANCLAJE_EXTERNO.md
- **Lo que se puede afirmar:** En el árbol, doc/ANCLAJE_EXTERNO.md es un marcador vacío de una línea, sin cambios desde el commit más antiguo de este clon superficial. El diseño del anclaje externo solo está descrito en AUDITORIA §174 y en las filas B10.6/B10.7 de doc/CONFIANZA_RESIDUAL.md: publicar la raíz en un medio append-only, por lotes de M latidos. No hay nada construido ni desplegado. Lo que §174 propone anclar sí existe hoy bajo la firma: acusesRoot desde la v2 y recepRoot desde la v6 (§570). No se debe citar ANCLAJE_EXTERNO.md como fuente de contenido.
- **Fuentes:** `doc/ANCLAJE_EXTERNO.md: una sola línea, '[el contenido de ANCLAJE_EXTERNO.md]', sin salto final; así desde el commit de importación 087aecc`; `SECURITY.md:225-226 'interfaz diseñada: doc/ANCLAJE_EXTERNO.md; pendiente de despliegue'`; `AUDITORIA.md §174 (el diseño descrito: publicar la raíz en un medio append-only, por lotes cada M latidos)`; `doc/CONFIANZA_RESIDUAL.md:487-488 B10.6/B10.7`; `doc/ANCLAJE_EXTERNO.md (37 B, una línea)`; `git rev-parse --is-shallow-repository -> true; git log --oneline \| tail -1 -> 087aecc`; `AUDITORIA.md §174 (12659-12689)`; `AUDITORIA.md §121.2 (9823-9840)`

### L8

**PARCIAL** · gravedad alta · cambió con el árbol (S566–S582)

- **Borrador:** Una vez publicada y atada a una cabeza de época, la historia se vuelve inalterable y auditable de forma independiente con el nodo apagado
- **Lo que se puede afirmar:** Hay que matizarlo. La historia no es inalterable: es evidente a la manipulación para quien custodia una cabeza anterior de la misma clave (TOFU, sin ancla externa). Con el nodo apagado, el titular prueba con su paquete tres cosas: que la cabeza la firmó la clave que trae el paquete, que sus campos la recomponen y que su acuse (solo en Send y Claim) está bajo acusesRoot. No prueba qué dice la entrada. Con el sobre de completitud (RFC-0010) puede además probar que una operación recibida por applySend o applyClaim se resolvió en su ventana, o señalar de forma oponible que no. No se pueden auditar la historia completa ni la validez de cada transición: las pruebas no se conservan y el operador no sirve el histórico. Lo que nunca recibió recibo, y lo que entra por applyMany o pledge, no deja rastro. Sin ancla ni custodia comprobada, la firma no identifica a nadie ante terceros.
- **Fuentes:** `crates/zk-ssl/src/log.rs:561-583 (detectable, no oponible sin firma/ancla; 'la independencia de los testigos es un supuesto social')`; `spec/PAQUETE.md:674-685 (no hace oponibles las firmas; no dice qué contiene la entrada)`; `crates/zk-ssl-node/src/main.rs:2661-2667 (acuse solo en applySend/applyClaim; las delegadas no lo emiten)`; `spec/RPC.md:643-700 (el operador no sirve histórico; solo diarios de testigos)`; `SECURITY.md:208 'No hay recuperación si el nodo desaparece'`; `crates/zk-ssl-cli/src/witness.rs:108-114`; `crates/zk-ssl/src/log.rs:558-583 (doc rancio en lo de la firma)`; `spec/PAQUETE.md:723-734`

### L9b

**PARCIAL** · gravedad alta · cambió con el árbol (S566–S582)

- **Borrador:** Existe la release arqueo-verify-v0.2.0 utilizable contra la cabeza publicada
- **Lo que se puede afirmar:** La release arqueo-verify-v0.2.0 existe: commit 1528943, tarball sha256 2fe9030a…79f5ce de 2.891.949 B, binario 6356debd…478b y tres catálogos. Es anterior a §451, así que acepta cabezas v2 a v4 y rechaza la v5 y la v6. Tampoco incluye los sobres de rechazo, edad, cobro, pago, prenda ni completitud. No sirve, por tanto, para las cabezas v6 que el nodo firma desde §570. El artefacto del árbol actual, con nueve familias y el crate aún numerado 0.2.0, no se ha publicado como release.
- **Fuentes:** `GitHub API get_release_by_tag arqueo-verify-v0.2.0: publicada 2026-09-09T11:39:34Z, commit 1528943fdfb9399f56fd836f75ffbe655d004d78, tarball sha256 2fe9030a...79f5ce, 2.891.949 B; `; `list_releases: es la última (antes: arqueo-verify-v0.1.0, v1.0.0)`; `AUDITORIA.md §443 (release) frente a §451/§452 (v5 en núcleo y nodo, VERSION_FORMATO 4 -> 5)`; `spec/PAQUETE.md:699 (hasta §451 'rechazo-formatVersion-5' era un vector NEGATIVO)`; `spec/PAQUETE.md:889-894 (el artefacto del árbol ya lleva seis catálogos)`; `GitHub API get_release_by_tag arqueo-verify-v0.2.0 y list_releases (2026-09-30)`; `AUDITORIA.md §443 (32823 y ss.)`; `README.md:47-52, 194`

### L10b

**CONFIRMADA** · gravedad baja

- **Borrador:** README: 'responsabilidad demostrable, al modo de Certificate Transparency'
- **Lo que se puede afirmar:** El README se sitúa en la responsabilidad demostrable al modo de CT, no en el consenso. Declara que tiene firmante, guardián del índice, latido, verificador independiente y testigos, y que le faltan un ancla previa al primer encuentro y una custodia de clave comprobada. La cota n = 1440 cabezas (24 h al latido de 60 s) sigue el precedente del MMD de CT.
- **Fuentes:** `README.md:152-155`; `crates/zk-ssl/src/log.rs:44-46; ARQUITECTURA.md:397; PAPER.md:887`; `crates/zk-ssl-node/src/vista_acuses.rs:17-19 (N = 1440 cabezas = 24 h, 'con el precedente del MMD de Certificate Transparency')`; `README.md:157-162`; `crates/zk-ssl/src/log.rs:44-46`; `crates/zk-ssl-node/src/vista_acuses.rs:17-28`

### L11a

**CONFIRMADA** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** Partes del NÚCLEO congelado
- **Lo que se puede afirmar:** El núcleo congela el lado del verificador: hash y serialización, composiciones de cabeza v1 a v6, firma y preámbulos, acuses y recibos de recepción, el árbol de historia, inclusión, congelados y cuentas. Lo fijan KAT en spec/vectors/nucleo/ y lo vigila tools/check_nucleo.py; quedan fuera el cable y el libro. digest_of_proof, los sellos, commit_operation y reverificar son de clase REGISTRO. La composición de chain_digest no está ni en el núcleo ni en los crates del verificador. Queda deriva documental: NUCLEO §5 y PAQUETE §4 no nombran la v6.
- **Fuentes:** `spec/NUCLEO.md:1-5 'normativa vigente desde §407 (RFC-0005, etapa E1)'`; `spec/NUCLEO.md:36-45 (cuatro clases: NÚCLEO, REFERENCIA, LIBRO, REGISTRO)`; `spec/NUCLEO.md:66-67 'LIBRO 5, NÚCLEO 97, REFERENCIA 7, REGISTRO 15' (76 en verify + 48 en hash)`; `spec/NUCLEO.md:186-200 (digest_of_proof, sellos, commit_operation, reverificar = REGISTRO)`; `spec/NUCLEO.md:203-227 (familias HASH, CABEZA, FIRMA, ACUSES, MMR, INCLUSIÓN, CONGELADOS, CUENTAS)`; `spec/NUCLEO.md:1-7`; `spec/NUCLEO.md:36-45`; `spec/NUCLEO.md:50`

### L11b

**CONFIRMADA** · gravedad baja · cambió con el árbol (S566–S582)

- **Borrador:** RFC-0005 (núcleo congelado) está PROPUESTO
- **Lo que se puede afirmar:** El RFC-0005 está PROPUESTO. Aun así, NUCLEO.md es normativa vigente desde §407, con E1 a E4 selladas. El criterio E5, una segunda implementación independiente, no existe y queda fuera del árbol. El RFC-0010 ya está ACEPTADO (§577).
- **Fuentes:** `spec/rfc/0005-nucleo-congelado.md:3 '- **Estado:** PROPUESTO'`; `spec/rfc/0005-nucleo-congelado.md:10-18 (E1-E4 selladas; E5 'fuera del árbol')`; `README.md:185 '0005 (el núcleo congelado) y 0010 (el recibo de recepción) propuestos'`; `spec/rfc/0005-nucleo-congelado.md:3-18`; `README.md:193`; `spec/NUCLEO.md:3`

### L11c

**CONFIRMADA** · gravedad baja

- **Borrador:** Versión del protocolo zkssl/0.4
- **Lo que se puede afirmar:** El cable está en zkssl/0.4 desde §538 (RFC-0009 E3b-2). Antes rigieron la 0.3 desde §354 y la 0.2 desde §209. Es un eje distinto del formato de cabeza firmada, que hoy es formatVersion = 6. Varios comentarios y párrafos aún dicen 0.3 y no deben citarse.
- **Fuentes:** `spec/RPC.md:1 y :1171-1172 'La versión vigente es zkssl/0.4 desde §538'`; `crates/zk-ssl-node/src/main.rs:1593 zkssl_protocolVersion -> "zkssl/0.4"`; `crates/zk-ssl-wire/src/openrpc.rs:155`; `crates/zk-ssl-verify/src/lib.rs:158-161 (comentario rancio: 'el cable sigue en zkssl/0.3')`; `crates/zk-ssl-node/src/main.rs:1783`; `crates/zk-ssl-wire/src/openrpc.rs:159`; `spec/RPC.md:1, 1243`; `crates/zk-ssl-verify/src/lib.rs:158-166`

### L4b

**CONFIRMADA** · gravedad baja

- **Borrador:** Con prueba de extensión (consistency proof) por RPC
- **Lo que se puede afirmar:** zkssl_consistencyProof({oldSize}) devuelve {available, mmrSize, camino?, reason?}. El camino de tamaño t lo firma la cabeza siguiente, a lo sumo un latido después. Hay tres respuestas sin camino: oldSize 0, identidad y 'va por detrás' (spec/RPC.md:985-1001).
- **Fuentes:** `spec/RPC.md:88 'zkssl_consistencyProof \| {oldSize: Q} \| {available, mmrSize: Q, camino?: Digest[], reason?}'`; `spec/RPC.md:918-944 (tres negativas: oldSize 0, identidad, 'va por detrás')`; `README.md:130 fila 3 'medida (spec/RPC.md, zkssl_consistencyProof)'`; `AUDITORIA.md §293`; `spec/RPC.md:88`; `spec/RPC.md:985-1011`; `README.md:130`

### L4c

**CONFIRMADA** · gravedad baja

- **Borrador:** El verificador comprueba la extensión sin el nodo
- **Lo que se puede afirmar:** El paquete de extensión ({v:1, tipo:'extension', vieja, nueva, camino}) se verifica sin el nodo en tres pasos. Primero, las dos cabezas (v3 a v6) recomponen su epochDigest y verifican su firma. Segundo, deben tener la misma publicKey. Tercero, mmr::verificar_consistencia ha de dar que la cima nueva extiende a la vieja (crates/zk-ssl-verify/src/main.rs:608-640).
- **Fuentes:** `crates/zk-ssl-verify/src/main.rs:578-601 verificar_extension: 1/3 dos cabezas v3+ recomponen su digest y verifican su firma; 2/3 misma publicKey; 3/3 mmr::verificar_consistencia(ci`; `spec/PAQUETE.md:82-93 y :421-423`; `crates/zk-ssl-cli/src/witness.rs:59-80 (el testigo: canal de historia, 'no-extiende' SE DETIENE, 'por-detras' anota y sigue)`; `crates/zk-ssl-verify/src/main.rs:476-602`; `crates/zk-ssl-verify/src/main.rs:608-640`; `spec/PAQUETE.md:82-93`; `crates/zk-ssl-cli/src/witness.rs:59-80`

### L5a

**CONFIRMADA** · gravedad baja · cambió con el árbol (S566–S582)

- **Borrador:** El paquete de evidencia contiene cabeza firmada, acuse con camino y cofirmas
- **Lo que se puede afirmar:** Ese es el paquete de POSICIÓN v2 (spec/PAQUETE.md:60-80), con la cabeza (el result de zkssl_signedEpochHead), el acuse opcional {seq, hashPrueba, s, camino} de zkssl_ackPath y las cofirmas opcionales tal como las sirve zkssl_cosigs. El binario acepta once formas: posición v1 y v2, extensión, consumo, conflicto, rechazo, edad, cobro_pendiente, pago_en_curso, prenda y, desde §573, completitud.
- **Fuentes:** `README.md:114-115 'Un paquete de evidencia lleva una cabeza firmada, un acuse con su camino y las cofirmas'`; `spec/PAQUETE.md:41-80 (v1: cabeza + acuse opcional {seq, hashPrueba, s, camino}; v2: + cofirmas de zkssl_cosigs tal cual)`; `spec/PAQUETE.md:35-37 'El binario acepta diez objetos'`; `README.md:114-115`; `spec/PAQUETE.md:35-80`; `spec/PAQUETE.md:359-399`; `crates/zk-ssl-verify/src/main.rs:164-187`

### L7a

**CONFIRMADA** · gravedad baja

- **Borrador:** La no-reescribibilidad necesita una cabeza autenticada externamente
- **Lo que se puede afirmar:** El repositorio lo reconoce: hoy un tercero verifica la firma contra la clave que le entrega el mismo nodo, lo cual es circular. No hay ancla anterior al primer encuentro, y §246 lo resume así: no hay clave de producción que anclar. La custodia de la clave es declarada y no comprobada, salvo la modalidad 'fichero'.
- **Fuentes:** `README.md:152-155 'le falta un ancla anterior al primer encuentro y una custodia de clave comprobada, no sólo declarada'`; `SECURITY.md:122-170 ('Lo que TOFU NO da: el primer encuentro'; 'NO HAY CLAVE QUE ANCLAR' §246; 'Es lo que Certificate Transparency resuelve publicando las claves de log fuera del l`; `SECURITY.md:176-180 custodia declarada, solo 'fichero' comprobable`; `README.md:157-162`; `SECURITY.md:122-170`; `SECURITY.md:171-175`; `spec/RPC.md:694-696`

### L7b

**CONFIRMADA** · gravedad baja

- **Borrador:** Los testigos fijan la clave por TOFU
- **Lo que se puede afirmar:** El testigo de referencia (zk-ssl-cli witness, §245) fija la clave en el primer encuentro y se detiene ante una vista dividida (mismo índice, digest distinto) o un cambio de clave. Si lo opera el propio operador, no prueba nada.
- **Fuentes:** `crates/zk-ssl-cli/src/witness.rs:1-45 ('fija la clave que ve la primera vez'; se DETIENE ante vista dividida o cambio de clave)`; `SECURITY.md:122-138`; `README.md:112-113`; `crates/zk-ssl-cli/src/witness.rs:1-45`; `SECURITY.md:122-143`; `README.md:113-114`

### L9c

**CONFIRMADA** · gravedad baja

- **Borrador:** Hay un kit del verificador documentado
- **Lo que se puede afirmar:** doc/KIT.md, con traducción en doc/KIT_EN.md, guía cuatro comprobaciones sin red con el tarball de la release v0.2.0. Son una posición en VERDE, un paquete manipulado en ROJO con la regla nombrada, un conflicto entre dos libros y un intercambio de libros rechazado. Sus vectores son históricos (cabezas v3).
- **Fuentes:** `doc/KIT.md:1-134 (descarga, sha256sum -c SHA256SUMS, cuatro pasos, conformidad.sh)`; `README.md:44-75`; `AUDITORIA.md §442-§443`; `doc/KIT.md:1-134`; `doc/KIT_EN.md`

## Bloque C — pago en dos fases, custodia, qué revela una prueba, conservación, uso único, recuperación y poderes del operador (Arqueo)

### C10a

**FALSA** · gravedad alta

- **Borrador:** ECST-R: extendemos Arqueo para que el estado esté enmascarado mediante Pedersen Commitments
- **Lo que se puede afirmar:** Los compromisos de Pedersen se apoyan en el logaritmo discreto sobre curvas. Contradicen la tesis post-cuántica y «sin curvas» del núcleo (SECURITY.md:551), y CONTRIBUTING exige discutirlos antes con números (CONTRIBUTING.md:147-151). Presentarlos como extensión de Arqueo es falso: nada de eso está hecho ni es compatible con el núcleo tal como está.
- **Fuentes:** `CONTRIBUTING.md:149-151 ('No encaja sin discusión previa con números: sistemas de prueba que reintroduzcan ceremonias de confianza o supuestos no post-cuánticos contra la tesis del`; `SECURITY.md:508-518 (autoridad de gasto sin curvas; 'no hay curva que romper')`; `grep: 'Pedersen' aparece 0 veces en los .md del repo`; `CONTRIBUTING.md:147-151`; `SECURITY.md:551`; `SECURITY.md:553-559`

### C2a

**FALSA** · gravedad alta

- **Borrador:** Custodia cero garantizada por tipos: apply_send() solo acepta la prueba y compromisos públicos, haciendo técnicamente imposible que la clave secreta sea transmitida a la capa del ledger
- **Lo que se puede afirmar:** No hay barrera de tipos en la capa. SovereignLayer expone métodos públicos que reciben la clave de gasto: open_account(_wide), send, claim, refund(_v2), burn, audit, disclose_exact y prove_minimum (el RFC-0009 lo declara deuda en sus líneas 849-850). Lo que sí existe: (1) una superficie JSON-RPC sin ningún método de producción que reciba la clave (abrir cuenta viaja como publicId, viewId y leafSalt); (2) el encapsulado del SDK, donde Wallet::spend_key es privado y Wallet no implementa Serialize. Hasta el §538 la clave viajaba literal dentro de la prueba que apply_send acepta. La custodia es del cable y del SDK, no del tipo (SECURITY.md:476-516; spec/RPC.md:53-61).
- **Fuentes:** `crates/zk-ssl/src/accounts.rs:126 (pub fn open_account(&mut self, spend_key: BaseElement)) y :164 (open_account_wide(spend_key: Digest))`; `crates/zk-ssl/src/two_phase.rs:525, 579, 791, 1138 (refund, refund_v2, send, claim reciben spend_key)`; `crates/zk-ssl/src/burn.rs:19-21 y crates/zk-ssl/src/audit.rs:29-31, 96-98, 113-115 (burn, audit, disclose_exact, prove_minimum reciben spend_key; sin variante de cliente)`; `spec/rfc/0009-lo-que-revela-una-prueba.md:849-850 ('que audit, prove_minimum y burn reciban la clave de gasto como argumento de un método de la capa' = deuda nombrada y no pagada)`; `crates/zk-ssl/src/lib.rs:55 (el propio ejemplo de la API: layer.open_account(sk))`; `crates/zk-ssl-sdk/src/lib.rs:92-133 (Wallet.spend_key privado, pub(crate), sin Serialize)`; `crates/zk-ssl/src/accounts.rs:97-106`; `crates/zk-ssl/src/accounts.rs:126`

### C4c

**FALSA** · gravedad alta

- **Borrador:** el auditor obtiene certeza matemática sin acceso a las bases de datos
- **Lo que se puede afirmar:** Un tercero no obtiene «certeza matemática» de la conservación. Sin el nodo, el kit verifica cabezas firmadas (que comprometen totalSupply y las raíces), inclusión, extensión, cofirmas, consumos, causas de rechazo y, desde el RFC-0010, el sobre de completitud. No verifica ningún agregado de saldos. El registro guarda de envío, cobro, quema, reembolso y migración sólo el digest de la prueba, y de las operaciones delegadas (emisión, congelación, recuperación, gobernanza) sólo el sello de la autorización. Ninguna de esas pruebas es re-verificable por un tercero. El invariante Σ saldos + pendientes = suministro lo comprueba la capa del operador al reabrir, sobre saldos en claro (§379, §387–§394). Todo descansa además en la completitud de los AIR (sin especificación formal ni auditoría; SECURITY.md:304-318), en un verificador que el operador puede cambiar sin rastro (SECURITY.md:176-199) y en ~127 bits conjeturados (crates/zk-ssl/src/lib.rs:205-222).
- **Fuentes:** `crates/zk-ssl-verify/src/reverificacion.rs:44-47 ('Send, Claim, Burn, Refund y Migration asientan el resumen de una prueba REAL, y el registro no guarda la prueba: no seran recompu`; `AUDITORIA.md §379 (línea 28640: la conservación del agregado se comprueba AL ABRIR, en la capa)`; `crates/zk-ssl/src/tests.rs:2706 (a_supply_that_does_not_match_the_balances_is_detected_at_startup) y :2987 (balances_plus_pending_always_equal_total_supply; suma sobre l.records en`; `SECURITY.md:299-313 (§3.1: sin especificación formal del AIR; 'Un fallo de solidez es dinero falso invisible'; estado abierto)`; `SECURITY.md:176-199 (el operador puede cambiar el verificador y hoy eso es invisible)`; `README.md:36 (no auditado por terceros)`; `crates/zk-ssl/src/lib.rs:205-222 (proof_options: '127 bits conjeturados')`; `crates/zk-ssl-verify/src/reverificacion.rs:1-52`

### C5a

**FALSA** · gravedad alta

- **Borrador:** Detección de Reutilización de Etiquetas (Nullifiers): garantiza que ningún activo pueda ser gastado dos veces en el árbol de consumos
- **Lo que se puede afirmar:** El árbol de consumos no impide que un activo se gaste dos veces. Un consumo es una etiqueta pública y precomputable, H(dominio, identificador acordado). Por zkssl_publishConsumo el libro la publica sin prueba ni autorización, y no está atada a ningún pago (E5 fuera de alcance). La marca de prenda del RFC-0008 entra tras verificar una prueba, pero no bloquea el cobro ni el reembolso. El doble gasto de saldos lo impide el encadenamiento de raíces bajo el orden total del nodo único, y en el cobro la hoja del pendiente pasa a vacía en el circuito. El árbol de nulificadores de la vía de un paso se retiró (§32, §36): «evitado, no resuelto», y vuelve si se distribuye el orden (SECURITY.md:384-400; crates/zk-ssl/src/consumo.rs:11-18).
- **Fuentes:** `README.md:244 (árbol de nullificadores 'retirado con migración verificada', §32, §36)`; `AUDITORIA.md §32 (línea 2567) y §36 (línea 2951: retirada; ZKSSL3 -> ZKSSL4; 'El límite del cumpleaños sigue evitado, no resuelto')`; `SECURITY.md:379-395 (§3.4: la vía de producción en dos fases no usa nulificadores; reenvío = raíz obsoleta; 'cerrado en el nodo único, abierto para cualquier distribución')`; `crates/zk-ssl/src/consumo.rs:11-18 ('No prueba que el consumo corresponda a un pago ... Tampoco autentica a quien publica ... quien publica primero bloquea')`; `spec/rfc/0006-consumo-publicado.md:21 (E5, el atado en circuito: fuera del alcance) y :148-153`; `doc/README-CLI.md:26-33 (el doble gasto se cierra con nonce + hoja + árbol de pendientes; los nullifiers que existen son anti-replay de custodios)`; `README.md:251`; `AUDITORIA.md §32 (línea 2567)`

### C7a

**FALSA** · gravedad alta

- **Borrador:** Al forzar que el contador de recuperación sea público, el historial de gobernanza queda sujeto a la misma rigidez matemática que las transacciones, eliminando los canales laterales administrativos
- **Lo que se puede afirmar:** Los contadores de recuperación, congelación y gobernanza hacen contables esas intervenciones, pero no las impiden: «El contador no impide el abuso: nada en un circuito puede» (recovery.rs:12-19). Sólo la subida de recuperación prueba el +1 en circuito; la gobernanza lo comprueba en la capa. Ninguna de esas pruebas queda en el registro. Siguen abiertos canales administrativos fuera de toda prueba: el operador puede subir el cupo de custodios o cambiar la T de caducidad sin autorización (visible desde la cabeza v5 en paramsDigest), ordenar y cambiar el verificador sin rastro.
- **Fuentes:** `crates/zk-ssl/src/recovery.rs:12-19 ('El contador no impide el abuso: nada en un circuito puede. Lo hace contable')`; `crates/zk-ssl/src/governance.rs:96-103 ('La garantia se muda del circuito a esta funcion': el contador de gobernanza se comprueba en la capa, sin circuito)`; `crates/zk-ssl/src/governance.rs:42-56 (set_max_custodian_uses: 'No lo protege ninguna autorización ... Un operador puede subirlo y así anular la rotación')`; `spec/NUCLEO.md:265-268 (params_digest incluye max_custodian_uses y refund_ttl: el cambio es visible en la cabeza v5, no impedido)`; `SECURITY.md:176-199 (el operador puede cambiar el verificador sin dejar rastro)`; `SECURITY.md:113-115 y :221-226 (el operador ve, ordena, censura y omite)`; `crates/zk-ssl/src/recovery.rs:12-19`; `crates/zk-ssl/src/governance.rs:42-56`

### C9b

**FALSA** · gravedad alta

- **Borrador:** respetó las reglas del contrato inteligente
- **Lo que se puede afirmar:** Arqueo no tiene contratos inteligentes. Las reglas son circuitos AIR fijos en Rust (fork de winterfell) más las comprobaciones de la capa. Una prueba afirma que una transición cumple su AIR, y el cambio de verificador no queda registrado (doc/README-CLI.md:8-21; SECURITY.md:176-199).
- **Fuentes:** `doc/README-CLI.md:8-21 ('Esta capa no tiene EVM: es lógica Rust nativa sobre STARK/FRI')`; `grep en todo el repo: 0 apariciones de 'smart contract' / 'contrato inteligente'`; `SECURITY.md:182-187 (OpKind dice qué circuito usar; las reglas son el AIR, que es código)`; `doc/README-CLI.md:8-21`; `SECURITY.md:180-184`

### C10c

**SIN_FUENTE** · gravedad alta

- **Borrador:** ECST-R: claves de gasto por umbral, TEE, máquina de estados de migración de claves Healthy/Suspected/Quarantined/Revoked/Migrated, detección de anomalías
- **Lo que se puede afirmar:** Nada de ECST-R está implementado en ninguno de los dos repos. Lo más cercano: autorizaciones de 2 custodios para operaciones de custodia (no claves de gasto por umbral); recuperación asistida por dos custodios, que sustituye la identidad sin ninguna máquina de estados y copia el view_id y el leaf_salt viejos; y rotación de custodios por cupo de uso. La rotación de claves (entrada 52) sigue abierta, y no hay TEE ni detección de anomalías.
- **Fuentes:** `grep en ambos repos (Arqueo y hbs-state): 0 apariciones de Quarantined/Suspected/Healthy/TEE/enclave/SGX como diseño`; `crates/zk-ssl/src/lib.rs:84-91 (umbral fijo de 2 solo para operaciones de custodio, no para claves de gasto)`; `BACKLOG.md:3041-3053 (entrada 52: rotación de claves sin diseño único, abierta)`; `BACKLOG.md:1729-1740 (entrada 20: rotación de custodios por uso; recuperación con custodios; sin recifrado del libro)`; `SECURITY.md:212-214 (rotar la clave exige dos custodios)`; `crates/zk-ssl/src/lib.rs:84-91`; `BACKLOG.md:3041-3053`; `BACKLOG.md:1729-1740`

### C7b

**SIN_FUENTE** · gravedad alta

- **Borrador:** los circuitos ZK aplicados a primitivas administrativas cierran la brecha de confianza de los sistemas de gobernanza descentralizada
- **Lo que se puede afirmar:** Arqueo no es un sistema de gobernanza descentralizada, y ningún documento del repo dice que cierre esa brecha. La gobernanza es un conjunto inmutable que cambia el conjunto de custodios con 2 firmas. Si se compromete, la única salida es un libro nuevo, y el código dice que la circularidad «se traslada», no desaparece (crates/zk-ssl/src/governance.rs:18-25; AUDITORIA.md:53-56).
- **Fuentes:** `README.md:33 ('No es una cadena: un nodo, un escritor, sin consenso distribuido ni token')`; `AUDITORIA.md:53-56 ('Adversario NO contemplado: dos gobernadores comprometidos ... No hay salida salvo crear un ledger nuevo')`; `crates/zk-ssl/src/governance.rs:18-25 ('La circularidad no desaparece: se traslada')`; `CONTRIBUTING.md:149-154 (no encaja nada que presuponga una descentralización que el proyecto no tiene)`; `README.md:33`; `AUDITORIA.md:53-56`; `crates/zk-ssl/src/governance.rs:18-25`; `CONTRIBUTING.md:147-154`

### C10b

**PARCIAL** · gravedad media

- **Borrador:** ECST-R: ...o árboles de Merkle privados
- **Lo que se puede afirmar:** El estado de Arqueo ya es un árbol de Merkle disperso de hojas ocultantes basadas en hash: Rescue-Prime Rp64_256 sobre Goldilocks, con un leaf_salt derivado de la clave (entrada 50). Oculta frente a terceros, no frente al operador, que guarda los saldos en claro. MerkleConSal es otra cosa: el compromiso vectorial con sal (Blake3) que el probador STARK usa desde el §538 para ocultar la traza. La privacidad frente al operador (B11, operador ciego) está diseñada en el backlog y no construida.
- **Fuentes:** `crates/zk-ssl-hash/src/lib.rs:100-106, 145-169 (native_merge = permutación Rp64_256; hoja = merge(merge(merge(public_id, saldo), nonce), leaf_salt))`; `crates/zk-ssl/src/pending.rs:70-80 (compromiso de pendiente = merge(merge(receptor, sal), importe))`; `SECURITY.md:315-342 (§3.2: 'El compromiso de hoja ya ES ocultante — MEDIDO y RESUELTO (entrada 50)')`; `crates/zk-ssl-air/src/sal.rs:1-3, 36-40 (MerkleConSal: compromiso vectorial con sal Blake3 del probador STARK, no el árbol de estado)`; `SECURITY.md:228-233 (privacidad frente al operador: solo con operador ciego B11 o federación)`; `BACKLOG.md:2793-2797 (B11 operador ciego: va con la entrada 47, no construido)`; `crates/zk-ssl-hash/src/lib.rs:100-106`; `crates/zk-ssl-hash/src/lib.rs:145-169`

### C12a

**PARCIAL** · gravedad media · **el escéptico revocó el primer veredicto** (CONFIRMADA) · cambió con el árbol (S566–S582)

- **Borrador:** ECST 4.1: el operador del nodo puede censurar una transacción recibida u omitirla antes de su inclusión sin dejar rastro
- **Lo que se puede afirmar:** El operador puede censurar, pero desde el RFC-0010 (ACEPTADO en el §577) ya no siempre sin rastro. Toda operación que el nodo evalúa por applySend o applyClaim recibe un recibo de recepción, también si la capa la rechaza, y queda como hoja bajo la recepRoot de la cabeza v6 firmada. Si no se resuelve en N = 1.440 cabezas (aplicada, o rechazada con prueba portable), el titular forma sin nodo un sobre de completitud que da un ROJO nombrado. Es evidencia oponible firmada por el propio operador, no prueba criptográfica de ausencia. La censura sigue sin dejar rastro oponible cuando el nodo no emite recibo (no contesta, contesta sin recepcion o con -32603), cuando la operación entra por applyMany o zkssl_pledge, y en los métodos que no reservan recibo, como zkssl_openAccount o zkssl_publishConsumo. El titular lo sabe al instante, pero no puede probarlo (RFC-0010 D-E, D-F, D-H; spec/PAQUETE.md:359-399; README.md:136-156). El operador sigue decidiendo el orden.
- **Fuentes:** `README.md:33-34 ('puede omitir una operación sin dejar rastro') y :143 ('la censura no deja rastro')`; `SECURITY.md:115 y :221-226 (2.bis: orden y completitud, residuo nº 1)`; `spec/RPC.md:449-482 (receptionSeq, §253: dos titulares que cooperan DETECTAN la reordenación pero no pueden PROBARLA; 'Es un número que el nodo dice y que nada ata')`; `spec/rfc/0010-el-recibo-de-recepcion.md:66-70 ('La censura vive en el hueco entre recibir y aplicar, y hoy ese tramo no deja huella firmada')`; `spec/rfc/0010-el-recibo-de-recepcion.md:3-19`; `spec/rfc/0010-el-recibo-de-recepcion.md:98-101`; `spec/rfc/0010-el-recibo-de-recepcion.md:182-239`; `spec/rfc/0010-el-recibo-de-recepcion.md:250-266`

### C12b

**PARCIAL** · gravedad media · **el escéptico revocó el primer veredicto** (CONFIRMADA) · cambió con el árbol (S566–S582)

- **Borrador:** (pregunta) ¿Qué pretende cambiar el RFC-0010 (recibo de recepción, PROPUESTO)?
- **Lo que se puede afirmar:** El RFC-0010 (ACEPTADO en el §577, hito H5b) hace que toda operación que el nodo llega a evaluar por applySend o applyClaim, aunque la rechace, deje una hoja «recibo de recepción», recibo_digest(hashPrueba, era, n), bajo una raíz firmada en la cabeza v6 (recepRoot, recepCount). El recibo viaja en la respuesta y en el error.data. Hereda la firma al cerrar la era, y el camino se pide con zkssl_recepPath. Con él el titular forma, sin nodo, un sobre de completitud con tres veredictos: aplicada; rechazada con prueba; o NO RESUELTA en N = 1.440 cabezas firmadas, medidas por el índice XMSS. Hay un cuarto estado, «declarada, no probada», para las causas sin prueba portable. El ROJO es evidencia oponible, no prueba de ausencia: la censura sigue siendo posible, pero deja de ser gratuita. Residuo declarado: el operador que no emite recibo (D-H); applyMany y zkssl_pledge, fuera por decisión del §576; y la retención del registro, sin decidir (§580). Todas las etapas están selladas (§557–§574) (spec/rfc/0010-el-recibo-de-recepcion.md:3-19, 182-318; spec/PAQUETE.md:359-399).
- **Fuentes:** `spec/rfc/0010-el-recibo-de-recepcion.md:3 (Estado: PROPUESTO) y :26-33 (tabla: E1 sellada §556; E2a sellada §557-§559, §563; E2b sellada §562; E2c y E2d pendientes; E3-E5 propuesta`; `AUDITORIA.md §565 (línea 40946: E2c-1, el registro de recepción en crates/zk-ssl-node/src/registro_recepcion.rs, commit 9208b58)`; `spec/rfc/0010-el-recibo-de-recepcion.md:104-121 (D-C: la cabeza v6 firma (recepRoot, recepCount)) y :143-151 (D-E: cuenta lo que el nodo llegó a EVALUAR, incluidos los rechazos de `; `spec/rfc/0010-el-recibo-de-recepcion.md:153-169 (D-F: sobre de completitud con tres veredictos; N = 1.440 cabezas; el tercero es evidencia oponible, no prueba de ausencia)`; `spec/rfc/0010-el-recibo-de-recepcion.md:180-190 (D-H: residuo: la operación sin recibo sigue sin poder probarse)`; `spec/NUCLEO.md:261-264 (v6 y recibo_digest con el dominio RECEP_V1)`; `spec/rfc/0010-el-recibo-de-recepcion.md:3-19`; `spec/rfc/0010-el-recibo-de-recepcion.md:44-50`

### C1a

**PARCIAL** · gravedad media

- **Borrador:** send_materials() proporciona únicamente materiales públicos y caminos de autenticación
- **Lo que se puede afirmar:** zkssl_sendMaterials exige la clave de vista del remitente (§261) y devuelve caminos y raíces de los árboles de cuentas, congelados y pendientes, la posición de pendiente reservada, el límite regulatorio y el suministro. Devuelve además la vista privada del remitente: saldo, nonce y leaf_salt, que el operador ya conoce porque guarda los saldos en claro. El nodo recibe en claro el receptor, el importe y la sal. Los caminos no revelan saldos a terceros sólo porque la hoja lleva el leaf_salt del §117 (crates/zk-ssl/src/client.rs:222-270; spec/RPC.md:116, 818-854, 900-908). El «sólo datos públicos» es la fórmula del propio README (README.md:107-109), no una propiedad literal.
- **Fuentes:** `crates/zk-ssl/src/client.rs:220-270 (send_materials_inner devuelve SendMaterials con sender: AccountView, sender_path, frozen_path, pending_path, pending_position, receiver_id, reg`; `crates/zk-ssl/src/client.rs:67-76 (AccountView lleva public_id, balance, nonce y leaf_salt)`; `spec/RPC.md:115 (zkssl_sendMaterials params {sender, viewKey, receiverId, amount, salt})`; `spec/RPC.md:832-840 (§261: sendMaterials exige la viewKey de la cuenta cuyo camino se devuelve; error -32004)`; `spec/RPC.md:750-786 ('Lo que un camino expone: enumeración del saldo'; el salt de §117 es lo único que lo impide)`; `crates/zk-ssl-node/src/main.rs:2328-2336 (exige_credencial antes de tocar la capa)`; `crates/zk-ssl/src/client.rs:222-270`; `crates/zk-ssl/src/client.rs:61-76`

### C1b

**PARCIAL** · gravedad alta

- **Borrador:** prove_send() se ejecuta exclusivamente en la máquina del cliente con la clave secreta
- **Lo que se puede afirmar:** prove_send y prove_claim son funciones libres que no tocan la capa. El SDK (Account::pay/claim) las ejecuta en la máquina del titular con su clave ancha, y el nodo JSON-RPC no genera pruebas de pago. El código no impone dónde se ejecutan: la capa conserva SovereignLayer::send/claim, que reciben una clave estrecha de 64 bits y prueban dentro de la capa. Sólo los usan tests y métricas, y no llevan ninguna marca (two_phase.rs:791, 866-867, 1138). «Exclusivamente en la máquina del cliente» es una propiedad del despliegue con el SDK, no del código (client.rs:330-346; AUDITORIA §33).
- **Fuentes:** `crates/zk-ssl/src/client.rs:331-346 (prove_send es una función libre; su doc: 'SovereignLayer::send hace lo mismo, pero es un método de la capa que recibe la clave. Esa forma no im`; `crates/zk-ssl/src/two_phase.rs:791 (pub fn send(&self, spend_key: BaseElement, ...) genera la misma prueba dentro de la capa)`; `crates/zk-ssl-sdk/src/lib.rs:1-18 y :207 (Account::pay: materiales del nodo, prueba EN LOCAL)`; `AUDITORIA.md §33 ('La propiedad puede seguir siendo cierta; lo que falta es enseñarla')`; `crates/zk-ssl/src/client.rs:330-346`; `crates/zk-ssl/src/client.rs:473`; `crates/zk-ssl/src/two_phase.rs:791-793`; `crates/zk-ssl/src/two_phase.rs:856-878`

### C1c

**PARCIAL** · gravedad alta

- **Borrador:** apply_send() recibe y verifica la prueba sin haber tenido acceso jamás a la clave privada
- **Lo que se puede afirmar:** apply_send no recibe la clave de gasto como argumento. Recibe el recibo (prueba, entradas públicas, compromiso y aviso), el estado declarado en claro (identidad, saldo y nonce) y el importe. Hasta el §538 (zkssl/0.3 y anteriores), la prueba que recibía llevaba literal la clave de gasto, y el nodo tenía acceso a ella. Desde el §538 (RFC-0009 E3b-2, zkssl/0.4) la suite de E2 mide cero literales. Lo deducible de las aperturas lo acota D-I, y ni el fork de winterfell ni la construcción están auditados (SECURITY.md:476-516; spec/RPC.md:53-61).
- **Fuentes:** `crates/zk-ssl/src/two_phase.rs:1118-1135 (apply_send(receipt, sender_index, sender_state, amount): ningún argumento es la clave)`; `crates/zk-ssl/src/commitment.rs:71-75 (ClientState = {public_id, balance, nonce} en claro)`; `SECURITY.md:441-481 (§3.bis: hasta el §538 cada prueba abría 42 filas de la traza en claro y la clave de gasto salía literal en envío, cobro y prenda)`; `spec/RPC.md:55-61 ('Entre el §521 y el §538 la implementación no cumplía este principio ... el nodo las recibía')`; `spec/rfc/0009-lo-que-revela-una-prueba.md:96-99 (confianza residual desde §538: del testigo no sale ningún literal; lo deducible lo acota D-I, no lo mide E2)`; `crates/zk-ssl/src/two_phase.rs:1118-1135`; `crates/zk-ssl/src/commitment.rs:70-75`; `SECURITY.md:476-516`

### C4a

**PARCIAL** · gravedad alta

- **Borrador:** Circuito STARK que demuestra que la suma de saldos de salida es idéntica a la de entrada
- **Lo que se puede afirmar:** No hay un circuito de «suma de salidas = suma de entradas»: el modelo es de cuentas y en dos fases. Cada transición prueba en circuito su conservación local. En el envío, saldo_nuevo = saldo − importe, y el compromiso del pendiente lleva ese mismo importe, con el suministro sin cambios. En el cobro, saldo_nuevo = saldo + importe, y la hoja del pendiente queda vacía. En la emisión el suministro sube exactamente el importe, sin pasar del tope; en la quema baja; en la recuperación el saldo no cambia. El invariante global Σ saldos + Σ pendientes = suministro no es un circuito: lo comprueba la capa al abrir el libro (AUDITORIA §379).
- **Fuentes:** `crates/stark-experiment/src/circuit_send.rs:1092 (C_BALANCE: bal_new = bal - amt) y :1101 (C_SUPPLY: supply_new = supply_old)`; `crates/stark-experiment/src/circuit_claim_v2.rs:856-858 (bal_new = bal + amt; suministro constante)`; `crates/stark-experiment/src/circuit_send.rs:708-724 (SendPublicInputs: root_old, root_new, frozen_root, pending_root_old/new, amount, regulatory_limit, supply_old, supply_new)`; `crates/zk-ssl/src/two_phase.rs:1325 (el cobro vacía la hoja del pendiente: 'Consumido')`; `AUDITORIA.md §379 (línea 28654: 'El AIR guarda la conservacion POR TRANSICION — nueve testigos negativos')`; `crates/stark-experiment/src/circuit_send.rs:1091-1101`; `crates/stark-experiment/src/circuit_send.rs:1198`; `crates/stark-experiment/src/circuit_send.rs:708-724`

### C4b

**PARCIAL** · gravedad alta

- **Borrador:** ...sin revelar los saldos individuales
- **Lo que se puede afirmar:** Frente a terceros que sólo ven una prueba, desde el §538 no sale literal ningún saldo; antes del §538, el envío y el cobro publicaban el saldo antes y después. El importe es entrada pública de las pruebas de envío y cobro. Por metadatos se ven el emisor o el receptor y la posición del pendiente. Frente al operador no hay privacidad: ve todos los saldos (README.md:34; SECURITY.md:201-210; RFC-0009 D-A, D-B).
- **Fuentes:** `README.md:34 ('El operador ve todos los saldos') y :141`; `spec/rfc/0009-lo-que-revela-una-prueba.md:96-99 (desde §538: quien recibe una prueba ve el enunciado público, las raíces y lo que el recibo lleva en claro; ningún literal del testi`; `spec/rfc/0009-lo-que-revela-una-prueba.md:111 (fila envío antes de §538: salían la clave, emisor, receptor, saldo antes y después, sal, leaf_salt; el importe es público por diseño)`; `crates/stark-experiment/src/circuit_send.rs:719 (amount es entrada pública)`; `SECURITY.md:201-210 (metadatos: un envío revela emisor, importe y notice.position; un cobro, receptor, importe y la misma posición)`; `README.md:33-34`; `README.md:145`; `spec/rfc/0009-lo-que-revela-una-prueba.md:96-99`

### C6b

**PARCIAL** · gravedad alta

- **Borrador:** recovery: vaciado atómico de la clave antigua
- **Lo que se puede afirmar:** No hay vaciado de una cuenta A hacia otra B. La misma hoja, en la misma posición, cambia de identidad (id_viejo → id_nuevo) con el mismo saldo y el nonce + 1, y la clave vieja pierde la autoridad de gasto (test recovery_locks_out_the_compromised_key). Se copian el view_id y el leaf_salt derivados de la clave vieja. Deducido de la lectura, sin medir: la clave comprometida sigue autenticando la vista de la cuenta y conoce el salt de la hoja, y la nueva no reproduce la credencial de vista que el nodo exige desde el §261. El test que el código cita para esta costura no existe. Tampoco hay protección frente a un gasto del atacante anterior a la recuperación (crates/zk-ssl/src/recovery.rs:84-105; crates/stark-experiment/src/circuit_recovery.rs:57-59).
- **Fuentes:** `crates/zk-ssl/src/recovery.rs:84-106 (misma posición, public_id nuevo, nonce+1; view_id y leaf_salt de la clave VIEJA se copian: 'COSTURA 49-A <-> 52 ... el recuperado conserva una`; `crates/zk-ssl/src/client.rs:123-133 (account_view_authenticated compara la clave de vista contra el view_id guardado)`; `crates/zk-ssl/src/tests.rs:2070 (recovery_locks_out_the_compromised_key: la clave vieja ya no gasta)`; `crates/zk-ssl/src/recovery.rs:101 cita el test 'recovery_deja_view_id_viejo', que no existe en el árbol (grep sin definición)`; `crates/stark-experiment/src/circuit_recovery.rs:57-62 ('Nada impide una carrera: si el atacante gasta antes ... el dinero se va')`; `BACKLOG.md:3041-3053 (entrada 52: rotación, abierta)`; `crates/zk-ssl/src/recovery.rs:84-105`; `crates/zk-ssl/src/client.rs:113-133`

### C6c

**PARCIAL** · gravedad baja · **el escéptico revocó el primer veredicto** (CONFIRMADA)

- **Borrador:** recovery: incremento de un contador público de recuperación (Count_B = Count_A + 1)
- **Lo que se puede afirmar:** La recuperación incrementa en uno un contador GLOBAL de recuperaciones del libro (meta:recoveries), no un contador por identidad. La subida lo prueba en circuito como entradas públicas (count_viejo, count_nuevo = count_viejo + 1), y el contador entra en el compromiso que firman los dos custodios, donde hace de nonce anti-repetición (§393). No es un campo de la cabeza firmada ni del RPC, y la prueba que lo asierta no se guarda. Un tercero lo obtiene contando las entradas 'Recovery' del registro público (crates/stark-experiment/src/circuit_recovery_climb.rs:124-125, 325-333; crates/zk-ssl/src/recovery.rs:30-36, 79-81, 176).
- **Fuentes:** `crates/stark-experiment/src/circuit_recovery_climb.rs:120-125 (C_COUNT: 'EL CONTADOR INCREMENTA EXACTAMENTE EN UNO') y :325-333, 638-647 (recovery_count_old/new asertados como entr`; `crates/zk-ssl/src/recovery.rs:79-81, 147-151 (count_new = count_old + 1, dentro de commit_operation(OP_RECOVERY, ...))`; `AUDITORIA.md §393 (línea 29600: los contadores son el NONCE anti-repetición; load los deriva del registro)`; `spec/RPC.md:78 (la cabeza no lleva el contador; spec/RPC.md:84 zkssl_logEntries sin credencial, kind 'Recovery')`; `crates/stark-experiment/src/circuit_recovery_climb.rs:67-70`; `crates/stark-experiment/src/circuit_recovery_climb.rs:124-125`; `crates/stark-experiment/src/circuit_recovery_climb.rs:325-333`; `crates/stark-experiment/src/circuit_recovery_climb.rs:638-647`

### C6d

**PARCIAL** · gravedad media

- **Borrador:** recovery: doble autorización administrativa
- **Lo que se puede afirmar:** La doble autorización no está en el circuito de recuperación que usa la capa (circuit_recovery_climb), sino en la capa. apply_recovery_delegated exige dos pruebas de circuit_threshold_single_nullifier, de dos custodios distintos del conjunto de custodios; la gobernanza no sirve. Cada prueba está atada a esta transición (raíces y contadores) y consume cupo de custodios. El umbral es fijo en 2, sin k-de-n. El repo declara la garantía como «dos claves comprometidas, no dos voluntades independientes». Nada prueba que el nuevo titular sea legítimo. En el nodo ejecutable, las claves de custodio son constantes de prueba (crates/zk-ssl/src/recovery.rs:147-170; circuit_recovery_climb.rs:22-24; lib.rs:84-91).
- **Fuentes:** `crates/zk-ssl/src/recovery.rs:147-170 (verify_threshold_pair con CUSTODIAN_DOMAIN y custodian_set_root; operación = commit_operation(OP_RECOVERY, root_old‖root_new‖count_old‖count_`; `crates/stark-experiment/src/circuit_threshold_single_nullifier.rs:636-698 (dominio, raíz, operación, nulificadores distintos, pruebas válidas)`; `crates/zk-ssl/src/recovery.rs:410 (the_same_custodian_twice_cannot_recover) y :450 (governance_keys_cannot_recover)`; `crates/zk-ssl/src/lib.rs:84-91 ('No hay umbral configurable ... dos claves comprometidas en vez de una, no dos voluntades independientes')`; `crates/stark-experiment/src/circuit_recovery.rs:51-56 ('El circuito no verifica que el nuevo titular sea legítimo')`; `crates/zk-ssl/src/tests_support.rs:33-43 (custodios de la suite = constantes públicas)`; `crates/stark-experiment/src/circuit_recovery_climb.rs:1-6`; `crates/stark-experiment/src/circuit_recovery_climb.rs:22-24`

### C6e

**PARCIAL** · gravedad alta · **el escéptico revocó el primer veredicto** (FALSA)

- **Borrador:** recovery: entradas públicas = raíces de la identidad antigua A y nueva B
- **Lo que se puede afirmar:** Las entradas públicas de la subida de recuperación son cuatro: la raíz del árbol de cuentas antes y después (el estado entero, que compromete la hoja con la identidad vieja o con la nueva) y el contador global de recuperaciones antes y después. Las identidades vieja y nueva son testigo. La nueva viaja en claro como argumento de apply_recovery_delegated, y la vieja salía literal en la prueba hasta el §538 (crates/stark-experiment/src/circuit_recovery_climb.rs:325-340; RFC-0009:133).
- **Fuentes:** `crates/stark-experiment/src/circuit_recovery_climb.rs:325-333 (RecoveryClimbPublicInputs = {root_old, root_new, recovery_count_old, recovery_count_new})`; `crates/stark-experiment/src/circuit_recovery_climb.rs:59-62 (COL_ID_OLD, COL_ID_NEW son columnas de testigo)`; `crates/stark-experiment/src/circuit_recovery.rs:439-448 (el circuito completo añade custodian_set_root; no lo usa la capa)`; `spec/rfc/0009-lo-que-revela-una-prueba.md:127 (subida de la recuperación: antes del §538 salían literales la identidad vieja, el saldo y el leaf_salt; la identidad nueva es pública`; `crates/stark-experiment/src/circuit_recovery_climb.rs:325-340`; `crates/stark-experiment/src/circuit_recovery_climb.rs:59-63`; `crates/stark-experiment/src/circuit_recovery.rs:439-448`; `crates/zk-ssl/src/recovery.rs:108-138`

### C8

**PARCIAL** · gravedad alta

- **Borrador:** Claim 2 ECST: la clave secreta permanece en la máquina del cliente, haciendo técnicamente imposible que el operador emita transiciones no autorizadas sin una firma/prueba válida
- **Lo que se puede afirmar:** Hay que acotarlo. Gastar, cobrar, quemar y reembolsar exigen una prueba con la clave del titular. Emitir, congelar y recuperar exigen dos custodios, y cambiar el conjunto de custodios dos miembros de gobernanza. El operador, solo, puede abrir cuentas a cero, publicar consumos, ajustar el cupo y la T, y ordenar. Cuatro cosas quitan el «técnicamente imposible»: (a) hasta el §538 las pruebas entregaban al nodo las claves de gasto y de custodio; (b) el operador puede sustituir el verificador sin dejar rastro; (c) la solidez de los AIR no está verificada formalmente ni auditada, y la posición de congelados no está atada en el AIR; (d) de las operaciones delegadas sólo queda el sello de la autorización, no las pruebas. Además, en el nodo ejecutable actual los custodios son claves de prueba públicas (crates/zk-ssl/src/lib.rs:41-49; SECURITY.md:176-199, 304-318, 416-434, 492-496).
- **Fuentes:** `crates/zk-ssl/src/lib.rs:41-49 (tabla: gastar exige autoridad de gasto; emitir exige 'Dos custodios demostrados en circuito (ante el operador: §538)')`; `spec/rfc/0009-lo-que-revela-una-prueba.md:118 y SECURITY.md:457-461 (hasta el §538, tras una sola operación delegada el nodo tenía las claves de dos custodios y podía autorizar la `; `crates/zk-ssl/src/accounts.rs:120-125 (abrir cuenta no necesita prueba) y crates/zk-ssl/src/consumo.rs:128-134 (publicar un consumo: sin prueba y sin autorización)`; `SECURITY.md:176-199 (cambiar el verificador redefine qué es una transición válida, sin rastro)`; `SECURITY.md:299-313 y :411-429 (AIR sin especificación formal; §3.6: la posición de congelado no está atada en el AIR, cerrado solo en la capa)`; `crates/zk-ssl-node/src/main.rs:1156-1167 + crates/zk-ssl/src/tests_support.rs:33-49 (el único nodo que arranca usa custodios de prueba con claves públicas)`; `crates/zk-ssl/src/lib.rs:41-49`; `README.md:132`

### C9a

**PARCIAL** · gravedad media

- **Borrador:** Arqueo asume un dictador de bloque
- **Lo que se puede afirmar:** Arqueo asume un único operador que ordena todas las operaciones (un nodo, un escritor), no una cadena de bloques. Las unidades son transiciones en un registro encadenado, cabezas de época firmadas con XMSS a cada latido y lotes applyMany que arma el agregador. «Dictador de bloque» es terminología ajena al repo (README.md:33; SECURITY.md:29-31).
- **Fuentes:** `README.md:33 ('un nodo, un escritor, sin consenso distribuido ni token')`; `SECURITY.md:29-31 ('el operador ve el estado, ordena las operaciones y puede censurar')`; `README.md:111-114 (cabeza de época firmada; no bloques)`; `spec/RPC.md:119, 249-263 (applyMany: N operaciones contra una raíz de arranque)`; `README.md:33`; `SECURITY.md:29-31`; `README.md:111-114`; `spec/RPC.md:120`

### C9c

**PARCIAL** · gravedad media

- **Borrador:** casos de uso: infraestructuras financieras centralizadas, registros de propiedad
- **Lo que se puede afirmar:** USE_CASES lista libros cerrados donde el operador emite y retira la unidad. Seis dominios están revisados: devolución de depósitos, garantías de origen y derechos de emisión, monedas comunitarias y comunidades energéticas, préstamo bibliotecario digital y salvaguarda de fondos de clientes. El resto son candidatos sin medir. Encajan los registros de derechos y la compensación entre operadores y entre administraciones; en parte, los registros de valores de bajo volumen. No encajan las entidades de contrapartida central, y la moneda de banco central es un argumento, no un caso de uso. Es un prototipo de 1,5-1,9 TPS medidos (doc/USE_CASES.md:46-144).
- **Fuentes:** `doc/USE_CASES.md:16-18 (la unidad nace y muere dentro del libro)`; `doc/USE_CASES.md:42-105 (seis dominios revisados: devolución de depósitos, garantías de origen y derechos de emisión, monedas comunitarias y comunidades energéticas, préstamo bibli`; `doc/USE_CASES.md:107-124 (encaja: registros de derechos, compensación entre operadores y entre administraciones; en parte: registros de valores de bajo volumen; NO encaja: entidade`; `doc/USE_CASES.md:126-136 (moneda de banco central: argumento, no caso de uso; 'Arqueo is not a component of such a system')`; `SECURITY.md:27-28 (prototipo; no maneja dinero real)`; `doc/USE_CASES.md:16-18`; `doc/USE_CASES.md:46-113`; `doc/USE_CASES.md:115-132`

### C11

**CONFIRMADA** · gravedad baja

- **Borrador:** Prueba STARK generada en la máquina del pagador, sin ceremonia y sin curvas
- **Lo que se puede afirmar:** Correcto, con matices. La prueba la genera el titular con client::prove_send (SDK Account::pay), aunque la capa conserva SovereignLayer::send, que prueba con la clave. Es STARK/FRI sobre un fork de winterfell 0.13.1, con ocultación desde el §538, hashes Rescue-Prime Rp64_256 y Blake3 y campo Goldilocks. En la vía de pago no hay setup de confianza ni curvas elípticas. La seguridad son ~127 bits conjeturados, no demostrados, y «post-cuántico» significa sin supuestos de curva, no invulnerable (SECURITY.md:553-559). Ni los circuitos ni el fork están auditados.
- **Fuentes:** `README.md:24-27 ('pagos en dos fases cuya prueba STARK se genera en la máquina del pagador —sin ceremonia de setup y sin curvas—')`; `SECURITY.md:516-532 (sin firma clásica en la vía de pago; STARK/FRI solo usa hashes; arranque sin generar claves: 0,67 ms medidos)`; `crates/zk-ssl/src/lib.rs:205-222 (proof_options: 42 consultas, blowup 16, grinding 21, extensión cuadrática; '127 bits conjeturados')`; `spec/rfc/0009-lo-que-revela-una-prueba.md:207-227 (D-F: fork de winterfell 0.13.1 en crates/winter-*)`; `README.md:274-276 (crates/ceremony es código de Penumbra para la comparativa, no la vía de pago)`; `README.md:24-27`; `SECURITY.md:551`; `SECURITY.md:553-564`

### C2b

**CONFIRMADA** · gravedad alta

- **Borrador:** (pregunta) ¿Hay otra vía en el nodo o la capa donde el nodo reciba o custodie una clave de gasto?
- **Lo que se puede afirmar:** Sí hay vías en las que el nodo o la herramienta tiene claves. (1) dev_openSeeded: el nodo deriva una clave de gasto de una semilla pública y abre la cuenta con ella. (2) dev_fund y dev_freeze: el nodo firma como custodio con las claves de prueba de tests_support, que son constantes del fuente. (3) La CLI simulate ejecuta capa y cliente en un mismo proceso con claves de sandbox; su send/claim va por client::prove_send. (4) Los tests y las métricas usan SovereignLayer::send/claim, que reciben la clave; el nodo no los llama. (5) Hasta el §538, toda prueba recibida publicaba la clave de gasto o la de custodio. Los dev_* exigen feature dev y --dev. Sin la feature, el nodo no arranca: faltan raíces reales de custodios y gobernanza (crates/zk-ssl-node/src/main.rs:1336-1348, 3053-3118).
- **Fuentes:** `crates/zk-ssl-node/src/main.rs:2732-2745 (dev_openSeeded: 'let sk = ts::wide_key(p.seed.0); let index = l.open_account_wide(sk);' — el nodo deriva la clave de una semilla pública)`; `crates/zk-ssl-node/src/main.rs:2717-2730 y 2752-2767 (dev_fund y dev_freeze generan en el nodo las dos autorizaciones de custodio con ts::delegated_pair: el nodo tiene las claves d`; `crates/zk-ssl/src/tests_support.rs:33-49 (claves de custodio 0xC0570D1A..1E y de gobernanza 0x605E00..03, constantes en el fuente)`; `crates/zk-ssl-node/Cargo.toml:21-22 (default = ["dev"]; dev = ["zk-ssl/sandbox"])`; `crates/zk-ssl-node/src/main.rs:1156-1167 (sin feature dev: anyhow::bail!("build sin feature 'dev': pasar raíces de custodios/gobernanza reales (pendiente de flags --custodian-root/`; `crates/zk-ssl-cli/src/sandbox.rs:1-10, 44-48, 99 (simulate: claves derivadas de semilla pública, open_account_wide(key) en proceso; send/claim por client::prove_send)`; `crates/zk-ssl/src/tests_support.rs:82-110 (two_phase_transfer usa layer.send/claim con la clave: vía de los tests y de metrics.rs)`; `crates/zk-ssl-node/src/main.rs:3053-3059`

### C2c

**CONFIRMADA** · gravedad media

- **Borrador:** tools/check_publicadas.py: 'la vía de la capa send/claim' frente a 'la vía DOCUMENTADA send_materials -> client::prove_send -> apply_send'
- **Lo que se puede afirmar:** Las cifras de bytes publicadas se miden por la vía de la capa (SovereignLayer::send/claim, que recibe la clave y prueba dentro de la capa). El 2026-08-26 se midió que la vía del cliente daba los mismos bytes (66.739 y 66.692 B, antes del §538). Desde el §538 son una banda: envío 73.571..84.244 B, cobro 72.382..83.723 B y pago 145.953..167.967 B. El testigo que la ata prueba por la vía del cliente (tools/check_publicadas.py:37-48; crates/zk-ssl/src/metrics.rs:82-83, 683-731). La vía de la capa sigue viva en el código, aunque el cable no la exponga.
- **Fuentes:** `tools/check_publicadas.py:38-42 ('La cifra se mide sobre send/claim de la capa, que es la via que las cifras publicadas DESCRIBEN, y no sobre la del cliente')`; `tools/check_publicadas.py:44-48 ('MEDIDO el 2026-08-26: la via DOCUMENTADA -- send_materials -> client::prove_send -> apply_send -- da los MISMOS bytes, 66_739 y 66_692, en cinco r`; `crates/zk-ssl/src/metrics.rs:683 (fn los_dos_lados_del_pago_atan_la_banda)`; `tools/check_publicadas.py:37-48`; `crates/zk-ssl/src/metrics.rs:60-83`; `crates/zk-ssl/src/metrics.rs:121-127`; `crates/zk-ssl/src/metrics.rs:683-731`; `AUDITORIA.md §538 (línea 39270)`

### C3

**CONFIRMADA** · gravedad alta

- **Borrador:** README: 'la clave de gasto no viaja por la API y, desde el §538, tampoco sale literal en la prueba (RFC-0009 E3b-2, zkssl/0.4); entre el §521 y el §538 la prueba la publicaba' — los borradores ECST no lo mencionan
- **Lo que se puede afirmar:** El README dice que la clave de gasto no viaja por la API y que, desde el §538, tampoco sale literal en la prueba (RFC-0009 E3b-2, zkssl/0.4). El §521 (20-21 sep 2026) es cuando se midió la fuga y se dejó de negar, no cuando empezó. Winterfell 0.13 no oculta el testigo, y lo medido en la 0.3 fue esto: la clave de gasto salía literal en envío, cobro, prenda, auditoría y quema, y la del custodio en cada autorización delegada. Desde el §538 la suite de E2 mide cero literales, con la cota D-I sin auditar. Los vectores 0.3 conservados siguen llevando los literales (README.md:107-111; SECURITY.md:476-516; RFC-0009:44-61, 113-133). Un informe ECST debe incluirlo.
- **Fuentes:** `README.md:107-111`; `SECURITY.md:441-481 (§3.bis: medido en las sesiones 162-163, 20-21 sep 2026; 42 filas abiertas; clave de gasto en envío, cobro y prenda; clave de custodio en cada autorización dele`; `AUDITORIA.md §521 (línea 38231: 'las pruebas STARK de la casa (winterfell 0.13) NO ocultan su testigo'; un cazador deriva dos claves sandbox del catálogo de rechazos)`; `AUDITORIA.md §538 (línea 39270: encendido con MerkleConSal y m=64; zkssl/0.4; vectores 0.3 bajo spec/vectors/0.3/)`; `spec/rfc/0009-lo-que-revela-una-prueba.md:43-61 (Motivación) y :101-150 (tabla D-B)`; `https://github.com/facebook/winterfell (README, consultado con WebFetch: 'The current implementation provides succinct proofs but NOT perfect zero-knowledge'; ZK perfecto en 'Plann`; `SECURITY.md:476-516`; `AUDITORIA.md §521 (línea 38231)`

### C4d

**CONFIRMADA** · gravedad media

- **Borrador:** (pregunta) ¿Qué prueba el repo y a quién? README: 'Prueba conservación, no solvencia'; 'suministro = saldos + en vuelo'
- **Lo que se puede afirmar:** El repo prueba conservación dentro del libro, no solvencia ni la existencia de las unidades fuera de él. La fila 1 del README («medida, en vuelo y al reabrir», §387–§394) se apoya en las comprobaciones fail-closed del propio nodo al reabrir. Lo que llega a un tercero es la cabeza firmada con totalSupply y raíces. La solidez de cada transición la comprobó el nodo, y sus pruebas no quedan en el registro. No debe presentarse la fila 1 como una comprobación independiente del agregado hecha por el tercero.
- **Fuentes:** `README.md:18 ('Prueba conservación, no solvencia: las pruebas hablan del libro, no del mundo')`; `README.md:28-29 y :128 (fila 1: 'medida, en vuelo y al reabrir (AUDITORIA.md §387–§394)')`; `AUDITORIA.md §387 (línea 29201: root:pending guardada y comprobada al abrir; invariante sagrado sum(saldos)+sum(pendientes vivos)==suministro)`; `doc/USE_CASES.md:20-21, 27`; `README.md:18`; `README.md:28-29`; `README.md:128`; `AUDITORIA.md §387 (línea 29201)`

### C5b

**CONFIRMADA** · gravedad media

- **Borrador:** (pregunta) ¿Qué sustituyó al árbol de nulificadores y qué afirma exactamente?
- **Lo que se puede afirmar:** Al gasto no lo sustituyó el árbol de consumos, sino el encadenamiento de raíces (SECURITY §3.4). El árbol de consumos del RFC-0006 aporta la unicidad de una etiqueta acordada dentro de un libro. La capa rechaza el consumo repetido con ConsumoRepetido o ConsumoColision, y consRoot y consCount entran firmados en la cabeza v4 y siguientes. Entre libros, un tercero con las dos cabezas firmadas detecta después la misma etiqueta, sin nodos (E4a). Un nodo que custodia cabezas ajenas puede negarse a tramitarla (E4b). No hay prevención entre libros, ni atado en circuito, ni autenticación del publicador. El mismo árbol aloja la marca de prenda del RFC-0008 (spec/rfc/0006-consumo-publicado.md:13-22, 175-199; spec/NUCLEO.md:259-260).
- **Fuentes:** `README.md:29-30, 35, 129 (uso único dentro de un libro; entre libros 'detecta, no previene')`; `spec/rfc/0006-consumo-publicado.md:13-22 (E1 árbol y root:cons; E2 cabeza v4; E3 prueba portable; E4a detección entre libros; E4b bloqueo en tramitación; E4c registro autoritativo `; `spec/rfc/0006-consumo-publicado.md:175-199 (D-4: identificador acordado = gobernanza; quien publica primero bloquea; inobservabilidad ninguna; entre libros detección)`; `crates/zk-ssl/src/consumo.rs:134-151 (apply_consumo: ConsumoRepetido / ConsumoColision; OpKind::Consumo con el consumo como compromiso)`; `spec/NUCLEO.md:255-256 (v4 = merge(v3, merge(cons_root, cons_count)))`; `README.md:28-30`; `README.md:129`; `spec/rfc/0006-consumo-publicado.md:13-22`

### C6a

**CONFIRMADA** · gravedad baja

- **Borrador:** recovery: conservación estricta de saldo (S_B = S_A)
- **Lo que se puede afirmar:** Confirmado. circuit_recovery_climb, que es el que usa la capa, construye la hoja vieja y la nueva con la misma columna de saldo (rango de 64 bits): la recuperación no mueve dinero. El saldo es testigo, no entrada pública. La prueba la verifica el nodo, y el registro sólo guarda el sello de la autorización (crates/stark-experiment/src/circuit_recovery_climb.rs:8-20; crates/zk-ssl/src/recovery.rs:176).
- **Fuentes:** `crates/stark-experiment/src/circuit_recovery_climb.rs:8-12 ('Los dos carriles construyen su hoja con la MISMA columna COL_BAL, y el segmento de rango la descompone en 64 bits')`; `crates/stark-experiment/src/circuit_recovery_climb.rs:877 (test recovery_cannot_change_the_balance)`; `crates/zk-ssl/src/tests.rs:2109 (recovery_preserves_the_balance_and_the_supply)`; `crates/zk-ssl/src/recovery.rs:84-92 (updated.balance = account.balance)`; `crates/stark-experiment/src/circuit_recovery_climb.rs:8-20`; `crates/stark-experiment/src/circuit_recovery_climb.rs:877`; `crates/zk-ssl/src/tests.rs:2109`; `crates/zk-ssl/src/recovery.rs:84-88`

### C6f

**CONFIRMADA** · gravedad media

- **Borrador:** (pregunta) ¿Está la recuperación en la vía de producción?
- **Lo que se puede afirmar:** La recuperación existe como API de la capa: apply_recovery_delegated verifica la subida y el par de custodios. No está expuesta en el JSON-RPC del nodo, y en el árbol sólo la ejercitan los tests. La prueba de subida necesita el registro de la cuenta (saldo, leaf_salt y camino), así que en la práctica la genera el operador. La capa no usa el circuito circuit_recovery con el umbral dentro (crates/zk-ssl/src/tests_support.rs:504-518).
- **Fuentes:** `crates/zk-ssl/src/recovery.rs:52 (pub fn apply_recovery_delegated en la biblioteca)`; `crates/zk-ssl-node/src/main.rs (métodos despachados: 27 zkssl_* y dev_fund/dev_openSeeded/dev_freeze; ninguno de recuperación, gobernanza, quema ni reembolso)`; `grep: apply_recovery_delegated solo se llama desde crates/zk-ssl/src/tests.rs:2162, 2198 y tests_support.rs:527`; `crates/stark-experiment/src/lib.rs:83 (circuit_recovery completo: solo lo usan sus propios tests)`; `crates/zk-ssl/src/tests_support.rs:504-518 (la subida la construye quien tiene el registro: saldo, leaf_salt y camino)`; `crates/zk-ssl/src/recovery.rs:52-60`; `crates/zk-ssl/src/tests.rs:2162`; `crates/zk-ssl/src/tests.rs:2198`

### C7c

**CONFIRMADA** · gravedad alta · cambió con el árbol (S566–S582)

- **Borrador:** (pregunta) ¿Qué poderes administrativos conserva el operador o los custodios?
- **Lo que se puede afirmar:** Con 2 custodios, el emisor puede emitir (con el tope en circuito), emitir a pendiente, congelar o descongelar cualquier cuenta sin justificación en circuito y reasignar cualquier cuenta. Con 2 miembros de gobernanza se cambia el conjunto de custodios. El operador, solo, puede: ver todos los saldos; ordenar; abrir cuentas a cero; publicar consumos; fijar el cupo y la T sin autorización (visible en paramsDigest desde la cabeza v5); declarar un modelo de custodia de su clave que no se comprueba; y cambiar el verificador sin rastro. Censurar le sigue siendo posible. Desde el RFC-0010, si censura una operación de applySend o applyClaim a la que ya dio recibo, queda un rastro firmado. Sin recibo, en lote o prenda, no deja rastro oponible (RFC-0010 D-E, D-H). El titular no puede rotar su propia clave sin dos custodios (SECURITY.md:171-214).
- **Fuentes:** `crates/zk-ssl/src/mint.rs:24-146 (emisión: 2 custodios + subida; tope max_supply en circuito)`; `crates/zk-ssl/src/two_phase.rs:1559 (apply_mint_pending_delegated)`; `crates/zk-ssl/src/freeze.rs:81-165 (congelar/descongelar: 2 custodios; 'Nada justifica la congelación en el circuito', :28-29)`; `crates/zk-ssl/src/recovery.rs:8-10 ('dos custodios pueden reasignar cualquier cuenta')`; `crates/zk-ssl/src/governance.rs:104-170 (2 de gobernanza cambian la raíz de custodios y reinician el cupo)`; `crates/zk-ssl/src/two_phase.rs:255-259 (set_refund_ttl sin autorización)`; `SECURITY.md:171-175 (custodia de la clave de cabeza: declarada, solo 'fichero' comprobado)`; `SECURITY.md:212-214 ('Rotar la clave exige dos custodios')`

### C7d

**CONFIRMADA** · gravedad media

- **Borrador:** congelaciones sin política de caducidad (README: 'una política de caducidad para las congelaciones' falta)
- **Lo que se puede afirmar:** Una congelación dura hasta que dos custodios la levantan: no hay caducidad ni justificación en circuito. Una cuenta congelada no puede gastar, cobrar ni quemar. Los pendientes dirigidos a ella no los puede cobrar. Si tienen meta, su emisor puede reembolsarlos pasada la T; si son anteriores a la caducidad, quedan inmovilizados. La recuperación no levanta una congelación (crates/zk-ssl/src/freeze.rs:18-30; README.md:163-164; SECURITY.md:42-49).
- **Fuentes:** `README.md:156-157 ('Lo que falta ... una política de caducidad para las congelaciones')`; `crates/zk-ssl/src/freeze.rs:30 ('No hay caducidad. Dura hasta que alguien la levante.')`; `crates/zk-ssl/src/freeze.rs:18-27 (una cuenta congelada recibe hacia un pendiente que no puede cobrar: el dinero queda en el limbo; AUDITORIA §29)`; `crates/zk-ssl/src/tests.rs:1368 (recovery_does_not_lift_a_freeze)`; `README.md:163-164`; `crates/zk-ssl/src/freeze.rs:18-30`; `crates/zk-ssl/src/client.rs:153-158`; `crates/zk-ssl/src/tests.rs:1368`

## Bloque M — las cifras de las «evaluaciones» de los borradores, verificación formal y backends

### M1

**FALSA** · gravedad alta

- **Borrador:** Entorno de pruebas estandarizado: AMD EPYC 7763, 64 cores, 128 GB RAM, NVMe SSD
- **Lo que se puede afirmar:** Ninguna medición del repositorio declara un AMD EPYC 7763, 64 núcleos, 128 GB de RAM ni un disco NVMe. La máquina que el repo llama 'de referencia' es un portátil Intel Core i5-1135G7 (4 núcleos físicos, 8 hilos) bajo WSL2, con MemTotal 12.248.696 kB (RFC-0007:351-352; AUDITORIA §229, §234, §462). Las series anteriores (§89, §130-§131, FIVE_BACKENDS.md) sólo dicen 'WSL2 sobre un portátil de consumo', y §181 menciona un 'Ryzen del piloto', así que el hardware de esas series no está documentado (metrics.rs:24-28). La única otra máquina es un pod con RTX 5090, cuyo anfitrión no se identifica (cgroup de 15,3 núcleos y 99,65 GiB), y sólo se usó para la envoltura en zkVM (§306-§307). Los '64 núcleos' del repo son un supuesto de dimensionado (ESCALADO.md; §89.5; metrics.rs:338), no hardware medido.
- **Fuentes:** `grep -rniE 'EPYC\|7763\|128 GB\|128GB' en todo el repo (sin target/): 0 coincidencias; 'NVMe' sólo aparece como hipótesis ('Un NVMe rápido puede dar fsync de ~100 µs') en crates/zk`; `spec/rfc/0007-pruebas-sobre-el-estado-comprometido.md:351-352: 'en la máquina de referencia (i5-1135G7, 8 núcleos, 12.248.696 kB de RAM, WSL2)'`; `AUDITORIA.md §462 (l.34035-34036): 'i5-1135G7, 8 nucleos, MemTotal 12.248.696 kB, WSL2, rustc 1.97.1'`; `AUDITORIA.md §229 (l.16303, 16378-16394): 'portátil de cuatro núcleos'; 'cuatro pares usan cuatro de ocho hilos lógicos'; 'es lo que da un i5-1135G7'`; `crates/zk-ssl-guardian/src/lib.rs:60: 'Los umbrales salen de UNA máquina —WSL2 sobre un i5-1135G7—'`; `FIVE_BACKENDS.md:3-5: 'en la misma máquina (WSL2 sobre un portátil de consumo)'`; `AUDITORIA.md §305 (l.22763-22764): 'portatil de ocho nucleos SIN GPU, bajo WSL'`; `AUDITORIA.md §306 (l.22858-22882, 22945-22949): RTX 5090 alquilada; 'nproc y free mienten: veian 128 nucleos y 754 GiB ... el cgroup daba 15,3 nucleos y 99,65 GiB'`

### M2a

**FALSA** · gravedad alta

- **Borrador:** Tamaño de la prueba ZK (Pi_i) ~62.4 KB
- **Lo que se puede afirmar:** Desde el §538 (2026-09-24; zkssl/0.4, pruebas con ocultación y MerkleConSal), una prueba de ENVÍO midió 77.444-80.232 B y una de COBRO 76.192-79.736 B (15 muestras por eje), y un pago, que son dos pruebas, 155.337-159.329 B. Con el margen declarado del 5 %, las bandas publicadas son: envío 73.571-84.244 B, cobro 72.382-83.723 B y pago 145.953-167.967 B, es decir, 139,2-160,2 MiB por mil pagos (AUDITORIA §538; crates/zk-ssl/src/metrics.rs:62-83, 114-122). El tamaño ya no es determinista: depende de q y de la sal. Una ejecución suelta puede quedar fuera del rango medido y dentro de la banda: el 2026-09-30 un envío midió unos 80,8 kB. La banda no cubre los demás circuitos. '~62 KB' es prosa antigua, POR PRUEBA y anterior a la ocultación (log.rs:196-204; §31; §304). La cifra de 62,4 KB no tiene fuente.
- **Fuentes:** `grep '62[,.]4' en el repo: 0 coincidencias`; `crates/zk-ssl/src/metrics.rs:114-122: PUBLICADA_FECHA '2026-09-24'; 'envio 77.444..80.232 B, cobro 76.192..79.736 B'; BANDA_ENVIO_B (73_571, 84_244); BANDA_COBRO_B (72_382, 83_723)`; `crates/zk-ssl/src/metrics.rs:82-83: PUBLICADA_PAGO_MIN_B = 145_953; PUBLICADA_PAGO_MAX_B = 167_967 (un pago = DOS pruebas)`; `spec/rfc/0009-lo-que-revela-una-prueba.md:624-629: 'Quince muestras por eje ... envío 77.444..80.232 B, cobro 76.192..79.736 B; con un margen declarado del 5 % ... pago 145.953..16`; `AUDITORIA.md §538 (l.39270-39330): encendido de la ocultación, zkssl/0.4, cifras en BANDA`; `'~62 KB por PRUEBA' sólo como prosa antigua: crates/zk-ssl/src/log.rs:196,204; PREGUNTAS.md:84; AUDITORIA §304 (l.22718-22720): 'dicen «62 KB por transferencia» ... el 62 KB es por`; `PAPER.md:489: 'Transferencia ~620 ms ~4 ms 61.966 B' (vía de un paso, retirada; README.md:209-211)`; `grep '62[,.]4' en el árbol: 0 coincidencias`

### M3b

**FALSA** · gravedad alta

- **Borrador:** Tiempo de verificación 8,1 ms
- **Lo que se puede afirmar:** Verificar una prueba de envío o de cobro, sin árbol ni disco, midió 2,32-2,41 ms (5 ejecuciones, media 2,35 ms; §89.1) y 2,43-2,49 ms (§204 A.2); el verificador aislado, 2,37-2,86 ms (§305). Todo es anterior a la ocultación del §538, y el repo no tiene una re-medida posterior. En una microVM (2026-09-28 y 2026-09-30, una ejecución cada día), el apply ENTERO, que incluye la verificación con sal, tardó 5-6 ms. Por RPC, el coste por operación aplicada es 4,035 ms (§229). La cifra de 8,1 ms no tiene fuente. Los 8 ms que aparecen en las tablas son la verificación PLONK/KZG del circuito de comparación (FIVE_BACKENDS.md:38).
- **Fuentes:** `AUDITORIA.md §89.1 (l.7426-7433): 'el_coste_de_verificar_una_prueba, cinco ejecuciones independientes en release: 2,36 / 2,34 / 2,41 / 2,32 / 2,35 ms; Media 2,35 ms, dispersion 4 %`; `crates/zk-ssl/src/metrics.rs:241-345: instrumento #[ignore], N=50 verificaciones tras un calentamiento, media entre envío y cobro`; `AUDITORIA.md §89.2 (l.7445-7449): verificar = 3,2 % del apply`; `AUDITORIA.md §305 (l.22745-22748): verificador sólo-verificador en nativo 2,37-2,86 ms`; `AUDITORIA.md §229 (l.16318-16330): por RPC 4,035 ms/op (verificación STARK + candado + deserialización + transporte)`; `AUDITORIA.md §462 (l.34039): prueba de edad, verificar 1,1-1,6 ms`; `FIVE_BACKENDS.md: 8 ms es la verificación de PLONK/KZG, no la de STARK`; `No se encontró ninguna re-medición de la verificación de send/claim posterior al §538 (grep sobre AUDITORIA.md desde la l.39270)`

### M4a

**FALSA** · gravedad alta

- **Borrador:** Sobrecarga de encadenamiento hash < 0,002 ms por transición
- **Lo que se puede afirmar:** El repo no mide directamente el sobrecoste de encadenar una entrada, pero los componentes que sí mide lo sitúan un orden de magnitud por encima de 0,002 ms. El resumen de la prueba (Blake3 sobre ~66 KB) midió 0,011 ms (§204 A.4, §209). El chain_digest son 5 permutaciones Rescue-Prime Rp64_256, 6 en la era 2 (log.rs:250-294; zk-ssl-hash/src/lib.rs:100-106), a 7,44-8,91 µs cada una (§217). Eso da una ESTIMACIÓN derivada de 37-54 µs por entrada. Tras el §209, el apply_send cuesta 3,11-3,33 ms, y lo que domina es la verificación STARK (2,43-2,49 ms, §204 A.2), no el hash; el encadenamiento sería del orden del 1-2 % (derivado). Todo es anterior al §538.
- **Fuentes:** `AUDITORIA.md §204 A.4 (l.13876-13893): prueba de envío de 65.840 B; 'digest_of_proof (Rescue) 30,99 ms'; 'Blake3 sobre los mismos bytes 0,011 ms'; 'chain_digest (5 merges) SE QUEDA`; `AUDITORIA.md §209 (l.14160-14166): tras el cambio, digest_of_proof '~0,01 ms', apply_send 3,11-3,33 ms`; `crates/zk-ssl-hash/src/lib.rs:1303-1320: digest_of_proof = Blake3_256(DOMINIO_PRUEBA \|\| len u64 LE \|\| prueba)`; `crates/zk-ssl/src/log.rs:250-262 (chain_digest = 5 native_merge) y 281-294 (chain_digest_v2 = +1 merge); crates/zk-ssl-hash/src/lib.rs:100-106 (native_merge = permutación Rp64_256)`; `AUDITORIA.md §217 (l.14746-14747): 'El µs por merge implícito sale 7,53 en §204, 7,44 en el set_leaf de B.2, y 8,91 en el arranque de B.4'`; `crates/zk-ssl/src/log.rs:250-262 (chain_digest = 5 native_merge), 281-294 (chain_digest_v2 = +1)`; `crates/zk-ssl-hash/src/lib.rs:100-106 (native_merge = Rp64_256::apply_permutation)`; `AUDITORIA.md §217 (l.14746-14747) 'El µs por merge implícito sale 7,53 en §204, 7,44 ... y 8,91'`

### M4c

**FALSA** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** Crecimiento ~256 bytes por transición
- **Lo que se puede afirmar:** Cada entrada del registro encadenado ocupa en disco 137 B (era 1) o 169 B (era 2, con el compromiso autorizante) (crates/zk-ssl/src/store.rs:415-455; test la_ida_y_vuelta_del_disco_conserva_la_era, log.rs:1528). Desde el S569, el nodo anota además 40 B, con fsync, por cada applySend o applyClaim directo que evalúa, lo aplique o lo rechace, en el registro de recepción (registro_recepcion.rs:110, 141-167; AUDITORIA §569). Ese registro hoy crece sin poda (§580). El lote de applyMany y la prenda no lo llevan (§576). La entrada guarda el resumen de la prueba, no la prueba (log.rs:194-198); si se retienen las pruebas, cada envío o cobro añade 76-80 KB (§538). Ninguna cifra del repo da 256 B.
- **Fuentes:** `crates/zk-ssl/src/store.rs:418-445: log_entry_to_bytes: seq(8)+kind(1)+root_old+root_new+proof_digest+chain (4×32) = 137 B (era 1); +compromiso 32 B = 169 B (era 2); 'se esperaban `; `crates/zk-ssl/src/log.rs:1469-1497 test la_ida_y_vuelta_del_disco_conserva_la_era: assert_eq!(b2.len(), 169) / assert_eq!(b1.len(), 137)`; `crates/zk-ssl/src/log.rs:194-206: la entrada guarda el resumen, no la prueba ('~62 KB por PRUEBA', hoy la banda del S538)`; `crates/zk-ssl-node/src/registro_recepcion.rs:109: registro de recepción de 40 B por entrada (rx u64 + digest 32)`; `crates/zk-ssl/src/store.rs:415-455 ('137 = seq(8) \| kind(1) \| root_old(32) \| root_new(32) \| proof_digest(32) \| chain(32)'; '169'; 'se esperaban 137 (era 1) o 169 (era 2)')`; `crates/zk-ssl/src/log.rs:1528-1556 (la_ida_y_vuelta_del_disco_conserva_la_era: assert_eq!(b2.len(), 169); assert_eq!(b1.len(), 137))`; `crates/zk-ssl-node/src/registro_recepcion.rs:110 (ANCHO_ENTRADA = 40), 141-167 (anotar: write_all + sync_all del fichero y, si es nuevo, del directorio)`; `crates/zk-ssl-node/src/main.rs:2669-2673 (recibir -> anotar -> apply_send)`

### M3a

**SIN_FUENTE** · gravedad alta · **el escéptico revocó el primer veredicto** (FALSA)

- **Borrador:** Tiempo de proving 412 ms
- **Lo que se puede afirmar:** Ninguna medición del repo da 412 ms: la cifra no tiene fuente. En el portátil del autor, en release y antes de la ocultación, generar una prueba de envío costó 322-353 ms y una de cobro 218-243 ms, en dos tandas de 5 muestras (σ intra-tanda ~0,5 %, deriva sistemática entre tandas ~9 %; AUDITORIA §130-§131). Tras la ocultación del §538, cada lado costó 697-742 ms, tomado como el mínimo de repeticiones entrelazadas con la primera descartada, en dos o tres corridas (metrics.rs:128-133; RFC-0009:744-748). Esta última cifra no figura en AUDITORIA. Son valores absolutos de una sola máquina, sin gate (§304), y el probador es monohilo. En otra máquina (microVM, una ejecución por día), el envío tardó 970 y 1.333 ms y el cobro 1.198 y 879 ms: el orden entre los dos lados se invierte de una corrida a otra.
- **Fuentes:** `grep '412 ms' en el repo: 0 coincidencias`; `AUDITORIA.md §130.1 (l.10660-10668): send generación 353,2 ms, claim 243,1 ms (caliente 236,9), apply 36,4 / 38,1 ms; n=5, un proceso por muestra, release; '⚠️ Cifras de UN context`; `AUDITORIA.md §131.1-131.2 (l.10724-10740): segunda tanda send 322,2 ± 1,8 ms, claim 217,7 ± 0,9 ms; 'σ ≈ 0,5 % dentro y ~9 % entre ellas'; 'Publicar σ 0,6 % sugiere reproducibilida`; `crates/zk-ssl/src/metrics.rs:128-133: 'encendida la ocultacion ... envio 697,9-741,6 ms y cobro 696,9-715,6 ms como minimos'`; `spec/rfc/0009-lo-que-revela-una-prueba.md:744-748: 'Encendida la ocultación, cada lado cuesta 700-750 ms'`; `crates/zk-ssl/src/metrics.rs:604-620: medido apareado el 2026-09-19, envío 240,8->158,9 ms y cobro 160,2->278,4 ms (antes de la ocultación)`; `grep '412 ms' en el árbol: 0 coincidencias`; `AUDITORIA.md §219 (l.15115-15121) '282 · 461 · 220 ms ... aun promediando tres varía un ±50 %'; §220 (l.15181-15182) 'la generación de una prueba —220 a 461 ms con ±50 % de ruido (`

### M4b

**SIN_FUENTE** · gravedad alta

- **Borrador:** Velocidad de auditoría histórica offline ~125.000 bloques/seg
- **Lo que se puede afirmar:** El repo no mide el rendimiento del reverificador ni el de ninguna auditoría histórica offline (zk-ssl-verify/src/reverificacion.rs; §279-§282). Arqueo no tiene 'bloques': tiene entradas de registro y cabezas de época. Sólo caben ESTIMACIONES derivadas, y deben declararse como tales. Recomponer la cadena (5-6 permutaciones Rescue por entrada, a 7,44-8,91 µs; §217) daría unas 19.000-27.000 entradas/s por núcleo. Re-verificar las pruebas, que el registro no guarda, daría unas 425 por segundo y núcleo (§89.1, antes del §538). La cifra de 125.000/s no tiene fuente. La más parecida del repo, 122.850 derive_public_id/s por núcleo (§82.3), mide otra cosa.
- **Fuentes:** `grep '125[.,]000' en el repo: 0 coincidencias`; `AUDITORIA.md §279 (l.20976-21000) y §282: el reverificador de zk-ssl-verify (reverificacion.rs) da un veredicto por entrada, sin cifra de rendimiento`; `doc/KIT.md: sin tiempos de verificación offline`; `grep '125[.,]000' en el árbol: 0 coincidencias`; `crates/zk-ssl-verify/src/reverificacion.rs:1-40 (veredicto por entrada; sin cifra de rendimiento); AUDITORIA.md §279 (l.20976-21000), §282`; `AUDITORIA.md §82.3 (l.6765) '122.850' derive_public_id por segundo y núcleo (otra magnitud)`; `AUDITORIA.md §217 (l.14746-14747); §89.1 (l.7432) 'Pruebas/s por nucleo \| 425'`; `crates/zk-ssl/src/log.rs:194-198 'Resumen de la prueba, no la prueba entera'`

### M4d

**SIN_FUENTE** · gravedad media

- **Borrador:** Reducción del 99,6 %
- **Lo que se puede afirmar:** El repo no publica ninguna 'reducción del 99,6 %'. Ese porcentaje es exactamente el cociente de dos cifras sin fuente de los propios borradores: 256 B frente a 62,4 KB, 1 − 256/62.400 = 99,59 %. Con cifras del repo, y como DERIVACIÓN, guardar la entrada de 169 B (era 2) en lugar de la prueba (76.192-80.232 B, §538) ahorra un 99,78-99,79 %. La frase más cercana es cualitativa, anterior a la era 2 y a la ocultación: doc/CONSECUENCIAS.md:45-47, '137 bytes por operación en vez de ~65 KB: ~472 veces menos'.
- **Fuentes:** `grep '99[,.]6' en el repo: 0 coincidencias`; `doc/CONSECUENCIAS.md:45-47: 'Con retención distribuida, el operador guarda 137 bytes por operación en vez de ~65 KB: ~472 veces menos'`; `grep '99[,.]6' en el árbol: 0 coincidencias`; `python3: 1-256/62400 = 0,99590; 1-169/80232 = 0,99789; 1-169/76192 = 0,99778`; `doc/CONSECUENCIAS.md:45-47 '137 bytes por operación en vez de ~65 KB: ~472 veces menos'`; `crates/zk-ssl/src/store.rs:415-417; crates/zk-ssl/src/metrics.rs:114-122`

### M5a

**SIN_FUENTE** · gravedad alta

- **Borrador:** fsync: ext4 1,82 ms vs tmpfs 0,03 ms
- **Lo que se puede afirmar:** Ninguno de los dos repositorios registra 1,82 ms ni 0,03 ms. La única medición del repo es el banco K.1 (§234), en ext4 bajo WSL2: fsync de 0,907 ms, y de 0,002 ms en tmpfs. Los valores absolutos dependen del disco y del entorno. Por eso el guardián no decide por el valor absoluto sino por la RAZÓN con escribir sin persistir (crates/zk-ssl-guardian/src/lib.rs:60-63, 96-106).
- **Fuentes:** `grep '1[,.]82' y '0[,.]03 ms' en el repo: 0 coincidencias relevantes`; `AUDITORIA.md §234 (l.16831-16840): K.1: '$HOME (ext4) 0,907 ms 382× \| /tmp (tmpfs) 0,002 ms 1×'`; `grep '1[,.]82' y '0[,.]03 ms' en Arqueo y en hbs-state@a960828: 0 coincidencias`; `AUDITORIA.md §234 (l.16835-16838) '$HOME (ext4) \| 0,907 ms \| 382× ... /tmp (tmpfs) \| 0,002 ms \| 1×'`; `crates/zk-ssl-guardian/src/lib.rs:60-63 'Un NVMe rápido puede dar fsync de ~100 µs legítimos; por eso el discriminante principal es la razón'`; `doc/ecst/borrador/REPRODUCCION.md (sonda-fsync, dos tandas de 30): ext4 p50 128,9/173,1 µs; tmpfs 0,537/0,536 µs`

### M6a

**SIN_FUENTE** · gravedad media

- **Borrador:** Queda como trabajo futuro la verificación formal TLA+ de R(C,K)
- **Lo que se puede afirmar:** Ni Arqueo ni hbs-state mencionan TLA+ ni ningún modelo formal de la reconciliación R(C,K); lo comprobé con grep en ambos árboles. El horizonte formal que declara Arqueo, FV-3 (Lean4/Coq o K-Framework), se refiere a la solidez del AIR y está marcado como 'dirección, no deuda' (doc/VERIFICACION_FORMAL.md §3). Un modelo TLA+ del protocolo de persistir y luego firmar frente a caídas sería una propuesta del propio informe, y así debe presentarse. Hoy ese protocolo sólo está probado con tests y con el banco K.1 (kill -9 en 25 muertes; §234), no frente a cortes de corriente.
- **Fuentes:** `grep -i 'TLA' en todo el repo: 0 coincidencias (AUDITORIA.md incluido)`; `doc/VERIFICACION_FORMAL.md §3 (l.96-104): 'FV-3 — Lean4/Coq ... o K-Framework ... Se declaran como dirección ... no como deuda'`; `crates/zk-ssl-guardian/src/lib.rs:34-38: la reconciliación (Reconciliacion) está probada con tests y con el banco K.1, sin modelo formal`; `grep -rwiE 'TLA\|tlaplus\|PlusCal' en Arqueo y en hbs-state@a960828: 0 coincidencias`; `doc/VERIFICACION_FORMAL.md:96-104 (§3 FV-3: 'Lean4/Coq ... o K-Framework ... Se declaran como dirección ... no como deuda')`; `AUDITORIA.md §234 (l.16862-16865, 16907-16911): kill -9, 25/25; corte de corriente sin medir`; `crates/zk-ssl-guardian/src/lib.rs:324-353 (enum Reconciliacion, cuatro variantes)`

### M7c

**SIN_FUENTE** · gravedad alta

- **Borrador:** Comparación ECST vs IVC (implícita en los borradores)
- **Lo que se puede afirmar:** No hay ninguna comparación medida entre el modelo ECST (verificación nativa más encadenamiento con hash) e IVC/PCD sobre la misma carga, máquina y circuito. Las cifras de Nova, con un paso trivial y a nivel de prueba de concepto, y las de la zkVM (§305-§307: otra máquina, envoltura de una prueba) no son comparables entre sí ni con la capa. Un informe riguroso debe presentar la ventaja de ECST frente a IVC como argumento CUALITATIVO. La frase 'el mismo circuito en cinco sistemas' (PAPER.md:12-15; README.md:196) es inexacta para Nova.
- **Fuentes:** `FIVE_BACKENDS.md:86-90: 'Comparar de verdad exigiría portar la partida doble al StepCircuit'`; `PAPER.md:477-481: Nova 'se evalúa por separado por su naturaleza distinta'`; `PAPER.md:12-15 (Resumen): 'el mismo circuito ... en cinco sistemas', frente a FIVE_BACKENDS.md:86-88 (Nova no implementa ese circuito)`; `FIVE_BACKENDS.md:86-90 ('Comparar de verdad exigiría portar la partida doble al StepCircuit'; 'No implementa el circuito de cumplimiento')`; `PAPER.md:12-15 ('el mismo circuito de liquidación financiera en cinco sistemas ... Nova/plegado'); PAPER.md:477-481`; `README.md:196 'El diseño se eligió midiendo el mismo circuito en cinco sistemas de prueba'`; `AUDITORIA.md §305-§307 (envoltura en zkVM, no IVC)`

### M11

**PARCIAL** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** CONTRIBUTING: 188 pasan / 93 fallan en depuración frente a 307/0 en release el 26-08-2026
- **Lo que se puede afirmar:** CONTRIBUTING.md:91-99 dice literalmente eso, pero mezcla dos momentos. La medición del 26-08-2026 (AUDITORIA §20, §366) dio 188/93/9 en depuración y 287/0/3 en release. El 307 es el pin posterior del §394, y el propio AUDITORIA declara rancia la cifra de depuración. Hoy el canon fija para zk-ssl 436 tests que pasan y 7 ignorados en release (tools/canon.sh:109). Lo relevante para reproducir es que la capa sólo se valida en release. En depuración, winterfell comprueba que se realice el grado declarado, y hay restricciones cuyo grado depende del testigo; los 93 fallos son todos de esa clase (§366). Los bancos de integración quedan fuera del canon: 'canon.sh --bancos' los corre desde el §582, pero no es compuerta.
- **Fuentes:** `CONTRIBUTING.md:92-99: 'Medido el 26-08-2026: cargo test -p zk-ssl en depuración da 188 pasan, 93 fallan y 9 ignorados; en release, 307 pasan, 0 fallan y 3 ignorados'`; `AUDITORIA.md §366 (l.27845-27850) y §367 (l.27909-27911): la medida de ese día: '188 pasan / 93 fallan / 9 ignorados en 383,87 s, y 287 / 0 / 3 en release'`; `AUDITORIA.md §20 (l.1423-1427): '188 pasan, 93 fallan, 9 ignorados en depuracion, y 287 pasan, 0 fallan, 3 ignorados en release'`; `AUDITORIA.md l.29274-29277, 29364-29365, 29515: la cifra de depuración de CONTRIBUTING se declara 'rancia' y sin compuerta (deuda de la nota 41)`; `tools/canon.sh:89: pin actual de zk-ssl en release = 421 pasan, 7 ignorados`; `AUDITORIA.md §366 (l.27852-27855): los 93 fallos son todos de evaluation_table.rs:214, el límite de grados que winterfell comprueba sólo en depuración`; `README.md:85-86: '--release es obligatorio para la capa'`; `CONTRIBUTING.md:91-99 'Medido el 26-08-2026 ... 188 pasan, 93 fallan y 9 ignorados; en release, 307 pasan, 0 fallan y 3 ignorados'`

### M12b

**PARCIAL** · gravedad baja

- **Borrador:** Regla de doc/preprints/ERRATA.md: 'una cifra publicada que se corrige no se borra, se marca'
- **Lo que se puede afirmar:** La frase literal está en README.md:207-209, en la sección 'Publicación', no en ERRATA.md. ERRATA.md formula la regla equivalente ('no se edita el cuerpo: se registra aquí') y aclara que registrar una errata no la corrige para quien ya leyó el PDF. Tiene cuatro entradas y cubre sólo los tres preprints técnicos de doc/preprints/, no los tres depósitos económicos.
- **Fuentes:** `README.md:200-202: 'Las versiones anteriores siguen accesibles y se citan aquí: una cifra publicada que se corrige no se borra, se marca'`; `doc/preprints/ERRATA.md:3-6: 'Cuando se detecta un error o una omisión en uno de ellos, no se edita el cuerpo: se registra aquí'; l.15-20: 'Registrar una errata no la corrige para `; `doc/preprints/ERRATA.md: entradas 1-4 (l.23, 63, 90, 137), todas sobre los tres preprints de doc/preprints/`; `doc/preprints/README.md:11-17: 'Los tres' (sólo 21736125, 21736082 y 21905595)`; `README.md:207-209 'una cifra publicada que se corrige no se borra, se marca'`; `doc/preprints/ERRATA.md:3-6 ('no se edita el cuerpo: se registra aquí'), 15-19; entradas en l.23, 63, 90, 137`; `doc/preprints/README.md:11-17 (los tres)`

### M14a

**PARCIAL** · gravedad media

- **Borrador:** t1_cabeza_ata_la_historia ... valida el Teorema 1
- **Lo que se puede afirmar:** El test existe y pasa, pero es un caso concreto (12 entradas, una alteración en la entrada 5, con pruebas sintéticas), no una demostración. Comprueba en una instancia que la cabeza del registro en n depende de la entrada k < n (crates/zk-ssl/src/log.rs:1189-1285). La validez general se sigue de la resistencia a colisiones de Rescue-Prime Rp64_256 en chain_digest, que es un supuesto y no algo verificado. El repo lo presenta como la 'premisa medida' de la cadencia de firma de 1/min (§115), no como la validación de un teorema.
- **Fuentes:** `crates/zk-ssl/src/log.rs:1131-1227: mod t1_chain_retroactivo, con N = 12 entradas y la mentira en K = 5; t1_cabeza_ata_la_historia recomputa desde los campos crudos con chain_diges`; `AUDITORIA.md §115.1 (l.9468-9483): 'Medido: log.rs, modulo t1_chain_retroactivo, 3/3 en verde ... El segundo es, literalmente, «la firma en n cubre la mentira en k»'`; `crates/zk-ssl/src/log.rs:250-262: la propiedad descansa en la resistencia a colisiones de native_merge (Rescue-Prime Rp64_256)`; `crates/zk-ssl/src/log.rs:1189-1285 (mod t1_chain_retroactivo; N = 12, K = 5 en 1198-1199; t1_cabeza_ata_la_historia en 1237)`; `diff del módulo entre d531c80 y HEAD: idéntico`; `AUDITORIA.md §115.1 (l.9468-9483) 'la firma en n cubre la mentira en k'`; `crates/zk-ssl/src/log.rs:250-262 (chain_digest sobre native_merge/Rp64_256)`; `doc/ecst/borrador/REPRODUCCION.md (3 passed en d531c80 el 2026-09-28 y en 71c5aad el 2026-09-30; microVM)`

### M14b

**PARCIAL** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** El banco de vectores de HBS-STATE verifica que implementaciones dispares pueden someterse al mismo análisis (referencia de Arqueo a otras implementaciones)
- **Lo que se puede afirmar:** En Arqueo, la conformidad es un contrato para una segunda implementación que no existe, y NUCLEO.md lo dice expresamente: 'no afirma que exista' (spec/NUCLEO.md:284). Hay KAT byte a byte del núcleo (spec/vectors/nucleo/, 23 ficheros), vectores de protocolo por versión (zkssl-0.1 a 0.4) y catálogos de paquete, 356 ficheros en total, con un arnés (tools/conformidad.sh) ejecutable sobre cualquier binario verificador que cumpla el contrato de salida 0/1/2/3. Hoy sólo se han ejercitado contra la implementación de referencia; una re-ejecución externa sobre 71c5aad dio 257 de 257 entradas (REPRODUCCION.md, 2026-09-30). Desde el §538, la conformidad de la 0.4 es 'todo IDENTICO en lo que el circuito fija'. Arqueo no afirma que los vectores de HBS-STATE validen implementaciones dispares: cita un vector (A4) y una frase de su especificación en registro_recepcion.rs, recepcion.rs:218 y §565, y el 'M5' que menciona no está en el clon hbs-state@a960828.
- **Fuentes:** `spec/NUCLEO.md:22-25, 229-232, 280: la segunda implementación 'tiene que producir los mismos bytes'; KAT en spec/vectors/nucleo/ (23 ficheros); 'No afirma que una segunda implement`; `README.md:93-94 y doc/README-CLI.md:72-73: 'el contrato de una segunda implementación: re-ejecuta el escenario canónico y compara campo a campo'`; `tools/canon.sh:285-292: el canon exige 'zkssl-0.4.json -> todo IDENTICO (en lo que el circuito fija, D-AI)' y 0.3/0.2/0.1 'RECHAZADO (otra version)'`; `AUDITORIA.md §538 (l.39299-39302): el comparador 0.4 declara no reproducibles proof_digest, chain y epoch_digest`; `find spec/vectors -type f \| wc -l = 317 (README.md:185)`; `Arqueo cita HBS-STATE sólo en crates/zk-ssl-node/src/registro_recepcion.rs:61-79 y AUDITORIA §565 (l.41003-41027): el vector A4 ('counter=7 key=5 -> CounterAhead') y la frase 'No d`; `hbs-state/spec/HBS-STATE-v0.3 (md):168-172: esa frase se refiere a KeyAtZero; grep 'M5' en el clon hbs-state@a960828: 0 coincidencias`; `spec/NUCLEO.md:24-25 ('tiene que producir los mismos bytes'), 233-234, 284 ('No afirma que una segunda implementación exista (E5 del RFC-0005)')`

### M2b

**PARCIAL** · gravedad media

- **Borrador:** Banda publicada 66.739 / 66.692 B (¿por qué unidad?)
- **Lo que se puede afirmar:** 66.739 B (envío) y 66.692 B (cobro) son el tamaño de UNA prueba de cada lado tras el arreglo B. Se midieron en el §512 (2026-09-19), sin ocultación, y eran deterministas. Suman 133.431 B por pago (127,2 MiB por mil), la 'cifra apagada' que el §538 retiró (RFC-0009:632). El comentario de metrics.rs:74-78 les pone la fecha 2026-08-26, pero ese día (§362) la vía documentada dio 66.998 y 65.313 B, las cifras del §304. No son una banda ni la cifra vigente. La vigente es la banda del §538: 145.953-167.967 B por pago.
- **Fuentes:** `crates/zk-ssl/src/metrics.rs:67-71: 'MEDIDO el 2026-08-26: la via DOCUMENTADA -- send_materials -> client::prove_send -> apply_send -- da los MISMOS bytes, 66_739 y 66_692, en cinc`; `tools/check_publicadas.py (docstring): mismo texto`; `spec/rfc/0009-lo-que-revela-una-prueba.md:614-633: 'La cifra apagada (133.431 B, 127,2 MiB) queda en los asientos y en la entrada 22'`; `AUDITORIA.md §304 (l.22651-22652): 'Los bytes son deterministas —66.998 y 65.313, identicos en cuatro corridas—' (2026-08-14)`; `AUDITORIA.md §130.1 (l.10660-10664): send 64.722 B, claim 65.250 B`; `AUDITORIA.md §218 (l.14936-14942): mint 54.858/55.568 B, send 66.164 B, claim 66.820 B`; `AUDITORIA.md §512 (l.37370-37380, 37388-37391): 'ENVIO 66.998 -> 66.739 B ... COBRO 65.313 -> 66.692; el pago, 133.431'; 'Medido APAREADO el 2026-09-19'`; `AUDITORIA.md §362 (l.27567-27571): la vía documentada, 'cinco repeticiones, 66.998 y 65.313 sin una sola diferencia'`

### M2c

**PARCIAL** · gravedad media

- **Borrador:** STARK 36,7 KB (PAPER) como tamaño de prueba del sistema
- **Lo que se puede afirmar:** 36,7 KB es el tamaño de la prueba STARK del circuito de COMPARACIÓN de FIVE_BACKENDS.md (blowup 16, extensión cuadrática; 127 bits conjeturados frente a 29/63 demostrables), medido en una sola ejecución (FIVE_BACKENDS.md:39, 146, 301-303). No es el tamaño de una transición de la capa. Los circuitos de producción midieron 54.858-66.820 B (53,6-65,3 KB con KB = 1024 B) antes de la ocultación (§218). Desde el §538, una prueba de envío o de cobro mide 76.192-80.232 B, con banda publicada de 72.382-84.244 B (metrics.rs:114-122). Los demás circuitos no tienen cifra publicada tras el §538.
- **Fuentes:** `PAPER.md:151-154: 'Las pruebas STARK del circuito de comparación ocupan 36,7 KB ... Los circuitos de producción de la capa son mayores —53,6 a 65,3 KB, medidos en §218—'`; `FIVE_BACKENDS.md:36 y tabla 'brecha' (l.~150-156): 'blowup 16, ext. cuadrática \| 36,7 KB \| 39 ms \| 127 bits \| 29 / 63'`; `AUDITORIA.md §218 (l.14944-14951): '36,7 KB es una medida correcta —del circuito de comparación ... Son 1,5-1,8× más grandes' los de producción`; `SECURITY.md:548: 'Los 36,7 KB de las tablas comparativas son del circuito de comparación, no de éstos'`; `FIVE_BACKENDS.md:39 (tabla: 'Tamaño de prueba ... 36,7 KB'); :146 ('blowup 16, ext. cuadrática \| 36,7 KB \| 39 ms \| 127 bits \| 29 / 63'); :301-303 (una sola ejecución)`; `PAPER.md:151-154 'Las pruebas STARK del circuito de comparación ocupan 36,7 KB ... 53,6 a 65,3 KB, medidos en §218'`; `AUDITORIA.md §218 (l.14936-14951) '36,7 KB es una medida correcta —del circuito de comparación'`; `SECURITY.md:585 'Los 36,7 KB de las tablas comparativas son del circuito de comparación'`

### M5b

**PARCIAL** · gravedad media

- **Borrador:** fsync 0,907 ms vs 0,002 ms (382x)
- **Lo que se puede afirmar:** El banco K.1 (AUDITORIA §234) midió fsync en dos sistemas de ficheros de la misma máquina (WSL2 sobre un i5-1135G7): 0,907 ms en ext4 y 0,002 ms en tmpfs. El factor 382× NO es el cociente ext4/tmpfs, que sería 453×. Es lo que cuesta fsync en ext4 frente a escribir sin persistir en ese mismo ext4. En tmpfs esa razón es 1×, porque fsync devuelve éxito sin persistir nada. De ahí el umbral declarado del guardián: razón mínima de 10× y suelo de 20 µs, sobre 20 escrituras (crates/zk-ssl-guardian/src/lib.rs:94-106). K.1 probó durabilidad frente a la muerte del proceso (25 de 25 sin firma por delante; 13 de 25 con el contador adelantado), no frente a un corte de corriente.
- **Fuentes:** `AUDITORIA.md §234 (l.16835-16840): tabla '\| \| fsync \| frente a no persistir \|': ext4 0,907 ms **382×**; tmpfs 0,002 ms **1×**`; `crates/zk-ssl-guardian/src/lib.rs:40-47 y 96-106: misma tabla; RAZON_MINIMA 10.0; SUELO_MICROS 20.0`; `hbs-state/README.md:82-85 y hbs-state/src/lib.rs:40-45: la misma tabla ('against not persisting')`; `AUDITORIA.md §234 (l.16864-16868): kill -9, '25 de 25 sin una sola firma por delante'; '13 de 25 dejaron el contador adelantado'`; `AUDITORIA.md l.16907-16914 y crates/zk-ssl-guardian/src/lib.rs:64-70: 'Nada frente a un corte de corriente ... no se ha hecho'; 'Los umbrales salen de UNA máquina'`; `AUDITORIA.md §290 (l.21575-21577): en WSL /tmp es tmpfs, razón 1,0× frente al mínimo 10×`; `AUDITORIA.md §234 (l.16831-16848) tabla 'fsync \| frente a no persistir'; 'el umbral se fija en 10×'`; `crates/zk-ssl-guardian/src/lib.rs:40-47 (misma tabla), 94-106 (MUESTRAS_AUTOCOMPROBACION = 20; RAZON_MINIMA = 10.0; SUELO_MICROS = 20.0)`

### M6b

**PARCIAL** · gravedad media

- **Borrador:** Falta de verificación formal mecanizada
- **Lo que se puede afirmar:** No hay ninguna demostración mecanizada de la solidez del AIR, de FRI, de Rescue ni de las propiedades de la capa. Sí hay trabajo formal parcial. FV-1 es un censo sintáctico de celdas y es compuerta del canon en seis circuitos (mint_climb, credit_climb, recovery_climb, mint, send y claim; tools/check_constraint_layout.py:964-972). Garantiza que ninguna celda de su traza queda sin restricción ni declaración ('referenciada', no 'determinada'), y no cubre las variantes v2 ni los circuitos posteriores. FV-2 fue un sondeo SMT con cvc5 --ff sobre circuit_refund: dio unsat en la consistencia de la cadena, con Rescue abstraído, y la pregunta de determinación completa agotó 6 GB en ~21 s, lo que cerró con datos su extensión (AUDITORIA §183-§190, §196). FV-3 (Lean4/Coq/K) es horizonte declarado. SECURITY.md:86-88 aún presenta FV-2 como horizonte, en contra del §190.
- **Fuentes:** `doc/VERIFICACION_FORMAL.md §0 (l.5-12): 'La suite ... prueba PUNTOS ... No prueba el universal'`; `doc/VERIFICACION_FORMAL.md §1 y AUDITORIA §188-§189, §196 (l.13148-13226, 13446-13492): FV-1, censo sintáctico de celdas ('cada celda tiene dueño'), compuerta de tools/check_constr`; `AUDITORIA.md §190 (l.13226-13272): FV-2 sobre circuit_refund (20 restricciones, 12 aserciones, traza 16×12) con cvc5 1.3.2 --ff: 'q2_cadena_uf → unsat en 18/43/46 ms' con las ronda`; `tools/canon.sh:272: check_constraint_layout es una de las herramientas del canon`; `doc/VERIFICACION_FORMAL.md §3: FV-3 (Lean/Coq/K) sin empezar`; `doc/VERIFICACION_FORMAL.md §0 (l.5-12), §1, §2 ('EJECUTADA en §190: q2 unsat · q1 intratable por memoria'), §3`; `AUDITORIA.md §190 (l.13226-13262): 'q2_cadena_uf → unsat en 18/43/46 ms'; q1 'agotan la jaula de 6 GB en ~21 s'; 'Escalar a circuit_send ... CERRADO con datos'`; `tools/check_constraint_layout.py:641-672 (censo_mint_climb ... censo_claim), 964-972 (los seis ficheros censados)`

### M7a

**PARCIAL** · gravedad media

- **Borrador:** ECST evita la verificación recursiva intra-circuito, reduciendo el coste
- **Lo que se puede afirmar:** En Arqueo, cada prueba se verifica de forma nativa en la capa (~2,35 ms, antes del §538) y su resumen se encadena en un registro con hash, sin recursión (§47.2, para el camino de producción; la única recursión del árbol es la prueba de concepto nova-experiment). El ahorro sólo tiene un apoyo medido indirecto: envolver UNA prueba de la capa verificándola dentro de una zkVM (RISC Zero) cuesta 47,5 M ciclos, ~5 h de CPU en el portátil o ~65 s de GPU en una RTX 5090, y el receipt sucinto (223.234 B) es MAYOR que la prueba suelta. Sólo ahorra bytes al agregar unas cuatro pruebas o más (§305-§307, con pruebas anteriores al §538). Es un experimento de envoltura, no un IVC, y la nota 22 sigue suspendida. La contrapartida es que, sin recursión, verificar la historia exige verificar cada prueba, y el registro no las guarda.
- **Fuentes:** `AUDITORIA.md §47.2 (l.4226-4231): 'no hay recursion, folding, agregacion ni verificacion-de-prueba-en-circuito en ningun crate'`; `AUDITORIA.md §204 (l.14013-14018): recursión/migrar a Plonky3-Miden descartada: 'winterfell no trae verificador recursivo; Miden es un zkVM ... y la etapa 2 no necesita recursion'`; `AUDITORIA.md §89.1: verificación nativa 2,35 ms`; `AUDITORIA.md §305 (l.22722-22777): el verificador dentro de la zkVM RISC Zero: 47.513.440 ciclos; portátil sin GPU, proyección 5 h 07 por prueba ('ROJA por COSTE')`; `AUDITORIA.md §306 (l.22875-22901): en RTX 5090, prueba composite 40,9 s + compresión 23,8 s; receipt sucinto 223.234 B, '3,3 veces mas GRANDE' que una prueba de 66.998 B; equilibri`; `AUDITORIA.md §307 (l.23005-23070): sucinto constante (223.234 B a 49 y 222 segmentos), join ~0,49 s`; `BACKLOG.md:1768-1860 (entrada 22): 'Agregar cuesta unos 65 s de GPU por prueba y ahorra unos 67 KB ... NO HAY CRUCE DE CURVAS'; SUSPENDIDA (§318)`; `AUDITORIA.md §47.2 (l.4226-4229) 'no hay recursion, folding, agregacion ni verificacion-de-prueba-en-circuito en ningun crate'; Cargo.toml:10 (miembro crates/nova-experiment)`

### M7b

**PARCIAL** · gravedad media

- **Borrador:** IVC impone una sobrecarga computacional masiva
- **Lo que se puede afirmar:** Lo único medido de IVC o plegado es una prueba de concepto con Nova (nova-snark 0.73, BN254/Grumpkin, HyperKZG) cuyo paso es un único hash Poseidon. Cuesta 10.764 restricciones por paso, casi todas del circuito verificador que inserta el plegado, y ~250 ms constantes por paso (el paso 9 costó 0,77 veces el paso 1). A eso se suman 4,02 s de setup, 1,84 s de compresión final y 108 ms / 50 ms de verificación (FIVE_BACKENDS.md:57-90). El repo califica el sobrecoste de 'sustancial', constante y amortizable, y advierte que no es comparable con los otros backends porque no implementa el circuito de cumplimiento. 'Masiva' no está medido. Nova quedó descartado por diseño: usa curvas y un compromiso que en producción exige ceremonia.
- **Fuentes:** `FIVE_BACKENDS.md:57-90: Nova: setup 4,02 s; prove_step ~250 ms CONSTANTE; RecursiveSNARK::verify 108 ms; CompressedSNARK::prove 1,84 s; verify 50 ms; 10.764/10.538 restricciones po`; `crates/nova-experiment/src/folding_chain.rs:39-49: Bn256EngineKZG/GrumpkinEngine, HyperKZG + IPA, Spartan`; `crates/nova-experiment/Cargo.toml (features): HyperKZG::setup deshabilitado en producción; exige ptau de una ceremonia (test-setup)`; `crates/nova-experiment/src/lib.rs:88-103: 'el mecanismo de plegado funciona y su coste es constante, no que Nova sea más rápido'`; `FIVE_BACKENDS.md:57-90 (setup 4,02 s; prove_step ~250 ms CONSTANTE; RecursiveSNARK::verify 108 ms; CompressedSNARK::prove 1,84 s; verify 50 ms; 10.764/10.538 restricciones; 'El sob`; `crates/nova-experiment/src/lib.rs:81-103 ('el mecanismo de plegado funciona y su coste es constante, no que Nova sea más rápido')`; `crates/nova-experiment/src/folding_chain.rs:39-49 (Bn256EngineKZG/GrumpkinEngine, HyperKZG + IPA, Spartan)`; `crates/nova-experiment/Cargo.toml [features] ('HyperKZG::setup is disabled in production builds ... ptau files from a trusted setup ceremony')`

### M8a

**PARCIAL** · gravedad alta

- **Borrador:** El STARK de Arqueo es post-cuántico
- **Lo que se puede afirmar:** En Arqueo, 'post-cuántico' significa que el camino de producción no depende de curvas, sólo de hashes: Blake3_256 en los compromisos FRI/Merkle (MerkleConSal<Blake3> desde el §538) y Rescue-Prime Rp64_256 sobre Goldilocks en los árboles y la cadena. No significa una seguridad cuántica cuantificada (SECURITY.md:553-559). Las opciones de producción (42 consultas, blowup 16, grinding 21, extensión cuadrática) están documentadas como 127 bits CONJETURADOS (crates/zk-ssl/src/lib.rs:207-221). El repo no publica su seguridad demostrable, y en la configuración de comparación equivalente los 127 bits conjeturados conviven con 29-63 demostrables (FIVE_BACKENDS.md:146). Alcanzar 128 bits demostrables cuesta ~125,6 KB por prueba y sólo se usa en el puente ISO. Nada de esto está auditado.
- **Fuentes:** `SECURITY.md:508-524: 'post-cuántico aquí significa sin supuestos de curva, no invulnerable. Grover degrada los hashes; ... techo de 63 bits de solidez sin extensión de campo ... fr`; `ARQUITECTURA.md:1568-1571: 'La resistencia post-cuántica del backend STARK no es gratuita ... exige doblar el tamaño de salida'`; `crates/zk-ssl/src/lib.rs:205-222: proof_options() = ProofOptions::new(42, 16, 21, FieldExtension::Quadratic, 8, 31, Linear, Linear), documentado como 'Configuración de 127 bits con`; `crates/zk-ssl/src/lib.rs:200 y metrics.rs:296-300: hash de los compromisos FRI/Merkle = Blake3_256; desde el §538 MerkleConSal<Blake3>`; `crates/zk-ssl-hash/src/lib.rs:100-106: el hash algebraico de árboles y cadena es Rescue-Prime Rp64_256 sobre Goldilocks`; `FIVE_BACKENDS.md tabla 'brecha': 127 bits conjeturados frente a 29/63 demostrables; 128/128 sólo con 120 queries, grinding 20 y extensión cúbica, a 125,6 KB`; `AUDITORIA.md §48.3 (l.4330-4340): la configuración de 128 bits demostrables se usa por defecto sólo en el puente ISO`; `AUDITORIA.md §106.2 (l.8894-8906): camino de producción sin dependencia de curva ('una sola familia de supuestos')`

### M8b

**PARCIAL** · gravedad media

- **Borrador:** XMSS aporta firma post-cuántica a las cabezas de época
- **Lo que se puede afirmar:** Las cabezas de época se firman con XMSS^MT (XmssMtSha2_40_8_256, RFC 8391: altura 40, 8 capas, SHA2-256), un esquema basado en hash que no añade supuestos nuevos a los del STARK (§106.3). Es un esquema CON ESTADO: reusar un índice rompe la firma de un solo uso de ese índice y permite falsificar. El repo lo mitiga con un guardián de índice persistido con fsync, probado sólo frente a la muerte del proceso (§234). La implementación es el crate xmss 0.1.0-pre.0 de RustCrypto: pre-release, sin auditoría independiente y clavado con '='. Medido: firmar 144,5-160,5 ms, verificar 2,4-2,7 ms, 18.469 B por firma (doc/xmss-evaluacion.md:133; §236). La custodia de la clave no está comprobada (§238).
- **Fuentes:** `AUDITORIA.md §106.3-106.5 (l.8908-8958): XMSS elegido porque 'no añade una familia de supuestos nueva'; 'tiene estado'; 'reusar un indice filtra la clave privada'`; `spec/NUCLEO.md:210 y 273-275: esquema XmssMtSha2_40_8_256 (RFC 8391); el apaño del OID es de xmss 0.1.0-pre.0`; `doc/xmss-evaluacion.md:21: 'Pre-release y sin auditoría independiente, declarado por el propio crate'; l.133: MT 40/8, 160,5 ms firmar (medida), 2,7 ms verificar, 18.469 B`; `AUDITORIA.md §238 (l.17268-17274): 'sin custodia declarada de la clave, una firma no tiene valor probatorio'`; `AUDITORIA.md §234: guardián del índice, durabilidad probada frente a kill -9, no frente a un corte de corriente`; `AUDITORIA.md §106.3-106.5 (l.8910-8958): 'no añade una familia de supuestos nueva'; 'tiene estado'; 'reusar un indice filtra la clave privada'`; `spec/NUCLEO.md:214, 277-278 (XmssMtSha2_40_8_256, RFC 8391; apaño de OID de xmss 0.1.0-pre.0)`; `doc/xmss-evaluacion.md:19-21 ('Pre-release y sin auditoría independiente'), :133 (MT 40/8: 160,5 ms, 2,7 ms, 18.469 B)`

### M9

**PARCIAL** · gravedad media

- **Borrador:** Evaluación: las evaluaciones no deben reportar medias
- **Lo que se puede afirmar:** La práctica del repo no es 'nunca medias', sino un protocolo. (1) Publicar RANGOS con las dos dispersiones declaradas: intra-tanda (σ ~0,5 %) y entre tandas (~9 %, sistemática), en lugar de una cifra con σ (§131). (2) Comparar sólo dentro de la misma corrida, de forma APAREADA dentro de cada repetición y con control interno (§130.3, §263). (3) No declarar consistencia de signo con n = 3 (§263). (4) Elegir el estimador según el instrumento: el MÍNIMO de repeticiones entrelazadas con la primera descartada (metrics.rs:742-756), la mediana (instrumento_cobro.rs:44) o la media acompañada de sus muestras (§89.1). Desde el §538 los tamaños se publican como banda. Los tiempos absolutos se consideran dependientes de la máquina y no se atan con gates (§304).
- **Fuentes:** `AUDITORIA.md §131.2-131.4 (l.10732-10762): 'Intra-tanda σ 0,5 % ... Entre tandas ~9 %, sistematico'; 'De cifra con σ a rango con las dos dispersiones declaradas'. El texto de §131.`; `AUDITORIA.md §130.3 (l.10686-10700): protocolo APAREADO, Δ con control interno (Δ gen ≡ 0)`; `AUDITORIA.md §263 (l.19782-19810): '1 · Un tiempo sólo se compara contra otro de la misma corrida ... 2 · La comparación válida es la EMPAREJADA dentro de la repetición ... Las med`; `crates/zk-ssl/src/metrics.rs:717-735 (los_dos_lados_del_pago_atan_la_banda): estimador = MÍNIMO de repeticiones entrelazadas, primera descartada`; `crates/zk-ssl/src/instrumento_cobro.rs:44,181: 'se reporta la mediana' (de 5)`; `AUDITORIA.md §89.1 (l.7430): sí publica una 'Media 2,35 ms, dispersion 4 %' junto a las 5 muestras`; `README.md:189-191: 'Los tiempos y tamaños medidos, con su dispersión, están en AUDITORIA.md (§130 y §131 ...)'`; `AUDITORIA.md §131.1-131.4 (l.10724-10762); la l.10762 termina en 'declaradas**:### 131.5' (truncado)`

### M10a

**CONFIRMADA** · gravedad baja · cambió con el árbol (S566–S582)

- **Borrador:** No auditado por terceros
- **Lo que se puede afirmar:** Confirmado: nada de Arqueo ha sido auditado por terceros (README.md:36; SECURITY.md:8-9). Tampoco su fork en el árbol de winterfell 0.13.1, que desde el §533/§538 añade la ocultación y desde el §575 un lector acotado de pruebas (SECURITY.md 3.7), ni su construcción (SECURITY.md:505-509).
- **Fuentes:** `README.md:36: 'No está auditado por terceros. Ninguna cantidad de tests propios lo sustituye.'`; `SECURITY.md:8-9: 'Nada de este proyecto ha sido auditado por terceros ... No la hay.'`; `SECURITY.md:470-474: 'ni el fork ni la construcción están auditados (H7)'`; `FIVE_BACKENDS.md (última línea): 'Nada de esto ha sido auditado por terceros.'`; `README.md:36 'No está auditado por terceros. Ninguna cantidad de tests propios lo sustituye.'`; `SECURITY.md:8-9; SECURITY.md:505-509 ('ni el fork ni la construcción están auditados (H7)')`; `SECURITY.md §3.7 (l.436-464) y AUDITORIA.md §575 (l.41736-41790): crates/winter-air/src/proof/acotado.rs`; `FIVE_BACKENDS.md:317 'Nada de esto ha sido auditado por terceros.'`

### M10b

**CONFIRMADA** · gravedad baja

- **Borrador:** Dependencia xmss pre-release clavada con '='
- **Lo que se puede afirmar:** Confirmado. El nodo, el verificador independiente y la CLI dependen de xmss con la versión clavada '=0.1.0-pre.0' (RustCrypto: pre-release, sin auditoría según declara el propio crate), fijada también en Cargo.lock. El probador STARK es, a su vez, un fork en el árbol de winterfell 0.13.1 ([patch.crates-io] en Cargo.toml:66-72).
- **Fuentes:** `crates/zk-ssl-node/Cargo.toml:45: xmss = "=0.1.0-pre.0"`; `crates/zk-ssl-verify/Cargo.toml:45: xmss = "=0.1.0-pre.0"`; `crates/zk-ssl-cli/Cargo.toml:78: xmss = "=0.1.0-pre.0"`; `Cargo.lock:3296-3299: name = "xmss", version = "0.1.0-pre.0", checksum 5715f4f2…`; `README.md:36-38: 'Una dependencia criptográfica (xmss, pre-release) va clavada con = y declarada'`; `AUDITORIA.md §238 (l.17262-17266): 'xmss 0.1.0-pre.0 — pre-release, sin auditoría independiente por declaración del propio crate'`; `crates/zk-ssl-node/Cargo.toml:45; crates/zk-ssl-verify/Cargo.toml:45; crates/zk-ssl-cli/Cargo.toml:78 (xmss = "=0.1.0-pre.0")`; `Cargo.lock:3296-3298 (name = "xmss", version = "0.1.0-pre.0")`

### M12a

**CONFIRMADA** · gravedad media

- **Borrador:** Los seis depósitos de Zenodo de Arqueo y sus DOI (README 'Publicación')
- **Lo que se puede afirmar:** El repo lista seis depósitos con DOI vigente (README.md:205-239; doc/ZENODO.md:326-345). Los tres primeros son preprints técnicos (21736125, 21736082 y 21905595) y citan sus versiones anteriores. Los tres últimos (22078086, 22077991 y 22076721) son trabajos de economía y política sobre confianza residual. La cita preferida es 10.5281/zenodo.21736125 (CITATION.cff). No se ha comprobado desde este entorno que los DOI resuelvan. El repo advierte que los seis depósitos PRECEDEN a correcciones del árbol (README.md:39-40).
- **Fuentes:** `README.md:196-231: (1) 'Comparative Implementation of a Zero-Knowledge Settlement Layer across Five Proof Systems: Design Findings and Measurements' 10.5281/zenodo.21736125 (anteri`; `doc/ZENODO.md:326-345: los mismos seis DOI`; `CITATION.cff: preferred-citation doi 10.5281/zenodo.21736125; autor 'Toranzo Portela, Angel Jose'`; `README.md:39-40: 'Los seis depósitos con DOI preceden a correcciones del árbol'`; `WebFetch https://doi.org/10.5281/zenodo.21736125 -> EGRESS_BLOCKED (doi.org); una WebSearch del título no devolvió el registro`; `README.md:205-239 (seis títulos y DOI 21736125, 21736082, 21905595, 22078086, 22077991, 22076721; versiones anteriores 21683239, 21677737, 21678396, 21679208)`; `doc/ZENODO.md:326-345; CITATION.cff:15 doi 10.5281/zenodo.21736125`; `README.md:39-40 'Los seis depósitos con DOI preceden a correcciones del árbol'`

### M13

**CONFIRMADA** · gravedad media

- **Borrador:** AUDITORIA incluye una sección con 'los puntos donde el autor tiene menos confianza'
- **Lo que se puede afirmar:** AUDITORIA.md §16 enumera nueve puntos de baja confianza del autor. 1) La apertura de cuentas sin autorización, con sólo un tope de cupo. 2) La congelación sin motivo ni caducidad. 3) Los grados de restricción. 4) El argumento general del lockstep C_SIBLING, no revisado por terceros. 5) Tests negativos que pueden no discriminar. 6) El bloqueo de directorio de sled al reabrir. 7) El techo 2^63−1, sin validar max_supply. 8) La instantánea que se queda atrás al añadir estado. 9) Las colisiones del árbol de nullifiers. §17 añade el orden de ataque que sugiere el autor. Es una sección temprana: el punto 16.9 quedó superado al retirarse el árbol de nullifiers (§36). Un apartado honesto de limitaciones debe citarla junto a SECURITY §3.1 (sin especificación formal del AIR) y §2.
- **Fuentes:** `README.md:167-168: remite a esa sección`; `AUDITORIA.md §16 (l.1208-1332): 16.1 open_account sin autorización (mitigado a medias con max_accounts: un atacante puede agotar el cupo); 16.2 la congelación sin justificación ni `; `AUDITORIA.md §17 (l.1334-1352): por dónde empezaría a romperlo: C_NULL_EMPTY, nonce/nullifier/recuperación, C_ACC en umbral, orden de comprobaciones en apply, serialización en stor`; `AUDITORIA.md §36 (l.2951-2965) y SECURITY.md:379-395: el árbol de nullifiers se RETIRÓ; 'cerrado en el nodo único, abierto para cualquier distribución'`; `SECURITY.md:299-313: §3.1 la ausencia de especificación del AIR, prioridad más alta`; `AUDITORIA.md §16 (l.1208-1332): 16.1 open_account, 16.2 congelación, 16.3 grados, 16.4 lockstep, 16.5 tests negativos, 16.6 bloqueo de sled, 16.7 techo 2^63−1, 16.8 instantánea, 16`; `AUDITORIA.md §17 (l.1334-1352); §36 (l.2951)`; `README.md:182 (remite a esa sección); SECURITY.md:304-317 (§3.1)`

### M4e

**CONFIRMADA** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** Techo del nodo vía RPC (§229)
- **Lo que se puede afirmar:** El techo del nodo por RPC es 248 op/s. Sale de un ajuste lineal, 0,225 + 4,035·n ms, sobre lotes de zkssl_applyMany de 1, 4, 8 y 15 envíos con tres repeticiones cada uno (AUDITORIA §229). Condiciones: nodo en memoria sin --ledger, un solo cliente en serie, sólo envíos, i5-1135G7 bajo WSL2, y antes de la ocultación del §538. La concurrencia no está medida, y con varios emisores sobre la misma raíz se pierde el 75 % de las pruebas (§230). El ciclo completo en un portátil da 4,95 ± 0,15 pagos/s, y el nodo es el 4 % de ese ciclo (§222, §229, §238). Desde el RFC-0010 (S569-S577), la vía directa con recibo hace un fsync más por operación, y ese coste no está medido. Los 248 op/s son de la vía agregada, que no lleva recibo (§576).
- **Fuentes:** `AUDITORIA.md §229 (l.16311-16325): banco H.1, lotes de 1, 4, 8 y 15 con tres repeticiones; 'Recta: 0,225 + 4,035·n'; 'TECHO DEL NODO POR RPC: 248 op/s'; RTGS 21 op/s = 8,5 %`; `AUDITORIA.md §229 (l.16383-16394): 'La CONCURRENCIA no está medida ... el nodo corrió en memoria, sin --ledger ... Solo envíos ... el techo de 248 op/s es de este hardware ... i5-1`; `AUDITORIA.md §238 (l.17243-17252): 4,95 ± 0,15 pagos/s es el ciclo completo en un portátil, no el nodo`; `AUDITORIA.md §230: con 4 clientes concurrentes contra la misma raíz, uno aplica y tres son rechazados (75 % de pruebas perdidas)`; `spec/RPC.md:387`; `AUDITORIA.md §229 (l.16305-16330) 'Recta: 0,225 + 4,035·n'; 'TECHO DEL NODO POR RPC: 248 op/s'`; `AUDITORIA.md §229 (l.16383-16394) 'La CONCURRENCIA no está medida ... sin --ledger ... Solo envíos ... es lo que da un i5-1135G7'`; `AUDITORIA.md §238 (l.17247-17251); §230 (l.16405-16410); spec/RPC.md:388`

### M6c

**CONFIRMADA** · gravedad alta

- **Borrador:** ¿Existe especificación formal del AIR?
- **Lo que se puede afirmar:** No existe especificación formal del AIR. SECURITY.md §3.1 la declara la carencia de mayor prioridad y la mantiene abierta. Sólo hay dos especificaciones en prosa, escritas por el propio autor, circuit_burn y circuit_mint, y el documento dice expresamente que 'no son un contrato' (doc/air/). La propiedad de no creación de dinero descansa en que las restricciones sean completas, y eso no está demostrado.
- **Fuentes:** `SECURITY.md:299-313 (§3.1): 'Ausencia de especificación formal del AIR — prioridad más alta ... Estado: abierto'`; `SECURITY.md:59-61: la corrección de las propiedades 'no está formalmente especificado ni auditado'`; `doc/air/ contiene sólo circuit_burn.md y circuit_mint.md; doc/air/circuit_mint.md:6-9: 'Escrita por el autor del circuito. Hereda sus puntos ciegos y NO es un contrato'`; `BACKLOG.md:2356-2370 (entrada 55) y AUDITORIA §105 (l.8798-8830): 'Las otras 26 NO se escriben ... escribirlas con la auditoria'`; `SECURITY.md:304-317 (§3.1 'Ausencia de especificación formal del AIR — prioridad más alta ... Estado: abierto')`; `ls doc/air/: circuit_burn.md, circuit_mint.md; doc/air/circuit_mint.md:6-9 'Escrita por el autor del circuito ... NO es un contrato'`; `AUDITORIA.md §105 (l.8798-8830)`

## Bloque B — la bibliografía

### B10a

**FALSA** · gravedad alta

- **Borrador:** Pillai et al. OSDI 14: "All file systems are not created equal: On the fidelity of crash-consistency applications"
- **Lo que se puede afirmar:** Título: «All File Systems Are Not Created Equal: On the Complexity of Crafting Crash-Consistent Applications».
- **Fuentes:** `WebFetch raw slebok/bibsleigh corpus/SYS/2014/OSDI-2014/OSDI-2014-PillaiCAAAA.json (dblp conf/osdi/PillaiCAAAA14): title 'All File Systems Are Not Created Equal: On the Complexity `; `GitHub code search Vonng/ddia content/en/ch8 (md): mismo título, OSDI octubre de 2014`; `https://raw.githubusercontent.com/slebok/bibsleigh/905fd7d58bf3134c14a69e43f29e14cab0134d78/corpus/SYS/2014/OSDI-2014/OSDI-2014-PillaiCAAAA.json`; `https://www.usenix.org/conference/osdi14/technical-sessions/presentation/pillai (vía WebSearch)`

### B10b

**FALSA** · gravedad media

- **Borrador:** Pillai et al. con autor "Arpaci-Dusseau, G. P."
- **Lo que se puede afirmar:** Autores: Thanumalayan Sankaranarayana Pillai, Vijay Chidambaram, Ramnatthan Alagappan, Samer Al-Kiswany, Andrea C. Arpaci-Dusseau y Remzi H. Arpaci-Dusseau.
- **Fuentes:** `bibsleigh OSDI-2014-PillaiCAAAA.json: autores Thanumalayan Sankaranarayana Pillai, Vijay Chidambaram, Ramnatthan Alagappan, Samer Al-Kiswany, Andrea C. Arpaci-Dusseau, Remzi H. Arp`; `https://raw.githubusercontent.com/slebok/bibsleigh/905fd7d58bf3134c14a69e43f29e14cab0134d78/corpus/SYS/2014/OSDI-2014/OSDI-2014-PillaiCAAAA.json`

### B11b

**FALSA** · gravedad alta

- **Borrador:** DOI 10.1145/2517349.2517373
- **Lo que se puede afirmar:** DOI 10.1145/2517349.2522726.
- **Fuentes:** `bibsleigh SOSP-2013-ChidambaramPAA.json: doi 10.1145/2517349.2522726, ISBN 978-1-4503-2388-8`; `GitHub code search bibtex/bibtex.github.io SOSP-2013-ChidambaramPAA.html: doi 10.1145/2517349.2522726`; `GitHub code search '"2517349.2517373"': 0 resultados`; `https://raw.githubusercontent.com/slebok/bibsleigh/905fd7d58bf3134c14a69e43f29e14cab0134d78/corpus/SYS/2013/SOSP-2013/SOSP-2013-ChidambaramPAA.json`; `https://dl.acm.org/doi/10.1145/2517349.2522726 (vía WebSearch)`

### B17b

**FALSA** · gravedad alta

- **Borrador:** Etiqueta "MEASURED" para el coste de reutilización en HBS-STATE v0.3
- **Lo que se puede afirmar:** El coste ~2^34 es una cifra CITADA de fuente externa (QRL, que remite a Groot Bruinderink–Hülsing). No es MEASURED ni DEDUCED y debe etiquetarse como externa.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):174 '**Cost of getting it wrong** (MEASURED, QRL curve)'; la definición de MEASURED en la línea 24 es 'taken byte by byte from a production imple`; `AUDITORIA.md §287: 'ESTATUTO, sin adornos: TODO es RAZONADO, no medido'`; `AUDITORIA.md §288: 'con su procedencia (QRL, consultado 2026-08-12)' ... 'Es medible en el propio arbol y merece un banco' (es decir, no medido todavía)`; `hbs-state/spec/HBS-STATE-v0.3 (md):24,174`; `AUDITORIA.md:21433,21490`

### B1a

**FALSA** · gravedad alta

- **Borrador:** RFC 8391: "Huelsing, A., Butin, D., Gazdag, S., Rijneveld, J., and A. Mohassel"
- **Lo que se puede afirmar:** Huelsing, A., Butin, D., Gazdag, S., Rijneveld, J., y A. Mohaisen, «XMSS: eXtended Merkle Signature Scheme», RFC 8391, DOI 10.17487/RFC8391, mayo de 2018 (IRTF, CFRG, Informational).
- **Fuentes:** `WebFetch raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC8391.yaml: contribuyentes "A. Huelsing, D. Butin, S. Gazdag, J. Rijneveld, A. Mohaisen"`; `GitHub code search matcdac/IETF_RFCs RFC-cite-refs/bib/rfc8391.bib: author = {Andreas Huelsing and Denis Butin and Stefan-Lukas Gazdag and Joost Rijneveld and Aziz Mohaisen}`; `GitHub code search PrZ3r/MSRBot.io .../RFC8391.json rawRef: "Huelsing, A., Butin, D., Gazdag, S., Rijneveld, J., and A. Mohaisen, ..."`; `GitHub code search tex2e/rfc-translater html/rfc8391.html cabecera: "A. Mohaisen / University of Central Florida"`; `https://raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC8391.yaml`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc8391.txt`; `https://raw.githubusercontent.com/liudonghua123/rfc/main/data/rfc8391.json`

### B1b

**FALSA** · gravedad alta

- **Borrador:** BibTeX de RFC 8391 con autor "Azam Mohassel"
- **Lo que se puede afirmar:** BibTeX: author = {Andreas Huelsing and Denis Butin and Stefan-Lukas Gazdag and Joost Rijneveld and Aziz Mohaisen}. El RFC escribe «Huelsing», sin diéresis.
- **Fuentes:** `matcdac/IETF_RFCs RFC-cite-refs/bib/rfc8391.bib (BibTeX del RFC Editor): "Aziz Mohaisen"`; `relaton RFC8391.yaml: "A. Mohaisen"`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc8391.txt`

### B2b

**FALSA** · gravedad media

- **Borrador:** BibTeX de RFC 8554 con "McGrew秩序"
- **Lo que se puede afirmar:** author = {David McGrew and Michael Curcio and Scott Fluhrer}.
- **Fuentes:** `squinky86/SwATips tips/20220919.bib y yazhsab/qbitel-bridge docs/research_paper/references.bib: author = {David McGrew and Michael Curcio and Scott Fluhrer}`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc8554.txt`

### B3a

**FALSA** · gravedad alta

- **Borrador:** RFC 9162 citado como "Laurie, B., Lin, C., Kasper, E., & Messeri, E."
- **Lo que se puede afirmar:** Laurie, B., Messeri, E., y R. Stradling, «Certificate Transparency Version 2.0», RFC 9162, DOI 10.17487/RFC9162, diciembre de 2021 (IETF, Experimental; deja obsoleto el RFC 6962).
- **Fuentes:** `relaton RFC9162.yaml: "B. Laurie, E. Messeri, R. Stradling", December 2021, DOI 10.17487/RFC9162, "obsoletes RFC 6962"`; `GitHub code search cleverbase/scal3 docs/report/thresholds.bib: author = {Ben Laurie and Eran Messeri and Rob Stradling}`; `WebSearch: "RFC 9162 is authored by B. Laurie, E. Messeri, and R. Stradling, published in December 2021 as an Experimental RFC"`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc9162.txt`; `https://raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC9162.yaml`; `https://raw.githubusercontent.com/liudonghua123/rfc/main/data/rfc9162.json`

### B3b

**FALSA** · gravedad alta

- **Borrador:** BibTeX de RFC 9162: "Ben Laurie and Adam Langley and Eran Kasper and Emilia Messeri"
- **Lo que se puede afirmar:** RFC 9162: author = {Ben Laurie and Eran Messeri and Rob Stradling}. RFC 6962: author = {Ben Laurie and Adam Langley and Emilia Kasper}.
- **Fuentes:** `matcdac/IETF_RFCs RFC-cite-refs/bib/rfc6962.bib: author = {Ben Laurie and Adam Langley and Emilia Kasper} (esos son los autores del RFC 6962)`; `cleverbase/scal3 thresholds.bib (RFC 9162): {Ben Laurie and Eran Messeri and Rob Stradling}`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc6962.txt`; `https://raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC6962.yaml`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc9162.txt`

### B7a

**FALSA** · gravedad alta

- **Borrador:** Nova: "Kothapalli, Setty & Tzialla (2022). Nova: Recursive Zero-Knowledge Proofs without Trusted Setup. CCS 2022"
- **Lo que se puede afirmar:** Kothapalli, A., Setty, S., y Tzialla, I., «Nova: Recursive Zero-Knowledge Arguments from Folding Schemes», en Y. Dodis y T. Shrimpton (eds.), CRYPTO 2022, Parte IV, LNCS 13510, Springer, 2022, pp. 359–388, DOI 10.1007/978-3-031-15985-5_13; ePrint 2021/370.
- **Fuentes:** `WebFetch raw cryptobib bib/crypto_C22.bib, C:KotSetTzi22: 'Nova: Recursive Zero-Knowledge Arguments from Folding Schemes', pages 359--388, doi 10.1007/978-3-031-15985-5_13`; `GitHub code search BaDaaS/cryptography.academy (ePrint 2022/1565): '... In Yevgeniy Dodis and Thomas Shrimpton, editors, CRYPTO 2022, Part IV, volume 13510 of LNCS, pages 359–388'`; `GitHub code search arnaucube/math paper-notes.bib: ePrint 2021/370`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_C22.bib`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_EPRINT21.bib`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/abbrev3.bib`

### B7b

**FALSA** · gravedad alta

- **Borrador:** DOI de Nova 10.1145/3548606.3560610
- **Lo que se puede afirmar:** 10.1145/3548606.3560610 = Roy Chowdhury, A., Ding, B., Jha, S., Liu, W., Zhou, J., «Strengthening Order Preserving Encryption with Differential Privacy», ACM CCS 2022, pp. 2519–2533. El DOI de Nova es 10.1007/978-3-031-15985-5_13.
- **Fuentes:** `GitHub code search freezed-corpse-143/topaperlist PAPERS/A/CSS/2022.jsonl (BibTeX de dblp conf/ccs/0001DJLZ22): 'Strengthening Order Preserving Encryption with Differential Privacy`; `WebSearch '"10.1145/3548606.3560610"': dl.acm.org/doi/10.1145/3548606.3560610 = 'Strengthening Order Preserving Encryption with Differential Privacy'`; `https://dl.acm.org/doi/10.1145/3548606.3560610 (vía WebSearch)`; `github.com/freezed-corpse-143/topaperlist PAPERS/A/CSS/2022.jsonl`; `github.com/mali-kh/papers papers/ccs/2022/papers.bib`

### B7c

**FALSA** · gravedad media

- **Borrador:** Nova en "pp. 2161-2174"
- **Lo que se puede afirmar:** Nova: pp. 359–388 de CRYPTO 2022, Parte IV (LNCS 13510).
- **Fuentes:** `cryptobib bib/crypto_C22.bib: pages 359--388`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_C22.bib`

### B7d

**FALSA** · gravedad media

- **Borrador:** BibTeX de Nova con nombre de pila "Tsvitcha"
- **Lo que se puede afirmar:** author = {Abhiram Kothapalli and Srinath Setty and Ioanna Tzialla}.
- **Fuentes:** `cryptobib bib/crypto_C22.bib: author = "Abhiram Kothapalli and Srinath Setty and Ioanna Tzialla"`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_C22.bib`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_EPRINT21.bib`

### B8a

**FALSA** · gravedad alta

- **Borrador:** IVC: "Valiant, P. (2008). Incrementally verifiable computation or proof of execution. FOCS 2008, pp. 137-146"
- **Lo que se puede afirmar:** Valiant, P., «Incrementally Verifiable Computation or Proofs of Knowledge Imply Time/Space Efficiency», en R. Canetti (ed.), TCC 2008, LNCS 4948, Springer, 2008, pp. 1–18, DOI 10.1007/978-3-540-78524-8_1.
- **Fuentes:** `WebFetch raw cryptobib bib/crypto_TCC08.bib, TCC:Valiant08: 'Incrementally Verifiable Computation or Proofs of Knowledge Imply Time/Space Efficiency', pages 1--18, doi 10.1007/978-`; `GitHub code search abbrev3.bib (cryptobib): tcc08vol = "4948", tcc08ed = "Ran Canetti"`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_TCC08.bib`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/abbrev3.bib`

### B8b

**FALSA** · gravedad alta

- **Borrador:** DOI de Valiant 10.1109/FOCS.2008.82
- **Lo que se puede afirmar:** 10.1109/FOCS.2008.82 = Indyk, P., y Ružić, M., «Near-Optimal Sparse Recovery in the L1 Norm», FOCS 2008, pp. 199–207. El artículo de Valiant sobre IVC no es de FOCS.
- **Fuentes:** `GitHub code search lizichen/Scholars-Collaboration-Network (export Scopus): 'Indyk, P., Ružić, M., "Near-optimal sparse recovery in the L1 norm", 2008, FOCS, 199–207, 10.1109/FOCS.`; `GitHub code search vintageplayer/SciBase-ACMTemp (referencias ACM TODS 35(4)): 'Piotr Indyk, Milan Ruzic, Near-Optimal Sparse Recovery in the L1 Norm, ... FOCS 2008, p.199-207 [doi`; `thomwiggers/cryptobib-export-split-demo bib/crypto_FOCS08.bib contiene doi 10.1109/FOCS.2008.82 en una entrada que no es de Valiant`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_FOCS08.bib`

### B9a

**FALSA** · gravedad alta

- **Borrador:** PCD: "Chiesa & Tromer (2013). Proof-carrying data and incrementally verifiable computation. ITCS 2013, pp. 310-321"
- **Lo que se puede afirmar:** Chiesa, A., y Tromer, E., «Proof-Carrying Data and Hearsay Arguments from Signature Cards», en A. C.-C. Yao (ed.), Innovations in Computer Science (ICS 2010), Tsinghua University Press, Pekín, enero de 2010, pp. 310–331 (sin DOI).
- **Fuentes:** `WebFetch raw cryptobib bib/crypto_ITCS10.bib, ITCS:ChiTro10: 'Proof-Carrying Data and Hearsay Arguments from Signature Cards', pages 310--331, year 2010 (sin doi)`; `GitHub code search pfasante/phd_thesis abbrev3.bib: itcs10name = "ICS 2010"; itcs10ed = "Andrew Chi-Chih Yao"; itcs10month = jan`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_ITCS10.bib`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/abbrev3.bib`

### B9b

**FALSA** · gravedad alta

- **Borrador:** DOI de Chiesa-Tromer 10.1145/2422436.2422472
- **Lo que se puede afirmar:** 10.1145/2422436.2422472 pertenece a las actas de ITCS 2013 pero no designa ningún artículo sobre PCD de Chiesa y Tromer (con toda probabilidad es un registro de sesión). Debe citarse el artículo de ICS 2010, que no tiene DOI.
- **Fuentes:** `WebFetch raw boudinfl/acm-cr data/acm-dl/icts/icts-2013.bib (volcado ACM DL de ITCS'13): no hay entrada con doi …2422472; las vecinas son …2422471 = Klauck & Prakash, 'Streaming Co`; `mismo volcado: el único artículo de ITCS'13 con Chiesa y Tromer es Ben-Sasson, Chiesa, Genkin, Tromer, 'Fast Reductions from RAMs to Delegatable Succinct Constraint Satisfaction Pr`; `WebSearch '"10.1145/2422436.2422472"': sin resultado directo`; `https://raw.githubusercontent.com/boudinfl/acm-cr/0e65d68f5caae0eb902f4a3881575fd37fdb53e8/data/acm-dl/icts/icts-2013.bib`; `WebSearch '"10.1145/2422436.2422472"' (sin coincidencia directa)`

### B11a

**PARCIAL** · gravedad media · **el escéptico revocó el primer veredicto** (FALSA)

- **Borrador:** Chidambaram et al., "Optimistic crash consistency", SOSP 2013, pp. 361-377
- **Lo que se puede afirmar:** Chidambaram, V., Pillai, T. S., Arpaci-Dusseau, A. C., y Arpaci-Dusseau, R. H., «Optimistic Crash Consistency», SOSP '13 (24th ACM Symposium on Operating Systems Principles), Farmington (PA), ACM, 2013, pp. 228–243 (no 361–377).
- **Fuentes:** `WebFetch raw slebok/bibsleigh corpus/SYS/2013/SOSP-2013/SOSP-2013-ChidambaramPAA.json (dblp conf/sosp/ChidambaramPAA13): pages 228-243`; `GitHub code search davidar/dblp.yaml conf/sosp.bib: conf/sosp/ChidambaramPAA13 pages = {228-243}`; `https://raw.githubusercontent.com/slebok/bibsleigh/905fd7d58bf3134c14a69e43f29e14cab0134d78/corpus/SYS/2013/SOSP-2013/SOSP-2013-ChidambaramPAA.json`; `https://sigops.org/s/conferences/sosp/2013/papers/p228-chidambaram.pdf (vía WebSearch)`

### B12

**PARCIAL** · gravedad media

- **Borrador:** POSIX fsync: "IEEE Std 1003.1-2017"
- **Lo que se puede afirmar:** IEEE Std 1003.1-2024 (POSIX.1-2024, The Open Group Base Specifications Issue 8), System Interfaces, fsync(); o bien IEEE Std 1003.1-2017 (Issue 7, 2018 edition) declarando la edición. Su RATIONALE admite una implementación nula de fsync() cuando _POSIX_SYNCHRONIZED_IO no está definido.
- **Fuentes:** `WebSearch (IEEE SA / IEEE Xplore 8277153): POSIX.1-2017 = IEEE Std 1003.1-2017 = The Open Group Base Specifications Issue 7; publicado el 31-01-2018; "This standard has been obsole`; `WebSearch (IEEE Xplore 10555529): IEEE Std 1003.1-2024, Issue 8, publicado en junio de 2024`; `WebSearch (man fsync(3p), RATIONALE): "If _POSIX_SYNCHRONIZED_IO is not defined, the wording relies heavily on the conformance document to tell the user what can be expected from t`; `https://raw.githubusercontent.com/bscothern/SwiftyPOSIX/HEAD/POSIX.1-2017/functions/fsync.html`; `https://ieeexplore.ieee.org/document/10555529 (vía WebSearch)`

### B14b

**PARCIAL** · gravedad media

- **Borrador:** "EIP-8310 y otras especificaciones recomiendan marcas de agua altas"
- **Lo que se puede afirmar:** EIP-8310 (Draft) exige una marca de agua alta autoritativa, persistida de forma duradera antes de entregar la firma, y la detección de su regresión. No se conoce otra especificación de las citadas que use ese término.
- **Fuentes:** `eip-8310 (md): "Verify the index is strictly greater than the authoritative high-water mark"; "Durably persist the advanced high-water mark, written, flushed (fsync or platform equ`; `raw draft-ietf-pquip-hbs-state (md): no contiene 'high-water' ni 'watermark' (habla de counter/state)`; `eip-8310 (md) no cita RFC 8391, SP 800-208 ni RFC 10033 (WebFetch)`; `https://raw.githubusercontent.com/ethereum/EIPs/master/EIPS/eip-8310 (md)`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc10033.txt`

### B17a

**PARCIAL** · gravedad alta

- **Borrador:** "2^34 evaluaciones de hash tras reutilizar un índice": origen de la cifra
- **Lo que se puede afirmar:** La curva 2^34/2^23/2^18 (dos, tres y cuatro firmas con el mismo índice) procede del blog de QRL «Statefulness and security». La fuente primaria es Groot Bruinderink y Hülsing (SAC 2017; ePrint 2016/1042), cuyo resumen concluye que WOTS con parámetros típicos no ofrece seguridad razonable ante ataques de dos mensajes. La cifra concreta ~2^34 (m=256, w=16) debe cotejarse con el PDF antes de atribuírsela. Fluhrer (ePrint 2023/1905) revisa ese análisis.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):174-175 ('MEASURED, QRL curve ... 2^34 hashes'); hbs-state/src/lib.rs:341-342; hbs-state/spec/state-vectors-v0.3.json:99`; `AUDITORIA.md §288: 'La CURVA de la reutilizacion, incorporada a la 84 con su procedencia (QRL, consultado 2026-08-12): 2 firmas ~2^34 hashes, 3 ~2^23, 4 ~2^18'; BACKLOG.md:629-633`; `WebSearch: blog QRL 'Statefulness and security' (Peter Waterland, theqrl.org/blog y Medium): 'With 2 signatures this requires 2^34 hashes, 3 signatures require 2^23 hashes, and 4 s`; `WebSearch (dos extractos sobre eprint.iacr.org/2016/1042): para WOTS con m=256 y w=16, tras dos firmas de mensajes aleatorios hacen falta ~2^34 mensajes/hashes para encontrar un te`; `raw draft-ietf-pquip-hbs-state (md), Introduction: 'If an attacker is able to obtain signatures for two different messages created using the same OTS key, it is computationally fea`; `https://www.theqrl.org/blog/statefulness-and-security/ (vía WebSearch)`; `https://eprint.iacr.org/2016/1042 (vía WebSearch; PDF bloqueado)`; `AUDITORIA.md:21490`

### B17c

**PARCIAL** · gravedad media

- **Borrador:** "at the second reuse of an index ... 2^34 hashes"
- **Lo que se puede afirmar:** ~2^34 hashes corresponde a dos firmas con el mismo índice (un reúso), según la curva de QRL.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):174-175; hbs-state/src/lib.rs:341`; `crates/zk-ssl-guardian/src/lib.rs:343-344 ('curva QRL: a la segunda repetición, ~2^34 hashes') y crates/zk-ssl-cli/src/witness.rs:1555 ('a la cuarta repeticion, ~2^18 hashes'): 're`; `AUDITORIA.md §288: '2 firmas ~2^34'`; `hbs-state/src/lib.rs:341`; `hbs-state/spec/state-vectors-v0.3.json:99`; `crates/zk-ssl-guardian/src/lib.rs:344`

### B17d

**PARCIAL** · gravedad media

- **Borrador:** "reusing one [index] leaks the key"
- **Lo que se puede afirmar:** Reutilizar un índice permite falsificar firmas con coste factible y la clave debe darse por perdida, pero no revela la semilla XMSS completa.
- **Fuentes:** `hbs-state/README.md:15; hbs-state/src/lib.rs:4, 274, 587`; `relaton RFC10033.yaml, resumen: 'as double-signing with the same OTS key allows forgeries'`; `raw draft-ietf-pquip-hbs-state (md), Introduction: 'it is computationally feasible for that attacker to create forgeries'`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc10033.txt`; `https://raw.githubusercontent.com/ethereum/EIPs/master/EIPS/eip-8310 (md)`

### B4c

**PARCIAL** · gravedad media

- **Borrador:** La sección 4 de RFC 10033 "demands"/exige las cuatro propiedades ACID sobre el estado
- **Lo que se puede afirmar:** El §4 de RFC 10033 (Informational, sin BCP 14) dice que los sistemas de gestión de estado «should satisfy all ACID properties»: lo recomienda, no lo exige de forma normativa. Sí afirma, en minúscula, que el sistema «must not release a signature without irrevocably and correctly updating the state».
- **Fuentes:** `raw draft-ietf-pquip-hbs-state (md), sección 'Requirements for Secure State Management': "State management systems should satisfy all _ACID_ properties: _Atomicity_ ... _Consistenc`; `hbs-state/README.md:21-23 ('its section 4 demands the four ACID properties'); hbs-state/spec/HBS-STATE-v0.3 (md):14-16; hbs-state/.zenodo.json:19`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc10033.txt`; `https://raw.githubusercontent.com/kesara/watcher/main/rfc/rfc10033.txt`

### B5c

**PARCIAL** · gravedad media

- **Borrador:** "NIST is working on a revision that would enable key export"
- **Lo que se puede afirmar:** NIST anunció en noviembre de 2024 (pqc-forum) que planea revisar SP 800-208 para permitir la exportación de claves; no consta un borrador público de la revisión. El §8.1 vigente dice «The cryptographic module shall not allow for the export of private keying material».
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):318-320`; `WebSearch (hilo pqc-forum 'Update on SP 800-208', 8 de noviembre de 2024): NIST "plans to revise SP 800-208 to enable key export while also mitigating security concerns"; John Kels`; `WebSearch 'SP 800-208 Rev. 1 draft': no se localiza ningún borrador público de revisión (csrc.nist.gov bloqueado)`; `https://groups.google.com/a/list.nist.gov/g/pqc-forum/c/t8SSD639Z0M (vía WebSearch)`; `https://raw.githubusercontent.com/neurogaussthm/NIST-SP800-Data/HEAD/try2/SP800-208.txt`

### B18a

**NO_VERIFICABLE** · gravedad media

- **Borrador:** DOI Zenodo de hbs-state 10.5281/zenodo.22993572
- **Lo que se puede afirmar:** No citar 10.5281/zenodo.22993572 hasta abrir el registro de Zenodo y comprobar si es el DOI de versión de hbs-state 0.2.0.
- **Fuentes:** `grep en ambos repositorios (sin target/): 0 apariciones de 22993572`; `WebSearch '"10.5281/zenodo.22993572"': sin resultado`; `zenodo.org y doi.org bloqueados (EGRESS_BLOCKED)`; `github.com/atoranzo/hbs-state/releases/tag/v0.2.0 y v0.1.0 (WebFetch): las notas no mencionan ningún DOI`; `hbs-state git log 476eaaf, a960828`; `https://raw.githubusercontent.com/atoranzo/hbs-state/main/CITATION.cff`; `WebSearch '"zenodo.22993572"' (sin resultado)`

### B18b

**NO_VERIFICABLE** · gravedad media

- **Borrador:** DOI Zenodo de hbs-state 10.5281/zenodo.22980547
- **Lo que se puede afirmar:** 10.5281/zenodo.22980547 es el DOI de concepto que el autor declara en CITATION.cff; conviene citarlo como tal hasta verificarlo en Zenodo.
- **Fuentes:** `hbs-state/CITATION.cff:16 'doi: 10.5281/zenodo.22980547'`; `commit a960828 (hbs-state, 2026-09-27): 'This release exists so that the concept DOI resolves to the corrected text, and so that CITATION.cff is inside the archive'`; `WebSearch '"10.5281/zenodo.22980547"': sin resultado; zenodo.org bloqueado`; `hbs-state/CITATION.cff:16`; `hbs-state git log 476eaaf`

### B18c

**NO_VERIFICABLE** · gravedad baja

- **Borrador:** DOI Zenodo de Arqueo: 21736125, 21736082, 21905595, 22078086, 22077991, 22076721
- **Lo que se puede afirmar:** Los seis DOI de Arqueo son los que declara el autor. Deben citarse así, o comprobarse en Zenodo antes de publicar.
- **Fuentes:** `README.md:206, 215, 221, 226, 229, 232; doc/ZENODO.md:326-345; CITATION.cff:15 (21736125 como cita preferida)`; `doc/preprints/README.md:81-87: 'Las dos SÍ estaban depositadas desde el 2026-08-01, comprobado abriendo los registros' (21736125 y 21736082, anotado el 2026-08-27); 'El registro de`; `README.md:209-223: versiones anteriores 21683239, 21677737, 21678396, 21679208`; `WebSearch por cada DOI o título: solo devuelve el README de GitHub del propio proyecto (circular) o nada; zenodo.org bloqueado`; `README.md`; `doc/ZENODO.md`; `CITATION.cff`

### B10c

**CONFIRMADA** · gravedad baja

- **Borrador:** Pillai et al., OSDI 14, pp. 433-448
- **Lo que se puede afirmar:** 11th USENIX Symposium on Operating Systems Design and Implementation (OSDI '14), Broomfield (CO), octubre de 2014, pp. 433–448, USENIX Association (sin DOI).
- **Fuentes:** `bibsleigh OSDI-2014-PillaiCAAAA.json: pages 433-448, publisher USENIX Association, ee https://www.usenix.org/conference/osdi14/technical-sessions/presentation/pillai`; `bojieli/ai-infra-book references/text/checkfreq.txt: 'OSDI ’14, Broomfield, CO, USA, October 6-8, 2014, pages 433–448. USENIX Association'`; `https://raw.githubusercontent.com/slebok/bibsleigh/905fd7d58bf3134c14a69e43f29e14cab0134d78/corpus/SYS/2014/OSDI-2014/OSDI-2014-PillaiCAAAA.json`

### B13

**CONFIRMADA** · gravedad baja

- **Borrador:** Merkle, "A certified digital signature", CRYPTO '89, LNCS 435, pp. 218-238, DOI 10.1007/0-387-34805-0_21
- **Lo que se puede afirmar:** Merkle, R. C., «A Certified Digital Signature», en G. Brassard (ed.), CRYPTO '89, LNCS 435, Springer, 1990, pp. 218–238, DOI 10.1007/0-387-34805-0_21.
- **Fuentes:** `WebFetch raw cryptobib bib/crypto_C89.bib, C:Merkle89a: 'A Certified Digital Signature', pages 218--238, year 1990, doi 10.1007/0-387-34805-0_21`; `GitHub code search abbrev3.bib: crypto89ed = "Gilles Brassard", crypto89vol = "435"`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_C89.bib`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/abbrev3.bib`

### B14a

**CONFIRMADA** · gravedad baja

- **Borrador:** Existe EIP-8310 y trata del estado de firmas con estado
- **Lo que se puede afirmar:** EIP-8310, «Post-Quantum Keystore for Stateful Keys» (Shukla, Wagner, Singh, Ballet, Drake, Moroz Liebl, Ramanujam, Naiyer, Coratger, Leepaisalsuwanna), Draft, Standards Track (Interface), creado el 2026-06-19.
- **Fuentes:** `WebFetch raw.githubusercontent.com/ethereum/EIPs/master/EIPS/eip-8310 (md) front matter: eip 8310, title 'Post-Quantum Keystore for Stateful Keys', status Draft, type Standards Tra`; `WebSearch: ethereum-magicians.org/t/eip-8310-post-quantum-keystore-for-stateful-keys/28853 y eips.ethereum.org/EIPS/eip-8310`; `https://raw.githubusercontent.com/ethereum/EIPs/master/EIPS/eip-8310 (md)`; `https://ethereum-magicians.org/t/eip-8310-post-quantum-keystore-for-stateful-keys/28853 (vía WebSearch)`

### B15a

**CONFIRMADA** · gravedad baja

- **Borrador:** RFC 3552 (Rescorla, Korver, julio de 2003, BCP 72)
- **Lo que se puede afirmar:** Rescorla, E., y B. Korver, «Guidelines for Writing RFC Text on Security Considerations», BCP 72, RFC 3552, DOI 10.17487/RFC3552, julio de 2003.
- **Fuentes:** `WebFetch relaton RFC3552.yaml: 'Guidelines for Writing RFC Text on Security Considerations', DOI 10.17487/RFC3552, 2003-07, E. Rescorla, B. Korver, BCP 72, rama IAB, sin relaciones`; `WebSearch: existe el esfuerzo RFC3552bis (github.com/IETF-SAAG/RFC3552bis) sin RFC publicado que lo sustituya`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc3552.txt`; `https://raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC3552.yaml`; `https://raw.githubusercontent.com/liudonghua123/rfc/main/data/rfc3552.json`

### B15b

**CONFIRMADA** · gravedad baja

- **Borrador:** RFC 8126 (Cotton, Leiba, Narten, junio de 2017, BCP 26)
- **Lo que se puede afirmar:** Cotton, M., Leiba, B., y T. Narten, «Guidelines for Writing an IANA Considerations Section in RFCs», BCP 26, RFC 8126, DOI 10.17487/RFC8126, junio de 2017 (deja obsoleto el RFC 5226).
- **Fuentes:** `WebFetch relaton RFC8126.yaml: 'Guidelines for Writing an IANA Considerations Section in RFCs', DOI 10.17487/RFC8126, 2017-06, M. Cotton, B. Leiba, T. Narten, BCP 26, IETF`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc8126.txt`; `https://raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC8126.yaml`

### B15c

**CONFIRMADA** · gravedad baja

- **Borrador:** RFC 2119
- **Lo que se puede afirmar:** Bradner, S., «Key words for use in RFCs to Indicate Requirement Levels», BCP 14, RFC 2119, DOI 10.17487/RFC2119, marzo de 1997.
- **Fuentes:** `WebFetch relaton RFC2119.yaml: 'Key words for use in RFCs to Indicate Requirement Levels', DOI 10.17487/RFC2119, March 1997, S. Bradner, BCP 14`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc2119.txt`; `https://raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC2119.yaml`

### B15d

**CONFIRMADA** · gravedad baja

- **Borrador:** RFC 8174
- **Lo que se puede afirmar:** Leiba, B., «Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words», BCP 14, RFC 8174, DOI 10.17487/RFC8174, mayo de 2017.
- **Fuentes:** `WebFetch relaton RFC8174.yaml: 'Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words', DOI 10.17487/RFC8174, May 2017, B. Leiba, BCP 14, Updates RFC 2119`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc8174.txt`; `https://raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC8174.yaml`

### B15e

**CONFIRMADA** · gravedad baja

- **Borrador:** FIPS 202 (agosto de 2015, DOI 10.6028/NIST.FIPS.202)
- **Lo que se puede afirmar:** NIST, «SHA-3 Standard: Permutation-Based Hash and Extendable-Output Functions», FIPS PUB 202, agosto de 2015, DOI 10.6028/NIST.FIPS.202.
- **Fuentes:** `GitHub code search usnistgov/800-63-3 sp800-63b/references (md): '[FIPS 202] ... SHA-3 Standard: Permutation-Based Hash and Extendable-Output Functions, August 2015, http://dx.doi.`; `pingidentity/ldapsdk docs/specs/draft-melnikov-scram-sha3-512-05.txt: 'Dworkin, M., "SHA-3 Standard ...", FIPS PUB 202, DOI 10.6028/nist.fips.202, August 2015'`; `https://raw.githubusercontent.com/CloudSecurityAlliance-DataSets/dataset-public-laws-regulations-standards/main/reference/nist.gov/fips-202/fips-202 (md)`; `https://raw.githubusercontent.com/ietf-tools/bibxml-data-archive/main/bibxml-nist/reference.NIST.FIPS.202.xml`

### B16a

**CONFIRMADA** · gravedad baja

- **Borrador:** Omitido: McGrew, Kampanakis, Fluhrer, Gazdag, Butin, Buchmann, "State Management for Hash-Based Signatures" (SSR 2016, ePrint 2016/357)
- **Lo que se puede afirmar:** McGrew, D., Kampanakis, P., Fluhrer, S., Gazdag, S.-L., Butin, D., y Buchmann, J., «State Management for Hash-Based Signatures», Security Standardisation Research (SSR 2016), LNCS 10074, Springer, 2016, pp. 244–260, DOI 10.1007/978-3-319-49100-4_11; ePrint 2016/357.
- **Fuentes:** `GitHub code search tex2e/rfc-translater html/rfc8554.html, ref [STMGMT]: '... SSR 2016: Security Standardisation Research (SSR) pp. 244-260, Lecture Notes in Computer Science Vol. `; `Braxvang/nist-llm-digital-assistant (texto de NIST SP 800-208, ref. [8]): 'Cryptology ePrint Archive, Report 2016/357'`; `raw draft-ietf-pquip-hbs-state (md), ref. MCGREW (el propio RFC 10033 la cita)`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc8554.txt`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_EPRINT16.bib`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc10033.txt`

### B16b

**CONFIRMADA** · gravedad baja

- **Borrador:** Omitido: Bruinderink & Hülsing, "Oops, I did it again" (SAC 2017)
- **Lo que se puede afirmar:** Groot Bruinderink, L., y Hülsing, A., «“Oops, I Did It Again” – Security of One-Time Signatures Under Two-Message Attacks», en C. Adams y J. Camenisch (eds.), SAC 2017, LNCS 10719, Springer, 2018, pp. 299–322, DOI 10.1007/978-3-319-72565-9_15; ePrint 2016/1042.
- **Fuentes:** `WebFetch raw cryptobib bib/crypto_SAC17.bib, SAC:BruHul17: 'Leon Groot Bruinderink and Andreas Hülsing', '“Oops, I Did It Again” - Security of One-Time Signatures Under Two-Message`; `abbrev3.bib: sac17vol = "10719", sac17ed = "Carlisle Adams and Jan Camenisch"`; `raw draft-ietf-pquip-hbs-state (md) ref BH16: eprint.iacr.org/2016/1042`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_SAC17.bib`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_EPRINT16.bib`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/abbrev3.bib`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc10033.txt`

### B16c

**CONFIRMADA** · gravedad baja

- **Borrador:** Omitido: Haber & Stornetta, "How to time-stamp a digital document" (J. Cryptology 1991)
- **Lo que se puede afirmar:** Haber, S., y Stornetta, W. S., «How to Time-Stamp a Digital Document», Journal of Cryptology 3(2):99–111, 1991, DOI 10.1007/BF00196791.
- **Fuentes:** `WebFetch raw cryptobib bib/crypto_JC91.bib, JC:HabSto91: 'Stuart Haber and W. Scott Stornetta', pages 99--111, volume 3, number 2, year 1991, doi 10.1007/BF00196791`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_JC91.bib`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_C90.bib`

### B16d

**CONFIRMADA** · gravedad baja

- **Borrador:** Omitido: Crosby & Wallach, "Efficient Data Structures for Tamper-Evident Logging" (USENIX Security 2009)
- **Lo que se puede afirmar:** Crosby, S. A., y Wallach, D. S., «Efficient Data Structures for Tamper-Evident Logging», 18th USENIX Security Symposium, Montreal, agosto de 2009, pp. 317–334.
- **Fuentes:** `GitHub code search vintageplayer/SciBase-ACMTemp (referencias ACM): 'Scott A. Crosby, Dan S. Wallach, Efficient data structures for tamper-evident logging, Proceedings of the 18th `; `WebSearch: dl.acm.org/doi/10.5555/1855768.1855788; usenix.org/conference/usenixsecurity09/...`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_USENIX09.bib`; `https://www.usenix.org/conference/usenixsecurity09/technical-sessions/presentation/efficient-data-structures-tamper-evident (vía WebSearch)`

### B16e

**CONFIRMADA** · gravedad baja

- **Borrador:** Omitido: Schneier & Kelsey, "Secure audit logs to support computer forensics" (ACM TISSEC 1999)
- **Lo que se puede afirmar:** Schneier, B., y Kelsey, J., «Secure Audit Logs to Support Computer Forensics», ACM Transactions on Information and System Security 2(2):159–176, mayo de 1999, DOI 10.1145/317087.317089.
- **Fuentes:** `GitHub code search paritytech/consensus pdf/sec.bib y jeremylongshore/startaitools.com: 'Schneier, B., & Kelsey, J. (1999). Secure Audit Logs to Support Computer Forensics. ACM TIS`; `github.com/davidar/dblp.yaml journals/tissec.bib`; `github.com/vintageplayer/SciBase-ACMTemp Journals/TISSEC/Volumes/Volume 2 Issue 2, May 1999/Pages: 159-176/doi.txt`

### B16f

**CONFIRMADA** · gravedad baja

- **Borrador:** Omitido: Haeberlen, Kouznetsov, Druschel, "PeerReview" (SOSP 2007)
- **Lo que se puede afirmar:** Haeberlen, A., Kouznetsov, P., y Druschel, P., «PeerReview: Practical Accountability for Distributed Systems», SOSP '07, ACM, 2007, pp. 175–188, DOI 10.1145/1294261.1294279.
- **Fuentes:** `WebFetch raw slebok/bibsleigh corpus/SYS/2007/SOSP-2007/SOSP-2007-HaeberlenKD.json (dblp conf/sosp/HaeberlenKD07): pages 175-188, doi 10.1145/1294261.1294279`; `https://raw.githubusercontent.com/slebok/bibsleigh/905fd7d58bf3134c14a69e43f29e14cab0134d78/corpus/SYS/2007/SOSP-2007/SOSP-2007-HaeberlenKD.json`

### B16g

**CONFIRMADA** · gravedad baja

- **Borrador:** Omitido: Laurie, "Certificate Transparency" (CACM 2014) o RFC 6962
- **Lo que se puede afirmar:** Laurie, B., «Certificate Transparency», Communications of the ACM 57(10):40–46, octubre de 2014, DOI 10.1145/2659897; o RFC 6962 (Laurie, Langley, Kasper, junio de 2013).
- **Fuentes:** `GitHub code search textbrowser/spot-on-shared-pages (referencias de ACM Queue): 'Laurie, B. 2014. Certificate transparency. Communications of the ACM 57(10), 40–46; https://dl.acm.`; `ept/ddia-references chapter-12-refs (md): 'Ben Laurie: Certificate Transparency, ACM Queue, volume 12, number 8, pages 10-19, August 2014, doi:10.1145/2668152.2668154'`; `relaton RFC6962.yaml`; `github.com/aj-stein-nist/semi-transparent README.md`; `github.com/syclops/caps doc/paper/bib.bib`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc6962.txt`

### B16h

**CONFIRMADA** · gravedad baja

- **Borrador:** Omitido: Melara et al., "CONIKS" (USENIX Security 2015)
- **Lo que se puede afirmar:** Melara, M. S., Blankstein, A., Bonneau, J., Felten, E. W., y Freedman, M. J., «CONIKS: Bringing Key Transparency to End Users», 24th USENIX Security Symposium, Washington D.C., agosto de 2015, pp. 383–398.
- **Fuentes:** `GitHub code search nicola/decentralized-research coniks.bib y foks-proj/foks-whitepaper refs.bib: 'Marcela S. Melara and Aaron Blankstein and Joseph Bonneau and Edward W. Felten an`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_USENIX15.bib`

### B16i

**CONFIRMADA** · gravedad baja

- **Borrador:** Omitido: Bitansky, Canetti, Chiesa, Tromer, "Recursive composition and bootstrapping for SNARKs and proof-carrying data" (STOC 2013)
- **Lo que se puede afirmar:** Bitansky, N., Canetti, R., Chiesa, A., y Tromer, E., «Recursive Composition and Bootstrapping for SNARKs and Proof-Carrying Data», 45th ACM STOC, 2013, pp. 111–120, DOI 10.1145/2488608.2488623; ePrint 2012/095.
- **Fuentes:** `WebFetch raw cryptobib bib/crypto_STOC13.bib, STOC:BCCT13: pages 111--120, doi 10.1145/2488608.2488623`; `cryptobib bib/crypto_EPRINT12.bib: ePrint 2012/095`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_STOC13.bib`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/abbrev3.bib`

### B16j

**CONFIRMADA** · gravedad baja

- **Borrador:** Omitido: Kothapalli & Setty (trabajos posteriores a Nova)
- **Lo que se puede afirmar:** Kothapalli, A., y Setty, S. T. V., «HyperNova: Recursive Arguments for Customizable Constraint Systems», en L. Reyzin y D. Stebila (eds.), CRYPTO 2024, Parte X, LNCS 14929, Springer, 2024, pp. 345–379, DOI 10.1007/978-3-031-68403-6_11; ePrint 2023/573.
- **Fuentes:** `WebFetch raw cryptobib bib/crypto_C24.bib, C:KotSet24: 'Abhiram Kothapalli and Srinath T. V. Setty', '{HyperNova}: Recursive Arguments for Customizable Constraint Systems', pages 3`; `arnaucube/math paper-notes.bib: ePrint 2023/573`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_C24.bib`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_EPRINT23.bib`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/abbrev3.bib`

### B1c

**CONFIRMADA** · gravedad baja

- **Borrador:** RFC 8391, mayo de 2018, DOI 10.17487/RFC8391
- **Lo que se puede afirmar:** RFC 8391, mayo de 2018, DOI 10.17487/RFC8391.
- **Fuentes:** `relaton RFC8391.yaml: "DOI: 10.17487/RFC8391", "Publication Date: May 2018"`; `https://raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC8391.yaml`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc8391.txt`

### B1d

**CONFIRMADA** · gravedad baja

- **Borrador:** RFC 8391: rama IRTF (CFRG), categoría Informational
- **Lo que se puede afirmar:** RFC 8391 es Informational, de la rama IRTF y producto del CFRG (procede de draft-irtf-cfrg-xmss-hash-based-signatures-12). No es un estándar del IETF.
- **Fuentes:** `relaton RFC8391.yaml: "Series: RFC 8391 (IRTF stream)"`; `WebSearch (resultado de rfc-editor.org/rfc/rfc8391): "published in May 2018 by the Internet Research Task Force (IRTF)"; "This document is not an Internet Standards Track specifica`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc8391.txt`; `https://raw.githubusercontent.com/liudonghua123/rfc/main/data/rfc8391.json`; `https://raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC8391.yaml`

### B2a

**CONFIRMADA** · gravedad baja

- **Borrador:** RFC 8554: "McGrew, D., Curcio, M., and S. Fluhrer", abril de 2019
- **Lo que se puede afirmar:** McGrew, D., Curcio, M., y S. Fluhrer, «Leighton-Micali Hash-Based Signatures», RFC 8554, DOI 10.17487/RFC8554, abril de 2019 (IRTF, CFRG, Informational).
- **Fuentes:** `relaton RFC8554.yaml: "D. McGrew, M. Curcio, S. Fluhrer", "April 2019", DOI 10.17487/RFC8554, rama IRTF`; `GitHub code search squinky86/SwATips tips/20220919.bib: author = {David McGrew and Michael Curcio and Scott Fluhrer}`; `WebSearch: RFC 8554 es Informational, IRTF, producto del CFRG, autores de Cisco Systems`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc8554.txt`; `https://raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC8554.yaml`; `https://raw.githubusercontent.com/liudonghua123/rfc/main/data/rfc8554.json`

### B3c

**CONFIRMADA** · gravedad baja

- **Borrador:** RFC 9162 de 2021
- **Lo que se puede afirmar:** RFC 9162, diciembre de 2021. Los algoritmos MTH/PATH/SUBPROOF que usa Arqueo son los del RFC 6962 (crates/zk-ssl-verify/src/mmr.rs:21-22), que es el que debe citarse como origen, indicando que el 9162 lo deja obsoleto.
- **Fuentes:** `relaton RFC9162.yaml: "Published: December 2021"`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc9162.txt`

### B4a

**CONFIRMADA** · gravedad baja

- **Borrador:** RFC 10033 (informational, September 2026): título, autores, fecha, categoría, rama y DOI
- **Lo que se puede afirmar:** Wiggers, T., Bashiri, K., Kölbl, S., Goodman, J., y S. Kousidis, «Hash-Based Signatures: State and Backup Management», RFC 10033, DOI 10.17487/RFC10033, septiembre de 2026 (rama IETF, grupo PQUIP, Informational, documento de consenso del IETF).
- **Fuentes:** `WebFetch raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC10033.yaml: title 'Hash-Based Signatures: State and Backup Management'; docid 10.17487/RFC10033; publis`; `WebFetch raw.githubusercontent.com/hbs-guidance/draft-hbs-state/main/draft-ietf-pquip-hbs-state (md) front matter: category: info; workgroup: Post-Quantum Use In Protocols; nombres`; `WebSearch: título de rfc-editor.org/info/rfc10033 'RFC 10033: Hash-Based Signatures: State and Backup Management'`; `hbs-state/spec/HBS-STATE-v0.3 (md):14-15`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc10033.txt`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc10033.xml`; `https://raw.githubusercontent.com/liudonghua123/rfc/main/data/rfc10033.json`; `https://raw.githubusercontent.com/kesara/watcher/main/rfc/rfc10033.txt`

### B4b

**CONFIRMADA** · gravedad baja

- **Borrador:** RFC 10033 procede de draft-ietf-pquip-hbs-state
- **Lo que se puede afirmar:** RFC 10033 procede de draft-ietf-pquip-hbs-state-04 (grupo PQUIP del IETF), sucesor del borrador individual draft-wiggers-hbs-state.
- **Fuentes:** `WebFetch github.com/hbs-guidance/draft-hbs-state: repositorio del borrador del grupo PQUIP, fichero draft-ietf-pquip-hbs-state (md), docname draft-ietf-pquip-hbs-state-latest`; `relaton RFC10033.yaml: grupo editorial pquip (sin relación derivedFrom explícita)`; `WebSearch: datatracker draft-ietf-pquip-hbs-state-03 y su antecesor individual draft-wiggers-hbs-state-00..02`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc10033.xml`; `https://raw.githubusercontent.com/liudonghua123/rfc/main/data/rfc10033.json`; `https://raw.githubusercontent.com/hbs-guidance/draft-hbs-state/main/draft-ietf-pquip-hbs-state (md)`; `github.com/ietf-artarea/ad-comments iesg-evaluation/draft-ietf-pquip-hbs-state-03/idnits-report.txt`

### B4d

**CONFIRMADA** · gravedad baja · **el escéptico revocó el primer veredicto** (PARCIAL)

- **Borrador:** RFC 10033 recomienda hardware dedicado; cita literal: "in particular, this enables implementing rollback resistant counters, which can be difficult to achieve in a software-only fashion"
- **Lo que se puede afirmar:** RFC 10033 §4: «Using dedicated cryptographic hardware is recommended to enforce these requirements, ensure correct behavior, and handle the complexity of state management. In particular, this enables implementing rollback resistant counters, which can be difficult to achieve in a software-only fashion.» (cita literal del RFC publicado, con coma).
- **Fuentes:** `raw draft-ietf-pquip-hbs-state (md) (sección 4): "Using dedicated cryptographic hardware is recommended to enforce these requirements, ensure correct behavior, and handle the compl`; `hbs-state/README.md:22-25; hbs-state/spec/HBS-STATE-v0.3 (md):16-18`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc10033.txt`; `https://raw.githubusercontent.com/kesara/watcher/main/rfc/rfc10033.txt`; `https://raw.githubusercontent.com/liudonghua123/rfc/main/data/rfc10033.txt`; `https://raw.githubusercontent.com/hbs-guidance/draft-hbs-state/main/draft-ietf-pquip-hbs-state (md)`

### B4e

**CONFIRMADA** · gravedad baja

- **Borrador:** La sección 5 de RFC 10033 da nueve estrategias de gestión de estado (y ninguna trae un vector ejecutable)
- **Lo que se puede afirmar:** El §5 de RFC 10033 analiza nueve «enfoques potenciales» de gestión de estado (5.1–5.9) sin recomendar ninguno en particular, y el documento no incluye vectores de prueba.
- **Fuentes:** `raw draft-ietf-pquip-hbs-state (md): sección 'Potential State Management Approaches' con 9 subsecciones: Multiple Public Keys (SP 800-208), Distributed Multi-trees (SP 800-208), Se`; `misma fuente: el documento no contiene «test vector», «fsync» ni «high-water»`; `hbs-state/README.md:25-29`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc10033.txt`

### B5a

**CONFIRMADA** · gravedad baja

- **Borrador:** NIST SP 800-208: Cooper, Apon, Dang, Davidson, Dworkin, Miller, octubre de 2020, DOI 10.6028/NIST.SP.800-208
- **Lo que se puede afirmar:** Cooper, D. A., Apon, D. C., Dang, Q. H., Davidson, M. S., Dworkin, M. J., y Miller, C. A., «Recommendation for Stateful Hash-Based Signature Schemes», NIST Special Publication 800-208, octubre de 2020, DOI 10.6028/NIST.SP.800-208.
- **Fuentes:** `WebSearch (nist.gov y bibcite RIS): autores David Cooper, Daniel Apon, Quynh Dang, Michael Davidson, Morris Dworkin, Carl Miller; RIS "Cooper, David A. AU - Apon, Daniel C. AU - Da`; `raw draft-ietf-pquip-hbs-state (md), referencia SP.800-208: target https://doi.org/10.6028/NIST.SP.800-208, date October 2020 (esa referencia trae la errata «David Apon»)`; `https://raw.githubusercontent.com/ietf-tools/bibxml-data-archive/main/bibxml-nist/reference.NIST.SP.800-208.xml`; `https://raw.githubusercontent.com/neurogaussthm/NIST-SP800-Data/HEAD/try2/SP800-208.txt`; `https://raw.githubusercontent.com/mnot/rfc-refs/main/rfcs/rfc10033.txt`

### B5b

**CONFIRMADA** · gravedad baja

- **Borrador:** URL de SP 800-208: https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-208.pdf
- **Lo que se puede afirmar:** https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-208.pdf es el PDF oficial; como identificador estable, https://doi.org/10.6028/NIST.SP.800-208.
- **Fuentes:** `hbs-state/README.md:16`; `WebSearch: nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-208.pdf aparece como 'NIST Special Publication 800-208 Recommendation for Stateful ...'`; `https://raw.githubusercontent.com/ietf-tools/bibxml-data-archive/main/bibxml-nist/reference.NIST.SP.800-208.xml`

### B6a

**CONFIRMADA** · gravedad baja

- **Borrador:** STARK: Ben-Sasson, Bentov, Horesh, Riabzev, IACR ePrint 2018/046
- **Lo que se puede afirmar:** Ben-Sasson, E., Bentov, I., Horesh, Y., y Riabzev, M., «Scalable, transparent, and post-quantum secure computational integrity», Cryptology ePrint Archive, Paper 2018/046, 2018.
- **Fuentes:** `WebSearch: eprint.iacr.org/2018/046, 'Scalable, transparent, and post-quantum secure computational integrity', Eli Ben-Sasson, Iddo Bentov, Yinon Horesh, Michael Riabzev (también d`; `GitHub code search acl2/acl2 books/kestrel/air/top.lisp: misma atribución y 'Cryptology ePrint Archive Paper 2018/046'`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_EPRINT18.bib`

### B6b

**CONFIRMADA** · gravedad baja

- **Borrador:** Versión CRYPTO 2019 del artículo STARK
- **Lo que se puede afirmar:** Ben-Sasson, E., Bentov, I., Horesh, Y., y Riabzev, M., «Scalable Zero Knowledge with No Trusted Setup», en A. Boldyreva y D. Micciancio (eds.), CRYPTO 2019, Parte III, LNCS 11694, Springer, 2019, pp. 701–732, DOI 10.1007/978-3-030-26954-8_23.
- **Fuentes:** `WebFetch raw cryptobib thomwiggers/cryptobib-export-split-demo bib/crypto_C19.bib, C:BBHR19: title 'Scalable Zero Knowledge with No Trusted Setup', pages 701--732, doi 10.1007/978-`; `GitHub code search BaDaaS/cryptography.academy (ePrint 2022/1626): 'In CRYPTO (3), volume 11694 of Lecture Notes in Computer Science, pages 701–732. Springer, 2019'`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/crypto_C19.bib`; `https://raw.githubusercontent.com/thomwiggers/cryptobib-export-split-demo/a03c9a5a89afab3ec53b3f22ae3cfdfff679fbad/bib/abbrev3.bib`

## Bloque P — el proceso IETF/IRTF, el Internet-Draft tal como está escrito, y la lógica formal de ECST

### P11a

**FALSA** · gravedad alta

- **Borrador:** «IVC … el coste de verificación O(log n) en el auditor permanece constante»
- **Lo que se puede afirmar:** O(log n) no es constante. Con IVC, el coste del verificador no depende del número n de transiciones: depende del parámetro de seguridad y del tamaño del circuito de paso, y con compresión Spartan puede ser logarítmico en ese tamaño. O(log n) en n transiciones es lo que cuesta una prueba de inclusión o de consistencia en un MMR (Arqueo tiene MMR de cabezas y zkssl_consistencyProof), y O(polylog T) es lo que cuesta verificar un STARK de traza T.
- **Fuentes:** `WebSearch: Valiant, TCC 2008, LNCS 4948: el verificador IVC usa tiempo y espacio «essentially constant»`; `WebSearch: Nova (Kothapalli, Setty, Tzialla, CRYPTO 2022; ePrint 2021/370)`; `FIVE_BACKENDS.md:59-96: PoC Nova con RecursiveSNARK::verify 108 ms y CompressedSNARK::verify 50 ms; el paso 9 cuesta 0,77 veces el paso 1; 10.764 restricciones por paso; prueba de `; `WebSearch: Ben-Sasson et al., ePrint 2018/046: la verificación STARK escala de forma polilogarítmica`; `FIVE_BACKENDS.md:59-96 (Nova: RecursiveSNARK::verify 108 ms, CompressedSNARK::verify 50 ms, 10.764 restricciones por paso)`; `crates/nova-experiment/src/folding_chain.rs:39-50 (E1 Bn256EngineKZG/HyperKZG, E2 Grumpkin/IPA, Spartan RelaxedR1CSSNARK)`; `crates/nova-experiment/Cargo.toml:32-53 (HyperKZG::setup deshabilitado en producción: exige ceremonia)`; `WebSearch: Valiant, TCC 2008, LNCS 4948; Kothapalli, Setty y Tzialla, CRYPTO 2022 (ePrint 2021/370)`

### P12a

**FALSA** · gravedad alta

- **Borrador:** ECST-R: «un salto de instrucción inducido por hardware no sirve de nada porque el verificador externo computará Verify(Π_i)=0»
- **Lo que se puede afirmar:** Un fallo inyectado que haga al nodo aceptar una transición inválida no se impide: solo se detecta después, y solo si Π_i se publica y un tercero la vuelve a verificar con las reglas vigentes y con solidez suficiente (en Arqueo el registro no guarda las pruebas y hasta §73 el nodo aceptó sin verificar). Lo que ninguna manipulación del probador consigue es una prueba válida de una transición inválida, porque la solidez no depende del hardware. En el lado de la firma, Arqueo verifica cada firma de cabeza antes de emitirla, lo que corta los fallos cuya salida no verifica. Pero un fallo de tipo «grafting» en una capa intermedia de XMSS^MT da una firma que sí verifica y expone una segunda firma WOTS+, con la que se puede falsificar (Genêt et al., ePrint 2018/674; TCHES 2023). En esquemas con estado lo evita cachear las firmas intermedias, y no se ha medido si la pila de Arqueo lo hace.
- **Fuentes:** `SECURITY.md:66-69: hasta el 31-07-2026 la capa no verificaba las pruebas de la vía de pago: aceptación sin verificación que ningún tercero detectó`; `crates/zk-ssl-verify/src/reverificacion.rs:44-47: las pruebas reales no se guardan`; `SECURITY.md:57-59: «hoy nadie fuera del operador observa cabezas»`; `SECURITY.md:176-200: el verificador es sustituible sin que se vea`; `WebSearch: Castelnovi, Martinelli y Prest, «Grafting Trees», PQCrypto 2018: un fallo al construir un subárbol no superior de XMSS^MT/HSS da una firma WOTS+ defectuosa y permite una`; `spec/NUCLEO.md:210: las cabezas de Arqueo se firman con XmssMtSha2_40_8_256`; `SECURITY.md:519-522: techo de 63 bits de solidez`; `SECURITY.md:66-69 (§73: aceptación sin verificación hasta el 31-07-2026)`

### P12c

**FALSA** · gravedad alta

- **Borrador:** «Ningún ataque microarquitectónico puede falsificar … sin conocer las claves»
- **Lo que se puede afirmar:** La frase es falsa en los dos sentidos. Los canales laterales son una forma de conocer las claves: uno en la máquina del pagador, donde viven Wallet::spend_key y el probador, rompe la autoría con pruebas válidas e indistinguibles. Y hay ataques microarquitectónicos de fallo que falsifican firmas hash sin extraer la clave (Rowhammer contra SLH-DSA, arXiv 2509.13048). La formulación defendible es: «sin extraer claves, sin romper la solidez y sin inducir fallos en el firmante, ningún ataque sobre el ejecutor produce una prueba aceptada de una transición inválida».
- **Fuentes:** `crates/zk-ssl-sdk/src/lib.rs:1-20 y 88-104: la clave de gasto vive en el cliente (Wallet) y la prueba se genera en local`; `SECURITY.md:56: «Autoridad de gasto: solo quien controla la clave puede gastar»`; `SECURITY.md:441-481: entre §521 y §538 las pruebas publicaban en claro la clave de gasto y las de custodio`; `WebSearch: Kannwischer, Genêt, Butin, Krämer y Buchmann, «Differential Power Analysis of XMSS and SPHINCS», COSADE 2018`; `crates/zk-ssl-sdk/src/lib.rs:1-20, :88-104 (Wallet: «Nunca sale de aquí»; prueba en local)`; `SECURITY.md:56`; `SECURITY.md:476-516 (en d531c80, 441-481)`; `WebSearch: Kannwischer, Genêt, Butin, Krämer y Buchmann, «Differential Power Analysis of XMSS and SPHINCS», COSADE 2018 (link.springer.com/chapter/10.1007/978-3-319-89641-0_10)`

### P13

**FALSA** · gravedad media

- **Borrador:** «El umbral eleva exponencialmente la complejidad del ataque» (luego retractado en los borradores)
- **Lo que se puede afirmar:** Un umbral k-de-n eleva de 1 a k el número de claves que hay que comprometer; no multiplica exponencialmente el trabajo criptográfico. Solo con compromisos independientes y de probabilidad p la probabilidad de éxito cae como p^k. En Arqueo (2-de-N), el probador necesita las dos claves a la vez, y entre el §523 y el §538 el umbral colapsó porque las pruebas revelaban las claves de custodio. La retractación de los borradores es correcta.
- **Fuentes:** `ARQUITECTURA.md:860-866: «Da: robar una clave ya no basta. Se necesitan dos»; «No da … quien genera la prueba necesita las dos claves a la vez»`; `ARQUITECTURA.md:836-839: entre §523 y §538, tras una emisión delegada el nodo tenía las claves de dos custodios`; `SECURITY.md:441-481`; `ARQUITECTURA.md:859-866 («Da: robar una clave ya no basta. Se necesitan dos»; «quien genera la prueba necesita las dos claves a la vez»)`; `ARQUITECTURA.md:836-840`; `SECURITY.md:476-516 (en d531c80, 441-481)`

### P14

**FALSA** · gravedad alta

- **Borrador:** Uso de «compromisos de Pedersen» en un diseño post-cuántico
- **Lo que se puede afirmar:** Arqueo no usa compromisos de Pedersen: sus compromisos son hash (Rescue-Prime sobre Goldilocks, con sal para ocultar). Pedersen es vinculante solo bajo el logaritmo discreto, que el algoritmo de Shor rompe (permite abrir el mismo compromiso a otro valor), aunque sigue siendo perfectamente ocultante. No es coherente con un diseño que se presenta como post-cuántico; lo coherente son compromisos hash, vinculantes por resistencia a colisiones, o reticulares.
- **Fuentes:** `grep -i pedersen en Arqueo (.md y .rs, sin target): 0 resultados`; `spec/NUCLEO.md:249-262: hojas y compromisos por merge Rescue-Prime; SECURITY.md:315: el compromiso de hoja es ocultante (hash con sal)`; `crates/nova-experiment/Cargo.toml:11-12 y src/folding_chain.rs:39-47: el único componente sobre curvas, el PoC Nova, usa BN254/Grumpkin con HyperKZG/IPA`; `WebSearch: Pedersen es perfectamente ocultante y computacionalmente vinculante bajo logaritmo discreto, y Shor rompe la vinculación`; `grep -ril pedersen en los .md y .rs de Arqueo (sin doc/ecst ni target): 0`; `spec/NUCLEO.md:237-266 (en d531c80, 233-262: merge Rescue-Prime y composiciones)`; `SECURITY.md:320 (en d531c80, :315): §3.2, compromiso de hoja ocultante`; `crates/nova-experiment/Cargo.toml:11-12`

### P15a

**FALSA** · gravedad alta

- **Borrador:** «KeyAtZero ⇒ NoSign»
- **Lo que se puede afirmar:** En el modelo derivado de semilla, todo reinicio real da KeyAtZero, así que si KeyAtZero implicara NoSign el nodo no volvería a firmar nunca. La regla correcta, la de HBS-STATE v0.3 (el juez y N3) y la de Arqueo desde S335/S337, es que KeyAtZero no es fatal: se falla cerrado dando por consumidos los índices 0..contador-1, se resincroniza la clave al índice `contador` (que nunca se reservó) y se sigue. Solo se detiene si un segundo registro prueba que el contador retrocedió: el diario del nodo con `<` o las cofirmas del testigo con `<=`. El único estado que prohíbe firmar es KeyAhead.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):103-107: en el modelo derivado de semilla, tras un reinicio «always KeyAtZero»; :141-146: KeyAtZero no es fatal; :179-185: el juez solo marca Key`; `hbs-state/src/lib.rs:370-377: is_fatal devuelve false para KeyAtZero`; `crates/zk-ssl-node/src/main.rs:576-609: ClaveEnCero -> ArrancaResincronizando hasta el contador («SE ABANDONAN … de 0 a contador-1 … PERDIDOS y NO REUTILIZABLES»); NoArranca solo s`; `crates/zk-ssl-cli/src/witness.rs:2731-2757: el testigo hace lo mismo, con '<=' contra las cofirmas`; `hbs-state/spec/HBS-STATE-v0.3 (md):103-107 («always KeyAtZero»), :144, :163-176, :179-185 (juez), :202 (N3 «and only there»)`; `hbs-state/src/lib.rs:370-377 (is_fatal: KeyAtZero false), :663-680 (counter_one_key_zero_fails_closed)`; `crates/zk-ssl-guardian/src/lib.rs:369-375 (no_admite_matiz: ClaveEnCero false), :649-666 (con_el_contador_en_uno_y_la_clave_en_cero_falla_cerrada), :337-352 (comentario mal atado)`; `crates/zk-ssl-node/src/main.rs:608-641 (ClaveEnCero -> ArrancaResincronizando; NoArranca solo si 'Some(d) if *contador < d'), :994-1000 (resincronizar_a)`

### P16b

**FALSA** · gravedad alta

- **Borrador:** Tabla: «Recursión ZK intra-circuito: IVC Sí (Requerida)»
- **Lo que se puede afirmar:** Arqueo no tiene recursión dentro del circuito: cada transición lleva su propio STARK y la historia se ata con una cadena hash fuera del circuito. La IVC es opcional y ortogonal a ECST. El único experimento con IVC (Nova) es una prueba de concepto sobre BN254/Grumpkin, con HyperKZG que exige ceremonia, y no es post-cuántico. La celda contradice además la tesis de los propios borradores de que «ECST no es IVC».
- **Fuentes:** `crates/zk-ssl/src/log.rs:250-294: el encadenado se hace fuera del circuito y cada transición lleva su propia prueba`; `FIVE_BACKENDS.md:59-96: Nova es solo una prueba de concepto, que «no implementa el circuito de cumplimiento ni la partida doble»`; `crates/nova-experiment/Cargo.toml:11-12 y 31-50: curvas BN254/Grumpkin; HyperKZG exige ceremonia`; `Los propios borradores afirman que «ECST no es IVC» (ver P17)`; `crates/zk-ssl/src/log.rs:250-294`; `FIVE_BACKENDS.md:93-96 (Nova: «no implementa el circuito de cumplimiento ni la partida doble»)`; `crates/nova-experiment/Cargo.toml:11-12, :32-53`

### P18c

**FALSA** · gravedad alta

- **Borrador:** «ECST/HBS-STATE establece un nuevo estándar»
- **Lo que se puede afirmar:** No es un estándar. Es una especificación en borrador derivada de una única implementación, que la propia especificación llama «programa» hasta que se mida un segundo sujeto independiente. Tampoco consta que exista un Internet-Draft enviado.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):422-427: «v0.3 -- DRAFT … What is still missing is the only thing that turns this into a standard: a second INDEPENDENT subject … Until then it d`; `hbs-state/spec/HBS-STATE-v0.3 (md):468-473: pq-xmss no cuenta como segundo implementador`; `P8d: el I-D ni siquiera consta como enviado`; `hbs-state/spec/HBS-STATE-v0.3 (md):422-427 («Until then it describes a programme»)`; `hbs-state/spec/HBS-STATE-v0.3 (md):468-473`; `P8d`

### P2

**FALSA** · gravedad alta

- **Borrador:** «Vía Grupo de Trabajo: el CFRG adopta el borrador, que pasa a llamarse draft-ietf-cfrg-hbs-state»
- **Lo que se puede afirmar:** Si el CFRG lo adoptara, se llamaría draft-irtf-cfrg-hbs-state-00 (flujo IRTF). El prefijo draft-ietf- es para documentos adoptados por un WG del IETF (por ejemplo draft-ietf-pquip-…). Antes de la adopción, lo habitual es draft-toranzo-cfrg-hbs-state (guía del CFRG; naming-your-internet-draft (md)).
- **Fuentes:** `CFRG-Process (md): «The draft name usually changes from draft-yourname-cfrg-topic to draft-irtf-cfrg-topic»`; `raw.githubusercontent.com/ietf/authors.ietf.org/main/naming-your-internet-draft (md):55-66: «"irtf" for the IRTF stream … any I-D submitted with one of these stream/organization st`; `WebSearch: RFC 8391 (XMSS) se tramitó como draft-irtf-cfrg-xmss-hash-based-signatures (datatracker.ietf.org/doc/rfc8391/writeup/)`; `raw.githubusercontent.com/ietf/wiki.ietf.org/main/group/cfrg/CFRG-Process (md) (re-leído 2026-09-30; publicado 2026-09-23): «The draft name usually changes from draft-yourname-cfrg`; `raw.githubusercontent.com/ietf/authors.ietf.org/main/naming-your-internet-draft (md): «"irtf" for the IRTF stream … will either be rejected or replaced»; «the version number of thi`

### P3

**FALSA** · gravedad alta

- **Borrador:** «El IESG aprueba formalmente el documento» (para un documento del CFRG)
- **Lo que se puede afirmar:** En el flujo IRTF el documento pasa la RGLC del CFRG, el writeup del shepherd y la revisión del IRTF Chair. Después lo aprueba el IRSG (RFC 5743), y el IESG solo hace la revisión de conflictos (RFC 5742). No es el IESG quien lo aprueba.
- **Fuentes:** `CFRG-Process (md) pasos 5-7: «IRTF Chair Review … IRSG Poll: IRSG members vote on the publication … IESG Conflict Review: The IESG checks for conflicts with ongoing IETF work»`; `WebSearch RFC 5743: «the IRSG reviews the document and approves it for publication … the IESG reviews the document to assure that there are no conflicts»`; `WebSearch RFC 5742 (BCP 92): el IESG hace conflict review de los flujos IRTF e Independent, con cinco conclusiones posibles`; `WebSearch: RFC 7209 es «Requirements for Ethernet VPN (EVPN)», sin relación`; `raw.githubusercontent.com/ietf/wiki.ietf.org/main/group/cfrg/CFRG-Process (md) (re-leído 2026-09-30; publicado 2026-09-23): pasos «IRTF Chair Review», «IRSG Poll», «IESG Conflict R`; `WebSearch RFC 5743 (rfc-editor.org/info/rfc5743, datatracker.ietf.org/doc/html/rfc5743): el IRSG aprueba; el IESG comprueba que no haya conflictos`; `WebSearch RFC 5742 (datatracker.ietf.org/doc/html/rfc5742): «IESG Procedures for Handling of Independent and IRTF Stream Submissions»`

### P5b

**FALSA** · gravedad media

- **Borrador:** LAMPS es un destino natural para HBS-STATE
- **Lo que se puede afirmar:** LAMPS trata formatos PKIX, CMS y S/MIME (por ejemplo RFC 9802 y RFC 9708). HBS-STATE no define identificadores, ASN.1 ni formatos de certificado, así que no es su destino natural.
- **Fuentes:** `WebSearch: los productos de LAMPS sobre firmas con estado son RFC 9802 (identificadores X.509 para HSS/XMSS) y RFC 9708 (HSS/LMS en CMS): ámbito PKIX/CMS`; `hbs-state/spec/HBS-STATE-v0.3 (md):3-10: «not an XMSS implementation. It is not a parameter profile»; no contiene ASN.1, X.509 ni CMS`; `WebSearch: RFC 9802 (rfc-editor.org/info/rfc9802; draft-ietf-lamps-x509-shbs) y RFC 9708 (datatracker.ietf.org/doc/html/rfc9708), ambos del WG LAMPS`; `hbs-state/spec/HBS-STATE-v0.3 (md):8-10: «It is not an XMSS implementation. It is not a parameter profile»`

### P5d

**FALSA** · gravedad media

- **Borrador:** Recomendación de los borradores: presentarlo en «CFRG y LAMPS» (sin mencionar PQUIP)
- **Lo que se puede afirmar:** El destino natural es el WG PQUIP (lista pqc@ietf.org). Ese grupo produjo RFC 10033, cuyo hueco (ningún vector ejecutable) HBS-STATE quiere cubrir, y su carta abarca la guía operacional sin mecanismos criptográficos nuevos. Si PQUIP no lo acepta, el cauce para decidir dónde va es SECDISPATCH. El CFRG puede revisarlo, y LAMPS no encaja.
- **Fuentes:** `WebSearch, carta de PQUIP (datatracker.ietf.org/doc/charter-ietf-pquip/): «Documenting design guidance and best practices … will not … define new cryptographic mechanisms»; publica`; `Copia del editor de RFC 10033: venue PQUIP, pqc@ietf.org`; `WebSearch (search_code en ietf/wiki.ietf.org): existe el grupo secdispatch`; `WebSearch, carta de PQUIP (datatracker.ietf.org/doc/charter-ietf-pquip/): «will not update existing protocols, specify new protocols, define new cryptographic mechanisms … will doc`; `raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC10033.yaml (2026-09, IETF)`; `scratchpad/rfc10033.xml: docName draft-ietf-pquip-hbs-state-04`

### P7e

**FALSA** · gravedad alta

- **Borrador:** Implícito: el apéndice de vectores (bloque JSON abierto con tres acentos graves y nunca cerrado) se procesa como código
- **Lo que se puede afirmar:** Los vectores tienen que ir en `~~~ json` … `~~~`, que genera <sourcecode type="json">. Con acentos graves, cerrados o no, kramdown-rfc no produce un bloque de código y no avisa, y eso corrompe justo el entregable principal.
- **Fuentes:** `kramdown REL_2_4_0 lib/kramdown/parser/kramdown/codeblock.rb:33-34: FENCED_CODEBLOCK_START = /^~{3,}/ (solo tildes); kramdown-rfc usa input 'RFC2629Kramdown' < Kramdown (lib/kramdo`; `Prueba en el scratchpad (kramdown-rfc 1.7.43 y xml2rfc 3.34.1, documento mínimo): '~~~ json' genera <sourcecode type="json">; con acentos graves cerrados sale <spanx style="verb"> `; `No hay copia del I-D pegado en disco (grep sin resultados); el defecto se toma de la descripción del orquestador`; `scratchpad/gems/gems/kramdown-2.4.0/lib/kramdown/parser/kramdown/codeblock.rb:33: FENCED_CODEBLOCK_START = /^~{3,}/`; `scratchpad/skp/t (md) -> t.xml (prueba de este escéptico): <sourcecode type="json"> para ~~~; <spanx style="verb">json … para ''' cerrado; <t>'''json … para ''' sin cerrar; rc=0 y `

### P7f

**FALSA** · gravedad media

- **Borrador:** Implícito: el diagrama ASCII de decisión, colapsado en una sola línea, se reproduce
- **Lo que se puede afirmar:** El diagrama tiene que ir línea a línea en un bloque `~~~ ascii-art`, con un máximo de 72 columnas. Colapsado en una línea con barras verticales, kramdown lo interpreta como tabla y xml2rfc aborta.
- **Fuentes:** `Prueba en el scratchpad: una línea «+-----+ --> +-----+ \| A \| --> \| B \| …» se interpreta como tabla (texttable sin filas) y xml2rfc 3.34.1 aborta con «Expecting an element , go`; `scratchpad/skp/d (md) -> d.xml: <texttable> con cinco <ttcol> y sin filas; xml2rfc: «Error: Expecting an element , got nothing, at /rfc/middle/section[1]/table … Unable to complete`

### P7g

**FALSA** · gravedad alta

- **Borrador:** Referencias escritas como líneas «[*RFC2119]» en vez de bloques YAML
- **Lo que se puede afirmar:** Las referencias se declaran en la cabecera YAML (normative:/informative:) o en línea con {{!RFC2119}} y {{?RFC10033}}. Para las palabras clave se usa `{::boilerplate bcp14-tagged}`, que cita RFC 2119 y RFC 8174. «[*RFC2119]» no es sintaxis de kramdown-rfc.
- **Fuentes:** `Prueba en el scratchpad: avisos «No link definition for link ID '*rfc2119' found», salida «[<em>RFC2119] [</em>RFC10033]» y ninguna sección de referencias generada`; `kramdown-rfc README:173-186: {{!RFC2119}} para normativas, {{?RFC1925}} para informativas, o bloques normative:/informative: en el YAML`; `lib/kramdown-rfc/command.rb:99-114: el boilerplate bcp14 produce «BCP 14 {{!RFC2119}} {{!RFC8174}}»`; `scratchpad/skp/d (md) -> d.xml: «<t>Keywords per [<em>RFC2119] and [</em>RFC10033].</t>»`; `scratchpad/gems/gems/kramdown-rfc2629-1.7.43/lib/kramdown-rfc/command.rb:108-114: el boilerplate bcp14 genera «BCP 14 {{!RFC2119}} {{!RFC8174}}»`; `kramdown-rfc README:173-186`

### P7h

**FALSA** · gravedad media

- **Borrador:** Correo del autor «autor@ejemplo.com»
- **Lo que se puede afirmar:** El I-D debe llevar el correo real del autor, porque es el contacto que se conserva en el RFC y el que recibe la verificación del envío. Un marcador como autor@ejemplo.com no sirve, y ejemplo.com no es un dominio reservado (RFC 2606).
- **Fuentes:** `WebSearch RFC 2606: solo están reservados example.com/.net/.org y los TLD .test, .example, .invalid y .localhost; ejemplo.com no`; `authors.ietf.org submitting-your-internet-draft (md):26: «the authors will receive email with a request to verify the submission»`; `required-content (md):140: el I-D debe incluir el contacto de los autores`; `submitting-your-internet-draft (md): «the authors will receive email with a request to verify the submission … If the submitter is logged into the Datatracker and listed as an auth`; `required-content (md):132-140 (copia del scratchpad): Authors' Addresses`; `WebSearch RFC 2606`

### P18b

**SIN_FUENTE** · gravedad media

- **Borrador:** Autoevaluaciones del tipo «8.5/10»
- **Lo que se puede afirmar:** Hay que quitar las notas numéricas, que no tienen método ni fuente. En su lugar se da el estado declarado: HBS-STATE v0.3 es un borrador derivado de una sola implementación, y Arqueo no tiene auditoría externa (SECURITY.md:8).
- **Fuentes:** `grep en ambos repositorios: no aparece ninguna puntuación de ese tipo`; `hbs-state/spec/HBS-STATE-v0.3 (md):422-427: DRAFT derivado de una sola implementación`; `SECURITY.md:3-17: sin auditoría externa; «Si algo aquí suena más seguro de lo que puedes verificar … trátalo como afirmación pendiente»`; `CONTRIBUTING.md:10-14: «Medir antes que afirmar … Distinguir lo medido de lo estimado de lo supuesto»`; `grep -rIn "8[.,]5/10" en los .md de Arqueo (sin doc/ecst): 0`; `hbs-state/spec/HBS-STATE-v0.3 (md):422-427`; `CONTRIBUTING.md:10-16`

### P18d

**SIN_FUENTE** · gravedad media

- **Borrador:** «Cambio de paradigma», «revolucionario»
- **Lo que se puede afirmar:** Hay que quitar los calificativos promocionales. Lo que se puede afirmar es la combinación concreta, con su evidencia: pruebas por transición, resúmenes encadenados con cabezas firmadas XMSS^MT, testigos y un verificador sin conexión, todo con sus límites declarados. Los componentes están en la literatura y deben citarse.
- **Fuentes:** `grep en ambos repositorios: «paradigm» solo aparece en el sentido de «paradigmas de prueba» (PAPER_EN.md:23-138), nunca como autoevaluación`; `P9c: la propiedad central (encadenado por hash) viene de Haber y Stornetta (1991)`; `grep «paradigm» en Arqueo: solo en el sentido de «paradigmas de prueba» (PAPER_EN.md:23-138; QUESTIONS.md:66, :239); 0 apariciones de «revolucionari»`

### P1

**PARCIAL** · gravedad media

- **Borrador:** «CFRG (Crypto Forum Research Group), el brazo de investigación criptográfica del IETF»
- **Lo que se puede afirmar:** El CFRG es un grupo de investigación (Research Group) del IRTF, no un grupo de trabajo del IETF. Sus RFC salen por el flujo IRTF (Informational o Experimental) y no son estándares del IETF, aunque el CFRG asesora a los WG del IETF en criptografía (guía de proceso del CFRG en wiki.ietf.org; RFC 5743).
- **Fuentes:** `raw.githubusercontent.com/ietf/wiki.ietf.org/main/group/cfrg/CFRG-Process (md) (editado 2026-09-23): «CFRG documents are not IETF standards»; recorrido RGLC -> IRTF Chair -> IRSG P`; `WebSearch: datatracker.ietf.org/doc/charter-irtf-cfrg/ y irtf.org/cfrg.html: es un Research Group del IRTF, lista cfrg@irtf.org`; `raw.githubusercontent.com/ietf/wiki.ietf.org/main/group/cfrg/CFRG-Process (md) (re-leído 2026-09-30; publicado 2026-09-23): «CFRG documents are not IETF standards»`; `WebSearch RFC 5743 (rfc-editor.org/info/rfc5743): «the IRSG reviewing the document and approving it for publication … The IESG then reviews the document to assure that there are no`

### P10b

**PARCIAL** · gravedad alta · cambió con el árbol (S566–S582)

- **Borrador:** «CryptoValid + HistoryOK dan base para reconstruir la secuencia»
- **Lo que se puede afirmar:** Solo es cierto con hipótesis explícitas. (a) Todas las Π_i y sus entradas públicas tienen que estar disponibles por una vía que el operador no controle: el registro de Arqueo guarda solo resúmenes y el operador no sirve histórico. (b) La cabeza tiene que estar autenticada frente a un ancla previa (hoy es TOFU). (c) Hay que conocer las reglas (AIR y verificador) vigentes en cada transición. (d) La historia tiene que estar completa. Desde el RFC-0010 (aceptado en §577), una operación recibida por applySend o applyClaim y no resuelta en la ventana deja un recibo firmado y oponible, pero la omisión sin recibo y lo que entra por applyMany o pledge no dejan rastro. Lo reconstruible es la secuencia de raíces más lo que cada prueba publica como entrada pública (en un envío, importe, límite y suministro). Los saldos, las claves y las sales no se reconstruyen: el testigo oculto no los publica desde el §538.
- **Fuentes:** `crates/zk-ssl-verify/src/reverificacion.rs:44-47: en Send, Claim, Burn, Refund y Migration «el registro no guarda la prueba: no seran recomputables desde el registro nunca»`; `spec/RPC.md:659-700 (§248): el operador NO sirve el histórico; los testigos guardan cabezas firmadas`; `crates/zk-ssl/src/log.rs:52-56: «El operador podría no publicar el registro», «No impide la censura»`; `SECURITY.md:176-200: el operador puede cambiar el verificador y hoy eso es invisible`; `SECURITY.md:122-165: TOFU`; `crates/zk-ssl-verify/src/reverificacion.rs:44-47`; `spec/RPC.md:714-755 (en d531c80, 659-700; §248: el operador NO sirve el histórico)`; `crates/zk-ssl/src/log.rs:50-56`

### P11b

**PARCIAL** · gravedad media

- **Borrador:** «Tiempo de validación sublineal respecto al tamaño de la transición»
- **Lo que se puede afirmar:** Por transición es cierto: un STARK se verifica en tiempo polilogarítmico en la traza. En Arqueo, verify_audit tarda 1,6 ms frente a 274 ms de generación (ARQUITECTURA, anterior al §538), y aplicar, que verifica, muta el árbol y escribe, costó 4-6 ms frente a 345-1198 ms de generación en una única ejecución del 2026-09-28. Pero un auditor que verifica N transiciones sin IVC paga N verificaciones y N pasos de cadena, un coste lineal en N. Ese es el compromiso que ECST debe declarar. Sin IVC: verificación lineal en N, fallo localizado por transición y sin maquinaria recursiva. Con IVC: verificación independiente de N, a cambio del sobrecoste del verificador dentro de cada paso (el PoC Nova usa 10.764 restricciones para un paso de un solo hash) y, en la pila medida, de supuestos de curva y de una ceremonia para HyperKZG.
- **Fuentes:** `ARQUITECTURA.md:620-632: verificar cuesta ~1,5-4 ms frente a 105-620 ms de generar, por prueba (medido)`; `WebSearch: ePrint 2018/046, verificación STARK polilogarítmica`; `crates/zk-ssl/src/log.rs:438-470: verify recorre las N entradas, coste O(N)`; `ARQUITECTURA.md:620-647 (tabla y nota: «Esa razón es la de la AUDITORÍA», verify_audit 1,6 ms frente a 274, apply el 28,5 %)`; `doc/ecst/borrador/simulate-2026-09-28.log (64,3-77,6 KB; 345-1198 ms de generación; apply 4-6 ms)`; `crates/zk-ssl/src/metrics.rs:82-83 (banda de un pago: 145.953-167.967 B)`; `crates/zk-ssl/src/log.rs:438-470 (verify recorre las N entradas)`; `FIVE_BACKENDS.md:86-96`

### P12b

**PARCIAL** · gravedad alta

- **Borrador:** «ECST traslada la carga de la prueba desde el hardware a la verificación matemática»
- **Lo que se puede afirmar:** ECST traslada la detección de transiciones inválidas: deja de depender de confiar en el hardware que ejecuta y pasa a depender de verificar la evidencia, siempre que la evidencia esté disponible y haya verificadores independientes. No traslada la custodia de los secretos (claves de gasto y clave XMSS del operador) ni la del índice XMSS, que siguen dependiendo del hardware y del sistema. Por eso SP 800-208 y RFC 10033 recomiendan hardware dedicado.
- **Fuentes:** `hbs-state/README.md:94-100: «Nothing against a power cut»`; `hbs-state/spec/HBS-STATE-v0.3 (md):12-21: SP 800-208 exige módulos hardware; RFC 10033: «rollback resistant counters … difficult … in a software-only fashion»`; `Copia del editor de RFC 10033 (líneas 504-511): «how an accidental/intentional failure/glitch might affect the state security»`; `SECURITY.md:8: «Nada de este proyecto ha sido auditado por terceros»`; `hbs-state/README.md:94-100 («Nothing against a power cut»)`; `hbs-state/spec/HBS-STATE-v0.3 (md):12-21`; `scratchpad/rfc10033.txt:458-459 («how an accidental/intentional failure/glitch might affect the state security»), :485-486`; `SECURITY.md:8`

### P15b

**PARCIAL** · gravedad media

- **Borrador:** «S ∈ U ⇒ NoSign» (U = conjunto de estados inseguros)
- **Lo que se puede afirmar:** La regla es correcta si U = {KeyAhead}, que es el juez de HBS-STATE. Arqueo añade, como política propia y solo en la rama KeyAtZero, parar si el contador está por debajo de lo que prueba un segundo registro: el diario del nodo con `<` o las cofirmas del testigo con `<=`. Meter KeyAtZero o CounterAhead en U viola N1 y N3. Hay dos límites: el gate se desactiva borrando el diario (declarado), y un contador borrado o puesto a cero cae en Coincide{0}, donde ninguna de las dos políticas consulta el segundo registro. Esto último está deducido por lectura, sin medir, y no está declarado.
- **Fuentes:** `hbs-state/src/lib.rs:412-432: reconcile_values; hbs-state/spec/HBS-STATE-v0.3 (md):204-206: fusionar CounterAhead con KeyAhead «does not meet N1»`; `crates/zk-ssl-node/src/main.rs:586-590: «El gate se apaga quitando un fichero» (límite declarado)`; `main.rs:558 y witness.rs:2338: comentarios antiguos que aún dicen que ClaveEnCero «SI para» / «NO arranca», contradichos por el código de S335/S337`; `hbs-state/src/lib.rs:412 (reconcile_values)`; `hbs-state/spec/HBS-STATE-v0.3 (md):204-206`; `crates/zk-ssl-guardian/src/lib.rs:426-437 (fichero ausente -> 0)`; `crates/zk-ssl-node/src/main.rs:597-601 (Coincide -> Arranca sin mirar tope_diario), :617-622 (límite declarado), :785 (test con None)`; `crates/zk-ssl-cli/src/witness.rs:2701-2703 (Coincide -> Arranca sin mirar tope_cofirmas)`

### P16a

**PARCIAL** · gravedad alta

- **Borrador:** Tabla: «Custodia Cero en API: Garantizada por tipos»
- **Lo que se puede afirmar:** Los tipos impiden serializar la clave (Wallet no implementa Serialize), pero no lo «garantizan». Entre el §521 y el §538 la clave de gasto y las de custodio salían en claro dentro de las pruebas enviadas por la API. Desde el §538 el probador, un fork no auditado de winterfell 0.13.1, oculta el testigo, y la propiedad se cumple en lo que mide la suite. Formulación correcta: «la clave no se serializa por diseño de tipos; que no se filtre a través de las pruebas está medido, no demostrado».
- **Fuentes:** `SECURITY.md:441-446: «Wallet::spend_key es privado y ni siquiera implementa Serialize»`; `SECURITY.md:446-481: con winterfell 0.13 las pruebas «no ocultaban su testigo» y publicaban la clave de gasto y las de custodio; desde §538 «se cumple … en lo que la suite mide», s`; `spec/RPC.md:53-61`; `SECURITY.md:476-481 (Wallet::spend_key privado y sin Serialize), :481-516 (winterfell 0.13 no ocultaba el testigo; desde §538 «en lo que la suite mide»)`

### P16c

**PARCIAL** · gravedad media

- **Borrador:** Tabla: «Verificación offline: un ledger convencional requiere sincronización»
- **Lo que se puede afirmar:** Los ledgers convencionales tienen clientes ligeros (SPV): basta con las cabeceras y una rama de Merkle. El paquete de Arqueo se verifica sin conexión con una cabeza firmada, pero depende de un ancla de clave (hoy TOFU) y de testigos independientes para detectar vistas divididas. La comparación honesta es «sin conexión frente a un ancla» contra «en línea frente a la cadena de cabeceras».
- **Fuentes:** `spec/PAQUETE.md:20-34: verificación sin nodo, sin capa y sin probador (firma de cabeza, camino de acuse y cofirmas)`; `SECURITY.md:122-165: el ancla es TOFU`; `spec/RPC.md:659-700: una vista dividida solo se detecta comparando diarios de testigos independientes`; `WebSearch, whitepaper de Bitcoin §8 (SPV): basta con las cabeceras y la rama de Merkle, no la cadena entera`; `spec/PAQUETE.md:20-31`; `SECURITY.md:122-165`; `spec/RPC.md:714-755 (en d531c80, 659-700)`; `WebSearch: whitepaper de Bitcoin §8 (SPV)`

### P16d

**PARCIAL** · gravedad media

- **Borrador:** Tabla: «Semántica de estado físico: explícita»
- **Lo que se puede afirmar:** Es defendible para HBS-STATE, con matices. Los cuatro estados, el juez y el modelo de SK son explícitos y tienen vectores. Pero la persistencia solo se comprueba con un heurístico de coste de fsync (umbrales declarados y medidos en una sola máquina), no hay garantía ante un corte de corriente, y un contador borrado o restaurado no se detecta: vuelve a 0. La comparación con un ledger convencional no está medida.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):93-133 (eje del modelo de SK), :141-146 (cuatro estados), :380-388 (no cubre cómo se persiste el contador, varios firmantes, ni LMS/HSS probados)`; `hbs-state/src/lib.rs:547: el autocontrol rechaza solo si ratio<10 Y con fsync <20 µs; :57-61 y :97-108: umbrales declarados, de una sola máquina`; `hbs-state/README.md:94-100: nada contra un corte de corriente`; `hbs-state/src/lib.rs:455-465: si falta el fichero del contador, el contador vale 0 sin aviso`; `hbs-state/spec/HBS-STATE-v0.3 (md):93-133, :141-146, :380-388`; `hbs-state/src/lib.rs:97-108, :548 (ratio < MIN_RATIO && with < FLOOR_MICROS)`; `hbs-state/README.md:94-100`; `hbs-state/src/lib.rs:455-465`

### P18a

**PARCIAL** · gravedad media

- **Borrador:** Una versión dice en §3 que C<K es «colapso» y en §11.7 que K está en RAM y desaparece
- **Lo que se puede afirmar:** Las dos frases son ciertas, pero en modelos distintos, y los borradores no declaran cuál usan (el nivel N0 de HBS-STATE). C<K (KeyAhead) es el único estado fatal. En el modelo derivado de semilla, K no sobrevive al reinicio (vuelve a 0), así que KeyAhead solo se observa dentro del proceso vivo. En el modelo con SK persistida sí puede observarse tras reiniciar. El informe debe declarar el modelo antes de clasificar.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):103-108 (tabla de modelos), :141-146, :179-185`; `hbs-state/src/lib.rs:323-356: CounterAhead se midió dentro de un proceso; tras un reinicio el caso es KeyAtZero`; `hbs-state/spec/HBS-STATE-v0.3 (md):103-108, :141-146, :179-185`; `hbs-state/src/lib.rs:326-336`

### P5c

**PARCIAL** · gravedad media

- **Borrador:** El CFRG es el destino natural de HBS-STATE
- **Lo que se puede afirmar:** El CFRG alojó XMSS (RFC 8391) y LMS (RFC 8554) y puede revisar el trabajo. Pero, según su propia guía, la orientación de ingeniería a implementadores con criptografía conocida encaja mejor en un WG del IETF. HBS-STATE (cuatro estados, un juez y vectores) es orientación operacional, no un resultado criptográfico nuevo.
- **Fuentes:** `CFRG-Process (md): «If your problem predominantly concerns implementation details, performance optimization, or configuration choices using well-understood cryptography, it might b`; `WebSearch: el CFRG produjo RFC 8391 (XMSS) y RFC 8554 (LMS)`; `hbs-state/spec/HBS-STATE-v0.3 (md):422-427: «DRAFT … Derived from one single implementation … Until then it describes a programme»`; `raw.githubusercontent.com/ietf/wiki.ietf.org/main/group/cfrg/CFRG-Process (md) (re-leído 2026-09-30; publicado 2026-09-23): «might be more appropriately handled within an IETF work`; `hbs-state/spec/HBS-STATE-v0.3 (md):422-427`

### P6b

**PARCIAL** · gravedad media

- **Borrador:** «Todo el proceso es gratuito»
- **Lo que se puede afirmar:** Enviar I-Ds, participar en las listas de correo y publicar un RFC no cuesta nada, y asistir a las reuniones no es obligatorio. Las reuniones plenarias tienen cuota, presencial y remota, pero RFC 9501 (BCP 239) obliga a ofrecer una opción remota gratuita.
- **Fuentes:** `WebSearch RFC 9501 (BCP 239): «there must be a free option for online participation»`; `WebSearch ietf-announce, IETF 126: las reuniones tienen cuota de inscripción presencial y remota`; `WebSearch RFC 9501 (datatracker.ietf.org/doc/rfc9501/): «there must be a free option for online participation to IETF meetings»`; `WebSearch ietf-announce IETF 126 / IETF 127: cuotas de inscripción presencial y remota`

### P6c

**PARCIAL** · gravedad media

- **Borrador:** «Asistir en persona cuesta 700-1000 (USD)»
- **Lo que se puede afirmar:** Asistir en persona cuesta más de lo que dicen los borradores. En IETF 127 (San Francisco, 14-20 de noviembre de 2026) el pase semanal era de 950 USD solo con la tarifa super early, cerrada el 28-09-2026. Hoy rige la tarifa early, más cara, y desde el 3-11 la estándar (en IETF 126 fueron 1.180 y unos 1.300 USD). Todo ello sin viaje ni alojamiento. Las cifras vienen de resultados de búsqueda de ietf-announce, no de la página de inscripción.
- **Fuentes:** `WebSearch, ietf-announce «IETF 126 registration reminder» (Viena, 18-24 jul 2026): Onsite Week Pass 950 USD super early (1140 con IVA), 1180 USD early, estándar anunciada unos 1300`; `WebSearch «IETF 127 registration fees» (resultados: www.mail-archive.com/ietf-announce@ietf.org/msg26982.html y msg27102.html; registration.ietf.org/127/): super early hasta el 28-`; `WebSearch IETF 126 (mailarchive.ietf.org/arch/msg/ietf-announce/ZRxdSW_xJwhcRR9bsIv6m90oWgc/): early 1.180 USD, estándar 1.300 USD`

### P6d

**PARCIAL** · gravedad media

- **Borrador:** «La participación remota es gratuita (o de coste simbólico)»
- **Lo que se puede afirmar:** La inscripción remota tiene cuota (en IETF 127, 270 USD con tarifa super early; en IETF 126, 335 USD con tarifa early y 390 con tarifa estándar), así que no es un coste simbólico. RFC 9501 (BCP 239) obliga a ofrecer una opción gratuita, que el IETF da mediante exenciones.
- **Fuentes:** `WebSearch IETF 126: Remote Week Pass 270 USD super early; estándar anunciada 390 USD`; `WebSearch RFC 9501 / ietf.org fee waiver: «unlimited number of fee waivers available to remote participants and does not ask participants to explain»`; `WebSearch IETF 127: «Remote Week Pass: $270» (super early), «Remote Full-time Student Week Pass: $60»`; `WebSearch IETF 126: remoto early 335 USD, estándar 390 USD`; `WebSearch RFC 9501 (BCP 239)`

### P7a

**PARCIAL** · gravedad baja

- **Borrador:** «gem install kramdown-rfc2629»
- **Lo que se puede afirmar:** El nombre vigente es kramdown-rfc (`gem install kramdown-rfc`). La gema kramdown-rfc2629 sigue publicada en la misma versión (1.7.43) por compatibilidad.
- **Fuentes:** `rubygems.org API: kramdown-rfc 1.7.43 y kramdown-rfc2629 1.7.43, ambos del 2026-08-27`; `Instalación en el scratchpad: «Successfully installed kramdown-rfc2629-1.7.43 / kramdown-rfc-1.7.43» (kramdown-rfc arrastra el antiguo)`; `github.com/cabo/kramdown-rfc README:27 «gem install kramdown-rfc»; :62-64 «what was called kramdown-rfc2629 is now called kramdown-rfc»`; `rubygems.org/api/v1/gems/kramdown-rfc.json y kramdown-rfc2629.json: 1.7.43, 2026-08-27`; `raw.githubusercontent.com/cabo/kramdown-rfc/master/README.md:27, :62-64`; `scratchpad/gems/gems/kramdown-rfc2629-1.7.43/lib/kramdown-rfc/command.rb:102-114`

### P7d

**PARCIAL** · gravedad media

- **Borrador:** «workgroup: Crypto Forum Research Group» en un -00 individual llamado draft-toranzo-hbs-state
- **Lo que se puede afirmar:** Es práctica habitual poner el CFRG en el workgroup de un -00 individual que busca ese grupo, con `submissiontype: IRTF` y `workgroup: "Crypto Forum"`. Por convención, el nombre debería llevar el grupo: draft-toranzo-cfrg-hbs-state-00, o draft-toranzo-pquip-hbs-state si el destino es PQUIP. Nunca debe empezar por draft-ietf- ni por draft-irtf- antes de la adopción.
- **Fuentes:** `Borradores individuales dirigidos al CFRG: draft-connolly-cfrg-xwing-kem (md) y draft-chen-cfrg-vdaf-pine (md) usan «submissiontype: IRTF» y «workgroup: "Crypto Forum"» (raw.github`; `authors.ietf.org naming-your-internet-draft (md):72-75: convención draft-authors-wgname-subject para buscar adopción`; `CFRG-Process (md): «draft-yourname-cfrg-topic»`; `raw.githubusercontent.com/dconnolly/draft-connolly-cfrg-xwing-kem/main/draft-connolly-cfrg-xwing-kem (md):6-13`; `naming-your-internet-draft (md): «an individual submission seeking adoption by a working group will include the working group's acronym as the second component»`

### P7i

**PARCIAL** · gravedad media

- **Borrador:** RFC 10033 figura como referencia informativa aunque el texto se apoya en ella
- **Lo que se puede afirmar:** Si el I-D necesita definiciones o requisitos de RFC 10033 (por ejemplo, los requisitos ACID de su §4) para entenderse o implementarse, la referencia es normativa. Si solo motiva el hueco, basta con informativa. En un documento Informational la downref no es obstáculo.
- **Fuentes:** `WebSearch, IESG Statement «Normative and Informative References»: «Normative references specify documents that must be read to understand or implement the technology»`; `hbs-state/spec/HBS-STATE-v0.3 (md):12-21: usa RFC 10033 §4/§5 como motivación (el hueco), no para definir los estados`; `WebSearch RFC 3967: las reglas de downref se refieren a documentos Standards Track`; `WebSearch, declaración de la IESG «Normative and Informative References»`; `hbs-state/spec/HBS-STATE-v0.3 (md):12-21`; `WebSearch RFC 3967 (downrefs en Standards Track)`

### P7j

**PARCIAL** · gravedad media

- **Borrador:** «normative: NIST-SP800-208» es correcto
- **Lo que se puede afirmar:** Es razonable que SP 800-208 sea normativa, pero la entrada tiene que resolverse: con un bloque YAML completo (título, autores, fecha y DOI 10.6028/NIST.SP.800-208), con un ancla BibXML válida o con DOI.10.6028/NIST.SP.800-208. Escrita como «NIST-SP800-208» a secas, xml2rfc aborta.
- **Fuentes:** `Prueba en el scratchpad: la clave YAML «NIST-SP800-208:» sin campos da «*** don't know how to expand ref NIST-SP800-208 nil», y xml2rfc: «IDREF attribute target references an unkno`; `Prueba: «NIST.SP.800-208» se mapea a la entidad bibxml2 https://bib.ietf.org/public/rfc/bibxml2/reference.NIST.SP.800-208.xml (no se comprobó que exista; bib.ietf.org bloqueado)`; `Copia del editor de RFC 10033: declara «SP.800-208» como normativa con título, autores, fecha y target https://doi.org/10.6028/NIST.SP.800-208`; `scratchpad/skp/n (md) -> n.err / salida de xml2rfc (prueba de este escéptico)`; `scratchpad/rfc10033.txt:101, :119 (SP.800-208 entre las referencias normativas)`

### P7k

**PARCIAL** · gravedad baja

- **Borrador:** El I-D remite a «la ecuación de la Sección 3.1»
- **Lo que se puede afirmar:** Las referencias cruzadas deben ser anclas ({{sec-reconciliation}}) o, si son externas, escribirse «{{Section N of RFCxxxx}}». RFCXML no numera ecuaciones: la fórmula R(C,K) debe ir en un bloque con ancla. Ni HBS-STATE v0.3 ni RFC 10033 tienen una §3.1 con ecuación, y la del propio I-D no se puede comprobar sin su texto.
- **Fuentes:** `hbs-state/spec/HBS-STATE-v0.3 (md):179-185: §3 «The judge» no tiene §3.1 ni ecuaciones numeradas`; `Copia del editor de RFC 10033: §3.1 es «Assessing Operational Costs», sin ecuación`; `kramdown-rfc lib/kramdown-rfc2629.rb:110-116 (XREF_SINGLE/SECTIONS_RE): admite {{Section N of RFCxxxx}} y anclas internas`; `hbs-state/spec/HBS-STATE-v0.3 (md):179-192`; `scratchpad/rfc10033.txt:311-313`; `kramdown-rfc lib/kramdown-rfc2629.rb (sintaxis {{Section N of RFCxxxx}})`

### P8b

**PARCIAL** · gravedad media

- **Borrador:** El anuncio cita «v0.2.0»
- **Lo que se puede afirmar:** El anuncio debe decir «hbs-state 0.2.0 (crate), que implementa la especificación HBS-STATE v0.3 revisada el 2026-09-27». La versión del crate y la de la especificación son independientes, y la release 0.1.0 lleva una v0.3 anterior con dos afirmaciones que luego se corrigieron.
- **Fuentes:** `hbs-state: tag v0.2.0 en a960828 (2026-09-27); Cargo.toml:45; CITATION.cff:10-11`; `hbs-state/.zenodo.json: «crate version 0.2.0 … implements specification HBS-STATE v0.3 … The two version numbers are independent»`; `hbs-state/spec/HBS-STATE-v0.3 (md):37-41: la release v0.1.0 (2026-09-26) lleva la v0.3 anterior a las dos correcciones`; `hbs-state/Cargo.toml:3`; `hbs-state/CITATION.cff:10-11`; `hbs-state/.zenodo.json:4 y la descripción: «The two version numbers are independent»`; `hbs-state/spec/HBS-STATE-v0.3 (md):37-41`

### P8c

**PARCIAL** · gravedad media

- **Borrador:** Licencia «CC0 / Dual MIT/Apache-2.0»
- **Lo que se puede afirmar:** El código está bajo MIT OR Apache-2.0, a elección del receptor. Los vectores de spec/ están bajo CC0-1.0, y el texto de la especificación bajo MIT OR Apache-2.0, no CC0. La etiqueta «mit» de Zenodo no es la licencia. Lo que se copie dentro de un I-D queda además bajo BCP 78, con los componentes de código bajo Revised BSD (TLP del IETF Trust §4).
- **Fuentes:** `hbs-state/README.md:182: «MIT OR Apache-2.0. The vectors under spec/, CC0-1.0.»`; `hbs-state/spec/HBS-STATE-v0.3 (md):479-482: «The vectors under spec/, CC0-1.0. Everything else, MIT OR Apache-2.0.»`; `hbs-state/.zenodo.json:6: license «mit» (una etiqueta, según su propia descripción); CITATION.cff:4`; `WebSearch, TLP del IETF Trust §4: los componentes de código de documentos IETF quedan bajo la Revised BSD License`; `hbs-state/spec/HBS-STATE-v0.3 (md):481-482`; `hbs-state/.zenodo.json:5 (license «mit») y la descripción («a label, not the terms»)`; `hbs-state/Cargo.toml:5`; `hbs-state/CITATION.cff:4, :12-14`

### P9a

**PARCIAL** · gravedad alta

- **Borrador:** Teorema 1 (atado retroactivo bajo resistencia a colisiones): cambiar Π_k por Π'_k hace H'_n ≠ H_n salvo con probabilidad despreciable
- **Lo que se puede afirmar:** Enunciado correcto: si un adversario de tiempo polinómico produce dos historias distintas, con la misma génesis y la misma longitud n, que dan el mismo H_n, se construye a partir de él un adversario que encuentra una colisión de la función de encadenamiento o de la función de resumen de pruebas. En Arqueo, el test T1 comprueba en un caso concreto (12 entradas, alteración en la 5, pruebas sintéticas) que la alteración se propaga a la cabeza. Es una comprobación del encadenado, no una medida de la resistencia a colisiones (AUDITORIA §116).
- **Fuentes:** `crates/zk-ssl/src/log.rs:250-262: chain_digest = merge(merge(cuerpo, proof_digest), previous)`; `log.rs:1131-1225, módulo t1_chain_retroactivo (N=12, K=5; «alterar la epoca K falsifica la cabeza de N»); AUDITORIA.md §115.1: 3/3 en verde`; `AUDITORIA.md §116: «T1 mide el encadenamiento, no la resistencia a colision del resumen»`; `crates/zk-ssl/src/log.rs:250-262 (chain_digest), :281-294 (chain_digest_v2), :1189-1295 (t1_chain_retroactivo; N=12 y K=5 en :1198-1199)`; `AUDITORIA.md §115.1 («3/3 en verde»), §116 («T1 mide el encadenamiento, no la resistencia a colision del resumen»)`

### P9b

**PARCIAL** · gravedad alta · **el escéptico revocó el primer veredicto** (FALSA)

- **Borrador:** Basta la resistencia a colisiones de H para el atado retroactivo
- **Lo que se puede afirmar:** La resistencia a colisiones basta para el atado retroactivo siempre que se cumplan tres hipótesis del propio teorema: (1) la codificación de cada entrada es inyectiva (campos de anchura fija o con longitud y dominio); (2) tanto el encadenado como el resumen de pruebas son resistentes a colisiones (en Arqueo, merge Rescue-Prime Rp64_256 y Blake3-256 reducido a Goldilocks); (3) la entrada ata los bytes reales de Π_i. Arqueo violó (1) en §116 y §124 y (3) en seis vías hasta §278, sin romper ningún hash. Para que un tercero saque provecho del teorema hacen falta además una cabeza H_n autenticada frente a un ancla previa (hoy es TOFU), la génesis y n conocidos, y las preimágenes disponibles. Esas condiciones son del despliegue, no del teorema.
- **Fuentes:** `AUDITORIA.md §116 (relleno sin longitud: dos pruebas que difieren en ceros finales colisionan) y §124.1 (la reducción % p hacía colisionar un bloque de valor p con ceros): colision`; `crates/zk-ssl-hash/src/lib.rs:1287-1320: digest_of_proof = Blake3(dominio ‖ longitud ‖ prueba) reducido a 4 elementos Goldilocks`; `AUDITORIA.md §271 y log.rs:161-231: seis vías asentaban digest_of_proof(&[]) = 74de079f…cfa1 (una constante) hasta §278`; `log.rs:385-392: verify_chain «No detecta que falte el principio»`; `SECURITY.md:122-165: ancla TOFU y «NO HAY CLAVE QUE ANCLAR»`; `crates/zk-ssl-hash/src/lib.rs:210,224: «native_merge no separa hoja de nodo»`; `crates/zk-ssl-hash/src/lib.rs:1290 (dominio b"ZK-SSL-proof-digest-v2"), :1303-1320 (longitud u64 LE por delante; Blake3-256 en cuatro elementos Goldilocks)`; `crates/zk-ssl/src/log.rs:250-262 (merges de digests de anchura fija; as_digest(seq) y as_digest(tag))`

### P9c

**PARCIAL** · gravedad media

- **Borrador:** El atado retroactivo del Teorema 1 como aportación propia de ECST
- **Lo que se puede afirmar:** El atado retroactivo es la propiedad clásica del enlazado por hash (Haber y Stornetta, 1991), base de los registros a prueba de manipulación (Crosby y Wallach, 2009) y de Certificate Transparency (RFC 6962 y RFC 9162). ECST puede usarlo citando esas fuentes, pero no es una aportación nueva.
- **Fuentes:** `WebSearch: Haber y Stornetta, «How to time-stamp a digital document», J. Cryptology 3:99-111, 1991 (link.springer.com/article/10.1007/BF00196791): enlazado de sellos por hash`; `WebSearch: Crosby y Wallach, «Efficient Data Structures for Tamper-Evident Logging», USENIX Security 2009`; `crates/zk-ssl/src/log.rs:44-46: Arqueo se compara con Certificate Transparency`; `WebSearch: Haber y Stornetta, «How to time-stamp a digital document», J. Cryptology 3(2):99-111, 1991, DOI 10.1007/BF00196791`; `WebSearch: Crosby y Wallach, USENIX Security 2009`; `crates/zk-ssl/src/log.rs:44-46`

### P8d

**NO_VERIFICABLE** · gravedad alta

- **Borrador:** El correo de anuncio al CFRG da una URL de datatracker del draft como si ya existiera
- **Lo que se puede afirmar:** No hay constancia de que draft-toranzo-hbs-state se haya enviado: no aparece en búsquedas ni en el espejo de metadatos de I-D del IETF. El anuncio no debe dar una URL de datatracker hasta que el envío exista. Lo correcto es subir primero el -00 y anunciarlo después.
- **Fuentes:** `datatracker.ietf.org y www.ietf.org bloqueados (EGRESS_BLOCKED / 403)`; `WebSearch «draft-toranzo-hbs-state» OR «draft-toranzo»: sin resultados`; `grep en ambos repos: 0 menciones de draft-toranzo, datatracker o cfrg`; `curl raw.githubusercontent.com/ietf-tools/relaton-data-ids/main/data/draft-toranzo-hbs-state-00.yaml -> 404 (igual -01 y draft-toranzo-cfrg-hbs-state-00); draft-ietf-pquip-hbs-stat`; `WebSearch «draft-toranzo-hbs-state»: sin resultados propios`; `www.ietf.org/archive/id y datatracker: EGRESS_BLOCKED`

### P10a

**CONFIRMADA** · gravedad media

- **Borrador:** «HistoryOK no implica CryptoValid»
- **Lo que se puede afirmar:** Correcto. La coherencia de la cadena (secuencia, continuidad de raíces y recomposición de H_i) no dice nada sobre la validez de las pruebas cuyos resúmenes encadena (TransitionLog::verify, log.rs:425-470).
- **Fuentes:** `crates/zk-ssl/src/log.rs:436-437: «No verifica las pruebas —no las guarda— pero sí que cada entrada esté atada a una prueba concreta»`; `spec/PAQUETE.md:680-681: el paquete demuestra «que la entrada está acusada bajo esa cabeza firmada, no qué dice»`; `spec/PAQUETE.md:729-730 (en d531c80, 680-681): «que la entrada está acusada bajo esa cabeza firmada, no qué dice»`

### P17

**CONFIRMADA** · gravedad media · cambió con el árbol (S566–S582)

- **Borrador:** «ECST no es IVC: no compone la validez de Π_{i-1} dentro de Π_i; compromete su representación mediante el digest» (y la duda de si Arqueo compromete los resúmenes de prueba)
- **Lo que se puede afirmar:** Es correcto, y Arqueo sí compromete los resúmenes de prueba. H(Π_i) corresponde a proof_digest = Blake3-256("ZK-SSL-proof-digest-v2" ‖ longitud_le64 ‖ Π_i), llevado a cuatro elementos Goldilocks. En las vías delegadas es el resumen del sello de autorización, y en OpenAccount el de la ausencia declarada (desde §278). H_i corresponde a chain_digest_v2(seq, tipo, raíz_vieja, raíz_nueva, proof_digest, H_{i-1}, compromiso), con merges Rescue-Prime y génesis cero. H_n corresponde a TransitionLog::head(), que entra como chain_digest en epoch_digest y, desde §570, en la cabeza v6, firmada con XMSS^MT una vez por época (1/min más a demanda) y no por transición. El mismo H(Π_i) aparece además en la hoja de acuse (acuse_digest) y, desde RFC-0010, en la hoja del recibo de recepción (recibo_digest) de applySend y applyClaim, aplicadas o rechazadas. Ninguna prueba verifica a la anterior dentro del circuito, y el registro no guarda las Π_i.
- **Fuentes:** `crates/zk-ssl/src/log.rs:20-27 (resumen = H(numero, tipo, raiz_antigua, raiz_nueva, H(prueba), resumen_anterior)), :250-262, :281-294, :358-378 (append: proof_digest = digest_of_pr`; `log.rs:330-336: head(); crates/zk-ssl/src/lib.rs:867: 'chain_digest: self.log.head()' en la cabeza de época`; `crates/zk-ssl-hash/src/lib.rs:256-266: epoch_digest incluye chain_digest; :696-699: acuse_digest(hash_prueba, epoca, n)`; `spec/NUCLEO.md:244-262: preámbulo firmado b"ZK-SSL-epoch-head" ‖ version ‖ epoch_digest y composiciones v1-v6`; `AUDITORIA.md §115: firma 1/min más a demanda; §278 y log.rs:161-231: sellos de las vías delegadas`; `crates/zk-ssl-verify/src/reverificacion.rs:44-47: el registro no guarda las pruebas`; `crates/zk-ssl/src/log.rs:20-27, :250-262, :281-294, :358-378`; `crates/zk-ssl/src/lib.rs:875 ('chain_digest: self.log.head()')`

### P4

**CONFIRMADA** · gravedad media · **el escéptico revocó el primer veredicto** (PARCIAL)

- **Borrador:** «Vía envío individual: un Area Director de Seguridad puede patrocinarlo»
- **Lo que se puede afirmar:** Es correcto. Un documento de autor individual puede publicarse como «individual submission» en el flujo IETF si lo patrocina un Area Director; un AD de Seguridad es el candidato natural. El patrocinio es discrecional, está pensado para documentos pequeños y no conflictivos y exige IETF Last Call (RFC 4846; página de patrocinio de la IESG). Hay otras tres vías: la adopción por un WG del IETF (por ejemplo PQUIP), la adopción por el CFRG en el flujo IRTF (aprobación del IRSG) y el flujo de Envíos Independientes del ISE (RFC 4846), estas dos últimas con conflict review del IESG (RFC 5742).
- **Fuentes:** `raw.githubusercontent.com/ietf/wiki.ietf.org/main/group/iesg/adsponsoring.html: «AD sponsored documents skip the Working Group phase … Set the Stream to "IETF" … do an IETF Last Ca`; `WebSearch RFC 4846: flujo de Envíos Independientes a cargo del ISE designado por el IAB, distinto del patrocinio de AD en el flujo IETF`; `WebSearch RFC 5742: el conflict review del IESG también se aplica al flujo Independent`; `WebSearch RFC 4846 (rfc-editor.org/rfc/rfc4846): «Documents produced by individuals … but submitted for publication via the IESG under Area Director sponsorship, are known as "indi`; `raw.githubusercontent.com/ietf/wiki.ietf.org/main/group/iesg/adsponsoring.html: «Set the Stream to "IETF"», «the AD will do an IETF Last Call», «reasonably small, and does not cont`

### P5a

**CONFIRMADA** · gravedad baja

- **Borrador:** RFC 10033 salió del WG PQUIP (lo que los borradores no mencionan)
- **Lo que se puede afirmar:** RFC 10033 («Hash-Based Signatures: State and Backup Management», de Wiggers, Bashiri, Kölbl, Goodman y Kousidis) es el antiguo draft-ietf-pquip-hbs-state-04 del WG PQUIP del IETF, en flujo IETF, Informational y con consenso del IETF. Se publicó en septiembre de 2026 (relaton-data-rfcs). Su §4 enumera los requisitos ACID y su §5 contiene nueve enfoques (5.1-5.9), ninguno con vectores ejecutables (copia del RFC en el scratchpad, coherente con la copia del editor).
- **Fuentes:** `raw.githubusercontent.com/hbs-guidance/draft-hbs-state/main/draft-ietf-pquip-hbs-state (md) (commit 4ee80a2, 2026-02-27), cabecera: docname draft-ietf-pquip-hbs-state, workgroup «P`; `WebSearch: rfc-editor.org/info/rfc10033 «Hash-Based Signatures: State and Backup Management»; datatracker draft-ietf-pquip-hbs-state`; `hbs-state/spec/HBS-STATE-v0.3 (md):14-19 cita RFC 10033 como Informational, septiembre de 2026`; `raw.githubusercontent.com/ietf-tools/relaton-data-rfcs/main/data/RFC10033.yaml: «value: 2026-09», stream «IETF», DOI 10.17487/RFC10033`; `scratchpad/rfc10033.xml, cabecera: docName="draft-ietf-pquip-hbs-state-04" submissionType="IETF" number="10033" prepTime="2026-09-15T18:17:37" (origen de la copia no registrado)`; `scratchpad/rfc10033.txt:311-313 (§3, §3.1 Assessing Operational Costs), :420 (§4), :504 (§5), :516-832 (5.1-5.9), :485-486 (rollback resistant counters); grep -c vector = 0`; `raw.githubusercontent.com/hbs-guidance/draft-hbs-state/main/draft-ietf-pquip-hbs-state (md):1-25 (workgroup «Post-Quantum Use In Protocols», pqc@ietf.org)`; `hbs-state/spec/HBS-STATE-v0.3 (md):12-21`

### P6a

**CONFIRMADA** · gravedad baja

- **Borrador:** «Un Internet-Draft tiene una validez de 6 meses»
- **Lo que se puede afirmar:** Un I-D caduca 185 días (unos seis meses) después de subirse al repositorio, salvo que se suba una revisión o que esté en un estado que lo impida (en tramitación por la IESG o por el ISE). Las versiones caducadas siguen en el archivo (submitting-your-internet-draft (md)).
- **Fuentes:** `raw.githubusercontent.com/ietf/authors.ietf.org/main/submitting-your-internet-draft (md):54: «An I-D expires 185 days after it was placed in the Repository, unless it is in a state`; `required-content (md):50: boilerplate «valid for a maximum of six months»`; `WebSearch: draft-thomson-gendispatch-no-expiry propone sustituir la caducidad por active/inactive; no se encontró evidencia de que se haya aprobado`; `raw.githubusercontent.com/ietf/authors.ietf.org/main/submitting-your-internet-draft (md): «An I-D expires 185 days after it was placed in the Repository, unless it is in a state th`; `required-content (md):50 (copia del scratchpad): «valid for a maximum of six months»`

### P6e

**CONFIRMADA** · gravedad baja

- **Borrador:** «Las decisiones se toman en las listas de correo»
- **Lo que se puede afirmar:** Es correcto. El consenso aproximado se comprueba en la lista de correo: RFC 2418 §3.2 exige revisar en la lista las decisiones de las reuniones, y en el CFRG la llamada a adopción y la RGLC se hacen en la lista.
- **Fuentes:** `WebSearch RFC 2418 §3.2: «decisions at meetings … are not final and must be reviewed on the mailing list»`; `CFRG-Process (md): la llamada a adopción y la RGLC se hacen en la lista de correo`; `WebSearch RFC 2418 §3.2 y la declaración de la IESG «Guidance on Interim Meetings, Conference Calls and Jabber Sessions» (datatracker.ietf.org/doc/statement-iesg-guidance-on-interi`; `raw.githubusercontent.com/ietf/wiki.ietf.org/main/group/cfrg/CFRG-Process (md) (re-leído 2026-09-30; publicado 2026-09-23): adopción y RGLC «an announcement is sent to the mailing `

### P7b

**CONFIRMADA** · gravedad baja

- **Borrador:** Comandos «kramdown-rfc2629 file (md) > file.xml; xml2rfc --text --html file.xml»
- **Lo que se puede afirmar:** Los comandos funcionan. La forma documentada hoy es `kramdown-rfc draft (md) > draft.xml && xml2rfc --text --html draft.xml`, o `kdrfc draft (md)`, que encadena los dos pasos.
- **Fuentes:** `Prueba en el scratchpad: 'kramdown-rfc2629' produce la misma salida que 'kramdown-rfc' (cmp idéntico), y xml2rfc 3.34.1 (PyPI) con '--text --html' escribe «Created file legacy.txt `; `kramdown-rfc README:42-56 documenta 'kramdown-rfc mydraft (md) >mydraft.xml', 'xml2rfc mydraft.xml' y 'kdrfc'`; `scratchpad/gems/bin: kramdown-rfc, kramdown-rfc2629, kdrfc`; `scratchpad/xv/bin/xml2rfc --version: 3.34.1`; `kramdown-rfc README:50-56 (kdrfc)`

### P7c

**CONFIRMADA** · gravedad baja

- **Borrador:** «ipr: trust200902»
- **Lo que se puede afirmar:** `ipr: trust200902` es correcto. El texto del I-D queda bajo BCP 78/79 (IETF Trust).
- **Fuentes:** `authors.ietf.org required-content (md):62-66: la mayoría de los I-D usan «trust200902», que genera el aviso de TLP §6.b`; `kramdown-rfc examples/stupid-s.mkd: «ipr: trust200902»`; `authors.ietf.org required-content (md):62-66 (copia del scratchpad)`; `raw.githubusercontent.com/dconnolly/draft-connolly-cfrg-xwing-kem/main/draft-connolly-cfrg-xwing-kem (md) (cabecera kramdown-rfc de un borrador del CFRG)`

### P8a

**CONFIRMADA** · gravedad baja

- **Borrador:** El anuncio enlaza https://github.com/atoranzo/hbs-state
- **Lo que se puede afirmar:** El repositorio existe: github.com/atoranzo/hbs-state, commit a960828 «Release 0.2.0», con tag v0.2.0 del 2026-09-27.
- **Fuentes:** `hbs-state: git remote origin https://github.com/atoranzo/hbs-state; Cargo.toml:53`; `WebSearch: el repositorio aparece indexado en GitHub`; `hbs-state: git remote -v; git tag --points-at HEAD = v0.2.0`; `hbs-state/Cargo.toml:11: repository = "https://github.com/atoranzo/hbs-state"`; `hbs-state/CITATION.cff:15`

