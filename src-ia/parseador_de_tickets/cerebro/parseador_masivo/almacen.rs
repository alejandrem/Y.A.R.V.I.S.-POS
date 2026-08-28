use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};

use super::items::{a_centavos, round2};
use crate::cerebro::analizador_tickets::Item;
use crate::cerebro::vinculador_inventario::normalizar;
use crate::embeddings::{cosine_similarity, HashEmbedder, Embedder};

/// Asegura la columna `folio_ticket` en `ventas` (las DBs viejas creadas con
/// `CREATE TABLE IF NOT EXISTS` no se migran solas). Idempotente vía PRAGMA.
pub(super) fn garantizar_columna_folio(conn: &Connection) {
    let ok = match conn.prepare("PRAGMA table_info(ventas)") {
        Ok(mut stmt) => stmt
            .query_map([], |r| r.get::<_, String>(1))
            .map(|rows| {
                rows.flatten()
                    .any(|c| c.eq_ignore_ascii_case("folio_ticket"))
            })
            .unwrap_or(false),
        Err(_) => false,
    };
    if !ok {
        let _ = conn.execute_batch("ALTER TABLE ventas ADD COLUMN folio_ticket TEXT");
    }
}

/// Intenta resolver un producto por nombre normalizado exacto y, si falla,
/// por similitud de embeddings (HashEmbedder 384d trigram). Evita el viejo
/// `WHERE LOWER(nombre)=LOWER(?)` que era frágil ante truncados tipo
/// `ACEITE` vs `ACEITE 123 1L` y que causaba filas fantasma con costo $0.
pub fn resolver_producto_id(
    nombre_ticket: &str,
    productos_por_nombre: &HashMap<String, Option<i64>>,
    inventario: &[crate::cerebro::vinculador_inventario::ProductoInventario],
) -> Option<i64> {
    // 1) Exacto normalizado y no ambiguo
    let norm = normalizar(nombre_ticket);
    if let Some(Some(id)) = productos_por_nombre.get(&norm) {
        return Some(*id);
    }
    // Si hay ambigüedad exacta (None por duplicado), no inventar
    if productos_por_nombre.contains_key(&norm) {
        return None;
    }
    // 2) Fuzzy por embedding: solo si hay un único ganador claro >0.55 y segundo <0.45
    // Así "ACEITE" no crea fantasma si ya existe "ACEITE 123 1L" / "ACEITE NUTRIOLI 1L"
    if inventario.is_empty() {
        return None;
    }
    let embedder = HashEmbedder;
    let q_emb = embedder.texto_a_embedding(nombre_ticket)?;
    let mut mejor: Option<(i64, f64)> = None;
    let mut segundo = 0.0f64;
    for p in inventario {
        // Si el producto ya tiene embedding de knowledge_base, úsalo; si no, genera al vuelo
        let p_emb = if let Some(ref e) = p.embedding {
            e
        } else {
            // Generar al vuelo para productos sin backfill no es costoso (1M * 384)
            // Lo calculamos lazy: solo si hace falta, creamos vector temporal
            // Para no alocar en hot loop, usamos el HashEmbedder directo sobre nombre
            // y descartamos si es None
            continue;
        };
        let score = cosine_similarity(&q_emb, p_emb);
        if let Some((_, best_score)) = mejor {
            if score > best_score {
                segundo = best_score;
                mejor = Some((p.id, score));
            } else if score > segundo {
                segundo = score;
            }
        } else {
            mejor = Some((p.id, score));
        }
    }
    // Fallback sin embeddings: generar embedding del nombre del inventario al vuelo
    if mejor.is_none() {
        for p in inventario {
            let p_emb = match embedder.texto_a_embedding(&p.nombre) {
                Some(v) => v,
                None => continue,
            };
            let score = cosine_similarity(&q_emb, &p_emb);
            if let Some((_, best_score)) = mejor {
                if score > best_score {
                    segundo = best_score;
                    mejor = Some((p.id, score));
                } else if score > segundo {
                    segundo = score;
                }
            } else {
                mejor = Some((p.id, score));
            }
        }
    }
    if let Some((id, best)) = mejor {
        // Umbral 0.55 generaliza truncados "ACEITE" -> "ACEITE 123 1L" (~0.60) sin falsos "pepsi" -> "coca" (0.0)
        // y exige que el segundo no esté pegado (ambigüedad tipo 2 aceites)
        if best >= 0.55 && segundo < 0.52 {
            return Some(id);
        }
    }
    None
}

/// Construye un mapa `nombre_normalizado → producto_id` desde la tabla
/// `productos`. Si dos o más productos comparten el mismo nombre normalizado,
/// el valor es `None` (ambiguo: no se vincula automáticamente).
///
/// Esto permite que `insertar_venta` asigne `producto_id` al detalle cuando la
/// coincidencia por nombre es **exacta y no ambigua**, y que el descuento de
/// stock se haga por ID (más fiable) en lugar de por nombre.
pub(super) fn cargar_productos_por_nombre(conn: &Connection) -> HashMap<String, Option<i64>> {
    let mut acumulador: HashMap<String, Vec<i64>> = HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT id, nombre FROM productos") {
        if let Ok(rows) = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let nombre: String = row.get(1)?;
            Ok((id, nombre))
        }) {
            for r in rows.flatten() {
                let (id, nombre) = r;
                acumulador
                    .entry(normalizar(&nombre))
                    .or_default()
                    .push(id);
            }
        }
    }
    // Vec<i64> → Option<i64>: Some(id) solo si hay exactamente uno.
    acumulador
        .into_iter()
        .map(|(k, ids)| {
            (
                k,
                if ids.len() == 1 {
                    Some(ids[0])
                } else {
                    None
                },
            )
        })
        .collect()
}

