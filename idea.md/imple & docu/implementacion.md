# Plan de Implementacion — Y.A.R.V.I.S. POS

> Actualizado 2026-09-06. Convive con refactor-seguridad.md (auditoria senior, migracion a centavos, endurecimiento y tools). Este documento describe el estado real en Rust. Las olas marcadas como completadas estan implementadas y verificadas. Las zonas sin marca son planes pendientes. No se usa RAG; la estrategia de IA es fine-tuning + tools SQL + embeddings propios HashEmbedder.

Bienvenido al mapa de batalla. Para asegurar que Y.A.R.V.I.S. sea robusto, escalable y mantenible, se implemento por fases.

> Regla de oro del codigo
> Ningun archivo .rs, .ts o .tsx debe pasar de ~400 lineas. Al acercarse, se divide en carpeta por tema con re-exports que conservan las rutas (ver detector/, segmentador/, inventory/, metricas/, cortes/, parser_txt/, lector_txt/, procesador/).

---

## Ola 1: La Fundacion de Hierro (Infraestructura y BD) — COMPLETADA

Punto de Venta en Modo Clasico. La caja no depende de la IA.

- Workspace: Tauri v2 + Vite + React + TypeScript, Tailwind CSS.
- Base de datos: SQLite (yarvis.db) con modo WAL. Tablas clasicas (productos, ventas, detalle_ventas, clientes, usuarios, cortes_caja, ventas_diarias, predicciones_futuras, catalogos_importados, gastos_recurrentes, etc.) creadas en src-tauri/src/backventanas/db/db.rs con migraciones versionadas.
- Conexion Rust <-> Interfaz: CRUD y cobro via comandos Tauri (#[tauri::command] en backventanas/), consumidos con invoke() desde React.
- Regla de escritura: Rust es el unico que escribe en SQLite. Dinero en INTEGER centavos via dinero.rs y migracion 0005.

## Ola 2: El Cerebro Asincrono (sin sidecar) — COMPLETADA

La idea original era un sidecar Python con FastAPI. Se descarto y se migro todo a Rust nativo (ver migracion-rust.md).

- Motor de IA en Rust: crate local src-ia (se enlaza por ruta desde yarvis-app/src-tauri/Cargo.toml con feature llm-local).
- Arranque: binario unico. Sin python3 main.py, sin ai_service, sin externalBin.
- Chat cloud: src-ia/motor-chat/cloud (OpenCode Zen / Gemini via reqwest + SSE) con fallback a local y ciclo de tools con 10 herramientas.
- Chat local: Qwen2.5-Coder 1.5B Instruct GGUF fine-tuneado con llama-cpp-4 (src-ia/motor-chat/llm), carga bajo demanda (lazy), ventana 4096, recorte de historial conservador. Opera 100% offline con 10 tools SQL.

## Ola 3: El Parseador y la Ingesta Masiva — COMPLETADA (estadística sin IA)

- Parseador de tickets/catalogos: src-ia/parseador_de_tickets (regex + estadística en cerebro/, lectores en formatos/). Sin LLM en el pipeline.
- Detección de formato: cerebro/analizador_tickets/detector/ (hipótesis A/B + familia C por consistencia de precios + diagnóstico en lenguaje normal).
- Folio/fecha/hora: cerebro/analizador_tickets/segmentador/ (10/10 formatos reales, apertura fuerte, sin tickets fantasma) + fechas validadas con pivote.
- Idempotencia y orden: clave en 3 niveles (folio → AUTO-YYYYMMDD-HHMM → SIN-FOLIO) guardada en ventas.folio_ticket; inserción cronológica de archivos y segmentos.
- Procesamiento por lotes: cerebro/parseador_masivo/ (streaming por canal + modo síncrono; transaccion por archivo con rollback). Verificado: 1000 tickets reales en ~250ms release, re-importación 0 duplicados (example bench1000).
- Vinculacion con inventario: cerebro/vinculador_inventario/ (TF-IDF + fuzzy como interino, sin vectores).
- Comandos: adminparser/ (parser_txt/ por tema: archivos, catalogo, deteccion, lote; parser_csv/excel; get_tickets_total para el historial real).
- Frontend: providers persistentes por proceso en segundo plano (BatchProgressProvider, ChatProvider, CartProvider); montaje condicional por pestaña (solo la activa en el DOM, sin árboles ocultos).
- Búsqueda semántica propia operativa: HashEmbedder 384d (buscar_producto_similar, backfill_embeddings). No se usara all-MiniLM ni ONNX externo.

## Ola 4: El Chatbot y su motor — COMPLETADA (cloud + local con tools)

- Comandos nativos (admintarvis/chat.rs): send_chat_message, send_chat_stream, get_cloud_models, get_model_status, load_chat_model, unload_chat_model, stop_chat_stream, set_local_model_path.
- Tools: 10 herramientas de solo lectura (src-ia/motor-chat/llm/tools/): query_sales, compare_periods, get_top_products, query_inventory, forecast_sales, get_product_info, get_restock_analysis, search_products, list_categories, get_products_by_category. Todas con SQL parametrizado y escape_like. Ejecutor compartido cloud/local, roles enforced en herramientas_rol.rs.
- Dataset fine-tuning: tools_arreglado.jsonl con forma <tool_call>{"name":...,"arguments":...}</tool_call> congelada en TOOLS_LINEA (src-ia/motor-chat/cloud/prompts.rs:28 y src-ia/motor-chat/llm/mod.rs:107).
- Separador de bloques think/response aislado (cloud/think.rs y llm/mod.rs:148).
- Ciclo de tools con MAX_RONDAS_TOOLS=3 y re-inyeccion de resultado como mensaje role tool (ciclo_tools.rs).
- Fallback 429 entre proveedores cloud y degradacion graceful a local.
- Frontend: adminyarvis/ChatWidget.tsx y front-empleado/empleayarvis/yarvis.tsx comparten logica.

Pendiente en IA: fine-tuning final de Qwen2.5-Coder 1.5B Instruct para que genere SQL/tools con mayor precision (intento previo no estable). Predicciones ya no son pendiente (ver ola 5).

## Ola 5: Domo, seguridad, predicciones y produccion — COMPLETADA (parcial)

- Autenticacion: Argon2 para admins y empleados; login por roles (adminconfig/auth.rs); Google OAuth PKCE (google.rs); estado de sesion en AuthState.
- Gestion comercial (admin): inventario (CRUD + importar_catalogo con hash y transaccion), tickets, cortes X/Z, finanzas (gastos recurrentes, alertas, metricas, graficas, export stubs), empleados (metas/bonos, turnos, salario), clientes.
- Predicciones: Holt-Winters triple aditivo operativo (src-ia/predicciones/holt_winters.rs + ventas.rs). Comandos get_predictions (admintickets/tickets.rs:188) y get_predicciones_financieras (adminfinanzas/graficas.rs:195) via spawn_blocking, con validacion de horizonte 1..365 y banda 95%.
- Gestion operativa (empleado): nueva venta con control de stock (UPDATE WHERE stock >= ? + rows_affected), perfil, tickets, cortes, chat.
- Empaquetado: yarvis-app/build.sh -> binario unico + .deb + .rpm + .AppImage. Sin PyInstaller. Requiere fuse2 y NO_STRIP=1 en Arch.
- Primer inicio: PrimerInicio.tsx (alta de admin + tienda + empleado) refactorizado.
- Temas: ThemeProvider light/dark.

Pendiente en produccion:
- Impresion termica ESC/POS y facturacion electronica (XML/PAC): sin implementar (flujo visual preparado).
- CI/CD, empaquetado .exe estable en Windows y bateria de pruebas finales (concurrencia, cortes, SSE, flujos completos).

---

## Orden de ataque de lo pendiente (actualizado)

1. Finalizar todos los modulos funcionales sin bloqueos por IA (ventas, inventario, finanzas, tickets, empleados, clientes, configuracion).
2. Drivers de impresion termica (ESC/POS) y facturacion electronica.
3. Fine-tuning final de Qwen2.5-Coder 1.5B Instruct para tools/SQL y despliegue del GGUF resultante.
4. CI/CD, empaquetado .exe y pruebas finales end-to-end en Windows y Linux.

## Archivos clave

| Componente | Ubicacion |
|---|---|
| Frontend admin | yarvis-app/src/front-admin/ |
| Frontend empleado | yarvis-app/src/front-empleado/ |
| Backend Rust (comandos) | yarvis-app/src-tauri/src/backventanas/ |
| Motor de IA (Rust) | src-ia/ |
| Predicciones | src-ia/predicciones/ |
| Registro de comandos | yarvis-app/src-tauri/src/lib.rs (98 comandos) |
| DB (init + WAL + migraciones) | yarvis-app/src-tauri/src/backventanas/db/db.rs |
| Conversion monetaria | yarvis-app/src-tauri/src/dinero.rs |
| Build de produccion | yarvis-app/build.sh |
