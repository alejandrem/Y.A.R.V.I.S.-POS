from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
import sqlite3
import base64

from core.embeddings import texto_a_embedding, embedding_a_blob, blob_a_embedding, cosine_similarity

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
    """Busca los items mas similares en knowledge_base usando cosine similarity."""
    try:
        query_vec = texto_a_embedding(request.query)

        conn = sqlite3.connect(request.db_path)
        conn.enable_load_extension(True)
        try:
            import sqlite_vec
            sqlite_vec.load(conn)
        except Exception:
            pass
        conn.enable_load_extension(False)

        if request.categoria:
            rows = conn.execute(
                "SELECT id, contenido, categoria, embedding FROM knowledge_base WHERE categoria = ?",
                (request.categoria,)
            ).fetchall()
        else:
            rows = conn.execute(
                "SELECT id, contenido, categoria, embedding FROM knowledge_base"
            ).fetchall()
        conn.close()

        resultados = []
        for row_id, contenido, categoria, blob in rows:
            if blob is None:
                continue
            stored_vec = blob_a_embedding(blob)
            score = cosine_similarity(query_vec, stored_vec)
            resultados.append({
                "id": row_id,
                "contenido": contenido,
                "categoria": categoria,
                "score": round(score, 4)
            })

        resultados.sort(key=lambda x: x["score"], reverse=True)
        return {"status": "ok", "results": resultados[:request.top_k]}

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
