mod sidecars;

use tauri::Manager;
use std::sync::Mutex;
use std::process::Child as StdChild;
use tauri_plugin_shell::process::CommandChild;

// Structure to hold our running processes
struct AppState {
    pg_process: Mutex<Option<StdChild>>,
    windmill_server: Mutex<Option<CommandChild>>,
    windmill_worker: Mutex<Option<CommandChild>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            pg_process: Mutex::new(None),
            windmill_server: Mutex::new(None),
            windmill_worker: Mutex::new(None),
        })
        .setup(|app| {
            let handle = app.handle();
            let (pg, server, worker) = sidecars::start_services(&handle)
                .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn std::error::Error>)?;
            
            let state = handle.state::<AppState>();
            *state.pg_process.lock().unwrap() = Some(pg);
            *state.windmill_server.lock().unwrap() = Some(server);
            *state.windmill_worker.lock().unwrap() = Some(worker);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit => {
                let state = app_handle.state::<AppState>();
                
                // Kill PostgreSQL
                if let Some(mut pg) = state.pg_process.lock().unwrap().take() {
                    let _ = pg.kill();
                };
                
                // Kill Windmill Server
                if let Some(server) = state.windmill_server.lock().unwrap().take() {
                    let _ = server.kill();
                };
                
                // Kill Windmill Worker
                if let Some(worker) = state.windmill_worker.lock().unwrap().take() {
                    let _ = worker.kill();
                };
            }
            _ => {}
        });
}
