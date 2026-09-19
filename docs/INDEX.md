# Documentación Oficial — Y.A.R.V.I.S. POS

Bienvenido al centro de documentación técnica y operativa de **Y.A.R.V.I.S. POS** (*Yet Another Really Versatile Intelligent System*). Aquí encontrarás toda la arquitectura, especificaciones de hardware, lógica de negocio, guías operativas e historial de desarrollo del sistema.

---

## Mapa Rápido de Documentación

| Sección | Descripción | Documentos Clave |
|---|---|---|
| 🏗️ [**Arquitectura**](#1-arquitectura-y-stack-tecnológico) | Diseño general, stack validado y protocolos IPC | [¿Qué es Yarvis?](arquitectura/que-es-yarvis.md), [Arquitectura](arquitectura/arquitectura.md), [Tecnologías](arquitectura/tecnologias.md), [Interconexión](arquitectura/interconexion.md) |
| ⚙️ [**Módulos y Hardware**](#2-módulos-técnicos-y-hardware) | Periféricos (ESC/POS, HID), parseador y dinero | [Parseador](modulos/parseador.md), [Impresora](modulos/impresora.md), [Escáner](modulos/escaner.md), [Lógica de Proceso](modulos/logica-proceso.md) |
| 📜 [**Historial y Auditoría**](#3-historial-y-auditorías) | Olas de implementación, bugs y seguridad | [Implementación](historial/implementacion.md), [Bugs Resueltos](historial/bugs-resueltos.md), [Migración Rust](historial/migracion-rust.md), [Refactor y Seguridad](historial/refactor-seguridad.md) |
| 💼 [**Operación y Negocio**](#4-operación-y-negocio) | Comandos de desarrollo, build y propuesta comercial | [Comandos](negocio/comandos.md), [Propuesta Comercial](negocio/idea-comercial.md) |

---

## 1. Arquitectura y Stack Tecnológico

Documentos sobre el diseño del sistema, estructura de código y comunicación entre capas:

- 📘 [**¿Qué es Y.A.R.V.I.S.?**](arquitectura/que-es-yarvis.md):
  Visión general del sistema de punto de venta inteligente. Principios offline-first, ausencia de sidecars externos y arquitectura híbrida de IA.
- 📐 [**Arquitectura de Software**](arquitectura/arquitectura.md):
  Estructura de directorios del monorepo, ciclo de vida de la aplicación, comunicación IPC Tauri v2 y árbol de componentes.
- 💻 [**Tecnologías y Stack Técnico**](arquitectura/tecnologias.md):
  Stack 100% verificado: Tauri v2, Rust (2021), React 18, Vite, TypeScript, Tailwind CSS, SQLite con WAL y motor de chat híbrido (Gemini / NVIDIA / Qwen local).
- 🔌 [**Interconexión y Comunicación**](arquitectura/interconexion.md):
  Protocolo de interacción entre la interfaz de usuario en React y el backend en Rust mediante comandos `invoke()`, eventos y contratos de datos.

---

## 2. Módulos Técnicos y Hardware

Especificaciones de los subsistemas del POS, soporte para periféricos y reglas de negocio:

- 🧾 [**Parseador Masivo de Tickets y Catálogos**](modulos/parseador.md):
  Motor de ingesta de datos con detección heurística de formatos (A/B/C), segmentación precisa de tickets, idempotencia y procesamiento sin necesidad de LLM.
- 🖨️ [**Impresora Térmica ESC/POS**](modulos/impresora.md):
  Driver de impresión térmica nativo: Spooler Windows RAW (Camino A), tickets ESC/POS con soporte de QR y conexión de red TCP 9100 (Camino C).
- 🔍 [**Escáner de Códigos de Barras**](modulos/escaner.md):
  Integración con lectores de código de barras HID en modo emulación de teclado, normalización de lecturas, índice único 0011 y búsqueda optimizada.
- 💰 [**Lógica de Proceso y Transacciones**](modulos/logica-proceso.md):
  Reglas del dominio: manejo de dinero en centavos (`INTEGER`), flujo de ventas, cortes de caja X y Z, arqueos e integridad contable.

---

## 3. Historial y Auditorías

Bitácoras técnicas, seguimiento de hitos y lecciones aprendidas durante el desarrollo:

- 🚀 [**Plan y Estado de Implementación**](historial/implementacion.md):
  Mapa de ruta por fases ("Olas" 1 a 5) que documenta qué características están completadas y qué queda planificado.
- 🐛 [**Bitácora de Bugs Resueltos**](historial/bugs-resueltos.md):
  Registro detallado de problemas encontrados, análisis de causa raíz y soluciones definitivas aplicadas en producción.
- 🦀 [**Historia de la Migración a Rust**](historial/migracion-rust.md):
  Crónica de la transición de un backend inicial en Python hacia una solución 100% Rust nativa, reduciendo consumo de memoria y eliminando dependencias.
- 🛡️ [**Refactor y Auditoría de Seguridad**](historial/refactor-seguridad.md):
  Auditoría senior de código, políticas de seguridad con Argon2, hardening de bases de datos y roles de usuario.

---

## 4. Operación y Negocio

Guías para desarrolladores, despliegue y visión de producto:

- ⚡ [**Guía de Comandos Útiles**](negocio/comandos.md):
  Comandos para compilación, ejecución en desarrollo, empaquetado (.deb, .rpm, .AppImage, .exe) y resolución de problemas frecuentes.
- 🎯 [**Propuesta de Valor Comercial**](negocio/idea-comercial.md):
  Pitch de ventas, diferenciales competitivos en el mercado de puntos de venta y argumentos clave para clientes.
