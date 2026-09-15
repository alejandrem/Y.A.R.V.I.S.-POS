// ============================================================
// impresion — Impresión térmica directa de cortes X/Z.
//
// `imprimir_corte` recalcula el corte desde su ventana guardada
// [fecha_apertura, fecha_cierre] (reimpresión fiel aunque haya
// ventas posteriores) y manda bytes ESC/POS al spooler Windows
// o a térmica en red. Sin diálogos: directo a la térmica.
// ============================================================

use super::comun::{
    productos_en_ventana, tickets_en_ventana, totales_en_ventana, ProductoAgregado, TicketResumen,
    TotalesVentana,
};
use crate::backventanas::auth::{AuthState, Role};
use crate::impresora::commands::Destino;
use crate::impresora::memoria::MemoriaDriver;
use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::{JustifyMode, Protocol};
use sqlx::{Row, SqlitePool};

/// Ancho útil de térmica 80mm en fuente A.
const COLS: usize = 48;
/// Tope de líneas por corte para no saturar el spooler.
const MAX_LINEAS_CORTE: usize = 500;

fn dinero(monto: f64) -> String {
    format!("${:.2}", monto.max(0.0))
}

/// Fila monoespaciada: izquierda recortada + importe a la derecha.
fn fila(importe: &str, izquierda: &str) -> String {
    let der = importe.chars().count();
    let max_izq = COLS.saturating_sub(der + 1);
    let mut izq: String = izquierda.chars().take(max_izq).collect();
    while izq.chars().count() + der < COLS {
        izq.push(' ');
    }
    format!("{izq}{importe}")
}

fn linea_corte() -> String {
    "-".repeat(COLS)
}

/// "YYYY-MM-DD HH:MM:SS" -> "DD/MM/YYYY HH:MM".
fn fmt_fecha(s: &str) -> String {
    let p: Vec<&str> = s.split(&['-', ' ', ':'][..]).collect();
    if p.len() >= 5 {
        format!("{}/{}/{} {}:{}", p[2], p[1], p[0], p[3], p[4])
    } else {
        s.to_string()
    }
}

fn fmt_cant(c: f64) -> String {
    if c.fract() == 0.0 {
        format!("{}", c as i64)
    } else {
        format!("{c}")
    }
}

struct DatosCorte {
    tipo: String,
    cajero_id: i64,
    cajero_nombre: String,
    tienda: String,
    apertura: String,
    cierre: String,
    numero: i64,
}

async fn cargar_datos(pool: &SqlitePool, corte_id: i64) -> Result<DatosCorte, String> {
    let row = sqlx::query(
        r#"SELECT c.tipo_corte, c.usuario_id, c.fecha_apertura, c.fecha_cierre,
                  COALESCE(u.nombre, 'GENERAL') AS cajero,
                  COALESCE((SELECT tienda FROM usuarios WHERE rol = 'admin' LIMIT 1), 'MI TIENDA') AS tienda
           FROM cortes_caja c
           LEFT JOIN usuarios u ON u.id = c.usuario_id
           WHERE c.id = ?"#,
    )
    .bind(corte_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Corte no encontrado".to_string())?;

    let cierre: Option<String> = row.try_get("fecha_cierre").ok().flatten();
    Ok(DatosCorte {
        tipo: row.get("tipo_corte"),
        cajero_id: row.get("usuario_id"),
        cajero_nombre: row.get("cajero"),
        tienda: row.get("tienda"),
        apertura: row.get("fecha_apertura"),
        cierre: cierre.unwrap_or_else(super::comun::ahora_str),
        numero: corte_id,
    })
}

fn encabezado(d: &DatosCorte) -> Vec<String> {
    vec![
        format!("*** CORTE {} EN MONEDA:MXN ***", d.tipo),
        d.tienda.clone(),
        format!("Cajero: {}", d.cajero_nombre.to_uppercase()),
        linea_corte(),
        format!("Corte {} #{}", d.tipo, d.numero),
        format!("{} - {}", fmt_fecha(&d.apertura), fmt_fecha(&d.cierre)),
        linea_corte(),
        "** VENTAS DEL TURNO **".into(),
    ]
}

fn pie_totales(t: &TotalesVentana) -> Vec<String> {
    vec![
        linea_corte(),
        fila(&dinero(t.total_ventas), "Total ventas"),
        fila(&dinero(t.total_efectivo), "Efectivo"),
        fila(&dinero(t.total_tarjeta), "Tarjeta"),
        fila(&dinero(t.total_transferencia), "Transferencia"),
        linea_corte(),
        format!("Tickets: {}", t.num_tickets),
        linea_corte(),
    ]
}

fn a_bytes(titulo_centrado: &[String], cuerpo_izq: &[String]) -> Result<Vec<u8>, String> {
    let total = titulo_centrado.len() + cuerpo_izq.len();
    if total > MAX_LINEAS_CORTE {
        return Err(format!("Corte demasiado largo ({total} lineas)."));
    }
    let driver = MemoriaDriver::default();
    let sonda = driver.clone();
    let mut p = Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()));
    p.init().map_err(|e| format!("INIT termica: {e}"))?;
    p.justify(JustifyMode::CENTER)
        .map_err(|e| format!("Corte: {e}"))?;
    p.bold(true).map_err(|e| format!("Corte: {e}"))?;
    for l in titulo_centrado {
        p.writeln(l).map_err(|e| format!("Corte: {e}"))?;
    }
    p.bold(false).map_err(|e| format!("Corte: {e}"))?;
    p.justify(JustifyMode::LEFT)
        .map_err(|e| format!("Corte: {e}"))?;
    for l in cuerpo_izq {
        p.writeln(l).map_err(|e| format!("Corte: {e}"))?;
    }
    p.justify(JustifyMode::CENTER)
        .map_err(|e| format!("Corte: {e}"))?;
    p.feed().map_err(|e| format!("Corte: {e}"))?;
    p.print_cut().map_err(|e| format!("Corte: {e}"))?;
    let bytes = sonda.bytes();
    if bytes.len() > 512 * 1024 {
        return Err("Corte demasiado grande (limite 512KB).".into());
    }
    Ok(bytes)
}

