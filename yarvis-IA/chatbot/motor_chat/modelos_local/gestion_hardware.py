"""
🧠 gestion_hardware.py — Gestión de hardware y modelos Qwen.

Se encarga de:
    - Verificar la RAM disponible y cargar/descargar los modelos Qwen (0.5B/0.8B/1.7B)
      según los umbrales de RAM_REQUERIDA (modelos en Q4: 0.5B → 0.0GB, 0.8B → 0.5GB, 1.7B → 1.3GB).
    - Cargar, descargar y consultar el estado de los modelos.
    - Ejecutar la inferencia del modelo (respuestas completas).

No sabe nada de la base de datos ni de la API web.
"""

import gc
import threading

from llama_cpp import Llama
from parseador_de_tickets.llm.rutas_modelos import qwen0_5, qwen0_8, qwen1_7

from .prompts import limpiar_think
from .variables import RAM_REQUERIDA, WORD_LIMITS

_llm_0_5 = None
_llm_0_8 = None
_llm_1_7 = None

# Un solo lock para toda la carga/descarga de modelos Qwen: evita que dos
# requests concurrentes carguen el mismo modelo dos veces (doble RAM) o que
# se descargue mientras otro hilo está cargando.
_carga_lock = threading.Lock()

_LOADERS = {
    "0.5B": (lambda: Llama(model_path=qwen0_5, n_ctx=4096, n_gpu_layers=-1, n_threads=4, verbose=False), "_llm_0_5"),
    "0.8B": (lambda: Llama(model_path=qwen0_8, n_ctx=4096, n_gpu_layers=-1, n_threads=4, verbose=False), "_llm_0_8"),
    "1.7B": (lambda: Llama(model_path=qwen1_7, n_ctx=4096, n_gpu_layers=-1, n_threads=4, verbose=False), "_llm_1_7"),
}


def get_ram_gb() -> float:
    """RAM total del sistema en GB (solo Linux; fallback 8 GB)."""
    try:
        with open("/proc/meminfo") as f:
            for line in f:
                if "MemTotal" in line:
                    kb = int(line.split()[1])
                    return kb / 1048576
    except Exception:
        pass
    return 8.0


def puede_cargar_modelo(model_key: str) -> tuple[bool, str]:
    """Verifica si la RAM disponible alcanza para cargar el modelo."""
    ram_gb = get_ram_gb()
    needed = RAM_REQUERIDA.get(model_key, 2.0)
    if ram_gb < needed:
        return False, f"RAM insuficiente: {ram_gb:.1f}GB disponibles, {model_key} necesita ≥{needed}GB"
    return True, f"OK ({ram_gb:.1f}GB disponibles)"


def cargar_modelo(model_key: str) -> Llama:
    """Carga un modelo Qwen (0.5B/0.8B/1.7B) o devuelve el ya cargado.

    Usa double-checked locking sobre `_carga_lock` para que dos requests
    concurrentes jamás carguen el mismo modelo en paralelo (doble RAM).
    """
    ok, msg = puede_cargar_modelo(model_key)
    if not ok:
        raise RuntimeError(msg)

    loader_fn, attr_name = _LOADERS[model_key]

    # Primer chequeo sin lock: fast path cuando el modelo ya está cargado.
    if globals()[attr_name] is not None:
        return globals()[attr_name]

    with _carga_lock:
        # Segundo chequeo: otro hilo pudo cargarlo mientras esperábamos el lock.
        if globals()[attr_name] is not None:
            return globals()[attr_name]
        print(f"[YARVIS-CHAT] Cargando Qwen {model_key}...")
        model = loader_fn()
        globals()[attr_name] = model
        print(f"[YARVIS-CHAT] Qwen {model_key} listo.")
        return model


def descargar_modelo(model_key: str):
    """Descarga un modelo Qwen de la RAM/VRAM y libera memoria."""
    with _carga_lock:
        attr = {"0.5B": "_llm_0_5", "0.8B": "_llm_0_8", "1.7B": "_llm_1_7"}.get(model_key)
        model = globals().get(attr)
        if model is not None:
            try:
                model.close()
            except Exception:
                pass
            globals()[attr] = None
            gc.collect()
            print(f"[YARVIS-CHAT] Qwen {model_key} descargado.")


def estado_modelos() -> dict:
    """Estado actual de los modelos cargados y la RAM disponible."""
    ram_gb = get_ram_gb()
    return {
        "ram_gb": round(ram_gb, 1),
        "models": {
            "0.5B": _llm_0_5 is not None,
            "0.8B": _llm_0_8 is not None,
            "1.7B": _llm_1_7 is not None,
        },
        "can_load_1_7b": ram_gb >= RAM_REQUERIDA["1.7B"],
    }

def ejecutar_chat(model: Llama, messages: list, max_words: int) -> str:
    """Ejecuta una conversación completa con el modelo y recorta el exceso."""
    respuesta = model.create_chat_completion(
        messages=messages,
        temperature=0.6,
        max_tokens=max_words * 4,
        top_p=0.85,
    )
    contenido = respuesta["choices"][0]["message"]["content"]
    contenido = limpiar_think(contenido)
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
