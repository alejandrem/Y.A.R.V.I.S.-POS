// Tests del detector: formatos básicos, alternativos, robustez y
// familia C. Se corre contra la API pública del módulo, igual que la
// usa el backend Tauri.

use super::*;

fn detectar(lineas: &[&str]) -> DeteccionMapeo {
    detectar_mapeo(lineas).expect("debió detectar un mapeo")
}

// ---------- Formatos básicos ----------

#[test]
fn formato_clasico_cantidad_producto_precio_total() {
    let lineas = [
        "2 COCA 25.00 50.00",
        "5 PAN 10.00 50.00",
        "1 LECHE 22.50 22.50",
        "3 JABON 15.00 45.00",
    ];
    let d = detectar(&lineas);
    assert_eq!(d.mapeo.cantidad, Some(0));
    assert_eq!(d.mapeo.precio_unitario, Some(-2));
    assert_eq!(d.mapeo.total, Some(-1));
    assert_eq!(d.mapeo.producto, Some(vec![1, -3]));
    assert_eq!(d.confianza, 1.0);
}

#[test]
fn formato_con_dolar_y_producto_de_largo_variable() {
    let lineas = [
        "2 FANTA NARANJA 600ML $15.50 $31.00",
        "1 COCA $25.00 $25.00",
        "10 SABRITAS ORIGINAL $12.00 $120.00",
        "3 GALLETAS EMPERADOR CHOCOLATE $8.50 $25.50",
    ];
    let d = detectar(&lineas);
    assert_eq!(d.mapeo.cantidad, Some(0));
    assert_eq!(d.mapeo.precio_unitario, Some(-2));
    assert_eq!(d.mapeo.total, Some(-1));
    assert_eq!(d.confianza, 1.0);
}

#[test]
fn cantidad_compacta_nx() {
    let lineas = [
        "2x COCA $25.00 $50.00",
        "5x PAN $10.00 $50.00",
        "1x LECHE $22.50 $22.50",
    ];
    let d = detectar(&lineas);
    assert_eq!(d.mapeo.cantidad, Some(0));
    assert_eq!(d.confianza, 1.0);
}

#[test]
fn dolar_separado_del_numero_se_preprocesa() {
    let lineas = [
        "2 COCA $ 25.00 $ 50.00",
        "1 PAN $ 10.00 $ 10.00",
        "6 HUEVO $ 4.50 $ 27.00",
    ];
    let d = detectar(&lineas);
    assert_eq!(d.mapeo.cantidad, Some(0));
    assert_eq!(d.mapeo.total, Some(-1));
    assert_eq!(d.confianza, 1.0);
}

// ---------- Formatos alternativos ----------

#[test]
fn producto_primero_cantidad_en_medio() {
    // Incluye un producto de 2 palabras para exigir índices negativos.
    let lineas = [
        "COCA 2 25.00 50.00",
        "PAN BIMBO 5 10.00 50.00",
        "LECHE 1 22.50 22.50",
    ];
    let d = detectar(&lineas);
    assert_eq!(d.mapeo.cantidad, Some(-3));
    assert_eq!(d.mapeo.precio_unitario, Some(-2));
    assert_eq!(d.mapeo.total, Some(-1));
    assert_eq!(d.confianza, 1.0);
}

#[test]
fn codigo_de_barras_como_primera_columna() {
    let lineas = [
        "7501034567890 1 COCA 25.00 25.00",
        "7501034567891 2 PAN 10.00 20.00",
        "7501034567892 3 LECHE 22.50 67.50",
    ];
    let d = detectar(&lineas);
    assert_eq!(d.mapeo.cantidad, Some(1));
    assert_eq!(d.confianza, 1.0);
}

#[test]
fn columna_de_descuento_numerico() {
    // CANT PROD PRECIO DESCUENTO TOTAL
    let lineas = [
        "2 COCA 25.00 2.00 48.00",
        "5 PAN 10.00 5.00 45.00",
        "1 LECHE 22.50 0.00 22.50",
        "3 JABON 15.00 3.00 42.00",
    ];
    let d = detectar(&lineas);
    assert_eq!(d.mapeo.total, Some(-1));
    assert!(d.confianza >= 0.5);
}

