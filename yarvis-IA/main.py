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
    print(f"[YARVIS-IA] Endpoints registrados:")
    print(f"[YARVIS-IA]   /health                    → Health check")
    print(f"[YARVIS-IA]   /generar_embedding          → Embeddings (all-MiniLM-L6-v2)")
    print(f"[YARVIS-IA]   /buscar_similar             → Busqueda semantica knowledge_base")
    print(f"[YARVIS-IA]   /insertar_knowledge         → Insertar en knowledge_base")
    print(f"[YARVIS-IA]   /analizar_ticket            → LLM Qwen (extrae fecha + items)")
    print(f"[YARVIS-IA]   /parsear_con_mapeo          → Parser manual con mapeo de columnas")
    print(f"[YARVIS-IA]   /parsear_carpeta            → Parsear carpeta de tickets")
    print(f"[YARVIS-IA]   /parsear_carpeta_stream     → Parsear carpeta (stream SSE)")
    print(f"[YARVIS-IA]   /parsear_excel              → Parsear catalogo Excel")
    print(f"[YARVIS-IA]   /parsear_catalogo_visual    → Parsear catalogo TXT visual")
    print(f"[YARVIS-IA]   /parsear_catalogo_csv       → Parsear catalogo CSV")
    print(f"[YARVIS-IA]   /recalcular_predicciones    → Prophet: prediccion de ventas")
    print(f"[YARVIS-IA]   /chat                       → Chat YARVIS (placeholder)")
    print(f"[YARVIS-IA]   /load_llm /unload_llm       → Carga/descarga modelos Qwen")
    print(f"[YARVIS-IA]   /vincular_inventario        → Vincular productos con inventario")
    print(f"[YARVIS-IA]   /guardar_vinculacion        → Guardar vinculacion producto-DB")
    uvicorn.run(app, host="127.0.0.1", port=PORT)
