// ═══════════════════════════════════════════════════════════════════════
//  §256 · LA INCLUSIÓN, VERIFICADA SIN EL NODO
//
//  Tres eslabones, y **el tercero es el que hace que valga**:
//
//      hoja + camino  →  raíz  →  cabeza firmada
//
//  ⚠️ **Una raíz suelta no prueba nada**: el operador puede servir la que
//  quiera. Lo que ata el recibo a algo que el operador **no puede cambiar
//  sin firmar** es que la raíz entre en un `epoch_digest` que él firmó.
//
//  ## ⚠️ Por qué esto NO reconstruye la hoja
//
//  En `CommitmentLayer`, `open(index, leaf)` recibe **la hoja ya calculada
//  por el cliente**, y **la capa no guarda saldos** —hay un test que lo
//  comprueba buscando el saldo entre todos sus bytes—.
//
//  La hoja es **un dato que el titular ya tiene**; el nodo solo confirma
//  haberla colocado. **Pedirle al nodo que la componga rompería la
//  propiedad central del proyecto.**
//
//  ## ⚠️ La forma es la que el ACUSE reutiliza
//
//  `verificar_inclusion` toma **la raíz como parámetro** en vez de asumir
//  cuál es. Para el acuse basta pasar `acuse_root` donde aquí va
//  `accounts_root`, y el acuse como hoja. **Nada más.**
//
//  ## Lo que un recibo de inclusión NO dice
//
//  Prueba que **la hoja estaba en el árbol**, no **qué significaba**. Si el
//  titular pierde `public_id`, `balance` o `nonce`, tiene la prueba de
//  inclusión de un dato **que ya no puede interpretar** — y eso es
//  correcto: es la misma propiedad que impide que la capa lo sepa.
// ═══════════════════════════════════════════════════════════════════════

// ⚠️ De `zk-ssl-hash` (§254-§255): **las primitivas de FORMATO que el nodo
//    y este verificador tienen que componer IGUAL**. Reusarlas —en vez de
//    reimplementarlas— es lo que impide que **divergan en silencio**.
use zk_ssl_hash::{epoch_digest, path_root, Digest};

/// Lo que un tercero necesita para comprobar una inclusión **sin el nodo**.
///
/// ⚠️ Los cinco campos de la cabeza van enteros **porque el titular tiene
/// que recomponer el `epoch_digest` él mismo**: si solo recibiera el digest
/// ya hecho, estaría creyéndose la palabra del operador otra vez.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReciboInclusion {
    /// Posición de la hoja en el árbol.
    pub indice: u64,
    /// El compromiso, **tal como el titular lo entregó**.
    pub hoja: Digest,
    /// Hermanos desde la hoja hasta la raíz.
    pub hermanos: Vec<Digest>,
    /// `true` = el nodo actual va a la derecha.
    pub derecha: Vec<bool>,
    // ── los cinco de la cabeza ──
    pub seq: u64,
    pub accounts_root: Digest,
    pub pending_root: Digest,
    pub frozen_root: Digest,
    pub chain_digest: Digest,
}

/// Lo que puede ir mal al comprobar una inclusión.
///
/// ⚠️ **Tres cosas distintas con tres significados distintos**, como en
/// §246 y §250: un camino descuadrado es **un recibo mal formado**; una
/// raíz que no sale es **una hoja que no estaba**; y un digest que no
/// coincide es **un recibo de OTRA cabeza**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InclusionError {
    /// `hermanos` y `derecha` no miden lo mismo.
    CaminoDescuadrado { hermanos: usize, derecha: usize },
    /// ⚠️ **La hoja no está en ese árbol**: subir el camino da otra raíz.
    RaizDistinta,
    /// ⚠️ **El recibo es de OTRA cabeza.** Los cinco campos componen un
    /// `epoch_digest` que no es el de la cabeza firmada que se le pasó.
    ///
    /// Sin esta comprobación, un operador serviría un recibo correcto de
    /// una época **en la que la hoja sí estaba**, para una cabeza en la que
    /// ya no. **Un recibo sin época es un recibo de cualquier época.**
    CabezaDistinta,
}

