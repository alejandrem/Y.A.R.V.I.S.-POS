// Tests de la capa de comandos: rechazar archivos que no son respaldos
// y resolver la contraseña sin exponerla nunca al frontend.

use super::*;

fn dir_temp(nombre: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "yarvis-comandos-{nombre}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&base).unwrap();
    base
}

#[test]
fn validar_archivo_rechaza_lo_que_no_es_respaldo() {
    let dir = dir_temp("validar");
    let db = dir.join("yarvis.db");
    std::fs::write(&db, b"x").unwrap();
    let txt = dir.join("notas.txt");
    std::fs::write(&txt, b"x").unwrap();

    assert!(validar_archivo(&db.to_string_lossy()).is_err());
    assert!(validar_archivo(&txt.to_string_lossy()).is_err());
    assert!(validar_archivo(&dir.join("no-existe.yarvisbak").to_string_lossy()).is_err());

    let bueno = dir.join("yarvis-20260101-120000.yarvisbak");
    std::fs::write(&bueno, b"x").unwrap();
    assert!(validar_archivo(&bueno.to_string_lossy()).is_ok());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn resolver_pass_prioriza_lo_que_escribe_el_admin() {
    let cfg = ConfigRespaldo::default();
    assert_eq!(
        resolver_pass(&cfg, Some("clave-escrita".into())).unwrap(),
        "clave-escrita"
    );
    assert!(resolver_pass(&cfg, Some("   ".into())).is_err());
}

#[test]
fn resolver_pass_usa_la_guardada_si_no_viene_ninguna() {
    let dir = dir_temp("resolver");
    let destino = dir.to_string_lossy().to_string();
    let cfg = config::aplicar(
        &ConfigRespaldo::default(),
        ConfigEntrada {
            activo: true,
            destino: Some(destino),
            cada_horas: 24,
            retener: 30,
            passphrase: Some("clave-guardada-ok".into()),
        },
    )
    .unwrap();

    assert_eq!(
        resolver_pass(&cfg, None).unwrap(),
        "clave-guardada-ok"
    );
    assert_eq!(
        resolver_pass(&cfg, Some("".into())).unwrap(),
        "clave-guardada-ok"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn el_respaldo_no_puede_vivir_dentro_de_la_carpeta_de_datos() {
    let base = dir_temp("dentro");
    let sub = base.join("backups");
    std::fs::create_dir_all(&sub).unwrap();
    let hermano = base.with_file_name(format!(
        "{}-otro",
        base.file_name().unwrap().to_string_lossy()
    ));
    std::fs::create_dir_all(&hermano).unwrap();

    assert!(es_dentro_de(&base, &base), "la carpeta exacta debe rechazarse");
    assert!(es_dentro_de(&base, &sub), "una subcarpeta debe rechazarse");
    assert!(!es_dentro_de(&base, &hermano), "una carpeta vecina sí es válida");

    let _ = std::fs::remove_dir_all(&base);
    let _ = std::fs::remove_dir_all(&hermano);
}
