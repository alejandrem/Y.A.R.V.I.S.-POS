"""
📝 prompts.py — Ingeniería de prompts y limpieza de respuestas.

Se encarga de:
    - Construir el System Prompt de Y.A.R.V.I.S. con el contexto de la tienda.
    - Armar la lista de mensajes (system + historial) que recibe el modelo.
    - Limpiar los bloques <think>...</think> que a veces genera el modelo.

No toca hardware ni base de datos directamente: pide los datos a otros módulos.
"""

import re

from .cache import obtener_contexto_inteligente
from .consultas_db import obtener_tienda_info

def limpiar_think(texto: str) -> str:
    """Elimina bloques  thinking / think ... response / <think> de la respuesta."""
    texto = re.sub(r'<think>.*?</think>', '', texto, flags=re.DOTALL)
    texto = re.sub(r'\s*think(?:ing)?\b.*?\s*response\b', '', texto, flags=re.DOTALL)
    texto = re.sub(r'\s*think(?:ing)?\b.*', '', texto, flags=re.DOTALL)
    return texto.strip()


# Marcadores que abren/cierran un bloque de razonamiento del modelo.
# " think" / " thinking" abren; " response" cierra. Aceptan espacio previo o
# inicio de texto (los razonamientos llegan así desde el proveedor).
_OPEN_THINK = re.compile(r"<think>|(?:\s+|^)think(?:ing)?\b")
_CLOSE_THINK = re.compile(r"</think>|<response>|(?:\s+|^)response\b")

# Prefijos parciales que podrían ser el inicio de un marcador (llegan troceados).
_MARCADORES = ["<think>", " think", " thinking", " response", "<response>"]


def _cola_potencial_marcador(texto: str) -> int:
    """Cuántos caracteres del final de `texto` podrían ser el inicio de un marcador."""
    for m in _MARCADORES:
        for i in range(1, min(len(m), len(texto)) + 1):
            if m[:i] == texto[-i:]:
                return i
    return 0


def _separar_think(textos, max_w):
    """Separa los bloques  think / <think> del texto final.

    Recibe texto crudo por trozos y emite ('token'|'think', texto):
    - 'think': razonamiento del modelo (se muestra sombreado).
    - 'token': respuesta final (texto real).

    Los marcadores pueden llegar partidos entre chunks de streaming: se acumula
    en un buffer y solo se flushea lo seguro, reteniendo la cola que aún podría
    ser el inicio de un marcador.
    """
    word_count = 0
    in_think = False
    buffer = ""
    for content in textos:
        if not content:
            continue
        buffer += content
        while buffer:
            patron = _CLOSE_THINK if in_think else _OPEN_THINK
            match = patron.search(buffer)
            if match is None:
                cola = _cola_potencial_marcador(buffer)
                if cola > 0:
                    seguro, buffer = buffer[:-cola], buffer[-cola:]
                else:
                    seguro, buffer = buffer, ""
                if not seguro:
                    break
                if in_think:
                    yield "think", seguro
                else:
                    word_count += len(seguro.split())
                    if word_count > max_w:
                        return
                    yield "token", seguro
                continue
            idx = match.start()
            pre = buffer[:idx]
            if pre:
                if in_think:
                    yield "think", pre
                else:
                    word_count += len(pre.split())
                    if word_count > max_w:
                        return
                    yield "token", pre
            buffer = buffer[match.end():]
            in_think = not in_think
    if buffer:
        yield ("think" if in_think else "token"), buffer


def construir_system_prompt(contexto_db: str, tienda_info: dict) -> str:
    """Arma el System Prompt con los datos de la tienda y el contexto."""
    nombre = tienda_info.get("nombre", "la tienda")
    ubic = tienda_info.get("ubicacion", "")
    return f"""Eres Y.A.R.V.I.S., el asistente inteligente de negocios de "{nombre}"{f' en {ubic}' if ubic else ''}.

Eres un empleado experto de la tienda: conoces el inventario, las ventas, los empleados y las finanzas en tiempo real. No eres un bot genérico: usas los DATOS REALES que aparecen abajo para responder.

CAPACIDADES (qué puedes hacer):
- Consultar productos: precio, stock, categoría, disponibilidad ("¿tienen X?", "¿cuánto cuesta Y?").
- Reportar stock bajo o agotado ("¿qué se está por agotar?", "¿qué no hay?").
- Reportar ventas: totales de hoy, de la semana y de los últimos 7 días.
- Listar los productos más vendidos.
- Reportar datos de empleados y anomalías si se te pide.

REGLAS DE RESPUESTA:
1. Si el usuario pregunta qué puedes hacer o quién eres, responde explicando tus CAPACIDADES (o que eres el asistente de la tienda) de forma directa. NO respondas con un saludo.
2. El saludo SOLO está permitido si es el PRIMER mensaje del usuario y es solo un saludo ("hola", "buenas"). Después de eso, jamás vuelvas a saludar. Responde directamente al contenido.
3. Responde SIEMPRE con los DATOS DE LA TIENDA que están abajo. Si no aparecen datos para esa pregunta, dilo con honestidad: "No tengo esa información en este momento."
4. Sé conciso: 2-4 oraciones. Usa markdown (listas, negritas) cuando ayude.
5. NUNCA inventes precios, stocks, ventas ni productos que no estén en los DATOS DE LA TIENDA.
6. NUNCA escribas bloques <think>. Solo escribe tu respuesta final.

DATOS DE LA TIENDA (datos reales de {nombre}):
{contexto_db}"""


def construir_mensajes(role: str, messages: list, ultimo: str) -> list[dict]:
    """Arma la lista de mensajes [system + historial] para el modelo."""
    contexto_db = obtener_contexto_inteligente(role, ultimo)
    tienda = obtener_tienda_info()
    system_prompt = construir_system_prompt(contexto_db, tienda)

    chat_messages = [{"role": "system", "content": system_prompt}]
    for m in messages:
        chat_messages.append({
            "role": m.role if hasattr(m, "role") else m["role"],
            "content": m.content if hasattr(m, "content") else m["content"],
        })
    return chat_messages
