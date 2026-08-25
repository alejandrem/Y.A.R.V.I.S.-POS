// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Rate limiter del login (backventanas/auth).
// Ventana móvil por fallos: bloqueo al llegar al máximo, permiso debajo,
// limpieza al login exitoso, e integración con AuthState::default().
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use std::time::Duration;
use yarvis_app_lib::backventanas::auth::{AuthState, LoginRateLimiter};

/// `LoginRateLimiter::con_fallos` es #[cfg(test)] (solo visible dentro del
/// crate), así que aquí replicamos el constructor vía API pública.
fn limiter_con_fallos(count: u32, max: u32) -> LoginRateLimiter {
    let l = LoginRateLimiter::new(max, Duration::from_secs(3600));
    for _ in 0..count {
        l.registrar_fallo();
    }
    l
}

#[test]
fn limite_alcanzado_bloquea_con_segundos_positivos() {
    let limiter = limiter_con_fallos(5, 5);
    match limiter.verificar() {
        Err(segundos) => assert!(segundos > 0, "segundos de espera deben ser > 0, got {segundos}"),
        Ok(()) => panic!("con max fallos debe BLOQUEAR el intento"),
    }
}

#[test]
fn debajo_del_limite_permite_intento() {
    let limiter = limiter_con_fallos(4, 5);
    assert_eq!(limiter.verificar(), Ok(()));
}

#[test]
fn exito_registrado_limpia_historial_de_fallos() {
    let limiter = limiter_con_fallos(5, 5);
    assert!(limiter.verificar().is_err());
    limiter.registrar_exito();
    assert_eq!(limiter.verificar(), Ok(()), "login exitoso debe resetear el contador");
}

#[test]
fn auth_state_default_bloquea_tras_cinco_fallos() {
    // Integración: AuthState::default() usa ventana 300s / 5 intentos.
    let state = AuthState::default();
    for _ in 0..5 {
        state.rate_limiter.registrar_fallo();
    }
    match state.rate_limiter.verificar() {
        Err(segundos) => assert!(segundos > 0 && segundos <= 300),
        Ok(()) => panic!("AuthState debe bloquear tras 5 fallos"),
    }
}

#[test]
fn un_solo_fallo_no_bloquea_auth_state() {
    let state = AuthState::default();
    state.rate_limiter.registrar_fallo();
    assert_eq!(state.rate_limiter.verificar(), Ok(()));
}
