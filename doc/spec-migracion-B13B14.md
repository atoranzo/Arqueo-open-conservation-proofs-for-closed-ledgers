# B13/B14 · Paso 1 — Spec del EVENTO DE MIGRACIÓN (cuatro sub-frentes, réplica determinista)

Entregable del paso 1 de la campaña. Censo completo ejecutado (regla
§137: TODAS las estructuras, no la muestra). Correcciones al enunciado:
**DIEZ AIR, no ocho** (FROZEN_DEPTH vive en burn, claim, freeze,
frozen_climb, send + settlement-helpers; unión con los 8 de hoja = 10);
y un **cuarto sub-frente**: el keyspace `acct:{índice}` de sled se
reescribe. El mapa IBAN→índice NO se persiste (memoria): responsabilidad
del operador re-registrar, fuera del estado.

## Decisiones de diseño, con veredicto y precedente

**D1 — El salt de hoja SE GUARDA EN EL RECORD.** La capa recomputa hojas
en cada apply (25 call-sites de `native_leaf` en 11 ficheros); sin el
salt no puede, y la clave no la tiene (§93.4). Guardarlo es coherente
con el modelo de amenaza: la 50 protege de TERCEROS (diccionario sobre
hermanos), no del operador, que ya ve saldos. El cliente lo deriva de su
clave (determinista) para probar; la capa lo lee del record para aplicar.
→ `AccountRecord` gana `leaf_salt: Digest`. Playbook 49-A P2 re-corrido:
campo + 13 constructores (operar preserva; abrir puebla; cargar del
formato) + `record_to_bytes_v3` (112 B) + **snapshot v6** (MAGIC_V6;
v5/v4/v3 importables, salt-cero centinela). Regresión análoga a
`operar_preserva_el_view_id`.

**D2 — La migración se registra SIN prueba ZK, CON réplica determinista.**
Precedente exacto: `open_account`, «la única transición sin prueba»,
digest cero visible en el log. Un circuito que pruebe N re-hasheos sería
una obra en sí; la honestidad del sistema ya admite transiciones
registradas-no-probadas cuando son deterministas y auditables. La entrada
compromete: (root_cuentas_vieja, nueva, root_frozen_vieja, nueva,
n_cuentas, versión_del_procedimiento). **Verificación = réplica**: quien
tenga los records pre-migración recomputa ambas raíces nuevas y compara.

**D3 — `OpKind::Migration` al FINAL del enum** (append-safe para el log
persistido; censo 4).

**D4 — Despliegue ATÓMICO con fixture legacy para el test.** Los AIR
nuevos verifican el mundo nuevo (hoja envuelta, frozen-32) y no el viejo:
circuitos + capa + migración van en UNA release. Las capas frescas de
test nacen post-mundo (salt real desde apertura, frozen-32, posición
derivada). El test de la migración construye el mundo viejo vía un
constructor legacy SOLO-test (secuencial, sin salt, frozen-24) — el
patrón de las instantáneas v3 artesanales que ya existe.

## El procedimiento, determinista y numerado (la réplica ES la spec)

Entrada: records pre-migración {índice_viejo → (public_id, balance,
nonce, view_id)}, marcas frozen pre {índice_viejo}, cuota next_index.
1. Ordenar los records por **índice viejo ascendente** (el orden fija el
   sondeo: réplica exacta).
2. Para cada record, en ese orden: `pos = public_id[0] mod 2^32`; sondeo
   lineal `pos = (pos+1) mod 2^32` mientras ocupado. Registrar
   `índice_viejo → pos`.
3. Hoja nueva en `pos`: `native_leaf_salted(id, balance, nonce,
   r.leaf_salt)` — **el salt DEL RECORD** [ENMIENDA 1b]: cero para lo
   cargado de formatos legacy, REAL para lo abierto post-1a. El texto
   original decía salt-cero-para-todo porque se escribió antes de 1a;
   1a hizo del record la fuente de verdad. Fijado en test
   (`cada_cuenta_sobrevive_con_su_hoja_envuelta`, commit dcc2af0).
4. Frozen nuevo a **profundidad 32**: para cada marca vieja en
   `índice_viejo`, marca FROZ en `mapa[índice_viejo]`.
5. Sled: por cada cuenta, borrar `acct:{viejo}`, insertar `acct:{nuevo}`
   con `record_to_bytes_v3`. (`pend:` y `meta:` intactos.)
6. Log: `OpKind::Migration`, digest de prueba cero, compromisos de D2.
7. `next_index` se conserva como censo de altas (cuota); deja de ser
   posición para siempre.
Post: altas nuevas = posición derivada + sondeo + salt real derivado en
apertura (la estrecha lo deriva del sk; `open_with_id` lo recibe como
PARÁMETRO, igual que view_id — mismo muro §90/§93.4, misma solución).

## Secuencia de implementación (cada paso compila y pasa antes del siguiente)

1a. **Formatos** (playbook 49-A): `leaf_salt` en record + 13
    constructores + `_v3`/112 B + snapshot v6 + `derive_leaf_salt_wide`
    ya existe (§117). Tests: roundtrip, dual-carga v5→centinela, operar
    preserva.
1b. **El evento**: `migrate_to_salted_positions()` según el procedimiento
    + fixture legacy + test de réplica (dos ejecuciones = mismas raíces)
    + test de que frozen sobrevive el remapa.
1.5 **Spec de la máquina de hoja** (la 7 adelantada, §134-real): las
    restricciones C_LEAF_* de los dos carriles ESCRITAS antes de tocarlas
    — el §72 demostró que una spec caza el bug de carril leyéndola.
2.  **Piloto `circuit_send`**: hoja envuelta (un merge más, ROW_SALT_*)
    + frozen-32 (su camino crece 8 niveles) + TEST DE MUTACIÓN (sabotear
    salt → el AIR rechaza, o la restricción es decorativa).
3.  Replicar a los NUEVE restantes, uno a uno, cada uno con su mutación.
4.  T2b-circuito (§126): la aceptación extremo a extremo.
5.  Medir con el instrumento de §130 (apareado si compara) y corregir la
    etiqueta de coste de la entrada 50.

## Lo que esta spec deja honestamente abierto
El coste de trace del frozen-32 en cada circuito (¿+64 filas caben en
1024? se verifica POR CIRCUITO en el paso 2-3, no se asume); y si la
release atómica exige un flag de arranque «pre/post» o basta el orden de
despliegue en nodo único (decisión del operador, documentar en runbook).

## Enmiendas de implementación (paso 1b, commits dcc2af0/f1c08f2)
E1: paso 3 del procedimiento — el salt del RECORD, no cero forzado (ver
arriba). E2: el sub-frente sled ganó el marcador `meta:migrated` (sin
sellar: su semántica es presencia y las raíces almacenadas arbitran) y
la CARGA geometría-consciente (hoja envuelta + frozen-32 con marcador;
legacy sin él). E3: el guardián de idempotencia tiene dos patas (log en
memoria O marcador en disco): sin la segunda, reabrir y re-migrar
re-envolvería las hojas. E4: cuestión abierta resuelta parcialmente —
un snapshot exportado post-migración NO reimporta hasta el flip (v7).
