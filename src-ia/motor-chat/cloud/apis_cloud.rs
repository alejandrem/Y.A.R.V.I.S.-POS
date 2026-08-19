//! apis_cloud.rs — Respuestas por API de proveedores de IA (Gemini y OpenCode Zen).
//!
//! Port de `yarvis-IA/chatbot/motor_chat/modelos_API/apis_cloud.py`.
//!
//! Se encarga de:
//!     - Definir la configuración de cada proveedor (URL base, modelo por defecto, formato).
//!     - Convertir los mensajes de Y.A.R.V.I.S. al formato que espera cada proveedor.
//!     - Generar respuestas completas o por streaming vía HTTP (reqwest).
//!     - Listar los modelos gratuitos disponibles de cada proveedor (con caché).
//!
//! No toca hardware ni base de datos: recibe los mensajes ya construidos.
//! (A diferencia de Python, aquí se omiten las tools/function calling: el modelo
//! cloud ya no llama search_inventory.)

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::prompts::Mensaje;
use super::think::{limpiar_think, SeparadorThink, TipoFragmento};
use super::variables::{
    ESPERA_429_MAX_SECS, ESPERA_429_MIN_SECS, MAX_MODELOS_A_PROBAR, MAX_TOKENS,
    MODELOS_CACHE_TTL_SECS, MODELOS_FREE_EXTRA, ORDEN_FALLBACK_FREE, PROVIDERS, TIMEOUT_CONNECT_SECS,
    TIMEOUT_READ_SECS,
};

/// Uso de tokens reportado por el proveedor (se rellena durante el streaming).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

/// Evento que produce el stream de [`generar_stream`].
#[derive(Debug, Clone)]
pub enum Evento {
    /// Un trozo de texto crudo del modelo (puede contener marcadores think).
    /// Lleva el texto y el modelo real que lo generó (para el relevo 429).
    Texto { texto: String, modelo: String },
    /// Uso de tokens reportado por el proveedor.
    Uso { usage: Usage, modelo: String },
}

/// Modelo disponible en un proveedor de nube.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeloDisponible {
    pub id: String,
    pub name: String,
}

/// Error interno del motor cloud (distingue HTTP de red para el relevo 429).
#[derive(Debug)]
enum ErrorCloud {
    Http(u16, Option<String>),
    Red(String),
}

impl ErrorCloud {
    fn es_429(&self) -> bool {
        matches!(self, ErrorCloud::Http(429, _))
    }

