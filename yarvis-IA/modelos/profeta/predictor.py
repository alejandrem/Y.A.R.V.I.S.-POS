import sqlite3
import pandas as pd
from prophet import Prophet
import os

def run_prediction(db_path, days=30):
    if not os.path.exists(db_path):
        return {"error": f"Base de datos no encontrada en {db_path}"}

    try:
        # 1. Conectar a la DB
        conn = sqlite3.connect(db_path)
        
        # 2. Cargar datos de ventas diarias (Agregadas por día)
        # Prophet requiere columnas 'ds' (fecha) y 'y' (valor)
        query = "SELECT strftime('%Y-%m-%d', fecha) as ds, total_ventas as y FROM ventas_diarias ORDER BY fecha ASC"
        df = pd.read_sql_query(query, conn)
        conn.close()

        if len(df) < 5:
            return {"error": "Insuficientes datos históricos para predecir (mínimo 5 días)"}

        # 3. Entrenar modelo Prophet
        m = Prophet(daily_seasonality=True, yearly_seasonality=False)
        m.fit(df)

        # 4. Crear dataframe para el futuro
        future = m.make_future_dataframe(periods=days)
        forecast = m.predict(future)

        # 5. Formatear resultados
        # Solo nos interesan los días futuros
        predictions = forecast.tail(days)[['ds', 'yhat', 'yhat_lower', 'yhat_upper']]
        
        result = []
        for index, row in predictions.iterrows():
            result.append({
                "fecha": row['ds'].strftime('%Y-%m-%d'),
                "prediccion": round(row['yhat'], 2),
                "minimo": round(row['yhat_lower'], 2),
                "maximo": round(row['yhat_upper'], 2)
            })

        return {"status": "success", "data": result}

    except Exception as e:
        return {"error": str(e)}

if __name__ == "__main__":
    # Prueba rapida
    import os
    db_test = os.path.join(os.path.expanduser("~"), ".local", "share", "com.yarvis.app", "yarvis.db")
    print(run_prediction(db_test))
