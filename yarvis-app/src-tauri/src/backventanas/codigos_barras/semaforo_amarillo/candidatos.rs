// ============================================================
// candidatos — Top-5 con bloqueo DURO de presentacion.
//
// Orden del issue #11: normalizar + presentacion canonica, BLOQUEO
// (medida distinta = score 0, ni se sugiere), veto a ean en
// conflicto, aprendizaje previo y scoring. Jamas auto-asigna:
// solo ordena para que el humano confirme.
//
// Los tickets reales abrevian ("cocacola 600" sin unidad): el numero
// suelto vale contra la cantidad del producto (600 == 600ml), pero
// jamas contra otra medida (600 != 3L). El humano confirma igual.
// ============================================================

use super::score::puntuar;
use super::tipos::Candidato;
use super::umbral::decide;
use crate::backventanas::codigos_barras::semaforo_verde::{
    extraer_presentacion, misma_presentacion, presentacion_de_catalogo,
    quitar_presentacion, Presentacion,
};
use sqlx::SqlitePool;

struct Fila {
    id: i64,
    nombre: String,
    marca: Option<String>,
    cantidad: Option<f64>,
    unidad: Option<String>,
}

/// Medida del ticket: con unidad ("600ml") o numero suelto ("600").
enum Medida {
    Unidad(Presentacion),
    Suelta(f64),
}

struct Ctx {
    base: String,
    medida: Medida,
    marca: Option<String>,
}

/// Ultimo token puramente numerico (de atras hacia adelante) y la
/// base sin el. "cocacola 600" -> (600, "cocacola").
fn numero_suelto(norm: &str) -> Option<(f64, String)> {
    let mut toks: Vec<&str> = norm.split_whitespace().collect();
    let pos = toks.iter().rposition(|t| t.replace(',', ".").parse::<f64>().is_ok())?;
    let n: f64 = toks[pos].replace(',', ".").parse().ok()?;
    toks.remove(pos);
    Some((n, toks.join(" ")))
}

/// Ticket listo, o None si no trae medida alguna (sin medida no hay
/// bloqueo posible: todo se veta, cae a rojo).
fn armar_ctx(nombre_crudo: &str, marca: Option<&str>) -> Option<Ctx> {
    let norm = src_ia::embeddings::normalizar(nombre_crudo);
    let marca = marca.map(|s| s.to_string());
    if let Some(p) = extraer_presentacion(&norm) {
        let base = quitar_presentacion(&norm);
        return Some(Ctx { base, medida: Medida::Unidad(p), marca });
    }
    let (n, base) = numero_suelto(&norm)?;
    Some(Ctx { base, medida: Medida::Suelta(n), marca })
}

async fn cargar_filas(pool: &SqlitePool) -> Result<Vec<Fila>, String> {
    sqlx::query_as::<_, (i64, String, Option<String>, Option<f64>, Option<String>)>(
        "SELECT id, nombre, marca, cantidad_presentacion, unidad_presentacion FROM productos",
    )
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(id, nombre, marca, cantidad, unidad)| Fila { id, nombre, marca, cantidad, unidad })
            .collect()
    })
    .map_err(|e| e.to_string())
}

fn pres_de_fila(f: &Fila) -> Option<Presentacion> {
    presentacion_de_catalogo(f.cantidad?, f.unidad.as_deref()?)
}

/// ¿La medida del ticket cuadra con la fila? Suelta solo empata con
/// su propio numero (600 == 600ml, 600 != 3L).
fn pasa_medida(medida: &Medida, cantidad_orig: f64, pres: &Presentacion) -> bool {
    match medida {
        Medida::Unidad(p) => misma_presentacion(p, pres),
        Medida::Suelta(n) => (n - cantidad_orig).abs() < 1e-6 || (n - pres.cantidad_base).abs() < 1e-6,
    }
}

/// Puntua UNA fila o la veta (None): sin presentacion registrada
/// o medida distinta, ni se sugiere.
fn puntuar_fila(ctx: &Ctx, f: &Fila) -> Option<Candidato> {
    let cantidad_orig = f.cantidad?;
    let pres = pres_de_fila(f)?;
    if !pasa_medida(&ctx.medida, cantidad_orig, &pres) {
        return None;
    }
    let norm = src_ia::embeddings::normalizar(&f.nombre);
    let base = quitar_presentacion(&norm);
    let score = puntuar(&ctx.base, &base, ctx.marca.as_deref(), f.marca.as_deref());
    Some(Candidato { producto_id: f.id, nombre: f.nombre.clone(), score, origen: "scoring".into() })
}

/// ¿2 productos reclaman el mismo ean? Entonces no se sugiere.
pub async fn hay_conflicto_ean(pool: &SqlitePool, ean: &str) -> Result<bool, String> {
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT producto_id) FROM vinculos_codigos WHERE ean = ?",
    )
    .bind(ean)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(n >= 2)
}

/// Busca vinculo previo por nombre exacto y por base (los vinculos
/// viejos se guardaron con o sin presentacion en el texto).
async fn vinculo_previo(pool: &SqlitePool, norm: &str) -> Result<Option<i64>, String> {
    let rojo = crate::backventanas::codigos_barras::semaforo_rojo::buscar_aprendizaje_impl;
    if let Some(id) = rojo(pool, norm).await? {
        return Ok(Some(id));
    }
    rojo(pool, &quitar_presentacion(norm)).await
}

/// Aprendizaje (issue #12): nombre ya confirmado antes -> primer
/// candidato con 1.0. Sigue pidiendo confirmacion humana.
pub async fn aprendizaje(pool: &SqlitePool, norm: &str) -> Result<Option<Candidato>, String> {
    let Some(pid) = vinculo_previo(pool, norm).await? else { return Ok(None) };
    let nombre: Option<String> = sqlx::query_scalar("SELECT nombre FROM productos WHERE id = ?")
        .bind(pid)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .flatten();
    Ok(nombre.map(|n| Candidato { producto_id: pid, nombre: n, score: 1.0, origen: "aprendizaje".into() }))
}

/// Top-5 por score desc, solo >= UMBRAL. Vacio = cae a rojo.
pub async fn top5(pool: &SqlitePool, nombre_crudo: &str, marca: Option<&str>) -> Result<Vec<Candidato>, String> {
    let Some(ctx) = armar_ctx(nombre_crudo, marca) else { return Ok(vec![]) };
    let mut lista: Vec<Candidato> = vec![];
    for f in cargar_filas(pool).await? {
        if let Some(c) = puntuar_fila(&ctx, &f) {
            if decide(c.score) {
                lista.push(c);
            }
        }
    }
    lista.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    lista.truncate(5);
    Ok(lista)
}
