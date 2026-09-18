-- ═══════════════════════════════════════════════════════════════════
-- 0018_abasto_trazabilidad — Órdenes de compra, historial de costos,
-- lotes/caducidades y sucursales para las tools del chat.
--
-- Contexto: `proveedores` + `compras` ya existen (0012/0013) pero el chat
-- no los consultaba; las órdenes de compra (pedidos pendientes), el
-- historial de precio_costo, los lotes con caducidad y las sucursales no
-- existían en ninguna tabla. Todo aquí es lectura para las tools: las
-- tools jamás escriben, solo SELECT.
--
-- Reglas del repo que se respetan:
--   * Dinero en INTEGER centavos (0012 ya lo hace; aquí igual).
--   * Cantidades/stock en REAL. Fechas DATETIME/DATE como texto.
--   * IF NOT EXISTS en todo lo creable (adopción de DBs viejas).
--   * El trigger trg_historial_costos registra solo cambios futuros de
--     precio_costo (no inventa historia pasada: el pasado se deriva de
--     compras_items.precio_sugerido en la propia tool).
-- ═══════════════════════════════════════════════════════════════════

-- ---------- órdenes de compra (el pedido, antes de la recepción) ----------
-- La recepción real sigue viviendo en `compras` (0012); la orden es el
-- papel previo: qué se pidió, a quién, en qué estado va.
CREATE TABLE IF NOT EXISTS ordenes_compra (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proveedor_id INTEGER NOT NULL REFERENCES proveedores(id) ON DELETE RESTRICT,
    fecha DATETIME DEFAULT CURRENT_TIMESTAMP,
    estado TEXT DEFAULT 'pendiente'
        CHECK(estado IN ('pendiente', 'parcial', 'recibida', 'cancelada')),
    total_estimado INTEGER DEFAULT 0,
    notas TEXT,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_ordenes_proveedor ON ordenes_compra(proveedor_id);
CREATE INDEX IF NOT EXISTS idx_ordenes_estado ON ordenes_compra(estado);
CREATE INDEX IF NOT EXISTS idx_ordenes_fecha ON ordenes_compra(fecha);

CREATE TABLE IF NOT EXISTS ordenes_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    orden_id INTEGER NOT NULL REFERENCES ordenes_compra(id) ON DELETE CASCADE,
    producto_id INTEGER REFERENCES productos(id) ON DELETE SET NULL,
    nombre TEXT NOT NULL,
    cantidad REAL NOT NULL DEFAULT 0,
    cantidad_recibida REAL DEFAULT 0,
    costo_unitario INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_ordenes_items_orden ON ordenes_items(orden_id);

-- Vínculo recepción→pedido (NULL = compra directa sin orden previa).
ALTER TABLE compras ADD COLUMN orden_id INTEGER;

-- ---------- historial de precio_costo (trazabilidad de costos) ----------
CREATE TABLE IF NOT EXISTS historial_costos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    producto_id INTEGER REFERENCES productos(id) ON DELETE SET NULL,
    producto_nombre TEXT NOT NULL,
    costo_anterior INTEGER DEFAULT 0,
    costo_nuevo INTEGER NOT NULL,
    proveedor_id INTEGER REFERENCES proveedores(id) ON DELETE SET NULL,
    compra_id INTEGER REFERENCES compras(id) ON DELETE SET NULL,
    fecha DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_historial_costos_producto ON historial_costos(producto_id);
CREATE INDEX IF NOT EXISTS idx_historial_costos_fecha ON historial_costos(fecha);

-- Solo cambios futuros: cada UPDATE de precio_costo deja su rastro solo.
CREATE TRIGGER IF NOT EXISTS trg_historial_costos
AFTER UPDATE OF precio_costo ON productos
WHEN OLD.precio_costo IS NOT NEW.precio_costo
BEGIN
    INSERT INTO historial_costos (producto_id, producto_nombre, costo_anterior, costo_nuevo)
    VALUES (OLD.id, OLD.nombre, COALESCE(OLD.precio_costo, 0), COALESCE(NEW.precio_costo, 0));
END;

-- ---------- lotes y caducidades (trazabilidad por lote) ----------
CREATE TABLE IF NOT EXISTS lotes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    producto_id INTEGER REFERENCES productos(id) ON DELETE SET NULL,
    producto_nombre TEXT NOT NULL,
    lote TEXT NOT NULL DEFAULT '',
    caducidad DATE,
    cantidad REAL DEFAULT 0,
    compra_id INTEGER REFERENCES compras(id) ON DELETE SET NULL,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_lotes_producto ON lotes(producto_id);
CREATE INDEX IF NOT EXISTS idx_lotes_caducidad ON lotes(caducidad);

-- ---------- sucursales y su stock (multisucursal mínimo) ----------
-- El stock GLOBAL sigue en productos.stock (la tienda original); cada
-- sucursal extra lleva el suyo en stock_sucursal. Sin sucursales
-- registradas, las tools responden vacío en vez de fallar.
CREATE TABLE IF NOT EXISTS sucursales (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    direccion TEXT,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sucursales_nombre
    ON sucursales(nombre COLLATE NOCASE);

CREATE TABLE IF NOT EXISTS stock_sucursal (
    sucursal_id INTEGER NOT NULL REFERENCES sucursales(id) ON DELETE CASCADE,
    producto_id INTEGER NOT NULL REFERENCES productos(id) ON DELETE CASCADE,
    stock REAL DEFAULT 0,
    PRIMARY KEY (sucursal_id, producto_id)
);

CREATE INDEX IF NOT EXISTS idx_stock_sucursal_producto ON stock_sucursal(producto_id);
