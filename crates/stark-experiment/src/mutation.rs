//! **Prueba por mutación**: encontrar restricciones que no imponen nada.
//!
//! ## El problema que resuelve
//!
//! Una restricción puede existir, estar declarada, tener su grado
//! asignado — y **no imponer nada**. Los casos vistos en este proyecto:
//!
//! - Una restricción **idénticamente cero**: `result[C_X] = 0 * algo`.
//! - Una restricción que **nunca se asigna**: `result[C_X]` se queda a cero.
//! - Una restricción multiplicada por una **columna periódica vacía**, que
//!   vale cero en todas las filas.
//!
//! **Ningún test los detecta.** El testigo honesto las satisface —valen
//! cero, como deben— y los adversariales fallan por otras restricciones
//! antes de llegar a ellas.
//!
//! `AUDITORIA.md` documenta tres apariciones de este modo de fallo, y que
//! la comprobación estática **no puede distinguirlo** de código correcto.
//!
//! ## La técnica
//!
//! Si **ninguna perturbación** de una celda hace que una restricción se
//! vuelva no nula, esa restricción no impone nada.
//!
//! ```text
//! 1. Construir una traza VÁLIDA (todas las restricciones a cero).
//! 2. Para cada celda (columna, fila): cambiarla.
//! 3. Evaluar las DOS transiciones afectadas: (fila−1 → fila) y (fila → fila+1).
//! 4. Anotar qué índices de restricción se vuelven no nulos.
//! 5. Los que NUNCA se disparan: vacíos.
//! ```
//!
//! No genera ni una prueba: solo evalúa restricciones.
//!
//! ## ⚠️ Lo que NO encuentra
//!
//! - Restricciones que solo reaccionan a cambios de **varias celdas a la
//!   vez**. Aparecerán como falsos positivos.
//! - Restricciones que **sí se disparan pero imponen lo que no se cree**.
//!   Eso ninguna herramienta lo detecta: exige leer y entender.
//! - Restricciones de **frontera** (aserciones), que son otro mecanismo.
//! - Restricciones del tramo **auxiliar**: sólo evalúa las principales. En
//!   la edad, las 9 auxiliares quedan sin mirar.
//!
//! Un resultado limpio **no significa que el circuito sea correcto**.
//! Significa que no tiene este fallo concreto.

use winterfell::math::{fields::f64::BaseElement, FieldElement};
use winterfell::{Air, EvaluationFrame};

/// Lee la traza entera como filas.
///
/// Se necesita para poder perturbar celdas sin tocar la `TraceTable`.
pub fn rows_of(trace: &winterfell::TraceTable<BaseElement>, width: usize, length: usize) -> Vec<Vec<BaseElement>> {
    (0..length)
        .map(|r| (0..width).map(|c| trace.get(c, r)).collect())
        .collect()
}

/// Evalúa las restricciones de transición en un paso concreto.
fn evaluar<A>(air: &A, rows: &[Vec<BaseElement>], periodic: &[Vec<BaseElement>], paso: usize) -> Vec<BaseElement>
where
    A: Air<BaseField = BaseElement>,
{
    let width = rows[0].len();
    let mut frame = EvaluationFrame::<BaseElement>::new(width);
    frame.current_mut().copy_from_slice(&rows[paso]);
    frame.next_mut().copy_from_slice(&rows[paso + 1]);

    // El valor de cada columna periódica en este paso.
    let vals: Vec<BaseElement> = periodic
        .iter()
        .map(|col| col[paso % col.len()])
        .collect();

    // ⚠️ Las PRINCIPALES, no todas: `evaluate_transition` sólo llena esas.
    // Con un tramo auxiliar, `num_transition_constraints` las suma también, y
    // las auxiliares saldrían como vacías sin serlo (la edad: 110 + 9). En un
    // AIR de un solo tramo las dos cuentas coinciden.
    let mut result = vec![BaseElement::ZERO; air.context().num_main_transition_constraints()];
    air.evaluate_transition::<BaseElement>(&frame, &vals, &mut result);
    result
}

/// Resultado del análisis.
pub struct Informe {
    /// Índices de restricción que **ninguna perturbación dispara**.
    pub nunca_disparadas: Vec<usize>,
    /// Cuántas restricciones hay en total.
    pub total: usize,
    /// Cuántas celdas se perturbaron.
    pub celdas: usize,
}

