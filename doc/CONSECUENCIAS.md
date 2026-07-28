# Consecuencias de implantar un sistema tipo ZK-SSL

*Liquidación con privacidad ZK, cumplimiento demostrable, sin ceremonia de
setup, límites explícitos.*

> Análisis prospectivo. Describe lo que **ocurriría** si algo así se
> implantara, no lo que este código hace hoy. Para eso está
> [`AUDITORIA.md`](../AUDITORIA.md).

---

## 1. Por dimensiones

### Financieras

**Positivas**
- Menos fricción en auditoría: pruebas selectivas en lugar de cesión masiva
  de libros.
- Reducción de costes de intermediación y conciliación.
- Menor riesgo de creación opaca de valor si las reglas están en circuito.
- Trazabilidad de integridad del historial.

**Negativas**
- Coste computacional de generar pruebas. **Medido**: 620 ms una
  transferencia, 32 ms un pago sin conexión.
- Complejidad operativa nueva: claves, clientes que prueban, recuperación.
- Riesgo de concentración en quien opera nodos.
- Transición cara desde cores bancarios legados.

### Económicas

**Positivas**
- Puede abaratar cumplimiento y aumentar competencia en liquidación.
- Facilita dinero programable con privacidad acotada.
- Reduce asimetrías de información en auditorías.

**Negativas**
- No resuelve crédito, política monetaria ni estabilidad macro.
- Dualidad de sistemas durante años.
- Beneficios capturados primero por actores con capacidad técnica.

### Sistémicas

**Positivas**
- Menos réplicas masivas de datos sensibles. **Con retención distribuida,
  el operador guarda 137 bytes por operación en vez de 62 KB**: 463 veces
  menos.
- Diseño minimalista más eficiente que redes infladas.
- Sin ceremonias, menos dependencias frágiles de setup global.

**Negativas**
- Generación de pruebas consume energía y hardware.
- Riesgo de estandarizar una capa crítica con pocos implementadores.
- Efectos reales dependen de escala; hoy es demostrativo.

### Crisis actual

**Positivas**
- Responde a la erosión de confianza: verificación en lugar de fe opaca.
- Cumplimiento sin vigilancia total.
- Encaja donde no todos confían en el mismo intermediario.

**Negativas**
- En crisis aguda puede prevalecer el control sobre la privacidad.
- Un nodo único sigue siendo punto de fallo y censura.
- **Puede usarse para endurecer control selectivo con mejor relato
  técnico.**

### Tecnológicas

**Positivas**
- Empuja estándares sin setup y post-cuánticos.
- Separa prueba (cliente) de aplicación (capa).
- Obliga a tests discriminantes y honestidad de límites.

**Negativas**
- Madurez desigual de librerías. **Cuatro de seis permiten en silencio un
  setup inseguro.**
- Superficie de error en circuitos, nullifiers, árboles y APIs. **Este
  proyecto encontró seis fallos en código que ya pasaba sus tests.**
- Falsa sensación de seguridad si se ignora el poder residual del operador.

### Humanas

**Positivas**
- Dignidad informacional: demostrar sin desnudarse.
- Cultura de límites explícitos.
- Menos custodia forzada de claves en el intermediario.

**Negativas**
- Exige alfabetización criptográfica; riesgo de nueva élite técnica.
- **Perder la clave es perder el acceso.** La recuperación por custodios lo
  mitiga a cambio de darles poder.
- Puede aumentar la ansiedad: *"si es matemático, ¿quién responde?"*

### Geopolíticas

**Positivas**
- Soberanía operativa para quien no quiera depender de stacks ajenos.
- Cumplimiento verificable sin ceder bases de datos completas.
- Menos dependencia de ceremonias o proveedores globales.

**Negativas**
- Fragmentación de estándares por bloques.
- Estados pueden exigir excepciones que rompan el modelo.
- Tensión entre privacidad ciudadana y soberanía de supervisión.

### Cambio de paradigma

