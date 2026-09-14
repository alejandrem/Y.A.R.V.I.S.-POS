# Documentacion de Parseo - Y.A.R.V.I.S. POS

> Actualizado 2026-09-06: **EL PARSEO DE TICKETS YA NO USA LLM**. La estructura
> de columnas la detecta el detector estadistico (`cerebro/analizador_tickets/detector/`)
> verificando la ecuacion `cantidad x precio - descuento ≈ total` contra cientos de
> lineas reales del lote; el mapeo ganador esta matematicamente demostrado, no
> "adivinado" por un modelo. El Qwen 1.7B queda SOLO para el chat local.
>
> Novedades de septiembre (verificadas con 1000 tickets reales de 6 meses):
> - **Folio 10/10 formatos**: etiquetas FOLIO/FOL/TICKET/SERIE/NOTA/RECIBO,
>   separadores `: . # | = -`, muletillas (`TICKET NO. 1927` → `1927`) y valor
>   con dígito obligatorio (cero colisiones).
> - **Clave de idempotencia en 3 niveles** (`TicketSegmento::clave`): folio
>   impreso → `AUTO-YYYYMMDD-HHMM-hash` (fecha+hora) → `SIN-FOLIO-hash`.
>   Re-importar ya no duplica, con o sin folio.
> - **Inserción cronológica**: archivos y segmentos se ordenan por fecha/hora;
>   los IDs de venta crecen en el orden en que se generaron.
> - **Familia C con denominador correcto**: la confianza es
>   consistentes/repetidas (lo único observable), no consistentes/muestra.
> - **Errores en lenguaje normal** (`diagnosticar_muestra`): con números y
>   siguiente paso, no "agrupa tus tickets" a secas.
> - **Idempotencia por folio** (2026-09): re-importar la misma carpeta no duplica
>   ventas ni descuenta stock dos veces.
> - **Fechas validadas** (2026-09): fechas imposibles ("99/99/9999") nunca entran a
>   `ventas.fecha`; años de 2 digitos con pivote (98 → 1998, no 2098); horas validadas;
>   mes abreviado día-primero ("05-mar-2026") e ISO con hora pegada ("2026-03-04T20:11:03").
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
│   │                               #   esquema.rs, totales.rs (+ detector/ y segmentador/)
│   │   ├── detector/               #     Detección estadística (SIN LLM), un tema por archivo:
│   │   │                         #     mod.rs (orquesta detectar_mapeo), hipotesis.rs
│   │   │                         #     (linea_cuadra + familias A/B), familia_c.rs
│   │   │                         #     (consistencia de precios), muestra.rs,
│   │   │                         #     diagnostico.rs (por qué falló, para la UI)
│   │   ├── segmentador/            #     Un archivo → N tickets, un tema por archivo:
│   │                               #     mod.rs (TicketSegmento + segmentar),
│   │                               #     marcadores.rs (aperturas/cierres/folio),
│   │                               #     clave.rs (idempotencia + orden cronológico)
│   ├── filtrador/                  #   Filtro de lineas utiles (niveles 1/2/3).
│   ├── parseador_masivo/           #   archivos.rs, procesador/ (stream.rs + carpeta.rs),
│   │                               #   items.rs, resumen.rs, almacen.rs, tests.rs
│   └── vinculador_inventario/      #   inventario.rs, similitud.rs (TF-IDF+fuzzy), vinculo.rs, persistencia.rs
├── formatos/                       # Lectores mecanicos por formato.
│   ├── lector_csv.rs               #   CSV (auto-detect separador/header).
│   ├── lector_excel.rs             #   Excel (calamine).
│   └── lector_txt/                 #   Tickets .txt y catalogo visual, por tema:
│                               #     patrones.rs (regex), linea.rs (7 patrones),
│                               #     visual.rs (CSV vs visual), tests.rs
└── rutas/                          # Resolucion de modelos + generacion local. SOLO CHAT.
    ├── analizador_json.rs          #   extraer_json (generico, sin uso en parseo).
    ├── analizador_modelos.rs       #   descargar/cargar/verificar el GGUF del chat.
    ├── analizador_inferencia.rs    #   generar_bajo_lock (llama.cpp) — chat.
    └── rutas_modelos_api|config|detect.rs   # API + config + deteccion (LM Studio en ~/.lmstudio/models).
