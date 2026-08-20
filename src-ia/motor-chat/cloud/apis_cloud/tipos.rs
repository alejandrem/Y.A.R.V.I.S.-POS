// ============================================================
// tipos — Tipos compartidos del motor cloud: eventos del stream,
// uso de tokens y modelo disponible de un proveedor.
// Parte de apis_cloud.
// ============================================================

use serde::{Deserialize, Serialize};

/// Uso de tokens reportado por el proveedor (se rellena durante el streaming).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

/// Evento que produce el stream de [`super::generacion::generar_stream`].
#[derive(Debug, Clone)]
pub enum Evento {
    /// Un trozo de texto crudo del modelo (puede contener marcadores think).
    /// Lleva el texto y el modelo real que lo generó (para el relevo 429).
    Texto { texto: String, modelo: String },
    /// Uso de tokens reportado por el proveedor.
    Uso { usage: Usage, modelo: String },
}

/// Modelo disponible en un proveedor de nube.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeloDisponible {
    pub id: String,
    pub name: String,
    /// Ventana de contexto reportada por el proveedor, si el endpoint la incluye.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u64>,
}
