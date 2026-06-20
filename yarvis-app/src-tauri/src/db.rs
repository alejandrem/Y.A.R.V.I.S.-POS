//base de datos de yarvis (tablas solamente)
use std::fs;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tauri::Manager;

pub fn initialize_db(app: &tauri::AppHandle) -> SqlitePool {
    let app_dir = app.path().app_data_dir().expect("No se pudo obtener el directorio de datos");
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).expect("No se pudo crear el directorio de datos");
    }

    let db_path = app_dir.join("yarvis.db");

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
            cp TEXT
        )").execute(&pool).await.expect("Fallo al crear tabla de usuarios");

        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN ubicacion TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE usuarios ADD COLUMN cp TEXT").execute(&pool).await;

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

        pool
    })
}


