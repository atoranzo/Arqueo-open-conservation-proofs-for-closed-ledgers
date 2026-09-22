//! **RFC-0009 E2 (§531): el TESTIGO de la tabla de D-B, lo que revela una prueba.**
//!
//! Mientras el probador no oculte su testigo (D-A), lo que una prueba lleva en una columna
//! CONSTANTE de su traza sale literal en sus bytes. Este modulo produce cada tipo de prueba que el
//! nodo recibe o que un sobre publica -las diecinueve AIR del cable- y las cuatro experimentales
//! que la tabla tabula, cuenta en los bytes de cada una los valores de su testigo y asierta la
//! cuenta EXACTA que la tabla dice, con un control que no puede aparecer. Es SOLO de tests
//! (`cfg(test)` en `lib.rs`): no crece la API de la capa ni la del probador.
//!
//! - **La unidad** (D-M, medida en el PASTE-E2-M: 107 valores de 107, con `q` de 38 a 42): un
//!   ELEMENTO de columna constante sale `k(q+2)` veces -una por fila abierta y dos fuera del
//!   dominio, en `z` y en `z*g`- y un DIGEST de cuatro columnas contiguas, `k*q` -fuera del
//!   dominio la extension cuadratica lo intercala con ceros-. `q` son las posiciones UNICAS que
//!   la prueba abre (`num_unique_queries`, a lo sumo las 42 de `proof_options`), leidas de la
//!   propia prueba; `k`, las columnas o los bloques que lo llevan. Lo que no sale, cero.
//! - **Todo escalar grande de una columna constante es una celda**, del testigo o publico: el
//!   suministro, el limite, el techo, el maximo. Y las celdas de una prueba valen distinto dos a
//!   dos -`asierta` lo exige antes de contar-: si dos coincidieran, la cuenta de una contaria la
//!   otra. Asi leyo el PASTE-E2-M <<dos columnas>> en la quema y en la emision a pendiente, donde
//!   el suministro valia lo que el saldo o el importe; el ENSAYO-531 lo cazo en el envio (§531).
//!   Lo pequeno -nonces, contadores, nacidos, cotas inferiores- no se puede contar y no se cuenta.
//! - **La tabla como datos** (D-K): cada fila es una lista de celdas con su `k`. E3b, que la pone
//!   a cero, cambia datos y no codigo.
//! - **El censo** (D-L): un test lee los fuentes y falla si una AIR que la capa, el par de umbral
//!   o el kit verifican no tiene fila, si una fila del cable ya no la verifica nadie o si una
//!   fila nombra una AIR que no existe.
//! - Prueban STARK reales: en depuracion se saltan (nota 41) y `--release` los corre. Claves y
//!   valores de alta entropia, dentro del rango de cada circuito.

use crate::prueba_cobro::{prueba_de_cobro_pendiente, CabezaDePendientes, FotoDelCobro};
use crate::prueba_pago::{prueba_de_pago_en_curso, AperturaDelPago};
use crate::prueba_prenda::{prueba_de_prenda, CabezaDeLaPrenda};
use crate::store::{digest_to_bytes, element_to_bytes};
use crate::tests_support::{
    custodian_pair_with, freeze_climb_proof, mint_climb_proof, mint_pending_climb_proof,
    new_layer, open_and_fund, open_and_fund_wide, recovery_climb_proof, salt_de, state_of,
    wide_key, SK_ALICE, SK_BOB,
};
use crate::two_phase::{PendingNotice, RefundReceipt};
use crate::{client, proof_options, AccountIndex, Digest, SovereignLayer};
use stark_experiment::circuit_threshold_single_nullifier as auth;
use stark_experiment::double_entry as de;
use stark_experiment::merkle::{MerklePath, TREE_DEPTH};
use stark_experiment::native::derive_view_key_wide;
use std::collections::BTreeSet;
use std::path::Path;
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::FieldElement;
use winterfell::Prover;

// ---- valores de alta entropia dentro del rango de cada circuito ----
// La capa de test tiene un suministro de 100_000_000 y un limite regulatorio de 500_000.
const FONDO_A: u64 = 73_918_265;
const SALDO_B: u64 = 23_570_119;
const IMPORTE: u64 = 413_977;
const DELTA: u64 = 1_580_880_327;
const IMPORTE_EMISION: u64 = 7_381_229;
const SALDO_AUD: u64 = 31_415_927;
const UMBRAL_AUD: u64 = 12_345_679;
const QUEMA: u64 = 271_829;
const SALDO_BANDA: u64 = 64_120_387;
const PEDIDO_LEJOS: u64 = 91_604_417;
const FONDO_C: u64 = 42_097_553;
const FONDO_D: u64 = 17_302_861;
const IMPORTE_MINT: u64 = 3_811_279;
const SOLV_SALDO: u64 = 987_654_321_987;
const SOLV_IMPORTE: u64 = 123_456_789_123;
const SOLV_LIMITE: u64 = 500_000_000_000;
// Claves estrechas: la auditoria y la quema, los dos reembolsos y el segundo emisor de la edad.
const SK_AUD: u64 = 0x7A3B_91C4_5E2D_0F68;
const SK_R1: u64 = 0x3C5A_E21F_9B04_7D61;
const SK_R1_RECEPTOR: u64 = 0x71E9_4D2A_C6B3_0F58;
const SK_R2: u64 = 0x5B8F_D2E1_49A7_0C36;
const SK_R2_RECEPTOR: u64 = 0xA6C3_17E9_D05B_428F;
const SK_OTRO_EMISOR: u64 = 0xD00D_5EED;
/// Un valor que ninguna prueba lleva: si sale, la cuenta no vale.
const CONTROL: u64 = 0xDEAD_BEEF_CAFE_1234;
/// El techo publico de la banda del sobre de cobro: `2^62 - 1`, el del campo.
const TECHO_DE_LA_BANDA: u64 = stark_experiment::circuit_cobro_pendiente::MAX_VALOR;