    fn amigable(&self, display: &str) -> String {
        match self {
            ErrorCloud::Http(status, _) => error_amigable(*status, display),
            ErrorCloud::Red(e) => format!("No se pudo conectar con {display}: {e}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers de proveedor (espejo de apis_cloud.py)
// ---------------------------------------------------------------------------

/// Nombre amigable del proveedor (para mostrarlo en el modelo usado).
pub fn nombre_proveedor(provider: &str) -> String {
    PROVIDERS
        .iter()
        .find(|p| p.key == provider)
        .map(|p| p.name.to_string())
        .unwrap_or_else(|| provider.to_string())
}

/// Un modelo de OpenCode es gratuito si termina en '-free' o está en la lista extra.
fn es_free(model_id: &str) -> bool {
    model_id.ends_with("-free") || MODELOS_FREE_EXTRA.contains(&model_id)
}

/// Orden de los modelos a probar cuando el proveedor satura (429).
///
/// Espejo de `_cola_modelos_a_probar`: para OpenCode free arranca por el modelo
/// pedido y recorre `ORDEN_FALLBACK_FREE` limitado a `MAX_MODELOS_A_PROBAR`; para
/// cualquier otra combinación solo el modelo original.
fn cola_modelos_a_probar(provider: &str, model: &str) -> Vec<String> {
    if provider == "opencode" && es_free(model) {
        let mut cola: Vec<String>;
        if let Some(idx) = ORDEN_FALLBACK_FREE.iter().position(|m| *m == model) {
            let mut rotada: Vec<&str> = ORDEN_FALLBACK_FREE[idx..].to_vec();
            rotada.extend_from_slice(&ORDEN_FALLBACK_FREE[..idx]);
            cola = rotada.into_iter().map(|s| s.to_string()).collect();
        } else {
            cola = vec![model.to_string()];
            cola.extend(ORDEN_FALLBACK_FREE.iter().map(|s| s.to_string()));
        }
        let mut unicos: Vec<String> = Vec::with_capacity(cola.len());
        for m in cola {
            if !unicos.contains(&m) {
                unicos.push(m);
            }
        }
        unicos.truncate(MAX_MODELOS_A_PROBAR);
        unicos
    } else {
        vec![model.to_string()]
    }
}

/// Segundos a esperar ante un 429: respeta retry-after clavado al rango.
fn espera_429(retry_after: Option<&str>) -> u64 {
    let retry = retry_after.unwrap_or("");
    if let Ok(n) = retry.parse::<u64>() {
        return n.clamp(ESPERA_429_MIN_SECS, ESPERA_429_MAX_SECS);
    }
    (ESPERA_429_MIN_SECS + ESPERA_429_MAX_SECS) / 2
}

/// Traduce errores HTTP del proveedor a mensajes claros en español.
fn error_amigable(status: u16, display: &str) -> String {
    match status {
        429 => "Error 429: muchas preguntas al mismo tiempo. Espera 1 minuto y reintenta.".to_string(),
        401 | 403 => format!("API key inválida (error {status}). Revisa tu clave en 'Agregar API'."),
        402 => "Cuota agotada (error 402): revisa tu plan del proveedor.".to_string(),
        404 => format!("Modelo no disponible (error 404). Revisa el nombre del modelo del proveedor."),
        _ => format!("Error {status} del proveedor ({display})."),
    }
}

/// Junta mensajes consecutivos con el mismo rol (evita rechazos de APIs).
///
/// Espejo de `_normalizar_mensajes` (sin las ramas de tool_calls, ya que el
/// motor cloud ya no usa function calling).
fn normalizar_mensajes(messages: &[Mensaje]) -> Vec<Mensaje> {
    let mut normalized: Vec<Mensaje> = Vec::new();
    for m in messages {
        let role = if m.role.is_empty() { "user" } else { &m.role };
        if let Some(last) = normalized.last_mut() {
            if last.role == role {
                last.content.push('\n');
                last.content.push_str(&m.content);
                continue;
            }
        }
        normalized.push(Mensaje {
            role: role.to_string(),
            content: m.content.clone(),
        });
    }
    normalized
}

// ---------------------------------------------------------------------------
// Streaming SSE de cada proveedor
// ---------------------------------------------------------------------------

/// Lee un cuerpo SSE y emite cada línea `data: ...` (sin el prefijo).
fn sse_lineas(resp: reqwest::Response) -> impl futures_util::Stream<Item = Result<String, ErrorCloud>> {
    async_stream::stream! {
        let mut stream = resp.bytes_stream();
        let mut buf: Vec<u8> = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| ErrorCloud::Red(e.to_string()))?;
            buf.extend_from_slice(&chunk);
            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let mut line: Vec<u8> = buf.drain(..=pos).collect();
                line.pop();
                let line = String::from_utf8_lossy(&line).trim().to_string();
                if line.is_empty() {
                    continue;
                }
                yield Ok(line);
            }
        }
        if !buf.is_empty() {
            let line = String::from_utf8_lossy(&buf).trim().to_string();
            if !line.is_empty() {
                yield Ok(line);
            }
        }
    }
}

/// Genera el stream de un modelo específico (gemini u openai-compatible).
///
/// Espejo de `_iter_openai_compatible` + `_iter_google`: las respuestas llegan
/// como texto crudo que puede traer los marcadores ` think ... response `
/// (el caller los separa con [`SeparadorThink`]).
async fn stream_modelo<'a>(
    cfg: &'a super::variables::Provider,
    client: &'a Client,
    api_key: &'a str,
    modelo: &'a str,
    messages: &'a [Mensaje],
) -> std::pin::Pin<
    Box<dyn futures_util::Stream<Item = Result<Evento, ErrorCloud>> + Send + 'a>,
> {
    if cfg.key == "google" {
        Box::pin(stream_google(cfg, client, api_key, modelo, messages).await)
    } else {
        Box::pin(stream_openai_compatible(cfg, client, api_key, modelo, messages).await)
    }
}

