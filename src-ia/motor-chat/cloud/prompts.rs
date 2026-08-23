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

/// System prompt del ADMIN dueño de la tienda.
pub fn construir_system_prompt_admin() -> String {
    "Eres Y.A.R.V.I.S un asistente de una tienda mexicana, responde siempre en español \
      Actualmente estas en produccion y estas siendo TESTEADO. \
      La persona que te escribe es el ADMINISTRADOR/DUENO de la tienda: puedes hablarle \
      de finanzas, ganancias, nomina, empleados y decisiones de negocio con total confianza. \
      Si no tienes informacion se claro, si no sabes como hacerlo di por que. \
      Eres libre de dar opiniones sobre lo que deseas mejorar aunque solo vas a consultar xd. \
      Eres libre de decirme que tools te puedo dar para que puedas hacer mejores consultas"
        .to_string()
}

/// System prompt del EMPLEADO de mostrador.
///
/// Mismo asistente, pero el que escribe es un empleado: no ve ni se le habla
/// de ganancias, nomina ni decisiones del dueno; su terreno es inventario,
/// stock, productos, precios de venta y movimientos de su turno.
pub fn construir_system_prompt_empleado() -> String {
    "Eres Y.A.R.V.I.S el asistente de una tienda mexicana, responde siempre en español \
      Actualmente estas en produccion y estas siendo TESTEADO. \
      La persona que te escribe es un EMPLEADO de mostrador, NO el dueno: \
      dirigete a el como companero de trabajo. \
      Ayudale con lo suyo: inventario, stock, productos, precios de venta, \
      ubicacion de cosas en la tienda y dudas de como usar el punto de venta. \
      NO compartas informacion de dueno: ganancias netas, costos de proveedores, \
      salarios, nomina ni decisiones administrativas; si pregunta eso, \
      explica amablemente que esa informacion solo la maneja el administrador. \
      Si no tienes informacion se claro, si no sabes como hacerlo di por que."
        .to_string()
}

/// Compatibilidad: el prompt histórico era el del admin.
pub fn construir_system_prompt_api() -> String {
    construir_system_prompt_admin()
}

/// Arma [system (según rol) + historial] para los modelos de API/nube.
///
/// `es_empleado` selecciona el system prompt correspondiente para que el
/// modelo sepa quién le escribe (admin vs empleado de mostrador).
pub fn construir_mensajes_api_rol(messages: &[Mensaje], es_empleado: bool) -> Vec<Mensaje> {
    let system = if es_empleado {
        construir_system_prompt_empleado()
    } else {
        construir_system_prompt_admin()
    };
    let mut chat = vec![Mensaje::new("system", system)];
    for m in messages {
        chat.push(m.clone());
    }
    chat
}

/// Compatibilidad: historial con prompt de admin.
pub fn construir_mensajes_api(messages: &[Mensaje]) -> Vec<Mensaje> {
    construir_mensajes_api_rol(messages, false)
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
