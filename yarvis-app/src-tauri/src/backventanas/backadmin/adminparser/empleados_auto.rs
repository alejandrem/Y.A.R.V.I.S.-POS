// ============================================================
// empleados_auto — Alta automática de empleados desde tickets.
//
// Si el parseador detecta cajeros que no existen como usuarios, el sistema
// los crea solos (como se rellena el inventario solo): nombre del ticket,
// contraseña = nombre + "123" ("1234", "12345"… si choca), rol empleado,
// salario 0 (eso se captura manual después). Además vincula cajero_id en
// las ventas para que las estadísticas por empleado funcionen.
//
// Reglas:
// - Mismo nombre normalizado = misma persona: se reutiliza (nunca se
//   duplica solo; si no, cada re-importe crearía usuarios sin fin).
// - Si ≥2 usuarios comparten nombre (creado manual aparte), se vincula
//   al más antiguo: es lo único estable entre corridas.
// - La contraseña debe ser ÚNICA en todo el sistema porque el login de
//   empleado es solo-password (el primero que coincida gana): por eso el
//   sufijo 123→1234→12345 cuando la derivada ya la tiene otra persona.
// - Se omiten: SISTEMA, IMPORTADOR, SIN ASIGNAR y vacíos.
// - Idempotente: re-correr sobre lo mismo no crea ni re-vincula nada.
// ============================================================

use crate::backventanas::backadmin::adminconfig::auth::{hash_password, verify_password};

/// Cajeros que nunca son personas (sistema / importador / sin dato).
const OMITIR: &[&str] = &["SISTEMA", "IMPORTADOR", "SIN ASIGNAR", ""];

/// Normaliza para comparar personas: mayúsculas, espacios colapsados,
/// sin punto final ("MARIA G." y "MARIA G" son la misma persona).
pub fn normalizar_empleado(nombre: &str) -> String {
    let colapsado = nombre.split_whitespace().collect::<Vec<_>>().join(" ");
    colapsado
        .trim_end_matches('.')
        .trim()
        .to_uppercase()
}

/// Credencial generada para mostrar UNA vez en el resumen de importación
/// (el admin se la comunica al empleado). En DB solo vive el hash.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CredencialAuto {
    pub nombre: String,
    pub password_plana: String,
}

pub struct ResultadoEmpleadosAuto {
    pub creados: Vec<CredencialAuto>,
    pub vinculados: usize,
}

/// Resuelve empleados desde los cajeros sin vincular de `ventas`:
/// crea los que falten y pone `cajero_id` en sus tickets.
pub fn resolver_empleados_desde_ventas(db_path: &str) -> Result<ResultadoEmpleadosAuto, String> {
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;
    resolver_empleados_desde_conn(&conn)
}

/// Núcleo testeable (misma lógica sobre una conexión abierta).
pub fn resolver_empleados_desde_conn(
    conn: &rusqlite::Connection,
) -> Result<ResultadoEmpleadosAuto, String> {
    // Usuarios empleados actuales: id + nombre normalizado + hashes (para
    // garantizar contraseñas únicas en el login solo-password).
    let mut usuarios: Vec<(i64, String)> = Vec::new();
    let mut hashes: Vec<String> = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT id, nombre, password FROM usuarios WHERE rol = 'empleado'")
            .map_err(|e| e.to_string())?;
        let filas = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for fila in filas {
            let fila = fila.map_err(|e| e.to_string())?;
            usuarios.push((fila.0, normalizar_empleado(&fila.1)));
            hashes.push(fila.2);
        }
    }

    // Cajeros distintos sin vincular.
    let mut cajeros: Vec<String> = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT DISTINCT cajero FROM ventas WHERE cajero_id IS NULL AND cajero IS NOT NULL")
            .map_err(|e| e.to_string())?;
        let filas = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        for fila in filas {
            cajeros.push(fila.map_err(|e| e.to_string())?);
        }
    }

    let mut creados = Vec::new();
    let mut vinculados = 0usize;

    for cajero in cajeros {
        let nombre = cajero.trim().to_string();
        if nombre.is_empty() {
            continue;
        }
        let norm = normalizar_empleado(&nombre);
        if OMITIR.contains(&norm.as_str()) {
            continue;
        }

        // ¿Ya existe(n) alguien con ese nombre? (el más antiguo manda).
        let mut conocidos: Vec<i64> = usuarios
            .iter()
            .filter(|(_, n)| *n == norm)
            .map(|(id, _)| *id)
            .collect();
        conocidos.sort_unstable();
        let empleado_id = if let Some(&id) = conocidos.first() {
            id
        } else {
            // Nuevo: contraseña nombre+123 ("1234", "12345"… si choca con
            // la de otra persona, porque el login solo-password exige
            // contraseñas únicas en todo el sistema).
            let mut password = format!("{nombre}123");
            let mut digito = 4;
            while hashes.iter().any(|h| verify_password(&password, h)) {
                password = format!("{password}{digito}");
                digito += 1;
            }
            let hash = hash_password(&password);
            conn.execute(
                "INSERT INTO usuarios (nombre, password, rol, estado, password_defecto) VALUES (?1, ?2, 'empleado', 'activo', 1)",
                rusqlite::params![nombre, hash],
            )
            .map_err(|e| e.to_string())?;
            let nuevo_id: i64 = conn.last_insert_rowid();
            usuarios.push((nuevo_id, norm.clone()));
            hashes.push(hash);
            creados.push(CredencialAuto {
                nombre: nombre.clone(),
                password_plana: password,
            });
            nuevo_id
        };

        // Vincula TODOS los tickets sin cajero_id de esa persona (los recién
        // importados y, en su caso, históricos pendientes).
        vinculados += vincular_exactos(&conn, empleado_id, &norm)? as usize;
    }

    Ok(ResultadoEmpleadosAuto {
        creados,
        vinculados,
    })
}

/// Vincula por nombre normalizado exacto calculado en Rust (UPPER + trim +
/// colapso de espacios + sin punto final). Seguro contra falsos positivos.
fn vincular_exactos(
    conn: &rusqlite::Connection,
    empleado_id: i64,
    norm: &str,
) -> Result<usize, String> {
    let mut pendientes: Vec<(i64, String)> = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT id, cajero FROM ventas WHERE cajero_id IS NULL AND cajero IS NOT NULL")
            .map_err(|e| e.to_string())?;
        let filas = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        for fila in filas {
            pendientes.push(fila.map_err(|e| e.to_string())?);
        }
    }
    let ids: Vec<i64> = pendientes
        .into_iter()
        .filter(|(_, c)| normalizar_empleado(c) == norm)
        .map(|(id, _)| id)
        .collect();
    if ids.is_empty() {
        return Ok(0);
    }
    let lista = ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
    let n = conn
        .execute(
            &format!("UPDATE ventas SET cajero_id = ?1 WHERE id IN ({lista})"),
            rusqlite::params![empleado_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(n)
}
