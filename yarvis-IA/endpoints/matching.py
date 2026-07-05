from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
import sqlite3
import re

from core.embeddings import texto_a_embedding, cosine_similarity

router = APIRouter()

_NORMALIZAR = re.compile(r'[^\w\s]')
_ESPACIOS = re.compile(r'\s+')


def _normalizar(nombre: str) -> str:
    """Normaliza un nombre para comparación: minúsculas, sin especiales, sin espacios extra."""
    limpio = nombre.lower().strip()
    limpio = _NORMALIZAR.sub('', limpio)
    limpio = _ESPACIOS.sub(' ', limpio)
    return limpio


def _cargar_inventario(db_path: str) -> list[dict]:
    """Carga productos del inventario con embedding pre-calculado."""
    productos = []
    try:
        conn = sqlite3.connect(db_path)
        rows = conn.execute(
            "SELECT id, nombre, precio_venta, embedding FROM productos WHERE embedding IS NOT NULL"
        ).fetchall()
        conn.close()

        import struct
        for pid, nombre, precio, blob in rows:
            if blob:
                n = len(blob) // 4
                embedding = list(struct.unpack(f"<{n}f", blob))
            else:
                embedding = None
            productos.append({
                "id": pid,
                "nombre": nombre,
                "nombre_norm": _normalizar(nombre),
                "precio_venta": precio,
                "embedding": embedding,
            })
    except Exception:
        pass
    return productos


def vincular_con_inventario(productos_parseados: list[dict], db_path: str, umbral_similitud: float = 0.85) -> dict:
    """
    Cruza productos parseados con el inventario existente.
    
    Flujo para cada producto parseado:
    1. Coincidencia exacta (nombre normalizado idéntico)
    2. Coincidencia por embedding (similitud coseno > umbral)
    3. Sin vincular → revisión manual
    
    Retorna:
    {
        "vinculados": [...],      # Match exacto o por embedding
        "sin_vincular": [...],    # Requieren revisión manual
        "estadisticas": {
            "total_parseados": N,
            "exactos": N,
            "por_embedding": N,
            "sin_vincular": N
        }
    }
    """
    inventario = _cargar_inventario(db_path)

    # Indexar por nombre normalizado para búsqueda exacta O(1)
    indice_nombre = {}
    for prod in inventario:
        indice_nombre[prod["nombre_norm"]] = prod

    # Pre-calcular embeddings del inventario
    inventario_con_embedding = [p for p in inventario if p["embedding"]]

    vinculados = []
    sin_vincular = []
    estadisticas = {
        "total_parseados": len(productos_parseados),
        "exactos": 0,
        "por_embedding": 0,
        "sin_vincular": 0,
    }

    for parseado in productos_parseados:
        nombre_parseado = parseado.get("producto", "")
        nombre_norm = _normalizar(nombre_parseado)
        precio_parseado = parseado.get("precio_unitario", 0)

        # 1. Coincidencia exacta
        if nombre_norm in indice_nombre:
            prod_db = indice_nombre[nombre_norm]
            vinculados.append({
                "producto_parseado": parseado,
                "producto_db": {
                    "id": prod_db["id"],
                    "nombre": prod_db["nombre"],
                    "precio_venta": prod_db["precio_venta"],
                },
                "tipo_match": "exacto",
                "score": 1.0,
            })
            estadisticas["exactos"] += 1
            continue

        # 2. Búsqueda por embedding
        if inventario_con_embedding:
            try:
                emb_parseado = texto_a_embedding(nombre_parseado)
                mejor_score = 0.0
                mejor_match = None

                for prod_inv in inventario_con_embedding:
                    if prod_inv["embedding"]:
                        score = cosine_similarity(emb_parseado, prod_inv["embedding"])
                        if score > mejor_score:
                            mejor_score = score
                            mejor_match = prod_inv

                if mejor_match and mejor_score >= umbral_similitud:
                    vinculados.append({
                        "producto_parseado": parseado,
                        "producto_db": {
                            "id": mejor_match["id"],
                            "nombre": mejor_match["nombre"],
                            "precio_venta": mejor_match["precio_venta"],
                        },
                        "tipo_match": "embedding",
                        "score": round(mejor_score, 4),
                    })
                    estadisticas["por_embedding"] += 1
                    continue
            except Exception:
                pass

        # 3. Sin vincular
        sin_vincular.append({
            "producto_parseado": parseado,
            "razon": "Sin coincidencia en inventario",
        })
        estadisticas["sin_vincular"] += 1

    return {
        "vinculados": vinculados,
        "sin_vincular": sin_vincular,
        "estadisticas": estadisticas,
    }


class VincularRequest(BaseModel):
    productos: list[dict]
    db_path: str
    umbral: float = 0.85


@router.post("/vincular_inventario")
async def vincular_inventario_endpoint(request: VincularRequest):
    """
    Recibe productos parseados y los cruza con el inventario.
    Retorna productos vinculados y sin vincular.
    """
    if not request.productos:
        raise HTTPException(status_code=400, detail="No hay productos para vincular")

    resultado = vincular_con_inventario(
        request.productos,
        request.db_path,
        request.umbral,
    )
    return {"status": "ok", **resultado}


class GuardarVinculacionRequest(BaseModel):
    vinculaciones: list[dict]
    db_path: str


@router.post("/guardar_vinculacion")
async def guardar_vinculacion(request: GuardarVinculacionRequest):
    """
    Guarda las vinculaciones aprobadas: actualiza producto_id en detalle_ventas.
    """
    if not request.vinculaciones:
        raise HTTPException(status_code=400, detail="No hay vinculaciones para guardar")

    try:
        conn = sqlite3.connect(request.db_path)
        actualizados = 0

        for v in request.vinculaciones:
            detalle_id = v.get("detalle_id")
            producto_id = v.get("producto_id")
            if detalle_id and producto_id:
                conn.execute(
                    "UPDATE detalle_ventas SET producto_id = ? WHERE id = ?",
                    (producto_id, detalle_id)
                )
                actualizados += 1

        conn.commit()
        conn.close()

        return {"status": "ok", "actualizados": actualizados}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
