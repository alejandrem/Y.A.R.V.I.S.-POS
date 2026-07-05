// ============================================================
// lib.rs — Setup principal de Tauri
// Inicializa SQLite y lanza el sidecar de Python en background.
// ============================================================

mod commands;
mod db;
mod models;
pub mod sidecar;
pub mod parser_rs;

use std::sync::Arc;
use tauri::Manager;
use sidecar::AiSidecar;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Crear el sidecar compartido ANTES del setup
    // Arc permite compartirlo entre el closure de setup y la tarea async
    let ai_sidecar = Arc::new(AiSidecar::new());
    let sidecar_for_setup = Arc::clone(&ai_sidecar);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            // ── 1. Inicializar SQLite ──────────────────────────────
            let (pool, db_path_str) = db::initialize_db(app.handle());
            app.manage(pool);
            app.manage(db::DbPath(db_path_str));

            // ── 2. Registrar el AiSidecar como estado global ───────
            // Esto permite que los commands de Tauri lo accedan via State<>
            app.manage(Arc::clone(&sidecar_for_setup));

            // ── 3. Lanzar el motor de Python en background ─────────
            // Usamos spawn para no bloquear el hilo principal de la UI.
            // Si Python falla, la app sigue funcionando en modo clásico.
            let sidecar_task = Arc::clone(&sidecar_for_setup);
            tauri::async_runtime::spawn(async move {
                sidecar::launch_ai_engine(sidecar_task).await;
            });

            Ok(())
        })
        // ── 4. Cleanup al cerrar: matar el proceso Python ──────────
        .on_window_event({
            let sidecar_cleanup = Arc::clone(&ai_sidecar);
            move |_window, event| {
                if let tauri::WindowEvent::Destroyed = event {
                    sidecar::shutdown_ai_engine(&sidecar_cleanup);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Auth
            commands::auth::check_setup_done,
            commands::auth::guardar_admin,
            commands::auth::validar_login_admin,
            commands::auth::get_admin_data,
            commands::auth::update_admin_data,
            commands::auth::guardar_empleado,
            commands::auth::validar_login_empleado,
            // Inventario
            commands::inventory::get_inventory,
            commands::inventory::add_inventory_item,
            commands::inventory::update_inventory_item,
            commands::inventory::delete_inventory_item,
            commands::inventory::importar_catalogo,
            commands::inventory::buscar_producto_similar,
            // Parser (nueva estructura)
            parser_rs::leer_archivo_raw,
            parser_rs::leer_archivo_bytes,
            parser_rs::parsear_catalogo,
            parser_rs::parsear_catalogo_csv,
            parser_rs::parsear_catalogo_visual,
            parser_rs::parsear_excel,
            parser_rs::parsear_ticket,
            parser_rs::analizar_ticket_llm,
            parser_rs::analizar_ticket_con_ia,
            parser_rs::parsear_con_mapeo,
            parser_rs::parsear_carpeta,
            parser_rs::parsear_carpeta_stream,
            commands::parser::vincular_inventario,
            commands::parser::guardar_vinculacion,
            commands::parser::get_db_path,
            commands::parser::descargar_modelos,
            // Tickets
            commands::tickets::get_tickets,
            commands::tickets::get_cortes,
            commands::tickets::guardar_ticket_parseado,
            // IA / Sidecar
            commands::ai::get_ai_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
