// ============================================================
// filtrador.rs — Port de yarvis-IA/parseador_de_tickets/cerebro/filtrador.py
// limpiar_producto + es_categoria (regex puro, sin modelos).
// ============================================================

use regex::Regex;
use std::sync::LazyLock;

// ---------- Patrones (1:1 con filtrador.py) ----------

// Códigos de barras de 8 a 14 dígitos
static CODIGO_BARRAS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d{8,14}\b").expect("regex codigo de barras"));

// Prefijos tipo "ART.", "COD ", "N°" (case-insensitive) más el espacio posterior
static PREFIJOS_ELIMINAR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:ART\.?|COD\.?|CODIGO\.?|SKU\.?|NO\.?|N[°º]\.?)\s*")
        .expect("regex prefijos a eliminar")
});

// Caracteres raros → se reemplazan por espacio
static CARACTERES_RAROS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[^\w\s$.,;:!?\-/%()"]"#).expect("regex caracteres raros"));

// Espacios múltiples → uno solo
static ESPACIOS_MULTIPLES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s{2,}").expect("regex espacios multiples"));

const PREFIJOS_COMUNES: &[&str] = &[
    "BEBIDAS ",
    "ABARROTES ",
    "LACTEOS ",
    "CARNES ",
    "FRUTAS Y VERDURAS ",
    "LIMPIEZA ",
    "HIGIENE ",
    "PANADERIA ",
    "CHARCUTERIA ",
    "POLLERIA ",
    "CERVEZAS ",
    "VINOS Y LICORES ",
    "BOTANAS ",
    "ENLATADOS ",
];

// ---------- Funciones públicas ----------

/// Limpia un nombre de producto (códigos de barras, prefijos, caracteres
/// raros, espacios y MAYÚSCULAS). Espejo de `limpiar_producto`.
pub fn limpiar_producto(nombre: &str) -> String {
    if nombre.is_empty() {
        return String::new();
    }

    let mut limpio = nombre.trim().to_string();

    limpio = CODIGO_BARRAS.replace_all(&limpio, "").into_owned();
    limpio = PREFIJOS_ELIMINAR.replace_all(&limpio, "").into_owned();
    limpio = CARACTERES_RAROS.replace_all(&limpio, " ").into_owned();
    limpio = ESPACIOS_MULTIPLES.replace_all(&limpio, " ").into_owned();

    let mut limpio = limpio.trim().to_uppercase();

    for prefijo in PREFIJOS_COMUNES {
        if limpio.starts_with(prefijo) {
            limpio = limpio[prefijo.len()..].to_string();
            break;
        }
    }

    limpio.trim().to_string()
}

/// Detecta si una línea es una categoría (MAYÚSCULAS, sin precios, corta).
/// Espejo de `es_categoria`.
pub fn es_categoria(linea: &str) -> bool {
    let linea = linea.trim();

    if linea.is_empty() {
        return false;
    }
    if linea.to_uppercase() != linea {
        return false;
    }
    if linea.contains('$') || linea.contains("--") || linea.contains('=') {
        return false;
    }
    if linea.chars().count() > 2 && linea.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }
    if linea.chars().count() > 40 {
        return false;
    }
    true
}

