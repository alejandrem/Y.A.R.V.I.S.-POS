"""
Motor de chat de Y.A.R.V.I.S.
RAG con cache de productos + embeddings all-MiniLM-L6-v2.
Modelos bajo demanda: solo 0.5B siempre listo.
"""

import gc
import json
import os
import sqlite3
import threading
import time
from datetime import datetime, timedelta
from fastapi import APIRouter, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel
from llama_cpp import Llama

from modelos.qwen.rutas import qwen0_5, qwen0_8, qwen1_7

router = APIRouter()

_llm_0_5 = None
_llm_0_8 = None
_llm_1_7 = None

WORD_LIMITS = {"0.5B": 2000, "0.8B": 3000, "1.7B": 4000}

COMPLEX_KEYWORDS = [
    "anomal", "reembolso", "estornad", "comparar", "tendencia",
    "predicc", "analizar", "análisis", "promedio", "estadístic",
    "rentabilidad", "utilidad", "margen", "ganancia", "pérdida",
    "robo", "sospech", "inusual", "raro", "diferente",
]

# ============================================================
# PRODUCT CACHE + EMBEDDINGS
# ============================================================

_inventory_cache: dict = {}
_inventory_embeddings: list[tuple[str, list[float]]] = []
_embedding_model = None
_cache_lock = threading.Lock()
_cache_last_refresh: float = 0
CACHE_REFRESH_INTERVAL = 60  # seconds


def _get_embedding_model():
    global _embedding_model
    if _embedding_model is None:
        print("[YARVIS-CHAT] Cargando modelo de embeddings all-MiniLM-L6-v2...")
        from sentence_transformers import SentenceTransformer
        _embedding_model = SentenceTransformer("all-MiniLM-L6-v2")
        print("[YARVIS-CHAT] Embeddings listos.")
    return _embedding_model


def _find_db_path() -> str:
    candidates = [
        os.path.expanduser("~/.local/share/com.yarvis.pos/yarvis.db"),
        os.path.expanduser("~/.local/share/yarvis-app/yarvis.db"),
        os.path.expanduser("~/.config/yarvis-app/yarvis.db"),
        "yarvis.db",
    ]
    for path in candidates:
        if os.path.exists(path):
            return path
    return ""


def _refresh_inventory_cache():
    """Carga todos los productos y pre-calcula embeddings."""
    global _inventory_cache, _inventory_embeddings, _cache_last_refresh
    db_path = _find_db_path()
    if not db_path:
        print("[YARVIS-CHAT] No se encontró DB para cache.")
        return

    try:
        conn = sqlite3.connect(db_path)
        conn.row_factory = sqlite3.Row
        cur = conn.cursor()
        cur.execute(
            "SELECT nombre, stock, precio_venta, precio_costo, categoria, stock_minimo "
            "FROM productos ORDER BY nombre"
        )
        rows = cur.fetchall()
        conn.close()

        new_cache = {}
        for r in rows:
            new_cache[r["nombre"]] = {
                "stock": r["stock"],
                "precio_venta": r["precio_venta"],
                "precio_costo": r["precio_costo"],
                "categoria": r["categoria"] or "Sin categoría",
                "stock_minimo": r["stock_minimo"] or 0,
            }

        model = _get_embedding_model()
        names = [r["nombre"] for r in rows]
        vectors = model.encode(names, show_progress_bar=False).tolist()
        new_embeddings = list(zip(names, vectors))

        with _cache_lock:
            _inventory_cache = new_cache
            _inventory_embeddings = new_embeddings
            _cache_last_refresh = time.time()

        print(f"[YARVIS-CHAT] Cache actualizado: {len(new_cache)} productos, {len(new_embeddings)} embeddings.")
    except Exception as e:
        print(f"[YARVIS-CHAT] Error refrescando cache: {e}")


def _ensure_cache():
    """Asegura que el cache esté cargado y fresco."""
    global _cache_last_refresh
    if not _inventory_cache or (time.time() - _cache_last_refresh > CACHE_REFRESH_INTERVAL):
        _refresh_inventory_cache()


def _scheduled_refresh():
    """Background timer para refrescar cache."""
    while True:
        time.sleep(CACHE_REFRESH_INTERVAL)
        _refresh_inventory_cache()


