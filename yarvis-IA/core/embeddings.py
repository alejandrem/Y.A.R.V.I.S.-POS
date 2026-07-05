import struct
import numpy as np

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
