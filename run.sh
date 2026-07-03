#!/bin/bash

echo "===================================================="
echo "  Iniciando Y.A.R.V.I.S. POS"
echo "===================================================="
echo ""

# Obtener el directorio donde se encuentra este script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/yarvis-app"

# Comprobar si existe la carpeta node_modules
if [ ! -d "node_modules" ]; then
    echo "[INFO] No se detectó la carpeta node_modules. Instalando dependencias..."
    npm install
    if [ $? -ne 0 ]; then
        echo "[ERROR] Error al instalar dependencias con npm install."
        read -p "Presiona Enter para salir..."
        exit 1
    fi
fi

echo ""
echo "[INFO] Iniciando la aplicación en modo desarrollo..."
npm run tauri dev

read -p "Presiona Enter para salir..."
