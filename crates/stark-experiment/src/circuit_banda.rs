//! **RFC-0007 E5, corte 4a: el PROBADOR de la banda del saldo.**
//!
//! El AIR vive en `zk-ssl-air` -el crate que el kit consume sin el probador- y aqui solo esta
//! lo que produce una prueba: la traza y el `Prover`. Lo que el AIR afirma, lo que NO prueba y
//! su confianza residual estan en la cabecera de `zk_ssl_air::banda`; aqui no se repiten.
//!
//! **La entrada es lo que el OPERADOR tiene**, y esa es la diferencia entera con
//! `circuit_audit`: el saldo, el nonce, el camino, el salt que la capa LEE del almacen
//! (`accounts.rs`, `stored_leaf_salt`) y la identidad de la cuenta. **Ninguna clave.** Por eso
//! el escenario de los testigos se construye sin `spend_key`: si hiciera falta una, el rechazo
//! no seria producible y este corte no existiria.
//!
//! Los testigos de punta a punta viven AQUI y no en el AIR: `zk-ssl-air` no compila al
//! probador, asi que alli no se puede producir una prueba. Es el reparto del S463.

use winterfell::crypto::hashers::{Blake3_256, Rp64_256};
use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
use winterfell::math::{fields::f64::BaseElement, FieldElement};
use winterfell::matrix::ColMatrix;
use winterfell::{
    AuxRandElements, CompositionPoly, CompositionPolyTrace, ConstraintCompositionCoefficients,
    DefaultConstraintCommitment, DefaultConstraintEvaluator, DefaultTraceLde, PartitionOptions,
    ProofOptions, Prover, StarkDomain, TraceInfo, TracePolyTable, TraceTable,
};

use crate::merkle::MerklePath;
use crate::rescue_hash::{NUM_ROUNDS, STATE_WIDTH};

pub use zk_ssl_air::banda::{
    comprobar_enunciado, verificar, BandaAir, BandaPublicInputs, ANCHO, COL_BAL, COL_BIT, COL_ID,
    COL_LEAF_SALT, COL_LOWER, COL_NONCE, COL_SACC, COL_SBIT, COL_UPPER, CYC_ACC, CYC_RAIZ,
    LARGO_SEGMENTO, MAX_VALOR, NUM_ASERCIONES, NUM_RESTRICCIONES, ROW_ENLACE_HOJA,
    ROW_ENLACE_SALT, ROW_HOJA_LISTA, ROW_RAIZ, SEGMENTOS, TRAZA,
};
pub use zk_ssl_air::{opciones, Digest, CICLO, ESTADO, PROFUNDIDAD};

type Blake3 = Blake3_256<BaseElement>;

// Los dos crates cuentan la misma ronda y el mismo estado. Si algun dia dejaran de contarlos
// igual, esto no compila -que es lo que se quiere-.
const _: () = assert!(NUM_ROUNDS == CICLO - 1);
const _: () = assert!(STATE_WIDTH == ESTADO);

fn bits_be(valor: u64) -> Vec<bool> {
    (0..LARGO_SEGMENTO)
        .map(|p| (valor >> (LARGO_SEGMENTO - 1 - p)) & 1 == 1)
        .collect()
}

/// **Lo que el operador aporta para probar la banda.** Sin clave: la hoja se compone con la
/// identidad, el saldo, el nonce y el salt LEIDO, y sube por el camino hasta la raiz que la
/// cabeza firma.
#[derive(Clone, Debug)]
pub struct BandaWitness {
    pub public_id: Digest,
    pub balance: u64,
    pub nonce: BaseElement,
    pub leaf_salt: Digest,
    pub path: MerklePath,
}

