// ============================================================
// lib.rs — Setup principal de Tauri
// Inicializa SQLite y lanza el sidecar de Python en background.
// ============================================================

mod backventanas;
mod models;
pub mod sidecar;

use std::sync::Arc;
use tauri::Manager;
use sidecar::AiSidecar;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let ai_sidecar = Arc::new(AiSidecar::new());
    let sidecar_for_setup = Arc::clone(&ai_sidecar);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let (pool, db_path_str) = backventanas::db::db::initialize_db(app.handle());
            app.manage(pool);
            app.manage(backventanas::db::db::DbPath(db_path_str));

            app.manage(Arc::clone(&sidecar_for_setup));

            let sidecar_task = Arc::clone(&sidecar_for_setup);
            tauri::async_runtime::spawn(async move {
                sidecar::launch_ai_engine(sidecar_task).await;
            });

            Ok(())
        })
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
            backventanas::adminconfig::auth::check_setup_done,
            backventanas::adminconfig::auth::guardar_admin,
            backventanas::adminconfig::auth::validar_login_admin,
            backventanas::adminconfig::auth::get_admin_data,
            backventanas::adminconfig::auth::update_admin_data,
            backventanas::adminconfig::auth::guardar_empleado,
            backventanas::adminconfig::auth::validar_login_empleado,
            // Inventario
            backventanas::admininventory::inventory::get_inventory,
            backventanas::admininventory::inventory::add_inventory_item,
            backventanas::admininventory::inventory::update_inventory_item,
            backventanas::admininventory::inventory::delete_inventory_item,
            backventanas::admininventory::inventory::importar_catalogo,
            backventanas::admininventory::inventory::buscar_producto_similar,
            backventanas::admininventory::inventory::get_catalogos_importados,
            backventanas::admininventory::inventory::get_productos_por_catalogo,
            // Parser
            backventanas::adminparser::leer_archivo_raw,
            backventanas::adminparser::leer_archivo_bytes,
            backventanas::adminparser::parsear_catalogo_csv,
            backventanas::adminparser::parsear_catalogo_visual,
            backventanas::adminparser::parsear_excel,
            backventanas::adminparser::parsear_ticket,
            backventanas::adminparser::analizar_ticket_llm,
            backventanas::adminparser::analizar_ticket_con_ia,
            backventanas::adminparser::parsear_con_mapeo,
            backventanas::adminparser::parsear_carpeta,
            backventanas::adminparser::parsear_carpeta_stream,
            backventanas::adminparser::parser_commands::vincular_inventario,
            backventanas::adminparser::parser_commands::guardar_vinculacion,
            backventanas::adminparser::parser_commands::get_db_path,
            backventanas::adminparser::parser_commands::descargar_modelos,
            backventanas::adminparser::listar_archivos_carpeta,
            // Tickets
            backventanas::admintickets::tickets::get_tickets,
            backventanas::admintickets::tickets::get_cortes,
            backventanas::admintickets::tickets::guardar_ticket_parseado,
            // IA / Sidecar
            backventanas::admintarvis::ai::get_ai_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
