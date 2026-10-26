// Tests de configuración: activar respaldos sin contraseña, retención
// absurda, rutas relativas y fugas de la contraseña en el JSON.

use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

const PASS: &str = "clave-de-respaldo-ok";

fn dir_temp(nombre: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "yarvis-config-{nombre}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&base).unwrap();
    base
}

fn entrada(activo: bool, destino: Option<&str>, pass: Option<&str>) -> ConfigEntrada {
    ConfigEntrada {
        activo,
        destino: destino.map(str::to_string),
        cada_horas: 24,
        retener: 30,
        passphrase: pass.map(str::to_string),
    }
}

#[test]
fn sin_archivo_devuelve_defaults() {
    let dir = dir_temp("default");
    let cfg = leer_en(&dir.join("no-existe.json")).unwrap();
    assert!(!cfg.activo);
    assert_eq!(cfg.cada_horas, 24);
    assert_eq!(cfg.retener, 30);
    assert!(cfg.destino.is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn activar_sin_contrasena_se_rechaza() {
    let dir = dir_temp("sin-pass");
    let destino = dir.to_string_lossy().to_string();
    let err = aplicar(&ConfigRespaldo::default(), entrada(true, Some(&destino), None)).unwrap_err();
    assert!(err.contains("contraseña"), "{err}");
}

#[test]
fn contrasena_corta_o_ruta_relativa_se_rechazan() {
    let actual = ConfigRespaldo::default();
    let err = aplicar(&actual, entrada(false, Some("respaldo"), Some("corta"))).unwrap_err();
    assert!(err.contains("al menos"), "{err}");

    let err = aplicar(&actual, entrada(true, Some("respaldo"), Some(PASS))).unwrap_err();
    assert!(err.contains("absoluta"), "{err}");
}

#[test]
fn intervalos_y_retencion_fuera_de_rango_se_rechazan() {
    let mut cfg = ConfigRespaldo::default();
    cfg.cada_horas = 0;
    assert!(cfg.validar().is_err());
    cfg.cada_horas = 169;
    assert!(cfg.validar().is_err());
    cfg.cada_horas = 24;
    cfg.retener = 0;
    assert!(cfg.validar().is_err());
    cfg.retener = 101;
    assert!(cfg.validar().is_err());
}

#[test]
fn guarda_y_recupera_sin_filtrar_la_contrasena() {
    let dir = dir_temp("roundtrip");
    let destino = dir.join("respaldos");
    fs::create_dir_all(&destino).unwrap();
    let ruta = dir.join(ARCHIVO);

    let cfg = aplicar(
        &ConfigRespaldo::default(),
        entrada(true, Some(&destino.to_string_lossy()), Some(PASS)),
    )
    .unwrap();
    guardar_en(&ruta, &cfg).unwrap();

    let leida = leer_en(&ruta).unwrap();
    assert_eq!(leida, cfg);
    assert_eq!(leida.passphrase().unwrap().as_deref(), Some(PASS));
    assert!(leida.publica().tiene_passphrase);
    assert!(!serde_json::to_string(&leida.publica()).unwrap().contains(PASS));

    let json = fs::read_to_string(&ruta).unwrap();
    assert!(!json.contains(PASS), "la contraseña quedó en texto claro");

    let sobrantes = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
        .count();
    assert_eq!(sobrantes, 0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn archivo_corrupto_da_error_y_no_se_borra() {
    let dir = dir_temp("corrupto");
    let ruta = dir.join(ARCHIVO);
    fs::write(&ruta, "{esto no es json").unwrap();

    let err = leer_en(&ruta).unwrap_err();
    assert!(err.contains("corrupta"), "{err}");
    assert!(ruta.exists());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn conserva_la_contrasena_si_la_entrada_no_la_trae() {
    let dir = dir_temp("conservar");
    let destino = dir.to_string_lossy().to_string();
    let cfg = aplicar(
        &ConfigRespaldo::default(),
        entrada(true, Some(&destino), Some(PASS)),
    )
    .unwrap();

    let igual = aplicar(&cfg, entrada(true, Some(&destino), None)).unwrap();
    assert_eq!(igual.passphrase_protegida, cfg.passphrase_protegida);
    assert_eq!(igual.passphrase().unwrap().as_deref(), Some(PASS));

    let _ = fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn el_archivo_tiene_permisos_0600() {
    use std::os::unix::fs::PermissionsExt;

    let dir = dir_temp("permisos");
    let ruta = dir.join(ARCHIVO);
    guardar_en(&ruta, &ConfigRespaldo::default()).unwrap();
    let modo = fs::metadata(&ruta).unwrap().permissions().mode() & 0o777;
    assert_eq!(modo, 0o600);
    let _ = fs::remove_dir_all(&dir);
}
