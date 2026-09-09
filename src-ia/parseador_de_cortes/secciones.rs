// ============================================================
// secciones — Detección de tipo, partición por marcadores y
// extracción de encabezado, totales e items (X y Z).
//
// Todo case-insensitive y tolerante a páginas (`p0`), líneas en
// blanco y separadores (`---`, `***`, `===`). Las etiquetas de
// totales se buscan SOLO dentro de su sección para no confundir
// "Impuesto 16%" con "Impuesto" ni "Total de ventas" con
// "Total ventas del dia".
// ============================================================

use regex::Regex;
use std::sync::LazyLock;

use super::fechas::a_iso;
use super::montos::limpiar_monto;
use super::tipos::ClaseArchivo;

// ── Marcadores ───────────────────────────────────────────────

static RE_TITULO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\*{3}\s*CORTE\s+([XZ])\b").expect("regex titulo corte")
});
static RE_MONEDA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)MONEDA\s*:\s*([A-Z]{3})").expect("regex moneda"));
static RE_FOLIO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?im)^\**\s*Corte\s+[XZ]\s+(\d+)").expect("regex folio corte")
});
static RE_ESTACION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^([A-Z0-9_]+)\s+(\d{2}/\d{2}/\d{4})\s+(\d{1,2}:\d{2}(?::\d{2})?\s*(?:[AaPp]\.\s*[Mm]\.?)?)\s*$")
        .expect("regex estacion")
});
static RE_CAJERO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?im)^Cajero\s*:\s*(.+?)\s*$").expect("regex cajero"));

/// Línea que no aporta nada: vacía, separador o número de página.
fn es_ruido(linea: &str) -> bool {
    let t = linea.trim();
    if t.is_empty() {
        return true;
    }
    // Solo adornos: - * = _ . y espacios (p. ej. "---", "***", "p0"... no:
    // "p0" tiene letra, se filtra abajo por no matchear nada útil).
    if t.chars().all(|c| matches!(c, '-' | '*' | '=' | '_' | '.' | ' ')) {
        return true;
    }
    false
}

/// Clasifica un archivo suelto (los datasets mezclan tickets y cortes).
pub fn clasificar(texto: &str) -> ClaseArchivo {
    match RE_TITULO.captures(texto).and_then(|c| c.get(1)).map(|m| m.as_str().to_ascii_uppercase()) {
        Some(t) if t == "X" => ClaseArchivo::CorteX,
        Some(t) if t == "Z" => ClaseArchivo::CorteZ,
        _ => ClaseArchivo::NoEsCorte,
    }
}

/// Moneda del título (`MONEDA:MXN`), default MXN.
pub fn extraer_moneda(texto: &str) -> String {
    RE_MONEDA
        .captures(texto)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_ascii_uppercase())
        .unwrap_or_else(|| "MXN".to_string())
}

/// Folio del corte (`*** Corte Z 54`).
pub fn extraer_folio(texto: &str) -> Option<String> {
    RE_FOLIO.captures(texto).and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
}

/// Empresa: primera línea con contenido tras el título que no sea
/// folio, estación ni cajero (p. ej. `EMPRESA, S.A. DE C.V.`).
pub fn extraer_empresa(texto: &str) -> Option<String> {
    let lineas: Vec<&str> = texto.lines().collect();
    let titulo = lineas.iter().position(|l| RE_TITULO.is_match(l))?;
    for l in lineas.iter().skip(titulo + 1) {
        let t = l.trim();
        if t.is_empty() || RE_FOLIO.is_match(t) || RE_ESTACION.is_match(t) || RE_CAJERO.is_match(t) {
            continue;
        }
        // Marcadores de página sueltos (`p0`) o secciones: no son empresa.
        if t.len() <= 3 || t.starts_with('*') {
            continue;
        }
        return Some(t.to_string());
    }
    None
}

