"""
🌐 endpoints.py — API web de Y.A.R.V.I.S. (FastAPI).

Se encarga de:
    - Definir los endpoints HTTP (/chat, /chat_stream, /model_status, /load_model, /unload_model).
    - Recibir y validar peticiones, devolver respuestas.
    - Orquestar: junta prompts, modelos y caché para responder al usuario.

No contiene lógica de negocio ni consultas SQL.
"""

import json
import threading

from fastapi import APIRouter, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel

from .apis_cloud import generar_completo, generar_stream, listar_modelos, nombre_proveedor
from .cache import cantidad_productos_cache, iniciar_cache
from .gestion_hardware import (
    WORD_LIMITS,
    cargar_modelo,
    descargar_modelo,
    ejecutar_chat,
    estado_modelos,
)
from .prompts import construir_mensajes
from .prompts_api import construir_mensajes_api

router = APIRouter()

# Evento global de cancelación: se setea al llamar /stop y los generadores
# de /chat_stream lo consultan entre tokens para cortar la generación.
_cancel_event = threading.Event()

# Auto-desactivación por inactividad: si no se habla con el chat en
# _INACTIVIDAD_SEGUNDOS, se descargan los modelos Qwen de la RAM.
_INACTIVIDAD_SEGUNDOS = 300  # 5 minutos
_timer_inactividad = None  # threading.Timer activo, o None si no hay


def _descargar_por_inactividad():
    """Descarga los modelos Qwen de la RAM tras 5 min sin actividad."""
    print("[YARVIS-CHAT] 5 min de inactividad: descargando modelos de la RAM.")
    for key in ("0.5B", "0.8B", "1.7B"):
        descargar_modelo(key)
    global _timer_inactividad
    _timer_inactividad = None


def _registrar_actividad():
    """Marca actividad en el chat: reinicia el timer de 5 min.

    Se llama en cada /chat, /chat_stream y /load_model.
    """
    global _timer_inactividad
    if _timer_inactividad is not None:
        _timer_inactividad.cancel()
    _timer_inactividad = threading.Timer(_INACTIVIDAD_SEGUNDOS, _descargar_por_inactividad)
    _timer_inactividad.daemon = True
    _timer_inactividad.start()


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
    provider: str = ""
    api_key: str = ""
    tienda_info: dict = {}


class CloudModelsRequest(BaseModel):
    provider: str
    api_key: str = ""


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


def _separar_think(textos, max_w):
    """Separa los bloques <think> del texto final.

    Recibe texto crudo por trozos y emite ('token'|'think', texto):
    - 'think': razonamiento del modelo (se muestra sombreado).
    - 'token': respuesta final (texto real).
    """
    word_count = 0
    in_think = False
    buffer = ""
    for content in textos:
        if not content:
            continue
        buffer += content
        while buffer:
            if not in_think:
                idx = buffer.find("<think>")
                if idx == -1:
                    word_count += len(buffer.split())
                    if word_count > max_w:
                        return
                    yield "token", buffer
                    buffer = ""
                    break
                pre = buffer[:idx]
                if pre:
                    word_count += len(pre.split())
                    if word_count > max_w:
                        return
                    yield "token", pre
                buffer = buffer[idx + len("<think>"):]
                in_think = True
            else:
                idx = buffer.find("</think>")
                if idx == -1:
                    yield "think", buffer
                    buffer = ""
                    break
                pre = buffer[:idx]
                if pre:
                    yield "think", pre
                buffer = buffer[idx + len("</think>"):]
                in_think = False
    if buffer:
        yield ("think" if in_think else "token"), buffer


# ============================================================
# ENDPOINTS
# ============================================================

@router.get("/model_status")
async def model_status():
    return _estado_completo()


@router.post("/load_model")
async def load_model(request: LoadModelRequest):
    _registrar_actividad()
    model_key = request.model.upper()
    if model_key not in ("0.5B", "0.8B", "1.7B"):
        raise HTTPException(status_code=400, detail=f"Modelo no válido: {request.model}")
    try:
        cargar_modelo(model_key)
        return {"status": "ok", "model": model_key, "message": f"Qwen {model_key} cargado", **_estado_completo()}
    except RuntimeError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error al cargar {model_key}: {e}")


