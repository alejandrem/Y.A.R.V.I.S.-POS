# Migracion Python -> Rust (Y.A.R.V.I.S.)

> Estrategia: Strangler Fig. Codigo Rust corre en paralelo al sidecar Python. Python no se elimina hasta que el ultimo endpoint tenga equivalente en Rust verificado. Criterio de exito de cada fase: endpoint equivalente + tests + misma salida JSON.
>
> Estado: COMPLETADA. El sidecar Python se retiro (sidecar.rs eliminado), no queda rastro de Python en el codigo (arranque = binario unico de Tauri + src-ia).

---

## Inventario del motor Python (20 endpoints)

| Modulo | Endpoints | Dificultad | Dependencia IA pesada |
|---|---|---|---|
| Chat | /chat, /chat_stream, /load_model, /stop, /unload_model, /model_status | Alta | llama-cpp (local), APIs HTTP (cloud -> Rust) |
| Parseador | /analizar_ticket, /parsear_con_mapeo, /parsear_catalogo_visual, /parsear_excel, /parsear_carpeta, /parsear_carpeta_stream, /vincular_inventario, /guardar_vinculacion | Alta | llama-cpp (Qwen2.5-Coder 1.5B Instruct) |
| Profeta | /recalcular_predicciones | Media | Prophet -> Holt-Winters propio |
| Embeddings/RAG | /generar_embedding, /buscar_similar, /backfill, /insertar_knowledge | Baja | sentence-transformers -> modelo propio (no all-MiniLM) |

Base comun reimplementada 1 sola vez (crate src-ia):
- SQLite (sqlx), agregaciones de ventas, consultas admin.
- SSE streaming helpers, normalizacion de mensajes, cola de fallback de modelos cloud.
- Manager de modelos locales (llama-cpp-4), liberacion de RAM -> port de gestion_hardware.py.

---

## Fase 1 — Fundamentos (sin IA)

Objetivo: reemplazar la capa de datos y HTTP con un crate Rust que el sidecar Python aun puede ignorar.

1. Crate src-ia con sqlx: mover queries de consultas_db.py a sqlx (ventas, productos, cajeros, margenes, top sellers, KPIs).
2. Module stream: helper SSE tipado (enum SseEvent).
3. Tests unitarios por query contra fixture SQLite. DoD: mismas rows que Python.

## Fase 2 — Chat cloud (HTTP puro)

Objetivo: eliminar la dependencia de Python para el modo cloud (90% del uso diario).

1. Module chat/cloud en Rust: reqwest + Streaming SSE, port completo de apis_cloud.py (OpenCode Zen + Gemini): cola de fallback 429, separador think/response, ciclo de tools.
2. Tauri command chat_stream en Rust; frontend cambia fetch a tauri::invoke.
3. Port de _separar_think/limpiar_think — regex + cola de marcadores.
4. DoD: misma salida SSE token a token que Python.

> Hecho 2026: src-ia/motor-chat/cloud porta apis_cloud.py, prompts_api.py y variables.py. Los comandos Tauri send_chat_message, send_chat_stream y get_cloud_models atienden el modo cloud con Rust (con fallback a local), y el paquete yarvis-IA/chatbot/motor_chat/modelos_API/ se elimino por completo. No se usa RAG.

## Fase 3 — Profeta (reemplaza Prophet sin deps)

1. Module predicciones en Rust: Holt-Winters aditivo (estacionalidad semanal, src-ia/predicciones/holt_winters.rs).
2. Lee historial con query agregada (GROUP BY date, misma logica de predictor.py:14) en src-ia/predicciones/ventas.rs.
3. Salida {prediccion, minimo, maximo, fecha} con banda 95% derivada de error estandar one-step-ahead.
4. Comandos Tauri get_predictions (admintickets/tickets.rs:188) y get_predicciones_financieras (adminfinanzas/graficas.rs:195) via spawn_blocking. DoD: contra datos reales, banda coherente y horizonte 1..365 validado.

> Hecho 2026-08-26: Holt-Winters operativo con estacionalidad m=7, grid 343 combos y recorte a >=0. Ver src-ia/predicciones/.

