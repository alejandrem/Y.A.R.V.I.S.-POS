// ============================================================
// impresora/commands.rs — Comandos Tauri del Camino A Fase 1.
//
//   listar_impresoras            -> impresoras instaladas (spooler)
//   imprimir_bytes_raw           -> manda bytes ya armados (Fase 2)
//   imprimir_lista_conciliacion  -> la que usa el boton "Imprimir Lista"
//
// Todo el I/O WinAPI es bloqueante: se corre en spawn_blocking para
// no atascar el runtime Tokio (mismo patron que predicciones).
// ============================================================

use chrono::Local;
use serde::{Deserialize, Serialize};

use super::builder::{self, FilaConciliacion};
use super::{enviar_bytes_raw, listar_impresoras_sistema};

#[derive(Debug, Clone, Serialize)]
pub struct ImpresoraInfo {
    pub nombre: String,
    pub predeterminada: bool,
}

/// Fila tal como la manda el frontend (pesos en f64, como el resto
/// del contrato IPC actual; el formateo final vive en builder.rs).
#[derive(Debug, Clone, Deserialize)]
pub struct FilaConciliacionPayload {
    pub nombre: String,
    pub fisico: i32,
    pub sistema: i32,
    #[serde(default)]
    pub precio_venta: f64,
}

#[tauri::command]
pub async fn listar_impresoras() -> Result<Vec<ImpresoraInfo>, String> {
    let lista = tokio::task::spawn_blocking(listar_impresoras_sistema)
        .await
        .map_err(|e| format!("Fallo interno listando impresoras: {e}"))??;
    tracing::info!(n = lista.len(), "impresoras listadas del spooler");
    Ok(lista
        .into_iter()
        .map(|i| ImpresoraInfo {
            nombre: i.nombre,
            predeterminada: i.predeterminada,
        })
        .collect())
}

#[tauri::command]
pub async fn imprimir_bytes_raw(
    nombre_impresora: String,
    bytes: Vec<u8>,
) -> Result<String, String> {
    let nombre = nombre_impresora.trim().to_string();
    if nombre.is_empty() {
        return Err("Elige una impresora instalada.".into());
    }
    let n = bytes.len();
    tokio::task::spawn_blocking(move || enviar_bytes_raw(&nombre, &bytes))
        .await
        .map_err(|e| format!("Fallo interno de impresion: {e}"))??;
    tracing::info!(impresora = %nombre_impresora, bytes = n, "RAW enviado al spooler");
    Ok(format!("Ticket enviado a '{nombre_impresora}' ({n} bytes)."))
}

#[tauri::command]
pub async fn imprimir_lista_conciliacion(
    nombre_impresora: String,
    tienda: Option<String>,
    filas: Vec<FilaConciliacionPayload>,
) -> Result<String, String> {
    let nombre = nombre_impresora.trim().to_string();
    if nombre.is_empty() {
        return Err("Elige una impresora instalada.".into());
    }
    if filas.is_empty() {
        return Err("No hay filas que imprimir.".into());
    }
    if filas.len() > builder::MAX_FILAS {
        return Err(format!(
            "Demasiadas filas ({}). Limite Fase 1: {}. Filtra la lista.",
            filas.len(),
            builder::MAX_FILAS
        ));
    }

    let tienda = tienda
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "MI TIENDA".to_string());
    let filas: Vec<FilaConciliacion> = filas
        .into_iter()
        .map(|f| FilaConciliacion {
            nombre: f.nombre,
            fisico: f.fisico.max(0),
            sistema: f.sistema.max(0),
            precio_venta: if f.precio_venta.is_finite() {
                f.precio_venta.max(0.0)
            } else {
                0.0
            },
        })
        .collect();

    let fecha = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let bytes = builder::construir_lista_conciliacion(&tienda, &fecha, &filas);
    let n = bytes.len();

    tokio::task::spawn_blocking(move || enviar_bytes_raw(&nombre, &bytes))
        .await
        .map_err(|e| format!("Fallo interno de impresion: {e}"))??;

    tracing::info!(impresora = %nombre_impresora, filas = filas.len(), "conciliacion impresa");
    Ok(format!(
        "Lista enviada a '{}' ({} productos, {} bytes).",
        nombre_impresora,
        filas.len(),
        n
    ))
}
