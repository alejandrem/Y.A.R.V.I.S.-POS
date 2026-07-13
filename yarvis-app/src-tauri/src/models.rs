use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AdminData {
    pub name: String,
    pub store: String,
    pub pass: String,
}

#[derive(Serialize, Deserialize)]
pub struct TicketItem {
    pub producto: String,
    pub cantidad: f64,
    #[serde(alias = "precio_unitario")]
    pub precio: f64,
    pub total: f64,
}

#[derive(Serialize)]
pub struct TicketDb {
    pub id: i32,
    pub fecha: String,
    pub total: f64,
    pub metodo_pago: String,
}

#[derive(Serialize)]
pub struct CorteDb {
    pub id: i32,
    pub fecha: String,
    pub total_ventas: f64,
    pub total_efectivo: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InventoryItem {
    pub id: Option<i32>,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub precio_costo: f64,
    pub precio_venta: f64,
    pub stock: f64,
    pub stock_minimo: f64,
    pub vendido: f64,
    pub codigo_barras: Option<String>,
    pub categoria: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AdminProfile {
    pub nombre: String,
    pub tienda: String,
    pub password: String,
    pub ubicacion: Option<String>,
    pub cp: Option<String>,
}

// ============================================================
// MODELOS PARA LA API DE EMBEDDINGS (Python ↔ Rust)
// ============================================================

#[derive(Deserialize)]
pub struct EmbeddingResponse {
    #[allow(dead_code)] // used by serde deserialization
    pub status: String,
    pub dimensions: usize,
    pub blob_b64: String,
}

#[derive(Serialize)]
pub struct SimilarSearchRequest {
    pub query: String,
    pub db_path: String,
    pub top_k: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categoria: Option<String>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct SimilarResult {
    pub id: i64,
    pub contenido: String,
    pub categoria: String,
    pub score: f64,
}

#[derive(Deserialize)]
pub struct SimilarSearchResponse {
    #[allow(dead_code)] // used by serde deserialization
    pub status: String,
    pub results: Vec<SimilarResult>,
}

#[derive(Serialize, Deserialize)]
pub struct EmployeeGoal {
    pub id: i32,
    pub employee_id: i32,
    pub goal_type: String,
    pub goal_name: Option<String>,
    pub ventas_threshold: String,
    pub bonus_percentage: f64,
    pub bonus_amount: f64,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
}

// ============================================================
// MODELOS PARA VENTA DE EMPLEADO
// ============================================================

#[derive(Deserialize, Clone)]
pub struct CartItemRequest {
    pub id: Option<i32>,
    pub nombre: String,
    pub precio_venta: f64,
    pub cantidad: f64,
}

#[derive(Deserialize)]
pub struct VentaRequest {
    pub items: Vec<CartItemRequest>,
    pub total: f64,
    pub subtotal: f64,
    pub descuento: f64,
    pub monto_efectivo: f64,
    pub monto_tarjeta: f64,
    pub monto_transferencia: f64,
    pub cajero: String,
    pub cliente_id: Option<i32>,
}

#[derive(Serialize)]
pub struct VentaResponse {
    pub venta_id: i64,
    pub ticket_number: i64,
    pub mensaje: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TiendaInfo {
    pub nombre: Option<String>,
    pub ubicacion: Option<String>,
    pub cp: Option<String>,
}

#[derive(Serialize)]
pub struct SalarioInfo {
    pub salario_diario: f64,
    pub horas_por_dia: f64,
    pub salario_hora: f64,
    pub salario_semanal: f64,
    pub salario_mensual: f64,
    pub dias_semana: i32,
}
