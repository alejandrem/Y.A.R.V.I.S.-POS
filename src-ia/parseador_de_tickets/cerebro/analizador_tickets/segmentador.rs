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
//     separador). Solo las líneas SEPARADOR son candidatas a marcador
//     (más la apertura "fuerte", que vale aunque parezca producto).
//   * Pass 2: sobre las líneas crudas (antes del filtro — el folio/fecha
//     hoy se descartan en es_linea_util):
//       - Marcadores de APERTURA (folio extraíble o fecha) inician un bloque.
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
// ============================================================
//
// Organización (un tema por archivo):
//   * marcadores.rs → aperturas, cierres y extracción de folio
//   * clave.rs       → identidad del ticket (`clave`, hash, orden cronológico)
//   * tests.rs       → suite del segmentador
// Este archivo solo define `TicketSegmento` y orquesta `segmentar`.

mod clave;
mod marcadores;
#[cfg(test)]
mod tests;

pub use clave::{comparar_cronologico, fnv1a64};
pub(crate) use marcadores::{es_apertura, es_apertura_fuerte, es_cierre, extraer_folio};

use super::{es_linea_util, extraer_cajero, extraer_fecha_hora_regex, extraer_metodo_pago};

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
        // La apertura fuerte vale aunque la línea parezca de producto.
        let abre_bloque = es_apertura_fuerte(linea) || (es_separador && es_apertura(linea));

        if abre_bloque {
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
