//! Verificación end-to-end del chat LOCAL: llama EXACTAMENTE a las mismas
//! funciones que usa el backend (`src_ia::motor_chat::llm::chat_1_7` /
//! `chat_1_7_raw`) para validar que el 1.7B conteste con el system prompt
//! de TESTING.
//! Ejecutar con: cargo run --example probar_chat_local --features llm-local

use src_ia::motor_chat::cloud::prompts::Mensaje;
use src_ia::motor_chat::llm::{SYSTEM_PROMPT_TEST, chat_1_7, chat_1_7_raw};
use src_ia::rutas::rutas_modelos::verificar_modelos;

fn main() {
    println!("=== Paso 1/2: Modelo 1.7B en disco ===");
    let mut encontrado_17 = false;
    for (key, info) in verificar_modelos() {
        if key == "1.7B" {
            encontrado_17 = info.existe;
            let status = if info.existe { "OK" } else { "NO" };
            println!("  [{status}] Qwen {key}: {}MB -> {}", info.tamano_mb, info.ruta.display());
        }
    }
    assert!(encontrado_17, "Falta el modelo Qwen 1.7B en ~/.lmstudio/models");

    let historial = vec![
        Mensaje::new("user", "Hola Y.A.R.V.I.S., solo prueba rapida: que estas haciendo?"),
    ];

    println!("\n=== Paso 2/2: chat_1_7 (inferencia real) ===");
    println!("System prompt de testing: {SYSTEM_PROMPT_TEST:?}");
    match chat_1_7(&historial) {
        Ok(respuesta) => {
            println!("\nRespuesta (limpia):\n{respuesta}");
            assert!(!respuesta.trim().is_empty(), "Respuesta vacía");
        }
        Err(e) => {
            eprintln!("chat_1_7 falló: {e}");
            std::process::exit(1);
        }
    }

    println!("{}", "-".repeat(60));
    println!("=== Extra: chat_1_7_raw (conserva bloques think, lo que usa el streaming) ===");
    match chat_1_7_raw(&historial) {
        Ok(raw) => {
            println!("\nRespuesta cruda:\n{raw}");
            assert!(!raw.trim().is_empty(), "Respuesta cruda vacía");
        }
        Err(e) => {
            eprintln!("chat_1_7_raw falló: {e}");
            std::process::exit(1);
        }
    }

    println!("\n=== CHAT LOCAL VERIFICADO: el 1.7B contesta desde Rust ===");
}