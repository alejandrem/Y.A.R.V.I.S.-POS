//! Equivalente a `python rutas_modelos.py` (bloque `__main__` de Python).
//! Ejecutar con: `cargo run --example verificar_modelos`

use src_ia::rutas::rutas_modelos::verificar_modelos;

fn main() {
    println!("=== Verificación de Modelos Qwen ===");
    for (key, info) in verificar_modelos() {
        let status = if info.existe { "✅" } else { "❌" };
        if info.existe {
            println!(
                "  {} Qwen {}: {}MB — {}",
                status,
                key,
                info.tamano_mb,
                info.ruta.display()
            );
        } else {
            println!(
                "  {} Qwen {}: NO ENCONTRADO — {}",
                status,
                key,
                info.ruta.display()
            );
        }
    }
}