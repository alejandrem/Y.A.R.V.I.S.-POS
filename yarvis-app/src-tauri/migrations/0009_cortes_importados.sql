-- ═══════════════════════════════════════════════════════════════════
-- 0009: Cortes de caja importados (parseo histórico X/Z).
--
-- `cortes_caja` queda SOLO para cortes operativos del POS (apertura,
-- cierre, movimientos en vivo). Lo parseado de archivos históricos va
-- aquí: trae folio/estación/impuestos/desgloses que el modelo
-- operativo no tiene, e idempotencia por hash de contenido (igual que
-- `catalogos_importados`): re-importar la misma carpeta no duplica.
--
-- Dinero en INTEGER centavos (regla de oro); cantidades en REAL.
-- ═══════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS cortes_importados (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- 'X' (parcial, no reinicia) | 'Z' (cierre definitivo del día)
    tipo TEXT NOT NULL,
    folio TEXT,
    estacion TEXT,
    cajero TEXT DEFAULT 'SISTEMA',
    empresa TEXT,
    moneda TEXT DEFAULT 'MXN',
    fecha DATETIME,
    total_ingresos INTEGER DEFAULT 0,
    total_egresos INTEGER DEFAULT 0,
    total_caja INTEGER DEFAULT 0,
    total_ventas INTEGER DEFAULT 0,
    ventas_gravadas INTEGER DEFAULT 0,
    impuesto INTEGER DEFAULT 0,
    ventas_no_gravadas INTEGER DEFAULT 0,
    redondeos INTEGER DEFAULT 0,
    ventas_credito INTEGER DEFAULT 0,
    total_unidades REAL DEFAULT 0,
    clientes_atendidos INTEGER DEFAULT 0,
    ruta_archivo TEXT,
    -- SHA256 del contenido: re-importar es seguro (se omite).
    hash TEXT UNIQUE NOT NULL,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_cortes_imp_tipo ON cortes_importados(tipo);
CREATE INDEX IF NOT EXISTS idx_cortes_imp_fecha ON cortes_importados(fecha);

-- Desglose del corte: renglones de "Ventas por artículo" (ARTICULO),
-- "Ventas por ticket" (TICKET) y líneas de Ingresos/Egresos
-- (INGRESO/EGRESO, sin cantidad).
CREATE TABLE IF NOT EXISTS cortes_importados_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    corte_id INTEGER NOT NULL REFERENCES cortes_importados(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    nombre TEXT NOT NULL,
    cantidad REAL,
    precio_unitario INTEGER DEFAULT 0,
    subtotal INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_cortes_imp_items_corte
    ON cortes_importados_items(corte_id);
