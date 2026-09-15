//! sql_readonly — SQL libre de SOLO LECTURA para modelos cloud (issue #15).
//!
//! En vez de una tool por pregunta, el modelo escribe `SELECT`/`WITH`
//! contra un snapshot del schema que viaja en su system prompt. Defensa
//! en 4 capas (ninguna confía en el modelo):
//!   1. Validador sintáctico: primera palabra SELECT/WITH, sin `;`
//!      interiores, sin comentarios, denylist de escritura/DDL/PRAGMA y
//!      veto a tablas internas `sqlite_%`.
//!   2. `LIMIT 1..100` obligatorio (se agrega si falta; se rechaza si
//!      pide más o `-1` = sin límite en SQLite).
//!   3. Conexión `SQLITE_OPEN_READ_ONLY` (la comparte con las demás tools).
//!   4. Timeout de 20s en el llamador + tope de filas/bytes en salida.
//!
//! Solo-admin: se filtra en `herramientas_rol.rs` (TOOLS_SOLO_ADMIN),
//! porque un SELECT puede leer costos y salarios. El empleado conserva
//! sus tools curadas.

use rusqlite::{Connection, types::ValueRef};
use serde_json::Value;

use super::helpers::{round2, str_arg};

/// Filas máximas por consulta y tope del JSON de salida.
pub(crate) const MAX_FILAS_SQL: i64 = 100;
const MAX_CHARS_QUERY: usize = 8000;
const MAX_BYTES_SALIDA: usize = 200_000;

/// Palabras que NUNCA aparecen en una lectura (mayúsculas para comparar).
const DENY: &[&str] = &[
    "INSERT", "UPDATE", "DELETE", "REPLACE", "ALTER", "CREATE", "DROP", "TRUNCATE", "VACUUM",
    "ATTACH", "DETACH", "PRAGMA", "REINDEX", "ANALYZE", "EXPLAIN", "GRANT", "REVOKE", "BEGIN",
    "COMMIT", "ROLLBACK", "SAVEPOINT", "RELEASE",
];

/// Tablas que el modelo puede tocar (el resto ni se le muestra en el schema).
const TABLAS_PERMITIDAS: &[&str] = &[
    "ventas",
    "detalle_ventas",
    "productos",
    "cortes_caja",
    "movimientos_caja",
    "gastos_recurrentes",
    "pagos_gastos",
    "usuarios",
    "asistencias",
    "empleado_horarios",
    "clientes",
    // Abasto y trazabilidad (migraciones 0012/0013/0018, solo lectura).
    "proveedores",
    "compras",
    "compras_items",
    "ordenes_compra",
    "ordenes_items",
    "historial_costos",
    "lotes",
    "sucursales",
    "stock_sucursal",
];

fn es_token(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Nombres de CTEs (`WITH t AS (...)`, con o sin lista de columnas):
/// sus referencias en FROM no son tablas reales y se saltan.
fn nombres_cte(q: &str) -> Vec<String> {
    let chars: Vec<char> = q.chars().collect();
    let mut nombres = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !es_token(chars[i]) {
            i += 1;
            continue;
        }
        let j0 = i;
        while i < chars.len() && es_token(chars[i]) {
            i += 1;
        }
        let tok: String = chars[j0..i].iter().collect();
        // Saltar espacios y opcional lista de columnas `(a, b)`.
        let mut k = i;
        while k < chars.len() && chars[k].is_whitespace() {
            k += 1;
        }
        if k < chars.len() && chars[k] == '(' {
            let mut depth = 0;
            while k < chars.len() {
                if chars[k] == '(' {
                    depth += 1;
                }
                if chars[k] == ')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                k += 1;
            }
            k += 1;
            while k < chars.len() && chars[k].is_whitespace() {
                k += 1;
            }
        }
        let resto: String = chars[k..].iter().collect::<String>().to_ascii_uppercase();
        if resto.starts_with("AS") {
            // Debe seguir `(`: si no, es alias de columna (`SELECT a AS b`).
            let mut m = k + 2;
            while m < chars.len() && chars[m].is_whitespace() {
                m += 1;
            }
            if m < chars.len() && chars[m] == '(' {
                let min = tok.to_ascii_lowercase();
                if !nombres.contains(&min) {
                    nombres.push(min);
                }
                i = m;
                continue;
            }
        }
    }
    nombres
}

