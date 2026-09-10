// ============================================================
// impresora/memoria.rs — Driver escpos que captura a memoria.
//
// Puente del Camino C Fase 2: el ticket se arma con la API rica
// de `escpos` (formato, QR, barcodes) sobre este driver, los bytes
// se recuperan con `bytes()` y se envian por el destino elegido:
// spooler RAW de Fase 1 o TCP directo (puerto 9100).
// ============================================================

use std::sync::{Arc, Mutex};

use escpos::driver::Driver;

/// Driver en memoria: `Clone` barato via `Arc`, los bytes se leen
/// con `bytes()` despues de `print()` / `print_cut()`.
#[derive(Debug, Clone, Default)]
pub struct MemoriaDriver {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl MemoriaDriver {
    /// Copia de los bytes capturados hasta ahora.
    pub fn bytes(&self) -> Vec<u8> {
        self.buf.lock().map(|b| b.clone()).unwrap_or_default()
    }
}

impl Driver for MemoriaDriver {
    fn name(&self) -> String {
        "yarvis-memoria".into()
    }

    fn write(&self, data: &[u8]) -> escpos::errors::Result<()> {
        if let Ok(mut buf) = self.buf.lock() {
            buf.extend_from_slice(data);
        }
        Ok(())
    }

    fn read(&self, _buf: &mut [u8]) -> escpos::errors::Result<usize> {
        // Fase 2 no lee estado de la impresora (sin DLE/EOT).
        Ok(0)
    }

    fn flush(&self) -> escpos::errors::Result<()> {
        Ok(())
    }
}
