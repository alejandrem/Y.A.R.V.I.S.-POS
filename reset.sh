#!/bin/bash
# Y.A.R.V.I.S. POS - Reset completo
# Borra: usuarios, tickets, inventario, cache
# Conserva: codigo fuente, modelos IA, .venv, .git

set -e

echo "🧹 Iniciando Y.A.R.V.I.S. reset..."

echo "🗃️ 1/4 - Borrando base de datos..."
DB_FILE="/home/alesito/.local/share/com.yarvis.pos/yarvis.db"
if [ -f "$DB_FILE" ]; then
    rm -f "$DB_FILE" "$DB_FILE-shm" "$DB_FILE-wal"
    echo "✅ Base de datos eliminada"
else
    echo "⚠️ No se encontro la base de datos"
fi

echo "🔍 2/4 - Limpiando cache de Python..."
find /home/alesito/Y.A.R.V.I.S.-POS -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
find /home/alesito/Y.A.R.V.I.S.-POS -name "*.pyc" -delete 2>/dev/null || true
echo "✅ Cache de Python limpia"

echo "🌐 3/4 - Limpiando cache del navegador..."
rm -rf /home/alesito/.cache/BraveSoftware/ 2>/dev/null || true
rm -rf /home/alesito/.cache/chrome-devtools-mcp/ 2>/dev/null || true
echo "✅ Cache del navegador limpia"

echo "📱 4/4 - Limpiando cache de la app..."
rm -rf /home/alesito/.local/share/com.yarvis.pos/CacheStorage/ 2>/dev/null || true
rm -rf /home/alesito/.local/share/com.yarvis.pos/localstorage/ 2>/dev/null || true
rm -rf /home/alesito/.local/share/com.yarvis.pos/storage/ 2>/dev/null || true
echo "✅ Cache de la app limpia"

echo ""
echo "✅ RESET COMPLETO"
echo "  Borrrado: usuarios, tickets, inventario, cache"
echo "  Conservado: codigo, modelos IA, .venv, .git"
