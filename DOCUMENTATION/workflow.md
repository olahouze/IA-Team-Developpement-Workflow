# Workflow de Lancement

Ce document décrit la séquence d'événements lors du lancement de l'application **IA-Team-Developpement-Workflow**.

## Étape 1 : Initialisation par le Lanceur (`run.py`)

1. **Vérification des binaires** :
    * Le script vérifie si le binaire *PostgreSQL* est présent dans `TAURI/src-tauri/pgsql/bin`. Si non, il le télécharge et l'extrait.
    * Il vérifie si le binaire *Windmill* est présent dans `TAURI/src-tauri/bin`. Si non, il le télécharge.
2. **Vérification de la Migration** :
    * Le script détecte si le conteneur Docker `windmill-db` est en cours d'exécution.
    * Si oui, il crée un dump SQL (`dump.sql`) et le place dans le dossier `MIGRATION/`.
    * Il crée un fichier marqueur `PENDING_MIGRATION`.
    * Il arrête les conteneurs Docker pour libérer les ports.
3. **Vérification des Ports** :
    * Le script s'assure que les ports 1420 (Tauri), 8000 (Windmill) et 5432 (PostgreSQL) sont libres. S'ils sont occupés, il arrête le processus et signale une erreur.
4. **Démarrage de Tauri** :
    * Il lance la commande npm pour démarrer l'application Tauri.

## Étape 2 : Démarrage du Backend Tauri (Rust)

1. **Initialisation de PostgreSQL** :
    * Si la base de données n'est pas initialisée (`pgdata/PG_VERSION` absent), Rust exécute `initdb`.
    * Rust démarre le processus `postgres`.
2. **Restauration (Migration)** :
    * Rust vérifie la présence de `MIGRATION/PENDING_MIGRATION`.
    * S'il existe, il invoque `psql` pour restaurer le fichier `dump.sql`.
    * Une fois terminé, il supprime le dump et le marqueur.
3. **Démarrage de Windmill** :
    * Rust lance le sidecar `windmill` (en tant que serveur).
    * Rust lance le sidecar `windmill worker`.
    * Il observe les sorties (stdout/stderr) de ces processus et émet des événements (`log-app`, `log-migration`) vers le frontend.

## Étape 3 : Chargement du Frontend Tauri (HTML/TS)

1. L'interface graphique s'affiche avec son état initial (Indicateurs visuels des processus).
2. Le frontend écoute les événements provenant de Rust (`log-app`, `log-migration`) pour :
    * Mettre à jour l'affichage des journaux (Logs).
    * Analyser les logs Windmill.
3. (Action Prévue) : Le frontend affiche une barre de progression pendant l'initialisation des services.
4. Une fois Windmill prêt (détecté via l'analyse des logs "Listening on..."), l'application ouvre automatiquement l'URL (<http://localhost:8000>) dans le navigateur par défaut de l'utilisateur.
5. L'interface Tauri maintient l'affichage de l'état des services et de l'historique des logs en temps réel.
