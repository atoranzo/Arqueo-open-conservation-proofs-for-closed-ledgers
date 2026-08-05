//! Congelación de cuentas.
//!
//! ## Lo que la hace real
//!
//! La congelación **no la impone esta capa**: la impone el circuito de
//! liquidación, que acredita que el emisor no está en el árbol de
//! congelados.
//!
//! Si solo la impusiera la capa, sería el operador negándose a procesar —
//! y **el operador ya puede censurar cualquier operación**. No añadiría
//! ninguna garantía verificable.
//!
//! Aquí la comprobación previa existe solo para fallar pronto con un
//! error claro, en vez de gastar ~600 ms generando una prueba que no
//! verificará.
//!
//! ## ⚠️ Lo que NO impide
//!
//! - **Recibir.** Una cuenta congelada no puede gastar, pero sí seguir
//!   recibiendo. Impedirlo exigiría comprobar también al receptor y
//!   dejaría fondos en el limbo.
//!
//!   ⚠️ **Esto ya NO es cierto por la vía en dos fases.** Cobrar un pendiente
//!   es una acción del receptor, y `circuit_claim` lleva `frozen_root`: una
//!   cuenta congelada **recibe hacia un pendiente que no puede cobrar**. El
//!   dinero queda en el limbo que esta nota decía evitar. Ver
//!   `AUDITORIA.md` §29.
//! - **Nada justifica la congelación en el circuito.** Demuestra que dos
//!   custodios la autorizaron, no que tuvieran razón.
//! - **No hay caducidad.** Dura hasta que alguien la levante.

use super::*;

impl SovereignLayer {
    /// Raíz del árbol de congelados. Pública: entra en cada prueba de
    /// liquidación.
    pub fn frozen_root(&self) -> Digest {
        self.frozen.root()
    }

    /// Contador público de congelaciones y descongelaciones.
    pub fn freeze_count(&self) -> u64 {
        self.freeze_count
    }

    /// Si una cuenta está congelada.
    pub fn is_frozen(&self, account_index: AccountIndex) -> bool {
        self.frozen.is_occupied(account_index)
    }

    /// Congela o descongela **sin que las claves de custodio lleguen al
    /// operador**: la via de la entrada 32/33 (§60).
    ///
    /// Recibe tres pruebas: `climb_proof` de `circuit_frozen_climb` -que la
    /// transicion del arbol es una sola posicion cambiada, con el mismo camino
    /// en las dos subidas- y dos de custodios **distintos**, generadas en sus
    /// maquinas, que autorizan **esta** transicion.
    ///
    /// # Por que la prueba del arbol si hace falta
    ///
    /// La capa **recalcula la raiz por su cuenta**, asi que para ella la prueba
    /// es redundante. Su valor es para **quien audita el registro desde
    /// fuera**: sin ella el log guardaria una transicion que nadie mas puede
    /// comprobar.
    ///
    /// AVISO: **No prueba que se haya escrito una marca de congelado**: las
    /// hojas son valores libres (§58.3). Lo que hace legitima la operacion es
    /// que dos custodios firmaran esta transicion concreta.
    pub fn apply_freeze_delegated(
        &mut self,
        climb_proof: winterfell::Proof,
        proof_a: winterfell::Proof,
        inputs_a: stark_experiment::circuit_threshold_single_nullifier::NullifierThresholdPublicInputs,
        proof_b: winterfell::Proof,
        inputs_b: stark_experiment::circuit_threshold_single_nullifier::NullifierThresholdPublicInputs,
        account_index: AccountIndex,
        now_frozen: bool,
    ) -> Result<(), LayerError> {
        use stark_experiment::circuit_frozen_climb::{FrozenClimbAir, FrozenClimbPublicInputs};
        use stark_experiment::circuit_threshold::CUSTODIAN_DOMAIN;
        use stark_experiment::circuit_threshold_single_nullifier::{
            commit_operation, verify_threshold_pair, PairRejection, OP_FREEZE,
        };

        let root_old = self.frozen.root();
        let count_old = self.freeze_count;
        let count_new = count_old + 1;

        // La capa calcula ella misma a que raiz se va. No la toma de nadie.
        let mut tentativo = self.frozen.clone();
        tentativo.set_leaf(account_index, frozen_leaf(now_frozen));
        let root_new = tentativo.root();

        let accepted = AcceptableOptions::OptionSet(vec![self.options.clone()]);

        verify::<FrozenClimbAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            climb_proof,
            FrozenClimbPublicInputs { root_a: root_old, root_b: root_new },
            &accepted,
        )
        .map_err(|e| LayerError::VerificationFailed(format!("subida a congelados: {e:?}")))?;