def _semantic_search(query: str, top_k: int = 8) -> list[tuple[str, float]]:
    """Busca productos más relevantes por similitud coseno."""
    if not _inventory_embeddings:
        return []

    model = _get_embedding_model()
    q_vec = model.encode(query).tolist()

    import numpy as np
    q_np = np.array(q_vec)
    q_norm = np.linalg.norm(q_np)
    if q_norm == 0:
        return []

    scored = []
    for name, vec in _inventory_embeddings:
        v_np = np.array(vec)
        v_norm = np.linalg.norm(v_np)
        if v_norm == 0:
            continue
        sim = float(np.dot(q_np, v_np) / (q_norm * v_norm))
        scored.append((name, sim))

    scored.sort(key=lambda x: x[1], reverse=True)
    return scored[:top_k]


def _format_producto_compacto(nombre: str, info: dict) -> str:
    """Formato compacto: NOMBRE | stock: X | $precio | categoría"""
    return f"{nombre} | stock: {info['stock']} | ${info['precio_venta']:.2f} | {info['categoria']}"


def _obtener_contexto_inteligente(role: str, pregunta: str) -> str:
    _ensure_cache()
    preg = pregunta.lower()
    partes = []

    # --- Detectar tipo de pregunta ---
    es_producto = any(k in preg for k in [
        "producto", "stock", "artículo", "articulo", "categoria",
        "categoría", "hay", "tengo", "cuántos", "cuantos", "falta",
        "agotad", "surtir", "comprar", "pedido", "inventario",
    ])
    es_venta = any(k in preg for k in [
        "venta", "vendí", "vendi", "gananc", "ingreso", "cobr",
        "dinero", "efectivo", "tarjeta", "transferencia", "ticket",
        "hoy", "ayer", "semana", "mes", "total", "caja", "corte",
    ])
    es_empleado = any(k in preg for k in [
        "empleado", "cajero", "juan", "maría", "maria", "turno",
        "salario", "meta", "reembolso", "cancelación", "cancelacion",
    ])
    es_anomalia = any(k in preg for k in [
        "anomal", "raro", "sospech", "inusual", "robo", "estornad",
        "reembolso", "cancelación", "cancelacion", "fraude",
    ])

    # Detectar si busca stock bajo/cero específicamente
    busca_sin_stock = any(k in preg for k in [
        "sin stock", "agotad", "cero", "no hay", "falta", "disponibilidad",
        "no tienen", "no hay de", "ninguno", "cuáles no",
    ])

    busca_stock_bajo = any(k in preg for k in [
        "por agotarse", "stock bajo", "poco stock", "quedan pocas",
        "casi se agota", "necesito surtir",
    ])

    # Saludos y mensajes cortos no necesitan productos
    es_saludo = len(preg.split()) <= 2 and not any([es_producto, es_venta, es_empleado, es_anomalia])

    # --- Productos por búsqueda semántica ---
    if (es_producto or busca_sin_stock or busca_stock_bajo) and not es_saludo:
        if busca_sin_stock:
            todos_sin_stock = [
                (n, info) for n, info in _inventory_cache.items()
                if info["stock"] <= 0
            ]
            todos_sin_stock.sort(key=lambda x: x[1]["stock"])
            if todos_sin_stock:
                lines = [f"  {n} | stock: {info['stock']}" for n, info in todos_sin_stock[:5]]
                partes.append(f"SIN STOCK ({len(todos_sin_stock)} total):\n" + "\n".join(lines))
        elif busca_stock_bajo:
            # Productos con stock bajo (<=10 unidades)
            bajos = [
                (n, info) for n, info in _inventory_cache.items()
                if 0 < info["stock"] <= 10
            ]
            bajos.sort(key=lambda x: x[1]["stock"])
            if bajos:
                lines = [f"  {n} | stock: {info['stock']}" for n, info in bajos[:8]]
                partes.append(f"STOCK BAJO ({len(bajos)} productos):\n" + "\n".join(lines))
            else:
                partes.append("No hay productos con stock bajo.")
        else:
            hits = _semantic_search(pregunta, top_k=5)
            relevantes = [(n, s) for n, s in hits if s > 0.30]
            if not relevantes:
                relevantes = hits[:3]

            lines = []
            for name, score in relevantes:
                info = _inventory_cache.get(name)
                if info:
                    lines.append(_format_producto_compacto(name, info))

            if lines:
                partes.append("\n".join(lines))

    # --- Ventas ---
    if es_venta:
        db_path = _find_db_path()
        if db_path:
            try:
                conn = sqlite3.connect(db_path)
                conn.row_factory = sqlite3.Row
                cur = conn.cursor()

                hoy = datetime.now().strftime("%Y-%m-%d")
                cur.execute(
                    "SELECT COUNT(*) as tickets, COALESCE(SUM(total), 0) as total "
                    "FROM ventas WHERE DATE(fecha) = ?", (hoy,)
                )
                v = cur.fetchone()
                partes.append(f"VENTAS HOY: {v['tickets']} tickets, ${v['total']:.2f}")

                hace_7 = (datetime.now() - timedelta(days=7)).strftime("%Y-%m-%d")
                cur.execute(
                    "SELECT COUNT(*) as tickets, COALESCE(SUM(total), 0) as total "
                    "FROM ventas WHERE DATE(fecha) >= ?", (hace_7,)
                )
                v7 = cur.fetchone()
                partes.append(f"VENTAS 7 DÍAS: {v7['tickets']} tickets, ${v7['total']:.2f}")

                if "cajero" in preg or "empleado" in preg or "quién" in preg:
                    cur.execute(
                        "SELECT cajero, COUNT(*) as n, SUM(total) as t "
                        "FROM ventas WHERE DATE(fecha) >= ? GROUP BY cajero ORDER BY t DESC",
                        (hace_7,),
                    )
                    for r in cur.fetchall():
                        partes.append(f"  {r['cajero']}: {r['n']} ventas, ${r['t']:.2f}")

                conn.close()
            except Exception as e:
                partes.append(f"Error ventas: {e}")

    # --- Empleados / Anomalías ---
    if es_empleado or es_anomalia:
        db_path = _find_db_path()
        if db_path:
            try:
                conn = sqlite3.connect(db_path)
                conn.row_factory = sqlite3.Row
                cur = conn.cursor()
                hace_7 = (datetime.now() - timedelta(days=7)).strftime("%Y-%m-%d")

                cur.execute(
                    "SELECT cajero, COUNT(*) as n FROM ventas "
                    "WHERE estado = 'cancelada' AND DATE(fecha) >= ? GROUP BY cajero",
                    (hace_7,),
                )
                canc = cur.fetchall()
                if canc:
                    for r in canc:
                        partes.append(f"CANCELACIONES - {r['cajero']}: {r['n']}")

                cur.execute(
                    "SELECT dv.producto_nombre, COUNT(*) as n "
                    "FROM detalle_ventas dv JOIN ventas v ON dv.venta_id = v.id "
                    "WHERE v.estado = 'cancelada' AND DATE(v.fecha) >= ? "
                    "GROUP BY dv.producto_nombre ORDER BY n DESC LIMIT 5",
                    (hace_7,),
                )
                estorn = cur.fetchall()
                if estorn:
                    for r in estorn:
                        partes.append(f"REEMBOLSOS - {r['producto_nombre']}: {r['n']}")

                if role == "admin":
                    cur.execute(
                        "SELECT nombre, turno, salario_semanal, meta_mensual, estado "
                        "FROM usuarios WHERE rol = 'empleado'"
                    )
                    emps = cur.fetchall()
                    for r in emps:
                        partes.append(
                            f"EMPLEADO: {r['nombre']}, turno: {r['turno']}, "
                            f"salario: ${r['salario_semanal']:.0f}/sem, "
                            f"meta: ${r['meta_mensual']:.0f}/mes, estado: {r['estado']}"
                        )

                conn.close()
            except Exception as e:
                partes.append(f"Error empleados: {e}")

    return "\n".join(partes) if partes else "Sin datos disponibles."


