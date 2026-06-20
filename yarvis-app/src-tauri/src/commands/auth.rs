use sqlx::SqlitePool;
use crate::models::{AdminData, AdminProfile};

#[tauri::command]
pub async fn check_setup_done(state: tauri::State<'_, SqlitePool>) -> Result<bool, String> {
    let result = sqlx::query_as::<_, (i32,)>("SELECT COUNT(*) FROM usuarios WHERE rol = 'admin'")
        .fetch_one(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.0 > 0)
}

#[tauri::command]
pub async fn guardar_admin(state: tauri::State<'_, SqlitePool>, data: AdminData) -> Result<String, String> {
    let _hashed_pass = hash_password_placeholder(&data.pass);

    sqlx::query("INSERT INTO usuarios (nombre, tienda, password, rol) VALUES (?, ?, ?, ?)")
        .bind(&data.name)
        .bind(&data.store)
        .bind(&data.pass)
        .bind("admin")
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Admin guardado correctamente".into())
}

#[tauri::command]
pub async fn validar_login_admin(state: tauri::State<'_, SqlitePool>, pass: String) -> Result<bool, String> {
    let result = sqlx::query_as::<_, (String,)>("SELECT password FROM usuarios WHERE rol = 'admin' LIMIT 1")
        .fetch_optional(&*state)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(row) = result {
        Ok(row.0 == pass)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn get_admin_data(state: tauri::State<'_, SqlitePool>) -> Result<Option<AdminProfile>, String> {
    let result = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>)>(
        "SELECT nombre, tienda, password, ubicacion, cp FROM usuarios WHERE rol = 'admin' LIMIT 1"
    )
    .fetch_optional(&*state)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = result {
        Ok(Some(AdminProfile {
            nombre: row.0,
            tienda: row.1,
            password: row.2,
            ubicacion: row.3,
            cp: row.4,
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn update_admin_data(
    state: tauri::State<'_, SqlitePool>,
    nombre: String,
    tienda: String,
    pass: String,
    ubicacion: String,
    cp: String
) -> Result<String, String> {
    let _hashed_pass = hash_password_placeholder(&pass);

    sqlx::query("UPDATE usuarios SET nombre = ?, tienda = ?, password = ?, ubicacion = ?, cp = ? WHERE rol = 'admin'")
        .bind(&nombre)
        .bind(&tienda)
        .bind(&pass)
        .bind(&ubicacion)
        .bind(&cp)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Datos actualizados correctamente".into())
}

#[tauri::command]
pub async fn guardar_empleado(state: tauri::State<'_, SqlitePool>, name: String, pass: String) -> Result<String, String> {
    let _hashed_pass = hash_password_placeholder(&pass);

    sqlx::query("INSERT INTO usuarios (nombre, password, rol) VALUES (?, ?, ?)")
        .bind(&name)
        .bind(&pass)
        .bind("empleado")
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Empleado guardado correctamente".into())
}

#[tauri::command]
pub async fn validar_login_empleado(state: tauri::State<'_, SqlitePool>, pass: String) -> Result<Option<String>, String> {
    let result = sqlx::query_as::<_, (String,)>("SELECT nombre FROM usuarios WHERE password = ? AND rol = 'empleado' LIMIT 1")
        .bind(&pass)
        .fetch_optional(&*state)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(row) = result {
        Ok(Some(row.0))
    } else {
        Ok(None)
    }
}

// ============================================================
// PLACEHOLDER PARA HASHING DE CONTRASEN~AS
// ============================================================

// TODO: Reemplazar con argon2::hash_default(password.as_bytes())
// Dependencia a agregar: argon2 = "0.5"
fn hash_password_placeholder(pass: &str) -> String {
    pass.to_string()
}
