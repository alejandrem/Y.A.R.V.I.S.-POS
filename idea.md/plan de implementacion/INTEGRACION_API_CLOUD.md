# Integración de APIs de Nube (Gemini + OpenCode Zen) — Guía Técnica

Documento que registra **cómo se hizo funcionar** el chat con proveedores de IA en la nube,
el **bug real** que rompía las respuestas, cómo se resolvió, y los últimos cambios aplicados.

> Aplicación: **Y.A.R.V.I.S. POS** (Tauri + React + sidecar Python FastAPI)

---

## 1. Contexto general

Y.A.R.V.I.S. tiene dos rutas de chat:

| Ruta | Motores | Uso |
|------|---------|-----|
| **Local** | Qwen 0.5B / 0.8B / 1.7B (llama.cpp) | Sin internet, con RAG sobre el catálogo |
| **Nube** | Gemini (Google) + OpenCode Zen | Modelos gratuitos por API, sin RAG |

Los modelos de nube **no leen la base de datos ni el RAG**: reciben un system prompt
corto de asistente de ventas + el historial del chat (ver `prompts_api.py`).

---

## 2. Proveedores configurados

Archivo: `yarvis-IA/chatbot/motor_chat/modelos_API/apis_cloud.py`

### 2.1 Google (Gemini)

- Base: `https://generativelanguage.googleapis.com/v1beta`
- Endpoint de modelos: `GET /v1beta/models`
- Formato de chat: `POST /models/{id}:streamGenerateContent`
- Autenticación: query param `?key=API_KEY`
- Modelo por defecto: `gemini-2.0-flash`

### 2.2 OpenCode (Zen)

- Base: `https://opencode.ai/zen/v1`
- Endpoint de modelos: `GET /zen/v1/models`
- Autenticación: `Authorization: Bearer` (opcional; **funciona sin key**)
- Modelo por defecto actual: `mimo-v2.5-free`
- **Modelos gratuitos**: todos terminan en `-free` (más `big-pickle`). Filtro:

```python
def _es_free(model_id: str) -> bool:
    return model_id.endswith("-free") or model_id in _MODELOS_FREE_EXTRA
```

### 2.3 Enrutado por formato de modelo (Zen)

`_formato_zen(model_id)` decide qué endpoint usar según el prefijo del modelo:

| Prefijo | Formato | Endpoint |
|---------|---------|----------|
| `claude-*`, `qwen3.*` | Anthropic | `/messages` |
| `gemini-*` | Google | `/models/{id}:streamGenerateContent` |
| `gpt-*`, `grok-*`, `o1-`, `o3-`, `o4-` | OpenAI Responses | `/responses` |
| resto | OpenAI Chat | `/chat/completions` |

---

## 3. Arquitectura del flujo (nube)

```
React (ChatWidget.tsx)
  └─ invoke("send_chat_stream", {messages, model, provider, apiKey})
      └─ Rust (chat.rs)
          └─ POST /chat_stream (sidecar Python)
              └─ endpoints.py → generar_stream() → _generar_por_formato()
                  └─ httpx.stream() al proveedor
                      └─ tokens → SSE "data: {...}"
                          └─ Rust emite eventos Tauri
                              └─ React: chat-token / chat-think / chat-usage / chat-complete / chat-error
```

Eventos emitidos por Rust (`chat.rs`) hacia la UI:

| Evento | Contenido | Uso en UI |
|--------|-----------|-----------|
| `chat-token` | `{token, model}` | Respuesta final en streaming |
| `chat-think` | `{token, model}` | Hilo de pensamiento (si hay) |
| `chat-usage` | `{usage}` | Barra de contexto con tokens reales |
| `chat-complete` | `{response, model}` | Finaliza la respuesta |
| `chat-error` | `{error}` | Muestra error rojo |
| `chat-done` | `{model}` | Evento interno (no escuchado en UI, inofensivo) |

---

## 4. Problema #1 — El error 429

### Síntoma
La UI mostraba:
```
Error 429: el proveedor está limitando las peticiones. Espera un minuto y reintenta.
```

### Investigación (lo que se descartó)

1. **React/Vue y bucles de peticiones** — Descartado tras auditar:
   - Todos los `useEffect` tienen dependencias correctas; no hay loops infinitos.
   - No existe ningún `onClick={fn()}` (llamada eager); todos usan arrow functions.
   - El envío está protegido contra doble clic (`if (!msg || isLoading) return`).
   - `React.StrictMode` estaba activo y duplicaba un fetch de arranque (ya eliminado), pero **no** re-dispara handlers de click.
2. **Rate-limiter propio de FastAPI** — No existe middleware de límites en el backend.
3. **Bug en el reintento** — El reintento espera máx 20s y reintenta **una sola vez**.

### Diagnóstico definitivo (prueba con curl)

Desde la terminal se probó directamente a Zen:

```bash
# Listado de modelos: OK (HTTP 200)
curl -s -o /dev/null -w "%{http_code}\n" https://opencode.ai/zen/v1/models

# Chat completion con deepseek-v4-flash-free: FALLA
curl -s -w "\nHTTP %{http_code}\n" -X POST https://opencode.ai/zen/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"deepseek-v4-flash-free","messages":[{"role":"user","content":"di hola"}],"max_tokens":5,"stream":false}'
```

