aqui definiremos como estaran conectadas y comunicadas todas las tecnologias para que funcione correctamente.

### =============== Python ================

Python es el "Cerebro Creativo". Su función no es gestionar la caja, sino analizar lo que pasa en ella.

Profeta es la joya de la corona, siendo una libreria muy potente para predecir el futuro basandose en el pasado, su trabajo seria leer los ultimos meses de ventas de `yarvis.db`, detecta que los viernes se vende mas cerveza, o que en diciembre sube el chocolate. Es el que le dice al dueño "el proximo fin de semana es 15 de septiembre, segun mis calculos vas a vender un 40% mas de refrescos. te sugiero comprar 5 cajas extra con un 95% de confianza"

el RAG + sqlite-vec: aquí es donde Python usa la IA para entender el inventario. Cuando el cajero crea un producto, Rust lo guarda y le envía un HTTP POST a Python (`/generar_embedding`). Python convierte la descripción en un "vector numérico" y se lo devuelve a Rust para que este lo guarde en sqlite-vec (respetando la Regla de Oro). Cuando el dueño pregunta al chatbot "qué productos de limpieza son aptos para madera", el bibliotecario (Python) solo lee la base de datos y busca no por palabras exactas sino por significado.

cuando el cliente llega con sus 12,000 tickets viejos en excel o txt, python tiene herramientas para masticar miles de datos en segundos, limpia nombres duplicados, arregla precios mal escritos y deja la base de datos reluciente para que rust pueda empezar a vender. este proceso debe hacerse en una laptop del creador del software. (Nota Técnica: Como este proceso de "Onboarding" se hace fuera del Punto de Venta y con el sistema apagado, aquí se hace una excepción a la "Regla de Oro": este script especial de Python sí escribe directamente en yarvis.db porque no hay riesgo de chocar con Rust).

y como rust y python hablan distintos idiomas, python levanta un pequeño puesto de traduccion. Su trabajo es escuchar las peticiones de rust via HTTP. Con esto recibe un mensaje de rust diciendo `get prediccion`, corre prophet y le devuelve a rust un json sencillo con los resultados.

Python no cobra, Python aconseja. Su tiempo se gasta en pensar, calcular probabilidades y entender el lenguaje natural.

### ================ Rust ==================

Rust en lugar de fallar debe ser quien gestione errores. Si la impresora esta desconectada o la IA tarda en responder, Rust no debe morir, debe capturar el error y decirle al usuario: "la venta se guardo pero la impresora no responde. ¿quieres reintentar imprimir el ticket?". Usar Result y Option en todo el codigo es clave.

Tambien rust nos obliga a definir que es una venta o un producto, crea structs que no permitan estados imposibles, como "que una venta no pueda tener un total negativo".

Aprovechando que rust es el rey de la concurrencia gracias a tokio: mientras rust esta esperando la respuesta de la IA (que puede tardar 2 a 5 segundos) no debe bloquear la caja, debe usar tareas asincronas para que el cajero pueda seguir cobrando al siguiente cliente mientras el "cerebro" de python procesa el ticket anterior.

A diferencia de otros lenguajes no tiene basurero y gracias a esto el consumo de RAM de rust sera bajisimo, como unos 20 o 30MB. Esto deja toda la ram libre para que los modelos de IA que si son pesados puedan correr sin problemas.

Rust hace el trabajo duro y aburrido (seguridad, base de datos, impresión, red) de forma perfecta, mientras Python hace el trabajo creativo (IA, predicción).

NUEVO: Rust es el ÚNICO que escribe en SQLite. Si Python necesita guardar algo (por ejemplo, embeddings nuevos de un producto), debe enviarle un HTTP POST a Rust, y que Rust sea quien lo inserte en SQLite. Si ambos intentan escribir al mismo tiempo, la DB se bloqueará y el cajero no podrá cobrar.

NUEVO: Al arrancar, Rust usa std::env::current_exe() para conocer su ubicación física real y calcular todas las rutas relativas desde ahí. Esto evita el bug del acceso directo de Windows, donde el sistema buscaría yarvis.db en el Escritorio en lugar de en la carpeta real.

