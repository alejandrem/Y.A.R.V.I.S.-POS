# Impresion termica ESC/POS — Y.A.R.V.I.S. POS

> Actualizado 2026-09-10: **IMPRESION TERMICA IMPLEMENTADA (Fases 1 y 2)**.
> Todo vive en `yarvis-app/src-tauri/src/impresora/` (carpeta dedicada,
> regla ~400 lineas por archivo). La caja nunca depende de la impresora:
> si esta apagada o sin papel, la venta se guarda igual y el error llega
> como toast legible.
>
> Pendiente de campo: verificacion con termica real conectada (ver §7).

## Resumen

Dos caminos combinados, misma carpeta:

- **Camino A Fase 1 — spooler de Windows con bytes RAW.** La impresora ya
  instalada con su driver; el backend abre el spooler por WinAPI
  (`OpenPrinterW` + `WritePrinter`, datatype `RAW`) y le manda los bytes
  ESC/POS tal cual. Cero config de hardware. Patron de referencia:
  `a-eid/tauri-pos-printer` (Windows escribe directo al share).
- **Camino C Fase 2 — crate `escpos` en el backend.** El ticket de venta
  enriquecido (formato, QR, totales) se arma con `escpos 0.20`
  (fabienbellanger, MIT) sobre un driver en memoria y los bytes viajan por
  el spooler RAW de Fase 1 **o** por TCP directo (puerto 9100 tipico).

## 1. Estructura (yarvis-app/src-tauri/src/impresora/)

```
impresora/
├── mod.rs              # Declara modulos + re-exporta spooler segun OS.
├── builder.rs          # Fase 1: conciliacion 80mm/48cols, puro Rust sin deps.
│                       # INIT, encabezado, tabla FIS/SIS/DIF/ESTADO, totales,
│                       # corte total (1D 56 00). Tildes transliteradas a ASCII.
├── memoria.rs          # Fase 2: driver `escpos` que captura a memoria (Arc+Mutex).
├── ticket.rs           # Fase 2: ticket de venta via `escpos` (QR Model2, corte
│                       # parcial 1D 56 41 00). Tope 500 lineas, 512KB.
├── spooler_windows.rs  # Solo Windows: EnumPrintersW + OpenPrinterW/WritePrinter.
├── spooler_stub.rs     # Fuera de Windows: error claro ("solo Windows en Fase 1").
└── commands.rs         # 5 comandos Tauri (ver §3). I/O en spawn_blocking.
```

Dependencias (`yarvis-app/src-tauri/Cargo.toml`):

```toml
escpos = "0.20"   # [dependencies], multiplataforma. Defaults: std + barcodes + codes_2d.
[windows] windows = { version = "0.58", features = ["Win32_Graphics_Printing", "Win32_Graphics_Gdi", "Win32_Security", "Win32_Foundation"] }
```

Sin features USB/serie/hidapi a proposito: el envio USB/serie instalado en
Windows ya sale por el spooler, y el codigo USB directo quedaria sin
verificar sin hardware fisico. `escpos` es MIT (compatible con GPL-3.0).

## 2. Camino A Fase 1 — spooler RAW (detalle)

`spooler_windows.rs`:

1. **Listar:** `EnumPrintersW` (doble llamada: sondeo de tamaño + buffer)
   con flags `LOCAL | CONNECTIONS`, nivel 2 (`PRINTER_INFO_2W`). La default
   (via `GetDefaultPrinterW`) va primero para preseleccion.
2. **Escribir:** `OpenPrinterW(nombre, RAW, PRINTER_ACCESS_USE)` →
   `StartDocPrinterW` (job id 0 = rechazo) → `StartPagePrinter` →
   `WritePrinter` (verifica bytes escritos == bytes totales) →
   `EndPage/EndDoc/ClosePrinter` siempre, aun en fallo.
3. Topes Fase 1: 2000 filas por lista, 512KB por trabajo.

`builder.rs` genera el ticket de conciliacion fisico-vs-sistema: tienda,
fecha, columnas `NOMBRE(20) FIS SIS DIF ESTADO`, conteo de faltantes/
sobrantes y perdida estimada (`|dif| x precio_venta`), doble salto + corte.

## 3. Comandos Tauri (lib.rs)

| Comando | Args (JS → Rust) | Que hace |
|---|---|---|
| `listar_impresoras` | — | Impresoras del spooler (`{nombre, predeterminada}`). |
| `imprimir_bytes_raw` | `nombreImpresora`, `bytes` | Manda bytes ya armados (puente a Fase 2). |
| `imprimir_lista_conciliacion` | `nombreImpresora`, `tienda?`, `filas[]` | Arma conciliacion en Rust y la manda RAW. |
| `imprimir_ticket_venta` | `destino`, `ticket` | Render `escpos` → Spooler o Red (§4). |
| `probar_red` | `ip`, `puerto` | Solo TCP connect + close. Sin gastar papel. |

