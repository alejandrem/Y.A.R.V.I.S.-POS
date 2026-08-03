"""
⏱️ cache.py — Caché de inventario, hilos de fondo y contexto inteligente.

Se encarga de:
    - Mantener en memoria el catálogo de productos.
    - Refrescar la caché en un hilo de fondo cada 60 segundos.
    - Construir el "contexto inteligente" que se le inyecta al LLM
      (cruza productos, ventas, empleados y anomalías según la pregunta).

Es el coordinador que une la base de datos (consultas_db) con la búsqueda
semántica (motor_rag). No expone endpoints HTTP.
"""

import re
import threading
import time

from .consultas_db import (
    cargar_productos,
    find_db_path,
    obtener_cancelaciones_por_cajero,
    obtener_empleados,
    obtener_productos_mas_vendidos,
    obtener_reembolsos_por_producto,
    obtener_ventas_7dias,
    obtener_ventas_hoy,
    obtener_ventas_por_cajero,
)
from .motor_rag import buscar_semantico, formatear_producto_compacto

CACHE_REFRESH_INTERVAL = 60  # segundos

_inventory_cache: dict = {}
_cache_lock = threading.Lock()
_cache_last_refresh: float = 0


# ============================================================
# REFRESCO DE CACHÉ
# ============================================================

def _refresh_inventory_cache():
    """Carga todos los productos del catálogo."""
    global _inventory_cache, _cache_last_refresh
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

        with _cache_lock:
            _inventory_cache = new_cache
            _cache_last_refresh = time.time()

        print(f"[YARVIS-CHAT] Cache actualizado: {len(new_cache)} productos.")
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


