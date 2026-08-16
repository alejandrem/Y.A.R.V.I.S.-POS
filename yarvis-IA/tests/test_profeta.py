"""
Tests del bug A5 — profeta/endpoints.py validaba days sin rango.

Corregido con pydantic Field(ge=1, le=365) + guardia en run_prediction.
Solo se prueba el contrato de validación (sin correr Prophet, que es lento).
"""
import pytest
from pydantic import ValidationError

from profeta.endpoints import PredictionRequest


def test_days_default_es_7():
    req = PredictionRequest(db_path="/tmp/x.db")
    assert req.days == 7


@pytest.mark.parametrize("days", [0, -1, -5, 366, 1000])
def test_days_fuera_de_rango_rechazado(days):
    with pytest.raises(ValidationError):
        PredictionRequest(db_path="/tmp/x.db", days=days)


@pytest.mark.parametrize("days", [1, 7, 30, 365])
def test_days_validos_aceptados(days):
    req = PredictionRequest(db_path="/tmp/x.db", days=days)
    assert req.days == days