Plan de corrección — bugs e riesgos de concurrencia
FASE A — Datos silenciosamente incorrectos (parseador + profeta)

> ⚠️ NOTA DE ALCANCE (2026-08-13): solo se resolvieron los bugs **relacionados con el RAG**
> (FASE B1, C3, C4 y la parte de embeddings de C5). El resto queda pendiente.
> La separación entre `modelos_local/gestion_hardware.py` (chatbot) y
> `parseador_de_tickets/llm/analizador_llm.py` (parser de tickets) es **intencional**
> y NO es un bug: cada uno se encarga de su propio ciclo de vida de modelos.

A1. lote.py:280-360 — stream commitea ventas a medias; lógica duplicada
- Archivo/líneas: parseador_de_tickets/cerebro/lote.py:280-364 (stream) vs 129-212 (síncrono), más _insertar_venta en 58-95.
- Causa real: el stream abre conn.execute("BEGIN") por batch (:284) y commitea al final (:360). Si _insertar_venta falla a mitad de un archivo (ej. insert de un producto inexistente), el except en :344 solo cuenta el error pero NO hace rollback: la venta parcial y los UPDATEs de stock de ese batch quedan en la transacción y se commitean igual. La versión síncrona cierra conn sin commit → rollback implícito. Y el bucle completo de parseo está copiado en ambos sitios (~80 líneas), por eso ya divergieron.
- Solución: extraer un generador compartido _procesar_archivo(texto, archivo, mapeo, db_path) -> (items_ok, stats) que use la conexión del hilo, y que cada archivo haga with conn: o try: ... conn.rollback() en el except. El stream iterará ese generador por archivo (transacción por archivo), y el síncrono lo llamará igual. Elimina la duplicación y el commit parcial.

A3. analizador.py:248-268 — _es_linea_util descarta productos reales
- Archivo/líneas: parseador_de_tickets/cerebro/analizador.py:248-268.
- Causa real: los patrones de salto usan if patron in linea_lower (subcadena). Un producto llamado "GATORADE TOTAL", "CAJA DE MADERA", "CANTINA", "TOTALMAX" o "PRECIOS JUSTOS" cae en "total", "caja", "cant", "precio" y el ticket lo pierde silenciosamente.
- Solución: activar los patrones solo si la línea empieza por el patrón o si coincide como token (borde de palabra): re.search(rf'\b{re.escape(patron)}\b', linea_lower) para los totales/pagos, y dejar los encabezados de ticket como "la línea termina en dígitos/sin columnas numéricas". Los patrones de cabecera genéricos como calle, colonia no deben evaluarse en líneas que tienen al menos 3 columnas con números.


A4. lector_txt.py:53-60 — el volumen "600" se come el nombre
- Archivo/líneas: parseador_de_tickets/formatos/lector_txt.py:53-60.
- Causa real: _extraer_nombre_cantidad usa rsplit(None, 1); para "COCA-COLA 600 10 $10 $5" el último token es "5"→ ok, pero en patrones sin cantidad final el 600 (volumen) queda como último token e isdigit() lo trata como cantidad → nombre truncado a "COCA-COLA".
- Solución: solo tratar como cantidad si el último token es un entero "pequeño de pieza" (≤ 999) y además la línea tiene otras columnas numéricas después (precio) o el token restante es número; mejor aún: usar el patrón de la línea completo (_PATRON_*) que ya fija columnas y extraer cantidad de la columna cant explícita, no de rsplit. Fallback: no quitar el token si el resto del nombre ya tiene dígitos (volumen).
A5. profeta/predictor.py:126-129 + endpoints.py:9 — days sin validar

- Archivo/líneas: profeta/predictor.py:126-129; causa raíz en profeta/endpoints.py:7-9 (days: int = 7 sin rango).
- Causa real: el frontend o un cliente puede mandar days=0 → forecast.tail(0) devuelve lista vacía con status:"success" (parece error); days<0 → tail(-1) devuelve filas históricas como si fueran predicciones futuras. No hay validación.
- Solución: en PredictionRequest usar days: int = Field(default=7, ge=1, le=365) (pydantic) y en run_prediction cláusula de guardia if days < 1: return {"error": ...} por si se llama directo. Opcional: with self.mock no aplica; solo validar.