NUEVO: Rust busca un puerto TCP libre al azar al iniciar (ej. 54321) y se lo pasa a Python como argumento: ./engine/ai_service.exe --port 54321. Esto evita colisiones con otros programas del cliente que ya usen el puerto 8000.

### ======== Frontend =========

El Frontend es la cara visual de Y.A.R.V.I.S., construido con **TypeScript**, Vite y React. Como es un software de escritorio y no queremos usar navegadores de terceros para la ejecución del sistema, **Tauri** actúa como nuestro "marco de la ventana".

Tauri levanta una ventana nativa del sistema operativo y renderiza la interfaz de React dentro de ella usando el Webview nativo (sin necesidad de abrir Google Chrome o Edge).

Comunicación Frontend <-> Backend: 
En lugar de depender únicamente de peticiones web para conectar el frontend con Rust, el código en TypeScript utiliza los **Tauri Commands** (`invoke()`). Esto permite que el frontend llame a funciones de Rust directamente por IPC (Inter-Process Communication) y reciba respuestas de manera rápida y altamente segura. La interfaz se encarga de mostrar indicadores de carga mientras Rust procesa tareas o interactúa con Python de fondo.

### ======== SQLite y SQLite-Vec =========

Para que la base de datos sea eficiente y compartida por ambos "mundos" (Rust y Python), la clave es tratar a SQLite como el corazón central. Usaremos SQLite-Vec, una extensión que le da "superpoderes" a SQLite para manejar Vectores (embeddings). Sin esto, el Chatbot no sabría qué productos recomendar.

Aunque Rust sea el "Jefe", ambos ejecutables pueden abrir el mismo archivo yarvis.db al mismo tiempo. SQLite es excelente para esto siempre que activemos el modo WAL (Write-Ahead Logging): varias personas leen la base de datos mientras otra está escribiendo, sin que nadie tenga que esperar.

Rust escribe las ventas y actualiza el inventario.
Python lee ese mismo archivo para entrenar a Prophet y para hacer búsquedas semánticas.

## Gestión de Datos con SQLite + sqlite-vec

Para que el sistema sea rápido e inteligente, la DB se configurará así:

- Modo WAL: Activado para permitir que Rust escriba ventas mientras Python analiza datos simultáneamente sin bloqueos.
- Carga de Extensión: El archivo `sqlite_vec.dll` residirá en `/engine`. Ambos servicios lo cargarán al iniciar su conexión.
- Tablas Híbridas:
    - `ventas`, `productos`, `clientes`: Tablas relacionales estándar (manejadas únicamente por Rust).
    - `knowledge_base`: Tabla unificada de sqlite-vec para búsqueda semántica (Python genera los embeddings, Rust los inserta).

### Flujo de Sincronización:

Cuando Rust registra un cierre de turno, le avisa vía HTTP a Python para que la IA actualice sus predicciones en tiempo real:

1. Rust guarda el cierre de turno con corte Z en yarvis.db.
2. Rust envía un "toque" (Ping) a Python: "Oye, acabaron de pasar las 12 horas o se hizo un corte Z, actualiza las predicciones".
3. Python recibe el aviso, lee la DB histórica, corre Prophet para calcular los próximos días, y le envía un HTTP POST a Rust con los resultados para que Rust los guarde en la tabla `predicciones_futuras`.

En caso de que no haya cierre de turno con corte Z las predicciones se actualizarán cada 12 horas de la misma forma para no perder inmediatez en el Chatbot.

### ======== Base de Conocimiento (Knowledge Base) =========

En lugar de 5 archivos distintos, toda la información de contexto vive en una sola tabla `knowledge_base` dentro de yarvis.db, diferenciada por categoría:

Contenido (Texto)                                              | Categoría (Tag)     
"16 de Septiembre: Independencia. Sube venta de tequila."      | festividad          
"Shampoo Marca X: Sin sal, ideal para cabello teñido."         | producto            
"Para corte de caja, ve al menú Finanzas y presiona F10."      | manual_herramienta  

