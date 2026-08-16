from fastapi import APIRouter, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel
import sqlite3
import json
import os
import glob
import asyncio

from .analizador import MapeoColumnas, _parsear_linea as parsear_linea, _extraer_fecha_hora_regex, _extraer_metodo_pago
from parseador_de_tickets.llm.analizador_llm import descargar_modelos

router = APIRouter()


def _descargar_todos_los_modelos():
    """Libera RAM/VRAM de todos los modelos Qwen (parser y chat) al terminar un parseo."""
    from chatbot.motor_chat.modelos_local.gestion_hardware import descargar_modelo
    from chatbot.motor_chat.modelos_local.variables import MODELOS

    try:
        descargar_modelos()
    except Exception:
        pass
    for key in MODELOS:
        try:
            descargar_modelo(key)
        except Exception:
            pass


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


def _insertar_venta(conn: sqlite3.Connection, items: list[dict], cajero: str, fecha_iso: str = None, metodo_pago: str = "efectivo") -> int:
    subtotal = _calcular_subtotal(items)
    iva = round(subtotal * 0.16, 2)
    total = round(subtotal + iva, 2)

    if fecha_iso:
        cursor = conn.execute("""
            INSERT INTO ventas (total, subtotal, iva, cajero, metodo_pago, estado, fecha)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        """, (total, subtotal, iva, cajero, metodo_pago, "completada", fecha_iso))
    else:
        cursor = conn.execute("""
            INSERT INTO ventas (total, subtotal, iva, cajero, metodo_pago, estado)
            VALUES (?, ?, ?, ?, ?, ?)
        """, (total, subtotal, iva, cajero, metodo_pago, "completada"))
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


