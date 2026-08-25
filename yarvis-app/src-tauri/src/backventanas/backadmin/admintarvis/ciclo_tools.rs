// ============================================================
// admintarvis/ciclo_tools.rs — Ciclo tool_call→ejecutar→re-preguntar
// y supresión de bloques <tool_call> del stream token a token.
// ============================================================

use src_ia::motor_chat::cloud::prompts::Mensaje;
use src_ia::motor_chat::llm::tools;

use super::herramientas_rol::ejecutar_tool_con_rol;

pub(super) type GeneradorFut =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>;
pub(super) type Generador<'a> = Box<dyn FnMut(Vec<Mensaje>) -> GeneradorFut + Send + 'a>;

/// Ciclo tool_call→ejecutar→re-preguntar. Mientras el modelo siga pidiendo
/// herramientas (hasta MAX rondas), ejecuta el SQL real y le devuelve el
/// resultado como mensaje role:"tool" hasta obtener una respuesta final.
pub(super) async fn resolver_ciclo_tools(
    mut respuesta: String,
    mut historial: Vec<Mensaje>,
    db_path: String,
    es_empleado: bool,
    generar: &mut Generador<'_>,
) -> Result<String, String> {
    for _ in 0..tools::MAX_RONDAS_TOOLS {
        let Some((nombre, args)) = tools::detectar_tool_call(&respuesta) else {
            return Ok(respuesta);
        };
        tracing::info!("[YARVIS-TOOLS] ejecutando {nombre}({args})");
        let json_res = ejecutar_tool_con_rol(&nombre, &args, &db_path, es_empleado).await;
        historial.push(Mensaje::new("assistant", respuesta));
        historial.push(Mensaje::new("tool", json_res));
        respuesta = (&mut *generar)(historial.clone()).await?;
    }
    // Agotó rondas: entregar limpio (sin bloques crudos)
    Ok(tools::respuesta_final_segura(&respuesta))
}

/// Suprime bloques <tool_call>...</tool_call> de un stream token a token,
/// reteniendo colas parciales que podrían ser el inicio del marcador.
pub(super) struct SupresorToolCall {
    retenido: String,
    en_bloque: bool,
}

impl SupresorToolCall {
    pub(super) fn new() -> Self {
        Self { retenido: String::new(), en_bloque: false }
    }

    pub(super) fn procesar(&mut self, frag: &str) -> String {
        self.retenido.push_str(frag);
        let mut out = String::new();
        loop {
            if self.en_bloque {
                if let Some(i) = self.retenido.find("</tool_call>") {
                    self.retenido.drain(..i + "</tool_call>".len());
                    self.en_bloque = false;
                    continue;
                }
                self.retenido.clear();
                break;
            }
            match self.retenido.find("<tool_call>") {
                Some(i) => {
                    out.push_str(&self.retenido[..i]);
                    self.retenido.drain(..i + "<tool_call>".len());
                    self.en_bloque = true;
                }
                None => {
                    let max_hold = "<tool_call>".len() - 1;
                    if self.retenido.len() > max_hold {
                        let mut corte = self.retenido.len() - max_hold;
                        while corte > 0 && !self.retenido.is_char_boundary(corte) {
                            corte -= 1;
                        }
                        out.push_str(&self.retenido[..corte]);
                        self.retenido.drain(..corte);
                    }
                    break;
                }
            }
        }
        out
    }

    pub(super) fn finalizar(&mut self) -> String {
        if self.en_bloque {
            self.retenido.clear();
            String::new()
        } else {
            std::mem::take(&mut self.retenido)
        }
    }
}
