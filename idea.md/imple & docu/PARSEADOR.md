# Documentacion de Parseo - Y.A.R.V.I.S. POS

> Actualizado 2026-09-03: **EL PARSEO DE TICKETS YA NO USA LLM**. La estructura
> de columnas la detecta el detector estadistico (`cerebro/analizador_tickets/detector.rs`)
> verificando la ecuacion `cantidad x precio - descuento ≈ total` contra cientos de
> lineas reales del lote; el mapeo ganador esta matematicamente demostrado, no
> "adivinado" por un modelo. El Qwen 1.7B queda SOLO para el chat local.
>
> Ademas:
> - **Idempotencia por folio** (2026-09): re-importar la misma carpeta no duplica
>   ventas ni descuenta stock dos veces.
> - **Fechas validadas** (2026-09): fechas imposibles ("99/99/9999") nunca entran a
>   `ventas.fecha`; años de 2 digitos con pivote (98 → 1998, no 2098); horas validadas.
> - **Mapeo por archivo**: si el mapeo general no reconoce un archivo (otra impresora
>   / formato), se le detecta uno propio en caliente (carpetas de formatos mezclados).

## Resumen General

Sistema de parseo de tickets y catalogos para el Modulo de Importacion Inteligente. Soporta TXT, CSV y Excel. La importacion masiva de tickets detecta el formato por VERIFICACION MATEMATICA (sin IA), con procesamiento por lotes en streaming y transaccion por archivo.

---

## 1. Estructura Modular (Rust — src-ia/parseador_de_tickets/)

```
parseador_de_tickets/
├── lib.rs                          # Entry point: declara cerebro, formatos, rutas, motor_chat, predicciones.
├── cerebro/                        # Logica de negocio y procesamiento masivo (sin modelos).
│   ├── analizador_tickets/         #   parser.rs, encabezado.rs, fechas.rs, pagos.rs,
│   │                               #   segmentador.rs, totales.rs, esquema.rs,
│   │                               #   detector.rs (mapeo estadistico, SIN LLM)
│   ├── filtrador/                  #   Filtro de lineas utiles (niveles 1/2/3).
│   ├── parseador_masivo/           #   archivos.rs, procesador.rs, items.rs, resumen.rs, almacen.rs
│   └── vinculador_inventario/      #   inventario.rs, similitud.rs (TF-IDF+fuzzy), vinculo.rs, persistencia.rs
├── formatos/                       # Lectores mecanicos por formato.
│   ├── lector_csv.rs               #   CSV (auto-detect separador/header).
│   ├── lector_excel.rs             #   Excel (calamine).
│   └── lector_txt.rs               #   Tickets .txt y catalogo visual.
└── rutas/                          # Resolucion de modelos + generacion local. SOLO CHAT.
    ├── analizador_json.rs          #   extraer_json (generico, sin uso en parseo).
    ├── analizador_modelos.rs       #   descargar/cargar/verificar el GGUF del chat.
    ├── analizador_inferencia.rs    #   generar_bajo_lock (llama.cpp) — chat.
    └── rutas_modelos_api|config|detect.rs   # API + config + deteccion (LM Studio en ~/.lmstudio/models).
```

### Backend Tauri (exposicion de comandos — yarvis-app/src-tauri/src/backventanas/backadmin/adminparser/)

| Archivo | Comandos que expone |
|---|---|
| parser_txt.rs | listar_archivos_carpeta, leer_archivo_raw, leer_archivo_bytes, parsear_catalogo_visual, **detectar_mapeo_estadistico (SIN IA)**, parsear_con_mapeo, parsear_carpeta, parsear_carpeta_stream |
| parser_csv.rs | parsear_catalogo_csv (auto-detect separador/header/columnas numericas) |
| parser_excel.rs | parsear_excel |
| parser_commands.rs | get_db_path, vincular_inventario, guardar_vinculacion, descargar_modelos |
| utils.rs | Utilidades compartidas (rutas, precio limpio) |

---

## 2. Funciones Eliminadas de ColumnMapper.tsx (ya resueltas)

Estas funciones se eliminaron porque el preview ahora lo da el LLM o el parser de reglas:

- parsearLinea(linea) — re-parseaba lineas. Eliminada: el preview viene de analysis.ejemplo_parseado.
- esLineaUtil(linea) — filtraba metadata. Eliminada: no se necesita para previsualizar.
- resolverIndice(columnas, nombre) — mapeo manual. Eliminada: el mapeo lo sugiere la IA.
- limpiarPrecio(precioStr) / limpiarProducto(productoStr) — Eliminadas: viene limpio de Rust (utils.rs / filtrador).

---

## 3. Fix de producto tipo Array

La IA o el parser pueden retornar producto como numero (2) en lugar de array ([2]). Se normaliza siempre:

```typescript
producto: Array.isArray(item.producto) ? item.producto : [item.producto]
```

En el flow actual se hace en el hook useParserActions.ts (config -> import) antes de guardar.

