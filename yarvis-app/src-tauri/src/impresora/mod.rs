// ============================================================
// impresora — Termicas ESC/POS: Camino A Fase 1 + Camino C Fase 2.
//
// Fase 1: spooler de Windows con bytes RAW. La impresora ya esta
// instalada con su driver; el backend abre el spooler por WinAPI
// (OpenPrinterW + WritePrinter, datatype RAW). Cero config.
//
// Fase 2: ticket de venta enriquecido con el crate `escpos`
// (formato, QR, barcodes). Se renderiza a memoria y se envia por
// el spooler RAW de Fase 1 o por TCP directo (puerto 9100).
//
// Estructura:
//   builder.rs          -> conciliacion 80mm/48cols, puro Rust (Fase 1)
//   memoria.rs          -> driver escpos que captura a memoria (Fase 2)
//   ticket.rs           -> ticket de venta con QR via `escpos` (Fase 2)
//   spooler_windows.rs  -> EnumPrintersW + OpenPrinterW/WritePrinter (win)
//   spooler_stub.rs     -> error claro fuera de Windows (Fase 1)
//   commands.rs         -> comandos Tauri que usa el frontend
// ============================================================

pub mod builder;
pub mod commands;
pub mod memoria;
pub mod ticket;

#[cfg(windows)]
mod spooler_windows;
#[cfg(not(windows))]
mod spooler_stub;

#[cfg(windows)]
pub use spooler_windows::{enviar_bytes_raw, listar_impresoras_sistema};
#[cfg(not(windows))]
pub use spooler_stub::{enviar_bytes_raw, listar_impresoras_sistema};
