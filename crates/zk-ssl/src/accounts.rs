//! Consulta del estado y apertura de cuentas.
//!
//! `open_account` crea siempre con **saldo cero**: no necesita prueba
//! porque no crea dinero. Para que una cuenta tenga fondos hay que
//! emitir, y eso exige la clave del emisor.
use super::*;

impl SovereignLayer {
    pub fn state_root(&self) -> Digest {
        self.accounts.root()
    }

    pub fn total_supply(&self) -> u64 {
        self.total_supply
    }

    pub fn regulatory_limit(&self) -> u64 {
        self.regulatory_limit
    }

    /// Tope de emisión del sistema.
    pub fn max_supply(&self) -> u64 {
        self.max_supply
    }

    /// Tope de cuentas del sistema.
    pub fn max_accounts(&self) -> u64 {
        self.max_accounts
    }

    pub fn custodian_set_root(&self) -> Digest {
        self.custodian_set_root
    }

    pub fn account_count(&self) -> usize {
        self.records.len()
    }

    pub fn balance_of(&self, index: AccountIndex) -> Option<u64> {
        self.records.get(&index).map(|r| r.balance)
    }

    /// Raíz del árbol de transferencias pendientes.
    pub fn pending_root(&self) -> Digest {
        self.pending.root()
    }

    /// Compromiso depositado en una posición, si lo hay.
    ///
    /// **No revela a quién va dirigido**: reconstruirlo exige el aviso del
    /// pagador y la clave del receptor.
    pub fn pending_at(&self, position: u64) -> Digest {
        self.pending.leaf(position)
    }

    /// Identidad pública de una cuenta.
    pub fn public_id_of(&self, index: AccountIndex) -> Option<Digest> {
        self.records.get(&index).map(|r| r.public_id)
    }

    /// Nonce de una cuenta.
    ///
    /// El cliente lo necesita para calcular su nullificador. **No es un
    /// secreto**: la protección viene de la clave de gasto, no de él.
    pub fn nonce_of(&self, index: AccountIndex) -> Option<BaseElement> {
        self.records.get(&index).map(|r| r.nonce)
    }

    /// Abre una cuenta **con saldo cero**.
    ///
    /// No necesita prueba porque **no crea dinero**. Para que tenga
    /// fondos hay que emitir, y eso exige la clave del emisor.
    /// **view_id almacenado de una cuenta** (49-A). Lo usa la vista
    /// autenticada del paso 4 para comparar contra la clave de vista que
    /// presenta el titular. Devuelve `None` si la cuenta no existe;
    /// `VIEW_ID_LEGACY` (cero) si es una cuenta pre-49-A.
    pub fn stored_view_id(&self, index: AccountIndex) -> Option<Digest> {
        self.records.get(&index).map(|r| r.view_id)
    }

    /// **leaf_salt almacenado** (B13/B14). La capa lo lee para recomputar
    /// la hoja salteada (no puede derivarlo: §93.4). `None` si no existe;
    /// `LEAF_SALT_LEGACY` en cuentas migradas/pre-B13.
    pub fn stored_leaf_salt(&self, index: AccountIndex) -> Option<Digest> {
        self.records.get(&index).map(|r| r.leaf_salt)
    }

