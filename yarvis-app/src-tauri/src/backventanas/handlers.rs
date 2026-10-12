// ============================================================
// handlers.rs — Mapa de dominios del invoke_handler (NO registra).
//
// Por qué el registro vive en UN solo `generate_handler!` en lib.rs:
//   · La macro `tauri::generate_handler!` solo acepta rutas literales
//     (`arbol::modulo::comando`); no acepta `pub use`, ni macros
//     anidadas, ni `include!` (su parser es `Path`, ver
//     tauri-macros/src/command/handler.rs).
//   · Su expansión es `move |invoke| { match cmd { ... } }`: el
//     `Invoke` se MUEVE al wrapper que acierta, así que NO se pueden
//     combinar varios handlers con `||` (el primero consume el valor).
//   · `Builder::invoke_handler` es setter único, no acumulador.
//
// Regla de alta (4 pasos, en orden):
//   1. `#[tauri::command]` en su dominio (backadmin/backempleado/...).
//   2. 1 línea en el bloque de su dominio en lib.rs (rutas completas).
//   3. `invokeTauri("nombre")` en `yarvis-app/src/services/`.
//   4. `npm run verificar:comandos` en verde (scripts/verificar-comandos.mjs).
//
// Dominios (secciones en lib.rs): sistema | auth | inventario |
// parser | tickets | empleados | operacion | finanzas | chat.
// El test de abajo impide duplicados y borrados accidentales.
// ============================================================

/// Dominios del handler, en el mismo orden que aparecen en lib.rs.
/// Solo documentación viva: el conteo real lo verifica el test.
pub const DOMINIOS: &[&str] = &[
    "sistema",    // principal_lista, impresora::*, hora::*
    "auth",       // api_config + adminconfig::auth + google
    "inventario", // inventory + codigos_barras (3 semáforos)
    "parser",     // adminparser + cortes_import
    "tickets",    // admintickets::tickets
    "empleados",  // adminempleados (empleados + modal + metas)
    "operacion",  // emplea_new_venta + empleaproveedores + empleaperfil + empleatickets
    "finanzas",   // gastos + cortes + backcortes + metricas + graficas + alertas + export
    "chat",       // admintarvis::chat
];

/// Mínimo de comandos esperados. Si agregas uno, este número SOLO
/// puede subir (el test falla si baja: borrado accidental).
pub const MIN_COMANDOS: usize = 145;

#[cfg(test)]
mod tests {
    use super::MIN_COMANDOS;

    /// Extrae el nombre del comando de cada línea del bloque
    /// `generate_handler!` de lib.rs (último segmento tras `::`,
    /// o ident desnudo como `principal_lista`).
    fn comandos_en_lib_rs() -> Vec<String> {
        let src = include_str!("../lib.rs");
        let inicio = src
            .find("generate_handler![")
            .expect("lib.rs debe contener generate_handler![");
        let resto = &src[inicio..];
        let fin_rel = resto.find("])").expect("cierre ])");
        let bloque = &resto[..fin_rel];
        let mut nombres = Vec::new();
        for linea in bloque.lines().skip(1) {
            let t = linea.trim();
            if t.is_empty() || t.starts_with("//") {
                continue;
            }
            // Quita coma final y toma el último segmento.
            let sin_coma = t.trim_end_matches(',').trim();
            if sin_coma.is_empty() {
                continue;
            }
            let nombre = sin_coma.rsplit("::").next().unwrap_or("").trim();
            if !nombre.is_empty()
                && nombre.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && nombre.starts_with(|c: char| c.is_ascii_lowercase())
            {
                nombres.push(nombre.to_string());
            }
        }
        nombres
    }

    #[test]
    fn handler_sin_duplicados_y_sin_borrados() {
        let nombres = comandos_en_lib_rs();
        assert!(
            nombres.len() >= MIN_COMANDOS,
            "se esperaban ≥{MIN_COMANDOS} comandos, hay {}: posible borrado accidental",
            nombres.len()
        );
        let mut vistos = std::collections::HashSet::new();
        for n in &nombres {
            assert!(vistos.insert(n.clone()), "comando duplicado en handler: {n}");
        }
    }
}
