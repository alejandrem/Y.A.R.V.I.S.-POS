// Tests del segmentador: corte de bloques, folios reales, clave de
// idempotencia y orden cronológico.

use super::*;

#[test]
fn dos_tickets_con_folio_y_total_generan_dos_segmentos() {
    let texto = "FOLIO: 0001\n\
                 12/05/2026\n\
                 2 COCA $25.00 $50.00\n\
                 TOTAL $50.00\n\
                 FOLIO: 0002\n\
                 13/05/2026\n\
                 1 PAN $10.00 $10.00\n\
                 TOTAL $10.00\n";

    let segs = segmentar(texto);
    assert_eq!(segs.len(), 2);

    assert_eq!(segs[0].index, 1);
    assert_eq!(segs[0].folio.as_deref(), Some("0001"));
    assert_eq!(segs[0].fecha_hora.as_deref(), Some("2026-05-12 00:00:00"));
    assert!(segs[0].lineas.iter().any(|l| l.contains("2 COCA")));
    assert!(!segs[0].lineas.iter().any(|l| l.contains("FOLIO: 0002")));

    assert_eq!(segs[1].index, 2);
    assert_eq!(segs[1].folio.as_deref(), Some("0002"));
    assert_eq!(segs[1].fecha_hora.as_deref(), Some("2026-05-13 00:00:00"));
    assert!(segs[1].lineas.iter().any(|l| l.contains("1 PAN")));
}

#[test]
fn sin_marcadores_un_solo_segmento_retrocompatible() {
    let texto = "2 COCA $25.00 $50.00\n1 PAN $10.00 $10.00\n";

    let segs = segmentar(texto);
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0].index, 1);
    assert_eq!(segs[0].folio, None);
    assert_eq!(segs[0].fecha_hora, None);
    assert_eq!(segs[0].lineas.len(), 2);
}

#[test]
fn fecha_nueva_inicia_ticket_sin_folio() {
    let texto = "15/05/2026\n2 COCA $25.00 $50.00\nTOTAL $50.00\n\
                 16/05/2026\n1 PAN $10.00 $10.00\nTOTAL $10.00\n";

    let segs = segmentar(texto);
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].folio, None);
    assert_eq!(segs[0].fecha_hora.as_deref(), Some("2026-05-15 00:00:00"));
    assert_eq!(segs[1].folio, None);
    assert_eq!(segs[1].fecha_hora.as_deref(), Some("2026-05-16 00:00:00"));
}

#[test]
fn marcadores_consecutivos_se_acumulan_en_un_solo_encabezado() {
    let texto = "FOLIO: 88\n12/05/2026\n2 COCA $25.00 $50.00\nTOTAL $50.00\n";

    let segs = segmentar(texto);
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0].folio.as_deref(), Some("88"));
    assert_eq!(segs[0].fecha_hora.as_deref(), Some("2026-05-12 00:00:00"));
    assert_eq!(segs[0].lineas.len(), 4);
}

#[test]
fn ticket_real_con_encabezado_folio_y_hora() {
    let texto = "Farmacia San Pablo\n\
                 Av. Juzarez 123, CDMX\n\
                 Ticket: 004582\n\
                 Fecha: 15/03/2024  14:32\n\
                 -----------------------------------\n\
                 2 Pan Bimbo Integral         42.00     84.00\n\
                 1 Leche Lala Light 1L        26.50     26.50\n\
                 -----------------------------------\n\
                 SUBTOTAL: 166.00\n\
                 IVA 16%: 26.56\n\
                 TOTAL: $192.56\n\
                 Tarjeta: **** 1234\n\
                 Gracias por su compra\n";

    let segs = segmentar(texto);
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0].folio.as_deref(), Some("004582"));
    assert_eq!(segs[0].fecha_hora.as_deref(), Some("2024-03-15 14:32:00"));
    assert!(segs[0].lineas.iter().any(|l| l.contains("Pan Bimbo")));
    // El bloque va de "Ticket:" hasta "TOTAL" y se queda el pie (pago/gracias).
    assert!(segs[0].lineas.first().unwrap().contains("Ticket:"));
    assert!(segs[0].lineas.iter().any(|l| l.contains("TOTAL:")));
    assert!(segs[0].lineas.iter().any(|l| l.contains("Gracias")));
}

