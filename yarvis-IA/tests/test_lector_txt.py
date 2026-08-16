"""
Tests de lector_txt.py:
- Bug 8: SIN_SEP ya no roba el separador ("Coca-Cola 600ML -- $25 $18").
- A4: _extraer_nombre_cantidad no se come volúmenes ("COCA-COLA 600", "Agua 1500").
- Formatos con separador `-`/`=`/`--` y tablas SIN separador (con/sin $).
- Catálogo end-to-end (múltiples productos por línea con | y categorías).

El fixture tmp_path es un directorio temporal que pytest crea/limpia solo.
"""
import pytest

from parseador_de_tickets.formatos.lector_txt import (
    _extraer_nombre_cantidad,
    _parsear_linea_catalogo,
    parsear_catalogo_visual,
)


# (línea, nombre esperado, precio_venta esperado)
CASOS_BUG8 = [
    ("Coca-Cola 600ML -- $25 $18", "COCA-COLA 600ML", 25.0),
    ("AGUA 1500 -- $20 $16", "AGUA 1500", 20.0),
    ("PAN BLANCO 12 -- $15 $10", "PAN BLANCO 12", 15.0),
    ("TOTAL -- $1,234.56 $1,000", "TOTAL", 1234.56),
    ("Producto -- $10 $5", "PRODUCTO", 10.0),
    ("Producto - $10 - $5", "PRODUCTO", 10.0),
    ("Producto = $10 $5", "PRODUCTO", 10.0),
]


@pytest.mark.parametrize("linea,nombre,venta", CASOS_BUG8)
def test_bug8_separador_no_contamina_nombre(linea, nombre, venta):
    res = _parsear_linea_catalogo(linea, "")
    assert len(res) == 1, f"{linea!r} debería dar 1 producto, dio {len(res)}"
    assert res[0]["nombre"] == nombre, f"{res[0]['nombre']!r} != {nombre!r}"
    assert res[0]["precio_venta"] == venta


# Tablas SIN separador: el nombre mantiene volúmenes y se lee cantidad/stock
CASOS_SIN_SEP = [
    ("Coca-Cola 600ML $25 $18", "COCA-COLA 600ML", 0),
    ("Coca-Cola 600ML 12 $29 $23", "COCA-COLA 600ML", 12),
    ("Coca-Cola 600ML 12 29 23", "COCA-COLA 600ML", 12),
    ("Sabritas 16 12", "SABRITAS", 0),           # nombre + 2 precios (sin cantidad)
    ("Sabritas 60 16 12", "SABRITAS", 60),        # nombre + cantidad + 2 precios
]


@pytest.mark.parametrize("linea,nombre,stock", CASOS_SIN_SEP)
def test_tablas_sin_separador_con_volumen(linea, nombre, stock):
    res = _parsear_linea_catalogo(linea, "")
    assert res and res[0]["nombre"] == nombre, f"nombre {res[0]['nombre'] if res else 'vacío'!r}"
    assert res[0]["stock"] == stock


def test_cantidad_al_inicio():
    res = _parsear_linea_catalogo("10Producto - $10 $5", "")
    assert res[0]["nombre"] == "PRODUCTO"
    assert res[0]["stock"] == 10


# A4: volúmenes que NO deben separarse como cantidad
@pytest.mark.parametrize("texto,esperado", [
    ("COCA-COLA 600ML 2880", ("COCA-COLA 600ML 2880", 0)),  # termina en volumen → nombre entero
    ("COCA-COLA 600", ("COCA-COLA", 600)),               # ≤999 y sin unidad → SÍ es cantidad (A4)
    ("Coca-Cola 600 ml 12", ("Coca-Cola 600 ml 12", 0)),  # termina en unidad → no tocar
    ("Agua 1500", ("Agua 1500", 0)),                      # >999 → no es cantidad de pieza
])
def test_extraer_nombre_cantidad_no_come_volumen(texto, esperado):
    assert _extraer_nombre_cantidad(texto) == esperado


def test_extraer_nombre_cantidad_si_es_pieza_pequena():
    assert _extraer_nombre_cantidad("Producto 12") == ("Producto", 12)


def test_catalogo_end_to_end(tmp_path):
    texto = """BEBIDAS
Coca-Cola 600ML -- $25 $18 | AGUA 1500 -- $20 $16
ABARROTES
PAN BLANCO 12 -- $15 $10 | SABRITAS 16 12
"""
    productos = parsear_catalogo_visual(texto)
    nombres = [p["nombre"] for p in productos]
    assert nombres == ["COCA-COLA 600ML", "AGUA 1500", "PAN BLANCO 12", "SABRITAS"]
    categorias = [p["categoria"] for p in productos]
    assert categorias == ["BEBIDAS", "BEBIDAS", "ABARROTES", "ABARROTES"]
    assert all(not p["nombre"].endswith("--") for p in productos)