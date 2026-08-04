//! **El evento de migración de B13/B14** (spec: `doc/`, paso 1b).
//!
//! Reposiciona cada cuenta a `public_id[0] mod capacidad` (sondeo lineal
//! determinista, orden de índice viejo ascendente), envuelve cada hoja
//! con `native_leaf_salted(.., r.leaf_salt)` —el salt DEL RECORD: cero
//! para lo legacy, real para lo abierto post-1a—, y reconstruye frozen a
//! profundidad 32. NO toca pendientes, cuota (`next_index` = censo) ni
//! sled (sub-bloque 3; precedente `limpieza.remove` de legacy_null).
//!
//! ⚠️ **No ejecutar en vivo hasta el flip de circuitos** (spec D4): los
//! AIR de hoy verifican hoja SIN salt y frozen-24; migrar antes rompería
//! toda prueba. Esta función existe para el flip y para sus tests.

use super::*;
use stark_experiment::circuit_settlement::native_leaf_salted;
use std::collections::HashMap;

/// Profundidad del árbol de congelados POST-migración (spec §137: crecer
/// frozen a 32 es la salida elegida; el flip redefine FROZEN_DEPTH).
pub(crate) const FROZEN_DEPTH_POST: usize = 32;
/// Profundidad PRE-migración (mundo viejo): las vías de importación
/// legacy (sled sin marcador, snapshots ≤v6) reconstruyen frozen a 24.
pub(crate) const FROZEN_DEPTH_PRE: usize = 24;

impl SovereignLayer {
    /// ¿Consta ya una migración en el registro? El log protegiéndose a
    /// sí mismo: re-envolver hojas ya envueltas sería corrupción
    /// silenciosa, así que la segunda ejecución se RECHAZA.
    pub fn has_migration_entry(&self) -> bool {
        self.log
            .entries()
            .iter()
            .any(|e| matches!(e.kind, crate::log::OpKind::Migration))
    }

