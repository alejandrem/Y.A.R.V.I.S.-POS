//! masivo.rs — Estrés del `parseador_masivo` a escala (tickets gigantes,
//! carpetas torturadas, entradas vacías y límites numéricos).
//!
//! Invariante vigilada aquí:
//!   4. El dinero del ticket cuadra: total DB == subtotal + IVA (redondeo 2).

use rusqlite::Connection;

use src_ia::cerebro::analizador_tickets::{parsear_linea, MapeoColumnas};
use src_ia::cerebro::parseador_masivo::procesar_carpeta_impl;

use crate::soporte::{contar, crear_bd, finito, mapeo, tmp_workspace};

// ---------------------------------------------------------------------------
// 10. Ticket GIGANTE (20 mil líneas) entra entero a la BD sin desmadre
// ---------------------------------------------------------------------------

#[test]
fn ticket_gigante_20mil_items_entra_completo() {
    let dir = tmp_workspace("gigante");
    let db = crear_bd(&dir);

    let mut texto = String::from("TICKET MEGA\n12/05/2026\n");
    for i in 0..20_000 {
        texto.push_str(&format!("2 PRODUCTO{i} $60.00 $120.00\n"));
    }
    texto.push_str("TOTAL $2,400,000.00\n");

    let ruta = dir.join("mega.txt");
    std::fs::write(&ruta, &texto).unwrap();

    let stats = procesar_carpeta_impl(
        vec![ruta.to_string_lossy().to_string()],
        mapeo(),
        db.clone(),
    );

    assert_eq!(stats.exitosos, 1);
    assert_eq!(stats.errores, 0);
    assert_eq!(stats.items_insertados, 20_000);
    assert_eq!(stats.duplicados_detectados, 0);
    assert_eq!(contar(&db, "detalle_ventas"), 20_000);
    // Dinero (regla D): el ticket declara TOTAL = $2,400,000 y SE USA ese real,
    // aunque difiera del calculado × 1.16 ($2,784,000). SUBTOTAL e IVA no vienen
    // en el ticket → fallback al cálculo (2,400,000 + 384,000).
    assert_eq!(stats.resumen_ventas[0].total, 2_400_000.0);

    let (total, subtotal, iva): (i64, i64, i64) = Connection::open(&db)
        .unwrap()
        .query_row("SELECT total, subtotal, iva FROM ventas", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .unwrap();
    // Dinero (regla D actual): los precios YA incluyen IVA → iva almacenado
    // es 0 y total = subtotal (o el real declarado por el ticket si existe).
    // La DB guarda CENTAVOS: $2,400,000 → 240,000,000.
    assert_eq!(subtotal, 240_000_000);
    assert_eq!(iva, 0);
    assert_eq!(total, 240_000_000);

    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 11. Carpeta con 300 archivos torturados mezclados
// ---------------------------------------------------------------------------

#[test]
fn carpeta_de_300_tickets_torturados_divide_bien_exitosos_y_errores() {
    let dir = tmp_workspace("tortura");
    let db = crear_bd(&dir);

    let mut archivos = Vec::new();
    let mut exitosos_esperados = 0usize;

    for i in 0..200 {
        let content = format!(
            "TICKET {i}\n15/03/2026\n1 ARTICULO{i} $9.99 $9.99\nEFECTIVO $10.00\nTOTAL $9.99\n"
        );
        let p = dir.join(format!("ok_{i}.txt"));
        std::fs::write(&p, content).unwrap();
        archivos.push(p.to_string_lossy().to_string());
        exitosos_esperados += 1;
    }

    for j in 0..100 {
        let content = match j % 4 {
            0 => String::from_utf8_lossy(b"\xFF\xFE\xFF-----=====-----").into_owned(),
            1 => "GRACIAS POR SU COMPRA\nCFDI: 4D8F2A1".to_string(),
            2 => "TWO-DAY SALE!!".to_string(),
            _ => "<<<<<,,,,,  .... ════".to_string(),
        };
        let p = dir.join(format!("trampa_{j}.txt"));
        std::fs::write(&p, content).unwrap();
        archivos.push(p.to_string_lossy().to_string());
    }

    let stats = procesar_carpeta_impl(archivos, mapeo(), db.clone());

    assert_eq!(stats.total_archivos, 300);
    assert_eq!(stats.exitosos, exitosos_esperados);
    assert_eq!(stats.errores, 100);
    assert_eq!(stats.ventas_creadas, exitosos_esperados);
    assert_eq!(contar(&db, "ventas"), exitosos_esperados as i64);
    assert_eq!(contar(&db, "detalle_ventas"), exitosos_esperados as i64);
    // Ningún total de venta puede ser no-finito en la BD.
    let no_finitos: i64 = Connection::open(&db)
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM ventas WHERE total IS NULL OR total <> total",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(no_finitos, 0);

    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 12. Tickets vacíos, gigantes en memoria y carpetas sin txt
// ---------------------------------------------------------------------------

#[test]
fn entradas_vacias_y_carpetas_sin_archivos_no_explotan() {
    let dir = tmp_workspace("extremos");
    let db = crear_bd(&dir);

    let vacio = dir.join("vacio.txt");
    std::fs::write(&vacio, "\n\n\n").unwrap();

    let gigante = dir.join("padre.txt");
    std::fs::write(&gigante, "A".repeat(3_000_000)).unwrap();

    let stats = procesar_carpeta_impl(
        vec![
            vacio.to_string_lossy().to_string(),
            gigante.to_string_lossy().to_string(),
        ],
        mapeo(),
        db.clone(),
    );

    assert_eq!(stats.exitosos, 0);
    assert_eq!(stats.errores, 2);
    assert_eq!(contar(&db, "ventas"), 0);

    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 13. Líneas fabricadas con cantidades locas en el límite permitido
// ---------------------------------------------------------------------------

#[test]
fn cantidades_en_limite_no_se_desbordan() {
    let m = MapeoColumnas {
        cantidad: Some(0),
        producto: Some(vec![1]),
        precio_unitario: None,
        total: None,
        descuento: None,
    };
    // 999 piezas × $1,000,000,000,000 (1e12, tope) → finito por guardias.
    let item = parsear_linea("999 LIMITE $1000000000000", &m, 4);
    if let Some(item) = item {
        assert!(finito(item.cantidad), "{item:?}");
    }
}
