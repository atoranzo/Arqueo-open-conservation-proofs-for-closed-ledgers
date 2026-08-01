//! **Transferencia en dos fases**: el pagador no ve el saldo del receptor.
//!
//! ## El problema
//!
//! La liquidación actual actualiza **las dos hojas**, así que quien
//! construye la prueba necesita el saldo del receptor para calcular su
//! hoja nueva.
//!
//! **Pagar un euro a alguien revela cuánto tiene.**
//!
//! Y una contraparte no es un tercero: el operador es **uno y está
//! declarado**, mientras que quien te paga puede ser **cualquiera**.
//!
//! ## El diseño
//!
//! El pagador **no toca la hoja del receptor**. Crea un compromiso
//! pendiente atado a su identidad, y el receptor lo reclama.
//!
//! ```text
//! FASE 1 (pagador)          FASE 2 (receptor)
//! · debita su cuenta        · demuestra que el pendiente es SUYO
//! · crea el pendiente       · acredita su cuenta
//!   P = H(H(id_r, s), imp)  · anula el pendiente
//! ```
//!
//! ## Qué necesita cada parte
//!
//! | Parte | Necesita | NO necesita |
//! |---|---|---|
//! | Pagador | La identidad pública del receptor, como dirección | **Su saldo. Ni su nonce.** |
//! | Receptor | Su propio estado y el pendiente | Nada del pagador |
//! | Un tercero | — | Ve un compromiso opaco |
//!
//! Es el modelo de Zcash: no se actualiza el saldo del receptor, se crea
//! una nota para él.
//!
//! ## ⚠️ El residuo
//!
//! **El pagador elige el aleatorio `s`, así que reconoce el pendiente
//! cuando se reclama.** Sabe **cuándo** cobra el receptor, no cuánto
//! tiene.
//!
//! Es mucho menor que revelar el saldo, pero es una fuga de
//! vinculabilidad y conviene nombrarla. Zcash lo cierra cifrando la nota
//! para que el receptor derive el aleatorio; aquí no está resuelto.
//!
//! ## ⚠️ El coste de usabilidad
//!
//! **El pago pasa a ser en dos pasos y el dinero queda pendiente hasta que
//! el receptor actúe.** Hoy un pago se completa solo.
//!
//! Un despliegue real necesitaría reclamación automática por el proveedor
//! del receptor — con lo que el proveedor vuelve a ver el saldo, aunque
//! repartido y no concentrado en el operador.

use std::collections::HashMap;
use winterfell::math::{fields::f64::BaseElement, FieldElement};

use crate::sparse_tree::SparseTree;
use crate::Digest;
use stark_experiment::circuit_settlement::derive_public_id;
use stark_experiment::merkle::native_merge;

/// Compromiso de una transferencia pendiente.
///
/// `P = H(H(identidad_receptor, aleatorio), importe)`
///
/// El pagador lo construye con la **identidad pública** del receptor —que
/// funciona como dirección— y **sin conocer su saldo**.
pub fn pending_commitment(receiver_id: Digest, salt: Digest, amount: u64) -> Digest {
    let inner = native_merge(receiver_id, salt);
    native_merge(
        inner,
        [
            BaseElement::new(amount),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ],
    )
}

/// Lo que el pagador entrega al receptor por un canal aparte.
///
/// Sin esto el receptor no puede reclamar: necesita el aleatorio y el
/// importe para reconstruir el compromiso.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingNotice {
    pub salt: Digest,
    pub amount: u64,
}

/// Árbol de transferencias pendientes.
#[derive(Clone, Debug, Default)]
pub struct PendingTransfers {
    tree: SparseTree,
    claimed: HashMap<u64, Digest>,
    next: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClaimError {
    /// El pendiente no existe en esa posición.
    NotFound,
    /// Ya se reclamó.
    AlreadyClaimed,
    /// El compromiso no corresponde a la identidad de quien reclama.
    ///
    /// Es lo que impide reclamar el pendiente de otro.
    NotYours,
}

impl std::fmt::Display for ClaimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaimError::NotFound => write!(f, "no hay ningun pendiente en esa posicion"),
            ClaimError::AlreadyClaimed => write!(f, "ese pendiente ya se reclamo"),
            ClaimError::NotYours => {
                write!(f, "ese pendiente no esta dirigido a tu identidad")
            }
        }
    }
}
impl std::error::Error for ClaimError {}