/// Valida y normaliza la query. Devuelve el SQL listo para preparar
/// (con `LIMIT 100` agregado si no traía).
fn validar_sql(query: &str) -> Result<String, String> {
    let mut q = query.trim().to_string();
    if q.is_empty() {
        return Err("query vacía.".into());
    }
    if q.len() > MAX_CHARS_QUERY {
        return Err(format!("query demasiado larga (máximo {MAX_CHARS_QUERY} caracteres)."));
    }
    // Sin comentarios: son la vía clásica para colar `SEL/**/ECT` y `--`.
    if q.contains("--") || q.contains("/*") {
        return Err("los comentarios no están permitidos en la query.".into());
    }
    // Una sola sentencia: se permite UN `;` final y nada más.
    if q.ends_with(';') {
        q.pop();
    }
    if q.contains(';') {
        return Err("solo se permite una sentencia por consulta.".into());
    }
    let mut q = q.trim().to_string();

    // Primera palabra: SELECT o WITH.
    let primera: String = q.chars().take_while(|c| !c.is_whitespace()).collect();
    let primera_up = primera.to_ascii_uppercase();
    if primera_up != "SELECT" && primera_up != "WITH" {
        return Err("solo se permiten consultas SELECT o WITH.".into());
    }

    // Denylist por tokens completos (case-insensitive).
    let upper = q.to_ascii_uppercase();
    let mut buf = String::new();
    for c in upper.chars().chain(std::iter::once(' ')) {
        if es_token(c) {
            buf.push(c);
        } else if !buf.is_empty() {
            if DENY.contains(&buf.as_str()) {
                return Err(format!("palabra prohibida en lectura: {buf}."));
            }
            buf.clear();
        }
    }

    // Veto a tablas internas sqlite_% (el schema ya se le da aparte).
    if upper.contains("SQLITE_") {
        return Err("las tablas internas sqlite_% no están permitidas.".into());
    }

    // Tablas mencionadas deben estar en la allowlist. Se revisan los
    // tokens que siguen a FROM/JOIN (con y sin comillas/alias). Los CTEs
    // y las subconsultas `(SELECT ...)` se saltan: sus FROM internos ya
    // se revisan por separado al recorrer todos los tokens.
    let ctes = nombres_cte(&q);
    let toks: Vec<&str> = q.split(|c: char| !(es_token(c) || c == '.' || c == '"' || c == '\'' || c == '`')).filter(|t| !t.is_empty()).collect();
    let mut i = 0;
    while i < toks.len() {
        let t = toks[i].to_ascii_uppercase();
        if t == "FROM" || t == "JOIN" {
            if let Some(tabla) = toks.get(i + 1) {
                let limpia = tabla.trim_matches(|c| c == '"' || c == '\'' || c == '`');
                let base = limpia.split('.').next_back().unwrap_or(limpia);
                let base_up = base.to_ascii_uppercase();
                // `FROM (SELECT ...)` / `FROM SELECT`: subconsulta; sus
                // FROM/JOIN internos ya se revisan por separado.
                if base_up == "SELECT" {
                    i += 1;
                    continue;
                }
                let base_low = base.to_ascii_lowercase();
                if !base.is_empty()
                    && !ctes.contains(&base_low)
                    && !TABLAS_PERMITIDAS.contains(&base_low.as_str())
                {
                    return Err(format!("tabla no permitida: {base}."));
                }
            }
        }
        i += 1;
    }

    // LIMIT 1..100 obligatorio.
    let limite = limite_pedido(&upper);
    match limite {
        Some(n) if (1..=MAX_FILAS_SQL).contains(&n) => {}
        Some(_) => {
            return Err(format!("usa LIMIT entre 1 y {MAX_FILAS_SQL}."));
        }
        None => {
            q.push_str(&format!(" LIMIT {MAX_FILAS_SQL}"));
        }
    }
    Ok(q)
}

