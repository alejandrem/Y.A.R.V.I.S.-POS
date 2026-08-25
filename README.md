<div align="center">

<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
  <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10"/>
  <path d="M6 9h12M6 12.5h12M9 9v3.5a3 3 0 0 0 6 0V9"/>
</svg>

# Y.A.R.V.I.S. POS

**Tu asistente local de punto de venta** — un binario único de escritorio con inteligencia artificial nativa.

<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
React &nbsp;·&nbsp; Rust &nbsp;·&nbsp; Tauri 2 &nbsp;·&nbsp; SQLite &nbsp;·&nbsp; Qwen 1.7B local + Opencode zen.

</div>

---

## Qué es Y.A.R.V.I.S.

Un sistema de punto de venta de escritorio para tiendas medianas y pequeñas, con IA que corre **localmente** (y en nube con fallback). Cobra como una caja registradora, pero además **parsea tickets viejos, sugiere compras, avisa anomalías y conversa** con el dueño.

<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg> Caja registradora (POS) que nunca depende de la IA para funcionar.

<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg> Importación inteligente: sube tus 12,000 tickets en TXT/CSV/Excel y YARVIS aprende de tu historia.

<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg> Chat que responde "¿cuánto gané hoy?" o "¿qué debería comprar para el fin de semana?".

<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg> Portable y auto-sostenible: un ejecutable, una base de datos, cero servicios externos obligatorios.

---

## Estado del proyecto

<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg> **Binario único** — Tauri v2 empaqueta frontend + backend + motor IA juntos.

<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg> **97 comandos Tauri** registrados en 21 módulos de backend.

<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg> **Parseador 100% Rust** — reglas + LLM local bajo demanda.

<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg> **Dinero en centavos enteros** — toda columna monetaria es INTEGER (migración 0005); cero errores de redondeo flotante.

<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg> **Seguridad endurecida** — SQL de tools parametrizado, roles enforced en la ejecución de tools, API keys fuera del webview (archivo 0600), CSP activa, sin fallback de contraseñas en texto plano.

<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg> **Chat cloud (OpenCode Zen/Gemini) con fallback local (Qwen 3 1.7B)**.

<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg> **Python eliminado** — migración completa documentada en `idea.md`.

---

## Características

| | Módulo | Qué hace |
|---|---|---|
| <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l1.5-5h15L21 9M4 9v11h16V9M3 9h18M9 20v-6h6v6"/></svg> | **Punto de venta** | Cobro con escáner o búsqueda, métodos de pago, descuentos, ticket de venta. |
| <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg> | **Inventario** | CRUD de productos, alertas de stock bajo, importación de catálogos. |
| <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg> | **Finanzas** | Cortes de caja X/Z, gastos recurrentes, alertas, métricas y exportación. |
| <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M19 8v6M22 11h-6"/></svg> | **Empleados** | Metas y bonos, turnos, salario, resumen de ventas por operador. |
| <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg> | **Importación de tickets** | Mapeo de columnas con IA, procesamiento por lotes con streaming, vinculación al inventario. |
| <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg> | **Asistente Y.A.R.V.I.S.** | Chat con IA local/cloud: ventas, inventario, detección de anomalías y consejos. |
| <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg> | **Configuración** | Identidad de tienda, seguridad (admin/empleado), temas claro/oscuro. |

---

## Stack tecnológico

**Frontend** — <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg> React 19 · TypeScript 5.8 · Vite 7 · Tailwind CSS 3 · Recharts · morphicons · react-markdown

**Backend** — <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg> Tauri 2.11 · sqlx 0.8 · Tokio · Serde · reqwest · Argon2 · chrono

**IA** — <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><path d="M9 1v3M15 1v3M9 20v3M15 20v3M1 9h3M1 15h3M20 9h3M20 15h3"/></svg> `src-ia` (crate Rust): Qwen 3 1.7B GGUF vía llama.cpp (local) · OpenCode Zen / Gemini vía SSE (cloud) con relevo 429 · **10 tools de consulta** (ventas, inventario y navegación de catálogo, solo lectura)

