// ============================================================
// ventas.rs — Capa de datos del cerebro de predicciones.
//
// Conecta el núcleo puro (holt_winters) con la DB SQLite de la app:
// lee el histórico de ventas completadas (agrupadas por día), llena
// los días sin venta con 0 y devuelve el pronóstico con fechas reales.
//
// La función pública `predecir_ventas` abarca el flujo completo que
// usan los comandos Tauri; los helpers internos se dejan separados
// para poder testearlos con una DB en memoria (determinista).
// ============================================================
use std::path::Path;

use chrono::{Duration, NaiveDate};
use rusqlite::Connection;
use serde::Serialize;

use super::holt_winters::{predecir, PrediccionError};

/// Un punto del pronóstico con fecha real (`YYYY-MM-DD`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PuntoConFecha {
    pub fecha: String,
    pub prediccion: f64,
    pub minimo: f64,
    pub maximo: f64,
}

/// Estacionalidad diaria: las ventas se mueven en ciclos semanales.
const PERIODO_ESTACIONAL: usize = 7;
/// Cuánta historia diaria se le da al modelo como máximo (últimos días).
const MAX_DIAS_HISTORIA: i64 = 365;
/// Mínimos puntos para estimar siquiera una tendencia (igual que el núcleo).
const MIN_DIAS_HISTORIA: usize = 4;

/// Predice `horizonte` días de ventas a partir del histórico de la DB.
pub fn predecir_ventas(ruta_db: &Path, horizonte: usize) -> Result<Vec<PuntoConFecha>, String> {
    let conn =
        Connection::open(ruta_db).map_err(|e| format!("No se pudo abrir la base de datos: {e}"))?;
    predecir_desde_conn(&conn, horizonte)
}

