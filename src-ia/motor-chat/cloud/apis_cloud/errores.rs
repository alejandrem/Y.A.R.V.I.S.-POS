// ============================================================
// errores — Clasificación de errores del motor cloud (HTTP vs red)
// y espera ante el 429 (rate limit). Parte de apis_cloud.
// ============================================================

use super::super::variables::{ESPERA_429_MAX_SECS, ESPERA_429_MIN_SECS};

/// Error interno del motor cloud (distingue HTTP de red para el relevo 429).
/// En HTTP se conserva el cuerpo de la respuesta (truncado al mostrar) para
/// no perder el mensaje real del servidor: antes solo se guardaba el status
/// y un 500 era imposible de diagnosticar.
#[derive(Debug)]
pub(crate) enum ErrorCloud {
    Http {
        status: u16,
        retry_after: Option<String>,
        body: Option<String>,
    },
    Red(String),
}

impl ErrorCloud {
    /// ¿Es un 429 (rate limited)? Necesario para decidir el relevo de modelos.
    pub(crate) fn es_429(&self) -> bool {
        matches!(self, ErrorCloud::Http { status: 429, .. })
    }

    /// Traduce el error a un mensaje claro en español (para el usuario final).
    pub(crate) fn amigable(&self, display: &str) -> String {
        match self {
            ErrorCloud::Http { status, body, .. } => {
                error_amigable(*status, display, body.as_deref())
            }
            ErrorCloud::Red(e) => format!("No se pudo conectar con {display}: {e}"),
        }
    }
}

/// Traduce errores HTTP del proveedor a mensajes claros en español.
/// Si el servidor mandó detalle en el cuerpo, se anexa (recortado).
fn error_amigable(status: u16, display: &str, body: Option<&str>) -> String {
    // Bloqueo de free tier de Zen a apps de terceros. Dos variantes vistas
    // en campo: 400 MissingSessionID ("can only be used in OpenCode") y
    // 403 ("can only be used from within OpenCode", a veces sin el tipo
    // MissingSessionID en el cuerpo). El tipo del servidor es críptico y
    // —peor— el genérico de abajo diría "API key inválida" cuando la key
    // SÍ sirve: traducirlo a acción.
    if display == "OpenCode"
        && body
            .map(|b| b.contains("MissingSessionID") || b.contains("free tier can only be used"))
            .unwrap_or(false)
    {
        return "Zen bloqueó el free tier: solo funciona dentro de OpenCode (tu key sí sirve). Cambia al proveedor Gemini con tu clave de Google, o usa el modelo local Qwen.".to_string();
    }
    // Key válida pero sin fondos: Zen responde 401 CreditsError "No payment
    // method" (verificado en campo 2026-09-19). No es key inválida: es
    // cartera vacía. No confundir con un 401 de auth real.
    if display == "OpenCode"
        && body
            .map(|b| b.contains("CreditsError") || b.contains("No payment method"))
            .unwrap_or(false)
    {
        return "Tu key de Zen sí sirve pero no tiene fondos: agrega método de pago en opencode.ai, o cambia al proveedor Gemini (gratis) o al Qwen local.".to_string();
    }
    let base = match status {
        429 => {
            "Error 429: muchas preguntas al mismo tiempo. Espera 1 minuto y reintenta.".to_string()
        }
        401 | 403 => {
            format!("API key inválida (error {status}). Revisa tu clave en 'Agregar API'.")
        }
        402 => "Cuota agotada (error 402): revisa tu plan del proveedor.".to_string(),
        404 => {
            format!("Modelo no disponible (error 404). Revisa el nombre del modelo del proveedor.")
        }
        _ => format!("Error {status} del proveedor ({display})."),
    };
    match recorte_mensaje(body) {
        Some(detalle) => format!("{base} Detalle del servidor: {detalle}"),
        None => base,
    }
}

/// Extrae un extracto legible del cuerpo de error (máx. 600 caracteres).
/// Si es JSON, prefiere `error.message` / `error` / `message`.
fn recorte_mensaje(body: Option<&str>) -> Option<String> {
    let body = body?.trim();
    if body.is_empty() {
        return None;
    }
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        for pointer in ["/error/message", "/error", "/message"] {
            if let Some(msg) = json.pointer(pointer).and_then(|v| v.as_str()) {
                let msg = msg.trim();
                if !msg.is_empty() {
                    return Some(recortar(msg, 600));
                }
            }
        }
    }
    Some(recortar(body, 600))
}

