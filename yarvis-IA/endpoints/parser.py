"""
Wrapper para compatibilidad con endpoints existentes.
Importa las funciones desde los nuevos módulos parser_py.
"""
from fastapi import APIRouter, HTTPException, Request
from pydantic import BaseModel
import re

from parser_py.parser_excel import parsear_excel
from parser_py.parser_txt import parsear_catalogo_visual
from core.utils import limpiar_producto

router = APIRouter()


# ---------------------------------------------------------------------------
# Extractor de fecha/hora con regex (fallback si el LLM no detecta)
# ---------------------------------------------------------------------------

# Meses en español
_MESES_ES = {
    "enero": "01", "febrero": "02", "marzo": "03", "abril": "04",
    "mayo": "05", "junio": "06", "julio": "07", "agosto": "08",
    "septiembre": "09", "octubre": "10", "noviembre": "11", "diciembre": "12",
    "jan": "01", "feb": "02", "mar": "03", "apr": "04",
    "may": "05", "jun": "06", "jul": "07", "aug": "08",
    "sep": "09", "oct": "10", "nov": "11", "dec": "12",
}


def _extraer_fecha_hora_regex(texto: str) -> tuple[str | None, str | None]:
    """
    Busca fecha y hora en el texto del ticket usando regex.
    Retorna: (fecha_iso "YYYY-MM-DD" o None, hora "HH:MM" o None)
    """
    fecha = None
    hora = None
    lineas = texto.splitlines()

    # Patrones de fecha ordenados de más específico a menos
    patrones_fecha = [
        # ISO: 2024-03-15
        (r'\b(\d{4})-(\d{2})-(\d{2})\b', lambda m: f"{m.group(1)}-{m.group(2)}-{m.group(3)}"),
        # DD/MM/YYYY o DD/MM/YY
        (r'\b(\d{1,2})/(\d{1,2})/(\d{2,4})\b', lambda m: (
            f"{m.group(3) if len(m.group(3)) == 4 else '20' + m.group(3)}-"
            f"{m.group(2).zfill(2)}-{m.group(1).zfill(2)}"
        )),
        # DD-MM-YYYY
        (r'\b(\d{1,2})-(\d{1,2})-(\d{4})\b', lambda m: (
            f"{m.group(3)}-{m.group(2).zfill(2)}-{m.group(1).zfill(2)}"
        )),
        # "15 de marzo de 2024" o "15 marzo 2024"
        (r'\b(\d{1,2})\s+(?:de\s+)?(enero|febrero|marzo|abril|mayo|junio|julio|agosto|septiembre|octubre|noviembre|diciembre)\s+(?:de\s+)?(\d{4})\b',
         lambda m: f"{m.group(3)}-{_MESES_ES[m.group(2).lower()]}-{m.group(1).zfill(2)}"),
    ]

    # Patrón de hora: HH:MM o HH:MM:SS o H:MM AM/PM
    patron_hora = re.compile(
        r'\b(\d{1,2}):(\d{2})(?::\d{2})?\s*(AM|PM|am|pm)?\b'
    )

    for linea in lineas:
        linea_lower = linea.lower()

        # ¿La línea menciona explícitamente la fecha? (ej: "Fecha: 15/03/2024")
        # Se usa para enriquecer el log: sabremos si fue una línea dedicada o una genérica.
        es_linea_fecha = any(kw in linea_lower for kw in ["fecha", "date", "emitido", "emisión"])

        if fecha is None:
            for patron, formateador in patrones_fecha:
                m = re.search(patron, linea, re.IGNORECASE)
                if m:
                    try:
                        fecha = formateador(m)
                        fuente = "línea de fecha explícita" if es_linea_fecha else "línea genérica"
                        print(f"[YARVIS-PARSER] 📅 Fecha detectada ({fuente}): '{fecha}' ← '{linea.strip()}'")
                    except Exception:
                        pass
                    break

        if hora is None:
            m_hora = patron_hora.search(linea)
            if m_hora:
                h = int(m_hora.group(1))
                mins = m_hora.group(2)
                ampm = m_hora.group(3)
                if ampm and ampm.upper() == "PM" and h < 12:
                    h += 12
                elif ampm and ampm.upper() == "AM" and h == 12:
                    h = 0
                hora = f"{h:02d}:{mins}"
                fuente_hora = "línea de fecha explícita" if es_linea_fecha else "línea genérica"
                print(f"[YARVIS-PARSER] ⏰ Hora detectada ({fuente_hora}): '{hora}' ← '{linea.strip()}'")

        if fecha and hora:
            break

    return fecha, hora


class AnalizarTicketRequest(BaseModel):
    texto: str


