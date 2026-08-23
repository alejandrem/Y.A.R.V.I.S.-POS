-- ═══════════════════════════════════════════════════════════════════
-- 0004: Registro de asistencia (asistencia/puntualidad/horas extra).
-- Un renglón por empleado por día: el PRIMER login del día marca la
-- entrada real (aunque haya N logins más, no se toca). Los logins
-- después de la hora de salida están permitidos (horas extra).
-- ═══════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS asistencias (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    empleado_id INTEGER NOT NULL,
    fecha TEXT NOT NULL,
    primer_login TEXT NOT NULL,
    ultimo_login TEXT,
    UNIQUE(empleado_id, fecha),
    FOREIGN KEY (empleado_id) REFERENCES usuarios(id) ON DELETE CASCADE
);