def _build_system_prompt(contexto_db: str, tienda_info: dict) -> str:
    nombre = tienda_info.get("nombre", "la tienda")
    ubic = tienda_info.get("ubicacion", "")
    return f"""Eres Y.A.R.V.I.S., asistente de "{nombre}"{f' en {ubic}' if ubic else ''}.

Responde usando SOLO los datos de abajo. Sé breve (2-4 oraciones). Usa markdown.

Ejemplo:
Pregunta: ¿qué shampoo hay?
Datos: SHAMPOO H S 200ML | stock: 720 | $55
Respuesta: **SHAMPOO H S 200ML** — Stock: 720 unidades — $55

DATOS:
{contexto_db}"""


# ============================================================
# MODEL MANAGEMENT
# ============================================================

def _get_system_ram_gb() -> float:
    try:
        with open("/proc/meminfo") as f:
            for line in f:
                if "MemTotal" in line:
                    kb = int(line.split()[1])
                    return kb / 1048576
    except Exception:
        pass
    return 8.0


def _can_load_model(model_key: str) -> tuple[bool, str]:
    ram_gb = _get_system_ram_gb()
    requirements = {"0.5B": 1.0, "0.8B": 1.5, "1.7B": 3.5}
    needed = requirements.get(model_key, 2.0)
    if ram_gb < needed:
        return False, f"RAM insuficiente: {ram_gb:.1f}GB disponibles, {model_key} necesita ~{needed}GB"
    if model_key == "1.7B" and ram_gb < 4.0:
        return False, f"RAM insuficiente para 1.7B: {ram_gb:.1f}GB (mínimo 4GB)"
    return True, f"OK ({ram_gb:.1f}GB disponibles)"


