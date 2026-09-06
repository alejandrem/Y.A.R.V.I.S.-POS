// ============================================================
// diagnostico — Explicación accionable de por qué falló la detección.
//
// Cuando `detectar_mapeo` devuelve `None` o confianza baja, esto dice QUÉ
// vio el detector (renglones útiles, mejor coincidencia, pinta de
// familia C), para explicárselo al usuario en su idioma desde el backend
// Tauri. Barato: reusa las mismas hipótesis sobre la muestra filtrada.
// ============================================================

use serde::Serialize;

use super::hipotesis::{hipotesis, linea_cuadra, MIN_VALIDAS};
use super::muestra::muestrear;

/// Diagnóstico de la muestra para mensajes de error accionables (sin IA).
/// Cuando `detectar_mapeo` devuelve `None` o confianza baja, esto dice
/// QUÉ vio el detector, para explicárselo al usuario en su idioma.
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticoMuestra {
    /// Líneas útiles con ≥3 columnas encontradas en la muestra.
    pub lineas_utiles: usize,
    /// Líneas que el MEJOR mapeo con precio logró cuadrar.
    pub mejor_coincidencia: usize,
    /// Hay renglones con pinta de familia C (un solo importe por línea):
    /// el problema puede ser falta de productos repetidos, no de formato.
    pub pinta_familia_c: bool,
}

/// Explica por qué una muestra no dio un mapeo confiable. Barato: reusa
/// las mismas hipótesis sobre la muestra ya filtrada.
pub fn diagnosticar_muestra(lineas: &[&str]) -> DiagnosticoMuestra {
    let muestras = muestrear(lineas);
    let mut mejor = 0usize;
    let mut mejor_c = 0usize;
    for (cant_i, precio_i, total_i, producto) in hipotesis() {
        let validas = muestras
            .iter()
            .filter(|cols| linea_cuadra(cols, cant_i, precio_i, total_i, &producto) == Some(true))
            .count();
        if precio_i.is_some() {
            mejor = mejor.max(validas);
        } else {
            mejor_c = mejor_c.max(validas);
        }
    }
    DiagnosticoMuestra {
        lineas_utiles: muestras.len(),
        mejor_coincidencia: mejor,
        pinta_familia_c: mejor_c >= MIN_VALIDAS,
    }
}
