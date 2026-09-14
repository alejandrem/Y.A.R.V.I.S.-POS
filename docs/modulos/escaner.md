# Escaner de codigos de barras — Y.A.R.V.I.S. POS

> Actualizado 2026-09-10. El escaner **no necesita drivers ni SDK**: un
> lector USB-HID se comporta como teclado (escribe el codigo + Enter).
> Todo el trabajo esta en normalizar y resolver con busqueda EXACTA.
> Migracion `0011_codigos_barras.sql`, modulo
> `backventanas/codigos_barras/mod.rs` (182 lineas, 4 tests).

## 1. Idea en una linea

```
Escaner USB ──teclea codigo + Enter──► input de busqueda ──► match EXACTO ──► al carrito
```

El frontend NO habla con hardware. El escaner escribe donde este el foco
(buscador de `nueva_venta.tsx`) y el backend resuelve el codigo contra
`productos.codigo_barras` con `=` (nunca `LIKE`: el cobro debe ser
determinista).

## 2. Reglas de normalizacion (las mismas en Rust y SQL)

`normalizar_codigo_barras` (`codigos_barras/mod.rs:24`):

- `None`, `""`, `"   "`, `" - "` → `None`. **Jamas se guarda `""`**:
  `"" != NULL` y colisionaria en el indice unico.
- Trim + sin espacios/guiones internos + MAYUSCULAS:
  `"  750-123 456 "` → `"750123456"`, `"unico123"` → `"UNICO123"`.
  Asi el historial matchea lo que el escaner entrega (`"750-123" == "750123"`).
- Validacion (`validar_codigo_barras`): max 64 caracteres, solo
  `ascii_graphic`. Nada de emojis ni ñ en un EAN.

La migracion `0011` aplica lo mismo a datos viejos en SQL puro:

1. Vacios → `NULL` (dos pasadas, antes y despues de normalizar).
2. `UPPER(REPLACE(REPLACE(TRIM(...))))` para historial existente.
3. Deduplicacion historica: se conserva el `MIN(id)` por codigo, el resto
   pasa a `NULL`. **No se borra ningun producto.**
4. `CREATE UNIQUE INDEX IF NOT EXISTS idx_productos_codigo_barras
   ON productos(codigo_barras) WHERE codigo_barras IS NOT NULL`
   (indice parcial: infinitos NULL conviven, cero duplicados reales).

## 3. Backend (codigos_barras/mod.rs)

| Pieza | Que hace |
|---|---|
| `normalizar_codigo_barras` / `normalizar_codigo_obligatorio` | Limpieza compartida por formularios, CSV y escaner. |
| `validar_codigo_barras` | Longitud y charset; error en lenguaje de tendero. |
| `es_error_codigo_duplicado` / `mensaje_error_codigo` | Traduce `UNIQUE constraint failed` a "El código ya está en otro producto". La unicidad la impone SQLite; aqui solo se traduce. |
| `get_product_by_barcode_impl` | Nucleo testeable: normaliza → `SELECT ... WHERE codigo_barras = ? LIMIT 1` → `Option<InventoryItem>`. `None` = no registrado (el frontend ofrece crearlo). |
| `get_product_by_barcode` | Comando Tauri (`lib.rs`), con `auth.require_operator()` (admin o empleado). Dinero convertido via `dinero::a_pesos` (centavos → pesos) como el resto del IPC. |

Tests (4, en el mismo archivo): vacios → None, espacios/guiones/mayusculas,
longitud+charset, deteccion de duplicado por mensaje SQLite.

## 4. Frontend (estado real, sin adornos)

- `nueva_venta.tsx:searchProducts`: el input filtra el inventario **ya
  cargado** por nombre, `codigo_barras` o categoria (top 8). Si no hay match
  y el texto tiene 3+ letras, cae a `buscarProductoSimilar` (IA). El Enter
  del escaner dispara este mismo flujo: con foco en el buscador, escanear
  es indistinguible de teclear rapido. Debounce 200ms.
- `buscador-productos.tsx:122`: la fila muestra icono de barras si el
  producto tiene codigo, bolsa si no.
- `services/inventario.ts`: `InventoryItem.codigo_barras?` viaja en el tipo
  compartido; el CRUD lo guarda normalizado desde el backend.
- Honestidad documental: el comando exacto `get_product_by_barcode`
  existe, esta registrado y testeado en su nucleo, pero la pantalla de
  venta hoy resuelve con el filtro local (cero latencia IPC por pitido).
  Cablear el comando como resolucion primaria es mejora pendiente menor,
  no bug: ambos usan la misma normalizacion.

## 5. Como probar con un lector real

1. Conectar cualquier lector USB-HID (modo teclado, el default de fabrica;
   programarlo con sus codigos de configuracion si viene en modo serie).
2. Abrir Nueva venta, foco en el buscador, escanear un producto con codigo
   cargado → debe quedar primero en el dropdown al instante.
3. Escanear un codigo inexistente → "sin resultados" (y la sugerencia IA
   solo si el texto parece nombre, no numeros puros).
4. Casos borde: etiqueta con guiones (`750-123` matchea `750123`), codigo
   duplicado en dos productos (el backend lo impide con mensaje claro),
   producto sin codigo (vende por nombre, sin drama).

## 6. Decisiones y lecciones

- `=` en vez de `LIKE`: un pitido = un producto. La busqueda difusa es
  para nombres tecleados, jamas para el escaner.
- `NULL` en vez de `""`: el que guardo `""` un dia rompio el indice unico
  para todos (BUG clasico; la 0011 lo purga).
- Mayusculas a proposito: EAN es numerico y Code128 queda case-insensitive
  para que el escaneo siempre matchee, venga como venga.
- El escaner no es un "modulo": son 3 capas delgadas (migracion + 1 archivo
  Rust + filtro existente). A proposito, para no crear una Fase solo para
  un pitido.
