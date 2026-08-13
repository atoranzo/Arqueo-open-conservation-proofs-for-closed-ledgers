//! # El MMR de cabezas (§291): «esta cabeza contiene aquella»
//!
//! El eslabon 2 de la nota 83, con su razon escrita alli: **un cliente
//! con una cabeza vieja debe poder verificar que la nueva la EXTIENDE**,
//! sin descargarse el registro. Hoy el encadenado existe entrada a
//! entrada; este modulo da el OBJETO que faltaba — un acumulador de
//! digests de cabeza con prueba de consistencia O(log N) — y sin el, el
//! eslabon 3 (la cofirma del testigo) es imposible.
//!
//! ## Que es y que no es todavia
//!
//! Es la ESTRUCTURA, pura: cima, prueba de inclusion, prueba de
//! consistencia, y sus verificaciones. **La cabeza de epoca NO la
//! compromete aun**: la atadura al formato firmado (la cima dentro del
//! digest, byte de version 3) es el sello (ii) de la familia, decision
//! aparte y rotura declarada. Hasta entonces, nada de lo que viaja
//! cambia.
//!
//! ## La forma, y de donde viene
//!
//! Los algoritmos son los del arbol de historia de RFC 6962 (el de
//! Certificate Transparency): `MTH`, `PATH` y `SUBPROOF`, trasladados
//! sobre las primitivas de la casa — [`mmr_hoja`] y [`mmr_nodo`] de
//! `zk-ssl-hash`, con sus dominios `MMRHOJA1`/`MMRNODO1` en el registro
//! de §286. La separacion hoja/interior es la defensa clasica de segunda
//! preimagen: un nodo interno presentado como hoja compone distinto.
//!
//! ⚠️ **La verificacion es la RECURSION ESPEJO de la generacion**: el
//! verificador recorre exactamente la misma particion `k = mayor
//! potencia de dos < n` y consume el camino posicion a posicion; al
//! final exige `consumido == len`. Un camino con sobras o con faltas no
//! puede pasar, y generador y verificador no pueden desalinearse porque
//! comparten la particion por construccion.
//!
//! ## Quien lo alimentara
//!
//! Las hojas son digests de cabeza. El nodo los tiene en su diario
//! (una linea por latido, con `epochDigest`), asi que la vista se
//! reconstruye como `vista_acuses`: nada nuevo se persiste. Eso llega
//! con el (iii); este modulo no sabe de diarios ni de cables.

use zk_ssl_hash::{mmr_hoja, mmr_nodo, Digest};

/// La mayor potencia de dos ESTRICTAMENTE menor que `n`. Solo para
/// `n >= 2`: es la particion de RFC 6962, compartida por generacion y
/// verificacion.
fn mitad(n: u64) -> u64 {
    debug_assert!(n >= 2);
    let mut k = 1u64;
    while k * 2 < n {
        k *= 2;
    }
    k
}

fn mth(hojas: &[Digest]) -> Digest {
    if hojas.len() == 1 {
        mmr_hoja(hojas[0])
    } else {
        let k = mitad(hojas.len() as u64) as usize;
        mmr_nodo(mth(&hojas[..k]), mth(&hojas[k..]))
    }
}

/// La cima del acumulador sobre estas hojas. `None` si no hay ninguna:
/// una historia vacia no tiene cima que prometer.
pub fn cima(hojas: &[Digest]) -> Option<Digest> {
    if hojas.is_empty() {
        None
    } else {
        Some(mth(hojas))
    }
}

/// El camino de inclusion de la hoja `i` bajo la cima de `hojas`.
pub fn prueba_de_inclusion(hojas: &[Digest], i: u64) -> Option<Vec<Digest>> {
    if i >= hojas.len() as u64 {
        return None;
    }
    let mut camino = Vec::new();
    ruta(hojas, i as usize, &mut camino);
    Some(camino)
}

fn ruta(hojas: &[Digest], i: usize, out: &mut Vec<Digest>) {
    if hojas.len() == 1 {
        return;
    }
    let k = mitad(hojas.len() as u64) as usize;
    if i < k {
        ruta(&hojas[..k], i, out);
        out.push(mth(&hojas[k..]));
    } else {
        ruta(&hojas[k..], i - k, out);
        out.push(mth(&hojas[..k]));
    }
}

