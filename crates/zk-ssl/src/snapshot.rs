//! **Instantáneas del ledger**: copia y restauración.
//!
//! Sin esto, un disco perdido es el ledger perdido.
//!
//! ## La decisión que importa: la restauración VERIFICA
//!
//! Exportar y volver a leer es lo fácil. Lo que importa es que
//! **restaurar no cargue en silencio un estado inconsistente**.
//!
//! Al importar se reconstruyen los dos árboles desde las hojas del
//! fichero, se recalculan sus raíces y se comparan con las que la
//! instantánea declara. Si no coinciden, la importación **falla**.
//!
//! Es el mismo razonamiento que en `persistence::load`: un ledger
//! restaurado a medias haría que el nodo generase **pruebas válidas de
//! transiciones sobre un estado que no es el real** — indetectable desde
//! fuera, porque las pruebas verificarían.
//!
//! ## Formato
//!
//! Binario explícito y documentado, no el formato interno del motor de
//! almacenamiento. Una copia de archivo debe poder leerse dentro de diez
//! años sin depender de la versión de `sled` que la escribió.
//!
//! ```text
//! MAGIC              8 B   "ZKSSL1\0\0"
//! custodian_root    32 B
//! governance_root   32 B
//! state_root        32 B   ← para verificar
//! nullifier_root    32 B   ← para verificar
//! frozen_root       32 B   ← para verificar
//! regulatory_limit   8 B
//! max_supply         8 B
//! max_accounts       8 B
//! total_supply       8 B
//! recovery_count     8 B
//! freeze_count       8 B
//! gov_change_count   8 B
//! next_index         8 B
//! n_accounts         8 B
//! n_nullifiers       8 B
//! n_frozen           8 B
//! n_log              8 B
//! ── cuentas ──      (8 + 48) B cada una
//! ── nullifiers ──   (8 + 32) B cada uno
//! ── congeladas ──   (8 + 32) B cada una
//! ── registro ──     137 B cada entrada
//! ```
//!
//! ## ⚠️ Lo que NO es
//!
//! - **No hay replicación en vivo.** Es una copia puntual; los cambios
//!   posteriores no se propagan. Replicar en caliente exige red y
//!   coordinación, que es el problema de sistemas distribuidos que este
//!   proyecto no aborda.
//! - **Va cifrada si la capa tiene clave**, con el mismo cifrado
//!   autenticado que el ledger. Sin clave va en claro, y entonces quien
//!   tenga el fichero **ve todos los saldos**.
//! - **No hay copias incrementales.** Cada instantánea es completa.

use std::collections::HashMap;
use std::io::{Read, Write};
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::FieldElement;

use stark_experiment::circuit_settlement::native_leaf;
use stark_experiment::merkle::Digest;

use super::*;
use crate::store::{digest_from_bytes, digest_to_bytes, record_from_bytes, record_to_bytes};

/// **Versión 3 del formato.** La 2 no incluía el registro de
/// transiciones, y restaurar perdía el historial.
///
/// La 1 La 1 no incluía las cuentas congeladas, y
/// restaurar desde ella levantaba todas las congelaciones. Cambiar la
/// firma hace que una copia antigua se rechace en vez de cargarse a
/// medias.
const MAGIC: &[u8; 8] = b"ZKSSL3\0\0";

/// Resumen de una instantánea, para registro y verificación externa.
#[derive(Clone, Debug)]
pub struct SnapshotInfo {
    pub accounts: u64,
    pub nullifiers: u64,
    pub frozen: u64,
    pub total_supply: u64,
    pub state_root: Digest,
    pub bytes: u64,
}

fn io_err<E: std::fmt::Display>(e: E) -> LayerError {
    LayerError::Store(crate::store::StoreError::Io(e.to_string()))
}

fn malformed(what: &str) -> LayerError {
    LayerError::Store(crate::store::StoreError::Malformed(what.to_string()))
}

