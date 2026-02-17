//! Sesión de Foco para Cliff-Watch v2.0
//!
//! Rastrea el tiempo de foco activo por archivo o workspace, permitiendo
//! medir "Deep Work" sin requerir movimiento físico constante.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};

/// Métricas de foco acumuladas para certificación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusMetrics {
    /// Minutos totales de foco activo en la sesión
    pub total_focus_mins: f64,
    /// Cantidad total de ráfagas de edición
    pub edit_burst_count: usize,
    /// Cantidad de caracteres editados (neto: inserciones - eliminaciones)
    pub chars_edited_net: i64,
    /// Cantidad de archivos únicos tocados
    pub unique_files: usize,
    /// Eventos de navegación (scroll, goto definition, etc.)
    pub navigation_events: usize,
    /// Flag de detección de actividad sintética en navegación
    pub is_synthetic: bool,
}

impl Default for FocusMetrics {
    fn default() -> Self {
        Self {
            total_focus_mins: 0.0,
            edit_burst_count: 0,
            chars_edited_net: 0,
            unique_files: 0,
            navigation_events: 0,
            is_synthetic: false,
        }
    }
}

/// Una sesión de foco activo en un archivo específico
#[derive(Debug, Clone)]
pub struct FocusSession {
    /// Ruta del archivo en foco (None si es focus general del workspace)
    pub file_path: Option<PathBuf>,
    /// Momento en que comenzó el foco
    pub started_at: SystemTime,
    /// Cantidad de ráfagas de edición durante esta sesión
    pub edit_bursts: usize,
    /// Total de caracteres modificados (puede ser negativo por eliminaciones)
    pub chars_delta: i64,
    /// Eventos de navegación durante esta sesión
    pub nav_events: usize,
}

impl FocusSession {
    /// Crea una nueva sesión de foco
    pub fn new(file_path: Option<PathBuf>) -> Self {
        Self {
            file_path,
            started_at: SystemTime::now(),
            edit_bursts: 0,
            chars_delta: 0,
            nav_events: 0,
        }
    }

    /// Registra una ráfaga de edición
    pub fn record_edit(&mut self, chars: i64) {
        self.edit_bursts += 1;
        self.chars_delta += chars;
    }

    /// Registra un evento de navegación
    pub fn record_navigation(&mut self) {
        self.nav_events += 1;
    }

    /// Calcula la duración de la sesión hasta ahora
    pub fn duration(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.started_at)
            .unwrap_or(Duration::ZERO)
    }

    /// Calcula los minutos de foco
    pub fn focus_minutes(&self) -> f64 {
        self.duration().as_secs_f64() / 60.0
    }
}

/// Tracker de sesiones de foco
/// 
/// Mantiene un registro de la sesión activa y el historial agregado
/// para poder calcular las métricas de certificación.
#[derive(Debug)]
pub struct FocusTracker {
    /// Sesión de foco activa (None si el IDE no tiene foco)
    current_session: Option<FocusSession>,
    /// Archivos únicos tocados en la sesión global
    unique_files: HashMap<PathBuf, ()>,
    /// Métricas acumuladas (globales, se filtrarán al exportar)
    cumulative: FocusMetrics,
    /// Timestamp del último heartbeat recibido
    last_heartbeat: Option<SystemTime>,
    /// Últimos timestamps de navegación para análisis de síntesis
    nav_timestamps: Vec<f64>,
    /// Minutos acumulados POR ARCHIVO
    file_focus_accum: HashMap<PathBuf, f64>,
    /// Eventos de navegación POR ARCHIVO
    file_nav_accum: HashMap<PathBuf, usize>,
    /// Archivos que han tenido cambios VALIDADOS (productivos)
    productive_files: HashSet<PathBuf>,
}

impl FocusTracker {
    /// Crea un nuevo tracker de foco
    pub fn new() -> Self {
        Self {
            current_session: None,
            unique_files: HashMap::new(),
            cumulative: FocusMetrics::default(),
            last_heartbeat: None,
            nav_timestamps: Vec::with_capacity(100),
            file_focus_accum: HashMap::new(),
            file_nav_accum: HashMap::new(),
            productive_files: HashSet::new(),
        }
    }

    /// Registra que el IDE ganó foco
    pub fn focus_gained(&mut self, file_path: Option<PathBuf>) {
        // Finalizar sesión anterior si existe
        self.finalize_current_session();

        // Registrar archivo único
        if let Some(ref path) = file_path {
            self.unique_files.insert(path.clone(), ());
        }

        // Iniciar nueva sesión
        self.current_session = Some(FocusSession::new(file_path));
    }

    /// Registra que el IDE perdió foco
    pub fn focus_lost(&mut self) {
        self.finalize_current_session();
    }

    /// Registra una ráfaga de edición
    pub fn edit_burst(&mut self, file_path: &str, chars_delta: i64) {
        let path = PathBuf::from(file_path);
        self.unique_files.insert(path.clone(), ());

        if let Some(ref mut session) = self.current_session {
            session.record_edit(chars_delta);
        } else {
            // Edición sin sesión activa - crear sesión temporal
            let mut session = FocusSession::new(Some(path));
            session.record_edit(chars_delta);
            self.current_session = Some(session);
        }

        self.cumulative.edit_burst_count += 1;
        self.cumulative.chars_edited_net += chars_delta;
    }

