-- ═══════════════════════════════════════════════════════════════════
-- 0010: Vínculo de cortes importados con el catálogo maestro.
--
-- Los ARTICULO de un corte se resuelven contra `productos` (el
-- catálogo que ya se importó): `producto_id` guarda el vínculo como
-- en `detalle_ventas`. NULL = sin vincular (no se inventa).
-- ═══════════════════════════════════════════════════════════════════

ALTER TABLE cortes_importados_items ADD COLUMN producto_id INTEGER;

CREATE INDEX IF NOT EXISTS idx_cortes_imp_items_producto
    ON cortes_importados_items(producto_id);
