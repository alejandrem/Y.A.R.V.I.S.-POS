# Arquitectura del Proyecto Y.A.R.V.I.S. POS

Esta documentacion refleja la estructura actual y verificada de todo el sistema. Ya no existe sidecar de Python: la app es un binario unico de Tauri v2 (frontend React + backend Rust) que incluye el motor de IA como crate local (src-ia) y el modulo de predicciones Holt-Winters.

> Este doc fue actualizado tras la migracion Python -> Rust y la implementacion de predicciones locales. Para el historico ver migracion-rust.md.

## Estructura de Archivos y Directorios

```text
Y.A.R.V.I.S.-POS/
├── run.sh                             # Lanzador Linux: verifica npm + cargo y corre `npm run tauri dev` en yarvis-app/
├── run.bat                            # Lanzador Windows: identico en batch
├── reset.sh                           # Limpieza: borra yarvis.db y caches en $HOME/.local/share/com.yarvis.pos
│
├── doc/                               # Documentacion (nombre futuro; hoy idea.md/)
│   ├── opencode/                      # Stack, arquitectura, vision y comandos de dev.
│   └── implementacion/                # Implementacion, interconexion, parseador, bugs, migracion.
│
├── src-ia/                            # CRATE RUST independiente: nucleo de IA.
│   ├── Cargo.toml                     # package "src-ia" v0.1.0; feature "llm-local" (llama-cpp-4 0.5).
│   ├── predicciones/                  # Holt-Winters + capa de datos de ventas.
│   │   ├── holt_winters.rs            #   Suavizado triple aditivo, grid 343 combos, banda 95%.
│   │   ├── ventas.rs                  #   Lectura de SQLite (ventas completadas), serie densa, predecir_ventas.
│   │   └── mod.rs                     #   Re-exporta predecir / predecir_ventas.
│   ├── parseador_de_tickets/          # Parseo de tickets/catalogos en Rust.
│   │   ├── lib.rs                     # Entry: declara cerebro, formatos, rutas, motor_chat, predicciones.
│   │   ├── cerebro/                   # Nucleo de regex/parseo sin modelo.
│   │   │   ├── analizador_tickets/    #   parser, encabezado, fechas, pagos, segmentador, totales, esquema.
│   │   │   ├── filtrador/             #   Filtro de lineas utiles (3 niveles).
│   │   │   ├── parseador_masivo/      #   Orquestador: archivos, procesador, items, resumen, almacen.
│   │   │   └── vinculador_inventario/ #   Vinculacion: inventario, similitud (TF-IDF+fuzzy), vinculo, persistencia.
│   │   ├── formatos/                  # Lectores: lector_csv, lector_excel (calamine), lector_txt.
│   │   └── rutas/                     # Resolucion de modelos + analisis LLM:
│   │       ├── analizador_ticket.rs   #   analizar_ticket (LLM local 1.5B Coder Instruct fine-tuneado).
│   │       ├── analizador_prompt.rs   #   SISTEMA_PROMPT.
│   │       ├── analizador_json.rs     #   extraer_json.
│   │       ├── analizador_modelos.rs  #   descargar/cargar/verificar modelos GGUF.
│   │       ├── analizador_inferencia.rs # generar_bajo_lock (llama.cpp).
│   │       └── rutas_modelos_*.rs     #   API + config + deteccion (LM Studio).
│   ├── motor-chat/
│   │   ├── mod.rs                     # pub mod cloud; pub mod llm.
│   │   ├── cloud/                     # Chat por API (nube).
│   │   │   ├── apis_cloud/            #   proveedores, generacion, catalogo, sse, tipos, helpers, errores.
│   │   │   ├── prompts.rs             #   construir_mensajes_api (TOOLS_LINEA fine-tuneada + TOOLS_EXTRAS).
│   │   │   ├── think.rs               #   SeparadorThink (bloques think/response).
│   │   │   └── variables.rs           #   API keys (archivo plano 0600 via backend).
│   │   └── llm/
│   │       ├── mod.rs                 # Chat LOCAL Qwen2.5-Coder 1.5B Instruct via llama-cpp-4 (feature llm-local) + recortar_historial.
│   │       └── tools/                 # Ejecutor de 10 tools (ventas.rs, inventario.rs, deteccion, helpers).
│   └── tests/                         # estres.rs, fuzzing, masivo, verificar_modelos, etc.
│
└── yarvis-app/                        # Aplicacion de Escritorio (Frontend React + Backend Rust).
    ├── package.json                   # React 19.1, Vite 7, Tailwind 3.4, morphicons 1.7, recharts 3.10.
    ├── vite.config.ts                 # Puerto 1420, plugin React, HMR para Tauri.
    ├── tailwind.config.js             # Estilos (darkMode class).
    ├── build.sh                       # Build de produccion: NO_STRIP + LD_LIBRARY_PATH para libllama.
    ├── src/                           # FRONTEND: React + TypeScript 5.8.
    │   ├── main.tsx                   # React root: StrictMode + ThemeProvider + App.
    │   ├── App.tsx                    # Orquestador: setup (paso 0) -> login (1) -> AdminDashboard (2)/EmployeeDashboard (3).
    │   ├── hooks/                     # ParserContext + ThemeContext/useTheme.
    │   ├── front-admin/               # Modulos del Administrador.
    │   │   ├── AdminDashboard.tsx     # Sidebar y enrutador del Admin.
    │   │   ├── PrimerInicio.tsx       # Asistente de configuracion inicial (admin + tienda + empleado).
    │   │   ├── types.ts               # Tipos TypeScript compartidos.
    │   │   └── ventanas/
    │   │       ├── adminclientes/clientes.tsx
    │   │       ├── adminconfig/       #   configuracion.tsx + components/ y hooks/
    │   │       │   ├── components/    #     ConfigHeader, IdentityForm, SecurityForm, AppearanceForm,
    │   │       │   │                  #     importmodule/ (ImportModule, ImportActions, ImportHeader, etc.)
    │   │       │   └── hooks/         #     useAdminData, useParserActions
    │   │       ├── adminempleados/    #   empleados.tsx, modalEmpleados.tsx, modalMetas.tsx, modalTurnos.tsx
    │   │       ├── adminfinanzas/     #   finanzas.tsx, FinanzasDashboard, AlertasPanel, CortesManager, etc.
    │   │       ├── admininventario/inventario.tsx
    │   │       ├── adminticket/       #   tickets.tsx + graficas.tsx (usan get_predictions ya operativo)
    │   │       ├── adminventas/ventas.tsx
    │   │       └── parseadodetickets/ #   BatchProcessor, ColumnMapper, CatalogosParseados
    │   └── front-empleado/            # Modulos del Empleado (Punto de Venta).
    │       ├── EmployeeDashboard.tsx
    │       └── ventanas/
    │           ├── emplea_new_venta/  #   nueva_venta.tsx (+ modalventa, modalticket)
    │           ├── empleaajustes/ajustes.tsx
    │           ├── empleaclientes/clientes.tsx
    │           ├── empleainventario/inventario.tsx
    │           ├── empleaperfil/perfil.tsx
    │           ├── empleaticket/ticket.tsx
    │           └── empleayarvis/yarvis.tsx
    │
    └── src-tauri/                     # BACKEND RUST + configuracion Tauri.
        ├── tauri.conf.json            # identifier com.yarvis.pos, devUrl 1420, CSP activa, bundle targets all.
        ├── capabilities/default.json  # Permisos: core, opener, dialog.
        ├── Cargo.toml                 # tauri 2.11, sqlx 0.8, tokio 1.38, serde, reqwest, argon2, chrono, src-ia.
        └── src/
            ├── main.rs                # Entry (windows_subsystem) -> yarvis_app_lib::run().
            ├── lib.rs                 # Builder Tauri: setup DB, registra 97 comandos, plugins.
            ├── models.rs              # Structs serde compartidas.
            ├── dinero.rs              # a_centavos / a_pesos (conversion centavos).
            ├── api_config.rs          # guardar_api_keys / leer_api_keys (archivo 0600).
            └── backventanas/
                ├── mod.rs
                ├── db/db.rs           # initialize_db: pool SQLite (WAL) + ruta de yarvis.db.
                ├── backadmin/         # Comandos exclusivos del administrador.
                │   ├── adminconfig/   #   auth.rs, google.rs (OAuth PKCE)
                │   ├── adminempleados/#   empleados.rs, modalempleado.rs, modalmetas.rs
                │   ├── adminfinanzas/ #   alertas, cortes, export (stubs), finanzas, gastos, graficas, metricas
                │   ├── admininventory/#   inventory.rs (CRUD + importar_catalogo + stubs embeddings)
                │   ├── adminparser/   #   parser_commands.rs, parser_csv.rs, parser_excel.rs, etc.
                │   ├── admintarvis/   #   chat.rs + ciclo_tools, cancelacion, herramientas_rol, rutas
                │   └── admintickets/  #   tickets.rs (get_predictions operativo)
                └── backempleado/      # Comandos del empleado (venta nueva, perfil, asistencia).
                    ├── emplea_new_venta/new_venta.rs
                    └── empleaperfil/{perfil.rs, asistencia.rs}
```

