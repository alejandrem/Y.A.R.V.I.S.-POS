# rutas de los modelos qwen (detecta automaticamente el home del usuario)
import os

_HOME = os.path.expanduser("~")
_LMSTUDIO_MODELS = os.path.join(_HOME, ".lmstudio", "models")

qwen0_5 = os.path.join(_LMSTUDIO_MODELS, "lmstudio-community", "Qwen2.5-0.5B-Instruct-GGUF", "Qwen2.5-0.5B-Instruct-Q4_K_M.gguf")

qwen1_7 = os.path.join(_LMSTUDIO_MODELS, "lmstudio-community", "Qwen3-1.7B-GGUF", "Qwen3-1.7B-Q4_K_M.gguf")