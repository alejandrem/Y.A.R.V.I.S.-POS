use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local, NaiveDate};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GastoRecurrente {
    pub id: i64,
    pub nombre: String,
    pub tipo: String,
    pub categoria: String,
    pub monto_proyectado: f64,
    pub monto_real: f64,
    pub frecuencia: String,
    pub dia_pago: Option<i32>,
    pub intervalo_dias: Option<i32>,
    pub fecha_inicio: String,
    pub fecha_fin: Option<String>,
    pub estado_pago: String,
    pub folio_comprobante: Option<String>,
    pub comprobante_url: Option<String>,
    pub notas: Option<String>,
    pub creado_en: String,
    pub actualizado_en: String,
    pub proxima_fecha_pago: Option<String>,
    pub dias_para_vencer: Option<i32>,
}

#[derive(Deserialize)]
pub struct CrearGastoRequest {
    pub nombre: String,
    pub tipo: String,
    pub categoria: String,
    pub monto_proyectado: f64,
    pub frecuencia: String,
    pub dia_pago: Option<i32>,
    pub intervalo_dias: Option<i32>,
    pub fecha_inicio: String,
    pub fecha_fin: Option<String>,
    pub folio_comprobante: Option<String>,
    pub notas: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PagoGasto {
    pub id: i64,
    pub gasto_id: i64,
    pub fecha_pago: String,
    pub monto_pagado: f64,
    pub metodo_pago: Option<String>,
    pub folio_comprobante: Option<String>,
    pub comprobante_url: Option<String>,
    pub notas: Option<String>,
    pub creado_en: String,
}

#[derive(Deserialize)]
pub struct RegistrarPagoRequest {
    pub gasto_id: i64,
    pub fecha_pago: String,
    pub monto_pagado: f64,
    pub metodo_pago: Option<String>,
    pub folio_comprobante: Option<String>,
    pub notas: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CorteCaja {
    pub id: i64,
    pub fecha_apertura: String,
    pub fecha_cierre: Option<String>,
    pub monto_inicial: f64,
    pub total_ventas: f64,
    pub total_efectivo: f64,
    pub total_tarjeta: f64,
    pub total_transferencia: f64,
    pub entradas_manuales: f64,
    pub retiros_manuales: f64,
    pub diferencia: f64,
    pub usuario_id: i64,
    pub usuario_nombre: String,
    pub estado: String,
    pub tipo_corte: String,
    pub turno: Option<String>,
    pub observaciones: Option<String>,
}

#[derive(Deserialize)]
pub struct CrearCorteRequest {
    pub monto_inicial: f64,
    pub tipo_corte: String,
    pub turno: Option<String>,
    pub observaciones: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MovimientoCaja {
    pub id: i64,
    pub corte_id: i64,
    pub tipo: String,
    pub concepto: String,
    pub monto: f64,
    pub metodo_pago: Option<String>,
    pub referencia_id: Option<i64>,
    pub creado_en: String,
}

#[derive(Deserialize)]
pub struct MovimientoCajaRequest {
    pub corte_id: i64,
    pub tipo: String,
    pub concepto: String,
    pub monto: f64,
    pub metodo_pago: Option<String>,
    pub referencia_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MetricasUtilidad {
    pub fecha: String,
    pub ventas_totales: f64,
    pub costo_ventas: f64,
    pub utilidad_bruta: f64,
    pub gastos_operativos: f64,
    pub utilidad_operativa: f64,
    pub impuestos_comisiones: f64,
    pub utilidad_neta: f64,
    pub margen_neto_pct: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResumenPeriodo {
    pub periodo_inicio: String,
    pub periodo_fin: String,
    pub total_ventas: f64,
    pub total_costo_ventas: f64,
    pub total_utilidad_bruta: f64,
    pub total_gastos_operativos: f64,
    pub total_utilidad_operativa: f64,
    pub total_impuestos_comisiones: f64,
    pub total_utilidad_neta: f64,
    pub margen_promedio_pct: f64,
    pub punto_equilibrio_ventas: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatoGraficaPL {
    pub fecha: String,
    pub ingresos: f64,
    pub gastos: f64,
    pub utilidad_neta: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatoGraficaGastosCategoria {
    pub categoria: String,
    pub monto: f64,
    pub porcentaje: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatoGraficaCortesZ {
    pub fecha: String,
    pub turno: String,
    pub total_ventas: f64,
    pub diferencia: f64,
    pub cajero: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PuntoEquilibrio {
    pub gastos_fijos_mensuales: f64,
    pub margen_contribucion_pct: f64,
    pub ventas_necesarias: f64,
    pub tickets_promedio: f64,
    pub tickets_necesarios: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AlertaFinanciera {
    pub id: i64,
    pub tipo: String,
    pub severidad: String,
    pub titulo: String,
    pub mensaje: String,
    pub entidad_id: Option<i64>,
    pub entidad_tipo: Option<String>,
    pub fecha_vencimiento: Option<String>,
    pub leida: bool,
    pub creada_en: String,
}

#[derive(Deserialize)]
pub struct FiltrosCortes {
    pub cajero_id: Option<i64>,
    pub fecha_inicio: Option<String>,
    pub fecha_fin: Option<String>,
    pub turno: Option<String>,
    pub tipo_corte: Option<String>,
    pub estado: Option<String>,
}

#[derive(Deserialize)]
pub struct FiltrosPeriodo {
    pub fecha_inicio: String,
    pub fecha_fin: String,
}