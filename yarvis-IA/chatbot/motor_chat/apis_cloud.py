"""
☁️ apis_cloud.py — Respuestas por API de proveedores de IA (Gemini y OpenCode Zen).

Se encarga de:
    - Definir la configuración de cada proveedor (URL base, modelo por defecto, formato).
    - Convertir los mensajes de Y.A.R.V.I.S. al formato que espera cada proveedor.
    - Generar respuestas completas o por streaming vía HTTP (httpx).

No toca hardware ni base de datos: recibe los mensajes ya construidos.
"""

import json

import httpx

PROVIDERS = {
    "google": {
        "name": "Gemini",
        "base_url": "https://generativelanguage.googleapis.com/v1beta",
        "default_model": "gemini-2.0-flash",
    },
    "opencode": {
        "name": "OpenCode",
        "base_url": "https://opencode.ai/zen/v1",
        "default_model": "deepseek-v4-flash-free",
    },
}

_TIMEOUT = httpx.Timeout(120.0, connect=30.0)


def nombre_proveedor(provider: str) -> str:
    """Nombre amigable del proveedor (para mostrarlo en el modelo usado)."""
    cfg = PROVIDERS.get(provider)
    return cfg["name"] if cfg else provider


def _normalizar_mensajes(messages: list[dict]) -> list[dict]:
    """Junta mensajes consecutivos con el mismo rol (evita rechazos de APIs)."""
    normalized: list[dict] = []
    for m in messages:
        role = m.get("role", "user")
        content = m.get("content", "")
        if normalized and normalized[-1]["role"] == role:
            normalized[-1]["content"] += f"\n{content}"
        else:
            normalized.append({"role": role, "content": content})
    return normalized


def _iter_openai_compatible(cfg: dict, api_key: str, model: str, messages: list, display: str):
    """OpenCode Zen (y cualquiera compatible con /chat/completions)."""
    url = f"{cfg['base_url']}/chat/completions"
    headers = {"Authorization": f"Bearer {api_key}"}
    body = {
        "model": model,
        "messages": _normalizar_mensajes(messages),
        "temperature": 0.6,
        "max_tokens": 2048,
        "stream": True,
    }
    with httpx.stream("POST", url, headers=headers, json=body, timeout=_TIMEOUT) as resp:
        resp.raise_for_status()
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
            delta = chunk.get("choices", [{}])[0].get("delta", {})
            token = delta.get("content", "")
            if token:
                yield token, display


def _iter_google(cfg: dict, api_key: str, model: str, messages: list, display: str):
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
            parts = data.get("candidates", [{}])[0].get("content", {}).get("parts", [])
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


def generar_stream(provider: str, api_key: str, model: str, messages: list):
    """Genera (token, nombre_mostrado) por streaming para el proveedor indicado."""
    cfg = PROVIDERS.get(provider)
    if cfg is None:
        raise ValueError(f"Proveedor no soportado: {provider}")
    if not api_key:
        raise ValueError("Falta la API key del proveedor.")

    model = model or cfg["default_model"]
    display = cfg["name"]

    try:
        if provider == "google":
            yield from _iter_google(cfg, api_key, model, messages, display)
        else:
            yield from _iter_openai_compatible(cfg, api_key, model, messages, display)
    except httpx.HTTPStatusError as e:
        raise ValueError(_error_amigable(e)) from e
    except httpx.RequestError as e:
        raise ValueError(f"No se pudo conectar con {display}: {e}") from e


def generar_completo(provider: str, api_key: str, model: str, messages: list) -> str:
    """Respuesta completa (sin streaming) consumiendo el generador de tokens."""
    return "".join(token for token, _ in generar_stream(provider, api_key, model, messages))
