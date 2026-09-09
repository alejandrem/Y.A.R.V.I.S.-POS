// ============================================================
// importacion — Lote de .txt clasificados a `cortes_importados`.
//
// Los datasets mezclan tickets y cortes: lo que no es corte se
// omite (contado, no error). Idempotencia por SHA256 de contenido,
// igual que `catalogos_importados`. Guardado todo-o-nada por corte.
// El parser ya entrega centavos INTEGER: aquí no se multiplica
// nada a mano.
// ============================================================

use crate::backventanas::auth::AuthState;
use sqlx::SqlitePool;
use src_ia::parseador_de_cortes::{clasificar_archivo, parse_corte, ClaseArchivo, CorteParseado};

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

/// Guarda corte + items en una transacción todo-o-nada.
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
