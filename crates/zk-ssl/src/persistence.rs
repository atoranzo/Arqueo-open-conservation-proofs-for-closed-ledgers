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
    /// La clave la aporta el operador y **no se guarda junto a los
    /// datos**: guardarla al lado no protegería nada. Eso significa que
    /// el nodo **no puede reiniciar solo** — alguien tiene que
    /// introducirla.
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
        let db = sled::open(path).map_err(|e| StoreError::Io(e.to_string()))?;

        let mut layer = Self {
            accounts: SparseTree::new(),
            pending: SparseTree::new(),
            next_pending: 0,
            pending_amounts: HashMap::new(),
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
            recovery_count: 0,
            regulatory_limit,
            max_supply,
            max_accounts,
            options: proof_options(),
            db: Some(db),
            key,
        };

        if layer.has_existing_ledger()? {
            layer.load()?;
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
        for item in db.scan_prefix(b"acct:") {
            let (k, v) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            let v = unseal_one(v)?;
            let idx = u64::from_le_bytes(
                k[5..]
                    .try_into()
                    .map_err(|_| StoreError::Malformed("indice de cuenta".into()))?,
            );
            let (public_id, balance, nonce) = record_from_bytes(&v)?;
            self.accounts.set_leaf(
                idx,
                native_leaf(public_id, BaseElement::new(balance), nonce),
            );
            self.records.insert(
                idx,
                AccountRecord {
                    public_id,
                    balance,
                    nonce,
                },
            );
        }

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

        for item in db.scan_prefix(b"froz:") {
            let (k, v) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            let v = unseal_one(v)?;
            let idx = u64::from_le_bytes(
                k[5..]
                    .try_into()
                    .map_err(|_| StoreError::Malformed("indice de congelada".into()))?,
            );
            self.frozen.set_leaf(idx, digest_from_bytes(&v)?);
        }

        // Hojas del arbol de PENDIENTES.
        //
        // Sin esto, un reinicio borraria las transferencias sin reclamar:
        // el dinero saldria de la cuenta del pagador y **no llegaria a
        // ninguna parte**.
        for item in db.scan_prefix(b"pend:") {
            let (k, v) = item.map_err(|e| StoreError::Io(e.to_string()))?;
            let v = unseal_one(v)?;
            let pos = u64::from_le_bytes(
                k[5..]
                    .try_into()
                    .map_err(|_| StoreError::Malformed("posicion de pendiente".into()))?,
            );
            self.pending.set_leaf(pos, digest_from_bytes(&v)?);
        }

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
        &self,
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
            Some(d) => d,
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
        batch.insert(b"meta:next_index".as_ref(), self.seal(self.next_index.to_le_bytes().to_vec())?);
        batch.insert(b"root:state".as_ref(), self.seal(digest_to_bytes(&self.accounts.root()).to_vec())?);

        // --- Cuentas afectadas ---
        for index in accounts {
            if let Some(r) = self.records.get(index) {
                let mut key = b"acct:".to_vec();
                key.extend_from_slice(&index.to_le_bytes());
                batch.insert(
                    key,
                    self.seal(record_to_bytes(&r.public_id, r.balance, r.nonce).to_vec())?,
                );
            }
        }

        // --- Registro de transiciones ---
        //
        // Se escriben todas las entradas en cada lote. Sin persistirlo, el
        // operador borraria el historial reiniciando el nodo — que es
        // justo lo que el registro existe para impedir.
        for e in self.log.entries() {
            let mut key = b"log:".to_vec();
            key.extend_from_slice(&e.seq.to_le_bytes());
            batch.insert(key, self.seal(crate::store::log_entry_to_bytes(e))?);
        }

        // --- Cuentas congeladas ---
        //
        // Se escriben TODAS en cada lote. Las congelaciones son raras, asi
        // que el conjunto es pequeno; hacerlo incremental complicaria el
        // codigo sin ganancia medible.
        //
        // Sin esto, reiniciar el nodo levantaria todas las congelaciones:
        // el contador sobreviviria pero el arbol no, y una cuenta bajo
        // investigacion volveria a poder gastar.
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
        }

        db.apply_batch(batch)
            .map_err(|e| StoreError::Io(e.to_string()))?;
        db.flush().map_err(|e| StoreError::Io(e.to_string()))?;
        Ok(())
    }
}
