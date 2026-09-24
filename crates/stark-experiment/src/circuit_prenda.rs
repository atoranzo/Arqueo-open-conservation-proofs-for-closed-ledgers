//! **RFC-0008 E3: el PROBADOR de la prenda.**
//!
//! El AIR vive en `zk-ssl-air` -el crate que el kit consume sin el probador- y aqui solo esta lo
//! que produce una prueba: la traza y el `Prover`. Lo que el AIR afirma, lo que NO prueba y lo
//! que es testigo estan en la cabecera de `zk_ssl_air::prenda`; aqui no se repiten.
//!
//! **La entrada es lo que el aviso le da al receptor** -`sal`, `importe`, el sobre `X` opaco- y
//! el camino de la foto del latido, MAS la clave de gasto, que es lo que separa a este probador
//! del de E1: alli el enunciado era de ESTADO y el pagador lo producia igual (D-G); aqui es de
//! AUTORIZACION y solo lo produce quien tiene la clave (D-AV).
//!
//! **El `receptor` no es un campo del testigo: se DERIVA de la clave** con
//! [`derive_public_id_wide`]. Un testigo que llevara los dos podria mentirse a si mismo, y el
//! AIR lo rechazaria -- pero el rojo saldria del sitio equivocado.
//!
//! ## El cruce que decide la D-AV, y por que vale
//!
//! El ciclo de la clave del AIR pone la capacidad a CERO y el rate a
//! `[SPEND_KEY_DOMAIN, 0, 0, 0 | clave]`, que es letra por letra lo que `native_merge` compone
//! -- y `crate::merkle::native_merge` **es** `zk_ssl_hash::native_merge`, reexportado desde el
//! §254, no una copia. La traza toma el dominio de `zk-ssl-hash`, como el AIR; el testigo lo
//! cruza contra `derive_public_id_wide`, que toma el suyo de `crate::native`. Las dos
//! declaraciones las ata la regla R2 de `tools/check_dominios.py`, y este testigo las ata
//! ademas por su VALOR compuesto: si alguna vez divergieran, sale rojo aqui.

use winterfell::crypto::hashers::{Blake3_256, Rp64_256};
use winterfell::crypto::DefaultRandomCoin;
use zk_ssl_air::sal::MerkleConSal;
use winterfell::math::{fields::f64::BaseElement, FieldElement};
use winterfell::matrix::ColMatrix;
use winterfell::{
    AuxRandElements, CompositionPoly, CompositionPolyTrace, ConstraintCompositionCoefficients,
    DefaultConstraintCommitment, DefaultConstraintEvaluator, DefaultTraceLde, PartitionOptions,
    ProofOptions, Prover, StarkDomain, TraceInfo, TracePolyTable, TraceTable,
};
use zk_ssl_hash::{DOMINIO_PRENDA, SPEND_KEY_DOMAIN};

use crate::merkle::MerklePath;
use crate::native::derive_public_id_wide;
use crate::rescue_hash::{NUM_ROUNDS, STATE_WIDTH};

pub use zk_ssl_air::prenda::{
    verificar, verificar_contra_cabeza, AfirmacionPrenda, CabezaPrenda, PrendaAir,
    PrendaPublicInputs, ANCHO, C_A, C_B, COL_BIT, COL_IMPORTE, COL_KEY, COL_RECEPTOR, COL_SAL,
    COL_X, CYC_ACC, CYC_MARCA, CYC_RAIZ, NUM_ASERCIONES, NUM_RESTRICCIONES, ROW_ENLACE_IMPORTE,
    ROW_ENLACE_X, ROW_HOJA_LISTA, ROW_MARCA, ROW_RAIZ, TRAZA,
};
pub use zk_ssl_air::{opciones, Digest, CICLO, ESTADO, PROFUNDIDAD};

type Blake3 = Blake3_256<BaseElement>;

const _: () = assert!(NUM_ROUNDS == CICLO - 1);
const _: () = assert!(STATE_WIDTH == ESTADO);

/// **Lo que el prendador aporta.** Sin `receptor` y sin meta: el primero se deriva de la clave y
/// la segunda no entra en el enunciado (D-AY).
#[derive(Clone, Debug)]
pub struct PrendaWitness {
    /// La clave de gasto, TESTIGO. Es lo que este AIR anade sobre el de E1.
    pub clave: Digest,
    pub sal: Digest,
    pub importe: u64,
    pub x: Digest,
    pub camino_pendiente: MerklePath,
}

