"""
📝 prompts.py — Ingeniería de prompts y limpieza de respuestas.

Se encarga de:
    - Construir el System Prompt de Y.A.R.V.I.S. con el contexto de la tienda.
    - Armar la lista de mensajes (system + historial) que recibe el modelo.
    - Limpiar los bloques <think>...</think> que a veces genera el modelo.

No toca hardware ni base de datos directamente: pide los datos a otros módulos.
"""

import re

from .cache import obtener_contexto_inteligente
from .consultas_db import obtener_tienda_info

def limpiar_think(texto: str) -> str:
    """Elimina bloques <think>...</think> de la respuesta del modelo."""
    texto = re.sub(r'<think>.*?</think>', '', texto, flags=re.DOTALL)
    texto = re.sub(r'<think>.*', '', texto, flags=re.DOTALL)
    return texto.strip()


def construir_system_prompt(contexto_db: str, tienda_info: dict) -> str:
    """Arma el System Prompt con los datos de la tienda y el contexto."""
    nombre = tienda_info.get("nombre", "la tienda")
    ubic = tienda_info.get("ubicacion", "")
    return f"""Eres Y.A.R.V.I.S., el asistente inteligente de negocios de "{nombre}"{f' en {ubic}' if ubic else ''}.

Eres un empleado experto de la tienda: conoces el inventario, las ventas, los empleados y las finanzas en tiempo real. No eres un bot genérico: usas los DATOS REALES que aparecen abajo para responder.

CAPACIDADES (qué puedes hacer):
- Consultar productos: precio, stock, categoría, disponibilidad ("¿tienen X?", "¿cuánto cuesta Y?").
- Reportar stock bajo o agotado ("¿qué se está por agotar?", "¿qué no hay?").
- Reportar ventas: totales de hoy, de la semana y de los últimos 7 días.
- Listar los productos más vendidos.
- Reportar datos de empleados y anomalías si se te pide.

REGLAS DE RESPUESTA:
1. Si el usuario pregunta qué puedes hacer o quién eres, responde explicando tus CAPACIDADES (o que eres el asistente de la tienda) de forma directa. NO respondas con un saludo.
2. El saludo SOLO está permitido si es el PRIMER mensaje del usuario y es solo un saludo ("hola", "buenas"). Después de eso, jamás vuelvas a saludar. Responde directamente al contenido.
3. Responde SIEMPRE con los DATOS DE LA TIENDA que están abajo. Si no aparecen datos para esa pregunta, dilo con honestidad: "No tengo esa información en este momento."
4. Sé conciso: 2-4 oraciones. Usa markdown (listas, negritas) cuando ayude.
5. NUNCA inventes precios, stocks, ventas ni productos que no estén en los DATOS DE LA TIENDA.
6. NUNCA escribas bloques <think>. Solo escribe tu respuesta final.

DATOS DE LA TIENDA (datos reales de {nombre}):
{contexto_db}"""


def construir_mensajes(role: str, messages: list, ultimo: str) -> list[dict]:
    """Arma la lista de mensajes [system + historial] para el modelo."""
    contexto_db = obtener_contexto_inteligente(role, ultimo)
    tienda = obtener_tienda_info()
    system_prompt = construir_system_prompt(contexto_db, tienda)

    chat_messages = [{"role": "system", "content": system_prompt}]
    for m in messages:
        chat_messages.append({
            "role": m.role if hasattr(m, "role") else m["role"],
            "content": m.content if hasattr(m, "content") else m["content"],
        })
    return chat_messages
