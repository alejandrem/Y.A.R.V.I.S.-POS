"""
Módulo de parsers para catálogos y tickets.
- parser_excel: Parseo de archivos Excel (.xlsx)
- parser_csv: Parseo de archivos CSV
- parser_txt: Parseo de formato visual (texto plano)

Los endpoints HTTP están en endpoints/parser.py (parser_router).
Este módulo solo exporta funciones puras, sin rutas duplicadas.
"""
from parser_py.parser_excel import parsear_excel
from parser_py.parser_csv import parsear_csv
from parser_py.parser_txt import parsear_catalogo_visual
