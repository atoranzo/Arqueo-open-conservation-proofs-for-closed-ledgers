//! **§718, la canonicidad u64 -> campo, en la capa.**
//!
//! Un `u64` que la capa compone en una hoja entra por `BaseElement::new`, que reduce modulo `p`:
//! `x` y `x + p` dan la misma hoja y la misma raiz. Donde ese `u64` lo lee la capa de su propio
//! almacen, la puerta de integridad compara raices y no lo ve. Estos tests reescriben el almacen
//! (o la instantanea) con el alias de un valor real, uno por campo, y exigen que no abra, con su
//! causa. Sin el arreglo, cada uno abria con las raices iguales (medido: la meta de los
//! pendientes, el saldo con el suministro, y la instantanea). Y la capa no asigna el
//! indice de cuenta que el centinela de la emision es en el campo.
//!
//! Lo mismo para lo que la cabeza firma sin raiz que lo cubra: los parametros de `paramsDigest`
//! (el plazo de reembolso y el tope de custodios, leidos del almacen; el limite y el tope de
//! cuentas, de la configuracion) y los contadores `nextPending` y `nextIndex`. Con `x + p`, el
//! digest de la cabeza era el mismo y la capa aplicaba otra regla.

use crate::store::StoreError;
use crate::tests_support as ts;
use crate::{AccountIndex, LayerError, SovereignLayer, INDICE_DEL_CENTINELA, REFUND_SENDER_NONE};
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::StarkField;

const P: u64 = BaseElement::MODULUS;

fn ruta(nombre: &str) -> String {
    let mut p = std::env::temp_dir();
    p.push(format!("zkssl_canonicidad_{}_{}", nombre, std::process::id()));
    let s = p.to_str().unwrap().to_string();
    ts::quitar_libro(&s);
    s
}

fn abre(ruta: &str) -> Result<SovereignLayer, LayerError> {
    ts::open_retry(
        ruta,
        ts::custodian_root(),
        ts::governance_root(),
        ts::LIMIT,
        ts::MAX_SUPPLY,
        ts::MAX_ACCOUNTS,
    )
}

/// La causa de un `IntegrityFailure`, o el error entero si es otro. `SovereignLayer` no
/// implementa `Debug` a proposito (lleva los saldos), asi que el `Ok` no se imprime.
fn causa(r: Result<SovereignLayer, LayerError>) -> String {
    match r {
        Ok(_) => "ABRE".into(),
        Err(LayerError::Store(StoreError::IntegrityFailure { what })) => what.into(),
        Err(e) => format!("{e:?}"),
    }
}

/// Un envio que se queda en vuelo: deja una posicion viva con su meta `(emisor, nacido)`.
fn envia(l: &mut SovereignLayer, de: AccountIndex, sk: u64, a: AccountIndex, cuanto: u64) {
    let estado = ts::state_of(l, de);
    let receptor = l.public_id_of(a).expect("la cuenta receptora existe");
    let recibo = l
        .send(BaseElement::new(sk), de, &estado, receptor, ts::salt_de(cuanto), cuanto)
        .expect("el envio prueba");
    l.apply_send(&recibo, de, &estado, cuanto).expect("el envio se aplica");
}

/// Un libro persistente con un pago en vuelo de Alice y una emision a pendiente. Devuelve la
/// posicion del pago, la de la emision y el indice de Alice.
fn libro(ruta: &str) -> (u64, u64, AccountIndex) {
    let mut l = abre(ruta).expect("abre");
    let alice = ts::open_and_fund(&mut l, ts::SK_ALICE, 10_000);
    let bob = ts::open_and_fund(&mut l, ts::SK_BOB, 0);
    envia(&mut l, alice, ts::SK_ALICE, bob, 100);
    let pago = l.next_pending - 1;
    assert_eq!(l.pending_meta_of(pago).map(|m| m.0), Some(alice));
    let receptor = l.public_id_of(bob).expect("bob");
    ts::mint_to_pending_delegated(&mut l, receptor, ts::salt_de(7), 50);
    let emision = l.next_pending - 1;
    assert_eq!(l.pending_meta_of(emision).map(|m| m.0), Some(REFUND_SENDER_NONE));
    (pago, emision, alice)
}

