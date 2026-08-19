# Plan de Implementación Riguroso - Y.A.R.V.I.S. POS 🚀

> ⚠️ **ACTUALIZADO (2026-Ago):** este documento ya no es un "mockup a futuro de Python". Es el **estado real** de la implementación en Rust. Las olas marcadas ✅ están **implementadas** y verificadas. Las zonas sin ✅ son planes pendientes.

Bienvenido al mapa de batalla. Este no es un proyecto de fin de semana, es una obra de ingeniería. Para asegurar que Y.A.R.V.I.S. sea un software robusto, escalable y mantenible a lo largo de los años, se implementó por fases.

> ⚠️ **REGLA DE ORO DEL CÓDIGO** ⚠️
> **Ningún archivo (ya sea `.rs`, `.ts` o `.tsx`) deberá pasar de las 600 a 650 líneas de código.**
> Si un archivo llega a las 650 líneas, detente, crea un archivo nuevo y divide la lógica a la mitad (modularización). Archivos enormes = deuda técnica = imposible rastrear bugs. ¡Mantenlo modular!

---

## ✅ Ola 1: La Fundación de Hierro (Infraestructura y BD) — COMPLETADA

Punto de Venta funcionando en "Modo Clásico" (caja registradora normal). La caja no depende de la IA.

- **Workspace**: Tauri v2 + Vite + React + TypeScript, Tailwind CSS.
- **Base de datos híbrida**: SQLite (`yarvis.db`) con **modo WAL**. Tablas clásicas (`productos`, `ventas`, `detalle_ventas`, `clientes`, `usuarios`, `cortes_caja`, `ventas_diarias`, `predicciones_futuras`) creadas en `src-tauri/src/backventanas/db/db.rs`.
- **Conexión Rust ↔ Interfaz**: CRUD y cobro vía **comandos Tauri** (`#[tauri::command]` en `backventanas/`), consumidos con `invoke()` desde React.
- **Regla de escritura**: Rust es el ÚNICO que escribe en SQLite.

## ✅ Ola 2: El Cerebro Asíncrono (el Jefe ya no llama al Empleado: no hay sidecar) — COMPLETADA

La idea original era un motor Python tipo sidecar con FastAPI en puertos libres. **Se descartó y se migró todo a Rust nativo** (ver `migracion_rust.md`).

- **Motor de IA en Rust**: crate local `src-ia` (se enlaza por ruta desde `yarvis-app/src-tauri/Cargo.toml` con la feature `llm-local`).
- **Arranque**: binario único. Sin `python3 main.py`, sin `find_free_port`, sin `ai_service.exe`, sin `LD_LIBRARY_PATH`.
- **Chat cloud**: `src-ia/motor-chat/cloud` (OpenCode Zen / Gemini vía `reqwest` + SSE) con **fallback a local**.
- **Chat local**: Qwen 3 1.7B GGUF con `llama-cpp-4` (`src-ia/motor-chat/llm`), carga bajo demanda (lazy).
- **Lightweight**: el sidecar Python se eliminó; no queda rastro en `.gitignore` (solo patrones fantasma), `run.sh`/`run.bat` ni `tauri.conf.json` (`externalBin` vacío).

## ✅ Ola 3: El Parseador y la Ingesta Masiva — COMPLETADA (reglas en Rust)

- **Parseador de tickets/catálogos**: `src-ia/parseador_de_tickets` (regex + reglas en `cerebro/`, lectores en `formatos/`).
- **Procesamiento por lotes**: `cerebro/parseador_masivo/` (eventos SSE al frontend; transacción por archivo con rollback).
- **Vinculación con inventario**: `cerebro/vinculador_inventario/` (similitud + persistencia).
- **Análisis LLM**: `rutas/` → `analizar_ticket` con Qwen local bajo demanda + detección de modelos GGUF.
- **Comandos**: `adminparser/parser_*.rs` (`parsear_catalogo_visual`, `parsear_carpeta_stream`, `analizar_ticket_con_ia`, `parsear_con_mapeo`, `parsear_catalogo_csv`, `parsear_excel`, `vincular_inventario`, `descargar_modelos`, ...).
- **Frontend**: `parseadodetickets/` (BatchProcessor, ColumnMapper, CatalogosParseados) integrado en el **Módulo de Importación Inteligente** (`adminconfig/components/importmodule/`).

