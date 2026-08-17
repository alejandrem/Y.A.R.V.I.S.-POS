//! vinculador.rs — Port de `yarvis-IA/parseador_de_tickets/cerebro/vinculador.py`
//!
//! Cruza productos parseados con el inventario existente:
//! 1. Coincidencia exacta (nombre normalizado idéntico).
//! 2. Coincidencia por embedding (similitud coseno > umbral).
//! 3. Sin vincular → revisión manual.
//!
//! El `texto_a_embedding` depende del modelo (FASE 4, ONNX). Aquí se expone un
//! trait `Embedder` para inyectarlo cuando exista; sin él, solo funciona el
//! match exacto y `por_embedding` queda en 0.

use regex::Regex;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Normalización y utilidades de embeddings (puero, sin modelos)
// ---------------------------------------------------------------------------

static RE_NO_WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^\w\s]").expect("regex no-word"));

static RE_ESPACIOS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+").expect("regex espacios"));

/// Normaliza un nombre para comparación: minúsculas, sin especiales, sin
/// espacios extra. Espejo de `_normalizar`.
pub fn normalizar(nombre: &str) -> String {
    let limpio = nombre.trim().to_lowercase();
    let limpio = RE_NO_WORD.replace_all(&limpio, "").into_owned();
    RE_ESPACIOS.replace_all(&limpio, " ").into_owned()
}

/// Deserializa un BLOB de SQLite a vector f32 (little-endian). Espejo de
/// `blob_a_embedding` (384 floats filas de `knowledge_base.embedding`).
pub fn blob_a_embedding(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Similitud coseno entre dos vectores. Espejo de `cosine_similarity`.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (*x as f64) * (*y as f64))
        .sum();
    let norm_a: f64 = a.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

// ---------------------------------------------------------------------------
// Tipos
// ---------------------------------------------------------------------------

/// Generador de embeddings de texto (FASE 4: ONNX/fastembed).
pub trait Embedder {
    fn texto_a_embedding(&self, texto: &str) -> Option<Vec<f32>>;
}

#[derive(Debug, Clone)]
pub struct ProductoInventario {
    pub id: i64,
    pub nombre: String,
    pub nombre_norm: String,
    pub precio_venta: f64,
    pub embedding: Option<Vec<f32>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProductoDb {
    pub id: i64,
    pub nombre: String,
    pub precio_venta: f64,
}

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

// ---------------------------------------------------------------------------
// Carga del inventario
// ---------------------------------------------------------------------------

/// Carga productos del inventario con su embedding (si existe en
/// `knowledge_base` con contenido "NOMBRE | $precio | stock: X").
pub fn cargar_inventario(db_path: &str) -> Vec<ProductoInventario> {
    let mut productos = Vec::new();
    let Ok(conn) = Connection::open(db_path) else {
        return productos;
    };

    // Mapa nombre-normalizado → embedding, desde knowledge_base.
    let mut embeddings_por_nombre: HashMap<String, Vec<f32>> = HashMap::new();
    if let Ok(mut stmt) =
        conn.prepare("SELECT contenido, embedding FROM knowledge_base WHERE embedding IS NOT NULL")
    {
        let rows = stmt.query_map([], |row| {
            let contenido: String = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            Ok((contenido, blob))
        });
        if let Ok(rows) = rows {
            for r in rows.flatten() {
                let (contenido, blob) = r;
                if let Some(nombre_kb) = contenido.split('|').next() {
                    embeddings_por_nombre.insert(normalizar(nombre_kb), blob_a_embedding(&blob));
                }
            }
        }
    }

    if let Ok(mut stmt) = conn.prepare("SELECT id, nombre, precio_venta FROM productos") {
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let nombre: String = row.get(1)?;
            let precio_venta: f64 = row.get(2)?;
            Ok((id, nombre, precio_venta))
        });
        if let Ok(rows) = rows {
            for r in rows.flatten() {
                let (id, nombre, precio_venta) = r;
                let nombre_norm = normalizar(&nombre);
                let embedding = embeddings_por_nombre.get(&nombre_norm).cloned();
                productos.push(ProductoInventario {
                    id,
                    nombre,
                    nombre_norm,
                    precio_venta,
                    embedding,
                });
            }
        }
    }

    productos
}

// ---------------------------------------------------------------------------
// Vinculación
// ---------------------------------------------------------------------------

