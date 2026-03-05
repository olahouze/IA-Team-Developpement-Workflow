# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "requests",
# ]
# ///

import os
import sys
import platform
import subprocess
import requests
import zipfile
import tarfile
import shutil
import time
import socket

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
TAURI_DIR = os.path.join(BASE_DIR, "TAURI")
MIGRATION_DIR = os.path.join(BASE_DIR, "MIGRATION")
PGSQL_DIR = os.path.join(TAURI_DIR, "src-tauri", "pgsql")

# Ports de l'application
TAURI_PORT = 1420
WINDMILL_PORT = 8000
PGSQL_PORT = 5432

PG_URLS = {
    "Windows": "https://get.enterprisedb.com/postgresql/postgresql-17.2-1-windows-x64-binaries.zip",
    "Darwin": "https://get.enterprisedb.com/postgresql/postgresql-17.2-1-osx-binaries.zip",
    "Linux": "https://get.enterprisedb.com/postgresql/postgresql-17.2-1-linux-x64-binaries.tar.gz"
}

WINDMILL_BINARY_URLS = {
    "Windows": "https://github.com/windmill-labs/windmill/releases/latest/download/windmill-ee.exe",
    "Linux": "https://github.com/windmill-labs/windmill/releases/latest/download/windmill-ee-amd64"
    # Mac ne fournit pas de binaire officiel pré-compilé sur le dépôt Github Release principal.
}

WINDMILL_TARGETS = {
    "Windows": ["windmill-x86_64-pc-windows-gnu.exe", "windmill-x86_64-pc-windows-msvc.exe"],
    "Linux": ["windmill-x86_64-unknown-linux-gnu"]
}

def is_docker_running(container_name):
    try:
        res = subprocess.run(["docker", "ps", "-q", "-f", f"name={container_name}"], capture_output=True, text=True)
        return bool(res.stdout.strip())
    except:
        return False

def download_and_extract_postgres():
    system = platform.system()
    if system not in PG_URLS:
        print(f"❌ OS non supporté pour le téléchargement automatique de PostgreSQL: {system}")
        sys.exit(1)
        
    url = PG_URLS[system]
    
    # Verify if postgres is fully extracted by checking the basic bin dir
    pg_bin_dir = os.path.join(PGSQL_DIR, "bin")
    if os.path.exists(pg_bin_dir) and os.listdir(pg_bin_dir):
        print("✅ PostgreSQL portable est déjà présent.")
        return

    print(f"⬇️ Téléchargement de PostgreSQL pour {system} depuis {url}...")
    
    # Créer le dossier parent si nécessaire
    os.makedirs(os.path.dirname(PGSQL_DIR), exist_ok=True)
    temp_file = os.path.join(BASE_DIR, "pg_archive.tmp")
    
    try:
        with requests.get(url, stream=True) as r:
            r.raise_for_status()
            with open(temp_file, 'wb') as f:
                for chunk in r.iter_content(chunk_size=8192): 
                    f.write(chunk)
                    
        print(f"📦 Extraction de l'archive...")
        if url.endswith(".zip"):
            with zipfile.ZipFile(temp_file, 'r') as zip_ref:
                zip_ref.extractall(os.path.dirname(PGSQL_DIR))
        elif url.endswith(".tar.gz"):
            with tarfile.open(temp_file, 'r:gz') as tar_ref:
                tar_ref.extractall(os.path.dirname(PGSQL_DIR))
                
    finally:
        if os.path.exists(temp_file):
            os.remove(temp_file)
            
    print("✅ PostgreSQL installé avec succès.")

def download_and_install_windmill():
    system = platform.system()
    if system not in WINDMILL_BINARY_URLS:
        print(f"⚠️ Windmill pré-compilé n'est pas disponible pour cet OS: {system}. Veuillez le compiler vous-même et le placer dans {TAURI_DIR}/src-tauri/bin/.")
        return
        
    url = WINDMILL_BINARY_URLS[system]
    target_names = WINDMILL_TARGETS[system]
    bin_dir = os.path.join(TAURI_DIR, "src-tauri", "bin")
    
    # Check if ANY of the targeted sidecars already exist
    for target in target_names:
        if os.path.exists(os.path.join(bin_dir, target)):
            print("✅ Binaire Windmill déjà présent dans src-tauri/bin/.")
            return
            
    print(f"⬇️ Téléchargement du binaire Windmill pour {system} depuis {url}...")
    os.makedirs(bin_dir, exist_ok=True)
    temp_file = os.path.join(BASE_DIR, "windmill.tmp")
    
    try:
        with requests.get(url, stream=True) as r:
            r.raise_for_status()
            with open(temp_file, 'wb') as f:
                for chunk in r.iter_content(chunk_size=8192): 
                    f.write(chunk)
                    
        # Copier le fichier pour chaque cible tauri possible (MSVC et GNU)
        for target in target_names:
            target_path = os.path.join(bin_dir, target)
            shutil.copy2(temp_file, target_path)
            # Rendre exécutable sous Linux
            if system == "Linux":
                os.chmod(target_path, 0o755)
    finally:
        if os.path.exists(temp_file):
            os.remove(temp_file)
            
    print(f"✅ Binaire Windmill installé avec succès pour les cibles Tauri: {', '.join(target_names)}.")