Se hace así por simplicidad: Python solo tiene que buscar en un solo lugar, SQLite busca entre miles de vectores en milisegundos, y si quieres agregar una fecha festiva o un nuevo consejo de venta, solo insertas la fila en la tabla y listo.

Ejemplo de fechas festivas:

Fecha    | Evento        | Descripción Contexto
16-Sept  | Independencia | Feriado oficial. La gente se reúne en casas; aumenta la venta de tequila, desechables, carnes y botanas. Los negocios suelen cerrar temprano o tener picos por la mañana.
14-Feb   | San Valentín  | Día de alto consumo en regalos, flores y chocolates. Si es retail, los artículos de regalo tienen rotación 3x.

### ======== Fase de Carga del LLM =========

El RAG es perfecto para las preguntas de "que shampoo no tiene sal" o "que productos vendi hoy". Funciona buscando en sqlite-vec los productos que coinciden, recuperando las descripciones y pasándoselas al LLM para que las resuma. También incluiremos en el RAG las fechas nacionales más importantes, ya que en esas épocas se suele vender más licor, cerveza, refrescos o dulces.

EL LLM CONSULTA A PROPHET, NO A LA BASE DE DATOS DIRECTAMENTE PARA PREDECIR, porque la base de datos solo tiene el PASADO (lo que ya vendiste). El LLM por sí solo no puede "imaginar" el futuro de forma precisa. Necesita a Prophet para crear ese FUTURO basado en modelos estadísticos.

El Function Calling (Tools) son las herramientas para el contador". Para las preguntas de dinero como "¿cuanto genere hoy?", el RAG no sirve porque no hay documento que diga cuanto ganaste hoy; los datos estan vivos en tablas y se cambian en cuestion de minutos y horas. El flujo sería:

Usuario: "¿Cuánto gané hoy?"
IA (Pensando): "Para responder esto, necesito sumar las utilidades de la tabla ventas de hoy. Voy a generar una consulta SQL."
(esto se cumple con Utiliza Function Calling (Tools) estandarizados. En lugar de que el modelo escriba SQL, dale una herramienta predefinida llamada obtener_utilidad_del_dia(fecha). El LLM sólo decide ejecutar esa herramienta, y Python ejecuta el SQL duro (SELECT...) que tú ya programaste a mano y es 100% a prueba de fallos. Luego Python le devuelve el número 2543 al LLM para que te responda.)

Python: Ejecuta esa consulta en yarvis.db y recibe el número (ej: 2543).
IA: Recibe el 2543 y responde: "Hoy tu utilidad neta fue de $2,543 MXN."

La solución estándar en IA es inyectar el contexto en el System Prompt de forma dinámica. Cada vez que el usuario abre el chat, Rust o Python construyen un prompt invisible que dice algo así:

"Eres Y.A.R.V.I.S., un asistente de punto de venta. Hoy es Viernes, 1 de Mayo de 2026, y son las 8:57 PM. Usa las herramientas a tu disposición para responder."

Gracias a esto, si el dueño teclea "¿Cuánto gané hoy?", el LLM razona: "Ah, hoy es 2026-05-01. Llamaré a la herramienta obtener_utilidad(fecha='2026-05-01')".


El "PROMPT ENGINEERING" + Tools: Como usaremos modelos Qwen, que son muy buenos siguiendo instrucciones, solo tenemos que dar un "System Prompt" con instrucciones maestras y límites claros:
"Eres el asistente de Y.A.R.V.I.S. POS. Si te preguntan por ventas o dinero, usa la herramienta SQL. Si te preguntan por productos, usa RAG. Si te piden cruces de datos muy complejos que no tienes en tus herramientas, responde educadamente que aún no tienes esa función, no inventes datos."

Compatibilidad de CPU: Debemos configurar la IA para que use solo el procesador (CPU). Si intentamos usar tarjetas gráficas (GPU), el .exe se volvería muy frágil y dejaría de ser portable (porque no todas las tiendas tienen una GPU compatible).

Detección automática de RAM para la ejecución del LLM: Rust primero verifica su variable interna `llm_estado`. Solo si el estado es 'descargado', mide la RAM libre para evitar el ciclo infinito de carga si el usuario cierra y abre el chat rápidamente.

