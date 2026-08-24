// ============================================================
// dinero — Tipos monetarios del sistema.
//
// REGLA DE ORO: el dinero vive en la DB como INTEGER en CENTAVOS.
// El contrato IPC (serde) habla PESOS (f64) con el frontend; toda
// conversión pasa por este módulo, nunca a mano con *100 / /100.
//
// Por qué: los f64 acumulan error de redondeo en SUMs de miles de
// filas y hacen las comparaciones frágiles (epsilon +0.01). Con
// enteros, `a_centavos(pagado) < a_centavos(total)` es EXACTO.
// ============================================================

/// Centavos por peso. Constante única para que quede explícito el factor.
pub const CENTAVOS_POR_PESO: i64 = 100;

/// Pesos (f64, contrato IPC) → centavos (i64, persistencia).
/// Redondea al centavo más cercano para absorber el ruido binario
/// del f64 (ej: 0.1 * 100 = 10.000000000000002 → 10).
pub fn a_centavos(pesos: f64) -> i64 {
    (pesos * CENTAVOS_POR_PESO as f64).round() as i64
}

/// Centavos (i64, persistencia) → pesos (f64, contrato IPC).
pub fn a_pesos(centavos: i64) -> f64 {
    centavos as f64 / CENTAVOS_POR_PESO as f64
}

/// Convierte un agregado SQL que mezcla cantidad REAL × precio en
/// centavos INTEGER (el resultado viene como f64 en unidades de
/// centavos) y lo redondea al centavo exacto. Útil tras
/// `SUM(cantidad * precio_cents)`.
pub fn centavos_f64_a_i64(centavos: f64) -> i64 {
    centavos.round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversiones_basicas() {
        assert_eq!(a_centavos(13.5), 1350);
        assert_eq!(a_centavos(0.1), 10);
        assert_eq!(a_centavos(19.99), 1999);
        assert_eq!(a_pesos(1350), 13.5);
        assert_eq!(a_pesos(0), 0.0);
        // Ida y vuelta estable para valores de 2 decimales
        for p in [0.05, 1.99, 42.50, 1234.56, 99999.99] {
            assert_eq!(a_pesos(a_centavos(p)), p);
        }
    }

    #[test]
    fn ruido_binario_se_absorbe() {
        // Sin .round(), esto daría 10000000000000001
        let ruido = 0.1 + 0.2;
        assert_eq!(a_centavos(ruido), 30);
        assert_eq!(a_centavos(ruido * 333.0), 9990);
    }
}
