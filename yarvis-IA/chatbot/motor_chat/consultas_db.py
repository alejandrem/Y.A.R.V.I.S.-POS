"""
🗄️ consultas_db.py — Capa de acceso a la base de datos.

Se encarga de:
    - Localizar el archivo yarvis.db en el sistema.
    - Leer productos, ventas, cancelaciones, reembolsos y empleados.
    - Obtener los datos de la tienda (nombre, ubicación).

Ninguna otra responsabilidad: aquí NO hay embeddings, prompts ni endpoints.
"""

import os
import sqlite3
from datetime import datetime, timedelta

# Rutas candidatas donde puede vivir yarvis.db (según el SO)
_RUTAS_DB = [
    os.path.expanduser("~/.local/share/com.yarvis.pos/yarvis.db"),
    os.path.expanduser("~/.local/share/yarvis-app/yarvis.db"),
    os.path.expanduser("~/.config/yarvis-app/yarvis.db"),
    "yarvis.db",
]


def find_db_path() -> str:
    """Localiza yarvis.db; devuelve '' si no existe en ninguna ruta."""
    for path in _RUTAS_DB:
        if os.path.exists(path):
            return path
    return ""


def _conectar():
    """Abre conexión SQLite; devuelve None si no existe la base de datos."""
    db_path = find_db_path()
    if not db_path:
        return None
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    return conn


def cargar_productos() -> list[dict] | None:
    """Devuelve todos los productos ordenados por nombre (para la caché RAG).
    Retorna None si no se encontró la base de datos."""
    conn = _conectar()
    if conn is None:
        return None
    try:
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
    finally:
        conn.close()


def obtener_tienda_info() -> dict:
    """Nombre y ubicación de la tienda desde la cuenta del admin."""
    conn = _conectar()
    if conn is None:
        return {"nombre": "la tienda", "ubicacion": ""}
    try:
        row = conn.execute(
            "SELECT tienda, ubicacion FROM usuarios WHERE rol = 'admin' LIMIT 1"
        ).fetchone()
        if row:
            return {"nombre": row["tienda"] or "la tienda", "ubicacion": row["ubicacion"] or ""}
    except Exception:
        pass
    finally:
        conn.close()
    return {"nombre": "la tienda", "ubicacion": ""}


def obtener_ventas_hoy() -> dict:
    """Tickets y total vendidos hoy."""
    conn = _conectar()
    if conn is None:
        return {"tickets": 0, "total": 0.0}
    try:
        hoy = datetime.now().strftime("%Y-%m-%d")
        v = conn.execute(
            "SELECT COUNT(*) as tickets, COALESCE(SUM(total), 0) as total "
            "FROM ventas WHERE DATE(fecha) = ?", (hoy,)
        ).fetchone()
        return {"tickets": v["tickets"], "total": v["total"]}
    finally:
        conn.close()


def obtener_ventas_7dias() -> dict:
    """Tickets y total vendidos en los últimos 7 días."""
    conn = _conectar()
    if conn is None:
        return {"tickets": 0, "total": 0.0}
    try:
        hace_7 = (datetime.now() - timedelta(days=7)).strftime("%Y-%m-%d")
        v = conn.execute(
            "SELECT COUNT(*) as tickets, COALESCE(SUM(total), 0) as total "
            "FROM ventas WHERE DATE(fecha) >= ?", (hace_7,)
        ).fetchone()
        return {"tickets": v["tickets"], "total": v["total"]}
    finally:
        conn.close()


def obtener_ventas_por_cajero(dias: int = 7) -> list[dict]:
    """Ventas agrupadas por cajero en los últimos N días."""
    conn = _conectar()
    if conn is None:
        return []
    try:
        fecha = (datetime.now() - timedelta(days=dias)).strftime("%Y-%m-%d")
        rows = conn.execute(
            "SELECT cajero, COUNT(*) as n, SUM(total) as t "
            "FROM ventas WHERE DATE(fecha) >= ? GROUP BY cajero ORDER BY t DESC",
            (fecha,)
        ).fetchall()
        return [{"cajero": r["cajero"], "ventas": r["n"], "total": r["t"]} for r in rows]
    finally:
        conn.close()


def obtener_cancelaciones_por_cajero(dias: int = 7) -> list[dict]:
    """Ventas canceladas agrupadas por cajero en los últimos N días."""
    conn = _conectar()
    if conn is None:
        return []
    try:
        fecha = (datetime.now() - timedelta(days=dias)).strftime("%Y-%m-%d")
        rows = conn.execute(
            "SELECT cajero, COUNT(*) as n FROM ventas "
            "WHERE estado = 'cancelada' AND DATE(fecha) >= ? GROUP BY cajero",
            (fecha,)
        ).fetchall()
        return [{"cajero": r["cajero"], "cancelaciones": r["n"]} for r in rows]
    finally:
        conn.close()


def obtener_reembolsos_por_producto(dias: int = 7) -> list[dict]:
    """Productos más devueltos (ventas canceladas) en los últimos N días."""
    conn = _conectar()
    if conn is None:
        return []
    try:
        fecha = (datetime.now() - timedelta(days=dias)).strftime("%Y-%m-%d")
        rows = conn.execute(
            "SELECT dv.producto_nombre, COUNT(*) as n "
            "FROM detalle_ventas dv JOIN ventas v ON dv.venta_id = v.id "
            "WHERE v.estado = 'cancelada' AND DATE(v.fecha) >= ? "
            "GROUP BY dv.producto_nombre ORDER BY n DESC LIMIT 5",
            (fecha,)
        ).fetchall()
        return [{"producto": r["producto_nombre"], "reembolsos": r["n"]} for r in rows]
    finally:
        conn.close()


def obtener_empleados() -> list[dict]:
    """Todos los empleados con su turno, salario, meta y estado."""
    conn = _conectar()
    if conn is None:
        return []
    try:
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
    finally:
        conn.close()
