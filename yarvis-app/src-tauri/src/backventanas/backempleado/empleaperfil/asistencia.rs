// ============================================================
// asistencia.rs — Registro de asistencia y turno del empleado.
//
// Reglas de negocio:
//   · El PRIMER login del día queda como hora de entrada real;
//     los logins siguientes del mismo día solo refrescan ultimo_login.
//   · Logins fuera del horario NO se bloquean (pueden ser horas extra).
//   · La comparación contra el horario se hace con los bloques de
//     empleado_horarios del día de hoy (convención L=0..D=6).
// ============================================================
use crate::backventanas::auth::AuthState;
use crate::dinero::a_pesos;
use chrono::Datelike;
use sqlx::Row;
use sqlx::SqlitePool;

/// Registra (o refresca) la asistencia del login de un empleado.
pub async fn registrar_asistencia(pool: &SqlitePool, empleado_id: i64) -> Result<(), String> {
    // INSERT que solo crea renglón si es el primer login del día; si ya
    // existe, actualiza ultimo_login y deja primer_login intacto.
    sqlx::query(
        r#"INSERT INTO asistencias (empleado_id, fecha, primer_login, ultimo_login)
           VALUES (?, date('now','localtime'), datetime('now','localtime'), datetime('now','localtime'))
           ON CONFLICT(empleado_id, fecha)
           DO UPDATE SET ultimo_login = datetime('now','localtime')"#,
    )
    .bind(empleado_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(serde::Serialize)]
pub struct BloqueHoy {
    pub hora_inicio: String,
    pub hora_fin: String,
}

#[derive(serde::Serialize)]
pub struct MiTurno {
    /// ¿Hoy es día laborable según sus bloques?
    pub dia_laborable: bool,
    /// Bloques de horario que aplican HOY (normalmente 1; jornadas partidas → 2+).
    pub bloques_hoy: Vec<BloqueHoy>,
    /// Hora del PRIMER login de hoy en formato HH:MM (None = aún no ha entrado).
    pub primer_login: Option<String>,
    pub horas_por_dia: f64,
    pub dias_semana: i32,
    pub ultimo_login: Option<String>,
    /// Fecha_cierre del último corte Z propio ("YYYY-MM-DD HH:MM:SS").
    /// La barra de extra congela ahí: el turno termina con el Z.
    pub ultimo_corte_z: Option<String>,
}

/// Índice de chip del día actual: Lunes=0 .. Domingo=6 (igual que el frontend).
fn chip_idx_hoy() -> i32 {
    // chrono: Mon=0..Sun=6 vía num_days_from_monday
    chrono::Local::now().weekday().num_days_from_monday() as i32
}

#[tauri::command]
pub async fn get_mi_turno(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<MiTurno, String> {
    let session = auth.require_operator()?;
    mi_turno_impl(&state, session.user_id).await
}

/// Versión admin: consulta el turno de CUALQUIER empleado por id
/// (para el detalle de personal en el panel administrativo).
#[tauri::command]
pub async fn get_asistencia_empleado(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    empleado_id: i64,
) -> Result<MiTurno, String> {
    auth.require_admin()?;
    mi_turno_impl(&state, empleado_id).await
}

/// Núcleo compartido, testeable sin runtime de Tauri.
pub async fn mi_turno_impl(pool: &SqlitePool, empleado_id: i64) -> Result<MiTurno, String> {

    // Bloques del empleado (todos); filtramos los que incluyen hoy.
    let filas = sqlx::query(
        "SELECT dias, hora_inicio, hora_fin FROM empleado_horarios WHERE empleado_id = ? ORDER BY id ASC",
    )
    .bind(empleado_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let hoy = chip_idx_hoy();
    let bloques_hoy: Vec<BloqueHoy> = filas
        .iter()
        .filter(|f| {
            f.get::<String, _>("dias")
                .split(',')
                .filter_map(|d| d.trim().parse::<i32>().ok())
                .any(|d| d == hoy)
        })
        .map(|f| BloqueHoy {
            hora_inicio: f.get("hora_inicio"),
            hora_fin: f.get("hora_fin"),
        })
        .collect();

    // Asistencia de hoy: substr(primer_login,12,5) → "HH:MM" desde
    // "YYYY-MM-DD HH:MM:SS".
    let asistencia = sqlx::query_as::<_, (Option<String>, Option<String>)>(
        "SELECT substr(primer_login, 12, 5), substr(ultimo_login, 12, 5)
         FROM asistencias WHERE empleado_id = ? AND fecha = date('now','localtime')",
    )
    .bind(empleado_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    // salario_diario vive en centavos (INTEGER); se decodifica y convierte.
    let perfil = sqlx::query_as::<_, (i64, i32)>(
        "SELECT salario_diario, dias_semana FROM usuarios WHERE id = ?",
    )
    .bind(empleado_id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let _salario_diario: f64 = a_pesos(perfil.0);
    // Horas/día derivadas de los bloques de HOY (más preciso que el promedio).
    let horas_dia = bloques_hoy
        .iter()
        .map(|b| {
            let mins = |t: &str| -> i64 {
                let p: Vec<i64> = t.split(':').map(|x| x.parse().unwrap_or(0)).collect();
                p.first().unwrap_or(&0) * 60 + p.get(1).unwrap_or(&0)
            };
            let ini = mins(&b.hora_inicio);
            let mut fin = mins(&b.hora_fin);
            if fin <= ini {
                fin += 24 * 60; // turno nocturno cruza medianoche
            }
            (fin - ini) as f64 / 60.0
        })
        .sum();

    // Último corte Z propio: ancla de cierre del turno (para congelar
    // la barra de extra; ver backcortes::comun::ancla_turno).
    let ultimo_z: Option<String> = sqlx::query_as::<_, (Option<String>,)>(
        "SELECT fecha_cierre FROM cortes_caja
          WHERE usuario_id = ? AND tipo_corte = 'Z'
            AND estado = 'cerrado' AND fecha_cierre IS NOT NULL
          ORDER BY fecha_cierre DESC LIMIT 1",
    )
        .bind(empleado_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .and_then(|r| r.0);

    Ok(MiTurno {
        dia_laborable: !bloques_hoy.is_empty(),
        bloques_hoy,
        primer_login: asistencia.as_ref().and_then(|a| a.0.clone()),
        horas_por_dia: horas_dia,
        dias_semana: perfil.1,
        ultimo_login: asistencia.and_then(|a| a.1),
        ultimo_corte_z: ultimo_z,
    })
}

// ═══════════════════════════════════════════════════════════════════════
// HISTORIAL DE HORAS EXTRA — Oro puro almacenado y compartido.
// Por cada día con asistencia se recalculan las extras comparando la
// entrada/salida reales contra los bloques de ese día de la semana.
// Solo se devuelven días con extra > 0 (si no hubo, ni aparecen).
// ═══════════════════════════════════════════════════════════════════════

#[derive(serde::Serialize, Clone)]
pub struct DiaExtra {
    pub fecha: String,
    pub dia_label: String,
    /// HH:MM del primer login del día
    pub primer_login: String,
    /// HH:MM del último registro de actividad
    pub ultimo_login: String,
    pub entrada_oficial: String,
    pub salida_oficial: String,
    pub extra_pre_min: i64,
    pub extra_post_min: i64,
    /// Minutos totales trabajados ese día (entrada→salida reales)
    pub trabajo_min: i64,
    /// Turno abierto: es hoy y aún no hay corte Z (la "salida" real la
    /// define el Z; ultimo_login es solo última actividad).
    pub en_curso: bool,
}

const UMBRAL_TEMPRANO_MIN: i64 = 15;

/// Regla de extra (#23, regla B): el extra post existe si la entrada fue
/// antes o durante el turno (`entrada <= bloque_fin`) y la salida lo
/// rebasó; y si entró DE NOCHE tras el fin, toda su presencia cuenta
/// (`salida − entrada`: 21:41→23:00 = 79 min). Simétrico en pre: solo
/// cuenta si siguió hasta la entrada oficial (`salida >= ini`), y
/// DESCUENTA los 15 min de cortesía (8:44 con entrada 9:00 = 1 min
/// extra, no 16). Todo en minutos (nocturnos ya extendidos +24h).
pub fn calcular_extras(
    entrada_min: i64,
    salida_min: i64,
    bloque_ini: i64,
    bloque_fin: i64,
) -> (i64, i64) {
    let presencia_previa = (salida_min.min(bloque_ini) - entrada_min).max(0);
    let extra_pre = if salida_min >= bloque_ini {
        (presencia_previa - UMBRAL_TEMPRANO_MIN).max(0)
    } else {
        0
    };
    let extra_post = if entrada_min <= bloque_fin {
        (salida_min - bloque_fin).max(0)
    } else {
        // Regla B: entró tras el fin → su presencia nocturna sí cuenta.
        (salida_min - entrada_min).max(0)
    };
    (extra_pre, extra_post)
}

fn mins_hhmm(t: &str) -> i64 {
    let p: Vec<i64> = t.split(':').map(|x| x.parse().unwrap_or(0)).collect();
    *p.first().unwrap_or(&0) * 60 + *p.get(1).unwrap_or(&0)
}

fn dia_label_es(wd: chrono::Weekday) -> &'static str {
    match wd {
        chrono::Weekday::Mon => "Lunes",
        chrono::Weekday::Tue => "Martes",
        chrono::Weekday::Wed => "Miércoles",
        chrono::Weekday::Thu => "Jueves",
        chrono::Weekday::Fri => "Viernes",
        chrono::Weekday::Sat => "Sábado",
        chrono::Weekday::Sun => "Domingo",
    }
}

/// Núcleo del historial, testeable sin runtime de Tauri.
/// Días recientes primero; máximo 90 días de ventana.
pub async fn historial_horas_extra_impl(
    pool: &SqlitePool,
    empleado_id: i64,
) -> Result<Vec<DiaExtra>, String> {
    let asistencias = sqlx::query(
        r#"SELECT fecha,
                  substr(primer_login, 12, 5) AS pl,
                  COALESCE(substr(ultimo_login, 12, 5), substr(primer_login, 12, 5)) AS ul
           FROM asistencias
           WHERE empleado_id = ?
           ORDER BY fecha DESC LIMIT 90"#,
    )
    .bind(empleado_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Bloques por índice de chip (L=0..D=6): puede haber varios por día.
    let bloques = sqlx::query(
        "SELECT dias, hora_inicio, hora_fin FROM empleado_horarios WHERE empleado_id = ?",
    )
    .bind(empleado_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    let mut por_dia: std::collections::HashMap<i32, Vec<(i64, i64)>> =
        std::collections::HashMap::new();
    for b in &bloques {
        let dias_txt: String = b.get("dias");
        let ini = mins_hhmm(&b.get::<String, _>("hora_inicio"));
        let mut fin = mins_hhmm(&b.get::<String, _>("hora_fin"));
        if fin <= ini {
            fin += 24 * 60;
        }
        for d in dias_txt.split(',').filter_map(|x| x.trim().parse::<i32>().ok()) {
            por_dia.entry(d).or_default().push((ini, fin));
        }
    }

    let mut resultado = Vec::new();
    for a in &asistencias {
        let fecha: String = a.get("fecha");
        let pl: String = a.get("pl");
        let ul: String = a.get("ul");
        let Ok(fecha_nueva) = chrono::NaiveDate::parse_from_str(&fecha, "%Y-%m-%d") else {
            continue;
        };
        let wd_idx = fecha_nueva.weekday().num_days_from_monday() as i32;
        let Some(bloques_del_dia) = por_dia.get(&wd_idx) else {
            continue; // día sin turno asignado: no aplica extra
        };

        let entrada = mins_hhmm(&pl);
        let salida = std::cmp::max(mins_hhmm(&ul), entrada);

        // Bloque principal = el más largo del día (para etiqueta oficial).
        let Some(&(bloque_ini, bloque_fin)) = bloques_del_dia
            .iter()
            .max_by_key(|(i, f)| f - i)
        else {
            continue;
        };

        // Extra PRE/POST con regla B (#23): nocturnos cuentan su presencia.
        let (extra_pre, extra_post) = calcular_extras(entrada, salida, bloque_ini, bloque_fin);

        if extra_pre + extra_post <= 0 {
            continue; // sin extras ese día: ni aparece
        }

        // ¿Sigue en curso? Es hoy y aún no hay corte Z: la salida real la
        // define el Z, ultimo_login es solo última actividad.
        let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();
        let en_curso = if fecha == hoy {
            let n: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM cortes_caja
                  WHERE usuario_id = ? AND tipo_corte = 'Z' AND estado = 'cerrado'
                    AND date(fecha_cierre) = ?",
            )
            .bind(empleado_id)
            .bind(&fecha)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            n == 0
        } else {
            false
        };

        resultado.push(DiaExtra {
            fecha: fecha.clone(),
            dia_label: dia_label_es(fecha_nueva.weekday()).to_string(),
            primer_login: pl,
            ultimo_login: ul,
            entrada_oficial: fmt_hhmm(bloque_ini),
            salida_oficial: fmt_hhmm(bloque_fin),
            extra_pre_min: extra_pre,
            extra_post_min: extra_post,
            trabajo_min: salida - entrada,
            en_curso,
        });
    }
    Ok(resultado)
}

fn fmt_hhmm(mins: i64) -> String {
    format!("{:02}:{:02}", (mins / 60) % 24, mins % 60)
}

#[tauri::command]
pub async fn get_mis_horas_extra(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<DiaExtra>, String> {
    let session = auth.require_operator()?;
    historial_horas_extra_impl(&state, session.user_id).await
}

/// Versión admin: historial de extras de CUALQUIER empleado por id.
#[tauri::command]
pub async fn get_horas_extra_empleado(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    empleado_id: i64,
) -> Result<Vec<DiaExtra>, String> {
    auth.require_admin()?;
    historial_horas_extra_impl(&state, empleado_id).await
}

#[cfg(test)]
mod tests {
    use super::calcular_extras;

    /// Caso del bug #23 con regla B: turno 09:00-17:00, login 20:33.
    /// Sin salida aún no hay nada que contar; con corte Z a las 23:00
    /// la presencia 20:33→23:00 sí cuenta (147 min).
    #[test]
    fn login_nocturno_cuenta_su_presencia() {
        assert_eq!(
            calcular_extras(20 * 60 + 33, 20 * 60 + 33, 9 * 60, 17 * 60),
            (0, 0)
        );
        assert_eq!(
            calcular_extras(20 * 60 + 33, 23 * 60, 9 * 60, 17 * 60),
            (0, 147)
        );
    }

    /// 21:41→23:00 con fin 17:00: 79 min nocturnos.
    #[test]
    fn noche_2141_al_cierre_2300() {
        let (pre, post) = calcular_extras(21 * 60 + 41, 23 * 60, 9 * 60, 17 * 60);
        assert_eq!((pre, post), (0, 79));
    }

    /// Extra genuino: entró 08:40 (20 min antes), salió 18:20.
    /// 20 − 15 de cortesía = 5 pre + 80 post.
    #[test]
    fn extra_genuino_pre_y_post() {
        let (pre, post) = calcular_extras(8 * 60 + 40, 18 * 60 + 20, 9 * 60, 17 * 60);
        assert_eq!((pre, post), (5, 80));
    }

    /// 08:44 con entrada 09:00: 16 − 15 = 1 min extra, no 16.
    #[test]
    fn solo_cuenta_lo_que_pasa_la_cortesia() {
        let (pre, post) = calcular_extras(8 * 60 + 44, 9 * 60 + 5, 9 * 60, 17 * 60);
        assert_eq!((pre, post), (1, 0));
    }

    /// Jornada extra 06:00 quedándose al turno: 180 − 15 = 165 pre.
    #[test]
    fn jornada_previa_quedandose_si_cuenta() {
        let (pre, post) = calcular_extras(6 * 60, 10 * 60, 9 * 60, 17 * 60);
        assert_eq!((pre, post), (165, 0));
    }

    /// Llegó 10 min antes (< umbral 15): no hay pre, y salió a tiempo: 0.
    #[test]
    fn llegada_puntual_no_cuenta_pre() {
        let (pre, post) = calcular_extras(8 * 60 + 50, 17 * 60, 9 * 60, 17 * 60);
        assert_eq!((pre, post), (0, 0));
    }

    /// Pasó temprano pero se fue antes de la entrada: pre en 0.
    #[test]
    fn visita_temprana_sin_quedarse_no_cuenta() {
        let (pre, post) = calcular_extras(6 * 60, 6 * 60 + 30, 9 * 60, 17 * 60);
        assert_eq!((pre, post), (0, 0));
    }

    /// Frontera: entró justo a la salida oficial y siguió 30 min → 30 post.
    #[test]
    fn entrada_en_la_frontera_si_cuenta_post() {
        let (pre, post) = calcular_extras(17 * 60, 17 * 60 + 30, 9 * 60, 17 * 60);
        assert_eq!((pre, post), (0, 30));
    }
}
