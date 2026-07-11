#!/bin/bash
# Y.A.R.V.I.S. Engine Cache Cleanup Script
# Elimina todos los archivos temporales, caches y build artifacts para un fresh start

set -e  # Salta errores para asegurar limpieza completa

echo "🧹 Iniciando limpieza del engine Y.A.R.V.I.S. ingenieria..."

echo "🧹 Limpiando cache de Python...
"
find /home/alesito/Y.A.R.V.I.S.-POS -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
find /home/alesito/Y.A.R.V.I.S.-POS -name "*.pyc" -delete 2>/dev/null || true
find /home/alesito/Y.A.R.V.I.S.-POS -name ".pytest_cache" -type d -exec rm -rf {} + 2>/dev/null || true
find /home/alesito/Y.A.R.V.I.S.-POS -name ".coverage" -delete 2>/dev/null || true
find /home/alesito/Y.A.R.V.I.S.-POS -name ".ruff_cache" -type d -exec rm -rf {} + 2>/dev/null || true
find /home/alesito/Y.A.R.V.I.S.-POS -name ".mypy_cache" -type d -exec rm -rf {} + 2>/dev/null || true
find /home/alesito/Y.A.R.V.I.S.-POS/.venv -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
find /home/alesito/Y.A.R.V.I.S.-POS/.venv -name "*.pyc" -delete 2>/dev/null || true

echo "🧹 Limpiando cache de Rust...
"
find /home/alesito/Y.A.R.V.I.S.-POS -name "Cargo.lock" -delete 2>/dev/null || true
find /home/alesito/Y.A.R.V.I.S.-POS/yarvis-app/src-tauri/target -type d -exec rm -rf {} + 2>/dev/null || true

echo "🧹 Limpiando directorios temporales...
"
rm -rf /tmp/yarvis* 2>/dev/null || true
rm -rf /tmp/temp* 2>/dev/null || true

echo "🧹 Limpiando caches de navegadores y sesiones...
"
echo "✅ Limpieza completada. El sistema está ahora limpio y listo para una nueva sesión."
echo "💡 Próximos pasos:"
echo "1. Ejecuta: cargo check (versión rust limpia)"
echo "2. Ejecuta: npm run tauri dev (frontend limpio)"
echo "3. Limpieza completa y lista para la siguiente importación sin residuos."
