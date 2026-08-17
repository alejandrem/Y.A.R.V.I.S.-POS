//! rutas_modelos.rs — Port de `yarvis-IA/parseador_de_tickets/llm/rutas_modelos.py`
//!
//! Resuelve las rutas de los modelos Qwen locales (LM Studio) detectando el
//! home del usuario. Por qué NO se usan nombres exactos: LM Studio puede bajar
//! el mismo modelo a distintos namespaces (Qwen/…, lmstudio-community/…,
//! unsloth/…) y con cualquier quantización (Q4_K_M, Q3_K_L, …). Entonces se
//! BUSCA el primer `.gguf` real (sin `mmproj`) dentro de los directorios
//! candidatos de cada modelo, con preferencia de quant.
//!
//! Conexión: en Python lo consumen `llm/analizador_llm.py` y
//! `modelos_local/gestion_hardware.py` vía `qwen0_5`, `qwen0_8`, `qwen1_7`.
//! El equivalente Rust se expone con `ruta_modelo` / `qwen0_5`/`qwen0_8`/`qwen1_7`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Estado verificado de un modelo (espejo del dict de Python).
#[derive(Debug, Clone, PartialEq)]
pub struct InfoModelo {
    pub ruta: PathBuf,
    pub existe: bool,
    pub tamano_mb: f64,
}

// Preferencia de quant (se usa la primera disponible).
const PREFERENCIA_QUANT: &[&str] = &["q4_k_m", "q3_k_l", "q3_k_m", "q4_0", "q5_k_m", "q8_0"];

// Namespaces/orgs reales en HF donde puede vivir cada modelo.
const MODELOS_CONFIG: &[(&str, &[&str])] = &[
    (
        "0.5B",
        &[
            "Qwen/Qwen2.5-0.5B-Instruct-GGUF",
            "lmstudio-community/Qwen2.5-0.5B-Instruct-GGUF",
        ],
    ),
    (
        "0.8B",
        &["unsloth/Qwen3.5-0.8B-GGUF", "Qwen/Qwen3-0.6B-GGUF"],
    ),
    (
        "1.7B",
        &[
            "lmstudio-community/Qwen3-1.7B-GGUF",
            "qwen/Qwen3-1.7B-GGUF",
        ],
    ),
];

/// Detecta el home del usuario (Linux/macOS: HOME; Windows: USERPROFILE).
fn obtener_dir_lmstudio() -> PathBuf {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    Path::new(&home).join(".lmstudio").join("models")
}

/// Devuelve la primera ruta `.gguf` (no `mmproj`) del directorio del modelo.
fn buscar_gguf(base: &Path, rel_path: &str) -> Option<PathBuf> {
    let folder = base.join(rel_path);

    if !folder.is_dir() {
        return None;
    }

    let mut archivos_gguf: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let es_gguf = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase() == "gguf")
                .unwrap_or(false);
            let es_mmproj = path
                .file_name()
                .map(|f| f.to_string_lossy().to_lowercase().contains("mmproj"))
                .unwrap_or(false);
            if es_gguf && !es_mmproj {
                archivos_gguf.push(path);
            }
        }
    }

    if archivos_gguf.is_empty() {
        return None;
    }

    for preferida in PREFERENCIA_QUANT {
        for archivo in &archivos_gguf {
            let nombre = archivo.file_name().map(|f| f.to_string_lossy().to_lowercase());
            if let Some(nombre) = nombre {
                if nombre.contains(preferida) {
                    return Some(archivo.clone());
                }
            }
        }
    }

    archivos_gguf.into_iter().next()
}

/// Busca en los directorios candidatos; fallback a la ruta predecible
/// `modelo_no_encontrado.gguf` (mismo comportamiento que Python).
fn resolver(candidatos: &[&str], base: &Path) -> PathBuf {
    for rel in candidatos {
        if let Some(ruta) = buscar_gguf(base, rel) {
            return ruta;
        }
    }

    base.join(candidatos[0]).join("modelo_no_encontrado.gguf")
}

