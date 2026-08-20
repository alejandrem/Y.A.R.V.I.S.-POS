-- ============================================================
-- 0001_inicial — Esquema base de Y.A.R.V.I.S. POS
--
-- Baseline que representa el estado ACTUAL de la DB (era la que
-- db.rs construía con CREATE + ALTER a ciegas). Todas las columnas
-- históricamente agregadas con ALTER TABLE ya están horneadas aquí.
--
-- REGLAS de este esquema:
--   * Todo es IF NOT EXISTS para poder adoptar DBs pre-existentes
--     sin tocarlas ni perder datos.
--   * NUNCA edites una migración ya aplicada (sqlx valida el hash).
--     Los cambios futuros van en 0002_asunto.sql, 0003_..., etc.
-- ============================================================

-- ------------------ usuarios ------------------
CREATE TABLE IF NOT EXISTS usuarios (
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
    ultimo_login DATETIME,
    salario_diario REAL DEFAULT 0,
    dias_semana INTEGER DEFAULT 6
);

-- ------------------ employee_goals ------------------
CREATE TABLE IF NOT EXISTS employee_goals (
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
);

-- ------------------ productos ------------------
-- FIX (auditoría): la unicidad de codigo_barras se aplica con un índice
-- parcial más abajo (ignora NULLs), NO con UNIQUE inline, para que una DB
-- nueva y una migrada se comporten idéntico.
CREATE TABLE IF NOT EXISTS productos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    descripcion TEXT,
    precio_costo REAL DEFAULT 0,
    precio_venta REAL DEFAULT 0,
    stock REAL DEFAULT 0,
    stock_minimo REAL DEFAULT 0,
    codigo_barras TEXT,
    categoria TEXT,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP,
    vendido REAL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_productos_codigo_barras
    ON productos(codigo_barras) WHERE codigo_barras IS NOT NULL;

-- FIX (auditoría): índices en las columnas que consulta el chatbot en cada
-- pregunta de ventas/productos (obtener_ventas_hoy, top vendidos...).
CREATE INDEX IF NOT EXISTS idx_productos_nombre ON productos(nombre);

-- ------------------ clientes ------------------
CREATE TABLE IF NOT EXISTS clientes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    rfc TEXT UNIQUE,
    email TEXT,
    telefono TEXT,
    direccion TEXT,
    credito_limite REAL DEFAULT 0,
    notas TEXT,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ------------------ ventas ------------------
CREATE TABLE IF NOT EXISTS ventas (
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
    folio_ticket TEXT,
    monto_efectivo REAL DEFAULT 0,
    monto_tarjeta REAL DEFAULT 0,
    monto_transferencia REAL DEFAULT 0,
    FOREIGN KEY (cliente_id) REFERENCES clientes(id)
);

-- FIX (auditoría): fecha → ventas por rango, cajero → por cajero,
-- estado → todos los queries filtran "estado != 'cancelada'".
CREATE INDEX IF NOT EXISTS idx_ventas_fecha ON ventas(fecha);
CREATE INDEX IF NOT EXISTS idx_ventas_cajero ON ventas(cajero);
CREATE INDEX IF NOT EXISTS idx_ventas_estado ON ventas(estado);

-- ------------------ detalle_ventas ------------------
-- producto_id es nullable (None para productos no linkeados aún).
CREATE TABLE IF NOT EXISTS detalle_ventas (
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
);

-- FIX (auditoría): venta_id → JOIN de cada query de ventas por producto;
-- producto_nombre → todos los GROUP BY producto_nombre.
CREATE INDEX IF NOT EXISTS idx_detalle_venta_id ON detalle_ventas(venta_id);
CREATE INDEX IF NOT EXISTS idx_detalle_producto_nombre ON detalle_ventas(producto_nombre);

