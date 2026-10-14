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

    /// RFC-0007 E5 (D-G): la HOJA de `index` en el arbol de CUENTAS y su
    /// CAMINO, para que quien recibio un `AccountNotFound(index)` pueda
    /// probar —o desmentir— esa causa contra el `accountsRoot` que una cabeza
    /// firmada compromete.
    ///
    /// Hermano de `frozen_path_of` (§458) y con la misma cautela: sirve el
    /// estado de AHORA, no el de un `seq` pasado. A diferencia de
    /// `inclusion_materials`, **no exige que la cuenta exista**: el caso que
    /// la causa nombra es justo el de la hoja VACIA.
    pub fn accounts_path_of(&self, index: AccountIndex) -> (Digest, MerklePath) {
        (self.accounts.leaf(index), self.accounts.path_for(index))
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
            .expect("abrir la cuenta: el tope, la identidad no disponible (§N) o el disco")
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
            .expect("abrir la cuenta: el tope, la identidad no disponible (§N) o el disco")
    }

    /// Igual que [`Self::open_account_wide`], con el error: el tope, la identidad no disponible (§N) o el disco.
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

    /// Igual que `open_account`, pero devuelve el error: el tope, la identidad no disponible (§N) o el disco.
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
        // §N: tras el tope -con el libro lleno sigue ganando el tope- y antes de mutar nada.
        self.identidad_libre(&public_id)?;
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
        self.identidades.insert(digest_to_bytes(&public_id));
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

/// RFC-0007 E5 (D-G): la profundidad del arbol de cuentas tiene DOS
/// productores —el arbol, que la toma de `SparseTree::new()`, y el nucleo, que
/// la publica para el verificador (`zk_ssl_hash::ACCOUNTS_DEPTH`)— y este test
/// los ata. No compara dos constantes: compara la constante contra lo que el
/// arbol PRODUCE, que es mas fuerte. De paso fija que el camino de una cuenta
/// LIBRE lleva la hoja vacia y sube a `state_root`.
#[cfg(test)]
mod tests_camino_cuentas {
    use super::*;
    use crate::tests_support::*;

