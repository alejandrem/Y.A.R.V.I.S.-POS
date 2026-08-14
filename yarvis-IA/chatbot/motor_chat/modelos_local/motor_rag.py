"""
🔍 motor_rag.py — Motor de búsqueda semántica (RAG) con sqlite-vec.

Se encarga de:
    - Convertir texto en embedding (all-MiniLM-L6-v2, compartido).
    - Buscar los registros más similares en knowledge_base usando
      vec_distance_cosine() de sqlite-vec (el motor de SQLite hace
      la matemática en C, no Python).
    - Formatear productos de forma compacta para inyectarlos al prompt.

El modelo all-MiniLM-L6-v2 se carga UNA sola vez en
chatbot/embeddings/modelo.py y se comparte con todo el sistema;
aquí NO se duplica en memoria.
"""

from ...embeddings.modelo import texto_a_embedding, embedding_a_blob
from .consultas_db import _conectar


def buscar_semantico(
    query: str,
    top_k: int = 5,
    categoria: str | None = None,
) -> list[dict]:
    """Busca los 'top_k' registros más similares en knowledge_base con sqlite-vec.

    La similitud de coseno la calcula SQLite en C vía vec_distance_cosine().
    Devuelve [{contenido, categoria, score}] ordenado por score desc.

    Usa la conexión SQLite reutilizada del hilo (consultas_db._conectar).
    La extensión sqlite-vec se carga UNA vez al conectar (ver
    consultas_db._cargar_extension_vec); aquí solo se ejecuta la consulta.
    """
    query_blob = embedding_a_blob(texto_a_embedding(query))

    conn = _conectar()
    if conn is None:
        return []

    sql = (
        "SELECT contenido, categoria, "
        "       MIN(vec_distance_cosine(embedding, ?)) AS dist "
        "FROM knowledge_base"
    )
    params: list = [query_blob]
    if categoria:
        sql += " WHERE categoria = ?"
        params.append(categoria)
    sql += " GROUP BY contenido ORDER BY dist ASC LIMIT ?"
    params.append(top_k)

    rows = conn.execute(sql, params).fetchall()

    return [
        {
            "contenido": contenido,
            "categoria": categoria_row,
            "score": round(1 - dist, 4),
        }
        for contenido, categoria_row, dist in rows
    ]


def formatear_producto_compacto(nombre: str, info: dict) -> str:
    """Formato compacto: NOMBRE | stock: X | $precio | categoría"""
    return f"{nombre} | stock: {info['stock']} | ${info['precio_venta']:.2f} | {info['categoria']}"
