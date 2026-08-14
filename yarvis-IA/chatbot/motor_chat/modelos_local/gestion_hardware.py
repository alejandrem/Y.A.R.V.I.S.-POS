"""
🧠 gestion_hardware.py — Gestión de hardware y modelos Qwen.

Se encarga de:
    - Medir la RAM disponible del sistema (/proc/meminfo en Linux).
    - Decidir qué modelo Qwen (0.5B / 0.8B / 1.7B) se puede cargar.
    - Cargar, descargar y consultar el estado de los modelos.
    - Ejecutar la inferencia del modelo (respuestas completas).

No sabe nada de la base de datos ni de la API web.
"""

import gc

from llama_cpp import Llama
from parseador_de_tickets.llm.rutas_modelos import qwen0_5, qwen0_8, qwen1_7

from .prompts import limpiar_think

_llm_0_5 = None
_llm_0_8 = None
_llm_1_7 = None

WORD_LIMITS = {"0.5B": 2000, "0.8B": 3000, "1.7B": 4000}

# RAM mínima (GB) requerida por cada modelo
_RAM_REQUERIDA = {"0.5B": 0.0, "0.8B": 1.0, "1.7B": 4.0}

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
    needed = _RAM_REQUERIDA.get(model_key, 2.0)
    if ram_gb < needed:
        return False, f"RAM insuficiente: {ram_gb:.1f}GB disponibles, {model_key} necesita ≥{needed}GB"
    return True, f"OK ({ram_gb:.1f}GB disponibles)"


def cargar_modelo(model_key: str) -> Llama:
    """Carga un modelo Qwen (0.5B/0.8B/1.7B) o devuelve el ya cargado."""
    ok, msg = puede_cargar_modelo(model_key)
    if not ok:
        raise RuntimeError(msg)

    if model_key == "0.5B" and _llm_0_5 is not None:
        return _llm_0_5
    if model_key == "0.8B" and _llm_0_8 is not None:
        return _llm_0_8
    if model_key == "1.7B" and _llm_1_7 is not None:
        return _llm_1_7

    loader_fn, attr_name = _LOADERS[model_key]
    print(f"[YARVIS-CHAT] Cargando Qwen {model_key}...")
    model = loader_fn()
    globals()[attr_name] = model
    print(f"[YARVIS-CHAT] Qwen {model_key} listo.")
    return model


def descargar_modelo(model_key: str):
    """Descarga un modelo Qwen de la RAM/VRAM y libera memoria."""
    global _llm_0_5, _llm_0_8, _llm_1_7
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
        "can_load_1_7b": ram_gb >= 4.0,
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
