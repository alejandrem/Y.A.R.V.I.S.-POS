// ============================================================
// respaldos::cifrado — Contraseña + Argon2id + ChaCha20-Poly1305.
//
// El respaldo real sale del disco en un archivo cifrado. La contraseña
// la elige el dueño y NO se guarda en el archivo: sin ella no hay forma
// de leer el respaldo, ni siquiera con la app. Para los respaldos
// automáticos la app guarda una copia de la contraseña protegida con
// DPAPI (Windows) en la config; el dueño sigue necesitándola para
// restaurar en otra máquina.
//
// Formato .yarvisbak v1:
//   "YRVBAK01" | salt(16) | nonce(12) | ciphertext+tag(16)
// El AAD es la magia: cambiar el encabezado invalida el tag.
// ============================================================

use std::fs;
use std::io::Write;
use std::path::Path;

use argon2::{Algorithm, Argon2, Params, Version};
use ring::aead::{self, Aad, LessSafeKey, Nonce, UnboundKey};
use ring::rand::{SecureRandom, SystemRandom};

pub const MIN_PASSPHRASE: usize = 8;
const MAGIA: &[u8; 8] = b"YRVBAK01";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;
const CABECERA: usize = MAGIA.len() + SALT_LEN + NONCE_LEN;
const LARGO_LLAVE: usize = 32;

/// Cifra bytes en memoria. Devuelve el archivo completo listo para disco.
pub fn cifrar(plano: &[u8], passphrase: &str) -> Result<Vec<u8>, String> {
    if passphrase.trim().len() < MIN_PASSPHRASE {
        return Err(format!(
            "La contraseña del respaldo debe tener al menos {MIN_PASSPHRASE} caracteres"
        ));
    }

    let rng = SystemRandom::new();
    let mut salt = [0u8; SALT_LEN];
    rng.fill(&mut salt)
        .map_err(|_| "No se pudo generar la sal del respaldo".to_string())?;
    let mut nonce = [0u8; NONCE_LEN];
    rng.fill(&mut nonce)
        .map_err(|_| "No se pudo generar el nonce del respaldo".to_string())?;

    let mut llave = derivar(passphrase, &salt)?;
    let resultado = (|| -> Result<Vec<u8>, String> {
        let unbound = UnboundKey::new(&aead::CHACHA20_POLY1305, &llave)
            .map_err(|_| "No se pudo inicializar el cifrado del respaldo".to_string())?;
        let llave_aead = LessSafeKey::new(unbound);

        let mut contenido = plano.to_vec();
        llave_aead
            .seal_in_place_append_tag(
                Nonce::assume_unique_for_key(nonce),
                Aad::from(MAGIA),
                &mut contenido,
            )
            .map_err(|_| "No se pudo cifrar el respaldo".to_string())?;

        let mut salida = Vec::with_capacity(CABECERA + contenido.len());
        salida.extend_from_slice(MAGIA);
        salida.extend_from_slice(&salt);
        salida.extend_from_slice(&nonce);
        salida.extend_from_slice(&contenido);
        Ok(salida)
    })();
    llave.fill(0);
    resultado
}

