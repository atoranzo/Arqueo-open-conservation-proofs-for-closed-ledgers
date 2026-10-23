//! §723, la ventana del saldo privado en la banda del kit. Un sobre de rechazo
//! `InsufficientBalance` que arma un operador con su propia clave y una raiz de cuentas hecha a
//! mano: una cabeza v5 FIRMADA con una clave XMSS de test de semilla fija, cuyo `accountsRoot`
//! sube de UNA hoja, y la prueba de banda de esa hoja con el pedido 1001 (la banda `[0, 1000]`).
//! El kit verifica la firma con la clave que trae la cabeza.
//!
//! Los casos:
//! - `ventana-v1`: la hoja tiene el saldo `2^63 - 2^32 + 1002`, de la ventana [2^62, 2^63), que
//!   ninguna capa de la casa firma (el tope de la capa); la prueba es de la banda v1, con el
//!   probador de la casa, que la produce sin queja porque con segmentos de 63 bits la traza cumple.
//!   Con el kit de antes, VERDE.
//! - `ventana-v2`: la misma hoja, con la banda v2 y un probador malicioso que oculta y no comprueba
//!   la traza (el de `stark-experiment/src/sin_comprobar.rs`, copiado): recorta el cociente a lo
//!   que cabe y prueba.
//! - `control`: la misma cabeza y la misma forma con una hoja honesta, saldo 1000, y la banda v2.
//!
//! Y los sobres de COMPLETITUD (`spec/vectors/completitud/`) cuya resolucion es un rechazo
//! `InsufficientBalance` con la banda de la v1 sobre una hoja HONESTA (saldo 1000, pedido 1001): la
//! que da cualquier nodo de antes de la v2. El mismo operador firma la cabeza del rechazo (indice 0)
//! y el cierre v6 (indice 1), cuya `recepRoot` sube de UNA hoja de recibo (rx 1, era 1, n 1440, Q
//! 0); el rechazo lleva su `data.recepcion`. Fijan que el cuarto estado de la banda se dice DESPUES
//! de las comprobaciones de cada brazo:
//! - `completitud-v1`: la via directa (`resolucion.tipo = rechazo`);
//! - `lote-v1` y `lote-v1-operacion-fuera`: un lote de UNA operacion, y el `data` nombra la 0 o la 1;
//! - `prenda-v1` y `prenda-v1-marca-otra`: el sobre de prenda se COPIA del vector capturado que se
//!   pasa como tercer argumento (`prenda-rechazada-por-la-capa.json`; la rama del rechazo no juzga
//!   su prueba, solo su digest, que es el `hashPrueba` del recibo), y `campos.consumo` es su marca u
//!   otra.
//!
//! Uso: `ventana-banda <caso> <salida.json> [<vector de prenda>]`. ⚠️ No es determinista: cada
//! corrida da otra prueba (y la misma cabeza, salvo la raiz). Los vectores salen de UNA corrida y
//! jamas se reescriben.

use stark_experiment::circuit_banda::{opciones, trazar, BandaProverDe, BandaWitness};
use stark_experiment::merkle::{native_merge, MerklePath};
use stark_experiment::native::{native_climb, native_leaf_salted};
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::{fft, FieldElement};
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AuxRandElements, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, EvaluationFrame, PartitionOptions, ProofOptions, Prover,
    StarkDomain, Trace, TraceInfo, TracePolyTable,
};
use xmss::KeyPair;
use zk_ssl_hash::{
    digest_of_proof, digest_to_bytes, epoch_digest_v5, epoch_digest_v6, hash_del_lote, Digest,
};
use zk_ssl_verify::{preambulo, recibos, Conjunto};

const PROFUNDIDAD: usize = 32;
/// El saldo de la ventana con el superior 0; con el superior `u`, `VENTANA + u`.
const VENTANA: u64 = (1u64 << 63) - (1u64 << 32) + 2;
const PEDIDO: u64 = 1001;
/// La ventana de los sobres de completitud: la `n` que firma el cierre y que lleva el recibo.
const N_VENTANA: u64 = 1440;
/// El recibo: el primero de la era (Q 0, recepCount 1), en la era 1 (la cabeza del rechazo es la
/// de indice embebido 0).
const RX: u64 = 1;
const ERA: u64 = 1;

// ------------------------------------------------- el probador malicioso (sin_comprobar.rs)

struct TrazaSinComprobar<T>(T);

impl<T: Trace> Trace for TrazaSinComprobar<T> {
    type BaseField = T::BaseField;

    fn info(&self) -> &TraceInfo {
        self.0.info()
    }

    fn main_segment(&self) -> &ColMatrix<Self::BaseField> {
        self.0.main_segment()
    }

