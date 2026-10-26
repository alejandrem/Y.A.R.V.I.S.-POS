// ============================================================
// db::error — Errores de arranque de la base de datos SIN panic.
//
// Antes, un fallo de conexión o una migración rota ejecutaba panic!
// dentro del hilo de arranque. El proceso seguía vivo, el hilo moría
// antes de registrar el pool y la ventana principal se mostraba vacía:
// "la caja abrió pero nada funciona". Este tipo lleva el detalle, una
// pista accionable y si el fallo es un VersionMismatch (migración
// aplicada con otro checksum).
// ============================================================

use std::fmt;
use std::path::Path;

/// Error legible para el dueño, con detalle técnico para el log.
#[derive(Debug, Clone)]
pub struct ErrorDb {
    /// Una línea para el título del diálogo.
    pub resumen: String,
    /// Texto completo: causa, versión de migración, ruta, etc.
    pub detalle: String,
    /// Qué puede hacer el dueño (restaurar, revisar log, no editar .sql).
    pub pista: String,
    /// true solo para VersionMismatch: el .sql embebido no coincide.
    pub version_mismatch: bool,
}

impl ErrorDb {
    pub fn directorio(detalle: impl Into<String>) -> Self {
        Self {
            resumen: "No se pudo preparar la carpeta de datos".to_string(),
            detalle: detalle.into(),
            pista: "Revisa permisos de la carpeta de datos y vuelve a intentar.".to_string(),
            version_mismatch: false,
        }
    }

    pub fn conexion(detalle: impl Into<String>) -> Self {
        Self {
            resumen: "No se pudo abrir la base de datos".to_string(),
            detalle: detalle.into(),
            pista: "El archivo yarvis.db puede estar dañado o bloqueado. Restaura un respaldo."
                .to_string(),
            version_mismatch: false,
        }
    }

    pub fn migracion(error: sqlx::migrate::MigrateError, db_path: &Path) -> Self {
        let version_mismatch = match &error {
            sqlx::migrate::MigrateError::VersionMismatch(v) => {
                tracing::error!(
                    "[DB] VersionMismatch en migración v{v}: el .sql embebido no coincide con el aplicado"
                );
                true
            }
            _ => false,
        };

        let pista = if version_mismatch {
            "No edites migraciones ya aplicadas ni permitas que un checkout cambie LF->CRLF. \
             Restaura un respaldo anterior o usa el ejecutable original que aplicó esa migración."
        } else {
            "Restaura el respaldo local más reciente o revisa el log para ver la migración exacta."
        };

        Self {
            resumen: if version_mismatch {
                "La base de datos pertenece a otra versión del programa".to_string()
            } else {
                "No se pudo actualizar la base de datos".to_string()
            },
            detalle: format!("{error}\nBase de datos: {}", db_path.display()),
            pista: pista.to_string(),
            version_mismatch,
        }
    }

    pub fn operacion(detalle: impl Into<String>) -> Self {
        Self {
            resumen: "La base de datos no respondió".to_string(),
            detalle: detalle.into(),
            pista: "Cierra la app, revisa el log y vuelve a intentar.".to_string(),
            version_mismatch: false,
        }
    }

    /// Mensaje completo para el diálogo nativo (sin markdown).
    pub fn mensaje(&self) -> String {
        format!("{}\n\n{}\n\nQué hacer: {}", self.resumen, self.detalle, self.pista)
    }
}

impl fmt::Display for ErrorDb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.resumen, self.detalle)
    }
}

impl std::error::Error for ErrorDb {}
