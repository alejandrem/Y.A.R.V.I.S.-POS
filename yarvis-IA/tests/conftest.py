import os
import sqlite3
import sys

import pytest

# Asegura que 'yarvis-IA' esté en sys.path para importar parseador_de_tickets, chatbot, profeta
_PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if _PROJECT_ROOT not in sys.path:
    sys.path.insert(0, _PROJECT_ROOT)


# Esquema real del sistema (mismo que crea src-tauri/src/backventanas/db/db.rs).
# Solo las tablas que intervienen en los tests.
ESQUEMA_BD = """
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
"""


@pytest.fixture
def bd_temporal(tmp_path):
    """Factory: crea una BD SQLite temporal con el esquema real y devuelve su ruta."""
    creadas = []

    def _crear(nombre: str = "test.db") -> str:
        ruta = str(tmp_path / nombre)
        conn = sqlite3.connect(ruta)
        conn.executescript(ESQUEMA_BD)
        conn.commit()
        conn.close()
        creadas.append(ruta)
        return ruta

    return _crear


def pytest_sessionfinish(session, exitstatus):
    """Limpia cualquier cosa pendiente tras correr la suite."""
    pass