fn ka() -> Digest {
    [
        BaseElement::new(0x9E37_79B9_7F4A_7C15),
        BaseElement::new(0xBF58_476D_1CE4_E5B9),
        BaseElement::new(0x94D0_49BB_1331_11EB),
        BaseElement::new(0x2545_F491_4F6C_DD1D),
    ]
}
fn kb() -> Digest {
    [
        BaseElement::new(0x1D8E_4E27_C47D_124F),
        BaseElement::new(0xB502_6F5A_A966_19E9),
        BaseElement::new(0x6595_A395_A2A1_A9F1),
        BaseElement::new(0x2B44_ACCA_B455_D165),
    ]
}
fn kc() -> Digest {
    [
        BaseElement::new(0x6C8E_2F41_A97D_3B05),
        BaseElement::new(0xD41A_97C3_0E6B_58F2),
        BaseElement::new(0x3B90_E7D5_1A2C_84F6),
        BaseElement::new(0x8F27_C1B4_6D03_E95A),
    ]
}
fn kd() -> Digest {
    [
        BaseElement::new(0x47D2_9A0E_C38B_61F5),
        BaseElement::new(0xA0E5_6B17_F24C_893D),
        BaseElement::new(0x5C3F_08A9_D71E_B246),
        BaseElement::new(0xE914_C752_3B8A_0F6D),
    ]
}
fn sal1() -> Digest {
    [
        BaseElement::new(0x2F9C_4E81_7B3A_D065),
        BaseElement::new(0x86E1_3D5A_C4F7_0B92),
        BaseElement::new(0x1B74_A92E_65D0_C38F),
        BaseElement::new(0xC5A8_0F36_E19B_7D24),
    ]
}
fn sal2() -> Digest {
    [
        BaseElement::new(0x7E05_B3C9_2A61_F48D),
        BaseElement::new(0x39D6_E0A4_8C1F_5B72),
        BaseElement::new(0xA2F3_5718_D0E9_6C4B),
        BaseElement::new(0x04BC_89E6_3F27_A15D),
    ]
}
fn sal_e() -> Digest {
    [
        BaseElement::new(0xB61D_27F0_49E3_8A5C),
        BaseElement::new(0x5A0C_E48B_16F9_D372),
        BaseElement::new(0xE8B3_4D71_A25C_06F9),
        BaseElement::new(0x13F6_9C2A_8E47_B0D5),
    ]
}
fn refund_f() -> Digest {
    [
        BaseElement::new(0x98A4_1F3D_C60B_E275),
        BaseElement::new(0x2DC5_B08F_7A13_964E),
        BaseElement::new(0xF147_6E92_3BD8_A05C),
        BaseElement::new(0x6B29_D0C5_E874_1FA3),
    ]
}
fn op_cus() -> Digest {
    [
        BaseElement::new(0x0F4B_97E2_D51C_6A38),
        BaseElement::new(0xC829_36A1_4FE0_7B5D),
        BaseElement::new(0x75E0_1C8B_A3D6_294F),
        BaseElement::new(0x3A6D_F215_9C87_E0B4),
    ]
}
fn op_gov() -> Digest {
    [
        BaseElement::new(0xE2A9_5C07_3F81_D46B),
        BaseElement::new(0x4C17_B8E3_0A95_62DF),
        BaseElement::new(0x9B5E_2D46_F7C0_1A83),
        BaseElement::new(0x21F8_67AD_C43E_950B),
    ]
}
fn nueva() -> Digest {
    [
        BaseElement::new(0x5D73_A1C8_E02F_946B),
        BaseElement::new(0xF0B6_3E59_1D7A_C824),
        BaseElement::new(0x8C21_D47F_B690_3E5A),
        BaseElement::new(0x36E9_0A2B_C5F1_874D),
    ]
}
/// Cinco claves de custodio: las del PASTE-360-M2 para el par, las del PASTE-367-M4 para el umbral.
fn custodios() -> Vec<BaseElement> {
    vec![
        BaseElement::new(0x8F3A_61C2_D94B_07E5),
        BaseElement::new(0x5B7E_2A90_C13F_D846),
        BaseElement::new(0xA4D1_7C38_0E9B_52F3),
        BaseElement::new(0x3E69_B5F0_847A_1DC2),
        BaseElement::new(0xC72F_0D5B_A916_E384),
    ]
}
/// Cuatro claves de miembro de gobernanza, las del PASTE-367-M3.
fn miembros() -> Vec<BaseElement> {
    vec![
        BaseElement::new(0x6A11_C3E9_27D4_B58F),
        BaseElement::new(0x2D8F_4B61_E09A_7C35),
        BaseElement::new(0xB3C7_1E52_8F46_A0D9),
        BaseElement::new(0x5E04_9A7B_C2D1_36F8),
    ]
}

// ---- la celda, la cuenta y la asercion ----

/// Un valor del testigo: un ELEMENTO de columna o un DIGEST de cuatro columnas contiguas.
#[derive(Clone, Copy)]
enum Valor {
    E(BaseElement),
    D(Digest),
}

/// Una celda de la tabla de D-B: el valor, cuantas columnas o bloques lo llevan, y su nombre.
struct Celda {
    que: String,
    valor: Valor,
    k: usize,
}

fn el(x: u64) -> Valor {
    Valor::E(BaseElement::new(x))
}

fn celda(que: &str, valor: Valor, k: usize) -> Celda {
    Celda { que: que.to_string(), valor, k }
}

/// Las apariciones de `x` en `pr`, ventana a ventana.
fn cuenta(pr: &[u8], x: &[u8]) -> usize {
    pr.windows(x.len()).filter(|v| *v == x).count()
}

/// Si valen lo mismo: dos digests, si son iguales; un elemento, si es alguno de los del otro (el
/// cero no cuenta: no es un valor que la cuenta pueda medir).
fn coinciden(a: &Valor, b: &Valor) -> bool {
    let partes = |v: &Valor| -> Vec<[u8; 8]> {
        match v {
            Valor::E(x) => vec![element_to_bytes(*x)],
            Valor::D(d) => d.iter().map(|x| element_to_bytes(*x)).collect(),
        }
    };
    match (a, b) {
        (Valor::D(x), Valor::D(y)) => x == y,
        _ => {
            let (pa, pb) = (partes(a), partes(b));
            pa.iter().any(|x| *x != [0u8; 8] && pb.contains(x))
        }
    }
}

/// Cuenta cada celda en los bytes de `pr` y la compara con la tabla: un elemento, `k(q+2)`; un
/// digest, `k*q`; con `q` leido de la propia prueba. Antes exige que ninguna celda que salga
/// valga lo que otra: la cuenta de una contaria la otra. El control tiene que dar cero. Junta
/// todas las discrepancias antes de fallar: un rojo dice cada celda que se movio.
fn asierta(fila: &str, pr: &[u8], celdas: &[Celda]) {
    for (i, a) in celdas.iter().enumerate() {
        for b in &celdas[i + 1..] {
            let alguna_sale = a.k > 0 || b.k > 0;
            assert!(
                !(alguna_sale && coinciden(&a.valor, &b.valor)),
                "{fila}: {} y {} valen lo mismo, y la cuenta de una contaria la otra",
                a.que,
                b.que
            );
        }
    }
    let prueba = winterfell::Proof::from_bytes(pr).expect("la prueba se deserializa");
    let q = prueba.num_unique_queries as usize;
    let tope = proof_options().num_queries();
    assert!(q >= 1 && q <= tope, "{fila}: q = {q}, fuera de 1..={tope}");
    let mut mal = Vec::new();
    for cel in celdas {
        let (bytes, esperada) = match cel.valor {
            Valor::E(x) => (element_to_bytes(x).to_vec(), cel.k * (q + 2)),
            Valor::D(d) => (digest_to_bytes(&d).to_vec(), cel.k * q),
        };
        let n = cuenta(pr, &bytes);
        println!("REVELA {fila} | {} | {n} veces (k {}, q {q})", cel.que, cel.k);
        if n != esperada {
            mal.push(format!("{} sale {n} veces y la tabla dice {esperada}", cel.que));
        }
    }
    let n = cuenta(pr, &element_to_bytes(BaseElement::new(CONTROL)));
    if n != 0 {
        mal.push(format!("el CONTROL sale {n} veces: la cuenta no vale"));
    }
    assert!(mal.is_empty(), "{fila} (q {q}): {}", mal.join("; "));
}

// ---- los montajes que comparten varias filas ----

