// ============================================================
// lectura — Vista previa de UN corte sin guardar (paso Revisión).
// ============================================================

use crate::backventanas::auth::AuthState;
use src_ia::parseador_de_cortes::{parse_corte, CorteParseado};

#[tauri::command]
pub async fn previsualizar_corte(
    auth: tauri::State<'_, AuthState>,
    path: String,
) -> Result<CorteParseado, String> {
    auth.require_admin()?;
    let safe = super::super::utils::sanitize_path(&path)?;
    let contenido = std::fs::read_to_string(safe).map_err(|e| e.to_string())?;
    parse_corte(&contenido)
}
