"""
⚙️ variables.py — Constantes y parámetros del motor de chat LOCAL (modelos Qwen).

Solo datos, SIN imports del proyecto (evita imports circulares).
Única fuente de verdad para límites/tuning; los demás módulos importan desde aquí.
"""

# Claves oficiales de los modelos Qwen locales. Se usan en endpoints.py,
# el parser de tickets y la descarga por inactividad.
MODELOS = ("0.5B", "0.8B", "1.7B")

# RAM mínima (GB) necesaria para cargar cada modelo (quant Q4).
# 0.5B → 0.0GB (casi nada) · 0.8B → 0.5GB · 1.7B → ~1.3GB.
RAM_REQUERIDA = {"0.5B": 0.0, "0.8B": 0.5, "1.7B": 1.3}

# Límite de palabras de la respuesta final por modelo. Pasado ese límite el
# modelo local empieza a divagar, así que se recorta la respuesta.
WORD_LIMITS = {"0.5B": 2000, "0.8B": 3000, "1.7B": 4000}

# Contrato de campos del producto que expone la tool search_inventory
# (function calling). Documenta qué columnas de `productos` devuelve.
CAMPOS_TOOL = ("nombre", "precio_venta", "stock", "categoria", "descripcion")