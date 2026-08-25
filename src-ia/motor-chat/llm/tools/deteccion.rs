//! deteccion — Parseo del protocolo textual `<tool_call>{json}</tool_call>`
//! que el fine-tuning de Qwen 1.7B aprendió a emitir (dataset
//! tools_arreglado.jsonl), incluyendo el fallback de JSON desnudo.

use serde_json::Value;

// ─────────────────────────────────────────────────────────────────────────────
// Detección
// ─────────────────────────────────────────────────────────────────────────────

/// Extrae la PRIMERA llamada `<tool_call>{json}</tool_call>` de una respuesta.
/// Devuelve `(nombre, arguments_serializado)` o None si no hay llamada válida.
pub fn detectar_tool_call(respuesta: &str) -> Option<(String, String)> {
    // Camino principal: bloques <tool_call>{json}</tool_call> (formato entrenado).
    if let Some(ini) = respuesta.find("<tool_call>") {
        let resto = &respuesta[ini + "<tool_call>".len()..];
        if let Some(fin) = resto.find("</tool_call>") {
            if let Ok(v) = serde_json::from_str::<Value>(resto[..fin].trim()) {
                if let Some(nombre) = v.get("name").and_then(|n| n.as_str()) {
                    let args = v.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));
                    return Some((nombre.to_string(), args.to_string()));
                }
            }
        }
    }
    // FALLBACK: el modelo a veces escupe el JSON DESNUDO sin etiquetas
    // (visto en pruebas reales): {"name": "...", "arguments": {...}}
    if let Some(i) = respuesta.find(r#"{"name":"#).or_else(|| respuesta.find(r#"{"name": "#)) {
        if let Some((objeto, _fin)) = extraer_objeto_balanceado(&respuesta[i..]) {
            if let Ok(v) = serde_json::from_str::<Value>(objeto) {
                if let Some(nombre) = v.get("name").and_then(|n| n.as_str()) {
                    if !nombre.is_empty() {
                        tracing::info!("[YARVIS-TOOLS] JSON sin etiquetas detectado (fallback)");
                        let args = v.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));
                        return Some((nombre.to_string(), args.to_string()));
                    }
                }
            }
        }
    }
    None
}

/// Extrae el primer objeto {...} BALANCEADO (respeta strings) desde el inicio.
pub(crate) fn extraer_objeto_balanceado(texto: &str) -> Option<(&str, usize)> {
    let bytes = texto.as_bytes();
    if bytes.first() != Some(&b'{') {
        return None;
    }
    let mut profundidad = 0i32;
    let mut en_string = false;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate() {
        if escape {
            escape = false;
            continue;
        }
        match b {
            b'\\' if en_string => escape = true,
            b'"' => en_string = !en_string,
            b'{' if !en_string => profundidad += 1,
            b'}' if !en_string => {
                profundidad -= 1;
                if profundidad == 0 {
                    return Some((&texto[..=i], i + 1));
                }
            }
            _ => {}
        }
    }
    None
}

/// Limpieza final + red de seguridad: si tras el ciclo la respuesta queda
/// vacía (el modelo a veces devuelve cadena nula tras un resultado de tool),
/// se entrega un mensaje digno en vez de una burbuja fantasma.
pub fn respuesta_final_segura(texto: &str) -> String {
    let limpio = quitar_tool_calls(texto);
    if limpio.trim().is_empty() {
        "Consulté el sistema pero no obtuve datos para mostrar. Prueba con otra pregunta o revisa que haya información registrada.".to_string()
    } else {
        limpio
    }
}

/// Quita TODOS los bloques <tool_call> de un texto (para mostrar limpio).
pub fn quitar_tool_calls(respuesta: &str) -> String {
    let mut out = String::new();
    let mut restante = respuesta;
    while let Some(i) = restante.find("<tool_call>") {
        out.push_str(&restante[..i]);
        match restante[i..].find("</tool_call>") {
            Some(j) => restante = &restante[i + j + "</tool_call>".len()..],
            None => return out, // bloque sin cerrar: descartar el resto
        }
    }
    out.push_str(restante);
    // Limpieza final: JSONs desnudos de tool_call sin etiquetas.
    let limpio = re_json_desnudo(&out);
    limpio.trim().to_string()
}

/// Elimina objetos {"name": "...", ...} sueltos (fallback de detección).
fn re_json_desnudo(texto: &str) -> String {
    let mut out = String::new();
    let mut restante = texto;
    while let Some(i) = restante.find(r#"{"name":"#).or_else(|| restante.find(r#"{"name": "#)) {
        out.push_str(&restante[..i]);
        let despues = &restante[i..];
        match extraer_objeto_balanceado(despues) {
            Some((objeto, fin_bytes)) => {
                let es_tool = serde_json::from_str::<Value>(objeto)
                    .ok()
                    .and_then(|v| v.get("name").cloned())
                    .is_some();
                if !es_tool {
                    out.push_str(objeto); // no era un tool_call: conservar
                }
                restante = &despues[fin_bytes..];
            }
            None => {
                // JSON incompleto al final: conservarlo tal cual y terminar
                out.push_str(despues);
                restante = "";
            }
        }
    }
    out.push_str(restante);
    out
}
