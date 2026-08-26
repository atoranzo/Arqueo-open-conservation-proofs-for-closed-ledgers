//! Consulta del estado y apertura de cuentas.
//!
//! `open_account` crea siempre con **saldo cero**: no necesita prueba
//! porque no crea dinero. Para que una cuenta tenga fondos hay que
//! emitir, y emitir exige **dos custodios distintos**, no una clave.
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

    /// Abre una cuenta **con saldo cero**.
    ///
    /// No necesita prueba porque **no crea dinero**. Para que tenga
    /// fondos hay que emitir, y emitir exige **dos custodios
    /// distintos**, no una clave.
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
            stark_experiment::native::derive_public_id_wide(spend_key),
            stark_experiment::native::view_id_of_wide(spend_key),
            stark_experiment::native::derive_leaf_salt_wide(spend_key),
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
            stark_experiment::native::view_id_of(spend_key),
            stark_experiment::native::derive_leaf_salt(spend_key),
        )
    }

    /// Cuerpo comun de las dos vias de apertura.
    ///
    /// Recibe la **identidad ya derivada**, que es lo unico que la cuenta
    /// guarda: la clave no se almacena en ningun sitio (§93.4). Por eso las
    /// dos anchuras comparten todo salvo la derivacion.
    pub fn open_with_id(
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
        // Colocación del MUNDO NUEVO (F3): la MISMA política que la
        // migración (paso 1b) — `public_id[0] mod capacidad` con sondeo
        // lineal determinista. `next_index` queda como CENSO (cuota),
        // no como posición: los índices dejan de ser secuenciales y
        // enumerables (contratos de client.rs, superficies 2-3), y las
        // altas de un atacante ya no eligen vecino.
        let cap = self.accounts.capacity();
        let mut index = public_id[0].as_int() % cap;
        while self.records.contains_key(&index) {
            index = (index + 1) % cap;
        }
        self.next_index += 1;
        let root_old = self.accounts.root();
        let nonce = BaseElement::ZERO;
        self.accounts
            .set_leaf(
                index,
                stark_experiment::native::native_leaf_salted(
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
        // Su resumen de prueba es el de la AUSENCIA DECLARADA (§278), no
        // el de la prueba vacia: quien verifique el registro sabe que esa
        // transicion no esta demostrada, y sabe ademas que eso es una
        // decision y no un olvido — las delegadas ya no comparten valor
        // con ella.
        self.log.append(
            OpKind::OpenAccount,
            root_old,
            self.accounts.root(),
            &crate::log::sello_sin_prueba(),
        );

        // Un solo lote atomico: la cuenta nueva y los metadatos.
        self.commit(&[index], None)?;
        Ok(index)
    }

    /// **Materiales de un recibo de inclusión** (§259).
    ///
    /// ⚠️ **Del árbol `accounts`, que es el que firma la cabeza**
    /// (`epoch_head`: `accounts_root: self.accounts.root()`). No de
    /// `CommitmentLayer`, que **solo se instancia dentro de su propio
    /// `mod tests`**: un camino suyo llevaría a una raíz **que nadie
    /// firma**. El asiento de §256 lo decía al revés.
    ///
    /// ⚠️ **La forma de la hoja se MIDE, no se declara.** La capa no
    /// guarda la geometría en memoria —al abrir la deciden
    /// `meta:migrated` **o** `meta:geometry_v7`—, así que aquí se compone
    /// de las dos formas y se reporta **la que casó**. Un campo observado
    /// no puede quedarse rancio como una bandera que alguien mantiene.
    ///
    /// `StaleState` si no casa ninguna: el registro y el árbol discrepan.
    /// Es el mismo idioma que `burn.rs` y `audit.rs` ya usan.
    ///
    /// ⚠️ **No devuelve el `leaf_salt`**: se deriva de la clave de gasto y
    /// es lo único que impide enumerar el saldo desde un camino (§117).
    pub fn inclusion_materials(
        &self,
        index: AccountIndex,
    ) -> Result<MaterialesInclusion, LayerError> {
        let r = self
            .records
            .get(&index)
            .ok_or(LayerError::AccountNotFound(index))?;
        let en_arbol = self.accounts.leaf(index);
        let saldo = BaseElement::new(r.balance);
        let con_sal = zk_ssl_hash::native_leaf_salted(r.public_id, saldo, r.nonce, r.leaf_salt);
        let sin_sal = zk_ssl_hash::native_leaf(r.public_id, saldo, r.nonce);
        let forma = if en_arbol == con_sal {
            FormaHoja::ConSal
        } else if en_arbol == sin_sal {
            FormaHoja::SinSal
        } else {
            return Err(LayerError::StaleState);
        };
        Ok(MaterialesInclusion {
            index,
            leaf: en_arbol,
            path: self.accounts.path_for(index),
            forma,
        })
    }

    // -----------------------------------------------------------------
}

/// Con qué forma está compuesta la hoja de una cuenta (§259).
///
/// ⚠️ **Las dos conviven y no son intercambiables**: `native_leaf` NO es
/// `native_leaf_salted` con salt cero — hay test en `zk-ssl-hash` (§258).
/// Cuál aplica es propiedad **del ledger entero**, no de la cuenta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormaHoja {
    /// `native_leaf_salted`: la hoja envuelta con el salt de §117.
    ConSal,
    /// `native_leaf`: la hoja del mundo viejo, sin envolver.
    SinSal,
}

impl FormaHoja {
    /// Nombre **estable** para el cable. Cambiarlo mueve el protocolo.
    pub fn como_cable(self) -> &'static str {
        match self {
            FormaHoja::ConSal => "salted",
            FormaHoja::SinSal => "unsalted",
        }
    }
}

/// Lo que el nodo necesita para servir un recibo de inclusión.
///
/// ⚠️ **Sin el `leaf_salt`**, a propósito: ver `inclusion_materials`.
#[derive(Debug, Clone)]
pub struct MaterialesInclusion {
    pub index: AccountIndex,
    /// La hoja **tal como está en el árbol**, no recompuesta al vuelo.
    pub leaf: Digest,
    pub path: MerklePath,
    pub forma: FormaHoja,
}


#[cfg(test)]
mod t_paso2_view_id {
    //! Paso 2 de 49-A verificado: el view_id se puebla y viaja correcto.
    use super::*;
    use crate::tests_support::*;
    use stark_experiment::native::{view_id_of, view_id_of_wide};
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
        fund_delegated(&mut layer, idx, 100_000);
        assert_eq!(layer.stored_view_id(idx), antes,
                   "operar cambió el view_id: la credencial no debe mutar al operar");
    }
}
