use rusqlite::Connection;
use std::collections::HashSet;
use std::sync::mpsc::Sender;

use super::almacen::{
    cargar_estado, cargar_folios_existentes, cargar_productos_por_nombre,
    garantizar_columna_folio, insertar_venta,
};
use super::archivos::{leer_archivo_tolerante, nombre_de_archivo};
use super::items::{a_centavos, resolver_totales_venta};
use super::resumen::{
    ArchivoResultado, EstadisticasCarpeta, ProductoNuevo, ResumenVenta, TicketFallido,
};
use crate::cerebro::analizador_tickets::{
    extraer_totales, parsear_linea, segmentar, Item, MapeoColumnas,
};
use crate::embeddings::Embedder;

pub fn procesar_archivos(
    archivos: &[String],
    mapeo: &MapeoColumnas,
    db_path: &str,
    tx: &Sender<ArchivoResultado>,
) {
    let mut productos_vistos = cargar_estado(db_path);

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            for archivo in archivos {
                let mut res = ArchivoResultado::info(
                    false,
                    Some(format!("no se pudo abrir la base de datos: {e}")),
                );
                res.archivo = nombre_de_archivo(archivo);
                let _ = tx.send(res);
            }
            return;
        }
    };
    // Columna folio_ticket: las DBs creadas antes de la migración no la tienen.
    garantizar_columna_folio(&conn);
    let productos_por_nombre = cargar_productos_por_nombre(&conn);
    // Cache para fuzzy matching (evita fantasmas tipo "ACEITE" vs "ACEITE 123 1L")
    let inventario_fuzzy = crate::cerebro::vinculador_inventario::cargar_inventario_cache(&conn);
    // Idempotencia: folios YA importados (de corridas anteriores). Un ticket
    // con folio conocido se omite entero: re-importar la misma carpeta ya no
    // duplica ventas, ni vuelve a descontar stock.
    let mut folios_importados = cargar_folios_existentes(&conn);

    for archivo in archivos {
        let nombre_archivo = nombre_de_archivo(archivo);

        let texto = match leer_archivo_tolerante(archivo) {
            Ok(t) => t,
            Err(e) => {
                let mut res = ArchivoResultado::info(false, Some(format!("error inesperado: {e}")));
                res.archivo = nombre_archivo;
                let _ = tx.send(res);
                continue;
            }
        };

        if texto.trim().is_empty() {
            let mut res = ArchivoResultado::info(false, Some("archivo vacío".to_string()));
            res.archivo = nombre_archivo;
            let _ = tx.send(res);
            continue;
        }

        let lineas: Vec<&str> = texto
            .trim()
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        if lineas.is_empty() {
            let mut res = ArchivoResultado::info(false, Some("sin líneas útiles".to_string()));
            res.archivo = nombre_archivo;
            let _ = tx.send(res);
            continue;
        }

        let total_cols = lineas
            .iter()
            .map(|l| l.split_whitespace().count())
            .max()
            .unwrap_or(0);

        // Un archivo puede traer N tickets concatenados → N ventas.
        let segmentos = segmentar(&texto);

        let mut items_totales = 0usize;
        let mut duplicados_totales = 0usize;
        let mut existentes_totales = 0usize;
        let mut omitidas_totales = 0usize;
        let mut nuevos_archivo: Vec<ProductoNuevo> = Vec::new();
        let mut ventas_info: Vec<ResumenVenta> = Vec::new();
        let mut total_archivo = 0.0;

        if let Err(e) = conn.execute_batch("BEGIN") {
            let mut res = ArchivoResultado::info(
                false,
                Some(format!(
                    "error al insertar en DB: no se pudo iniciar la transacción ({e})"
                )),
            );
            res.archivo = nombre_archivo;
            res.items = items_totales;
            res.duplicados = duplicados_totales;
            res.nuevos = nuevos_archivo;
            res.existentes = existentes_totales;
            let _ = tx.send(res);
            continue;
        }

        let mut error_db: Option<String> = None;

        for segmento in &segmentos {
            // Folio ya importado (esta u otra corrida): se omite el ticket
            // ENTERO — sin parseo de items, sin productos, sin venta, sin
            // tocar stock. La venta ya existe con sus mismos efectos.
            if let Some(folio) = segmento.folio.as_deref() {
                if folios_importados.contains(folio.trim()) {
                    omitidas_totales += 1;
                    continue;
                }
            }

            let mut items: Vec<Item> = Vec::new();
            // Dedupe POR TICKET: productos repetidos entre tickets distintos
            // de un mismo archivo ya no se pierden.
            let mut seen: HashSet<String> = HashSet::new();
            let mut duplicados = 0usize;
            let mut existentes = 0usize;
            let mut nuevos_seg: Vec<ProductoNuevo> = Vec::new();

            for linea in &segmento.lineas {
                let Some(item) = parsear_linea(linea, mapeo, total_cols) else {
                    continue;
                };

                // Clave estable en CENTAVOS enteros (mismo formato que
                // `cargar_estado` en almacen.rs); el f64 `{:.2}` viejo podía
                // divergir por ruido de redondeo.
                let dup_key = format!("{}|{}", item.producto, a_centavos(item.precio_unitario));
                if seen.contains(&dup_key) {
                    duplicados += 1;
                    continue;
                }
                seen.insert(dup_key.clone());

                if productos_vistos.contains(&dup_key) {
                    existentes += 1;
                } else {
                    // Antes se creaba fantasma con costo $0 sin validar. Ahora verificamos
                    // si el ticket trae "ACEITE" truncado y ya existe "ACEITE 123 1L" en inventario:
                    // si hay match fuzzy, NO creamos "ACEITE" suelto, lo contamos como existente
                    // para no poblar fantasmas que luego quedan con stock -158.
                    let es_truncado = crate::cerebro::parseador_masivo::almacen::resolver_producto_id(
                        &item.producto,
                        &productos_por_nombre,
                        &inventario_fuzzy,
                    )
                    .is_some();

                    // Solo crear si es producto realmente nuevo (no similar) y nombre tiene >=2 palabras
                    // o es único. Un token suelto tipo "ACEITE" con inventario no vacío se deja sin crear
                    // solo si hay algún producto algo similar (>0.3); si es "TAZAS" vs "ACEITE" (0.0) sí se crea.
                    let crear = if es_truncado {
                        existentes += 1;
                        false
                    } else if item.producto.split_whitespace().count() < 2
                        && !inventario_fuzzy.is_empty()
                    {
                        let embedder = crate::embeddings::HashEmbedder;
                        let q_emb_opt = embedder.texto_a_embedding(&item.producto);
                        let mut max_score = 0.0;
                        if let Some(q_emb) = q_emb_opt {
                            for p in &inventario_fuzzy {
                                if let Some(p_emb) = embedder.texto_a_embedding(&p.nombre) {
                                    let s = crate::embeddings::cosine_similarity(&q_emb, &p_emb);
                                    if s > max_score {
                                        max_score = s;
                                    }
                                }
                            }
                        }
                        if max_score > 0.30 {
                            existentes += 1;
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    };

                    if crear {
                        productos_vistos.insert(dup_key.clone());
                        let _ = conn.execute(
                            "INSERT INTO productos (nombre, precio_venta, precio_costo, stock, stock_minimo, vendido, categoria) VALUES (?1, ?2, ?3, 0, 5, ?4, '')",
                            rusqlite::params![item.producto.clone(), a_centavos(item.precio_unitario), a_centavos(0.0), item.cantidad],
                        );
                        nuevos_seg.push(ProductoNuevo {
                            nombre: item.producto.clone(),
                            precio: item.precio_unitario,
                        });
                    } else {
                        // No creamos, pero lo marcamos como visto para no reintentar
                        productos_vistos.insert(dup_key.clone());
                    }
                }
                items.push(item);
            }

            // Segmento sin productos reconocidos (solo encabezado) → no venta.
            if items.is_empty() {
                continue;
            }

            // Totales REALES del ticket con fallback al cálculo (× 1.16).
            let reales = extraer_totales(&segmento.texto());
            let (subtotal, iva, total) = resolver_totales_venta(&items, &reales);

            match insertar_venta(
                &conn,
                &items,
                &segmento.cajero,
                segmento.fecha_hora.as_deref(),
                &segmento.metodo_pago,
                segmento.folio.as_deref(),
                subtotal,
                iva,
                total,
                &productos_por_nombre,
            ) {
                Ok(venta_id) => {
                    // Registrar el folio: dos archivos con el mismo ticket en
                    // la MISMA corrida tampoco se duplican entre sí.
                    if let Some(folio) = segmento.folio.as_deref() {
                        folios_importados.insert(folio.trim().to_string());
                    }
                    items_totales += items.len();
                    duplicados_totales += duplicados;
                    existentes_totales += existentes;
                    total_archivo += total;
                    ventas_info.push(ResumenVenta {
                        archivo: nombre_archivo.clone(),
                        venta_id: Some(venta_id),
                        items: items.len(),
                        total,
                        folio: segmento.folio.clone(),
                        fecha_hora: segmento.fecha_hora.clone(),
                    });
                    nuevos_archivo.append(&mut nuevos_seg);
                }
                Err(e) => {
                    error_db = Some(format!("error al insertar en DB: {e}"));
                    break;
                }
            }
        }

        if let Some(motivo) = error_db {
            let _ = conn.execute_batch("ROLLBACK");
            let mut res = ArchivoResultado::info(false, Some(motivo));
            res.archivo = nombre_archivo;
            res.items = items_totales;
            res.duplicados = duplicados_totales;
            res.nuevos = nuevos_archivo;
            res.existentes = existentes_totales;
            res.ventas_omitidas = omitidas_totales;
            let _ = tx.send(res);
            continue;
        }

        if ventas_info.is_empty() {
            let _ = conn.execute_batch("ROLLBACK");
            // Re-importación: no es error que NADA sea nuevo; se informa.
            let mut res = if omitidas_totales > 0 {
                ArchivoResultado::info(
                    true,
                    Some(format!(
                        "{omitidas_totales} ticket(s) ya importados; no se duplicaron"
                    )),
                )
            } else {
                ArchivoResultado::info(
                    false,
                    Some("ningún producto reconocido con el mapeo actual".to_string()),
                )
            };
            res.archivo = nombre_archivo;
            res.ventas_omitidas = omitidas_totales;
            let _ = tx.send(res);
            continue;
        }

        let _ = conn.execute_batch("COMMIT");
        let mut res = ArchivoResultado::info(true, None);
        res.archivo = nombre_archivo;
        res.items = items_totales;
        res.duplicados = duplicados_totales;
        res.nuevos = nuevos_archivo;
        res.existentes = existentes_totales;
        res.ventas_omitidas = omitidas_totales;
        res.ventas = ventas_info.len();
        res.venta_id = ventas_info.first().and_then(|v| v.venta_id);
        res.total = total_archivo;
        res.ventas_info = ventas_info;
        let _ = tx.send(res);
    }
}

