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

use super::builder::{sanitizar, AnchoPapel};
use super::memoria::MemoriaDriver;

/// Tope de lineas por ticket para no saturar el spooler.
pub const MAX_LINEAS: usize = 500;

fn dinero(monto: f64) -> String {
    format!("${:.2}", monto.max(0.0))
}

/// Una linea del ticket: cantidad x producto ... importe NETO.
/// `descuento` es el monto en pesos rebajado a ESTA linea (0 = sin
/// descuento). El importe impreso ya es neto; si hay descuento se
/// agrega renglon `Desc: -$X.XX` para que el cliente vea por que
/// paga menos que la suma de etiquetas.
#[derive(Debug, Clone)]
pub struct LineaVenta {
    pub nombre: String,
    pub cantidad: f64,
    pub precio_unitario: f64,
    pub descuento: f64,
}

/// Todo lo que necesita el ticket de venta. `qr` es opcional
/// (folio o URL de factura); si viene vacio no se imprime QR.
/// `descuento_global` es monto adicional fuera de las lineas
/// (promos, redondeo); 0 = no hay.
#[derive(Debug, Clone)]
pub struct TicketVenta {
    pub tienda: String,
    pub ubicacion: Option<String>,
    pub folio: String,
    pub fecha: String,
    pub lineas: Vec<LineaVenta>,
    pub descuento_global: f64,
    pub total: f64,
    pub pagos: Vec<(String, f64)>,
    pub cambio: f64,
    pub qr: Option<String>,
}

/// Fila monoespaciada: izquierda recortada + importe a la derecha.
fn fila(importe: &str, izquierda: &str, cols: usize) -> String {
    let der = importe.chars().count();
    let max_izq = cols.saturating_sub(der + 1);
    let mut izq: String = izquierda.chars().take(max_izq).collect();
    while izq.chars().count() + der < cols {
        izq.push(' ');
    }
    format!("{izq}{importe}")
}