- Si tiene 2.5GB o más libres (>= 2.5GB): Carga el modelo de 1.7B en Q6 (Qwen 2.5 1.7B). Es muy inteligente y razona casi como un humano.
- Si tiene menos de 2.5GB libres (< 2.5GB): Carga el modelo de 0.5B en Q6 (Qwen 2.5 0.5B). Es más rápido y ligero, ideal para no "ahogar" una laptop viejita.

if ram_libre >= 2.5 { cargar(1.7B_Q6) } else { cargar(0.5B_Q6) }.


NUEVO - Lazy Loading del LLM: El motor de Python arranca junto con el sistema, pero NO carga el LLM a la RAM hasta que el usuario abra la pestaña del "Chatbot". Si el cajero solo está cobrando todo el día, la IA no consume ni 1 MB de memoria. Si no le hablan a la IA en 15 minutos, Python descarga el modelo de la RAM automáticamente.

- En 4GB (Vida o muerte): Si cargas el LLM desde que abre la app, la computadora se va a congelar. El Lazy Loading es obligatorio aquí.
- En 8GB o 16GB (Optimización Premium): Si el dueño abre YARVIS a las 8:00 AM y solo usa el Chatbot a las 9:00 PM, mantener el LLM cargado 13 horas le "secuestra" 1.5 o 2 GB de RAM a lo tonto. Con Lazy Loading, YARVIS es invisible en consumo de RAM hasta que realmente le hablen.

El flujo correcto (y el más seguro) es que el LLM no toca NADA directamente, es como un director ciego que tiene 3 teléfonos para pedir ayuda. El código de Python (tu orquestador) es el que hace el trabajo sucio dependiendo del "teléfono" que use el LLM:

Teléfono 1: Herramientas del Pasado/Presente (Consultas SQL hechas por ti).
Usuario: "¿Cuánto gané hoy?"
LLM: "Llamando a la herramienta ventas_del_dia(fecha='2026-05-01')".
Python: Recibe la orden, Python va a la Base de Datos (yarvis.db), ejecuta el SELECT SUM..., y le dice al LLM: "Fueron $2,543".
LLM: Le responde al humano: "Hoy ganaste $2,543".

Teléfono 2: Herramientas del Futuro (Prophet).
Usuario: "¿Cuánto pan voy a vender el fin de semana?"
LLM: "Llamando a la herramienta prediccion_ventas(producto='pan', dias=2)".
Python: Recibe la orden, Python va a la tabla `predicciones_futuras` (precalculada por Prophet en el último Corte Z) y le dice al LLM: "La tabla indica que sugerimos 40 piezas".
LLM: Le responde al humano: "Te sugiero comprar unas 40 piezas de pan".

Teléfono 3: Herramientas de Conocimiento (RAG + SQLite-Vec).
Usuario: "¿Qué shampoo no tiene sal?"
LLM: "Llamando a la herramienta buscar_producto(descripcion='shampoo sin sal')".
Python: Recibe la orden, Python va a la Base de Datos Vectorial, saca los textos y se los pasa al LLM.


### el LLM como tal no toca la DB, sólo genera "intenciones" (Function Calling). Pero Python sí lee la DB para alimentar al LLM cuando las preguntas son sobre el pasado o el presente, porque Prophet es inútil para responder "¿cuántos reembolsos hizo el cajero Juan a las 4pm?".

### ===Prophet===

El LLM es malo en matemáticas pero Prophet es un genio. El no lee texto, él calcula curvas, tendencias y picos.
Prophet no sabe lo que pasó "hoy", Prophet solo sirve para adivinar "mañana".

Ejemplo:
"¿Cuánto papel higiénico debo comprar para el próximo mes?"
El LLM (Director): Sabe que él no sabe de predicciones, así que llama a su especialista: Prophet.
Prophet (Especialista): Va a la base de datos, analiza los últimos 6 meses, detecta que los fines de mes vendes más papel, y le devuelve al LLM un reporte técnico: "Predicción: 40 paquetes. Margen de error: +/- 5. Confianza: 95%".
El LLM (Director): Traduce ese reporte técnico a un lenguaje humano: "Basado en tus ventas pasadas, te sugiero comprar 40 paquetes. Es muy probable que los necesites todos antes de que termine el mes".

