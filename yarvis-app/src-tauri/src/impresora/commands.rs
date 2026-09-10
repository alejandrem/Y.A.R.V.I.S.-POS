// ============================================================
// impresora/commands.rs — Comandos Tauri de impresion termica.
//
// Fase 1 (spooler Windows RAW):
//   listar_impresoras            -> impresoras instaladas (spooler)
//   imprimir_bytes_raw           -> manda bytes ya armados (Fase 2)
//   imprimir_lista_conciliacion  -> la que usa el boton "Imprimir Lista"
//
// Fase 2 (crate `escpos`, ticket de venta enriquecido):
//   imprimir_ticket_venta        -> destino Spooler o Red (TCP 9100)
//   probar_red                   -> chequeo TCP sin gastar papel
//
// Todo el I/O es bloqueante: corre en spawn_blocking para no
// atascar el runtime Tokio (mismo patron que predicciones).
// ============================================================

use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use chrono::Local;
use serde::{Deserialize, Serialize};

use super::builder::{self, FilaConciliacion};
use super::ticket::{LineaVenta, TicketVenta};
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

// ── Fase 2: ticket de venta enriquecido (crate `escpos`) ──

/// Destino del ticket. Spooler usa la Fase 1 (Windows); Red abre
/// TCP directo contra la termica (puerto 9100 tipico).
/// El frontend manda `{ "Spooler": { "nombre": "..." } }` o
/// `{ "Red": { "ip": "...", "puerto": 9100 } }`.
#[derive(Debug, Clone, Deserialize)]
pub enum Destino {
    Spooler { nombre: String },
    Red { ip: String, puerto: u16 },
}

