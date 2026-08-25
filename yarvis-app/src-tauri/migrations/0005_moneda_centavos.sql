-- ============================================================
-- 0005_moneda_centavos — El dinero pasa a vivir en CENTAVOS (INTEGER).
--
-- REGLA DE ORO (ver src/dinero.rs): toda columna monetaria es INTEGER
-- en centavos. Las CANTIDADES (stock, cantidad, temperatura, porcentajes)
-- siguen siendo REAL porque no son dinero.
--
-- Por qué reconstrucción completa: SQLite no soporta ALTER COLUMN, y una
-- columna con afinidad REAL convierte cualquier entero escrito a float,
-- así que no basta con escribir enteros en las columnas viejas.
--
-- IMPORTANTE sobre foreign_keys: sqlx-sqlite envuelve SIEMPRE cada
-- migración en una transacción (ignora el marcador `-- no-transaction`)
-- y SQLite ignora `PRAGMA foreign_keys` dentro de una transacción.
-- Por eso las FKs se apagan DESDE LA CONEXIÓN en db.rs (fase 1 de
-- initialize_db) y se reactivan al reabrir el pool (fase 2). No intentar
-- controlarlas desde aquí: sería un no-op silencioso que solo falla con
-- datos reales, nunca con DBs vacías de test.
--
-- Conversión: CAST(ROUND(col * 100) AS INTEGER) — redondea al centavo
-- más cercano para absorber el ruido binario del f64 histórico.
-- Los NULL se preservan (ROUND(NULL)=NULL).
-- ============================================================

-- ------------------ usuarios ------------------
CREATE TABLE usuarios_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    tienda TEXT,
    password TEXT NOT NULL,
    rol TEXT NOT NULL,
    ubicacion TEXT,
    cp TEXT,
    salario_semanal INTEGER DEFAULT 0,
    turno TEXT DEFAULT 'matutino',
    horario_inicio TEXT DEFAULT '08:00',
    horario_fin TEXT DEFAULT '14:00',
    meta_mensual INTEGER DEFAULT 0,
    bono INTEGER DEFAULT 0,
    estado TEXT DEFAULT 'activo',
    registrado_en DATETIME DEFAULT '2000-01-01 00:00:00',
    ultimo_login DATETIME,
    salario_diario INTEGER DEFAULT 0,
    dias_semana INTEGER DEFAULT 6
);
INSERT INTO usuarios_new
SELECT id, nombre, tienda, password, rol, ubicacion, cp,
       CAST(ROUND(salario_semanal * 100) AS INTEGER),
       turno, horario_inicio, horario_fin,
       CAST(ROUND(meta_mensual * 100) AS INTEGER),
       CAST(ROUND(bono * 100) AS INTEGER),
       estado, registrado_en, ultimo_login,
       CAST(ROUND(salario_diario * 100) AS INTEGER),
       dias_semana
FROM usuarios;
DROP TABLE usuarios;
ALTER TABLE usuarios_new RENAME TO usuarios;

-- ------------------ employee_goals ------------------
CREATE TABLE employee_goals_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    goal_type TEXT NOT NULL,
    goal_name TEXT,
    ventas_threshold TEXT DEFAULT '5',
    bonus_percentage REAL DEFAULT 0,
    bonus_amount INTEGER DEFAULT 0,
    is_completed INTEGER DEFAULT 0,
    completed_at TEXT,
    created_at TEXT DEFAULT (datetime('now','localtime')),
    FOREIGN KEY (employee_id) REFERENCES usuarios(id)
);
INSERT INTO employee_goals_new
SELECT id, employee_id, goal_type, goal_name, ventas_threshold,
       bonus_percentage,
       CAST(ROUND(bonus_amount * 100) AS INTEGER),
       is_completed, completed_at, created_at
FROM employee_goals;
DROP TABLE employee_goals;
ALTER TABLE employee_goals_new RENAME TO employee_goals;

-- ------------------ productos ------------------
CREATE TABLE productos_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    descripcion TEXT,
    precio_costo INTEGER DEFAULT 0,
    precio_venta INTEGER DEFAULT 0,
    stock REAL DEFAULT 0,
    stock_minimo REAL DEFAULT 0,
    codigo_barras TEXT,
    categoria TEXT,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP,
    vendido REAL DEFAULT 0
);
INSERT INTO productos_new
SELECT id, nombre, descripcion,
       CAST(ROUND(precio_costo * 100) AS INTEGER),
       CAST(ROUND(precio_venta * 100) AS INTEGER),
       stock, stock_minimo, codigo_barras, categoria, creado_en, vendido