/// Construye la traza. **No comprueba nada**: los testigos negativos le pasan material a mano
/// para que lo que tumbe la prueba sea el AIR y no esta funcion (el molde de `circuit_edad`).
pub fn trazar(w: &BandaWitness, lower: u64, upper: u64) -> TraceTable<BaseElement> {
    let cero = BaseElement::ZERO;
    let c_bal = BaseElement::new(w.balance);
    let c_lower = BaseElement::new(lower);
    let c_upper = BaseElement::new(upper);

    let mut filas: Vec<Vec<BaseElement>> = vec![vec![cero; ANCHO]; TRAZA];

    for fila in filas.iter_mut() {
        fila[COL_ID..COL_ID + 4].copy_from_slice(&w.public_id);
        fila[COL_BAL] = c_bal;
        fila[COL_NONCE] = w.nonce;
        fila[COL_LOWER] = c_lower;
        fila[COL_UPPER] = c_upper;
        fila[COL_LEAF_SALT..COL_LEAF_SALT + 4].copy_from_slice(&w.leaf_salt);
    }

    // Rangos: saldo, saldo - inferior, superior - saldo. Si el saldo estuviera fuera de la
    // banda, alguna resta daria la vuelta en el campo y su bit mas significativo seria uno, que
    // es lo que `C_FIRST_S` rechaza.
    let segmentos = [
        c_bal.as_int(),
        (c_bal - c_lower).as_int(),
        (c_upper - c_bal).as_int(),
    ];
    for (seg, valor) in segmentos.iter().enumerate() {
        let bits = bits_be(*valor);
        let mut acc = cero;
        for p in 0..LARGO_SEGMENTO {
            let r = seg * LARGO_SEGMENTO + p;
            let bit = if bits[p] { BaseElement::ONE } else { cero };
            acc = if p == 0 { bit } else { acc + acc + bit };
            filas[r][COL_SBIT] = bit;
            filas[r][COL_SACC] = acc;
        }
    }

    let sitio = |estado: &mut [BaseElement; ESTADO], digest: &Digest, nivel: usize| {
        debug_assert!(nivel < PROFUNDIDAD, "sitio: nivel {} sobre camino de {}", nivel, PROFUNDIDAD);
        if w.path.is_right[nivel] {
            estado[4..8].copy_from_slice(&w.path.siblings[nivel]);
            estado[8..ESTADO].copy_from_slice(digest);
        } else {
            estado[4..8].copy_from_slice(digest);
            estado[8..ESTADO].copy_from_slice(&w.path.siblings[nivel]);
        }
    };

    let mut estado = [cero; ESTADO];
    estado[4..8].copy_from_slice(&w.public_id);
    estado[8] = c_bal;
    filas[0][..ESTADO].copy_from_slice(&estado);

    // La tuberia acaba en la raiz: sin ciclo de titularidad no hay nada detras.
    for r in 0..ROW_RAIZ {
        let pos = r % CICLO;
        if pos < NUM_ROUNDS {
            Rp64_256::apply_round(&mut estado, pos);
        } else {
            let digest: Digest = [estado[4], estado[5], estado[6], estado[7]];
            estado = [cero; ESTADO];
            match r {
                ROW_ENLACE_HOJA => {
                    estado[4..8].copy_from_slice(&digest);
                    estado[8] = w.nonce;
                }
                ROW_ENLACE_SALT => {
                    // La envoltura: digest arrastrado y los CUATRO limbos del rate al salt.
                    estado[4..8].copy_from_slice(&digest);
                    estado[8..ESTADO].copy_from_slice(&w.leaf_salt);
                }
                ROW_HOJA_LISTA => sitio(&mut estado, &digest, 0),
                _ => {
                    let siguiente_ciclo = (r + 1) / CICLO;
                    if (CYC_ACC..CYC_RAIZ).contains(&siguiente_ciclo) {
                        sitio(&mut estado, &digest, siguiente_ciclo - CYC_ACC);
                    }
                }
            }
        }
        filas[r + 1][..ESTADO].copy_from_slice(&estado);
    }

    for nivel in 0..PROFUNDIDAD {
        let bit = if w.path.is_right[nivel] {
            BaseElement::ONE
        } else {
            cero
        };
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

pub struct BandaProver {
    options: ProofOptions,
}

impl BandaProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for BandaProver {
    type BaseField = BaseElement;
    type Air = BandaAir;
    type Trace = TraceTable<BaseElement>;
    type HashFn = Blake3;
    type VC = MerkleTree<Blake3>;
    type RandomCoin = DefaultRandomCoin<Blake3>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<E, Self::HashFn, Self::VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, Self::HashFn, Self::VC>;

    /// **Las entradas publicas DERIVADAS de la traza.** La identidad sale de `COL_ID` en la
    /// fila 0, igual que en el hermano; lo que impide que un probador declare una identidad
    /// que no es la suya son las CUATRO aserciones del AIR (D-2b), que el verificador comprueba
    /// contra el `public_id` que EL declara.
    fn get_pub_inputs(&self, trace: &Self::Trace) -> BandaPublicInputs {
        BandaPublicInputs {
            root: [
                trace.get(4, ROW_RAIZ),
                trace.get(5, ROW_RAIZ),
                trace.get(6, ROW_RAIZ),
                trace.get(7, ROW_RAIZ),
            ],
            public_id: [
                trace.get(COL_ID, 0),
                trace.get(COL_ID + 1, 0),
                trace.get(COL_ID + 2, 0),
                trace.get(COL_ID + 3, 0),
            ],
            lower: trace.get(COL_LOWER, 0),
            upper: trace.get(COL_UPPER, 0),
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
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
    // `BandaAir::new` la da el trait, no un `impl` propio: sin el trait en el ambito no
    // resuelve. Va AQUI y no arriba porque la produccion no lo necesita, y arriba seria un
    // import sin usar en el build sin tests.
    use winterfell::Air;
    use crate::merkle::native_merge;
    use crate::native::{native_leaf, native_leaf_salted, native_climb};

    /// El escenario del OPERADOR: **sin clave**. La identidad y el salt son datos que la capa
    /// tiene delante (`accounts.rs`), no cosas que derive de un secreto.
    fn escenario(balance: u64) -> (BandaWitness, Digest) {
        let mut vacios = vec![[BaseElement::ZERO; 4]];
        for k in 1..=PROFUNDIDAD {
            let previo = vacios[k - 1];
            vacios.push(native_merge(previo, previo));
        }
        let public_id = [
            BaseElement::new(0xA11CE),
            BaseElement::new(0xA0D17),
            BaseElement::new(0x0DDBA11),
            BaseElement::new(0x5EA51DE),
        ];
        // Salt LEIDO del almacen, no derivado: cuatro elementos cualesquiera y distintos.
        let leaf_salt = [
            BaseElement::new(0x5A17_0001),
            BaseElement::new(0x5A17_0002),
            BaseElement::new(0x5A17_0003),
            BaseElement::new(0x5A17_0004),
        ];
        let nonce = BaseElement::ZERO;

        let mut siblings = Vec::with_capacity(PROFUNDIDAD);
        let mut is_right = Vec::with_capacity(PROFUNDIDAD);
        for nivel in 0..PROFUNDIDAD {
            siblings.push(vacios[nivel]);
            // Direcciones MIXTAS: con todas iguales la traza degenera y las restricciones de
            // grado 2 colapsan a grado 1.
            is_right.push(nivel % 3 == 0);
        }
        let path = MerklePath { siblings, is_right };
        let root = native_climb(
            native_leaf_salted(public_id, BaseElement::new(balance), nonce, leaf_salt),
            &path,
        );
        (
            BandaWitness { public_id, balance, nonce, leaf_salt, path },
            root,
        )
    }

    fn pi(root: Digest, id: Digest, lower: u64, upper: u64) -> BandaPublicInputs {
        BandaPublicInputs {
            root,
            public_id: id,
            lower: BaseElement::new(lower),
            upper: BaseElement::new(upper),
        }
    }

    /// Produce y JUZGA con el juez del kit: lo que decide es `zk_ssl_air::banda::verificar`,
    /// no un `verify` propio, para que el testigo ejercite la puerta que corre el tercero.
    fn corre(w: &BandaWitness, lower: u64, upper: u64, declarado: &BandaPublicInputs) -> bool {
        let traza = trazar(w, lower, upper);
        let prover = BandaProver::new(opciones());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(traza)));
        match r {
            Err(_) | Ok(Err(_)) => false,
            Ok(Ok(prueba)) => verificar(&prueba.to_bytes(), declarado).is_ok(),
        }
    }

    /// **El positivo.** Saldo dentro de la banda que un rechazo declara: `lower = 0` y
    /// `upper = pedido - 1`.
    #[test]
    fn el_positivo_verifica() {
        let (w, root) = escenario(1_000_000);
        let declarado = pi(root, w.public_id, 0, 4_999_999);
        assert!(corre(&w, 0, 4_999_999, &declarado));
    }

    /// **El techo, por los DOS lados.** Con `upper == saldo` la banda contiene al saldo y
    /// verifica; con `upper == saldo - 1` la resta da la vuelta en el campo y `C_FIRST_S`
    /// rechaza. Un techo probado por un solo lado no distingue una banda de una tautologia.
    #[test]
    fn el_techo_se_prueba_por_los_dos_lados() {
        let (w, root) = escenario(1_000_000);
        let justo = pi(root, w.public_id, 0, w.balance);
        assert!(corre(&w, 0, w.balance, &justo), "el saldo en el techo debe verificar");
        let pasado = pi(root, w.public_id, 0, w.balance - 1);
        assert!(
            !corre(&w, 0, w.balance - 1, &pasado),
            "CRITICO: un saldo por encima del techo no puede probar la banda"
        );
    }

    /// **NADIE PRUEBA LA BANDA DE OTRO** -- el testigo que hereda
    /// `third_party_cannot_disclose_someone_elses_balance` cuando el ciclo de titularidad se
    /// va. Un tercero conoce identidad, saldo, nonce, salt y camino de una cuenta ajena y
    /// construye la traza con la identidad de la VICTIMA para que la raiz cuadre; declara la
    /// SUYA. Cae por las CUATRO aserciones de la D-2b, y solo por ellas: sin esas cuatro, el
    /// AIR no ataria ninguna cuenta y esto verificaria.
    #[test]
    fn un_tercero_no_prueba_la_banda_de_otro() {
        let (victima, root) = escenario(1_000_000);
        let otro = [
            BaseElement::new(0x1337),
            BaseElement::new(0xBADC0DE),
            BaseElement::new(0x0DDBA11),
            BaseElement::new(0x1CEB00DA),
        ];
        let declarado = pi(root, otro, 0, 4_999_999);
        assert!(
            !corre(&victima, 0, 4_999_999, &declarado),
            "CRITICO: la identidad declarada no ata la traza"
        );
    }

    /// Probar contra una raiz que no es la del estado comprometido no verifica: impide traer
    /// un estado antiguo favorable.
    #[test]
    fn otra_raiz_no_verifica() {
        let (w, _root) = escenario(1_000_000);
        let ajena: Digest = [BaseElement::new(999); 4];
        let declarado = pi(ajena, w.public_id, 0, 4_999_999);
        assert!(!corre(&w, 0, 4_999_999, &declarado));
    }

    /// La hoja SIN el merge del salt no sube a la raiz comprometida. **Cae por la MISMA puerta
    /// que `otra_raiz_no_verifica`** -la asercion de la raiz-, medido en la simulacion del
    /// corte: se conserva porque muta otra cosa y es barato, pero **son UNA regla y no dos**, y
    /// ni el asiento ni un catalogo futuro pueden contarlas como dos.
    #[test]
    fn la_hoja_sin_el_salt_no_sube_a_la_raiz() {
        let (w, _root) = escenario(1_000_000);
        let sin_sal = native_climb(
            native_leaf(w.public_id, BaseElement::new(w.balance), w.nonce),
            &w.path,
        );
        let declarado = pi(sin_sal, w.public_id, 0, 4_999_999);
        assert!(!corre(&w, 0, 4_999_999, &declarado));
    }

    /// **Puntos de referencia: separa <<la traza esta mal>> de <<las restricciones estan
    /// mal>>.** La hoja envuelta y la raiz de la traza son las nativas, y las entradas
    /// DERIVADAS de la traza son las DECLARADAS en todos sus campos. No produce prueba.
    #[test]
    fn los_puntos_de_referencia_espejan_el_nativo() {
        let (w, root) = escenario(1_000_000);
        let traza = trazar(&w, 0, 4_999_999);
        let sin_sal = native_leaf(w.public_id, BaseElement::new(w.balance), w.nonce);
        let con_sal =
            native_leaf_salted(w.public_id, BaseElement::new(w.balance), w.nonce, w.leaf_salt);
        for i in 0..4 {
            assert_eq!(traza.get(4 + i, ROW_ENLACE_SALT), sin_sal[i], "hoja sin envolver");
            assert_eq!(traza.get(4 + i, ROW_HOJA_LISTA), con_sal[i], "hoja envuelta");
            assert_eq!(traza.get(4 + i, ROW_RAIZ), root[i], "raiz, elemento {i}");
        }
        let derivadas = BandaProver::new(opciones()).get_pub_inputs(&traza);
        assert_eq!(derivadas, pi(root, w.public_id, 0, 4_999_999));
    }

    /// **PRUEBA POR MUTACION: ninguna restriccion esta vacia.** Si ninguna perturbacion de una
    /// celda hace que una restriccion se vuelva no nula, esa restriccion no impone nada y
    /// ningun test normal lo detecta. Se prueban TODAS las filas: con muestreo, una activa en
    /// una sola fila aparece como vacia sin serlo. Un resultado limpio no dice que el AIR sea
    /// correcto: dice que no tiene ESTE fallo.
    #[test]
    fn ninguna_restriccion_esta_vacia() {
        use crate::mutation::{buscar_vacias, rows_of};

        // La banda tiene que CONTENER el saldo, o la traza de referencia declara algo falso y
        // el informe no vale (la leccion que el hermano dejo escrita).
        let (w, root) = escenario(1_000_000);
        let traza = trazar(&w, 0, 4_999_999);
        let filas = rows_of(&traza, ANCHO, TRAZA);
        let air = BandaAir::new(
            TraceInfo::new(ANCHO, TRAZA),
            pi(root, w.public_id, 0, 4_999_999),
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
}