/// Un envio v2 de `alice` a `receptor`, aplicado: el aviso que el receptor recibe.
fn envio_v2_aplicado(
    l: &mut SovereignLayer,
    alice: AccountIndex,
    ka: Digest,
    receptor: Digest,
) -> PendingNotice {
    let estado = state_of(l, alice);
    let m = l
        .send_materials_v2(alice, receptor, IMPORTE, sal2(), refund_f(), DELTA)
        .expect("materiales v2");
    let r = client::prove_send(&m, ka, proof_options()).expect("el envio v2");
    l.apply_send(&r, alice, &estado, IMPORTE).expect("el envio v2 se aplica");
    r.notice
}

/// La cabeza y la foto que el nodo sirve de la posicion `pos` (molde del PASTE-ZK-M2).
fn foto_de(l: &SovereignLayer, pos: u64) -> (CabezaDePendientes, FotoDelCobro) {
    let (emisor, nacido) = l.pending_meta_of(pos).expect("el pendiente tiene meta");
    let cab = CabezaDePendientes {
        seq: l.log.len() as u64,
        pending_root: l.pending.root(),
        pmeta_root: l.pending_meta_tree.root(),
    };
    let foto = FotoDelCobro {
        camino_pendiente: l.pending.path_for(pos),
        hermanos_meta: l.pending_meta_tree.path_for(pos).siblings,
        emisor,
        nacido,
    };
    (cab, foto)
}

/// Las dos autorizaciones de gobernanza de los miembros `a` y `b` (molde del PASTE-367-M3).
fn par_de_gobernanza(
    gk: &[BaseElement],
    op: Digest,
    a: usize,
    b: usize,
) -> (winterfell::Proof, winterfell::Proof) {
    let (_, gp) = stark_experiment::circuit_governance::build_governance_set(gk);
    let d = BaseElement::new(stark_experiment::circuit_governance::GOVERNANCE_DOMAIN);
    let prover = auth::NullifierThresholdProver::new(proof_options());
    let pa = prover.prove(auth::build_trace(d, gk[a], &gp[a], op)).expect("autorizacion A");
    let pb = prover.prove(auth::build_trace(d, gk[b], &gp[b], op)).expect("autorizacion B");
    (pa, pb)
}

/// Los envios `(desde, clave estrecha, importe, semilla de la sal)` a la cuenta `a`, aplicados.
fn envia(l: &mut SovereignLayer, envios: &[(AccountIndex, u64, u64, u64)], a: AccountIndex) {
    let receptor = l.public_id_of(a).expect("la cuenta receptora existe");
    for &(desde, sk, cuanto, s) in envios {
        let estado = state_of(l, desde);
        let recibo = l
            .send(BaseElement::new(sk), desde, &estado, receptor, salt_de(s), cuanto)
            .expect("el envio prueba");
        l.apply_send(&recibo, desde, &estado, cuanto).expect("el envio se aplica");
    }
}

/// La cabeza declarada del libro, como la firma el latido.
fn cabeza_declarada(l: &SovereignLayer) -> crate::prueba_edad::CabezaDeclarada {
    crate::prueba_edad::CabezaDeclarada {
        seq: l.log.len() as u64,
        pending_root: l.pending_root(),
        pmeta_root: l.pending_meta_tree.root(),
        next_pending: l.next_pending,
    }
}

/// Un reembolso de un envio aplicado, con lo que su recibo abre y el estado del emisor.
struct Reembolso {
    recibo: RefundReceipt,
    emisor: Digest,
    receptor: Digest,
    sal: Digest,
    importe: u64,
    saldo: u64,
    leaf_salt: Digest,
    x: Option<Digest>,
}

/// El reembolso v1 de un envio v1 con clave estrecha.
fn reembolso_v1() -> Reembolso {
    let mut l = new_layer();
    let a = open_and_fund(&mut l, SK_R1, FONDO_A);
    let b = open_and_fund(&mut l, SK_R1_RECEPTOR, 0);
    let emisor = l.public_id_of(a).expect("emisor");
    let receptor = l.public_id_of(b).expect("receptor");
    let estado = state_of(&l, a);
    let recibo = l
        .send(BaseElement::new(SK_R1), a, &estado, receptor, sal1(), IMPORTE)
        .expect("el envio v1");
    l.apply_send(&recibo, a, &estado, IMPORTE).expect("el envio v1 se aplica");
    let aviso = recibo.notice;
    let post = state_of(&l, a);
    let leaf_salt = l.account_view(a).expect("vista").leaf_salt;
    let (pos, sal, importe) = (aviso.position, aviso.salt, aviso.amount);
    let rr = l
        .refund(BaseElement::new(SK_R1), a, &post, pos, receptor, sal, importe)
        .expect("el reembolso v1");
    Reembolso {
        recibo: rr,
        emisor,
        receptor,
        sal: aviso.salt,
        importe: aviso.amount,
        saldo: post.balance,
        leaf_salt,
        x: None,
    }
}

/// El reembolso v2 de un envio v2 con clave estrecha (el envio, molde de los tests de
/// `prueba_prenda.rs`; el reembolso, del `instrumento_pago.rs`).
fn reembolso_v2() -> Reembolso {
    let mut l = new_layer();
    let a = open_and_fund(&mut l, SK_R2, FONDO_A);
    let b = open_and_fund(&mut l, SK_R2_RECEPTOR, 0);
    let emisor = l.public_id_of(a).expect("emisor");
    let receptor = l.public_id_of(b).expect("receptor");
    let m = l
        .send_materials_v2(a, receptor, IMPORTE, sal2(), refund_f(), DELTA)
        .expect("materiales v2");
    let clave = [BaseElement::new(SK_R2), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO];
    let r = client::prove_send(&m, clave, proof_options()).expect("el envio v2");
    let estado = state_of(&l, a);
    l.apply_send(&r, a, &estado, IMPORTE).expect("el envio v2 se aplica");
    let aviso = r.notice;
    let post = state_of(&l, a);
    let leaf_salt = l.account_view(a).expect("vista").leaf_salt;
    let rr = l
        .refund_v2(
            BaseElement::new(SK_R2),
            a,
            &post,
            aviso.position,
            receptor,
            aviso.salt,
            aviso.amount,
            refund_f(),
            DELTA,
        )
        .expect("el reembolso v2");
    Reembolso {
        recibo: rr,
        emisor,
        receptor,
        sal: aviso.salt,
        importe: aviso.amount,
        saldo: post.balance,
        leaf_salt,
        x: aviso.x,
    }
}

// ======================= LA TABLA DE D-B, FILA A FILA =======================

