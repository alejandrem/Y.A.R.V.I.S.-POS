// ============================================================
// sidecar.rs — Motor de IA: arranque, health check y ciclo de vida
// Responsabilidad: Lanzar yarvis-IA/main.py como proceso hijo,
// esperar que esté listo y mantenerlo vivo durante la app.
// ============================================================

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::Duration;
use tokio::time::sleep;

// ============================================================
// ESTADO PÚBLICO DEL SIDECAR
// ============================================================

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub enum AiStatus {
    NotRunning,
    Starting,
    Ready,
    Error(String),
}

pub struct AiSidecar {
    pub port: Mutex<Option<u16>>,
    pub process: Mutex<Option<Child>>,
    pub status: Mutex<AiStatus>,
}

impl AiSidecar {
    pub fn new() -> Self {
        AiSidecar {
            port: Mutex::new(None),
            process: Mutex::new(None),
            status: Mutex::new(AiStatus::NotRunning),
        }
    }

    /// Lee el estado actual de manera segura
    pub fn get_status(&self) -> AiStatus {
        self.status.lock().unwrap().clone()
    }

    /// Lee el puerto actual de manera segura
    pub fn get_port(&self) -> Option<u16> {
        *self.port.lock().unwrap()
    }
}

// ============================================================
// 1. BUSCAR PUERTO TCP LIBRE
// Deja que el SO asigne un puerto libre al bindear al 0,
// luego lo libera para que Python lo use.
// ============================================================

pub fn find_free_port() -> Result<u16, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("No se pudo encontrar un puerto libre: {}", e))?;
    let port = listener.local_addr()
        .map_err(|e| format!("Error al leer el puerto: {}", e))?
        .port();
    // El listener se dropea aquí → libera el puerto para Python
    Ok(port)
}

// ============================================================
// 2. RESOLVER LA RUTA AL main.py
// - En dev (npm run tauri dev): busca relativo al workspace.
// - En producción: busca relativo al ejecutable compilado.
// ============================================================

pub fn get_python_path() -> PathBuf {
    // En producción: <dir del exe>/engine/main.py
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent().unwrap_or(std::path::Path::new("."));
        let prod_path = exe_dir.join("engine").join("main.py");
        if prod_path.exists() {
            return prod_path;
        }
    }

    // En desarrollo: subir desde src-tauri/target/... hasta la raíz del workspace
    // Intentamos varias rutas relativas al cwd por si acaso
    let dev_candidates = [
        PathBuf::from("../../yarvis-IA/main.py"),
        PathBuf::from("../../../yarvis-IA/main.py"),
        PathBuf::from("yarvis-IA/main.py"),
    ];

    for candidate in &dev_candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    // Fallback: retorna la ruta de producción aunque no exista,
    // el error se capturará en start_python()
    PathBuf::from("engine/main.py")
}

// ============================================================
// 3. LANZAR EL PROCESO PYTHON
// ============================================================

pub fn start_python(port: u16) -> Result<Child, String> {
    let script_path = get_python_path();

    println!(
        "[YARVIS-SIDECAR] Intentando lanzar Python en puerto {} desde: {:?}",
        port, script_path
    );

    // En Windows intentamos primero "python", luego "python3" como fallback
    let python_cmds = ["python", "python3"];

    for &cmd in &python_cmds {
        let result = Command::new(cmd)
            .arg(&script_path)
            .arg(port.to_string())
            // Separar stdout/stderr del proceso padre para no bloquear
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(child) => {
                println!(
                    "[YARVIS-SIDECAR] Python lanzado con PID {} usando '{}'",
                    child.id(),
                    cmd
                );
                return Ok(child);
            }
            Err(_) => continue, // Prueba el siguiente comando
        }
    }

    Err(format!(
        "No se pudo ejecutar Python. Asegúrate de que 'python' o 'python3' estén en PATH. Script: {:?}",
        script_path
    ))
}

