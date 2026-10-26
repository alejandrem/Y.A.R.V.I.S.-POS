// ============================================================
// respaldos — Respaldo REAL del POS: programado, cifrado y fuera
// del disco de la base de datos.
//
//   cifrado    Argon2id + ChaCha20-Poly1305 (.yarvisbak)
//   config     destino, intervalo, retención y contraseña protegida
//   archivo    foto consistente (VACUUM INTO) + cifrado + rotación
//   verificar  prueba real (integrity_check + tablas esenciales)
//   programador job de fondo cada 30 min (nunca bloquea una venta)
//   comandos   API Tauri del wizard de restauración
// ============================================================

pub mod archivo;
pub mod cifrado;
pub mod comandos;
pub mod config;
pub mod programador;
pub mod verificar;

/// Serializa la creación de respaldos: el job automático y el botón
/// "Crear ahora" no deben escribir el mismo destino al mismo tiempo.
pub static CANDADO: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
