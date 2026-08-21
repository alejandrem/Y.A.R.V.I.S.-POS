use crate::backventanas::auth::AuthState;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::SqlitePool;

/// Bloque de horario semanal. `dias` usa la convención L=0..D=6.
#[derive(Serialize, Deserialize)]
pub struct HorarioBloque {
    pub dias: Vec<i32>,
    pub hora_inicio: String,
    pub hora_fin: String,
}

#[derive(Serialize, Deserialize)]
pub struct EmpleadoProfile {
    pub id: i32,
    pub nombre: String,
    pub estado: String,
    pub turno: String,
    pub horario_inicio: String,
    pub horario_fin: String,
    pub salario_semanal: f64,
    pub salario_diario: f64,
    pub dias_semana: i32,
    pub meta_mensual: f64,
    pub bono: f64,
    pub registrado_en: Option<String>,
    pub ultimo_login: Option<String>,
    /// Bloques de horario completos (soporta jornadas partidas).
    #[serde(default)]
    pub horarios: Vec<HorarioBloque>,
}

#[derive(Serialize)]
pub struct EmpleadoVentas {
    pub empleado_id: i32,
    pub nombre: String,
    pub total_ventas: f64,
    pub ventas_canceladas: f64,
    pub total_canceladas_count: i32,
    pub ventas_con_descuento: f64,
    pub ticket_count: i32,
}

#[derive(Serialize)]
pub struct EmpleadoResumen {
    pub empleados_activos: i32,
    pub ventas_totales: f64,
    pub tickets_totales: i32,
    pub costo_nomina: f64,
    pub roi_neto: f64,
}

#[derive(Serialize)]
pub struct CorteEmpleado {
    pub id: i32,
    pub fecha_apertura: Option<String>,
    pub fecha_cierre: Option<String>,
    pub monto_inicial: f64,
    pub total_ventas: f64,
    pub estado: String,
}

fn decode_f64(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<f64, _>(col)
        .or_else(|_| row.try_get::<i64, _>(col).map(|v| v as f64))
        .unwrap_or(0.0)
}

