// ============================================================
// proveedores — Streams específicos por proveedor: OpenAI-compatible
// (OpenCode Zen) y Google Gemini. Convierten mensajes + API key en
// un stream SSE de [`Evento`]. Parte de apis_cloud.
// ============================================================

use futures_util::StreamExt;
use reqwest::Client;

use super::super::prompts::Mensaje;
use super::super::variables::{Provider, MAX_TOKENS, MAX_TOKENS_GOOGLE};
use super::errores::ErrorCloud;
use super::helpers::normalizar_mensajes;
use super::sse::sse_lineas;
use super::tipos::{Evento, Usage};

/// ID de sesión para el free tier de Zen.
 ///
 /// Zen exige un session id en `POST /chat/completions`: sin él responde
 /// 400 MissingSessionID ("free tier can only be used in OpenCode") para
 /// TODO modelo free (verificado 2026-09-13/15 con sondas directas).
 /// El CLI oficial manda `x-opencode-session` (+ `x-opencode-client`,
 /// `x-opencode-request`); con terceros acepta también el alias
 /// `X-Session-Id`. Mandamos AMBOS para máxima compatibilidad.
 ///
 /// Se genera UNO NUEVO POR REQUEST (no estable por proceso): un id estable
 /// puede quedar clavado por sticky-routing en una réplica rota y fallar
 /// el 100% de las veces hasta reiniciar (ver sst/opencode#46011).
 /// Fresco por request pierde algo de prompt-caching pero gana fiabilidad.
fn nueva_sesion_zen() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static CONTADOR: AtomicU64 = AtomicU64::new(0);
    let n = CONTADOR.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("yarvis-{:x}-{:x}-{:x}", std::process::id(), (nanos & 0xffff_ffff_ffff) ^ ((n as u128) << 32), n)
}

/// Modelos que NO hablan `/chat/completions` aunque salgan en `/models`.
/// `muse-spark-*-contributor-free` usa `/responses` (`@ai-sdk/openai` según
/// docs de Zen); mandarlo a chat/completions da 500 Internal server error.
/// Se rechaza temprano con mensaje accionable en vez de romper el stream.
fn es_solo_responses(modelo: &str) -> bool {
    modelo.starts_with("muse-spark")
}
/// Genera el stream de un modelo específico (gemini u openai-compatible).
///
/// Espejo de `_iter_openai_compatible` + `_iter_google`: las respuestas llegan
/// como texto crudo que puede traer los marcadores ` think ... response `
/// (el caller los separa con `SeparadorThink`).
pub(crate) async fn stream_modelo<'a>(
    cfg: &'a Provider,
    client: &'a Client,
    api_key: &'a str,
    modelo: &'a str,
    messages: &'a [Mensaje],
) -> std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<Evento, ErrorCloud>> + Send + 'a>> {
    if cfg.key == "google" {
        Box::pin(stream_google(cfg, client, api_key, modelo, messages).await)
    } else {
        Box::pin(stream_openai_compatible(cfg, client, api_key, modelo, messages).await)
    }
}

/// OpenCode Zen (y cualquiera compatible con /chat/completions). Sin tools.
pub(crate) async fn stream_openai_compatible<'a>(
    cfg: &'a Provider,
    client: &'a Client,
    api_key: &'a str,
    modelo: &'a str,
    messages: &'a [Mensaje],
) -> impl futures_util::Stream<Item = Result<Evento, ErrorCloud>> + 'a {
    async_stream::stream! {
        // Guard temprano: /responses-only a /chat/completions da 500.
        if cfg.key == "opencode" && es_solo_responses(modelo) {
            yield Err(ErrorCloud::Http {
                status: 400,
                retry_after: None,
                body: Some(format!(
                    "El modelo '{modelo}' usa el endpoint /responses de Zen (no /chat/completions). Elige uno chat-compatible: nemotron-3-ultra-free, mimo-v2.5-free o big-pickle."
                )),
            });
            return;
        }
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

            let mut req = client
                .post(&url)
                .header("Authorization", format!("Bearer {api_key}"));
            // Zen free tier exige session id. Se mandan AMBOS headers:
            // `x-opencode-session` (oficial CLI) + `X-Session-Id` (alias
            // terceros) + `x-opencode-client`. Id fresco por request para
            // no clavar sticky-routing en réplica rota.
            if cfg.key == "opencode" {
                let ses = nueva_sesion_zen();
                req = req
                    .header("x-opencode-session", ses.clone())
                    .header("X-Session-Id", ses)
                    .header("x-opencode-client", "yarvis-pos");
            }
            let resp = match req.json(&body).send().await {
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
                // El cuerpo trae el motivo real (p. ej. qué modelo falló
                // aguas arriba): conservarlo, antes se perdía.
                let body = resp.text().await.unwrap_or_default();
                tracing::warn!("[YARVIS] {} devolvió {}: {}", cfg.name, status.as_u16(), body.chars().take(500).collect::<String>());
                yield Err(ErrorCloud::Http {
                    status: status.as_u16(),
                    retry_after: retry,
                    body: Some(body),
                });
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

                // Zen/OpenRouter manda `reasoning`; otros compatibles mandan
                // `reasoning_content`. Soportar ambos o el pensamiento se pierde.
                let razonamiento = delta
                    .get("reasoning_content")
                    .or_else(|| delta.get("reasoning"))
                    .and_then(|v| v.as_str());
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
pub(crate) async fn stream_google<'a>(
    cfg: &'a Provider,
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
            // Techo propio de Gemini: MAX_TOKENS (39800) excede el límite de
            // salida de los modelos flash y provoca 400 en cada llamada.
            "generationConfig": {
                "maxOutputTokens": MAX_TOKENS_GOOGLE,
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
            let status = resp.status().as_u16();
            let retry = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!("[YARVIS] {} devolvió {}: {}", cfg.name, status, body.chars().take(500).collect::<String>());
            yield Err(ErrorCloud::Http {
                status,
                retry_after: retry,
                body: Some(body),
            });
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