/// Estación + fecha ISO (`ESTACION01 01/01/2025 11:25:57 a. m.`).
pub fn extraer_estacion_fecha(texto: &str) -> (Option<String>, Option<String>) {
    match RE_ESTACION.captures(texto) {
        None => (None, None),
        Some(c) => {
            let estacion = c[1].to_string();
            let fecha = a_iso(&c[2], &c[3]);
            (Some(estacion), fecha)
        }
    }
}

/// Cajero del encabezado (`Cajero: GENERAL`); SISTEMA si no viene
/// (los Z del dataset no lo traen).
pub fn extraer_cajero(texto: &str) -> String {
    RE_CAJERO
        .captures(texto)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "SISTEMA".to_string())
}

// ── Partición por secciones ──────────────────────────────────

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

/// Marca de sección si la línea es encabezado (contiene la palabra
/// clave, sin importar adornos `*`).
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

/// Reparte las líneas no-ruido en (marca, líneas). Todo lo previo a
/// la primera marca va a `Fin` (encabezado, se ignora aquí).
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

// ── Totales ──────────────────────────────────────────────────

/// Monto al final de la primera línea que contenga la etiqueta
/// (case-insensitive). La línea debe TERMINAR en el monto.
fn monto_tras(lineas: &[String], etiqueta: &str) -> Option<i64> {
    static RE_FIN: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\$?\s*([\d,]+\.\d{2}|\.\d{2}|\d+)\s*$").expect("regex monto final")
    });
    let et = etiqueta.to_lowercase();
    for l in lineas {
        if !l.to_lowercase().contains(&et) {
            continue;
        }
        // Porcentajes ("16%") no son el monto: se buscan tras los ':'.
        let cola = l.split(':').last().unwrap_or(l);
        if let Some(c) = RE_FIN.captures(cola) {
            if let Some(v) = limpiar_monto(&c[1]) {
                return Some(v);
            }
        }
    }
    None
}

/// Etiqueta anclada al inicio (`^impuesto\s*:`) para no confundirla
/// con `Impuesto 16%:` / `Impuesto 10%:`.
fn monto_anclado(lineas: &[String], etiqueta: &str) -> Option<i64> {
    let patron = format!(r"(?i)^\**\s*{etiqueta}\s*:");
    let re = Regex::new(&patron).ok()?;
    for l in lineas {
        if re.is_match(l) {
            let cola = l.split(':').last().unwrap_or(l);
            static RE_FIN: LazyLock<Regex> = LazyLock::new(|| {
                Regex::new(r"\$?\s*([\d,]+\.\d{2}|\.\d{2}|\d+)\s*$").expect("regex monto final 2")
            });
            if let Some(c) = RE_FIN.captures(cola) {
                if let Some(v) = limpiar_monto(&c[1]) {
                    return Some(v);
                }
            }
        }
    }
    None
}

/// Todos los totales del corte (faltantes quedan en None).
#[derive(Debug, Default)]
pub struct TotalesCorte {
    pub ingresos: Option<i64>,
    pub egresos: Option<i64>,
    pub caja: Option<i64>,
    pub ventas_16: Option<i64>,
    pub impuesto_16: Option<i64>,
    pub ventas_10: Option<i64>,
    pub impuesto_10: Option<i64>,
    pub gravadas: Option<i64>,
    pub impuesto: Option<i64>,
    pub no_gravadas: Option<i64>,
    pub redondeos: Option<i64>,
    pub total_ventas: Option<i64>,
    pub ventas_credito: Option<i64>,
    pub unidades: Option<f64>,
    pub clientes: Option<i64>,
}

