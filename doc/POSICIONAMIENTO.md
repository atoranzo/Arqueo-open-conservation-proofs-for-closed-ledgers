# ZK-SSL — Posicionamiento en una página

**Qué es.** El núcleo criptográfico verificable de una liquidación
soberana: propiedades monetarias demostrables en circuito
(conservación del valor, autoridad de gasto, tope de emisión, no
doble-gasto, caducidad de pendientes con doble cerrojo), cumplimiento supervisable **sin revelar el libro**
(revelación exacta / mínima / de banda), y **sin ceremonia de setup**
(STARK/FRI, post-cuántico). Cinco backends medidos sobre el mismo
circuito real (`FIVE_BACKENDS.md`); rendimiento y suites en
`PERFORMANCE.md` y en el registro (`AUDITORIA.md`).

**Qué lo distingue.** La combinación que resuelve la tensión clásica
confidencialidad↔supervisión: el supervisor verifica pruebas, no lee
estado. Y una documentación **radicalmente honesta**: cada residuo de
confianza está medido, con nombre y con su ataque diseñado
(`SECURITY.md`, `doc/CONFIANZA_RESIDUAL.md`).

**Qué NO es todavía.** Un sistema de producción: hoy es un nodo único
—el operador ve el estado; la privacidad es frente a terceros—, sin
consenso/replicación, sin auditoría externa, y con el rendimiento del
anclaje como trabajo explícito de escalado (`doc/ESCALADO.md`). Las
vías antiguas pendientes de retirada están inventariadas
(BACKLOG 32/33).

**Dirección.** *El consenso es el último intermediario*: este proyecto
lo deja mínimo, medido y con evidencia — ceremonia eliminada, claves de
custodio fuera del nodo, hoja envuelta (entrada 50 cerrada), acuse
diseñado (§121). El dinero cuántico podrá retirarlo del todo; esta es
la aproximación clásica disponible hoy (`PRINCIPIOS.md` §6.bis).

**Conversación que se propone.** Piloto acotado y verificable: p. ej.
supervisión de límites o reservas mediante **pruebas de banda** sobre
un instrumento de bajo volumen, en entorno cerrado, con dual-run —
precedido de auditoría externa formal del AIR y los circuitos.

*Angel Toranzo Portela · MIT / Apache-2.0 · código y registro públicos*
