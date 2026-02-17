//! Backend de Sensor IDE para Cliff-Watch v2.0
//!
//! Este backend escucha eventos de extensiones de IDE (VS Code, JetBrains, etc.)
//! a través de un Unix Domain Socket dedicado.
//!
//! ## Arquitectura
//! - Socket: `/tmp/cliff-watch-sensor.sock` (configurable)
//! - Protocolo: JSON newline-delimited
//! - Cada conexión representa una instancia de IDE
//!
//! ## Ventajas sobre evdev
//! - NO requiere permisos de root
//! - NO captura inputs globales (solo del IDE)
//! - Respeta la privacidad del usuario

use crate::focus_protocol::{SensorEvent, SensorResponse};
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

/// Ruta por defecto del socket de sensores
pub const DEFAULT_SENSOR_SOCKET: &str = "/tmp/cliff-watch-sensor.sock";

/// Backend para recibir eventos de extensiones de IDE
pub struct IdeSensorBackend {
    socket_path: PathBuf,
}

impl IdeSensorBackend {
    /// Crea un nuevo backend con la ruta de socket especificada
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
        }
    }

    /// Obtiene la ruta del socket
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Inicia el servidor de sensores
    ///
    /// Escucha conexiones de sensores IDE y envía los eventos recibidos
    /// a través del canal `event_tx`.
    ///
    /// ## Parámetros
    /// - `event_tx`: Canal para enviar eventos al monitor principal
    /// - `shutdown`: Token de cancelación para detener el servidor
    pub async fn start(
        &self,
        event_tx: mpsc::Sender<SensorEvent>,
        shutdown: CancellationToken,
    ) -> Result<()> {
        // Limpiar socket anterior si existe
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)
                .context("Failed to remove existing sensor socket")?;
        }

        let listener = UnixListener::bind(&self.socket_path)
            .context("Failed to bind sensor socket")?;

        info!(
            "IDE Sensor Backend listening on {:?}",
            self.socket_path
        );

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    info!("Sensor backend shutting down");
                    break;
                }

                result = listener.accept() => {
                    match result {
                        Ok((stream, _addr)) => {
                            let tx = event_tx.clone();
                            let cancel = shutdown.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_sensor_connection(stream, tx, cancel).await {
                                    warn!("Sensor connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept sensor connection: {}", e);
                        }
                    }
                }
            }
        }

        // Cleanup socket on shutdown
        let _ = std::fs::remove_file(&self.socket_path);
        Ok(())
    }
}

impl Default for IdeSensorBackend {
    fn default() -> Self {
        Self::new(DEFAULT_SENSOR_SOCKET)
    }
}

/// Maneja una conexión individual de un sensor IDE
async fn handle_sensor_connection(
    stream: UnixStream,
    event_tx: mpsc::Sender<SensorEvent>,
    shutdown: CancellationToken,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    info!("New IDE sensor connected");

    loop {
        line.clear();

        tokio::select! {
            _ = shutdown.cancelled() => {
                break;
            }

            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        // EOF - cliente desconectado
                        debug!("Sensor disconnected (EOF)");
                        break;
                    }
                    Ok(_) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }

                        // Parsear el evento JSON
                        match SensorEvent::from_json(trimmed) {
                            Ok(event) => {
                                debug!("Received sensor event: {:?}", event);

                                // Detectar desconexión explícita
                                let is_disconnect = matches!(event, SensorEvent::Disconnect { .. });

                                // Enviar evento al monitor
                                if event_tx.send(event).await.is_err() {
                                    warn!("Event channel closed");
                                    break;
                                }

                                // Responder con ACK (battery level placeholder)
                                let response = SensorResponse::Ack { battery_level: 50.0 };
                                if let Ok(json) = response.to_json() {
                                    let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
                                    let _ = writer.flush().await;
                                }

                                if is_disconnect {
                                    break;
                                }
                            }
                            Err(e) => {
                                warn!("Invalid sensor event JSON: {} - {}", trimmed, e);
                                let response = SensorResponse::Error {
                                    message: format!("Invalid JSON: {}", e),
                                };
                                if let Ok(json) = response.to_json() {
                                    let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
                                    let _ = writer.flush().await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading from sensor: {}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::UnixStream;
    use std::time::Duration;

    #[tokio::test]
    async fn test_backend_creation() {
        let backend = IdeSensorBackend::default();
        assert_eq!(backend.socket_path().to_str().unwrap(), DEFAULT_SENSOR_SOCKET);
    }

    #[tokio::test]
    async fn test_sensor_socket_lifecycle() {
        let socket_path = "/tmp/cliff-watch-sensor-test.sock";
        let backend = IdeSensorBackend::new(socket_path);
        
        // Asegurar que no existe
        let _ = std::fs::remove_file(socket_path);
        
        let (tx, mut rx) = mpsc::channel(16);
        let shutdown = CancellationToken::new();
        let shutdown_clone = shutdown.clone();

        // Iniciar backend en background
        let handle = tokio::spawn(async move {
            backend.start(tx, shutdown_clone).await
        });

        // Dar tiempo al socket para estar listo
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Conectar como sensor
        let mut stream = UnixStream::connect(socket_path).await.expect("connect failed");

        // Enviar evento de foco
        let event = SensorEvent::FocusGained {
            file_path: Some("/test/main.rs".to_string()),
            timestamp_ms: 12345,
        };
        let json = format!("{}\n", event.to_json().unwrap());
        stream.write_all(json.as_bytes()).await.unwrap();

        // Verificar que llegó
        let received = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("timeout")
            .expect("no event");

        assert!(matches!(received, SensorEvent::FocusGained { .. }));

        // Shutdown
        shutdown.cancel();
        let _ = handle.await;

        // Cleanup
        let _ = std::fs::remove_file(socket_path);
    }
}
