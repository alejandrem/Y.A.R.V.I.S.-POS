use crate::backventanas::auth::{AuthState, Role};
use crate::models::{AdminData, AdminProfile};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordVerifier};
use rand::thread_rng;
use sqlx::SqlitePool;

// ============================================================
// HASHING DE CONTRASEÑAS CON ARGON2ID (OWASP)
// ============================================================

fn hash_password(pass: &str) -> String {
    let salt = SaltString::generate(&mut thread_rng());
    let hash = Argon2::default()
        .hash_password(pass.as_bytes(), &salt)
        .expect("Error al hashear contraseña con Argon2");
    hash.to_string()
}

/// Verifica una contraseña contra su hash PHC de Argon2.
/// Si el hash no es válido (datos viejos en texto plano), compara directamente
/// para no bloquear al usuario existente.
fn verify_password(pass: &str, stored: &str) -> bool {
    match argon2::password_hash::PasswordHash::new(stored) {
        Ok(parsed) => Argon2::default()
            .verify_password(pass.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => pass == stored,
    }
}

// ============================================================
// COMMANDS
// ============================================================

#[tauri::command]
pub async fn check_setup_done(state: tauri::State<'_, SqlitePool>) -> Result<bool, String> {
    let result = sqlx::query_as::<_, (i32,)>("SELECT COUNT(*) FROM usuarios WHERE rol = 'admin'")
        .fetch_one(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.0 > 0)
}

#[tauri::command]
pub async fn guardar_admin(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    data: AdminData,
) -> Result<String, String> {
    let admins: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM usuarios WHERE rol = 'admin'")
        .fetch_one(&*state)
        .await
        .map_err(|e| e.to_string())?;
    if admins.0 > 0 || auth.require_authenticated().is_ok() {
        return Err("La configuración inicial ya fue completada".to_string());
    }

    let hashed = hash_password(&data.pass);

    sqlx::query("INSERT INTO usuarios (nombre, tienda, password, rol) VALUES (?, ?, ?, ?)")
        .bind(&data.name)
        .bind(&data.store)
        .bind(&hashed)
        .bind("admin")
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Admin guardado correctamente".into())
}

#[tauri::command]
pub async fn validar_login_admin(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    pass: String,
) -> Result<bool, String> {
    auth.logout();
    let result = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, nombre, password FROM usuarios WHERE rol = 'admin' LIMIT 1",
    )
    .fetch_optional(&*state)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = result {
        let valid = verify_password(&pass, &row.2);
        if valid {
            auth.login(row.0, Role::Admin, row.1);
        }
        Ok(valid)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn get_admin_data(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Option<AdminProfile>, String> {
    auth.require_admin()?;
    let result = sqlx::query_as::<_, (String, String, Option<String>, Option<String>)>(
        "SELECT nombre, tienda, ubicacion, cp FROM usuarios WHERE rol = 'admin' LIMIT 1",
    )
    .fetch_optional(&*state)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = result {
        Ok(Some(AdminProfile {
            nombre: row.0,
            tienda: row.1,
            ubicacion: row.2,
            cp: row.3,
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn update_admin_data(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    nombre: String,
    tienda: String,
    pass: String,
    ubicacion: String,
    cp: String,
) -> Result<String, String> {
    auth.require_admin()?;
    if pass.is_empty() {
        // Sin contraseña nueva: actualizar todo EXCEPTO password
        sqlx::query(
            "UPDATE usuarios SET nombre = ?, tienda = ?, ubicacion = ?, cp = ? WHERE rol = 'admin'",
        )
        .bind(&nombre)
        .bind(&tienda)
        .bind(&ubicacion)
        .bind(&cp)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        // Hay contraseña nueva: hashear y guardar todo
        let hashed = hash_password(&pass);
        sqlx::query("UPDATE usuarios SET nombre = ?, tienda = ?, password = ?, ubicacion = ?, cp = ? WHERE rol = 'admin'")
            .bind(&nombre)
            .bind(&tienda)
            .bind(&hashed)
            .bind(&ubicacion)
            .bind(&cp)
            .execute(&*state)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok("Datos actualizados correctamente".into())
}

#[tauri::command]
pub async fn guardar_empleado(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    name: String,
    pass: String,
) -> Result<String, String> {
    let admins: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM usuarios WHERE rol = 'admin'")
        .fetch_one(&*state)
        .await
        .map_err(|e| e.to_string())?;
    if admins.0 > 0 && auth.require_admin().is_err() {
        return Err("Se requiere una sesión de administrador".to_string());
    }
    let hashed = hash_password(&pass);

    sqlx::query("INSERT INTO usuarios (nombre, password, rol) VALUES (?, ?, ?)")
        .bind(&name)
        .bind(&hashed)
        .bind("empleado")
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Empleado guardado correctamente".into())
}

#[tauri::command]
pub async fn validar_login_empleado(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    pass: String,
) -> Result<Option<String>, String> {
    auth.logout();
    let rows = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT nombre, password, id FROM usuarios WHERE rol = 'empleado' AND estado = 'activo'",
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    for (nombre, hash, id) in rows {
        if verify_password(&pass, &hash) {
            let _ = sqlx::query(
                "UPDATE usuarios SET ultimo_login = datetime('now', 'localtime') WHERE id = ?",
            )
            .bind(id)
            .execute(&*state)
            .await;
            auth.login(id, Role::Employee, nombre.clone());
            return Ok(Some(nombre));
        }
    }

    Ok(None)
}

#[tauri::command]
pub async fn cerrar_sesion(auth: tauri::State<'_, AuthState>) -> Result<(), String> {
    auth.logout();
    Ok(())
}
