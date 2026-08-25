// ============================================================
// admintarvis/herramientas_rol.rs — Ejecución de tools con control
// de acceso por rol (admin vs empleado) en el punto de ejecución.
// ============================================================

use src_ia::motor_chat::llm::tools;

/// Tools que exponen métricas financieras GLOBALES (revenue total,
/// comparativas de periodo, análisis de recompra con costos). El prompt
/// le pide al modelo no usarlas con empleados, pero un prompt es
/// sugerencia — ESTO es control de acceso real en el punto de ejecución.
const TOOLS_SOLO_ADMIN: &[&str] = &["query_sales", "compare_periods", "get_restock_analysis"];

/// Ejecuta una tool respetando el rol de la sesión. Si está bloqueada, NO
/// se ejecuta: se le devuelve al modelo un error de permisos para que
/// responda con elegancia ("eso te lo puede decir el administrador").
pub(super) async fn ejecutar_tool_con_rol(
    nombre: &str,
    args: &str,
    db_path: &str,
    es_empleado: bool,
) -> String {
    if es_empleado && TOOLS_SOLO_ADMIN.contains(&nombre) {
        tracing::warn!("[YARVIS-TOOLS] BLOQUEADA por rol ({es_empleado}): {nombre}");
        return serde_json::json!({
            "error": "Permiso denegado: esta consulta financiera solo está disponible para el administrador."
        })
        .to_string();
    }
    let n = nombre.to_string();
    let a = args.to_string();
    let db = db_path.to_string();
    match tokio::task::spawn_blocking(move || tools::ejecutar_tool(&n, &a, &db)).await {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => serde_json::json!({ "error": e }).to_string(),
        Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
    }
}
