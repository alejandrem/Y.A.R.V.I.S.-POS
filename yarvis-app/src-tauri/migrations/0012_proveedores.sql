-- ═══════════════════════════════════════════════════════════════════
-- 0012: Proveedores y compras a proveedor (recepción de mercancía).
--
-- El módulo CLIENTES (shoppers) se retira: sus vistas eran placeholders
-- sin backend y la tabla `clientes` queda legacy sin uso (NO se borra:
-- las migraciones solo avanzan; si alguna DB tuviera filas manuales,
-- ahí siguen). El flujo real es con proveedores.
--
-- DILEMA RESUELTO (pago a proveedor no registrado): es imposible a
-- nivel disco — `compras.proveedor_id` es NOT NULL con FK. El flujo
-- normal exige proveedor existente (alta previa o "nuevo" inline en
-- el modal de pago: primero INSERT proveedor, luego la compra, todo
-- en una transacción). Si llega un id inexistente, la FK lo rechaza.
--
-- Dinero en INTEGER centavos (regla de oro); cantidades en REAL.
-- `producto_id` nace NULLABLE (lección de detalle_ventas): el vínculo
-- con el catálogo se rellena en fase 2 con más filtros.
-- ═══════════════════════════════════════════════════════════════════

-- Quién me vende: nombre forzoso (lo único que el empleado pide
-- siempre), teléfono y correo opcionales. El CHECK tumba el nombre
-- vacío a nivel disco (el backend también lo valida para el mensaje
-- amigable, pero la DB no confía en nadie).
CREATE TABLE IF NOT EXISTS proveedores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL CHECK(LENGTH(TRIM(nombre)) > 0),
    telefono TEXT,
    correo TEXT,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Sin duplicados silenciosos: "Coca" y "coca " son el mismo proveedor.
CREATE UNIQUE INDEX IF NOT EXISTS idx_proveedores_nombre
    ON proveedores(nombre COLLATE NOCASE);

-- La "factura": lo que se recibió y lo que se pagó en esa recepción.
-- monto_pagado lo escribe el empleado; monto_sugerido es la
-- recomendación (cantidad × costo de inventario, 0 = sin recomendación).
CREATE TABLE IF NOT EXISTS compras (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proveedor_id INTEGER NOT NULL REFERENCES proveedores(id) ON DELETE RESTRICT,
    fecha DATETIME DEFAULT CURRENT_TIMESTAMP,
    monto_pagado INTEGER DEFAULT 0,
    monto_sugerido INTEGER DEFAULT 0,
    metodo_pago TEXT DEFAULT 'efectivo',
    comentario TEXT,
    cajero_id INTEGER,
    creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_compras_proveedor ON compras(proveedor_id);
CREATE INDEX IF NOT EXISTS idx_compras_fecha ON compras(fecha);

-- Renglones de la recepción: presentación 'unidad' | 'paquete'
-- (etiqueta v1 con CHECK para que no entre basura; el factor
-- piezas-por-paquete llega en fase 2).
CREATE TABLE IF NOT EXISTS compras_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    compra_id INTEGER NOT NULL REFERENCES compras(id) ON DELETE CASCADE,
    producto_id INTEGER,
    nombre TEXT NOT NULL,
    presentacion TEXT DEFAULT 'unidad' CHECK(presentacion IN ('unidad', 'paquete')),
    cantidad REAL NOT NULL,
    precio_sugerido INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_compras_items_compra
    ON compras_items(compra_id);
