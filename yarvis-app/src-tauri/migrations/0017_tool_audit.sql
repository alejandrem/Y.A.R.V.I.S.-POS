-- ============================================================
-- 0017_tool_audit — Auditoría de herramientas del chat (issue #15).
--
-- Cada tool que ejecuta el chatbot (local o cloud) deja rastro: quién,
-- qué tool con qué args, cuánto tardó y si falló. Best-effort desde
-- Rust (si el INSERT falla, el chat sigue: solo se loguea el warning).
-- ============================================================

CREATE TABLE IF NOT EXISTS tool_audit (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha DATETIME DEFAULT CURRENT_TIMESTAMP,
    tool TEXT NOT NULL,
    args TEXT NOT NULL DEFAULT '{}',
    usuario_id INTEGER,
    rol TEXT NOT NULL DEFAULT 'admin',
    ms INTEGER NOT NULL DEFAULT 0,
    ok INTEGER NOT NULL DEFAULT 1,
    error TEXT
);

CREATE INDEX IF NOT EXISTS idx_tool_audit_fecha ON tool_audit(fecha);
CREATE INDEX IF NOT EXISTS idx_tool_audit_tool ON tool_audit(tool);
CREATE INDEX IF NOT EXISTS idx_tool_audit_usuario ON tool_audit(usuario_id);
