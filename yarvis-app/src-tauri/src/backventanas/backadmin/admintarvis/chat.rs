// ============================================================
// admintarvis/chat.rs — Comandos IPC para el chatbot de Y.A.R.V.I.S.
// Soporte streaming: cloud emitido por Rust; local = Qwen 1.7B nativo.
// ============================================================

use tauri::Emitter;

use crate::backventanas::auth::AuthState;
use futures_util::StreamExt;
use src_ia::motor_chat::cloud::apis_cloud::{
    generar_completo, generar_stream, nombre_proveedor, Evento,
};
use src_ia::motor_chat::cloud::prompts::{construir_mensajes_api_rol, Mensaje};
use src_ia::motor_chat::llm::tools;

/// Ruta del archivo SQLite desde el pool (para el ejecutor de tools).
fn db_path_de(pool: &sqlx::SqlitePool) -> String {
    pool.connect_options().get_filename().to_string_lossy().to_string()
}

type GeneradorFut = std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>;
type Generador<'a> = Box<dyn FnMut(Vec<Mensaje>) -> GeneradorFut + Send + 'a>;

/// Bandera de cancelación del stream en curso. `stop_chat_stream` la levanta;
/// los bucles de emisión (cloud y local) la consultan entre tokens/rondas y
/// cortan la respuesta. Es cooperativa: la generación local a bloqueo termina
/// su ciclo interno, pero NADA más se emite a la UI después de cancelar.
static STREAM_CANCELADO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn stream_cancelado() -> bool {
    STREAM_CANCELADO.load(std::sync::atomic::Ordering::Relaxed)
}

/// Tools que exponen métricas financieras GLOBALES (revenue total,
/// comparativas de periodo, análisis de recompra con costos). El prompt
/// le pide al modelo no usarlas con empleados, pero un prompt es
/// sugerencia — ESTO es control de acceso real en el punto de ejecución.
const TOOLS_SOLO_ADMIN: &[&str] = &["query_sales", "compare_periods", "get_restock_analysis"];

/// Ejecuta una tool respetando el rol de la sesión. Si está bloqueada, NO
/// se ejecuta: se le devuelve al modelo un error de permisos para que
/// responda con elegancia ("eso te lo puede decir el administrador").
async fn ejecutar_tool_con_rol(
    nombre: &str,
    args: &str,
    db_path: &str,
    es_empleado: bool,
) -> String {
    if es_empleado && TOOLS_SOLO_ADMIN.contains(&nombre) {
        tracing::warn!("[YARVIS-TOOLS] BLOQUEADA por rol ({es_empleado}): {nombre}");
        return serde_json::json!({
            "error": "Permiso denegado: esta consulta financiera solo está disponible para el administrador."
        })
        .to_string();
    }
    let n = nombre.to_string();
    let a = args.to_string();
    let db = db_path.to_string();
    match tokio::task::spawn_blocking(move || tools::ejecutar_tool(&n, &a, &db)).await {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => serde_json::json!({ "error": e }).to_string(),
        Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
    }
}

/// Ciclo tool_call→ejecutar→re-preguntar. Mientras el modelo siga pidiendo
/// herramientas (hasta MAX rondas), ejecuta el SQL real y le devuelve el
/// resultado como mensaje role:"tool" hasta obtener una respuesta final.
async fn resolver_ciclo_tools(
    mut respuesta: String,
    mut historial: Vec<Mensaje>,
    db_path: String,
    es_empleado: bool,
    generar: &mut Generador<'_>,
) -> Result<String, String> {
    for _ in 0..tools::MAX_RONDAS_TOOLS {
        let Some((nombre, args)) = tools::detectar_tool_call(&respuesta) else {
            return Ok(respuesta);
        };
        tracing::info!("[YARVIS-TOOLS] ejecutando {nombre}({args})");
        let json_res = ejecutar_tool_con_rol(&nombre, &args, &db_path, es_empleado).await;
        historial.push(Mensaje::new("assistant", respuesta));
        historial.push(Mensaje::new("tool", json_res));
        respuesta = (&mut *generar)(historial.clone()).await?;
    }
    // Agotó rondas: entregar limpio (sin bloques crudos)
    Ok(tools::respuesta_final_segura(&respuesta))
}

