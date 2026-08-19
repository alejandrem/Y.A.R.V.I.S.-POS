# Interconexión del Sistema — Y.A.R.V.I.S. POS

> ⚠️ **ACTUALIZADO (2026-Ago):** este doc describía el viejo modelo "Jefe (Rust) + Empleado (Python sidecar)" con HTTP local y puertos dinámicos. **Ese modelo ya NO existe.** Hoy el sistema es un **binario único de Tauri**: Frontend (React) → Rust (comandos Tauri) → IA en el crate local `src-ia` (mismo proceso). Mantenemos la filosofía central (un solo escritor de SQLite, rust gestiona errores) pero sin segundo ejecutable.

---

## Principios que se conservan

**Rust no falla: gestiona errores.** Cada comando devuelve `Result`/`Option`; si la impresora no responde o la IA tarda, el usuario recibe un mensaje claro y la venta nunca se pierde ("la venta se guardó, ¿quieres reintentar imprimir?").

**Rust define el dominio con tipos.** Structs `serde` que no permiten estados imposibles (una venta sin total negativo, etc.) — `models.rs`.

**Concurrencia con Tokio.** Mientras el chat cloud/LLM responde (1–5 s), el cajero sigue cobrando: los comandos `async` no bloquean la UI.

**Un solo escritor en SQLite.** El backend Rust es el único que escribe `yarvis.db` (pool `sqlx`, WAL). No hay un segundo proceso en la foto.

---

## Flujo de comunicación ACTUAL

| Origen | Destino | Mecanismo | Propósito |
|---|---|---|---|
| Frontend (React) | Backend Rust | **Tauri IPC** (`invoke()`) | Todo: ventas, inventario, tickets, chat, parseo, empleados, finanzas |
| Backend Rust | SQLite | **sqlx (pool, WAL)** | Lecturas + escrituras de `yarvis.db` |
| Backend Rust (chat) | Motor IA | **crate `src-ia` en proceso** | Chat cloud (HTTP+SSE) y local (llama.cpp) |
| Backend Rust (parser) | Motor IA | **crate `src-ia` en proceso** | Reglas de parseo + análisis LLM local |

*No hay HTTP local. No hay puertos libres. No hay `ai_service.exe`. No hay `externalBin` en `tauri.conf.json`.*

---

## Boot Sequence (arranque actual)

1. El usuario ejecuta `yarvis-app` (el binario único).
2. `lib.rs` abre el pool SQLite (`db/db.rs`): crea el archivo `yarvis.db` si no existe, activa WAL, crea tablas.
3. Se registran **~91 comandos Tauri** en el `invoke_handler`.
4. `main.tsx` monta React: el orquestador `App.tsx` decide la pantalla según `check_setup_done` (que es un comando real):
   - **Paso 0** `PrimerInicio`: primer registro de administrador + tienda + empleado (solo se muestra una vez).
   - **Paso 1** Login con selección de rol (Admin: escudo→corona; Empleado: persona+→espada) y contraseña.
   - **Paso 2** `AdminDashboard` o **Paso 3** `EmployeeDashboard`.
5. El LLM NO carga al arrancar. Se carga "lazy" cuando el chat/parseo lo necesita (`load_chat_model`, análisis de tickets), y el motor local comparte una única instancia Qwen 1.7B.

---

## Flujo del Chat (nube + fallback local)

1. El usuario escribe en `adminyarvis` o `empleayarvis`.
2. `ChatWidget` llama `send_chat_stream` (Tauri command).
3. `admintarvis/chat.rs` delega en `src-ia/motor-chat`:
   - **Cloud**: `cloud/apis_cloud/generacion.rs` abre un stream HTTP/SSE a OpenCode Zen o Gemini. Los bloques `思考` se separan del texto (`think.rs`). Si el proveedor devuelve 429 (rate limit), la **cola de fallback** releva al siguiente proveedor (espera 2–4 s, máx 3 modelos).
   - **Local**: `llm/mod.rs` genera con Qwen 3 1.7B (llama.cpp) en CPU.
4. La respuesta llega al frontend por el mismo stream (IPC `invoke` no bloquea la caja).
5. Si todo lo cloud falla → respuesta local. El usuario siempre recibe algo (degradación graceful).

## Flujo del Parseador (reglas + LLM bajo demanda)

1. El admin abre el **Módulo de Importación Inteligente** (`ImportModule.tsx`).
2. Sube TXT / CSV / Excel; se llaman los comandos `parser_*` (`adminparser/`).
3. `src-ia/parseador_de_tickets` aplica:
   - **Reglas** (`cerebro/`): filtrado de líneas, encabezados, fechas, pagos, totales, segmentación.
   - **Lectores** (`formatos/`): CSV (auto-detect separador), Excel (`calamine`), TXT.
   - **LLM** (si aplica): `analizar_ticket_con_ia` con análisis local; mapeo de columnas confirmado por el usuario (`ColumnMapper`).
4. `parsear_carpeta_stream` procesa carpetas enteras con **eventos SSE** al frontend y **transacción por archivo** (rollback ante fallo).
5. Vincular con inventario → `vincular_inventario` / `guardar_vinculacion` (SQLite, vía Rust).

---

## Base de Conocimiento y búsqueda semántica (PENDIENTE)

En la era Python existía `knowledge_base` con sqlite-vec y embeddings para búsqueda semántica y RAG. En la migración a Rust **quedaron como stubs**:

- `buscar_producto_similar` (inventario / nueva venta)
- `backfill_embeddings` (importación)
- `get_predictions` / `get_predicciones_financieras` (gráficas de tickets y finanzas)

**Plan**: reimplementar embeddings nativos (ONNX all-MiniLM-L6-v2 + `ort`/`fastembed-rs`) y pronósticos (Holt-Winters propio). Referencia de diseño en `migracion_rust.md`.

## Relaciones por rol

- **Administrador**: `front-admin/` + `backadmin/` → gestión total (ventas, inventario, tickets, finanzas, clientes, empleados, configuración, yarvis).
- **Empleado**: `front-empleado/` + `backempleado/` → caja (nueva venta), inventario de consulta, tickets/cortes propios, perfil, ajustes y chat. La mayoría reutiliza comandos admin registrados globalmente.

## Clima y predicción (PENDIENTE)

El diseño original correlacionaba el clima histórico (ej. frente frío → +pan) durante el corte Z para alimentar predicciones. Sigue siendo el objetivo para la reimplementación de pronósticos, pero **la API de clima y la tabla de correlaciones aún no se integran**. No hay bloqueo para la caja.