/// Busca restricciones que ninguna perturbación de una celda activa.
///
/// `muestra_filas` limita cuántas filas se prueban: `1` las prueba todas,
/// `8` una de cada ocho. Con trazas largas conviene muestrear, porque el
/// coste es `columnas × filas × 2` evaluaciones.
///
/// ⚠️ **Muestrear puede producir falsos positivos**: una restricción activa
/// solo en filas no muestreadas aparecerá como vacía.
pub fn buscar_vacias<A>(
    air: &A,
    rows: &[Vec<BaseElement>],
    muestra_filas: usize,
) -> Informe
where
    A: Air<BaseField = BaseElement>,
{
    let periodic = air.get_periodic_column_values();
    // Las principales: las auxiliares no se evalúan aquí (ver `evaluar`).
    let n = air.context().num_main_transition_constraints();
    let width = rows[0].len();
    let length = rows.len();

    let mut disparada = vec![false; n];
    let mut celdas = 0usize;

    // La traza válida no debe disparar nada: es la referencia.
    //
    // ⚠️ `assert!` y no `debug_assert!`: la suite corre en `--release`, y
    // con una referencia rota el informe marca como «disparadas»
    // restricciones que ya lo estaban antes de perturbar nada. Era un
    // `debug_assert!` y `circuit_audit` llegó a pasar así (`AUDITORIA.md`
    // §20).
    for paso in (0..length - 1).step_by(muestra_filas) {
        let base = evaluar(air, rows, &periodic, paso);
        for (i, v) in base.iter().enumerate() {
            assert_eq!(
                *v,
                BaseElement::ZERO,
                "la traza de referencia no es valida: restriccion {i} en el paso {paso}"
            );
        }
    }

    let delta = BaseElement::new(0x9E37_79B9_7F4A_7C15);

    for fila in (0..length).step_by(muestra_filas) {
        for col in 0..width {
            celdas += 1;
            let mut mut_rows = rows.to_vec();
            mut_rows[fila][col] += delta;

            // Solo las transiciones que tocan esta fila cambian.
            for paso in [fila.saturating_sub(1), fila] {
                if paso + 1 >= length {
                    continue;
                }
                let r = evaluar(air, &mut_rows, &periodic, paso);
                for (i, v) in r.iter().enumerate() {
                    if *v != BaseElement::ZERO {
                        disparada[i] = true;
                    }
                }
            }
        }
    }

    Informe {
        nunca_disparadas: (0..n).filter(|i| !disparada[*i]).collect(),
        total: n,
        celdas,
    }
}

/// **Los bits altos de un segmento de rango** (RFC-0017, §641).
///
/// Un segmento de rango de `largo` filas descompone un valor en `largo`
/// bits, del más alto (fila `inicio`) al 0 (fila `inicio + largo - 1`), y
/// la columna `col_acc` los acumula por Horner. Desde el RFC-0017 el
/// selector de la primera fila cubre también la segunda: el segmento
/// fuerza a cero sus DOS bits más altos. En los de 64 filas son el 63 y el
/// 62, y el rango baja a 62 bits; con uno solo era de 63, y una resta que
/// envuelve en Goldilocks cabía en él. En los de 8 filas (índices de
/// custodio) el rango es de 6 bits.
///
/// Para cada uno de los tres bits más altos pone a uno el de su fila y
/// rehace el acumulador desde ella. Bit y acumulador siguen coherentes: lo
/// único que cambia es el valor. Después evalúa las transiciones del
/// segmento, salvo la del enlace final, que compara el acumulador con el
/// valor y saltaría con cualquier bit.
///
/// Devuelve, del más alto al tercero, `true` si alguna restricción se
/// dispara. Dos bits forzados dan `[true, true, false]`. El rango de 63
/// bits que cerró el RFC-0017 daba `[true, false, false]`.
///
/// ⚠️ Mira el segmento, no el valor: no sabe de dónde sale el valor ni si
/// el enlace lo ata. Eso lo prueban los falsadores de cada circuito con el
/// probador y el verificador reales.
///
/// ⚠️ Precondición: en la referencia los tres bits valen cero (el valor del
/// segmento es menor que `2^(largo - 3)`) y el segmento cumple sus
/// restricciones.
pub fn bits_altos_rechazados<A>(
    air: &A,
    rows: &[Vec<BaseElement>],
    col_bit: usize,
    col_acc: usize,
    inicio: usize,
    largo: usize,
) -> [bool; 3]
where
    A: Air<BaseField = BaseElement>,
{
    let periodic = air.get_periodic_column_values();
    // Hasta el paso anterior al del enlace, que es `inicio + largo - 2`.
    let pasos = inicio..inicio + largo - 2;

    for paso in pasos.clone() {
        let base = evaluar(air, rows, &periodic, paso);
        assert!(
            base.iter().all(|v| *v == BaseElement::ZERO),
            "la referencia no cumple sus restricciones en el paso {paso}"
        );
    }

    let mut salida = [false; 3];
    for (p, disparada) in salida.iter_mut().enumerate() {
        assert_eq!(
            rows[inicio + p][col_bit],
            BaseElement::ZERO,
            "precondicion: el bit {} del segmento que empieza en {inicio} debe valer cero",
            largo - 1 - p
        );
        let mut mut_rows = rows.to_vec();
        mut_rows[inicio + p][col_bit] = BaseElement::ONE;
        // El bit de la fila `inicio + p` pesa 2^(q - p) en el acumulador
        // de la fila `inicio + q`.
        let mut peso = BaseElement::ONE;
        for q in p..largo {
            mut_rows[inicio + q][col_acc] += peso;
            peso = peso + peso;
        }
        *disparada = pasos.clone().any(|paso| {
            evaluar(air, &mut_rows, &periodic, paso)
                .iter()
                .any(|v| *v != BaseElement::ZERO)
        });
    }
    salida
}