// ============================================================
// 4. HEALTH CHECK ASÍNCRONO
// Hace GET /health cada 500ms hasta que responda 200
// o se acabe el timeout.
// ============================================================

pub async fn wait_health_check(port: u16, timeout_secs: u64) -> bool {
    let url = format!("http://127.0.0.1:{}/", port);
    let client = reqwest::Client::new();
    let total_attempts = timeout_secs * 2; // 1 intento cada 500ms

    println!(
        "[YARVIS-SIDECAR] Esperando health check en {} (timeout: {}s)...",
        url, timeout_secs
    );

    for attempt in 0..total_attempts {
        sleep(Duration::from_millis(500)).await;

        match client.get(&url).timeout(Duration::from_secs(2)).send().await {
            Ok(resp) if resp.status().is_success() => {
                println!(
                    "[YARVIS-SIDECAR] ✅ Python listo después de ~{}s",
                    (attempt as f32 * 0.5).ceil() as u64
                );
                return true;
            }
            _ => {
                // Python todavía arrancando, seguir esperando
                if attempt % 4 == 0 {
                    // Log cada 2 segundos
                    println!(
                        "[YARVIS-SIDECAR] Esperando... ({:.0}s / {}s)",
                        attempt as f32 * 0.5,
                        timeout_secs
                    );
                }
            }
        }
    }

    println!("[YARVIS-SIDECAR] ❌ Timeout: Python no respondió en {}s", timeout_secs);
    false
}

// ============================================================
// 5. FUNCIÓN PRINCIPAL: ORQUESTADORA DEL BOOT
// Llamada desde lib.rs en una tarea async de Tokio.
// Gestiona todo el ciclo: puerto → lanzar → esperar → estado.
// ============================================================

pub async fn launch_ai_engine(sidecar: std::sync::Arc<AiSidecar>) {
    // Marcar como "arrancando"
    *sidecar.status.lock().unwrap() = AiStatus::Starting;

    // 1. Buscar puerto libre
    let port = match find_free_port() {
        Ok(p) => p,
        Err(e) => {
            println!("[YARVIS-SIDECAR] Error buscando puerto: {}", e);
            *sidecar.status.lock().unwrap() = AiStatus::Error(e);
            return;
        }
    };
    println!("[YARVIS-SIDECAR] Puerto seleccionado: {}", port);
    *sidecar.port.lock().unwrap() = Some(port);

    // 2. Lanzar Python
    let child = match start_python(port) {
        Ok(c) => c,
        Err(e) => {
            println!("[YARVIS-SIDECAR] Error lanzando Python: {}", e);
            *sidecar.status.lock().unwrap() = AiStatus::Error(e);
            return;
        }
    };
    *sidecar.process.lock().unwrap() = Some(child);

    // 3. Esperar health check (30 segundos de gracia)
    let is_ready = wait_health_check(port, 30).await;

    if is_ready {
        *sidecar.status.lock().unwrap() = AiStatus::Ready;
        println!("[YARVIS-SIDECAR] 🚀 Motor de IA listo en puerto {}", port);
    } else {
        *sidecar.status.lock().unwrap() =
            AiStatus::Error("Python no respondió en 30s. La app funciona sin IA.".to_string());
        // Limpiar el proceso si no arrancó bien
        if let Some(mut proc) = sidecar.process.lock().unwrap().take() {
            let _ = proc.kill();
        }
    }
}

// ============================================================
// 6. CLEANUP: Matar Python al cerrar la app
// ============================================================

pub fn shutdown_ai_engine(sidecar: &AiSidecar) {
    if let Ok(mut guard) = sidecar.process.lock() {
        if let Some(mut proc) = guard.take() {
            println!("[YARVIS-SIDECAR] 🛑 Apagando motor de IA (PID: {})...", proc.id());
            let _ = proc.kill();
            let _ = proc.wait(); // Evita procesos zombie
            println!("[YARVIS-SIDECAR] Motor de IA apagado correctamente.");
        }
    }
}
