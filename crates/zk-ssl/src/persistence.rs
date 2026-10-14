//! Persistencia del ledger: apertura, recuperación y verificación de
//! integridad.
//!
//! La pieza que importa aquí no es guardar y cargar, sino **detectar un
//! ledger corrupto antes de operar sobre él**: al abrir se reconstruyen
//! los árboles, se recalculan sus raíces y se comparan con las del
//! último cierre. Si no coinciden, el arranque falla.
//!
//! Sin esa comprobación, el nodo generaría pruebas perfectamente válidas
//! de transiciones sobre un ledger que no es el real —
//! criptográficamente indetectable desde fuera.
use super::*;

impl SovereignLayer {
    /// Abre una capa **persistente**, creando el ledger si no existe.
    ///
    /// Al recuperar un ledger existente, **reconstruye los árboles y
    /// verifica que sus raíces coinciden con las guardadas**. Si no
    /// coinciden, falla en vez de operar sobre un estado corrupto.
    pub fn open(
        path: &str,
        custodian_set_root: Digest,
        governance_set_root: Digest,
        regulatory_limit: u64,
        max_supply: u64,
        max_accounts: u64,
    ) -> Result<Self, LayerError> {
        Self::open_encrypted(
            path,
            custodian_set_root,
            governance_set_root,
            regulatory_limit,
            max_supply,
            max_accounts,
            None,
        )
    }

    /// Abre una capa persistente **con cifrado en reposo**.
    ///
    /// La clave la aporta el operador y **no se guarda junto a los datos**: guardarla al lado no
    /// protegería nada. Eso significa que el nodo **no puede reiniciar solo** — alguien tiene que
    /// introducirla. §702 (RFC-0001): se fija al KDF que el libro guarda en claro, y un libro de la
    /// versión 1 se verifica y se migra a la 2 al abrirlo (`abrir_con_su_kdf`, al final).
    ///
    /// ⚠️ Protege contra el robo del disco o de una copia. **No contra el
    /// operador**, que ve los saldos en memoria.
    #[allow(clippy::too_many_arguments)]
    pub fn open_encrypted(
        path: &str,
        custodian_set_root: Digest,
        governance_set_root: Digest,
        regulatory_limit: u64,
        max_supply: u64,
        max_accounts: u64,
        key: Option<crate::crypto::LedgerKey>,
    ) -> Result<Self, LayerError> {
        // §668: el mismo tope que `new`, como error y no como panico.
        if max_supply > crate::MAX_SUMINISTRO {
            return Err(LayerError::SupplyCapExceeded { cap: crate::MAX_SUMINISTRO, would_be: max_supply });
        }
        let (db, fijada, de_v1) = Self::abrir_con_su_kdf(path, key.as_ref())?;

        let mut layer = Self {
            accounts: SparseTree::new(),
            pending: SparseTree::new(),
            next_pending: 0,
            reserved_pending: std::collections::BTreeSet::new(),
            pending_amounts: HashMap::new(),
            pending_meta: HashMap::new(),
            pending_meta_tree: crate::sparse_tree::SparseTree::new(),
            refund_ttl: crate::DEFAULT_REFUND_TTL,
            records: HashMap::new(),
            next_index: 0,
            custodian_set_root,
            governance_set_root,
            governance_change_count: 0,
            custodian_uses: 0,
            max_custodian_uses: crate::DEFAULT_MAX_CUSTODIAN_USES,
            total_supply: 0,
            frozen: SparseTree::with_depth(FROZEN_DEPTH),
            freeze_count: 0,
            log: TransitionLog::new(),
            log_persisted: 0,
            consumos: SparseTree::with_depth(crate::consumo::CONS_DEPTH),
            consumos_orden: Vec::new(),
            cons_persisted: 0,
            recovery_count: 0,
            regulatory_limit,
            max_supply,
            max_accounts,
            options: proof_options(),
            db: Some(db),
            key: fijada,
            identidades: std::collections::HashSet::new(),
            retiradas: Vec::new(),
            retir_cabeza: [BaseElement::ZERO; 4],
            retir_persisted: 0,
        };

        if layer.has_existing_ledger()? {
            layer.cargar_y_migrar(path, de_v1, key.as_ref())?;
        } else {
            // Ledger nuevo: escribir los parametros del sistema.
            layer.commit(&[], None)?;
        }
        Ok(layer)
    }

    pub(crate) fn db(&self) -> Option<&sled::Db> {
        self.db.as_ref()
    }

    /// Prepara un valor para escribirlo: cifrado si hay clave, en claro
    /// si no.
    fn seal(&self, plain: Vec<u8>) -> Result<Vec<u8>, LayerError> {
        match &self.key {
            None => Ok(plain),
            Some(k) => Ok(k.seal(&plain)?),
        }
    }

    pub(crate) fn has_existing_ledger(&self) -> Result<bool, LayerError> {
        match self.db() {
            None => Ok(false),
            Some(db) => Ok(db
                .contains_key(b"meta:custodians")
                .map_err(|e| StoreError::Io(e.to_string()))?),
        }
    }

    /// Reconstruye el estado y **verifica su integridad**.
    pub(crate) fn load(&mut self) -> Result<(), LayerError> {
        let db = match self.db() {
            None => return Ok(()),
            Some(d) => d.clone(),
        };
        // La clave se clona para que los cierres no tomen prestado `self`,
        // que se muta durante la carga.
        let key = self.key.clone();
        let unseal_one = move |v: sled::IVec| -> Result<Vec<u8>, LayerError> {
            match &key {
                None => Ok(v.to_vec()),
                Some(k) => Ok(k.open(&v)?),
            }
        };
        let get = |k: &[u8]| -> Result<Option<Vec<u8>>, LayerError> {
            match db.get(k).map_err(|e| StoreError::Io(e.to_string()))? {
                None => Ok(None),
                Some(v) => unseal_one(v).map(Some),
            }
        };
        let need = |k: &'static str, v: Option<Vec<u8>>| -> Result<Vec<u8>, LayerError> {
            v.ok_or_else(|| StoreError::Malformed(format!("falta la clave {k}")).into())
        };
        // Cada valor pasa por `unseal_one`: descifrado si hay clave, tal cual
        // si no. Con contrasena incorrecta el cifrado autenticado falla
        // aqui, en vez de producir datos plausibles pero falsos.

        // --- Parametros del sistema ---
        // El conjunto de GOBERNANZA es inmutable: se verifica.
        let stored_gov =
            digest_from_bytes(&need("meta:governance", get(b"meta:governance")?)?)?;
        if stored_gov != self.governance_set_root {
            return Err(StoreError::ParameterMismatch {
                what: "conjunto de gobernanza",
            }
            .into());
        }

        // El conjunto de CUSTODIOS es mutable por gobernanza: se LEE del
        // ledger, no se compara con el argumento. Comparar aqui obligaria
        // a que quien abre el ledger supiera de antemano el conjunto
        // vigente, que es justo lo que un cambio de gobernanza modifica.
        self.custodian_set_root =
            digest_from_bytes(&need("meta:custodians", get(b"meta:custodians")?)?)?;
        self.governance_change_count = match get(b"meta:gov_changes")? {
            Some(v) => u64::from_le_bytes(
                v.as_slice()
                    .try_into()
                    .map_err(|_| StoreError::Malformed("contador de gobernanza".into()))?,
            ),
            None => 0,
        };
        self.custodian_uses = match get(b"meta:cust_uses")? {
            Some(v) => u64::from_le_bytes(
                v.as_slice()
                    .try_into()
                    .map_err(|_| StoreError::Malformed("contador de gobernanza".into()))?,
            ),
            None => 0,
        };
        self.max_custodian_uses = match get(b"meta:cust_max")? {
            Some(v) => u64::from_le_bytes(
                v.as_slice()
                    .try_into()
                    .map_err(|_| StoreError::Malformed("maximo de custodios".into()))?,
            ),
            // Un ledger anterior a este campo: el valor por defecto es lo
            // unico que se puede suponer, y es lo que habia.
            None => crate::DEFAULT_MAX_CUSTODIAN_USES,
        };
        // ⚠️ **Los importes de los pendientes se restauran del disco.**
        //
        // Sin esto, tras un reinicio `total_pending()` valdria cero y la
        // invariante global —saldos + pendientes == suministro— pareceria
        // rota. El compilador lo detecto al anadir el campo.
        // Mismo patron que `froz:` y `pend:`: `unseal_one` es un cierre
        // porque `self` se muta durante la carga y no puede prestarse.
        self.pending_amounts.clear();
        self.pending_meta.clear();
        for item in db.scan_prefix(b"pmeta:") {
            let (k, v) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            let v = unseal_one(v)?;
            if v.len() != 16 {
                continue; // vacío = sin meta (vaciada o legado)
            }
            let pos = u64::from_le_bytes(
                k[6..]
                    .try_into()
                    .map_err(|_| StoreError::Malformed("posicion de meta".into()))?,
            );
            let sender = u64::from_le_bytes(v[..8].try_into().unwrap());
            let born = u64::from_le_bytes(v[8..].try_into().unwrap());
            self.pending_meta.insert(pos, (sender, born));
        }
        // §388: el arbol de meta se recompone de lo leido y se comprueba mas
        // abajo contra `root:pmeta`, igual que `pend:` contra `root:pending`.
        self.pending_meta_tree.rebuild_from(
            self.pending_meta
                .iter()
                .map(|(p, (s, b))| (*p, zk_ssl_hash::meta_pendiente_hoja(*s, *b))),
        );
        if let Some(v) = db.get(b"meta:refund_ttl").map_err(|e| StoreError::Io(e.to_string()))? {
            let v = unseal_one(v)?;
            self.refund_ttl = u64::from_le_bytes(
                v.as_slice()
                    .try_into()
                    .map_err(|_| StoreError::Malformed("refund_ttl".into()))?,
            );
        }
        for item in db.scan_prefix(b"pamt:") {
            let (k, v) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            let v = unseal_one(v)?;
            let pos = u64::from_le_bytes(
                k[5..]
                    .try_into()
                    .map_err(|_| StoreError::Malformed("posicion de pendiente".into()))?,
            );
            let importe = u64::from_le_bytes(
                v.as_slice()
                    .try_into()
                    .map_err(|_| StoreError::Malformed("importe de pendiente".into()))?,
            );
            // Un importe cero significa **cobrado**: no cuenta como
            // pendiente, igual que la hoja vacia no cuenta como ocupada.
            if importe != 0 {
                self.pending_amounts.insert(pos, importe);
            }
        }

        self.next_pending = match get(b"meta:next_pending")? {
            Some(v) => u64::from_le_bytes(
                v.as_slice()
                    .try_into()
                    .map_err(|_| StoreError::Malformed("contador de gobernanza".into()))?,
            ),
            None => 0,
        };
        let stored_limit = u64::from_le_bytes(
            need("meta:limit", get(b"meta:limit")?)?
                .as_slice()
                .try_into()
                .map_err(|_| StoreError::Malformed("limite".into()))?,
        );
        if stored_limit != self.regulatory_limit {
            return Err(StoreError::ParameterMismatch {
                what: "limite regulatorio",
            }
            .into());
        }

