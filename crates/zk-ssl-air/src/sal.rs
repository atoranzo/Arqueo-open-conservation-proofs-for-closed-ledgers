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
use winter_crypto::{BatchMerkleProof, Hasher, MerkleTree, VectorCommitment};
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
    /// `verify_many` con tantas sales como items, y tantos items como indices, o nada: falla
    /// cerrado
    Longitud,
    /// SEG-03b: el lote trae nodos que la reconstruccion de la raiz no consume
    NoCanonico,
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
        // §575: y no mas sales que bytes: con un `depth` mentido el tope no acota nada
        caben(source, n)?;
        let mut sales = Vec::with_capacity(n);
        for _ in 0..n {
            sales.push(<H::Digest as Deserializable>::read_from(source)?);
        }
        Ok(Self { lote, sales })
    }
}

/// §711: la causa con nombre de un lote cuya profundidad no cabe. Se lee como las de
/// `winter_air::causa`, y la segunda implementacion la repite.
pub const LOTE_FUERA_DE_RANGO: &str = "prueba mal formada: lote fuera de rango";

/// §711: la profundidad mayor de un lote que se lee: un dominio de 2^31 hojas, el mayor que admite
/// `Context::new` (`u32::MAX`).
const PROFUNDIDAD_MAXIMA: u8 = 31;

/// §575: ¿caben `n` elementos -de al menos un byte cada uno- en lo que queda? Falla cerrado ANTES
/// de reservar, con el mismo `UnexpectedEOF` que daria quedarse sin bytes. `n` se acota primero: el
/// `check_eor` de `SliceReader` suma `pos + n`, y con un `n` gigante esa suma da la vuelta.
fn caben<R: ByteReader>(source: &R, n: usize) -> Result<(), DeserializationError> {
    if n > isize::MAX as usize {
        return Err(DeserializationError::UnexpectedEOF);
    }
    source.check_eor(n)
}

/// El lote del arbol de debajo, leido de sus bytes: el formato de `BatchMerkleProof` de
/// winter-crypto 0.13.1 -`depth`, el numero de vectores de nodos y cada vector con su longitud-,
/// pero con las cuentas ACOTADAS (§575). Aquel reserva cada cuenta que lee sin mirar si sus bytes
/// existen, y una prueba malformada abortaba el proceso pidiendo cientos de GB. Las capas FRI
/// tambien pasan por aqui: `winter-fri`, que no se bifurca, lee su lote con el `MultiProof` de este
/// compromiso.
fn leer_lote<H: Hasher, R: ByteReader>(
    source: &mut R,
) -> Result<<MerkleTree<H> as VectorCommitment<H>>::MultiProof, DeserializationError> {
    let depth = source.read_u8()?;
    // §711: `get_multiproof_domain_len` de winter-crypto 0.13.1 desplaza `1 << depth`, y con 64 o
    // mas -32 o mas con `usize` de 32 bits, el kit en WebAssembly- desborda: panico en release
    // desde el §669, dentro de `VariasConSal::read_from`, que lo llama antes de comparar el
    // dominio. Un dominio de la casa cabe en 32 bits (`Context::new`): su arbol, en 31 niveles.
    if depth > PROFUNDIDAD_MAXIMA {
        return Err(DeserializationError::InvalidValue(format!(
            "{LOTE_FUERA_DE_RANGO}: un lote de {depth} niveles; caben {PROFUNDIDAD_MAXIMA}"
        )));
    }
    // §715 (la maleabilidad de los bytes de la prueba): cada cuenta en su escritura
    // MINIMA. `winter-fri`, sin bifurcar, lee el lote de cada capa con un `SliceReader`, cuyo
    // `read_usize` acepta el mismo valor escrito en mas bytes: la prueba verificaba igual, con
    // otros bytes y otra huella. La regla es la del lector del fork, una sola.
    let vectores = winter_air::proof::leer_usize_canonico(source)?;
    caben(source, vectores)?;
    let mut nodes = Vec::with_capacity(vectores);
    for _ in 0..vectores {
        let k = winter_air::proof::leer_usize_canonico(source)?;
        caben(source, k)?;
        let mut digests = Vec::with_capacity(k);
        for _ in 0..k {
            digests.push(<H::Digest as Deserializable>::read_from(source)?);
        }
        nodes.push(digests);
    }
    Ok(BatchMerkleProof { nodes, depth })
}