    fn read_main_frame(&self, row_idx: usize, frame: &mut EvaluationFrame<Self::BaseField>) {
        self.0.read_main_frame(row_idx, frame)
    }

    fn comprobar<A, E>(
        &self,
        _air: &A,
        _aux: Option<(&ColMatrix<E>, &AuxRandElements<E>)>,
    ) -> Result<(), winter_prover::ProverError>
    where
        A: Air<BaseField = Self::BaseField>,
        E: FieldElement<BaseField = Self::BaseField>,
    {
        Ok(())
    }
}

struct OcultaSinComprobar<P>(P);

impl<P: Prover> Prover for OcultaSinComprobar<P> {
    type BaseField = P::BaseField;
    type Air = P::Air;
    type Trace = TrazaSinComprobar<P::Trace>;
    type HashFn = P::HashFn;
    type VC = P::VC;
    type RandomCoin = P::RandomCoin;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> = P::TraceLde<E>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        P::ConstraintEvaluator<'a, E>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        P::ConstraintCommitment<E>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> <Self::Air as Air>::PublicInputs {
        self.0.get_pub_inputs(&trace.0)
    }

    fn options(&self) -> &ProofOptions {
        self.0.options()
    }

    fn ocultacion(&self) -> Option<winter_prover::Ocultacion> {
        self.0.ocultacion()
    }

    fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
        partition_option: PartitionOptions,
    ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
        self.0.new_trace_lde(trace_info, main_trace, domain, partition_option)
    }

    fn new_evaluator<'a, E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        air: &'a Self::Air,
        aux_rand_elements: Option<AuxRandElements<E>>,
        composition_coefficients: ConstraintCompositionCoefficients<E>,
    ) -> Self::ConstraintEvaluator<'a, E> {
        self.0.new_evaluator(air, aux_rand_elements, composition_coefficients)
    }

    fn build_constraint_commitment<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        composition_poly_trace: CompositionPolyTrace<E>,
        num_constraint_composition_columns: usize,
        domain: &StarkDomain<Self::BaseField>,
        partition_options: PartitionOptions,
    ) -> (Self::ConstraintCommitment<E>, CompositionPoly<E>) {
        let mut c = composition_poly_trace.into_inner();
        let n = c.len();
        let offset = domain.offset();
        let inversos = fft::get_inv_twiddles::<Self::BaseField>(n);
        fft::interpolate_poly_with_offset(&mut c, &inversos, offset);
        let m = domain.cociente_m();
        if m > 0 {
            let cabe = (domain.trace_length() - m) * num_constraint_composition_columns;
            for x in c.iter_mut().skip(cabe) {
                *x = E::ZERO;
            }
        }
        let recortado =
            fft::evaluate_poly_with_offset(&c, &fft::get_twiddles::<Self::BaseField>(n), offset, 1);
        self.0.build_constraint_commitment(
            CompositionPolyTrace::new(recortado),
            num_constraint_composition_columns,
            domain,
            partition_options,
        )
    }
}

// ------------------------------------------------------------------ el paquete

fn hex(b: &[u8]) -> String {
    let mut s = String::from("0x");
    for x in b {
        s.push_str(&format!("{x:02x}"));
    }
    s
}

fn hd(d: &Digest) -> String {
    hex(&digest_to_bytes(d))
}

fn q(x: u64) -> String {
    format!("{x:#x}")
}

fn de_hex(t: &str) -> Vec<u8> {
    let t = t.strip_prefix("0x").expect("0x");
    (0..t.len()).step_by(2).map(|i| u8::from_str_radix(&t[i..i + 2], 16).expect("hex")).collect()
}

fn dg(k: u64) -> Digest {
    [BaseElement::new(k), BaseElement::new(k + 1), BaseElement::new(k + 2), BaseElement::new(k + 3)]
}

/// La hoja de la cuenta y su camino: hermanos del arbol vacio y direcciones mixtas.
fn testigo(saldo: u64) -> (BandaWitness, Digest) {
    let mut vacios = vec![[BaseElement::ZERO; 4]];
    for k in 1..=PROFUNDIDAD {
        let previo = vacios[k - 1];
        vacios.push(native_merge(previo, previo));
    }
    let public_id = dg(0x0B5E);
    let leaf_salt = dg(0x5A17);
    let nonce = BaseElement::ZERO;
    let mut siblings = Vec::with_capacity(PROFUNDIDAD);
    let mut is_right = Vec::with_capacity(PROFUNDIDAD);
    for nivel in 0..PROFUNDIDAD {
        siblings.push(vacios[nivel]);
        is_right.push(nivel % 3 == 0);
    }
    let path = MerklePath { siblings, is_right };
    let raiz =
        native_climb(native_leaf_salted(public_id, BaseElement::new(saldo), nonce, leaf_salt), &path);
    (BandaWitness { public_id, balance: saldo, nonce, leaf_salt, path }, raiz)
}

