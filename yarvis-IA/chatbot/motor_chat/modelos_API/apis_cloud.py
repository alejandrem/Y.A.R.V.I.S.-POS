"""
☁️ apis_cloud.py — Respuestas por API de proveedores de IA (Gemini y OpenCode Zen).

Se encarga de:
    - Definir la configuración de cada proveedor (URL base, modelo por defecto, formato).
    - Convertir los mensajes de Y.A.R.V.I.S. al formato que espera cada proveedor.
    - Generar respuestas completas o por streaming vía HTTP (httpx).
    - Listar los modelos gratuitos disponibles de cada proveedor (con caché).

No toca hardware ni base de datos: recibe los mensajes ya construidos.
"""

import json
import time

import httpx

from ..modelos_local.prompts import _separar_think

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

_TIMEOUT = httpx.Timeout(120.0, connect=30.0)

# Modelos gratuitos de OpenCode que no terminan en "-free" (lista extra).
_MODELOS_FREE_EXTRA = {"big-pickle"}

# Caché del listado de modelos (TTL 60s): evita gastar cuota pegándole a
# los endpoints de /models en cada apertura del selector.
_MODELOS_CACHE: dict[str, tuple[float, list[dict]]] = {}
_MODELOS_CACHE_TTL = 60.0

# Orden de fallback para los modelos gratuitos de OpenCode cuando uno se
# satura (429). Empieza por el más estable y baja a los demás.
_ORDEN_FALLBACK_FREE = [
    "mimo-v2.5-free",
    "nemotron-3-ultra-free",
    "nemotron-3.5-lightning-free",
    "hy3-free",
    "laguna-s-2.1-free",
    "deepseek-v4-flash-free",
    "big-pickle",
]


def nombre_proveedor(provider: str) -> str:
    """Nombre amigable del proveedor (para mostrarlo en el modelo usado)."""
    cfg = PROVIDERS.get(provider)
    return cfg["name"] if cfg else provider


def _es_free(model_id: str) -> bool:
    """Un modelo de OpenCode es gratuito si termina en '-free' o está en la lista extra."""
    return model_id.endswith("-free") or model_id in _MODELOS_FREE_EXTRA


def _siguiente_modelo_free(model_id: str) -> str | None:
    """Devuelve el siguiente modelo gratuito de OpenCode a probar, o None si se agotaron."""
    if model_id in _ORDEN_FALLBACK_FREE:
        idx = _ORDEN_FALLBACK_FREE.index(model_id)
        if idx + 1 < len(_ORDEN_FALLBACK_FREE):
            return _ORDEN_FALLBACK_FREE[idx + 1]
        return None
    return _ORDEN_FALLBACK_FREE[0]


def _normalizar_mensajes(messages: list[dict]) -> list[dict]:
    """Junta mensajes consecutivos con el mismo rol (evita rechazos de APIs).

    Respeta los mensajes de tool_calls / de rol 'tool' (function calling):
    esos no se fusionan porque llevan tool_call_id y NO deben pegarse a otros.
    """
    normalized: list[dict] = []
    for m in messages:
        role = m.get("role", "user")
        content = m.get("content") or ""
        if m.get("tool_calls") or role == "tool" or m.get("tool_call_id"):
            normalized.append(m)
            continue
        if normalized and normalized[-1]["role"] == role and not normalized[-1].get("tool_calls"):
            normalized[-1]["content"] += f"\n{content}"
        else:
            normalized.append({"role": role, "content": content})
    return normalized


def _iter_delta_lineas(resp, usage: dict | None, display: str, tool_calls: dict | None = None):
    """Consume un stream SSE estilo OpenAI y captura tokens y uso.

    'usage' (si se pasa) se rellena con el chunk final de uso que los
    proveedores envían con `stream_options.include_usage` activo.

    'tool_calls' (dict índice -> dict, si se pasa) acumula los tool_calls
    que llegan troceados entre chunks de streaming (function calling).
    """
    for line in resp.iter_lines():
        if not line or not line.startswith("data:"):
            continue
        data = line[5:].strip()
        if data == "[DONE]":
            break
        try:
            chunk = json.loads(data)
        except json.JSONDecodeError:
            continue
        if usage is not None and chunk.get("usage"):
            usage.update(chunk["usage"])
        delta = (chunk.get("choices") or [{}])[0].get("delta") or {}
        if tool_calls is not None and delta.get("tool_calls"):
            for tc in delta["tool_calls"]:
                idx = tc.get("index", 0)
                entrada = tool_calls.setdefault(
                    idx, {"id": "", "name": "", "arguments": ""}
                )
                if tc.get("id"):
                    entrada["id"] = tc["id"]
                fn = tc.get("function", {})
                if fn.get("name"):
                    entrada["name"] = fn["name"]
                if fn.get("arguments"):
                    entrada["arguments"] += fn["arguments"]
        token = delta.get("content", "")
        if token:
            yield token, display