    #[deprecated(
        since = "0.1.0",
        note = "Crea cuentas con clave de 64 bits: agotar su espacio cuesta \
                2^63, medido en 2,38 millones de años-nucleo y con cota \
                floja (AUDITORIA 82). Usa `open_account_wide`, que acepta \
                cuatro elementos. Se conserva porque son 115 llamadas y 158 \
                usos de `open_and_fund`, y §90 garantiza que una clave \
                rellenada da la misma identidad: la migracion es opt-in \
                (AUDITORIA 97.4)."
    )]
    /// ⚠️ **Privacidad hasta el despliegue del salt de hoja (Backlog 50,
    /// `AUDITORIA.md` §126, §132):** la hoja se compromete hoy **sin
    /// salt**, así que un observador del árbol puede recuperar el saldo de
    /// una cuenta por **diccionario sobre el hermano de camino** —barriendo
    /// balances candidatos hasta reproducir la hoja—. El coste es bajo en el
    /// rango de saldos realista (cifra en Backlog 50); afecta **por igual a
    /// las dos anchuras de apertura** —no depende de la entropía de la
    /// clave, sino de la ausencia de salt en la hoja—. **No se corrige
    /// rotando ni esperando**: una cuenta abierta hoy queda barrible
    /// mientras su hoja no lleve salt, y el salt no es retroactivo (§126.4).
    /// El cierre —`native_leaf_salted` en los ocho AIR, con envoltura
    /// salt-cero para las cuentas previas— está diseñado en §131 (numeración
    /// del árbol: §132) y es trabajo mayor, no inmediato.

    pub fn open_account(&mut self, spend_key: BaseElement) -> AccountIndex {
        self.open_account_checked(spend_key)
            .expect("abrir una cuenta no deberia fallar sin persistencia")
    }

    /// **Abre una cuenta con una clave de CUATRO elementos.**
    ///
    /// Es la puerta que faltaba: los cinco circuitos de gasto verifican
    /// claves de 256 bits desde §92.19, y hasta aqui **ningun titular podia
    /// tener una** —`open_account` solo creaba cuentas de 64 (§92.14)—.
    ///
    /// ## Por que se añade en vez de cambiar la firma
    ///
    /// Cambiarla obliga a tocar **115 llamadas** —medido, no estimado— entre
    /// `open_account`, `send`, `claim`, `burn` y `audit`, cuyas firmas
    /// dependen de esta. Se intento y los errores de compilacion pasaron de
    /// 15 a 85 (§97).
    ///
    /// ⚠️ **Y no crea dos formatos de cuenta**, que era la objecion de
    /// §85.5: §90 probo que `[sk,0,0,0]` da la MISMA identidad que `sk`. El
    /// arbol no distingue una cuenta abierta por una via de otra abierta por
    /// la otra — solo distingue **cuanta entropia** tiene su clave.
    ///
    /// ⚠️ **Lo que cuesta**: la API queda con dos entradas. Es deuda, y va
    /// declarada aqui en vez de descubrirse luego.
    /// ⚠️ **Privacidad hasta el despliegue del salt de hoja (Backlog 50,
    /// `AUDITORIA.md` §126, §132):** la hoja se compromete hoy **sin
    /// salt**, así que un observador del árbol puede recuperar el saldo de
    /// una cuenta por **diccionario sobre el hermano de camino** —barriendo
    /// balances candidatos hasta reproducir la hoja—. El coste es bajo en el
    /// rango de saldos realista (cifra en Backlog 50); afecta **por igual a
    /// las dos anchuras de apertura** —no depende de la entropía de la
    /// clave, sino de la ausencia de salt en la hoja—. **No se corrige
    /// rotando ni esperando**: una cuenta abierta hoy queda barrible
    /// mientras su hoja no lleve salt, y el salt no es retroactivo (§126.4).
    /// El cierre —`native_leaf_salted` en los ocho AIR, con envoltura
    /// salt-cero para las cuentas previas— está diseñado en §131 (numeración
    /// del árbol: §132) y es trabajo mayor, no inmediato.
    pub fn open_account_wide(&mut self, spend_key: Digest) -> AccountIndex {
        self.open_account_wide_checked(spend_key)
            .expect("abrir una cuenta no deberia fallar sin persistencia")
    }

    /// Igual que [`Self::open_account_wide`], con el error de persistencia.
    pub fn open_account_wide_checked(
        &mut self,
        spend_key: Digest,
    ) -> Result<AccountIndex, LayerError> {
        self.open_with_id(
            stark_experiment::circuit_settlement::derive_public_id_wide(spend_key),
            stark_experiment::circuit_settlement::view_id_of_wide(spend_key),
            stark_experiment::circuit_settlement::derive_leaf_salt_wide(spend_key),
        )
    }

    /// Igual que `open_account`, pero devuelve el error de persistencia.
    pub fn open_account_checked(
        &mut self,
        spend_key: BaseElement,
    ) -> Result<AccountIndex, LayerError> {
        // Sin tope, cualquiera podria crear cuentas hasta agotar la
        // memoria del nodo: `open_account` no exige autorizacion alguna.
        if self.next_index >= self.max_accounts {
            return Err(LayerError::AccountLimitReached {
                limit: self.max_accounts,
            });
        }
        self.open_with_id(
            derive_public_id(spend_key),
            stark_experiment::circuit_settlement::view_id_of(spend_key),
            stark_experiment::circuit_settlement::derive_leaf_salt(spend_key),
        )
    }

    /// Cuerpo comun de las dos vias de apertura.
    ///
    /// Recibe la **identidad ya derivada**, que es lo unico que la cuenta
    /// guarda: la clave no se almacena en ningun sitio (§93.4). Por eso las
    /// dos anchuras comparten todo salvo la derivacion.
    fn open_with_id(
        &mut self,
        public_id: Digest,
        view_id: Digest,
        leaf_salt: Digest,
    ) -> Result<AccountIndex, LayerError> {
        if self.next_index >= self.max_accounts {
            return Err(LayerError::AccountLimitReached {
                limit: self.max_accounts,
            });
        }
        let index = self.next_index;
        self.next_index += 1;
        let root_old = self.accounts.root();
        let nonce = BaseElement::ZERO;
        self.accounts
            .set_leaf(
                index,
                stark_experiment::circuit_settlement::native_leaf_salted(
                    public_id, BaseElement::ZERO, nonce, leaf_salt,
                ),
            );
        self.records.insert(
            index,
            AccountRecord {
                public_id,
                balance: 0,
                nonce,
                view_id,
                leaf_salt,
            },
        );
        // ⚠️ **La unica transicion de estado SIN prueba.**
        //
        // Abrir una cuenta no genera prueba porque no crea dinero: nace a
        // cero. Pero SI mueve la raiz de estado, asi que tiene que dejar
        // entrada en el registro o la cadena se rompe.
        //
        // Su resumen de prueba es cero, y eso es visible para quien
        // verifique el registro: sabe que esa transicion no esta
        // demostrada, solo registrada.
        self.log
            .append(OpKind::OpenAccount, root_old, self.accounts.root(), &[]);

        // Un solo lote atomico: la cuenta nueva y los metadatos.
        self.commit(&[index], None)?;
        Ok(index)
    }

    // -----------------------------------------------------------------
}


