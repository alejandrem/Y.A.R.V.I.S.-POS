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

REM ============================================================
REM 0. Que bundles se piden (para exigir NSIS o WiX solo si toca)
REM ============================================================
set "BUNDLES=--bundles nsis"
if not "%~1"=="" set "BUNDLES=%*"
set "NEED_NSIS=0"
set "NEED_MSI=0"
echo %BUNDLES% | findstr /i "nsis" >nul && set "NEED_NSIS=1"
echo %BUNDLES% | findstr /i "msi" >nul && set "NEED_MSI=1"
REM Por defecto (sin args) BUNDLES ya trae nsis -> NEED_NSIS=1
if "%~1"=="" set "NEED_NSIS=1"

REM ============================================================
REM 1. Preflight: comandos esenciales (Node, Rust)
REM ============================================================
set "MISSING_DEPS=0"

where npm >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] 'npm' no esta instalado. Instala Node.js LTS 20+: https://nodejs.org/
    set "MISSING_DEPS=1"
) else (
    node --version 2>nul | findstr /r "^v\(1[89]\|2[0-9]\)" >nul
    if !errorlevel! neq 0 (
        echo [WARN] Node detectado pero no es v18+. Tauri 2 pide Node 18+; si falla el build actualiza Node.
    )
)

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] 'cargo' no esta instalado. Instala Rust estable: https://rustup.rs
    set "MISSING_DEPS=1"
)
where rustc >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] 'rustc' no esta instalado (rustup incompleto). Reinstala: https://rustup.rs
    set "MISSING_DEPS=1"
)

if "%MISSING_DEPS%"=="1" (
    echo.
    echo [ABORTADO] Instala los requisitos faltantes e intenta de nuevo.
    pause
    exit /b 1
)

REM ============================================================
REM 2. Preflight: toolchain MSVC de Rust (el error mas comun)
REM    Llama.cpp + sqlx/bundled compilan C/C++: sin MSVC no hay .exe.
REM ============================================================
rustup target list --installed 2>nul | findstr /c:"x86_64-pc-windows-msvc" >nul
if %errorlevel% neq 0 (
    echo [ERROR] Falta el target 'x86_64-pc-windows-msvc' de Rust.
    echo   Instalalo con:  rustup target add x86_64-pc-windows-msvc
    echo   Y fija estable: rustup default stable
    pause
    exit /b 1
)
rustup show active-toolchain 2>nul | findstr /i "stable" >nul
if %errorlevel% neq 0 (
    echo [WARN] Tu toolchain activo no es 'stable'. Si el build falla prueba:
    echo   rustup update stable ^&^& rustup default stable
)

REM Linker de MSVC (cl.exe) o al menos vswhere (detector de VS Build Tools)
where cl >nul 2>nul
if %errorlevel% neq 0 (
    if exist "%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" (
        "%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" -products * -requires Microsoft.VisualStudio.Component.VC.Tools -property installationPath >nul 2>nul
        if !errorlevel! neq 0 (
            echo [ERROR] Tienes Visual Studio pero SIN la carga "Desktop C++" (VC.Tools).
            echo   Abre Visual Studio Installer ^> Modificar ^> marca "Desarrollo para el escritorio con C++".
            pause
            exit /b 1
        )
    ) else (
        echo [ERROR] No se encontro el compilador C++ de Microsoft (cl.exe ni vswhere).
        echo   Instala "Build Tools para Visual Studio 2022" con la carga
        echo   "Desarrollo para el escritorio con C++":
        echo   https://visualstudio.microsoft.com/es/downloads/#build-tools-for-visual-studio-2022
        pause
        exit /b 1
    )
) else (
    echo [OK] MSVC (cl.exe) detectado.
)

REM ============================================================
REM 3. Preflight: instaladores (solo lo que se va a usar)
REM ============================================================
if "%NEED_NSIS%"=="1" (
    where makensis >nul 2>nul
    if !errorlevel! neq 0 (
        echo [ERROR] Pediste bundle NSIS pero 'makensis' no esta en el PATH.
        echo   Instala NSIS 3.x: https://nsis.sourceforge.io/Download
        echo   (marca "Add to PATH" en el instalador) o corre solo portable con:
        echo   build.bat --bundles none
        pause
        exit /b 1
    ) else (
        echo [OK] NSIS detectado.
    )
)
if "%NEED_MSI%"=="1" (
    where candle >nul 2>nul
    if !errorlevel! neq 0 (
        echo [ERROR] Pediste bundle MSI pero WiX Toolset (candle.exe) no esta en el PATH.
        echo   Instala WiX 3.14: https://wixtoolset.org/releases/ ^(o quita 'msi' de los bundles^)
        pause
        exit /b 1
    ) else (
        echo [OK] WiX detectado.
    )
)

REM ============================================================
REM 4. Preflight: Clang/libclang para bindgen (aviso, no aborta)
REM    Si falta, cargo falla a mitad con error raro de 'libclang not found'.
REM    winget lo resuelve en un comando.
REM ============================================================
set "TIENE_CLANG=0"
where clang >nul 2>nul && set "TIENE_CLANG=1"
if defined LIBCLANG_PATH set "TIENE_CLANG=1"
if "%TIENE_CLANG%"=="0" (
    echo [WARN] No se detecto Clang/libclang. Si 'cargo build' muere con
    echo   "libclang not found" o error de bindgen, instala LLVM y reintenta:
    echo   winget install -e --id LLVM.LLVM
    echo   (luego cierra y reabre la terminal para refrescar el PATH)
) else (
    echo [OK] Clang/libclang detectado.
)
REM Compatibilidad bindgen / Clang para MSVC 2026 en Windows (igual que run.bat)
set "BINDGEN_EXTRA_CLANG_ARGS=-D__clang_major__=20"

