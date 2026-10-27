-- Las aperturas manuales de caja anteriores al cambio usaban el DEFAULT
-- UTC de SQLite. Normalizar solo cajas abiertas; los cortes cerrados no se
-- alteran para conservar su historial.
UPDATE cortes_caja
SET fecha_apertura = datetime(fecha_apertura, 'localtime')
WHERE estado = 'abierto' AND fecha_apertura IS NOT NULL;
