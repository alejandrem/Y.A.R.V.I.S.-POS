// ============================================================
// hora.rs — Verificación del reloj del sistema (turnos y cortes).
//
// Problema: todo el POS confía en `chrono::Local::now()` y en
// `datetime('now','localtime')` de SQLite. Si el reloj de Windows
// está mal, los logins, horas extra, ventas y cortes Z salen mal.
//
// Este módulo NO corrige la hora (un POS offline-first no debe
// tocar el reloj ajeno). Solo la VERIFICA y avisa en facilito:
//   · Detecta retroceso: hora actual < último evento en DB
//     (ventas / cortes / asistencias). Indica reloj atrasado o
//     restauración con fecha vieja.
//   · Intenta hora de red (best-effort, timeout 4s) vía el header
//     `Date` de https://www.google.com/generate_204. Si no hay
//     internet, reporta `sin_red` y NO bloquea la caja.
//   · Si hay red y el desfase supera 5 min, reporta `desfasado`.
//
// Sin red, sin bloqueo, sin dependencias nuevas (reqwest ya existe).
// ============================================================
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

/// 5 minutos de tolerancia: por debajo es deriva normal de reloj
/// barato; por encima ya mueve turnos y cortes.
pub const UMBRAL_DESFASE_SEG: i64 = 300;

#[derive(serde::Serialize, Clone)]
pub struct EstadoHora {
    /// "YYYY-MM-DD HH:MM:SS" del sistema local.
    pub hora_sistema: String,
    pub zona: String,
    pub red_disponible: bool,
    pub hora_red: Option<String>,
    /// red − sistema, en segundos (negativo = sistema adelantado).
    pub desfase_seg: Option<i64>,
    /// "ok" | "desfasado" | "reloj_atrasado" | "sin_red"
    pub estado: String,
    /// Explicación en facilito para el dueño/empleado.
    pub mensaje: String,
    /// Último evento conocido en DB (ventas/cortes/asistencias).
    pub ultimo_evento: Option<String>,
}

/// Lógica pura, testeable sin DB ni red.
pub fn evaluar_estado(
    desfase_seg: Option<i64>,
    red_ok: bool,
    ultimo_evento: Option<&str>,
    hora_sistema: &str,
) -> (String, String) {
    // 1. Retroceso: el sistema está ANTES del último evento guardado.
    // Compara como texto porque todo es "YYYY-MM-DD HH:MM:SS" (orden
    // lexicográfico = cronológico). 5 min de colchón anti-falsos.
    if let Some(ult) = ultimo_evento {
        if hora_sistema < ult {
            return (
                "reloj_atrasado".to_string(),
                format!(
                    "Tu reloj está atrás del último movimiento guardado ({}). \
                     Revisa la hora de Windows: los turnos y cortes saldrán mal \
                     hasta que la corrijas. Activa hora automática.",
                    ult
                ),
            );
        }
    }
    // 2. Con red: ¿desfase grande?
    if red_ok {
        if let Some(d) = desfase_seg {
            if d.abs() > UMBRAL_DESFASE_SEG {
                let mins = d.abs() / 60;
                let lado = if d > 0 {
                    "atrasado"
                } else {
                    "adelantado"
                };
                return (
                    "desfasado".to_string(),
                    format!(
                        "Tu reloj va {} unos {} min contra la hora de internet. \
                         Activa hora automática en Windows para que los turnos \
                         cuadren.",
                        lado, mins
                    ),
                );
            }
            return (
                "ok".to_string(),
                "Hora correcta: coincide con internet.".to_string(),
            );
        }
    }
    // 3. Sin red y sin retroceso: no se puede comprobar.
    (
        "sin_red".to_string(),
        "Sin internet no pude comparar tu hora. Si ves turnos raros, \
         revisa que Windows tenga hora automática activada."
            .to_string(),
    )
}

