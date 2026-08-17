//! analizador_llm.rs — Port de `yarvis-IA/parseador_de_tickets/llm/analizador_llm.py`
//!
//! Parsing de tickets mediante LLM local (Qwen GGUF) con `llama-cpp-4`.
//! Escalada de confianza idéntica a Python:
//!   1. Carga el 0.5B; si la confianza es < 0.8, reintenta con el 0.8B.
//!   2. Si la confianza sigue < 0.8, reintenta con el 1.7B.
//!   3. Si un modelo no devuelve JSON válido, se salta directo al siguiente.
//!
//! La inferencia llama.cpp queda detrás del feature `llm-local` (off por
//! defecto): así el núcleo compila rápido en CI y solo el backend Tauri
//! (o quien la active) paga el costo de compilar llama.cpp.

// Estas imports y constantes solo existen si el feature `llm-local` está
// activo; si no, quedarían como "unused" y romperían clippy en el modo
// reducido (el que compila rápido en CI / sin llama.cpp).
#[cfg(feature = "llm-local")]
use std::collections::HashMap;
#[cfg(feature = "llm-local")]
use std::path::PathBuf;
#[cfg(feature = "llm-local")]
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(feature = "llm-local")]
use super::rutas_modelos::{qwen0_5, qwen0_8, qwen1_7};

#[cfg(feature = "llm-local")]
use std::num::NonZeroU32;

// ---------------------------------------------------------------------------
// Prompt del sistema (espejo de SYSTEM_PROMPT de Python)
// ---------------------------------------------------------------------------

pub const SISTEMA_PROMPT: &str = r#"Eres un experto en parseo de tickets de punto de venta mexicano.
Analiza el siguiente ticket de texto plano y extrae la estructura.

Reglas:
- Identifica qué columna es: cantidad, producto, precio unitario, total
- Los precios siempre tienen $ o están en formato decimal (15.00)
- La cantidad siempre es un número entero al inicio de la línea
- El total es siempre la última columna numérica
- El nombre del producto es texto entre la cantidad y los precios
- Detecta si hay descuentos, impuestos (IVA), o notas extra
- BUSCA la fecha del ticket: puede estar en formatos como "15/03/2024", "2024-03-15",
  "15 de marzo de 2024", "Mar 15 2024", "Fecha: 15/03/24", "15-03-2024", etc.
- BUSCA la hora del ticket: puede estar en formatos como "14:32", "14:32:05",
  "2:32 PM", "Hora: 14:32", etc.
- Si no encuentras fecha u hora, devuelve null para esos campos.

Responde SOLO con JSON válido, sin explicaciones.

FORMATO DE RESPUESTA:
{
  "mapeo": {
    "formato_detectado": "CANTIDAD PRODUCTO PRECIO TOTAL",
    "columnas": {
      "cantidad": INDICE,
      "producto": INDICE,
      "precio_unitario": INDICE,
      "total": INDICE,
      "descuento": INDICE_O_NULL
    },
    "delimitador": "espacios_multiples",
    "moneda": "$",
    "total_columnas": NUMERO,
    "tiene_descuento": true_o_false,
    "tiene_iva": true_o_false
  },
  "fecha_ticket": "YYYY-MM-DD_O_NULL",
  "hora_ticket": "HH:MM_O_NULL",
  "ejemplo_parseado": [
    {
      "cantidad": NUMERO_ENTERO,
      "producto": "TEXTO LIMPIO",
      "precio_unitario": NUMERO_DECIMAL,
      "total": NUMERO_DECIMAL,
      "descuento": NUMERO_O_NULL
    }
  ],
  "confianza": NUMERO_ENTRE_0_Y_1,
  "notas": "EXPLICACION DEL FORMATO"
}"#;

// Nombre de los modelos para los mensajes (espejo de _NOMBRES_MODELO).
#[cfg(feature = "llm-local")]
const NOMBRES_MODELO: [(&str, &str); 3] = [
    ("0.5B", "Qwen 2.5 0.5B"),
    ("0.8B", "Qwen 3.5 0.8B"),
    ("1.7B", "Qwen 3 1.7B"),
];

// ---------------------------------------------------------------------------
// Rutas (espejo de _RUTAS_MODELO)
// ---------------------------------------------------------------------------

#[cfg(feature = "llm-local")]
fn ruta_modelo(clave: &str) -> PathBuf {
    match clave {
        "0.5B" => qwen0_5(),
        "0.8B" => qwen0_8(),
        _ => qwen1_7(),
    }
}

