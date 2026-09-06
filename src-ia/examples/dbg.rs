// Herramienta de diagnóstico del detector contra carpetas reales de tickets.
//
// Uso:
//   cargo run --example dbg -- "/ruta/a/carpeta/de/tickets"
//
// Por cada .txt muestra: segmentos detectados (folio/fecha/líneas) y al
// final la detección estadística global del mapeo, igual que hace el
// backend (`detectar_mapeo_estadistico` en parser_txt.rs).
//
// No es parte del CI: es para probar formatos nuevos a mano antes de
// tocar el detector.

use std::fs;
use std::path::Path;

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Uso: cargo run --example dbg -- <carpeta_con_txt>");
        std::process::exit(1);
    });
    let mut archivos: Vec<_> = fs::read_dir(Path::new(&dir))
        .unwrap_or_else(|e| {
            eprintln!("No se pudo leer {dir}: {e}");
            std::process::exit(1);
        })
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("txt"))
        .collect();
    archivos.sort();
    if archivos.is_empty() {
        println!("Sin .txt en {dir}");
        return;
    }

    // Mismo muestreo que el backend: hasta 15 archivos espaciados,
    // 60 líneas por archivo, tope 900 líneas.
    let paso = (archivos.len() / 15).max(1);
    let mut lineas: Vec<String> = Vec::new();
    let mut contados = 0usize;
    let mut sin_folio = 0usize;
    for a in archivos.iter().step_by(paso) {
        let Ok(bytes) = fs::read(a) else { continue };
        let texto = String::from_utf8_lossy(&bytes).into_owned();
        let segs = src_ia::cerebro::analizador_tickets::segmentar(&texto);
        println!("--- {}: {} segmento(s)", a.display(), segs.len());
        for s in &segs {
            if s.folio.is_none() {
                sin_folio += 1;
            }
            println!(
                "    ticket#{} folio={:?} clave={} fecha={:?} lineas={} pago={}",
                s.index,
                s.folio,
                s.clave(),
                s.fecha_hora,
                s.lineas.len(),
                s.metodo_pago,
            );
        }
        lineas.extend(texto.lines().take(60).map(str::to_string));
        contados += 1;
        if lineas.len() >= 900 || contados >= 15 {
            break;
        }
    }
    let refs: Vec<&str> = lineas.iter().map(String::as_str).collect();
    match src_ia::cerebro::analizador_tickets::detectar_mapeo(&refs) {
        Some(d) => println!(
            "\nDETECTADO: {:?}\nconfianza={:.3} lineas={} validas={} archivos={contados} segmentos_sin_folio={sin_folio}",
            d.mapeo, d.confianza, d.lineas_evaluadas, d.lineas_validas
        ),
        None => println!(
            "\nSIN DETECCION ({} lineas de {contados} archivos, segmentos_sin_folio={sin_folio})",
            refs.len()
        ),
    }
}
