// impresora/spooler_stub.rs — Fuera de Windows, Fase 1 devuelve
// un error claro. Fase 2 traera `lpr` raw para macOS/Linux.

#[derive(Debug, Clone)]
pub struct ImpresoraSistema {
    pub nombre: String,
    pub predeterminada: bool,
}

pub fn listar_impresoras_sistema() -> Result<Vec<ImpresoraSistema>, String> {
    Err("Impresion RAW solo disponible en Windows en Fase 1.".into())
}

pub fn enviar_bytes_raw(_nombre: &str, _bytes: &[u8]) -> Result<(), String> {
    Err("Impresion RAW solo disponible en Windows en Fase 1.".into())
}
