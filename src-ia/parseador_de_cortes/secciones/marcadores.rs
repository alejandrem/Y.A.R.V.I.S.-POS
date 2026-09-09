// ============================================================
// marcadores — Partición del corte por secciones.
//
// Cada encabezado (`**Ingresos**`, `VENTAS DEL CORTE`, `Ventas por
// artículo`...) abre una sección; todo lo previo a la primera marca
// es encabezado y se ignora aquí. Tolerante a páginas (`p0`),
// líneas en blanco y separadores.
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marca {
    Ingresos,
    Egresos,
    VentasCorte,
    PorArticulo,
    PorTicket,
    PorCliente,
    Cobranza,
    Fin,
}

/// Línea que no aporta nada: vacía o puro adorno (`---`, `***`).
fn es_ruido(linea: &str) -> bool {
    let t = linea.trim();
    if t.is_empty() {
        return true;
    }
    if t.chars().all(|c| matches!(c, '-' | '*' | '=' | '_' | '.' | ' ')) {
        return true;
    }
    false
}

/// Marca de sección si la línea es encabezado (sin importar adornos).
fn marca_de(linea: &str) -> Option<Marca> {
    let l = linea.to_lowercase();
    let compacta: String = l.chars().filter(|c| !matches!(c, '*' | ' ')).collect();
    if compacta.contains("ventasporart") || l.contains("ventas por art") {
        return Some(Marca::PorArticulo);
    }
    if compacta.contains("ventasporticket") || l.contains("ventas por ticket") {
        return Some(Marca::PorTicket);
    }
    if compacta.contains("ventasporcliente") || l.contains("ventas por cliente") {
        return Some(Marca::PorCliente);
    }
    if l.contains("ventasdelcorte") || l.contains("ventas del corte") {
        return Some(Marca::VentasCorte);
    }
    if l.contains("cobranza") {
        return Some(Marca::Cobranza);
    }
    // "**Ingresos**" / "**Egresos**": solo si la línea es SOLO eso
    // (para no tragarse "Total de Ingresos").
    let pelada: String = l.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    if pelada == "ingresos" {
        return Some(Marca::Ingresos);
    }
    if pelada == "egresos" {
        return Some(Marca::Egresos);
    }
    None
}

/// Reparte las líneas no-ruido en (marca, líneas).
pub fn partir(texto: &str) -> Vec<(Marca, Vec<String>)> {
    let mut partes: Vec<(Marca, Vec<String>)> = Vec::new();
    let mut actual = Marca::Fin;
    for linea in texto.lines() {
        if es_ruido(linea) {
            continue;
        }
        if let Some(m) = marca_de(linea) {
            actual = m;
            partes.push((m, Vec::new()));
            continue;
        }
        match partes.last_mut() {
            Some((m, ls)) if *m == actual => ls.push(linea.trim().to_string()),
            _ => {
                partes.push((actual, vec![linea.trim().to_string()]));
            }
        }
    }
    partes
}

/// Líneas de una marca (vacío si no existe).
pub fn seccion(partes: &[(Marca, Vec<String>)], marca: Marca) -> &[String] {
    partes.iter().find(|(m, _)| *m == marca).map(|(_, ls)| ls.as_slice()).unwrap_or(&[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parte_por_marcas_y_conserva_orden() {
        let t = "p0   *** CORTE Z EN MONEDA:MXN***\n**Ingresos**\nEFE Ventas $10.00\n----------\n**Egresos**\nTotal de Egresos: $.00";
        let partes = partir(t);
        assert_eq!(seccion(&partes, Marca::Ingresos), &["EFE Ventas $10.00".to_string()]);
        assert_eq!(seccion(&partes, Marca::PorArticulo), &[] as &[String]);
    }

    #[test]
    fn total_de_ingresos_no_abre_seccion() {
        let t = "**Ingresos**\nTotal de Ingresos: $10.00";
        let partes = partir(t);
        // El total queda DENTRO de Ingresos, no abre marca nueva.
        assert_eq!(seccion(&partes, Marca::Ingresos).len(), 1);
    }
}
