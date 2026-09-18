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
El resultado llega como mensaje `user` con prefijo "Resultado de <tool>:".
Ejemplo correcto: "Estos son los productos por reabastecer:
1. LACTEOS - stock 0 (minimo 5)
2. PAN - stock 0 (minimo 5)"
Si el resultado viene vacio, dilo claramente: "No hay productos con bajo stock, todo esta surtido."
Si NINGUNA herramienta aplica a la pregunta, respondele directo sin tool_call."#;

/// Tools de abasto y trazabilidad (migraciones 0012/0013/0018).
/// Se APPEND al final del prompt: NO modifica TOOLS_LINEA ni
/// TOOLS_INSTRUCCIONES (el formato exacto del fine-tuning queda intacto).
/// Solo lectura: jamás escriben en la DB. Dinero en pesos.
const TOOLS_ABASTO: &str = r#"
Herramientas de ABASTO y TRAZABILIDAD (ademas de las anteriores):
- query_suppliers: search OPCIONAL (nombre parcial del proveedor), limit opcional; lista proveedores con cuantas compras y total comprado
- query_purchases: date_range OPCIONAL (today/yesterday/this_week/this_month/last_month), proveedor OPCIONAL (nombre parcial), limit opcional; recepciones con monto pagado
- get_purchase_detail: compra_id OBLIGATORIO (id de la recepcion); cabecera + renglones con costo sugerido
- query_purchase_orders: estado OPCIONAL (pendiente/parcial/recibida/cancelada/todas), limit opcional; pedidos con sus renglones y faltantes (cantidad - cantidad_recibida)
- query_cost_history: product_id OBLIGATORIO (nombre del producto); costo actual + cambios registrados + ultimos precios pagados
- query_expiring: dias OPCIONAL (1-365, default 30), product_id OPCIONAL; lotes vencidos y por vencer con dias_restantes
- list_branches: SIN argumentos; sucursales con lineas y stock conjunto (el stock GLOBAL vive en productos)
- query_branch_stock: sucursal OBLIGATORIO (nombre o id; usa list_branches primero), product_id OPCIONAL, limit opcional
Estrategia: "a quien le compro X?" -> query_suppliers con search X; "cuanto gaste en compras?" -> query_purchases; "que pedidos tengo pendientes?" -> query_purchase_orders con estado pendiente; "se va a caducar algo?" -> query_expiring; "hay stock en sucursal Y?" -> list_branches y luego query_branch_stock."#;

/// Subconjunto operativo para el empleado de mostrador (sin montos de
/// compra ni costos: eso lo maneja el administrador).
const TOOLS_ABASTO_EMPLEADO: &str = r#"
Herramientas de ABASTO operativas (ademas de las de inventario):
- query_suppliers: search OPCIONAL (nombre parcial), limit opcional; para localizar telefono y datos de un proveedor
- query_expiring: dias OPCIONAL (1-365, default 30), product_id OPCIONAL; lotes vencidos y por vencer
- list_branches: SIN argumentos; sucursales existentes
- query_branch_stock: sucursal OBLIGATORIO (nombre o id; usa list_branches primero), product_id OPCIONAL
Si pregunta por montos pagados, costos o margenes, explica amablemente que esa informacion solo la maneja el administrador."#;

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
        "{TOOLS_LINEA}{TOOLS_INSTRUCCIONES}{TOOLS_EXTRAS}{TOOLS_ABASTO}
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
        "{TOOLS_LINEA}{TOOLS_INSTRUCCIONES}{TOOLS_EXTRAS}{TOOLS_ABASTO_EMPLEADO}
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

/// SQL libre de solo lectura (issue #15, SOLO admin): el modelo escribe
/// SELECT/WITH contra el snapshot del schema que se anexa. El backend
/// valida (denylist + allowlist + LIMIT≤100) y ejecuta en read-only.
const BLOQUE_SQL_ADMIN: &str = r#"
Herramienta ESPECIAL sql_readonly (solo tú como admin la tienes): cuando
NINGUNA tool anterior sirva, escribe tú el SQL. Formato exacto:
<tool_call>
{"name": "sql_readonly", "arguments": {"query": "SELECT ..."}}
</tool_call>
Reglas duras (si las rompes, la consulta se rechaza y pierdes la ronda):
- Solo SELECT o WITH. Nada de INSERT/UPDATE/DELETE/DDL/PRAGMA ni `;` extra.
- LIMIT entre 1 y 100 SIEMPRE (si lo omites se agrega LIMIT 100).
- Solo estas tablas: ventas, detalle_ventas, productos, cortes_caja,
  movimientos_caja, gastos_recurrentes, pagos_gastos, usuarios,
  asistencias, empleado_horarios, clientes.
