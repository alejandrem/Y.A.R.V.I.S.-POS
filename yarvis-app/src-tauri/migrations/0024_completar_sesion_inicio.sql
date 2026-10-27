UPDATE asistencias
SET sesion_inicio = ultimo_login
WHERE turno_activo = 1
  AND sesion_inicio IS NULL
  AND ultimo_login IS NOT NULL;
