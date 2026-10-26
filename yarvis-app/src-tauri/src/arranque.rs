// ============================================================
// arranque — Orquestación del inicio de la caja.
//
// Antes: un fallo de DB ejecutaba panic! dentro del hilo de arranque.
// El proceso seguía vivo, el hilo moría antes de registrar el pool y a
// los 20 s aparecía la ventana principal VACÍA. En una tienda eso es
// "se descompuso la caja" sin explicación.
//
// Ahora: initialize_db devuelve Result, aquí se registra el error, se
// escribe en el log y se muestra un diálogo NATIVO con la ruta del log
// y un botón para restaurar un respaldo local. La ventana principal no
// se muestra hasta que DB_LISTA sea true.
// ============================================================

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::backventanas::auth::AuthState;
use crate::backventanas::db::db::{self, DbPath};
use crate::backventanas::db::{respaldo, ErrorDb};

/// La base abrió, migró y ya está registrada como State.
static DB_LISTA: AtomicBool = AtomicBool::new(false);

/// Último error de arranque, para que el failsafe lo deje en el log.
static ULTIMO_ERROR: Mutex<Option<String>> = Mutex::new(None);

pub fn db_lista() -> bool {
    DB_LISTA.load(Ordering::SeqCst)
}

pub fn ultimo_error() -> Option<String> {
    ULTIMO_ERROR.lock().ok().and_then(|guard| guard.clone())
}

fn guardar_error(mensaje: &str) {
    if let Ok(mut guard) = ULTIMO_ERROR.lock() {
        *guard = Some(mensaje.to_string());
    }
}

/// Arranca la DB en un hilo OS dedicado. Si falla, atiende el error con
/// un diálogo nativo y NO deja que la ventana principal se muestre.
pub fn iniciar(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        // Migrar API keys en claro antes de que el usuario abra el panel.
        crate::api_config::migrar_almacen(&app);

        let ruta_log = crate::logs::ruta_actual()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(log no disponible)".to_string());

        let (pool, db_path_str) = match db::initialize_db(&app) {
            Ok(valor) => valor,
            Err(error) => {
                tracing::error!("[ARRANQUE] no se pudo iniciar la base de datos: {error}");
                guardar_error(&error.mensaje());
                atender_error(&app, &error, &ruta_log);
                return;
            }
        };

        // Limpia .db en claro que haya dejado un cierre abrupto del
        // respaldo cifrado (carpeta local, nunca el USB).
        if let Ok(dir) = app.path().app_data_dir() {
            crate::respaldos::archivo::limpiar_temporales(&dir.join("tmp"));
        }

        backventanas_alerta(pool.clone());
        let pool_respaldos = pool.clone();

        app.manage(pool);
        app.manage(DbPath(db_path_str));
        app.manage(AuthState::default());

        DB_LISTA.store(true, Ordering::SeqCst);
        tracing::info!("[ARRANQUE] base de datos lista");

        crate::respaldos::programador::iniciar(app.clone(), pool_respaldos);
    });
}

fn backventanas_alerta(pool: sqlx::SqlitePool) {
    crate::backventanas::backadmin::adminfinanzas::alertas::iniciar_job_alertas(pool);
}

/// Diálogo nativo de fallo de arranque. Ofrece restaurar un respaldo
/// local en claro (.db). Los .yarvisbak cifrados se restauran desde el
/// wizard de Configuración cuando la caja ya abrió.
fn atender_error(app: &tauri::AppHandle, error: &ErrorDb, ruta_log: &str) {
    let db_path = db::ruta_db(app)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "(ruta desconocida)".to_string());
    let backups = db::ruta_db(app)
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("backups")));

    let mut mensaje = mensaje_error(
        error,
        &db_path,
        &backups
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(no disponible)".to_string()),
        ruta_log,
    );

    loop {
        let quiere_restaurar = app
            .dialog()
            .message(mensaje.clone())
            .title("Y.A.R.V.I.S. POS no pudo iniciar")
            .kind(MessageDialogKind::Error)
            .buttons(MessageDialogButtons::OkCancelCustom(
                "Restaurar respaldo".to_string(),
                "Cerrar la app".to_string(),
            ))
            .blocking_show();

        if !quiere_restaurar {
            tracing::error!("[ARRANQUE] el usuario cerró la app sin restaurar");
            app.exit(1);
            return;
        }

        let mut dialogo = app
            .dialog()
            .file()
            .set_title("Elige un respaldo local (.db)")
            .add_filter("Base de datos YARVIS", &["db"]);
        if let Some(dir) = backups.as_deref().filter(|d| d.is_dir()) {
            dialogo = dialogo.set_directory(dir);
        }
        let Some(elegido) = dialogo.blocking_pick_file() else {
            continue;
        };
        let Ok(origen) = elegido.into_path() else {
            mensaje = format!(
                "No se pudo leer la ruta del archivo elegido.\n\nLog: {ruta_log}"
            );
            continue;
        };

        match restaurar_desde_arranque(app, &origen) {
            Ok(seguridad) => {
                tracing::info!(
                    "[ARRANQUE] restaurado {}; copia previa: {}. Reiniciando app.",
                    origen.display(),
                    seguridad
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "(no existía base)".to_string())
                );
                app.restart();
            }
            Err(e) => {
                tracing::error!(
                    "[ARRANQUE] falló la restauración desde {}: {e}",
                    origen.display()
                );
                mensaje = format!(
                    "No se pudo restaurar el respaldo.\n\n{e}\n\n\
                     La base actual quedó intacta.\n\nLog: {ruta_log}"
                );
            }
        }
    }
}

fn restaurar_desde_arranque(
    app: &tauri::AppHandle,
    origen: &Path,
) -> Result<Option<PathBuf>, String> {
    let db_path = db::ruta_db(app).map_err(|e| e.to_string())?;
    validar_origen(origen, &db_path)?;
    let app_dir = db_path
        .parent()
        .ok_or_else(|| "La ruta de la base de datos no tiene carpeta padre".to_string())?;
    respaldo::restaurar_db_plana(origen, &db_path, app_dir)
}

/// Reglas del respaldo de arranque: solo .db y nunca el archivo actual.
fn validar_origen(origen: &Path, db_path: &Path) -> Result<(), String> {
    if origen
        .extension()
        .is_none_or(|ext| !ext.eq_ignore_ascii_case("db"))
    {
        return Err(
            "Selecciona un respaldo local .db. Los .yarvisbak están cifrados y se \
             restauran desde el asistente de Configuración."
                .to_string(),
        );
    }
    let misma = match (origen.canonicalize(), db_path.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => origen == db_path,
    };
    if misma {
        return Err("Ese es el archivo que no pudo abrir; elige un respaldo anterior".to_string());
    }
    Ok(())
}

fn mensaje_error(error: &ErrorDb, db_path: &str, backups: &str, ruta_log: &str) -> String {
    format!(
        "{}\n\n{}\n\nQué hacer: {}\n\n\
         Base de datos: {db_path}\n\
         Respaldos locales: {backups}\n\
         Log: {ruta_log}\n\n\
         También puedes restaurar un respaldo cifrado (.yarvisbak) desde \
         Configuración una vez que la caja abra.",
        error.resumen, error.detalle, error.pista
    )
}

#[cfg(test)]
mod tests;
