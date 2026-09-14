-- ============================================================
-- 0016_amarillo — Historial de rechazos del semaforo amarillo.
--
-- Issue #11: "todo rechazo se guarda: los NO alimentan el umbral
-- futuro (si todos rechazan sugerencias de 0.82, el umbral estaba
-- bajo)". Esta tabla es ese historial: quien/que/cuando se rechazo
-- con que score. El umbral NO se auto-ajusta (peligroso); se expone
-- via stats y el dueno lo recalibra a mano en UNA constante
-- (`semaforo_amarillo::umbral::UMBRAL_AMARILLO`).
--
-- La sugerencia ACTIVA vive en `pendientes_codigos.mejor_*` (0015);
-- aqui solo el historial de NOs. Borrados en cascada por diseño.
-- ============================================================

CREATE TABLE IF NOT EXISTS rechazos_amarillos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pendiente_id INTEGER NOT NULL REFERENCES pendientes_codigos(id) ON DELETE CASCADE,
    producto_id INTEGER REFERENCES productos(id) ON DELETE SET NULL,
    score REAL,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_rechazos_pendiente ON rechazos_amarillos(pendiente_id);
CREATE INDEX IF NOT EXISTS idx_rechazos_producto ON rechazos_amarillos(producto_id);
