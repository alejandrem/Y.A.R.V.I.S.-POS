// ============================================================
// impresora/builder.rs — Generador ESC/POS puro Rust (sin deps).
//
// 80mm / 48 columnas. Todo el texto se sanitiza a ASCII imprimible
// (transliteracion de tildes/ñ) porque la termica generica habla
// CP437/latin1 y un UTF-8 crudo imprime basura.
//
// Comandos usados (todos estandar ESC/POS):
//   1B 40       INIT
//   1B 61 n     alineacion (0 izq, 1 centro)
//   1B 45 n     negrita on/off
//   0A          salto de linea
//   1D 56 00    corte total
// ============================================================

/// Columnas de una termica de 80mm con fuente A.
pub const ANCHO_80MM_COLS: usize = 48;
/// Tope de filas por trabajo para no saturar el spooler en Fase 1.
pub const MAX_FILAS: usize = 2000;

const INIT: &[u8] = &[0x1B, 0x40];
const ALIGN_LEFT: &[u8] = &[0x1B, 0x61, 0x00];
const ALIGN_CENTER: &[u8] = &[0x1B, 0x61, 0x01];
const BOLD_ON: &[u8] = &[0x1B, 0x45, 0x01];
const BOLD_OFF: &[u8] = &[0x1B, 0x45, 0x00];
const CORTE: &[u8] = &[0x1D, 0x56, 0x00];

/// Una fila de la tabla de conciliacion fisico vs sistema.
#[derive(Debug, Clone)]
pub struct FilaConciliacion {
    pub nombre: String,
    pub fisico: i32,
    pub sistema: i32,
    pub precio_venta: f64,
}

/// Translitera a ASCII imprimible. La termica no entiende UTF-8.
fn sanitizar(texto: &str) -> String {
    texto
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' | 'Á' | 'À' | 'Ä' | 'Â' => 'a',
            'é' | 'è' | 'ë' | 'ê' | 'É' | 'È' | 'Ë' | 'Ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' | 'Í' | 'Ì' | 'Ï' | 'Î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'Ó' | 'Ò' | 'Ö' | 'Ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' | 'Ú' | 'Ù' | 'Ü' | 'Û' => 'u',
            'ñ' | 'Ñ' => 'n',
            'ç' | 'Ç' => 'c',
            c if c.is_ascii_graphic() || c == ' ' => c,
            _ => '?',
        })
        .collect()
}

fn linea(out: &mut Vec<u8>, texto: &str) {
    out.extend_from_slice(sanitizar(texto).as_bytes());
    out.push(0x0A);
}

fn separador(out: &mut Vec<u8>) {
    out.extend_from_slice(vec![b'-'; ANCHO_80MM_COLS].as_slice());
    out.push(0x0A);
}

fn centrada(out: &mut Vec<u8>, texto: &str) {
    out.extend_from_slice(ALIGN_CENTER);
    linea(out, texto);
    out.extend_from_slice(ALIGN_LEFT);
}

/// Construye el ticket de conciliacion de inventario.
/// `fecha_str` ya viene formateada del llamador (ej. "2026-09-09 15:30").
pub fn construir_lista_conciliacion(
    tienda: &str,
    fecha_str: &str,
    filas: &[FilaConciliacion],
) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(2048 + filas.len() * 64);
    out.extend_from_slice(INIT);
    out.extend_from_slice(ALIGN_LEFT);

    // Encabezado
    out.extend_from_slice(BOLD_ON);
    centrada(&mut out, tienda);
    out.extend_from_slice(BOLD_OFF);
    centrada(&mut out, "Conciliacion de inventario");
    centrada(&mut out, "Fisico vs Sistema");
    centrada(&mut out, fecha_str);
    separador(&mut out);

    // Columnas: NOMBRE(20) FIS(5) SIS(5) DIF(6) ESTADO(8) = 44 + espacios
    linea(&mut out, "NOMBRE               FIS   SIS   DIF  ESTADO");
    separador(&mut out);

    let mut faltantes: i32 = 0;
    let mut sobrantes: i32 = 0;
    let mut por_conciliar: i32 = 0;
    let mut perdida_total: f64 = 0.0;

    for f in filas.iter().take(MAX_FILAS) {
        let dif = f.fisico - f.sistema;
        // Sobreventa: sistema en negativo = reabasto sin capturar.
        // Nunca OK aunque dif == 0 (ej: -1/-1).
        let estado = if f.sistema < 0 {
            por_conciliar += 1;
            "CONCIL"
        } else if dif == 0 {
            "OK"
        } else if dif > 0 {
            sobrantes += 1;
            "SOBRA"
        } else {
            faltantes += 1;
            perdida_total += (dif.abs() as f64) * f.precio_venta;
            "FALTA"
        };
        let mut nombre = sanitizar(&f.nombre);
        if nombre.len() > 20 {
            nombre.truncate(20);
        }
        // Fila monoespaciada de 48 cols aprox.
        let fila = format!(
            "{:<20} {:>5} {:>5} {:>+5}  {:<8}",
            nombre, f.fisico, f.sistema, dif, estado
        );
        linea(&mut out, &fila);
    }

    separador(&mut out);
    out.extend_from_slice(BOLD_ON);
    linea(
        &mut out,
        &format!(
            "Items:{} Faltan:{} Sobran:{} Concil:{}",
            filas.len().min(MAX_FILAS),
            faltantes,
            sobrantes,
            por_conciliar
        ),
    );
    linea(&mut out, &format!("Perdida est.: ${:.2}", perdida_total));
    out.extend_from_slice(BOLD_OFF);
    out.push(0x0A);
    out.push(0x0A);
    centrada(&mut out, "Y.A.R.V.I.S. POS");
    out.push(0x0A);
    out.push(0x0A);
    out.extend_from_slice(CORTE);
    out
}

