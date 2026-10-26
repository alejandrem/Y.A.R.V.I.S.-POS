// ============================================================
// respaldos::verificar — Prueba REAL de un respaldo antes de confiar.
//
// Un respaldo que no se puede descifrar o cuya DB está dañada es peor
// que no tener respaldo: da falsa seguridad. Aquí se descifra a un
// temporal, se ejecuta PRAGMA integrity_check, se comprueban las tablas
// esenciales y se cuentan filas clave. El temporal se borra al salir.
// ============================================================

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::backventanas::db::respaldo::{probar_db, ResumenDb};
use crate::respaldos::cifrado;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PruebaRespaldo {
    pub ok: bool,
    pub mensaje: String,
    pub tablas: i64,
    pub usuarios: i64,
    pub ventas: i64,
    pub productos: i64,
    pub detalle_ventas: i64,
    pub version_esquema: i64,
}

/// Archivo .db descifrado en un temporal; se borra automáticamente.
#[derive(Debug)]
pub struct DbTemporal {
    ruta: PathBuf,
}

impl DbTemporal {
    pub fn ruta(&self) -> &Path {
        &self.ruta
    }
}

impl Drop for DbTemporal {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.ruta);
    }
}

pub fn probar(
    archivo: &Path,
    passphrase: &str,
    tmp_dir: &Path,
) -> Result<PruebaRespaldo, String> {
    let temp = descifrar_a_temp(archivo, passphrase, tmp_dir)?;
    probar_plana(temp.ruta())
}

/// Descifra a un .db temporal. No valida todavía: el llamador debe usar
/// `probar` o `probar_plana` antes de restaurar.
pub fn descifrar_a_temp(
    archivo: &Path,
    passphrase: &str,
    tmp_dir: &Path,
) -> Result<DbTemporal, String> {
    fs::create_dir_all(tmp_dir)
        .map_err(|e| format!("No se pudo crear la carpeta temporal: {e}"))?;
    let blob = fs::read(archivo)
        .map_err(|e| format!("No se pudo leer {}: {e}", archivo.display()))?;
    let plano = cifrado::descifrar(&blob, passphrase)?;

    let sello = chrono::Local::now().format("%Y%m%d%H%M%S%f").to_string();
    let ruta = tmp_dir.join(format!(".yarvis-restaurar-{}-{sello}.db", std::process::id()));
    fs::write(&ruta, plano).map_err(|e| format!("No se pudo escribir el temporal: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&ruta, fs::Permissions::from_mode(0o600));
    }

    Ok(DbTemporal { ruta })
}

/// Prueba un .db en claro (respaldo local de arranque).
pub fn probar_plana(ruta: &Path) -> Result<PruebaRespaldo, String> {
    let resumen = probar_db(ruta)?;
    Ok(desde_resumen(resumen))
}

fn desde_resumen(resumen: ResumenDb) -> PruebaRespaldo {
    PruebaRespaldo {
        ok: true,
        mensaje: format!(
            "Respaldo válido: {} ventas, {} productos y {} usuarios.",
            resumen.ventas, resumen.productos, resumen.usuarios
        ),
        tablas: resumen.tablas,
        usuarios: resumen.usuarios,
        ventas: resumen.ventas,
        productos: resumen.productos,
        detalle_ventas: resumen.detalle_ventas,
        version_esquema: resumen.version_esquema,
    }
}

#[cfg(test)]
mod tests;
