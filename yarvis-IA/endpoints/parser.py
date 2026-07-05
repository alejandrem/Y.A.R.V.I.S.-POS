"""
Wrapper para compatibilidad con endpoints existentes.
Importa las funciones desde los nuevos módulos parser_py.
"""
from fastapi import APIRouter, HTTPException, Request
from pydantic import BaseModel

from parser_py.parser_excel import parsear_excel
from parser_py.parser_txt import parsear_catalogo_visual
from core.utils import limpiar_producto

router = APIRouter()


class AnalizarTicketRequest(BaseModel):
    texto: str


@router.post("/analizar_ticket")
async def analizar_ticket_endpoint(request: AnalizarTicketRequest):
    """
    Recibe: { "texto": "contenido del ticket .txt" }
    Retorna: { "status": "ok", "mapeo": {...}, "confianza": 0.95 }
    """
    from modelos.qwen.parser_llm import analizar_ticket, descargar_modelos
    try:
        resultado = analizar_ticket(request.texto)
    finally:
        descargar_modelos()
    if resultado.get("status") == "error":
        raise HTTPException(status_code=400, detail=resultado.get("error", "Error desconocido"))
    return resultado


class MapeoColumnas(BaseModel):
    cantidad: int | None = None
    producto: list[int] | None = None
    precio_unitario: int | None = None
    total: int | None = None
    descuento: int | None = None


class ParseRequest(BaseModel):
    texto: str
    mapeo: MapeoColumnas


def _resolver_indice(col: int | None, total_cols: int) -> int | None:
    if col is None:
        return None
    if col < 0:
        return total_cols + col
    return col


def _limpiar_precio(texto: str) -> float:
    """Limpia un string de precio: '$1,234.56' -> 1234.56"""
    if not texto:
        return 0.0
    limpio = texto.replace("$", "").replace(",", "").replace(" ", "")
    try:
        return float(limpio)
    except ValueError:
        return 0.0


def _es_linea_util(linea: str) -> bool:
    """Detecta si una linea es datos de producto o es encabezado/separador."""
    linea_lower = linea.lower().strip()

    if not linea_lower:
        return False

    if all(c in "-=_*~" for c in linea_lower.replace(" ", "")):
        return False

    patrones_skip = [
        "ticket", "factura", "cfdi", "rfc", "serie", "folio",
        "fecha", "hora", "caja", "cajero", "turno",
        "subtotal", "iva", "total a pagar", "cambio",
        "metodo de pago", "forma de pago",
        "nombre", "direccion", "telefono", "correo",
        "gracias", "vuelva", "auxiliar", "copias",
        "articulo", "producto", "descripcion", "cant", "precio",
        "empresa", "razon social", "regimen",
    ]
    for patron in patrones_skip:
        if patron in linea_lower:
            return False

    return True


def _parsear_linea(linea: str, mapeo: MapeoColumnas, total_cols: int) -> dict | None:
    """
    Parsea UNA linea del ticket usando el mapeo del usuario.
    """
    linea = linea.strip()

    if not _es_linea_util(linea):
        return None

    cols = linea.split()
    if len(cols) < 2:
        return None

    idx_cant = _resolver_indice(mapeo.cantidad, total_cols)
    idx_precio = _resolver_indice(mapeo.precio_unitario, total_cols)
    idx_total = _resolver_indice(mapeo.total, total_cols)
    idx_desc = _resolver_indice(mapeo.descuento, total_cols)

    producto = ""
    if mapeo.producto and len(mapeo.producto) >= 2:
        ini = _resolver_indice(mapeo.producto[0], total_cols)
        fin = _resolver_indice(mapeo.producto[-1], total_cols)
        if ini is not None and fin is not None:
            start = max(0, min(ini, fin))
            end = min(len(cols) - 1, max(ini, fin))
            producto = " ".join(cols[start:end + 1])
    elif mapeo.producto and len(mapeo.producto) == 1:
        idx = _resolver_indice(mapeo.producto[0], total_cols)
        if idx is not None and 0 <= idx < len(cols):
            producto = cols[idx]

    cantidad = _limpiar_precio(cols[idx_cant] if idx_cant is not None and idx_cant < len(cols) else "")
    precio = _limpiar_precio(cols[idx_precio] if idx_precio is not None and idx_precio < len(cols) else "")
    total = _limpiar_precio(cols[idx_total] if idx_total is not None and idx_total < len(cols) else "")
    descuento = _limpiar_precio(cols[idx_desc] if idx_desc is not None and idx_desc < len(cols) else "")

    if not producto and cantidad == 0 and total == 0:
        return None

    return {
        "producto": limpiar_producto(producto),
        "cantidad": round(cantidad, 3),
        "precio_unitario": round(precio, 2),
        "precio": round(precio, 2),
        "total": round(total, 2),
        "descuento": round(descuento, 2) if descuento > 0 else None,
    }


