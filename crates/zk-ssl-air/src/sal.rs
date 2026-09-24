//! **El compromiso con SAL** (RFC-0009, E3b-0, D-X): [`MerkleConSal`], un `VectorCommitment` que
//! sala cada hoja -`merge(item, sal)`- con una sal fresca de la entropia del sistema y que lleva
//! la sal en la apertura: el verificador la recompone sin conocer ninguna semilla.
//!
//! Es el `VC` que los probadores con fila en la tabla de D-B y el kit recibiran cuando E3b
//! encienda la ocultacion (D-Z). Hoy NADIE lo declara como `VC`: entra como tipo, con sus
//! testigos, y no mueve un byte de ninguna prueba; la foto de D-R sigue midiendo winterfell byte
//! a byte.
//!
//! Traido del spike-b-p4r3 (`spike/src/main.rs`, `145e83ecea366126`, lineas 77-209, leidas por
//! el PASTE-E3b-M): el spike sacaba la sal de un SplitMix64 sembrado por una estatica de proceso
//! porque necesitaba repetir bytes; aqui sale de `rand_core::OsRng` (`getrandom`), que es lo que
//! el propio spike declaraba para produccion y lo que D-J promete. Consecuencia, dicha: una
//! prueba con sal NO se reproduce byte a byte, ni en tests; se vigila por lo que verifica, lo que
//! rechaza y lo que pesa (D-X).
//!
//! Por que un solo tipo para las dos orillas: `VectorCommitment::new(items)` llama a
//! `with_options(items, Options::default())` y no admite semilla, y `winter-fri` -que no se
//! bifurca- construye sus capas con `V::new`; la sal solo puede nacer dentro del tipo. Es la
//! excepcion medida al <<nada de lo que hace falta para PRODUCIRLA>> de este crate: el kit compila
//! `with_options` y no lo llama; `winter-prover` sigue fuera de su clausura.
//!
//! Lo que cuesta, medido en el spike (SPIKE-B-P4 r1; S6 del PASTE-E3b-M): 32 bytes por hoja
//! abierta, 6.567 por prueba con 42 consultas (D-I). El testigo de abajo lo fija en 32 por hoja.
//!
//! `open` y `open_many` devuelven la HOJA salada, no el item: es lo que el arbol de debajo guarda,
//! y el nucleo no usa ese valor -manda las filas aparte y el verificador las vuelve a hashear-.
//! Medido en el spike, donde B1 y E1..E4 verifican asi; el testigo `una_apertura_...` lo exige.

use rand_core::{OsRng, RngCore};
use winter_crypto::{Hasher, MerkleTree, VectorCommitment};
use winter_verifier::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable};

/// Compromiso con sal: cada hoja es `merge(item, sal)` y la sal viaja en la apertura.
pub struct MerkleConSal<H: Hasher> {
    arbol: MerkleTree<H>,
    sales: Vec<H::Digest>,
}

/// Una apertura: el camino del arbol de debajo y la sal de esa hoja.
pub struct UnaConSal<H: Hasher> {
    camino: <MerkleTree<H> as VectorCommitment<H>>::Proof,
    sal: H::Digest,
}

/// Un lote de aperturas: el lote del arbol de debajo y las sales, una por hoja abierta y en su
/// orden.
pub struct VariasConSal<H: Hasher> {
    lote: <MerkleTree<H> as VectorCommitment<H>>::MultiProof,
    sales: Vec<H::Digest>,
}

/// Por que no verifica: lo que dijo el arbol de debajo, o un lote con un numero de sales distinto
/// del de items.
#[derive(Debug)]
pub enum ErrorSal<X> {
    /// lo que dijo `MerkleTree`
    Arbol(X),
    /// `verify_many` con tantas sales como items o nada: falla cerrado
    Longitud,
}

impl<H: Hasher> Clone for UnaConSal<H> {
    fn clone(&self) -> Self {
        Self { camino: self.camino.clone(), sal: self.sal.clone() }
    }
}

impl<H: Hasher> Serializable for UnaConSal<H> {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.camino.write_into(target);
        self.sal.write_into(target);
    }
}

