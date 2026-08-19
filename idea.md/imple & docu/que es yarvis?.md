# Y.A.R.V.I.S. POS — Documentación Completa de Implementación

> ⚠️ **ACTUALIZADO (2026-Ago):** documentación sincronizada con el estado real. La IA ya es **100% Rust** (`src-ia`), sin sidecar Python. Ver `migracion_rust.md`.

## Índice

1. [Visión General](#1-visión-general)
2. [Arquitectura del Sistema](#2-arquitectura-del-sistema)
3. [Estructura de Archivos](#3-estructura-de-archivos)
4. [Frontend (React + TypeScript)](#4-frontend-react--typescript)
5. [Backend Rust (Tauri)](#5-backend-rust-tauri)
6. [Motor IA en Rust (src-ia)](#6-motor-ia-en-rust-src-ia)
7. [Modelos de IA](#7-modelos-de-ia)
8. [Base de Datos](#8-base-de-datos)
9. [Comandos Tauri (Rust)](#9-comandos-tauri-rust)
10. [Casos pendientes (stubs)](#10-casos-pendientes-stubs)
11. [Problemas Resueltos](#11-problemas-resueltos)

---

## 1. Visión General

Y.A.R.V.I.S. POS es un sistema de punto de venta de escritorio con capacidades de inteligencia artificial, pensado para tiendas medianas/pequeñas de México. Es un **binario único de Tauri v2** (React + Rust) con motor de IA nativo.

Capacidades:

- **Registro de ventas** (POS de caja del empleado) y gestión de inventario con CRUD completo.
- **Atención a clientes** con CRM básico lite (perfil, historial).
- **Cortes de caja X/Z**, gastos recurrentes, alertas financieras, métricas y exportación.
- **Empleados**: metas/bonos, turnos, salario, resumen de ventas.
- **Parseo de tickets y catálogos** (TXT/CSV/Excel) con reglas + **LLM local** para mapeo automático de columnas y procesamiento por lotes con streaming.
- **Chat con IA**: cloud (OpenCode Zen/Gemini) con fallback a local (Qwen 3 1.7B).

**Stack tecnológico (verificado):**
- Frontend: React 19 + TypeScript 5.8 + Tailwind CSS 3 + Vite 7 (+ recharts, morphicons, react-markdown).
- Backend: Rust, Tauri 2.11, sqlx 0.8 (SQLite), tokio, serde, argon2, reqwest.
- IA: crate `src-ia` — chat local con llama-cpp-4 (Qwen GGUF), chat cloud con HTTP/SSE y fallback.
- Base de datos: SQLite (WAL) via sqlx.
- Seguridad: Argon2id para contraseñas; Google OAuth.

---

## 2. Arquitectura del Sistema

```
  Frontend (React + Vite)  ──invoke──►  Backend Rust (Tauri, ~91 comandos)  ──►  SQLite (WAL)
        ▲                                        │
        └──── respuesta IPC ◄────────────────────┘
  Motor IA (crate src-ia, en proceso): chat cloud (SSE) + chat local (llama.cpp) + parseador.
```

**Ciclo de vida:**
1. `run.sh`/`run.bat` → `npm run tauri dev` (o el binario empaquetado).
2. `lib.rs` inicializa SQLite (WAL, tablas) y registra los comandos.
3. `App.tsx` decide la pantalla por `check_setup_done` (PrimerInicio / Login / Dashboard).
4. El frontend se comunica con Rust vía `invoke()` (IPC nativo; sin HTTP ni puertos).
5. La IA corre dentro del mismo proceso (crate `src-ia`); el LLM local se carga bajo demanda.

---

## 3. Estructura de Archivos

Ver el árbol completo y verificado en `../opencode/arquitectura.md`. Resumen:

- `src-ia/` — crate Rust con el motor de IA (parseador + chat cloud/local).
- `yarvis-app/src/` — frontend (front-admin, front-empleado, hooks).
- `yarvis-app/src-tauri/src/backventanas/` — comandos Tauri por dominio (backadmin/backempleado).

---

## 4. Frontend (React + TypeScript)

### 4.1 `App.tsx` — Orquestador de pantallas

Estados de paso: `0 = PrimerInicio`, `1 = Login`, `2 = AdminDashboard`, `3 = EmployeeDashboard`. La primer pantalla visible depende de `check_setup_done`.

### 4.2 `PrimerInicio.tsx` — Setup

Alta de **administrador** (nombre + contraseña con confirmación), **tienda** (nombre/identidad) y **empleados** opcionales (`+ AGREGAR EMPLEADO` con nombre + contraseña). Campos con **ojos espejados** (morphicons). Solo se muestra una vez (hasta que haya admin).

### 4.3 `Login` (App.tsx)

- Dos botones de rol: **ADMINISTRADOR** (escudo → corona al seleccionar) y **EMPLEADO** (persona+ → espada al seleccionar).
- Campo de contraseña con ojo abierto/cerrado (espejado) y botón **ENTRAR AL POS** que en hover morph la **flecha → palomita**.

### 4.4 `front-admin/` — Panel del Administrador

- `AdminDashboard.tsx`: sidebar + enrutador (ventas, inventario, tickets, finanzas, clientes, empleados, configuración, yarvis).
- **`adminconfig/`**: Configuración refactorizada en componentes (`ConfigHeader`, `IdentityForm`, `SecurityForm`, `AppearanceForm`, `importmodule/`) + hooks (`useAdminData`, `useParserActions`).
- **`parseadodetickets/`**: `BatchProcessor` (lotes con SSE), `ColumnMapper` (mapeo con IA), `CatalogosParseados`.
- **`adminfinanzas/`**: dashboard, alertas, cortes X/Z, gastos, gráficas, metrías.
- **`admininventario/`**: CRUD + importar catálogo + búsqueda semántica (stub).
- **`adminempleados/`**: empleados + modales de edición, metas y turnos.
- **`adminyarvis/`**: chat (`ChatWidget`).
- **`adminticket/`**: tickets + gráficas (usan `get_predictions` — stub).

### 4.5 `front-empleado/` — Punto de Venta

- `EmployeeDashboard.tsx`: nueva venta, inventario, tickets/cortes, clientes, perfil, yarvis, ajustes.
- `nueva_venta.tsx`: carrito con búsqueda (usa `buscar_producto_similar` — stub), modal de venta y vista de ticket.

### 4.6 Hooks globales

- `ParserContext.tsx`: estado global del parseo (items, modo, análisis LLM).
- `ThemeContext.tsx`/`useTheme.ts`: temas claro/oscuro.

---

## 5. Backend Rust (Tauri)

### 5.1 `lib.rs` — Setup principal

Inicializa DB, registra ~91 comandos en el `invoke_handler` (21 archivos), plugins (`opener`, `dialog`).

### 5.2 `db.rs` — Inicialización de SQLite

Tablas principales: `usuarios` (Argon2), `productos`, `ventas`, `detalle_ventas`, `clientes`, `ventas_diarias`, `cortes_caja`, `predicciones_futuras`. **WAL activado.**

### 5.3 Módulos `backventanas/`

| Módulo | Contenido |
|---|---|
| `backadmin/adminconfig` | auth (setup + login + datos admin/empleado), google (OAuth) |
| `backadmin/admininventory` | CRUD inventario, `importar_catalogo`, stubs embeddings |
| `backadmin/adminparser` | parseo TXT/CSV/Excel, carpetas, vinculación, modelos |
| `backadmin/admintickets` | tickets, cortes; `get_predictions` (stub) |
| `backadmin/adminfinanzas` | gastos, cortes de caja, alertas, métricas, gráficas, export |
| `backadmin/adminempleados` | empleados + modales (metas, turnos) |
| `backadmin/admintarvis` | chat (send_chat_message/stream, modelos, status) |
| `backempleado` | venta nueva (`completar_venta`, `get_next_ticket_number`), perfil |

---

## 6. Motor IA en Rust (src-ia)

### 6.1 `parseador_de_tickets/`

- `cerebro/`: reglas (analizador_tickets), filtrador (3 niveles), parseador_masivo (lotes SSE, transacción por archivo), vinculador_inventario.
- `formatos/`: lector CSV, Excel (`calamine`), TXT.
- `rutas/`: resolución de modelos GGUF + análisis LLM (`analizar_ticket`, `generar_bajo_lock`).

### 6.2 `motor-chat/`

- `cloud/`: proveedores (OpenCode Zen, Gemini), generación (completo/stream), catálogo de modelos, lector SSE, cola de fallback 429, separador de bloques `思考`, variables/API keys.
- `llm/`: Qwen 3 1.7B vía llama-cpp-4 (feature `llm-local`), CPU.

---

## 7. Modelos de IA

### 7.1 LLM Local (chat + parseo)

- **Qwen 3 1.7B GGUF** — único modelo local; se carga bajo demanda (chat y análisis de tickets). Resolución de rutas: `src-ia/rutas/rutas_modelos_*` (incluye detección de modelos de LM Studio).

### 7.2 Cloud

- **OpenCode Zen / Gemini** — HTTP + SSE, con relevo automático ante 429 (`max_tokens` 39800). El thinking se separa del texto de respuesta.

### 7.3 Embeddings / Prophet (PENDIENTE — stubs)

- Embeddings (RAG) y pronósticos (Prophet) quedaron **fuera de alcance** de la migración. Ver [sección 10](#10-casos-pendientes-stubs).

---

## 8. Base de Datos

SQLite, un archivo (`yarvis.db`), modo WAL, acceso asíncrono con `sqlx`. **Único escritor: Rust.** Tablas: `usuarios`, `productos`, `ventas`, `detalle_ventas`, `clientes`, `ventas_diarias`, `cortes_caja`, `predicciones_futuras`.

---

## 9. Comandos Tauri (Rust)

**~91 comandos `#[tauri::command]` en 21 archivos** de `backventanas/`. Resumen por módulo:

**Auth / Setup** (`adminconfig/auth.rs`, `google.rs`)
`check_setup_done`, `guardar_admin`, `validar_login_admin`, `get_admin_data`, `update_admin_data`, `guardar_empleado`, `validar_login_empleado`, `login_con_google`

**Inventario** (`admininventory/inventory.rs`)
`get_inventory`, `add_inventory_item`, `update_inventory_item`, `delete_inventory_item`, `importar_catalogo`, `get_catalogos_importados`, `get_productos_por_catalogo`, `buscar_producto_similar` *(stub)*, `backfill_embeddings` *(stub)*

**Parser** (`adminparser/*`)
`listar_archivos_carpeta`, `leer_archivo_raw`, `leer_archivo_bytes`, `parsear_catalogo_visual`, `parsear_catalogo_csv`, `parsear_excel`, `analizar_ticket_llm`, `analizar_ticket_con_ia`, `parsear_con_mapeo`, `parsear_carpeta`, `parsear_carpeta_stream`, `get_db_path`, `vincular_inventario`, `guardar_vinculacion`, `descargar_modelos`

**Tickets y cortes** (`admintickets/tickets.rs`)
`get_tickets`, `get_cortes`, `guardar_ticket_parseado`, `get_predictions` *(stub)*

**Empleados** (`adminempleados/*`)
`get_empleados`, `get_empleado_ventas`, `get_resumen_empleados`, `get_cortes_empleado`, `update_empleado`, `delete_empleado`, `get_salario_info`, `save_salario`, `get_employee_goals`, `save_employee_goal`, `save_custom_goal`, `delete_employee_goal`, `check_employee_goals`, `get_turnos_empleados`

**Finanzas** (`adminfinanzas/*`)
Gastos: `get_gastos_recurrentes`, `crear_gasto`, `actualizar_gasto`, `eliminar_gasto`, `registrar_pago_gasto`, `get_pagos_gasto`, `get_proximos_vencimientos`, `actualizar_estados_gastos`; Cortes: `get_cortes_caja`, `get_corte_detalle`, `crear_corte_x`, `crear_corte_z`, `cerrar_corte`, `agregar_movimiento_caja`, `get_movimientos_corte`, `get_cortes_...`; Métricas, gráficas, alertas y export.

**Chat** (`admintarvis/chat.rs`)
`send_chat_message`, `send_chat_stream`, `get_cloud_models`, `get_model_status`, `load_chat_model`, `unload_chat_model`, `stop_chat_stream`

**Empleado operativo** (`backempleado/*`)
`completar_venta`, `get_next_ticket_number`, `get_tienda_info`, `get_employee_profile`

---

## 10. Casos pendientes (stubs)

| Comando | Función | Estado |
|---|---|---|
| `buscar_producto_similar` | Búsqueda semántica en inventario / nueva venta | STUB |
| `backfill_embeddings` | Generar embeddings de la base | STUB |
| `get_predictions` | Pronósticos de ventas (gráficas tickets) | STUB |
| `get_predicciones_financieras` | Pronósticos financieros (GraficasPanel) | STUB |

**Planes:** embeddings ONNX (`all-MiniLM-L6-v2` + `ort`/`fastembed-rs`) y pronósticos **Holt-Winters** (sin deps pesadas). Detalles en `migracion_rust.md`. Impresión térmica ESC/POS y facturación electrónica: pendientes.

---

## 11. Problemas Resueltos

- **Migración Python → Rust completa**: sidecar eliminado, binario único (commit histórico; ver `migracion_rust.md`).
- **Parseo robusto**: bugfixes A1/A3/A4/Bug8 (líneas útiles, volúmenes, separador, transacciones) conservados en el port (detalle en `Bugs resueltos uwu.md` y `PARSEADOR.md`).
- **UI/UX refinada**: primer inicio con confirmación de contraseña y ojos espejados; login con morph de iconos por rol; botón ENTRAR AL POS con morph flecha→palomita; botón Cancelar con borde punteado.