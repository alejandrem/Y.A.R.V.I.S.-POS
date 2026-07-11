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
    pub process: Mutex<Option<Child>>,
    pub status: Mutex<AiStatus>,
    /// Cliente HTTP compartido (reutiliza pool de conexiones)
    pub http_client: reqwest::Client,
}

impl AiSidecar {
    pub fn new() -> Self {
        AiSidecar {
            port: Mutex::new(None),
            process: Mutex::new(None),
            status: Mutex::new(AiStatus::NotRunning),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(900))
                .pool_max_idle_per_host(4)
                .build()
                .expect("Error creando HTTP client compartido"),
        }
    }
    
    pub fn get_status(&self) -> AiStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn get_port(&self) -> Option<u16> {
        *self.port.lock().unwrap()
    }

    /// Retorna la URL base del motor de IA (ej: http://127.0.0.1:54321)
    pub fn base_url(&self) -> Option<String> {
        self.get_port().map(|p| format!("http://127.0.0.1:{}", p))
    }

    /// Verifica si el proceso Python sigue vivo. Si murió, limpia el estado.
    pub fn check_process_alive(&self) {
        if let Ok(mut guard) = self.process.lock() {
            if let Some(ref mut proc) = *guard {
                match proc.try_wait() {
                    Ok(Some(_status)) => {
                        // Proceso terminó
                        println!("[YARVIS-SIDECAR] Proceso Python terminó, limpiando estado...");
                        *guard = None;
                        *self.port.lock().unwrap() = None;
                        *self.status.lock().unwrap() = AiStatus::Error("Proceso Python terminó".to_string());
                    }
                    Ok(None) => { /* Proceso sigue vivo */ }
                    Err(e) => {
                        println!("[YARVIS-SIDECAR] Error verificando proceso: {}", e);
                    }
                }
            }
        }
    }
}

// ============================================================
// 1. BUSCAR PUERTO TCP LIBRE
// ============================================================

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

    // Buscar libs CUDA de LM Studio para LD_LIBRARY_PATH
    let cuda_lib_paths = [
        // LM Studio CUDA 12 vendor libs
        "/home/alesito/.lmstudio/extensions/backends/vendor/linux-llama-cuda12-vendor-v1",
        "../../.lmstudio/extensions/backends/vendor/linux-llama-cuda12-vendor-v1",
        "../../../.lmstudio/extensions/backends/vendor/linux-llama-cuda12-vendor-v1",
        // nvidia pip package paths (cu12)
        "../../yarvis-IA/.venv/lib/python3.14/site-packages/nvidia/cublas/lib",
        "../../../yarvis-IA/.venv/lib/python3.14/site-packages/nvidia/cublas/lib",
        "../../yarvis-IA/.venv/lib/python3.14/site-packages/nvidia/cuda_runtime/lib",
        "../../../yarvis-IA/.venv/lib/python3.14/site-packages/nvidia/cuda_runtime/lib",
    ];

    let mut extra_libs = Vec::new();
    for candidate in &cuda_lib_paths {
        let path = std::path::Path::new(candidate);
        if path.exists() {
            extra_libs.push(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
        }
    }

    let mut cmd = Command::new(&python_exe);
    cmd.arg(&script_path)
        .arg(port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    // Agregar rutas CUDA a LD_LIBRARY_PATH
    if !extra_libs.is_empty() {
        let existing = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
        let new_paths: Vec<String> = extra_libs.iter().map(|p| p.to_string_lossy().to_string()).collect();
        let mut all_paths = new_paths;
        if !existing.is_empty() {
            all_paths.push(existing);
        }
        let ld_path = all_paths.join(":");
        println!("[YARVIS-SIDECAR] LD_LIBRARY_PATH (CUDA): {}", ld_path);
        cmd.env("LD_LIBRARY_PATH", ld_path);
    }

    let result = cmd.spawn();

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
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("Error creando client para health check");
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

    // 1. Buscar un puerto libre
    let port = match find_free_port() {
        Ok(p) => p,
        Err(e) => {
            println!("[YARVIS-SIDECAR] Error buscando puerto: {}", e);
            *sidecar.status.lock().unwrap() = AiStatus::Error(e);
            return;
        }
    };
    println!("[YARVIS-SIDECAR] Puerto seleccionado: IA={}", port);
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