Resultado:

```
HTTP 429
{"type":"error","error":{"type":"FreeUsageLimitError",
"message":"Error from provider (Console): Rate limit exceeded. Please try again later."}}
```

Y probando **todos** los modelos free:

```
deepseek-v4-flash-free   → HTTP 429  FreeUsageLimitError (saturado)
mimo-v2.5-free           → HTTP 200  OK
nemotron-3-ultra-free    → HTTP 200  OK
hy3-free                 → HTTP 200  OK
nemotron-3.5-lightning   → HTTP 200  OK
laguna-s-2.1-free        → HTTP 200  OK
big-pickle               → HTTP 200  OK
```

**Conclusión:** el 429 es **real y del proveedor**. Los modelos `*-free` de Zen son de
cuota compartida mundial; cuando se agota, responden 429 con `FreeUsageLimitError`.
`deepseek-v4-flash-free` estaba saturado globalmente.

### Mitigaciones aplicadas

1. **Modelo por defecto cambiado** a `mimo-v2.5-free` (backend + frontend).
2. **Auto-fallback en 429** (`generar_stream`): si un modelo free está saturado en el
   primer intento, se cambia automáticamente al siguiente modelo free disponible:

```python
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
```

3. **Caché del listado de modelos** (TTL 60s) para no pegarle a los endpoints de
   modelos (Gemini gastaba cuota) en cada apertura del selector.
4. **Quitado el auto-fetch de modelos al arrancar la app** (antes con `React.StrictMode`
   se duplicaba en dev).
5. **Logs de diagnóstico** en `endpoints.py`:

```
[YARVIS-CHAT] Cloud: 3 msgs, 420 chars (~105 tok est)
[YARVIS-CHAT] Usage real del proveedor: 55 prompt + 120 completion = 175 total
```

6. **Historial limitado a los últimos 10 mensajes** (`slice(-10)`) para no acumular tokens.

---

## 5. Problema #2 — EL BUG REAL: `list index out of range`

### Síntoma
1. Envías un mensaje.
2. La respuesta del modelo **sí aparece** (ves tokens escribirse).
3. De pronto **desaparece** y la UI muestra `list index out of range` en rojo.

### Causa raíz

En `_iter_openai_compatible` (y el mismo patrón en los demás formatos) se leía el
primer elemento de `choices`:

```python
# ANTES (vulnerable)
delta = chunk.get("choices", [{}])[0].get("delta", {})
```

El default `[{}]` **solo** se usa cuando la clave `choices` está **ausente**.
Cuando el proveedor manda el **chunk final de uso** con `stream_options.include_usage`,
envía:

```json
{"choices": [], "usage": {"prompt_tokens": ..., "completion_tokens": ..., "total_tokens": ...}}
```

`choices` está presente pero **vacío** → `[][0]` lanza `IndexError: list index out of range`.

### Por qué se veía "la respuesta y luego desaparecía"

1. Los tokens de contenido llegan bien (por eso se escribían).
2. Al final llega el chunk de uso con `choices: []`.
3. `[0]` revienta → la excepción sale del generador.
4. `generate_cloud()` tiene `except Exception` → emite el error por SSE.
5. Rust reenvía `chat-error` → `fail()` en React **limpia el stream** y muestra el error.

En una línea: **la respuesta venía completa pero el chunk de usage mataba el flujo**.

### Solución

```python
# DESPUÉS (robusto): `or [{}]` cubre tanto ausente como vacío
delta = (chunk.get("choices") or [{}])[0].get("delta", {})
```

Se corrigió el mismo patrón en **3 puntos**:

| Archivo | Línea aprox. | Formato |
|---------|-------------|---------|
| `apis_cloud.py` | 155 | OpenAI Chat (`/chat/completions`) |
| `apis_cloud.py` | 281 | Google (`candidates`) |
| `endpoints.py` | 311 | Qwen local |

Verificado con chunks reales:

```python
chunks = [
    {"choices": [], "usage": {"total_tokens": 175}},          # ← el culpable
    {"choices": [{"delta": {"content": "hola"}}]},            # normal
    {},                                                        # sin choices
]
for chunk in chunks:
    delta = (chunk.get("choices") or [{}])[0].get("delta", {})
    # OK, sin IndexError en ningún caso
```

---

## 6. Investigación de "tokens inflados" (descarta RAG)

Se sospechaba que la nube recibía el RAG local y eso inflaba el prompt. **Falso**:

- Nube (`/chat` y `/chat_stream`) usa `construir_mensajes_api()` → prompt de 3 líneas
  sin RAG (`prompts_api.py`).
- Solo los modelos locales (Qwen) usan `construir_mensajes()` de `prompts.py` con RAG.
- `tienda_info` se manda en el body desde Rust pero Python lo ignora para la nube (a propósito).

La estimación `chars / 4` del frontend solo afecta a la **barra de contexto (display)**,
no a lo que se envía ni a lo que el proveedor cobra. Los logs `[YARVIS-CHAT]` con el
`usage` real del proveedor permiten verificar si el prompt_tokens es alto o no.

