#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# Tests del FRONTEND (React) — src/test/
#   1 archivo por módulo: finanzas, empleados, ventas, inventario, tickets,
#   yarvis, configuracion, clientes, parseador + estres-*
#   La capa nativa de Tauri está mockeada; ningún test toca el backend real.
#
# Uso: ./run_test.sh            → toda la suite
#      ./run_test.sh watch      → modo vigilancia (re-corre al guardar)
# ═══════════════════════════════════════════════════════════════════════════
set -e
cd "$(dirname "$0")/../../"   # raíz de yarvis-app

if [ ! -d node_modules/vitest ]; then
    echo "[*] Instalando dependencias de test..."
    npm install -D vitest @testing-library/react @testing-library/jest-dom jsdom
fi

if [ "${1:-}" = "watch" ]; then
    npx vitest src/test
else
    npx vitest run --reporter=verbose src/test
    echo ""
    echo "✅ FRONTEND EN VERDE"
fi
