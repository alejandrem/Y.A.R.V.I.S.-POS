import re

_PREFIJOSEliminar = re.compile(r'\b(?:ART\.?|COD\.?|CODIGO\.?|SKU\.?|NO\.?|N[°º]\.?)\s*', re.IGNORECASE)
_CODIGO_BARRAS = re.compile(r'\b\d{8,14}\b')
_CARACTERES_RAROS = re.compile(r'[^\w\s\$\.,;:!?\-/%()"]')
_ESPACIOS_MULTIPLES = re.compile(r'\s{2,}')
_PREFIJOS_COMUNES = [
    "BEBIDAS ", "ABARROTES ", "LACTEOS ", "CARNES ", "FRUTAS Y VERDURAS ",
    "LIMPIEZA ", "HIGIENE ", "PANADERIA ", "CHARCUTERIA ", "POLLERIA ",
    "CERVEZAS ", "VINOS Y LICORES ", "BOTANAS ", "ENLATADOS ",
]


def limpiar_producto(nombre: str) -> str:
    """Limpia nombre de producto: elimina codigos de barras, caracteres raros, normaliza espacios, MAYUSCULAS."""
    if not nombre:
        return ""
    limpio = nombre.strip()
    limpio = _CODIGO_BARRAS.sub('', limpio)
    limpio = _PREFIJOSEliminar.sub('', limpio)
    limpio = _CARACTERES_RAROS.sub(' ', limpio)
    limpio = _ESPACIOS_MULTIPLES.sub(' ', limpio).strip().upper()
    for prefijo in _PREFIJOS_COMUNES:
        if limpio.startswith(prefijo):
            limpio = limpio[len(prefijo):]
            break
    return limpio.strip()


def es_categoria(linea: str) -> bool:
    """Detecta si una linea es una categoria (MAYUSCULAS, sin precios, corta)."""
    linea = linea.strip()
    if not linea:
        return False
    if linea.upper() != linea:
        return False
    if '$' in linea or '--' in linea or '=' in linea:
        return False
    if len(linea) > 2 and any(c.isdigit() for c in linea):
        return False
    if len(linea) > 40:
        return False
    return True