        let mut params: Vec<BaseElement> = root_old.to_vec();
        params.extend_from_slice(&root_new);
        params.push(BaseElement::new(count_old));
        params.push(BaseElement::new(count_new));
        let operation = commit_operation(OP_FREEZE, &params);

        verify_threshold_pair(
            proof_a,
            inputs_a,
            proof_b,
            inputs_b,
            // Custodios, no gobernanza: congelar es custodia, no cambiar
            // quien custodia (§57.1).
            BaseElement::new(CUSTODIAN_DOMAIN),
            self.custodian_set_root,
            operation,
            &accepted,
        )
        .map_err(|r| match r {
            PairRejection::SameCustodian
            | PairRejection::WrongCustodianSet
            | PairRejection::WrongIdentityDomain => LayerError::NotTheIssuer,
            PairRejection::WrongOperation => LayerError::StaleState,
            PairRejection::InvalidProof => {
                LayerError::VerificationFailed("autorizacion invalida".into())
            }
        })?;

        // El cupo se consume DESPUES de verificar la autoridad.
        self.consume_custodian_use()?;

        self.frozen = tentativo;
        self.freeze_count = count_new;

        let raiz = self.accounts.root();
        self.log.append(OpKind::Freeze, raiz, raiz, &[]);
        self.commit(&[], None)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests_delegada {
    use super::*;
    use crate::tests_support::*;
    use stark_experiment::circuit_frozen_climb as climb;
    use stark_experiment::circuit_threshold::{build_custodian_set, CUSTODIAN_DOMAIN};
    use stark_experiment::circuit_threshold_single_nullifier as auth;

    const LIMIT: u64 = 1_000_000;
    const MAX_SUPPLY: u64 = 1_000_000_000;
    const MAX_ACCOUNTS: u64 = 1_000;

    fn dominio() -> BaseElement {
        BaseElement::new(CUSTODIAN_DOMAIN)
    }

    /// Igual que lo calcula `apply_freeze_delegated`. Si divergieran, el
    /// positivo fallaria: por eso esta el positivo.
    fn compromiso(root_old: Digest, root_new: Digest, count_old: u64) -> Digest {
        let mut v: Vec<BaseElement> = root_old.to_vec();
        v.extend_from_slice(&root_new);
        v.push(BaseElement::new(count_old));
        v.push(BaseElement::new(count_old + 1));
        auth::commit_operation(auth::OP_FREEZE, &v)
    }

    fn autorizar(
        key: BaseElement,
        path: &stark_experiment::circuit_threshold::CustodianPath,
        op: Digest,
    ) -> (winterfell::Proof, auth::NullifierThresholdPublicInputs) {
        let trace = auth::build_trace(dominio(), key, path, op);
        let prover = auth::NullifierThresholdProver::new(proof_options());
        let inputs = prover.get_pub_inputs(&trace);
        (prover.prove(trace).expect("autorizacion"), inputs)
    }

    fn prueba_subida(layer: &SovereignLayer, idx: AccountIndex, ahora: bool) -> winterfell::Proof {
        let path = layer.frozen.path_for(idx);
        let trace = climb::build_trace(frozen_leaf(!ahora), frozen_leaf(ahora), &path);
        climb::FrozenClimbProver::new(proof_options())
            .prove(trace)
            .expect("subida")
    }

    fn capa() -> SovereignLayer {
        let mut l = SovereignLayer::new(
            custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS);
        l.open_account_checked(BaseElement::new(SK_ALICE)).expect("abrir");
        l
    }

    /// El camino honesto: dos custodios distintos congelan sin entregar
    /// sus claves.
    #[test]
    fn a_delegated_freeze_applies() {
        let mut layer = capa();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let idx = 0;

        let root_old = layer.frozen_root();
        let mut t = layer.frozen.clone();
        t.set_leaf(idx, frozen_leaf(true));
        let op = compromiso(root_old, t.root(), layer.freeze_count());

        let subida = prueba_subida(&layer, idx, true);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);

        layer
            .apply_freeze_delegated(subida, pa, ia, pb, ib, idx, true)
            .expect("la congelacion legitima debe aplicarse");
        assert!(layer.is_frozen(idx), "la cuenta debe quedar congelada");
    }

