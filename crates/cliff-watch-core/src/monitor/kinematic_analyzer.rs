//! Mouse kinematic analysis via MouseSentinel integration.

use super::battery_manager::AttentionBattery;
use super::file_watcher::EditEvent;
use crate::complexity::{estimate_entropic_cost, calculate_compression_ratio, calculate_ncd_against_context};
use crate::focus_protocol::SensorEvent;
use crate::focus_session::FocusTracker;
use crate::mouse_sentinel::{InputEvent, KinematicMetrics, MouseSentinel};
use crate::stats::calculate_coupling_score;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

/// Configuration for GitMonitor (kinematic analysis).
#[derive(Debug, Clone)]
pub struct GitMonitorConfig {
    pub analysis_interval: Duration,
    pub mouse_buffer_size: usize,
    pub min_entropy: f64,
}

impl Default for GitMonitorConfig {
    fn default() -> Self {
        Self {
            analysis_interval: Duration::from_secs(5),
            mouse_buffer_size: 1024,
            min_entropy: 2.5,
        }
    }
}

/// GitMonitor for mouse kinematic analysis and cognitive coupling.
pub struct GitMonitor {
    shutdown: CancellationToken,
    mouse_sentinel: MouseSentinel,
    input_rx: mpsc::Receiver<InputEvent>,
    sensor_rx: mpsc::Receiver<SensorEvent>,
    file_rx: mpsc::Receiver<EditEvent>,
    analysis_interval: Duration,
    latest_metrics: Arc<RwLock<Option<KinematicMetrics>>>,
    latest_coupling: Arc<RwLock<f64>>,
    battery: Arc<RwLock<AttentionBattery>>,
    events_captured: Arc<RwLock<usize>>,
    keyboard_hits: Arc<AtomicU64>,
    watch_root: PathBuf,
    min_entropy: f64,
    focus_tracker: Arc<RwLock<FocusTracker>>,
    score_history: Arc<RwLock<VecDeque<f64>>>,
    latest_ncd: Arc<RwLock<f64>>,
    repo_context_cache: Arc<RwLock<(SystemTime, String)>>,
}

impl GitMonitor {
    pub fn new(
        config: GitMonitorConfig,
        input_rx: mpsc::Receiver<InputEvent>,
        sensor_rx: mpsc::Receiver<SensorEvent>,
        file_rx: mpsc::Receiver<EditEvent>,
        watch_root: PathBuf,
        shutdown: CancellationToken,
    ) -> Result<Self, GitMonitorError> {
        Ok(Self {
            shutdown,
            mouse_sentinel: MouseSentinel::new(config.mouse_buffer_size),
            input_rx,
            sensor_rx,
            file_rx,
            analysis_interval: config.analysis_interval,
            latest_metrics: Arc::new(RwLock::new(None)),
            latest_coupling: Arc::new(RwLock::new(1.0)),
            battery: Arc::new(RwLock::new(AttentionBattery::new())),
            events_captured: Arc::new(RwLock::new(0)),
            keyboard_hits: Arc::new(AtomicU64::new(0)),
            watch_root,
            min_entropy: config.min_entropy,
            focus_tracker: Arc::new(RwLock::new(FocusTracker::new())),
            score_history: Arc::new(RwLock::new(VecDeque::with_capacity(50))),
            latest_ncd: Arc::new(RwLock::new(0.5)), // Start neutral to avoid bias
            repo_context_cache: Arc::new(RwLock::new((UNIX_EPOCH, String::new()))),
        })
    }

    pub fn get_focus_metrics(&self) -> crate::focus_session::FocusMetrics {
        self.focus_tracker
            .read()
            .expect("Focus tracker RwLock poisoned")
            .get_metrics()
    }

    pub fn get_metrics_ref(&self) -> Arc<RwLock<Option<KinematicMetrics>>> {
        self.latest_metrics.clone()
    }

    pub fn get_battery_ref(&self) -> Arc<RwLock<AttentionBattery>> {
        self.battery.clone()
    }

    pub fn get_coupling_ref(&self) -> Arc<RwLock<f64>> {
        self.latest_coupling.clone()
    }

    pub fn get_focus_tracker_ref(&self) -> Arc<RwLock<FocusTracker>> {
        self.focus_tracker.clone()
    }

    pub fn get_events_captured_ref(&self) -> Arc<RwLock<usize>> {
        self.events_captured.clone()
    }

    pub fn get_score_history_ref(&self) -> Arc<RwLock<VecDeque<f64>>> {
        self.score_history.clone()
    }

    pub fn get_ncd_ref(&self) -> Arc<RwLock<f64>> {
        self.latest_ncd.clone()
    }

