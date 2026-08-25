// ============================================================
// api_config — Persistencia SEGURA de API keys del chat cloud.
//
// Por qué ya no localStorage: el webview guarda localStorage en
// texto plano en disco y es legible por cualquier XSS del frontend.
// Aquí viven en un JSON dentro del app_data_dir con permisos 0600
// (solo el usuario dueño puede leerlo).
//
// Escalamiento futuro: OS keychain (libsecret/Windows Credential
// Manager) vía crate `keyring`. El contrato de comandos no cambiaría.
// ============================================================

use serde_json::Value;
use tauri::Manager;

/// Ruta del archivo de configuración de APIs.
fn ruta_config(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "No se pudo resolver el directorio de datos".to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("api_keys.json"))
}

fn escribir_con_permisos(ruta: &std::path::Path, contenido: &str) -> Result<(), String> {
    std::fs::write(ruta, contenido).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(ruta, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn guardar_api_keys(
    app: tauri::AppHandle,
    auth: tauri::State<'_, crate::backventanas::auth::AuthState>,
    keys: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    auth.require_operator()?;
    let ruta = ruta_config(&app)?;
    let json = serde_json::to_string_pretty(&keys).map_err(|e| e.to_string())?;
    escribir_con_permisos(&ruta, &json)
}

#[tauri::command]
pub fn leer_api_keys(
    app: tauri::AppHandle,
    auth: tauri::State<'_, crate::backventanas::auth::AuthState>,
) -> Result<std::collections::HashMap<String, String>, String> {
    auth.require_operator()?;
    let ruta = ruta_config(&app)?;
    if !ruta.exists() {
        return Ok(Default::default());
    }
    let contenido = std::fs::read_to_string(&ruta).map_err(|e| e.to_string())?;
    let parsed: Value = serde_json::from_str(&contenido).map_err(|e| e.to_string())?;
    let mut out = std::collections::HashMap::new();
    if let Value::Object(mapa) = parsed {
        for (k, v) in mapa {
            if let Some(s) = v.as_str() {
                out.insert(k, s.to_string());
            }
        }
    }
    Ok(out)
}
