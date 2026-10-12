@echo off
setlocal enabledelayedexpansion
title Y.A.R.V.I.S. POS - Startup
echo ====================================================
echo   Iniciando Y.A.R.V.I.S. POS
echo ====================================================
echo.

set "ROOT_DIR=%~dp0"
REM Quitar barra invertida final si la hay
if "%ROOT_DIR:~-1%"=="\" set "ROOT_DIR=%ROOT_DIR:~0,-1%"

REM 1. Comprobar comandos esenciales
set "MISSING_DEPS=0"

where npm >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] 'npm' no esta instalado. Por favor instala Node.js y npm.
    echo https://nodejs.org/en/download/
    echo https://npm.github.io/
    set "MISSING_DEPS=1"
)

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] 'cargo' no esta instalado. Por favor instala Rust https://rustup.rs
    set "MISSING_DEPS=1"
)

if "%MISSING_DEPS%"=="1" (
    echo.
    echo [ABORTADO] Instala los requisitos faltantes arriba indicados e intenta de nuevo.
    pause
    exit /b 1
)

REM 1b. Preflight: toolchain MSVC de Rust (llama.cpp + sqlx/bundled compilan
REM     C/C++: sin esto 'cargo' muere a mitad con "linker `link.exe` not found").
REM     Falla rapido aqui con instrucciones, en vez de compilar 30 crates para nada.
REM     NOTA batch: estructura plana con goto, SIN bloques if anidados: los
REM     parentesis dentro de un bloque rompen el parseo del .bat.
where cl >nul 2>nul
if %errorlevel% equ 0 (
    echo [OK] MSVC -cl.exe- detectado.
    goto MSVC_LISTO
)
set "VSWHERE_EXE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "%VSWHERE_EXE%" goto MSVC_FALTA
"%VSWHERE_EXE%" -products * -requires Microsoft.VisualStudio.Component.VC.Tools -property installationPath >nul 2>nul
if %errorlevel% equ 0 goto MSVC_LISTO
echo [ERROR] Tienes Visual Studio pero SIN la carga "Desarrollo para el escritorio con C++".
echo   Abre Visual Studio Installer desde el menu inicio, dale Modificar y marca
echo   la carga "Desarrollo para el escritorio con C++".
pause
exit /b 1
:MSVC_FALTA
echo [ERROR] No se encontro el compilador C++ de Microsoft -link.exe/cl.exe-.
echo   Tauri + llama.cpp lo exigen para compilar el backend Rust.
echo   Instala "Build Tools para Visual Studio 2022" con la carga
echo   "Desarrollo para el escritorio con C++":
echo   https://visualstudio.microsoft.com/es/downloads/#build-tools-for-visual-studio-2022
echo   Luego cierra y reabre la terminal y vuelve a correr run.bat.
pause
exit /b 1
:MSVC_LISTO
rustup target list --installed 2>nul | findstr /c:"x86_64-pc-windows-msvc" >nul
if %errorlevel% neq 0 (
    echo [ERROR] Falta el target 'x86_64-pc-windows-msvc' de Rust.
    echo   Instalalo con:  rustup target add x86_64-pc-windows-msvc
    pause
    exit /b 1
)

REM 1c. libclang para bindgen -lo exige llama-cpp-sys al compilar-.
REM     Si LLVM esta instalado pero LIBCLANG_PATH no, se apunta solo.
REM     NOTA batch: estructura plana con goto, sin bloques anidados.
if defined LIBCLANG_PATH goto LIBCLANG_LISTO
if not exist "C:\Program Files\LLVM\bin\libclang.dll" goto LIBCLANG_AVISO
set "LIBCLANG_PATH=C:\Program Files\LLVM\bin"
echo [OK] libclang detectado en LLVM.
goto LIBCLANG_LISTO
:LIBCLANG_AVISO
echo [WARN] No se detecto libclang. Si 'cargo' muere con "libclang not found", instala LLVM:
echo   winget install -e --id LLVM.LLVM
echo   -luego cierra y reabre la terminal para refrescar el PATH-
goto LIBCLANG_LISTO
:LIBCLANG_LISTO

REM 1d. CMake + Ninja en PATH -los exige llama-cpp-sys al compilar-.
REM     Se agregan solo para esta sesion si existen en sus rutas tipicas.
REM     NOTA batch: estructura plana con goto, sin bloques anidados.
set "CMAKE_BIN=C:\Program Files\CMake\bin"
if not exist "%CMAKE_BIN%\cmake.exe" goto CMAKE_AVISO
set "PATH=%CMAKE_BIN%;%PATH%"
echo [OK] CMake detectado.
goto CMAKE_LISTO
:CMAKE_AVISO
echo [WARN] No se encontro CMake en su ruta tipica. Si 'cargo' muere con "program not found - cmake", instala:
echo   winget install -e --id Kitware.CMake
goto CMAKE_LISTO
:CMAKE_LISTO
set "NINJA_VS=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja"
if exist "%NINJA_VS%\ninja.exe" goto NINJA_VS_OK
if exist "%LOCALAPPDATA%\Microsoft\WinGet\Links\ninja.exe" goto NINJA_WINGET_OK
echo [WARN] No se encontro Ninja. Si 'cargo' muere con MSB1001 por -j12, instala:
echo   winget install -e --id Ninja-build.Ninja
goto NINJA_LISTO
:NINJA_VS_OK
set "PATH=%NINJA_VS%;%PATH%"
echo [OK] Ninja detectado -Visual Studio-.
goto NINJA_LISTO
:NINJA_WINGET_OK
set "PATH=%LOCALAPPDATA%\Microsoft\WinGet\Links;%PATH%"
echo [OK] Ninja detectado -winget-.
goto NINJA_LISTO
:NINJA_LISTO

REM 2. Comprobar dependencias de Node en yarvis-app
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

REM 3. Ajustes de compatibilidad bindgen / Clang para MSVC 2026 en Windows
set "BINDGEN_EXTRA_CLANG_ARGS=-D__clang_major__=20"

REM 4. Refrescar datasets incluidos (dataset\ es canonico; src-tauri\datasets\
REM    es la copia que viaja en resources del bundle y que usa el dev).
if not exist "src-tauri\datasets\" mkdir "src-tauri\datasets"
copy /y "..\dataset\*.csv" "src-tauri\datasets\" >nul

echo.
echo [INFO] Iniciando Y.A.R.V.I.S. POS en modo desarrollo...
call npm run tauri dev

pause