/// Último evento conocido: MAX de ventas.fecha, cortes_caja
/// (cierre o apertura) y asistencias (último login). Best-effort:
/// si una tabla no existe aún (primer arranque), se ignora.
pub async fn ultimo_evento_impl(pool: &SqlitePool) -> Option<String> {
    let mut candidatos: Vec<String> = Vec::new();
    for q in [
        "SELECT MAX(fecha) FROM ventas",
        "SELECT MAX(COALESCE(fecha_cierre, fecha_apertura)) FROM cortes_caja",
        "SELECT MAX(COALESCE(ultimo_login, primer_login)) FROM asistencias",
    ] {
        if let Ok(row) = sqlx::query_scalar::<_, Option<String>>(q)
            .fetch_one(pool)
            .await
        {
            if let Some(v) = row {
                // Filtra defaults tipo '2000-01-01' y vacíos.
                if v.len() >= 16 && !v.starts_with("2000-") {
                    candidatos.push(v);
                }
            }
        }
    }
    candidatos.into_iter().max()
}

/// Hora de red vía header `Date` (RFC2822, siempre GMT). Endpoint
/// ligero 204 sin cuerpo. Timeout corto para no frenar la caja.
async fn hora_red_impl() -> Option<DateTime<Utc>> {
    let cliente = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .no_proxy()
        .build()
        .ok()?;
    let resp = cliente
        .get("https://www.google.com/generate_204")
        .send()
        .await
        .ok()?;
    let fecha_txt = resp.headers().get("date")?.to_str().ok()?;
    DateTime::parse_from_rfc2822(fecha_txt)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

/// Comando Tauri: verifica el reloj. Nunca falla por falta de red
/// (reporta `sin_red`); solo falla si ni la hora local se puede leer.
#[tauri::command]
pub async fn verificar_hora_sistema(
    state: tauri::State<'_, SqlitePool>,
) -> Result<EstadoHora, String> {
    let ahora_local = chrono::Local::now();
    let hora_sistema = ahora_local.format("%Y-%m-%d %H:%M:%S").to_string();
    let zona = ahora_local.format("%:z").to_string();

    let ultimo = ultimo_evento_impl(&state).await;

    let (red_ok, hora_red_txt, desfase) = match hora_red_impl().await {
        Some(red) => {
            let d = red.timestamp() - ahora_local.with_timezone(&Utc).timestamp();
            (
                true,
                Some(red.format("%Y-%m-%d %H:%M:%S").to_string()),
                Some(d),
            )
        }
        None => (false, None, None),
    };

    let (estado, mensaje) = evaluar_estado(desfase, red_ok, ultimo.as_deref(), &hora_sistema);

    Ok(EstadoHora {
        hora_sistema,
        zona,
        red_disponible: red_ok,
        hora_red: hora_red_txt,
        desfase_seg: desfase,
        estado,
        mensaje,
        ultimo_evento: ultimo,
    })
}

#[cfg(test)]
mod tests {
    use super::{evaluar_estado, UMBRAL_DESFASE_SEG};

    #[test]
    fn desfase_pequeno_es_ok() {
        let (e, _) = evaluar_estado(Some(60), true, None, "2026-10-12 09:00:00");
        assert_eq!(e, "ok");
        let (e2, _) = evaluar_estado(Some(-UMBRAL_DESFASE_SEG), true, None, "2026-10-12 09:00:00");
        assert_eq!(e2, "ok");
    }

    #[test]
    fn desfase_grande_avisa_lado() {
        let (e, m) = evaluar_estado(Some(600), true, None, "2026-10-12 09:00:00");
        assert_eq!(e, "desfasado");
        assert!(m.contains("atrasado"));
        let (e2, m2) = evaluar_estado(Some(-900), true, None, "2026-10-12 09:00:00");
        assert_eq!(e2, "desfasado");
        assert!(m2.contains("adelantado"));
    }

    #[test]
    fn reloj_atrasado_gana_a_red() {
        // Sistema antes del último ticket: aunque la red diga ok, manda el retroceso.
        let (e, _) = evaluar_estado(
            Some(10),
            true,
            Some("2026-10-12 10:00:00"),
            "2026-10-12 08:00:00",
        );
        assert_eq!(e, "reloj_atrasado");
    }

    #[test]
    fn sin_red_no_bloquea() {
        let (e, _) = evaluar_estado(None, false, None, "2026-10-12 09:00:00");
        assert_eq!(e, "sin_red");
    }
}