impl core::fmt::Display for InclusionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InclusionError::CaminoDescuadrado { hermanos, derecha } => write!(
                f,
                "camino descuadrado: {hermanos} hermanos y {derecha} orientaciones"
            ),
            InclusionError::RaizDistinta => {
                write!(f, "la hoja no esta en ese arbol: el camino da otra raiz")
            }
            InclusionError::CabezaDistinta => {
                write!(f, "el recibo es de OTRA cabeza")
            }
        }
    }
}

impl std::error::Error for InclusionError {}

/// **Comprueba una inclusión contra una cabeza firmada.**
///
/// El `epoch_digest` que se pasa es **el que salió de [`verificar_cabeza`]**
/// — es decir, uno que ya se sabe **firmado por la clave anclada**. Esta
/// función añade el eslabón que faltaba: que **esa** cabeza contiene **esta**
/// hoja.
///
/// ⚠️ **No recibe la raíz por separado del resto de la cabeza.** Si lo
/// hiciera, un operador podría dar una raíz cualquiera junto a un digest
/// legítimo, y las dos mitades no se compararían nunca.
pub fn verificar_inclusion(
    recibo: &ReciboInclusion,
    epoch_digest_firmado: Digest,
) -> Result<(), InclusionError> {
    if recibo.hermanos.len() != recibo.derecha.len() {
        return Err(InclusionError::CaminoDescuadrado {
            hermanos: recibo.hermanos.len(),
            derecha: recibo.derecha.len(),
        });
    }

    // ── 1 · la hoja sube hasta la raiz que el recibo declara ──
    let raiz = path_root(recibo.hoja, &recibo.hermanos, &recibo.derecha);
    if raiz != recibo.accounts_root {
        return Err(InclusionError::RaizDistinta);
    }

    // ── 2 · esa raiz compone la cabeza que se firmo ──
    // ⚠️ AQUI esta la diferencia entre un recibo y una promesa: sin este
    //    paso, la raiz seria un numero que el operador dice.
    let compuesto = epoch_digest(
        recibo.seq,
        recibo.accounts_root,
        recibo.pending_root,
        recibo.frozen_root,
        recibo.chain_digest,
    );
    if compuesto != epoch_digest_firmado {
        return Err(InclusionError::CabezaDistinta);
    }
    Ok(())
}

#[cfg(test)]
mod tests_inclusion {
    use super::*;

    use winter_math::fields::f64::BaseElement;

    fn d(n: u64) -> Digest {
        [
            BaseElement::new(n),
            BaseElement::new(n + 1),
            BaseElement::new(n + 2),
            BaseElement::new(n + 3),
        ]
    }

    /// Un recibo coherente: el camino sube de verdad, y la cabeza cuadra.
    fn recibo_bueno() -> (ReciboInclusion, Digest) {
        let hoja = d(100);
        let hermanos = vec![d(200), d(300), d(400)];
        let derecha = vec![false, true, false];
        let accounts_root = path_root(hoja, &hermanos, &derecha);
        let (seq, pending_root, frozen_root, chain_digest) = (42, d(500), d(600), d(700));
        let firmado = epoch_digest(seq, accounts_root, pending_root, frozen_root, chain_digest);
        let r = ReciboInclusion {
            indice: 5,
            hoja,
            hermanos,
            derecha,
            seq,
            accounts_root,
            pending_root,
            frozen_root,
            chain_digest,
        };
        (r, firmado)
    }

    #[test]
    fn un_recibo_coherente_verifica() {
        let (r, firmado) = recibo_bueno();
        assert_eq!(verificar_inclusion(&r, firmado), Ok(()));
    }

    // ── ⚠️ LOS NEGATIVOS. Un verificador que solo se ha visto ACEPTAR no
    //    esta probado: falta saber que RECHAZA cuando debe.

    #[test]
    fn un_hermano_alterado_no_verifica() {
        let (mut r, firmado) = recibo_bueno();
        r.hermanos[1] = d(999);
        assert_eq!(verificar_inclusion(&r, firmado), Err(InclusionError::RaizDistinta));
    }

    #[test]
    fn cambiar_la_posicion_no_verifica() {
        // ⚠️ Si el orden no importara, CUALQUIER hoja probaria CUALQUIER
        // posicion, y el recibo no diria nada.
        let (mut r, firmado) = recibo_bueno();
        r.derecha[0] = !r.derecha[0];
        assert_eq!(verificar_inclusion(&r, firmado), Err(InclusionError::RaizDistinta));
    }

