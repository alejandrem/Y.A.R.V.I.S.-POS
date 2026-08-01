#!/bin/bash
# Y.A.R.V.I.S. POS - Reset completo
# Borra: usuarios, tickets, inventario, cache, entrenamiento de IA
# Conserva: codigo fuente, modelos IA, .venv, .git

set -e

echo "🧹 Iniciando Y.A.R.V.I.S. reset..."

APP_DATA="$HOME/.local/share/com.yarvis.pos"

echo "🗃️ 1/4 - Borrando base de datos (usuarios, productos, ventas, knowledge_base)..."
DB_FILE="$APP_DATA/yarvis.db"
if [ -f "$DB_FILE" ]; then
    rm -f "$DB_FILE" "$DB_FILE-shm" "$DB_FILE-wal"
    echo "✅ Base de datos eliminada"
else
    echo "⚠️ No se encontro la base de datos"
fi

echo "🔍 2/4 - Limpiando cache de Python..."
find "$HOME/Documentos/Y.A.R.V.I.S.-POS" -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
find "$HOME/Documentos/Y.A.R.V.I.S.-POS" -name "*.pyc" -delete 2>/dev/null || true
echo "✅ Cache de Python limpia"

echo "🌐 3/4 - Limpiando cache del navegador..."
rm -rf "$HOME/.cache/BraveSoftware/" 2>/dev/null || true
rm -rf "$HOME/.cache/chrome-devtools-mcp/" 2>/dev/null || true
echo "✅ Cache del navegador limpia"

echo "📱 4/4 - Limpiando cache de la app..."
rm -rf "$APP_DATA/CacheStorage/" 2>/dev/null || true
rm -rf "$APP_DATA/localstorage/" 2>/dev/null || true
rm -rf "$APP_DATA/storage/" 2>/dev/null || true
rm -rf "$APP_DATA/WebKitCache/" 2>/dev/null || true
rm -rf "$APP_DATA/mediakeys/" 2>/dev/null || true
rm -f "$APP_DATA/hsts-storage.sqlite" 2>/dev/null || true
echo "✅ Cache de la app limpia"

echo ""
echo "✅ RESET COMPLETO"
echo "  Borrado: usuarios, tickets, inventario, cache, entrenamiento de IA"
echo "  Conservado: codigo, modelos IA, .venv, .git"
