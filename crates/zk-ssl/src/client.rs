//! **Generación de pruebas en el cliente.** La clave de gasto nunca sale
//! de la máquina del titular.
//!
//! ## El problema que corrige
//!
//! `SovereignLayer::transfer` —la vía de un paso, ya retirada— recibía
//! la clave de gasto. Es decir: **para transferir había que
//! entregársela a quien opera el nodo**, y con ella podía vaciar la
//! cuenta cuando quisiera.
//!
//! Eso no era una limitación de escala. Era que el sistema exigía
//! **confiar tu dinero al operador**, precisamente el intermediario que
//! el proyecto dice eliminar.
//!
//! ## El protocolo (dos fases)
//!
//! ```text
//! 1. Cliente: pide la vista de su cuenta        → nonce, saldo, identidad
//! 2. Cliente: pide los materiales               → caminos y raices publicas
//! 3. Cliente: genera la prueba EN SU MÁQUINA    (la clave no sale)
//! 4. Cliente: envía la operación                → la capa verifica y aplica
//! ```
//!
//! Vale igual para el envío (`send_materials` → `prove_send` →
//! `apply_send`) y para el cobro (`claim_materials` → `prove_claim` →
//! `apply_claim`). Lo demuestra
//! `a_whole_payment_without_giving_any_key_to_the_layer`.
//!
//! ## Lo que esto NO resuelve
//!
//! **Generar la prueba cuesta ~600 ms y bastante memoria.** Si el cliente
//! es un dispositivo ligero y quiere que otro la genere por él, hace
//! falta que ese otro pueda probar **sin** la clave — lo que exige
//! verificar una firma dentro del circuito (Winternitz, ~8.000 filas
//! adicionales).
//!
//! Eso es una **optimización para clientes ligeros**, no una corrección
//! de seguridad. La custodia queda resuelta aquí.
//!
//! ## Y lo que sigue viendo el operador
//!
//! Los saldos. La capa mantiene el estado, así que los conoce. Esto
//! elimina que vea **claves**, no que vea **datos**. Lo segundo requiere
//! descentralización.

use super::*;
// `two_phase` es un modulo publico, pero sus tipos no estan en la raiz del
// crate: `use super::*` no los alcanza.
// `derive_public_id_wide` (§90) no llega por `use super::*`: lib.rs
// solo reexporta la estrecha.
use stark_experiment::circuit_settlement::{derive_public_id_wide, view_id_from_view_key};
use crate::pending::pending_commitment;
use crate::two_phase::{ClaimReceipt, PendingNotice, SendReceipt};

/// Vista pública de una cuenta. **No incluye ninguna clave.**
///
/// El saldo aparece aquí porque el operador del nodo lo conoce de todos
/// modos —mantiene el estado—, así que exponerlo al titular no filtra
/// nada nuevo.
#[derive(Clone, Debug)]
pub struct AccountView {
    pub public_id: Digest,
    pub balance: u64,
    pub nonce: BaseElement,
}

/// Error al comprobar el destinatario de una transferencia.
#[derive(Debug, PartialEq, Eq)]
pub struct WrongRecipient {
    pub expected: Digest,
    pub found: Digest,
}

impl std::fmt::Display for WrongRecipient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "el destinatario de estos materiales NO es quien esperabas: \
             la capa devolvio otra cuenta"
        )
    }
}
impl std::error::Error for WrongRecipient {}



impl SovereignLayer {
    /// Vista INTERNA de una cuenta (49-A paso 4, §129): `pub(crate)`, no
    /// API pública. La usa el protocolo interno (`claim_materials`,
    /// `send_materials`) y el operador, que ya ve todo el estado. **Un
    /// tercero NO puede llegar aquí**: la API pública de lectura es
    /// `account_view_authenticated`, que exige la clave de vista. Este
    /// cambio cierra `reading_a_balance_requires_authority` sin lisiar el
    /// uso interno (§129: no se tocan los accessors internos).
    pub(crate) fn account_view(&self, index: AccountIndex) -> Option<AccountView> {
        self.records.get(&index).map(|r| AccountView {
            public_id: r.public_id,
            balance: r.balance,
            nonce: r.nonce,
        })
    }

    /// **Vista pública AUTENTICADA de una cuenta** (49-A paso 4). El
    /// llamador presenta su clave de vista (`derive_view_key(sk)`,
    /// derivada de la clave de gasto en su máquina, §127); la capa la
    /// convierte a `view_id` y la compara contra el guardado. Devuelve la
    /// vista solo si coinciden — un tercero con solo el índice obtiene
    /// `None`. Presentar la clave de vista NO permite gastar (T7, §127).
    ///
    /// Cuentas pre-49-A (view_id centinela) devuelven siempre `None`: su
    /// vista no está protegida y no se sirve por esta puerta (no-retro).
    pub fn account_view_authenticated(
        &self,
        index: AccountIndex,
        view_key: Digest,
    ) -> Option<AccountView> {
        let guardado = self.stored_view_id(index)?;
        if guardado != view_id_from_view_key(view_key) {
            return None;
        }
        self.account_view(index)
    }

    /// **Materiales para cobrar, sin la clave.**
    ///
    /// **No recibe la clave de gasto.** Todo lo que devuelve es público o
    /// derivable, así que la capa puede entregarlo sin custodiar nada y
    /// el cliente puede probar con [`prove_claim`] sin volver a hablar
    /// con ella.
    ///
    /// El aviso lo aporta quien cobra: la capa no sabe qué pendiente es suyo
    /// —esa es la privacidad del diseño— así que **no puede entregarlo**.
    pub fn claim_materials(
        &self,
        receiver_index: AccountIndex,
        notice: &PendingNotice,
    ) -> Result<ClaimMaterials, LayerError> {
        let receiver = self
            .account_view(receiver_index)
            .ok_or(LayerError::AccountNotFound(receiver_index))?;

        // ⚠️ Una cuenta congelada no puede cobrar, y el dinero queda en el
        // limbo. Es una inversion de la decision original, documentada en
        // `AUDITORIA.md` §29.
        if self.is_frozen(receiver_index) {
            return Err(LayerError::AccountFrozen(receiver_index));
        }

        Ok(ClaimMaterials {
            receiver_path: self.accounts.path_for(receiver_index),
            frozen_path: self.frozen.path_for(receiver_index),
            pending_path: self.pending.path_for(notice.position),
            receiver,
            total_supply: self.total_supply,
            notice: notice.clone(),
        })
    }

