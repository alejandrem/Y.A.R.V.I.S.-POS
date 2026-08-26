# Interconexion del Sistema — Y.A.R.V.I.S. POS

> Actualizado 2026-08-26. El sistema es un binario unico de Tauri: Frontend (React) -> Rust (comandos Tauri) -> IA en el crate local src-ia (mismo proceso). Sin sidecar Python, sin HTTP local.

---

## Principios que se conservan

Rust no falla: gestiona errores. Cada comando devuelve Result/Option; si la impresora no responde o la IA tarda, el usuario recibe un mensaje claro y la venta nunca se pierde.

Rust define el dominio con tipos. Structs serde que no permiten estados imposibles (models.rs).

Concurrencia con Tokio. Mientras el chat cloud/LLM responde (1-5 s), el cajero sigue cobrando: los comandos async no bloquean la UI.

Un solo escritor en SQLite. El backend Rust es el unico que escribe yarvis.db (pool sqlx, WAL). No hay segundo proceso.

---

## Flujo de comunicacion ACTUAL

| Origen | Destino | Mecanismo | Proposito |
|---|---|---|---|
| Frontend (React) | Backend Rust | Tauri IPC (invoke()) | Todo: ventas, inventario, tickets, chat, parseo, empleados, finanzas |
| Backend Rust | SQLite | sqlx (pool, WAL) | Lecturas + escrituras de yarvis.db (dinero en INTEGER centavos) |
| Backend Rust (chat) | Motor IA | crate src-ia en proceso | Chat cloud (HTTP+SSE) y local (llama.cpp Qwen GGUF) |
| Backend Rust (parser) | Motor IA | crate src-ia en proceso | Reglas de parseo + analisis LLM local bajo demanda |
| Backend Rust (predicciones) | Motor IA | crate src-ia en proceso | Holt-Winters local sin red |

No hay HTTP local. No hay puertos libres. No hay ai_service. No hay externalBin en tauri.conf.json. La CSP esta activa.

---

## Boot Sequence (arranque actual)

1. El usuario ejecuta yarvis-app (el binario unico).
2. lib.rs abre el pool SQLite (db/db.rs): crea el archivo yarvis.db si no existe, activa WAL, aplica migraciones (dos fases para foreign_keys), crea tablas.
3. Se registran 97 comandos Tauri en el invoke_handler y se inicia el job de alertas cada hora.
4. main.tsx monta React: el orquestador App.tsx decide la pantalla segun check_setup_done (comando real):
   - Paso 0 PrimerInicio: primer registro de administrador + tienda + empleado (solo se muestra una vez).
   - Paso 1 Login con seleccion de rol y contrasena.
   - Paso 2 AdminDashboard o Paso 3 EmployeeDashboard.
5. El LLM no carga al arrancar. Se carga lazy cuando el chat/parseo lo necesita (load_chat_model, analizar_ticket_con_ia), y el motor local comparte una unica instancia Qwen (1.5B Coder Instruct fine-tuneado, futuro 1.5B Coder).

---

## Flujo del Chat (nube + fallback local con tools)

1. El usuario escribe en adminyarvis o empleayarvis.
2. ChatWidget llama send_chat_stream o send_chat_message (Tauri command).
3. admintarvis/chat.rs delega en src-ia/motor-chat:
   - Cloud: cloud/apis_cloud/generacion.rs abre un stream HTTP/SSE a OpenCode Zen o Gemini. Los bloques think/response se separan (think.rs). Si el proveedor devuelve 429, la cola de fallback releva al siguiente proveedor (espera 2-4 s, max 3 modelos). Las 10 tools se resuelven via ciclo_tools.rs con re-inyeccion del resultado como mensaje role tool (MAX_RONDAS_TOOLS=3).
   - Local: llm/mod.rs genera con Qwen GGUF (llama.cpp) en CPU, con recortar_historial para no exceder 4096 tokens. Usa el mismo ejecutor de tools que cloud.
4. Roles: herramientas_rol.rs filtra TOOLS_SOLO_ADMIN (query_sales, compare_periods, get_restock_analysis) para empleados. El prompt es sugerencia; el guard es control de acceso real.
5. La respuesta llega al frontend por el mismo mecanismo (invoke no bloquea la caja).
6. Si todo lo cloud falla -> respuesta local. El usuario siempre recibe algo (degradacion graceful). Cancelacion via STREAM_CANCELADO atomico (stop_chat_stream).

No se usa RAG. Las tools consultan SQLite directamente con SQL parametrizado.

## Flujo del Parseador (reglas + LLM bajo demanda)

1. El admin abre el Modulo de Importacion Inteligente (ImportModule.tsx).
2. Sube TXT / CSV / Excel; se llaman los comandos parser_* (adminparser/).
3. src-ia/parseador_de_tickets aplica:
   - Reglas (cerebro/): filtrado de lineas (3 niveles), encabezados, fechas, pagos, totales, segmentacion.
   - Lectores (formatos/): CSV (auto-detect separador), Excel (calamine), TXT.
   - LLM (si aplica): analizar_ticket_con_ia con analisis local; mapeo de columnas confirmado por el usuario (ColumnMapper).
4. parsear_carpeta_stream procesa carpetas enteras con eventos al frontend y transaccion por archivo (rollback ante fallo).
5. Vincular con inventario -> vincular_inventario / guardar_vinculacion (SQLite, via Rust, TF-IDF + fuzzy como similitud interina).

## Flujo de Predicciones (Holt-Winters local)

1. Frontend de tickets o finanzas llama get_predictions o get_predicciones_financieras con horizonte 1..365.
2. Backend hace spawn_blocking y llama src-ia/predicciones/ventas.rs:predecir_ventas(ruta_db, horizonte).
3. ventas.rs lee ventas completadas agrupadas por dia (SUM total en centavos -> pesos), densifica dias sin venta con 0.
4. holt_winters.rs:predecir(serie, 7, horizonte) ajusta alpha/beta/gamma por grid 343 combos y genera pronostico con banda 95% z=1.96 * s * sqrt(k), recortado a >=0.
5. Retorna [{fecha, prediccion, minimo, maximo}] al frontend para graficas con bandas.

---

## Base de Conocimiento y busqueda semantica (PENDIENTE)

En la era Python existia knowledge_base con sqlite-vec y embeddings para busqueda semantica y RAG. En la migracion a Rust quedaron como stubs y la estrategia cambio:

- buscar_producto_similar (inventario / nueva venta) — STUB
- backfill_embeddings (importacion) — STUB

Plan actual: modelo de embeddings propio para busqueda semantica (no all-MiniLM, no ONNX externo). Interino: similitud TF-IDF + fuzzy en vinculador_inventario/similitud.rs. Las predicciones con Holt-Winters ya estan operativas, asi que solo quedan pendientes los embeddings.

## Relaciones por rol

- Administrador: front-admin/ + backadmin/ -> gestion total (ventas, inventario, tickets, finanzas, clientes, empleados, configuracion, yarvis). Ve finanzas, nomina y herramientas de admin.
- Empleado: front-empleado/ + backempleado/ -> caja (nueva venta), inventario de consulta, tickets/cortes propios, perfil, ajustes y chat. La mayoria reutiliza comandos admin registrados globalmente pero con filtro de rol en tools.

## Clima y prediccion

El diseno original correlacionaba el clima historico (frente frio -> +pan) durante el corte Z para alimentar predicciones. Sigue siendo objetivo para Holt-Winters, pero la API de clima y la tabla de correlaciones aun no se integran. No hay bloqueo para la caja.
