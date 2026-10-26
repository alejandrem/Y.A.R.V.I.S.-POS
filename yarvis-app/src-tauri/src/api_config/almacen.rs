// ============================================================
// api_config::almacen — Almacenamiento cifrado de API keys.
//
// Las claves NO se guardan en texto plano. En Windows se cifran con
// DPAPI (CryptProtectData), que las ata a la cuenta de Windows actual:
// copiar api_keys.bin a otra máquina (o a otro usuario) no sirve para
// leerlas. En Linux/macOS no hay DPAPI; el respaldo es un archivo con
// permisos 0600 (solo el dueño puede leerlo).
//
// Formato de api_keys.bin:
//   "YARVIS-AK1" | esquema (1 byte) | carga
//   esquema 'D' = carga cifrada con DPAPI (Windows)
//   esquema 'P' = carga JSON en claro con permisos 0600 (Unix)
//
// Migración: si existe el antiguo api_keys.json en claro, se lee, se
// reescribe como api_keys.bin y se borra el JSON. El borrado es parte
// del guardado: si falla, la operación falla para no dejar una copia
// en texto plano silenciosa.
//
// LÍMITE HONESTO DE DPAPI: protege contra robo/copia del archivo y
// contra otros usuarios de la máquina. Un proceso que ya corre con la
// MISMA cuenta de Windows puede pedirle a DPAPI que descifre; para eso
// haría falta un keychain con aislamiento adicional (p. ej. Credential
// Manager con UI) o un secreto que no viva en la sesión. No prometemos
// más de lo que DPAPI da.
// ============================================================

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

const ARCHIVO: &str = "api_keys.bin";
const ARCHIVO_LEGACY: &str = "api_keys.json";
const MAGIA: &[u8] = b"YARVIS-AK1";
const ESQUEMA_DPAPI: u8 = b'D';
const ESQUEMA_PERMISOS: u8 = b'P';

#[cfg(windows)]
const ENTROPIA: &[u8] = b"Y.A.R.V.I.S.-POS/api_keys/v1";

/// Ruta del archivo cifrado dentro del directorio de datos de la app.
pub fn ruta_archivo(dir: &Path) -> PathBuf {
    dir.join(ARCHIVO)
}

pub fn ruta_legacy(dir: &Path) -> PathBuf {
    dir.join(ARCHIVO_LEGACY)
}

/// Lee el mapa de claves. Si no existe ningún archivo, devuelve un mapa
/// vacío (primera configuración). Migra transparentemente el JSON
/// antiguo en texto plano.
pub fn leer_mapa(dir: &Path) -> Result<HashMap<String, String>, String> {
    let ruta = ruta_archivo(dir);

    if ruta.exists() {
        let bytes =
            std::fs::read(&ruta).map_err(|e| format!("No se pudo leer {}: {e}", ruta.display()))?;
        let (esquema, carga) = desenvolver(&bytes)?;
        let plano = desproteger(esquema, carga)?;
        let mapa = parsear_json(&plano)?;

        // Si el archivo estaba en claro (p. ej. copiado desde Linux),
        // lo reescribimos cifrado en Windows. En Unix 'P' es lo normal.
        #[cfg(windows)]
        if esquema == ESQUEMA_PERMISOS {
            guardar_mapa(dir, &mapa)?;
        }
        #[cfg(not(windows))]
        let _ = esquema;

        // Un crash entre "escribir bin" y "borrar legacy" puede dejar
        // ambos. El bin manda, pero el JSON en claro no debe quedarse.
        eliminar_legacy(dir)?;
        return Ok(mapa);
    }

    let legacy = ruta_legacy(dir);
    if legacy.exists() {
        let contenido = std::fs::read_to_string(&legacy)
            .map_err(|e| format!("No se pudo leer {}: {e}", legacy.display()))?;
        let mapa = parsear_json(contenido.as_bytes())?;
        // guardar_mapa escribe api_keys.bin y borra api_keys.json.
        guardar_mapa(dir, &mapa)?;
        return Ok(mapa);
    }

    Ok(HashMap::new())
}