/// Renderiza el ticket a bytes ESC/POS listos para el spooler o red.
/// Todo texto humano se translitera a ASCII (igual que builder.rs): la
/// termica generica no entiende UTF-8 y "Hernández" saldria basura.
/// El QR NO se toca: es payload, no texto visible.
pub fn construir_ticket_venta(t: &TicketVenta, ancho: AnchoPapel) -> Result<Vec<u8>, String> {
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
    let cols = ancho.cols();
    // QR mas chico en 58mm: el modulo 5 no cabe en 384 puntos.
    let qr_size = match ancho {
        AnchoPapel::Mm80 => 5,
        AnchoPapel::Mm58 => 3,
    };
    let tienda = sanitizar(&t.tienda);
    let ubicacion = t.ubicacion.as_deref().map(sanitizar);

    let driver = MemoriaDriver::default();
    let sonda = driver.clone();

    let mut p = Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()));
    p.init().map_err(|e| format!("INIT termica: {e}"))?;

    // Encabezado
    p.justify(JustifyMode::CENTER)
        .map_err(|e| format!("Ticket: {e}"))?;
    p.bold(true).map_err(|e| format!("Ticket: {e}"))?;
    p.size(2, 2).map_err(|e| format!("Ticket: {e}"))?;
    p.writeln(&tienda).map_err(|e| format!("Ticket: {e}"))?;
    p.bold(false).map_err(|e| format!("Ticket: {e}"))?;
    p.size(1, 1).map_err(|e| format!("Ticket: {e}"))?;
    if let Some(u) = ubicacion.as_deref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
        p.writeln(&u).map_err(|e| format!("Ticket: {e}"))?;
    }
    p.writeln(&format!("Folio {}  {}", sanitizar(&t.folio), sanitizar(&t.fecha)))
        .map_err(|e| format!("Ticket: {e}"))?;
    p.writeln(&"-".repeat(cols))
        .map_err(|e| format!("Ticket: {e}"))?;

    // Lineas (importe NETO; el descuento de la linea se muestra
    // en renglon propio para que el cliente vea la rebaja).
    p.justify(JustifyMode::LEFT)
        .map_err(|e| format!("Ticket: {e}"))?;
    let mut subtotal_bruto = 0.0;
    let mut desc_lineas = 0.0;
    for l in &t.lineas {
        let bruto = l.cantidad.max(0.0) * l.precio_unitario.max(0.0);
        let desc = l.descuento.max(0.0).min(bruto);
        subtotal_bruto += bruto;
        desc_lineas += desc;
        let cant = if l.cantidad.fract() == 0.0 {
            format!("{}x", l.cantidad as i64)
        } else {
            format!("{}x", l.cantidad)
        };
        p.writeln(&fila(
            &dinero(bruto - desc),
            &format!("{cant} {}", sanitizar(&l.nombre)),
            cols,
        ))
        .map_err(|e| format!("Ticket: {e}"))?;
        if desc > 0.0 {
            p.writeln(&fila(&format!("-{}", dinero(desc)), "  Desc:", cols))
                .map_err(|e| format!("Ticket: {e}"))?;
        }
    }
    p.writeln(&"-".repeat(cols))
        .map_err(|e| format!("Ticket: {e}"))?;

    // Subtotal + descuento (solo si hubo) + total + pagos + cambio.
    // El DESCUENTO impreso es lineas + global: lo que el cliente
    // se ahorro contra etiquetas.
    let desc_total = desc_lineas + t.descuento_global.max(0.0);
    if desc_total > 0.0 {
        p.writeln(&fila(&dinero(subtotal_bruto), "SUBTOTAL", cols))
            .map_err(|e| format!("Ticket: {e}"))?;
        p.writeln(&fila(&format!("-{}", dinero(desc_total)), "DESCUENTO", cols))
            .map_err(|e| format!("Ticket: {e}"))?;
        p.writeln(&"-".repeat(cols))
            .map_err(|e| format!("Ticket: {e}"))?;
    }
    p.bold(true).map_err(|e| format!("Ticket: {e}"))?;
    p.size(2, 1).map_err(|e| format!("Ticket: {e}"))?;
    p.writeln(&fila(&dinero(t.total), "TOTAL", cols))
        .map_err(|e| format!("Ticket: {e}"))?;
    p.bold(false).map_err(|e| format!("Ticket: {e}"))?;
    p.size(1, 1).map_err(|e| format!("Ticket: {e}"))?;
    for (metodo, monto) in &t.pagos {
        if *monto > 0.0 {
            p.writeln(&fila(&dinero(*monto), &sanitizar(metodo), cols))
                .map_err(|e| format!("Ticket: {e}"))?;
        }
    }
    if t.cambio > 0.0 {
        p.writeln(&fila(&dinero(t.cambio), "CAMBIO", cols))
            .map_err(|e| format!("Ticket: {e}"))?;
    }

    // QR opcional + despedida + corte
    p.justify(JustifyMode::CENTER)
        .map_err(|e| format!("Ticket: {e}"))?;
    p.feed().map_err(|e| format!("Ticket: {e}"))?;
    if let Some(qr) = t.qr.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        p.qrcode_option(qr, QRCodeOption::new(QRCodeModel::Model2, qr_size, QRCodeCorrectionLevel::M))
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
                descuento: 0.0,
            }],
            descuento_global: 0.0,
            total: 40.0,
            pagos: vec![("Efectivo".into(), 50.0)],
            cambio: 10.0,
            qr: Some("YARVIS-42".into()),
        }
    }

    #[test]
    fn ticket_arranca_con_init_y_trae_folio() {
        let bytes = construir_ticket_venta(&demo(), AnchoPapel::Mm80).expect("render");
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
        let bytes = construir_ticket_venta(&t, AnchoPapel::Mm80).expect("render");
        // print_cut() de escpos 0.20 cierra con GS V A (corte parcial).
        assert!(bytes.ends_with(&[0x1D, 0x56, 0x41, 0x00]), "sin corte");
    }

    #[test]
    fn ticket_vacio_da_error_claro() {
        let mut t = demo();
        t.lineas.clear();
        assert!(construir_ticket_venta(&t, AnchoPapel::Mm80).is_err());
    }

    #[test]
    fn ticket_con_descuento_muestra_subtotal_y_desc() {
        let mut t = demo();
        t.lineas[0].descuento = 5.0;
        t.descuento_global = 3.0;
        t.total = 32.0;
        let bytes = construir_ticket_venta(&t, AnchoPapel::Mm80).expect("render");
        let txt = String::from_utf8_lossy(&bytes);
        assert!(txt.contains("SUBTOTAL"), "falta SUBTOTAL");
        assert!(txt.contains("DESCUENTO"), "falta DESCUENTO");
        assert!(txt.contains("Desc:"), "falta renglon de linea");
        assert!(txt.contains("$40.00"), "falta bruto");
        assert!(txt.contains("-$8.00"), "falta descuento total 5+3");
        assert!(txt.contains("$32.00"), "falta neto");
    }

    #[test]
    fn ticket_sin_descuento_no_muestra_subtotal() {
        let bytes = construir_ticket_venta(&demo(), AnchoPapel::Mm80).expect("render");
        let txt = String::from_utf8_lossy(&bytes);
        assert!(
            !txt.contains("SUBTOTAL"),
            "sin descuento no debe salir SUBTOTAL"
        );
        assert!(
            !txt.contains("DESCUENTO"),
            "sin descuento no debe salir DESCUENTO"
        );
    }

    #[test]
    fn ticket_translitera_tildes_en_papel() {
        // La termica generica no entiende UTF-8: "Hernández/Café" debe
        // salir "Hernandez/Cafe", nunca bytes crudos multibyte.
        let mut t = demo();
        t.tienda = "Abarrotes Hernández".into();
        t.lineas[0].nombre = "Café 500g".into();
        let bytes = construir_ticket_venta(&t, AnchoPapel::Mm80).expect("render");
        let txt = String::from_utf8_lossy(&bytes);
        assert!(txt.contains("Hernandez"), "tienda sin transliterar");
        assert!(txt.contains("Cafe 500g"), "producto sin transliterar");
        assert!(!txt.contains('é'), "quedo UTF-8 crudo en el papel");
    }

    #[test]
    fn ticket_58mm_usa_separadores_de_32() {
        let bytes = construir_ticket_venta(&demo(), AnchoPapel::Mm58).expect("render");
        assert!(bytes.starts_with(&[0x1B, 0x40]), "sin INIT");
        let txt = String::from_utf8_lossy(&bytes);
        assert!(txt.contains(&"-".repeat(32)), "sin separador de 32");
        assert!(!txt.contains(&"-".repeat(33)), "se colo un separador de 80mm");
    }
}