**Positivas**
- De *"confía en la institución"* a *"verifica la propiedad"*.
- De transparencia o vigilancia totales a **revelación mínima
  demostrable**.
- De promesas absolutas a sistemas con límites declarados.

**Negativas**
- **Riesgo de tecno-solucionismo**: creer que el circuito resuelve política
  y poder.
- Resistencia al cambio de base de legitimidad.

### Arquitectónicas

**Positivas**
- Capa estrecha, invariantes claras, API que no exige entregar claves.
- Registro encadenado, instantáneas verificables y cifradas.

**Negativas**
- **Nodo único**: arquitectura incompleta respecto al principio.
- Complejidad de estado no legible por el operador frente a usabilidad.
- Integración con ISO 20022 y legado no trivial.

### Regulatorias

**Positivas**
- Auditoría selectiva alineable con prevención de blanqueo basada en riesgo.
- Mejor evidencia de controles sin acceso masivo a datos.
- Trazabilidad de intervenciones mediante contadores públicos.

**Negativas**
- Los marcos actuales asumen acceso a libros y responsables centralizados.
- Ambigüedad legal sobre pruebas ZK como evidencia.
- Riesgo de regulación hostil o de excepciones que rompan la privacidad.

---

## 2. DAFO

| Fortalezas | Debilidades |
|---|---|
| Propiedades demostrables | **Nodo único** |
| Sin ceremonia de setup | Coste de prueba y madurez operativa |
| Honestidad estructural | **No auditado por nadie** |
| Separación clave / nodo | Usabilidad y recuperación de claves |
| | **Contrapartes ven más de lo debido** (§5 de APORTACION) |

| Oportunidades | Amenazas |
|---|---|
| Dinero mayorista con privacidad acotada | Exigencia de puertas traseras |
| Reporting regulatorio por pruebas | Captura por operadores concentrados |
| Jurisdicciones con soberanía tecnológica | Stacks institucionales más completos |
| Estándares post-cuánticos tempranos | Errores criptográficos o de implementación |

---

## 3. PESTEL

| Factor | Lectura |
|---|---|
| **Político** | Soberanía frente a control; la fragmentación favorece y complica a la vez |
| **Económico** | Ahorro en cumplimiento a medio plazo; coste de transición alto |
| **Social** | Demanda de privacidad **y** de control del delito; tensión no resuelta |
| **Tecnológico** | ZK madura rápido; interoperabilidad y talento son cuellos de botella |
| **Ecológico** | Impacto energético de las pruebas; posible compensación si reduce réplicas |
| **Legal** | Encaje incierto con blanqueo, privacidad, responsabilidad y valor probatorio |

---

## 4. Cuatro cuadrantes

**Individual-interior.** Fomenta responsabilidad sobre claves y límites.
Puede generar confianza lúcida o **fe mágica en lo matemático**.

**Individual-exterior.** Nuevas prácticas: custodia de claves, verificación
de pruebas, operación de nodos. Errores humanos más duros.

**Colectivo-interior.** Cultura de verificación y de no vender lo no
construido. Choque con culturas de opacidad institucional o de relato
tecno-utópico.

**Colectivo-exterior.** Nueva capa en el stack de pagos. Rediseño de
supervisión y reporting. Riesgo de arquitectura dual y de concentración
técnica.

---

## 5. Síntesis

### Lo que aporta si se implanta con coherencia

Un desplazamiento **parcial** de la fe institucional hacia la verificación
criptográfica, con privacidad frente a terceros y cumplimiento sin desnudez
total del libro — **siempre que se declaren y acoten los poderes residuales
del operador**.

### Lo que no aporta

Ni soberanía automática, ni justicia macroeconómica, ni neutralidad
política, ni eliminación del poder.

Sin consenso distribuido, auditoría externa y marco legal, **sigue siendo un
recipiente poderoso pero incompleto**.

### Condición de impacto positivo

Mantener la honestidad fundacional: **no vender como soberano o sin
confianza lo que aún depende de un operador, de claves humanas y de
instituciones que autorizan emisión y resuelven conflictos.**