        let stored_cap = u64::from_le_bytes(
            need("meta:max_supply", get(b"meta:max_supply")?)?
                .as_slice()
                .try_into()
                .map_err(|_| StoreError::Malformed("tope".into()))?,
        );
        if stored_cap != self.max_supply {
            return Err(StoreError::ParameterMismatch {
                what: "tope de emision",
            }
            .into());
        }

        let stored_accts = match get(b"meta:max_accounts")? {
            Some(v) => u64::from_le_bytes(
                v.as_slice()
                    .try_into()
                    .map_err(|_| StoreError::Malformed("tope de cuentas".into()))?,
            ),
            // Ledger creado antes de existir el tope: se acepta el
            // declarado en vez de inventar un valor.
            None => self.max_accounts,
        };
        if stored_accts != self.max_accounts {
            return Err(StoreError::ParameterMismatch {
                what: "tope de cuentas",
            }
            .into());
        }

        self.total_supply = u64::from_le_bytes(
            need("meta:supply", get(b"meta:supply")?)?
                .as_slice()
                .try_into()
                .map_err(|_| StoreError::Malformed("suministro".into()))?,
        );
        // El contador de recuperaciones. Si faltara, se asume cero: un
        // ledger creado antes de existir esta funcionalidad no ha tenido
        // ninguna.
        self.freeze_count = match get(b"meta:freezes")? {
            Some(v) => u64::from_le_bytes(
                v.as_slice()
                    .try_into()
                    .map_err(|_| StoreError::Malformed("contador de congelaciones".into()))?,
            ),
            None => 0,
        };

        self.recovery_count = match get(b"meta:recoveries")? {
            Some(v) => u64::from_le_bytes(
                v.as_slice()
                    .try_into()
                    .map_err(|_| StoreError::Malformed("contador de recuperaciones".into()))?,
            ),
            None => 0,
        };

        self.next_index = u64::from_le_bytes(
            need("meta:next_index", get(b"meta:next_index")?)?
                .as_slice()
                .try_into()
                .map_err(|_| StoreError::Malformed("next_index".into()))?,
        );

        // --- Cuentas ---
        // ¿Ledger MIGRADO (B13/B14)? La presencia del marcador decide la
        // geometria de reconstruccion: hoja envuelta + frozen a 32. No va
        // sellado: su semantica es presencia, y las raices almacenadas
        // arbitran — marcador falso o ausente => IntegrityFailure.
        let migrated = db
            .get(b"meta:migrated")
            .map_err(|e| StoreError::Io(e.to_string()))?
            .is_some()
            // …o nacido ya en el mundo nuevo (flip D4): misma geometría.
            || db
                .get(b"meta:geometry_v7")
                .map_err(|e| StoreError::Io(e.to_string()))?
                .is_some();
        // §221: se recogen las hojas y el arbol se construye DE ABAJO
        // ARRIBA una sola vez, en vez de N `set_leaf` a 32 merges cada
        // uno. §217 (banco B.4) midio que ese `set_leaf` es el **82 %**
        // del arranque, y que a 1e5 cuentas son 29,68 s.
        let mut hojas: Vec<(u64, Digest)> = Vec::new();
        for item in db.scan_prefix(b"acct:") {
            let (k, v) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            let v = unseal_one(v)?;
            let idx = u64::from_le_bytes(
                k[5..]
                    .try_into()
                    .map_err(|_| StoreError::Malformed("indice de cuenta".into()))?,
            );
            let (public_id, balance, nonce, view_id, leaf_salt) =
                crate::store::record_from_bytes_v3(&v)?;
            hojas.push((
                idx,
                if migrated {
                    stark_experiment::native::native_leaf_salted(
                        public_id, BaseElement::new(balance), nonce, leaf_salt,
                    )
                } else {
                    native_leaf(public_id, BaseElement::new(balance), nonce)
                },
            ));
            self.records.insert(
                idx,
                AccountRecord {
                    public_id,
                    balance,
                    nonce,
                    view_id,
                    leaf_salt,
                },
            );
        }
        self.accounts.rebuild_from(hojas);

        // --- Registro de transiciones ---
        // El registro se lee ORDENADO por numero de secuencia: sled
        // devuelve las claves en orden lexicografico, y los u64 en
        // little-endian NO lo respetan.
        let mut entradas: Vec<(u64, Vec<u8>)> = Vec::new();
        for item in db.scan_prefix(b"log:") {
            let (k, v) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            let v = unseal_one(v)?;
            let seq = u64::from_le_bytes(
                k[4..]
                    .try_into()
                    .map_err(|_| StoreError::Malformed("secuencia del registro".into()))?,
            );
            entradas.push((seq, v));
        }
        entradas.sort_by_key(|(s, _)| *s);
        self.log = TransitionLog::from_entries(
            entradas
                .into_iter()
                .map(|(_, v)| crate::store::log_entry_from_bytes(&v))
                .collect::<Result<Vec<_>, _>>()?,
        );
        // Lo reconstruido viene del disco: esas entradas YA estan
        // escritas. Unico punto donde la marca se adelanta.
        self.log_persisted = self.log.len();

        if migrated {
            // El arbol de congelados migrado vive a profundidad 32.
            self.frozen = SparseTree::with_depth(crate::migration::FROZEN_DEPTH_POST);
        } else {
            // Mundo viejo en disco: frozen se reconstruye a 24 (PRE), o las
            // raíces almacenadas no cuadrarían (§128).
            self.frozen = SparseTree::with_depth(crate::migration::FROZEN_DEPTH_PRE);
        }
        let mut congeladas: Vec<(u64, Digest)> = Vec::new();
        for item in db.scan_prefix(b"froz:") {
            let (k, v) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            let v = unseal_one(v)?;
            let idx = u64::from_le_bytes(
                k[5..]
                    .try_into()
                    .map_err(|_| StoreError::Malformed("indice de congelada".into()))?,
            );
            congeladas.push((idx, digest_from_bytes(&v)?));
        }
        // Las congelaciones son raras, asi que aqui el ahorro es
        // pequeño; se hace igual por coherencia y porque un ledger con
        // muchas congelaciones pagaria lo mismo que el de cuentas.
        self.frozen.rebuild_from(congeladas);

        // Hojas del arbol de PENDIENTES.
        //
        // Sin esto, un reinicio borraria las transferencias sin reclamar:
        // el dinero saldria de la cuenta del pagador y **no llegaria a
        // ninguna parte**.
        let mut pendientes: Vec<(u64, Digest)> = Vec::new();
        for item in db.scan_prefix(b"pend:") {
            let (k, v) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            let v = unseal_one(v)?;
            let pos = u64::from_le_bytes(
                k[5..]
                    .try_into()
                    .map_err(|_| StoreError::Malformed("posicion de pendiente".into()))?,
            );
            pendientes.push((pos, digest_from_bytes(&v)?));
        }
        self.pending.rebuild_from(pendientes);

        // --- MIGRACION: el arbol de nullificadores fue retirado ---
        //
        // Los ledgers escritos antes de la retirada de la via de un paso
        // (`AUDITORIA.md` §32) contienen claves `null:{pos}` y una raiz
        // `root:nullifier`. Nada las lee ya, pero **descartarlas sin
        // verificarlas seria cargar en silencio**: se reconstruye el
        // arbol legado y, tras la verificacion de integridad de abajo, se
        // comprueba contra su raiz guardada y se elimina en un lote
        // atomico. Un ledger nuevo no entra aqui.
        let mut legacy_null = SparseTree::new();
        let mut legacy_null_keys: Vec<Vec<u8>> = Vec::new();
        for item in db.scan_prefix(b"null:") {
            let (k, v) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            let v = unseal_one(v)?;
            let pos = u64::from_le_bytes(
                k[5..]
                    .try_into()
                    .map_err(|_| StoreError::Malformed("posicion de nullifier".into()))?,
            );
            legacy_null.set_leaf(pos, digest_from_bytes(&v)?);
            legacy_null_keys.push(k.to_vec());
        }

        // --- VERIFICACION DE INTEGRIDAD ---
        // Reconstruir no basta: hay que comprobar que lo reconstruido es
        // lo que se guardo. Un ledger corrupto que se cargue en silencio
        // haria que el nodo generase pruebas validas de transiciones
        // sobre un estado que no es el real.
        let stored_state = digest_from_bytes(&need("root:state", get(b"root:state")?)?)?;
        if self.accounts.root() != stored_state {
            return Err(StoreError::IntegrityFailure {
                what: "arbol de cuentas",
            }
            .into());
        }

        // --- ARBOL DE PENDIENTES ---
        //
        // La raiz del arbol de pendientes se guarda en `root:pending` desde
        // el §387 y se comprueba aqui igual que `root:state`. Antes se
        // reconstruia de `pend:` y se creia: era una afirmacion sin
        // falsador dentro del invariante sagrado (BACKLOG 21, AUDITORIA
        // §387). Un ledger anterior al §387 no lleva la clave y NO ABRE
        // (`need` devuelve `Malformed`): fail-closed, sin era silenciosa.
        // Migrar un libro asi es una operacion explicita y aparte.
        //
        // LIMITE DECLARADO: la hoja es el compromiso
        // H(H(receptor, sal), importe) y la capa no guarda receptor ni sal,
        // asi que esta raiz cubre `pend:` y NO `pamt:` ni `pmeta:`. Una
        // mentira aislada en `pamt:` la caza la conservacion de abajo; una
        // coordinada con `meta:supply` sigue sin falsador.
        let stored_pending =
            digest_from_bytes(&need("root:pending", get(b"root:pending")?)?)?;
        if self.pending.root() != stored_pending {
            return Err(StoreError::IntegrityFailure {
                what: "arbol de pendientes",
            }
            .into());
        }

        // --- META DE LOS PENDIENTES ---
        //
        // §388 (punto 41): `pmeta:` decide A QUIEN y CUANDO vuelve el dinero
        // de un pendiente y hasta aqui se creia. Su arbol se recompuso arriba
        // de lo leido; la raiz guardada tiene que cuadrar, y un ledger sin
        // `root:pmeta` NO ABRE (fail-closed, como `root:pending` en el §387).
        let stored_meta =
            digest_from_bytes(&need("root:pmeta", get(b"root:pmeta")?)?)?;
        if self.pending_meta_tree.root() != stored_meta {
            return Err(StoreError::IntegrityFailure {
                what: "meta de pendientes",
            }
            .into());
        }

