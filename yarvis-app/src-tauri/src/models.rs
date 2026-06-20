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
