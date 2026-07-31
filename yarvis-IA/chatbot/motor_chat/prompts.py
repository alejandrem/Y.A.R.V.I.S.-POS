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

COMPLEX_KEYWORDS = [
    "anomal", "reembolso", "estornad", "comparar", "tendencia",
    "predicc", "analizar", "análisis", "promedio", "estadístic",
    "rentabilidad", "utilidad", "margen", "ganancia", "pérdida",
    "robo", "sospech", "inusual", "raro", "diferente",
]


def limpiar_think(texto: str) -> str:
    """Elimina bloques <think>...</think> de la respuesta del modelo."""
    texto = re.sub(r'<think>.*?</think>', '', texto, flags=re.DOTALL)
    texto = re.sub(r'<think>.*', '', texto, flags=re.DOTALL)
    return texto.strip()


def es_pregunta_compleja(texto: str) -> bool:
    """Detecta si la pregunta requiere un modelo más grande."""
    return any(kw in texto.lower() for kw in COMPLEX_KEYWORDS)


def construir_system_prompt(contexto_db: str, tienda_info: dict) -> str:
    """Arma el System Prompt con los datos de la tienda y el contexto."""
    nombre = tienda_info.get("nombre", "la tienda")
    ubic = tienda_info.get("ubicacion", "")
    return f"""Eres Y.A.R.V.I.S., el asistente inteligente de "{nombre}"{f' en {ubic}' if ubic else ''}.

Sé amable, profesional y cercano. Habla como un buen empleado de confianza, no como un robot.

REGLAS:
- Si el usuario solo saluda o hace plática casual, responde de forma amable y breve. No exijas datos de la tienda.
- Si pregunta por productos, ventas o inventario, responde con los datos de abajo.
- Sé conciso (2-4 oraciones máximo). Usa markdown cuando ayude.
- NUNCA escribas bloques <think>. Solo escribe la respuesta final directa.

EJEMPLOS DE SALUDOS:
- "Hola, ¿en qué te puedo ayudar?"
- "Hey! Aquí estoy para lo que necesites."
- "¡Hola! Pregúntame lo que quieras sobre la tienda."

DATOS DE LA TIENDA:
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