/// Suprime bloques <tool_call>...</tool_call> de un stream token a token,
/// reteniendo colas parciales que podrían ser el inicio del marcador.
struct SupresorToolCall {
    retenido: String,
    en_bloque: bool,
}

impl SupresorToolCall {
    fn new() -> Self {
        Self { retenido: String::new(), en_bloque: false }
    }

    fn procesar(&mut self, frag: &str) -> String {
        self.retenido.push_str(frag);
        let mut out = String::new();
        loop {
            if self.en_bloque {
                if let Some(i) = self.retenido.find("</tool_call>") {
                    self.retenido.drain(..i + "</tool_call>".len());
                    self.en_bloque = false;
                    continue;
                }
                self.retenido.clear();
                break;
            }
            match self.retenido.find("<tool_call>") {
                Some(i) => {
                    out.push_str(&self.retenido[..i]);
                    self.retenido.drain(..i + "<tool_call>".len());
                    self.en_bloque = true;
                }
                None => {
                    let max_hold = "<tool_call>".len() - 1;
                    if self.retenido.len() > max_hold {
                        let mut corte = self.retenido.len() - max_hold;
                        while corte > 0 && !self.retenido.is_char_boundary(corte) {
                            corte -= 1;
                        }
                        out.push_str(&self.retenido[..corte]);
                        self.retenido.drain(..corte);
                    }
                    break;
                }
            }
        }
        out
    }

    fn finalizar(&mut self) -> String {
        if self.en_bloque {
            self.retenido.clear();
            String::new()
        } else {
            std::mem::take(&mut self.retenido)
        }
    }
}
use src_ia::motor_chat::cloud::think::{SeparadorThink, TipoFragmento};
use src_ia::motor_chat::llm::{
    cargar_modelo_1_7, chat_1_7, descargar_modelo_1_7, modelo_1_7_cargado, nombre_modelo_local,
    ram_libre_gb, ram_total_gb, CONTEXTO_LOCAL, MODELO_CHAT, RAM_GB_MINIMA_1_7,
};

/// Máximo de palabras que se consideran razonamiento en el stream cloud
/// (espejo del `max_w` que usaba el motor original para `_separar_think`).
const CLOUD_MAX_W: usize = 1000;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub model_used: String,
}

/// Estado de los modelos locales y la RAM del sistema (nativo).
#[tauri::command]
pub async fn get_model_status(
    auth: tauri::State<'_, AuthState>,
) -> Result<serde_json::Value, String> {
    auth.require_operator()?;
    let ram_libre = ram_libre_gb().unwrap_or(0.0);
    let ram_total = ram_total_gb().unwrap_or(0.0);
    Ok(serde_json::json!({
        "status": "ok",
        // Único modelo local: el 1.7B (parseo de tickets y conversación).
        "models": { MODELO_CHAT: modelo_1_7_cargado() },
        "ram_gb": ram_total,
        "ram_libre_gb": ram_libre,
        "local_model_path": src_ia::motor_chat::llm::ruta_modelo_local().to_string_lossy(),
        "local_model_name": nombre_modelo_local(),
        "local_context_window": CONTEXTO_LOCAL,
    }))
}

/// Guarda la ruta GGUF elegida por el usuario y libera el modelo actual para
/// que la siguiente carga use el archivo nuevo.
#[tauri::command]
pub async fn set_local_model_path(
    auth: tauri::State<'_, AuthState>,
    path: String,
) -> Result<serde_json::Value, String> {
    auth.require_operator()?;
    let path = std::path::PathBuf::from(path.trim());
    if !path.is_file() {
        return Err(format!(
            "El archivo del modelo no existe: {}",
            path.display()
        ));
    }

    tokio::task::spawn_blocking(descargar_modelo_1_7)
        .await
        .map_err(|e| format!("El hilo de descarga falló: {e}"))?;
    src_ia::rutas::configurar_ruta_modelo(Some(path))?;

    Ok(serde_json::json!({
        "status": "ok",
        "path": src_ia::motor_chat::llm::ruta_modelo_local().to_string_lossy(),
        "name": nombre_modelo_local(),
        "context_window": CONTEXTO_LOCAL,
        "models": { MODELO_CHAT: false },
    }))
}

