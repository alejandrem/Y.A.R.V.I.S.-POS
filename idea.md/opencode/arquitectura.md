# Arquitectura del Proyecto Y.A.R.V.I.S. POS

Esta documentacion refleja la estructura actual y verificada de todo el sistema. Ya no existe sidecar de Python: la app es un binario unico de Tauri v2 (frontend React + backend Rust) que incluye el motor de IA como crate local (src-ia) y el modulo de predicciones Holt-Winters.

> Este doc fue actualizado tras la migracion Python -> Rust, las predicciones locales y la estabilización del parseador 2026-09 (folio 10/10, clave de idempotencia, orden cronológico). Para el historico ver migracion-rust.md.

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
│   ├── examples/                      # dbg (diagnóstico vs carpetas reales), bench1000 (prueba de carga).
│   ├── predicciones/                  # Holt-Winters + capa de datos de ventas.
│   │   ├── holt_winters.rs            #   Suavizado triple aditivo, grid 343 combos, banda 95%.
│   │   ├── ventas.rs                  #   Lectura de SQLite (ventas completadas), serie densa, predecir_ventas.
│   │   └── mod.rs                     #   Re-exporta predecir / predecir_ventas.
│   ├── parseador_de_tickets/          # Parseo de tickets/catalogos en Rust.
│   │   ├── lib.rs                     # Entry: declara cerebro, formatos, rutas, motor_chat, predicciones.
│   │   ├── cerebro/                   # Nucleo de regex/parseo sin modelo.
│   │   │   ├── analizador_tickets/    #   parser, encabezado, fechas, pagos, totales, esquema.
│   │   │   │   ├── detector/          #     Detección estadística SIN LLM: mod (orquesta),
│   │   │   │   │                     #     hipotesis (A/B), familia_c, muestra, diagnostico.
│   │   │   │   └── segmentador/       #     Un archivo → N tickets: mod, marcadores
│   │   │   │                         #     (folio 10/10 formatos), clave (idempotencia
│   │   │   │                         #     folio/AUTO/SIN-FOLIO + orden cronológico).
│   │   │   ├── filtrador/             #   Filtro de lineas utiles (3 niveles).
│   │   │   ├── parseador_masivo/      #   archivos, procesador/ (stream + carpeta),
│   │   │   │                         #   items, resumen, almacen, tests.rs.
│   │   │   └── vinculador_inventario/ #   Vinculacion: inventario, similitud (TF-IDF+fuzzy), vinculo, persistencia.
│   │   ├── formatos/                  # Lectores: lector_csv, lector_excel (calamine),
│   │   │                             # lector_txt/ (patrones, linea, visual).
│   │   └── rutas/                     # Resolucion de modelos + generacion local. SOLO CHAT:
│   │       ├── analizador_json.rs     #   extraer_json (generico, sin uso en parseo).
│   │       ├── analizador_modelos.rs  #   descargar/cargar/verificar modelos GGUF.
│   │       ├── analizador_inferencia.rs # generar_bajo_lock (llama.cpp) — chat.
│   │       └── rutas_modelos_api|config|detect.rs # API + config + deteccion (LM Studio).
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
    │   ├── hooks/                     # ThemeContext/useTheme (el progreso del lote vive en BatchProgressProvider, parseador/tickets/).
    │   ├── front-admin/               # Modulos del Administrador.
    │   │   ├── AdminDashboard.tsx     # Sidebar + montaje condicional (providers persistentes arriba del switch).
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
    │   │       └── parseador/         #   parseador.tsx + tickets/ (BatchProgressProvider, historial, progreso) y cortes/
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
            ├── lib.rs                 # Builder Tauri: setup DB, registra 98 comandos, plugins.
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
                │   ├── admininventory/#   inventory/ (catalogo, crud, importar, semantica, historial)
                │   ├── adminparser/   #   parser_commands.rs, parser_csv.rs, parser_excel.rs, parser_txt/ (archivos, catalogo, deteccion, lote), utils.rs
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
|  |                  | <------ |   (98 #[tauri::command])         |  |
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
- Rust <-> IA: crate local src-ia (mismo proceso). Chat cloud (OpenCode Zen / Gemini via reqwest + SSE) con fallback a local (Qwen GGUF con llama-cpp-4). Parseo de tickets 100% reglas + estadística (detector/), sin LLM. Predicciones usan Holt-Winters puro sin red.

## Comandos registrados

98 comandos en lib.rs. Ver tecnologias.md para el conteo por dominio.

## Casos pendientes (stubs) — devuelven error claro, no rompen la caja

| Comando / funcion | Donde se llama | Estado |
|---|---|---|
| exportar_balance_pdf / exportar_gastos_csv | finanzas export | STUB — export pendiente |

Nota: buscar_producto_similar y backfill_embeddings YA están implementados (HashEmbedder propio 384d por trigramas, sin red neuronal); get_predictions y get_predicciones_financieras responden con fecha/prediccion/minimo/maximo. La importación masiva de 1000 tickets tarda ~250ms en release con 0 errores (example bench1000).