FROM productos;
DROP TABLE productos;
ALTER TABLE productos_new RENAME TO productos;

CREATE UNIQUE INDEX IF NOT EXISTS idx_productos_codigo_barras
    ON productos(codigo_barras) WHERE codigo_barras IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_productos_nombre ON productos(nombre);

-- ------------------ clientes ------------------
CREATE TABLE clientes_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    rfc TEXT UNIQUE,
    email TEXT,
    telefono TEXT,
    direccion TEXT,
    credito_limite INTEGER DEFAULT 0,
    notas TEXT,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO clientes_new
SELECT id, nombre, rfc, email, telefono, direccion,
       CAST(ROUND(credito_limite * 100) AS INTEGER),
       notas, creado_en
FROM clientes;
DROP TABLE clientes;
ALTER TABLE clientes_new RENAME TO clientes;

-- ------------------ ventas ------------------
-- cajero_id fue agregado en 0003; el rebuild lo conserva.
CREATE TABLE ventas_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha DATETIME DEFAULT CURRENT_TIMESTAMP,
    total INTEGER NOT NULL,
    subtotal INTEGER,
    iva INTEGER,
    descuento INTEGER DEFAULT 0,
    metodo_pago TEXT DEFAULT 'efectivo',
    cajero TEXT NOT NULL,
    cliente_id INTEGER,
    clima TEXT,
    estado TEXT DEFAULT 'completada',
    folio_ticket TEXT,
    monto_efectivo INTEGER DEFAULT 0,
    monto_tarjeta INTEGER DEFAULT 0,
    monto_transferencia INTEGER DEFAULT 0,
    cajero_id INTEGER,
    FOREIGN KEY (cliente_id) REFERENCES clientes(id)
);
INSERT INTO ventas_new
SELECT id, fecha,
       CAST(ROUND(total * 100) AS INTEGER),
       CAST(ROUND(subtotal * 100) AS INTEGER),
       CAST(ROUND(iva * 100) AS INTEGER),
       CAST(ROUND(descuento * 100) AS INTEGER),
       metodo_pago, cajero, cliente_id, clima, estado, folio_ticket,
       CAST(ROUND(monto_efectivo * 100) AS INTEGER),
       CAST(ROUND(monto_tarjeta * 100) AS INTEGER),
       CAST(ROUND(monto_transferencia * 100) AS INTEGER),
       cajero_id
FROM ventas;
DROP TABLE ventas;
ALTER TABLE ventas_new RENAME TO ventas;

CREATE INDEX IF NOT EXISTS idx_ventas_fecha ON ventas(fecha);
CREATE INDEX IF NOT EXISTS idx_ventas_cajero ON ventas(cajero);
CREATE INDEX IF NOT EXISTS idx_ventas_estado ON ventas(estado);
CREATE INDEX IF NOT EXISTS idx_ventas_cajero_id ON ventas(cajero_id);

-- ------------------ detalle_ventas ------------------
CREATE TABLE detalle_ventas_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    venta_id INTEGER NOT NULL,
    producto_id INTEGER,
    producto_nombre TEXT NOT NULL,
    cantidad REAL NOT NULL,
    precio_unitario INTEGER NOT NULL,
    descuento INTEGER DEFAULT 0,
    subtotal INTEGER NOT NULL,
    FOREIGN KEY (venta_id) REFERENCES ventas(id) ON DELETE CASCADE,
    FOREIGN KEY (producto_id) REFERENCES productos(id)
);
INSERT INTO detalle_ventas_new
SELECT id, venta_id, producto_id, producto_nombre, cantidad,
       CAST(ROUND(precio_unitario * 100) AS INTEGER),
       CAST(ROUND(descuento * 100) AS INTEGER),
       CAST(ROUND(subtotal * 100) AS INTEGER)
FROM detalle_ventas;
DROP TABLE detalle_ventas;
ALTER TABLE detalle_ventas_new RENAME TO detalle_ventas;

