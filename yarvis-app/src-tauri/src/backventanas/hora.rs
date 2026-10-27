// ============================================================
// Reloj operativo basado exclusivamente en la hora local de Windows.
// Nunca consulta internet ni bloquea ventas o cortes.
// ============================================================
use crate::backventanas::auth::{AuthState, Role};
use chrono::{DateTime, Local};
use sqlx::SqlitePool;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// Saltos mayores a dos minutos entre lecturas monotónicas y del reloj local.
const UMBRAL_SALTO_RELOJ_SEG: i64 = 120;

static ULTIMA_LECTURA_RELOJ: OnceLock<Mutex<Option<(Instant, i64)>>> = OnceLock::new();

#[derive(serde::Serialize, Clone)]
pub struct EstadoHora {
    /// "YYYY-MM-DD HH:MM:SS" del sistema local.
    pub hora_sistema: String,
    pub zona: String,
    /// "ok" | "reloj_atrasado" | "cambio_brusco"
    pub estado: String,
    /// Explicación en facilito para el dueño/empleado.
    pub mensaje: String,
    /// Último evento conocido en DB (ventas/cortes/asistencias).
    pub ultimo_evento: Option<String>,
    /// ID del aviso persistente asociado al empleado conectado, si existe.
    pub evento_reloj_id: Option<i64>,
    /// True únicamente cuando esta consulta acaba de crear un aviso.
    pub evento_reloj_nuevo: bool,
}

/// Detecta un salto del reloj del sistema comparando tiempo de pared con
/// tiempo monotónico. Tolera pequeñas correcciones y pausas de la app.
fn detectar_salto_reloj(ahora: &DateTime<Local>) -> Option<i64> {
    let lecturas = ULTIMA_LECTURA_RELOJ.get_or_init(|| Mutex::new(None));
    let monotono = Instant::now();
    let pared = ahora.timestamp();
    let mut guard = lecturas.lock().ok()?;
    let salto = guard.as_ref().and_then(|(mono_anterior, pared_anterior)| {
        let transcurrido = monotono.duration_since(*mono_anterior).as_secs() as i64;
        let cambio_pared = pared - *pared_anterior;
        let diferencia = cambio_pared - transcurrido;
        (diferencia.abs() >= UMBRAL_SALTO_RELOJ_SEG).then_some(diferencia)
    });
    *guard = Some((monotono, pared));
    salto
}

async fn registrar_o_actualizar_evento(
    pool: &SqlitePool,
    empleado_id: i64,
    tipo: &str,
    hora_sistema: &str,
    mensaje: &str,
) -> Result<(i64, bool), String> {
    let abierto = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM eventos_reloj WHERE empleado_id = ? AND resuelto_en IS NULL ORDER BY id DESC LIMIT 1",
    )
    .bind(empleado_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(id) = abierto {
        sqlx::query(
            "UPDATE eventos_reloj SET tipo = ?, hora_sistema = ?, hora_referencia = NULL, desfase_seg = NULL, mensaje = ?, ultima_observacion = datetime('now','localtime') WHERE id = ?",
        )
        .bind(tipo)
        .bind(hora_sistema)
        .bind(mensaje)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        return Ok((id, false));
    }

    let result = sqlx::query(
        "INSERT INTO eventos_reloj (empleado_id, tipo, hora_sistema, mensaje, hora_referencia, desfase_seg) VALUES (?, ?, ?, ?, NULL, NULL)",
    )
    .bind(empleado_id)
    .bind(tipo)
    .bind(hora_sistema)
    .bind(mensaje)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok((result.last_insert_rowid(), true))
}

/// Compara únicamente fechas locales con fechas locales guardadas.
pub fn evaluar_estado(ultimo_evento: Option<&str>, hora_sistema: &str) -> (String, String) {
    if let Some(ult) = ultimo_evento {
        if hora_sistema < ult {
            return (
                "reloj_atrasado".to_string(),
                format!(
                    "La hora local está antes del último movimiento guardado ({}). \
                     Revisa la fecha, hora y zona horaria de Windows. Las ventas y cortes continúan disponibles.",
                    ult
                ),
            );
        }
    }
    (
        "ok".to_string(),
        "Se está usando la hora local de Windows para ventas, turnos y cortes.".to_string(),
    )
}

/// Último evento conocido: MAX de ventas.fecha, cortes_caja
/// (cierre o apertura) y asistencias (último login). Best-effort:
/// si una tabla no existe aún (primer arranque), se ignora.
pub async fn ultimo_evento_impl(pool: &SqlitePool) -> Option<String> {
    let mut candidatos: Vec<String> = Vec::new();
    for q in [
        "SELECT MAX(fecha) FROM ventas WHERE cajero_id IS NOT NULL",
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

/// Comando Tauri: verifica coherencia con el historial local, sin red.
#[tauri::command]
pub async fn verificar_hora_sistema(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<EstadoHora, String> {
    let sesion = auth.require_authenticated()?;
    let ahora_local = chrono::Local::now();
    let hora_sistema = ahora_local.format("%Y-%m-%d %H:%M:%S").to_string();
    let zona = ahora_local.format("%:z").to_string();
    let salto = detectar_salto_reloj(&ahora_local);

    let ultimo = ultimo_evento_impl(&state).await;

    let (mut estado, mut mensaje) = evaluar_estado(ultimo.as_deref(), &hora_sistema);
    if let Some(segundos) = salto {
        let direccion = if segundos < 0 { "retrocedió" } else { "avanzó" };
        estado = "cambio_brusco".to_string();
        mensaje = format!(
                "La hora local de Windows {direccion} bruscamente (aprox. {} min). Revisa la fecha, hora y zona horaria. Las ventas y los cortes siguen disponibles.",
            segundos.abs() / 60
        );
    }

    let (evento_reloj_id, evento_reloj_nuevo) = if sesion.role == Role::Employee {
        if matches!(estado.as_str(), "reloj_atrasado" | "cambio_brusco") {
            let (id, nuevo) = registrar_o_actualizar_evento(
                &state,
                sesion.user_id,
                &estado,
                &hora_sistema,
                &mensaje,
            )
            .await?;
            (Some(id), nuevo)
        } else {
            if estado == "ok" {
                sqlx::query("UPDATE eventos_reloj SET resuelto_en = datetime('now','localtime') WHERE empleado_id = ? AND resuelto_en IS NULL")
                    .bind(sesion.user_id)
                    .execute(&*state)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            (None, false)
        }
    } else {
        (None, false)
    };

    Ok(EstadoHora {
        hora_sistema,
        zona,
        estado,
        mensaje,
        ultimo_evento: ultimo,
        evento_reloj_id,
        evento_reloj_nuevo,
    })
}

#[cfg(test)]
mod tests {
    use super::evaluar_estado;

    #[test]
    fn compara_hora_local_con_ultimo_movimiento_local() {
        let (estado, _) = evaluar_estado(Some("2026-10-26 23:18:36"), "2026-10-26 23:19:42");
        assert_eq!(estado, "ok");
    }

    #[test]
    fn advierte_si_la_hora_local_va_atras_del_historial_local() {
        let (estado, mensaje) = evaluar_estado(Some("2026-10-26 23:18:36"), "2026-10-26 22:00:00");
        assert_eq!(estado, "reloj_atrasado");
        assert!(mensaje.contains("hora local"));
    }
}
