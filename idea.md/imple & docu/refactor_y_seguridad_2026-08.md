# Auditoría, refactors y endurecimiento — Sesión 2026-08-24

> Registro maestro de la sesión de auditoría nivel senior + fixes. Complementa
> a `Bugs resueltos uwu.md` (bugs puntuales) con la vista de arquitectura:
> qué se cambió, por qué y qué queda pendiente.

---

## 1. Diagnóstico general de la auditoría

Auditoría completa de backend Rust, frontend React y crate src-ia. Veredicto:
base sólida para un proyecto personal (migraciones versionadas, Argon2id,
transacción en el flujo crítico de cobro, ~210 tests reales, cero secrets en
git). Los problemas graves encontrados caían en 4 familias:

1. **Operaciones multi-paso no atómicas** + errores tragados en silencio.
2. **Dinero en f64** en todo el esquema y código.
3. **Frontend sin capa de servicio** + duplicación masiva admin↔empleado.
4. **Seguridad**: SQL interpolado, roles solo por prompt, secrets en localStorage.

---

## 2. Fixes de atomicidad y silencio (fase 1)

| Fix | Archivo | Qué cambió |
|---|---|---|
| Stock negativo imposible | `new_venta.rs` | `UPDATE ... WHERE stock >= ?` + chequeo de `rows_affected`; la venta completa revierte si un item no alcanza |
| Importación de tickets atómica | `admintickets/tickets.rs` | Transacción todo-o-nada; los `let _ =` eliminados; items sin coincidencia de nombre SE REPORTAN al usuario ("3/5 items vinculados") |
| Importación de catálogos atómica | `admininventory/inventory.rs` | Una tx para todo el lote; el registro del hash va dentro de la misma tx; errores de INSERT ya no se tragan |
| Pago de gasto atómico | `adminfinanzas/gastos.rs` | INSERT del pago + UPDATE del acumulado en una sola tx |
| Helpers testeables | `tickets.rs`, `gastos.rs` | Extracción `guardar_ticket_parseado_impl` / patrón `_impl` consistente |

Tests nuevos: stock insuficiente, rollback multi-item, vinculación
case-insensitive, reporte de items sin vincular, atomicidad ante item corrupto.

## 3. Migración f64 → centavos (fase 2)

**REGLA DE ORO nueva:** toda columna monetaria es `INTEGER` en **centavos**
en SQLite. Las cantidades (stock, cantidad) y porcentajes siguen en REAL.

- **Migración `0005_moneda_centavos.sql`**: reconstruye las 12 tablas con
  columnas INTEGER, convirtiendo datos históricos ×100 con redondeo al centavo.
- **Módulo `src/dinero.rs`**: `a_centavos()` / `a_pesos()` — TODA conversión
  pasa por ahí. El contrato IPC sigue hablando pesos (f64) → frontend intacto.
- **Backend completo migrado**: ventas, tickets, inventario, cortes (cierre con
  aritmética exacta en enteros), gastos, métricas (incluida la caché materializada),
  gráficas, alertas, empleados, metas, auth, perfil.
- **src-ia migrado**: las tools devuelven pesos al LLM; Holt-Winters recibe la
  serie en pesos; el parser masivo escribe centavos.

### ⚠️ Lección dura: sqlx-sqlite y PRAGMA foreign_keys

sqlx-sqlite envuelve SIEMPRE cada migración en una transacción e IGNORA el
marcador `-- no-transaction`. Dentro de una transacción SQLite ignora
`PRAGMA foreign_keys`, así que los DROP TABLE de la reconstrucción fallaban
con datos reales (FKs activas) pero pasaban con DBs vacías de test.

**Solución definitiva en `db.rs`** (y en el fixture de tests): conexión en dos
fases — Fase 1 migra con `foreign_keys(false)` desde las opciones de conexión,
Fase 2 reabre con `foreign_keys(true)` para operación normal.

**Lección meta:** las rutas de migración deben probarse con DATOS REALES;
los fixtures frescos no cubren ese camino. Herramienta de diagnóstico:
`cargo run --example diag_migracion -- <ruta-db-copia>`.

## 4. Chat cloud (fase 3)

