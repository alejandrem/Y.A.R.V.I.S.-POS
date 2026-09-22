#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# build.sh — Build de producción de Y.A.R.V.I.S. POS
#
# Genera binario release + .deb + .rpm + .AppImage.
# Hermano de build.bat (Windows): mismo orden de fases y mismos avisos.
#
# Por qué las variables de entorno:
#   LD_LIBRARY_PATH → libllama.so.0 vive en target/release (la compila
#     llama-cpp-sys); sin esta ruta linuxdeploy no la encuentra y el
#     AppImage sale sin el motor de IA.
#   NO_STRIP=1 → el strip viejo que trae linuxdeploy no reconoce la sección
#     ELF moderna .relr.dyn de las libs de Arch y aborta el bundle.
# ═══════════════════════════════════════════════════════════════════════════

set -e
cd "$(dirname "$0")"

FALTA=0
aviso() { echo "[WARN] $*"; }
error() { echo "[ERROR] $*"; FALTA=1; }

# ── 1. Preflight: Node + Rust ─────────────────────────────────────────────
command -v npm >/dev/null 2>&1 || error "'npm' no instalado. Instala Node.js LTS 20+: https://nodejs.org/"
command -v cargo >/dev/null 2>&1 || error "'cargo' no instalado. Instala Rust estable: https://rustup.rs"
command -v rustc >/dev/null 2>&1 || error "'rustc' no instalado (rustup incompleto). Reinstala: https://rustup.rs"

if command -v node >/dev/null 2>&1; then
  if ! node --version | grep -Eq '^v(1[89]|2[0-9])'; then
    aviso "Node $(node --version) detectado pero no es v18+. Tauri 2 pide Node 18+."
  fi
fi

if command -v rustup >/dev/null 2>&1; then
  rustup target list --installed 2>/dev/null | grep -q "x86_64-unknown-linux-gnu" \
    || error "Falta el target 'x86_64-unknown-linux-gnu'. Instalalo: rustup target add x86_64-unknown-linux-gnu"
  rustup show active-toolchain 2>/dev/null | grep -qi "stable" \
    || aviso "Tu toolchain no es 'stable'. Si falla: rustup update stable && rustup default stable"
fi

# ── 2. Preflight: compilador C/C++ y pkg-config (llama.cpp los exige) ─────
command -v cc >/dev/null 2>&1 || error "Falta compilador C (cc/gcc). Debian/Ubuntu: sudo apt install build-essential | Arch: sudo pacman -S base-devel"
command -v pkg-config >/dev/null 2>&1 || error "Falta 'pkg-config'. Debian/Ubuntu: sudo apt install pkg-config | Arch: sudo pacman -S pkgconf"
command -v curl >/dev/null 2>&1 || error "Falta 'curl'. Debian/Ubuntu: sudo apt install curl | Arch: sudo pacman -S curl"
command -v file >/dev/null 2>&1 || aviso "Falta 'file' (lo usa Tauri al empaquetar). Debian/Ubuntu: sudo apt install file"

# ── 3. Preflight: librerías del sistema para Tauri 2 / WebKit ─────────────
# Se chequea con pkg-config (vale para apt, pacman, dnf sin adivinar distros).
for lib in gtk+-3.0 webkit2gtk-4.1; do
  if command -v pkg-config >/dev/null 2>&1 && ! pkg-config --exists "$lib" 2>/dev/null; then
    error "Falta la librería del sistema '$lib'."
    echo "        Debian/Ubuntu: sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev"
    echo "        Arch:          sudo pacman -S gtk3 webkit2gtk-4.1 libayatana-appindicator librsvg openssl"
    echo "        Fedora:        sudo dnf install gtk3-devel webkit2gtk4.1-devel libayatana-appindicator-gtk3-devel librsvg2-devel openssl-devel"
  fi
done