/// Envio (SEND-v1 y SEND-v2): la clave, las dos identidades, el saldo antes y despues, la sal y
/// el `leaf_salt` salen en los dos; en el v2, ademas, `X`. Ni la clave de vista ni, en el v2, el
/// `refund_id` o el `delta`. Publicos: el importe, el limite y el suministro, este en dos
/// columnas porque el envio no lo mueve. Bob tiene saldo: con el suyo a cero, el suministro seria
/// el saldo de alice y la guarda de `asierta` lo rechaza (el falsador M3 del ENSAYO-531-r2).
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_el_envio() {
    let (ka, kb, rf) = (ka(), kb(), refund_f());
    let mut l = new_layer();
    let alice = open_and_fund_wide(&mut l, ka, FONDO_A);
    let bob = open_and_fund_wide(&mut l, kb, SALDO_B);
    let id_alice = l.public_id_of(alice).expect("alice");
    let id_bob = l.public_id_of(bob).expect("bob");
    let leaf_salt = l.account_view(alice).expect("vista de alice").leaf_salt;
    let (limite, suministro) = (l.regulatory_limit(), l.total_supply());
    let comunes = |sal: Digest| {
        vec![
            celda("la clave de gasto", Valor::D(ka), 1),
            celda("la identidad del receptor", Valor::D(id_bob), 1),
            celda("la identidad del emisor", Valor::D(id_alice), 1),
            celda("el saldo", el(FONDO_A), 1),
            celda("el saldo despues", el(FONDO_A - IMPORTE), 1),
            celda("el importe (publico)", el(IMPORTE), 1),
            celda("el limite regulatorio (publico)", el(limite), 1),
            celda("el suministro (publico), en dos columnas", el(suministro), 2),
            celda("la sal", Valor::D(sal), 1),
            celda("el leaf_salt", Valor::D(leaf_salt), 1),
            celda("la clave de vista", Valor::D(derive_view_key_wide(ka)), 0),
        ]
    };
    let m1 = l.send_materials(alice, id_bob, IMPORTE, sal1()).expect("materiales v1");
    let r1 = client::prove_send(&m1, ka, proof_options()).expect("el envio v1");
    asierta("envio v1 (SendAir)", &r1.proof, &comunes(sal1()));
    let m2 = l
        .send_materials_v2(alice, id_bob, IMPORTE, sal2(), rf, DELTA)
        .expect("materiales v2");
    let r2 = client::prove_send(&m2, ka, proof_options()).expect("el envio v2");
    let x = r2.notice.x.expect("el aviso v2 lleva X");
    let mut celdas = comunes(sal2());
    celdas.push(celda("el sobre X", Valor::D(x), 1));
    celdas.push(celda("el refund_id", Valor::D(rf), 0));
    celdas.push(celda("el delta", el(DELTA), 0));
    asierta("envio v2 (SendV2Air)", &r2.proof, &celdas);
}

/// Cobro (CLAIM-v1 y CLAIM-v2): la clave, la identidad del receptor en dos bloques -la de la
/// cuenta y la del pendiente, que el circuito obliga a ser la misma-, sus dos saldos, la sal y
/// su `leaf_salt`; en el v2, ademas, `X`. Del pagador, nada; ni, en el v2, el `refund_id` o el
/// `delta`. Publicos: el importe y el suministro, este en dos columnas.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_el_cobro() {
    let (ka, kb, rf) = (ka(), kb(), refund_f());
    let mut l = new_layer();
    let alice = open_and_fund_wide(&mut l, ka, FONDO_A);
    let bob = open_and_fund_wide(&mut l, kb, SALDO_B);
    let id_alice = l.public_id_of(alice).expect("alice");
    let id_bob = l.public_id_of(bob).expect("bob");
    let leaf_salt = l.account_view(bob).expect("vista de bob").leaf_salt;
    let suministro = l.total_supply();
    let comunes = |sal: Digest| {
        vec![
            celda("el suministro (publico), en dos columnas", el(suministro), 2),
            celda("la clave de gasto", Valor::D(kb), 1),
            celda("la identidad del receptor, en dos bloques", Valor::D(id_bob), 2),
            celda("el saldo del receptor antes", el(SALDO_B), 1),
            celda("el saldo del receptor despues", el(SALDO_B + IMPORTE), 1),
            celda("el importe (publico)", el(IMPORTE), 1),
            celda("la sal", Valor::D(sal), 1),
            celda("el leaf_salt del receptor", Valor::D(leaf_salt), 1),
            celda("la identidad del pagador", Valor::D(id_alice), 0),
        ]
    };
    let estado = state_of(&l, alice);
    let m1 = l.send_materials(alice, id_bob, IMPORTE, sal1()).expect("materiales v1");
    let r1 = client::prove_send(&m1, ka, proof_options()).expect("el envio v1");
    l.apply_send(&r1, alice, &estado, IMPORTE).expect("el envio v1 se aplica");
    let cm1 = l.claim_materials(bob, &r1.notice).expect("materiales del cobro v1");
    let c1 = client::prove_claim(&cm1, kb, proof_options()).expect("el cobro v1");
    asierta("cobro v1 (ClaimAir)", &c1.proof, &comunes(sal1()));
    let estado = state_of(&l, alice);
    let m2 = l
        .send_materials_v2(alice, id_bob, IMPORTE, sal2(), rf, DELTA)
        .expect("materiales v2");
    let r2 = client::prove_send(&m2, ka, proof_options()).expect("el envio v2");
    l.apply_send(&r2, alice, &estado, IMPORTE).expect("el envio v2 se aplica");
    let x = r2.notice.x.expect("el aviso v2 lleva X");
    let cm2 = l.claim_materials(bob, &r2.notice).expect("materiales del cobro v2");
    let c2 = client::prove_claim(&cm2, kb, proof_options()).expect("el cobro v2");
    let mut celdas = comunes(sal2());
    celdas.push(celda("el sobre X", Valor::D(x), 1));
    celdas.push(celda("el refund_id", Valor::D(rf), 0));
    celdas.push(celda("el delta", el(DELTA), 0));
    asierta("cobro v2 (ClaimAirV2)", &c2.proof, &celdas);
}

/// Prenda: la clave de gasto, la sal, el importe y `X`; la identidad del prendador es publica.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_prenda() {
    let (ka, kb) = (ka(), kb());
    let mut l = new_layer();
    let alice = open_and_fund_wide(&mut l, ka, FONDO_A);
    let bob = open_and_fund_wide(&mut l, kb, 0);
    let id_bob = l.public_id_of(bob).expect("bob");
    let aviso = envio_v2_aplicado(&mut l, alice, ka, id_bob);
    let x = aviso.x.expect("el aviso v2 lleva X");
    let cab = CabezaDeLaPrenda { seq: l.log.len() as u64, pending_root: l.pending.root() };
    let camino = l.pending.path_for(aviso.position);
    let s = prueba_de_prenda(&cab, kb, &aviso, &camino).expect("la prenda");
    let celdas = [
        celda("la clave de gasto", Valor::D(kb), 1),
        celda("la sal", Valor::D(aviso.salt), 1),
        celda("el importe", el(aviso.amount), 1),
        celda("el sobre X", Valor::D(x), 1),
        celda("la identidad del prendador (publica)", Valor::D(id_bob), 1),
    ];
    asierta("prenda (PrendaAir)", &s.prueba, &celdas);
}

/// Emision a pendiente: la identidad del receptor y la sal. Publicos: el importe, el suministro
/// antes y despues y el maximo. Hay suministro previo: con el libro vacio, el suministro despues
/// seria el importe, y el PASTE-E2-M leyo ahi <<el importe en dos columnas>>.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_emision_a_pendiente() {
    let mut l = new_layer();
    let _tercero = open_and_fund_wide(&mut l, kd(), FONDO_D);
    let bob = open_and_fund_wide(&mut l, kb(), 0);
    let receptor = l.public_id_of(bob).expect("bob");
    let (suministro, maximo) = (l.total_supply(), l.max_supply());
    let p = mint_pending_climb_proof(&l, receptor, sal_e(), IMPORTE_EMISION);
    let celdas = [
        celda("la identidad del receptor", Valor::D(receptor), 1),
        celda("la sal", Valor::D(sal_e()), 1),
        celda("el importe (publico)", el(IMPORTE_EMISION), 1),
        celda("el suministro antes (publico)", el(suministro), 1),
        celda("el suministro despues (publico)", el(suministro + IMPORTE_EMISION), 1),
        celda("el maximo de suministro (publico)", el(maximo), 1),
    ];
    asierta("emision a pendiente (MintPendingClimbAir)", &p.to_bytes(), &celdas);
}

