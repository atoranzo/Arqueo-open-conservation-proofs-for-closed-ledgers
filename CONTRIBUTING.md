# Guía de contribución — Arqueo (antes ZK-SSL)

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

⚠️ **Si usas un asistente de IA generativa, dilo en el PR**: qué
modelo y para qué. Este repositorio declara su propio uso en
[`GENAI.md`](./GENAI.md), y aquí manda la misma regla del párrafo anterior:
el mensaje describe lo que el commit contiene. Lo generado sin medir, sin
ejecutar y sin que alguien responda de ello no entra.

⚠️ **Lo que un PR no ve: los sellos y sus asientos** (§793). Aquí un cambio
entra como un *sello*: un commit que lleva su asiento en
[`AUDITORIA.md`](AUDITORIA.md), con su commit, sus contadores, lo que el
cambio **no** cierra y las lecciones que dejó (`GENAI.md` §«Dónde está el
registro»), y un asiento no se reescribe después: se corrige con otro. Cómo
se escribe, cómo se corrige un documento vivo, las convenciones del
`BACKLOG.md`, los RFC con sus etapas y los niveles del canon y los bancos los
cuenta [`doc/METODO.md`](doc/METODO.md), con la cita al fichero que
establece cada cosa.

## 4. Pruebas y validación

```bash
cargo test --workspace --all-features
```

⚠️ **Esa orden corre en DEPURACION, y la capa NO se mide ahí.**
Medido el 26-08-2026: `cargo test -p zk-ssl` en depuración da 188 pasan,
93 fallan y 9 ignorados; en release, 307 pasan, 0 fallan y 3 ignorados. Esos
fallos **no son un defecto de solidez**: son el límite de grados de la
entrada 41 y del §78, declarado y decidido -winterfell comprueba en
depuración que el grado declarado se realice, y hay restricciones cuyo
grado depende del testigo-. Pero verás rojo si corres la orden de arriba
tal cual. **La capa se mide con `cargo test -p zk-ssl --release`**, que es lo
que corre `tools/canon.sh`.

⚠️ **§814: remedido el 15-10-2026, y la capa ya no da rojo en depuración.**
`cargo test --locked -p zk-ssl` en depuración da 443 pasan, 0 fallan y 45
ignorados, sobre `0315b14` (el §813), en 3.253,94 s con otro trabajo en la
máquina (4 CPU, carga de 9,2 a 18,6). En release, la fila de la capa en
`tools/canon.sh` fija hoy 515 pasan y 7 ignorados (⚠️ §839: decía 481; el 482.º
es el censo de las longitudes, que corre también en depuración, donde serían
444 pasan: los 443 son de `0315b14`, y la capa entera no se ha vuelto a correr
ahí. ⚠️ §840: decía 482; los dos de más, el censo de los `native_leaf`, corren
también ahí: serían 446. ⚠️ §842: decía 484; el de más, el censo de las bandas de
bytes, también: serían 447. ⚠️ §816: decía 485; los once de más, los del cerrojo del libro, no
se han corrido en depuración. ⚠️ §817: decía 496; el de más, el de la instantánea en claro,
tampoco. ⚠️ §819: decía 497; el de más, el de la negativa del barrido, tampoco. ⚠️ §820: decía 498;
los tres de la exportación, tampoco. ⚠️ §821: decía 501; los dos de la capa importada con clave,
tampoco. ⚠️ §828: decía 503; los doce del recifrado y la migración del libro de otro dueño, tampoco). De los 45, 7 son instrumentos de medida, ignorados en todo perfil, y 38 llevan
`cfg_attr(debug_assertions, ignore = …)`: en depuración no corren, y corridos ahí con `--ignored`
pasan los 38. Ya no hay fallos que atribuir al límite de grados, y con esta medida se cierra la
entrada 41 de `BACKLOG.md`. Sigue en pie: esos 38 solo los corre release, y la capa se mide con
`--release`. Si la orden de arriba, el workspace entero con `--all-features`, da rojo hoy no se
sabe: no se ha remedido. Y el «307» de release no era del 26-08-2026: aquel día fueron 287 (§366),
y los sellos lo subieron con el pin hasta el §394 sin tocar la fecha.

⚠️ **§855: el canon, en un clon recién hecho.** Se corre desde la raíz con
`bash tools/canon.sh --sello`; los niveles y lo que cubre cada uno, en
[`doc/METODO.md`](doc/METODO.md). Antes, una vez y con red:

```bash
cargo fetch --locked
```

`tools/check_curvas.py`, una de las herramientas del canon, lee la clausura de
los binarios de producción con `cargo tree --offline --target all`, que pide el
fuente de cada crate en todos los targets, y un `cargo build` sólo baja los de
tu máquina (§843). Sin ese paso, o con la caché de cargo vacía, el canon avisa
al empezar, con el remedio, y `check_curvas` sale ROJO en su bloque. No baja
nada. Los binarios los busca donde los deja cargo: lo dice `tools/target_dir.sh`,
que lo lee de `cargo metadata` y sigue `CARGO_TARGET_DIR`,
`CARGO_BUILD_TARGET_DIR` y el `build.target-dir` de un `.cargo/config.toml` (⚠️ §824: con `build.target-dir`, o con cualquier otra clave de `[build]`, en el `.cargo/config.toml` o el `.cargo/config` de la raíz o de un directorio padre, o en el `config.toml` o el `config` de `CARGO_HOME`, el «3 ter» del canon sale ROJO: `tools/artefacto.sh` compila en su propio `target` y rechaza la tabla antes de compilar, porque otras claves suyas cambian lo que se compila; `doc/KIT.md` §«Reproducir el kit, para quien no se fíe de la descarga»).

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