/// La identidad que la clave del testigo deriva, y que la traza pone en `COL_RECEPTOR`.
pub fn receptor_de(w: &PrendaWitness) -> Digest {
    derive_public_id_wide(w.clave)
}

/// Construye la traza. **No comprueba nada**: los testigos negativos le pasan material a mano
/// para que lo que tumbe la prueba sea el AIR y no esta funcion.
pub fn trazar(w: &PrendaWitness) -> TraceTable<BaseElement> {
    trazar_con(w, w.clave, None, BaseElement::ZERO)
}

/// La traza con tres palancas que solo usan los testigos: la clave que viaja en la traza (la
/// honesta es la del testigo, que es la que deriva el receptor), el `C2` que se hashea en el
/// ciclo de la marca (el honesto es el que sube al arbol) y un limbo del rate del ciclo del
/// importe (el honesto es cero).
fn trazar_con(
    w: &PrendaWitness,
    clave_traza: Digest,
    c2_marca: Option<Digest>,
    relleno: BaseElement,
) -> TraceTable<BaseElement> {
    let cero = BaseElement::ZERO;
    let receptor = receptor_de(w);
    let c_importe = BaseElement::new(w.importe);

    let mut filas: Vec<Vec<BaseElement>> = vec![vec![cero; ANCHO]; TRAZA];
    for fila in filas.iter_mut() {
        fila[COL_RECEPTOR..COL_RECEPTOR + 4].copy_from_slice(&receptor);
        fila[COL_IMPORTE] = c_importe;
        fila[COL_KEY..COL_KEY + 4].copy_from_slice(&clave_traza);
        fila[COL_SAL..COL_SAL + 4].copy_from_slice(&w.sal);
        fila[COL_X..COL_X + 4].copy_from_slice(&w.x);
    }

    let sitio = |estado: &mut [BaseElement; ESTADO], dig: &Digest, herm: &Digest, der: bool| {
        if der {
            estado[4..8].copy_from_slice(herm);
            estado[8..ESTADO].copy_from_slice(dig);
        } else {
            estado[4..8].copy_from_slice(dig);
            estado[8..ESTADO].copy_from_slice(herm);
        }
    };

    // Carril A: el compromiso y su subida. Carril B: el CICLO DE LA CLAVE, desde la fila 0.
    let mut a = [cero; ESTADO];
    a[4..8].copy_from_slice(&receptor);
    a[8..ESTADO].copy_from_slice(&w.sal);
    let mut b = [cero; ESTADO];
    b[4] = BaseElement::new(SPEND_KEY_DOMAIN);
    b[8..ESTADO].copy_from_slice(&clave_traza);
    filas[0][C_A..C_A + ESTADO].copy_from_slice(&a);
    filas[0][C_B..C_B + ESTADO].copy_from_slice(&b);

    // El carril B hashea SOLO en su ciclo 0 (la clave) y en el CYC_MARCA (la marca).
    let ciclo_marca = CYC_MARCA * CICLO..(CYC_MARCA + 1) * CICLO;
    let hashea_b = |r: usize| r < CICLO || ciclo_marca.contains(&r);

    for r in 0..ROW_RAIZ {
        let pos = r % CICLO;
        if pos < NUM_ROUNDS {
            Rp64_256::apply_round(&mut a, pos);
            if hashea_b(r) {
                Rp64_256::apply_round(&mut b, pos);
            }
        } else {
            let da: Digest = [a[4], a[5], a[6], a[7]];
            a = [cero; ESTADO];
            b = [cero; ESTADO];
            match r {
                ROW_ENLACE_IMPORTE => {
                    a[4..8].copy_from_slice(&da);
                    a[8] = c_importe;
                    a[9] = relleno;
                }
                ROW_ENLACE_X => {
                    a[4..8].copy_from_slice(&da);
                    a[8..ESTADO].copy_from_slice(&w.x);
                }
                // `C2` esta listo: entra al arbol Y siembra el ciclo de la marca, en la MISMA
                // fila. No hay dos `C2`: el que sube y el que se hashea son el mismo.
                ROW_HOJA_LISTA => {
                    let cp = &w.camino_pendiente;
                    sitio(&mut a, &da, &cp.siblings[0], cp.is_right[0]);
                    b[0] = BaseElement::new(DOMINIO_PRENDA);
                    b[4..8].copy_from_slice(&c2_marca.unwrap_or(da));
                }
                _ => {
                    let sig = (r + 1) / CICLO;
                    if (CYC_ACC..CYC_RAIZ).contains(&sig) {
                        let n = sig - CYC_ACC;
                        let cp = &w.camino_pendiente;
                        sitio(&mut a, &da, &cp.siblings[n], cp.is_right[n]);
                    }
                }
            }
        }
        filas[r + 1][C_A..C_A + ESTADO].copy_from_slice(&a);
        filas[r + 1][C_B..C_B + ESTADO].copy_from_slice(&b);
    }
    debug_assert_eq!(ROW_HOJA_LISTA, CYC_ACC * CICLO - 1);
    debug_assert_eq!(ROW_MARCA, (CYC_MARCA + 1) * CICLO - 1);

    for nivel in 0..PROFUNDIDAD {
        let bit = if w.camino_pendiente.is_right[nivel] { BaseElement::ONE } else { cero };
        for p in 0..CICLO {
            filas[(CYC_ACC + nivel) * CICLO + p][COL_BIT] = bit;
        }
    }

    let mut traza = TraceTable::new(ANCHO, TRAZA);
    traza.fill(
        |s| s.copy_from_slice(&filas[0]),
        |paso, s| s.copy_from_slice(&filas[paso + 1]),
    );
    traza
}