def _cargar_modelo(model_key: str) -> Llama:
    global _llm_0_5, _llm_0_8, _llm_1_7
    ok, msg = _can_load_model(model_key)
    if not ok:
        raise RuntimeError(msg)

    if model_key == "0.5B" and _llm_0_5 is not None:
        return _llm_0_5
    if model_key == "0.8B" and _llm_0_8 is not None:
        return _llm_0_8
    if model_key == "1.7B" and _llm_1_7 is not None:
        return _llm_1_7

    loaders = {
        "0.5B": (lambda: Llama(model_path=qwen0_5, n_ctx=4096, n_gpu_layers=-1, n_threads=4, verbose=False), "_llm_0_5"),
        "0.8B": (lambda: Llama(model_path=qwen0_8, n_ctx=4096, n_gpu_layers=-1, n_threads=4, verbose=False), "_llm_0_8"),
        "1.7B": (lambda: Llama(model_path=qwen1_7, n_ctx=4096, n_gpu_layers=-1, n_threads=4, verbose=False), "_llm_1_7"),
    }
    loader_fn, attr_name = loaders[model_key]
    print(f"[YARVIS-CHAT] Cargando Qwen {model_key}...")
    model = loader_fn()
    globals()[attr_name] = model
    print(f"[YARVIS-CHAT] Qwen {model_key} listo.")
    return model


def _descargar_modelo(model_key: str):
    global _llm_0_5, _llm_0_8, _llm_1_7
    if model_key == "0.5B" and _llm_0_5 is not None:
        del _llm_0_5
        _llm_0_5 = None
    elif model_key == "0.8B" and _llm_0_8 is not None:
        del _llm_0_8
        _llm_0_8 = None
    elif model_key == "1.7B" and _llm_1_7 is not None:
        del _llm_1_7
        _llm_1_7 = None
    gc.collect()
    print(f"[YARVIS-CHAT] Qwen {model_key} descargado.")


def _get_model_status() -> dict:
    ram_gb = _get_system_ram_gb()
    return {
        "ram_gb": round(ram_gb, 1),
        "models": {
            "0.5B": _llm_0_5 is not None,
            "0.8B": _llm_0_8 is not None,
            "1.7B": _llm_1_7 is not None,
        },
        "can_load_1_7b": ram_gb >= 4.0,
        "cache_products": len(_inventory_cache),
    }


def _es_pregunta_compleja(texto: str) -> bool:
    return any(kw in texto.lower() for kw in COMPLEX_KEYWORDS)


def _ejecutar_chat(model: Llama, messages: list, max_words: int) -> str:
    respuesta = model.create_chat_completion(
        messages=messages,
        temperature=0.1,
        max_tokens=max_words * 4,
        top_p=0.7,
    )
    contenido = respuesta["choices"][0]["message"]["content"]
    words = contenido.split()
    if len(words) > max_words:
        truncated = " ".join(words[:max_words])
        last_period = truncated.rfind(".")
        last_newline = truncated.rfind("\n")
        cut_at = max(last_period, last_newline)
        if cut_at > len(truncated) * 0.4:
            truncated = truncated[:cut_at + 1]
        contenido = truncated
    return contenido


def _build_chat_messages(role: str, messages: list, ultimo: str):
    contexto_db = _obtener_contexto_inteligente(role, ultimo)
    tienda = {"nombre": "la tienda"}
    system_prompt = _build_system_prompt(contexto_db, tienda)
    chat_messages = [{"role": "system", "content": system_prompt}]
    for m in messages:
        chat_messages.append({
            "role": m.role if hasattr(m, "role") else m["role"],
            "content": m.content if hasattr(m, "content") else m["content"],
        })
    return chat_messages