| Problema | Causa raíz | Fix |
|---|---|---|
| Gemini nunca funcionaba (404) | Google retiró gemini-2.x **para keys nuevas** | default → `gemini-3.6-flash` (`variables.rs` + `ChatWidget.tsx`) |
| Streams largos morían a los 120 s | `Client::timeout` aplica al ciclo completo petición+cuerpo | `read_timeout(90s)`: timeout de INACTIVIDAD entre chunks; un stream vivo nunca se corta |
| max_tokens rompía Gemini | 39800 > techo de salida flash | `MAX_TOKENS_GOOGLE = 8192` separado |

## 5. Seguridad (fase 4)

1. **SQL de tools parametrizado** (`tools/mod.rs`): cero interpolación; helper
   `escape_like()` + `ESCAPE '\'` para que `%`/`_` del input del LLM no alteren
   patrones. `LIMIT` también parametrizado.
2. **API keys fuera de localStorage**: nuevo módulo `api_config.rs` con comandos
   `leer_api_keys` / `guardar_api_keys`; JSON en `app_data_dir/api_keys.json`
   con permisos 0600. El frontend usa caché EN MEMORIA alimentada por el backend
   (`ChatWidget.tsx`: `refrescarApiKeysCache` / `setApiKeysCache`). Escalamiento
   futuro: OS keychain vía crate `keyring` sin cambiar contratos.
3. **CSP activa** (`tauri.conf.json`): antes `"csp": null` renderizando output
   de LLMs; ahora política restrictiva con nonces automáticos de Tauri v2.
4. **Roles enforced en ejecución de tools** (`chat.rs`): `TOOLS_SOLO_ADMIN =
   [query_sales, compare_periods, get_restock_analysis]`; guard en el punto de
   ejecución hilado por las 5 rutas (cloud msg/stream, local msg/stream). El
   prompt es sugerencia; esto es control de acceso real.
5. **Sin fallback plaintext en `verify_password`** (`auth.rs`): un hash que no
   parsea como Argon2id deniega acceso siempre. Test de regresión incluido.

## 6. Tools de navegación de inventario para cloud (fase 5)

3 tools nuevas de SOLO LECTURA en el ejecutor compartido (`tools/mod.rs`):

| Tool | Args | Qué hace |
|---|---|---|
| `search_products` | `query` (obligatorio), `limit` | Búsqueda parcial por nombre, ordenada por vendido, precio en pesos |
| `list_categories` | — | Categorías con conteo de productos y stock total |
| `get_products_by_category` | `category` opcional, `limit` | Hojear categoría (case-insensitive) o catálogo completo |

- Documentadas en el prompt cloud (`prompts.rs`, const `TOOLS_EXTRAS`) con
  estrategia recomendada: list → browse → search. TOOLS_LINEA y
  TOOLS_INSTRUCCIONES (formato del fine-tuning) quedaron INTACTOS — verificado
  por test `prompts_cloud_documentan_tools_de_navegacion`.
- Decisión de diseño: protocolo textual `<tool_call>` compartido por local y
  cloud (un ejecutor, un formato), NO function calling nativo por proveedor.
  Revisar si se agregan tools con argumentos complejos.
- Escritura desde tools: descartada deliberadamente (riesgo).

## 7. Empaquetado

`yarvis-app/build.sh` reemplaza a `npm run tauri build` directo. Exporta
`LD_LIBRARY_PATH` (para que linuxdeploy empaquete `libllama.so.0` dentro del
AppImage) y `NO_STRIP=1` (el strip viejo de linuxdeploy no entiende `.relr.dyn`
de Arch). Requiere `fuse2` y `/usr/lib/gdk-pixbuf-2.0/2.10.0/loaders` (ver
Bug B1/B1b en la bitácora).

## 8. Pendiente (backlog priorizado)

1. Unwraps sobre input del front (`metricas.rs`, `alertas.rs`) — crashes por fecha malformada.
2. Capa de servicios en frontend + eliminar fork empleado/admin del chat (~370 líneas duplicadas).
3. Errores visibles al usuario (41 `console.error` + 20 `alert()`).
4. `cerrar_corte` sin cablear en la UI (backend ya recalcula server-side).
5. OS keychain para API keys · rate limiting de login · TOCTOU en setup.
6. Monolitos frontend: empleados.tsx (824 líneas), nueva_venta.tsx (525), perfil.tsx (495).
7. Tests: permisos por rol e2e, race de stock mismo producto, red/SSE.
8. Contexto local sin truncamiento (>4096 tokens falla duro) y recorte a 20
   líneas en ruta LLM del parser.