    pub async fn start(mut self) -> Result<(), GitMonitorError> {
        info!("GovMonitor (Cliff-Watch) started with Cognitive Coupling");
        let mut interval = tokio::time::interval(self.analysis_interval);

        loop {
            tokio::select! {
                _ = self.shutdown.cancelled() => {
                    info!("Shutdown signal received");
                    break;
                }

                Some(event) = self.input_rx.recv() => {
                    self.handle_input_event(event);
                }

                Some(sensor_event) = self.sensor_rx.recv() => {
                    self.handle_sensor_event(sensor_event);
                }

                Some(file_event) = self.file_rx.recv() => {
                    self.handle_file_event(file_event).await;
                }

                _ = interval.tick() => {
                    self.run_analysis();
                }
            }
        }

        info!("GitMonitor stopped cleanly");
        Ok(())
    }

    fn handle_input_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::Mouse { x, y, .. } => {
                self.mouse_sentinel.capture_event(x, y);
            }
            InputEvent::Keyboard { .. } => {
                self.keyboard_hits.fetch_add(1, Ordering::SeqCst);
            }
        }
        if let Ok(mut count) = self.events_captured.write() {
            *count += 1;
        }
    }

    fn handle_sensor_event(&mut self, event: SensorEvent) {
        if let Ok(mut tracker) = self.focus_tracker.write() {
            match event {
                SensorEvent::FocusGained { file_path, .. } => {
                    tracker.focus_gained(file_path.clone().map(PathBuf::from));
                    info!("Focus Gained: {:?}", file_path);
                }
                SensorEvent::FocusLost { .. } => {
                    tracker.focus_lost();
                    info!("Focus Lost");
                }
                SensorEvent::EditBurst { file_path, chars_delta, .. } => {
                    tracker.edit_burst(&file_path, chars_delta);
                }
                SensorEvent::Navigation {
                    file_path,
                    nav_type,
                    timestamp_ms,
                    ..
                } => {
                    tracker.navigation(&file_path, timestamp_ms);
                    if nav_type == crate::focus_protocol::NavigationType::FileSwitch {
                        info!("Switched to file: {}", file_path);
                    }
                }
                SensorEvent::Heartbeat { .. } => {
                    tracker.heartbeat();
                }
                SensorEvent::Disconnect { .. } => {
                    tracker.reset();
                    warn!("IDE Sensor disconnected");
                }
                SensorEvent::Keystroke { .. } => {
                    // Por ahora solo notificamos presencia cinemática, 
                    // en el futuro esto alimentará el motor de entropía NCD.
                    tracker.heartbeat();
                }
            }
        }
    }

    async fn get_repo_context_cached(&self) -> String {
        const CACHE_TTL: Duration = Duration::from_secs(300); // 5 min
        
        if let Ok(cache) = self.repo_context_cache.read() {
            if cache.0.elapsed().unwrap_or(CACHE_TTL) < CACHE_TTL && !cache.1.is_empty() {
                return cache.1.clone(); // Cache hit
            }
        }
        
        // Cache miss: regenerar
        let mut context = String::new();
        let mut count = 0;

        if let Ok(mut entries) = tokio::fs::read_dir(&self.watch_root).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if count >= 8 { break; } // Muestra de 8 archivos max
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "rs") {
                    if let Ok(content) = tokio::fs::read_to_string(path).await {
                        context.push_str(&content);
                        context.push('\n');
                        count += 1;
                    }
                }
            }
        }

        if let Ok(mut cache) = self.repo_context_cache.write() {
            *cache = (SystemTime::now(), context.clone());
        }
        
        context
    }

    async fn handle_file_event(&mut self, event: EditEvent) {
        if event.kind == super::file_watcher::EditKind::Delete {
            return;
        }

        let full_path = self.watch_root.join(&event.rel_path);
        
        // [LEAN] Solo leemos archivos pequeños o fragmentos para evitar lag de IO
        if let Ok(content) = tokio::fs::read_to_string(&full_path).await {
            let compression_ratio = calculate_compression_ratio(&content);
            
            // Novedad real contra el contexto del repositorio (Caché v5.2)
            let repo_context = self.get_repo_context_cached().await;
            let ncd_novelty = if !repo_context.is_empty() {
                calculate_ncd_against_context(&content, &repo_context)
            } else {
                compression_ratio // Fallback si el repo está vacío
            };

            // Detección de Copy-Paste / Boilerplate masivo con Threshold Variable
            let paste_threshold = match full_path.extension().and_then(|e| e.to_str()) {
                Some("rs") => 0.15,   // Rust: estricto
                Some("toml") => 0.10, // Config: muy estricto
                Some("md") => 0.25,   // Docs: más permisivo
                _ => 0.15,
            };

            if ncd_novelty < paste_threshold {
                warn!(
                    "⚠️  PASTE DETECTED: {:?} novelty score {:.2} (Target Threshold: {:.2})",
                    event.rel_path.file_name().unwrap_or_default(),
                    ncd_novelty,
                    paste_threshold
                );
            }

            if let Ok(mut latest) = self.latest_ncd.write() {
                // Promedio móvil exponencial (EMA) usando Novedad (Novelty)
                *latest = (*latest * 0.8) + (ncd_novelty * 0.2);
            }

            let entropic_cost = estimate_entropic_cost(&content, Some(&full_path));
            
            // Obtenemos la entropía motora actual (usamos velocity_entropy como proxy)
            let motor_entropy = self
                .latest_metrics
                .read()
                .ok()
                .and_then(|m| m.as_ref().map(|metrics| (metrics.velocity_entropy / 8.0).min(1.0)))
                .unwrap_or(0.0);

            // [TERMODINÁMICA] APLICAMOS DIFICULTAD (min_entropy)
            // Default 2.5 -> Factor 1.0. Higher min_entropy -> Higher cost.
            let difficulty_factor = self.min_entropy / 2.5;
            let adjusted_cost = entropic_cost * difficulty_factor;

            let has_energy = if let Ok(mut batt) = self.battery.write() {
                batt.consume(adjusted_cost)
            } else {
                false
            };

            let coupling = calculate_coupling_score(entropic_cost / 100.0, motor_entropy);
            
            if let Ok(mut latest) = self.latest_coupling.write() {
                *latest = (*latest * 0.7) + (coupling * 0.3); // Suavizado exponencial
            }

            if has_energy {
                if let Ok(mut tracker) = self.focus_tracker.write() {
                    tracker.mark_as_productive(full_path.clone());
                }
                info!(
                    "File change validated (Energy Balance): {:?} | Cost: {:.2} | Coupling: {:.2}",
                    event.rel_path.file_name().unwrap_or_default(),
                    entropic_cost,
                    coupling
                );
            } else {
                warn!(
                    "THERMODYNAMIC ANOMALY: Code injected without enough energy! {:?} | Cost: {:.2} | Battery: LOW",
                    event.rel_path.file_name().unwrap_or_default(),
                    entropic_cost
                );
            }
        }
    }

    fn run_analysis(&mut self) {
        match self.mouse_sentinel.analyze() {
            Ok(metrics) => {
                let events = self.events_captured.read().ok().map(|g| *g).unwrap_or(0);

                // [TERMODINÁMICA] Cargamos la batería con el esfuerzo detectado Y validación causal
                if let Ok(mut batt) = self.battery.write() {
                    // Carga v1.0 (Kinética)
                    let k_hits = self.keyboard_hits.swap(0, Ordering::SeqCst);
                    batt.charge(metrics.velocity_entropy / 8.0, self.analysis_interval, events, k_hits as usize);

                    // Carga v2.0 (Deep Work / Focus)
                    if let Ok(tracker) = self.focus_tracker.read() {
                        let tracker_metrics = tracker.get_metrics();
                        // Pasamos Duración aproximada de foco en este intervalo o el acumulado?
                        // La batería v2 maneja acumulados incrementales.
                        // Para simplificar, cargamos según minutos de foco detectados en este tick.
                        // Pero focus_tracker.get_metrics() es acumulativo.
                        // Necesitamos calcular el delta de foco.
                        
                        // NOTA: Para v2.0, AttentionBattery::charge_focus debería recibir 
                        // métricas incrementales o el tracker debería proveerlas.
                        // Implementaremos charge_focus con el acumulado total del tracker 
                        // pero la batería solo sube hasta capacity.
                        batt.charge_focus(
                            Duration::from_secs_f64(tracker_metrics.total_focus_mins * 60.0),
                            tracker_metrics.edit_burst_count,
                            tracker_metrics.navigation_events
                        );
                    }
                }

                if let Ok(mut latest) = self.latest_metrics.write() {
                     *latest = Some(metrics.clone());
                }

                let code_ncd = self.latest_ncd.read().map(|v| *v).unwrap_or(0.2);

                // Calcular Score Humano para el historial
                // Usamos una aproximación basada en métricas actuales para el sparkline
                let h_score = crate::stats::calculate_human_score(
                    metrics.burstiness,
                    code_ncd,
                    0.0, // Focus handled separately in battery
                    events,
                    false
                );
                
                if let Ok(mut history) = self.score_history.write() {
                    history.push_back(h_score);
                    if history.len() > 50 {
                        history.pop_front();
                    }
                }
            }
            Err(e) => {
                warn!("Mouse analysis failed: {}", e);
            }
        }
    }
}

/// GitMonitor error types.
#[derive(Debug, Error)]
pub enum GitMonitorError {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Monitor error: {0}")]
    Monitor(String),
}
