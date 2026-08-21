-- ═══════════════════════════════════════════════════════════════════
-- 0002: Horarios multiples por empleado.
-- Un empleado puede tener varios bloques de horario (ej: L,X,J,V de 8-17
-- y S,D de 8-12). La convencion de dias es indice de chip L=0..D=6.
-- Las columnas legacy horario_inicio/horario_fin/dias_semana de usuarios
-- se mantienen espejando el primer bloque por compatibilidad.
-- ═══════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS empleado_horarios (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    empleado_id INTEGER NOT NULL,
    dias TEXT NOT NULL DEFAULT '',
    hora_inicio TEXT NOT NULL DEFAULT '00:00',
    hora_fin TEXT NOT NULL DEFAULT '00:00',
    FOREIGN KEY (empleado_id) REFERENCES usuarios(id) ON DELETE CASCADE
);
