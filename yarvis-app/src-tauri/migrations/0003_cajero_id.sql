-- ═══════════════════════════════════════════════════════════════════
-- 0003: Ventas vinculadas por ID de cajero (no por nombre).
-- Antes ventas.cajero guardaba el nombre del cajero: renombrar un
-- empleado rompía su historial y dos empleados con el mismo nombre
-- compartían estadísticas. Ahora:
--   · ventas.cajero_id  → INTEGER, fuente canónica (FK lógica a usuarios.id)
--   · ventas.cajero     → se conserva como etiqueta de display/legado
-- Backfill: vincula ventas históricas al empleado por nombre. Si había
-- nombres duplicados se toma el primer id (indeterminista pero estable).
-- Ventas sin empleado real (ej. IMPORTADOR) quedan con cajero_id NULL.
-- ═══════════════════════════════════════════════════════════════════

ALTER TABLE ventas ADD COLUMN cajero_id INTEGER;

UPDATE ventas SET cajero_id = (
    SELECT id FROM usuarios WHERE usuarios.nombre = ventas.cajero LIMIT 1
);

CREATE INDEX IF NOT EXISTS idx_ventas_cajero_id ON ventas(cajero_id);
