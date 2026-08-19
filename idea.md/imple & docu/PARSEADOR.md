# Documentación de Parseo - Y.A.R.V.I.S. POS

> ⚠️ **ACTUALIZADO (2026-Ago):** el parseador ya no vive en Python (`yarvis-IA/...`) ni se habla por HTTP con un servidor FastAPI. Todo el motor quedó portado a **Rust** en el crate `src-ia/parseador_de_tickets`, expuesto al frontend vía **comandos Tauri** (`adminparser/parser_*.rs`). Esta doc describe el estado real.

## Resumen General

Sistema de parseo de tickets y catálogos para el Módulo de Importación Inteligente. Soporta TXT, CSV y Excel con análisis LLM local para interpretación automática de columnas, más procesamiento por lotes con streaming.

---

## 1. Estructura Modular (Rust — `src-ia/parseador_de_tickets/`)

```
parseador_de_tickets/
├── lib.rs                          # Entry point: declara cerebro, formatos, rutas, motor_chat.
├── cerebro/                        # Lógica de negocio y procesamiento masivo (sin modelos).
│   ├── analizador_tickets/         #   parser.rs, encabezado.rs, fechas.rs, pagos.rs,
│   │                               #   segmentador.rs, totales.rs, esquema.rs
│   ├── filtrador/                  #   Filtro de líneas útiles (niveles 1/2/3).
│   ├── parseador_masivo/           #   archivos.rs, procesador.rs, items.rs, resumen.rs, almacen.rs
│   └── vinculador_inventario/      #   inventario.rs, similitud.rs, vinculo.rs, persistencia.rs
├── formatos/                       # Lectores mecánicos por formato.
│   ├── lector_csv.rs               #   CSV (auto-detect separador/header).
│   ├── lector_excel.rs             #   Excel (calamine).
│   └── lector_txt.rs               #   Tickets .txt y catálogo visual.
└── rutas/                          # Resolución de modelos + análisis LLM.
    ├── analizador_ticket.rs        #   analizar_ticket (LLM local 1.7B).
    ├── analizador_prompt.rs        #   SISTEMA_PROMPT.
    ├── analizador_json.rs          #   extraer_json.
    ├── analizador_modelos.rs       #   descargar/cargar/verificar modelos GGUF.
    ├── analizador_inferencia.rs    #   generar_bajo_lock (llama.cpp).
    └── rutas_modelos_api.rs|config.rs|detect.rs   # API + config + detección (LM Studio).
```

### Backend Tauri (exposición de comandos — `yarvis-app/src-tauri/src/backventanas/backadmin/adminparser/`)

| Archivo | Comandos que expone |
|---|---|
| `parser_txt.rs` | `listar_archivos_carpeta`, `leer_archivo_raw`, `leer_archivo_bytes`, `parsear_catalogo_visual`, `analizar_ticket_llm`, `analizar_ticket_con_ia`, `parsear_con_mapeo`, `parsear_carpeta`, `parsear_carpeta_stream` |
| `parser_csv.rs` | `parsear_catalogo_csv` (auto-detect separador/header/columnas numéricas) |
| `parser_excel.rs` | `parsear_excel` |
| `parser_commands.rs` | `get_db_path`, `vincular_inventario`, `guardar_vinculacion`, `descargar_modelos` |
| `utils.rs` | Utilidades compartidas (rutas, precio limpio) |

Última referencia a Python en el backend: comentarios tipo `// port de lector_csv.py` (migración).

---

## 2. Funciones Eliminadas de ColumnMapper.tsx (ya resueltas)

Estas funciones se eliminaron porque el preview ahora lo da el LLM (o el parser de reglas):

- `parsearLinea(linea)` — re-parseaba líneas. **Eliminada:** el preview viene de `analysis.ejemplo_parseado`.
- `esLineaUtil(linea)` — filtraba metadata. **Eliminada:** no se necesita para previsualizar.
- `resolverIndice(columnas, nombre)` — mapeo manual. **Eliminada:** el mapeo lo sugiere la IA.
- `limpiarPrecio(precioStr)` / `limpiarProducto(productoStr)` — **Eliminadas:** viene limpio de Rust (`utils.rs` / `filtrador`).

---

## 3. Fix de `producto` tipo Array

La IA o el parser pueden retornar `producto` como número (`2`) en lugar de array (`[2]`). Se normaliza siempre:

```typescript
producto: Array.isArray(item.producto) ? item.producto : [item.producto]
```

En el flow actual se hace en el hook `useParserActions.ts` (config → import) antes de guardar.

---

## 4. Preview con `ejemplo_parseado` del LLM (o del parseador de reglas)

- **Antes:** se re-parseaban las primeras 10 líneas (fallaba por metadata: fecha, cajero, subtotal).
- **Ahora:** el preview usa `analysis.ejemplo_parseado || []` — lo que ya parseó el análisis LLM, o el resultado del parseo de reglas con mapeo confirmado.

