// Tests del calendario de respaldos: nunca respaldar si está apagado,
// respaldar la primera vez, respetar el intervalo y no entrar en bucle
// si el reloj de la tienda se atrasa.

use super::*;
use chrono::TimeZone;

fn cfg_base() -> ConfigRespaldo {
    ConfigRespaldo {
        activo: true,
        destino: Some("C:\\Respaldos".to_string()),
        cada_horas: 24,
        retener: 30,
        ultimo_exito: None,
        passphrase_protegida: Some("x".to_string()),
    }
}

fn fecha(anio: i32, mes: u32, dia: u32, hora: u32) -> DateTime<Local> {
    Local.with_ymd_and_hms(anio, mes, dia, hora, 0, 0).unwrap()
}

#[test]
fn apagado_o_sin_destino_no_toca() {
    let ahora = fecha(2026, 10, 26, 12);
    let mut cfg = cfg_base();
    cfg.activo = false;
    assert!(!toca(&cfg, ahora));

    let mut cfg = cfg_base();
    cfg.destino = None;
    assert!(!toca(&cfg, ahora));
}

#[test]
fn la_primera_vez_siempre_toca() {
    assert!(toca(&cfg_base(), fecha(2026, 10, 26, 12)));
}

#[test]
fn respeta_el_intervalo_configurado() {
    let mut cfg = cfg_base();
    cfg.ultimo_exito = Some(fecha(2026, 10, 25, 12).to_rfc3339());
    assert!(!toca(&cfg, fecha(2026, 10, 26, 11)));
    assert!(toca(&cfg, fecha(2026, 10, 26, 12)));
    assert!(toca(&cfg, fecha(2026, 10, 26, 13)));
}

#[test]
fn un_ultimo_exito_corrupto_no_bloquea_para_siempre() {
    let mut cfg = cfg_base();
    cfg.ultimo_exito = Some("no-es-fecha".to_string());
    assert!(toca(&cfg, fecha(2026, 10, 26, 12)));
}

#[test]
fn reloj_atrasado_no_provoca_bucle_de_respaldos() {
    let mut cfg = cfg_base();
    // El último respaldo "fue" en el futuro porque el reloj se atrasó.
    cfg.ultimo_exito = Some(fecha(2026, 10, 27, 12).to_rfc3339());
    assert!(!toca(&cfg, fecha(2026, 10, 26, 12)));
}
