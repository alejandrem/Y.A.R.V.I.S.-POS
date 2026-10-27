ALTER TABLE asistencias
ADD COLUMN turno_activo INTEGER NOT NULL DEFAULT 0
CHECK (turno_activo IN (0, 1));

ALTER TABLE asistencias ADD COLUMN sesion_inicio TEXT;

-- Recupera el estado previo usando el último login y el último corte Z.
-- Solo puede quedar activo el registro de asistencia más reciente.
UPDATE asistencias
SET turno_activo = 1
WHERE id = (
    SELECT a2.id
    FROM asistencias a2
    WHERE a2.empleado_id = asistencias.empleado_id
    ORDER BY a2.fecha DESC, a2.id DESC
    LIMIT 1
)
AND ultimo_login IS NOT NULL
AND ultimo_login > COALESCE((
    SELECT MAX(c.fecha_cierre)
    FROM cortes_caja c
    WHERE c.usuario_id = asistencias.empleado_id
      AND c.tipo_corte = 'Z'
      AND c.estado = 'cerrado'
      AND c.fecha_cierre IS NOT NULL
), '');

UPDATE asistencias
SET sesion_inicio = ultimo_login
WHERE turno_activo = 1;