/// Autorizacion delegada de un custodio: la clave de SU custodio y la operacion (publica); ni
/// la del otro custodio ni la del que no firma.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_autorizacion_delegada() {
    let ck = custodios();
    let op = op_cus();
    let (pa, _, pb, _) = custodian_pair_with(&ck, op, 1, 3);
    for (fila, prueba, suya, otra) in [
        ("autorizacion delegada, custodio 1", pa, 1, 3),
        ("autorizacion delegada, custodio 3", pb, 3, 1),
    ] {
        let celdas = [
            celda("la clave de SU custodio", Valor::E(ck[suya]), 1),
            celda("la operacion autorizada (publica)", Valor::D(op), 1),
            celda("la clave del otro custodio", Valor::E(ck[otra]), 0),
            celda("la clave del que no firma", Valor::E(ck[0]), 0),
        ];
        asierta(fila, &prueba.to_bytes(), &celdas);
    }
}

/// Gobernanza delegada: la clave de cada miembro y la operacion (publica); ni la del otro
/// miembro ni la del que no firma.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_gobernanza_delegada() {
    let gk = miembros();
    let op = op_gov();
    let (pa, pb) = par_de_gobernanza(&gk, op, 1, 3);
    for (fila, prueba, suya, otra) in [
        ("gobernanza delegada, miembro 1", pa, 1, 3),
        ("gobernanza delegada, miembro 3", pb, 3, 1),
    ] {
        let celdas = [
            celda("la clave del miembro", Valor::E(gk[suya]), 1),
            celda("la operacion autorizada (publica)", Valor::D(op), 1),
            celda("la clave del otro miembro", Valor::E(gk[otra]), 0),
            celda("la clave del que no firma", Valor::E(gk[0]), 0),
        ];
        asierta(fila, &prueba.to_bytes(), &celdas);
    }
}

/// Umbral conjunto (`circuit_threshold`): las dos claves; la del custodio que no firma, no.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_el_umbral_conjunto() {
    let ck = custodios();
    let (_, caminos) = stark_experiment::circuit_threshold::build_custodian_set(&ck);
    let traza = stark_experiment::circuit_threshold::build_trace(
        ck[1],
        1,
        &caminos[1],
        ck[3],
        3,
        &caminos[3],
    );
    let p = stark_experiment::circuit_threshold::ThresholdProver::new(proof_options())
        .prove(traza)
        .expect("el umbral conjunto");
    let celdas = [
        celda("la clave del custodio 1", Valor::E(ck[1]), 1),
        celda("la clave del custodio 3", Valor::E(ck[3]), 1),
        celda("la clave del que no firma", Valor::E(ck[0]), 0),
    ];
    asierta("umbral conjunto (ThresholdAir)", &p.to_bytes(), &celdas);
}

/// Auditoria: la clave de gasto, el saldo exacto y el `leaf_salt`. Publicos: la identidad, el
/// umbral y el techo (`prove_minimum` pide <<al menos>>: el techo es `2^62 - 1`).
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_auditoria() {
    let mut l = new_layer();
    let i = open_and_fund(&mut l, SK_AUD, SALDO_AUD);
    let v = l.account_view(i).expect("vista");
    let au = l
        .prove_minimum(BaseElement::new(SK_AUD), i, &state_of(&l, i), UMBRAL_AUD)
        .expect("la auditoria");
    let celdas = [
        celda("la clave de gasto", el(SK_AUD), 1),
        celda("el saldo exacto", el(SALDO_AUD), 1),
        celda("el leaf_salt", Valor::D(v.leaf_salt), 1),
        celda("la identidad de la cuenta (publica)", Valor::D(v.public_id), 1),
        celda("el umbral (publico)", el(UMBRAL_AUD), 1),
        celda("el techo (publico)", el(stark_experiment::circuit_audit::MAX_VALUE), 1),
    ];
    asierta("auditoria (AuditAir)", &au.proof, &celdas);
}

/// Quema: la clave de gasto, el saldo antes y despues, el `leaf_salt` y la identidad de la
/// cuenta. Publicos: el importe y el suministro antes y despues. Un tercero tiene saldo: con una
/// sola cuenta, el suministro seria el saldo, y el PASTE-E2-M leyo ahi <<dos columnas>>.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_quema() {
    let mut l = new_layer();
    let _tercero = open_and_fund_wide(&mut l, kd(), FONDO_D);
    let i = open_and_fund(&mut l, SK_AUD, SALDO_AUD);
    let v = l.account_view(i).expect("vista");
    let suministro = l.total_supply();
    let qu = l.burn(BaseElement::new(SK_AUD), i, &state_of(&l, i), QUEMA).expect("la quema");
    let celdas = [
        celda("la clave de gasto", el(SK_AUD), 1),
        celda("el saldo antes", el(SALDO_AUD), 1),
        celda("el saldo despues", el(SALDO_AUD - QUEMA), 1),
        celda("el importe (publico)", el(QUEMA), 1),
        celda("el suministro antes (publico)", el(suministro), 1),
        celda("el suministro despues (publico)", el(suministro - QUEMA), 1),
        celda("el leaf_salt", Valor::D(v.leaf_salt), 1),
        celda("la identidad de la cuenta", Valor::D(v.public_id), 1),
    ];
    asierta("quema (BurnAir)", &qu.proof, &celdas);
}

/// Solvencia (`stark-experiment`): el saldo y el importe; el limite es publico.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_solvencia() {
    let traza = stark_experiment::solvency::build_trace(SOLV_SALDO, SOLV_IMPORTE, SOLV_LIMITE);
    let p = stark_experiment::solvency::SolvencyProver::new(proof_options())
        .prove(traza)
        .expect("la solvencia");
    let celdas = [
        celda("el saldo", el(SOLV_SALDO), 1),
        celda("el importe", el(SOLV_IMPORTE), 1),
        celda("el limite (publico)", el(SOLV_LIMITE), 1),
    ];
    asierta("solvencia (SolvencyAir)", &p.to_bytes(), &celdas);
}

