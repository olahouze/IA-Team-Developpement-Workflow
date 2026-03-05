use std::process::{Child, Command, Stdio};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

pub fn start_services(app: &AppHandle) -> Result<(Child, tauri_plugin_shell::process::CommandChild, tauri_plugin_shell::process::CommandChild), String> {
    // 1. Resolve PostgreSQL paths
    // The Python runner downloads it to TAURI/src-tauri/pgsql
    // In dev, let's look at relative path, in prod we would need to know where it's deployed.
    // For now, let's resolve relative to the current working directory, which should be the root or TAURI folder
    let mut pgsql_base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if !pgsql_base.join("src-tauri").join("pgsql").exists() {
        // We might be running from inside TAURI/src-tauri
        if std::path::Path::new("pgsql").exists() {
            pgsql_base = PathBuf::from(".");
        } else if std::path::Path::new("../TAURI/src-tauri/pgsql").exists() {
            pgsql_base = PathBuf::from("../TAURI/src-tauri");
        } else {
             // Fallback to resource dir logic just in case
             if let Ok(res_dir) = app.path().resolve("pgsql", tauri::path::BaseDirectory::Resource) {
                 pgsql_base = res_dir.parent().unwrap_or(&res_dir).to_path_buf();
             }
        }
    } else {
        pgsql_base = pgsql_base.join("src-tauri");
    }

    let resource_dir = pgsql_base.join("pgsql");
    let initdb_exe = resource_dir.join("bin").join(if cfg!(windows) { "initdb.exe" } else { "initdb" });
    let postgres_exe = resource_dir.join("bin").join(if cfg!(windows) { "postgres.exe" } else { "postgres" });
    let psql_exe = resource_dir.join("bin").join(if cfg!(windows) { "psql.exe" } else { "psql" });
    
    // Data directory in AppData
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Could not find app_data_dir: {}", e))?;
    let pg_data_dir = app_data_dir.join("pgdata");

    // Initialize DB if not exists
    if !pg_data_dir.exists() {
        std::fs::create_dir_all(&pg_data_dir).map_err(|e| e.to_string())?;
        
        let status = Command::new(&initdb_exe)
            .arg("-D")
            .arg(&pg_data_dir)
            .arg("-U")
            .arg("postgres")
            .arg("--auth=trust")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .status()
            .map_err(|e| format!("Failed to run initdb: {}", e))?;
            
        if !status.success() {
            return Err("initdb failed".to_string());
        }
    }

    // Start PostgreSQL
    let mut pg_child = Command::new(&postgres_exe)
        .arg("-D")
        .arg(&pg_data_dir)
        .arg("-p")
        .arg("5432")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start PostgreSQL: {}", e))?;

    // Wait a brief moment to let PG accept connections
    std::thread::sleep(std::time::Duration::from_secs(3));
    
    // Check for pending migration
    let migration_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("..").join("MIGRATION");
    let pending_file = migration_dir.join("PENDING_MIGRATION");
    
    if pending_file.exists() {
        if let Ok(dump_path) = std::fs::read_to_string(&pending_file) {
            let dump_path = dump_path.trim();
            println!("Restoring database from {}...", dump_path);
            let status = Command::new(&psql_exe)
                .arg("-h").arg("localhost")
                .arg("-p").arg("5432")
                .arg("-U").arg("postgres")
                .arg("-d").arg("postgres")
                .arg("-f").arg(dump_path)
                .status();
                
            if let Ok(s) = status {
                if s.success() {
                    println!("Migration restored successfully.");
                    let _ = std::fs::remove_file(pending_file);
                    let _ = std::fs::remove_file(dump_path);
                } else {
                    println!("Failed to restore migration.");
                }
            }
        }
    }

    // Env vars for Windmill
    let database_url = "postgres://postgres@localhost:5432/postgres";
    
    // Start Windmill Server (Sidecar)
    let (mut rx_server, server_child) = app.shell()
        .sidecar("windmill")
        .map_err(|e| e.to_string())?
        .env("DATABASE_URL", database_url)
        .spawn()
        .map_err(|e| e.to_string())?;

    // Log Server output in background
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx_server.recv().await {
            if let CommandEvent::Stdout(line) = event {
                println!("Windmill Server: {}", String::from_utf8_lossy(&line));
            }
        }
    });

    // Start Windmill Worker (Sidecar)
    let (mut rx_worker, worker_child) = app.shell()
        .sidecar("windmill")
        .map_err(|e| e.to_string())?
        .arg("worker")
        .env("DATABASE_URL", database_url)
        .spawn()
        .map_err(|e| e.to_string())?;

    // Log Worker output in background
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx_worker.recv().await {
            if let CommandEvent::Stdout(line) = event {
                println!("Windmill Worker: {}", String::from_utf8_lossy(&line));
            }
        }
    });

    Ok((pg_child, server_child, worker_child))
}
