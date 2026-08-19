# Tecnologías Detalladas - Y.A.R.V.I.S. POS

> ⚠️ **Actualizado (2026-Ago):** este doc refleja el **stack real y verificado**. El motor de IA ya no es Python: todo corre como un **binario único de Tauri v2** con el núcleo de IA en un crate Rust local (`src-ia`). Para el histórico de la migración ver `../imple & docu/migracion_rust.md`.

## Lenguajes de Programación

> **Rust**: Todo el backend (Tauri) y el motor de IA (`src-ia`). Seguridad de memoria, confianza cero, tipos fuertes.
> **TypeScript**: Frontend con tipos seguros (React + Vite).
> **SQL (SQLite)**: Base de datos relacional (sqlx, modo WAL).
> **Bash / Batch**: Scripts de arranque (`run.sh` / `run.bat`) y limpieza (`reset.sh`).
>
> **Python: ELIMINADO.** No hay sidecar, no hay `yarvis-IA/`, no hay `ai_service.exe`.

## Frontend (Interfaz de Usuario — verificado en `package.json`)

> **Contenedor de Escritorio**: Tauri v2 (ventana nativa, sin navegadores externos).
> **Lenguaje**: TypeScript 5.8.
> **Framework**: React 19 + Vite 7.
> **Estilos**: Tailwind CSS 3.
> **Iconografía / animaciones de iconos**: **morphicons** (morph entre iconos SVG path a path).
> **Gráficas**: Recharts.
> **Markdown (chat)**: react-markdown + remark-gfm.
> **API Tauri**: @tauri-apps/api v2; plugins `dialog` y `opener`.
> **Tema**: context de React propio (ThemeProvider light/dark; no Zustand).

## Backend (Rust — verificado en `Cargo.toml`)

> **Comunicación con Frontend**: Tauri IPC (comandos `#[tauri::command]`, ~91 registrados) vía `invoke()`.
> **Framework**: Tauri 2.11.
> **Runtime Asíncrono**: Tokio.
> **Serialización**: Serde / serde_json.
> **Base de datos**: SQLx 0.8 (SQLite, runtime tokio), pool asíncrono, modo WAL.
> **Seguridad**: Argon2 (hash de contraseñas), sha2, rand.
> **HTTP**: reqwest + futures-util (SSE) para APIs cloud y OAuth. Google Sign-In (OAuth) en `adminconfig/google.rs`.
> **Tiempo**: chrono.
> **IA**:
>   - Local: **llama-cpp-4** (bindings de llama.cpp) para Qwen 3 1.7B GGUF (crate `src-ia`, feature `llm-local`).
>   - Cloud: **OpenCode Zen / Gemini** vía HTTP + SSE con fallback a local (`src-ia/motor-chat/cloud`).
>   - Parseo de tickets: reglas regex + análisis LLM local bajo demanda (`src-ia/parseador_de_tickets`).

## Base de Datos

> **Motor**: SQLite 3 (un solo archivo `yarvis.db`).
> **Acceso**: SQLx (pool asíncrono), `PRAGMA journal_mode=WAL`.
> **Escritor único**: Rust (comandos Tauri). No hay otro proceso que toque la DB.
> **Búsqueda vectorial (sqlite-vec)**: **deshabilitada** — los comandos de embeddings/RAG son stubs (`backfill_embeddings`, `buscar_producto_similar`). Pendiente reimplementar con ONNX/fastembed-rs propio u otro motor nativo (ver `migracion_rust.md`, Fase 4 fuera de alcance).

## Modelo de despliegue (reemplaza al viejo "Jefe/Empleado")

> El modelo "sidecar Rust + Python empaquetado en /engine" **ya no existe**. Ahora:
> - Arranque = **un solo binario** (`npm run tauri dev` → binario único; `npm run tauri build` → `yarvis-app.exe` / binario Linux).
> - El motor de IA (`src-ia`) se enlaza como dependency por ruta y corre **dentro del mismo proceso**.
> - Sin puertos libres, sin health-check HTTP, sin `find_free_port`, sin `LD_LIBRARY_PATH` para Python.
> - La DB (`yarvis.db`) se guarda en el directorio de datos de la app (Tauri) y se resetea con `reset.sh`.

## IA y Ciencia de Datos (estado real)

> **Chat local**: Qwen 3 1.7B GGUF vía llama.cpp (`src-ia/motor-chat/llm`), CPU. Comandos `send_chat_message`, `send_chat_stream`, `get_cloud_models`, `get_model_status`, `load_chat_model`, `unload_chat_model`, `stop_chat_stream` (`admintarvis/chat.rs`).
> **Chat cloud**: OpenCode Zen + Gemini con **cola de fallback 429** (máx 3 modelos, espera 2–4 s), `max_tokens` 39800, streaming SSE con separador de bloques `思考` (`src-ia/motor-chat/cloud`).
> **Parseador**: 100% Rust (reglas + LLM local). Comandos `parser_*` (`adminparser/`).
> **Predicciones/Prophet**: **PENDIENTE** — `get_predictions` y `get_predicciones_financieras` son stubs. Reimplementación nativa planeada con Holt-Winters (sin deps pesadas).
> **Embeddings/RAG**: **PENDIENTE** — stubs (`backfill_embeddings`, `buscar_producto_similar`). Plan: all-MiniLM-L6-v2 ONNX + `ort`/`fastembed-rs`.

## Empaquetamiento y Producción

1. `npm run tauri build` compila el frontend (Vite) y el backend Rust + crate `src-ia` en **un solo ejecutable**.
2. La DB `yarvis.db` se genera en el primer arranque (solo si no existe), con WAL habilitado.
3. **Portabilidad**: sin rutas quemadas, sin dependencias externas de runtime (el modelo GGUF se resuelve en `src-ia/rutas/rutas_modelos_detect.rs`, incluye detección de `~/.lmstudio/models`).
4. Windows Defender: sin sidecar PyInstaller, ya no hay binarios extra que firmar.

## Herramientas de Desarrollo

> **Control de Versiones**: Git.
> **Gestores de Paquetes**: Cargo (Rust) + npm (JS/TS).
> **Solución de errores de compilación de Rust**: ver `comandos.md` (rustup default stable + cargo clean).

## Pro-Tips para la Portabilidad

- **Nunca usar `./` estático**: la DB y los modelos se resuelven por rutas de datos/contexto de la app (Tauri) y detección dinámica.
- **Un solo escritor de SQLite**: Rust. Cero conflictos de lock.
- **Modo degradado**: si el LLM local no está disponible o la nube falla, el POS sigue funcionando y el chat usa el fallback configurado.
- **Regla de oro de líneas**: ningún archivo `.rs`/`.ts`/`.tsx` debe pasar de ~600–650 líneas; modularizar apenas se acerque.