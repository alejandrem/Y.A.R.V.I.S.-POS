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
    /// Puerto principal del motor de IA (embeddings + Prophet + chat)
    pub port: Mutex<Option<u16>>,
    /// Puerto reservado para el LLM futuro (Qwen)
    pub port_llm: Mutex<Option<u16>>,
    pub process: Mutex<Option<Child>>,
    pub status: Mutex<AiStatus>,
}

impl AiSidecar {
    pub fn new() -> Self {
        AiSidecar {
            port: Mutex::new(None),
            port_llm: Mutex::new(None),
            process: Mutex::new(None),
            status: Mutex::new(AiStatus::NotRunning),
        }
    }

    pub fn get_status(&self) -> AiStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn get_port(&self) -> Option<u16> {
        *self.port.lock().unwrap()
    }

    pub fn get_port_llm(&self) -> Option<u16> {
        *self.port_llm.lock().unwrap()
    }

    /// Retorna la URL base del motor de IA (ej: http://127.0.0.1:54321)
    pub fn base_url(&self) -> Option<String> {
        self.get_port().map(|p| format!("http://127.0.0.1:{}", p))
    }
}

// ============================================================
// 1. BUSCAR PUERTOS TCP LIBRES
// Retorna dos puertos: uno para el motor principal, otro reservado para LLM.
// ============================================================

pub fn find_two_free_ports() -> Result<(u16, u16), String> {
    let port1 = find_free_port()?;

    // Buscar un segundo puerto que no sea adyacente al primero
    let mut port2 = find_free_port()?;
    while port2 == port1 || port2.abs_diff(port1) <= 1 {
        port2 = find_free_port()?;
    }

    Ok((port1, port2))
}

pub fn find_free_port() -> Result<u16, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("No se pudo encontrar un puerto libre: {}", e))?;
    let port = listener.local_addr()
        .map_err(|e| format!("Error al leer el puerto: {}", e))?
        .port();
    Ok(port)
}

// ============================================================
// 2. RESOLVER LA RUTA AL main.py
// ============================================================

pub fn get_python_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent().unwrap_or(std::path::Path::new("."));
        let prod_path = exe_dir.join("engine").join("main.py");
        if prod_path.exists() {
            return prod_path;
        }
    }

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

    PathBuf::from("engine/main.py")
}

/// Busca el ejecutable de Python con el venv activado
pub fn get_python_executable() -> String {
    // En desarrollo: buscar venv en yarvis-IA/.venv/bin/python3
    let venv_candidates = [
        "../../yarvis-IA/.venv/bin/python3",
        "../../../yarvis-IA/.venv/bin/python3",
        "yarvis-IA/.venv/bin/python3",
    ];

    for candidate in &venv_candidates {
        let path = std::path::Path::new(candidate);
        if path.exists() {
            return path.to_string_lossy().to_string();
        }
    }

    // Fallback al sistema
    let sys_cmds = ["python3", "python"];
    for &cmd in &sys_cmds {
        if Command::new(cmd).arg("--version").output().is_ok() {
            return cmd.to_string();
        }
    }

    "python3".to_string()
}

// ============================================================
// 3. LANZAR EL PROCESO PYTHON
// ============================================================

pub fn start_python(port: u16) -> Result<Child, String> {
    let script_path = get_python_path();
    let python_exe = get_python_executable();

    println!(
        "[YARVIS-SIDECAR] Lanzando Python en puerto {} desde: {:?} (exe: {})",
        port, script_path, python_exe
    );

    let result = Command::new(&python_exe)
        .arg(&script_path)
        .arg(port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    match result {
        Ok(child) => {
            println!(
                "[YARVIS-SIDECAR] Python lanzado con PID {}",
                child.id()
            );
            Ok(child)
        }
        Err(e) => Err(format!(
            "No se pudo ejecutar Python ({}) para el script {:?}: {}",
            python_exe, script_path, e
        )),
    }
}

// ============================================================
// 4. HEALTH CHECK ASÍNCRONO
// ============================================================

pub async fn wait_health_check(port: u16, timeout_secs: u64) -> bool {
    let url = format!("http://127.0.0.1:{}/", port);
    let client = reqwest::Client::new();
    let total_attempts = timeout_secs * 2;

    println!(
        "[YARVIS-SIDECAR] Esperando health check en {} (timeout: {}s)...",
        url, timeout_secs
    );

    for attempt in 0..total_attempts {
        sleep(Duration::from_millis(500)).await;

        match client.get(&url).timeout(Duration::from_secs(2)).send().await {
            Ok(resp) if resp.status().is_success() => {
                println!(
                    "[YARVIS-SIDECAR] Python listo después de ~{}s",
                    (attempt as f32 * 0.5).ceil() as u64
                );
                return true;
            }
            _ => {
                if attempt % 4 == 0 {
                    println!(
                        "[YARVIS-SIDECAR] Esperando... ({:.0}s / {}s)",
                        attempt as f32 * 0.5,
                        timeout_secs
                    );
                }
            }
        }
    }

    println!("[YARVIS-SIDECAR] Timeout: Python no respondió en {}s", timeout_secs);
    false
}

// ============================================================
// 5. ORQUESTADORA DEL BOOT
// ============================================================

pub async fn launch_ai_engine(sidecar: std::sync::Arc<AiSidecar>) {
    *sidecar.status.lock().unwrap() = AiStatus::Starting;

    // 1. Buscar 2 puertos libres
    let (port, port_llm) = match find_two_free_ports() {
        Ok(p) => p,
        Err(e) => {
            println!("[YARVIS-SIDECAR] Error buscando puertos: {}", e);
            *sidecar.status.lock().unwrap() = AiStatus::Error(e);
            return;
        }
    };
    println!("[YARVIS-SIDECAR] Puertos seleccionados: IA={}, LLM={}", port, port_llm);
    *sidecar.port.lock().unwrap() = Some(port);
    *sidecar.port_llm.lock().unwrap() = Some(port_llm);

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

    // 3. Esperar health check
    let is_ready = wait_health_check(port, 30).await;

    if is_ready {
        *sidecar.status.lock().unwrap() = AiStatus::Ready;
        println!("[YARVIS-SIDECAR] Motor de IA listo en puerto {}", port);
    } else {
        *sidecar.status.lock().unwrap() =
            AiStatus::Error("Python no respondió en 30s. La app funciona sin IA.".to_string());
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
            println!("[YARVIS-SIDECAR] Apagando motor de IA (PID: {})...", proc.id());
            let _ = proc.kill();
            let _ = proc.wait();
            println!("[YARVIS-SIDECAR] Motor de IA apagado correctamente.");
        }
    }
}