CREATE INDEX IF NOT EXISTS idx_detalle_venta_id ON detalle_ventas(venta_id);
CREATE INDEX IF NOT EXISTS idx_detalle_producto_nombre ON detalle_ventas(producto_nombre);

-- ------------------ ventas_diarias ------------------
CREATE TABLE ventas_diarias_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha DATE NOT NULL UNIQUE,
    total_ventas INTEGER DEFAULT 0,
    cantidad_tickets INTEGER DEFAULT 0,
    temperatura_promedio REAL,
    clima TEXT,
    utilidad_bruta INTEGER,
    utilidad_operativa INTEGER,
    utilidad_neta INTEGER
);
INSERT INTO ventas_diarias_new
SELECT id, fecha,
       CAST(ROUND(total_ventas * 100) AS INTEGER),
       cantidad_tickets, temperatura_promedio, clima,
       CAST(ROUND(utilidad_bruta * 100) AS INTEGER),
       CAST(ROUND(utilidad_operativa * 100) AS INTEGER),
       CAST(ROUND(utilidad_neta * 100) AS INTEGER)
FROM ventas_diarias;
DROP TABLE ventas_diarias;
ALTER TABLE ventas_diarias_new RENAME TO ventas_diarias;

-- ------------------ cortes_caja ------------------
CREATE TABLE cortes_caja_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha_apertura DATETIME DEFAULT CURRENT_TIMESTAMP,
    fecha_cierre DATETIME,
    monto_inicial INTEGER DEFAULT 0,
    total_ventas INTEGER DEFAULT 0,
    total_efectivo INTEGER DEFAULT 0,
    total_tarjeta INTEGER DEFAULT 0,
    total_transferencia INTEGER DEFAULT 0,
    diferencia INTEGER DEFAULT 0,
    usuario_id INTEGER,
    estado TEXT DEFAULT 'abierto',
    tipo_corte TEXT DEFAULT 'Z',
    turno TEXT,
    entradas_manuales INTEGER DEFAULT 0,
    retiros_manuales INTEGER DEFAULT 0,
    observaciones TEXT,
    FOREIGN KEY (usuario_id) REFERENCES usuarios(id)
);
INSERT INTO cortes_caja_new
SELECT id, fecha_apertura, fecha_cierre,
       CAST(ROUND(monto_inicial * 100) AS INTEGER),
       CAST(ROUND(total_ventas * 100) AS INTEGER),
       CAST(ROUND(total_efectivo * 100) AS INTEGER),
       CAST(ROUND(total_tarjeta * 100) AS INTEGER),
       CAST(ROUND(total_transferencia * 100) AS INTEGER),
       CAST(ROUND(diferencia * 100) AS INTEGER),
       usuario_id, estado, tipo_corte, turno,
       CAST(ROUND(entradas_manuales * 100) AS INTEGER),
       CAST(ROUND(retiros_manuales * 100) AS INTEGER),
       observaciones
FROM cortes_caja;
DROP TABLE cortes_caja;
ALTER TABLE cortes_caja_new RENAME TO cortes_caja;

-- ------------------ gastos_recurrentes ------------------
CREATE TABLE gastos_recurrentes_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    tipo TEXT NOT NULL,
    categoria TEXT NOT NULL,
    monto_proyectado INTEGER NOT NULL,
    monto_real INTEGER DEFAULT 0,
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
INSERT INTO gastos_recurrentes_new
SELECT id, nombre, tipo, categoria,
       CAST(ROUND(monto_proyectado * 100) AS INTEGER),
       CAST(ROUND(monto_real * 100) AS INTEGER),
       frecuencia, dia_pago, intervalo_dias, fecha_inicio, fecha_fin,
       estado_pago, folio_comprobante, comprobante_url, notas,
       creado_en, actualizado_en
FROM gastos_recurrentes;
DROP TABLE gastos_recurrentes;
ALTER TABLE gastos_recurrentes_new RENAME TO gastos_recurrentes;

CREATE INDEX IF NOT EXISTS idx_gastos_fecha_inicio ON gastos_recurrentes(fecha_inicio);
CREATE INDEX IF NOT EXISTS idx_gastos_estado ON gastos_recurrentes(estado_pago);
CREATE INDEX IF NOT EXISTS idx_gastos_tipo ON gastos_recurrentes(tipo);

