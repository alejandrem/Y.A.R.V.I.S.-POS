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
            backventanas::backadmin::adminconfig::auth::check_setup_done,
            backventanas::backadmin::adminconfig::auth::guardar_admin,
            backventanas::backadmin::adminconfig::auth::validar_login_admin,
            backventanas::backadmin::adminconfig::auth::get_admin_data,
            backventanas::backadmin::adminconfig::auth::update_admin_data,
            backventanas::backadmin::adminconfig::auth::guardar_empleado,
            backventanas::backadmin::adminconfig::auth::validar_login_empleado,
            // Inventario
            backventanas::backadmin::admininventory::inventory::get_inventory,
            backventanas::backadmin::admininventory::inventory::add_inventory_item,
            backventanas::backadmin::admininventory::inventory::update_inventory_item,
            backventanas::backadmin::admininventory::inventory::delete_inventory_item,
            backventanas::backadmin::admininventory::inventory::importar_catalogo,
            backventanas::backadmin::admininventory::inventory::buscar_producto_similar,
            backventanas::backadmin::admininventory::inventory::get_catalogos_importados,
            backventanas::backadmin::admininventory::inventory::get_productos_por_catalogo,
            // Parser
            backventanas::backadmin::adminparser::leer_archivo_raw,
            backventanas::backadmin::adminparser::leer_archivo_bytes,
            backventanas::backadmin::adminparser::parsear_catalogo_csv,
            backventanas::backadmin::adminparser::parsear_catalogo_visual,
            backventanas::backadmin::adminparser::parsear_excel,
            backventanas::backadmin::adminparser::parsear_ticket,
            backventanas::backadmin::adminparser::analizar_ticket_llm,
            backventanas::backadmin::adminparser::analizar_ticket_con_ia,
            backventanas::backadmin::adminparser::parsear_con_mapeo,
            backventanas::backadmin::adminparser::parsear_carpeta,
            backventanas::backadmin::adminparser::parsear_carpeta_stream,
            backventanas::backadmin::adminparser::parser_commands::vincular_inventario,
            backventanas::backadmin::adminparser::parser_commands::guardar_vinculacion,
            backventanas::backadmin::adminparser::parser_commands::get_db_path,
            backventanas::backadmin::adminparser::parser_commands::descargar_modelos,
            backventanas::backadmin::adminparser::listar_archivos_carpeta,
            // Tickets
            backventanas::backadmin::admintickets::tickets::get_tickets,
            backventanas::backadmin::admintickets::tickets::get_cortes,
            backventanas::backadmin::admintickets::tickets::guardar_ticket_parseado,
            // Empleados - Dashboard
            backventanas::backadmin::adminempleados::empleados::get_empleados,
            backventanas::backadmin::adminempleados::empleados::get_empleado_ventas,
            backventanas::backadmin::adminempleados::empleados::get_resumen_empleados,
            backventanas::backadmin::adminempleados::empleados::get_cortes_empleado,
            // Empleados - Modal empleado
            backventanas::backadmin::adminempleados::modalempleado::update_empleado,
            backventanas::backadmin::adminempleados::modalempleado::delete_empleado,
            // Empleados - Modal metas
            backventanas::backadmin::adminempleados::modalmetas::get_salario_info,
            backventanas::backadmin::adminempleados::modalmetas::save_salario,
            backventanas::backadmin::adminempleados::modalmetas::get_employee_goals,
            backventanas::backadmin::adminempleados::modalmetas::save_employee_goal,
            backventanas::backadmin::adminempleados::modalmetas::save_custom_goal,
            backventanas::backadmin::adminempleados::modalmetas::delete_employee_goal,
            backventanas::backadmin::adminempleados::modalmetas::check_employee_goals,
            // Empleados - Modal turnos
            backventanas::backadmin::adminempleados::modalturnos::get_turnos_empleados,
            // Empleado - Nueva Venta
            backventanas::backempleado::emplea_new_venta::new_venta::completar_venta,
            backventanas::backempleado::emplea_new_venta::new_venta::get_next_ticket_number,
            backventanas::backempleado::emplea_new_venta::new_venta::get_tienda_info,
            // Empleado - Perfil
            backventanas::backempleado::empleaperfil::perfil::get_employee_profile,
            // IA / Sidecar
            backventanas::backadmin::admintarvis::ai::get_ai_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
