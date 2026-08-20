// ============================================================
// holt_winters.rs — Suavizado exponencial triple (aditivo) + bandas
// de confianza. El "Prophet local" de Y.A.R.V.I.S.
//
// Modelo:
//   nivel   : L_t   = α·(Y_t − S_{t−m}) + (1−α)·(L_{t−1} + T_{t−1})
//   tendencia: T_t  = β·(L_t − L_{t−1}) + (1−β)·T_{t−1}
//   estacional: S_t = γ·(Y_t − L_t) + (1−γ)·S_{t−m}
//   pronóstico  k..: F_t+k = L_t + k·T_t + S_{t−m+k}
//
// Intervalo de confianza al 95%: z·s·√k, con `s` = error estándar de
// los errores one-step-ahead del ajuste (la banda se abre con la
// raíz del horizonte, como hace Prophet).
//
// Notas de dominio (ventas):
//  - El pronóstico se fuerza a >= 0: no existen ventas negativas.
//  - Estacionalidad ADITIVA: el "plus" de un día alto es fijo en pesos.
//    Si el efecto crece con el nivel (tienda que escala), migrar a una
//    versión MULTIPLICATIVA (Y ≈ (L+T)·S) para eliminar sesgo.
// ============================================================
use serde::Serialize;

/// Un punto del pronóstico con su banda de confianza.
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct PuntoPrediccion {
    pub prediccion: f64,
    pub minimo: f64,
    pub maximo: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrediccionError {
    /// Menos puntos que el mínimo para estimar una tendencia (4).
    DatosInsuficientes(usize),
    /// Horizonte pedido en 0 (no pronosticar nada no tiene sentido).
    HorizonteInvalido(usize),
}

impl std::fmt::Display for PrediccionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatosInsuficientes(n) => write!(
                f,
                "Datos insuficientes para predecir (se recibieron {n} puntos)"
            ),
            Self::HorizonteInvalido(h) => write!(f, "Horizonte inválido: {h}"),
        }
    }
}

impl std::error::Error for PrediccionError {}

/// Valor z para una banda de confianza del 95%.
const Z_95: f64 = 1.96;
/// Malla de ajuste de α, β y γ: 7³ = 343 combos por ajuste.
/// Con cientos de días es barato; si algún día entra una serie diaria de
/// muchos años, se puede reducir la malla o fijar γ con menos valores.
const GRID: [f64; 7] = [0.05, 0.1, 0.2, 0.4, 0.6, 0.8, 0.99];
/// Mínimo de puntos para siquiera intentar una tendencia.
const MIN_PUNTOS_TENDENCIA: usize = 4;

/// Pronostica `horizonte` pasos desde el final de `serie`.
///
/// `periodo` es la estacionalidad (7 = semanal, 12 = mensual, 4 = trimestral).
/// Si `periodo` es menor a 2, la estacionalidad se ignora por completo y el
/// modelo queda en Holt-Lineal. Si `serie` no llega a 2 temporadas completas,
/// la estacionalidad también se desactiva para no inventar un patrón de ruido.
///
/// El pronóstico y la cota inferior se recortan a >= 0 (dominio de ventas).
pub fn predecir(
    serie: &[f64],
    periodo: usize,
    horizonte: usize,
) -> Result<Vec<PuntoPrediccion>, PrediccionError> {
    let n = serie.len();
    if n < MIN_PUNTOS_TENDENCIA {
        return Err(PrediccionError::DatosInsuficientes(n));
    }
    if horizonte == 0 {
        return Err(PrediccionError::HorizonteInvalido(horizonte));
    }

    // Estacionalidad activa solo si caben al menos 2 temporadas completas.
    let m = if periodo >= 2 && n >= 2 * periodo {
        periodo
    } else {
        0
    };
    let estacional = m > 0;

    let ajuste = fit(serie, m, estacional);

    // Error estándar de los errores one-step-ahead del ajuste.
    let s = if ajuste.n_err > 0 {
        (ajuste.sse / ajuste.n_err as f64).sqrt()
    } else {
        0.0
    };

    let mut pronostico = Vec::with_capacity(horizonte);
    for k in 1..=horizonte {
        let pos = n + k - 1; // posición futura (0-indexada)
        let sea = if estacional {
            ajuste.estacional[pos % m]
        } else {
            0.0
        };
        let valor = ajuste.nivel + k as f64 * ajuste.tendencia + sea;

        // Banda al 95%: crece con √k (la incertidumbre se acumula).
        let diff = if s.is_finite() {
            Z_95 * s * (k as f64).sqrt()
        } else {
            0.0
        };
        // Ventas no son negativas: recorta el punto y su cota inferior.
        let prediccion = valor.max(0.0);
        let minimo = (valor - diff).max(0.0);
        let maximo = (valor + diff).max(0.0);
        pronostico.push(PuntoPrediccion {
            prediccion,
            minimo,
            maximo,
        });
    }

    Ok(pronostico)
}

