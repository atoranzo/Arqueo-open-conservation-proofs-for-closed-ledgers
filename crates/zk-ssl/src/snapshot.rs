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
//! formato v9 -- lo que escribe export_snapshot (§802; hasta el §802 esta linea decia v7 y
//!               describia la v8)
//!
//! fichero = marca (1 B) + cuerpo
//!
//!   marca 0x00   el cuerpo va EN CLARO
//!   marca 0x01   el cuerpo va SELLADO con la clave de la version 1 del KDF (solo se lee, §702)
//!   marca 0x02   (§702, RFC-0001) 29 B en claro, la cabecera del KDF del libro -version 2,
//!                coste y sal-, y el cuerpo SELLADO con la clave que deriva. Con 0x01 y 0x02
//!                los desplazamientos de abajo son DEL CUERPO, una vez abierto el sello. ⚠️ §821: «del libro», o, si la exporta una capa importada con clave, la de su instantanea, o una de sal nueva si venia de la v1 (la 121; `conservar_la_clave`, al final).
//!
//! cuerpo (desplazamiento desde su byte 0):
//!
//!     0  MAGIC              8 B   "ZKSSL9\0\0"  (MAGIC_V9)
//!     8  custodian_root    32 B
//!    40  governance_root   32 B
//!    72  state_root        32 B   ← para verificar
//!   104  frozen_root       32 B   ← para verificar
//!   136  cons_root         32 B   ← para verificar (v8, RFC-0006 E1, §413)
//!   168  regulatory_limit   8 B
//!   176  max_supply         8 B
//!   184  max_accounts       8 B
//!   192  total_supply       8 B
//!   200  recovery_count     8 B
//!   208  freeze_count       8 B
//!   216  gov_change_count   8 B
//!   224  next_index         8 B
//!   232  n_accounts         8 B
//!   240  n_frozen           8 B
//!   248  n_cons             8 B   (v8)
//!   256  n_log              8 B   bit 63 = LOG_SECCION_V2 (§281):
//!                                 la sección del registro es era 2.
//!                                 bits 0..62 = número de entradas
//!   264  ── cuentas ──      n_accounts x (8 + 112) B
//!                           índice 8 | public_id 32 | saldo 8 |
//!                           nonce 8 | view_id 32 | leaf_salt 32
//!        ── congeladas ──   n_frozen x (8 + 32) B
//!        ── consumos ──     n_cons x (8 + 32) B   posicion | consumo (v8)
//!        ── registro ──     n_log x (2 B longitud u16, luego longitud B)
//!                           longitud vale 137, o 169 si la entrada
//!                           lleva compromiso
//!        ── retiradas ──    n_retir 8 B, luego n_retir x 32 B (v9, §802):
//!                           las identidades que retiraron las
//!                           recuperaciones, en su orden
//! ```
//!
//! Esto es lo que `export_snapshot` **escribe**: v9 (⚠️ §802: hasta el §802, v8). Lo que se
//! **lee** es v3..v9 — la tabla de mágicos, con lo que cambia en cada versión,
//! vive pegada a las constantes `MAGIC`..`MAGIC_V9`, más abajo en este fichero. La cola
//! de la v9 va DETRAS del registro: los desplazamientos de la cabecera no se mueven.
//!
//! ## ⚠️ Lo que NO es
//!
//! - **No hay replicación en vivo.** Es una copia puntual; los cambios
//!   posteriores no se propagan. Replicar en caliente exige red y
//!   coordinación, que es el problema de sistemas distribuidos que este
//!   proyecto no aborda.
//! - **Va cifrada si la capa tiene clave**, con el mismo cifrado
//!   autenticado que el ledger y la cabecera de su KDF (§702). Sin clave va
//!   en claro, y entonces quien tenga el fichero **ve todos los saldos**. ⚠️ §817: con clave sólo se importa una cifrada (`rehusar_en_claro`, al final). ⚠️ §865: ya no está al final; detrás va lo del §819 (`avisar_del_barrido`, `ruta_del_borrado`), lo del §820 (`escribir_entera`) y lo del §821 (`conservar_la_clave`).
//! - **⚠️ §802: en claro, las retiradas no van atadas a nada.** La cola de la
//!   v9 no lleva raíz, y `SnapshotInfo` solo trae `state_root`, el
//!   suministro y los recuentos: una cola cambiada importa, y la identidad
//!   que una recuperación retiró vuelve a abrir (medido). El resto del cuerpo
//!   lo atan `state_root` y la cadena del registro; la cola, solo el sellado
//!   de una instantánea cifrada (`SECURITY.md` §3.13; ⚠️ §817: y, hasta el §817, ni ese: con clave se importaba también una en claro; §3.16).
//! - **No hay copias incrementales.** Cada instantánea es completa.
//! - **No lleva los pagos en vuelo.** El formato (v9, como la v8) no tiene hueco
//!   para el árbol de pendientes ni su raíz: una copia de un ledger
//!   con dinero en tránsito lo perdería callando, y la verificación
//!   pasaría igual. Por eso `export_snapshot` **rehúsa** exportar
//!   mientras haya alguno (§359); llevar el pendiente con su raíz
//!   es la vía futura declarada.

use std::collections::HashMap;
use std::io::{Read, Write};
use winterfell::math::fields::f64::BaseElement;

use stark_experiment::native::native_leaf;
use stark_experiment::merkle::Digest;

use super::*;
use crate::store::{digest_from_bytes, digest_to_bytes, record_from_bytes_v3, record_to_bytes_v3};

/// **Versión 4 del formato.** La 3 incluía el árbol de nullificadores,
/// retirado con la vía de un paso (`AUDITORIA.md` §32); una copia v3
/// **sigue importándose**: sus nullificadores se verifican contra la
/// raíz que declara y se descartan.
///
/// La 2 no incluía el registro de transiciones, y restaurar perdía el
/// historial. La 1 no incluía las cuentas congeladas, y restaurar desde
/// ella levantaba todas las congelaciones. Cambiar la firma hace que una
/// copia antigua se rechace en vez de cargarse a medias.
const MAGIC: &[u8; 8] = b"ZKSSL4\0\0";
/// Firma de la versión anterior, aceptada SOLO al importar.
const MAGIC_V3: &[u8; 8] = b"ZKSSL3\0\0";
/// v5 (49-A): registros de cuenta de 80 B, con view_id. v4 y v3 (48 B, sin
/// view_id) se siguen importando con centinela.
const MAGIC_V5: &[u8; 8] = b"ZKSSL5\0\0";
/// v6 (B13/B14): registros de 112 B con leaf_salt. v5/v4/v3 (80/48) se
/// importan con salt centinela.
const MAGIC_V6: &[u8; 8] = b"ZKSSL6\0\0";
/// v7 (flip D4): mismo registro de 112 B, pero el árbol se reconstruye
/// con hoja ENVUELTA (`native_leaf_salted`) y frozen a 32. v6 y
/// anteriores se importan como mundo viejo (sin salt, frozen a 24) y se
/// migran después.
const MAGIC_V7: &[u8; 8] = b"ZKSSL7\0\0";
/// v8 (RFC-0006 E1, §413): el arbol de consumos publicados —`cons_root`
/// tras `frozen_root`, `n_cons` tras `n_frozen`, la seccion tras las
/// congeladas—. v7 y anteriores se importan con el conjunto vacio.
const MAGIC_V8: &[u8; 8] = b"ZKSSL8\0\0";
/// v9 (§802): la v8 byte a byte, mas una COLA tras el registro: `n_retir` (8 B LE) y
/// `n_retir` x 32 B, las identidades que retiraron las recuperaciones, en su orden. Al
/// importar se exige una por cada entrada Recovery del registro importado; v3..v8 no la
/// traen, y por eso solo importan si su registro no tiene ninguna.
const MAGIC_V9: &[u8; 8] = b"ZKSSL9\0\0";

/// Resumen de una instantánea, para registro y verificación externa.
#[derive(Clone, Debug)]
pub struct SnapshotInfo {
    pub accounts: u64,
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
/// ⚠️ §281 · **El bit alto de `n_log` versiona la sección del registro.**
/// Un stream concatenado no puede discriminar por longitud —el lector
/// no sabe dónde corta— así que la sección v2 prefija cada entrada con
/// su longitud (u16 LE) y lo anuncia en el propio contador: `n_log`
/// jamás usa ese bit como cuenta. Un fichero viejo (bit apagado) se lee
/// como siempre, a 137. La alternativa —marcas nuevas de fichero— se
/// descartó: había más de un escritor con marca y este bit toca UNO.
const LOG_SECCION_V2: u64 = 1 << 63;

const SNAPSHOT_PLAIN: u8 = 0x00;
/// Marca de instantánea cifrada con la clave de la versión 1 del KDF: desde el §702, solo se lee.
const SNAPSHOT_ENCRYPTED: u8 = 0x01;

impl SovereignLayer {
    /// Importa una instantánea cifrada, y sólo una cifrada (⚠️ §817: hasta el §817 también una en claro; `rehusar_en_claro`, al final). ⚠️ §865: ya no está al final; detrás va lo del §819 (`avisar_del_barrido`, `ruta_del_borrado`), lo del §820 (`escribir_entera`) y lo del §821 (`conservar_la_clave`).
    ///
    /// Existe aparte porque `import_snapshot` no tiene forma de conocer la clave: es una función
    /// asociada, no un método. §702: la fija al KDF de la instantánea (`abrir_sellada`, al final). ⚠️ §821: y la capa que devuelve la conserva, y se reexporta cifrada; hasta el §821 nacía sin clave y se reexportaba en claro (la 121; `conservar_la_clave`, al final). ⚠️ §865: `abrir_sellada` ya no está al final; detrás van `barrer_descifrada` (§812), `rehusar_en_claro` (§817), lo del §819, lo del §820 (`escribir_entera`) y `conservar_la_clave` (§821), que sí es lo último.
    pub fn import_snapshot_with_key(
        path: &str,
        key: &crate::crypto::LedgerKey,
    ) -> Result<Self, LayerError> {
        barrer_descifrada(path)?; // ⚠️ §812: el claro que dejaba una importacion muerta (al final) ⚠️ §865: ya no; detrás van `rehusar_en_claro` (§817) y lo del §819, el §820 y el §821
        let mut buf = Vec::new();
        std::fs::File::open(path)
            .map_err(io_err)?
            .read_to_end(&mut buf)
            .map_err(io_err)?;

        let (plano, fijada) = match buf.first() { // ⚠️ §821: y la clave fijada a su KDF
            Some(&SNAPSHOT_ENCRYPTED) | Some(&SNAPSHOT_ENCRYPTED_V2) => abrir_sellada(&buf, key)?,
            Some(&SNAPSHOT_PLAIN) => return Err(rehusar_en_claro()), // ⚠️ §817: la importaba sin mirar la clave
            _ => {
                return Err(LayerError::Store(crate::store::StoreError::Malformed(
                    "instantanea sin marca de cifrado".into(),
                )))
            }
        };

        // ⚠️ §774: el texto en claro se importa DESDE MEMORIA. Hasta el §774 se escribia en
        //    `{path}.tmp-descifrada`, junto a la instantanea cifrada, y se borraba despues: el
        //    estado entero en claro en disco durante la importacion, y para siempre si el proceso
        //    moria entre medias.
        conservar_la_clave(Self::desde_instantanea_en_claro(plano)?, fijada, key) // ⚠️ §821: hasta el §821, la capa salía sin clave (la 121)
    }

