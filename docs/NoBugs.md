# NoBugs — Respuestas del autor al review

Este archivo es la réplica oficial a los puntos del review externo. No es para esconder bugs, es para dejar por escrito por qué se decidió así.

## 1. IA local: arma, no lastre

> "Empaquetar libllama, LD_LIBRARY_PATH, NO_STRIP, modelo GGUF de 1.7B = instalador pesadísimo y laptops de tienda viejas sufriendo."

Aclaración: **no se empaquetará**.

- El modelo `.gguf` va como archivo independiente al `.exe`, no dentro del binario.
- El chatbot solo se conecta por la ruta donde esté el modelo (file path / http local).
- Ya hay soporte para poner la ruta de donde tienes el modelo descargado (`set_local_model_path` + `load_chat_model` / `unload_chat_model` en `chat.rs`).
- La caja (ventas, cortes, inventario) **nunca depende de la IA para funcionar**. Si no hay modelo, el POS cobra igual. El Qwen queda solo para el chat.

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

Es **provisional a propósito**. Cuando se importan empleados (o se dan de alta en lote), el sistema les asigna `NOMBRE123` como contraseña de arranque y prende el flag `password_defecto`. El flujo esperado es que el **admin entre después y les ponga contraseña real, sueldo, horario y turno** a cada uno — es parte del alta, no un olvido.

Por eso no se "arregla" generando claves aleatorias: si el admin no puede saber la inicial, no puede entregársela al empleado nuevo. Lo que sí está prohibido es dejarlas así para siempre: el banner de "password por defecto" le recuerda al empleado y al admin hasta que se cambie.
