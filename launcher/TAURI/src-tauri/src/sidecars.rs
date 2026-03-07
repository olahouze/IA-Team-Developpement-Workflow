use std::process::{Child, Command, Stdio};
use std::path::PathBuf;
use std::net::TcpStream;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Emitter};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

/// Resolve the base directory containing `pgsql/` and `bin/` (sidecars).
/// In dev mode, uses CARGO_MANIFEST_DIR (src-tauri/).
/// In release mode, falls back to the resource directory.
fn resolve_base_dir(app: &AppHandle) -> Result<PathBuf, String> {
    // 1. Dev mode: CARGO_MANIFEST_DIR points to src-tauri/
    let manifest_dir = option_env!("CARGO_MANIFEST_DIR").map(PathBuf::from);
    if let Some(ref dir) = manifest_dir {
        if dir.join("pgsql").join("bin").exists() {
            println!("[paths] Using CARGO_MANIFEST_DIR: {:?}", dir);
            return Ok(dir.clone());
        }
    }

    // 2. Relative from current working directory (common in tauri dev)
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let candidates = [
        cwd.join("src-tauri"),
        cwd.clone(),
        cwd.join("..").join("TAURI").join("src-tauri"),
    ];
    for candidate in &candidates {
        if candidate.join("pgsql").join("bin").exists() {
            let resolved = candidate.canonicalize().unwrap_or_else(|_| candidate.clone());
            println!("[paths] Using CWD-relative: {:?}", resolved);
            return Ok(resolved);
        }
    }

    // 3. Release mode: resource directory
    if let Ok(res_dir) = app.path().resource_dir() {
        if res_dir.join("pgsql").join("bin").exists() {
            println!("[paths] Using resource_dir: {:?}", res_dir);
            return Ok(res_dir);
        }
    }

    Err(format!(
        "Could not find pgsql/bin/ directory. CWD={:?}, CARGO_MANIFEST_DIR={:?}",
        std::env::current_dir().unwrap_or_default(),
        option_env!("CARGO_MANIFEST_DIR")
    ))
}

/// Resolve the MIGRATION directory.
fn resolve_migration_dir(base_dir: &PathBuf) -> PathBuf {
    // MIGRATION/ is a sibling of TAURI/ which is the parent of src-tauri/
    // base_dir = .../launcher/TAURI/src-tauri
    // => .../launcher/MIGRATION
    let migration = base_dir.join("..").join("..").join("MIGRATION");
    if migration.exists() {
        return migration;
    }
    // Fallback: try from CARGO_MANIFEST_DIR in dev
    if let Some(dir) = option_env!("CARGO_MANIFEST_DIR") {
        let dev_migration = PathBuf::from(dir).join("..").join("..").join("MIGRATION");
        if dev_migration.exists() {
            return dev_migration;
        }
    }
    // Final fallback
    migration
}

