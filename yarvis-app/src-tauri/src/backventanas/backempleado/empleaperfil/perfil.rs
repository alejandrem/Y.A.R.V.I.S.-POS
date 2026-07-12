use sqlx::SqlitePool;
use sqlx::Row;
use serde::Serialize;

#[derive(Serialize)]
pub struct EmployeeProfile {
    pub id: i32,
    pub nombre: String,
    pub turno: String,
    pub horario_inicio: String,
    pub horario_fin: String,
    pub salario_diario: f64,
    pub salario_semanal: f64,
    pub salario_mensual: f64,
    pub salario_hora: f64,
    pub horas_por_dia: f64,
    pub dias_semana: i32,
    pub meta_mensual: f64,
    pub bono: f64,
    pub ultimo_login: Option<String>,
    pub estado: String,
}

#[derive(Serialize)]
pub struct EmployeeGoalSummary {
    pub goal_type: String,
    pub goal_name: Option<String>,
    pub bonus_amount: f64,
    pub bonus_percentage: f64,
    pub ventas_threshold: String,
    pub is_completed: bool,
}

#[derive(Serialize)]
pub struct EmployeeProfileFull {
    pub profile: EmployeeProfile,
    pub goals: Vec<EmployeeGoalSummary>,
}

fn decode_f64(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<f64, _>(col)
        .or_else(|_| row.try_get::<i64, _>(col).map(|v| v as f64))
        .unwrap_or(0.0)
}

#[tauri::command]
pub async fn get_employee_profile(
    state: tauri::State<'_, SqlitePool>,
    nombre: String,
) -> Result<EmployeeProfileFull, String> {
    let row = sqlx::query(
        "SELECT id, nombre, turno, horario_inicio, horario_fin, salario_diario, salario_semanal,
                dias_semana, meta_mensual, bono, ultimo_login, estado
         FROM usuarios WHERE nombre = ? AND rol = 'empleado'"
    )
    .bind(&nombre)
    .fetch_optional(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let r = match row {
        Some(r) => r,
        None => return Err("Empleado no encontrado".into()),
    };

    let salario_diario = decode_f64(&r, "salario_diario");
    let dias_semana: i32 = r.get("dias_semana");
    let horario_inicio: String = r.get("horario_inicio");
    let horario_fin: String = r.get("horario_fin");

    let horas_por_dia = calcular_horas(&horario_inicio, &horario_fin);
    let salario_hora = if horas_por_dia > 0.0 { salario_diario / horas_por_dia } else { 0.0 };
    let salario_semanal = salario_diario * dias_semana as f64;
    let salario_mensual = salario_semanal * 4.33;

    let profile = EmployeeProfile {
        id: r.get("id"),
        nombre: r.get("nombre"),
        turno: r.get("turno"),
        horario_inicio,
        horario_fin,
        salario_diario,
        salario_semanal,
        salario_mensual,
        salario_hora,
        horas_por_dia,
        dias_semana,
        meta_mensual: decode_f64(&r, "meta_mensual"),
        bono: decode_f64(&r, "bono"),
        ultimo_login: r.try_get("ultimo_login").ok(),
        estado: r.get("estado"),
    };

    let goal_rows = sqlx::query(
        "SELECT goal_type, goal_name, bonus_amount, bonus_percentage, ventas_threshold, is_completed
         FROM employee_goals WHERE employee_id = ?"
    )
    .bind(profile.id)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let goals = goal_rows.into_iter().map(|gr| EmployeeGoalSummary {
        goal_type: gr.get("goal_type"),
        goal_name: gr.try_get("goal_name").ok(),
        bonus_amount: decode_f64(&gr, "bonus_amount"),
        bonus_percentage: decode_f64(&gr, "bonus_percentage"),
        ventas_threshold: gr.get("ventas_threshold"),
        is_completed: gr.get::<i32, _>("is_completed") != 0,
    }).collect();

    Ok(EmployeeProfileFull { profile, goals })
}

fn calcular_horas(inicio: &str, fin: &str) -> f64 {
    let parse_h = |t: &str| -> f64 {
        let parts: Vec<&str> = t.split(':').collect();
        if parts.len() >= 2 {
            let h: f64 = parts[0].parse().unwrap_or(0.0);
            let m: f64 = parts[1].parse().unwrap_or(0.0);
            h + m / 60.0
        } else {
            0.0
        }
    };
    let h_inicio = parse_h(inicio);
    let h_fin = parse_h(fin);
    if h_fin > h_inicio {
        h_fin - h_inicio
    } else if h_fin < h_inicio {
        (24.0 - h_inicio) + h_fin
    } else {
        0.0
    }
}