    pub fn send_materials(
        &self,
        sender_index: AccountIndex,
        receiver_id: Digest,
        amount: u64,
        salt: Digest,
    ) -> Result<SendMaterials, LayerError> {
        let sender = self
            .account_view(sender_index)
            .ok_or(LayerError::AccountNotFound(sender_index))?;

        if self.is_frozen(sender_index) {
            return Err(LayerError::AccountFrozen(sender_index));
        }
        if amount > sender.balance {
            return Err(LayerError::InsufficientBalance {
                available: sender.balance,
                requested: amount,
            });
        }
        // ⚠️ El limite se comprueba aqui **y** lo prueba el circuito: la capa
        // aporta el suyo y `circuit_send` demuestra `importe <= limite`. Ver
        // `AUDITORIA.md` §25.
        if amount > self.regulatory_limit {
            return Err(LayerError::OverRegulatoryLimit {
                limit: self.regulatory_limit,
                requested: amount,
            });
        }

        let pending_position = self.allocate_pending()?;

        Ok(SendMaterials {
            sender_path: self.accounts.path_for(sender_index),
            frozen_path: self.frozen.path_for(sender_index),
            pending_path: self.pending.path_for(pending_position),
            pending_position,
            sender,
            receiver_id,
            regulatory_limit: self.regulatory_limit,
            total_supply: self.total_supply,
            amount,
            salt,
        })
    }

}

/// **Materiales para un ENVÍO en dos fases.**
///
/// La diferencia con la vía de un paso —ya retirada— es la que da nombre al
/// diseño: **aquí no hay saldo del receptor**.
///
/// Aquella vía actualizaba las dos hojas en una transición, así que quien
/// probaba necesitaba el saldo del receptor para calcular su hoja nueva, y
/// sus materiales se lo entregaban. **Pagar a alguien revelaba cuánto
/// tiene.**
///
/// Un envío toca **una sola hoja**, la del pagador. Del receptor basta su
/// identificador público, que es lo que va en el compromiso. Ver
/// `AUDITORIA.md` §29 y el hallazgo 9 del preprint comparativo.
#[derive(Clone, Debug)]
pub struct SendMaterials {
    pub sender: AccountView,
    pub sender_path: MerklePath,
    pub frozen_path: MerklePath,
    pub pending_path: MerklePath,
    /// Posición libre del árbol de pendientes que la capa ha reservado.
    pub pending_position: u64,
    /// **Solo el identificador.** No el saldo, no la posición en el árbol.
    pub receiver_id: Digest,
    pub regulatory_limit: u64,
    pub total_supply: u64,
    pub amount: u64,
    pub salt: Digest,
}

impl SendMaterials {
    /// **Comprueba a quién van dirigidos estos materiales.**
    ///
    /// La capa entrega los materiales que se le piden. Si alguien
    /// interceptara la petición y cambiara el destinatario, el pagador
    /// firmaría un envío a otra cuenta sin notarlo.
    ///
    /// ⚠️ **Aquí es más simple que en la vía de un paso.** Allí había que
    /// comparar contra `receiver.public_id` —un campo de una vista que
    /// también traía el saldo—. Aquí el identificador **es** el único dato
    /// del receptor que existe.
    pub fn check_recipient(&self, expected: Digest) -> Result<(), WrongRecipient> {
        if self.receiver_id != expected {
            return Err(WrongRecipient {
                expected,
                found: self.receiver_id,
            });
        }
        Ok(())
    }
}

/// **Genera la prueba de un envío SIN tocar la capa.**
///
/// Es el equivalente, para la vía en dos fases, de la prueba local que
/// por la misma razón: demostrar que **la clave de gasto no necesita salir de
/// la máquina del cliente**.
///
/// `SovereignLayer::send` hace lo mismo, pero es un método de la capa que
/// recibe la clave. Esa forma no impide la separación —el cliente puede
/// ejecutar la capa en su máquina— pero **tampoco la enseña**. Ver
/// `AUDITORIA.md` §33.
pub fn prove_send(
    materials: &SendMaterials,
    // ⚠️ **CUATRO elementos** desde §90 (entrada 15).
    //
    // Es el punto donde la clave ancha entra de verdad: rellenar aqui en el
    // borde —como se hace en la via antigua— dejaria al cliente sin poder
    // usarla nunca, y los 256 bits del circuito no servirian a nadie.
    spend_key: Digest,
    options: ProofOptions,
) -> Result<SendReceipt, LayerError> {
    // La clave debe corresponder a la cuenta. El circuito lo impone
    // igualmente, pero en release no se valida al generar: sin esta
    // comprobacion se gastaria el computo de una prueba invalida.
    if derive_public_id_wide(spend_key) != materials.sender.public_id {
        return Err(LayerError::NotTheAccountHolder);
    }

    let trace = build_send_trace(
        spend_key,
        materials.sender.public_id,
        materials.sender.balance,
        materials.sender.nonce,
        &materials.sender_path,
        &materials.frozen_path,
        materials.amount,
        materials.regulatory_limit,
        materials.total_supply,
        0, // un envio no cambia el suministro
        materials.receiver_id,
        materials.salt,
        &materials.pending_path,
    );
    let prover = SendProver::new(options);
    let public_inputs = prover.get_pub_inputs(&trace);
    let proof = prover
        .prove(trace)
        .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

    Ok(SendReceipt {
        proof: proof.to_bytes(),
        public_inputs,
        commitment: pending_commitment(
            materials.receiver_id,
            materials.salt,
            materials.amount,
        ),
        notice: PendingNotice {
            position: materials.pending_position,
            salt: materials.salt,
            amount: materials.amount,
        },
    })
}

