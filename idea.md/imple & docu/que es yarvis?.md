# Y.A.R.V.I.S. POS — Documentacion Completa de Implementacion

> Actualizado 2026-08-26: IA 100% Rust (src-ia), sin sidecar Python. Chat con fine-tuning + tools SQL, predicciones Holt-Winters operativas, busqueda semantica pendiente con modelo propio. Ver migracion-rust.md.

## Indice

1. Vision General
2. Arquitectura del Sistema
3. Estructura de Archivos
4. Frontend (React + TypeScript)
5. Backend Rust (Tauri)
6. Motor IA en Rust (src-ia)
7. Modelos de IA
8. Base de Datos
9. Comandos Tauri (Rust)
10. Casos pendientes (stubs)
11. Problemas Resueltos

---

## 1. Vision General

Y.A.R.V.I.S. POS es un sistema de punto de venta de escritorio con inteligencia artificial, pensado para tiendas medianas y pequenas de Mexico. Es un binario unico de Tauri v2 (React + Rust) con motor de IA nativo.

Capacidades:

- Registro de ventas (POS de caja del empleado) y gestion de inventario con CRUD completo.
- Atencion a clientes con CRM basico (perfil, historial).
- Cortes de caja X/Z, gastos recurrentes, alertas financieras, metricas y exportacion.
- Empleados: metas/bonos, turnos, salario, resumen de ventas, asistencia y horas extra.
- Parseo de tickets y catalogos (TXT/CSV/Excel) con reglas + LLM local para mapeo automatico y lotes con streaming.
- Chat con IA: cloud (OpenCode Zen/Gemini) con fallback a local (Qwen2.5-Coder 1.5B Instruct fine-tuneado (unico modelo local)) y 10 tools de consulta.
- Predicciones de ventas con Holt-Winters e intervalos de confianza al 95% (src-ia/predicciones).

Stack verificado:
- Frontend: React 19.1 + TypeScript 5.8 + Tailwind CSS 3.4 + Vite 7 (+ recharts, morphicons, react-markdown).
- Backend: Rust, Tauri 2.11, sqlx 0.8 (SQLite), tokio, serde, argon2, reqwest.
- IA: crate src-ia — chat local con llama-cpp-4 (Qwen GGUF), chat cloud con HTTP/SSE y ciclo de tools, parseador y predicciones.
- Base de datos: SQLite (WAL) via sqlx, dinero en INTEGER centavos.
- Seguridad: Argon2id, Google OAuth PKCE, CSP activa, API keys en archivo 0600.

---

## 2. Arquitectura del Sistema

```
  Frontend (React + Vite)  --invoke-->  Backend Rust (Tauri, 97 comandos)  -->  SQLite (WAL)
         |                                        |
         +---- respuesta IPC <--------------------+
   Motor IA (crate src-ia, en proceso): chat cloud (SSE) + chat local (llama.cpp) + parseador + predicciones.
```

Ciclo de vida:
1. run.sh/run.bat -> npm run tauri dev (o el binario empaquetado via yarvis-app/build.sh).
2. lib.rs inicializa SQLite (WAL, tablas, migraciones) y registra los comandos.
3. App.tsx decide la pantalla por check_setup_done (PrimerInicio / Login / Dashboard).
4. El frontend se comunica con Rust via invoke() (IPC nativo; sin HTTP ni puertos).
5. La IA corre dentro del mismo proceso (crate src-ia); el LLM local se carga bajo demanda y comparte cache con el parseador.

---

## 3. Estructura de Archivos

Ver el arbol completo en arquitectura.md. Resumen:

- src-ia/ — crate Rust con el motor de IA (parseador + chat cloud/local + predicciones).
- yarvis-app/src/ — frontend (front-admin, front-empleado, hooks).
- yarvis-app/src-tauri/src/backventanas/ — comandos Tauri por dominio (backadmin/backempleado).
- doc/ — documentacion (antes idea.md).

---

## 4. Frontend (React + TypeScript)

### 4.1 App.tsx — Orquestador de pantallas

Estados de paso: 0 = PrimerInicio, 1 = Login, 2 = AdminDashboard, 3 = EmployeeDashboard. La primer pantalla depende de check_setup_done.

### 4.2 PrimerInicio.tsx — Setup

Alta de administrador (nombre + contrasena con confirmacion), tienda (nombre/identidad) y empleados opcionales (+ AGREGAR EMPLEADO con nombre + contrasena). Solo se muestra una vez (hasta que haya admin).

### 4.3 Login (App.tsx)

- Dos botones de rol: ADMINISTRADOR y EMPLEADO.
- Campo de contrasena con ojo abierto/cerrado y boton ENTRAR AL POS.

### 4.4 front-admin/ — Panel del Administrador

