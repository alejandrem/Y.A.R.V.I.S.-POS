// ============================================================
// analizador_modelos — Carga, caché y descarga de modelos
// llama.cpp (feature `llm-local`). Stubs sin el feature.
// Porción de analizador_llm.rs.
// ============================================================

#[cfg(feature = "llm-local")]
use std::collections::HashMap;
#[cfg(feature = "llm-local")]
use std::path::PathBuf;
#[cfg(feature = "llm-local")]
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(feature = "llm-local")]
use super::analizador_prompt::NOMBRES_MODELO;
#[cfg(feature = "llm-local")]
use super::rutas_modelos_api::qwen1_7;

#[cfg(feature = "llm-local")]
fn ruta_modelo(_clave: &str) -> PathBuf {
    qwen1_7()
}

#[cfg(feature = "llm-local")]
use llama_cpp_4::prelude::*;

#[cfg(feature = "llm-local")]
pub type Resultado<T> = std::result::Result<T, String>;

#[cfg(feature = "llm-local")]
pub(crate) const N_CTX: u32 = 4096;
#[cfg(feature = "llm-local")]
pub(crate) const N_BATCH: usize = 512;
#[cfg(feature = "llm-local")]
/// Techo de generación para el CHAT (la respuesta de un turno no debe cortarse).
pub(crate) const MAX_TOKENS: i32 = 2048;
#[cfg(feature = "llm-local")]
/// Techo de generación para el PARSEO: el JSON del mapeo cabe en <500 tokens,
/// pero el Qwen3 puede razonar (bloque `<thinking>` ~200-400 tokens) antes de
/// responder, así que 1536 cubre "pensamiento + JSON" sin tocar el techo.
pub(crate) const MAX_TOKENS_PARSEO: i32 = 1536;
#[cfg(feature = "llm-local")]
pub(crate) const TEMPERATURA: f32 = 0.1;
#[cfg(feature = "llm-local")]
pub(crate) const TOP_P: f32 = 0.9;

/// Núcleos para llama.cpp, igual que el default de llama-cpp-python (`os.cpu_count`)
/// pero acotado a 8 para no sobresaturar la app de escritorio (Tauri + SQLite).
/// Sobreescritible por la env `YARVIS_LLM_THREADS`.
#[cfg(feature = "llm-local")]
pub(crate) fn n_threads_llm() -> i32 {
    if let Ok(v) = std::env::var("YARVIS_LLM_THREADS") {
        if let Ok(n) = v.trim().parse::<i32>() {
            return n.max(1);
        }
    }
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(8) as i32
}

