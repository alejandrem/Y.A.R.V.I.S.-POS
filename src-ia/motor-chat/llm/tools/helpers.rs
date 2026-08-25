//! helpers — Utilidades puras compartidas por las tools (rangos de fechas,
//! argumentos JSON, escape LIKE y conversión monetaria).

use serde_json::Value;

// ─────────────────────────────────────────────────────────────────────────────
// Rangos de fechas (chrono) → cláusulas SQL
// ─────────────────────────────────────────────────────────────────────────────

pub(crate) struct Rango {
    pub desde: String,
    pub hasta: String,
    pub etiqueta: String,
}

pub(crate) fn rango_de(valor: &str) -> Rango {
    use chrono::{Datelike, Duration, Local};
    let hoy = Local::now().date_naive();
    let fmt = |d: chrono::NaiveDate| d.format("%Y-%m-%d").to_string();

    match valor {
        "today" => Rango { desde: fmt(hoy), hasta: fmt(hoy), etiqueta: valor.into() },
        "yesterday" => {
            let ayer = hoy - Duration::days(1);
            Rango { desde: fmt(ayer), hasta: fmt(ayer), etiqueta: valor.into() }
        }
        "this_week" => {
            let lunes = hoy - Duration::days(hoy.weekday().num_days_from_monday() as i64);
            Rango { desde: fmt(lunes), hasta: fmt(hoy), etiqueta: valor.into() }
        }
        "last_week" => {
            let lunes_esta = hoy - Duration::days(hoy.weekday().num_days_from_monday() as i64);
            let lunes_pasada = lunes_esta - Duration::days(7);
            Rango { desde: fmt(lunes_pasada), hasta: fmt(lunes_esta - Duration::days(1)), etiqueta: valor.into() }
        }
        "this_month" => {
            let primero = chrono::NaiveDate::from_ymd_opt(hoy.year(), hoy.month(), 1).unwrap_or(hoy);
            Rango { desde: fmt(primero), hasta: fmt(hoy), etiqueta: valor.into() }
        }
        "last_month" => {
            let primero_mes = chrono::NaiveDate::from_ymd_opt(hoy.year(), hoy.month(), 1).unwrap_or(hoy);
            let fin_mes_pasado = primero_mes - Duration::days(1);
            let inicio_mes_pasado =
                chrono::NaiveDate::from_ymd_opt(fin_mes_pasado.year(), fin_mes_pasado.month(), 1)
                    .unwrap_or(fin_mes_pasado);
            Rango { desde: fmt(inicio_mes_pasado), hasta: fmt(fin_mes_pasado), etiqueta: valor.into() }
        }
        otro => Rango { desde: fmt(hoy), hasta: fmt(hoy), etiqueta: otro.into() },
    }
}

pub(crate) fn str_arg<'a>(args: &'a Value, clave: &str, default: &'a str) -> String {
    args.get(clave).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

/// Escapa los comodines de LIKE (`%`, `_` y el propio `\`) para que el
/// texto de búsqueda se trate literalmente. Usar SIEMPRE con
/// `ESCAPE '\'` en la consulta. Sin esto, un producto llamado "50%"
/// alteraba la semántica del patrón.
pub(crate) fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

pub(crate) const MONEDA: &str = "MXN";

/// La DB almacena el dinero en CENTAVOS enteros (migración f64 → i64).
/// Estas tools leen centavos y devuelven PESOS al LLM (÷100 antes de
/// serializar), porque el modelo consume/produce montos en pesos.
pub(crate) fn centavos_a_pesos(centavos: f64) -> f64 {
    round2(centavos / 100.0)
}

/// Redondeo de limpieza a 2 decimales. Ya NO convierte unidades: la
/// conversión centavos→pesos vive en [`centavos_a_pesos`].
pub(crate) fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
