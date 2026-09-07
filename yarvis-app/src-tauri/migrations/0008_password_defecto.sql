-- ═══════════════════════════════════════════════════════════════════
-- 0008: Flag de contraseña predeterminada (empleados auto-creados).
-- Los empleados que el sistema crea solos desde tickets traen pass débil
-- (nombre+123): se les avisa en cada login que pidan cambiarla y el flag
-- se apaga cuando el admin les pone una nueva (editar_empleado).
-- ═══════════════════════════════════════════════════════════════════════════

ALTER TABLE usuarios ADD COLUMN password_defecto INTEGER DEFAULT 0;