/// Marca de instantánea sin cifrar.
const SNAPSHOT_PLAIN: u8 = 0x00;
/// Marca de instantánea cifrada.
const SNAPSHOT_ENCRYPTED: u8 = 0x01;

impl SovereignLayer {
    /// Importa una instantánea cifrada.
    ///
    /// Existe aparte porque `import_snapshot` no tiene forma de conocer la
    /// clave: es una función asociada, no un método.
    pub fn import_snapshot_with_key(
        path: &str,
        key: &crate::crypto::LedgerKey,
    ) -> Result<Self, LayerError> {
        let mut buf = Vec::new();
        std::fs::File::open(path)
            .map_err(io_err)?
            .read_to_end(&mut buf)
            .map_err(io_err)?;

        let plano = match buf.first() {
            Some(&SNAPSHOT_ENCRYPTED) => key.open(&buf[1..]).map_err(LayerError::Store)?,
            Some(&SNAPSHOT_PLAIN) => buf[1..].to_vec(),
            _ => {
                return Err(LayerError::Store(crate::store::StoreError::Malformed(
                    "instantanea sin marca de cifrado".into(),
                )))
            }
        };

        let tmp = format!("{path}.tmp-descifrada");
        let mut con_marca = vec![SNAPSHOT_PLAIN];
        con_marca.extend_from_slice(&plano);
        std::fs::write(&tmp, &con_marca).map_err(io_err)?;
        let r = Self::import_snapshot(&tmp);
        let _ = std::fs::remove_file(&tmp);
        r
    }

    /// Exporta el estado completo a un fichero.
    /// Exporta una instantánea.
    ///
    /// **Si la capa tiene clave, la instantánea va cifrada**, con la misma
    /// clave y el mismo cifrado autenticado que el ledger.
    ///
    /// Una versión anterior nunca cifraba: el disco quedaba protegido con
    /// XChaCha20-Poly1305 y la instantánea en claro. Eran **dos niveles de
    /// protección distintos para el mismo dato**, y la instantánea es
    /// justo la que se copia fuera del nodo.
    pub fn export_snapshot(&self, path: &str) -> Result<SnapshotInfo, LayerError> {
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&digest_to_bytes(&self.custodian_set_root));
        out.extend_from_slice(&digest_to_bytes(&self.governance_set_root));
        out.extend_from_slice(&digest_to_bytes(&self.accounts.root()));
        out.extend_from_slice(&digest_to_bytes(&self.nullifiers.root()));
        out.extend_from_slice(&digest_to_bytes(&self.frozen.root()));
        out.extend_from_slice(&self.regulatory_limit.to_le_bytes());
        out.extend_from_slice(&self.max_supply.to_le_bytes());
        out.extend_from_slice(&self.max_accounts.to_le_bytes());
        out.extend_from_slice(&self.total_supply.to_le_bytes());
        out.extend_from_slice(&self.recovery_count.to_le_bytes());
        out.extend_from_slice(&self.freeze_count.to_le_bytes());
        out.extend_from_slice(&self.governance_change_count.to_le_bytes());
        out.extend_from_slice(&self.next_index.to_le_bytes());

        // Orden determinista: dos instantaneas del mismo estado deben dar
        // ficheros identicos, o compararlas seria inutil.
        let mut accounts: Vec<(&AccountIndex, &AccountRecord)> = self.records.iter().collect();
        accounts.sort_by_key(|(i, _)| **i);
        let mut nullifiers: Vec<(u64, Digest)> = self.nullifiers.occupied();
        nullifiers.sort_by_key(|(p, _)| *p);
        let mut frozen: Vec<(u64, Digest)> = self.frozen.occupied();
        frozen.sort_by_key(|(p, _)| *p);

        out.extend_from_slice(&(accounts.len() as u64).to_le_bytes());
        out.extend_from_slice(&(nullifiers.len() as u64).to_le_bytes());
        out.extend_from_slice(&(frozen.len() as u64).to_le_bytes());
        out.extend_from_slice(&(self.log.len() as u64).to_le_bytes());

