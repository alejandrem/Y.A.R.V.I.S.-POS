// Tests del lector TXT (valores verificados contra Python).

use super::linea::{extraer_nombre_cantidad, parsear_linea_catalogo};
use super::visual::parsear_catalogo_visual;
use super::super::ProductoCatalogo;

fn parse_una(linea: &str) -> ProductoCatalogo {
    let res = parsear_linea_catalogo(linea, "");
    assert!(res.len() == 1, "esperaba 1 producto, dio {}", res.len());
    res[0].clone()
}

// ---------- Bug 8: separador no contamina el nombre ----------

#[test]
fn bug8_separador_no_contamina_nombre() {
    for (linea, nombre, venta) in [
        ("Coca-Cola 600ML -- $25 $18", "COCA-COLA 600ML", 25.0),
        ("AGUA 1500 -- $20 $16", "AGUA 1500", 20.0),
        ("PAN BLANCO 12 -- $15 $10", "PAN BLANCO 12", 15.0),
        ("TOTAL -- $1,234.56 $1,000", "TOTAL", 1234.56),
        ("Producto -- $10 $5", "PRODUCTO", 10.0),
        ("Producto - $10 - $5", "PRODUCTO", 10.0),
        ("Producto = $10 $5", "PRODUCTO", 10.0),
    ] {
        let p = parse_una(linea);
        assert_eq!(p.nombre, nombre, "{linea}");
        assert_eq!(p.precio_venta, venta, "{linea}");
    }
}

// ---------- Tablas SIN separador: nombre conserva volúmenes ----------

#[test]
fn tablas_sin_separador_con_volumen() {
    for (linea, nombre, stock) in [
        ("Coca-Cola 600ML $25 $18", "COCA-COLA 600ML", 0),
        ("Coca-Cola 600ML 12 $29 $23", "COCA-COLA 600ML", 12),
        ("Coca-Cola 600ML 12 29 23", "COCA-COLA 600ML", 12),
        ("Sabritas 16 12", "SABRITAS", 0),
        ("Sabritas 60 16 12", "SABRITAS", 60),
    ] {
        let p = parse_una(linea);
        assert_eq!(p.nombre, nombre, "{linea}");
        assert_eq!(p.stock, stock, "{linea}");
    }
}

#[test]
fn cantidad_al_inicio() {
    let p = parse_una("10Producto - $10 $5");
    assert_eq!(p.nombre, "PRODUCTO");
    assert_eq!(p.stock, 10);
}

// ---------- A4: volúmenes que NO deben separarse como cantidad ----------

#[test]
fn extraer_nombre_cantidad_no_come_volumen() {
    for (texto, esperado) in [
        (
            "COCA-COLA 600ML 2880",
            ("COCA-COLA 600ML 2880".to_string(), 0),
        ),
        ("COCA-COLA 600", ("COCA-COLA".to_string(), 600)),
        (
            "Coca-Cola 600 ml 12",
            ("Coca-Cola 600 ml 12".to_string(), 0),
        ),
        ("Agua 1500", ("Agua 1500".to_string(), 0)),
    ] {
        assert_eq!(extraer_nombre_cantidad(texto), esperado, "{texto}");
    }
}

#[test]
fn extraer_nombre_cantidad_si_es_pieza_pequena() {
    assert_eq!(
        extraer_nombre_cantidad("Producto 12"),
        ("Producto".to_string(), 12)
    );
}

// ---------- End-to-end con categorías y múltiples productos ----------

#[test]
fn catalogo_end_to_end() {
    let texto = "BEBIDAS
Coca-Cola 600ML -- $25 $18 | AGUA 1500 -- $20 $16
ABARROTES
PAN BLANCO 12 -- $15 $10 | SABRITAS 16 12
";
    let productos = parsear_catalogo_visual(texto);
    let nombres: Vec<&str> = productos.iter().map(|p| p.nombre.as_str()).collect();
    assert_eq!(
        nombres,
        vec!["COCA-COLA 600ML", "AGUA 1500", "PAN BLANCO 12", "SABRITAS"]
    );
    let categorias: Vec<&str> = productos.iter().map(|p| p.categoria.as_str()).collect();
    assert_eq!(
        categorias,
        vec!["BEBIDAS", "BEBIDAS", "ABARROTES", "ABARROTES"]
    );
    for p in &productos {
        assert!(
            !p.nombre.ends_with("--"),
            "nombre contaminado: {}",
            p.nombre
        );
    }
}