#[test]
fn descuento_porcentual() {
    let lineas = [
        "2 Rockaleta $6.00 10% $10.80",
        "1 Heineken 473ml $28.00 - $28.00",
        "1 Tocino 200g $48.00 - $48.00",
        "4 Gomitas $5.00 10% $18.00",
    ];
    let d = detectar(&lineas);
    assert_eq!(d.mapeo.cantidad, Some(0));
    assert_eq!(d.mapeo.precio_unitario, Some(-3));
    assert_eq!(d.mapeo.total, Some(-1));
    assert_eq!(d.confianza, 1.0);
}

// ---------- Robustez ----------

#[test]
fn lineas_basura_no_tumban_la_deteccion() {
    let lineas = [
        "GRACIAS POR SU COMPRA",
        "2 COCA 25.00 50.00",
        "------------------------",
        "5 PAN 10.00 50.00",
        "TOTAL $100.00",
        "1 LECHE 22.50 22.50",
        "3 JABON 15.00 45.00",
    ];
    let d = detectar(&lineas);
    assert_eq!(d.confianza, 1.0);
    assert_eq!(d.lineas_evaluadas, 4);
}

#[test]
fn nombre_largo_no_cuela_letra_fuera_del_producto() {
    // Si el detector eligiera producto=[1,1] para "FANTA NARANJA 600ML",
    // la regla de cobertura lo tira: quedan letras sin explicar.
    let lineas = [
        "2 FANTA NARANJA 600ML 15.00 30.00",
        "5 PAN BLANCO GRANDE 10.00 50.00",
        "1 LECHE ENTERA 22.50 22.50",
    ];
    let d = detectar(&lineas);
    assert_eq!(d.mapeo.producto, Some(vec![1, -3]));
    assert_eq!(d.confianza, 1.0);
}

#[test]
fn linea_atipica_baja_la_confianza_no_el_resultado() {
    let lineas = [
        "2 COCA 25.00 50.00",
        "5 PAN 10.00 50.00",
        "1 LECHE 22.50 22.50",
        "3 JABON 15.00 45.00",
        "PROMO DEL DIA HOY 99.99 88.88", // no cuadra: baja confianza
    ];
    let d = detectar(&lineas);
    assert_eq!(d.lineas_validas, 4);
    assert!(d.confianza < 1.0);
    assert!(d.confianza >= 0.79);
}

#[test]
fn mapeo_detectado_parsea_cada_linea_sin_alucinaciones() {
    let lineas = [
        "2 COCA 25.00 50.00",
        "7 PAN 10.00 70.00",
        "3 LECHE 22.50 67.50",
        "4 JABON 15.00 60.00",
    ];
    let d = detectar(&lineas);
    for linea in &lineas {
        let item = crate::cerebro::analizador_tickets::parsear_linea(
            linea,
            &d.mapeo,
            linea.split_whitespace().count(),
        )
        .expect("cada línea debe parsear con el mapeo detectado");
        let esperado = item.cantidad * item.precio_unitario;
        assert!(
            (esperado - item.total).abs() <= 0.06,
            "{linea} no cuadró tras el mapeo: {item:?}"
        );
    }
    assert_eq!(d.lineas_validas, 4);
}

// ---------- Familia C: CANT PRODUCTO IMPORTE (un solo importe) ----------

#[test]
fn formato_un_solo_importe_con_productos_repetidos() {
    // Estilo "ticket de la esquina": misma mercancía, mismo precio por KG.
    let mut lineas = Vec::new();
    for _ in 0..6 {
        lineas.extend_from_slice(&[
            "4 PLATANO TABASCO KG 88.00",
            "2 CHILE SERRANO KG 60.00",
            "3 RUFFLES QUESO 60G 54.00",
        ]);
    }
    let d = detectar(&lineas);
    assert_eq!(d.mapeo.cantidad, Some(0));
    assert_eq!(d.mapeo.precio_unitario, None, "un solo importe: sin precio");
    assert_eq!(d.mapeo.total, Some(-1));
    assert_eq!(d.mapeo.producto, Some(vec![1, -2]));
    assert_eq!(d.confianza, 1.0);

    // Y el mapeo detectado produce precios unitarios correctos.
    let item = crate::cerebro::analizador_tickets::parsear_linea(
        "4 PLATANO TABASCO KG 88.00",
        &d.mapeo,
        4,
    )
    .unwrap();
    assert_eq!(item.producto, "PLATANO TABASCO KG");
    assert_eq!(item.cantidad, 4.0);
    assert_eq!(item.precio_unitario, 22.0);
    assert_eq!(item.total, 88.0);
}