**Datos** — <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg> SQLite (WAL) con un solo escritor (Rust) · `yarvis.db`

---

## Estructura del repositorio

```text
├── src-ia/                  # Motor de IA en Rust (crate local)
│   ├── parseador_de_tickets/  # Reglas + lectores + análisis LLM
│   ├── motor-chat/            # Chat cloud (SSE) y local (llama.cpp)
│   └── tests/                 # Estrés del parseador, verificación de modelos
├── yarvis-app/
│   ├── src/                   # Frontend React
│   │   ├── front-admin/       #   Panel del administrador
│   │   ├── front-empleado/    #   Punto de venta del operador
│   │   └── hooks/             #   ThemeContext, ParserContext
│   └── src-tauri/             # Backend Rust + configuración Tauri
│       └── src/backventanas/  #   ~91 comandos (backadmin/ y backempleado/)
├── idea.md/                   # Documentación completa (arquitectura, stack,
│                              #   parseador, migración, bugs)
├── run.sh / run.bat           # Lanzadores (dev)
└── reset.sh                   # Borra la DB y caches
```

---

## Primeros pasos

**Requisitos:** Node.js + npm, Rust (`rustup`) y las dependencias del sistema para Tauri.

```bash
./run.sh            # Linux: instala deps si faltan y arranca `npm run tauri dev`
.\run.bat           # Windows

# dentro de yarvis-app/
./build.sh          # empaqueta binario + .deb + .rpm + .AppImage
                    # (exporta LD_LIBRARY_PATH para libllama y NO_STRIP)
```

La primera ejecución muestra el **asistente de primer inicio** (alta de administrador, tienda y empleado). Después solo aparece el login por rol.

---

## Arquitectura en una línea

```text
Frontend (React) ──invoke──► Backend Rust (Tauri) ──► SQLite (WAL)
                                 │
                                 └─► src-ia (en proceso): chat nube/local + parseador
```

Sin HTTP local, sin puertos libres, sin procesos externos. La IA vive dentro del ejecutable.

---

## Documentación

Toda la documentación vive en `idea.md/`:

| Documento | Contenido |
|---|---|
| `opencode/arquitectura.md` | Árbol del proyecto y diagrama de comunicación |
| `opencode/tecnologias.md` | Stack verificado y decisiones técnicas |
| `imple & docu/implementacion.md` | Estado por fases (completadas y pendientes) |
| `imple & docu/que es yarvis?.md` | Documentación completa de implementación |
| `imple & docu/PARSEADOR.md` | El módulo de parseo a detalle |
| `imple & docu/migracion_rust.md` | Historia de la migración Python → Rust |
| `imple & docu/Bugs resueltos uwu.md` | Bitácora de bugs y lecciones aprendidas |

---

## Roadmap

- <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/></svg> Predicciones de ventas nativas (Holt-Winters) con intervalos de confianza.

- <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/></svg> Búsqueda semántica hecha con TF-IDF + fuzzy (más ligero) para inventario y nueva venta.

- <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/></svg> Impresión térmica ESC/POS y facturación electrónica (XML/PAC).

- <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/></svg> Atajos de teclado de caja (F5 cobrar, F6 caja, F7 buscar, F8 cliente).

---

<div align="center">

<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9.937 15.5A2 2 0 0 0 8.5 14.063l-6.135-1.582a.5.5 0 0 1 0-.962L8.5 9.936A2 2 0 0 0 9.937 8.5l1.582-6.135a.5.5 0 0 1 .963 0L14.063 8.5A2 2 0 0 0 15.5 9.937l6.135 1.581a.5.5 0 0 1 0 .964L15.5 14.063a2 2 0 0 0-1.437 1.437l-1.582 6.135a.5.5 0 0 1-.963 0z"/></svg>
**Y.A.R.V.I.S. — Yet Another Really Versatile Intelligent System**

*Un ejecutable. Una base de datos. Tu tienda, más inteligente.*

</div>