/// OpenCode Zen (y cualquiera compatible con /chat/completions). Sin tools.
async fn stream_openai_compatible<'a>(
    cfg: &'a super::variables::Provider,
    client: &'a Client,
    api_key: &'a str,
    modelo: &'a str,
    messages: &'a [Mensaje],
) -> impl futures_util::Stream<Item = Result<Evento, ErrorCloud>> + 'a {
    async_stream::stream! {
        let url = format!("{}/chat/completions", cfg.base_url);
        let normalized = normalizar_mensajes(messages);

        // Reintento: primero con include_usage; si el proveedor no lo acepta
        // (400 antes de ceder tokens), reintenta sin él.
        let mut con_uso = true;
        loop {
            let mut body = serde_json::json!({
                "model": modelo,
                "messages": normalized.iter().map(|m| serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })).collect::<Vec<_>>(),
                "temperature": 0.6,
                "max_tokens": MAX_TOKENS,
                "stream": true,
            });
            if con_uso {
                body["stream_options"] = serde_json::json!({ "include_usage": true });
            }

            let resp = match client
                .post(&url)
                .header("Authorization", format!("Bearer {api_key}"))
                .json(&body)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    yield Err(ErrorCloud::Red(e.to_string()));
                    return;
                }
            };

            let status = resp.status();
            if status == reqwest::StatusCode::BAD_REQUEST && con_uso {
                con_uso = false;
                continue;
            }
            if !status.is_success() {
                let retry = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());
                yield Err(ErrorCloud::Http(status.as_u16(), retry));
                return;
            }

            let lineas = sse_lineas(resp);
            futures_util::pin_mut!(lineas);
            while let Some(line) = lineas.next().await {
                let line = line?;
                let Some(data) = line.strip_prefix("data:") else { continue };
                let data = data.trim();
                if data == "[DONE]" {
                    break;
                }
                let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) else {
                    continue;
                };

                // Chunk final de uso (stream_options.include_usage activo).
                if let Some(u) = chunk.get("usage") {
                    yield Ok(Evento::Uso {
                        usage: Usage {
                            prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()),
                            completion_tokens: u.get("completion_tokens").and_then(|v| v.as_u64()),
                            total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()),
                        },
                        modelo: modelo.to_string(),
                    });
                }

                let delta = chunk
                    .get("choices")
                    .and_then(|c| c.as_array())
                    .and_then(|c| c.first())
                    .and_then(|c| c.get("delta"))
                    .cloned()
                    .unwrap_or_default();

                let razonamiento = delta.get("reasoning_content").and_then(|v| v.as_str());
                if let Some(r) = razonamiento {
                    if !r.is_empty() {
                        yield Ok(Evento::Texto {
                            texto: format!(" think {r} response "),
                            modelo: modelo.to_string(),
                        });
                    }
                }
                let token = delta.get("content").and_then(|v| v.as_str());
                if let Some(t) = token {
                    if !t.is_empty() {
                        yield Ok(Evento::Texto {
                            texto: t.to_string(),
                            modelo: modelo.to_string(),
                        });
                    }
                }
            }
            return; // stream completado
        }
    }
}

