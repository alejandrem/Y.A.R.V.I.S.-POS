@echo off
setlocal enabledelayedexpansion
title Y.A.R.V.I.S. POS - Reset
echo ====================================================
echo   Y.A.R.V.I.S. POS - Reset completo (Windows)
echo   Borra: usuarios, tickets, inventario, cache, entrenamiento IA
echo   Conserva: codigo fuente, modelos IA, .git
echo ====================================================
echo.

REM Hermano de reset.sh (Linux). Limpia solo el app_data_dir de Tauri.
REM   Roaming: %APPDATA%\com.yarvis.pos  (yarvis.db + caches)
REM   Local:   %LOCALAPPDATA%\com.yarvis.pos (cache extra WebView2)
REM No toca la carpeta del repo, ni node_modules, ni src-ia\target.

set "APP_DATA=%APPDATA%\com.yarvis.pos"
set "APP_CACHE=%LOCALAPPDATA%\com.yarvis.pos"

echo [1/3] Borrando base de datos (usuarios, productos, ventas, knowledge_base)...
if exist "%APP_DATA%\yarvis.db" (
    del /f /q "%APP_DATA%\yarvis.db" "%APP_DATA%\yarvis.db-shm" "%APP_DATA%\yarvis.db-wal" 2>nul
    echo [OK] Base de datos eliminada. Al reabrir la app se recrea vacia.
) else (
    echo [AVISO] No se encontro la base de datos en "%APP_DATA%\yarvis.db"
)

echo.
echo [2/3] Limpiando caches web de la app...
for %%D in ("gpucache" "cache" "CacheStorage" "localstorage" "storage" "WebKitCache" "mediakeys" "EBWebView") do (
    if exist "%APP_DATA%\%%~D" rmdir /s /q "%APP_DATA%\%%~D" 2>nul
    if exist "%APP_CACHE%\%%~D" rmdir /s /q "%APP_CACHE%\%%~D" 2>nul
)
if exist "%APP_DATA%\hsts-storage.sqlite" del /f /q "%APP_DATA%\hsts-storage.sqlite" 2>nul
if exist "%APP_CACHE%\hsts-storage.sqlite" del /f /q "%APP_CACHE%\hsts-storage.sqlite" 2>nul
echo [OK] Cache web limpia

echo.
echo [3/3] Limpiando cache extra de WebView2...
if exist "%APP_CACHE%" (
    REM No borramos todo %APP_CACHE% a ciegas, solo subcarpetas conocidas de arriba.
    echo [OK] Cache extra limpia
) else (
    echo [AVISO] No hay carpeta de cache en "%APP_CACHE%"
)

echo.
echo ===========================================================
echo  RESET COMPLETO
echo   Borrado: usuarios, tickets, inventario, cache, entrenamiento IA
echo   Conservado: codigo, modelos IA, .git
echo ===========================================================
pause
