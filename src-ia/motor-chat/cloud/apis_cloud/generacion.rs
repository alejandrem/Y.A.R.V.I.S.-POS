// ============================================================
// generacion — API pública del motor cloud: `generar_stream` y
// `generar_completo`. HTTP client compartido y relevo 429 entre
// modelos (cola de modelos free de OpenCode). Parte de apis_cloud.
// ============================================================

use std::time::Duration;

use futures_util::StreamExt;
use reqwest::Client;

use super::super::prompts::Mensaje;
use super::super::think::{limpiar_think, SeparadorThink, TipoFragmento};
use super::super::variables::{PROVIDERS, TIMEOUT_CONNECT_SECS, TIMEOUT_IDLE_SECS};
use super::errores::{espera_429, ErrorCloud};
use super::helpers::cola_modelos_a_probar;
use super::proveedores::stream_modelo;
use super::tipos::Evento;

/// HTTP client compartido por toda la API.
///
/// Sin timeout GLOBAL: con SSE, `Client::timeout` aplica al ciclo completo
/// petición+cuerpo y mataba generaciones largas (>120 s) a mitad de stream.
/// En su lugar, `read_timeout` limita el SILENCIO entre chunks: un stream
/// vivo nunca se corta; un servidor colgado se detecta a los IDLE segundos.
pub(crate) fn cliente() -> Client {
    Client::builder()
        .connect_timeout(Duration::from_secs(TIMEOUT_CONNECT_SECS))
        .read_timeout(Duration::from_secs(TIMEOUT_IDLE_SECS))
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
                            cedio = true;
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
                tracing::warn!(
                    "[YARVIS] {modelo} saturado (429), cambiando a {sig} (espera {espera}s)"
                );
                tokio::time::sleep(Duration::from_secs(espera)).await;
                continue;
            }

            // Último modelo: un respiro y un último intento real.
            tracing::warn!("[YARVIS] {modelo} saturado (429), último intento tras {espera}s");
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