#[derive(Debug, Clone, Deserialize)]
pub struct LineaVentaPayload {
    pub nombre: String,
    pub cantidad: f64,
    pub precio_unitario: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PagoPayload {
    pub metodo: String,
    pub monto: f64,
}

/// Ticket tal como lo manda el modal de venta (pesos en f64, como
/// el resto del contrato IPC; el formateo final vive en ticket.rs).
#[derive(Debug, Clone, Deserialize)]
pub struct TicketVentaPayload {
    pub tienda: String,
    #[serde(default)]
    pub ubicacion: Option<String>,
    pub folio: String,
    #[serde(default)]
    pub fecha: Option<String>,
    pub lineas: Vec<LineaVentaPayload>,
    pub total: f64,
    #[serde(default)]
    pub pagos: Vec<PagoPayload>,
    #[serde(default)]
    pub qr: Option<String>,
}

fn limpiar(s: &str) -> String {
    s.trim().to_string()
}

/// TCP directo sin gastar papel: solo conecta y cierra.
fn checar_red(ip: &str, puerto: u16) -> Result<(), String> {
    let ip = ip.trim();
    if ip.is_empty() {
        return Err("Escribe la IP de la impresora de red.".into());
    }
    if puerto == 0 {
        return Err("Puerto invalido (tipico: 9100).".into());
    }
    let dir = format!("{ip}:{puerto}")
        .to_socket_addrs()
        .map_err(|e| format!("IP/host invalido '{ip}': {e}"))?
        .next()
        .ok_or_else(|| format!("No se resolvio '{ip}'."))?;
    TcpStream::connect_timeout(&dir, Duration::from_secs(5))
        .map_err(|e| format!("Sin conexion a {ip}:{puerto} ({e}). Revisa red y puerto."))?;
    Ok(())
}

fn enviar_red(ip: &str, puerto: u16, bytes: &[u8]) -> Result<(), String> {
    let ip = ip.trim().to_string();
    let dir = format!("{ip}:{puerto}")
        .to_socket_addrs()
        .map_err(|e| format!("IP/host invalido '{ip}': {e}"))?
        .next()
        .ok_or_else(|| format!("No se resolvio '{ip}'."))?;
    let mut stream = TcpStream::connect_timeout(&dir, Duration::from_secs(5))
        .map_err(|e| format!("Sin conexion a {ip}:{puerto} ({e})."))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("No se pudo configurar timeout: {e}"))?;
    stream
        .write_all(bytes)
        .map_err(|e| format!("Fallo enviando a {ip}:{puerto}: {e}"))?;
    stream.flush().map_err(|e| format!("Fallo flush a {ip}:{puerto}: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn probar_red(ip: String, puerto: u16) -> Result<String, String> {
    let ip_c = limpiar(&ip);
    tokio::task::spawn_blocking(move || checar_red(&ip_c, puerto))
        .await
        .map_err(|e| format!("Fallo interno probando red: {e}"))??;
    Ok(format!("Conexion OK con {ip}:{puerto} (sin gastar papel)."))
}

#[tauri::command]
pub async fn imprimir_ticket_venta(
    destino: Destino,
    ticket: TicketVentaPayload,
) -> Result<String, String> {
    if ticket.lineas.is_empty() {
        return Err("El ticket no trae productos.".into());
    }
    if ticket.lineas.len() > super::ticket::MAX_LINEAS {
        return Err(format!(
            "Demasiadas lineas ({}). Limite: {}.",
            ticket.lineas.len(),
            super::ticket::MAX_LINEAS
        ));
    }
    if !ticket.total.is_finite() || ticket.total < 0.0 {
        return Err("Total invalido.".into());
    }

    let tienda = limpiar(&ticket.tienda);
    if tienda.is_empty() {
        return Err("Falta el nombre de la tienda.".into());
    }
    let folio = limpiar(&ticket.folio);
    if folio.is_empty() {
        return Err("Falta el folio del ticket.".into());
    }

    let lineas: Vec<LineaVenta> = ticket
        .lineas
        .into_iter()
        .map(|l| LineaVenta {
            nombre: limpiar(&l.nombre),
            cantidad: if l.cantidad.is_finite() { l.cantidad.max(0.0) } else { 0.0 },
            precio_unitario: if l.precio_unitario.is_finite() {
                l.precio_unitario.max(0.0)
            } else {
                0.0
            },
        })
        .collect();
    let pagos: Vec<(String, f64)> = ticket
        .pagos
        .into_iter()
        .map(|p| {
            let monto = if p.monto.is_finite() { p.monto.max(0.0) } else { 0.0 };
            (limpiar(&p.metodo), monto)
        })
        .filter(|(_, m)| *m > 0.0)
        .collect();
    let pagado: f64 = pagos.iter().map(|(_, m)| m).sum();
    let cambio = (pagado - ticket.total).max(0.0);
    let fecha = ticket
        .fecha
        .map(|f| limpiar(&f))
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| Local::now().format("%Y-%m-%d %H:%M").to_string());

    let venta = TicketVenta {
        tienda,
        ubicacion: ticket.ubicacion.map(|u| limpiar(&u)).filter(|u| !u.is_empty()),
        folio: folio.clone(),
        fecha,
        lineas,
        total: ticket.total,
        pagos,
        cambio,
        qr: ticket.qr.map(|q| limpiar(&q)).filter(|q| !q.is_empty()),
    };
    let bytes = super::ticket::construir_ticket_venta(&venta)?;
    let n = bytes.len();

    match destino {
        Destino::Spooler { nombre } => {
            let nombre_c = limpiar(&nombre);
            if nombre_c.is_empty() {
                return Err("Elige una impresora instalada.".into());
            }
            tokio::task::spawn_blocking(move || enviar_bytes_raw(&nombre_c, &bytes))
                .await
                .map_err(|e| format!("Fallo interno de impresion: {e}"))??;
            tracing::info!(folio = %folio, bytes = n, "ticket vendido por spooler");
            Ok(format!("Ticket {folio} enviado al spooler ({n} bytes)."))
        }
        Destino::Red { ip, puerto } => {
            let ip_c = limpiar(&ip);
            tokio::task::spawn_blocking(move || enviar_red(&ip_c, puerto, &bytes))
                .await
                .map_err(|e| format!("Fallo interno de impresion: {e}"))??;
            tracing::info!(folio = %folio, bytes = n, ip = %ip, "ticket vendido por red");
            Ok(format!("Ticket {folio} enviado a {ip}:{puerto} ({n} bytes)."))
        }
    }
}