/// ¿La hoja `hoja` esta en la posicion `i` bajo `cima`, siendo `n` el
/// total de hojas? El camino se consume ENTERO o no vale.
pub fn verificar_inclusion(
    hoja: Digest,
    i: u64,
    n: u64,
    cima: Digest,
    camino: &[Digest],
) -> bool {
    if n == 0 || i >= n {
        return false;
    }
    match sube(i, n, hoja, camino, 0) {
        Some((raiz, usados)) => usados == camino.len() && raiz == cima,
        None => false,
    }
}

fn sube(i: u64, n: u64, hoja: Digest, camino: &[Digest], pos: usize) -> Option<(Digest, usize)> {
    if n == 1 {
        return Some((mmr_hoja(hoja), pos));
    }
    let k = mitad(n);
    if i < k {
        let (h, p) = sube(i, k, hoja, camino, pos)?;
        let hermano = *camino.get(p)?;
        Some((mmr_nodo(h, hermano), p + 1))
    } else {
        let (h, p) = sube(i - k, n - k, hoja, camino, pos)?;
        let hermano = *camino.get(p)?;
        Some((mmr_nodo(hermano, h), p + 1))
    }
}

/// El camino de consistencia: prueba que la cima de las primeras
/// `viejo` hojas esta CONTENIDA en la cima de todas. `SUBPROOF` de
/// RFC 6962, tal cual.
pub fn prueba_de_consistencia(hojas: &[Digest], viejo: u64) -> Option<Vec<Digest>> {
    let n = hojas.len() as u64;
    if viejo == 0 || viejo > n {
        return None;
    }
    let mut camino = Vec::new();
    subprueba(hojas, viejo, true, &mut camino);
    Some(camino)
}

fn subprueba(hojas: &[Digest], m: u64, borde: bool, out: &mut Vec<Digest>) {
    let n = hojas.len() as u64;
    if m == n {
        if !borde {
            out.push(mth(hojas));
        }
        return;
    }
    let k = mitad(n);
    if m <= k {
        subprueba(&hojas[..k as usize], m, borde, out);
        out.push(mth(&hojas[k as usize..]));
    } else {
        subprueba(&hojas[k as usize..], m - k, false, out);
        out.push(mth(&hojas[..k as usize]));
    }
}

/// ¿`cima_nueva` (de `nuevo` hojas) EXTIENDE a `cima_vieja` (de
/// `viejo`)? El corazon del eslabon 2: quien custodia una cabeza vieja
/// comprueba esto y sabe que la historia no se reescribio por debajo.
pub fn verificar_consistencia(
    cima_vieja: Digest,
    viejo: u64,
    cima_nueva: Digest,
    nuevo: u64,
    camino: &[Digest],
) -> bool {
    if viejo == 0 || nuevo == 0 || viejo > nuevo {
        return false;
    }
    if viejo == nuevo {
        return camino.is_empty() && cima_vieja == cima_nueva;
    }
    match recompone(viejo, nuevo, true, cima_vieja, camino, 0) {
        Some((vieja, nueva, usados)) => {
            usados == camino.len() && vieja == cima_vieja && nueva == cima_nueva
        }
        None => false,
    }
}

