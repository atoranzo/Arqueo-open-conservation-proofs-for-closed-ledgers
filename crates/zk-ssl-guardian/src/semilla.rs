//! **La semilla del firmante: leida, comprobada y descodificada en UN sitio.**
//!
//! ⚠️⚠️ Hasta el §330 esto vivia en `main.rs` del nodo —un BINARIO—, asi que el
//! testigo **no podia reutilizarlo aunque quisiera**: hacia `fs::read` y le
//! pasaba a `from_seed` lo que hubiera en el fichero, **sin mirar permisos ni
//! longitud**, mientras su propio doc llamaba a eso «material de clave».
//!
//! ⚠️ **Los DOS FORMATOS siguen vivos y son del proyecto**: el nodo lee HEX
//! (`--clave-fichero`), el testigo BINARIO crudo (`--cofirmar`), declarado en el
//! asiento del §301. Lo que deja de ser distinto es **lo que se comprueba**.
//!
//! ⚠️ El orden importa: **permisos y longitud ANTES de abrir el guardian**, que
//! crea el fichero del contador al abrirse. Un arranque que muere por la semilla
//! no debe haber dejado nada escrito.

use crate::GuardianError;
use std::path::Path;

/// Longitud de la semilla XMSS: **96 = 3×32** (`SK_SEED ‖ SK_PRF ‖ PUB_SEED`).
///
/// No es una eleccion de esta casa: `xmss` la exige y la rechaza con
/// `InvalidSeedLength` si no cuadra (leido en `xmss-0.1.0-pre.0`,
/// `params.rs:1177`). Aqui se comprueba **antes**, para poder decir QUE formato
/// se esperaba —upstream solo sabe de longitudes—.
pub const SEMILLA_LEN: usize = 96;

/// Un fichero de material de clave no puede ser legible por grupo ni por otros.
///
/// ⚠️ Crear con `0600` no impide que alguien afloje despues: **se comprueba al
/// LEER**, no al escribir. El keystore de §199 creaba bien y nadie miraba.
pub fn comprobar_permisos(ruta: &Path) -> Result<(), GuardianError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let modo = std::fs::metadata(ruta)
            .map_err(|e| GuardianError::Io(e.to_string()))?
            .permissions()
            .mode()
            & 0o777;
        if modo & 0o077 != 0 {
            return Err(GuardianError::PermisosAbiertos {
                ruta: ruta.display().to_string(),
                modo,
            });
        }
    }
    #[cfg(not(unix))]
    {
        let _ = ruta;
    }
    Ok(())
}

/// ¿Son estos bytes la representacion HEX de una semilla?
///
/// Sirve **solo para que el error diga que se ha confundido**: 192 caracteres,
/// todos digitos hexadecimales, es casi con seguridad el fichero del NODO
/// entregado al testigo. Es el error que de verdad comete la gente.
fn parece_hex(bytes: &[u8]) -> bool {
    bytes.len() == SEMILLA_LEN * 2 && bytes.iter().all(|b| b.is_ascii_hexdigit())
}

/// Descodifica una semilla en hexadecimal. Acepta `0x` delante y blancos
/// alrededor.
///
/// ⚠️ El error cuenta **CARACTERES, no bytes derivados**: con division entera,
/// un hex de 193 caracteres decia «y tiene 96», que es exactamente la cifra que
/// estaba exigiendo. Se reporta lo medido.
pub fn descodificar_hex(hex: &str) -> Result<Vec<u8>, GuardianError> {
    let h = hex.trim().trim_start_matches("0x");
    if h.len() != SEMILLA_LEN * 2 {
        return Err(GuardianError::SemillaHexLongitud {
            esperado_car: SEMILLA_LEN * 2,
            encontrado_car: h.len(),
        });
    }
    (0..SEMILLA_LEN)
        .map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|e| GuardianError::SemillaNoHex {
            detalle: e.to_string(),
        })
}

/// Lee una semilla en **HEX** de un fichero, comprobando antes los permisos.
pub fn leer_hex(ruta: &Path) -> Result<Vec<u8>, GuardianError> {
    comprobar_permisos(ruta)?;
    let texto = std::fs::read_to_string(ruta).map_err(|e| GuardianError::Io(e.to_string()))?;
    descodificar_hex(&texto)
}

