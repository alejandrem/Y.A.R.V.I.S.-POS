-- ═══════════════════════════════════════════════════════════════════
-- 0014: Rectificativas de compras (antifraude).
--
-- Las facturas de compra son INMUTABLES: no se editan ni se borran.
-- Si el empleado se equivocó, se crea una compra NUEVA que apunta a
-- la original (`rectifica_a`); la original se conserva intacta para
-- que el dueño vea ambas. Sin este rastro, editar un pago serviría
-- para desviar dinero sin dejar huella.
-- ═══════════════════════════════════════════════════════════════════

ALTER TABLE compras ADD COLUMN rectifica_a INTEGER REFERENCES compras(id);

CREATE INDEX IF NOT EXISTS idx_compras_rectifica ON compras(rectifica_a);
