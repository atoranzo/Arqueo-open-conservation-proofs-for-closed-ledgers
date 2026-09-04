# Guía de contribución — ZK-SSL

Lee antes [`SECURITY.md`](SECURITY.md): el estado del proyecto y sus límites
conocidos condicionan qué tipo de contribución tiene sentido aquí.

---

## 1. Filosofía

- **Medir antes que afirmar.** La aportación distintiva de este repositorio es
  la comparación empírica de sistemas de prueba. Toda afirmación de rendimiento
  o de seguridad debe traer el número, el método o la prueba que la respalda.
  **Cifras sin medición detrás no entran.**
- **Distinguir lo medido de lo estimado de lo supuesto.** Un «~X» sin origen es
  una deuda, no un dato.
- **La solidez manda sobre la velocidad.** Un cambio en un circuito o en la
  función de transición es un cambio en **qué es dinero válido**. Se revisa con
  más cuidado que cualquier otra cosa.
- **Honestidad sobre el estado.** Si un componente es andamiaje, dilo en el
  código y en el PR. No presentes un esqueleto como una implementación.
- **Sin exageración.** Este proyecto no promete un nuevo orden mundial: propone
  y mide propiedades criptográficas concretas.

⚠️ **Y una regla que este proyecto aprendió a su costa** (`AUDITORIA.md`
§59.2): **toda herramienta, ayudante o barrido declara su cobertura, y la
declaración se comprueba.** Una comprobación aplicada a parte del código sin
decir a qué parte es peor que ninguna, porque parece cobertura.

## 2. Entorno de desarrollo

- **Lenguaje**: Rust, edición estable actual.
- **Compilación**: el código debe compilar **sin advertencias de `rustc`**, y
  hoy lo hace.

### Formato y linting

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
```

⚠️ **`clippy -D warnings` NO se exige, y conviene saber por qué.** Medido el
01-08-2026:

| | avisos |
|---|---|
| Workspace completo | **189** |
| `zk-ssl` + `stark-experiment` —el sistema— | **155** |
| De ellos, «the loop variable is used to index» | **122** |
| Código muerto en el sistema | **0** |

Los 122 son un **falso positivo del dominio**: en un circuito el índice **es el
dato** —posición en la traza, ranura de restricción, columna—, y reescribir
`for i in 0..4 { result[C_KEY + i] = next[8 + i] - current[COL_KEY + i] }` con
iteradores perdería exactamente lo que hay que poder leer.

**No se silencian con un `allow` global**: se declaran aquí. Un `allow` global
convertiría los 33 restantes en invisibles, y entre ellos hay avisos que este
proyecto ha visto señalar cosas reales dos veces.

⚠️ **El código muerto que clippy encuentra está todo en `crates/ceremony`** —la
maquinaria de setup de los backends que **no se eligieron**—. Es material de la
comparativa, no del camino de producción.

**Para una contribución**: no empeores el número en los ficheros que toques.

## 3. Flujo de trabajo

1. *Fork* y clón local.
2. Rama con nombre descriptivo: `feat/`, `fix/`, `crypto/` —circuitos o
   primitivas—, `docs/`.
3. Commits pequeños y con mensajes claros.

⚠️ **El mensaje de un commit describe lo que ese commit contiene**, no lo que
pretendías hacer. Este proyecto tiene registrados **tres mensajes que
afirmaban lo que su commit no hacía** (`AUDITORIA.md` §91), y el historial es
tan documento del proyecto como el README.

## 4. Pruebas y validación

```bash
cargo test --workspace --all-features
```

⚠️ **Esa orden corre en DEPURACION, y la capa NO se mide ahí.**
Medido el 26-08-2026: `cargo test -p zk-ssl` en depuración da 188 pasan,
93 fallan y 9 ignorados; en release, 301 pasan, 0 fallan y 3 ignorados. Esos
fallos **no son un defecto de solidez**: son el límite de grados de la
entrada 41 y del §78, declarado y decidido -winterfell comprueba en
depuración que el grado declarado se realice, y hay restricciones cuyo
grado depende del testigo-. Pero verás rojo si corres la orden de arriba
tal cual. **La capa se mide con `cargo test -p zk-ssl --release`**, que es lo
que corre `tools/canon.sh`.

Y para los circuitos:

```bash
python3 tools/check_constraint_layout.py
python3 tools/verificar_citas.py
```

Cruza los índices de `result[...]` **y** de `periodic[...]` en los 27
circuitos: colisiones, desbordes y ranuras muertas. Tiene autotest propio
—`--autotest`— sobre casos que fallaron de verdad.

### Si tocas un circuito

- **Ejecuta en release y en depuración.** No dan lo mismo: en depuración
  winterfell valida al generar y **nombra la restricción y el paso**
  (`AUDITORIA.md` §77.1).
- **Añade tests de solidez negativos**: casos que **deben** fallar. Un cambio en
  un circuito sin un test que demuestre qué deja de ser aceptable está
  incompleto.
- ⚠️ **El positivo va primero y solo** (§66.2). Con el positivo roto, **todos
  los negativos pasan** — y pasan por el motivo equivocado.
- ⚠️ **Un test negativo que pasa puede estar pasando por la razón
  equivocada.** Comprueba que falla por lo que crees: §77 registra uno que
  fallaba por el error correcto en un modo y por otro distinto en el otro.

**Ningún cambio se fusiona sin pasar la suite.**

## 5. Pull Requests

- **Descripción**: qué problema resuelve, la lógica implementada y **cómo lo
  validaste** —con números o pruebas si aplica—.
- **Cambios en circuitos, en la transición de estado o en el verificador**:
  requieren revisión explícita del mantenedor.
  ⚠️ **No hay especificación formal del AIR contra la que contrastar**
  (`SECURITY.md` §3.1). Mientras no exista, el PR debe **explicitar contra qué
  reglas dice cumplir**, porque se está trabajando sobre el código y no sobre
  un contrato — y eso hay que decirlo.
- **Afirmaciones de seguridad o rendimiento**: con su medición, o etiquetadas
  como conjetura.
- **Alcance**: un PR, un cambio conceptual. Los cambios de formato o de estado
  que afecten a la privacidad o a la solidez van **separados y señalados**.

## 6. Qué encaja y qué no

**Encaja**: mejoras medibles; **especificación formal y tests de solidez de los
circuitos** —la carencia n.º 1—; cierre de los límites de `SECURITY.md` mediante
mecanismos verificables; más rigor, no más superficie.

**No encaja sin discusión previa con números**: sistemas de prueba que
reintroduzcan ceremonias de confianza o supuestos no post-cuánticos contra la
tesis del núcleo; un token, *staking* o mercados de validadores que presupongan
una descentralización que el proyecto **no tiene**; apilar componentes —zkVM,
identidad, disponibilidad modular— que aumenten la superficie sin resolver un
problema presente y medido.

**El minimalismo aquí no es estética**: cada pieza añadida es superficie que
alguien tendrá que auditar, y **no hay auditoría**.

---

*Mantén este documento consistente con el código. Cuando dejen de coincidir,
el código gana.*