fn main() {
    let mut a = std::env::args().skip(1);
    let caso = a.next().expect("caso");
    let salida = a.next().expect("salida.json");
    let superior = PEDIDO - 1;
    let completitud = matches!(
        caso.as_str(),
        "completitud-v1" | "lote-v1" | "lote-v1-operacion-fuera" | "prenda-v1"
            | "prenda-v1-marca-otra"
    );
    let saldo = match caso.as_str() {
        "ventana-v1" | "ventana-v2" => VENTANA + superior,
        "control" => superior,
        _ if completitud => superior,
        otro => panic!("caso desconocido: {otro}"),
    };
    let (w, accounts) = testigo(saldo);
    let traza = trazar(&w, 0, superior);
    let prueba = match caso.as_str() {
        "ventana-v1" => BandaProverDe::<1>::new(opciones()).prove(traza),
        "ventana-v2" => OcultaSinComprobar(BandaProverDe::<2>::new(opciones()))
            .prove(TrazaSinComprobar(traza)),
        // la banda HONESTA de la v1: la de cualquier nodo de antes de la v2
        _ if completitud => BandaProverDe::<1>::new(opciones()).prove(traza),
        _ => BandaProverDe::<2>::new(opciones()).prove(traza),
    }
    .expect("probar")
    .to_bytes();

    let seq = 5u64;
    let (pending_root, frozen, chain, acuses, mmr, cons, params, pmeta_root) =
        (dg(100), dg(200), dg(300), dg(400), dg(500), dg(600), dg(700), dg(800));
    // la `n` de la cabeza es un parametro del nodo: en los de completitud, la de la ventana
    let n = if completitud { N_VENTANA } else { 3u64 };
    let (mmr_size, cons_count, next_pending, next_index, supply) = (5u64, 0u64, 0u64, 2u64, 2000u64);
    let ed = epoch_digest_v5(
        seq, accounts, pending_root, frozen, chain, acuses, n, mmr, mmr_size, cons, cons_count,
        params, pmeta_root, next_pending, next_index, supply,
    );
    let ed_b = digest_to_bytes(&ed);

    // Una clave XMSS^MT de test (el conjunto de la casa), de semilla fija: la del operador.
    let mut semilla = [0u8; 96];
    for (i, b) in semilla.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(31).wrapping_add(11);
    }
    let mut kp = KeyPair::<Conjunto>::from_seed(&semilla).expect("clave");
    let pk = kp.verifying_key().as_ref().to_vec();
    let firma = kp.signing_key().sign(&preambulo(5, &ed_b)).expect("firmar");

    let cabeza = serde_json::json!({
        "available": true,
        "formatVersion": q(5),
        "seq": q(seq),
        "accountsRoot": hd(&accounts),
        "pendingRoot": hd(&pending_root),
        "frozenRoot": hd(&frozen),
        "chainDigest": hd(&chain),
        "acusesRoot": hd(&acuses),
        "n": q(n),
        "mmrRoot": hd(&mmr),
        "mmrSize": q(mmr_size),
        "consRoot": hd(&cons),
        "consCount": q(cons_count),
        "paramsDigest": hd(&params),
        "pmetaRoot": hd(&pmeta_root),
        "nextPending": q(next_pending),
        "nextIndex": q(next_index),
        "totalSupply": q(supply),
        "epochDigest": hex(&ed_b),
        "publicKey": hex(&pk),
        "signature": hex(firma.as_ref()),
        // el declarado va por encima del embebido (0): lo que el productor de la casa hace
        "index": q(1),
    });
    let mut data = serde_json::json!({
        "causa": "InsufficientBalance",
        "campos": {"requested": q(PEDIDO)},
        "seq": q(seq),
    });

    // La completitud: lo que el recibo ata (el `hashPrueba`) y la resolucion, por caso.
    let mut resolucion = serde_json::json!({});
    let mut hash = None;
    if completitud {
        let h = match caso.as_str() {
            "completitud-v1" => {
                resolucion = serde_json::json!({"tipo": "rechazo"});
                dg(0xACE)
            }
            "lote-v1" | "lote-v1-operacion-fuera" => {
                let comp = [(dg(0xB0A), 7u64, 0u64)];
                let j = if caso == "lote-v1" { 0 } else { 1 };
                data["operacion"] = serde_json::json!(q(j));
                resolucion = serde_json::json!({
                    "tipo": "lote",
                    "composicion": [{
                        "hashPrueba": hd(&comp[0].0),
                        "cuenta": q(comp[0].1),
                        "posicion": q(comp[0].2),
                    }],
                });
                hash_del_lote(&comp)
            }
            _ => {
                let ruta = a.next().expect("el vector de prenda (tercer argumento)");
                let v: serde_json::Value =
                    serde_json::from_str(&std::fs::read_to_string(&ruta).expect("leer")).expect("json");
                let sobre = v["resolucion"]["sobre"].clone();
                assert_eq!(sobre["tipo"], "prenda", "{ruta}: su resolucion no lleva sobre de prenda");
                let marca = sobre["enunciado"]["marca"].as_str().expect("marca").to_string();
                let consumo = if caso == "prenda-v1" { marca } else { hd(&dg(0xA1E)) };
                data["campos"]["consumo"] = serde_json::json!(consumo);
                let h = digest_of_proof(&de_hex(sobre["prueba"].as_str().expect("prueba")));
                resolucion = serde_json::json!({"tipo": "prenda", "sobre": sobre});
                h
            }
        };
        data["recepcion"] = serde_json::json!({
            "era": q(ERA),
            "hashPrueba": hd(&h),
            "n": q(N_VENTANA),
            "rx": q(RX),
        });
        hash = Some(h);
    }

    let paquete = serde_json::json!({
        "v": 1,
        "tipo": "rechazo",
        "cabeza": cabeza,
        "data": data,
        "banda": {
            "prueba": hex(&prueba),
            "publicId": hd(&w.public_id),
            "requested": q(PEDIDO),
            "s": q(seq),
        },
    });

    let texto = match hash {
        None => serde_json::to_string(&paquete).expect("json"),
        Some(h) => {
            // El recibo bajo la recepRoot del cierre: UNA hoja, en la posicion 0, con hermanos cero.
            let pos = recibos::indice_de_recibo(RX, 0);
            let hermanos = vec![[BaseElement::ZERO; 4]; recibos::RECEP_DEPTH];
            let lados = vec![false; recibos::RECEP_DEPTH];
            let hoja = recibos::hoja_de_recibo(h, ERA, N_VENTANA);
            let recep_root =
                recibos::raiz_de_camino_de_recibo(pos, hoja, &hermanos, &lados).expect("camino");
            // El cierre: una cabeza v6 del MISMO operador, firmada despues (indice 1).
            let (seq_c, recep_count) = (6u64, 1u64);
            let ed6 = epoch_digest_v6(
                seq_c, accounts, pending_root, frozen, chain, acuses, N_VENTANA, mmr, mmr_size,
                cons, cons_count, params, pmeta_root, next_pending, next_index, supply, recep_root,
                recep_count,
            );
            let ed6_b = digest_to_bytes(&ed6);
            let firma6 = kp.signing_key().sign(&preambulo(6, &ed6_b)).expect("firmar el cierre");
            let cierre = serde_json::json!({
                "available": true,
                "formatVersion": q(6),
                "seq": q(seq_c),
                "accountsRoot": hd(&accounts),
                "pendingRoot": hd(&pending_root),
                "frozenRoot": hd(&frozen),
                "chainDigest": hd(&chain),
                "acusesRoot": hd(&acuses),
                "n": q(N_VENTANA),
                "mmrRoot": hd(&mmr),
                "mmrSize": q(mmr_size),
                "consRoot": hd(&cons),
                "consCount": q(cons_count),
                "paramsDigest": hd(&params),
                "pmetaRoot": hd(&pmeta_root),
                "nextPending": q(next_pending),
                "nextIndex": q(next_index),
                "totalSupply": q(supply),
                "recepRoot": hd(&recep_root),
                "recepCount": q(recep_count),
                "epochDigest": hex(&ed6_b),
                "publicKey": hex(&pk),
                "signature": hex(firma6.as_ref()),
                "index": q(2),
            });
            let clave = if caso.starts_with("prenda") { "rechazo" } else { "sobre" };
            resolucion[clave] = paquete;
            let sobre = serde_json::json!({
                "v": 1,
                "tipo": "completitud",
                "cierre": cierre,
                "recepcion": {"era": q(ERA), "hashPrueba": hd(&h), "n": q(N_VENTANA), "rx": q(RX)},
                "limiteAnterior": q(0),
                "camino": {
                    "siblings": hermanos.iter().map(hd).collect::<Vec<_>>(),
                    "isRight": lados,
                },
                "resolucion": resolucion,
            });
            serde_json::to_string(&sobre).expect("json")
        }
    };
    std::fs::write(&salida, &texto).expect("escribir");
    println!("PAQUETE\t{caso}\tsaldo\t{saldo}\tprueba\t{}\tpaquete\t{}", prueba.len(), texto.len());
}