fn clave(prefijo: &[u8], x: u64) -> Vec<u8> {
    let mut k = prefijo.to_vec();
    k.extend_from_slice(&x.to_le_bytes());
    k
}

/// Reescribe `(sender, born)` de un `pmeta:` en el almacen en claro con `f`.
fn reescribe_pmeta(ruta: &str, pos: u64, f: impl Fn(u64, u64) -> (u64, u64)) {
    let db = ts::sled_open_retry(ruta);
    let k = clave(b"pmeta:", pos);
    let v = db.get(&k).unwrap().expect("pmeta existe");
    let (s, b) = f(
        u64::from_le_bytes(v[..8].try_into().unwrap()),
        u64::from_le_bytes(v[8..16].try_into().unwrap()),
    );
    let mut nv = s.to_le_bytes().to_vec();
    nv.extend_from_slice(&b.to_le_bytes());
    db.insert(k, nv).unwrap();
    db.flush().unwrap();
}

/// Suma `d` al `u64` little-endian en `[off, off + 8)` del valor de `k`.
fn suma_en(ruta: &str, k: &[u8], off: usize, d: u64) {
    let db = ts::sled_open_retry(ruta);
    let mut v = db.get(k).unwrap().expect("la clave existe").to_vec();
    let x = u64::from_le_bytes(v[off..off + 8].try_into().unwrap());
    v[off..off + 8].copy_from_slice(&x.wrapping_add(d).to_le_bytes());
    db.insert(k, v).unwrap();
    db.flush().unwrap();
}

/// La premisa: en el campo, el centinela de la emision ES el indice `2^32 - 2`, y la hoja de
/// meta no los distingue. Sin esto, los tests del indice no probarian nada.
#[test]
fn el_centinela_en_el_campo_es_el_indice_reservado() {
    assert_eq!(INDICE_DEL_CENTINELA, (1u64 << 32) - 2);
    assert_eq!(BaseElement::new(REFUND_SENDER_NONE).as_int(), INDICE_DEL_CENTINELA);
    for b in [0, 7, 1_000_000] {
        assert_eq!(
            zk_ssl_hash::meta_pendiente_hoja(REFUND_SENDER_NONE, b),
            zk_ssl_hash::meta_pendiente_hoja(INDICE_DEL_CENTINELA, b)
        );
    }
}

/// `open_with_id` no asigna el indice del centinela, ni pedido de frente ni por un primer
/// elemento que reduce a el: cae en el siguiente libre. Falsador: sin el salto, la primera
/// apertura cae en `2^32 - 2` (medido, y se pedia por la boca de `zkssl_openAccount`).
#[test]
fn open_with_id_salta_el_indice_del_centinela() {
    let mut l = ts::new_layer();
    let id = |x: u64| {
        [BaseElement::new(x), BaseElement::new(7), BaseElement::new(8), BaseElement::new(9)]
    };
    let a = l.open_with_id(id(INDICE_DEL_CENTINELA), ts::salt_de(1), ts::salt_de(2)).unwrap();
    let b = l
        .open_with_id(id(INDICE_DEL_CENTINELA + 5 * (1u64 << 32)), ts::salt_de(3), ts::salt_de(4))
        .unwrap();
    assert_eq!(a, INDICE_DEL_CENTINELA + 1, "la primera, en el siguiente");
    assert_eq!(b, 0, "la segunda da la vuelta al arbol, y tampoco cae en el");
    assert!(l.balance_of(INDICE_DEL_CENTINELA).is_none(), "nadie en el indice del centinela");
}

/// `sender + p` en el `pmeta:` de un pago: la hoja es la misma y la raiz tambien. No abre, con
/// su causa. El control, la misma reescritura con el valor de verdad, abre.
#[test]
fn pmeta_sender_alias_no_abre() {
    let r = ruta("pmeta_sender");
    let (pago, _, alice) = libro(&r);
    reescribe_pmeta(&r, pago, |s, b| (s, b));
    assert_eq!(causa(abre(&r)), "ABRE", "el control: la reescritura con lo mismo abre");
    reescribe_pmeta(&r, pago, |s, b| (s + P, b));
    assert_eq!(causa(abre(&r)), "meta de pendientes: emisor fuera de las cuentas", "alice {alice}");
    ts::quitar_libro(&r);
}