- AdminDashboard.tsx: sidebar + enrutador (ventas, inventario, tickets, finanzas, clientes, empleados, configuracion, yarvis).
- adminconfig/: Configuracion refactorizada en componentes (ConfigHeader, IdentityForm, SecurityForm, AppearanceForm, importmodule/) + hooks (useAdminData, useParserActions).
- parseadodetickets/: BatchProcessor (lotes con streaming), ColumnMapper (mapeo con IA), CatalogosParseados.
- adminfinanzas/: dashboard, alertas, cortes X/Z, gastos, graficas (usa get_predicciones_financieras ya operativo), metricas.
- admininventario/: CRUD + importar catalogo + busqueda semantica (stub TF-IDF interino).
- adminempleados/: empleados + modales de edicion, metas y turnos (ya subdividido, 271 lineas).
- adminyarvis/: chat (ChatWidget).
- adminticket/: tickets + graficas (usan get_predictions ya operativo).

### 4.5 front-empleado/ — Punto de Venta

- EmployeeDashboard.tsx: nueva venta, inventario, tickets/cortes, clientes, perfil, yarvis, ajustes.
- nueva_venta.tsx: carrito con busqueda (usa buscar_producto_similar — stub), modal de venta y vista de ticket. Ya modularizado (198 lineas).
- empleaperfil/perfil.tsx: perfil y asistencia (130 lineas tras refactorizacion).

### 4.6 Hooks globales

- ParserContext.tsx: estado global del parseo (items, modo, analisis LLM).
- ThemeContext.tsx/useTheme.ts: temas claro/oscuro.

---

## 5. Backend Rust (Tauri)

### 5.1 lib.rs — Setup principal

Inicializa DB, registra 97 comandos en el invoke_handler, plugins (opener, dialog), job de alertas cada hora, tracing con RUST_LOG.

### 5.2 db.rs — Inicializacion de SQLite

Tablas principales: usuarios (Argon2), productos, ventas, detalle_ventas, clientes, ventas_diarias, cortes_caja, predicciones_futuras, catalogos_importados, gastos. WAL activado. Migraciones con foreign_keys off durante migracion y on en operacion.

### 5.3 Modulos backventanas/

| Modulo | Contenido |
|---|---|
| backadmin/adminconfig | auth (setup + login + datos admin/empleado, cerrar_sesion), google (OAuth) |
| backadmin/admininventory | CRUD inventario, importar_catalogo, stubs embeddings (buscar_producto_similar, backfill_embeddings) |
| backadmin/adminparser | parseo TXT/CSV/Excel, carpetas, vinculacion, modelos |
| backadmin/admintickets | tickets, cortes; get_predictions operativo (Holt-Winters) |
| backadmin/adminfinanzas | gastos, cortes X/Z, alertas, metricas, graficas (incluye get_predicciones_financieras operativo), export stubs |
| backadmin/adminempleados | empleados + modales (metas, turnos) |
| backadmin/admintarvis | chat (send_chat_message/stream, modelos, status, ciclo_tools, cancelacion, herramientas_rol, rutas) |
| backempleado | venta nueva (completar_venta, get_next_ticket_number), perfil y asistencia |

---

## 6. Motor IA en Rust (src-ia)

### 6.1 parseador_de_tickets/

- cerebro/: reglas (analizador_tickets), filtrador (3 niveles), parseador_masivo (lotes SSE, transaccion por archivo), vinculador_inventario (TF-IDF + fuzzy).
- formatos/: lector CSV, Excel (calamine), TXT.
- rutas/: resolucion de modelos GGUF + analisis LLM (analizar_ticket, generar_bajo_lock), deteccion de ~/.lmstudio/models.

### 6.2 motor-chat/

- cloud/: proveedores (OpenCode Zen, Gemini), generacion (completo/stream), catalogo de modelos, lector SSE, cola de fallback 429, separador think/response, variables/API keys.
- llm/: Qwen2.5-Coder 1.5B Instruct via llama-cpp-4 (feature llm-local, CPU, ventana 4096, fine-tuneado, recortar_historial); tools/ con 10 herramientas y dataset tools_arreglado.jsonl.

### 6.3 predicciones/

- holt_winters.rs: suavizado exponencial triple aditivo, periodo 7, grid 343 combos, banda 95% z=1.96, recorte a >=0.
- ventas.rs: capa de datos que lee ventas completadas, agrupa por dia, densifica huecos con 0 y genera puntos con fecha (YYYY-MM-DD, prediccion, minimo, maximo).

---

## 7. Modelos de IA

### 7.1 LLM Local (chat + parseo)

- Qwen2.5-Coder 1.5B Instruct GGUF fine-tuneado — unico modelo local; se carga bajo demanda (chat y analisis de tickets) via llama.cpp y funciona offline sin internet. Ruta configurable via set_local_model_path; resolucion en src-ia/rutas/rutas_modelos_*. Ventana 4096, recorte conservador de historial.

### 7.2 Cloud

- OpenCode Zen / Gemini — HTTP + SSE, con relevo automatico ante 429, max_tokens 39800 (Google con tope separado 8192), read_timeout de inactividad 90s. El thinking se separa del texto de respuesta.

