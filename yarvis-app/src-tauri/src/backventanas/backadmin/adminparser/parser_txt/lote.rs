// ============================================================
// lote — Parseo de tickets con mapeo: uno, carpeta o streaming.
//
// `parsear_carpeta_stream` es la vía del frontend: corre el lote en un
// hilo worker y emite `batch-progress` en TIEMPO REAL (el productor no
// bloquea al emisor, así el 0% no se congela con 12,000 tickets).
// ============================================================

use super::super::empleados_auto::{resolver_empleados_desde_ventas, ResultadoEmpleadosAuto};
use crate::backventanas::auth::AuthState;
use src_ia::cerebro::analizador_tickets::{parsear_linea, MapeoColumnas};
use src_ia::cerebro::parseador_masivo::{
    procesar_archivos, procesar_carpeta_impl, ArchivoResultado,
};
use tauri::Emitter;

#[tauri::command]
pub fn parsear_con_mapeo(
    auth: tauri::State<'_, AuthState>,
    texto: String,
    mapeo: serde_json::Value,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;
    let texto = texto.trim();
    if texto.is_empty() {
        return Ok(serde_json::json!({ "status": "error", "error": "El texto esta vacio" }));
    }

    let lineas: Vec<&str> = texto
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lineas.is_empty() {
        return Ok(serde_json::json!({ "status": "error", "error": "No hay lineas para parsear" }));
    }

    let mapeo: MapeoColumnas =
        serde_json::from_value(mapeo).map_err(|e| format!("Mapeo inválido: {}", e))?;
    let total_cols = lineas
        .iter()
        .map(|l| l.split_whitespace().count())
        .max()
        .unwrap_or(0);

    let mut items = Vec::new();
    let mut errores: Vec<String> = Vec::new();

    for (i, linea) in lineas.iter().enumerate() {
        match parsear_linea(linea, &mapeo, total_cols) {
            Some(item) => items.push(item),
            None => errores.push(format!("Linea {}: formato no reconocido", i + 1)),
        }
    }

    Ok(serde_json::json!({
        "status": "ok",
        "items": items,
        "total_lineas": lineas.len(),
        "lineas_parseadas": items.len(),
        "errores": errores.iter().take(20).collect::<Vec<_>>(),
    }))
}

#[tauri::command]
pub fn parsear_carpeta(
    auth: tauri::State<'_, AuthState>,
    carpeta: String,
    mapeo: serde_json::Value,
    db_path: String,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;
    let archivos = src_ia::cerebro::parseador_masivo::obtener_archivos_txt(&carpeta);
    if archivos.is_empty() {
        return Err("No se encontraron archivos .txt en la carpeta".to_string());
    }

    let mapeo: MapeoColumnas =
        serde_json::from_value(mapeo).map_err(|e| format!("Mapeo inválido: {}", e))?;

    let stats = procesar_carpeta_impl(archivos, mapeo, db_path.clone());
    let mut valor =
        serde_json::to_value(&stats).map_err(|e| format!("Error serializando resultado: {}", e))?;
    if let Some(obj) = valor.as_object_mut() {
        obj.insert("status".to_string(), serde_json::json!("ok"));
        // Alta automática de empleados (igual que en el stream); las
        // credenciales se consultan en el historial, aquí solo el conteo.
        let empleados = resolver_empleados_desde_ventas(&db_path).unwrap_or(ResultadoEmpleadosAuto {
            creados: Vec::new(),
            vinculados: 0,
        });
        obj.insert(
            "empleados_creados".to_string(),
            serde_json::json!(empleados.creados.len()),
        );
        obj.insert(
            "ventas_vinculadas_empleados".to_string(),
            serde_json::json!(empleados.vinculados),
        );
    }
    Ok(valor)
}

#[tauri::command]
pub async fn parsear_carpeta_stream(
    app_handle: tauri::AppHandle,
    auth: tauri::State<'_, AuthState>,
    carpeta: String,
    mapeo: serde_json::Value,
    db_path: String,
) -> Result<String, String> {
    auth.require_admin()?;
    let archivos = src_ia::cerebro::parseador_masivo::obtener_archivos_txt(&carpeta);
    if archivos.is_empty() {
        return Err("No se encontraron archivos .txt en la carpeta".to_string());
    }
    let total = archivos.len();

    let mapeo: MapeoColumnas =
        serde_json::from_value(mapeo).map_err(|e| format!("Mapeo inválido: {}", e))?;

    tauri::async_runtime::spawn_blocking(move || {
        emitir_stream_batch(&app_handle, &archivos, &mapeo, &db_path, total)
    })
    .await
    .map_err(|e| format!("Error en el worker de procesamiento: {}", e))?
}

