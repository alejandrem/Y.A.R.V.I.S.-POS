// ============================================================
// rutas_modelos_api — API pública de resolución de rutas de
// modelos (conexión con analizador_llm / gestion_hardware).
// Porción de rutas_modelos.rs + sus tests.
// ============================================================

use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use super::rutas_modelos_config::{InfoModelo, MODELOS_CONFIG};
#[cfg(test)]
use super::rutas_modelos_detect::buscar_gguf;
use super::rutas_modelos_detect::{obtener_dir_lmstudio, resolver, verificar_en};

/// Ruta GGUF elegida explícitamente por el usuario desde la configuración.
/// Cuando existe, se comparte entre el parser y el chat local.
static RUTA_MODELO_PERSONALIZADA: OnceLock<RwLock<Option<PathBuf>>> = OnceLock::new();

fn ruta_personalizada() -> &'static RwLock<Option<PathBuf>> {
    RUTA_MODELO_PERSONALIZADA.get_or_init(|| RwLock::new(None))
}

/// Ruta resuelta del modelo local "1.7B" (único del Y.A.R.V.I.S.; lo usan
/// el parseo de tickets y la conversación local).
pub fn ruta_modelo(clave: &str) -> PathBuf {
    if let Some(ruta) = ruta_personalizada()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
    {
        return ruta;
    }

    let base = obtener_dir_lmstudio();
    let (_, candidatos) = MODELOS_CONFIG
        .iter()
        .find(|(k, _)| *k == clave)
        .expect("clave de modelo inválida (1.7B)");
    resolver(candidatos, &base)
}

/// Configura una ruta GGUF personalizada para el modelo local.
/// `None` restaura la detección automática de LM Studio.
pub fn configurar_ruta_modelo(ruta: Option<PathBuf>) -> Result<(), String> {
    let validada = ruta
        .map(|ruta| {
            if !ruta.is_file() {
                return Err(format!(
                    "El archivo del modelo no existe: {}",
                    ruta.display()
                ));
            }
            if ruta
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase())
                != Some("gguf".to_string())
            {
                return Err("El modelo local debe ser un archivo .gguf".to_string());
            }
            std::fs::canonicalize(&ruta)
                .map(Some)
                .map_err(|e| format!("No se pudo resolver la ruta del modelo: {e}"))
        })
        .transpose()?
        .flatten();

    *ruta_personalizada()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = validada;
    Ok(())
}

/// Devuelve la ruta personalizada si el usuario configuró una.
pub fn ruta_modelo_personalizada() -> Option<PathBuf> {
    ruta_personalizada()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

pub fn qwen1_7() -> PathBuf {
    ruta_modelo("1.7B")
}

/// Verifica que los archivos de modelo existan y retorna su estado.
pub fn verificar_modelos() -> Vec<(&'static str, InfoModelo)> {
    verificar_en(&obtener_dir_lmstudio())
}

#[cfg(test)]
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_base(nombre: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("rutas_modelos_{}_{}", nombre, nanos));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn crear_gguf(dir: &Path, nombre: &str) -> PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        let p = dir.join(nombre);
        std::fs::write(&p, make_blob(100)).unwrap();
        p
    }

    // Contenido de ~100 KB para que el tamaño en MB no sea 0.0.
    fn make_blob(mb: usize) -> Vec<u8> {
        vec![0u8; 1024 * 1024 * mb / 10]
    }

    #[test]
    fn elegida_quant_preferida_q4_k_m() {
        let base = tmp_base("quant");
        let dir = base.join("lmstudio-community/Qwen3-1.7B-GGUF");
        crear_gguf(&dir, "model-q8_0.gguf");
        let preferida = crear_gguf(&dir, "model-q4_k_m.gguf");

        let ruta = buscar_gguf(&base, "lmstudio-community/Qwen3-1.7B-GGUF").unwrap();
        assert_eq!(ruta, preferida);

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn mmproj_nunca_se_elige() {
        let base = tmp_base("mmproj");
        let dir = base.join("lmstudio-community/Qwen3-1.7B-GGUF");
        crear_gguf(&dir, "mmproj-model-q4_k_m.gguf");
        let real = crear_gguf(&dir, "Qwen3-1.7B-Instruct-q5_k_m.gguf");

        let ruta = buscar_gguf(&base, "lmstudio-community/Qwen3-1.7B-GGUF").unwrap();
        assert_eq!(ruta, real);

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn carpeta_vacia_o_inexistente_devuelve_none() {
        let base = tmp_base("vacia");
        std::fs::create_dir_all(base.join("lmstudio-community/Qwen3-1.7B-GGUF")).unwrap();

        assert!(buscar_gguf(&base, "lmstudio-community/Qwen3-1.7B-GGUF").is_none());
        assert!(buscar_gguf(&base, "Qwen/NoExiste-GGUF").is_none());

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn fallback_completo_si_nada_coincide() {
        let base = tmp_base("fallback");
        let ruta = resolver(MODELOS_CONFIG[0].1, &base);
        assert!(!ruta.exists());
        assert!(
            ruta.ends_with("Qwen3-1.7B-GGUF/modelo_no_encontrado.gguf"),
            "fallback equivocado: {}",
            ruta.display()
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn verificar_en_reporta_tamano_y_existencia() {
        let base = tmp_base("verificar");
        let dir = base.join("lmstudio-community/Qwen3-1.7B-GGUF");
        crear_gguf(&dir, "model-q4_k_m.gguf");

        let res = verificar_en(&base);
        let (_, info) = &res[0];
        assert!(info.existe);
        assert!(info.tamano_mb > 0.0);
        assert_eq!(
            info.ruta.file_name().unwrap().to_string_lossy(),
            "model-q4_k_m.gguf"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn segundo_candidato_se_usa_cuando_primero_no_existe() {
        let base = tmp_base("candidatos");
        // Solo el 2º candidato del 1.7B existe.
        let dir = base.join("qwen/Qwen3-1.7B-GGUF");
        let real = crear_gguf(&dir, "model-q4_k_m.gguf");

        let candidatos = &MODELOS_CONFIG[0].1;
        let ruta = resolver(candidatos, &base);

        assert_eq!(ruta, real);
        assert!(ruta.exists());

        let _ = std::fs::remove_dir_all(&base);
    }
}