        for (index, r) in &accounts {
            out.extend_from_slice(&index.to_le_bytes());
            out.extend_from_slice(&record_to_bytes(&r.public_id, r.balance, r.nonce));
        }
        for (position, n) in &nullifiers {
            out.extend_from_slice(&position.to_le_bytes());
            out.extend_from_slice(&digest_to_bytes(n));
        }
        for (index, leaf) in &frozen {
            out.extend_from_slice(&index.to_le_bytes());
            out.extend_from_slice(&digest_to_bytes(leaf));
        }
        // El registro entero. Sin el, restaurar perderia el historial y
        // la capa restaurada encadenaria desde la nada.
        for e in self.log.entries() {
            out.extend_from_slice(&crate::store::log_entry_to_bytes(e));
        }

        // Cifrado con la misma clave que el ledger, si la hay. Se antepone
        // un byte de marca para que la importacion sepa que tiene delante
        // sin adivinar.
        let bytes = match &self.key {
            Some(k) => {
                let mut v = vec![SNAPSHOT_ENCRYPTED];
                v.extend_from_slice(&k.seal(&out).map_err(LayerError::Store)?);
                v
            }
            None => {
                let mut v = vec![SNAPSHOT_PLAIN];
                v.extend_from_slice(&out);
                v
            }
        };

        let mut f = std::fs::File::create(path).map_err(io_err)?;
        f.write_all(&bytes).map_err(io_err)?;
        f.sync_all().map_err(io_err)?;

