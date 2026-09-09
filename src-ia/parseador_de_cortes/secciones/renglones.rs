// ============================================================
// renglones — Líneas de detalle: artículos, tickets y pagos.
//
//  ARTICULO  `JACK DNL HONEY 70 -   1 -   $535.00` (nombres
//            truncados terminan en " -": se recorta)
//  TICKET    `REM - 10834      94.00` (folios reales no llevan ':'
//            ni la palabra "total": se rechazan)
//  PAGO      `EFE Pago de clientes  $2,023.80` (ningún concepto real
//            contiene "total": así `Total en caja`, que vive dentro
//            de Egresos, no se cuela como egreso)
// ============================================================

use regex::Regex;
use std::sync::LazyLock;

use crate::parseador_de_cortes::valores::montos::limpiar_monto;

static RE_ARTICULO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<nombre>.+?)\s+-\s+(?P<cant>[\d.,]+)\s+-\s+\$?(?P<monto>[\d,]+\.\d{2}|\.\d{2})\s*$")
        .expect("regex articulo")
});
static RE_TICKET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<folio>[^*:\n]+?)\s{2,}(?P<monto>[\d,]+\.\d{2})\s*$").expect("regex ticket corte")
});
static RE_PAGO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<c>.+?)\s+\$?(?P<m>[\d,]+\.\d{2}|\.\d{2}|\d+)\s*$").expect("regex pago")
});

/// Artículo con cantidad y subtotal de línea.
pub fn linea_articulo(linea: &str) -> Option<(String, f64, i64)> {
    let c = RE_ARTICULO.captures(linea.trim())?;
    let mut nombre = c["nombre"].trim().to_string();
    while nombre.ends_with('-') {
        nombre.pop();
    }
    nombre = nombre.trim().to_string();
    if nombre.is_empty() {
        return None;
    }
    let cant: f64 = c["cant"].replace(',', "").parse().ok()?;
    let sub = limpiar_monto(&c["monto"])?;
    Some((nombre, cant, sub))
}

/// Ticket con folio y monto.
pub fn linea_ticket(linea: &str) -> Option<(String, i64)> {
    let t = linea.trim();
    if t.is_empty() || t.contains(':') || t.to_lowercase().contains("total") {
        return None;
    }
    let c = RE_TICKET.captures(t)?;
    let folio = c["folio"].trim().to_string();
    if folio.is_empty() {
        return None;
    }
    Some((folio, limpiar_monto(&c["monto"])?))
}

/// Línea de ingreso/egreso con concepto y monto.
pub fn linea_pago(linea: &str) -> Option<(String, i64)> {
    let t = linea.trim();
    if t.is_empty() || t.to_lowercase().contains("total") {
        return None;
    }
    let c = RE_PAGO.captures(t)?;
    let concepto = c["c"].trim().to_string();
    if concepto.is_empty() {
        return None;
    }
    Some((concepto, limpiar_monto(&c["m"])?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn articulos_truncados_y_cantidades() {
        assert_eq!(
            linea_articulo("JACK DNL HONEY 70 -       1 -   $535.00"),
            Some(("JACK DNL HONEY 70".into(), 1.0, 53500))
        );
        assert_eq!(
            linea_articulo("NEGRA MODELO 1L C -       3 -   $136.80"),
            Some(("NEGRA MODELO 1L C".into(), 3.0, 13680))
        );
        assert_eq!(linea_articulo("-----------------------------------"), None);
    }

    #[test]
    fn tickets_con_folio() {
        assert_eq!(linea_ticket("REM - 10834\t\t                  94.00"), Some(("REM - 10834".into(), 9400)));
        assert_eq!(linea_ticket("**Total ventas del dia :     702.00"), None);
        assert_eq!(linea_ticket("T-1                  300.00"), Some(("T-1".into(), 30000)));
    }

    #[test]
    fn pagos_con_codigo_y_concepto() {
        assert_eq!(
            linea_pago("EFE Pago de clientes  $2,023.80"),
            Some(("EFE Pago de clientes".into(), 202380))
        );
        assert_eq!(
            linea_pago(" 04 TARJETA BANCARIA    $305.00"),
            Some(("04 TARJETA BANCARIA".into(), 30500))
        );
        assert_eq!(linea_pago("  Total de Ingresos:    $702.00"), None);
    }

    #[test]
    fn total_en_caja_no_es_egreso() {
        assert_eq!(linea_pago("   Total en caja:     $1,000.00"), None);
    }
}
