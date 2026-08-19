//! lector_excel.rs — Port de `yarvis-IA/parseador_de_tickets/formatos/lector_excel.py`
//!
//! Parser de catálogos en formato Excel (.xlsx, .xls) con `calamine`
//! (equivalente a openpyxl read_only + values_only). Detecta columnas por
//! nombre o posición, igual que Python:
//!   - Header en fila 1 (o fila 2 si la 1 parece datos).
//!   - Columnas: nombre/producto, costo, venta, categoría, stock.
//!   - Precio mayor → costo si están invertidos; stock por columna.
//!   - Una celda que sea categoría pura no se convierte en producto.

use calamine::{Data, Reader, Xlsx};
use std::io::Cursor;

use super::ProductoCatalogo;
use crate::cerebro::analizador_tickets::PRECIO_MAXIMO;
use crate::cerebro::filtrador::{es_categoria, limpiar_producto};

const SIN_CATEGORIA: &str = "SIN CATEGORÍA";

// Palabras clave por columna (1:1 con lector_excel.py).
const KW_NOMBRE: &[&str] = &[
    "nombre",
    "producto",
    "descripción",
    "descripcion",
    "articulo",
    "artículo",
    "name",
    "product",
    "description",
    "item",
];
const KW_COSTO: &[&str] = &["costo", "cost", "precio_compra", "precio costo", "cost price"];
const KW_VENTA: &[&str] = &[
    "venta",
    "publico",
    "público",
    "precio_venta",
    "precio venta",
    "precio",
    "price",
    "selling",
    "sale",
    "retail",
];
const KW_CATEGORIA: &[&str] = &[
    "categoría",
    "categoria",
    "tipo",
    "seccion",
    "sección",
    "departamento",
    "category",
    "type",
    "section",
];
const KW_STOCK: &[&str] = &["stock", "existencia", "cantidad", "inventario", "quantity", "inventory"];
const KW_HEADER: &[&str] = &[
    "nombre",
    "producto",
    "name",
    "cost",
    "price",
    "venta",
    "costo",
    "precio",
    "categor",
    "category",
    "tipo",
    "stock",
];

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

// ---------------------------------------------------------------------------
// Valores de celda (equivalente a `iter_rows(values_only=True)` de Python)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Celda {
    Vacio,
    Texto(String),
    Numero(f64),
    Entero(i64),
    Booleano(bool),
}

fn a_celda(d: &Data) -> Celda {
    match d {
        Data::Empty => Celda::Vacio,
        Data::String(s) => Celda::Texto(s.clone()),
        Data::Float(f) => Celda::Numero(*f),
        Data::Int(i) => Celda::Entero(*i),
        Data::Bool(b) => Celda::Booleano(*b),
        // Fechas/durante/errores no aportan como texto de catálogo.
        Data::DateTime(..) | Data::DateTimeIso(_) | Data::DurationIso(_) | Data::Error(_) => {
            Celda::Vacio
        }
    }
}

/// `str(v).lower().strip() if v else ''` para headers.
fn valor_a_header(c: &Celda) -> String {
    match c {
        Celda::Vacio => String::new(),
        Celda::Texto(s) => s.to_lowercase().trim().to_string(),
        Celda::Numero(f) => f.to_string(),
        Celda::Entero(i) => i.to_string(),
        Celda::Booleano(b) => b.to_string(),
    }
}

/// `str(v).strip()` para el nombre del producto.
fn valor_a_texto(c: &Celda) -> String {
    match c {
        Celda::Vacio => String::new(),
        Celda::Texto(s) => s.trim().to_string(),
        Celda::Numero(f) => f.to_string(),
        Celda::Entero(i) => i.to_string(),
        Celda::Booleano(b) => b.to_string(),
    }
}

