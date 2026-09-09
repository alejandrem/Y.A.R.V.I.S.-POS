// ============================================================
// parser_cortes — Importación histórica de cortes de caja X/Z.
//
// Los datasets mezclan tickets y cortes en la misma carpeta: cada
// .txt se CLASIFICA primero (`src-ia`) y lo que no es corte se omite
// (contado, no error). Idempotencia por SHA256 de contenido, igual
// que `catalogos_importados`: re-importar es seguro.
//
// `cortes_caja` NO se toca: es solo para operativos en vivo. Lo
// parseado va a `cortes_importados` (+ items) vía migración 0009.
// ============================================================

use crate::backventanas::auth::AuthState;
use crate::dinero::a_pesos;
use sqlx::SqlitePool;
use src_ia::parseador_de_cortes::{clasificar_archivo, parse_corte, ClaseArchivo, CorteParseado};

/// Vista previa de UN corte sin guardar (paso Revisión de la UI).
#[tauri::command]
pub async fn previsualizar_corte(
    auth: tauri::State<'_, AuthState>,
    path: String,
) -> Result<CorteParseado, String> {
    auth.require_admin()?;
    let safe = super::utils::sanitize_path(&path)?;
    let contenido = std::fs::read_to_string(safe).map_err(|e| e.to_string())?;
    parse_corte(&contenido)
}

/// Resumen de una importación de carpeta.
#[derive(serde::Serialize, Debug, PartialEq)]
pub struct ResumenCortes {
    pub archivos: usize,
    pub cortes_x: usize,
    pub cortes_z: usize,
    pub omitidos_no_corte: usize,
    pub omitidos_duplicados: usize,
    pub errores: Vec<String>,
}

/// Importa todos los .txt de una carpeta. Tolera encodings rotos
/// (lossy) y capa los errores reportados a 20 para no inundar la UI.
#[tauri::command]
pub async fn importar_carpeta_cortes(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    carpeta: String,
) -> Result<ResumenCortes, String> {
    auth.require_admin()?;
    importar_carpeta_cortes_impl(&state, &carpeta).await
}

pub async fn importar_carpeta_cortes_impl(pool: &SqlitePool, carpeta: &str) -> Result<ResumenCortes, String> {
    let dir = std::path::Path::new(carpeta);
    if !dir.is_dir() {
        return Err(format!("La ruta no es una carpeta: {carpeta}"));
    }
    let mut rutas: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("Error leyendo carpeta: {e}"))?
        .filter_map(|e| e.ok().map(|x| x.path()))
        .filter(|p| p.is_file() && p.extension().and_then(|x| x.to_str()).unwrap_or("").eq_ignore_ascii_case("txt"))
        .collect();
    rutas.sort();

    let mut r = ResumenCortes {
        archivos: rutas.len(),
        cortes_x: 0,
        cortes_z: 0,
        omitidos_no_corte: 0,
        omitidos_duplicados: 0,
        errores: Vec::new(),
    };

    for ruta in rutas {
        let nombre = ruta.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string();
        let bytes = match std::fs::read(&ruta) {
            Ok(b) => b,
            Err(e) => {
                if r.errores.len() < 20 {
                    r.errores.push(format!("{nombre}: no se pudo leer ({e})"));
                }
                continue;
            }
        };
        let contenido = String::from_utf8_lossy(&bytes);
        match clasificar_archivo(&contenido) {
            ClaseArchivo::NoEsCorte => r.omitidos_no_corte += 1,
            clase => {
                let hash = calcular_hash(&contenido);
                match corte_ya_importado(pool, &hash).await {
                    Ok(true) => r.omitidos_duplicados += 1,
                    Ok(false) => match parse_corte(&contenido) {
                        Err(e) => {
                            if r.errores.len() < 20 {
                                r.errores.push(format!("{nombre}: {e}"));
                            }
                        }
                        Ok(corte) => match guardar_corte(pool, &corte, &ruta.to_string_lossy(), &hash).await {
                            Err(e) => {
                                if r.errores.len() < 20 {
                                    r.errores.push(format!("{nombre}: {e}"));
                                }
                            }
                            Ok(()) => match clase {
                                ClaseArchivo::CorteX => r.cortes_x += 1,
                                ClaseArchivo::CorteZ => r.cortes_z += 1,
                                ClaseArchivo::NoEsCorte => {}
                            },
                        },
                    },
                    Err(e) => {
                        if r.errores.len() < 20 {
                            r.errores.push(format!("{nombre}: {e}"));
                        }
                    }
                }
            }
        }
    }
    Ok(r)
}

