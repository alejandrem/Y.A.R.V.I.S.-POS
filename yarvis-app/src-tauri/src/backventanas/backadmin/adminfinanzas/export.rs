use crate::backventanas::auth::AuthState;
use crate::dinero::{a_pesos, centavos_f64_a_i64};
use chrono::NaiveDate;
use sqlx::{Row, SqlitePool};

// ============================================================================
// EXPORTACIÓN DE REPORTES FINANCIEROS (implementación real, sin dependencias)
// - CSV de gastos: texto RFC4180 con escape de comas/comillas/saltos.
// - Balance PDF: PDF 1.4 mínimo generado a mano (Helvetica, 1 página,
//   solo texto). Sin printpdf para no inflar el binario.
// ============================================================================

fn validar_rango(fecha_inicio: &str, fecha_fin: &str) -> Result<(), String> {
    let ini = NaiveDate::parse_from_str(fecha_inicio, "%Y-%m-%d")
        .map_err(|_| format!("Fecha de inicio inválida: '{fecha_inicio}' (se espera YYYY-MM-DD)"))?;
    let fin = NaiveDate::parse_from_str(fecha_fin, "%Y-%m-%d")
        .map_err(|_| format!("Fecha de fin inválida: '{fecha_fin}' (se espera YYYY-MM-DD)"))?;
    if fin < ini {
        return Err("La fecha de fin es anterior a la de inicio".to_string());
    }
    Ok(())
}

/// Escapa un campo CSV según RFC4180.
pub fn escapar_csv(campo: &str) -> String {
    if campo.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", campo.replace('"', "\"\""))
    } else {
        campo.to_string()
    }
}

fn fila_csv(campos: &[String]) -> String {
    campos.iter().map(|c| escapar_csv(c)).collect::<Vec<_>>().join(",")
}

/// Fila ya leída para el CSV de gastos (montos en pesos).
pub struct FilaGastoCsv {
    pub fecha: String,
    pub nombre: String,
    pub tipo: String,
    pub categoria: String,
    pub proyectado: f64,
    pub real: f64,
    pub frecuencia: String,
    pub estado: String,
    pub folio: String,
}

/// Núcleo testeable: arma el CSV a partir de filas ya leídas.
pub fn construir_csv_gastos(filas: &[FilaGastoCsv]) -> String {
    let mut out = String::from(
        "Fecha,Nombre,Tipo,Categoria,Monto Proyectado,Monto Real,Frecuencia,Estado,Folio\n",
    );
    for f in filas {
        out.push_str(&fila_csv(&[
            f.fecha.clone(),
            f.nombre.clone(),
            f.tipo.clone(),
            f.categoria.clone(),
            format!("{:.2}", f.proyectado),
            format!("{:.2}", f.real),
            f.frecuencia.clone(),
            f.estado.clone(),
            f.folio.clone(),
        ]));
        out.push('\n');
    }
    out
}

/// Exportar Gastos a CSV (filtra por fecha_inicio del gasto en el rango).
#[tauri::command]
pub async fn exportar_gastos_csv(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    fecha_inicio: String,
    fecha_fin: String,
) -> Result<String, String> {
    auth.require_admin()?;
    validar_rango(&fecha_inicio, &fecha_fin)?;
    exportar_gastos_csv_impl(&state, &fecha_inicio, &fecha_fin).await
}