#[test]
fn familia_c_rechaza_precios_inconsistentes() {
    // El mismo "producto" cobra totalmente distinto cada vez → NO hay
    // consistencia → no hay verdad matemática → no detectar.
    let lineas = [
        "1 X 10.00", "1 X 97.00", "1 X 55.00", "1 X 33.00", "1 X 76.00",
        "1 Y 20.00", "1 Y 88.00", "1 Y 41.00",
    ];
    assert!(detectar_mapeo(&lineas).is_none());
}

#[test]
fn familia_c_confianza_ignora_productos_no_repetidos() {
    // Los productos que aparecen una sola vez no son observables: ni
    // confirman ni niegan. La confianza es consistentes/repetidas, no
    // consistentes/muestra (esto daba 37% en una carpeta real de 1000
    // tickets con precios perfectos).
    let lineas = [
        "4 PLATANO TABASCO KG 88.00",
        "2 PLATANO TABASCO KG 44.00",
        "1 PLATANO TABASCO KG 22.00",
        "3 RUFFLES QUESO 60G 54.00",
        "1 RUFFLES QUESO 60G 18.00",
        "1 KIWI UNICO 10.00",
        "1 MANGO RARO 20.00",
        "1 PAPAYA SOLA 15.00",
    ];
    let d = detectar(&lineas);
    assert_eq!(d.mapeo.precio_unitario, None);
    assert_eq!(d.lineas_evaluadas, 5, "solo las repetidas son observables");
    assert_eq!(d.lineas_validas, 5);
    assert_eq!(d.confianza, 1.0);
}

#[test]
fn familia_c_tolera_un_cambio_historico_de_precio() {
    // El plátano estaba a $20 y en marzo subió a $22: hay un precio
    // DOMINANTE y la detección sobrevive con confianza razonable.
    let mut lineas = Vec::new();
    for _ in 0..8 {
        lineas.extend_from_slice(&["2 PLATANO KG 40.00", "3 RUFFLES 54.00"]);
    }
    lineas.push("2 PLATANO KG 44.00"); // subió una vez
    lineas.push("2 PLATANO KG 44.00");
    let d = detectar(&lineas);
    assert_eq!(d.mapeo.precio_unitario, None);
    assert!(d.confianza >= 0.6, "confianza {}", d.confianza);
}

#[test]
fn formato_con_precio_gana_sobre_familia_c_en_empate() {
    // En formato clásico (precio + total), la familia A debe ganar
    // aunque la C también "cuadre estructuralmente": el mapeo con
    // precio unitario es mas informativo y verificable línea a línea.
    let mut lineas = Vec::new();
    for _ in 0..5 {
        lineas.extend_from_slice(&["2 COCA 25.00 50.00", "1 PAN 10.00 10.00"]);
    }
    let d = detectar(&lineas);
    assert_eq!(d.mapeo.precio_unitario, Some(-2));
    assert_eq!(d.confianza, 1.0);
}

#[test]
fn formato_de_un_solo_importe_no_se_detecta() {
    let lineas = ["2 COCA 50.00", "5 PAN 50.00", "1 LECHE 22.50"];
    assert!(detectar_mapeo(&lineas).is_none());
}

#[test]
fn texto_libre_sin_estructura_no_se_detecta() {
    let lineas = ["hoy fue un buen dia", "manana compro mas", "el proveedor no vino"];
    assert!(detectar_mapeo(&lineas).is_none());
}

#[test]
fn muestra_demasiado_chica_no_detecta() {
    assert!(detectar_mapeo(&["2 COCA 25.00 50.00", "1 PAN 10.00 10.00"]).is_none());
    assert!(detectar_mapeo(&[]).is_none());
}
