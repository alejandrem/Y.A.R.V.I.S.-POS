"""
Tests de la capa cloud (modelos_API/apis_cloud.py):
- _cola_modelos_a_probar: relevo acotado de modelos free ante 429.
- generar_stream: recorre la cola (limitada), espera 2-4 s por modelo y avisa.
- generar_stream: si el modelo ya cedió tokens y luego falla, NO cambia de
  modelo (no duplica salida).

NO se hacen llamadas HTTP reales: se simulan los generadores internos.
"""
import httpx
import pytest

from chatbot.motor_chat.modelos_API import apis_cloud as ac
from chatbot.motor_chat.modelos_API.variables import MAX_MODELOS_A_PROBAR, ORDEN_FALLBACK_FREE


def test_cola_opencode_free_acotada_a_MAX_MODELOS():
    """Un modelo free en la lista genera una cola limitada a MAX_MODELOS_A_PROBAR."""
    primero = ORDEN_FALLBACK_FREE[0]
    cola = ac._cola_modelos_a_probar("opencode", primero)
    assert len(cola) <= MAX_MODELOS_A_PROBAR
    assert set(cola) <= set(ORDEN_FALLBACK_FREE)
    assert cola[0] == primero
    # Sin duplicados
    assert len(set(cola)) == len(cola)


def test_cola_opencode_free_modelo_no_listado():
    """Un free no listado se pone primero y sigue con la cola acotada."""
    cola = ac._cola_modelos_a_probar("opencode", "cualquier-libre-free")
    assert cola[0] == "cualquier-libre-free"
    assert len(cola) <= MAX_MODELOS_A_PROBAR
    assert set(cola) == {"cualquier-libre-free", *ORDEN_FALLBACK_FREE[:MAX_MODELOS_A_PROBAR - 1]}


def test_cola_no_opencode_solo_el_modelo():
    """Gemini o modelos pagados: solo el modelo original, sin relevo."""
    assert ac._cola_modelos_a_probar("google", "gemini-2.0-flash") == ["gemini-2.0-flash"]
    assert ac._cola_modelos_a_probar("opencode", "modelo-pago") == ["modelo-pago"]


def _error_429(con_retry_after: str | None = None):
    """Construye un httpx.HTTPStatusError con status 429."""
    request = httpx.Request("POST", "https://test")
    headers = {}
    if con_retry_after is not None:
        headers["retry-after"] = con_retry_after
    response = httpx.Response(429, request=request, headers=headers)
    return httpx.HTTPStatusError("429 Too Many Requests", request=request, response=response)


def test_espera_429_respeta_rango():
    """La espera se clava a [minimo, maximo] aunque retry-after pida más."""
    assert ac._espera_429(_error_429("60"), 2, 4) == 4
    assert ac._espera_429(_error_429("1"), 2, 4) == 2
    assert ac._espera_429(_error_429("3"), 2, 4) == 3
    # Sin retry-after → punto medio del rango
    assert ac._espera_429(_error_429(), 2, 4) == 3


def test_generar_stream_recorre_la_cola_acotada_y_avisa(monkeypatch):
    """Con los modelos probados dando 429 salvo el último, se llega a él."""
    llamados = []

    class _Iterador:
        def __init__(self, modelo):
            self.modelo = modelo

        def __iter__(self):
            llamados.append(self.modelo)
            if self.modelo == ORDEN_FALLBACK_FREE[MAX_MODELOS_A_PROBAR - 1]:
                def _gen():
                    yield "respuesta-final", "OpenCode"
                return _gen()
            raise _error_429()

    def fake_iterar_modelo(provider, cfg, api_key, modelo, messages, display, usage, tools, ejecutar_tool):
        return _Iterador(modelo)

    monkeypatch.setattr(ac, "_iterar_modelo", fake_iterar_modelo)
    monkeypatch.setattr(ac, "_iter_openai_compatible", fake_iterar_modelo)

    avisos: list[str] = []
    tokens = list(ac.generar_stream(
        "opencode", "key", ORDEN_FALLBACK_FREE[0],
        [{"role": "user", "content": "hola"}], avisos=avisos,
    ))

    assert llamados == ORDEN_FALLBACK_FREE[:MAX_MODELOS_A_PROBAR]
    assert [t for t, _ in tokens] == ["respuesta-final"]
    assert avisos, "Debería haber avisos de cambio de modelo"
    assert len(avisos) == MAX_MODELOS_A_PROBAR - 1


def test_generar_stream_no_cambia_si_ya_cedio_tokens(monkeypatch):
    """Si el modelo ya cedió tokens y luego falla, el error se propaga tal cual."""
    llamados = []

    def fake_iterar_modelo(provider, cfg, api_key, modelo, messages, display, usage, tools, ejecutar_tool):
        llamados.append(modelo)

        def _gen():
            yield "parcial-ya-visto", "OpenCode"
            raise _error_429()
        return _gen()

    monkeypatch.setattr(ac, "_iterar_modelo", fake_iterar_modelo)
    monkeypatch.setattr(ac, "_iter_openai_compatible", fake_iterar_modelo)

    avisos: list[str] = []
    gen = ac.generar_stream(
        "opencode", "key", ORDEN_FALLBACK_FREE[0],
        [{"role": "user", "content": "hola"}], avisos=avisos,
    )
    tokens = []
    with pytest.raises(ValueError):
        for tok, _ in gen:
            tokens.append(tok)

    assert len(llamados) == 1
    assert avisos == []