//! estres.rs — Suite de ESTRÉS para destrozar el parseador de tickets.
//!
//! No son tests de "funciona": son ataques directos de fuzzing y dardos a las
//! invariantes. Objetivo: que CUALQUIER entrada (basura, gigante, maliciosa o
//! alucinada) NUNCA produzca panic, NaN/inf ni dinero absurdo en la BD.
//!
//! Invariantes que se vigilan:
//!   1. Ninguna función panfique con entrada arbitraria (incl. UTF-8 roto).
//!   2. Los precios/cantidades SIEMPRE son finitos y con magnitud <= 1e12.
//!   3. Método de pago SIEMPRE cae en el conjunto conocido.
//!   4. El dinero del ticket cuadra: total DB == subtotal + IVA (redondeo 2).

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use src_ia::cerebro::analizador::{
    es_linea_util, extraer_fecha_hora_regex, extraer_metodo_pago, limpiar_precio, parsear_linea,
    MapeoColumnas, PRECIO_MAXIMO,
};
use src_ia::cerebro::filtrador::{es_categoria, limpiar_producto};
use src_ia::cerebro::lote::procesar_carpeta_impl;
use src_ia::cerebro::vinculador::normalizar;
use src_ia::formatos::lector_csv::parsear_csv;
use src_ia::formatos::lector_txt::parsear_catalogo_visual;

// ---------------------------------------------------------------------------
// Fuzzer determinista (LCG, sin dependencias)
// ---------------------------------------------------------------------------

struct Fuzzer(u64);

impl Fuzzer {
    fn rnd(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 8
    }

    fn usize(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.rnd() % n as u64) as usize
        }
    }

    fn char_alp(&mut self) -> char {
        const ALP: &[char] = &[
            '0', '1', '2', '9', '.', ',', '$', '-', '=', '*', '~', '>', 'A', 'Z', 'a', 'z', ' ',
            '\t', '®', '²', 'á', 'ñ', '€', '#', '@', '%', 'O', 'I', ':', ';', '|', '\n', '(', ')',
            '[', ']', '"', '\'', 'é', '_', '/', '\\', '\0', '─', '+', '&', '?', '!', 'L', 'C', 'F',
        ];
        ALP[self.usize(ALP.len())]
    }

    fn string(&mut self, max_len: usize) -> String {
        let n = self.usize(max_len);
        (0..n).map(|_| self.char_alp()).collect()
    }
}

