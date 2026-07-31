# rutas de los modelos qwen (detecta automaticamente el home del usuario)
import os

_HOME = os.path.expanduser("~")
_LMSTUDIO_MODELS = os.path.join(_HOME, ".lmstudio", "models")

qwen0_5 = os.path.join(_LMSTUDIO_MODELS, "lmstudio-community", "Qwen2.5-0.5B-Instruct-GGUF", "Qwen2.5-0.5B-Instruct-Q4_K_M.gguf")

qwen0_8 = os.path.join(_LMSTUDIO_MODELS, "unsloth", "Qwen3.5-0.8B-GGUF", "Qwen3.5-0.8B-Q4_K_M.gguf")

qwen1_7 = os.path.join(_LMSTUDIO_MODELS, "lmstudio-community", "Qwen3-1.7B-GGUF", "Qwen3-1.7B-Q3_K_L.gguf")


def verificar_modelos():
    """Verifica que los archivos de modelo existan y retorna estado."""
    modelos = {
        "0.5B": {"ruta": qwen0_5, "existe": os.path.exists(qwen0_5), "tamano_mb": 0},
        "0.8B": {"ruta": qwen0_8, "existe": os.path.exists(qwen0_8), "tamano_mb": 0},
        "1.7B": {"ruta": qwen1_7, "existe": os.path.exists(qwen1_7), "tamano_mb": 0},
    }
    for key, info in modelos.items():
        if info["existe"]:
            info["tamano_mb"] = round(os.path.getsize(info["ruta"]) / (1024 * 1024), 1)
    return modelos


if __name__ == "__main__":
    print("=== Verificación de Modelos Qwen ===")
    for key, info in verificar_modelos().items():
        status = "✅" if info["existe"] else "❌"
        if info["existe"]:
            print(f"  {status} Qwen {key}: {info['tamano_mb']}MB — {info['ruta']}")
        else:
            print(f"  {status} Qwen {key}: NO ENCONTRADO — {info['ruta']}")