### ==== Empaquetamiento y Produccion =====

- Paso 1: Ejecutamos el comando de Tauri (`npm run tauri build`).
- Paso 2: Tauri compila automáticamente el frontend de Vite en TypeScript y el backend de Rust, empaquetándolos juntos.
- Paso 3: Al final, tienes un solo archivo .exe que al abrirlo levanta la ventana nativa de escritorio con la interfaz adentro.
- Paso 4: La DB vive en un solo archivo llamado yarvis.db. Solo copiamos ese archivo junto al ejecutable y listo.
- Paso 5: Usar PyInstaller para convertir los scripts de IA en un segundo .exe independiente usando FastAPI.
- Paso 6: Dentro de la carpeta de distribución veríamos esto:

/YARVIS-POS
  ├── yarvis-app.exe        (El corazón: Rust + Frontend)
  ├── yarvis.db             (Toda la base de datos)
  └── /engine               (La IA y procesos de Python ya compilados)
        ├── ai_service.exe
        ├── sqlite_vec.dll
        └── /models
              ├── qwen-1.7b.gguf
              └── qwen-0.5b.gguf
              └──all-MiniLM-L6-v2.gguf

- Paso 7: Para mantener la portabilidad, usamos HTTP Local (FastAPI). Es como si tuvieras un pequeño internet privado dentro de la dentro de la computadora del cliente.

### Pro tips para no morir en el intento

- Evitar dependencias de terceros: Esto nos pone en una mejor posición para que el sistema sea portable al cambiar de entornos virtuales y usar rutas relativas.
- Empaquetar la fase 1: Cuando tengamos la fase 1 lista con la base de datos y el script para parsear los tickets, vamos a intentar moverlo de una carpeta a otra y luego de una USB a otra laptop.
- El problema de los 2 .exe: Como vamos a tener dos archivos .exe (el de Rust y el de la IA), el de Rust será el "Jefe". Cuando abras yarvis-app.exe, este se encargará de levantar automáticamente al "empleado" (engine/ai_service.exe) en segundo plano. Así el usuario solo tiene que dar un solo clic.
- Rutas con current_exe(): Nunca usar "./" estático. Siempre calcular rutas desde la ubicación física del ejecutable para evitar el bug del acceso directo de Windows.
- Un solo escritor en SQLite: Rust escribe, Python lee. Si Python necesita guardar algo, le pide a Rust via HTTP que lo haga.

## 1. El Modelo "Sidecar" (Jefe y Empleado)

El ejecutable de Rust (yarvis-app.exe) actúa como el orquestador principal y el ejecutable de Python (ai_service.exe) actúa como un servicio de apoyo especializado.

### Secuencia de Inicio (Boot Sequence):

1. Arranque del Jefe: El usuario ejecuta yarvis-app.exe.
2. Detección de Entorno:
   - Rust localiza yarvis.db usando std::env::current_exe() (no rutas relativas simples).
3. Verificación de Predicciones (Anti-Apagón): Rust lee el `timestamp` de la tabla `predicciones_futuras`. Si los datos son de ayer o tienen más de 12 horas, Rust asume que la PC se apagó abruptamente y preparará un Ping a Python tras el inicio.
4. Selección de Puertos: Rust busca DOS puertos TCP libres al azar al iniciar (ej. 54321 para Python y 54322 para sí mismo) para evitar colisiones con otros programas.
5. Lanzamiento del Empleado (Sin LLM): Rust ejecuta en segundo plano el archivo `./engine/ai_service.exe`, pasándole los puertos y ordenando cargar SOLO el modelo de embeddings pequeño: `--embed-only`. (El LLM de Qwen NO se carga aún).
6. Verificación de Salud: Rust espera a que el servidor local de FastAPI responda "OK" en GET /health y lanza el Ping de predicciones si fue necesario en el Paso 3.
7. Se abre la interfaz de usuario para el cajero (Modo Punto de Venta normal).
8. Lazy Loading (Solo al usar IA): Cuando el usuario abre la pestaña del Chatbot, Rust primero revisa su `llm_estado`. Si está descargado, usa `sysinfo` para medir la RAM, elige el modelo y hace la petición HTTP (`POST /load_llm`).

