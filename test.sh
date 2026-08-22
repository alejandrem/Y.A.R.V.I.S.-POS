#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# Y.A.R.V.I.S. POS — Ejecutor único de tests del frontend.
# Corre TODA la suite de Vitest (funcionales + estrés, 1 archivo por módulo)
# en una sola pasada y reporta el resumen. Salida 0 = todo verde.
#
# Uso:  ./test.sh            (desde la raíz del repo)
# ═══════════════════════════════════════════════════════════════════════════
set -e

cd "$(dirname "$0")/yarvis-app"

echo "╔══════════════════════════════════════════╗"
echo "║   Y.A.R.V.I.S. POS · Suite de Frontend   ║"
echo "╚══════════════════════════════════════════╝"

if [ ! -d node_modules/vitest ]; then
    echo "[*] Instalando dependencias de test..."
    npm install -D vitest @testing-library/react @testing-library/jest-dom jsdom
fi

npx vitest run --reporter=verbose "$@"
