"""
Parser de tickets mediante LLM (Qwen).
Carga primero el 0.5B; si la confianza es < 0.8, reintenta con el 1.7B.
GPU: llama-cpp-python detecta CUDA/Metal automáticamente. Si no hay GPU, usa CPU.
"""

import gc
import json
import re
from llama_cpp import Llama

from modelos.qwen.rutas import qwen0_5, qwen1_7

_llm_0_5 = None
_llm_1_7 = None

SYSTEM_PROMPT = """Eres un experto en parseo de tickets de punto de venta mexicano.
Analiza el siguiente ticket de texto plano y extrae la estructura.

Reglas:
- Identifica qué columna es: cantidad, producto, precio unitario, total
- Los precios siempre tienen $ o están en formato decimal (15.00)
- La cantidad siempre es un número entero al inicio de la línea
- El total es siempre la última columna numérica
- El nombre del producto es texto entre la cantidad y los precios
- Detecta si hay descuentos, impuestos (IVA), o notas extra

Responde SOLO con JSON válido, sin explicaciones.

FORMATO DE RESPUESTA:
{
  "mapeo": {
    "formato_detectado": "CANTIDAD PRODUCTO PRECIO TOTAL",
    "columnas": {
      "cantidad": INDICE,
      "producto": INDICE,
      "precio_unitario": INDICE,
      "total": INDICE,
      "descuento": INDICE_O_NULL
    },
    "delimitador": "espacios_multiples",
    "moneda": "$",
    "total_columnas": NUMERO,
    "tiene_descuento": true_o_false,
    "tiene_iva": true_o_false
  },
  "ejemplo_parseado": [
    {
      "cantidad": NUMERO_ENTERO,
      "producto": "TEXTO LIMPIO",
      "precio_unitario": NUMERO_DECIMAL,
      "total": NUMERO_DECIMAL,
      "descuento": NUMERO_O_NULL
    }
  ],
  "confianza": NUMERO_ENTRE_0_Y_1,
  "notas": "EXPLICACION DEL FORMATO"
}"""


def _cargar_modelo_0_5() -> Llama:
    global _llm_0_5
    if _llm_0_5 is None:
        print("[YARVIS-IA] Cargando Qwen 2.5 0.5B para parseo de tickets...")
        _llm_0_5 = Llama(
            model_path=qwen0_5,
            n_ctx=4096,
            n_gpu_layers=-1,
            n_threads=4,
            verbose=False
        )
        print("[YARVIS-IA] Qwen 2.5 0.5B listo.")
    return _llm_0_5


def _cargar_modelo_1_7() -> Llama:
    global _llm_1_7
    if _llm_1_7 is None:
        print("[YARVIS-IA] Cargando Qwen 3 1.7B para parseo de tickets (confianza baja)...")
        _llm_1_7 = Llama(
            model_path=qwen1_7,
            n_ctx=4096,
            n_gpu_layers=-1,
            n_threads=4,
            verbose=False
        )
        print("[YARVIS-IA] Qwen 3 1.7B listo.")
    return _llm_1_7


def descargar_modelos():
    global _llm_0_5, _llm_1_7
    count = 0
    if _llm_0_5 is not None:
        del _llm_0_5
        _llm_0_5 = None
        count += 1
    if _llm_1_7 is not None:
        del _llm_1_7
        _llm_1_7 = None
        count += 1
    gc.collect()
    if count > 0:
        print(f"[YARVIS-IA] {count} modelo(s) descargado(s) de VRAM.")
    return count


def _extraer_json(respuesta: str) -> dict | None:
    match = re.search(r'\{[\s\S]*\}', respuesta)
    if match:
        try:
            return json.loads(match.group())
        except json.JSONDecodeError:
            return None
    return None


def _ejecutar_analisis(model: Llama, texto: str) -> dict | None:
    lineas = [l for l in texto.strip().splitlines() if l.strip()]
    texto_analizar = "\n".join(lineas[:20])

    user_prompt = f"""TICKET A ANALIZAR:
---
{texto_analizar}
---

Analiza este ticket y responde SOLAMENTE con el JSON válido."""

    respuesta = model.create_chat_completion(
        messages=[
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": user_prompt}
        ],
        temperature=0.1,
        max_tokens=2048,
        top_p=0.9
    )

    contenido = respuesta["choices"][0]["message"]["content"]
    return _extraer_json(contenido)


def analizar_ticket(texto_ticket: str) -> dict:
    """
    Analiza un ticket TXT.
    1. Intenta con Qwen 2.5 0.5B.
    2. Si confianza < 0.8, reintenta con Qwen 3 1.7B.
    Retorna: { "status": "ok", "mapeo": {...}, "confianza": 0.95 }
    """
    if not texto_ticket or not texto_ticket.strip():
        return {"status": "error", "error": "El texto del ticket está vacío"}

    try:
        # Intento 1: Qwen 2.5 0.5B
        model_0_5 = _cargar_modelo_0_5()
        resultado = _ejecutar_analisis(model_0_5, texto_ticket)

        if resultado and "mapeo" in resultado:
            confianza = float(resultado.get("confianza", 0))
            resultado["confianza"] = confianza

            # Intento 2: Si confianza < 0.8, usar 1.7B
            if confianza < 0.8:
                print(f"[YARVIS-IA] Confianza baja ({confianza}), reintentando con Qwen 3 1.7B...")
                model_1_7 = _cargar_modelo_1_7()
                resultado_1_7 = _ejecutar_analisis(model_1_7, texto_ticket)

                if resultado_1_7 and "mapeo" in resultado_1_7:
                    confianza_1_7 = float(resultado_1_7.get("confianza", 0))
                    if confianza_1_7 > confianza:
                        resultado_1_7["confianza"] = confianza_1_7
                        resultado_1_7["reintentado_con"] = "qwen3_1_7b"
                        return {"status": "ok", **resultado_1_7}

            resultado["reintentado_con"] = None
            return {"status": "ok", **resultado}

        # Si el 0.5B no devolvió JSON válido, intentar directo con 1.7B
        print("[YARVIS-IA] Qwen 0.5B no pudo analizar, usando Qwen 3 1.7B directamente...")
        model_1_7 = _cargar_modelo_1_7()
        resultado_1_7 = _ejecutar_analisis(model_1_7, texto_ticket)

        if resultado_1_7 and "mapeo" in resultado_1_7:
            confianza_1_7 = float(resultado_1_7.get("confianza", 0))
            resultado_1_7["confianza"] = confianza_1_7
            resultado_1_7["reintentado_con"] = "qwen3_1_7b"
            return {"status": "ok", **resultado_1_7}

        return {
            "status": "error",
            "error": "Ningún modelo pudo analizar el ticket"
        }

    except Exception as e:
        return {"status": "error", "error": f"Error al analizar ticket: {str(e)}"}
