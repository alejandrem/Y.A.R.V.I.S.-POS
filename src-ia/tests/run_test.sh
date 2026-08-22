#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════
# Tests del motor de IA (src-ia) — tests/
#   estres.rs            : fuzzing + invariantes del parseador de tickets
#                          (automatizable, corre en esta suite).
#   verificar_conexion.rs, verificar_modelos.rs, test_chat_1_7_real.rs :
#                          pruebas MANUALES contra modelo real / API cloud.
#                          Requieren recursos externos; NO corren aquí.
#
# Uso: ./run_test.sh              → suite automatizable (estrés)
#      ./run_test.sh manuales     → las de verificación con modelo/API
# ═══════════════════════════════════════════════════════════════════════════
set -e
cd "$(dirname "$0")/.."   # raíz de src-ia

MODO="${1:-auto}"

case "$MODO" in
  manuales)
    echo "[*] Estas pruebas requieren el modelo GGUF local y/o API key."
    cargo test --test verificar_conexion -- --nocapture
    cargo test --test verificar_modelos -- --nocapture
    cargo test --test test_chat_1_7_real -- --nocapture
    ;;
  *)
    cargo test --test estres -- --nocapture
    echo ""
    echo "✅ IA (suite automatizable) EN VERDE"
    echo "    ℹ️  Las pruebas con modelo real: ./run_test.sh manuales"
    ;;
esac