        // --- ARBOL DE CONGELADOS ---
        //
        // §391 (punto 45): `froz:` decide QUIEN puede gastar y hasta aqui se
        // creia: el arbol se recompuso arriba de lo leido. La raiz guardada
        // tiene que cuadrar -borrar o anadir una clave `froz:` en disco es
        // descongelar o congelar sin custodios-, y un ledger sin `root:froz`
        // NO ABRE (fail-closed, como `root:pending` y `root:pmeta`).
        let stored_froz =
            digest_from_bytes(&need("root:froz", get(b"root:froz")?)?)?;
        if self.frozen.root() != stored_froz {
            return Err(StoreError::IntegrityFailure {
                what: "arbol de congelados",
            }
            .into());
        }

        // --- ARBOL DE CONSUMOS (RFC-0006, E1) ---
        //
        // §413: `cons:` se reconstruye de disco y su raiz guardada tiene que
        // cuadrar; un libro sin `root:cons` NO ABRE (fail-closed, como las
        // cinco anteriores). Migrar un libro anterior es operacion aparte.
        let mut consumos = SparseTree::with_depth(crate::consumo::CONS_DEPTH);
        let mut orden: Vec<(u64, Digest)> = Vec::new();
        for item in db.scan_prefix(b"cons:") {
            let (k, v) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            let v = unseal_one(v)?;
            let pos = u64::from_le_bytes(
                k[5..]
                    .try_into()
                    .map_err(|_| StoreError::Malformed("posicion de consumo".into()))?,
            );
            let c = digest_from_bytes(&v)?;
            consumos.set_leaf(pos, c);
            orden.push((pos, c));
        }
        orden.sort_by_key(|(p, _)| *p);
        let stored_cons =
            digest_from_bytes(&need("root:cons", get(b"root:cons")?)?)?;
        if consumos.root() != stored_cons {
            return Err(StoreError::IntegrityFailure {
                what: "arbol de consumos",
            }
            .into());
        }
        self.cons_persisted = orden.len();
        self.consumos = consumos;
        self.consumos_orden = orden;

        // --- REGISTRO DE TRANSICIONES ---
        //
        // §392 (punto 46/48): el registro se reconstruyo arriba de `log:` y hasta
        // aqui se creia, mientras su `head()` va FIRMADO en la cabeza. Se recomputa
        // la cadena entrada a entrada (`verify_chain`, el mismo juez que ya usa la
        // instantanea) y la raiz guardada tiene que cuadrar con la cabeza: una copia
        // ATRASADA verifica la cadena y NO cuadra la raiz. Un ledger sin `root:log`
        // NO ABRE (fail-closed, como las otras cuatro raices).
        if self.log.verify_chain().is_err() {
            return Err(StoreError::IntegrityFailure {
                what: "registro de transiciones",
            }
            .into());
        }
        let stored_log =
            digest_from_bytes(&need("root:log", get(b"root:log")?)?)?;
        if self.log.head() != stored_log {
            return Err(StoreError::IntegrityFailure {
                what: "registro de transiciones",
            }
            .into());
        }

        // --- CONTADORES CUSTODIADOS ---
        //
        // §393 (punto 46): `meta:freezes`, `meta:recoveries` y `meta:gov_changes` no
        // son estadistica: son el NONCE anti-repeticion de cada operacion de
        // custodios. Entran en `commit_operation` como (count_old, count_new), y una
        // pareja de pruebas vieja solo deja de servir porque el contador avanzo.
        // Se leian con `None => 0` y nadie los cruzaba: rebobinar uno en reposo
        // devolvia a la vida una autorizacion ya gastada. El registro -integro desde
        // el §392- los deriva: cada una de las tres operaciones appendea su entrada
        // en la MISMA funcion que mueve su contador.
        //
        // UN what para los tres: la propiedad es UNA (un contador custodiado es lo
        // que el registro dice), y `what` es &'static str -no puede decir cual ni
        // con que cifras-. Cual fue se lee del registro, que es justo lo que la
        // puerta acaba de verificar.
        let mut n_freeze = 0u64;
        let mut n_recovery = 0u64;
        let mut n_governance = 0u64;
        for e in self.log.entries() {
            match e.kind {
                crate::log::OpKind::Freeze => n_freeze += 1,
                crate::log::OpKind::Recovery => n_recovery += 1,
                crate::log::OpKind::Governance => n_governance += 1,
                _ => {}
            }
        }
        if self.freeze_count != n_freeze
            || self.recovery_count != n_recovery
            || self.governance_change_count != n_governance
        {
            return Err(StoreError::IntegrityFailure {
                what: "contadores custodiados",
            }
            .into());
        }
        // §802: las identidades retiradas, una por recuperacion del registro (al final del fichero).
        self.cargar_retiradas(&db, n_recovery)?;

        // --- CUOTA DE CUSTODIOS ---
        //
        // §394 (punto 46, ultimo escalon): `meta:cust_uses` no es un nonce -no entra
        // en ningun `commit_operation`-: es la CUOTA del conjunto de custodios, y
        // rebobinarla en reposo le devuelve cupo agotado. La deriva el registro
        // (`usos_custodiados`, el unico productor). El TOPE (`meta:cust_max`) NO es
        // derivable: es politica, no historia, y queda declarado en el asiento.
        if self.custodian_uses != self.log.usos_custodiados() {
            return Err(StoreError::IntegrityFailure {
                what: "cuota de custodios",
            }
            .into());
        }

        // --- CONSERVACION DEL DINERO ---
        //
        // El invariante sagrado: sum(saldos) + sum(pendientes vivos) ==
        // suministro.
        //
        // `root:state` cubre los saldos -la hoja lleva el balance-, pero el
        // SUMINISTRO es un escalar que NINGUNA raiz cubre: se lee de
        // `meta:supply` y hasta aqui se creia. Esta puerta es el unico
        // eslabon que ata ese escalar al arbol que si esta cubierto.
        //
        // El AIR ya guarda la conservacion POR TRANSICION (nueve testigos
        // negativos). Lo que no guardaba nadie es que el AGREGADO cargado de
        // disco siga cuadrando: el hueco es de REAPERTURA, no de transicion.
        //
        // Va ANTES del `match` de `root:nullifier` porque ese `match`
        // ESCRIBE -aplica un lote de limpieza-: fail-closed es verificar
        // antes de mutar.
        //
        // LIMITE DECLARADO: `pamt:` NO tiene raiz guardada, asi que la suma
        // de pendientes esta tan sin cubrir como el suministro. Esta puerta
        // caza cualquier mentira AISLADA, no una coordinada en las dos.
        //
        // Se suma en u128 y no con `total_pending()`: ese devuelve u64, y un
        // `pamt:` corrupto podria DESBORDAR EN SILENCIO en release y colar
        // la mentira. El operador se deriva del invariante.
        let saldos: u128 = self.records.values().map(|r| r.balance as u128).sum();
        let pendientes: u128 = self.pending_amounts.values().map(|v| *v as u128).sum();
        if saldos + pendientes != self.total_supply as u128 {
            return Err(StoreError::IntegrityFailure {
                what: "conservacion del suministro",
            }
            .into());
        }

        // La raiz legada se verifica ANTES de migrar: un arbol legado
        // corrupto se trata igual que uno vivo — el arranque se detiene.
        match get(b"root:nullifier")? {
            Some(v) => {
                if legacy_null.root() != digest_from_bytes(&v)? {
                    return Err(StoreError::IntegrityFailure {
                        what: "arbol de nullifiers (legado)",
                    }
                    .into());
                }
                let mut limpieza = sled::Batch::default();
                for k in &legacy_null_keys {
                    limpieza.remove(k.as_slice());
                }
                limpieza.remove(b"root:nullifier".as_ref());
                db.apply_batch(limpieza)
                    .map_err(|e| StoreError::Io(e.to_string()))?;
                db.flush().map_err(|e| StoreError::Io(e.to_string()))?;
            }
            None if !legacy_null_keys.is_empty() => {
                // Claves sin raiz que las respalde: no hay contra que
                // verificarlas, y descartar sin verificar no se hace.
                return Err(StoreError::IntegrityFailure {
                    what: "nullifiers legados sin raiz guardada",
                }
                .into());
            }
            None => {}
        }

