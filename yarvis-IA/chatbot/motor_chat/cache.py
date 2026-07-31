"""
⏱️ cache.py — Caché de inventario, hilos de fondo y contexto inteligente.

Se encarga de:
    - Mantener en memoria el catálogo de productos y sus embeddings.
    - Refrescar la caché en un hilo de fondo cada 60 segundos.
    - Construir el "contexto inteligente" que se le inyecta al LLM
      (cruza productos, ventas, empleados y anomalías según la pregunta).

Es el coordinador que une la base de datos (consultas_db) con la búsqueda
semántica (motor_rag). No expone endpoints HTTP.
"""

import threading
import time

from .consultas_db import (
    cargar_productos,
    obtener_cancelaciones_por_cajero,
    obtener_empleados,
    obtener_reembolsos_por_producto,
    obtener_ventas_7dias,
    obtener_ventas_hoy,
    obtener_ventas_por_cajero,
)
from .motor_rag import buscar_similares, codificar_lista, formatear_producto_compacto

CACHE_REFRESH_INTERVAL = 60  # segundos

_inventory_cache: dict = {}
_inventory_embeddings: list[tuple[str, list[float]]] = []
_cache_lock = threading.Lock()
_cache_last_refresh: float = 0


# ============================================================
# REFRESCO DE CACHÉ
# ============================================================

def _refresh_inventory_cache():
    """Carga todos los productos y pre-calcula sus embeddings."""
    global _inventory_cache, _inventory_embeddings, _cache_last_refresh
    try:
        productos = cargar_productos()
        if productos is None:
            print("[YARVIS-CHAT] No se encontró DB para caché.")
            return

        new_cache = {
            p["nombre"]: {
                "stock": p["stock"],
                "precio_venta": p["precio_venta"],
                "precio_costo": p["precio_costo"],
                "categoria": p["categoria"] or "Sin categoría",
                "stock_minimo": p["stock_minimo"] or 0,
            }
            for p in productos
        }

        nombres = list(new_cache.keys())
        vectores = codificar_lista(nombres)
        new_embeddings = list(zip(nombres, vectores))

        with _cache_lock:
            _inventory_cache = new_cache
            _inventory_embeddings = new_embeddings
            _cache_last_refresh = time.time()

        print(f"[YARVIS-CHAT] Cache actualizado: {len(new_cache)} productos, {len(new_embeddings)} embeddings.")
    except Exception as e:
        print(f"[YARVIS-CHAT] Error refrescando cache: {e}")


def _ensure_cache():
    """Asegura que la caché esté cargada y fresca."""
    global _cache_last_refresh
    if not _inventory_cache or (time.time() - _cache_last_refresh > CACHE_REFRESH_INTERVAL):
        _refresh_inventory_cache()


def _scheduled_refresh():
    """Hilo de fondo: refresca la caché cada CACHE_REFRESH_INTERVAL segundos."""
    while True:
        time.sleep(CACHE_REFRESH_INTERVAL)
        _refresh_inventory_cache()


def iniciar_cache():
    """Arranca la carga inicial de la caché en un hilo de fondo."""
    def _init():
        _refresh_inventory_cache()
        t = threading.Thread(target=_scheduled_refresh, daemon=True)
        t.start()
    threading.Thread(target=_init, daemon=True).start()


def cantidad_productos_cache() -> int:
    """Número de productos en caché (para el estado del modelo)."""
    return len(_inventory_cache)


# ============================================================
# CONTEXTO INTELIGENTE
# ============================================================

