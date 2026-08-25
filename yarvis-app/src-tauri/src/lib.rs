// ============================================================
// lib.rs — Setup principal de Tauri
// Inicializa SQLite. Todo corre 100% nativo en Rust.
// ============================================================

pub mod api_config;
pub mod backventanas;
pub mod dinero;
pub mod models;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            // Logging estructurado: respeta RUST_LOG, default "info"
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
                )
                .init();

            let (pool, db_path_str) = backventanas::db::db::initialize_db(app.handle());

            // Job de fondo de finanzas: cada hora genera alertas automáticas
            // y actualiza el estado de vencimiento de los gastos recurrentes.
            backventanas::backadmin::adminfinanzas::alertas::iniciar_job_alertas(pool.clone());

            app.manage(pool);
            app.manage(backventanas::db::db::DbPath(db_path_str.clone()));
            app.manage(backventanas::auth::AuthState::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth
            api_config::guardar_api_keys,
            api_config::leer_api_keys,
            backventanas::backadmin::adminconfig::auth::check_setup_done,
            backventanas::backadmin::adminconfig::auth::guardar_admin,
            backventanas::backadmin::adminconfig::auth::validar_login_admin,
            backventanas::backadmin::adminconfig::auth::get_admin_data,
            backventanas::backadmin::adminconfig::auth::update_admin_data,
            backventanas::backadmin::adminconfig::auth::guardar_empleado,
            backventanas::backadmin::adminconfig::auth::validar_login_empleado,
            backventanas::backadmin::adminconfig::auth::cerrar_sesion,
            backventanas::backadmin::adminconfig::google::login_con_google,
            // Inventario
            backventanas::backadmin::admininventory::inventory::get_inventory,
            backventanas::backadmin::admininventory::inventory::add_inventory_item,
            backventanas::backadmin::admininventory::inventory::update_inventory_item,
            backventanas::backadmin::admininventory::inventory::delete_inventory_item,
            backventanas::backadmin::admininventory::inventory::importar_catalogo,
            backventanas::backadmin::admininventory::inventory::buscar_producto_similar,
            backventanas::backadmin::admininventory::inventory::backfill_embeddings,
            backventanas::backadmin::admininventory::inventory::get_catalogos_importados,
            backventanas::backadmin::admininventory::inventory::get_productos_por_catalogo,
            // Parser
            backventanas::backadmin::adminparser::leer_archivo_raw,
            backventanas::backadmin::adminparser::leer_archivo_bytes,
            backventanas::backadmin::adminparser::parsear_catalogo_csv,
            backventanas::backadmin::adminparser::parsear_catalogo_visual,
            backventanas::backadmin::adminparser::parsear_excel,
            backventanas::backadmin::adminparser::analizar_ticket_llm,
            backventanas::backadmin::adminparser::analizar_ticket_con_ia,
            backventanas::backadmin::adminparser::analizar_muestras_carpeta,
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
            backventanas::backadmin::admintickets::tickets::get_predictions,
            // Empleados - Dashboard
            backventanas::backadmin::adminempleados::empleados::get_empleados,
            backventanas::backadmin::adminempleados::empleados::get_empleado_ventas,
            backventanas::backadmin::adminempleados::empleados::get_resumen_empleados,
            backventanas::backadmin::adminempleados::empleados::get_cortes_empleado,
            // Empleados - Modal empleado
            backventanas::backadmin::adminempleados::modalempleado::editar_empleado,
            backventanas::backadmin::adminempleados::modalempleado::set_estado_empleado,
            // Empleados - Modal metas
            backventanas::backadmin::adminempleados::modalmetas::get_employee_goals,
            backventanas::backadmin::adminempleados::modalmetas::save_employee_goal,
            backventanas::backadmin::adminempleados::modalmetas::save_custom_goal,
            backventanas::backadmin::adminempleados::modalmetas::delete_employee_goal,
            backventanas::backadmin::adminempleados::modalmetas::check_employee_goals,
            // Empleado - Nueva Venta
            backventanas::backempleado::emplea_new_venta::new_venta::completar_venta,
            backventanas::backempleado::emplea_new_venta::new_venta::get_next_ticket_number,
            backventanas::backempleado::emplea_new_venta::new_venta::get_tienda_info,
            // Empleado - Perfil
            backventanas::backempleado::empleaperfil::perfil::get_employee_profile,
            backventanas::backempleado::empleaperfil::asistencia::get_mi_turno,
            backventanas::backempleado::empleaperfil::asistencia::get_asistencia_empleado,
            backventanas::backempleado::empleaperfil::asistencia::get_mis_horas_extra,
            backventanas::backempleado::empleaperfil::asistencia::get_horas_extra_empleado,
            // Finanzas - Gastos
            backventanas::backadmin::adminfinanzas::gastos::get_gastos_recurrentes,
            backventanas::backadmin::adminfinanzas::gastos::crear_gasto,
            backventanas::backadmin::adminfinanzas::gastos::actualizar_gasto,
            backventanas::backadmin::adminfinanzas::gastos::eliminar_gasto,
            backventanas::backadmin::adminfinanzas::gastos::registrar_pago_gasto,
            backventanas::backadmin::adminfinanzas::gastos::get_pagos_gasto,
            backventanas::backadmin::adminfinanzas::gastos::get_proximos_vencimientos,
            backventanas::backadmin::adminfinanzas::gastos::actualizar_estados_gastos,
            // Finanzas - Cortes X/Z
            backventanas::backadmin::adminfinanzas::cortes::get_cortes_caja,
            backventanas::backadmin::adminfinanzas::cortes::get_corte_detalle,
            backventanas::backadmin::adminfinanzas::cortes::crear_corte_x,
            backventanas::backadmin::adminfinanzas::cortes::crear_corte_z,
            backventanas::backadmin::adminfinanzas::cortes::cerrar_corte,
            backventanas::backadmin::adminfinanzas::cortes::agregar_movimiento_caja,
            backventanas::backadmin::adminfinanzas::cortes::get_movimientos_corte,
            backventanas::backadmin::adminfinanzas::cortes::get_cortes_por_cajero_fecha,
            // Finanzas - Métricas y Utilidades
            backventanas::backadmin::adminfinanzas::metricas::get_metricas_diarias,
            backventanas::backadmin::adminfinanzas::metricas::get_resumen_periodo,
            backventanas::backadmin::adminfinanzas::metricas::recalcular_resumen_diario,
            backventanas::backadmin::adminfinanzas::metricas::get_punto_equilibrio,
            // Finanzas - Gráficas
            backventanas::backadmin::adminfinanzas::graficas::get_datos_grafica_pl,
            backventanas::backadmin::adminfinanzas::graficas::get_gastos_por_categoria,
            backventanas::backadmin::adminfinanzas::graficas::get_tendencia_cortes_z,
            backventanas::backadmin::adminfinanzas::graficas::get_ventas_vs_gastos_mensual,
            backventanas::backadmin::adminfinanzas::graficas::get_predicciones_financieras,
            // Finanzas - Alertas
            backventanas::backadmin::adminfinanzas::alertas::get_alertas,
            backventanas::backadmin::adminfinanzas::alertas::marcar_alerta_leida,
            backventanas::backadmin::adminfinanzas::alertas::generar_alertas_automaticas,
            // Finanzas - Export (TODO)
            backventanas::backadmin::adminfinanzas::export::exportar_balance_pdf,
            backventanas::backadmin::adminfinanzas::export::exportar_gastos_csv,
            // Chat (local Qwen 1.7B + nube)
            backventanas::backadmin::admintarvis::chat::send_chat_message,
            backventanas::backadmin::admintarvis::chat::send_chat_stream,
            backventanas::backadmin::admintarvis::chat::get_cloud_models,
            backventanas::backadmin::admintarvis::chat::stop_chat_stream,
            backventanas::backadmin::admintarvis::chat::get_model_status,
            backventanas::backadmin::admintarvis::chat::set_local_model_path,
            backventanas::backadmin::admintarvis::chat::load_chat_model,
            backventanas::backadmin::admintarvis::chat::unload_chat_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
