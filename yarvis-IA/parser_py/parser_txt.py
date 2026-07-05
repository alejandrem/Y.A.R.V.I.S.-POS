"""
Parser de catálogos en formato visual (texto plano .txt).
Soporta múltiples formatos:
  - Producto -- $VENTA $COSTO
  - Producto - $VENTA - $COSTO
  - Producto = $VENTA $COSTO
  - Producto VENTA COSTO (sin $)
  - Múltiples productos por línea con |
"""
import re
from core.utils import limpiar_producto, es_categoria

# Patrón flexible: nombre + separador + precio1 + precio2
_PATRON_PRODUCTO = re.compile(
    r'([A-Za-z0-9áéíóúñüÁÉÍÓÚÑÜ\s\.\-\'\"°®™]+?)'
    r'\s*[-=\*\~>]+\s*'
    r'[\$]?\s*([\d,]+(?:\.\d+)?)?'
    r'(?:\s+[\$]?\s*([\d,]+(?:\.\d+)?)?)?'
)

_PATRON_LINEA_HEADER = re.compile(r'^[\s─]+$|PRODUCTO.*CANT.*VTA|^\s*$')


def _parsear_linea_catalogo(linea: str, categoria_actual: str) -> list[dict]:
    """Parsea una línea del catálogo que puede contener múltiples productos separados por '|'."""
    productos = []
    
    segmentos = linea.split('|')
    
    for segmento in segmentos:
        segmento = segmento.strip()
        if not segmento:
            continue
        
        match = _PATRON_PRODUCTO.search(segmento)
        if match:
            nombre = match.group(1).strip()
            venta_str = (match.group(2) or '').replace(',', '')
            costo_str = (match.group(3) or '').replace(',', '')
            
            try:
                venta = float(venta_str) if venta_str else 0
                costo = float(costo_str) if costo_str else 0
            except ValueError:
                continue
            
            if nombre:
                productos.append({
                    "nombre": limpiar_producto(nombre),
                    "precio_costo": round(costo, 2),
                    "precio_venta": round(venta, 2),
                    "stock": 0,
                    "categoria": categoria_actual,
                })
        else:
            # Sin separador: tratar toda la línea como nombre de producto
            nombre = segmento.strip()
            if nombre and not es_categoria(nombre) and len(nombre) > 2:
                # Verificar que no sea solo números
                clean = nombre.replace('$', '').replace(',', '').replace('.', '').strip()
                if not clean.isdigit():
                    productos.append({
                        "nombre": limpiar_producto(nombre),
                        "precio_costo": 0,
                        "precio_venta": 0,
                        "stock": 0,
                        "categoria": categoria_actual,
                    })
    
    return productos


def _parsear_visual(texto: str) -> list[dict]:
    """
    Parsea un catálogo en formato de tabla visual.
    
    Formato esperado:
        CATEGORIA
        Producto 1 -- $VENTA $COSTO | Producto 2 -- $VENTA $COSTO
        ...
    """
    productos = []
    categoria_actual = "SIN CATEGORÍA"
    
    for linea in texto.splitlines():
        linea_limpia = linea.strip()
        
        if not linea_limpia or _PATRON_LINEA_HEADER.match(linea_limpia):
            continue
        
        if es_categoria(linea_limpia):
            categoria_actual = linea_limpia.rstrip(':').strip()
            continue
        
        productos_linea = _parsear_linea_catalogo(linea_limpia, categoria_actual)
        productos.extend(productos_linea)
    
    return productos


def parsear_catalogo_visual(texto: str) -> list[dict]:
    """
    Parsea un catálogo detectando automáticamente el formato:
    - CSV (con , o ; como separador)
    - Formato visual (con --, -, =, > como separador)
    """
    from parser_py.parser_csv import _detectar_separador_csv, parsear_csv
    
    if not texto or not texto.strip():
        return []
    
    # Detectar si es CSV
    primera_linea = texto.strip().splitlines()[0]
    if _detectar_separador_csv(primera_linea):
        return parsear_csv(texto)
    
    # Si no es CSV, usar parser visual
    return _parsear_visual(texto)
