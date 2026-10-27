-- Devoluciones de ticket completo. Una venta solo puede devolverse una vez.
-- corte_id/movimiento_id quedan NULL si no hay un corte abierto; la
-- devolución se conserva y puede identificarse como egreso pendiente.
CREATE TABLE IF NOT EXISTS devoluciones (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    venta_id INTEGER NOT NULL UNIQUE REFERENCES ventas(id) ON DELETE RESTRICT,
    empleado_id INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    corte_id INTEGER REFERENCES cortes_caja(id) ON DELETE SET NULL,
    movimiento_id INTEGER,
    monto INTEGER NOT NULL,
    metodo_pago TEXT NOT NULL DEFAULT 'efectivo',
    estado_movimiento TEXT NOT NULL DEFAULT 'pendiente'
        CHECK (estado_movimiento IN ('registrado', 'pendiente')),
    creado_en DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_devoluciones_fecha ON devoluciones(creado_en);
CREATE INDEX IF NOT EXISTS idx_devoluciones_cajero ON devoluciones(empleado_id, creado_en);
