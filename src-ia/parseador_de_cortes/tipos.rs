// ============================================================
// tipos — Estructuras del corte parseado (contrato con el backend).
//
// Dinero en INTEGER centavos (regla de oro); cantidades en REAL.
// `TipoCorte` se serializa como "X"/"Z" para Tauri y la DB.
// ============================================================

use serde::{Deserialize, Serialize};

/// Corte X (parcial, no reinicia) vs Z (cierre definitivo del día).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoCorte {
    X,
    Z,
}

impl TipoCorte {
    pub fn etiqueta(self) -> &'static str {
        match self {
            TipoCorte::X => "X",
            TipoCorte::Z => "Z",
        }
    }
}

/// Clasificación de un archivo suelto (los datasets mezclan tickets
/// y cortes en la misma carpeta).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaseArchivo {
    CorteX,
    CorteZ,
    NoEsCorte,
}

/// Renglón del desglose: artículos, tickets o líneas de ingreso/egreso.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCorte {
    /// ARTICULO | TICKET | INGRESO | EGRESO
    pub kind: String,
    pub nombre: String,
    pub cantidad: Option<f64>,
    pub precio_unitario: i64,
    pub subtotal: i64,
}

/// Resultado de la verificación matemática (centavos exactos).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verificacion {
    pub caja_ok: bool,
    pub ventas_ok: bool,
    pub advertencias: Vec<String>,
}

/// Corte completo listo para guardar en `cortes_importados`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorteParseado {
    pub tipo: TipoCorte,
    pub folio: Option<String>,
    pub estacion: Option<String>,
    /// ISO local "YYYY-MM-DD HH:MM:SS".
    pub fecha: Option<String>,
    pub cajero: String,
    pub empresa: Option<String>,
    pub moneda: String,
    pub total_ingresos: i64,
    pub total_egresos: i64,
    pub total_caja: i64,
    pub total_ventas: i64,
    pub ventas_gravadas: i64,
    pub impuesto: i64,
    pub ventas_no_gravadas: i64,
    pub redondeos: i64,
    pub ventas_credito: i64,
    pub total_unidades: f64,
    pub clientes_atendidos: i64,
    pub items: Vec<ItemCorte>,
    pub verificacion: Verificacion,
}