---

## 5. Persistencia de Catálogo

Estados en el frontend (`ImportModule` / `useParserActions`):

```typescript
const [lastCatalogPath, setLastCatalogPath] = useState<string>('');
const [lastCatalogItems, setLastCatalogItems] = useState<Producto[]>([]);
```

Flujo: parsea catálogo → se guarda en memoria → cambia de modo → vuelve → se restaura la selección automáticamente.

---

## 6. Botones Unificados

Antes existían "Aceptar Mapeo" + "Guardar Ticket Analizado" por separado; ahora el guardado une **mapeo + persistencia en DB** en un solo comando/acción (`guardar_ticket_parseado`).

---

## 7. VRAM / descarga de modelos — (HISTÓRICO, era Python)

La gestión `descargar_modelos()` de Python (auto-unload en `finally`, endpoints `/unload_llm`) **ya no existe como HTTP**. Hoy:

- El comando Tauri `descargar_modelos` existe por compatibilidad (`adminparser/parser_commands.rs`).
- El análisis de tickets usa el LLM local **bajo demanda** (`rutas/analizador_inferencia.rs` con lock) y **no deja cargado** el modelo global a menos que el chat lo necesite.

---

## 8. Errores Comunes y Soluciones (vigentes)

| Error | Causa | Solución |
|---|---|---|
| `producto` no es array | Parser retorna entero | Normalizar con `Array.isArray()` |
| Preview no muestra productos | Metadata filtraba todo | Usar `ejemplo_parseado` del LLM |
| Catálogo pierde datos al cambiar modo | Estado no persistía | `lastCatalogPath` + `lastCatalogItems` |
| Nombre con " -- " (separador del catálogo) | Patrón SIN_SEP se comía el separador | Bug 8 resuelto en Rust: reorden de patrones; los lectores no arrastran el separador (ver `Bugs resueltos uwu.md`) |
| Produto legítimo descartado ("GATORADE TOTAL", "CAJA DE MADERA") | substring `if x in linea_lower` | `_es_linea_util` con 3 niveles + word-boundary (portado a `cerebro/filtrador`) |

> Los bugfix A1 (transacción por archivo con rollback), A3 (filtro 3 niveles), A4 (volúmenes) y Bug 8 (separador robado) fueron **verificados en Python y conservados en el port a Rust**.

---

## 9. Flujos de Datos (estado actual)

### Flujo de Parseo de Ticket (entrenar IA)
```
1. Usuario sube TXT/CSV/Excel en el Módulo de Importación Inteligente.
2. Frontend llama al comando correspondiente (analizar_ticket_con_ia / parsear_*).
3. Rust (src-ia) lee el archivo con el lector de formatos correspondiente.
4. El LLM (o reglas) deduce columnas; retorna LLMAnalysis con ejemplo_parseado JSON.
5. Se muestra preview en ColumnMapper; el usuario ajusta el mapeo.
6. "Guardar Ticket" → guarda mapeo + persiste en DB (Rust escribe SQLite).
7. El modelo no se queda cargado en RAM.
```

### Flujo de Parseo de Catálogo
```
1. Usuario sube archivo (TXT/CSV/Excel).
2. Comando parsear_catalogo_* → Rust parsea según formato.
3. Retorna productos parseados → TablaPreview → mapeo → "Guardar" persiste en DB.
```

### Flujo de Batch Processing
```
1. Usuario selecciona carpeta.
2. parsear_carpeta_stream procesa con eventos SSE.
3. Rust procesa cada archivo con SU PROPIA transacción (rollback ante fallo).
4. Frontend muestra progreso en tiempo real (BatchProcessor).
5. Al terminar: vincular con inventario existente (vincular_inventario / guardar_vinculacion).
```

---

## 10. Tipos TypeScript (vigentes)

```typescript
interface ColumnMapping {
  [key: string]: {
    columna_origen: string;
    tipo_dato: 'producto' | 'precio' | 'cantidad' | 'categoria';
    indice: number;
  };
}

interface LLMAnalysis {
  columnas_detectadas: string[];
  mapeo_sugerido: { [key: string]: string };
  ejemplo_parseado: Producto[];
  confianza: number;
}

interface Producto {
  producto: number[];
  nombre: string;
  precio: number;
  cantidad: number;
  categoria?: string;
}
```

---

## 11. Notas Importantes

- **Sin precios = productos con $0:** todos los parsers retornan productos aunque falte la columna de precio.
- **ColumnMapper inline**: aparece dentro del "MÓDULO DE IMPORTACIÓN INTELIGENTE" (reutilizado en `adminconfig/components/importmodule/`).
- **Rust como escritor único**: el parseo lee archivos, pero la escritura en DB siempre pasa por comandos Tauri.
- **Idioma**: español para México (pesos mexicanos).