/// Guarda el mapa de claves cifrado y elimina cualquier copia legacy
/// en texto plano. La escritura es atómica (temporal + rename) para no
/// dejar un archivo a medias si la app se cierra a mitad.
pub fn guardar_mapa(dir: &Path, mapa: &HashMap<String, String>) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("No se pudo crear {}: {e}", dir.display()))?;

    let json = serde_json::to_vec(mapa)
        .map_err(|e| format!("No se pudo serializar la configuración de APIs: {e}"))?;
    let (esquema, carga) = proteger(&json)?;

    let mut bytes = Vec::with_capacity(MAGIA.len() + 1 + carga.len());
    bytes.extend_from_slice(MAGIA);
    bytes.push(esquema);
    bytes.extend_from_slice(&carga);

    escribir_atomico(&ruta_archivo(dir), &bytes)?;
    eliminar_legacy(dir)?;
    Ok(())
}

/// Protege un secreto corto (p. ej. la contraseña de los respaldos).
/// En Windows usa DPAPI; en Unix devuelve los bytes para que el
/// llamador los guarde en un archivo 0600. No incluye cabecera: el
/// formato del archivo lo decide el dueño del secreto.
pub fn proteger_secreto(datos: &[u8]) -> Result<Vec<u8>, String> {
    proteger(datos).map(|(_, carga)| carga)
}

/// Inversa de `proteger_secreto`. En Windows solo descifra DPAPI; un
/// archivo de otra cuenta/máquina falla con un mensaje entendible.
pub fn desproteger_secreto(datos: &[u8]) -> Result<Vec<u8>, String> {
    #[cfg(windows)]
    {
        descifrar_dpapi(datos)
    }
    #[cfg(not(windows))]
    {
        Ok(datos.to_vec())
    }
}

fn parsear_json(bytes: &[u8]) -> Result<HashMap<String, String>, String> {
    let parsed: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| format!("El archivo de API keys está corrupto: {e}"))?;
    let serde_json::Value::Object(mapa) = parsed else {
        return Err("El archivo de API keys debe ser un objeto JSON".to_string());
    };
    let mut out = HashMap::new();
    for (k, v) in mapa {
        if let Some(s) = v.as_str() {
            out.insert(k, s.to_string());
        }
    }
    Ok(out)
}

fn desenvolver(bytes: &[u8]) -> Result<(u8, &[u8]), String> {
    let cabecera = MAGIA.len() + 1;
    if bytes.len() < cabecera || &bytes[..MAGIA.len()] != MAGIA {
        return Err(
            "El archivo de API keys no tiene el formato esperado (¿está corrupto?)".to_string(),
        );
    }
    let esquema = bytes[MAGIA.len()];
    Ok((esquema, &bytes[cabecera..]))
}

fn eliminar_legacy(dir: &Path) -> Result<(), String> {
    let ruta = ruta_legacy(dir);
    match std::fs::remove_file(&ruta) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!(
            "Las claves se guardaron cifradas, pero no se pudo borrar el archivo en texto plano {}: {e}. Bórralo manualmente.",
            ruta.display()
        )),
    }
}

fn escribir_atomico(ruta: &Path, bytes: &[u8]) -> Result<(), String> {
    let dir = ruta
        .parent()
        .ok_or_else(|| "Ruta de API keys sin directorio padre".to_string())?;
    let sello = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temporal = dir.join(format!(".{ARCHIVO}.tmp-{}-{sello}", std::process::id()));

    let resultado = (|| -> Result<(), String> {
        let mut archivo = std::fs::File::create(&temporal)
            .map_err(|e| format!("No se pudo crear {}: {e}", temporal.display()))?;
        archivo
            .write_all(bytes)
            .map_err(|e| format!("No se pudo escribir {}: {e}", temporal.display()))?;
        archivo
            .sync_all()
            .map_err(|e| format!("No se pudo sincronizar {}: {e}", temporal.display()))?;
        drop(archivo);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temporal, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| format!("No se pudieron aplicar permisos 0600: {e}"))?;
        }

        std::fs::rename(&temporal, ruta).map_err(|e| {
            format!(
                "No se pudo reemplazar {} de forma atómica: {e}",
                ruta.display()
            )
        })
    })();

    if resultado.is_err() {
        let _ = std::fs::remove_file(&temporal);
    }
    resultado
}

#[cfg(windows)]
fn proteger(datos: &[u8]) -> Result<(u8, Vec<u8>), String> {
    Ok((ESQUEMA_DPAPI, cifrar_dpapi(datos)?))
}

#[cfg(not(windows))]
fn proteger(datos: &[u8]) -> Result<(u8, Vec<u8>), String> {
    Ok((ESQUEMA_PERMISOS, datos.to_vec()))
}

