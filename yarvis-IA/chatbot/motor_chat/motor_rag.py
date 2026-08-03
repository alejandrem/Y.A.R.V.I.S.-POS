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

import sqlite3

from ..embeddings.modelo import texto_a_embedding, embedding_a_blob


def buscar_semantico(
    db_path: str,
    query: str,
    top_k: int = 5,
    categoria: str | None = None,
) -> list[dict]:
    """Busca los 'top_k' registros más similares en knowledge_base con sqlite-vec.

    La similitud de coseno la calcula SQLite en C vía vec_distance_cosine().
    Devuelve [{id, contenido, categoria, score}] ordenado por score desc.
    Lanza RuntimeError si la extensión sqlite-vec no está disponible.
    """
    query_blob = embedding_a_blob(texto_a_embedding(query))

    conn = sqlite3.connect(db_path)
    conn.enable_load_extension(True)
    try:
        import sqlite_vec
        sqlite_vec.load(conn)
    except Exception as e:
        conn.close()
        raise RuntimeError(
            f"sqlite-vec no disponible: {e}. Instálalo con: pip install sqlite-vec"
        ) from e
    conn.enable_load_extension(False)

    sql = (
        "SELECT id, contenido, categoria, "
        "       vec_distance_cosine(embedding, ?) AS dist "
        "FROM knowledge_base"
    )
    params: list = [query_blob]
    if categoria:
        sql += " WHERE categoria = ?"
        params.append(categoria)
    sql += " ORDER BY dist ASC LIMIT ?"
    params.append(top_k)

    rows = conn.execute(sql, params).fetchall()
    conn.close()

    return [
        {
            "id": row_id,
            "contenido": contenido,
            "categoria": categoria_row,
            "score": round(1 - dist, 4),
        }
        for row_id, contenido, categoria_row, dist in rows
    ]


def formatear_producto_compacto(nombre: str, info: dict) -> str:
    """Formato compacto: NOMBRE | stock: X | $precio | categoría"""
    return f"{nombre} | stock: {info['stock']} | ${info['precio_venta']:.2f} | {info['categoria']}"
