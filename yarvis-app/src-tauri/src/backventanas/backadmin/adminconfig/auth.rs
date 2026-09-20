use crate::backventanas::auth::{AuthState, Role};
use crate::dinero::a_centavos;
use crate::models::{AdminData, AdminProfile};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordVerifier};
use rand::thread_rng;
use sqlx::SqlitePool;

// ============================================================
// HASHING DE CONTRASEÑAS CON ARGON2ID (OWASP)
// ============================================================

pub fn hash_password(pass: &str) -> String {
    let salt = SaltString::generate(&mut thread_rng());
    let hash = Argon2::default()
        .hash_password(pass.as_bytes(), &salt)
        .expect("Error al hashear contraseña con Argon2");
    hash.to_string()
}

/// Verifica una contraseña contra su hash PHC de Argon2.
///
/// SIN fallback a texto plano: si el hash no parsea, el acceso se deniega.
/// El fallback anterior perpetuaba credenciales sin hash indefinidamente;
/// todos los usuarios actuales fueron creados vía hash_password (Argon2id),
/// así que un valor que no parsea es corrupción o tampering, nunca legacy.
/// En ese caso el admin debe resetear la contraseña desde su panel.
pub fn verify_password(pass: &str, stored: &str) -> bool {
    match argon2::password_hash::PasswordHash::new(stored) {
        Ok(parsed) => Argon2::default()
            .verify_password(pass.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
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
    if auth.require_authenticated().is_ok() {
        return Err("La configuración inicial ya fue completada".to_string());
    }

    // FIX TOCTOU: antes era COUNT seguido de INSERT (dos sentencias) — dos
    // llamadas simultáneas antes del primer admin podían crear DOS admins.
    // Ahora la verificación y el alta son UNA sola sentencia atómica.
    let hashed = hash_password(&data.pass);
    let result = sqlx::query(
        "INSERT INTO usuarios (nombre, tienda, password, rol)
         SELECT ?, ?, ?, 'admin'
         WHERE (SELECT COUNT(*) FROM usuarios WHERE rol = 'admin') = 0",
    )
    .bind(&data.name)
    .bind(&data.store)
    .bind(&hashed)
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err("La configuración inicial ya fue completada".to_string());
    }

    Ok("Admin guardado correctamente".into())
}

#[tauri::command]
pub async fn validar_login_admin(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    pass: String,
) -> Result<bool, String> {
    auth.logout();
    auth.rate_limiter.verificar().map_err(|segundos| {
        format!("Demasiados intentos fallidos. Espera {segundos} segundos antes de reintentar.")
    })?;
    let result = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, nombre, password FROM usuarios WHERE rol = 'admin' LIMIT 1",
    )
    .fetch_optional(&*state)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = result {
        let valid = verify_password(&pass, &row.2);
        if valid {
            auth.rate_limiter.registrar_exito();
            auth.login(row.0, Role::Admin, row.1);
        } else {
            auth.rate_limiter.registrar_fallo();
        }
        Ok(valid)
    } else {
        auth.rate_limiter.registrar_fallo();
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

/// Bloque de horario semanal que llega desde el registro unificado.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BloqueHorario {
    pub dias: Vec<i32>,
    pub hora_inicio: String,
    pub hora_fin: String,
}

#[tauri::command]
pub async fn guardar_empleado(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    name: String,
    pass: String,
    salario_semanal: Option<f64>,
    horarios: Option<Vec<BloqueHorario>>,
) -> Result<String, String> {
    let admins: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM usuarios WHERE rol = 'admin'")
        .fetch_one(&*state)
        .await
        .map_err(|e| e.to_string())?;
    if admins.0 > 0 && auth.require_admin().is_err() {
        return Err("Se requiere una sesión de administrador".to_string());
    }
    guardar_empleado_impl(&*state, name, pass, salario_semanal, horarios).await
}

/// Núcleo de alta de empleado, testeable sin runtime de Tauri.
pub async fn guardar_empleado_impl(
    pool: &SqlitePool,
    name: String,
    pass: String,
    salario_semanal: Option<f64>,
    horarios: Option<Vec<BloqueHorario>>,
) -> Result<String, String> {
    // El login de empleado es solo por contrasena (sin usuario), por lo que
    // dos empleados NO pueden compartir la misma clave: seria ambiguo.
    let hashes: Vec<(String,)> =
        sqlx::query_as("SELECT password FROM usuarios WHERE rol = 'empleado'")
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
    for (hash,) in &hashes {
        if verify_password(&pass, hash) {
            return Err("Ya existe un empleado con esa contraseña. Debe ser distinta para poder identificarlo en el login.".to_string());
        }
    }

    // Registro unificado: alta con bloques de horario y pago SEMANAL en una
    // sola llamada. Los campos opcionales conservan compatibilidad con el
    // flujo de primer inicio (que solo envia nombre y contrasena).
    let bloques = horarios.unwrap_or_default();
    for b in &bloques {
        if b.dias.is_empty() {
            return Err("Cada horario debe tener al menos un día seleccionado".to_string());
        }
        if b.hora_inicio.is_empty() || b.hora_fin.is_empty() {
            return Err("Cada horario debe tener hora de entrada y salida".to_string());
        }
    }
    // Un día no puede pertenecer a dos bloques distintos (sería ambiguo).
    let mut todos_dias: Vec<i32> = bloques.iter().flat_map(|b| b.dias.iter().copied()).collect();
    let total_dias = todos_dias.len() as i32;
    todos_dias.sort_unstable();
    todos_dias.dedup();
    if todos_dias.len() as i32 != total_dias {
        return Err("Un día no puede estar en dos horarios distintos".to_string());
    }

    let semanal_c = a_centavos(salario_semanal.unwrap_or(0.0).max(0.0));
    // División entera redondeada: salario diario derivado en centavos.
    let diario_c = if total_dias > 0 {
        (semanal_c as f64 / total_dias as f64).round() as i64
    } else {
        0
    };
    let inicio = bloques.first().map(|b| b.hora_inicio.clone()).unwrap_or_else(|| "00:00".into());
    let fin = bloques.first().map(|b| b.hora_fin.clone()).unwrap_or_else(|| "00:00".into());

    let hashed = hash_password(&pass);

    let result = sqlx::query(
        "INSERT INTO usuarios (nombre, password, rol, turno, horario_inicio, horario_fin, salario_diario, dias_semana, salario_semanal)
         VALUES (?, ?, 'empleado', '', ?, ?, ?, ?, ?)",
    )
    .bind(&name)
    .bind(&hashed)
    .bind(&inicio)
    .bind(&fin)
    .bind(diario_c)
    .bind(total_dias)
    .bind(semanal_c)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Persistir los bloques completos del empleado.
    let empleado_id = result.last_insert_rowid();
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

    Ok("Empleado guardado correctamente".into())
}

#[tauri::command]
pub async fn validar_login_empleado(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    pass: String,
) -> Result<Option<String>, String> {
    auth.logout();
    auth.rate_limiter.verificar().map_err(|segundos| {
        format!("Demasiados intentos fallidos. Espera {segundos} segundos antes de reintentar.")
    })?;
    let rows = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT nombre, password, id FROM usuarios WHERE rol = 'empleado' AND estado = 'activo'",
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    for (nombre, hash, id) in rows {
        if verify_password(&pass, &hash) {
            auth.rate_limiter.registrar_exito();
            let _ = sqlx::query(
                "UPDATE usuarios SET ultimo_login = datetime('now', 'localtime') WHERE id = ?",
            )
            .bind(id)
            .execute(&*state)
            .await;
            // Asistencia: el PRIMER login del día queda como entrada real;
            // los siguientes solo refrescan ultimo_login de la asistencia.
            if let Err(e) =
                crate::backventanas::backempleado::empleaperfil::asistencia::registrar_asistencia(&state, id).await
            {
                tracing::warn!("[ASISTENCIA] no se pudo registrar el login del empleado {id}: {e}");
            }
            auth.login(id, Role::Employee, nombre.clone());
            return Ok(Some(nombre));
        }
    }

    // Ningún hash coincidió: fallo contable para el rate limiter.
    auth.rate_limiter.registrar_fallo();
    Ok(None)
}

#[tauri::command]
pub async fn cerrar_sesion(auth: tauri::State<'_, AuthState>) -> Result<(), String> {
    auth.logout();
    Ok(())
}