    /// Ejecuta la migración única. Devuelve el mapa `(índice_viejo,
    /// índice_nuevo)` — lo necesita el operador (p. ej. re-registrar el
    /// mapa IBAN→índice, que no se persiste) y los tests.
    pub fn migrate_to_salted_positions(
        &mut self,
    ) -> Result<Vec<(AccountIndex, AccountIndex)>, LayerError> {
        // Doble pata: el log en memoria O el marcador en disco. Si el log
        // no persistiera, reabrir y re-migrar re-envolveria las hojas.
        if self.has_migration_entry()
            || self
                .db()
                .and_then(|db| db.get(b"meta:migrated").ok().flatten())
                .is_some()
        {
            // TODO: variante propia de LayerError cuando el enum se toque.
            return Err(LayerError::VerificationFailed(
                "la migracion ya consta en el registro: no se re-ejecuta".into(),
            ));
        }
        let cap = self.accounts.capacity();
        let root_old = self.accounts.root();
        let frozen_old = self.frozen.root();

        // Orden determinista: índice viejo ascendente. La réplica exacta
        // depende de este orden (fija el sondeo).
        let mut olds: Vec<AccountIndex> = self.records.keys().copied().collect();
        olds.sort_unstable();

        let mut new_records: HashMap<AccountIndex, AccountRecord> = HashMap::new();
        let mut new_accounts = SparseTree::new();
        let mut mapa = Vec::with_capacity(olds.len());
        for old in olds {
            let r = self.records.get(&old).expect("censado").clone();
            let mut pos = r.public_id[0].as_int() % cap;
            while new_records.contains_key(&pos) {
                pos = (pos + 1) % cap;
            }
            new_accounts.set_leaf(
                pos,
                native_leaf_salted(
                    r.public_id,
                    BaseElement::new(r.balance),
                    r.nonce,
                    r.leaf_salt,
                ),
            );
            new_records.insert(pos, r);
            mapa.push((old, pos));
        }

        // Frozen a profundidad 32, marcas remapeadas. Una marca sin
        // cuenta asociada (caso raro) se CONSERVA en su índice viejo —
        // cabe en 32, y descartar una congelación sería grave.
        let lookup: HashMap<AccountIndex, AccountIndex> = mapa.iter().copied().collect();
        let mut new_frozen = SparseTree::with_depth(FROZEN_DEPTH_POST);
        for (old_idx, leaf) in self.frozen.occupied() {
            let dest = lookup.get(&old_idx).copied().unwrap_or(old_idx);
            new_frozen.set_leaf(dest, leaf);
        }

        self.accounts = new_accounts;
        self.records = new_records;
        self.frozen = new_frozen;

        // Compromiso de la transición de frozen: payload de 64 B
        // (frozen_old || frozen_new). NO es una prueba — es el compromiso
        // replicable de la segunda raíz, encadenado por su digest.
        let mut payload = Vec::with_capacity(64);
        payload.extend_from_slice(&crate::store::digest_to_bytes(&frozen_old));
        payload.extend_from_slice(&crate::store::digest_to_bytes(&self.frozen.root()));
        self.log
            .append(crate::log::OpKind::Migration, root_old, self.accounts.root(), &payload);
        // --- Sub-frente sled: reescritura de los keyspaces indexados ---
        // acct:{viejo} y froz:{viejo} quedarian stale (un reinicio los
        // cargaria junto a los nuevos). Precedente: limpieza legacy_null.
        // ⚠️ Un snapshot exportado post-migracion NO reimporta hasta el
        // flip (v7 reconstruye con salt); coherente con D4.
        if self.db().is_some() {
            let nuevos: Vec<AccountIndex> = mapa.iter().map(|&(_, n)| n).collect();
            {
                let db = self.db().expect("comprobado");
                let mut limpieza = sled::Batch::default();
                for pref in [b"acct:".as_ref(), b"froz:".as_ref()] {
                    for item in db.scan_prefix(pref) {
                        let (k, _) = item
                            .map_err(|e| crate::store::StoreError::Io(e.to_string()))?;
                        limpieza.remove(k);
                    }
                }
                db.apply_batch(limpieza)
                    .map_err(|e| crate::store::StoreError::Io(e.to_string()))?;
                db.insert(b"meta:migrated", &[1u8][..])
                    .map_err(|e| crate::store::StoreError::Io(e.to_string()))?;
            }
            self.commit(&nuevos, None)?;
            if let Some(db) = self.db() {
                db.flush()
                    .map_err(|e| crate::store::StoreError::Io(e.to_string()))?;
            }
        }
        Ok(mapa)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_support::*;

    /// **Réplica determinista: la verificación del evento sin prueba.**
    /// Dos capas construidas igual migran a raíces y mapas IDÉNTICOS —
    /// esa es la propiedad que sustituye a la prueba ZK (D2 de la spec).
    #[test]
    fn dos_capas_iguales_migran_a_raices_identicas() {
        let mut a = new_layer();
        let mut b = new_layer();
        for l in [&mut a, &mut b] {
            open_and_fund(l, SK_ALICE, 1_000_000);
            open_and_fund(l, SK_BOB, 500_000);
        }
        let ma = a.migrate_to_salted_positions().expect("migra a");
        let mb = b.migrate_to_salted_positions().expect("migra b");
        assert_eq!(ma, mb, "mapas distintos: la replica no es exacta");
        assert_eq!(a.accounts.root(), b.accounts.root(), "raices de cuentas difieren");
        assert_eq!(a.frozen.root(), b.frozen.root(), "raices de frozen difieren");
    }

    /// **Conservación**: cada cuenta reencontrable en su posición nueva
    /// con el record intacto, y cada hoja del árbol ES la envuelta con
    /// el salt DEL RECORD (la enmienda a la spec). Pendientes intactos.
    #[test]
    fn cada_cuenta_sobrevive_con_su_hoja_envuelta() {
        use stark_experiment::circuit_settlement::native_leaf_salted;
        let mut l = new_layer();
        open_and_fund(&mut l, SK_ALICE, 1_000_000);
        open_and_fund(&mut l, SK_BOB, 250_000);
        let antes = l.records.clone();
        let pending_antes = l.pending.root();

        let mapa = l.migrate_to_salted_positions().expect("migra");
        assert_eq!(mapa.len(), antes.len(), "el mapa censa todas las cuentas");
        for (viejo, nuevo) in &mapa {
            let r0 = antes.get(viejo).expect("estaba antes");
            let r = l.records.get(nuevo).expect("reencontrable en su posicion nueva");
            assert_eq!(
                (r.public_id, r.balance, r.nonce, r.view_id, r.leaf_salt),
                (r0.public_id, r0.balance, r0.nonce, r0.view_id, r0.leaf_salt),
                "el record debe cruzar la migracion intacto"
            );
        }
        for (pos, leaf) in l.accounts.occupied() {
            let r = l.records.get(&pos).expect("record en cada hoja");
            assert_eq!(
                leaf,
                native_leaf_salted(r.public_id, BaseElement::new(r.balance), r.nonce, r.leaf_salt),
                "la hoja migrada debe ser native_leaf_salted con el salt DEL RECORD"
            );
        }
        assert_eq!(l.pending.root(), pending_antes, "los pendientes no se tocan");
    }

    /// **Una congelación sobrevive el remapa** a profundidad 32.
    #[test]
    fn una_congelacion_sobrevive_el_remapa() {
        let mut l = new_layer();
        let alice = open_and_fund(&mut l, SK_ALICE, 1_000_000);
        let marca: Digest = [BaseElement::new(0x46524F5A); 4]; // "FROZ"
        l.frozen.set_leaf(alice, marca);
        assert!(l.is_frozen(alice), "precondicion: congelada");

        let mapa = l.migrate_to_salted_positions().expect("migra");
        let (_, nuevo) = mapa.iter().find(|(v, _)| *v == alice).copied().expect("en el mapa");
        assert!(l.is_frozen(nuevo), "la marca debe seguir al indice nuevo");
        assert!(!l.is_frozen(alice), "el indice viejo queda libre");
    }

    /// **Idempotencia + registro**: la entrada Migration consta con las
    /// raíces correctas, y la segunda ejecución se RECHAZA (re-envolver
    /// hojas ya envueltas sería corrupción silenciosa).
    #[test]
    fn la_segunda_migracion_se_rechaza_y_el_log_la_registra() {
        let mut l = new_layer();
        open_and_fund(&mut l, SK_ALICE, 1_000);
        let root_antes = l.accounts.root();
        assert!(!l.has_migration_entry());

        l.migrate_to_salted_positions().expect("primera");
        assert!(l.has_migration_entry(), "la entrada Migration debe constar");
        let e = l.log.entries().iter()
            .find(|e| matches!(e.kind, crate::log::OpKind::Migration))
            .expect("entrada Migration");
        assert_eq!(e.root_old, root_antes, "root_old = cuentas pre-migracion");
        assert_eq!(e.root_new, l.accounts.root(), "root_new = cuentas post");

        assert!(l.migrate_to_salted_positions().is_err(),
                "la segunda migracion debe RECHAZARSE");
    }

    /// **Colisión → sondeo determinista** (heredado del intento de 49-B,
    /// §137: «sobrevive del intento»). Dos identidades con el mismo
    /// primer elemento: la de índice viejo menor toma la base, la otra
    /// sondea al siguiente.
    #[test]
    fn colision_de_posicion_sondea_al_siguiente() {
        use stark_experiment::circuit_settlement::native_leaf;
        let mut l = new_layer();
        let x = BaseElement::new(0xC011);
        let vid: Digest = [BaseElement::new(9); 4];
        let salt: Digest = [BaseElement::new(7); 4];
        for (i, limb) in [(0u64, 1u64), (1, 2)] {
            let id: Digest = [x, BaseElement::new(limb), BaseElement::new(limb), BaseElement::new(limb)];
            l.accounts.set_leaf(i, native_leaf(id, BaseElement::ZERO, BaseElement::ZERO));
            l.records.insert(i, AccountRecord {
                public_id: id, balance: 0, nonce: BaseElement::ZERO,
                view_id: vid, leaf_salt: salt,
            });
        }
        let mapa = l.migrate_to_salted_positions().expect("migra");
        let lookup: std::collections::HashMap<_, _> = mapa.into_iter().collect();
        let base = x.as_int() % l.accounts.capacity();
        assert_eq!(lookup[&0], base, "el primero (orden viejo) toma la base");
        assert_eq!(lookup[&1], base + 1, "el segundo sondea al siguiente slot");
    }

    fn temp_path(name: &str) -> String {
        let s = format!(
            "{}/zkssl-mig-{}-{}",
            std::env::temp_dir().display(),
            name,
            std::process::id()
        );
        let _ = std::fs::remove_dir_all(&s);
        s
    }

    /// **Sub-frente sled**: una capa PERSISTENTE migrada REINICIA. Purga
    /// de claves stale, cuentas en sus indices nuevos, la congelacion en
    /// el nuevo, y la verificacion de raices de la carga (que reconstruye
    /// hoja envuelta + frozen-32 al ver el marcador) en verde. La
    /// re-migracion del ledger reabierto se RECHAZA por el marcador.
    #[test]
    fn una_capa_persistente_migrada_reinicia() {
        let path = temp_path("migra-reinicio");
        let (n_cuentas, alice_new, supply) = {
            let mut l = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            ).expect("abrir");
            let alice = open_and_fund(&mut l, SK_ALICE, 750_000);
            open_and_fund(&mut l, SK_BOB, 250_000);
            let marca: Digest = [BaseElement::new(0x46524F5A); 4];
            l.frozen.set_leaf(alice, marca);
            l.commit(&[alice], None).expect("persistir la marca");
            let mapa = l.migrate_to_salted_positions().expect("migra");
            let (_, n) = mapa.iter().find(|(v, _)| *v == alice).copied().expect("alice");
            (mapa.len(), n, l.total_supply())
        };
        let mut l = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        ).expect("REABRIR: aqui muere si la carga no honra el marcador");
        assert_eq!(l.records.len(), n_cuentas, "claves stale: hay duplicados");
        assert!(l.records.contains_key(&alice_new), "alice en su indice nuevo");
        assert!(l.is_frozen(alice_new), "la congelacion reinicia en el indice NUEVO");
        assert_eq!(l.total_supply(), supply);
        assert!(l.migrate_to_salted_positions().is_err(),
                "re-migrar el ledger reabierto debe RECHAZARSE (marcador)");
        let _ = std::fs::remove_dir_all(&path);
    }
}