/// Las hojas saladas: `merge(item, sal)`, item a item.
fn con_sal<H: Hasher>(items: &[H::Digest], sales: &[H::Digest]) -> Vec<H::Digest> {
    items.iter().zip(sales).map(|(item, sal)| H::merge(&[item.clone(), sal.clone()])).collect()
}

/// E3b2-M3: una semilla fresca de 32 bytes, la misma entropia que la sal.
///
/// ⚠️ §652 (CRIPTO-01): hasta aqui este comentario decia «de 64 bits, de la misma entropia que la
/// sal» y devolvia un `u64` de 8 bytes de OsRng, mientras la sal toma 32. La equivalencia era
/// falsa, y nadie la contrasto: con 2^64 semillas posibles las filas aleatorias de la ocultacion se
/// recuperaban por fuerza bruta y, con ellas, cualquier columna constante del testigo.
pub fn semilla() -> [u8; 32] {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes
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
        // §715: y tantos items como indices. `get_root` de winter-crypto 0.13.1 recorre
        // los indices y no mira si sobran hojas: una fila de la traza, o un grupo de una capa FRI,
        // con su sal de mas, verificaba con otros bytes y otra huella. El canal del fork lo exige
        // tambien (`winter-verifier/src/channel.rs`); aqui, para todo el que juzgue con sal.
        if items.len() != indexes.len() {
            return Err(ErrorSal::Longitud);
        }
        let hojas = con_sal::<H>(items, &proof.sales);
        <MerkleTree<H> as VectorCommitment<H>>::verify_many(
            commitment,
            indexes,
            &hojas,
            &proof.lote,
        )
        .map_err(ErrorSal::Arbol)?;
        // SEG-03b: la raiz cuadra, pero `get_root` de winter-crypto 0.13.1 no mira si sobran
        // nodos: un digest de mas al final de un vector del lote no se lee, la prueba verifica
        // igual y sus bytes -y su huella- son otros. Eso deshacia el §653 por dentro y dejaba
        // reenviar un recibo ya aplicado con otra huella (§654). Una prueba son sus bytes: cada
        // vector tiene que traer exactamente los nodos que la reconstruccion consume.
        match nodos_consumidos(indexes, proof.lote.depth as usize) {
            Some(n)
                if n.len() == proof.lote.nodes.len()
                    && n.iter().zip(&proof.lote.nodes).all(|(k, v)| *k == v.len()) =>
            {
                Ok(())
            }
            _ => Err(ErrorSal::NoCanonico),
        }
    }
}

