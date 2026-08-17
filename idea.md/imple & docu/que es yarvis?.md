# Y.A.R.V.I.S. POS — Documentación Completa de Implementación

## Índice

1. [Visión General](#1-visión-general)
2. [Arquitectura del Sistema](#2-arquitectura-del-sistema)
3. [Estructura de Archivos](#3-estructura-de-archivos)
4. [Frontend (React + TypeScript)](#4-frontend-react--typescript)
5. [Backend Rust (Tauri)](#5-backend-rust-tauri)
6. [Backend Python (FastAPI Sidecar)](#6-backend-python-fastapi-sidecar)
7. [Modelos de IA](#7-modelos-de-ia)
8. [Base de Datos](#8-base-de-datos)
9. [Flujo de Datos](#9-flujo-de-datos)
10. [Endpoint API (Python)](#10-endpoint-api-python)
11. [Comandos Tauri (Rust)](#11-comandos-tauri-rust)
12. [Historial de Implementación](#12-historial-de-implementación)
13. [Problemas Resueltos](#13-problemas-resueltos)

---

## 1. Visión General

Y.A.R.V.I.S. POS es un sistema de punto de venta de escritorio con capacidades de inteligencia artificial. Permite:

- **Parseo de tickets** usando LLM (Qwen 2.5 0.5B / Qwen 3 1.7B) para detectar automáticamente columnas
- **Importación de catálogos** desde archivos Excel (.xlsx), CSV y TXT
- **Procesamiento por lotes** de miles de tickets .txt con streaming SSE
- **Búsqueda semántica** de productos usando embeddings (all-MiniLM-L6-v2, 384 dimensiones)
- **Predicción de ventas** con Facebook Prophet
- **Gestión de inventario** con CRUD completo y alertas de stock bajo
- **Gestión de corte de caja** y historial de ventas

**Stack tecnológico:**
- Frontend: React + TypeScript + Tailwind CSS
- Backend: Rust (Tauri v2) + Python (FastAPI)
- IA: llama-cpp-python (Qwen GGUF) + sentence-transformers + Prophet
- Base de datos: SQLite (WAL mode) via sqlx
- Seguridad: Argon2id para contraseñas

---

## 2. Arquitectura del Sistema

    // ... (Código omitido para brevedad) ...

**Flujo de ciclo de vida:**
1. El usuario ejecuta `run.sh` → `npm run tauri dev`
2. Rust `lib.rs` inicializa SQLite y lanza el sidecar Python en un puerto libre
3. El sidecar Python levanta FastAPI con uvicorn en ese puerto
4. El frontend se comunica con Rust via `invoke()` (IPC de Tauri)
5. Rust se comunica con Python via HTTP (reqwest)
6. Los modelos de IA se cargan lazy cuando se necesitan y se descargan al terminar

---

## 3. Estructura de Archivos

### 3.1 Frontend (`yarvis-app/src/`)

    // ... (Código omitido para brevedad) ...

### 3.2 Backend Rust (`yarvis-app/src-tauri/src/`)

    // ... (Código omitido para brevedad) ...

### 3.3 Backend Python (`yarvis-IA/`)

    // ... (Código omitido para brevedad) ...

---

## 4. Frontend (React + TypeScript)

### 4.1 `types.ts` — Interfaces Compartidas

    // ... (Código omitido para brevedad) ...

### 4.2 `Configuracion.tsx` — Panel de Configuración + Parseo

**Propósito:** Panel principal de configuración con 3 modos de parseo de archivos.

**Estados principales:**
- `parserMode`: `"catalogo"` | `"entrenar IA"` | `"insertar"`
- `parsedItems`: Items parseados actualmente
- `llmAnalysis`: Resultado del análisis de la IA
- `catalogParsed`, `iaTrained`, `ticketsParsed`: Flags de progreso del pipeline
- `lastCatalogPath`, `lastCatalogItems`: Persistencia del último catálogo parseado

**Funciones clave:**

| Función | Descripción |
|---------|-------------|
| `handleFileSelect()` | Abre diálogo de archivos, detecta extensión (.txt/.csv/.xlsx), delega al parser correcto |
| `handleGuardarTicket(items, analysis)` | Guarda ticket en DB via `invoke("guardar_ticket_parseado")`, actualiza flags de pipeline |
| `handleTrainIA()` | Para modo "catalogo": importa catálogo al inventario. Para "insertar": no hace nada |

**Flujo por modo:**

1. **"entrenar IA"**: Carga archivo → ColumnMapper aparece → Analiza con IA → "Guardar Ticket" → Guarda en DB
2. **"catalogo"**: Carga archivo → Parsea catálogo → Preview → "Entrenar IA con Catálogo" → Importa a inventario
3. **"insertar"**: Selecciona carpeta → BatchProcessor aparece → Procesa todos los .txt → Vincula productos

**Indicador de estado del pipeline:**
- Gris: "Esperando datos"
- Naranja: "Esperando entrenamiento de IA"
- Amarillo: "Esperando parseamiento de tickets"
- Verde: "N tickets parseado(s) con éxito"

### 4.3 `ColumnMapper.tsx` — Mapeo de Columnas con IA

**Propósito:** Interfaz para que la IA detecte las columnas del ticket y el usuario ajuste el mapeo.

**Props:**
- `onGuardarTicket(items, analysis)`: Callback para guardar el ticket
- `onPreviewUpdate(items)`: Callback para actualizar la previsualización en el padre
- `fileContent`: Texto crudo del archivo
- `selectedPath`: Ruta del archivo seleccionado

**Flujo:**
1. Usuario hace clic en "Analizar con IA"
2. Se llama `invoke("analizar_ticket_con_ia", { texto: fileContent })`
3. La IA retorna `LLMAnalysis` con mapeo de columnas y `ejemplo_parseado`
4. Se normaliza `producto` de número a array: `2` → `[2]`
5. Se muestra panel de ajuste con 5 dropdowns (Cantidad, Producto, Precio, Total, Descuento)
6. `useEffect` pasa `ejemplo_parseado` al padre via `onPreviewUpdate`
7. Usuario hace clic en "Guardar Ticket" → se llama `onGuardarTicket`

**Nota importante:** `previewItems` usa `analysis.ejemplo_parseado` (los items que la IA ya parseó) en vez de re-parsear el texto. Esto es porque `esLineaUtil` filtra líneas con metadata del ticket (fecha, cajero, subtotal, etc.) y las primeras 10 líneas suelen ser metadata.

### 4.4 `BatchProcessor.tsx` — Procesamiento por Lotes

**Propósito:** Procesar una carpeta completa de archivos .txt de tickets.

**Funcionamiento:**
1. Selecciona carpeta
2. Llama `invoke("parsear_carpeta_stream", { carpeta, mapeo, dbPath })`
3. Recibe respuesta SSE con progreso
4. Muestra estadísticas: procesados, exitosos, errores, ventas creadas
5. Al terminar, muestra productos nuevos y ofrece "Vincular con Inventario Existente"

**Mapeo hardcodeado:** `{ cantidad: 0, producto: [1], precio_unitario: 2, total: 3 }`

---

## 5. Backend Rust (Tauri)

### 5.1 `lib.rs` — Setup Principal

**Función `run()`:**
1. Crea `AiSidecar` compartido via `Arc`
2. Inicializa SQLite via `db::initialize_db()`
3. Registra todos los comandos Tauri
4. Lanza el sidecar Python en background via `tauri::async_runtime::spawn`
5. Al cerrar la ventana, llama `shutdown_ai_engine()` para matar Python

**Comandos registrados (28 total):**
- Auth: 7 comandos
- Inventario: 6 comandos
- Parser: 14 comandos (incluyendo los de parser_rs)
- Tickets: 3 comandos
- IA: 1 comando

### 5.2 `sidecar.rs` — Ciclo de Vida del Sidecar

**Estructura `AiSidecar`:**
    // ... (Código omitido para brevedad) ...

**Métodos:**
| Método | Descripción |
|--------|-------------|
| `base_url()` | Retorna `Some("http://127.0.0.1:{port}")` o `None` |
| `get_status()` | Retorna el estado actual del sidecar |
| `check_process_alive()` | Verifica si Python sigue vivo, limpia si murió |

**Flujo de arranque (`launch_ai_engine`):**
1. `find_two_free_ports()` — Busca 2 puertos libres via bind a `127.0.0.1:0`
2. Guarda puertos en el estado
3. `start_python(port)` — Lanza `python3 main.py {port}` con `LD_LIBRARY_PATH` para CUDA
4. `wait_health_check(port, 30)` — Polling cada 500ms por 30 segundos
5. Si responde: status = `Ready`. Si no: status = `Error`, mata el proceso

**CUDA/LD_LIBRARY_PATH:**
El sidecar busca libs CUDA en rutas de LM Studio y nvidia pip packages, y las agrega a `LD_LIBRARY_PATH` antes de lanzar Python.

### 5.3 `db.rs` — Inicialización de SQLite

**Tablas creadas (8):**
1. `usuarios` — Admin y empleados (Argon2id hash)
2. `productos` — Inventario con precios y stock
3. `ventas` — Cabeceras de venta (total, IVA, cajero, método pago)
4. `detalle_ventas` — Líneas de venta (producto, cantidad, precio, descuento)
5. `ventas_diarias` — Resumen diario para Prophet
6. `cortes_caja` — Cortes de caja
7. `predicciones_futuras` — Predicciones de Prophet
8. `knowledge_base` — Embeddings de productos para búsqueda semántica

**WAL mode:** Habilitado para mejor concurrencia.

### 5.4 `models.rs` — Modelos de Datos

    // ... (Código omitido para brevedad) ...

### 5.5 `commands/inventory.rs` — CRUD de Inventario

**Funciones:**

| Función | Descripción |
|---------|-------------|
| `get_inventory()` | Retorna todos los productos |
| `add_inventory_item()` | Inserta producto + genera embedding en background |
| `update_inventory_item()` | Actualiza producto + regenera embedding |
| `delete_inventory_item()` | Elimina producto por ID |
| `importar_catalogo()` | Inserta múltiples productos + genera embeddings |
| `buscar_producto_similar()` | Búsqueda semántica via Python `/buscar_similar` |

**Generación de embeddings:**
- Se ejecuta en `tokio::spawn` (background, no bloquea)
- Verifica `sidecar.get_status() == Ready` antes de llamar
- Usa `check_process_alive()` para detectar procesos muertos
- Si falla, imprime advertencia una sola vez (AtomicBool)

### 5.6 `commands/parser.rs` — Puente al Sidecar

| Función | Endpoint Python | Descripción |
|---------|----------------|-------------|
| `get_db_path()` | — | Retorna la ruta de la DB (managed state) |
| `vincular_inventario()` | POST /vincular_inventario | Vincula productos parseados con inventario existente |
| `guardar_vinculacion()` | POST /guardar_vinculacion | Guarda vinculaciones en la DB |
| `descargar_modelos()` | POST /unload_llm | Descarga modelos Qwen de VRAM |

### 5.7 `parser_rs/` — Parsers Locales en Rust

| Archivo | Funciones | Descripción |
|---------|-----------|-------------|
| `utils.rs` | `sanitize_path()` | Canonicaliza rutas, bloquea directorios del sistema |
| `parser_csv.rs` | `parsear_catalogo()` | Parser CSV auto-detect (separador, header, columnas numéricas) |
| `parser_excel.rs` | `parsear_excel()` | Envía bytes al sidecar Python `/parsear_excel` |
| `parser_txt.rs` | `leer_archivo_raw()`, `leer_archivo_bytes()`, `parsear_ticket()`, `parsear_catalogo_visual()`, `analizar_ticket_llm()`, `analizar_ticket_con_ia()`, `parsear_con_mapeo()`, `parsear_carpeta()`, `parsear_carpeta_stream()` | wrappers de Tauri commands que delegan a Python |

---

## 6. Backend Python (FastAPI Sidecar)

### 6.1 `main.py` — Entry Point

    // ... (Código omitido para brevedad) ...

### 6.2 `parseador_de_tickets/cerebro/analizador.py` — Endpoints de Parseo

| Endpoint | Método | Descripción |
|----------|--------|-------------|
| `/analizar_ticket` | POST | Recibe `{"texto": "..."}`, llama a Qwen, retorna mapeo + ejemplo_parseado JSON. **Descarga modelos al terminar.** |
| `/parsear_con_mapeo` | POST | Parsea texto usando mapeo de columnas confirmado por el usuario (sin LLM) |
| `/parsear_excel` | POST | Recibe bytes de Excel, retorna productos detectados |

### 6.3 `parseador_de_tickets/cerebro/lote.py` — Procesamiento Masivo

| Endpoint | Método | Descripción |
|----------|--------|-------------|
| `/parsear_carpeta_stream` | POST | Procesa carpeta con SSE streaming asíncrono. |

**Descarga modelos de VRAM antes de empezar** para liberar memoria.

**Flujo de `/parsear_carpeta_stream`:**
1. Carga estado de productos existentes de la DB
2. Procesa archivos en batches optimizados
3. Yield de eventos SSE con progreso después de cada batch
4. Evento final con estadísticas completas

### 6.4 `chatbot/embeddings/endpoints.py` — Vectores Semánticos

| Endpoint | Método | Descripción |
|----------|--------|-------------|
| `/generar_embedding` | POST | `{"texto": "..."}` → vector 384d → base64 |
| `/buscar_similar` | POST | Búsqueda por cosine similarity en knowledge_base |

### 6.5 `chatbot/motor_chat/endpoints.py` — Chat y Cerebro LLM

| Endpoint | Método | Descripción |
|----------|--------|-------------|
| `/chat` | POST | Inferencia con RAG y SQL tools |
| `/load_llm` | POST | Carga modelo forzando reserva de memoria RAM |
| `/unload_llm` | POST | Libera RAM manualmente |

---

## 7. Modelos de IA

### 7.1 LLM del Parseador (`parseador_de_tickets/llm/analizador_llm.py`)

**Modelos (Escalada por Confianza):**

**Flujo de análisis:**
1. Intenta con Qwen 0.5B
2. Si confianza < 0.8, reintenta con Qwen 1.7B
3. Si 0.5B falla, usa 1.7B directamente
4. **Descarga modelos después de cada análisis** para liberar VRAM

**Parámetros de carga:**
    // ... (Código omitido para brevedad) ...

**Función `descargar_modelos()`:**
    // ... (Código omitido para brevedad) ...

**Prompt del sistema:**
Le pide a la IA que analice un ticket y retorne JSON con:
- `mapeo.columnas` — Índices de cada columna
- `ejemplo_parseado` — Items parseados
- `confianza` — Nivel de confianza (0-1)

### 7.2 LLM del Chatbot (`chatbot/motor_chat/gestion_hardware.py`)

A diferencia del parseador, el chatbot NO escala de esta manera. En su arranque **activa un único modelo (Lazy Loading)** permanentemente mientras la pestaña del chat esté abierta, para asegurar velocidad y respuesta conversacional fluida. `gestion_hardware.py` valida la RAM contra `_RAM_REQUERIDA` (Q4: 0.5B → 0.0GB, 0.8B → 0.5GB, 1.7B → 1.3GB) y el frontend descarga el modelo actual antes de cargar otro (solo uno a la vez).

### 7.3 Embeddings (`chatbot/embeddings/modelo.py`)

**Modelo:** all-MiniLM-L6-v2 (sentence-transformers)
- Dimensiones: 384
- Normalización: L2
- Uso: Base de Conocimiento (RAG) y coincidencia de inventarios

**Funciones:**
- `texto_a_embedding(texto)` → vector 384d
- `embedding_a_blob(vec)` → bytes base64 para SQLite

### 7.3 Prophet (`modelos/profeta/predictor.py`)

**Modelo:** Facebook Prophet
- Entrena con `ventas_diarias`
- Genera predicciones N días hacia adelante
- Incluye intervalos de confianza

---

## 8. Base de Datos

### Esquema SQLite

    -- (Tablas omitidas: usuarios, productos, ventas, detalle_ventas, ventas_diarias, cortes_caja, predicciones_futuras, knowledge_base)

---

## 9. Flujo de Datos

### 9.1 Flujo de Parseo de Ticket (entrenar IA)
*(Flujo omitido. Consultar código fuente.)*
    // ... (Código omitido para brevedad) ...
### 9.2 Flujo de Importación de Catálogo
*(Flujo omitido. Consultar código fuente.)*
    // ... (Código omitido para brevedad) ...
### 9.3 Flujo de Procesamiento por Lotes
*(Flujo omitido. Consultar código fuente.)*
    // ... (Código omitido para brevedad) ...
## 10. Endpoint API (Python)

### Tabla Completa de Endpoints

| # | Endpoint | Método | Router | Descripción |
|---|----------|--------|--------|-------------|
| 1 | `/` | GET | main | Health check |
| 2 | `/generar_embedding` | POST | embeddings | Genera embedding 384d de texto |
| 3 | `/buscar_similar` | POST | embeddings | Búsqueda semántica en knowledge_base |
| 4 | `/insertar_knowledge` | POST | embeddings | Inserta en knowledge_base |
| 5 | `/recalcular_predicciones` | POST | predictions | Ejecuta Prophet y guarda predicciones |
| 6 | `/analizar_ticket` | POST | parser | Analiza ticket con LLM + descarga modelos |
| 7 | `/parsear_con_mapeo` | POST | parser | Parsea texto con mapeo de columnas |
| 8 | `/parsear_catalogo_visual` | POST | parser | Parsea catálogo visual |
| 9 | `/parsear_excel` | POST | parser | Parsea Excel (bytes) |
| 10 | `/parsear_carpeta` | POST | carpeta | Procesa carpeta sincrónicamente |
| 11 | `/parsear_carpeta_stream` | POST | carpeta | Procesa carpeta con SSE + descarga modelos |
| 12 | `/vincular_inventario` | POST | matching | Vincula productos con inventario |
| 13 | `/guardar_vinculacion` | POST | matching | Guarda vinculaciones |
| 14 | `/chat` | POST | chat | Placeholder chatbot |
| 15 | `/load_llm` | POST | chat | Placeholder carga LLM |
| 16 | `/unload_llm` | POST | chat | Descarga modelos de VRAM |

---

## 11. Comandos Tauri (Rust)

### Tabla Completa de Comandos

| # | Comando | Módulo | Descripción |
|---|---------|--------|-------------|
| 1 | `check_setup_done` | auth | Verifica si hay admin configurado |
| 2 | `guardar_admin` | auth | Crea admin (Argon2id hash) |
| 3 | `validar_login_admin` | auth | Login de admin |
| 4 | `get_admin_data` | auth | Obtiene datos del admin |
| 5 | `update_admin_data` | auth | Actualiza datos del admin |
| 6 | `guardar_empleado` | auth | Crea empleado |
| 7 | `validar_login_empleado` | auth | Login de empleado |
| 8 | `get_inventory` | inventory | Lista todos los productos |
| 9 | `add_inventory_item` | inventory | Agrega producto + embedding |
| 10 | `update_inventory_item` | inventory | Actualiza producto + embedding |
| 11 | `delete_inventory_item` | inventory | Elimina producto |
| 12 | `importar_catalogo` | inventory | Importa catálogo + embeddings |
| 13 | `buscar_producto_similar` | inventory | Búsqueda semántica |
| 14 | `get_db_path` | parser | Retorna ruta de la DB |
| 15 | `vincular_inventario` | parser | Vincula productos |
| 16 | `guardar_vinculacion` | parser | Guarda vinculaciones |
| 17 | `descargar_modelos` | parser | Descarga modelos de VRAM |
| 18 | `get_tickets` | tickets | Historial de ventas |
| 19 | `get_cortes` | tickets | Cortes de caja |
| 20 | `guardar_ticket_parseado` | tickets | Guarda ticket parseado |
| 21 | `get_ai_status` | ai | Estado del sidecar |
| 22 | `leer_archivo_raw` | parser_rs | Lee archivo como texto |
| 23 | `leer_archivo_bytes` | parser_rs | Lee archivo como bytes |
| 24 | `parsear_catalogo` | parser_rs | Parser CSV local |
| 25 | `parsear_catalogo_csv` | parser_rs | Parser CSV |
| 26 | `parsear_catalogo_visual` | parser_rs | Parser visual |
| 27 | `parsear_excel` | parser_rs | Parser Excel (→ Python) |
| 28 | `parsear_ticket` | parser_rs | Parser ticket |
| 29 | `analizar_ticket_llm` | parser_rs | Análisis LLM |
| 30 | `analizar_ticket_con_ia` | parser_rs | Análisis IA (→ Python) |
| 31 | `parsear_con_mapeo` | parser_rs | Parseo con mapeo |
| 32 | `parsear_carpeta` | parser_rs | Procesamiento de carpeta |
| 33 | `parsear_carpeta_stream` | parser_rs | Procesamiento con SSE |

---

## 12. Historial de Implementación

*(Historial de fases consolidado en Git.)*
## 13. Problemas Resueltos

*(Problemas menores resueltos, ver commits.)*