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

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Y.A.R.V.I.S. POS  ARQUITECTURA                    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────┐    IPC (invoke)    ┌──────────────┐ │
│  │   React Frontend (Vite)     │ ◄═════════════════►│  Tauri Shell │ │
│  │                             │                     │  (WebView)   │ │
│  │  App.tsx → Routing          │                     └──────┬───────┘ │
│  │  AdminDashboard → 8 tabs    │                            │         │
│  │  EmployeeDashboard → POS    │                   Tauri Commands     │
│  │  ThemeContext (dark/light)   │                     ┌──────┴───────┐ │
│  └─────────────────────────────┘                     │  Rust Core   │ │
│                                                       │              │ │
│                                                       │  db.rs       │ │
│                                                       │  models.rs   │ │
│                                                       │  commands/*  │ │
│                                                       │  parser_rs/* │ │
│                                                       └──────┬───────┘ │
│                                                              │         │
│                                              ┌───────────────┤         │
│                                              │               │         │
│                                    SQLite (sqlx)    HTTP (reqwest)     │
│                                    ┌──────────┐     ┌──────────────┐  │
│                                    │ yarvis.db│     │ Python Sidecar│  │
│                                    │ (WAL)    │     │ (FastAPI)    │  │
│                                    │          │     │              │  │
│                                    │ 8 tables │     │ /embeddings  │  │
│                                    │ usuarios │     │ /predictions │  │
│                                    │ productos│     │ /parser      │  │
│                                    │ ventas   │     │ /carpeta     │  │
│                                    │ cortes   │     │ /matching    │  │
│                                    │ KB+emb   │     │ /chat        │  │
│                                    └──────────┘     └──────┬───────┘  │
│                                                             │          │
│                                              ┌──────────────┤          │
│                                              │              │          │
│                                    Sentence-Transformers   LlamaCPP   │
│                                    all-MiniLM-L6-v2     Qwen 0.5B/1.7B│
│                                    (384-dim embeddings)   (ticket     │
│                                    cosine similarity)      parsing)   │
│                                         │                    │         │
│                                    Facebook Prophet                   │
│                                    (sales forecasting)                │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

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

```
src/
├── main.tsx                          # Entry point de React
├── App.tsx                           # Router principal (Setup → Login → Dashboard)
├── App.css                           # Estilos globales + variables CSS para temas
│
├── hooks/
│   ├── ThemeContext.tsx               # Context para tema claro/oscuro/sistema
│   └── useTheme.ts                   # Hook para persistir tema en localStorage
│
├── front-admin/
│   ├── types.ts                      # Interfaces: ColumnMapping, LLMAnalysis
│   ├── AdminDashboard.tsx            # Panel admin con 8 tabs en sidebar
│   ├── PrimerInicio.tsx              # Wizard de primera ejecución
│   └── ventanas/
│       ├── Inventario.tsx            # CRUD de productos con búsqueda y alertas
│       ├── Configuracion.tsx         # Settings + 3 modos de parseo de archivos
│       ├── ColumnMapper.tsx          # Mapeo de columnas asistido por IA
│       ├── BatchProcessor.tsx        # Procesamiento por lotes de tickets
│       └── Tickets.tsx              # Historial de ventas + cortes de caja
│
└── front-empleado/
    └── EmployeeDashboard.tsx         # Interfaz POS del empleado
```

### 3.2 Backend Rust (`yarvis-app/src-tauri/src/`)

```
src-tauri/src/
├── main.rs                           # Entry point binario
├── lib.rs                            # Setup de Tauri: SQLite, sidecar, comandos
├── models.rs                         # Modelos de datos compartidos
├── db.rs                             # Inicialización de SQLite (8 tablas)
├── sidecar.rs                        # Ciclo de vida del sidecar Python
│
├── commands/
│   ├── mod.rs                        # Re-exports de módulos
│   ├── auth.rs                       # Autenticación (Argon2id)
│   ├── tickets.rs                    # Comandos de ventas/tickets
│   ├── inventory.rs                  # CRUD de inventario + embeddings
│   ├── parser.rs                     # Puente al sidecar Python
│   └── ai.rs                         # Estado del motor de IA
│
└── parser_rs/
    ├── mod.rs                        # Re-exports de módulos
    ├── utils.rs                      # Sanitización de rutas
    ├── parser_csv.rs                 # Parser CSV local (Rust)
    ├── parser_excel.rs               # Parser Excel (delega a Python)
    └── parser_txt.rs                 # Parser TXT/tickets (delega a Python)
```

### 3.3 Backend Python (`yarvis-IA/`)

```
yarvis-IA/
├── main.py                           # Entry point FastAPI
├── requirements.txt                  # Dependencias Python
│
├── endpoints/
│   ├── embeddings.py                 # /generar_embedding, /buscar_similar
│   ├── predictions.py                # /recalcular_predicciones (Prophet)
│   ├── parser.py                     # /analizar_ticket, /parsear_con_mapeo, /parsear_excel
│   ├── carpeta.py                    # /parsear_carpeta, /parsear_carpeta_stream (SSE)
│   ├── matching.py                   # /vincular_inventario, /guardar_vinculacion
│   └── chat.py                       # /chat, /load_llm, /unload_llm
│
├── core/
│   ├── embeddings.py                 # Motor de embeddings (MiniLM)
│   └── utils.py                      # limpiar_producto(), es_categoria()
│
├── parser_py/
│   ├── __init__.py                   # Re-exports de parsers
│   ├── parser_excel.py               # Parser Excel (openpyxl)
│   ├── parser_csv.py                 # Parser CSV auto-detect
│   └── parser_txt.py                 # Parser visual de formato de tabla
│
└── modelos/
    ├── qwen/
    │   ├── rutas.py                  # Rutas a modelos GGUF
    │   └── parser_llm.py            # Parser LLM: Qwen 0.5B → 1.7B fallback
    └── profeta/
        └── predictor.py             # Predictor de ventas con Prophet
```

---

## 4. Frontend (React + TypeScript)

### 4.1 `types.ts` — Interfaces Compartidas

```typescript
export interface ColumnMapping {
  cantidad: number | null;       // Índice de columna de cantidad
  producto: number[] | null;     // Rango de columnas de producto (ej: [1, 2])
  precio_unitario: number | null; // Índice de columna de precio
  total: number | null;          // Índice de columna de total
  descuento: number | null;      // Índice de columna de descuento
}

export interface LLMAnalysis {
  status: string;
  mapeo: {
    formato_detectado: string;   // Ej: "CANTIDAD PRODUCTO PRECIO TOTAL"
    columnas: ColumnMapping;
    delimitador: string;         // Ej: "espacios_multiples"
    moneda: string;              // Ej: "$"
    total_columnas: number;
    tiene_descuento: boolean;
    tiene_iva: boolean;
  };
  ejemplo_parseado: any[];       // Items parseados por la IA
  confianza: number;             // 0.0 - 1.0
  notas: string;                 // Observaciones de la IA
  reintentado_con: string | null; // "qwen3_1_7b" si se reintentó
}
```

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
```rust
pub struct AiSidecar {
    pub port: Mutex<Option<u16>>,          // Puerto del motor de IA
    pub port_llm: Mutex<Option<u16>>,     // Puerto reservado para LLM futuro
    pub process: Mutex<Option<Child>>,     // Proceso Python hijo
    pub status: Mutex<AiStatus>,           // NotRunning | Starting | Ready | Error
    pub http_client: reqwest::Client,      // Cliente HTTP compartido
}
```

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

```rust
pub struct AdminData { id, nombre, tienda, pass_hash, ubicacion, cp }
pub struct TicketItem { id, venta_id, producto_nombre, cantidad, precio_unitario, descuento, subtotal }
pub struct InventoryItem { id, nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, codigo_barras, categoria }
pub struct EmbeddingResponse { status, dimensions, blob_b64 }
pub struct SimilarSearchResponse { status, results }
pub struct SimilarResult { id, nombre, similitud, precio_venta, stock }
pub struct DbPath(pub String)  // Ruta de la DB, managed state
```

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

```python
# Registra 6 routers:
# - embeddings (/generar_embedding, /buscar_similar, /insertar_knowledge)
# - predictions (/recalcular_predicciones)
# - parser (/analizar_ticket, /parsear_con_mapeo, /parsear_excel, etc.)
# - carpeta (/parsear_carpeta, /parsear_carpeta_stream)
# - matching (/vincular_inventario, /guardar_vinculacion)
# - chat (/chat, /load_llm, /unload_llm)

PORT = int(sys.argv[1])  # Recibe puerto de Rust
uvicorn.run(app, host="127.0.0.1", port=PORT)
```

**15 endpoints totales.**

### 6.2 `endpoints/parser.py` — Endpoints de Parseo

| Endpoint | Método | Descripción |
|----------|--------|-------------|
| `/analizar_ticket` | POST | Recibe `{"texto": "..."}`, llama al LLM, retorna mapeo + ejemplo_parseado. **Descarga modelos al terminar.** |
| `/parsear_con_mapeo` | POST | Parsea texto usando mapeo de columnas del usuario (sin LLM) |
| `/parsear_catalogo_visual` | POST | Parsea catálogo en formato visual de tabla |
| `/parsear_excel` | POST | Recibe bytes de Excel, retorna productos detectados |

### 6.3 `endpoints/carpeta.py` — Procesamiento por Lotes

| Endpoint | Método | Descripción |
|----------|--------|-------------|
| `/parsear_carpeta` | POST | Procesa carpeta sincrónicamente, retorna stats |
| `/parsear_carpeta_stream` | POST | Procesa carpeta con SSE streaming (batches de 50) |

**Descarga modelos de VRAM antes de empezar** para liberar memoria.

**Flujo de `/parsear_carpeta_stream`:**
1. Carga estado de productos existentes de la DB
2. Procesa archivos en batches de 50
3. Para cada archivo: lee texto, parsea lineas, inserta venta en SQLite
4. Yield de eventos SSE con progreso después de cada batch
5. Evento final con estadísticas completas

### 6.4 `endpoints/embeddings.py` — Embeddings

| Endpoint | Método | Descripción |
|----------|--------|-------------|
| `/generar_embedding` | POST | `{"texto": "..."}` → vector 384d → base64 |
| `/buscar_similar` | POST | Búsqueda por cosine similarity en knowledge_base |
| `/insertar_knowledge` | POST | Inserta item en knowledge_base |

### 6.5 `endpoints/chat.py` — Chat y Modelos

| Endpoint | Método | Descripción |
|----------|--------|-------------|
| `/chat` | POST | Placeholder para chatbot |
| `/load_llm` | POST | Placeholder para cargar modelo |
| `/unload_llm` | POST | **Descarga modelos Qwen de VRAM** via `descargar_modelos()` |

### 6.6 `endpoints/matching.py` — Vinculación de Productos

| Endpoint | Método | Descripción |
|----------|--------|-------------|
| `/vincular_inventario` | POST | Vincula productos parseados con inventario (exact match + embeddings) |
| `/guardar_vinculacion` | POST | Guarda las vinculaciones en detalle_ventas |

**Algoritmo de vinculación:**
1. Match exacto por nombre normalizado + precio
2. Si no hay match exacto, usa embedding similarity (umbral 0.85)
3. Retorna productos vinculados y sin vincular

---

## 7. Modelos de IA

### 7.1 Qwen LLM (`modelos/qwen/parser_llm.py`)

**Modelos:**
- **Qwen 2.5 0.5B** (Q4_K_M GGUF) — Modelo principal, rápido
- **Qwen 3 1.7B** (Q4_K_M GGUF) — Modelo de respaldo, más preciso

**Flujo de análisis:**
1. Intenta con Qwen 0.5B
2. Si confianza < 0.8, reintenta con Qwen 1.7B
3. Si 0.5B falla, usa 1.7B directamente
4. **Descarga modelos después de cada análisis** para liberar VRAM

**Parámetros de carga:**
```python
Llama(
    model_path=...,
    n_ctx=4096,
    n_gpu_layers=-1,  # Todos los layers en GPU
    n_threads=4,
    verbose=False
)
```

**Función `descargar_modelos()`:**
```python
def descargar_modelos():
    global _llm_0_5, _llm_1_7
    if _llm_0_5 is not None:
        del _llm_0_5
        _llm_0_5 = None
    if _llm_1_7 is not None:
        del _llm_1_7
        _llm_1_7 = None
    gc.collect()
```

**Prompt del sistema:**
Le pide a la IA que analice un ticket y retorne JSON con:
- `mapeo.columnas` — Índices de cada columna detectada
- `ejemplo_parseado` — Items parseados de ejemplo
- `confianza` — Nivel de confianza (0-1)
- `notas` — Observaciones

### 7.2 Embeddings (`core/embeddings.py`)

**Modelo:** all-MiniLM-L6-v2 (sentence-transformers)
- Dimensiones: 384
- Normalización: L2
- Uso: Búsqueda semántica de productos

**Funciones:**
- `texto_a_embedding(texto)` → vector 384d
- `embedding_a_blob(vec)` → bytes para SQLite
- `blob_a_embedding(blob)` → vector 384d
- `cosine_similarity(a, b)` → float

### 7.3 Prophet (`modelos/profeta/predictor.py`)

**Modelo:** Facebook Prophet
- Entrena con `ventas_diarias`
- Genera predicciones N días hacia adelante
- Incluye intervalos de confianza

---

## 8. Base de Datos

### Esquema SQLite

```sql
-- Usuarios (admin y empleados)
CREATE TABLE usuarios (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    tienda TEXT,
    pass_hash TEXT NOT NULL,
    rol TEXT DEFAULT 'empleado',
    ubicacion TEXT,
    cp TEXT
);

-- Productos (inventario)
CREATE TABLE productos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    descripcion TEXT,
    precio_costo REAL DEFAULT 0,
    precio_venta REAL DEFAULT 0,
    stock REAL DEFAULT 0,
    stock_minimo REAL DEFAULT 5,
    codigo_barras TEXT,
    categoria TEXT
);

-- Ventas (cabeceras)
CREATE TABLE ventas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    total REAL NOT NULL,
    subtotal REAL,
    iva REAL,
    cajero TEXT,
    metodo_pago TEXT DEFAULT 'efectivo',
    estado TEXT DEFAULT 'completada',
    fecha DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Detalle de ventas (líneas)
CREATE TABLE detalle_ventas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    venta_id INTEGER,
    producto_nombre TEXT,
    producto_id INTEGER,
    cantidad REAL,
    precio_unitario REAL,
    descuento REAL DEFAULT 0,
    subtotal REAL,
    FOREIGN KEY (venta_id) REFERENCES ventas(id),
    FOREIGN KEY (producto_id) REFERENCES productos(id)
);

-- Ventas diarias (para Prophet)
CREATE TABLE ventas_diarias (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha DATE UNIQUE,
    total_ventas REAL,
    num_transacciones INTEGER
);

-- Cortes de caja
CREATE TABLE cortes_caja (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha_inicio DATETIME,
    fecha_fin DATETIME,
    total_ventas REAL,
    total_efectivo REAL,
    total_tarjeta REAL,
    observaciones TEXT
);

-- Predicciones futuras
CREATE TABLE predicciones_futuras (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha DATE,
    prediccion REAL,
    lower REAL,
    upper REAL
);

-- Knowledge base (embeddings)
CREATE TABLE knowledge_base (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contenido TEXT,
    categoria TEXT,
    embedding BLOB
);
```

---

## 9. Flujo de Datos

### 9.1 Flujo de Parseo de Ticket (entrenar IA)

```
Usuario carga .txt
    ↓
Configuracion.tsx → handleFileSelect()
    ↓
invoke("leer_archivo_raw", { path }) → Rust lee el archivo
    ↓
fileContent se guarda en estado
    ↓
ColumnMapper aparece
    ↓
Usuario hace clic "Analizar con IA"
    ↓
invoke("analizar_ticket_con_ia", { texto }) → Rust → Python /analizar_ticket
    ↓
Python: analizar_ticket() → carga Qwen 0.5B → analiza → retorna JSON
    ↓
Si confianza < 0.8: reintenta con Qwen 1.7B
    ↓
Python: descargar_modelos() → libera VRAM
    ↓
Rust retorna LLMAnalysis al frontend
    ↓
ColumnMapper muestra dropdowns de ajuste + badges de confianza
    ↓
useEffect pasa ejemplo_parseado → setParsedItems en Configuracion
    ↓
Tabla "Previsualización de Datos Estructurados" muestra items
    ↓
Usuario hace clic "Guardar Ticket"
    ↓
handleGuardarTicket() → invoke("guardar_ticket_parseado", { items, total })
    ↓
Rust: guardar_ticket_parseado() → INSERT en ventas + detalle_ventas
    ↓
Se actualizan flags: iaTrained=true, ticketsParsed=true
```

### 9.2 Flujo de Importación de Catálogo

```
Usuario carga .xlsx/.csv/.txt en modo "catalogo"
    ↓
handleFileSelect() detecta extensión
    ↓
Si .xlsx: invoke("leer_archivo_bytes") → invoke("parsear_excel") → Python
Si .txt/.csv: invoke("parsear_catalogo_visual") → Python
    ↓
Productos se guardan en parsedItems
    ↓
Tabla muestra: Nombre, Costo, Venta, Categoría
    ↓
Usuario hace clic "Entrenar IA con Catálogo"
    ↓
handleTrainIA() → invoke("importar_catalogo", { items })
    ↓
Rust: INSERT en productos + genera embeddings en background
    ↓
catalogParsed=true
```

### 9.3 Flujo de Procesamiento por Lotes

```
Usuario selecciona carpeta en modo "insertar"
    ↓
BatchProcessor aparece
    ↓
invoke("parsear_carpeta_stream", { carpeta, mapeo, dbPath })
    ↓
Rust → Python /parsear_carpeta_stream
    ↓
Python: descargar_modelos() → libera VRAM
    ↓
Procesa archivos en batches de 50
    ↓
Para cada archivo:
    - Lee texto
    - Parsea lineas con _parsear_linea() (rule-based, sin LLM)
    - INSERT venta + detalle_ventas en SQLite
    ↓
Yield eventos SSE con progreso
    ↓
Frontend muestra: procesados, exitosos, errores, ventas creadas
    ↓
Al terminar: muestra productos nuevos
    ↓
Usuario puede "Vincular con Inventario Existente"
    ↓
invoke("vincular_inventario") → Python /vincular_inventario
    ↓
Match exacto + embedding similarity
```

---

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

### Fase 1: Estructura Base
- Creación de la app Tauri con React + Rust
- Setup de SQLite con 8 tablas
- Sistema de autenticación con Argon2id
- CRUD de inventario básico
- Panel de configuración

### Fase 2: Motor de IA (Python Sidecar)
- Implementación del sidecar Python con FastAPI
- Detección automática de puertos libres
- Health check polling
- CUDA LD_LIBRARY_PATH para GPU
- Shutdown graceful al cerrar la app

### Fase 3: Parser de Tickets
- Parser visual de formato de tabla (`parser_txt.py`)
- Parser CSV auto-detect (`parser_csv.py`)
- Parser Excel con openpyxl (`parser_excel.py`)
- Endpoints: `/parsear_catalogo_visual`, `/parsear_excel`, `/parsear_con_mapeo`

### Fase 4: LLM para Tickets
- Integración de Qwen 2.5 0.5B y Qwen 3 1.7B
- Sistema de fallback: 0.5B → 1.7B si confianza < 0.8
- Prompt optimizado para tickets mexicanos
- Extracción de JSON con mapeo de columnas
- Endpoint `/analizar_ticket`

### Fase 5: Frontend de Parseo
- `Configuracion.tsx` con 3 modos (catalogo, entrenar IA, insertar)
- `ColumnMapper.tsx` con dropdowns ajustables
- Indicador de confianza y badges de la IA
- Tabla de previsualización

### Fase 6: Procesamiento por Lotes
- `BatchProcessor.tsx` con SSE streaming
- Procesamiento en batches de 50
- Inserción masiva en SQLite
- Productos nuevos vs existentes

### Fase 7: Embeddings y Búsqueda Semántica
- Integración de all-MiniLM-L6-v2 (384 dims)
- Generación automática de embeddings al insertar productos
- Búsqueda semántica con cosine similarity
- Vinculación de productos parseados con inventario existente

### Fase 8: Gestión de VRAM
- Función `descargar_modelos()` en Python
- Endpoint `/unload_llm`
- Comando Tauri `descargar_modelos`
- Auto-descarga antes de batch processing
- Auto-descarga después de análisis individual

### Fase 9: Correcciones y Optimizaciones (Sesión Actual)

#### Fix: Error `/generar_embedding`
- **Problema:** El error se imprimía 4 veces y el sidecar no estaba listo
- **Causa:** `generate_and_store_embedding` solo verificaba `base_url()` pero no `get_status()`
- **Solución:** Agregué `check_process_alive()` y verificación de `AiStatus::Ready` antes de cada llamada HTTP
- **Archivos:** `inventory.rs`, `sidecar.rs`

#### Fix: Tipo `producto` incorrecto
- **Problema:** La IA retornaba `"producto": 2` (número) pero el frontend esperaba `[2]` (array)
- **Causa:** El prompt del LLM pedía `INDICE` singular, pero `ColumnMapping` esperaba `number[]`
- **Solución:** Normalización en `handleAnalizar`: `Array.isArray() ? ... : [value]`
- **Archivo:** `ColumnMapper.tsx`

#### Fix: Preview vacía
- **Problema:** "Previsualización de Datos Estructurados" mostraba "Sin datos para previsualizar"
- **Causa:** `previewItems` re-parseaba las primeras 10 líneas con `parsearLinea`, pero `esLineaUtil` filtraba metadata del ticket (fecha, cajero, subtotal, etc.)
- **Solución:** Cambiar `previewItems` para usar `analysis.ejemplo_parseado` (los items que la IA ya parseó)
- **Archivo:** `ColumnMapper.tsx`

#### Fix: Botones redundantes
- **Problema:** "Aceptar Mapeo" y "Guardar Ticket Analizado" eran 2 pasos separados
- **Solución:** Unificar en un solo botón "Guardar Ticket" que hace ambas cosas
- **Archivos:** `ColumnMapper.tsx`, `Configuracion.tsx`

#### Fix: Catálogo perdido al cambiar de modo
- **Problema:** Al cambiar entre modos se borraban `parsedItems` y `fileContent`
- **Solución:** Estados `lastCatalogPath` y `lastCatalogItems` para persistir el último catálogo
- **Archivo:** `Configuracion.tsx`

#### Fix: Error de embedding ruidoso
- **Problema:** El error `/generar_embedding` se imprimía 4 veces (1 por item)
- **Solución:** `AtomicBool` para imprimir solo 1 vez
- **Archivo:** `inventory.rs`

#### Fix: Layout de previsualización
- **Problema:** La tabla quedaba debajo del ColumnMapper después de analizar
- **Solución:** Mover "Previsualización de Datos Estructurados" ANTES del ColumnMapper
- **Archivo:** `Configuracion.tsx`

---

## 13. Problemas Resueltos

### 13.1 GPU/VRAM
- **Problema:** Los modelos Qwen se quedaban en VRAM después de usarlos
- **Solución:** Auto-descarga con `descargar_modelos()` + endpoint `/unload_llm`
- **Resultado:** Los modelos se cargan solo cuando se necesitan y se descargan al terminar

### 13.2 Procesos Huérfanos
- **Problema:** El proceso Python no se mataba al cerrar la app
- **Solución:** `shutdown_ai_engine()` en `WindowEvent::Destroyed` + `check_process_alive()` para detectar procesos muertos

### 13.3 Parser No Encontraba Productos
- **Problema:** `esLineaUtil` filtraba líneas válidas que contenían palabras como "total", "precio", etc.
- **Solución:** Usar `ejemplo_parseado` del LLM en vez de re-parsear con reglas

### 13.4 Mapeo de Columnas Incorrecto
- **Problema:** El dropdown de Producto mostraba "No detectada" aunque la IA lo detectó
- **Solución:** Normalizar `producto` de número a array

---

*Documentación generada el 2026-07-04.*
*Última actualización: Sesión de correcciones y optimización del parseador de IA.*