def _buscar_productos(pregunta: str, top_k: int = 5) -> list[str]:
    """Busca productos relevantes: primero RAG (sqlite-vec) y si la base de
    conocimiento está vacía, respaldo por palabras clave en el catálogo."""
    db_path = find_db_path()
    if db_path:
        try:
            hits = buscar_semantico(db_path, pregunta, top_k=top_k)
            if hits:
                return [h["contenido"] for h in hits]
        except Exception:
            pass

    _STOPWORDS = {
        "que", "para", "con", "los", "las", "una", "unas", "unos",
        "tiene", "tienen", "hay", "en", "de", "del", "como", "cual",
        "cuál", "cuáles", "cuanto", "cuánto", "estan", "están", "son",
        "quiero", "quien", "quién", "dame", "me", "se", "por",
    }
    palabras = [w.lower().strip("¿?.,¡!") for w in pregunta.split()
                if len(w) > 3 and w.lower() not in _STOPWORDS]
    if not palabras:
        return []

    scored = []
    for nombre, info in _inventory_cache.items():
        n_clean = nombre.lower()
        matches = sum(1 for w in palabras if w in n_clean)
        if matches:
            scored.append((matches, nombre, info))
    scored.sort(key=lambda x: (x[0], len(x[1])), reverse=True)

    lines = []
    for _, nombre, info in scored[:top_k]:
        lines.append(formatear_producto_compacto(nombre, info))
    return lines


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
        "venta", "vendí", "vendi", "vende", "venden", "vendidos", "vendidas",
        "gananc", "ingreso", "cobr", "dinero", "efectivo", "tarjeta",
        "transferencia", "ticket", "hoy", "ayer", "semana", "mes", "total",
        "caja", "corte",
    ])
    es_empleado = any(k in preg for k in [
        "empleado", "cajero", "juan", "maría", "maria", "turno",
        "salario", "meta", "reembolso", "cancelación", "cancelacion",
    ])
    es_anomalia = any(k in preg for k in [
        "anomal", "raro", "sospech", "inusual", "robo", "estornad",
        "reembolso", "cancelación", "cancelacion", "fraude",
    ])

    # Preguntas sobre qué puede hacer Y.A.R.V.I.S.
    es_capacidades = any(k in preg for k in [
        "que puedes hacer", "qué puedes hacer", "que haces", "qué haces",
        "capacidades", "funciones", "para que sirves", "para qué sirves",
        "como funcionas", "cómo funcionas", "que sabes", "qué sabes",
        "que eres", "qué eres", "que me puedes", "qué me puedes",
        "ayudarme", "que puedes", "qué puedes", "puedes hacer",
    ])

    # Detectar si busca stock bajo/cero específicamente
    busca_sin_stock = any(k in preg for k in [
        "sin stock", "agotad", "cero", "no hay", "falta", "disponibilidad",
        "no tienen", "no hay de", "ninguno", "cuáles no",
    ])
    # Detectar stock bajo con umbral numérico ("menos de 5", "con menos de X", "pocas unidades")
    umbral_stock = None
    m = re.search(r"menos de (\d+)|con menos de (\d+)|(\d+) (unidad|unidades|piezas|uds)", preg)
    if m:
        umbral_stock = int(next(g for g in m.groups() if g))
    busca_stock_bajo = any(k in preg for k in [
        "por agotarse", "stock bajo", "poco stock", "quedan pocas",
        "casi se agota", "necesito surtir", "queden poc",
        "poco inventario", "pocos",
    ]) or umbral_stock is not None
    busca_mas_vendidos = any(k in preg for k in [
        "mas vendid", "más vendid", "mas vendido", "más vendido",
        "se venden", "se vende", "mejores", "top ventas",
        "productos mas", "productos más", "bestseller", "best seller",
    ])

    # Saludos y mensajes cortos no necesitan productos
    es_saludo = len(preg.split()) <= 2 and not any([es_producto, es_venta, es_empleado, es_anomalia, es_capacidades])

    # Si la pregunta es sobre qué puede hacer Y.A.R.V.I.S., responder capacidades
    if es_capacidades and not es_saludo:
        return (
            "CAPACIDADES DE Y.A.R.V.I.S.:\n"
            "- Consultar productos (precio, stock, categoría, disponibilidad).\n"
            "- Reportar stock bajo o productos agotados.\n"
            "- Reportar ventas de hoy, de la semana y últimos 7 días.\n"
            "- Listar los productos más vendidos.\n"
            "- Reportar empleados, cancelaciones y reembolsos.\n"
            "- Responder preguntas sobre la tienda con datos en tiempo real."
        )

    # Si la pregunta no es saludo ni de ventas/empleados/anomalías, se asume
    # que pregunta por productos: el RAG (sqlite-vec) confirmará los hits.
    es_producto = es_producto or not any([es_saludo, es_venta, es_empleado, es_anomalia])

    # --- Productos por búsqueda semántica ---
    if (es_producto or busca_sin_stock or busca_stock_bajo) and not es_saludo and not busca_mas_vendidos:
        if busca_sin_stock:
            todos_sin_stock = [
                (n, info) for n, info in _inventory_cache.items() if info["stock"] <= 0
            ]
            todos_sin_stock.sort(key=lambda x: x[1]["stock"])
            if todos_sin_stock:
                lines = [f"  {n} | stock: {info['stock']}" for n, info in todos_sin_stock[:5]]
                partes.append(f"SIN STOCK ({len(todos_sin_stock)} total):\n" + "\n".join(lines))
        elif busca_stock_bajo:
            limite = umbral_stock if umbral_stock is not None else 10
            bajos = [
                (n, info) for n, info in _inventory_cache.items() if 0 < info["stock"] <= limite
            ]
            bajos.sort(key=lambda x: x[1]["stock"])
            if bajos:
                lines = [f"  {n} | stock: {info['stock']:.0f}" for n, info in bajos[:15]]
                if len(bajos) > 15:
                    lines.append(f"  ... y {len(bajos) - 15} productos más con stock menor a {limite}.")
                partes.append(f"PRODUCTOS CON STOCK BAJO (menos de {limite} unidades, {len(bajos)} total):\n" + "\n".join(lines))
            else:
                partes.append(f"No hay productos con stock menor a {limite} unidades.")
        else:
            hits = _buscar_productos(pregunta, top_k=5)
            if hits:
                partes.append("PRODUCTOS RELACIONADOS:\n" + "\n".join(hits))
            else:
                partes.append("No encontré productos relacionados.")

    # --- Ventas ---
    if es_venta:
        try:
            if busca_mas_vendidos:
                for r in obtener_productos_mas_vendidos(dias=30, top=8):
                    partes.append(
                        f"  {r['producto']}: {r['cantidad']:.0f} vendidos (${r['total']:.2f})"
                    )
            else:
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