/// `born + p` en el `pmeta:` de un pago: bloqueaba el reembolso para siempre
/// (`RefundTooEarly { born: p + .. }`, medido). No abre, con su causa.
#[test]
fn pmeta_born_alias_no_abre() {
    let r = ruta("pmeta_born");
    let (pago, _, _) = libro(&r);
    reescribe_pmeta(&r, pago, |s, b| (s, b + P));
    assert_eq!(causa(abre(&r)), "meta de pendientes: nacimiento no canonico");
    ts::quitar_libro(&r);
}

/// La emision reescrita al indice del centinela: misma hoja, y el reembolso la acreditaba a esa
/// cuenta en vez de des-emitirla (medido hasta la verificacion de la prueba). Sin cuenta en ese
/// indice, no abre, con su causa.
#[test]
fn pmeta_centinela_alias_no_abre() {
    let r = ruta("pmeta_centinela");
    let (_, emision, _) = libro(&r);
    reescribe_pmeta(&r, emision, |_, b| (INDICE_DEL_CENTINELA, b));
    assert_eq!(causa(abre(&r)), "meta de pendientes: el indice del centinela sin cuenta");
    ts::quitar_libro(&r);
}

/// El saldo `+ p` en `acct:` y el suministro `+ p` en `meta:supply`, coordinados: la raiz de
/// cuentas y la conservacion en enteros cuadraban, y abria (medido). Ahora cae el suministro, y
/// con solo el saldo, el saldo (antes caia la conservacion: la raiz era la misma). Y un importe
/// pendiente `+ p`, por su lado.
#[test]
fn almacen_saldo_y_suministro_alias_no_abre() {
    let r = ruta("saldo_suministro");
    let (pago, _, alice) = libro(&r);
    // El saldo va en los bytes 32..40 del registro v3 (`store::record_to_bytes_v3`).
    suma_en(&r, &clave(b"acct:", alice), 32, P);
    suma_en(&r, b"meta:supply", 0, P);
    assert_eq!(causa(abre(&r)), "suministro fuera del tope de la capa", "coordinados");
    suma_en(&r, b"meta:supply", 0, P.wrapping_neg());
    assert_eq!(causa(abre(&r)), "saldo fuera del tope de la capa", "solo el saldo");
    // Se deshace y se prueba el importe pendiente, que no cubre ninguna raiz.
    suma_en(&r, &clave(b"acct:", alice), 32, P.wrapping_neg());
    assert_eq!(causa(abre(&r)), "ABRE", "deshecho, abre");
    suma_en(&r, &clave(b"pamt:", pago), 0, P);
    assert_eq!(causa(abre(&r)), "importe de pendiente fuera del tope de la capa");
    ts::quitar_libro(&r);
}

/// La instantanea: el saldo `+ p` y el suministro `+ p` coordinados importaban (medido). Ahora
/// cae el suministro, y con solo el saldo, el saldo; un tope de emision sobre el de la capa cae
/// como en `open`. Los desplazamientos se buscan en el fichero, no se teclean.
#[test]
fn instantanea_alias_no_importa() {
    let f = format!("{}.snap", ruta("instantanea"));
    let mut l = ts::new_layer();
    ts::open_and_fund(&mut l, ts::SK_ALICE, 1_000_000);
    l.export_snapshot(&f).expect("exportar");
    let bytes = std::fs::read(&f).expect("leer");
    let aguja = 1_000_000u64.to_le_bytes();
    let donde: Vec<usize> =
        (0..bytes.len() - 8).filter(|&i| bytes[i..i + 8] == aguja).collect();
    assert_eq!(donde.len(), 2, "el suministro y el saldo de Alice, y nada mas: {donde:?}");
    let (sup, saldo) = (donde[0], donde[1]);
    let tope_emision = sup - 16; // max_supply, max_accounts, total_supply (cabecera del modulo)
    assert_eq!(bytes[tope_emision..tope_emision + 8], ts::MAX_SUPPLY.to_le_bytes());
    let con = |cambios: &[(usize, u64)]| {
        let mut b = bytes.clone();
        for &(off, x) in cambios {
            b[off..off + 8].copy_from_slice(&x.to_le_bytes());
        }
        std::fs::write(&f, &b).expect("escribir");
        match SovereignLayer::import_snapshot(&f) {
            Ok(_) => "IMPORTA".to_string(),
            Err(LayerError::Store(StoreError::IntegrityFailure { what })) => what.to_string(),
            Err(e) => format!("{e:?}"),
        }
    };
    assert_eq!(con(&[]), "IMPORTA", "el control");
    let alias = 1_000_000 + P;
    assert_eq!(
        con(&[(sup, alias), (saldo, alias)]),
        "suministro de la instantanea fuera del tope de la capa"
    );
    assert_eq!(con(&[(saldo, alias)]), "saldo de la instantanea fuera del tope de la capa");
    let e = con(&[(tope_emision, crate::MAX_SUMINISTRO + 1)]);
    assert!(e.starts_with("SupplyCapExceeded"), "{e}");
    let _ = std::fs::remove_file(&f);
}