/// Gemini — formato contents + system_instruction.
async fn stream_google<'a>(
    cfg: &'a super::variables::Provider,
    client: &'a Client,
    api_key: &'a str,
    modelo: &'a str,
    messages: &'a [Mensaje],
) -> impl futures_util::Stream<Item = Result<Evento, ErrorCloud>> + 'a {
    async_stream::stream! {
        let url = format!("{}/models/{modelo}:streamGenerateContent", cfg.base_url);
        let system = messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let mut contents: Vec<serde_json::Value> = Vec::new();
        for m in normalizar_mensajes(messages) {
            if m.role == "system" {
                continue;
            }
            let role = if m.role == "assistant" { "model" } else { "user" };
            contents.push(serde_json::json!({
                "role": role,
                "parts": [{ "text": m.content }],
            }));
        }

        let mut body = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": MAX_TOKENS,
                "temperature": 0.6,
            },
        });
        if !system.is_empty() {
            body["system_instruction"] = serde_json::json!({ "parts": [{ "text": system }] });
        }

        let resp = match client
            .post(&url)
            .query(&[("key", api_key), ("alt", "sse")])
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                yield Err(ErrorCloud::Red(e.to_string()));
                return;
            }
        };

        if !resp.status().is_success() {
            let retry = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            yield Err(ErrorCloud::Http(resp.status().as_u16(), retry));
            return;
        }

        let lineas = sse_lineas(resp);
        futures_util::pin_mut!(lineas);
        while let Some(line) = lineas.next().await {
            let line = line?;
            let Some(data) = line.strip_prefix("data:") else { continue };
            let data = data.trim();
            if data == "[DONE]" {
                break;
            }
            let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) else {
                continue;
            };

            if let Some(meta) = chunk.get("usageMetadata") {
                yield Ok(Evento::Uso {
                    usage: Usage {
                        prompt_tokens: meta.get("promptTokenCount").and_then(|v| v.as_u64()),
                        completion_tokens: meta
                            .get("candidatesTokenCount")
                            .and_then(|v| v.as_u64()),
                        total_tokens: meta.get("totalTokenCount").and_then(|v| v.as_u64()),
                    },
                    modelo: modelo.to_string(),
                });
            }

            let parts = chunk
                .get("candidates")
                .and_then(|c| c.as_array())
                .and_then(|c| c.first())
                .and_then(|c| c.get("content"))
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.as_array())
                .cloned()
                .unwrap_or_default();

            for part in parts {
                let Some(texto) = part.get("text").and_then(|t| t.as_str()) else {
                    continue;
                };
                if texto.is_empty() {
                    continue;
                }
                let pensamiento = part.get("thought").and_then(|t| t.as_bool()).unwrap_or(false);
                if pensamiento {
                    yield Ok(Evento::Texto {
                        texto: format!(" think {texto} response "),
                        modelo: modelo.to_string(),
                    });
                } else {
                    yield Ok(Evento::Texto {
                        texto: texto.to_string(),
                        modelo: modelo.to_string(),
                    });
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// API pública
// ---------------------------------------------------------------------------

fn cliente() -> Client {
    Client::builder()
        .connect_timeout(Duration::from_secs(TIMEOUT_CONNECT_SECS))
        .timeout(Duration::from_secs(TIMEOUT_READ_SECS))
        .build()
        .expect("Error creando HTTP client compartido")
}

/// Genera el streaming del proveedor indicado, con relevo 429 entre modelos.
///
/// Espejo de `generar_stream` de Python:
///     - OpenCode free → prueba hasta `MAX_MODELOS_A_PROBAR` modelos gratuitos.
///     - Cualquier otro modelo → espera 2-4 s y reintenta una vez.
///     - Si el modelo YA cedió tokens y luego falla, el error se propaga tal cual.
///
/// Devuelve [`Evento::Texto`] (texto + modelo real) y [`Evento::Uso`].
pub fn generar_stream<'a>(
    provider: &'a str,
    api_key: &'a str,
    model: &'a str,
    messages: Vec<Mensaje>,
) -> impl futures_util::Stream<Item = Result<Evento, String>> + 'a {
    async_stream::stream! {
        let cfg = match PROVIDERS.iter().find(|p| p.key == provider) {
            Some(c) => c,
            None => {
                yield Err(format!("Proveedor no soportado: {provider}"));
                return;
            }
        };
        if api_key.is_empty() {
            yield Err("Falta la API key del proveedor.".to_string());
            return;
        }

        let model = if model.is_empty() { cfg.default_model } else { model };
        let display = cfg.name;
        let client = cliente();
        let cola = cola_modelos_a_probar(provider, model);

        for idx in 0..cola.len() {
            let modelo = cola[idx].clone();
            let mut cedio = false;

            let inner = stream_modelo(cfg, &client, api_key, &modelo, &messages).await;
            futures_util::pin_mut!(inner);
            let mut error: Option<ErrorCloud> = None;

            while let Some(item) = inner.next().await {
                match item {
                    Ok(ev) => match ev {
                        Evento::Texto { texto, modelo: m } => {
                            cedio = true;
                            yield Ok(Evento::Texto { texto, modelo: m });
                        }
                        Evento::Uso { usage, modelo: m } => {
                            yield Ok(Evento::Uso { usage, modelo: m });
                        }
                    },
                    Err(e) => {
                        error = Some(e);
                        break;
                    }
                }
            }
            let Some(err) = error else {
                // Stream terminó sin error: éxito.
                return;
            };

            // Si ya cedimos tokens, el modelo SÍ conectó: el error es real.
            if !err.es_429() || cedio {
                yield Err(err.amigable(display));
                return;
            }

            let espera = espera_429(match &err { ErrorCloud::Http(_, Some(r)) => Some(r.as_str()), _ => None });
            let siguiente = cola.get(idx + 1);

            if let Some(sig) = siguiente {
                println!(
                    "[YARVIS] {modelo} saturado (429), cambiando a {sig} (espera {espera}s)"
                );
                tokio::time::sleep(Duration::from_secs(espera)).await;
                continue;
            }

            // Último modelo: un respiro y un último intento real.
            println!("[YARVIS] {modelo} saturado (429), último intento tras {espera}s");
            tokio::time::sleep(Duration::from_secs(espera)).await;
            let inner2 = stream_modelo(cfg, &client, api_key, &modelo, &messages).await;
            futures_util::pin_mut!(inner2);
            let mut error_final: Option<ErrorCloud> = None;
            while let Some(item) = inner2.next().await {
                match item {
                    Ok(ev) => match ev {
                        Evento::Texto { texto, modelo: m } => {
                            yield Ok(Evento::Texto { texto, modelo: m });
                        }
                        Evento::Uso { usage, modelo: m } => {
                            yield Ok(Evento::Uso { usage, modelo: m });
                        }
                    },
                    Err(e) => {
                        error_final = Some(e);
                        break;
                    }
                }
            }
            if error_final.is_none() {
                return;
            }
            yield Err(error_final.unwrap().amigable(display));
            return;
        }

        yield Err(format!("No se pudo completar la respuesta con {display}"));
    }
}