    #[test]
    fn otra_hoja_no_verifica() {
        let (mut r, firmado) = recibo_bueno();
        r.hoja = d(101);
        assert_eq!(verificar_inclusion(&r, firmado), Err(InclusionError::RaizDistinta));
    }

    #[test]
    fn un_camino_mas_corto_no_verifica() {
        let (mut r, firmado) = recibo_bueno();
        r.hermanos.pop();
        r.derecha.pop();
        assert_eq!(verificar_inclusion(&r, firmado), Err(InclusionError::RaizDistinta));
    }

    #[test]
    fn un_camino_descuadrado_se_rechaza_antes_de_subirlo() {
        let (mut r, firmado) = recibo_bueno();
        r.derecha.pop();
        assert_eq!(
            verificar_inclusion(&r, firmado),
            Err(InclusionError::CaminoDescuadrado { hermanos: 3, derecha: 2 })
        );
    }

    #[test]
    fn un_recibo_de_otra_cabeza_no_verifica() {
        // ⚠️⚠️ EL CASO QUE JUSTIFICA EL TERCER ESLABON. El camino es
        // PERFECTO y la raiz sale: lo que falla es que esa raiz NO ES la de
        // la cabeza firmada. Sin este paso, un operador serviria un recibo
        // correcto de una epoca EN LA QUE LA HOJA SI ESTABA.
        let (r, _) = recibo_bueno();
        let otra = epoch_digest(43, r.accounts_root, r.pending_root, r.frozen_root, r.chain_digest);
        assert_eq!(verificar_inclusion(&r, otra), Err(InclusionError::CabezaDistinta));
        // y el camino, por su lado, era correcto:
        assert_eq!(path_root(r.hoja, &r.hermanos, &r.derecha), r.accounts_root);
    }

    #[test]
    fn una_raiz_declarada_que_no_es_la_firmada_no_verifica() {
        // ⚠️ Si `verificar_inclusion` recibiera la raiz APARTE de la cabeza,
        // las dos mitades nunca se compararian y esto pasaria.
        let (mut r, firmado) = recibo_bueno();
        let hoja_falsa = d(101);
        r.accounts_root = path_root(hoja_falsa, &r.hermanos, &r.derecha);
        r.hoja = hoja_falsa;
        // El camino sube bien hasta la raiz NUEVA...
        assert_eq!(path_root(r.hoja, &r.hermanos, &r.derecha), r.accounts_root);
        // ...pero esa raiz no compone la cabeza firmada.
        assert_eq!(verificar_inclusion(&r, firmado), Err(InclusionError::CabezaDistinta));
    }

    #[test]
    fn cada_campo_de_la_cabeza_cuenta() {
        // ⚠️ Si alguno no entrara en la composicion, el operador podria
        // cambiarlo sin que la cabeza firmada lo delatara.
        let (base, firmado) = recibo_bueno();
        for (nombre, mut r) in [
            ("seq", ReciboInclusion { seq: 43, ..base.clone() }),
            ("pending_root", ReciboInclusion { pending_root: d(501), ..base.clone() }),
            ("frozen_root", ReciboInclusion { frozen_root: d(601), ..base.clone() }),
            ("chain_digest", ReciboInclusion { chain_digest: d(701), ..base.clone() }),
        ] {
            r.indice = base.indice;
            assert_eq!(
                verificar_inclusion(&r, firmado),
                Err(InclusionError::CabezaDistinta),
                "cambiar {nombre} tiene que romper el recibo"
            );
        }
    }

    #[test]
    fn el_error_dice_que_paso_y_se_puede_leer() {
        // ⚠️ Un tipo de error lleva Debug, Display y Error desde que nace.
        let e = InclusionError::CabezaDistinta;
        assert!(format!("{e}").contains("OTRA cabeza"));
        let d = InclusionError::CaminoDescuadrado { hermanos: 3, derecha: 2 };
        assert!(format!("{d}").contains("3") && format!("{d}").contains("2"));
        let _: &dyn std::error::Error = &e;
    }
}
