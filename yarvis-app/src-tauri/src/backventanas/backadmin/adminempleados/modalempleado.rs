use crate::backventanas::auth::AuthState;
use crate::backventanas::backadmin::adminconfig::auth::{verify_password, hash_password, BloqueHorario};
use sqlx::SqlitePool;#[tauri::command]
pub async fn editar_empleado(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    empleado_id: i32,
    nombre: String,
    salario_semanal: Option<f64>,
    horarios: Option<Vec<BloqueHorario>>,
    nueva_password: Option<String>,
) -> Result<String, String> {
    auth.require_admin()?;
    editar_empleado_impl(&*state, empleado_id, nombre, salario_semanal, horarios, nueva_password).await
}

/// Núcleo de edición de empleado, testeable sin runtime de Tauri.
pub async fn editar_empleado_impl(
    pool: &SqlitePool,
    empleado_id: i32,
    nombre: String,
    salario_semanal: Option<f64>,
    horarios: Option<Vec<BloqueHorario>>,
    nueva_password: Option<String>,
) -> Result<String, String> {
    // Mismas validaciones que el alta: bloques completos y sin días repetidos.
    let bloques = horarios.unwrap_or_default();
    for b in &bloques {
        if b.dias.is_empty() {
            return Err("Cada horario debe tener al menos un día seleccionado".to_string());
        }
        if b.hora_inicio.is_empty() || b.hora_fin.is_empty() {
            return Err("Cada horario debe tener hora de entrada y salida".to_string());
        }
    }
    let mut todos_dias: Vec<i32> = bloques.iter().flat_map(|b| b.dias.iter().copied()).collect();
    let total_dias = todos_dias.len() as i32;
    todos_dias.sort_unstable();
    todos_dias.dedup();
    if todos_dias.len() as i32 != total_dias {
        return Err("Un día no puede estar en dos horarios distintos".to_string());
    }

    // Cambio de contraseña opcional (vacío = dejar la actual).
    let pass_nueva = match nueva_password.as_deref() {
        Some(p) if !p.is_empty() => {
            // Login solo por contraseña: no puede duplicar la de otros empleados.
            let hashes: Vec<(String,)> =
                sqlx::query_as("SELECT password FROM usuarios WHERE rol = 'empleado' AND id != ?")
                    .bind(empleado_id)
                    .fetch_all(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            for (hash,) in &hashes {
                if verify_password(p, hash) {
                    return Err("Ya existe otro empleado con esa contraseña. Debe ser distinta para poder identificarlo en el login.".to_string());
                }
            }
            Some(hash_password(p))
        }
        _ => None,
    };

    let semanal = salario_semanal.unwrap_or(0.0).max(0.0);
    let diario = if total_dias > 0 { semanal / total_dias as f64 } else { 0.0 };
    let inicio = bloques.first().map(|b| b.hora_inicio.clone()).unwrap_or_else(|| "00:00".into());
    let fin = bloques.first().map(|b| b.hora_fin.clone()).unwrap_or_else(|| "00:00".into());

    sqlx::query(
        "UPDATE usuarios SET nombre = ?, salario_semanal = ?, salario_diario = ?, dias_semana = ?,
         horario_inicio = ?, horario_fin = ? WHERE id = ? AND rol = 'empleado'",
    )
    .bind(&nombre)
    .bind(semanal)
    .bind(diario)
    .bind(total_dias)
    .bind(&inicio)
    .bind(&fin)
    .bind(empleado_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(hashed) = pass_nueva {
        sqlx::query("UPDATE usuarios SET password = ? WHERE id = ?")
            .bind(&hashed)
            .bind(empleado_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Reemplazo completo de bloques de horario.
    sqlx::query("DELETE FROM empleado_horarios WHERE empleado_id = ?")
        .bind(empleado_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    for b in &bloques {
        let dias_txt: Vec<String> = b.dias.iter().map(|d| d.to_string()).collect();
        sqlx::query("INSERT INTO empleado_horarios (empleado_id, dias, hora_inicio, hora_fin) VALUES (?, ?, ?, ?)")
            .bind(empleado_id)
            .bind(dias_txt.join(","))
            .bind(&b.hora_inicio)
            .bind(&b.hora_fin)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok("Empleado actualizado correctamente".into())
}

/// Activa o desactiva un empleado sin borrarlo: su historial de ventas,
/// cortes y nómina permanecen intactos; solo pierde el acceso al login
/// (validar_login_empleado solo acepta estado='activo').
#[tauri::command]
pub async fn set_estado_empleado(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    empleado_id: i32,
    estado: String,
) -> Result<String, String> {
    auth.require_admin()?;
    set_estado_empleado_impl(&*state, empleado_id, estado).await
}

/// Núcleo de activación/desactivación, testeable sin runtime de Tauri.
pub async fn set_estado_empleado_impl(
    pool: &SqlitePool,
    empleado_id: i32,
    estado: String,
) -> Result<String, String> {
    if estado != "activo" && estado != "inactivo" {
        return Err("Estado inválido".to_string());
    }
    let result = sqlx::query("UPDATE usuarios SET estado = ? WHERE id = ? AND rol = 'empleado'")
        .bind(&estado)
        .bind(empleado_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    if result.rows_affected() == 0 {
        return Err("Empleado no encontrado".into());
    }
    Ok(if estado == "activo" {
        "Empleado reactivado".into()
    } else {
        "Empleado desactivado".into()
    })
}