# ── 4. Preflight: empaquetadores según lo pedido ───────────────────────────
# "$@" se pasa tal cual a 'tauri build'. Si no se pide nada, Tauri saca todo.
ARGS="$*"
QUIERE_DEB=1; QUIERE_RPM=1; QUIERE_APPIMAGE=1
if [ -n "$ARGS" ]; then
  case "$ARGS" in *deb*) QUIERE_DEB=1;; *) QUIERE_DEB=0;; esac
  case "$ARGS" in *rpm*) QUIERE_RPM=1;; *) QUIERE_RPM=0;; esac
  case "$ARGS" in *appimage*) QUIERE_APPIMAGE=1;; *) QUIERE_APPIMAGE=0;; esac
  case "$ARGS" in *none*) QUIERE_DEB=0; QUIERE_RPM=0; QUIERE_APPIMAGE=0;; esac
fi
if [ "$QUIERE_DEB" = 1 ] && ! command -v dpkg-deb >/dev/null 2>&1; then
  error "Falta 'dpkg-deb' para el .deb. Debian/Ubuntu: sudo apt install dpkg | o pasa --bundles rpm appimage"
fi
if [ "$QUIERE_RPM" = 1 ] && ! command -v rpmbuild >/dev/null 2>&1; then
  aviso "Falta 'rpmbuild': el .rpm se saltará. Debian/Ubuntu: sudo apt install rpm | Arch: sudo pacman -S rpm-tools"
fi
if [ "$QUIERE_APPIMAGE" = 1 ]; then
  # fuse2: sin esto el AppImage se genera pero NO ABRE (error clásico B1).
  ldconfig -p 2>/dev/null | grep -q "libfuse.so.2" \
    || error "Falta fuse2 (el AppImage no abriría). Debian/Ubuntu: sudo apt install libfuse2 | Arch: sudo pacman -S fuse2"
  # gdk-pixbuf 2.44 ya no crea /usr/lib/gdk-pixbuf-2.0/2.10.0 pero el plugin
  # gtk de linuxdeploy lo copia por rutina (ver bugs-resueltos.md Bug B1).
  [ -d /usr/lib/gdk-pixbuf-2.0/2.10.0 ] \
    || aviso "Falta /usr/lib/gdk-pixbuf-2.0/2.10.0 (gdk-pixbuf 2.44+). Si linuxdeploy falla créalo: sudo mkdir -p /usr/lib/gdk-pixbuf-2.0/2.10.0"
fi

if [ "$FALTA" = 1 ]; then
  echo ""
  echo "[ABORTADO] Instala lo faltante de arriba e intenta de nuevo."
  exit 1
fi
echo "[OK] Preflight del sistema en verde."

# ── 5. datasets/ incluidos ────────────────────────────────────────────────
# dataset/ (raíz) es canónico; src-tauri/datasets/ es la copia que viaja
# en resources del bundle.
if ! ls ../dataset/*.csv >/dev/null 2>&1; then
  echo "[ERROR] No hay CSV en ../dataset/. Esa carpeta es canónica y no debe estar vacía."
  exit 1
fi
mkdir -p src-tauri/datasets
cp -f ../dataset/*.csv src-tauri/datasets/

# ── 6. Dependencias de Node ───────────────────────────────────────────────
if [ ! -d node_modules ]; then
  echo "[INFO] No se detectó node_modules. Instalando dependencias..."
  npm install
fi

# ── 7. Build ──────────────────────────────────────────────────────────────
export NO_STRIP=1
export LD_LIBRARY_PATH="$PWD/src-tauri/target/release${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

echo ""
echo "[INFO] Ejecutando: npm run tauri build $ARGS"
# shellcheck disable=SC2086
npm run tauri build $ARGS

echo ""
echo "═══════════════════════════════════════════════════"
echo " Artefactos generados en src-tauri/target/release/:"
echo "   binario : ./yarvis-app"
echo "   .deb    : bundle/deb/"
echo "   .rpm    : bundle/rpm/"
echo "   AppImage: bundle/appimage/  (necesita fuse2 en la PC destino)"
echo "═══════════════════════════════════════════════════"