FASE B — Backend HTTP / nube
B1. embeddings/endpoints.py:76-104 — backfill 500 por descripcion + NULLs
- Archivo/líneas: chatbot/embeddings/endpoints.py:76-89 (query en :78, formato en :83).
- Causa real: el SELECT incluye productos.descripcion, columna que ningún otro módulo usa (los demás selects del esquema real no la tienen) → OperationalError en DBs reales. Además f"{p['precio_venta']:.2f}" y stock:.0f crashean con TypeError si son NULL.
- Solución: hacer la query robusta: SELECT id, nombre, COALESCE(descripcion,'') ... COALESCE(precio_venta,0), COALESCE(stock,0) ... envuelto en try/except con logger y, idealmente, verificar PRAGMA table_info(productos) para no depender de descripcion. Usar _conectar() con busy_timeout en vez de sqlite3.connect a secas.
- ✅ RESUELTO (2026-08-13): reescrito chatbot/embeddings/endpoints.py → /backfill construye el SELECT según PRAGMA table_info (solo columnas existentes), COALESCE en precios/stock, logger.exception, busy_timeout y cierre con `closing()`. Probado con schema completo y schema mínimo.

B2. gestion_hardware.py:36-55 — RAM: MemTotal ≠ disponible
- Archivo/líneas: chatbot/motor_chat/modelos_local/gestion_hardware.py:36-46 (get_ram_gb), usada en :49-55.
- Causa real: lee MemTotal (RAM total del sistema) pero puede_cargar_modelo y estado_modelos la reportan como "GB disponibles" y la comparan contra el requisito del modelo. En una máquina de 8 GB siempre deja cargar 1.7B aunque el proceso ya consuma 6 GB → carga falla/swap. El fallback inventado 8.0 tampoco es real.
- Solución: leer MemAvailable de /proc/meminfo (Linux), restar el uso del proceso actual si se puede (/proc/self/status VmRSS), y loggear/print en el except en vez de devolver 8.0 ciego.

B3. endpoints.py:264-274, 311-319 — SSE sin evento terminal
- Archivo/líneas: chatbot/motor_chat/endpoints.py:264-274 (nube) y 311-319 (local).
- Causa real: el break por cancelación en :264/:313 y el except en :274/:319 terminan el generador sin emitir {'done': True} ni event: end. Un cliente SSE que espera la señal de cierre se queda colgado (spinner infinito). Además, si el proveedor manda un error a mitad, :274 emite {'error':...} pero tampoco done.
- Solución: en ambos generadores, mover el yield de done a un finally (o emitirlo siempre tras el bucle, aun si _cancel_event o hubo error): yield data: {'done': True, 'cancelled': bool(...), 'model':...}. El cliente siempre recibe cierre.

B4. endpoints.py:211 — /chat nube devuelve el thinking crudo
- Archivo/líneas: chatbot/motor_chat/endpoints.py:199-215 (generar_completo en :211); la concatenación cruda está en apis_cloud.py:282-283.
- Causa real: generar_completo concatena todos los tokens ("".join(token for token,_ in ...)), incluidos los bloques thinking. El endpoint no aplica limpiar_think/_separar_think (la rama local sí limpia vía ejecutar_chat → limpiar_think). El usuario final ve el razonamiento intercalado en la respuesta corta.
- Solución: en apis_cloud.py, filtrar del stream solo los segmentos fuera de thinking — reutilizar _separar_think (o copiar el filtro) y reconstruir solo la parte "token", o directamente limpiar_think("".join(...)). Lo más limpio: generar_completo construye con "".join(t for k,t in _separar_think(stream, 1e9) if k == "token").

B5. apis_cloud.py:117-135 — reintento 400 duplica tokens
- Archivo/líneas: chatbot/motor_chat/modelos_API/apis_cloud.py:117-135.
- Causa real: el bucle for intentar_con_uso in (True, False) hace el POST con include_usage; si el servidor rechaza con 400 a mitad del stream (tras ya entregar tokens con yield from), el except reintenta sin usage y vuelve a emitir todo desde cero → el cliente ve los primeros tokens repetidos.
- Solución: validar include_usage con una primera llamada sin generar (HEAD/OPTIONS o un POST de 1 token que se descarta), o detectar el 400 antes del yield from comprobando el resp.status_code sin consumir líneas. Fallback pragmático: si el 400 llega después de haber cedido algún token, no reintentar (propagar el error); solo reintentar si aún no se cedieron tokens.

FASE C — Concurrencia