/// Estado final del suavizado + métricas de bondad de ajuste.
struct Ajuste {
    nivel: f64,
    tendencia: f64,
    estacional: Vec<f64>,
    sse: f64,
    n_err: usize,
}

/// Búsqueda de malla sobre α/β/γ minimizando el SSE one-step-ahead.
fn fit(serie: &[f64], m: usize, estacional: bool) -> Ajuste {
    let mut mejor: Option<Ajuste> = None;
    let mut mejor_sse = f64::INFINITY;

    if estacional {
        for &alpha in &GRID {
            for &beta in &GRID {
                for &gamma in &GRID {
                    let a = recursar(serie, m, alpha, beta, gamma);
                    if a.sse < mejor_sse {
                        mejor_sse = a.sse;
                        mejor = Some(a);
                    }
                }
            }
        }
    } else {
        for &alpha in &GRID {
            for &beta in &GRID {
                let a = recursar(serie, m, alpha, beta, 0.0);
                if a.sse < mejor_sse {
                    mejor_sse = a.sse;
                    mejor = Some(a);
                }
            }
        }
    }

    mejor.unwrap_or_else(|| recursar(serie, m, 0.5, 0.5, 0.0))
}

/// Recorre la serie aplicando el suavizado con los parámetros dados y
/// devuelve el estado final + SSE de los errores one-step-ahead.
fn recursar(serie: &[f64], m: usize, alpha: f64, beta: f64, gamma: f64) -> Ajuste {
    let n = serie.len();
    let estacional = m > 0;

    let (mut nivel, mut tendencia, mut estacional_idx, inicio) = if estacional {
        // Inicialización clásica: una temporada para nivel/estacionalidad,
        // la diferencia entre la 1ª y 2ª temporada para la tendencia.
        let primera: f64 = serie[0..m].iter().sum::<f64>() / m as f64;
        let segunda: f64 = serie[m..2 * m].iter().sum::<f64>() / m as f64;
        (
            primera,
            (segunda - primera) / m as f64,
            (0..m).map(|i| serie[i] - primera).collect::<Vec<_>>(),
            m,
        )
    } else {
        // Sin estacionalidad: nivel = primer valor (el pronóstico arranca
        // desde ahí hacia delante), tendencia = pendiente total de la serie.
        let nivel_inicial = serie[0];
        let pendiente = if n >= 2 {
            (serie[n - 1] - serie[0]) / (n as f64 - 1.0)
        } else {
            0.0
        };
        (nivel_inicial, pendiente, vec![], 1)
    };

    let mut sse = 0.0;
    let mut n_err = 0usize;

    for t in inicio..n {
        // Pronóstico one-step-ahead con el estado ANTES de la actualización.
        let yhat = nivel
            + tendencia
            + if estacional {
                estacional_idx[t % m]
            } else {
                0.0
            };
        let e = serie[t] - yhat;
        sse += e * e;
        n_err += 1;

        // Actualización del estado con el valor real.
        let sea_actual = if estacional {
            estacional_idx[t % m]
        } else {
            0.0
        };
        let nivel_nuevo = alpha * (serie[t] - sea_actual) + (1.0 - alpha) * (nivel + tendencia);
        let tendencia_nueva = beta * (nivel_nuevo - nivel) + (1.0 - beta) * tendencia;
        if estacional {
            estacional_idx[t % m] =
                gamma * (serie[t] - nivel_nuevo) + (1.0 - gamma) * estacional_idx[t % m];
        }
        nivel = nivel_nuevo;
        tendencia = tendencia_nueva;
    }

    Ajuste {
        nivel,
        tendencia,
        estacional: estacional_idx,
        sse,
        n_err,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mismos_banda(p: &[PuntoPrediccion]) {
        for punto in p {
            assert!(
                punto.minimo <= punto.prediccion + 1e-9 && punto.prediccion - 1e-9 <= punto.maximo,
                "banda corrupta: {}..{} no contiene {}",
                punto.minimo,
                punto.maximo,
                punto.prediccion
            );
        }
    }

    #[test]
    fn serie_constante_pronostica_constante() {
        let serie = vec![100.0; 40];
        let p = predecir(&serie, 7, 5).unwrap();
        assert_eq!(p.len(), 5);
        for punto in &p {
            assert!((punto.prediccion - 100.0).abs() < 1e-6);
        }
        mismos_banda(&p);
    }

    #[test]
    fn serie_lineal_sigue_la_tendencia() {
        // i*10 → el último punto es 400; el próximo paso debería ser ~410.
        let serie: Vec<f64> = (0..41).map(|i| i as f64 * 10.0).collect();
        let p = predecir(&serie, 0, 3).unwrap();
        assert!((p[0].prediccion - 410.0).abs() < 0.5);
        assert!((p[2].prediccion - 430.0).abs() < 0.5);
        mismos_banda(&p);
    }

    #[test]
    fn serie_con_estacionalidad_captura_el_patron() {
        // 100 + tendencia leve + patrón estacional de 4 posiciones.
        let temporadas = [0.0, 10.0, -5.0, 20.0];
        let n = 24; // 6 temporadas completas (m = 4)
        let serie: Vec<f64> = (0..n)
            .map(|i| 100.0 + (i as f64) * 0.5 + temporadas[i % 4])
            .collect();
        let p = predecir(&serie, 4, 8).unwrap();
        mismos_banda(&p);
        // El pronóstico debe conservar la oscilación estacional (rango real).
        let max = p.iter().map(|x| x.prediccion).fold(f64::MIN, f64::max);
        let min = p.iter().map(|x| x.prediccion).fold(f64::MAX, f64::min);
        assert!(
            max - min > 10.0,
            "la estacionalidad no se refleja: {max} - {min}"
        );
    }

    #[test]
    fn datos_insuficientes_es_error() {
        let r = predecir(&[1.0, 2.0, 3.0], 7, 5);
        assert_eq!(r, Err(PrediccionError::DatosInsuficientes(3)));
    }

    #[test]
    fn horizonte_cero_es_error() {
        let serie = vec![10.0; 10];
        assert_eq!(
            predecir(&serie, 7, 0),
            Err(PrediccionError::HorizonteInvalido(0))
        );
    }

    #[test]
    fn ruido_abre_la_banda() {
        // Serie con ruido determinista (LCG por turnos) → bandas > 0.
        let serie: Vec<f64> = (0..60)
            .map(|i| 50.0 + (((i * 7) % 11) as f64) * 0.7)
            .collect();
        let p = predecir(&serie, 7, 7).unwrap();
        mismos_banda(&p);
        for punto in &p {
            assert!(
                punto.maximo - punto.minimo > 0.0,
                "con ruido la banda no puede ser cero"
            );
        }
    }

    #[test]
    fn pronostico_negativo_se_recorta_a_cero() {
        // Serie que cae fuerte: la tendencia negativa llevaría el pronóstico
        // a valores negativos; deben recortarse a 0 (no hay ventas negativas).
        let serie: Vec<f64> = (0..40).map(|i| 100.0 - i as f64 * 10.0).collect();
        let p = predecir(&serie, 0, 5).unwrap();
        for punto in &p {
            assert!(
                punto.prediccion >= 0.0 && punto.minimo >= 0.0,
                "valores negativos: {:?}",
                punto
            );
        }
    }
}
