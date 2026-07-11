from fastapi import APIRouter, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel
import sqlite3
import json
import os
import glob
import asyncio

from endpoints.parser import MapeoColumnas, _parsear_linea as parsear_linea
from modelos.qwen.parser_llm import descargar_modelos

router = APIRouter()


class ParseCarpetaRequest(BaseModel):
    carpeta: str
    mapeo: MapeoColumnas
    db_path: str


def _obtener_archivos_txt(carpeta: str) -> list[str]:
    patron = os.path.join(carpeta, "*.txt")
    archivos = glob.glob(patron)
    archivos.sort()
    return archivos


def _calcular_subtotal(items: list[dict]) -> float:
    return sum(i.get("total", 0) or (i.get("cantidad", 0) * i.get("precio_unitario", 0)) for i in items)


def _extraer_cajero(texto: str) -> str:
    for linea in texto.splitlines()[:10]:
        lower = linea.lower()
        if "cajero" in lower or "empleado" in lower or "vendedor" in lower:
            partes = linea.split(":", 1)
            if len(partes) == 2:
                return partes[1].strip()
    return "SISTEMA"


def _insertar_venta(conn: sqlite3.Connection, items: list[dict], cajero: str, archivo: str) -> int:
    subtotal = _calcular_subtotal(items)
    iva = round(subtotal * 0.16, 2)
    total = round(subtotal + iva, 2)

    cursor = conn.execute("""
        INSERT INTO ventas (total, subtotal, iva, cajero, metodo_pago, estado)
        VALUES (?, ?, ?, ?, ?, ?)
    """, (total, subtotal, iva, cajero, "efectivo", "completada"))
    venta_id = cursor.lastrowid

    for item in items:
        cant = item.get("cantidad", 1)
        precio = item.get("precio_unitario", 0)
        desc = item.get("descuento", 0) or 0
        sub = round(cant * precio - desc, 2)

        conn.execute("""
            INSERT INTO detalle_ventas (venta_id, producto_nombre, cantidad, precio_unitario, descuento, subtotal)
            VALUES (?, ?, ?, ?, ?, ?)
        """, (venta_id, item["producto"], cant, precio, desc, sub))

        # Actualizar stock y vendido en productos (case-insensitive)
        conn.execute("""
            UPDATE productos SET stock = stock - ? WHERE LOWER(nombre) = LOWER(?)
        """, (cant, item["producto"]))
        conn.execute("""
            UPDATE productos SET vendido = vendido + ? WHERE LOWER(nombre) = LOWER(?)
        """, (cant, item["producto"]))

    return venta_id


def _cargar_estado(db_path: str) -> dict:
    state = {"productos_db": {}, "duplicados": 0}
    try:
        conn = sqlite3.connect(db_path)
        rows = conn.execute("SELECT id, nombre, precio_venta FROM productos").fetchall()
        for pid, nombre, precio in rows:
            key = f"{nombre.upper().strip()}|{precio:.2f}"
            state["productos_db"][key] = pid
        conn.close()
    except Exception:
        pass
    return state


def _procesar_carpeta_impl(archivos: list[str], mapeo: dict, db_path: str) -> dict:
    stats = {
        "total_archivos": len(archivos),
        "procesados": 0,
        "exitosos": 0,
        "errores": 0,
        "ventas_creadas": 0,
        "items_insertados": 0,
        "productos_nuevos": 0,
        "productos_existentes": 0,
        "duplicados_detectados": 0,
        "productos_nuevos_lista": [],
        "resumen_ventas": [],
    }

    state = _cargar_estado(db_path)

    for archivo in archivos:
        try:
            with open(archivo, "r", encoding="utf-8", errors="ignore") as f:
                texto = f.read()

            if not texto.strip():
                stats["errores"] += 1
                continue

            lineas = [l for l in texto.strip().splitlines() if l.strip()]
            if not lineas:
                stats["errores"] += 1
                continue

            total_cols = max(len(l.split()) for l in lineas)
            items = []
            seen = set()

            for linea in lineas:
                try:
                    item = parsear_linea(linea, MapeoColumnas(**mapeo), total_cols)
                    if item:
                        dup_key = f"{item['producto']}|{item.get('precio_unitario', 0):.2f}"
                        if dup_key in seen:
                            stats["duplicados_detectados"] += 1
                            continue
                        seen.add(dup_key)

                        db_key = dup_key
                        if db_key in state["productos_db"]:
                            stats["productos_existentes"] += 1
                        else:
                            stats["productos_nuevos"] += 1
                            state["productos_db"][db_key] = None
                            if item["producto"] not in [p["nombre"] for p in stats["productos_nuevos_lista"]]:
                                stats["productos_nuevos_lista"].append({
                                    "nombre": item["producto"],
                                    "precio": item.get("precio_unitario", 0),
                                })

                        items.append(item)
                except Exception:
                    pass

            if items:
                try:
                    conn = sqlite3.connect(db_path)
                    cajero = _extraer_cajero(texto)
                    venta_id = _insertar_venta(conn, items, cajero, archivo)
                    conn.commit()
                    conn.close()

                    stats["exitosos"] += 1
                    stats["ventas_creadas"] += 1
                    stats["items_insertados"] += len(items)
                    stats["resumen_ventas"].append({
                        "archivo": os.path.basename(archivo),
                        "venta_id": venta_id,
                        "items": len(items),
                        "total": round(_calcular_subtotal(items) * 1.16, 2),
                    })
                except Exception:
                    stats["errores"] += 1
            else:
                stats["errores"] += 1

        except Exception:
            stats["errores"] += 1

        stats["procesados"] += 1

    stats["productos_nuevos_lista"] = stats["productos_nuevos_lista"][:100]
    return stats