        Ok(())
    }

    /// **Escribe todos los cambios de una operación en UN SOLO LOTE
    /// ATÓMICO.**
    ///
    /// ## El problema que resuelve
    ///
    /// Antes había tres métodos separados y una transferencia hacía
    /// **cuatro llamadas con nueve escrituras**: dos cuentas, un
    /// nullifier y seis valores de metadatos. Si el proceso moría en
    /// medio, quedaban aplicadas unas sí y otras no, y el arranque
    /// siguiente detectaba la inconsistencia y **se detenía hasta
    /// intervención manual**.
    ///
    /// ## Por qué un lote y no un log de escritura anticipada
    ///
    /// `sled` ya garantiza que un `Batch` se aplica entero o no se aplica.
    /// Construir un WAL propio encima sería reimplementar —con más
    /// superficie de fallo— algo que el motor de almacenamiento ya hace.
    ///
    /// ## Qué garantiza, con precisión
    ///
    /// | Momento del fallo | Estado resultante |
    /// |---|---|
    /// | Antes de `apply_batch` | El anterior, coherente |
    /// | Entre `apply_batch` y `flush` | Uno de los dos, **nunca a medias** |
    /// | Después de `flush` | El nuevo, coherente |
    ///
    /// En los tres casos el ledger queda coherente. Lo que se pierde en
    /// el caso intermedio es **durabilidad** —la operación puede haberse
    /// perdido— pero no **integridad**.
    ///
    /// Perder una operación es recuperable: se vuelve a enviar. Un estado
    /// a medias no lo es.
    pub(crate) fn commit(
        &mut self,
        accounts: &[AccountIndex],
        // Hoja del árbol de pendientes que la operación crea o consume.
        //
        // Consumir es escribir la hoja **vacía**: el árbol disperso no
        // distingue "nunca hubo" de "ya se reclamó", y esa distinción no
        // hace falta aquí.
        pending_leaf: Option<(u64, Digest)>,
    ) -> Result<(), LayerError> {
        let db = match self.db() {
            None => return Ok(()),
            Some(d) => d.clone(),
        };

        let mut batch = sled::Batch::default();

        // --- Metadatos y raices ---
        batch.insert(b"meta:custodians".as_ref(), self.seal(digest_to_bytes(&self.custodian_set_root).to_vec())?);
        batch.insert(b"meta:limit".as_ref(), self.seal(self.regulatory_limit.to_le_bytes().to_vec())?);
        batch.insert(b"meta:max_supply".as_ref(), self.seal(self.max_supply.to_le_bytes().to_vec())?);
        batch.insert(b"meta:max_accounts".as_ref(), self.seal(self.max_accounts.to_le_bytes().to_vec())?);
        batch.insert(b"meta:supply".as_ref(), self.seal(self.total_supply.to_le_bytes().to_vec())?);
        batch.insert(b"meta:recoveries".as_ref(), self.seal(self.recovery_count.to_le_bytes().to_vec())?);
        batch.insert(b"meta:freezes".as_ref(), self.seal(self.freeze_count.to_le_bytes().to_vec())?);
        // Partida de nacimiento del mundo nuevo (flip D4): todo estado
        // persistido tras el flip lleva hoja envuelta y frozen-32. Es un
        // marcador de GEOMETRÍA, separado del acta histórica
        // `meta:migrated` (que el guard anti-re-ejecución de la migración
        // lee en exclusiva). Presencia sin sellar; las raíces arbitran.
        batch.insert(b"meta:geometry_v7".as_ref(), vec![7u8]);
        batch.insert(b"meta:governance".as_ref(), self.seal(digest_to_bytes(&self.governance_set_root).to_vec())?);
        batch.insert(b"meta:gov_changes".as_ref(), self.seal(self.governance_change_count.to_le_bytes().to_vec())?);
        // El cupo de custodios. **Si no persistiera, reiniciar el nodo
        // renovaria las intervenciones** y la rotacion no serviria de
        // nada: bastaria con reiniciar para seguir usando un conjunto
        // agotado.
        batch.insert(b"meta:cust_uses".as_ref(), self.seal(self.custodian_uses.to_le_bytes().to_vec())?);
        // ⚠️ **El MAXIMO tambien, no solo el contador.**
        //
        // El cupo son dos cosas. Persistiendo solo el contador, reiniciar
        // devolvia el maximo al valor por defecto y **renovaba un cupo
        // agotado**: quien hubiera restringido a un conjunto de custodios
        // bajo sospecha veia la restriccion levantada por un reinicio.
        //
        // Lo encontro `a_restart_does_not_renew_an_exhausted_custodian_quota`,
        // escrito al revisar los once tests de reinicio que solo comparaban
        // valores. Ver `AUDITORIA.md` §28.
        batch.insert(b"meta:cust_max".as_ref(), self.seal(self.max_custodian_uses.to_le_bytes().to_vec())?);
        // El contador de pendientes. **Si no persistiera, tras reiniciar
        // se reutilizarian posiciones y un pendiente nuevo pisaria a otro
        // sin reclamar**: su receptor perderia el dinero.
        batch.insert(b"meta:next_pending".as_ref(), self.seal(self.next_pending.to_le_bytes().to_vec())?);
        batch.insert(b"meta:refund_ttl".as_ref(), self.seal(self.refund_ttl.to_le_bytes().to_vec())?);
        batch.insert(b"meta:next_index".as_ref(), self.seal(self.next_index.to_le_bytes().to_vec())?);
        batch.insert(b"root:state".as_ref(), self.seal(digest_to_bytes(&self.accounts.root()).to_vec())?);
        // La raiz del arbol de pendientes, al lado de la de cuentas (§387).
        batch.insert(b"root:pending".as_ref(), self.seal(digest_to_bytes(&self.pending.root()).to_vec())?);
        // La raiz del arbol de meta, al lado de las otras dos (§388).
        batch.insert(b"root:pmeta".as_ref(), self.seal(digest_to_bytes(&self.pending_meta_tree.root()).to_vec())?);
        // La raiz del arbol de congelados, al lado de las otras tres (§391).
        batch.insert(b"root:froz".as_ref(), self.seal(digest_to_bytes(&self.frozen.root()).to_vec())?);
        // La raiz del registro de transiciones (su cabeza encadenada), al lado de las
        // otras cuatro (§392, punto 46/48).
        batch.insert(b"root:log".as_ref(), self.seal(digest_to_bytes(&self.log.head()).to_vec())?);
        batch.insert(b"root:cons".as_ref(), self.seal(digest_to_bytes(&self.consumos.root()).to_vec())?);

        // --- Cuentas afectadas ---
        for index in accounts {
            if let Some(r) = self.records.get(index) {
                let mut key = b"acct:".to_vec();
                key.extend_from_slice(&index.to_le_bytes());
                batch.insert(
                    key,
                    self.seal(crate::store::record_to_bytes_v3(&r.public_id, r.balance, r.nonce, &r.view_id, &r.leaf_salt).to_vec())?,
                );
            }
        }

        // --- Registro de transiciones ---
        //
        // Se escriben todas las entradas en cada lote. Sin persistirlo, el
        // operador borraria el historial reiniciando el nodo — que es
        // justo lo que el registro existe para impedir.
        for e in self.log.entries().iter().skip(self.log_persisted) {
            let mut key = b"log:".to_vec();
            key.extend_from_slice(&e.seq.to_le_bytes());
            batch.insert(key, self.seal(crate::store::log_entry_to_bytes(e))?);
        }
        // RFC-0006 (E1, §413): los consumos nuevos, por contador; un conjunto
        // que solo crece no tiene huerfanas que barrer, y `load` caza igual
        // un `cons:` de mas: la raiz no cuadra.
        for (pos, c) in self.consumos_orden.iter().skip(self.cons_persisted) {
            let mut key = b"cons:".to_vec();
            key.extend_from_slice(&pos.to_le_bytes());
            batch.insert(key, self.seal(digest_to_bytes(c).to_vec())?);
        }
        // §802: las identidades retiradas y su raiz, en el MISMO lote (al final del fichero).
        self.retiradas_al_lote(&mut batch)?;

        // --- Cuentas congeladas ---
        //
        // Se escriben TODAS en cada lote. Las congelaciones son raras, asi
        // que el conjunto es pequeno; hacerlo incremental complicaria el
        // codigo sin ganancia medible.
        //
        // Sin esto, reiniciar el nodo levantaria todas las congelaciones:
        // el contador sobreviviria pero el arbol no, y una cuenta bajo
        // investigacion volveria a poder gastar.
        //
        // Y se BORRAN las que ya no lo estan (§390): descongelar quita la
        // hoja del arbol, pero una clave `froz:` huerfana en disco volvia a
        // congelar la cuenta al reiniciar. El lote es un mapa por clave: el
        // `insert` de abajo gana sobre este `remove` en las que siguen.
        for item in db.scan_prefix(b"froz:") {
            let (k, _) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            batch.remove(k);
        }
        for (index, leaf) in self.frozen.occupied() {
            let mut key = b"froz:".to_vec();
            key.extend_from_slice(&index.to_le_bytes());
            batch.insert(key, self.seal(digest_to_bytes(&leaf).to_vec())?);
        }

        // --- Pendiente creado o consumido, si la operacion lo produce ---
        if let Some((position, p)) = pending_leaf {
            let mut key = b"pend:".to_vec();
            key.extend_from_slice(&position.to_le_bytes());
            batch.insert(key, self.seal(digest_to_bytes(&p).to_vec())?);

            // ⚠️ **El importe va con la hoja, en el mismo lote.**
            //
            // Antes se escribian todos los importes actuales en un bucle
            // aparte, y **eso nunca borraba los de los pendientes ya
            // cobrados**: tras reiniciar, `total_pending()` los seguia
            // contando.
            //
            // El mecanismo correcto es el que ya usaba la hoja: escribir el
            // valor VACIO al consumir. Al cargar, un importe cero se omite,
            // igual que `set_leaf` con el digest cero elimina la hoja.
            let importe = self.pending_amounts.get(&position).copied().unwrap_or(0);
            let mut key = b"pamt:".to_vec();
            key.extend_from_slice(&position.to_le_bytes());
            batch.insert(key, self.seal(importe.to_le_bytes().to_vec())?);

            // El META viaja con la hoja, en el MISMO lote (lección §169):
            // 16 bytes (emisor||nacimiento) si existe; vacío si la posición
            // quedó sin meta (vaciada, o legado sin caducidad).
            let mut key = b"pmeta:".to_vec();
            key.extend_from_slice(&position.to_le_bytes());
            let mv = match self.pending_meta.get(&position) {
                Some((s, b)) => {
                    let mut v = s.to_le_bytes().to_vec();
                    v.extend_from_slice(&b.to_le_bytes());
                    v
                }
                None => Vec::new(),
            };
            batch.insert(key, self.seal(mv)?);
        }

        // §702 (RFC-0001): la cabecera del KDF va EN CLARO en cada lote, con lo
        // que se sella bajo ella: el primer lote de un libro nuevo la escribe
        // junto a sus datos. Sin clave, o con la de la version 1, no hay.
        if let Some(c) = self.key.as_ref().and_then(|k| k.kdf()).and_then(|k| k.cabecera()) {
            batch.insert(CLAVE_KDF, c.to_vec());
        }
        db.apply_batch(batch)
            .map_err(|e| StoreError::Io(e.to_string()))?;
        db.flush().map_err(|e| StoreError::Io(e.to_string()))?;
        // Despues del flush: si algo fallara arriba, la marca no se
        // mueve y la proxima pasada reescribe. Fallo hacia el lado
        // seguro.
        self.log_persisted = self.log.len();
        self.cons_persisted = self.consumos_orden.len();
        self.retir_persisted = self.retiradas.len();
        Ok(())
    }
}

/// §702 (RFC-0001): la cabecera del KDF del libro cifrado, EN CLARO: la
/// version, el coste y la sal no son secretos, y sin ellos no se deriva la
/// clave. Un libro sin ella y con datos cifrados es de la version 1.
pub(crate) const CLAVE_KDF: &[u8] = b"meta:kdf";

/// §702: las claves del libro que van EN CLARO aunque el libro este cifrado.
/// `meta:geometry_v7` y `meta:migrated` son marcadores de presencia (las
/// raices arbitran), y `meta:kdf` es la cabecera. Todo lo demas va sellado, y
/// la migracion lo comprueba: lo que no abre con la clave vieja la para.
pub(crate) const EN_CLARO: [&[u8]; 3] = [b"meta:geometry_v7", b"meta:migrated", CLAVE_KDF];

/// §702 (RFC-0001): el cifrado en reposo con su version. Va al final del
/// fichero para que las citas `persistence.rs:linea` de los documentos que
/// las llevan no se muevan.
impl SovereignLayer {
    /// **Abre el `sled` y fija la clave al KDF del libro.** Devuelve la clave
    /// fijada y si el libro es de la version 1.
    ///
    /// - Con cabecera (`meta:kdf`), la clave se fija a ella.
    /// - Sin cabecera y con datos, el libro es de la version 1 (SHA-256 sin
    ///   sal): la clave se fija a ella, y `cargar_y_migrar` lo migra despues
    ///   de verificarlo.
    /// - Sin cabecera y sin datos, el libro nace en la version 2, con una sal
    ///   nueva; el primer `commit` escribe la cabecera con sus datos.
    /// - Con cabecera y sin clave, no se abre en claro: `ParameterMismatch`
    ///   de «cifrado en reposo», en vez de un error de formato del primer
    ///   valor sellado que se intentara leer.
    /// - ⚠️ §812 (RFC-0001, E4): antes de abrir, `resolver_restos` resuelve lo que
    ///   dejara un recifrado muerto a mitad junto al libro (`<libro>.recifrando`,
    ///   `<libro>.retirado`): hacia atras antes del compromiso, hacia delante
    ///   despues. Sin hermanos, no hace nada mas que mirar si estan.
    #[allow(clippy::type_complexity)]
    fn abrir_con_su_kdf(
        path: &str,
        key: Option<&crate::crypto::LedgerKey>,
    ) -> Result<(sled::Db, Option<crate::crypto::LedgerKey>, bool), LayerError> {
        use crate::crypto::Kdf;
        resolver_restos(path)?;
        let db = sled::open(path).map_err(|e| StoreError::Io(e.to_string()))?;
        let guardado = db
            .get(CLAVE_KDF)
            .map_err(|e| StoreError::Io(e.to_string()))?
            .map(|v| Kdf::de_cabecera(&v))
            .transpose()?;
        let con_datos = db
            .contains_key(b"meta:custodians")
            .map_err(|e| StoreError::Io(e.to_string()))?;
        let (fijada, de_v1) = match (key, guardado) {
            (None, None) => (None, false),
            (None, Some(_)) => {
                return Err(StoreError::ParameterMismatch { what: "cifrado en reposo" }.into())
            }
            (Some(k), Some(kdf)) => (Some(k.fijar(&kdf)?), false),
            (Some(k), None) if con_datos => (Some(k.fijar(&Kdf::Sha256V1)?), true),
            (Some(k), None) => (Some(k.fijar(&Kdf::nuevo())?), false),
        };
        Ok((db, fijada, de_v1))
    }

