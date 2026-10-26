// ============================================================
// api_config — Persistencia SEGURA de API keys del chat cloud.
//
// Por qué ya no localStorage: el webview guarda localStorage en
// texto plano en disco y es legible por cualquier XSS del frontend.
// Aquí viven en api_keys.bin dentro del app_data_dir:
//   - Windows: cifradas con DPAPI (CryptProtectData), atadas a la
//     cuenta de Windows actual. Copiar el archivo a otra máquina no
//     permite leerlas.
//   - Linux/macOS: JSON con permisos 0600 (respaldo; no hay DPAPI).
// El detalle de formato, migración desde api_keys.json y escritura
// atómica vive en el submódulo `almacen`.
//
// AUTORIZACIÓN (enforcement en backend, nunca en React):
// el ADMIN escribe tal cual (reemplazo total, puede borrar) y es el
// ÚNICO que puede LEER las claves (require_admin en leer_api_keys:
// si el empleado pudiera leerlas, robaría la cuota cloud del admin
// con un simple invoke desde devtools);
// el EMPLEADO solo puede AGREGAR claves de proveedores que aún
// no tienen una: cualquier intento de sobrescribir o eliminar
// una clave existente se rechaza con error. El candado del
// frontend (PanelYarvis) es solo UX; la regla vive aquí.
// Ver issue #4.
//
// Escalamiento futuro: si se necesita aislar también de procesos que
// corren con la misma cuenta de Windows, migrar a Credential Manager
// (crate `keyring`). El contrato de comandos no cambiaría.
// ============================================================

pub(crate) mod almacen;

use tauri::Manager;

use crate::backventanas::auth::Role;

/// Directorio de datos de la app donde vive api_keys.bin.
fn directorio_datos(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "No se pudo resolver el directorio de datos".to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
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

/// Migra el antiguo api_keys.json en claro al almacén cifrado al
/// arrancar, sin bloquear el splash ni tumbar la app si falla. Si la
/// migración no se puede hacer, el panel la reintenta al leer/guardar.
pub fn migrar_almacen(app: &tauri::AppHandle) {
    let dir = match directorio_datos(app) {
        Ok(dir) => dir,
        Err(e) => {
            tracing::warn!("[API KEYS] No se pudo resolver el directorio de datos: {e}");
            return;
        }
    };
    let habia_legacy = almacen::ruta_legacy(&dir).exists();
    match almacen::leer_mapa(&dir) {
        Ok(_) => {
            if habia_legacy && !almacen::ruta_legacy(&dir).exists() {
                tracing::info!("[API KEYS] api_keys.json migrado a api_keys.bin");
            }
        }
        Err(e) => {
            tracing::warn!("[API KEYS] No se pudo migrar/leer el almacén de claves: {e}");
        }
    }
}

#[tauri::command]
pub fn guardar_api_keys(
    app: tauri::AppHandle,
    auth: tauri::State<'_, crate::backventanas::auth::AuthState>,
    keys: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let sesion = auth.require_operator()?;
    let dir = directorio_datos(&app)?;
    let previas = almacen::leer_mapa(&dir)?;
    let finales = fusionar_claves_por_rol(&previas, &keys, sesion.role)?;
    almacen::guardar_mapa(&dir, &finales)
}

#[tauri::command]
pub fn leer_api_keys(
    app: tauri::AppHandle,
    auth: tauri::State<'_, crate::backventanas::auth::AuthState>,
) -> Result<std::collections::HashMap<String, String>, String> {
    // SOLO admin: las claves son secreto del dueño (cuota cloud). Un
    // empleado con lectura roba la cuota con un invoke desde devtools.
    // El empleado configura las SUYAS a ciegas: guardar_api_keys le
    // permite agregar proveedores nuevos sin ver los existentes.
    auth.require_admin()?;
    let dir = directorio_datos(&app)?;
    almacen::leer_mapa(&dir)
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
        let entrantes = mapa(&[("nube", "NUEVA")]);
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
        let entrantes = mapa(&[("google", "CLAVE-ADMIN"), ("nube", "MIA")]);
        let r = fusionar_claves_por_rol(&previas, &entrantes, Role::Employee).unwrap();
        assert_eq!(r.get("google").unwrap(), "CLAVE-ADMIN");
        assert_eq!(r.get("nube").unwrap(), "MIA");
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
        let previas = mapa(&[("google", "CLAVE-ADMIN"), ("nube", "OTRA")]);
        // Ataque del issue #4: mapa vacío para wipear todo.
        let r = fusionar_claves_por_rol(&previas, &HashMap::new(), Role::Employee);
        assert!(r.is_err(), "borrado total debe rechazarse");
        // Borrado parcial (omitir una) también.
        let r = fusionar_claves_por_rol(
            &previas,
            &mapa(&[("google", "CLAVE-ADMIN")]),
            Role::Employee,
        );
        assert!(r.is_err(), "borrado parcial debe rechazarse");
        // Vaciar el valor es borrar con otro nombre.
        let r = fusionar_claves_por_rol(
            &previas,
            &mapa(&[("google", ""), ("nube", "OTRA")]),
            Role::Employee,
        );
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
