import os
import sys

# Asegura que 'yarvis-IA' esté en sys.path para importar parseador_de_tickets, chatbot, profeta
_PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if _PROJECT_ROOT not in sys.path:
    sys.path.insert(0, _PROJECT_ROOT)


def pytest_sessionfinish(session, exitstatus):
    """Limpia cualquier cosa pendiente tras correr la suite."""
    pass