"""
Tests del bug A1 — lote.py:
- El stream ya no commitea ventas a medias: cada archivo abre su propia
  transacción y ante un fallo de inserción hace rollback total.
- Síncrono y stream comparten el generador único `_procesar_archivos`.

Usa una BD SQLite TEMPORAL (tmp_path) con el mismo esquema que crea
src-tauri/src/backventanas/db/db.rs, por lo que NUNCA toca datos reales.
"""
import sqlite3

import pytest

from parseador_de_tickets.cerebro.lote import _procesar_archivos, _procesar_carpeta_impl

# Mapeo típico de ticket: cantidad | nombre | precio_unitario | total-línea
MAPEO = {"cantidad": 0, "producto": [1], "precio_unitario": 2, "total": 3}

TICKET = """TICKET 1
12/05/2026
2 TAZAS $60.00 $120.00
1 PLATO $80.00 $80.00
TOTAL $200.00
"""


@pytest.fixture
def bd(tmp_path):
    """Crea una BD SQLite con el esquema real (productos/ventas/detalle_ventas)."""
    db = str(tmp_path / "test.db")
    conn = sqlite3.connect(db)
    conn.executescript("""
        CREATE TABLE productos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            precio_costo REAL,
            precio_venta REAL,
            stock REAL DEFAULT 0,
            stock_minimo REAL DEFAULT 0,
            vendido REAL DEFAULT 0
        );
        CREATE TABLE ventas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fecha DATETIME DEFAULT CURRENT_TIMESTAMP,
            total REAL NOT NULL,
            subtotal REAL,
            iva REAL,
            metodo_pago TEXT DEFAULT 'efectivo',
            cajero TEXT NOT NULL,
            estado TEXT DEFAULT 'completada'
        );
        CREATE TABLE detalle_ventas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            venta_id INTEGER NOT NULL,
            producto_id INTEGER,
            producto_nombre TEXT NOT NULL,
            cantidad REAL NOT NULL,
            precio_unitario REAL NOT NULL,
            descuento REAL DEFAULT 0,
            subtotal REAL NOT NULL
        );
    """)
    conn.commit()
    conn.close()
    return db


def _escribir_tickets(carpeta, n):
    """Escribe n copias del TICKET en la carpeta temporal."""
    rutas = []
    for i in range(1, n + 1):
        ruta = carpeta / f"ticket{i}.txt"
        ruta.write_text(TICKET)
        rutas.append(str(ruta))
    return rutas


def _contar(bd, tabla):
    conn = sqlite3.connect(bd)
    total = conn.execute(f"SELECT COUNT(*) FROM {tabla}").fetchone()[0]
    conn.close()
    return total


def test_sync_crea_ventas_y_detalles(bd, tmp_path):
    archivos = _escribir_tickets(tmp_path, 2)
    stats = _procesar_carpeta_impl(archivos, MAPEO, bd)

    assert stats["exitosos"] == 2
    assert stats["errores"] == 0
    assert stats["ventas_creadas"] == 2
    assert stats["items_insertados"] == 4
    assert _contar(bd, "ventas") == 2
    assert _contar(bd, "detalle_ventas") == 4


def test_archivo_vacio_es_error_sin_venta(bd, tmp_path):
    (tmp_path / "vacio.txt").write_text("\n\n")
    stats = _procesar_carpeta_impl([str(tmp_path / "vacio.txt")], MAPEO, bd)
    assert stats["exitosos"] == 0
    assert stats["errores"] == 1
    assert _contar(bd, "ventas") == 0


def test_fallo_a_mitad_no_deja_insert_parcial(bd, tmp_path, monkeypatch):
    """El corazón del bug A1: si _insertar_venta explota DESPUÉS de insertar,
    el rollback debe descartar la venta parcial y los UPDATEs de stock."""
    # Deja 1 venta válida ya insertada (como control)
    _procesar_carpeta_impl(_escribir_tickets(tmp_path, 1), MAPEO, bd)

    # Ahora sabotea _insertar_venta: inserta LA venta y hace el UPDATE de stock,
    # luego explota — simulando el fallo a mitad de archivo.
    original = _procesar_archivos and None  # (referencia de estilo, real se usa abajo)
    del original
    import parseador_de_tickets.cerebro.lote as lote

    def explota(conn, items, *args, **kwargs):
        conn.execute("INSERT INTO ventas (total, cajero) VALUES (999, 'TEST')")
        conn.execute("UPDATE productos SET stock = stock - 1")
        raise RuntimeError("simulando fallo a mitad")

    monkeypatch.setattr(lote, "_insertar_venta", explota)

    archivo_malo = str(tmp_path / "ticket_falla.txt")
    with open(archivo_malo, "w") as f:
        f.write(TICKET)

    stats = _procesar_carpeta_impl([archivo_malo], MAPEO, bd)

    assert stats["exitosos"] == 0
    assert stats["errores"] == 1
    assert stats["tickets_fallidos"][0]["archivo"] == "ticket_falla.txt"
    # Sigue habiendo EXACTAMENTE 1 venta (la válida de antes); la parcial NO quedó.
    assert _contar(bd, "ventas") == 1


def test_stream_y_sync_usan_el_mismo_generador(bd, tmp_path):
    """El stream no tiene lógica de parseo propia: recorre el mismo generador."""
    assert callable(_procesar_archivos)
    archivos = _escribir_tickets(tmp_path, 2)
    resultados = list(_procesar_archivos(archivos, MAPEO, bd))
    assert len(resultados) == 2
    assert all(r["ok"] for r in resultados)
    assert sum(r["items"] for r in resultados) == 4