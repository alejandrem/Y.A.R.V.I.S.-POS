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
import time

from fastapi import APIRouter, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel

from .modelos_API.apis_cloud import generar_completo, generar_stream, listar_modelos, nombre_proveedor
from .modelos_API.prompts_api import construir_mensajes_api
from .modelos_local.cache import cantidad_productos_cache, iniciar_cache
from .modelos_local.gestion_hardware import (
    cargar_modelo,
    descargar_modelo,
    ejecutar_chat,
    estado_modelos,
)
from .modelos_local.variables import MODELOS, WORD_LIMITS
from .modelos_local.herramientas import TOOLS_SCHEMA, ejecutar_tool
from .modelos_local.prompts import _separar_think, construir_mensajes

# Versión en minúsculas de las claves (el usuario manda "0.5b" / "0.8b" / "1.7b").
_MODELOS_B = tuple(m.lower() for m in MODELOS)

router = APIRouter()

# ------------------------------------------------------------
# Concurrencia aislada por stream (C1) y descarga segura (C2):
#
# C1 — Cada /chat_stream recibe su PROPIO threading.Event. El registro
#      {stream_id -> Event} permite que /stop cancele SOLO la generación
#      más reciente (o la que traiga stream_id) sin afectar a las demás.
# C2 — La descarga por inactividad espera un contador de "modelos en uso":
#      si hay un stream o una llamada iterando los Qwen, NUNCA descarga.
# ------------------------------------------------------------

_INACTIVIDAD_SEGUNDOS = 300  # 5 minutos
_INACTIVIDAD_CHECK_SEGUNDOS = 15  # frecuencia del hilo vigilante (s)

# Registro de streams bajo un lock (diccionario con orden de inserción).
_streams_lock = threading.Lock()
_registry = {
    "next_id": 0,     # último id asignado a un stream
    "streams": {},    # stream_id -> threading.Event (cancelación por-stream)
    "activos": 0,     # nº de llamadas/streams usando los modelos AHORA
}

_actividad_lock = threading.Lock()
_ultima_actividad = time.monotonic()


def _nuevo_stream_event() -> tuple[int, threading.Event]:
    """Crea y registra un evento de cancelación por-stream; devuelve (id, event)."""
    with _streams_lock:
        _registry["next_id"] += 1
        event = threading.Event()
        _registry["streams"][_registry["next_id"]] = event
        return _registry["next_id"], event


def _marcar_uso_ia():
    """Marca el motor de IA como 'en uso' (el vigilante no debe descargar)."""
    with _streams_lock:
        _registry["activos"] += 1


def _liberar_uso_ia():
    """Marca fin de uso del motor de IA."""
    with _streams_lock:
        if _registry["activos"] > 0:
            _registry["activos"] -= 1


def _terminar_stream(stream_id: int):
    """Da de baja un stream y reinicia la ventana de inactividad.

    Se ejecuta SIEMPRE en el finally del generador (fin, cancelación o error),
    garantizando que el contador de 'modelos en uso' nunca quede desincronizado.
    """
    with _streams_lock:
        _registry["streams"].pop(stream_id, None)
        if _registry["activos"] > 0:
            _registry["activos"] -= 1
        global _ultima_actividad
        _ultima_actividad = time.monotonic()


def _cancelar_stream(stream_id: int | None = None) -> dict:
    """Cancela un stream: el indicado por stream_id o, si no, el más reciente."""
    with _streams_lock:
        if stream_id is not None:
            event = _registry["streams"].get(stream_id)
        elif _registry["streams"]:
            stream_id, event = next(reversed(_registry["streams"].items()))
        else:
            event = None
    if event is None:
        return {"cancelled": False, "stream_id": None}
    event.set()
    return {"cancelled": True, "stream_id": stream_id}


def _registrar_actividad():
    """Marca actividad en el chat: retrasa la descarga por inactividad.

    Se llama en cada /chat, /chat_stream y /load_model.
    """
    global _ultima_actividad
    with _actividad_lock:
        _ultima_actividad = time.monotonic()


