//base de datos de yarvis (tablas solamente)
use std::fs;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tauri::Manager;

/// Estado simple para exponer la ruta de la DB al frontend.
pub struct DbPath(pub String);

pub fn initialize_db(app: &tauri::AppHandle) -> (SqlitePool, String) {
    let app_dir = app.path().app_data_dir().expect("No se pudo obtener el directorio de datos");
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).expect("No se pudo crear el directorio de datos");
    }

    let db_path = app_dir.join("yarvis.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    tauri::async_runtime::block_on(async move {
        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await.expect("Fallo al conectar a SQLite");

        // ========================
        // ACTIVAR MODO WAL
        // ========================
        sqlx::query("PRAGMA journal_mode=WAL;")
            .execute(&pool)
            .await
            .expect("Fallo al activar modo WAL");

        // ========================
        // TABLA: usuarios
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS usuarios (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            tienda TEXT,
            password TEXT NOT NULL,
            rol TEXT NOT NULL,
            ubicacion TEXT,
            cp TEXT,
            salario_semanal REAL DEFAULT 0,
            turno TEXT DEFAULT 'matutino',
            horario_inicio TEXT DEFAULT '08:00',
            horario_fin TEXT DEFAULT '14:00',
            meta_mensual REAL DEFAULT 0,
            bono REAL DEFAULT 0,
            estado TEXT DEFAULT 'activo',
            registrado_en DATETIME DEFAULT '2000-01-01 00:00:00',
            ultimo_login DATETIME
        )").execute(&pool).await.expect("Fallo al crear tabla de usuarios");

        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN ubicacion TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN cp TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN salario_semanal REAL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN turno TEXT DEFAULT 'matutino'").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN horario_inicio TEXT DEFAULT '08:00'").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN horario_fin TEXT DEFAULT '14:00'").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN meta_mensual REAL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN bono REAL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN estado TEXT DEFAULT 'activo'").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN registrado_en DATETIME DEFAULT '2000-01-01 00:00:00'").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN ultimo_login DATETIME").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN salario_diario REAL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN dias_semana INTEGER DEFAULT 6").execute(&pool).await;

        // ========================
        // TABLA: employee_goals
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS employee_goals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            employee_id INTEGER NOT NULL,
            goal_type TEXT NOT NULL,
            goal_name TEXT,
            ventas_threshold TEXT DEFAULT '5',
            bonus_percentage REAL DEFAULT 0,
            bonus_amount REAL DEFAULT 0,
            is_completed INTEGER DEFAULT 0,
            completed_at TEXT,
            created_at TEXT DEFAULT (datetime('now','localtime')),
            FOREIGN KEY (employee_id) REFERENCES usuarios(id)
        )").execute(&pool).await.expect("Fallo al crear tabla employee_goals");

        // ========================
        // TABLA: productos
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS productos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            descripcion TEXT,
            precio_costo REAL,
            precio_venta REAL,
            stock REAL DEFAULT 0,
            stock_minimo REAL DEFAULT 0,
            codigo_barras TEXT UNIQUE,
            categoria TEXT,
            creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
        )").execute(&pool).await.expect("Fallo al crear tabla de productos");

        // Migraciones para columnas faltantes en productos
        let _ = sqlx::query("ALTER TABLE productos ADD COLUMN descripcion TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE productos ADD COLUMN categoria TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE productos ADD COLUMN codigo_barras TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE productos ADD COLUMN stock_minimo REAL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE productos ADD COLUMN vendido REAL DEFAULT 0").execute(&pool).await;

        // ========================
        // TABLA: clientes
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS clientes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            rfc TEXT UNIQUE,
            email TEXT,
            telefono TEXT,
            direccion TEXT,
            credito_limite REAL DEFAULT 0,
            notas TEXT,
            creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
        )").execute(&pool).await.expect("Fallo al crear tabla de clientes");

        // ========================
        // TABLA: ventas
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS ventas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fecha DATETIME DEFAULT CURRENT_TIMESTAMP,
            total REAL NOT NULL,
            subtotal REAL,
            iva REAL,
            descuento REAL DEFAULT 0,
            metodo_pago TEXT DEFAULT 'efectivo',
            cajero TEXT NOT NULL,
            cliente_id INTEGER,
            clima TEXT,
            estado TEXT DEFAULT 'completada',
            FOREIGN KEY (cliente_id) REFERENCES clientes(id)
        )").execute(&pool).await.expect("Fallo al crear tabla de ventas");

        // ========================
        // TABLA: detalle_ventas
        // ========================
        // FIX: producto_id es nullable (None para productos no linkeados aun)
        sqlx::query("CREATE TABLE IF NOT EXISTS detalle_ventas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            venta_id INTEGER NOT NULL,
            producto_id INTEGER,
            producto_nombre TEXT NOT NULL,
            cantidad REAL NOT NULL,
            precio_unitario REAL NOT NULL,
            descuento REAL DEFAULT 0,
            subtotal REAL NOT NULL,
            FOREIGN KEY (venta_id) REFERENCES ventas(id) ON DELETE CASCADE,
            FOREIGN KEY (producto_id) REFERENCES productos(id)
        )").execute(&pool).await.expect("Fallo al crear tabla detalle_ventas");

        // ========================
        // TABLA: ventas_diarias (historial con clima)
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS ventas_diarias (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fecha DATE NOT NULL UNIQUE,
            total_ventas REAL DEFAULT 0,
            cantidad_tickets INTEGER DEFAULT 0,
            temperatura_promedio REAL,
            clima TEXT,
            utilidad_bruta REAL,
            utilidad_operativa REAL,
            utilidad_neta REAL
        )").execute(&pool).await.expect("Fallo al crear tabla ventas_diarias");

        // ========================
        // TABLA: cortes_caja
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS cortes_caja (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fecha_apertura DATETIME DEFAULT CURRENT_TIMESTAMP,
            fecha_cierre DATETIME,
            monto_inicial REAL DEFAULT 0,
            total_ventas REAL DEFAULT 0,
            total_efectivo REAL DEFAULT 0,
            total_tarjeta REAL DEFAULT 0,
            total_transferencia REAL DEFAULT 0,
            diferencia REAL DEFAULT 0,
            usuario_id INTEGER,
            estado TEXT DEFAULT 'abierto',
            FOREIGN KEY (usuario_id) REFERENCES usuarios(id)
        )").execute(&pool).await.expect("Fallo al crear tabla cortes_caja");

        // ========================
        // TABLA: predicciones_futuras
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS predicciones_futuras (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fecha_prediccion DATE NOT NULL,
            producto TEXT,
            cantidad_sugerida REAL,
            margen_error REAL,
            confianza REAL,
            generado_en DATETIME DEFAULT CURRENT_TIMESTAMP,
            notas TEXT
        )").execute(&pool).await.expect("Fallo al crear tabla predicciones_futuras");

        // ========================
        // TABLA: knowledge_base (sqlite-vec placeholder)
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS knowledge_base (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            contenido TEXT NOT NULL,
            categoria TEXT NOT NULL,
            embedding BLOB,
            creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
        )").execute(&pool).await.expect("Fallo al crear tabla knowledge_base");

        // Migraciones para pagos parciales en ventas
        let _ = sqlx::query("ALTER TABLE ventas ADD COLUMN monto_efectivo REAL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE ventas ADD COLUMN monto_tarjeta REAL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE ventas ADD COLUMN monto_transferencia REAL DEFAULT 0").execute(&pool).await;

        // ========================
        // TABLA: catalogos_importados (control de duplicados)
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS catalogos_importados (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            hash TEXT UNIQUE NOT NULL,
            ruta_archivo TEXT,
            fecha_importacion DATETIME DEFAULT CURRENT_TIMESTAMP,
            total_productos INTEGER DEFAULT 0
        )").execute(&pool).await.expect("Fallo al crear tabla catalogos_importados");

        // ========================
        // TABLA: gastos_recurrentes (Módulo Finanzas)
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS gastos_recurrentes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            tipo TEXT NOT NULL,
            categoria TEXT NOT NULL,
            monto_proyectado REAL NOT NULL,
            monto_real REAL DEFAULT 0,
            frecuencia TEXT NOT NULL,
            dia_pago INTEGER,
            intervalo_dias INTEGER,
            fecha_inicio DATE NOT NULL,
            fecha_fin DATE,
            estado_pago TEXT DEFAULT 'pendiente',
            folio_comprobante TEXT,
            comprobante_url TEXT,
            notas TEXT,
            creado_en DATETIME DEFAULT CURRENT_TIMESTAMP,
            actualizado_en DATETIME DEFAULT CURRENT_TIMESTAMP
        )").execute(&pool).await.expect("Fallo al crear tabla gastos_recurrentes");

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_gastos_fecha_inicio ON gastos_recurrentes(fecha_inicio)").execute(&pool).await;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_gastos_estado ON gastos_recurrentes(estado_pago)").execute(&pool).await;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_gastos_tipo ON gastos_recurrentes(tipo)").execute(&pool).await;

        // ========================
        // TABLA: pagos_gastos (Historial de pagos)
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS pagos_gastos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            gasto_id INTEGER NOT NULL,
            fecha_pago DATETIME NOT NULL,
            monto_pagado REAL NOT NULL,
            metodo_pago TEXT,
            folio_comprobante TEXT,
            comprobante_url TEXT,
            notas TEXT,
            creado_en DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (gasto_id) REFERENCES gastos_recurrentes(id) ON DELETE CASCADE
        )").execute(&pool).await.expect("Fallo al crear tabla pagos_gastos");

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_pagos_gasto_id ON pagos_gastos(gasto_id)").execute(&pool).await;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_pagos_fecha ON pagos_gastos(fecha_pago)").execute(&pool).await;

        // ========================
        // MIGRACIONES: cortes_caja (extender para cortes X/Z)
        // ========================
        let _ = sqlx::query("ALTER TABLE cortes_caja ADD COLUMN tipo_corte TEXT DEFAULT 'Z'").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE cortes_caja ADD COLUMN turno TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE cortes_caja ADD COLUMN entradas_manuales REAL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE cortes_caja ADD COLUMN retiros_manuales REAL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE cortes_caja ADD COLUMN observaciones TEXT").execute(&pool).await;

        // ========================
        // TABLA: movimientos_caja (Detalle de entradas/salidas en corte)
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS movimientos_caja (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            corte_id INTEGER NOT NULL,
            tipo TEXT NOT NULL,
            concepto TEXT NOT NULL,
            monto REAL NOT NULL,
            metodo_pago TEXT,
            referencia_id INTEGER,
            creado_en DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (corte_id) REFERENCES cortes_caja(id) ON DELETE CASCADE
        )").execute(&pool).await.expect("Fallo al crear tabla movimientos_caja");

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_movimientos_corte ON movimientos_caja(corte_id)").execute(&pool).await;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_movimientos_tipo ON movimientos_caja(tipo)").execute(&pool).await;

        // ========================
        // TABLA: resumen_financiero_diario (Materialized view para gráficas P&L)
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS resumen_financiero_diario (
            fecha DATE PRIMARY KEY,
            ventas_totales REAL DEFAULT 0,
            ventas_efectivo REAL DEFAULT 0,
            ventas_tarjeta REAL DEFAULT 0,
            ventas_transferencia REAL DEFAULT 0,
            costo_ventas REAL DEFAULT 0,
            utilidad_bruta REAL DEFAULT 0,
            gastos_operativos REAL DEFAULT 0,
            utilidad_operativa REAL DEFAULT 0,
            impuestos_comisiones REAL DEFAULT 0,
            utilidad_neta REAL DEFAULT 0,
            margen_neto_pct REAL DEFAULT 0,
            cortes_z_count INTEGER DEFAULT 0,
            diferencia_caja_total REAL DEFAULT 0,
            actualizado_en DATETIME DEFAULT CURRENT_TIMESTAMP
        )").execute(&pool).await.expect("Fallo al crear tabla resumen_financiero_diario");

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_resumen_fecha ON resumen_financiero_diario(fecha)").execute(&pool).await;

        // ========================
        // TABLA: alertas_financieras (Semáforo de vencimientos)
        // ========================
        sqlx::query("CREATE TABLE IF NOT EXISTS alertas_financieras (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tipo TEXT NOT NULL,
            severidad TEXT NOT NULL,
            titulo TEXT NOT NULL,
            mensaje TEXT NOT NULL,
            entidad_id INTEGER,
            entidad_tipo TEXT,
            fecha_vencimiento DATE,
            leida INTEGER DEFAULT 0,
            creada_en DATETIME DEFAULT CURRENT_TIMESTAMP
        )").execute(&pool).await.expect("Fallo al crear tabla alertas_financieras");

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_alertas_tipo ON alertas_financieras(tipo)").execute(&pool).await;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_alertas_severidad ON alertas_financieras(severidad)").execute(&pool).await;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_alertas_leida ON alertas_financieras(leida)").execute(&pool).await;

        (pool, db_path_str)
    })
}


