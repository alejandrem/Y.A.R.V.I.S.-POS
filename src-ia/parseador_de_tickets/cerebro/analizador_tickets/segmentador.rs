// ============================================================
// segmentador — Un archivo → N tickets (segmentación de bloques)
// ============================================================
//
// Resuelve el bug "1 archivo = 1 venta": un archivo con N tickets
// concatenados ahora se divide en N segmentos, cada uno con su propio
// folio, fecha/hora y cajero.
//
// El algoritmo:
//   * Pass 1: clasificar cada línea con `es_linea_util` (producto vs
//     separador). Solo las líneas SEPARADOR son candidatas a marcador.
//   * Pass 2: sobre las líneas crudas (antes del filtro — el folio/fecha
//     hoy se descartan en es_linea_util):
//       - Marcadores de APERTURA (FOLIO, TICKET #, NO. TICKET, SERIE o una
//         fecha) inician un bloque.
//       - Marcadores de CIERRE (TOTAL, GRACIAS POR SU COMPRA, EFECTIVO
//         RECIBIDO, CAMBIO o línea de pago) cierran el bloque.
//   * Sin ningún marcador → un solo segmento con todas las líneas
//     (retrocompatible con el comportamiento actual).
//
// Notas:
//   - Los marcadores SE INCLUYEN en `lineas` del segmento: parsear_linea
//     los descarta igual, y extraer_fecha_hora/extraer_cajero leen el
//     bloque completo sin perder el TOTAL de pie.
//   - Los marcadores consecutivos de apertura (ej. "FOLIO" + fecha del
//     mismo ticket) se acumulan en UN solo bloque mientras no lleguen
//     líneas de producto, para no partir cada ticket en pedazos.
//   - El pie del ticket (METODO DE PAGO, CFDI, GRACIAS, CAMBIO...) va
//     DESPUÉS del TOTAL: se anexa al último bloque cerrado para no perder
//     el método de pago; solo líneas de PRODUCTO desconectadas se descartan.

use regex::Regex;
use std::sync::LazyLock;

use super::{
    es_linea_util, extraer_cajero, extraer_fecha_hora_regex, extraer_metodo_pago, tiene_fecha,
};

/// Un ticket dentro de un archivo: bloque de líneas crudas + metadatos.
#[derive(Debug, Clone, PartialEq)]
pub struct TicketSegmento {
    /// Orden del ticket dentro del archivo (1 = primero).
    pub index: usize,
    /// Líneas crudas del bloque (apertura y cierre incluidas), sin vacías.
    pub lineas: Vec<String>,
    /// Folio/número de ticket de la apertura (None si no se detectó).
    pub folio: Option<String>,
    /// Fecha/hora del bloque en formato "YYYY-MM-DD HH:MM:00" (None si no hay).
    pub fecha_hora: Option<String>,
    /// Cajero/empleado del bloque ("SISTEMA" si no se detecta).
    pub cajero: String,
    /// Método de pago del bloque (extraído de sus últimas 25 líneas).
    pub metodo_pago: String,
}

impl TicketSegmento {
    /// Texto completo del bloque (para extraer fecha/cajero/pago o parsear).
    pub fn texto(&self) -> String {
        self.lineas.join("\n")
    }
}

// ---------------------------------------------------------------------------
// Detección de marcadores (solo se evalúa sobre líneas NO útiles)
// ---------------------------------------------------------------------------

/// Apertura: "FOLIO", "SERIE", "TICKET #"... o cualquier línea con fecha.
static RE_APERTURA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(?:folio|serie|ticket)\b").expect("regex apertura"));