REM ============================================================
REM 5. Preflight: WebView2 (aviso, no aborta)
REM    NO se necesita para COMPILAR (Tauri baja el SDK solo), pero el .exe
REM    NO ABRE en PCs sin WebView2 Runtime. Win10/11 ya lo trae; Win7/8 o
REM    VMs viejas, no. Se avisa aqui para no descubrirlo en la tienda.
REM ============================================================
set "TIENE_WV2=0"
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>nul && set "TIENE_WV2=1"
reg query "HKLM\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>nul && set "TIENE_WV2=1"
reg query "HKCU\Software\Microsoft\EdgeUpdate\ClientState\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>nul && set "TIENE_WV2=1"
if "%TIENE_WV2%"=="0" (
    echo [WARN] No se detecto WebView2 Runtime en ESTA maquina. El build sigue,
    echo   pero el .exe no abrira donde falte. Instalalo en cada PC destino:
    echo   https://developer.microsoft.com/es-es/microsoft-edge/webview2/#download
) else (
    echo [OK] WebView2 Runtime detectado.
)
echo.

REM ============================================================
REM 6. Situarse en yarvis-app (donde vive este .bat)
REM ============================================================
cd /d "%~dp0"
if not exist "package.json" (
    echo [ERROR] No se encontro package.json. Este .bat debe vivir en yarvis-app\ junto a build.sh.
    pause
    exit /b 1
)

REM ============================================================
REM 7. Dependencias de Node
REM ============================================================
if not exist "node_modules\" (
    echo [INFO] No se detecto node_modules. Instalando dependencias...
    call npm install
    if !errorlevel! neq 0 (
        echo [ERROR] Fallo npm install.
        pause
        exit /b 1
    )
)

REM ============================================================
REM 8. Fase 1/2: compilar release para generar las DLLs frescas de llama.cpp.
REM    El bundle las toma de src-tauri\dlls\, asi que hay que refrescarlas
REM    ANTES de empaquetar (si no, el instalador llevaria DLLs viejas).
REM ============================================================
echo.
echo [INFO] Fase 1/2: cargo build --release (genera las DLLs)...
echo [INFO] Esto tarda varios minutos la primera vez (llama.cpp + Rust)...
echo [INFO] Bundles pedidos: %BUNDLES%
echo.
call cargo build --release --manifest-path src-tauri\Cargo.toml
if !errorlevel! neq 0 (
    echo.
    echo [ERROR] Fallo cargo build --release. Prueba en orden:
    echo   1. rustup update stable ^&^& rustup default stable
    echo   2. set LIBCLANG_PATH=C:\Program Files\LLVM\bin ^(si fue error de bindgen^)
    echo   3. cd src-tauri ^&^& cargo clean ^(solo si cambiaste de toolchain^)
    pause
    exit /b 1
)

echo [INFO] Refrescando src-tauri\dlls\ con las DLLs recien compiladas...
set "DLLS_OK=1"
for %%D in (llama.dll ggml.dll ggml-base.dll ggml-cpu.dll mtmd.dll) do (
    if not exist "src-tauri\target\release\%%D" (
        echo [ERROR] Falta src-tauri\target\release\%%D tras compilar.
        echo   Posible cambio de nombres en llama-cpp. Revisa target\release\*.dll
        echo   y actualiza la lista en build.bat + tauri.conf.json (resources).
        set "DLLS_OK=0"
    ) else (
        copy /y "src-tauri\target\release\%%D" "src-tauri\dlls\" >nul
        if !errorlevel! neq 0 set "DLLS_OK=0"
    )
)
if "%DLLS_OK%"=="0" (
    echo [ERROR] No se pudieron refrescar las DLLs en src-tauri\dlls\.
    pause
    exit /b 1
)

echo [INFO] Refrescando src-tauri\datasets\ con los CSV canonicos de dataset\...
if not exist "src-tauri\datasets\" mkdir "src-tauri\datasets"
if not exist "..\dataset\*.csv" (
    echo [ERROR] No hay CSV en ..\dataset\. Esa carpeta es canonica y no debe estar vacia.
    pause
    exit /b 1
)
copy /y "..\dataset\*.csv" "src-tauri\datasets\" >nul
if !errorlevel! neq 0 (
    echo [ERROR] No se pudieron copiar los CSV a src-tauri\datasets\.
    pause
    exit /b 1
)

REM ============================================================
REM 9. Fase 2/2: bundles.
REM ============================================================
echo.
echo [INFO] Fase 2/2: npm run tauri build -- %BUNDLES%
echo.
call npm run tauri build -- %BUNDLES%
if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Fallo el build. Prueba en orden:
    echo   rustup update stable ^&^& rustup default stable
    echo   cd src-tauri ^&^& cargo clean
    echo   Revisa tambien que makensis/WiX esten instalados si pediste esos bundles.
    pause
    exit /b 1
)

echo.
echo ===========================================================
echo  Artefactos generados en src-tauri\target\release\:
echo    portable : .\src-tauri\target\release\yarvis-app.exe
echo    setupexe : .\src-tauri\target\release\bundle\nsis\
echo    msi      : .\src-tauri\target\release\bundle\msi\  (solo si pediste msi)
echo  NOTA: el .exe necesita WebView2 Runtime en la PC destino (Win10/11 ya lo trae).
echo  El modelo IA GGUF NO va dentro, se descarga aparte en la app.
echo ===========================================================
pause
