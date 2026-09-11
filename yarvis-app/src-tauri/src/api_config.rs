// ============================================================
// api_config — Persistencia SEGURA de API keys del chat cloud.
//
// Por qué ya no localStorage: el webview guarda localStorage en
// texto plano en disco y es legible por cualquier XSS del frontend.
// Aquí viven en un JSON dentro del app_data_dir con permisos 0600
// (solo el usuario dueño puede leerlo).
//
// AUTORIZACIÓN (enforcement en backend, nunca en React):
// el ADMIN escribe tal cual (reemplazo total, puede borrar);
// el EMPLEADO solo puede AGREGAR claves de proveedores que aún
// no tienen una: cualquier intento de sobrescribir o eliminar
// una clave existente se rechaza con error. El candado del
// frontend (PanelYarvis) es solo UX; la regla vive aquí.
// Ver issue #4.
//
// Escalamiento futuro: OS keychain (libsecret/Windows Credential
// Manager) vía crate `keyring`. El contrato de comandos no cambiaría.
// ============================================================

use serde_json::Value;
use tauri::Manager;

use crate::backventanas::auth::Role;

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

/// Fusiona las claves entrantes con las guardadas según el rol.
///
/// - Admin: reemplazo total (puede agregar, cambiar y borrar).
/// - Empleado: solo puede AGREGAR claves de proveedores que aún no
///   tienen una (vacía o ausente). Sobrescribir una clave existente
///   con otro valor, o eliminarla (omitirla / vaciarla), es un error.
///
/// Función pura para poder probarla sin contexto Tauri (los comandos
/// necesitan `AppHandle` + `State`, que no se construyen en tests).
/// Los valores se comparan recortados; los vacíos entrantes se ignoran.
pub fn fusionar_claves_por_rol(
    previas: &std::collections::HashMap<String, String>,
    entrantes: &std::collections::HashMap<String, String>,
    rol: Role,
) -> Result<std::collections::HashMap<String, String>, String> {
    if rol == Role::Admin {
        return Ok(entrantes.clone());
    }
    let mut fusionadas = previas.clone();
    for (proveedor, valor) in entrantes {
        let valor = valor.trim();
        if valor.is_empty() {
            continue;
        }
        let existe = fusionadas
            .get(proveedor)
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);
        if existe && fusionadas.get(proveedor).map(|v| v.trim()) != Some(valor) {
            return Err(format!(
                "Sin permiso: la clave de '{proveedor}' la configuró el administrador y no se puede modificar"
            ));
        }
        fusionadas.insert(proveedor.clone(), valor.to_string());
    }
    for (proveedor, valor) in previas {
        if valor.trim().is_empty() {
            continue;
        }
        let conserva = entrantes
            .get(proveedor)
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);
        if !conserva {
            return Err(format!(
                "Sin permiso: la clave de '{proveedor}' la configuró el administrador y no se puede eliminar"
            ));
        }
    }
    Ok(fusionadas)
}

