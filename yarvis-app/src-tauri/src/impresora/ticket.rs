// ============================================================
// impresora/ticket.rs — Ticket de venta enriquecido (Camino C Fase 2).
//
// El ticket se arma con la API del crate `escpos` (formato, QR)
// sobre `MemoriaDriver`; los bytes resultantes viajan por spooler
// RAW (Fase 1) o TCP directo. La conciliacion de inventario sigue
// en `builder.rs` sin cambios: lo que funciona no se toca.
// ============================================================

use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::{
    JustifyMode, Protocol, QRCodeCorrectionLevel, QRCodeModel, QRCodeOption,
};

use super::memoria::MemoriaDriver;

/// Ancho util de termica 80mm en fuente A (igual que builder.rs).
const COLS: usize = 48;
/// Tope de lineas por ticket para no saturar el spooler.
pub const MAX_LINEAS: usize = 500;

fn dinero(monto: f64) -> String {
    format!("${:.2}", monto.max(0.0))
}

/// Una linea del ticket: cantidad x producto ... importe.
#[derive(Debug, Clone)]
pub struct LineaVenta {
    pub nombre: String,
    pub cantidad: f64,
    pub precio_unitario: f64,
}

/// Todo lo que necesita el ticket de venta. `qr` es opcional
/// (folio o URL de factura); si viene vacio no se imprime QR.
#[derive(Debug, Clone)]
pub struct TicketVenta {
    pub tienda: String,
    pub ubicacion: Option<String>,
    pub folio: String,
    pub fecha: String,
    pub lineas: Vec<LineaVenta>,
    pub total: f64,
    pub pagos: Vec<(String, f64)>,
    pub cambio: f64,
    pub qr: Option<String>,
}

/// Fila monoespaciada: izquierda recortada + importe a la derecha.
fn fila(importe: &str, izquierda: &str) -> String {
    let der = importe.chars().count();
    let max_izq = COLS.saturating_sub(der + 1);
    let mut izq: String = izquierda.chars().take(max_izq).collect();
    while izq.chars().count() + der < COLS {
        izq.push(' ');
    }
    format!("{izq}{importe}")
}