/// `double_entry` (`stark-experiment`, molde del PASTE-367-M4): las dos identidades, los cuatro
/// saldos, el importe y los dos nonces; el limite es publico.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_el_double_entry() {
    let s_id = BaseElement::new(0x51D0_A7E3_C2B9_4F18);
    let s_nonce = BaseElement::new(0x0A11_7E55_31C9_D2B4);
    let r_id = BaseElement::new(0x7B3E_9D02_F4A1_6C85);
    let r_nonce = BaseElement::new(0x2C6F_81B7_05DA_E39E);
    const S_BAL: u64 = 987_653;
    const R_BAL: u64 = 424_243;
    const IMPORTE_DE: u64 = 123_457;
    const LIMITE_DE: u64 = 500_000;
    let s_leaf_new = de::native_leaf(
        s_id,
        BaseElement::new(S_BAL) - BaseElement::new(IMPORTE_DE),
        s_nonce + BaseElement::ONE,
    );
    let r_leaf_old = de::native_leaf(r_id, BaseElement::new(R_BAL), r_nonce);
    let mut s_sib: Vec<Digest> = vec![r_leaf_old];
    let mut s_der: Vec<bool> = vec![false];
    let mut r_sib: Vec<Digest> = vec![s_leaf_new];
    let mut r_der: Vec<bool> = vec![true];
    for nivel in 1..TREE_DEPTH {
        let n = 0x6100_0000_0000_0000u64 + (nivel as u64) * 0x0101_0101;
        let d: Digest = [
            BaseElement::new(n),
            BaseElement::new(n + 1),
            BaseElement::new(n + 2),
            BaseElement::new(n + 3),
        ];
        s_sib.push(d);
        s_der.push(false);
        r_sib.push(d);
        r_der.push(false);
    }
    let emisor = de::PartyWitness {
        account_id: s_id,
        balance: S_BAL,
        nonce: s_nonce,
        path: MerklePath { siblings: s_sib, is_right: s_der },
    };
    let receptor = de::PartyWitness {
        account_id: r_id,
        balance: R_BAL,
        nonce: r_nonce,
        path: MerklePath { siblings: r_sib, is_right: r_der },
    };
    let traza = de::build_trace(&emisor, &receptor, IMPORTE_DE, IMPORTE_DE, LIMITE_DE);
    let p = de::DoubleEntryProver::new(proof_options()).prove(traza).expect("el double_entry");
    let celdas = [
        celda("la identidad del emisor", Valor::E(s_id), 1),
        celda("el saldo del emisor antes", el(S_BAL), 1),
        celda("el saldo del emisor despues", el(S_BAL - IMPORTE_DE), 1),
        celda("la identidad del receptor", Valor::E(r_id), 1),
        celda("el saldo del receptor antes", el(R_BAL), 1),
        celda("el saldo del receptor despues", el(R_BAL + IMPORTE_DE), 1),
        celda("el importe", el(IMPORTE_DE), 1),
        celda("el nonce del emisor", Valor::E(s_nonce), 1),
        celda("el nonce del receptor", Valor::E(r_nonce), 1),
        celda("el limite (publico)", el(LIMITE_DE), 1),
    ];
    asierta("double_entry (DoubleEntryAir)", &p.to_bytes(), &celdas);
}

/// Banda: el saldo y el `leaf_salt`; la identidad y la cota superior (`pedido - 1`) son
/// publicas. Con el pedido en el saldo mas uno, saldo y cota coinciden: dos columnas.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_banda() {
    let mut l = new_layer();
    let a = open_and_fund_wide(&mut l, ka(), SALDO_BANDA);
    let v = l.account_view(a).expect("vista");
    let cab = crate::prueba_banda::CabezaDeCuentas {
        seq: l.log.len() as u64,
        accounts_root: l.state_root(),
    };
    let lejos = l.prueba_de_banda(&cab, a, PEDIDO_LEJOS).expect("la banda con el pedido lejos");
    let celdas = [
        celda("el saldo", el(SALDO_BANDA), 1),
        celda("el leaf_salt", Valor::D(v.leaf_salt), 1),
        celda("la cota superior, pedido - 1 (publica)", el(PEDIDO_LEJOS - 1), 1),
        celda("la identidad de la cuenta (publica)", Valor::D(v.public_id), 1),
    ];
    asierta("banda, pedido lejos (BandaAir)", &lejos.prueba, &celdas);
    let pegada = l.prueba_de_banda(&cab, a, SALDO_BANDA + 1).expect("la banda pegada");
    let celdas = [
        celda("el saldo, que es la cota: dos columnas", el(SALDO_BANDA), 2),
        celda("el leaf_salt", Valor::D(v.leaf_salt), 1),
        celda("la identidad de la cuenta (publica)", Valor::D(v.public_id), 1),
    ];
    asierta("banda, pedido = saldo + 1 (BandaAir)", &pegada.prueba, &celdas);
}

/// Edad: el emisor, cuando todos los pendientes son del mismo; con emisores distintos, ninguno.
/// Ni el `nacido` ni la hoja de los pendientes.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_edad() {
    // Seiscientas cuentas delante: el `nacido` sale grande y su cero significa algo.
    let mut l = new_layer();
    for i in 0..600u64 {
        let _ = l.open_account_wide(wide_key(0x5EED_0000 + i));
    }
    let ea = open_and_fund(&mut l, SK_ALICE, 10_000);
    let eb = open_and_fund(&mut l, SK_BOB, 0);
    envia(&mut l, &[(ea, SK_ALICE, 100, 1), (ea, SK_ALICE, 200, 2)], eb);
    let s = l.prueba_de_edad(&cabeza_declarada(&l), 1, None).expect("la edad, un emisor");
    let (emisor, _) = l.pending_meta_of(0).expect("un vivo tiene meta");
    let mut celdas = vec![celda("el emisor, el mismo en todos los pendientes", el(emisor), 1)];
    for p in 0..l.next_pending {
        let (em, na) = l.pending_meta_of(p).expect("un vivo tiene meta");
        assert_eq!(em, emisor, "el montaje quiere un solo emisor");
        celdas.push(celda(&format!("el nacido del pendiente {p}"), el(na), 0));
        celdas.push(celda(&format!("la hoja del pendiente {p}"), Valor::D(l.pending_at(p)), 0));
    }
    asierta("edad, un emisor (EdadAir)", &s.prueba, &celdas);

    let mut l = new_layer();
    let ea = open_and_fund(&mut l, SK_ALICE, 10_000);
    let ed = open_and_fund(&mut l, SK_OTRO_EMISOR, 10_000);
    let eb = open_and_fund(&mut l, SK_BOB, 0);
    envia(&mut l, &[(ea, SK_ALICE, 100, 1), (ed, SK_OTRO_EMISOR, 200, 2)], eb);
    let s = l.prueba_de_edad(&cabeza_declarada(&l), 1, None).expect("la edad, dos emisores");
    let mut celdas = Vec::new();
    for p in 0..l.next_pending {
        let (em, _) = l.pending_meta_of(p).expect("un vivo tiene meta");
        celdas.push(celda(&format!("el emisor del pendiente {p}"), el(em), 0));
    }
    asierta("edad, dos emisores (EdadAir)", &s.prueba, &celdas);
}

/// Sobre de cobro (portable): la sal, `X`, el importe exacto y el emisor -el indice de su
/// cuenta-. Publicos: la identidad del receptor y el techo de la banda (el sobre dice <<al menos
/// `inferior`>>); la cota inferior y el nacido tambien, pero son pequenos y no se cuentan.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_el_sobre_de_cobro() {
    let (ka, kb) = (ka(), kb());
    let mut l = new_layer();
    let alice = open_and_fund_wide(&mut l, ka, FONDO_A);
    let bob = open_and_fund_wide(&mut l, kb, 0);
    let id_bob = l.public_id_of(bob).expect("bob");
    let aviso = envio_v2_aplicado(&mut l, alice, ka, id_bob);
    let x = aviso.x.expect("el aviso v2 lleva X");
    let (cab, foto) = foto_de(&l, aviso.position);
    let s = prueba_de_cobro_pendiente(&cab, id_bob, &aviso, &foto, 1).expect("el sobre de cobro");
    let celdas = [
        celda("la sal", Valor::D(aviso.salt), 1),
        celda("el sobre X", Valor::D(x), 1),
        celda("el importe exacto", el(aviso.amount), 1),
        celda("el emisor, el indice de su cuenta", el(foto.emisor), 1),
        celda("la identidad del receptor (publica)", Valor::D(id_bob), 1),
        celda("el techo de la banda (publico)", el(TECHO_DE_LA_BANDA), 1),
    ];
    asierta("sobre de cobro (CobroPendienteAir)", &s.prueba, &celdas);
}