/// Lee el mapa de claves desde disco. Si el archivo no existe,
/// devuelve mapa vacío (primera configuración).
fn leer_mapa(ruta: &std::path::Path) -> Result<std::collections::HashMap<String, String>, String> {
    let mut out = std::collections::HashMap::new();
    if !ruta.exists() {
        return Ok(out);
    }
    let contenido = std::fs::read_to_string(ruta).map_err(|e| e.to_string())?;
    let parsed: Value = serde_json::from_str(&contenido).map_err(|e| e.to_string())?;
    if let Value::Object(mapa) = parsed {
        for (k, v) in mapa {
            if let Some(s) = v.as_str() {
                out.insert(k, s.to_string());
            }
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn guardar_api_keys(
    app: tauri::AppHandle,
    auth: tauri::State<'_, crate::backventanas::auth::AuthState>,
    keys: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let sesion = auth.require_operator()?;
    let ruta = ruta_config(&app)?;
    let previas = leer_mapa(&ruta)?;
    let finales = fusionar_claves_por_rol(&previas, &keys, sesion.role)?;
    let json = serde_json::to_string_pretty(&finales).map_err(|e| e.to_string())?;
    escribir_con_permisos(&ruta, &json)
}

#[tauri::command]
pub fn leer_api_keys(
    app: tauri::AppHandle,
    auth: tauri::State<'_, crate::backventanas::auth::AuthState>,
) -> Result<std::collections::HashMap<String, String>, String> {
    auth.require_operator()?;
    let ruta = ruta_config(&app)?;
    leer_mapa(&ruta)
}

#[cfg(test)]
mod tests {
    use super::{fusionar_claves_por_rol, Role};
    use std::collections::HashMap;

    fn mapa(pares: &[(&str, &str)]) -> HashMap<String, String> {
        pares
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn admin_reemplaza_total_incluso_borrando() {
        let previas = mapa(&[("google", "CLAVE-VIEJA")]);
        let entrantes = mapa(&[("opencode", "NUEVA")]);
        let r = fusionar_claves_por_rol(&previas, &entrantes, Role::Admin).unwrap();
        assert_eq!(r, entrantes, "el admin escribe tal cual");
        // Borrado total también es válido para el admin.
        let r = fusionar_claves_por_rol(&previas, &HashMap::new(), Role::Admin).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn empleado_agrega_proveedor_nuevo() {
        let previas = mapa(&[("google", "CLAVE-ADMIN")]);
        // Flujo normal del frontend: reenvía previas + la nueva.
        let entrantes = mapa(&[("google", "CLAVE-ADMIN"), ("opencode", "MIA")]);
        let r = fusionar_claves_por_rol(&previas, &entrantes, Role::Employee).unwrap();
        assert_eq!(r.get("google").unwrap(), "CLAVE-ADMIN");
        assert_eq!(r.get("opencode").unwrap(), "MIA");
    }

    #[test]
    fn empleado_reenvia_iguales_sin_cambios_ok() {
        let previas = mapa(&[("google", "CLAVE-ADMIN")]);
        let r = fusionar_claves_por_rol(&previas, &previas, Role::Employee).unwrap();
        assert_eq!(r, previas);
    }

    #[test]
    fn empleado_no_puede_sobrescribir_clave_del_admin() {
        let previas = mapa(&[("google", "CLAVE-ADMIN")]);
        let entrantes = mapa(&[("google", "HACKEADA")]);
        let r = fusionar_claves_por_rol(&previas, &entrantes, Role::Employee);
        assert!(r.is_err(), "sobrescribir debe rechazarse");
    }

    #[test]
    fn empleado_no_puede_borrar_claves_del_admin() {
        let previas = mapa(&[("google", "CLAVE-ADMIN"), ("opencode", "OTRA")]);
        // Ataque del issue #4: mapa vacío para wipear todo.
        let r = fusionar_claves_por_rol(&previas, &HashMap::new(), Role::Employee);
        assert!(r.is_err(), "borrado total debe rechazarse");
        // Borrado parcial (omitir una) también.
        let r = fusionar_claves_por_rol(&previas, &mapa(&[("google", "CLAVE-ADMIN")]), Role::Employee);
        assert!(r.is_err(), "borrado parcial debe rechazarse");
        // Vaciar el valor es borrar con otro nombre.
        let r = fusionar_claves_por_rol(&previas, &mapa(&[("google", ""), ("opencode", "OTRA")]), Role::Employee);
        assert!(r.is_err(), "vaciar debe rechazarse");
    }

    #[test]
    fn empleado_con_tienda_sin_claves_puede_configurar() {
        let previas = HashMap::new();
        let entrantes = mapa(&[("google", "PRIMERA")]);
        let r = fusionar_claves_por_rol(&previas, &entrantes, Role::Employee).unwrap();
        assert_eq!(r.get("google").unwrap(), "PRIMERA");
    }
}