    /// `load`, y si el libro es de la version 1, la migracion: verificar
    /// antes de mutar. §812: recibe la ruta, porque la migracion escribe al
    /// lado del libro (`migrar_a_kdf_v2`).
    fn cargar_y_migrar(
        &mut self,
        path: &str,
        de_v1: bool,
        frase: Option<&crate::crypto::LedgerKey>,
    ) -> Result<(), LayerError> {
        self.load()?;
        match (de_v1, frase) {
            (true, Some(f)) => self.migrar_a_kdf_v2(path, f),
            _ => Ok(()),
        }
    }

    /// **La migracion del cifrado en reposo, de la version 1 a la 2.** La
    /// llama `cargar_y_migrar` con un libro de la version 1 que `load` acaba
    /// de verificar entero con la clave vieja.
    ///
    /// Abre CADA valor sellado con la clave v1 y lo vuelve a sellar con una
    /// v2 de sal nueva, y escribe la cabecera. Lo que no se sella son las tres
    /// claves de `EN_CLARO`. Un valor que no abre con la clave v1 para la
    /// migracion sin tocar el libro y la dice: descartar sin verificar no se
    /// hace.
    ///
    /// ⚠️ §812: hasta aqui lo hacia «todo en UN lote: sled lo aplica entero o no
    /// lo aplica, asi que el libro queda en una version o en la otra, nunca a
    /// medias». Era cierto de lo que se LEE, no de lo que queda en el DISCO:
    /// sled es un log, y el lote dejaba en los ficheros del directorio los
    /// valores sellados con la clave v1, sin sal, que se fuerzan a la velocidad
    /// de un hash (`SECURITY.md` §3.14). Desde el §812 la migracion es el
    /// recifrado de la E4 con la misma frase (`recifrar_en_hermano`, al
    /// final): un directorio nuevo con lo vivo, un cambio de nombre y el viejo
    /// borrado entero. Entero o nada sigue valiendo.
    fn migrar_a_kdf_v2(
        &mut self,
        path: &str,
        frase: &crate::crypto::LedgerKey,
    ) -> Result<(), LayerError> {
        if self.db().is_none() {
            return Ok(());
        }
        match &self.key {
            Some(k) if k.kdf() == Some(crate::crypto::Kdf::Sha256V1) => {}
            _ => return Ok(()),
        }
        let nueva = frase.fijar(&crate::crypto::Kdf::nuevo())?;
        self.recifrar_en_hermano(path, nueva, QUEDA_MIGRADO)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// §802: LAS IDENTIDADES RETIRADAS, EN REPOSO. Va al final del fichero, como el §702,
// para que las citas `persistence.rs:linea` de los documentos no se muevan.
//
// `retir:{k u64 LE}` = la identidad que retiro la recuperacion de ordinal k, y
// `root:retir` = la cadena sobre ellas (`cabeza_de_retiradas`). Las dos van SELLADAS
// (ninguna entra en `EN_CLARO`), y la migracion del KDF las vuelve a sellar sin codigo
// propio. La raiz no es estado: el sellado no lleva AAD (`crypto.rs`), asi que un valor
// sellado de 32 B se puede cortar de otra clave y pegar en `retir:k` sin la frase; la
// cuenta seguiria cuadrando, y la identidad retirada volveria a abrir. La raiz lo caza.
// ─────────────────────────────────────────────────────────────────────────────
impl SovereignLayer {
    /// `commit`: la raiz de las retiradas y las que aun no estan en disco, por ordinal.
    /// Una lista que solo crece no tiene huerfanas que barrer, como `cons:`.
    fn retiradas_al_lote(&self, batch: &mut sled::Batch) -> Result<(), LayerError> {
        batch.insert(b"root:retir".as_ref(), self.seal(digest_to_bytes(&self.retir_cabeza).to_vec())?);
        for (k, id) in self.retiradas.iter().enumerate().skip(self.retir_persisted) {
            let mut key = b"retir:".to_vec();
            key.extend_from_slice(&(k as u64).to_le_bytes());
            batch.insert(key, self.seal(digest_to_bytes(id).to_vec())?);
        }
        Ok(())
    }

    /// `load`: lee `retir:` y `root:retir`, los exige contra el registro ya verificado
    /// -`n_recovery` lo contaron los contadores del §393- y reindexa.
    ///
    /// - Sin `root:retir`, con ninguna recuperacion y ninguna `retir:`: un libro anterior
    ///   al §802, que no tiene ninguna retirada. Abre, y su primer `commit` escribe la raiz.
    /// - Sin `root:retir` en cualquier otro caso: no abre. Un libro con recuperaciones y
    ///   sin retiradas se ve igual que uno al que le han borrado las dos claves en reposo,
    ///   y abrirlo vacio devolveria a la vida las identidades retiradas.
    /// - Con `root:retir`: los ordinales exactamente `0..len`, `len == n_recovery` y la
    ///   cadena que cuadra con la raiz. Si no, no abre.
    ///
    /// No rechaza dos cuentas con la misma identidad que entraran antes del §802: el indice
    /// las colapsa.
    fn cargar_retiradas(&mut self, db: &sled::Db, n_recovery: u64) -> Result<(), LayerError> {
        const WHAT: &str = "identidades retiradas";
        let integridad = || -> LayerError { StoreError::IntegrityFailure { what: WHAT }.into() };
        let key = self.key.clone();
        let abrir = |v: sled::IVec| -> Result<Vec<u8>, LayerError> {
            match &key {
                None => Ok(v.to_vec()),
                Some(k) => Ok(k.open(&v)?),
            }
        };
        // sled ordena las claves como bytes, y un u64 en little-endian no sigue ese orden.
        let mut leidas: Vec<(u64, Digest)> = Vec::new();
        for item in db.scan_prefix(b"retir:") {
            let (k, v) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            let k = u64::from_le_bytes(
                k[6..]
                    .try_into()
                    .map_err(|_| StoreError::Malformed("ordinal de identidad retirada".into()))?,
            );
            leidas.push((k, digest_from_bytes(&abrir(v)?)?));
        }
        leidas.sort_by_key(|(k, _)| *k);
        let raiz = match db.get(b"root:retir").map_err(|e| StoreError::Io(e.to_string()))? {
            Some(v) => Some(digest_from_bytes(&abrir(v)?)?),
            None => None,
        };
        let ids: Vec<Digest> = leidas.iter().map(|(_, id)| *id).collect();
        match raiz {
            None if n_recovery != 0 || !ids.is_empty() => return Err(integridad()),
            None => {}
            Some(r) => {
                if leidas.iter().enumerate().any(|(i, (k, _))| *k != i as u64)
                    || crate::accounts::cabeza_de_retiradas(&ids) != r
                {
                    return Err(integridad());
                }
            }
        }
        self.adoptar_retiradas(ids, n_recovery, WHAT)?;
        self.retir_persisted = self.retiradas.len();
        Ok(())
    }
}

/// §802: `cargar_retiradas`, por dentro.
#[cfg(test)]
mod tests_retiradas {
    use super::*;

    /// **`cargar_retiradas` ordena por ORDINAL, no por los bytes de la clave.** sled devuelve
    /// `retir:{k u64 LE}` en orden de bytes, y en little-endian ese orden se separa del numerico
    /// desde k = 256: `retir:0`, `retir:256`, `retir:1`... Ningun test llega a 257 recuperaciones
    /// -cada una son tres pruebas-, asi que aqui se plantan 300 retiradas con su raiz en un `sled`
    /// en claro y se cargan: la lista sale en su orden, la cadena cuadra y las 300 quedan en el
    /// indice.
    #[test]
    fn cargar_retiradas_ordena_por_ordinal_y_no_por_los_bytes() {
        let mut p = std::env::temp_dir();
        p.push(format!("zkssl_retir_orden_{}", std::process::id()));
        let path = p.to_str().expect("ruta").to_string();
        let _ = std::fs::remove_dir_all(&path);
        let e = BaseElement::new;
        let ids: Vec<Digest> = (0..300u64).map(|k| [e(k + 1), e(7), e(0), e(0)]).collect();
        {
            let db = crate::tests_support::sled_open_retry(&path);
            for (k, id) in ids.iter().enumerate() {
                let mut key = b"retir:".to_vec();
                key.extend_from_slice(&(k as u64).to_le_bytes());
                db.insert(key, digest_to_bytes(id).to_vec()).expect("plantar");
            }
            let raiz = crate::accounts::cabeza_de_retiradas(&ids);
            db.insert(b"root:retir", digest_to_bytes(&raiz).to_vec()).expect("la raiz");
            let segunda = db.scan_prefix(b"retir:").nth(1).expect("hay dos").expect("leer").0;
            assert_eq!(&segunda[6..], &256u64.to_le_bytes(), "el orden de bytes no es el numerico");

            let mut l = crate::tests_support::new_layer();
            l.cargar_retiradas(&db, 300).expect("CRITICO: 300 retiradas en orden no cargan");
            assert_eq!(l.retiradas, ids, "la lista, en el orden de los ordinales");
            assert_eq!((l.retir_cabeza, l.retir_persisted), (raiz, 300));
            for k in [0, 1, 255, 256, 299] {
                assert!(matches!(l.identidad_libre(&ids[k]), Err(LayerError::IdentityUnavailable)), "{k}");
            }
        }
        let _ = std::fs::remove_dir_all(&path);
    }
}

// =====================================================================
// ⚠️ §812 (RFC-0001, E4): EL RECIFRADO DEL LIBRO, y la migracion de la E2 por
// el mismo camino. Va al final, como el bloque del §702. Delante se reescriben
// en sitio la apertura y la migracion, y lo de detras sube cuatro lineas.
//
// Por que un directorio HERMANO y no un lote en sitio: sled es un log. Un
// `Batch` que sobrescribe cada valor cambia lo que se lee entero o nada, pero
// los valores viejos siguen en los ficheros del directorio hasta que sled
// reaprovecha su segmento, y eso puede no pasar nunca. Medido en el §812 sobre
// sled 0.34.7, con un libro de dos cuentas: tras la migracion v1 -> v2 del §702,
// sus 25 valores sellados con la clave v1 -SHA-256 sin sal- seguian en los
// ficheros tras 200 lotes mas y tres reaperturas (`SECURITY.md` §3.14); desde
// el §802 el mismo libro lleva 26, con `root:retir`, y en sitio quedan 26. Quien se
// lleve el disco de un libro que migro el codigo del §702 prueba frases contra
// ese resto a la velocidad de un hash, y la frase buena abre tambien la v2.
//
// El recifrado copia lo VIVO, ya sellado con la clave nueva, a un directorio
// nuevo junto al libro, lo verifica leyendolo del disco, y cambia los nombres;
// el directorio viejo se borra entero. Los nombres, reservados:
//
//   P = el libro    R = `<P>.recifrando`    V = `<P>.retirado`
//
// El compromiso es `rename(P, V)`: antes, una muerte deja P con la frase
// vieja y R a medias, y la apertura siguiente borra R; despues, deja V y R
// -o P y V-, y la apertura siguiente termina hacia delante, con la nueva.
//
// Lo que esto NO alcanza, y se dice en `SECURITY.md` §3.14 y en el RFC: los
// bloques que el sistema de ficheros libera al borrar (borrar no sobrescribe),
// las copias que guarden un SSD, un sistema copy-on-write, una instantanea o el
// journal, y cualquier copia del directorio hecha antes.
// =====================================================================

use std::path::{Path, PathBuf};

/// §812: el directorio hermano en el que se escribe el libro recifrado.
const SUFIJO_RECIFRANDO: &str = ".recifrando";
/// §812: el nombre que toma el libro viejo en el compromiso, hasta borrarse.
const SUFIJO_RETIRADO: &str = ".retirado";
/// §812: cuantas veces se mira, cada 50 ms, si el cerrojo de sled esta libre.
/// Las mismas que `open_retry` en los tests: sled tarda a veces en soltarlo
/// tras cerrar (la entrada 18 del `BACKLOG.md`).
const ESPERAS_CERROJO: u32 = 10;
/// §812 · Lo que dice un error pasado el compromiso: con que frase queda el
/// libro. La pieza la usan dos, y no lo deja igual: el recifrado, con la frase
/// nueva; la migracion de la version 1, con la misma con que se abrio, que es
/// la vieja cuando la migracion la hace `recifrar` antes de recifrar.
const QUEDA_RECIFRADO: &str = "recifrado pasado el compromiso: el libro queda con la frase NUEVA";
const QUEDA_MIGRADO: &str = "migracion a la version 2 pasado el compromiso: el libro queda con la MISMA \
     frase con que se abrio -la vieja, si migraba para `recifrar`-, ya en la version 2";

impl SovereignLayer {
    /// **Recifra el libro ENTERO con una frase nueva y una sal nueva**, entero
    /// o nada (RFC-0001, E4, §812). Devuelve la capa abierta con la clave nueva.
    ///
    /// Los parametros son los de [`SovereignLayer::open_encrypted`], porque
    /// el libro se abre con la frase vieja y `load` lo verifica entero antes
    /// de tocar nada. `nueva` tiene que llevar su frase
    /// ([`crate::crypto::LedgerKey::from_passphrase`]): se fija a una sal
    /// nueva con el coste de la version 2. Una clave ya fijada no puede
    /// derivar otra sal y da `Malformed` antes de escribir nada. La misma frase
    /// se admite: cambia la sal y deja el directorio limpio, pero no protege las
    /// copias que ya existan. Un libro de la version 1 se migra primero, con la
    /// vieja, y despues se recifra.
    ///
    /// No cambia la raiz ni el contenido: el libro recifrado tiene las mismas
    /// claves y, abiertos sus valores, los mismos bytes -tambien las
    /// identidades retiradas del §802 y su raiz-; y lo abre tambien el codigo
    /// del §702, con la frase nueva: el de antes del §802, como a cualquier
    /// libro desde el §802, sin hacer valer las identidades retiradas
    /// (SECURITY.md §3.13).
    ///
    /// **Lo que exige.** El libro existe, es un directorio de verdad -no un
    /// enlace- y solo lleva lo que escribe sled; nadie lo tiene abierto; y se
    /// puede escribir en su directorio padre, con sitio para una copia. Los
    /// nombres `<libro>.recifrando` y `<libro>.retirado` son del recifrado: un
    /// directorio de sled con uno de ellos junto al libro lo BORRA la apertura
    /// siguiente (`resolver_restos`), asi que no sirven para copias.
    ///
    /// **El contrato.** `Ok`: el libro esta entero con la frase nueva, y ni el
    /// ni sus hermanos guardan nada sellado con la vieja. `Err`: el libro esta
    /// entero con UNA de las dos frases: la vieja si fallo antes del
    /// compromiso, y la nueva si el mensaje dice que el RECIFRADO lo paso; en
    /// ese caso, hasta la apertura siguiente, el `<libro>.retirado` -si sigue
    /// en el disco- guarda lo sellado con la vieja, y esa apertura lo borra. Con
    /// un libro de la version 1 hay un compromiso antes, el de su migracion: si
    /// falla pasado ese, el mensaje lo dice, y el libro queda con la vieja, ya
    /// en la version 2. Si el proceso muere a mitad, la apertura siguiente lo
    /// resuelve, y el libro abre con una de las dos: antes del compromiso
    /// vuelve atras -el libro sigue entero con la frase VIEJA y el recifrado
    /// hay que repetirlo-, y despues termina hacia delante, con la nueva. Quien
    /// llama prueba las dos.
    ///
    /// ⚠️ Lo que borra no lo sobrescribe: los bloques libres del sistema de
    /// ficheros, un SSD, un sistema copy-on-write, sus instantaneas y las
    /// copias que ya existan pueden seguir guardando lo sellado con la frase
    /// vieja (`SECURITY.md` §3.14).
    #[allow(clippy::too_many_arguments)]
    pub fn recifrar(
        path: &str,
        custodian_set_root: Digest,
        governance_set_root: Digest,
        regulatory_limit: u64,
        max_supply: u64,
        max_accounts: u64,
        vieja: crate::crypto::LedgerKey,
        nueva: crate::crypto::LedgerKey,
    ) -> Result<Self, LayerError> {
        // a. La clave nueva, primero: sin frase no hay sal nueva que fijar, y
        //    el Argon2id se paga antes de escribir nada.
        let nueva = nueva.fijar(&crate::crypto::Kdf::nuevo())?;
        // b. Lo que dejara una muerte anterior.
        resolver_restos(path)?;
        // c. El libro existe, es de sled, y nadie lo tiene abierto. Sin esto,
        //    `open_encrypted` crearia uno vacio donde no hay libro.
        let (p, _, _) = hermanos(path).ok_or_else(|| sin_nombre(path))?;
        es_de_sled(&p, true)?;
        cerrojo_libre(&p)?;
        // d. Abrirlo con la vieja lo verifica entero (`load`); un libro de la
        //    version 1 se migra antes, por la misma pieza y con la misma frase.
        let mut layer = Self::open_encrypted(
            path,
            custodian_set_root,
            governance_set_root,
            regulatory_limit,
            max_supply,
            max_accounts,
            Some(vieja),
        )?;
        // e. El recifrado.
        layer.recifrar_en_hermano(path, nueva, QUEDA_RECIFRADO)?;
        Ok(layer)
    }

    /// **La pieza de la E4**: copia el libro abierto en `self` a
    /// `<path>.recifrando` con la clave `nueva`, lo verifica, cambia los
    /// nombres y borra el viejo. La usan `recifrar` y `migrar_a_kdf_v2`, y cada
    /// uno dice en `queda` con que frase deja el libro un error pasado el
    /// compromiso (`QUEDA_RECIFRADO`, `QUEDA_MIGRADO`).
    ///
    /// Precondicion: `self.db` esta abierto sobre `path` y `load` lo verifico;
    /// `self.key` es la clave vieja, fijada; `nueva` esta fijada a una sal
    /// nueva de la version 2. El libro sigue abierto -con su cerrojo- hasta el
    /// paso 7, para que nadie lo abra en medio: tras el paso 5 sled escribe por
    /// su descriptor, en el inodo de `<path>.retirado`, y despues de abrir solo
    /// usa la ruta para los blobs. El libro no tiene: sled 0.34.7 pasa a blob un
    /// fragmento de mas de 128 KiB (`segment_size / 4 - 20`), parte las hojas
    /// por encima de 16 entradas, y un valor del libro mide unos cientos de
    /// bytes (la entrada del registro, 137 o 169 en claro). Y todo se lee
    /// antes del paso 5.
    fn recifrar_en_hermano(
        &mut self,
        path: &str,
        nueva: crate::crypto::LedgerKey,
        queda: &'static str,
    ) -> Result<(), LayerError> {
        let (p, r, v) = hermanos(path).ok_or_else(|| sin_nombre(path))?;
        let padre = padre_de(&p).to_path_buf();

        // --- 0. Comprobaciones: si alguna falla, no se ha escrito nada ---
        let db = self
            .db()
            .cloned()
            .ok_or_else(|| StoreError::Malformed("recifrado: la capa no tiene libro en disco".into()))?;
        let vieja = self
            .key
            .clone()
            .ok_or_else(|| StoreError::Malformed("recifrado: el libro no esta cifrado".into()))?;
        es_de_sled(&p, true)?;
        if db.tree_names().len() != 1 {
            return Err(StoreError::Malformed(format!(
                "{}: lleva arboles de sled que la capa no escribe; no se toca",
                p.display()
            ))
            .into());
        }
        let cabecera = nueva
            .kdf()
            .and_then(|k| k.cabecera())
            .ok_or_else(|| StoreError::Malformed("la clave nueva no es de la version 2".into()))?;
        if existe(&v)? {
            return Err(ya_existe(&v));
        }

        // --- 1. R nace vacio y con 0700; si ya existe, no es nuestro ---
        let mut nace = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            nace.mode(0o700);
        }
        nace.create(&r).map_err(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => ya_existe(&r),
            _ => error_io(&r, e),
        })?;

        // --- 2 a 4: copiar, verificar leyendo del disco, sincronizar ---
        if let Err(e) = preparar_hermano(&db, &p, &r, &padre, &vieja, &nueva, &cabecera) {
            deshacer_hermano(&r, &padre);
            return Err(e);
        }

        // --- 5. EL COMPROMISO ---
        // Si el `rename` mismo falla -un punto de montaje da EBUSY; Windows no
        // renombra un directorio con ficheros abiertos-, P sigue intacto.
        if let Err(e) = std::fs::rename(&p, &v) {
            deshacer_hermano(&r, &padre);
            return Err(error_io(&p, e));
        }
        // Pasado el compromiso no se deshace nada: un error lo dice, con la
        // frase con que queda el libro -la nueva al recifrar, la misma al
        // migrar-, y la apertura siguiente termina lo que falte.
        let pasado = |e: LayerError| -> LayerError {
            StoreError::Io(format!(
                "{queda}; la apertura siguiente lo termina, y hasta entonces el `.retirado`, si \
                 sigue en el disco, guarda lo sellado con la clave de antes ({e})"
            ))
            .into()
        };
        sincronizar(&padre).map_err(pasado)?;
        punto(5).map_err(pasado)?;

        // --- 6. R pasa a ser el libro ---
        std::fs::rename(&r, &p).map_err(|e| pasado(error_io(&r, e)))?;
        sincronizar(&padre).map_err(pasado)?;
        punto(6).map_err(pasado)?;

        // --- 7. Se suelta el libro viejo y se borra entero ---
        // Se le da a sled su tiempo para cerrar, sin exigirlo: en Unix un
        // fichero abierto se borra igual, y lo que sled escriba al cerrar va a
        // un inodo sin nombre. Donde no se puede, falla, y la apertura
        // siguiente lo borra.
        drop(db);
        self.db = None;
        let _ = cerrojo_libre(&v);
        quitar(&v).map_err(pasado)?;
        sincronizar(&padre).map_err(pasado)?;
        punto(7).map_err(pasado)?;

        // --- 8. Se reabre el libro, ya recifrado ---
        // Nada se recarga: lo que hay en memoria es lo que el paso 3 comprobo
        // que dice la copia.
        cerrojo_libre(&p).map_err(pasado)?;
        let db = sled::open(&p).map_err(|e| pasado(StoreError::Io(e.to_string()).into()))?;
        self.db = Some(db);
        self.key = Some(nueva);
        Ok(())
    }
}

