"""
Parser de catálogos en formato visual (texto plano .txt).
Soporta múltiples formatos:
  - Producto -- $VENTA $COSTO
  - Producto - $VENTA - $COSTO
  - Producto = $VENTA $COSTO
  - Producto VENTA COSTO (sin $)
  - Múltiples productos por línea con |
  - Cantidad al inicio: 10Producto $10 $5
  - Formato de tabla: Producto  CANT  $VTA  $CST (sin separador)
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

# Patrón para detectar cantidad al inicio de la línea (ej: "10Producto $10 $5")
_PATRON_CANTIDAD_INICIO = re.compile(
    r'^(\d+)\s*'
    r'([A-Za-z0-9áéíóúñüÁÉÍÓÚÑÜ\s\.\-\'\"°®™]+?)'
    r'\s*[-=\*\~>]+\s*'
    r'[\$]?\s*([\d,]+(?:\.\d+)?)?'
    r'(?:\s+[\$]?\s*([\d,]+(?:\.\d+)?)?)?'
)

# Patrones SIN separador (solo espacios): Nombre  CANT  $VTA  $CST
# Patrón 1: Con cantidad y con $ en precios
_PATRON_SIN_SEP_CANT = re.compile(
    r'^(.+?)\s+(\d+)\s+\$([\d,]+(?:\.\d+)?)\s+\$([\d,]+(?:\.\d+)?)'
)
# Patrón 2: Sin cantidad, con $ en precios
_PATRON_SIN_SEP = re.compile(
    r'^(.+?)\s+\$([\d,]+(?:\.\d+)?)\s+\$([\d,]+(?:\.\d+)?)'
)
# Patrón 3: Con cantidad, sin $ en precios
_PATRON_SIN_SEP_CANT_SINDOL = re.compile(
    r'^(.+?)\s+(\d+)\s+([\d,]+(?:\.\d+)?)\s+([\d,]+(?:\.\d+)?)'
)
# Patrón 4: Sin cantidad, sin $
_PATRON_SIN_SEP_SINDOL = re.compile(
    r'^(.+?)\s+([\d,]+(?:\.\d+)?)\s+([\d,]+(?:\.\d+)?)'
)

_PATRON_LINEA_HEADER = re.compile(r'^[\s─]+$|PRODUCTO.*CANT.*VTA|^\s*$')


def _extraer_nombre_cantidad(texto_limpio: str) -> tuple[str, int]:
    """Extrae nombre y cantidad de un texto que termina en número (ej: 'Coca-Cola 600ml 2880')."""
    partes = texto_limpio.rsplit(None, 1)
    if len(partes) == 2:
        posibles_numeros = partes[1].replace(',', '')
        if posibles_numeros.isdigit():
            return partes[0].strip(), int(posibles_numeros)
    return texto_limpio.strip(), 0


def _parsear_linea_catalogo(linea: str, categoria_actual: str) -> list[dict]:
    """Parsea una línea del catálogo que puede contener múltiples productos separados por '|'."""
    productos = []
    
    segmentos = linea.split('|')
    
    for segmento in segmentos:
        segmento = segmento.strip()
        if not segmento:
            continue
        
        # 1. Patrón SIN separador: Nombre  CANT  $VTA  $CST (más específico, primero)
        match = _PATRON_SIN_SEP_CANT.match(segmento)
        if match:
            texto_completo = match.group(1).strip()
            cantidad = int(match.group(2))
            venta_str = match.group(3).replace(',', '')
            costo_str = match.group(4).replace(',', '')
            
            nombre, _ = _extraer_nombre_cantidad(texto_completo)
            try:
                venta = float(venta_str)
                costo = float(costo_str)
            except ValueError:
                continue
            
            if nombre:
                productos.append({
                    "nombre": limpiar_producto(nombre),
                    "precio_costo": round(costo, 2),
                    "precio_venta": round(venta, 2),
                    "stock": cantidad,
                    "categoria": categoria_actual,
                })
                continue
        
        # 2. Patrón SIN separador, SIN cantidad: Nombre  $VTA  $CST
        match = _PATRON_SIN_SEP.match(segmento)
        if match:
            nombre = match.group(1).strip()
            venta_str = match.group(2).replace(',', '')
            costo_str = match.group(3).replace(',', '')
            
            try:
                venta = float(venta_str)
                costo = float(costo_str)
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
                continue
        
        # 3. Patrón SIN separador, con cantidad, sin $: Nombre  CANT  VTA  CST
        match = _PATRON_SIN_SEP_CANT_SINDOL.match(segmento)
        if match:
            texto_completo = match.group(1).strip()
            cantidad = int(match.group(2))
            venta_str = match.group(3).replace(',', '')
            costo_str = match.group(4).replace(',', '')
            
            nombre, _ = _extraer_nombre_cantidad(texto_completo)
            try:
                venta = float(venta_str)
                costo = float(costo_str)
            except ValueError:
                continue
            
            if nombre:
                productos.append({
                    "nombre": limpiar_producto(nombre),
                    "precio_costo": round(costo, 2),
                    "precio_venta": round(venta, 2),
                    "stock": cantidad,
                    "categoria": categoria_actual,
                })
                continue
        
        # 4. Patrón SIN separador, SIN cantidad, sin $: Nombre  VTA  CST
        match = _PATRON_SIN_SEP_SINDOL.match(segmento)
        if match:
            nombre = match.group(1).strip()
            venta_str = match.group(2).replace(',', '')
            costo_str = match.group(3).replace(',', '')
            
            try:
                venta = float(venta_str)
                costo = float(costo_str)
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
                continue
        
        # 5. Intentar cantidad al inicio con separador (ej: "10Producto - $10 $5")
        match_cantidad = _PATRON_CANTIDAD_INICIO.search(segmento)
        if match_cantidad:
            cantidad = int(match_cantidad.group(1))
            nombre = match_cantidad.group(2).strip()
            venta_str = (match_cantidad.group(3) or '').replace(',', '')
            costo_str = (match_cantidad.group(4) or '').replace(',', '')
            
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
                    "stock": cantidad,
                    "categoria": categoria_actual,
                })
                continue
        
        # 6. Patrón con separador explícito (ej: "Producto -- $10 $5")
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
                continue
        
        # 7. Sin ningún patrón: tratar toda la línea como nombre de producto
        nombre = segmento.strip()
        if nombre and not es_categoria(nombre) and len(nombre) > 2:
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