---

## Diagrama de Comunicacion General

```text
+---------------------------------------------------------------------+
|                        yarvis-app (binario unico)                   |
|                                                                     |
|  +------------------+         +----------------------------------+  |
|  |   Frontend       | invoke  |   Backend Rust (Tauri)           |  |
|  |  (React + Vite)  | ------> |   src-tauri/src/backventanas     |  |
|  |                  | <------ |   (97 #[tauri::command])         |  |
|  +------------------+  IPC    +-------+--------------+-----------+  |
|                                   |              |             |
|                      +------------v----+  +------v-----------+ |
|                      |  SQLite (WAL)   |  |  Motor IA Rust   | |
|                      |  yarvis.db      |  |  (crate src-ia)  | |
|                      |  sqlx (pool)    |  |  EN PROCESO      | |
|                      +-----------------+  +------+-----------+ |
|                                                   |             |
|                    +-------------------------------+-----------+ |
|                    | local: llama.cpp (Qwen GGUF) |           | |
|                    | cloud: HTTP + SSE (Opencode/Gemini) <-> Internet |
|                    | predicciones: Holt-Winters (sin red)     | |
+---------------------------------------------------------------------+
```

- Frontend <-> Rust: IPC nativo de Tauri (invoke). Sin HTTP local, sin puertos.
- Rust <-> SQLite: sqlx en modo asincrono (pool), WAL activado, unico escritor.
- Rust <-> IA: crate local src-ia (mismo proceso). Chat cloud (OpenCode Zen / Gemini via reqwest + SSE) con fallback a local (Qwen GGUF con llama-cpp-4). Parseo de tickets usa reglas (cerebro/) y LLM local bajo demanda. Predicciones usan Holt-Winters puro sin red.

## Comandos registrados

97 comandos en lib.rs:38. Ver tecnologias.md para el conteo por dominio.

## Casos pendientes (stubs) — devuelven error claro, no rompen la caja

| Comando / funcion | Donde se llama | Estado |
|---|---|---|
| buscar_producto_similar | inventario + nueva venta (busqueda semantica) | STUB — embeddings propios pendientes; hoy usa TF-IDF+fuzzy solo en vinculador |
| backfill_embeddings | config / importacion | STUB — mismo motivo |
| exportar_balance_pdf / exportar_gastos_csv | finanzas export | STUB — export pendiente |

Nota: get_predictions y get_predicciones_financieras ya no son stubs: estan implementados via src-ia/predicciones y responden con fecha/prediccion/minimo/maximo.
