# Triaje de las 14 medidas (documentos externos) contra el registro

**Origen.** Las 14 medidas están ancladas a un artículo de prensa
(«el consenso es el último intermediario»), no a `AUDITORIA.md`. Describen
la dirección del proyecto pero reflejan un estado de hace semanas.
Colacionadas 2026-08-03. La regla de la casa: nada que revierta un asiento
sin revertir el asiento explícitamente. Resumen citable en BACKLOG entrada 70.

## GRUPO 1 — YA HECHAS o EN CURSO (implementar la medida = deshacer trabajo)
- **3a** «accessors exijan credencial o se eliminen» → §129 decidió lo
  CONTRARIO: son del operador (97 usos legítimos); se AÑADE
  `account_view_authenticated` (49-A paso 4), no se lisian. NO implementar.
- **3b** «salt en native_leaf» → entrada 50 / B13-B14, diseñada (§132).
- **1/2/11** «custodios vía B, medir» → entradas 32/33, diseño cerrado
  (§47/§51). Matiz que la lista no ve: migrar sin resolver el orden
  estricto IDX_B-IDX_A-1 abre doble-uso de custodio (§51).
- **5b** vinculabilidad del pendiente → residuo ya registrado.

## GRUPO 2 — CONTRADICEN un asiento
- **4a** «eliminar transfer()» → ni está deprecated; su retirada ES la 32.
- **4b/14** «actualizar preprints» → §135 PROHÍBE editar publicados; van por errata.

## GRUPO 3 — VÁLIDAS y alineadas
- **8** reordenar backlog por confianza → ya es el orden de facto.
- **9** tests de «qué aprende cada participante» (operador/contraparte/tercero),
  extendiendo §16 → NUEVA, la mejor: mecanismo, no narrativa. IMPLEMENTAR.
- **6** notas/UTXO → arquitectura mayor, sesión propia.

## GRUPO 4 — NUEVAS documentales (no tocan solidez)
- **7** sección «dinero cuántico» en docs vivos (preprints por errata).
- **10** interfaz de anclaje externo → conecta con el intermediario residual
  = orden+completitud, que el acuse (§119/§121) ataca.
- **12** actualizar SECURITY.md/modelo de amenaza.
- **13** documento de posicionamiento público.

## Veredicto
De 14: ~4 ya hechas o en curso con más rigor (3a/3b/1/2/11), 2 revierten
asientos (4b/14→§135, 3a→§129), y las nuevas sin choque son la 9 (la mejor)
y las documentales 7/10/12/13. Implementarlas en bloque desharía trabajo.
