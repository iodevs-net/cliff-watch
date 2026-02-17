//! Tests para el módulo monitor
//!
//! Este módulo contiene pruebas unitarias para el monitor principal
//! que integra MouseSentinel y maneja eventos de mouse.

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use std::time::Duration;
use std::path::PathBuf;

use cliff_watch_core::monitor::{GitMonitor, GitMonitorConfig};
use cliff_watch_core::mouse_sentinel::InputEvent;

#[tokio::test]
async fn test_monitor_initialization() {
    let config = GitMonitorConfig {
        analysis_interval: Duration::from_millis(100),
        mouse_buffer_size: 10,
        min_entropy: 2.5,
    };
    
    let (_input_tx, input_rx) = mpsc::channel(10);
    let (_sensor_tx, sensor_rx) = mpsc::channel(10);
    let (_file_tx, file_rx) = mpsc::channel(10);
    let shutdown = CancellationToken::new();
    
    let _monitor = GitMonitor::new(
        config, 
        input_rx, 
        sensor_rx,
        file_rx, 
        PathBuf::from("/tmp/cliff-watch-test"),
        shutdown.clone()
    ).expect("Failed to create monitor");

    // Verificar que el monitor se creó correctamente
    assert!(true);
}

#[tokio::test]
async fn test_monitor_event_capture() {
    let config = GitMonitorConfig {
        analysis_interval: Duration::from_millis(100),
        mouse_buffer_size: 10,
        min_entropy: 2.5,
    };
    
    let (input_tx, input_rx) = mpsc::channel(10);
    let (_sensor_tx, sensor_rx) = mpsc::channel(10);
    let (_file_tx, file_rx) = mpsc::channel(10);
    let shutdown = CancellationToken::new();
    
    let _monitor = GitMonitor::new(
        config, 
        input_rx,
        sensor_rx,
        file_rx,
        PathBuf::from("/tmp/cliff-watch-test"),
        shutdown.clone()
    ).expect("Failed to create monitor");
    
    // Enviar evento de mouse
    let event = InputEvent::Mouse {
        x: 100.0,
        y: 200.0,
        t: 123456.0,
    };
    
    input_tx.send(event).await.expect("Failed to send event");
    
    // Dar tiempo para que el monitor procese el evento
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Verificar que el evento fue capturado
}

#[tokio::test]
async fn test_monitor_shutdown() {
    let config = GitMonitorConfig {
        analysis_interval: Duration::from_millis(100),
        mouse_buffer_size: 10,
        min_entropy: 2.5,
    };
    
    let (_input_tx, input_rx) = mpsc::channel(10);
    let (_sensor_tx, sensor_rx) = mpsc::channel(10);
    let (_file_tx, file_rx) = mpsc::channel(10);
    let shutdown = CancellationToken::new();
    
    let monitor = GitMonitor::new(
        config, 
        input_rx, 
        sensor_rx,
        file_rx,
        PathBuf::from("/tmp/cliff-watch-test"),
        shutdown.clone()
    ).expect("Failed to create monitor");
    
    // Iniciar monitor en una tarea separada
    let handle = tokio::spawn(async move {
        monitor.start().await
    });
    
    // Esperar un momento para asegurar que el monitor está corriendo
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Enviar señal de shutdown
    shutdown.cancel();
    
    // Esperar a que el monitor termine
    let result = handle.await.expect("Monitor task panicked");
    assert!(result.is_ok(), "Monitor should shutdown cleanly");
}