// ---------------------------------------------------------------------------
// Extracción de JSON de la respuesta del modelo (espejo de _extraer_json:
// regex `\{[\s\S]*\}` → primer `{` hasta el último `}`).
// ---------------------------------------------------------------------------

pub fn extraer_json(respuesta: &str) -> Option<serde_json::Value> {
    let inicio = respuesta.find('{')?;
    let fin = respuesta.rfind('}')?;
    let candidato = &respuesta[inicio..=fin];
    serde_json::from_str(candidato).ok()
}

/// Inserta `"status": "ok"` dentro del JSON del modelo (espejo de
/// `{ "status": "ok", **resultado }` de Python; Rust no tiene spread en `json!`).
#[cfg(feature = "llm-local")]
fn con_status_ok(mut valor: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = valor.as_object_mut() {
        obj.insert("status".to_string(), serde_json::json!("ok"));
    }
    valor
}

// ===========================================================================
// Inferencia llama.cpp (feature `llm-local`)
// ===========================================================================

#[cfg(feature = "llm-local")]
use llama_cpp_4::prelude::*;

/// Resultado con error `String` propio del módulo. El `use prelude::*` de
/// llama-cpp-4 trae su propio alias `Result<T>` (uno solo genérico), así que
/// este alias evita la colisión con `std::result::Result`.
#[cfg(feature = "llm-local")]
type Resultado<T> = std::result::Result<T, String>;

#[cfg(feature = "llm-local")]
const N_CTX: u32 = 4096;
#[cfg(feature = "llm-local")]
const N_BATCH: usize = 512;
#[cfg(feature = "llm-local")]
const N_THREADS: i32 = 4;
#[cfg(feature = "llm-local")]
const MAX_TOKENS: i32 = 2048;
#[cfg(feature = "llm-local")]
const TEMPERATURA: f32 = 0.1;
#[cfg(feature = "llm-local")]
const TOP_P: f32 = 0.9;

/// Lee el número de capas a descargar a GPU desde el entorno (default CPU).
/// El binario debe compilarse con CUDA (feature `cuda` de llama-cpp-4) para
/// que un valor > 0 funcione.
#[cfg(feature = "llm-local")]
fn gpu_layers() -> u32 {
    std::env::var("YARVIS_GPU_LAYERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

/// Modelo cargado: backend (singleton del proceso) + modelo inmutable. Se
/// comparte con `Arc` porque `LlamaModel` no se puede clonar (es una envoltura
/// sobre un puntero C). El contexto se crea POR LLAMADA (cada contexto tiene
/// su propio KV cache; varios contextos sobre el mismo modelo son seguros en
/// llama.cpp).
#[cfg(feature = "llm-local")]
pub(crate) struct ModeloChat {
    model: LlamaModel,
}

/// `LlamaBackend::init()` solo se puede llamar UNA vez por proceso (devuelve
/// `BackendAlreadyInitialized` en llamadas posteriores), así que se comparte un
/// único backend global para todos los modelos de la caché.
#[cfg(feature = "llm-local")]
static BACKEND: OnceLock<LlamaBackend> = OnceLock::new();

/// Devuelve el backend global inicializándolo la primera vez.
#[cfg(feature = "llm-local")]
fn backend_global() -> Resultado<&'static LlamaBackend> {
    if BACKEND.get().is_none() {
        let backend = LlamaBackend::init()
            .map_err(|e| format!("No se pudo iniciar llama.cpp: {e}"))?;
        let _ = BACKEND.set(backend);
    }
    BACKEND.get().ok_or_else(|| "llama.cpp no inicializado".to_string())
}

/// Caché global de modelos cargados (0.5B / 0.8B / 1.7B), igual que el dict
/// `_MODELOS_LLM` de Python. `send`+`sync` porque los tipos lo son.
#[cfg(feature = "llm-local")]
static CACHE: OnceLock<Mutex<HashMap<String, Arc<ModeloChat>>>> = OnceLock::new();

/// Serializa la inferencia app-wide (espejo del `_MODEL_LOCK` global de
/// llama-cpp-python): un mismo modelo no se usa desde dos hilos a la vez.
/// Cargar dos modelos distintos entre sí no queda bloqueado.
#[cfg(feature = "llm-local")]
static INFERENCIA_LOCK: Mutex<()> = Mutex::new(());

/// Carga el modelo Qwen indicado (0.5B / 0.8B / 1.7B) o devuelve el ya
/// cargado. Es un port de `analizador_llm::_cargar_modelo` + `puede_cargar_modelo`.
/// Se expone `pub(crate)` para que el chat local (`motor-chat/llm`) reutilice
/// el MISMO caché: así el 1.7B de conversación y el del parseo comparten
/// instancia y no se duplican en VRAM.
#[cfg(feature = "llm-local")]
pub(crate) fn cargar_modelo(clave: &str) -> Resultado<Arc<ModeloChat>> {
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // Fast path: ya cargado.
    if let Some(m) = cache.lock().unwrap().get(clave) {
        return Ok(Arc::clone(m));
    }

    // Load path con double-checked locking.
    let mut guard = cache.lock().map_err(|_| "cache de modelos envenenado".to_string())?;
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
    println!("[YARVIS-IA] Cargando {nombre} para parseo de tickets...");

    let backend = backend_global()?;
    let params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers());
    let model = LlamaModel::load_from_file(backend, &ruta, &params)
        .map_err(|e| format!("No se pudo cargar {} ({e})", ruta.display()))?;

    println!("[YARVIS-IA] {nombre} listo.");
    let modelo = Arc::new(ModeloChat { model });
    guard.insert(clave.to_string(), Arc::clone(&modelo));
    Ok(modelo)
}

