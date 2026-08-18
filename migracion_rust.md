# Migración Python → Rust (Y.A.R.V.I.S.)

> Estrategia: **Strangler Fig**. Código Rust corre en paralelo al sidecar Python.
> Python NO se elimina hasta que el último endpoint tenga equivalente en Rust verificado.
> Criterio de éxito de cada fase: endpoint equivalente + tests + misma salida JSON.
>
> **ESTADO: COMPLETADA.** El sidecar Python se retiró (`sidecar.rs` eliminado), no queda
> rastro de Python en el código (arranque = binario único de Tauri + `src-ia`).

---

## Inventario del motor Python (20 endpoints)

| Módulo | Endpoints | Dificultad | Dependencia IA pesada |
|---|---|---|---|
| Chat | `/chat`, `/chat_stream`, `/load_model`, `/stop`, `/unload_model`, `/model_status` | **Alta** | llama-cpp (local), APIs HTTP (cloud → Rust) |
| Parseador | `/analizar_ticket`, `/parsear_con_mapeo`, `/parsear_catalogo_visual`, `/parsear_excel`, `/parsear_carpeta`, `/parsear_carpeta_stream`, `/vincular_inventario`, `/guardar_vinculacion` | **Alta** | llama-cpp (Qwen 0.5/0.8/1.7B) |
| Profeta | `/recalcular_predicciones` | **Media** | Prophet → Holt-Winters (propia) |
| Embeddings/RAG | `/generar_embedding`, `/buscar_similar`, `/backfill`, `/insertar_knowledge` | **Baja** | sentence-transformers → ONNX/fastembed |

Base común que se reimplementa 1 sola vez (crate `yarvis-engine`):
- SQLite (`sqlx`), agregaciones de ventas, consultas admin (mismas queries de `consultas_db.py`).
- SSE streaming helpers, normalización de mensajes, cola de fallback de modelos cloud.
- Manager de modelos locales (`llama-cpp-rs`), liberación de VRAM → port de `gestion_hardware.py`.

---

## FASE 1 — Fundamentos (sin IA)
**Objetivo:** reemplazar la capa de datos y HTTP con un crate Rust que el sidecar Python aún puede ignorar.

1. Crate `yarvis-engine` (en `src-tauri/src/engine/`): mueve TODAS las queries de `consultas_db.py` a `sqlx` (ventas, productos, cajeros, márgenes, top sellers, KPIs). Mismo SQL, mismo orden de columnas.
2. Module `stream`: helper SSE (`data: {...}\n\n`, `[DONE]`, `event:`), como `_iter_delta_lineas` pero tipado (`enum SseEvent`).
3. Tests unitarios por query contra fixture SQLite. **DoD:** mismas rows que Python (diff por JSON).

## FASE 2 — Chat cloud (HTTP puro)
**Objetivo:** eliminar la dependencia de Python para el modo cloud (el 90% del uso diario).

1. Module `chat/cloud` en Rust: `reqwest` + Streaming SSE, port completo de `apis_cloud.py` (OpenCode Zen + Gemini):
   - `_normalizar_mensajes`, `_iter_delta_lineas` (lee `reasoning_content` y `delta.content`).
   - `_iter_google` (formato `streamGenerateContent`, partes `thought`).
   - Cola de fallback 429 (relevo de `variables.py`, límite 3 modelos, espera 2–4 s), `max_tokens` 39800.
   - Tools: port de `herramientas.py` + `ejecutar_tool` (search_inventory, get_sales_history, get_top_sellers).
2. Tauri command `chat_stream` en Rust; frontend cambia `base_url` del fetch de `chat_stream` al `tauri::invoke`.
3. Port de `_separar_think`/`limpiar_think` (`prompts.py`) — regex + cola de marcadores parciales.
4. **DoD:** misma salida SSE token a token que Python (comparación con captura real guardada en `/tmp`).

> ✅ HECHO (2026): `src-ia/motor-chat/cloud` porta `apis_cloud.py`, `prompts_api.py` y
> `variables.py`. Los comandos Tauri `send_chat_message`, `send_chat_stream` y
> `get_cloud_models` atienden el modo cloud con Rust (con fallback a local), y el
> paquete `yarvis-IA/chatbot/motor_chat/modelos_API/` se eliminó por completo.

## FASE 3 — Profeta (reemplaza Prophet sin deps)
1. Module `profeta` en Rust: Holt-Winters (estacionalidad semanal, ~150 líneas, sin deps).
2. Lee historial con la query agregada de Fase 1 (`strftime` GROUP BY — misma de `predictor.py:14`).
3. Salida `{prediccion, minimo, maximo, fecha}` idéntica a `run_prediction` (intervalos derivados de desviación de residuos).
4. Endpoint Tauri command `recalcular_predicciones`. **DoD:** contra datos históricos reales, ±10% del Prophet actual en `yhat` medio.
   > Dejar `augurs` (prophet-wasmstan) documentado como upgrade futuro si se necesitan festivos/regresores.

