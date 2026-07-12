use sqlx::SqlitePool;

#[tauri::command]
pub async fn update_empleado(
    state: tauri::State<'_, SqlitePool>,
    empleado_id: i32,
    nombre: String,
    estado: String,
    turno: String,
    horario_inicio: String,
    horario_fin: String,
    salario_semanal: f64,
    salario_diario: f64,
    dias_semana: i32,
    meta_mensual: f64,
    bono: f64,
) -> Result<String, String> {
    sqlx::query(
        "UPDATE usuarios SET nombre = ?, estado = ?, turno = ?, horario_inicio = ?, horario_fin = ?,
         salario_semanal = ?, salario_diario = ?, dias_semana = ?, meta_mensual = ?, bono = ? WHERE id = ?"
    )
    .bind(&nombre)
    .bind(&estado)
    .bind(&turno)
    .bind(&horario_inicio)
    .bind(&horario_fin)
    .bind(salario_semanal)
    .bind(salario_diario)
    .bind(dias_semana)
    .bind(meta_mensual)
    .bind(bono)
    .bind(empleado_id)
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok("Empleado actualizado".into())
}

#[tauri::command]
pub async fn delete_empleado(state: tauri::State<'_, SqlitePool>, empleado_id: i32) -> Result<String, String> {
    sqlx::query("DELETE FROM usuarios WHERE id = ? AND rol = 'empleado'")
        .bind(empleado_id)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Empleado eliminado".into())
}
