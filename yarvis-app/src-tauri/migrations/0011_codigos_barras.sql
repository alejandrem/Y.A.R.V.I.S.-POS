-- 0011: Soporte robusto de codigos de barras por producto.
--
-- El escaner (HID teclado) entrega el codigo + Enter y el backend resuelve
-- con busqueda exacta. Para que eso sea determinista:
--   1) Los vacios / solo-espacios pasan a NULL ("" NO es NULL y romperia
--      el indice parcial con colisiones de "").
--   2) Se normaliza como en Rust (codigos.rs): sin espacios/guiones
--      internos y en mayusculas, para que el historial matchee con lo que
--      el escaner entrega ("750-123" == "750123").
--   3) Los duplicados historicos se desvinculan: se conserva el MIN(id)
--      por codigo y el resto pasa a NULL (no se borra el producto).
--   4) Se garantiza el indice unico parcial para DBs viejas creadas antes
--      de que existiera en 0001 (IF NOT EXISTS lo hace idempotente).

UPDATE productos SET codigo_barras = NULL WHERE codigo_barras IS NOT NULL AND TRIM(codigo_barras) = '';

UPDATE productos SET codigo_barras = UPPER(REPLACE(REPLACE(TRIM(codigo_barras), ' ', ''), '-', '')) WHERE codigo_barras IS NOT NULL;

UPDATE productos SET codigo_barras = NULL WHERE codigo_barras IS NOT NULL AND TRIM(codigo_barras) = '';

UPDATE productos SET codigo_barras = NULL WHERE codigo_barras IS NOT NULL AND id NOT IN (SELECT MIN(id) FROM productos WHERE codigo_barras IS NOT NULL GROUP BY codigo_barras);

CREATE UNIQUE INDEX IF NOT EXISTS idx_productos_codigo_barras ON productos(codigo_barras) WHERE codigo_barras IS NOT NULL;
