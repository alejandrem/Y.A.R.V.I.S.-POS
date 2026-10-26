// ============================================================
// db::respaldo — Red de seguridad LOCAL antes de migrar/restaurar.
//
// Estos respaldos viven junto a la DB (misma carpeta) y están en claro:
// son para recuperar el arranque, NO son el respaldo real. El respaldo
// programado, cifrado y fuera del disco vive en `crate::respaldos`.
//
// Reglas:
//   · Un backup fallido nunca tumba el arranque (solo warn).
//   · VACUUM INTO da una copia consistente; fs::copy es el fallback.
//   · Restaurar valida integridad y tablas esenciales ANTES de tocar
//     la DB actual. Si el archivo está corrupto, no se reemplaza nada.
// ============================================================

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
use rusqlite::{Connection, OpenFlags};
use sqlx::SqlitePool;

/// Cuántos respaldos locales pre-migración se conservan.
pub const MAX_BACKUPS: usize = 5;
const PREFIJO: &str = "yarvis-";
const PREFIJO_SEGURIDAD: &str = "yarvis-antes-restaurar-";

/// Resumen de una base que pasó `PRAGMA integrity_check`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumenDb {
    pub tablas: i64,
    pub usuarios: i64,
    pub ventas: i64,
    pub productos: i64,
    pub detalle_ventas: i64,
    pub version_esquema: i64,
}

/// Copia consistente antes de aplicar migraciones. Devuelve `None` si no
/// se pudo crear (el arranque continúa, pero se registra el warning).
pub fn backup_db_antes_de_migrar(app_dir: &Path, db_path: &Path) -> Option<PathBuf> {
    let backups_dir = app_dir.join("backups");
    fs::create_dir_all(&backups_dir).ok()?;
    let destino = ruta_unica(&backups_dir, PREFIJO, "db");

    let vacuum_ok = src_ia::sqlite::abrir_db(db_path)
        .ok()
        .and_then(|conn| {
            let dest_sql = destino.to_string_lossy().replace('\'', "''");
            conn.execute_batch(&format!("VACUUM INTO '{dest_sql}';")).ok()
        })
        .is_some()
        && destino.exists();

    if !vacuum_ok {
        if fs::copy(db_path, &destino).is_err() {
            let _ = fs::remove_file(&destino);
            return None;
        }
    }

    rotar_backups(&backups_dir, MAX_BACKUPS);
    Some(destino)
}

/// Copia consistente de una DB abierta por el pool (antes de restaurar).
/// Es deliberadamente local y en claro: sirve para deshacer una restauración.
pub async fn backup_local_desde_pool(pool: &SqlitePool, app_dir: &Path) -> Result<PathBuf, String> {
    let backups_dir = app_dir.join("backups");
    fs::create_dir_all(&backups_dir)
        .map_err(|e| format!("No se pudo crear {}: {e}", backups_dir.display()))?;
    let destino = ruta_unica(&backups_dir, PREFIJO_SEGURIDAD, "db");
    let dest_sql = destino.to_string_lossy().replace('\'', "''");

    sqlx::query(&format!("VACUUM INTO '{dest_sql}'"))
        .execute(pool)
        .await
        .map_err(|e| format!("No se pudo crear el respaldo de seguridad: {e}"))?;

    if !destino.exists() {
        return Err("VACUUM INTO no produjo el respaldo de seguridad".to_string());
    }
    rotar_backups(&backups_dir, MAX_BACKUPS);
    Ok(destino)
}

/// Valida integridad y forma de una DB YARVIS sin modificarla.
/// Se usa tanto al restaurar desde el diálogo de arranque como desde el
/// wizard de respaldos cifrados.
pub fn probar_db(ruta: &Path) -> Result<ResumenDb, String> {
    if !ruta.is_file() {
        return Err(format!("No existe el archivo {}", ruta.display()));
    }
    let conn = Connection::open_with_flags(ruta, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("No se pudo abrir {}: {e}", ruta.display()))?;

    let integridad: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(|e| {
            let texto = e.to_string();
            if texto.contains("not a database") {
                format!(
                    "La base está dañada: {} no es un archivo SQLite válido. No se tocará la actual.",
                    ruta.display()
                )
            } else {
                format!("No se pudo verificar la integridad: {texto}")
            }
        })?;
    if integridad != "ok" {
        return Err(format!(
            "La base está dañada (integrity_check = {integridad}). No se tocará la actual."
        ));
    }

    let tablas: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Tablas mínimas del POS. Si faltan, el archivo no es un respaldo YARVIS
    // (o está a medias) y restaurarlo dejaría la caja inservible.
    for esencial in ["usuarios", "ventas", "productos"] {
        let existe: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [esencial],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if existe == 0 {
            return Err(format!(
                "El archivo no parece una base de YARVIS POS: falta la tabla '{esencial}'"
            ));
        }
    }

    let usuarios = contar(&conn, "usuarios")?;
    let ventas = contar(&conn, "ventas")?;
    let productos = contar(&conn, "productos")?;
    let detalle_ventas = contar_opcional(&conn, "detalle_ventas");
    let version_esquema = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(ResumenDb {
        tablas,
        usuarios,
        ventas,
        productos,
        detalle_ventas,
        version_esquema,
    })
}

