"""
Parser de tickets mediante LLM (Qwen).
Carga primero el 0.5B; si la confianza es < 0.8, reintenta con el 0.8B.
Si la confianza sigue siendo < 0.8, reintenta con el 1.7B.
GPU: llama-cpp-python detecta CUDA/Metal automáticamente. Si no hay GPU, usa CPU.
"""

import gc
import json
import re
from llama_cpp import Llama

from parseador_de_tickets.llm.rutas_modelos import qwen0_5, qwen0_8, qwen1_7

_MODELOS_LLM: dict[str, Llama | None] = {"0.5B": None, "0.8B": None, "1.7B": None}

_NOMBRES_MODELO = {
    "0.5B": "Qwen 2.5 0.5B",
    "0.8B": "Qwen 3.5 0.8B",
    "1.7B": "Qwen 3 1.7B",
}

SYSTEM_PROMPT = """Eres un experto en parseo de tickets de punto de venta mexicano.
Analiza el siguiente ticket de texto plano y extrae la estructura.

Reglas:
- Identifica qué columna es: cantidad, producto, precio unitario, total
- Los precios siempre tienen $ o están en formato decimal (15.00)
- La cantidad siempre es un número entero al inicio de la línea
- El total es siempre la última columna numérica
- El nombre del producto es texto entre la cantidad y los precios
- Detecta si hay descuentos, impuestos (IVA), o notas extra
- BUSCA la fecha del ticket: puede estar en formatos como "15/03/2024", "2024-03-15",
  "15 de marzo de 2024", "Mar 15 2024", "Fecha: 15/03/24", "15-03-2024", etc.
- BUSCA la hora del ticket: puede estar en formatos como "14:32", "14:32:05",
  "2:32 PM", "Hora: 14:32", etc.
- Si no encuentras fecha u hora, devuelve null para esos campos.

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
  "fecha_ticket": "YYYY-MM-DD_O_NULL",
  "hora_ticket": "HH:MM_O_NULL",
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



_RUTAS_MODELO = {
    "0.5B": qwen0_5,
    "0.8B": qwen0_8,
    "1.7B": qwen1_7,
}


def _cargar_modelo(key: str) -> Llama:
    """Carga el modelo Qwen indicado (0.5B / 0.8B / 1.7B) o devuelve el ya cargado."""
    if _MODELOS_LLM[key] is None:
        nombre = _NOMBRES_MODELO[key]
        print(f"[YARVIS-IA] Cargando {nombre} para parseo de tickets...")
        _MODELOS_LLM[key] = Llama(
            model_path=_RUTAS_MODELO[key],
            n_ctx=4096,
            n_gpu_layers=-1,
            n_threads=4,
            verbose=False,
        )
        print(f"[YARVIS-IA] {nombre} listo.")
    return _MODELOS_LLM[key]


def _liberar(model):
    """Libera un modelo llama.cpp de RAM/VRAM (close() libera la memoria nativa)."""
    if model is None:
        return
    try:
        model.close()
    except Exception:
        pass


def descargar_modelos():
    count = 0
    for key in list(_MODELOS_LLM):
        if _MODELOS_LLM[key] is not None:
            _liberar(_MODELOS_LLM[key])
            _MODELOS_LLM[key] = None
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
    2. Si confianza < 0.8, reintenta con Qwen 3.5 0.8B.
    3. Si confianza sigue < 0.8, reintenta con Qwen 3 1.7B.
    Retorna: { "status": "ok", "mapeo": {...}, "confianza": 0.95 }
    """
    if not texto_ticket or not texto_ticket.strip():
        return {"status": "error", "error": "El texto del ticket está vacío"}

    try:
        # Intento 1: Qwen 2.5 0.5B
        model_0_5 = _cargar_modelo("0.5B")
        resultado = _ejecutar_analisis(model_0_5, texto_ticket)

        if resultado and "mapeo" in resultado:
            confianza = float(resultado.get("confianza", 0))
            resultado["confianza"] = confianza

            # Intento 2: Si confianza < 0.8, usar 0.8B
            if confianza < 0.8:
                print(f"[YARVIS-IA] Confianza baja ({confianza}), reintentando con Qwen 3.5 0.8B...")
                model_0_8 = _cargar_modelo("0.8B")
                resultado_0_8 = _ejecutar_analisis(model_0_8, texto_ticket)

                if resultado_0_8 and "mapeo" in resultado_0_8:
                    confianza_0_8 = float(resultado_0_8.get("confianza", 0))
                    if confianza_0_8 > confianza:
                        resultado_0_8["confianza"] = confianza_0_8
                        resultado_0_8["reintentado_con"] = "qwen3_5_0_8b"
                        return {"status": "ok", **resultado_0_8}

                # Intento 3: Si confianza sigue < 0.8, usar 1.7B
                print(f"[YARVIS-IA] Confianza aún baja ({confianza}), reintentando con Qwen 3 1.7B...")
                model_1_7 = _cargar_modelo("1.7B")
                resultado_1_7 = _ejecutar_analisis(model_1_7, texto_ticket)

                if resultado_1_7 and "mapeo" in resultado_1_7:
                    confianza_1_7 = float(resultado_1_7.get("confianza", 0))
                    if confianza_1_7 > confianza:
                        resultado_1_7["confianza"] = confianza_1_7
                        resultado_1_7["reintentado_con"] = "qwen3_1_7b"
                        return {"status": "ok", **resultado_1_7}

            resultado["reintentado_con"] = None
            return {"status": "ok", **resultado}

        # Si el 0.5B no devolvió JSON válido, intentar directo con 0.8B
        print("[YARVIS-IA] Qwen 0.5B no pudo analizar, usando Qwen 3.5 0.8B directamente...")
        model_0_8 = _cargar_modelo("0.8B")
        resultado_0_8 = _ejecutar_analisis(model_0_8, texto_ticket)

        if resultado_0_8 and "mapeo" in resultado_0_8:
            confianza_0_8 = float(resultado_0_8.get("confianza", 0))
            resultado_0_8["confianza"] = confianza_0_8
            resultado_0_8["reintentado_con"] = "qwen3_5_0_8b"
            return {"status": "ok", **resultado_0_8}

        # Si el 0.8B tampoco, intentar con 1.7B
        print("[YARVIS-IA] Qwen 0.8B no pudo analizar, usando Qwen 3 1.7B directamente...")
        model_1_7 = _cargar_modelo("1.7B")
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