/// Precarga el set de claves `NOMBRE|precio_en_centavos` ya existentes en
/// `productos`. La clave usa el precio en CENTAVOS enteros (no `{:.2}` de un
/// f64) para que sea estable: debe construirse igual que `dup_key` en
/// `procesador.rs`, con [`a_centavos`] sobre el precio del item.
pub(super) fn cargar_estado(db_path: &str) -> HashSet<String> {
    let mut vistos = HashSet::new();
    if let Ok(conn) = Connection::open(db_path) {
        if let Ok(mut stmt) = conn.prepare("SELECT nombre, precio_venta FROM productos") {
            if let Ok(rows) = stmt.query_map([], |row| {
                let nombre: String = row.get(0)?;
                // Columna INTEGER en centavos desde la migración.
                let precio_centavos: i64 = row.get(1)?;
                Ok((nombre, precio_centavos))
            }) {
                for row in rows.flatten() {
                    let (nombre, precio_centavos) = row;
                    vistos.insert(format!("{}|{}", nombre.trim().to_uppercase(), precio_centavos));
                }
            }
        }
    }
    vistos
}

/// Fecha/hora UTC actual "YYYY-MM-DD HH:MM:SS" (mismo formato que el
/// `CURRENT_TIMESTAMP` de SQLite). Fallback para tickets sin fecha: así la
/// venta no se pierde de ventas_diarias / conteos por día.
fn ahora_iso_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let dias = (secs / 86400) as i64;
    let resto = secs % 86400;

    // Algoritmo civil_from_days (Howard Hinnant), sin dependencias.
    let z = dias + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let anio = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dia = doy - (153 * mp + 2) / 5 + 1;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 };
    let anio = if mes <= 2 { anio + 1 } else { anio };

    let hora = resto / 3600;
    let min = (resto % 3600) / 60;
    let seg = resto % 60;
    format!("{anio:04}-{mes:02}-{dia:02} {hora:02}:{min:02}:{seg:02}")
}

/// Inserta una venta con sus totales YA resueltos (reales del ticket o
/// cálculo) y el folio del ticket si se detectó. Requiere que el llamador
/// haya corrido [`garantizar_columna_folio`] al abrir la conexión.
///
/// Los parámetros monetarios llegan en PESOS f64 (dominio del parser) y se
/// escriben en CENTAVOS enteros vía [`a_centavos`]: la DB migrada guarda
/// `ventas.total/subtotal/iva` y `detalle_ventas.precio_unitario/descuento/
/// subtotal` como INTEGER.
///
/// `productos_por_nombre` asigna `producto_id` al detalle cuando el nombre
/// normalizado del item coincide **exactamente** con un único producto del
/// catálogo. Si hay 0 o más de 1 coincidencia, el detalle queda con
/// `producto_id = NULL` (sin vincular) y el stock se descuenta por nombre.
#[allow(clippy::too_many_arguments)]
pub(super) fn insertar_venta(
    conn: &Connection,
    items: &[Item],
    cajero: &str,
    fecha_iso: Option<&str>,
    metodo_pago: &str,
    folio: Option<&str>,
    subtotal: f64,
    iva: f64,
    total: f64,
    productos_por_nombre: &HashMap<String, Option<i64>>,
) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO ventas (total, subtotal, iva, cajero, metodo_pago, estado, fecha, folio_ticket)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            a_centavos(total),
            a_centavos(subtotal),
            a_centavos(iva),
            cajero,
            metodo_pago,
            "completada",
            // Ticket sin fecha → ahora (UTC), no NULL (se perdería del conteo).
            fecha_iso.map(str::to_owned).unwrap_or_else(ahora_iso_utc),
            folio
        ],
    )?;
    let venta_id = conn.last_insert_rowid();

    // Inventario para fuzzy (se carga una vez por venta, no por item, para no hacer N queries)
    let inventario_cache: Vec<crate::cerebro::vinculador_inventario::ProductoInventario> =
        crate::cerebro::vinculador_inventario::cargar_inventario_cache(conn);

    for item in items {
        let sub = round2(item.cantidad * item.precio_unitario - item.descuento.unwrap_or(0.0));

        // Resolver producto_id: exacto normalizado -> fuzzy embedding -> None (sin vincular)
        let producto_id = resolver_producto_id(&item.producto, productos_por_nombre, &inventario_cache);

        conn.execute(
            "INSERT INTO detalle_ventas (venta_id, producto_id, producto_nombre, cantidad, precio_unitario, descuento, subtotal)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                venta_id,
                producto_id,
                item.producto,
                item.cantidad,
                a_centavos(item.precio_unitario),
                a_centavos(item.descuento.unwrap_or(0.0)),
                a_centavos(sub)
            ],
        )?;

        // Solo tocar stock si hay producto_id fiable. Si es None (truncado tipo "ACEITE" ambiguo
        // o producto realmente nuevo), NO hacer UPDATE por LOWER — eso creaba fantasmas con costo $0
        // y dejaba stock -158 sin freno. El stock se ajustará cuando el vinculador lo resuelva.
        if let Some(pid) = producto_id {
            conn.execute(
                "UPDATE productos SET stock = stock - ?1 WHERE id = ?2",
                params![item.cantidad, pid],
            )?;
            conn.execute(
                "UPDATE productos SET vendido = vendido + ?1 WHERE id = ?2",
                params![item.cantidad, pid],
            )?;
        }
    }

    Ok(venta_id)
}
