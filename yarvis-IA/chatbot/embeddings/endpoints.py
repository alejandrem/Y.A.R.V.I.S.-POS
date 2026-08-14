"""
🧠 embeddings/endpoints.py — Endpoints de embeddings y knowledge_base (RAG).

Endpoints:
    /generar_embedding   → vector 384 dims de un texto (BLOB en base64)
    /buscar_similar      → búsqueda semántica en knowledge_base (sqlite-vec)
    /backfill            → genera embeddings del catálogo (robusto a NULLs/schema)
    /insertar_knowledge  → inserta contenido + embedding en knowledge_base

Notas de robustez:
    - Los handlers son `def` (no `async def`) para que FastAPI los ejecute en el
      threadpool y no bloqueen el event loop (model.encode tarda segundos).
    - Toda conexión SQLite se cierra SIEMPRE en `finally` (no hay fugas de FD)
      y se setea `PRAGMA busy_timeout` para escrituras concurrentes.
    - /backfill verifica las columnas reales de `productos` (PRAGMA table_info)
      y usa COALESCE para tolerar NULLs en precios/stock.
"""

import base64
import logging
import sqlite3
from contextlib import closing

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from chatbot.embeddings.modelo import (
    texto_a_embedding,
    textos_a_embeddings,
    embedding_a_blob,
)
from chatbot.motor_chat.modelos_local.motor_rag import buscar_semantico

logger = logging.getLogger("yarvis.embeddings")

router = APIRouter()

# Columnas opcionales que puede (o no) tener la tabla `productos`.
# Si faltan, /backfill usa valores por defecto en vez de crashear.
_COLUMNAS_OPCIONALES = ("descripcion", "categoria", "stock")


def _conectar_db(db_path: str) -> sqlite3.Connection:
    """Abre una conexión SQLite con busy_timeout (escribir/leer concurrente)."""
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    try:
        conn.execute("PRAGMA busy_timeout = 5000")
    except Exception:
        pass
    return conn


def _columnas_existentes(conn: sqlite3.Connection, tabla: str) -> set[str]:
    """Conjunto de nombres de columnas reales de una tabla (PRAGMA table_info)."""
    try:
        return {r["name"] for r in conn.execute(f"PRAGMA table_info({tabla})").fetchall()}
    except Exception as e:
        logger.exception("No se pudo leer el schema de la tabla %s: %s", tabla, e)
        return set()


class EmbeddingRequest(BaseModel):
    texto: str


@router.post("/generar_embedding")
def generar_embedding(request: EmbeddingRequest):
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
        logger.exception("Error generando embedding")
        raise HTTPException(status_code=500, detail=str(e))


class SearchRequest(BaseModel):
    query: str
    top_k: int = 5
    categoria: str | None = None


@router.post("/buscar_similar")
def buscar_similar(request: SearchRequest):
    """Busca los items mas similares en knowledge_base con sqlite-vec.

    La matemática la ejecuta el motor de SQLite en C, no Python:
    vec_distance_cosine() ordena los embeddings por distancia de coseno.
    """
    try:
        resultados = buscar_semantico(request.query, request.top_k, request.categoria)
        return {"status": "ok", "results": resultados}
    except Exception as e:
        logger.exception("Error en búsqueda semántica")
        raise HTTPException(status_code=500, detail=str(e))


class BackfillRequest(BaseModel):
    db_path: str


@router.post("/backfill")
def backfill(request: BackfillRequest):
    """Genera embeddings para TODOS los productos sin embedding en knowledge_base.

    Recorre la tabla `productos`, genera el embedding (nombre + descripción +
    categoría) y lo inserta en knowledge_base. Omite los que ya tienen uno
    (comparando por contenido). Devuelve el conteo de insertados/omitidos.

    Robusto: tolera que falten columnas (descripcion/categoria) y que haya
    NULLs en precios/stock vía COALESCE + PRAGMA table_info.
    """
    try:
        with closing(_conectar_db(request.db_path)) as conn:
            columnas = _columnas_existentes(conn, "productos")

            # Construir el SELECT solo con las columnas que existen de verdad.
            # id/nombre/precio_venta se asumen presentes; lo demás es opcional.
            select_cols = ["id", "nombre", "precio_venta"]
            for c in ("stock", "categoria", "descripcion"):
                if c in columnas:
                    select_cols.append(c)
            select = (
                "SELECT "
                + ", ".join(f"COALESCE({c}, 0) AS {c}" for c in select_cols)
                + " FROM productos"
            )

            # Contenidos ya existentes en knowledge_base (para no duplicar)
            existentes = {
                r["contenido"]
                for r in conn.execute("SELECT contenido FROM knowledge_base").fetchall()
            }

            # Todos los productos
            productos = conn.execute(select).fetchall()

            pendientes = []
            for p in productos:
                precio = p["precio_venta"] or 0
                stock = p["stock"] if "stock" in p.keys() and p["stock"] is not None else 0
                contenido = f"{p['nombre']} | ${precio:.2f} | stock: {stock:.0f}"
                if contenido in existentes:
                    continue
                piezas = [p["nombre"]]
                for c in ("descripcion", "categoria"):
                    if c in select_cols and p[c]:
                        piezas.append(p[c])
                texto = " ".join(piezas)
                categoria = (p["categoria"] if "categoria" in p.keys() else None) or "producto"
                pendientes.append((contenido, categoria, texto))

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

        return {
            "status": "ok",
            "total_productos": len(productos),
            "insertados": insertados,
            "omitidos": len(productos) - insertados,
        }
    except Exception as e:
        logger.exception("Error en backfill")
        raise HTTPException(status_code=500, detail=str(e))


class KnowledgeRequest(BaseModel):
    contenido: str
    categoria: str
    db_path: str


@router.post("/insertar_knowledge")
def insertar_knowledge(request: KnowledgeRequest):
    """Inserta contenido + embedding en knowledge_base."""
    try:
        vec = texto_a_embedding(request.contenido)
        blob = embedding_a_blob(vec)

        with closing(_conectar_db(request.db_path)) as conn:
            conn.execute(
                "INSERT INTO knowledge_base (contenido, categoria, embedding) VALUES (?, ?, ?)",
                (request.contenido, request.categoria, blob)
            )
            conn.commit()

        return {"status": "ok", "dimensions": len(vec)}
    except Exception as e:
        logger.exception("Error insertando knowledge")
        raise HTTPException(status_code=500, detail=str(e))
