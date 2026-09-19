// ============================================================
// verde — Auto-asignacion 100% segura (issue #10).
//
// Solo entra lo INDISCUTIBLE. Un falso positivo = cobrar mal, asi
// que el criterio es conservador a proposito. Orden exacto:
//   1. ean normalizado + checksum valido (ean.rs).
//   2. nombre ticket normalizado (mismo `normalizar` del vinculador)
//      + base sin presentacion (presentacion.rs).
//   3. Bloqueo marca + cantidad + unidad (la medida manda).
//   4. Nombre base EXACTO + ean libre + producto sin codigo distinto.
//   5. INSERT del vinculo origen='auto-verde' para auditoria.
//
// Anti-colisiones:
//   * ean reclamado por otro producto -> revision, jamas se pisa.
//   * producto con codigo distinto -> revision, no se sobrescribe.
//   * ean con historial en 2 productos -> cola de conflicto.
// ============================================================

use super::ean::validar_ean;
use super::presentacion::{
    extraer_presentacion, misma_presentacion, presentacion_de_catalogo, quitar_presentacion,
};
use crate::backventanas::auth::AuthState;
use crate::backventanas::codigos_barras::semaforo_amarillo::sugerir_impl;
use crate::backventanas::codigos_barras::{
    normalizar_codigo_obligatorio, validar_codigo_barras,
};
use tauri::Manager;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

// ---------- Tipos que viajan por Tauri ----------

