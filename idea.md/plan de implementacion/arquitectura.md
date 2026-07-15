# Arquitectura del Proyecto Y.A.R.V.I.S. POS

Esta documentación refleja la estructura de directorios y archivos de todo el sistema de manera exhaustiva. Cada módulo está dividido entre la Aplicación de Escritorio (Tauri + React + Rust) y el Motor de Inteligencia Artificial (FastAPI + Python). 
*Nota: Los archivos vacíos o que solo contienen comentarios se listan intencionalmente sin descripción adicional.*

## Estructura de Archivos y Directorios

```text
Y.A.R.V.I.S.-POS/
├── run.bat                            # Script para Windows: instala dependencias y ejecuta Tauri dev.
├── run.sh                             # Script para Linux: levanta el entorno de desarrollo.
├── reset.sh                           # Script de limpieza: borra la BD y datos cacheados para dejar el sistema en blanco.
├── README.md                          # Documentación principal del repositorio.
│
├── idea.md/                           # Documentación de planificación y diseño del sistema.
│   └── plan de implementacion/
│       ├── idea a vender.md           # Concepto del negocio y propuesta de valor.
│       ├── tecnologias.md             # Stack tecnológico elegido y su justificación.
│       ├── arquitectura.md            # ESTE ARCHIVO: Mapa completo del proyecto.
│       ├── comandos.md                # Comandos útiles de desarrollo.
│       ├── implementacion.md          # Fases y roadmap de desarrollo.
│       ├── interconexion.md           # Diseño de comunicación entre Rust y Python.
│       └── logica de proceso.md       # Lógica de negocio y flujos del POS.
│
├── yarvis-app/                        # Aplicación de Escritorio (Frontend React + Backend Rust).
│   ├── package.json                   # Dependencias de Node.js (React, Vite, Tauri, Tailwind).
│   ├── vite.config.ts                 # Configuración del empaquetador Vite (puerto 1420).
│   ├── tailwind.config.js             # Configuración de estilos y diseño visual.
│   ├── tsconfig.json                  # Reglas de TypeScript para el frontend.
│   │
│   ├── src/                           # FRONTEND: Interfaz Gráfica (React + TypeScript).
│   │   ├── main.tsx                   # Punto de entrada de React, monta la aplicación en el DOM.
│   │   ├── App.tsx                    # Enrutador principal, maneja el estado de login y navegación.
│   │   ├── App.css                    # Estilos globales de CSS.
│   │   ├── vite-env.d.ts              
│   │   │
│   │   ├── front-admin/               # Módulos exclusivos del Administrador.
│   │   │   ├── AdminDashboard.tsx     # Panel de control lateral y enrutador del Admin.
│   │   │   ├── PrimerInicio.tsx       # Asistente de configuración inicial.
│   │   │   ├── types.ts               # Tipos de TypeScript compartidos.
│   │   │   └── ventanas/              
│   │   │       ├── adminclientes/
│   │   │       │   └── clientes.tsx   # Gestión de base de datos de clientes.
│   │   │       ├── adminconfig/
│   │   │       │   └── configuracion.tsx # Ajustes globales del sistema.
│   │   │       ├── adminempleados/
│   │   │       │   ├── empleados.tsx  # Vista principal de gestión de personal.
│   │   │       │   ├── modalEmpleados.tsx # Modal de edición de empleado.
│   │   │       │   ├── modalMetas.tsx # Modal de metas del empleado.
│   │   │       │   └── modalTurnos.tsx# Modal de turnos del empleado.
│   │   │       ├── adminfinanzas/
│   │   │       │   └── finanzas.tsx   # Vista de contabilidad y reportes financieros.
│   │   │       ├── admininventario/
│   │   │       │   └── inventario.tsx # Gestión de productos, alertas de stock y CRUD.
│   │   │       ├── adminticket/
│   │   │       │   └── tickets.tsx    # Historial de ventas y cortes de caja.
│   │   │       ├── adminventas/
│   │   │       │   └── ventas.tsx     # Reporte y métricas de ventas.
│   │   │       ├── adminyarvis/
│   │   │       │   └── yarvis.tsx     # Interfaz directa con la IA administrativa.
│   │   │       └── parseadodetickets/
│   │   │           ├── BatchProcessor.tsx # Procesamiento en lote de archivos.
│   │   │           ├── CatalogosParseados.tsx # Vista de catálogos subidos e historial.
│   │   │           └── ColumnMapper.tsx # Asignación de columnas para la importación.
│   │   │
│   │   └── front-empleado/            # Módulos del Empleado (Punto de Venta).
│   │       ├── EmployeeDashboard.tsx  # Interfaz principal de la caja registradora.
│   │       └── ventanas/
│   │           ├── emplea_new_venta/
│   │           │   └── nueva_venta.tsx# Carrito de compras, búsqueda por código e integración de IA.
│   │           ├── empleaajustes/
│   │           │   └── ajustes.tsx    # Configuración básica del perfil del empleado.
│   │           ├── empleaclientes/
│   │           │   └── clientes.tsx   # Búsqueda y selección de clientes en caja.
│   │           ├── empleainventario/
│   │           │   └── inventario.tsx # Consulta de stock (sin permisos de edición).
│   │           ├── empleaperfil/
│   │           │   └── perfil.tsx     # Datos del turno y empleado.
│   │           ├── empleaticket/
│   │           │   └── ticket.tsx     # Impresión y revisión de tickets recientes.
│   │           └── empleayarvis/
│   │               └── yarvis.tsx     # Interacción con asistente de IA en caja.
│   │
│   └── src-tauri/                     # BACKEND RUST: Lógica nativa y base de datos local.
│       ├── tauri.conf.json            # Configuración de Tauri (permisos, plugins, Sidecar de Python).
│       ├── Cargo.toml                 # Dependencias de Rust (sqlx, tokio, serde, tauri).
│       │
│       └── src/                       # Código fuente de Rust.
│           ├── main.rs                # Punto de entrada de Rust, invoca la librería principal.
│           ├── lib.rs                 # Orquestador de Tauri: inicializa SQLite y Sidecar.
│           ├── models.rs              # Structs de Rust para mapeo de datos.
│           ├── sidecar.rs             # Controlador del ciclo de vida de la IA (yarvis-IA).
│           │
│           └── backventanas/          # Comandos IPC divididos por dominio.
│               ├── mod.rs             # Archivo raíz que exporta todos los módulos backend.
│               ├── db/                
│               │   ├── db.rs          # Gestor de base de datos SQLite WAL, creación de tablas.
│               │   └── mod.rs         
│               ├── backadmin/         # Comandos IPC exclusivos para el administrador.
│               │   ├── mod.rs         
│               │   ├── adminclientes/ 
│               │   │   ├── clientes.rs
│               │   │   └── mod.rs     
│               │   ├── adminconfig/   
│               │   │   ├── auth.rs    # Lógica de autenticación y configuración.
│               │   │   └── mod.rs     
│               │   ├── adminempleados/
│               │   │   ├── empleados.rs # Gestión de personal.
│               │   │   ├── modalempleado.rs # Comandos para modal de empleados.
│               │   │   ├── modalmetas.rs # Comandos para modal de metas.
│               │   │   ├── modalturnos.rs # Comandos para modal de turnos.
│               │   │   └── mod.rs     
│               │   ├── adminfinanzas/ 
│               │   │   ├── finanzas.rs
│               │   │   └── mod.rs     
│               │   ├── admininventory/
│               │   │   ├── inventory.rs # Comandos CRUD de inventario.
│               │   │   └── mod.rs     
│               │   ├── adminparser/   # Utilerías de análisis de documentos.
│               │   │   ├── parser_commands.rs # Comandos de Tauri para parseo.
│               │   │   ├── parser_csv.rs # Lógica en Rust para parsear CSV.
│               │   │   ├── parser_excel.rs # Lógica en Rust para parsear Excel.
│               │   │   ├── parser_txt.rs # Lógica en Rust para parsear TXT.
│               │   │   ├── utils.rs   # Funciones de ayuda general para el parseo.
│               │   │   └── mod.rs     
│               │   ├── admintarvis/   
│               │   │   ├── ai.rs      # Conexión directa del Admin con endpoints de IA.
│               │   │   └── mod.rs     
│               │   ├── admintickets/  
│               │   │   ├── tickets.rs # Comandos de Tauri para historial de tickets.
│               │   │   └── mod.rs     
│               │   └── adminventas/   
│               │       ├── ventas.rs  
│               │       └── mod.rs     
│               └── backempleado/      # Comandos IPC para el empleado.
│                   ├── mod.rs         
│                   ├── emplea_new_venta/
│                   │   ├── new_venta.rs # Lógica nativa para procesar la nueva venta.
│                   │   └── mod.rs     
│                   ├── empleaconfig/  
│                   │   ├── config.rs  
│                   │   └── mod.rs     
│                   ├── empleafinanzas/
│                   │   ├── finanzas.rs
│                   │   └── mod.rs     
│                   ├── empleainventario/
│                   │   ├── inventario.rs
│                   │   └── mod.rs     
│                   ├── empleaparser/  
│                   │   ├── parser.rs  
│                   │   └── mod.rs     
│                   ├── empleaperfil/  
│                   │   ├── perfil.rs  # Consulta de perfil y permisos del empleado.
│                   │   └── mod.rs     
│                   ├── empleatickets/ 
│                   │   ├── tickets.rs 
│                   │   └── mod.rs     
│                   ├── empleaventas/  
│                   │   ├── ventas.rs  
│                   │   └── mod.rs     
│                   └── empleayarvis/  
│                       ├── yarvis.rs  
│                       └── mod.rs     
│
└── yarvis-IA/                         # BACKEND IA: Motor de Inteligencia Artificial (Python).
    ├── main.py                        # Servidor FastAPI: Punto de entrada, expone endpoints (/chat, etc).
    ├── requirements.txt               # Dependencias de Python.
    ├── clean_all.sh                   # Script para limpiar cachés y temporales de IA.
    ├── core/                          # Lógica central.
    │   ├── embeddings.py              # Inferencia del modelo "all-MiniLM-L6-v2" para búsqueda semántica.
    │   ├── utils.py                   # Utilidades generales de la IA.
    │   └── __init__.py                
    ├── endpoints/                     # Controladores de la API FastAPI.
    │   ├── chat.py                    # Interacción con LLM chatbot.
    │   ├── embeddings.py              # Vectorización de texto.
    │   ├── matching.py                # Búsqueda de similitud (Vector Search).
    │   ├── parser.py                  # Extracción de datos de documentos.
    │   ├── predictions.py             # Predicciones de Prophet.
    │   └── __init__.py                
    ├── modelos/                       # Modelos ML.
    │   ├── profeta/                   
    │   │   ├── predictor.py           # Inferencia usando Meta Prophet.
    │   │   └── .gitkeep               
    │   └── qwen/                      
    │       ├── rutas.py               # Configuración de rutas a archivos .gguf en LM Studio.
    │       ├── parser_llm.py          # Lógica para extraer JSON usando Qwen.
    │       └── .gitkeep               
    └── parser_py/                     # Procesadores de documentos en Python.
        ├── parser_csv.py              # Analizador de CSV.
        ├── parser_excel.py            # Analizador de XLSX.
        ├── parser_txt.py              # Analizador de texto plano.
        └── __init__.py                
```

