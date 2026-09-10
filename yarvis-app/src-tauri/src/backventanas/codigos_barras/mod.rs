// ============================================================
// codigos_barras — Soporte de codigos de barras por producto.
//
// Modulo COMPARTIDO de `backventanas` (no es backadmin ni backempleado):
// admin y empleado lo usan por igual. Todo lo relacionado con codigos
// de barras vive aqui: normalizacion, validacion y busqueda exacta.
//
// El escaner USB-HID se comporta como teclado: escribe el codigo y manda
// Enter. El frontend NO necesita drivers; el backend resuelve con busqueda
// EXACTA sobre `productos.codigo_barras` (indice unico parcial 0011).
//
// Reglas:
//   * `codigo_barras` siempre se guarda NORMALIZADO (trim + sin espacios/
//     guiones internos + mayusculas) o NULL si viene vacio.
//   * "" nunca se guarda como "" (colisionaria en el indice: "" != NULL).
//   * La unicidad la impone SQLite; aqui solo se traduce el error a un
//     mensaje que el tendero entiende.
// ============================================================

use crate::backventanas::auth::AuthState;
use crate::models::InventoryItem;
use sqlx::SqlitePool;

/// Normaliza un codigo opcional tal como viene del formulario/CSV/escaner.
/// - `None` -> `None`
/// - `"  "` / `""` -> `None` (nunca guardar cadena vacia)
/// - `"  750-123 456 "` -> `Some("750123456")`
/// - minusculas -> mayusculas (el EAN es numerico; el Code128 queda
///   case-insensitive a proposito para que el escaneo siempre matchee).
pub fn normalizar_codigo_barras(raw: Option<&str>) -> Option<String> {
    let t = raw?.trim();
    if t.is_empty() {
        return None;
    }
    let mut s = String::with_capacity(t.len());
    for c in t.chars() {
        if c.is_whitespace() || c == '-' {
            continue;
        }
        s.push(c.to_ascii_uppercase());
    }
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Variante para el codigo obligatorio que manda el escaner/lector.
pub fn normalizar_codigo_obligatorio(raw: &str) -> Option<String> {
    normalizar_codigo_barras(Some(raw))
}

/// Valida un codigo YA normalizado. Devuelve Err con mensaje para la UI.
pub fn validar_codigo_barras(codigo: &Option<String>) -> Result<(), String> {
    let Some(c) = codigo else { return Ok(()) };
    if c.len() > 64 {
        return Err("El código de barras no puede pasar de 64 caracteres.".into());
    }
    if !c.chars().all(|ch| ch.is_ascii_graphic()) {
        return Err("El código de barras tiene caracteres no válidos.".into());
    }
    Ok(())
}

/// Detecta si un error de sqlx es por duplicado de codigo de barras.
pub fn es_error_codigo_duplicado(msg: &str) -> bool {
    msg.contains("idx_productos_codigo_barras")
        || (msg.contains("UNIQUE constraint failed") && msg.contains("productos.codigo_barras"))
}

/// Traduce el error crudo de SQLite a mensaje para el tendero.
pub fn mensaje_error_codigo(e: impl ToString) -> String {
    let msg = e.to_string();
    if es_error_codigo_duplicado(&msg) {
        "El código de barras ya está registrado en otro producto.".to_string()
    } else {
        msg
    }
}

type FilaProducto = (
    Option<i32>,
    String,
    Option<String>,
    i64,
    i64,
    f64,
    f64,
    f64,
    Option<String>,
    Option<String>,
);

fn fila_a_item(row: FilaProducto) -> InventoryItem {
    InventoryItem {
        id: row.0,
        nombre: row.1,
        descripcion: row.2,
        precio_costo: crate::dinero::a_pesos(row.3),
        precio_venta: crate::dinero::a_pesos(row.4),
        stock: row.5,
        stock_minimo: row.6,
        vendido: row.7,
        codigo_barras: row.8,
        categoria: row.9,
    }
}

/// Nucleo testeable: busca un producto por su codigo EXACTO (normalizado).
/// `None` = no existe (el frontend muestra "producto no registrado" y ofrece
/// crearlo). Nunca hace LIKE: el escaner debe ser determinista.
pub async fn get_product_by_barcode_impl(
    pool: &SqlitePool,
    codigo: &str,
) -> Result<Option<InventoryItem>, String> {
    let normalizado = normalizar_codigo_obligatorio(codigo);
    let Some(n) = normalizado else {
        return Err("Código de barras vacío.".into());
    };
    let row: Option<FilaProducto> = sqlx::query_as(
        "SELECT id, nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, vendido, codigo_barras, categoria FROM productos WHERE codigo_barras = ? LIMIT 1",
    )
    .bind(&n)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row.map(fila_a_item))
}

/// Comando Tauri para el lector: operario (admin o empleado) puede escanear.
/// El frontend lo llama con el texto que el escaner "tecleo" + Enter.
#[tauri::command]
pub async fn get_product_by_barcode(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    codigo: String,
) -> Result<Option<InventoryItem>, String> {
    auth.require_operator()?;
    get_product_by_barcode_impl(&*state, &codigo).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normaliza_a_none_si_vacio() {
        assert_eq!(normalizar_codigo_barras(None), None);
        assert_eq!(normalizar_codigo_barras(Some("")), None);
        assert_eq!(normalizar_codigo_barras(Some("   ")), None);
        assert_eq!(normalizar_codigo_barras(Some(" - ")), None);
    }

    #[test]
    fn normaliza_quitando_espacios_guiones_y_mayusculas() {
        assert_eq!(
            normalizar_codigo_barras(Some("  750-123 456 ")),
            Some("750123456".to_string())
        );
        assert_eq!(
            normalizar_codigo_barras(Some("unico123")),
            Some("UNICO123".to_string())
        );
    }

    #[test]
    fn valida_longitud_y_caracteres() {
        assert!(validar_codigo_barras(&None).is_ok());
        assert!(validar_codigo_barras(&Some("7501234567890".into())).is_ok());
        assert!(validar_codigo_barras(&Some("a".repeat(65))).is_err());
        assert!(validar_codigo_barras(&Some("código😀".into())).is_err());
    }

    #[test]
    fn detecta_duplicado_por_mensaje_sqlite() {
        assert!(es_error_codigo_duplicado(
            "UNIQUE constraint failed: productos.codigo_barras"
        ));
        assert!(!es_error_codigo_duplicado("no such table: productos"));
    }
}
