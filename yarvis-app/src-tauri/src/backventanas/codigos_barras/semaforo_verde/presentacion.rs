// ============================================================
// presentacion — Presentacion canonica del semaforo verde.
//
// Sin regex a proposito (el backend Tauri no trae `regex` como
// dependencia y no queremos agrandarlo por un parseo chico):
// escaneo manual numero + unidad sobre el texto YA normalizado
// (minusculas, ver `src_ia::embeddings::normalizar`).
//
// Canon:
//   * "600ml", "600 ml", "600ML" y "0.6L" son lo mismo (600 ml).
//   * "1kg" == "1000g". "12pzs" vive en su propia base.
//   * Unidades del catalogo cerrado: ml | l | g | kg | pzs.
//     Se aceptan alias de ticket ("gr"->g, "lt"->l, "pz/pza"->pzs)
//     SOLO para comparar; lo que se guarda sigue siendo canonico.
// ============================================================

/// Presentacion en unidades BASE: ml para volumen, g para peso,
/// pzs para piezas. Asi `0.6L == 600ml` sin decimales traicioneros.
#[derive(Debug, Clone, PartialEq)]
pub struct Presentacion {
    pub cantidad_base: f64,
    pub unidad_base: &'static str,
}

/// Mapea una unidad cruda (ya en minusculas) a canonica.
/// Devuelve `None` si no es unidad conocida: no se inventa nada.
pub fn normalizar_unidad(raw: &str) -> Option<&'static str> {
    match raw {
        "ml" | "mililitro" | "mililitros" => Some("ml"),
        "l" | "lt" | "lts" | "litro" | "litros" => Some("l"),
        "g" | "gr" | "grs" | "gramo" | "gramos" => Some("g"),
        "kg" | "kgs" | "kilo" | "kilos" | "kilogramo" | "kilogramos" => Some("kg"),
        "pzs" | "pz" | "pza" | "pzas" | "pieza" | "piezas" | "p" => Some("pzs"),
        _ => None,
    }
}

/// Convierte (cantidad, unidad canonica) a base comparable.
pub fn a_base(cantidad: f64, unidad_canon: &str) -> Option<Presentacion> {
    match unidad_canon {
        "ml" => Some(Presentacion { cantidad_base: cantidad, unidad_base: "ml" }),
        "l" => Some(Presentacion { cantidad_base: cantidad * 1000.0, unidad_base: "ml" }),
        "g" => Some(Presentacion { cantidad_base: cantidad, unidad_base: "g" }),
        "kg" => Some(Presentacion { cantidad_base: cantidad * 1000.0, unidad_base: "g" }),
        "pzs" => Some(Presentacion { cantidad_base: cantidad, unidad_base: "pzs" }),
        _ => None,
    }
}

/// Extrae la ULTIMA presentacion del texto (la del final manda:
/// "coca cola original 600 ml" -> 600 ml, no el "cola" ni nada).
/// Devuelve (presentacion, inicio_byte, fin_byte) para poder recortar
/// la base del nombre despues.
fn ultima_presentacion_span(texto: &str) -> Option<(Presentacion, usize, usize)> {
    let bytes = texto.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    let mut ultima: Option<(Presentacion, usize, usize)> = None;

    while i < n {
        // 1. Buscar inicio de numero.
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let ini = i;
        // Enteros + decimales con . o ,  ("1.5", "0,6").
        let mut j = i;
        let mut sep_visto = false;
        while j < n && (bytes[j].is_ascii_digit() || (!sep_visto && (bytes[j] == b'.' || bytes[j] == b','))) {
            if bytes[j] == b'.' || bytes[j] == b',' {
                sep_visto = true;
            }
            j += 1;
        }
        let num_raw: String = texto[ini..j].replace(',', ".");
        let cantidad: f64 = num_raw.parse().ok()?;
        // 2. Saltar espacios entre numero y unidad ("600 ml").
        let mut k = j;
        while k < n && bytes[k] == b' ' {
            k += 1;
        }
        // 3. Leer letras de la unidad.
        let ini_u = k;
        while k < n && bytes[k].is_ascii_alphabetic() {
            k += 1;
        }
        if ini_u == k {
            i = j;
            continue;
        }
        let unidad_raw = texto[ini_u..k].to_ascii_lowercase();
        if let Some(canon) = normalizar_unidad(&unidad_raw) {
            if let Some(p) = a_base(cantidad, canon) {
                ultima = Some((p, ini, k));
            }
        }
        i = k.max(j.max(ini + 1));
    }
    ultima
}