impl PendingTransfers {
    pub fn new() -> Self {
        Self {
            tree: SparseTree::new(),
            claimed: HashMap::new(),
            next: 0,
        }
    }

    pub fn root(&self) -> Digest {
        self.tree.root()
    }

    /// **FASE 1.** El pagador crea el pendiente.
    ///
    /// No recibe ni consulta nada del receptor salvo su identidad pública.
    pub fn create(&mut self, receiver_id: Digest, salt: Digest, amount: u64) -> u64 {
        let p = pending_commitment(receiver_id, salt, amount);
        let pos = self.next;
        self.tree.set_leaf(pos, p);
        self.next += 1;
        pos
    }

    /// **FASE 2.** El receptor reclama, demostrando que es suyo.
    ///
    /// La comprobación es reconstruir el compromiso con **su propia
    /// identidad**: si no coincide, el pendiente era de otro.
    pub fn claim(
        &mut self,
        pos: u64,
        claimer_key: BaseElement,
        notice: &PendingNotice,
    ) -> Result<u64, ClaimError> {
        if self.claimed.contains_key(&pos) {
            return Err(ClaimError::AlreadyClaimed);
        }
        // `leaf` devuelve la hoja vacia si la posicion esta libre: no hay
        // forma de distinguir "nunca existio" de "ya se reclamo" mirando
        // solo el arbol, por eso el registro de reclamados va aparte.
        let guardado = self.tree.leaf(pos);
        if guardado == [BaseElement::ZERO; 4] {
            return Err(ClaimError::NotFound);
        }

        let mio = pending_commitment(
            derive_public_id(claimer_key),
            notice.salt,
            notice.amount,
        );
        if mio != guardado {
            return Err(ClaimError::NotYours);
        }

        self.tree.set_leaf(pos, [BaseElement::ZERO; 4]);
        self.claimed.insert(pos, guardado);
        Ok(notice.amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SK_ALICE: u64 = 0xA11CE;
    const SK_BOB: u64 = 0xB0B;
    const SK_MALLORY: u64 = 0xBADCAFE;

    fn salt(seed: u64) -> Digest {
        [
            BaseElement::new(seed),
            BaseElement::new(seed + 1),
            BaseElement::new(seed + 2),
            BaseElement::new(seed + 3),
        ]
    }

    fn id(sk: u64) -> Digest {
        derive_public_id(BaseElement::new(sk))
    }

    /// **EL PAGADOR NO NECESITA NADA DEL RECEPTOR SALVO SU IDENTIDAD.**
    ///
    /// Es la propiedad que cierra la fuga. La firma de `create` lo fija en
    /// el tipo: **no hay forma de pasarle un saldo ni un nonce**.
    #[test]
    fn the_payer_needs_nothing_but_the_recipients_identity() {
        let mut p = PendingTransfers::new();
        // Alice paga a Bob sin consultar su estado.
        let pos = p.create(id(SK_BOB), salt(0x5EED), 250_000);
        assert_eq!(pos, 0);
        assert_ne!(p.root(), PendingTransfers::new().root());
    }

    /// **EL RECEPTOR RECLAMA LO SUYO.**
    #[test]
    fn the_recipient_can_claim() {
        let mut p = PendingTransfers::new();
        let aviso = PendingNotice { salt: salt(0x5EED), amount: 250_000 };
        let pos = p.create(id(SK_BOB), aviso.salt, aviso.amount);

        let importe = p
            .claim(pos, BaseElement::new(SK_BOB), &aviso)
            .expect("Bob deberia poder reclamar lo suyo");
        assert_eq!(importe, 250_000);
    }

    /// **NADIE MÁS PUEDE RECLAMARLO.**
    ///
    /// Mallory tiene el aviso —el aleatorio y el importe— pero **no la
    /// clave de Bob**. Reconstruir el compromiso con su identidad da otro
    /// valor.
    ///
    /// Sin esto, quien interceptara el aviso cobraría el pago.
    #[test]
    fn nobody_else_can_claim_it() {
        let mut p = PendingTransfers::new();
        let aviso = PendingNotice { salt: salt(0x5EED), amount: 250_000 };
        let pos = p.create(id(SK_BOB), aviso.salt, aviso.amount);

        assert_eq!(
            p.claim(pos, BaseElement::new(SK_MALLORY), &aviso),
            Err(ClaimError::NotYours),
            "CRITICO: quien intercepte el aviso no debe poder cobrarlo"
        );
        // Y Bob sigue pudiendo.
        assert!(p.claim(pos, BaseElement::new(SK_BOB), &aviso).is_ok());
    }

    /// **NI EL PROPIO PAGADOR.**
    ///
    /// Alice conoce el aleatorio —lo eligió ella— pero el compromiso está
    /// atado a la identidad de Bob.
    #[test]
    fn not_even_the_payer_can_claim_it_back() {
        let mut p = PendingTransfers::new();
        let aviso = PendingNotice { salt: salt(0x5EED), amount: 250_000 };
        let pos = p.create(id(SK_BOB), aviso.salt, aviso.amount);

        assert_eq!(
            p.claim(pos, BaseElement::new(SK_ALICE), &aviso),
            Err(ClaimError::NotYours),
            "conocer el aleatorio no basta: hace falta la clave del receptor"
        );
    }

    /// **NO SE RECLAMA DOS VECES.**
    ///
    /// Sin esto, un receptor cobraría el mismo pago indefinidamente.
    #[test]
    fn a_pending_transfer_cannot_be_claimed_twice() {
        let mut p = PendingTransfers::new();
        let aviso = PendingNotice { salt: salt(0x5EED), amount: 250_000 };
        let pos = p.create(id(SK_BOB), aviso.salt, aviso.amount);

        assert!(p.claim(pos, BaseElement::new(SK_BOB), &aviso).is_ok());
        assert_eq!(
            p.claim(pos, BaseElement::new(SK_BOB), &aviso),
            Err(ClaimError::AlreadyClaimed),
            "CRITICO: reclamar dos veces seria cobrar dos veces"
        );
    }

    /// **NO SE PUEDE MENTIR SOBRE EL IMPORTE AL RECLAMAR.**
    ///
    /// El importe va dentro del compromiso. Declarar otro da un
    /// compromiso distinto.
    #[test]
    fn claiming_a_different_amount_is_rejected() {
        let mut p = PendingTransfers::new();
        let real = PendingNotice { salt: salt(0x5EED), amount: 250_000 };
        let pos = p.create(id(SK_BOB), real.salt, real.amount);

        let inflado = PendingNotice { salt: real.salt, amount: 999_999 };
        assert_eq!(
            p.claim(pos, BaseElement::new(SK_BOB), &inflado),
            Err(ClaimError::NotYours),
            "CRITICO: inflar el importe al reclamar crearia dinero"
        );
    }

    /// **EL COMPROMISO NO REVELA A QUIÉN VA DIRIGIDO.**
    ///
    /// Un tercero que vea el árbol de pendientes no puede saber de quién
    /// es cada uno sin conocer el aleatorio.
    #[test]
    fn the_commitment_does_not_reveal_the_recipient() {
        let a = pending_commitment(id(SK_BOB), salt(0x1111), 250_000);
        let b = pending_commitment(id(SK_BOB), salt(0x2222), 250_000);
        assert_ne!(
            a, b,
            "dos pagos al mismo receptor no deben ser vinculables entre si"
        );

        // Y sin el aleatorio no se puede comprobar una hipotesis.
        let hipotesis = pending_commitment(id(SK_BOB), salt(0x9999), 250_000);
        assert_ne!(hipotesis, a);
    }

    /// **⚠️ PERO EL PAGADOR SÍ RECONOCE CUÁNDO SE COBRA.**
    ///
    /// Eligió el aleatorio, así que puede recalcular el compromiso y ver
    /// cuándo desaparece del árbol.
    ///
    /// Es el residuo del diseño: sabe **cuándo** cobra el receptor, no
    /// cuánto tiene. Mucho menor que revelar el saldo, pero real.
    #[test]
    fn the_payer_can_still_tell_when_it_is_claimed() {
        let mut p = PendingTransfers::new();
        let aviso = PendingNotice { salt: salt(0x5EED), amount: 250_000 };
        let pos = p.create(id(SK_BOB), aviso.salt, aviso.amount);

        // Alice recalcula el compromiso porque eligio el aleatorio.
        let suyo = pending_commitment(id(SK_BOB), aviso.salt, aviso.amount);
        assert_eq!(p.tree.leaf(pos), suyo, "Alice lo reconoce");

        p.claim(pos, BaseElement::new(SK_BOB), &aviso).expect("reclamar");
        assert_ne!(
            p.tree.leaf(pos),
            suyo,
            "y ve cuando desaparece: sabe CUANDO cobro Bob"
        );
    }
}


#[cfg(test)]
mod t3a_reversion_como_segundo_cobro {
    //! T3a - entrada 12/§88: la exclusion del "segundo cobro", sometida a
    //! uso sobre el arbol real ANTES de construir circuito alguno. El
    //! compromiso v2 es prototipo de test; el formato real llega con las
    //! piezas 1-3.
    use super::*;
    use std::collections::HashMap;

    fn d(x: u64) -> Digest {
        [BaseElement::new(x), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
    }

    /// v2 = merge(v1, merge(refund_id, created_seq)): v1 como prefijo.
    fn commitment_v2(rec: Digest, salt: Digest, amt: u64, refund: Digest, seq: u64) -> Digest {
        native_merge(
            pending_commitment(rec, salt, amt),
            native_merge(refund, d(seq)),
        )
    }

    /// Reversion prototipo: mismo esqueleto que `claim`, reconstruyendo
    /// con la identidad de RETORNO. No comprueba Δ ni congelados: eso es
    /// del circuito (T3b).
    fn revert_v2(
        set: &mut PendingTransfers,
        reclamados: &mut HashMap<u64, Digest>,
        pos: u64,
        refund_key: BaseElement,
        rec: Digest,
        salt: Digest,
        amt: u64,
        seq: u64,
    ) -> Result<u64, &'static str> {
        if reclamados.contains_key(&pos) {
            return Err("ya consumido");
        }
        let guardado = set.tree.leaf(pos);
        if guardado == [BaseElement::ZERO; 4] {
            return Err("no existe o ya consumido");
        }
        let mio = commitment_v2(rec, salt, amt, derive_public_id(refund_key), seq);
        if mio != guardado {
            return Err("no eres el retorno");
        }
        set.tree.set_leaf(pos, [BaseElement::ZERO; 4]);
        reclamados.insert(pos, guardado);
        Ok(amt)
    }

    const SK_REFUND: u64 = 0xF00D;
    const SK_OTRO: u64 = 0xBAD;

    fn alta_v2(set: &mut PendingTransfers) -> (u64, Digest, Digest, u64, u64) {
        let (rec, salt, amt, seq) = (d(0xB0B0), d(0x5EED), 250_000u64, 42u64);
        let p = commitment_v2(rec, salt, amt, derive_public_id(BaseElement::new(SK_REFUND)), seq);
        let pos = set.next;
        set.tree.set_leaf(pos, p);
        set.next += 1;
        (pos, rec, salt, amt, seq)
    }

    #[test]
    fn t3a_exclusion_mutua() {
        let mut set = PendingTransfers::new();
        let mut rc = HashMap::new();
        let (pos, rec, salt, amt, seq) = alta_v2(&mut set);
        // reversion valida consume la hoja...
        assert_eq!(
            revert_v2(&mut set, &mut rc, pos, BaseElement::new(SK_REFUND), rec, salt, amt, seq),
            Ok(amt)
        );
        // ...y el segundo movimiento — cualquiera — muere en el arbol.
        assert!(revert_v2(&mut set, &mut rc, pos, BaseElement::new(SK_REFUND), rec, salt, amt, seq).is_err());
        assert_eq!(set.tree.leaf(pos), [BaseElement::ZERO; 4]);
    }

    #[test]
    fn t3a_solo_el_retorno_designado() {
        let mut set = PendingTransfers::new();
        let mut rc = HashMap::new();
        let (pos, rec, salt, amt, seq) = alta_v2(&mut set);
        assert!(revert_v2(&mut set, &mut rc, pos, BaseElement::new(SK_OTRO), rec, salt, amt, seq).is_err());
        // y el pendiente sigue integro para el legitimo
        assert!(revert_v2(&mut set, &mut rc, pos, BaseElement::new(SK_REFUND), rec, salt, amt, seq).is_ok());
    }

    #[test]
    fn t3a_sin_retroactividad_por_construccion() {
        // Un pendiente v1 NO es reversible: ningun refund_id reconstruye
        // su compromiso. La promesa "los viejos quedan como estaban" es
        // estructural, no disciplinaria.
        let (rec, salt, amt) = (d(0xB0B0), d(0x5EED), 250_000u64);
        let v1 = pending_commitment(rec, salt, amt);
        for rk in [SK_REFUND, SK_OTRO, 0u64, 1] {
            for seq in [0u64, 42, u64::MAX] {
                assert_ne!(v1, commitment_v2(rec, salt, amt, derive_public_id(BaseElement::new(rk)), seq));
            }
        }
    }
}
