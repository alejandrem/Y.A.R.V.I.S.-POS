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

use super::builder::{self, AnchoPapel, FilaConciliacion};
use super::ticket::{LineaVenta, TicketVenta};
use super::{enviar_bytes_raw, listar_impresoras_sistema};

/// Tope de caracteres del QR: mas alla el modulo crece tanto que no cabe
/// en el papel (ni en 80mm) e infla el trabajo al tope de 512KB.
const QR_MAX_CHARS: usize = 500;

/// Resuelve el ancho de papel del payload (80/58, default 80).
fn ancho_de(mm: Option<u8>) -> AnchoPapel {
    AnchoPapel::desde_mm(mm.unwrap_or(80))
}

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
    ancho_mm: Option<u8>,
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
            // El sistema NO se aplana: un negativo por sobreventa es la
            // señal que el builder marca como CONCIL.
            sistema: f.sistema,
            precio_venta: if f.precio_venta.is_finite() {
                f.precio_venta.max(0.0)
            } else {
                0.0
            },
        })
        .collect();

    let fecha = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let bytes = builder::construir_lista_conciliacion(&tienda, &fecha, &filas, ancho_de(ancho_mm));
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

/// Fila tal como la manda el frontend para la alerta de stock bajo.
/// El stock puede venir negativo por sobreventa: se imprime tal cual.
#[derive(Debug, Clone, Deserialize)]
pub struct FilaStockBajoPayload {
    pub nombre: String,
    pub stock: f64,
    pub minimo: f64,
}

