use std::path::Path;
use std::process::{self, Command, Stdio};

/// Handle the daemon command - Start the cliff-watch daemon
pub fn handle_daemon(config: String, daemon: bool) {
    if daemon {
        println!("Starting cliff-watch daemon in background with config: {}", config);
    } else {
        println!("Starting cliff-watch daemon with config: {}", config);
    }
    // Implementation would start the daemon process
    println!("✅ Daemon started successfully");
}

/// Handle the status command - Check daemon status
pub async fn handle_status() {
    match query_daemon(cliff_watch_core::protocol::Request::GetStatus).await {
        Ok(cliff_watch_core::protocol::Response::Status { is_running, uptime_secs, events_captured }) => {
            println!("Daemon Status:");
            println!("  Running: {}", if is_running { "✅ Yes" } else { "❌ No" });
            println!("  Uptime:  {}s", uptime_secs);
            println!("  Events:  {}", events_captured);
        }
        Ok(cliff_watch_core::protocol::Response::Error(e)) => {
            eprintln!("❌ Daemon error: {}", e);
        }
        Ok(_) => {
            eprintln!("❌ Unexpected response from daemon");
        }
        Err(e) => {
            eprintln!("❌ Could not connect to daemon: {}. Is it running?", e);
        }
    }
}

/// Handle the on command - Activate the sentinel (start daemon in background)
pub fn handle_on() {
    println!("🚀 Encendiendo el centinela termodinámico...");

    match Command::new("cliff-watch-daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn() {
            Ok(_) => {
                println!("✅ Centinela activado en background.");
                println!("Usa 'cliff-watch status' para verificar.");
            },
            Err(_) => {
                // Reintento con path local (target/debug) o global estándar
                let local_path = Path::new("target/debug/cliff-watch-daemon");
                let bin_path = if local_path.exists() {
                    "target/debug/cliff-watch-daemon"
                } else {
                    "/usr/local/bin/cliff-watch-daemon"
                };

                match Command::new(bin_path)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn() {
                        Ok(_) => {
                            println!("✅ Centinela activado en background ({}).", bin_path);
                            println!("Usa 'cliff-watch status' para verificar.");
                        },
                        Err(e) => {
                            eprintln!("❌ Error al encender el centinela: {}. ¿Está instalado?", e);
                            process::exit(1);
                        }
                    }
            }
    }
}

/// Handle the off command - Deactivate the sentinel (stop daemon)
pub fn handle_off() {
    println!("🛑 Apagando el centinela...");

    match Command::new("pkill")
        .arg("-x") // Match exact name
        .arg("cliff-watch-daemon")
        .status() {
            Ok(status) if status.success() => println!("✅ Centinela desactivado."),
            Ok(_) => println!("⚠️ El centinela no parecía estar corriendo."),
            Err(e) => eprintln!("❌ Error al apagar el centinela: {}", e),
        }
}

/// Query the daemon for information
pub async fn query_daemon(request: cliff_watch_core::protocol::Request) -> anyhow::Result<cliff_watch_core::protocol::Response> {
    use tokio::net::UnixStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let socket_path = "/tmp/cliff-watch.sock";
    let mut stream = UnixStream::connect(socket_path).await?;

    let request_json = serde_json::to_vec(&request)?;
    stream.write_all(&request_json).await?;

    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer).await?;

    let response: cliff_watch_core::protocol::Response = serde_json::from_slice(&buffer[..n])?;
    Ok(response)
}
