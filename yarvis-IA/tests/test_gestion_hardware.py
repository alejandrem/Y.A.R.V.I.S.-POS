"""
Tests del bug C5 — gestion_hardware.py cargaba modelos sin lock.

Corregido con `_carga_lock` (threading.Lock) + double-checked locking.
NO se importa llama_cpp en runtime real: se reemplazan los loaders por uno
que cuenta llamadas, para verificar que N threads concurrentes cargan el
modelo exactamente UNA vez.
"""
import importlib
import threading

_spec = importlib.util.find_spec("chatbot.motor_chat.modelos_local.gestion_hardware")


def _cargar_modulo():
    import sys

    m = importlib.util.module_from_spec(_spec)
    sys.modules["chatbot.motor_chat.modelos_local.gestion_hardware"] = m
    _spec.loader.exec_module(m)
    return m


def test_double_checked_locking_una_sola_carga():
    m = _cargar_modulo()

    contador = {"calls": 0}
    lock_contador = threading.Lock()

    def loader_fake():
        with lock_contador:
            contador["calls"] += 1
        return object()

    # Reemplaza los 3 loaders reales por uno que solo cuenta (sin modelo pesado)
    m._LOADERS = {key: (loader_fake, attr) for key, (_, attr) in m._LOADERS.items()}

    N = 20
    barrera = threading.Barrier(N)
    resultados = []
    errores = []

    def worker():
        try:
            barrera.wait()  # dispara los 20 al mismo tiempo
            resultados.append(m.cargar_modelo("0.5B"))
        except Exception as e:  # noqa: BLE001
            errores.append(e)

    hilos = [threading.Thread(target=worker) for _ in range(N)]
    for h in hilos:
        h.start()
    for h in hilos:
        h.join()

    assert not errores
    assert len(resultados) == N
    assert contador["calls"] == 1, f"loader llamado {contador['calls']} veces, debe ser 1"
    assert m._llm_0_5 is not None


def test_descarga_libera_el_modelo():
    m = _cargar_modulo()
    m._llm_0_5 = object()
    m._llm_1_7 = object()
    m.descargar_modelo("0.5B")
    assert m._llm_0_5 is None
    # No debe tocar los demás
    assert m._llm_1_7 is not None