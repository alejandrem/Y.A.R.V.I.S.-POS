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

if "%MISSING_DEPS%"=="1" (
    echo.
    echo [ABORTADO] Instala los requisitos faltantes arriba indicados e intenta de nuevo.
    pause
    exit /b 1
)

:: 2. Comprobar dependencias de Node en yarvis-app
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
