// ============================================================
// rutas_modelos_config — Catálogo de modelos y preferencia de
// quant. Porción de rutas_modelos.rs.
// ============================================================

use std::path::PathBuf;

/// Estado verificado de un modelo (espejo del dict de Python).
#[derive(Debug, Clone, PartialEq)]
pub struct InfoModelo {
    pub ruta: PathBuf,
    pub existe: bool,
    pub tamano_mb: f64,
}

// Preferencia de quant (se usa la primera disponible).
pub(crate) const PREFERENCIA_QUANT: &[&str] = &["q4_k_m", "q3_k_l", "q3_k_m", "q4_0", "q5_k_m", "q8_0"];

// Namespaces/orgs reales en HF donde puede vivir el modelo.
pub(crate) const MODELOS_CONFIG: &[(&str, &[&str])] = &[(
    "1.7B",
    &[
        "lmstudio-community/Qwen3-1.7B-GGUF",
        "qwen/Qwen3-1.7B-GGUF",
    ],
)];