/// Construye el ticket de alerta de stock bajo (productos en/cajo el mínimo).
/// `stock` puede venir negativo por sobreventa: se imprime tal cual (es la
/// señal para reabastecer), no se aplana a 0.
pub fn construir_lista_stock_bajo(
    tienda: &str,
    fecha_str: &str,
    filas: &[FilaStockBajo],
) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(1024 + filas.len() * 48);
    out.extend_from_slice(INIT);
    out.extend_from_slice(ALIGN_LEFT);

    out.extend_from_slice(BOLD_ON);
    centrada(&mut out, tienda);
    out.extend_from_slice(BOLD_OFF);
    centrada(&mut out, "Alerta de stock bajo");
    centrada(&mut out, fecha_str);
    separador(&mut out);

    linea(&mut out, "NOMBRE               STOCK   MIN");
    separador(&mut out);

    for f in filas.iter().take(MAX_FILAS) {
        let mut nombre = sanitizar(&f.nombre);
        if nombre.len() > 20 {
            nombre.truncate(20);
        }
        let fila = format!(
            "{:<20} {:>7} {:>5}",
            nombre,
            fmt_cant(f.stock),
            fmt_cant(f.minimo)
        );
        linea(&mut out, &fila);
    }

    separador(&mut out);
    out.extend_from_slice(BOLD_ON);
    linea(
        &mut out,
        &format!("Criticos:{}", filas.len().min(MAX_FILAS)),
    );
    out.extend_from_slice(BOLD_OFF);
    out.push(0x0A);
    out.push(0x0A);
    centrada(&mut out, "Y.A.R.V.I.S. POS");
    out.push(0x0A);
    out.push(0x0A);
    out.extend_from_slice(CORTE);
    out
}

/// Cantidad REAL del inventario: entero si es exacto, si no 2 decimales
/// (los KG fraccionarios deben leerse igual en papel y pantalla).
fn fmt_cant(v: f64) -> String {
    if !v.is_finite() {
        return "0".to_string();
    }
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{:.2}", v)
    }
}

/// Una fila de la alerta de stock bajo: lo que hay vs el mínimo.
#[derive(Debug, Clone)]
pub struct FilaStockBajo {
    pub nombre: String,
    pub stock: f64,
    pub minimo: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arma_ticket_con_init_y_corte() {
        let filas = vec![FilaConciliacion {
            nombre: "Coca-Cola 600ml".into(),
            fisico: 10,
            sistema: 12,
            precio_venta: 20.0,
        }];
        let bytes = construir_lista_conciliacion("Mi Tienda", "2026-09-09", &filas);
        assert!(bytes.starts_with(INIT));
        assert!(bytes.ends_with(CORTE));
        let texto = String::from_utf8_lossy(&bytes);
        assert!(texto.contains("Mi Tienda"));
        assert!(texto.contains("Coca-Cola"));
        assert!(texto.contains("FALTA"));
    }

    #[test]
    fn sistema_negativo_marca_concil_aunque_dif_cero() {
        // Sobreventa (-1/-1): nunca OK, marca CONCIL.
        let filas = vec![FilaConciliacion {
            nombre: "Coca-Cola 600ml".into(),
            fisico: -1,
            sistema: -1,
            precio_venta: 20.0,
        }];
        let bytes = construir_lista_conciliacion("Mi Tienda", "2026-09-09", &filas);
        let texto = String::from_utf8_lossy(&bytes);
        assert!(texto.contains("CONCIL"));
        assert!(texto.contains("Concil:1"));
    }

    #[test]
    fn stock_bajo_lista_criticos_con_negativo() {
        // El stock negativo por sobreventa se imprime tal cual (-1), no
        // aplanado: es la señal de reabastecer.
        let filas = vec![
            FilaStockBajo { nombre: "Coca-Cola 600ml".into(), stock: -1.0, minimo: 5.0 },
            FilaStockBajo { nombre: "Jumex Durazno 500ml".into(), stock: 3.0, minimo: 5.0 },
        ];
        let bytes = construir_lista_stock_bajo("Mi Tienda", "2026-09-09", &filas);
        assert!(bytes.starts_with(INIT));
        assert!(bytes.ends_with(CORTE));
        let texto = String::from_utf8_lossy(&bytes);
        assert!(texto.contains("stock bajo"));
        assert!(texto.contains("Coca-Cola"));
        assert!(texto.contains("-1"));
        assert!(texto.contains("Criticos:2"));
    }

    #[test]
    fn sanitiza_tildes_sin_romper() {
        let bytes = construir_lista_conciliacion(
            "Abarrotes Hernández",
            "2026-09-09",
            &[],
        );
        let texto = String::from_utf8_lossy(&bytes);
        assert!(texto.contains("Hernandez"));
    }
}