/// SEG-03b: cuantos nodos consume de cada vector del lote la reconstruccion de la raiz de
/// `BatchMerkleProof::get_root` (winter-crypto 0.13.1, `merkle/proofs.rs`), contados sobre los
/// indices solos y con su mismo recorrido: los mismos indices normalizados, el mismo puntero por
/// POSICION en cada nivel y la misma regla de hermanos. Se llama solo despues de que la raiz
/// cuadre, asi que lo imposible ya lo rechazo aquella; aqui devuelve `None`.
fn nodos_consumidos(indexes: &[usize], depth: usize) -> Option<Vec<usize>> {
    use std::collections::BTreeSet;
    let hojas: BTreeSet<usize> = indexes.iter().copied().collect();
    let normal: BTreeSet<usize> = indexes.iter().map(|i| i - (i & 1)).collect();
    let offset = 1usize.checked_shl(depth as u32)?;
    let mut punteros = Vec::with_capacity(normal.len());
    let mut siguientes = Vec::with_capacity(normal.len());
    for &i in &normal {
        let pareja = hojas.contains(&i) && hojas.contains(&(i + 1));
        punteros.push(if pareja { 0 } else { 1 });
        siguientes.push((offset + i) >> 1);
    }
    for _ in 1..depth {
        let nivel = std::mem::take(&mut siguientes);
        let mut i = 0;
        while i < nivel.len() {
            if i + 1 < nivel.len() && nivel[i + 1] == (nivel[i] ^ 1) {
                i += 1;
            } else {
                *punteros.get_mut(i)? += 1;
            }
            siguientes.push(nivel[i] >> 1);
            i += 1;
        }
    }
    Some(punteros)
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

    /// SEG-03b: un nodo de mas en cualquier vector del lote se rechazaba solo si movia la raiz, y
    /// al final de un vector no la mueve: `get_root` no lo lee. Ahora falla cerrado, en todas las
    /// formas de abrir (hojas sueltas, una pareja de hermanas, todas), y el lote honesto pasa.
    #[test]
    fn un_nodo_de_mas_en_el_lote_no_verifica() {
        let (it, c) = con_sal();
        let formas = [vec![1usize, 3, 6], vec![0, 1], vec![2, 3, 4], (0..8).collect(), vec![7]];
        for idx in formas {
            let (_, lote) = c.open_many(&idx).expect("abrir");
            let abiertos: Vec<_> = idx.iter().map(|&k| it[k].clone()).collect();
            ConSal::verify_many(c.commitment(), &idx, &abiertos, &lote).expect("el honesto pasa");
            for v in 0..lote.lote.nodes.len() {
                let mut maleado =
                    VariasConSal::<Blake3>::read_from(&mut SliceReader::new(&lote.to_bytes()))
                        .expect("el lote se lee de sus bytes");
                let extra = maleado.lote.nodes[v].first().cloned().unwrap_or_else(|| it[0].clone());
                maleado.lote.nodes[v].push(extra);
                assert!(
                    matches!(
                        ConSal::verify_many(c.commitment(), &idx, &abiertos, &maleado),
                        Err(ErrorSal::NoCanonico)
                    ),
                    "un nodo de mas en el vector {v} de {idx:?} tiene que fallar cerrado"
                );
            }
        }
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

    // ── §575 · una cuenta que no cabe en los bytes falla CERRADA, sin reservar ──
    // Antes de este sello, cualquiera de estos cuatro pedia al sistema cientos de GB y el proceso
    // ABORTABA -una reserva que falla no es un panico-: esta suite no llegaba al final. Un testigo
    // por sitio que lee una cuenta.

    /// Los bytes que escribe `f`, con el `ByteWriter` de winter.
    fn bytes_de(f: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
        let mut b = Vec::new();
        f(&mut b);
        b
    }

    fn es_eof<T>(r: Result<T, DeserializationError>) -> bool {
        matches!(r, Err(DeserializationError::UnexpectedEOF))
    }

    #[test]
    fn un_lote_que_declara_mas_vectores_que_bytes_falla_cerrado_sin_reservar() {
        let b = bytes_de(|b| {
            b.write_u8(3);
            b.write_usize(1 << 40);
        });
        assert!(es_eof(VariasConSal::<Blake3>::read_from(&mut SliceReader::new(&b))));
    }

    #[test]
    fn un_vector_de_nodos_con_una_longitud_gigante_falla_cerrado_sin_reservar() {
        let b = bytes_de(|b| {
            b.write_u8(3);
            b.write_usize(1);
            b.write_usize(1 << 40);
        });
        assert!(es_eof(VariasConSal::<Blake3>::read_from(&mut SliceReader::new(&b))));
    }

    #[test]
    fn mas_sales_que_bytes_falla_cerrado_aunque_quepan_en_el_arbol() {
        // un `depth` de 31 deja pasar el tope de hojas con 2^31 sales: lo para la cuenta.
        // §711: aqui decia 40 y 2^32 - 1; desde el §711 un lote de mas de 31 niveles se rechaza
        // antes, por su profundidad (el testigo de abajo), y el de la cuenta se mide con el mayor
        // que se lee.
        let b = bytes_de(|b| {
            b.write_u8(31);
            b.write_usize(0);
            b.write_u32(1 << 31);
        });
        assert!(es_eof(VariasConSal::<Blake3>::read_from(&mut SliceReader::new(&b))));
    }

    #[test]
    fn una_prueba_que_declara_mas_bytes_de_los_que_trae_falla_cerrado_sin_reservar() {
        // Un contexto de verdad y, tras el, la primera longitud que `Proof` lee con `read_many`:
        // los `values` de la primera `Queries`. Sin el lector acotado del fork, pedia un terabyte.
        use winter_air::proof::{Context, Proof};
        use winter_air::{BatchingMethod, FieldExtension, ProofOptions, TraceInfo};
        let opciones = ProofOptions::new(
            28,
            8,
            0,
            FieldExtension::None,
            4,
            7,
            BatchingMethod::Linear,
            BatchingMethod::Linear,
        );
        let contexto = Context::new::<BaseElement>(TraceInfo::new(8, 8), opciones, 1)
            .expect("un contexto valido");
        let b = bytes_de(|b| {
            contexto.write_into(b);
            b.write_u8(1); // num_unique_queries
            b.write_u16(0); // los compromisos: cero bytes
            b.write_usize(1 << 40); // los `values` de la primera `Queries`
        });
        assert!(es_eof(Proof::from_bytes(&b)));
    }

    /// §715 (la maleabilidad de los bytes de la prueba): un item de mas, con su sal, no
    /// verifica. `get_root` recorre los indices y no mira si sobran hojas, y antes este lote
    /// verificaba con otros bytes: es la fila de mas de la traza, o el grupo de mas de una capa
    /// FRI. Falsador: sin la comprobacion de `verify_many`, las cinco formas verifican.
    #[test]
    fn un_item_de_mas_con_su_sal_no_verifica() {
        let (it, c) = con_sal();
        let formas = [vec![1usize, 3, 6], vec![0, 1], vec![2, 3, 4], (0..7).collect(), vec![7]];
        for idx in formas {
            let (_, lote) = c.open_many(&idx).expect("abrir");
            let mut abiertos: Vec<_> = idx.iter().map(|&k| it[k].clone()).collect();
            ConSal::verify_many(c.commitment(), &idx, &abiertos, &lote).expect("el honesto pasa");
            let mut maleado =
                VariasConSal::<Blake3>::read_from(&mut SliceReader::new(&lote.to_bytes()))
                    .expect("el lote se lee de sus bytes");
            abiertos.push(abiertos[0].clone());
            maleado.sales.push(maleado.sales[0].clone());
            assert!(
                matches!(
                    ConSal::verify_many(c.commitment(), &idx, &abiertos, &maleado),
                    Err(ErrorSal::Longitud)
                ),
                "un item de mas sobre {idx:?} tiene que fallar cerrado"
            );
        }
    }

    /// §715: la cuenta de vectores y la de nodos de un lote, escritas con un byte de mas
    /// o con nueve, son error con la causa del `usize`, y no se leen. Antes se leian igual, y la
    /// prueba que las llevaba verificaba con otros bytes. Falsador: con `read_usize` en
    /// `leer_lote`, las cuatro se leen.
    #[test]
    fn un_lote_con_una_cuenta_escrita_de_mas_no_se_lee() {
        let (_, c) = con_sal();
        let (_, lote) = c.open_many(&[1usize, 3, 6]).expect("abrir");
        let buena = lote.to_bytes();
        // `depth`, la cuenta de vectores (un byte: son pocos) y la de nodos del primero (otro)
        assert!(buena[1] & 1 == 1 && buena[2] & 1 == 1, "dos cuentas de un byte");
        for (que, en) in [("vectores", 1usize), ("nodos", 2)] {
            let v = (buena[en] >> 1) as u64;
            for largo in [2usize, 9] {
                let escrita = if largo == 9 {
                    let mut o = vec![0u8];
                    o.extend_from_slice(&v.to_le_bytes());
                    o
                } else {
                    ((((v << 1) | 1) << (largo - 1)).to_le_bytes())[..largo].to_vec()
                };
                let mut b = buena[..en].to_vec();
                b.extend(escrita);
                b.extend_from_slice(&buena[en + 1..]);
                match VariasConSal::<Blake3>::read_from(&mut SliceReader::new(&b)) {
                    Err(DeserializationError::InvalidValue(m)) => {
                        assert!(m.starts_with(winter_air::causa::USIZE), "{que} en {largo}: {m}")
                    },
                    otro => panic!("{que} en {largo}: se esperaba la causa, y dio {:?}", otro.err()),
                }
            }
        }
    }

    /// §711: un lote de mas de 31 niveles es error con su causa, y no un panico. Antes:
    /// `get_multiproof_domain_len` desplazaba `1 << depth` dentro de `read_from`, y con 64 o mas
    /// desbordaba (panico en release desde el §669). Falsador: sin la cota, los de 64, 200 y 255
    /// entran en panico, y el de 32, que en 64 bits desplaza sin desbordar, no da esta causa.
    #[test]
    fn un_lote_demasiado_profundo_es_error_con_causa_y_no_panico() {
        for depth in [32u8, 64, 200, 255] {
            let b = bytes_de(|b| {
                b.write_u8(depth);
                b.write_usize(0);
                b.write_u32(0);
            });
            let r = std::panic::catch_unwind(|| {
                VariasConSal::<Blake3>::read_from(&mut SliceReader::new(&b))
            })
            .unwrap_or_else(|_| panic!("CRITICO: un lote de {depth} niveles hace entrar en panico"));
            match r {
                Err(DeserializationError::InvalidValue(m)) => {
                    assert!(m.starts_with(LOTE_FUERA_DE_RANGO), "{depth}: {m}")
                },
                otro => panic!("{depth}: se esperaba la causa del lote, y dio {:?}", otro.err()),
            }
        }
    }
}