### ⏳ Pendiente en parseador
- Embeddings/RAG para **búsqueda semántica** de productos (`buscar_producto_similar`, `backfill_embeddings` son stubs).

## ✅ Ola 4: El Chatbot y su motor — COMPLETADA (cloud + local)

- **Comandos nativos** (`admintarvis/chat.rs`): `send_chat_message`, `send_chat_stream`, `get_cloud_models`, `get_model_status`, `load_chat_model`, `unload_chat_model`, `stop_chat_stream`.
- **Separador de bloques `思考`**: el thinking se aisla del texto de respuesta (`cloud/think.rs`).
- **Cola de fallback 429**: relevo automático entre proveedores cloud (máx 3 modelos, espera 2–4 s), `max_tokens` 39800.
- El usuario siempre recibe respuesta: si la nube falla → cae al modelo local (degradación graceful).
- **Frontend**: `adminyarvis/ChatWidget.tsx` + `front-empleado/empleayarvis/yarvis.tsx`.

### ⏳ Pendiente en IA
- **Predicciones de ventas** con intervalos de confianza (`get_predictions`, `get_predicciones_financieras`): stubs. Plan: **Holt-Winters** propio (estacionalidad semanal, ~150 líneas, sin deps pesadas).
- **RAG / knowledge_base**: embeddings nativos en Rust (ONNX/fastembed) para reactivar búsqueda semántica y base de conocimiento.

## ✅ Ola 5: Domo, seguridad y producción — COMPLETADA (parcial)

- **Autenticación**: Argon2 para admins y empleados; login por roles (`adminconfig/auth.rs`); Google OAuth (`google.rs`).
- **Gestión comercial** (admin): inventario, tickets, cortes de caja X/Z, finanzas (gastos recurrentes, alertas, métricas, exportación), empleados (metas/bonos, turnos, salario), clientes.
- **Gestión operativa** (empleado): nueva venta (+ `buscar_producto_similar` stub), perfil, tickets, cortes, chat.
- **Empaquetado**: `npm run tauri build` → binario único. Sin PyInstaller.
- **Primer inicio**: `PrimerInicio.tsx` (alta de admin + tienda + empleado) con ojos espejados y morphicons.
- **Temas**: ThemeProvider light/dark (config → AppearanceForm).

### ⏳ Pendiente en Producción
- Impresión térmica ESC/POS y facturación electrónica (XML/PAC): **aún sin implementar** (flujo visual preparado).
- Predicciones de compra con clima histórico e intervalos de confianza.
- Auditorías inventario físico vs. sistema.

---

## Orden de ataque de lo pendiente (sugerido)

1. **Pronósticos nativos (Holt-Winters)** → reactiva `get_predictions` y `get_predicciones_financieras`.
2. **Embeddings/RAG en Rust (ONNX/fastembed)** → reactiva `buscar_producto_similar` y `backfill_embeddings`.
3. **Impresión térmica (ESC/POS)** y facturación electrónica.
4. End-to-end y portabilidad en equipos viejos de Windows.

## Archivos clave

| Componente | Ubicación |
|---|---|
| Frontend admin | `yarvis-app/src/front-admin/` |
| Frontend empleado | `yarvis-app/src/front-empleado/` |
| Backend Rust (comandos) | `yarvis-app/src-tauri/src/backventanas/` |
| Motor de IA (Rust) | `src-ia/` |
| Registro de comandos | `yarvis-app/src-tauri/src/lib.rs` |
| DB (init + WAL) | `yarvis-app/src-tauri/src/backventanas/db/db.rs` |