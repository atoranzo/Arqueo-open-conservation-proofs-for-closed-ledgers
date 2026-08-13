//! # El contador de recepción
//!
//! Un `u64` monótono y **persistido** que dice **en qué orden llegó cada
//! operación que el nodo se puso a evaluar**.
//!
//! ## ⚠️ Por qué `seq` no vale
//!
//! `seq` es **el orden de aplicación**: sale de `entries.len()` dentro de
//! `append`, y **solo existe si la operación se aplicó**. El camino real
//! tiene cuatro etapas:
//!
//! ```text
//!   parse(params)   →  try_into()  →  apply_send(...)  →  append() → seq
//!   ↑ ruido            ↑ el cable     ↑ la capa            ↑ AQUÍ
//! ```
//!
//! > **La censura vive en el hueco entre recibir y aplicar**, y hoy ese
//! > tramo no deja ninguna huella. Un operador que censura simplemente **no
//! > incrementa `seq`**, y nada lo delata.
//!
//! ## Qué cuenta, exactamente
//!
//! **Lo que el nodo llegó a EVALUAR.** Ni lo que llegó al puerto, ni lo que
//! aceptó:
//!
//! | falla en | ¿consume contador? | por qué |
//! |---|---|---|
//! | el parseo o el cable | **NO** | eso es ruido, no una operación. Si contara, **cualquiera podría abrir huecos en el registro ajeno mandando basura** |
//! | la capa (prueba inválida, raíz movida) | **SÍ** | el nodo **verificó una operación de verdad** y decidió no aplicarla — y **ahí es donde se escondería un censor**: rechazar alegando prueba inválida |
//!
//! ## ⚠️ El detector, y por qué es inmune a la congestión
//!
//! *«Una operación posterior entró y la mía no»* es **reordenación**, y no
//! hace falta elegir ningún umbral `N` para verla:
//!
//! > **La congestión retrasa a todos; solo la censura adelanta.**
//!
//! ## ⚠️⚠️ LO QUE ESTO **NO** ES: evidencia oponible
//!
//! El contador es **un número que el nodo dice y que nada ata**.
//! `chain_digest` autentica `seq`, `kind`, las dos raíces, el digest de
//! prueba y el anterior — **y nada más**. Meterlo ahí rompería la
//! conformidad de `zkssl/0.2` y la propiedad retroactiva de §115.
//!
//! Así que:
//!
//! - **Dos titulares que cooperan DETECTAN la reordenación**: A tiene
//!   recepción 100, B tiene 101, la de B está en el log y la de A no.
//! - **No pueden PROBARLA.** Ninguno tiene nada firmado por el operador que
//!   diga *«recibí la tuya la 100»*. **El operador puede negar haberlo
//!   dicho.**
//!
//! > **Esto detecta entre partes que cooperan; no produce evidencia
//! > oponible.** Lo que falta es **el acuse como hoja bajo una raíz en la
//! > cabeza** (entrada 62), que hereda la firma **sin gastar índices XMSS**.
//!
//! **No cierra la censura: la prepara.** Venderlo como detector oponible
//! sería el mismo falso «limpio» que §250 encontró en el auditor, una capa
//! más arriba.
//!
//! ## ⚠️ Y por eso persiste desde el principio
//!
//! Un contador que se reinicia en silencio **es peor que no tenerlo**: sin
//! contador, el titular sabe que no tiene detector; con uno que vuelve a
//! cero, tiene un detector que dice *«todo en orden»* mientras **dos
//! operaciones distintas llevan el mismo número**.
//!
//! ⚠️ Y hay un agravante: §242 declaró que **habrá huecos por reinicio** —la
//! firma vive en memoria— y el testigo los clasifica como **benignos**. Si
//! el contador también se reiniciara, **el reinicio dejaría de ser un hueco
//! benigno y pasaría a producir colisiones activas**. Dos límites que por
//! separado son declarables, **juntos rompen la propiedad**.

use std::path::Path;

use zk_ssl_guardian::{GuardianError, GuardianIndice};