/// Sobre de pago (portable): la sal, el `delta`, el `refund_id` y el emisor; el importe es
/// publico. `X` no sale.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_el_sobre_de_pago() {
    let (ka, kb) = (ka(), kb());
    let mut l = new_layer();
    let alice = open_and_fund_wide(&mut l, ka, FONDO_A);
    let bob = open_and_fund_wide(&mut l, kb, 0);
    let id_bob = l.public_id_of(bob).expect("bob");
    let aviso = envio_v2_aplicado(&mut l, alice, ka, id_bob);
    let x = aviso.x.expect("el aviso v2 lleva X");
    let (cab, foto) = foto_de(&l, aviso.position);
    let ap = AperturaDelPago {
        posicion: aviso.position,
        sal: aviso.salt,
        importe: aviso.amount,
        refund_id: refund_f(),
        delta: DELTA,
    };
    let s = prueba_de_pago_en_curso(&cab, id_bob, &ap, &foto, foto.nacido).expect("el pago");
    let celdas = [
        celda("la sal", Valor::D(aviso.salt), 1),
        celda("el delta", el(DELTA), 1),
        celda("el refund_id", Valor::D(refund_f()), 1),
        celda("el emisor, el indice de su cuenta", el(foto.emisor), 1),
        celda("el importe (publico)", el(aviso.amount), 1),
        celda("el sobre X", Valor::D(x), 0),
    ];
    asierta("sobre de pago (PagoEnCursoAir)", &s.prueba, &celdas);
}

/// Camino de merkle (`stark-experiment`, molde del PASTE-367-M3): nada; ni la hoja ni los
/// hermanos son constantes.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_el_camino_de_merkle() {
    let hoja: Digest = [
        BaseElement::new(0x9C1B_7E35_48A2_D06F),
        BaseElement::new(0x07F3_5D1C_B2E8_964A),
        BaseElement::new(0xE54A_0B97_3C6D_21F8),
        BaseElement::new(0x4B2D_F860_1A95_C7E3),
    ];
    let hermanos: Vec<Digest> = (0..TREE_DEPTH)
        .map(|i| {
            let n = 0x5100_0000_0000_0000u64 + (i as u64) * 0x0101_0101;
            [
                BaseElement::new(n),
                BaseElement::new(n + 1),
                BaseElement::new(n + 2),
                BaseElement::new(n + 3),
            ]
        })
        .collect();
    let a_la_derecha: Vec<bool> = (0..TREE_DEPTH).map(|i| i % 3 == 0).collect();
    let camino = MerklePath { siblings: hermanos.clone(), is_right: a_la_derecha };
    let traza = stark_experiment::merkle::build_trace(hoja, &camino);
    let p = stark_experiment::merkle::MerkleProver::new(proof_options())
        .prove(traza)
        .expect("el camino de merkle");
    let celdas = [
        celda("la hoja", Valor::D(hoja), 0),
        celda("el hermano del nivel 5", Valor::D(hermanos[5]), 0),
        celda("el primer elemento de la hoja", Valor::E(hoja[0]), 0),
    ];
    asierta("camino de merkle (MerkleAir)", &p.to_bytes(), &celdas);
}

/// Apertura del reembolso y de la des-emision (v1 y v2, el mismo constructor): nada, porque su
/// traza no tiene columnas constantes. El importe, y en el v2 la apertura, van publicos en el
/// recibo, no en la prueba.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_apertura_del_reembolso() {
    for (fila, r) in [
        ("apertura del reembolso v1 (RefundAir)", reembolso_v1()),
        ("apertura del reembolso v2 (RefundAirV2)", reembolso_v2()),
    ] {
        let mut celdas = vec![
            celda("la identidad del receptor", Valor::D(r.receptor), 0),
            celda("la sal", Valor::D(r.sal), 0),
            celda("el importe (publico en el recibo)", el(r.importe), 0),
        ];
        if let Some(x) = r.x {
            celdas.push(celda("el refund_id (publico en el recibo)", Valor::D(refund_f()), 0));
            celdas.push(celda("el delta (publico en el recibo)", el(DELTA), 0));
            celdas.push(celda("el sobre X", Valor::D(x), 0));
        }
        asierta(fila, &r.recibo.refund_proof, &celdas);
    }
}

/// Subida de credito del reembolso: la identidad de la cuenta que recupera el dinero, sus dos
/// saldos, el importe (publico) y su `leaf_salt`.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_subida_de_credito() {
    let r = reembolso_v1();
    let celdas = [
        celda("la identidad de la cuenta", Valor::D(r.emisor), 1),
        celda("el saldo antes", el(r.saldo), 1),
        celda("el saldo despues", el(r.saldo + r.importe), 1),
        celda("el importe (publico)", el(r.importe), 1),
        celda("el leaf_salt", Valor::D(r.leaf_salt), 1),
    ];
    asierta("subida de credito (CreditClimbAir)", &r.recibo.credit_proof, &celdas);
}

/// Subida de la emision delegada: la identidad de la cuenta, sus dos saldos y su `leaf_salt`.
/// Publicos: el importe, el suministro antes y despues y el maximo.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_subida_de_la_emision() {
    let mut l = new_layer();
    let _otra = open_and_fund_wide(&mut l, kd(), FONDO_D);
    let c = open_and_fund_wide(&mut l, kc(), FONDO_C);
    let v = l.account_view(c).expect("vista");
    let (suministro, maximo) = (l.total_supply(), l.max_supply());
    let p = mint_climb_proof(&l, c, IMPORTE_MINT);
    let celdas = [
        celda("la identidad de la cuenta", Valor::D(v.public_id), 1),
        celda("el saldo antes", el(FONDO_C), 1),
        celda("el saldo despues", el(FONDO_C + IMPORTE_MINT), 1),
        celda("el leaf_salt", Valor::D(v.leaf_salt), 1),
        celda("el importe (publico)", el(IMPORTE_MINT), 1),
        celda("el suministro antes (publico)", el(suministro), 1),
        celda("el suministro despues (publico)", el(suministro + IMPORTE_MINT), 1),
        celda("el maximo de suministro (publico)", el(maximo), 1),
    ];
    asierta("subida de la emision delegada (MintClimbAir)", &p.to_bytes(), &celdas);
}

/// Subida de la congelacion delegada: nada; ni el indice de la cuenta ni la marca de congelada.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_subida_de_la_congelacion() {
    let mut l = new_layer();
    let c = open_and_fund_wide(&mut l, kc(), 0);
    let p = freeze_climb_proof(&l, c, true);
    let celdas = [
        celda("el indice de la cuenta", el(c), 0),
        celda("la marca de congelada", el(stark_experiment::circuit_freeze::FROZEN_MARK), 0),
    ];
    asierta("subida de la congelacion delegada (FrozenClimbAir)", &p.to_bytes(), &celdas);
}

