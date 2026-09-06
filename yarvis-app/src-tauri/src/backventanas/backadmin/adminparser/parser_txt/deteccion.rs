// ============================================================
// deteccion — Detección estadística del mapeo de columnas, SIN IA.
//
// Reemplaza a la calibración con el modelo local (Qwen 1.7B): se toma
// una muestra determinista y ESPACIADA de archivos de la carpeta y se
// ensayan hipótesis de mapeo que deben verificar la ecuación
// `cantidad × precio ≈ total` en las líneas reales. El mapeo ganador
// está demostrado, no "adivinado" por un modelo.
//
// Los errores hablan en lenguaje normal (con números de la muestra y el
// siguiente paso sugerido) gracias a `diagnosticar_muestra`.
// ============================================================

use crate::backventanas::auth::AuthState;
use std::fs;

/// Confianza mínima exigida para aceptar la detección (fracción de
/// líneas de la muestra cuya ecuación cantidad×precio≈total cuadró).
const UMBRAL_CONFIANZA_MAPEO: f64 = 0.55;

#[tauri::command]
pub fn detectar_mapeo_estadistico(
    auth: tauri::State<'_, AuthState>,
    carpeta: String,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;

    let archivos = src_ia::cerebro::parseador_masivo::obtener_archivos_txt(&carpeta);
    if archivos.is_empty() {
        return Err("No se encontraron archivos .txt en la carpeta".to_string());
    }

    // Muestra determinista: hasta 15 archivos ESPACIADOS en la lista
    // (alfabética ≈ cronológica en la práctica), así cubrimos distintas
    // épocas del lote sin depender del azar. Tope global de líneas.
    const MAX_ARCHIVOS_MUESTRA: usize = 15;
    const MAX_LINEAS_TOTAL: usize = 900;
    const MAX_LINEAS_POR_ARCHIVO: usize = 60;

    let paso = (archivos.len() / MAX_ARCHIVOS_MUESTRA).max(1);
    let mut lineas: Vec<String> = Vec::new();
    let mut archivos_muestra = 0usize;
    for archivo in archivos.iter().step_by(paso) {
        let Ok(bytes) = fs::read(archivo) else {
            continue;
        };
        archivos_muestra += 1;
        let texto = String::from_utf8_lossy(&bytes);
        lineas.extend(texto.lines().take(MAX_LINEAS_POR_ARCHIVO).map(str::to_string));
        if lineas.len() >= MAX_LINEAS_TOTAL || archivos_muestra >= MAX_ARCHIVOS_MUESTRA {
            break;
        }
    }

    let refs: Vec<&str> = lineas.iter().map(String::as_str).collect();
    match src_ia::cerebro::analizador_tickets::detectar_mapeo(&refs) {
        Some(d) if d.confianza >= UMBRAL_CONFIANZA_MAPEO => Ok(serde_json::json!({
            "status": "ok",
            "mapeo": d.mapeo,
            "confianza": d.confianza,
            "lineas_evaluadas": d.lineas_evaluadas,
            "lineas_validas": d.lineas_validas,
            "archivos_muestra": archivos_muestra,
        })),
        Some(d) => {
            let pct = (d.confianza * 100.0).round() as i64;
            let minimo = (UMBRAL_CONFIANZA_MAPEO * 100.0).round() as i64;
            Err(format!(
                "Tus tickets parecen venir de más de un formato distinto: solo el {pct}% de los \
                 renglones cuadran con un solo formato y necesitamos al menos {minimo}% \
                 (revisamos {archivos_muestra} archivos).\n\n\
                 Separa los tickets por tienda o impresora en carpetas distintas e importa cada \
                 carpeta por separado. Si todos son de la misma tienda, quita fotos o archivos \
                 que no sean tickets de venta."
            ))
        }
        None => {
            let diag = src_ia::cerebro::analizador_tickets::diagnosticar_muestra(&refs);
            Err(mensaje_sin_formato(&diag, archivos_muestra))
        }
    }
}

/// Explica en lenguaje normal por qué no se identificó el formato,
/// con números de la muestra y el siguiente paso sugerido.
fn mensaje_sin_formato(
    diag: &src_ia::cerebro::analizador_tickets::DiagnosticoMuestra,
    archivos_muestra: usize,
) -> String {
    if diag.lineas_utiles < 3 {
        return format!(
            "No encontramos renglones de productos en tus tickets (revisamos {archivos_muestra} \
             archivos y solo vimos {} renglones con datos; necesitamos al menos 3).\n\n\
             Revisa que la carpeta tenga tickets de venta en .txt con renglones tipo \
             \"2 COCA 25.00 50.00\" (cantidad, producto, precio y total por renglón).",
            diag.lineas_utiles
        );
    }
    let pct = if diag.lineas_utiles > 0 {
        (diag.mejor_coincidencia as f64 / diag.lineas_utiles as f64 * 100.0).round() as i64
    } else {
        0
    };
    let mut msg = format!(
        "No pudimos identificar el formato de columnas de tus tickets: el mejor candidato solo \
         explica {} de {} renglones ({pct}%).\n\n\
         Para leerlos necesitamos renglones donde cantidad × precio ≈ total \
         (ej. 2 × $25.00 = $50.00). Usa tickets de una sola tienda e impresora a la vez.",
        diag.mejor_coincidencia, diag.lineas_utiles
    );
    if diag.pinta_familia_c {
        msg.push_str(
            "\n\nTus renglones traen un solo importe (cantidad + producto + total, sin precio \
             unitario): para ese formato necesitamos ver los mismos productos repetidos varias \
             veces al mismo precio. Agrega más tickets del mismo periodo e inténtalo de nuevo.",
        );
    }
    msg
}
