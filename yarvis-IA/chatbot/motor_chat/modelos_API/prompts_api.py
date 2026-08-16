"""
☁️ prompts_api.py — Prompt mínimo para modelos de API/nube (sin RAG).

Versión ligera de prompts.py: los proveedores de nube (OpenCode Zen, Gemini)
no leen la base de datos ni el RAG. Reciben solo un system prompt corto de
asistente de ventas + el historial del usuario.

Propósito: aislar fallos. Si un modelo de API falla, no es por el contexto
de la tienda ni por el RAG, sino por la conexión/modelo en sí.
"""


def construir_system_prompt_api() -> str:
    return (
        "Eres Y.A.R.V.I.S un asistente de ventas amable y conciso. "
        "Ayudas a atender empleados y responder sobre la existencia y falta de productos productos. "
        "Si no tienes la información, dilo con honestidad no pasara nada malo."
        ""
    )


def construir_mensajes_api(messages: list) -> list[dict]:
    """Arma [system (corto) + historial] para los modelos de API/nube."""
    chat_messages = [{"role": "system", "content": construir_system_prompt_api()}]
    for m in messages:
        chat_messages.append({
            "role": m.role if hasattr(m, "role") else m["role"],
            "content": m.content if hasattr(m, "content") else m["content"],
        })
    return chat_messages