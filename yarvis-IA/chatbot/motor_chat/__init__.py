"""
chatbot/motor_chat — Paquete del motor de chat de Y.A.R.V.I.S.

Divide el antiguo motor_chat.py (666 líneas) en módulos por responsabilidad,
agrupados en dos subcarpetas según el tipo de modelo:

    modelos_local/  → 🧠 Modelos LOCALES (Qwen + RAG)
        consultas_db.py      → 🗄️ base de datos (SQLite)
        gestion_hardware.py  → 🧠 RAM + ciclo de vida de los modelos Qwen
        motor_rag.py         → 🔍 embeddings + búsqueda semántica (RAG)
        prompts.py           → 📝 system prompts + contexto de la tienda
        cache.py             → ⏱️ caché de inventario + hilos de fondo

    modelos_API/     → ☁️ Modelos de API / nube (sin RAG)
        apis_cloud.py        → ☁️ proveedores de nube (Gemini, OpenCode Zen)
        prompts_api.py       → 📝 prompt mínimo para modelos de nube

    endpoints.py     → 🌐 endpoints HTTP (FastAPI), orquesta local + nube

El RAG (búsqueda semántica con sqlite-vec) SOLO se usa con los modelos
locales; los proveedores de nube reciben un prompt mínimo sin contexto.

Uso:
    from chatbot.motor_chat import router
"""

from .endpoints import router

__all__ = ["router"]