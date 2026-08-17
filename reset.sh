#!/bin/bash
# Y.A.R.V.I.S. POS - Reset completo
# Borra: usuarios, tickets, inventario, cache, entrenamiento de IA
# Conserva: codigo fuente, modelos IA, .venv, .git

set -e

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
APP_DATA="$HOME/.local/share/com.yarvis.pos"

echo "🧹 Iniciando Y.A.R.V.I.S. reset..."

echo "🗃️ 1/4 - Borrando base de datos (usuarios, productos, ventas, knowledge_base)..."
DB_FILE="$APP_DATA/yarvis.db"
if [ -f "$DB_FILE" ]; then
    rm -f "$DB_FILE" "$DB_FILE-shm" "$DB_FILE-wal"
    echo "✅ Base de datos eliminada. Al reabrir la app se recrea vacía (re-parsea todo)."
else
    echo "⚠️ No se encontro la base de datos"
fi

echo "🔍 2/4 - Limpiando cache de Python del repo..."
find "$ROOT_DIR" -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
find "$ROOT_DIR" -name "*.pyc" -delete 2>/dev/null || true
echo "✅ Cache de Python limpia"

echo "🌐 3/4 - Limpiando cachés web de la app..."
rm -rf "$APP_DATA/gpucache/" 2>/dev/null || true
rm -rf "$APP_DATA/cache/" 2>/dev/null || true
echo "✅ Cache web de la app limpia"

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
