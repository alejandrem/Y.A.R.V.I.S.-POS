// ============================================================
// formatos — Lectores de catálogos (espejo de
// yarvis-IA/parseador_de_tickets/formatos/)
// TXT visual, CSV/TSV y Excel (.xlsx/.xls) con calamine.
// ============================================================

pub mod lector_csv;
pub mod lector_excel;
pub mod lector_txt;

use serde::Serialize;

/// Un producto parseado de un catálogo (espejo del dict de Python).
#[derive(Debug, Clone, Serialize)]
pub struct ProductoCatalogo {
    pub nombre: String,
    pub precio_costo: f64,
    pub precio_venta: f64,
    pub stock: i64,
    pub categoria: String,
}
