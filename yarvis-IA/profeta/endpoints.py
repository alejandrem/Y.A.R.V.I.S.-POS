from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

router = APIRouter()


class PredictionRequest(BaseModel):
    db_path: str
    days: int = Field(default=7, ge=1, le=365)


@router.post("/recalcular_predicciones")
def recalcular_predicciones(request: PredictionRequest):
    """Ejecuta Prophet y devuelve predicciones para los proximos N dias.

    `def` (no `async def`): Prophet fit es CPU-bound y tarda minutos;
    así FastAPI lo ejecuta en el threadpool y no congela el event loop.
    """
    from .predictor import run_prediction

    result = run_prediction(request.db_path, request.days)
    if "error" in result:
        raise HTTPException(status_code=400, detail=result["error"])
    return result