    /// Exporta el estado completo a un fichero, entero o nada (⚠️ §820: `escribir_entera`, al final). ⚠️ §865: ya no está al final; detrás de lo del §820 va `conservar_la_clave` (§821).
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
        // (b') del §359: el formato (v9, como la v8) NO lleva el pendiente -- ni
        // arbol, ni raiz, ni meta. Exportar con pagos en vuelo fabricaria
        // una copia que MIENTE: restaura sin ellos y la verificacion pasa
        // igual. Se rehusa ANTES de escribir un solo byte; la via real es
        // una version futura que lleve el pendiente con su raiz.
        let en_vuelo = self.total_pending();
        if en_vuelo > 0 || !self.pending_amounts.is_empty() || !self.reserved_pending.is_empty() {
            return Err(LayerError::VerificationFailed(format!(
                "instantanea: {en_vuelo} en transito en {} posiciones; el formato v9 no lleva el pendiente y exportarlo lo perderia callando",
                self.pending_amounts.len() + self.reserved_pending.len()
            )));
        }
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(MAGIC_V9);
        out.extend_from_slice(&digest_to_bytes(&self.custodian_set_root));
        out.extend_from_slice(&digest_to_bytes(&self.governance_set_root));
        out.extend_from_slice(&digest_to_bytes(&self.accounts.root()));
        out.extend_from_slice(&digest_to_bytes(&self.frozen.root()));
        out.extend_from_slice(&digest_to_bytes(&self.consumos.root()));
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
        let mut frozen: Vec<(u64, Digest)> = self.frozen.occupied();
        frozen.sort_by_key(|(p, _)| *p);
        let mut cons: Vec<(u64, Digest)> = self.consumos_orden.clone();
        cons.sort_by_key(|(p, _)| *p);

        out.extend_from_slice(&(accounts.len() as u64).to_le_bytes());
        out.extend_from_slice(&(frozen.len() as u64).to_le_bytes());
        out.extend_from_slice(&(cons.len() as u64).to_le_bytes());
        out.extend_from_slice(&((self.log.len() as u64) | LOG_SECCION_V2).to_le_bytes());

        for (index, r) in &accounts {
            out.extend_from_slice(&index.to_le_bytes());
            out.extend_from_slice(&record_to_bytes_v3(&r.public_id, r.balance, r.nonce, &r.view_id, &r.leaf_salt));
        }
        for (index, leaf) in &frozen {
            out.extend_from_slice(&index.to_le_bytes());
            out.extend_from_slice(&digest_to_bytes(leaf));
        }
        for (pos, c) in &cons {
            out.extend_from_slice(&pos.to_le_bytes());
            out.extend_from_slice(&digest_to_bytes(c));
        }
        // El registro entero. Sin el, restaurar perderia el historial y
        // la capa restaurada encadenaria desde la nada.
        for e in self.log.entries() {
            let b = crate::store::log_entry_to_bytes(e);
            out.extend_from_slice(&(b.len() as u16).to_le_bytes());
            out.extend_from_slice(&b);
        }
        // §802 (v9): la cola de las identidades retiradas, en su orden. Va sin raiz: con
        // clave, el cuerpo entero es un sellado autenticado; en claro no la ata nada (ver la
        // doc del modulo).
        out.extend_from_slice(&(self.retiradas.len() as u64).to_le_bytes());
        for id in &self.retiradas {
            out.extend_from_slice(&digest_to_bytes(id));
        }

        // Cifrado con la misma clave que el ledger, si la hay. Se antepone
        // un byte de marca para que la importacion sepa que tiene delante
        // sin adivinar.
        let bytes = match &self.key {
            Some(k) => {
                let mut v = marca_y_cabecera(k);
                v.extend_from_slice(&k.seal(&out).map_err(LayerError::Store)?);
                v
            }
            None => {
                let mut v = vec![SNAPSHOT_PLAIN];
                v.extend_from_slice(&out);
                v
            }
        };

        // ⚠️ §820: hasta el §820, `File::create(path)`, que truncaba en sitio la anterior (la 120 del
        // `BACKLOG.md`). Entera o nada: un temporal al lado y un cambio de nombre (al final). ⚠️ §865: ya no; detrás de lo del §820 va `conservar_la_clave` (§821).
        escribir_entera(path, &bytes)?;

