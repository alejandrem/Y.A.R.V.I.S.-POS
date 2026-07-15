# Tecnologías Detalladas - Y.A.R.V.I.S. POS

## Lenguajes de Programación
> * **Rust**: Núcleo del backend y lógica de alto rendimiento. Rust (Axum, Tokio, SQLx).
> * **Python**: Procesamiento de datos, modelos predictivos e IA. FastAPI, Meta Prophet, llama-cpp-python.
> * **TypeScript**: Desarrollo del frontend y tipos seguros. TypeScript (Vite, React, Tailwind CSS, Shadcn UI).
> * **SQL (SQLite)**: Consultas de base de datos relacional. SQLite 3 + sqlite-vec.
> * **Bash**: Scripts de automatización y despliegue local.

## Frontend (Interfaz de Usuario Premium)
> * **Contenedor de Escritorio**: Tauri (Crea la ventana nativa como software de Windows sin usar navegadores externos).
> * **Lenguaje**: TypeScript.
> * **Framework**: Vite + React. Es rapidísimo al compilar y se integra de forma perfecta con Tauri.
> * **Estilos**: Tailwind CSS.
> * **Componentes**: Shadcn UI.
> * **Animaciones**: Framer Motion.
> * **Iconografía**: Lucide React.
> * **Gráficas**: Recharts.
> * **Gestión de Estado**: Zustand.
> * **Consumo de APIs**: TanStack Query (React Query).
> * **Validación**: Zod + React Hook Form.

## Backend (Motor de Alto Rendimiento en Rust)
- Framework Web: Axum.
- Runtime Asíncrono: Tokio.
- Serialización: Serde.
- ORM/Query Builder: SQLx (con soporte para SQLite).
- Middleware: Tower & Tower-HTTP.
- Logging/Tracing: Tracing.
- Manejo de Errores: Anyhow & Thiserror.
- Seguridad: Argon2 (Hashing) & Jsonwebtoken (JWT).
- Detección de Hardware: sysinfo (detecta RAM disponible para seleccionar el modelo de IA correcto).

## IA y Ciencia de Datos (Cerebro en Python)
- Predicción de Ventas: Meta Prophet.
- Inferencia de LLM Local: llama-cpp-python (modo CPU exclusivo, sin GPU para máxima portabilidad).
- Modelo local a usar: Qwen 2.5 en formato GGUF. Se selecciona 1.7B o 0.5B según la RAM detectada.
- Orquestación de IA: Usaremos solo llama-cpp (ya no se usarán frameworks pesados como LangChain o LlamaIndex debido a su alto consumo de RAM).
- Procesamiento de Datos: Pandas, NumPy, Scikit-learn.
- Servidor de IA Interno: FastAPI + Uvicorn.
Excel/CSV y TXT en Fase 1.

## El Cerebro de IA (RAG + Prophet + SQL)

- RAG (Búsqueda Semántica): Tabla unificada `knowledge_base` en SQLite con categorías (productos, festividades, manuales). Busca por significado, no por palabras exactas.
- Meta Prophet: Convierte el pasado (historial) en futuro (proyecciones matemáticas) con intervalos de confianza. El LLM consulta a Prophet para no alucinar números.
- agregar un modelo minúsculo especializado únicamente en embeddings (como all-MiniLM-L6-v2.gguf o nomic-embed-text). Pesan unos 40MB, son rapidísimos y le quitarán una carga inmensa de trabajo a Qwen, quien podrá dedicarse a "platicar" en lugar de hacer matemáticas de vectores.

---

### Base de Datos
- Motor Principal: SQLite 3.
- Búsqueda Vectorial: sqlite-vec.
- Modo WAL: Permite que Rust escriba ventas mientras Python lee datos simultáneamente sin bloqueos.
- Regla de Oro: Rust es el ÚNICO que escribe en SQLite. Python solo lee. Para guardar embeddings: cuando Rust inserta un producto, le envía un HTTP POST a Python (`/generar_embedding`). Python calcula el vector numérico y se lo devuelve a Rust, quien hace el INSERT.
- Sincronización: Las predicciones deben recalcularse en lotes (en el corte Z o cada 12 horas). Rust envía un "Ping" vía HTTP a Python. Python calcula las proyecciones de los próximos 7 días con Prophet y envía un HTTP POST a Rust para que este las guarde en la tabla `predicciones_futuras` en SQLite. Así el Chatbot responde al instante.