fn verificar_en(base: &Path) -> Vec<(&'static str, InfoModelo)> {
    let mut resultado = Vec::new();

    for &(key, candidatos) in MODELOS_CONFIG {
        let ruta = resolver(candidatos, base);
        let existe = ruta.exists();
        let mut tamano_mb = 0.0;

        if existe {
            if let Ok(metadata) = fs::metadata(&ruta) {
                tamano_mb = (metadata.len() as f64 / (1024.0 * 1024.0) * 10.0).round() / 10.0;
            }
        }

        resultado.push((key, InfoModelo { ruta, existe, tamano_mb }));
    }

    resultado
}

// ---------------------------------------------------------------------------
// API pública (conexión con analizador_llm / gestion_hardware)
// ---------------------------------------------------------------------------

/// Ruta resuelta del modelo para la clave "0.5B" | "0.8B" | "1.7B".
pub fn ruta_modelo(clave: &str) -> PathBuf {
    let base = obtener_dir_lmstudio();
    let (_, candidatos) = MODELOS_CONFIG
        .iter()
        .find(|(k, _)| *k == clave)
        .expect("clave de modelo inválida (0.5B/0.8B/1.7B)");
    resolver(candidatos, &base)
}

pub fn qwen0_5() -> PathBuf {
    ruta_modelo("0.5B")
}

pub fn qwen0_8() -> PathBuf {
    ruta_modelo("0.8B")
}

pub fn qwen1_7() -> PathBuf {
    ruta_modelo("1.7B")
}

/// Verifica que los archivos de modelo existan y retorna su estado.
pub fn verificar_modelos() -> Vec<(&'static str, InfoModelo)> {
    verificar_en(&obtener_dir_lmstudio())
}

// ---------------------------------------------------------------------------
// Tests (con base temporal verificable, iguales a lo que Python devuelve)
// ---------------------------------------------------------------------------

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
        let dir = base.join("Qwen/Qwen2.5-0.5B-Instruct-GGUF");
        crear_gguf(&dir, "model-q8_0.gguf");
        let preferida = crear_gguf(&dir, "model-q4_k_m.gguf");

        let ruta = buscar_gguf(&base, "Qwen/Qwen2.5-0.5B-Instruct-GGUF").unwrap();
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
        std::fs::create_dir_all(base.join("Qwen/Qwen2.5-0.5B-Instruct-GGUF")).unwrap();

        assert!(buscar_gguf(&base, "Qwen/Qwen2.5-0.5B-Instruct-GGUF").is_none());
        assert!(buscar_gguf(&base, "Qwen/NoExiste-GGUF").is_none());

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn fallback_completo_si_nada_coincide() {
        let base = tmp_base("fallback");
        let ruta = resolver(MODELOS_CONFIG[0].1, &base);
        assert!(!ruta.exists());
        assert!(
            ruta.ends_with("Qwen2.5-0.5B-Instruct-GGUF/modelo_no_encontrado.gguf"),
            "fallback equivocado: {}",
            ruta.display()
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn verificar_en_reporta_tamano_y_existencia() {
        let base = tmp_base("verificar");
        let dir = base.join("Qwen/Qwen2.5-0.5B-Instruct-GGUF");
        crear_gguf(&dir, "model-q4_k_m.gguf");

        let res = verificar_en(&base);
        let (_, info) = &res[0];
        assert!(info.existe);
        assert!(info.tamano_mb > 0.0);
        assert_eq!(info.ruta.file_name().unwrap().to_string_lossy(), "model-q4_k_m.gguf");

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn segundo_candidato_se_usa_cuando_primero_no_existe() {
        let base = tmp_base("candidatos");
        // Solo el 2º candidato del 0.5B existe.
        let dir = base.join("lmstudio-community/Qwen2.5-0.5B-Instruct-GGUF");
        let real = crear_gguf(&dir, "model-q4_k_m.gguf");

        let candidatos = &MODELOS_CONFIG[0].1;
        let ruta = resolver(candidatos, &base);

        assert_eq!(ruta, real);
        assert!(ruta.exists());

        let _ = std::fs::remove_dir_all(&base);
    }
}