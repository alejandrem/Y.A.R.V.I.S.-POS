# Auditoria, Refactors y Endurecimiento — Sesion 2026-08-24

> Registro maestro de la sesion de auditoria nivel senior + fixes. Complementa a bugs-resueltos.md (bugs puntuales) con la vista de arquitectura: que se cambio, por que y que queda pendiente. Actualizado 2026-08-26 para reflejar que las predicciones ya estan operativas y que no se usara RAG.

---

## 1. Diagnostico general de la auditoria

Auditoria completa de backend Rust, frontend React y crate src-ia. Veredicto: base solida para un proyecto personal (migraciones versionadas, Argon2id, transaccion en el flujo critico de cobro, tests reales, cero secrets en git). Los problemas graves encontrados caian en 4 familias:

1. Operaciones multi-paso no atomicas + errores tragados en silencio.
2. Dinero en f64 en todo el esquema y codigo.
3. Frontend sin capa de servicio + duplicacion masiva admin/empleado.
4. Seguridad: SQL interpolado, roles solo por prompt, secrets en localStorage.

---

## 2. Fixes de atomicidad y silencio (fase 1)

| Fix | Archivo | Que cambio |
|---|---|---|
| Stock negativo imposible | new_venta.rs | UPDATE ... WHERE stock >= ? + chequeo de rows_affected; la venta completa revierte si un item no alcanza |
| Importacion de tickets atomica | admintickets/tickets.rs | Transaccion todo-o-nada; los let _ = eliminados; items sin coincidencia se reportan ("3/5 items vinculados") |
| Importacion de catalogos atomica | admininventory/inventory.rs | Una tx para todo el lote; el registro del hash va dentro de la misma tx; errores de INSERT ya no se tragan |
| Pago de gasto atomico | adminfinanzas/gastos.rs | INSERT del pago + UPDATE del acumulado en una sola tx |
| Helpers testeables | tickets.rs, gastos.rs | Extraccion guardar_ticket_parseado_impl / patron _impl consistente |

Tests nuevos: stock insuficiente, rollback multi-item, vinculacion case-insensitive, reporte de items sin vincular, atomicidad ante item corrupto.

## 3. Migracion f64 -> centavos (fase 2)

Regla de oro nueva: toda columna monetaria es INTEGER en centavos en SQLite. Las cantidades (stock, cantidad) y porcentajes siguen en REAL.

- Migracion 0005_moneda_centavos.sql: reconstruye las 12 tablas con columnas INTEGER, convirtiendo datos historicos x100 con redondeo al centavo.
- Modulo src/dinero.rs: a_centavos() / a_pesos() — toda conversion pasa por ahi. El contrato IPC sigue hablando pesos (f64) -> frontend intacto.
- Backend completo migrado: ventas, tickets, inventario, cortes (cierre con aritmetica exacta en enteros), gastos, metricas (incluida la cache materializada), graficas, alertas, empleados, metas, auth, perfil.
- src-ia migrado: las tools devuelven pesos al LLM; Holt-Winters recibe la serie en pesos; el parser masivo escribe centavos.

Leccion dura: sqlx-sqlite y PRAGMA foreign_keys. sqlx-sqlite envuelve siempre cada migracion en una transaccion e ignora el marcador -- no-transaction. Dentro de una transaccion SQLite ignora PRAGMA foreign_keys, asi que los DROP TABLE de la reconstruccion fallaban con datos reales (FKs activas) pero pasaban con DBs vacias de test.

Solucion definitiva en db.rs (y en el fixture de tests): conexion en dos fases — Fase 1 migra con foreign_keys(false) desde las opciones de conexion, Fase 2 reabre con foreign_keys(true) para operacion normal.

Leccion meta: las rutas de migracion deben probarse con datos reales; los fixtures frescos no cubren ese camino. Herramienta de diagnostico: cargo run --example diag_migracion -- <ruta-db-copia>.

## 4. Chat cloud (fase 3)

| Problema | Causa raiz | Fix |
|---|---|---|
| Gemini nunca funcionaba (404) | Google retiro gemini-2.x para keys nuevas | default -> gemini-3.6-flash (variables.rs + ChatWidget.tsx) |
| Streams largos morian a los 120 s | Client::timeout aplica al ciclo completo peticion+cuerpo | read_timeout(90s): timeout de inactividad entre chunks; un stream vivo nunca se corta |
| max_tokens rompia Gemini | 39800 > techo de salida flash | MAX_TOKENS_GOOGLE = 8192 separado |
| Contexto local sin truncamiento | Historial largo excedia 4096 | recortar_historial en src-ia/motor-chat/llm/mod.rs:42 (presupuesto por caracteres, conserva system + recientes) |

## 5. Seguridad (fase 4)