@router.post("/parsear_carpeta")
async def parsear_carpeta(request: ParseCarpetaRequest):
    carpeta = request.carpeta
    mapeo = request.mapeo.model_dump()
    db_path = request.db_path

    if not os.path.isdir(carpeta):
        raise HTTPException(status_code=400, detail=f"Carpeta no encontrada: {carpeta}")

    archivos = _obtener_archivos_txt(carpeta)
    if not archivos:
        raise HTTPException(status_code=400, detail="No se encontraron archivos .txt en la carpeta")

    descargar_modelos()

    stats = _procesar_carpeta_impl(archivos, mapeo, db_path)

    return {
        "status": "ok",
        **stats,
    }


@router.post("/parsear_carpeta_stream")
async def parsear_carpeta_stream(request: ParseCarpetaRequest):
    carpeta = request.carpeta
    mapeo = request.mapeo.model_dump()
    db_path = request.db_path

    if not os.path.isdir(carpeta):
        raise HTTPException(status_code=400, detail=f"Carpeta no encontrada: {carpeta}")

    archivos = _obtener_archivos_txt(carpeta)
    if not archivos:
        raise HTTPException(status_code=400, detail="No se encontraron archivos .txt en la carpeta")

    total = len(archivos)
    state = _cargar_estado(db_path)

    descargar_modelos()

    async def event_generator():
        procesados = 0
        exitosos = 0
        errores = 0
        ventas_creadas = 0
        items_insertados = 0
        productos_nuevos = 0
        productos_existentes = 0
        duplicados_detectados = 0
        productos_nuevos_set = set()

        # UNA sola conexion SQLite para todo el proceso
        conn = sqlite3.connect(db_path)
        try:
            batch_size = 50
            for i in range(0, total, batch_size):
                batch = archivos[i:i + batch_size]

                # Una transaccion por batch (no por archivo)
                conn.execute("BEGIN")

                for archivo in batch:
                    try:
                        with open(archivo, "r", encoding="utf-8", errors="ignore") as f:
                            texto = f.read()

                        if not texto.strip():
                            errores += 1
                            procesados += 1
                            continue

                        lineas = [l for l in texto.strip().splitlines() if l.strip()]
                        if not lineas:
                            errores += 1
                            procesados += 1
                            continue

                        total_cols = max(len(l.split()) for l in lineas)
                        items = []
                        seen = set()

                        for linea in lineas:
                            try:
                                item = parsear_linea(linea, MapeoColumnas(**mapeo), total_cols)
                                if item:
                                    dup_key = f"{item['producto']}|{item.get('precio_unitario', 0):.2f}"
                                    if dup_key in seen:
                                        duplicados_detectados += 1
                                        continue
                                    seen.add(dup_key)

                                    db_key = dup_key
                                    if db_key in state["productos_db"]:
                                        productos_existentes += 1
                                    else:
                                        productos_nuevos += 1
                                        state["productos_db"][db_key] = None
                                        productos_nuevos_set.add(item["producto"])

                                    items.append(item)
                            except Exception:
                                pass

                        if items:
                            try:
                                cajero = _extraer_cajero(texto)
                                _insertar_venta(conn, items, cajero, archivo)
                                exitosos += 1
                                ventas_creadas += 1
                                items_insertados += len(items)
                            except Exception:
                                errores += 1
                        else:
                            errores += 1

                    except Exception:
                        errores += 1

                    procesados += 1

                # Commit al final de cada batch
                conn.commit()

                yield f"data: {json.dumps({'type': 'progress', 'procesados': procesados, 'total': total, 'exitosos': exitosos, 'errores': errores, 'ventas_creadas': ventas_creadas, 'items_insertados': items_insertados, 'productos_nuevos': productos_nuevos, 'productos_existentes': productos_existentes, 'duplicados_detectados': duplicados_detectados})}\n\n"
                await asyncio.sleep(0.01)
        finally:
            conn.close()

        yield f"data: {json.dumps({'type': 'complete', 'total_archivos': total, 'procesados': procesados, 'exitosos': exitosos, 'errores': errores, 'ventas_creadas': ventas_creadas, 'items_insertados': items_insertados, 'productos_nuevos': productos_nuevos, 'productos_existentes': productos_existentes, 'duplicados_detectados': duplicados_detectados, 'productos_nuevos_lista': list(productos_nuevos_set)[:100]})}\n\n"

    return StreamingResponse(
        event_generator(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
        }
    )