/// Cruza productos parseados con el inventario existente.
///
/// `embedder` es opcional: si es `None`, solo se usa la coincidencia exacta
/// (el camino de embeddings queda deshabilitado).
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

// ---------------------------------------------------------------------------
// Persistencia de vinculaciones aprobadas
// ---------------------------------------------------------------------------

/// Guarda las vinculaciones aprobadas: actualiza `producto_id` en
/// `detalle_ventas`. Port de `/guardar_vinculacion` de vinculador.py.
/// Espejo exacto: se cuenta 1 por par `detalle_id` + `producto_id` válido
/// (Python no usa el rowcount del UPDATE).
pub fn guardar_vinculacion(
    vinculaciones: &[serde_json::Value],
    db_path: &str,
) -> Result<usize, String> {
    if vinculaciones.is_empty() {
        return Err("No hay vinculaciones para guardar".to_string());
    }

    let conn = Connection::open(db_path)
        .map_err(|e| format!("No se pudo abrir la base de datos: {e}"))?;

    let mut actualizados = 0usize;
    for v in vinculaciones {
        let detalle_id = v.get("detalle_id").and_then(|x| x.as_i64());
        let producto_id = v.get("producto_id").and_then(|x| x.as_i64());
        if let (Some(detalle_id), Some(producto_id)) = (detalle_id, producto_id) {
            conn.execute(
                "UPDATE detalle_ventas SET producto_id = ?1 WHERE id = ?2",
                rusqlite::params![producto_id, detalle_id],
            )
            .map_err(|e| format!("Error actualizando vinculación {detalle_id}: {e}"))?;
            actualizados += 1;
        }
    }

    Ok(actualizados)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_workspace(nombre: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cerebro_vinculador_{}_{}", nombre, nanos));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn crear_bd(dir: &Path) -> String {
        let path = dir.join("test.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE productos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                nombre TEXT NOT NULL,
                precio_venta REAL DEFAULT 0
             );
             CREATE TABLE knowledge_base (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                contenido TEXT,
                embedding BLOB
             );
             CREATE TABLE detalle_ventas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                venta_id INTEGER,
                producto_id INTEGER
             );",
        )
        .unwrap();
        drop(conn);
        path.to_string_lossy().to_string()
    }

    fn producto(nombre: &str) -> serde_json::Value {
        serde_json::json!({
            "producto": nombre,
            "cantidad": 1.0,
            "precio_unitario": 10.0,
            "total": 10.0,
            "descuento": null,
        })
    }

    // ---------- normalizar (verificado contra Python) ----------

    #[test]
    fn normaliza_nombres() {
        assert_eq!(normalizar("COCA-COLA 600ML   "), "cocacola 600ml");
        assert_eq!(normalizar("  coca  cola  "), "coca cola");
        assert_eq!(normalizar("TAZAS®"), "tazas");
        assert_eq!(normalizar("Coca-Cola Classic"), "cocacola classic");
        assert_eq!(normalizar("Día! día"), "día día");
    }

    // ---------- blob / coseno ----------

    #[test]
    fn blob_a_embeddings_correcto() {
        let mut blob = Vec::new();
        for v in [1.0f32, 2.0, 3.0, 4.0] {
            blob.extend_from_slice(&v.to_le_bytes());
        }
        assert_eq!(blob_a_embedding(&blob), vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn coseno_iguales_y_ortogonales() {
        assert_eq!(cosine_similarity(&[1.0, 0.0, 0.0, 0.0], &[1.0, 0.0, 0.0, 0.0]), 1.0);
        assert_eq!(cosine_similarity(&[1.0, 0.0, 0.0, 0.0], &[0.0, 1.0, 0.0, 0.0]), 0.0);
        assert_eq!(cosine_similarity(&[0.0, 0.0], &[0.0, 0.0]), 0.0);
    }

    // ---------- vinculación ----------

    struct EmbedderFijo(Vec<f32>);
    impl Embedder for EmbedderFijo {
        fn texto_a_embedding(&self, _texto: &str) -> Option<Vec<f32>> {
            Some(self.0.clone())
        }
    }

    #[test]
    fn match_exacto_por_nombre_normalizado() {
        let dir = tmp_workspace("exacto");
        let db = crear_bd(&dir);
        let conn = Connection::open(&db).unwrap();
        conn.execute(
            "INSERT INTO productos (nombre, precio_venta) VALUES ('TAZAS', 60.0)",
            [],
        )
        .unwrap();
        drop(conn);

        let parseados = vec![producto("TAZAS")];
        let res = vincular_con_inventario(&parseados, &db, 0.85, None);

        assert_eq!(res.estadisticas.exactos, 1);
        assert_eq!(res.estadisticas.sin_vincular, 0);
        assert_eq!(res.vinculados.len(), 1);
        let m = &res.vinculados[0];
        assert_eq!(m.tipo_match, "exacto");
        assert_eq!(m.score, 1.0);
        assert_eq!(m.producto_db.nombre, "TAZAS");
        assert_eq!(m.producto_db.precio_venta, 60.0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn match_exacto_ignora_caracteres_raros() {
        let dir = tmp_workspace("exacto2");
        let db = crear_bd(&dir);
        let conn = Connection::open(&db).unwrap();
        conn.execute(
            "INSERT INTO productos (nombre, precio_venta) VALUES ('Coca-Cola Classic', 25.0)",
            [],
        )
        .unwrap();
        drop(conn);

        let parseados = vec![producto("  COCA-COLA   CLASSIC  ")];
        let res = vincular_con_inventario(&parseados, &db, 0.85, None);
        assert_eq!(res.estadisticas.exactos, 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn match_exacto_gana_el_ultimo_si_duplicado() {
        // Python asigna el índice por nombre normalizado con overwrite: el
        // último de dos productos con el mismo nombre gana. Verificado contra
        // Python (resuelve al id 3, precio 30.0).
        let dir = tmp_workspace("ultimo_gana");
        let db = crear_bd(&dir);
        let conn = Connection::open(&db).unwrap();
        conn.execute(
            "INSERT INTO productos (nombre, precio_venta) VALUES ('TAZAS', 60.0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO productos (nombre, precio_venta) VALUES ('Coca-Cola Classic', 25.0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO productos (nombre, precio_venta) VALUES ('TAZAS', 30.0)",
            [],
        )
        .unwrap();
        drop(conn);

        let parseados = vec![
            producto("TAZAS"),
            producto("  coca-cola classic "),
            producto("JUGUETE RARO"),
        ];
        let res = vincular_con_inventario(&parseados, &db, 0.85, None);

        assert_eq!(res.estadisticas.exactos, 2);
        assert_eq!(res.estadisticas.sin_vincular, 1);
        assert_eq!(res.estadisticas.total_parseados, 3);
        assert_eq!(res.estadisticas.por_embedding, 0);
        // El producto "TAZAS" matchea con el ÚLTIMO de nombre normalizado "tazas".
        assert_eq!(res.vinculados[0].producto_db.id, 3);
        assert_eq!(res.vinculados[0].producto_db.precio_venta, 30.0);
        assert_eq!(res.vinculados[1].producto_db.nombre, "Coca-Cola Classic");
        assert_eq!(res.sin_vincular[0].producto_parseado["producto"], "JUGUETE RARO");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sin_coincidencia_va_a_revision_manual() {
        let dir = tmp_workspace("sin_match");
        let db = crear_bd(&dir);
        let conn = Connection::open(&db).unwrap();
        conn.execute(
            "INSERT INTO productos (nombre, precio_venta) VALUES ('TAZAS', 60.0)",
            [],
        )
        .unwrap();
        drop(conn);

        let parseados = vec![producto("JUGUETE DE PLÁSTICO RARO")];
        let res = vincular_con_inventario(&parseados, &db, 0.85, None);

        assert_eq!(res.estadisticas.sin_vincular, 1);
        assert_eq!(res.sin_vincular[0].razon, "Sin coincidencia en inventario");
        assert!(res.vinculados.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn match_por_embedding_sobre_umbral() {
        let dir = tmp_workspace("embedding");
        let db = crear_bd(&dir);
        let conn = Connection::open(&db).unwrap();
        conn.execute(
            "INSERT INTO productos (nombre, precio_venta) VALUES ('TAZAS ROJAS', 60.0)",
            [],
        )
        .unwrap();

        // Embedding en knowledge_base con contenido "NOMBRE | $precio | stock".
        let mut blob = Vec::new();
        for v in [1.0f32, 0.0, 0.0, 0.0] {
            blob.extend_from_slice(&v.to_le_bytes());
        }
        conn.execute(
            "INSERT INTO knowledge_base (contenido, embedding) VALUES ('TAZAS ROJAS | $60.00 | stock: 10', ?1)",
            rusqlite::params![blob],
        )
        .unwrap();
        drop(conn);

        // Producto parseado distinto textualmente, con embedding igual.
        let parseados = vec![producto("tazas rojas jumbo")];
        let embedder = EmbedderFijo(vec![1.0, 0.0, 0.0, 0.0]);
        let res = vincular_con_inventario(&parseados, &db, 0.85, Some(&embedder));

        assert_eq!(res.estadisticas.por_embedding, 1);
        assert_eq!(res.estadisticas.exactos, 0);
        assert_eq!(res.vinculados[0].tipo_match, "embedding");
        assert_eq!(res.vinculados[0].producto_db.nombre, "TAZAS ROJAS");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn match_por_embedding_bajo_umbral_no_vincula() {
        let dir = tmp_workspace("embedding_bajo");
        let db = crear_bd(&dir);
        let conn = Connection::open(&db).unwrap();
        conn.execute(
            "INSERT INTO productos (nombre, precio_venta) VALUES ('TAZAS ROJAS', 60.0)",
            [],
        )
        .unwrap();
        let mut blob = Vec::new();
        for v in [1.0f32, 0.0, 0.0, 0.0] {
            blob.extend_from_slice(&v.to_le_bytes());
        }
        conn.execute(
            "INSERT INTO knowledge_base (contenido, embedding) VALUES ('TAZAS ROJAS | $60.00 | stock: 10', ?1)",
            rusqlite::params![blob],
        )
        .unwrap();
        drop(conn);

        // Embedding ortogonal → coseno 0 < umbral.
        let parseados = vec![producto("algo totalmente distinto")];
        let embedder = EmbedderFijo(vec![0.0, 1.0, 0.0, 0.0]);
        let res = vincular_con_inventario(&parseados, &db, 0.85, Some(&embedder));

        assert_eq!(res.estadisticas.por_embedding, 0);
        assert_eq!(res.estadisticas.sin_vincular, 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sin_embedder_no_hay_match_por_embedding() {
        let dir = tmp_workspace("sin_embedder");
        let db = crear_bd(&dir);
        let conn = Connection::open(&db).unwrap();
        conn.execute(
            "INSERT INTO productos (nombre, precio_venta) VALUES ('TAZAS ROJAS', 60.0)",
            [],
        )
        .unwrap();
        drop(conn);

        let parseados = vec![producto("tazas rojas jumbo")];
        let res = vincular_con_inventario(&parseados, &db, 0.85, None);
        assert_eq!(res.estadisticas.por_embedding, 0);
        assert_eq!(res.estadisticas.sin_vincular, 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---------- guardar_vinculacion (port de /guardar_vinculacion) ----------

    #[test]
    fn guardar_vinculacion_actualiza_producto_id() {
        let dir = tmp_workspace("guardar");
        let db = crear_bd(&dir);
        let conn = Connection::open(&db).unwrap();
        conn.execute(
            "INSERT INTO detalle_ventas (id, venta_id, producto_id) VALUES (1, 1, NULL)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO detalle_ventas (id, venta_id, producto_id) VALUES (2, 1, NULL)",
            [],
        )
        .unwrap();
        drop(conn);

        let vinculaciones = serde_json::json!([
            { "detalle_id": 1, "producto_id": 7 },
            // Python cuenta el par válido aunque la fila no exista (actualizados += 1).
            { "detalle_id": 3, "producto_id": 9 },
            { "detalle_id": 2, "producto_id": null }, // sin producto_id → se salta
        ]);
        let lista = vinculaciones.as_array().cloned().unwrap();

        let actualizados = guardar_vinculacion(&lista, &db).unwrap();
        assert_eq!(actualizados, 2);

        let conn = Connection::open(&db).unwrap();
        let (p1, p2): (Option<i64>, Option<i64>) = conn
            .query_row(
                "SELECT producto_id, (SELECT producto_id FROM detalle_ventas WHERE id = 2) FROM detalle_ventas WHERE id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(p1, Some(7));
        assert_eq!(p2, None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn guardar_vinculacion_sin_datos_error() {
        let dir = tmp_workspace("guardar_vacio");
        let db = crear_bd(&dir);
        let res = guardar_vinculacion(&[], &db);
        assert!(res.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}