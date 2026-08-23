use std::sync::Mutex;

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

/// Estado de autenticación en memoria.
///
/// Tauri conserva este estado en el proceso nativo; el frontend no puede
/// cambiar el rol enviando un parámetro arbitrario a cada comando.
pub struct AuthState {
    session: Mutex<Option<Session>>,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            session: Mutex::new(None),
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
