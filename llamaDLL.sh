#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# llamaDLL.sh — Descarga las 5 DLLs de llama.cpp (Windows) que Tauri
# empaqueta como resources (ver yarvis-app/src-tauri/tauri.conf.json).
#
# Por qué existe: esas DLLs viven en yarvis-app/src-tauri/target/release/,
# que es CARPETA DE CACHE (gitignored). Si borras target/ (limpieza,
# cambio de PC, CI), el build de Tauri aborta con:
#   resource path `target/release/<algo>.dll` doesn't exist
# Este script las repone. Correlo una vez después de limpiar.
#
# Uso:
#   ./llamaDLL.sh            # Linux, macOS o Windows con Git Bash
#
# En Windows: ábrelo con Git Bash (viene con Git). No funciona en
# cmd/powershell tal cual. Requiere: curl + (python3 | python | py).
#
# Versión pineada: llama.cpp b10235 = la que trae vendada el crate
# llama-cpp-sys-4 0.5.1 (ver su README). Si subes ese crate, cambia
# LLAMA_TAG aquí al build correspondiente. NO mezcles versiones: el .exe
# carga estas DLLs por link dinámico y una versión distinta puede tronar.
# ═══════════════════════════════════════════════════════════════════════════

set -e

LLAMA_TAG="b10235"
ZIP_NAME="llama-${LLAMA_TAG}-bin-win-cpu-x64.zip"
URL="https://github.com/ggml-org/llama.cpp/releases/download/${LLAMA_TAG}/${ZIP_NAME}"

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
DEST="$ROOT_DIR/yarvis-app/src-tauri/target/release"
TMP_ZIP="$(mktemp "${TMPDIR:-/tmp}/llama-dll-XXXXXX.zip")"

# ggml-cpu.dll se partió por arquitectura desde b10xxx: la variante
# x64 es la genérica (las demás son alderlake, zen4, etc.).
declare -A QUIERO=(
  ["llama.dll"]="llama.dll"
  ["ggml.dll"]="ggml.dll"
  ["ggml-base.dll"]="ggml-base.dll"
  ["ggml-cpu-x64.dll"]="ggml-cpu.dll"
  ["mtmd.dll"]="mtmd.dll"
)

echo "===================================================="
echo "  Descargando DLLs de llama.cpp ${LLAMA_TAG}"
echo "===================================================="

command -v curl >/dev/null || { echo "[ERROR] falta 'curl'."; exit 1; }
if command -v python3 >/dev/null; then
  PY=python3
elif command -v python >/dev/null; then
  PY=python
elif command -v py >/dev/null; then
  PY="py -3"
else
  echo "[ERROR] falta python (python3 | python | py) para descomprimir el zip."
  exit 1
fi

mkdir -p "$DEST"

echo "[INFO] Bajando ${ZIP_NAME}..."
curl -sSL -o "$TMP_ZIP" "$URL"

echo "[INFO] Extrayendo 5 DLLs a $DEST ..."
$PY - "$TMP_ZIP" "$DEST" <<'EOF'
import sys, zipfile, shutil, os
zip_path, dest = sys.argv[1], sys.argv[2]
quiero = {
    "llama.dll": "llama.dll",
    "ggml.dll": "ggml.dll",
    "ggml-base.dll": "ggml-base.dll",
    "ggml-cpu-x64.dll": "ggml-cpu.dll",
    "mtmd.dll": "mtmd.dll",
}
with zipfile.ZipFile(zip_path) as z:
    disponibles = set(z.namelist())
    for src, nombre in quiero.items():
        if src not in disponibles:
            raise SystemExit(f"[ERROR] el zip no trae {src}. Revisa LLAMA_TAG.")
        with z.open(src) as f, open(os.path.join(dest, nombre), "wb") as o:
            shutil.copyfileobj(f, o)
        print(f"  ok {nombre}")
EOF

rm -f "$TMP_ZIP"

echo ""
echo "✅ DLLs listas en $DEST:"
ls -la "$DEST"/*.dll
echo ""
echo "Ya puedes correr ./run.sh (dev) o empaquetar el .exe en Windows."
