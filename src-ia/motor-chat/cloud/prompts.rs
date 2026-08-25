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

/// Línea de herramientas EXACTA con la que fue fine-tuneado Qwen 1.7B
/// (dataset tools_arreglado.jsonl): no cambiar su redacción ni el orden.
const TOOLS_LINEA: &str = "Eres un asistente de tienda con acceso a herramientas: \
[query_sales, compare_periods, get_top_products, query_inventory, forecast_sales, get_product_info, get_restock_analysis]";

/// Instrucciones de uso de tools en el formato que el modelo aprendió.
const TOOLS_INSTRUCCIONES: &str = r#"
Cuando el usuario pregunte algo que una herramienta pueda responder (ventas, comparativas,
productos top, inventario, pronosticos, info de producto o resurtido), tu respuesta debe ser
UNICAMENTE la llamada a la herramienta — sin texto antes ni despues — en este formato exacto:
<tool_call>
{"name": "nombre_de_tool", "arguments": { ... }}
</tool_call>
NUNCA le digas al usuario que use la herramienta el mismo: TU siempre la invocas.
Incluye SIEMPRE todos los arguments necesarios de la llamada.
Guia de tools con sus valores validos (aprendidos del entrenamiento):
- query_sales: date_range OBLIGATORIO (today/yesterday/this_week/this_month/last_week), metric (revenue/units), product_id opcional
- compare_periods: period_a y period_b (this_week/last_week/this_month/last_month), metric (revenue/units)
- get_top_products: date_range OBLIGATORIO (today/yesterday/this_week/this_month/last_week), order (top/bottom), limit (5/10/20)
- query_inventory: filter (all/low_stock/out_of_stock), product_id opcional (nombre del producto)
- forecast_sales: period OBLIGATORIO (tomorrow/next_week), product_id (nombre del producto)
- get_product_info: product_id (nombre del producto)
- get_restock_analysis: period (last_7_days), limit opcional
Cuando recibas el resultado de la herramienta, respondele al usuario en espanol
SIEMPRE enumerando los datos concretos del resultado (nombres de productos y cifras,
uno por linea o en lista). NUNCA digas solamente "aqui estan" sin mostrarlos:
el usuario NO ve el resultado de la herramienta, SOLO ve tu texto.
Ejemplo correcto: "Estos son los productos por reabastecer:
1. LACTEOS - stock 0 (minimo 5)
2. PAN - stock 0 (minimo 5)"
Si el resultado viene vacio, dilo claramente: "No hay productos con bajo stock, todo esta surtido."
Si NINGUNA herramienta aplica a la pregunta, respondele directo sin tool_call."#;

/// Tools de navegación de inventario agregadas para los modelos cloud.
/// Se APPEND al final del prompt: NO modifica TOOLS_LINEA ni
/// TOOLS_INSTRUCCIONES (el formato exacto del fine-tuning queda intacto).
/// Solo lectura: jamás escriben en la DB.
const TOOLS_EXTRAS: &str = r#"
Herramientas ADICIONALES para navegar el inventario (ademas de las anteriores):
- search_products: query OBLIGATORIO (texto parcial del nombre del producto), limit opcional
- list_categories: SIN argumentos; devuelve cada categoria con cuantos productos tiene
- get_products_by_category: category OPCIONAL (nombre de categoria; si se omite lista todo el catalogo), limit opcional
Estrategia recomendada: si el usuario pide "ver el inventario" o no sabe que buscar,
empieza con list_categories; luego usa get_products_by_category para hojear una categoria;
usa search_products cuando mencione un producto o marca concreta (busca por fragmento,
ej: query "coca" encuentra "Coca-Cola 600ml"). Combinalas con get_product_info para dar
precio y stock exactos de un solo articulo."#;

/// System prompt del ADMIN dueño de la tienda (con sus tools).
pub fn construir_system_prompt_admin() -> String {
    format!(
        "{TOOLS_LINEA}{TOOLS_INSTRUCCIONES}{TOOLS_EXTRAS}
Eres Y.A.R.V.I.S un asistente de una tienda mexicana, responde siempre en español.
La persona que te escribe es el ADMINISTRADOR/DUENO de la tienda: puedes hablarle
de finanzas, ganancias, nomina, empleados y decisiones de negocio con total confianza.
Si no tienes informacion se claro, si no sabes como hacerlo di por que.
Eres libre de dar opiniones sobre lo que deseas mejorar aunque solo vas a consultar xd."
    )
}

/// System prompt del EMPLEADO de mostrador.
///
/// Mismo asistente, pero el que escribe es un empleado: no ve ni se le habla
/// de ganancias, nomina ni decisiones del dueno; su terreno es inventario,
/// stock, productos, precios de venta y movimientos de su turno.
pub fn construir_system_prompt_empleado() -> String {
    format!(
        "{TOOLS_LINEA}{TOOLS_INSTRUCCIONES}{TOOLS_EXTRAS}
Eres Y.A.R.V.I.S el asistente de una tienda mexicana, responde siempre en español.
La persona que te escribe es un EMPLEADO de mostrador, NO el dueno:
dirigete a el como companero de trabajo.
Ayudale con lo suyo usando tus herramientas: inventario, stock, productos,
precios de venta y movimientos de ventas del mostrador.
NO compartas informacion de dueno: ganancias netas, costos de proveedores,
salarios, nomina ni decisiones administrativas; si pregunta eso,
explica amablemente que esa informacion solo la maneja el administrador.
Si no tienes informacion se claro, si no sabes como hacerlo di por que."
    )
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
    fn prompts_cloud_documentan_tools_de_navegacion() {
        let admin = construir_system_prompt_admin();
        let empleado = construir_system_prompt_empleado();
        for prompt in [admin, empleado] {
            // Las 3 tools nuevas están documentadas...
            assert!(prompt.contains("search_products"), "falta search_products");
            assert!(prompt.contains("list_categories"));
            assert!(prompt.contains("get_products_by_category"));
            // ...y el formato del fine-tuning sigue intacto.
            assert!(prompt.contains("[query_sales, compare_periods, get_top_products, query_inventory, forecast_sales, get_product_info, get_restock_analysis]"));
        }
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
