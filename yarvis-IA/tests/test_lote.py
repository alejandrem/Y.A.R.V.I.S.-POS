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
def bd(bd_temporal):
    """Crea una BD SQLite con el esquema real (productos/ventas/detalle_ventas)."""
    return bd_temporal()


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


def test_rollback_fallo_a_mitad_no_deja_insert_parcial(bd, tmp_path, monkeypatch):
    """El corazón del bug A1: si _insertar_venta explota DESPUÉS de insertar,
    el rollback debe descartar la venta parcial y los UPDATEs de stock."""
    # Deja 1 venta válida ya insertada (como control)
    _procesar_carpeta_impl(_escribir_tickets(tmp_path, 1), MAPEO, bd)

    # Ahora sabotea _insertar_venta: inserta LA venta y hace el UPDATE de stock,
    # luego explota — simulando el fallo a mitad de archivo.
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


def test_linea_duplicada_en_mismo_ticket_no_se_descuenta_dos_veces(bd, tmp_path):
    """Si un ticket repite el mismo producto+precio (misma línea), se cuenta como
    duplicado y NO vuelve a descontar stock 2 veces."""
    ticket_dup = """TICKET 1
12/05/2026
2 TAZAS $60.00 $120.00
2 TAZAS $60.00 $120.00
TOTAL $240.00
"""
    (tmp_path / "dup.txt").write_text(ticket_dup)

    stats = _procesar_carpeta_impl([str(tmp_path / "dup.txt")], MAPEO, bd)

    assert stats["exitosos"] == 1
    assert stats["items_insertados"] == 1
    assert stats["duplicados_detectados"] == 1
    assert _contar(bd, "detalle_ventas") == 1


def test_stock_se_actualiza_con_cantidad_del_item(bd, tmp_path):
    """Con productos preexistentes en productos, el stock baja la cantidad vendida."""
    conn = sqlite3.connect(bd)
    conn.execute("INSERT INTO productos (nombre, stock, vendido) VALUES ('TAZAS', 100, 0)")
    conn.commit()
    conn.close()

    archivo = str(tmp_path / "ticket_stock.txt")
    with open(archivo, "w") as f:
        f.write(TICKET)

    _procesar_carpeta_impl([archivo], MAPEO, bd)

    conn = sqlite3.connect(bd)
    stock, vendido = conn.execute("SELECT stock, vendido FROM productos WHERE nombre = 'TAZAS'").fetchone()
    conn.close()
    assert stock == 98  # 100 - 2
    assert vendido == 2


def test_fallo_en_un_archivo_no_afecta_a_los_demas(bd, tmp_path):
    """Un archivo que no se parsea (sin productos reconocibles) marca error
    SIN impedir que el archivo válido se procese (transacción por archivo)."""
    bueno = str((tmp_path / "bueno.txt"))
    with open(bueno, "w") as f:
        f.write(TICKET)

    # Archivo solo con cabeceras: el parser no encuentra ningún producto.
    solo_cabeceras = str(tmp_path / "cabeceras.txt")
    with open(solo_cabeceras, "w") as f:
        f.write("GRACIAS POR SU COMPRA\nCFDI: 4D8F2A1\n")

    stats = _procesar_carpeta_impl([bueno, solo_cabeceras], MAPEO, bd)

    assert stats["exitosos"] == 1      # el bueno
    assert stats["errores"] == 1       # el de cabeceras
    assert stats["tickets_fallidos"][0]["archivo"] == "cabeceras.txt"
    assert _contar(bd, "ventas") == 1  # solo la venta válida
    assert _contar(bd, "detalle_ventas") == 2