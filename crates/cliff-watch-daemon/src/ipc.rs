use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;
use tracing::{info, error};
use anyhow::Result;
use std::path::Path;
use std::fs;

use cliff_watch_core::protocol::{Request, Response};
use cliff_watch_core::mouse_sentinel::KinematicMetrics;
use cliff_watch_core::monitor::AttentionBattery;
use cliff_watch_core::stats::calculate_human_score;
use cliff_watch_core::focus_session::FocusTracker;
use cliff_watch_core::git::WitnessData;
use cliff_watch_core::crypto::zkp::HumanityProof;

pub struct IpcServer {
    socket_path: String,
    metrics: Arc<RwLock<Option<KinematicMetrics>>>,
    coupling_ref: Arc<RwLock<f64>>,
    battery_ref: Arc<RwLock<AttentionBattery>>,
    focus_tracker: Arc<RwLock<FocusTracker>>,
    events_captured: Arc<RwLock<usize>>,
    score_history_ref: Arc<RwLock<VecDeque<f64>>>,
    ncd_ref: Arc<RwLock<f64>>,
    shutdown: CancellationToken,
    start_time: std::time::Instant,
    signing_key: Arc<cliff_watch_core::crypto::SigningKey>,
    min_entropy: f64,
    audit_mode: bool,
}

impl IpcServer {
    pub fn new(
        socket_path: String,
        metrics: Arc<RwLock<Option<KinematicMetrics>>>,
        coupling_ref: Arc<RwLock<f64>>,
        battery_ref: Arc<RwLock<AttentionBattery>>,
        focus_tracker: Arc<RwLock<FocusTracker>>,
        events_captured: Arc<RwLock<usize>>,
        score_history_ref: Arc<RwLock<VecDeque<f64>>>,
        ncd_ref: Arc<RwLock<f64>>,
        shutdown: CancellationToken,
        signing_key: cliff_watch_core::crypto::SigningKey,
        min_entropy: f64,
        audit_mode: bool,
    ) -> Self {
        let verifying_key = signing_key.verifying_key();
        let pubkey_hex = cliff_watch_core::crypto::bytes_to_hex(verifying_key.as_bytes());
        info!("Daemon started with Public Key: {}", pubkey_hex);

        Self {
            socket_path,
            metrics,
            coupling_ref,
            battery_ref,
            focus_tracker,
            events_captured,
            score_history_ref,
            ncd_ref,
            shutdown,
            start_time: std::time::Instant::now(),
            signing_key: Arc::new(signing_key),
            min_entropy,
            audit_mode,
        }
    }