        Ok(SnapshotInfo {
            accounts: accounts.len() as u64,
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
            Some(&SNAPSHOT_ENCRYPTED) | Some(&SNAPSHOT_ENCRYPTED_V2) => {
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
        Self::desde_instantanea_en_claro(buf)
    }

    /// ⚠️ §774 · **El parseo de una instantanea en claro, sin su marca, desde bytes.** UN
    /// productor para las dos importaciones: la que lee el fichero en claro y la que lo descifra
    /// en memoria; ninguna escribe nada.
    fn desde_instantanea_en_claro(buf: Vec<u8>) -> Result<Self, LayerError> {
        let mut cursor = 0usize;
        let mut take = |n: usize, what: &str| -> Result<&[u8], LayerError> {
            if cursor + n > buf.len() {
                return Err(malformed(&format!("instantanea truncada en '{what}'")));
            }
            let s = &buf[cursor..cursor + n];
            cursor += n;
            Ok(s)
        };

        // Una copia v3 —anterior a la retirada del arbol de
        // nullificadores— sigue siendo importable: sus nullificadores se
        // verifican contra la raiz que declara y se descartan despues.
        // ⚠️ §802: se ESCRIBE v9; se LEEN v3..v9. El reparto no se enumera aqui:
        // cada magico fija su quintupla (legacy_v3, rec_len, salted, con_cons, con_retir)
        // justo debajo, y esa es la unica lista que manda.
        let (legacy_v3, rec_len, salted, con_cons, con_retir) = {
            let magic = take(8, "magic")?;
            if magic == MAGIC_V3 {
                (true, 48usize, false, false, false)
            } else if magic == MAGIC {
                (false, 48usize, false, false, false)
            } else if magic == MAGIC_V5 {
                (false, 80usize, false, false, false)
            } else if magic == MAGIC_V6 {
                (false, 112usize, false, false, false)
            } else if magic == MAGIC_V7 {
                // Mundo nuevo (flip D4): hoja envuelta y frozen a 32.
                (false, 112usize, true, false, false)
            } else if magic == MAGIC_V8 {
                // v8 (RFC-0006 E1): como v7, mas el arbol de consumos.
                (false, 112usize, true, true, false)
            } else if magic == MAGIC_V9 {
                // v9 (§802): como v8, mas la cola de las identidades retiradas.
                (false, 112usize, true, true, true)
            } else {
                return Err(malformed(
                    "cabecera desconocida: no es una instantanea ZK-SSL",
                ));
            }
        };
        let custodian_set_root = digest_from_bytes(take(32, "custodian_root")?)?;
        let governance_set_root = digest_from_bytes(take(32, "governance_root")?)?;
        let declared_state = digest_from_bytes(take(32, "state_root")?)?;
        let declared_null = if legacy_v3 {
            Some(digest_from_bytes(take(32, "nullifier_root")?)?)
        } else {
            None
        };
        let declared_frozen = digest_from_bytes(take(32, "frozen_root")?)?;
        let declared_cons = if con_cons {
            Some(digest_from_bytes(take(32, "cons_root")?)?)
        } else {
            None
        };

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
        let n_nullifiers = if legacy_v3 { u64_at("n_nullifiers")? } else { 0 };
        let n_frozen = u64_at("n_frozen")?;
        let n_cons = if con_cons { u64_at("n_cons")? } else { 0 };
        let n_log = u64_at("n_log")?;

        // ⚠️ §718, la canonicidad: el tope de emision, el suministro y cada saldo, por
        // debajo del tope de la capa (§668) y asi muy por debajo de `p`, ANTES de componer una
        // hoja. Sin esto, `saldo + p` daba la misma raiz de cuentas, y con `suministro + p`
        // coordinado la instantanea importaba (medido). `open` ya exigia el tope de emision;
        // la instantanea no.
        if max_supply > crate::MAX_SUMINISTRO {
            return Err(LayerError::SupplyCapExceeded {
                cap: crate::MAX_SUMINISTRO,
                would_be: max_supply,
            });
        }
        if total_supply > crate::MAX_SUMINISTRO {
            return Err(LayerError::Store(crate::store::StoreError::IntegrityFailure {
                what: "suministro de la instantanea fuera del tope de la capa",
            }));
        }
        // Lo mismo para lo que va a `paramsDigest` y a la cabeza sin raiz que lo cubra: el limite y
        // el tope de cuentas, canonicos (los comprueba `open`), y el censo de cuentas, dentro del
        // arbol (lo comprueba `load`). Con `+ p`, la capa importada firmaba el mismo digest con
        // otra regla, o servia una cabeza que el kit rechaza.
        let fuera = |what: &'static str| {
            LayerError::Store(crate::store::StoreError::IntegrityFailure { what })
        };
        if zk_ssl_hash::u64_canonico(regulatory_limit).is_err() {
            return Err(fuera("limite regulatorio de la instantanea no canonico"));
        }
        if zk_ssl_hash::u64_canonico(max_accounts).is_err() {
            return Err(fuera("tope de cuentas de la instantanea no canonico"));
        }
        if next_index > SparseTree::new().capacity() {
            return Err(fuera("censo de cuentas de la instantanea fuera del arbol"));
        }

        let mut layer = Self {
            custodian_uses: 0,
            max_custodian_uses: crate::DEFAULT_MAX_CUSTODIAN_USES,
            accounts: SparseTree::new(),
            pending: SparseTree::new(),
            next_pending: 0,
            reserved_pending: std::collections::BTreeSet::new(),
            pending_amounts: HashMap::new(),
            pending_meta: HashMap::new(),
            pending_meta_tree: crate::sparse_tree::SparseTree::new(),
            refund_ttl: crate::DEFAULT_REFUND_TTL,
            records: HashMap::new(),
            next_index,
            custodian_set_root,
            governance_set_root,
            governance_change_count,
            frozen: SparseTree::with_depth(if salted {
                FROZEN_DEPTH
            } else {
                crate::migration::FROZEN_DEPTH_PRE
            }),
            freeze_count,
            log: TransitionLog::new(),
            log_persisted: 0,
            consumos: SparseTree::with_depth(crate::consumo::CONS_DEPTH),
            consumos_orden: Vec::new(),
            cons_persisted: 0,
            total_supply,
            recovery_count,
            regulatory_limit,
            max_supply,
            max_accounts,
            options: proof_options(),
            db: None,
            // Una capa importada arranca en memoria y sin cifrar: quien
            // la persista decidira con que clave. ⚠️ §821: la de `import_snapshot_with_key` ya no: sale con la de la instantanea (`conservar_la_clave`, al final).
            key: None,
            identidades: std::collections::HashSet::new(),
            retiradas: Vec::new(),
            retir_cabeza: [BaseElement::ZERO; 4],
            retir_persisted: 0,
        };

        for _ in 0..n_accounts {
            let index = u64::from_le_bytes(
                take(8, "indice de cuenta")?
                    .try_into()
                    .map_err(|_| malformed("indice de cuenta"))?,
            );
            let (public_id, balance, nonce, view_id, leaf_salt) =
                record_from_bytes_v3(take(rec_len, "registro")?)?;
            if balance > crate::MAX_SUMINISTRO {
                return Err(LayerError::Store(crate::store::StoreError::IntegrityFailure {
                    what: "saldo de la instantanea fuera del tope de la capa",
                }));
            }
            // ⚠️ Ninguna cuenta en el indice del centinela: en el campo es el de la emision, y una
            // capa arrancada de aqui tendria, en cuanto esa cuenta pagara, un `pmeta:` que `load`
            // abre y en el que la reescritura pago -> centinela no se ve (medido: importaba). Lo
            // mismo que `open_with_id` y la migracion, que no colocan nada en el.
            if index == crate::INDICE_DEL_CENTINELA {
                return Err(LayerError::Store(crate::store::StoreError::IntegrityFailure {
                    what: "cuenta de la instantanea en el indice del centinela",
                }));
            }
            layer.accounts.set_leaf(
                index,
                if salted {
                    stark_experiment::native::native_leaf_salted(
                        public_id, BaseElement::new(balance), nonce, leaf_salt,
                    )
                } else {
                    native_leaf(public_id, BaseElement::new(balance), nonce)
                },
            );
            layer.records.insert(
                index,
                AccountRecord {
                    public_id,
                    balance,
                    nonce,
                    view_id,
                    leaf_salt,
                },
            );
        }
        // Nullificadores de una copia v3: se reconstruyen SOLO para
        // verificarlos contra la raiz declarada, y se descartan. El arbol
        // fue retirado con la via de un paso (`AUDITORIA.md` §32).
        let mut legacy_null = SparseTree::new();
        for _ in 0..n_nullifiers {
            let position = u64::from_le_bytes(
                take(8, "posicion de nullifier")?
                    .try_into()
                    .map_err(|_| malformed("posicion de nullifier"))?,
            );
            let n = digest_from_bytes(take(32, "nullifier")?)?;
            legacy_null.set_leaf(position, n);
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

        for _ in 0..n_cons {
            let pos = u64::from_le_bytes(
                take(8, "posicion de consumo")?
                    .try_into()
                    .map_err(|_| malformed("posicion de consumo"))?,
            );
            let c = digest_from_bytes(take(32, "consumo")?)?;
            layer.consumos.set_leaf(pos, c);
            layer.consumos_orden.push((pos, c));
        }

        // §281: el bit alto de n_log dice la era de la SECCION. Se
        // separa ANTES de reservar: un contador con el bit puesto no es
        // una cuenta.
        let seccion_v2 = n_log & LOG_SECCION_V2 != 0;
        let n_log = n_log & !LOG_SECCION_V2;
        let mut entradas = Vec::with_capacity(n_log as usize);
        for _ in 0..n_log {
            if seccion_v2 {
                let len = u16::from_le_bytes(
                    take(2, "longitud de entrada del registro")?
                        .try_into()
                        .map_err(|_| malformed("longitud de entrada del registro"))?,
                ) as usize;
                entradas.push(crate::store::log_entry_from_bytes(take(len, "entrada del registro")?)?);
            } else {
                entradas.push(crate::store::log_entry_from_bytes(take(137, "entrada del registro")?)?);
            }
        }
        layer.log = TransitionLog::from_entries(entradas);
        // §802 (v9): la cola de las identidades retiradas. Sin reservar con su cuenta: una
        // cuenta falsa no reserva memoria, se queda sin bytes.
        let mut retiradas: Vec<Digest> = Vec::new();
        if con_retir {
            let n_retir = u64::from_le_bytes(
                take(8, "n_retir")?.try_into().map_err(|_| malformed("n_retir"))?,
            );
            for _ in 0..n_retir {
                retiradas.push(digest_from_bytes(take(32, "identidad retirada")?)?);
            }
        }

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

        // §394: la cuota de custodios se REPONE del registro. Antes se importaba en
        // cero, asi que exportar e importar le devolvia el cupo a un conjunto
        // agotado; y con la puerta del §394 puesta, un libro importado legitimo no
        // abriria. Mismo productor que `load`: las dos no pueden divergir. El TOPE
        // sigue viniendo del defecto -es politica y no viaja-, y va declarado.
        layer.custodian_uses = layer.log.usos_custodiados();
        // §802: una retirada por cada recuperacion del registro VERIFICADO -no de la cabecera,
        // que no se cruza-, y el indice de identidades. Una v3..v8 trae la lista vacia.
        let n_rec = layer
            .log
            .entries()
            .iter()
            .filter(|e| matches!(e.kind, OpKind::Recovery))
            .count() as u64;
        layer.adoptar_retiradas(retiradas, n_rec, "identidades retiradas de la instantanea")?;

        if layer.frozen.root() != declared_frozen {
            return Err(LayerError::Store(
                crate::store::StoreError::IntegrityFailure {
                    what: "arbol de congelados de la instantanea",
                },
            ));
        }
        if let Some(declared) = declared_cons {
            if layer.consumos.root() != declared {
                return Err(LayerError::Store(
                    crate::store::StoreError::IntegrityFailure {
                        what: "arbol de consumos de la instantanea",
                    },
                ));
            }
        }
        if let Some(declared) = declared_null {
            if legacy_null.root() != declared {
                return Err(LayerError::Store(
                    crate::store::StoreError::IntegrityFailure {
                        what: "arbol de nullifiers de la instantanea (v3)",
                    },
                ));
            }
        }

        // --- CONSERVACION DEL DINERO (gemela de `persistence::load`) ---
        //
        // `state_root` cubre los saldos: la hoja lleva el balance. El
        // SUMINISTRO no lo cubre ninguna raiz, y hasta aqui se creia.
        //
        // El formato v3..v7 NO transporta pendientes, asi que aqui la
        // invariante se reduce a sum(saldos) == suministro. Es cierto porque
        // el §359 hizo que `export_snapshot` REHUSE con pagos en vuelo.
        //
        // ROTURA DECLARADA: una instantanea ANTERIOR al §359, tomada con
        // dinero en transito, tiene sum(saldos) < suministro y a partir de
        // aqui NO importa. Es fail-closed: ante duda se para, no se sigue
        // por compatibilidad.
        let saldos: u128 = layer.records.values().map(|r| r.balance as u128).sum();
        if saldos != layer.total_supply as u128 {
            return Err(LayerError::Store(
                crate::store::StoreError::IntegrityFailure {
                    what: "conservacion del suministro de la instantanea",
                },
            ));
        }

        Ok(layer)
    }
}

#[cfg(test)]
mod tests {
    // Estos tests ejercitan la via ANTIGUA a proposito: sigue siendo la
    // unica para `mint` y `mint_pending`, y sus propiedades hay que
    // comprobarlas igual. El aviso de obsolescencia se silencia aqui, no en
    // la definicion, para que siga saltando en codigo nuevo.

    use crate::tests_support::*;
    use crate::*;

    fn temp_file(name: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!("zkssl_snap_{}_{}.bin", name, std::process::id()));
        let s = p.to_str().unwrap().to_string();
        let _ = std::fs::remove_file(&s);
        s
    }

    /// Paso 3 de 49-A: el snapshot v5 PERSISTE el view_id. Antes de este
    /// paso el WRITE escribia 48 B y el view_id se perdia en cada copia,
    /// reapareciendo como centinela al reimportar. El test prueba que
    /// ahora cruza el disco intacto.
    /// B13/B14: el snapshot v6 PERSISTE el leaf_salt. Antes de v6, el
    /// WRITE escribia 80 B y el salt se perdia. Cruza el disco intacto.
    #[test]
    fn snapshot_v6_preserva_leaf_salt() {
        use stark_experiment::native::derive_leaf_salt;
        use winterfell::math::fields::f64::BaseElement;
        const SK: u64 = 0x5A17;

        let mut l1 = new_layer();
        let idx = open_and_fund(&mut l1, SK, 1_000_000);
        let salt = l1.stored_leaf_salt(idx);
        // Salt REAL derivado de la clave, no centinela.
        assert_eq!(salt, Some(derive_leaf_salt(BaseElement::new(SK))));
        assert_ne!(salt, Some(crate::store::LEAF_SALT_LEGACY));

        let path = temp_file("v6_leaf_salt");
        l1.export_snapshot(&path).expect("export v6");
        let l2 = SovereignLayer::import_snapshot(&path).expect("import v6");
        let _ = std::fs::remove_file(&path);

        assert_eq!(l2.stored_leaf_salt(idx), salt,
                   "el leaf_salt no sobrevivio el round-trip de snapshot");
        // y el view_id tambien sigue (v6 lleva ambos).
        assert_eq!(l2.stored_view_id(idx), l1.stored_view_id(idx),
                   "v6 debe preservar view_id ademas de leaf_salt");
    }

    /// (b') del §359: el export FALLA CERRADO con pagos en vuelo --
    /// el formato (v7) no lleva el pendiente y una copia lo perderia
    /// callando, con la verificacion pasando igual. Sin transito, exporta.
    #[test]
    fn un_export_con_pagos_en_vuelo_rebota() {
        let mut l = new_layer();
        let _ = open_and_fund(&mut l, 0xE59A, 1_000_000);
        // Un pago en vuelo, plantado en la boca exacta que el gate mira.
        l.pending_amounts.insert(0, 300_000);
        assert!(!l.pending_amounts.is_empty(), "premisa: hay transito");
        let path = temp_file("export_con_vuelo");
        let r = l.export_snapshot(&path);
        assert!(r.is_err(), "con pagos en vuelo el export rehusa");
        assert!(!std::path::Path::new(&path).exists(),
                "y no deja fichero a medias");
        // El transito termina: el export vuelve.
        l.pending_amounts.clear();
        l.export_snapshot(&path).expect("sin transito, exporta");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn snapshot_v5_preserva_view_id() {
        use stark_experiment::native::view_id_of;
        use winterfell::math::fields::f64::BaseElement;
        const SK: u64 = 0xA11CE;

        let mut layer1 = new_layer();
        let idx = open_and_fund(&mut layer1, SK, 1_000_000);
        let vid = layer1.stored_view_id(idx);
        // Era un view_id REAL, no el centinela.
        assert_eq!(vid, Some(view_id_of(BaseElement::new(SK))));
        assert_ne!(vid, Some(crate::store::VIEW_ID_LEGACY));

        let path = temp_file("v5_view_id");
        layer1.export_snapshot(&path).expect("export v5");
        let layer2 = SovereignLayer::import_snapshot(&path).expect("import v5");
        let _ = std::fs::remove_file(&path);

        // El view_id cruzo el disco: reimportado == original.
        assert_eq!(layer2.stored_view_id(idx), vid,
                   "el view_id no sobrevivio el round-trip de snapshot");
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

    /// Construye una instantánea **v3** mínima a mano: cero cuentas, UN
    /// nullificador, cero congeladas, registro vacío. Con `honest` la
    /// raíz de nullificadores declarada es la verdadera; sin él, falsa.
    ///
    /// Existe porque el código actual ya no puede EXPORTAR v3: la única
    /// forma de probar que una copia antigua sigue importándose es
    /// fabricar sus bytes con el formato documentado.
    fn v3_snapshot_bytes(honest: bool) -> Vec<u8> {
        let mut null_tree = SparseTree::new();
        let leaf = crate::store::digest_from_bytes(&[1u8; 32]).expect("digest");
        null_tree.set_leaf(7, leaf);

        let declared_null = if honest {
            null_tree.root()
        } else {
            crate::store::digest_from_bytes(&[2u8; 32]).expect("digest")
        };

        let mut out: Vec<u8> = vec![0x00]; // marca: sin cifrar
        out.extend_from_slice(b"ZKSSL3\0\0");
        out.extend_from_slice(&crate::store::digest_to_bytes(&custodian_root()));
        out.extend_from_slice(&crate::store::digest_to_bytes(&governance_root()));
        out.extend_from_slice(&crate::store::digest_to_bytes(&SparseTree::new().root()));
        out.extend_from_slice(&crate::store::digest_to_bytes(&declared_null));
        out.extend_from_slice(&crate::store::digest_to_bytes(
            &SparseTree::with_depth(crate::migration::FROZEN_DEPTH_PRE).root(),
        ));
        out.extend_from_slice(&LIMIT.to_le_bytes());
        out.extend_from_slice(&MAX_SUPPLY.to_le_bytes());
        out.extend_from_slice(&MAX_ACCOUNTS.to_le_bytes());
        out.extend_from_slice(&0u64.to_le_bytes()); // total_supply
        out.extend_from_slice(&0u64.to_le_bytes()); // recovery_count
        out.extend_from_slice(&0u64.to_le_bytes()); // freeze_count
        out.extend_from_slice(&0u64.to_le_bytes()); // gov_change_count
        out.extend_from_slice(&0u64.to_le_bytes()); // next_index
        out.extend_from_slice(&0u64.to_le_bytes()); // n_accounts
        out.extend_from_slice(&1u64.to_le_bytes()); // n_nullifiers
        out.extend_from_slice(&0u64.to_le_bytes()); // n_frozen
        out.extend_from_slice(&0u64.to_le_bytes()); // n_log
        out.extend_from_slice(&7u64.to_le_bytes()); // posicion del nullifier
        out.extend_from_slice(&crate::store::digest_to_bytes(&leaf));
        out
    }

    /// **UNA COPIA v3 SIGUE SIENDO IMPORTABLE.**
    ///
    /// El formato prometió poder leerse dentro de diez años. La v4 retiró
    /// el árbol de nullificadores; una v3 se importa igual: sus
    /// nullificadores se verifican contra la raíz que declara y se
    /// descartan después.
    #[test]
    fn a_v3_snapshot_with_nullifiers_imports_verified() {
        let file = temp_file("v3_ok");
        std::fs::write(&file, v3_snapshot_bytes(true)).expect("escribir v3");
        let restored = SovereignLayer::import_snapshot(&file).expect("importar v3");
        assert_eq!(restored.total_supply(), 0);
        assert_eq!(restored.state_root(), SparseTree::new().root());
        let _ = std::fs::remove_file(&file);
    }

    /// **Y UNA v3 MANIPULADA SE RECHAZA.** Descartar sin verificar sería
    /// restaurar en silencio — lo mismo que este módulo impide en v4.
    #[test]
    fn a_v3_snapshot_with_a_forged_nullifier_root_is_rejected() {
        let file = temp_file("v3_mal");
        std::fs::write(&file, v3_snapshot_bytes(false)).expect("escribir v3");
        // `map(|_| ())` descarta la capa: `SovereignLayer` no implementa
        // `Debug`, y aqui solo interesa formatear el error.
        let r = SovereignLayer::import_snapshot(&file).map(|_| ());
        assert!(
            r.is_err(),
            "una raiz de nullifiers falsa debe rechazarse: {r:?}"
        );
        let _ = std::fs::remove_file(&file);
    }

    /// **EL TEST CLAVE**: el estado sobrevive a la exportación y vuelta.
    #[test]
    fn a_snapshot_restores_the_full_state() {
        let file = temp_file("full");
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);
        two_phase_transfer(&mut layer, alice, SK_ALICE, bob, SK_BOB, 250_000, salt_de(0x51A1))
            .expect("transferencia en dos fases");

        let info = layer.export_snapshot(&file).expect("exportar");
        println!(
            "Instantanea: {} cuentas, {} bytes",
            info.accounts, info.bytes
        );

        let restored = SovereignLayer::import_snapshot(&file).expect("importar");
        assert_eq!(restored.balance_of(alice), Some(750_000));
        assert_eq!(restored.balance_of(bob), Some(300_000));
        assert_eq!(restored.total_supply(), 1_050_000);
        assert_eq!(restored.state_root(), layer.state_root());
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
        //
        // ⚠️ La constante codifica la GEOMETRIA del formato —v8: cinco
        // raices, cuatro contadores— y ya se ha quedado rancia TRES veces:
        // cuando el formato crecio (el registro, lo cuenta el test
        // siguiente), cuando encogio (la retirada del arbol de
        // nullificadores, `AUDITORIA.md` §36) y cuando volvio a crecer (la
        // raiz de consumos, RFC-0006 E1, §413: este test cayo en el canon
        // antes de moverla). Un cambio de formato hay que contrastarlo con
        // todo test que codifique desplazamientos, no solo con los que
        // nombran lo cambiado. ⚠️ §802: la v9 añade una cola DETRAS del
        // registro y la cabecera no se mueve: la constante sigue valiendo.
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

    /// **UNA INSTANTANEA CUYO SUMINISTRO NO CUADRA SE RECHAZA.**
    ///
    /// Gemela de la de arriba, por el OTRO termino. Manipular un saldo mueve
    /// `state_root` y lo caza la puerta de cuentas; manipular el SUMINISTRO
    /// no mueve ninguna raiz, y hasta el §379 se importaba en silencio.
    ///
    /// El formato v8 -y la v9, §802- NO transporta pendientes, asi que aqui la invariante se
    /// reduce a sum(saldos) == suministro; es cierto porque el §359 hizo
    /// que `export_snapshot` rehuse con pagos en vuelo.
    #[test]
    fn a_snapshot_with_a_lying_supply_is_rejected() {
        let file = temp_file("supply_mentiroso");
        let mut layer = new_layer();
        open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let info = layer.export_snapshot(&file).expect("exportar");

        // El desplazamiento del `total_supply` se DERIVA de lo que
        // `export_snapshot` escribe: marca 1 + magico 8 + CINCO raices de 32
        // (v8: `cons_root` tras `frozen_root`, §413) + limite 8 + max_supply 8
        // + max_accounts 8. La cabecera del modulo dice lo mismo (192 en el
        // cuerpo): dos productores del mismo numero.
        //
        // El test vecino declara que una constante asi ya se quedo rancia DOS
        // veces. Por eso esta se AUTOVERIFICA antes de usarse. ⚠️ §802: la v9 no
        // la mueve; su cola va detras del registro.
        const OFF: usize = 1 + 8 + 32 * 5 + 8 * 3;
        let mut bytes = std::fs::read(&file).expect("leer");
        let mut ocho = [0u8; 8];
        ocho.copy_from_slice(&bytes[OFF..OFF + 8]);
        let leido = u64::from_le_bytes(ocho);
        assert_eq!(
            leido, info.total_supply,
            "el desplazamiento del suministro en la v9 se ha movido: el formato cambio y esta constante no"
        );

        bytes[OFF..OFF + 8].copy_from_slice(&(info.total_supply + 1).to_le_bytes());
        std::fs::write(&file, &bytes).expect("escribir");

        // No se imprime el resultado: `SovereignLayer` NO implementa `Debug` a
        // proposito, porque lleva los saldos de todas las cuentas.
        let detectado = matches!(
            SovereignLayer::import_snapshot(&file),
            Err(LayerError::Store(StoreError::IntegrityFailure {
                what: "conservacion del suministro de la instantanea"
            }))
        );
        assert!(
            detectado,
            "CRITICO: una instantanea cuyo suministro no cuadra con sus saldos debe rechazarse ANTES de operar"
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

        set_frozen_delegated(&mut layer, alice, true);

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

    /// §394: una instantanea NO RENUEVA la cuota de custodios. Antes de este sello
    /// `import_snapshot` la ponia en cero, asi que exportar e importar le devolvia
    /// el cupo a un conjunto agotado. Ahora se repone del registro, que si viaja.
    ///
    /// La cuota de partida se DERIVA, no se teclea: fundir tambien consume cupo
    /// (`apply_mint_delegated` llama a `consume_custodian_use`), asi que cuantos
    /// usos lleva este ledger lo dice el propio ledger. El `> 0` es la prueba de
    /// vida: sin ella, un cero a los dos lados pasaria por verde.
    #[test]
    fn una_instantanea_no_renueva_la_cuota_de_custodios() {
        let file = temp_file("cuota");
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        set_frozen_delegated(&mut layer, alice, true);
        let antes = layer.custodian_uses();
        assert!(antes > 0, "las operaciones custodiadas consumieron cupo");

        layer.export_snapshot(&file).expect("exportar");
        let restored = SovereignLayer::import_snapshot(&file).expect("importar");
        assert_eq!(
            restored.custodian_uses(),
            antes,
            "CRITICO: importar no debe devolver cupo a un conjunto de custodios"
        );
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
        two_phase_transfer(&mut layer, alice, SK_ALICE, bob, SK_BOB, 1000, salt_de(0x51A2))
            .expect("transferencia en dos fases");

        layer.export_snapshot(&file).expect("exportar");
        let restored = SovereignLayer::import_snapshot(&file).expect("importar");

        // **Cinco, no cuatro.** Dos aperturas, una emision, y **un envio mas
        // un cobro**: la via en dos fases deja dos entradas donde `transfer`
        // dejaba una.
        assert_eq!(restored.transition_log().len(), 5);
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
        two_phase_transfer(&mut layer, alice, SK_ALICE, bob, SK_BOB, 1000, salt_de(0x51A3))
            .expect("transferencia en dos fases");
        layer.export_snapshot(&file).expect("exportar");

        // El registro esta al final del fichero. ⚠️ §802: desde la v9 le sigue
        // la cola de las identidades retiradas, 8 B sin ninguna, como aqui: el
        // byte `len - 20` sigue cayendo en la ultima entrada del registro.
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
            let mut layer = open_encrypted_retry(
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
        quitar_libro(&path);
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
            let mut layer = open_encrypted_retry(
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
        quitar_libro(&path);
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
            let mut layer = open_encrypted_retry(
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
        quitar_libro(&path);
    }

    /// ⚠️ §774 · **La instantanea cifrada se importa sin escribir el texto en claro en disco.**
    /// Discrimina: donde la importacion de antes escribia `{file}.tmp-descifrada` hay un
    /// DIRECTORIO, y escribir ahi falla tambien como root; la de hoy importa desde memoria, y el
    /// directorio queda como estaba.
    #[test]
    fn una_instantanea_cifrada_se_importa_sin_escribir_el_claro_en_disco() {
        let path = temp_path("snapclaro");
        let file = temp_file("snapclaro");
        let key = crypto::LedgerKey::from_passphrase("una contrasena larga de prueba");
        let raiz;
        {
            let mut layer = open_encrypted_retry(
                &path,
                custodian_root(),
                governance_root(),
                LIMIT,
                MAX_SUPPLY,
                MAX_ACCOUNTS,
                Some(key.clone()),
            )
            .expect("abrir");
            open_and_fund(&mut layer, SK_ALICE, 1_000);
            raiz = layer.state_root();
            layer.export_snapshot(&file).expect("exportar");
        }
        let hueco = format!("{file}.tmp-descifrada");
        let _ = std::fs::remove_dir_all(&hueco);
        std::fs::create_dir(&hueco).expect("el directorio en el sitio del temporal");
        let r = SovereignLayer::import_snapshot_with_key(&file, &key);
        let vacio = std::fs::read_dir(&hueco).map(|mut d| d.next().is_none());
        let _ = std::fs::remove_dir_all(&hueco);
        let _ = std::fs::remove_file(&file);
        quitar_libro(&path);
        assert_eq!(
            r.expect("importar sin pasar por el disco").state_root(),
            raiz
        );
        assert_eq!(
            vacio.ok(),
            Some(true),
            "nada se escribio junto a la instantanea"
        );
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
            let mut layer = open_encrypted_retry(
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
        quitar_libro(&path);
    }

    // -----------------------------------------------------------------
    // §702 (RFC-0001): la instantánea cifrada, con la versión de su KDF
    // -----------------------------------------------------------------

    /// **§702: la instantánea cifrada lleva la cabecera del KDF de su libro y abre con la
    /// frase.** La marca es 0x02, detrás van los 29 bytes de `meta:kdf` del libro, y con la frase
    /// se importa sin el libro; con un bit de la sal cambiado, la clave es otra y no abre.
    #[test]
    fn la_instantanea_cifrada_lleva_la_cabecera_de_su_libro() {
        use super::SNAPSHOT_ENCRYPTED_V2;
        use crate::crypto::{LedgerKey, LARGO_CABECERA};
        let path = temp_path("snapkdf");
        let file = temp_file("snapkdf");
        let frase = "una contrasena larga de prueba";
        let (alice, cabecera) = {
            let mut layer = open_encrypted_retry(
                &path,
                custodian_root(),
                governance_root(),
                LIMIT,
                MAX_SUPPLY,
                MAX_ACCOUNTS,
                Some(LedgerKey::from_passphrase(frase)),
            )
            .expect("abrir cifrada");
            let alice = open_and_fund(&mut layer, SK_ALICE, 250_000);
            layer.export_snapshot(&file).expect("exportar");
            let c = layer.db().unwrap().get(crate::persistence::CLAVE_KDF).unwrap().unwrap();
            (alice, c.to_vec())
        };
        let bytes = std::fs::read(&file).expect("leer");
        assert_eq!(bytes[0], SNAPSHOT_ENCRYPTED_V2, "la marca de la version 2");
        assert_eq!(bytes[1..1 + LARGO_CABECERA], cabecera[..], "la cabecera del libro");
        let r = SovereignLayer::import_snapshot_with_key(&file, &LedgerKey::from_passphrase(frase))
            .expect("importar con la frase");
        assert_eq!(r.balance_of(alice), Some(250_000));

        let mut otra = bytes.clone();
        otra[1 + LARGO_CABECERA - 1] ^= 0x01;
        std::fs::write(&file, &otra).unwrap();
        assert!(
            SovereignLayer::import_snapshot_with_key(&file, &LedgerKey::from_passphrase(frase))
                .is_err(),
            "CRITICO: una sal cambiada abrio la instantanea"
        );
        let _ = std::fs::remove_file(&file);
        quitar_libro(&path);
    }

    /// **§702: una instantánea cifrada de la versión 1 se sigue importando.** Este código ya no
    /// la escribe: se fabrica con la regla documentada -marca 0x01 y el cuerpo sellado con la
    /// clave de SHA-256 sin sal-, como `v3_snapshot_bytes`. Con su frase restaura el estado; con
    /// otra, no. Una escrita por el código de `590caae` se importó, fuera del árbol, en el §702.
    #[test]
    fn una_instantanea_cifrada_de_la_version_1_se_sigue_importando() {
        use super::{SNAPSHOT_ENCRYPTED, SNAPSHOT_PLAIN};
        use crate::crypto::{Kdf, LedgerKey};
        let file = temp_file("snapv1");
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 123_456);
        layer.export_snapshot(&file).expect("exportar en claro");
        let claro = std::fs::read(&file).expect("leer");
        assert_eq!(claro[0], SNAPSHOT_PLAIN);
        let v1 = LedgerKey::derivar("la frase vieja", &Kdf::Sha256V1).unwrap();
        let mut viejo = vec![SNAPSHOT_ENCRYPTED];
        viejo.extend_from_slice(&v1.seal(&claro[1..]).unwrap());
        std::fs::write(&file, &viejo).unwrap();

        let r = SovereignLayer::import_snapshot_with_key(
            &file,
            &LedgerKey::from_passphrase("la frase vieja"),
        )
        .expect("la v1 se importa con su frase");
        assert_eq!(r.balance_of(alice), Some(123_456));
        assert_eq!(r.state_root(), layer.state_root());
        assert!(
            SovereignLayer::import_snapshot_with_key(&file, &LedgerKey::from_passphrase("otra"))
                .is_err(),
            "CRITICO: otra frase abrio la instantanea v1"
        );
        let _ = std::fs::remove_file(&file);
    }

    // -----------------------------------------------------------------
    // §802: la instantanea v9, con las identidades retiradas
    // -----------------------------------------------------------------

    /// Una capa con Alice recuperada de X hacia Y, exportada en claro; devuelve (X, Y).
    fn v9_con_una_recuperacion(file: &str) -> (Digest, Digest) {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000);
        let x = layer.public_id_of(alice).expect("alice");
        let y = derive_public_id(winterfell::math::fields::f64::BaseElement::new(0xA11CE_2));
        recover_delegated(&mut layer, alice, y);
        layer.export_snapshot(file).expect("exportar");
        (x, y)
    }

    /// La misma instantanea, como la habria escrito el codigo de antes: el magico de la v8 y sin
    /// la cola de `n_retir` identidades.
    fn a_v8(bytes: &mut Vec<u8>, n_retir: usize) {
        bytes[1..9].copy_from_slice(b"ZKSSL8\0\0");
        bytes.truncate(bytes.len() - 8 - 32 * n_retir);
    }

    fn abrir_ajena(l: &mut SovereignLayer, id: Digest) -> Result<AccountIndex, LayerError> {
        use winterfell::math::fields::f64::BaseElement;
        l.open_with_id(id, [BaseElement::new(9); 4], [BaseElement::new(7); 4])
    }

    fn es_de_las_retiradas<T>(r: &Result<T, LayerError>) -> bool {
        matches!(
            r,
            Err(LayerError::Store(StoreError::IntegrityFailure {
                what: "identidades retiradas de la instantanea"
            }))
        )
    }

    /// **§802: la instantanea v9 lleva las retiradas.** Exportar tras X->Y escribe el magico de la
    /// v9, e importarla deja la invariante en pie: ni X ni Y abren otra cuenta en la capa
    /// restaurada.
    #[test]
    fn la_instantanea_v9_lleva_las_retiradas() {
        let file = temp_file("v9_retiradas");
        let (x, y) = v9_con_una_recuperacion(&file);
        let bytes = std::fs::read(&file).expect("leer");
        assert_eq!(&bytes[1..9], b"ZKSSL9\0\0", "la instantanea se escribe en v9");
        let mut r = SovereignLayer::import_snapshot(&file).expect("importar la v9");
        for (nombre, id) in [("X, la retirada", x), ("Y, la recuperada", y)] {
            let a = abrir_ajena(&mut r, id);
            assert!(
                matches!(a, Err(LayerError::IdentityUnavailable)),
                "CRITICO: en la capa restaurada, {nombre} abrio otra cuenta: {a:?}"
            );
        }
        let _ = std::fs::remove_file(&file);
    }

    /// **§802: una v8 con recuperaciones no importa, y sin ellas si.** La v8 no trae las
    /// retiradas: con una entrada Recovery en su registro, importarla vacia devolveria la
    /// identidad retirada; sin ninguna, no hay nada que perder, e importa con el indice rehecho
    /// de los registros.
    #[test]
    fn una_v8_con_recuperaciones_no_importa_y_sin_ellas_si() {
        let file = temp_file("v8_con_recuperacion");
        v9_con_una_recuperacion(&file);
        let mut bytes = std::fs::read(&file).expect("leer");
        a_v8(&mut bytes, 1);
        std::fs::write(&file, &bytes).expect("escribir");
        let r = SovereignLayer::import_snapshot(&file);
        assert!(
            es_de_las_retiradas(&r),
            "CRITICO: una v8 con una recuperacion se importo sin sus retiradas: {:?}",
            r.as_ref().err()
        );
        drop(r);

        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000);
        let x = layer.public_id_of(alice).expect("alice");
        layer.export_snapshot(&file).expect("exportar");
        let mut bytes = std::fs::read(&file).expect("leer");
        a_v8(&mut bytes, 0);
        std::fs::write(&file, &bytes).expect("escribir");
        let mut r = SovereignLayer::import_snapshot(&file).expect("una v8 sin recuperaciones importa");
        let a = abrir_ajena(&mut r, x);
        assert!(matches!(a, Err(LayerError::IdentityUnavailable)), "el indice de la v8: {a:?}");
        let _ = std::fs::remove_file(&file);
    }

    /// **§802: una v9 con una retirada de menos no importa.** La cola dice cero sobre un registro
    /// con una Recovery: la cuenta se cruza con el registro verificado, no con la cabecera.
    #[test]
    fn una_v9_con_una_retirada_de_menos_no_importa() {
        let file = temp_file("v9_retirada_de_menos");
        v9_con_una_recuperacion(&file);
        let mut bytes = std::fs::read(&file).expect("leer");
        let n = bytes.len();
        bytes.truncate(n - 32);
        let fin = bytes.len();
        bytes[fin - 8..].copy_from_slice(&0u64.to_le_bytes());
        std::fs::write(&file, &bytes).expect("escribir");
        let r = SovereignLayer::import_snapshot(&file);
        assert!(
            es_de_las_retiradas(&r),
            "CRITICO: una v9 sin la retirada de su recuperacion se importo: {:?}",
            r.as_ref().err()
        );
        drop(r);
        let _ = std::fs::remove_file(&file);
    }
}

// ── §702 (RFC-0001): la instantánea cifrada con la versión 2 del KDF ──
// Va al final del fichero, detrás de los tests, para que las citas
// `snapshot.rs:línea` de los documentos que las llevan no se muevan.

/// Marca de instantánea cifrada con la versión 2 del KDF: tras ella, la cabecera del KDF del libro
/// en claro (29 B) y el cuerpo sellado. Es la que se escribe desde el §702. ⚠️ §821: «del libro», o, si la exporta una capa importada con clave, la de su instantanea, o una de sal nueva si venia de la v1 (la 121; `conservar_la_clave`, al final).
const SNAPSHOT_ENCRYPTED_V2: u8 = 0x02;

/// La marca de una instantánea cifrada con la clave `k` y, con la versión 2, la cabecera del KDF
/// detrás: la instantánea se abre con la frase, sin el libro. La versión 1 solo se vería aquí si
/// la clave de la capa no se hubiera migrado, y `open_encrypted` la migra al abrir. ⚠️ §821: y la de una capa importada de una v1, `import_snapshot_with_key` (`conservar_la_clave`, abajo).
fn marca_y_cabecera(k: &crate::crypto::LedgerKey) -> Vec<u8> {
    match k.kdf().and_then(|kdf| kdf.cabecera()) {
        Some(c) => {
            let mut v = vec![SNAPSHOT_ENCRYPTED_V2];
            v.extend_from_slice(&c);
            v
        }
        None => vec![SNAPSHOT_ENCRYPTED],
    }
}

/// El cuerpo de una instantánea cifrada -marca 0x01 o 0x02-, con la clave fijada al KDF que dice
/// su marca: la versión 1 sin cabecera, o la cabecera de la 2. Una cabecera que no es exactamente
/// la de la versión 2 no se lee, y una sal cambiada deriva otra clave y no abre. ⚠️ §821: devuelve también esa clave fijada, que la capa importada conserva (`conservar_la_clave`, abajo).
fn abrir_sellada(buf: &[u8], key: &crate::crypto::LedgerKey) -> Result<(Vec<u8>, crate::crypto::LedgerKey), LayerError> {
    use crate::crypto::{Kdf, LARGO_CABECERA};
    let (kdf, cuerpo) = match buf.first() {
        Some(&SNAPSHOT_ENCRYPTED_V2) => {
            let fin = 1 + LARGO_CABECERA;
            if buf.len() < fin {
                return Err(malformed("instantanea cifrada sin la cabecera del KDF entera"));
            }
            (Kdf::de_cabecera(&buf[1..fin]).map_err(LayerError::Store)?, &buf[fin..])
        }
        _ => (Kdf::Sha256V1, buf.get(1..).unwrap_or(&[])),
    };
    key.fijar(&kdf).and_then(|k| Ok((k.open(cuerpo)?, k))).map_err(LayerError::Store)
}

/// ⚠️ §812 · **Barre el `{path}.tmp-descifrada` que dejaba una importacion muerta.** Entre `5d2833e`
/// (28-07-2026) y `c87d7d6` (§774, 08-10-2026), `import_snapshot_with_key` escribia el estado
/// entero EN CLARO en `{path}.tmp-descifrada`, junto a la instantanea cifrada, y lo borraba
/// despues: un proceso que muriera entre medias lo dejaba en el disco para siempre. El §774 quito
/// el escritor, y no barria. Desde el §812, la importacion de esa misma instantanea lo borra si es un
/// fichero -un directorio, un enlace o cualquier otra cosa con ese nombre no los escribio aquel
/// codigo, y no se tocan- y lo dice por la salida de errores: quien lo lea sabe asi que el estado
/// de su libro estuvo en claro en ese disco, y borrar no sobrescribe -los bloques libres, un SSD,
/// un sistema copy-on-write o el journal pueden seguir guardandolo (`SECURITY.md` §3.14)-. Si no
/// puede borrarlo, la importacion falla y lo dice: no se importa con el claro al lado sin que nadie
/// lo vea. Devuelve si lo borro. El de una instantanea que no se vuelva a importar se busca junto a
/// ella y se borra a mano.
///
/// ⚠️ §819: esa negativa y el aviso tienen test desde el §819
/// (`si_no_puede_barrer_el_claro_la_importacion_falla_sin_importar_nada`): la negativa, con el
/// borrado MISMO hecho fallar (`ruta_del_borrado`, abajo); el aviso, leido por el test en lo que
/// guarda el hilo (`avisar_del_barrido`, abajo). Que vaya a la salida de errores no lo ve el test.
pub(crate) fn barrer_descifrada(path: &str) -> Result<bool, LayerError> {
    // El nombre, como lo formaba aquel codigo: pegando el sufijo a la ruta.
    let resto = format!("{path}.tmp-descifrada");
    match std::fs::symlink_metadata(&resto) {
        Ok(m) if m.file_type().is_file() => {}
        _ => return Ok(false),
    }
    std::fs::remove_file(&*ruta_del_borrado(&resto)).map_err(|e| {
        io_err(format!(
            "{resto}: es el estado del libro EN CLARO que dejo una importacion muerta de una version \
             entre 5d2833e y c87d7d6, y no se pudo borrar ({e}); no se importa con el al lado: hay \
             que borrarlo a mano"
        ))
    })?;
    // ⚠️ §819: hasta el §819, `eprintln!` aqui mismo; ahora por `avisar_del_barrido` (abajo), que lo
    // escribe igual y deja que un test lo lea.
    avisar_del_barrido(format_args!(
        "aviso (§812): se borro {resto}, el estado del libro EN CLARO que dejo una importacion muerta \
         de una version entre 5d2833e y c87d7d6; borrar no sobrescribe, y su contenido puede seguir \
         en el disco (SECURITY.md §3.14)"
    ));
    Ok(true)
}

/// ⚠️ §817 · **Con clave, una instantánea en claro no se importa.** Hasta el §817,
/// `import_snapshot_with_key` importaba también la marca `0x00`, en claro, sin error y sin mirar la
/// clave: si se autenticaba o no lo decidía la marca, que pone el fichero. Quien pudiera escribir
/// donde se guarda la copia cifrada ponía en su sitio una en claro, hecha con esta biblioteca, y
/// quien la restauraba con su frase recibía Ok y la capa de quien la puso, sus saldos incluidos
/// (`SECURITY.md` §3.16). Una en claro no la autentica nadie: `import_snapshot`, que no recibe
/// clave, la importa sin autenticar nada, y el error lo dice, porque quien esperaba una cifrada y
/// encuentra una en claro puede tener delante la de quien escribe donde se guarda. Lo ata
/// `con_clave_una_instantanea_en_claro_no_se_importa`, al final de `tests.rs`. ⚠️ §828: ya no es el último: detrás van otros, y los últimos son los del §828 (`SECURITY.md` §3.17).
fn rehusar_en_claro() -> LayerError {
    LayerError::Store(crate::store::StoreError::Malformed(
        "la instantanea esta en claro y no la autentica la clave: con clave solo se importa una \
         cifrada; si esperabas una cifrada, pudo ponerla quien escriba donde se guarda, e \
         import_snapshot la importaria sin autenticar nada: solo si sabes de donde sale"
            .into(),
    ))
}

/// ⚠️ §819 · **El aviso de `barrer_descifrada`.** Fuera de los tests, `eprintln!`, como desde el
/// §812. En ellos lo guarda el hilo en `AVISOS_DEL_BARRIDO`, y
/// `si_no_puede_barrer_el_claro_la_importacion_falla_sin_importar_nada` lo lee: que se avisa al
/// borrar, que no se avisa de un borrado que no se hizo, y que el aviso nombra el fichero y dice
/// que estaba en claro. Lo que el test no ve es la salida de errores misma: que el aviso vaya a ella
/// lo dice esta funcion, un `eprintln!` de una linea, sin test. Se declara; dejarlo sin test es de
/// la sesion, reversible.
#[cfg(not(test))]
fn avisar_del_barrido(aviso: std::fmt::Arguments<'_>) {
    eprintln!("{aviso}");
}

#[cfg(test)]
fn avisar_del_barrido(aviso: std::fmt::Arguments<'_>) {
    AVISOS_DEL_BARRIDO.with(|a| a.borrow_mut().push(aviso.to_string()));
}

#[cfg(test)]
thread_local! {
    pub(crate) static AVISOS_DEL_BARRIDO: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// ⚠️ §819 · **La ruta que borra `barrer_descifrada`, y su gancho en los tests.** Fuera de los tests
/// es la del resto, tal cual. En ellos, si el hilo lo pidio en `FALLAR_BORRADO`, es la del resto
/// con una barra al final: `remove_file` la rehusa (`ENOTDIR`: un fichero no es un directorio) y el
/// resto se queda. Es el borrado MISMO el que falla, no un paso aparte antes de el: el `Result` que
/// la importacion propaga es el de `remove_file`, y un mutante que lo ignore cae en
/// `si_no_puede_barrer_el_claro_la_importacion_falla_sin_importar_nada`.
#[cfg(not(test))]
#[inline(always)]
fn ruta_del_borrado(resto: &str) -> std::borrow::Cow<'_, str> {
    std::borrow::Cow::Borrowed(resto)
}

#[cfg(test)]
fn ruta_del_borrado(resto: &str) -> std::borrow::Cow<'_, str> {
    if FALLAR_BORRADO.with(|f| f.get()) {
        std::borrow::Cow::Owned(format!("{resto}/"))
    } else {
        std::borrow::Cow::Borrowed(resto)
    }
}

#[cfg(test)]
thread_local! {
    pub(crate) static FALLAR_BORRADO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

// ── §820: la exportacion, entera o nada (la 120 del `BACKLOG.md`) ──
// Va al final, como el §702 y el §812, para que las citas `snapshot.rs:linea` no se muevan.

/// ⚠️ §820 · **El temporal de la exportacion**: `{instantanea}.exportando`, en su mismo directorio.
/// Es un nombre reservado: la exportacion no crea ni borra uno que ya exista.
pub(crate) const SUFIJO_EXPORTANDO: &str = ".exportando";

/// ⚠️ §820 · **La instantanea, entera o nada.** Hasta el §820, `export_snapshot` abria el destino con
/// `File::create`, que lo trunca, y escribia encima: un proceso que muriera a mitad dejaba en el
/// sitio de la buena una a medias, que no importa, y la anterior ya no estaba. Ahora:
///
/// 1. El destino, si existe, tiene que ser un fichero regular que se pueda abrir para escribir,
///    como lo pedia la escritura en sitio. Lo que no lo es se rehusa con su nombre, como el
///    recifrado rehusa un enlace: un enlace no se sigue -el cambio de nombre lo sustituiria y
///    dejaria su destino como estaba-, ni un FIFO, un socket o un dispositivo, que tambien
///    sustituiria; ni un directorio, que el cambio de nombre rehusaria igual (`EISDIR`), como lo
///    rehusaba `File::create`. Cambia lo que hacia `File::create` con los demas, que escribia a
///    traves de ellos: `/dev/stdout` -un enlace- o un FIFO ya no son destinos de una exportacion
///    (una copia por una tuberia).
/// 2. El temporal `{path}.exportando`, en el MISMO directorio, se CREA (`create_new`; en Unix, con
///    0600): exportar pide ahora poder escribir en el directorio del destino, aunque el destino se
///    pueda escribir. Si el temporal ya existe no se toca, y se dice: es otra exportacion a ese
///    destino en marcha o lo que dejo una que murio, y la biblioteca no sabe cual. Si no hay ninguna
///    en marcha, se borra a mano; barrerlo sin saberlo romperia el entera o nada de la que siguiera
///    viva. No barrerlo es de la sesion, reversible: falla cerrado.
/// 3. Se escribe entero y se sincroniza. Lleva lo que llevaria la nueva: cifrado si la capa tiene
///    clave, en claro si no. Con la capa que escribio la anterior no es un claro junto a una cifrada
///    (la leccion del §774); una capa sin clave que exporta sobre una cifrada -la importada de una,
///    que nace sin clave: la 121- deja un claro junto a ella, el mismo que dejaria si acabara
///    (medido en la sesion con un test que no queda en el arbol: el destino, `0x02`; el temporal,
///    `0x00` y el saldo en claro). ⚠️ §821: «la importada de una, que nace sin clave», ya no: desde el §821 conserva la clave de la instantanea (la 121, `conservar_la_clave`, al final); lo deja una sin clave, como la de `import_snapshot` o una nueva.
/// 4. Si el destino existia, el temporal toma, con privilegios, su dueno, y su grupo y sus bits de
///    modo (`dar_lo_de_la_anterior`); si no puede tomar su grupo, se queda sin los bits de grupo y
///    de otros. Sus ACL POSIX y sus demas atributos extendidos, que la escritura en sitio
///    conservaba, no los toma: ver `dar_lo_de_la_anterior`. Uno nuevo nace con 0600 en Unix; hasta
///    el §820, con 0666 menos la umask.
/// 5. Se cambia de nombre por el destino y se sincroniza el directorio, donde vive el nombre.
///
/// Un error antes del cambio de nombre borra el temporal y deja la anterior como estaba; uno al
/// sincronizar el directorio deja la nueva en su sitio, entera, y lo dice. Una muerte antes del
/// cambio de nombre deja la anterior entera y el temporal, quizas a medias: la importacion lee el
/// destino y nunca el temporal, y uno a medias no importa -la verificacion o el sellado lo
/// rechazan-. Lo miden `una_exportacion_que_falla_a_mitad_deja_la_anterior_entera` y
/// `una_exportacion_muerta_a_mitad_deja_la_anterior_y_su_temporal_no_se_confunde`. No se mide un
/// corte de corriente: quitar el `sync_all` del temporal o la sincronizacion del directorio no lo
/// tumba ningun test (medido). Ni que un destino que no se puede abrir para escribir se rehuse: en
/// la maquina de la sesion los tests corren como root, que lo abre igual. Un enlace duro que alguien
/// hiciera a la anterior se queda con la anterior, que ya no se trunca.
fn escribir_entera(path: &str, bytes: &[u8]) -> Result<(), LayerError> {
    use std::io::ErrorKind;
    let previo = match std::fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_file() => {
            // Lo que la escritura en sitio no podia abrir para escribir, esta no lo sustituye.
            std::fs::OpenOptions::new()
                .write(true)
                .open(path)
                .map_err(|e| io_err(format!("{path}: {e}; la exportacion no lo sustituye")))?;
            Some(m)
        }
        Ok(_) => {
            return Err(io_err(format!(
                "{path}: no es un fichero regular -un enlace, un directorio u otra cosa-, y la \
                 exportacion no lo toca: un enlace o un FIFO los sustituiria el cambio de nombre"
            )))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(e) => return Err(io_err(format!("{path}: {e}"))),
    };
    let tmp = format!("{path}{SUFIJO_EXPORTANDO}");
    let mut o = std::fs::OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    let f = o.open(&tmp).map_err(|e| {
        if e.kind() == ErrorKind::AlreadyExists {
            io_err(format!(
                "{tmp}: ya existe: es otra exportacion a {path} en marcha, o lo que dejo una que murio \
                 -lo que llevaria la nueva, quizas a medias-; no se toca, y si no hay ninguna en marcha \
                 se borra a mano"
            ))
        } else {
            io_err(format!("{tmp}: {e}"))
        }
    })?;
    if let Err(e) = escribir_y_cambiar(f, &tmp, path, bytes, previo) {
        let _ = std::fs::remove_file(&tmp);
        return Err(io_err(format!(
            "{path}: la exportacion no acabo ({e}); la instantanea anterior, si la habia, queda como estaba"
        )));
    }
    sincronizar_directorio(path).map_err(|e| {
        io_err(format!(
            "{path}: la nueva esta en su sitio, entera, pero no se pudo sincronizar el directorio \
             ({e}): tras un corte de corriente el nombre podria volver a la anterior"
        ))
    })
}

/// Los pasos 3 a 5 de `escribir_entera`, hasta el cambio de nombre: lo que, si falla, deja la
/// anterior como estaba.
fn escribir_y_cambiar(
    mut f: std::fs::File,
    tmp: &str,
    path: &str,
    bytes: &[u8],
    previo: Option<std::fs::Metadata>,
) -> std::io::Result<()> {
    let mitad = bytes.len() / 2;
    f.write_all(&bytes[..mitad])?;
    punto_exportacion(1)?;
    f.write_all(&bytes[mitad..])?;
    if let Some(anterior) = previo {
        dar_lo_de_la_anterior(&f, &anterior)?;
    }
    f.sync_all()?;
    drop(f);
    punto_exportacion(2)?;
    std::fs::rename(tmp, path)
}

/// ⚠️ §820 · **Lo que la nueva toma de la anterior: su dueno, su grupo y sus bits de modo**, parte de
/// lo que la escritura en sitio conservaba sin hacer nada. El cambio de nombre deja en el sitio un
/// fichero nuevo, del usuario y del grupo del proceso -o del directorio, si tiene setgid-: con los
/// permisos de la anterior y otro grupo, sus bits de grupo abririan la instantanea -en claro, si la
/// capa no tiene clave- a quien antes no la leia. En Unix, primero el dueno y el grupo
/// (`fchown`), y despues los permisos. Sin privilegios, cambiar el dueno falla, y se pone solo el
/// grupo, que el dueno de un fichero puede poner si es uno de los suyos. Si tampoco se puede, la
/// nueva se queda con los bits del dueno (`& 0o700`): sin los de grupo y sin los de otros, que con
/// otro grupo tambien cambian a quien alcanzan. Fuera de Unix, los permisos. Lo mide
/// `la_exportacion_guarda_los_permisos_de_la_anterior_y_no_sustituye_lo_que_no_es_un_fichero`, (e)
/// y (f); el (e), como root.
///
/// Lo que NO toma: las ACL POSIX de la anterior y sus demas atributos extendidos, que la escritura
/// en sitio conservaba. Con una ACL extendida, los bits de grupo del modo son su mascara, y la nueva,
/// sin ACL, se los da al grupo dueno. Medido en la sesion con un test que no queda en el arbol: una
/// anterior 0640 con `user:4242:r--`, `group::---` y `mask::r--` deja una nueva 0640 sin ACL, que el
/// grupo dueno lee -en claro, si la capa no tiene clave- y antes no leia. Copiarlas pide
/// `fgetxattr` y `fsetxattr`, una dependencia que la capa no tiene. Se declara, sin test.
fn dar_lo_de_la_anterior(f: &std::fs::File, anterior: &std::fs::Metadata) -> std::io::Result<()> {
    #[cfg(unix)]
    let permisos = {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let modo = anterior.permissions().mode();
        let con_su_grupo = cambiar_dueno(f, Some(anterior.uid()), Some(anterior.gid())).is_ok()
            || cambiar_dueno(f, None, Some(anterior.gid())).is_ok();
        let modo = if con_su_grupo { modo } else { modo & 0o700 };
        std::fs::Permissions::from_mode(modo)
    };
    #[cfg(not(unix))]
    let permisos = anterior.permissions();
    f.set_permissions(permisos)
}

/// `fchown`, y en los tests su gancho: si el hilo lo pidio en `FALLAR_DUENO`, falla como sin
/// privilegios. ⚠️ §828: la comparte el recifrado del libro (`persistence::dar_grupo_y_modo`), que solo pone el grupo: con `uid` siempre `None`.
#[cfg(all(unix, not(test)))]
#[inline(always)]
pub(crate) fn cambiar_dueno(f: &std::fs::File, uid: Option<u32>, gid: Option<u32>) -> std::io::Result<()> {
    std::os::unix::fs::fchown(f, uid, gid)
}

#[cfg(all(unix, test))]
pub(crate) fn cambiar_dueno(f: &std::fs::File, uid: Option<u32>, gid: Option<u32>) -> std::io::Result<()> {
    if FALLAR_DUENO.with(|c| c.get()) {
        return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
    }
    std::os::unix::fs::fchown(f, uid, gid)
}

/// El paso 5, despues del cambio de nombre: el directorio, donde vive el nombre. Una ruta sin
/// directorio (`copia.bin`) es del directorio actual.
fn sincronizar_directorio(path: &str) -> std::io::Result<()> {
    punto_exportacion(3)?;
    #[cfg(unix)]
    {
        let p = std::path::Path::new(path);
        let dir = match p.parent() {
            Some(d) if !d.as_os_str().is_empty() => d,
            _ => std::path::Path::new("."),
        };
        std::fs::File::open(dir)?.sync_all()?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

// §820 · Lo que inyectan los tests de la exportacion en el punto k: 1, con la mitad del temporal
// escrita; 2, entero, sincronizado y cerrado, antes del cambio de nombre; 3, despues, al sincronizar
// el directorio. `punto_exportacion(k)` entra en panico si el hilo lo pidio en
// `MORIR_EN_EXPORTACION`, o devuelve un error si lo pidio en `FALLAR_EN_EXPORTACION`, que sigue el
// camino de un error de verdad en ese punto. El panico es la aproximacion de la muerte de
// `persistence::punto`: corre los `Drop`, y aqui lo unico abierto es el temporal, que se cierra sin
// mas. Fuera de los tests no hace nada y no se compila nada. Y `FALLAR_DUENO` hace fallar el
// `fchown` de `dar_lo_de_la_anterior`, como sin privilegios. ⚠️ §828: y el del recifrado del libro, que comparte `cambiar_dueno` (`persistence::dar_grupo_y_modo`).
#[cfg(test)]
thread_local! {
    pub(crate) static MORIR_EN_EXPORTACION: std::cell::Cell<Option<u8>> = const { std::cell::Cell::new(None) };
    pub(crate) static FALLAR_EN_EXPORTACION: std::cell::Cell<Option<u8>> = const { std::cell::Cell::new(None) };
    pub(crate) static FALLAR_DUENO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn punto_exportacion(k: u8) -> std::io::Result<()> {
    if MORIR_EN_EXPORTACION.with(|m| m.get()) == Some(k) {
        panic!("§820: muerte inyectada en el punto {k} de la exportacion");
    }
    if FALLAR_EN_EXPORTACION.with(|m| m.get()) == Some(k) {
        return Err(std::io::Error::other(format!("§820: fallo inyectado en el punto {k} de la exportacion")));
    }
    Ok(())
}

#[cfg(not(test))]
#[inline(always)]
fn punto_exportacion(_: u8) -> std::io::Result<()> {
    Ok(())
}

// ── §821: la capa importada con clave conserva la clave (la 121 del `BACKLOG.md`) ──
// Va al final, como el §702, el §812 y la 120, para que las citas `snapshot.rs:linea` no se muevan.

/// ⚠️ §821 · **La capa que devuelve `import_snapshot_with_key` conserva la clave de la instantánea.**
/// Hasta el §821 nacía sin clave (`key: None`, en `desde_instantanea_en_claro`), y `export_snapshot`,
/// que sólo cifra si la capa tiene clave, la reexportaba EN CLARO, con la marca `0x00`: quien
/// restauraba una copia cifrada y la volvía a guardar la dejaba en claro, y nada lo decía (la 121
/// del `BACKLOG.md`). Desde el §821, por decisión del autor (21-10-2026), sale con la clave fijada a la
/// cabecera de la instantánea, y `export_snapshot` la reexporta cifrada, sin cambiar ningún formato:
///
/// - **De la versión 2** (marca `0x02`): la clave que `abrir_sellada` fijó al KDF y a la sal de su
///   cabecera para abrirla. `marca_y_cabecera` escribe con ella esa misma cabecera, byte a byte, y
///   la reexportada abre con la misma frase. No cuesta otro Argon2id: se deduce del código, y lo
///   mide sólo el tiempo —una capa que derivara otra vez la misma clave daría lo mismo, y ningún
///   test lo ve—; lo midió la sesión, con un test que no queda en el árbol (el asiento del §821).
/// - **De la versión 1** (marca `0x01`, SHA-256 sin sal, que desde el §702 sólo se lee): la clave se
///   fija a una versión 2 de sal nueva, como `open_encrypted` migra un libro de la versión 1, y la
///   reexportada lleva la marca `0x02` y esa cabecera; la frase es la misma. La sal es nueva en cada
///   importación, y no una fija para toda v1, que haría que todas las copias llevadas a la v2
///   compartieran sal y un mismo precálculo por frase sirviera para todas. Cuesta un Argon2id al
///   importar (medido, en el asiento del §821). Si la clave ya está fijada y no lleva la frase, no
///   puede derivarla, y la importación falla con un error que lo dice, sin devolver la capa:
///   reexportada en la versión 1 escribiría lo que nada escribe desde el §702, y sin clave, en
///   claro. Llevar la v1 a la v2, y fallar sin la frase, es de la sesión, reversible: falla cerrado.
///
/// Fuera de la reexportación, la clave no cambia nada: la capa importada no tiene libro (`db: None`),
/// y `commit` no sella nada sin él, ni le quita la clave. Lo atan
/// `una_capa_importada_de_una_cifrada_se_reexporta_cifrada_con_su_cabecera` -su (f), que la clave
/// sigue tras operar- y `una_capa_importada_de_una_v1_se_reexporta_en_la_version_2` -su (f), que la
/// sal es nueva en cada importación-, al final de `tests.rs`. ⚠️ §828: ya no son los últimos: detrás van los del §828 (`SECURITY.md` §3.17).
fn conservar_la_clave(
    mut capa: SovereignLayer,
    fijada: crate::crypto::LedgerKey,
    key: &crate::crypto::LedgerKey,
) -> Result<SovereignLayer, LayerError> {
    use crate::crypto::Kdf;
    let clave = match fijada.kdf() {
        Some(Kdf::Argon2idV2 { .. }) => fijada,
        _ => key.fijar(&Kdf::nuevo()).map_err(|e| {
            malformed(&format!(
                "la instantanea es de la version 1 del KDF (SHA-256 sin sal), que ya no se escribe: la \
                 capa importada se queda con una clave de la version 2 de sal nueva, y derivarla pide la \
                 frase, que esta clave, ya fijada, no lleva ({e}); importala con \
                 LedgerKey::from_passphrase"
            ))
        })?,
    };
    capa.key = Some(clave);
    Ok(capa)
}