    /// Registra un evento de navegación
    pub fn navigation(&mut self, file_path: &str, timestamp_ms: u64) {
        let path = PathBuf::from(file_path);
        self.unique_files.insert(path.clone(), ());

        if let Some(ref mut session) = self.current_session {
            session.record_navigation();
        }

        // Registrar navegación por archivo
        let nav_count = self.file_nav_accum.entry(path).or_insert(0);
        *nav_count += 1;

        self.cumulative.navigation_events += 1;
        
        // Registrar timestamp para análisis cinemático de navegación
        self.nav_timestamps.push(timestamp_ms as f64);
        if self.nav_timestamps.len() > 100 {
            self.nav_timestamps.remove(0);
        }
    }

    /// Registra un heartbeat
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = Some(SystemTime::now());
    }

    /// Finaliza la sesión actual y acumula sus métricas
    fn finalize_current_session(&mut self) {
        if let Some(session) = self.current_session.take() {
            let mins = session.focus_minutes();
            if let Some(path) = &session.file_path {
                let accum = self.file_focus_accum.entry(path.clone()).or_insert(0.0);
                *accum += mins;
            }
            self.cumulative.total_focus_mins += mins;
        }
    }

    /// Marca un archivo como productivo (el foco en él contará para el score)
    pub fn mark_as_productive(&mut self, path: PathBuf) {
        self.productive_files.insert(path);
    }

    /// Obtiene las métricas actuales (FILTRADAS por productividad)
    pub fn get_metrics(&self) -> FocusMetrics {
        let mut metrics = FocusMetrics { unique_files: self.productive_files.len(), ..Default::default() };

        // Calcular foco acumulado SOLO en archivos productivos
        for (path, mins) in &self.file_focus_accum {
            if self.productive_files.contains(path) {
                metrics.total_focus_mins += mins;
            }
        }

        // Sumar tiempo de sesión actual si el archivo es productivo
        if let Some(ref session) = self.current_session {
            if let Some(path) = &session.file_path {
                if self.productive_files.contains(path) {
                    metrics.total_focus_mins += session.focus_minutes();
                }
            }
        }

        // Calcular eventos de navegación SOLO en archivos productivos
        for (path, nav) in &self.file_nav_accum {
            if self.productive_files.contains(path) {
                metrics.navigation_events += nav;
            }
        }

        // Pasar otros datos globales
        metrics.edit_burst_count = self.cumulative.edit_burst_count;
        metrics.chars_edited_net = self.cumulative.chars_edited_net;

        // Detectar si el patrón de navegación es sintético (bot de scroll)
        if crate::stats::is_synthetic_pattern(&self.nav_timestamps) {
            metrics.is_synthetic = true;
        }

        metrics
    }

    /// Indica si hay una sesión de foco activa
    pub fn is_focused(&self) -> bool {
        self.current_session.is_some()
    }

    /// Indica si el daemon está recibiendo heartbeats recientes
    pub fn is_alive(&self) -> bool {
        self.last_heartbeat
            .map(|t| t.elapsed().unwrap_or(Duration::MAX) < Duration::from_secs(60))
            .unwrap_or(false)
    }

    /// Resetea el tracker (para nueva sesión de trabajo)
    pub fn reset(&mut self) {
        self.current_session = None;
        self.unique_files.clear();
        self.cumulative = FocusMetrics::default();
        self.last_heartbeat = None;
    }
}

impl Default for FocusTracker {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_focus_session_duration() {
        let session = FocusSession::new(Some(PathBuf::from("/test.rs")));
        sleep(Duration::from_millis(100));
        let duration = session.duration();
        assert!(duration >= Duration::from_millis(100));
    }

    #[test]
    fn test_focus_tracker_basic_flow() {
        let mut tracker = FocusTracker::new();
        
        // Ganar foco en un archivo
        tracker.focus_gained(Some(PathBuf::from("/src/main.rs")));
        assert!(tracker.is_focused());
        
        // Hacer algunas ediciones
        tracker.edit_burst("/src/main.rs", 50);
        tracker.edit_burst("/src/main.rs", -10);
        
        // Marcar como productivo para que no se filtre en get_metrics
        tracker.mark_as_productive(PathBuf::from("/src/main.rs"));

        // Verificar métricas
        let metrics = tracker.get_metrics();
        assert_eq!(metrics.edit_burst_count, 2);
        assert_eq!(metrics.chars_edited_net, 40);
        assert_eq!(metrics.unique_files, 1);
    }

    #[test]
    fn test_focus_tracker_multiple_files() {
        let mut tracker = FocusTracker::new();
        
        tracker.focus_gained(Some(PathBuf::from("/a.rs")));
        tracker.edit_burst("/a.rs", 10);
        
        tracker.focus_gained(Some(PathBuf::from("/b.rs")));
        tracker.edit_burst("/b.rs", 20);
        tracker.edit_burst("/c.rs", 5); // Archivo diferente
        
        tracker.mark_as_productive(PathBuf::from("/a.rs"));
        tracker.mark_as_productive(PathBuf::from("/b.rs"));
        tracker.mark_as_productive(PathBuf::from("/c.rs"));
        
        let metrics = tracker.get_metrics();
        assert_eq!(metrics.unique_files, 3);
    }

    #[test]
    fn test_focus_lost_accumulates() {
        let mut tracker = FocusTracker::new();
        
        tracker.focus_gained(Some(PathBuf::from("/test.rs")));
        sleep(Duration::from_millis(100));
        tracker.focus_lost();

        tracker.mark_as_productive(PathBuf::from("/test.rs"));

        let metrics = tracker.get_metrics();
        assert!(metrics.total_focus_mins > 0.0);
        assert!(!tracker.is_focused());
    }

    #[test]
    fn test_heartbeat_tracking() {
        let mut tracker = FocusTracker::new();
        
        assert!(!tracker.is_alive());
        tracker.heartbeat();
        assert!(tracker.is_alive());
    }
}
