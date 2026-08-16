"""
⚙️ variables.py — Constantes y parámetros del motor de chat EN LA NUBE (APIs).

Solo datos, SIN imports del proyecto (evita imports circulares).
Única fuente de verdad para proveedores/timeouts; apis_cloud.py importa desde aquí.
"""

# Proveedores de nube soportados: URL base + modelo por defecto de cada uno.
PROVIDERS = {
    "google": {
        "name": "Gemini",
        "base_url": "https://generativelanguage.googleapis.com/v1beta",
        "default_model": "gemini-2.0-flash",
    },
    "opencode": {
        "name": "OpenCode",
        "base_url": "https://opencode.ai/zen/v1",
        "default_model": "mimo-v2.5-free",
    },
}

# Timeouts HTTP hacia los proveedores (segundos). apis_cloud.py los pasa a
# httpx.Timeout(TIMEOUT_READ, connect=TIMEOUT_CONNECT).
TIMEOUT_READ = 120.0
TIMEOUT_CONNECT = 30.0

# Modelos gratuitos de OpenCode que NO terminan en "-free" pero sí lo son.
MODELOS_FREE_EXTRA = {"big-pickle"}

# Orden de fallback cuando un modelo free de OpenCode satura (429): se cambia
# automáticamente al siguiente de la lista hasta agotarlos.
ORDEN_FALLBACK_FREE = [
    "mimo-v2.5-free",
    "nemotron-3-ultra-free",
    "nemotron-3.5-lightning-free",
    "hy3-free",
    "laguna-s-2.1-free",
    "deepseek-v4-flash-free",
    "big-pickle",
]

# TTL (segundos) de la caché del listado de modelos de /cloud_models.
# Evita golpear los endpoints /models de los proveedores en cada apertura.
MODELOS_CACHE_TTL = 60.0