impl<H: Hasher> Deserializable for UnaConSal<H> {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let camino =
            <<MerkleTree<H> as VectorCommitment<H>>::Proof as Deserializable>::read_from(source)?;
        let sal = <H::Digest as Deserializable>::read_from(source)?;
        Ok(Self { camino, sal })
    }
}

impl<H: Hasher> Serializable for VariasConSal<H> {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.lote.write_into(target);
        target.write_u32(self.sales.len() as u32);
        for sal in &self.sales {
            sal.write_into(target);
        }
    }
}

impl<H: Hasher> Deserializable for VariasConSal<H> {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let lote = leer_lote::<H, R>(source)?;
        let n = source.read_u32()? as usize;
        // falla cerrado antes de reservar: no puede haber mas hojas abiertas que hojas
        let tope = <MerkleTree<H> as VectorCommitment<H>>::get_multiproof_domain_len(&lote);
        if n > tope {
            return Err(DeserializationError::InvalidValue(format!(
                "un lote con {n} sales sobre un arbol de {tope} hojas"
            )));
        }
        let mut sales = Vec::with_capacity(n);
        for _ in 0..n {
            sales.push(<H::Digest as Deserializable>::read_from(source)?);
        }
        Ok(Self { lote, sales })
    }
}

/// El lote del arbol de debajo, leido de sus bytes.
fn leer_lote<H: Hasher, R: ByteReader>(
    source: &mut R,
) -> Result<<MerkleTree<H> as VectorCommitment<H>>::MultiProof, DeserializationError> {
    <<MerkleTree<H> as VectorCommitment<H>>::MultiProof as Deserializable>::read_from(source)
}

/// Las hojas saladas: `merge(item, sal)`, item a item.
fn con_sal<H: Hasher>(items: &[H::Digest], sales: &[H::Digest]) -> Vec<H::Digest> {
    items.iter().zip(sales).map(|(item, sal)| H::merge(&[item.clone(), sal.clone()])).collect()
}

/// E3b2-M3: una semilla fresca de 64 bits, de la misma entropia que la sal.
pub fn semilla() -> u64 {
    let mut bytes = [0u8; 8];
    OsRng.fill_bytes(&mut bytes);
    u64::from_le_bytes(bytes)
}

/// Una sal fresca: 32 bytes de la entropia del sistema, pasados por `H`. Aqui, y solo aqui, este
/// crate toma azar; el kit lo compila y nunca lo llama (D-X).
fn sal<H: Hasher>() -> H::Digest {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    H::hash(&bytes)
}

impl<H: Hasher> VectorCommitment<H> for MerkleConSal<H> {
    type Options = ();
    type Proof = UnaConSal<H>;
    type MultiProof = VariasConSal<H>;
    type Error = ErrorSal<<MerkleTree<H> as VectorCommitment<H>>::Error>;

    fn with_options(items: Vec<H::Digest>, _options: Self::Options) -> Result<Self, Self::Error> {
        let sales: Vec<H::Digest> = (0..items.len()).map(|_| sal::<H>()).collect();
        let hojas = con_sal::<H>(&items, &sales);
        let arbol = <MerkleTree<H> as VectorCommitment<H>>::new(hojas).map_err(ErrorSal::Arbol)?;
        Ok(Self { arbol, sales })
    }

    fn commitment(&self) -> H::Digest {
        self.arbol.commitment()
    }

    fn domain_len(&self) -> usize {
        self.arbol.domain_len()
    }

    fn get_proof_domain_len(proof: &Self::Proof) -> usize {
        <MerkleTree<H> as VectorCommitment<H>>::get_proof_domain_len(&proof.camino)
    }

    fn get_multiproof_domain_len(proof: &Self::MultiProof) -> usize {
        <MerkleTree<H> as VectorCommitment<H>>::get_multiproof_domain_len(&proof.lote)
    }

    fn open(&self, index: usize) -> Result<(H::Digest, Self::Proof), Self::Error> {
        let (hoja, camino) = self.arbol.open(index).map_err(ErrorSal::Arbol)?;
        Ok((hoja, UnaConSal { camino, sal: self.sales[index].clone() }))
    }