/// Cierre: la palabra TOTAL como encabezado. "SUBTOTAL" NO cierra: `\b`
/// exige un límite de palabra, y dentro de "subtotal" no hay ningún límite
/// entre "total" y la letra previa.
static RE_TOTAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\btotal\b").expect("regex total"));
static RE_GRACIAS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bgracias\b").expect("regex gracias"));
static RE_EFECTIVO_RECIBIDO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\befectivo\s+recibido\b").expect("regex efectivo recibido"));
/// "CAMBIO $X" / "CAMBIO: $X" (evita "CAMBIO DE ACEITE" como cierre).
static RE_CAMBIO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?:^|\b)cambio(?:$|[\s.:]+[$0-9])").expect("regex cambio"));
/// Línea de pago del pie ("EFECTIVO $500", "TARJETA DEBITO 123.45"...).
static RE_PAGO_FOOTER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(?:efectivo|tarjeta\s+(?:debito|credito)|debito|credito|transferencia|cheque)\s*[:.\s]+[$0-9]",
    )
    .expect("regex pago pie")
});

fn es_apertura(linea: &str) -> bool {
    tiene_fecha(linea) || RE_APERTURA.is_match(linea)
}

fn es_cierre(linea: &str) -> bool {
    RE_TOTAL.is_match(linea)
        || RE_GRACIAS.is_match(linea)
        || RE_EFECTIVO_RECIBIDO.is_match(linea)
        || RE_CAMBIO.is_match(linea)
        || RE_PAGO_FOOTER.is_match(linea)
}

/// Extrae el folio/número de ticket de una línea de apertura.
/// "FOLIO: 004582" → "004582", "TICKET # A-123" → "A-123",
/// "NO. TICKET: 0002" → "0002", "SERIE A-123" → "A-123".
/// Una línea que solo trae fecha devuelve None (no hay folio).
fn extraer_folio(linea: &str) -> Option<String> {
    static RE_FOLIO: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(?:FOLIO|NO\.?\s*TICKET|TICKET\s*#?|SERIE)\s*[:.#]?\s*([A-Za-z0-9\-]+)")
            .expect("regex folio")
    });
    RE_FOLIO.captures(linea).map(|c| c[1].to_string())
}

// ---------------------------------------------------------------------------
// Segmentación
// ---------------------------------------------------------------------------

fn fecha_hora_de_bloque(texto: &str) -> Option<String> {
    let (fecha, hora) = extraer_fecha_hora_regex(texto);
    fecha.map(|f| match &hora {
        Some(h) => format!("{f} {h}:00"),
        None => format!("{f} 00:00:00"),
    })
}