```

### Backend Tauri (exposicion de comandos — yarvis-app/src-tauri/src/backventanas/backadmin/adminparser/)

| Carpeta | Comandos que expone |
|---|---|
| parser_txt/archivos.rs | listar_archivos_carpeta, leer_archivo_raw, leer_archivo_bytes |
| parser_txt/catalogo.rs | parsear_catalogo_visual |
| parser_txt/deteccion.rs | **detectar_mapeo_estadistico (SIN IA)** + mensajes en lenguaje normal |
| parser_txt/lote.rs | parsear_con_mapeo, parsear_carpeta, parsear_carpeta_stream |
| parser_csv.rs | parsear_catalogo_csv (auto-detect separador/header/columnas numericas) |
| parser_excel.rs | parsear_excel |
| empleados_auto.rs | (interno) resolver_empleados_desde_ventas al terminar cada lote |
| parser_commands.rs | get_db_path, vincular_inventario, guardar_vinculacion, descargar_modelos |
| utils.rs | Utilidades compartidas (rutas, precio limpio) |

---

## 1b. Folio, clave de idempotencia y orden cronológico (2026-09)

Verificado contra 10 formatos reales de tiendas mexicanas (folio 10/10) y
una carpeta real de 1000 tickets de 6 meses (1000/1000 ventas con total,
fecha y folio exactos al centavo).

**Detección de folio** (`segmentador/marcadores.rs::extraer_folio`):

| Formato real | Resultado |
|---|---|
| `FOLIO: 000482`, `FOLIO:2288` | `000482`, `2288` |
| `FOLIO\|55190`, `FOLIO=88231` | `55190`, `88231` (separadores `\|` y `=`) |
| `Fol 3341`, `FOL 00721` | `3341`, `00721` (abreviatura) |
| `TICKET NO. 1927` | `1927` (la muletilla `NO.` se salta; antes capturaba `"NO"`) |
| `Ticket #6650`, `TICKET: A-004471` | `6650`, `A-004471` |
| `CONSERVE SU TICKET`, `TICKET DE VENTA` | `None` (el valor exige dígito; sin dígito no es folio) |

La apertura "fuerte" (etiqueta al inicio + valor) vale aunque la línea
parezca de producto; la palabra sola jamás abre ticket fantasma.

**Clave de idempotencia** (`segmentador/clave.rs::TicketSegmento::clave`),
en 3 niveles: 1. folio impreso → 2. `AUTO-YYYYMMDD-HHMM-hash` (fecha+hora,
ancho fijo = ordenable) → 3. `SIN-FOLIO-hash` (último recurso). Se guarda
en `ventas.folio_ticket`, así la siguiente corrida encuentra el ticket con
o sin folio. Límite honesto: dos ventas distintas con mismo contenido,
misma fecha y sin folio colisionarían.

**Orden cronológico**: archivos (`ordenar_archivos_cronologicamente`, un
pre-pass barato de solo-regex) y segmentos (`comparar_cronologico`) se
ordenan por fecha/hora antes de insertar; los IDs crecen en el orden en
que se generaron. Las predicciones igual agrupan por `date(fecha)`, así
que la matemática nunca dependió del orden físico.

**Confianza de familia C**: el denominador son solo las líneas de
productos repetidos (las únicas observables), no toda la muestra. Una
carpeta real con precios perfectos daba 37% por este bug; hoy da 100%.

---

## 1c. Alta automática de empleados desde tickets (2026-09)

Como el inventario se rellena solo, la sección de empleados también: al
terminar cada importación, `adminparser/empleados_auto.rs` resuelve los
cajeros sin vincular de `ventas`.