#[test]
fn cierre_sin_apertura_se_ignora() {
    let texto = "FOLIO: 1\n2 COCA $25.00 $50.00\nTOTAL $50.00\n\
                 GRACIAS POR SU COMPRA\n2 PAN $10.00 $10.00\n";

    let segs = segmentar(texto);
    assert_eq!(segs.len(), 1);
    // El segundo "ticket" sin encabezado (tras el cierre) queda fuera.
    assert!(!segs[0].lineas.iter().any(|l| l.contains("PAN")));
}

#[test]
fn texto_vacio_no_genera_segmentos() {
    assert!(segmentar("   \n  \n").is_empty());
    assert!(segmentar("").is_empty());
}

#[test]
fn extraer_folio_soporta_todos_los_formatos() {
    assert_eq!(extraer_folio("FOLIO: 004582").as_deref(), Some("004582"));
    assert_eq!(extraer_folio("TICKET # A-123").as_deref(), Some("A-123"));
    assert_eq!(extraer_folio("Ticket: 004582").as_deref(), Some("004582"));
    assert_eq!(extraer_folio("NO. TICKET: 0002").as_deref(), Some("0002"));
    assert_eq!(extraer_folio("SERIE A-123").as_deref(), Some("A-123"));
    assert_eq!(extraer_folio("12/05/2026"), None);
}

#[test]
fn extraer_folio_formatos_reales_de_tiendas() {
    // Separadores `|` y `=` de sistemas con campos delimitados.
    assert_eq!(extraer_folio("FOLIO|55190").as_deref(), Some("55190"));
    assert_eq!(
        extraer_folio("TIENDA=ABARROTES_LOPEZ;FOLIO=88231;FECHA=2026-03-09")
            .as_deref(),
        Some("88231")
    );
    // Abreviatura "FOL" con y sin dos puntos.
    assert_eq!(
        extraer_folio("Fol 3341 - 06/03/2026 08:55").as_deref(),
        Some("3341")
    );
    assert_eq!(
        extraer_folio("FOL 00721  07/03/26 12:30").as_deref(),
        Some("00721")
    );
    assert_eq!(extraer_folio("FOLIO:2288 04/03/26 21:03").as_deref(), Some("2288"));
    // La muletilla "NO." no se captura como folio (antes daba "NO").
    assert_eq!(extraer_folio("TICKET NO. 1927").as_deref(), Some("1927"));
    assert_eq!(extraer_folio("Ticket #6650").as_deref(), Some("6650"));
    assert_eq!(
        extraer_folio("TICKET: A-004471        10/03/2026").as_deref(),
        Some("A-004471")
    );
    // Sin dígito no es folio: nada de folios basura ni colisiones.
    assert_eq!(extraer_folio("CONSERVE SU TICKET"), None);
    assert_eq!(extraer_folio("TICKET DE VENTA"), None);
    assert_eq!(extraer_folio("No. ARTICULOS: 8"), None);
    assert_eq!(extraer_folio("FOLIO:"), None);
}

#[test]
fn conserve_su_ticket_no_abre_segmento_fantasma() {
    // "CONSERVE SU TICKET" trae la palabra ticket pero sin folio ni
    // fecha: es pie, no un segundo ticket.
    let texto = "TICKET: A-004471        10/03/2026  20:15:09\n\
                 2 COCA $25.00 $50.00\n\
                 TOTAL: 50.00\n\
                 ================================================\n\
                 CONSERVE SU TICKET\n\
                 ================================================\n";
    let segs = segmentar(texto);
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0].folio.as_deref(), Some("A-004471"));
    assert!(segs[0].lineas.iter().any(|l| l.contains("CONSERVE")));
}

#[test]
fn metodo_pago_se_extrae_de_cada_segmento() {
    let texto = "FOLIO: 1\n12/05/2026\n1 COCA $25.00 $25.00\nTOTAL $25.00\n\
                 METODO DE PAGO: TARJETA DEBITO\n\
                 FOLIO: 2\n13/05/2026\n1 PAN $10.00 $10.00\nTOTAL $10.00\n\
                 FORMA DE PAGO: EFECTIVO\n";

    let segs = segmentar(texto);
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].metodo_pago, "tarjeta");
    assert_eq!(segs[1].metodo_pago, "efectivo");
}

