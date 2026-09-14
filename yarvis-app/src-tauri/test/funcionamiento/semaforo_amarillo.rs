// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Semáforo amarillo (issue #11, el difícil).
// Con EANs reales del dataset: sugiere parecido con medida igual,
// veta medida distinta (score 0, ni se sugiere), hunde marca distinta
// (Coca vs Pepsi), confirma/audita, guarda NOs, NINGUNO cae a rojo,
// aprendizaje previo y veto a ean en conflicto. Idempotente.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::{db, escalar_i64};
use yarvis_app_lib::backventanas::codigos_barras::semaforo_amarillo::{
    confirmar_impl, ninguno_impl, rechazar_impl, stats_rechazos_impl, sugerir_impl,
};
use yarvis_app_lib::backventanas::codigos_barras::semaforo_rojo::{
    registrar_pendiente_impl, resolver_asignando_impl,
};

async fn seed(pool: &sqlx::SqlitePool, nombre: &str, marca: &str, cant: f64, uni: &str) -> i64 {
    let r = sqlx::query(
        "INSERT INTO productos (nombre, precio_costo, precio_venta, stock, stock_minimo, vendido, marca, cantidad_presentacion, unidad_presentacion) VALUES (?, 0, 0, 0, 0, 0, ?, ?, ?)",
    )
    .bind(nombre)
    .bind(marca)
    .bind(cant)
    .bind(uni)
    .execute(pool)
    .await
    .unwrap();
    r.last_insert_rowid()
}

#[tokio::test]
async fn sugiere_parecido_con_medida_igual() {
    let pool = db().await;
    let id = seed(&pool, "COCA-COLA 600 ML", "Coca-Cola", 600.0, "ml").await;
    // Ticket realista: pegado, sin guion, misma medida.
    let s = sugerir_impl(&pool, "cocacola 600", None, None).await.unwrap();
    assert_eq!(s.estado, "amarillo");
    assert!(!s.candidatos.is_empty());
    assert_eq!(s.candidatos[0].producto_id, id);
    assert!(s.candidatos[0].score >= 0.80, "score = {}", s.candidatos[0].score);
}

#[tokio::test]
async fn bloqueo_duro_medida_distinta_ni_se_sugiere() {
    let pool = db().await;
    seed(&pool, "COCA COLA ORIGINAL", "Coca-Cola", 3.0, "l").await;
    let s = sugerir_impl(&pool, "coca cola original 600 ml", Some("Coca-Cola"), None)
        .await
        .unwrap();
    assert_eq!(s.estado, "rojo", "600ml vs 3L: cae a rojo");
    assert!(s.candidatos.is_empty());
}

#[tokio::test]
async fn marca_distinta_hunde_coca_vs_pepsi() {
    let pool = db().await;
    seed(&pool, "COLA ORIGINAL", "Pepsi", 600.0, "ml").await;
    // Mismo nombre base y medida, distinta marca: NO sugerir.
    let s = sugerir_impl(&pool, "cola original 600 ml", Some("Coca-Cola"), None)
        .await
        .unwrap();
    assert_eq!(s.estado, "rojo");
}

#[tokio::test]
async fn si_es_este_asigna_y_audita() {
    let pool = db().await;
    let id = seed(&pool, "COCA-COLA 600 ML", "Coca-Cola", 600.0, "ml").await;
    let s = sugerir_impl(&pool, "cocacola 600", None, None).await.unwrap();
    confirmar_impl(&pool, s.pendiente_id, id, Some("0000075007614"), None)
        .await
        .unwrap();
    let cb: Option<String> = sqlx::query_scalar("SELECT codigo_barras FROM productos WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(cb.as_deref(), Some("0000075007614"));
    assert_eq!(
        escalar_i64(&pool, "SELECT COUNT(*) FROM vinculos_codigos WHERE origen = 'confirmado-amarillo'").await,
        1
    );
    let estado: String = sqlx::query_scalar("SELECT estado FROM pendientes_codigos WHERE id = ?")
        .bind(s.pendiente_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(estado, "resuelto");
}

#[tokio::test]
async fn no_es_guarda_rechazo_y_stats() {
    let pool = db().await;
    let id = seed(&pool, "COCA-COLA 600 ML", "Coca-Cola", 600.0, "ml").await;
    let s = sugerir_impl(&pool, "cocacola 600", None, None).await.unwrap();
    let score = s.candidatos[0].score;
    rechazar_impl(&pool, s.pendiente_id, id, score).await.unwrap();
    let stats = stats_rechazos_impl(&pool).await.unwrap();
    assert_eq!(stats.total, 1);
    assert!(stats.score_maximo >= score);
}

#[tokio::test]
async fn ninguno_cae_a_rojo() {
    let pool = db().await;
    seed(&pool, "COCA-COLA 600 ML", "Coca-Cola", 600.0, "ml").await;
    let s = sugerir_impl(&pool, "cocacola 600", None, None).await.unwrap();
    ninguno_impl(&pool, s.pendiente_id).await.unwrap();
    let estado: String = sqlx::query_scalar("SELECT estado FROM pendientes_codigos WHERE id = ?")
        .bind(s.pendiente_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(estado, "rojo");
}

#[tokio::test]
async fn aprendizaje_previo_llega_minimo_amarillo() {
    let pool = db().await;
    let id = seed(&pool, "PAN DULCE SURTIDO", "Bimbo", 500.0, "g").await;
    // El rojo manual de ayer: pitaste y asignaste una vez.
    let p = registrar_pendiente_impl(&pool, Some("7501000111206"), "PAN DULCE SURTIDO")
        .await
        .unwrap();
    resolver_asignando_impl(&pool, p, id, None, None).await.unwrap();
    // Hoy el mismo nombre ya no parte de cero.
    let s = sugerir_impl(&pool, "pan dulce surtido 500 g", None, None).await.unwrap();
    assert_eq!(s.estado, "aprendizaje");
    assert_eq!(s.candidatos[0].origen, "aprendizaje");
}

#[tokio::test]
async fn ean_en_conflicto_no_se_sugiere() {
    let pool = db().await;
    let a = seed(&pool, "SABRITAS ORIGINAL", "Sabritas", 42.0, "g").await;
    let b = seed(&pool, "SABRITAS ORIGINAL", "Sabritas", 42.0, "g").await;
    for pid in [a, b] {
        sqlx::query("INSERT INTO vinculos_codigos (ean, producto_id, origen) VALUES ('7501011101456', ?, 'manual-rojo')")
            .bind(pid)
            .execute(&pool)
            .await
            .unwrap();
    }
    let s = sugerir_impl(&pool, "sabritas original 42 g", Some("Sabritas"), Some("7501011101456"))
        .await
        .unwrap();
    assert_eq!(s.estado, "conflicto");
    assert!(s.candidatos.is_empty());
}

#[tokio::test]
async fn resugerir_no_duplica_pendientes() {
    let pool = db().await;
    seed(&pool, "COCA-COLA 600 ML", "Coca-Cola", 600.0, "ml").await;
    let s1 = sugerir_impl(&pool, "cocacola 600", None, None).await.unwrap();
    let s2 = sugerir_impl(&pool, "cocacola 600", None, None).await.unwrap();
    assert_eq!(s1.pendiente_id, s2.pendiente_id);
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM pendientes_codigos").await, 1);
}
