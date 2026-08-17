#!/usr/bin/env bash
# Ejecuta la suite de tests de Y.A.R.V.I.S.-IA.
# Uso:   ./test.sh                 → todos los tests
#        ./test.sh -v              → con detalle
#        ./test.sh tests/test_lote.py  → un archivo específico
#        ./test.sh -k rollback     → solo tests que matcheen 'rollback'
set -euo pipefail

echo "===================================================="
echo "  Iniciando TESTS PARA Y.A.R.V.I.S. POS"
echo "===================================================="
echo

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
YA_DIR="$SCRIPT_DIR/yarvis-IA"
cd "$YA_DIR"

PYTHON="${PYTHON:-$YA_DIR/.venv/bin/python}"

if [ ! -x "$PYTHON" ]; then
    echo "❌ No se encontró el venv en: $PYTHON" >&2
    echo "   Crea el entorno con:" >&2
    echo "     cd yarvis-IA" >&2
    echo "     python -m venv .venv" >&2
    echo "     .venv/bin/python -m pip install -r requirements.txt -r requirements-dev.txt" >&2
    exit 1
fi

export PYTHONPATH="$YA_DIR"

echo "🧪 Y.A.R.V.I.S.-IA — suite de tests (pytest)"
echo

exec "$PYTHON" -m pytest "$@"