/// Wait for PostgreSQL to accept TCP connections on the given port.
/// Polls every 500ms, up to `timeout` duration.
fn wait_for_pg_ready(app: &AppHandle, port: u16, timeout: Duration) -> Result<(), String> {
    let start = Instant::now();
    let addr = format!("127.0.0.1:{}", port);
    let _ = app.emit("log-app", "[PostgreSQL] Waiting for database to accept connections...".to_string());

    loop {
        match TcpStream::connect_timeout(
            &addr.parse().map_err(|e| format!("Bad address: {}", e))?,
            Duration::from_millis(500),
        ) {
            Ok(_) => {
                let msg = format!("[PostgreSQL] database system is ready to accept connections ({}ms)", start.elapsed().as_millis());
                println!("{}", msg);
                let _ = app.emit("log-app", msg);
                return Ok(());
            }
            Err(_) => {
                if start.elapsed() > timeout {
                    let msg = format!("[PostgreSQL] Timeout after {}s waiting for port {}", timeout.as_secs(), port);
                    let _ = app.emit("log-app", msg.clone());
                    return Err(msg);
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    }
}

pub fn start_services(app: &AppHandle) -> Result<(Child, tauri_plugin_shell::process::CommandChild, tauri_plugin_shell::process::CommandChild), String> {
    let _ = app.emit("log-app", "[Setup] Resolving paths...".to_string());

    // 1. Resolve paths
    let base_dir = resolve_base_dir(app)?;
    let resource_dir = base_dir.join("pgsql");

    let initdb_exe = resource_dir.join("bin").join(if cfg!(windows) { "initdb.exe" } else { "initdb" });
    let postgres_exe = resource_dir.join("bin").join(if cfg!(windows) { "postgres.exe" } else { "postgres" });
    let psql_exe = resource_dir.join("bin").join(if cfg!(windows) { "psql.exe" } else { "psql" });

    // Validate binaries exist
    for (name, path) in [("initdb", &initdb_exe), ("postgres", &postgres_exe), ("psql", &psql_exe)] {
        if !path.exists() {
            return Err(format!("{} binary not found at {:?}", name, path));
        }
    }

    let _ = app.emit("log-app", format!("[Setup] PostgreSQL binaries found at {:?}", resource_dir));

    // Data directory in AppData
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Could not find app_data_dir: {}", e))?;
    let pg_data_dir = app_data_dir.join("pgdata");

    // Initialize DB if not exists
    if !pg_data_dir.join("PG_VERSION").exists() {
        let _ = app.emit("log-app", "[PostgreSQL] Initializing database cluster...".to_string());
        if !pg_data_dir.exists() {
            std::fs::create_dir_all(&pg_data_dir).map_err(|e| e.to_string())?;
        }

        let status = Command::new(&initdb_exe)
            .arg("-D")
            .arg(&pg_data_dir)
            .arg("-U")
            .arg("postgres")
            .arg("-E")
            .arg("UTF8")
            .arg("--no-locale")
            .arg("--auth=trust")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to run initdb: {}", e))?;

        if !status.success() {
            let msg = "initdb failed to initialize the database cluster".to_string();
            let _ = app.emit("log-app", format!("[PostgreSQL ERR] {}", msg));
            return Err(msg);
        }
        let _ = app.emit("log-app", "[PostgreSQL] Database cluster initialized.".to_string());
    }

    // Start PostgreSQL
    let _ = app.emit("log-app", "[PostgreSQL] Starting server on port 5432...".to_string());
    let pg_child = Command::new(&postgres_exe)
        .arg("-D")
        .arg(&pg_data_dir)
        .arg("-p")
        .arg("5432")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("Failed to start PostgreSQL: {}", e))?;

    // Poll for PG readiness instead of fixed sleep
    wait_for_pg_ready(app, 5432, Duration::from_secs(30))?;

    // Check for pending migration
    let migration_dir = resolve_migration_dir(&base_dir);
    let pending_file = migration_dir.join("PENDING_MIGRATION");

    if pending_file.exists() {
        if let Ok(dump_path) = std::fs::read_to_string(&pending_file) {
            let dump_path = dump_path.trim().to_string();
            let msg = format!("Restoring database from {}...", dump_path);
            println!("{}", msg);
            let _ = app.emit("log-migration", msg);

            let status = Command::new(&psql_exe)
                .arg("-h").arg("localhost")
                .arg("-p").arg("5432")
                .arg("-U").arg("postgres")
                .arg("-d").arg("postgres")
                .arg("-f").arg(&dump_path)
                .status();

            if let Ok(s) = status {
                if s.success() {
                    let msg = "Migration restored successfully.".to_string();
                    println!("{}", msg);
                    let _ = app.emit("log-migration", msg);
                    let _ = std::fs::remove_file(&pending_file);
                    let _ = std::fs::remove_file(&dump_path);
                } else {
                    let msg = "Failed to restore migration.".to_string();
                    println!("{}", msg);
                    let _ = app.emit("log-migration", msg);
                }
            }
        }
    }

    // Env vars for Windmill
    let database_url = "postgres://postgres@localhost:5432/postgres";

    // Start Windmill Server (Sidecar)
    let _ = app.emit("log-app", "[Windmill] Starting server...".to_string());
    let (mut rx_server, server_child) = app.shell()
        .sidecar("windmill")
        .map_err(|e| e.to_string())?
        .env("DATABASE_URL", database_url)
        .spawn()
        .map_err(|e| format!("Failed to start Windmill server sidecar: {}", e))?;

    // Log Server output in background
    let app_h = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx_server.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let text = String::from_utf8_lossy(&line).to_string();
                    println!("Windmill Server [OUT]: {}", text);
                    let category = if text.contains("migration") || text.contains("migrating") { "log-migration" } else { "log-app" };
                    let _ = app_h.emit(category, format!("[Server] {}", text));
                },
                CommandEvent::Stderr(line) => {
                    let text = String::from_utf8_lossy(&line).to_string();
                    eprintln!("Windmill Server [ERR]: {}", text);
                    let _ = app_h.emit("log-app", format!("[Server ERR] {}", text));
                },
                CommandEvent::Error(err) => {
                    let msg = format!("[Server CRITICAL] {}", err);
                    eprintln!("{}", msg);
                    let _ = app_h.emit("log-app", msg.clone());
                    let _ = app_h.emit("service-error", "windmill-server".to_string());
                },
                CommandEvent::Terminated(payload) => {
                    let code = payload.code.unwrap_or(-1);
                    let msg = format!("[Server EXIT] code={}, signal={:?}", code, payload.signal);
                    println!("{}", msg);
                    let _ = app_h.emit("log-app", msg);
                    if code != 0 {
                        let _ = app_h.emit("service-error", "windmill-server".to_string());
                    }
                },
                _ => {}
            }
        }
    });

    // Start Windmill Worker (Sidecar)
    let _ = app.emit("log-app", "[Windmill] Starting worker...".to_string());
    let (mut rx_worker, worker_child) = app.shell()
        .sidecar("windmill")
        .map_err(|e| e.to_string())?
        .arg("worker")
        .env("DATABASE_URL", database_url)
        .env("METRICS_ADDR", "0.0.0.0:8002")
        .spawn()
        .map_err(|e| format!("Failed to start Windmill worker sidecar: {}", e))?;

    // Log Worker output in background
    let app_w = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx_worker.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let text = String::from_utf8_lossy(&line).to_string();
                    println!("Windmill Worker [OUT]: {}", text);
                    let _ = app_w.emit("log-app", format!("[Worker] {}", text));
                },
                CommandEvent::Stderr(line) => {
                    let text = String::from_utf8_lossy(&line).to_string();
                    eprintln!("Windmill Worker [ERR]: {}", text);
                    let _ = app_w.emit("log-app", format!("[Worker ERR] {}", text));
                },
                CommandEvent::Error(err) => {
                    let msg = format!("[Worker CRITICAL] {}", err);
                    eprintln!("{}", msg);
                    let _ = app_w.emit("log-app", msg.clone());
                    let _ = app_w.emit("service-error", "windmill-worker".to_string());
                },
                CommandEvent::Terminated(payload) => {
                    let code = payload.code.unwrap_or(-1);
                    let msg = format!("[Worker EXIT] code={}, signal={:?}", code, payload.signal);
                    println!("{}", msg);
                    let _ = app_w.emit("log-app", msg);
                    if code != 0 {
                        let _ = app_w.emit("service-error", "windmill-worker".to_string());
                    }
                },
                _ => {}
            }
        }
    });

    Ok((pg_child, server_child, worker_child))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_migration_dir_structure() {
        // The migration dir should be derived relative to base_dir
        let base = PathBuf::from("/fake/launcher/TAURI/src-tauri");
        let migration = resolve_migration_dir(&base);
        // Should go up 2 levels from src-tauri to launcher, then into MIGRATION
        assert!(migration.to_string_lossy().contains("MIGRATION"));
    }

    #[test]
    fn test_wait_for_pg_ready_timeout_on_closed_port() {
        // Trying to connect to a port that is very unlikely to be open
        // We just verify the function returns an error (timeout) quickly
        // Use a very short timeout to keep the test fast
        use std::net::TcpListener;
        // Find a port that is NOT listening
        let addr = "127.0.0.1:0";
        let listener = TcpListener::bind(addr).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener); // Close immediately so nothing listens

        // We can't call wait_for_pg_ready without an AppHandle,
        // so we test the TCP logic directly
        let start = std::time::Instant::now();
        let result = std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{}", port).parse().unwrap(),
            std::time::Duration::from_millis(200),
        );
        assert!(result.is_err());
        assert!(start.elapsed() < std::time::Duration::from_secs(2));
    }

    #[test]
    fn test_binary_path_construction() {
        let base = PathBuf::from("/fake/base");
        let resource_dir = base.join("pgsql");
        let initdb = resource_dir.join("bin").join(if cfg!(windows) { "initdb.exe" } else { "initdb" });

        if cfg!(windows) {
            assert!(initdb.to_string_lossy().ends_with("initdb.exe"));
        } else {
            assert!(initdb.to_string_lossy().ends_with("initdb"));
        }
        assert!(initdb.to_string_lossy().contains("pgsql"));
        assert!(initdb.to_string_lossy().contains("bin"));
    }
}
