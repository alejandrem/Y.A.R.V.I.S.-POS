//! Verificación del resolutor de modelos (LM Studio): el modelo local debe
//! estar en el reporte con estado coherente. No requiere feature `llm-local`.
//!
//! Ejecutar: cargo test --test verificar_modelos

use src_ia::rutas::verificar_modelos;

#[test]
fn modelo_1_7_siempre_esta_en_el_resolutor() {
    let modelos = verificar_modelos();
    let keys: Vec<_> = modelos.iter().map(|(k, _)| *k).collect();
    assert!(keys.contains(&"1.7B"), "falta la clave 1.7B: {keys:?}");

    for (key, info) in &modelos {
        if info.existe {
            assert!(info.tamano_mb > 0.0, "{key} existe pero reporta 0MB");
            assert!(
                info.ruta.exists(),
                "{key} existe=true pero la ruta ya no existe"
            );
        } else {
            eprintln!(
                "[verificar_modelos] {key} NO descargado en ~/.lmstudio/models ({})",
                info.ruta.display()
            );
        }
    }
}