pub struct PrendaProver {
    options: ProofOptions,
}

impl PrendaProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

/// **Prueba con las opciones de la casa**, las UNICAS que el juez acepta.
pub fn probar(traza: TraceTable<BaseElement>) -> Result<(Vec<u8>, PrendaPublicInputs), String> {
    probar_con(traza, opciones())
}

/// Prueba con OTRAS opciones: solo para el testigo de que el juez no las acepta.
pub fn probar_con(
    traza: TraceTable<BaseElement>,
    opts: ProofOptions,
) -> Result<(Vec<u8>, PrendaPublicInputs), String> {
    let prover = PrendaProver::new(opts);
    let pi = prover.get_pub_inputs(&traza);
    let prueba = prover.prove(traza).map_err(|e| format!("{e:?}"))?;
    Ok((prueba.to_bytes(), pi))
}

impl Prover for PrendaProver {
    type BaseField = BaseElement;
    type Air = PrendaAir;
    type Trace = TraceTable<BaseElement>;
    type HashFn = Blake3;
    type VC = MerkleConSal<Blake3>;
    type RandomCoin = DefaultRandomCoin<Blake3>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<E, Self::HashFn, Self::VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, Self::HashFn, Self::VC>;

    /// **Las entradas publicas DERIVADAS de la traza.** Lo que impide declarar otras son las
    /// aserciones del AIR, que el verificador comprueba contra las que EL declara.
    fn get_pub_inputs(&self, trace: &Self::Trace) -> PrendaPublicInputs {
        let dig = |base: usize, fila: usize| -> Digest {
            [
                trace.get(base, fila),
                trace.get(base + 1, fila),
                trace.get(base + 2, fila),
                trace.get(base + 3, fila),
            ]
        };
        PrendaPublicInputs {
            pending_root: dig(C_A + 4, ROW_RAIZ),
            receptor: dig(COL_RECEPTOR, 0),
            marca: dig(C_B + 4, ROW_MARCA),
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }

    /// E3b2-M3: encendido, sembrado de la entropia del sistema (D-Z, D-AE).
    fn ocultacion(&self) -> Option<winter_prover::Ocultacion> {
        Some(crate::ocultacion_encendida())
    }

    fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
        partition_option: PartitionOptions,
    ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
        DefaultTraceLde::new(trace_info, main_trace, domain, partition_option)
    }

