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
    /// versión 1 se verifica y se migra a la 2 al abrirlo (`abrir_con_su_kdf`, al final). ⚠️ §816: con el cerrojo del libro, `<libro>.cerrojo`, tomado hasta que vuelve.
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
        let (db, fijada, de_v1, cerrojo) = Self::abrir_con_su_kdf(path, key.as_ref())?;

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
            layer.cargar_y_migrar(path, de_v1, key.as_ref(), &cerrojo)?;
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
            // raíces almacenadas no cuadrarían (§128). ⚠️ §860: un sled sin marcador es anterior al §387 y no abre: le falta `root:pending`, abajo; esta rama ya no lleva a abrir ningun libro (razonado, no medido contra un store real).
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
    ///   despues. ⚠️ §816: decia «Sin hermanos, no hace nada mas que mirar si
    ///   estan». Antes de mirar toma el cerrojo del libro (`Cerrojo`, al final) y
    ///   lo devuelve: quien llama lo tiene hasta volver, con el libro abierto y, si
    ///   era de la version 1, migrado. Sin el, un recifrado que llegaba a su
    ///   compromiso entre mirar y `sled::open` -que crea el libro si no esta- dejaba
    ///   aqui un libro VACIO (`SECURITY.md` §3.14). Una ruta sin nombre no se abre;
    ///   crea antes el directorio del libro si falta, como `sled::open`, y antes de
    ///   `sled::open` mira que el cerrojo siga en su sitio (`Cerrojo::sigue`).
    #[allow(clippy::type_complexity)]
    fn abrir_con_su_kdf(
        path: &str,
        key: Option<&crate::crypto::LedgerKey>,
    ) -> Result<(sled::Db, Option<crate::crypto::LedgerKey>, bool, Cerrojo), LayerError> {
        use crate::crypto::Kdf;
        let cerrojo = Cerrojo::tomar(path, Quien::Apertura)?;
        resolver_restos(path, &cerrojo)?;
        cerrojo.sigue("abrir:antes_de_sled")?;
        let db = sled::open(path).map_err(|e| StoreError::Io(e.to_string()))?;
        pausa("abrir:tras_sled");
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
        Ok((db, fijada, de_v1, cerrojo))
    }

    /// `load`, y si el libro es de la version 1, la migracion: verificar
    /// antes de mutar. §812: recibe la ruta, porque la migracion escribe al
    /// lado del libro (`migrar_a_kdf_v2`). ⚠️ §816: y el cerrojo del libro, que
    /// la apertura tiene tomado y la migracion pide.
    fn cargar_y_migrar(
        &mut self,
        path: &str,
        de_v1: bool,
        frase: Option<&crate::crypto::LedgerKey>,
        cerrojo: &Cerrojo,
    ) -> Result<(), LayerError> {
        self.load()?;
        match (de_v1, frase) {
            (true, Some(f)) => self.migrar_a_kdf_v2(path, f, cerrojo),
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
    /// borrado entero. Entero o nada sigue valiendo. ⚠️ §828: y desde el §828 solo la hace el dueno del libro: la apertura con clave de otro usuario -root, en un mantenimiento-, o la de un libro cuyo `db`, `conf` o `blobs/` son de otro usuario que su directorio, rehusa al nacer el hermano, en su paso 1, y falla entera sin copiar, sellar ni renombrar nada (`es_del_dueno`, al final); pero despues de que sled abra el libro, y esa apertura hace en el lo que hace cualquier apertura de quien abre -con el libro sin asentar, un `snap.*` suyo y con su umask-, y antes crea el cerrojo del libro, suyo, si no estaba: aqui no se compara antes de abrir, como en `recifrar` (`quien_recifra_es_el_dueno`, al final), porque que el libro es v1 lo dice lo que guarda sled, y rehusar antes toda apertura con clave de otro usuario rehusaria tambien la de un v2, que no migra (deducido; SECURITY.md §3.17); hasta el §828, el libro migrado quedaba del usuario y del grupo de quien lo abria.
    fn migrar_a_kdf_v2(
        &mut self,
        path: &str,
        frase: &crate::crypto::LedgerKey,
        cerrojo: &Cerrojo,
    ) -> Result<(), LayerError> {
        if self.db().is_none() {
            return Ok(());
        }
        match &self.key {
            Some(k) if k.kdf() == Some(crate::crypto::Kdf::Sha256V1) => {}
            _ => return Ok(()),
        }
        let nueva = frase.fijar(&crate::crypto::Kdf::nuevo())?;
        self.recifrar_en_hermano(path, nueva, QUEDA_MIGRADO, cerrojo)
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
//   C = `<P>.cerrojo` (⚠️ §816)
//
// El compromiso es `rename(P, V)`: antes, una muerte deja P con la frase
// vieja y R a medias, y la apertura siguiente borra R; despues, deja V y R
// -o P y V-, y la apertura siguiente termina hacia delante, con la nueva.
//
// ⚠️ §816: EL CERROJO DEL LIBRO. Hasta el §816 la apertura miraba el cerrojo de sled
// de cada hermano, lo tomaba y lo SOLTABA en el acto, y despues actuaba y abria;
// sin hermanos no miraba ninguno, y `sled::open` crea el libro si no esta. Nada
// retenia un cerrojo entre mirar y actuar: un recifrado que llegaba a su
// compromiso en medio dejaba a la apertura crear un libro VACIO en P, o borrar el
// `.retirado` NUEVO del recifrado, y el libro se perdia entero (medido con dos
// hilos y pausas inyectadas; `SECURITY.md` §3.14). El cerrojo de sled no podia
// excluir: va sobre `P/db`, y P se renombra. C es un fichero que la capa no
// renombra ni borra, con `flock` exclusivo: lo tiene toda apertura desde antes de
// resolver los hermanos hasta que vuelve con el libro abierto -y, si era v1,
// migrado-, y `recifrar` desde antes de mirar los hermanos hasta el final; y se
// mira que siga en su sitio antes de `sled::open` y antes del compromiso
// (`Cerrojo`, al final). Excluye a las aperturas y recifrados de la capa desde el
// §816 por el mismo nombre del libro; lo que no alcanza, en `SECURITY.md` §3.14.
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
/// ⚠️ §816: el cerrojo del libro, un fichero que la capa no renombra ni borra (`Cerrojo`).
const SUFIJO_CERROJO: &str = ".cerrojo";
/// §812: cuantas veces se mira, cada 50 ms, si el cerrojo de sled esta libre.
/// Las mismas que `open_retry` en los tests: sled tarda a veces en soltarlo
/// tras cerrar (la entrada 18 del `BACKLOG.md`). ⚠️ §816: y las mismas para tomar
/// el cerrojo del libro: una espera, no dos (`tomar_con_espera`).
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
    /// (SECURITY.md §3.13). ⚠️ §828: y desde el §828 solo lo recifra el dueno del directorio del libro y de su `db`, su `conf` y su `blobs/`: otro usuario rehusa antes de tomar el cerrojo del libro y de abrirlo, sin crear, abrir ni escribir nada del libro ni de su cerrojo (`quien_recifra_es_el_dueno`, al final; que compare ahi es de la sesion, y reversible), y, de respaldo -si esa comparacion no pudo crear su sonda-, en el paso 1, sin copiar nada (`es_del_dueno`), aunque despues de abrir el libro con sled en el paso d, que hace en el lo que hace cualquier apertura de quien abre; y la copia toma el grupo y los modos de cada fichero y del directorio (`dar_lo_del_libro`), o, si su dueno no es del grupo del libro, se queda en el grupo con que nacio -el principal de quien recifra, o el del padre con setgid- y sin bits de grupo ni de otros; salvo el `snap.*` que sled escriba al reabrirlo en el paso 8, de quien recifra y con su umask, como en cualquier apertura; hasta el §828, el libro recifrado quedaba del usuario y del grupo de quien recifraba (SECURITY.md §3.17).
    ///
    /// **Lo que exige.** El libro existe, es un directorio de verdad -no un
    /// enlace- y solo lleva lo que escribe sled; nadie lo tiene abierto; y se
    /// puede escribir en su directorio padre, con sitio para una copia (⚠️ §828: y, en Unix, quien recifra es el dueno del libro y de su `db`, su `conf` y su `blobs/`). Los
    /// nombres `<libro>.recifrando` y `<libro>.retirado` son del recifrado: un
    /// directorio de sled con uno de ellos junto al libro lo BORRA la apertura
    /// siguiente (`resolver_restos`), asi que no sirven para copias. ⚠️ §816: y el
    /// cerrojo del libro, `<libro>.cerrojo`, que `recifrar` tiene desde antes de
    /// mirar los hermanos hasta el final -tras el Argon2id de la frase nueva, antes
    /// de tocar el disco-: lo toma con la espera de siempre -diez veces cada 50 ms-
    /// y, si lo tiene otra apertura u otro recifrado, da `Io` con su nombre sin
    /// tocar nada. Si no esta y hay libro, lo crea, vacio; nunca lo borra ni lo
    /// renombra, y no crea el directorio del libro.
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
    /// llama prueba las dos. ⚠️ §816: hasta el §816, todo esto con nadie mas a la vez:
    /// una apertura que miraba los hermanos antes del compromiso y abria despues
    /// creaba un libro vacio en el sitio del libro, el paso 6 fallaba, y el libro
    /// se quedaba sin abrir o se perdia (`SECURITY.md` §3.14). Lo cierra el cerrojo
    /// del libro.
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
        // ⚠️ §816: el cerrojo del libro, de aqui al final. La apertura con la vieja
        //    de d, y su migracion, lo encuentran tomado por este hilo.
        let cerrojo = quien_recifra_es_el_dueno(path).and_then(|()| Cerrojo::tomar(path, Quien::Recifrado))?; // ⚠️ §828: antes de tomarlo y de abrir el libro, que quien recifra sea su dueno y el libro de un solo dueno (`quien_recifra_es_el_dueno`, al final): si no, rehusa sin crear, abrir ni escribir nada del libro ni de su cerrojo; si no lo puede saber -la sonda no se crea-, sigue, y rehusa de respaldo el paso 1, despues de tomar el cerrojo -y de crearlo, si no estaba- y de abrir el libro; y si la sonda no se puede borrar, rehusa nombrandola
        // b. Lo que dejara una muerte anterior.
        resolver_restos(path, &cerrojo)?;
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
        layer.recifrar_en_hermano(path, nueva, QUEDA_RECIFRADO, &cerrojo)?;
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
    /// nueva de la version 2; y quien llama tiene el cerrojo del libro
    /// (`cerrojo`), de antes del paso 0 a despues del 8, y mira antes del 5 que
    /// siga en su sitio (⚠️ §816). El libro sigue
    /// abierto -con el cerrojo de sled- hasta el paso 7 (⚠️ §816: decia «para que
    /// nadie lo abra en medio», y no lo impedia: ese cerrojo va con el inodo, que
    /// tras el paso 5 es el de `<path>.retirado`, y en P no queda ninguno; quien
    /// excluye es el del libro): tras el paso 5 sled escribe por
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
        cerrojo: &Cerrojo,
    ) -> Result<(), LayerError> {
        pausa("recifrar:0");
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

        // --- 1. R nace vacio y con 0700; si ya existe, no es nuestro --- ⚠️ §828: y si no es del dueno del libro y de su `db`, su `conf` y su `blobs/` -nace del usuario del proceso-, o ya no es el directorio vacio que nacio, se borra y se rehusa
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
        // ⚠️ §816: y, antes del compromiso, que el cerrojo del libro siga en su sitio.
        let preparado = es_del_dueno(&p, &r, queda).and_then(|nacio| preparar_hermano(&db, &p, &r, &padre, &vieja, &nueva, &cabecera, nacio)); // ⚠️ §828: antes de copiar nada, que R sea el directorio vacio que nacio y del dueno del libro y de su `db`, su `conf` y su `blobs/` (`es_del_dueno`, al final), con su dispositivo y su inodo, `nacio`, para el paso 4; si no, el `deshacer_hermano` de abajo lo borra
        if let Err(e) = preparado.and_then(|()| cerrojo.sigue("recifrar:4")) {
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
        pausa("recifrar:5");

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
#[allow(clippy::too_many_arguments)] fn preparar_hermano( // ⚠️ §828: con `nacio`
    db: &sled::Db,
    p: &Path,
    r: &Path,
    padre: &Path,
    vieja: &crate::crypto::LedgerKey,
    nueva: &crate::crypto::LedgerKey,
    cabecera: &[u8], nacio: Identidad, // ⚠️ §828: el dispositivo y el inodo de R al nacer (`es_del_dueno`), que el paso 4 exige
) -> Result<(), LayerError> {
    punto(1)?;
    // R sigue en 0700 hasta el final del paso 4: sled crea sus ficheros con la
    // umask -un `db` en 0644-, y mientras no tomen los modos de P solo entra
    // en R su dueno. Los permisos de P los toma alli, ficheros y directorio. ⚠️ §828: y antes que los permisos, el grupo de P, por descriptor (`dar_lo_del_libro`, al final); el dueno de R ya es el del libro y el de su `db`, su `conf` y su `blobs/` (`es_del_dueno`, en el paso 1).

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
    // un 0600 del operador pasarian a 0755 o a 0644. ⚠️ §828: y antes que los modos de cada uno, su grupo, por su descriptor y sin seguir un enlace; el directorio, el ultimo, despues de sincronizar: sin el grupo, la copia era del grupo de quien recifra o migra.
    dar_lo_del_libro(p, r, nacio)?; // ⚠️ §828: era `copiar_modos(p, r)?;`, que solo daba los modos de los ficheros
    // ⚠️ §828: aqui iban `let permisos = std::fs::metadata(p)?.permissions();` y, debajo, el modo del
    // directorio por su ruta, `std::fs::set_permissions(r, permisos)?`: lo da ya `dar_lo_del_libro`, el ultimo.
    // sled no sincroniza `conf` ni el nombre de `snap.*`; sin esto, un corte
    // de corriente tras el compromiso podria dejar un R incompleto.
    // ⚠️ §828: aqui iba `sincronizar_arbol(r)?;`: lo hace ya `dar_lo_del_libro`, con R aun en 0700, antes de dar al directorio su grupo y su modo.
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
/// sled. Nunca crea un libro vacio donde habia un recifrado a medias (⚠️ §816:
/// hasta el §816, solo con nadie mas a la vez; una apertura que miraba antes del
/// compromiso de un recifrado vivo y abria despues lo creaba). ⚠️ §828: y desde el §828 rehusa tambien, sin tocar nada, un `.recifrando` que no es -con su `db`, su `conf` y su `blobs/`- del dueno del libro -el del directorio de P y de su `db`, su `conf` y su `blobs/` o, si falta P, los de V-: no lo borra ni lo pone en el sitio del libro (`recifrando_del_dueno`, al final).
///
/// ⚠️ §816: quien llama tiene el cerrojo del libro (`cerrojo`), y lo tiene hasta
/// despues de `sled::open`: ninguna apertura ni recifrado de la capa desde el §816,
/// por el mismo nombre del libro, recifra ni resuelve mientras tanto, y lo que
/// aqui se mira es lo que hay cuando se actua y cuando se abre (lo que no
/// alcanza, en `Cerrojo` y en `SECURITY.md` §3.14).
///
/// Antes de actuar mira el cerrojo de sled de cada uno: si alguno sigue
/// tomado, el libro esta abierto o recifrandose en otro proceso, y la
/// respuesta es Io, sin tocar nada (⚠️ §816: lo toma y lo suelta en el acto, asi
/// que dice como estaba al mirar; no excluye a nadie mientras se actua: eso es
/// del cerrojo del libro). Y si no puede borrar V, falla y lo dice:
/// no se abre con lo viejo al lado sin que nadie lo vea.
///
/// ⚠️ Los dos nombres son RESERVADOS: no hay marca que distinga un hermano
/// que dejo el recifrado de un directorio de sled que alguien llamo asi. Con
/// el libro al lado, la apertura BORRA un `<libro>.recifrando` o un
/// `<libro>.retirado` de sled que nadie tenga abierto -una copia de seguridad
/// con ese nombre, tambien-. Lo que no es de sled no lo toca. ⚠️ §816: y un
/// tercero, `<libro>.cerrojo`, que la capa crea y no borra nunca. ⚠️ §828: y un `.recifrando` que no es del dueno del libro no lo borra: rehusa.
fn resolver_restos(path: &str, _cerrojo: &Cerrojo) -> Result<(), LayerError> {
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
    recifrando_del_dueno(&p, &r, &v, hay_p, hay_r)?; for d in [&p, &r, &v] { // ⚠️ §828: antes de actuar, que un `.recifrando` sea del dueno del libro (al final)
        if existe(d)? {
            cerrojo_libre(d)?;
        }
    }
    pausa("resolver:tras_cerrojos");
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
/// nada, y recifrar o migrar rehusan. ⚠️ §816: y desde el §816 la apertura tampoco
/// la abre: sin nombre no hay cerrojo del libro (`Cerrojo`).
fn hermanos(path: &str) -> Option<(PathBuf, PathBuf, PathBuf)> {
    Some((con_sufijo(path, "")?, con_sufijo(path, SUFIJO_RECIFRANDO)?, con_sufijo(path, SUFIJO_RETIRADO)?))
}

/// §812 · `<libro><sufijo>`, en el directorio del libro y formado con su nombre:
/// los hermanos y, desde el §816, el cerrojo del libro. Una ruta sin nombre no
/// tiene ninguno.
fn con_sufijo(path: &str, sufijo: &str) -> Option<PathBuf> {
    let ruta = Path::new(path);
    let nombre = ruta.file_name()?;
    let padre = ruta.parent().unwrap_or(Path::new(""));
    let mut n = nombre.to_os_string();
    n.push(sufijo);
    Some(padre.join(n))
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
    if tomar_con_espera(&f, &ruta)? {
        let _ = f.unlock();
        return Ok(());
    }
    Err(StoreError::Io(format!(
        "{}: el libro esta abierto o recifrandose en otro proceso",
        dir.display()
    ))
    .into())
}

/// ⚠️ §816 · El `flock` exclusivo de `f`, con la espera acotada: `ESPERAS_CERROJO`
/// veces, cada 50 ms. `Ok(false)` si en el tope sigue tomado. La comparten
/// `cerrojo_libre` y `Cerrojo::tomar`: una espera, no dos.
fn tomar_con_espera(f: &std::fs::File, ruta: &Path) -> Result<bool, LayerError> {
    for intento in 1..=ESPERAS_CERROJO {
        match f.try_lock() {
            Ok(()) => return Ok(true),
            Err(std::fs::TryLockError::WouldBlock) if intento < ESPERAS_CERROJO => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(std::fs::TryLockError::WouldBlock) => break,
            Err(std::fs::TryLockError::Error(e)) => return Err(error_io(ruta, e)),
        }
    }
    Ok(false)
}

/// ⚠️ §816 · Lo que identifica el fichero del cerrojo: su dispositivo y su inodo, en
/// Unix. Fuera de Unix `std` no los da en estable, y solo se mira que el nombre
/// siga siendo un fichero.
#[cfg(unix)]
type Identidad = (u64, u64);
#[cfg(not(unix))]
type Identidad = ();

#[cfg(unix)]
fn identidad(m: &std::fs::Metadata) -> Identidad {
    use std::os::unix::fs::MetadataExt;
    (m.dev(), m.ino())
}

#[cfg(not(unix))]
fn identidad(_: &std::fs::Metadata) -> Identidad {}

/// ⚠️ §816 · Si `ruta` sigue nombrando, sin seguir enlaces, el fichero `id`.
fn es_el_mismo(ruta: &Path, id: &Identidad) -> Result<bool, LayerError> {
    match std::fs::symlink_metadata(ruta) {
        Ok(m) => Ok(m.file_type().is_file() && identidad(&m) == *id),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(error_io(ruta, e)),
    }
}

/// ⚠️ §816 · Abre `<libro>.cerrojo` sin crear nada a traves de un enlace. Si no esta,
/// lo crea con `create_new` -`O_CREAT|O_EXCL`, que no sigue un enlace en el ultimo
/// nombre-, vacio y, en Unix, con 0600; si esta y es un fichero, lo abre sin crear.
/// Si cambia entre mirar y abrir -lo borran, u otro lo crea antes-, vuelve a mirar,
/// tres veces como mucho. Un enlace que aparezca entre mirar y abrir uno que ya
/// estaba se sigue -no se abre con `O_NOFOLLOW`: `std` no lo da sin `libc`-, sin
/// crear ni escribir nada en su destino, aunque le toma el `flock` un instante; y
/// lo rehusa `Cerrojo::tomar` al comparar el inodo, antes de tocar nada.
///
/// Se abre para escribir, aunque al `flock` le baste leer: con 0600 otro usuario
/// tampoco lo leeria, uno que otro puede leer lo podria tomar y dejar el libro sin
/// abrir, y en NFS el cliente emula el `flock` exclusivo con `fcntl`, que pide el
/// fichero abierto para escribir. Salvo en un sistema de ficheros de solo lectura
/// (`EROFS`) -un libro que es un punto de montaje en un padre de solo lectura-,
/// donde nadie puede escribir ni cambiar el nombre: alli el que ya estaba se abre
/// para leer, y si no esta se dice que hay que crearlo antes, donde se pueda.
fn abrir_el_cerrojo(ruta: &Path) -> Result<std::fs::File, LayerError> {
    for _ in 0..3 {
        let esta = match std::fs::symlink_metadata(ruta) {
            Ok(m) if !m.file_type().is_file() => {
                return Err(StoreError::Malformed(format!(
                    "{}: no es un fichero de verdad (un enlace no se sigue); el cerrojo del \
                     libro no se toma, y no se toca nada",
                    ruta.display()
                ))
                .into())
            }
            Ok(_) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(e) => return Err(error_io(ruta, e)),
        };
        pausa("cerrojo:tras_mirar");
        let mut abrir = std::fs::OpenOptions::new();
        abrir.read(true).write(true).create_new(!esta);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            abrir.mode(0o600);
        }
        match abrir.open(ruta) {
            Ok(f) => return Ok(f),
            Err(e) if esta && e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) if !esta && e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) if esta && e.kind() == std::io::ErrorKind::ReadOnlyFilesystem => {
                match std::fs::File::open(ruta) {
                    Ok(f) => return Ok(f),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(e) => return Err(error_io(ruta, e)),
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::ReadOnlyFilesystem => {
                return Err(StoreError::Io(format!(
                    "{}: {e}; el directorio del libro es de solo lectura y el cerrojo del libro \
                     no esta: hay que crearlo antes, vacio, donde se pueda escribir; no se toca nada",
                    ruta.display()
                ))
                .into())
            }
            Err(e) if esta && e.kind() == std::io::ErrorKind::PermissionDenied => {
                return Err(StoreError::Io(format!(
                    "{}: {e}; quien abre el libro tiene que poder abrir su cerrojo para escribir: \
                     con el libro cerrado por todos, quien pueda se lo da (`chown`) o lo borra, y \
                     la apertura siguiente lo crea; no se toca nada",
                    ruta.display()
                ))
                .into())
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                return Err(StoreError::Io(format!(
                    "{}: {e}; el cerrojo del libro no esta, y quien abre no puede crearlo en el \
                     directorio del libro: quien pueda lo crea, vacio, y se lo da (`chown`); no se \
                     toca nada",
                    ruta.display()
                ))
                .into())
            }
            Err(e) => return Err(error_io(ruta, e)),
        }
    }
    Err(StoreError::Io(format!(
        "{}: cambia mientras se abre; el cerrojo del libro no se toma, y no se toca nada",
        ruta.display()
    ))
    .into())
}

thread_local! {
    /// ⚠️ §816 · Los cerrojos de libro que tiene ESTE hilo, por su ruta y con lo que
    /// identifica su fichero: `recifrar` toma el suyo hasta el final, y la apertura
    /// con la vieja que hace dentro -y su migracion- lo encuentran tomado. Otro hilo
    /// u otro proceso esperan.
    static CERROJOS_DEL_HILO: std::cell::RefCell<Vec<(PathBuf, Identidad)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// ⚠️ §816 · **El cerrojo del libro**: `flock` exclusivo sobre `<libro>.cerrojo`,
/// un fichero que la capa no renombra ni borra. El de sled no sirve para esto: va
/// sobre `P/db`, que el recifrado renombra con P, y quien llega tras el compromiso
/// encuentra otro inodo o ninguno. Lo toman toda apertura (`abrir_con_su_kdf`),
/// desde antes de resolver los hermanos hasta que vuelve con el libro abierto -y,
/// si era de la version 1, migrado-, y `recifrar`, desde antes de mirar los
/// hermanos hasta el final; y `resolver_restos` y `recifrar_en_hermano` lo piden
/// como argumento. Se suelta al soltarse el valor; si el proceso muere, lo suelta
/// el sistema, y un muerto no deja el libro cerrado.
///
/// Reentrante en el hilo que lo tiene: tomarlo otra vez no espera ni suelta
/// nada. Uno que no se toma en el tope -la espera de `cerrojo_libre`- es `Io` con
/// su nombre, sin tocar nada. Si no esta se crea vacio y, en Unix, con 0600: solo
/// su dueno puede tomarlo y dejar el libro sin abrir (`abrir_el_cerrojo`). La
/// apertura crea antes el directorio del libro si no existe, como lo creaba
/// `sled::open`; `recifrar`, no. Un `<libro>.cerrojo` que no es un fichero de
/// verdad -un directorio, un enlace- no se toma: `Malformed`, sin tocar nada; y
/// con el `flock` tomado se comprueba que el nombre sigue siendo ese fichero, y
/// otra vez antes de lo que no se deshace (`sigue`). Una ruta sin nombre -`.`,
/// `..`, o una que acabe en `..`, como `<libro>/blobs/..`- no tiene de que formar
/// el cerrojo, y aun asi puede nombrar el libro: por ella no se abre ni se recifra
/// (`Malformed`, sin tocar nada), porque quien abriera por ahi lo haria sin cerrojo.
///
/// No es `Send` ni `Sync`: la lista que lo hace reentrante es del hilo que lo
/// tomo, y soltarlo en otro hilo dejaria en ella una entrada que ya no es de nadie.
pub(crate) struct Cerrojo {
    /// La ruta del `.cerrojo` y lo que identifica su fichero.
    en: (PathBuf, Identidad),
    /// El fichero con el `flock`; `None` si este hilo ya lo tenia: lo suelta quien
    /// lo tomo.
    f: Option<std::fs::File>,
    /// Lo que le quita `Send` y `Sync`.
    _del_hilo: std::marker::PhantomData<*const ()>,
}

/// ⚠️ §816 · Quien toma el cerrojo del libro, y lo que eso le deja hacer antes de
/// tomarlo.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Quien {
    /// La apertura, que crea el libro si no esta: si falta el directorio del libro,
    /// lo crea antes, como `sled::open`. Es lo unico que hace antes de tomarlo, y con
    /// el directorio ya hecho no escribe nada.
    Apertura,
    /// `recifrar`, que exige un libro que ya existe: no crea directorios, y si no
    /// hay libro ni hermanos rehusa como su paso c, sin crear el `.cerrojo`.
    Recifrado,
}

impl Cerrojo {
    fn tomar(path: &str, quien: Quien) -> Result<Self, LayerError> {
        let Some(ruta) = con_sufijo(path, SUFIJO_CERROJO) else {
            return Err(StoreError::Malformed(format!(
                "{path}: la ruta no nombra el directorio del libro, y sin nombre no hay cerrojo del \
                 libro: por ella no se abre ni se recifra; no se toca nada"
            ))
            .into());
        };
        let del_hilo = CERROJOS_DEL_HILO.with(|t| t.borrow().iter().find(|(r, _)| *r == ruta).map(|(_, id)| *id));
        if let Some(id) = del_hilo {
            return Ok(Cerrojo { en: (ruta, id), f: None, _del_hilo: std::marker::PhantomData });
        }
        match quien {
            Quien::Apertura => {
                if let Some(padre) = ruta.parent().filter(|d| !d.as_os_str().is_empty()) {
                    if !existe(padre)? {
                        std::fs::create_dir_all(padre).map_err(|e| error_io(padre, e))?;
                    }
                }
            }
            Quien::Recifrado if !existe(&ruta)? => {
                let (p, r, v) = hermanos(path).ok_or_else(|| sin_nombre(path))?;
                if !existe(&r)? && !existe(&v)? {
                    es_de_sled(&p, true)?;
                }
            }
            Quien::Recifrado => {}
        }
        let f = abrir_el_cerrojo(&ruta)?;
        if !tomar_con_espera(&f, &ruta)? {
            return Err(StoreError::Io(format!(
                "{}: el cerrojo del libro lo tiene otra apertura o un recifrado; no se toca nada",
                ruta.display()
            ))
            .into());
        }
        let id = identidad(&f.metadata().map_err(|e| error_io(&ruta, e))?);
        if !es_el_mismo(&ruta, &id)? {
            return Err(StoreError::Malformed(format!(
                "{}: cambio mientras se tomaba -por un enlace o por otro fichero-; el cerrojo \
                 del libro no se toma, y no se toca nada",
                ruta.display()
            ))
            .into());
        }
        CERROJOS_DEL_HILO.with(|t| t.borrow_mut().push((ruta.clone(), id)));
        Ok(Cerrojo { en: (ruta, id), f: Some(f), _del_hilo: std::marker::PhantomData })
    }

    /// ⚠️ §816 · Antes de lo que no se deshace -el `sled::open` de la apertura, que crea
    /// el libro si no esta, y el compromiso del recifrado-: que el nombre siga siendo
    /// el fichero que se tomo. Si alguien lo borro o lo cambio mientras se tenia,
    /// otra apertura o un recifrado pueden haber tomado uno nuevo y estar dentro:
    /// `Malformed`, y no se sigue. `donde` es el punto de pausa de los tests, justo
    /// antes de mirar.
    fn sigue(&self, donde: &'static str) -> Result<(), LayerError> {
        pausa(donde);
        let (ruta, id) = &self.en;
        if es_el_mismo(ruta, id)? {
            return Ok(());
        }
        Err(StoreError::Malformed(format!(
            "{}: ya no es el cerrojo que se tomo -alguien lo borro o lo cambio mientras se \
             tenia-; otra apertura o un recifrado pueden estar dentro, y no se sigue: no se toca \
             nada",
            ruta.display()
        ))
        .into())
    }
}

impl Drop for Cerrojo {
    fn drop(&mut self) {
        if let (Some(f), (ruta, _)) = (self.f.take(), &self.en) {
            let _ = CERROJOS_DEL_HILO.try_with(|t| t.borrow_mut().retain(|(r, _)| r != ruta));
            let _ = f.unlock();
        }
    }
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
/// pasaria a 0644. Fuera de Unix no hay modos que copiar. ⚠️ §828: y no daba el grupo, que era el de quien recifraba: desde el §828, en Unix, no la llama nadie -el grupo y los modos los da `dar_lo_del_libro`, al final, por descriptor y sin seguir un enlace-, y fuera de Unix la llama `dar_lo_del_libro`, como antes; se queda para que no se mueva ninguna linea.
#[cfg_attr(unix, allow(dead_code))] fn copiar_modos(p: &Path, r: &Path) -> Result<(), LayerError> {
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

// ⚠️ §816 · Los ganchos de pausa de los tests de la carrera: `pausa(punto)` llama al
// gancho que haya puesto un test, con el nombre del punto, y el gancho para el
// hilo que le toca hasta que el otro llega al suyo. Los puntos: `abrir:antes_de_sled`
// y `abrir:tras_sled` (la apertura), `resolver:tras_cerrojos` (`resolver_restos`),
// `recifrar:0`, `recifrar:4` y `recifrar:5` (la pieza de la E4), y
// `cerrojo:tras_mirar` (`abrir_el_cerrojo`, entre mirar el nombre y abrirlo). Los de
// `Cerrojo::sigue` van justo antes de mirar que el cerrojo siga en su sitio. Fuera
// de los tests no hace nada y no se compila nada, como `punto`. ⚠️ §828: y `recifrar:1`, al empezar `es_del_dueno`, en el paso 1 de la pieza, y `recifrar:grupo`, al empezar `dar_lo_del_libro`, en el paso 4; y `recifrar:sonda`, en `quien_recifra_es_el_dueno`, con la sonda creada y antes de borrarla.
#[cfg(test)]
pub(crate) type Gancho = std::sync::Arc<dyn Fn(&'static str) + Send + Sync>;
#[cfg(test)]
pub(crate) static PAUSA: std::sync::Mutex<Option<Gancho>> = std::sync::Mutex::new(None);

#[cfg(test)]
fn pausa(donde: &'static str) {
    let g = PAUSA.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if let Some(g) = g {
        g(donde)
    }
}

#[cfg(not(test))]
#[inline(always)]
fn pausa(_: &'static str) {}

/// ⚠️ §816 · **El cerrojo del libro no cruza de hilo.** Lo que lo hace reentrante es la
/// lista del hilo que lo tomo (`CERROJOS_DEL_HILO`), que se vacia al soltarlo: si
/// se soltara en otro hilo, la entrada se quedaria en el primero, y ese hilo creeria
/// tener un cerrojo que ya no tiene -la carrera (a) otra vez, como con el mutante
/// M11 de un revisor-. Ningun camino lo mueve de hilo; que no se pueda lo dice el
/// compilador, y esto lo mide: la sonda elige la constante de su `impl` propio si
/// el tipo es `Send`, y la del rasgo si no lo es.
#[cfg(test)]
mod tests_cerrojo {
    #[allow(dead_code)]
    struct Sonda<T>(std::marker::PhantomData<T>);

    trait NoEsSend {
        const ES_SEND: bool = false;
        const ES_SYNC: bool = false;
    }

    impl<T> NoEsSend for Sonda<T> {}

    impl<T: Send> Sonda<T> {
        const ES_SEND: bool = true;
    }

    #[allow(dead_code)]
    struct SondaSync<T>(std::marker::PhantomData<T>);

    impl<T> NoEsSend for SondaSync<T> {}

    impl<T: Sync> SondaSync<T> {
        const ES_SYNC: bool = true;
    }

    #[test]
    fn el_cerrojo_del_libro_no_cruza_de_hilo() {
        assert!(<Sonda<std::fs::File>>::ES_SEND, "la sonda no ve que un fichero es Send");
        assert!(<SondaSync<std::fs::File>>::ES_SYNC, "la sonda no ve que un fichero es Sync");
        assert!(
            !<Sonda<super::Cerrojo>>::ES_SEND,
            "CRITICO: el cerrojo del libro es Send: soltado en otro hilo, su entrada se queda en la lista del primero"
        );
        assert!(!<SondaSync<super::Cerrojo>>::ES_SYNC, "el cerrojo del libro es Sync");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ⚠️ §828: EL LIBRO DE OTRO DUENO NO SE RECIFRA NI SE MIGRA, Y LA COPIA TOMA EL GRUPO DEL LIBRO. Va al
// final del fichero, como el §802 y el §816, para que las citas `persistence.rs:linea` de los
// documentos no se muevan: las tres llamadas van en lineas que ya estaban -la guarda, en la del paso 1
// de `recifrar_en_hermano` que llama a `preparar_hermano`; el grupo, en la de `copiar_modos`, en el
// paso 4; y la de `resolver_restos`, en la de su vuelta por los cerrojos-, y lo que el paso 1 le pasa
// al 4, `nacio`, en dos lineas de la firma de `preparar_hermano`.
// ─────────────────────────────────────────────────────────────────────────────

/// ⚠️ §828 · **Quien recifra o migra tiene que ser el dueno del libro.** Hasta el §828, R -el
/// `.recifrando`, que crea el proceso, y los ficheros que sled crea dentro- nacia del usuario y del
/// grupo de quien recifraba o migraba -o del grupo del padre, si tiene setgid-; el paso 4 le daba solo
/// los modos del libro (`copiar_modos`), y el paso 6 lo ponia en su sitio: recifrado o migrado por
/// root, el libro pasaba a ser de root y su dueno ya no lo abria (`SECURITY.md` §3.17). La migracion
/// corre en toda apertura con clave de un libro de la version 1.
///
/// Lo del autor (21-10-2026, en el chat) es una eleccion: a una pregunta de opcion multiple de la
/// sesion, eligio la opcion «Rehusar si no es el dueño (Recomendado)», cuya descripcion, escrita por la
/// sesion, se cita entera en `SECURITY.md` §3.17. Todo lo demas es de la sesion, y reversible: como lee
/// esa descripcion (abajo, «Nunca se cambia el usuario de nada»), y donde y como. En el recifrado,
/// `recifrar` compara lo mismo antes de tomar el cerrojo del libro y de abrirlo (`quien_recifra_es_el_dueno`,
/// al final), y esto queda de respaldo -con test, si la sonda no se crea-; en la migracion, es lo unico. En Unix, en cuanto nace
/// R (el paso 1), se compara su dueno -el usuario del proceso, que lo acaba de crear- con el del
/// directorio del libro y con el de su `db`, su `conf` y su `blobs/` (`de_quien_es`): los que sled abre
/// para escribir, y los que el paso 6 pondria de quien recifra. Si alguno no es el mismo, R, que esta
/// vacio, se borra (`deshacer_hermano`), y se rehusa con `Malformed`: el error nombra lo que no es
/// suyo, dice de quien es, quien intenta recifrarlo o migrarlo, que asi quedaria suyo, y que se haga
/// como su dueno. Un libro con el directorio de un usuario y su `db`, su `conf` o su `blobs/` de otro
/// -el directorio de un `install -d -o root -g svc -m 2770` con los ficheros del servicio- no lo
/// recifra ni lo migra ninguno de los dos: falla cerrado, y el error lo dice. Los `snap.*` no se
/// comparan: una apertura de root deja uno suyo en el libro de otro sin que el libro deje de ser de su
/// dueno, y rehusar por el seria de mas. Antes de copiar, sellar o renombrar nada: no deja
/// `.recifrando` ni `.retirado`, y no cambia el dueno, el grupo ni los modos de lo que habia. La
/// apertura que tendria que migrar falla entera: no se abre sin migrar. Lo que no evita en la
/// migracion, por ir la guarda aqui: la negativa llega despues de que la capa abra el libro con sled
/// -abrirlo es lo que dispara la migracion-, y esa apertura hace en el libro lo que sled hace en
/// cualquier apertura, de quien abre: con el libro sin asentar y la umask 0077 -o cualquier umask que
/// quite la lectura a otros, como la 0027 para quien no es del grupo 0 (deducido)-, la de root deja un
/// `snap.*` de 0:0 que su dueno no lee, y su dueno no abre el libro hasta que root se lo de o lo quite;
/// y antes, quien abre crea el cerrojo del libro, suyo, si no estaba. Aqui no se puede comparar antes
/// de abrir: que el libro es v1 lo dice lo que guarda sled, y rehusar antes toda apertura con clave de
/// otro usuario rehusaria tambien la de un v2, que no migra, y la opcion elegida no dice eso
/// (deducido; `SECURITY.md` §3.17, «Lo que no hace»). En el recifrado, lo mismo solo si la comparacion
/// de antes no pudo crear su sonda.
///
/// R tiene que ser, ademas, lo que nacio: un directorio, sin seguir un enlace, y vacio; si no, se
/// rehusa igual. Lo que devuelve es su dispositivo y su inodo, que el paso 4 exige antes de darle a
/// nada el grupo y los modos del libro (`dar_lo_del_libro`): quien pudiera escribir en el padre y
/// cambiara R por otro directorio antes de esa comprobacion no haria que root le diera el grupo y los
/// modos del libro a lo que lleve; despues de ella, si a lo del dueno (en `dar_lo_del_libro`, deducido). Nunca se cambia el usuario de nada: lo que el paso 6 pone en el sitio del libro es de quien
/// recifra, que es el dueno del directorio del libro y de su `db`, su `conf` y su `blobs/`. Pero root,
/// cuando es el dueno del libro, si cambia el grupo -`fchown(fd, None, Some(gid))`, en
/// `dar_grupo_y_modo`- de lo que el mismo creo dentro de un R suyo con 0700, y de R; a lo que no creo
/// -un enlace, un enlace duro, una entrada de otro dueno, otro directorio en el sitio de R que no esta
/// vacio al compararlo aqui o que en el paso 4 tiene otro dispositivo u otro inodo- no le da grupo ni
/// modos: rehusa, y el recifrado que rehusa borra R con lo que lleve (`deshacer_hermano`, como desde el
/// §812). Lo que no alcanza esa guarda esta en `SECURITY.md` §3.17, «Lo que no hace»: otro directorio
/// vacio del dueno del libro, puesto en el sitio de R antes de compararlo, recibe la copia y el grupo
/// como el de verdad, y uno puesto entre la comprobacion del paso 4 y su vuelta por las entradas da el
/// grupo y los modos a las suyas que sean del dueno (`dar_lo_del_libro`; deducido, sin test). Esa es la lectura que hace
/// la sesion del «Nunca hace chown como root» de la opcion elegida: que no cambia el usuario de nada, ni
/// el grupo ni los modos de lo que no creo; el grupo de lo suyo, si. Fuera de Unix, como antes: no se
/// compara nada.
fn es_del_dueno(p: &Path, r: &Path, queda: &str) -> Result<Identidad, LayerError> {
    pausa("recifrar:1");
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let de_r = std::fs::symlink_metadata(r).map_err(|e| error_io(r, e))?;
        if !de_r.is_dir() || std::fs::read_dir(r).map_err(|e| error_io(r, e))?.next().is_some() {
            return Err(rehusa_dar(r, "no es el `.recifrando` que nacio en el paso 1, un directorio vacio"));
        }
        let de_quien = de_r.uid();
        let duenos = de_quien_es(p)?;
        comparar_duenos(p, &duenos, de_quien, queda)?;
        Ok(identidad(&de_r))
    }
    #[cfg(not(unix))]
    {
        let _ = (p, r, queda);
        Ok(())
    }
}

/// ⚠️ §828 · De quien es un libro: el usuario de su directorio y el de su `db`, su `conf` y su `blobs/`,
/// los que haya, sin seguir enlaces; el directorio, el primero. Lo miran `es_del_dueno` y
/// `recifrando_del_dueno`.
#[cfg(unix)]
fn de_quien_es(libro: &Path) -> Result<Vec<(PathBuf, u32)>, LayerError> {
    use std::os::unix::fs::MetadataExt;
    let mut out = Vec::new();
    for (i, ruta) in [libro.to_path_buf(), libro.join("db"), libro.join("conf"), libro.join("blobs")].into_iter().enumerate() {
        match std::fs::symlink_metadata(&ruta) {
            Ok(m) => out.push((ruta, m.uid())),
            Err(e) if i > 0 && e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(error_io(&ruta, e)),
        }
    }
    Ok(out)
}

/// ⚠️ §828 · **El paso 4 del recifrado y de la migracion v1->v2: R toma el grupo y los modos del
/// libro**, no solo los modos. Hasta el §828 tomaba solo los de los ficheros (`copiar_modos`) y el del
/// directorio por su ruta, y el grupo era el del proceso -o el del padre, si tiene setgid-: recifrado
/// o migrado por su dueno con otro grupo principal, el grupo del libro perdia la lectura y el principal
/// del dueno la ganaba (`SECURITY.md` §3.17). El dueno ya es el del directorio del libro y el de su
/// `db`, su `conf` y su `blobs/`: lo exige `es_del_dueno`, en el paso 1.
///
/// Cada entrada de R -`conf`, `db`, los `snap.*`, `blobs/` y lo que lleve `blobs/`- toma el grupo y
/// los modos de su homonimo en P, y la que no lo tiene -un `snap.*` o un blob llevan su numero-, los de
/// `P/db`; el directorio R, el ultimo, los de P. La regla es la de la exportacion de la instantanea (la
/// 120), sin el dueno (`dar_grupo_y_modo`): primero el grupo, `fchown` con el dueno en `None`
/// (`snapshot::cambiar_dueno`, con su gancho `FALLAR_DUENO`), que el dueno de un fichero puede poner si
/// es uno de sus grupos, y root, cualquiera; si no se puede, la entrada se queda sin los bits de grupo
/// ni los de otros (`& 0o700`), que con otro grupo alcanzarian a otros; y despues los modos, tambien el
/// setuid, que el `chown` de un fichero quita siempre, y el setgid, que quita si el fichero tiene el bit
/// x del grupo (medido en la sesion, en Linux 6.18: un `fchown` de root deja 2640 en 2640 y lleva 2650
/// a 0650 y 4640 a 0640). La descripcion de la opcion que eligio el autor (21-10-2026) decia «Si es el
/// dueño, conserva el grupo del libro si es uno de los suyos; si no, quita los bits de grupo y de otros,
/// como la 120». Leer «uno de los suyos» como el que el `fchown` deja poner -a root, cualquiera, aunque
/// no sea de sus grupos-, y quitar los bits tambien si el `fchown` falla por otra cosa, es de la
/// sesion, y reversible. Quitarlos y seguir tiene, ademas, una razon de la sesion: rehusar dejaria sin
/// abrir el libro v1 de un dueno que no es del grupo del libro, porque abrirlo es migrarlo (deducido).
///
/// **Sin seguir un enlace.** Antes de tocar nada, R tiene que ser un directorio de verdad, del dueno
/// del libro y con 0700, y el que nacio en el paso 1 -su dispositivo y su inodo, `nacio`, de
/// `es_del_dueno`-, en el que solo entra su dueno. Cada entrada se abre
/// solo si es un fichero o un directorio de verdad (`abrir_sin_seguir`: lo que dice su nombre sin
/// seguir enlaces, antes y despues de abrirla, y lo abierto son lo mismo, dispositivo e inodo; en Linux
/// x86 y arm, ademas con `O_NOFOLLOW`), del dueno de R, y, si es un fichero, con un solo nombre: un
/// enlace duro no pasa. El grupo y los modos se ponen por el descriptor. Un enlace o lo que no es un
/// fichero ni un directorio, en R o en su sitio, un directorio dentro de `blobs/`, una entrada de otro
/// dueno o con otro nombre, o algo que cambie mientras se abre, hace rehusar con `Malformed`, antes del
/// compromiso: el recifrado borra R y el libro sigue como estaba. Esas negativas son de la sesion, y
/// reversibles: fallan cerrado. Lo que no alcanza: `std` no tiene
/// `openat` ni `fchownat`, y cada mirada resuelve la ruta otra vez; un cambio que se deshace entre dos
/// miradas no se ve, y dentro de R solo puede hacerlo su dueno, o root. Y el hueco entre la
/// comprobacion de que `dir` es el R que nacio y la vuelta por sus entradas, que `dar_lo_de_dentro` lee
/// por la ruta (`read_dir(ruta)`) y abre por la ruta: si en ese hueco alguien pone otro directorio en
/// el sitio de R, las entradas de ese otro que sean del dueno de R -el del libro, que es quien recifra-
/// y, si son ficheros, con un solo nombre, toman el grupo y los modos de su homonimo en el libro, o los
/// de `db`, y lo que lleve un directorio suyo de dentro, igual; las demas hacen rehusar. `sigue_siendo`
/// ve el cambio al acabar las entradas y rehusa, pero despues de darlos; al directorio puesto no se le
/// da ni el grupo ni el modo del libro, y el recifrado que rehusa borra despues lo que haya en el sitio
/// de R, si quien lo puso no lo ha quitado antes. Desde el §828 le da tambien el GRUPO del libro; hasta el
/// §828, `copiar_modos` daba, por la ruta y desde el paso 4 hasta el 6, solo los modos a lo que hubiera
/// alli, de quien fuera. Puede hacerlo solo quien escribe en el padre del libro -sin el bit sticky, o su
/// dueno, o root-, y con un directorio que ya este en ese padre, porque llevar un directorio a otro
/// padre pide escribir en el (`rename(2)`). Lo que gana: nada que no fuera ya del dueno del libro -lo
/// de quien hace el cambio, si no es el dueno, hace rehusar-; si, que lo del dueno que hubiera en ese
/// directorio tome el grupo y los modos del libro, y lo lea el grupo del libro si puede entrar en el,
/// porque su modo no se toca. Todo esto, deducido del codigo, sin test (`SECURITY.md` §3.17, «Lo que no
/// hace»).
///
/// Despues de las entradas y antes que el directorio, sincroniza R entero (`sincronizar_arbol`, que el
/// paso 4 hacia despues de darle al directorio su modo): asi abre por su ruta lo que hay en R mientras R
/// sigue en 0700, y no cuando ya lo alcanza el grupo del libro. El directorio, con su grupo y su modo,
/// se sincroniza por su descriptor.
///
/// Fuera de Unix, como antes del §828: `copiar_modos` -que alli no hace nada-, los permisos de P en R y
/// `sincronizar_arbol`.
fn dar_lo_del_libro(p: &Path, r: &Path, nacio: Identidad) -> Result<(), LayerError> {
    pausa("recifrar:grupo");
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let del_libro = de_verdad(p, true)?;
        let del_db = de_verdad(&p.join("db"), false)?;
        let dir = abrir_sin_seguir(r)?;
        let de_r = dir.metadata().map_err(|e| error_io(r, e))?;
        if identidad(&de_r) != nacio || !de_r.is_dir() || de_r.uid() != del_libro.uid() || de_r.mode() & 0o077 != 0 {
            return Err(rehusa_dar(
                r,
                "no es el `.recifrando` que nacio en el paso 1, un directorio del dueno del libro con 0700",
            ));
        }
        dar_lo_de_dentro(&dir, r, p, de_r.uid(), &del_libro, &del_db, true)?;
        sincronizar_arbol(r)?;
        sigue_siendo(r, &dir)?;
        dar_grupo_y_modo(&dir, r, &del_libro)?;
        dir.sync_all().map_err(|e| error_io(r, e))?;
    }
    #[cfg(not(unix))]
    {
        let _ = nacio;
        copiar_modos(p, r)?;
        let permisos = std::fs::metadata(p).map_err(|e| error_io(p, e))?.permissions();
        std::fs::set_permissions(r, permisos).map_err(|e| error_io(r, e))?;
        sincronizar_arbol(r)?;
    }
    Ok(())
}

/// ⚠️ §828 · Las entradas de `dir` -abierto, y en `ruta`-, cada una con el grupo y los modos de su
/// homonimo en `p`, o de `P/db` si es un fichero sin homonimo, o de P si es un directorio sin el; y, si
/// `bajar`, antes que un directorio de dentro -`blobs/`-, lo que lleva. `uid` es el dueno de R.
#[cfg(unix)]
fn dar_lo_de_dentro(
    dir: &std::fs::File,
    ruta: &Path,
    p: &Path,
    uid: u32,
    del_libro: &std::fs::Metadata,
    del_db: &std::fs::Metadata,
    bajar: bool,
) -> Result<(), LayerError> {
    use std::os::unix::fs::MetadataExt;
    for e in std::fs::read_dir(ruta).map_err(|e| error_io(ruta, e))? {
        let e = e.map_err(|e| error_io(ruta, e))?;
        let hijo = e.path();
        let f = abrir_sin_seguir(&hijo)?;
        let m = f.metadata().map_err(|x| error_io(&hijo, x))?;
        if m.uid() != uid {
            return Err(rehusa_dar(&hijo, "no es del dueno del `.recifrando`: no lo creo este recifrado"));
        }
        if m.is_file() && m.nlink() != 1 {
            return Err(rehusa_dar(&hijo, "tiene otro nombre, un enlace duro: no lo creo este recifrado"));
        }
        let homonimo = p.join(e.file_name());
        let de = match std::fs::symlink_metadata(&homonimo) {
            Ok(h) if (m.is_dir() && h.is_dir()) || (m.is_file() && h.is_file()) => h,
            _ if m.is_dir() => del_libro.clone(),
            _ => del_db.clone(),
        };
        if m.is_dir() {
            if !bajar {
                return Err(rehusa_dar(&hijo, "un directorio dentro de `blobs/` no es de sled"));
            }
            dar_lo_de_dentro(&f, &hijo, &homonimo, uid, del_libro, del_db, false)?;
        }
        dar_grupo_y_modo(&f, &hijo, &de)?;
    }
    sigue_siendo(ruta, dir)
}

/// ⚠️ §828 · La regla de la 120 sin el dueno: el grupo de `de` (`fchown` con el dueno en `None`) y, si se
/// pudo, sus modos; si no, sus modos sin los bits de grupo ni los de otros. Por el descriptor.
#[cfg(unix)]
fn dar_grupo_y_modo(f: &std::fs::File, ruta: &Path, de: &std::fs::Metadata) -> Result<(), LayerError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let con_su_grupo = crate::snapshot::cambiar_dueno(f, None, Some(de.gid())).is_ok();
    let modo = if con_su_grupo { de.mode() & 0o7777 } else { de.mode() & 0o700 };
    f.set_permissions(std::fs::Permissions::from_mode(modo)).map_err(|e| error_io(ruta, e))
}

/// ⚠️ §828 · Abre lo que hay en `ruta` solo si es un fichero o un directorio de verdad: lo que dice su
/// nombre sin seguir enlaces, antes y despues de abrirlo, y lo abierto son lo mismo -dispositivo e
/// inodo-. Donde se sabe su valor sin `libc` -Linux en x86 y en arm-, se abre ademas con `O_NOFOLLOW`,
/// y un enlace que aparezca entre mirar y abrir no se abre.
#[cfg(unix)]
fn abrir_sin_seguir(ruta: &Path) -> Result<std::fs::File, LayerError> {
    let antes = std::fs::symlink_metadata(ruta).map_err(|e| error_io(ruta, e))?;
    if !(antes.is_file() || antes.is_dir()) {
        return Err(rehusa_dar(ruta, "no es un fichero ni un directorio de verdad (un enlace no se sigue)"));
    }
    let f = abrir_sin_enlace(ruta).map_err(|e| {
        if O_NOFOLLOW != 0 && e.raw_os_error() == Some(ELOOP) {
            rehusa_dar(ruta, "se cambio por un enlace mientras se abria, y un enlace no se sigue")
        } else {
            error_io(ruta, e)
        }
    })?;
    if identidad(&antes) != identidad(&f.metadata().map_err(|e| error_io(ruta, e))?) {
        return Err(rehusa_dar(ruta, "cambio mientras se abria (un enlace no se sigue)"));
    }
    sigue_siendo(ruta, &f)?;
    Ok(f)
}

/// ⚠️ §828 · `File::open`, con `O_NOFOLLOW` donde se sabe su valor sin `libc`: el de la ABI de Linux en
/// x86 y x86_64 (`0o400000`) y en arm y aarch64 (`0o100000`). En los demas, 0: no se pone, y la guarda
/// es solo la del tipo y el inodo de `abrir_sin_seguir`. `std` no lo da (§816).
#[cfg(unix)]
pub(crate) fn abrir_sin_enlace(ruta: &Path) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new().read(true).custom_flags(O_NOFOLLOW).open(ruta)
}

#[cfg(all(target_os = "linux", any(target_arch = "x86", target_arch = "x86_64")))]
pub(crate) const O_NOFOLLOW: i32 = 0o400000;
#[cfg(all(target_os = "linux", any(target_arch = "arm", target_arch = "aarch64")))]
pub(crate) const O_NOFOLLOW: i32 = 0o100000;
#[cfg(all(
    unix,
    not(all(target_os = "linux", any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64")))
))]
pub(crate) const O_NOFOLLOW: i32 = 0;
/// `ELOOP` en Linux, lo que da `open` con `O_NOFOLLOW` sobre un enlace.
#[cfg(unix)]
pub(crate) const ELOOP: i32 = 40;

/// ⚠️ §828 · Que lo que hay en `ruta`, sin seguir enlaces, siga siendo lo abierto en `f`.
#[cfg(unix)]
fn sigue_siendo(ruta: &Path, f: &std::fs::File) -> Result<(), LayerError> {
    let ahora = std::fs::symlink_metadata(ruta).map_err(|e| error_io(ruta, e))?;
    if identidad(&ahora) != identidad(&f.metadata().map_err(|e| error_io(ruta, e))?) {
        return Err(rehusa_dar(ruta, "ya no es lo que se abrio (un enlace no se sigue)"));
    }
    Ok(())
}

/// ⚠️ §828 · Lo que da el libro: lo que hay en `ruta` sin seguir enlaces, y solo si es un directorio
/// (`dir`) o un fichero de verdad.
#[cfg(unix)]
fn de_verdad(ruta: &Path, dir: bool) -> Result<std::fs::Metadata, LayerError> {
    let m = std::fs::symlink_metadata(ruta).map_err(|e| error_io(ruta, e))?;
    if (dir && m.is_dir()) || (!dir && m.is_file()) {
        return Ok(m);
    }
    Err(rehusa_dar(ruta, "no es lo que da su grupo y sus modos al recifrado (un enlace no se sigue)"))
}

#[cfg(unix)]
fn rehusa_dar(ruta: &Path, por: &str) -> LayerError {
    StoreError::Malformed(format!(
        "{}: {por}; el recifrado no le da el grupo ni los modos del libro, y no sigue: no se toca el libro",
        ruta.display()
    ))
    .into()
}

/// ⚠️ §828 · **La apertura no borra ni termina un `.recifrando` que no es del dueno del libro.** Lo deja
/// un recifrado o una migracion de otro usuario con el codigo de antes del §828, que no lo rehusaba -root
/// que muere a mitad-, o lo pone alguien. Con P al lado -murio antes del compromiso- el libro esta
/// entero con la frase de antes, y la apertura borraba R; con V y sin P -murio entre los pasos 5 y 6-,
/// lo ponia en el sitio del libro: el libro de otro dueno, que su dueno ya no abriria. Desde el §828, si
/// el dueno de R no es el del libro -el del directorio de P y el de su `db`, su `conf` y su `blobs/`, o
/// los de V si falta P (`de_quien_es`)-, o el `db`, el `conf` o el `blobs/` de R no son del dueno de R,
/// rehusa con `Malformed`, sin tocar nada y sin abrir, y el error dice de quien es cada uno y que hacer.
/// No pierde el libro: con P, P esta entero; con V, estan V, con la frase de antes, y R, con la nueva.
/// Rehusar en los dos casos, y no borrar el R ajeno que esta junto a un P entero, es de la sesion,
/// reversible: falla cerrado; y un libro que no es de un solo dueno -el directorio de un usuario y los
/// ficheros de otro- tampoco resuelve. Un R del mismo dueno y de otro grupo -el que dejaba el codigo de
/// antes del §828 recifrado por su dueno con otro grupo principal- se resuelve como antes, con su grupo.
/// Fuera de Unix, como antes.
fn recifrando_del_dueno(p: &Path, r: &Path, v: &Path, hay_p: bool, hay_r: bool) -> Result<(), LayerError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if hay_r {
            let libro = if hay_p { p } else { v };
            let del_libro = de_quien_es(libro)?;
            let de_r = std::fs::symlink_metadata(r).map_err(|e| error_io(r, e))?.uid();
            let ajeno = de_quien_es(r)?.into_iter().chain(del_libro.iter().cloned()).find(|(_, u)| *u != de_r);
            if let Some((ruta, otro)) = ajeno {
                let del_libro = del_libro[0].1;
                let por = if hay_p {
                    format!(
                        "el libro, del usuario {del_libro}, esta entero con la frase de antes: si nadie lo esta \
                         recifrando, quien pueda borra el `.recifrando` y el libro abre"
                    )
                } else {
                    format!(
                        "terminar el recifrado lo pondria en el sitio del libro, que es del usuario {del_libro} \
                         -su `.retirado`-, y su dueno no lo abriria; el libro esta entero en el `.recifrando`, con \
                         la frase nueva, y en el `.retirado`, con la de antes: quien pueda le da el `.recifrando` \
                         al dueno del libro (`chown -R`) y la apertura siguiente lo termina"
                    )
                };
                return Err(StoreError::Malformed(format!(
                    "{}: es del usuario {de_r}, y {} del usuario {otro}: no es del dueno del libro, o no es de un \
                     solo dueno: lo dejo un recifrado o una migracion de otro usuario, o lo puso alguien, y la \
                     apertura no lo borra ni lo termina; {por}. No se toca nada",
                    r.display(),
                    ruta.display()
                ))
                .into());
            }
        }
    }
    #[cfg(not(unix))]
    let _ = (p, r, v, hay_p, hay_r);
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// ⚠️ §828: LA COMPARACION PREVIA DEL RECIFRADO. Va al final, como lo de arriba: la llamada va en la linea
// de `recifrar` que toma el cerrojo del libro.
// ─────────────────────────────────────────────────────────────────────────────

/// ⚠️ §828 · **Antes de tomar el cerrojo del libro y de abrirlo, `recifrar` compara quien recifra con el
/// dueno del libro.** La guarda del paso 1 (`es_del_dueno`) llega despues de que `recifrar` tome el
/// cerrojo del §816 -y lo cree, suyo, si no estaba- y abra el libro con sled para verificarlo con la
/// frase vieja (su paso d): con el libro sin asentar, esa apertura deja en el un `snap.*` de quien abre
/// y con su umask, y con la 0077 -o cualquier umask que quite la lectura a otros, como la 0027 para
/// quien no es del grupo 0 (deducido)- el dueno ya no abre el libro hasta que root se lo de o lo quite
/// (`SECURITY.md` §3.17). Aqui se compara antes, con la misma comparacion y los mismos textos que el
/// paso 1 (`comparar_duenos`): el usuario de quien recifra con el del directorio del libro y el de su
/// `db`, su `conf` y su `blobs/` (`de_quien_es`). Si no es el mismo, o el libro no es de un solo dueno,
/// rehusa con `Malformed` sin crear, abrir ni escribir nada del libro ni de su cerrojo, y sin pagar mas
/// Argon2id que el de la clave nueva (el paso a). Que compare aqui, y ademas en el paso 1, es de la
/// sesion, y reversible.
///
/// **Quien recifra**, sin `libc`: el dueno de una sonda que se crea y se borra en el momento, un fichero
/// vacio `<libro>.sonda.<pid>.<n>` junto al libro -en el directorio donde el paso 1 crearia R-, con
/// `create_new`, que no sigue un enlace en el ultimo nombre ni abre lo que ya lleve ese nombre, y 0600, y
/// mirado por su descriptor. Su dueno es el usuario con que el proceso crea en ese directorio, el que
/// tendria R: el mismo sistema de ficheros, y tambien con un NFS con `root_squash` o un montaje que cambie
/// los duenos, que darian a R el mismo (deducido). No `/proc/self`: da el usuario efectivo y no el de los
/// ficheros (`setfsuid`), solo esta en Linux con `/proc` montado, y no sabe del sistema de ficheros del
/// libro; ni el directorio temporal, que puede ser otro sistema de ficheros, y una sonda que se quedara
/// alli no se veria junto al libro (deducido). Si no se puede saber -la sonda no se puede crear, porque
/// quien recifra no escribe en el padre, porque ya hay algo con ese nombre o porque el nombre no cabe; el
/// libro no esta, no es un directorio de verdad o no es de sled; no se puede mirar de quien es-, no rehusa
/// por eso: sigue, y la guarda del paso 1 queda de respaldo, con lo que deja esa apertura -tomar el cerrojo
/// del libro, y crearlo si no estaba, y abrir el libro con sled- (lo ata
/// `la_sonda_que_no_se_puede_crear_no_hace_rehusar_y_el_paso_1_queda_de_respaldo`, en `tests.rs`, con un
/// nombre que no cabe y con el nombre ya puesto -un enlace simbolico a un fichero de fuera que no esta, uno
/// a uno que esta y uno duro a ese-, que la sonda no sigue). Lo que no es de sled no recibe la sonda: se
/// mira antes (`es_de_sled`, que solo lee), y rehusa despues `Cerrojo::tomar` o el paso c, como antes del
/// §828, sin escribir nada; un borrador anterior del §828 la creaba y la borraba antes de mirarlo, y cambiaba
/// asi la fecha de modificacion del padre (lo vio su revision). La sonda se borra antes de comparar. Si no
/// se puede borrar -un padre con `chattr +a`, donde se crea y no se borra-, rehusa con `Malformed`,
/// nombrandola, sin tomar el cerrojo ni abrir el libro: se queda, vacia, y se borra a mano; hasta un
/// borrador anterior del §828 se quedaba sin que nada lo dijera (lo midio su revision). `Malformed`, como las
/// demas negativas, y no `Io`: quien reintenta un `Io` crearia otra sonda en cada intento (medido en la
/// sesion: con `Io`, `recifrar_e4`, de los tests, que lo reintenta hasta diez veces, dejo diez). Si el
/// proceso muere entre crearla y borrarla, tambien se queda -vacia, del usuario que recifraba-, y nadie la
/// mira ni la borra (deducido, sin test). Crearla y borrarla cambia la fecha de modificacion del padre
/// (medido en la sesion, en Linux 6.18 y ext4: el recifrado de root que rehusa el libro de otro, de la
/// version 2 o de la 1, la cambia, sin dejar nada en el padre; y el que rehusa lo que no es de sled, que no
/// recibe la sonda, no), como la cambia el paso 1 al crear R (deducido).
///
/// La migracion v1->v2 no pasa por aqui: que el libro es v1 solo lo dice lo que guarda sled, y eso se
/// lee al abrirlo; su negativa sigue en el paso 1, despues de esa apertura (`SECURITY.md` §3.17, «Lo
/// que no hace»). Un libro v1 que se recifra no llega a migrarse: se rehusa aqui. Fuera de Unix, como
/// antes: no se compara nada.
fn quien_recifra_es_el_dueno(path: &str) -> Result<(), LayerError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
        static SONDAS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let Some(p) = con_sufijo(path, "") else { return Ok(()) };
        if es_de_sled(&p, true).is_err() {
            return Ok(());
        }
        let Ok(del_libro) = de_quien_es(&p) else { return Ok(()) };
        let n = SONDAS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let Some(sonda) = con_sufijo(path, &format!(".sonda.{}.{n}", std::process::id())) else { return Ok(()) };
        let Ok(f) = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&sonda) else {
            return Ok(());
        };
        let de_quien = f.metadata().map(|m| m.uid());
        drop(f);
        pausa("recifrar:sonda");
        if let Err(e) = std::fs::remove_file(&sonda) {
            return Err(StoreError::Malformed(format!(
                "{}: la sonda con que el recifrado mira quien recifra no se pudo borrar ({e}): se queda, vacia; \
                 borrala a mano y recifra otra vez. No se toma el cerrojo ni se abre el libro",
                sonda.display()
            ))
            .into());
        }
        let Ok(de_quien) = de_quien else { return Ok(()) };
        comparar_duenos(&p, &del_libro, de_quien, QUEDA_RECIFRADO)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

/// ⚠️ §828 · La comparacion de `es_del_dueno` (el paso 1) y de `quien_recifra_es_el_dueno` (antes del
/// cerrojo), con sus dos textos: `duenos`, lo que da `de_quien_es(p)`, con el directorio el primero;
/// `de_quien`, el usuario de quien recifra o migra; y `queda`, si recifra o migra. Rehusa si el libro no
/// es de un solo dueno, o si su dueno no es `de_quien`; y si `duenos` esta vacio, que no se alcanza
/// -`de_quien_es` da siempre el directorio, o un error-, tambien: falla cerrado (deducido, sin test;
/// hasta un borrador anterior del §828, aceptaba).
#[cfg(unix)]
fn comparar_duenos(p: &Path, duenos: &[(PathBuf, u32)], de_quien: u32, queda: &str) -> Result<(), LayerError> {
    let (que, como) = if queda == QUEDA_MIGRADO {
        ("migrar a la version 2 -abrirlo con su frase lo migra-", "abrelo con su frase como su dueno, que lo migra")
    } else {
        ("recifrar", "recifralo como su dueno")
    };
    let Some(&(_, del_dir)) = duenos.first() else {
        return Err(StoreError::Malformed(format!(
            "{}: no se sabe de quien es el libro, y el usuario {de_quien} no lo puede {que}. No se copia, no \
             se sella ni se renombra nada",
            p.display()
        ))
        .into());
    };
    if let Some((ruta, otro)) = duenos.iter().find(|(_, u)| *u != del_dir) {
        return Err(StoreError::Malformed(format!(
            "{}: es del usuario {otro}, y el directorio del libro del usuario {del_dir}: el libro no es de un \
             solo dueno, y lo intenta {que} el usuario {de_quien}, que lo dejaria todo suyo; que sea todo de su \
             dueno, y {como}. No se copia, no se sella ni se renombra nada",
            ruta.display()
        ))
        .into());
    }
    if de_quien != del_dir {
        return Err(StoreError::Malformed(format!(
            "{}: el libro es del usuario {del_dir}, y lo intenta {que} el usuario {de_quien}: asi \
             quedaria del usuario {de_quien}, y su dueno no lo abriria; {como}. No se copia, no se sella ni se \
             renombra nada",
            p.display()
        ))
        .into());
    }
    Ok(())
}
