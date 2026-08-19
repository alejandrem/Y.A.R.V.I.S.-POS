// Parser TXT/Visual en Rust.
// Los comandos de catálogo, mapeo, carpetas y el análisis con LLM usan el
// crate `src-ia`. El LLM local corre
// vía llama.cpp dentro del feature `llm-local` de `src-ia`.
use std::fs;
use std::path;
use super::utils::sanitize_path;
use tauri::Emitter;
use src_ia::cerebro::analizador_tickets::{parsear_linea, MapeoColumnas};
use src_ia::cerebro::parseador_masivo::{procesar_archivos, procesar_carpeta_impl, ArchivoResultado};

#[derive(serde::Serialize)]
pub struct ArchivoCarpeta {
    pub nombre: String,
    pub ruta: String,
    pub tamano: u64,
    pub preview: String,
}

#[tauri::command]
pub fn listar_archivos_carpeta(carpeta: String) -> Result<Vec<ArchivoCarpeta>, String> {
    let dir = path::Path::new(&carpeta);
    if !dir.is_dir() {
        return Err(format!("La ruta no es una carpeta: {}", carpeta));
    }

    let mut archivos = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| format!("Error leyendo carpeta: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Error leyendo entrada: {}", e))?;
        let file_path = entry.path();

        if !file_path.is_file() {
            continue;
        }

        let ext = file_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext != "txt" {
            continue;
        }

        let nombre = file_path.file_name()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        let ruta = file_path.to_string_lossy().to_string();

        let tamano = fs::metadata(&file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Leer primeras 5 lineas para preview
        let preview = fs::read_to_string(&file_path)
            .map(|content| {
                content.lines()
                    .take(5)
                    .collect::<Vec<&str>>()
                    .join("\n")
            })
            .unwrap_or_else(|_| "Error al leer archivo".to_string());

        archivos.push(ArchivoCarpeta {
            nombre,
            ruta,
            tamano,
            preview,
        });
    }

    archivos.sort_by(|a, b| a.nombre.cmp(&b.nombre));
    Ok(archivos)
}

#[tauri::command]
pub fn leer_archivo_raw(path: String) -> Result<String, String> {
    let safe_path = sanitize_path(&path)?;
    fs::read_to_string(safe_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn leer_archivo_bytes(path: String) -> Result<Vec<u8>, String> {
    let safe_path = sanitize_path(&path)?;
    fs::read(safe_path).map_err(|e| e.to_string())
}

// ============================================================
// Parser de catálogo visual (nativo)
// ============================================================

#[tauri::command]
pub fn parsear_catalogo_visual(path: String) -> Result<serde_json::Value, String> {
    let safe_path = sanitize_path(&path)?;
    let content = fs::read_to_string(safe_path).map_err(|e| e.to_string())?;

    let productos = src_ia::formatos::lector_txt::parsear_catalogo_visual(&content);
    if productos.is_empty() {
        return Err("No se encontraron productos en el catálogo".to_string());
    }

    let mut categorias: Vec<String> = productos
        .iter()
        .map(|p| p.categoria.clone())
        .filter(|c| !c.is_empty())
        .collect();
    categorias.sort();
    categorias.dedup();

    Ok(serde_json::json!({
        "status": "ok",
        "productos": productos,
        "total": productos.len(),
        "categorias": categorias,
    }))
}

// ============================================================
// Análisis de tickets con LLM
// ============================================================

/// Convierte el JSON de `analizar_ticket` en `Result` (ok → Ok, error → Err).
fn resultado_a_result(resultado: serde_json::Value) -> Result<serde_json::Value, String> {
    if resultado.get("status").and_then(|s| s.as_str()) == Some("ok") {
        Ok(resultado)
    } else {
        Err(resultado
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("Ningún modelo pudo analizar el ticket")
            .to_string())
    }
}

/// Ejecuta la inferencia fuera del hilo principal. Los comandos síncronos de
/// Tauri corren en el main thread: cargar el GGUF (mmap ~463 MB) y generar en
/// CPU congela la ventana (WebKit queda "No responde" y pide forzar cierre).
/// `spawn_blocking` la corre en otro hilo y la UI sigue viva durante el análisis.
#[tauri::command]
pub async fn analizar_ticket_llm(path: String) -> Result<serde_json::Value, String> {
    let safe_path = sanitize_path(&path)?;
    let contenido = fs::read_to_string(&safe_path)
        .map_err(|e| format!("No se pudo leer el archivo: {}", e))?;

    tauri::async_runtime::spawn_blocking(move || {
        resultado_a_result(src_ia::rutas::analizar_ticket(&contenido))
    })
    .await
    .map_err(|e| format!("Tarea de análisis abortada: {}", e))?
}

#[tauri::command]
pub async fn analizar_ticket_con_ia(texto: String) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        resultado_a_result(src_ia::rutas::analizar_ticket(&texto))
    })
    .await
    .map_err(|e| format!("Tarea de análisis abortada: {}", e))?
}

// ============================================================
// Parseo con mapeo de columnas (nativo)
// ============================================================

#[tauri::command]
pub fn parsear_con_mapeo(texto: String, mapeo: serde_json::Value) -> Result<serde_json::Value, String> {
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

// ============================================================
// Parseo de carpetas (nativo)
// ============================================================

#[tauri::command]
pub fn parsear_carpeta(
    carpeta: String,
    mapeo: serde_json::Value,
    db_path: String,
) -> Result<serde_json::Value, String> {
    let archivos = src_ia::cerebro::parseador_masivo::obtener_archivos_txt(&carpeta);
    if archivos.is_empty() {
        return Err("No se encontraron archivos .txt en la carpeta".to_string());
    }

    let mapeo: MapeoColumnas =
        serde_json::from_value(mapeo).map_err(|e| format!("Mapeo inválido: {}", e))?;

    let stats = procesar_carpeta_impl(archivos, mapeo, db_path);
    let mut valor = serde_json::to_value(&stats)
        .map_err(|e| format!("Error serializando resultado: {}", e))?;
    if let Some(obj) = valor.as_object_mut() {
        obj.insert("status".to_string(), serde_json::json!("ok"));
    }
    Ok(valor)
}

// ============================================================
// Parseo de carpetas con streaming (emite eventos batch-progress)
// ============================================================

#[tauri::command]
pub async fn parsear_carpeta_stream(
    app_handle: tauri::AppHandle,
    carpeta: String,
    mapeo: serde_json::Value,
    db_path: String,
) -> Result<String, String> {
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

/// Procesa los archivos con `src-ia::lote::procesar_archivos` y emite los
/// mismos eventos SSE (`progress` / `complete`) que emitía el motor original.
fn emitir_stream_batch(
    app: &tauri::AppHandle,
    archivos: &[String],
    mapeo: &MapeoColumnas,
    db_path: &str,
    total: usize,
) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel::<ArchivoResultado>();
    procesar_archivos(archivos, mapeo, db_path, &tx);
    drop(tx);

    let mut procesados = 0usize;
    let mut exitosos = 0usize;
    let mut errores = 0usize;
    let mut ventas_creadas = 0usize;
    let mut items_insertados = 0usize;
    let mut productos_nuevos = 0usize;
    let mut productos_existentes = 0usize;
    let mut duplicados_detectados = 0usize;
    let mut productos_nuevos_set: std::collections::HashSet<String> = std::collections::HashSet::new();
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
            let _ = app.emit("batch-progress", serde_json::json!({
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
            }));
        }
    }

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
    }));

    Ok("ok".to_string())
}