pub fn extraer_totales(partes: &[(Marca, Vec<String>)], texto: &str) -> TotalesCorte {
    let ing = seccion(partes, Marca::Ingresos);
    let egr = seccion(partes, Marca::Egresos);
    let vc = seccion(partes, Marca::VentasCorte);
    let mut t = TotalesCorte::default();
    t.ingresos = monto_tras(ing, "total de ingresos");
    t.egresos = monto_tras(egr, "total de egresos");
    t.caja = monto_tras(ing, "total en caja")
        .or_else(|| monto_tras(egr, "total en caja"))
        .or_else(|| monto_tras(&partes.iter().flat_map(|(_, ls)| ls.clone()).collect::<Vec<_>>(), "total en caja"));
    t.ventas_16 = monto_tras(vc, "ventas 16%");
    t.impuesto_16 = monto_tras(vc, "impuesto 16%");
    t.ventas_10 = monto_tras(vc, "ventas 10%");
    t.impuesto_10 = monto_tras(vc, "impuesto 10%");
    t.gravadas = monto_anclado(vc, "ventas gravadas");
    t.impuesto = monto_anclado(vc, "impuesto");
    t.no_gravadas = monto_anclado(vc, "ventas no gravadas");
    t.redondeos = monto_tras(vc, "redondeos");
    t.total_ventas = monto_tras(vc, "total de ventas")
        .or_else(|| monto_tras(&partes.iter().flat_map(|(_, ls)| ls.clone()).collect::<Vec<_>>(), "total ventas del dia"));
    t.ventas_credito = monto_tras(vc, "ventas credito");
    // Unidades: `**Total venta en unidades:      20.00` (puede ser decimal).
    for l in partes.iter().flat_map(|(_, ls)| ls.iter()) {
        if l.to_lowercase().contains("total venta en unidades") {
            let cola = l.split(':').last().unwrap_or(l);
            let num: String = cola.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
            t.unidades = num.parse::<f64>().ok();
            break;
        }
    }
    // `Clientes atendidos: 9`.
    for l in partes.iter().flat_map(|(_, ls)| ls.iter()) {
        if l.to_lowercase().contains("clientes atendidos") {
            let num: String = l.chars().filter(|c| c.is_ascii_digit()).collect();
            t.clientes = num.parse::<i64>().ok();
            break;
        }
    }
    // `fecha` no vive aquí; silencio el unused.
    let _ = texto;
    t
}

// ── Items ────────────────────────────────────────────────────

static RE_ARTICULO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<nombre>.+?)\s+-\s+(?P<cant>[\d.,]+)\s+-\s+\$?(?P<monto>[\d,]+\.\d{2}|\.\d{2})\s*$")
        .expect("regex articulo")
});
static RE_TICKET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<folio>[^*:\n]+?)\s{2,}(?P<monto>[\d,]+\.\d{2})\s*$").expect("regex ticket corte")
});
static RE_PAGO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<c>.+?)\s+\$?(?P<m>[\d,]+\.\d{2}|\.\d{2}|\d+)\s*$").expect("regex pago")
});

/// `JACK DNL HONEY 70 -       1 -   $535.00` → (nombre, cant, subtotal).
pub fn linea_articulo(linea: &str) -> Option<(String, f64, i64)> {
    let c = RE_ARTICULO.captures(linea.trim())?;
    let mut nombre = c["nombre"].trim().to_string();
    // Nombres truncados terminan en " -": se recorta.
    while nombre.ends_with('-') {
        nombre.pop();
    }
    nombre = nombre.trim().to_string();
    if nombre.is_empty() {
        return None;
    }
    let cant: f64 = c["cant"].replace(',', "").parse().ok()?;
    let sub = limpiar_monto(&c["monto"])?;
    Some((nombre, cant, sub))
}

/// `REM - 10834 \t\t 94.00` → (folio, monto). Rechaza totales y
/// líneas con ':' (folios reales no lo llevan).
pub fn linea_ticket(linea: &str) -> Option<(String, i64)> {
    let t = linea.trim();
    if t.is_empty() || t.contains(':') || t.to_lowercase().contains("total") {
        return None;
    }
    let c = RE_TICKET.captures(t)?;
    let folio = c["folio"].trim().to_string();
    if folio.is_empty() {
        return None;
    }
    Some((folio, limpiar_monto(&c["monto"])?))
}