/// §812 · Pasos 2 a 4 del recifrado, todos antes del compromiso: si uno falla,
/// quien llama borra R y el libro sigue como estaba.
fn preparar_hermano(
    db: &sled::Db,
    p: &Path,
    r: &Path,
    padre: &Path,
    vieja: &crate::crypto::LedgerKey,
    nueva: &crate::crypto::LedgerKey,
    cabecera: &[u8],
) -> Result<(), LayerError> {
    punto(1)?;
    // R sigue en 0700 hasta el final del paso 4: sled crea sus ficheros con la
    // umask -un `db` en 0644-, y mientras no tomen los modos de P solo entra
    // en R su dueno. Los permisos de P los toma alli, ficheros y directorio.

    // --- 2. Copiar lo vivo, sellado con la nueva, en UN lote ---
    {
        let copia = abrir_sled(r)?;
        let mut lote = sled::Batch::default();
        for item in db.iter() {
            let (k, v) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            if k.as_ref() == CLAVE_KDF {
                continue;
            }
            if EN_CLARO.contains(&k.as_ref()) {
                lote.insert(k, v);
                continue;
            }
            // Un valor que no abre con la vieja para el recifrado: descartar
            // sin verificar no se hace.
            let claro = zeroize::Zeroizing::new(vieja.open(&v).map_err(|_| {
                StoreError::Malformed(format!(
                    "recifrado del libro: el valor de {} no abre con la clave vieja",
                    String::from_utf8_lossy(&k)
                ))
            })?);
            lote.insert(k, nueva.seal(&claro)?);
        }
        lote.insert(CLAVE_KDF, cabecera.to_vec());
        copia.apply_batch(lote).map_err(|e| StoreError::Io(e.to_string()))?;
        copia.flush().map_err(|e| StoreError::Io(e.to_string()))?;
    }
    punto(2)?;

    // --- 3. Verificar la copia leyendola DEL DISCO, en claro ---
    // Las mismas claves que el libro, mas la cabecera nueva; lo que va en
    // claro, igual; y cada valor sellado, abierto con la nueva, dice lo mismo
    // que el del libro abierto con la vieja. Las raices son valores del libro
    // (`root:*`), asi que van dentro; y `pamt:` y `pmeta:`, que ninguna raiz
    // cubre, tambien. Y las identidades retiradas del §802, `retir:{k}` y su
    // raiz `root:retir`: van selladas, asi que el paso 2 las copia con la nueva
    // y aqui se comparan como las demas; sin ellas, el libro recifrado no
    // abriria, o abriria devolviendo a la vida lo que una recuperacion retiro.
    cerrojo_libre(r)?;
    {
        let copia = abrir_sled(r)?;
        let mal = |por: String| -> LayerError {
            StoreError::Malformed(format!(
                "recifrado del libro: la copia de {} no es el libro: {por}",
                r.display()
            ))
            .into()
        };
        let (mut en_copia, mut con_cabecera) = (0usize, false);
        for item in copia.iter() {
            let (k, en_r) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            en_copia += 1;
            let nombre = String::from_utf8_lossy(&k).into_owned();
            if k.as_ref() == CLAVE_KDF {
                if en_r.as_ref() != cabecera {
                    return Err(mal("la cabecera no es la de la clave nueva".into()));
                }
                con_cabecera = true;
                continue;
            }
            let en_p = db
                .get(&k)
                .map_err(|e| StoreError::Io(e.to_string()))?
                .ok_or_else(|| mal(format!("lleva {nombre}, que el libro no lleva")))?;
            if EN_CLARO.contains(&k.as_ref()) {
                if en_r != en_p {
                    return Err(mal(format!("{nombre} no es el del libro")));
                }
                continue;
            }
            let antes = zeroize::Zeroizing::new(vieja.open(&en_p)?);
            let ahora = zeroize::Zeroizing::new(
                nueva.open(&en_r).map_err(|_| mal(format!("{nombre} no abre con la clave nueva")))?,
            );
            if *antes != *ahora {
                return Err(mal(format!("{nombre} no dice lo mismo que en el libro")));
            }
        }
        let mut en_libro = 0usize;
        for k in db.iter().keys() {
            let k = k.map_err(|e| StoreError::Io(e.to_string()))?;
            if k.as_ref() != CLAVE_KDF {
                en_libro += 1;
            }
        }
        if !con_cabecera || en_copia != en_libro + 1 {
            return Err(mal(format!(
                "{en_copia} claves en la copia, y {en_libro} en el libro mas la cabecera"
            )));
        }
    }
    cerrojo_libre(r)?;
    punto(3)?;

    // --- 4. Los permisos de P, y sincronizar: R entero, y su nombre en el padre ---
    // Primero los modos de los ficheros y despues el del directorio, que hasta
    // aqui era 0700: sin esto, sled dejaria la copia con la umask, y un 0700 o
    // un 0600 del operador pasarian a 0755 o a 0644.
    copiar_modos(p, r)?;
    let permisos = std::fs::metadata(p).map_err(|e| error_io(p, e))?.permissions();
    std::fs::set_permissions(r, permisos).map_err(|e| error_io(r, e))?;
    // sled no sincroniza `conf` ni el nombre de `snap.*`; sin esto, un corte
    // de corriente tras el compromiso podria dejar un R incompleto.
    sincronizar_arbol(r)?;
    sincronizar(padre)?;
    punto(4)?;
    Ok(())
}