/// `float(str(v).replace('$','').replace(',','').strip())` con guardia
/// anti-explosión (finito y ≤ PRECIO_MAXIMO, como el resto del crate).
fn valor_a_numero(c: &Celda) -> f64 {
    let v = match c {
        Celda::Vacio => 0.0,
        Celda::Numero(f) => *f,
        Celda::Entero(i) => *i as f64,
        Celda::Booleano(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Celda::Texto(s) => {
            let limpio: String = s.chars().filter(|ch| *ch != '$' && *ch != ',').collect();
            limpio.trim().parse().unwrap_or(0.0)
        }
    };
    if v.is_finite() && v.abs() <= PRECIO_MAXIMO {
        v
    } else {
        0.0
    }
}

fn tiene_digito(h: &str) -> bool {
    h.chars().any(|c| c.is_ascii_digit())
}

fn contiene_kw(header: &str, kws: &[&str]) -> bool {
    kws.iter().any(|k| header.contains(*k))
}

// ---------------------------------------------------------------------------
// Parser principal
// ---------------------------------------------------------------------------

/// Parsea un catálogo desde los bytes de un archivo Excel.
pub fn parsear_excel(bytes: &[u8]) -> Result<Vec<ProductoCatalogo>, String> {
    let source = Cursor::new(bytes.to_vec());
    let mut wb = Xlsx::new(source)
        .map_err(|e| format!("Error al leer Excel: {e}"))?;

    let mut productos = Vec::new();

    for name in wb.sheet_names().to_vec() {
        let range = wb
            .worksheet_range(&name)
            .map_err(|e| format!("Error al leer la hoja '{name}': {e}"))?;

        // Filas como valores (los huecos al final se rellenan si faltan).
        let mut rows: Vec<Vec<Celda>> = Vec::new();
        for row in range.rows() {
            rows.push(row.iter().map(a_celda).collect());
        }
        if rows.is_empty() {
            continue;
        }

        // Headers: fila 0, o fila 1 si la 0 parece datos.
        let mut headers: Vec<String> = rows[0].iter().map(valor_a_header).collect();
        let mut data_start = 0usize;

        let es_header = headers
            .iter()
            .any(|h| !h.is_empty() && contiene_kw(h, KW_HEADER));

        if !es_header && rows.len() > 1 {
            let second_headers: Vec<String> = rows[1].iter().map(valor_a_header).collect();
            if second_headers
                .iter()
                .any(|h| !h.is_empty() && contiene_kw(h, KW_HEADER))
            {
                headers = second_headers;
                data_start = 2;
            } else {
                data_start = 1;
            }
        }

        // Buscar columnas por nombre (if/elif como Python: nombre gana).
        let mut col_nombre: Option<usize> = None;
        let mut col_costo: Option<usize> = None;
        let mut col_venta: Option<usize> = None;
        let mut col_categoria: Option<usize> = None;
        let mut col_stock: Option<usize> = None;

        for (i, h) in headers.iter().enumerate() {
            if h.is_empty() {
                continue;
            }
            if contiene_kw(h, KW_NOMBRE) {
                col_nombre = Some(i);
            } else if contiene_kw(h, KW_COSTO) {
                col_costo = Some(i);
            } else if contiene_kw(h, KW_VENTA) {
                col_venta = Some(i);
            } else if contiene_kw(h, KW_CATEGORIA) {
                col_categoria = Some(i);
            } else if contiene_kw(h, KW_STOCK) {
                col_stock = Some(i);
            }
        }

        // Si no encontramos nombre, detectar por posición (primer header sin dígitos).
        if col_nombre.is_none() {
            for (i, h) in headers.iter().enumerate() {
                if !h.is_empty() && !tiene_digito(&h.replace('$', "").replace(',', "")) {
                    col_nombre = Some(i);
                    break;
                }
            }
        }

        // Detectar categoría por nombre (redundante en Python, se espeja).
        if col_categoria.is_none() {
            for (i, h) in headers.iter().enumerate() {
                if !h.is_empty() && contiene_kw(h, KW_CATEGORIA) {
                    col_categoria = Some(i);
                    break;
                }
            }
        }

        // Detectar categoría por posición si no se encontró por nombre.
        if col_categoria.is_none() && col_nombre.is_some() {
            for (i, h) in headers.iter().enumerate() {
                if i != col_nombre.unwrap()
                    && !h.is_empty()
                    && !tiene_digito(&h.replace('$', "").replace(',', ""))
                    && h.chars().count() < 30
                {
                    col_categoria = Some(i);
                    break;
                }
            }
        }

        // Buscar col_nombre en filas de datos (solo si no se encontró).
        if col_nombre.is_none() {
            let inicio = if data_start == 1 { 0 } else { data_start };
            'scan_filas: for row in &rows[inicio..] {
                for (i, celda) in row.iter().enumerate() {
                    if col_nombre.is_none()
                        && !matches!(celda, Celda::Vacio | Celda::Numero(_) | Celda::Entero(_))
                    {
                        col_nombre = Some(i);
                        break 'scan_filas;
                    }
                }
            }
        }

        let Some(col_nombre) = col_nombre else {
            continue;
        };

        for row in &rows[data_start..] {
            if row.len() <= col_nombre {
                continue;
            }

            let nombre_celda = &row[col_nombre];
            let nombre_str = valor_a_texto(nombre_celda);
            if nombre_str.is_empty() {
                continue;
            }
            if es_categoria(&nombre_str) {
                continue;
            }

            // Precios (opcionales). Solo si hay columna de venta (comportamiento Python).
            let mut precio_venta = 0.0f64;
            let mut precio_costo = 0.0f64;
            if col_venta.is_some() {
                let venta_raw = col_venta
                    .filter(|i| *i < row.len())
                    .map(|i| valor_a_numero(&row[i]))
                    .unwrap_or(0.0);
                let costo_raw = col_costo
                    .filter(|i| *i < row.len())
                    .map(|i| valor_a_numero(&row[i]))
                    .unwrap_or(0.0);

                if venta_raw > 0.0 && costo_raw > 0.0 && venta_raw < costo_raw {
                    precio_venta = costo_raw;
                    precio_costo = venta_raw;
                } else {
                    precio_venta = venta_raw;
                    precio_costo = costo_raw;
                }
            }

            // Categoría.
            let categoria = col_categoria
                .filter(|i| *i < row.len())
                .map(|i| valor_a_texto(&row[i]))
                .filter(|c| !c.is_empty())
                .map(|c| c.to_uppercase())
                .unwrap_or_else(|| SIN_CATEGORIA.to_string());

            // Stock.
            let mut stock = 0i64;
            if let Some(i) = col_stock {
                if i < row.len() {
                    let texto = valor_a_texto(&row[i]).replace(',', "");
                    stock = texto.trim().parse::<f64>().map(|v| v as i64).unwrap_or(0);
                }
            }

            productos.push(ProductoCatalogo {
                nombre: limpiar_producto(&nombre_str),
                precio_costo: round2(precio_costo),
                precio_venta: round2(precio_venta),
                stock,
                categoria,
            });
        }
    }

    Ok(productos)
}