"""
chatbot/motor_chat — Paquete del motor de chat de Y.A.R.V.I.S.

Divide el antiguo motor_chat.py (666 líneas) en módulos por responsabilidad:

    consultas_db.py      → 🗄️ base de datos (SQLite)
    gestion_hardware.py  → 🧠 RAM + ciclo de vida de los modelos Qwen
    motor_rag.py         → 🔍 embeddings + búsqueda semántica
    prompts.py           → 📝 system prompts + limpieza <think>
    cache.py             → ⏱️ caché de inventario + hilos de fondo
    endpoints.py         → 🌐 endpoints HTTP (FastAPI)

Uso:
    from chatbot.motor_chat import router
"""

from .endpoints import router

__all__ = ["router"]