/// Genera texto completo dado el prompt ya formateado con el chat template.
#[cfg(feature = "llm-local")]
fn generar(modelo: &ModeloChat, prompt: &str) -> Resultado<String> {
    let model = &modelo.model;
    let backend = backend_global()?;

    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(N_CTX))
        .with_n_batch(N_BATCH as u32)
        .with_n_threads(N_THREADS)
        .with_n_threads_batch(N_THREADS);
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| format!("No se pudo crear el contexto: {e}"))?;

    let tokens = model
        .str_to_token(prompt, AddBos::Always)
        .map_err(|e| format!("No se pudo tokenizar el prompt: {e}"))?;
    let n_prompt = tokens.len() as i32;
    if n_prompt >= N_CTX as i32 {
        return Err("El prompt excede n_ctx".to_string());
    }

    // Prefill por CHUNKS de N_BATCH: el prompt (SISTEMA_PROMPT + ticket)
    // suele superar las 512 posiciones y `add` falla con "Insufficient Space"
    // si no cabe en un solo batch (llama.cpp decodifica en lotes del tamaño
    // de `n_batch`).
    let mut batch = LlamaBatch::new(N_BATCH, 1);
    let ultimo_token = tokens.len() - 1;
    for (start, seg) in tokens.chunks(N_BATCH).enumerate() {
        batch.clear();
        for (idx, token) in seg.iter().enumerate() {
            let pos = start * N_BATCH + idx;
            let es_ultimo = pos == ultimo_token;
            batch
                .add(*token, pos as i32, &[0], es_ultimo)
                .map_err(|e| format!("Error llenando batch del prompt: {e}"))?;
        }
        ctx.decode(&mut batch).map_err(|e| format!("decode(prompt) falló: {e}"))?;
    }

    let sampler = LlamaSampler::chain_simple([
        // Mismo conjunto de samplers por defecto que llama-cpp-python (el
        // resto de create_chat_completion quedó en defaults): top_k=40,
        // repeat_penalty=1.1, min_p=0.05 (además de temp/top_p del .py).
        LlamaSampler::penalties_simple(64, 1.1),
        LlamaSampler::top_k(40),
        LlamaSampler::min_p(0.05, 1),
        LlamaSampler::top_p(TOP_P, 1),
        LlamaSampler::temp(TEMPERATURA),
        LlamaSampler::dist(0),
    ]);

    // Se acumulan los bytes de todos los tokens y se decodifica UTF-8 una sola
    // vez al final: llama.cpp puede partir un carácter multi-byte entre dos
    // tokens, y `String::from_utf8_lossy` aplicado al buffer completo lo
    // reconstruye igual que el `decode('utf-8')` de Python (sin dep externa).
    let mut salida = Vec::with_capacity(MAX_TOKENS as usize * 4);
    let llena = n_prompt + MAX_TOKENS;
    // Posición del siguiente token = longitud TOTAL del prompt (no el tamaño
    // del último chunk del prefill; si no, el token se solapa con el KV cache
    // y llama_decode devuelve -1).
    let mut n_cur = n_prompt;

    while n_cur < llena {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        if model.is_eog_token(token) {
            break;
        }

        let bytes = model
            .token_to_bytes(token, Special::Tokenize)
            .map_err(|e| format!("Error decodificando token: {e}"))?;
        salida.extend_from_slice(&bytes);

        batch.clear();
        batch
            .add(token, n_cur, &[0], true)
            .map_err(|e| format!("Error llenando batch de generación: {e}"))?;
        n_cur += 1;
        ctx.decode(&mut batch).map_err(|e| format!("decode(generación) falló: {e}"))?;
    }

    Ok(String::from_utf8_lossy(&salida).into_owned())
}

