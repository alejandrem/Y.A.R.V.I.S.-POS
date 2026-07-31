"""
Cerebro de parseo: endpoints HTTP para el procesamiento de tickets y catalogos.
- parser: Analisis de tickets (regex + LLM Qwen) y parseo con mapeo de columnas
- carpeta: Procesamiento masivo de carpetas .txt (batch + stream SSE)
- matching: Vinculacion de productos parseados con el inventario

Los routers se montan en main.py.
"""