## FASE 4 — Embeddings/RAG (ONNX)
1. Modelo `all-MiniLM-L6-v2` en ONNX + `ort` (ONNX Runtime) → `fastembed-rs` si prefiere crate alto nivel. Blob 384 floats (`embedding_a_blob` → `.pack`/`bytes`).
2. Port de `/generar_embedding`, `/buscar_similar` (coseno ≈ dot/size), `/backfill`, `/insertar_knowledge`.
3. **DoD:** mismo vector para el mismo texto (validar con texto de prueba fijo, cosine ≈ 1 contra Python).

## FASE 5 — Parseador de tickets (reglas primero, LLM después)
1. Port de **toda la lógica de reglas** (regex, sin modelos): `_extraer_fecha_hora_regex`, `_PATRONES_PAGO`, `_es_linea_util` (niveles 1/2/3), `_parsear_linea`, `parsear_catalogo_visual`, lectores TXT/CSV/Excel (`calamine`), `filtrador`, `vinculador`, `lote`.
   - **DoD:** mismos items/errores que Python con los tickets de prueba reales.
2. LLM local del parseo: `llama-cpp-rs` cargando Qwen 0.5B → 0.8B → 1.7B (misma escalada de confianza de `analizador_llm.py`).
   - Usar el mismo `LlamaVocabulary`/pipeline C de llama.cpp; `create_chat_completion` -> `ChatCompletionRequest`.
3. Endpoints `/analizar_ticket`, `/parsear_carpeta`, `/parsear_carpeta_stream`. **DoD:** confianza ≥ 0.8 en los mismos tickets de prueba.

## FASE 6 — Chat híbrido/local + cierre
1. Port del chat híbrido con modelos locales (`motor_rag.py`, `prompts.py`, `cache.py`, `motor_chat` endpoints local).
2. `/model_status`, `/load_model`, `/unload_model`, `/stop` en Rust (manager de VRAM con `llama-cpp-rs`, swap 0.5/0.8/1.7 + modelo de chat).
3. **Retirar Python**: cuando los 20 endpoints pasen la suite de equivalencia:
   - Apagar sidecar (`sidecar.rs`), quitar `yarvis-IA/` del repo, borrar binario Python del build.
   - Frontend: todos los fetch a `http://127.0.0.1:{port}/...` → `tauri::invoke`/comandos nativos.
   - Eliminar `find_free_port`/`check_process_alive`; arranque = binario único.

> ✅ **HECHO (2026-Ago):** cierre completado.
> - El chat local usa Qwen 3 1.7B nativo (`src-ia/motor-chat/llm` con llama.cpp) y el cloud
>   corre en Rust (`src-ia/motor-chat/cloud`), con fallback local. `get_model_status`,
>   `load_chat_model`, `unload_chat_model`, `stop_chat_stream` son comandos nativos.
> - El parseo de tickets es 100% Rust/llama.cpp (Qwen 0.5B → 1.7B) en `src-ia`.
> - `sidecar.rs`, `ai.rs` y `get_ai_status` se ELIMINARON. El arranque de Tauri ya no lanza
>   Python (`YARVIS_PYTHON` desapareció) ni hace backfill al iniciar: binario único.
> - `yarvis-IA/` no existe en el repo. `run.sh`/`run.bat`/`reset.sh` no mencionan Python.
> - Rutas que dependían del motor Python quedaron como stubs con error claro:
>   `buscar_producto_similar`, `backfill_embeddings`, `get_predictions`, `get_predicciones_financieras`
>   (embeddings/RAG y Prophet quedan pendientes de reimplementación nativa).

---

## Orden recomendado de ataque
1 → 2 → 4 (rápidas, dan victorias tempranas) → 3 (fácil) → 5 (la larga) → 6 (cierre).

## Riesgos
- `augurs`/ONNX en toolchain Tauri (compilación): validar en spike de Fase 1.
- Esfuerzo en Fase 5 paso 2 (bindings llama-cpp): mitigar dejando reglas funcionando primero.
- Página Kanban del frontend con streaming (`/chat_stream`): probar timing SSE en Fase 2.

## Hecho fuera de alcance por ahora
- NO tocar `yarvis.db` (schema compartido; solo lecturas desde el crate).
- Reimplementar embeddings/RAG y Prophet en Rust (ONNX/fastembed y Holt-Winters) para
  reactivar `buscar_producto_similar`/`backfill_embeddings`/`get_predictions`/`get_predicciones_financieras`.