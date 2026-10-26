// Tests adversarios del cifrado: contraseña equivocada, bytes alterados,
// archivo truncado y contraseñas débiles. Un falso positivo aquí
// significado perder los datos del cliente para siempre.

use super::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const PASS: &str = "clave-segura-2026";

fn dir_temp(nombre: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "yarvis-cifrado-{nombre}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&base).unwrap();
    base
}

#[test]
fn ida_y_vuelta_con_binarios_y_acentos() {
    let original = b"Venta: 12.50\nCliente: Jos\xc3\xa9\n\x00\x01\x02\xff".to_vec();
    let blob = cifrar(&original, PASS).unwrap();
    assert_eq!(descifrar(&blob, PASS).unwrap(), original);
    assert_eq!(&blob[..MAGIA.len()], MAGIA);
    assert_ne!(blob, original);
}

#[test]
fn contrasena_equivocada_no_devuelve_nada() {
    let blob = cifrar(b"datos secretos", PASS).unwrap();
    let err = descifrar(&blob, "otra-clave-distinta").unwrap_err();
    assert!(err.contains("contraseña incorrecta") || err.contains("dañado"), "{err}");
}

#[test]
fn un_bit_alterado_en_el_cifrado_invalida_el_archivo() {
    let mut blob = cifrar(b"datos secretos", PASS).unwrap();
    let ultimo = blob.len() - 1;
    blob[ultimo] ^= 0x01;
    assert!(descifrar(&blob, PASS).is_err());
}

#[test]
fn cambiar_la_magia_invalida_el_archivo() {
    let mut blob = cifrar(b"datos", PASS).unwrap();
    blob[0] = b'X';
    let err = descifrar(&blob, PASS).unwrap_err();
    assert!(err.contains("formato"), "{err}");
}

#[test]
fn archivo_truncado_falla_sin_panic() {
    let blob = cifrar(b"datos largos para truncar", PASS).unwrap();
    for corte in [0, 3, CABECERA, blob.len() - 1] {
        assert!(descifrar(&blob[..corte], PASS).is_err(), "corte {corte}");
    }
}

#[test]
fn exige_contrasena_minima_y_no_vacia() {
    assert!(cifrar(b"x", "corta").is_err());
    assert!(cifrar(b"x", "         ").is_err());
    let blob = cifrar(b"x", PASS).unwrap();
    assert!(descifrar(&blob, "").is_err());
}

#[test]
fn archivo_cifrado_no_contiene_el_texto_en_claro() {
    let dir = dir_temp("archivo");
    let origen = dir.join("origen.db");
    let destino = dir.join("salida.yarvisbak");
    let secreto = b"PRODUCTO-SECRETO-QUE-NO-DEBE-VERSE";
    fs::write(&origen, secreto).unwrap();

    cifrar_archivo(&origen, &destino, PASS).unwrap();
    let bytes = fs::read(&destino).unwrap();
    assert!(
        !bytes.windows(secreto.len()).any(|v| v == secreto),
        "el texto plano quedó dentro del respaldo"
    );

    let recuperado = dir.join("recuperado.db");
    descifrar_archivo(&destino, &recuperado, PASS).unwrap();
    assert_eq!(fs::read(&recuperado).unwrap(), secreto);

    let _ = fs::remove_dir_all(&dir);
}