- **Mismo nombre normalizado = misma persona** (mayúsculas, espacios
  colapsados, sin punto final): se reutiliza, nunca se duplica solo — si
  no, cada re-importe crearía usuarios sin fin. Si ≥2 usuarios comparten
  nombre, se vincula al más antiguo (lo único estable entre corridas).
- **Contraseña = nombre + "123"** (`"1234"`, `"12345"…` si choca con la de
  otra persona — obligatorio porque el login de empleado es solo-password:
  dos passwords iguales = uno nunca entra). Se guarda hasheada (Argon2);
  la plana se muestra UNA vez en el resumen final para comunicarla.
- **Se omiten**: SISTEMA, IMPORTADOR, SIN ASIGNAR y vacíos.
- **Vinculación**: pone `cajero_id` en sus tickets (nuevos e históricos
  pendientes) para que cuenten en sus estadísticas. Idempotente.
- **Lo demás es manual**: rol empleado, estado activo, salario 0; sueldo,
  horarios y días los captura el admin en Empleados.
- **Aviso cada login**: los auto-creados traen `password_defecto=1` y ven
  un banner hasta que el admin les cambie la contraseña (comando
  `aviso_password_defecto`; `editar_empleado` apaga el flag al guardar
  una nueva). Migración 0008.

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
| "37% cuadran, separa por tienda" en carpeta uniforme | confianza C dividía entre toda la muestra | denominador = solo repetidas (`detector/familia_c.rs`); hoy esa carpeta da 100% |
| Historial muestra 20 de 1000 | `.slice(0, 20)` + contador con `length` | sin recorte (backend trae 500) + `get_tickets_total` para el total real |
| Al cambiar de pestaña "no se está parseando nada" | estado + listener vivían en el componente desmontado | `BatchProgressProvider` en el dashboard (un solo listener global + bloqueo de doble importación); las pestañas se desmontan pero el provider no |

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
4. Al terminar el lote: alta automática de empleados detectados en tickets
   (ver 1c) + vinculación de sus ventas. Las credenciales se muestran UNA
   vez en el resumen final.
5. El frontend muestra progreso en vivo y un resumen (ventas creadas,
   omitidas por folio, archivos con formato distinto rescatados,
   empleados creados con sus contraseñas).
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

---

## 12. Parseo de cortes de caja X/Z

> Igual que tickets: **100% reglas en Rust, sin IA**. Cada número se verifica
> con matemática exacta en centavos. Si un corte trae otro acomodo de
> impresora, se agregan marcadores/etiquetas, nunca un modelo.

### 12.1 Qué es cada uno

- **Corte X**: informe parcial del día (mitad de turno, cambio de cajero). NO reinicia acumulados; puede haber varios al día. En el POS también servirá para consultar ventas del día en vivo (pendiente).
- **Corte Z**: cierre definitivo del día. Reinicia contadores. Es el documento contable/fiscal.
- Un corte puede traer decenas de ventas/tickets adentro; el parseador NO crea ventas en `ventas` desde cortes (evita duplicar lo que ya entró por tickets).

### 12.2 Estructura (src-ia/parseador_de_cortes/)

```
parseador_de_cortes/          # 1 archivo = 1 tarea; loners en la raíz
├── mod.rs                    # Entry point + re-exports
├── tipos.rs                  # Contrato con backend (dinero en centavos INTEGER)
├── parser.rs                 # Orquesta extracción + verificación (loner)
├── valores/                  # Limpieza de primitivas impresas
│   ├── montos.rs             #   "$2,462.80", "$.00"→0 (signo sin entero = cero)
│   └── fechas.rs             #   dd/mm/yyyy + 12h ("a. m."/"p. m.") o 24h → ISO
└── secciones/                # Una tarea por archivo
    ├── encabezado.rs         #   Título (X/Z, acepta "CORTE DE CAJA X"), folio,
    │                         #   empresa (solo zona de encabezado), estación,
    │                         #   fecha, cajero (default SISTEMA), moneda, clasificador
    ├── marcadores.rs         #   Partición por secciones (**Ingresos**, VENTAS DEL
    │                         #   CORTE, Ventas por artículo/ticket/cliente, Cobranza)
    ├── totales.rs            #   Los 13 totales, cada etiqueta SOLO en su sección
    │                         #   ("Impuesto" pelado no se confunde con "Impuesto 16%")
    └── renglones.rs          #   ARTICULO (`NOMBRE - cant - $`), TICKET (`REM - n`),
                              #   pagos (`EFE...`/`04 TARJETA...`; nada con "total"
                              #   es concepto: `Total en caja` no se cuela como egreso)
```