-- ------------------ ventas_diarias ------------------
CREATE TABLE IF NOT EXISTS ventas_diarias (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha DATE NOT NULL UNIQUE,
    total_ventas REAL DEFAULT 0,
    cantidad_tickets INTEGER DEFAULT 0,
    temperatura_promedio REAL,
    clima TEXT,
    utilidad_bruta REAL,
    utilidad_operativa REAL,
    utilidad_neta REAL
);

-- ------------------ cortes_caja ------------------
CREATE TABLE IF NOT EXISTS cortes_caja (
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
    tipo_corte TEXT DEFAULT 'Z',
    turno TEXT,
    entradas_manuales REAL DEFAULT 0,
    retiros_manuales REAL DEFAULT 0,
    observaciones TEXT,
    FOREIGN KEY (usuario_id) REFERENCES usuarios(id)
);

-- ------------------ predicciones_futuras ------------------
CREATE TABLE IF NOT EXISTS predicciones_futuras (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha_prediccion DATE NOT NULL,
    producto TEXT,
    cantidad_sugerida REAL,
    margen_error REAL,
    confianza REAL,
    generado_en DATETIME DEFAULT CURRENT_TIMESTAMP,
    notas TEXT
);

-- ------------------ knowledge_base (sqlite-vec placeholder) ------------------
CREATE TABLE IF NOT EXISTS knowledge_base (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contenido TEXT NOT NULL,
    categoria TEXT NOT NULL,
    embedding BLOB,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ------------------ catalogos_importados (control de duplicados) ------------------
CREATE TABLE IF NOT EXISTS catalogos_importados (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hash TEXT UNIQUE NOT NULL,
    ruta_archivo TEXT,
    fecha_importacion DATETIME DEFAULT CURRENT_TIMESTAMP,
    total_productos INTEGER DEFAULT 0
);

-- ------------------ gastos_recurrentes (Módulo Finanzas) ------------------
CREATE TABLE IF NOT EXISTS gastos_recurrentes (
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
);

CREATE INDEX IF NOT EXISTS idx_gastos_fecha_inicio ON gastos_recurrentes(fecha_inicio);
CREATE INDEX IF NOT EXISTS idx_gastos_estado ON gastos_recurrentes(estado_pago);
CREATE INDEX IF NOT EXISTS idx_gastos_tipo ON gastos_recurrentes(tipo);

-- ------------------ pagos_gastos (historial de pagos) ------------------
CREATE TABLE IF NOT EXISTS pagos_gastos (
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
);

CREATE INDEX IF NOT EXISTS idx_pagos_gasto_id ON pagos_gastos(gasto_id);
CREATE INDEX IF NOT EXISTS idx_pagos_fecha ON pagos_gastos(fecha_pago);

-- ------------------ movimientos_caja ------------------
CREATE TABLE IF NOT EXISTS movimientos_caja (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    corte_id INTEGER NOT NULL,
    tipo TEXT NOT NULL,
    concepto TEXT NOT NULL,
    monto REAL NOT NULL,
    metodo_pago TEXT,
    referencia_id INTEGER,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (corte_id) REFERENCES cortes_caja(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_movimientos_corte ON movimientos_caja(corte_id);
CREATE INDEX IF NOT EXISTS idx_movimientos_tipo ON movimientos_caja(tipo);

-- ------------------ resumen_financiero_diario (vista materializada P&L) ------------------
CREATE TABLE IF NOT EXISTS resumen_financiero_diario (
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
);

CREATE INDEX IF NOT EXISTS idx_resumen_fecha ON resumen_financiero_diario(fecha);

-- ------------------ alertas_financieras (semáforo de vencimientos) ------------------
CREATE TABLE IF NOT EXISTS alertas_financieras (
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
);

CREATE INDEX IF NOT EXISTS idx_alertas_tipo ON alertas_financieras(tipo);
CREATE INDEX IF NOT EXISTS idx_alertas_severidad ON alertas_financieras(severidad);
CREATE INDEX IF NOT EXISTS idx_alertas_leida ON alertas_financieras(leida);