/// Renderiza el ticket a bytes ESC/POS listos para el spooler o red.
pub fn construir_ticket_venta(t: &TicketVenta) -> Result<Vec<u8>, String> {
    if t.lineas.is_empty() {
        return Err("El ticket no trae productos.".into());
    }
    if t.lineas.len() > MAX_LINEAS {
        return Err(format!(
            "Demasiadas lineas ({}). Limite Fase 2: {}.",
            t.lineas.len(),
            MAX_LINEAS
        ));
    }

    let driver = MemoriaDriver::default();
    let sonda = driver.clone();

    let mut p = Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()));
    p.init().map_err(|e| format!("INIT termica: {e}"))?;

    // Encabezado
    p.justify(JustifyMode::CENTER)
        .map_err(|e| format!("Ticket: {e}"))?;
    p.bold(true).map_err(|e| format!("Ticket: {e}"))?;
    p.size(2, 2).map_err(|e| format!("Ticket: {e}"))?;
    p.writeln(&t.tienda).map_err(|e| format!("Ticket: {e}"))?;
    p.bold(false).map_err(|e| format!("Ticket: {e}"))?;
    p.size(1, 1).map_err(|e| format!("Ticket: {e}"))?;
    if let Some(u) = t.ubicacion.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        p.writeln(u).map_err(|e| format!("Ticket: {e}"))?;
    }
    p.writeln(&format!("Folio {}  {}", t.folio, t.fecha))
        .map_err(|e| format!("Ticket: {e}"))?;
    p.writeln(&"-".repeat(COLS))
        .map_err(|e| format!("Ticket: {e}"))?;

    // Lineas
    p.justify(JustifyMode::LEFT)
        .map_err(|e| format!("Ticket: {e}"))?;
    for l in &t.lineas {
        let importe = l.cantidad.max(0.0) * l.precio_unitario.max(0.0);
        let cant = if l.cantidad.fract() == 0.0 {
            format!("{}x", l.cantidad as i64)
        } else {
            format!("{}x", l.cantidad)
        };
        p.writeln(&fila(&dinero(importe), &format!("{cant} {}", l.nombre)))
            .map_err(|e| format!("Ticket: {e}"))?;
    }
    p.writeln(&"-".repeat(COLS))
        .map_err(|e| format!("Ticket: {e}"))?;

    // Total + pagos + cambio
    p.bold(true).map_err(|e| format!("Ticket: {e}"))?;
    p.size(2, 1).map_err(|e| format!("Ticket: {e}"))?;
    p.writeln(&fila(&dinero(t.total), "TOTAL"))
        .map_err(|e| format!("Ticket: {e}"))?;
    p.bold(false).map_err(|e| format!("Ticket: {e}"))?;
    p.size(1, 1).map_err(|e| format!("Ticket: {e}"))?;
    for (metodo, monto) in &t.pagos {
        if *monto > 0.0 {
            p.writeln(&fila(&dinero(*monto), metodo))
                .map_err(|e| format!("Ticket: {e}"))?;
        }
    }
    if t.cambio > 0.0 {
        p.writeln(&fila(&dinero(t.cambio), "CAMBIO"))
            .map_err(|e| format!("Ticket: {e}"))?;
    }

    // QR opcional + despedida + corte
    p.justify(JustifyMode::CENTER)
        .map_err(|e| format!("Ticket: {e}"))?;
    p.feed().map_err(|e| format!("Ticket: {e}"))?;
    if let Some(qr) = t.qr.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        p.qrcode_option(qr, QRCodeOption::new(QRCodeModel::Model2, 5, QRCodeCorrectionLevel::M))
            .map_err(|e| format!("QR: {e}"))?;
        p.feed().map_err(|e| format!("Ticket: {e}"))?;
    }
    p.writeln("Gracias por su compra")
        .map_err(|e| format!("Ticket: {e}"))?;
    p.feed().map_err(|e| format!("Ticket: {e}"))?;
    p.print_cut().map_err(|e| format!("Corte: {e}"))?;

    let bytes = sonda.bytes();
    if bytes.len() > 512 * 1024 {
        return Err("Ticket demasiado grande (limite 512KB).".into());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo() -> TicketVenta {
        TicketVenta {
            tienda: "Abarrotes Demo".into(),
            ubicacion: Some("Calle 1".into()),
            folio: "#42".into(),
            fecha: "2026-09-10 12:00".into(),
            lineas: vec![LineaVenta {
                nombre: "Coca-Cola 600ml".into(),
                cantidad: 2.0,
                precio_unitario: 20.0,
            }],
            total: 40.0,
            pagos: vec![("Efectivo".into(), 50.0)],
            cambio: 10.0,
            qr: Some("YARVIS-42".into()),
        }
    }

    #[test]
    fn ticket_arranca_con_init_y_trae_folio() {
        let bytes = construir_ticket_venta(&demo()).expect("render");
        assert!(bytes.starts_with(&[0x1B, 0x40]), "sin INIT");
        assert!(bytes.len() > 100, "ticket sospechosamente chico");
        let txt = String::from_utf8_lossy(&bytes);
        assert!(txt.contains("Abarrotes Demo"));
        assert!(txt.contains("#42"));
        assert!(txt.contains("CAMBIO"));
    }

    #[test]
    fn ticket_sin_qr_tambien_corta() {
        let mut t = demo();
        t.qr = None;
        t.ubicacion = None;
        let bytes = construir_ticket_venta(&t).expect("render");
        // print_cut() de escpos 0.20 cierra con GS V A (corte parcial).
        assert!(bytes.ends_with(&[0x1D, 0x56, 0x41, 0x00]), "sin corte");
    }

    #[test]
    fn ticket_vacio_da_error_claro() {
        let mut t = demo();
        t.lineas.clear();
        assert!(construir_ticket_venta(&t).is_err());
    }
}
