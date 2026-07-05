from fastapi import APIRouter
from pydantic import BaseModel

from modelos.qwen.parser_llm import descargar_modelos

router = APIRouter()


@router.post("/chat")
async def chat(message: str):
    """Chatbot placeholder para Qwen (Paso 4.2)."""
    return {"response": f"Y.A.R.V.I.S. procesando: {message}"}


class LoadLLMRequest(BaseModel):
    model: str = "0.5B_Q6"


@router.post("/load_llm")
async def load_llm(request: LoadLLMRequest):
    """Carga el modelo LLM en RAM/VRAM."""
    return {"status": "ok", "model": request.model, "message": "LLM loaded (placeholder)"}


@router.post("/unload_llm")
async def unload_llm():
    """Descarga todos los modelos Qwen de VRAM para liberar memoria."""
    count = descargar_modelos()
    return {"status": "ok", "unloaded": count, "message": f"{count} modelo(s) descargado(s)"}