/// Fila del dataset real (`dataset/*.csv`, header
/// ean,nombre,marca,cantidad,unidad,categoria). El front lee el CSV
/// y manda el lote; el backend NO lee archivos (portable).
#[derive(Debug, Clone, Deserialize)]
pub struct CatalogoRow {
    pub ean: String,
    pub nombre: String,
    pub marca: Option<String>,
    pub cantidad: f64,
    pub unidad: String,
    pub categoria: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResumenImportCatalogo {
    pub catalogo_upserts: usize,
    pub verde_asignados: usize,
    pub sin_match: usize,
    pub conflictos: usize,
    /// Pendientes NUEVOS que el lote derivó al semáforo (amarillo /
    /// rojo / conflicto) para administración manual. Lo ya existente
    /// o resuelto no se toca ni se cuenta aquí.
    pub pendientes_nuevos: usize,
    pub errores: Vec<String>,
}

/// Veredicto de UN intento verde (siempre explicito, nunca silencioso).
#[derive(Debug, Clone, Serialize)]
pub struct VeredictoVerde {
    pub asignado: bool,
    pub motivo: String,
}

/// Contadores para la pestaña Códigos del front (issue #13).
#[derive(Debug, Clone, Serialize)]
pub struct ConteosVerde {
    pub verdes_hoy: i64,
    pub total_vinculos: i64,
}

// ---------- Helpers puros ----------

fn normalizar_marca(m: Option<&str>) -> Option<String> {
    let t = m?.trim().to_lowercase();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

/// Si alguno no trae marca, NO bloquea (dato ausente != marca distinta).
fn marcas_iguales(a: Option<&str>, b: Option<&str>) -> bool {
    match (normalizar_marca(a), normalizar_marca(b)) {
        (Some(x), Some(y)) => x == y,
        _ => true,
    }
}

// ---------- Chequeos contra la DB ----------

async fn ean_ocupado_por_otro(
    pool: &SqlitePool,
    ean: &str,
    producto_id: i64,
) -> Result<bool, String> {
    let otro: Option<i64> = sqlx::query_scalar("SELECT id FROM productos WHERE codigo_barras = ?")
        .bind(ean)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(otro.is_some_and(|id| id != producto_id))
}

async fn producto_con_codigo_distinto(
    pool: &SqlitePool,
    producto_id: i64,
    ean: &str,
) -> Result<bool, String> {
    let actual: Option<String> =
        sqlx::query_scalar("SELECT codigo_barras FROM productos WHERE id = ?")
            .bind(producto_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .flatten();
    Ok(actual.is_some_and(|c| c != ean))
}

async fn ean_en_conflicto(pool: &SqlitePool, ean: &str, producto_id: i64) -> Result<bool, String> {
    let rows: Vec<i64> = sqlx::query_scalar(
        "SELECT DISTINCT producto_id FROM vinculos_codigos WHERE ean = ?",
    )
    .bind(ean)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows.iter().any(|id| *id != producto_id))
}

// ---------- Nucleo: UN intento verde ----------

/// Intenta asignar `ean` a `producto_id` si TODO cuadra.
/// `nombre_ticket` es el texto crudo del ticket ("cocacola 600").
pub async fn intentar_verde_impl(
    pool: &SqlitePool,
    nombre_ticket: &str,
    marca_ticket: Option<&str>,
    ean_crudo: &str,
    producto_id: i64,
    confirmado_por: Option<i64>,
) -> Result<VeredictoVerde, String> {
    // 1. ean normalizado + valido + checksum.
    let ean = normalizar_codigo_obligatorio(ean_crudo)
        .ok_or_else(|| "Código de barras vacío.".to_string())?;
    validar_codigo_barras(&Some(ean.clone()))?;
    if !validar_ean(&ean) {
        return Ok(VeredictoVerde { asignado: false, motivo: "ean con dígito verificador inválido".into() });
    }
    // 2. Anti-colisiones ANTES de comparar nombres (barato primero).
    if ean_ocupado_por_otro(pool, &ean, producto_id).await? {
        return Ok(VeredictoVerde { asignado: false, motivo: "ean reclamado por otro producto".into() });
    }
    if producto_con_codigo_distinto(pool, producto_id, &ean).await? {
        return Ok(VeredictoVerde { asignado: false, motivo: "el producto ya tiene otro código".into() });
    }
    if ean_en_conflicto(pool, &ean, producto_id).await? {
        return Ok(VeredictoVerde { asignado: false, motivo: "ean en conflicto (2 productos lo reclaman)".into() });
    }
    // 3. Nombres base exactos (misma normalizacion que el vinculador).
    let ticket_norm = src_ia::embeddings::normalizar(nombre_ticket);
    if ticket_norm.is_empty() {
        return Ok(VeredictoVerde { asignado: false, motivo: "nombre de ticket vacío".into() });
    }
    let base_ticket = quitar_presentacion(&ticket_norm);
    let pres_ticket = extraer_presentacion(&ticket_norm);
    let (p_nombre, p_marca, p_cant, p_unidad): (String, Option<String>, Option<f64>, Option<String>) =
        sqlx::query_as(
            "SELECT nombre, marca, cantidad_presentacion, unidad_presentacion FROM productos WHERE id = ?",
        )
        .bind(producto_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Producto no existe.".to_string())?;
    let base_producto = src_ia::embeddings::normalizar(&p_nombre);
    if base_ticket != base_producto {
        return Ok(VeredictoVerde { asignado: false, motivo: "nombre base no exacto (va a amarillo/rojo)".into() });
    }
    // 4. Bloqueo marca + presentacion (la medida manda).
    if !marcas_iguales(marca_ticket, p_marca.as_deref()) {
        return Ok(VeredictoVerde { asignado: false, motivo: "marca distinta".into() });
    }
    let pres_producto = match (p_cant, p_unidad) {
        (Some(c), Some(u)) => presentacion_de_catalogo(c, &u),
        _ => None,
    };
    match (pres_ticket, pres_producto) {
        (Some(t), Some(p)) if misma_presentacion(&t, &p) => {}
        _ => {
            return Ok(VeredictoVerde { asignado: false, motivo: "presentación distinta o ausente".into() });
        }
    }
    // 5. TODO cuadra: asigna + audita (idempotente por OR IGNORE).
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("UPDATE productos SET codigo_barras = ? WHERE id = ? AND codigo_barras IS NULL")
        .bind(&ean)
        .bind(producto_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query(
        "INSERT OR IGNORE INTO vinculos_codigos (ean, producto_id, nombre_ticket_crudo, nombre_ticket_norm, origen, score, confirmado_por) VALUES (?, ?, ?, ?, 'auto-verde', 1.0, ?)",
    )
    .bind(&ean)
    .bind(producto_id)
    .bind(nombre_ticket)
    .bind(&ticket_norm)
    .bind(confirmado_por)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(VeredictoVerde { asignado: true, motivo: "auto-verde asignado".into() })
}

// ---------- Lote: importar dataset + auto-asignar ----------

/// Embudo del lote al semáforo: lo que no logró verde entra a la cola
/// (amarillo / rojo / conflicto) por el MISMO `sugerir` del pitazo en
/// caja — el admin lo administra igual, venga de donde venga.
///
/// Si el pendiente ya existe (re-import o ya resuelto), no se toca:
/// lo resuelto queda en 'resuelto' y lo pendiente sigue su flujo.
/// Devuelve true si se creó un pendiente nuevo.
async fn derivar_a_semaforo(
    pool: &SqlitePool,
    ticket_txt: &str,
    marca: Option<&str>,
    ean: &str,
) -> Result<bool, String> {
    let nombre_norm = src_ia::embeddings::normalizar(ticket_txt);
    let existe: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pendientes_codigos WHERE nombre_norm = ? AND ean = ?",
    )
    .bind(&nombre_norm)
    .bind(ean)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    if existe > 0 {
        return Ok(false);
    }
    sugerir_impl(pool, ticket_txt, marca, Some(ean)).await?;
    Ok(true)
}

/// Guarda el dataset en `catalogo_barras` (upsert idempotente) y por
/// cada fila busca UN producto gemelo (mismo nombre_norm + misma
/// presentacion + misma marca + sin codigo). 0 gemelos = sin_match,
/// 1 = intento verde, 2+ = conflicto (a revision, jamas auto).
pub async fn importar_catalogo_barras_impl(
    pool: &SqlitePool,
    filas: &[CatalogoRow],
    confirmado_por: Option<i64>,
) -> Result<ResumenImportCatalogo, String> {
    // Cache de productos para no hacer 308 selects por lote.
    let prods: Vec<(i64, String, Option<String>, Option<f64>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT id, nombre, marca, cantidad_presentacion, unidad_presentacion, codigo_barras FROM productos",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    let mut res = ResumenImportCatalogo {
        catalogo_upserts: 0,
        verde_asignados: 0,
        sin_match: 0,
        conflictos: 0,
        pendientes_nuevos: 0,
        errores: vec![],
    };
    for f in filas {
        let ean = match normalizar_codigo_obligatorio(&f.ean) {
            Some(e) => e,
            None => {
                res.errores.push(format!("ean vacío para '{}'", f.nombre));
                continue;
            }
        };
        if let Err(m) = validar_codigo_barras(&Some(ean.clone())) {
            res.errores.push(format!("'{}': {m}", f.nombre));
            continue;
        }
        if !validar_ean(&ean) {
            res.errores.push(format!("'{}': ean {ean} con dígito inválido", f.nombre));
            continue;
        }
        let unidad_canon = f.unidad.trim().to_ascii_lowercase();
        if !["ml", "l", "g", "kg", "pzs"].contains(&unidad_canon.as_str()) {
            res.errores.push(format!("'{}': unidad '{}' fuera de catálogo", f.nombre, f.unidad));
            continue;
        }
        let nombre_norm = src_ia::embeddings::normalizar(&f.nombre);
        sqlx::query(
            "INSERT INTO catalogo_barras (ean, nombre, nombre_norm, marca, cantidad, unidad, categoria) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT(ean) DO UPDATE SET nombre = excluded.nombre, nombre_norm = excluded.nombre_norm, marca = excluded.marca, cantidad = excluded.cantidad, unidad = excluded.unidad, categoria = excluded.categoria",
        )
        .bind(&ean)
        .bind(&f.nombre)
        .bind(&nombre_norm)
        .bind(&f.marca)
        .bind(f.cantidad)
        .bind(&unidad_canon)
        .bind(&f.categoria)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        res.catalogo_upserts += 1;

        // Gemelos en productos (mismo nombre_norm + presentacion + marca).
        let pres_cat = presentacion_de_catalogo(f.cantidad, &unidad_canon);
        let mut gemelos = vec![];
        for (id, nombre, marca, cant, unidad, codigo) in &prods {
            if codigo.is_some() {
                continue; // con codigo distinto/igual: no se pisa
            }
            if src_ia::embeddings::normalizar(nombre) != nombre_norm {
                continue;
            }
            if !marcas_iguales(f.marca.as_deref(), marca.as_deref()) {
                continue;
            }
            let pres_prod = match (cant, unidad) {
                (Some(c), Some(u)) => presentacion_de_catalogo(*c, u),
                _ => None,
            };
            match (&pres_cat, pres_prod) {
                (Some(a), Some(b)) if misma_presentacion(a, &b) => gemelos.push(*id),
                _ => continue,
            }
        }
        // El `nombre` del dataset viene SIN presentacion
        // (dataset/README.md), pero el verde compara texto de
        // ticket (que SI la trae: "cocacola 600"). Se reconstruye
        // el texto-ticket para el intento y para el semáforo; el
        // vinculo guarda ese texto, que es justo lo que pitara la
        // tienda despues.
        let cant_txt = if f.cantidad.fract() == 0.0 {
            format!("{}", f.cantidad as i64)
        } else {
            format!("{}", f.cantidad)
        };
        let ticket_txt = format!("{} {}{}", f.nombre, cant_txt, unidad_canon);
        match gemelos.len() {
            0 => {
                res.sin_match += 1;
                if derivar_a_semaforo(pool, &ticket_txt, f.marca.as_deref(), &ean).await? {
                    res.pendientes_nuevos += 1;
                }
            }
            1 => {
                let v = intentar_verde_impl(pool, &ticket_txt, f.marca.as_deref(), &ean, gemelos[0], confirmado_por).await?;
                if v.asignado {
                    res.verde_asignados += 1;
                } else if v.motivo.contains("conflicto") || v.motivo.contains("reclamado") {
                    res.conflictos += 1;
                    if derivar_a_semaforo(pool, &ticket_txt, f.marca.as_deref(), &ean).await? {
                        res.pendientes_nuevos += 1;
                    }
                } else {
                    res.sin_match += 1;
                    if derivar_a_semaforo(pool, &ticket_txt, f.marca.as_deref(), &ean).await? {
                        res.pendientes_nuevos += 1;
                    }
                }
            }
            _ => {
                res.conflictos += 1;
                if derivar_a_semaforo(pool, &ticket_txt, f.marca.as_deref(), &ean).await? {
                    res.pendientes_nuevos += 1;
                }
            }
        }
    }
    Ok(res)
}

// ---------- Comandos Tauri (solo admin: escriben inventario) ----------

#[tauri::command]
pub async fn verde_autoasignar(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    nombre_ticket: String,
    marca_ticket: Option<String>,
    ean: String,
    producto_id: i64,
) -> Result<VeredictoVerde, String> {
    let ses = auth.require_admin()?;
    intentar_verde_impl(&*state, &nombre_ticket, marca_ticket.as_deref(), &ean, producto_id, Some(ses.user_id)).await
}

#[tauri::command]
pub async fn verde_importar_catalogo(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    filas: Vec<CatalogoRow>,
) -> Result<ResumenImportCatalogo, String> {
    let ses = auth.require_admin()?;
    importar_catalogo_barras_impl(&*state, &filas, Some(ses.user_id)).await
}

/// Catálogos incluidos en el instalador (`tauri.conf.json` resources →
/// `datasets/*.csv`; en dev viven en `src-tauri/datasets/`). Son la
/// copia de `dataset/` de la raíz del repo.
const CATALOGOS_INCLUIDOS: &[&str] = &["abarrotes", "botana", "lacteos", "refrescos"];

/// Parsea CSV con formato dataset (`ean,nombre,marca,cantidad,unidad,
/// categoria`). Puro y testeable: ignora BOM, header, líneas vacías y
/// filas sin ean/nombre o con cantidad no numérica (esas las reporta
/// `importar_catalogo_barras_impl` en `errores`). Los campos no traen
/// comas por regla del dataset, así que el split simple basta.
pub fn parsear_csv_barras(texto: &str, categoria_defecto: &str) -> Vec<CatalogoRow> {
    let mut filas = vec![];
    for linea in texto.replace('\u{FEFF}', "").lines() {
        let linea = linea.trim();
        if linea.is_empty() || linea.to_lowercase().starts_with("ean,") {
            continue;
        }
        let partes: Vec<&str> = linea.split(',').map(str::trim).collect();
        let ean = partes.first().copied().unwrap_or("");
        let nombre = partes.get(1).copied().unwrap_or("");
        if ean.is_empty() || nombre.is_empty() {
            continue;
        }
        let cantidad: f64 = match partes.get(3).copied().unwrap_or("").parse() {
            Ok(c) => c,
            Err(_) => continue,
        };
        let marca_txt = partes.get(2).copied().unwrap_or("");
        let categoria_txt = partes.get(5).copied().unwrap_or("");
        filas.push(CatalogoRow {
            ean: ean.into(),
            nombre: nombre.into(),
            marca: if marca_txt.is_empty() { None } else { Some(marca_txt.into()) },
            cantidad,
            unidad: partes.get(4).copied().unwrap_or("").into(),
            categoria: Some(if categoria_txt.is_empty() {
                categoria_defecto.into()
            } else {
                categoria_txt.into()
            }),
        });
    }
    filas
}

/// Comando Tauri: carga los 4 CSV incluidos en el `.exe` y los cruza
/// con el inventario (mismo embudo que `verde_importar_catalogo`).
/// Un clic, sin buscar archivos. Solo admin (escribe inventario).
#[tauri::command]
pub async fn cargar_catalogo_incluido(
    app: tauri::AppHandle,
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<ResumenImportCatalogo, String> {
    let ses = auth.require_admin()?;
    let base = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("datasets");
    let mut filas = vec![];
    let mut faltantes = vec![];
    for nombre in CATALOGOS_INCLUIDOS {
        match std::fs::read_to_string(base.join(format!("{nombre}.csv"))) {
            Ok(texto) => filas.extend(parsear_csv_barras(&texto, nombre)),
            Err(e) => faltantes.push(format!("{nombre}.csv: {e}")),
        }
    }
    if filas.is_empty() {
        return Err(if faltantes.is_empty() {
            "Los catálogos incluidos no traen filas válidas.".into()
        } else {
            format!("No se pudieron leer los catálogos incluidos: {}", faltantes.join(" · "))
        });
    }
    let mut res = importar_catalogo_barras_impl(&*state, &filas, Some(ses.user_id)).await?;
    res.errores = faltantes.into_iter().chain(res.errores).collect();
    Ok(res)
}

/// Auto-verdes de hoy + vinculos totales (contadores del front).
/// Solo lectura: operario (admin o empleado) puede verlos.
pub async fn contar_verde_impl(pool: &SqlitePool) -> Result<ConteosVerde, String> {
    let verdes_hoy: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM vinculos_codigos WHERE origen = 'auto-verde' AND date(creado_en) = date('now', 'localtime')",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    let total_vinculos: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vinculos_codigos")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(ConteosVerde { verdes_hoy, total_vinculos })
}

#[tauri::command]
pub async fn verde_contar_hoy(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<ConteosVerde, String> {
    auth.require_operator()?;
    contar_verde_impl(&*state).await
}