Nota IPC: Tauri convierte `camelCase` (JS) → `snake_case` (Rust) en
argumentos por defecto (verificado en `tauri-macros` `wrapper.rs`,
`ArgumentCase::Camel`). Por eso el frontend manda `nombreImpresora` y el
backend recibe `nombre_impresora` sin `rename_all`.

## 4. Camino C Fase 2 — ticket de venta (detalle)

`ticket.rs` + `memoria.rs`:

1. El frontend manda `TicketVentaPayload` (pesos en f64, como el resto del
   contrato IPC; el formateo vive en Rust): tienda, ubicacion?, folio,
   fecha? (= ahora si viene vacia), lineas `{nombre, cantidad, precio}`,
   total, pagos `[{metodo, monto}]`, qr?.
2. El backend valida (lineas no vacias, total finito ≥ 0, tienda y folio
   obligatorios), calcula `cambio = max(0, pagado − total)` y renderiza con
   `Printer::new(MemoriaDriver, Protocol::default(), PrinterOptions)`:
   tienda 2x centrada negrita → folio/fecha → separadores → lineas
   monoespaciadas 48 cols (`2x Nombre ... $importe`) → TOTAL 2x → pagos →
   CAMBIO → feed → **QR del folio** (`Model2, tamaño 5, corrección M`) →
   "Gracias por su compra" → `print_cut()`.
3. Destino (`enum Destino`, serde externally-tagged):
   - `{"Spooler": {"nombre"}}` → `enviar_bytes_raw` de Fase 1 (Windows).
   - `{"Red": {"ip", "puerto"}}` → `TcpStream::connect_timeout` 5s +
     `write_timeout` 10s + `write_all` + `flush`.

## 5. Frontend (src/services/impresora.ts — unica fuente de verdad)

Tipos: `ImpresoraInfo`, `FilaConciliacionPrint`, `DestinoPrint`,
`LineaVentaPrint`, `PagoPrint`, `TicketVentaPrint`. Nunca `invoke` crudo.

- `PanelInventario.tsx` (Conciliacion → **Imprimir Lista**): abre modal
  oscuro con las impresoras reales del spooler (default preseleccionada),
  manda la tabla visible ya filtrada/ordenada. Sin impresoras: mensaje que
  pide instalar la termica 80mm.
- `modalticket.tsx` (venta → bloque de impresion): pestañas **Local/Red**;
  en Local un select del spooler, en Red inputs IP/puerto + boton **Probar**
  (no gasta papel). Imprime carrito + pagos + QR `YARVIS-{folio}`.

## 6. Verificacion (laboratorio, sin termica fisica)

- `cargo build` en verde (solo warning benigno del linker por crate-type).
- `builder.rs`: 2/2 tests; `ticket.rs`: 3/3 tests (INIT, folio, corte,
  ticket vacio → error). Render real comprobado: 553 bytes, `1B 40`…
  `1D 56 41 00`, tildes intactas via `encoding_rs`.
- `tsc` limpio; `vitest` 83/83 (suite completa).
- Contrato frontend↔backend auditado comando por comando (§3).

## 7. Prueba de campo pendiente (con la termica conectada)

1. Instalar la termica 80mm en Windows + pagina de prueba del sistema.
2. Conciliacion → Imprimir Lista (ticket simple; valida canal RAW).
3. Venta real → imprimir (valida formato, totales, QR escaneable, corte).
4. Si hay termica de red: Probar (debe dar OK sin papel) → imprimir.
5. Fallos tipicos a reportar con modelo exacto: basura en vez de letras
   (codepage), no corta (variante de corte), no sale nada (driver/spooler).

## 8. Decisiones y lecciones

- RAW por spooler en vez de GDI: el dialogo nativo de Windows sirve para
  HTML/PDF, no para termica; para ESC/POS el selector propio poblado del
  spooler es el patron correcto (como `tauri-pos-printer`).
- `builder.rs` no se toco en Fase 2: lo que funciona no se toca; el ticket
  nuevo vive en `ticket.rs` en paralelo.
- `print_cut()` de `escpos 0.20` emite corte parcial `1D 56 41 00`
  (no `1D 56 00` como el builder manual): ambos validos, no mezclar por
  estetica, cada ticket usa el suyo.
- Sin `imprimir_bytes_raw` como puente publico, la Fase 2 no podria
  reutilizar la Fase 1: el comando existe para eso.
