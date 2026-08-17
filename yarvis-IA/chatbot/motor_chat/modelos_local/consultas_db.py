"""
🗄️ consultas_db.py — Capa de acceso a la base de datos.

Se encarga de:
    - Localizar el archivo yarvis.db en el sistema (env var + rutas por SO).
    - Leer productos, ventas, cancelaciones, reembolsos y empleados.
    - Obtener los datos de la tienda (nombre, ubicación).

Rendimiento: la conexión SQLite se abre UNA vez por hilo y se reutiliza en
todas las consultas (thread-local), en vez de abrir/cerrar por llamada.

Ninguna otra responsabilidad: aquí NO hay embeddings, prompts ni endpoints.
"""

import os
import sqlite3
import sys
import threading
from datetime import datetime, timedelta


def _rutas_candidatas() -> list[str]:
    """Rutas donde puede vivir yarvis.db según el sistema operativo."""
    rutas: list[str] = []
    if sys.platform == "win32":
        appdata = os.environ.get("APPDATA", "")
        local = os.environ.get("LOCALAPPDATA", "")
        for base in (local, appdata):
            if base:
                rutas.append(os.path.join(base, "com.yarvis.pos", "yarvis.db"))
                rutas.append(os.path.join(base, "yarvis-app", "yarvis.db"))
    elif sys.platform == "darwin":
        base = os.path.expanduser("~/Library/Application Support")
        rutas.append(os.path.join(base, "com.yarvis.pos", "yarvis.db"))
        rutas.append(os.path.join(base, "yarvis-app", "yarvis.db"))

    home_linux = os.path.expanduser("~/.local/share")
    rutas.append(os.path.join(home_linux, "com.yarvis.pos", "yarvis.db"))
    rutas.append(os.path.join(home_linux, "yarvis-app", "yarvis.db"))
    rutas.append(os.path.expanduser("~/.config/yarvis-app/yarvis.db"))
    rutas.append(os.path.join(os.getcwd(), "yarvis.db"))
    rutas.append("yarvis.db")

    # Dedupe conservando el orden
    return list(dict.fromkeys(rutas))


_RUTAS_DB = _rutas_candidatas()

# Conexión por hilo (thread-local): se reutiliza, no se abre/cierra por consulta.
_local = threading.local()


def find_db_path() -> str:
    """Localiza yarvis.db; da prioridad a la env var YARVIS_DB_PATH.

    Devuelve '' si no existe en ninguna ruta.
    """
    env = os.environ.get("YARVIS_DB_PATH", "").strip()
    if env and os.path.exists(env):
        return env
    for path in _RUTAS_DB:
        if os.path.exists(path):
            return path
    return ""


def _cargar_extension_vec(conn) -> None:
    """Carga la extensión sqlite-vec UNA vez por conexión.

    Se ejecuta al abrir la conexión (no en cada búsqueda), para evitar
    re-inyectar la biblioteca C en la memoria de SQLite por llamada.
    Si la extensión no está instalada, se ignora: la búsqueda semántica
    simplemente caerá al respaldo por palabras clave.
    """
    try:
        conn.enable_load_extension(True)
        import sqlite_vec
        sqlite_vec.load(conn)
    except Exception:
        pass
    finally:
        try:
            conn.enable_load_extension(False)
        except Exception:
            pass


def _conectar():
    """Conexión SQLite reutilizada por hilo; None si no existe la base de datos.

    Si la ruta de la DB cambia o aparece después, reconecta automáticamente
    (por ejemplo, cuando Rust crea el archivo la primera vez).
    """
    conn = getattr(_local, "conn", None)
    ruta = find_db_path()
    if conn is not None:
        if getattr(_local, "ruta", None) == ruta:
            return conn
        try:
            conn.close()
        except Exception:
            pass
        _local.conn = None
    if not ruta:
        _local.ruta = ""
        _local.conn = None
        return None
    conn = sqlite3.connect(ruta)
    conn.row_factory = sqlite3.Row
    try:
        conn.execute("PRAGMA busy_timeout = 5000")
    except Exception:
        pass
    _cargar_extension_vec(conn)
    _local.ruta = ruta
    _local.conn = conn
    return conn


