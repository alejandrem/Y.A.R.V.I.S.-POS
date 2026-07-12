use sqlx::SqlitePool;
use sqlx::Row;

#[derive(serde::Serialize)]
pub struct TurnoInfo {
    pub empleado_id: i32,
    pub nombre: String,
    pub turno: String,
    pub horario_inicio: String,
    pub horario_fin: String,
}

#[tauri::command]
pub async fn get_turnos_empleados(
    state: tauri::State<'_, SqlitePool>,
) -> Result<Vec<TurnoInfo>, String> {
    let rows = sqlx::query(
        "SELECT id, nombre, turno, horario_inicio, horario_fin FROM usuarios WHERE rol = 'empleado' AND estado = 'activo' ORDER BY nombre ASC"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| TurnoInfo {
        empleado_id: r.get("id"),
        nombre: r.get("nombre"),
        turno: r.get("turno"),
        horario_inicio: r.get("horario_inicio"),
        horario_fin: r.get("horario_fin"),
    }).collect())
}