pub async fn exportar_gastos_csv_impl(
    pool: &SqlitePool,
    fecha_inicio: &str,
    fecha_fin: &str,
) -> Result<String, String> {
    let rows = sqlx::query(
        "SELECT fecha_inicio, nombre, tipo, categoria, monto_proyectado, monto_real,
                frecuencia, estado_pago, COALESCE(folio_comprobante,'') as folio
         FROM gastos_recurrentes
         WHERE date(fecha_inicio) BETWEEN date(?1) AND date(?2)
         ORDER BY date(fecha_inicio) ASC",
    )
    .bind(fecha_inicio)
    .bind(fecha_fin)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut filas = Vec::with_capacity(rows.len());
    for r in rows {
        let proy_c: i64 = r.try_get("monto_proyectado").unwrap_or(0);
        let real_c: i64 = r.try_get("monto_real").unwrap_or(0);
        filas.push(FilaGastoCsv {
            fecha: r.try_get::<String, _>("fecha_inicio").unwrap_or_default(),
            nombre: r.try_get::<String, _>("nombre").unwrap_or_default(),
            tipo: r.try_get::<String, _>("tipo").unwrap_or_default(),
            categoria: r.try_get::<String, _>("categoria").unwrap_or_default(),
            proyectado: a_pesos(proy_c),
            real: a_pesos(real_c),
            frecuencia: r.try_get::<String, _>("frecuencia").unwrap_or_default(),
            estado: r.try_get::<String, _>("estado_pago").unwrap_or_default(),
            folio: r.try_get::<String, _>("folio").unwrap_or_default(),
        });
    }
    Ok(construir_csv_gastos(&filas))
}

// ── Balance PDF ─────────────────────────────────────────────────────────────

/// Escapa texto para un literal PDF entre paréntesis (latin1 básico).
fn escapar_pdf(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '(' | ')' | '\\' => format!("\\{c}"),
            c if (c as u32) < 32 || (c as u32) > 126 => "?".to_string(),
            c => c.to_string(),
        })
        .collect()
}

/// Genera un PDF 1.4 de una página con las líneas dadas (Helvetica 11pt).
/// Retorna los bytes listos para guardar/descargar.
pub fn generar_pdf_simple(titulo: &str, lineas: &[String]) -> Vec<u8> {
    // Contenido: título 16pt + líneas 11pt, con salto de 16px.
    let mut contenido = String::from("BT /F1 16 Tf 50 800 Td (");
    contenido.push_str(&escapar_pdf(titulo));
    contenido.push_str(") Tj ET\n");
    let mut y = 775;
    for linea in lineas {
        contenido.push_str(&format!(
            "BT /F1 11 Tf 50 {y} Td ({}) Tj ET\n",
            escapar_pdf(linea)
        ));
        y -= 16;
        if y < 40 {
            break; // una página: se recorta, no se rompe el PDF
        }
    }
    let stream = contenido.into_bytes();

    let mut pdf: Vec<u8> = Vec::new();
    let mut offsets: Vec<usize> = Vec::new();
    let push = |pdf: &mut Vec<u8>, s: &str| pdf.extend_from_slice(s.as_bytes());

    push(&mut pdf, "%PDF-1.4\n");
    // 1: catálogo
    offsets.push(pdf.len());
    push(&mut pdf, "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    // 2: páginas
    offsets.push(pdf.len());
    push(&mut pdf, "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    // 3: página
    offsets.push(pdf.len());
    push(
        &mut pdf,
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>\nendobj\n",
    );
    // 4: fuente
    offsets.push(pdf.len());
    push(
        &mut pdf,
        "4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
    );
    // 5: contenido
    offsets.push(pdf.len());
    push(
        &mut pdf,
        &format!("5 0 obj\n<< /Length {} >>\nstream\n", stream.len()),
    );
    pdf.extend_from_slice(&stream);
    push(&mut pdf, "\nendstream\nendobj\n");

    let xref_pos = pdf.len();
    push(&mut pdf, &format!("xref\n0 {}\n", offsets.len() + 1));
    push(&mut pdf, "0000000000 65535 f \n");
    for off in &offsets {
        push(&mut pdf, &format!("{off:010} 00000 n \n"));
    }
    push(
        &mut pdf,
        &format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_pos}\n%%EOF",
            offsets.len() + 1
        ),
    );
    pdf
}