#[cfg(feature = "llm-local")]
fn gpu_layers() -> u32 {
    std::env::var("YARVIS_GPU_LAYERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

#[cfg(feature = "llm-local")]
pub struct ModeloChat {
    pub(crate) model: LlamaModel,
}

/// `LlamaBackend::init()` solo se puede llamar UNA vez por proceso (devuelve
/// `BackendAlreadyInitialized` en llamadas posteriores), así que se comparte un
/// único backend global para todos los modelos de la caché.
#[cfg(feature = "llm-local")]
static BACKEND: OnceLock<LlamaBackend> = OnceLock::new();

/// Devuelve el backend global inicializándolo la primera vez.
#[cfg(feature = "llm-local")]
pub(crate) fn backend_global() -> Resultado<&'static LlamaBackend> {
    if BACKEND.get().is_none() {
        let backend =
            LlamaBackend::init().map_err(|e| format!("No se pudo iniciar llama.cpp: {e}"))?;
        let _ = BACKEND.set(backend);
    }
    BACKEND
        .get()
        .ok_or_else(|| "llama.cpp no inicializado".to_string())
}

/// Caché global de modelos cargados. El Y.A.R.V.I.S. usa un SOLO modelo
/// local: el Qwen 3 1.7B, compartido entre el parseo de tickets y la
/// conversación (1 solo GGUF en RAM). Igual que el dict `_MODELOS_LLM` de
/// Python. `send`+`sync` porque los tipos lo son.
#[cfg(feature = "llm-local")]
static CACHE: OnceLock<Mutex<HashMap<String, Arc<ModeloChat>>>> = OnceLock::new();

/// Serializa la inferencia app-wide (espejo del `_MODEL_LOCK` global de
/// llama-cpp-python): un mismo modelo no se usa desde dos hilos a la vez.
/// Cargar dos modelos distintos entre sí no queda bloqueado.
#[cfg(feature = "llm-local")]
pub(crate) static INFERENCIA_LOCK: Mutex<()> = Mutex::new(());

/// Carga el modelo local Qwen 3 1.7B o devuelve el ya cargado. Es un port de
/// `analizador_llm::_cargar_modelo` + `puede_cargar_modelo`. Se expone `pub`
/// para que el chat local (`motor-chat/llm`) reutilice el MISMO caché: el
/// parseo y el chat comparten el 1.7B (1 solo GGUF, no se duplica).
#[cfg(feature = "llm-local")]
pub fn cargar_modelo(clave: &str) -> Resultado<Arc<ModeloChat>> {
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // Fast path: ya cargado.
    if let Some(m) = cache.lock().unwrap().get(clave) {
        return Ok(Arc::clone(m));
    }

    // Load path con double-checked locking.
    let mut guard = cache
        .lock()
        .map_err(|_| "cache de modelos envenenado".to_string())?;
    if let Some(m) = guard.get(clave) {
        return Ok(Arc::clone(m));
    }

    let ruta = ruta_modelo(clave);
    if !ruta.exists() {
        return Err(format!(
            "Modelo no encontrado: {}. Descárgalo con LM Studio.",
            ruta.display()
        ));
    }

    let nombre = NOMBRES_MODELO
        .iter()
        .find(|(k, _)| *k == clave)
        .map(|(_, n)| *n)
        .unwrap_or(clave);
    tracing::info!("[YARVIS-IA] Cargando {nombre} para parseo de tickets...");
    let inicio = std::time::Instant::now();

    let backend = backend_global()?;
    let params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers());
    let model = LlamaModel::load_from_file(backend, &ruta, &params)
        .map_err(|e| format!("No se pudo cargar {} ({e})", ruta.display()))?;

    tracing::info!(
        "[YARVIS-IA] {nombre} listo. Carga: {:.1}s, hilos: {}.",
        inicio.elapsed().as_secs_f32(),
        n_threads_llm()
    );
    let modelo = Arc::new(ModeloChat { model });
    guard.insert(clave.to_string(), Arc::clone(&modelo));
    Ok(modelo)
}

/// Libera todos los modelos de RAM/VRAM. Espejo de `descargar_modelos`.
#[cfg(feature = "llm-local")]
pub fn descargar_modelos() -> usize {
    let count = CACHE
        .get()
        .map(|cache| {
            let mut guard = cache.lock().unwrap_or_else(|p| p.into_inner());
            let n = guard.len();
            guard.clear();
            n
        })
        .unwrap_or(0);
    if count > 0 {
        tracing::info!("[YARVIS-IA] {count} modelo(s) descargado(s) de VRAM.");
    }
    count
}

/// ¿Está cargada la clave indicada en el caché? (consume `motor-chat/llm`).
#[cfg(feature = "llm-local")]
pub fn modelo_cargado(clave: &str) -> bool {
    CACHE
        .get()
        .map(|cache| {
            cache
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .contains_key(clave)
        })
        .unwrap_or(false)
}

/// Descarga SOLO la clave indicada del caché (devuelve true si estaba cargada).
#[cfg(feature = "llm-local")]
pub fn descargar_modelo(clave: &str) -> bool {
    let quitado = CACHE
        .get()
        .map(|cache| {
            cache
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .remove(clave)
                .is_some()
        })
        .unwrap_or(false);
    if quitado {
        tracing::info!("[YARVIS-IA] Modelo {clave} descargado de RAM/VRAM.");
    }
    quitado
}

#[cfg(not(feature = "llm-local"))]
pub fn descargar_modelos() -> usize {
    0
}

// Stubs de API para el build sin `llm-local`: `motor-chat` los importa solo
// con el feature activo, así que aquí quedan sin uso → se silencian.
#[cfg(not(feature = "llm-local"))]
#[allow(dead_code)]
pub(crate) fn modelo_cargado(_clave: &str) -> bool {
    false
}

#[cfg(not(feature = "llm-local"))]
#[allow(dead_code)]
pub(crate) fn descargar_modelo(_clave: &str) -> bool {
    false
}