/// §812 · Antes del compromiso, un fallo borra R -que solo lleva lo sellado con
/// la nueva- y deja el libro como estaba. Sin errores: es limpieza, y si no se
/// pudiera, la apertura siguiente lo borra.
fn deshacer_hermano(r: &Path, padre: &Path) {
    let _ = cerrojo_libre(r);
    let _ = std::fs::remove_dir_all(r);
    let _ = sincronizar(padre);
}

/// §812 · **Lo que deja una muerte a mitad de un recifrado, resuelto antes de
/// abrir.** La llama `abrir_con_su_kdf`, asi que vale para toda apertura:
/// `open`, `open_encrypted` y `recifrar`; tambien la del libro que el nodo
/// abre en claro (`SovereignLayer::open`), con el mismo borrado y la misma
/// negativa. Sin hermanos no hace nada mas que mirar si estan.
///
/// | en disco | murio                        | la apertura                   | abre con |
/// |----------|------------------------------|-------------------------------|----------|
/// | P + R    | pasos 1 a 4, o antes del 5   | borra R (solo lleva lo nuevo) | la vieja |
/// | V + R    | entre el 5 y el 6            | R pasa a P, y V se borra      | la nueva |
/// | P + V    | tras el rename del 6, o en 7 | borra V (todo lo de la vieja) | la nueva |
/// | otro     | -                            | rehusa, sin tocar nada        | -        |
///
/// «Otro» es P + R + V, R solo, V solo, o V + R con un R sin `db`: el
/// recifrado no los deja, pero los deja, por ejemplo, el codigo de antes del
/// §812, que con V + R y sin P crea un libro vacio en P. Y rehusa tambien un
/// hermano que no es un directorio de verdad o que lleva algo que no es de
/// sled. Nunca crea un libro vacio donde habia un recifrado a medias.
///
/// Antes de actuar mira el cerrojo de sled de cada uno: si alguno sigue
/// tomado, el libro esta abierto o recifrandose en otro proceso, y la
/// respuesta es Io, sin tocar nada. Y si no puede borrar V, falla y lo dice:
/// no se abre con lo viejo al lado sin que nadie lo vea.
///
/// ⚠️ Los dos nombres son RESERVADOS: no hay marca que distinga un hermano
/// que dejo el recifrado de un directorio de sled que alguien llamo asi. Con
/// el libro al lado, la apertura BORRA un `<libro>.recifrando` o un
/// `<libro>.retirado` de sled que nadie tenga abierto -una copia de seguridad
/// con ese nombre, tambien-. Lo que no es de sled no lo toca.
fn resolver_restos(path: &str) -> Result<(), LayerError> {
    let Some((p, r, v)) = hermanos(path) else {
        return Ok(());
    };
    let (hay_r, hay_v) = (existe(&r)?, existe(&v)?);
    if !hay_r && !hay_v {
        return Ok(());
    }
    let hay_p = existe(&p)?;
    let padre = padre_de(&p).to_path_buf();
    if hay_r {
        es_de_sled(&r, false)?;
    }
    if hay_v {
        es_de_sled(&v, false)?;
    }
    let rehusa = |que: &str| -> LayerError {
        StoreError::Malformed(format!(
            "{}: {que}; el recifrado no deja este estado, y no se toca nada: lo resuelve una persona",
            p.display()
        ))
        .into()
    };
    match (hay_p, hay_r, hay_v) {
        (true, true, false) | (true, false, true) => {}
        (false, true, true) => es_de_sled(&r, true)?,
        (true, true, true) => return Err(rehusa("estan el libro, su `.recifrando` y su `.retirado`")),
        (false, true, false) => return Err(rehusa("esta su `.recifrando` y no el libro")),
        // (false, false, true); sin ningun hermano ya se volvio arriba.
        _ => return Err(rehusa("esta su `.retirado` y no el libro")),
    }
    for d in [&p, &r, &v] {
        if existe(d)? {
            cerrojo_libre(d)?;
        }
    }
    if hay_r && hay_p {
        // Murio antes del compromiso: R solo lleva lo sellado con la nueva.
        quitar(&r)?;
        sincronizar(&padre)?;
    }
    if hay_r && !hay_p {
        // Murio entre los dos nombres: R esta entero, verificado y
        // sincronizado, porque los pasos 3 y 4 van antes del 5.
        std::fs::rename(&r, &p).map_err(|e| error_io(&r, e))?;
        sincronizar(&padre)?;
    }
    if hay_v {
        // V es TODO lo sellado con la vieja.
        quitar(&v)?;
        sincronizar(&padre)?;
    }
    Ok(())
}

