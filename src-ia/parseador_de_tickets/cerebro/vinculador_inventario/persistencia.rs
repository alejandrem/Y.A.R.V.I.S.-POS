use rusqlite::Connection;

pub fn guardar_vinculacion(
    vinculaciones: &[serde_json::Value],
    db_path: &str,
) -> Result<usize, String> {
    if vinculaciones.is_empty() {
        return Err("No hay vinculaciones para guardar".to_string());
    }

    let conn = Connection::open(db_path)
        .map_err(|e| format!("No se pudo abrir la base de datos: {e}"))?;

    let mut actualizados = 0usize;
    for v in vinculaciones {
        let detalle_id = v.get("detalle_id").and_then(|x| x.as_i64());
        let producto_id = v.get("producto_id").and_then(|x| x.as_i64());
        if let (Some(detalle_id), Some(producto_id)) = (detalle_id, producto_id) {
            conn.execute(
                "UPDATE detalle_ventas SET producto_id = ?1 WHERE id = ?2",
                rusqlite::params![producto_id, detalle_id],
            )
            .map_err(|e| format!("Error actualizando vinculación {detalle_id}: {e}"))?;
            actualizados += 1;
        }
    }

    Ok(actualizados)
}
