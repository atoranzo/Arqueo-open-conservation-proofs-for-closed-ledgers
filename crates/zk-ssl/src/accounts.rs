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

    /// ⚠️ **EL ÁRBOL DE NULLIFICADORES YA NO SE ESCRIBE.**
    ///
    /// La única vía que insertaba nullificadores era `transfer` + `apply`,
    /// retirada. Hoy **todas** las llamadas a `commit` pasan `None`, y las
    /// únicas escrituras que quedan —en `persistence` y `snapshot`—
    /// **restauran de disco lo que se escribió antes de la retirada**.
    ///
    /// **Esto cierra el límite del cumpleaños de `AUDITORIA.md` §13**: la
    /// capacidad efectiva de ~65.000 pagos dependía de que la posición se
    /// derivara del propio nullificador, y ya no se genera ninguno.
    ///
    /// El árbol se conserva por **compatibilidad de formato**: quitarlo
    /// cambiaría el fichero en disco y el instantáneo, y obligaría a migrar
    /// los ledgers existentes. Su raíz sigue siendo parte del estado
    /// persistido y del instantáneo, y para un ledger nuevo es siempre la
    /// del árbol vacío.
    ///
    /// ⚠️ **Vuelve a hacer falta con consenso distribuido.** El
    /// encadenamiento de raíces que lo sustituye exige un orden total, y en
    /// un sistema distribuido no lo hay. Ver `AUDITORIA.md` §13.
    pub fn nullifier_root(&self) -> Digest {
        self.nullifiers.root()
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
    pub fn open_account(&mut self, spend_key: BaseElement) -> AccountIndex {
        self.open_account_checked(spend_key)
            .expect("abrir una cuenta no deberia fallar sin persistencia")
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
        let index = self.next_index;
        self.next_index += 1;
        let root_old = self.accounts.root();
        let public_id = derive_public_id(spend_key);
        let nonce = BaseElement::ZERO;
        self.accounts
            .set_leaf(index, native_leaf(public_id, BaseElement::ZERO, nonce));
        self.records.insert(
            index,
            AccountRecord {
                public_id,
                balance: 0,
                nonce,
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
        self.commit(&[index], None, None)?;
        Ok(index)
    }

    // -----------------------------------------------------------------
}
