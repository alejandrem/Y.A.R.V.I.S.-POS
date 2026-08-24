use crate::backventanas::auth::AuthState;
use crate::dinero::{a_centavos, a_pesos};
use crate::models::EmployeeGoal;
use sqlx::Row;
use sqlx::SqlitePool;

fn decode_f64(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<f64, _>(col)
        .or_else(|_| row.try_get::<i64, _>(col).map(|v| v as f64))
        .unwrap_or(0.0)
}

/// Columna monetaria INTEGER en centavos → pesos para structs IPC.
fn decode_cents(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    a_pesos(row.try_get::<i64, _>(col).unwrap_or(0))
}

#[tauri::command]
pub async fn get_employee_goals(
    auth: tauri::State<'_, AuthState>,
    state: tauri::State<'_, SqlitePool>,
    empleado_id: i32,
) -> Result<Vec<EmployeeGoal>, String> {
    auth.require_admin()?;
    let rows = sqlx::query(
        "SELECT id, employee_id, goal_type, goal_name, ventas_threshold, bonus_percentage, bonus_amount, is_completed, completed_at, created_at
         FROM employee_goals WHERE employee_id = ? ORDER BY id ASC"
    )
    .bind(empleado_id)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| EmployeeGoal {
            id: r.get("id"),
            employee_id: r.get("employee_id"),
            goal_type: r.get("goal_type"),
            goal_name: r.try_get("goal_name").ok(),
            ventas_threshold: r.get("ventas_threshold"),
            bonus_percentage: decode_f64(&r, "bonus_percentage"),
            bonus_amount: decode_cents(&r, "bonus_amount"),
            is_completed: r.get::<i32, _>("is_completed") != 0,
            completed_at: r.try_get("completed_at").ok(),
            created_at: r.try_get("created_at").ok(),
        })
        .collect())
}