def perform_docker_migration():
    if not is_docker_running("windmill-db"):
        return
        
    print("🚨 Conteneur Docker 'windmill-db' détecté !")
    dump_file = os.path.join(MIGRATION_DIR, "dump.sql")
    
    print("💾 Création du dump de la base de données Docker...")
    # Essaie avec windmill puis postgres
    res = subprocess.run(["docker", "exec", "windmill-db", "pg_dump", "-U", "windmill", "-d", "windmill", "-c", "-f", "/tmp/dump.sql"], capture_output=True)
    if res.returncode != 0:
        res = subprocess.run(["docker", "exec", "windmill-db", "pg_dump", "-U", "postgres", "-d", "postgres", "-c", "-f", "/tmp/dump.sql"])
        
    if res.returncode == 0:
        subprocess.run(["docker", "cp", f"windmill-db:/tmp/dump.sql", dump_file])
        print(f"✅ Dump sauvegardé dans {dump_file}")
        
        # Write a flag file that Tauri will read to know it needs to restore
        flag_file = os.path.join(MIGRATION_DIR, "PENDING_MIGRATION")
        with open(flag_file, "w") as f:
            f.write(dump_file)
    else:
        print("❌ Échec du dump de la base Docker.")
    
    print("🛑 Arrêt des conteneurs Docker pour libérer les ports (5432, 8000)...")
    subprocess.run(["docker", "stop", "windmill-db"])
    subprocess.run(["docker", "stop", "windmill-server", "windmill-worker"], stderr=subprocess.DEVNULL)

def start_tauri():
    print("🚀 Démarrage de l'interface Tauri...")
    os.chdir(TAURI_DIR)
    
    # Pour le dev, on utilise npm. Si on a buildé le binaire on pourrait lancer le .exe
    # Sur pc windows on utilise npm.cmd
    npm_cmd = "npm.cmd" if platform.system() == "Windows" else "npm"
    
    # Nettoyage et installation systématique des dépendances NPM
    print("🧹 Nettoyage du cache NPM...")
    subprocess.run([npm_cmd, "cache", "clean", "--force"])
    print("📦 Installation / Mise à jour des dépendances NPM...")
    subprocess.run([npm_cmd, "install"])

    subprocess.run([npm_cmd, "run", "tauri", "dev"])

def check_ports():
    print("🔍 Vérification des ports...")
    ports = {
        "Tauri": TAURI_PORT,
        "Windmill": WINDMILL_PORT,
        "PostgreSQL": PGSQL_PORT
    }

    # 1. Vérifier si les ports sont distincts
    port_names = {}
    for name, port in ports.items():
        if port in port_names:
            print(f"❌ Impossible de lancer l'application : Le port {port} est configuré pour '{name}' et '{port_names[port]}'.")
            print("Les ports doivent impérativement être différents. Veuillez modifier les paramètres.")
            sys.exit(1)
        port_names[port] = name

    # 2. Vérifier si les ports sont déjà utilisés par un autre programme (hors docker qu'on vient de stopper)
    in_use = []
    for name, port in ports.items():
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.settimeout(1)
            # connect_ex renvoie 0 si la connexion réussit (port utilisé)
            if s.connect_ex(('127.0.0.1', port)) == 0:
                in_use.append((name, port))

    if in_use:
        for name, port in in_use:
            print(f"❌ Erreur critique : Le port {port} (prévu pour {name}) est déjà utilisé par une autre application système.")
        print("\n➡️ Impossible de démarrer. Veuillez libérer ces ports ou changer leur configuration.")
        sys.exit(1)
    
    print("✅ Configuration des ports vérifiée et validée.")

def main():
    print("=== Native Windmill Launcher ===")
    os.makedirs(MIGRATION_DIR, exist_ok=True)
    
    download_and_extract_postgres()
    download_and_install_windmill()
    perform_docker_migration()
    check_ports()
    start_tauri()

if __name__ == "__main__":
    main()