/// §812 · El libro y sus dos hermanos, en su mismo directorio. Se forman con
/// `parent` y `file_name`, como el temporal del keystore, nunca pegando texto
/// a la ruta: con una ruta acabada en `/`, pegar caeria DENTRO del libro. Una
/// ruta sin nombre (`.`, `..`) no tiene hermanos: la apertura no resuelve
/// nada, y recifrar o migrar rehusan.
fn hermanos(path: &str) -> Option<(PathBuf, PathBuf, PathBuf)> {
    let ruta = Path::new(path);
    let nombre = ruta.file_name()?;
    let padre = ruta.parent().unwrap_or(Path::new(""));
    let con = |sufijo: &str| {
        let mut n = nombre.to_os_string();
        n.push(sufijo);
        padre.join(n)
    };
    Some((padre.join(nombre), con(SUFIJO_RECIFRANDO), con(SUFIJO_RETIRADO)))
}

/// El directorio que guarda el nombre de `p`: una ruta sin directorio es del
/// actual.
fn padre_de(p: &Path) -> &Path {
    match p.parent() {
        Some(d) if !d.as_os_str().is_empty() => d,
        _ => Path::new("."),
    }
}

/// Si hay algo con ese nombre, sin seguir enlaces.
fn existe(p: &Path) -> Result<bool, LayerError> {
    match std::fs::symlink_metadata(p) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(error_io(p, e)),
    }
}

/// §812 · Un directorio de verdad -no un enlace- cuyas entradas son todas de
/// sled 0.34.7: `conf`, `db`, `blobs/` con ficheros, y `snap.*`. Lo que no lo
/// es no se copia, no se renombra y no se borra.
fn es_de_sled(dir: &Path, exige_db: bool) -> Result<(), LayerError> {
    let rehusa = |por: String| -> LayerError {
        StoreError::Malformed(format!("{}: {por}; no se toca", dir.display())).into()
    };
    let md = std::fs::symlink_metadata(dir).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => rehusa("no existe".into()),
        _ => error_io(dir, e),
    })?;
    if !md.file_type().is_dir() {
        return Err(rehusa("no es un directorio de verdad (un enlace no se sigue)".into()));
    }
    let mut hay_db = false;
    for e in std::fs::read_dir(dir).map_err(|e| error_io(dir, e))? {
        let e = e.map_err(|e| error_io(dir, e))?;
        let tipo = e.file_type().map_err(|x| error_io(&e.path(), x))?;
        let nombre = e.file_name();
        let nombre = nombre.to_string_lossy();
        let de_sled = match &*nombre {
            "db" if tipo.is_file() => {
                hay_db = true;
                true
            }
            "conf" if tipo.is_file() => true,
            "blobs" if tipo.is_dir() => solo_ficheros(&e.path())?,
            n => n.starts_with("snap.") && tipo.is_file(),
        };
        if !de_sled {
            return Err(rehusa(format!("lleva «{nombre}», que no es de sled")));
        }
    }
    if exige_db && !hay_db {
        return Err(rehusa("no lleva `db`: no es un libro".into()));
    }
    Ok(())
}

fn solo_ficheros(dir: &Path) -> Result<bool, LayerError> {
    for e in std::fs::read_dir(dir).map_err(|e| error_io(dir, e))? {
        let e = e.map_err(|e| error_io(dir, e))?;
        if !e.file_type().map_err(|x| error_io(&e.path(), x))?.is_file() {
            return Ok(false);
        }
    }
    Ok(true)
}

/// §812 · Espera, acotado, a que nadie tenga el cerrojo de sled sobre
/// `<dir>/db`. Es el mismo `flock` que toma sled 0.34.7 por fs2, y el de `std`
/// es compatible con el. Si sigue tomado, el libro esta abierto: Io, que es lo
/// que reintentan los `*_retry` de los tests. Sin `db`, no hay cerrojo.
fn cerrojo_libre(dir: &Path) -> Result<(), LayerError> {
    let ruta = dir.join("db");
    let f = match std::fs::File::open(&ruta) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(error_io(&ruta, e)),
    };
    for intento in 1..=ESPERAS_CERROJO {
        match f.try_lock() {
            Ok(()) => {
                let _ = f.unlock();
                return Ok(());
            }
            Err(std::fs::TryLockError::WouldBlock) if intento < ESPERAS_CERROJO => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(std::fs::TryLockError::WouldBlock) => break,
            Err(std::fs::TryLockError::Error(e)) => return Err(error_io(&ruta, e)),
        }
    }
    Err(StoreError::Io(format!(
        "{}: el libro esta abierto o recifrandose en otro proceso",
        dir.display()
    ))
    .into())
}

/// §812 · Borra un directorio entero; si ya no esta, esta hecho.
fn quitar(dir: &Path) -> Result<(), LayerError> {
    match std::fs::remove_dir_all(dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(StoreError::Io(format!(
            "{}: no se pudo borrar ({e}); el libro no se abre con eso al lado",
            dir.display()
        ))
        .into()),
    }
}

/// §812 · Sincroniza un directorio: es donde viven los nombres. Como el
/// keystore del §702, solo en Unix; fuera, la durabilidad de un nombre es la
/// que de el sistema.
fn sincronizar(dir: &Path) -> Result<(), LayerError> {
    #[cfg(unix)]
    {
        std::fs::File::open(dir)
            .and_then(|d| d.sync_all())
            .map_err(|e| error_io(dir, e))?;
    }
    #[cfg(not(unix))]
    let _ = dir;
    Ok(())
}

/// §812 · Sincroniza cada fichero de R y de `R/blobs`, y los dos directorios.
fn sincronizar_arbol(dir: &Path) -> Result<(), LayerError> {
    for e in std::fs::read_dir(dir).map_err(|e| error_io(dir, e))? {
        let e = e.map_err(|e| error_io(dir, e))?;
        let ruta = e.path();
        let tipo = e.file_type().map_err(|x| error_io(&ruta, x))?;
        if tipo.is_dir() {
            sincronizar_arbol(&ruta)?;
        } else {
            std::fs::OpenOptions::new()
                .read(true)
                .write(cfg!(windows))
                .open(&ruta)
                .and_then(|f| f.sync_all())
                .map_err(|x| error_io(&ruta, x))?;
        }
    }
    sincronizar(dir)
}

/// §812 · Los modos de los ficheros del libro pasan a los de la copia: cada uno
/// toma el de su homonimo en P, y el que no lo tiene -los `snap.*` llevan su
/// numero-, el de `P/db`. Sled los crea con la umask, y un 0600 del operador
/// pasaria a 0644. Fuera de Unix no hay modos que copiar.
fn copiar_modos(p: &Path, r: &Path) -> Result<(), LayerError> {
    #[cfg(unix)]
    {
        let del_db = std::fs::metadata(p.join("db")).map_err(|e| error_io(p, e))?.permissions();
        for e in std::fs::read_dir(r).map_err(|e| error_io(r, e))? {
            let e = e.map_err(|e| error_io(r, e))?;
            let ruta = e.path();
            let homonimo = p.join(e.file_name());
            let permisos = match std::fs::symlink_metadata(&homonimo) {
                Ok(m) => m.permissions(),
                Err(_) if e.file_type().map(|t| t.is_file()).unwrap_or(false) => del_db.clone(),
                Err(_) => continue,
            };
            std::fs::set_permissions(&ruta, permisos).map_err(|x| error_io(&ruta, x))?;
        }
    }
    #[cfg(not(unix))]
    let _ = (p, r);
    Ok(())
}

fn abrir_sled(dir: &Path) -> Result<sled::Db, LayerError> {
    sled::open(dir).map_err(|e| StoreError::Io(format!("{}: {e}", dir.display())).into())
}

fn error_io(p: &Path, e: std::io::Error) -> LayerError {
    StoreError::Io(format!("{}: {e}", p.display())).into()
}

fn ya_existe(p: &Path) -> LayerError {
    StoreError::Malformed(format!(
        "{}: ya existe, y el recifrado no lo crea ni lo borra; no se toca",
        p.display()
    ))
    .into()
}

fn sin_nombre(path: &str) -> LayerError {
    StoreError::Malformed(format!(
        "{path}: la ruta no nombra el directorio del libro; el recifrado escribe a su lado"
    ))
    .into()
}

// §812 · Lo que inyectan los tests del recifrado al acabar el paso k: en ellos,
// `punto(k)` entra en panico si el hilo lo pidio en `MORIR_EN`, o devuelve un
// error si lo pidio en `FALLAR_EN`, que sigue el camino de un error de verdad
// en ese paso -antes del compromiso, se borra R; despues, el mensaje dice con
// que frase queda el libro-. Fuera de los tests no hace nada y no se compila
// nada.
//
// El panico es una aproximacion de la muerte: desenrolla la pila y corre los
// `Drop`, y el de sled hace `flush` y `sync_all` de lo que tenga abierto. Una
// muerte de verdad -un SIGKILL, un `abort`- no. En cada punto, lo unico
// abierto es el libro viejo, que el recifrado solo lee, y R se escribio con
// `flush` en el paso 2, asi que a ese `Drop` no le queda nada pendiente que
// cambie lo que se mide. Fuera de la suite se midio tambien con muertes de
// verdad, sin `Drop` (§812): un `abort` al acabar cada paso y SIGKILL a
// destiempo, en el recifrado, en la migracion y en la apertura que resuelve,
// con el mismo resultado que aqui.
#[cfg(test)]
thread_local! {
    pub(crate) static MORIR_EN: std::cell::Cell<Option<u8>> = const { std::cell::Cell::new(None) };
    pub(crate) static FALLAR_EN: std::cell::Cell<Option<u8>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
fn punto(k: u8) -> Result<(), LayerError> {
    if MORIR_EN.with(|m| m.get()) == Some(k) {
        panic!("§812: muerte inyectada al acabar el paso {k} del recifrado");
    }
    if FALLAR_EN.with(|m| m.get()) == Some(k) {
        return Err(StoreError::Io(format!("§812: fallo inyectado al acabar el paso {k}")).into());
    }
    Ok(())
}

#[cfg(not(test))]
#[inline(always)]
fn punto(_: u8) -> Result<(), LayerError> {
    Ok(())
}