    #[test]
    fn el_camino_de_cuentas_mide_la_profundidad_que_el_nucleo_publica() {
        // Los dos arboles tienen que DIVERGIR o un falsador que los
        // intercambie NO discrimina (PRECISION 171): vacios y de la misma
        // profundidad, sus caminos y sus raices son los MISMOS.
        let mut layer = new_layer();
        let idx = open_and_fund(&mut layer, 1, 100);
        assert_ne!(idx, 7, "la cuenta abierta aterrizo en la 7: elegir otro indice libre");
        assert_ne!(
            layer.state_root(),
            layer.frozen_root(),
            "los dos arboles no divergen: el testigo no probaria nada"
        );
        let (hoja, camino) = layer.accounts_path_of(7);
        assert_eq!(hoja, [BaseElement::ZERO; 4], "la 7 esta libre: su hoja es la vacia");
        assert_eq!(camino.siblings.len(), zk_ssl_hash::ACCOUNTS_DEPTH);
        assert_eq!(camino.is_right.len(), zk_ssl_hash::ACCOUNTS_DEPTH);
        assert_eq!(
            zk_ssl_hash::path_root(hoja, &camino.siblings, &camino.is_right),
            layer.state_root(),
            "el camino servido tiene que subir a la raiz que la cabeza compromete"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// §N: UNA IDENTIDAD, UNA CUENTA, EN LA VIDA DEL LIBRO.
//
// Hasta el §N `open_with_id` no miraba si la identidad era nueva: solo sondeaba la
// posicion `public_id[0] mod capacidad`. Un tercero sin la clave abria una segunda
// cuenta con la identidad de otro, y tras una recuperacion quien conservara la clave
// VIEJA abria otra cuenta con la identidad retirada y cobraba ahi un pendiente en
// vuelo hacia ella (SECURITY.md §3.13). La regla: una identidad -el `public_id`
// entero, los cuatro elementos- tiene a lo sumo una cuenta, y la que una
// recuperacion retira no vuelve a tener ninguna. El conjunto «en uso ∪ retiradas»
// solo crece. Va al final del fichero para que las citas `accounts.rs:linea` de los
// documentos no se muevan.
// ─────────────────────────────────────────────────────────────────────────────

impl SovereignLayer {
    /// **El unico juez de la invariante del §N.** `IdentityUnavailable` si la identidad
    /// ya tiene cuenta o una recuperacion la retiro. Compara el `Digest` ENTERO, no su
    /// primer elemento ni la posicion de sondeo: la recuperacion cambia la identidad sin
    /// mover la cuenta, que se queda en la cadena de sondeo de la vieja.
    ///
    /// Lo llaman `open_with_id` -por donde pasan todas las vias que abren- y
    /// `apply_recovery_delegated`, siempre antes de mutar nada.
    pub(crate) fn identidad_libre(&self, id: &Digest) -> Result<(), LayerError> {
        if self.identidades.contains(&digest_to_bytes(id)) {
            return Err(LayerError::IdentityUnavailable);
        }
        Ok(())
    }

    /// **El unico productor del indice al cargar y al importar.** Lo vacia y lo rellena
    /// con las identidades de todos los registros y con todas las retiradas. Un libro
    /// escrito antes del §N puede traer dos cuentas con la misma identidad: aqui
    /// colapsan, y no se rechazan.
    pub(crate) fn reindexar_identidades(&mut self) {
        self.identidades.clear();
        self.identidades.extend(self.records.values().map(|r| digest_to_bytes(&r.public_id)));
        self.identidades.extend(self.retiradas.iter().map(digest_to_bytes));
    }

    /// Una recuperacion retira la identidad `vieja`: entra en la lista, con su eslabon,
    /// y se queda en el indice (ya estaba: tenia cuenta).
    pub(crate) fn retirar(&mut self, vieja: Digest) {
        self.retir_cabeza = eslabon_de_retirada(self.retir_cabeza, vieja);
        self.retiradas.push(vieja);
        self.identidades.insert(digest_to_bytes(&vieja));
    }

    /// Fija las retiradas que se leyeron del disco o de una instantanea, y reindexa. Exige
    /// una por recuperacion del registro: `n_recovery` se cuenta del registro ya
    /// verificado, no de una cabecera.
    pub(crate) fn adoptar_retiradas(
        &mut self,
        ids: Vec<Digest>,
        n_recovery: u64,
        what: &'static str,
    ) -> Result<(), LayerError> {
        if ids.len() as u64 != n_recovery {
            return Err(StoreError::IntegrityFailure { what }.into());
        }
        self.retir_cabeza = cabeza_de_retiradas(&ids);
        self.retiradas = ids;
        self.reindexar_identidades();
        Ok(())
    }
}

/// §N: un eslabon de la cadena de retiradas. La cadena es `c_0 = [0; 4]`,
/// `c_{k+1} = native_merge(c_k, id_k)`: ata el orden y el numero, y cuesta un hash por
/// recuperacion. No es un arbol disperso: ese trata el digest cero como hoja libre, y
/// el cable acepta abrir con `public_id = [0; 4]`.
pub(crate) fn eslabon_de_retirada(cabeza: Digest, id: Digest) -> Digest {
    stark_experiment::merkle::native_merge(cabeza, id)
}

/// §N: la cabeza de la cadena de retiradas, desde `[0; 4]`. El paso incremental de
/// `retirar` es un eslabon de este mismo pliegue: una sola definicion.
pub(crate) fn cabeza_de_retiradas(ids: &[Digest]) -> Digest {
    ids.iter().fold([BaseElement::ZERO; 4], |c, id| eslabon_de_retirada(c, *id))
}

/// §N: una identidad, una cuenta en la vida del libro (SECURITY.md §3.13). Los dos primeros
/// casos medidos en privado antes del arreglo -un tercero sin la clave abria otra cuenta con la
/// identidad de Bob, y tras una recuperacion la clave vieja abria otra cuenta con la identidad
/// retirada y cobraba ahi un pendiente en vuelo- quedan aqui invertidos: el escenario se monta
/// entero, y la apertura se rechaza sin mover nada.
#[cfg(test)]
mod tests_una_identidad {
    use super::*;
    use crate::tests_support::*;
    use stark_experiment::native::{derive_leaf_salt, view_id_of};

    const SK_TERCERO: u64 = 0x3A11;
    const SK_CAROL: u64 = 0xCA201;
    const SK_ALICE_NUEVA: u64 = 0xA11CE_2;

    /// Lo que un rechazo no puede mover: el censo, la raiz, el registro y las cuentas.
    fn foto(l: &SovereignLayer) -> (u64, Digest, usize, Digest, usize) {
        (l.next_index, l.state_root(), l.log.len(), l.log.head(), l.account_count())
    }

    /// La vista y la sal de una clave estrecha: lo que abre quien tiene esa clave.
    fn abrir_con(l: &mut SovereignLayer, id: Digest, sk: u64) -> Result<AccountIndex, LayerError> {
        l.open_with_id(id, view_id_of(BaseElement::new(sk)), derive_leaf_salt(BaseElement::new(sk)))
    }

    /// Un pendiente en vuelo de `importe` desde `emisor` hacia la identidad `hacia`.
    fn enviar(
        l: &mut SovereignLayer,
        emisor: AccountIndex,
        sk: u64,
        hacia: Digest,
        importe: u64,
    ) -> crate::two_phase::SendReceipt {
        let e = state_of(l, emisor);
        let envio = l
            .send(BaseElement::new(sk), emisor, &e, hacia, salt_de(0x7E57), importe)
            .expect("enviar");
        l.apply_send(&envio, emisor, &e, importe).expect("aplicar el envio");
        envio
    }

    /// El primer caso, invertido: con la identidad de Bob y su propia vista y su propia sal, un
    /// tercero SIN la clave abria una segunda cuenta (`Ok`, otro indice). Ahora no, y el
    /// rechazo no avanza el censo, ni la raiz, ni el registro.
    #[test]
    fn abrir_una_identidad_con_cuenta_se_rechaza_sin_tocar_nada() {
        let mut l = new_layer();
        let bob = open_and_fund(&mut l, SK_BOB, 1_000);
        let id_bob = l.public_id_of(bob).expect("bob");
        let antes = foto(&l);
        let r = abrir_con(&mut l, id_bob, SK_TERCERO);
        assert!(
            matches!(r, Err(LayerError::IdentityUnavailable)),
            "CRITICO: un tercero sin la clave abrio otra cuenta con la identidad de Bob: {r:?}"
        );
        assert_eq!(foto(&l), antes, "el rechazo movio el libro");
        assert_eq!(l.balance_of(bob), Some(1_000));
    }

    /// §90: la clave estrecha `sk` y la ancha `[sk, 0, 0, 0]` dan la MISMA identidad, asi que son
    /// una sola cuenta, se abra por la via que se abra primero. La puerta mira la identidad, no la
    /// clave ni la via.
    #[test]
    fn la_estrecha_y_la_ancha_rellenada_son_una_sola_identidad() {
        let rellena = [BaseElement::new(SK_ALICE), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO];
        let mut a = new_layer();
        a.open_account_checked(BaseElement::new(SK_ALICE)).expect("la estrecha");
        let r = a.open_account_wide_checked(rellena);
        assert!(matches!(r, Err(LayerError::IdentityUnavailable)), "la ancha tras la estrecha: {r:?}");
        let mut b = new_layer();
        b.open_account_wide_checked(rellena).expect("la ancha");
        let r = b.open_account_checked(BaseElement::new(SK_ALICE));
        assert!(matches!(r, Err(LayerError::IdentityUnavailable)), "la estrecha tras la ancha: {r:?}");
    }

    /// La identidad es el `Digest` ENTERO: dos que solo comparten el primer elemento -y por tanto
    /// la posicion de sondeo- son dos identidades, y las dos abren.
    #[test]
    fn una_identidad_distinta_con_el_mismo_primer_elemento_abre() {
        let mut l = new_layer();
        let x = BaseElement::new(0xC011);
        let (vid, sal) = ([BaseElement::new(9); 4], [BaseElement::new(7); 4]);
        let e = BaseElement::new;
        let a = l.open_with_id([x, e(1), e(1), e(1)], vid, sal).expect("la primera");
        let b = l
            .open_with_id([x, e(2), e(1), e(1)], vid, sal)
            .expect("CRITICO: el mismo primer elemento no es la misma identidad");
        let c = l
            .open_with_id([x, e(1), e(1), e(2)], vid, sal)
            .expect("CRITICO: ni difiriendo solo en el ultimo");
        assert!(a != b && b != c && a != c, "tres cuentas, tres indices: {a} {b} {c}");
    }

    /// Tras recuperar X->Y, la identidad NUEVA tampoco abre otra cuenta. La cuenta recuperada
    /// sigue en la posicion de X, en su cadena de sondeo: buscar desde `Y[0] mod capacidad` no
    /// la veria.
    #[test]
    fn tras_recuperar_la_identidad_nueva_tampoco_abre_otra_cuenta() {
        let mut l = new_layer();
        let alice = open_and_fund(&mut l, SK_ALICE, 1_000);
        let y = derive_public_id(BaseElement::new(SK_ALICE_NUEVA));
        recover_delegated(&mut l, alice, y);
        let antes = foto(&l);
        let r = abrir_con(&mut l, y, SK_ALICE_NUEVA);
        assert!(
            matches!(r, Err(LayerError::IdentityUnavailable)),
            "CRITICO: la identidad recuperada abrio una segunda cuenta: {r:?}"
        );
        assert_eq!(foto(&l), antes, "el rechazo movio el libro");
    }

    /// El segundo caso, invertido: Alice tiene un pendiente en vuelo de 7.000 hacia su identidad X;
    /// los custodios la recuperan hacia Y porque la clave vieja se dio por robada. Antes, el
    /// ladron abria otra cuenta con X y cobraba ahi el pendiente. Ahora la apertura se rechaza
    /// sin mover nada, el pendiente sigue en vuelo, y al caducar el emisor lo recupera con su
    /// reembolso: el suministro cuadra.
    #[test]
    fn el_robo_tras_una_recuperacion_no_abre_y_el_pendiente_vuelve_al_emisor() {
        let mut l = new_layer();
        l.set_refund_ttl(1);
        let alice = open_and_fund(&mut l, SK_ALICE, 1_000);
        let carol = open_and_fund(&mut l, SK_CAROL, 500_000);
        let x = derive_public_id(BaseElement::new(SK_ALICE));
        let envio = enviar(&mut l, carol, SK_CAROL, x, 7_000);
        recover_delegated(&mut l, alice, derive_public_id(BaseElement::new(SK_ALICE_NUEVA)));

        let antes = foto(&l);
        let r = abrir_con(&mut l, x, SK_ALICE);
        assert!(
            matches!(r, Err(LayerError::IdentityUnavailable)),
            "CRITICO: la clave robada abrio otra cuenta con la identidad retirada: {r:?}"
        );
        assert_eq!(foto(&l), antes, "el rechazo movio el libro");
        assert_eq!(l.total_pending(), 7_000, "el pendiente sigue en vuelo");

        let ec = state_of(&l, carol);
        let reembolso = l
            .refund(
                BaseElement::new(SK_CAROL), carol, &ec,
                envio.notice.position, x, envio.notice.salt, 7_000,
            )
            .expect("materiales del reembolso");
        l.apply_refund(&reembolso).expect("el emisor lo recupera al caducar");
        assert_eq!(l.balance_of(carol), Some(500_000));
        assert_eq!(l.total_pending(), 0);
        let saldos: u64 = [alice, carol].iter().map(|i| l.balance_of(*i).expect("cuenta")).sum();
        assert_eq!(saldos + l.total_pending(), l.total_supply(), "el suministro cuadra");
    }

    /// La D-5 de `doc/ROTACION_DE_CUENTAS.md`, medida y no deducida: tras recuperar X->Y, la clave
    /// nueva genera la prueba de cobro del pendiente hacia X, y aplicarla da `StaleState`. Ni el
    /// saldo de Alice ni el pendiente se mueven. Es una medida fijada, no un falsador del §N: la
    /// tumbaria una rotacion que redirigiera los pendientes, que es lo que la D-5 discute.
    #[test]
    fn la_clave_nueva_no_cobra_el_pendiente_hacia_la_vieja() {
        let mut l = new_layer();
        let alice = open_and_fund(&mut l, SK_ALICE, 1_000);
        let carol = open_and_fund(&mut l, SK_CAROL, 500_000);
        let x = derive_public_id(BaseElement::new(SK_ALICE));
        let envio = enviar(&mut l, carol, SK_CAROL, x, 7_000);
        recover_delegated(&mut l, alice, derive_public_id(BaseElement::new(SK_ALICE_NUEVA)));

        let ea = state_of(&l, alice);
        let cobro = l
            .claim(BaseElement::new(SK_ALICE_NUEVA), alice, &ea, &envio.notice)
            .expect("la prueba de cobro se genera");
        let r = l.apply_claim(&cobro, alice, &ea, &envio.notice);
        assert!(matches!(r, Err(LayerError::StaleState)), "la clave nueva cobro el pendiente hacia X: {r:?}");
        assert_eq!(l.balance_of(alice), Some(1_000));
        assert_eq!(l.total_pending(), 7_000);
    }
}
