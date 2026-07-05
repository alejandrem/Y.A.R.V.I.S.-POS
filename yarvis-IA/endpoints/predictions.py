from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

router = APIRouter()


class PredictionRequest(BaseModel):
    db_path: str
    days: int = 7


@router.post("/recalcular_predicciones")
async def recalcular_predicciones(request: PredictionRequest):
    """Ejecuta Prophet y devuelve predicciones para los proximos N dias."""
    from modelos.profeta.predictor import run_prediction

    result = run_prediction(request.db_path, request.days)
    if "error" in result:
        raise HTTPException(status_code=400, detail=result["error"])
    return result
