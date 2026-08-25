#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# build.sh — Build de producción de Y.A.R.V.I.S. POS
#
# Genera binario release + .deb + .rpm + .AppImage.
# Requisitos ya resueltos en el sistema (ver idea.md Bug B1):
#   · fuse2 instalado (pacman -S fuse2)
#   · /usr/lib/gdk-pixbuf-2.0/2.10.0 existe (gdk-pixbuf 2.44 ya no lo crea,
#     pero el plugin gtk de linuxdeploy lo copia por rutina)
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

export NO_STRIP=1
export LD_LIBRARY_PATH="$PWD/src-tauri/target/release${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

npm run tauri build "$@"

echo ""
echo "═══════════════════════════════════════════════════"
echo " Artefactos generados en src-tauri/target/release/:"
echo "   binario : ./yarvis-app"
echo "   .deb    : bundle/deb/"
echo "   .rpm    : bundle/rpm/"
echo "   AppImage: bundle/appimage/"
echo "═══════════════════════════════════════════════════"
