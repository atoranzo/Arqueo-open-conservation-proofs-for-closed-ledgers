//! # El juez del umbral del medio (RFC-0013 D-D, §635)
//!
//! El kit verifica el sobre `ancla-cofirmada` y REPORTA qué testigos lo cofirman con las vkeys
//! que trae el propio sobre (§633): que un testigo VERIFIQUE no dice que VALGA, porque esas
//! claves las pone quien arma el sobre, que puede ser el operador. Este juez es la otra mitad,
//! la del cliente, como `--verificar-cofirmas --testigos --k` lo es para las cofirmas XMSS
//! (S319): con SU política —las vkeys de los testigos del medio en los que confía— cuenta los
//! testigos NOMBRADOS y distintos cuya cofirma verifica sobre el checkpoint de la nota, y por
//! debajo de `k` no acredita.
//!
//! ⚠️ **Composición, no sustitución.** El juez no rehace lo del kit —la cabeza, el atado del
//! publicador a la clave XMSS, la inclusión del ancla—: lo que acredita es el CHECKPOINT de la
//! nota. Una cabeza está anclada y cofirmada cuando el kit da VERDE sobre el sobre y este juez
//! lo acredita, los dos sobre el MISMO fichero. La nota sí se vuelve a verificar con su
//! publicador, porque una cofirma solo vale sobre el checkpoint que la nota dice.
//!
//! ⚠️ **La política entra por FUERA**, como en el S319: no del nodo ni del sobre. Las vkeys del
//! campo `testigos` del sobre se IGNORAN aquí.

use std::collections::BTreeSet;

use serde_json::Value;
use zk_ssl_medio::hash::sha256;
use zk_ssl_medio::nota::{verificar_nota, ClaveDeNota};

/// Las vkeys de los testigos del medio que el cliente acepta, leídas de un fichero: una por
/// línea, en el formato `nombre+keyid+base64(0x06 || clave)` de `signed-note`; se ignoran las
/// líneas en blanco y las que empiezan por `#`. Una vkey repetida es un error: la política
/// tendría un testigo que cuenta dos veces.
pub fn leer_politica(lineas: &[String]) -> Result<Vec<ClaveDeNota>, String> {
    let mut p: Vec<ClaveDeNota> = Vec::new();
    for (i, l) in lineas.iter().enumerate() {
        let t = l.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let k = ClaveDeNota::leer_vkey(t).map_err(|e| format!("linea {}: {e}", i + 1))?;
        if p.iter().any(|o| o.nombre() == k.nombre() && o.id() == k.id()) {
            return Err(format!("linea {}: {} ya esta en la politica", i + 1, k.nombre()));
        }
        p.push(k);
    }
    Ok(p)
}

