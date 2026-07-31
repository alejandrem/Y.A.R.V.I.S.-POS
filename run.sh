#!/bin/bash

echo "===================================================="
echo "  Iniciando Y.A.R.V.I.S. POS"
echo "===================================================="
echo ""

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR="$SCRIPT_DIR"

# 1. Comprobar comandos esenciales
MISSING_DEPS=0

if ! command -v npm &> /dev/null; then
    echo "[ERROR] 'npm' no está instalado. Por favor instala Node.js y npm."
    MISSING_DEPS=1
fi

if ! command -v cargo &> /dev/null; then
    echo "[ERROR] 'cargo' no está instalado. Por favor instala Rust (https://rustup.rs)."
    MISSING_DEPS=1
fi

if ! command -v python3 &> /dev/null; then
    echo "[ERROR] 'python3' no está instalado. Por favor instala Python 3."
    MISSING_DEPS=1
fi

if [ $MISSING_DEPS -eq 1 ]; then
    echo ""
    echo "[ABORTADO] Instala los requisitos faltantes arriba indicados e intenta de nuevo."
    read -p "Presiona Enter para salir..."
    exit 1
fi

# 2. Comprobar e instalar entorno virtual de Python en yarvis-IA
VENV_DIR="$ROOT_DIR/yarvis-IA/.venv"
if [ ! -f "$VENV_DIR/bin/python3" ] || ! "$VENV_DIR/bin/python3" -c "import fastapi" &> /dev/null; then
    echo "[INFO] No se detectó un entorno Python completo en yarvis-IA."
    echo "[INFO] Creando venv e instalando dependencias en yarvis-IA..."
    rm -rf "$VENV_DIR"
    python3 -m venv "$VENV_DIR"
    "$VENV_DIR/bin/pip" install --upgrade pip
    "$VENV_DIR/bin/pip" install -r "$ROOT_DIR/yarvis-IA/requirements.txt"
    if [ $? -ne 0 ]; then
        echo "[ERROR] Falló la instalación de dependencias de Python."
        read -p "Presiona Enter para salir..."
        exit 1
    fi
    echo "[INFO] Entorno de Python configurado correctamente."
fi

# 3. Comprobar dependencias de Node en yarvis-app
cd "$ROOT_DIR/yarvis-app"
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
echo "[INFO] Iniciando Y.A.R.V.I.S. POS en modo desarrollo..."
npm run tauri dev

read -p "Presiona Enter para salir..."

