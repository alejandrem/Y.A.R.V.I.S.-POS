# Documentación de Parseo - Y.A.R.V.I.S. POS

## Resumen General

Sistema de parseo de productos y catálogos para el módulo de importación inteligente. Soporta múltiples formatos (TXT, CSV, Excel) con IA para interpretación automática de columnas.

---

## 1. Estructura Modular de Parsers

### **Python (`parser_py/`)**

```
parser_py/
├── __init__.py          # Endpoints FastAPI
├── parser_txt.py        # Parseo de tickets TXT
├── parser_csv.py        # Parseo de catálogos CSV
├── parser_excel.py      # Parseo de catálogos Excel
└── utils.py             # Funciones compartidas
```

#### **`__init__.py`**
- Registro de endpoints `/parsear_excel` y `/parsear_catalogo_visual`
- **Cambio:** Endpoints duplicados eliminados (antes existían también en `parser_txt.py`)

#### **`parser_txt.py`**
- `parsear_archivo(ruta)`: Parsea archivos TXT línea por línea
- `parsear_linea(linea)`: Extrae producto y precio de una línea

#### **`parser_csv.py`**
- `parsear_csv(ruta)`: Parsea archivos CSV con detección automática de delimitador

#### **`parser_excel.py`**
- `parsear_excel(ruta)`: Parsea archivos Excel (.xlsx, .xls)

#### **`utils.py`**
- `limpiar_producto(texto)`: Limpia caracteres especiales de nombres de producto
- `es_categoria(linea)`: Determina si una línea es categoría (no producto)

---

### **Rust (`parser_rs/`)**

```
parser_rs/
├── lib.rs               # Módulos públicos
├── parser_txt.rs        # Parseo de tickets TXT
├── parser_csv.rs        # Parseo de catálogos CSV
└── utils.rs             # Funciones compartidas
```

#### **`lib.rs`**
- Exporta `parsear_txt` y `parsear_csv`

#### **`parser_txt.rs`**
- `parsear_txt(ruta)`: Parsea archivos TXT

#### **`parser_csv.rs`**
- `parsear_csv(ruta)`: Parsea archivos CSV

#### **`utils.rs`**
- `sanitize_path(path)`: Normaliza rutas de archivos
- `limpiar_precio(precio_str)`: Limpia formato de precios

---

## 2. Funciones Eliminadas de ColumnMapper.tsx

Estas funciones se eliminaron porque el preview ahora viene del LLM:

### **`parsearLinea(linea)`**
- Parseaba una línea del catálogo
- Extraía producto y precio
- **Eliminada:** Preview viene de `analysis.ejemplo_parseado`

### **`esLineaUtil(linea)`**
- Filtraba metadata de tickets (fecha, cajero, subtotal, etc.)
- **Eliminada:** No se necesita para previsualizar

### **`resolverIndice(columnas, nombre)`**
- Resolvía índice de columna por nombre
- **Eliminada:** Mapeo ahora se hace con AI

### **`limpiarPrecio(precioStr)`**
- Limpiaba caracteres especiales de precios
- **Eliminada:** Precio viene limpio del LLM

### **`limpiarProducto(productoStr)`**
- Limpiaba caracteres especiales de productos
- **Eliminada:** Producto viene limpio del LLM

---

## 3. Fix de `producto` tipo Array

### **Problema**
La IA retornaba:
```json
{
  "producto": 2,
  "nombre": "Producto Ejemplo",
  "precio": 100
}
```

El frontend esperaba:
```json
{
  "producto": [2],
  "nombre": "Producto Ejemplo",
  "precio": 100
}
```

### **Solución**
En `handleAnalizar` (Configuracion.tsx):
```typescript
const normalizedItems = analysis.ejemplo_parseado.map(item => ({
  ...item,
  producto: Array.isArray(item.producto) ? item.producto : [item.producto]
}));
```

---

## 4. Preview con `ejemplo_parseado` del LLM

### **Antes**
```typescript
// Re-parseaba primeras 10 líneas
const lines = fileContent.split('\n').filter(line => line.trim());
const previewItems = lines.slice(0, 10).map(line => parsearLinea(line));
```

### **Ahora**
```typescript
// Usa el ejemplo parseado por la IA
const previewItems = analysis.ejemplo_parseado || [];
```

### **Por qué**
- `esLineaUtil` filtraba metadata de tickets (fecha, cajero, subtotal)
- Las primeras 10 líneas no siempre son productos
- El LLM ya parseó correctamente el contenido

---

## 5. Persistencia de Catálogo

### **Estados Nuevos en Configuracion.tsx**
```typescript
const [lastCatalogPath, setLastCatalogPath] = useState<string>('');
const [lastCatalogItems, setLastCatalogItems] = useState<Producto[]>([]);
```