/// Aplica el chat template a los mensajes y genera la respuesta bajo el lock
/// global de inferencia (compartido con el parseo de tickets: llama.cpp no
/// tolera dos generaciones a la vez sobre el mismo backend).
///
/// Lo consume el chat local (`motor-chat/llm`) para el modelo 1.7B de
/// conversación, reutilizando el MISMO caché de modelos del parseo.
#[cfg(feature = "llm-local")]
pub(crate) fn generar_bajo_lock(
    modelo: &Arc<ModeloChat>,
    messages: &[LlamaChatMessage],
) -> Resultado<String> {
    let prompt = modelo
        .model
        .apply_chat_template(None, messages, true)
        .map_err(|e| format!("No se pudo aplicar el chat template: {e}"))?;
    let _lock = INFERENCIA_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    generar(modelo, &prompt)
}

/// Ejecuta un análisis sobre el modelo indicado (espejo de `_ejecutar_analisis`).
#[cfg(feature = "llm-local")]
fn ejecutar_analisis(modelo: &Arc<ModeloChat>, texto: &str) -> Option<serde_json::Value> {
    let lineas: Vec<&str> = texto
        .trim()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    let texto_analizar = lineas[..lineas.len().min(20)].join("\n");

    let user_prompt = format!(
        "TICKET A ANALIZAR:\n---\n{texto_analizar}\n---\n\nAnaliza este ticket y responde SOLAMENTE con el JSON válido."
    );

    let messages = vec![
        LlamaChatMessage::new("system".to_string(), SISTEMA_PROMPT.to_string()).ok()?,
        LlamaChatMessage::new("user".to_string(), user_prompt).ok()?,
    ];

    let prompt = modelo
        .model
        .apply_chat_template(None, &messages, true)
        .ok()?;

    // La inferencia está serializada por el lock global (llama-cpp-python igual).
    let _lock = INFERENCIA_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let contenido = generar(modelo, &prompt).ok()?;
    extraer_json(&contenido)
}

/// Carga un modelo y, si falla, hace visible el error (sin romper la escalada).
#[cfg(feature = "llm-local")]
fn cargar_con_log(clave: &str) -> Option<Arc<ModeloChat>> {
    match cargar_modelo(clave) {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("[YARVIS-IA] No se pudo cargar {clave}: {e}");
            None
        }
    }
}