fn recortar(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

/// Segundos a esperar ante un 429: respeta retry-after clavado al rango.
pub(crate) fn espera_429(retry_after: Option<&str>) -> u64 {
    let retry = retry_after.unwrap_or("");
    if let Ok(n) = retry.parse::<u64>() {
        return n.clamp(ESPERA_429_MIN_SECS, ESPERA_429_MAX_SECS);
    }
    (ESPERA_429_MIN_SECS + ESPERA_429_MAX_SECS) / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn espera_429_respeta_retry_after_clavado() {
        assert_eq!(espera_429(Some("1")), 2);
        assert_eq!(espera_429(Some("99")), 4);
        assert_eq!(espera_429(Some("3")), 3);
        assert_eq!(espera_429(None), 3);
    }

    #[test]
    fn http_500_anexa_detalle_del_servidor() {
        let err = ErrorCloud::Http {
            status: 500,
            retry_after: None,
            body: Some(r#"{"error":{"message":"upstream overloaded"}}"#.to_string()),
        };
        assert!(!err.es_429());
        let msg = err.amigable("OpenCode");
        assert!(msg.contains("Error 500"), "conserva el status: {msg}");
        assert!(msg.contains("upstream overloaded"), "anexa el detalle: {msg}");
    }

    #[test]
    fn http_500_sin_cuerpo_sigue_sienco_claro() {
        let err = ErrorCloud::Http {
            status: 500,
            retry_after: None,
            body: None,
        };
        assert_eq!(err.amigable("OpenCode"), "Error 500 del proveedor (OpenCode).");
    }

    #[test]
    fn bloqueo_free_tier_zen_se_traduce_a_accion() {
        let err = ErrorCloud::Http {
            status: 400,
            retry_after: None,
            body: Some(
                r#"{"type":"error","error":{"type":"MissingSessionID","message":"Error from provider (Console): OpenCode's free tier can only be used in OpenCode"}}"#.to_string(),
            ),
        };
        assert_eq!(
            err.amigable("OpenCode"),
            "Zen bloqueó el free tier: solo funciona dentro de OpenCode (tu key sí sirve). Cambia al proveedor Gemini con tu clave de Google, o usa el modelo local Qwen."
        );
    }

    #[test]
    fn bloqueo_free_tier_zen_403_within_tambien_es_bloqueo_no_key_invalida() {
        // Variante vista en campo (2026-09-19): 403 sin MissingSessionID.
        // Antes caía al genérico "API key inválida" con key válida.
        let err = ErrorCloud::Http {
            status: 403,
            retry_after: None,
            body: Some(
                r#"{"error":"Error from provider (Console): OpenCode's free tier can only be used from within OpenCode"}"#.to_string(),
            ),
        };
        let msg = err.amigable("OpenCode");
        assert!(msg.contains("tu key sí sirve"), "no debe culpar a la key: {msg}");
        assert!(!msg.contains("inválida"), "cero mención a key inválida: {msg}");
    }

    #[test]
    fn error_403_de_otro_proveedor_sigue_siendo_key_invalida() {
        let err = ErrorCloud::Http {
            status: 403,
            retry_after: None,
            body: Some(r#"{"error":{"message":"invalid api key"}}"#.to_string()),
        };
        let msg = err.amigable("Google");
        assert!(msg.contains("API key inválida (error 403)"), "{msg}");
    }

    #[test]
    fn error_401_sin_fondos_zen_no_es_key_invalida() {
        // Verificado en campo 2026-09-19 con key gratuita real: 401
        // CreditsError "No payment method" (free bloqueado + sin fondos).
        let err = ErrorCloud::Http {
            status: 401,
            retry_after: None,
            body: Some(
                r#"{"type":"error","error":{"type":"CreditsError","message":"No payment method. Add a payment method here: https://opencode.ai/billing"}}"#.to_string(),
            ),
        };
        let msg = err.amigable("OpenCode");
        assert!(msg.contains("no tiene fondos"), "{msg}");
        assert!(!msg.contains("inválida"), "cero mención a key inválida: {msg}");
    }

    #[test]
    fn error_401_auth_real_sigue_siendo_key_invalida() {
        let err = ErrorCloud::Http {
            status: 401,
            retry_after: None,
            body: Some(r#"{"error":{"message":"incorrect api key"}}"#.to_string()),
        };
        let msg = err.amigable("OpenCode");
        assert!(msg.contains("API key inválida (error 401)"), "{msg}");
    }
}