### **Flujo**
1. **Parsea catálogo** → se guarda en `lastCatalogPath` + `lastCatalogItems`
2. **Cambia a modo "entrenar IA"** → catálogo persiste en memoria
3. **Vuelve a modo "catálogo"** → catálogo restaurado automáticamente

### **Código de restauración**
```typescript
useEffect(() => {
  if (parserMode === 'catalogo' && lastCatalogPath && lastCatalogItems.length > 0) {
    setSelectedPath(lastCatalogPath);
    setParsedItems(lastCatalogItems);
  }
}, [parserMode]);
```

---

## 6. Botones Unificados

### **Antes**
1. **ColumnMapper.tsx:** "Aceptar Mapeo" (guardaba mapeo)
2. **Configuracion.tsx:** "Guardar Ticket Analizado" (persistía en DB)

### **Ahora**
1. **ColumnMapper.tsx:** "Guardar Ticket" (guarda mapeo + persiste en DB)

### **Código de unified save**
```typescript
// ColumnMapper.tsx
const handleGuardarTicket = async () => {
  // 1. Guarda mapeo
  onGuardarTicket(columnMapping);
  
  // 2. Persiste en DB
  await invoke('guardar_ticket', {
    ticketData: {
      ruta_archivo: selectedPath,
      contenido: fileContent,
      mapeo_columnas: columnMapping,
      productos_parseados: parsedItems
    }
  });
  
  // 3. Limpia estado
  setShowColumnMapper(false);
};
```

---

## 7. VRAM Management

### **Función `descargar_modelos()` en parser_llm.py**
```python
def descargar_modelos():
    global modelo_txt, modelo_excel
    
    # Elimina modelos globales
    if modelo_txt is not None:
        del modelo_txt
        modelo_txt = None
    
    if modelo_excel is not None:
        del modelo_excel
        modelo_excel = None
    
    # Limpia memoria
    gc.collect()
    print("Modelos descargados de VRAM")
```

### **Auto-unload en 2 puntos**

#### **1. Antes de batch processing (`carpeta.py`)**
```python
@app.post("/parsear_carpeta_stream")
async def parsear_carpeta_stream(carpeta_data: CarpetaData):
    # Descarga modelos antes de empezar
    descargar_modelos()
    
    # Procesa carpeta...
```

#### **2. Después de ticket analysis (`parser.py`)**
```python
@app.post("/analizar_ticket")
async def analizar_ticket(ticket: TicketRequest):
    try:
        # Analiza ticket...
        return resultado
    finally:
        # Descarga modelos después de analizar
        descargar_modelos()
```

### **Endpoint `/unload_llm` en chat.py**
```python
@app.post("/unload_llm")
async def unload_llm():
    descargar_modelos()
    return {"status": "ok", "message": "Modelos descargados de VRAM"}
```

### **Comando Tauri `descargar_modelos` en commands/parser.rs**
```rust
#[tauri::command]
pub async fn descargar_modelos(sidecar: tauri::State<'_, Arc<AiSidecar>>) -> Result<(), String> {
    sidecar.get("unload_llm").await.map_err(|e| e.to_string())?;
    Ok(())
}
```

---

## 8. Fix de `check_process_alive`

### **Problema**
```rust
// No compilaba
*guard.take()
```

### **Solución**
```rust
// Ahora compila
*guard = None
```

---

## 9. Errores de Embedding Reducidos

### **Problema**
- `/generar_embedding` fallaba 4 veces por batch (ruido en consola)

### **Solución**
```rust
// AtomicBool para asegurar error print solo 1 vez
static EMBEDDING_ERROR_PRINTED: AtomicBool = AtomicBool::new(false);

// En generar_embedding
if EMBEDDING_ERROR_PRINTED.compare_and_swap(false, true, Ordering::SeqCst) {
    eprintln!("Error generando embedding: {}", error);
}
```

---

## 10. Funciones Compartidas

### **Python `utils.py`**
```python
def limpiar_producto(texto: str) -> str:
    # Elimina caracteres especiales
    texto = re.sub(r'[^\w\sáéíóúñ]', '', texto)
    return texto.strip()

def es_categoria(linea: str) -> bool:
    # Detecta líneas que son categorías
    categorias = ['CATEGORIA', 'SECCION', 'DEPARTAMENTO']
    return any(c in linea.upper() for c in categorias)
```

### **Rust `utils.rs`**
```rust
pub fn sanitize_path(path: &str) -> String {
    // Normaliza rutas
    path.replace('\\', "/")
}

pub fn limpiar_precio(precio_str: &str) -> f64 {
    // Limpia formato de precios
    let cleaned = precio_str.replace(['$', ',', ' '], "");
    cleaned.parse().unwrap_or(0.0)
}
```