### Hardware y Comunicación
- Impresión Térmica: Protocolo ESC/POS.
- API de Impresión: Windows Print Spooler API.
- Entorno de Ejecución: Node.js (para herramientas de soporte) y Bun.

### El Modelo "Sidecar" (Jefe y Empleado)
- Rust (Jefe): Gestiona ventas, inventario, impresión y orquestación. Único escritor de SQLite.
- Python (Empleado): Gestiona IA, predicciones y chatbot. Solo lectura en la DB.
- Comunicación: HTTP Local vía FastAPI en puerto dinámico (seleccionado por Rust al inicio para evitar colisiones).

### Selección Adaptativa de IA (Semaforización de RAM Libre)
- >= 2.5GB RAM libre: Carga Qwen 2.5 1.7B GGUF en cuantización Q6 (Cerebro Inteligente).
- < 2.5GB RAM libre: Carga Qwen 2.5 0.5B GGUF en cuantización Q6 (Cerebro Ligero).

if ram_libre >= 2.5 { cargar(1.7B_Q6) } else { cargar(0.5B_Q6) }.

### Lazy Loading del LLM
Para lograr un Lazy Loading real, Rust arranca a Python en segundo plano pero **sin cargar ningún LLM** pesado. El consumo de memoria es mínimo. 
Justo en el instante en que el usuario intenta abrir el Chatbot, Rust verifica una variable interna (ej. `llm_estado`). Solo si el estado es 'descargado', Rust mide la memoria RAM libre en ese momento, toma la decisión del modelo (1.7B o 0.5B) y le envía una petición HTTP a Python: "Carga el modelo X ahora" (ej. `POST /load_llm`). Si ya estaba cargado, omite medir la RAM para evitar colapsos.
Nota: A diferencia de Qwen, el pequeño modelo de embeddings de 40MB (`all-MiniLM`) sí se carga al iniciar el sistema, ya que si el cajero da de alta un "nuevo producto", Python necesita crear su vector numérico en segundo plano sin importar si el Chatbot está abierto o no.

### Empaquetamiento y Produccion

1. Para empaquetar el Frontend y Rust:
   - Utilizar Tauri (`npm run tauri build`). Esta herramienta compila el frontend (TypeScript/Vite) y el backend de Rust automáticamente en un solo ejecutable `.exe`.
   - sysinfo: para detectar la ram del usuario (incluido en el código de Rust).

2. Para empaquetar la IA y python:
   - PyInstaller: La herramienta principal para convertir los scripts de IA y todas sus librerías en un solo archivo .exe.
   - UPX (Ultimate Packer for eXecutables): (Opcional pero recomendado) Comprime los archivos .exe finales para que pesen menos en la USB.
   - GGUF: El formato de archivo que permite que los modelos de IA sean un solo archivo "pesado" fácil de copiar y mover.

3. Binario final (Rust):
   - cargo build --release: El comando nativo de Rust que compila todo (incluyendo el frontend ya embebido) en el ejecutable final optimizado.

- - Frontend y Rust: Empaquetados en una ventana nativa mediante Tauri.
- IA Engine: PyInstaller (compila Python a .exe) + UPX (compresión).
- Distribución: Carpeta raíz portable con yarvis-app.exe, yarvis.db y carpeta /engine.

## Pro-Tips para la Portabilidad
- Rutas con current_exe(): Nunca usar "./" estático. Siempre calcular rutas desde la ubicación física del ejecutable para evitar el bug del acceso directo de Windows.
- Un solo escritor en SQLite: Rust escribe, Python lee. Si Python necesita guardar algo, le pide a Rust via HTTP.
- Dependencias Externas: conexión a internet para API del clima y facturacion (opcional). Para que Prophet pueda predecir con el clima, Rust debe consultar el clima diario durante el Corte Z y guardarlo junto a las ventas en la base de datos (historial climático).
- Fase 1 Testing: Probar la portabilidad en cuanto la base de datos y el primer script de Rust estén listos.
- Modo Degradado: Si el motor de IA falla, el POS sigue funcionando en modo "Venta Clásica" sin Chatbot. La caja nunca depende de la IA.

### Herramientas de Desarrollo
- Control de Versiones: Git.
- Gestores de Paquetes: Cargo (Rust), Pip/Poetry (Python), npm/bun (JS/TS).
- para los embeddings usaremos de txt -> embeddings usaremos el modelo phi-3.5-embeddings