def _procesar_archivos(archivos: list[str], mapeo: dict, db_path: str):
    """Procesa cada archivo con transacción propia; cede un dict por archivo.

    Devuelve por cada archivo algo así:
        {
          "archivo", "ok", "motivo", "items", "duplicados",
          "nuevos", "existentes", "venta_id", "total",
        }

    Es la ÚNICA implementación del bucle de parseo: tanto el modo síncrono
    (/parsear_carpeta) como el streaming (/parsear_carpeta_stream) lo
    recorren, evitando la duplicación de ~80 líneas que ya había divergido.

    Transaccionalidad: cada archivo abre UNA transacción (BEGIN...COMMIT).
    Si _insertar_venta falla a mitad (ej. producto inexistente), se hace
    rollback: la venta parcial y los UPDATEs de stock de ese archivo se
    descartan. NUNCA queda un insert a medias commiteado.
    """
    state = _cargar_estado(db_path)
    conn = sqlite3.connect(db_path)
    mapeo_obj = MapeoColumnas(**mapeo)

    def _info(ok, motivo, **kw):
        base = {
            "archivo": "", "ok": ok, "motivo": motivo, "items": 0,
            "duplicados": 0, "nuevos": [], "existentes": 0,
            "venta_id": None, "total": 0.0,
        }
        base.update(kw)
        return base

    try:
        for archivo in archivos:
            nombre_archivo = os.path.basename(archivo)
            try:
                with open(archivo, "r", encoding="utf-8", errors="ignore") as f:
                    texto = f.read()

                if not texto.strip():
                    yield _info(False, "archivo vacío", archivo=nombre_archivo)
                    continue

                lineas = [l for l in texto.strip().splitlines() if l.strip()]
                if not lineas:
                    yield _info(False, "sin líneas útiles", archivo=nombre_archivo)
                    continue

                total_cols = max(len(l.split()) for l in lineas)
                items = []
                seen = set()
                duplicados = 0
                existentes = 0
                nuevos = []

                for linea in lineas:
                    try:
                        item = parsear_linea(linea, mapeo_obj, total_cols)
                    except Exception as e:
                        print(f"[YARVIS-PARSER] Error parseando línea en {nombre_archivo}: {e}")
                        continue
                    if not item:
                        continue
                    dup_key = f"{item['producto']}|{item.get('precio_unitario', 0):.2f}"
                    if dup_key in seen:
                        duplicados += 1
                        continue
                    seen.add(dup_key)
                    if dup_key in state["productos_db"]:
                        existentes += 1
                    else:
                        state["productos_db"][dup_key] = None
                        nuevos.append({"nombre": item["producto"], "precio": item.get("precio_unitario", 0)})
                    items.append(item)

                if not items:
                    yield _info(False, "ningún producto reconocido con el mapeo actual", archivo=nombre_archivo)
                    continue

                fecha, hora = _extraer_fecha_hora_regex(texto)
                fecha_iso = None
                if fecha:
                    fecha_iso = f"{fecha} {hora}:00" if hora else f"{fecha} 00:00:00"

                cajero = _extraer_cajero(texto)
                metodo_pago = _extraer_metodo_pago(texto)

                try:
                    conn.execute("BEGIN")
                    try:
                        venta_id = _insertar_venta(conn, items, cajero, fecha_iso, metodo_pago)
                        conn.commit()
                    except Exception:
                        conn.rollback()
                        raise
                except Exception as e:
                    yield _info(
                        False, f"error al insertar en DB: {e}",
                        archivo=nombre_archivo, items=len(items),
                        duplicados=duplicados, nuevos=nuevos, existentes=existentes,
                    )
                    continue

                yield _info(
                    True, None,
                    archivo=nombre_archivo, items=len(items), duplicados=duplicados,
                    nuevos=nuevos, existentes=existentes, venta_id=venta_id,
                    total=round(_calcular_subtotal(items) * 1.16, 2),
                )
            except Exception as e:
                print(f"[YARVIS-PARSER] Error inesperado procesando {nombre_archivo}: {e}")
                yield _info(False, f"error inesperado: {e}", archivo=nombre_archivo)
    finally:
        conn.close()


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
        "tickets_fallidos": [],
    }

    nombres_nuevos_vistos: set = set()

    for res in _procesar_archivos(archivos, mapeo, db_path):
        stats["procesados"] += 1
        if res["ok"]:
            stats["exitosos"] += 1
            stats["ventas_creadas"] += 1
            stats["items_insertados"] += res["items"]
            stats["duplicados_detectados"] += res["duplicados"]
            stats["productos_existentes"] += res["existentes"]
            stats["productos_nuevos"] += len(res["nuevos"])
            for nuevo in res["nuevos"]:
                if nuevo["nombre"] not in nombres_nuevos_vistos:
                    nombres_nuevos_vistos.add(nuevo["nombre"])
                    stats["productos_nuevos_lista"].append(nuevo)
            stats["resumen_ventas"].append({
                "archivo": res["archivo"],
                "venta_id": res["venta_id"],
                "items": res["items"],
                "total": res["total"],
            })
        else:
            stats["errores"] += 1
            stats["tickets_fallidos"].append({"archivo": res["archivo"], "motivo": res["motivo"]})

    stats["productos_nuevos_lista"] = stats["productos_nuevos_lista"][:100]
    stats["tickets_fallidos"] = stats["tickets_fallidos"][:500]  # máx 500 para no inflar la respuesta
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

    try:
        stats = _procesar_carpeta_impl(archivos, mapeo, db_path)
    finally:
        _descargar_todos_los_modelos()

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
        tickets_fallidos = []  # lista de {archivo, motivo} para los que no se parsearon

        try:
            # Misma lógica de parseo/inserción que el modo síncrono; aquí solo
            # se acumulan contadores y se emiten eventos de progreso.
            for res in _procesar_archivos(archivos, mapeo, db_path):
                procesados += 1
                if res["ok"]:
                    exitosos += 1
                    ventas_creadas += 1
                    items_insertados += res["items"]
                    duplicados_detectados += res["duplicados"]
                    productos_existentes += res["existentes"]
                    for nuevo in res["nuevos"]:
                        productos_nuevos_set.add(nuevo["nombre"])
                    productos_nuevos += len(res["nuevos"])
                else:
                    errores += 1
                    tickets_fallidos.append({"archivo": res["archivo"], "motivo": res["motivo"]})

                if procesados % 50 == 0 or procesados == total:
                    yield f"data: {json.dumps({'type': 'progress', 'procesados': procesados, 'total': total, 'exitosos': exitosos, 'errores': errores, 'ventas_creadas': ventas_creadas, 'items_insertados': items_insertados, 'productos_nuevos': productos_nuevos, 'productos_existentes': productos_existentes, 'duplicados_detectados': duplicados_detectados})}\n\n"
                    await asyncio.sleep(0.01)

            yield f"data: {json.dumps({'type': 'complete', 'total_archivos': total, 'procesados': procesados, 'exitosos': exitosos, 'errores': errores, 'ventas_creadas': ventas_creadas, 'items_insertados': items_insertados, 'productos_nuevos': productos_nuevos, 'productos_existentes': productos_existentes, 'duplicados_detectados': duplicados_detectados, 'productos_nuevos_lista': list(productos_nuevos_set)[:100], 'tickets_fallidos': tickets_fallidos[:500]})}\n\n"

        finally:
            # Siempre se ejecuta: aunque el cliente cierre la conexión SSE a mitad del proceso
            _descargar_todos_los_modelos()

    return StreamingResponse(
        event_generator(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
        }
    )