/// Exportar Balance General a PDF (resumen del periodo + cortes Z).
#[tauri::command]
pub async fn exportar_balance_pdf(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    fecha_inicio: String,
    fecha_fin: String,
) -> Result<Vec<u8>, String> {
    auth.require_admin()?;
    validar_rango(&fecha_inicio, &fecha_fin)?;
    exportar_balance_pdf_impl(&state, &fecha_inicio, &fecha_fin).await
}

pub async fn exportar_balance_pdf_impl(
    pool: &SqlitePool,
    fecha_inicio: &str,
    fecha_fin: &str,
) -> Result<Vec<u8>, String> {
    let ventas_c: Option<f64> = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total),0) FROM ventas WHERE estado='completada' AND date(fecha) BETWEEN date(?1) AND date(?2)",
    )
    .bind(fecha_inicio)
    .bind(fecha_fin)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    let gastos_c: Option<f64> = sqlx::query_scalar(
        "SELECT COALESCE(SUM(monto_real),0) FROM gastos_recurrentes WHERE date(fecha_inicio) BETWEEN date(?1) AND date(?2)",
    )
    .bind(fecha_inicio)
    .bind(fecha_fin)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    let cortes_z: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM cortes_caja WHERE tipo_corte='Z' AND date(fecha_apertura) BETWEEN date(?1) AND date(?2)",
    )
    .bind(fecha_inicio)
    .bind(fecha_fin)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let ventas = a_pesos(centavos_f64_a_i64(ventas_c.unwrap_or(0.0)));
    let gastos = a_pesos(centavos_f64_a_i64(gastos_c.unwrap_or(0.0)));
    let utilidad = ventas - gastos;
    let hoy = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let lineas = vec![
        format!("Periodo: {fecha_inicio} a {fecha_fin}"),
        format!("Generado: {hoy} - YARVIS POS"),
        String::from(" "),
        format!("Ventas totales: ${ventas:.2}"),
        format!("Gastos reales: ${gastos:.2}"),
        format!("Utilidad neta: ${utilidad:.2}"),
        format!("Cortes Z: {cortes_z}"),
    ];
    Ok(generar_pdf_simple("Balance General YARVIS", &lineas))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_escapa_comas_y_comillas() {
        assert_eq!(escapar_csv("simple"), "simple");
        assert_eq!(escapar_csv("con, coma"), "\"con, coma\"");
        assert_eq!(escapar_csv("dice \"hola\""), "\"dice \"\"hola\"\"\"");
        assert_eq!(escapar_csv("linea\nsalto"), "\"linea\nsalto\"");
    }

    #[test]
    fn csv_construye_encabezado_y_filas() {
        let csv = construir_csv_gastos(&[FilaGastoCsv {
            fecha: "2026-01-05".to_string(),
            nombre: "Renta, local".to_string(),
            tipo: "fijo".to_string(),
            categoria: "operacion".to_string(),
            proyectado: 5000.0,
            real: 5000.0,
            frecuencia: "mensual".to_string(),
            estado: "pagado".to_string(),
            folio: "F-1".to_string(),
        }]);
        assert!(csv.starts_with("Fecha,Nombre,Tipo,Categoria,"));
        assert!(csv.contains("\"Renta, local\""));
        assert!(csv.contains("5000.00"));
    }

    #[test]
    fn pdf_generado_es_valido_minimo() {
        let bytes = generar_pdf_simple("Titulo", &["linea 1".to_string(), "linea (2)".to_string()]);
        assert!(bytes.starts_with(b"%PDF-1.4"));
        assert!(bytes.windows(5).any(|w| w == b"%%EOF"));
        // El paréntesis de la línea 2 debe ir escapado, no romper el stream.
        assert!(bytes.windows(5).any(|w| w == b"\\(2\\)"));
    }

    #[test]
    fn rango_invertido_se_rechaza() {
        assert!(validar_rango("2026-02-01", "2026-01-01").is_err());
        assert!(validar_rango("2026-13-40", "2026-01-01").is_err());
        assert!(validar_rango("2026-01-01", "2026-01-31").is_ok());
    }
}