## 2. Flujo de Comunicación (Protocolo HTTP Local)

Toda la comunicación entre el Frontend, el Backend de Rust y el Motor de IA se hace mediante peticiones locales, simulando un "internet privado" dentro de la computadora.

Origen                  | Destino                | Método     | Propósito
Frontend (Vite/TS)      | Rust (Tauri)           | IPC/invoke | Registro de ventas, Inventario, Interfaz nativa.
Rust (Axum)             | Python (FastAPI)       | REST API   | Consultas al Chatbot, Predicción de ventas.
Rust (Axum)             | Python (FastAPI)       | HTTP POST  | Ping de venta nueva: "actualiza tus gráficas".
Python (IA)             | SQLite (yarvis.db)     | SQL Directo| Solo LECTURA: historial para Prophet y RAG.
Python (FastAPI)        | Rust (Axum)            | HTTP POST  | Escritura indirecta: Python pide a Rust que inserte embeddings.

## 3. Lógica de Selección Adaptativa de IA

- Perfil A (Moderno): >= 2.5GB RAM Libre -> Carga Qwen-2.5-1.7B (Q6).
- Perfil B (Legacy): < 2.5GB RAM Libre -> Carga Qwen-2.5-0.5B (Q6).

if ram_libre >= 2.5 { cargar(1.7B_Q6) } else { cargar(0.5B_Q6) }.

IMPORTANTE: Si el motor de IA falla o la laptop es demasiado antigua para el LLM, el POS seguirá funcionando en modo "Venta Clásica", desactivando solo el Chatbot para no bloquear la caja.

## 4. Gestión de Rutas Portables

Para evitar errores al mover la USB, el sistema usará std::env::current_exe() para calcular todas las rutas relativas al ejecutable:

- DB_PATH       = <directorio del exe>/yarvis.db
- AI_ENGINE_PATH = <directorio del exe>/engine/ai_service.exe
- MODELS_PATH   = <directorio del exe>/engine/models/


## 5. Parseador de tickts mediante script

Tenemoos 1 sola forma de parsear los tickets es primero subir un archivo en txt donde se muestre la arquitectura del ticket con la opcion de que el instalador del software mapee las columnas visualmente desde la interfaz del sistema (esto para evitar errores) ademas desde desde la interfaz el modelo debe mostrar que columnas son precio que columnas son producto etc y despues el que use el sistema tiene que validar esta informacion (Ej: "Selecciona qué columna es el Producto y cuál es el Precio").

y de ahi se escribira un script para adapar la forma definitiva del ticket por naturalidad todos seran de la misma forma solo cambiarian los productos  precios descuentos y alguna que otra cosa entonces primero parseamos 1 ticket dejamos que el LLM escriba el script de como es el formato del ticket y despues parseamos los otros 11,999 de esta manera podriamos parsear los 12,000 en solo unos 15 minutos o menos el parseo de los tickets sera en la laptop personal del creador del software para usar el modelo mas alto uno de para que a la hora de parsear los tickets sea de manera segura.

## la facturacion electronica
el sistema podra conectarse a internet para poder predecir cosas o para poder hacer la facturacion electronica para intercambuar XML con un PAC 

##  NO SE USARA LA USB como ejecutable
la usb solo sera el transporte del modelo el modelo siempre sera ejecutable dentro del dispositivo. 

## API del clima e Historial

Funcionará para predecir mejor cosas con Prophet (ej. Frente Frío = Más pan). Pero la regla de oro de la predicción es: "Prophet necesita el clima histórico para hacer correlaciones". Por lo tanto, durante cada Corte Z, Rust hace una llamada a la API y guarda el clima del día en la DB.
try:
    # Intenta conectarse a internet para ver el clima y guardarlo en el historial
    clima = requests.get("https://api.clima.com/...").json()
except:
    # Si no hay internet, guarda el registro como "Desconocido"
    clima = "Clima desconocido (Offline)"
