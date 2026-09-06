// ============================================================
// marcadores — Detección de aperturas, cierres y folios.
//
// Una APERTURA (folio extraíble o fecha real) inicia un bloque; un CIERRE
// (TOTAL, GRACIAS, pago...) lo cierra. La palabra sola no basta:
// "CONSERVE SU TICKET" abría tickets fantasma, por eso la apertura REAL
// la confirman `extraer_folio` / `tiene_fecha`.
//
// La apertura "fuerte" (etiqueta de folio al INICIO + valor) vale aunque
// la línea parezca de producto: "Fol 3341 - 06/03/2026" tiene tan pocas
// columnas numéricas que el filtro se la tragaría antes de verla.
// ============================================================

use regex::Regex;
use std::sync::LazyLock;

use super::super::tiene_fecha;

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

pub(crate) fn es_apertura(linea: &str) -> bool {
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
pub(crate) fn es_apertura_fuerte(linea: &str) -> bool {
    static RE_INICIO: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)^\s*(?:FOLIO|FOL|TICKET|SERIE|NOTA|RECIBO)\b").expect("regex inicio folio")
    });
    RE_INICIO.is_match(linea) && extraer_folio(linea).is_some()
}

pub(crate) fn es_cierre(linea: &str) -> bool {
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
pub(crate) fn extraer_folio(linea: &str) -> Option<String> {
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
