# Arquitectura del Proyecto Y.A.R.V.I.S. POS

Esta documentación refleja la **estructura actual y verificada** de todo el sistema. Ya NO existe sidecar de Python: la app es un **binario único de Tauri v2** (frontend React + backend Rust) que incluye el motor de IA como crate local (`src-ia`).

> ⚠️ Este doc fue actualizado tras la migración Python → Rust. Para el histórico de esa migración ver `../imple & docu/migracion_rust.md`.

## Estructura de Archivos y Directorios

```text
Y.A.R.V.I.S.-POS/
├── run.sh                             # Lanzador Linux: verifica npm + cargo y corre `npm run tauri dev`
├── run.bat                            # Lanzador Windows: idéntico en batch
├── reset.sh                           # Limpieza: borra yarvis.db y caches en $HOME/.local/share/com.yarvis.pos
├── plan_YARVIS_V2_HIBRIDO.md         # (PLAN HISTÓRICO de la era Python — SUPERSEDIDO)
│
├── idea.md/                           # Documentación de planificación y diseño del sistema.
│   ├── opencode/                      # Stack, arquitectura, visión y comandos de dev.
│   └── imple & docu/                 # Implementación, interconexión, parseador, bugs, migración.
│
├── src-ia/                            # CRATE RUST independiente: núcleo de IA (motor Rust).
│   ├── Cargo.toml                     # package "src-ia" v0.1.0; feature "llm-local" (llama-cpp-4).
│   ├── parseador_de_tickets/          # Parseo de tickets/catálogos en Rust.
│   │   ├── lib.rs                     # Entry: declara cerebro, formatos, rutas, motor_chat.
│   │   ├── cerebro/                   # Núcleo de regex/parseo sín modelo (espejo de Python).
│   │   │   ├── analizador_tickets/    #   parser, encabezado, fechas, pagos, segmentador, totales, esquema.
│   │   │   ├── filtrador/             #   Filtro de líneas útiles.
│   │   │   ├── parseador_masivo/      #   Orquestador: archivos, procesador, items, resumen, almacen.
│   │   │   └── vinculador_inventario/ #   Vinculación: inventario, similitud, vinculo, persistencia.
│   │   ├── formatos/                  # Lectores: lector_csv, lector_excel (calamine), lector_txt.
│   │   └── rutas/                     # Resolución de modelos + análisis LLM:
│   │       ├── analizador_ticket.rs   #   analizar_ticket (LLM local).
│   │       ├── analizador_prompt.rs   #   SISTEMA_PROMPT.
│   │       ├── analizador_json.rs     #   extraer_json.
│   │       ├── analizador_modelos.rs  #   descargar/cargar/verificar modelos GGUF.
│   │       ├── analizador_inferencia.rs # generar_bajo_lock (llama.cpp).
│   │       └── rutas_modelos_*.rs     #   API + config + detección (LM Studio).
│   ├── motor-chat/
│   │   ├── mod.rs                     # pub mod cloud; pub mod llm.
│   │   ├── cloud/                     # Chat por API (nube).
│   │   │   ├── apis_cloud/            #   proveedores, generacion, catalogo, sse, tipos, helpers, errores.
│   │   │   ├── prompts.rs             #   construir_mensajes_api.
│   │   │   ├── think.rs               #   SeparadorThink (bloques 思考).
│   │   │   └── variables.rs           #   API keys (archivo plano).
│   │   └── llm/mod.rs                 # Chat LOCAL Qwen 1.7B vía llama-cpp-4 (feature llm-local).
│   └── tests/                         # estres.rs, test_chat_1_7_real.rs, verificar_conexion.rs, verificar_modelos.rs
│
└── yarvis-app/                        # Aplicación de Escritorio (Frontend React + Backend Rust).
    ├── package.json                   # React, Vite, Tailwind, Tauri CLI, morphicons, recharts, react-markdown.
    ├── vite.config.ts                 # Puerto 1420, plugin React, HMR para Tauri.
    ├── tailwind.config.js             # Estilos (darkMode class).
    ├── src/                           # FRONTEND: React + TypeScript.
    │   ├── main.tsx                   # React root: StrictMode + ThemeProvider + App.
    │   ├── App.tsx                    # Orquestador: setup (paso 0) → login (1) → AdminDashboard (2)/EmployeeDashboard (3).
    │   ├── hooks/                     # ParserContext (estado global del parseo) + ThemeContext/useTheme.
    │   ├── front-admin/               # Módulos del Administrador.
    │   │   ├── AdminDashboard.tsx     # Sidebar y enrutador del Admin (menú: ventas, inventario, tickets,
    │   │   │                          #   finanzas, clientes, empleados, configuración, yarvis).
    │   │   ├── PrimerInicio.tsx       # Asistente de configuración inicial (admin + tienda + empleado).
    │   │   ├── types.ts               # Tipos TypeScript compartidos.
    │   │   └── ventanas/
    │   │       ├── adminclientes/clientes.tsx
    │   │       ├── adminconfig/       #   configuracion.tsx (+ components/ y hooks/ refactorizados)
    │   │       │   ├── components/    #     ConfigHeader, IdentityForm, SecurityForm, AppearanceForm,
    │   │       │   │                  #     importmodule/ (ImportModule, ImportActions, ImportHeader,
    │   │       │   │                  #       ImportStatus, LlmAnalysisCard, PreviewTable, RawDataViewer)
    │   │       │   └── hooks/         #     useAdminData, useParserActions
    │   │       ├── adminempleados/    #   empleados.tsx, modalEmpleados.tsx, modalMetas.tsx, modalTurnos.tsx
    │   │       ├── adminfinanzas/     #   finanzas.tsx, FinanzasDashboard, AlertasPanel, CortesManager,
    │   │       │                      #   GastosManager, GraficasPanel, hooks.ts, types.ts, utils.ts
    │   │       ├── admininventario/inventario.tsx   # CRUD + importación + búsqueda semántica.
    │   │       ├── adminticket/       #   tickets.tsx + graficas.tsx (llama a get_predictions — stub).
    │   │       ├── adminventas/ventas.tsx
    │   │       ├── adminyarvis/       #   yarvis.tsx + ChatWidget.tsx
    │   │       └── parseadodetickets/ #   BatchProcessor.tsx, CatalogosParseados.tsx, ColumnMapper.tsx
    │   │                              #   (reutilizados por adminconfig/components/importmodule)
    │   └── front-empleado/            # Módulos del Empleado (Punto de Venta).
    │       ├── EmployeeDashboard.tsx  # Dashboard de caja (nueva venta, inventario, tickets, clientes,
    │       │                          #   perfil, yarvis, ajustes).
    │       └── ventanas/
    │           ├── emplea_new_venta/  #   nueva_venta.tsx (+ modalventa.tsx, modalticket.tsx)
    │           ├── empleaajustes/ajustes.tsx
    │           ├── empleaclientes/clientes.tsx
    │           ├── empleainventario/inventario.tsx
    │           ├── empleaperfil/perfil.tsx
    │           ├── empleaticket/ticket.tsx
    │           └── empleayarvis/yarvis.tsx
    │
    └── src-tauri/                     # BACKEND RUST + configuración Tauri.
        ├── tauri.conf.json            # identifier com.yarvis.pos, devUrl 1420. SIN externalBin (no sidecar).
        ├── capabilities/default.json  # Permisos: core, opener, dialog.
        ├── Cargo.toml                 # tauri 2.11, sqlx 0.8 (sqlite), tokio, serde, reqwest, argon2,
        │                              #   chrono, rand, sha2, futures; src-ia = path "../../src-ia" (feature llm-local).
        └── src/
            ├── main.rs                # Entry (windows_subsystem) → yarvis_app_lib::run().
            ├── lib.rs                 # Builder Tauri: setup DB, registra ~91 comandos, plugins.
            ├── models.rs              # Structs serde compartidas (AdminData, InventoryItem, VentaRequest, ...).
            └── backventanas/
                ├── mod.rs
                ├── db/db.rs           # initialize_db: pool SQLite (WAL) + ruta de yarvis.db.
                ├── backadmin/         # Comandos exclusivos del administrador.
                │   ├── adminclientes/ #   clientes.rs
                │   ├── adminconfig/   #   auth.rs (setup/login admin+empleado, Argon2), google.rs (OAuth).
                │   ├── adminempleados/#   empleados.rs, modalempleado.rs, modalmetas.rs, modalturnos.rs
                │   ├── adminfinanzas/ #   alertas.rs, cortes.rs, export.rs, finanzas.rs, gastos.rs,
                │   │                  #   graficas.rs, metricas.rs, models.rs
                │   ├── admininventory/#   inventory.rs (CRUD + importar_catalogo + stubs embeddings)
                │   ├── adminparser/   #   parser_commands.rs, parser_csv.rs, parser_excel.rs,
                │   │                  #   parser_txt.rs, utils.rs
                │   ├── admintarvis/   #   chat.rs (chat cloud/local + fallback)
                │   ├── admintickets/  #   tickets.rs (+ get_predictions — stub)
                │   └── adminventas/   #   ventas.rs
                └── backempleado/      # Comandos del empleado (venta nueva, perfil).
                    ├── emplea_new_venta/new_venta.rs   # completar_venta, get_next_ticket_number, ...
                    ├── empleaperfil/perfil.rs
                    └── ...            # El resto de ventanas de empleado reutilizan comandos admin
                                       # registrados globalmente (mismo invoke).
```