/// Respuesta completa (sin streaming) limpiando los bloques thinking.
///
/// Espejo de `generar_completo` de Python: reconstruye solo la parte 'token'.
/// Devuelve `(texto, modelo_real)`: el modelo que realmente respondió (puede
/// diferir del pedido por el relevo 429) o vacío si no se reportó.
pub async fn generar_completo(
    provider: &str,
    api_key: &str,
    model: &str,
    messages: Vec<Mensaje>,
) -> Result<(String, String), String> {
    let stream = generar_stream(provider, api_key, model, messages);
    futures_util::pin_mut!(stream);

    let mut sep = SeparadorThink::new(usize::MAX);
    let mut salida = String::new();
    let mut modelo_final = String::new();

    while let Some(item) = stream.next().await {
        match item {
            Ok(Evento::Texto { texto, modelo }) => {
                if modelo_final.is_empty() {
                    modelo_final = modelo;
                }
                for (tipo, frag) in sep.procesar(&texto) {
                    if tipo == TipoFragmento::Token {
                        salida.push_str(&frag);
                    }
                }
            }
            Ok(Evento::Uso { .. }) => {}
            Err(e) => return Err(e),
        }
    }
    for (tipo, frag) in sep.finalizar() {
        if tipo == TipoFragmento::Token {
            salida.push_str(&frag);
        }
    }
    Ok((limpiar_think(&salida), modelo_final))
}

// ---------------------------------------------------------------------------
// Listado de modelos de los proveedores (caché TTL 60s)
// ---------------------------------------------------------------------------

static _MODELOS_CACHE: OnceLock<Mutex<Vec<(String, Instant, Vec<ModeloDisponible>)>>> =
    OnceLock::new();

fn cache_modelos() -> &'static Mutex<Vec<(String, Instant, Vec<ModeloDisponible>)>> {
    _MODELOS_CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