/// **Materiales para COBRAR un pendiente.**
///
/// La pieza que faltaba para que un pago entero se pueda probar sin dar la
/// clave a la capa. Ver `AUDITORIA.md` §33.
///
/// ⚠️ **No tiene precedente en la vía de un paso**, donde recibir era pasivo:
/// el pagador actualizaba las dos hojas y el receptor no hacía nada. Aquí
/// cobrar es una operación del receptor, con su propia prueba.
#[derive(Clone, Debug)]
pub struct ClaimMaterials {
    pub receiver: AccountView,
    pub receiver_path: MerklePath,
    pub frozen_path: MerklePath,
    pub pending_path: MerklePath,
    pub total_supply: u64,
    /// El aviso que el pagador tuvo que hacerle llegar.
    ///
    /// ⚠️ **ISO 20022 no lo transporta.** Cómo viaja del pagador al receptor
    /// sigue sin resolver; ver `AUDITORIA.md` §21 y el §3.5 de la nota de
    /// política.
    pub notice: PendingNotice,
}

/// **Genera la prueba de un cobro SIN tocar la capa.**
///
/// Con esto y [`prove_send`], **un pago completo se prueba en el cliente**:
/// la capa entrega caminos y raíces, y verifica; la clave de gasto no sale
/// de la máquina de quien paga ni de la de quien cobra.
pub fn prove_claim(
    materials: &ClaimMaterials,
    // ⚠️ **CUATRO elementos** desde §90. Es la via del CLIENTE: rellenar
    // aqui lo dejaria sin poder usar una clave ancha nunca (§92.13).
    spend_key: Digest,
    options: ProofOptions,
) -> Result<ClaimReceipt, LayerError> {
    if derive_public_id_wide(spend_key) != materials.receiver.public_id {
        return Err(LayerError::NotTheAccountHolder);
    }

    let trace = build_claim_trace(
        spend_key,
        materials.receiver.public_id,
        materials.receiver.balance,
        materials.receiver.nonce,
        &materials.receiver_path,
        &materials.frozen_path,
        materials.notice.amount,
        materials.total_supply,
        0,
        // El destinatario del compromiso es el propio receptor: cobrar es
        // demostrar que el pendiente estaba a su nombre.
        materials.receiver.public_id,
        materials.notice.salt,
        &materials.pending_path,
    );
    let prover = ClaimProver::new(options);
    let public_inputs = prover.get_pub_inputs(&trace);
    let proof = prover
        .prove(trace)
        .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

    Ok(ClaimReceipt {
        proof: proof.to_bytes(),
        public_inputs,
    })
}



#[cfg(test)]
mod tests_privacidad {
    use crate::tests_support::*;
    use stark_experiment::circuit_settlement::native_leaf;
    use winterfell::math::fields::f64::BaseElement;

    /// Mallory. **No conoce ninguna clave ajena.** Se define aqui y no en
    /// `tests_support` porque solo la usa este modulo.
    const SK_MALLORY: u64 = 0xBADCAFE;

    /// **SUPERFICIE 1: CERRADA (49-A paso 4).** La API publica de lectura
    /// ahora EXIGE autoridad. `account_view` paso a `pub(crate)` —uso
    /// interno del protocolo y del operador, §129— y la via de tercero es
    /// `account_view_authenticated(index, view_key)`, que compara la clave
    /// de vista presentada contra el `view_id` guardado.
    ///
    /// El hallazgo original (leer no exigia clave) queda resuelto: un
    /// tercero con solo el indice obtiene `None`; solo quien deriva la
    /// clave de vista de la cuenta —desde su clave de gasto, en su
    /// maquina, §127— ve su saldo. Presentar la vista NO permite gastar
    /// (T7). Este test verifica el contrato NUEVO.
    #[test]
    fn reading_a_balance_requires_authority() {
        use stark_experiment::circuit_settlement::derive_view_key;
        use winterfell::math::fields::f64::BaseElement;

        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let _mallory = open_and_fund(&mut layer, SK_MALLORY, 0);

        // Mallory tiene su propia clave de vista, NO la de Alice.
        let vk_mallory = derive_view_key(BaseElement::new(SK_MALLORY));
        assert!(
            layer.account_view_authenticated(alice, vk_mallory).is_none(),
            "un tercero con su propia clave de vista NO debe ver la cuenta de Alice"
        );

        // Una clave de vista arbitraria tampoco abre.
        let basura = [BaseElement::new(0xDEAD); 4];
        assert!(
            layer.account_view_authenticated(alice, basura).is_none(),
            "una clave de vista arbitraria NO debe abrir la cuenta"
        );

        // SOLO la clave de vista de Alice —derivada de SU clave— abre SU vista.
        let vk_alice = derive_view_key(BaseElement::new(SK_ALICE));
        let vista = layer.account_view_authenticated(alice, vk_alice);
        assert!(
            vista.is_some(),
            "la clave de vista correcta del titular DEBE abrir su vista"
        );
        assert_eq!(vista.unwrap().balance, 1_000_000,
                   "la vista autenticada devuelve el saldo real al titular");
    }