/// Carga el modelo local de conversación (1.7B) SOLO si hay ≥1GB de RAM libre.
#[tauri::command]
pub async fn load_chat_model(
    auth: tauri::State<'_, AuthState>,
    model: String,
) -> Result<serde_json::Value, String> {
    auth.require_operator()?;
    if model != MODELO_CHAT {
        return Err(format!(
            "El único modelo local es {MODELO_CHAT} (parseo de tickets y conversación)."
        ));
    }

    let libre = ram_libre_gb()?;
    if libre < RAM_GB_MINIMA_1_7 {
        return Err(format!(
            "RAM insuficiente para {MODELO_CHAT}: hay {libre:.2}GB libres, se necesitan ≥{RAM_GB_MINIMA_1_7}GB."
        ));
    }

    let modelo = tokio::task::spawn_blocking(cargar_modelo_1_7)
        .await
        .map_err(|e| format!("El hilo de carga falló: {e}"))??;

    Ok(serde_json::json!({
        "status": "ok",
        "model": modelo,
        "models": { MODELO_CHAT: true },
        "ram_gb": ram_total_gb().unwrap_or(0.0),
        "ram_libre_gb": ram_libre_gb().unwrap_or(0.0),
    }))
}

/// Lista los modelos disponibles de un proveedor de nube (dinámico).
///
/// Devuelve `{"models": [{"id": ..., "name": ...}]}` con el mismo JSON que
/// el endpoint `/cloud_models` al que reemplaza.
#[tauri::command]
pub async fn get_cloud_models(
    auth: tauri::State<'_, AuthState>,
    provider: String,
    api_key: Option<String>,
) -> Result<serde_json::Value, String> {
    auth.require_operator()?;
    let modelos = src_ia::motor_chat::cloud::apis_cloud::listar_modelos(
        &provider,
        api_key.as_deref().unwrap_or(""),
    )
    .await?;
    Ok(serde_json::json!({ "models": modelos }))
}

/// Detiene la generación en curso. La inferencia local del 1.7B es a bloqueo
/// (llama.cpp), así que no hay nada que interrumpir: se responde ok para no
/// romper el contrato con la UI (idéntico al endpoint `/stop` del motor original).
/// Detiene la emisión del stream en curso: levanta la bandera que los bucles
/// de generación (cloud y local) consultan entre tokens y rondas de tools.
/// La inferencia local a bloqueo puede tardar en llegar a un punto de chequeo,
/// pero ningún token posterior se emite a la UI tras esta llamada.
#[tauri::command]
pub async fn stop_chat_stream(auth: tauri::State<'_, AuthState>) -> Result<String, String> {
    auth.require_operator()?;
    STREAM_CANCELADO.store(true, std::sync::atomic::Ordering::Relaxed);
    tracing::info!("[YARVIS-CHAT] stop solicitado por el usuario");
    Ok("ok".to_string())
}

/// Descarga el modelo local de conversación (1.7B) para liberar RAM.
#[tauri::command]
pub async fn unload_chat_model(
    auth: tauri::State<'_, AuthState>,
    model: String,
) -> Result<serde_json::Value, String> {
    auth.require_operator()?;
    let descargado = tokio::task::spawn_blocking(descargar_modelo_1_7)
        .await
        .map_err(|e| format!("El hilo de descarga falló: {e}"))?;
    tracing::info!("[YARVIS-CHAT] Descarga pedida para {model}: descargado = {descargado}");

    Ok(serde_json::json!({
        "status": "ok",
        "models": { MODELO_CHAT: false },
        "ram_gb": ram_total_gb().unwrap_or(0.0),
        "ram_libre_gb": ram_libre_gb().unwrap_or(0.0),
    }))
}

