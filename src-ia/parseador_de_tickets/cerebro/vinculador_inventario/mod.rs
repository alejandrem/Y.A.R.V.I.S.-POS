// ============================================================
// vinculador_inventario — Cruza productos parseados con el
// inventario existente. Port de vinculador.py.
//
//   * similitud.rs   → normalización, embeddings y coseno
//   * inventario.rs  → carga de productos + knowledge_base
//   * vinculo.rs     → decisión del match (exacto + embedding)
//   * persistencia.rs→ guardado de vinculaciones aprobadas
//
// El `texto_a_embedding` depende del modelo (FASE 4, ONNX): aquí se
// expone un trait `Embedder` para inyectarlo cuando exista; sin él,
// solo funciona el match exacto y `por_embedding` queda en 0.
// ============================================================

mod inventario;
mod persistencia;
mod similitud;
mod vinculo;

pub use inventario::{cargar_inventario, ProductoDb, ProductoInventario};
pub use persistencia::guardar_vinculacion;
pub use similitud::{blob_a_embedding, cosine_similarity, normalizar, Embedder};
pub use vinculo::{
    vincular_con_inventario, EstadisticasVinculacion, Match, ResultadoVinculacion, SinVincular,
};

// Imports que los tests del módulo usan vía `use super::*`.
#[cfg(test)]
use rusqlite::Connection;
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
        assert_eq!(
            cosine_similarity(&[1.0, 0.0, 0.0, 0.0], &[1.0, 0.0, 0.0, 0.0]),
            1.0
        );
        assert_eq!(
            cosine_similarity(&[1.0, 0.0, 0.0, 0.0], &[0.0, 1.0, 0.0, 0.0]),
            0.0
        );
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
        assert_eq!(
            res.sin_vincular[0].producto_parseado["producto"],
            "JUGUETE RARO"
        );

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
