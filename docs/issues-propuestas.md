# Issues propuestas — listas para crear en GitHub

> Nota: no hay `gh` en este Windows, así que se dejan redactadas aquí para pegarlas en GitHub UI. Una sección = una issue.

---

## ISSUE 1: Tools de lectura + SQL para modelos cloud (empleado vs admin)

**Título:** `feat(ia): tools de lectura + SQL solo-lectura para modelos cloud con roles`

**Cuerpo:**
Darle a los modelos cloud (Zen/Gemini) capacidad de responder: ¿cuánto vendí hoy?, ¿cuánto venderé mañana?, ¿qué productos me generan más?, ¿cuáles vendo más/menos?

Propuesta:
- Enseñarles SQL en vez de un tool por pregunta: pasarles snapshot del schema + ejemplos few-shot (ventas por día, top productos por margen, rotación).
- Un solo tool `sql_readonly`: valida que la query sea `SELECT`/`WITH` únicamente (bloquear INSERT/UPDATE/DELETE/DDL/PRAGMA write), exige `LIMIT`, tope de filas y timeout. Todo parametrizado, sin concatenar input del usuario.
- `pronostico_ventas` separado (Holt-Winters) para "cuánto venderé mañana", no dejar que el modelo invente números.
- Roles: el backend ya tiene `herramientas_rol.rs`. Filtrar tools por rol + system prompt distinto: empleado solo ve agregados y sus propias ventas; admin ve todo. El modelo nunca decide el rol, lo impone el backend por sesión.
- Prohibido: cualquier tool que edite la DB (altas/bajas de productos, etc.).
- Loguear cada query ejecutada para auditoría.

---

## ISSUE 2: Atajo F2 — reimprimir ticket por número (modal)

**Título:** `feat(caja): atajo F2 reimprime ticket por número`

**Cuerpo:**
Idea: F2 abre una modal donde se coloca el número del ticket y se reimprime en la térmica (Camino A/C según impresora configurada). Debe validar que el ticket exista, mostrar mini-resumen antes de imprimir y errores claros (sin impresora, sin papel).

> ⚠️ Si eres un agente de código: antes de programar, pregúntale al usuario exactamente cómo quiere que se vea y funcione este atajo (layout de la modal, validaciones, qué impresora usar por defecto). No asumas, para que no haya problemas.

---

## ISSUE 3: Atajo F3 — corte de caja X/Z (modal)

**Título:** `feat(caja): atajo F3 abre modal de corte de caja X y Z`

**Cuerpo:**
Idea: F3 abre una modal para hacer corte de caja, eligiendo X (informativo, no cierra) o Z (cierre definitivo). Debe mostrar ventas del turno, movimientos, diferencia físico vs sistema y pedir confirmación explícita para Z.

> ⚠️ Si eres un agente de código: antes de programar, pregúntale al usuario exactamente cómo quiere que se vea y funcione este atajo (campos de la modal, flujo X vs Z, confirmaciones). No asumas, para que no haya problemas.

---

## ISSUE 4: Atajo F4 — pagar al proveedor (modal directa)

**Título:** `feat(caja): atajo F4 abre modal de pago al proveedor`

**Cuerpo:**
Idea: F4 abre directo la modal de la tarjeta "Pagar al proveedor — Directo a compra y pago, sin configurar nada" (tarjeta negra con `$`). Flujo corto: elegir proveedor, monto, método, registrar compra + pago juntos.

> ⚠️ Si eres un agente de código: antes de programar, pregúntale al usuario exactamente cómo quiere que se vea y funcione este atajo (qué campos precargar, si permite proveedor genérico, confirmaciones). No asumas, para que no haya problemas.

---

## ISSUE 5: Atajo F5 — cobrar (modal de cobranza)

**Título:** `feat(caja): atajo F5 abre modal de cobranza`

**Cuerpo:**
Idea: F5 abre la modal del botón "Cobrar $0.00", donde se coloca la cantidad con la que están pagando. Debe calcular cambio, soportar métodos de pago, validar monto >= total y cerrar la venta al confirmar.

> ⚠️ Si eres un agente de código: antes de programar, pregúntale al usuario exactamente cómo quiere que se vea y funcione este atajo (layout, métodos de pago, atajos internos como Enter/Esc). No asumas, para que no haya problemas.

---

## ISSUE 6: Atajo F6 — abrir caja

**Título:** `feat(caja): atajo F6 abre la caja`

**Cuerpo:**
Idea: F6 solo abre la caja (pulso de apertura a la térmica / comando ESC/POS). Probablemente el más difícil porque depende del driver, permisos del spooler Windows y cada modelo de cajón. Debe funcionar sin bloquear la venta y reportar error si no hay cajón conectado.

> ⚠️ Si eres un agente de código: antes de programar, pregúntale al usuario exactamente cómo quiere que se vea y funcione este atajo (qué hardware tiene, comportamiento si falla, si pide confirmación). No asumas, para que no haya problemas.

---

## ISSUE 7: Atajo F8 — menú de ayuda de atajos

**Título:** `feat(caja): atajo F8 abre menú de ayuda de atajos`

**Cuerpo:**
Idea (la más entretenida): F8 abre un menú que lista qué hace cada atajo de forma abreviada pero entendible (F2 reimprimir, F3 cortes, F4 proveedor, F5 cobrar, F6 caja, F8 este menú). Debe generarse desde una sola tabla de atajos para no desincronizarse cuando se agreguen más.

> ⚠️ Si eres un agente de código: antes de programar, pregúntale al usuario exactamente cómo quiere que se vea y funcione este atajo (diseño del menú, si es modal o drawer, si permite hacer clic para ejecutar). No asumas, para que no haya problemas.

---

## ISSUE 8: Animación del libro DatosInutiles + mover su backend a admin

**Título:** `feat(manual): animación de página del libro DatosInutiles y mover backend a admin`

**Cuerpo:**
El libro `DatosInutiles` (manual de usuario) cambia de página todo plano, sin animación, y se ve robótico/feo. Agregar animación de paso de página (flip/slide suave) para que se sienta como libro. Además mover su backend de empleado a administrador (solo admin lo gestiona; el empleado como mucho lo lee, según defina el dueño).

---

## ISSUE 9: Bug sospechoso en horarios del empleado + más filtros

**Título:** `bug(empleados): horarios se estarían registrando de forma errónea; agregar filtros`

**Cuerpo:**
Sospecha del dueño: la sección de horarios del empleado estaría registrando de manera errónea (turnos/bloques que no cuadran). Revisar: creación de bloques, solapamientos, zona horaria, días asignados a más de un bloque y qué se guarda vs qué se muestra. Agregar más filtros (por empleado, día, semana, estado) para poder auditarlo. Incluir test de regresión con 6 bloques donde cada día pertenezca a un solo bloque.