/// Escalada de confianza completa. Espejo 1:1 de `analizador_llm.analizar_ticket`.
#[cfg(feature = "llm-local")]
pub fn analizar_ticket(texto_ticket: &str) -> serde_json::Value {
    if texto_ticket.trim().is_empty() {
        return serde_json::json!({ "status": "error", "error": "El texto del ticket está vacío" });
    }

    let resultado_0_5 = match cargar_modelo("0.5B") {
        Ok(m) => ejecutar_analisis(&m, texto_ticket),
        Err(e) => return serde_json::json!({ "status": "error", "error": format!("Error al analizar ticket: {e}") }),
    };

    // Intento 1: Qwen 2.5 0.5B con resultado con "mapeo".
    if let Some(mut resultado) = resultado_0_5 {
        if resultado.get("mapeo").is_some() {
            let confianza = resultado.get("confianza").and_then(|c| c.as_f64()).unwrap_or(0.0);
            resultado["confianza"] = serde_json::json!(confianza);

            // Intento 2: confianza < 0.8 → Qwen 3.5 0.8B.
            if confianza < 0.8 {
                println!("[YARVIS-IA] Confianza baja ({confianza}), reintentando con Qwen 3.5 0.8B...");
                if let Some(m_0_8) = cargar_con_log("0.8B") {
                    if let Some(mut r_0_8) = ejecutar_analisis(&m_0_8, texto_ticket) {
                        if r_0_8.get("mapeo").is_some() {
                            let conf_0_8 = r_0_8.get("confianza").and_then(|c| c.as_f64()).unwrap_or(0.0);
                            if conf_0_8 > confianza {
                                r_0_8["confianza"] = serde_json::json!(conf_0_8);
                                r_0_8["reintentado_con"] = serde_json::json!("qwen3_5_0_8b");
                                return con_status_ok(r_0_8);
                            }
                        }
                    }
                }

                // Intento 3: confianza sigue < 0.8 → Qwen 3 1.7B.
                println!("[YARVIS-IA] Confianza aún baja ({confianza}), reintentando con Qwen 3 1.7B...");
                if let Some(m_1_7) = cargar_con_log("1.7B") {
                    if let Some(mut r_1_7) = ejecutar_analisis(&m_1_7, texto_ticket) {
                        if r_1_7.get("mapeo").is_some() {
                            let conf_1_7 = r_1_7.get("confianza").and_then(|c| c.as_f64()).unwrap_or(0.0);
                            if conf_1_7 > confianza {
                                r_1_7["confianza"] = serde_json::json!(conf_1_7);
                                r_1_7["reintentado_con"] = serde_json::json!("qwen3_1_7b");
                                return con_status_ok(r_1_7);
                            }
                        }
                    }
                }
            }

            resultado["reintentado_con"] = serde_json::Value::Null;
            return con_status_ok(resultado);
        }
    }

    // El 0.5B no devolvió JSON válido → Qwen 3.5 0.8B directo.
    println!("[YARVIS-IA] Qwen 0.5B no pudo analizar, usando Qwen 3.5 0.8B directamente...");
    if let Some(m_0_8) = cargar_con_log("0.8B") {
        if let Some(mut r_0_8) = ejecutar_analisis(&m_0_8, texto_ticket) {
            if r_0_8.get("mapeo").is_some() {
                let conf_0_8 = r_0_8.get("confianza").and_then(|c| c.as_f64()).unwrap_or(0.0);
                r_0_8["confianza"] = serde_json::json!(conf_0_8);
                r_0_8["reintentado_con"] = serde_json::json!("qwen3_5_0_8b");
                return con_status_ok(r_0_8);
            }
        }
    }

    // Ni 0.8B → Qwen 3 1.7B directo.
    println!("[YARVIS-IA] Qwen 0.8B no pudo analizar, usando Qwen 3 1.7B directamente...");
    if let Some(m_1_7) = cargar_con_log("1.7B") {
        if let Some(mut r_1_7) = ejecutar_analisis(&m_1_7, texto_ticket) {
            if r_1_7.get("mapeo").is_some() {
                let conf_1_7 = r_1_7.get("confianza").and_then(|c| c.as_f64()).unwrap_or(0.0);
                r_1_7["confianza"] = serde_json::json!(conf_1_7);
                r_1_7["reintentado_con"] = serde_json::json!("qwen3_1_7b");
                return con_status_ok(r_1_7);
            }
        }
    }

    serde_json::json!({ "status": "error", "error": "Ningún modelo pudo analizar el ticket" })
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
        println!("[YARVIS-IA] {count} modelo(s) descargado(s) de VRAM.");
    }
    count
}

// ===========================================================================
// Sin feature `llm-local`: API presente pero reporta que no hay backend.
// ===========================================================================

#[cfg(not(feature = "llm-local"))]
pub fn analizar_ticket(_texto_ticket: &str) -> serde_json::Value {
    serde_json::json!({
        "status": "error",
        "error": "El feature 'llm-local' de src-ia no está habilitado (sin soporte llama.cpp)."
    })
}

#[cfg(not(feature = "llm-local"))]
pub fn descargar_modelos() -> usize {
    0
}

// ---------------------------------------------------------------------------
// Tests (la lógica pura no depende de llama.cpp)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extraer_json_une_primer_abre_y_ultimo_cierra() {
        // Espejo de Python: regex greedy `\{[\s\S]*\}` (primer `{` al último `}`).
        // Si hay DOS objetos, el tramo entre ambos no es JSON válido → None.
        let respuesta = "claro!\n{\"hola\": 1}\n{\"extra\": true}\ny eso";
        assert!(extraer_json(respuesta).is_none());
    }

    #[test]
    fn extraer_json_con_texto_dentro_corchetes() {
        let respuesta = r#"{"notas": "algo {trozo} aquí"}"#;
        let v = extraer_json(respuesta).unwrap();
        assert_eq!(v["notas"], "algo {trozo} aquí");
    }

    #[test]
    fn extraer_json_sin_objeto_devuelve_none() {
        assert!(extraer_json("nada de json, solo texto").is_none());
        assert!(extraer_json("").is_none());
    }

    #[test]
    fn texto_vacio_devuelve_error() {
        let v = analizar_ticket("   ");
        assert_eq!(v["status"], "error");
    }
}