/// Reemplaza la DB actual por un archivo YA VALIDADO, conservando una
/// copia de la anterior. Si algo falla, intenta dejar la DB original.
pub fn restaurar_db_plana(
    origen: &Path,
    db_path: &Path,
    app_dir: &Path,
) -> Result<Option<PathBuf>, String> {
    // Nunca confiar en el llamador: validar aquí es la última barrera.
    probar_db(origen)?;

    let seguridad = if db_path.exists() {
        let backups = app_dir.join("backups");
        fs::create_dir_all(&backups).map_err(|e| e.to_string())?;
        let copia = ruta_unica(&backups, PREFIJO_SEGURIDAD, "db");
        fs::copy(db_path, &copia).map_err(|e| {
            format!(
                "No se pudo respaldar la base actual antes de restaurar: {e}. Restauración cancelada."
            )
        })?;
        Some(copia)
    } else {
        None
    };

    instalar_db(origen, db_path)?;
    Ok(seguridad)
}

/// Valida e instala una DB ya respaldada, SIN crear otra copia de
/// seguridad (el llamador ya la hizo con VACUUM desde el pool abierto).
/// Si la validación falla, la DB actual no se toca.
pub fn instalar_db(origen: &Path, db_path: &Path) -> Result<(), String> {
    probar_db(origen)?;
    reemplazar_archivo(db_path, origen)
}

fn reemplazar_archivo(db_path: &Path, origen: &Path) -> Result<(), String> {
    let temp = db_path.with_extension("db.nuevo");
    let viejo = db_path.with_extension("db.viejo");
    let _ = fs::remove_file(&temp);
    let _ = fs::remove_file(&viejo);

    fs::copy(origen, &temp).map_err(|e| format!("No se pudo preparar la restauración: {e}"))?;

    let tenia_actual = db_path.exists();
    if tenia_actual {
        fs::rename(db_path, &viejo)
            .map_err(|e| format!("No se pudo apartar la base actual: {e}"))?;
    }
    cerrar_archivos_wal(db_path);

    match fs::rename(&temp, db_path) {
        Ok(()) => {
            let _ = fs::remove_file(&viejo);
            Ok(())
        }
        Err(e) => {
            // Rollback: devolver la base original si el rename falló.
            if tenia_actual && viejo.exists() {
                let _ = fs::rename(&viejo, db_path);
            }
            let _ = fs::remove_file(&temp);
            Err(format!("No se pudo instalar la base restaurada: {e}"))
        }
    }
}

fn cerrar_archivos_wal(db_path: &Path) {
    for sufijo in ["-wal", "-shm"] {
        let _ = fs::remove_file(format!("{}{sufijo}", db_path.display()));
    }
}

fn contar(conn: &Connection, tabla: &str) -> Result<i64, String> {
    // `tabla` solo llega desde nombres fijos de este archivo, nunca de UI.
    conn.query_row(&format!("SELECT COUNT(*) FROM {tabla}"), [], |r| r.get(0))
        .map_err(|e| format!("No se pudo contar {tabla}: {e}"))
}

fn contar_opcional(conn: &Connection, tabla: &str) -> i64 {
    contar(conn, tabla).unwrap_or(0)
}

/// Rota SOLO los `yarvis-AAAA...db`. Los `yarvis-antes-restaurar-*.db`
/// son evidencia de una restauración y no se borran automáticamente.
pub fn rotar_backups(backups_dir: &Path, max: usize) {
    let mut archivos: Vec<PathBuf> = fs::read_dir(backups_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| es_backup_rotativo(p))
                .collect()
        })
        .unwrap_or_default();
    archivos.sort();
    let max = max.max(1);
    if archivos.len() > max {
        for viejo in archivos.iter().take(archivos.len() - max) {
            let _ = fs::remove_file(viejo);
        }
    }
}

fn es_backup_rotativo(ruta: &Path) -> bool {
    let Some(nombre) = ruta.file_name().map(|n| n.to_string_lossy().to_string()) else {
        return false;
    };
    ruta.extension().is_some_and(|ext| ext == "db")
        && nombre.starts_with(PREFIJO)
        && !nombre.starts_with(PREFIJO_SEGURIDAD)
}

fn ruta_unica(dir: &Path, prefijo: &str, extension: &str) -> PathBuf {
    let sello = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let mut ruta = dir.join(format!("{prefijo}{sello}.{extension}"));
    let mut n = 1;
    while ruta.exists() {
        ruta = dir.join(format!("{prefijo}{sello}-{n}.{extension}"));
        n += 1;
    }
    ruta
}

#[cfg(test)]
mod tests;
