//! Verificación end-to-end real del parseo con IA: llama EXACTAMENTE a la
//! misma función que usa el backend (`analizar_ticket`, que parsea con el
//! Qwen 3 1.7B, único modelo local, compartido con el chat). Si el GGUF no
//! está descargado en ~/.lmstudio/models, el test se OMITE (no falla).
//!
//! Ejecutar: cargo test --features llm-local --test verificar_conexion -- --nocapture

#[cfg(feature = "llm-local")]
#[test]
fn analizar_ticket_responde_mapeo_ok() {
    use src_ia::rutas::analizar_ticket;
    use src_ia::rutas::verificar_modelos;

    // El parseo usa el 1.7B (único modelo local).
    let info = verificar_modelos()
        .into_iter()
        .find(|(key, _)| *key == "1.7B")
        .map(|(_, info)| info);
    if let Some(info) = info {
        if !info.existe {
            eprintln!(
                "[verificar_conexion] Falta el modelo 1.7B en ~/.lmstudio/models ({}) — test omitido.",
                info.ruta.display()
            );
            return;
        }
    }

    let ticket = "\
Farmacia San Pablo
Av. Juzarez 123, CDMX
Ticket: 004582
Fecha: 15/03/2024  14:32
-----------------------------------
2 Pan Bimbo Integral         42.00     84.00
1 Leche Lala Light 1L        26.50     26.50
3 Refresco Coca-Cola 600ml   18.50     55.50
-----------------------------------
SUBTOTAL: 166.00
IVA 16%: 26.56
TOTAL: $192.56
Tarjeta: **** 1234
Gracias por su compra";

    let resultado = analizar_ticket(ticket);
    println!(
        "{}",
        serde_json::to_string_pretty(&resultado).unwrap_or_else(|_| resultado.to_string())
    );

    let status = resultado.get("status").and_then(|s| s.as_str()).unwrap_or("?");
    assert_eq!(status, "ok", "El análisis debe devolver status ok, se obtuvo {status}");
    assert!(resultado.get("mapeo").is_some(), "Falta el mapeo de columnas");
}
