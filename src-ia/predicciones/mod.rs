// ============================================================
// predicciones — Matemáticas pesadas de predicción de Y.A.R.V.I.S.
//
// El "cerebro" que corre LOCAL en vez de Prophet. Está dividido en
// dos capas:
//
//   holt_winters.rs — matemática PURA: recibe una serie de valores
//     + periodo estacional + horizonte, y devuelve el pronóstico con
//     bandas de confianza (simula el `yhat(y_low,y_high)` de Prophet).
//     No conoce la DB ni el front.
//
//   ventas.rs — capa de DATOS: abre la DB (rusqlite), lee el histórico
//     de ventas completadas por día, alimenta la capa pura y devuelve
//     puntos con fecha real (`fecha`, `prediccion`, `minimo`, `maximo`).
//
// Decisiones:
//   * Ajusta sus propios parámetros (alpha/beta/gamma de Holt-Winters)
//     por búsqueda de malla minimizando el SSE one-step-ahead.
//   * Autodetección: con >= 2 temporadas completas usa estacionalidad
//     aditiva; con menos cae a Holt-Lineal (solo tendencia).
// ============================================================
pub mod holt_winters;
pub mod ventas;

pub use holt_winters::{predecir, PrediccionError, PuntoPrediccion};
pub use ventas::{predecir_ventas, PuntoConFecha};
