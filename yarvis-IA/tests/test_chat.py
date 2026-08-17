"""
Tests de los endpoints de chat (/chat y /chat_stream) SIN cargar modelos
reales: se mockean cargar_modelo, ejecutar_chat y la nube (generar_completo /
generar_stream) para probar la lógica de orquestación, validación y fallbacks.

Este archivo es el que más mocks necesita, porque el chat real carga Qwen
(llama.cpp) o habla con la nube — nada de eso debe ejecutarse en CI.
"""
import json

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient

# Importamos el módulo de endpoints del chat (validación) sin disparar servidor.
from chatbot.motor_chat import endpoints as ep


@pytest.fixture
def client():
    """App FastAPI con SOLO el router de chat (sin efectos secundarios reales)."""
    app = FastAPI()
    app.include_router(ep.router)
    return TestClient(app)


@pytest.fixture
def fake_local(monkeypatch):
    """Hace que el chat local devuelva una respuesta fija."""
    fake = object()

    def fake_cargar(model_key):
        return fake

    def fake_ejecutar(llm, messages, max_words):
        return "RESPUESTA LOCAL FIJA"

    monkeypatch.setattr(ep, "cargar_modelo", fake_cargar)
    monkeypatch.setattr(ep, "ejecutar_chat", fake_ejecutar)
    monkeypatch.setattr(ep, "construir_mensajes", lambda role, msgs, ultimo: msgs)
    return fake


MSG_VALIDO = [{"role": "user", "content": "hola"}]


def test_chat_sin_mensajes_400(client):
    # messages=[] es válido para pydantic pero el endpoint responde 400
    r = client.post("/chat", json={"messages": [], "role": "admin"})
    assert r.status_code == 400


def test_chat_mensaje_vacio_400(client):
    r = client.post("/chat", json={
        "messages": [{"role": "user", "content": "   "}], "role": "admin",
    })
    assert r.status_code == 400


def test_chat_falta_role_es_422(client):
    # pydantic: role es obligatorio y MSG_VALIDO no lo trae
    r = client.post("/chat", json={"messages": MSG_VALIDO})
    assert r.status_code == 422


def test_chat_local_usa_modelo_0_5(client, fake_local):
    r = client.post("/chat", json={
        "messages": MSG_VALIDO, "role": "admin", "model": "auto",
    })
    assert r.status_code == 200
    data = r.json()
    assert data["response"] == "RESPUESTA LOCAL FIJA"
    assert data["model_used"] == "0.5B"


def test_chat_provider_falla_fallback_local(client, fake_local, monkeypatch):
    """La nube falla (429/red) → _fallback_local responde (B5)."""

    def generar_que_falla(provider, api_key, model, messages, **kw):
        raise RuntimeError("429 Too Many Requests")

    monkeypatch.setattr(ep, "generar_completo", generar_que_falla)
    r = client.post("/chat", json={
        "messages": MSG_VALIDO,
        "role": "admin",
        "provider": "opencode",
        "model": "x-free",
    })
    assert r.status_code == 200
    data = r.json()
    assert data["response"] == "RESPUESTA LOCAL FIJA"
    assert data["model_used"] == "local-fallback"


def test_chat_provider_ok_usuario(client, fake_local, monkeypatch):
    """La nube responde bien → se usa la respuesta del proveedor (sin fallback)."""

    def generar_ok(provider, api_key, model, messages, **kw):
        return "RESPUESTA DE LA NUBE"

    monkeypatch.setattr(ep, "generar_completo", generar_ok)
    monkeypatch.setattr(ep, "nombre_proveedor", lambda p: "OpenCode Zen")
    r = client.post("/chat", json={
        "messages": MSG_VALIDO, "role": "admin", "provider": "opencode", "model": "x-free",
    })
    assert r.status_code == 200
    assert r.json()["response"] == "RESPUESTA DE LA NUBE"


def test_chat_stream_provider_eventos_sse(client, fake_local, monkeypatch):
    """El SSE de la nube emite tokens (token/think) y un evento done."""

    def generar_stream(provider, api_key, model, messages, **kw):
        for tok in ["Hola", " ", "mundo"]:
            yield tok, None

    monkeypatch.setattr(ep, "generar_stream", generar_stream)
    monkeypatch.setattr(ep, "nombre_proveedor", lambda p: "opencode")
    with client.stream("POST", "/chat_stream", json={
        "messages": MSG_VALIDO, "role": "admin", "provider": "opencode", "model": "libre-free",
    }) as resp:
        assert resp.status_code == 200
        body = "".join(resp.iter_text())

    eventos = [json.loads(l[len("data: "):]) for l in body.split("\n\n") if l.startswith("data: ")]
    assert eventos  # al menos un evento
    assert any(e.get("done") for e in eventos)
    assert any("token" in e or "think" in e for e in eventos)


def test_chat_stream_falta_messages_400(client):
    r = client.post("/chat_stream", json={"messages": [], "role": "admin"})
    assert r.status_code == 400


def test_stop_sin_streams(client):
    """/stop sin streams activos devuelve cancelled: False (no rompe)."""
    r = client.post("/stop")
    assert r.status_code == 200
    assert r.json()["status"] == "stopped"


def test_cancel_stream_aislado_por_stream():
    """C1: cancelar un stream id NO afecta a otro registro con distinto id."""
    _registry = ep._registry
    _registry["next_id"] = 0
    _registry["streams"] = {}
    _registry["activos"] = 0

    id_a = ep._nuevo_stream_event()[1]
    id_b = ep._nuevo_stream_event()[1]
    ev_a = ep._registry["streams"].get(list(_registry["streams"])[0]) if ep._registry["streams"] else None
    ids = list(ep._registry["streams"])
    assert len(ids) == 2

    stream_id_b = ids[1]
    res = ep._cancelar_stream(stream_id_b)
    assert res["cancelled"] is True
    assert res["stream_id"] == stream_id_b
    # El evento del stream A sigue sin dispararse
    assert not id_a.is_set()