---

## 11. Configuración de IA

### **Parámetros de parseo**
- `temperature`: 0.1 (baja para respuestas consistentes)
- `max_tokens`: 1000
- `top_p`: 0.9

### **Modelos utilizados**
- **Qwen 0.5B:** Para parseo inicial de tickets
- **Qwen 1.7B:** Para parseo de catálogos complejos

### **Fallback automático**
- Si Qwen 0.5B tiene confianza < 0.8 → retry con Qwen 1.7B

---

## 12. Endpoints Relacionados

| Endpoint | Método | Descripción |
|----------|--------|-------------|
| `/analizar_ticket` | POST | Analiza ticket individual |
| `/parsear_carpeta_stream` | POST | Procesa carpeta con streaming |
| `/parsear_excel` | POST | Parsea archivos Excel |
| `/parsear_catalogo_visual` | POST | Parsea catálogos visuales |
| `/generar_embedding` | POST | Genera embedding de producto |
| `/buscar_producto_similar` | POST | Busca producto similar |
| `/unload_llm` | POST | Descarga modelos de VRAM |

---

## 13. Tipos TypeScript

### **ColumnMapping**
```typescript
interface ColumnMapping {
  [key: string]: {
    columna_origen: string;
    tipo_dato: 'producto' | 'precio' | 'cantidad' | 'categoria';
    indice: number;
  };
}
```

### **LLMAnalysis**
```typescript
interface LLMAnalysis {
  columnas_detectadas: string[];
  mapeo_sugerido: { [key: string]: string };
  ejemplo_parseado: Producto[];
  confianza: number;
}
```

### **Producto**
```typescript
interface Producto {
  producto: number[];
  nombre: string;
  precio: number;
  cantidad: number;
  categoria?: string;
}
```

---

## 14. Flujos de Datos

### **Flujo de Parseo de Ticket**
```
1. Usuario selecciona archivo TXT
2. Frontend envía a `/analizar_ticket`
3. Python parsea archivo con Qwen 0.5B
4. Si confianza < 0.8 → retry con Qwen 1.7B
5. Retorna LLMAnalysis con ejemplo_parseado
6. Frontend muestra preview en ColumnMapper
7. Usuario ajusta mapeo de columnas
8. Click "Guardar Ticket" → persiste en DB
9. Modelos se descargan de VRAM
```

### **Flujo de Parseo de Catálogo**
```
1. Usuario selecciona archivo (TXT/CSV/Excel)
2. Frontend envía a `/parsear_catalogo_visual`
3. Python parsea archivo según formato
4. Retorna productos parseados
5. Frontend muestra en TablaPreview
6. Usuario ajusta mapeo si es necesario
7. Click "Guardar" → persiste en DB
```

### **Flujo de Batch Processing**
```
1. Usuario selecciona carpeta
2. Frontend envía a `/parsear_carpeta_stream`
3. Python descarga modelos de VRAM
4. Python procesa cada archivo en carpeta
5. Retorna resultados por SSE
6. Frontend muestra progreso en tiempo real
7. Al finalizar, modelos se descargan
```

---

## 15. Errores Comunes y Soluciones

### **Error: `producto` no es array**
- **Causa:** IA retorna entero en lugar de array
- **Solución:** Normalizar con `Array.isArray()`

### **Error: Preview no muestra productos**
- **Causa:** `esLineaUtil` filtraba metadata
- **Solución:** Usar `ejemplo_parseado` del LLM

### **Error: VRAM llena después de análisis**
- **Causa:** Modelos no se descargaban
- **Solución:** Auto-unload en `finally` block

### **Error: Catálogo pierde datos al cambiar modo**
- **Causa:** Estado no persistía
- **Solución:** `lastCatalogPath` + `lastCatalogItems`

### **Error: Embedding falla 4 veces por batch**
- **Causa:** Error se imprimía por cada archivo
- **Solución:** `AtomicBool` para imprimir solo 1 vez

---

## 16. Próximos Pasos

1. **Verificar app completa end-to-end** con todos los cambios
2. **Implementar gráficas** para predicciones Prophet
3. **Test de catálogo Excel** con flujo completo Tauri → Python → Frontend

---

## 17. Notas Importantes

- **Sin precios = productos con $0:** Todos los parsers retornan productos incluso sin columnas de precio
- **ColumnMapper inline:** Aparece dentro de "MÓDULO DE IMPORTACIÓN INTELIGENTE"
- **GPU auto-detect:** `LD_LIBRARY_PATH` se establece por el sidecar para CUDA
- **Lazy loading:** Modelos se cargan bajo demanda y se descargan después de usar
- **Idioma:** Sistema diseñado en español para México (pesos mexicanos)