#[tauri::command]
pub async fn imprimir_lista_stock_bajo(
    nombre_impresora: String,
    tienda: Option<String>,
    filas: Vec<FilaStockBajoPayload>,
    ancho_mm: Option<u8>,
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
    let filas: Vec<builder::FilaStockBajo> = filas
        .into_iter()
        .map(|f| builder::FilaStockBajo {
            nombre: f.nombre,
            stock: if f.stock.is_finite() { f.stock } else { 0.0 },
            minimo: if f.minimo.is_finite() {
                f.minimo.max(0.0)
            } else {
                0.0
            },
        })
        .collect();

    let fecha = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let bytes = builder::construir_lista_stock_bajo(&tienda, &fecha, &filas, ancho_de(ancho_mm));
    let n = bytes.len();

    tokio::task::spawn_blocking(move || enviar_bytes_raw(&nombre, &bytes))
        .await
        .map_err(|e| format!("Fallo interno de impresion: {e}"))??;

    tracing::info!(impresora = %nombre_impresora, filas = filas.len(), "stock bajo impreso");
    Ok(format!(
        "Lista enviada a '{}' ({} criticos, {} bytes).",
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
    /// Descuento en pesos de esta linea. `default` = 0 (frontends viejos).
    #[serde(default)]
    pub descuento: f64,
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
    /// Descuento global en pesos (fuera de las lineas). `default` = 0.
    #[serde(default)]
    pub descuento_global: f64,
    /// Ancho de papel en mm (80/58). `None` = 80mm.
    #[serde(default)]
    pub ancho_mm: Option<u8>,
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
    // Misma validacion que checar_red (antes daba "IP/host invalido ''").
    if ip.trim().is_empty() {
        return Err("Escribe la IP de la impresora de red.".into());
    }
    if puerto == 0 {
        return Err("Puerto invalido (tipico: 9100).".into());
    }
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

    let mut subtotal_bruto = 0.0;
    let mut desc_lineas = 0.0;
    let mut lineas: Vec<LineaVenta> = Vec::with_capacity(ticket.lineas.len());
    for l in ticket.lineas {
        let cantidad = if l.cantidad.is_finite() {
            l.cantidad.max(0.0)
        } else {
            0.0
        };
        let precio = if l.precio_unitario.is_finite() {
            l.precio_unitario.max(0.0)
        } else {
            0.0
        };
        let bruto = cantidad * precio;
        let desc = if l.descuento.is_finite() {
            l.descuento.max(0.0).min(bruto)
        } else {
            0.0
        };
        subtotal_bruto += bruto;
        desc_lineas += desc;
        lineas.push(LineaVenta {
            nombre: limpiar(&l.nombre),
            cantidad,
            precio_unitario: precio,
            descuento: desc,
        });
    }
    let desc_global = if ticket.descuento_global.is_finite() {
        ticket.descuento_global.max(0.0)
    } else {
        0.0
    };
    if desc_lineas + desc_global > subtotal_bruto {
        return Err("Descuentos mayores al subtotal del ticket.".into());
    }
    let pagos: Vec<(String, f64)> = ticket
        .pagos
        .into_iter()
        .map(|p| {
            let monto = if p.monto.is_finite() {
                p.monto.max(0.0)
            } else {
                0.0
            };
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

    let qr = ticket.qr.map(|q| limpiar(&q)).filter(|q| !q.is_empty());
    if let Some(q) = &qr {
        if q.chars().count() > QR_MAX_CHARS {
            return Err(format!(
                "QR demasiado largo ({} caracteres, maximo {QR_MAX_CHARS}). Usa un folio corto o URL corta.",
                q.chars().count()
            ));
        }
    }

    let venta = TicketVenta {
        tienda,
        ubicacion: ticket.ubicacion.map(|u| limpiar(&u)).filter(|u| !u.is_empty()),
        folio: folio.clone(),
        fecha,
        lineas,
        descuento_global: desc_global,
        total: ticket.total,
        pagos,
        cambio,
        qr,
    };
    let ancho = ancho_de(ticket.ancho_mm);
    let bytes = super::ticket::construir_ticket_venta(&venta, ancho)?;
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

/// Pulso de apertura de cajon (ESC p) por spooler o red.
/// El cajon va conectado por RJ11 a la termica: no necesita driver
/// ni impresora aparte, sale por el mismo canal que el ticket.
#[tauri::command]
pub async fn abrir_cajon(destino: Destino) -> Result<String, String> {
    let bytes = builder::construir_apertura_cajon();
    match destino {
        Destino::Spooler { nombre } => {
            let nombre_c = limpiar(&nombre);
            if nombre_c.is_empty() {
                return Err("Elige una impresora instalada.".into());
            }
            tokio::task::spawn_blocking(move || enviar_bytes_raw(&nombre_c, &bytes))
                .await
                .map_err(|e| format!("Fallo interno abriendo cajon: {e}"))??;
            tracing::info!(impresora = %nombre, "cajon abierto por spooler");
            Ok("Pulso de apertura enviado al cajon.".to_string())
        }
        Destino::Red { ip, puerto } => {
            let ip_c = limpiar(&ip);
            tokio::task::spawn_blocking(move || enviar_red(&ip_c, puerto, &bytes))
                .await
                .map_err(|e| format!("Fallo interno abriendo cajon: {e}"))??;
            tracing::info!(ip = %ip, "cajon abierto por red");
            Ok(format!("Pulso de apertura enviado a {ip}:{puerto}."))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    /// Puerto cerrado garantizado: se abre un listener, se toma su puerto
    /// y se suelta (nadie lo ocupa en ese instante).
    fn puerto_cerrado() -> u16 {
        std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    #[test]
    fn red_loopback_recibe_bytes_exactos() {
        // Transporte de red real (sin impresora): lo que entra por un
        // extremo sale identico por el otro.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let puerto = listener.local_addr().unwrap().port();
        let esperados = builder::construir_apertura_cajon();
        let clon = esperados.clone();
        let hilo = std::thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let mut buf = Vec::new();
            s.read_to_end(&mut buf).unwrap();
            buf
        });
        enviar_red("127.0.0.1", puerto, &clon).unwrap();
        let recibidos = hilo.join().unwrap();
        assert_eq!(recibidos, esperados);
    }

    #[test]
    fn red_sin_ip_o_puerto_da_error_claro() {
        let e = enviar_red("", 9100, &[1, 2, 3]).unwrap_err();
        assert!(e.contains("IP"), "mensaje confuso: {e}");
        let e = enviar_red("127.0.0.1", 0, &[1, 2, 3]).unwrap_err();
        assert!(e.contains("Puerto"), "mensaje confuso: {e}");
        let e = enviar_red("127.0.0.1", puerto_cerrado(), &[1, 2, 3]).unwrap_err();
        assert!(e.contains("Sin conexion"), "mensaje confuso: {e}");
    }

    #[test]
    fn checar_red_rechazada_da_error_claro() {
        let e = checar_red("127.0.0.1", puerto_cerrado()).unwrap_err();
        assert!(e.contains("Sin conexion"), "mensaje confuso: {e}");
        assert!(checar_red("", 9100).is_err());
    }

    #[test]
    fn ancho_raro_cae_a_80mm() {
        assert_eq!(ancho_de(None), AnchoPapel::Mm80);
        assert_eq!(ancho_de(Some(80)), AnchoPapel::Mm80);
        assert_eq!(ancho_de(Some(58)), AnchoPapel::Mm58);
        assert_eq!(ancho_de(Some(0)), AnchoPapel::Mm80);
    }
}
