use std::collections::HashMap;

use super::inventario::{cargar_inventario, ProductoDb, ProductoInventario};
use super::similitud::{cosine_similarity, normalizar, Embedder};

/// Un producto parseado vinculado (tipo_match: "exacto" | "embedding").
#[derive(Debug, Clone, serde::Serialize)]
pub struct Match {
    pub producto_parseado: serde_json::Value,
    pub producto_db: ProductoDb,
    pub tipo_match: String,
    pub score: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SinVincular {
    pub producto_parseado: serde_json::Value,
    pub razon: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EstadisticasVinculacion {
    pub total_parseados: usize,
    pub exactos: usize,
    pub por_embedding: usize,
    pub sin_vincular: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResultadoVinculacion {
    pub vinculados: Vec<Match>,
    pub sin_vincular: Vec<SinVincular>,
    pub estadisticas: EstadisticasVinculacion,
}

pub fn vincular_con_inventario(
    productos_parseados: &[serde_json::Value],
    db_path: &str,
    umbral_similitud: f64,
    embedder: Option<&dyn Embedder>,
) -> ResultadoVinculacion {
    let inventario = cargar_inventario(db_path);

    // Indexar por nombre normalizado para búsqueda exacta O(1).
    // Coincide con Python: si dos productos comparten nombre normalizado,
    // el último gana (asignación directa, no or_insert).
    let mut indice_nombre: HashMap<String, &ProductoInventario> = HashMap::new();
    for prod in &inventario {
        indice_nombre.insert(prod.nombre_norm.clone(), prod);
    }

    // Inventario con embedding para el match por similitud.
    let inventario_con_embedding: Vec<&ProductoInventario> =
        inventario.iter().filter(|p| p.embedding.is_some()).collect();

    let mut vinculados = Vec::new();
    let mut sin_vincular = Vec::new();
    let mut exactos = 0usize;
    let mut por_embedding = 0usize;

    for parseado in productos_parseados {
        let nombre_parseado = parseado
            .get("producto")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let nombre_norm = normalizar(nombre_parseado);

        // 1. Coincidencia exacta.
        if let Some(prod_db) = indice_nombre.get(&nombre_norm) {
            vinculados.push(Match {
                producto_parseado: parseado.clone(),
                producto_db: ProductoDb {
                    id: prod_db.id,
                    nombre: prod_db.nombre.clone(),
                    precio_venta: prod_db.precio_venta,
                },
                tipo_match: "exacto".to_string(),
                score: 1.0,
            });
            exactos += 1;
            continue;
        }

        // 2. Búsqueda por embedding (solo si hay embedder disponible).
        if let Some(emb) = embedder {
            if !inventario_con_embedding.is_empty() {
                if let Some(emb_parseado) = emb.texto_a_embedding(nombre_parseado) {
                    let mut mejor_score = 0.0f64;
                    let mut mejor_match: Option<&ProductoInventario> = None;

                    for prod_inv in &inventario_con_embedding {
                        let score = cosine_similarity(
                            &emb_parseado,
                            prod_inv.embedding.as_deref().unwrap_or(&[]),
                        );
                        if score > mejor_score {
                            mejor_score = score;
                            mejor_match = Some(prod_inv);
                        }
                    }

                    if let Some(m) = mejor_match {
                        if mejor_score >= umbral_similitud {
                            vinculados.push(Match {
                                producto_parseado: parseado.clone(),
                                producto_db: ProductoDb {
                                    id: m.id,
                                    nombre: m.nombre.clone(),
                                    precio_venta: m.precio_venta,
                                },
                                tipo_match: "embedding".to_string(),
                                score: (mejor_score * 10000.0).round() / 10000.0,
                            });
                            por_embedding += 1;
                            continue;
                        }
                    }
                }
            }
        }

        // 3. Sin vincular.
        sin_vincular.push(SinVincular {
            producto_parseado: parseado.clone(),
            razon: "Sin coincidencia en inventario".to_string(),
        });
    }

    ResultadoVinculacion {
        vinculados,
        estadisticas: EstadisticasVinculacion {
            total_parseados: productos_parseados.len(),
            exactos,
            por_embedding,
            sin_vincular: sin_vincular.len(),
        },
        sin_vincular,
    }
}