def cargar_productos() -> list[dict] | None:
    """Devuelve todos los productos ordenados por nombre (para la caché RAG).
    Retorna None si no se encontró la base de datos."""
    conn = _conectar()
    if conn is None:
        return None
    rows = conn.execute(
        "SELECT nombre, stock, precio_venta, precio_costo, categoria, stock_minimo "
        "FROM productos ORDER BY nombre"
    ).fetchall()
    return [
        {
            "nombre": r["nombre"],
            "stock": r["stock"],
            "precio_venta": r["precio_venta"],
            "precio_costo": r["precio_costo"],
            "categoria": r["categoria"] or "Sin categoría",
            "stock_minimo": r["stock_minimo"] or 0,
        }
        for r in rows
    ]


def obtener_tienda_info() -> dict:
    """Nombre y ubicación de la tienda desde la cuenta del admin."""
    conn = _conectar()
    if conn is None:
        return {"nombre": "la tienda", "ubicacion": ""}
    try:
        row = conn.execute(
            "SELECT tienda, ubicacion FROM usuarios WHERE rol = 'admin' LIMIT 1"
        ).fetchone()
    except Exception:
        return {"nombre": "la tienda", "ubicacion": ""}
    if row:
        return {"nombre": row["tienda"] or "la tienda", "ubicacion": row["ubicacion"] or ""}
    return {"nombre": "la tienda", "ubicacion": ""}


def obtener_ventas_hoy() -> dict:
    """Tickets y total vendidos hoy."""
    conn = _conectar()
    if conn is None:
        return {"tickets": 0, "total": 0.0}
    hoy = datetime.now().strftime("%Y-%m-%d")
    v = conn.execute(
        "SELECT COUNT(*) as tickets, COALESCE(SUM(total), 0) as total "
        "FROM ventas WHERE DATE(fecha) = ?", (hoy,)
    ).fetchone()
    return {"tickets": v["tickets"], "total": v["total"]}


def obtener_ventas_7dias() -> dict:
    """Tickets y total vendidos en los últimos 7 días."""
    conn = _conectar()
    if conn is None:
        return {"tickets": 0, "total": 0.0}
    hace_7 = (datetime.now() - timedelta(days=7)).strftime("%Y-%m-%d")
    v = conn.execute(
        "SELECT COUNT(*) as tickets, COALESCE(SUM(total), 0) as total "
        "FROM ventas WHERE DATE(fecha) >= ?", (hace_7,)
    ).fetchone()
    return {"tickets": v["tickets"], "total": v["total"]}


def obtener_ventas_por_cajero(dias: int = 7) -> list[dict]:
    """Ventas agrupadas por cajero en los últimos N días."""
    conn = _conectar()
    if conn is None:
        return []
    fecha = (datetime.now() - timedelta(days=dias)).strftime("%Y-%m-%d")
    rows = conn.execute(
        "SELECT cajero, COUNT(*) as n, SUM(total) as t "
        "FROM ventas WHERE DATE(fecha) >= ? GROUP BY cajero ORDER BY t DESC",
        (fecha,)
    ).fetchall()
    return [{"cajero": r["cajero"], "ventas": r["n"], "total": r["t"]} for r in rows]


def obtener_cancelaciones_por_cajero(dias: int = 7) -> list[dict]:
    """Ventas canceladas agrupadas por cajero en los últimos N días."""
    conn = _conectar()
    if conn is None:
        return []
    fecha = (datetime.now() - timedelta(days=dias)).strftime("%Y-%m-%d")
    rows = conn.execute(
        "SELECT cajero, COUNT(*) as n FROM ventas "
        "WHERE estado = 'cancelada' AND DATE(fecha) >= ? GROUP BY cajero",
        (fecha,)
    ).fetchall()
    return [{"cajero": r["cajero"], "cancelaciones": r["n"]} for r in rows]


def obtener_reembolsos_por_producto(dias: int = 30) -> list[dict]:
    """Productos más devueltos (ventas canceladas) en los últimos N días."""
    conn = _conectar()
    if conn is None:
        return []
    fecha = (datetime.now() - timedelta(days=dias)).strftime("%Y-%m-%d")
    rows = conn.execute(
        "SELECT dv.producto_nombre, COUNT(*) as n "
        "FROM detalle_ventas dv JOIN ventas v ON dv.venta_id = v.id "
        "WHERE v.estado = 'cancelada' AND DATE(v.fecha) >= ? "
        "GROUP BY dv.producto_nombre ORDER BY n DESC LIMIT 5",
        (fecha,)
    ).fetchall()
    return [{"producto": r["producto_nombre"], "reembolsos": r["n"]} for r in rows]