def _formato_tool_calls(tool_calls: dict) -> list[dict]:
    """Convierte tool_calls acumulados del stream al formato OpenAI estándar."""
    return [
        {
            "id": tc.get("id") or f"call_{idx}",
            "type": "function",
            "function": {"name": tc.get("name", ""), "arguments": tc.get("arguments", "{}")},
        }
        for idx, tc in sorted(tool_calls.items())
    ]


def _iter_openai_compatible(
    cfg: dict,
    api_key: str,
    model: str,
    messages: list,
    display: str,
    usage: dict | None = None,
    tools: list | None = None,
    ejecutar_tool=None,
):
    """OpenCode Zen (y cualquiera compatible con /chat/completions).

    Si se pasan 'tools' + 'ejecutar_tool' y el modelo pide llamar una tool,
    se ejecuta localmente y se hace una segunda llamada con los resultados
    (streaming del texto final).
    """
    url = f"{cfg['base_url']}/chat/completions"
    headers = {"Authorization": f"Bearer {api_key}"}
    normalized = _normalizar_mensajes(messages)
    for intentar_con_uso in (True, False):
        body = {
            "model": model,
            "messages": normalized,
            "temperature": 0.6,
            "max_tokens": 2048,
            "stream": True,
        }
        if intentar_con_uso:
            body["stream_options"] = {"include_usage": True}
        if tools:
            body["tools"] = tools
            body["tool_choice"] = "auto"
        cedio_tokens = False
        try:
            with httpx.stream("POST", url, headers=headers, json=body, timeout=_TIMEOUT) as resp:
                resp.raise_for_status()
                tool_calls: dict = {}
                for token, _ in _iter_delta_lineas(resp, usage, display, tool_calls):
                    cedio_tokens = True
                    yield token, display
            if tool_calls:
                yield from _resolver_tool_calls(
                    cfg, api_key, model, normalized, display, usage,
                    tools, ejecutar_tool, _formato_tool_calls(tool_calls),
                )
            return
        except httpx.HTTPStatusError as e:
            # Reintento sin include_usage SOLO si el 400 llegó antes de ceder
            # ningún token; si ya cedimos, el cliente vería la salida repetida.
            if e.response.status_code != 400 or not intentar_con_uso or cedio_tokens:
                raise
            print("[YARVIS-CHAT] El proveedor no aceptó include_usage; reintento sin él.")


def _resolver_tool_calls(
    cfg, api_key, model, normalized, display, usage, tools, ejecutar_tool, tool_calls
):
    """Ejecuta las tools pedidas por el modelo y continúa el streaming con los resultados."""
    if not ejecutar_tool:
        yield "…", display
        return

    mensaje_assistant = {
        "role": "assistant",
        "content": None,
        "tool_calls": tool_calls,
    }
    mensajes_tool: list[dict] = []
    for tc in tool_calls:
        try:
            resultado = ejecutar_tool(tc)
        except Exception as e:  # la tool nunca debe tumbar el chat
            resultado = json.dumps({"error": str(e)}, ensure_ascii=False)
        mensajes_tool.append({
            "role": "tool",
            "tool_call_id": tc.get("id") or f"call_{tool_calls.index(tc)}",
            "content": resultado,
        })

    yield from _iter_openai_compatible(
        cfg, api_key, model,
        [*normalized, mensaje_assistant, *mensajes_tool],
        display, usage, tools, ejecutar_tool,
    )


def _iter_google(cfg: dict, api_key: str, model: str, messages: list, display: str, usage: dict | None = None):
    """Gemini — formato contents + system_instruction."""
    url = f"{cfg['base_url']}/models/{model}:streamGenerateContent"
    system = next((m["content"] for m in messages if m.get("role") == "system"), "")
    contents = []
    for m in _normalizar_mensajes(messages):
        if m["role"] in ("user", "assistant"):
            contents.append({
                "role": "model" if m["role"] == "assistant" else "user",
                "parts": [{"text": m["content"]}],
            })
    body = {"contents": contents}
    if system:
        body["system_instruction"] = {"parts": [{"text": system}]}
    params = {"key": api_key, "alt": "sse"}
    with httpx.stream("POST", url, params=params, json=body, timeout=_TIMEOUT) as resp:
        resp.raise_for_status()
        for line in resp.iter_lines():
            if not line or not line.startswith("data:"):
                continue
            try:
                data = json.loads(line[5:].strip())
            except json.JSONDecodeError:
                continue
            if usage is not None:
                meta = data.get("usageMetadata")
                if meta:
                    usage["prompt_tokens"] = meta.get("promptTokenCount", 0)
                    usage["completion_tokens"] = meta.get("candidatesTokenCount", 0)
                    usage["total_tokens"] = meta.get("totalTokenCount", 0)
            parts = (data.get("candidates") or [{}])[0].get("content", {}).get("parts", [])
            for part in parts:
                token = part.get("text", "")
                if token:
                    yield token, display