/// El digest de la cabeza que la capa firmaria ahora, con lo que no es de la capa a cero.
fn cabeza(l: &SovereignLayer) -> zk_ssl_hash::Digest {
    let cero = zk_ssl_hash::as_digest(0);
    l.epoch_head(cero, 0, cero, 0, cero, 0).digest()
}

/// `meta:refund_ttl + p`: la `T` entra en `paramsDigest`, que la cabeza firma, y ninguna raiz la
/// cubre. La premisa, en memoria: con `T + p` el digest de la cabeza es el mismo, y una
/// des-emision que con `T = 0` se aplica queda bloqueada (`RefundTooEarly`, para siempre). En el
/// almacen, el control abre y el alias no, con su causa. Falsador: sin la cota de `load`, abre.
#[test]
fn meta_refund_ttl_alias_no_abre() {
    let mut l = ts::new_layer();
    let bob = ts::open_and_fund(&mut l, ts::SK_BOB, 0);
    let id = l.public_id_of(bob).expect("bob");
    ts::mint_to_pending_delegated(&mut l, id, ts::salt_de(7), 50);
    let pos = l.next_pending - 1;
    l.refund_ttl = 0;
    let d0 = cabeza(&l);
    l.refund_ttl = P;
    assert_eq!(cabeza(&l), d0, "premisa: T + p firma lo mismo que T");
    let des = l.deissue(pos, id, ts::salt_de(7), 50).expect("la apertura");
    let e = l.apply_deissue(&des).expect_err("con T = p");
    assert!(matches!(e, LayerError::RefundTooEarly { ttl: P, .. }), "{e:?}");
    l.refund_ttl = 0;
    l.apply_deissue(&des).expect("con T = 0, la misma des-emision se aplica");

    let r = ruta("refund_ttl");
    libro(&r);
    suma_en(&r, b"meta:refund_ttl", 0, 0);
    assert_eq!(causa(abre(&r)), "ABRE", "el control");
    suma_en(&r, b"meta:refund_ttl", 0, P);
    assert_eq!(causa(abre(&r)), "plazo de reembolso no canonico");
    ts::quitar_libro(&r);
}

/// `meta:cust_max + p`: el tope de custodios entra en `paramsDigest` y ninguna raiz lo cubre. La
/// premisa, en memoria: con `tope + p` el digest de la cabeza es el mismo, y un cupo agotado deja
/// de estarlo. En el almacen, el control abre y el alias no. Falsador: sin la cota, abre.
#[test]
fn meta_cust_max_alias_no_abre() {
    let mut l = ts::new_layer();
    l.custodian_uses = l.max_custodian_uses;
    let d0 = cabeza(&l);
    assert!(l.consume_custodian_use().is_err(), "premisa: el cupo esta agotado");
    l.max_custodian_uses += P;
    assert_eq!(cabeza(&l), d0, "premisa: tope + p firma lo mismo que tope");
    assert!(l.consume_custodian_use().is_ok(), "y el cupo agotado vuelve a tener sitio");

    let r = ruta("cust_max");
    libro(&r);
    suma_en(&r, b"meta:cust_max", 0, 0);
    assert_eq!(causa(abre(&r)), "ABRE", "el control");
    suma_en(&r, b"meta:cust_max", 0, P);
    assert_eq!(causa(abre(&r)), "tope de custodios no canonico");
    ts::quitar_libro(&r);
}