fn recompone(
    m: u64,
    n: u64,
    borde: bool,
    cima_vieja: Digest,
    camino: &[Digest],
    pos: usize,
) -> Option<(Digest, Digest, usize)> {
    if m == n {
        return if borde {
            Some((cima_vieja, cima_vieja, pos))
        } else {
            let h = *camino.get(pos)?;
            Some((h, h, pos + 1))
        };
    }
    let k = mitad(n);
    if m <= k {
        let (vieja, nueva, p) = recompone(m, k, borde, cima_vieja, camino, pos)?;
        let derecha = *camino.get(p)?;
        Some((vieja, mmr_nodo(nueva, derecha), p + 1))
    } else {
        let (vieja, nueva, p) = recompone(m - k, n - k, false, cima_vieja, camino, pos)?;
        let izquierda = *camino.get(p)?;
        Some((mmr_nodo(izquierda, vieja), mmr_nodo(izquierda, nueva), p + 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zk_ssl_hash::as_digest;

    fn hojas(n: u64) -> Vec<Digest> {
        (1..=n).map(as_digest).collect()
    }

    #[test]
    fn la_cima_de_una_hoja_es_la_hoja_etiquetada() {
        let d = as_digest(7);
        assert_eq!(cima(&[d]), Some(mmr_hoja(d)));
        assert_ne!(mmr_hoja(d), d, "la etiqueta de hoja tiene que separar");
        assert_eq!(cima(&[]), None, "una historia vacia no tiene cima");
    }

    #[test]
    fn un_interior_no_puede_hacerse_pasar_por_hoja() {
        // La defensa clasica de segunda preimagen: si un nodo interno se
        // presenta como hoja, la etiqueta lo compone distinto y la cima
        // no casa. Sin los dos dominios, esta igualdad seria posible.
        let h = hojas(2);
        let interior = cima(&h).unwrap();
        assert_ne!(cima(&[interior]).unwrap(), interior);
        assert_ne!(mmr_nodo(h[0], h[1]), mmr_hoja(mmr_nodo(h[0], h[1])));
    }

    #[test]
    fn una_extension_honesta_prueba_su_consistencia() {
        for (m, n) in [(1, 2), (2, 3), (3, 7), (4, 8), (7, 13), (13, 64)] {
            let todas = hojas(n);
            let vieja = cima(&todas[..m as usize]).unwrap();
            let nueva = cima(&todas).unwrap();
            let camino = prueba_de_consistencia(&todas, m).unwrap();
            assert!(
                verificar_consistencia(vieja, m, nueva, n, &camino),
                "consistencia {m}->{n} tenia que verificar"
            );
        }
        // Y el caso identidad: misma historia, camino vacio.
        let h = hojas(5);
        let c = cima(&h).unwrap();
        assert!(verificar_consistencia(c, 5, c, 5, &[]));
    }

    #[test]
    fn una_historia_bifurcada_no_prueba_consistencia() {
        // El operador reescribe la hoja 0 y presenta su cima nueva con un
        // camino perfectamente formado SOBRE LA HISTORIA REESCRITA: la
        // cima vieja del cliente no aparece y la verificacion cae.
        let m = 3u64;
        let honestas = hojas(7);
        let mut reescritas = honestas.clone();
        reescritas[0] = as_digest(999);
        let vieja_del_cliente = cima(&honestas[..m as usize]).unwrap();
        let nueva_reescrita = cima(&reescritas).unwrap();
        let camino = prueba_de_consistencia(&reescritas, m).unwrap();
        assert!(!verificar_consistencia(
            vieja_del_cliente, m, nueva_reescrita, 7, &camino
        ));
    }

    #[test]
    fn una_historia_recortada_no_extiende() {
        let todas = hojas(5);
        let corta = cima(&todas[..3]).unwrap();
        let larga = cima(&todas).unwrap();
        // Hacia atras no hay consistencia que valga.
        assert!(!verificar_consistencia(larga, 5, corta, 3, &[]));
        let camino = prueba_de_consistencia(&todas, 3).unwrap();
        assert!(!verificar_consistencia(larga, 5, corta, 3, &camino));
        // Y con el mismo tamano, dos cimas distintas no son la misma.
        assert!(!verificar_consistencia(corta, 3, larga, 3, &[]));
    }

    #[test]
    fn el_camino_no_admite_sobras_ni_faltas() {
        let todas = hojas(7);
        let vieja = cima(&todas[..3]).unwrap();
        let nueva = cima(&todas).unwrap();
        let camino = prueba_de_consistencia(&todas, 3).unwrap();
        assert!(verificar_consistencia(vieja, 3, nueva, 7, &camino));
        let mut sobra = camino.clone();
        sobra.push(as_digest(1));
        assert!(!verificar_consistencia(vieja, 3, nueva, 7, &sobra));
        let falta = &camino[..camino.len() - 1];
        assert!(!verificar_consistencia(vieja, 3, nueva, 7, falta));
    }

    #[test]
    fn la_inclusion_va_y_la_adulterada_no() {
        let todas = hojas(6);
        let c = cima(&todas).unwrap();
        for i in 0..6u64 {
            let camino = prueba_de_inclusion(&todas, i).unwrap();
            assert!(
                verificar_inclusion(todas[i as usize], i, 6, c, &camino),
                "la hoja {i} tenia que estar"
            );
            assert!(
                !verificar_inclusion(as_digest(999), i, 6, c, &camino),
                "una hoja adulterada no puede estar en {i}"
            );
        }
        // Un camino de una posicion no vale para otra.
        let camino_2 = prueba_de_inclusion(&todas, 2).unwrap();
        assert!(!verificar_inclusion(todas[3], 3, 6, c, &camino_2));
        // Y fuera de rango, no.
        assert!(prueba_de_inclusion(&todas, 6).is_none());
        assert!(!verificar_inclusion(todas[0], 6, 6, c, &[]));
    }
}