---

## Diagrama de Comunicación General

```text
┌─────────────────────────────────────────────────────────────┐
│                       yarvis-app.exe                        │
│                                                             │
│  ┌──────────────────┐         ┌──────────────────────────┐  │
│  │   Frontend       │ ◄─────► │    Backend Rust          │  │
│  │   (React+Vite)   │ invoke  │    (Tauri IPC)           │  │
│  │                  │         │    Módulo `backventanas` │  │
│  └────────┬─────────┘         └──────────┬───────────┬───┘  │
│           │                              │           │      │
│           │                  ┌───────────▼────────┐  │      │
│           │                  │   SQLite (WAL)     │  │      │
│           │                  │   yarvis.db        │  │      │
│           │                  └────────────────────┘  │      │
│           │                                          │      │
│           │ (Peticiones HTTP o control de procesos)  │      │
└───────────┼──────────────────────────────────────────┼──────┘
            │                                          │
            │ Sidecar (Process Management)             │
  ┌─────────▼──────────────────────────────────────────▼─────┐
  │                        yarvis-IA/                        │
  │                     FastAPI (Python)                     │
  │                      127.0.0.1:XXXX                      │
  │                                                          │
  │  ┌────────────┐   ┌──────────────┐   ┌────────────────┐  │
  │  │   Prophet  │   │     Qwen     │   │   Embeddings   │  │
  │  │ (Ventas)   │   │ (LLM .gguf)  │   │  (MiniLM-L6)   │  │
  │  └────────────┘   └──────────────┘   └────────────────┘  │
  └──────────────────────────────────────────────────────────┘
```

- **Frontend ↔ Rust**: Comunicación ultrarrápida usando IPC nativo de Tauri (`invoke()`).
- **Rust ↔ SQLite**: Interacción directa con `sqlx` en modo asíncrono para asegurar que la UI nunca se congele.
- **Rust ↔ Python (IA)**: Rust inicia y apaga el servidor de Python como un proceso hijo (Sidecar). Rust o React se comunican con Python enviando peticiones HTTP REST a un puerto asignado dinámicamente en `localhost`.