- Dinero en INTEGER CENTAVOS: divide entre 100.0 para pesos.
- Solo estado='completada' para ventas reales.
Ejemplos few-shot (adáptalos, no los repitas tal cual):
1. ¿Cuánto vendí hoy?
   SELECT COALESCE(SUM(total),0)/100.0 AS pesos_hoy FROM ventas
   WHERE estado='completada' AND date(fecha)=date('now','localtime')
2. Top 5 por margen en 30 días:
   SELECT d.producto_nombre, SUM(d.cantidad) AS unidades,
     SUM(d.cantidad*(COALESCE(p.precio_venta,0)-COALESCE(p.precio_costo,0)))/100.0 AS margen_pesos
   FROM detalle_ventas d JOIN ventas v ON v.id=d.venta_id
   LEFT JOIN productos p ON p.id=d.producto_id
   WHERE v.estado='completada' AND date(v.fecha)>=date('now','localtime','-30 days')
   GROUP BY d.producto_nombre ORDER BY margen_pesos DESC LIMIT 5
3. Rotación (unidades/día, 7 días):
   SELECT d.producto_nombre, SUM(d.cantidad)/7.0 AS uds_por_dia
   FROM detalle_ventas d JOIN ventas v ON v.id=d.venta_id
   WHERE v.estado='completada' AND date(v.fecha)>=date('now','localtime','-7 days')
   GROUP BY d.producto_nombre ORDER BY uds_por_dia DESC LIMIT 10
Las cifras SIEMPRE salen de tools: jamás inventes números."#;

/// System prompt del admin CON schema vivo de la DB (issue #15).
/// Si el schema viene vacío (falló su lectura), equivale al base.
pub fn construir_system_prompt_admin_con_schema(schema: &str) -> String {
    let base = construir_system_prompt_admin();
    if schema.trim().is_empty() {
        return base;
    }
    format!("{base}\n{BLOQUE_SQL_ADMIN}\n{schema}")
}

/// Arma [system (según rol) + historial] con schema vivo para el admin.
/// Al empleado no se le muestra ni documenta sql_readonly (además el
/// backend se lo bloquea en ejecución).
pub fn construir_mensajes_api_rol_con_schema(
    messages: &[Mensaje],
    es_empleado: bool,
    schema: &str,
) -> Vec<Mensaje> {
    let system = if es_empleado {
        construir_system_prompt_empleado()
    } else {
        construir_system_prompt_admin_con_schema(schema)
    };
    let mut chat = vec![Mensaje::new("system", system)];
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
    fn prompts_abasto_por_rol() {
        let admin = construir_system_prompt_admin();
        for tool in [
            "query_suppliers",
            "query_purchases",
            "get_purchase_detail",
            "query_purchase_orders",
            "query_cost_history",
            "query_expiring",
            "list_branches",
            "query_branch_stock",
        ] {
            assert!(admin.contains(tool), "admin sin {tool}");
        }
        // El formato del fine-tuning sigue intacto.
        assert!(admin.contains("[query_sales, compare_periods, get_top_products, query_inventory, forecast_sales, get_product_info, get_restock_analysis]"));
        let empleado = construir_system_prompt_empleado();
        for tool in ["query_suppliers", "query_expiring", "list_branches", "query_branch_stock"] {
            assert!(empleado.contains(tool), "empleado sin {tool}");
        }
        // Al empleado no se le documentan montos ni costos.
        for tool in ["query_purchases", "get_purchase_detail", "query_purchase_orders", "query_cost_history"] {
            assert!(!empleado.contains(tool), "empleado no debe ver {tool}");
        }
    }

    #[test]
    fn mensaje_vacio_conserva_orden() {
        let historial = vec![Mensaje::new("user", "")];
        let chat = construir_mensajes_api(&historial);
        assert_eq!(chat.len(), 2);
        assert_eq!(chat[1].role, "user");
    }

    #[test]
    fn sql_readonly_solo_en_prompt_admin_con_schema() {
        let hist = vec![Mensaje::new("user", "hola")];
        let sin_schema = construir_mensajes_api_rol_con_schema(&hist, false, "");
        assert!(!sin_schema[0].content.contains("sql_readonly"));
        let admin = construir_mensajes_api_rol_con_schema(
            &hist,
            false,
            "ESQUEMA (SQLite):\n- ventas(id:INTEGER, total:INTEGER)\n",
        );
        assert!(admin[0].content.contains("sql_readonly"));
        assert!(admin[0].content.contains("ventas(id:INTEGER"));
        // Al empleado ni se le menciona, con o sin schema.
        let emp = construir_mensajes_api_rol_con_schema(&hist, true, "ESQUEMA:\n- ventas(id:INTEGER)\n");
        assert!(!emp[0].content.contains("sql_readonly"));
    }
}
