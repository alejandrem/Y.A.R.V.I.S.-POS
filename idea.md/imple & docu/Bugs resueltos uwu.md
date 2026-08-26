# Auditoria Completa de Y.A.R.V.I.S. IA — Veredicto de Claudio

> Nota de estado 2026-08-26: bitacora historica de la era Python (FastAPI). El motor ya se migro a Rust (src-ia); los archivos .py citados no existen. Los bugfixes fueron especificacion para el port y se conservaron en Rust (filtro 3 niveles, transacciones por archivo, separador -- del catalogo). Actualizacion: predicciones Holt-Winters ya operativas en src-ia/predicciones (get_predictions y get_predicciones_financieras ya no son stubs), busqueda semantica pendiente con modelo de embeddings propio (no all-MiniLM), y fine-tuning objetivo es Qwen2.5-Coder 1.5B Instruct para SQL/tools. Se mantiene como historial y lista de trampas conocidas.

Revisé **cada archivo** del proyecto. Aquí va el diagnóstico honesto, separando **bugs reales** de **cosas que parecen bugs pero NO lo son**.

---

## Calificacion General: 7.5 / 10

> [!TIP]
> **No es un proyecto roto.** Es un proyecto ambicioso y bien pensado con algunos bugs legítimos pendientes, pero la arquitectura base es sólida. Las cosas que "parecen bugs" en su mayoría son decisiones de diseño correctas que alguien sin contexto confundiría con errores.

---

## Lo Que Esta Bien (y que parece bug pero no lo es)

### 1. Los handlers `def` (no `async def`) en embeddings —  CORRECTO
```python
# chatbot/embeddings/endpoints.py
@router.post("/generar_embedding")
def generar_embedding(request: EmbeddingRequest):  # ← def, NO async def
```
**Parece bug:** "¿Por qué no es `async`? ¿No se está bloqueando?"
**Realidad:** FastAPI envía los `def` síncronos al **threadpool automáticamente**. Si fueran `async def`, `model.encode()` (que tarda segundos) bloquearía el event loop y congelaría TODO el servidor. **Esto es la forma correcta.**

---

### 2. El double-checked locking del modelo de embeddings —  CORRECTO
```python
# chatbot/embeddings/modelo.py
def get_embedding_model():
    global _embedding_model
    if _embedding_model is None:          # check 1 (sin lock, rápido)
        with _embedding_lock:
            if _embedding_model is None:  # check 2 (con lock, seguro)
                _embedding_model = SentenceTransformer(...)
```
**Parece bug:** "¿Por qué el doble `if`? Parece redundante."
**Realidad:** Es un patrón estándar de concurrencia. Sin el segundo `if`, dos hilos podrían pasar el primero y ambos cargarían el modelo (doble RAM). Está **perfectamente implementado.**

---

### 3. La descarga por inactividad con hilo daemon —  CORRECTO
```python
# chatbot/motor_chat/endpoints.py
def _vigilante_inactividad():
    while True:
        time.sleep(_INACTIVIDAD_CHECK_SEGUNDOS)
        # solo descarga si NO hay streams activos
        if activos > 0:
            continue
```
**Parece bug:** "¡Un `while True` en un hilo! ¡Se va a colgar!"
**Realidad:** Es un hilo daemon (muere con el proceso), revisa cada 15s, y respeta el contador de streams activos. **Es el patrón correcto** para este tipo de vigilancia, y ya corrigieron el bug original donde el timer global podía descargar modelos en uso (C2 resuelto ).

---

### 4. El `globals()[attr]` en gestion_hardware —  FUNCIONAL
```python
# gestion_hardware.py
globals()[attr_name] = model  # Línea 70
```
**Parece bug:** "¡`globals()` es sucio!"
**Realidad:** Es un patrón pragmático para 3 variables. Funciona correctamente. ¿Es lo más elegante? No. ¿Es un bug? **Tampoco.** Un dict sería más limpio, pero no es incorrecto.

---

### 5. El sistema de keywords dinámicas del catálogo —  INTELIGENTE
```python
# cache.py - Las keywords se regeneran de datos reales
_catalogo_keywords: set = set()  # Se llena con nombres/categorías de la DB
_empleado_keywords: set = set()  # Se llena con nombres de empleados reales
```
**Parece bug:** "¡Tiene keywords hardcodeadas!"
**Realidad:** Las keywords **estructurales** (como "producto", "stock", "venta") sí están en código, pero las de **catálogo y empleados se derivan automáticamente de la DB** cada 60 segundos. Es un sistema híbrido bien pensado.

---

### 6. El fallback local cuando la nube falla —  BUEN DISEÑO
```python
# endpoints.py
except Exception as e:
    return {"response": _fallback_local(...), "model_used": "local-fallback"}
```
**Parece bug:** "¡Silencia errores!"
**Realidad:** El usuario SIEMPRE recibe respuesta. Si Gemini/OpenCode fallan, cae a Qwen 0.5B local. Es **degradación graceful**, no silenciamiento de errores.

---