def obtener_contexto_inteligente(role: str, pregunta: str) -> str:
    """Construye el contexto que se le inyecta al LLM según el tipo de pregunta."""
    _ensure_cache()
    preg = pregunta.lower()
    partes = []

    # --- Detectar tipo de pregunta ---
    es_producto = any(k in preg for k in [
        "producto", "stock", "artículo", "articulo", "categoria",
        "categoría", "hay", "tengo", "cuántos", "cuantos", "falta",
        "agotad", "surtir", "comprar", "pedido", "inventario",
    ])
    es_venta = any(k in preg for k in [
        "venta", "vendí", "vendi", "gananc", "ingreso", "cobr",
        "dinero", "efectivo", "tarjeta", "transferencia", "ticket",
        "hoy", "ayer", "semana", "mes", "total", "caja", "corte",
    ])
    es_empleado = any(k in preg for k in [
        "empleado", "cajero", "juan", "maría", "maria", "turno",
        "salario", "meta", "reembolso", "cancelación", "cancelacion",
    ])
    es_anomalia = any(k in preg for k in [
        "anomal", "raro", "sospech", "inusual", "robo", "estornad",
        "reembolso", "cancelación", "cancelacion", "fraude",
    ])

    # Detectar si busca stock bajo/cero específicamente
    busca_sin_stock = any(k in preg for k in [
        "sin stock", "agotad", "cero", "no hay", "falta", "disponibilidad",
        "no tienen", "no hay de", "ninguno", "cuáles no",
    ])
    busca_stock_bajo = any(k in preg for k in [
        "por agotarse", "stock bajo", "poco stock", "quedan pocas",
        "casi se agota", "necesito surtir",
    ])

    # Saludos y mensajes cortos no necesitan productos
    es_saludo = len(preg.split()) <= 2 and not any([es_producto, es_venta, es_empleado, es_anomalia])

    # --- Productos por búsqueda semántica ---
    if (es_producto or busca_sin_stock or busca_stock_bajo) and not es_saludo:
        if busca_sin_stock:
            todos_sin_stock = [
                (n, info) for n, info in _inventory_cache.items() if info["stock"] <= 0
            ]
            todos_sin_stock.sort(key=lambda x: x[1]["stock"])
            if todos_sin_stock:
                lines = [f"  {n} | stock: {info['stock']}" for n, info in todos_sin_stock[:5]]
                partes.append(f"SIN STOCK ({len(todos_sin_stock)} total):\n" + "\n".join(lines))
        elif busca_stock_bajo:
            bajos = [
                (n, info) for n, info in _inventory_cache.items() if 0 < info["stock"] <= 10
            ]
            bajos.sort(key=lambda x: x[1]["stock"])
            if bajos:
                lines = [f"  {n} | stock: {info['stock']}" for n, info in bajos[:8]]
                partes.append(f"STOCK BAJO ({len(bajos)} productos):\n" + "\n".join(lines))
            else:
                partes.append("No hay productos con stock bajo.")
        else:
            hits = buscar_similares(pregunta, _inventory_embeddings, top_k=5)
            relevantes = [(n, s) for n, s in hits if s > 0.30]
            if not relevantes:
                relevantes = hits[:3]

            lines = []
            for nombre, score in relevantes:
                info = _inventory_cache.get(nombre)
                if info:
                    lines.append(formatear_producto_compacto(nombre, info))
            if lines:
                partes.append("\n".join(lines))

    # --- Ventas ---
    if es_venta:
        try:
            v_hoy = obtener_ventas_hoy()
            partes.append(f"VENTAS HOY: {v_hoy['tickets']} tickets, ${v_hoy['total']:.2f}")

            v_7 = obtener_ventas_7dias()
            partes.append(f"VENTAS 7 DÍAS: {v_7['tickets']} tickets, ${v_7['total']:.2f}")

            if "cajero" in preg or "empleado" in preg or "quién" in preg:
                for r in obtener_ventas_por_cajero():
                    partes.append(f"  {r['cajero']}: {r['ventas']} ventas, ${r['total']:.2f}")
        except Exception as e:
            partes.append(f"Error ventas: {e}")

    # --- Empleados / Anomalías ---
    if es_empleado or es_anomalia:
        try:
            for r in obtener_cancelaciones_por_cajero():
                partes.append(f"CANCELACIONES - {r['cajero']}: {r['cancelaciones']}")

            for r in obtener_reembolsos_por_producto():
                partes.append(f"REEMBOLSOS - {r['producto']}: {r['reembolsos']}")

            if role == "admin":
                for r in obtener_empleados():
                    partes.append(
                        f"EMPLEADO: {r['nombre']}, turno: {r['turno']}, "
                        f"salario: ${r['salario_semanal']:.0f}/sem, "
                        f"meta: ${r['meta_mensual']:.0f}/mes, estado: {r['estado']}"
                    )
        except Exception as e:
            partes.append(f"Error empleados: {e}")

    return "\n".join(partes) if partes else "Sin datos disponibles."
