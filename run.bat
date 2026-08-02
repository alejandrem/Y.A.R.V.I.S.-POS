@echo off
setlocal enabledelayedexpansion
title Y.A.R.V.I.S. POS - Startup
echo ====================================================
echo   Iniciando Y.A.R.V.I.S. POS
echo ====================================================
echo.

set "ROOT_DIR=%~dp0"
:: Quitar barra invertida final si la hay
if "%ROOT_DIR:~-1%"=="\" set "ROOT_DIR=%ROOT_DIR:~0,-1%"

:: 1. Comprobar comandos esenciales
set "MISSING_DEPS=0"

where npm >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] 'npm' no esta instalado. Por favor instala Node.js y npm.
    set "MISSING_DEPS=1"
)

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] 'cargo' no esta instalado. Por favor instala Rust (https://rustup.rs).
    set "MISSING_DEPS=1"
)

:: En Windows suele ser 'python', intentaremos con python y python3.
set "PYTHON_CMD=python"
where python >nul 2>nul
if %errorlevel% neq 0 (
    where python3 >nul 2>nul
    if %errorlevel% neq 0 (
        echo [ERROR] 'python' no esta instalado. Por favor instala Python.
        set "MISSING_DEPS=1"
    ) else (
        set "PYTHON_CMD=python3"
    )
)

if "%MISSING_DEPS%"=="1" (
    echo.
    echo [ABORTADO] Instala los requisitos faltantes arriba indicados e intenta de nuevo.
    pause
    exit /b 1
)

:: 2. Comprobar e instalar entorno virtual de Python en yarvis-IA
set "VENV_DIR=%ROOT_DIR%\yarvis-IA\.venv"
set "PYTHON_VENV_EXE=%VENV_DIR%\Scripts\python.exe"
set "PIP_VENV_EXE=%VENV_DIR%\Scripts\pip.exe"

set "SETUP_VENV=0"
if not exist "%PYTHON_VENV_EXE%" set "SETUP_VENV=1"

if "%SETUP_VENV%"=="0" (
    "%PYTHON_VENV_EXE%" -c "import fastapi" >nul 2>nul
    if !errorlevel! neq 0 set "SETUP_VENV=1"
)

if "%SETUP_VENV%"=="1" (
    echo [INFO] No se detecto un entorno Python completo en yarvis-IA.
    echo [INFO] Creando venv e instalando dependencias en yarvis-IA...
    if exist "%VENV_DIR%" rmdir /s /q "%VENV_DIR%"
    
    "%PYTHON_CMD%" -m venv "%VENV_DIR%"
    if !errorlevel! neq 0 (
        echo [ERROR] Fallo la creacion del entorno virtual.
        pause
        exit /b 1
    )
    
    "%PIP_VENV_EXE%" install --upgrade pip
    "%PIP_VENV_EXE%" install -r "%ROOT_DIR%\yarvis-IA\requirements.txt"
    if !errorlevel! neq 0 (
        echo [ERROR] Fallo la instalacion de dependencias de Python.
        pause
        exit /b 1
    )
    echo [INFO] Entorno de Python configurado correctamente.
)

:: 3. Comprobar dependencias de Node en yarvis-app
cd /d "%ROOT_DIR%\yarvis-app"
if not exist "node_modules\" (
    echo [INFO] No se detecto la carpeta node_modules. Instalando dependencias...
    call npm install
    if !errorlevel! neq 0 (
        echo [ERROR] Error al instalar dependencias con npm install.
        pause
        exit /b 1
    )
)

echo.
echo [INFO] Iniciando Y.A.R.V.I.S. POS en modo desarrollo...
call npm run tauri dev

pause
