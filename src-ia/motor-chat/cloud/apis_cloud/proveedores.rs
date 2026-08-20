// ============================================================
// proveedores — Streams específicos por proveedor: OpenAI-compatible
// (OpenCode Zen) y Google Gemini. Convierten mensajes + API key en
// un stream SSE de [`Evento`]. Parte de apis_cloud.
// ============================================================

use futures_util::StreamExt;
use reqwest::Client;

use super::super::prompts::Mensaje;
use super::super::variables::{Provider, MAX_TOKENS};
use super::errores::ErrorCloud;
use super::helpers::normalizar_mensajes;
use super::sse::sse_lineas;
use super::tipos::{Evento, Usage};

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