// ---------------------------------------------------------------------------
// Modo síncrono: agrega los resultados por archivo en estadísticas totales
// ---------------------------------------------------------------------------

pub fn procesar_carpeta_impl(
    archivos: Vec<String>,
    mapeo: MapeoColumnas,
    db_path: String,
) -> EstadisticasCarpeta {
    let mut stats = EstadisticasCarpeta {
        total_archivos: archivos.len(),
        ..Default::default()
    };

    let mut nombres_nuevos_vistos: HashSet<String> = HashSet::new();
    let (tx, rx) = std::sync::mpsc::channel::<ArchivoResultado>();

    procesar_archivos(&archivos, &mapeo, &db_path, &tx);
    drop(tx);

    for res in rx {
        stats.procesados += 1;
        if res.ok {
            stats.exitosos += 1;
            stats.ventas_creadas += res.ventas;
            stats.ventas_omitidas += res.ventas_omitidas;
            stats.items_insertados += res.items;
            stats.duplicados_detectados += res.duplicados;
            stats.productos_existentes += res.existentes;
            stats.productos_nuevos += res.nuevos.len();
            for nuevo in res.nuevos {
                if nombres_nuevos_vistos.insert(nuevo.nombre.clone()) {
                    stats.productos_nuevos_lista.push(nuevo);
                }
            }
            stats.resumen_ventas.extend(res.ventas_info);
        } else {
            stats.errores += 1;
            stats.tickets_fallidos.push(TicketFallido {
                archivo: res.archivo,
                motivo: res.motivo,
            });
        }
    }

    stats.productos_nuevos_lista.truncate(100);
    stats.tickets_fallidos.truncate(500);
    stats
}
