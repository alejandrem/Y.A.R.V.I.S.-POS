// ============================================================
// ean — Digito verificador para el semaforo verde.
//
// El issue #10 lo exige: "el ean tiene digito verificador valido".
// Sin esto, un dedo gordo al capturar el dataset vincula el codigo
// del vecino y la cajera cobra mal desde el dia 1.
//
// Soporta lo que trae dataset/*.csv (numericos 12-13, algun 8):
//   * EAN-13 (750... Mexico), UPC-A 12 (algunos importados), EAN-8.
//   * GTIN-14 se valida con la misma regla EAN (mod10).
// Alfanumericos (Code128 interno "UNICO123") NO tienen checksum:
// se permiten, la unicidad la impone SQLite igual.
// ============================================================

/// ¿El codigo normalizado pasa el checksum? `""` -> false.
pub fn validar_ean(codigo_norm: &str) -> bool {
    if codigo_norm.is_empty() {
        return false;
    }
    if !codigo_norm.chars().all(|c| c.is_ascii_digit()) {
        // Code128 / interno: sin checksum que validar, se permite.
        return true;
    }
    match codigo_norm.len() {
        8 => validar_gtin(codigo_norm),
        12 => validar_gtin(codigo_norm),
        13 => validar_gtin(codigo_norm),
        14 => validar_gtin(codigo_norm),
        _ => false,
    }
}

/// Mod10 GTIN (vale para 8/12/13/14): desde la derecha sin el digito,
/// 3-1-3-1..., check = (10 - suma%10) % 10.
fn validar_gtin(digitos: &str) -> bool {
    let nums: Vec<u32> = digitos.chars().map(|c| c.to_digit(10).unwrap()).collect();
    let (cuerpo, check) = nums.split_at(nums.len() - 1);
    let mut suma = 0u32;
    // Recorre el cuerpo de derecha a izquierda: posiciones impares x3.
    for (i, d) in cuerpo.iter().rev().enumerate() {
        if i % 2 == 0 {
            suma += d * 3;
        } else {
            suma += d;
        }
    }
    let esperado = (10 - (suma % 10)) % 10;
    esperado == check[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_real_pasa() {
        // Muestra del dataset real (botana + refrescos + lacteos).
        assert!(validar_ean("7501011101456")); // SABRITAS ORIGINAL 42g
        assert!(validar_ean("7501055304745")); // COCA 3L
        assert!(validar_ean("7501055370276")); // SANTA CLARA cafe 250ml
        assert!(validar_ean("0000075007614")); // COCA 600ml (13 con ceros)
    }

    #[test]
    fn un_digito_mal_falla() {
        assert!(!validar_ean("7501011101457")); // ultimo cambiado
        assert!(!validar_ean("7501055304746"));
        assert!(!validar_ean("123"));
        assert!(!validar_ean(""));
    }

    #[test]
    fn code128_sin_checksum_se_permite() {
        assert!(validar_ean("UNICO123"));
        assert!(validar_ean("CAJA-001"));
    }
}