C1. endpoints.py:35,247 — _cancel_event global
- Archivo/líneas: chatbot/motor_chat/endpoints.py:35 (definición), :247 (clear), :264/:313 (consulta).
- Causa real: un Event único por proceso. El /stop de un cliente setea el evento de todos (cancela streams ajenos), y cada /chat_stream nuevo hace clear(), "des-cancelando" streams en curso. Es un bug de sesión-compartida.
- Solución: por-request: generador recibe su propio threading.Event creado en chat_stream; /stop cancela solo el actual (guardando referencia del último evento activo en un dict {stream_id: Event} con lock, o un registro con token). Al terminar el stream se limpia la entrada.

C2. endpoints.py:43-62 — timer descarga modelo en uso
- Archivo/líneas: chatbot/motor_chat/endpoints.py:40-62 (timer) + descargar_modelo en modelos_local/gestion_hardware.py:79-91.
- Causa real: a los 5 min, _descargar_por_inactividad cierra el Llama que un /chat_stream largo está iterando → el generador lanza al siguiente .next() (crash del stream). Además _registrar_actividad reinicia el mismo timer global sin lock.
- Solución: contador atómico de streams activos (con threading.Lock); el timer solo descarga si el contador es 0; y descargar_modelo debe dar AttributeError... mejor: que el timer sea por dato de actividad mínima con lock (_last_activity timestamp + thread que verifica), o pausar/cancelar el timer cuando hay stream activo.

C3. embeddings/endpoints.py:20,47,95,128 + profeta/endpoints.py:13 — event loop bloqueado
- Archivo/líneas: chatbot/embeddings/endpoints.py handlers async def en :16,38,56,122 haciendo model.encode (segundos) y sqlite3.connect; profeta/endpoints.py:12-17 llamando run_prediction (Prophet, minutos).
- Causa real: FastAPI ejecuta async def en el event loop único. Un encode o un fit de Prophet congelan todos los endpoints (health, chat, parser) mientras corren.
- Solución: declarar los handlers como def (síncronos) para que FastAPI los mande al threadpool: en embeddings/endpoints.py:16,38,56,122 y profeta/endpoints.py:12. Si hay código async obligatorio, await run_in_threadpool(...).
- ✅ RESUELTO (parte RAG, 2026-08-13): los 4 handlers de chatbot/embeddings/endpoints.py (/generar_embedding, /buscar_similar, /backfill, /insertar_knowledge) ahora son `def` síncronos → FastAPI los ejecuta en el threadpool. Pendiente: profeta/endpoints.py (no es RAG, no se tocó).

C4. embeddings/endpoints.py:67-115,131-137 — conexiones SQLite con fugas
- Archivo/líneas: chatbot/embeddings/endpoints.py: sqlite3.connect en :67 (sin close en :114), :104-107 (close solo feliz), :131-137.
- Causa real: conn.close() solo en el camino feliz; en el except la conexión queda abierta (fuga de FD) y sin PRAGMA busy_timeout frente a escrituras concurrentes → database is locked.
- Solución: delegar a consultas_db._conectar() (que ya reutiliza por hilo, tiene busy_timeout y cierra/reconecta), o usar with closing(sqlite3.connect(...)) + conn.execute("PRAGMA busy_timeout=5000"), y añadir logger.exception en cada except.
- ✅ RESUELTO (2026-08-13): /backfill e /insertar_knowledge usan `with closing(_conectar_db(...))` (cierre garantizado en cualquier camino) + PRAGMA busy_timeout=5000 + logger.exception en cada except.

C5. gestion_hardware.py:20-33 + embeddings/modelo.py:4-14 — carga global sin lock
- Archivo/líneas: gestion_hardware.py:20-22 (globals), :71-75 (globals()[attr] = model), :79-91 (descargar_modelo); embeddings/modelo.py:4-14 (_embedding_model + carga perezosa).
- Causa real: dos requests concurrentes que cargan el mismo modelo (o el primer embed) ejecutan Llama(...) / SentenceTransformer(...) dos veces en paralelo → doble ocupación de VRAM/RAM y delay. GIL no protege la carga I/O-bound.
- Solución: un threading.Lock por recurso; cargar bajo el lock y verificar de nuevo dentro (double-checked locking): en gestion_hardware.py un _lock_carga que envuelva :71-75, y en embeddings/modelo.py un _lock_embeddings con el doble chequeo. Precalentar opcional en lifespan del main.py.
- ✅ RESUELTO (parte RAG, 2026-08-13): chatbot/embeddings/modelo.py tiene `_embedding_lock` con double-checked locking en get_embedding_model(). Pendiente: la parte de gestion_hardware.py (Qwen local, no es RAG, no se tocó).
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