/// Descifra un archivo .yarvisbak. Cualquier alteración (contraseña
/// incorrecta, bytes cambiados, archivo truncado) devuelve error.
pub fn descifrar(blob: &[u8], passphrase: &str) -> Result<Vec<u8>, String> {
    if passphrase.is_empty() {
        return Err("Escribe la contraseña del respaldo".to_string());
    }
    if blob.len() < CABECERA + TAG_LEN {
        return Err("El archivo de respaldo está truncado o no es un .yarvisbak".to_string());
    }
    if &blob[..MAGIA.len()] != MAGIA {
        return Err("El archivo no tiene el formato de respaldo de YARVIS".to_string());
    }

    let salt = &blob[MAGIA.len()..MAGIA.len() + SALT_LEN];
    let nonce: [u8; NONCE_LEN] = blob[MAGIA.len() + SALT_LEN..CABECERA]
        .try_into()
        .map_err(|_| "Archivo de respaldo corrupto".to_string())?;
    let mut contenido = blob[CABECERA..].to_vec();

    let mut llave = derivar(passphrase, salt)?;
    let resultado = (|| -> Result<Vec<u8>, String> {
        let unbound = UnboundKey::new(&aead::CHACHA20_POLY1305, &llave)
            .map_err(|_| "No se pudo inicializar el descifrado".to_string())?;
        let llave_aead = LessSafeKey::new(unbound);
        let claro = llave_aead
            .open_in_place(
                Nonce::assume_unique_for_key(nonce),
                Aad::from(MAGIA),
                &mut contenido,
            )
            .map_err(|_| {
                "No se pudo descifrar: contraseña incorrecta o archivo dañado".to_string()
            })?;
        Ok(claro.to_vec())
    })();
    llave.fill(0);
    resultado
}

pub fn cifrar_archivo(origen: &Path, destino: &Path, passphrase: &str) -> Result<(), String> {
    let plano = fs::read(origen)
        .map_err(|e| format!("No se pudo leer {}: {e}", origen.display()))?;
    let cifrado = cifrar(&plano, passphrase)?;
    escribir_atomico(destino, &cifrado)
}

pub fn descifrar_archivo(origen: &Path, destino: &Path, passphrase: &str) -> Result<(), String> {
    let blob = fs::read(origen)
        .map_err(|e| format!("No se pudo leer {}: {e}", origen.display()))?;
    let plano = descifrar(&blob, passphrase)?;
    escribir_atomico(destino, &plano)
}

fn derivar(passphrase: &str, salt: &[u8]) -> Result<[u8; LARGO_LLAVE], String> {
    // Argon2id, 19 MiB, 2 pasadas, 1 hilo: ~0.1-0.5 s por respaldo,
    // suficiente para que un atacante con el archivo no lo abra de golpe.
    let params = Params::new(19_456, 2, 1, Some(LARGO_LLAVE))
        .map_err(|e| format!("Parámetros de cifrado inválidos: {e}"))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut llave = [0u8; LARGO_LLAVE];
    argon
        .hash_password_into(passphrase.as_bytes(), salt, &mut llave)
        .map_err(|e| format!("No se pudo derivar la llave del respaldo: {e}"))?;
    Ok(llave)
}

fn escribir_atomico(ruta: &Path, bytes: &[u8]) -> Result<(), String> {
    let dir = ruta
        .parent()
        .ok_or_else(|| "Ruta de respaldo sin carpeta padre".to_string())?;
    fs::create_dir_all(dir).map_err(|e| format!("No se pudo crear {}: {e}", dir.display()))?;
    let temporal = dir.join(format!(
        ".{}.tmp-{}-{}",
        ruta.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "respaldo".to_string()),
        std::process::id(),
        chrono::Local::now().format("%Y%m%d%H%M%S%f")
    ));

    let resultado = (|| -> Result<(), String> {
        let mut archivo = fs::File::create(&temporal)
            .map_err(|e| format!("No se pudo crear {}: {e}", temporal.display()))?;
        archivo
            .write_all(bytes)
            .map_err(|e| format!("No se pudo escribir {}: {e}", temporal.display()))?;
        archivo
            .sync_all()
            .map_err(|e| format!("No se pudo sincronizar {}: {e}", temporal.display()))?;
        drop(archivo);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&temporal, fs::Permissions::from_mode(0o600))
                .map_err(|e| format!("No se pudieron aplicar permisos 0600: {e}"))?;
        }

        fs::rename(&temporal, ruta)
            .map_err(|e| format!("No se pudo instalar {}: {e}", ruta.display()))
    })();

    if resultado.is_err() {
        let _ = fs::remove_file(&temporal);
    }
    resultado
}

#[cfg(test)]
mod tests;
