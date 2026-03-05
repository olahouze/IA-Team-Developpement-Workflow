# Native Windmill Launcher

Ce dossier contient le script principal de lancement et de configuration de l'écosystème **IA-Team-Developpement-Workflow**.

Le script `run.py` automatise l'installation des dépendances lourdes (PostgreSQL, Windmill) et gère la migration des données depuis une installation Docker vers une exécution native via **Tauri**.

## 📋 Prérequis

- **Python 3.11+**
- **Node.js & npm** (pour l'environnement Tauri)
- **uv** (recommandé pour l'exécution rapide des scripts Python)
- **Docker** (uniquement nécessaire si vous souhaitez migrer vos données existantes)

## 🚀 Utilisation

Pour lancer l'environnement complet, utilisez la commande suivante depuis ce dossier :

```powershell
uv run run.py
```

*Note : Vous pouvez aussi utiliser `python run.py`, mais assurez-vous d'avoir installé les dépendances listées dans les commentaires `inline metadata` du script (`requests`).*

## 🛠️ Fonctionnalités du script

### 1. Installation automatique de PostgreSQL

Le script télécharge et extrait une version **portable** de PostgreSQL (v17.2) adaptée à votre système d'exploitation (Windows/Linux/macOS) dans le dossier :
`TAURI/src-tauri/pgsql/`

### 2. Configuration du binaire Windmill

Il récupère la dernière version du binaire Windmill (Sidecar) correspondant à votre architecture et le place directement dans :
`TAURI/src-tauri/bin/`

### 3. Migration (Docker vers Natif)

Si un conteneur Docker nommé `windmill-db` est détecté en cours d'exécution :

1. Un dump de la base de données Docker est effectué.
2. Le dump est sauvegardé dans le dossier `MIGRATION/`.
3. Un flag `PENDING_MIGRATION` est créé pour que l'application Tauri restaure ces données au premier démarrage.
4. Les conteneurs Docker (DB, Server, Worker) sont arrêtés pour libérer les ports **5432** et **8000**.

### 4. Lancement de l'interface

Une fois la configuration terminée, le script lance automatiquement l'environnement de développement Tauri (`npm run tauri dev`).

## 📁 Structure du dossier

- **/TAURI** : Contient le code source de l'application frontend et backend Rust.
- **/MIGRATION** : Dossier de transit pour les sauvegardes de base de données en vue d'une migration.
- **run.py** : Le chef d'orchestre du projet.
