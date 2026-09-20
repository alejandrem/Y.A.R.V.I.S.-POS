-- ============================================================
-- 0019_google_admin — Login del admin con su cuenta de Google.
--
-- El correo del dueño (para comparar contra el userinfo de OAuth) y
-- el Client ID (para no depender de la variable de entorno en el
-- .exe instalado) viven en la fila del admin. La contraseña local se
-- conserva: sin internet se entra con clave como siempre.
-- ============================================================

ALTER TABLE usuarios ADD COLUMN google_email TEXT DEFAULT '';
ALTER TABLE usuarios ADD COLUMN google_client_id TEXT DEFAULT '';
