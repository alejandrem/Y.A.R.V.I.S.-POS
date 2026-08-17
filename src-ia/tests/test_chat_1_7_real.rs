//! Test de integración REAL: carga el Qwen 1.7B y le pide una respuesta.
//! Solo funciona en máquinas con el GGUF descargado.
//!
//! Ejecutar:  cargo test --features llm-local -p src-ia --test test_chat_1_7_real -- --nocapture

#[cfg(feature = "llm-local")]
#[test]
fn chat_1_7_responde_algo() {
    use src_ia::motor_chat::cloud::prompts::Mensaje;
    use src_ia::motor_chat::llm::chat_1_7;

    let mensajes = vec![Mensaje::new("user", "Hola, ¿quién eres?")];

    println!("[TEST] Enviando mensaje al Qwen 1.7B local...");
    let inicio = std::time::Instant::now();
    let resultado = chat_1_7(&mensajes);
    let duracion = inicio.elapsed();

    match resultado {
        Ok(respuesta) => {
            println!("[TEST] ✅ Respuesta en {:.1}s:", duracion.as_secs_f64());
            println!("─────────────────────────────────────");
            println!("{respuesta}");
            println!("─────────────────────────────────────");
            assert!(!respuesta.is_empty(), "La respuesta está vacía");
            // El modelo debería identificarse como YARVIS o al menos responder algo
            println!("[TEST] Longitud de respuesta: {} chars", respuesta.len());
        }
        Err(e) => {
            panic!("[TEST] ❌ Error del modelo: {e}");
        }
    }
}