    fn new_evaluator<'a, E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        air: &'a Self::Air,
        aux_rand_elements: Option<AuxRandElements<E>>,
        composition_coefficients: ConstraintCompositionCoefficients<E>,
    ) -> Self::ConstraintEvaluator<'a, E> {
        DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
    }

    fn build_constraint_commitment<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        composition_poly_trace: CompositionPolyTrace<E>,
        num_constraint_composition_columns: usize,
        domain: &StarkDomain<Self::BaseField>,
        partition_options: PartitionOptions,
    ) -> (Self::ConstraintCommitment<E>, CompositionPoly<E>) {
        DefaultConstraintCommitment::new(
            composition_poly_trace,
            num_constraint_composition_columns,
            domain,
            partition_options,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::native_merge;
    use crate::native::native_climb;
    use winterfell::Air;
    use zk_ssl_hash::marca_prenda;

    const P_IMPORTE: u64 = 250_000;

    fn dg(k: u64) -> Digest {
        [
            BaseElement::new(k),
            BaseElement::new(k.wrapping_mul(5) + 1),
            BaseElement::new(k.wrapping_mul(13) + 2),
            BaseElement::new(k.wrapping_mul(17) + 3),
        ]
    }

    fn embebe(v: u64) -> Digest {
        [BaseElement::new(v), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
    }

    /// La hoja v2 compuesta en nativo, como `pending_commitment_v2` de la capa.
    fn hoja_v2(receptor: Digest, sal: Digest, importe: u64, x: Digest) -> Digest {
        native_merge(native_merge(native_merge(receptor, sal), embebe(importe)), x)
    }

    /// El escenario: el pendiente vive en una posicion con direcciones MIXTAS y hermanos
    /// distintos por nivel -con todas iguales la traza degenera-. `X` es la de los tres
    /// productores del arbol: el `public_id` de quien paga y `delta = 96`.
    struct Escenario {
        w: PrendaWitness,
        receptor: Digest,
        pending_root: Digest,
        hoja: Digest,
        marca: Digest,
    }

    fn escenario() -> Escenario {
        let clave = dg(0xC1A7E);
        let sal = dg(0x5A17);
        let pagador = dg(0xA11CE);
        let x = native_merge(pagador, embebe(96));
        let receptor = derive_public_id_wide(clave);
        let hoja = hoja_v2(receptor, sal, P_IMPORTE, x);

        let mut bits = vec![false];
        let mut hermanos = vec![[BaseElement::ZERO; 4]];
        for nivel in 1..PROFUNDIDAD {
            bits.push(nivel % 3 == 0);
            hermanos.push(dg(100 + nivel as u64));
        }
        let camino_pendiente = MerklePath { siblings: hermanos, is_right: bits };
        let pending_root = native_climb(hoja, &camino_pendiente);

        Escenario {
            w: PrendaWitness { clave, sal, importe: P_IMPORTE, x, camino_pendiente },
            receptor,
            pending_root,
            hoja,
            marca: marca_prenda(hoja),
        }
    }

    fn pi(e: &Escenario, receptor: Digest, marca: Digest) -> PrendaPublicInputs {
        PrendaPublicInputs { pending_root: e.pending_root, receptor, marca }
    }

    /// Produce y JUZGA con el juez del kit: lo que decide es `verificar`, no un `verify` propio.
    fn juzga(traza: TraceTable<BaseElement>, declarado: &PrendaPublicInputs) -> bool {
        let prover = PrendaProver::new(opciones());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(traza)));
        match r {
            Err(_) | Ok(Err(_)) => false,
            Ok(Ok(prueba)) => verificar(&prueba.to_bytes(), declarado).is_ok(),
        }
    }

    /// **El positivo**: quien tiene la clave prueba <<bajo esta raiz hay una hoja a mi nombre y
    /// esta es su marca>>.
    #[test]
    fn el_positivo_verifica() {
        let e = escenario();
        let declarado = pi(&e, e.receptor, e.marca);
        assert!(juzga(trazar(&e.w), &declarado));
    }

    /// **LA PUERTA DE LA D-AV, y es la razon de ser de este corte.** La fila 7 del carril B es
    /// `derive_public_id_wide(clave)` recomputado en NATIVO -con el dominio de `crate::native`,
    /// mientras la traza usa el de `zk-ssl-hash`- y es el receptor; la 23 del carril A es la
    /// hoja v2 nativa; y la 31 del B es su `marca_prenda`. Si la deduccion del diseno fuera
    /// falsa, muere aqui.
    #[test]
    fn la_clave_deriva_el_receptor_y_la_marca_es_la_nativa() {
        let e = escenario();
        let traza = trazar(&e.w);
        let nativa = derive_public_id_wide(e.w.clave);
        for i in 0..4 {
            assert_eq!(traza.get(C_B + 4 + i, ROW_ENLACE_IMPORTE), nativa[i], "identidad, fila 7");
            assert_eq!(traza.get(COL_RECEPTOR + i, 0), nativa[i], "el receptor de la traza");
            assert_eq!(traza.get(C_A + 4 + i, ROW_HOJA_LISTA), e.hoja[i], "la hoja v2");
            assert_eq!(traza.get(C_B + 4 + i, ROW_MARCA), e.marca[i], "la marca");
        }
        let derivadas = PrendaProver::new(opciones()).get_pub_inputs(&traza);
        assert_eq!(derivadas, pi(&e, e.receptor, e.marca));
    }

    /// **Negativo 1: otra clave en la traza no verifica.** El receptor declarado y el del arbol
    /// son los honestos; lo unico que cambia es la clave que sube por el carril B. Cae por
    /// `C_PK_CHECK`, que es la titularidad entera.
    #[test]
    fn otra_clave_no_verifica() {
        let e = escenario();
        let declarado = pi(&e, e.receptor, e.marca);
        let traza = trazar_con(&e.w, dg(0xBAD), None, BaseElement::ZERO);
        assert!(!juzga(traza, &declarado), "CRITICO: la titularidad no ata");
    }

    /// **Negativo 2: otro receptor declarado no verifica** (las cuatro aserciones del receptor).
    #[test]
    fn otro_receptor_declarado_no_verifica() {
        let e = escenario();
        let declarado = pi(&e, dg(0x1337), e.marca);
        assert!(!juzga(trazar(&e.w), &declarado), "CRITICO: el receptor no ata");
    }

    /// **Negativo 3: otra marca declarada no verifica** (las cuatro aserciones de la marca).
    #[test]
    fn otra_marca_declarada_no_verifica() {
        let e = escenario();
        let declarado = pi(&e, e.receptor, dg(0xBEEF));
        assert!(!juzga(trazar(&e.w), &declarado), "CRITICO: la marca no ata");
    }

    /// **Negativo 4 (D-AW), y es el que sostiene <<no hay dos `C2`>>:** con la marca hashada
    /// desde OTRO `C2`, el arbol sigue siendo el honesto y la marca declarada es la que esa
    /// traza alcanza -- y aun asi no verifica, porque `C_MARCA_IN` lee el digest del carril A en
    /// la misma fila.
    #[test]
    fn la_marca_es_la_del_c2_que_sube_al_arbol() {
        let e = escenario();
        let otro = dg(0xC02);
        let traza = trazar_con(&e.w, e.w.clave, Some(otro), BaseElement::ZERO);
        let derivadas = PrendaProver::new(opciones()).get_pub_inputs(&traza);
        assert_eq!(derivadas.pending_root, e.pending_root, "el arbol no tenia que moverse");
        assert_eq!(derivadas.marca, marca_prenda(otro), "la marca es la del C2 metido a mano");
        assert_ne!(derivadas.marca, e.marca);
        assert!(!juzga(traza, &derivadas), "CRITICO: la marca no esta atada al C2 del arbol");
    }

    /// **Negativo 5 (asiento 489): un relleno distinto de cero en el ciclo del importe no
    /// verifica**, aunque la raiz declarada sea la que esa traza alcanza. Lo unico que cae es el
    /// atado del rate ENTERO.
    #[test]
    fn un_relleno_en_el_ciclo_del_importe_no_verifica() {
        let e = escenario();
        let traza = trazar_con(&e.w, e.w.clave, None, BaseElement::ONE);
        let declarado = PrendaProver::new(opciones()).get_pub_inputs(&traza);
        assert_ne!(declarado.pending_root, e.pending_root, "el relleno tenia que mover la hoja");
        assert!(!juzga(traza, &declarado), "CRITICO: el rate del importe no esta atado");
    }

    /// **Negativo 6 (D-AW): ni la clave ni `C2` entran en el transcripto.** Las DOCE entradas
    /// publicas no contienen un limbo de la clave, de la sal, de `X` ni de la hoja.
    #[test]
    fn ni_la_clave_ni_c2_estan_en_las_entradas_publicas() {
        let e = escenario();
        let declarado = pi(&e, e.receptor, e.marca);
        let v = winterfell::math::ToElements::to_elements(&declarado);
        assert_eq!(v.len(), 12);
        for oculto in [e.w.clave, e.w.sal, e.w.x, e.hoja] {
            assert!(oculto.iter().all(|z| !v.contains(z)), "un limbo del testigo, publicado");
        }
        let (bytes, derivado) = probar(trazar(&e.w)).expect("probar");
        assert_eq!(derivado, declarado);
        assert!(verificar(&bytes, &declarado).is_ok());
    }

    /// **D-AV: la cabeza fija la raiz.** Con la del escenario la prueba se enlaza y el juez
    /// devuelve SU enunciado; con otra, no.
    #[test]
    fn el_enlace_toma_la_raiz_de_la_cabeza() {
        let e = escenario();
        let (bytes, declarado) = probar(trazar(&e.w)).expect("probar");
        let af = AfirmacionPrenda { receptor: e.receptor, marca: e.marca };
        let buena = CabezaPrenda { pending_root: e.pending_root };
        assert_eq!(verificar_contra_cabeza(&bytes, &af, &buena), Ok(declarado));
        let mut otra = e.pending_root;
        otra[0] += BaseElement::ONE;
        let mala = CabezaPrenda { pending_root: otra };
        assert!(verificar_contra_cabeza(&bytes, &af, &mala).is_err(), "enlazo con otra raiz");
    }

    /// **PRUEBA POR MUTACION: ninguna restriccion esta vacia**, en TODAS las filas.
    #[test]
    fn ninguna_restriccion_esta_vacia() {
        use crate::mutation::{buscar_vacias, rows_of};
        let e = escenario();
        let traza = trazar(&e.w);
        let filas = rows_of(&traza, ANCHO, TRAZA);
        let air = PrendaAir::new(
            TraceInfo::new(ANCHO, TRAZA),
            pi(&e, e.receptor, e.marca),
            opciones(),
        );
        let informe = buscar_vacias(&air, &filas, 1);
        assert!(
            informe.nunca_disparadas.is_empty(),
            "restricciones que NINGUNA perturbacion activa (de {} totales, {} celdas): {:?}",
            informe.total,
            informe.celdas,
            informe.nunca_disparadas
        );
    }

    /// El juez acepta UN conjunto de opciones y solo uno.
    #[test]
    fn otras_opciones_no_se_aceptan() {
        use winterfell::{BatchingMethod, FieldExtension};
        let e = escenario();
        let otras = ProofOptions::new(
            42,
            16,
            0,
            FieldExtension::Quadratic,
            8,
            31,
            BatchingMethod::Linear,
            BatchingMethod::Linear,
        );
        let (bytes, declarado) = probar_con(trazar(&e.w), otras).expect("probar");
        assert_eq!(declarado.pending_root, e.pending_root, "el escenario no es el que se cree");
        assert!(verificar(&bytes, &declarado).is_err(), "el juez acepto otras opciones");
    }

    /// **INSTRUMENTO, no comprobacion** (RFC-0008 E3). En release, a mano:
    /// `cargo test --release -p stark-experiment instrumento_del_air_de_e3 -- --ignored
    /// --nocapture`. La geometria del AIR y el positivo con las opciones de la casa: bytes,
    /// probar y verificar. La D-AX predijo 213-218 ms por comparacion con E2; con 106
    /// restricciones la prediccion corregida es el orden de E1, 127-137 ms.
    #[test]
    #[ignore = "instrumento de medida, no comprobacion: correr a mano, en release"]
    fn instrumento_del_air_de_e3() {
        use std::time::Instant;
        let e = escenario();
        println!(
            "E3-AIR| geometria: {ANCHO} columnas x {TRAZA} filas . {NUM_RESTRICCIONES} \
             restricciones . {NUM_ASERCIONES} aserciones . raiz en la fila {ROW_RAIZ}"
        );
        let declarado = pi(&e, e.receptor, e.marca);
        for corrida in 0..3 {
            let traza = trazar(&e.w);
            let t = Instant::now();
            let (bytes, _) = probar(traza).expect("probar");
            let probar_s = t.elapsed().as_secs_f64();
            let t = Instant::now();
            let ok = verificar(&bytes, &declarado).is_ok();
            let verificar_ms = t.elapsed().as_secs_f64() * 1000.0;
            println!(
                "E3-AIR| corrida {corrida} . prueba {} B . probar {probar_s:.2} s . verificar \
                 {verificar_ms:.1} ms . verifica {ok} . prediccion 127-137 ms",
                bytes.len()
            );
        }
    }
}
