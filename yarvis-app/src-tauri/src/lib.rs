// ============================================================
// lib.rs — Setup principal de Tauri
// Inicializa SQLite. Todo corre 100% nativo en Rust.
//
// Registro de comandos: UN solo `generate_handler!` (Tauri lo exige:
// solo acepta rutas literales y consume el Invoke, ver
// backventanas/handlers.rs). Dominios en orden: sistema | auth |
// inventario | parser | tickets | empleados | operacion | finanzas
// | chat. Alta nueva: ver backventanas/handlers.rs (4 pasos).
// ============================================================

pub mod api_config;
pub mod arranque;
pub mod backventanas;
pub mod dinero;
pub mod impresora;
pub mod logs;
pub mod models;
pub mod respaldos;

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
    // Cinturón y tirantes: aunque el frontend avise, la ventana NUNCA
    // se muestra si la DB no abrió. Antes el failsafe la mostraba vacía.
    if !arranque::db_lista() {
        tracing::warn!(
            "[ARRANQUE] el frontend pidió la principal antes de que la base esté lista"
        );
        return;
    }
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
            // Log a archivo (además de consola): en el .exe de Windows no
            // hay consola, y el diálogo de error necesita una ruta real.
            let handle = app.handle().clone();
            let ruta_log = logs::iniciar(&handle);
            tracing::info!("[ARRANQUE] log: {}", ruta_log.display());

            // El init pesado (backup + SQLite + migraciones + jobs) corre
            // en un hilo aparte para no bloquear la pintura del splash.
            // Si la DB falla, arranque muestra un diálogo nativo con el
            // log y la opción de restaurar; la principal NO se muestra.
            arranque::iniciar(handle.clone());

            // Failsafe: solo si el frontend murió antes de avisar que
            // montó su primera pantalla. A diferencia del anterior, este
            // JAMÁS muestra la principal con la DB caída.
            let respaldo = handle.clone();
            std::thread::spawn(move || {
                for _ in 0..40 {
                    if PRINCIPAL_LISTA.load(std::sync::atomic::Ordering::SeqCst) {
                        return;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
                if arranque::db_lista() {
                    mostrar_principal(&respaldo);
                } else if let Some(error) = arranque::ultimo_error() {
                    tracing::error!(
                        "[ARRANQUE] la ventana principal no se mostró: {error}"
                    );
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // ── sistema (handlers.rs §sistema): splash -> principal ──
            principal_lista,
            // ── auth (handlers.rs §auth) ──
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
            // ── inventario (handlers.rs §inventario) ──
            backventanas::backadmin::admininventory::inventory::get_inventory,
            backventanas::backadmin::admininventory::inventory::add_inventory_item,
            backventanas::backadmin::admininventory::inventory::update_inventory_item,
            backventanas::backadmin::admininventory::inventory::delete_inventory_item,
            backventanas::codigos_barras::get_product_by_barcode,
            backventanas::codigos_barras::semaforo_verde::verde_autoasignar,
            backventanas::codigos_barras::semaforo_verde::verde_importar_catalogo,
            backventanas::codigos_barras::semaforo_verde::cargar_catalogo_incluido,
            backventanas::codigos_barras::semaforo_verde::verde_contar_hoy,
            backventanas::codigos_barras::semaforo_rojo::rojo_registrar_pendiente,
            backventanas::codigos_barras::semaforo_rojo::rojo_listar_pendientes,
            backventanas::codigos_barras::semaforo_rojo::rojo_contar_pendientes,
            backventanas::codigos_barras::semaforo_rojo::rojo_resolver_con_alta,
            backventanas::codigos_barras::semaforo_rojo::rojo_resolver_asignando,
            backventanas::codigos_barras::semaforo_amarillo::amarillo_sugerir,
            backventanas::codigos_barras::semaforo_amarillo::amarillo_confirmar,
            backventanas::codigos_barras::semaforo_amarillo::amarillo_rechazar,
            backventanas::codigos_barras::semaforo_amarillo::amarillo_ninguno,
            backventanas::codigos_barras::semaforo_amarillo::amarillo_stats_rechazos,
            backventanas::backadmin::admininventory::inventory::importar_catalogo,
            backventanas::backadmin::admininventory::inventory::buscar_producto_similar,
            backventanas::backadmin::admininventory::inventory::backfill_embeddings,
            backventanas::backadmin::admininventory::inventory::get_catalogos_importados,
            backventanas::backadmin::admininventory::inventory::get_productos_por_catalogo,
            // ── parser (handlers.rs §parser) ──
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
            // ── tickets (handlers.rs §tickets) ──
            backventanas::backadmin::admintickets::tickets::get_tickets,
            backventanas::backadmin::admintickets::tickets::get_tickets_total,
            backventanas::backadmin::admintickets::tickets::get_cortes,
            backventanas::backadmin::admintickets::tickets::guardar_ticket_parseado,
            backventanas::backadmin::admintickets::tickets::get_predictions,
            backventanas::backadmin::admintickets::tickets::get_ventas_con_pronostico,
            backventanas::backadmin::admintickets::tickets::get_kpis_ventas,
            backventanas::backadmin::admintickets::tickets::get_ventas_por_empleado_dia,
            backventanas::backadmin::admintickets::tickets::get_top_productos,
            // ── empleados admin (handlers.rs §empleados): dashboard ──
            backventanas::backadmin::adminempleados::empleados::get_empleados,
            backventanas::backadmin::adminempleados::empleados::get_empleado_ventas,
            backventanas::backadmin::adminempleados::empleados::get_resumen_empleados,
            backventanas::backadmin::adminempleados::empleados::get_cortes_empleado,
            // ── empleados admin: modal empleado ──
            backventanas::backadmin::adminempleados::modalempleado::editar_empleado,
            backventanas::backadmin::adminempleados::modalempleado::set_estado_empleado,
            backventanas::backadmin::adminempleados::modalempleado::aviso_password_defecto,
            // ── empleados admin: modal metas ──
            backventanas::backadmin::adminempleados::modalmetas::get_employee_goals,
            backventanas::backadmin::adminempleados::modalmetas::save_employee_goal,
            backventanas::backadmin::adminempleados::modalmetas::save_custom_goal,
            backventanas::backadmin::adminempleados::modalmetas::delete_employee_goal,
            backventanas::backadmin::adminempleados::modalmetas::check_employee_goals,
            // ── operacion empleado (handlers.rs §operacion): nueva venta ──
            backventanas::backempleado::emplea_new_venta::new_venta::completar_venta,
            backventanas::backempleado::emplea_new_venta::new_venta::get_next_ticket_number,
            backventanas::backempleado::emplea_new_venta::new_venta::get_tienda_info,
            // ── operacion empleado: proveedores (operator-scoped) ──
            backventanas::backempleado::empleaproveedores::proveedores::guardar_proveedor,
            backventanas::backempleado::empleaproveedores::proveedores::crear_proveedor_generico,
            backventanas::backempleado::empleaproveedores::proveedores::listar_proveedores,
            backventanas::backempleado::empleaproveedores::sugerencia::sugerir_pago,
            backventanas::backempleado::empleaproveedores::compras::registrar_compra,
            backventanas::backempleado::empleaproveedores::compras::rectificar_compra,
            backventanas::backempleado::empleaproveedores::compras::historial_compras,
            backventanas::backempleado::empleaproveedores::compras::get_compra_detalle,
            // ── operacion empleado: perfil + hora del sistema ──
            backventanas::backempleado::empleaperfil::perfil::get_employee_profile,
            backventanas::backempleado::empleaperfil::asistencia::get_mi_turno,
            backventanas::backempleado::empleaperfil::asistencia::get_asistencia_empleado,
            backventanas::backempleado::empleaperfil::asistencia::get_mis_horas_extra,
            backventanas::backempleado::empleaperfil::asistencia::get_horas_extra_empleado,
            // Hora del sistema (reloj de Windows vs red, aviso de desfase)
            backventanas::hora::verificar_hora_sistema,
            // ── operacion empleado: mis tickets (operator-scoped) ──
            backventanas::backempleado::empleatickets::mis_tickets::get_mis_tickets,
            backventanas::backempleado::empleatickets::mis_tickets::get_mis_kpis,
            backventanas::backempleado::empleatickets::mis_tickets::get_mis_ventas_por_dia,
            backventanas::backempleado::empleatickets::mis_tickets::get_mi_ticket_detalle,
            backventanas::backempleado::empleatickets::mis_cortes::get_mis_cortes,
            // ── finanzas (handlers.rs §finanzas): gastos ──
            backventanas::backadmin::adminfinanzas::gastos::get_gastos_recurrentes,
            backventanas::backadmin::adminfinanzas::gastos::crear_gasto,
            backventanas::backadmin::adminfinanzas::gastos::actualizar_gasto,
            backventanas::backadmin::adminfinanzas::gastos::eliminar_gasto,
            backventanas::backadmin::adminfinanzas::gastos::registrar_pago_gasto,
            backventanas::backadmin::adminfinanzas::gastos::get_pagos_gasto,
            backventanas::backadmin::adminfinanzas::gastos::get_proximos_vencimientos,
            backventanas::backadmin::adminfinanzas::gastos::actualizar_estados_gastos,
            // ── finanzas: cortes X/Z ──
            backventanas::backadmin::adminfinanzas::cortes::get_cortes_caja,
            backventanas::backadmin::adminfinanzas::cortes::get_corte_detalle,
            backventanas::backadmin::adminfinanzas::cortes::crear_corte_x,
            backventanas::backadmin::adminfinanzas::cortes::crear_corte_z,
            backventanas::backadmin::adminfinanzas::cortes::cerrar_corte,
            backventanas::backadmin::adminfinanzas::cortes::agregar_movimiento_caja,
            backventanas::backadmin::adminfinanzas::cortes::get_movimientos_corte,
            backventanas::backadmin::adminfinanzas::cortes::get_cortes_por_cajero_fecha,
            // ── finanzas: cortes X/Z diferenciados (backcortes admin+empleado) ──
            backventanas::backcortes::corte_x::corte_x_reporte,
            backventanas::backcortes::corte_z::corte_z_cierre,
            backventanas::backcortes::impresion::imprimir_corte,
            // ── finanzas: métricas y utilidades ──
            backventanas::backadmin::adminfinanzas::metricas::get_metricas_diarias,
            backventanas::backadmin::adminfinanzas::metricas::get_resumen_periodo,
            backventanas::backadmin::adminfinanzas::metricas::recalcular_resumen_diario,
            backventanas::backadmin::adminfinanzas::metricas::get_punto_equilibrio,
            // ── finanzas: gráficas ──
            backventanas::backadmin::adminfinanzas::graficas::get_datos_grafica_pl,
            backventanas::backadmin::adminfinanzas::graficas::get_gastos_por_categoria,
            backventanas::backadmin::adminfinanzas::graficas::get_tendencia_cortes_z,
            backventanas::backadmin::adminfinanzas::graficas::get_ventas_vs_gastos_mensual,
            backventanas::backadmin::adminfinanzas::graficas::get_predicciones_financieras,
            // ── finanzas: alertas ──
            backventanas::backadmin::adminfinanzas::alertas::get_alertas,
            backventanas::backadmin::adminfinanzas::alertas::marcar_alerta_leida,
            backventanas::backadmin::adminfinanzas::alertas::generar_alertas_automaticas,
            // ── finanzas: export ──
            backventanas::backadmin::adminfinanzas::export::exportar_balance_pdf,
            backventanas::backadmin::adminfinanzas::export::exportar_gastos_csv,
            // ── chat (handlers.rs §chat): local Qwen + nube ──
            backventanas::backadmin::admintarvis::chat::send_chat_message,
            backventanas::backadmin::admintarvis::chat::send_chat_stream,
            backventanas::backadmin::admintarvis::chat::get_cloud_models,
            backventanas::backadmin::admintarvis::chat::stop_chat_stream,
            backventanas::backadmin::admintarvis::chat::get_model_status,
            backventanas::backadmin::admintarvis::chat::set_local_model_path,
            backventanas::backadmin::admintarvis::chat::load_chat_model,
            backventanas::backadmin::admintarvis::chat::unload_chat_model,
            // ── respaldos cifrados programados (respaldos/comandos.rs) ──
            respaldos::comandos::respaldo_config_leer,
            respaldos::comandos::respaldo_config_guardar,
            respaldos::comandos::respaldo_crear_ahora,
            respaldos::comandos::respaldo_listar,
            respaldos::comandos::respaldo_probar,
            respaldos::comandos::respaldo_restaurar,
            // ── sistema: impresora térmica A (spooler Windows RAW) ──
            impresora::commands::listar_impresoras,
            impresora::commands::imprimir_bytes_raw,
            impresora::commands::imprimir_lista_conciliacion,
            impresora::commands::imprimir_lista_stock_bajo,
            // ── sistema: impresora térmica C (ticket `escpos` + red) ──
            impresora::commands::imprimir_ticket_venta,
            impresora::commands::probar_red,
            impresora::commands::abrir_cajon,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