/// `meta:next_pending + p` y `meta:next_index + p`: los dos van a la cabeza (`nextPending`,
/// `nextIndex`) por `as_digest`, y ninguna raiz los cubre. La premisa, en memoria: el digest de la
/// cabeza es el mismo. En el almacen, cada uno cae con su causa y el control abre. Falsador: sin
/// las cotas, los dos abren.
#[test]
fn meta_contadores_alias_no_abren() {
    let r = ruta("contadores");
    libro(&r);
    {
        let mut l = abre(&r).expect("abre");
        let d0 = cabeza(&l);
        l.next_pending += P;
        l.next_index += P;
        assert_eq!(cabeza(&l), d0, "premisa: los dos alias firman lo mismo");
    }
    let mut dicen = Vec::new();
    for k in [&b"meta:next_pending"[..], &b"meta:next_index"[..]] {
        suma_en(&r, k, 0, 0);
        dicen.push(causa(abre(&r)));
        suma_en(&r, k, 0, P);
        dicen.push(causa(abre(&r)));
        suma_en(&r, k, 0, P.wrapping_neg());
    }
    let deben = [
        "ABRE",
        "contador de pendientes fuera del arbol",
        "ABRE",
        "censo de cuentas fuera del arbol",
    ];
    assert_eq!(dicen, deben, "cada control abre y cada alias cae con su causa");
    assert_eq!(causa(abre(&r)), "ABRE", "deshecho, abre");
    ts::quitar_libro(&r);
}

/// La capa no arranca, no abre ni se ajusta con un parametro de `paramsDigest` que no es
/// canonico: `new` y los dos ajustes entran en panico con su texto, y `open` devuelve el error.
/// El mayor canonico, `p - 1`, pasa en todos. Falsador: sin las cotas, todo pasa.
#[test]
fn parametros_alias_no_arrancan() {
    let panico = |f: &dyn Fn()| -> String {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        match r {
            Ok(()) => "PASA".into(),
            Err(e) => e.downcast_ref::<String>().cloned().unwrap_or_default(),
        }
    };
    let texto = |que: &str, x: u64| {
        format!("{que} = {x} no es canonico: no es menor que p (paramsDigest)")
    };
    let nueva = |limite: u64, cuentas: u64| {
        let _ = SovereignLayer::new(
            ts::custodian_root(),
            ts::governance_root(),
            limite,
            ts::MAX_SUPPLY,
            cuentas,
        );
    };
    let r = ruta("parametros");
    let abre_con = |limite: u64, cuentas: u64| {
        ts::quitar_libro(&r);
        let l = SovereignLayer::open(
            &r,
            ts::custodian_root(),
            ts::governance_root(),
            limite,
            ts::MAX_SUPPLY,
            cuentas,
        );
        match l {
            Ok(_) => "ABRE".to_string(),
            Err(LayerError::Store(StoreError::Malformed(m))) => m,
            Err(e) => format!("{e:?}"),
        }
    };
    let dicen = [
        panico(&|| nueva(ts::LIMIT + P, ts::MAX_ACCOUNTS)),
        panico(&|| nueva(ts::LIMIT, P)),
        panico(&|| nueva(P - 1, P - 1)),
        panico(&|| ts::new_layer().set_refund_ttl(P)),
        panico(&|| ts::new_layer().set_max_custodian_uses(u64::MAX)),
        panico(&|| ts::new_layer().set_refund_ttl(P - 1)),
        panico(&|| ts::new_layer().set_max_custodian_uses(P - 1)),
        abre_con(ts::LIMIT, ts::MAX_ACCOUNTS + P),
        abre_con(ts::LIMIT + P, ts::MAX_ACCOUNTS),
    ];
    ts::quitar_libro(&r);
    let deben = [
        texto("regulatory_limit", ts::LIMIT + P),
        texto("max_accounts", P),
        "PASA".into(),
        texto("refund_ttl", P),
        texto("max_custodian_uses", u64::MAX),
        "PASA".into(),
        "PASA".into(),
        texto("max_accounts", ts::MAX_ACCOUNTS + P),
        texto("regulatory_limit", ts::LIMIT + P),
    ];
    assert_eq!(dicen, deben);
    assert_eq!(abre_con(ts::LIMIT, ts::MAX_ACCOUNTS), "ABRE", "el control");
    ts::quitar_libro(&r);
}

