use serde::{Deserialize, Serialize};

/// Mapeo de columnas definido por el usuario (contrato frontend ↔ núcleo).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapeoColumnas {
    pub cantidad: Option<i32>,
    pub producto: Option<Vec<i32>>,
    #[serde(rename = "precio_unitario")]
    pub precio_unitario: Option<i32>,
    pub total: Option<i32>,
    pub descuento: Option<i32>,
}

/// Resultado de parsear UNA línea de ticket (espejo del dict de Python).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Item {
    pub producto: String,
    pub cantidad: f64,
    pub precio_unitario: f64,
    pub total: f64,
    pub descuento: Option<f64>,
}

/// Resuelve un índice de columna (admite negativos: -1 = última columna).
pub fn resolver_indice(col: Option<i32>, total_cols: usize) -> Option<usize> {
    let col = col?;
    if col < 0 {
        let res = total_cols as i64 + col as i64;
        if res < 0 {
            None
        } else {
            Some(res as usize)
        }
    } else {
        Some(col as usize)
    }
}
