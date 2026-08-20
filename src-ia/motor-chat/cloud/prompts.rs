//! prompts.rs — Prompt mínimo para modelos de API/nube (sin RAG ni tools).
//!
//! Versión ligera de prompts.py: los proveedores de nube (OpenCode Zen, Gemini)
//! no leen la base de datos ni el RAG. Reciben solo un system prompt corto de
//! asistente de ventas + el historial del usuario.
//! Espejo de `yarvis-IA/chatbot/motor_chat/modelos_API/prompts_api.py`.

use serde::{Deserialize, Serialize};

/// Mensaje del chat en formato {role, content} estándar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mensaje {
    pub role: String,
    pub content: String,
}

impl Mensaje {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Mensaje {
            role: role.into(),
            content: content.into(),
        }
    }
}

/// System prompt de los modelos API (espejo de `construir_system_prompt_api`).
pub fn construir_system_prompt_api() -> String {
"Eres Y.A.R.V.I.S un asistente de una tienda mexicana, responde siempre en español \
      Actualmente estas en produccion y estas siendo TESTEADO. \
      Si no tienes informacion se claro, si no sabes como hacerlo di por que. \
      Eres libre de dar opiniones sobre lo que deseas mejorar aunque solo vas a consultar xd. \
      Eres libre de decirme que tools te puedo dar para que puedas hacer mejores consultas"
        .to_string()
}

/// Arma [system (corto) + historial] para los modelos de API/nube.
///
/// Espejo de `construir_mensajes_api`: prepende el system prompt y conserva
/// el historial tal cual viene del frontend.
pub fn construir_mensajes_api(messages: &[Mensaje]) -> Vec<Mensaje> {
    let mut chat = vec![Mensaje::new("system", construir_system_prompt_api())];
    for m in messages {
        chat.push(m.clone());
    }
    chat
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mensajes_api_prepende_system_prompt() {
        let historial = vec![
            Mensaje::new("user", "hola"),
            Mensaje::new("assistant", "hola!"),
        ];
        let chat = construir_mensajes_api(&historial);
        assert_eq!(chat.len(), 3);
        assert_eq!(chat[0].role, "system");
        assert!(chat[0].content.contains("Y.A.R.V.I.S"));
        assert_eq!(chat[1].content, "hola");
        assert_eq!(chat[2].content, "hola!");
    }

    #[test]
    fn mensaje_vacio_conserva_orden() {
        let historial = vec![Mensaje::new("user", "")];
        let chat = construir_mensajes_api(&historial);
        assert_eq!(chat.len(), 2);
        assert_eq!(chat[1].role, "user");
        assert_eq!(chat[1].content, "");
    }
}