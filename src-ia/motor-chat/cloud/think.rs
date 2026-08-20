//! think.rs — Separación de bloques  think / response en streaming.
//!
//! Espejo de `_separar_think` y `limpiar_think` de
//! `yarvis-IA/chatbot/motor_chat/modelos_local/prompts.py`.
//!
//! Los modelos de razonamiento intercalan su razonamiento entre marcadores:
//! ` think ... response `. En streaming estos marcadores pueden llegar PARTIDOS
//! entre chunks, así que esta lógica acumula en un buffer y solo flushea lo
//! seguro, reteniendo la cola que aún podría ser el inicio de un marcador.

use regex::Regex;
use std::sync::OnceLock;

/// Tipo de fragmento emitido por [`SeparadorThink`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoFragmento {
    /// Razonamiento del modelo (se muestra sombreado).
    Think,
    /// Respuesta final (texto real).
    Token,
}

// Marcadores que abren/cierran un bloque de razonamiento del modelo.
// " think" / " thinking" abren; " response" cierra. Aceptan espacio previo o
// inicio de texto (los razonamientos llegan así desde el proveedor).
const _OPEN_THINK_SRC: &str = r#" thinking|(?:\s+|^)think(?:ing)?\b"#;
const _CLOSE_THINK_SRC: &str = r#" response|<response>|(?:\s+|^)response\b"#;

// Prefijos parciales que podrían ser el inicio de un marcador (llegan troceados).
// Espejo de `_MARCADORES` de Python (orden y valores idénticos).
const _MARCADORES: &[&str] = &[
    " thinking",
    " think",
    " thinking",
    " response",
    "<response>",
];

fn open_think() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(_OPEN_THINK_SRC).expect("regex de apertura think válida"))
}

fn close_think() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(_CLOSE_THINK_SRC).expect("regex de cierre think válida"))
}

/// Cuántos caracteres del final de `texto` podrían ser el inicio de un marcador.
///
/// Espejo de `_cola_potencial_marcador` de Python. Como los prefijos de
/// marcador son ASCII, el recuento en caracteres equivale al recuento en bytes,
/// y la frontera de corte cae en un límite UTF-8 válido.
fn cola_potencial_marcador(texto: &str) -> usize {
    for m in _MARCADORES {
        let max_i = m.len().min(texto.len());
        for i in 1..=max_i {
            if texto.ends_with(&m[..i]) {
                return i;
            }
        }
    }
    0
}

/// Separador de bloques think/response en streaming (port de `_separar_think`).
///
/// Recibe texto crudo por trozos vía [`SeparadorThink::procesar`] y emite
/// fragmentos `('think'|'token', texto)`. Al terminar, [`SeparadorThink::finalizar`]
/// devuelve el buffer residual.
#[derive(Debug)]
pub struct SeparadorThink {
    max_w: usize,
    word_count: usize,
    in_think: bool,
    buffer: String,
    /// Dejó de emitir por superar `max_w` (como el `return` de Python).
    agotado: bool,
}

impl SeparadorThink {
    pub fn new(max_w: usize) -> Self {
        SeparadorThink {
            max_w,
            word_count: 0,
            in_think: false,
            buffer: String::new(),
            agotado: false,
        }
    }

    /// Si [`TipoFragmento`] es texto final (no razonamiento).
    pub fn es_texto(tipo: TipoFragmento) -> bool {
        tipo == TipoFragmento::Token
    }

    /// Procesa un trozo de texto y devuelve los fragmentos emitidos.
    pub fn procesar(&mut self, contenido: &str) -> Vec<(TipoFragmento, String)> {
        let mut out = Vec::new();
        if self.agotado {
            return out;
        }
        if contenido.is_empty() {
            return out;
        }
        self.buffer.push_str(contenido);

        'outer: while !self.buffer.is_empty() {
            if self.agotado {
                break 'outer;
            }
            let patron = if self.in_think {
                close_think()
            } else {
                open_think()
            };

            if let Some(mat) = patron.find(&self.buffer) {
                let idx = mat.start();
                let fin = mat.end();
                let pre = self.buffer[..idx].to_string();
                let resto = self.buffer[fin..].to_string();

                // `pre` va ANTES del marcador: se emite con el estado ACTUAL
                // (espejo exacto de `_separar_think`, que emite pre y LUEGO
                // hace `buffer = buffer[match.end():]` y `in_think = not in_think`).
                if !pre.is_empty() {
                    if self.in_think {
                        // Cerramos un bloque: lo anterior era razonamiento.
                        out.push((TipoFragmento::Think, pre));
                    } else {
                        // Abrimos un bloque: lo anterior era texto final.
                        self.emitir_token(pre, &mut out);
                    }
                }
                self.buffer = resto;
                self.in_think = !self.in_think;
                continue 'outer;
            }

            // Sin marcador en el buffer: flushear lo seguro y retener la cola.
            let cola = cola_potencial_marcador(&self.buffer);
            if cola == 0 {
                let seguro = self.buffer.clone();
                self.buffer.clear();
                if self.in_think {
                    out.push((TipoFragmento::Think, seguro));
                } else {
                    self.emitir_token(seguro, &mut out);
                }
                continue 'outer;
            }

            let corte = self.buffer.len() - cola;
            let seguro = self.buffer[..corte].to_string();
            let retener = self.buffer[corte..].to_string();
            self.buffer = retener;
            if seguro.is_empty() {
                // Todo el buffer es cola potencial de marcador: esperar más.
                break 'outer;
            }
            if self.in_think {
                out.push((TipoFragmento::Think, seguro));
            } else {
                self.emitir_token(seguro, &mut out);
            }
        }
        out
    }

    /// Devuelve el buffer residual al terminar el stream.
    pub fn finalizar(&mut self) -> Vec<(TipoFragmento, String)> {
        if self.agotado || self.buffer.is_empty() {
            self.buffer.clear();
            return Vec::new();
        }
        let mut out = Vec::new();
        let tipo = if self.in_think {
            TipoFragmento::Think
        } else {
            TipoFragmento::Token
        };
        out.push((tipo, self.buffer.clone()));
        self.buffer.clear();
        out
    }

    fn emitir_token(&mut self, texto: String, out: &mut Vec<(TipoFragmento, String)>) {
        self.word_count += texto.split_whitespace().count();
        if self.word_count > self.max_w {
            self.agotado = true;
            self.buffer.clear();
            return;
        }
        out.push((TipoFragmento::Token, texto));
    }
}