#[test]
fn clave_es_folio_si_hay_y_hash_estable_si_no() {
    let con_folio = &segmentar("FOLIO: 9\n2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
    assert_eq!(con_folio.clave(), "9");

    let a = &segmentar("2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
    let b = &segmentar("2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
    assert!(a.clave().starts_with("SIN-FOLIO-"));
    // Estable entre corridas: mismo contenido → misma clave.
    assert_eq!(a.clave(), b.clave());

    // Contenido distinto → clave distinta (no se come ventas ajenas).
    let c = &segmentar("3 COCA $25.00 $75.00\nTOTAL $75.00\n")[0];
    assert_ne!(a.clave(), c.clave());
}

#[test]
fn clave_auto_desde_fecha_y_hora_es_legible_y_ordenable() {
    let a = &segmentar("12/05/2026 14:30\n2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
    let b = &segmentar("12/05/2026 14:30\n2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
    assert!(
        a.clave().starts_with("AUTO-20260512-1430-"),
        "clave: {}",
        a.clave()
    );
    // Determinista: re-importar da el mismo folio automático.
    assert_eq!(a.clave(), b.clave());

    // Distinto contenido el mismo minuto → distinto sufijo.
    let c = &segmentar("12/05/2026 14:30\n3 COCA $25.00 $75.00\nTOTAL $75.00\n")[0];
    assert_ne!(a.clave(), c.clave());

    // Orden lexicográfico = orden cronológico (ancho fijo).
    let d = &segmentar("13/05/2026 09:00\n2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
    assert!(a.clave() < d.clave());
}

#[test]
fn clave_prioriza_folio_impreso_sobre_fecha() {
    let s = &segmentar("FOLIO: 7\n12/05/2026\n2 COCA $25.00 $50.00\nTOTAL $50.00\n")[0];
    assert_eq!(s.clave(), "7");
}

#[test]
fn comparar_cronologico_ordena_y_manda_sin_fecha_al_final() {
    let mut segs = segmentar(
        "14/05/2026\n2 COCA $25.00 $50.00\nTOTAL $50.00\n\
         12/05/2026\n1 PAN $10.00 $10.00\nTOTAL $10.00\n",
    );
    segs.sort_by(comparar_cronologico);
    assert_eq!(segs[0].fecha_hora.as_deref(), Some("2026-05-12 00:00:00"));
    assert_eq!(segs[1].fecha_hora.as_deref(), Some("2026-05-14 00:00:00"));

    let mut mixtos = segmentar("2 COCA $25.00 $50.00\nTOTAL $50.00\n");
    mixtos.extend(segmentar("12/05/2026\n1 PAN $10.00 $10.00\nTOTAL $10.00\n"));
    mixtos.sort_by(comparar_cronologico);
    assert!(mixtos[0].fecha_hora.is_some());
    assert!(mixtos[1].fecha_hora.is_none());
}

#[test]
fn tickets_reales_fol_y_fecha_abreviada() {
    // ticket_06: "Fol" sin dos puntos + productos de un solo importe.
    let t06 = "*** MISCELANEA \"LAS DELICIAS\" ***\n\
               Fol 3341 - 06/03/2026 08:55\n\
               2x Coca 600         $36.00\n\
               1x Sabritas Original $18.50\n\
               TOTAL A PAGAR $129.50\n\
               Gracias - vuelva pronto\n";
    let segs = segmentar(t06);
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0].folio.as_deref(), Some("3341"));
    assert_eq!(segs[0].fecha_hora.as_deref(), Some("2026-03-06 08:55:00"));

    // ticket_03: campos con `|` e ISO con hora pegada.
    let t03 = "TIENDA|LA GUADALUPANA|SUC01\n\
               FOLIO|55190\n\
               FECHA|2026-03-04T20:11:03\n\
               TOTAL|220.70\n";
    let segs = segmentar(t03);
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0].folio.as_deref(), Some("55190"));
    assert_eq!(segs[0].fecha_hora.as_deref(), Some("2026-03-04 20:11:00"));
}
