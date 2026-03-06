# Application API Documentation

Currently, the project is a launcher and does not define a traditional REST API. Instead, it relies on sidecars (Windmill and PostgreSQL) and Tauri IPC commands for internal communication.

## Tauri Commands (Rust -> Frontend)

- `greet(name: &str) -> String`: A basic stub command.
- Events emitted:
  - `log-app`: General application logs emitted from Rust to the frontend.
  - `log-migration`: Migration-specific logs emitted from Rust to the frontend.

## Windmill API

The Windmill backend exposes its own API once it is started on Port 8000. Refer to [Windmill Official API Documentation](https://www.windmill.dev/docs) for specific endpoints regarding workflows, scripts, and schedules.
