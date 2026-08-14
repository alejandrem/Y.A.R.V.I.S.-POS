"""
🛠️ herramientas.py — Capa de abstracción de Tools de Y.A.R.V.I.S.

La ÚNICA responsabilidad de este módulo es exponer una función (search_inventory)
que consulte la base de datos de productos. El modelo (local o nube) NUNCA toca
SQL: solo llama a search_inventory().

Se usa en AMBOS modos:
    - LOCAL:  buscar_semantico() del RAG (sqlite-vec) con fallback a LIKE.
    - NUBE:   se registra como tool de function calling (TOOLS_SCHEMA) para que
              OpenCode Zen / Gemini la invoquen cuando el usuario pregunta por
              productos, precios o stock.
"""

import json

from .consultas_db import _conectar
from .motor_rag import buscar_semantico

# Columnas reales de la tabla `productos` (verificado en la DB de yarvis):
# id, nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo,
# codigo_barras, categoria, creado_en, vendido
_CAMPOS_TOOL = ("nombre", "precio_venta", "stock", "categoria", "descripcion")


def _columnas_productos() -> set[str]:
    """Columnas disponibles en la tabla productos (puede variar entre DBs)."""
    conn = _conectar()
    if conn is None:
        return set()
    try:
        filas = conn.execute("PRAGMA table_info(productos)").fetchall()
        return {fila["name"] for fila in filas}
    except Exception:
        return set()


def _select_base(columnas: set[str]) -> tuple[str, bool]:
    """Construye el SELECT base; devuelve (sql, tiene_descripcion)."""
    campos = ["nombre"]
    tiene_descripcion = "descripcion" in columnas
    for c in ("precio_venta", "stock", "categoria"):
        if c in columnas:
            campos.append(f"COALESCE({c}, 0) AS {c}" if c != "categoria" else f"COALESCE({c}, '') AS {c}")
    if tiene_descripcion:
        campos.append("COALESCE(descripcion, '') AS descripcion")
    return ", ".join(campos), tiene_descripcion


def search_inventory(query: str, limit: int = 5) -> list[dict]:
    """Busca productos en el inventario por nombre o para qué sirve.

    1. Intenta búsqueda semántica primero (sqlite-vec sobre knowledge_base).
    2. Si no encuentra nada, cae a LIKE sobre la tabla productos.
    3. Si tampoco hay LIKE, intenta búsqueda sobre categoría/descripción.

    Devuelve [{nombre, precio_venta, stock, categoria, descripcion, score}].
    """
    query = (query or "").strip()
    if not query:
        return []

    resultados = _buscar_semantico(query, limit)
    if resultados:
        return resultados

    resultados = _buscar_like(query, limit)
    if resultados:
        return resultados

    return _buscar_secundario(query, limit)


def _buscar_semantico(query: str, limit: int) -> list[dict]:
    """Búsqueda semántica vía knowledge_base (sqlite-vec)."""
    try:
        rows = buscar_semantico(query, top_k=limit)
    except Exception:
        return []
    out: list[dict] = []
    for r in rows:
        contenido = (r.get("contenido") or "").strip()
        if not contenido:
            continue
        # Contenido de knowledge_base tiene formato: "NOMBRE | $precio | stock: X"
        out.append({
            **_campos_fallback(),
            "nombre": contenido.split("|")[0].strip(),
            "detalle": contenido,
            "score": r.get("score"),
        })
    return out


def _buscar_like(query: str, limit: int) -> list[dict]:
    """Fallback por LIKE en nombre de producto."""
    conn = _conectar()
    if conn is None:
        return []
    columnas = _columnas_productos()
    select, _ = _select_base(columnas)
    try:
        rows = conn.execute(
            f"SELECT {select} FROM productos WHERE nombre LIKE ? COLLATE NOCASE "
            "ORDER BY nombre LIMIT ?",
            (f"%{query}%", limit),
        ).fetchall()
    except Exception:
        return []
    return [_fila_a_dict(r) for r in rows]


def _buscar_secundario(query: str, limit: int) -> list[dict]:
    """Último recurso: por categoría o descripción (tokens sueltos)."""
    conn = _conectar()
    if conn is None:
        return []
    columnas = _columnas_productos()
    select, tiene_descripcion = _select_base(columnas)
    if "categoria" not in columnas:
        return []
    terminos = [t for t in query.replace(",", " ").split() if len(t) >= 2][:3]
    if not terminos:
        return []
    if tiene_descripcion:
        clausula_base = "(categoria LIKE ? COLLATE NOCASE OR descripcion LIKE ? COLLATE NOCASE)"
        params_por = 2
    else:
        clausula_base = "categoria LIKE ? COLLATE NOCASE"
        params_por = 1
    clausulas = " AND ".join(clausula_base for _ in terminos)
    params: list = []
    for t in terminos:
        params.extend([f"%{t}%"] * params_por)
    try:
        rows = conn.execute(
            f"SELECT {select} FROM productos WHERE {clausulas} ORDER BY nombre LIMIT ?",
            [*params, limit],
        ).fetchall()
    except Exception:
        return []
    return [_fila_a_dict(r) for r in rows]


def _fila_a_dict(row) -> dict:
    """Convierte una fila SQLite en el dict de la tool (columnas opcionales)."""
    return {
        "nombre": row["nombre"],
        "precio_venta": float(row["precio_venta"] or 0) if "precio_venta" in row.keys() else 0.0,
        "stock": float(row["stock"] or 0) if "stock" in row.keys() else 0.0,
        "categoria": (row["categoria"] or "Sin categoría") if "categoria" in row.keys() else "Sin categoría",
        "descripcion": (row["descripcion"] or "") if "descripcion" in row.keys() else "",
        "score": None,
    }


def _campos_fallback() -> dict:
    return {
        "nombre": "",
        "precio_venta": 0.0,
        "stock": 0.0,
        "categoria": "Sin categoría",
        "descripcion": "",
        "score": None,
    }


TOOLS_SCHEMA = [
    {
        "type": "function",
        "function": {
            "name": "search_inventory",
            "description": (
                "Busca productos en el inventario de la tienda por nombre o por qué son. "
                "Usa esta herramienta cuando el usuario pregunte por productos, precios, "
                "stock, disponibilidad o qué hay en la tienda."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Nombre del producto o descripción de lo que busca el cliente.",
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Máximo de resultados a devolver (por defecto 5).",
                    },
                },
                "required": ["query"],
            },
        },
    }
]


def ejecutar_tool(tool_call: dict) -> str:
    """Ejecuta la tool que pidió el modelo (modo nube) y devuelve su contenido.

    Recibe un tool_call en formato OpenAI:
        {"id", "type": "function", "function": {"name", "arguments"}}
    y devuelve un JSON string con el resultado para inyectarlo en el mensaje
    de rol 'tool' de la segunda llamada.
    """
    try:
        nombre = tool_call["function"]["name"]
        args = json.loads(tool_call["function"].get("arguments") or "{}")
    except (KeyError, json.JSONDecodeError):
        return json.dumps({"error": "Tool inválida del modelo"}, ensure_ascii=False)

    if nombre == "search_inventory":
        return json.dumps(
            search_inventory(args.get("query", ""), int(args.get("limit", 5))),
            ensure_ascii=False,
            default=str,
        )
    return json.dumps({"error": f"Tool desconocida: {nombre}"}, ensure_ascii=False)