#[cfg(windows)]
fn desproteger(esquema: u8, datos: &[u8]) -> Result<Vec<u8>, String> {
    match esquema {
        ESQUEMA_DPAPI => descifrar_dpapi(datos),
        // Archivo venido de Unix: se acepta y leer_mapa lo reescribe
        // cifrado con DPAPI.
        ESQUEMA_PERMISOS => Ok(datos.to_vec()),
        otro => Err(format!(
            "Esquema de protección de API keys desconocido: {otro}"
        )),
    }
}

#[cfg(not(windows))]
fn desproteger(esquema: u8, datos: &[u8]) -> Result<Vec<u8>, String> {
    match esquema {
        ESQUEMA_PERMISOS => Ok(datos.to_vec()),
        ESQUEMA_DPAPI => Err(
            "El archivo de API keys fue cifrado con DPAPI de Windows y no se puede leer en este sistema"
                .to_string(),
        ),
        otro => Err(format!("Esquema de protección de API keys desconocido: {otro}")),
    }
}

#[cfg(windows)]
fn cifrar_dpapi(datos: &[u8]) -> Result<Vec<u8>, String> {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{
        CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let tam = u32::try_from(datos.len())
        .map_err(|_| "La configuración de APIs es demasiado grande para DPAPI".to_string())?;
    let entrada = CRYPT_INTEGER_BLOB {
        cbData: tam,
        pbData: datos.as_ptr() as *mut u8,
    };
    let entropia = CRYPT_INTEGER_BLOB {
        cbData: ENTROPIA.len() as u32,
        pbData: ENTROPIA.as_ptr() as *mut u8,
    };
    let mut salida = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptProtectData(
            &entrada,
            PCWSTR::null(),
            Some(&entropia),
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut salida,
        )
        .map_err(|e| format!("No se pudo cifrar la API key con DPAPI: {e}"))?;

        let cifrado = std::slice::from_raw_parts(salida.pbData, salida.cbData as usize).to_vec();
        let _ = LocalFree(HLOCAL(salida.pbData as *mut core::ffi::c_void));
        Ok(cifrado)
    }
}

