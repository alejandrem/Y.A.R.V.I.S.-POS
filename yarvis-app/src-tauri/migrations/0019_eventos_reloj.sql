-- Registra avisos de desfase del reloj asociados al empleado conectado.
-- Los avisos no bloquean ventas ni cortes; quedan visibles para el admin.
CREATE TABLE IF NOT EXISTS eventos_reloj (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    empleado_id INTEGER NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    tipo TEXT NOT NULL,
    hora_sistema TEXT NOT NULL,
    hora_referencia TEXT,
    desfase_seg INTEGER,
    mensaje TEXT NOT NULL,
    detectado_en TEXT NOT NULL DEFAULT (datetime('now','localtime')),
    ultima_observacion TEXT NOT NULL DEFAULT (datetime('now','localtime')),
    resuelto_en TEXT
);

CREATE INDEX IF NOT EXISTS idx_eventos_reloj_empleado
    ON eventos_reloj(empleado_id, id DESC);

CREATE INDEX IF NOT EXISTS idx_eventos_reloj_abiertos
    ON eventos_reloj(empleado_id, resuelto_en);
