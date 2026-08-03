from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
import sqlite3
import base64

from chatbot.embeddings.modelo import texto_a_embedding, embedding_a_blob
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
    db_path: str
    top_k: int = 5
    categoria: str | None = None


@router.post("/buscar_similar")
async def buscar_similar(request: SearchRequest):
    """Busca los items mas similares en knowledge_base con sqlite-vec.

    La matemática la ejecuta el motor de SQLite en C, no Python:
    vec_distance_cosine() ordena los embeddings por distancia de coseno.
    """
    try:
        resultados = buscar_semantico(
            request.db_path, request.query, request.top_k, request.categoria
        )
        return {"status": "ok", "results": resultados}
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