/// Lee una semilla en **BINARIO CRUDO** de un fichero, comprobando antes los
/// permisos.
pub fn leer_cruda(ruta: &Path) -> Result<Vec<u8>, GuardianError> {
    comprobar_permisos(ruta)?;
    let bytes = std::fs::read(ruta).map_err(|e| GuardianError::Io(e.to_string()))?;
    if bytes.len() != SEMILLA_LEN {
        return Err(GuardianError::SemillaLongitud {
            esperado: SEMILLA_LEN,
            encontrado: bytes.len(),
            parece_hex: parece_hex(&bytes),
        });
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un directorio propio por test: el crate no tiene dependencias, asi que
    /// nada de `tempfile`.
    fn dir(nombre: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!(
            "zkssl-semilla-{}-{}",
            nombre,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("crear dir");
        d
    }

    fn escribir(d: &Path, nombre: &str, bytes: &[u8]) -> std::path::PathBuf {
        let p = d.join(nombre);
        std::fs::write(&p, bytes).expect("escribir");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600))
                .expect("chmod 600");
        }
        p
    }

    fn hex_de_prueba() -> String {
        "5b".repeat(SEMILLA_LEN)
    }

    fn cruda_de_prueba() -> Vec<u8> {
        (0..SEMILLA_LEN).map(|i| ((i * 31 + 9) % 256) as u8).collect()
    }

    #[test]
    fn el_hex_de_192_caracteres_da_96_bytes() {
        let b = descodificar_hex(&hex_de_prueba()).expect("hex valido");
        assert_eq!(b.len(), SEMILLA_LEN);
        assert!(b.iter().all(|x| *x == 0x5b));
    }

    #[test]
    fn el_hex_acepta_el_0x_y_los_blancos_de_alrededor() {
        let con_adornos = format!("  0x{}\n", hex_de_prueba());
        assert_eq!(
            descodificar_hex(&con_adornos).expect("hex valido"),
            descodificar_hex(&hex_de_prueba()).expect("hex valido")
        );
    }

    /// ⚠️ El defecto que este sello repara: con division entera, 193 caracteres
    /// reportaban «96», que es la cifra que se estaba exigiendo. El mensaje se
    /// contradecia a si mismo.
    #[test]
    fn un_hex_de_193_caracteres_no_dice_96() {
        let mal = format!("{}a", hex_de_prueba());
        match descodificar_hex(&mal) {
            Err(e @ GuardianError::SemillaHexLongitud { .. }) => {
                let msg = format!("{e}");
                assert!(msg.contains("193"), "tiene que decir lo MEDIDO: {msg}");
                assert!(
                    !msg.contains("y tiene 96"),
                    "no puede decir que tiene justo lo que exige: {msg}"
                );
            }
            otro => panic!("debia ser SemillaHexLongitud y dio: {otro:?}"),
        }
    }

    #[test]
    fn un_caracter_que_no_es_hexadecimal_se_rechaza_sin_reventar() {
        let mut mal = hex_de_prueba();
        mal.replace_range(0..1, "z");
        assert!(matches!(
            descodificar_hex(&mal),
            Err(GuardianError::SemillaNoHex { .. })
        ));
    }

    #[test]
    fn una_semilla_cruda_de_96_bytes_pasa() {
        let d = dir("cruda-ok");
        let p = escribir(&d, "semilla.bin", &cruda_de_prueba());
        assert_eq!(leer_cruda(&p).expect("cruda valida"), cruda_de_prueba());
        let _ = std::fs::remove_dir_all(&d);
    }

    /// ⚠️⚠️ **El error que de verdad comete la gente**: darle a `--cofirmar` el
    /// fichero HEX del nodo. Antes del §330 esto llegaba a `from_seed` y salia
    /// un volcado `Debug` de upstream que no nombraba ningun formato.
    #[test]
    fn el_hex_del_nodo_entregado_al_testigo_dice_que_parece_hex() {
        let d = dir("hex-al-testigo");
        let p = escribir(&d, "semilla-nodo.hex", hex_de_prueba().as_bytes());
        match leer_cruda(&p) {
            Err(e @ GuardianError::SemillaLongitud { .. }) => {
                let msg = format!("{e}");
                assert!(msg.contains("CRUDOS"), "debe decir que quiere crudos: {msg}");
                assert!(msg.contains("HEX"), "y que le han dado hex: {msg}");
            }
            otro => panic!("debia ser SemillaLongitud y dio: {otro:?}"),
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    /// ⚠️ Los dos formatos son **dos productores del mismo contrato**: se atan
    /// con un test, no con prosa.
    #[test]
    fn los_dos_formatos_dan_exactamente_los_mismos_bytes() {
        let d = dir("atado");
        let crudos = cruda_de_prueba();
        let hex: String = crudos.iter().map(|b| format!("{b:02x}")).collect();
        let ph = escribir(&d, "s.hex", hex.as_bytes());
        let pb = escribir(&d, "s.bin", &crudos);
        assert_eq!(
            leer_hex(&ph).expect("hex"),
            leer_cruda(&pb).expect("cruda"),
            "el mismo material en los dos formatos da los mismos bytes"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[cfg(unix)]
    #[test]
    fn un_fichero_legible_por_el_grupo_se_rechaza_al_leer() {
        use std::os::unix::fs::PermissionsExt;
        let d = dir("permisos");
        let p = escribir(&d, "semilla.bin", &cruda_de_prueba());
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o640)).expect("chmod 640");
        match leer_cruda(&p) {
            Err(e @ GuardianError::PermisosAbiertos { .. }) => {
                assert!(format!("{e}").contains("chmod 600"));
            }
            otro => panic!("un secreto legible por el grupo se rechaza y dio: {otro:?}"),
        }
        let _ = std::fs::remove_dir_all(&d);
    }
}