#[tauri::command]
pub async fn save_employee_goal(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    empleado_id: i32,
    goal_type: String,
    goal_name: Option<String>,
    ventas_threshold: Option<String>,
    bonus_percentage: Option<f64>,
    bonus_amount: Option<f64>,
) -> Result<String, String> {
    auth.require_admin()?;
    let existing = sqlx::query_as::<_, (i32,)>(
        "SELECT id FROM employee_goals WHERE employee_id = ? AND goal_type = ?",
    )
    .bind(empleado_id)
    .bind(&goal_type)
    .fetch_optional(&*state)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = existing {
        sqlx::query(
            "UPDATE employee_goals SET goal_name = ?, ventas_threshold = ?, bonus_percentage = ?, bonus_amount = ? WHERE id = ?"
        )
        .bind(&goal_name)
        .bind(ventas_threshold.as_deref().unwrap_or("5"))
        .bind(bonus_percentage.unwrap_or(0.0))
        .bind(a_centavos(bonus_amount.unwrap_or(0.0)))
        .bind(row.0)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        sqlx::query(
            "INSERT INTO employee_goals (employee_id, goal_type, goal_name, ventas_threshold, bonus_percentage, bonus_amount) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(empleado_id)
        .bind(&goal_type)
        .bind(&goal_name)
        .bind(ventas_threshold.as_deref().unwrap_or("5"))
        .bind(bonus_percentage.unwrap_or(0.0))
        .bind(a_centavos(bonus_amount.unwrap_or(0.0)))
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok("Meta guardada".into())
}

#[tauri::command]
pub async fn save_custom_goal(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    empleado_id: i32,
    goal_name: String,
    bonus_amount: f64,
) -> Result<String, String> {
    auth.require_admin()?;
    sqlx::query(
        "INSERT INTO employee_goals (employee_id, goal_type, goal_name, bonus_amount) VALUES (?, 'custom', ?, ?)"
    )
    .bind(empleado_id)
    .bind(&goal_name)
    .bind(a_centavos(bonus_amount))
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok("Meta personalizada creada".into())
}

#[tauri::command]
pub async fn delete_employee_goal(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    goal_id: i32,
) -> Result<String, String> {
    auth.require_admin()?;
    sqlx::query("DELETE FROM employee_goals WHERE id = ?")
        .bind(goal_id)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Meta eliminada".into())
}

#[tauri::command]
pub async fn check_employee_goals(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    empleado_id: i32,
) -> Result<Vec<EmployeeGoal>, String> {
    auth.require_admin()?;

    let horario_row =
        sqlx::query_as::<_, (String,)>("SELECT horario_inicio FROM usuarios WHERE id = ?")
            .bind(empleado_id)
            .fetch_one(&*state)
            .await
            .map_err(|e| e.to_string())?;
    let horario_inicio = horario_row.0;

    let goal_rows = sqlx::query(
        "SELECT id, employee_id, goal_type, goal_name, ventas_threshold, bonus_percentage, bonus_amount, is_completed, completed_at, created_at
         FROM employee_goals WHERE employee_id = ?"
    )
    .bind(empleado_id)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let mut result = Vec::new();

    for row in goal_rows {
        let goal_id: i32 = row.get("id");
        let g_type: String = row.get("goal_type");
        let mut is_completed: bool = row.get::<i32, _>("is_completed") != 0;

        let bp = decode_f64(&row, "bonus_percentage");
        let ba = decode_cents(&row, "bonus_amount");

        if !is_completed {
            match g_type.as_str() {
                "ventas" => {
                    // SUM(total) sobre columna INTEGER: resultado en centavos.
                    let ventas_row = sqlx::query_as::<_, (i64,)>(
                        "SELECT COALESCE(SUM(total), 0) FROM ventas WHERE cajero_id = ? AND estado = 'completada'
                         AND fecha >= date('now', 'start of week') AND fecha < date('now', '+1 day', 'start of week', '+7 days')"
                    )
                    .bind(empleado_id)
                    .fetch_one(&*state)
                    .await
                    .map_err(|e| e.to_string())?;
                    let ventas_semana = ventas_row.0;

                    let vt: String = row.get("ventas_threshold");
                    let umbral: f64 = vt.parse().unwrap_or(0.0);

                    if umbral > 0.0 && ventas_semana >= a_centavos(umbral) {
                        is_completed = true;
                        sqlx::query("UPDATE employee_goals SET is_completed = 1, completed_at = datetime('now','localtime') WHERE id = ?")
                            .bind(goal_id)
                            .execute(&*state)
                            .await
                            .map_err(|e| e.to_string())?;
                    }
                }
                "puntualidad" => {
                    let ultimo_login = sqlx::query_as::<_, (Option<String>,)>(
                        "SELECT ultimo_login FROM usuarios WHERE id = ?",
                    )
                    .bind(empleado_id)
                    .fetch_one(&*state)
                    .await
                    .map_err(|e| e.to_string())?;

                    if let Some(ref login_str) = ultimo_login.0 {
                        if es_puntual(login_str, &horario_inicio) {
                            is_completed = true;
                            sqlx::query("UPDATE employee_goals SET is_completed = 1, completed_at = datetime('now','localtime') WHERE id = ?")
                                .bind(goal_id)
                                .execute(&*state)
                                .await
                                .map_err(|e| e.to_string())?;
                        }
                    }
                }
                _ => {}
            }
        }

        let vt: String = row.get("ventas_threshold");
        let gn: Option<String> = row.try_get("goal_name").ok();
        let ca: Option<String> = row.try_get("completed_at").ok();
        let cr: Option<String> = row.try_get("created_at").ok();

        result.push(EmployeeGoal {
            id: goal_id,
            employee_id: empleado_id,
            goal_type: g_type,
            goal_name: gn,
            ventas_threshold: vt,
            bonus_percentage: bp,
            bonus_amount: ba,
            is_completed,
            completed_at: ca,
            created_at: cr,
        });
    }

    Ok(result)
}

fn es_puntual(login_str: &str, horario_inicio: &str) -> bool {
    let parse_dt = |s: &str| -> Option<(i32, i32, i32, i32, i32)> {
        let s = s.replace("T", " ").replace("Z", "");
        let parts: Vec<&str> = s.split(|c| c == ' ' || c == ':').collect();
        if parts.len() >= 5 {
            Some((
                parts[0].parse().unwrap_or(0),
                parts[1].parse().unwrap_or(0),
                parts[2].parse().unwrap_or(0),
                parts[3].parse().unwrap_or(0),
                parts[4].parse().unwrap_or(0),
            ))
        } else {
            None
        }
    };

    let login = match parse_dt(login_str) {
        Some(v) => v,
        None => return false,
    };

    let h_inicio: Vec<&str> = horario_inicio.split(':').collect();
    let inicio_h: i32 = h_inicio[0].parse().unwrap_or(0);
    let inicio_m: i32 = h_inicio.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);

    let login_minutos = login.3 * 60 + login.4;
    let inicio_minutos = inicio_h * 60 + inicio_m;

    login_minutos <= inicio_minutos + 5
}
