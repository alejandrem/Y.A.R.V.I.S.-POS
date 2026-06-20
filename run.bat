@echo off
title Y.A.R.V.I.S. POS - Startup
echo ====================================================
echo   Iniciando Y.A.R.V.I.S. POS
echo ====================================================
echo.

:: Cambiar al directorio yarvis-app
cd /d "%~dp0yarvis-app"

:: Comprobar si existe la carpeta node_modules
if not exist node_modules (
    echo [INFO] No se detecto la carpeta node_modules. Instalando dependencias...
    call npm install
    if %errorlevel% neq 0 (
        echo [ERROR] Error al instalar dependencias con npm install.
        pause
        exit /b %errorlevel%
    )
)

echo.
echo [INFO] Iniciando la aplicacion en modo desarrollo...
call npm run tauri dev

pause

