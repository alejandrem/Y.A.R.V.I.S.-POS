// ============================================================
// catalogo — Listado de modelos de los proveedores (Gemini lista
// todos, OpenCode solo los gratuitos) con caché TTL de 60 s.
// Parte de apis_cloud.
// ============================================================

use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use super::super::variables::{MODELOS_CACHE_TTL_SECS, PROVIDERS};
use super::generacion::cliente;
use super::helpers::es_free;
use super::tipos::ModeloDisponible;

static _MODELOS_CACHE: OnceLock<Mutex<Vec<(String, Instant, Vec<ModeloDisponible>)>>> =
    OnceLock::new();

fn cache_modelos() -> &'static Mutex<Vec<(String, Instant, Vec<ModeloDisponible>)>> {
    _MODELOS_CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

/// Lista los modelos disponibles de un proveedor (solo gratuitos en OpenCode).
/// Devuelve `[{'id', 'name'}]` con caché de 60 segundos.
pub async fn listar_modelos(
    provider: &str,
    api_key: &str,
) -> Result<Vec<ModeloDisponible>, String> {
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
                let raw = m
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let id = raw.strip_prefix("models/").unwrap_or(&raw).to_string();
                let name = m
                    .get("displayName")
                    .and_then(|n| n.as_str())
                    .unwrap_or(&id)
                    .to_string();
                let context_window = m
                    .get("inputTokenLimit")
                    .or_else(|| m.get("contextWindow"))
                    .and_then(|v| v.as_u64());
                ModeloDisponible {
                    id,
                    name,
                    context_window,
                }
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
                let id = m
                    .get("id")
                    .and_then(|i| i.as_str())
                    .unwrap_or("")
                    .to_string();
                if !es_free(&id) {
                    return None;
                }
                let name = m
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or(&id)
                    .to_string();
                let context_window = m
                    .get("context_length")
                    .or_else(|| m.get("context_window"))
                    .or_else(|| m.get("max_input_tokens"))
                    .or_else(|| m.get("input_token_limit"))
                    .and_then(|v| v.as_u64());
                Some(ModeloDisponible {
                    id,
                    name,
                    context_window,
                })
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
