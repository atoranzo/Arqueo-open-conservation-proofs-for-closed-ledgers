//! Gobernanza: actualización del conjunto de custodios.
//!
//! ## El cambio de fondo
//!
//! Hasta ahora `custodian_set_root` era **inmutable**: si un custodio se
//! comprometía, conservaba para siempre el poder de emitir y —desde la
//! recuperación— de reasignar cualquier cuenta. La única salida era crear
//! un ledger nuevo.
//!
//! Ahora es **mutable por gobernanza**, y el parámetro inmutable pasa a
//! ser el conjunto de gobernanza.
//!
//! | Conjunto | Puede | Mutabilidad |
//! |---|---|---|
//! | Custodios | Emitir, recuperar cuentas | **Cambiable** |
//! | Gobernanza | Cambiar los custodios | **Inmutable** |
//!
//! ## Dónde para la cadena, dicho sin adornos
//!
//! **La circularidad no desaparece: se traslada.** Si el conjunto de
//! gobernanza se compromete, no hay salida salvo crear un ledger nuevo.
//!
//! Pero se traslada a claves que se usan casi nunca y pueden guardarse
//! sin conexión, frente a claves operativas expuestas a diario. Eso es
//! una mejora real, y no es lo mismo que resolver el problema.

use super::*;

impl SovereignLayer {
    /// Raíz del conjunto de gobernanza. **Inmutable**: cambiarla exige
    /// crear un ledger nuevo.
    pub fn governance_set_root(&self) -> Digest {
        self.governance_set_root
    }

    /// Contador público de cambios de gobernanza.
    /// Intervenciones consumidas por el conjunto de custodios vigente.
    pub fn custodian_uses(&self) -> u64 {
        self.custodian_uses
    }

    /// Fija el cupo de intervenciones por conjunto de custodios.
    ///
    /// ⚠️ **No lo protege ninguna autorización**: es configuración del
    /// despliegue, no una operación del sistema. Un operador puede subirlo
    /// y así **anular la rotación**.
    ///
    /// Es coherente con el modelo declarado —el operador ya controla la
    /// capa— pero conviene saberlo: la rotación es una política que el
    /// operador aplica, no una garantía que le vincule.
    ///
    /// Imponerla exigiría llevar el cupo a los circuitos de emisión,
    /// congelación y recuperación. **No está hecho.**
    pub fn set_max_custodian_uses(&mut self, quota: u64) {
        self.max_custodian_uses = quota;
    }

    /// Cupo del conjunto vigente.
    pub fn max_custodian_uses(&self) -> u64 {
        self.max_custodian_uses
    }

    /// Comprueba que quedan intervenciones y **consume una**.
    ///
    /// La llaman emitir, congelar y recuperar. Va **después** de verificar
    /// la autoridad: si fuera antes, agotar el cupo ajeno sería posible sin
    /// ser custodio.
    pub(crate) fn consume_custodian_use(&mut self) -> Result<(), LayerError> {
        if self.custodian_uses >= self.max_custodian_uses {
            return Err(LayerError::CustodianSetExhausted {
                uses: self.custodian_uses,
                max: self.max_custodian_uses,
            });
        }
        self.custodian_uses += 1;
        Ok(())
    }

    pub fn governance_change_count(&self) -> u64 {
        self.governance_change_count
    }

