# Architecture du Projet

L'écosystème **IA-Team-Developpement-Workflow** repose sur une architecture native orchestrée par un lanceur Python et une interface Tauri. Ce choix remplace une ancienne architecture basée sur Docker pour offrir une meilleure intégration au système d'exploitation de l'utilisateur.

## Composants Principaux

### 1. Le Lanceur (`run.py`)

Le chef d'orchestre du projet. Il est responsable de la mise en place de l'environnement :

* **Téléchargement** et **décompression** de PostgreSQL portable.
* **Téléchargement** du binaire Windmill pré-compilé adapté au système hôte.
* **Migration automatique** des données : Si une ancienne installation Docker est détectée, le script effectue un dump de la base de données et prépare une restauration.
* **Contrôle des ports** : Vérification de la disponibilité des ports requis.
* **Lancement** de l'application Tauri.

### 2. L'Application Frontend (Tauri + Vue/HTML/JS)

L'interface utilisateur de lancement (le "Launcher").

* Affiche l'état d'initialisation des différents services (PostgreSQL, Windmill Server, Windmill Worker).
* Gère la redirection automatique vers l'interface web de Windmill une fois le système prêt.
* Offre une vue sur les journaux de bord (logs) pour faciliter le débogage.

### 3. Le Backend Rust (Tauri Core)

Le moteur natif qui gère le cycle de vie des processus secondaires (Sidecars).

* **Gestion des processus** : Démarre, surveille et arrête gracieusement PostgreSQL et les instances Windmill (Server et Worker).
* **Restauration des données** : Exécute le script SQL de migration si le flag `PENDING_MIGRATION` est présent.
* **Communication** : Remonte les événements d'état et les journaux de bord au frontend via les mécanismes d'événements de Tauri.

### 4. Les Services Tiers (Sidecars)

* **PostgreSQL** : Base de données relationnelle locale utilisée par Windmill.
* **Windmill** : Plateforme d'exécution de scripts et de flux de travail. Composé d'un "Server" (API et Web Interface) et d'un "Worker" (exécuteur des tâches).

## Schéma Conceptuel

```mermaid
graph TD
    A[run.py (Python)] -->|Télécharge/Extrait| B(PostgreSQL Portable)
    A -->|Télécharge| C(Windmill Binary)
    A -->|Dump DB si Docker actif| D[Dossier MIGRATION]
    A -->|Lance| E[Tauri App (Natif)]
    
    E -->|Frontend (HTML/TS)| F[Interface Utilisateur]
    E -->|Backend (Rust)| G[Gestion des Sidecars]
    
    G -->|Démarre/Arrête| H[Processus PostgreSQL]
    G -->|Restaure si flag| D
    G -->|Démarre/Arrête| I[Windmill Server]
    G -->|Démarre/Arrête| J[Windmill Worker]
    
    H <--> I
    H <--> J
```