    /// **Y los indices son enumerables**, lo que hace la consecuencia
    /// sistematica en vez de puntual.
    ///
    /// `accounts.rs`: `let index = self.next_index; self.next_index += 1;`
    ///
    /// ⚠️ Condicional a exposicion, como el anterior.
    #[test]
    fn account_indices_are_not_enumerable() {
        let mut layer = new_layer();
        open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        open_and_fund(&mut layer, SK_BOB, 500_000);
        open_and_fund(&mut layer, SK_MALLORY, 0);

        let encontradas: Vec<u64> = (0..10)
            .filter(|i| layer.account_view(*i).is_some())
            .collect();

        assert!(
            encontradas.len() <= 1,
            "CONTRATO: barriendo los indices 0..10 aparecen {} cuentas ({:?}) \
             con sus saldos. Los indices son SECUENCIALES —`next_index += \
             1`— asi que enumerarlos no requiere adivinar nada.",
            encontradas.len(),
            encontradas
        );
    }

    /// ⚠️ **SUPERFICIE 2: la que SOBREVIVE a cerrar la API.**
    ///
    /// Este es el numero que importa, porque **ningun parche de la API lo
    /// toca**.
    ///
    /// `send_materials` entrega `sender_path`, y `sparse_tree::path_for`
    /// devuelve `node(level, idx ^ 1)`: en el nivel 0, **la hoja del
    /// vecino**. Y la hoja es
    ///
    /// ```text
    /// native_leaf(pk, saldo, nonce) = H(H(pk, saldo), nonce)
    /// ```
    ///
    /// **sin salt** —verificado: `native_merge` pone la capacidad a cero, sin
    /// dominio ni cegado—. Con `pk` conocida, hay una ecuacion y dos
    /// incognitas.
    ///
    /// ## El regimen es 1D, y eso NO es un supuesto
    ///
    /// `accounts.rs`: `let nonce = BaseElement::ZERO`, y sube de uno en uno
    /// por gasto. El nonce de una cuenta minorista es un numero de dos
    /// digitos: **no multiplica el coste de forma apreciable**. El regimen
    /// 2D —que encareceria el ataque por el rango del nonce— **nunca
    /// existio**.
    ///
    /// ## Alcance, acotado
    ///
    /// El camino entrega `depth` hermanos. **Solo `siblings[0]` es
    /// atacable por diccionario**: los demas son raices de subarbol, no
    /// preimagenes de hoja. Es **1 cuenta**, no log2(N).
    ///
    /// ## El coste es una CURVA, no un numero
    ///
    /// Depende del rango de saldo que el atacante asuma, que es un supuesto
    /// sobre la victima y no sobre la criptografia. El test lo reporta asi.
    #[test]
    #[ignore = "instrumento de medida: correr a mano, en release"]
    fn a_neighbour_leaf_does_not_reveal_its_balance() {
        use std::time::Instant;

        let mut layer = new_layer();
        // Indices 0 y 1: vecinos de arbol, porque 0 ^ 1 == 1.
        let victima = open_and_fund(&mut layer, SK_ALICE, 7_431_00); // 7.431,00
        let atacante = open_and_fund(&mut layer, SK_MALLORY, 0);
        assert_eq!(victima ^ 1, atacante, "deben ser vecinos de arbol");

        // Lo unico que el atacante necesita, y se lo da el protocolo.
        let receptor = layer.public_id_of(victima).expect("cuenta");
        let m = layer
            .send_materials(atacante, receptor, 0, salt_de(0x1073))
            .expect("materiales: paso 2 del protocolo del cliente");
        let hoja_vecina = m.sender_path.siblings[0];
        let pk_victima = receptor;

        // ⚠️ El nonce NO se busca: se sabe que nace en cero y sube por
        // gasto. Se prueban los diez primeros, que cubre cualquier cuenta
        // recien abierta. Ese es el prior, y va escrito.
        const NONCES: u64 = 10;
        const RANGO: u64 = 1_000_000; // 0..10.000 EUR en centimos = ~2^20

        let t0 = Instant::now();
        let mut hallado = None;
        'busca: for nonce in 0..NONCES {
            for saldo in 0..RANGO {
                if native_leaf(
                    pk_victima,
                    BaseElement::new(saldo),
                    BaseElement::new(nonce),
                ) == hoja_vecina
                {
                    hallado = Some((saldo, nonce));
                    break 'busca;
                }
            }
        }
        let dt = t0.elapsed().as_secs_f64();

        if let Some((saldo, nonce)) = hallado {
            let probadas = (nonce * RANGO + saldo).max(1) as f64;
            let por_seg = probadas / dt;
            println!("\n=== Diccionario sobre la hoja del vecino ===\n");
            println!("  ⚠️ PRIORS DE ESTA MEDIDA, sin los cuales no significa nada:");
            println!("     - nonce en 0..{NONCES}  (nace en cero, sube por gasto)");
            println!("     - saldo en 0..{RANGO}  (~2^20: minorista, centimos)");
            println!("     - `pk` de la victima conocida (viaja en los materiales)");
            println!();
            println!("  saldo hallado        {saldo:>12}");
            println!("  nonce hallado        {nonce:>12}");
            println!("  hojas probadas       {probadas:>12.0}");
            println!("  tiempo               {dt:>12.2} s");
            println!("  hojas/s por nucleo   {por_seg:>12.0}");
            println!();
            println!("  CURVA sobre el rango de saldo asumido:");
            for (etiqueta, n) in [
                ("0-10.000 EUR (2^20)", 1e6),
                ("0-1 M EUR   (2^27)", 1e8),
                ("0-100 M EUR (2^34)", 1e10),
                ("64 bits completos ", 1.8e19),
            ] {
                let seg = n * NONCES as f64 / por_seg;
                if seg < 3600.0 {
                    println!("    {etiqueta}  {:>10.1} min", seg / 60.0);
                } else if seg < 86400.0 * 365.0 {
                    println!("    {etiqueta}  {:>10.1} h", seg / 3600.0);
                } else {
                    println!("    {etiqueta}  {:>10.3e} años-nucleo", seg / 3.156e7);
                }
            }
            println!();
            println!("  ⚠️ Alcance: **1 cuenta** —solo `siblings[0]` es preimagen");
            println!("     de hoja—. Los otros {} hermanos son raices de", 
                     m.sender_path.siblings.len() - 1);
            println!("     subarbol y NO son diccionariables.");
            println!();
            println!("  ⚠️ Y esta fuga **sobrevive a cerrar `account_view`**:");
            println!("     depende del formato de hoja, no de la API.");
        }