/// La huella SHA-256 de una clave entera, en hex: lo que el kit imprime de cada testigo.
pub fn huella(k: &ClaveDeNota) -> String {
    sha256(k.bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// Lo que la política dice del checkpoint de un sobre.
#[derive(Debug)]
pub struct Juicio {
    pub origen: String,
    pub tamano: u64,
    /// `(nombre, huella de la clave, marca)` de cada testigo NOMBRADO cuya cofirma verifica.
    pub cofirman: Vec<(String, String, u64)>,
    /// Líneas de testigos nombrados que NO verifican: se descartan y se cuentan.
    pub no_verifican: usize,
    /// Líneas de claves que la política no nombra.
    pub no_nombradas: usize,
    /// Segundas líneas de un testigo nombrado que ya cuenta: no cuentan dos veces.
    pub repetidas: usize,
}

impl Juicio {
    /// ⚠️ FALLA CERRADA: `k = 0` no es una política —acreditaría cualquier nota— y se rechaza
    /// antes de contar.
    pub fn acredita(&self, k: usize) -> Result<bool, String> {
        if k == 0 {
            return Err("k = 0 acreditaria cualquier nota: no es una politica".into());
        }
        Ok(self.cofirman.len() >= k)
    }
}

/// **Juzga el checkpoint de un sobre `ancla-cofirmada` con la política del cliente.**
pub fn juzgar(sobre: &Value, politica: &[ClaveDeNota]) -> Result<Juicio, String> {
    if sobre.get("tipo").and_then(|t| t.as_str()) != Some("ancla-cofirmada") {
        return Err("el fichero no es un sobre `tipo: \"ancla-cofirmada\"`".into());
    }
    let nota = sobre
        .get("nota")
        .and_then(|x| x.as_str())
        .ok_or("falta nota o no es cadena")?;
    let publicador = ClaveDeNota::leer_vkey(
        sobre
            .get("publicador")
            .and_then(|x| x.as_str())
            .ok_or("falta publicador o no es cadena")?,
    )
    .map_err(|e| format!("publicador: {e}"))?;
    let v = verificar_nota(nota, &publicador).map_err(|e| format!("la nota: {e}"))?;
    let mut j = Juicio {
        origen: v.checkpoint.origen.clone(),
        tamano: v.checkpoint.tamano,
        cofirman: Vec::new(),
        no_verifican: 0,
        no_nombradas: 0,
        repetidas: 0,
    };
    let mut contados = BTreeSet::new();
    for linea in &v.ajenas {
        let Some((i, k)) = politica
            .iter()
            .enumerate()
            .find(|(_, k)| k.nombre() == linea.nombre && k.id() == linea.id)
        else {
            j.no_nombradas += 1;
            continue;
        };
        match k.verificar_cofirma(&v.checkpoint, linea) {
            Err(_) => j.no_verifican += 1,
            Ok(_) if contados.contains(&i) => j.repetidas += 1,
            Ok(marca) => {
                contados.insert(i);
                j.cofirman.push((k.nombre().to_string(), huella(k), marca));
            }
        }
    }
    Ok(j)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un sobre del catálogo, firmado de verdad: la cabeza de un nodo real, la nota de su medio
    /// y las cofirmas de `testigo.invalid/uno` y `testigo.invalid/dos` (§634).
    fn sobre_de_dos() -> Value {
        serde_json::from_str(include_str!(
            "../../../spec/vectors/ancla-cofirmada/cofirmada-dos-testigos.json"
        ))
        .unwrap()
    }

    fn vkeys(s: &Value) -> Vec<String> {
        s["testigos"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn la_politica_cuenta_testigos_nombrados_y_falla_cerrada_bajo_k() {
        let s = sobre_de_dos();
        let todas = leer_politica(&vkeys(&s)).unwrap();
        let j = juzgar(&s, &todas).unwrap();
        assert_eq!(j.cofirman.len(), 2);
        assert!(j.origen.starts_with("zkssl/v1/"));
        assert!(j.acredita(2).unwrap());
        assert!(!j.acredita(3).unwrap());
        assert!(j.acredita(0).is_err(), "k = 0 no es una politica");
        // Con UNA sola vkey en la politica, la otra cofirma no esta nombrada.
        let una = leer_politica(&vkeys(&s)[..1]).unwrap();
        let j = juzgar(&s, &una).unwrap();
        assert_eq!((j.cofirman.len(), j.no_nombradas), (1, 1));
        assert!(j.acredita(1).unwrap() && !j.acredita(2).unwrap());
        // Sin politica, nada acredita: las vkeys del SOBRE no cuentan.
        let j = juzgar(&s, &[]).unwrap();
        assert_eq!((j.cofirman.len(), j.no_nombradas), (0, 2));
        assert!(!j.acredita(1).unwrap());
    }

    #[test]
    fn una_cofirma_que_no_verifica_no_cuenta_y_una_repetida_tampoco() {
        let s: Value = serde_json::from_str(include_str!(
            "../../../spec/vectors/ancla-cofirmada/neg-cofirma-tocada.json"
        ))
        .unwrap();
        let j = juzgar(&s, &leer_politica(&vkeys(&s)).unwrap()).unwrap();
        assert_eq!((j.cofirman.len(), j.no_verifican), (0, 1));
        let s: Value = serde_json::from_str(include_str!(
            "../../../spec/vectors/ancla-cofirmada/neg-cofirma-repetida.json"
        ))
        .unwrap();
        let j = juzgar(&s, &leer_politica(&vkeys(&s)).unwrap()).unwrap();
        assert_eq!((j.cofirman.len(), j.repetidas), (1, 1));
        assert!(!j.acredita(2).unwrap(), "un testigo dos veces no son dos testigos");
    }

    #[test]
    fn la_politica_se_lee_estricta_y_la_nota_se_verifica() {
        let s = sobre_de_dos();
        let v = vkeys(&s);
        let con_comentarios = vec![
            "# los testigos del medio en los que confio".to_string(),
            String::new(),
            v[0].clone(),
        ];
        assert_eq!(leer_politica(&con_comentarios).unwrap().len(), 1);
        let e = leer_politica(&[v[0].clone(), v[0].clone()]).unwrap_err();
        assert!(e.contains("ya esta en la politica"), "{e}");
        let e = leer_politica(&["no es una vkey".to_string()]).map(|_| ()).unwrap_err();
        assert!(e.starts_with("linea 1:"), "{e}");
        // Una nota que no verifica con su publicador no se juzga: no hay checkpoint.
        let mala: Value = serde_json::from_str(include_str!(
            "../../../spec/vectors/ancla-cofirmada/neg-nota-firma-tocada.json"
        ))
        .unwrap();
        let e = juzgar(&mala, &leer_politica(&v).unwrap()).map(|_| ()).unwrap_err();
        assert!(e.starts_with("la nota:"), "{e}");
        let e = juzgar(&serde_json::json!({ "v": 1, "tipo": "ancla" }), &[])
            .map(|_| ())
            .unwrap_err();
        assert!(e.contains("no es un sobre"), "{e}");
    }
}
