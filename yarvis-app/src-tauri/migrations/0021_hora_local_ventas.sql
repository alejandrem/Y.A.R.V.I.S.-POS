-- Las ventas operativas anteriores usaban el DEFAULT UTC de SQLite,
-- mientras turnos y cortes guardan la hora local. Convertir solo las
-- ventas vinculadas a un cajero; los tickets importados ya traen su hora
-- local de origen y no se deben desplazar.
UPDATE ventas
SET fecha = datetime(fecha, 'localtime')
WHERE cajero_id IS NOT NULL;