@router.post("/analizar_ticket")
async def analizar_ticket_endpoint(request: AnalizarTicketRequest):
    """
    Recibe: { "texto": "contenido del ticket .txt" }
    Retorna: { "status": "ok", "mapeo": {...}, "fecha_ticket": "...", "hora_ticket": "...", "confianza": 0.95 }
    """
    from modelos.qwen.parser_llm import analizar_ticket, descargar_modelos
    try:
        resultado = analizar_ticket(request.texto)
    finally:
        descargar_modelos()

    if resultado.get("status") == "error":
        raise HTTPException(status_code=400, detail=resultado.get("error", "Error desconocido"))

    # --- Fecha y hora: LLM primero, regex como fallback ---
    fecha_llm = resultado.get("fecha_ticket")
    hora_llm = resultado.get("hora_ticket")

    # Limpiar valores "null" en string que el LLM a veces devuelve
    if isinstance(fecha_llm, str) and fecha_llm.lower() in ("null", "none", ""):
        fecha_llm = None
    if isinstance(hora_llm, str) and hora_llm.lower() in ("null", "none", ""):
        hora_llm = None

    fecha_regex, hora_regex = _extraer_fecha_hora_regex(request.texto)

    fecha_final = fecha_llm or fecha_regex
    hora_final = hora_llm or hora_regex

    resultado["fecha_ticket"] = fecha_final
    resultado["hora_ticket"] = hora_final

    # --- Imprimir en terminal para verificacion ---
    print("\n[YARVIS-PARSER] ====== RESULTADO ANALISIS TICKET ======")
    print(f"[YARVIS-PARSER] 📅 Fecha ticket : {fecha_final or 'NO DETECTADA'} (LLM={fecha_llm}, regex={fecha_regex})")
    print(f"[YARVIS-PARSER] ⏰ Hora ticket  : {hora_final or 'NO DETECTADA'} (LLM={hora_llm}, regex={hora_regex})")
    print(f"[YARVIS-PARSER] 🤖 Confianza LLM: {resultado.get('confianza', '?')}")
    print(f"[YARVIS-PARSER] 📦 Modelo usado : {resultado.get('reintentado_con', 'qwen2_5_0_5b')}")
    if resultado.get('ejemplo_parseado'):
        print(f"[YARVIS-PARSER] 🛍️  Productos    : {len(resultado['ejemplo_parseado'])} detectados")
        for item in resultado['ejemplo_parseado'][:3]:
            print(f"[YARVIS-PARSER]   - {item.get('cantidad', '?')}x {item.get('producto', '?')} = ${item.get('total', '?')}")
    print("[YARVIS-PARSER] ==========================================\n")

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
        "caja", "cajero", "turno",
        "subtotal", "iva", "total a pagar", "cambio",
        "metodo de pago", "forma de pago",
        "nombre", "direccion", "telefono", "tel:", "correo",
        "gracias", "vuelva", "auxiliar", "copias",
        "articulo", "producto", "descripcion", "cant", "precio",
        "empresa", "razon social", "regimen",
        # Encabezados de ticket de tienda
        "miscelánea", "miscelanea", "av.", "av ", "calle",
        "colonia", "ciudad", "estado", "cp:", "rfc:",
        "tel ", "tel.", "pagina", "www", "ticket #",
        # Línea de total
        "total", "pago:", "efectivo", "tarjeta",
    ]
    for patron in patrones_skip:
        if patron in linea_lower:
            return False

    # Si la línea termina con un número de 4+ dígitos (ticket number)
    if re.search(r'#?\d{4,}$', linea_lower):
        return False

    return True


def _preprocesar_linea(linea: str) -> str:
    """Une '$' con el número siguiente para evitar columnas separadas."""
    # Reemplazar "$ NUMERO" con "$NUMERO"
    linea = re.sub(r'\$\s+(\d)', r'$\1', linea)
    return linea


def _parsear_linea(linea: str, mapeo: MapeoColumnas, total_cols: int) -> dict | None:
    """
    Parsea UNA linea del ticket usando el mapeo del usuario.
    """
    linea = linea.strip()

    if not _es_linea_util(linea):
        return None

    # Preprocessing: unir $ con el número siguiente
    linea = _preprocesar_linea(linea)

    cols = linea.split()
    if len(cols) < 2:
        return None

    # Usar el largo real de la línea para índices negativos
    line_cols = len(cols)

    idx_cant = _resolver_indice(mapeo.cantidad, line_cols)
    idx_precio = _resolver_indice(mapeo.precio_unitario, line_cols)
    idx_total = _resolver_indice(mapeo.total, line_cols)
    idx_desc = _resolver_indice(mapeo.descuento, line_cols)

    producto = ""
    if mapeo.producto and len(mapeo.producto) >= 2:
        ini = _resolver_indice(mapeo.producto[0], line_cols)
        fin = _resolver_indice(mapeo.producto[-1], line_cols)
        if ini is not None and fin is not None:
            start = max(0, min(ini, fin))
            end = min(len(cols) - 1, max(ini, fin))
            producto = " ".join(cols[start:end + 1])
    elif mapeo.producto and len(mapeo.producto) == 1:
        idx = _resolver_indice(mapeo.producto[0], line_cols)
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
