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

    /// Clave de idempotencia del ticket, en 3 niveles de prioridad:
    ///   1. Folio impreso (manda: es la identidad que dio la tienda).
    ///   2. Fecha+hora → folio automático `AUTO-YYYYMMDD-HHMM-xxxxxx`
    ///      (legible, ordenable cronológicamente y determinista: el mismo
    ///      ticket re-importado genera el mismo folio).
    ///   3. Sin fecha → `SIN-FOLIO-<hash>` del contenido.
    ///
    /// Antes, un ticket sin folio detectable NO se podía deduplicar y
    /// re-importar la carpeta duplicaba ventas y descontaba stock dos
    /// veces. Límite honesto: dos ventas DISTINTAS con mismo contenido,
    /// misma fecha y sin folio colisionarían; el folio impreso sigue
    /// siendo la identificación preferible.
    pub fn clave(&self) -> String {
        // 1. Folio impreso.
        if let Some(f) = self.folio.as_deref().map(str::trim).filter(|f| !f.is_empty()) {
            return f.to_string();
        }
        // Hash corto del contenido: distingue dos ventas del mismo minuto.
        let hash_corto = format!("{:012x}", fnv1a64(&self.base_hash()) >> 16);
        // 2. Folio automático desde fecha+hora ("2026-03-04 20:11:00" →
        // "AUTO-20260304-2011-xxxxxx"). Ancho fijo: orden lexicográfico =
        // orden cronológico.
        if let Some(fh) = self.fecha_hora.as_deref() {
            let digitos: String = fh.chars().filter(|c| c.is_ascii_digit()).collect();
            if digitos.len() >= 12 {
                return format!("AUTO-{}-{}-{hash_corto}", &digitos[..8], &digitos[8..12]);
            }
        }
        // 3. Último recurso: hash del contenido (estable entre corridas).
        format!("SIN-FOLIO-{:016x}", fnv1a64(&self.base_hash()))
    }

    /// Contenido normalizado para el hash: espacios colapsados + fecha.
    /// Dos importaciones del mismo ticket dan la misma base (y por tanto
    /// la misma clave); distinto contenido o distinta fecha, distinta clave.
    fn base_hash(&self) -> String {
        let normalizado: Vec<String> = self
            .lineas
            .iter()
            .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
            .collect();
        format!(
            "{}|{}",
            normalizado.join("\n"),
            self.fecha_hora.as_deref().unwrap_or("")
        )
    }
}

/// Hash FNV-1a de 64 bits: estable entre corridas y procesos (a
/// diferencia de `DefaultHasher`, que usa semilla aleatoria por proceso
/// y NO sirve para deduplicar entre una corrida y la siguiente).
pub fn fnv1a64(texto: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in texto.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Orden cronológico de segmentos para inserción: por fecha/hora (el
/// formato ISO de ancho fijo hace que comparar strings = comparar
/// tiempo), sin fecha al final, y por orden de archivo en empates.
/// Pensado para `sort_by` (estable): los IDs de venta crecen en el orden
/// en que se generaron los tickets.
pub fn comparar_cronologico(a: &TicketSegmento, b: &TicketSegmento) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (&a.fecha_hora, &b.fecha_hora) {
        (Some(x), Some(y)) => x.cmp(y).then(a.index.cmp(&b.index)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.index.cmp(&b.index),
    }
}
// ---------------------------------------------------------------------------
// Detección de marcadores (solo se evalúa sobre líneas NO útiles)
// ---------------------------------------------------------------------------

// NOTA: no hay regex de "palabras de apertura" por separado: la palabra
// sola ("CONSERVE SU TICKET") abría tickets fantasma. La apertura REAL la
// confirman `extraer_folio` / `tiene_fecha` en `es_apertura`.

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
    // La palabra sola ("CONSERVE SU TICKET") NO abre ticket: se exige
    // folio extraíble o fecha real en la misma línea.
    tiene_fecha(linea) || extraer_folio(linea).is_some()
}

/// Apertura "fuerte": la línea EMPIEZA con etiqueta de folio + valor
/// ("Fol 3341 - 06/03/2026"). Estas líneas a veces parecen de producto
/// (pocas columnas numéricas) y el filtro se las tragaría antes de que
/// `es_apertura` las vea; por eso se evalúan SIN exigir que sean separador.
/// Se exige etiqueta al INICIO para no partir tickets por productos que
/// mencionen la palabra (ej. "RIFA TICKET 2024" a media línea).
fn es_apertura_fuerte(linea: &str) -> bool {
    static RE_INICIO: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)^\s*(?:FOLIO|FOL|TICKET|SERIE|NOTA|RECIBO)\b").expect("regex inicio folio")
    });
    RE_INICIO.is_match(linea) && extraer_folio(linea).is_some()
}