@router.post("/parsear_con_mapeo")
async def parsear_con_mapeo(request: ParseRequest):
    """
    Recibe: { "texto": "...", "mapeo": { "cantidad": 0, "producto": [1,2], ... } }
    Retorna: { "status": "ok", "items": [...], "total_lineas": 45, "lineas_parseadas": 43, "errores": [...] }
    """
    texto = request.texto
    mapeo = request.mapeo

    if not texto or not texto.strip():
        return {"status": "error", "error": "El texto esta vacio"}

    lineas = [l for l in texto.strip().splitlines() if l.strip()]

    if not lineas:
        return {"status": "error", "error": "No hay lineas para parsear"}

    total_cols = max(len(l.split()) for l in lineas)

    items = []
    errores = []

    for i, linea in enumerate(lineas, 1):
        try:
            item = _parsear_linea(linea, mapeo, total_cols)
            if item:
                items.append(item)
            else:
                errores.append(f"Linea {i}: formato no reconocido")
        except Exception as e:
            errores.append(f"Linea {i}: {str(e)}")

    return {
        "status": "ok",
        "items": items,
        "total_lineas": len(lineas),
        "lineas_parseadas": len(items),
        "errores": errores[:20],
    }


class CatalogoVisualRequest(BaseModel):
    texto: str


@router.post("/parsear_catalogo_visual")
async def parsear_catalogo_visual_endpoint(request: CatalogoVisualRequest):
    """
    Recibe el texto de un catálogo en formato de tabla visual.
    Retorna: { "status": "ok", "productos": [...], "total": N, "categorias": [...] }
    """
    if not request.texto or not request.texto.strip():
        raise HTTPException(status_code=400, detail="El texto está vacío")
    
    productos = parsear_catalogo_visual(request.texto)
    
    if not productos:
        raise HTTPException(status_code=400, detail="No se encontraron productos en el catálogo")
    
    categorias = list(set(p["categoria"] for p in productos if p.get("categoria")))
    categorias.sort()
    
    return {
        "status": "ok",
        "productos": productos,
        "total": len(productos),
        "categorias": categorias,
    }


@router.post("/parsear_excel")
async def parsear_excel_endpoint(request: Request):
    """
    Recibe bytes de un archivo Excel (.xlsx) en el body.
    Retorna: { "status": "ok", "productos": [...], "total": N, "categorias": [...] }
    """
    body = await request.body()
    if not body:
        raise HTTPException(status_code=400, detail="No se recibió el archivo")
    
    productos = parsear_excel(body)
    
    if not productos:
        raise HTTPException(status_code=400, detail="No se encontraron productos en el archivo Excel")
    
    categorias = list(set(p["categoria"] for p in productos if p.get("categoria")))
    categorias.sort()
    
    return {
        "status": "ok",
        "productos": productos,
        "total": len(productos),
        "categorias": categorias,
    }
