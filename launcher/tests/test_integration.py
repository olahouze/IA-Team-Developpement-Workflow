"""
Test d'intégration : vérifie que le système complet démarre et que Windmill est accessible.
Ce script est à exécuter manuellement — il ne fait PAS partie de la CI.

Usage: python test_integration.py
"""
import socket
import sys
import time
import subprocess
import os
import signal
import urllib.request
import urllib.error

WINDMILL_URL = "http://localhost:8000/"
WINDMILL_PORT = 8000
PGSQL_PORT = 5432
TIMEOUT_SECONDS = 120
POLL_INTERVAL = 3


def is_port_free(port: int) -> bool:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.settimeout(1)
        return s.connect_ex(('127.0.0.1', port)) != 0


def wait_for_url(url: str, timeout: int) -> bool:
    start = time.time()
    while time.time() - start < timeout:
        try:
            resp = urllib.request.urlopen(url, timeout=5)
            if resp.status < 500:
                return True
        except (urllib.error.URLError, OSError):
            pass
        time.sleep(POLL_INTERVAL)
    return False


def main():
    print("=== Test d'intégration : IA-Team-Developpement-Workflow ===\n")

    # 1. Vérifier que les ports sont libres
    print("[1/5] Vérification des ports...")
    for name, port in [("PostgreSQL", PGSQL_PORT), ("Windmill", WINDMILL_PORT)]:
        if not is_port_free(port):
            print(f"  FAIL: Port {port} ({name}) est déjà occupé. Libérez-le avant de lancer le test.")
            sys.exit(1)
    print("  OK: Ports libres.\n")

    # 2. Lancer run.py
    launcher_dir = os.path.join(os.path.dirname(__file__), '..')
    run_py = os.path.join(launcher_dir, 'run.py')
    if not os.path.exists(run_py):
        print(f"  FAIL: run.py introuvable à {run_py}")
        sys.exit(1)

    print(f"[2/5] Lancement de run.py (timeout={TIMEOUT_SECONDS}s)...")
    proc = subprocess.Popen(
        [sys.executable, run_py],
        cwd=launcher_dir,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        creationflags=subprocess.CREATE_NEW_PROCESS_GROUP if os.name == 'nt' else 0,
    )

    try:
        # 3. Attendre que Windmill soit accessible
        print(f"[3/5] Attente de Windmill sur {WINDMILL_URL}...")
        if wait_for_url(WINDMILL_URL, TIMEOUT_SECONDS):
            print(f"  OK: Windmill est accessible sur {WINDMILL_URL}\n")
        else:
            print(f"  FAIL: Windmill n'est pas accessible après {TIMEOUT_SECONDS}s.")
            # Collect output for debug
            proc.terminate()
            stdout, _ = proc.communicate(timeout=10)
            print("--- Sortie du processus ---")
            print(stdout[-5000:] if len(stdout) > 5000 else stdout)
            sys.exit(1)

        # 4. Vérifier que la réponse HTTP est valide
        print("[4/5] Vérification de la réponse HTTP Windmill...")
        try:
            resp = urllib.request.urlopen(WINDMILL_URL, timeout=10)
            body = resp.read().decode('utf-8', errors='replace')
            if resp.status == 200 and ('windmill' in body.lower() or '<html' in body.lower()):
                print(f"  OK: HTTP {resp.status}, page HTML valide reçue.\n")
            else:
                print(f"  WARN: HTTP {resp.status}, contenu inattendu (taille={len(body)} bytes).\n")
        except Exception as e:
            print(f"  FAIL: Erreur HTTP : {e}\n")
            sys.exit(1)

        print("[5/5] Test réussi ! Windmill est fonctionnel.")
        print(f"       URL: {WINDMILL_URL}")

    finally:
        # 5. Cleanup
        print("\n--- Nettoyage ---")
        if os.name == 'nt':
            # Windows: kill process tree
            subprocess.run(['taskkill', '/F', '/T', '/PID', str(proc.pid)],
                           capture_output=True)
        else:
            os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
        proc.wait(timeout=15)

        # Vérifier que les ports sont libérés
        time.sleep(3)
        all_free = True
        for name, port in [("PostgreSQL", PGSQL_PORT), ("Windmill", WINDMILL_PORT)]:
            if not is_port_free(port):
                print(f"  WARN: Port {port} ({name}) encore occupé après fermeture.")
                all_free = False
        if all_free:
            print("  OK: Tous les ports libérés.")


if __name__ == "__main__":
    main()