    fn open_many(
        &self,
        indexes: &[usize],
    ) -> Result<(Vec<H::Digest>, Self::MultiProof), Self::Error> {
        let (hojas, lote) = self.arbol.open_many(indexes).map_err(ErrorSal::Arbol)?;
        let sales = indexes.iter().map(|&i| self.sales[i].clone()).collect();
        Ok((hojas, VariasConSal { lote, sales }))
    }

    fn verify(
        commitment: H::Digest,
        index: usize,
        item: H::Digest,
        proof: &Self::Proof,
    ) -> Result<(), Self::Error> {
        let hoja = H::merge(&[item, proof.sal.clone()]);
        <MerkleTree<H> as VectorCommitment<H>>::verify(commitment, index, hoja, &proof.camino)
            .map_err(ErrorSal::Arbol)
    }

    fn verify_many(
        commitment: H::Digest,
        indexes: &[usize],
        items: &[H::Digest],
        proof: &Self::MultiProof,
    ) -> Result<(), Self::Error> {
        if items.len() != proof.sales.len() {
            return Err(ErrorSal::Longitud);
        }
        let hojas = con_sal::<H>(items, &proof.sales);
        <MerkleTree<H> as VectorCommitment<H>>::verify_many(
            commitment,
            indexes,
            &hojas,
            &proof.lote,
        )
        .map_err(ErrorSal::Arbol)
    }
}

// LOS TESTIGOS (RFC-0009 E3b-0): lo que la sal tiene que hacer, y lo que tiene que rechazar.
// Ningun STARK: ocho hojas de Blake3 sobre f64, en depuracion y en release.
#[cfg(test)]
mod tests {
    use super::*;
    use winter_crypto::hashers::Blake3_256;
    use winter_math::fields::f64::BaseElement;
    use winter_verifier::SliceReader;

    type Blake3 = Blake3_256<BaseElement>;
    type Digest = <Blake3 as Hasher>::Digest;
    type ConSal = MerkleConSal<Blake3>;
    type Lisa = MerkleTree<Blake3>;

    /// Ocho items distintos, deterministas: los hashes de 0..8.
    fn items() -> Vec<Digest> {
        (0u64..8).map(|i| Blake3::hash(&i.to_le_bytes())).collect()
    }

    fn con_sal() -> (Vec<Digest>, ConSal) {
        let it = items();
        let c = ConSal::new(it.clone()).expect("el compromiso con sal sobre ocho hojas");
        (it, c)
    }

    fn lisa() -> Lisa {
        <Lisa as VectorCommitment<Blake3>>::new(items()).expect("el arbol sin sal")
    }

    #[test]
    fn una_apertura_verifica_con_el_item_y_va_y_vuelve_en_bytes() {
        let (it, c) = con_sal();
        let (hoja, prueba) = c.open(5).expect("abrir la hoja 5");
        // la hoja que devuelve es la salada, no el item; el item es lo que verifica
        let esperada = Blake3::merge(&[it[5].clone(), prueba.sal.clone()]);
        assert_eq!(hoja, esperada, "open devuelve la hoja salada");
        assert_ne!(hoja, it[5], "la hoja salada no es el item");
        ConSal::verify(c.commitment(), 5, it[5].clone(), &prueba)
            .expect("el item verifica con su sal");
        assert_eq!(ConSal::get_proof_domain_len(&prueba), 8);
        assert_eq!(c.domain_len(), 8);
        let bytes = prueba.to_bytes();
        let leida = UnaConSal::<Blake3>::read_from(&mut SliceReader::new(&bytes))
            .expect("la apertura se lee de sus bytes");
        ConSal::verify(c.commitment(), 5, it[5].clone(), &leida).expect("y sigue verificando");
    }

    #[test]
    fn muchas_aperturas_verifican_y_van_y_vuelven_en_bytes() {
        let (it, c) = con_sal();
        let idx = [1usize, 3, 6];
        let (_, lote) = c.open_many(&idx).expect("abrir tres hojas");
        let abiertos = [it[1].clone(), it[3].clone(), it[6].clone()];
        ConSal::verify_many(c.commitment(), &idx, &abiertos, &lote).expect("las tres verifican");
        assert_eq!(ConSal::get_multiproof_domain_len(&lote), 8);
        let bytes = lote.to_bytes();
        let leido = VariasConSal::<Blake3>::read_from(&mut SliceReader::new(&bytes))
            .expect("el lote se lee de sus bytes");
        assert_eq!(leido.sales.len(), 3, "las tres sales viajan en el lote");
        ConSal::verify_many(c.commitment(), &idx, &abiertos, &leido).expect("y sigue verificando");
    }