### 12.3 Cómo funciona el pipeline

1. **Clasificar**: `clasificar_archivo` lee el título (`*** CORTE X|Z`, `CORTE DE CAJA X|Z`). Sin título → `NoEsCorte` (los tickets sueltos de la carpeta se omiten, contados, sin error).
2. **Partir**: se troza por marcadores de sección; ruido (`---`, `***`, `p0`, blancos) se tira.
3. **Extraer**: encabezado + 13 totales + renglones. Faltantes quedan en cero/None, no tumban el parseo.
4. **Verificar** (centavos exactos): `caja == ingresos − egresos` y `ventas == Σ renglones vendibles`. El descuadre se REPORTA en `advertencias`, no se tira el corte.
5. **Vincular** (backend): cada ARTICULO se cruza con el catálogo maestro (`productos`) con la misma maquinaria de tickets (exacto no-ambiguo → fuzzy 0.55/0.52). Si matchea guarda `producto_id` y acumula `vendido`; si es realmente nuevo se crea (stock 0). **El stock JAMÁS se descuenta**: son ventas de días pasados y el stock es el presente (descontarlo lo corrompería, peor si esos días ya entraron por tickets).

### 12.4 Base de datos (migraciones 0009 + 0010)

- `cortes_caja` NO se toca: queda solo para operativos en vivo (apertura/cierre X/Z del POS).
- `cortes_importados`: tipo, folio, estación, cajero, empresa, moneda, fecha, 9 totales en centavos, unidades, clientes, ruta, hash SHA256 UNIQUE, creado_en.
- `cortes_importados_items`: kind (ARTICULO|TICKET|INGRESO|EGRESO), nombre, cantidad, precio_unitario, subtotal, producto_id (FK lógica al catálogo, NULL = sin vincular).
- Idempotencia por hash de contenido: re-importar la misma carpeta omite duplicados.

### 12.5 Backend (yarvis-app/src-tauri/.../adminparser/cortes_import/)

```
cortes_import/
├── lectura.rs      # previsualizar_corte (un archivo, sin guardar)
├── vinculacion.rs  # cruce con catálogo (exacto + fuzzy, crear-nuevo, sin tocar stock)
├── importacion.rs  # importar_carpeta_cortes (clasifica, omite no-cortes, todo-o-nada por corte)
└── historial.rs    # get_cortes_importados + get_corte_importado_detalle (recalcula caja_ok/ventas_ok)
```

### 12.6 Interfaz (admin → Parseador → Cortes, espejo de Tickets)

Flujo **catálogo maestro → carpeta de cortes → historial** (el catálogo es el mismo componente de tickets: los artículos se vinculan contra ese inventario). Cada bloque trae su rango independiente; la lista arranca en Todos. El detalle muestra tarjetas de montos, badges ✓/! de verificación e items agrupados. Misma estética y mismos componentes base que tickets.

### 12.7 Lecciones (bugs que ya picaron)

- `Total en caja` vive DENTRO de la sección Egresos pero no es un egreso: ningún concepto real contiene "total", así se filtra.
- La empresa se busca SOLO antes de que empiecen las secciones; si no hay, es None (antes pescaba renglones de Ingresos).
- Estaciones en minúsculas (`caja_norte`) también matchean.
- `$.00` es cero válido, no error.