### 7.3 Embeddings / Busqueda semantica

- Sin RAG. buscar_producto_similar y backfill_embeddings son stubs que devuelven error claro. Hoy la vinculacion usa TF-IDF + fuzzy sin vectores. Plan: modelo de embeddings propio (no all-MiniLM), con entrenamiento y pipeline local pendiente.

### 7.4 Predicciones

- Holt-Winters operativo. Reemplaza a Prophet. Sin dependencias externas. Horizontes 1..365 validados.

---

## 8. Base de Datos

SQLite, un archivo (yarvis.db), modo WAL, acceso asincrono con sqlx. Unico escritor: Rust. Dinero en INTEGER centavos (conversion en dinero.rs). Tablas: usuarios, productos, ventas, detalle_ventas, clientes, ventas_diarias, cortes_caja, predicciones_futuras, catalogos_importados, gastos_recurrentes, etc.

---

## 9. Comandos Tauri (Rust)

97 comandos #[tauri::command] en lib.rs:38. Resumen por modulo:

Auth / Setup (adminconfig/auth.rs, google.rs)
check_setup_done, guardar_admin, validar_login_admin, get_admin_data, update_admin_data, guardar_empleado, validar_login_empleado, cerrar_sesion, login_con_google

Inventario (admininventory/inventory.rs)
get_inventory, add_inventory_item, update_inventory_item, delete_inventory_item, importar_catalogo, get_catalogos_importados, get_productos_por_catalogo, buscar_producto_similar (stub), backfill_embeddings (stub)

Parser (adminparser/*)
listar_archivos_carpeta, leer_archivo_raw, leer_archivo_bytes, parsear_catalogo_visual, parsear_catalogo_csv, parsear_excel, analizar_ticket_llm, analizar_ticket_con_ia, analizar_muestras_carpeta, parsear_con_mapeo, parsear_carpeta, parsear_carpeta_stream, get_db_path, vincular_inventario, guardar_vinculacion, descargar_modelos

Tickets y cortes (admintickets/tickets.rs)
get_tickets, get_cortes, guardar_ticket_parseado, get_predictions (operativo Holt-Winters)

Empleados (adminempleados/*)
get_empleados, get_empleado_ventas, get_resumen_empleados, get_cortes_empleado, editar_empleado, set_estado_empleado, get_employee_goals, save_employee_goal, save_custom_goal, delete_employee_goal, check_employee_goals

Finanzas (adminfinanzas/*)
Gastos: get_gastos_recurrentes, crear_gasto, actualizar_gasto, eliminar_gasto, registrar_pago_gasto, get_pagos_gasto, get_proximos_vencimientos, actualizar_estados_gastos; Cortes: get_cortes_caja, get_corte_detalle, crear_corte_x, crear_corte_z, cerrar_corte, agregar_movimiento_caja, get_movimientos_corte, get_cortes_por_cajero_fecha; Metricas, graficas (incluye get_predicciones_financieras operativo), alertas y export (stubs exportar_balance_pdf, exportar_gastos_csv).

Chat (admintarvis/chat.rs)
send_chat_message, send_chat_stream, get_cloud_models, get_model_status, load_chat_model, unload_chat_model, stop_chat_stream, set_local_model_path

Empleado operativo (backempleado/*)
completar_venta, get_next_ticket_number, get_tienda_info, get_employee_profile, get_mi_turno, get_asistencia_empleado, get_mis_horas_extra, get_horas_extra_empleado

API keys (api_config.rs)
guardar_api_keys, leer_api_keys

---

## 10. Casos pendientes (stubs)

| Comando | Funcion | Estado |
|---|---|---|
| buscar_producto_similar | Busqueda semantica en inventario / nueva venta | STUB — embeddings propios pendientes |
| backfill_embeddings | Generar embeddings de la base | STUB — embeddings propios pendientes |
| exportar_balance_pdf | Exportar balance a PDF | STUB |
| exportar_gastos_csv | Exportar gastos a CSV | STUB |

Planes: embeddings con modelo propio y fine-tuning de Qwen2.5-Coder 1.5B Instruct para tools/SQL. Predicciones ya operativas via Holt-Winters. Impresion termica ESC/POS y facturacion electronica: pendientes.

---

## 11. Problemas Resueltos

- Migracion Python -> Rust completa: sidecar eliminado, binario unico.
- Dinero en centavos: migracion 0005 y modulo dinero.rs eliminan errores de redondeo.
- Parseo robusto: bugfixes A1/A3/A4/Bug8 conservados en el port.
- Seguridad: SQL parametrizado, roles por tool, API keys 0600, CSP activa, sin fallback plaintext en passwords, atomicidad en ventas/importaciones/pagos.
- Modularizacion frontend: empleados.tsx, nueva_venta.tsx y perfil.tsx subdivididos bajo 650 lineas.
- Predicciones locales sin red: Holt-Winters con banda 95% reemplaza a Prophet.
