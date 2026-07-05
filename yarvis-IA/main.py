from fastapi import FastAPI
import uvicorn
import sys

from endpoints.embeddings import router as embeddings_router
from endpoints.predictions import router as predictions_router
from endpoints.parser import router as parser_router
from endpoints.carpeta import router as carpeta_router
from endpoints.chat import router as chat_router
from endpoints.matching import router as matching_router

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8000
app = FastAPI(title="Y.A.R.V.I.S. IA Engine")

app.include_router(embeddings_router)
app.include_router(predictions_router)
app.include_router(parser_router)
app.include_router(carpeta_router)
app.include_router(chat_router)
app.include_router(matching_router)


@app.get("/")
async def root():
    return {"status": "online", "message": "Y.A.R.V.I.S. AI Brain is running"}


@app.get("/health")
async def health():
    """Health check que Rust consulta al arrancar."""
    return {"status": "ok", "port": PORT}


if __name__ == "__main__":
    print(f"[YARVIS-IA] Motor de IA arrancando en puerto {PORT}...")
    print(f"[YARVIS-IA] Endpoints: /health, /generar_embedding, /buscar_similar, /recalcular_predicciones, /analizar_ticket, /parsear_con_mapeo, /parsear_carpeta, /parsear_carpeta_stream, /parsear_excel, /parsear_catalogo_visual, /parsear_catalogo_csv, /chat, /load_llm, /vincular_inventario, /guardar_vinculacion")
    uvicorn.run(app, host="127.0.0.1", port=PORT)