/// Extrae la presentacion del texto normalizado, si hay.
pub fn extraer_presentacion(texto_norm: &str) -> Option<Presentacion> {
    ultima_presentacion_span(texto_norm).map(|(p, _, _)| p)
}

/// Quita la presentacion final para comparar nombres base:
/// "cocacola 600ml" -> "cocacola", "coca cola original 600 ml" ->
/// "coca cola original". Si no hay presentacion, devuelve el texto
/// recortado tal cual.
pub fn quitar_presentacion(texto_norm: &str) -> String {
    match ultima_presentacion_span(texto_norm) {
        Some((_, ini, fin)) => {
            let mut base = String::with_capacity(texto_norm.len());
            base.push_str(texto_norm[..ini].trim_end());
            let resto = texto_norm[fin..].trim_start();
            if !resto.is_empty() {
                if !base.is_empty() {
                    base.push(' ');
                }
                base.push_str(resto);
            }
            base.trim().to_string()
        }
        None => texto_norm.trim().to_string(),
    }
}

/// Presentacion de una fila de catalogo/dataset (cantidad + unidad
/// ya vienen separadas): "600"+"ml" -> 600 ml base.
pub fn presentacion_de_catalogo(cantidad: f64, unidad: &str) -> Option<Presentacion> {
    let canon = normalizar_unidad(&unidad.trim().to_ascii_lowercase())?;
    a_base(cantidad, canon)
}

/// Igualdad con tolerancia de flotante (0.6*1000 no es exacto).
pub fn misma_presentacion(a: &Presentacion, b: &Presentacion) -> bool {
    a.unidad_base == b.unidad_base && (a.cantidad_base - b.cantidad_base).abs() < 1e-6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_y_bases() {
        // 0.6L == 600ml, 1kg == 1000g.
        let a = extraer_presentacion("cocacola 600ml").unwrap();
        let b = extraer_presentacion("cocacola 0.6l").unwrap();
        assert!(misma_presentacion(&a, &b));
        let k = extraer_presentacion("pan 1kg").unwrap();
        let g = extraer_presentacion("pan 1000g").unwrap();
        assert!(misma_presentacion(&k, &g));
    }

    #[test]
    fn espacios_y_mayusculas() {
        // El texto llega normalizado (minusculas), pero el parser
        // tolera el espacio intermedio y decimales con coma.
        assert!(extraer_presentacion("coca cola 600 ml").is_some());
        assert!(extraer_presentacion("leche 1,5 l").is_some());
        assert!(extraer_presentacion("sabritas 42 g").is_some());
        assert!(extraer_presentacion("jugo sin presentacion").is_none());
    }

    #[test]
    fn medida_manda_mas_que_nombre() {
        // 600ml vs 3L: misma marca, distinta medida -> NO iguales.
        let a = extraer_presentacion("coca 600ml").unwrap();
        let b = extraer_presentacion("coca 3l").unwrap();
        assert!(!misma_presentacion(&a, &b));
    }

    #[test]
    fn quita_presentacion_para_base() {
        assert_eq!(quitar_presentacion("cocacola 600ml"), "cocacola");
        assert_eq!(
            quitar_presentacion("coca cola original 600 ml"),
            "coca cola original"
        );
        assert_eq!(quitar_presentacion("pan dulce"), "pan dulce");
    }

    #[test]
    fn catalogo_cerrado() {
        assert!(presentacion_de_catalogo(600.0, "ml").is_some());
        assert!(presentacion_de_catalogo(1.5, "l").is_some());
        assert!(presentacion_de_catalogo(42.0, "g").is_some());
        assert!(presentacion_de_catalogo(12.0, "pzs").is_some());
        // "gr" es alias de ticket, NO unidad de guardado: el parser lo
        // acepta en texto pero el CHECK de catalogo_barras lo rechaza.
        assert!(presentacion_de_catalogo(42.0, "gr").is_some());
        assert!(presentacion_de_catalogo(1.0, "litros").is_some());
        assert!(presentacion_de_catalogo(1.0, "onzas").is_none());
    }
}
