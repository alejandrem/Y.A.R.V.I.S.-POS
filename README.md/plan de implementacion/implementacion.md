# Plan de Implementación Riguroso - Y.A.R.V.I.S. POS 🚀

Bienvenido al mapa de batalla. Este no es un proyecto de fin de semana, es una obra de ingeniería. Para asegurar que Y.A.R.V.I.S. sea un software robusto, escalable y mantenible a lo largo de los años, seguiremos este plan de implementación dividido en "Olas". 

> ⚠️ **REGLA DE ORO DEL CÓDIGO** ⚠️
> **Ningún archivo (ya sea `.rs`, `.py`, `.ts` o `.tsx`) deberá pasar de las 600 a 650 líneas de código.** 
> Si un archivo (por ejemplo, el gestor de inventario en Rust o el endpoint del LLM en Python) llega a las 650 líneas, debes detenerte, crear un nuevo archivo y dividir la lógica a la mitad (modularización). Archivos enormes equivalen a deuda técnica e imposibilidad de rastrear bugs. ¡Mantenlo modular!

---

## 🌊 Primera Ola: La Fundación de Hierro (Infraestructura y BD)

El objetivo de esta ola es que el Punto de Venta pueda funcionar en "Modo Clásico" (como una caja registradora normal), sin tocar la IA. Si la caja no funciona perfecto, la IA no tiene propósito.