// ============================================================
// Tests (espejo de los casos de comportamiento de Python)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- limpiar_producto ----------

    #[test]
    fn vacio_devuelve_vacio() {
        assert_eq!(limpiar_producto(""), "");
        assert_eq!(limpiar_producto("   "), "");
        assert_eq!(limpiar_producto("        "), "");
    }

    #[test]
    fn elimina_codigo_de_barras() {
        assert_eq!(
            limpiar_producto("7501181016275 COCA-COLA 600ML"),
            "COCA-COLA 600ML"
        );
        // 8 dígitos (mínimo del rango)
        assert_eq!(limpiar_producto("12345678 GATORADE"), "GATORADE");
        // No debe borrar números cortos (precios códigos pequeños)
        assert_eq!(limpiar_producto("PRODUCTO 500ML 25"), "PRODUCTO 500ML 25");
    }

    #[test]
    fn elimina_prefijos_articulo_codigo() {
        assert_eq!(limpiar_producto("ART. COCA-COLA 600ML"), "COCA-COLA 600ML");
        assert_eq!(limpiar_producto("COD. FANTA 500ML"), "FANTA 500ML");
        assert_eq!(limpiar_producto("SKU 12345678 MANZANA"), "MANZANA");
        assert_eq!(limpiar_producto("art. GATORADE"), "GATORADE");
        // Comportamiento verificado en Python: la alternation COD\.? gana
        // sobre CODIGO\.?, por lo que solo se borra el prefijo "COD".
        assert_eq!(limpiar_producto("CODIGO GATORADE"), "IGO GATORADE");
        // N° elimina el prefijo pero deja el número pequeño (no es código de barras).
        assert_eq!(limpiar_producto("N° 12345 SABRITAS"), "12345 SABRITAS");
    }

    #[test]
    fn reemplaza_caracteres_raros() {
        assert_eq!(limpiar_producto("COCA*COLA 600ML"), "COCA COLA 600ML");
        // '_' es \w en regex → se conserva (mismo comportamiento que Python).
        assert_eq!(limpiar_producto("PAN_BLANCO"), "PAN_BLANCO");
        assert_eq!(limpiar_producto("PAN#BLANCO"), "PAN BLANCO");
        assert_eq!(limpiar_producto("GAL&LETAS"), "GAL LETAS");
    }

    #[test]
    fn colapsa_espacios_multiples() {
        assert_eq!(
            limpiar_producto("COCA      COLA   600ML"),
            "COCA COLA 600ML"
        );
    }

    #[test]
    fn normaliza_a_mayusculas() {
        assert_eq!(limpiar_producto("coca-cola 600ml"), "COCA-COLA 600ML");
        assert_eq!(
            limpiar_producto("Sabritas 16 12"),
            "SABRITAS 16 12"
        );
    }

    #[test]
    fn quita_prefijo_de_categoria() {
        assert_eq!(
            limpiar_producto("BEBIDAS COCA-COLA 600ML"),
            "COCA-COLA 600ML"
        );
        assert_eq!(
            limpiar_producto("FRUTAS Y VERDURAS MANZANA"),
            "MANZANA"
        );
        assert_eq!(
            limpiar_producto("limpieza cloro"),
            "CLORO"
        );
    }

    // ---------- es_categoria ----------

    #[test]
    fn linea_vacia_no_es_categoria() {
        assert!(!es_categoria(""));
        assert!(!es_categoria("   "));
    }

    #[test]
    fn categoria_valida() {
        assert!(es_categoria("BEBIDAS"));
        assert!(es_categoria("ABARROTES"));
        assert!(es_categoria("FRUTAS Y VERDURAS"));
        assert!(es_categoria("VINOS Y LICORES"));
    }

    #[test]
    fn minusculas_no_es_categoria() {
        assert!(!es_categoria("bebidas"));
        assert!(!es_categoria("BEBIDAS y"));
    }

    #[test]
    fn con_precios_no_es_categoria() {
        assert!(!es_categoria("BEBIDAS $5"));
        assert!(!es_categoria("BEBIDAS --"));
        assert!(!es_categoria("BEBIDAS ="));
    }

    #[test]
    fn con_digitos_no_es_categoria() {
        assert!(!es_categoria("SECCION 2"));
        assert!(!es_categoria("PASILLO 01"));
    }

    #[test]
    fn muy_larga_no_es_categoria() {
        let larga = "A".repeat(41);
        assert!(!es_categoria(&larga));
        // Justo en el límite (40) sí es válida
        let limite = "B".repeat(40);
        assert!(es_categoria(&limite));
    }
}