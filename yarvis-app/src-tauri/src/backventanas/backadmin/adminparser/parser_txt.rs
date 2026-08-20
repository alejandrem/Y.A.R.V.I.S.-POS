// Parser TXT/Visual en Rust.
// Los comandos de catálogo, mapeo, carpetas y el análisis con LLM usan el
// crate `src-ia`. El LLM local corre
// vía llama.cpp dentro del feature `llm-local` de `src-ia`.
use super::utils::sanitize_path;
use crate::backventanas::auth::AuthState;
use rand::seq::SliceRandom;
use src_ia::cerebro::analizador_tickets::{parsear_linea, MapeoColumnas};
use src_ia::cerebro::parseador_masivo::{
    procesar_archivos, procesar_carpeta_impl, ArchivoResultado,
};
use std::collections::HashMap;
use std::fs;
use std::path;
use tauri::Emitter;

#[derive(serde::Serialize)]
pub struct ArchivoCarpeta {
    pub nombre: String,
    pub ruta: String,
    pub tamano: u64,
    pub preview: String,
}

#[tauri::command]
pub fn listar_archivos_carpeta(
    auth: tauri::State<'_, AuthState>,
    carpeta: String,
) -> Result<Vec<ArchivoCarpeta>, String> {
    auth.require_admin()?;
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

        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext != "txt" {
            continue;
        }

        let nombre = file_path
            .file_name()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        let ruta = file_path.to_string_lossy().to_string();

        let tamano = fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);

        // Leer primeras 5 lineas para preview
        let preview = fs::read_to_string(&file_path)
            .map(|content| content.lines().take(5).collect::<Vec<&str>>().join("\n"))
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
pub fn leer_archivo_raw(auth: tauri::State<'_, AuthState>, path: String) -> Result<String, String> {
    auth.require_admin()?;
    let safe_path = sanitize_path(&path)?;
    fs::read_to_string(safe_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn leer_archivo_bytes(
    auth: tauri::State<'_, AuthState>,
    path: String,
) -> Result<Vec<u8>, String> {
    auth.require_admin()?;
    let safe_path = sanitize_path(&path)?;
    fs::read(safe_path).map_err(|e| e.to_string())
}

// ============================================================
// Parser de catálogo visual (nativo)
// ============================================================

#[tauri::command]
pub fn parsear_catalogo_visual(
    auth: tauri::State<'_, AuthState>,
    path: String,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;
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
pub async fn analizar_ticket_llm(
    auth: tauri::State<'_, AuthState>,
    path: String,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;
    let safe_path = sanitize_path(&path)?;
    let contenido =
        fs::read_to_string(&safe_path).map_err(|e| format!("No se pudo leer el archivo: {}", e))?;

    tauri::async_runtime::spawn_blocking(move || {
        resultado_a_result(src_ia::rutas::analizar_ticket(&contenido))
    })
    .await
    .map_err(|e| format!("Tarea de análisis abortada: {}", e))?
}

#[tauri::command]
pub async fn analizar_ticket_con_ia(
    auth: tauri::State<'_, AuthState>,
    texto: String,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;
    tauri::async_runtime::spawn_blocking(move || {
        resultado_a_result(src_ia::rutas::analizar_ticket(&texto))
    })
    .await
    .map_err(|e| format!("Tarea de análisis abortada: {}", e))?
}

/// Analiza hasta cinco tickets elegidos al azar y obtiene un mapeo estable
/// para procesar automáticamente el resto de la carpeta.
///
/// Esto es calibración de estructura, no fine-tuning del modelo GGUF: el
/// modelo actual es de inferencia y no modifica sus pesos. Se vota entre los
/// mapeos válidos para evitar que un ticket anómalo defina toda la importación.
#[tauri::command]
pub async fn analizar_muestras_carpeta(
    app_handle: tauri::AppHandle,
    auth: tauri::State<'_, AuthState>,
    carpeta: String,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;

    let mut archivos = src_ia::cerebro::parseador_masivo::obtener_archivos_txt(&carpeta);
    if archivos.is_empty() {
        return Err("No se encontraron archivos .txt en la carpeta".to_string());
    }

    archivos.shuffle(&mut rand::thread_rng());
    archivos.truncate(5);

    tauri::async_runtime::spawn_blocking(move || {
        let total = archivos.len();
        let mut votos: HashMap<String, (serde_json::Value, usize)> = HashMap::new();
        let mut exitosos = 0usize;
        let mut muestras = Vec::with_capacity(total);

        for (indice, archivo) in archivos.iter().enumerate() {
            let nombre = path::Path::new(archivo)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(archivo)
                .to_string();

            let resultado = match fs::read(archivo) {
                Ok(bytes) => {
                    let texto = String::from_utf8_lossy(&bytes);
                    src_ia::rutas::analizar_ticket(&texto)
                }
                Err(error) => serde_json::json!({
                    "status": "error",
                    "error": format!("No se pudo leer el archivo: {error}")
                }),
            };

            if let Some(mapeo) = normalizar_mapeo_analisis(&resultado) {
                let clave = serde_json::to_string(&mapeo)
                    .map_err(|e| format!("No se pudo serializar el mapeo: {e}"))?;
                let entrada = votos.entry(clave).or_insert_with(|| (mapeo.clone(), 0));
                entrada.1 += 1;
                exitosos += 1;
                muestras.push(serde_json::json!({
                    "archivo": nombre,
                    "estado": "ok"
                }));
                let _ = app_handle.emit("parser-training-progress", serde_json::json!({
                    "indice": indice + 1,
                    "total": total,
                    "archivo": muestras.last().and_then(|m| m.get("archivo")).and_then(|v| v.as_str()).unwrap_or("ticket"),
                    "estado": "ok",
                    "mensaje": format!("Ticket {} de {} analizado", indice + 1, total)
                }));
            } else {
                let error = resultado
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("El modelo no devolvió un mapeo válido");
                muestras.push(serde_json::json!({
                    "archivo": nombre,
                    "estado": "error",
                    "error": error
                }));
                let _ = app_handle.emit("parser-training-progress", serde_json::json!({
                    "indice": indice + 1,
                    "total": total,
                    "archivo": muestras.last().and_then(|m| m.get("archivo")).and_then(|v| v.as_str()).unwrap_or("ticket"),
                    "estado": "error",
                    "mensaje": format!("Ticket {} de {} necesita revisión", indice + 1, total)
                }));
            }
        }

        let (_, (mapeo, votos_ganadores)) = votos
            .into_iter()
            .max_by_key(|(_, (_, cantidad))| *cantidad)
            .ok_or_else(|| "Ninguno de los tickets de muestra produjo un mapeo válido".to_string())?;

        Ok(serde_json::json!({
            "status": "ok",
            "mapeo": mapeo,
            "muestras": muestras,
            "analizados": exitosos,
            "total_muestras": total,
            "votos_ganadores": votos_ganadores
        }))
    })
    .await
    .map_err(|e| format!("Error en la calibración de tickets: {e}"))?
}

fn normalizar_mapeo_analisis(resultado: &serde_json::Value) -> Option<serde_json::Value> {
    let columnas = resultado.get("mapeo")?.get("columnas")?;
    let cantidad = columnas.get("cantidad")?.as_i64()? as i32;
    let precio_unitario = columnas.get("precio_unitario")?.as_i64()? as i32;
    let total = columnas.get("total")?.as_i64()? as i32;
    let producto = match columnas.get("producto")? {
        serde_json::Value::Array(valores) => valores
            .iter()
            .filter_map(|valor| valor.as_i64().map(|v| v as i32))
            .collect::<Vec<_>>(),
        valor => vec![valor.as_i64()? as i32],
    };

    if producto.is_empty() {
        return None;
    }

    Some(serde_json::json!({
        "cantidad": cantidad,
        "producto": producto,
        "precio_unitario": precio_unitario,
        "total": total,
        "descuento": columnas.get("descuento").and_then(|v| v.as_i64()).map(|v| v as i32)
    }))
}

// ============================================================
// Parseo con mapeo de columnas (nativo)
// ============================================================

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

// ============================================================
// Parseo de carpetas (nativo)
// ============================================================

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

    let stats = procesar_carpeta_impl(archivos, mapeo, db_path);
    let mut valor =
        serde_json::to_value(&stats).map_err(|e| format!("Error serializando resultado: {}", e))?;
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