def obtener_productos_mas_vendidos(dias: int = 30, top: int = 8) -> list[dict]:
    """Productos más vendidos (por cantidad) en los últimos N días.

    Incluye margen de ganancia estimado = Σ (precio_venta - costo) * cantidad;
    si un producto no tiene costo registrado se toma costo 0 (margen alto de más).
    """
    conn = _conectar()
    if conn is None:
        return []
    fecha = (datetime.now() - timedelta(days=dias)).strftime("%Y-%m-%d")
    rows = conn.execute(
        "SELECT dv.producto_nombre, SUM(dv.cantidad) as cantidad, "
        "       SUM(dv.subtotal) as total, "
        "       SUM((dv.precio_unitario - COALESCE(p.precio_costo, 0)) * dv.cantidad) as margen "
        "FROM detalle_ventas dv JOIN ventas v ON dv.venta_id = v.id "
        "LEFT JOIN productos p ON p.nombre = dv.producto_nombre COLLATE NOCASE "
        "WHERE v.estado != 'cancelada' "
        "      AND (DATE(v.fecha) >= ? OR substr(v.fecha, 7, 4) || '-' || "
        "           substr(v.fecha, 4, 2) || '-' || substr(v.fecha, 1, 2) >= ?) "
        "GROUP BY dv.producto_nombre ORDER BY cantidad DESC LIMIT ?",
        (fecha, fecha, top)
    ).fetchall()
    return [
        {
            "producto": r["producto_nombre"],
            "cantidad": r["cantidad"],
            "total": r["total"] or 0.0,
            "margen": r["margen"] or 0.0,
        }
        for r in rows
    ]


def obtener_ventas_por_producto(periodo: str = "mes", top: int = 10) -> list[dict]:
    """Cuánto se vendió de cada producto en el período ('hoy' | 'semana' | 'mes').

    SOLO LECTURA: nunca modifica la base de datos. El período 'hoy' mira desde
    el inicio de hoy, 'semana' los últimos 7 días y 'mes' los últimos 30.
    """
    conn = _conectar()
    if conn is None:
        return []
    rango = {"hoy": 0, "semana": 7, "mes": 30}.get(periodo, 30)
    fecha = (datetime.now() - timedelta(days=rango)).strftime("%Y-%m-%d")
    clausula_fecha = (
        "DATE(v.fecha) >= ? OR "
        "substr(v.fecha, 7, 4) || '-' || substr(v.fecha, 4, 2) || '-' || substr(v.fecha, 1, 2) >= ?"
    )
    rows = conn.execute(
        "SELECT dv.producto_nombre, SUM(dv.cantidad) as cantidad, "
        "       SUM(dv.subtotal) as total, "
        "       SUM((dv.precio_unitario - COALESCE(p.precio_costo, 0)) * dv.cantidad) as margen "
        "FROM detalle_ventas dv JOIN ventas v ON dv.venta_id = v.id "
        "LEFT JOIN productos p ON p.nombre = dv.producto_nombre COLLATE NOCASE "
        f"WHERE v.estado != 'cancelada' AND ({clausula_fecha}) "
        "GROUP BY dv.producto_nombre ORDER BY cantidad DESC LIMIT ?",
        (fecha, fecha, top)
    ).fetchall()
    return [
        {
            "producto": r["producto_nombre"],
            "cantidad": r["cantidad"],
            "total": r["total"] or 0.0,
            "margen": r["margen"] or 0.0,
        }
        for r in rows
    ]


def obtener_empleados() -> list[dict]:
    """Todos los empleados con su turno, salario, meta y estado."""
    conn = _conectar()
    if conn is None:
        return []
    rows = conn.execute(
        "SELECT nombre, turno, salario_semanal, meta_mensual, estado "
        "FROM usuarios WHERE rol = 'empleado'"
    ).fetchall()
    return [
        {
            "nombre": r["nombre"],
            "turno": r["turno"],
            "salario_semanal": r["salario_semanal"],
            "meta_mensual": r["meta_mensual"],
            "estado": r["estado"],
        }
        for r in rows
    ]