# ============================================================
# PYDANTIC MODELS
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
# STARTUP: Cargar cache + embeddings
# ============================================================

def _startup_cache():
    """Carga inicial del cache en background."""
    def _init():
        _refresh_inventory_cache()
        # Iniciar refresh timer
        t = threading.Thread(target=_scheduled_refresh, daemon=True)
        t.start()
    threading.Thread(target=_init, daemon=True).start()


# Se ejecuta al importar el módulo
_startup_cache()


# ============================================================
# ENDPOINTS
# ============================================================

@router.get("/model_status")
async def model_status():
    return _get_model_status()


@router.post("/load_model")
async def load_model(request: LoadModelRequest):
    model_key = request.model.upper().replace("B", "B")
    if model_key not in ("0.5B", "0.8B", "1.7B"):
        raise HTTPException(status_code=400, detail=f"Modelo no válido: {request.model}")
    try:
        _cargar_modelo(model_key)
        return {"status": "ok", "model": model_key, "message": f"Qwen {model_key} cargado", **_get_model_status()}
    except RuntimeError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error al cargar {model_key}: {e}")


@router.post("/unload_model")
async def unload_model(request: LoadModelRequest):
    model_key = request.model.upper().replace("B", "B")
    if model_key not in ("0.5B", "0.8B", "1.7B"):
        raise HTTPException(status_code=400, detail=f"Modelo no válido: {request.model}")
    _descargar_modelo(model_key)
    return {"status": "ok", "model": model_key, "message": f"Qwen {model_key} descargado", **_get_model_status()}


@router.post("/chat")
async def chat(request: ChatRequest):
    if not request.messages:
        raise HTTPException(status_code=400, detail="No hay mensajes")
    ultimo = request.messages[-1].content
    if not ultimo or not ultimo.strip():
        raise HTTPException(status_code=400, detail="Mensaje vacío")

    chat_messages = _build_chat_messages(request.role, request.messages, ultimo)
    selected = request.model.lower()

    if selected in ("0.5b", "0.8b", "1.7b"):
        mk = selected.replace("b", "B")
        try:
            llm = _cargar_modelo(mk)
            return {"response": _ejecutar_chat(llm, chat_messages, WORD_LIMITS[mk]), "model_used": mk}
        except RuntimeError as e:
            return {"response": str(e), "model_used": "none"}
        except Exception as e:
            return {"response": f"Error: {e}", "model_used": "none"}

    try:
        llm = _cargar_modelo("0.5B")
        respuesta = _ejecutar_chat(llm, chat_messages, WORD_LIMITS["0.5B"])
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

    chat_messages = _build_chat_messages(request.role, request.messages, ultimo)
    selected = request.model.lower()

    llm = None
    model_key = "0.5B"

    if selected in ("0.5b", "0.8b", "1.7b"):
        mk = selected.replace("b", "B")
        try:
            llm = _cargar_modelo(mk)
            model_key = mk
        except RuntimeError as e:
            raise HTTPException(status_code=400, detail=str(e))
        except Exception as e:
            raise HTTPException(status_code=500, detail=f"Error cargando {mk}: {e}")
    else:
        try:
            llm = _cargar_modelo("0.5B")
            model_key = "0.5B"
        except Exception as e:
            raise HTTPException(status_code=500, detail=f"Error: {e}")

    max_w = WORD_LIMITS.get(model_key, 2000)

    def generate():
        try:
            stream = llm.create_chat_completion(
                messages=chat_messages,
                temperature=0.1,
                max_tokens=max_w * 4,
                top_p=0.7,
                stream=True,
            )
            word_count = 0
            for chunk in stream:
                delta = chunk["choices"][0].get("delta", {})
                content = delta.get("content", "")
                if content:
                    word_count += len(content.split())
                    if word_count > max_w:
                        break
                    yield f"data: {json.dumps({'token': content, 'model': model_key})}\n\n"
            yield f"data: {json.dumps({'done': True, 'model': model_key})}\n\n"
        except Exception as e:
            yield f"data: {json.dumps({'error': str(e)})}\n\n"

    return StreamingResponse(generate(), media_type="text/event-stream")