---

## 4. Preview con ejemplo_parseado del LLM (o del parseador de reglas)

- Antes: se re-parseaban las primeras 10 lineas (fallaba por metadata: fecha, cajero, subtotal).
- Ahora: el preview usa analysis.ejemplo_parseado || [] — lo que ya parseo el analisis LLM, o el resultado del parseo de reglas con mapeo confirmado.

---

## 5. Persistencia de Catalogo

Estados en el frontend (ImportModule / useParserActions):

```typescript
const [lastCatalogPath, setLastCatalogPath] = useState<string>('');
const [lastCatalogItems, setLastCatalogItems] = useState<Producto[]>([]);
```

Flujo: parsea catalogo -> se guarda en memoria -> cambia de modo -> vuelve -> se restaura la seleccion automaticamente. En backend, importar_catalogo usa hash SHA256 para evitar duplicados y transaccion todo-o-nada.

---

## 6. Botones Unificados

Antes existian "Aceptar Mapeo" + "Guardar Ticket Analizado" por separado; ahora el guardado une mapeo + persistencia en DB en un solo comando/accion (guardar_ticket_parseado).

---

## 7. VRAM / descarga de modelos

La gestion descargar_modelos() de Python (auto-unload en finally, endpoints /unload_llm) ya no existe como HTTP. Hoy:

- El comando Tauri descargar_modelos existe por compatibilidad (adminparser/parser_commands.rs) y libera el modelo compartido si es necesario.
- El parseo de tickets ya NO toca el modelo: puro regex + verificacion matematica, instantaneo incluso en laptops viejas. El Qwen 1.7B solo lo carga el chat local, controlado via load_chat_model / unload_chat_model con verificacion de RAM.

---

## 8. Errores Comunes y Soluciones (vigentes)

| Error | Causa | Solucion |
|---|---|---|
| producto no es array | Parser retorna entero | Normalizar con Array.isArray() |
| Preview no muestra productos | Metadata filtraba todo | Usar ejemplo_parseado del LLM |
| Catalogo pierde datos al cambiar modo | Estado no persistia | lastCatalogPath + lastCatalogItems |
| Nombre con " -- " (separador del catalogo) | Patron SIN_SEP se comia el separador | Bug 8 resuelto en Rust: reorden de patrones; los lectores no arrastran el separador |
| Producto legitimo descartado ("GATORADE TOTAL") | substring if x in linea_lower | _es_linea_util con 3 niveles + word-boundary (portado a cerebro/filtrador) |

> Los bugfix A1 (transaccion por archivo con rollback), A3 (filtro 3 niveles), A4 (volumenes) y Bug 8 (separador robado) fueron verificados en Python y conservados en el port a Rust. Ver bugs-resueltos.md.

---

## 9. Flujos de Datos (estado actual)

### Flujo de Parseo de Tickets (importacion masiva, SIN IA)

```
1. Usuario elige la CARPETA de tickets .txt.
2. detectar_mapeo_estadistico toma una muestra determinista y espaciada
   (hasta 15 archivos) y elige el mapeo que MAS lineas cuadra la ecuacion
   cantidad x precio - descuento ≈ total. Devuelve confianza y conteos.
   - Si la confianza es baja o ningun formato cuadra: error claro al usuario
     ("agrupa tickets de la misma impresora"). Nada se escribe en la DB.
3. parsear_carpeta_stream procesa cada archivo con mapeo + fallback por
   archivo (el archivo cuyo formato no cuadra recibe deteccion propia).
   - Idempotencia: tickets con folio ya importado se omiten enteros.
4. El frontend muestra progreso en vivo y un resumen (ventas creadas,
   omitidas por folio, archivos con formato distinto rescatados).
```

### Flujo de Parseo de Catalogo

```
1. Usuario sube archivo (TXT/CSV/Excel).
2. Comando parsear_catalogo_* -> Rust parsea segun formato.
3. Retorna productos parseados -> TablaPreview -> mapeo -> "Guardar" persiste en DB (importar_catalogo con deduplicacion max 2 por nombre).
```

### Flujo de Batch Processing

```
1. Usuario selecciona carpeta.
2. parsear_carpeta_stream procesa con eventos SSE.
3. Rust procesa cada archivo con su propia transaccion (rollback ante fallo).
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

- Sin precios = productos con $0: todos los parsers retornan productos aunque falte la columna de precio.
- ColumnMapper inline: aparece dentro del Modulo de Importacion Inteligente (reutilizado en adminconfig/components/importmodule/).
- Rust como escritor unico: el parseo lee archivos, pero la escritura en DB siempre pasa por comandos Tauri.
- Idioma: espanol para Mexico (pesos mexicanos).
- El mapeo de columnas es estadistico (ver seccion 9): el unico LLM del sistema es el CHAT (Qwen 3 1.7B local + cloud fallback), que nunca entra al pipeline de parseo.