    #[test]
    fn la_sal_mueve_la_raiz_y_dos_compromisos_de_lo_mismo_difieren() {
        let (_, a) = con_sal();
        let (_, b) = con_sal();
        assert_ne!(a.commitment(), b.commitment(), "dos sales, dos raices");
        assert_ne!(a.commitment(), lisa().commitment(), "con sal no es la raiz sin sal");
        // y ninguna sal se repite dentro de un compromiso
        for (i, x) in a.sales.iter().enumerate() {
            for y in &a.sales[i + 1..] {
                assert_ne!(x, y, "dos hojas con la misma sal");
            }
        }
    }

    #[test]
    fn una_sal_ajena_o_un_item_ajeno_rechazan() {
        let (it, c) = con_sal();
        let (_, prueba) = c.open(2).expect("abrir la hoja 2");
        let otro = ConSal::verify(c.commitment(), 2, it[3].clone(), &prueba);
        assert!(matches!(otro, Err(ErrorSal::Arbol(_))), "otro item con la misma sal");
        let ajena = UnaConSal { camino: prueba.camino.clone(), sal: sal::<Blake3>() };
        let con_ajena = ConSal::verify(c.commitment(), 2, it[2].clone(), &ajena);
        assert!(matches!(con_ajena, Err(ErrorSal::Arbol(_))), "el item con una sal ajena");
        let idx = [0usize, 4];
        let (_, mut lote) = c.open_many(&idx).expect("abrir dos hojas");
        lote.sales[1] = sal::<Blake3>();
        assert!(matches!(
            ConSal::verify_many(c.commitment(), &idx, &[it[0].clone(), it[4].clone()], &lote),
            Err(ErrorSal::Arbol(_))
        ));
    }

    #[test]
    fn una_sal_de_menos_falla_cerrada() {
        let (it, c) = con_sal();
        let idx = [0usize, 4, 7];
        let (_, mut lote) = c.open_many(&idx).expect("abrir tres hojas");
        lote.sales.pop();
        assert!(matches!(
            ConSal::verify_many(
                c.commitment(),
                &idx,
                &[it[0].clone(), it[4].clone(), it[7].clone()],
                &lote
            ),
            Err(ErrorSal::Longitud)
        ));
    }

    #[test]
    fn la_sal_pesa_treinta_y_dos_bytes_por_hoja_abierta() {
        let (_, c) = con_sal();
        let l = lisa();
        let una = c.open(6).expect("abrir la 6").1.to_bytes().len();
        let una_lisa = l.open(6).expect("abrir la 6 sin sal").1.to_bytes().len();
        assert_eq!(una, una_lisa + 32, "una apertura: el camino y 32 bytes de sal");
        let idx = [1usize, 2, 5, 7];
        let varias = c.open_many(&idx).expect("abrir cuatro").1.to_bytes().len();
        let varias_lisa = l.open_many(&idx).expect("abrir cuatro sin sal").1.to_bytes().len();
        assert_eq!(varias, varias_lisa + 4 + 4 * 32, "un lote: la cuenta y 32 bytes por hoja");
    }

    #[test]
    fn unos_bytes_cortos_o_un_lote_imposible_no_se_leen() {
        let (_, c) = con_sal();
        let bytes = c.open(1).expect("abrir la 1").1.to_bytes();
        let cortos = &bytes[..bytes.len() - 1];
        assert!(UnaConSal::<Blake3>::read_from(&mut SliceReader::new(cortos)).is_err());
        let idx = [2usize, 3];
        let (_, lote) = c.open_many(&idx).expect("abrir dos");
        let mut bytes = lote.to_bytes();
        // la cuenta de sales va justo tras el lote del arbol: la hinchamos por encima de las hojas
        let n = bytes.len() - 2 * 32 - 4;
        bytes[n..n + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(
            VariasConSal::<Blake3>::read_from(&mut SliceReader::new(&bytes)),
            Err(DeserializationError::InvalidValue(_))
        ));
    }
}