    /// Genera la prueba de un cambio del conjunto de custodios.
    /// **No modifica el estado.**
    ///
    /// Exige **dos miembros distintos del conjunto de gobernanza**, no de
    /// custodios: quien puede emitir y recuperar cuentas no puede cambiar
    /// quién tiene ese poder.
    #[deprecated(
        since = "0.1.0",
        note = "Exige las claves de custodio EN EL OPERADOR: es el fallo de la \
                entrada 32. Usa la via delegada, donde cada custodio prueba en \
                su maquina. Las CINCO operaciones ya la tienen desde el \
                31-07-2026 (AUDITORIA 71), asi que esta ya no hace falta: se \
                conserva solo hasta migrar sus usos, inventariados en \
                AUDITORIA 80."
    )]
    pub fn update_custodians(
        &self,
        auth: &GovernanceAuth,
        new_custodian_root: Digest,
    ) -> Result<GovernanceReceipt, LayerError> {
        if auth.index_a >= auth.index_b {
            return Err(LayerError::NotTheIssuer);
        }
        if new_custodian_root == self.custodian_set_root {
            return Err(LayerError::RecoveryToSameIdentity);
        }

        let trace = build_governance_trace(auth, self.governance_change_count, 1);
        let prover = GovernanceProver::new(
            self.options.clone(),
            self.custodian_set_root,
            new_custodian_root,
        );
        let public_inputs = prover.get_pub_inputs(&trace);
        let proof = prover
            .prove(trace)
            .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

        Ok(GovernanceReceipt {
            proof: proof.to_bytes(),
            public_inputs,
        })
    }

    /// Verifica un cambio de gobernanza y, si es válido y parte del
    /// estado actual, lo aplica.
    pub fn apply_governance(&mut self, receipt: &GovernanceReceipt) -> Result<(), LayerError> {
        let pi = &receipt.public_inputs;

        // La autoridad de gobernanza es la del sistema, y es inmutable.
        if pi.governance_set_root != self.governance_set_root {
            return Err(LayerError::NotTheIssuer);
        }
        if pi.custodian_root_old != self.custodian_set_root
            || pi.change_count_old != BaseElement::new(self.governance_change_count)
        {
            return Err(LayerError::StaleState);
        }

        let proof = winterfell::Proof::from_bytes(&receipt.proof)
            .map_err(|e| LayerError::VerificationFailed(format!("prueba mal formada: {e:?}")))?;
        let min_opts = AcceptableOptions::OptionSet(vec![self.options.clone()]);
        verify::<GovernanceAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            pi.clone(),
            &min_opts,
        )
        .map_err(|e| LayerError::VerificationFailed(format!("{e:?}")))?;

        self.custodian_set_root = pi.custodian_root_new;

        // ===== ROTAR REINICIA EL CUPO =====
        //
        // Es lo que hace que la rotación sirva: el conjunto nuevo empieza
        // con sus intervenciones enteras, y el viejo deja de poder actuar
        // porque su raíz ya no es la vigente.
        self.custodian_uses = 0;
        self.governance_change_count = pi.change_count_new.as_int();

        // El cambio de custodios no toca cuentas ni nullifiers, pero sí
        // los metadatos: se persiste en el mismo lote atómico.
        // La cadena va SIEMPRE sobre la raiz de CUENTAS, no sobre la del
        // arbol que esta operacion modifica.
        //
        // Encadenar raices de arboles distintos no funciona: la raiz de
        // custodios de una entrada no tiene por que ser la de cuentas de
        // la siguiente. Esta operacion no toca el arbol de cuentas, asi
        // que su raiz no cambia — y el detalle de lo que SI cambio queda
        // atado por el resumen de la prueba.
        let raiz = self.accounts.root();
        // Deja constancia en el registro ANTES de persistir: si el
        // proceso muere en medio, el lote atomico incluye o excluye
        // ambas cosas.
        self.log
            .append(OpKind::Governance, raiz, raiz, &receipt.proof);
        self.commit(&[], None)?;
        Ok(())
    }

    /// Aplica un cambio de custodios **sin que las claves lleguen al
    /// operador**: la via de la entrada 32/33 (§57).
    ///
    /// Cada miembro del conjunto de gobernanza genera en su maquina una
    /// prueba de `circuit_threshold_single_nullifier`, y la capa exige dos
    /// de miembros **distintos** que autoricen **este** cambio concreto.
    ///
    /// # Que se mueve, y de donde a donde
    ///
    /// `apply_governance` confia en que el circuito demostro la
    /// autorizacion; para construir su traza la capa necesita **las claves
    /// en crudo** (§41), que es el fallo de la 32. Aqui la autorizacion
    /// llega ya probada y la capa solo la **verifica**.
    ///
    /// ⚠️ **La garantia se muda del circuito a esta funcion.** Si esta
    /// comprobacion se omitiera, cualquiera cambiaria el conjunto de
    /// custodios: no queda ningun circuito que lo impida, porque lo unico
    /// que probaba `circuit_governance` ademas de la autorizacion era que
    /// el contador sube en uno, y eso se comprueba aqui en tres lineas.
    /// Por eso lleva sus propios tests de rechazo.
    pub fn apply_governance_delegated(
        &mut self,
        proof_a: winterfell::Proof,
        inputs_a: stark_experiment::circuit_threshold_single_nullifier::NullifierThresholdPublicInputs,
        proof_b: winterfell::Proof,
        inputs_b: stark_experiment::circuit_threshold_single_nullifier::NullifierThresholdPublicInputs,
        new_custodian_root: Digest,
    ) -> Result<(), LayerError> {
        use stark_experiment::circuit_threshold_single_nullifier::{
            commit_operation, verify_threshold_pair, PairRejection, OP_GOVERNANCE,
        };

        if new_custodian_root == self.custodian_set_root {
            return Err(LayerError::RecoveryToSameIdentity);
        }

        let count_old = self.governance_change_count;
        let count_new = count_old + 1;

        // El compromiso cubre TODO lo que el cambio decide: de que conjunto
        // se sale, a cual se entra, y en que punto del contador. Sin el, una
        // autorizacion para un cambio serviria para cualquier otro (§56.2).
        let mut params: Vec<BaseElement> = self.custodian_set_root.to_vec();
        params.extend_from_slice(&new_custodian_root);
        params.push(BaseElement::new(count_old));
        params.push(BaseElement::new(count_new));
        let operation = commit_operation(OP_GOVERNANCE, &params);

        let accepted = AcceptableOptions::OptionSet(vec![self.options.clone()]);
        verify_threshold_pair(
            proof_a,
            inputs_a,
            proof_b,
            inputs_b,
            // ⚠️ **La jerarquia, declarada dos veces.** El dominio dice que
            // esto es una autorizacion de GOBERNANZA; la raiz, de que
            // conjunto. Las identidades de custodio se derivan con otro
            // dominio (§57.1), asi que un custodio no puede pasar por
            // miembro de gobernanza ni aunque conociera la raiz.
            BaseElement::new(stark_experiment::circuit_governance::GOVERNANCE_DOMAIN),
            self.governance_set_root,
            operation,
            &accepted,
        )
        .map_err(|r| match r {
            PairRejection::SameCustodian => LayerError::NotTheIssuer,
            PairRejection::WrongCustodianSet => LayerError::NotTheIssuer,
            PairRejection::WrongIdentityDomain => LayerError::NotTheIssuer,
            PairRejection::WrongOperation => LayerError::StaleState,
            PairRejection::InvalidProof => {
                LayerError::VerificationFailed("autorizacion invalida".into())
            }
        })?;

        self.custodian_set_root = new_custodian_root;
        self.custodian_uses = 0;
        self.governance_change_count = count_new;

        let raiz = self.accounts.root();
        self.log.append(OpKind::Governance, raiz, raiz, &[]);
        self.commit(&[], None)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests_delegada {
    use super::*;
    use crate::tests_support::*;
    use stark_experiment::circuit_governance::build_governance_set;
    use stark_experiment::circuit_threshold_single_nullifier as auth;

    const LIMIT: u64 = 1_000_000;
    const MAX_SUPPLY: u64 = 1_000_000_000;
    const MAX_ACCOUNTS: u64 = 1_000;

    /// Calcula el compromiso igual que lo hace la capa. Si esto y
    /// `apply_governance_delegated` divergieran, los tests pasarian sin
    /// probar nada: por eso el positivo comprueba que el cambio SE APLICA.
    fn compromiso(root_old: Digest, root_new: Digest, count_old: u64) -> Digest {
        let mut p: Vec<BaseElement> = root_old.to_vec();
        p.extend_from_slice(&root_new);
        p.push(BaseElement::new(count_old));
        p.push(BaseElement::new(count_old + 1));
        auth::commit_operation(auth::OP_GOVERNANCE, &p)
    }

    fn autorizar(
        key: BaseElement,
        path: &stark_experiment::circuit_threshold::CustodianPath,
        op: Digest,
    ) -> (winterfell::Proof, auth::NullifierThresholdPublicInputs) {
        let trace = auth::build_trace(
            BaseElement::new(stark_experiment::circuit_governance::GOVERNANCE_DOMAIN),
            key,
            path,
            op,
        );
        let prover = auth::NullifierThresholdProver::new(proof_options());
        let inputs = prover.get_pub_inputs(&trace);
        (prover.prove(trace).expect("la autorizacion deberia probar"), inputs)
    }

    /// Un conjunto de custodios distinto del vigente, para cambiar hacia el.
    /// Se define aqui porque el de `tests.rs` no es alcanzable desde este
    /// modulo.
    fn new_custodian_root() -> Digest {
        let keys: Vec<BaseElement> = (0..5)
            .map(|i| BaseElement::new(0xD0_0D_00 + i))
            .collect();
        stark_experiment::circuit_threshold::build_custodian_set(&keys).0
    }

    fn capa() -> SovereignLayer {
        SovereignLayer::new(custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
    }

    /// El camino honesto: dos miembros distintos de gobernanza cambian el
    /// conjunto de custodios **sin entregar sus claves**.
    #[test]
    fn a_delegated_governance_change_applies() {
        let mut layer = capa();
        let gk = governance_keys();
        let (_, gp) = build_governance_set(&gk);
        let nueva = new_custodian_root();

        let op = compromiso(layer.custodian_set_root(), nueva, layer.governance_change_count());
        let (pa, ia) = autorizar(gk[1], &gp[1], op);
        let (pb, ib) = autorizar(gk[3], &gp[3], op);

        layer
            .apply_governance_delegated(pa, ia, pb, ib, nueva)
            .expect("el cambio legitimo debe aplicarse");
        assert_eq!(layer.custodian_set_root(), nueva, "la raiz debe haber cambiado");
        assert_eq!(layer.governance_change_count(), 1);
    }

    /// ⚠️ **El mismo miembro dos veces no es un umbral.** Sin esta
    /// comprobacion, cualquiera con UNA clave de gobernanza cambiaria quien
    /// tiene el poder de emitir.
    #[test]
    fn the_same_governance_member_twice_is_rejected() {
        let mut layer = capa();
        let gk = governance_keys();
        let (_, gp) = build_governance_set(&gk);
        let nueva = new_custodian_root();

        let op = compromiso(layer.custodian_set_root(), nueva, layer.governance_change_count());
        let (pa, ia) = autorizar(gk[1], &gp[1], op);
        let (pb, ib) = autorizar(gk[1], &gp[1], op);

        let r = layer.apply_governance_delegated(pa, ia, pb, ib, nueva);
        assert!(
            matches!(r, Err(LayerError::NotTheIssuer)),
            "SOLIDEZ: dos autorizaciones del mismo miembro no son 2-de-N, fue {r:?}"
        );
        assert_eq!(layer.custodian_set_root(), custodian_root(), "no debe cambiar nada");
    }

    /// ⚠️ **La atadura a la operacion, al nivel de la capa.** Se autoriza
    /// un cambio hacia una raiz y se intenta ejecutar otro distinto. Sin el
    /// compromiso de §56, una autorizacion de gobernanza serviria para
    /// cualquier cambio de custodios.
    #[test]
    fn an_authorization_for_another_root_does_not_apply() {
        let mut layer = capa();
        let gk = governance_keys();
        let (_, gp) = build_governance_set(&gk);
        let autorizada = new_custodian_root();
        let pretendida = build_governance_set(&custodian_keys()).0;

        let op = compromiso(layer.custodian_set_root(), autorizada, layer.governance_change_count());
        let (pa, ia) = autorizar(gk[1], &gp[1], op);
        let (pb, ib) = autorizar(gk[3], &gp[3], op);

        let r = layer.apply_governance_delegated(pa, ia, pb, ib, pretendida);
        assert!(
            matches!(r, Err(LayerError::StaleState)),
            "SOLIDEZ: una autorizacion para un cambio no vale para otro, fue {r:?}"
        );
        assert_eq!(layer.custodian_set_root(), custodian_root());
    }

    /// ⚠️ **La jerarquia se sostiene.** Los custodios pueden emitir, pero
    /// NO pueden cambiar quien es custodio: su conjunto no es el de
    /// gobernanza, y `verify_threshold_pair` exige esa raiz.
    #[test]
    fn custodian_keys_cannot_change_the_custodian_set() {
        let mut layer = capa();
        let ck = custodian_keys();
        let (_, cp) = build_governance_set(&ck); // su propio conjunto, no el de gobernanza
        let nueva = new_custodian_root();

        let op = compromiso(layer.custodian_set_root(), nueva, layer.governance_change_count());
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);

        let r = layer.apply_governance_delegated(pa, ia, pb, ib, nueva);
        assert!(
            matches!(r, Err(LayerError::NotTheIssuer)),
            "SOLIDEZ: quien puede emitir no puede cambiar quien emite, fue {r:?}"
        );
        assert_eq!(layer.custodian_set_root(), custodian_root());
    }
}
