# Tecnologias Detalladas - Y.A.R.V.I.S. POS

> Actualizado 2026-08-26: refleja el stack real y verificado. El motor de IA es 100% Rust (crate src-ia) en binario unico Tauri v2. La estrategia de IA es fine-tuning de Qwen + tools SQL, sin RAG.

## Lenguajes de Programacion

> Rust: todo el backend (Tauri) y el motor de IA (src-ia). Seguridad de memoria, tipos fuertes.
> TypeScript: frontend con tipos seguros (React + Vite).
> SQL (SQLite): base de datos relacional (sqlx, modo WAL).
> Bash / Batch: scripts de arranque (run.sh / run.bat), limpieza (reset.sh) y build (yarvis-app/build.sh).

Python eliminado. No hay sidecar, no hay yarvis-IA/, no hay ai_service.

## Frontend (Interfaz de Usuario — verificado en package.json)

> Contenedor de Escritorio: Tauri v2 (ventana nativa).
> Lenguaje: TypeScript 5.8.
> Framework: React 19.1 + Vite 7.
> Estilos: Tailwind CSS 3.4.
> Iconografia / animaciones: morphicons 1.7 (morph entre iconos SVG).
> Graficas: Recharts 3.10.
> Markdown (chat): react-markdown 10 + remark-gfm 4.
> API Tauri: @tauri-apps/api v2; plugins dialog y opener.
> Tema: context de React propio (ThemeProvider light/dark).

## Backend (Rust — verificado en Cargo.toml)

