"""
chatbot/motor_chat — Paquete del motor de chat de Y.A.R.V.I.S.

Divide el antiguo motor_chat.py (666 líneas) en módulos por responsabilidad,
agrupados en una subcarpeta según el tipo de modelo:

    modelos_local/  → 🧠 Modelos LOCALES (Qwen + RAG)
        consultas_db.py      → 🗄️ base de datos (SQLite)
        gestion_hardware.py  → 🧠 RAM + ciclo de vida de los modelos Qwen
        motor_rag.py         → 🔍 embeddings + búsqueda semántica (RAG)
        prompts.py           → 📝 system prompts + contexto de la tienda
        cache.py             → ⏱️ caché de inventario + hilos de fondo

    endpoints.py     → 🌐 endpoints HTTP (FastAPI) para el modo local

> Nota: el modo nube (Gemini, OpenCode Zen) vive en Rust (`src-ia`,
> `motor-chat/cloud`) y se expone vía comandos Tauri; el sidecar Python
> solo atiende el modo local.

Uso:
    from chatbot.motor_chat import router
"""

from .endpoints import router

__all__ = ["router"]