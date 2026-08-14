# PLAN DE IMPLEMENTACIÓN Y.A.R.V.I.S. V2 — HÍBRIDO

> Objetivo: **Qwen 0.5B local por defecto, OpenCode API con tools cuando hay internet.**
> El modelo no toca SQL: solo llama a `search_inventory()`, que es la ÚNICA función que lee la DB.

---

## FASE 1 — Base actual (lo que ya tienes, no tocar)

- ✅ `sqlite-vec` con `buscar_semantico(query, top_k)` en `modelos_local/motor_rag.py` (salvavidas offline).
- ✅ Separación de carpetas: `modelos_local/` (Qwen + RAG) y `modelos_API/` (nube sin RAG).
- ✅ `chatbot/embeddings/modelo.py` con `texto_a_embedding()` compartido.
- ✅ OpenCode Zen funcionando (~80%): streaming, usage, auto-fallback 429.

## FASE 2 — Capa de abstracción de Tools (HOY)

**Archivo nuevo:** `yarvis-IA/chatbot/motor_chat/modelos_local/herramientas.py`

```python
def search_inventory(query: str, limit: int = 5) -> list[dict]:
    # 1. Búsqueda semántica primero (sqlite-vec)
    results = buscar_semantico(query, top_k=limit)
    # 2. Si no encuentra nada → fallback a LIKE
    if not results:
        results = ... "SELECT * FROM productos WHERE nombre LIKE ?"
    return [{"nombre", "precio", "stock", ...}]
```

- Esta función la usan **AMBOS modos** (local y nube). El modelo nunca hace SQL.
- Se registra un `TOOLS_SCHEMA` (formato OpenAI function-calling) para la nube.

## FASE 3 — Modo Local — RAG determinístico (ya lo tienes)

```python
def handle_local(query):
    contexto = search_inventory(query)          # TÚ buscas, no el modelo
    prompt = f"Contexto: {contexto}\nPregunta: {query}\nResponde usando solo el contexto."
    return qwen_0_5b.generate(prompt)           # 1 sola llamada
```

Costo: 0 · Latencia: ~3s · Internet: no necesaria

## FASE 4 — Modo Nube — OpenCode + Function Calling (HOY)

En `modelos_API/apis_cloud.py`:

```python
tools = [{
  "type": "function",
  "function": {
    "name": "search_inventory",
    "description": "Busca productos en el inventario por nombre o para qué sirve",
    "parameters": {
      "type": "object",
      "properties": {"query": {"type": "string"}, "limit": {"type": "integer"}},
      "required": ["query"]
    }
  }
}]

def handle_cloud(query):
    response = client.chat.completions.create(model="...", messages=[...], tools=tools, tool_choice="auto")
    if response.choices[0].message.tool_calls:
        args = json.loads(response.choices[0].message.tool_calls[0].function.arguments)
        resultados_db = search_inventory(args["query"])          # TU función local
        response2 = client.chat.completions.create(              # 2ª llamada con resultados
            model="...", messages=[..., response.choices[0].message,
            {"role": "tool", "tool_call_id": "...", "content": str(resultados_db)}])
        return response2.choices[0].message.content
```

Costo: ~$0.001/pregunta · Latencia: ~1s · Internet: sí

## FASE 5 — El Switch Automático (HOY)

En `modelos_API/apis_cloud.py` (o `config.py`):

```python
def get_answer(query):
    if has_internet() and config["provider"] == "opencode":
        try:
            return handle_cloud(query)
        except:
            return handle_local(query)     # fallback si falla la nube
    return handle_local(query)
```

---

## Orden para picar hoy

1. **Fase 2**: encapsular `sqlite-vec` en `search_inventory()`.
2. **Fase 4**: OpenCode con `big-pickle`/`mimo-v2.5-free` y 1 tool. Probarlo con curl/TestClient.
3. **Fase 5**: el switch automático con fallback a local.

---

## Archivos

| Archivo | Rol |
|---------|-----|
| `yarvis-IA/chatbot/motor_chat/modelos_local/herramientas.py` | `search_inventory()` + `TOOLS_SCHEMA` (NUEVO) |
| `yarvis-IA/chatbot/motor_chat/modelos_API/apis_cloud.py` | function calling + fallback 429 (MODIFICADO) |
| `yarvis-IA/chatbot/motor_chat/endpoints.py` | orquesta tools en `/chat` y `/chat_stream` (MODIFICADO) |
| `yarvis-IA/chatbot/motor_chat/modelos_local/motor_rag.py` | `buscar_semantico()` (sin cambios) |