async fn construir_corte_x(pool: &SqlitePool, d: &DatosCorte) -> Result<Vec<u8>, String> {
    let tickets: Vec<TicketResumen> =
        tickets_en_ventana(pool, d.cajero_id, &d.apertura, &d.cierre).await?;
    let totales: TotalesVentana =
        totales_en_ventana(pool, d.cajero_id, &d.apertura, &d.cierre).await?;

    let mut cuerpo: Vec<String> = tickets
        .iter()
        .map(|t| fila(&dinero(t.total), &t.folio))
        .collect();
    cuerpo.extend(pie_totales(&totales));
    cuerpo.push("X informativo: no cierra tu turno".into());
    cuerpo.push(linea_corte());

    let head = encabezado(d);
    a_bytes(&head[..3], &[head[3..].to_vec(), cuerpo].concat())
}

async fn construir_corte_z(pool: &SqlitePool, d: &DatosCorte) -> Result<Vec<u8>, String> {
    let productos: Vec<ProductoAgregado> =
        productos_en_ventana(pool, d.cajero_id, &d.apertura, &d.cierre).await?;
    let totales: TotalesVentana =
        totales_en_ventana(pool, d.cajero_id, &d.apertura, &d.cierre).await?;

    let mut cuerpo: Vec<String> = productos
        .iter()
        .map(|pr| {
            fila(
                &dinero(pr.monto),
                &format!("{} {}x", pr.producto_nombre, fmt_cant(pr.cantidad)),
            )
        })
        .collect();
    cuerpo.extend(pie_totales(&totales));
    cuerpo.push("*** TURNO CERRADO ***".into());
    cuerpo.push("Contador reiniciado a $0.00".into());
    cuerpo.push(linea_corte());

    let head = encabezado(d);
    a_bytes(&head[..3], &[head[3..].to_vec(), cuerpo].concat())
}

fn enviar_red(ip: &str, puerto: u16, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;
    let dir = format!("{ip}:{puerto}")
        .to_socket_addrs()
        .map_err(|e| format!("IP/host invalido '{ip}': {e}"))?
        .next()
        .ok_or_else(|| format!("No se resolvio '{ip}'."))?;
    let mut stream = TcpStream::connect_timeout(&dir, Duration::from_secs(5))
        .map_err(|e| format!("Sin conexion a {ip}:{puerto} ({e})."))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("No se pudo configurar timeout: {e}"))?;
    stream
        .write_all(bytes)
        .map_err(|e| format!("Fallo enviando a {ip}:{puerto}: {e}"))?;
    stream.flush().map_err(|e| format!("Fallo flush: {e}"))?;
    Ok(())
}

/// Imprime un corte X/Z ya guardado en la térmica (spooler o red).
/// El empleado solo puede imprimir sus propios cortes; el admin, todos.
#[tauri::command]
pub async fn imprimir_corte(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    destino: Destino,
    corte_id: i64,
) -> Result<String, String> {
    let session = auth.require_operator()?;
    let datos = cargar_datos(&*state, corte_id).await?;
    if session.role == Role::Employee && datos.cajero_id != session.user_id {
        return Err("Solo puedes imprimir tus propios cortes.".into());
    }
    if datos.tipo != "X" && datos.tipo != "Z" {
        return Err("Tipo de corte desconocido.".into());
    }

    let bytes = if datos.tipo == "X" {
        construir_corte_x(&*state, &datos).await?
    } else {
        construir_corte_z(&*state, &datos).await?
    };
    let n = bytes.len();
    let etiqueta = format!("CORTE-{}-#{}", datos.tipo, datos.numero);

    match destino {
        Destino::Spooler { nombre } => {
            let nombre_c = nombre.trim().to_string();
            if nombre_c.is_empty() {
                return Err("Elige una impresora instalada.".into());
            }
            tokio::task::spawn_blocking(move || {
                crate::impresora::enviar_bytes_raw(&nombre_c, &bytes)
            })
            .await
            .map_err(|e| format!("Fallo interno de impresion: {e}"))??;
            Ok(format!("{etiqueta} enviado al spooler ({n} bytes)."))
        }
        Destino::Red { ip, puerto } => {
            let ip_c = ip.trim().to_string();
            if ip_c.is_empty() || puerto == 0 {
                return Err("IP/puerto de red invalidos.".into());
            }
            tokio::task::spawn_blocking(move || enviar_red(&ip_c, puerto, &bytes))
                .await
                .map_err(|e| format!("Fallo interno de impresion: {e}"))??;
            Ok(format!("{etiqueta} enviado a {ip}:{puerto} ({n} bytes)."))
        }
    }
}