/// Lista los modelos disponibles de un proveedor (solo gratuitos en OpenCode).
/// Devuelve `[{'id', 'name'}]` con caché de 60 segundos.
pub async fn listar_modelos(provider: &str, api_key: &str) -> Result<Vec<ModeloDisponible>, String> {
    let cfg = PROVIDERS
        .iter()
        .find(|p| p.key == provider)
        .ok_or_else(|| format!("Proveedor no soportado: {provider}"))?;

    {
        let cache = cache_modelos().lock().unwrap_or_else(|p| p.into_inner());
        if let Some((_, time, lista)) = cache.iter().find(|(p, _, _)| p == provider) {
            if time.elapsed().as_secs_f64() < MODELOS_CACHE_TTL_SECS {
                return Ok(lista.clone());
            }
        }
    }

    let client = cliente();
    let modelos = if provider == "google" {
        if api_key.is_empty() {
            return Err("Falta la API key de Google (Gemini).".to_string());
        }
        let resp = client
            .get(format!("{}/models", cfg.base_url))
            .query(&[("key", api_key)])
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("Error {} listando modelos", resp.status()));
        }
        let payload: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        payload
            .get("models")
            .and_then(|m| m.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|m| {
                m.get("supportedGenerationMethods")
                    .and_then(|s| s.as_array())
                    .map(|m| m.iter().any(|x| x.as_str() == Some("generateContent")))
                    .unwrap_or(false)
            })
            .map(|m| {
                let raw = m.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                let id = raw.strip_prefix("models/").unwrap_or(&raw).to_string();
                let name = m
                    .get("displayName")
                    .and_then(|n| n.as_str())
                    .unwrap_or(&id)
                    .to_string();
                ModeloDisponible { id, name }
            })
            .collect()
    } else {
        let url = format!("{}/models", cfg.base_url);
        let mut req = client.get(&url);
        if !api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {api_key}"));
        }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("Error {} listando modelos", resp.status()));
        }
        let payload: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let lista = payload
            .get("data")
            .or_else(|| payload.as_array().map(|a| a.get(1).unwrap_or(&payload)))
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_else(|| {
                if payload.is_array() {
                    payload.as_array().cloned().unwrap_or_default()
                } else {
                    Vec::new()
                }
            });
        lista
            .into_iter()
            .filter_map(|m| {
                let id = m.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
                if !es_free(&id) {
                    return None;
                }
                let name = m.get("name").and_then(|n| n.as_str()).unwrap_or(&id).to_string();
                Some(ModeloDisponible { id, name })
            })
            .collect()
    };

    let mut lista: Vec<_> = modelos;
    lista.sort_by(|a, b| a.id.cmp(&b.id));

    let mut cache = cache_modelos().lock().unwrap_or_else(|p| p.into_inner());
    if let Some(entry) = cache.iter_mut().find(|(p, _, _)| p == provider) {
        entry.1 = Instant::now();
        entry.2 = lista.clone();
    } else {
        cache.push((provider.to_string(), Instant::now(), lista.clone()));
    }

    Ok(lista)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nombre_proveedor_devuelve_nombre_amigable() {
        assert_eq!(nombre_proveedor("google"), "Gemini");
        assert_eq!(nombre_proveedor("opencode"), "OpenCode");
        assert_eq!(nombre_proveedor("nope"), "nope");
    }

    #[test]
    fn es_free_detecta_sufijo_y_extra() {
        assert!(es_free("mimo-v2.5-free"));
        assert!(es_free("big-pickle"));
        assert!(!es_free("gemini-2.0-flash"));
    }

    #[test]
    fn cola_fallback_opencode_free_rota_desde_el_pedido() {
        let cola = cola_modelos_a_probar("opencode", "nemotron-3-ultra-free");
        assert_eq!(cola.len(), MAX_MODELOS_A_PROBAR);
        assert_eq!(cola[0], "nemotron-3-ultra-free");
        assert_eq!(cola[1], "nemotron-3.5-lightning-free");
        assert_eq!(cola[2], "hy3-free");
    }

    #[test]
    fn cola_fallback_opencode_pedido_desconocido() {
        let cola = cola_modelos_a_probar("opencode", "raro-free");
        assert_eq!(cola.len(), MAX_MODELOS_A_PROBAR);
        assert_eq!(cola[0], "raro-free");
    }

    #[test]
    fn cola_fallback_modelo_no_free_o_otro_proveedor() {
        assert_eq!(cola_modelos_a_probar("opencode", "gemini-2.0-flash"), vec!["gemini-2.0-flash"]);
        assert_eq!(cola_modelos_a_probar("google", "mimo-v2.5-free"), vec!["mimo-v2.5-free"]);
    }

    #[test]
    fn normalizar_mensajes_junta_roles_consecutivos() {
        let msgs = vec![
            Mensaje::new("user", "hola"),
            Mensaje::new("user", "mundo"),
            Mensaje::new("assistant", "ok"),
            Mensaje::new("user", "otra"),
        ];
        let norm = normalizar_mensajes(&msgs);
        assert_eq!(norm.len(), 3);
        assert_eq!(norm[0].content, "hola\nmundo");
    }

    #[test]
    fn espera_429_respeta_retry_after_clavado() {
        assert_eq!(espera_429(Some("1")), 2);
        assert_eq!(espera_429(Some("99")), 4);
        assert_eq!(espera_429(Some("3")), 3);
        assert_eq!(espera_429(None), 3);
    }
}