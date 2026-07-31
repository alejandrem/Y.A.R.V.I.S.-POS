"""
🌐 endpoints.py — API web de Y.A.R.V.I.S. (FastAPI).

Se encarga de:
    - Definir los endpoints HTTP (/chat, /chat_stream, /model_status, /load_model, /unload_model).
    - Recibir y validar peticiones, devolver respuestas.
    - Orquestar: junta prompts, modelos y caché para responder al usuario.

No contiene lógica de negocio ni consultas SQL.
"""

import json

from fastapi import APIRouter, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel

from .cache import cantidad_productos_cache, iniciar_cache
from .gestion_hardware import (
    WORD_LIMITS,
    cargar_modelo,
    descargar_modelo,
    ejecutar_chat,
    estado_modelos,
)
from .prompts import construir_mensajes

router = APIRouter()


# ============================================================
# MODELOS PYDANTIC
# ============================================================

class ChatMessage(BaseModel):
    role: str
    content: str


class ChatRequest(BaseModel):
    messages: list[ChatMessage]
    role: str
    model: str = "auto"
    tienda_info: dict = {}


class LoadModelRequest(BaseModel):
    model: str


# ============================================================
# HELPERS
# ============================================================

def _estado_completo() -> dict:
    """Estado del modelo + productos en caché, para /model_status y afines."""
    estado = estado_modelos()
    estado["cache_products"] = cantidad_productos_cache()
    return estado


# ============================================================
# ENDPOINTS
# ============================================================

@router.get("/model_status")
async def model_status():
    return _estado_completo()


@router.post("/load_model")
async def load_model(request: LoadModelRequest):
    model_key = request.model.upper().replace("B", "B")
    if model_key not in ("0.5B", "0.8B", "1.7B"):
        raise HTTPException(status_code=400, detail=f"Modelo no válido: {request.model}")
    try:
        cargar_modelo(model_key)
        return {"status": "ok", "model": model_key, "message": f"Qwen {model_key} cargado", **_estado_completo()}
    except RuntimeError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error al cargar {model_key}: {e}")


@router.post("/unload_model")
async def unload_model(request: LoadModelRequest):
    model_key = request.model.upper().replace("B", "B")
    if model_key not in ("0.5B", "0.8B", "1.7B"):
        raise HTTPException(status_code=400, detail=f"Modelo no válido: {request.model}")
    descargar_modelo(model_key)
    return {"status": "ok", "model": model_key, "message": f"Qwen {model_key} descargado", **_estado_completo()}


@router.post("/chat")
async def chat(request: ChatRequest):
    if not request.messages:
        raise HTTPException(status_code=400, detail="No hay mensajes")
    ultimo = request.messages[-1].content
    if not ultimo or not ultimo.strip():
        raise HTTPException(status_code=400, detail="Mensaje vacío")

    chat_messages = construir_mensajes(request.role, request.messages, ultimo)
    selected = request.model.lower()

    if selected in ("0.5b", "0.8b", "1.7b"):
        mk = selected.replace("b", "B")
        try:
            llm = cargar_modelo(mk)
            return {"response": ejecutar_chat(llm, chat_messages, WORD_LIMITS[mk]), "model_used": mk}
        except RuntimeError as e:
            return {"response": str(e), "model_used": "none"}
        except Exception as e:
            return {"response": f"Error: {e}", "model_used": "none"}

    try:
        llm = cargar_modelo("0.5B")
        respuesta = ejecutar_chat(llm, chat_messages, WORD_LIMITS["0.5B"])
        return {"response": respuesta, "model_used": "0.5B"}
    except Exception as e:
        return {"response": f"Error: {e}", "model_used": "none"}


@router.post("/chat_stream")
async def chat_stream(request: ChatRequest):
    if not request.messages:
        raise HTTPException(status_code=400, detail="No hay mensajes")
    ultimo = request.messages[-1].content
    if not ultimo or not ultimo.strip():
        raise HTTPException(status_code=400, detail="Mensaje vacío")

    chat_messages = construir_mensajes(request.role, request.messages, ultimo)
    selected = request.model.lower()

    llm = None
    model_key = "0.5B"

    if selected in ("0.5b", "0.8b", "1.7b"):
        mk = selected.replace("b", "B")
        try:
            llm = cargar_modelo(mk)
            model_key = mk
        except RuntimeError as e:
            raise HTTPException(status_code=400, detail=str(e))
        except Exception as e:
            raise HTTPException(status_code=500, detail=f"Error cargando {mk}: {e}")
    else:
        try:
            llm = cargar_modelo("0.5B")
            model_key = "0.5B"
        except Exception as e:
            raise HTTPException(status_code=500, detail=f"Error: {e}")

    max_w = WORD_LIMITS.get(model_key, 2000)

    def generate():
        try:
            stream = llm.create_chat_completion(
                messages=chat_messages,
                temperature=0.6,
                max_tokens=max_w * 4,
                top_p=0.85,
                stream=True,
            )
            word_count = 0
            in_think = False
            buffer = ""
            for chunk in stream:
                delta = chunk["choices"][0].get("delta", {})
                content = delta.get("content", "")
                if content:
                    buffer += content
                    if "<think>" in buffer:
                        in_think = True
                        buffer = buffer.split("<think>")[0]
                        content = buffer
                        buffer = ""
                    elif "</think>" in buffer and in_think:
                        in_think = False
                        buffer = buffer.split("</think>", 1)[1]
                        content = buffer
                        buffer = ""
                    elif in_think:
                        continue
                    else:
                        content = buffer
                        buffer = ""
                    if content:
                        word_count += len(content.split())
                        if word_count > max_w:
                            break
                        yield f"data: {json.dumps({'token': content, 'model': model_key})}\n\n"
            yield f"data: {json.dumps({'done': True, 'model': model_key})}\n\n"
        except Exception as e:
            yield f"data: {json.dumps({'error': str(e)})}\n\n"

    return StreamingResponse(generate(), media_type="text/event-stream")


# Se arranca la caché al importar el módulo (carga en segundo plano)
iniciar_cache()