/// Subida de la recuperacion delegada: la identidad vieja, el saldo y el `leaf_salt`; la nueva
/// la nombra la operacion.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn revela_la_subida_de_la_recuperacion() {
    let mut l = new_layer();
    let c = open_and_fund_wide(&mut l, kc(), FONDO_C);
    let v = l.account_view(c).expect("vista");
    let p = recovery_climb_proof(&l, c, nueva());
    let celdas = [
        celda("la identidad vieja", Valor::D(v.public_id), 1),
        celda("la identidad nueva (la operacion la nombra)", Valor::D(nueva()), 1),
        celda("el saldo", el(FONDO_C), 1),
        celda("el leaf_salt", Valor::D(v.leaf_salt), 1),
    ];
    asierta("subida de la recuperacion delegada (RecoveryClimbAir)", &p.to_bytes(), &celdas);
}

// ======================= EL CENSO DEL CABLE (D-L) =======================

/// Cada AIR con su fila de la tabla. `true`: la verifica la capa, el par de umbral o el kit (el
/// cable). `false`: experimental, en la tabla pero fuera del cable.
const FILAS: &[(&str, &str, bool)] = &[
    ("SendAir", "envio", true),
    ("SendV2Air", "envio", true),
    ("ClaimAir", "cobro", true),
    ("ClaimAirV2", "cobro", true),
    ("PrendaAir", "prenda", true),
    ("MintPendingClimbAir", "emision a pendiente", true),
    ("NullifierThresholdAir", "autorizacion delegada y gobernanza delegada", true),
    ("ThresholdAir", "umbral conjunto", false),
    ("AuditAir", "auditoria", true),
    ("BurnAir", "quema", true),
    ("SolvencyAir", "solvencia", false),
    ("DoubleEntryAir", "double_entry", false),
    ("BandaAir", "banda", true),
    ("EdadAir", "edad", true),
    ("CobroPendienteAir", "sobre de cobro", true),
    ("PagoEnCursoAir", "sobre de pago", true),
    ("MerkleAir", "camino de merkle", false),
    ("RefundAir", "apertura del reembolso", true),
    ("RefundAirV2", "apertura del reembolso", true),
    ("CreditClimbAir", "subida de credito", true),
    ("MintClimbAir", "subida de la emision delegada", true),
    ("FrozenClimbAir", "subida de la congelacion delegada", true),
    ("RecoveryClimbAir", "subida de la recuperacion delegada", true),
];

/// Los `.rs` bajo `dir`, con su nombre y su texto, bajando a las subcarpetas.
fn fuentes(dir: &Path) -> Vec<(String, String)> {
    let mut v = Vec::new();
    let mut pendientes = vec![dir.to_path_buf()];
    while let Some(d) = pendientes.pop() {
        let entradas = std::fs::read_dir(&d)
            .unwrap_or_else(|e| panic!("el censo no puede leer {}: {e}", d.display()));
        for entrada in entradas {
            let ruta = entrada.expect("una entrada del directorio").path();
            if ruta.is_dir() {
                pendientes.push(ruta);
            } else if ruta.extension().map_or(false, |x| x == "rs") {
                let nombre = ruta.file_name().expect("un nombre").to_string_lossy().into_owned();
                let texto = std::fs::read_to_string(&ruta)
                    .unwrap_or_else(|e| panic!("el censo no puede leer {}: {e}", ruta.display()));
                v.push((nombre, texto));
            }
        }
    }
    v
}

/// Los identificadores que siguen a cada aparicion de `patron` en `texto`.
fn nombres_tras(texto: &str, patron: &str) -> Vec<String> {
    let mut v = Vec::new();
    let mut resto = texto;
    while let Some(i) = resto.find(patron) {
        resto = &resto[i + patron.len()..];
        let nombre: String = resto
            .trim_start()
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !nombre.is_empty() {
            v.push(nombre);
        }
    }
    v
}

/// El censo del cable, leido de los fuentes y no tecleado: la capa (los `verify` de
/// `crates/zk-ssl/src`, fuera de los instrumentos), el par de umbral (lo que verifica
/// `verify_threshold_pair`) y el kit (las AIR de `crates/zk-ssl-air`). Toda AIR del cable tiene
/// fila; toda fila del cable la verifica alguien; toda fila nombra una AIR que existe. Una AIR
/// nueva en el cable sin fila, o una fila que se queda sin AIR, lo pone rojo.
#[test]
fn el_censo_del_cable_da_fila_a_cada_air_verificada() {
    let zk = Path::new(env!("CARGO_MANIFEST_DIR"));
    let crates = zk.parent().expect("la carpeta crates");
    // Los patrones se parten en dos literales: este fichero no se censa a si mismo.
    let verifica = concat!("verify", "::<");
    let implementa = concat!("impl Air", " for");
    let mut capa = BTreeSet::new();
    let mut hay_par = false;
    for (nombre, texto) in fuentes(&zk.join("src")) {
        if nombre.starts_with("instrumento_") {
            continue;
        }
        capa.extend(nombres_tras(&texto, verifica));
        hay_par |= texto.contains(concat!("verify_threshold", "_pair("));
    }
    let mut par = BTreeSet::new();
    if hay_par {
        let ruta = crates.join("stark-experiment/src/circuit_threshold_single_nullifier.rs");
        let texto = std::fs::read_to_string(&ruta).expect("el circuito del par de umbral");
        par.extend(nombres_tras(&texto, verifica));
    }
    let mut kit = BTreeSet::new();
    for (_, texto) in fuentes(&crates.join("zk-ssl-air/src")) {
        kit.extend(nombres_tras(&texto, implementa));
    }
    let mut arbol = BTreeSet::new();
    for entrada in std::fs::read_dir(crates).expect("la carpeta crates") {
        let src = entrada.expect("un crate").path().join("src");
        if src.is_dir() {
            for (_, texto) in fuentes(&src) {
                arbol.extend(nombres_tras(&texto, implementa));
            }
        }
    }
    let mut cable = BTreeSet::new();
    cable.extend(capa.iter().cloned());
    cable.extend(par.iter().cloned());
    cable.extend(kit.iter().cloned());
    println!(
        "REVELA censo | capa {} + par {} + kit {} = cable {} ; AIR en el arbol {} ; filas {}",
        capa.len(),
        par.len(),
        kit.len(),
        cable.len(),
        arbol.len(),
        FILAS.len()
    );
    let mut mal = Vec::new();
    for air in &cable {
        match FILAS.iter().find(|(n, _, _)| *n == air.as_str()) {
            None => mal.push(format!("{air}: el cable la verifica y la tabla no le da fila")),
            Some((_, fila, false)) => {
                mal.push(format!("{air}: esta en el cable y su fila ({fila}) dice experimental"))
            }
            Some(_) => {}
        }
    }
    for (air, fila, del_cable) in FILAS {
        if !arbol.contains(*air) {
            mal.push(format!("{air} ({fila}): la fila nombra una AIR que no existe"));
        }
        if *del_cable && !cable.contains(*air) {
            mal.push(format!("{air} ({fila}): fila del cable que ya no verifica nadie"));
        }
    }
    assert!(mal.is_empty(), "el censo del cable: {}", mal.join("; "));
}
