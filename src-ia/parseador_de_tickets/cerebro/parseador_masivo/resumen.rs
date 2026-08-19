// ============================================================
// Modelos de resumen/estadísticas del procesamiento masivo
// ============================================================

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProductoNuevo {
    pub nombre: String,
    pub precio: f64,
}

/// Resultado de procesar UN archivo (equivalente a un `yield` de Python).
///
/// Un archivo puede contener N tickets → `ventas` cuenta cuántas ventas se
/// crearon y `ventas_info` trae el detalle por ticket creado.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchivoResultado {
    pub archivo: String,
    pub ok: bool,
    pub motivo: Option<String>,
    pub items: usize,
    pub duplicados: usize,
    pub nuevos: Vec<ProductoNuevo>,
    pub existentes: usize,
    pub venta_id: Option<i64>,
    pub total: f64,
    /// Venta(s) creadas a partir de ESTE archivo (1 por ticket detectado).
    pub ventas: usize,
    /// Detalle por venta creada (folio/fecha/hora/items/total por ticket).
    pub ventas_info: Vec<ResumenVenta>,
}

impl ArchivoResultado {
    pub(super) fn info(ok: bool, motivo: Option<String>) -> Self {
        Self {
            archivo: String::new(),
            ok,
            motivo,
            items: 0,
            duplicados: 0,
            nuevos: Vec::new(),
            existentes: 0,
            venta_id: None,
            total: 0.0,
            ventas: 0,
            ventas_info: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResumenVenta {
    pub archivo: String,
    pub venta_id: Option<i64>,
    pub items: usize,
    pub total: f64,
    pub folio: Option<String>,
    pub fecha_hora: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TicketFallido {
    pub archivo: String,
    pub motivo: Option<String>,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct EstadisticasCarpeta {
    pub total_archivos: usize,
    pub procesados: usize,
    pub exitosos: usize,
    pub errores: usize,
    pub ventas_creadas: usize,
    pub items_insertados: usize,
    pub productos_nuevos: usize,
    pub productos_existentes: usize,
    pub duplicados_detectados: usize,
    pub productos_nuevos_lista: Vec<ProductoNuevo>,
    pub resumen_ventas: Vec<ResumenVenta>,
    pub tickets_fallidos: Vec<TicketFallido>,
}