/// Cuenta, con `fsync`, las operaciones que el nodo **llega a evaluar**.
///
/// ⚠️ Reusa [`GuardianIndice`] **entero** en vez de reimplementarlo: la
/// persistencia con `fsync`, la comprobación de que el medio persiste de
/// verdad (K.1, §234: **se niega a operar en `tmpfs`**) y la reconciliación
/// ya están medidas y probadas. Dos implementaciones del mismo problema
/// pueden discrepar.
pub struct ContadorRecepcion {
    guardian: GuardianIndice,
}

impl ContadorRecepcion {
    /// ⚠️ **Se niega a arrancar si el medio no persiste.** Es la propiedad
    /// que §234 midió —ext4 `fsync` 0,907 ms frente a 1× en `tmpfs`— y que
    /// en §247 evitó un daño real: sin ella, un reinicio reusaría números.
    pub fn abrir(ruta: impl AsRef<Path>) -> Result<Self, GuardianError> {
        Ok(ContadorRecepcion { guardian: GuardianIndice::abrir(ruta)? })
    }

    /// Reserva el siguiente número **con `fsync` ANTES de devolverlo**.
    ///
    /// ⚠️ Ese orden importa por la misma razón que en §234: si el proceso
    /// muere en medio, queda **un número quemado sin operación** —el caso
    /// seguro—. Lo contrario, devolver un número que no persistió, **lo
    /// reusaría en el siguiente arranque**.
    pub fn recibir(&mut self) -> Result<u64, GuardianError> {
        self.guardian.reservar()
    }

    /// Cuántas operaciones ha evaluado el nodo, en toda su vida.
    pub fn actual(&self) -> u64 {
        self.guardian.actual()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn en_disco(nombre: &str) -> std::path::PathBuf {
        let d = crate::tests_dir(nombre);
        d.join("recepcion.bin")
    }

    #[test]
    fn el_contador_avanza_de_uno_en_uno() {
        let mut c = ContadorRecepcion::abrir(en_disco("rec_avanza")).expect("abrir");
        assert_eq!(c.actual(), 0, "empieza en cero");
        for esperado in 1..=5u64 {
            assert_eq!(c.recibir().expect("recibir"), esperado);
            assert_eq!(c.actual(), esperado);
        }
    }

    #[test]
    fn sobrevive_al_reinicio_y_no_reusa_numeros() {
        // ⚠️⚠️ LA PROPIEDAD DE §253. Un contador que vuelve a cero da un
        // detector que dice "todo en orden" mientras DOS OPERACIONES
        // DISTINTAS LLEVAN EL MISMO NUMERO — peor que no tenerlo.
        let p = en_disco("rec_reinicio");
        let mut primero = ContadorRecepcion::abrir(&p).expect("abrir");
        for _ in 0..7 {
            primero.recibir().expect("recibir");
        }
        assert_eq!(primero.actual(), 7);
        drop(primero);

        let mut segundo = ContadorRecepcion::abrir(&p).expect("reabrir");
        assert_eq!(segundo.actual(), 7, "el reinicio NO puede volver a cero");
        assert_eq!(segundo.recibir().expect("recibir"), 8, "sigue donde estaba");
    }

    #[test]
    fn el_numero_esta_en_disco_antes_de_devolverse() {
        // ⚠️ El orden de §234: `fsync` ANTES. Si muriera en medio, queda un
        // numero QUEMADO —el caso seguro—, no uno reutilizable.
        let p = en_disco("rec_fsync");
        let mut c = ContadorRecepcion::abrir(&p).expect("abrir");
        let n = c.recibir().expect("recibir");
        let en_disco = std::fs::read(&p).expect("leer el contador");
        assert_eq!(
            u64::from_le_bytes(en_disco.try_into().expect("8 bytes")),
            n,
            "CRITICO: el numero se devolvio sin persistir"
        );
    }

    #[test]
    fn dos_contadores_distintos_no_se_pisan() {
        // El de recepcion y el de firma (§234) son ficheros distintos:
        // compartirlos mezclaria dos secuencias con significados distintos.
        let a = en_disco("rec_a");
        let b = en_disco("rec_b");
        let mut ca = ContadorRecepcion::abrir(&a).expect("abrir a");
        let mut cb = ContadorRecepcion::abrir(&b).expect("abrir b");
        ca.recibir().expect("a");
        ca.recibir().expect("a");
        cb.recibir().expect("b");
        assert_eq!(ca.actual(), 2);
        assert_eq!(cb.actual(), 1, "cada contador lleva SU cuenta");
    }
}