def _descargar_por_inactividad():
    """Descarga los modelos Qwen de la RAM tras 5 min sin actividad."""
    print("[YARVIS-CHAT] 5 min de inactividad: descargando modelos de la RAM.")
    for key in MODELOS:
        descargar_modelo(key)


def _vigilante_inactividad():
    """Hilo daemon: descarga SOLO si no hay streams ni llamadas en uso (C2).

    Reemplaza al threading.Timer global (que reiniciaba sin lock y podía
    descargar un Llama en plena generación de un /chat_stream).
    """
    global _ultima_actividad
    while True:
        time.sleep(_INACTIVIDAD_CHECK_SEGUNDOS)
        with _actividad_lock:
            inactividad = time.monotonic() - _ultima_actividad
        with _streams_lock:
            activos = _registry["activos"]
        if inactividad < _INACTIVIDAD_SEGUNDOS or activos > 0:
            continue
        _descargar_por_inactividad()
        with _actividad_lock:
            _ultima_actividad = time.monotonic()


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


def _fallback_local(messages: list, role: str, ultimo: str, err: str = "") -> str:
    """Respuesta local de emergencia si la nube falla (switch automático).

    Réplica exacta del modo local: Qwen 0.5B con RAG de inventario vía
    construir_mensajes (que ya inyecta buscar_semantico en el contexto).
    """
    try:
        chat_messages = construir_mensajes(role, messages, ultimo)
        llm = cargar_modelo("0.5B")
        return ejecutar_chat(llm, chat_messages, WORD_LIMITS["0.5B"])
    except Exception as e:
        return f"Error nube: {err}\nError local: {e}"


def _stream_fallback_local(messages: list, role: str, ultimo: str) -> str:
    """SSE del fallback local: una ventana 'token' con la respuesta Qwen 0.5B.

    Se inyecta como único evento del stream cuando la nube falla.
    """
    try:
        texto = _fallback_local(messages, role, ultimo)
        datos = json.dumps({"token": texto, "model": "local-fallback"})
    except Exception:
        datos = json.dumps({"error": "Fallback local también falló."})
    return f"data: {datos}\n\n"


@router.get("/model_status")
async def model_status():
    return _estado_completo()


@router.post("/load_model")
async def load_model(request: LoadModelRequest):
    _registrar_actividad()
    model_key = request.model.upper()
    if model_key not in MODELOS:
        raise HTTPException(status_code=400, detail=f"Modelo no válido: {request.model}")
    try:
        cargar_modelo(model_key)
        return {"status": "ok", "model": model_key, "message": f"Qwen {model_key} cargado", **_estado_completo()}
    except RuntimeError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error al cargar {model_key}: {e}")


@router.post("/stop")
async def stop(stream_id: int | None = None):
    """Detiene la generación en curso (local o nube).

    Cada stream tiene su propio evento (C1): sin stream_id se cancela la
    generación más reciente; con stream_id se cancela exactamente esa.
    Nunca afecta a otros streams en curso.
    """
    return {"status": "stopped", **_cancelar_stream(stream_id)}


@router.post("/unload_model")
async def unload_model(request: LoadModelRequest):
    model_key = request.model.upper()
    if model_key not in MODELOS:
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

    _marcar_uso_ia()
    try:
        if request.provider:
            avisos: list[str] = []
            try:
                chat_messages = construir_mensajes_api(request.messages)
                respuesta = generar_completo(
                    request.provider,
                    request.api_key,
                    request.model,
                    chat_messages,
                    tools=TOOLS_SCHEMA,
                    ejecutar_tool=ejecutar_tool,
                    avisos=avisos,
                )
                return {"response": respuesta, "model_used": nombre_proveedor(request.provider), "avisos": avisos}
            except Exception as e:
                print(f"[YARVIS-CHAT] Error proveedor ({request.provider}): {e}")
                return {
                    "response": _fallback_local(request.messages, request.role, ultimo, str(e)),
                    "model_used": "local-fallback",
                    "avisos": avisos,
                }

        chat_messages = construir_mensajes(request.role, request.messages, ultimo)
        selected = request.model.lower()

        if selected in _MODELOS_B:
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
    finally:
        _liberar_uso_ia()