/// Chat sin streaming (respuesta completa).
#[tauri::command]
pub async fn send_chat_message(
    auth: tauri::State<'_, AuthState>,
    state: tauri::State<'_, sqlx::SqlitePool>,
    messages: Vec<serde_json::Value>,
    role: String,
    model: String,
    provider: Option<String>,
    api_key: Option<String>,
) -> Result<ChatResponse, String> {
    auth.require_operator()?;
    let provider = provider.unwrap_or_default();
    let db_path = db_path_de(&state);
    // Modo cloud: lo responde Rust directamente (port de generar_completo).
    // Si falla, cae al modelo local.
    if !provider.is_empty() {
        let api_key = api_key.unwrap_or_default();
        let es_empleado = auth.es_empleado();
        let chat = construir_mensajes_api_rol(&mensajes_serde_a_rust(&messages), es_empleado);
        match generar_completo(&provider, &api_key, &model, chat.clone()).await {
            Ok((respuesta, modelo_real)) => {
                let usado = if modelo_real.is_empty() {
                    nombre_proveedor(&provider)
                } else {
                    modelo_real
                };
                let mut generador: Generador = Box::new(move |hist| {
                    let p = provider.clone();
                    let k = api_key.clone();
                    let m = model.clone();
                    Box::pin(async move { generar_completo(&p, &k, &m, hist).await.map(|(r, _)| r) })
                });
                let final_resp =
                    resolver_ciclo_tools(respuesta, chat, db_path, es_empleado, &mut generador).await?;
                return Ok(ChatResponse {
                    response: final_resp,
                    model_used: usado,
                });
            }
            Err(e) => {
                tracing::warn!("[YARVIS-CHAT] Error proveedor ({provider}): {e}");
                return _chat_local(messages, db_path, auth.es_empleado()).await;
            }
        }
    }

    // Modo local: Qwen 1.7B nativo de Rust.
    tracing::info!(
        "[YARVIS-CHAT] Modo local (model pedido: {model}, role: {role}) → usando {MODELO_CHAT}."
    );
    _chat_local(messages, db_path, auth.es_empleado()).await
}

/// Chat con streaming — modo cloud lo emite Rust (port de generar_stream),
/// el modo local trocea la respuesta del Qwen 1.7B nativo.
#[tauri::command]
pub async fn send_chat_stream(
    app: tauri::AppHandle,
    auth: tauri::State<'_, AuthState>,
    state: tauri::State<'_, sqlx::SqlitePool>,
    messages: Vec<serde_json::Value>,
    role: String,
    model: String,
    provider: Option<String>,
    api_key: Option<String>,
) -> Result<String, String> {
    auth.require_operator()?;
    STREAM_CANCELADO.store(false, std::sync::atomic::Ordering::Relaxed); // nueva generación: cancelación limpia
    let provider = provider.unwrap_or_default();
    let db_path = db_path_de(&state);

    // ---- Modo cloud: streaming en Rust. ----
    if !provider.is_empty() {
        let api_key = api_key.unwrap_or_default();
        let es_empleado = auth.es_empleado();
        let chat = construir_mensajes_api_rol(&mensajes_serde_a_rust(&messages), es_empleado);
        match _stream_cloud(&app, &provider, &api_key, &model, chat, &db_path, es_empleado, 0).await {
            Ok(respuesta) => return Ok(respuesta),
            Err(e) => {
                tracing::warn!("[YARVIS-CHAT] Error proveedor ({provider}), fallback local: {e}");
                return _stream_local(&app, messages, db_path, es_empleado).await;
            }
        }
    }

    // ---- Modo local: Qwen 1.7B nativo de Rust. ----
    tracing::info!(
        "[YARVIS-CHAT] Modo local (model pedido: {model}, role: {role}) → usando {MODELO_CHAT}."
    );
    _stream_local(&app, messages, db_path, auth.es_empleado()).await
}

