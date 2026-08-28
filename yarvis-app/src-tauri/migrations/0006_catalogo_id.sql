-- ═══════════════════════════════════════════════════════════
-- 0006: Productos y embeddings vinculados a su catálogo.
-- Fix BUG-02: get_productos_por_catalogo ignoraba el id y devolvía
-- los últimos 100 globales. Ahora filtra por catalogo_id.
-- Fix BUG-07: knowledge_base guarda producto_id para evitar
-- el fallback a _kb_id cuando el nombre del KB no matchea.
-- ═══════════════════════════════════════════════════════════

ALTER TABLE productos ADD COLUMN catalogo_id INTEGER REFERENCES catalogos_importados(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_productos_catalogo_id ON productos(catalogo_id);

ALTER TABLE knowledge_base ADD COLUMN producto_id INTEGER REFERENCES productos(id) ON DELETE CASCADE;
CREATE INDEX IF NOT EXISTS idx_kb_producto_id ON knowledge_base(producto_id);