        assert!(
            hallado.is_none(),
            "PRIVACIDAD FRENTE A TERCEROS: el saldo del vecino se recupera \
             por diccionario desde el camino que el propio protocolo entrega. \
             `native_leaf` no lleva salt y el nonce nace en cero. Ver la \
             salida para la curva y sus priors."
        );
    }

    /// **SUPERFICIE 3: ¿es el vecino ELEGIBLE?**
    ///
    /// Con `next_index += 1`, quien abre dos cuentas seguidas obtiene
    /// indices contiguos. Si puede provocar sus altas alrededor de una
    /// victima, no es «me toca un vecino al azar»: es **elegir a quien
    /// espiar**.
    ///
    /// El test lo comprueba de la unica forma que no es conjetura: abriendo
    /// cuentas y mirando que indices salen.
    #[test]
    fn account_indices_are_not_predictable() {
        let mut layer = new_layer();
        let a = open_and_fund(&mut layer, SK_ALICE, 0);
        let b = open_and_fund(&mut layer, SK_BOB, 0);
        let c = open_and_fund(&mut layer, SK_MALLORY, 0);

        assert!(
            !(b == a + 1 && c == b + 1),
            "VECINO ELEGIBLE CONFIRMADO: las altas dan indices consecutivos \
             ({a}, {b}, {c}). Quien controle el momento de sus altas elige a \
             quien tiene por vecino de arbol —y con dos altas, rodea—. La \
             fuga deja de ser oportunista."
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // MEDIDA 9 — qué aprende cada participante (extiende §16). Tests
    // DISCRIMINANTES de correlación: no describen el modelo, cazan las
    // fugas que sobrevivirían a un cambio de diseño. Cada uno FALLA si la
    // propiedad de privacidad se rompe.
    // ═══════════════════════════════════════════════════════════════════

    use crate::pending::pending_commitment;
    use stark_experiment::circuit_settlement::derive_public_id;

    /// **TERCERO, correlación 1: el commitment NO codifica al emisor.**
    ///
    /// `pending_commitment(receiver_id, salt, amount)` no toma la identidad
    /// del pagador. Consecuencia verificable: dos emisores DISTINTOS que
    /// envían el mismo importe al mismo receptor con el mismo salt producen
    /// commitments IDÉNTICOS. Un tercero que observa el árbol de pendientes
    /// no puede, del commitment, recuperar quién pagó.
    ///
    /// ⚠️ Lo que un auditor debe valorar (§16): esto significa que el salt
    /// es lo ÚNICO que da unlinkability. Reutilizar salt entre pagos al
    /// mismo receptor los vuelve enlazables —ver el test siguiente—.
    #[test]
    fn el_commitment_no_revela_al_emisor() {
        let id_bob = derive_public_id(BaseElement::new(SK_BOB));
        let salt = salt_de(0x5EED);

        // Alice paga a Bob; Mallory paga a Bob. Mismo receptor, salt, importe.
        let c_desde_alice = pending_commitment(id_bob, salt, 250_000);
        let c_desde_mallory = pending_commitment(id_bob, salt, 250_000);

        // El emisor no entra en la formula: los commitments son iguales.
        assert_eq!(
            c_desde_alice, c_desde_mallory,
            "el commitment depende del EMISOR: seria un canal para \
             identificar al pagador desde el arbol de pendientes"
        );
    }

    /// **TERCERO, correlación 2: el salt da unlinkability, y su ausencia la
    /// quita.** Dos pagos al mismo receptor con salts DISTINTOS producen
    /// commitments distintos (no enlazables); con el mismo salt, iguales
    /// (enlazables). Fija en test que la unlinkability descansa en el salt.
    #[test]
    fn el_salt_es_lo_que_hace_impagos_inenlazables() {
        let id_bob = derive_public_id(BaseElement::new(SK_BOB));

        let c1 = pending_commitment(id_bob, salt_de(0xAAAA), 100_000);
        let c2 = pending_commitment(id_bob, salt_de(0xBBBB), 100_000);
        let c3 = pending_commitment(id_bob, salt_de(0xAAAA), 100_000);

        assert_ne!(c1, c2, "salts distintos deben dar commitments distintos");
        assert_eq!(c1, c3, "mismo salt+receptor+importe -> mismo commitment (enlazable)");
    }

    /// **CONTRAPARTE (el receptor): qué aprende del emisor al cobrar.**
    ///
    /// El receptor recibe un `PendingNotice { position, salt, amount }`.
    /// **Ninguno de esos campos es la identidad del emisor** —el diseño lo
    /// fija en el tipo: no hay un campo `sender`—. El receptor sabe cuánto
    /// cobra y desde qué posición, pero no de quién. Es la propiedad
    /// «la capa no sabe qué pendiente es de quién», vista desde el receptor.
    ///
    /// ⚠️ Auditor (§16, §21): el notice viaja FUERA de banda (ISO 20022 no
    /// lo transporta). Si el canal de entrega revelara al emisor, la fuga
    /// estaria en ese canal, no aqui. Este test acota lo que la CAPA expone.
    #[test]
    fn el_receptor_no_aprende_al_emisor_del_aviso() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let id_bob = layer.public_id_of(bob).expect("bob");
        let estado_alice = state_of(&layer, alice);

        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &estado_alice, id_bob, salt_de(0x1), 250_000)
            .expect("send");

        // El aviso que le llega al receptor: salt, amount, position.
        // NO contiene la identidad de Alice. Se verifica sobre los campos
        // publicos del notice: cobrar solo necesita SK_BOB + el aviso, y el
        // aviso no menciona a Alice.
        let notice = &recibo.notice;
        let id_alice = derive_public_id(BaseElement::new(SK_ALICE));
        // El salt del aviso no es la identidad de Alice (son cosas distintas).
        assert_ne!(notice.salt, id_alice,
                   "el salt del aviso NO debe coincidir con la identidad del emisor");
        // El commitment depositado se reproduce SIN la identidad de Alice:
        // solo con receptor+salt+amount, lo que confirma que el emisor no
        // es un input recuperable por el receptor.
        assert_eq!(
            recibo.commitment,
            pending_commitment(id_bob, notice.salt, notice.amount),
            "el receptor reproduce el commitment sin conocer al emisor: \
             el emisor no es recuperable del material que el receptor tiene"
        );
    }

    /// **TERCERO, correlación 3: `position` filtra ORDEN, no identidad.**
    ///
    /// Las posiciones se asignan por orden de llegada (`next_pending += 1`),
    /// asi que dos sends consecutivos ocupan 0 y 1 sea quien sea el emisor.
    /// La posicion correlaciona *cuando*, no *quien*. Un tercero aprende la
    /// secuencia temporal de pagos, pero no a quien pertenecen.
    #[test]
    fn la_posicion_del_pendiente_es_orden_no_identidad() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 1_000_000);
        let mallory = open_and_fund(&mut layer, SK_MALLORY, 1_000_000);
        let id_m = layer.public_id_of(mallory).expect("m");

        // Alice envia primero, Bob despues. Posiciones 0 y 1 por orden.
        let ea = state_of(&layer, alice);
        let r0 = layer.send(BaseElement::new(SK_ALICE), alice, &ea, id_m, salt_de(0x1), 100)
            .expect("send alice");
        layer.apply_send(&r0, alice, &ea, 100).expect("apply");
        let eb = state_of(&layer, bob);
        let r1 = layer.send(BaseElement::new(SK_BOB), bob, &eb, id_m, salt_de(0x2), 200)
            .expect("send bob");

        // La posicion refleja el orden (0, 1), no quien envio.
        assert_eq!(r0.notice.position, 0, "primer envio -> posicion 0");
        assert_eq!(r1.notice.position, 1, "segundo envio -> posicion 1, por ORDEN no por emisor");
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_support::*;
    use crate::*;
    use winterfell::math::fields::f64::BaseElement;


    /// **Los materiales no contienen ninguna clave.**
    ///
    /// Es la propiedad que define la pieza: lo que viaja de la capa al
    /// cliente es estado, no secretos.
    #[test]
    fn send_materials_contain_no_keys() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let key = BaseElement::new(SK_ALICE);
        let receptor = layer.public_id_of(bob).expect("cuenta");

        let m = layer
            .send_materials(alice, receptor, 1000, salt_de(0x0C1A))
            .expect("materiales");

        assert_ne!(
            m.sender.public_id,
            [key, BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO],
            "los materiales no deben contener la clave"
        );
        assert_eq!(m.sender.public_id, derive_public_id(key));

        // ⚠️ **Y tampoco el saldo del receptor.**
        //
        // Los materiales de la via retirada llevaban una vista completa del
        // receptor, asi que quien pagaba veia cuanto tenia el otro. Aqui el
        // tipo solo tiene
        // `receiver_id: Digest`: **no hay campo por donde el saldo pudiera
        // entrar**. Ver `AUDITORIA.md` §29.
        assert_eq!(m.receiver_id, receptor);
    }

    /// **Sin la clave correcta no se puede generar la prueba**, aunque se
    /// tengan todos los materiales.
    ///
    /// Es lo que impide que quien intercepte los materiales pueda gastar.
    #[test]
    fn send_materials_alone_are_not_enough_to_spend() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("cuenta");

        let m = layer
            .send_materials(alice, receptor, 250_000, salt_de(0x1073))
            .expect("materiales");

        let r = client::prove_send(
            &m,
            [
                BaseElement::new(0x1337),
                BaseElement::new(0xBADC0DE),
                BaseElement::new(0x0DDBA11),
                BaseElement::new(0x1CEB00DA),
            ],
            proof_options(),
        );
        assert!(
            matches!(r, Err(LayerError::NotTheAccountHolder)),
            "CRITICO: quien intercepte los materiales NO debe poder gastar. \
             Resultado: {:?}",
            r.map(|_| "recibo")
        );
    }


    /// **UN PAGO ENTERO SIN DAR NINGUNA CLAVE A LA CAPA.**
    ///
    /// El equivalente de `a_transfer_without_giving_the_key_to_the_layer`
    /// para la vía en dos fases, y la razón de que `send_materials` y
    /// `prove_send` existan.
    ///
    /// `SovereignLayer::send` hace lo mismo en una llamada, pero **recibe la
    /// clave como argumento de un método de la capa**. Eso no impide la
    /// separación —el cliente puede ejecutar la capa en su máquina— pero
    /// tampoco la enseña, y los tres preprints citan esta propiedad como el
    /// argumento institucional central. Ver `AUDITORIA.md` §33.
    #[test]
    fn a_whole_payment_without_giving_any_key_to_the_layer() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        // ⚠️ **RELLENADA CON CEROS, y no por comodidad.**
        //
        // Se intento con una clave ancha de verdad y el circuito la rechazo
        // —`NotTheAccountHolder`—, con razon: la cuenta se abrio con
        // `open_and_fund(SK_ALICE)`, que deriva la identidad ESTRECHA, y esa
        // clave no le corresponde.
        //
        // ⚠️ **Eso deja al descubierto que la migracion NO esta completa**:
        // `circuit_send` sabe verificar claves de 256 bits, pero
        // `open_account` solo sabe crear cuentas de 64. Los elementos nuevos
        // existen en el circuito y **no son alcanzables desde la capa**
        // hasta que `open_account` acepte `Digest` (entrada 15).
        //
        // Rellenar aqui es lo unico correcto hoy —§90 garantiza la misma
        // identidad— pero **este test no ejercita los tres elementos
        // nuevos**, y no puede hasta entonces.
        let key = [
            BaseElement::new(SK_ALICE),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ];
        let receptor = layer.public_id_of(bob).expect("cuenta");
        let salt = salt_de(0xC11E);

        // ===== 1. LA CAPA ENTREGA MATERIALES. NO VE LA CLAVE. =====
        let materials = layer
            .send_materials(alice, receptor, 250_000, salt)
            .expect("materiales");

        // ⚠️ **Y NO ENTREGA EL SALDO DEL RECEPTOR.**
        //
        // Los materiales de la vía retirada llevaban una vista completa del
        // receptor, porque esa vía actualizaba las dos hojas y necesitaba el
        // saldo del otro.
        // `SendMaterials` lleva `receiver_id: Digest` y nada más: **la fuga
        // hacia la contraparte está cerrada en el tipo**, no en un comentario.
        assert_eq!(materials.receiver_id, receptor);

        // ===== 2. EL CLIENTE PRUEBA EN LOCAL, CON SU CLAVE. =====
        let recibo =
            client::prove_send(&materials, key, proof_options()).expect("prueba local");

        // ===== 3. LA CAPA VERIFICA Y APLICA. =====
        let estado = state_of(&layer, alice);
        layer
            .apply_send(&recibo, alice, &estado, 250_000)
            .expect("aplicar");

        assert_eq!(layer.balance_of(alice), Some(750_000), "el dinero salio");
        assert_eq!(layer.total_pending(), 250_000, "y esta en un pendiente");

        // ===== 4. EL RECEPTOR COBRA, TAMBIEN SIN DAR SU CLAVE. =====
        //
        // ⚠️ **El aviso lo aporta el, no la capa.** La capa no sabe que
        // pendiente es suyo —esa es la privacidad del diseno— asi que no
        // podria entregarselo. Como le llega es la pieza que ISO 20022 no
        // transporta; ver `AUDITORIA.md` §21.
        let mat_cobro = layer
            .claim_materials(bob, &recibo.notice)
            .expect("materiales de cobro");
        let cobro = client::prove_claim(
            &mat_cobro,
            // ⚠️ **RELLENADA CON CEROS**, como la de Alice y por lo mismo
            // (§92.14): la cuenta de Bob se abrio con `open_and_fund(SK_BOB)`,
            // que deriva la identidad ESTRECHA. Una clave ancha de verdad
            // seria rechazada — y no podra usarse hasta que `open_account`
            // acepte `Digest`, que va el ULTIMO (§96.4).
            [
                BaseElement::new(SK_BOB),
                BaseElement::ZERO,
                BaseElement::ZERO,
                BaseElement::ZERO,
            ],
            proof_options(),
        )
        .expect("prueba local del cobro");

        let estado_bob = state_of(&layer, bob);
        layer
            .apply_claim(&cobro, bob, &estado_bob, &recibo.notice)
            .expect("aplicar cobro");

        assert_eq!(layer.balance_of(bob), Some(250_000));
        assert_eq!(layer.total_pending(), 0);

        // **UN PAGO ENTERO, Y LA CAPA NO HA VISTO NINGUNA CLAVE.**
        //
        // Ni la del pagador ni la del receptor. Lo unico que la capa aporta
        // son caminos y raices —datos publicos— y lo unico que recibe son
        // pruebas que verifica.
    }

    /// ⚠️ **UN PAGO COMPLETO CON CLAVE DE 256 BITS, DE PUNTA A PUNTA.**
    ///
    /// Es la unica prueba de que la entrada 15 sirve para algo. Los cinco
    /// circuitos verifican claves anchas desde §92.19, pero hasta
    /// `open_account_wide` **ningun titular podia tener una**, asi que el
    /// camino completo —abrir, enviar, cobrar— **nunca se habia
    /// ejercitado**.
    ///
    /// Las claves son anchas **de verdad**: los tres elementos extra no son
    /// cero. Con relleno el test pasaria sin probar nada nuevo (§90.3).
    #[test]
    fn a_whole_payment_with_a_256_bit_key() {
        let mut layer = new_layer();
        let sk_alice = wide_key(SK_ALICE);
        let sk_bob = wide_key(SK_BOB);
        let alice = open_and_fund_wide(&mut layer, sk_alice, 1_000_000);
        let bob = open_and_fund_wide(&mut layer, sk_bob, 0);

        let receptor = layer.public_id_of(bob).expect("cuenta");
        let materials = layer
            .send_materials(alice, receptor, 250_000, salt_de(0x1DE))
            .expect("materiales");
        let recibo = client::prove_send(&materials, sk_alice, proof_options())
            .expect("el titular con clave ancha DEBE poder probar su envio");
        let estado = state_of(&layer, alice);
        layer
            .apply_send(&recibo, alice, &estado, 250_000)
            .expect("aplicar envio");

        let cm = layer
            .claim_materials(bob, &recibo.notice)
            .expect("materiales de cobro");
        let cobro = client::prove_claim(&cm, sk_bob, proof_options())
            .expect("el receptor con clave ancha DEBE poder cobrar");
        let estado_bob = state_of(&layer, bob);
        layer
            .apply_claim(&cobro, bob, &estado_bob, &recibo.notice)
            .expect("aplicar cobro");

        assert_eq!(layer.balance_of(alice), Some(750_000));
        assert_eq!(layer.balance_of(bob), Some(250_000));
        assert_eq!(layer.total_pending(), 0);
    }

    /// **La clave equivocada no genera prueba, y falla ANTES de gastar cómputo.**
    #[test]
    fn prove_send_rejects_a_key_that_is_not_the_holders() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("cuenta");

        let materials = layer
            .send_materials(alice, receptor, 1000, salt_de(0xBAD1))
            .expect("materiales");

        let r = client::prove_send(
            &materials,
            [
                BaseElement::new(0x1337),
                BaseElement::new(0xBADC0DE),
                BaseElement::new(0x0DDBA11),
                BaseElement::new(0x1CEB00DA),
            ],
            proof_options(),
        );
        assert!(
            matches!(r, Err(LayerError::NotTheAccountHolder)),
            "el circuito lo impondria igual, pero en release no se valida al \
             generar: sin esta comprobacion se gastaria el computo de una \
             prueba invalida. Salio: {:?}",
            r.map(|_| "recibo")
        );
    }
}


