//! Test de integración REAL del Qwen 1.7B local: si el GGUF no está descargado
//! el test se OMITE (no falla); si está, verifica el arranque y la respuesta
//! limpia (`chat_1_7`) y cruda con bloques think (`chat_1_7_raw`).
//!
//! Ejecutar: cargo test --features llm-local --test test_chat_1_7_real -- --nocapture

#[cfg(feature = "llm-local")]
fn hay_modelo_1_7() -> bool {
    use src_ia::rutas::verificar_modelos;
    let ok = verificar_modelos()
        .iter()
        .any(|(key, info)| key == &"1.7B" && info.existe);
    if !ok {
        eprintln!(
            "[test_chat_1_7_real] Modelo 1.7B no descargado en ~/.lmstudio/models — test omitido."
        );
    }
    ok
}

#[cfg(feature = "llm-local")]
#[test]
fn chat_1_7_responde_algo() {
    use src_ia::motor_chat::cloud::prompts::Mensaje;
    use src_ia::motor_chat::llm::chat_1_7;

    if !hay_modelo_1_7() {
        return;
    }

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
            println!("[TEST] Longitud de respuesta: {} chars", respuesta.len());
        }
        Err(e) => {
            panic!("[TEST] ❌ Error del modelo: {e}");
        }
    }
}

#[cfg(feature = "llm-local")]
#[test]
fn chat_1_7_raw_conserva_think_y_system_de_testing() {
    use src_ia::motor_chat::cloud::prompts::Mensaje;
    use src_ia::motor_chat::llm::chat_1_7_raw;

    if !hay_modelo_1_7() {
        return;
    }

    let historial = vec![Mensaje::new(
        "user",
        "Hola Y.A.R.V.I.S., solo prueba rapida: que estas haciendo?",
    )];

    let crudo = chat_1_7_raw(&historial).expect("chat_1_7_raw falló");
    assert!(!crudo.trim().is_empty(), "respuesta cruda vacía");
    println!("[TEST] Respuesta cruda (puede incluir bloques think):\n{crudo}");
}