1. SQL de tools parametrizado (tools/mod.rs): cero interpolacion; helper escape_like() + ESCAPE '\' para que %/_ del input del LLM no alteren patrones. LIMIT tambien parametrizado.
2. API keys fuera de localStorage: nuevo modulo api_config.rs con comandos leer_api_keys / guardar_api_keys; JSON en app_data_dir/api_keys.json con permisos 0600. El frontend usa cache en memoria alimentada por el backend (ChatWidget.tsx: refrescarApiKeysCache / setApiKeysCache). Escalamiento futuro: OS keychain via crate keyring sin cambiar contratos.
3. CSP activa (tauri.conf.json): antes "csp": null renderizando output de LLMs; ahora politica restrictiva con nonces automaticos de Tauri v2.
4. Roles enforced en ejecucion de tools (chat.rs + herramientas_rol.rs): TOOLS_SOLO_ADMIN = [query_sales, compare_periods, get_restock_analysis]; guard en el punto de ejecucion hilado por las 5 rutas (cloud msg/stream, local msg/stream). El prompt es sugerencia; esto es control de acceso real.
5. Sin fallback plaintext en verify_password (auth.rs): un hash que no parsea como Argon2id deniega acceso siempre. Test de regresion incluido.

## 6. Tools de navegacion de inventario para cloud (fase 5)

3 tools nuevas de solo lectura en el ejecutor compartido (tools/mod.rs):

| Tool | Args | Que hace |
|---|---|---|
| search_products | query (obligatorio), limit | Busqueda parcial por nombre, ordenada por vendido, precio en pesos |
| list_categories | — | Categorias con conteo de productos y stock total |
| get_products_by_category | category opcional, limit | Hojear categoria (case-insensitive) o catalogo completo |

- Documentadas en el prompt cloud (prompts.rs, const TOOLS_EXTRAS) con estrategia recomendada: list -> browse -> search. TOOLS_LINEA y TOOLS_INSTRUCCIONES (formato del fine-tuning) quedaron intactos — verificado por test prompts_cloud_documentan_tools_de_navegacion.
- Decision de diseno: protocolo textual <tool_call> compartido por local y cloud (un ejecutor, un formato), no function calling nativo por proveedor. Escritura desde tools: descartada deliberadamente (riesgo).

## 7. Predicciones Holt-Winters (fase 6 — 2026-08-26)

Implementacion sin dependencias en src-ia/predicciones/:

- holt_winters.rs: suavizado triple aditivo (nivel L_t, tendencia T_t, estacional S_t), periodo 7 (semanal), grid 343 combos alpha/beta/gamma en [0.05, 0.1, 0.2, 0.4, 0.6, 0.8, 0.99], seleccion por minimo SSE one-step-ahead, banda 95% z=1.96 * s * sqrt(k), recorte a >=0.
- ventas.rs: lee ventas completadas agrupadas por dia (SUM total en centavos /100 -> pesos), densifica huecos con 0, valida horizonte 1..365 y minimo 4 dias, genera fechas YYYY-MM-DD consecutivas.
- Comandos Tauri: get_predictions (admintickets/tickets.rs:188) y get_predicciones_financieras (adminfinanzas/graficas.rs:195) hacen spawn_blocking(predecir_ventas) y retornan {data: [{fecha, prediccion, minimo, maximo}]}.
- Sin Prophet, sin augurs, sin servicios externos.

## 8. Empaquetado

yarvis-app/build.sh reemplaza a npm run tauri build directo. Exporta LD_LIBRARY_PATH (para que linuxdeploy empaquete libllama.so.0 dentro del AppImage) y NO_STRIP=1 (el strip viejo de linuxdeploy no entiende .relr.dyn de Arch). Requiere fuse2 y /usr/lib/gdk-pixbuf-2.0/2.10.0/loaders (ver bugs-resueltos.md Bug B1).

## 9. Pendiente (backlog priorizado — actualizado 2026-08-26)

1. Unwraps sobre input del front (metricas.rs, alertas.rs) — crashes por fecha malformada.
2. Capa de servicios en frontend + eliminar duplicacion residual del chat.
3. Errores visibles al usuario (41 console.error + 20 alert() dispersos).
4. cerrar_corte sin cablear en la UI (backend ya recalcula server-side).
5. OS keychain para API keys, rate limiting de login, TOCTOU en setup.
6. Tests: permisos por rol e2e, race de stock mismo producto, red/SSE.
7. Fine-tuning de Qwen2.5-Coder 1.5B Instruct para tools/SQL (work in progress) y despliegue del GGUF.
8. Modelo de embeddings propio para busqueda semantica (TF-IDF interino en similitud.rs).
9. Drivers ESC/POS y facturacion electronica, y CI/CD + empaquetado .exe final.

Notas cerradas: contexto local ya tiene recorte (recortar_historial), predicciones ya operativas, monolitos empleados.tsx/nueva_venta.tsx/perfil.tsx ya subdivididos bajo 650 lineas (ver git log 30fc449, 284de92).
