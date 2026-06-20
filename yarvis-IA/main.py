from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import uvicorn
import sys
from modelos.profeta.predictor import run_prediction

# Puerto global: Rust lo pasa como argumento (python main.py 54321)
# Fallback 8000 para desarrollo manual sin sidecar.
PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8000

app = FastAPI(title="Y.A.R.V.I.S. IA Engine")

class PredictionRequest(BaseModel):
    db_path: str
    days: int = 30

@app.get("/")
async def root():
    return {"status": "online", "message": "Y.A.R.V.I.S. AI Brain is running"}

@app.get("/health")
async def health():
    """Health check endpoint que Rust consulta al arrancar."""
    return {"status": "ok", "port": PORT}

@app.post("/predict")
async def predict(request: PredictionRequest):
    result = run_prediction(request.db_path, request.days)
    if "error" in result:
        raise HTTPException(status_code=400, detail=result["error"])
    return result

@app.post("/chat")
async def chat(message: str):
    # Placeholder para Qwen — se implementa en el Paso 4.2
    return {"response": f"Y.A.R.V.I.S. procesando: {message}"}

if __name__ == "__main__":
    print(f"[YARVIS-IA] Motor de IA arrancando en puerto {PORT}...")
    uvicorn.run(app, host="127.0.0.1", port=PORT)