/// Flujo completo (consulta + suavizado + fechas). Separado de
/// `predecir_ventas` para poder testearlo con una DB en memoria.
pub fn predecir_desde_conn(
    conn: &Connection,
    horizonte: usize,
) -> Result<Vec<PuntoConFecha>, String> {
    if horizonte == 0 {
        return Err("El horizonte de predicción debe ser mayor a 0".to_string());
    }

    // Histórico de ventas completadas, agregado por día (YYYY-MM-DD → total).
    let mut stmt = conn
        .prepare(
            r#"SELECT date(fecha) as dia, COALESCE(SUM(total), 0) as total
               FROM ventas
               WHERE estado = 'completada'
               GROUP BY date(fecha)
               ORDER BY dia ASC"#,
        )
        .map_err(|e| format!("No se pudieron leer las ventas: {e}"))?;

    let filas: Vec<(NaiveDate, f64)> = stmt
        .query_map([], |row| {
            let dia: String = row.get("dia")?;
            let total: f64 = row.get("total")?;
            let fecha = NaiveDate::parse_from_str(&dia, "%Y-%m-%d").map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok((fecha, total))
        })
        .map_err(|e| format!("No se pudieron leer las ventas: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("No se pudieron leer las ventas: {e}"))?;

    if filas.is_empty() {
        return Err("No hay ventas registradas para predecir.".to_string());
    }

    // Serie densa: del primer día con ventas (o los últimos MAX_DIAS_HISTORIA)
    // hasta el último día con datos, rellenando los huecos con 0.
    let ultimo = filas.iter().map(|(d, _)| *d).max().unwrap();
    let primer_dia = filas.iter().map(|(d, _)| *d).min().unwrap();
    let inicio = primer_dia.max(ultimo - Duration::days(MAX_DIAS_HISTORIA - 1));

    let mapa: std::collections::HashMap<NaiveDate, f64> = filas.into_iter().collect();
    let mut serie: Vec<f64> = Vec::new();
    let mut dia = inicio;
    while dia <= ultimo {
        let total = mapa.get(&dia).copied().unwrap_or(0.0);
        serie.push(total);
        dia = dia + Duration::days(1);
    }

    if serie.len() < MIN_DIAS_HISTORIA {
        return Err(format!(
            "Datos insuficientes para predecir (solo se tienen {} días de ventas).",
            serie.len()
        ));
    }

    let pronostico = predecir(&serie, PERIODO_ESTACIONAL, horizonte)
        .map_err(|e: PrediccionError| e.to_string())?;

    let mut puntos = Vec::with_capacity(horizonte);
    for (k, punto) in pronostico.into_iter().enumerate() {
        let fecha = ultimo + Duration::days(k as i64 + 1);
        puntos.push(PuntoConFecha {
            fecha: fecha.format("%Y-%m-%d").to_string(),
            prediccion: punto.prediccion,
            minimo: punto.minimo,
            maximo: punto.maximo,
        });
    }

    Ok(puntos)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conexion_con_datos(dias: &[(&str, f64)]) -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"CREATE TABLE ventas (
                id INTEGER PRIMARY KEY,
                fecha TEXT NOT NULL,
                total REAL NOT NULL,
                estado TEXT NOT NULL DEFAULT 'completada'
            );"#,
        )
        .unwrap();
        for (fecha, total) in dias {
            conn.execute(
                "INSERT INTO ventas (fecha, total, estado) VALUES (?1, ?2, 'completada')",
                rusqlite::params![fecha, total],
            )
            .unwrap();
        }
        conn
    }

    #[test]
    fn agrupa_por_dia_suma_completadas_y_excluye_canceladas() {
        let conn = conexion_con_datos(&[
            ("2026-07-01 09:00:00", 100.0),
            ("2026-07-01 18:00:00", 250.0),
            ("2026-07-02 12:00:00", 400.0),
            ("2026-07-03 12:00:00", 999.0),
            ("2026-07-04 12:00:00", 500.0),
        ]);
        conn.execute(
            "UPDATE ventas SET estado = 'cancelada' WHERE fecha = '2026-07-03 12:00:00'",
            [],
        )
        .unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT date(fecha), SUM(total) FROM ventas WHERE estado = 'completada' GROUP BY date(fecha) ORDER BY date(fecha)",
            )
            .unwrap();
        let resumen: Vec<(String, f64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(
            resumen,
            vec![
                ("2026-07-01".to_string(), 350.0),
                ("2026-07-02".to_string(), 400.0),
                ("2026-07-04".to_string(), 500.0),
            ]
        );

        // La fecha del último dato válido es 07-04; la venta cancelada del
        // 07-03 no debe convertirse en un día con ventas.
        let puntos = predecir_desde_conn(&conn, 2).unwrap();
        assert_eq!(
            puntos.iter().map(|p| p.fecha.as_str()).collect::<Vec<_>>(),
            ["2026-07-05", "2026-07-06"]
        );
    }

    #[test]
    fn predice_con_historico_real() {
        // 3 semanas de ventas con un patrón de fin de semana alto.
        let mut dias = Vec::new();
        for i in 0..21 {
            let total = if i % 7 == 5 || i % 7 == 6 {
                1000.0
            } else {
                300.0
            };
            dias.push((format!("2026-07-{:02}", 1 + i), total));
        }
        let conn = conexion_con_datos(
            &dias
                .iter()
                .map(|(d, v)| (d.as_str(), *v))
                .collect::<Vec<_>>(),
        );

        let puntos = predecir_desde_conn(&conn, 7).unwrap();
        assert_eq!(puntos.len(), 7);
        // Las fechas arrancan el día siguiente al último con datos.
        assert_eq!(puntos[0].fecha, "2026-07-22");
        assert_eq!(puntos[6].fecha, "2026-07-28");
        // La estacionalidad de fin de semana debe aparecer en el pronóstico.
        let variacion = puntos.iter().map(|p| p.prediccion).fold(f64::MIN, f64::max)
            - puntos.iter().map(|p| p.prediccion).fold(f64::MAX, f64::min);
        assert!(
            variacion > 100.0,
            "no se capturó el patrón semanal: {variacion}"
        );
        for punto in &puntos {
            assert!(punto.minimo <= punto.prediccion + 1e-9);
            assert!(punto.prediccion - 1e-9 <= punto.maximo);
            assert!(punto.minimo >= 0.0 && punto.prediccion >= 0.0);
        }
    }

    #[test]
    fn huecos_dias_sin_venta_se_rellenan_con_cero() {
        let conn = conexion_con_datos(&[
            ("2026-07-01", 500.0),
            ("2026-07-03", 700.0), // 07-02 no existe → debe contar como 0
            ("2026-07-04", 300.0),
            ("2026-07-05", 900.0),
            ("2026-07-06", 400.0),
        ]);
        let puntos = predecir_desde_conn(&conn, 3).unwrap();
        assert_eq!(puntos[0].fecha, "2026-07-07");
        assert_eq!(puntos.len(), 3);
    }

    #[test]
    fn sin_ventas_es_error() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"CREATE TABLE ventas (
                id INTEGER PRIMARY KEY,
                fecha TEXT NOT NULL,
                total REAL NOT NULL,
                estado TEXT NOT NULL DEFAULT 'completada'
            );"#,
        )
        .unwrap();
        let err = predecir_desde_conn(&conn, 7).unwrap_err();
        assert!(err.contains("No hay ventas"), "{err}");
    }

    #[test]
    fn con_muy_poca_historia_es_error() {
        let conn = conexion_con_datos(&[("2026-07-01", 500.0), ("2026-07-02", 700.0)]);
        let err = predecir_desde_conn(&conn, 7).unwrap_err();
        assert!(err.contains("Datos insuficientes"), "{err}");
    }

    #[test]
    fn horizonte_cero_es_error() {
        let conn = conexion_con_datos(&[("2026-07-01", 500.0), ("2026-07-02", 700.0)]);
        assert!(predecir_desde_conn(&conn, 0).is_err());
    }
}