#[cfg(windows)]
fn descifrar_dpapi(datos: &[u8]) -> Result<Vec<u8>, String> {
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let tam = u32::try_from(datos.len())
        .map_err(|_| "El archivo de API keys es demasiado grande para DPAPI".to_string())?;
    let entrada = CRYPT_INTEGER_BLOB {
        cbData: tam,
        pbData: datos.as_ptr() as *mut u8,
    };
    let entropia = CRYPT_INTEGER_BLOB {
        cbData: ENTROPIA.len() as u32,
        pbData: ENTROPIA.as_ptr() as *mut u8,
    };
    let mut salida = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptUnprotectData(
            &entrada,
            None,
            Some(&entropia),
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut salida,
        )
        .map_err(|e| {
            format!(
                "No se pudo descifrar la API key con DPAPI (¿archivo copiado de otra cuenta u otra máquina?): {e}"
            )
        })?;

        let plano = std::slice::from_raw_parts(salida.pbData, salida.cbData as usize).to_vec();
        let _ = LocalFree(HLOCAL(salida.pbData as *mut core::ffi::c_void));
        Ok(plano)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct DirTemporal(PathBuf);

    impl DirTemporal {
        fn nuevo(nombre: &str) -> Self {
            static CONTADOR: AtomicUsize = AtomicUsize::new(0);
            let n = CONTADOR.fetch_add(1, Ordering::SeqCst);
            let sello = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!(
                "yarvis-almacen-{}-{nombre}-{sello}-{n}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn ruta(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for DirTemporal {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn mapa(pares: &[(&str, &str)]) -> HashMap<String, String> {
        pares
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn archivo_inexistente_devuelve_mapa_vacio() {
        let dir = DirTemporal::nuevo("vacio");
        assert!(leer_mapa(dir.ruta()).unwrap().is_empty());
    }

    #[test]
    fn ida_y_vuelta_conserva_las_claves() {
        let dir = DirTemporal::nuevo("ida-vuelta");
        let original = mapa(&[("google", "CLAVE-GOOGLE"), ("nvidia", "CLAVE-NVIDIA")]);

        guardar_mapa(dir.ruta(), &original).unwrap();
        assert!(ruta_archivo(dir.ruta()).exists());

        let leidas = leer_mapa(dir.ruta()).unwrap();
        assert_eq!(leidas, original);
    }

    #[test]
    fn sobrescribir_reemplaza_el_contenido_anterior() {
        let dir = DirTemporal::nuevo("sobrescribir");
        guardar_mapa(dir.ruta(), &mapa(&[("google", "VIEJA")])).unwrap();
        guardar_mapa(dir.ruta(), &mapa(&[("google", "NUEVA")])).unwrap();

        let leidas = leer_mapa(dir.ruta()).unwrap();
        assert_eq!(leidas.get("google").map(String::as_str), Some("NUEVA"));
    }

    #[test]
    fn migra_json_legacy_y_borra_el_texto_plano() {
        let dir = DirTemporal::nuevo("legacy");
        let legacy = ruta_legacy(dir.ruta());
        std::fs::write(&legacy, r#"{"google":"SECRETO-LEGACY"}"#).unwrap();

        let leidas = leer_mapa(dir.ruta()).unwrap();
        assert_eq!(
            leidas.get("google").map(String::as_str),
            Some("SECRETO-LEGACY")
        );
        assert!(ruta_archivo(dir.ruta()).exists(), "debe crear api_keys.bin");
        assert!(!legacy.exists(), "debe borrar el api_keys.json en claro");
    }

    #[test]
    fn json_legacy_corrupto_no_se_borra() {
        let dir = DirTemporal::nuevo("legacy-corrupto");
        let legacy = ruta_legacy(dir.ruta());
        std::fs::write(&legacy, "{esto no es json").unwrap();

        assert!(leer_mapa(dir.ruta()).is_err());
        assert!(legacy.exists(), "no se debe borrar evidencia/archivo roto");
    }

    #[test]
    fn json_legacy_que_no_es_objeto_no_se_borra() {
        let dir = DirTemporal::nuevo("legacy-no-objeto");
        let legacy = ruta_legacy(dir.ruta());
        std::fs::write(&legacy, r#"["no", "es", "un", "mapa"]"#).unwrap();

        let err = leer_mapa(dir.ruta()).unwrap_err();
        assert!(err.contains("objeto JSON"), "error inesperado: {err}");
        assert!(
            legacy.exists(),
            "no se debe borrar un legacy con forma rara"
        );
    }

    #[test]
    fn archivo_bin_corrupto_da_error_claro() {
        let dir = DirTemporal::nuevo("corrupto");
        std::fs::write(ruta_archivo(dir.ruta()), b"no-es-un-archivo-valido").unwrap();

        let err = leer_mapa(dir.ruta()).unwrap_err();
        assert!(err.contains("formato"), "error inesperado: {err}");
    }

    #[test]
    fn no_deja_archivos_temporales() {
        let dir = DirTemporal::nuevo("temporales");
        guardar_mapa(dir.ruta(), &mapa(&[("google", "CLAVE")])).unwrap();

        let sobrantes: Vec<_> = std::fs::read_dir(dir.ruta())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(sobrantes.is_empty(), "quedaron temporales: {sobrantes:?}");
    }

    #[cfg(windows)]
    #[test]
    fn el_archivo_cifrado_no_contiene_la_clave_en_claro() {
        let dir = DirTemporal::nuevo("sin-plano");
        let secreto = "SECRETO-QUE-NO-DEBE-APARECER";
        guardar_mapa(dir.ruta(), &mapa(&[("google", secreto)])).unwrap();

        let bytes = std::fs::read(ruta_archivo(dir.ruta())).unwrap();
        assert!(
            !bytes
                .windows(secreto.len())
                .any(|v| v == secreto.as_bytes()),
            "la clave apareció en texto plano dentro de api_keys.bin"
        );
        assert_eq!(&bytes[..MAGIA.len()], MAGIA);
        assert_eq!(bytes[MAGIA.len()], ESQUEMA_DPAPI);
    }

    #[cfg(unix)]
    #[test]
    fn el_archivo_tiene_permisos_0600() {
        use std::os::unix::fs::PermissionsExt;

        let dir = DirTemporal::nuevo("permisos");
        guardar_mapa(dir.ruta(), &mapa(&[("google", "CLAVE")])).unwrap();

        let modo = std::fs::metadata(ruta_archivo(dir.ruta()))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(modo, 0o600);
    }
}