/// Elimina bloques  thinking / think ... response /  thinking de la respuesta.
///
/// Espejo de `limpiar_think` de Python (regex con DOTALL).
pub fn limpiar_think(texto: &str) -> String {
    let mut s = texto.to_string();
    let re1 = Regex::new(r"(?s) thinking.*? response").expect("regex válida");
    s = re1.replace_all(&s, "").into_owned();
    let re2 = Regex::new(r"(?s)\s*think(?:ing)?\b.*?\s*response\b").expect("regex válida");
    s = re2.replace_all(&s, "").into_owned();
    let re3 = Regex::new(r"(?s)\s*think(?:ing)?\b.*").expect("regex válida");
    s = re3.replace_all(&s, "").into_owned();
    s.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limpiar_think_elimina_bloques_completos() {
        let texto = "  thinking interno  response final.";
        let limpio = limpiar_think(texto);
        assert_eq!(limpio, "final.");
    }

    #[test]
    fn separador_basico_separa_token_y_think() {
        let mut sep = SeparadorThink::new(1000);
        let frags = sep.procesar("hola  think razon  response mundo");
        let tipos: Vec<TipoFragmento> = frags.iter().map(|(t, _)| *t).collect();
        assert_eq!(
            tipos,
            vec![
                TipoFragmento::Token,
                TipoFragmento::Think,
                TipoFragmento::Token
            ]
        );
        let texto: String = frags.iter().map(|(_, s)| s.clone()).collect();
        assert!(
            texto.contains("hola") && texto.contains("razon") && texto.contains("mundo"),
            "esperaba hola/razon/mundo en: {texto:?}"
        );
    }

    #[test]
    fn marcador_partido_se_acumula_en_buffer() {
        let mut sep = SeparadorThink::new(1000);
        // "  thi" → " thi" es cola potencial de " think": no se emite aún.
        let frags = sep.procesar("hola  thi");
        let tokens: Vec<(TipoFragmento, String)> = frags
            .into_iter()
            .filter(|(t, _)| *t == TipoFragmento::Token)
            .collect();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].1, "hola ");

        // Llega el resto del marcador + razonamiento + cierre.
        let frags = sep.procesar("nking razonar  response final");
        let tipos_vistos: Vec<TipoFragmento> = frags.iter().map(|(t, _)| *t).collect();
        assert!(tipos_vistos.contains(&TipoFragmento::Think));
        let texto: String = frags.iter().map(|(_, s)| s.clone()).collect();
        assert!(
            texto.contains("razonar") && texto.contains("final"),
            "esperaba razonar/final en: {texto:?}"
        );
    }

    #[test]
    fn separador_respeta_max_w() {
        let mut sep = SeparadorThink::new(2);
        let mut frags = vec![];
        frags.extend(sep.procesar("una dos tres cuatro"));
        frags.extend(sep.finalizar());
        let total: usize = frags
            .iter()
            .filter(|(t, _)| *t == TipoFragmento::Token)
            .map(|(_, s)| s)
            .flat_map(|s| s.split_whitespace())
            .count();
        assert!(total <= 2, "word_count excedió max: {total}");
    }

    #[test]
    fn finalizar_devuelve_buffer_residual() {
        let mut sep = SeparadorThink::new(1000);
        let frags = sep.procesar("sin marcadores aquí");
        assert_eq!(frags.len(), 1);
        assert_eq!(sep.finalizar(), Vec::new());
    }

    #[test]
    fn think_sin_cierre_se_trata_como_token_al_finalizar() {
        let mut sep = SeparadorThink::new(1000);
        let mut frags = vec![];
        frags.extend(sep.procesar("hola  think incompl"));
        frags.extend(sep.finalizar());
        let texto: String = frags.iter().map(|(_, s)| s.clone()).collect();
        assert_eq!(texto, "hola incompl");
    }
}
