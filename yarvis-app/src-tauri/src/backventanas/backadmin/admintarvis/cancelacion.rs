// ============================================================
// admintarvis/cancelacion.rs — Bandera de cancelación cooperativa
// del stream del chatbot (stop_chat_stream ↔ bucles de emisión).
// ============================================================

/// Bandera de cancelación del stream en curso. `stop_chat_stream` la levanta;
/// los bucles de emisión (cloud y local) la consultan entre tokens/rondas y
/// cortan la respuesta. Es cooperativa: la generación local a bloqueo termina
/// su ciclo interno, pero NADA más se emite a la UI después de cancelar.
pub(super) static STREAM_CANCELADO: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub(super) fn stream_cancelado() -> bool {
    STREAM_CANCELADO.load(std::sync::atomic::Ordering::Relaxed)
}

/// Nueva generación: cancelación limpia.
pub(super) fn reset_stream_cancelado() {
    STREAM_CANCELADO.store(false, std::sync::atomic::Ordering::Relaxed);
}
