# Qué aporta ZK-SSL

Comparado con el sistema bancario público/soberano actual y con las
blockchains privadas, su aportación no es *sustituir todo*, sino cambiar la
base de la confianza en un punto concreto: **la liquidación con privacidad
y cumplimiento demostrable**.

> ⚠️ **Corrección respecto a la versión inicial de este documento.** Decía
> que la privacidad es *"frente a terceros que solo ven pruebas"*. Es
> cierto pero incompleto: **una contraparte no es un tercero**. Ver §5.

---

## 1. Respecto al sistema bancario público / soberano actual

| Dimensión | Banca actual | ZK-SSL |
|---|---|---|
| **Base de confianza** | Instituciones, reguladores, auditorías y procesos legales | Pruebas criptográficas verificables + confianza residual **declarada** en el operador |
| **Privacidad** | Opaca para el público; visible para el banco y, bajo requerimiento, para el Estado | Frente a terceros que solo ven pruebas. ⚠️ El operador ve el estado, y **una contraparte ve tu saldo** (§5) |
| **Cumplimiento** | Reportes, inspecciones, acceso al libro | **Sin entregar el libro**: el titular prueba banda/mínimo/saldo; el supervisor verifica sin ledger |
| **Creación de dinero** | Política monetaria e institucional | Reglas explícitas en circuito: tope inmutable, emisión con umbral. **No se puede crear valor sin romper la prueba** |
| **Integridad del historial** | Controles internos y supervisión | El operador **no puede reescribirlo en silencio** (registro encadenado). Sí puede censurar y ordenar |
| **Finalidad** | Legal y operativa (T+N) | Criptográfica, por transición |
| **Transparencia sobre límites** | Implícita o diluida en complejidad institucional | Límites y poderes residuales **en primer plano** |

### Aportación central

Introduce **verificabilidad matemática** en propiedades que hoy se sostienen
sobre fe institucional y auditoría retrospectiva: conservación del valor,
autorización de gasto, no doble gasto y cumplimiento selectivo sin
exposición total del libro.

**No elimina al Estado ni al regulador**: cambia el tipo de evidencia que
pueden exigir y verificar.

---

## 2. Respecto a las blockchains privadas

| Dimensión | Blockchain privada típica | ZK-SSL |
|---|---|---|
| **Quién valida** | Consorcio de nodos conocidos | Hoy: **nodo único**, declarado |
| **Privacidad** | Parcial (canales, acceso restringido) o inexistente entre miembros | Conocimiento cero: terceros ven pruebas, no importes ni identidades |
| **Cumplimiento** | Suele requerir acceso privilegiado a datos | Revelación selectiva **sin acceso al ledger** |
| **Ceremonia de setup** | Depende de la pila | **Prohibida** como dependencia soberana |
| **Integridad** | Consenso del consorcio | Transiciones demostradas; historial no reescribible en secreto |
| **Modelo de confianza** | En el conjunto de validadores | **Mínima y explícita**: se nombra lo que depende del operador |
| **Minimalismo** | Contratos, gobernanza compleja, permisos | Capa estrecha + invariantes + auditoría selectiva |

### Aportación central

La mayoría de cadenas permisionadas **desplazan la confianza del banco al
consorcio**. Esto la desplaza hacia pruebas verificables, y **se niega a
esconder el poder residual del operador**.

Además rechaza backends que reintroducen ceremonias inauditables, **aunque
sean más rápidos**: Groth16 produce pruebas 320 veces menores y se descartó.

No es *"otra blockchain privada con ZK"*. Es una capa de liquidación con
propiedades demostrables y una ética de diseño: **no vender soberanía donde
aún hay intermediario**.

---

## 3. Aportación específica

**Cumplimiento sin libro mayor.** El supervisor verifica banda, mínimo o
saldo exacto sin ver el estado completo. Raro tanto en banca core como en
redes privadas.

**Ausencia de ceremonia como decisión de soberanía.** No es preferencia
técnica: si el setup permite crear dinero invisible, se descarta.

**Honestidad estructural.** Declara que el operador ve saldos y puede
censurar. Ni la banca ni la mayoría de cadenas privadas formulan así su
poder.

**Separación clave / nodo.** La clave de gasto no viaja al operador; la
prueba se genera en el cliente.

**Comparativa empírica sobre una aplicación real.** No benchmarks de
SHA-256: el mismo circuito de liquidación en cinco backends, con hallazgos
que solo aparecen al portar estado.

---

## 4. Qué NO aporta

- No sustituye política monetaria, supervisión macro ni marco legal.
- **No es hoy una red descentralizada.**
- No elimina al intermediario operativo.
- **No es más soberano que un banco central por usar ZK**: es más
  verificable en un subconjunto de propiedades.
- **No está auditado.** Ni parcialmente ni a ninguna escala.

---

## 5. ⚠️ Los límites de privacidad que este documento omitía

Encontrados aplicando la pregunta *"¿qué aprende cada participante?"* a
código que ya funcionaba. Detalle en [`AUDITORIA.md`](../AUDITORIA.md).

**El pagador ve el saldo del receptor.** La liquidación actualiza las dos
hojas, así que quien construye la prueba necesita el saldo del receptor.
**Pagar un euro a alguien revela cuánto tiene.** Corrección analizada
—transferencias por notas— sin implementar.

**El banco enlaza los pagos sin conexión.** El mismo compromiso aparece en
la emisión y en el gasto: el banco sabe quién pagó a quién. **No es
privacidad como el efectivo.**

Ninguno estaba declarado cuando se escribió la primera versión de este
documento. **Una contraparte no es un tercero**, y la distinción no era un
matiz.

---

## Resumen

Frente a la banca soberana actual, aporta **evidencia criptográfica donde
hoy hay fe institucional**.

Frente a las blockchains privadas, aporta **privacidad con cumplimiento
demostrable y rechazo explícito a la confianza oculta** —ceremonias,
poderes no declarados— en lugar de limitarse a cambiar de intermediario.

Y su límite, dicho sin suavizar: **sigue habiendo un operador que lo ve
todo, contrapartes que aprenden más de lo debido, y ninguna auditoría
externa.**