fn mapeo() -> MapeoColumnas {
    serde_json::from_str(r#"{"cantidad": 0, "producto": [1], "precio_unitario": 2, "total": 3}"#)
        .unwrap()
}

fn finito(x: f64) -> bool {
    x.is_finite() && x.abs() <= PRECIO_MAXIMO
}

// ---------------------------------------------------------------------------
// 1. Fuzzing masivo: ningún parseador debe panfiquear
// ---------------------------------------------------------------------------

#[test]
fn fuzzing_basura_no_panica_en_ningun_parser() {
    let mut fz = Fuzzer(0x5EED_C0DE_2026);
    let metodos_validos = ["efectivo", "tarjeta", "transferencia", "cheque"];

    for i in 0..30_000 {
        let s = fz.string(120);
        let _ = es_linea_util(&s);
        let _ = limpiar_producto(&s);
        let _ = es_categoria(&s);
        let _ = normalizar(&s);

        let (f, h) = extraer_fecha_hora_regex(&s);
        if let Some(h) = &h {
            // La hora SIEMPRE sale con forma "HH:MM" → len 5.
            assert!(h.len() == 5, "hora corrupta en iter {i}: {h:?} de {s:?}");
        }
        if let Some(fecha) = &f {
            assert!(fecha.len() == 10 && fecha.as_bytes()[4] == b'-', "fecha corrupta {fecha:?}");
        }

        let metodo = extraer_metodo_pago(&s);
        assert!(
            metodos_validos.contains(&metodo.as_str()),
            "método de pago fuera del conjunto: {metodo:?} en iter {i}"
        );

        let _ = parsear_catalogo_visual(&s);
        let _ = parsear_csv(&s);
        let _ = limpiar_precio(&s);

        // parsear_linea con mapeos hostiles (índices negativos incluidos).
        let mapeo_hostil = MapeoColumnas {
            cantidad: Some(i % 7 - 1),
            producto: Some(vec![i % 5, i % 3]),
            precio_unitario: Some(i % 4),
            total: Some(i % 6),
            descuento: Some(i % 2),
        };
        let _ = parsear_linea(&s, &mapeo_hostil, 100);

        let _ = parsear_linea(&s, &mapeo(), 100);
    }
}

// ---------------------------------------------------------------------------
// 2. Cualquier Item salido del parseo es FINITO y sobrio
// ---------------------------------------------------------------------------

#[test]
fn ningun_precio_es_nan_inf_o_absurdo() {
    let mut fz = Fuzzer(0xDEAD_BEEF);
    for _ in 0..50_000 {
        let s = fz.string(80);
        for mapeo in [
            mapeo(),
            MapeoColumnas {
                cantidad: None,
                producto: Some(vec![0, 1, 2]),
                precio_unitario: None,
                total: Some(-1),
                descuento: Some(-2),
            },
            MapeoColumnas {
                cantidad: Some(0),
                producto: Some(vec![3]),
                precio_unitario: Some(2),
                total: Some(1),
                descuento: None,
            },
        ] {
            if let Some(item) = parsear_linea(&s, &mapeo, 20) {
                assert!(finito(item.cantidad), "cantidad loca {item:?} de {s:?}");
                assert!(finito(item.precio_unitario), "precio loco {item:?} de {s:?}");
                assert!(finito(item.total), "total loco {item:?} de {s:?}");
                if let Some(d) = item.descuento {
                    assert!(finito(d), "descuento loco {item:?} de {s:?}");
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 3. El bug viral `inf`/`nan`/gasolina ya no entra a la BD
// ---------------------------------------------------------------------------

#[test]
fn inf_nan_y_magnitudes_absurdas_se_neutralizan() {
    // Antes de los guardias: "inf" se parseaba como precio infinito.
    let precio_roto = MapeoColumnas {
        cantidad: Some(0),
        producto: Some(vec![1]),
        precio_unitario: None,
        total: Some(2),
        descuento: None,
    };
    for (linea, nombre) in [
        ("2 COCA inf", "COCA"),
        ("2 COCA nan", "COCA"),
        ("2 COCA -inf", "COCA"),
    ] {
        let item = parsear_linea(linea, &precio_roto, 4)
            .unwrap_or_else(|| panic!("no parseó {linea:?}"));
        assert_eq!(item.producto, nombre, "{linea}");
        assert_eq!(item.total, 0.0, "{linea} dejó pasar dinero roto");
        assert!(finito(item.total), "{linea}");
    }

    // Un número de 32 dígitos es folio/número de ticket → se descarta (RE_NUM_FINAL).
    let folio = "2 COCA 99999999999999999999999999999999";
    assert!(parsear_linea(folio, &precio_roto, 4).is_none(), "folio gigante entró como item");

    // CSV: una celda "inf" no debe convertirse en precio infinito.
    let csv_roto = "nombre,venta\nX,inf\n";
    let res = parsear_csv(csv_roto);
    assert!(!res.is_empty());
    assert_eq!(res[0].precio_venta, 0.0, "CSV dejó pasar inf");

    // Catálogo visual: mismo escudo.
    let txt_roto = "PRODUCTO -- inf $5\n";
    let res = parsear_catalogo_visual(txt_roto);
    assert!(res.is_empty() || res.iter().all(|p| finito(p.precio_venta)));
}

// ---------------------------------------------------------------------------
// 4. Dardos financieros: dinero real mexicano (millones) entra limpio
// ---------------------------------------------------------------------------

#[test]
fn numeros_mexicanos_millonarios_cuadran() {
    let m = MapeoColumnas {
        cantidad: Some(0),
        producto: Some(vec![1]),
        precio_unitario: Some(2),
        total: Some(3),
        descuento: None,
    };
    let item = parsear_linea("2 PRODUCTO $1,234,567.89 $2,469,135.78", &m, 5).unwrap();
    assert_eq!(item.cantidad, 2.0);
    assert_eq!(item.precio_unitario, 1234567.89);
    assert_eq!(item.total, 2469135.78);

    let item2 = parsear_linea("$1,234,567.89 PRODUCTO 7.5", &mapeo(), 4);
    assert!(item2.is_none() || item2.unwrap().total.is_finite());
}

// ---------------------------------------------------------------------------
// 5. Línea con miles de columnas (bomba de tokens)
// ---------------------------------------------------------------------------

#[test]
fn linea_gigante_de_5000_columnas_no_explota() {
    let mut tokens: Vec<String> = (0..4997).map(|i| i.to_string()).collect();
    tokens.extend_from_slice(&["AJUA".to_string(), "12.5".to_string(), "99".to_string()]);
    let linea_gigante = tokens.join(" ");

    let m = MapeoColumnas {
        cantidad: None,
        producto: Some(vec![4997]),
        precio_unitario: Some(4998),
        total: Some(4999),
        descuento: Some(101),
    };
    assert!(es_linea_util(&linea_gigante));
    let item = parsear_linea(&linea_gigante, &m, 5000).expect("debió parsear la mega-línea");
    assert_eq!(item.producto, "AJUA");
    assert_eq!(item.precio_unitario, 12.5);
    assert_eq!(item.total, 99.0);
}

// ---------------------------------------------------------------------------
// 6. Fechas/horas alucinantes nunca panfiquean ni corrompen el formato
// ---------------------------------------------------------------------------

#[test]
fn fechas_y_horas_imposibles_no_rompen() {
    for s in [
        "31/02/2026",
        "99/99/9999",
        "13/13/13",
        "32:99:99",
        "25:00 PM",
        "0:00 AM",
        "9999-99-99",
        "29 de febrero de 2023",
        "1:1:1",
        "",
        "::::",
        "FECHA: 12/05/2026 75:99 PM",
    ] {
        let (f, h) = extraer_fecha_hora_regex(s);
        if let Some(fecha) = &f {
            assert!(fecha.len() == 10 && fecha.as_bytes()[4] == b'-', "{fecha:?} de {s:?}");
        }
        if let Some(h) = &h {
            assert!(h.len() == 5, "{h:?} de {s:?}");
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Métodos de pago manipulados → siempre en el conjunto conocido
// ---------------------------------------------------------------------------

#[test]
fn metodos_de_pago_trampa_se_mantienen_en_el_conjunto() {
    for texto in [
        "METODO DE PAGO: EFECTIVO FALSO",
        "TARJETA DE CREDITO VENCIDA $1.00",
        "PAGO CON: TRANSFERENCIA (FRAUDE)",
        "FORMA DE PAGO: CHEQUE POSFECHADO",
        "EFECTIVO........... $1,234.56",
        "TARJETA DEBITO $500.00\nTOTAL $500.00",
        "no se pudo cobrar ningun metodo xd",
        "METODO DE PAGO:" ,
    ] {
        let m = extraer_metodo_pago(texto);
        assert!(
            ["efectivo", "tarjeta", "transferencia", "cheque"].contains(&m.as_str()),
            "{texto:?} → {m:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// 8. Separadores puros y basura total nunca son "producto"
// ---------------------------------------------------------------------------

#[test]
fn separadores_y_glitch_no_pasan_a_producto() {
    for s in [
        "----------",
        "===== - - - = ==",
        "~~~*~=",
        ".... ....",
        "    ",
        "\t\t",
        "-----=====-----",
        "***~~~",
    ] {
        assert!(!es_linea_util(s), "separador como util: {s:?}");
        assert!(parsear_linea(s, &mapeo(), 10).is_none(), "separador como item: {s:?}");
    }
}

// ---------------------------------------------------------------------------
// 9. Unicode ZALGO / glifos rotos / UTF-8 inválido
// ---------------------------------------------------------------------------

#[test]
fn zalgo_y_utf8_roto_no_explotan() {
    let zalgo = format!("CAF{}", "́".repeat(500));
    let _ = limpiar_producto(&zalgo);
    let _ = es_linea_util(&zalgo);
    let _ = parsear_linea(&zalgo, &mapeo(), 3);

    // Bytes inválidos forzados a lossy (equivale a open(errors=ignore)).
    let bytes = [0xFFu8, 0xFE, 0x00, 0x41, 0x22, 0x5C, 0xF0, 0x28, 0x8C];
    let lossy = String::from_utf8_lossy(&bytes).into_owned();
    let _ = parsear_catalogo_visual(&lossy);
    let _ = limpiar_producto(&lossy);
    let _ = extraer_metodo_pago(&lossy);
}

// ---------------------------------------------------------------------------
// 10. Ticket GIGANTE (20 mil líneas) entra entero a la BD sin desmadre
// ---------------------------------------------------------------------------

fn tmp_workspace(nombre: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("estres_{}_{}", nombre, nanos));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn crear_bd(dir: &Path) -> String {
    let path = dir.join("estres.db");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        "CREATE TABLE productos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            precio_venta REAL DEFAULT 0,
            stock REAL DEFAULT 0,
            vendido REAL DEFAULT 0
         );
         CREATE TABLE ventas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            total REAL, subtotal REAL, iva REAL,
            cajero TEXT, metodo_pago TEXT, estado TEXT, fecha TEXT
         );
         CREATE TABLE detalle_ventas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            venta_id INTEGER, producto_nombre TEXT,
            cantidad REAL, precio_unitario REAL,
            descuento REAL, subtotal REAL
         );",
    )
    .unwrap();
    drop(conn);
    path.to_string_lossy().to_string()
}

fn contar(db: &str, tabla: &str) -> i64 {
    let conn = Connection::open(db).unwrap();
    conn.query_row(&format!("SELECT COUNT(*) FROM {tabla}"), [], |r| r.get(0))
        .unwrap()
}

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

    let stats = procesar_carpeta_impl(vec![ruta.to_string_lossy().to_string()], mapeo(), db.clone());

    assert_eq!(stats.exitosos, 1);
    assert_eq!(stats.errores, 0);
    assert_eq!(stats.items_insertados, 20_000);
    assert_eq!(stats.duplicados_detectados, 0);
    assert_eq!(contar(&db, "detalle_ventas"), 20_000);
    // Dinero: 20,000 × (2 × $60) = $2,400,000 ; × 1.16 = $2,784,000.00
    assert_eq!(stats.resumen_ventas[0].total, 2_784_000.0);

    let (total, subtotal, iva): (f64, f64, f64) = Connection::open(&db)
        .unwrap()
        .query_row("SELECT total, subtotal, iva FROM ventas", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .unwrap();
    assert_eq!(subtotal, 2_400_000.0);
    assert_eq!(iva, 384_000.0);
    assert_eq!(total, 2_784_000.0);

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