// ============================================================
// patrones — Regex del lector de catálogos visuales.
//
// Nota regex: Python usaba lookbehind `(?<![-=*~>])` para que el nombre no
// robe un separador real ("Coca-Cola 600ML -- $25 $18" cae en _PATRON_PRODUCTO).
// El crate `regex` de Rust NO soporta lookaround, así que se reescribe como
// clase negada final: `(.+?[^-=*~>])`. Semántica equivalente, verificada
// contra los mismos casos de Python (test_lector_txt.py).
// ============================================================

use regex::Regex;
use std::sync::LazyLock;

// Patrón flexible: nombre + separador + precio1 + precio2
pub(super) static PATRON_PRODUCTO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"([A-Za-z0-9áéíóúñüÁÉÍÓÚÑÜ\s\.\-'"°®™]+?)\s*[-=*~>]+\s*\$?\s*([\d,]+(?:\.\d+)?)(?:\s*[-=*~>]*\s*\$?\s*([\d,]+(?:\.\d+)?))?"#,
    )
    .expect("regex producto")
});

// Patrón para detectar cantidad al inicio de la línea (ej: "10Producto $10 $5")
pub(super) static PATRON_CANTIDAD_INICIO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^(\d+)\s*([A-Za-z0-9áéíóúñüÁÉÍÓÚÑÜ\s\.\-'"°®™]+?)\s*[-=*~>]+\s*\$?\s*([\d,]+(?:\.\d+)?)?(?:\s+\$?\s*([\d,]+(?:\.\d+)?)?)?"#,
    )
    .expect("regex cantidad inicio")
});

// Patrones SIN separador (solo espacios): Nombre  CANT  $VTA  $CST
// El lookbehind original (?<![-=*~>]) se reescribe como `(.+?[^-=*~>])` para
// que el nombre NO termine en un separador real (bug 8: SIN_SEP ganaba por
// precedencia sobre _PATRON_PRODUCTO y se comía el "--").
// Patrón 1: Con cantidad y con $ en precios
pub(super) static PATRON_SIN_SEP_CANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.+?[^-=*~>])\s+(\d+)\s+\$([\d,]+(?:\.\d+)?)\s+\$([\d,]+(?:\.\d+)?)")
        .expect("regex sin separador con cantidad")
});
// Patrón 2: Sin cantidad, con $ en precios
pub(super) static PATRON_SIN_SEP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.+?[^-=*~>])\s+\$([\d,]+(?:\.\d+)?)\s+\$([\d,]+(?:\.\d+)?)")
        .expect("regex sin separador")
});
// Patrón 3: Con cantidad, sin $ en precios
pub(super) static PATRON_SIN_SEP_CANT_SINDOL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.+?[^-=*~>])\s+(\d+)\s+([\d,]+(?:\.\d+)?)\s+([\d,]+(?:\.\d+)?)")
        .expect("regex sin separador con cantidad sin dolar")
});
// Patrón 4: Sin cantidad, sin $
pub(super) static PATRON_SIN_SEP_SINDOL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.+?[^-=*~>])\s+([\d,]+(?:\.\d+)?)\s+([\d,]+(?:\.\d+)?)")
        .expect("regex sin separador sin dolar")
});

pub(super) static PATRON_LINEA_HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[\s─]+$|PRODUCTO.*CANT.*VTA|^\s*$").expect("regex linea header")
});

pub(super) static CANTIDAD_FINAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.*?)\s+(\d{1,3})$").expect("regex cantidad final"));

pub(super) const UNIDADES: &[&str] = &["ml", "l", "kg", "g", "gr", "oz", "lb"];