/// Emite un stream cloud completo con Rust y devuelve la respuesta acumulada.
async fn _stream_cloud(
    app: &tauri::AppHandle,
    provider: &str,
    api_key: &str,
    model: &str,
    chat: Vec<Mensaje>,
    db_path: &str,
    es_empleado: bool,
    ronda: usize,
) -> Result<String, String> {
    let stream = generar_stream(provider, api_key, model, chat.clone());
    futures_util::pin_mut!(stream);

    let mut sep = SeparadorThink::new(CLOUD_MAX_W);
    // Supresor: los bloques <tool_call> NUNCA llegan a la UI.
    let mut supresor = SupresorToolCall::new();
    let mut full_response = String::new();
    let mut model_used = String::from("unknown");

    while let Some(item) = stream.next().await {
        if stream_cancelado() {
            tracing::info!("[YARVIS-CHAT] stream cancelado por el usuario");
            return Ok(full_response); // nada más se emite tras el stop
        }
        match item {
            Ok(Evento::Texto { texto, modelo }) => {
                if model_used == "unknown" {
                    model_used = modelo.clone();
                }
                for (tipo, frag) in sep.procesar(&texto) {
                    if tipo == TipoFragmento::Token {
                        let visible = supresor.procesar(&frag);
                        full_response.push_str(&frag);
                        if !visible.is_empty() {
                            let _ = app.emit(
                                "chat-token",
                                serde_json::json!({
                                    "token": visible,
                                    "model": modelo,
                                }),
                            );
                        }
                    } else {
                        let _ = app.emit(
                            "chat-think",
                            serde_json::json!({
                                "token": frag,
                                "model": modelo,
                            }),
                        );
                    }
                }
            }
            Ok(Evento::Uso { usage, modelo }) => {
                if model_used == "unknown" {
                    model_used = modelo;
                }
                let _ = app.emit(
                    "chat-usage",
                    serde_json::json!({
                        "usage": serde_json::json!({
                            "prompt_tokens": usage.prompt_tokens,
                            "completion_tokens": usage.completion_tokens,
                            "total_tokens": usage.total_tokens,
                        }),
                    }),
                );
            }
            Err(e) => return Err(e),
        }
    }
    for (tipo, frag) in sep.finalizar() {
        if tipo == TipoFragmento::Token {
            let visible = supresor.procesar(&frag);
            full_response.push_str(&frag);
            if !visible.is_empty() {
                let _ = app.emit(
                    "chat-token",
                    serde_json::json!({
                        "token": visible,
                        "model": model_used,
                    }),
                );
            }
        }
    }
    let _ = supresor.finalizar();

    // ¿El stream pidió herramientas? Ejecutarlas y re-generar en silencio;
    // la respuesta final (sin tool_calls) se emite troceada al usuario.
    if !stream_cancelado()
        && tools::detectar_tool_call(&full_response).is_some()
        && ronda < tools::MAX_RONDAS_TOOLS
    {
        tracing::info!("[YARVIS-TOOLS] cloud pidió tool en ronda {ronda}");
        let mut generador: Generador = Box::new({
            let provider = provider.to_string();
            let api_key = api_key.to_string();
            let model = model.to_string();
            move |hist| {
                let p = provider.clone();
                let k = api_key.clone();
                let m = model.clone();
                Box::pin(async move { generar_completo(&p, &k, &m, hist).await.map(|(r, _)| r) })
            }
        });
        let final_resp =
            resolver_ciclo_tools(full_response, chat, db_path.to_string(), es_empleado, &mut generador).await?;
        return _emitir_como_stream(app, &final_resp, &model_used);
    }

    let _ = app.emit("chat-done", serde_json::json!({ "model": model_used }));
    let _ = app.emit(
        "chat-complete",
        serde_json::json!({
            "response": full_response,
            "model": model_used,
        }),
    );
    Ok(full_response)
}

/// Emite un texto final troceado (~40 chars) como si fuera streaming local.
fn _emitir_como_stream(
    app: &tauri::AppHandle,
    texto: &str,
    modelo: &str,
) -> Result<String, String> {
    let mut seg = String::new();
    for c in texto.chars() {
        if stream_cancelado() {
            tracing::info!("[YARVIS-CHAT] emisión cancelada por el usuario");
            return Ok(String::new());
        }
        seg.push(c);
        if seg.chars().count() >= 40 {
            let _ = app.emit("chat-token", serde_json::json!({ "token": seg, "model": modelo }));
            seg = String::new();
        }
    }
    if !seg.is_empty() {
        let _ = app.emit("chat-token", serde_json::json!({ "token": seg, "model": modelo }));
    }
    let _ = app.emit("chat-done", serde_json::json!({ "model": modelo }));
    let _ = app.emit(
        "chat-complete",
        serde_json::json!({ "response": texto, "model": modelo }),
    );
    Ok(texto.to_string())
}

