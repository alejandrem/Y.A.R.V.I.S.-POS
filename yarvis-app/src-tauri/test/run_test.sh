#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# Tests del BACKEND Tauri (Rust) — src-tauri/test/
#   funcionamiento/ : 1 binario por módulo (auth, ventas, gastos,
#                     inventario, empleados) contra DB real temporal.
#   estres/         : integridad bajo carga y queries sobre 10k+ filas.
#
# Uso: ./run_test.sh              → funcionales + estrés
#      ./run_test.sh funcional    → solo funcionales
#      ./run_test.sh estres       → solo estrés
# ═══════════════════════════════════════════════════════════════════════════
set -e
cd "$(dirname "$0")/../"   # raíz de src-tauri

MODO="${1:-all}"

case "$MODO" in
  funcional)
    cargo test --test func_ventas --test func_gastos --test func_inventario \
               --test func_empleados --test func_auth
    ;;
  estres)
    cargo test --release --test estres_ventas --test estres_consultas -- --nocapture
    ;;
  *)
    cargo test --test func_ventas --test func_gastos --test func_inventario \
               --test func_empleados --test func_auth
    echo ""
    cargo test --test estres_ventas --test estres_consultas -- --nocapture
    ;;
esac

echo ""
echo "✅ BACKEND EN VERDE"
