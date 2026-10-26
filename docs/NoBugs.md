# NoBugs — Respuestas del autor al review

Este archivo es la réplica oficial a los puntos del review externo. No es para esconder bugs, es para dejar por escrito por qué se decidió así.

## 1. IA local: arma, no lastre

> "Empaquetar libllama, LD_LIBRARY_PATH, NO_STRIP, modelo GGUF de 1.5B = instalador pesadísimo y laptops de tienda viejas sufriendo."

Aclaración: **no se empaquetará**.

- El modelo `.gguf` va como archivo independiente al `.exe`, no dentro del binario.
- El chatbot solo se conecta por la ruta donde esté el modelo (file path / http local).
- Ya hay soporte para poner la ruta de donde tienes el modelo descargado (`set_local_model_path` + `load_chat_model` / `unload_chat_model` en `chat.rs`).
- La caja (ventas, cortes, inventario) **nunca depende de la IA para funcionar**. Si no hay modelo, el POS cobra igual. El Qwen2.5-Coder 1.5B queda solo para el chat.

## 2. `DatosInutiles` no es deuda, es el manual

> "backventanas, carpeta DatosInutiles"

`DatosInutiles` es el **manual de usuario**, no código muerto ni otra cosa. Se renombró de `datos inutiles` a `DatosInutiles` sin espacios en el commit `9ff0b80` solo por el path.

Pendiente: renombrarlo a `ManualUsuario` para que no confunda a agentes externos.

## 3. ¿Qué es "deuda de naming"?

Es cuando los nombres históricos dificultan leer el proyecto para alguien nuevo:

- `backventanas` viene de la primera estructura y hoy ya no son "ventanas", son dominios (`backadmin/`, `backempleado/`, `codigos_barras/`, `impresora/`, `db/`).
- No rompe nada, pero espanta a contribuidores y a los agentes de IA que hacen review.

Plan: no renombrar ahorita para no romper 137 invokes. Se renombra después del primer `.exe` estable.

## 4. Scope gigante

> "POS + inventario + finanzas X/Z + empleados + proveedores + parser + impresora + facturación + chat local + cloud + predicciones. Mucho para 1 sola persona."

Respuesta del autor: es algo normal para mí. Es un reto personal y no será imposible para mí. Prefiero un sistema completo para tienda real que 5 mini-proyectos a medias.

Estrategia igual válida: congelar IA/predicciones hasta tener v1.0 que cobre e imprima.

## 5. Prueba de fuego

> "¿Ya cobra en una tienda real con 12k tickets, impresora térmica china y Windows sin internet?"

Eso es real, aún no. Pero primero necesitamos el primer `.exe`. Sin instalador no hay tienda piloto. Prioridad actual: empaquetar, probar en Windows limpio, luego piloto real.

## 6. Contraseñas provisionales `NOMBRE123`: intencional, no bug

> "Empleados auto-creados con contraseña predecible + login solo-password: cualquiera que sepa el nombre del cajero entra como él."

Es **provisional a propósito** y solo para un caso: empleados **auto-detectados al importar tickets** (el sistema no puede adivinar su contraseña real). Esos reciben `NOMBRE123` como arranque y prenden el flag `password_defecto`. Las altas manuales (PrimerInicio y modal de Empleados) ya piden **contraseña real + horarios + pago semanal en un solo paso**, así que nunca pasan por `NOMBRE123`.

Por eso no se "arregla" generando claves aleatorias: si el admin no puede saber la inicial, no puede entregársela al empleado nuevo. Lo que sí está prohibido es dejarlas así para siempre: el banner de "password por defecto" **ya no se puede ocultar** (EmployeeDashboard lo muestra en cada login) hasta que el admin la cambie en Empleados, momento en que `editar_empleado` apaga el flag.

---

### 7 CI — DIFERIDO A PROPÓSITO LO ACTUALIZAREMOS CONFORME CREZCA EL PROYEXTO 

Prioridad actual: primer `.exe` estable y primer cliente piloto. CI no bloquea la venta; la disciplina de release sí. Mientras tanto:

- Checklist manual antes de cada instalación: build limpio, tests, venta, corte X/Z, impresión, backup/restore, login/logout y modo offline.
- Tag por cliente (`v0.1.0-piloto-1`) y changelog.
- Arreglar la prueba vieja de `empleados` (espera 90 min de extra previo; el código da 75 por la cortesía de 15). No es bug de producto, pero `cargo test` debe quedar verde antes del primer cliente.
- CI (GitHub Actions + instalador firmado + auto-update) cuando haya 3+ clientes o actualizaciones remotas.

---

### 8. Login solo contraseña (empleados): diseño intencional, no descuido

> "Empleados: SELECT * + iterar hashes Argon2. Único factor = contraseña. Colisión si dos comparten clave."

**Diseño deliberado para flujo de caja rápido:**

1. **El login es "solo contraseña" a propósito**: en una tienda con rotación alta y turnos cortos, el empleado llega, teclea su clave y vende. No hay tiempo para "usuario + contraseña" ni 2FA. El threat model es **físico** (la caja está en el mostrador, solo el cajero de turno la toca), no remoto.

2. **Colisión imposible por construcción**: `guardar_empleado` (y `guardar_empleado_impl`) **rechaza la alta** si la contraseña elegida coincide con el hash de *cualquier* empleado activo (`verify_password` contra todos). Dos empleados **no pueden** compartir clave. El error lo ve el admin al dar de alta: "Ya existe un empleado con esa contraseña. Debe ser distinta para poder identificarlo en el login."

3. **Rate-limiting mitiga fuerza bruta offline**: `LoginRateLimiter` (5 intentos / 5 min, ventana deslizante, en memoria, se reinicia en éxito). Un atacante con acceso físico a la caja ya tiene la llave del local; el límite ralentiza prueba-y-error pero no es el perímetro principal.

4. **Sesión en Rust, no en React**: `AuthState` guarda `Session { user_id, role, name }` en `Mutex<Option<Session>>` dentro del proceso Tauri. El frontend **no** manda rol ni user_id en cada comando; el backend valida `require_operator()` / `require_admin()` contra esa sesión. Un XSS en el webview no escala privilegios.

5. **Rastreabilidad por `cajero_id`**: cada venta persiste `cajero_id` (FK a `usuarios.id`), no solo el nombre. Si el admin renombra al empleado después, el histórico sigue íntegro.

**Qué NO cubre (y por qué se acepta):**
- Robo de contraseña por hombro (shoulder surfing) → mitigación: contraseñas alfanuméricas mínimas 6 chars, sin políticas de rotación forzada (fatiga = contraseñas peores).
- Keylogger en la PC → mitigación: Windows Defender + PC dedicada a caja sin navegador.
- Insider threat (admin malicioso) → mitigación: logs de auditoría (`ultimo_login`, `ventas.cajero_id`), respaldos cifrados inmutables.

**Futuro (post v1.0)**: si un cliente pide 2FA o login con usuario+pass, se añade columna `username` UNIQUE y se cambia el flujo; el contrato Tauri (`validar_login_empleado`) admite parámetros extra sin breaking change.
