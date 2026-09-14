-- ============================================================
-- 0015_semaforo — Base de datos del automatizador de codigos.
--
-- Fase 2 (issues #10 verde, #12 rojo, base para #11 amarillo):
--   * productos gana presentacion canonica (marca + cantidad + unidad)
--     para el bloqueo verde sin JOINs.
--   * catalogo_barras es el espejo exacto de dataset/*.csv
--     (ean PK, nombre sin presentacion + columnas de presentacion).
--   * vinculos_codigos audita CADA vinculo (auto-verde o manual-rojo)
--     con origen + quien confirmo + fecha. Es el "aprendizaje": el
--     amarillo futuro consulta aqui por nombre_norm.
--   * pendientes_codigos es la cola semaforo (rojo hoy, amarillo manana).
--     UNIQUE parcial + codigo idempotente: re-importar no duplica.
--
-- REGLAS:
--   * Todo IF NOT EXISTS / ADD COLUMN simple (migracion unica, sqlx
--     valida hash: nunca editar este archivo una vez aplicado).
--   * Unidades del catalogo cerrado: ml | l | g | kg | pzs (ver
--     dataset/README.md). El CHECK solo vive en catalogo_barras;
--     productos no lleva CHECK para no romper historial viejo.
--   * ean NULLable en vinculos/pendientes: el rojo puede nacer sin
--     ean (ticket sin codigo, solo nombre crudo). Los UNIQUE son
--     parciales (WHERE ean IS NOT NULL) porque en SQLite NULL != NULL
--     y el dedup de NULLs vive en el codigo (SELECT antes de INSERT).
-- ============================================================

-- ---------- 1. Presentacion canonica en productos ----------
-- Bloqueo verde: marca + cantidad + unidad sin JOIN al catalogo.
-- cantidad_presentacion acepta decimales (ej. 1.5 l de Nutralat).
ALTER TABLE productos ADD COLUMN marca TEXT;
ALTER TABLE productos ADD COLUMN cantidad_presentacion REAL;
ALTER TABLE productos ADD COLUMN unidad_presentacion TEXT;

-- ---------- 2. Espejo del dataset real ----------
CREATE TABLE IF NOT EXISTS catalogo_barras (
    ean TEXT PRIMARY KEY,
    nombre TEXT NOT NULL,
    nombre_norm TEXT NOT NULL,
    marca TEXT,
    cantidad REAL,
    unidad TEXT CHECK (unidad IS NULL OR unidad IN ('ml', 'l', 'g', 'kg', 'pzs')),
    categoria TEXT,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_catalogo_nombre_norm ON catalogo_barras(nombre_norm);
CREATE INDEX IF NOT EXISTS idx_catalogo_marca ON catalogo_barras(marca);
CREATE INDEX IF NOT EXISTS idx_catalogo_categoria ON catalogo_barras(categoria);

-- ---------- 3. Auditoria + aprendizaje ----------
CREATE TABLE IF NOT EXISTS vinculos_codigos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ean TEXT,
    producto_id INTEGER NOT NULL REFERENCES productos(id) ON DELETE CASCADE,
    nombre_ticket_crudo TEXT,
    nombre_ticket_norm TEXT,
    origen TEXT NOT NULL CHECK (origen IN ('auto-verde', 'manual-rojo', 'manual-amarillo', 'confirmado-amarillo')),
    score REAL,
    confirmado_por INTEGER REFERENCES usuarios(id) ON DELETE SET NULL,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Un ean + producto solo se vincula una vez (el re-import no duplica).
CREATE UNIQUE INDEX IF NOT EXISTS idx_vinculos_ean_producto
    ON vinculos_codigos(ean, producto_id) WHERE ean IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_vinculos_ean ON vinculos_codigos(ean);
CREATE INDEX IF NOT EXISTS idx_vinculos_producto ON vinculos_codigos(producto_id);
CREATE INDEX IF NOT EXISTS idx_vinculos_origen ON vinculos_codigos(origen);
CREATE INDEX IF NOT EXISTS idx_vinculos_nombre_norm ON vinculos_codigos(nombre_ticket_norm);

-- ---------- 4. Cola semaforo (rojo hoy, amarillo manana) ----------
CREATE TABLE IF NOT EXISTS pendientes_codigos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ean TEXT,
    nombre_crudo TEXT NOT NULL,
    nombre_norm TEXT NOT NULL,
    estado TEXT NOT NULL DEFAULT 'rojo' CHECK (estado IN ('rojo', 'amarillo', 'conflicto', 'resuelto')),
    mejor_candidato_id INTEGER REFERENCES productos(id) ON DELETE SET NULL,
    mejor_score REAL,
    veces_visto INTEGER NOT NULL DEFAULT 1,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP,
    actualizado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Mismo (nombre_norm, ean) = mismo pendiente (veces_visto++ en codigo).
CREATE UNIQUE INDEX IF NOT EXISTS idx_pendientes_nombre_ean
    ON pendientes_codigos(nombre_norm, ean) WHERE ean IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_pendientes_estado ON pendientes_codigos(estado);
CREATE INDEX IF NOT EXISTS idx_pendientes_ean ON pendientes_codigos(ean);
CREATE INDEX IF NOT EXISTS idx_pendientes_nombre_norm ON pendientes_codigos(nombre_norm);
