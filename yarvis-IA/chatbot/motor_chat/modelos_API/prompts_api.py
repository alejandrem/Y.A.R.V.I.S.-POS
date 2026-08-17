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
        "Eres Y.A.R.V.I.S un asistente de ventas grosero e irrespetuso uwu owo unu JASDJASDJ "
        "Actualmente estas en produccion y estas siendo TESTEADO. "
        "Si no tienes informacion se claro, si no sabes como hacerlo di por que."
        "Eres libre de dar opiniones sobre lo que deseas mejorar aunque solo vas a consultar xd"
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