    pub async fn start(self) -> Result<()> {
        if Path::new(&self.socket_path).exists() {
            let _ = fs::remove_file(&self.socket_path);
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        info!("IPC Server listening on {}", self.socket_path);

        loop {
            tokio::select! {
                _ = self.shutdown.cancelled() => {
                    info!("IPC Server shutting down");
                    break;
                }
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((mut stream, _)) => {
                            let metrics_lock = self.metrics.clone();
                            let coupling_lock = self.coupling_ref.clone();
                            let battery_lock = self.battery_ref.clone();
                            let focus_tracker_lock = self.focus_tracker.clone();
                            let events_captured_lock = self.events_captured.clone();
                            let score_history_lock = self.score_history_ref.clone();
                            let ncd_lock = self.ncd_ref.clone();
                            let start_time = self.start_time;
                            let signing_key_lock = self.signing_key.clone();
                            let difficulty_factor = self.min_entropy / 2.5;
                            
                            tokio::spawn(async move {
                                let mut buffer = vec![0; 1024];
                                let n = match stream.read(&mut buffer).await {
                                    Ok(n) if n > 0 => n,
                                    _ => return,
                                };

                                let request_res: Result<Request, _> = serde_json::from_slice(&buffer[..n]);
                                let response = match request_res {
                                    Ok(Request::GetStatus) => {
                                        let events = events_captured_lock.read().map(|g| *g).unwrap_or(0);
                                        Response::Status {
                                            is_running: true,
                                            uptime_secs: start_time.elapsed().as_secs(),
                                            events_captured: events,
                                        }
                                    }
                                    Ok(Request::GetMetrics) => {
                                        let (focus_time_mins, edit_bursts, nav_events, is_synthetic_focus, is_focused) = if let Ok(ft) = focus_tracker_lock.read() {
                                            let m = ft.get_metrics();
                                            (m.total_focus_mins, m.edit_burst_count, m.navigation_events, m.is_synthetic, ft.is_focused())
                                        } else {
                                            (0.0, 0, 0, false, false)
                                        };

                                        let coupling = coupling_lock.read().map(|g| *g).unwrap_or(1.0);
                                        let battery_level = battery_lock.read().map(|g| g.level).unwrap_or(0.0);
                                        
                                        let history = if let Ok(h) = score_history_lock.read() {
                                            h.iter().cloned().collect()
                                        } else {
                                            Vec::new()
                                        };

                                        // LEER CODE NCD (Real Zstd Compression)
                                        let code_ncd = ncd_lock.read().map(|v| *v).unwrap_or(0.2);

                                        if let Ok(m_guard) = metrics_lock.read() {
                                            let (burstiness, _kinematic_ncd, is_synthetic_kinematic) = if let Some(m) = m_guard.as_ref() {
                                                (m.burstiness, m.ncd, m.is_synthetic)
                                            } else {
                                                (0.0, 0.5, false) // Valores base
                                            };

                                            let is_synthetic = is_synthetic_focus || is_synthetic_kinematic;
                                            
                                            // Usamos code_ncd para el Human Score (Anti-Paste)
                                            let human_score = calculate_human_score(burstiness, code_ncd, focus_time_mins, nav_events, is_synthetic);
                                            
                                            // Generar ZKP si el score es humano (>= threshold)
                                            let zkp_proof = if human_score >= 0.5 {
                                                let score_u64 = (human_score * 100.0) as u64;
                                                HumanityProof::generate(score_u64, 50).ok()
                                            } else {
                                                None
                                            };

                                            if let Some(m) = m_guard.as_ref() {
                                                Response::Metrics {
                                                    ldlj: m.ldlj,
                                                    entropy: m.velocity_entropy,
                                                    throughput: m.throughput,
                                                    human_score,
                                                    coupling,
                                                    battery_level,
                                                    focus_time_mins,
                                                    edit_bursts,
                                                    is_focused,
                                                    zkp_proof: zkp_proof.map(|_| "ZKP_ACTIVE_B64_PLACEHOLDER".to_string()),
                                                    score_history: history,
                                                }
                                            } else {
                                                Response::Metrics {
                                                    ldlj: 0.0,
                                                    entropy: 0.0,
                                                    throughput: 0.0,
                                                    human_score,
                                                    coupling,
                                                    battery_level,
                                                    focus_time_mins,
                                                    edit_bursts,
                                                    is_focused,
                                                    zkp_proof: zkp_proof.map(|_| "ZKP_ACTIVE_B64_PLACEHOLDER".to_string()),
                                                    score_history: history,
                                                }
                                            }
                                        } else {
                                            Response::Error("Failed to lock metrics".to_string())
                                        }
                                    }
                                    Ok(Request::GetTicket { cost }) => {
                                        let mut battery = battery_lock.write().map_err(|_| "Lock failed").unwrap();
                                        // APLICAR DIFICULTAD
                                        let adjusted_cost = cost * difficulty_factor;
                                        
                                        if battery.consume(adjusted_cost) {
                                            let message = format!("VALID:cost={:.2}:ts={}", cost, start_time.elapsed().as_secs());
                                            let signature = cliff_watch_core::crypto::sign_data(&signing_key_lock, message.as_bytes()).ok();
                                            Response::Ticket {
                                                success: true,
                                                signature,
                                                message: "Ticket issued. Thermodynamic balance verified.".to_string(),
                                            }
                                        } else if self.audit_mode {
                                            // AUDIT MODE: Permitir commit pero con advertencia (Soft Enforcement)
                                            let message = format!("AUDIT:cost={:.2}:ts={}", cost, start_time.elapsed().as_secs());
                                            let signature = cliff_watch_core::crypto::sign_data(&signing_key_lock, message.as_bytes()).ok();
                                            Response::Ticket {
                                                success: true,
                                                signature,
                                                message: format!(
                                                    "AUDIT WARNING: Battery at {:.2}. Threshold reached due to Audit Mode. Please focus more next time!",
                                                    battery.level
                                                ),
                                            }
                                        } else {
                                            Response::Ticket {
                                                success: false,
                                                signature: None,
                                                message: format!(
                                                    "THERMODYNAMIC FAILURE: Required {:.2} (difficulty factor {:.2}), Battery at {:.2}. Focus more!",
                                                    adjusted_cost, difficulty_factor, battery.level
                                                ),
                                            }
                                        }
                                    }

                                    Ok(Request::GetWitness { reset }) => {
                                        if let Ok(mut tracker) = focus_tracker_lock.write() {
                                            let metrics = tracker.get_metrics();
                                            let witness = WitnessData::from_metrics(&metrics);
                                            let data = witness.to_json();
                                            
                                            if reset {
                                                tracker.reset();
                                                info!("FocusTracker reset after GetWitness");
                                            }
                                            
                                            Response::Witness { data }
                                        } else {
                                            Response::Error("Failed to lock FocusTracker".to_string())
                                        }
                                    }

                                    Ok(Request::Ping) => Response::Pong,
                                    Err(e) => Response::Error(format!("Invalid request: {}", e)),
                                };

                                let response_json = serde_json::to_vec(&response).unwrap_or_default();
                                let _ = stream.write_all(&response_json).await;
                            });
                        }
                        Err(e) => error!("Failed to accept IPC connection: {}", e),
                    }
                }
            }
        }

        let _ = fs::remove_file(&self.socket_path);
        Ok(())
    }
}