> Comunicacion con Frontend: Tauri IPC (comandos #[tauri::command], 97 registrados) via invoke().
> Framework: Tauri 2.11.
> Runtime Asincrono: Tokio 1.38 full.
> Serializacion: Serde / serde_json.
> Base de datos: SQLx 0.8 (SQLite, runtime tokio-native-tls), pool asincrono, modo WAL.
> Seguridad: Argon2 0.5 (hash de contrasenas), sha2, rand.
> HTTP: reqwest 0.12 + futures-util (SSE) para APIs cloud y OAuth. Google Sign-In (OAuth PKCE) en adminconfig/google.rs.
> Tiempo: chrono 0.4.
> Logging: tracing + tracing-subscriber (RUST_LOG, default info).
> IA:
>   - Local: llama-cpp-4 0.5 via crate src-ia, feature llm-local, modelo Qwen2.5-Coder 1.5B Instruct GGUF fine-tuneado (ruta configurable, resolucion en rutas/rutas_modelos_* incluyendo deteccion ~/.lmstudio/models). Planificado migrar a Qwen2.5-Coder 1.5B Instruct fine-tuneado para generar tools/SQL con mayor precision.
>   - Cloud: OpenCode Zen / Gemini via HTTP + SSE con fallback a local (src-ia/motor-chat/cloud), separador de bloques think/response y ciclo de tools con MAX_RONDAS_TOOLS=3.
>   - Parseo de tickets: reglas regex + analisis LLM local bajo demanda (src-ia/parseador_de_tickets).
>   - Predicciones: Holt-Winters triple aditivo sin dependencias (src-ia/predicciones), operativo via get_predictions / get_predicciones_financieras.

## Base de Datos

> Motor: SQLite 3 (un solo archivo yarvis.db en app_data_dir/com.yarvis.pos).
> Acceso: SQLx (pool asincrono), PRAGMA journal_mode=WAL.
> Escritor unico: Rust (comandos Tauri). No hay otro proceso que toque la DB.
> Dinero: INTEGER en centavos (migracion 0005_moneda_centavos.sql, modulo dinero.rs: a_centavos/a_pesos; el contrato IPC sigue en pesos f64 para no romper el frontend).
> Busqueda vectorial (sqlite-vec): deshabilitada. Los comandos buscar_producto_similar y backfill_embeddings son stubs. Hoy la vinculacion usa TF-IDF + fuzzy (src-ia/parseador_de_tickets/cerebro/vinculador_inventario/similitud.rs) como interino. Pendiente reimplementar con modelo de embeddings propio (decision: no se usara all-MiniLM ni ONNX externo).

## Modelo de despliegue

> Arranque = un solo binario (npm run tauri dev -> binario unico; yarvis-app/build.sh -> yarvis-app + .deb + .rpm + .AppImage).
> El motor de IA (src-ia) se enlaza como dependency por ruta y corre dentro del mismo proceso (feature llm-local).
> Sin puertos libres, sin health-check HTTP, sin find_free_port.
> La DB se guarda en el directorio de datos de la app (Tauri) y se resetea con reset.sh. Los modelos GGUF se resuelven por rutas dinamicas y deteccion de LM Studio.

## IA y Ciencia de Datos (estado real)

> Chat local: Qwen2.5-Coder 1.5B Instruct GGUF via llama.cpp (src-ia/motor-chat/llm, CPU, ventana 4096, fine-tuneado con tools) tokens, recorte de historial conservador (src-ia/motor-chat/llm/mod.rs:42). Comandos send_chat_message, send_chat_stream, get_cloud_models, get_model_status, load_chat_model, unload_chat_model, stop_chat_stream (admintarvis/chat.rs).
> Chat cloud: OpenCode Zen + Gemini con cola de fallback 429, streaming SSE, instrucciones de tools congeladas segun dataset tools_arreglado.jsonl (TOOLS_LINEA en src-ia/motor-chat/cloud/prompts.rs:28 y src-ia/motor-chat/llm/tools/).
> Tools: 10 tools de solo lectura, SQL parametrizado con escape_like, LIMIT parametrizado, ejecutor compartido cloud/local (src-ia/motor-chat/llm/tools/mod.rs). Roles enforced en herramientas_rol.rs (empleado no ve finanzas/nomina).
> Parseador: 100% Rust (reglas + LLM local). Comandos parser_* (adminparser/).
> Predicciones: implementadas con Holt-Winters aditivo (src-ia/predicciones/holt_winters.rs:70 predecir, ventana 7 dias, grid 343 combos alpha/beta/gamma, banda 95% z 1.96). Capa de datos lee ventas completadas, agrupa por dia, densifica huecos con 0 y devuelve fecha/prediccion/minimo/maximo (src-ia/predicciones/ventas.rs:37).
> Embeddings / RAG: no hay RAG. Pendiente busqueda semantica con modelo de embeddings propio. El plan anterior de all-MiniLM-L6-v2 + ort/fastembed fue descartado.
> Fine-tuning: Qwen local con system prompt de testing (src-ia/motor-chat/llm/mod.rs:107 SYSTEM_PROMPT_TEST, 7 tools core). Dataset tools_arreglado.jsonl congelado. Siguiente paso: fine-tune de Qwen2.5-Coder 1.5B Instruct para mejorar generacion de SQL/tools (work in progress, aun no estable).

## Empaquetamiento y Produccion

1. yarvis-app/build.sh compila el frontend (Vite) y el backend Rust + crate src-ia en un solo ejecutable. Exporta LD_LIBRARY_PATH para que linuxdeploy empaquete libllama.so.0 y NO_STRIP=1 para evitar el bug de strip viejo con .relr.dyn de Arch.
2. La DB yarvis.db se genera en el primer arranque (solo si no existe), con WAL habilitado y migraciones aplicadas en dos fases (foreign_keys off durante migracion, on en operacion normal).
3. Portabilidad: sin rutas quemadas, sin dependencias externas de runtime. El modelo GGUF se resuelve en rutas_modelos_detect.rs (incluye deteccion de ~/.lmstudio/models).
4. Requiere fuse2 y /usr/lib/gdk-pixbuf-2.0/2.10.0/loaders para el bundle AppImage en Arch (ver bugs-resueltos.md).

## Herramientas de Desarrollo

> Control de Versiones: Git.
> Gestores de Paquetes: Cargo (Rust) + npm (JS/TS).
> Tests: cargo test (backend Rust + src-ia) y npm test / vitest (frontend). Fixtures SQLite en memoria para pruebas deterministas.
> Solucion de errores de compilacion de Rust: ver comandos.md.

## Principios de mantenibilidad

- Nunca usar ./ estatico: la DB y los modelos se resuelven por rutas de datos/contexto de la app (Tauri) y deteccion dinamica.
- Un solo escritor de SQLite: Rust. Cero conflictos de lock.
- Modo degradado: si el LLM local no esta disponible o la nube falla, el POS sigue funcionando y el chat usa el fallback configurado.
- Regla de oro de lineas: ningun archivo .rs/.ts/.tsx debe pasar de 600-650 lineas; modularizar apenas se acerque (empleados.tsx ya subdividido en 3 archivos).