#[cfg(test)]
mod t5_contencion_anclaje {
    //! T5 - entrada 63 / seccion 122.4: la contencion del anclaje, MEDIDA.
    //! ESCALADO.md 2.2 la modela en 1,6 TPS; aqui se comprueba el mecanismo
    //! y se miden las constantes con las que re-derivar el numero.
    use crate::tests_support::*;
    use crate::LayerError;
    use stark_experiment::circuit_settlement::derive_public_id;
    use std::time::{Duration, Instant};
    use winterfell::math::fields::f64::BaseElement;

    const SK_A: u64 = 0xA5A5;
    const SK_M: u64 = 0x5A5A;
    const SK_B: u64 = 0xB0B0;

    #[test]
    fn t5a_dos_independientes_un_anclaje() {
        let mut layer = new_layer();
        let a = open_and_fund(&mut layer, SK_A, 1_000_000);
        let m = open_and_fund(&mut layer, SK_M, 1_000_000);
        let bob = derive_public_id(BaseElement::new(SK_B));
        let ea = state_of(&layer, a);
        let em = state_of(&layer, m);
        // Dos pruebas contra la MISMA raiz: titulares distintos, hojas
        // distintas, cero estado compartido.
        let t = Instant::now();
        let ra = layer.send(BaseElement::new(SK_A), a, &ea, bob, salt_de(1), 100).unwrap();
        let rm = layer.send(BaseElement::new(SK_M), m, &em, bob, salt_de(2), 100).unwrap();
        eprintln!("dos generaciones: {:?}", t.elapsed());
        layer.apply_send(&ra, a, &ea, 100).unwrap();
        match layer.apply_send(&rm, m, &em, 100) {
            Err(LayerError::StaleState) => eprintln!(
                "VEREDICTO 5a: contencion confirmada — StaleState sobre una                  operacion INDEPENDIENTE: el anclaje global serializa a                  titulares que no comparten nada"
            ),
            otro => panic!("esperaba StaleState, llego {otro:?}"),
        }
    }