        Ok(SnapshotInfo {
            accounts: accounts.len() as u64,
            nullifiers: nullifiers.len() as u64,
            frozen: frozen.len() as u64,
            total_supply: self.total_supply,
            state_root: self.accounts.root(),
            bytes: out.len() as u64,
        })
    }

    /// Restaura una capa **en memoria** desde una instantánea.
    ///
    /// **Verifica la integridad**: reconstruye los árboles y comprueba
    /// que sus raíces coinciden con las declaradas. Si no, falla.
    pub fn import_snapshot(path: &str) -> Result<Self, LayerError> {
        let mut buf = Vec::new();
        std::fs::File::open(path)
            .map_err(io_err)?
            .read_to_end(&mut buf)
            .map_err(io_err)?;

        // ⚠️ `import_snapshot` es una funcion asociada: no tiene capa ni
        // clave. Una instantanea cifrada exige `import_snapshot_with_key`.
        let buf = match buf.first() {
            Some(&SNAPSHOT_PLAIN) => buf[1..].to_vec(),
            Some(&SNAPSHOT_ENCRYPTED) => {
                return Err(LayerError::Store(crate::store::StoreError::Malformed(
                    "la instantanea esta cifrada: usa import_snapshot_with_key".into(),
                )))
            }
            _ => {
                return Err(LayerError::Store(crate::store::StoreError::Malformed(
                    "instantanea sin marca de cifrado: formato desconocido".into(),
                )))
            }
        };

        let mut cursor = 0usize;
        let mut take = |n: usize, what: &str| -> Result<&[u8], LayerError> {
            if cursor + n > buf.len() {
                return Err(malformed(&format!("instantanea truncada en '{what}'")));
            }
            let s = &buf[cursor..cursor + n];
            cursor += n;
            Ok(s)
        };

        if take(8, "magic")? != MAGIC {
            return Err(malformed("cabecera desconocida: no es una instantanea ZK-SSL"));
        }
        let custodian_set_root = digest_from_bytes(take(32, "custodian_root")?)?;
        let governance_set_root = digest_from_bytes(take(32, "governance_root")?)?;
        let declared_state = digest_from_bytes(take(32, "state_root")?)?;
        let declared_null = digest_from_bytes(take(32, "nullifier_root")?)?;
        let declared_frozen = digest_from_bytes(take(32, "frozen_root")?)?;

        let mut u64_at = |what: &str| -> Result<u64, LayerError> {
            Ok(u64::from_le_bytes(
                take(8, what)?
                    .try_into()
                    .map_err(|_| malformed(what))?,
            ))
        };
        let regulatory_limit = u64_at("regulatory_limit")?;
        let max_supply = u64_at("max_supply")?;
        let max_accounts = u64_at("max_accounts")?;
        let total_supply = u64_at("total_supply")?;
        let recovery_count = u64_at("recovery_count")?;
        let freeze_count = u64_at("freeze_count")?;
        let governance_change_count = u64_at("gov_change_count")?;
        let next_index = u64_at("next_index")?;
        let n_accounts = u64_at("n_accounts")?;
        let n_nullifiers = u64_at("n_nullifiers")?;
        let n_frozen = u64_at("n_frozen")?;
        let n_log = u64_at("n_log")?;

        let mut layer = Self {
            custodian_uses: 0,
            max_custodian_uses: crate::DEFAULT_MAX_CUSTODIAN_USES,
            accounts: SparseTree::new(),
            nullifiers: SparseTree::new(),
            records: HashMap::new(),
            next_index,
            custodian_set_root,
            governance_set_root,
            governance_change_count,
            frozen: SparseTree::with_depth(FROZEN_DEPTH),
            freeze_count,
            log: TransitionLog::new(),
            total_supply,
            recovery_count,
            regulatory_limit,
            max_supply,
            max_accounts,
            options: proof_options(),
            db: None,
            // Una capa importada arranca en memoria y sin cifrar: quien
            // la persista decidira con que clave.
            key: None,
        };

        for _ in 0..n_accounts {
            let index = u64::from_le_bytes(
                take(8, "indice de cuenta")?
                    .try_into()
                    .map_err(|_| malformed("indice de cuenta"))?,
            );
            let (public_id, balance, nonce) = record_from_bytes(take(48, "registro")?)?;
            layer
                .accounts
                .set_leaf(index, native_leaf(public_id, BaseElement::new(balance), nonce));
            layer.records.insert(
                index,
                AccountRecord {
                    public_id,
                    balance,
                    nonce,
                },
            );
        }
        for _ in 0..n_nullifiers {
            let position = u64::from_le_bytes(
                take(8, "posicion de nullifier")?
                    .try_into()
                    .map_err(|_| malformed("posicion de nullifier"))?,
            );
            let n = digest_from_bytes(take(32, "nullifier")?)?;
            layer.nullifiers.set_leaf(position, n);
        }

        for _ in 0..n_frozen {
            let index = u64::from_le_bytes(
                take(8, "indice de congelada")?
                    .try_into()
                    .map_err(|_| malformed("indice de congelada"))?,
            );
            let leaf = digest_from_bytes(take(32, "hoja congelada")?)?;
            layer.frozen.set_leaf(index, leaf);
        }

        let mut entradas = Vec::with_capacity(n_log as usize);
        for _ in 0..n_log {
            entradas.push(crate::store::log_entry_from_bytes(take(137, "entrada del registro")?)?);
        }
        layer.log = TransitionLog::from_entries(entradas);

        // ===== VERIFICACION DE INTEGRIDAD =====
        // Reconstruir no basta: hay que comprobar que lo reconstruido es
        // lo que la instantanea declara. Restaurar en silencio un estado
        // inconsistente haria que el nodo generase pruebas validas sobre
        // un ledger que no es el real.
        if layer.accounts.root() != declared_state {
            return Err(LayerError::Store(
                crate::store::StoreError::IntegrityFailure {
                    what: "arbol de cuentas de la instantanea",
                },
            ));
        }
        // El registro tambien se verifica: sin esto, manipular el
        // historial dentro de una instantanea pasaria inadvertido, porque
        // alterarlo NO cambia ninguna raiz de estado.
        layer.log.verify_chain().map_err(|e| {
            LayerError::Store(crate::store::StoreError::Malformed(format!(
                "registro de transiciones de la instantanea: {e}"
            )))
        })?;

        if layer.frozen.root() != declared_frozen {
            return Err(LayerError::Store(
                crate::store::StoreError::IntegrityFailure {
                    what: "arbol de congelados de la instantanea",
                },
            ));
        }
        if layer.nullifiers.root() != declared_null {
            return Err(LayerError::Store(
                crate::store::StoreError::IntegrityFailure {
                    what: "arbol de nullifiers de la instantanea",
                },
            ));
        }

        Ok(layer)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_support::*;
    use crate::*;
    use winterfell::math::fields::f64::BaseElement;

    fn temp_file(name: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!("zkssl_snap_{}_{}.bin", name, std::process::id()));
        let s = p.to_str().unwrap().to_string();
        let _ = std::fs::remove_file(&s);
        s
    }

    /// Directorio temporal para un ledger de prueba.
    ///
    /// `temp_file` da un fichero; los tests de cifrado necesitan además un
    /// directorio, porque `open_encrypted` abre un ledger persistente.
    fn temp_path(name: &str) -> String {
        let s = format!(
            "{}/zkssl-snap-{}-{}",
            std::env::temp_dir().display(),
            name,
            std::process::id()
        );
        // Se limpia ANTES, como hace `temp_file`. Si un test falla, su
        // directorio queda y el siguiente encontraria un ledger viejo con
        // estado que no le corresponde.
        let _ = std::fs::remove_dir_all(&s);
        s
    }

    /// **EL TEST CLAVE**: el estado sobrevive a la exportación y vuelta.
    #[test]
    fn a_snapshot_restores_the_full_state() {
        let file = temp_file("full");
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);
        let s = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 250_000)
            .expect("transferencia");
        layer.apply(&s, alice, bob, 250_000).expect("aplicar");

        let info = layer.export_snapshot(&file).expect("exportar");
        println!(
            "Instantanea: {} cuentas, {} nullifiers, {} bytes",
            info.accounts, info.nullifiers, info.bytes
        );

        let restored = SovereignLayer::import_snapshot(&file).expect("importar");
        assert_eq!(restored.balance_of(alice), Some(750_000));
        assert_eq!(restored.balance_of(bob), Some(300_000));
        assert_eq!(restored.total_supply(), 1_050_000);
        assert_eq!(restored.state_root(), layer.state_root());
        assert_eq!(
            restored.nullifier_root(),
            layer.nullifier_root(),
            "los nullifiers gastados deben sobrevivir, o restaurar permitiria regastar"
        );
        let _ = std::fs::remove_file(&file);
    }

    /// **UNA INSTANTÁNEA MANIPULADA SE DETECTA.**
    ///
    /// Sin esta verificación, restaurar una copia alterada haría que el
    /// nodo generase pruebas válidas sobre un ledger que no es el real —
    /// indetectable desde fuera.
    #[test]
    fn a_tampered_snapshot_is_rejected() {
        let file = temp_file("tampered");
        let mut layer = new_layer();
        open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        layer.export_snapshot(&file).expect("exportar");

        // Alterar un byte del REGISTRO DE CUENTA, sin tocar la raiz
        // declarada.
        //
        // La posicion se calcula desde la cabecera en vez de contar desde
        // el final: el final del fichero es ahora el registro de
        // transiciones, y manipularlo prueba otra cosa (ver el test
        // siguiente).
        const CABECERA: usize = 8 + 32 * 5 + 8 * 8 + 8 * 4; // 264
        let mut bytes = std::fs::read(&file).expect("leer");
        bytes[CABECERA + 20] ^= 0xFF;
        std::fs::write(&file, &bytes).expect("escribir");

        // No se imprime `r`: `SovereignLayer` NO implementa `Debug` a
        // proposito, porque contiene los saldos de todas las cuentas y no
        // debe acabar en un registro de diagnostico por accidente. Misma
        // razon por la que `LedgerKey` oculta su contenido.
        let detected = matches!(
            SovereignLayer::import_snapshot(&file),
            Err(LayerError::Store(StoreError::IntegrityFailure { .. }))
        );
        assert!(
            detected,
            "CRITICO: una instantanea manipulada debe detectarse ANTES de operar"
        );
        let _ = std::fs::remove_file(&file);
    }

    /// Un fichero que no es una instantánea se rechaza con claridad.
    #[test]
    fn a_foreign_file_is_rejected() {
        let file = temp_file("foreign");
        std::fs::write(&file, b"esto no es una instantanea del ledger").expect("escribir");
        assert!(SovereignLayer::import_snapshot(&file).is_err());
        let _ = std::fs::remove_file(&file);
    }

    /// Una instantánea truncada se rechaza en vez de interpretarse.
    #[test]
    fn a_truncated_snapshot_is_rejected() {
        let file = temp_file("truncated");
        let mut layer = new_layer();
        open_and_fund(&mut layer, SK_ALICE, 1000);
        layer.export_snapshot(&file).expect("exportar");

        let bytes = std::fs::read(&file).expect("leer");
        std::fs::write(&file, &bytes[..bytes.len() / 2]).expect("truncar");

        assert!(
            SovereignLayer::import_snapshot(&file).is_err(),
            "un fichero truncado debe rechazarse, no interpretarse a medias"
        );
        let _ = std::fs::remove_file(&file);
    }

    /// **Dos instantáneas del mismo estado son idénticas.**
    ///
    /// Sin orden determinista, comparar copias para detectar divergencias
    /// sería inútil.
    #[test]
    fn snapshots_of_the_same_state_are_identical() {
        let f1 = temp_file("det1");
        let f2 = temp_file("det2");
        let mut layer = new_layer();
        open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        open_and_fund(&mut layer, SK_BOB, 50_000);

        layer.export_snapshot(&f1).expect("exportar 1");
        layer.export_snapshot(&f2).expect("exportar 2");

        assert_eq!(
            std::fs::read(&f1).unwrap(),
            std::fs::read(&f2).unwrap(),
            "dos instantaneas del mismo estado deben ser byte a byte iguales"
        );
        let _ = std::fs::remove_file(&f1);
        let _ = std::fs::remove_file(&f2);
    }

    /// **LAS CONGELACIONES SOBREVIVEN A LA INSTANTÁNEA.**
    ///
    /// La versión 1 del formato no las incluía: restaurar desde una copia
    /// **levantaba todas las congelaciones**, el mismo fallo que ya tenía
    /// la persistencia por otra vía.
    #[test]
    fn a_snapshot_preserves_frozen_accounts() {
        let file = temp_file("frozen");
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 500_000);

        let f = layer.set_frozen(&valid_auth(), alice, true).expect("congelar");
        layer.apply_freeze(&f, alice).expect("aplicar");

        let info = layer.export_snapshot(&file).expect("exportar");
        assert_eq!(info.frozen, 1, "la instantanea debe incluir la congelada");

        let restored = SovereignLayer::import_snapshot(&file).expect("importar");
        assert!(
            restored.is_frozen(alice),
            "CRITICO: restaurar una copia NO debe levantar las congelaciones"
        );
        assert!(!restored.is_frozen(bob));
        assert_eq!(restored.freeze_count(), 1);
        assert_eq!(restored.frozen_root(), layer.frozen_root());
        let _ = std::fs::remove_file(&file);
    }

    /// **EL HISTORIAL SOBREVIVE A LA INSTANTÁNEA.**
    ///
    /// La versión 2 del formato no lo incluía: restaurar perdía el
    /// registro, y la capa restaurada encadenaba desde la nada.
    #[test]
    fn a_snapshot_preserves_the_transition_log() {
        let file = temp_file("log");
        let mut layer = new_layer();
        let genesis = layer.state_root();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let s = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 1000)
            .expect("transferencia");
        layer.apply(&s, alice, bob, 1000).expect("aplicar");

        layer.export_snapshot(&file).expect("exportar");
        let restored = SovereignLayer::import_snapshot(&file).expect("importar");

        // Dos aperturas + una emision + una transferencia.
        assert_eq!(restored.transition_log().len(), 4);
        assert_eq!(
            restored.log_head(),
            layer.log_head(),
            "CRITICO: restaurar NO debe perder el historial"
        );
        restored
            .transition_log()
            .verify(genesis)
            .expect("el registro restaurado debe verificar");
        let _ = std::fs::remove_file(&file);
    }

    /// **MANIPULAR EL HISTORIAL DENTRO DE UNA INSTANTÁNEA SE DETECTA.**
    ///
    /// Es un caso distinto del anterior: alterar el registro **no cambia
    /// ninguna raíz de estado**, así que la comprobación de integridad de
    /// los árboles no lo vería. Lo detecta `verify_chain`, por el
    /// encadenamiento de resúmenes.
    ///
    /// Sin este test, el hueco habría pasado inadvertido: la primera
    /// versión de `a_tampered_snapshot_is_rejected` alteraba un byte del
    /// final del fichero, que tras añadir el registro dejó de ser un
    /// saldo y pasó a ser una entrada del historial.
    #[test]
    fn tampering_with_the_log_inside_a_snapshot_is_detected() {
        let file = temp_file("logtamper");
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let s = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 1000)
            .expect("transferencia");
        layer.apply(&s, alice, bob, 1000).expect("aplicar");
        layer.export_snapshot(&file).expect("exportar");

        // El registro esta al final del fichero.
        let mut bytes = std::fs::read(&file).expect("leer");
        let len = bytes.len();
        bytes[len - 20] ^= 0xFF;
        std::fs::write(&file, &bytes).expect("escribir");

        assert!(
            SovereignLayer::import_snapshot(&file).is_err(),
            "CRITICO: manipular el historial dentro de una copia debe \
             detectarse, aunque no cambie ninguna raiz de estado"
        );
        let _ = std::fs::remove_file(&file);
    }

    // -----------------------------------------------------------------
    // Cifrado de instantáneas
    // -----------------------------------------------------------------

    /// **EL SALDO NO APARECE EN UNA INSTANTÁNEA CIFRADA.**
    ///
    /// La instantánea es lo que se copia fuera del nodo: a una cinta, a
    /// otro servidor, a un disco que alguien se lleva. Era el artefacto
    /// **más expuesto con la protección más débil**.
    #[test]
    fn an_encrypted_snapshot_does_not_show_balances() {
        let path = temp_path("snapcrypt");
        let file = temp_file("snapcrypt");
        const SALDO: u64 = 0x05A3_B7C9; // 94.615.497, distintivo
        let key = crypto::LedgerKey::from_passphrase("una contrasena larga de prueba");
        {
            let mut layer = SovereignLayer::open_encrypted(
                &path,
                custodian_root(),
                governance_root(),
                LIMIT,
                MAX_SUPPLY,
                MAX_ACCOUNTS,
                Some(key.clone()),
            )
            .expect("abrir cifrada");
            open_and_fund(&mut layer, SK_ALICE, SALDO);
            layer.export_snapshot(&file).expect("exportar");
        }

        let bytes = std::fs::read(&file).expect("leer");
        let patron = &SALDO.to_le_bytes()[..4];
        assert!(
            !bytes.windows(4).any(|w| w == patron),
            "CRITICO: el saldo aparece EN CLARO en la instantanea cifrada"
        );
        let _ = std::fs::remove_file(&file);
        let _ = std::fs::remove_dir_all(&path);
    }

    /// **Y el que valida al anterior**: sin clave, el saldo SÍ aparece.
    ///
    /// Sin esto, el test anterior pasaría aunque la búsqueda estuviera mal
    /// construida o el saldo no llegara a guardarse.
    #[test]
    fn an_unencrypted_snapshot_does_show_balances() {
        let file = temp_file("snapplain");
        const SALDO: u64 = 0x05A3_B7C9;
        let mut layer = new_layer();
        open_and_fund(&mut layer, SK_ALICE, SALDO);
        layer.export_snapshot(&file).expect("exportar");

        let bytes = std::fs::read(&file).expect("leer");
        let patron = &SALDO.to_le_bytes()[..4];
        assert!(
            bytes.windows(4).any(|w| w == patron),
            "sin cifrado el saldo DEBE aparecer, o el test anterior no \
             comprueba nada"
        );
        let _ = std::fs::remove_file(&file);
    }

    /// **UNA INSTANTÁNEA CIFRADA NO SE ABRE SIN CLAVE.**
    ///
    /// Y el error dice qué usar, en vez de un fallo de formato
    /// incomprensible.
    #[test]
    fn an_encrypted_snapshot_cannot_be_imported_without_the_key() {
        let path = temp_path("snapnokey");
        let file = temp_file("snapnokey");
        let key = crypto::LedgerKey::from_passphrase("una contrasena larga de prueba");
        {
            let mut layer = SovereignLayer::open_encrypted(
                &path,
                custodian_root(),
                governance_root(),
                LIMIT,
                MAX_SUPPLY,
                MAX_ACCOUNTS,
                Some(key.clone()),
            )
            .expect("abrir");
            open_and_fund(&mut layer, SK_ALICE, 100_000);
            layer.export_snapshot(&file).expect("exportar");
        }
        assert!(
            SovereignLayer::import_snapshot(&file).is_err(),
            "una instantanea cifrada no debe abrirse sin clave"
        );
        let _ = std::fs::remove_file(&file);
        let _ = std::fs::remove_dir_all(&path);
    }

    /// **Y CON LA CLAVE SÍ, con el estado intacto.**
    #[test]
    fn an_encrypted_snapshot_restores_with_the_key() {
        let path = temp_path("snapkey");
        let file = temp_file("snapkey");
        const SALDO: u64 = 250_000;
        let key = crypto::LedgerKey::from_passphrase("una contrasena larga de prueba");
        let alice;
        let raiz;
        {
            let mut layer = SovereignLayer::open_encrypted(
                &path,
                custodian_root(),
                governance_root(),
                LIMIT,
                MAX_SUPPLY,
                MAX_ACCOUNTS,
                Some(key.clone()),
            )
            .expect("abrir");
            alice = open_and_fund(&mut layer, SK_ALICE, SALDO);
            raiz = layer.state_root();
            layer.export_snapshot(&file).expect("exportar");
        }

        let restaurada =
            SovereignLayer::import_snapshot_with_key(&file, &key).expect("importar con clave");
        assert_eq!(restaurada.balance_of(alice), Some(SALDO));
        assert_eq!(restaurada.state_root(), raiz);
        let _ = std::fs::remove_file(&file);
        let _ = std::fs::remove_dir_all(&path);
    }

    /// **Y una clave incorrecta no la abre.**
    ///
    /// El cifrado es autenticado: una clave equivocada no descifra basura,
    /// falla.
    #[test]
    fn the_wrong_key_does_not_open_a_snapshot() {
        let path = temp_path("snapbadkey");
        let file = temp_file("snapbadkey");
        let buena = crypto::LedgerKey::from_passphrase("la correcta y larga");
        let mala = crypto::LedgerKey::from_passphrase("la incorrecta y larga");
        {
            let mut layer = SovereignLayer::open_encrypted(
                &path,
                custodian_root(),
                governance_root(),
                LIMIT,
                MAX_SUPPLY,
                MAX_ACCOUNTS,
                Some(buena.clone()),
            )
            .expect("abrir");
            open_and_fund(&mut layer, SK_ALICE, 100_000);
            layer.export_snapshot(&file).expect("exportar");
        }
        assert!(
            SovereignLayer::import_snapshot_with_key(&file, &mala).is_err(),
            "CRITICO: una clave incorrecta no debe abrir la instantanea"
        );
        let _ = std::fs::remove_file(&file);
        let _ = std::fs::remove_dir_all(&path);
    }
}
