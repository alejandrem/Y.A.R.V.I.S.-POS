-- ═══════════════════════════════════════════════════════════════════
-- 0007: Limpia cajero contaminado con HORA/FECHA del mismo renglón.
-- Antes el extractor guardaba "MARIA G.     HORA: 09:14:22" completo,
-- creando cientos de "empleados" fantasma (nombre + hora distinta).
-- El código ya corta en esos marcadores; esto repara las filas viejas.
-- Solo toca espacios al final: los nombres se conservan tal cual.
-- ═══════════════════════════════════════════════════════════════════

UPDATE ventas SET cajero = TRIM(SUBSTR(cajero, 1, INSTR(UPPER(cajero), 'HORA:') - 1))
WHERE UPPER(cajero) LIKE '%HORA:%';

UPDATE ventas SET cajero = TRIM(SUBSTR(cajero, 1, INSTR(UPPER(cajero), 'FECHA:') - 1))
WHERE UPPER(cajero) LIKE '%FECHA:%' AND UPPER(cajero) NOT LIKE '%HORA:%';

UPDATE ventas SET cajero = TRIM(cajero) WHERE cajero != TRIM(cajero);