---

## Diagrama de Comunicación General

```text
┌─────────────────────────────────────────────────────────────────────┐
│                        yarvis-app.exe (único)                       │
│                                                                     │
│  ┌──────────────────┐         ┌──────────────────────────────────┐  │
│  │   Frontend       │ invoke  │   Backend Rust (Tauri)           │  │
│  │  (React + Vite)  │ ─────►  │   src-tauri/src/backventanas     │  │
│  │                  │ ◄─────  │   (~91 #[tauri::command])        │  │
│  └──────────────────┘  IPC    └───────┬───────────────┬──────────┘  │
│                                       │               │             │
│                          ┌────────────▼────┐   ┌──────▼───────────┐ │
│                          │  SQLite (WAL)   │   │  Motor IA Rust   │ │
│                          │  yarvis.db      │   │  (crate src-ia)  │ │
│                          │  sqlx (pool)    │   │  EN PROCESO      │ │
│                          └─────────────────┘   └──────┬───────────┘ │
│                                                       │             │
│                    ┌──────────────────────────────────┼───────────┐ │
│                    │ local: llama.cpp (Qwen 1.7B GGUF)│           │ │
│                    │ cloud: HTTP + SSE (OpenCode/Gemini) ◄──► Internet
└─────────────────────┴────────────────────────────────────────────────┘
```

- **Frontend ↔ Rust**: IPC nativo de Tauri (`invoke()`). Sin HTTP local, sin puertos.
- **Rust ↔ SQLite**: `sqlx` en modo asíncrono (pool), **WAL activado** para lecturas/escrituras concurrentes.
- **Rust ↔ IA**: El motor de IA es el crate local `src-ia` (mismo proceso). Chat **cloud** (OpenCode Zen / Gemini vía `reqwest` + SSE) con **fallback a local** (Qwen 1.7B con `llama-cpp-4`). El parseo de tickets usa el parseador de reglas (`cerebro/`) y LLM local bajo demanda.

## Casos pendientes (stubs) — actualmente devuelven error claro

| Comando / función | Dónde se llama | Estado |
|---|---|---|
| `buscar_producto_similar` | inventario + nueva venta (búsqueda semántica) | STUB (embeddings/RAG nativos pendientes en Rust) |
| `backfill_embeddings` | config / importación | STUB |
| `get_predictions` | gráficas de tickets del admin | STUB (pronósticos pendientes) |
| `get_predicciones_financieras` | GraficasPanel de finanzas | STUB |