/// La instantanea: el limite y el tope de cuentas `+ p`, y el censo `+ p`, no importan, con su
/// causa; y una cuenta en el indice del centinela tampoco (antes importaba, medido). Los
/// desplazamientos se buscan en el fichero, como en `instantanea_alias_no_importa`.
#[test]
fn instantanea_parametros_y_centinela_no_importan() {
    let f = format!("{}.snap", ruta("instantanea_parametros"));
    let importa = |b: &[u8]| {
        std::fs::write(&f, b).expect("escribir");
        match SovereignLayer::import_snapshot(&f) {
            Ok(_) => "IMPORTA".to_string(),
            Err(LayerError::Store(StoreError::IntegrityFailure { what })) => what.to_string(),
            Err(e) => format!("{e:?}"),
        }
    };
    let mut l = ts::new_layer();
    let alice = ts::open_and_fund(&mut l, ts::SK_ALICE, 1_000_000);
    l.export_snapshot(&f).expect("exportar");
    let bytes = std::fs::read(&f).expect("leer");
    let aguja = 1_000_000u64.to_le_bytes();
    let donde: Vec<usize> =
        (0..bytes.len() - 8).filter(|&i| bytes[i..i + 8] == aguja).collect();
    assert_eq!(donde.len(), 2, "el suministro y el saldo de Alice: {donde:?}");
    let sup = donde[0];
    // regulatory_limit, max_supply, max_accounts, total_supply, recovery_count, freeze_count,
    // gov_change_count, next_index: la cabecera del modulo.
    let (limite, cuentas, censo) = (sup - 24, sup - 8, sup + 32);
    let lee = |off: usize| u64::from_le_bytes(bytes[off..off + 8].try_into().unwrap());
    assert_eq!((lee(limite), lee(cuentas), lee(censo)), (ts::LIMIT, ts::MAX_ACCOUNTS, 1));
    assert_eq!(importa(&bytes), "IMPORTA", "el control");
    let mut dicen: Vec<String> = [limite, cuentas, censo]
        .into_iter()
        .map(|off| {
            let mut b = bytes.clone();
            b[off..off + 8].copy_from_slice(&(lee(off) + P).to_le_bytes());
            importa(&b)
        })
        .collect();

    // La cuenta de Alice, movida en memoria al indice del centinela: la raiz de cuentas es la
    // de la instantanea, asi que solo la cota la para.
    let r = l.records.remove(&alice).expect("alice");
    let hoja = l.accounts.leaf(alice);
    l.accounts.set_leaf(alice, [BaseElement::new(0); 4]);
    l.accounts.set_leaf(INDICE_DEL_CENTINELA, hoja);
    l.records.insert(INDICE_DEL_CENTINELA, r);
    l.export_snapshot(&f).expect("exportar");
    dicen.push(importa(&std::fs::read(&f).expect("leer")));
    let _ = std::fs::remove_file(&f);
    let deben = [
        "limite regulatorio de la instantanea no canonico",
        "tope de cuentas de la instantanea no canonico",
        "censo de cuentas de la instantanea fuera del arbol",
        "cuenta de la instantanea en el indice del centinela",
    ];
    assert_eq!(dicen, deben);
}

/// El ancho del indice de cuenta del juez (`zk_ssl_air::BITS_CUENTA`) es la capacidad del arbol
/// de cuentas de la capa. Si el arbol creciera sin el juez, el juez rechazaria a un emisor real.
#[test]
fn el_indice_de_cuenta_del_juez_es_el_del_arbol() {
    let l = ts::new_layer();
    assert_eq!(1u64 << zk_ssl_air::BITS_CUENTA, l.accounts.capacity());
    assert_eq!(1u64 << zk_ssl_air::BITS_CUENTA, crate::sparse_tree::SparseTree::new().capacity());
}