**Paso 1.1: El Esqueleto del Workspace**
- Inicializarás el proyecto usando Tauri + Vite + React + TypeScript.
- Configurarás `tailwind.css` y `shadcn/ui` para empezar a construir la interfaz.
- En la carpeta `src-tauri`, prepararás tu `Cargo.toml` con las librerías críticas: `sqlx`, `tokio`, `serde`, y `sysinfo`.
usando como principal socio al comando `npm create tauri-app@latest`. con este comando se aplicaran los cambios indemadiatamente
aqui los comandos 
alee@alee-Vivobook-Go-E1404FA-E1404FA:~/Documentos/Y.A.R.V.I.S. POS$ npm create tauri-app@latest
✔ Project name · yarvis-app
✔ Identifier · com.yarvis.pos
✔ Choose which language to use for your frontend · TypeScript / JavaScript - (pnpm, yarn, npm, deno, bun)
✔ Choose your package manager · npm
✔ Choose your UI template · React - (https://react.dev/)
✔ Choose your UI flavor · TypeScript

Template created!

Your system is missing dependencies (or they do not exist in $PATH):
╭────────────────────┬───────────────────────────────────────────────────────────────────╮
│ Rust               │ Visit https://www.rust-lang.org/learn/get-started#installing-rust │
├────────────────────┼───────────────────────────────────────────────────────────────────┤
│ webkit2gtk & rsvg2 │ Visit https://tauri.app/guides/prerequisites/#linux               │
╰────────────────────┴───────────────────────────────────────────────────────────────────╯

Make sure you have installed the prerequisites for your OS: https://tauri.app/start/prerequisites/, then run:
  cd yarvis-app
  npm install
  npm run tauri android init

For Desktop development, run:
  npm run tauri dev

For Android development, run:
  npm run tauri android dev


me aparecio eso por que no tenia instalados lenguajes principales pero tu si instalalos. para que no te aparezca eso 
usaremos CSS Tailwind 3

**Paso 1.2: Diseñar la Base de Datos Híbrida**
- Crearás el archivo `yarvis.db` y forzarás a SQLite a activar el **Modo WAL** (`PRAGMA journal_mode=WAL;`).
- Primero crearás las tablas clásicas: `productos`, `ventas`, `detalle_ventas`, `clientes` y `ventas_diarias` (esta última tendrá columnas `temperatura_promedio` y `clima` para guardar el historial meteorológico).
- Después, configurarás la tabla para las proyecciones: `predicciones_futuras`.
- Y finalmente la tabla vectorial con `sqlite-vec`: `knowledge_base` (donde vivirán los embeddings).

**Paso 1.3: Conexión Rust <-> Interfaz**
- En Rust, programarás los `Tauri Commands` (`#[tauri::command]`) para hacer el CRUD: crear productos, cobrar tickets, ver inventario.
- En TypeScript (React), consumirás estos comandos. Aquí probarás que la caja registradora funciona al 100% y que Rust escribe perfectamente en SQLite.

---

## 🌊 Segunda Ola: El Cerebro Asíncrono (El Jefe llama al Empleado)

Aquí es donde entra la magia del "Sidecar". Vas a crear el motor de Python aislado y conectarás a Rust con él de manera invisible para el usuario.

**Paso 2.1: El Motor en Python**
- Crearás la carpeta `ai_engine` y configurarás un entorno virtual (`venv`).
- Programarás un servidor `FastAPI` súper rápido con rutas vacías: `GET /health`, `POST /load_llm`, `POST /generar_embedding`.
- Configurarás que Python arranque **únicamente** con el modelo pequeño de embeddings de 40MB (`all-MiniLM-L6-v2.gguf`). El LLM grande aún no se toca.

**Paso 2.2: La Secuencia de Boot (Arranque)**
- En Rust, modificarás el método de inicio. Rust buscará dos puertos TCP libres (ej. 54321 y 54322).
- Rust usará `std::process::Command` para ejecutar `ai_service.exe` pasándole esos puertos por argumentos.
- Rust creará un bucle esperando (timeout 30s) haciendo peticiones a `GET /health` hasta que Python responda "OK, estoy listo".
- *Control Anti-Apagones:* Justo al arrancar, Rust leerá la tabla `predicciones_futuras`. Si los datos son de ayer, activará una bandera para forzar una actualización en cuanto la CPU esté libre.

**Paso 2.3: La Prueba de Fuego de los Embeddings**
- Desde la interfaz de React, agregarás un producto: "Galletas de Chocolate".
- Rust lo insertará en la tabla `productos`.
- Inmediatamente, Rust hará una llamada `HTTP POST` a Python (a `127.0.0.1:54321/generar_embedding`).
- Python masticará las palabras "Galletas de Chocolate", devolverá el vector `[0.15, -0.42...]`, y **Rust** (obedeciendo la Regla de Oro) insertará ese vector en `knowledge_base`.

---

## 🌊 Tercera Ola: El Profeta y la Ingesta Masiva

Es hora de enseñar a Y.A.R.V.I.S. a ver el futuro.

**Paso 3.1: El Parseador Premium (Fuera del Sistema)**
- Crearás un script de Python independiente (en tu laptop de desarrollador).
- Este script tragará los 12,000 tickets históricos del cliente en formato TXT/Excel, limpiará nombres y guardará todo directamente en `yarvis.db` (la excepción a la Regla de Oro, pues esto se hace offline).

**Paso 3.2: El Corte Z y el Ping del Futuro**
- En Rust, cuando el cajero presione "Hacer Corte de Caja", Rust primero hará un `HTTP GET` a la API de clima (ej. OpenWeather), guardará la temperatura del día en la tabla de ventas, y cerrará la caja.
- Luego, Rust enviará un `HTTP POST` a Python: `/recalcular_predicciones`.
- Python despertará a Meta Prophet. Prophet leerá las ventas pasadas y el clima histórico, y calculará los próximos 7 días (ej. Venderás 40 panes mañana).
- Python enviará un `POST` de regreso a Rust con un JSON masivo, y Rust actualizará la tabla `predicciones_futuras`.

---

## 🌊 Cuarta Ola: El Chatbot y el "Lazy Loading" 

Aquí el usuario finalmente sentirá la inteligencia del sistema.

**Paso 4.1: Semaforización de RAM y Lazy Loading**
- En React, el usuario da clic en la pestaña "Consultar Y.A.R.V.I.S.".
- Rust captura el clic, revisa su variable `llm_estado`. Si dice "descargado", Rust usa `sysinfo` para revisar la RAM libre.
- ¿Hay 2.5GB libres? Rust manda `POST /load_llm {"model": "1.7B_Q6"}`. ¿Hay menos de 2.5GB libres? Manda `{"model": "0.5B_Q6"}`.
- Rust actualiza `llm_estado = "cargado"`. El chat se abre.

**Paso 4.2: RAG y Function Calling**
- Crearás el *System Prompt* en Python explicándole los límites a Qwen ("Si te preguntan cruces muy complejos, sé educado y di que no puedes").
- Conectarás las herramientas: Si el modelo pide la utilidad, Python leerá `yarvis.db` (ventas) y le devolverá el número. Si el modelo pide pronósticos, leerá `predicciones_futuras`. Si pide un producto, usará el vector para consultar `knowledge_base`.

---

## 🌊 Quinta Ola: Producción, Periféricos y Guerrillas

Esta es la ola del dolor. Convertir código hermoso en un ejecutable que sobreviva en la jungla de Windows.

**Paso 5.1: Domando a la Impresora**
- En Rust, escribirás el módulo `impresion.rs`. Utilizarás los comandos ESC/POS básicos.
- Probarás mandar comandos por medio del "Print Spooler" nativo de Windows. Imprimirás tickets, cortarás papel y abrirás la caja registradora de metal enviando el código hex `27 112 0 25 250`.

**Paso 5.2: El "Infierno del Empaquetado"**
- Ejecutarás `PyInstaller` para el servidor de Python. Asegúrate de incluir las banderas `--onedir` o `--onefile` (recomendable `--onedir` para evitar que Windows Defender lo detecte falso positivo por desempaquetado en RAM).
- Moverás ese ejecutable compilado a la carpeta `/engine`.
- En Rust, cambiarás todas tus rutas quemadas por `std::env::current_exe()` para que busque a Python de manera relativa.
- Ejecutarás `npm run tauri build` para obtener tu joya final: `yarvis-app.exe`.

**Paso 5.3: Prueba de Trinchera**
- Agarrarás la carpeta resultante, la meterás en una memoria USB y te irás a la laptop con Windows 10 más vieja de un amigo o familiar.
- Insertarás la USB, darás doble clic y rezarás. Si Windows Defender salta, modificarás las firmas. Si falta un DLL, lo agregarás al empaquetado. 

¡Una vez finalices la Quinta Ola, tendrás un POS invencible!