## Fase 4 — Busqueda semantica (modelo de embeddings propio)

Objetivo original era all-MiniLM-L6-v2 + ort/fastembed. Descartado por decision de arquitectura: se construira modelo de embeddings propio, entrenado para el dominio de productos de tienda (nombres, categorias, variantes). No se usara all-MiniLM ni ONNX externo.

1. Modelo propio: entrenamiento y pipeline local pendiente (despues de finalizar modulos funcionales).
2. Port de /generar_embedding, /buscar_similar (coseno), /backfill, /insertar_knowledge con el modelo propio.
3. DoD: mismo ranking semantico que con el modelo previo para textos de prueba del dominio; buscar_producto_similar y backfill_embeddings dejan de ser stubs.

Interino: vinculador_inventario/similitud.rs usa TF-IDF + fuzzy sin vectores.

## Fase 5 — Parseador de tickets (reglas primero, LLM despues)

1. Port de toda la logica de reglas (regex, sin modelos): _extraer_fecha_hora_regex, _PATRONES_PAGO, _es_linea_util (niveles 1/2/3), _parsear_linea, parsear_catalogo_visual, lectores TXT/CSV/Excel (calamine), filtrador, vinculador, lote.
   - DoD: mismos items/errores que Python con los tickets de prueba reales.
2. LLM local del parseo: llama-cpp-4 cargando Qwen2.5-Coder 1.5B Instruct GGUF fine-tuneado (unico modelo local, compartido con chat). Pipeline llama.cpp con generar_bajo_lock. Funciona offline.
3. Endpoints /analizar_ticket, /parsear_carpeta, /parsear_carpeta_stream. DoD: confianza >= 0.8 en los mismos tickets de prueba.

## Fase 6 — Chat hibrido/local + cierre

1. Port del chat hibrido con modelo local (prompts, cache, motor_chat endpoints local) + 10 tools de solo lectura.
2. /model_status, /load_model, /unload_model, /stop en Rust (manager de RAM con llama-cpp-4, unico modelo para parseo y chat).
3. Retirar Python: cuando los endpoints pasen la suite de equivalencia:
   - Apagar sidecar (sidecar.rs), quitar yarvis-IA/ del repo.
   - Frontend: todos los fetch a http://127.0.0.1:{port}/... -> tauri::invoke/comandos nativos.
   - Eliminar find_free_port/check_process_alive; arranque = binario unico.

> Hecho 2026-08: cierre completado. El chat local usa Qwen2.5-Coder 1.5B Instruct nativo (src-ia/motor-chat/llm con llama.cpp) y el cloud corre en Rust (src-ia/motor-chat/cloud), con fallback local. get_model_status, load_chat_model, unload_chat_model, stop_chat_stream y set_local_model_path son comandos nativos. El parseo es 100% Rust/llama.cpp. sidecar.rs, ai.rs y get_ai_status se eliminaron. yarvis-IA/ no existe en el repo. run.sh/run.bat/reset.sh no mencionan Python. Quedan como stubs solo buscar_producto_similar, backfill_embeddings y los exports; las predicciones ya estan operativas.

---

## Orden recomendado de ataque (actualizado)

1 -> 2 -> 5 (parseador) -> 3 (predicciones, ya hecho) -> 4 (embeddings propio) -> 6 (cierre, ya hecho). Siguiente: finalizar modulos -> embeddings propio -> drivers -> fine-tuning 1.5B Coder.

## Riesgos (actualizados)

- Fine-tuning de Qwen2.5-Coder 1.5B Instruct: estabilizar dataset tools_arreglado.jsonl y pipeline de entrenamiento antes de desplegar el GGUF.
- Entrenamiento del modelo de embeddings propio: requiere corpus de productos etiquetados y evaluacion de ranking semantico.
- Drivers ESC/POS y facturacion: dependencia de hardware y PAC; validar en Windows real.

## Fuera de alcance por ahora

- Tocar yarvis.db mas alla de migraciones versionadas con conversion a centavos.
- CI/CD y empaquetado .exe final (siguiente fase despues de embeddings y drivers).
