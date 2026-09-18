@echo off
setlocal enabledelayedexpansion
title Y.A.R.V.I.S. POS - Build Windows
echo ====================================================
echo   Y.A.R.V.I.S. POS - Build de produccion Windows
echo   Genera .exe portable + instalador NSIS en un solo .exe
echo ====================================================
echo.

REM Este archivo es hermano de build.sh (Linux).
REM Uso:
REM   build.bat              :: genera instalador NSIS (un solo .exe)
REM   build.bat --bundles msi :: genera MSI en vez de NSIS
REM   build.bat --bundles nsis msi :: genera ambos
REM Artefactos en src-tauri\target\release\bundle\

REM 1. Comprobar comandos esenciales
set "MISSING_DEPS=0"

where npm >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] 'npm' no esta instalado. Instala Node.js LTS: https://nodejs.org/
    set "MISSING_DEPS=1"
)

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] 'cargo' no esta instalado. Instala Rust: https://rustup.rs
    set "MISSING_DEPS=1"
)

if "%MISSING_DEPS%"=="1" (
    echo.
    echo [ABORTADO] Instala los requisitos faltantes e intenta de nuevo.
    pause
    exit /b 1
)

REM 2. Situarse en yarvis-app (donde vive este .bat)
cd /d "%~dp0"
if not exist "package.json" (
    echo [ERROR] No se encontro package.json. Este .bat debe vivir en yarvis-app\ junto a build.sh.
    pause
    exit /b 1
)

REM 3. Dependencias de Node
if not exist "node_modules\" (
    echo [INFO] No se detecto node_modules. Instalando dependencias...
    call npm install
    if !errorlevel! neq 0 (
        echo [ERROR] Fallo npm install.
        pause
        exit /b 1
    )
)

REM 4. Compatibilidad bindgen / Clang para MSVC (igual que run.bat)
set "BINDGEN_EXTRA_CLANG_ARGS=-D__clang_major__=20"

REM 5. Fase 1: compilar release para generar las DLLs frescas de llama.cpp.
REM    El bundle las toma de src-tauri\dlls\, asi que hay que refrescarlas
REM    ANTES de empaquetar (si no, el instalador llevaria DLLs viejas).
echo.
echo [INFO] Fase 1/2: cargo build --release (genera las DLLs)...
echo [INFO] Esto tarda varios minutos la primera vez (llama.cpp + Rust)...
echo.
call cargo build --release --manifest-path src-tauri\Cargo.toml
if !errorlevel! neq 0 (
    echo.
    echo [ERROR] Fallo cargo build --release. Si es toolchain de Rust prueba:
    echo   rustup update stable ^&^& rustup default stable
    pause
    exit /b 1
)

echo [INFO] Refrescando src-tauri\dlls\ con las DLLs recien compiladas...
copy /y "src-tauri\target\release\llama.dll" "src-tauri\dlls\" >nul
copy /y "src-tauri\target\release\ggml.dll" "src-tauri\dlls\" >nul
copy /y "src-tauri\target\release\ggml-base.dll" "src-tauri\dlls\" >nul
copy /y "src-tauri\target\release\ggml-cpu.dll" "src-tauri\dlls\" >nul
copy /y "src-tauri\target\release\mtmd.dll" "src-tauri\dlls\" >nul
if !errorlevel! neq 0 (
    echo [ERROR] No se pudieron copiar las DLLs a src-tauri\dlls\.
    pause
    exit /b 1
)

REM 6. Fase 2: bundles. Por defecto NSIS (un solo .exe instalador).
REM    Si pasas argumentos, se usan tal cual. Ej: build.bat --bundles msi
set "BUNDLES=--bundles nsis"
if not "%~1"=="" set "BUNDLES=%*"

echo.
echo [INFO] Fase 2/2: npm run tauri build -- %BUNDLES%
echo.
call npm run tauri build -- %BUNDLES%
if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Fallo el build. Si es toolchain de Rust prueba:
    echo   rustup update stable ^&^& rustup default stable
    echo   cd src-tauri ^&^& cargo clean
    pause
    exit /b 1
)

echo.
echo ===========================================================
echo  Artefactos generados en src-tauri\target\release\:
echo    portable : .\src-tauri\target\release\yarvis-app.exe
echo    setupexe : .\src-tauri\target\release\bundle\nsis\
echo    msi      : .\src-tauri\target\release\bundle\msi\  (solo si pediste msi)
echo  NOTA: el .exe necesita WebView2 (Win10/11 ya lo trae).
echo  El modelo IA GGUF NO va dentro, se descarga aparte en la app.
echo ===========================================================
pause