-- ------------------ pagos_gastos ------------------
CREATE TABLE pagos_gastos_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    gasto_id INTEGER NOT NULL,
    fecha_pago DATETIME NOT NULL,
    monto_pagado INTEGER NOT NULL,
    metodo_pago TEXT,
    folio_comprobante TEXT,
    comprobante_url TEXT,
    notas TEXT,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (gasto_id) REFERENCES gastos_recurrentes(id) ON DELETE CASCADE
);
INSERT INTO pagos_gastos_new
SELECT id, gasto_id, fecha_pago,
       CAST(ROUND(monto_pagado * 100) AS INTEGER),
       metodo_pago, folio_comprobante, comprobante_url, notas, creado_en
FROM pagos_gastos;
DROP TABLE pagos_gastos;
ALTER TABLE pagos_gastos_new RENAME TO pagos_gastos;

CREATE INDEX IF NOT EXISTS idx_pagos_gasto_id ON pagos_gastos(gasto_id);
CREATE INDEX IF NOT EXISTS idx_pagos_fecha ON pagos_gastos(fecha_pago);

-- ------------------ movimientos_caja ------------------
CREATE TABLE movimientos_caja_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    corte_id INTEGER NOT NULL,
    tipo TEXT NOT NULL,
    concepto TEXT NOT NULL,
    monto INTEGER NOT NULL,
    metodo_pago TEXT,
    referencia_id INTEGER,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (corte_id) REFERENCES cortes_caja(id) ON DELETE CASCADE
);
INSERT INTO movimientos_caja_new
SELECT id, corte_id, tipo, concepto,
       CAST(ROUND(monto * 100) AS INTEGER),
       metodo_pago, referencia_id, creado_en
FROM movimientos_caja;
DROP TABLE movimientos_caja;
ALTER TABLE movimientos_caja_new RENAME TO movimientos_caja;

CREATE INDEX IF NOT EXISTS idx_movimientos_corte ON movimientos_caja(corte_id);
CREATE INDEX IF NOT EXISTS idx_movimientos_tipo ON movimientos_caja(tipo);

-- ------------------ resumen_financiero_diario ------------------
-- margen_neto_pct sigue REAL (es un porcentaje, no dinero).
CREATE TABLE resumen_financiero_diario_new (
    fecha DATE PRIMARY KEY,
    ventas_totales INTEGER DEFAULT 0,
    ventas_efectivo INTEGER DEFAULT 0,
    ventas_tarjeta INTEGER DEFAULT 0,
    ventas_transferencia INTEGER DEFAULT 0,
    costo_ventas INTEGER DEFAULT 0,
    utilidad_bruta INTEGER DEFAULT 0,
    gastos_operativos INTEGER DEFAULT 0,
    utilidad_operativa INTEGER DEFAULT 0,
    impuestos_comisiones INTEGER DEFAULT 0,
    utilidad_neta INTEGER DEFAULT 0,
    margen_neto_pct REAL DEFAULT 0,
    cortes_z_count INTEGER DEFAULT 0,
    diferencia_caja_total INTEGER DEFAULT 0,
    actualizado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO resumen_financiero_diario_new
SELECT fecha,
       CAST(ROUND(ventas_totales * 100) AS INTEGER),
       CAST(ROUND(ventas_efectivo * 100) AS INTEGER),
       CAST(ROUND(ventas_tarjeta * 100) AS INTEGER),
       CAST(ROUND(ventas_transferencia * 100) AS INTEGER),
       CAST(ROUND(costo_ventas * 100) AS INTEGER),
       CAST(ROUND(utilidad_bruta * 100) AS INTEGER),
       CAST(ROUND(gastos_operativos * 100) AS INTEGER),
       CAST(ROUND(utilidad_operativa * 100) AS INTEGER),
       CAST(ROUND(impuestos_comisiones * 100) AS INTEGER),
       CAST(ROUND(utilidad_neta * 100) AS INTEGER),
       margen_neto_pct, cortes_z_count,
       CAST(ROUND(diferencia_caja_total * 100) AS INTEGER),
       actualizado_en
FROM resumen_financiero_diario;
DROP TABLE resumen_financiero_diario;
ALTER TABLE resumen_financiero_diario_new RENAME TO resumen_financiero_diario;

CREATE INDEX IF NOT EXISTS idx_resumen_fecha ON resumen_financiero_diario(fecha);

