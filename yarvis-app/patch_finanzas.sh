#!/bin/bash
sed -i '3a use sqlx::Row;\nuse chrono::Datelike;' src-tauri/src/backventanas/backadmin/adminfinanzas/alertas.rs
sed -i 's/let dia = dia_pago.unwrap_or(1).clamp(1, 15);/let dia = dia_pago.unwrap_or(1).clamp(1, 15) as u32;/g' src-tauri/src/backventanas/backadmin/adminfinanzas/alertas.rs
sed -i 's/let dia = dia_pago.unwrap_or(1).clamp(1, 28);/let dia = dia_pago.unwrap_or(1).clamp(1, 28) as u32;/g' src-tauri/src/backventanas/backadmin/adminfinanzas/alertas.rs

sed -i '2a use sqlx::Row;\nuse serde::{Serialize, Deserialize};' src-tauri/src/backventanas/backadmin/adminfinanzas/cortes.rs
