# Arquitectura del Proyecto Y.A.R.V.I.S. POS

## Estructura de Archivos

```
Y.A.R.V.I.S.-POS/
├── .gitignore                         # Archivos ignorados por Git
├── run.bat                            # Launcher: instala dependencias y ejecuta npm run tauri dev
│
├── README.md/                         # Documentación del proyecto
│   └── plan de implementacion/
│       ├── idea a vender.md           # Concepto del negocio y propuesta de valor
│       ├── tecnologias.md             # Stack tecnológico elegido y por qué
│       ├── arquitectura.md            # Este archivo: estructura del proyecto
│       ├── comandos.md                # Comandos útiles de desarrollo
│       ├── implementacion.md          # Plan de fases de desarrollo
│       ├── interconexion.md           # Cómo se comunican Rust ↔ Python
│       └── logica de proceso.md       # Lógica de negocio del POS
│
├── yarvis-app/                        # Aplicación de escritorio (Tauri)
│   │
│   ├── index.html                     # Entry point HTML que carga React
│   ├── package.json                   # Dependencias Node: React 19, Vite 7, Tauri API, Tailwind
│   ├── package-lock.json              # Lock de versiones exactas de npm
│   ├── vite.config.ts                 # Config de Vite: proxy a Tauri, puerto 1420
│   ├── tsconfig.json                  # Config TypeScript para el frontend
│   ├── tsconfig.node.json             # Config TypeScript para Vite/Node
│   ├── tailwind.config.js             # Config de Tailwind CSS
│   ├── postcss.config.js              # Plugin PostCSS para procesar Tailwind
│   ├── README.md                      # Instrucciones de desarrollo de yarvis-app
│   ├── .gitignore                     # Ignorar node_modules, dist, etc.
│   │
│   ├── public/                        # Archivos estáticos servidos por Vite
│   │   ├── vite.svg                   # Logo de Vite
│   │   └── tauri.svg                  # Logo de Tauri
│   │
│   ├── src/                           # Código fuente del frontend React
│   │   ├── main.tsx                   # Entry point: renderiza <App /> en StrictMode
│   │   ├── App.tsx                    # Componente raíz: estado global, routing de pantallas, login
│   │   ├── App.css                    # Estilos globales + directivas Tailwind
│   │   ├── vite-env.d.ts              # Tipado de Vite para TypeScript
│   │   │
│   │   ├── assets/                    # Imágenes y recursos estáticos
│   │   │   └── react.svg              # Logo de React
│   │   │
│   │   ├── front-admin/               # Panel de administración
│   │   │   ├── AdminDashboard.tsx     # Sidebar con 8 módulos + routing de contenido admin
│   │   │   ├── PrimerInicio.tsx       # Wizard de setup inicial: formulario admin + empleado
│   │   │   │
│   │   │   └── ventanas/              # Componentes funcionales del admin
│   │   │       ├── Inventario.tsx     # CRUD productos, filtros, alertas stock, conciliación
│   │   │       ├── Tickets.tsx        # Historial tickets/cortes, métricas, filtros por rango
│   │   │       └── Configuracion.tsx  # Datos tienda, contraseña, temas, parseador de tickets
│   │   │
│   │   └── front-empleado/            # Panel de empleado (POS)
│   │       └── EmployeeDashboard.tsx   # Interfaz de cobro: carrito, búsqueda IA, botón cobrar
│   │
│   └── src-tauri/                     # Backend Rust (Tauri v2)
│       │
│       ├── tauri.conf.json            # Config Tauri: ventana, permisos, plugins
│       ├── Cargo.toml                 # Dependencias Rust: tauri, sqlx, tokio, serde
│       ├── Cargo.lock                 # Lock de versiones exactas de Cargo
│       ├── build.rs                   # Script de build de Tauri
│       │
│       ├── capabilities/
│       │   └── default.json           # Permisos de la app (dialogos, archivos)
│       │
│       ├── icons/                     # Iconos para diferentes plataformas
│       ├── target/                    # Artefactos de compilación (yarvis-app.exe)
│       │
│       └── src/                       # Código fuente Rust
│           ├── main.rs                # Entry point: llama a lib::run()
│           ├── lib.rs                 # Setup Tauri: inicializa DB, registra 17 commands
│           ├── db.rs                  # SQLite WAL: crea 8 tablas (usuarios, productos, etc.)
│           ├── models.rs              # Structs: AdminData, InventoryItem, TicketItem, etc.
│           │
│           └── commands/              # Commands IPC (Rust ↔ Frontend)
│               ├── mod.rs             # Módulo raíz que expone todos los commands
│               ├── auth.rs            # 7 commands: login admin/empleado, setup, CRUD admin
│               ├── inventory.rs       # 5 commands: CRUD productos, importar catálogo
│               ├── tickets.rs         # 3 commands: tickets, cortes, guardar parseados
│               └── parser.rs          # 3 commands: leer archivo, parsear catálogo/ticket
│
└── yarvis-IA/                         # Motor de Inteligencia Artificial (Python)
    │
    ├── main.py                        # FastAPI: /health, /predict (Prophet), /chat (Qwen)
    ├── requirements.txt               # Dependencias: fastapi, prophet, pandas, llama-cpp
    │
    └── modelos/
        ├── profeta/
        │   ├── predictor.py           # Predicción de ventas con Meta Prophet + intervalos
        │   └── .gitkeep               # Placeholder
        │
        └── qwen/
            └── .gitkeep               # Placeholder para modelo Qwen GGUF
```

---

## Diagrama de Comunicación

```
┌─────────────────────────────────────────────────────┐
│                  yarvis-app.exe                      │
│                                                     │
│  ┌──────────────┐         ┌──────────────────────┐  │
│  │   Frontend   │ ◄─────► │    Backend Rust      │  │
│  │   React +    │ invoke  │    (Tauri IPC)       │  │
│  │   TypeScript │         │                      │  │
│  │   Tailwind   │         │  auth, inventory,    │  │
│  │              │         │  tickets, parser     │  │
│  └──────────────┘         └──────────┬───────────┘  │
│                                       │              │
│                              ┌────────▼────────┐    │
│                              │   SQLite (WAL)   │    │
│                              │   yarvis.db      │    │
│                              └────────┬────────┘    │
│                                       │              │
└───────────────────────────────────────┼──────────────┘
                                        │
                              ┌─────────▼─────────┐
                              │  yarvis-IA/        │
                              │  FastAPI (Python)  │
                              │  127.0.0.1:8000   │
                              │                   │
                              │  Prophet (ventas) │
                              │  Qwen (chatbot)   │
                              └───────────────────┘
```

- **Frontend ↔ Rust**: Tauri IPC con `invoke()`
- **Rust ↔ SQLite**: sqlx async con modo WAL
- **Rust ↔ Python**: HTTP local en puerto dinámico (pendiente)
