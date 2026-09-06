// ============================================================
// clave — Identidad del ticket para idempotencia y orden.
//
// Tres niveles de prioridad:
//   1. Folio impreso (manda: es la identidad que dio la tienda).
//   2. Fecha+hora → folio automático `AUTO-YYYYMMDD-HHMM-xxxxxx`
//      (legible, ordenable cronológicamente y determinista).
//   3. Sin fecha → `SIN-FOLIO-<hash>` del contenido (último recurso).
//
// Más `comparar_cronologico` para insertar los segmentos en el orden en
// que se generaron (los IDs de venta crecen cronológicamente).
// ============================================================

use super::TicketSegmento;

impl TicketSegmento {
    /// Clave de idempotencia del ticket, en 3 niveles de prioridad:
    ///   1. Folio impreso (manda: es la identidad que dio la tienda).
    ///   2. Fecha+hora → folio automático `AUTO-YYYYMMDD-HHMM-xxxxxx`
    ///      (legible, ordenable cronológicamente y determinista: el mismo
    ///      ticket re-importado genera el mismo folio).
    ///   3. Sin fecha → `SIN-FOLIO-<hash>` del contenido.
    ///
    /// Antes, un ticket sin folio detectable NO se podía deduplicar y
    /// re-importar la carpeta duplicaba ventas y descontaba stock dos
    /// veces. Límite honesto: dos ventas DISTINTAS con mismo contenido,
    /// misma fecha y sin folio colisionarían; el folio impreso sigue
    /// siendo la identificación preferible.
    pub fn clave(&self) -> String {
        // 1. Folio impreso.
        if let Some(f) = self.folio.as_deref().map(str::trim).filter(|f| !f.is_empty()) {
            return f.to_string();
        }
        // Hash corto del contenido: distingue dos ventas del mismo minuto.
        let hash_corto = format!("{:012x}", fnv1a64(&self.base_hash()) >> 16);
        // 2. Folio automático desde fecha+hora ("2026-03-04 20:11:00" →
        // "AUTO-20260304-2011-xxxxxx"). Ancho fijo: orden lexicográfico =
        // orden cronológico.
        if let Some(fh) = self.fecha_hora.as_deref() {
            let digitos: String = fh.chars().filter(|c| c.is_ascii_digit()).collect();
            if digitos.len() >= 12 {
                return format!("AUTO-{}-{}-{hash_corto}", &digitos[..8], &digitos[8..12]);
            }
        }
        // 3. Último recurso: hash del contenido (estable entre corridas).
        format!("SIN-FOLIO-{:016x}", fnv1a64(&self.base_hash()))
    }

    /// Contenido normalizado para el hash: espacios colapsados + fecha.
    /// Dos importaciones del mismo ticket dan la misma base (y por tanto
    /// la misma clave); distinto contenido o distinta fecha, distinta clave.
    fn base_hash(&self) -> String {
        let normalizado: Vec<String> = self
            .lineas
            .iter()
            .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
            .collect();
        format!(
            "{}|{}",
            normalizado.join("\n"),
            self.fecha_hora.as_deref().unwrap_or("")
        )
    }
}

/// Hash FNV-1a de 64 bits: estable entre corridas y procesos (a
/// diferencia de `DefaultHasher`, que usa semilla aleatoria por proceso
/// y NO sirve para deduplicar entre una corrida y la siguiente).
pub fn fnv1a64(texto: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in texto.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Orden cronológico de segmentos para inserción: por fecha/hora (el
/// formato ISO de ancho fijo hace que comparar strings = comparar
/// tiempo), sin fecha al final, y por orden de archivo en empates.
/// Pensado para `sort_by` (estable): los IDs de venta crecen en el orden
/// en que se generaron los tickets.
pub fn comparar_cronologico(a: &TicketSegmento, b: &TicketSegmento) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (&a.fecha_hora, &b.fecha_hora) {
        (Some(x), Some(y)) => x.cmp(y).then(a.index.cmp(&b.index)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.index.cmp(&b.index),
    }
}