/// Procesa los archivos con `src-ia::parseador_masivo::procesar_archivos` y
/// emite los mismos eventos (`progress` / `complete`) que emitía el motor
/// original, ahora en TIEMPO REAL: el productor corre en su propio hilo para
/// que el loop emisor no se quede bloqueado mientras procesa la carpeta.
fn emitir_stream_batch(
    app: &tauri::AppHandle,
    archivos: &[String],
    mapeo: &MapeoColumnas,
    db_path: &str,
    total: usize,
) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel::<ArchivoResultado>();

    // `procesar_archivos` es CPU-bound pesado (SQLite + parseo por archivo).
    // Si corriese aquí, ningún `progress` saldría hasta terminar TODA la
    // carpeta (12000 tickets = muchos minutos en 0%). En su propio hilo el
    // loop de abajo va emitiendo resultados conforme se procesan.
    let archivos_owned: Vec<String> = archivos.to_vec();
    let mapeo_owned = mapeo.clone();
    let db_owned = db_path.to_string();
    let _worker = std::thread::spawn(move || {
        procesar_archivos(&archivos_owned, &mapeo_owned, &db_owned, &tx);
        drop(tx);
    });

    let mut procesados = 0usize;
    let mut exitosos = 0usize;
    let mut errores = 0usize;
    let mut ventas_creadas = 0usize;
    let mut items_insertados = 0usize;
    let mut productos_nuevos = 0usize;
    let mut productos_existentes = 0usize;
    let mut duplicados_detectados = 0usize;
    let mut productos_nuevos_set: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let mut tickets_fallidos: Vec<serde_json::Value> = Vec::new();

    for res in rx {
        procesados += 1;
        if res.ok {
            exitosos += 1;
            // Un archivo puede traer N tickets → N ventas (regla B).
            ventas_creadas += res.ventas;
            items_insertados += res.items;
            duplicados_detectados += res.duplicados;
            productos_existentes += res.existentes;
            for nuevo in &res.nuevos {
                productos_nuevos_set.insert(nuevo.nombre.clone());
            }
            productos_nuevos += res.nuevos.len();
        } else {
            errores += 1;
            tickets_fallidos.push(serde_json::json!({
                "archivo": res.archivo,
                "motivo": res.motivo.unwrap_or_default(),
            }));
        }

        if procesados % 50 == 0 || procesados == total {
            let _ = app.emit(
                "batch-progress",
                serde_json::json!({
                    "type": "progress",
                    "procesados": procesados,
                    "total": total,
                    "exitosos": exitosos,
                    "errores": errores,
                    "ventas_creadas": ventas_creadas,
                    "items_insertados": items_insertados,
                    "productos_nuevos": productos_nuevos,
                    "productos_existentes": productos_existentes,
                    "duplicados_detectados": duplicados_detectados,
                }),
            );
        }
    }

    // Lote terminado: alta automática de empleados detectados en tickets
    // (sistema, no manual) + vinculación de sus ventas vía cajero_id.
    // Si falla no se aborta la importación: ya quedó guardada arriba.
    let empleados = resolver_empleados_desde_ventas(db_path).unwrap_or(ResultadoEmpleadosAuto {
        creados: Vec::new(),
        vinculados: 0,
    });

    let _ = app.emit("batch-progress", serde_json::json!({
        "type": "complete",
        "total_archivos": total,
        "procesados": procesados,
        "exitosos": exitosos,
        "errores": errores,
        "ventas_creadas": ventas_creadas,
        "items_insertados": items_insertados,
        "productos_nuevos": productos_nuevos,
        "productos_existentes": productos_existentes,
        "duplicados_detectados": duplicados_detectados,
        "productos_nuevos_lista": productos_nuevos_set.into_iter().take(100).collect::<Vec<_>>(),
        "tickets_fallidos": tickets_fallidos.iter().take(500).collect::<Vec<_>>(),
        "empleados_creados": empleados.creados,
        "ventas_vinculadas_empleados": empleados.vinculados,
    }));

    Ok("ok".to_string())
}
