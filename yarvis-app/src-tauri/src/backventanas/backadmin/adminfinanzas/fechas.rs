// ============================================================
// fechas.rs — Cálculo de la próxima fecha de pago de gastos
// recurrentes. Lógica compartida entre `gastos` (vencimientos) y
// `alertas` (semáforo), para que no existan dos copias que puedan
// divergir. Devuelve la próxima ocurrencia EN O DESPUÉS de `desde`
// (el día de vencimiento incluido → el día 0 cuenta como "vence hoy").
// ============================================================
use chrono::{Datelike, Duration, NaiveDate};

pub fn calcular_proxima_fecha(
    fecha_inicio: &str,
    frecuencia: &str,
    dia_pago: Option<i32>,
    intervalo_dias: Option<i32>,
    desde: NaiveDate,
) -> Option<NaiveDate> {
    let inicio = NaiveDate::parse_from_str(fecha_inicio, "%Y-%m-%d").ok()?;

    match frecuencia {
        "semanal" => {
            let mut fecha = inicio;
            while fecha < desde {
                fecha += Duration::days(7);
            }
            Some(fecha)
        }
        "quincenal" => {
            let dia = dia_pago.unwrap_or(1).clamp(1, 15) as u32;
            let mut fecha = NaiveDate::from_ymd_opt(desde.year(), desde.month(), dia)?;
            if fecha < desde {
                fecha = avanzar_mes(fecha, 1)?;
            }
            Some(fecha)
        }
        "mensual" => {
            let dia = dia_pago.unwrap_or(1).clamp(1, 28) as u32;
            let mut fecha = NaiveDate::from_ymd_opt(desde.year(), desde.month(), dia)?;
            if fecha < desde {
                fecha = avanzar_mes(fecha, 1)?;
            }
            Some(fecha)
        }
        "trimestral" => {
            let dia = dia_pago.unwrap_or(1).clamp(1, 28) as u32;
            // Brazos trimestrales: abr, jul, oct y ene (siguiente año).
            let brazo = ((desde.month() - 1) / 3 + 1) * 3 + 1;
            let mut fecha = if brazo > 12 {
                NaiveDate::from_ymd_opt(desde.year() + 1, 1, dia)?
            } else {
                NaiveDate::from_ymd_opt(desde.year(), brazo, dia)?
            };
            if fecha < desde {
                fecha = avanzar_mes(fecha, 3)?;
            }
            Some(fecha)
        }
        "personalizado" => {
            let intervalo = intervalo_dias.unwrap_or(30) as i64;
            let mut fecha = inicio;
            while fecha < desde {
                fecha += Duration::days(intervalo);
            }
            Some(fecha)
        }
        _ => None,
    }
}

/// Avanza `fecha` `meses` hacia adelante manteniendo el día; si el día no
/// existe en el mes destino (p.ej. 31 en febrero) se usa el último día válido.
fn avanzar_mes(fecha: NaiveDate, meses: u32) -> Option<NaiveDate> {
    let mut anio = fecha.year();
    let mut mes = fecha.month() + meses;
    while mes > 12 {
        mes -= 12;
        anio += 1;
    }
    let dia = fecha.day().min(dias_del_mes(anio, mes));
    NaiveDate::from_ymd_opt(anio, mes, dia)
}

fn dias_del_mes(anio: i32, mes: u32) -> u32 {
    match mes {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (anio % 4 == 0 && anio % 100 != 0) || anio % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}
