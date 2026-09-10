// ============================================================
// lib.rs — Setup principal de Tauri
// Inicializa SQLite. Todo corre 100% nativo en Rust.
// ============================================================

pub mod api_config;
pub mod backventanas;
pub mod dinero;
pub mod models;

use tauri::Manager;

/// El frontend ya montó su primera pantalla: recién ahí se muestra la
/// ventana principal y se cierra el splash. Así el primer frame visible
/// nunca es un render vacío en negro.
static PRINCIPAL_LISTA: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Muestra la ventana principal (con el login ya montado) y cierra el
/// splash nativo. Idempotente: llamadas de más no hacen nada.
#[tauri::command]
fn principal_lista(app: tauri::AppHandle) {
    PRINCIPAL_LISTA.store(true, std::sync::atomic::Ordering::SeqCst);
    mostrar_principal(&app);
}

/// Muestra la principal y cierra el splash (no-op si ya se hizo).
/// La principal se maximiza ANTES de mostrarse: si se muestra a 1200x800
/// y el sistema la maximiza después, se ve el "estirón" al abrir.
fn mostrar_principal(app: &tauri::AppHandle) {
    // Primero se muestra la principal y luego se cierra el splash:
    // así nunca hay un hueco sin ventana visible.
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.maximize();
        let _ = main.show();
        let _ = main.set_focus();
    }
    if let Some(splash) = app.get_webview_window("splash") {
        let _ = splash.close();
    }
}

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

            // El init pesado (backup + SQLite + migraciones + job de
            // alertas) corre en un hilo aparte para no bloquear la
            // pintura de la ventana splash. La ventana principal nace
            // oculta (ver tauri.conf.json) y se muestra solo cuando el
            // backend ya puede responder comandos.
            // NOTA: hilo OS dedicado, NO tokio::spawn: initialize_db usa
            // block_on por dentro y bloquear un worker del runtime es
            // panic seguro.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let (pool, db_path_str) = backventanas::db::db::initialize_db(&handle);

                // Job de fondo de finanzas: cada hora genera alertas automáticas
                // y actualiza el estado de vencimiento de los gastos recurrentes.
                backventanas::backadmin::adminfinanzas::alertas::iniciar_job_alertas(pool.clone());

                handle.manage(pool);
                handle.manage(backventanas::db::db::DbPath(db_path_str.clone()));
                handle.manage(backventanas::auth::AuthState::default());

                // Failsafe: si el frontend muere antes de avisar que montó
                // su primera pantalla, mostrar la principal de todos modos
                // (mostrar_principal es idempotente si ya se hizo).
                let respaldo = handle.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(20));
                    if !PRINCIPAL_LISTA.load(std::sync::atomic::Ordering::SeqCst) {
                        mostrar_principal(&respaldo);
                    }
                });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Ventanas (splash nativo -> principal)
            principal_lista,
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
            backventanas::codigos_barras::get_product_by_barcode,
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
            backventanas::backadmin::adminparser::detectar_mapeo_estadistico,
            backventanas::backadmin::adminparser::parsear_con_mapeo,
            backventanas::backadmin::adminparser::parsear_carpeta,
            backventanas::backadmin::adminparser::parsear_carpeta_stream,
            backventanas::backadmin::adminparser::parser_commands::vincular_inventario,
            backventanas::backadmin::adminparser::parser_commands::guardar_vinculacion,
            backventanas::backadmin::adminparser::parser_commands::get_db_path,
            backventanas::backadmin::adminparser::cortes_import::lectura::previsualizar_corte,
            backventanas::backadmin::adminparser::cortes_import::importacion::importar_carpeta_cortes,
            backventanas::backadmin::adminparser::cortes_import::historial::get_cortes_importados,
            backventanas::backadmin::adminparser::cortes_import::historial::get_corte_importado_detalle,
            backventanas::backadmin::adminparser::parser_commands::descargar_modelos,
            backventanas::backadmin::adminparser::listar_archivos_carpeta,
            // Tickets
            backventanas::backadmin::admintickets::tickets::get_tickets,
            backventanas::backadmin::admintickets::tickets::get_tickets_total,
            backventanas::backadmin::admintickets::tickets::get_cortes,
            backventanas::backadmin::admintickets::tickets::guardar_ticket_parseado,
            backventanas::backadmin::admintickets::tickets::get_predictions,
            backventanas::backadmin::admintickets::tickets::get_ventas_con_pronostico,
            backventanas::backadmin::admintickets::tickets::get_kpis_ventas,
            backventanas::backadmin::admintickets::tickets::get_ventas_por_empleado_dia,
            backventanas::backadmin::admintickets::tickets::get_top_productos,
            // Empleados - Dashboard
            backventanas::backadmin::adminempleados::empleados::get_empleados,
            backventanas::backadmin::adminempleados::empleados::get_empleado_ventas,
            backventanas::backadmin::adminempleados::empleados::get_resumen_empleados,
            backventanas::backadmin::adminempleados::empleados::get_cortes_empleado,
            // Empleados - Modal empleado
            backventanas::backadmin::adminempleados::modalempleado::editar_empleado,
            backventanas::backadmin::adminempleados::modalempleado::set_estado_empleado,
            backventanas::backadmin::adminempleados::modalempleado::aviso_password_defecto,
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
            // Empleado - Mis tickets (operator-scoped por session.user_id)
            backventanas::backempleado::empleatickets::mis_tickets::get_mis_tickets,
            backventanas::backempleado::empleatickets::mis_tickets::get_mis_kpis,
            backventanas::backempleado::empleatickets::mis_tickets::get_mis_ventas_por_dia,
            backventanas::backempleado::empleatickets::mis_tickets::get_mi_ticket_detalle,
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