#[tauri::command]
pub async fn get_empleados(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<EmpleadoProfile>, String> {
    auth.require_admin()?;
    let rows = sqlx::query(
        "SELECT id, nombre, estado, turno, horario_inicio, horario_fin, salario_semanal, salario_diario, dias_semana, meta_mensual, bono, registrado_en, ultimo_login
         FROM usuarios WHERE rol = 'empleado' ORDER BY nombre ASC"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    // Bloques de horario en una sola query, agrupados por empleado.
    let bloques = sqlx::query(
        "SELECT empleado_id, dias, hora_inicio, hora_fin FROM empleado_horarios ORDER BY id ASC",
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;
    let mut horarios_por_empleado: std::collections::HashMap<i32, Vec<HorarioBloque>> =
        std::collections::HashMap::new();
    for b in bloques {
        let empleado_id: i32 = b.get("empleado_id");
        let dias_txt: String = b.get("dias");
        let dias: Vec<i32> = dias_txt
            .split(',')
            .filter_map(|d| d.trim().parse::<i32>().ok())
            .collect();
        horarios_por_empleado.entry(empleado_id).or_default().push(HorarioBloque {
            dias,
            hora_inicio: b.get("hora_inicio"),
            hora_fin: b.get("hora_fin"),
        });
    }

    Ok(rows
        .into_iter()
        .map(|r| {
            let id: i32 = r.get("id");
            EmpleadoProfile {
                id,
                nombre: r.get("nombre"),
                estado: r.get("estado"),
                turno: r.get("turno"),
                horario_inicio: r.get("horario_inicio"),
                horario_fin: r.get("horario_fin"),
                salario_semanal: decode_f64(&r, "salario_semanal"),
                salario_diario: decode_f64(&r, "salario_diario"),
                dias_semana: r.get("dias_semana"),
                meta_mensual: decode_f64(&r, "meta_mensual"),
                bono: decode_f64(&r, "bono"),
                registrado_en: r.try_get("registrado_en").ok(),
                ultimo_login: r.try_get("ultimo_login").ok(),
                horarios: horarios_por_empleado.remove(&id).unwrap_or_default(),
            }
        })
        .collect())
}

#[tauri::command]
pub async fn get_empleado_ventas(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    empleado_id: i32,
) -> Result<EmpleadoVentas, String> {
    auth.require_admin()?;
    let nombre_row = sqlx::query_as::<_, (String,)>("SELECT nombre FROM usuarios WHERE id = ?")
        .bind(empleado_id)
        .fetch_optional(&*state)
        .await
        .map_err(|e| e.to_string())?;

    let nombre = match nombre_row {
        Some(r) => r.0,
        None => return Err("Empleado no encontrado".into()),
    };

    let ventas_row = sqlx::query_as::<_, (f64, i32)>(
        "SELECT COALESCE(SUM(total), 0) * 1.0, COUNT(*) FROM ventas WHERE cajero = ? AND estado = 'completada'"
    )
    .bind(&nombre)
    .fetch_one(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let canceladas_row = sqlx::query_as::<_, (f64, i32)>(
        "SELECT COALESCE(SUM(total), 0) * 1.0, COUNT(*) FROM ventas WHERE cajero = ? AND estado = 'cancelada'"
    )
    .bind(&nombre)
    .fetch_one(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let descuento_row = sqlx::query_as::<_, (f64,)>(
        "SELECT COALESCE(SUM(total), 0) * 1.0 FROM ventas WHERE cajero = ? AND estado = 'completada' AND descuento > 0"
    )
    .bind(&nombre)
    .fetch_one(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(EmpleadoVentas {
        empleado_id,
        nombre,
        total_ventas: ventas_row.0,
        ventas_canceladas: canceladas_row.0,
        total_canceladas_count: canceladas_row.1,
        ventas_con_descuento: descuento_row.0,
        ticket_count: ventas_row.1,
    })
}

#[tauri::command]
pub async fn get_resumen_empleados(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<EmpleadoResumen, String> {
    auth.require_admin()?;
    let activos = sqlx::query_as::<_, (i32,)>(
        "SELECT COUNT(*) FROM usuarios WHERE rol = 'empleado' AND estado = 'activo'",
    )
    .fetch_one(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let ventas = sqlx::query_as::<_, (f64, i32)>(
        "SELECT COALESCE(SUM(total), 0) * 1.0, COUNT(*) FROM ventas WHERE estado = 'completada'",
    )
    .fetch_one(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let nomina = sqlx::query_as::<_, (f64,)>("SELECT COALESCE(SUM(salario_semanal), 0) * 1.0 FROM usuarios WHERE rol = 'empleado' AND estado = 'activo'")
        .fetch_one(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok(EmpleadoResumen {
        empleados_activos: activos.0,
        ventas_totales: ventas.0,
        tickets_totales: ventas.1,
        costo_nomina: nomina.0,
        roi_neto: ventas.0 - nomina.0,
    })
}

#[tauri::command]
pub async fn get_cortes_empleado(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    empleado_id: i32,
) -> Result<Vec<CorteEmpleado>, String> {
    auth.require_admin()?;
    let nombre_row = sqlx::query_as::<_, (String,)>("SELECT nombre FROM usuarios WHERE id = ?")
        .bind(empleado_id)
        .fetch_optional(&*state)
        .await
        .map_err(|e| e.to_string())?;

    let nombre = match nombre_row {
        Some(r) => r.0,
        None => return Err("Empleado no encontrado".into()),
    };

    let rows = sqlx::query(
        "SELECT id, fecha_apertura, fecha_cierre, monto_inicial, total_ventas, estado
         FROM cortes_caja WHERE usuario_id IN (SELECT id FROM usuarios WHERE nombre = ?)
         ORDER BY fecha_apertura DESC LIMIT 20",
    )
    .bind(&nombre)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| CorteEmpleado {
            id: r.get("id"),
            fecha_apertura: r.try_get("fecha_apertura").ok(),
            fecha_cierre: r.try_get("fecha_cierre").ok(),
            monto_inicial: decode_f64(&r, "monto_inicial"),
            total_ventas: decode_f64(&r, "total_ventas"),
            estado: r.get("estado"),
        })
        .collect())
}
