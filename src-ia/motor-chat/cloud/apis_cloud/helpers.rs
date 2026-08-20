// ============================================================
// helpers — Utilidades de proveedor: nombre amigable, detección
// de modelos free, cola de relevo 429 y normalización de mensajes.
// Parte de apis_cloud (espejo de apis_cloud.py).
// ============================================================

use super::super::prompts::Mensaje;
use super::super::variables::{
    MAX_MODELOS_A_PROBAR, MODELOS_FREE_EXTRA, ORDEN_FALLBACK_FREE, PROVIDERS,
};

/// Nombre amigable del proveedor (para mostrarlo en el modelo usado).
pub fn nombre_proveedor(provider: &str) -> String {
    PROVIDERS
        .iter()
        .find(|p| p.key == provider)
        .map(|p| p.name.to_string())
        .unwrap_or_else(|| provider.to_string())
}

/// Un modelo de OpenCode es gratuito si termina en '-free' o está en la lista extra.
pub(crate) fn es_free(model_id: &str) -> bool {
    model_id.ends_with("-free") || MODELOS_FREE_EXTRA.contains(&model_id)
}

/// Orden de los modelos a probar cuando el proveedor satura (429).
///
/// Espejo de `_cola_modelos_a_probar`: para OpenCode free arranca por el modelo
/// pedido y recorre `ORDEN_FALLBACK_FREE` limitado a `MAX_MODELOS_A_PROBAR`; para
/// cualquier otra combinación solo el modelo original.
pub(crate) fn cola_modelos_a_probar(provider: &str, model: &str) -> Vec<String> {
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

/// Junta mensajes consecutivos con el mismo rol (evita rechazos de APIs).
///
/// Espejo de `_normalizar_mensajes` (sin las ramas de tool_calls, ya que el
/// motor cloud ya no usa function calling).
pub(crate) fn normalizar_mensajes(messages: &[Mensaje]) -> Vec<Mensaje> {
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
        assert_eq!(
            cola_modelos_a_probar("opencode", "gemini-2.0-flash"),
            vec!["gemini-2.0-flash"]
        );
        assert_eq!(
            cola_modelos_a_probar("google", "mimo-v2.5-free"),
            vec!["mimo-v2.5-free"]
        );
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
}
