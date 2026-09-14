// ============================================================
// tipos — Formas del semaforo amarillo (issue #11).
// Solo datos, cero logica.
// ============================================================

use serde::Serialize;

/// Un candidato del top-5, ordenado por `score` desc.
#[derive(Debug, Clone, Serialize)]
pub struct Candidato {
    pub producto_id: i64,
    pub nombre: String,
    pub score: f64,
    /// "aprendizaje" (ya lo confirmaste antes) o "scoring".
    pub origen: String,
}

/// Respuesta de `amarillo_sugerir`: el front muestra el PRIMERO
/// con [SI ES ESTE] [NO ES]; con NO recorre el resto y con
/// NINGUNO cae a rojo. `pendiente_id` amarra todo el flujo.
#[derive(Debug, Clone, Serialize)]
pub struct SugerenciaTop {
    pub pendiente_id: i64,
    /// "amarillo" | "rojo" | "conflicto" | "resuelto" | "aprendizaje".
    pub estado: String,
    pub candidatos: Vec<Candidato>,
}

/// Stats de NOs para recalibrar el umbral a mano.
#[derive(Debug, Clone, Serialize)]
pub struct StatsRechazos {
    pub total: i64,
    pub score_promedio: f64,
    pub score_maximo: f64,
}
