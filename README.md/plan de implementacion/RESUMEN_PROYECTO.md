# Y.A.R.V.I.S. POS - Documentación Completa del Proyecto

## Índice
1. [Descripción General](#1-descripción-general)
2. [Arquitectura del Sistema](#2-arquitectura-del-sistema)
3. [Estructura del Proyecto](#3-estructura-del-proyecto)
4. [Componentes Principales](#4-componentes-principales)
5. [Sistema de Parsing](#5-sistema-de-parsing)
6. [Inventario](#6-inventario)
7. [Procesamiento Masivo (Batch)](#7-procesamiento-masivo-batch)
8. [Deduplicación](#8-deduplicación)
9. [Gestión de Estado](#9-gestión-de-estado)
10. [Modelos de LLM](#10-modelos-de-llm)
11. [Base de Datos](#11-base-de-datos)
12. [Bug Fixes y Mejoras](#12-bug-fixes-y-mejoras)
13. [Estado Actual](#13-estado-actual)

---

## 1. Descripción General

Y.A.R.V.I.S. POS es un sistema punto de venta con inteligencia artificial que permite:
- Importar catálogos de productos desde archivos TXT, CSV y Excel
- Parsear tickets de venta automáticamente usando LLM
- Procesar miles de tickets en modo batch
- Gestionar inventario con seguimiento de ventas
- Predecir demanda con Prophet (análisis temporal)

---

## 2. Arquitectura del Sistema

```
┌─────────────────────────────────────────────────────────────┐
│                    FRONTEND (React + TypeScript)             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │  Inventario  │ │   Tickets   │ │  Ajustes    │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    RUST TAURI (Backend)                      │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │  Comandos   │ │  Parser RS  │ │   DB (SQLx)  │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                 PYTHON SIDECAR (FastAPI)                     │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │ Parser LLM  │ │  Carpy.py   │ │  Modelos    │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
```

### Comunicación
- **Frontend → Rust**: Invocación de comandos Tauri
- **Rust → Python**: HTTP requests al sidecar FastAPI
- **Python**: Ejecuta modelos LLM (Qwen) y parsing de tickets

---

## 3. Estructura del Proyecto

```
Y.A.R.V.I.S.-POS/
├── yarvis-app/                    # Frontend + Rust
│   ├── src/                       # Código React
│   │   ├── front-admin/
│   │   │   └── ventanas/          # Componentes de UI
│   │   │       ├── Inventario.tsx
│   │   │       ├── Tickets.tsx
│   │   │       ├── Configuracion.tsx
│   │   │       ├── BatchProcessor.tsx
│   │   │       ├── ColumnMapper.tsx
│   │   │       └── CatalogosParseados.tsx
│   │   ├── hooks/
│   │   │   ├── ParserContext.tsx   # Estado global del parser
│   │   │   └── ThemeContext.tsx
│   │   └── App.tsx
│   └── src-tauri/                 # Código Rust
│       ├── src/
│       │   ├── commands/
│       │   │   ├── inventory.rs   # CRUD inventario
│       │   │   ├── tickets.rs     # Guardar tickets
│       │   │   └── parser.rs      # Parsing
│       │   ├── parser_rs/         # Parser Rust nativo
│       │   │   ├── parser_txt.rs
│       │   │   ├── parser_csv.rs
│       │   │   └── parser_excel.rs
│       │   ├── db.rs              # Inicialización DB
│       │   ├── models.rs          # Estructuras de datos
│       │   ├── sidecar.rs         # Comunicación Python
│       │   └── lib.rs
│       └── Cargo.toml
└── yarvis-IA/                     # Backend Python
    ├── main.py                    # FastAPI server
    ├── endpoints/
    │   ├── carpeta.py             # Batch processing
    │   └── parser.py              # Parsing de archivos
    ├── parser_py/                 # Parser Python modular
    │   ├── parser_txt.py
    │   ├── parser_csv.py
    │   └── parser_excel.py
    └── modelos/
        └── qwen/
            ├── parser_llm.py      # Análisis con LLM
            └── rutas.py           # Rutas de modelos
```

---

## 4. Componentes Principales

### 4.1 Configuracion.tsx
**Ubicación**: `yarvis-app/src/front-admin/ventanas/Configuracion.tsx`

**Función**: Panel principal de configuración e importación de datos.

**Características**:
- Selector de modo: "Entrenar IA", "Catálogo", "Insertar"
- Botón para seleccionar archivo/carpeta
- Visualizador de datos raw
- Preview de productos parseados
- Indicadores de estado:
  - `catalogParsed`: ¿Catálogo importado?
  - `iaTrained`: ¿IA entrenada?
  - `ticketsParsed`: ¿Tickets parseados?
  - `ticketsGuardados`: Número de tickets guardados
  - `ticketsCount`: Total productos parseados

**Flujos**:
1. **Modo Catálogo**: Selecciona archivo → Parsea productos → Importa al inventario
2. **Modo Entrenar IA**: Selecciona ticket → LLM analiza → Guarda en DB
3. **Modo Insertar**: Selecciona carpeta → Batch processor procesa todos los tickets

### 4.2 BatchProcessor.tsx
**Ubicación**: `yarvis-app/src/front-admin/ventanas/BatchProcessor.tsx`

**Función**: Procesamiento masivo de tickets con progreso en tiempo real.

**Características**:
- Selector de carpeta
- Barra de progreso SSE
- Estadísticas: procesados, exitosos, errores, ventas creadas
- Lista de productos nuevos detectados
- Botón para vincular productos nuevos al inventario

**Props**:
```typescript
interface BatchProcessorProps {
  onVolver: () => void;
  initialFolder?: string;  # Carpeta preseleccionada desde Configuracion
}
```

### 4.3 ColumnMapper.tsx
**Ubicación**: `yarvis-app/src/front-admin/ventanas/ColumnMapper.tsx`

**Función**: Interfaz para mapear columnas de tickets manualmente o con IA.

**Características**:
- Mapeo automático por LLM
- Mapeo manual con dropdowns
- Preview de items parseados
- Botón "Guardar Ticket" unificado

### 4.4 Inventario.tsx
**Ubicación**: `yarvis-app/src/front-admin/ventanas/Inventario.tsx`

**Función**: Gestión de inventario de productos.

**Columnas**:
- Nombre
- Stock (editable)
- Vendido (total vendido, editable)
- Costo (editable)
- Venta (editable)
- Categoría

### 4.5 CatalogosParseados.tsx
**Ubicación**: `yarvis-app/src/front-admin/ventanas/CatalogosParseados.tsx`

**Función**: Muestra catálogos previamente importados.

**Características**:
- Lista de catálogos con fecha y total de productos
- Botón "Recargar" para volver a importar
- SHA256 hash para detectar duplicados

---

## 5. Sistema de Parsing

### 5.1 Parser de Catálogos

#### TXT (`parser_py/parser_txt.py`)
**Patrones soportados**:
```python
# Formato con separador
_PATRON_CON_SEP_CANT    # "10 Coca Cola -- $15 -- $8"
_PATRON_CON_SEP         # "Coca Cola -- $15 -- $8"

# Formato sin separador (espacios)
_PATRON_SIN_SEP_CANT    # "10Coca Cola $15 $8"
_PATRON_SIN_SEP         # "Coca Cola $15 $8"
_PATRON_SIN_SEP_CANT_SINDOL  # "10Coca Cola 15 8"
_PATRON_SIN_SEP_SINDOL  # "Coca Cola 15 8"
```

**Detección de stock**:
- Número al inicio de la línea: `10Coca Cola $15 $8` → stock=10
- Columna "stock" en CSV
- Excel: columna detectada por nombre

#### CSV (`parser_py/parser_csv.py`)
**Detección automática de columnas**:
- Busca headers: `stock`, `existencia`, `cantidad`, `inventario`, `qty`, `quantity`
- Soporta delimitadores: `,`, `;`, `\t`
- Precios con `$` o sin él

#### Excel (`parser_rs/parser_excel.rs`)
- Lectura de bytes via Tauri
- Parseo con `calamine`
- Soporta múltiples hojas

### 5.2 Parser de Tickets con LLM

**Flujo** (`modelos/qwen/parser_llm.py`):
```
1. Intentar con Qwen 0.5B (rápido, menos preciso)
2. Si confianza < 0.7 → Reintentar con 0.8B
3. Si confianza < 0.7 → Reintentar con 1.7B (lento, más preciso)
```

**Estructura de respuesta**:
```json
{
  "formato_detectado": "ticket_super",
  "confianza": 0.92,
  "mapeo": {
    "total_columnas": 5,
    "delimitador": "espacios",
    "columnas": {
      "cantidad": 0,
      "producto": 1,
      "precio_unitario": 2,
      "descuento": null,
      "total": 4
    },
    "tiene_descuento": false,
    "tiene_iva": true
  },
  "ejemplo_parseado": [...],
  "notas": "Ticket de supermercado con IVA incluido"
}
```

### 5.3 Parser Rust Nativo

**Archivos**:
- `parser_txt.rs`: Lee archivos TXT raw
- `parser_csv.rs`: Parsea CSV con `csv` crate
- `parser_excel.rs`: Parsea Excel con `calamine`

**Ventaja**: Más rápido que Python para archivos simples.

---

## 6. Inventario

### 6.1 Estructura de Producto
```sql
CREATE TABLE productos (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  nombre TEXT NOT NULL,
  descripcion TEXT,
  precio_costo REAL DEFAULT 0,
  precio_venta REAL DEFAULT 0,
  stock REAL DEFAULT 0,
  vendido REAL DEFAULT 0,        -- Total vendido
  stock_minimo REAL DEFAULT 5,
  codigo_barras TEXT,
  categoria TEXT,
  fecha_agregado DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 6.2 Operaciones

**Agregar producto** (`inventory.rs`):
```rust
pub async fn add_inventory_item(
  db: State<'_, Arc<DbPath>>,
  item: InventoryItem,
) -> Result<String, String>
```

**Actualizar producto**:
```rust
pub async fn update_inventory_item(
  db: State<'_, Arc<DbPath>>,
  item: InventoryItem,
) -> Result<String, String>
```

**Auto-descuento al guardar ticket** (`tickets.rs`):
```sql
UPDATE productos 
SET stock = MAX(0, stock - ?)
WHERE LOWER(nombre) = LOWER(?)
```

**Incrementar vendido**:
```sql
UPDATE productos 
SET vendido = vendido + ?
WHERE LOWER(nombre) = LOWER(?)
```

### 6.3 Matching Case-Insensitive
Todas las comparaciones de nombre usan `LOWER()`:
```sql
WHERE LOWER(nombre) = LOWER(?)
```
Esto permite encontrar "COCA COLA" aunque en la DB esté "Coca Cola".

---

## 7. Procesamiento Masivo (Batch)

### 7.1 Flujo
1. Usuario selecciona carpeta con tickets
2. Rust llama a Python sidecar via HTTP
3. Python procesa cada archivo .txt
4. SSE envía progreso en tiempo real
5. Frontend muestra barra de progreso
6. Al terminar, productos nuevos se listan
7. Usuario puede vincular productos al inventario

### 7.2 Mapeo de Columnas
```rust
// Mapeo dinámico con índices negativos
{
  cantidad: -3,           // 3ra columna desde el final
  producto: [0, -4],      // Palabras de la 1ra columna hasta 4to desde el final
  precio_unitario: -2,    // 2da columna desde el final
  total: -1               // Última columna
}
```

**Ventaja**: Funciona con cualquier número de columnas de producto.

### 7.3 Preprocesamiento
```python
def _preprocesar_linea(linea: str) -> str:
    # Une $ con el número: "$ 18" → "$18"
    return re.sub(r'\$\s+', '$', linea)
```

### 7.4 Filtro de Líneas No Útiles
```python
def _es_linea_util(linea: str) -> bool:
    # Filtra:
    # - miscelánea
    # - av. (avenida)
    # - tel. (teléfono)
    # - total, pago:, efectivo, tarjeta
    # - #NNNNN (números de ticket)
    # - Líneas muy cortas (< 5 chars)
```

### 7.5 Respuesta SSE
```json
{
  "type": "progress",
  "procesados": 150,
  "total": 12000,
  "exitosos": 145,
  "errores": 5,
  "ventas_creadas": 145,
  "items_insertados": 450,
  "productos_nuevos": 23,
  "productos_existentes": 122,
  "duplicados_detectados": 3,
  "productos_nuevos_lista": ["Producto A", "Producto B"]
}
```

---

## 8. Deduplicación

### 8.1 Catálogos (SHA256)
**Tabla**:
```sql
CREATE TABLE catalogos_importados (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  hash TEXT UNIQUE NOT NULL,
  ruta_archivo TEXT NOT NULL,
  fecha_importacion DATETIME DEFAULT CURRENT_TIMESTAMP,
  total_productos INTEGER DEFAULT 0
);
```

**Flujo**:
1. Calcular SHA256 del contenido del archivo
2. Verificar si el hash ya existe en la DB
3. Si existe → Rechazar importación
4. Si no existe → Importar y registrar hash

**Código**:
```rust
fn calcular_hash_catalogo(contenido: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(contenido.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

### 8.2 Productos (Máximo 2)
```sql
-- Contar productos con mismo nombre
SELECT COUNT(*) FROM productos 
WHERE LOWER(nombre) = LOWER(?)

-- Si count >= 2, no insertar
```

**Razón**: Permite variantes intencionales (ej: "Coca Cola 600ml" y "Coca Cola 2L") pero previene duplicados excesivos.

---

## 9. Gestión de Estado

### 9.1 ParserContext
**Ubicación**: `yarvis-app/src/hooks/ParserContext.tsx`

**Estados persistidos**:
```typescript
interface ParserState {
  parsedItems: any[];           // Items parseados
  fileContent: string;          // Contenido del archivo
  selectedPath: string;         // Ruta seleccionada
  parserMode: "catalogo" | "entrenar IA" | "insertar";
  showColumnMapper: boolean;
  llmAnalysis: any | null;      // Resultado del LLM
  lastCatalogPath: string;      // Último catálogo importado
  lastCatalogItems: any[];
  catalogParsed: boolean;
  iaTrained: boolean;
  ticketsParsed: boolean;
  ticketsCount: number;
  ticketsGuardados: number;
}
```

**Persistencia**: localStorage
```typescript
const STORAGE_KEY = "yarvis_parser_state";

function saveToLocalStorage(state: ParserState) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
}

function loadFromLocalStorage(): ParserState {
  const saved = localStorage.getItem(STORAGE_KEY);
  return saved ? { ...defaultState, ...JSON.parse(saved) } : defaultState;
}
```

**Ventaja**: El estado sobrevive:
- Cambios de pestaña (inventario, tickets, ajustes)
- Recarga de página
- Cierre y apertura de la app

### 9.2 Uso en Componentes
```tsx
// En Configuracion.tsx
const {
  parsedItems, setParsedItems,
  selectedPath, setSelectedPath,
  parserMode, setParserMode,
  // ... etc
} = useParserContext();
```

---

## 10. Modelos de LLM

### 10.1 Modelos Disponibles
| Modelo | Tamaño | Velocidad | Precisión |
|--------|--------|-----------|-----------|
| Qwen 0.5B | 400MB | Rápido | Baja |
| Qwen 0.8B | 500MB | Medio | Media |
| Qwen 1.7B | 1GB | Lento | Alta |

### 10.2 Rutas (`modelos/qwen/rutas.py`)
```python
qwen0_5 = "/home/alesito/.cache/lm-studio/models/Qwen/Qwen3-0.6B-Q8_0.gguf"
qwen0_8 = "/home/alesito/.cache/lm-studio/models/Qwen/Qwen3.5-0.8B-Q4_K_M.gguf"
qwen1_7 = "/home/alesito/.cache/lm-studio/models/Qwen/Qwen3-1.7B-Q4_K_M.gguf"
```

### 10.3 VRAM Management
**Carga perezosa**: Los modelos solo se cargan cuando se necesitan.

**Descarga automática**:
- Después de procesamiento batch
- Después de análisis individual
- Endpoint: `POST /unload_llm`

**Código**:
```python
def descargar_modelos():
    global modelo_0_5, modelo_0_8, modelo_1_7
    modelo_0_5 = None
    modelo_0_8 = None
    modelo_1_7 = None
    gc.collect()
```

### 10.4 GPU Auto-Detect
```rust
// sidecar.rs
fn check_gpu_available() -> bool {
    // Detecta NVIDIA GPU
    // Si no hay GPU, usa CPU
}
```

**LD_LIBRARY_PATH**: Se configura automáticamente para encontrar CUDA.

---

## 11. Base de Datos

### 11.1 Esquema Completo

```sql
-- Productos/Inventario
CREATE TABLE productos (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  nombre TEXT NOT NULL,
  descripcion TEXT,
  precio_costo REAL DEFAULT 0,
  precio_venta REAL DEFAULT 0,
  stock REAL DEFAULT 0,
  vendido REAL DEFAULT 0,
  stock_minimo REAL DEFAULT 5,
  codigo_barras TEXT,
  categoria TEXT,
  fecha_agregado DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Ventas
CREATE TABLE ventas (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  fecha DATETIME DEFAULT CURRENT_TIMESTAMP,
  total REAL NOT NULL,
  descuento REAL DEFAULT 0,
  notas TEXT
);

-- Items de venta
CREATE TABLE venta_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  venta_id INTEGER NOT NULL,
  producto TEXT NOT NULL,
  cantidad REAL NOT NULL,
  precio_unitario REAL NOT NULL,
  total REAL NOT NULL,
  FOREIGN KEY (venta_id) REFERENCES ventas(id)
);

-- Catálogos importados (deduplicación)
CREATE TABLE catalogos_importados (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  hash TEXT UNIQUE NOT NULL,
  ruta_archivo TEXT NOT NULL,
  fecha_importacion DATETIME DEFAULT CURRENT_TIMESTAMP,
  total_productos INTEGER DEFAULT 0
);

-- Cache de tickets (LRU)
CREATE TABLE ticket_cache (
  ticket_hash TEXT PRIMARY KEY,
  resultado TEXT NOT NULL,
  fecha_analisis DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 11.2 Inicialización
```rust
// db.rs
pub async fn initialize_db() -> Result<(SqlitePool, String), sqlx::Error> {
    let db_path = get_db_path();
    let pool = SqlitePool::connect(&format!("sqlite:{}?mode=rwc", db_path)).await?;
    
    // Crear tablas si no existen
    sqlx::query(CREATE_TABLES).execute(&pool).await?;
    
    Ok((pool, db_path))
}
```

---

## 12. Bug Fixes y Mejoras

### 12.1 Fixes Críticos
1. **Rutas duplicadas FastAPI**: Eliminadas rutas duplicadas en `main.py`
2. **Puerto hardcodeado**: Ahora usa puerto dinámico del sidecar
3. **Ruta DB hardcodeada**: Ahora usa `DbPath` managed state

### 12.2 Fixes de Parsing
1. **`$` split**: `_preprocesar_linea()` une `$` con número: `$ 18` → `$18`
2. **Espacios vs guiones**: Patrones de espacio primero, guiones después
3. **Case-insensitive**: Todas las comparaciones usan `LOWER()`
4. **Líneas no útiles**: Filtro mejorado para miscelánea, teléfonos, etc.

### 12.3 Fixes de UI
1. **Indicador corregido**: Muestra "X tickets · Y productos" en vez de solo tickets
2. **Preview mejorado**: Muestra stock en vez de categoría en modo catálogo
3. **ColumnMapper unificado**: Un solo botón "Guardar Ticket" en vez de dos

### 12.4 Fixes de Backend
1. **VRAM management**: Descarga automática de modelos
2. **Check process alive**: Detecta procesos Python muertos
3. **LRU cache**: Previene re-análisis de tickets

---

## 13. Estado Actual

### ✅ Completado
- Parser de catálogos (TXT/CSV/Excel) con 423 productos reales
- Parser de tickets con LLM (Qwen 0.5B → 0.8B → 1.7B)
- Batch processor con SSE para 12000+ tickets
- Inventario con campo `vendido` y auto-descuento
- Deduplicación de catálogos (SHA256) y productos (max 2)
- ParserContext + localStorage para persistencia
- Selección de carpeta en batch processor
- 7 tickets / 31 productos parseados (según screenshot)

### 🔧 Pendiente
- Conexión del servidor sidecar Python (error "sin servidor")
- Gráficas/charts para predicciones Prophet
- Filtrar 23 items de sub-categoría en catálogo

### 📊 Métricas
- **Productos en catálogo**: 423 (400 con datos completos)
- **Tickets disponibles**: 12000+
- **Tickets procesados**: 7
- **Productos parseados**: 31

---

## 14. Comandos Útiles

```bash
# Limpiar cache
rm -rf yarvis-app/src-tauri/target
rm -rf yarvis-app/node_modules/.vite
find . -name "*.pyc" -delete
find . -name "__pycache__" -type d -exec rm -rf {} +

# Compilar Rust
cd yarvis-app/src-tauri && cargo check

# Verificar TypeScript
cd yarvis-app && npx tsc --noEmit

# Verificar Python
cd yarvis-IA && python -m py_compile main.py

# Ejecutar app
./run.sh
```

---

## 15. Dependencias Principales

### Rust (Cargo.toml)
- `tauri` 2.x
- `sqlx` 0.8 (SQLite)
- `reqwest` 0.12 (HTTP client)
- `sha2` 0.10 (SHA256)
- `calamine` (Excel)

### Python (requirements.txt)
- `fastapi`
- `uvicorn`
- `llama-cpp-python` (con CUDA)
- `pandas`
- `openpyxl` (Excel)

### Frontend (package.json)
- `@tauri-apps/api`
- `@tauri-apps/plugin-dialog`
- React
- TypeScript
- Tailwind CSS

---

*Documento generado automáticamente - Y.A.R.V.I.S. POS*