fn calcular_hash(contenido: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(contenido.as_bytes());
    format!("{:x}", h.finalize())
}

async fn corte_ya_importado(pool: &SqlitePool, hash: &str) -> Result<bool, String> {
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cortes_importados WHERE hash = ?")
        .bind(hash)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(n > 0)
}

/// Guarda corte + items en una transacción todo-o-nada. El parser ya
/// entrega centavos INTEGER: aquí no se multiplica nada a mano.
async fn guardar_corte(
    pool: &SqlitePool,
    corte: &CorteParseado,
    ruta: &str,
    hash: &str,
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let id: i64 = sqlx::query(
        "INSERT INTO cortes_importados
         (tipo, folio, estacion, cajero, empresa, moneda, fecha,
          total_ingresos, total_egresos, total_caja, total_ventas,
          ventas_gravadas, impuesto, ventas_no_gravadas, redondeos,
          ventas_credito, total_unidades, clientes_atendidos,
          ruta_archivo, hash)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(corte.tipo.etiqueta())
    .bind(corte.folio.as_deref())
    .bind(corte.estacion.as_deref())
    .bind(&corte.cajero)
    .bind(corte.empresa.as_deref())
    .bind(&corte.moneda)
    .bind(corte.fecha.as_deref())
    .bind(corte.total_ingresos)
    .bind(corte.total_egresos)
    .bind(corte.total_caja)
    .bind(corte.total_ventas)
    .bind(corte.ventas_gravadas)
    .bind(corte.impuesto)
    .bind(corte.ventas_no_gravadas)
    .bind(corte.redondeos)
    .bind(corte.ventas_credito)
    .bind(corte.total_unidades)
    .bind(corte.clientes_atendidos)
    .bind(ruta)
    .bind(hash)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?
    .last_insert_rowid();

    for it in &corte.items {
        sqlx::query(
            "INSERT INTO cortes_importados_items
             (corte_id, kind, nombre, cantidad, precio_unitario, subtotal)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(&it.kind)
        .bind(&it.nombre)
        .bind(it.cantidad)
        .bind(it.precio_unitario)
        .bind(it.subtotal)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

// ── Historial ────────────────────────────────────────────────

/// Renglón del historial (montos ya en pesos para el IPC).
#[derive(serde::Serialize, Debug)]
pub struct CorteImportadoRow {
    pub id: i64,
    pub tipo: String,
    pub folio: Option<String>,
    pub estacion: Option<String>,
    pub cajero: String,
    pub fecha: Option<String>,
    pub total_caja: f64,
    pub total_ventas: f64,
    pub clientes_atendidos: i64,
    pub verificado: bool,
}

#[tauri::command]
pub async fn get_cortes_importados(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<CorteImportadoRow>, String> {
    auth.require_admin()?;
    let limit = limit.unwrap_or(100).clamp(1, 500);
    let offset = offset.unwrap_or(0).max(0);
    let rows = sqlx::query(
        "SELECT id, tipo, folio, estacion, cajero, fecha, total_ingresos,
                total_egresos, total_caja, total_ventas, clientes_atendidos
         FROM cortes_importados ORDER BY fecha DESC, id DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    use sqlx::Row;
    Ok(rows
        .into_iter()
        .map(|r| {
            let ing: i64 = r.try_get("total_ingresos").unwrap_or(0);
            let egr: i64 = r.try_get("total_egresos").unwrap_or(0);
            let caja: i64 = r.try_get("total_caja").unwrap_or(0);
            let ventas: i64 = r.try_get("total_ventas").unwrap_or(0);
            CorteImportadoRow {
                id: r.try_get("id").unwrap_or(0),
                tipo: r.try_get("tipo").unwrap_or_default(),
                folio: r.try_get("folio").ok().flatten(),
                estacion: r.try_get("estacion").ok().flatten(),
                cajero: r.try_get("cajero").unwrap_or_default(),
                fecha: r.try_get("fecha").ok().flatten(),
                total_caja: a_pesos(caja),
                total_ventas: a_pesos(ventas),
                clientes_atendidos: r.try_get("clientes_atendidos").unwrap_or(0),
                // Chequeo rápido sin re-parsear: caja == ingresos − egresos.
                // La verificación completa (incluye Σ renglones) vive en
                // el detalle, que sí lee los items.
                verificado: caja == ing - egr,
            }
        })
        .collect())
}

/// Detalle completo de un corte importado.
#[derive(serde::Serialize, Debug)]
pub struct ItemImportado {
    pub kind: String,
    pub nombre: String,
    pub cantidad: Option<f64>,
    pub precio_unitario: f64,
    pub subtotal: f64,
}

#[derive(serde::Serialize, Debug)]
pub struct CorteImportadoDetalle {
    pub id: i64,
    pub tipo: String,
    pub folio: Option<String>,
    pub estacion: Option<String>,
    pub cajero: String,
    pub empresa: Option<String>,
    pub moneda: String,
    pub fecha: Option<String>,
    pub total_ingresos: f64,
    pub total_egresos: f64,
    pub total_caja: f64,
    pub total_ventas: f64,
    pub ventas_gravadas: f64,
    pub impuesto: f64,
    pub ventas_no_gravadas: f64,
    pub redondeos: f64,
    pub ventas_credito: f64,
    pub total_unidades: f64,
    pub clientes_atendidos: i64,
    pub caja_ok: bool,
    pub ventas_ok: bool,
    pub items: Vec<ItemImportado>,
}

#[tauri::command]
pub async fn get_corte_importado_detalle(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    corte_id: i64,
) -> Result<CorteImportadoDetalle, String> {
    auth.require_admin()?;
    use sqlx::Row;
    let r = sqlx::query("SELECT * FROM cortes_importados WHERE id = ?")
        .bind(corte_id)
        .fetch_optional(&*state)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Corte no encontrado".to_string())?;

    let get = |col: &str| -> i64 { r.try_get(col).unwrap_or(0) };
    let ing = get("total_ingresos");
    let egr = get("total_egresos");
    let caja = get("total_caja");
    let ventas = get("total_ventas");

    let filas = sqlx::query(
        "SELECT kind, nombre, cantidad, precio_unitario, subtotal
         FROM cortes_importados_items WHERE corte_id = ? ORDER BY id ASC",
    )
    .bind(corte_id)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let mut suma_vendible: i64 = 0;
    let items: Vec<ItemImportado> = filas
        .into_iter()
        .map(|f| {
            let kind: String = f.try_get("kind").unwrap_or_default();
            let sub: i64 = f.try_get("subtotal").unwrap_or(0);
            if kind == "ARTICULO" || kind == "TICKET" {
                suma_vendible += sub;
            }
            ItemImportado {
                kind,
                nombre: f.try_get("nombre").unwrap_or_default(),
                cantidad: f.try_get("cantidad").ok().flatten(),
                precio_unitario: a_pesos(f.try_get("precio_unitario").unwrap_or(0)),
                subtotal: a_pesos(sub),
            }
        })
        .collect();

    Ok(CorteImportadoDetalle {
        id: r.try_get("id").unwrap_or(0),
        tipo: r.try_get("tipo").unwrap_or_default(),
        folio: r.try_get("folio").ok().flatten(),
        estacion: r.try_get("estacion").ok().flatten(),
        cajero: r.try_get("cajero").unwrap_or_default(),
        empresa: r.try_get("empresa").ok().flatten(),
        moneda: r.try_get("moneda").unwrap_or_default(),
        fecha: r.try_get("fecha").ok().flatten(),
        total_ingresos: a_pesos(ing),
        total_egresos: a_pesos(egr),
        total_caja: a_pesos(caja),
        total_ventas: a_pesos(ventas),
        ventas_gravadas: a_pesos(get("ventas_gravadas")),
        impuesto: a_pesos(get("impuesto")),
        ventas_no_gravadas: a_pesos(get("ventas_no_gravadas")),
        redondeos: a_pesos(get("redondeos")),
        ventas_credito: a_pesos(get("ventas_credito")),
        total_unidades: r.try_get("total_unidades").unwrap_or(0.0),
        clientes_atendidos: r.try_get("clientes_atendidos").unwrap_or(0),
        caja_ok: caja == ing - egr,
        ventas_ok: ventas == suma_vendible,
        items,
    })
}