/// `EFE Pago de clientes  $2,023.80` → (concepto, monto). Rechaza la
/// línea del total, `Total en caja` (vive dentro de Egresos pero NO es
/// un egreso) y separadores residuales: ningún concepto real contiene
/// la palabra "total".
pub fn linea_pago(linea: &str, total_etiqueta: &str) -> Option<(String, i64)> {
    let t = linea.trim();
    if t.is_empty()
        || t.to_lowercase().contains("total")
        || t.to_lowercase().contains(&total_etiqueta.to_lowercase())
    {
        return None;
    }
    let c = RE_PAGO.captures(t)?;
    let concepto = c["c"].trim().to_string();
    if concepto.is_empty() {
        return None;
    }
    Some((concepto, limpiar_monto(&c["m"])?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clasifica_x_z_y_descarta_tickets() {
        assert_eq!(clasificar("*** CORTE Z EN MONEDA:MXN***\n*** Corte Z 54"), ClaseArchivo::CorteZ);
        assert_eq!(clasificar("p0   *** CORTE X EN MONEDA:MXN***\nCorte X 1"), ClaseArchivo::CorteX);
        assert_eq!(clasificar("2 Pan Bimbo 42.00 84.00\nTOTAL: $84.00"), ClaseArchivo::NoEsCorte);
    }

    #[test]
    fn encabezado_completo_del_z() {
        let t = "*** CORTE Z EN MONEDA:MXN***\nEMPRESA, S.A. DE C.V.\n\n*** Corte Z 54\nESTACION01 01/01/2025 11:25:57 a. m.\n**Ingresos**";
        assert_eq!(extraer_folio(t), Some("54".into()));
        assert_eq!(extraer_empresa(t), Some("EMPRESA, S.A. DE C.V.".into()));
        assert_eq!(extraer_estacion_fecha(t), (Some("ESTACION01".into()), Some("2025-01-01 11:25:57".into())));
        assert_eq!(extraer_cajero(t), "SISTEMA");
        assert_eq!(extraer_moneda(t), "MXN");
    }

    #[test]
    fn cajero_solo_en_x() {
        assert_eq!(extraer_cajero("Cajero: GENERAL\nCorte X 1"), "GENERAL");
    }

    #[test]
    fn articulos_truncados_y_cantidades() {
        assert_eq!(
            linea_articulo("JACK DNL HONEY 70 -       1 -   $535.00"),
            Some(("JACK DNL HONEY 70".into(), 1.0, 53500))
        );
        assert_eq!(
            linea_articulo("NEGRA MODELO 1L C -       3 -   $136.80"),
            Some(("NEGRA MODELO 1L C".into(), 3.0, 13680))
        );
        assert_eq!(linea_articulo("-----------------------------------"), None);
    }

    #[test]
    fn tickets_con_folio_rem() {
        assert_eq!(linea_ticket("REM - 10834\t\t                  94.00"), Some(("REM - 10834".into(), 9400)));
        assert_eq!(linea_ticket("**Total ventas del dia :     702.00"), None);
    }

    #[test]
    fn pagos_con_codigo_y_concepto() {
        assert_eq!(
            linea_pago("EFE Pago de clientes  $2,023.80", "total de ingresos"),
            Some(("EFE Pago de clientes".into(), 202380))
        );
        assert_eq!(
            linea_pago(" 04 TARJETA BANCARIA    $305.00", "total de ingresos"),
            Some(("04 TARJETA BANCARIA".into(), 30500))
        );
        assert_eq!(linea_pago("  Total de Ingresos:    $702.00", "total de ingresos"), None);
    }

    #[test]
    fn total_en_caja_no_es_egreso() {
        assert_eq!(linea_pago("   Total en caja:     $1,000.00", "total de egresos"), None);
    }

    #[test]
    fn impuesto_pelado_no_confunde_tasas() {
        let vc = vec![
            "Impuesto 16%      :       $.00".to_string(),
            "Ventas gravadas   :       $.00".to_string(),
            "Impuesto          :       $.00".to_string(),
        ];
        assert_eq!(monto_anclado(&vc, "impuesto"), Some(0));
    }
}
