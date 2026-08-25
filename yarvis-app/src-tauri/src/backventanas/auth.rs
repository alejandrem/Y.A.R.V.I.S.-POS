use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Rol de la sesión activa dentro de la aplicación.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Admin,
    Employee,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub user_id: i64,
    pub role: Role,
    pub name: String,
}

/// Rate limiter del login en memoria: máximo de FALLOS por ventana móvil.
/// Mitiga fuerza bruta offline iterando hashes de empleados (el login es
/// solo por contraseña). En éxito se resetea el contador.
pub struct LoginRateLimiter {
    max_intentos: u32,
    ventana: Duration,
    fallos: Mutex<HashMap<String, (u32, Instant)>>,
}

/// Clave única del limitador: en esta app el login no distingue usuario
/// antes de autenticar (es solo contraseña), así que se limita global.
const CLAVE_LOGIN: &str = "login";

impl LoginRateLimiter {
    pub fn new(max_intentos: u32, ventana: Duration) -> Self {
        Self {
            max_intentos,
            ventana,
            fallos: Mutex::new(HashMap::new()),
        }
    }

    /// ¿Se permite otro intento? `segundos_espera` indica cuánto falta
    /// para reintentar cuando la respuesta es false.
    pub fn verificar(&self) -> Result<(), u64> {
        let mut mapa = self
            .fallos
            .lock()
            .map_err(|_| self.ventana.as_secs().max(1))?;
        if let Some((count, desde)) = mapa.get(CLAVE_LOGIN) {
            let transcurrido = desde.elapsed();
            if transcurrido < self.ventana && *count >= self.max_intentos {
                let resta = (self.ventana - transcurrido).as_secs().max(1);
                return Err(resta);
            }
            if transcurrido >= self.ventana {
                mapa.remove(CLAVE_LOGIN); // ventana expirada: limpiar
            }
        }
        Ok(())
    }

    /// Registra un fallo de login.
    pub fn registrar_fallo(&self) {
        if let Ok(mut mapa) = self.fallos.lock() {
            match mapa.get_mut(CLAVE_LOGIN) {
                Some((conteo, desde)) => {
                    if desde.elapsed() >= self.ventana {
                        // La ventana expiró entre verificar y fallar: reinicia.
                        *desde = Instant::now();
                        *conteo = 1;
                    } else {
                        *conteo += 1;
                    }
                }
                None => {
                    mapa.insert(CLAVE_LOGIN.to_string(), (1, Instant::now()));
                }
            }
        }
    }

    /// Login exitoso: limpia el historial de fallos.
    pub fn registrar_exito(&self) {
        if let Ok(mut mapa) = self.fallos.lock() {
            mapa.remove(CLAVE_LOGIN);
        }
    }
}

#[cfg(test)]
impl LoginRateLimiter {
    /// Constructor de prueba pre-cargado con `count` fallos.
    pub fn con_fallos(count: u32, max: u32) -> Self {
        let l = Self::new(max, Duration::from_secs(3600));
        for _ in 0..count {
            l.registrar_fallo();
        }
        l
    }
}

/// Estado de autenticación en memoria.
///
/// Tauri conserva este estado en el proceso nativo; el frontend no puede
/// cambiar el rol enviando un parámetro arbitrario a cada comando.
pub struct AuthState {
    session: Mutex<Option<Session>>,
    pub rate_limiter: LoginRateLimiter,
}

impl Default for AuthState {
    fn default() -> Self {
        // OWASP: 5 intentos fallidos → bloqueo ~5 minutos (ventana deslizante).
        Self {
            session: Mutex::new(None),
            rate_limiter: LoginRateLimiter::new(5, Duration::from_secs(300)),
        }
    }
}

impl AuthState {
    pub fn login(&self, user_id: i64, role: Role, name: String) {
        if let Ok(mut session) = self.session.lock() {
            *session = Some(Session {
                user_id,
                role,
                name,
            });
        }
    }

    /// ¿La sesión activa es de empleado (no admin)? Para personalizar
    /// el system prompt del chat según quién le escribe a Y.A.R.V.I.S.
    pub fn es_empleado(&self) -> bool {
        self.session
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|ses| matches!(ses.role, Role::Employee)))
            .unwrap_or(false)
    }

    pub fn logout(&self) {
        if let Ok(mut session) = self.session.lock() {
            *session = None;
        }
    }

    pub fn require_authenticated(&self) -> Result<Session, String> {
        self.session
            .lock()
            .map_err(|_| "No se pudo leer la sesión activa".to_string())?
            .clone()
            .ok_or_else(|| "Sesión no autenticada".to_string())
    }

    pub fn require_admin(&self) -> Result<Session, String> {
        let session = self.require_authenticated()?;
        if session.role == Role::Admin {
            Ok(session)
        } else {
            Err("Se requiere una sesión de administrador".to_string())
        }
    }

    pub fn require_operator(&self) -> Result<Session, String> {
        let session = self.require_authenticated()?;
        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthState, Role};

    #[test]
    fn una_sesion_vacia_no_puede_acceder() {
        let auth = AuthState::default();
        assert!(auth.require_authenticated().is_err());
        assert!(auth.require_admin().is_err());
    }

    #[test]
    fn empleado_no_puede_usar_comandos_de_admin() {
        let auth = AuthState::default();
        auth.login(7, Role::Employee, "Operador".to_string());
        assert!(auth.require_operator().is_ok());
        assert!(auth.require_admin().is_err());
    }

    #[test]
    fn admin_puede_usar_comandos_de_admin_y_logout_revoca() {
        let auth = AuthState::default();
        auth.login(1, Role::Admin, "Admin".to_string());
        assert_eq!(auth.require_admin().unwrap().user_id, 1);
        auth.logout();
        assert!(auth.require_admin().is_err());
    }
}