#[cfg(test)]
mod t_paso2_view_id {
    //! Paso 2 de 49-A verificado: el view_id se puebla y viaja correcto.
    use super::*;
    use crate::tests_support::*;
    use stark_experiment::circuit_settlement::{view_id_of, view_id_of_wide};
    use crate::store::VIEW_ID_LEGACY;

    const SK: u64 = 0xA11CE;

    #[test]
    fn apertura_puebla_view_id_real_no_centinela() {
        let mut layer = new_layer();
        let idx = open_and_fund(&mut layer, SK, 1_000_000);
        let esperado = view_id_of(BaseElement::new(SK));
        assert_eq!(layer.stored_view_id(idx), Some(esperado));
        assert_ne!(layer.stored_view_id(idx), Some(VIEW_ID_LEGACY),
                   "una cuenta nueva NO debe llevar el centinela");
    }

    #[test]
    fn via_ancha_hereda_90() {
        // §90: [sk,0,0,0] y sk dan la misma cuenta -> mismo view_id.
        let mut layer = new_layer();
        let sk_wide = [BaseElement::new(SK), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO];
        let idx = open_and_fund_wide(&mut layer, sk_wide, 0);
        assert_eq!(layer.stored_view_id(idx), Some(view_id_of_wide(sk_wide)));
        // y coincide con la estrecha del mismo sk (herencia §90)
        assert_eq!(view_id_of_wide(sk_wide), view_id_of(BaseElement::new(SK)));
    }

    #[test]
    fn cuenta_inexistente_es_none() {
        let layer = new_layer();
        assert_eq!(layer.stored_view_id(9999), None);
    }

    #[test]
    fn operar_preserva_el_view_id() {
        // Emitir sobre una cuenta NO debe cambiar su view_id (mint parte
        // del record guardado). Regresión de la corrección de seguridad.
        let mut layer = new_layer();
        let idx = open_and_fund(&mut layer, SK, 500_000);
        let antes = layer.stored_view_id(idx);
        let receipt = layer.mint(&valid_auth(), idx, 100_000).expect("mint");
        layer.apply_mint(&receipt, idx).expect("apply_mint");
        assert_eq!(layer.stored_view_id(idx), antes,
                   "operar cambió el view_id: la credencial no debe mutar al operar");
    }
}
