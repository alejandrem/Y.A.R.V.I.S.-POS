// ============================================================
// analizador_json — Extracción y post-procesado del JSON que
// devuelve el modelo. Porción de analizador_llm.rs.
// ============================================================

// ---------------------------------------------------------------------------
// Extracción de JSON de la respuesta del modelo (espejo de _extraer_json).
// El Qwen3 puede razonar ANTES de responder (bloques `thinking`/`response`
// con distintos marcadores según la generación, y hasta ejemplos `{...}` en
// medio), así que en vez de fiarse de marcadores: se toma el último `}` y se
// intenta parsear desde CADA `{` anterior hasta él; el primer JSON válido que
// contenga `mapeo` (o el mejor válido) es la respuesta real.
// ---------------------------------------------------------------------------

pub fn extraer_json(respuesta: &str) -> Option<serde_json::Value> {
    let fin = respuesta.rfind('}')?;
    let mut mejor: Option<serde_json::Value> = None;
    for (i, c) in respuesta.char_indices() {
        if c == '{' {
            // Un '{' después del último '}' no puede formar JSON válido;
            // antes hacía panic "byte range starts at X but ends at Y".
            if i > fin {
                continue;
            }
            // rfind y char_indices devuelven índices de byte en ASCII ('{' y '}' son 1 byte),
            // pero por si el modelo mete multi-byte, solo intentamos si ambos son char boundaries.
            if !respuesta.is_char_boundary(i) || !respuesta.is_char_boundary(fin + 1) {
                continue;
            }
            let v: serde_json::Value = match serde_json::from_str(&respuesta[i..=fin]) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if v.get("mapeo").is_some() {
                return Some(v);
            }
            mejor = Some(v);
        }
    }
    mejor
}

#[cfg(test)]
mod tests {
    use super::extraer_json;

    #[test]
    fn razonamiento_con_pensamiento() {
        let r = " thinking\nraciocinio con { ejemplo: 1 }\n</thinking>\n\n{\n  \"mapeo\": {\"a\": 1}\n}";
        let v = extraer_json(r).unwrap();
        assert_eq!(v["mapeo"]["a"], 1);
    }

    #[test]
    fn razonamiento_sin_tags_por_response() {
        let r = " pensamiento...\n response\n\n{\"confianza\": 0.9}";
        let v = extraer_json(r).unwrap();
        assert_eq!(v["confianza"], 0.9);
    }

    #[test]
    fn directo_sin_razonamiento() {
        let v = extraer_json("{\"mapeo\": {\"b\": 2}}").unwrap();
        assert_eq!(v["mapeo"]["b"], 2);
    }

    #[test]
    fn basura_sin_json() {
        assert!(extraer_json("hola\nmundo sin llaves").is_none());
    }
}

/// Inserta `"status": "ok"` dentro del JSON del modelo (espejo de
/// `{ "status": "ok", **resultado }` de Python; Rust no tiene spread en `json!`).
#[cfg(feature = "llm-local")]
pub(crate) fn con_status_ok(mut valor: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = valor.as_object_mut() {
        obj.insert("status".to_string(), serde_json::json!("ok"));
    }
    valor
}
