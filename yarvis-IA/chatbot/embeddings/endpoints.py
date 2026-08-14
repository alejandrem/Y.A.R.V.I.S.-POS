from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
import sqlite3
import base64

from chatbot.embeddings.modelo import texto_a_embedding, textos_a_embeddings, embedding_a_blob
from chatbot.motor_chat.motor_rag import buscar_semantico

router = APIRouter()


class EmbeddingRequest(BaseModel):
    texto: str


@router.post("/generar_embedding")
async def generar_embedding(request: EmbeddingRequest):
    """Genera un embedding de 384 dims para el texto dado."""
    try:
        vec = texto_a_embedding(request.texto)
        blob = embedding_a_blob(vec)
        return {
            "status": "ok",
            "dimensions": len(vec),
            "blob_b64": base64.b64encode(blob).decode("utf-8")
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


class SearchRequest(BaseModel):
    query: str
    top_k: int = 5
    categoria: str | None = None


@router.post("/buscar_similar")
async def buscar_similar(request: SearchRequest):
    """Busca los items mas similares en knowledge_base con sqlite-vec.

    La matemática la ejecuta el motor de SQLite en C, no Python:
    vec_distance_cosine() ordena los embeddings por distancia de coseno.
    """
    try:
        resultados = buscar_semantico(request.query, request.top_k, request.categoria)
        return {"status": "ok", "results": resultados}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


class BackfillRequest(BaseModel):
    db_path: str


@router.post("/backfill")
async def backfill(request: BackfillRequest):
    """Genera embeddings para TODOS los productos sin embedding en knowledge_base.

    Recorre la tabla `productos`, genera el embedding (nombre + descripción +
    categoría) y lo inserta en knowledge_base. Omite los que ya tienen uno
    (comparando por contenido). Devuelve el conteo de insertados/omitidos.
    """
    try:
        conn = sqlite3.connect(request.db_path)
        conn.row_factory = sqlite3.Row

        # Contenidos ya existentes en knowledge_base (para no duplicar)
        existentes = {
            r["contenido"]
            for r in conn.execute("SELECT contenido FROM knowledge_base").fetchall()
        }

        # Todos los productos
        productos = conn.execute(
            "SELECT id, nombre, descripcion, precio_venta, stock, categoria FROM productos"
        ).fetchall()

        pendientes = []
        for p in productos:
            contenido = f"{p['nombre']} | ${p['precio_venta']:.2f} | stock: {p['stock']:.0f}"
            if contenido in existentes:
                continue
            texto = " ".join(
                t for t in [p["nombre"], p["descripcion"], p["categoria"]] if t
            )
            pendientes.append((contenido, p["categoria"] or "producto", texto))

        insertados = 0
        if pendientes:
            blobs = [
                embedding_a_blob(v)
                for v in textos_a_embeddings([p[2] for p in pendientes])
            ]
            conn.executemany(
                "INSERT INTO knowledge_base (contenido, categoria, embedding) VALUES (?, ?, ?)",
                [
                    (contenido, categoria, blob)
                    for (contenido, categoria, _), blob in zip(pendientes, blobs)
                ],
            )
            conn.commit()
            insertados = len(pendientes)

        conn.close()
        return {
            "status": "ok",
            "total_productos": len(productos),
            "insertados": insertados,
            "omitidos": len(productos) - insertados,
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


class KnowledgeRequest(BaseModel):
    contenido: str
    categoria: str
    db_path: str


@router.post("/insertar_knowledge")
async def insertar_knowledge(request: KnowledgeRequest):
    """Inserta contenido + embedding en knowledge_base."""
    try:
        vec = texto_a_embedding(request.contenido)
        blob = embedding_a_blob(vec)

        conn = sqlite3.connect(request.db_path)
        conn.execute(
            "INSERT INTO knowledge_base (contenido, categoria, embedding) VALUES (?, ?, ?)",
            (request.contenido, request.categoria, blob)
        )
        conn.commit()
        conn.close()

        return {"status": "ok", "dimensions": len(vec)}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
