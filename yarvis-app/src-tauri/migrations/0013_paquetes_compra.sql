-- ═══════════════════════════════════════════════════════════════════
-- 0013: Desglose de paquetes en renglones de compra.
--
-- Cuando la presentación es 'paquete', el empleado captura cuántas
-- piezas trae cada paquete y cuántos paquetes recibe; el total de
-- unidades (piezas × paquetes) es lo que suma al stock y lo que usa
-- la sugerencia. En 'unidad' ambas quedan NULL.
-- ═══════════════════════════════════════════════════════════════════

ALTER TABLE compras_items ADD COLUMN piezas_por_paquete REAL;

ALTER TABLE compras_items ADD COLUMN paquetes REAL;
