//! Verificación end-to-end real del parseo con IA: llama EXACTAMENTE a la
//! misma función que usa el backend (`analizar_ticket`, que arranca con el
//! Qwen 0.5B y escala al 1.7B si la confianza es baja).
//! Solo funciona en máquinas con los GGUFs descargados en ~/.lmstudio/models.
//!
//! Ejecutar: cargo test --features llm-local --test verificar_conexion -- --nocapture

#[cfg(feature = "llm-local")]
#[test]
fn analizar_ticket_responde_mapeo_ok() {
    use src_ia::rutas::analizador_llm::analizar_ticket;
    use src_ia::rutas::rutas_modelos::verificar_modelos;

    for (key, info) in verificar_modelos() {
        assert!(
            info.existe,
            "Falta el modelo {key} en ~/.lmstudio/models ({})",
            info.ruta.display()
        );
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
