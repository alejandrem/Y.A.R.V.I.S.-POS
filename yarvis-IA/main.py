from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import uvicorn
import sys
import sqlite3
import json
import struct
import numpy as np

# ============================================================
# MODELO DE EMBEDDINGS (sentence-transformers)
# Se carga UNA VEZ al arrancar para no demorar peticiones.
# ============================================================

_embedding_model = None

def get_embedding_model():
    global _embedding_model
    if _embedding_model is None:
        print("[YARVIS-IA] Cargando modelo de embeddings all-MiniLM-L6-v2...")
        from sentence_transformers import SentenceTransformer
        _embedding_model = SentenceTransformer("all-MiniLM-L6-v2")
        print("[YARVIS-IA] Modelo de embeddings listo.")
    return _embedding_model


def texto_a_embedding(texto: str) -> list:
    """Convierte texto a vector de 384 dimensiones."""
    model = get_embedding_model()
    vec = model.encode(texto)
    return vec.tolist()


def embedding_a_blob(vec: list) -> bytes:
    """Serializa un vector a BLOB (384 floats, little-endian)."""
    return struct.pack(f"<{len(vec)}f", *vec)


def blob_a_embedding(blob: bytes) -> list:
    """Deserializa un BLOB a vector."""
    n = len(blob) // 4
    return list(struct.unpack(f"<{n}f", blob))


def cosine_similarity(a: list, b: list) -> float:
    """Calcula similitud coseno entre dos vectores."""
    a_np, b_np = np.array(a), np.array(b)
    dot = np.dot(a_np, b_np)
    norm_a = np.linalg.norm(a_np)
    norm_b = np.linalg.norm(b_np)
    if norm_a == 0 or norm_b == 0:
        return 0.0
    return float(dot / (norm_a * norm_b))


# ============================================================
# FASTAPI
# ============================================================

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8000
app = FastAPI(title="Y.A.R.V.I.S. IA Engine")


@app.get("/")
async def root():
    return {"status": "online", "message": "Y.A.R.V.I.S. AI Brain is running"}


@app.get("/health")
async def health():
    """Health check que Rust consulta al arrancar."""
    return {"status": "ok", "port": PORT}


# ============================================================
# EMBEDDINGS: /generar_embedding
# Rust llama a este endpoint cuando se crea un producto nuevo.
# Recibe texto (nombre + descripción) y devuelve el vector BLOB.
# ============================================================

class EmbeddingRequest(BaseModel):
    texto: str

@app.post("/generar_embedding")
async def generar_embedding(request: EmbeddingRequest):
    """Genera un embedding de 384 dims para el texto dado."""
    try:
        vec = texto_a_embedding(request.texto)
        blob = embedding_a_blob(vec)
        # Devolver como base64 para que Rust lo reciba fácil
        import base64
        return {
            "status": "ok",
            "dimensions": len(vec),
            "blob_b64": base64.b64encode(blob).decode("utf-8")
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# ============================================================
# BÚSQUEDA SEMÁNTICA: /buscar_similar
# Recibe un texto de consulta, busca en la DB los más similares.
# ============================================================

class SearchRequest(BaseModel):
    query: str
    db_path: str
    top_k: int = 5
    categoria: str | None = None

@app.post("/buscar_similar")
async def buscar_similar(request: SearchRequest):
    """Busca los items más similares en knowledge_base usando cosine similarity."""
    try:
        query_vec = texto_a_embedding(request.query)

        conn = sqlite3.connect(request.db_path)
        conn.enable_load_extension(True)
        try:
            import sqlite_vec
            sqlite_vec.load(conn)
        except Exception:
            pass  # Si no hay sqlite-vec, no pasa nada (usamos cosine manual)
        conn.enable_load_extension(False)

        # Obtener todos los embeddings de la categoría (o todos)
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

        # Calcular cosine similarity manualmente
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

        # Ordenar por score descendente y tomar top_k
        resultados.sort(key=lambda x: x["score"], reverse=True)
        return {"status": "ok", "results": resultados[:request.top_k]}

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# ============================================================
# INSERTAR KNOWLEDGE: /insertar_knowledge
# Rust llama para guardar texto + embedding en knowledge_base
# ============================================================

class KnowledgeRequest(BaseModel):
    contenido: str
    categoria: str
    db_path: str

@app.post("/insertar_knowledge")
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


# ============================================================
# RECALCULAR PREDICCIONES: /recalcular_predicciones
# Rust llama cada corte Z o cada 12h para actualizar Prophet.
# ============================================================

class PredictionRequest(BaseModel):
    db_path: str
    days: int = 7

@app.post("/recalcular_predicciones")
async def recalcular_predicciones(request: PredictionRequest):
    """Ejecuta Prophet y devuelve predicciones para los próximos N días."""
    from modelos.profeta.predictor import run_prediction

    result = run_prediction(request.db_path, request.days)
    if "error" in result:
        raise HTTPException(status_code=400, detail=result["error"])
    return result


# ============================================================
# PREDICCIÓN LEGACY: /predict (mantener compatibilidad)
# ============================================================

@app.post("/predict")
async def predict(request: PredictionRequest):
    from modelos.profeta.predictor import run_prediction
    result = run_prediction(request.db_path, request.days)
    if "error" in result:
        raise HTTPException(status_code=400, detail=result["error"])
    return result


# ============================================================
# CHATBOT (Placeholder para Qwen — Paso 4.2)
# ============================================================

@app.post("/chat")
async def chat(message: str):
    return {"response": f"Y.A.R.V.I.S. procesando: {message}"}


# ============================================================
# LOAD LLM (Lazy Loading — Paso 4.1)
# ============================================================

class LoadLLMRequest(BaseModel):
    model: str = "0.5B_Q6"

@app.post("/load_llm")
async def load_llm(request: LoadLLMRequest):
    """Carga el modelo LLM en RAM. Se llama desde Rust al abrir el chatbot."""
    # Placeholder — se implementa en Cuarta Ola
    return {"status": "ok", "model": request.model, "message": "LLM loaded (placeholder)"}


if __name__ == "__main__":
    print(f"[YARVIS-IA] Motor de IA arrancando en puerto {PORT}...")
    print(f"[YARVIS-IA] Endpoints: /health, /generar_embedding, /buscar_similar, /recalcular_predicciones")
    uvicorn.run(app, host="127.0.0.1", port=PORT)
