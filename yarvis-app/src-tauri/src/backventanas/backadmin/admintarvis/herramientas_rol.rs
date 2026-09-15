// ============================================================
// admintarvis/herramientas_rol.rs — Ejecución de tools con control
// de acceso por rol (admin vs empleado) en el punto de ejecución,
// timeout anti-cuelgue y auditoría best-effort (issue #15).
// ============================================================

use std::time::{Duration, Instant};

use src_ia::motor_chat::llm::tools;

/// Tools que exponen métricas financieras GLOBALES (revenue total y
/// futuro, comparativas de periodo, análisis de recompra con costos)
/// o SQL libre. El prompt le pide al modelo no usarlas con empleados,
/// pero un prompt es sugerencia — ESTO es control de acceso real en el
/// punto de ejecución.
const TOOLS_SOLO_ADMIN: &[&str] = &[
    "query_sales",
    "compare_periods",
    "get_restock_analysis",
    "forecast_sales",
    "sql_readonly",
];

/// Tope de espera por tool: si el SQL se atora, se devuelve error al
/// modelo en vez de colgar el chat (el hilo bloqueante se abandona;
//  la conexión es de solo lectura, no deja escrituras a medias).
const TIMEOUT_TOOL: Duration = Duration::from_secs(20);

/// Ejecuta una tool respetando el rol de la sesión. Si está bloqueada, NO
/// se ejecuta: se le devuelve al modelo un error de permisos para que
/// responda con elegancia ("eso te lo puede decir el administrador").
/// Cada ejecución (ok o fallo) se registra en `tool_audit` best-effort.
pub(super) async fn ejecutar_tool_con_rol(
    nombre: &str,
    args: &str,
    db_path: &str,
    es_empleado: bool,
    usuario_id: i64,
) -> String {
    let inicio = Instant::now();
    let rol = if es_empleado { "empleado" } else { "admin" };

    if es_empleado && TOOLS_SOLO_ADMIN.contains(&nombre) {
        tracing::warn!("[YARVIS-TOOLS] BLOQUEADA por rol ({es_empleado}): {nombre}");
        let r = serde_json::json!({
            "error": "Permiso denegado: esta consulta financiera solo está disponible para el administrador."
        })
        .to_string();
        registrar_auditoria(db_path, nombre, args, usuario_id, rol, inicio, true, None);
        return r;
    }
    let n = nombre.to_string();
    let a = args.to_string();
    let db = db_path.to_string();
    let salida = tokio::time::timeout(TIMEOUT_TOOL, async move {
        tokio::task::spawn_blocking(move || tools::ejecutar_tool(&n, &a, &db)).await
    })
    .await;
    let (texto, ok, error) = match salida {
        Ok(Ok(Ok(r))) => (r, true, None),
        Ok(Ok(Err(e))) => (serde_json::json!({ "error": e }).to_string(), false, None),
        Ok(Err(e)) => (
            serde_json::json!({ "error": e.to_string() }).to_string(),
            false,
            None,
        ),
        Err(_) => (
            serde_json::json!({ "error": "La consulta tardó demasiado y se canceló." }).to_string(),
            false,
            Some("timeout 20s".to_string()),
        ),
    };
    // Si la tool devolvió {"error": ...} de negocio, también queda en la
    // bitácora como fallo para distinguirlo de un OK real.
    let fallo_negocio = serde_json::from_str::<serde_json::Value>(&texto)
        .ok()
        .and_then(|v| v.get("error").cloned())
        .is_some();
    registrar_auditoria(
        db_path,
        nombre,
        args,
        usuario_id,
        rol,
        inicio,
        ok && !fallo_negocio,
        error,
    );
    texto
}

/// Guarda una fila en `tool_audit`. Best-effort a propósito: si la DB
/// está ocupada o la tabla aún no existe (migración pendiente), solo se
/// loguea el warning y el chat sigue funcionando.
fn registrar_auditoria(
    db_path: &str,
    tool: &str,
    args: &str,
    usuario_id: i64,
    rol: &str,
    inicio: Instant,
    ok: bool,
    error: Option<String>,
) {
    let args_corto: String = args.chars().take(2000).collect();
    let ms = inicio.elapsed().as_millis().min(i64::MAX as u128) as i64;
    let Ok(conn) = src_ia::sqlite::abrir_db(db_path) else {
        tracing::warn!("[YARVIS-TOOLS] auditoría omitida: no se pudo abrir la DB");
        return;
    };
    if let Err(e) = conn.execute(
        "INSERT INTO tool_audit (tool, args, usuario_id, rol, ms, ok, error) VALUES (?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![tool, args_corto, usuario_id, rol, ms, ok as i32, error],
    ) {
        tracing::warn!("[YARVIS-TOOLS] auditoría omitida: {e}");
    }
}
