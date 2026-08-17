//! Verificación end-to-end temporal: llama EXACTAMENTE a la misma función
//! que usa el backend (`src_ia::rutas::analizador_llm::analizar_ticket`).
//! Ejecutar con: cargo run --example verificar_conexion --features llm-local

use src_ia::rutas::analizador_llm::analizar_ticket;
use src_ia::rutas::rutas_modelos::verificar_modelos;

fn main() {
    println!("=== Paso 1/2: Modelos en disco ===");
    for (key, info) in verificar_modelos() {
        let status = if info.existe { "OK" } else { "NO" };
        println!("  [{status}] Qwen {key}: {}MB -> {}", info.tamano_mb, info.ruta.display());
        assert!(info.existe, "Falta el modelo {key}");
    }

    println!("\n=== Paso 2/2: analizar_ticket (inferencia real) ===");
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
    println!("{}", serde_json::to_string_pretty(&resultado).unwrap_or_else(|_| resultado.to_string()));

    let status = resultado.get("status").and_then(|s| s.as_str()).unwrap_or("?");
    assert_eq!(status, "ok", "El análisis debe devolver status ok, se obtuvo {status}");
    assert!(resultado.get("mapeo").is_some(), "Falta el mapeo de columnas");

    println!("\n=== CONEXION VERIFICADA: analizar_ticket responde desde Rust ===");
}