    /// **REENVIAR una congelacion delegada NO cuenta dos veces.** El
    /// compromiso ata `count_old → count_old+1` y las raices: aplicado
    /// una vez, el contador avanzo y los mismos materiales quedan
    /// huerfanos. Releva en B-3 a `replaying_a_freeze_is_rejected`
    /// (via-recibo, §167).
    #[test]
    fn replaying_a_delegated_freeze_is_rejected() {
        let mut layer = capa();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let idx = 0;

        let root_old = layer.frozen_root();
        let mut t = layer.frozen.clone();
        t.set_leaf(idx, frozen_leaf(true));
        let op = compromiso(root_old, t.root(), layer.freeze_count());
        let subida = prueba_subida(&layer, idx, true);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);
        layer
            .apply_freeze_delegated(
                subida.clone(), pa.clone(), ia.clone(), pb.clone(), ib.clone(), idx, true,
            )
            .expect("la primera aplicacion");
        let cuenta = layer.freeze_count();

        // Los MISMOS materiales, otra vez.
        let r = layer.apply_freeze_delegated(subida, pa, ia, pb, ib, idx, true);
        assert!(r.is_err(), "CRITICO: reenviar contaria dos intervenciones: {r:?}");
        assert_eq!(layer.freeze_count(), cuenta, "el contador no avanza");
        assert!(layer.is_frozen(idx), "y el estado no se toca");
    }

    /// El mismo custodio dos veces no es un umbral.
    #[test]
    fn the_same_custodian_twice_cannot_freeze() {
        let mut layer = capa();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let idx = 0;

        let root_old = layer.frozen_root();
        let mut t = layer.frozen.clone();
        t.set_leaf(idx, frozen_leaf(true));
        let op = compromiso(root_old, t.root(), layer.freeze_count());

        let subida = prueba_subida(&layer, idx, true);
        let (pa, ia) = autorizar(ck[2], &cp[2], op);
        let (pb, ib) = autorizar(ck[2], &cp[2], op);

        let r = layer.apply_freeze_delegated(subida, pa, ia, pb, ib, idx, true);
        assert!(matches!(r, Err(LayerError::NotTheIssuer)), "fue {r:?}");
        assert!(!layer.is_frozen(idx), "no debe congelarse");
    }

    /// Una autorizacion para descongelar no sirve para congelar: el
    /// compromiso cubre las dos raices.
    #[test]
    fn an_authorization_for_another_transition_does_not_apply() {
        let mut layer = capa();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let idx = 0;

        let root_old = layer.frozen_root();
        let otra = compromiso(root_old, root_old, layer.freeze_count());

        let subida = prueba_subida(&layer, idx, true);
        let (pa, ia) = autorizar(ck[1], &cp[1], otra);
        let (pb, ib) = autorizar(ck[3], &cp[3], otra);

        let r = layer.apply_freeze_delegated(subida, pa, ia, pb, ib, idx, true);
        assert!(matches!(r, Err(LayerError::StaleState)), "fue {r:?}");
        assert!(!layer.is_frozen(idx));
    }

    /// La jerarquia al reves: gobernanza no congela. Puede cambiar quien
    /// custodia, no ejercer la custodia.
    #[test]
    fn governance_keys_cannot_freeze() {
        let mut layer = capa();
        let gk = governance_keys();
        let (_, gp) = build_custodian_set(&gk);
        let idx = 0;

        let root_old = layer.frozen_root();
        let mut t = layer.frozen.clone();
        t.set_leaf(idx, frozen_leaf(true));
        let op = compromiso(root_old, t.root(), layer.freeze_count());

        let subida = prueba_subida(&layer, idx, true);
        let (pa, ia) = autorizar(gk[1], &gp[1], op);
        let (pb, ib) = autorizar(gk[3], &gp[3], op);

        let r = layer.apply_freeze_delegated(subida, pa, ia, pb, ib, idx, true);
        assert!(matches!(r, Err(LayerError::NotTheIssuer)), "fue {r:?}");
        assert!(!layer.is_frozen(idx));
    }
}