### 7. Los Event por-stream para cancelación (C1 resuelto) —  CORRECTO
```python
# Cada stream tiene su PROPIO Event
stream_id, cancel_event = _nuevo_stream_event()
```
**Parece bug:** "¿Por qué tanto boilerplate para /stop?"
**Realidad:** Antes había UN solo Event global que cancelaba todos los streams. Ahora cada stream tiene el suyo, y `/stop` solo cancela el indicado (o el más reciente). **Es el fix correcto.**

---

## Bugs Reales que Si Existen

### Bug 1: `_es_linea_util` descarta productos legítimos (A3) —  MEDIO (RESUELTO)
**Archivo:** [analizador.py](file:///home/ale/Proyectos/Y.A.R.V.I.S.-POS/yarvis-IA/parseador_de_tickets/cerebro/analizador.py#L239-L275)

```python
patrones_skip = ["total", "caja", "cant", "precio", ...]
for patron in patrones_skip:
    if patron in linea_lower:  # ← subcadena, no palabra completa
        return False
```

Un producto llamado **"GATORADE TOTAL"**, **"CAJA DE CERVEZA"** o **"CANTIMPLORA"** se descarta silenciosamente porque contienen "total", "caja", "cant". Se necesita `re.search(rf'\b{patron}\b', ...)` o reglas más finas.

---

### Bug 2: `_extraer_nombre_cantidad` se come volúmenes (A4) —  BAJO (RESUELTO)
**Archivo:** [lector_txt.py](file:///home/ale/Proyectos/Y.A.R.V.I.S.-POS/yarvis-IA/parseador_de_tickets/formatos/lector_txt.py#L53-L60)

```python
def _extraer_nombre_cantidad(texto_limpio):
    partes = texto_limpio.rsplit(None, 1)
    if posibles_numeros.isdigit():  # "600" ← lo confunde con cantidad
        return partes[0].strip(), int(posibles_numeros)
```

"COCA-COLA 600ML" → el "600" (volumen) se interpreta como cantidad y el nombre queda truncado. Fix: solo tratar como cantidad si es ≤ 999 y no tiene letras adyacentes.

---

### Bug 3: `days` sin validación en Prophet (A5) —  BAJO (RESUELTO)
**Archivo:** [profeta/endpoints.py](file:///home/ale/Proyectos/Y.A.R.V.I.S.-POS/yarvis-IA/profeta/endpoints.py#L7-L9)

```python
class PredictionRequest(BaseModel):
    db_path: str
    days: int = 7  # ← sin ge=1, le=365
```

`days=0` devuelve lista vacía con `status: "success"`. `days=-1` devuelve filas históricas como predicciones. Fix simple: `days: int = Field(default=7, ge=1, le=365)`.

---

### Bug 4: Stream de lote duplica lógica y puede commitear parcial (A1) —  ALTO (RESUELTO)
**Archivo:** [lote.py](file:///home/ale/Proyectos/Y.A.R.V.I.S.-POS/yarvis-IA/parseador_de_tickets/cerebro/lote.py#L264-L383)

La lógica de parseo está **copiada** entre `_procesar_carpeta_impl` (síncrono, ~80 líneas) y `event_generator` (stream, ~80 líneas). Además, el stream usa `BEGIN` + `commit` por batch, pero si `_insertar_venta` falla a mitad, no hace rollback → la venta parcial y los UPDATEs de stock se commitean con el resto del batch.
Bug 4 (A1, lote) —  SÍ resuelto: generador único _procesar_archivos compartido por síncrono y stream, transacción por archivo con rollback. Verificado con tests.

---

### Bug 5: `profeta/endpoints.py` sigue siendo `async def` (C3 parcial) —  MEDIO (RESUELTO)
**Archivo:** [profeta/endpoints.py](file:///home/ale/Proyectos/Y.A.R.V.I.S.-POS/yarvis-IA/profeta/endpoints.py#L12)

```python
@router.post("/recalcular_predicciones")
async def recalcular_predicciones(request: PredictionRequest):  # ← async!
    result = run_prediction(...)  # ← Prophet fit tarda MINUTOS
```

Mientras Prophet entrena, **todos los demás endpoints se congelan** (health, chat, parser). Fix: cambiar `async def` → `def` para que FastAPI lo mande al threadpool.

---

### Bug 6: `gestion_hardware.py` carga modelos sin lock (C5 parcial) —  BAJO (RESUELTO)
**Archivo:** [gestion_hardware.py](file:///home/ale/Proyectos/Y.A.R.V.I.S.-POS/yarvis-IA/chatbot/motor_chat/modelos_local/gestion_hardware.py#L54-L72)

```python
def cargar_modelo(model_key):
    if model_key == "0.5B" and _llm_0_5 is not None:
        return _llm_0_5
    # Sin lock → dos requests podrían cargar el mismo modelo en paralelo
    model = loader_fn()
    globals()[attr_name] = model
```

El modelo de embeddings ya tiene su lock (resuelto ), pero los Qwen locales no. Dos requests concurrentes podrían cargar el mismo modelo dos veces → doble RAM. En la práctica es raro porque el chat serializa con el `_marcar_uso_ia()`, pero técnicamente es un race condition.

---

### Bug 7: Typo menor en `prompts_api.py` —  COSMÉTICO (RESUELTO)
**Archivo:** [prompts_api.py](file:///home/ale/Proyectos/Y.A.R.V.I.S.-POS/yarvis-IA/chatbot/motor_chat/modelos_API/prompts_api.py#L16-L17)

```python
"Ayudas a atender empleados y responder sobre la existencia y falta de productos productos. "
#                                                                    ^^^^^^^^^ ^^^^^^^^^ duplicado
```

"productos productos" duplicado. Cosmético pero afecta la calidad del system prompt.

---

### Bug 8 (Nuevo — se le escapó a Claudio) — El patrón SIN_SEP roba productos con separador  BAJO-MEDIO

**Archivo:** yarvis-IA/parseador_de_tickets/formatos/lector_txt.py
**Tipo:** False match por orden de patrones (no es un bug de lógica, es un bug de precedencia).


#### 1. La línea problemática
Coca-Cola 600ML -- $25 $18
Según el encabezado del propio archivo (lector_txt.py:6), este es el formato oficial soportado:
Producto -- $VENTA $COSTO
O sea: -- es un separador, NO parte del nombre. El nombre debería quedar como COCA-COLA 600ML.
#### 2. Qué pasa realmente (el trace)
_parsear_linea_catalogo intenta los patrones en orden (líneas 75–216). Este es el orden real y el resultado con nuestro ejemplo:
0	Patrón
1	SIN_SEP_CANT
2	SIN_SEP
3	SIN_SEP_CANT_SINDOL
4	SIN_SEP_SINDOL
5	CANTIDAD_INICIO
6	PRODUCTO
- Evidencia instrumentada:
SIN_SEP: groups=('Coca-Cola 600ML --', '25', '18')   ← el ganador
PRODUCTO: groups=('Coca', None, None)                ← el que debía ganar
Por qué el #2 gana: _PATRON_SIN_SEP es ^(.+?)\s+\$precio1 \s+ \$precio2. El .+? perezoso crece hasta encontrar un espacio seguido de $. El primer \s+\$ aparece justo antes de $25, así que el grupo se traga todo lo anterior, incluido el -- (".", el comodín, acepta guiones). El patrón #2 valida encima (match anclado al inicio), se marca como válido, y hace continue → el #6 jamás se evalúa.
#### 3. Impacto real
Cada línea con separador del tipo X -- $venta $costo entra al sistema con nombre contaminado:
```
{'nombre': 'COCA-COLA 600ML --',  'stock': 0, 'precio_venta': 25.0, ...}
{'nombre': 'AGUA 1500 --',        'stock': 0, 'precio_venta': 20.0, ...}
{'nombre': 'PAN BLANCO 12 --',    'stock': 0, 'precio_venta': 15.0, ...}
```
Resultado: al guardarse en la DB, el producto se llama "COCA-COLA 600ML --" (con los guiones colgando). Luego es más difícil que el RAG/búsqueda semántica lo empareje por nombre exacto, y poluciona la lista de productos. No rompe nada (los precios y stock se leen bien), por eso es BAJO.
#### 4. Por qué es distinto de todos los bugs de Claudio
- No es el A4 (ese era _extraer_nombre_cantidad comiéndose volúmenes → ya corregido hoy).
- No es el A3 (analizador.py descartando líneas con "total"/"caja" → está en otro archivo).
- Es un tercer defecto en el mismo archivo lector_txt.py, que ni siquiera aparece en su auditoría: un conflicto de precedencia entre SIN_SEP (genérico) y PRODUCTO (específico para separadores). El específico debería ganar y no gana porque evalúa después.
El patrón base del problema: un patrón genérico que matchea más de lo que debería y se evalúa antes que el patrón específico — el clásico "greedy swallows the wrong thing" pero a nivel de orden de intentos.
#### 5. Propuestas de fix (cuando me digas)
1. Reordenar: evaluar el patrón con separador (#6) antes que el SIN_SEP genérico — el específico siempre gana.
2. Endurecer el #2: que el nombre capturado no pueda contener -=*~> al final (.rstrip("-=*~>") en group(1)), o excluir esos chars del . con una clase [^=*~>\$]+.
3. Combinar: reordenar y en _PATRON_PRODUCTO anclar con .match() para que el nombre no pueda robarse el separador (hoy usa .search(), que también contribuye).



## Resumen por Modulo

| Módulo | Archivos | Estado | Nota |
|--------|----------|--------|------|
| **chatbot/embeddings** | 2 |  Sólido | Bugs anteriores resueltos (locks, def síncrono, conexiones). Código limpio. |
| **chatbot/motor_chat/endpoints** | 1 |  Sólido | Bugs C1, C2, B3 ya resueltos. Buen manejo de concurrencia. |
| **chatbot/motor_chat/modelos_local** | 7 |  Sólido | C5 completo: lock en carga/descarga Qwen (double-checked locking). Cache y RAG bien hechos. |
| **chatbot/motor_chat/modelos_API** | 3 |  Sólido | Bug B5 ya resuelto. Buen fallback 429. Solo queda el typo cosmético "productos productos". |
| **parseador_de_tickets/cerebro** | 4 |  Sólido | A1 (lote unificado con rolback por archivo) + A3 (_es_linea_util 3 niveles) resueltos. |
| **parseador_de_tickets/formatos** | 3 |  Sólido | A4 (volúmenes), A2 y Bug 8 (SIN_SEP robando separador) resueltos. Parsers robustos. |
| **parseador_de_tickets/llm** | 2 |  Sólido | Escalado inteligente modelo unico 1.5B Coder. Buen diseño. |
| **profeta** | 2 |  Sólido | A5 (validación days) + C3 (handler síncrono, ya no bloquea) resueltos. |
| **main.py / scripts** | 3 |  Bien | Buen startup, health check, scripts de arranque completos. |
---

## Desglose de la Calificacion: 7.5/10

| Criterio | Puntos | Max |
|----------|--------|-----|
| **Arquitectura y diseño** | 9 | 10 |
| **Separación de responsabilidades** | 9 | 10 |
| **Documentación interna (docstrings)** | 9 | 10 |
| **Manejo de errores** | 8 | 10 |
| **Concurrencia y thread safety** | 9 | 10 |
| **Robustez del parseador** | 8 | 10 |
| **Tests** | 6 | 10 |
| **Corrección de bugs previos** | 10 | 10 |

### Lo que sube la nota:
- 🏗 Arquitectura modular excelente (modelos_local/ vs modelos_API/, cerebro/, formatos/)
- 📝 Docstrings de nivel profesional en casi todos los módulos
- 🔄 Bugs complejos de concurrencia (C1, C2, C3, C5, B3, B4, B5) **todos corregidos**
- 🤖 El sistema de escalado LLM (modelo unico 1.5B Coder) es muy inteligente
- 🛡 Fallbacks graceful en toda la cadena (nube→local, RAG→keywords→LIKE)
-  Fase A completa: A1 (lote + rollback), A3 (3 niveles), A4 (volúmenes), A5 (days) resueltos
- 🧪 Suite de regresión (2026-08-16): 56 tests en yarvis-IA/tests/ cubriendo A1, A3, A4, A5, C5 y bug 8

### Lo que baja la nota:
- 🧪 Cobertura parcial: faltan tests E2E del frontend y de los endpoints HTTP (SSE, chat); los 56 actuales son unitarios/lógicos
- ✏ Typo cosmético "productos productos" en prompts_api.py (sin impacto funcional)

---

Plan de corrección — bugs e riesgos de concurrencia
FASE A — Datos silenciosamente incorrectos (parseador + profeta)

A1. lote.py:280-360 — stream commitea ventas a medias; lógica duplicada
- Archivo/líneas: parseador_de_tickets/cerebro/lote.py:280-364 (stream) vs 129-212 (síncrono), más _insertar_venta en 58-95.
- Causa real: el stream abre conn.execute("BEGIN") por batch (:284) y commitea al final (:360). Si _insertar_venta falla a mitad de un archivo (ej. insert de un producto inexistente), el except en :344 solo cuenta el error pero NO hace rollback: la venta parcial y los UPDATEs de stock de ese batch quedan en la transacción y se commitean igual. La versión síncrona cierra conn sin commit → rollback implícito. Y el bucle completo de parseo está copiado en ambos sitios (~80 líneas), por eso ya divergieron.
- Solución: extraer un generador compartido _procesar_archivo(texto, archivo, mapeo, db_path) -> (items_ok, stats) que use la conexión del hilo, y que cada archivo haga with conn: o try: ... conn.rollback() en el except. El stream iterará ese generador por archivo (transacción por archivo), y el síncrono lo llamará igual. Elimina la duplicación y el commit parcial.

-  RESUELTO (2026-08-16): nuevo generador único `_procesar_archivos(archivos, mapeo, db_path)` que cede un dict por archivo (`{archivo, ok, motivo, items, duplicados, nuevos, existentes, venta_id, total}`). Cada archivo abre SU PROPIA transacción (`BEGIN` → `_insertar_venta` → `commit`), y ante cualquier fallo hace `conn.rollback()` en el `finally`, descartando venta parcial + UPDATEs de stock. `_procesar_carpeta_impl` (síncrono) y `event_generator` (stream) ahora solo acumulan contadores recorriendo el mismo generador: se eliminaron ~80 líneas duplicadas. Verificado: sync 2 archivos → 2 ventas/4 detalles, y un archivo con inserción que explota a mitad → no queda nada commiteado (rollback confirmado). El stream además deshueca una conexión por batch; ahora hay una sola conexión con cierre garantizado en `finally`.

A3. analizador.py:248-268 — _es_linea_util descarta productos reales
- Archivo/líneas: parseador_de_tickets/cerebro/analizador.py:248-268.
- Causa real: los patrones de salto usan if patron in linea_lower (subcadena). Un producto llamado "GATORADE TOTAL", "CAJA DE MADERA", "CANTINA", "TOTALMAX" o "PRECIOS JUSTOS" cae en "total", "caja", "cant", "precio" y el ticket lo pierde silenciosamente.
- Solución: activar los patrones solo si la línea empieza por el patrón o si coincide como token (borde de palabra): re.search(rf'\b{re.escape(patron)}\b', linea_lower) para los totales/pagos, y dejar los encabezados de ticket como "la línea termina en dígitos/sin columnas numéricas". Los patrones de cabecera genéricos como calle, colonia no deben evaluarse en líneas que tienen al menos 3 columnas con números.

-  RESUELTO (2026-08-16): `_es_linea_util` reescrito con 3 niveles. Nivel 1 (subcadena segura): frases que nunca están en un producto y patrones con ":" ("total:", "caja:", "iva:", "fecha:"...) → matchean el caso "CAJA: $500.00" sin tocar "CAJA DE MADERA". Nivel 2 (word-boundary): "total", "caja", "cant", "precio", "colonia", "iva", "tel"… solo descartan si la línea NO tiene columnas numéricas (≈cabecera) → proteger "GATORADE TOTAL", "CANTIMPLORA", "PRECIOS JUSTOS". Nivel 3: palabras de total/pago arrancando la línea ("TOTAL ---- $1,234.56", "EFECTIVO $500", "IVA 16%") → cabecera. Se eliminaron los substrings peligrosos "iva" (asesinaba "OLIVA"/"DIVA") y "colonia" (asesinaba el perfume "COLONIA 900 $150"). Verificado con 19 casos: 11 productos legítimos conservados + 8 cabeceras/pies descartados.


A4. lector_txt.py:53-60 — el volumen "600" se come el nombre
- Archivo/líneas: parseador_de_tickets/formatos/lector_txt.py:53-60.
- Causa real: _extraer_nombre_cantidad usa rsplit(None, 1); para "COCA-COLA 600 10 $10 $5" el último token es "5"→ ok, pero en patrones sin cantidad final el 600 (volumen) queda como último token e isdigit() lo trata como cantidad → nombre truncado a "COCA-COLA".
- Solución: solo tratar como cantidad si el último token es un entero "pequeño de pieza" (≤ 999) y además la línea tiene otras columnas numéricas después (precio) o el token restante es número; mejor aún: usar el patrón de la línea completo (_PATRON_*) que ya fija columnas y extraer cantidad de la columna cant explícita, no de rsplit. Fallback: no quitar el token si el resto del nombre ya tiene dígitos (volumen).

A5. profeta/predictor.py:126-129 + endpoints.py:9 — days sin validar
- Archivo/líneas: profeta/predictor.py:126-129; causa raíz en profeta/endpoints.py:7-9 (days: int = 7 sin rango).
- Causa real: el frontend o un cliente puede mandar days=0 → forecast.tail(0) devuelve lista vacía con status:"success" (parece error); days<0 → tail(-1) devuelve filas históricas como si fueran predicciones futuras. No hay validación.
- Solución: en PredictionRequest usar days: int = Field(default=7, ge=1, le=365) (pydantic) y en run_prediction cláusula de guardia if days < 1: return {"error": ...} por si se llama directo. Opcional: with self.mock no aplica; solo validar.

-  RESUELTO (2026-08-16): `PredictionRequest.days` ahora usa `Field(default=7, ge=1, le=365)` (pydantic) → la API rechaza days=0 o negativos con 422. Además `run_prediction` (predictor.py) tiene guardia propia: `if not isinstance(days, int) or days < 1 or days > 365: return {"error": ...}`, por si se llama directo desde otro módulo sin pasar por el endpoint.


FASE B — Backend HTTP / nube
B1. embeddings/endpoints.py:76-104 — backfill 500 por descripcion + NULLs
- Archivo/líneas: chatbot/embeddings/endpoints.py:76-89 (query en :78, formato en :83).
- Causa real: el SELECT incluye productos.descripcion, columna que ningún otro módulo usa (los demás selects del esquema real no la tienen) → OperationalError en DBs reales. Además f"{p['precio_venta']:.2f}" y stock:.0f crashean con TypeError si son NULL.
- Solución: hacer la query robusta: SELECT id, nombre, COALESCE(descripcion,'') ... COALESCE(precio_venta,0), COALESCE(stock,0) ... envuelto en try/except con logger y, idealmente, verificar PRAGMA table_info(productos) para no depender de descripcion. Usar _conectar() con busy_timeout en vez de sqlite3.connect a secas.

-  RESUELTO (2026-08-13): reescrito chatbot/embeddings/endpoints.py → /backfill construye el SELECT según PRAGMA table_info (solo columnas existentes), COALESCE en precios/stock, logger.exception, busy_timeout y cierre con `closing()`. Probado con schema completo y schema mínimo.

B3. endpoints.py:264-274, 311-319 — SSE sin evento terminal
- Archivo/líneas: chatbot/motor_chat/endpoints.py:264-274 (nube) y 311-319 (local).
- Causa real: el break por cancelación en :264/:313 y el except en :274/:319 terminan el generador sin emitir {'done': True} ni event: end. Un cliente SSE que espera la señal de cierre se queda colgado (spinner infinito). Además, si el proveedor manda un error a mitad, :274 emite {'error':...} pero tampoco done.
- Solución: en ambos generadores, mover el yield de done a un finally (o emitirlo siempre tras el bucle, aun si _cancel_event o hubo error): yield data: {'done': True, 'cancelled': bool(...), 'model':...}. El cliente siempre recibe cierre.

-  RESUELTO (2026-08-16): ambos generadores (nube y local) emiten SIEMPRE el evento `{'done': True, 'cancelled': bool, 'model': ...}` al terminar, tanto en final normal, por cancelación como tras un error. El `done` nunca se emite dentro del `finally` (para no crashear con GeneratorExit si el cliente se desconecta); se imprime justo antes del retorno natural del generador.


B4. endpoints.py:211 — /chat nube devuelve el thinking crudo
- Archivo/líneas: chatbot/motor_chat/endpoints.py:199-215 (generar_completo en :211); la concatenación cruda está en apis_cloud.py:282-283.
- Causa real: generar_completo concatena todos los tokens ("".join(token for token,_ in ...)), incluidos los bloques thinking. El endpoint no aplica limpiar_think/_separar_think (la rama local sí limpia vía ejecutar_chat → limpiar_think). El usuario final ve el razonamiento intercalado en la respuesta corta.
- Solución: en apis_cloud.py, filtrar del stream solo los segmentos fuera de thinking — reutilizar _separar_think (o copiar el filtro) y reconstruir solo la parte "token", o directamente limpiar_think("".join(...)). Lo más limpio: generar_completo construye con "".join(t for k,t in _separar_think(stream, 1e9) if k == "token").

-  RESUELTO (2026-08-16): `_separar_think` se movió de endpoints.py a modelos_local/prompts.py (compartido). `generar_completo` (apis_cloud.py:385+) ahora reconstruye solo los segmentos `"token"`, descartando los bloques `thinking...response`. Verificado con un proveedor simulado.


B5. apis_cloud.py:117-135 — reintento 400 duplica tokens
- Archivo/líneas: chatbot/motor_chat/modelos_API/apis_cloud.py:117-135.
- Causa real: el bucle for intentar_con_uso in (True, False) hace el POST con include_usage; si el servidor rechaza con 400 a mitad del stream (tras ya entregar tokens con yield from), el except reintenta sin usage y vuelve a emitir todo desde cero → el cliente ve los primeros tokens repetidos.
- Solución: validar include_usage con una primera llamada sin generar (HEAD/OPTIONS o un POST de 1 token que se descarta), o detectar el 400 antes del yield from comprobando el resp.status_code sin consumir líneas. Fallback pragmático: si el 400 llega después de haber cedido algún token, no reintentar (propagar el error); solo reintentar si aún no se cedieron tokens.


-  RESUELTO (2026-08-16): `_iter_openai_compatible` rastrea `cedio_tokens`; solo reintenta sin `include_usage` si el 400 llegó ANTES de ceder tokens. Si llega a mitad de stream, propaga el error y el cliente nunca ve salida repetida (el fallback local de /chat_stream lo absorbe). Verificado con un stream simulado que falla a mitad.



FASE C — Concurrencia

C1. endpoints.py:35,247 — _cancel_event global
- Archivo/líneas: chatbot/motor_chat/endpoints.py:35 (definición), :247 (clear), :264/:313 (consulta).
- Causa real: un Event único por proceso. El /stop de un cliente setea el evento de todos (cancela streams ajenos), y cada /chat_stream nuevo hace clear(), "des-cancelando" streams en curso. Es un bug de sesión-compartida.
- Solución: por-request: generador recibe su propio threading.Event creado en chat_stream; /stop cancela solo el actual (guardando referencia del último evento activo en un dict {stream_id: Event} con lock, o un registro con token). Al terminar el stream se limpia la entrada.

-  RESUELTO (2026-08-16): cada /chat_stream crea su propio Event vía `_nuevo_stream_event()` y lo registra en `_registry["streams"]` bajo `_streams_lock`. `/stop` cancela solo el más reciente (o el indicado por `?stream_id=`); ya no existe el `clear()` global que des-cancelaba streams ajenos. El `finally` del generador llama `_terminar_stream()` para limpiar registro y contador.

C2. endpoints.py:43-62 — timer descarga modelo en uso
- Archivo/líneas: chatbot/motor_chat/endpoints.py:40-62 (timer) + descargar_modelo en modelos_local/gestion_hardware.py:79-91.
- Causa real: a los 5 min, _descargar_por_inactividad cierra el Llama que un /chat_stream largo está iterando → el generador lanza al siguiente .next() (crash del stream). Además _registrar_actividad reinicia el mismo timer global sin lock.
- Solución: contador atómico de streams activos (con threading.Lock); el timer solo descarga si el contador es 0; y descargar_modelo debe dar AttributeError... mejor: que el timer sea por dato de actividad mínima con lock (_last_activity timestamp + thread que verifica), o pausar/cancelar el timer cuando hay stream activo.

-  RESUELTO (2026-08-16): el threading.Timer global se reemplazó por un hilo daemon (`_vigilante_inactividad`) que revisa cada 15 s la inactividad (`_ultima_actividad`, con `_actividad_lock`) y un contador `_registry["activos"]` de llamadas/streams usando el motor (`_marcar_uso_ia`/`_liberar_uso_ia`). Solo descarga si no hay uso activo, y `_terminar_stream()` reinicia la ventana al finalizar cada stream.



C3. embeddings/endpoints.py:20,47,95,128 + profeta/endpoints.py:13 — event loop bloqueado
- Archivo/líneas: chatbot/embeddings/endpoints.py handlers async def en :16,38,56,122 haciendo model.encode (segundos) y sqlite3.connect; profeta/endpoints.py:12-17 llamando run_prediction (Prophet, minutos).
- Causa real: FastAPI ejecuta async def en el event loop único. Un encode o un fit de Prophet congelan todos los endpoints (health, chat, parser) mientras corren.
- Solución: declarar los handlers como def (síncronos) para que FastAPI los mande al threadpool: en embeddings/endpoints.py:16,38,56,122 y profeta/endpoints.py:12. Si hay código async obligatorio, await run_in_threadpool(...).
-  RESUELTO (parte RAG, 2026-08-13): los 4 handlers de chatbot/embeddings/endpoints.py (/generar_embedding, /buscar_similar, /backfill, /insertar_knowledge) ahora son `def` síncronos → FastAPI los ejecuta en el threadpool. Pendiente: profeta/endpoints.py (no es RAG, no se tocó).
-  RESUELTO (completo, 2026-08-16): profeta/endpoints.py `/recalcular_predicciones` ahora es `def` síncrono (verificado con `inspect.iscoroutinefunction` → False). Mientras Prophet entrena, health/chat/parser siguen respondiendo porque el fit corre en el threadpool de FastAPI.

C4. embeddings/endpoints.py:67-115,131-137 — conexiones SQLite con fugas
- Archivo/líneas: chatbot/embeddings/endpoints.py: sqlite3.connect en :67 (sin close en :114), :104-107 (close solo feliz), :131-137.
- Causa real: conn.close() solo en el camino feliz; en el except la conexión queda abierta (fuga de FD) y sin PRAGMA busy_timeout frente a escrituras concurrentes → database is locked.
- Solución: delegar a consultas_db._conectar() (que ya reutiliza por hilo, tiene busy_timeout y cierra/reconecta), o usar with closing(sqlite3.connect(...)) + conn.execute("PRAGMA busy_timeout=5000"), y añadir logger.exception en cada except.
-  RESUELTO (2026-08-13): /backfill e /insertar_knowledge usan `with closing(_conectar_db(...))` (cierre garantizado en cualquier camino) + PRAGMA busy_timeout=5000 + logger.exception en cada except.

C5. gestion_hardware.py:20-33 + embeddings/modelo.py:4-14 — carga global sin lock
- Archivo/líneas: gestion_hardware.py:20-22 (globals), :71-75 (globals()[attr] = model), :79-91 (descargar_modelo); embeddings/modelo.py:4-14 (_embedding_model + carga perezosa).
- Causa real: dos requests concurrentes que cargan el mismo modelo (o el primer embed) ejecutan Llama(...) / SentenceTransformer(...) dos veces en paralelo → doble ocupación de VRAM/RAM y delay. GIL no protege la carga I/O-bound.
- Solución: un threading.Lock por recurso; cargar bajo el lock y verificar de nuevo dentro (double-checked locking): en gestion_hardware.py un _lock_carga que envuelva :71-75, y en embeddings/modelo.py un _lock_embeddings con el doble chequeo. Precalentar opcional en lifespan del main.py.
-  RESUELTO (parte RAG, 2026-08-13): chatbot/embeddings/modelo.py tiene `_embedding_lock` con double-checked locking en get_embedding_model(). Pendiente: la parte de gestion_hardware.py (Qwen local, no es RAG, no se tocó).
-  RESUELTO (completo, 2026-08-16): gestion_hardware.py ahora tiene `_carga_lock` (threading.Lock) que protege carga + descarga de los 3 Qwen. `cargar_modelo()` usa double-checked locking (primer chequeo sin lock, segundo bajo el lock) y `descargar_modelo()` usa el mismo lock. Verificado con 20 threads concurrentes cargando 0.5B → el loader se ejecutó 1 sola vez.
Nota sobre el punto ya resuelto
buscar_semantico ignora db_path — este quedó resuelto en la sesión anterior: quité el parámetro db_path de la firma motor_rag.py:22 y ahora _conectar() resuelve la ruta automáticamente. Ya no hay búsqueda sobre "otra DB" silenciosa; el contrato es único (si quieres búsqueda multi-DB real, sería un feature nuevo, no un bug).
Nota: tras la reestructuración, motor_rag.py vive en chatbot/motor_chat/modelos_local/motor_rag.py. El endpoint /backfill sigue aceptando db_path explícito (Rust lo manda) pero con cierre garantizado y busy_timeout.

Orden de implementación sugerido (por impacto/poco riesgo)
1. A2, A5 (cambios de condición/validación, 2 líneas cada uno) → verificación unitaria fácil.
2. B3, B5 (SSE: cierre garantizado + reintento sin duplicar) → verificación con curl.
3. A3, A4 (parseador: no perder productos reales) → probar con tickets de ejemplo.
4. C3, C4, C5 (bloqueo de loop, fugas, locks) → riesgo medio, requiere probar concurrencia.
5. B1, B2 (backfill COALESCE/schema check, RAM MemAvailable).
6. A1, C1, C2 (los más invasivos: refactor del stream de lote y aislamiento de sesión) → por último, requieren refactores y pruebas end-to-end.

---

## Fase F — Frontend Finanzas (Tauri + React)

### Bug F1: Dashboard de Finanzas muestra $0.00 en todos los KPIs — ALTO (RESUELTO)

**Archivo:** `yarvis-app/src/front-admin/ventanas/adminfinanzas/finanzas.tsx`

**Causa raiz (doble):**

1. **Rango de fechas por defecto demasiado angosto:** `rangoPorDefecto()` usaba 30 dias hacia atras. Las funciones del backend (`get_resumen_periodo`, `get_datos_grafica_pl`, `get_gastos_por_categoria`, etc.) reciben las fechas del frontend y filtran con `WHERE date(fecha) BETWEEN ? AND ?`. Si no hay ventas completadas ni pagos en los ultimos 30 dias, todo retorna $0. La grafica "Ventas vs Gastos" SÍ mostraba datos porque `get_ventas_vs_gastos_mensual` calcula sus propias fechas internamente en Rust (6 meses hacia atras con `chrono::Local::now()`), ignorando el rango del frontend.

2. **`Promise.all` bloqueante:** `cargarResumen()` usaba `Promise.all([get_resumen_periodo, get_punto_equilibrio])`. Si UNA de las dos fallaba (auth, BD, schema), la entera fallaba y NINGUNO de los dos estados se seteaba. `cargarGraficas()` usaba el mismo patron con 4 invokes: si `get_datos_grafica_pl` fallaba, ni `get_ventas_vs_gastos_mensual` ni `get_gastos_por_categoria` ni `get_tendencia_cortes_z` se seteaban. Los `catch {}` vacios tragaban los errores silenciosamente.

**Por que era confuso:**
- El usuario veia $0.00 en KPIs y "SIN DATOS" en graficas
- Pero la barra "Ventas vs Gastos" mostraba datos (porque usa fechas propias)
- No habia errores visibles en la UI (los catch tragaban todo)
- Parecia que el backend no funcionaba, pero en realidad funcionaba: el frontend pasaba fechas que no incluian datos

**Solucion:**

1. `rangoPorDefecto()`: cambiado de 30 dias a 6 meses (`ini.setMonth(ini.getMonth() - 6)`)
2. Cada invoke ahora es independiente con su propio `try/catch`: si `get_resumen_periodo` falla, las graficas igual se cargan
3. `cargarPuntoEq()` separado de `cargarResumen()` para que no se bloqueen mutuamente
4. `Promise.allSettled()` en `cargarTodo()` en vez de `Promise.all()`
5. Todos los `catch` ahora tienen `console.error` con tag `[FINANZAS]` para debuggar desde la consola del navegador (F12)

**Archivos modificados:**
- `yarvis-app/src/front-admin/ventanas/adminfinanzas/finanzas.tsx` — rango, carga de datos, error handling
- `yarvis-app/src/front-admin/types.ts` — 15 interfaces TypeScript nuevas para finanzas

**Leccion aprendida:** Cuando el frontend y el backend calculan rangos de fechas por separado, pueden divergir silenciosamente. Idealmente el frontend deberia enviar el rango al backend y el backend deberia usar ese rango en TODAS las funciones (como ya hace `get_resumen_periodo`). Las funciones que calculan sus propias fechas (como `get_ventas_vs_gastos_mensual` y `get_punto_equilibrio`) crean una fuente de verdad inconsistente.

---

### Bug B1: `npm run tauri build` falla al generar el AppImage (`failed to run linuxdeploy`) — MEDIO (RESUELTO)

**Contexto:** Arch Linux. El build compila perfecto (binario release, `.deb` y `.rpm` se generan), pero el paso final del bundle AppImage muere con:
```
failed to bundle project `failed to run linuxdeploy`
```

**Causa raiz:**

El bundler de Tauri descarga `linuxdeploy-x86_64.AppImage` a `src-tauri/target/cache/` y lo EJECUTA para armar el AppImage. Los AppImage requieren FUSE (`fusermount`) para montarse a si mismos al ejecutarse. En el sistema no habia `fuse2` instalado, asi que linuxdeploy no podia ni arrancar.

Diagnostico rapido:
```bash
fusermount --version   # -> "fusermount no encontrado"
pacman -Q fuse2        # -> no instalado
```

**Solucion (cualquiera de las dos):**

1. La limpia — instalar FUSE (requiere reiniciar sesion si es la primera vez):
```bash
sudo pacman -S fuse2
npm run tauri build
```

2. La portable — sin instalar nada, decirle al AppImage que se extraiga solo en vez de montarse:
```bash
APPIMAGE_EXTRACT_AND_RUN=1 npm run tauri build
```

**Importante NO confundir:** el error aparece AL FINAL del output pero NO invalida el build. El binario de produccion (`target/release/yarvis-app`), el `.deb` y el `.rpm` ya estaban generados y funcionan. El AppImage es solo un formato mas de distribucion (el mas portable: un solo archivo sin instalacion).

**Leccion aprendida:** leer la ultima linea de un fallo de bundling con lupa pero sin panico: Tauri genera varios formatos de paquete en secuencia y un fallo en el ultimo no tira los anteriores. Y en Arch, `fuse2` NO viene por defecto aunque `fuse3` si venga en algunos sistemas — linuxdeploy todavia usa FUSE 2.