/// Extrae el primer entero tras LIMIT, si hay cláusula.
fn limite_pedido(upper: &str) -> Option<i64> {
    let toks: Vec<&str> = upper.split_whitespace().collect();
    for (k, t) in toks.iter().enumerate() {
        if *t == "LIMIT" {
            // Formas: LIMIT 10 / LIMIT 10 OFFSET 5 / LIMIT -1
            let num = toks.get(k + 1).and_then(|s| {
                s.trim_matches(|c| c == '(' || c == ')' || c == ',').parse::<i64>().ok()
            });
            return num;
        }
    }
    None
}

fn valor_a_json(v: ValueRef<'_>) -> Value {
    match v {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => serde_json::json!(i),
        ValueRef::Real(f) => serde_json::json!(round2(f)),
        ValueRef::Text(t) => Value::String(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(_) => Value::String("<blob>".to_string()),
    }
}

/// Ejecuta SQL validado y devuelve {columnas, filas, filas_total}.
/// Si el JSON supera el tope, se recorta con aviso (sin romper el chat).
/// Los rechazos del validador y los errores de sintaxis regresan
/// Ok({"error": ...}) para que el modelo corrija y reintente.
pub(crate) fn sql_readonly(conn: &Connection, args: &Value) -> Result<Value, String> {
    let query = str_arg(args, "query", "");
    let sql = match validar_sql(&query) {
        Ok(s) => s,
        Err(e) => return Ok(serde_json::json!({ "error": e })),
    };

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => return Ok(serde_json::json!({ "error": format!("SQL inválido: {e}") })),
    };
    let ncols = stmt.column_count();
    let columnas: Vec<String> = (0..ncols)
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();

    let mut filas: Vec<Value> = Vec::new();
    let mut filas_total: i64 = 0;
    let mut bytes = 0usize;
    let mut recortado = false;
    let mut rows = stmt.query([]).map_err(|e| format!("error al ejecutar: {e}"))?;
    while let Some(row) = rows.next().map_err(|e| format!("error al leer: {e}"))? {
        filas_total += 1;
        if filas_total > MAX_FILAS_SQL {
            recortado = true;
            break;
        }
        let mut obj = serde_json::Map::with_capacity(ncols);
        for (i, col) in columnas.iter().enumerate() {
            obj.insert(col.clone(), valor_a_json(row.get_ref(i).map_err(|e| e.to_string())?));
        }
        let v = Value::Object(obj);
        bytes += v.to_string().len();
        if bytes > MAX_BYTES_SALIDA {
            recortado = true;
            break;
        }
        filas.push(v);
    }

    let mut out = serde_json::Map::new();
    out.insert("columnas".into(), serde_json::json!(columnas));
    out.insert("filas".into(), Value::Array(filas));
    out.insert("filas_total".into(), serde_json::json!(filas_total));
    if recortado {
        out.insert(
            "aviso".into(),
            serde_json::json!("resultado recortado: afina con WHERE/LIMIT."),
        );
    }
    Ok(Value::Object(out))
}

/// Snapshot del schema (tablas permitidas) para el system prompt del
/// admin: el modelo escribe SQL conociendo columnas y tipos. Dinero en
/// INTEGER centavos; fechas "YYYY-MM-DD HH:MM:SS".
pub fn snapshot_schema(db_path: &str) -> Result<String, String> {
    let conn = Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| format!("no se pudo abrir la base de datos: {e}"))?;
    let mut out = String::from("ESQUEMA (SQLite; dinero en INTEGER centavos):\n");
    for t in TABLAS_PERMITIDAS {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({t})"))
            .map_err(|e| e.to_string())?;
        let cols: Vec<String> = stmt
            .query_map([], |r| {
                let nombre: String = r.get(1)?;
                let tipo: String = r.get(2)?;
                Ok(format!("{nombre}:{tipo}"))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|c| c.ok())
            .collect();
        if !cols.is_empty() {
            out.push_str(&format!("- {t}({})\n", cols.join(", ")));
        }
    }
    Ok(out)
}