/// Chat local (sin provider): responde el Qwen 1.7B nativo de Rust.
async fn _chat_local(
    messages: Vec<serde_json::Value>,
    db_path: String,
    es_empleado: bool,
) -> Result<ChatResponse, String> {
    let mut historial = mensajes_serde_a_rust(&messages);
    let mut respuesta = tokio::task::spawn_blocking({
        let h = historial.clone();
        move || chat_1_7(&h)
    })
    .await
    .map_err(|e| format!("El hilo del modelo 1.7B falló: {e}"))??;

    for _ in 0..tools::MAX_RONDAS_TOOLS {
        let Some((nombre, args)) = tools::detectar_tool_call(&respuesta) else { break };
        tracing::info!("[YARVIS-TOOLS] ejecutando {nombre}({args})");
        let json_res = ejecutar_tool_con_rol(&nombre, &args, &db_path, es_empleado).await;
        historial.push(Mensaje::new("assistant", respuesta));
        historial.push(Mensaje::new("tool", json_res));
        respuesta = tokio::task::spawn_blocking({
            let h = historial.clone();
            move || chat_1_7(&h)
        })
        .await
        .map_err(|e| format!("El hilo del modelo 1.7B falló: {e}"))??;
    }
    let response = tools::respuesta_final_segura(&respuesta);

    Ok(ChatResponse {
        response,
        model_used: nombre_modelo_local(),
    })
}

/// Chat local con streaming: la inferencia del 1.7B es a bloqueo (llama.cpp),
/// así que se genera la respuesta completa (ya limpia de bloques  thinking ,
/// con el limpiador local robusto) y se emite en trozos para que la UI la
/// muestre de forma progresiva (eventos `chat-token`).
async fn _stream_local(
    app: &tauri::AppHandle,
    messages: Vec<serde_json::Value>,
    db_path: String,
    es_empleado: bool,
) -> Result<String, String> {
    let mut historial = mensajes_serde_a_rust(&messages);

    let mut cleaned = tokio::task::spawn_blocking({
        let h = historial.clone();
        move || chat_1_7(&h)
    })
    .await
    .map_err(|e| format!("El hilo del modelo 1.7B falló: {e}"))??;

    // Ciclo de herramientas ANTES de emitir nada a la UI.
    for _ in 0..tools::MAX_RONDAS_TOOLS {
        let Some((nombre, args)) = tools::detectar_tool_call(&cleaned) else { break };
        tracing::info!("[YARVIS-TOOLS] ejecutando {nombre}({args})");
        let json_res = ejecutar_tool_con_rol(&nombre, &args, &db_path, es_empleado).await;
        historial.push(Mensaje::new("assistant", cleaned));
        historial.push(Mensaje::new("tool", json_res));
        cleaned = tokio::task::spawn_blocking({
            let h = historial.clone();
            move || chat_1_7(&h)
        })
        .await
        .map_err(|e| format!("El hilo del modelo 1.7B falló: {e}"))??;
    }
    let cleaned = tools::respuesta_final_segura(&cleaned);

    // Emite la respuesta en trozos (~40 chars) preservando saltos de línea.
    let mut seg = String::new();
    let mut n = 0;
    for c in cleaned.chars() {
        if stream_cancelado() {
            tracing::info!("[YARVIS-CHAT] emisión local cancelada por el usuario");
            return Ok(String::new());
        }
        seg.push(c);
        n += 1;
        if n >= 40 {
            let _ = app.emit(
                "chat-token",
                serde_json::json!({
                    "token": seg,
                    "model": nombre_modelo_local(),
                }),
            );
            seg.clear();
            n = 0;
        }
    }
    if !seg.is_empty() {
        let _ = app.emit(
            "chat-token",
            serde_json::json!({
                "token": seg,
                "model": nombre_modelo_local(),
            }),
        );
    }

    let _ = app.emit(
        "chat-done",
        serde_json::json!({ "model": nombre_modelo_local() }),
    );
    let _ = app.emit(
        "chat-complete",
        serde_json::json!({
            "response": cleaned,
            "model": nombre_modelo_local(),
        }),
    );
    Ok(cleaned)
}

/// Convierte los mensajes `{role, content}` que manda el frontend a [`Mensaje`].
fn mensajes_serde_a_rust(messages: &[serde_json::Value]) -> Vec<Mensaje> {
    messages
        .iter()
        .map(|m| Mensaje {
            role: m
                .get("role")
                .and_then(|r| r.as_str())
                .unwrap_or("user")
                .to_string(),
            content: m
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect()
}