/// Divide un archivo de tickets en segmentos.
///
/// Un bloque se construye así:
///   - una línea de APERTURA inicia un bloque (si ya había uno CON productos,
///     el anterior se cierra y se inicia uno nuevo);
///   - una línea de CIERRE cierra el bloque actual;
///   - las demás líneas se acumulan dentro del bloque actual;
///   - si un bloque terminó y llega un marcador de cierre suelto, se ignora.
pub fn segmentar(texto: &str) -> Vec<TicketSegmento> {
    let lineas: Vec<String> = texto
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    if lineas.is_empty() {
        return Vec::new();
    }

    // Bloques detectados: (líneas del bloque, folio de la apertura).
    let mut bloques: Vec<(Vec<String>, Option<String>)> = Vec::new();
    // Bloque en construcción: (líneas, folio, ¿recibió ya líneas de producto?).
    let mut actual: Option<(Vec<String>, Option<String>, bool)> = None;
    // Índice del último bloque cerrado: el pie que venga después del cierre
    // (METODO DE PAGO, CFDI, GRACIAS...) se anexa ahí.
    let mut ultimo_cerrado: Option<usize> = None;

    for linea in &lineas {
        let es_separador = !es_linea_util(linea);

        if es_separador && es_apertura(linea) {
            match actual.take() {
                Some((bloque, folio, con_contenido)) if con_contenido => {
                    // Ya había productos en el bloque: se cierra y este
                    // marcador inicia el siguiente ticket.
                    bloques.push((bloque, folio));
                    actual = Some((vec![linea.clone()], extraer_folio(linea), false));
                }
                Some((mut bloque, folio, _)) => {
                    // Aún sin productos: es la continuación del mismo
                    // encabezado (ej. "FOLIO" + fecha del mismo ticket).
                    bloque.push(linea.clone());
                    actual = Some((bloque, folio.or(extraer_folio(linea)), false));
                }
                None => {
                    actual = Some((vec![linea.clone()], extraer_folio(linea), false));
                }
            }
            ultimo_cerrado = None;
            continue;
        }

        if es_separador && es_cierre(linea) {
            if let Some((mut bloque, folio, _)) = actual.take() {
                bloque.push(linea.clone());
                bloques.push((bloque, folio));
                ultimo_cerrado = Some(bloques.len() - 1);
            } else if let Some(idx) = ultimo_cerrado {
                // Cierre suelto (TOTAL extra, GRACIAS, CAMBIO...) → pie.
                bloques[idx].0.push(linea.clone());
            }
            continue;
        }

        if let Some((bloque, _, con_contenido)) = actual.as_mut() {
            if es_separador {
                // Separador dentro del encabezado: no cuenta como producto.
                bloque.push(linea.clone());
            } else {
                bloque.push(linea.clone());
                *con_contenido = true;
            }
            continue;
        }

        // Sin bloque abierto: solo el pie/separador se anexa al último
        // bloque cerrado; una línea de PRODUCTO desconectada se descarta.
        if es_separador {
            if let Some(idx) = ultimo_cerrado {
                bloques[idx].0.push(linea.clone());
            }
        }
    }

    if let Some((bloque, folio, _)) = actual.take() {
        bloques.push((bloque, folio));
    }

    // Retrocompatibilidad: sin marcadores → un solo segmento con todo.
    if bloques.is_empty() {
        bloques.push((lineas, None));
    }

    bloques
        .into_iter()
        .enumerate()
        .map(|(i, (bloque, folio))| {
            let texto_bloque = bloque.join("\n");
            TicketSegmento {
                index: i + 1,
                lineas: bloque,
                folio,
                fecha_hora: fecha_hora_de_bloque(&texto_bloque),
                cajero: extraer_cajero(&texto_bloque),
                metodo_pago: extraer_metodo_pago(&texto_bloque),
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dos_tickets_con_folio_y_total_generan_dos_segmentos() {
        let texto = "FOLIO: 0001\n\
                     12/05/2026\n\
                     2 COCA $25.00 $50.00\n\
                     TOTAL $50.00\n\
                     FOLIO: 0002\n\
                     13/05/2026\n\
                     1 PAN $10.00 $10.00\n\
                     TOTAL $10.00\n";

        let segs = segmentar(texto);
        assert_eq!(segs.len(), 2);

        assert_eq!(segs[0].index, 1);
        assert_eq!(segs[0].folio.as_deref(), Some("0001"));
        assert_eq!(segs[0].fecha_hora.as_deref(), Some("2026-05-12 00:00:00"));
        assert!(segs[0].lineas.iter().any(|l| l.contains("2 COCA")));
        assert!(!segs[0].lineas.iter().any(|l| l.contains("FOLIO: 0002")));

        assert_eq!(segs[1].index, 2);
        assert_eq!(segs[1].folio.as_deref(), Some("0002"));
        assert_eq!(segs[1].fecha_hora.as_deref(), Some("2026-05-13 00:00:00"));
        assert!(segs[1].lineas.iter().any(|l| l.contains("1 PAN")));
    }

    #[test]
    fn sin_marcadores_un_solo_segmento_retrocompatible() {
        let texto = "2 COCA $25.00 $50.00\n1 PAN $10.00 $10.00\n";

        let segs = segmentar(texto);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].index, 1);
        assert_eq!(segs[0].folio, None);
        assert_eq!(segs[0].fecha_hora, None);
        assert_eq!(segs[0].lineas.len(), 2);
    }

    #[test]
    fn fecha_nueva_inicia_ticket_sin_folio() {
        let texto = "15/05/2026\n2 COCA $25.00 $50.00\nTOTAL $50.00\n\
                     16/05/2026\n1 PAN $10.00 $10.00\nTOTAL $10.00\n";

        let segs = segmentar(texto);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].folio, None);
        assert_eq!(segs[0].fecha_hora.as_deref(), Some("2026-05-15 00:00:00"));
        assert_eq!(segs[1].folio, None);
        assert_eq!(segs[1].fecha_hora.as_deref(), Some("2026-05-16 00:00:00"));
    }

    #[test]
    fn marcadores_consecutivos_se_acumulan_en_un_solo_encabezado() {
        let texto = "FOLIO: 88\n12/05/2026\n2 COCA $25.00 $50.00\nTOTAL $50.00\n";

        let segs = segmentar(texto);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].folio.as_deref(), Some("88"));
        assert_eq!(segs[0].fecha_hora.as_deref(), Some("2026-05-12 00:00:00"));
        assert_eq!(segs[0].lineas.len(), 4);
    }

    #[test]
    fn ticket_real_con_encabezado_folio_y_hora() {
        let texto = "Farmacia San Pablo\n\
                     Av. Juzarez 123, CDMX\n\
                     Ticket: 004582\n\
                     Fecha: 15/03/2024  14:32\n\
                     -----------------------------------\n\
                     2 Pan Bimbo Integral         42.00     84.00\n\
                     1 Leche Lala Light 1L        26.50     26.50\n\
                     -----------------------------------\n\
                     SUBTOTAL: 166.00\n\
                     IVA 16%: 26.56\n\
                     TOTAL: $192.56\n\
                     Tarjeta: **** 1234\n\
                     Gracias por su compra\n";

        let segs = segmentar(texto);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].folio.as_deref(), Some("004582"));
        assert_eq!(segs[0].fecha_hora.as_deref(), Some("2024-03-15 14:32:00"));
        assert!(segs[0].lineas.iter().any(|l| l.contains("Pan Bimbo")));
        // El bloque va de "Ticket:" hasta "TOTAL" y se queda el pie (pago/gracias).
        assert!(segs[0].lineas.first().unwrap().contains("Ticket:"));
        assert!(segs[0].lineas.iter().any(|l| l.contains("TOTAL:")));
        assert!(segs[0].lineas.iter().any(|l| l.contains("Gracias")));
    }

    #[test]
    fn cierre_sin_apertura_se_ignora() {
        let texto = "FOLIO: 1\n2 COCA $25.00 $50.00\nTOTAL $50.00\n\
                     GRACIAS POR SU COMPRA\n2 PAN $10.00 $10.00\n";

        let segs = segmentar(texto);
        assert_eq!(segs.len(), 1);
        // El segundo "ticket" sin encabezado (tras el cierre) queda fuera.
        assert!(!segs[0].lineas.iter().any(|l| l.contains("PAN")));
    }

    #[test]
    fn texto_vacio_no_genera_segmentos() {
        assert!(segmentar("   \n  \n").is_empty());
        assert!(segmentar("").is_empty());
    }

    #[test]
    fn extraer_folio_soporta_todos_los_formatos() {
        assert_eq!(extraer_folio("FOLIO: 004582").as_deref(), Some("004582"));
        assert_eq!(extraer_folio("TICKET # A-123").as_deref(), Some("A-123"));
        assert_eq!(extraer_folio("Ticket: 004582").as_deref(), Some("004582"));
        assert_eq!(extraer_folio("NO. TICKET: 0002").as_deref(), Some("0002"));
        assert_eq!(extraer_folio("SERIE A-123").as_deref(), Some("A-123"));
        assert_eq!(extraer_folio("12/05/2026"), None);
    }

    #[test]
    fn metodo_pago_se_extrae_de_cada_segmento() {
        let texto = "FOLIO: 1\n12/05/2026\n1 COCA $25.00 $25.00\nTOTAL $25.00\n\
                     METODO DE PAGO: TARJETA DEBITO\n\
                     FOLIO: 2\n13/05/2026\n1 PAN $10.00 $10.00\nTOTAL $10.00\n\
                     FORMA DE PAGO: EFECTIVO\n";

        let segs = segmentar(texto);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].metodo_pago, "tarjeta");
        assert_eq!(segs[1].metodo_pago, "efectivo");
    }
}
