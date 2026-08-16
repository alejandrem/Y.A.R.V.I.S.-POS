# rutas de los modelos qwen (detecta automaticamente el home del usuario)
#
# LM Studio puede bajar el mismo modelo a distintos namespaces/filenames
# según la org de HuggingFace que se elija (Qwen/…, lmstudio-community/…,
# unsloth/…) y con cualquier quantización (Q4_K_M, Q3_K_L, …). Para no
# depender de nombres exactos, aquí se BUSCA el primer .gguf real (sin
# mmproj) dentro de los directorios candidatos de cada modelo.
import glob
import os

_HOME = os.path.expanduser("~")
_LMSTUDIO_MODELS = os.path.join(_HOME, ".lmstudio", "models")

# Namespaces/orgs reales en HF donde puede vivir cada modelo.
_CANDIDATOS: dict[str, list[str]] = {
    "0.5B": [
        os.path.join("Qwen", "Qwen2.5-0.5B-Instruct-GGUF"),
        os.path.join("lmstudio-community", "Qwen2.5-0.5B-Instruct-GGUF"),
    ],
    "0.8B": [
        os.path.join("unsloth", "Qwen3.5-0.8B-GGUF"),
        os.path.join("Qwen", "Qwen3-0.6B-GGUF"),
    ],
    "1.7B": [
        os.path.join("lmstudio-community", "Qwen3-1.7B-GGUF"),
        os.path.join("qwen", "Qwen3-1.7B-GGUF"),
    ],
}

# Preferencia de quant (se usa la primera disponible).
_PREFERENCIA_QUANT = ("Q4_K_M", "Q3_K_L", "Q3_K_M", "Q4_0", "Q5_K_M", "Q8_0")


def _buscar_gguf(rel: str) -> str | None:
    """Devuelve la primera ruta .gguf (no mmproj) del directorio del modelo."""
    folder = os.path.join(_LMSTUDIO_MODELS, rel)
    if not os.path.isdir(folder):
        return None
    archivos = [
        f for f in glob.glob(os.path.join(folder, "*.gguf"))
        if "mmproj" not in os.path.basename(f)
    ]
    if not archivos:
        return None
    for preferida in _PREFERENCIA_QUANT:
        for f in archivos:
            if preferida.lower() in os.path.basename(f).lower():
                return f
    return archivos[0]


def _resolver(key: str) -> str:
    """Busca el modelo en los directorios candidatos; fallback a la ruta vieja."""
    for rel in _CANDIDATOS[key]:
        ruta = _buscar_gguf(rel)
        if ruta:
            return ruta
    return os.path.join(_LMSTUDIO_MODELS, _CANDIDATOS[key][0], "modelo_no_encontrado.gguf")


qwen0_5 = _resolver("0.5B")
qwen0_8 = _resolver("0.8B")
qwen1_7 = _resolver("1.7B")


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