fn es_cierre(linea: &str) -> bool {
    RE_TOTAL.is_match(linea)
        || RE_GRACIAS.is_match(linea)
        || RE_EFECTIVO_RECIBIDO.is_match(linea)
        || RE_CAMBIO.is_match(linea)
        || RE_PAGO_FOOTER.is_match(linea)
}

/// Extrae el folio/número de ticket de una línea de apertura.
///
/// Formatos reales soportados (verificados contra tickets mexicanos):
/// "FOLIO: 004582", "FOLIO|55190", "FOLIO=88231", "FOLIO:2288",
/// "Fol 3341", "FOL 00721", "TICKET #6650", "TICKET: A-004471",
/// "TICKET NO. 1927", "NO. TICKET: 0002", "SERIE A-123", "NOTA 45".
///
/// Reglas:
/// - Etiquetas: FOLIO, FOL, TICKET, SERIE, NOTA, RECIBO.
/// - Separadores: `: . # | = -` y espacios (los tickets usan `|` y `=`
///   cuando vienen de sistemas con campos delimitados).
/// - Muletillas entre etiqueta y valor ("TICKET NO. 1927", "NOTA DE
///   VENTA 12") se saltan; sin esto "TICKET NO. 1927" capturaba "NO".
/// - El valor debe contener al menos un dígito: evita que "No.
///   ARTICULOS: 8" (sin etiqueta válida igual) o "TICKET DE VENTA"
///   produzcan folios basura, y que dos tickets distintos colisionen.
/// - Una línea que solo trae fecha devuelve None (no hay folio).
fn extraer_folio(linea: &str) -> Option<String> {
    static RE_ETIQUETA: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(?:FOLIO|FOL|TICKET|SERIE|NOTA|RECIBO)\b").expect("regex etiqueta folio")
    });
    // Separadores + muletillas al inicio del resto ("NO.", "NUM", "#"...).
    // Sin `\b` global: `N°` termina en símbolo y el boundary fallaría.
    // Solo `DE` lo conserva para no comerse folios tipo "DELTA-9".
    static RE_RESTO: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"^(?:[\s:.\-#|=]|N°|NO\.?|NUM\.?(?:ERO)?|DE\b)+"#)
            .expect("regex resto folio")
    });
    // Valor del folio: letras/dígitos/guiones (para en el primer
    // espacio o `;`, así "FOLIO=88231;FECHA=..." da "88231").
    static RE_VALOR: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"[A-Za-z0-9][A-Za-z0-9\-]*").expect("regex valor folio"));

    let m = RE_ETIQUETA.find(linea)?;
    let resto = RE_RESTO.replace(linea[m.end()..].trim_start(), "");
    let valor = RE_VALOR.find(&resto).map(|v| v.as_str().to_string())?;
    // Sin dígito no es folio ("TICKET DE VENTA", "NOTA IMPORTANTE"...).
    valor.chars().any(|c| c.is_ascii_digit()).then_some(valor)
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
    fn extraer_folio_formatos_reales_de_tiendas() {
        // Separadores `|` y `=` de sistemas con campos delimitados.
        assert_eq!(extraer_folio("FOLIO|55190").as_deref(), Some("55190"));
        assert_eq!(
            extraer_folio("TIENDA=ABARROTES_LOPEZ;FOLIO=88231;FECHA=2026-03-09")
                .as_deref(),
            Some("88231")
        );
        // Abreviatura "FOL" con y sin dos puntos.
        assert_eq!(
            extraer_folio("Fol 3341 - 06/03/2026 08:55").as_deref(),
            Some("3341")
        );
        assert_eq!(
            extraer_folio("FOL 00721  07/03/26 12:30").as_deref(),
            Some("00721")
        );
        assert_eq!(extraer_folio("FOLIO:2288 04/03/26 21:03").as_deref(), Some("2288"));
        // La muletilla "NO." no se captura como folio (antes daba "NO").
        assert_eq!(extraer_folio("TICKET NO. 1927").as_deref(), Some("1927"));
        assert_eq!(extraer_folio("Ticket #6650").as_deref(), Some("6650"));
        assert_eq!(
            extraer_folio("TICKET: A-004471        10/03/2026").as_deref(),
            Some("A-004471")
        );
        // Sin dígito no es folio: nada de folios basura ni colisiones.
        assert_eq!(extraer_folio("CONSERVE SU TICKET"), None);
        assert_eq!(extraer_folio("TICKET DE VENTA"), None);
        assert_eq!(extraer_folio("No. ARTICULOS: 8"), None);
        assert_eq!(extraer_folio("FOLIO:"), None);
    }

    #[test]
    fn conserve_su_ticket_no_abre_segmento_fantasma() {
        // "CONSERVE SU TICKET" trae la palabra ticket pero sin folio ni
        // fecha: es pie, no un segundo ticket.
        let texto = "TICKET: A-004471        10/03/2026  20:15:09\n\
                     2 COCA $25.00 $50.00\n\
                     TOTAL: 50.00\n\
                     ================================================\n\
                     CONSERVE SU TICKET\n\
                     ================================================\n";
        let segs = segmentar(texto);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].folio.as_deref(), Some("A-004471"));
        assert!(segs[0].lineas.iter().any(|l| l.contains("CONSERVE")));
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

    #[test]
    fn clave_es_folio_si_hay_y_hash_estable_si_no() {
        let con_folio = &segmentar("FOLIO: 9\n2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
        assert_eq!(con_folio.clave(), "9");

        let a = &segmentar("2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
        let b = &segmentar("2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
        assert!(a.clave().starts_with("SIN-FOLIO-"));
        // Estable entre corridas: mismo contenido → misma clave.
        assert_eq!(a.clave(), b.clave());

        // Contenido distinto → clave distinta (no se come ventas ajenas).
        let c = &segmentar("3 COCA $25.00 $75.00\nTOTAL $75.00\n")[0];
        assert_ne!(a.clave(), c.clave());
    }

    #[test]
    fn clave_auto_desde_fecha_y_hora_es_legible_y_ordenable() {
        let a = &segmentar("12/05/2026 14:30\n2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
        let b = &segmentar("12/05/2026 14:30\n2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
        assert!(
            a.clave().starts_with("AUTO-20260512-1430-"),
            "clave: {}",
            a.clave()
        );
        // Determinista: re-importar da el mismo folio automático.
        assert_eq!(a.clave(), b.clave());

        // Distinto contenido el mismo minuto → distinto sufijo.
        let c = &segmentar("12/05/2026 14:30\n3 COCA $25.00 $75.00\nTOTAL $75.00\n")[0];
        assert_ne!(a.clave(), c.clave());

        // Orden lexicográfico = orden cronológico (ancho fijo).
        let d = &segmentar("13/05/2026 09:00\n2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
        assert!(a.clave() < d.clave());
    }

    #[test]
    fn clave_prioriza_folio_impreso_sobre_fecha() {
        let s = &segmentar("FOLIO: 7\n12/05/2026\n2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
        assert_eq!(s.clave(), "7");
    }

    #[test]
    fn comparar_cronologico_ordena_y_manda_sin_fecha_al_final() {
        let mut segs = segmentar(
            "14/05/2026\n2 COCA $25.00 $50.00\nTOTAL $50.00\n\
             12/05/2026\n1 PAN $10.00 $10.00\nTOTAL $10.00\n",
        );
        segs.sort_by(comparar_cronologico);
        assert_eq!(segs[0].fecha_hora.as_deref(), Some("2026-05-12 00:00:00"));
        assert_eq!(segs[1].fecha_hora.as_deref(), Some("2026-05-14 00:00:00"));

        let mut mixtos = segmentar("2 COCA $25.00 $50.00\nTOTAL $50.00\n");
        mixtos.extend(segmentar("12/05/2026\n1 PAN $10.00 $10.00\nTOTAL $10.00\n"));
        mixtos.sort_by(comparar_cronologico);
        assert!(mixtos[0].fecha_hora.is_some());
        assert!(mixtos[1].fecha_hora.is_none());
    }

    #[test]
    fn tickets_reales_fol_y_fecha_abreviada() {
        // ticket_06: "Fol" sin dos puntos + productos de un solo importe.
        let t06 = "*** MISCELANEA \"LAS DELICIAS\" ***\n\
                   Fol 3341 - 06/03/2026 08:55\n\
                   2x Coca 600         $36.00\n\
                   1x Sabritas Original $18.50\n\
                   TOTAL A PAGAR $129.50\n\
                   Gracias - vuelva pronto\n";
        let segs = segmentar(t06);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].folio.as_deref(), Some("3341"));
        assert_eq!(segs[0].fecha_hora.as_deref(), Some("2026-03-06 08:55:00"));

        // ticket_03: campos con `|` e ISO con hora pegada.
        let t03 = "TIENDA|LA GUADALUPANA|SUC01\n\
                   FOLIO|55190\n\
                   FECHA|2026-03-04T20:11:03\n\
                   TOTAL|220.70\n";
        let segs = segmentar(t03);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].folio.as_deref(), Some("55190"));
        assert_eq!(segs[0].fecha_hora.as_deref(), Some("2026-03-04 20:11:00"));
    }
}
