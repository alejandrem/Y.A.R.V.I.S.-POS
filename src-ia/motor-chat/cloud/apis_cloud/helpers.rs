// ============================================================
// helpers — Utilidades de proveedor: nombre amigable y normalización
// de mensajes. Parte de apis_cloud (espejo de apis_cloud.py).
// ============================================================

use super::super::prompts::Mensaje;
use super::super::variables::PROVIDERS;

/// Nombre amigable del proveedor (para mostrarlo en el modelo usado).
pub fn nombre_proveedor(provider: &str) -> String {
    PROVIDERS
        .iter()
        .find(|p| p.key == provider)
        .map(|p| p.name.to_string())
        .unwrap_or_else(|| provider.to_string())
}

/// Modelos a probar cuando el proveedor satura (429): hoy siempre el
/// pedido (el relevo multi-modelo murió con OpenCode). Se conserva la
/// forma de cola porque `generacion.rs` reintenta sobre ella.
pub(crate) fn cola_modelos_a_probar(_provider: &str, model: &str) -> Vec<String> {
    vec![model.to_string()]
}

/// Junta mensajes consecutivos con el mismo rol (evita rechazos de APIs).
///
/// Los gateways OpenAI-compatibles rechazan `role:"tool"` sin `tool_calls`
/// previos con `400 Upstream request failed`. Como YARVIS usa tools
/// textuales `<tool_call>` (sin function calling nativo), el resultado se
/// reinyecta como `user` con prefijo `Resultado de ...:` para que el
/// modelo lo entienda en la ronda 2.
/// El modelo local (llama.cpp) sigue recibiendo `role:"tool"` sin pasar
/// por aquí.
pub(crate) fn normalizar_mensajes(messages: &[Mensaje]) -> Vec<Mensaje> {
    let mut normalized: Vec<Mensaje> = Vec::new();
    for m in messages {
        let mut role = if m.role.is_empty() { "user" } else { m.role.as_str() };
        let mut content = m.content.clone();
        // El gateway no acepta "tool" suelto: va como "user".
        if role == "tool" {
            role = "user";
            if !content.starts_with("Resultado de") {
                content = format!("Resultado de herramienta:\n{content}");
            }
        }
        if let Some(last) = normalized.last_mut() {
            if last.role == role {
                last.content.push('\n');
                last.content.push_str(&content);
                continue;
            }
        }
        normalized.push(Mensaje {
            role: role.to_string(),
            content,
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
        assert_eq!(nombre_proveedor("nope"), "nope");
    }

    #[test]
    fn cola_siempre_es_solo_el_modelo_pedido() {
        assert_eq!(
            cola_modelos_a_probar("google", "gemini-3.5-flash-lite"),
            vec!["gemini-3.5-flash-lite"]
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

    #[test]
    fn normalizar_convierte_tool_a_user() {
        let msgs = vec![
            Mensaje::new("user", "cuanto vendi hoy?"),
            Mensaje::new("assistant", "<tool_call>{\"name\":\"query_sales\"}</tool_call>"),
            Mensaje::new("tool", "{\"ventas_totales\":300.0}"),
        ];
        let norm = normalizar_mensajes(&msgs);
        assert!(norm.iter().all(|m| m.role != "tool"), "nada de role tool al gateway");
        assert_eq!(norm[2].role, "user");
        assert!(norm[2].content.starts_with("Resultado de"));
    }
}