@router.post("/chat_stream")
async def chat_stream(request: ChatRequest):
    _registrar_actividad()
    if not request.messages:
        raise HTTPException(status_code=400, detail="No hay mensajes")
    ultimo = request.messages[-1].content
    if not ultimo or not ultimo.strip():
        raise HTTPException(status_code=400, detail="Mensaje vacío")

    stream_id, cancel_event = _nuevo_stream_event()
    _marcar_uso_ia()

    if request.provider:
        chat_messages = construir_mensajes_api(request.messages)
        cloud_display = nombre_proveedor(request.provider)
        max_w = 1000

        def generate_cloud():
            try:
                usage: dict = {}
                total_chars = sum(len(m.get("content") or "") for m in chat_messages)
                print(f"[YARVIS-CHAT] Cloud: {len(chat_messages)} msgs, {total_chars} chars (~{total_chars // 4} tok est)")
                modelo_actual = request.model

                def _con_modelo():
                    nonlocal modelo_actual
                    for token, modelo in generar_stream(
                        request.provider,
                        request.api_key,
                        request.model,
                        chat_messages,
                        usage=usage,
                        tools=TOOLS_SCHEMA,
                        ejecutar_tool=ejecutar_tool,
                    ):
                        modelo_actual = modelo
                        yield token

                for kind, text in _separar_think(_con_modelo(), max_w):
                    if cancel_event.is_set():
                        break
                    yield f"data: {json.dumps({kind: text, 'model': modelo_actual})}\n\n"
                if usage:
                    print(f"[YARVIS-CHAT] Usage real del proveedor: {usage.get('prompt_tokens')} prompt + {usage.get('completion_tokens')} completion = {usage.get('total_tokens')} total")
                    yield f"data: {json.dumps({'usage': usage, 'model': modelo_actual})}\n\n"
                yield f"data: {json.dumps({'done': True, 'cancelled': cancel_event.is_set(), 'model': modelo_actual})}\n\n"
            except Exception as e:
                print(f"[YARVIS-CHAT] Error proveedor ({cloud_display}), fallback a local: {e}")
                yield _stream_fallback_local(request.messages, request.role, ultimo)
                yield f"data: {json.dumps({'done': True, 'cancelled': False, 'model': 'local-fallback'})}\n\n"
            finally:
                _terminar_stream(stream_id)

        return StreamingResponse(generate_cloud(), media_type="text/event-stream")

    chat_messages = construir_mensajes(request.role, request.messages, ultimo)
    selected = request.model.lower()

    llm = None
    model_key = "0.5B"

    try:
        if selected in _MODELOS_B:
            mk = selected.replace("b", "B")
            llm = cargar_modelo(mk)
            model_key = mk
        else:
            llm = cargar_modelo("0.5B")
            model_key = "0.5B"
    except RuntimeError as e:
        _terminar_stream(stream_id)
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        _terminar_stream(stream_id)
        raise HTTPException(status_code=500, detail=f"Error cargando {model_key}: {e}")

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
                if cancel_event.is_set():
                    break
                yield f"data: {json.dumps({kind: text, 'model': model_key})}\n\n"
            yield f"data: {json.dumps({'done': True, 'cancelled': cancel_event.is_set(), 'model': model_key})}\n\n"
        except Exception as e:
            yield f"data: {json.dumps({'error': str(e)})}\n\n"
            yield f"data: {json.dumps({'done': True, 'cancelled': False, 'model': model_key})}\n\n"
        finally:
            _terminar_stream(stream_id)

    return StreamingResponse(generate(), media_type="text/event-stream")


# Se arrancan la caché y el vigilante de inactividad al importar el módulo.
iniciar_cache()

_vigilante_inactividad_thread = threading.Thread(
    target=_vigilante_inactividad,
    name="yarvis-vigilante-inactividad",
    daemon=True,
)
_vigilante_inactividad_thread.start()