def _error_amigable(e: httpx.HTTPStatusError) -> str:
    """Traduce errores HTTP del proveedor a mensajes claros en español."""
    status = e.response.status_code
    if status == 429:
        return "Error 429: muchas preguntas al mismo tiempo. Espera 1 minuto y reintenta."
    if status in (401, 403):
        return f"API key inválida (error {status}). Revisa tu clave en 'Agregar API'."
    if status == 402:
        return "Cuota agotada (error 402): revisa tu plan del proveedor."
    if status == 404:
        return f"Modelo no disponible (error 404). Revisa el nombre del modelo del proveedor."
    return f"Error {status} del proveedor: {e}"


def listar_modelos(provider: str, api_key: str = "") -> list[dict]:
    """Lista los modelos disponibles de un proveedor (solo gratuitos en OpenCode).

    Devuelve [{'id': ..., 'name': ...}] con caché de 60 segundos.
    """
    cfg = PROVIDERS.get(provider)
    if cfg is None:
        raise ValueError(f"Proveedor no soportado: {provider}")

    ahora = time.time()
    cached = _MODELOS_CACHE.get(provider)
    if cached and ahora - cached[0] < _MODELOS_CACHE_TTL:
        return cached[1]

    if provider == "google":
        if not api_key:
            raise ValueError("Falta la API key de Google (Gemini).")
        url = f"{cfg['base_url']}/models"
        resp = httpx.get(url, params={"key": api_key}, timeout=_TIMEOUT)
        resp.raise_for_status()
        modelos = [
            {
                "id": m["name"].replace("models/", ""),
                "name": m.get("displayName") or m["name"].replace("models/", ""),
            }
            for m in resp.json().get("models", [])
            if "generateContent" in m.get("supportedGenerationMethods", [])
        ]
    else:
        url = f"{cfg['base_url']}/models"
        headers = {}
        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"
        resp = httpx.get(url, headers=headers, timeout=_TIMEOUT)
        resp.raise_for_status()
        payload = resp.json()
        lista = payload.get("data", payload if isinstance(payload, list) else [])
        modelos = [
            {"id": m.get("id", ""), "name": m.get("name") or m.get("id", "")}
            for m in lista
            if _es_free(m.get("id", ""))
        ]

    modelos.sort(key=lambda x: x["id"])
    _MODELOS_CACHE[provider] = (time.time(), modelos)
    return modelos


def generar_stream(
    provider: str,
    api_key: str,
    model: str,
    messages: list,
    usage: dict | None = None,
    tools: list | None = None,
    ejecutar_tool=None,
):
    """Genera (token, nombre_mostrado) por streaming para el proveedor indicado.

    Si se pasa 'usage' (dict), se rellena con los tokens reales del proveedor
    (prompt/completion/total) cuando el chunk final de uso lo reporta.

    Si se pasan 'tools' + 'ejecutar_tool', el modelo puede pedir llamar una
    herramienta (function calling): se ejecuta localmente y se continúa con
    el texto final.

    Anti-saturación: ante un 429 de un modelo free de OpenCode se cambia
    automáticamente al siguiente modelo gratuito disponible; si no hay más
    modelos a los que saltar, respeta el header 'retry-after' y reintenta
    una sola vez antes de rendirse.
    """
    cfg = PROVIDERS.get(provider)
    if cfg is None:
        raise ValueError(f"Proveedor no soportado: {provider}")
    if not api_key:
        raise ValueError("Falta la API key del proveedor.")

    model = model or cfg["default_model"]
    display = cfg["name"]

    for intento in range(2):
        try:
            if provider == "google":
                yield from _iter_google(cfg, api_key, model, messages, display, usage)
            else:
                yield from _iter_openai_compatible(
                    cfg, api_key, model, messages, display, usage, tools, ejecutar_tool
                )
            return
        except httpx.HTTPStatusError as e:
            if e.response.status_code == 429 and intento == 0:
                if provider == "opencode" and _es_free(model):
                    siguiente = _siguiente_modelo_free(model)
                    if siguiente:
                        print(f"[YARVIS] {model} saturado (429), cambiando a {siguiente}")
                        model = siguiente
                        continue
                retry = e.response.headers.get("retry-after", "")
                espera = min(int(retry), 20) if retry.isdigit() else 5
                time.sleep(espera)
                continue
            raise ValueError(_error_amigable(e)) from e
        except httpx.RequestError as e:
            raise ValueError(f"No se pudo conectar con {display}: {e}") from e


def generar_completo(
    provider: str,
    api_key: str,
    model: str,
    messages: list,
    tools: list | None = None,
    ejecutar_tool=None,
) -> str:
    """Respuesta completa (sin streaming) limpiando los bloques  thinking.

    El modelo puede intercalar su razonamiento (🧠 thinking...response) en el
    texto; aquí se filtra igual que en el streaming con _separar_think y se
    reconstruye solo la parte 'token' (respuesta final).
    """
    stream = (token for token, _ in generar_stream(
        provider, api_key, model, messages, tools=tools, ejecutar_tool=ejecutar_tool
    ))
    return "".join(
        texto for tipo, texto in _separar_think(stream, 10**9) if tipo == "token"
    )
