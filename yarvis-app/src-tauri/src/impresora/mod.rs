// ============================================================
// impresora — Camino A Fase 1: spooler de Windows con bytes RAW.
//
// La impresora ya esta instalada con su driver; el backend abre el
// spooler por WinAPI (OpenPrinterW + WritePrinter, datatype RAW)
// y le manda los bytes ESC/POS tal cual. Cero config de hardware.
//
// Estructura:
//   builder.rs          -> genera Vec<u8> ESC/POS (80mm/48cols), puro Rust
//   spooler_windows.rs  -> EnumPrintersW + OpenPrinterW/WritePrinter (win)
//   spooler_stub.rs     -> error claro fuera de Windows (Fase 1)
//   commands.rs         -> comandos Tauri que usa el frontend
//
// Fase 2 (Camino C, crate `escpos`) reutilizara `builder.rs` sin cambios.
// ============================================================

pub mod builder;
pub mod commands;

#[cfg(windows)]
mod spooler_windows;
#[cfg(not(windows))]
mod spooler_stub;

#[cfg(windows)]
pub use spooler_windows::{enviar_bytes_raw, listar_impresoras_sistema};
#[cfg(not(windows))]
pub use spooler_stub::{enviar_bytes_raw, listar_impresoras_sistema};
