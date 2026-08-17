from fastapi import FastAPI
import uvicorn
import sys

from chatbot.embeddings.endpoints import router as embeddings_router
from profeta.endpoints import router as predictions_router
from chatbot.motor_chat import router as chat_router

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8000
app = FastAPI(title="Y.A.R.V.I.S. IA Engine")

app.include_router(embeddings_router)
app.include_router(predictions_router)
app.include_router(chat_router)


@app.get("/")
async def root():
    return {"status": "online", "message": "Y.A.R.V.I.S. AI Brain is running"}


@app.get("/health")
async def health():
    """Health check que Rust consulta al arrancar."""
    return {"status": "ok", "port": PORT}


if __name__ == "__main__":
    print(f"[YARVIS-IA] Motor de IA arrancando en puerto {PORT}...")
    print(f"[YARVIS-IA] Endpoints registrados:")
    print(f"[YARVIS-IA]   /health                    → Health check")
    print(f"[YARVIS-IA]   /generar_embedding          → Embeddings (all-MiniLM-L6-v2)")
    print(f"[YARVIS-IA]   /buscar_similar             → Busqueda semantica knowledge_base")
    print(f"[YARVIS-IA]   /insertar_knowledge         → Insertar en knowledge_base")
    print(f"[YARVIS-IA]   /recalcular_predicciones    → Prophet: prediccion de ventas")
    print(f"[YARVIS-IA]   /chat | /chat_stream         → Chat YARVIS (local y nube)")
    print(f"[YARVIS-IA]   /model_status /load_model /unload_model /stop → Modelos Qwen")
    print(f"[YARVIS-IA]   /cloud_models                → Modelos de proveedores de nube")
    print(f"[YARVIS-IA]   /backfill                    → Generar embeddings del catalogo")
    uvicorn.run(app, host="127.0.0.1", port=PORT)