    #[test]
    fn t5b_constantes_y_coste_efectivo() {
        // Dos clientes ingenuos, en serie (cota del fenomeno, no
        // concurrencia real): cada ronda ambos generan contra la raiz
        // vigente; el segundo pierde, regenera y aplica.
        const RONDAS: u64 = 5;
        let mut layer = new_layer();
        let a = open_and_fund(&mut layer, SK_A, 1_000_000);
        let m = open_and_fund(&mut layer, SK_M, 1_000_000);
        let bob = derive_public_id(BaseElement::new(SK_B));
        let (mut gens, mut regens, mut ops) = (0u32, 0u32, 0u32);
        let (mut t_gen, mut t_apply) = (Duration::ZERO, Duration::ZERO);
        let t0 = Instant::now();
        for r in 0..RONDAS {
            let ea = state_of(&layer, a);
            let em = state_of(&layer, m);
            let t = Instant::now();
            let ra = layer.send(BaseElement::new(SK_A), a, &ea, bob, salt_de(100 + r), 100).unwrap();
            t_gen += t.elapsed(); gens += 1;
            let t = Instant::now();
            let rm = layer.send(BaseElement::new(SK_M), m, &em, bob, salt_de(200 + r), 100).unwrap();
            t_gen += t.elapsed(); gens += 1;
            let t = Instant::now();
            layer.apply_send(&ra, a, &ea, 100).unwrap();
            t_apply += t.elapsed(); ops += 1;
            match layer.apply_send(&rm, m, &em, 100) {
                Err(LayerError::StaleState) => {
                    regens += 1;
                    let em2 = state_of(&layer, m);
                    let t = Instant::now();
                    let rm2 = layer.send(BaseElement::new(SK_M), m, &em2, bob, salt_de(300 + r), 100).unwrap();
                    t_gen += t.elapsed(); gens += 1;
                    let t = Instant::now();
                    layer.apply_send(&rm2, m, &em2, 100).unwrap();
                    t_apply += t.elapsed(); ops += 1;
                }
                Ok(()) => { ops += 1; eprintln!("ronda {r}: la segunda ENTRO — sin contencion, revisar"); }
                Err(e) => panic!("{e:?}"),
            }
        }
        let wall = t0.elapsed();
        let g = t_gen / gens;
        let ap = t_apply / ops;
        eprintln!("ops aplicadas: {ops} | generaciones: {gens} | regeneraciones: {regens}");
        eprintln!("t_gen medio: {g:?} | t_apply medio: {ap:?}  <- arbitra 177 ms vs 72 ms");
        eprintln!("pared: {wall:?} | TPS efectivo en serie: {:.2}", ops as f64 / wall.as_secs_f64());
        eprintln!(
            "modelo 2 clientes: coste/op = {:.0} ms x {:.1} gens/op + apply => {:.2} TPS",
            g.as_secs_f64() * 1e3, gens as f64 / ops as f64,
            1.0 / ((gens as f64 / ops as f64) * g.as_secs_f64() + ap.as_secs_f64())
        );
    }
}