---

## 7. Últimos cambios aplicados (resumen)

### Backend Python
- `apis_cloud.py`
  - `listar_modelos()` con **caché** (TTL 60s) y filtro de modelos free.
  - **Auto-fallback en 429** a otro modelo free.
  - Default cambiado a `mimo-v2.5-free`.
  - **Fix del IndexError** (`choices`/`candidates` vacíos) con `or [{}]`.
  - **Razonamiento (`reasoning_content` / partes `thought`) enrutado al hilo de
    pensamiento** envolviéndolo con marcadores ` think … response `.
- `endpoints.py`
  - Fix de `choices` vacío en el streaming local (Qwen).
  - Logs de diagnóstico (`[YARVIS-CHAT] Cloud: …` y `Usage real del proveedor: …`).
  - Log de errores del proveedor.

### Frontend React (`ChatWidget.tsx` y `yarvis.tsx`)
- Selector de modelos de nube en el picker (listado dinámico + persistencia en
  `localStorage.yarvis_cloud_model`).
- **Historial limitado** a los últimos 10 mensajes al enviar.
- **Indicador "Escribiendo..." animado** cuando se está generando pero aún no hay
  pensamiento ni respuesta (antes fondo blanco).
- Hilo de pensamiento (THINKING BOX) que ahora también funciona para nube.
- Barra de contexto con uso real (`chat-usage`) o estimación `chars/4`.
- Botón "Actualizar" para refrescar modelos; sin auto-fetch al arrancar.

### Rust (`chat.rs`, `lib.rs`)
- Comando `get_cloud_models` para listar modelos.
- Manejo de eventos `chat-usage` / `chat-done`.
- `send_chat_stream` con `provider`, `api_key` y envío del body completo.

---

## 8. Cómo probar / verificar

1. Cierra la app por completo (Ctrl+C en la terminal).
2. Vuelve a lanzar: `./run.sh`
3. En la terminal, al enviar un mensaje de nube deberías ver:

```
[YARVIS-CHAT] Cloud: 3 msgs, 420 chars (~105 tok est)
[YARVIS-CHAT] Usage real del proveedor: 55 prompt + 120 completion = 175 total
```

4. En la UI:
   - Mientras espera el primer token → animación "Escribiendo...".
   - Si el modelo razona → caja "El modelo está pensando..." con el hilo.
   - La respuesta final se queda (ya no desaparece).
   - Si el modelo free está saturado → en la terminal:
     ```
     [YARVIS] deepseek-v4-flash-free saturado (429), cambiando a mimo-v2.5-free
     ```
     y la respuesta sale igual con otro modelo.

### Prueba rápida del proveedor (sin abrir la app)

```bash
curl -s -w "\nHTTP %{http_code}\n" -X POST https://opencode.ai/zen/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"mimo-v2.5-free","messages":[{"role":"user","content":"di hola"}],"max_tokens":10,"stream":false}'
```

---

## 9. Archivos clave

| Archivo | Rol |
|---------|-----|
| `yarvis-IA/chatbot/motor_chat/modelos_API/apis_cloud.py` | Proveedores, formatos, streaming, usage, fallback, caché |
| `yarvis-IA/chatbot/motor_chat/modelos_API/prompts_api.py` | System prompt corto de nube (sin RAG) |
| `yarvis-IA/chatbot/motor_chat/endpoints.py` | `/chat`, `/chat_stream`, `/cloud_models`, logs |
| `yarvis-app/src-tauri/src/backventanas/backadmin/admintarvis/chat.rs` | `send_chat_stream`, `get_cloud_models`, eventos |
| `yarvis-app/src-tauri/src/lib.rs` | Registro de comandos Tauri |
| `yarvis-app/src/front-admin/ventanas/adminyarvis/ChatWidget.tsx` | Chat, streaming, thinking, barra de contexto, "Escribiendo..." |
| `yarvis-app/src/front-admin/ventanas/adminyarvis/yarvis.tsx` | Selector de modelos de nube, API keys |
| `yarvis-IA/requirements.txt` | `httpx` (cliente HTTP del backend) |

---

## 10. Lecciones aprendidas

1. **Siempre probar al proveedor con `curl`** antes de culpar al frontend. Un `HTTP 429`
   con `FreeUsageLimitError` es la firma de un modelo free saturado, no de un bug propio.
2. **Los streams de OpenAI/Anthropic/Google siempre pueden mandar chunks "vacíos"**
   (uso, pensamiento, finalización). Todo `[0]` sobre listas del proveedor debe ir
   protegido con `or [{}]` o similar.
3. **El chunk de `usage` con `choices: []` es 100% válido** según la spec de streaming:
   no es un error del proveedor, era un error nuestro al procesarlo.
4. **Separar el prompt de nube del RAG local** aísla fallos: si la nube falla, no es
   por el contexto de la tienda.
5. El reinicio completo (matar el proceso y `./run.sh`) es obligatorio al cambiar Python
   o Rust: de lo contrario quedan procesos viejos corriendo.
