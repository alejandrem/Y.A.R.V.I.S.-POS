"""
Tests del bug A3 — _es_linea_util descartaba productos legítimos por
substring ("GATORADE TOTAL", "CAJA DE MADERA", "OLIVA", "COLONIA"...).

Corregido con 3 niveles (subcadena segura / word-boundary / columnas).
Revisado: 19 casos (11 productos + 8 cabeceras).
"""
import pytest

from parseador_de_tickets.cerebro.analizador import _es_linea_util


# Produccos legítimos que NO deben ser descartados (True = es útil/dato)
PRODUCTOS = [
    "GATORADE TOTAL 600ML $35.00",
    "CAJA DE MADERA 30X30 $120.00",
    "CAJA DE 24 CERVEZAS $540.00",
    "CANTIMPLORA LITRO $45.00",
    "PRECIOS JUSTOS COMBO $99.00",
    "OLIVA EXTRA VIRGEN 500ML $185.00",
    "COLONIA 900 PERFUME $150.00",
    "COCA-COLA 600ML $25.00",
    "TOTALMAX $18.00",
    "PAN WHITE 680GR $22.00",
    "DIVA PLATINUM $65.00",
]

# Cabeceras/pies que SÍ deben ser descartados (False = no es dato)
CABECERAS = [
    "TOTAL ---- $1,234.56",
    "EFECTIVO $500.00",
    "IVA 16%",
    "SUBTOTAL $1,064.28",
    "GRACIAS POR SU COMPRA",
    "METODO DE PAGO: TARJETA",
    "CFDI: 4D8F2A1",
    "CAJA: 3",
]


@pytest.mark.parametrize("linea", PRODUCTOS)
def test_productos_legitimos_no_se_descartan(linea):
    assert _es_linea_util(linea) is True, f"Producto perdido silenciosamente: {linea!r}"


@pytest.mark.parametrize("linea", CABECERAS)
def test_cabeceras_se_descartan(linea):
    assert _es_linea_util(linea) is False, f"Cabecera tratada como producto: {linea!r}"


def test_linea_vacia():
    assert _es_linea_util("") is False
    assert _es_linea_util("   ") is False


def test_multiples_productos_por_linea():
    # Con 2+ columnas numéricas la línea es producto casi seguro
    assert _es_linea_util("2 TAZAS $60.00 $120.00") is True
    assert _es_linea_util("Coca-Cola 600ML $25 $18") is True