@router.post("/stop")
async def stop():
    """Detiene la generación en curso (local o nube)."""
    _cancel_event.set()
    return {"status": "stopped"}


@router.post("/unload_model")
async def unload_model(request: LoadModelRequest):
    model_key = request.model.upper()
    if model_key not in ("0.5B", "0.8B", "1.7B"):
        raise HTTPException(status_code=400, detail=f"Modelo no válido: {request.model}")
    descargar_modelo(model_key)
    return {"status": "ok", "model": model_key, "message": f"Qwen {model_key} descargado", **_estado_completo()}


@router.post("/cloud_models")
async def cloud_models(request: CloudModelsRequest):
    """Lista los modelos disponibles de un proveedor de nube (dinámico)."""
    try:
        return {"models": listar_modelos(request.provider, request.api_key)}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@router.post("/chat")
async def chat(request: ChatRequest):
    _registrar_actividad()
    if not request.messages:
        raise HTTPException(status_code=400, detail="No hay mensajes")
    ultimo = request.messages[-1].content
    if not ultimo or not ultimo.strip():
        raise HTTPException(status_code=400, detail="Mensaje vacío")

    if request.provider:
        try:
            chat_messages = construir_mensajes_api(request.messages)
            respuesta = generar_completo(request.provider, request.api_key, request.model, chat_messages)
            return {"response": respuesta, "model_used": nombre_proveedor(request.provider)}
        except Exception as e:
            print(f"[YARVIS-CHAT] Error proveedor ({request.provider}): {e}")
            return {"response": f"Error: {e}", "model_used": "none"}

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
    _registrar_actividad()
    if not request.messages:
        raise HTTPException(status_code=400, detail="No hay mensajes")
    ultimo = request.messages[-1].content
    if not ultimo or not ultimo.strip():
        raise HTTPException(status_code=400, detail="Mensaje vacío")

    _cancel_event.clear()

    if request.provider:
        chat_messages = construir_mensajes_api(request.messages)
        cloud_display = nombre_proveedor(request.provider)
        max_w = 1000

        def generate_cloud():
            try:
                usage: dict = {}
                total_chars = sum(len(m.get("content") or "") for m in chat_messages)
                print(f"[YARVIS-CHAT] Cloud: {len(chat_messages)} msgs, {total_chars} chars (~{total_chars // 4} tok est)")
                textos = (token for token, _ in generar_stream(
                    request.provider, request.api_key, request.model, chat_messages,
                    usage=usage,
                ))
                for kind, text in _separar_think(textos, max_w):
                    if _cancel_event.is_set():
                        break
                    yield f"data: {json.dumps({kind: text, 'model': cloud_display})}\n\n"
                if usage:
                    print(f"[YARVIS-CHAT] Usage real del proveedor: {usage.get('prompt_tokens')} prompt + {usage.get('completion_tokens')} completion = {usage.get('total_tokens')} total")
                    yield f"data: {json.dumps({'usage': usage, 'model': cloud_display})}\n\n"
                if not _cancel_event.is_set():
                    yield f"data: {json.dumps({'done': True, 'model': cloud_display})}\n\n"
            except Exception as e:
                print(f"[YARVIS-CHAT] Error proveedor ({cloud_display}): {e}")
                yield f"data: {json.dumps({'error': str(e)})}\n\n"

        return StreamingResponse(generate_cloud(), media_type="text/event-stream")

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
            deltas = (((c.get("choices") or [{}])[0].get("delta") or {}).get("content", "") for c in stream)
            for kind, text in _separar_think(deltas, max_w):
                if _cancel_event.is_set():
                    break
                yield f"data: {json.dumps({kind: text, 'model': model_key})}\n\n"
            if not _cancel_event.is_set():
                yield f"data: {json.dumps({'done': True, 'model': model_key})}\n\n"
        except Exception as e:
            yield f"data: {json.dumps({'error': str(e)})}\n\n"

    return StreamingResponse(generate(), media_type="text/event-stream")


# Se arranca la caché al importar el módulo (carga en segundo plano)
iniciar_cache()
