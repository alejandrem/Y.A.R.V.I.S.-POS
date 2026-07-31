"""
🔍 motor_rag.py — Motor de búsqueda semántica (RAG).

Se encarga de:
    - Convertir textos en vectores (usa el modelo compartido de embeddings).
    - Buscar los productos más parecidos por similitud de coseno.
    - Formatear productos de forma compacta para inyectarlos al prompt.

El modelo all-MiniLM-L6-v2 se carga UNA sola vez en
chatbot/embeddings/modelo.py y se comparte con todo el sistema;
aquí NO se duplica en memoria.
"""

import numpy as np

from ..embeddings.modelo import get_embedding_model


def codificar_lista(textos: list[str]) -> list[list[float]]:
    """Convierte una lista de textos en una lista de vectores."""
    if not textos:
        return []
    return get_embedding_model().encode(textos, show_progress_bar=False).tolist()


def codificar_texto(texto: str) -> list[float]:
    """Convierte un texto en un vector de 384 dimensiones."""
    return get_embedding_model().encode(texto).tolist()


def buscar_similares(
    query: str,
    embeddings: list[tuple[str, list[float]]],
    top_k: int = 8,
) -> list[tuple[str, float]]:
    """Busca los 'top_k' productos más parecidos por similitud de coseno.

    Recibe los pares (nombre, vector) ya calculados; devuelve (nombre, score).
    """
    if not embeddings:
        return []

    q_np = np.array(codificar_texto(query))
    q_norm = np.linalg.norm(q_np)
    if q_norm == 0:
        return []

    scored = []
    for nombre, vec in embeddings:
        v_np = np.array(vec)
        v_norm = np.linalg.norm(v_np)
        if v_norm == 0:
            continue
        sim = float(np.dot(q_np, v_np) / (q_norm * v_norm))
        scored.append((nombre, sim))

    scored.sort(key=lambda x: x[1], reverse=True)
    return scored[:top_k]


def formatear_producto_compacto(nombre: str, info: dict) -> str:
    """Formato compacto: NOMBRE | stock: X | $precio | categoría"""
    return f"{nombre} | stock: {info['stock']} | ${info['precio_venta']:.2f} | {info['categoria']}"
