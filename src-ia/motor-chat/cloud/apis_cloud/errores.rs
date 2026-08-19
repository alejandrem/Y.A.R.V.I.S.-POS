// ============================================================
// errores — Clasificación de errores del motor cloud (HTTP vs red)
// y espera ante el 429 (rate limit). Parte de apis_cloud.
// ============================================================

use super::super::variables::{ESPERA_429_MAX_SECS, ESPERA_429_MIN_SECS};

/// Error interno del motor cloud (distingue HTTP de red para el relevo 429).
#[derive(Debug)]
pub(crate) enum ErrorCloud {
    Http(u16, Option<String>),
    Red(String),
}

impl ErrorCloud {
    /// ¿Es un 429 (rate limited)? Necesario para decidir el relevo de modelos.
    pub(crate) fn es_429(&self) -> bool {
        matches!(self, ErrorCloud::Http(429, _))
    }

    /// Traduce el error a un mensaje claro en español (para el usuario final).
    pub(crate) fn amigable(&self, display: &str) -> String {
        match self {
            ErrorCloud::Http(status, _) => error_amigable(*status, display),
            ErrorCloud::Red(e) => format!("No se pudo conectar con {display}: {e}"),
        }
    }
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
}