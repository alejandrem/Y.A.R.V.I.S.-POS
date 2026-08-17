use crate::backventanas::db::db::DbPath;

/// Retorna la ruta absoluta de la base de datos SQLite.
#[tauri::command]
pub fn get_db_path(db_path: tauri::State<'_, DbPath>) -> Result<String, String> {
    Ok(db_path.0.clone())
}

// ============================================================
// Comandos que siguen en commands/parser (no movidos a parser_rs)
// ============================================================

#[tauri::command]
pub fn vincular_inventario(
    productos: serde_json::Value,
    db_path: String,
    umbral: f64,
) -> Result<serde_json::Value, String> {
    let parseados = productos.as_array().cloned().unwrap_or_default();
    if parseados.is_empty() {
        return Err("No hay productos para vincular".to_string());
    }

    // Sin embedder (FASE 4 aún en Python) → solo match exacto, por_embedding = 0.
    let resultado =
        src_ia::cerebro::vinculador::vincular_con_inventario(&parseados, &db_path, umbral, None);

    serde_json::to_value(resultado).map_err(|e| format!("Error serializando resultado: {}", e))
}

#[tauri::command]
pub fn guardar_vinculacion(
    vinculaciones: serde_json::Value,
    db_path: String,
) -> Result<serde_json::Value, String> {
    let vinculaciones = vinculaciones.as_array().cloned().unwrap_or_default();

    let actualizados =
        src_ia::cerebro::vinculador::guardar_vinculacion(&vinculaciones, &db_path)?;

    Ok(serde_json::json!({ "status": "ok", "actualizados": actualizados }))
}

#[tauri::command]
pub async fn descargar_modelos() -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let descargados = src_ia::rutas::analizador_llm::descargar_modelos();
        let msg = format!("{descargados} modelo(s) descargado(s) de VRAM");
        println!("[YARVIS-IA] {msg}");
        serde_json::json!({ "status": "ok", "descargados": descargados, "mensaje": msg })
    })
    .await
    .map_err(|e| format!("Tarea de liberación abortada: {}", e))
}
