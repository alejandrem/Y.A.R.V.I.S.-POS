"""
Predictor de ventas con Prophet (Meta).
Lee la tabla 'ventas' de SQLite y agrega por día para entrenar el modelo.
No depende de una tabla ventas_diarias separada.
"""
import sqlite3
import os

import pandas as pd
from prophet import Prophet


# Query de agregación: suma de totales por día desde la tabla ventas
QUERY_VENTAS_DIARIAS = """
    SELECT
        strftime('%Y-%m-%d', fecha) AS ds,
        SUM(total)                  AS y
    FROM ventas
    WHERE fecha IS NOT NULL
      AND fecha != ''
    GROUP BY ds
    ORDER BY ds ASC
"""

# Query alternativa si los items de venta están en una tabla de detalle
QUERY_VENTAS_DETALLE = """
    SELECT
        strftime('%Y-%m-%d', v.fecha) AS ds,
        SUM(vi.total)                 AS y
    FROM ventas v
    JOIN ventas_items vi ON vi.venta_id = v.id
    WHERE v.fecha IS NOT NULL
      AND v.fecha != ''
    GROUP BY ds
    ORDER BY ds ASC
"""


def _detectar_schema(conn: sqlite3.Connection) -> str:
    """Detecta qué tablas existen y elige la query correcta."""
    tablas = {
        row[0] for row in conn.execute(
            "SELECT name FROM sqlite_master WHERE type='table'"
        ).fetchall()
    }
    print(f"[YARVIS-PROFETA] 🗄️  Tablas en DB: {tablas}")

    if "ventas_items" in tablas and "ventas" in tablas:
        print("[YARVIS-PROFETA] 📋 Usando query ventas + ventas_items (detalle)")
        return QUERY_VENTAS_DETALLE
    elif "ventas" in tablas:
        print("[YARVIS-PROFETA] 📋 Usando query ventas directa (total por ticket)")
        return QUERY_VENTAS_DIARIAS
    else:
        raise ValueError(
            "No se encontró tabla 'ventas' en la base de datos. "
            "El parser de tickets debe guardar los datos en esta tabla."
        )


def run_prediction(db_path: str, days: int = 30) -> dict:
    """
    Entrena Prophet con los datos históricos de ventas y predice los próximos N días.

    Args:
        db_path: Ruta a la base de datos SQLite de YARVIS.
        days: Cuántos días hacia adelante predecir.

    Returns:
        {
          "status": "success",
          "datos_historicos": N,
          "fecha_inicio": "YYYY-MM-DD",
          "fecha_fin": "YYYY-MM-DD",
          "data": [{"fecha": ..., "prediccion": ..., "minimo": ..., "maximo": ...}, ...]
        }
    """
    if not os.path.exists(db_path):
        msg = f"Base de datos no encontrada en: {db_path}"
        print(f"[YARVIS-PROFETA] ❌ {msg}")
        return {"error": msg}

    try:
        print(f"\n[YARVIS-PROFETA] ====== INICIANDO PREDICCION ======")
        print(f"[YARVIS-PROFETA] 📂 DB: {db_path}")
        print(f"[YARVIS-PROFETA] 🔮 Días a predecir: {days}")

        conn = sqlite3.connect(db_path)
        query = _detectar_schema(conn)
        df = pd.read_sql_query(query, conn)
        conn.close()

        print(f"[YARVIS-PROFETA] 📊 Registros históricos diarios cargados: {len(df)}")

        if df.empty:
            msg = "La tabla 'ventas' está vacía. Se necesitan tickets parseados para predecir."
            print(f"[YARVIS-PROFETA] ⚠️  {msg}")
            return {"error": msg}

        if len(df) < 5:
            msg = f"Insuficientes datos históricos ({len(df)} días). Se necesitan mínimo 5 días de ventas."
            print(f"[YARVIS-PROFETA] ⚠️  {msg}")
            return {"error": msg}

        # Convertir columna ds a datetime
        df["ds"] = pd.to_datetime(df["ds"])
        df["y"] = pd.to_numeric(df["y"], errors="coerce").fillna(0)

        print(f"[YARVIS-PROFETA] 📅 Rango histórico: {df['ds'].min().date()} → {df['ds'].max().date()}")
        print(f"[YARVIS-PROFETA] 💰 Total ventas promedio/día: ${df['y'].mean():.2f}")
        print(f"[YARVIS-PROFETA] 💰 Máximo día: ${df['y'].max():.2f} | Mínimo día: ${df['y'].min():.2f}")

        # Entrenar Prophet
        print(f"[YARVIS-PROFETA] 🧠 Entrenando Prophet...")
        m = Prophet(
            daily_seasonality=len(df) >= 14,    # solo si hay al menos 2 semanas
            weekly_seasonality=len(df) >= 14,
            yearly_seasonality=len(df) >= 365,
        )
        m.fit(df)

        # Predicción
        future = m.make_future_dataframe(periods=days)
        forecast = m.predict(future)

        predictions = forecast.tail(days)[["ds", "yhat", "yhat_lower", "yhat_upper"]]

        result = []
        for _, row in predictions.iterrows():
            result.append({
                "fecha":      row["ds"].strftime("%Y-%m-%d"),
                "prediccion": round(max(row["yhat"], 0), 2),        # nunca negativo
                "minimo":     round(max(row["yhat_lower"], 0), 2),
                "maximo":     round(max(row["yhat_upper"], 0), 2),
            })

        print(f"[YARVIS-PROFETA] ✅ Prediccion completada:")
        for r in result[:3]:
            print(f"[YARVIS-PROFETA]   📆 {r['fecha']} → ${r['prediccion']} (${r['minimo']} - ${r['maximo']})")
        if len(result) > 3:
            print(f"[YARVIS-PROFETA]   ... y {len(result) - 3} días más")
        print(f"[YARVIS-PROFETA] ==========================================\n")

        return {
            "status":            "success",
            "datos_historicos":  len(df),
            "fecha_inicio":      df["ds"].min().strftime("%Y-%m-%d"),
            "fecha_fin":         df["ds"].max().strftime("%Y-%m-%d"),
            "data":              result,
        }

    except Exception as e:
        print(f"[YARVIS-PROFETA] ❌ Error: {e}")
        return {"error": str(e)}


if __name__ == "__main__":
    db_test = os.path.join(
        os.path.expanduser("~"), ".local", "share", "com.yarvis.app", "yarvis.db"
    )
    print(run_prediction(db_test))
